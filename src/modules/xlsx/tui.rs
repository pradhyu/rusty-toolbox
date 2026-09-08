use crate::modules::xlsx::loader::{query_to_table, ExcelCatalog};
use arboard::Clipboard;
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
use std::collections::HashSet;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct SheetEntry {
    wb_idx: usize,
    schema_name: String,
    sheet_name: String,
}

pub struct XlsxTuiApp {
    catalog: ExcelCatalog,
    conn_shared: Arc<Mutex<Option<Connection>>>,
    is_indexing: Arc<AtomicBool>,
    sheet_entries: Vec<SheetEntry>,
    selected_sheet_idx: usize,
    selected_row_idx: usize,
    multi_selected_rows: HashSet<usize>,
    current_columns: Vec<String>,
    current_rows: Vec<Vec<String>>,
    status_msg: String,
    query_input: String,
    in_query_mode: bool,
    has_more_rows: bool,
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

        let conn_shared = Arc::new(Mutex::new(None));
        let is_indexing = Arc::new(AtomicBool::new(true));

        // Spawn background worker to index SQLite without blocking UI startup
        let catalog_clone = catalog.clone();
        let conn_bg = Arc::clone(&conn_shared);
        let indexing_bg = Arc::clone(&is_indexing);

        thread::spawn(move || {
            if let Ok(conn) = catalog_clone.create_in_memory_sqlite() {
                if let Ok(mut lock) = conn_bg.lock() {
                    *lock = Some(conn);
                }
            }
            indexing_bg.store(false, Ordering::SeqCst);
        });

