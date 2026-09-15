#![allow(dead_code)]

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};

pub struct ProcessManager {
    processes: HashMap<u32, ManagedProcess>,
}

struct ManagedProcess {
    child: Child,
    name: String,
    command: String,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, name: &str, command: &str, args: &[String]) -> Result<u32> {
        let child = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn process")?;

        let pid = child.id();

        self.processes.insert(
            pid,
            ManagedProcess {
                child,
                name: name.to_string(),
                command: format!("{} {}", command, args.join(" ")),
            },
        );

        println!("✅ Spawned process '{}' with PID {}", name, pid);
        Ok(pid)
    }

    pub fn spawn_background(&mut self, name: &str, command: &str, args: &[String]) -> Result<u32> {
        let child = Command::new(command)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn background process")?;

        let pid = child.id();

        self.processes.insert(
            pid,
            ManagedProcess {
                child,
                name: name.to_string(),
                command: format!("{} {}", command, args.join(" ")),
            },
        );

        println!("✅ Spawned background process '{}' with PID {}", name, pid);
        Ok(pid)
    }

    pub fn kill(&mut self, pid: u32) -> Result<()> {
        if let Some(mut process) = self.processes.remove(&pid) {
            process.child.kill().context("Failed to kill process")?;
            println!("🛑 Killed process '{}' (PID: {})", process.name, pid);
        }
        Ok(())
    }

    pub fn kill_all(&mut self) -> Result<()> {
        let pids: Vec<u32> = self.processes.keys().cloned().collect();
        for pid in pids {
            self.kill(pid)?;
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<(u32, &str, &str)> {
        self.processes
            .iter()
            .map(|(pid, p)| (*pid, p.name.as_str(), p.command.as_str()))
            .collect()
    }

    pub fn status(&self, pid: u32) -> Option<ProcessStatus> {
        self.processes.get(&pid).map(|_p| ProcessStatus::Running)
    }
}

#[derive(Debug)]
pub enum ProcessStatus {
    Running,
    Exited(i32),
    Unknown,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Running => write!(f, "🟢 Running"),
            ProcessStatus::Exited(code) => write!(f, "🔴 Exited ({})", code),
            ProcessStatus::Unknown => write!(f, "❓ Unknown"),
        }
    }
}
