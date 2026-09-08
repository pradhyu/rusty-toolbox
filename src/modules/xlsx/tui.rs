use crate::modules::xlsx::loader::{query_to_table, ExcelCatalog};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use rusqlite::Connection;
use std::io;
use std::time::Duration;

#[derive(Clone)]
struct SheetEntry {
    wb_idx: usize,
    schema_name: String,
    sheet_name: String,
}

pub struct XlsxTuiApp {
    catalog: ExcelCatalog,
    conn: Option<Connection>,
    sheet_entries: Vec<SheetEntry>,
    selected_sheet_idx: usize,
    selected_row_idx: usize,
    current_columns: Vec<String>,
    current_rows: Vec<Vec<String>>,
    status_msg: String,
    query_input: String,
    in_query_mode: bool,
}

impl XlsxTuiApp {
    pub fn new(catalog: ExcelCatalog) -> Self {
        let mut sheet_entries = Vec::new();

        for (wb_idx, wb) in catalog.workbooks.iter().enumerate() {
            for sheet_name in &wb.sheet_names {
                sheet_entries.push(SheetEntry {
                    wb_idx,
                    schema_name: wb.schema_name.clone(),
                    sheet_name: sheet_name.clone(),
                });
            }
        }

        let mut app = Self {
            catalog,
            conn: None,
            sheet_entries,
            selected_sheet_idx: 0,
            selected_row_idx: 0,
            current_columns: Vec::new(),
            current_rows: Vec::new(),
            status_msg: "Ready. Press '/' for SQL query, 'Tab' to switch panels, 'q' to quit.".to_string(),
            query_input: String::new(),
            in_query_mode: false,
        };

        if let Ok(conn) = app.catalog.create_in_memory_sqlite() {
            app.conn = Some(conn);
        }

        app.load_active_sheet();
        app
    }

    fn load_active_sheet(&mut self) {
        if let Some(entry) = self.sheet_entries.get(self.selected_sheet_idx) {
            if let Some(wb) = self.catalog.workbooks.get(entry.wb_idx) {
                if let Some(sheet) = wb.sheets.get(&entry.sheet_name) {
                    self.current_columns = sheet.columns.clone();
                    self.current_rows = sheet.rows.clone();
                    self.selected_row_idx = 0;
                    if self.catalog.workbooks.len() > 1 {
                        self.status_msg = format!("Schema: '{}' | Sheet: '{}' ({} rows, {} cols)", entry.schema_name, entry.sheet_name, sheet.rows.len(), sheet.columns.len());
                    } else {
                        self.status_msg = format!("Sheet: '{}' ({} rows, {} cols)", entry.sheet_name, sheet.rows.len(), sheet.columns.len());
                    }
                }
            }
        }
    }

    fn execute_custom_query(&mut self) {
        let query = self.query_input.trim();
        if query.is_empty() {
            self.load_active_sheet();
            return;
        }

        let cleaned_query = crate::modules::xlsx::clean_sql_query(query);

        if let Some(ref conn) = self.conn {
            match query_to_table(conn, &cleaned_query) {
                Ok((cols, rows)) => {
                    self.current_columns = cols;
                    let count = rows.len();
                    self.current_rows = rows;
                    self.selected_row_idx = 0;
                    self.status_msg = format!("Query OK: {} rows returned", count);
                }
                Err(err) => {
                    self.status_msg = format!("SQL Error: {}", err);
                }
            }
        }
    }
}

