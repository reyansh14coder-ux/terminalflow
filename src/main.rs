#![allow(clippy::all)]

mod ai;
mod api;
mod bisect;
mod codegen;
mod commands;
mod git;
mod http;
mod logs;
mod multiplexer;
mod plugins;
mod process;
mod secrets;
mod ssh;
mod ui;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "terminalflow",
    about = "🚀 AI-Powered Terminal Dashboard for Developers - The Ultimate Dev Toolkit",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive dashboard
    Dashboard,

    /// Render the dashboard tabs to JSON (used to build the README screenshots)
    Screenshot {
        /// Where to write the snapshot JSON
        #[arg(short, long, default_value = "assets/dashboard.json")]
        out: std::path::PathBuf,
        /// Snapshot width in terminal cells
        #[arg(long, default_value_t = 110)]
        width: u16,
        /// Snapshot height in terminal cells
        #[arg(long, default_value_t = 34)]
        height: u16,
    },

    /// Ask AI a question
    Ai {
        /// Your question
        #[arg(trailing_var_arg = true)]
        question: Vec<String>,
    },

    /// Auto-fix the last error
    Fix,

    /// Generate commit message with AI
    Commit {
        /// Use AI to generate message
        #[arg(long)]
        ai: bool,
    },

    /// Review code
    Review {
        /// Commit or file to review
        #[arg(default_value = "HEAD")]
        target: String,
    },

    /// Show Git status with style
    Status,

    /// Manage Docker containers
    Docker,

    /// Run tests with beautiful output
    Test,

    /// System monitor
    Monitor,

    /// Plugin management
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Secret manager
    Secret {
        #[command(subcommand)]
        command: SecretCommands,
    },

    /// SSH tunnel management
    Ssh {
        #[command(subcommand)]
        command: SshCommands,
    },

    /// HTTP client
    Http {
        /// HTTP method (GET, POST, PUT, PATCH, DELETE) — or the URL, to default to GET
        #[arg(value_name = "METHOD_OR_URL")]
        method_or_url: String,

        /// URL (required when a method is given first)
        #[arg(value_name = "URL")]
        url: Option<String>,

        /// Request body
        #[arg(short, long)]
        body: Option<String>,

        /// Request header, e.g. -H "Accept: application/json" (repeatable)
        #[arg(short = 'H', long = "header")]
        header: Vec<String>,

        /// Request timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// API testing
    Api {
        #[command(subcommand)]
        command: ApiCommands,
    },

    /// Process monitor
    Ps {
        /// Filter by name
        #[arg(short, long)]
        filter: Option<String>,

        /// Show top CPU processes
        #[arg(short = 'c', long)]
        top_cpu: bool,

        /// Show top memory processes
        #[arg(short = 'm', long)]
        top_mem: bool,
    },

    /// Kill a process
    Kill {
        /// Process ID
        pid: u32,
    },

    /// Watch files for changes
    Watch {
        /// Path to watch
        path: String,

        /// Command to run on change
        #[arg(short, long)]
        command: Option<String>,
    },

    /// View and analyze logs
    Logs {
        /// Log file path
        file: String,

        /// Filter by level
        #[arg(short, long)]
        level: Option<String>,

        /// Search pattern
        #[arg(short, long)]
        search: Option<String>,

        /// Tail last N lines
        #[arg(short, long, default_value = "50")]
        tail: usize,
    },

    /// Generate code
    Gen {
        #[command(subcommand)]
        command: GenCommands,
    },

    /// Git bisect automation
    Bisect {
        #[command(subcommand)]
        command: BisectCommands,
    },

    /// Manage secrets vault
    Vault {
        #[command(subcommand)]
        command: VaultCommands,
    },

    /// Start API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List installed plugins
    List,
    /// Install a plugin
    Install { source: String },
    /// Uninstall a plugin
    Uninstall { name: String },
    /// Search plugins
    Search { query: String },
}

#[derive(Subcommand)]
enum SecretCommands {
    /// Set a secret
    Set { name: String, value: String },
    /// Get a secret
    Get { name: String },
    /// Delete a secret
    Delete { name: String },
    /// List all secrets
    List,
    /// Import from environment
    Import { prefix: String },
}

