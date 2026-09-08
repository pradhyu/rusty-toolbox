use crate::modules::hsql::parser::HsqlDatabase;
use rusqlite::types::ValueRef;
use rusqlite::Connection;
use std::path::Path;

pub struct SqliteBridge;

impl SqliteBridge {
    pub fn create_in_memory_from_hsql(db: &HsqlDatabase) -> Result<Connection, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;

        for table in db.tables.values() {
            let col_defs: Vec<String> = table
                .columns
                .iter()
                .map(|c| {
                    let sqlite_type = match c.col_type.to_uppercase().as_str() {
                        s if s.contains("INT") => "INTEGER",
                        s if s.contains("CHAR") || s.contains("TEXT") || s.contains("CLOB") => "TEXT",
                        s if s.contains("BOOL") => "INTEGER",
                        s if s.contains("DECIMAL") || s.contains("NUMERIC") || s.contains("DOUBLE") || s.contains("FLOAT") => "REAL",
                        s if s.contains("BLOB") || s.contains("BINARY") => "BLOB",
                        _ => "TEXT",
                    };
                    format!(
                        "\"{}\" {}{}{}",
                        c.name,
                        sqlite_type,
                        if c.is_primary_key { " PRIMARY KEY" } else { "" },
                        if !c.is_nullable && !c.is_primary_key { " NOT NULL" } else { "" }
                    )
                })
                .collect();

            let create_sql = format!("CREATE TABLE IF NOT EXISTS \"{}\" ({});", table.name, col_defs.join(", "));
            conn.execute(&create_sql, []).map_err(|e| format!("Failed to create table {}: {}", table.name, e))?;

            // Insert parsed memory records
            for row in &table.rows {
                if row.is_empty() {
                    continue;
                }
                let placeholders = vec!["?"; row.len()].join(", ");
                let insert_sql = format!("INSERT INTO \"{}\" VALUES ({});", table.name, placeholders);
                let params_vec: Vec<&dyn rusqlite::ToSql> = row.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

                let _ = conn.execute(&insert_sql, params_vec.as_slice());
            }
        }

        Ok(conn)
    }

    pub fn export_to_file<P: AsRef<Path>>(db: &HsqlDatabase, out_path: P) -> Result<(), String> {
        let conn = Connection::open(out_path).map_err(|e| e.to_string())?;
        
        for table in db.tables.values() {
            let col_defs: Vec<String> = table
                .columns
                .iter()
                .map(|c| {
                    format!("\"{}\" TEXT", c.name)
                })
                .collect();

            let create_sql = format!("CREATE TABLE IF NOT EXISTS \"{}\" ({});", table.name, col_defs.join(", "));
            conn.execute(&create_sql, []).map_err(|e| e.to_string())?;

            for row in &table.rows {
                if row.is_empty() {
                    continue;
                }
                let placeholders = vec!["?"; row.len()].join(", ");
                let insert_sql = format!("INSERT INTO \"{}\" VALUES ({});", table.name, placeholders);
                let params_vec: Vec<&dyn rusqlite::ToSql> = row.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

                let _ = conn.execute(&insert_sql, params_vec.as_slice());
            }
        }

        Ok(())
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
}
