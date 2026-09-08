use calamine::{open_workbook_auto, Data, Reader};
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SheetData {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ExcelWorkbook {
    pub path: PathBuf,
    pub sheets: HashMap<String, SheetData>,
    pub sheet_names: Vec<String>,
}

impl ExcelWorkbook {
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let path = file_path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(format!("File '{}' not found", path.display()));
        }

        let mut workbook = open_workbook_auto(&path).map_err(|e| format!("Failed to open spreadsheet: {}", e))?;
        let sheet_names = workbook.sheet_names().to_vec();

        if sheet_names.is_empty() {
            return Err("No worksheets found in the workbook".to_string());
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
                        columns,
                        rows: data_rows,
                    },
                );
            }
        }

        Ok(ExcelWorkbook {
            path,
            sheets,
            sheet_names,
        })
    }

    pub fn create_in_memory_sqlite(&self) -> Result<Connection, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;

        for sheet in self.sheets.values() {
            if sheet.columns.is_empty() {
                continue;
            }

            let col_defs: Vec<String> = sheet
                .columns
                .iter()
                .map(|col| format!("\"{}\" TEXT", col.replace('"', "\"\"")))
                .collect();

            let sanitized_table_name = sheet.name.replace('"', "\"\"");
            let create_sql = format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" ({});",
                sanitized_table_name,
                col_defs.join(", ")
            );
            conn.execute(&create_sql, []).map_err(|e| format!("Failed to create table for sheet '{}': {}", sheet.name, e))?;

            for row in &sheet.rows {
                let placeholders = vec!["?"; sheet.columns.len()].join(", ");
                let insert_sql = format!("INSERT INTO \"{}\" VALUES ({});", sanitized_table_name, placeholders);

                let mut params_vec: Vec<String> = Vec::new();
                for i in 0..sheet.columns.len() {
                    params_vec.push(row.get(i).cloned().unwrap_or_default());
                }

                let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                let _ = conn.execute(&insert_sql, params_refs.as_slice());
            }
        }

        Ok(conn)
    }

    pub fn export_to_sqlite<P: AsRef<Path>>(&self, out_path: P) -> Result<(), String> {
        let conn = Connection::open(out_path).map_err(|e| e.to_string())?;

        for sheet in self.sheets.values() {
            if sheet.columns.is_empty() {
                continue;
            }

            let col_defs: Vec<String> = sheet
                .columns
                .iter()
                .map(|col| format!("\"{}\" TEXT", col.replace('"', "\"\"")))
                .collect();

            let sanitized_table_name = sheet.name.replace('"', "\"\"");
            let create_sql = format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" ({});",
                sanitized_table_name,
                col_defs.join(", ")
            );
            conn.execute(&create_sql, []).map_err(|e| e.to_string())?;

            for row in &sheet.rows {
                let placeholders = vec!["?"; sheet.columns.len()].join(", ");
                let insert_sql = format!("INSERT INTO \"{}\" VALUES ({});", sanitized_table_name, placeholders);

                let mut params_vec: Vec<String> = Vec::new();
                for i in 0..sheet.columns.len() {
                    params_vec.push(row.get(i).cloned().unwrap_or_default());
                }

                let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
                let _ = conn.execute(&insert_sql, params_refs.as_slice());
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