#[derive(Subcommand)]
enum SshCommands {
    /// Connect to a host
    Connect { name: String },
    /// List connections
    List,
    /// Add a connection
    Add {
        name: String,
        host: String,
        user: String,
    },
    /// Start a tunnel
    Tunnel {
        #[arg(short, long)]
        local_port: u16,
        #[arg(short, long)]
        remote_host: String,
        #[arg(short, long)]
        remote_port: u16,
        #[arg(short, long)]
        ssh_host: String,
    },
}

#[derive(Subcommand)]
enum ApiCommands {
    /// Run API tests
    Test { file: String },
    /// Create a test
    Create { name: String },
}

#[derive(Subcommand)]
enum GenCommands {
    /// Generate a struct
    Struct { name: String },
    /// Generate an API endpoint
    Endpoint { method: String, path: String },
    /// Generate tests
    Tests { name: String },
}

#[derive(Subcommand)]
enum BisectCommands {
    /// Start bisect
    Start { good: String, bad: String },
    /// Run test
    Test,
    /// Reset
    Reset,
}

#[derive(Subcommand)]
enum VaultCommands {
    /// Unlock the vault with your password
    Unlock,
    /// Lock the vault and forget the session key
    Lock,
    /// List secrets stored in the vault
    List,
    /// Store a secret in the vault (needs an unlocked vault)
    Store {
        name: String,
        /// Value to store — read from stdin when omitted
        value: Option<String>,
    },
    /// Read a secret from the vault
    Get { name: String },
    /// Back the vault up to a single file (entries stay encrypted)
    Backup {
        /// Where to write the backup
        path: Option<std::path::PathBuf>,
    },
    /// Restore entries from a backup file
    Restore { path: std::path::PathBuf },
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

fn format_mb(bytes: u64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * MB;
    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else {
        format!("{:.1} MB", bytes / MB)
    }
}

fn stdin_is_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal()
}

