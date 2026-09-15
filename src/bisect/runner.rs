#![allow(dead_code)]

use anyhow::{Context, Result};
use std::process::Command;

pub struct GitBisect {
    good_commits: Vec<String>,
    bad_commits: Vec<String>,
    test_command: String,
    current_commit: Option<String>,
}

impl GitBisect {
    pub fn new(test_command: &str) -> Self {
        Self {
            good_commits: Vec::new(),
            bad_commits: Vec::new(),
            test_command: test_command.to_string(),
            current_commit: None,
        }
    }

    pub fn add_good(&mut self, commit: &str) {
        self.good_commits.push(commit.to_string());
    }

    pub fn add_bad(&mut self, commit: &str) {
        self.bad_commits.push(commit.to_string());
    }

    pub fn start(&mut self) -> Result<()> {
        if self.good_commits.is_empty() || self.bad_commits.is_empty() {
            return Err(anyhow::anyhow!("Need at least one good and one bad commit"));
        }

        let output = Command::new("git")
            .args(["bisect", "start"])
            .output()
            .context("Failed to start git bisect")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git bisect start failed: {}", stderr));
        }

        for bad in &self.bad_commits {
            Command::new("git").args(["bisect", "bad", bad]).output()?;
        }

        for good in &self.good_commits {
            Command::new("git")
                .args(["bisect", "good", good])
                .output()?;
        }

        println!("🔍 Git bisect started");
        println!("  Bad commits: {:?}", self.bad_commits);
        println!("  Good commits: {:?}", self.good_commits);
        println!("  Test command: {}", self.test_command);

        Ok(())
    }

    pub fn run_test(&mut self) -> Result<BisectResult> {
        let output = Command::new("sh")
            .args(["-c", &self.test_command])
            .output()
            .context("Failed to run test command")?;

        let success = output.status.success();

        // Get current commit
        let commit_output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

        let commit = String::from_utf8_lossy(&commit_output.stdout)
            .trim()
            .to_string();

        self.current_commit = Some(commit.clone());

        let mark = if success { "good" } else { "bad" };

        let bisect_output = Command::new("git")
            .args(["bisect", mark])
            .output()
            .context("Failed to mark commit")?;

        let message = String::from_utf8_lossy(&bisect_output.stdout).to_string();

        if message.contains("is the first bad commit") {
            Ok(BisectResult::Found {
                culprit_commit: commit,
                message,
            })
        } else {
            Ok(BisectResult::Continue {
                current_commit: commit,
                message,
            })
        }
    }

    pub fn auto_bisect(&mut self) -> Result<Option<String>> {
        loop {
            let result = self.run_test()?;

            match result {
                BisectResult::Found { culprit_commit, .. } => {
                    println!("🎯 Found the culprit commit: {}", culprit_commit);
                    return Ok(Some(culprit_commit));
                }
                BisectResult::Continue { current_commit, .. } => {
                    println!(
                        "📝 Testing commit: {}",
                        &current_commit[..7.min(current_commit.len())]
                    );
                }
            }
        }
    }

    pub fn visualize(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["bisect", "log"])
            .output()
            .context("Failed to get bisect log")?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn reset(&self) -> Result<()> {
        Command::new("git")
            .args(["bisect", "reset"])
            .output()
            .context("Failed to reset bisect")?;

        println!("🔄 Git bisect reset");
        Ok(())
    }
}

#[derive(Debug)]
pub enum BisectResult {
    Found {
        culprit_commit: String,
        message: String,
    },
    Continue {
        current_commit: String,
        message: String,
    },
}
