#![allow(dead_code)]

use anyhow::Result;
use std::process::{Child, Command, Stdio};

pub struct Pane {
    name: String,
    process: Option<Child>,
}

impl Pane {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            process: None,
        }
    }

    pub fn run(&mut self, command: &str) -> Result<()> {
        let child = Command::new("sh")
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn run_windows(&mut self, command: &str) -> Result<()> {
        let child = Command::new("cmd")
            .args(["/C", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.process = Some(child);
        Ok(())
    }

    pub fn kill(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
