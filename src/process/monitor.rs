#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_percent: f64,
    pub status: String,
    pub start_time: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percent: f64,
    pub disk_total: u64,
    pub disk_used: u64,
    pub disk_percent: f64,
    pub load_average: [f64; 3],
    pub uptime: u64,
    pub process_count: usize,
}

pub struct ProcessMonitor {
    processes: Vec<ProcessInfo>,
    stats: SystemStats,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            stats: SystemStats {
                cpu_usage: 0.0,
                memory_total: 0,
                memory_used: 0,
                memory_percent: 0.0,
                disk_total: 0,
                disk_used: 0,
                disk_percent: 0.0,
                load_average: [0.0, 0.0, 0.0],
                uptime: 0,
                process_count: 0,
            },
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_processes()?;
        self.refresh_stats()?;
        Ok(())
    }

    fn refresh_processes(&mut self) -> Result<()> {
        // In real implementation, this would use sysinfo crate
        self.processes = vec![
            ProcessInfo {
                pid: 1,
                name: "systemd".to_string(),
                cpu_usage: 0.1,
                memory_usage: 1024 * 1024,
                memory_percent: 0.5,
                status: "Running".to_string(),
                start_time: "2024-01-01 00:00:00".to_string(),
                command: "/sbin/init".to_string(),
            },
            ProcessInfo {
                pid: 1234,
                name: "node".to_string(),
                cpu_usage: 15.2,
                memory_usage: 256 * 1024 * 1024,
                memory_percent: 12.5,
                status: "Running".to_string(),
                start_time: "2024-01-15 10:30:00".to_string(),
                command: "node server.js".to_string(),
            },
            ProcessInfo {
                pid: 5678,
                name: "postgres".to_string(),
                cpu_usage: 5.8,
                memory_usage: 512 * 1024 * 1024,
                memory_percent: 25.0,
                status: "Running".to_string(),
                start_time: "2024-01-15 09:00:00".to_string(),
                command: "postgres -D /var/lib/postgresql/data".to_string(),
            },
        ];
        Ok(())
    }

    fn refresh_stats(&mut self) -> Result<()> {
        self.stats = SystemStats {
            cpu_usage: 45.2,
            memory_total: 8 * 1024 * 1024 * 1024,
            memory_used: 4 * 1024 * 1024 * 1024,
            memory_percent: 50.0,
            disk_total: 500 * 1024 * 1024 * 1024,
            disk_used: 250 * 1024 * 1024 * 1024,
            disk_percent: 50.0,
            load_average: [1.5, 1.2, 1.0],
            uptime: 86400,
            process_count: 150,
        };
        Ok(())
    }

    pub fn get_processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    pub fn get_stats(&self) -> &SystemStats {
        &self.stats
    }

    pub fn find_process(&self, query: &str) -> Vec<&ProcessInfo> {
        self.processes
            .iter()
            .filter(|p| {
                p.name.contains(query) || p.command.contains(query) || p.pid.to_string() == query
            })
            .collect()
    }

    pub fn kill_process(&self, pid: u32) -> Result<()> {
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output()
                .context("Failed to kill process")?;
        }
        #[cfg(windows)]
        {
            std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output()
                .context("Failed to kill process")?;
        }
        Ok(())
    }

    pub fn get_top_cpu(&self, limit: usize) -> Vec<&ProcessInfo> {
        let mut processes: Vec<&ProcessInfo> = self.processes.iter().collect();
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        processes.into_iter().take(limit).collect()
    }

    pub fn get_top_memory(&self, limit: usize) -> Vec<&ProcessInfo> {
        let mut processes: Vec<&ProcessInfo> = self.processes.iter().collect();
        processes.sort_by_key(|a| std::cmp::Reverse(a.memory_usage));
        processes.into_iter().take(limit).collect()
    }
}
