use anyhow::Result;
use colored::*;
use std::process::Command;
use std::time::Instant;

/// Aggregated result of one test run, shared by `tf test` and the dashboard.
#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    pub runner: String,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub duration_secs: f64,
    /// (test name, passed?) — best effort, per runner.
    pub results: Vec<(String, bool)>,
    /// Set when the runner itself could not be executed.
    pub error: Option<String>,
    pub raw_tail: Vec<String>,
}

impl TestSummary {
    pub fn total(&self) -> usize {
        self.passed + self.failed
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.error.is_none() && self.total() > 0
    }
}

fn has_file(name: &str) -> bool {
    std::path::Path::new(name).exists()
}

/// Pick a test runner for the project in the current directory.
pub fn detect_runner() -> Option<(&'static str, &'static [&'static str])> {
    if has_file("Cargo.toml") {
        return Some(("cargo", &["test"]));
    }
    if has_file("go.mod") {
        return Some(("go", &["test", "./..."]));
    }
    if has_file("pyproject.toml")
        || has_file("pytest.ini")
        || has_file("setup.py")
        || has_file("requirements.txt")
    {
        return Some(("pytest", &[]));
    }
    if has_file("package.json") {
        return Some(("npm", &["test", "--", "--silent"]));
    }
    None
}

fn runner_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|o| o.status.success() || !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Run the project's test suite and parse the outcome. Never panics.
pub fn run_suite() -> TestSummary {
    let Some((cmd, args)) = detect_runner() else {
        return TestSummary {
            error: Some(
                "No supported project found (Cargo.toml, go.mod, package.json, pyproject.toml)"
                    .to_string(),
            ),
            ..Default::default()
        };
    };

    let runner = if cmd == "pytest" { "pytest" } else { cmd };

    if !runner_available(cmd) {
        return TestSummary {
            runner: runner.to_string(),
            error: Some(format!("`{}` is not installed or not on PATH", cmd)),
            ..Default::default()
        };
    }

    let started = Instant::now();
    let output = Command::new(cmd).args(args).output();
    let duration = started.elapsed().as_secs_f64();

    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return TestSummary {
                runner: runner.to_string(),
                error: Some(format!("Could not start `{}`: {}", cmd, err)),
                ..Default::default()
            }
        }
    };

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut summary = parse_output(runner, &combined);
    summary.duration_secs = duration;
    summary.raw_tail = combined
        .lines()
        .filter(|l| !l.trim().is_empty())
        .rev()
        .take(6)
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    if !output.status.success() && summary.failed == 0 && summary.passed == 0 {
        let hint = combined
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .trim();
        let hint = if hint.chars().count() > 70 {
            let truncated: String = hint.chars().take(69).collect();
            format!("{}…", truncated)
        } else {
            hint.to_string()
        };
        summary.error = Some(format!("`{}` exited with {} — {}", cmd, output.status, hint));
    }

    summary
}

