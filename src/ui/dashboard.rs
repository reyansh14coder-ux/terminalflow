use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use super::components::{gradient_text, centered_rect, Header, InfoBox, StatusBar, TabsWidget};
use super::theme::Theme;
use crate::git::status::GitStatus;

pub async fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme::cyberpunk();
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| {
            ui(f, &app, &theme);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            break;
                        }
                        if app.quit_confirm {
                            break;
                        }
                        app.quit_confirm = true;
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if app.quit_confirm {
                            break;
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        if app.quit_confirm {
                            app.quit_confirm = false;
                        }
                    }
                    KeyCode::Tab => {
                        app.next_tab();
                        app.quit_confirm = false;
                    }
                    KeyCode::BackTab => {
                        app.prev_tab();
                        app.quit_confirm = false;
                    }
                    KeyCode::Char('g') | KeyCode::Char('G') => {
                        app.selected_tab = 0;
                        app.quit_confirm = false;
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        app.selected_tab = 1;
                        app.quit_confirm = false;
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        app.selected_tab = 2;
                        app.quit_confirm = false;
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        app.selected_tab = 3;
                        app.quit_confirm = false;
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        app.selected_tab = 4;
                        app.quit_confirm = false;
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

struct App {
    selected_tab: usize,
    quit_confirm: bool,
    git_status: Option<GitStatus>,
    tick_count: u64,
}

impl App {
    fn new() -> Self {
        Self {
            selected_tab: 0,
            quit_confirm: false,
            git_status: None,
            tick_count: 0,
        }
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % 5;
    }

    fn prev_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = 4;
        } else {
            self.selected_tab -= 1;
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
        // Simulate loading git status
        if self.git_status.is_none() && self.tick_count > 2 {
            self.git_status = Some(GitStatus::mock());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App, theme: &Theme) {
    let size = f.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(3),  // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(3),  // Status bar
        ])
        .split(size);

    // Header
    let header = Header::new("TerminalFlow", "2.0.0", theme);
    header.render(f, chunks[0]);

    // Tabs
    let tabs = vec!["📊 Git", "🐳 Docker", "🧪 Tests", "📈 Monitor", "🤖 AI"];
    let tabs_widget = TabsWidget::new(tabs, app.selected_tab, theme);
    tabs_widget.render(f, chunks[1]);

    // Content
    match app.selected_tab {
        0 => render_git_tab(f, chunks[2], app, theme),
        1 => render_docker_tab(f, chunks[2], theme),
        2 => render_test_tab(f, chunks[2], theme),
        3 => render_monitor_tab(f, chunks[2], theme),
        4 => render_ai_tab(f, chunks[2], theme),
        _ => {}
    }

    // Status bar
    let status = StatusBar::new(theme)
        .add_item("Project", "terminalflow", theme.style_primary())
        .add_item("Commits", "23 today", theme.style_success())
        .add_item("Uptime", "4h 23m", theme.style_accent())
        .add_item("[?] Help", "", theme.style_dim());
    status.render(f, chunks[3]);

    // Quit confirmation popup
    if app.quit_confirm {
        let popup = centered_rect(40, 20, size);
        let block = Block::default()
            .title(Span::styled(
                " ⚠️ Quit TerminalFlow? ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning));

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Are you sure you want to quit?",
                theme.style_text(),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [Y] Yes  ", theme.style_success()),
                Span::styled("[N] No   ", theme.style_error()),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, popup);
    }
}

fn render_git_tab(f: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Git status box
    let git_info = InfoBox::new("🔥 Git Status", theme)
        .border_color(theme.success)
        .add_line(Line::from(vec![
            Span::styled("  Branch: ", theme.style_dim()),
            Span::styled("main", theme.style_primary()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Modified: ", theme.style_dim()),
            Span::styled("5 files", theme.style_warning()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Added: ", theme.style_dim()),
            Span::styled("2 files", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Ready: ", theme.style_dim()),
            Span::styled("YES ✅", theme.style_success()),
        ]));

    git_info.render(f, chunks[0]);

    // Quick actions box
    let actions = InfoBox::new("⚡ Quick Actions", theme)
        .border_color(theme.accent)
        .add_line(Line::from(vec![
            Span::styled("  [c] Commit  ", theme.style_primary()),
            Span::styled("[p] Push", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  [l] Log     ", theme.style_secondary()),
            Span::styled("[d] Diff", theme.style_warning()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  [b] Branch  ", theme.style_accent()),
            Span::styled("[s] Stash", theme.style_dim()),
        ]));

    actions.render(f, chunks[1]);
}

fn render_docker_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    let block = InfoBox::new("🐳 Docker Containers", theme)
        .border_color(theme.primary)
        .add_line(Line::from(vec![
            Span::styled("  ✅ ", theme.style_success()),
            Span::styled("nginx:latest", theme.style_text()),
            Span::styled(" - Running", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  ✅ ", theme.style_success()),
            Span::styled("postgres:15", theme.style_text()),
            Span::styled(" - Running", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  ⏹️ ", theme.style_warning()),
            Span::styled("redis:7", theme.style_text()),
            Span::styled(" - Stopped", theme.style_warning()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  [s] Start  ", theme.style_primary()),
            Span::styled("[x] Stop   ", theme.style_error()),
            Span::styled("[r] Restart", theme.style_success()),
        ]));

    block.render(f, area);
}

fn render_test_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    let block = InfoBox::new("🧪 Test Results", theme)
        .border_color(theme.success)
        .add_line(Line::from(vec![
            Span::styled("  ✅ ", theme.style_success()),
            Span::styled("test_parse_config", theme.style_text()),
            Span::styled(" ... passed", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  ✅ ", theme.style_success()),
            Span::styled("test_ai_integration", theme.style_text()),
            Span::styled(" ... passed", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  ❌ ", theme.style_error()),
            Span::styled("test_network_timeout", theme.style_text()),
            Span::styled(" ... failed", theme.style_error()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  Total: ", theme.style_dim()),
            Span::styled("3", theme.style_text()),
            Span::styled(" | Passed: ", theme.style_dim()),
            Span::styled("2", theme.style_success()),
            Span::styled(" | Failed: ", theme.style_dim()),
            Span::styled("1", theme.style_error()),
        ]));

    block.render(f, area);
}

fn render_monitor_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    let block = InfoBox::new("📈 System Monitor", theme)
        .border_color(theme.secondary)
        .add_line(Line::from(vec![
            Span::styled("  CPU:     ", theme.style_dim()),
            Span::styled("████████░░ 78%", theme.style_warning()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Memory:  ", theme.style_dim()),
            Span::styled("██████░░░░ 62%", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Disk:    ", theme.style_dim()),
            Span::styled("███░░░░░░░ 34%", theme.style_success()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  Network: ", theme.style_dim()),
            Span::styled("↑ 1.2 MB/s  ↓ 3.4 MB/s", theme.style_primary()),
        ]));

    block.render(f, area);
}

fn render_ai_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    let block = InfoBox::new("🤖 AI Assistant", theme)
        .border_color(theme.accent)
        .add_line(Line::from(vec![
            Span::styled("  💡 Tip: ", theme.style_accent()),
            Span::styled("Ask me anything!", theme.style_text()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  Try: ", theme.style_dim()),
            Span::styled("\"Explain async/await\"", theme.style_primary()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Try: ", theme.style_dim()),
            Span::styled("\"Fix this error: ...\"", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Try: ", theme.style_dim()),
            Span::styled("\"Write a commit message\"", theme.style_warning()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  [Enter] Ask  ", theme.style_primary()),
            Span::styled("[↑↓] History", theme.style_dim()),
        ]));

    block.render(f, area);
}
