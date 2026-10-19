use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{CrosstermBackend, TestBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use super::components::{centered_rect, Header, InfoBox, StatusBar, TabsWidget};
use super::theme::Theme;
use crate::commands::docker::{self, Container};
use crate::commands::test::{self, TestSummary};
use crate::git::status::GitStatus;
use crate::process::{ProcessMonitor, SystemStats};

pub const TABS: [&str; 5] = ["📊 Git", "🐳 Docker", "🧪 Tests", "📈 Monitor", "🤖 AI"];

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
            let size = f.size();
            ui(f, &app, &theme, size);
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
                        app.select_tab(0);
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        app.select_tab(1);
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        app.select_tab(2);
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        app.select_tab(3);
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        app.select_tab(4);
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if app.selected_tab == 2 {
                            app.start_tests();
                        } else if app.selected_tab == 1 {
                            app.docker_loaded = false;
                        } else {
                            app.refresh_git();
                        }
                    }
                    KeyCode::Esc => app.quit_confirm = false,
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

enum TestState {
    NotRun,
    Running,
    Done(TestSummary),
}

struct App {
    selected_tab: usize,
    quit_confirm: bool,
    started: Instant,
    tick_count: u64,
    project: String,
    git: Option<GitStatus>,
    git_error: Option<String>,
    monitor: ProcessMonitor,
    stats: SystemStats,
    docker: Option<Vec<Container>>,
    docker_error: Option<String>,
    docker_loaded: bool,
    tests: TestState,
    test_rx: Option<mpsc::Receiver<TestSummary>>,
    ai_key: bool,
}

impl App {
    fn new() -> Self {
        let project = std::env::current_dir()
            .ok()
            .and_then(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "workspace".to_string());

        let mut monitor = ProcessMonitor::new();
        monitor.refresh_quick().ok();
        let stats = monitor.get_stats().clone();

        let git = GitStatus::from_repo(Path::new("."));
        let (git, git_error) = match git {
            Ok(status) => (Some(status), None),
            Err(err) => (None, Some(err.to_string())),
        };

        Self {
            selected_tab: 0,
            quit_confirm: false,
            started: Instant::now(),
            tick_count: 0,
            project,
            git,
            git_error,
            monitor,
            stats,
            docker: None,
            docker_error: None,
            docker_loaded: false,
            tests: TestState::NotRun,
            test_rx: None,
            ai_key: crate::ai::client::has_api_key(),
        }
    }

