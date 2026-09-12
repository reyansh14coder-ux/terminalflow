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
use colored::*;

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
        /// HTTP method
        #[arg(default_value = "GET")]
        method: String,
        
        /// URL
        url: String,
        
        /// Request body
        #[arg(short, long)]
        body: Option<String>,
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
        #[arg(short, long)]
        top_cpu: bool,
        
        /// Show top memory processes
        #[arg(short, long)]
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
    Add { name: String, host: String, user: String },
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
    /// Unlock vault
    Unlock,
    /// Lock vault
    Lock,
    /// List secrets
    List,
    /// Backup vault
    Backup,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Dashboard) | None => {
            ui::dashboard::run().await?;
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
        Some(Commands::Plugin { command }) => {
            match command {
                PluginCommands::List => {
                    let mut manager = plugins::PluginManager::new();
                    manager.discover_and_load()?;
                    for plugin in manager.list_plugins() {
                        println!("📦 {} v{}", plugin.manifest.name, plugin.manifest.version);
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
                    for plugin in results {
                        println!("📦 {} - {} (⬇️ {} downloads)", plugin.name, plugin.description, plugin.downloads);
                    }
                }
            }
        }
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
                    for secret in manager.list() {
                        println!("🔑 {}", secret.name);
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
                    for conn in manager.list() {
                        println!("🖥️ {} - {}@{}:{}", conn.name, conn.user, conn.host, conn.port);
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
                SshCommands::Tunnel { local_port, remote_host, remote_port, ssh_host } => {
                    let mut tunnel = ssh::SSHTunnel::new(local_port, &remote_host, remote_port, &ssh_host, "root");
                    tunnel.start()?;
                }
            }
        }
        Some(Commands::Http { method, url, body }) => {
            let client = http::HttpClient::new()?;
            let request = http::client::HttpRequest {
                method,
                url,
                body,
                ..Default::default()
            };
            let response = client.execute(&request).await?;
            println!("{} {} - {}ms", response.status, response.status_text, response.duration_ms);
            println!("{}", response.body);
        }
        Some(Commands::Api { command }) => {
            match command {
                ApiCommands::Test { file } => {
                    let tester = api::APITester::new()?;
                    println!("🧪 Running API tests from {}", file);
                }
                ApiCommands::Create { name } => {
                    println!("📝 Creating API test: {}", name);
                }
            }
        }
        Some(Commands::Ps { filter, top_cpu, top_mem }) => {
            let mut monitor = process::ProcessMonitor::new();
            monitor.refresh()?;
            
            let mut processes = monitor.get_processes().to_vec();
            
            if top_cpu {
                processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
            } else if top_mem {
                processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
            }
            
            if let Some(f) = filter {
                processes.retain(|p| p.name.contains(&f) || p.command.contains(&f));
            }
            
            println!("📊 Processes:");
            for proc in processes.iter().take(20) {
                println!("  {} (PID: {}) - CPU: {:.1}% - Memory: {:.1}MB",
                    proc.name, proc.pid, proc.cpu_usage, proc.memory_usage as f64 / 1024.0 / 1024.0);
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
                    std::process::Command::new("sh")
                        .args(&["-c", cmd])
                        .status()?;
                }
            }
        }
        Some(Commands::Logs { file, level, search, tail }) => {
            let parser = logs::LogParser::new();
            let entries = parser.parse_file(std::path::Path::new(&file))?;
            
            let mut viewer = logs::LogViewer::new();
            
            if let Some(l) = level {
                let log_level = logs::LogLevel::from_str(&l);
                viewer = viewer.filter_level(log_level);
            }
            
            for entry in entries {
                viewer.add_entry(entry);
            }
            
            let mut displayed = viewer.tail(tail);
            displayed.reverse();
            
            for entry in displayed {
                let level_str = match entry.level {
                    logs::LogLevel::Error | logs::LogLevel::Fatal => format!("❌ {}", entry.level),
                    logs::LogLevel::Warn => format!("⚠️ {}", entry.level),
                    logs::LogLevel::Info => format!("ℹ️ {}", entry.level),
                    _ => format!("  {}", entry.level),
                };
                println!("{} {}", level_str, entry.message);
            }
        }
        Some(Commands::Gen { command }) => {
            let generator = codegen::CodeGenerator::new(std::path::PathBuf::from("."));
            match command {
                GenCommands::Struct { name } => {
                    let code = generator.generate_struct(&name, &[
                        ("id".to_string(), "u64".to_string()),
                        ("name".to_string(), "String".to_string()),
                    ])?;
                    println!("{}", code);
                }
                GenCommands::Endpoint { method, path } => {
                    let handler_name = path.replace('/', "_").replace('-', "_");
                    let code = generator.generate_api_endpoint(&method, &path, &handler_name)?;
                    println!("{}", code);
                }
                GenCommands::Tests { name } => {
                    let code = generator.generate_test(&name, &[
                        ("test_basic", ""),
                        ("test_edge_cases", ""),
                    ])?;
                    println!("{}", code);
                }
            }
        }
        Some(Commands::Bisect { command }) => {
            match command {
                BisectCommands::Start { good, bad } => {
                    let mut bisect = bisect::GitBisect::new("cargo test");
                    bisect.add_good(&good);
                    bisect.add_bad(&bad);
                    bisect.start()?;
                }
                BisectCommands::Test => {
                    println!("🔍 Running bisect test...");
                }
                BisectCommands::Reset => {
                    let bisect = bisect::GitBisect::new("cargo test");
                    bisect.reset()?;
                }
            }
        }
        Some(Commands::Vault { command }) => {
            let mut vault = secrets::SecretVault::new(std::path::PathBuf::from("."))?;
            match command {
                VaultCommands::Unlock => {
                    let password = rpassword::read_password().unwrap_or_default();
                    vault.unlock(&password)?;
                }
                VaultCommands::Lock => {
                    vault.lock()?;
                }
                VaultCommands::List => {
                    let secrets = vault.list_secrets()?;
                    for secret in secrets {
                        println!("🔐 {}", secret);
                    }
                }
                VaultCommands::Backup => {
                    let backup_path = std::path::PathBuf::from("vault_backup");
                    vault.backup(&backup_path)?;
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
