use calamine::{open_workbook_auto, Data, Reader};
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SheetData {
    pub name: String,
    pub schema_name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExcelWorkbook {
    pub path: PathBuf,
    pub schema_name: String,
    pub sheets: HashMap<String, SheetData>,
    pub sheet_names: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ExcelCatalog {
    pub root_path: PathBuf,
    pub is_directory: bool,
    pub workbooks: Vec<ExcelWorkbook>,
}

impl ExcelWorkbook {
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let path = file_path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("File '{}' not found", path.display()));
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("sheet")
            .to_string();
        
        // Clean schema name (alphanumeric + underscore)
        let schema_name: String = stem
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect();

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if ext == "csv" || ext == "tsv" || ext == "txt" {
            let delimiter = if ext == "tsv" { b'\t' } else { b',' };
            return Self::load_from_csv(&path, &schema_name, delimiter);
        }

        let mut workbook = open_workbook_auto(&path).map_err(|e| format!("Failed to open spreadsheet '{}': {}", path.display(), e))?;
        let sheet_names = workbook.sheet_names().to_vec();

        if sheet_names.is_empty() {
            return Err(format!("No worksheets found in '{}'", path.display()));
        }

        let mut sheets = HashMap::new();

        for name in &sheet_names {
            if let Ok(range) = workbook.worksheet_range(name) {
                let mut iter = range.rows();
                let first_row = iter.next();

                let mut columns = Vec::new();
                let mut data_rows = Vec::new();

                if let Some(header_row) = first_row {
                    for (col_idx, cell) in header_row.iter().enumerate() {
                        let header_val = format_cell_value(cell);
                        if header_val.trim().is_empty() {
                            columns.push(format!("col_{}", col_idx + 1));
                        } else {
                            columns.push(header_val.trim().to_string());
                        }
                    }

                    // Process remaining rows
                    for row in iter {
                        let mut row_vec = Vec::new();
                        let mut has_content = false;
                        for cell in row.iter() {
                            let val = format_cell_value(cell);
                            if !val.is_empty() {
                                has_content = true;
                            }
                            row_vec.push(val);
                        }
                        if has_content {
                            while row_vec.len() < columns.len() {
                                row_vec.push(String::new());
                            }
                            data_rows.push(row_vec);
                        }
                    }
                }

                sheets.insert(
                    name.clone(),
                    SheetData {
                        name: name.clone(),
                        schema_name: schema_name.clone(),
                        columns,
                        rows: data_rows,
                    },
                );
            }
        }

        Ok(ExcelWorkbook {
            path,
            schema_name,
            sheets,
            sheet_names,
        })
    }

    fn load_from_csv(path: &Path, schema_name: &str, delimiter: u8) -> Result<Self, String> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .delimiter(delimiter)
            .from_path(path)
            .map_err(|e| format!("Failed to read CSV '{}': {}", path.display(), e))?;

        let headers = reader.headers().map_err(|e| format!("CSV headers error in '{}': {}", path.display(), e))?;
        let columns: Vec<String> = headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let s = h.trim();
                if s.is_empty() {
                    format!("col_{}", i + 1)
                } else {
                    s.to_string()
                }
            })
            .collect();

        let mut data_rows = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| format!("CSV row error in '{}': {}", path.display(), e))?;
            let mut row_vec = Vec::new();
            for field in record.iter() {
                row_vec.push(field.to_string());
            }
            while row_vec.len() < columns.len() {
                row_vec.push(String::new());
            }
            data_rows.push(row_vec);
        }

        let sheet_name = schema_name.to_string();
        let mut sheets = HashMap::new();
        sheets.insert(
            sheet_name.clone(),
            SheetData {
                name: sheet_name.clone(),
                schema_name: schema_name.to_string(),
                columns,
                rows: data_rows,
            },
        );

        Ok(ExcelWorkbook {
            path: path.to_path_buf(),
            schema_name: schema_name.to_string(),
            sheets,
            sheet_names: vec![sheet_name],
        })
    }
}