        let mut app = Self {
            catalog,
            conn_shared,
            is_indexing,
            sheet_entries,
            selected_sheet_idx: 0,
            selected_row_idx: 0,
            multi_selected_rows: HashSet::new(),
            current_columns: Vec::new(),
            current_rows: Vec::new(),
            status_msg: "Ready (Instant Preview). [Space] Select  [y] Yank  [/] SQL Query".to_string(),
            query_input: String::new(),
            in_query_mode: false,
            has_more_rows: true,
        };

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
                    self.multi_selected_rows.clear();
                    self.has_more_rows = true;
                    if self.catalog.workbooks.len() > 1 {
                        self.status_msg = format!("Schema: '{}' | Sheet: '{}' (previewing top {} rows)", entry.schema_name, entry.sheet_name, sheet.rows.len());
                    } else {
                        self.status_msg = format!("Sheet: '{}' (previewing top {} rows)", entry.sheet_name, sheet.rows.len());
                    }
                }
            }
        }
    }

    fn maybe_load_more_rows(&mut self) {
        if !self.has_more_rows {
            return;
        }

        if let Ok(lock) = self.conn_shared.lock() {
            if let Some(ref conn) = *lock {
                if let Some(entry) = self.sheet_entries.get(self.selected_sheet_idx) {
                    let table_ref = if self.catalog.workbooks.len() > 1 {
                        format!("\"{}\".\"{}\"", entry.schema_name, entry.sheet_name)
                    } else {
                        format!("\"{}\"", entry.sheet_name)
                    };

                    let offset = self.current_rows.len();
                    let page_sql = format!("SELECT * FROM {} LIMIT 1000 OFFSET {}", table_ref, offset);
                    if let Ok((_, more_rows)) = query_to_table(conn, &page_sql) {
                        if more_rows.is_empty() {
                            self.has_more_rows = false;
                        } else {
                            self.current_rows.extend(more_rows);
                            self.status_msg = format!("Loaded {} rows (scroll down for more)", self.current_rows.len());
                        }
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

        // If indexing is in progress in background, wait for it to complete
        while self.is_indexing.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
        }

        if let Ok(lock) = self.conn_shared.lock() {
            if let Some(ref conn) = *lock {
                match query_to_table(conn, &cleaned_query) {
                    Ok((cols, rows)) => {
                        self.current_columns = cols;
                        let count = rows.len();
                        self.current_rows = rows;
                        self.selected_row_idx = 0;
                        self.multi_selected_rows.clear();
                        self.has_more_rows = false;
                        self.status_msg = format!("Query OK: {} rows returned", count);
                        return;
                    }
                    Err(err) => {
                        self.status_msg = format!("SQL Error: {}", err);
                        return;
                    }
                }
            }
        }

        self.status_msg = "Database connection error".to_string();
    }

    fn yank_selection_to_clipboard(&mut self) {
        if self.current_rows.is_empty() {
            self.status_msg = "No rows to yank".to_string();
            return;
        }

        let rows_to_yank: Vec<usize> = if !self.multi_selected_rows.is_empty() {
            let mut indices: Vec<usize> = self.multi_selected_rows.iter().cloned().collect();
            indices.sort();
            indices
        } else {
            vec![self.selected_row_idx]
        };

        let mut lines = Vec::new();
        // Header
        if !self.current_columns.is_empty() {
            lines.push(self.current_columns.join("\t"));
        }

        for &idx in &rows_to_yank {
            if let Some(row) = self.current_rows.get(idx) {
                lines.push(row.join("\t"));
            }
        }

        let tsv_data = lines.join("\n");
        let count = rows_to_yank.len();

        match Clipboard::new().and_then(|mut cb| cb.set_text(tsv_data)) {
            Ok(_) => {
                self.status_msg = format!("✔ Yanked {} row(s) to clipboard (TSV format)", count);
            }
            Err(e) => {
                self.status_msg = format!("Clipboard error: {}", e);
            }
        }
    }

    fn toggle_row_selection(&mut self) {
        if self.current_rows.is_empty() {
            return;
        }
        if self.multi_selected_rows.contains(&self.selected_row_idx) {
            self.multi_selected_rows.remove(&self.selected_row_idx);
        } else {
            self.multi_selected_rows.insert(self.selected_row_idx);
        }
        let sel_count = self.multi_selected_rows.len();
        if sel_count > 0 {
            self.status_msg = format!("{} row(s) highlighted. Press 'y' to yank.", sel_count);
        } else {
            self.status_msg = "Selection cleared".to_string();
        }
    }

    fn toggle_select_all(&mut self) {
        if self.current_rows.is_empty() {
            return;
        }
        if self.multi_selected_rows.len() == self.current_rows.len() {
            self.multi_selected_rows.clear();
            self.status_msg = "Cleared all selections".to_string();
        } else {
            self.multi_selected_rows = (0..self.current_rows.len()).collect();
            self.status_msg = format!("Selected all {} visible rows. Press 'y' to yank.", self.current_rows.len());
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

        if event::poll(Duration::from_millis(50))? {
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
                        KeyCode::Char(' ') => {
                            if !focus_sidebar {
                                app.toggle_row_selection();
                            }
                        }
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            app.yank_selection_to_clipboard();
                        }
                        KeyCode::Char('a') => {
                            if !focus_sidebar {
                                app.toggle_select_all();
                            }
                        }
                        KeyCode::Char('c') => {
                            if !focus_sidebar {
                                app.multi_selected_rows.clear();
                                app.status_msg = "Cleared selection".to_string();
                            }
                        }
                        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !focus_sidebar {
                                let max_len = app.current_rows.len().saturating_sub(1);
                                app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                if app.selected_row_idx + 25 >= app.current_rows.len() {
                                    app.maybe_load_more_rows();
                                }
                            }
                        }
                        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !focus_sidebar {
                                app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                            }
                        }
                        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !focus_sidebar {
                                let max_len = app.current_rows.len().saturating_sub(1);
                                app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                if app.selected_row_idx + 25 >= app.current_rows.len() {
                                    app.maybe_load_more_rows();
                                }
                            }
                        }
                        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !focus_sidebar {
                                app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                            }
                        }
                        KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !focus_sidebar {
                                let max_len = app.current_rows.len().saturating_sub(1);
                                app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                if app.selected_row_idx + 25 >= app.current_rows.len() {
                                    app.maybe_load_more_rows();
                                }
                            }
                        }
                        KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !focus_sidebar {
                                app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                            }
                        }
                        KeyCode::Home | KeyCode::Char('g') => {
                            if !focus_sidebar {
                                app.selected_row_idx = 0;
                            }
                        }
                        KeyCode::End | KeyCode::Char('G') => {
                            if !focus_sidebar {
                                app.selected_row_idx = app.current_rows.len().saturating_sub(1);
                                app.maybe_load_more_rows();
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
                            } else {
                                if app.selected_row_idx + 1 < app.current_rows.len() {
                                    app.selected_row_idx += 1;
                                }
                                if app.selected_row_idx + 25 >= app.current_rows.len() {
                                    app.maybe_load_more_rows();
                                }
                            }
                        }
                        KeyCode::PageUp => {
                            if !focus_sidebar {
                                app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                            }
                        }
                        KeyCode::PageDown => {
                            if !focus_sidebar {
                                let max_len = app.current_rows.len().saturating_sub(1);
                                app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                if app.selected_row_idx + 25 >= app.current_rows.len() {
                                    app.maybe_load_more_rows();
                                }
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
    let index_badge = if app.is_indexing.load(Ordering::Relaxed) {
        " [Indexing...]"
    } else {
        ""
    };

    let header_title = if total_workbooks > 1 {
        format!(" 📊 Multi-Workbook Catalog (rtb xlsx) — {} ({} files/schemas, {} total sheets){} ", filename, total_workbooks, total_sheets, index_badge)
    } else {
        format!(" 📊 Excel & Spreadsheet Inspector (rtb xlsx) — {} ({} sheets){} ", filename, total_sheets, index_badge)
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
    
    // Prefix header with selection column indicator
    let mut header_cells = vec![Cell::from("SEL").style(Style::default().fg(Color::DarkGray))];
    header_cells.extend(app.current_columns.iter().map(|h| {
        Cell::from(h.as_str()).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    }));
    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app
        .current_rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_multi_selected = app.multi_selected_rows.contains(&i);
            let is_cursor = i == app.selected_row_idx && !focus_sidebar;

            let sel_icon = if is_multi_selected { " ✔ " } else { "   " };
            let mut cells = vec![Cell::from(sel_icon).style(if is_multi_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            })];

            cells.extend(row.iter().map(|c| Cell::from(c.as_str())));
            let mut r = Row::new(cells);

            if is_cursor && is_multi_selected {
                r = r.style(Style::default().bg(Color::Rgb(40, 90, 70)).fg(Color::White).add_modifier(Modifier::BOLD));
            } else if is_cursor {
                r = r.style(Style::default().bg(Color::Rgb(30, 60, 90)).fg(Color::White));
            } else if is_multi_selected {
                r = r.style(Style::default().bg(Color::Rgb(20, 60, 40)).fg(Color::Green));
            }

            r
        })
        .collect();

    let mut widths: Vec<Constraint> = vec![Constraint::Length(4)];
    if app.current_columns.is_empty() {
        widths.push(Constraint::Percentage(100));
    } else {
        widths.extend(vec![Constraint::Max(25); app.current_columns.len()]);
    }

    let sel_info = if !app.multi_selected_rows.is_empty() {
        format!(" ({} selected)", app.multi_selected_rows.len())
    } else {
        String::new()
    };
    let table_title = format!(" 📄 Data Grid ({} rows loaded){} ", app.current_rows.len(), sel_info);
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
        let footer_text = format!(" {} | [Space] Select  [y] Yank  [a] All  [Tab] Switch  [/] SQL  [q] Quit", app.status_msg);
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);
    }
}
