use crate::modules::hsql::parser::HsqlDatabase;
use crate::modules::hsql::sqlite_bridge::SqliteBridge;
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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Terminal,
};
use rusqlite::Connection;
use std::io::{self, Stdout};
use std::time::Duration;

pub struct HsqlTuiApp {
    db: HsqlDatabase,
    conn: Option<Connection>,
    table_names: Vec<String>,
    selected_table_idx: usize,
    selected_row_idx: usize,
    scroll_x: usize,
    current_columns: Vec<String>,
    current_rows: Vec<Vec<String>>,
    status_msg: String,
    query_input: String,
    in_query_mode: bool,
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
            scroll_x: 0,
            current_columns: Vec::new(),
            current_rows: Vec::new(),
            status_msg: "Ready".to_string(),
            query_input: String::new(),
            in_query_mode: false,
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
            self.current_columns = tbl.columns.iter().map(|c| format!("{} ({})", c.name, c.col_type)).collect();
            self.current_rows = tbl.rows.clone();
            self.selected_row_idx = 0;
            self.status_msg = format!("Viewing table '{}' ({} rows)", tbl_name, self.current_rows.len());
        }
    }

    pub fn run_custom_query(&mut self) {
        let sql = self.query_input.trim();
        if sql.is_empty() {
            return;
        }

        if let Some(ref conn) = self.conn {
            match SqliteBridge::query_to_table(conn, sql) {
                Ok((cols, rows)) => {
                    self.current_columns = cols;
                    self.current_rows = rows;
                    self.selected_row_idx = 0;
                    self.status_msg = format!("Query executed successfully: {} rows returned", self.current_rows.len());
                }
                Err(e) => {
                    self.status_msg = format!("SQL Error: {}", e);
                }
            }
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
                    Constraint::Length(1), // Status bar
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
                    let style = if i == app.selected_table_idx {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(vec![Span::styled(format!(" 📄 {} ", name), style)])
                })
                .collect();

            let sidebar_title = format!(" Tables ({}) ", app.table_names.len());
            let sidebar = Paragraph::new(table_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(Span::styled(sidebar_title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            );
            f.render_widget(sidebar, main_chunks[0]);

            // Right Table Data Grid
            let header_cells: Vec<Cell> = app
                .current_columns
                .iter()
                .map(|c| Cell::from(c.as_str()).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                .collect();
            let header = Row::new(header_cells).height(1).bottom_margin(1);

            let rows: Vec<Row> = app
                .current_rows
                .iter()
                .enumerate()
                .map(|(r_idx, row_data)| {
                    let style = if r_idx == app.selected_row_idx {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    let cells: Vec<Cell> = row_data.iter().map(|v| Cell::from(v.as_str())).collect();
                    Row::new(cells).style(style)
                })
                .collect();

            let widths: Vec<Constraint> = vec![Constraint::Min(15); app.current_columns.len()];
            let grid_title = format!(" Data View: {} rows ", app.current_rows.len());
            let data_table = Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan))
                        .title(Span::styled(grid_title, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                );
            f.render_widget(data_table, main_chunks[1]);

            // 3. Status Bar
            let status_line = Line::from(vec![
                Span::styled(" [Tab / ↑↓] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Switch Tables  "),
                Span::styled(" [/] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Custom SQL  "),
                Span::styled(" [q] ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" Quit  "),
                Span::styled(format!(" | {} ", app.status_msg), Style::default().fg(Color::Yellow)),
            ]);
            f.render_widget(Paragraph::new(status_line), chunks[2]);
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
                                KeyCode::Char('/') => {
                                    app.in_query_mode = true;
                                    app.query_input.clear();
                                }
                                KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                                    if !app.table_names.is_empty() {
                                        app.selected_table_idx = (app.selected_table_idx + 1) % app.table_names.len();
                                        app.load_current_table();
                                    }
                                }
                                KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                                    if !app.table_names.is_empty() {
                                        if app.selected_table_idx == 0 {
                                            app.selected_table_idx = app.table_names.len() - 1;
                                        } else {
                                            app.selected_table_idx -= 1;
                                        }
                                        app.load_current_table();
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
