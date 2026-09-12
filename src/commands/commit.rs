use anyhow::Result;
use colored::*;
use std::process::Command;

pub async fn generate(use_ai: bool) -> Result<()> {
    println!("{}", "📝 Generate Commit Message".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    if use_ai {
        // Get the diff
        let diff = Command::new("git")
            .args(["diff", "--cached"])
            .output()?;

        let diff_str = String::from_utf8_lossy(&diff.stdout);

        if diff_str.is_empty() {
            println!("{}", "⚠️  No staged changes. Run 'git add' first.".yellow());
            return Ok(());
        }

        println!("{}", "🤖 Generating commit message with AI...".yellow());
        
        let message = crate::ai::client::generate_commit_message(&diff_str).await?;
        
        println!();
        println!("{}", "📝 Suggested commit message:".green().bold());
        println!();
        println!("  {}", message.cyan());
        println!();
        println!("{}", "─".repeat(50).dimmed());
        println!();
        println!("  {} Use this message? (y/n)", "❓".yellow());
    } else {
        println!("{}", "Manual commit mode".dimmed());
        println!();
        println!("  Run: {} to stage all changes", "git add .".cyan());
        println!("  Run: {} to commit", "git commit".cyan());
    }

    Ok(())
}
