use anyhow::Result;
use colored::*;
use std::process::Command;

pub async fn review(target: &str) -> Result<()> {
    println!("{}", "🔍 Code Review".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let diff = if target == "HEAD" {
        // Review staged changes or last commit
        let output = Command::new("git").args(["diff", "--cached"]).output()?;

        if output.stdout.is_empty() {
            Command::new("git")
                .args(["diff", "HEAD~1", "HEAD"])
                .output()?
        } else {
            output
        }
    } else {
        Command::new("git")
            .args(["diff", &format!("{}~1", target), target])
            .output()?
    };

    let diff_str = String::from_utf8_lossy(&diff.stdout);

    if diff_str.is_empty() {
        println!("{}", "⚠️  No changes to review".yellow());
        return Ok(());
    }

    println!("{}", "🤖 AI Reviewing your code...".yellow());
    println!();

    crate::ai::client::review_code(&diff_str).await?;

    Ok(())
}
