mod ai;
mod commands;
mod git;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(
    name = "terminalflow",
    about = "🚀 AI-Powered Terminal Dashboard for Developers",
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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "terminalflow=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

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
    }

    Ok(())
}
