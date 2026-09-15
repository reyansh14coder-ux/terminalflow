#![allow(dead_code)]

use anyhow::Result;
use std::time::Duration;

pub struct ProcessWatcher {
    watched_processes: Vec<WatchedProcess>,
    interval: Duration,
}

struct WatchedProcess {
    name: String,
    command: String,
    args: Vec<String>,
    restart_count: u32,
    max_restarts: u32,
    status: WatchStatus,
}

#[derive(Debug, Clone)]
pub enum WatchStatus {
    Running,
    Stopped,
    Failed,
    Restarting,
}

impl ProcessWatcher {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            watched_processes: Vec::new(),
            interval: Duration::from_millis(interval_ms),
        }
    }

    pub fn watch(&mut self, name: &str, command: &str, args: Vec<String>) {
        self.watched_processes.push(WatchedProcess {
            name: name.to_string(),
            command: command.to_string(),
            args,
            restart_count: 0,
            max_restarts: 5,
            status: WatchStatus::Running,
        });
    }

    pub fn set_max_restarts(&mut self, name: &str, max: u32) {
        if let Some(process) = self.watched_processes.iter_mut().find(|p| p.name == name) {
            process.max_restarts = max;
        }
    }

    pub fn start_watching(&mut self) -> Result<()> {
        println!("👀 Starting process watcher...");

        loop {
            for process in &mut self.watched_processes {
                match process.status {
                    WatchStatus::Failed => {
                        if process.restart_count < process.max_restarts {
                            println!("🔄 Restarting process '{}'...", process.name);
                            process.restart_count += 1;
                            process.status = WatchStatus::Restarting;
                            // In real implementation, restart the process
                        } else {
                            println!("❌ Process '{}' exceeded max restarts", process.name);
                        }
                    }
                    WatchStatus::Running => {
                        // Check if process is still running
                    }
                    _ => {}
                }
            }

            std::thread::sleep(self.interval);
        }
    }

    pub fn get_status(&self) -> Vec<(String, WatchStatus)> {
        self.watched_processes
            .iter()
            .map(|p| (p.name.clone(), p.status.clone()))
            .collect()
    }

    pub fn stop(&mut self, name: &str) -> Result<()> {
        if let Some(process) = self.watched_processes.iter_mut().find(|p| p.name == name) {
            process.status = WatchStatus::Stopped;
            println!("🛑 Stopped watching '{}'", name);
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<()> {
        for process in &mut self.watched_processes {
            process.status = WatchStatus::Stopped;
        }
        println!("🛑 Stopped watching all processes");
        Ok(())
    }
}
