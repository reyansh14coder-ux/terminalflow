use anyhow::Result;
use colored::*;
use std::io::{IsTerminal, Read};
use std::process::Command;

pub async fn auto_fix() -> Result<()> {
    println!("{}", "🔧 Auto Fix".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    // 1) Piped error: tf fix < build.log  |  cargo check 2>&1 | tf fix
    if let Some(piped) = read_piped_stdin() {
        return diagnose(&piped).await;
    }

    // 2) Rust project: run the compiler and explain what it reports.
    if std::path::Path::new("Cargo.toml").exists() {
        println!("  {} Running cargo check…", "⏳".yellow());
        println!();

        let output = Command::new("cargo")
            .args(["check", "--message-format", "short"])
            .output();

        let output = match output {
            Ok(output) => output,
            Err(err) => {
                println!("  {} cargo is not installed: {}", "⚠️".yellow(), err);
                println!("  {} https://rustup.rs", "⬇".dimmed());
                return Ok(());
            }
        };

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        if output.status.success() {
            println!("  {} cargo check passed — no errors found", "✅".green().bold());
            println!();
            println!(
                "  {} pipe an error in for analysis: {}",
                "💡".dimmed(),
                "cargo check 2>&1 | tf fix".cyan()
            );
            return Ok(());
        }

        return diagnose(&combined).await;
    }

    // 3) Anything else: report which project files are present.
    println!("  {} Scanning project files…", "🔍".yellow());
    println!();

    let checks = [
        ("Cargo.toml", "Rust configuration"),
        ("package.json", "Node.js configuration"),
        ("requirements.txt", "Python dependencies"),
        ("pyproject.toml", "Python project configuration"),
        (".env", "Environment variables"),
        ("Dockerfile", "Docker configuration"),
    ];

    let mut found = false;
    for (file, description) in checks {
        if std::path::Path::new(file).exists() {
            found = true;
            println!("  {} {}", "✅".green(), description);
        }
    }

    if !found {
        println!("  {} No recognised project files here", "ℹ️".blue());
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!(
        "  {} Pipe an error and I'll explain it:",
        "💡 Tip:".cyan()
    );
    println!("    {}", "tf fix < error.log".cyan());
    println!("    {}", "cargo test 2>&1 | tf fix".cyan());

    Ok(())
}

fn read_piped_stdin() -> Option<String> {
    if std::io::stdin().is_terminal() {
        return None;
    }
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer).ok()?;
    let trimmed = buffer.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

async fn diagnose(text: &str) -> Result<()> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .collect();

    let error_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| is_error_line(line))
        .take(12)
        .collect();

    if error_lines.is_empty() {
        println!("  {} No error lines detected in the input", "ℹ️".blue());
        println!();
        println!("  {} {}", "Tip:".dimmed(), "make sure the error text is piped in or above".cyan());
        return Ok(());
    }

    println!(
        "  {} {} error line(s) found",
        "🚨".red(),
        error_lines.len().to_string().red()
    );
    println!();

    for line in &error_lines {
        println!("    {}", line.red());
    }

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("  {} Suggested fixes", "💡".cyan());
    println!();

    let mut hints: Vec<&str> = Vec::new();
    for line in &error_lines {
        if let Some(hint) = hint_for(line) {
            if !hints.contains(&hint) {
                hints.push(hint);
            }
        }
    }

    if hints.is_empty() {
        println!("    {}", "• Check the file:line reported above first".dimmed());
    } else {
        for hint in hints {
            println!("    {}", format!("• {}", hint).yellow());
        }
    }

    if crate::ai::client::has_api_key() {
        println!();
        println!("{}", "🤖 Asking the AI for a concrete fix…".yellow());
        let prompt = format!("Explain and fix this error, reply with a short fix:\n\n{}", text);
        match crate::ai::client::ask(&prompt).await {
            Ok(()) => {}
            Err(err) => println!("  {} {}", "⚠️".yellow(), err),
        }
    }

    println!();
    Ok(())
}

fn is_error_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let lower = trimmed.to_lowercase();
    lower.starts_with("error")
        || lower.starts_with("error[")
        || lower.contains(": error:")
        || lower.contains(" exception")
        || lower.starts_with("exception")
        || lower.contains("panic")
        || lower.contains("failed to compile")
        || lower.contains("cannot find")
        || lower.contains("undefined reference")
        || lower.contains("modulenotfounderror")
        || lower.contains("cannot find module")
        || lower.contains("command not found")
        || trimmed.starts_with("error:")
}

/// Maps a known error to an actionable next step.
fn hint_for(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();

    if lower.contains("cannot find crate") || lower.contains("unresolved import") || lower.contains("failed to resolve") {
        return Some("Missing dependency or module path — add it with `cargo add <crate>` or fix the `use` path");
    }
    if lower.contains("mismatched types") || lower.contains("expected") && lower.contains("found") {
        return Some("Type mismatch — compare the value with the signature the compiler expects");
    }
    if lower.contains("no method named") {
        return Some("The method does not exist on that type — check the type and any missing trait import");
    }
    if lower.contains("borrow of moved value") || lower.contains("value moved") || lower.contains("use of moved value") {
        return Some("Ownership error — borrow with `&`, or `clone()` if you need an owned copy");
    }
    if lower.contains("cannot borrow") || lower.contains("cannot mutably borrow") {
        return Some("Borrow conflict — split the borrow, clone, or restructure so two mutable borrows cannot overlap");
    }
    if lower.contains("missing") && lower.contains("await") {
        return Some("You likely forgot `.await` on an async call");
    }
    if lower.contains("unused variable") || lower.contains("unused import") {
        return Some("Dead code — remove it or prefix with `_` if intentional");
    }
    if lower.contains("modulenotfounderror") || lower.contains("cannot find module") {
        return Some("Missing package — install it (`pip install <name>` / `npm install <name>`)");
    }
    if lower.contains("command not found") {
        return Some("Required tool is not installed — install it or add it to PATH");
    }
    if lower.contains("permission denied") {
        return Some("Insufficient permissions — rerun with the right privileges or change file ownership");
    }
    if lower.contains("link failed") || lower.contains("link.exe") || lower.contains("cannot open input file") {
        return Some("Toolchain/linker problem — install your platform's C build tools (MSVC/Windows SDK or Xcode CLT)");
    }
    if lower.contains("timed out") || lower.contains("timeout") {
        return Some("Timeout — check network connectivity, proxies, or raise the timeout value");
    }
    if lower.contains("enoent") || lower.contains("no such file or directory") {
        return Some("Path does not exist — verify the file path and working directory");
    }
    if lower.contains("syntaxerror") {
        return Some("Syntax error — read the reported line and the one above it character by character");
    }
    if lower.contains("out of memory") || lower.contains("oom") {
        return Some("Out of memory — close other processes or raise the memory limit");
    }

    None
}