impl ExcelCatalog {
    pub fn load_from_path<P: AsRef<Path>>(input_path: P) -> Result<Self, String> {
        let path = input_path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("Path '{}' does not exist", path.display()));
        }

        let mut workbooks = Vec::new();

        if path.is_dir() {
            let mut entries: Vec<PathBuf> = Vec::new();
            for entry in fs::read_dir(&path).map_err(|e| e.to_string())?.flatten() {
                let p = entry.path();
                if let Some(ext) = p.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
                    if matches!(ext.as_str(), "xlsx" | "xls" | "ods" | "xlsb" | "csv" | "tsv") {
                        entries.push(p);
                    }
                }
            }
            entries.sort();

            if entries.is_empty() {
                return Err(format!("No spreadsheet or CSV files found in directory '{}'", path.display()));
            }

            for file in entries {
                match ExcelWorkbook::load_from_file(&file) {
                    Ok(wb) => workbooks.push(wb),
                    Err(e) => eprintln!("Warning: Skipping file '{}': {}", file.display(), e),
                }
            }

            if workbooks.is_empty() {
                return Err(format!("Could not load any valid spreadsheets or CSV files from '{}'", path.display()));
            }

            Ok(ExcelCatalog {
                root_path: path,
                is_directory: true,
                workbooks,
            })
        } else {
            let wb = ExcelWorkbook::load_from_file(&path)?;
            workbooks.push(wb);

            Ok(ExcelCatalog {
                root_path: path,
                is_directory: false,
                workbooks,
            })
        }
    }

    pub fn create_in_memory_sqlite(&self) -> Result<Connection, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        let is_multi = self.workbooks.len() > 1;

        for wb in &self.workbooks {
            // Attach in-memory schema if multiple workbooks
            if is_multi {
                let attach_sql = format!("ATTACH DATABASE ':memory:' AS \"{}\";", wb.schema_name);
                let _ = conn.execute(&attach_sql, []);
            }

            for sheet in wb.sheets.values() {
                if sheet.columns.is_empty() {
                    continue;
                }

                let col_defs: Vec<String> = sheet
                    .columns
                    .iter()
                    .map(|col| format!("\"{}\" TEXT", col.replace('"', "\"\"")))
                    .collect();

                let sanitized_sheet_name = sheet.name.replace('"', "\"\"");

                // 1. Target table in attached schema (or main)
                let target_table = if is_multi {
                    format!("\"{}\".\"{}\"", wb.schema_name, sanitized_sheet_name)
                } else {
                    format!("\"{}\"", sanitized_sheet_name)
                };

                let create_sql = format!("CREATE TABLE IF NOT EXISTS {} ({});", target_table, col_defs.join(", "));
                conn.execute(&create_sql, []).map_err(|e| format!("Failed to create table {}: {}", target_table, e))?;

                for row in &sheet.rows {
                    let placeholders = vec!["?"; sheet.columns.len()].join(", ");
                    let insert_sql = format!("INSERT INTO {} VALUES ({});", target_table, placeholders);

                    let mut params_vec: Vec<String> = Vec::new();
                    for i in 0..sheet.columns.len() {
                        params_vec.push(row.get(i).cloned().unwrap_or_default());
                    }

                    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                    let _ = conn.execute(&insert_sql, params_refs.as_slice());
                }

                // 2. Convenience aliases in main database
                if is_multi {
                    // Create view `<schema>_<sheet>` in main schema
                    let alias_name = format!("{}_{}", wb.schema_name, sanitized_sheet_name);
                    let view_sql = format!("CREATE VIEW IF NOT EXISTS \"{}\" AS SELECT * FROM {};", alias_name, target_table);
                    let _ = conn.execute(&view_sql, []);

                    // Create view `<schema>` directly if schema == sheet_name (e.g. CSV files)
                    if wb.schema_name == sanitized_sheet_name {
                        let direct_view = format!("CREATE VIEW IF NOT EXISTS \"{}\" AS SELECT * FROM {};", wb.schema_name, target_table);
                        let _ = conn.execute(&direct_view, []);
                    }

                    // Create view `"<schema>.<sheet>"` in main schema for dot syntax fallback
                    let dot_alias = format!("{}.{}", wb.schema_name, sanitized_sheet_name);
                    let dot_view_sql = format!("CREATE VIEW IF NOT EXISTS \"{}\" AS SELECT * FROM {};", dot_alias, target_table);
                    let _ = conn.execute(&dot_view_sql, []);
                } else {
                    // Single file mode: also alias 'data' for quick generic queries
                    let data_view = format!("CREATE VIEW IF NOT EXISTS \"data\" AS SELECT * FROM {};", target_table);
                    let _ = conn.execute(&data_view, []);
                }
            }
        }

        Ok(conn)
    }

    pub fn export_to_sqlite<P: AsRef<Path>>(&self, out_path: P) -> Result<(), String> {
        let conn = Connection::open(out_path).map_err(|e| e.to_string())?;
        let is_multi = self.workbooks.len() > 1;

        for wb in &self.workbooks {
            for sheet in wb.sheets.values() {
                if sheet.columns.is_empty() {
                    continue;
                }

                let col_defs: Vec<String> = sheet
                    .columns
                    .iter()
                    .map(|col| format!("\"{}\" TEXT", col.replace('"', "\"\"")))
                    .collect();

                let sanitized_sheet_name = sheet.name.replace('"', "\"\"");
                let table_name = if is_multi {
                    if wb.schema_name == sanitized_sheet_name {
                        wb.schema_name.clone()
                    } else {
                        format!("{}_{}", wb.schema_name, sanitized_sheet_name)
                    }
                } else {
                    sanitized_sheet_name
                };

                let create_sql = format!("CREATE TABLE IF NOT EXISTS \"{}\" ({});", table_name, col_defs.join(", "));
                conn.execute(&create_sql, []).map_err(|e| e.to_string())?;

                for row in &sheet.rows {
                    let placeholders = vec!["?"; sheet.columns.len()].join(", ");
                    let insert_sql = format!("INSERT INTO \"{}\" VALUES ({});", table_name, placeholders);

                    let mut params_vec: Vec<String> = Vec::new();
                    for i in 0..sheet.columns.len() {
                        params_vec.push(row.get(i).cloned().unwrap_or_default());
                    }

                    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                    let _ = conn.execute(&insert_sql, params_refs.as_slice());
                }
            }
        }

        Ok(())
    }
}

fn format_cell_value(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{:.0}", f)
            } else {
                format!("{}", f)
            }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
        Data::DateTime(dt) => format!("{}", dt),
        Data::Error(e) => format!("#ERR:{:?}", e),
        _ => format!("{}", cell),
    }
}

pub fn query_to_table(conn: &Connection, sql: &str) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let col_names: Vec<String> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();

    let mut rows = Vec::new();
    let mut query_rows = stmt.query([]).map_err(|e| e.to_string())?;

    while let Some(row) = query_rows.next().map_err(|e| e.to_string())? {
        let mut row_vals = Vec::new();
        for i in 0..col_names.len() {
            let val_ref = row.get_ref(i).map_err(|e| e.to_string())?;
            let val_str = match val_ref {
                ValueRef::Null => "NULL".to_string(),
                ValueRef::Integer(n) => n.to_string(),
                ValueRef::Real(f) => f.to_string(),
                ValueRef::Text(t) => String::from_utf8_lossy(t).to_string(),
                ValueRef::Blob(b) => format!("<blob: {} bytes>", b.len()),
            };
            row_vals.push(val_str);
        }
        rows.push(row_vals);
    }

    Ok((col_names, rows))
}