pub fn run_xlsx_tui(catalog: ExcelCatalog) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = XlsxTuiApp::new(catalog);
    let mut focus_sidebar = true;

    loop {
        terminal.draw(|f| ui(f, &app, focus_sidebar))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.in_query_mode {
                    match key.code {
                        KeyCode::Enter => {
                            app.in_query_mode = false;
                            app.execute_custom_query();
                        }
                        KeyCode::Esc => {
                            app.in_query_mode = false;
                            app.query_input.clear();
                            app.load_active_sheet();
                        }
                        KeyCode::Backspace => {
                            app.query_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.query_input.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Tab | KeyCode::BackTab => {
                            focus_sidebar = !focus_sidebar;
                        }
                        KeyCode::Char('/') => {
                            app.in_query_mode = true;
                            if app.query_input.is_empty() {
                                if let Some(entry) = app.sheet_entries.get(app.selected_sheet_idx) {
                                    if app.catalog.workbooks.len() > 1 {
                                        app.query_input = format!("SELECT * FROM {}.\"{}\" LIMIT 50", entry.schema_name, entry.sheet_name);
                                    } else {
                                        app.query_input = format!("SELECT * FROM \"{}\" LIMIT 50", entry.sheet_name);
                                    }
                                }
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if focus_sidebar {
                                if app.selected_sheet_idx > 0 {
                                    app.selected_sheet_idx -= 1;
                                    app.load_active_sheet();
                                }
                            } else if app.selected_row_idx > 0 {
                                app.selected_row_idx -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if focus_sidebar {
                                if app.selected_sheet_idx + 1 < app.sheet_entries.len() {
                                    app.selected_sheet_idx += 1;
                                    app.load_active_sheet();
                                }
                            } else if app.selected_row_idx + 1 < app.current_rows.len() {
                                app.selected_row_idx += 1;
                            }
                        }
                        KeyCode::PageUp => {
                            if !focus_sidebar {
                                app.selected_row_idx = app.selected_row_idx.saturating_sub(15);
                            }
                        }
                        KeyCode::PageDown => {
                            if !focus_sidebar {
                                let max_len = app.current_rows.len().saturating_sub(1);
                                app.selected_row_idx = (app.selected_row_idx + 15).min(max_len);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &XlsxTuiApp, focus_sidebar: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // 1. Header
    let filename = app.catalog.root_path.file_name().and_then(|s| s.to_str()).unwrap_or("Catalog");
    let total_sheets = app.sheet_entries.len();
    let total_workbooks = app.catalog.workbooks.len();
    let header_title = if total_workbooks > 1 {
        format!(" 📊 Multi-Workbook Catalog (rtb xlsx) — {} ({} files/schemas, {} total sheets) ", filename, total_workbooks, total_sheets)
    } else {
        format!(" 📊 Excel & Spreadsheet Inspector (rtb xlsx) — {} ({} sheets) ", filename, total_sheets)
    };

    let header = Paragraph::new(header_title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
    f.render_widget(header, chunks[0]);

    // 2. Main Body (Sidebar + Table)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(chunks[1]);

    // Sidebar: Schemas & Worksheets
    let sheet_items: Vec<ListItem> = app
        .sheet_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let row_count = app
                .catalog
                .workbooks
                .get(entry.wb_idx)
                .and_then(|wb| wb.sheets.get(&entry.sheet_name))
                .map(|s| s.rows.len())
                .unwrap_or(0);

            let prefix = if i == app.selected_sheet_idx { "▶ " } else { "  " };
            let label = if app.catalog.workbooks.len() > 1 {
                format!("{}{}.{} ({}r)", prefix, entry.schema_name, entry.sheet_name, row_count)
            } else {
                format!("{}{:<18} ({}r)", prefix, entry.sheet_name, row_count)
            };

            ListItem::new(label).style(if i == app.selected_sheet_idx {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            })
        })
        .collect();

    let sidebar_border_color = if focus_sidebar { Color::Green } else { Color::DarkGray };
    let sidebar_title = if app.catalog.workbooks.len() > 1 { " 📁 Schemas & Sheets " } else { " 📁 Worksheets " };
    let sheet_list = List::new(sheet_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(sidebar_title)
            .border_style(Style::default().fg(sidebar_border_color)),
    );
    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_sheet_idx));
    f.render_stateful_widget(sheet_list, body_chunks[0], &mut list_state);

    // Main Table
    let table_border_color = if !focus_sidebar { Color::Green } else { Color::DarkGray };
    let header_cells = app.current_columns.iter().map(|h| {
        Cell::from(h.as_str()).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    });
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .current_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let cells = row.iter().map(|c| Cell::from(c.as_str()));
            let mut r = Row::new(cells);
            if i == app.selected_row_idx && !focus_sidebar {
                r = r.style(Style::default().bg(Color::Rgb(30, 60, 90)).fg(Color::White));
            }
            r
        })
        .collect();

    let widths: Vec<Constraint> = if app.current_columns.is_empty() {
        vec![Constraint::Percentage(100)]
    } else {
        vec![Constraint::Max(25); app.current_columns.len()]
    };

    let table_title = format!(" 📄 Data Grid ({} rows) ", app.current_rows.len());
    let table_widget = Table::new(rows, widths)
        .header(header_row)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(table_title)
                .border_style(Style::default().fg(table_border_color)),
        )
        .column_spacing(2);

    let mut table_state = TableState::default();
    table_state.select(Some(app.selected_row_idx));
    f.render_stateful_widget(table_widget, body_chunks[1], &mut table_state);

    // 3. Footer / Query Bar
    if app.in_query_mode {
        let query_bar = Paragraph::new(format!("🔍 SQL > {}", app.query_input))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Execute SQL (Enter to Run, Esc to Cancel) ")
                    .border_style(Style::default().fg(Color::Yellow)),
            );
        f.render_widget(query_bar, chunks[2]);
    } else {
        let footer_text = format!(" {} | [Tab] Switch Pane  [/] SQL Query  [↑/↓] Navigate  [q] Quit", app.status_msg);
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
}