    fn select_tab(&mut self, tab: usize) {
        self.selected_tab = tab;
        self.quit_confirm = false;
    }

    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % TABS.len();
    }

    fn prev_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = TABS.len() - 1;
        } else {
            self.selected_tab -= 1;
        }
    }

    fn uptime(&self) -> String {
        let elapsed = self.started.elapsed().as_secs();
        let minutes = elapsed / 60;
        let seconds = elapsed % 60;
        if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    fn refresh_git(&mut self) {
        match GitStatus::from_repo(Path::new(".")) {
            Ok(status) => self.git = Some(status),
            Err(err) => {
                self.git = None;
                self.git_error = Some(err.to_string());
            }
        }
    }

    fn load_docker(&mut self) {
        self.docker_loaded = true;
        match docker::list_containers() {
            Ok(containers) => {
                self.docker = Some(containers);
                self.docker_error = None;
            }
            Err(err) => {
                self.docker = None;
                self.docker_error = Some(err.to_string());
            }
        }
    }

    fn start_tests(&mut self) {
        if matches!(self.tests, TestState::Running) {
            return;
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(test::run_suite());
        });
        self.test_rx = Some(rx);
        self.tests = TestState::Running;
    }

    fn poll_tests(&mut self) {
        let received = match &self.test_rx {
            Some(rx) => rx.try_recv().ok(),
            None => None,
        };
        if let Some(summary) = received {
            self.tests = TestState::Done(summary);
            self.test_rx = None;
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;

        if self.tick_count % 4 == 1 && self.monitor.refresh_quick().is_ok() {
            self.stats = self.monitor.get_stats().clone();
        }

        if self.tick_count == 1 || self.tick_count % 8 == 0 {
            self.refresh_git();
        }

        if self.selected_tab == 1 && !self.docker_loaded {
            self.load_docker();
        }

        if self.selected_tab == 2 {
            if matches!(self.tests, TestState::NotRun) {
                self.start_tests();
            } else {
                self.poll_tests();
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App, theme: &Theme, size: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(size);

    let ai_hint = if app.ai_key {
        "⚡ AI Mode: ON".to_string()
    } else {
        "⚡ AI Mode: OFF (set OPENAI_API_KEY)".to_string()
    };
    let header = Header::new("TerminalFlow", env!("CARGO_PKG_VERSION"), ai_hint, theme);
    header.render(f, chunks[0]);

    let tabs_widget = TabsWidget::new(TABS.to_vec(), app.selected_tab, theme);
    tabs_widget.render(f, chunks[1]);

    match app.selected_tab {
        0 => render_git_tab(f, chunks[2], theme, app),
        1 => render_docker_tab(f, chunks[2], theme, app),
        2 => render_test_tab(f, chunks[2], theme, app),
        3 => render_monitor_tab(f, chunks[2], theme, app),
        4 => render_ai_tab(f, chunks[2], theme, app),
        _ => {}
    }

    let status = StatusBar::new(theme)
        .add_item("Project", app.project.clone(), theme.style_primary())
        .add_item(
            "Branch",
            app.git
                .as_ref()
                .map(|g| g.branch.clone())
                .unwrap_or_else(|| "-".to_string()),
            theme.style_secondary(),
        )
        .add_item(
            "CPU",
            format!("{:.1}%", app.stats.cpu_percent()),
            theme.style_accent(),
        )
        .add_item("Session", app.uptime(), theme.style_success())
        .add_item("[Tab] Next [r] Refresh [q] Quit", "", theme.style_dim());
    status.render(f, chunks[3]);

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

fn file_line(label: &str, count: usize, names: &[String], theme: &Theme, style: Style) -> Line<'static> {
    if count == 0 {
        return Line::from(vec![
            Span::styled(format!("  {}: ", label), theme.style_dim()),
            Span::styled("0".to_string(), theme.style_dim()),
        ]);
    }

    let sample: Vec<String> = names.iter().take(2).cloned().collect();
    let more = count.saturating_sub(sample.len());

    let mut spans = vec![
        Span::styled(format!("  {}: ", label), theme.style_dim()),
        Span::styled(count.to_string(), style),
    ];
    if !sample.is_empty() {
        spans.push(Span::styled("  ", theme.style_dim()));
        spans.push(Span::styled(sample.join(", "), theme.style_text()));
    }
    if more > 0 {
        spans.push(Span::styled(format!(" (+{} more)", more), theme.style_dim()));
    }
    Line::from(spans)
}

fn render_git_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let mut git_box = InfoBox::new("🔥 Git Status", theme).border_color(theme.success);

    match &app.git {
        Some(git) => {
            git_box = git_box
                .add_line(Line::from(vec![
                    Span::styled("  Branch: ", theme.style_dim()),
                    Span::styled(git.branch.clone(), theme.style_primary()),
                    Span::styled(
                        format!("  ↑{} ↓{}", git.ahead, git.behind),
                        theme.style_dim(),
                    ),
                ]))
                .add_line(file_line(
                    "Staged",
                    git.staged.len(),
                    &git.staged,
                    theme,
                    theme.style_success(),
                ))
                .add_line(file_line(
                    "Modified",
                    git.modified.len(),
                    &git.modified,
                    theme,
                    theme.style_warning(),
                ))
                .add_line(file_line(
                    "New",
                    git.untracked.len(),
                    &git.untracked,
                    theme,
                    theme.style_secondary(),
                ))
                .add_line(file_line(
                    "Deleted",
                    git.deleted.len(),
                    &git.deleted,
                    theme,
                    theme.style_error(),
                ))
                .add_line(Line::from(""))
                .add_line(if git.is_clean() {
                    Line::from(vec![
                        Span::styled("  Working tree: ", theme.style_dim()),
                        Span::styled("clean — ready to commit ✅", theme.style_success()),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled("  Working tree: ", theme.style_dim()),
                        Span::styled(
                            format!("{} change(s) pending", git.total_changes()),
                            theme.style_warning(),
                        ),
                    ])
                });
        }
        None => {
            let message = app
                .git_error
                .clone()
                .unwrap_or_else(|| "Not a git repository".to_string());
            git_box = git_box
                .add_line(Line::from(vec![
                    Span::styled("  ⚠️  ", theme.style_warning()),
                    Span::styled(truncate(&message, 54), theme.style_text()),
                ]))
                .add_line(Line::from(""))
                .add_line(Line::from(vec![
                    Span::styled("  Run this inside a repository: ", theme.style_dim()),
                    Span::styled("git init", theme.style_primary()),
                ]));
        }
    }

    git_box.render(f, chunks[0]);

    let actions = InfoBox::new("⚡ Quick Actions", theme)
        .border_color(theme.accent)
        .add_line(Line::from(vec![
            Span::styled("  [g] Git     ", theme.style_primary()),
            Span::styled("[d] Docker ", theme.style_success()),
            Span::styled("[t] Tests", theme.style_secondary()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  [m] Monitor", theme.style_warning()),
            Span::styled("  [a] AI    ", theme.style_accent()),
            Span::styled("[Tab] Next", theme.style_dim()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  [r] Refresh panel", theme.style_dim()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  CLI equivalents:", theme.style_dim()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("    tf status · tf commit · tf review", theme.style_text()),
        ]));

    actions.render(f, chunks[1]);
}

fn render_docker_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme, app: &App) {
    let mut block = InfoBox::new("🐳 Docker Containers", theme).border_color(theme.primary);

    if !app.docker_loaded {
        block = block.add_line(Line::from(vec![
            Span::styled("  ⏳ ", theme.style_dim()),
            Span::styled("Querying docker…", theme.style_text()),
        ]));
    } else if let Some(containers) = &app.docker {
        if containers.is_empty() {
            block = block.add_line(Line::from(vec![
                Span::styled("  ℹ️  ", theme.style_dim()),
                Span::styled("No containers found", theme.style_text()),
            ]));
        }

        for container in containers.iter().take(8) {
            let (icon, style) = if container.running {
                ("  ✅ ", theme.style_success())
            } else {
                ("  ⏹️  ", theme.style_warning())
            };
            block = block.add_line(Line::from(vec![
                Span::styled(icon, style),
                Span::styled(container.name.clone(), theme.style_text()),
                Span::styled(format!("  {}", container.image), theme.style_dim()),
                Span::styled(
                    format!("  — {}", container.status),
                    if container.running {
                        theme.style_success()
                    } else {
                        theme.style_warning()
                    },
                ),
            ]));
        }

        block = block
            .add_line(Line::from(""))
            .add_line(Line::from(vec![
                Span::styled("  Total: ", theme.style_dim()),
                Span::styled(containers.len().to_string(), theme.style_text()),
                Span::styled(
                    format!(
                        "  running: {}",
                        containers.iter().filter(|c| c.running).count()
                    ),
                    theme.style_success(),
                ),
            ]));
    } else if let Some(error) = &app.docker_error {
        block = block
            .add_line(Line::from(vec![
                Span::styled("  ⚠️  ", theme.style_warning()),
                Span::styled(truncate(error, 54), theme.style_text()),
            ]))
            .add_line(Line::from(""))
            .add_line(Line::from(vec![
                Span::styled("  Docker is optional — everything else works without it.", theme.style_dim()),
            ]));
    }

    block = block.add_line(Line::from("")).add_line(Line::from(vec![
        Span::styled("  [r] Refresh   ", theme.style_primary()),
        Span::styled("tf docker", theme.style_dim()),
    ]));

    block.render(f, area);
}

fn render_test_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme, app: &App) {
    let mut block = InfoBox::new("🧪 Test Results", theme).border_color(theme.success);

    match &app.tests {
        TestState::NotRun => {
            block = block
                .add_line(Line::from(vec![
                    Span::styled("  ⏳ ", theme.style_dim()),
                    Span::styled("Preparing test run…", theme.style_text()),
                ]))
                .add_line(Line::from(""))
                .add_line(Line::from(vec![
                    Span::styled("  Detects: ", theme.style_dim()),
                    Span::styled("cargo · npm · pytest · go", theme.style_secondary()),
                ]));
        }
        TestState::Running => {
            block = block
                .add_line(Line::from(vec![
                    Span::styled("  ⏳ ", theme.style_accent()),
                    Span::styled("Running the test suite…", theme.style_text()),
                ]))
                .add_line(Line::from(""))
                .add_line(Line::from(vec![
                    Span::styled("  This runs in the background — results appear here.", theme.style_dim()),
                ]));
        }
        TestState::Done(summary) => {
            if let Some(error) = &summary.error {
                block = block
                    .add_line(Line::from(vec![
                        Span::styled("  ⚠️  ", theme.style_warning()),
                        Span::styled(truncate(error, 54), theme.style_text()),
                    ]))
                    .add_line(Line::from(""))
                    .add_line(Line::from(vec![
                        Span::styled("  [r] ", theme.style_primary()),
                        Span::styled("Retry", theme.style_text()),
                    ]));
            } else {
                let shown = summary.results.iter().take(9);
                for (name, passed) in shown {
                    if *passed {
                        block = block.add_line(Line::from(vec![
                            Span::styled("  ✅ ", theme.style_success()),
                            Span::styled(truncate(name, 44), theme.style_text()),
                            Span::styled(" ... passed", theme.style_success()),
                        ]));
                    } else {
                        block = block.add_line(Line::from(vec![
                            Span::styled("  ❌ ", theme.style_error()),
                            Span::styled(truncate(name, 44), theme.style_text()),
                            Span::styled(" ... failed", theme.style_error()),
                        ]));
                    }
                }

                if summary.results.len() > 9 {
                    block = block.add_line(Line::from(vec![
                        Span::styled(
                            format!("  … {} more", summary.results.len() - 9),
                            theme.style_dim(),
                        ),
                    ]));
                }

                block = block
                    .add_line(Line::from(""))
                    .add_line(Line::from(vec![
                        Span::styled("  Total: ", theme.style_dim()),
                        Span::styled(summary.total().to_string(), theme.style_text()),
                        Span::styled("  Passed: ", theme.style_dim()),
                        Span::styled(summary.passed.to_string(), theme.style_success()),
                        Span::styled("  Failed: ", theme.style_dim()),
                        Span::styled(summary.failed.to_string(), theme.style_error()),
                        Span::styled(
                            format!("  ({:.2}s)", summary.duration_secs),
                            theme.style_dim(),
                        ),
                    ]))
                    .add_line(Line::from(vec![
                        Span::styled("  Runner: ", theme.style_dim()),
                        Span::styled(summary.runner.clone(), theme.style_secondary()),
                        Span::styled("   [r] Re-run", theme.style_dim()),
                    ]));

                if summary.all_passed() {
                    block = block.add_line(Line::from(vec![
                        Span::styled("  🎉 ", theme.style_success()),
                        Span::styled("Everything is green", theme.style_success()),
                    ]));
                }
            }
        }
    }

    block.render(f, area);
}

fn bar(percent: f64, width: usize) -> String {
    let percent = percent.clamp(0.0, 100.0);
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(width.saturating_sub(filled))
    )
}

fn render_monitor_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme, app: &App) {
    let stats = &app.stats;
    let mut block = InfoBox::new("📈 System Monitor", theme).border_color(theme.secondary);

    let cpu_style = if stats.cpu_percent() > 80.0 {
        theme.style_error()
    } else if stats.cpu_percent() > 60.0 {
        theme.style_warning()
    } else {
        theme.style_success()
    };
    let mem_style = if stats.memory_percent() > 85.0 {
        theme.style_error()
    } else if stats.memory_percent() > 70.0 {
        theme.style_warning()
    } else {
        theme.style_success()
    };

    block = block
        .add_line(Line::from(vec![
            Span::styled("  CPU:    ", theme.style_dim()),
            Span::styled(
                format!("{} {:>5.1}%", bar(stats.cpu_percent(), 20), stats.cpu_percent()),
                cpu_style,
            ),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Memory: ", theme.style_dim()),
            Span::styled(
                format!(
                    "{} {:>5.1}%  ({} / {})",
                    bar(stats.memory_percent(), 20),
                    stats.memory_percent(),
                    crate::commands::monitor::format_bytes(stats.memory_used),
                    crate::commands::monitor::format_bytes(stats.memory_total),
                ),
                mem_style,
            ),
        ]))
        .add_line(Line::from(vec![
            Span::styled("  Disk:   ", theme.style_dim()),
            Span::styled(
                format!(
                    "{} {:>5.1}%  ({} / {})",
                    bar(stats.disk_percent(), 20),
                    stats.disk_percent(),
                    crate::commands::monitor::format_bytes(stats.disk_used),
                    crate::commands::monitor::format_bytes(stats.disk_total),
                ),
                theme.style_primary(),
            ),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  Uptime: ", theme.style_dim()),
            Span::styled(stats.uptime_human(), theme.style_text()),
            Span::styled("  Processes: ", theme.style_dim()),
            Span::styled(stats.process_count.to_string(), theme.style_text()),
        ]));

    let top = app.monitor.get_top_cpu(4);
    if !top.is_empty() {
        block = block.add_line(Line::from("")).add_line(Line::from(vec![
            Span::styled("  Top CPU:", theme.style_dim()),
        ]));
        for process in top {
            block = block.add_line(Line::from(vec![
                Span::styled("    ", theme.style_dim()),
                Span::styled(format!("{:<22}", crate::ui::dashboard::truncate(process.name.as_str(), 22)), theme.style_text()),
                Span::styled(format!("{:>5.1}%", process.cpu_usage), theme.style_warning()),
                Span::styled(
                    format!("   {}", crate::commands::monitor::format_bytes(process.memory_usage)),
                    theme.style_dim(),
                ),
            ]));
        }
    }

    block.render(f, area);
}

