use anyhow::Result;
use colored::*;
use std::process::Command;

pub async fn review(target: &str) -> Result<()> {
    println!("{}", "🔍 Code Review".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    let diff = collect_diff(target)?;

    let diff_str = diff.trim();
    if diff_str.is_empty() {
        println!("  {} No changes to review", "ℹ️".blue());
        println!();
        println!(
            "  {} stage changes with {} or commit first",
            "💡".dimmed(),
            "git add -p".cyan()
        );
        return Ok(());
    }

    let stats = diff_stats(diff_str);
    println!(
        "  {} {} file(s) changed  {} {}  {} {}",
        "📄".dimmed(),
        stats.files,
        "+".green(),
        stats.added,
        "-".red(),
        stats.removed
    );
    println!();

    let findings = local_review(diff_str);

    if crate::ai::client::has_api_key() {
        println!("{}", "🤖 AI reviewing your diff…".yellow());
        match crate::ai::client::review_code(diff_str).await {
            Ok(report) => {
                println!("{}", "─".repeat(60).dimmed());
                for line in report.lines() {
                    println!("  {}", line);
                }
                println!("{}", "─".repeat(60).dimmed());
                println!();
                print_findings(&findings);
                return Ok(());
            }
            Err(err) => {
                println!("  {} AI unavailable: {}", "⚠️".yellow(), err);
                println!("  {} showing local static review instead", "↩️".dimmed());
                println!();
            }
        }
    } else {
        println!(
            "  {} local static review (set OPENAI_API_KEY for AI review)",
            "🧠".dimmed()
        );
        println!();
    }

    print_findings(&findings);

    Ok(())
}

fn collect_diff(target: &str) -> Result<String> {
    let staged = git(&["diff", "--cached"])?;
    if !staged.trim().is_empty() {
        return Ok(staged);
    }

    let worktree = git(&["diff"])?;
    if !worktree.trim().is_empty() {
        return Ok(worktree);
    }

    if target == "HEAD" {
        let last = git(&["diff", "HEAD~1", "HEAD"])?;
        if !last.trim().is_empty() {
            return Ok(last);
        }
        return Ok(git(&["show", "--stat", "--oneline", "HEAD"])?);
    }

    Ok(git(&["diff", &format!("{}~1", target), target])?)
}

fn git(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() && args[0] != "diff" {
        anyhow::bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

struct DiffStats {
    files: usize,
    added: usize,
    removed: usize,
}

fn diff_stats(diff: &str) -> DiffStats {
    let mut stats = DiffStats {
        files: 0,
        added: 0,
        removed: 0,
    };
    let mut seen = std::collections::HashSet::new();

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            if seen.insert(path.to_string()) {
                stats.files += 1;
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            stats.added += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            stats.removed += 1;
        }
    }

    if stats.files == 0 {
        stats.files = seen.len();
    }
    stats
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Severity {
    High,
    Medium,
    Low,
}

impl Severity {
    fn label(self) -> ColoredString {
        match self {
            Severity::High => "HIGH".red().bold(),
            Severity::Medium => "MED ".yellow(),
            Severity::Low => "LOW ".blue(),
        }
    }
}

struct Finding {
    severity: Severity,
    file: String,
    line: usize,
    message: String,
}

fn local_review(diff: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut current_file = String::from("(diff)");
    let mut line_no = 0usize;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = path.to_string();
            continue;
        }
        if line.starts_with("@@") {
            line_no = parse_hunk_line(line);
            continue;
        }
        if line.starts_with('+') && !line.starts_with("+++") {
            let content = &line[1..];
            check_line(&mut findings, &current_file, line_no, content);
            line_no += 1;
        } else if !line.starts_with('-') && !line.starts_with('\\') {
            line_no += 1;
        }
    }

    findings
}

fn parse_hunk_line(line: &str) -> usize {
    line.split_whitespace()
        .nth(1)
        .and_then(|range| range.split(',').next())
        .and_then(|start| start.strip_prefix('-'))
        .and_then(|n| n.parse().ok())
        .unwrap_or(1)
}

fn check_line(findings: &mut Vec<Finding>, file: &str, line: usize, content: &str) {
    let mut push = |severity: Severity, message: &str| {
        findings.push(Finding {
            severity,
            file: file.to_string(),
            line,
            message: message.to_string(),
        });
    };

    let trimmed = content.trim();
    let lower = content.to_lowercase();

    if content.contains(".unwrap()") {
        push(
            Severity::High,
            "`unwrap()` panics on None/Err — prefer `?` or handle the error",
        );
    }
    if content.contains(".expect(") && !content.contains("//") {
        push(Severity::Medium, "`expect()` panics — make sure the message is reachable-safe");
    }
    if lower.contains("password") && content.contains('"') && !content.contains("env") {
        push(
            Severity::High,
            "Possible hardcoded password — move it to a secret/env var",
        );
    }
    if (lower.contains("api_key") || lower.contains("secret") || lower.contains("token"))
        && content.contains('"')
        && !content.contains("env")
        && !content.contains("arg")
    {
        push(
            Severity::High,
            "Possible hardcoded credential — use the secrets vault instead",
        );
    }
    if content.contains(" unsafe ") || trimmed.starts_with("unsafe") {
        push(Severity::Medium, "`unsafe` block — document the invariants it relies on");
    }
    if trimmed.starts_with("// TODO") || trimmed.starts_with("// FIXME") || trimmed.starts_with("// XXX") {
        push(Severity::Medium, "Unfinished work marker left in the diff");
    }
    if content.contains("dbg!(") {
        push(Severity::Low, "Debug output (`dbg!`) left behind");
    }
    if content.trim_end().len() > 120 {
        push(Severity::Low, "Line exceeds 120 characters");
    }
    if content != content.trim_end() && !content.trim().is_empty() {
        push(Severity::Low, "Trailing whitespace");
    }
    if content.contains("password =") || content.contains("api_key =") {
        push(Severity::High, "Hardcoded credential assignment");
    }
}

fn print_findings(findings: &[Finding]) {
    if findings.is_empty() {
        println!(
            "  {} No issues found in the added lines",
            "✅".green().bold()
        );
        println!();
        println!("  {} Ready to commit", "🚀".dimmed());
        return;
    }

    println!(
        "  {} {} finding(s)",
        "🧭".cyan(),
        findings.len().to_string().cyan()
    );
    println!();

    for finding in findings {
        println!(
            "  [{}] {} {}",
            finding.severity.label(),
            format!("{}:{}", finding.file, finding.line).cyan(),
            finding.message
        );
    }

    println!();
    let high = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();
    if high > 0 {
        println!("  {} {} high-severity issue(s) — fix before pushing", "⚠️".red(), high);
    } else {
        println!("  {} Nothing blocking — but review the notes above", "👍".green());
    }
    println!();
}
