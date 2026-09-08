pub mod loader;
pub mod tui;

use colored::Colorize;
use loader::{query_to_table, ExcelWorkbook};
use std::path::PathBuf;

pub fn execute_xlsx_command(
    path: PathBuf,
    query: Option<String>,
    interactive: bool,
    dump_sqlite: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let workbook = ExcelWorkbook::load_from_file(&path).map_err(|e| format!("Failed to load Excel spreadsheet: {}", e))?;

    if let Some(sqlite_out) = dump_sqlite {
        println!("🔄 Converting Excel '{}' to SQLite '{}'...", path.display(), sqlite_out.display());
        workbook.export_to_sqlite(&sqlite_out).map_err(|e| format!("Export error: {}", e))?;
        println!("{} Exported {} sheet(s) to {}", "✔".green(), workbook.sheet_names.len(), sqlite_out.display());
        return Ok(());
    }

    if let Some(ref q) = query {
        println!("{} Executing: {}", "🔍".cyan(), q.bright_white().bold());
        let conn = workbook.create_in_memory_sqlite().map_err(|e| format!("SQLite initialization error: {}", e))?;
        let (columns, rows) = query_to_table(&conn, q).map_err(|e| format!("Query failed: {}", e))?;

        render_ascii_table(&columns, &rows);
        println!("({} rows returned)\n", rows.len());
        return Ok(());
    }

    if interactive || query.is_none() {
        tui::run_xlsx_tui(workbook)?;
    }

    Ok(())
}

fn render_ascii_table(columns: &[String], rows: &[Vec<String>]) {
    if columns.is_empty() {
        println!("(No columns)");
        return;
    }

    let mut col_widths: Vec<usize> = columns.iter().map(|c| c.len().max(4)).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    // Top border
    let top_border: Vec<String> = col_widths.iter().map(|w| "─".repeat(w + 2)).collect();
    println!("┌{}┐", top_border.join("┬"));

    // Header row
    let header_cells: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, col)| format!(" {:<width$} ", col, width = col_widths[i]))
        .collect();
    println!("│{}│", header_cells.join("│"));

    // Divider
    let divider: Vec<String> = col_widths.iter().map(|w| "─".repeat(w + 2)).collect();
    println!("├{}┤", divider.join("┼"));

    // Data rows
    for row in rows {
        let row_cells: Vec<String> = (0..columns.len())
            .map(|i| {
                let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
                format!(" {:<width$} ", val, width = col_widths[i])
            })
            .collect();
        println!("│{}│", row_cells.join("│"));
    }

    // Bottom border
    let bottom_border: Vec<String> = col_widths.iter().map(|w| "─".repeat(w + 2)).collect();
    println!("└{}┘", bottom_border.join("┴"));
}