fn render_ai_tab(f: &mut ratatui::Frame, area: Rect, theme: &Theme, app: &App) {
    let mut block = InfoBox::new("🤖 AI Assistant", theme).border_color(theme.accent);

    if app.ai_key {
        block = block.add_line(Line::from(vec![
            Span::styled("  ✅ ", theme.style_success()),
            Span::styled("API key detected — AI features are live", theme.style_text()),
        ]));
    } else {
        block = block
            .add_line(Line::from(vec![
                Span::styled("  ⚠️  ", theme.style_warning()),
                Span::styled("No API key found — everything else still works.", theme.style_text()),
            ]))
            .add_line(Line::from(""))
            .add_line(Line::from(vec![
                Span::styled("  Enable AI with: ", theme.style_dim()),
                Span::styled("set OPENAI_API_KEY=sk-…", theme.style_primary()),
            ]));
    }

    block = block
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  From the shell:", theme.style_dim()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("    tf ai ", theme.style_primary()),
            Span::styled("\"Explain async/await\"", theme.style_text()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("    tf commit --ai", theme.style_success()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("    tf review src/", theme.style_secondary()),
        ]))
        .add_line(Line::from(vec![
            Span::styled("    tf fix", theme.style_warning()),
        ]))
        .add_line(Line::from(""))
        .add_line(Line::from(vec![
            Span::styled("  Offline fallbacks are built in — ", theme.style_dim()),
            Span::styled("review", theme.style_text()),
            Span::styled(" and ", theme.style_dim()),
            Span::styled("commit", theme.style_text()),
            Span::styled(" work without a key.", theme.style_dim()),
        ]));

    block.render(f, area);
}

