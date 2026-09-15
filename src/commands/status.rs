use anyhow::Result;
use colored::*;
use std::path::Path;

pub async fn show() -> Result<()> {
    println!("{}", "📊 Git Status".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let status = crate::git::status::GitStatus::from_repo(Path::new("."))?;

    println!("  {} {}", "Branch:".dimmed(), status.branch.green());
    println!();

    if status.modified.is_empty()
        && status.added.is_empty()
        && status.deleted.is_empty()
        && status.untracked.is_empty()
    {
        println!("  {} Working tree is clean", "✅".green());
    } else {
        if !status.modified.is_empty() {
            println!("  {} Modified:", "📝".yellow());
            for file in &status.modified {
                println!("    {}", file);
            }
        }

        if !status.added.is_empty() {
            println!("  {} Added:", "➕".green());
            for file in &status.added {
                println!("    {}", file);
            }
        }

        if !status.deleted.is_empty() {
            println!("  {} Deleted:", "🗑️".red());
            for file in &status.deleted {
                println!("    {}", file);
            }
        }

        if !status.untracked.is_empty() {
            println!("  {} Untracked:", "❓".yellow());
            for file in &status.untracked {
                println!("    {}", file);
            }
        }
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}