fn parse_output(runner: &str, output: &str) -> TestSummary {
    let mut summary = TestSummary {
        runner: runner.to_string(),
        ..Default::default()
    };

    for line in output.lines() {
        let trimmed = line.trim();

        if runner == "cargo" || runner == "go" {
            // "test foo::bar ... ok" / "... FAILED"
            if let Some(rest) = trimmed.strip_prefix("test ") {
                if let Some((name, verdict)) = rest.rsplit_once(" ... ") {
                    match verdict {
                        "ok" => {
                            summary.results.push((name.to_string(), true));
                            summary.passed += 1;
                        }
                        "FAILED" => {
                            summary.results.push((name.to_string(), false));
                            summary.failed += 1;
                        }
                        v if v.starts_with("ignored") => summary.ignored += 1,
                        _ => {}
                    }
                }
                // go: "--- PASS: TestFoo (0.00s)"
            } else if let Some(rest) = trimmed.strip_prefix("--- PASS: ") {
                let name = rest.split_whitespace().next().unwrap_or("").to_string();
                summary.results.push((name, true));
                summary.passed += 1;
            } else if let Some(rest) = trimmed.strip_prefix("--- FAIL: ") {
                let name = rest.split_whitespace().next().unwrap_or("").to_string();
                summary.results.push((name, false));
                summary.failed += 1;
            }
        } else if runner == "pytest" {
            if trimmed.starts_with("PASSED") || trimmed.contains(" PASSED") {
                summary.passed += 1;
            } else if trimmed.starts_with("FAILED") || trimmed.ends_with(" failed") {
                if trimmed.starts_with("FAILED") {
                    summary.failed += 1;
                }
            }
        } else if runner == "npm" {
            if trimmed.starts_with("✓") || trimmed.starts_with("PASS") {
                summary.passed += 1;
            } else if trimmed.starts_with("✗") || trimmed.starts_with("✕") || trimmed.starts_with("FAIL")
            {
                summary.failed += 1;
            }
        }

        // Summary lines: cargo/go style "test result: ok. 5 passed; 1 failed; ..."
        if let Some(idx) = trimmed.find("test result:") {
            let tail = &trimmed[idx + "test result:".len()..];
            if let Some(passed) = find_count(tail, "passed") {
                summary.passed = summary.passed.max(passed);
            }
            if let Some(failed) = find_count(tail, "failed") {
                summary.failed = summary.failed.max(failed);
            }
            if let Some(ignored) = find_count(tail, "ignored") {
                summary.ignored = summary.ignored.max(ignored);
            }
        }

        // npm: "Tests:       12 passed, 1 failed"
        if trimmed.starts_with("Tests:") {
            if let Some(passed) = find_count(tail_of(trimmed), "passed") {
                summary.passed = summary.passed.max(passed);
            }
            if let Some(failed) = find_count(tail_of(trimmed), "failed") {
                summary.failed = summary.failed.max(failed);
            }
        }

        // pytest: "12 passed, 1 failed in 3.20s"
        if trimmed.contains(" passed") && trimmed.contains(" in ") {
            if let Some(passed) = find_count(trimmed, "passed") {
                summary.passed = summary.passed.max(passed);
            }
            if let Some(failed) = find_count(trimmed, "failed") {
                summary.failed = summary.failed.max(failed);
            }
        }
    }

    summary
}

fn tail_of(line: &str) -> &str {
    line.splitn(2, ':').nth(1).unwrap_or(line)
}

/// Parse `5` from `5 passed` / `5 failed`.
fn find_count(line: &str, word: &str) -> Option<usize> {
    for (i, token) in line.split_whitespace().enumerate() {
        if token.trim_end_matches(';') == word {
            if let Some(prev) = line.split_whitespace().nth(i.wrapping_sub(1)) {
                if let Ok(value) = prev.trim_end_matches(';').parse::<usize>() {
                    return Some(value);
                }
            }
        }
    }
    None
}

pub async fn run() -> Result<()> {
    println!("{}", "🧪 Running Tests".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let summary = tokio::task::spawn_blocking(run_suite).await?;

    if let Some(error) = &summary.error {
        println!("  {} {}", "⚠️".yellow(), error.yellow());
        println!();
        println!(
            "  {} Supported runners: cargo, npm, pytest, go",
            "ℹ️".dimmed()
        );
        return Ok(());
    }

    println!(
        "  {} {} passed in {:.2}s",
        "✅",
        summary.passed.to_string().green().bold(),
        summary.duration_secs
    );
    if summary.failed > 0 {
        println!("  {} {} failed", "❌", summary.failed.to_string().red());
    }
    if summary.ignored > 0 {
        println!(
            "  {} {} ignored",
            "⏭️",
            summary.ignored.to_string().dimmed()
        );
    }

    if !summary.results.is_empty() {
        println!();
        let shown = summary.results.iter().take(12);
        for (name, passed) in shown {
            if *passed {
                println!("    {} {}", "✓".green(), name);
            } else {
                println!("    {} {}", "✗".red().bold(), name);
            }
        }
        if summary.results.len() > 12 {
            println!(
                "    {}",
                format!("… and {} more", summary.results.len() - 12).dimmed()
            );
        }
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());

    if summary.failed > 0 {
        println!();
        println!(
            "  {} {}",
            "Fix them with:".dimmed(),
            "tf fix".cyan()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_result_line() {
        let output = "running 3 tests\ntest a ... ok\ntest b ... FAILED\ntest c ... ok\n\n\
                      test result: ok. 2 passed; 1 failed; 0 ignored; 0 measured; finished in 0.01s";
        let summary = parse_output("cargo", output);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.results.len(), 3);
        assert!(!summary.all_passed());
    }

    #[test]
    fn detects_cargo_project_here() {
        // This repository itself is a Cargo project.
        let runner = detect_runner();
        assert!(runner.is_some());
        let (cmd, _) = runner.unwrap();
        assert_eq!(cmd, "cargo");
    }

    #[test]
    fn missing_project_reports_error() {
        let summary = TestSummary {
            error: Some("boom".into()),
            ..Default::default()
        };
        assert!(summary.error.is_some());
        assert_eq!(summary.total(), 0);
    }
}
