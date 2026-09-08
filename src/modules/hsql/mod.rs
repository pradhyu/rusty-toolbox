pub mod parser;
pub mod sqlite_bridge;
pub mod tui;

use colored::*;
use std::path::PathBuf;

pub fn execute_hsql_command(
    path: PathBuf,
    query: Option<String>,
    interactive: bool,
    dump_sqlite: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = parser::HsqlDatabase::load_from_path(&path)
        .map_err(|e| format!("Failed to load HSQLDB: {}", e))?;

    // 1. Dump to SQLite mode
    if let Some(out_sqlite_path) = dump_sqlite {
        println!("{} Converting HSQLDB '{}' to SQLite '{}'...", "🔄".cyan(), db.name.bold(), out_sqlite_path.display());
        sqlite_bridge::SqliteBridge::export_to_file(&db, &out_sqlite_path)
            .map_err(|e| format!("Export error: {}", e))?;
        println!("{} Successfully exported {} tables to {}", "✔".green(), db.tables.len(), out_sqlite_path.display());
        return Ok(());
    }

    // 2. Interactive TUI mode
    if interactive || query.is_none() {
        println!("{} Launching HSQL interactive viewer for '{}'...", "🚀".cyan(), db.name.bold());
        return tui::run_hsql_tui(db);
    }

    // 3. Direct SQL Query execution
    if let Some(sql) = query {
        let conn = sqlite_bridge::SqliteBridge::create_in_memory_from_hsql(&db)
            .map_err(|e| format!("In-memory bridge error: {}", e))?;

        let (cols, rows) = sqlite_bridge::SqliteBridge::query_to_table(&conn, &sql)
            .map_err(|e| format!("Query execution error: {}", e))?;

        println!("{} Executing: {}", "🔍".cyan(), sql.bold().yellow());
        print_ascii_table(&cols, &rows);
        println!("({} rows returned)", rows.len().to_string().green());
    }

    Ok(())
}

fn print_ascii_table(cols: &[String], rows: &[Vec<String>]) {
    if cols.is_empty() {
        println!("(Empty result)");
        return;
    }

    let mut col_widths: Vec<usize> = cols.iter().map(|c| c.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Top border
    let border_parts: Vec<String> = col_widths.iter().map(|w| "─".repeat(w + 2)).collect();
    println!("┌{}┐", border_parts.join("┬"));

    // Header
    let header_cells: Vec<String> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| format!(" {:<w$} ", c.bold().yellow(), w = col_widths[i]))
        .collect();
    println!("│{}│", header_cells.join("│"));

    // Divider
    println!("├{}┤", border_parts.join("┼"));

    // Rows
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let w = if i < col_widths.len() { col_widths[i] } else { val.len() };
                format!(" {:<w$} ", val, w = w)
            })
            .collect();
        println!("│{}│", cells.join("│"));
    }

    // Bottom border
    println!("└{}┘", border_parts.join("┴"));
}
