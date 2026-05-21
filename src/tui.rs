use crate::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph,
    },
    Frame,
};
use std::{
    io,
    time::{Duration, Instant},
};

/// Top-level screens in the TUI
#[derive(Clone, PartialEq)]
enum Screen {
    PasswordPrompt,
    Main,
    EntryDetail,
    AddEntry,
    ConfirmDelete,
}

/// TUI application state
pub struct TuiApp {
    /// The underlying wallet
    wallet: Wallet,
    /// Wallet password
    password: String,
    /// Current screen
    screen: Screen,
    /// Password input buffer
    password_input: String,
    /// Password input cursor position for hidden-field mask
    password_visible: bool,
    /// Password error message
    password_error: String,
    /// Key list state
    list_state: ListState,
    /// Sorted keys for display
    keys: Vec<String>,
    /// Selected key for detail/delete
    selected_key: Option<String>,
    /// Detail: whether to show value or mask it
    show_value: bool,
    /// Add entry: key input buffer
    add_key: String,
    /// Add entry: value input buffer
    add_value: String,
    /// Add entry: show value or mask
    add_value_visible: bool,
    /// Status message (shown temporarily)
    status_message: Option<(String, Instant)>,
    /// Error message
    error_message: Option<String>,
    /// Whether the app should quit
    should_quit: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        let wallet = Wallet::new();
        let keys = Vec::new();
        Self {
            wallet,
            password: String::new(),
            screen: Screen::PasswordPrompt,
            password_input: String::new(),
            password_visible: false,
            password_error: String::new(),
            list_state: ListState::default(),
            keys,
            selected_key: None,
            show_value: false,
            add_key: String::new(),
            add_value: String::new(),
            add_value_visible: false,
            status_message: None,
            error_message: None,
            should_quit: false,
        }
    }

    fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Refresh the sorted key list from the wallet
    fn refresh_keys(&mut self) {
        let mut keys = self.wallet.get_keys();
        keys.sort();
        self.keys = keys;
    }

    fn selected_key_value(&self) -> Option<(String, String)> {
        let idx = self.list_state.selected()?;
        let key = self.keys.get(idx)?.clone();
        let value = self.wallet.get(&key)?.clone();
        Some((key, value))
    }
}

/// Run the TUI event loop
pub fn run() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?;

    // Create app state
    let mut app = TuiApp::new();

    // Run
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    let mut stdout = io::stdout();
    execute!(stdout, DisableMouseCapture, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Err(ref e) = res {
        eprintln!("TUI error: {e}");
    }

    // Ensure wallet is saved on exit
    if !app.password.is_empty() {
        app.wallet.save(&app.password);
    }

    res
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut TuiApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            break;
        }

        // Check for stale status messages
        if let Some((_, when)) = &app.status_message {
            if when.elapsed() > Duration::from_secs(3) {
                app.clear_status();
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.screen.clone() {
                    Screen::PasswordPrompt => handle_password_prompt(app, key.code),
                    Screen::Main => handle_main(app, key.code),
                    Screen::EntryDetail => handle_entry_detail(app, key.code),
                    Screen::AddEntry => handle_add_entry(app, key.code),
                    Screen::ConfirmDelete => handle_confirm_delete(app, key.code),
                }
            }
        }
    }
    Ok(())
}

// ── Password Prompt Screen ──────────────────────────────────────