/// Read a value: hidden prompt on a TTY, raw stdin when piped.
fn read_secret_input(prompt: &str) -> Result<String> {
    use std::io::Write;

    if stdin_is_terminal() {
        eprint!("{}: ", prompt);
        let _ = std::io::stderr().flush();

        if let Ok(line) = console::Term::stdout().read_secure_line() {
            return Ok(line.trim().to_string());
        }
        println!();
    }

    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Password for the vault: `TF_VAULT_PASSWORD` (CI) wins, then a hidden prompt.
fn read_password() -> Result<String> {
    if let Ok(password) = std::env::var("TF_VAULT_PASSWORD") {
        if !password.trim().is_empty() {
            return Ok(password);
        }
    }

    if !stdin_is_terminal() {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let line = line.trim().to_string();
        if line.is_empty() {
            anyhow::bail!("No password provided — pipe one in or set TF_VAULT_PASSWORD");
        }
        return Ok(line);
    }

    let password = read_secret_input("Password")?;
    if password.is_empty() {
        anyhow::bail!("Password must not be empty");
    }
    Ok(password)
}

fn vault_root() -> Result<std::path::PathBuf> {
    match dirs::home_dir() {
        Some(home) => Ok(home.join(".terminalflow")),
        None => Ok(std::path::PathBuf::from(".")),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Dashboard) | None => {
            ui::dashboard::run().await?;
        }
        Some(Commands::Screenshot { out, width, height }) => {
            let json = ui::dashboard::snapshot_json(width, height)?;
            if let Some(parent) = out.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&out, json)?;
            println!("📸 Wrote dashboard snapshot to {}", out.display());
        }
        Some(Commands::Ai { question }) => {
            let q = question.join(" ");
            ai::client::ask(&q).await?;
        }
        Some(Commands::Fix) => {
            commands::fix::auto_fix().await?;
        }
        Some(Commands::Commit { ai: use_ai }) => {
            commands::commit::generate(use_ai).await?;
        }
        Some(Commands::Review { target }) => {
            commands::review::review(&target).await?;
        }
        Some(Commands::Status) => {
            commands::status::show().await?;
        }
        Some(Commands::Docker) => {
            commands::docker::manage().await?;
        }
        Some(Commands::Test) => {
            commands::test::run().await?;
        }
        Some(Commands::Monitor) => {
            commands::monitor::show().await?;
        }
        Some(Commands::Plugin { command }) => match command {
            PluginCommands::List => {
                let mut manager = plugins::PluginManager::new();
                manager.discover_and_load()?;
                let installed = manager.list_plugins();

                if installed.is_empty() {
                    println!("ℹ️  No plugins installed yet");
                    println!();
                    println!(
                        "  {} {}",
                        "Browse:".dimmed(),
                        "tf plugin search git".cyan()
                    );
                    println!(
                        "  {} {}",
                        "Install:".dimmed(),
                        "tf plugin install <folder-or-git-url>".cyan()
                    );
                } else {
                    println!("{}", "🔌 Installed plugins".cyan().bold());
                    println!("{}", "─".repeat(50).dimmed());
                    for plugin in installed {
                        println!(
                            "  📦 {} v{}",
                            plugin.manifest.name.green().bold(),
                            plugin.manifest.version
                        );
                        if !plugin.manifest.description.is_empty() {
                            println!("     {}", plugin.manifest.description.dimmed());
                        }
                        if !plugin.manifest.commands.is_empty() {
                            let names: Vec<&str> = plugin
                                .manifest
                                .commands
                                .iter()
                                .map(|c| c.name.as_str())
                                .collect();
                            println!("     {} {}", "commands:".dimmed(), names.join(", ").cyan());
                        }
                    }
                }
            }
            PluginCommands::Install { source } => {
                let mut manager = plugins::PluginManager::new();
                manager.install_plugin(&source)?;
            }
            PluginCommands::Uninstall { name } => {
                let mut manager = plugins::PluginManager::new();
                manager.uninstall_plugin(&name)?;
            }
            PluginCommands::Search { query } => {
                let registry = plugins::PluginRegistry::new();
                let results = registry.search(&query);
                if results.is_empty() {
                    println!("ℹ️  No plugins matched '{}'", query);
                }
                for plugin in results {
                    println!(
                        "📦 {} - {} (⬇️ {} downloads)",
                        plugin.name, plugin.description, plugin.downloads
                    );
                }
            }
        },
        Some(Commands::Secret { command }) => {
            let mut manager = secrets::SecretManager::new()?;
            match command {
                SecretCommands::Set { name, value } => {
                    manager.set(&name, &value, None)?;
                    println!("✅ Secret '{}' set", name);
                }
                SecretCommands::Get { name } => {
                    if let Some(value) = manager.get(&name)? {
                        println!("🔑 {}={}", name, value);
                    } else {
                        println!("❌ Secret '{}' not found", name);
                    }
                }
                SecretCommands::Delete { name } => {
                    if manager.delete(&name)? {
                        println!("✅ Secret '{}' deleted", name);
                    } else {
                        println!("❌ Secret '{}' not found", name);
                    }
                }
                SecretCommands::List => {
                    let secrets = manager.list();
                    if secrets.is_empty() {
                        println!("ℹ️  No secrets stored yet");
                        println!();
                        println!(
                            "  {} {}",
                            "Store one:".dimmed(),
                            "tf secret set DATABASE_URL \"postgres://...\"".cyan()
                        );
                        println!(
                            "  {} {}",
                            "Import all:".dimmed(),
                            "tf secret import AWS_".cyan()
                        );
                    } else {
                        println!("{}", "🔐 Stored secrets".cyan().bold());
                        println!("{}", "─".repeat(50).dimmed());
                        for secret in secrets {
                            println!("  🔑 {}", secret.name.green());
                        }
                        println!("{}", "─".repeat(50).dimmed());
                        println!(
                            "  {} values are encrypted at rest in ~/.terminalflow/secrets",
                            "🔒".dimmed()
                        );
                    }
                }
                SecretCommands::Import { prefix } => {
                    let count = manager.import_env(&prefix)?;
                    println!("✅ Imported {} secrets", count);
                }
            }
        }
        Some(Commands::Ssh { command }) => {
            let mut manager = ssh::SSHManager::new()?;
            match command {
                SshCommands::Connect { name } => {
                    manager.connect(&name)?;
                }
                SshCommands::List => {
                    let connections = manager.list();
                    if connections.is_empty() {
                        println!("ℹ️  No SSH connections saved yet");
                        println!();
                        println!(
                            "  {} {}",
                            "Add one:".dimmed(),
                            "tf ssh add prod example.com deploy".cyan()
                        );
                        println!(
                            "  {} {}",
                            "Then connect:".dimmed(),
                            "tf ssh connect prod".cyan()
                        );
                    } else {
                        println!("{}", "🖥️  SSH connections".cyan().bold());
                        println!("{}", "─".repeat(50).dimmed());
                        for conn in connections {
                            println!(
                                "  🖥️  {} — {}@{}:{}",
                                conn.name.green().bold(),
                                conn.user,
                                conn.host,
                                conn.port
                            );
                        }
                    }
                }
                SshCommands::Add { name, host, user } => {
                    let conn = ssh::SSHConnection {
                        name: name.clone(),
                        host,
                        port: 22,
                        user,
                        key_path: None,
                        tags: Vec::new(),
                    };
                    manager.add(conn)?;
                    println!("✅ Connection '{}' added", name);
                }
                SshCommands::Tunnel {
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_host,
                } => {
                    let mut tunnel = ssh::SSHTunnel::new(
                        local_port,
                        &remote_host,
                        remote_port,
                        &ssh_host,
                        "root",
                    );
                    tunnel.start()?;
                }
            }
        }
        Some(Commands::Http {
            method_or_url,
            url,
            body,
            header,
            timeout,
        }) => {
            let upper = method_or_url.to_uppercase();
            let methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

            let (method, url) = match url {
                Some(url) => {
                    if !methods.contains(&upper.as_str()) {
                        anyhow::bail!(
                            "Unknown HTTP method '{}'. Expected one of: {}",
                            method_or_url,
                            methods.join(", ")
                        );
                    }
                    (upper, url)
                }
                None => ("GET".to_string(), method_or_url),
            };

            let mut headers = std::collections::HashMap::new();
            for h in &header {
                match h.split_once(':') {
                    Some((k, v)) => {
                        headers.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    None => anyhow::bail!("Invalid header '{}'. Use -H \"Name: value\"", h),
                }
            }

            let client = http::HttpClient::new()?;
            let request = http::client::HttpRequest {
                method,
                url,
                body,
                headers,
                query: std::collections::HashMap::new(),
                timeout: Some(timeout),
                follow_redirects: true,
            };
            let response = client.execute(&request).await?;
            println!(
                "{} {} - {}ms",
                response.status, response.status_text, response.duration_ms
            );
            println!("{}", response.body);
        }
        Some(Commands::Api { command }) => match command {
            ApiCommands::Test { file } => {
                let mut tester = api::APITester::new()?;
                let count = tester.load_file(&file)?;
                if count == 0 {
                    println!("ℹ️  {} contains no tests", file);
                    println!(
                        "  {} {}",
                        "Create one:".dimmed(),
                        "tf api create example".cyan()
                    );
                } else {
                    println!("🧪 Running {} API test(s) from {}", count, file);
                    println!();
                    let results = tester.run_all().await;
                    tester.print_results(&results);
                    let failed = results.iter().filter(|r| !r.passed).count();
                    if failed > 0 {
                        anyhow::bail!("{} of {} API test(s) failed", failed, results.len());
                    }
                }
            }
            ApiCommands::Create { name } => {
                let path = api::APITester::write_template(&name)?;
                println!("📝 Created {}", path.cyan());
                println!(
                    "  {} {}",
                    "Next:".dimmed(),
                    format!("tf api test {}", path).cyan()
                );
            }
        },
        Some(Commands::Ps {
            filter,
            top_cpu,
            top_mem,
        }) => {
            let mut monitor = process::ProcessMonitor::new();
            monitor.refresh()?;

            let mut processes: Vec<process::ProcessInfo> = monitor.get_processes().to_vec();

            if top_cpu {
                processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            } else if top_mem {
                processes.sort_by(|a, b| {
                    b.memory_usage
                        .partial_cmp(&a.memory_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            } else {
                processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }

            if let Some(f) = filter {
                processes.retain(|p| {
                    p.name.to_lowercase().contains(&f.to_lowercase())
                        || p.command.to_lowercase().contains(&f.to_lowercase())
                        || p.pid.to_string() == f
                });
            }

            if processes.is_empty() {
                println!("{} No processes matched your filter", "ℹ️".blue());
                return Ok(());
            }

            println!("{}", "📊 Processes".cyan().bold());
            println!("{}", "─".repeat(70).dimmed());
            println!(
                "  {:<8} {:<24} {:>7} {:>10}  {}",
                "PID".dimmed(),
                "NAME".dimmed(),
                "CPU%".dimmed(),
                "MEMORY".dimmed(),
                "STATUS".dimmed()
            );

            let limit = if top_cpu || top_mem { 25 } else { 20 };
            for proc in processes.iter().take(limit) {
                let mem = format_mb(proc.memory_usage);
                let cpu_style = if proc.cpu_usage >= 50.0 {
                    format!("{:>6.1}", proc.cpu_usage).red()
                } else if proc.cpu_usage >= 10.0 {
                    format!("{:>6.1}", proc.cpu_usage).yellow()
                } else {
                    format!("{:>6.1}", proc.cpu_usage).green()
                };

                println!(
                    "  {:<8} {:<24} {} {:>10}  {}",
                    proc.pid,
                    truncate(&proc.name, 24),
                    cpu_style,
                    mem,
                    proc.status.green()
                );
            }

            println!("{}", "─".repeat(70).dimmed());
            if processes.len() > limit {
                println!(
                    "  {} showing {} of {} processes",
                    "ℹ️".blue(),
                    limit,
                    processes.len()
                );
            }
        }
        Some(Commands::Kill { pid }) => {
            let monitor = process::ProcessMonitor::new();
            monitor.kill_process(pid)?;
            println!("🛑 Killed process {}", pid);
        }
        Some(Commands::Watch { path, command }) => {
            let mut watcher = watcher::FileWatcher::new(std::path::PathBuf::from(&path));
            let receiver = watcher.watch()?;

            println!("👀 Watching {} for changes...", path);

            for event in receiver.iter() {
                println!("📝 {} - {:?}", event.event_type, event.path);

                if let Some(cmd) = &command {
                    println!("🏃 Running: {}", cmd);
                    #[cfg(unix)]
                    std::process::Command::new("sh")
                        .args(&["-c", cmd])
                        .status()?;
                    #[cfg(windows)]
                    std::process::Command::new("cmd")
                        .args(["/C", cmd])
                        .status()?;
                }
            }
        }
        Some(Commands::Logs {
            file,
            level,
            search,
            tail,
        }) => {
            let parser = logs::LogParser::new();
            let entries = parser.parse_file(std::path::Path::new(&file))?;

            if entries.is_empty() {
                println!("ℹ️  No entries found in {}", file);
                return Ok(());
            }

            let mut viewer = logs::LogViewer::new();

            if let Some(l) = &level {
                viewer = viewer.filter_level(logs::LogLevel::from_str(l));
            }
            if let Some(s) = &search {
                viewer = viewer.filter_text(s);
            }

            for entry in entries {
                viewer.add_entry(entry);
            }

            let mut displayed: Vec<logs::LogEntry> =
                viewer.tail(tail).into_iter().cloned().collect();
            displayed.reverse();

            if displayed.is_empty() {
                println!(
                    "ℹ️  No entries matched{}{}",
                    level.map(|l| format!(" --level {}", l)).unwrap_or_default(),
                    search
                        .as_ref()
                        .map(|s| format!(" --search \"{}\"", s))
                        .unwrap_or_default()
                );
                return Ok(());
            }

            let shown = displayed.len();
            for entry in displayed {
                let level_str = match entry.level {
                    logs::LogLevel::Error | logs::LogLevel::Fatal => format!("❌ {}", entry.level),
                    logs::LogLevel::Warn => format!("⚠️ {}", entry.level),
                    logs::LogLevel::Info => format!("ℹ️ {}", entry.level),
                    logs::LogLevel::Debug => format!("🐞 {}", entry.level),
                    logs::LogLevel::Trace => format!("🔍 {}", entry.level),
                };
                match &entry.timestamp {
                    Some(ts) => println!("{} {} {}", ts.to_string().dimmed(), level_str, entry.message),
                    None => println!("{} {}", level_str, entry.message),
                }
            }

            let stats = viewer.stats();
            println!();
            println!(
                "{} {} entries shown of {} total — {} errors, {} warnings",
                "📊".cyan(),
                shown,
                stats.total,
                stats.error + stats.fatal,
                stats.warn
            );
        }
        Some(Commands::Gen { command }) => {
            let generator = codegen::CodeGenerator::new(std::path::PathBuf::from("."));
            match command {
                GenCommands::Struct { name } => {
                    let code = generator.generate_struct(
                        &name,
                        &[
                            ("id".to_string(), "u64".to_string()),
                            ("name".to_string(), "String".to_string()),
                        ],
                    )?;
                    println!("{}", code);
                }
                GenCommands::Endpoint { method, path } => {
                    let handler_name = path.replace(['/', '-'], "_");
                    let code = generator.generate_api_endpoint(&method, &path, &handler_name)?;
                    println!("{}", code);
                }
                GenCommands::Tests { name } => {
                    let code = generator
                        .generate_test(&name, &[("test_basic", ""), ("test_edge_cases", "")])?;
                    println!("{}", code);
                }
            }
        }
        Some(Commands::Bisect { command }) => match command {
            BisectCommands::Start { good, bad } => {
                let mut bisect = bisect::GitBisect::new("cargo test");
                bisect.add_good(&good);
                bisect.add_bad(&bad);
                bisect.start()?;
            }
            BisectCommands::Test => {
                let test_command =
                    std::env::var("TF_BISECT_CMD").unwrap_or_else(|_| "cargo test".to_string());
                let mut bisect = bisect::GitBisect::new(&test_command);

                println!("🔍 Running: {}", test_command.cyan());
                match bisect.run_test()? {
                    bisect::BisectResult::Found {
                        culprit_commit,
                        message,
                    } => {
                        println!(
                            "\n{}",
                            "🎯 First bad commit:".green().bold()
                        );
                        println!("   {}", culprit_commit.yellow());
                        for line in message.lines().filter(|l| !l.trim().is_empty()) {
                            println!("   {}", line.dimmed());
                        }
                        println!(
                            "\n  {} {}",
                            "Finish with:".dimmed(),
                            "tf bisect reset".cyan()
                        );
                    }
                    bisect::BisectResult::Continue {
                        current_commit,
                        message,
                    } => {
                        let short = &current_commit[..7.min(current_commit.len())];
                        println!(
                            "\n⏳ Tested {} — continue with {}",
                            short.yellow(),
                            "tf bisect test".cyan()
                        );
                        if !message.trim().is_empty() {
                            println!("   {}", message.trim().dimmed());
                        }
                    }
                }
            }
            BisectCommands::Reset => {
                let bisect = bisect::GitBisect::new("cargo test");
                bisect.reset()?;
            }
        },
        Some(Commands::Vault { command }) => {
            let mut vault = secrets::SecretVault::new(vault_root()?)?;
            match command {
                VaultCommands::Unlock => {
                    if vault.is_unlocked() {
                        println!("ℹ️  Vault is already unlocked");
                    } else {
                        let password = read_password()?;
                        vault.unlock(&password)?;
                        let count = vault.list_secrets()?.len();
                        println!("   {} secret(s) available", count);
                        println!(
                            "   {} {}",
                            "List them:".dimmed(),
                            "tf vault list".cyan()
                        );
                    }
                }
                VaultCommands::Lock => {
                    if vault.is_unlocked() {
                        vault.lock()?;
                    } else {
                        println!("ℹ️  Vault is already locked");
                    }
                }
                VaultCommands::List => {
                    let secrets = vault.list_secrets()?;
                    if secrets.is_empty() {
                        println!("ℹ️  The vault is empty");
                        println!(
                            "  {} {}",
                            "Add one:".dimmed(),
                            "tf vault store API_KEY \"value\"".cyan()
                        );
                    } else {
                        println!("{}", "🗄️  Vault entries".cyan().bold());
                        for secret in &secrets {
                            println!("  🔐 {}", secret.green());
                        }
                    }
                }
                VaultCommands::Store { name, value } => {
                    let value = match value {
                        Some(value) => value,
                        None => read_secret_input(&format!("Value for '{}'", name))?,
                    };
                    if value.is_empty() {
                        anyhow::bail!("Value must not be empty");
                    }
                    vault.store(&name, &value)?;
                    println!("🔐 Stored '{}'", name.green());
                }
                VaultCommands::Get { name } => {
                    let value = vault.retrieve(&name)?;
                    if stdin_is_terminal() {
                        println!("{}", value.cyan());
                    } else {
                        println!("{}", value);
                    }
                }
                VaultCommands::Backup { path } => {
                    let path =
                        path.unwrap_or_else(|| std::path::PathBuf::from("vault-backup.json"));
                    vault.backup(&path)?;
                }
                VaultCommands::Restore { path } => {
                    vault.restore(&path)?;
                }
            }
        }
        Some(Commands::Serve { port }) => {
            let mut server = api::APIServer::new(port);
            server.get("/health", |_req| {
                api::server::Response::json(r#"{"status":"ok"}"#)
            });
            server.start().await?;
        }
    }

    Ok(())
}
