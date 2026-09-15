use anyhow::Result;
use colored::*;
use std::process::Command;

pub async fn manage() -> Result<()> {
    println!("{}", "🐳 Docker Manager".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "table {{.Names}}\t{{.Status}}\t{{.Image}}",
        ])
        .output()?;

    if output.status.success() {
        let containers = String::from_utf8_lossy(&output.stdout);

        println!("{}", "Containers:".green().bold());
        println!();

        for line in containers.lines() {
            if line.contains("Up") {
                println!("  {} {}", "✅".green(), line);
            } else if line.contains("Exited") {
                println!("  {} {}", "⏹️".red(), line);
            } else {
                println!("  {} {}", "❓".yellow(), line);
            }
        }
    } else {
        println!("{}", "⚠️  Docker not running or not installed".yellow());
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!(
        "  {} Start  {} Stop   {} Restart   {} Logs",
        "[s]".green(),
        "[x]".red(),
        "[r]".yellow(),
        "[l]".cyan()
    );

    Ok(())
}
