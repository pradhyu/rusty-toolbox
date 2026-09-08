use crate::modules::hsql::parser::HsqlDatabase;
use crate::modules::hsql::sqlite_bridge::SqliteBridge;
use arboard::Clipboard;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Terminal,
};
use rusqlite::Connection;
use std::collections::HashSet;
use std::io::{self, Stdout};
use std::time::Duration;

pub struct HsqlTuiApp {
    db: HsqlDatabase,
    conn: Option<Connection>,
    table_names: Vec<String>,
    selected_table_idx: usize,
    selected_row_idx: usize,
    multi_selected_rows: HashSet<usize>,
    current_columns: Vec<String>,
    current_rows: Vec<Vec<String>>,
    status_msg: String,
    query_input: String,
    in_query_mode: bool,
    focus_sidebar: bool,
}

impl HsqlTuiApp {
    pub fn new(db: HsqlDatabase) -> Self {
        let mut table_names: Vec<String> = db.tables.keys().cloned().collect();
        table_names.sort();

        let conn = SqliteBridge::create_in_memory_from_hsql(&db).ok();

        let mut app = Self {
            db,
            conn,
            table_names,
            selected_table_idx: 0,
            selected_row_idx: 0,
            multi_selected_rows: HashSet::new(),
            current_columns: Vec::new(),
            current_rows: Vec::new(),
            status_msg: "Ready. [Space] Select  [y] Yank  [/] SQL Query".to_string(),
            query_input: String::new(),
            in_query_mode: false,
            focus_sidebar: true,
        };

        app.load_current_table();
        app
    }

    pub fn load_current_table(&mut self) {
        if self.table_names.is_empty() {
            return;
        }
        let tbl_name = &self.table_names[self.selected_table_idx];
        if let Some(tbl) = self.db.tables.get(tbl_name) {
            self.current_columns = tbl.columns.iter().map(|c| c.name.clone()).collect();
            self.current_rows = tbl.rows.clone();
            self.selected_row_idx = 0;
            self.multi_selected_rows.clear();
            self.status_msg = format!("Viewing table '{}' ({} rows, {} cols)", tbl_name, self.current_rows.len(), self.current_columns.len());
        }
    }

    pub fn run_custom_query(&mut self) {
        let sql = self.query_input.trim();
        if sql.is_empty() {
            self.load_current_table();
            return;
        }

        let cleaned_sql = crate::modules::xlsx::clean_sql_query(sql);

        if let Some(ref conn) = self.conn {
            match SqliteBridge::query_to_table(conn, &cleaned_sql) {
                Ok((cols, rows)) => {
                    self.current_columns = cols;
                    let count = rows.len();
                    self.current_rows = rows;
                    self.selected_row_idx = 0;
                    self.multi_selected_rows.clear();
                    self.status_msg = format!("Query OK: {} rows returned", count);
                }
                Err(e) => {
                    self.status_msg = format!("SQL Error: {}", e);
                }
            }
        }
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
            self.status_msg = format!("Selected all {} rows. Press 'y' to yank.", self.current_rows.len());
        }
    }
}