fn handle_password_prompt(app: &mut TuiApp, key: KeyCode) {
    match key {
        KeyCode::Enter => {
            let pw = app.password_input.trim().to_string();
            if pw.is_empty() {
                app.password_error = "Password cannot be empty".to_string();
                return;
            }
            // Try to unlock
            app.password = pw;
            app.wallet.load(&app.password);
            app.refresh_keys();
            app.screen = Screen::Main;
        }
        KeyCode::Tab => {
            app.password_visible = !app.password_visible;
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Backspace => {
            app.password_input.pop();
            app.password_error.clear();
        }
        KeyCode::Char(c) => {
            app.password_input.push(c);
            app.password_error.clear();
        }
        _ => {}
    }
}

// ── Main Screen ─────────────────────────────────────────────────

fn handle_main(app: &mut TuiApp, key: KeyCode) {
    match key {
        KeyCode::Down | KeyCode::Char('j') => {
            let i = app.list_state.selected().unwrap_or(0);
            if !app.keys.is_empty() {
                let next = (i + 1).min(app.keys.len() - 1);
                app.list_state.select(Some(next));
                app.show_value = false;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let i = app.list_state.selected().unwrap_or(0);
            if !app.keys.is_empty() {
                let prev = i.saturating_sub(1);
                app.list_state.select(Some(prev));
                app.show_value = false;
            }
        }
        KeyCode::Enter => {
            if !app.keys.is_empty() {
                app.screen = Screen::EntryDetail;
                app.show_value = false;
            }
        }
        KeyCode::Char('a') => {
            app.screen = Screen::AddEntry;
            app.add_key.clear();
            app.add_value.clear();
            app.add_value_visible = false;
            app.error_message = None;
        }
        KeyCode::Char('d') => {
            if !app.keys.is_empty() {
                if let Some(idx) = app.list_state.selected() {
                    app.selected_key = Some(app.keys[idx].clone());
                    app.screen = Screen::ConfirmDelete;
                }
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('c') => {
            // Copy selected to clipboard
            if let Some((key, value)) = app.selected_key_value() {
                match crate::clipboard::Clipboard::copy(&value) {
                    Ok(()) => app.set_status(format!("✓ Copied '{key}' to clipboard")),
                    Err(e) => app.set_status(format!("✗ Clipboard error: {e}")),
                }
            }
        }
        _ => {}
    }
}

// ── Entry Detail Screen ─────────────────────────────────────────

fn handle_entry_detail(app: &mut TuiApp, key: KeyCode) {
    match key {
        KeyCode::Tab | KeyCode::Char('h') => {
            app.show_value = !app.show_value;
        }
        KeyCode::Char('c') => {
            if let Some((key, value)) = app.selected_key_value() {
                match crate::clipboard::Clipboard::copy(&value) {
                    Ok(()) => app.set_status(format!("✓ Copied '{key}' to clipboard")),
                    Err(e) => app.set_status(format!("✗ Clipboard error: {e}")),
                }
            }
        }
        KeyCode::Esc | KeyCode::Enter => {
            app.screen = Screen::Main;
        }
        _ => {}
    }
}

// ── Add Entry Screen ────────────────────────────────────────────

fn handle_add_entry(app: &mut TuiApp, key: KeyCode) {
    match key {
        KeyCode::Tab => {
            if app.add_value_visible {
                app.add_value_visible = false;
            } else {
                app.add_value_visible = true;
            }
        }
        KeyCode::Enter => {
            let key = app.add_key.trim().to_string();
            let value = app.add_value.trim().to_string();
            if key.is_empty() || value.is_empty() {
                app.error_message = Some("Both key and value are required".to_string());
                return;
            }
            app.wallet.add(key.clone(), value.clone());
            app.wallet.save(&app.password);
            app.refresh_keys();
            app.set_status(format!("✓ Added '{key}'"));
            app.screen = Screen::Main;
        }
        KeyCode::Esc => {
            app.screen = Screen::Main;
        }
        KeyCode::Backspace => {
            if !app.add_value_visible {
                app.add_key.pop();
            } else {
                app.add_value.pop();
            }
            app.error_message = None;
        }
        KeyCode::Char(c) => {
            if !app.add_value_visible {
                app.add_key.push(c);
            } else {
                app.add_value.push(c);
            }
            app.error_message = None;
        }
        _ => {}
    }
}

// ── Confirm Delete Screen ───────────────────────────────────────

fn handle_confirm_delete(app: &mut TuiApp, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(ref key) = app.selected_key.clone() {
                app.wallet.del(key);
                app.wallet.save(&app.password);
                app.refresh_keys();
                app.set_status(format!("✓ Removed '{key}'"));
                if app.keys.is_empty() {
                    app.list_state.select(None);
                } else {
                    let idx = app.list_state.selected().unwrap_or(0);
                    app.list_state.select(Some(idx.min(app.keys.len() - 1)));
                }
                app.selected_key = None;
                app.screen = Screen::Main;
            }
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.selected_key = None;
            app.screen = Screen::Main;
        }
        _ => {}
    }
}

// ── UI Rendering ────────────────────────────────────────────────

fn ui(f: &mut Frame, app: &mut TuiApp) {
    match app.screen {
        Screen::PasswordPrompt => draw_password_prompt(f, app),
        Screen::Main => draw_main(f, app),
        Screen::EntryDetail => draw_entry_detail(f, app),
        Screen::AddEntry => draw_add_entry(f, app),
        Screen::ConfirmDelete => draw_confirm_delete(f, app),
    }

    // Overlay status bar at the bottom for transient messages
    if let Some((msg, _)) = &app.status_message {
        let area = Rect {
            x: 0,
            y: f.area().height.saturating_sub(1),
            width: f.area().width,
            height: 1,
        };
        let status = Paragraph::new(Text::styled(
            msg.clone(),
            Style::default()
                .fg(Color::Green)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));
        f.render_widget(Clear, area);
        f.render_widget(status, area);
    }
}

// ── Password Prompt ─────────────────────────────────────────────

fn draw_password_prompt(f: &mut Frame, app: &mut TuiApp) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .vertical_margin(area.height / 3)
        .horizontal_margin(4)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    let title = Paragraph::new(Text::styled(
        "🔐  Wallet  —  Password Required",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(title, chunks[0]);

    let display: String = if app.password_visible {
        app.password_input.clone()
    } else {
        "\u{2022}".repeat(app.password_input.chars().count())
    };

    let input = Paragraph::new(display)
        .block(
            Block::default()
                .title("Master Password")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.password_error.is_empty() {
                    Color::White
                } else {
                    Color::Red
                })),
        )
        .style(Style::default().fg(Color::White));

    let inner = chunks[1];
    f.render_widget(input, inner);

    // Cursor for password input — always at end of content
    let cursor_offset = app.password_input.chars().count() as u16;
    let cursor_x = inner.x + 1 + cursor_offset;
    let cursor_y = inner.y + 1;
    f.set_cursor_position((cursor_x.min(inner.x + inner.width.saturating_sub(2)), cursor_y));

    let mut hints = vec![Line::from(Span::styled(
        "[Enter] submit  •  [Tab] toggle visibility  •  [Esc] quit",
        Style::default().fg(Color::DarkGray),
    ))];

    if !app.password_error.is_empty() {
        hints.push(Line::from(Span::styled(
            &app.password_error,
            Style::default().fg(Color::Red),
        )));
    }

    let help = Paragraph::new(Text::from(hints))
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(help, chunks[2]);
}

// ── Main Screen ─────────────────────────────────────────────────

fn draw_main(f: &mut Frame, app: &mut TuiApp) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        " Wallet  ".bold().cyan(),
        Span::raw("  "),
        "v1.0.0".dark_gray(),
        Span::raw("  │  "),
        format!("{} entries", app.keys.len()).dark_gray(),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Body: key list with fixed-length value mask (security: don't reveal length)
    let body_area = chunks[1];
    if app.keys.is_empty() {
        let empty = Paragraph::new(Text::styled(
            "  Wallet is empty\n  Press 'a' to add an entry",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty, body_area);
    } else {
        let items: Vec<ListItem> = app
            .keys
            .iter()
            .map(|k| {
                let masked: String = std::iter::repeat('\u{2022}').take(8).collect();
                ListItem::new(Line::from(vec![
                    Span::styled(k.clone(), Style::default().fg(Color::Cyan).bold()),
                    Span::raw("  "),
                    Span::styled(masked, Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        f.render_stateful_widget(list, body_area, &mut app.list_state);
    }

    // Footer — height 1, no border
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [\u{2191}\u{2193}/j/k] nav", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[Enter] view", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[a] add", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[d] delete", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[c] copy", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(footer, chunks[2]);
}

// ── Entry Detail (popup overlay) ────────────────────────────────

fn draw_entry_detail(f: &mut Frame, app: &mut TuiApp) {
    // Dimmed background
    let area = f.area();

    // Popup
    let popup = centered_rect(60, 10, area);
    let clear = Clear;
    f.render_widget(clear, popup);

    let (ref key, ref value) = match app.selected_key_value() {
        Some(kv) => kv,
        None => {
            app.screen = Screen::Main;
            return;
        }
    };

    let display_value = if app.show_value {
        value.clone()
    } else {
        "\u{2022}".repeat(8)
    };

    let key_span = Span::styled(key.clone(), Style::default().fg(Color::Cyan).bold());
    let value_span = Span::styled(display_value, Style::default().fg(Color::White));

    let inner = Paragraph::new(vec![
        Line::from(Span::styled("Entry Detail", Style::default().fg(Color::Yellow).bold())),
        Line::from(""),
        Line::from(vec![Span::raw("Key:    "), key_span]),
        Line::from(vec![Span::raw("Value:  "), value_span]),
        Line::from(""),
        Line::from(Span::styled(
            "[Tab/h] show/hide  •  [c] copy  •  [Esc] back",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(" Entry ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    )
    .wrap(ratatui::widgets::Wrap { trim: false });

    f.render_widget(inner, popup);
}

// ── Add Entry (popup overlay) ───────────────────────────────────

fn draw_add_entry(f: &mut Frame, app: &mut TuiApp) {
    let area = f.area();

    let popup = centered_rect(50, 9, area);
    let clear = Clear;
    f.render_widget(clear, popup);

    // Build the form
    let key_display: String = app.add_key.clone();
    let value_display: String = if app.add_value_visible {
        app.add_value.clone()
    } else {
        "\u{2022}".repeat(app.add_value.chars().count())
    };

    // Highlight active field indicator
    let key_indicator = if !app.add_value_visible { " ◀" } else { "" };
    let value_indicator = if app.add_value_visible { " ◀" } else { "" };

    let key_line = Line::from(vec![
        Span::styled(" Key:  ", Style::default().fg(Color::Cyan)),
        Span::raw(&key_display),
        Span::styled(key_indicator, Style::default().fg(Color::Green).bold()),
    ]);
    let value_line = Line::from(vec![
        Span::styled("Value: ", Style::default().fg(Color::Cyan)),
        Span::raw(&value_display),
        Span::styled(value_indicator, Style::default().fg(Color::Green).bold()),
    ]);

    let mut lines = vec![
        Line::from(Span::styled("Add Entry", Style::default().fg(Color::Yellow).bold())),
        Line::from(""),
        key_line,
        value_line,
    ];

    if let Some(ref err) = app.error_message {
        lines.push(Line::from(Span::styled(err, Style::default().fg(Color::Red))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Tab] switch field  •  [Enter] save  •  [Esc] cancel",
        Style::default().fg(Color::DarkGray),
    )));

    let inner = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" New Entry ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(inner, popup);

    // Cursor in the active field
    let field_len = if !app.add_value_visible {
        app.add_key.chars().count()
    } else {
        app.add_value.chars().count()
    } as u16;
    let cursor_x = popup.x + 7 + field_len;
    // Line 3 = header (0) + blank (1) + key line (2) = y: 3
    let cursor_y = popup.y + 3;
    f.set_cursor_position((cursor_x.min(popup.x + popup.width.saturating_sub(3)), cursor_y));
}

// ── Confirm Delete (popup overlay) ──────────────────────────────

fn draw_confirm_delete(f: &mut Frame, app: &mut TuiApp) {
    let area = f.area();

    let popup = centered_rect(46, 7, area);
    let clear = Clear;
    f.render_widget(clear, popup);

    let key_name = app.selected_key.as_deref().unwrap_or("?");
    let inner = Paragraph::new(vec![
        Line::from(Span::styled(
            "Confirm Delete",
            Style::default().fg(Color::Red).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Are you sure you want to delete '{key_name}'?"),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[y] yes  •  [n] no  •  [Esc] cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(" ⚠ Delete ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    f.render_widget(inner, popup);
}

// ── Helpers ─────────────────────────────────────────────────────

/// Create a centered rect within the given area
fn centered_rect(width_percent: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width_percent)) / 2),
            Constraint::Length(width_percent),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}
