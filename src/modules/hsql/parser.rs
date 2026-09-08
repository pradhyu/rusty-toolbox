use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TableDef {
    pub name: String,
    pub schema: String,
    pub is_memory: bool,
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HsqlDatabase {
    pub name: String,
    pub path: PathBuf,
    pub properties: HashMap<String, String>,
    pub tables: HashMap<String, TableDef>,
    pub users: Vec<String>,
    pub schemas: Vec<String>,
    pub has_data_file: bool,
    pub has_log_file: bool,
    pub has_script_file: bool,
}

impl HsqlDatabase {
    pub fn load_from_path<P: AsRef<Path>>(input_path: P) -> Result<Self, String> {
        let path = input_path.as_ref();
        
        let (db_dir, db_basename) = if path.is_dir() {
            // Directory provided: look for .script or .properties files
            let mut found_name: Option<String> = None;
            for entry in fs::read_dir(path).map_err(|e| e.to_string())?.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ext == "script" || ext == "properties" {
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            found_name = Some(stem.to_string());
                            break;
                        }
                    }
                }
            }
            let base = found_name.unwrap_or_else(|| "test".to_string());
            (path.to_path_buf(), base)
        } else {
            // Specific file provided (e.g. /path/to/mydb.script)
            let parent = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("test").to_string();
            (parent, stem)
        };

        let script_file = db_dir.join(format!("{}.script", db_basename));
        let props_file = db_dir.join(format!("{}.properties", db_basename));
        let data_file = db_dir.join(format!("{}.data", db_basename));
        let log_file = db_dir.join(format!("{}.log", db_basename));

        let has_script = script_file.exists();
        let has_props = props_file.exists();
        let has_data = data_file.exists();
        let has_log = log_file.exists();

        if !has_script && !has_props && !has_data {
            return Err(format!(
                "No HSQLDB database components found at '{}' (looked for {}.script / {}.properties)",
                path.display(),
                db_basename,
                db_basename
            ));
        }

        // Parse properties
        let mut properties = HashMap::new();
        if has_props {
            if let Ok(content) = fs::read_to_string(&props_file) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some(eq) = trimmed.find('=') {
                        let k = trimmed[..eq].trim().to_string();
                        let v = trimmed[eq + 1..].trim().to_string();
                        properties.insert(k, v);
                    }
                }
            }
        }

        let mut db = HsqlDatabase {
            name: db_basename,
            path: db_dir,
            properties,
            tables: HashMap::new(),
            users: Vec::new(),
            schemas: vec!["PUBLIC".to_string()],
            has_data_file: has_data,
            has_log_file: has_log,
            has_script_file: has_script,
        };

        // Parse .script file if available
        if has_script {
            if let Ok(content) = fs::read_to_string(&script_file) {
                db.parse_script(&content)?;
            }
        }

        Ok(db)
    }

    fn parse_script(&mut self, script: &str) -> Result<(), String> {
        let mut statements = Vec::new();
        let mut current_stmt = String::new();

        for line in script.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            current_stmt.push_str(trimmed);
            current_stmt.push(' ');
            if trimmed.ends_with(';') || (!trimmed.starts_with("/*") && !trimmed.contains("*/")) {
                // Statements in HSQL script usually newline-terminated or semicolon-delimited
                statements.push(current_stmt.trim().to_string());
                current_stmt.clear();
            }
        }
        if !current_stmt.is_empty() {
            statements.push(current_stmt.trim().to_string());
        }

        for stmt in statements {
            let upper = stmt.to_uppercase();
            if upper.starts_with("CREATE MEMORY TABLE") || upper.starts_with("CREATE TABLE") || upper.starts_with("CREATE CACHED TABLE") {
                self.parse_create_table(&stmt);
            } else if upper.starts_with("CREATE SCHEMA") {
                let parts: Vec<&str> = stmt.split_whitespace().collect();
                if parts.len() >= 3 {
                    self.schemas.push(parts[2].trim_end_matches(';').to_string());
                }
            } else if upper.starts_with("CREATE USER") {
                let parts: Vec<&str> = stmt.split_whitespace().collect();
                if parts.len() >= 3 {
                    self.users.push(parts[2].trim_end_matches(';').to_string());
                }
            } else if upper.starts_with("INSERT INTO") {
                self.parse_insert(&stmt);
            }
        }

        Ok(())
    }

    fn parse_create_table(&mut self, stmt: &str) {
        // Example: CREATE MEMORY TABLE PUBLIC.USERS(ID INTEGER GENERATED BY DEFAULT AS IDENTITY NOT NULL PRIMARY KEY, NAME VARCHAR(100), EMAIL VARCHAR(255))
        let upper = stmt.to_uppercase();
        let is_memory = upper.contains("MEMORY TABLE");

        if let Some(paren_start) = stmt.find('(') {
            if let Some(paren_end) = stmt.rfind(')') {
                let header = stmt[..paren_start].trim();
                let columns_part = &stmt[paren_start + 1..paren_end];

                let header_parts: Vec<&str> = header.split_whitespace().collect();
                if let Some(tbl_full_name) = header_parts.last() {
                    let (schema, tbl_name) = if let Some(dot) = tbl_full_name.find('.') {
                        (tbl_full_name[..dot].to_string(), tbl_full_name[dot + 1..].to_string())
                    } else {
                        ("PUBLIC".to_string(), tbl_full_name.to_string())
                    };

                    let mut columns = Vec::new();
                    // Split columns safely
                    for col_str in split_comma_respecting_parens(columns_part) {
                        let col_trimmed = col_str.trim();
                        if col_trimmed.is_empty() || col_trimmed.to_uppercase().starts_with("CONSTRAINT") || col_trimmed.to_uppercase().starts_with("PRIMARY KEY") {
                            continue;
                        }

                        let parts: Vec<&str> = col_trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name = parts[0].to_string();
                            let col_type = parts[1].to_string();
                            let is_pk = col_trimmed.to_uppercase().contains("PRIMARY KEY");
                            let is_nullable = !col_trimmed.to_uppercase().contains("NOT NULL");

                            columns.push(ColumnDef {
                                name,
                                col_type,
                                is_nullable,
                                is_primary_key: is_pk,
                            });
                        }
                    }

                    self.tables.insert(
                        tbl_name.clone(),
                        TableDef {
                            name: tbl_name,
                            schema,
                            is_memory,
                            columns,
                            rows: Vec::new(),
                        },
                    );
                }
            }
        }
    }

    fn parse_insert(&mut self, stmt: &str) {
        // Example: INSERT INTO PUBLIC.USERS VALUES(1, 'Alice', 'alice@example.com')
        let upper = stmt.to_uppercase();
        if let Some(val_idx) = upper.find("VALUES") {
            let target_tbl_part = stmt[11..val_idx].trim();
            let tbl_name = if let Some(dot) = target_tbl_part.find('.') {
                target_tbl_part[dot + 1..].trim().to_string()
            } else {
                target_tbl_part.to_string()
            };

            let values_part = stmt[val_idx + 6..].trim();
            if let (Some(start), Some(end)) = (values_part.find('('), values_part.rfind(')')) {
                let inner = &values_part[start + 1..end];
                let row_values = split_comma_respecting_quotes(inner);

                if let Some(table) = self.tables.get_mut(&tbl_name) {
                    table.rows.push(row_values);
                }
            }
        }
    }
}

fn split_comma_respecting_parens(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut current = String::new();

    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn split_comma_respecting_quotes(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();

    for c in s.chars() {
        match c {
            '\'' => {
                in_quote = !in_quote;
                current.push(c);
            }
            ',' if !in_quote => {
                parts.push(current.trim().trim_matches('\'').to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().trim_matches('\'').to_string());
    }
    parts
}