pub fn run_hsql_tui(db: HsqlDatabase) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = HsqlTuiApp::new(db);
    let res = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("TUI Error: {:?}", e);
    }
    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut HsqlTuiApp,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Query input bar
                    Constraint::Min(5),    // Main split (Tables list | Data Grid)
                    Constraint::Length(3), // Status bar
                ])
                .split(f.area());

            // 1. Top Query Bar
            let query_title = if app.in_query_mode {
                " SQL Query (Type & Press Enter, Esc to cancel) "
            } else {
                " SQL Query (Press '/' to edit) "
            };
            let query_block = Block::default()
                .borders(Borders::ALL)
                .border_style(if app.in_query_mode {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .title(query_title);
            let query_text = if app.query_input.is_empty() && !app.in_query_mode {
                Span::styled(format!("SELECT * FROM {};", app.table_names.get(app.selected_table_idx).unwrap_or(&"".to_string())), Style::default().fg(Color::DarkGray))
            } else {
                Span::raw(&app.query_input)
            };
            f.render_widget(Paragraph::new(query_text).block(query_block), chunks[0]);

            // 2. Main Area: Left Sidebar (Tables) | Right Data View
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(25), // Table list
                    Constraint::Min(20),    // Table Data Grid
                ])
                .split(chunks[1]);

            // Tables sidebar
            let table_items: Vec<Line> = app
                .table_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let prefix = if i == app.selected_table_idx { "▶ " } else { "  " };
                    let style = if i == app.selected_table_idx {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(vec![Span::styled(format!("{}{}", prefix, name), style)])
                })
                .collect();

            let sidebar_title = format!(" Tables ({}) ", app.table_names.len());
            let sidebar_border = if app.focus_sidebar { Color::Green } else { Color::DarkGray };
            let sidebar = Paragraph::new(table_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(sidebar_border))
                    .title(Span::styled(sidebar_title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            );
            f.render_widget(sidebar, main_chunks[0]);

            // Right Table Data Grid
            let mut header_cells = vec![Cell::from("SEL").style(Style::default().fg(Color::DarkGray))];
            header_cells.extend(app.current_columns.iter().map(|c| {
                Cell::from(c.as_str()).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            }));
            let header = Row::new(header_cells).height(1).bottom_margin(1);

            let rows: Vec<Row> = app
                .current_rows
                .iter()
                .enumerate()
                .map(|(r_idx, row_data)| {
                    let is_multi_selected = app.multi_selected_rows.contains(&r_idx);
                    let is_cursor = r_idx == app.selected_row_idx && !app.focus_sidebar;

                    let sel_icon = if is_multi_selected { " ✔ " } else { "   " };
                    let mut cells = vec![Cell::from(sel_icon).style(if is_multi_selected {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    })];

                    cells.extend(row_data.iter().map(|v| Cell::from(v.as_str())));
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
            let grid_title = format!(" Data View: {} rows{} ", app.current_rows.len(), sel_info);
            let table_border = if !app.focus_sidebar { Color::Green } else { Color::DarkGray };
            let data_table = Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(table_border))
                        .title(Span::styled(grid_title, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                );
            let mut table_state = TableState::default();
            table_state.select(Some(app.selected_row_idx));
            f.render_stateful_widget(data_table, main_chunks[1], &mut table_state);

            // 3. Status Bar
            let status_line = Line::from(vec![
                Span::styled(" [Tab] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Switch  "),
                Span::styled(" [Space] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Select  "),
                Span::styled(" [y] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Yank  "),
                Span::styled(" [a] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" All  "),
                Span::styled(" [/] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" SQL  "),
                Span::styled(format!(" | {} ", app.status_msg), Style::default().fg(Color::Yellow)),
            ]);
            let status_widget = Paragraph::new(status_line).block(Block::default().borders(Borders::ALL));
            f.render_widget(status_widget, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Release {
                        if app.in_query_mode {
                            match key.code {
                                KeyCode::Enter => {
                                    app.in_query_mode = false;
                                    app.run_custom_query();
                                }
                                KeyCode::Esc => {
                                    app.in_query_mode = false;
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
                                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                                KeyCode::Tab | KeyCode::BackTab => {
                                    app.focus_sidebar = !app.focus_sidebar;
                                }
                                KeyCode::Char('/') => {
                                    app.in_query_mode = true;
                                    app.query_input.clear();
                                }
                                KeyCode::Char(' ') => {
                                    if !app.focus_sidebar {
                                        app.toggle_row_selection();
                                    }
                                }
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    app.yank_selection_to_clipboard();
                                }
                                KeyCode::Char('a') => {
                                    if !app.focus_sidebar {
                                        app.toggle_select_all();
                                    }
                                }
                                KeyCode::Char('c') => {
                                    if !app.focus_sidebar {
                                        app.multi_selected_rows.clear();
                                        app.status_msg = "Cleared selection".to_string();
                                    }
                                }
                                KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if !app.focus_sidebar {
                                        let max_len = app.current_rows.len().saturating_sub(1);
                                        app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                    }
                                }
                                KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if !app.focus_sidebar {
                                        app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                                    }
                                }
                                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if !app.focus_sidebar {
                                        let max_len = app.current_rows.len().saturating_sub(1);
                                        app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                    }
                                }
                                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if !app.focus_sidebar {
                                        app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                                    }
                                }
                                KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if !app.focus_sidebar {
                                        let max_len = app.current_rows.len().saturating_sub(1);
                                        app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                    }
                                }
                                KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    if !app.focus_sidebar {
                                        app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                                    }
                                }
                                KeyCode::Home | KeyCode::Char('g') => {
                                    if !app.focus_sidebar {
                                        app.selected_row_idx = 0;
                                    }
                                }
                                KeyCode::End | KeyCode::Char('G') => {
                                    if !app.focus_sidebar {
                                        app.selected_row_idx = app.current_rows.len().saturating_sub(1);
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.focus_sidebar {
                                        if app.selected_table_idx > 0 {
                                            app.selected_table_idx -= 1;
                                            app.load_current_table();
                                        }
                                    } else if app.selected_row_idx > 0 {
                                        app.selected_row_idx -= 1;
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.focus_sidebar {
                                        if app.selected_table_idx + 1 < app.table_names.len() {
                                            app.selected_table_idx += 1;
                                            app.load_current_table();
                                        }
                                    } else if app.selected_row_idx + 1 < app.current_rows.len() {
                                        app.selected_row_idx += 1;
                                    }
                                }
                                KeyCode::PageUp => {
                                    if !app.focus_sidebar {
                                        app.selected_row_idx = app.selected_row_idx.saturating_sub(25);
                                    }
                                }
                                KeyCode::PageDown => {
                                    if !app.focus_sidebar {
                                        let max_len = app.current_rows.len().saturating_sub(1);
                                        app.selected_row_idx = (app.selected_row_idx + 25).min(max_len);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
