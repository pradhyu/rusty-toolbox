pub mod loader;
pub mod tui;

use colored::Colorize;
use loader::{query_to_table, ExcelCatalog};
use std::path::PathBuf;

pub fn execute_xlsx_command(
    path: PathBuf,
    query: Option<String>,
    interactive: bool,
    dump_sqlite: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let catalog = ExcelCatalog::load_from_path(&path).map_err(|e| format!("Failed to load Excel catalog: {}", e))?;

    if let Some(sqlite_out) = dump_sqlite {
        let total_sheets: usize = catalog.workbooks.iter().map(|w| w.sheet_names.len()).sum();
        println!("🔄 Converting Excel catalog '{}' ({} files, {} sheets) to SQLite '{}'...", path.display(), catalog.workbooks.len(), total_sheets, sqlite_out.display());
        catalog.export_to_sqlite(&sqlite_out).map_err(|e| format!("Export error: {}", e))?;
        println!("{} Exported {} workbook(s) ({} sheets) to {}", "✔".green(), catalog.workbooks.len(), total_sheets, sqlite_out.display());
        return Ok(());
    }

    if let Some(ref q) = query {
        let cleaned_query = clean_sql_query(q);
        println!("{} Executing: {}", "🔍".cyan(), cleaned_query.bright_white().bold());
        let conn = catalog.create_in_memory_sqlite().map_err(|e| format!("SQLite initialization error: {}", e))?;
        let (columns, rows) = query_to_table(&conn, &cleaned_query).map_err(|e| format!("Query failed: {}", e))?;

        render_ascii_table(&columns, &rows);
        println!("({} rows returned)\n", rows.len());
        return Ok(());
    }

    if interactive || query.is_none() {
        tui::run_xlsx_tui(catalog)?;
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

pub fn clean_sql_query(query: &str) -> String {
    // 1. Normalize line endings (CRLF -> LF)
    let normalized = query.replace("\r\n", "\n").replace('\r', "\n");

    // 2. Normalize smart quotes often copied from blogs/word/markdown
    let normalized = normalized
        .replace(['‘', '’'], "'")
        .replace(['“', '”'], "\"");

    // 3. Normalize shell escape characters outside string literals
    // Handles:
    // - Bash/Zsh line continuations: \
    // - PowerShell line continuations: `
    // - Windows CMD line continuations: ^
    let mut cleaned = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_is_space = false;

    for c in normalized.chars() {
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            cleaned.push(c);
            prev_is_space = false;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            cleaned.push(c);
            prev_is_space = false;
        } else if in_single_quote || in_double_quote {
            cleaned.push(c);
        } else if c == '\\' || c == '`' || c == '^' {
            // Stray shell line continuation outside of quotes
            if !prev_is_space {
                cleaned.push(' ');
                prev_is_space = true;
            }
        } else if c.is_whitespace() {
            if !prev_is_space {
                cleaned.push(' ');
                prev_is_space = true;
            }
        } else {
            cleaned.push(c);
            prev_is_space = false;
        }
    }

    cleaned.trim().trim_end_matches(';').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_multiline_backtick() {
        let ps_query = "SELECT * `\r\n FROM Orders `\r\n WHERE Amount > 100;";
        assert_eq!(clean_sql_query(ps_query), "SELECT * FROM Orders WHERE Amount > 100");
    }

    #[test]
    fn test_cmd_multiline_caret() {
        let cmd_query = "SELECT * ^\r\n FROM Orders ^\r\n WHERE Amount > 100;";
        assert_eq!(clean_sql_query(cmd_query), "SELECT * FROM Orders WHERE Amount > 100");
    }

    #[test]
    fn test_bash_multiline_backslash() {
        let bash_query = "SELECT o.ID, c.Name \\\n FROM Orders o \\\n JOIN Customers c ON o.CustID = c.ID;";
        assert_eq!(clean_sql_query(bash_query), "SELECT o.ID, c.Name FROM Orders o JOIN Customers c ON o.CustID = c.ID");
    }

    #[test]
    fn test_preserves_content_inside_quotes() {
        let query = "SELECT * FROM Users WHERE Email = 'alice\\test`^@example.com'";
        assert_eq!(clean_sql_query(query), "SELECT * FROM Users WHERE Email = 'alice\\test`^@example.com'");
    }
}