/// Render every dashboard tab into a machine-readable snapshot used to
/// generate the README screenshots (`tf screenshot`).
pub fn snapshot_json(width: u16, height: u16) -> anyhow::Result<String> {
    let theme = Theme::cyberpunk();
    let mut app = App::new();
    app.start_tests();
    app.poll_tests();
    // Blocking run so the Tests screenshot shows real results.
    if matches!(app.tests, TestState::Running) {
        if let Some(rx) = app.test_rx.take() {
            if let Ok(summary) = rx.recv() {
                app.tests = TestState::Done(summary);
            }
        }
    }
    app.load_docker();

    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    let mut tabs = Vec::new();

    for tab in 0..TABS.len() {
        app.select_tab(tab);
        terminal.draw(|frame| {
            let size = frame.size();
            ui(frame, &app, &theme, size);
        })?;

        let buffer = terminal.backend().buffer();
        let area = buffer.area;
        let mut rows = Vec::new();

        for y in area.top()..area.bottom() {
            let mut cells = Vec::new();
            for x in area.left()..area.right() {
                let cell = buffer.get(x, y);
                cells.push(serde_json::json!({
                    "s": cell.symbol().to_string(),
                    "fg": color_to_json(cell.fg),
                    "bg": color_to_json(cell.bg),
                    "b": cell.modifier.contains(Modifier::BOLD),
                }));
            }
            rows.push(serde_json::Value::Array(cells));
        }

        tabs.push(serde_json::json!({
            "name": TABS[tab],
            "rows": rows,
        }));
    }

    Ok(serde_json::to_string(&serde_json::json!({
        "width": width,
        "height": height,
        "project": app.project,
        "tabs": tabs,
    }))?)
}

fn color_to_json(color: Color) -> serde_json::Value {
    match color {
        Color::Rgb(r, g, b) => {
            serde_json::Value::String(format!("#{:02x}{:02x}{:02x}", r, g, b))
        }
        Color::Reset => serde_json::Value::Null,
        other => serde_json::Value::String(format!("{:?}", other)),
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        let mut out: String = value.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
