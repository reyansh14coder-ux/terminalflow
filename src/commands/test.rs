use anyhow::Result;
use colored::*;
use std::process::Command;

pub async fn run() -> Result<()> {
    println!("{}", "🧪 Running Tests".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let test_commands = vec![
        ("cargo", vec!["test"], "Rust"),
        ("npm", vec!["test"], "Node.js"),
        ("pytest", vec![], "Python"),
        ("go", vec!["test", "./..."], "Go"),
    ];

    let mut found = false;

    for (cmd, args, lang) in test_commands {
        if Command::new(cmd).arg("--version").output().is_ok() {
            println!("  {} Detected {} project", "✅".green(), lang.cyan());
            println!();

            let output = Command::new(cmd).args(&args).output()?;

            if output.status.success() {
                println!("  {} Tests passed!", "✅".green().bold());
                println!();

                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().collect();
                for line in lines.iter().rev().take(5) {
                    println!("  {}", line);
                }
            } else {
                println!("  {} Some tests failed", "❌".red().bold());
                println!();
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("  {}", stderr);
            }

            found = true;
            break;
        }
    }

    if !found {
        println!("  {}", "No test runner detected".yellow());
        println!();
        println!("  Supported: cargo, npm, pytest, go");
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}
