use anyhow::Result;
use colored::*;
use std::io::Write;
use std::process::Command;

pub async fn generate(use_ai: bool) -> Result<()> {
    println!("{}", "📝 Commit Message".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    let staged = git(&["diff", "--cached"])?;

    if staged.trim().is_empty() {
        println!("  {} No staged changes to commit", "ℹ️".blue());
        println!();
        println!("  {} {}", "Stage first:".dimmed(), "git add -p".cyan());
        println!("  {} {}", "or stage everything:".dimmed(), "git add .".cyan());
        return Ok(());
    }

    let stats = diff_stats(&staged);
    println!(
        "  {} {} file(s) staged  {} {}  {} {}",
        "📄".dimmed(),
        stats.files,
        "+".green(),
        stats.added,
        "-".red(),
        stats.removed
    );
    println!();

    let mut used_ai = false;
    let message = if use_ai {
        if crate::ai::client::has_api_key() {
            println!("{}", "🤖 Generating with AI…".yellow());
            used_ai = true;
            match crate::ai::client::generate_commit_message(&staged).await {
                Ok(message) => message.lines().next().unwrap_or("").trim().to_string(),
                Err(err) => {
                    println!("  {} AI unavailable: {}", "⚠️".yellow(), err);
                    println!("  {} falling back to the local generator", "↩️".dimmed());
                    println!();
                    offline_message(&staged)
                }
            }
        } else {
            println!(
                "  {} --ai needs an API key — using the local generator instead",
                "⚠️".yellow()
            );
            crate::ai::client::print_setup_help();
            println!();
            offline_message(&staged)
        }
    } else {
        offline_message(&staged)
    };

    println!(
        "{}",
        if used_ai {
            "✨ Suggested commit message:".green().bold()
        } else {
            "✨ Commit message:".green().bold()
        }
    );
    println!();
    println!("  {}", message.cyan().bold());
    println!();
    println!("{}", "─".repeat(60).dimmed());

    if !prompt_apply() {
        println!();
        println!(
            "  {} apply it yourself with:\n    {}",
            "💡".dimmed(),
            format!("git commit -m \"{}\"", message.replace('"', "'")).cyan()
        );
        return Ok(());
    }

    let status = Command::new("git")
        .args(["commit", "-m", &message])
        .output()?;

    if status.status.success() {
        println!();
        println!("  {} Committed", "✅".green().bold());
        println!();
        print!("{}", String::from_utf8_lossy(&status.stdout).dimmed());
    } else {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&status.stderr).trim()
        );
    }

    Ok(())
}

fn prompt_apply() -> bool {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return false;
    }
    print!("  {} ", "Apply this message? (y/n)".yellow());
    let _ = std::io::stdout().flush();
    let mut answer = String::new();
    match std::io::stdin().read_line(&mut answer) {
        Ok(0) | Err(_) => false,
        Ok(_) => matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"),
    }
}

fn git(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

struct DiffStats {
    files: usize,
    added: usize,
    removed: usize,
}

fn diff_stats(diff: &str) -> DiffStats {
    let mut files = std::collections::HashSet::new();
    let mut added = 0usize;
    let mut removed = 0usize;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            files.insert(path.to_string());
        } else if line.starts_with('+') && !line.starts_with("+++") {
            added += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            removed += 1;
        }
    }

    DiffStats {
        files: files.len(),
        added,
        removed,
    }
}

/// Deterministic conventional-commit message built from the staged diff —
/// used whenever the AI is unavailable so `tf commit` always works.
fn offline_message(diff: &str) -> String {
    let mut files: Vec<String> = diff
        .lines()
        .filter_map(|line| line.strip_prefix("+++ b/"))
        .map(|path| path.to_string())
        .collect();
    files.sort();
    files.dedup();

    let mut added_files = 0usize;
    let mut removed_files = 0usize;
    for line in diff.lines() {
        if line.starts_with("new file mode") {
            added_files += 1;
        } else if line.starts_with("deleted file mode") {
            removed_files += 1;
        }
    }

    let scope = files
        .first()
        .map(|path| path.split('/').next().unwrap_or(path).to_string());

    let kind = classify(&files, added_files, removed_files);

    let scope_part = match &scope {
        Some(scope) if files.len() == 1 || scope != "src" => format!("({})", scope),
        _ => String::new(),
    };

    let stats = diff_stats(diff);
    let summary = if files.len() == 1 {
        files
            .first()
            .and_then(|path| path.rsplit('/').next())
            .unwrap_or(files.first().map(|f| f.as_str()).unwrap_or("changes"))
            .to_string()
    } else {
        format!("{} files", files.len())
    };

    format!(
        "{}{}: update {} (+{} -{})",
        kind, scope_part, summary, stats.added, stats.removed
    )
}

fn classify(files: &[String], added: usize, removed: usize) -> &'static str {
    if files.iter().any(|f| f.starts_with(".github/") || f == "Makefile") {
        return "ci";
    }
    if files.iter().any(|f| f.starts_with("tests/") || f.contains("test")) {
        return "test";
    }
    if files.iter().all(|f| f.ends_with(".md")) {
        return "docs";
    }
    if files.iter().any(|f| f == "Cargo.lock" || f == "package-lock.json") {
        return "chore(deps)";
    }
    if files.iter().any(|f| f == "Cargo.toml" || f == "package.json") {
        return "build";
    }
    if added > 0 && removed == 0 {
        return "feat";
    }
    if files.is_empty() {
        return "chore";
    }
    "refactor"
}
