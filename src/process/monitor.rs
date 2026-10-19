#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

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

impl Default for SystemStats {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl SystemStats {
    pub fn cpu_percent(&self) -> f64 {
        self.cpu_usage.clamp(0.0, 100.0)
    }

    pub fn memory_percent(&self) -> f64 {
        self.memory_percent.clamp(0.0, 100.0)
    }

    pub fn disk_percent(&self) -> f64 {
        self.disk_percent.clamp(0.0, 100.0)
    }

    pub fn uptime_human(&self) -> String {
        let days = self.uptime / 86_400;
        let hours = (self.uptime % 86_400) / 3_600;
        let minutes = (self.uptime % 3_600) / 60;
        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

pub struct ProcessMonitor {
    sys: System,
    disks: Disks,
    processes: Vec<ProcessInfo>,
    stats: SystemStats,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            processes: Vec::new(),
            stats: SystemStats::default(),
        }
    }

    /// Full refresh that samples CPU usage over a real time interval.
    pub fn refresh(&mut self) -> Result<()> {
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.sys.refresh_all();
        self.disks.refresh_list();
        self.collect();
        Ok(())
    }

    /// Refresh without blocking — used by the dashboard tick loop.
    pub fn refresh_quick(&mut self) -> Result<()> {
        self.sys.refresh_all();
        self.disks.refresh_list();
        self.collect();
        Ok(())
    }

    fn collect(&mut self) {
        let total_mem = self.sys.total_memory().max(1);

        self.processes = self
            .sys
            .processes()
            .values()
            .map(|p| ProcessInfo {
                pid: p.pid().as_u32(),
                name: p.name().to_string(),
                cpu_usage: p.cpu_usage() as f64,
                memory_usage: p.memory(),
                memory_percent: p.memory() as f64 / total_mem as f64 * 100.0,
                status: format!("{}", p.status()),
                start_time: format_start_time(p.start_time()),
                command: command_of(p),
            })
            .collect();

        let cpus = self.sys.cpus();
        let cpu_usage = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };

        let mut disk_total = 0u64;
        let mut disk_used = 0u64;
        for disk in self.disks.list() {
            disk_total = disk_total.saturating_add(disk.total_space());
            disk_used = disk_used.saturating_add(
                disk.total_space().saturating_sub(disk.available_space()),
            );
        }

        let memory_used = self.sys.used_memory();

        self.stats = SystemStats {
            cpu_usage: cpu_usage as f64,
            memory_total: total_mem,
            memory_used,
            memory_percent: memory_used as f64 / total_mem as f64 * 100.0,
            disk_total,
            disk_used,
            disk_percent: if disk_total > 0 {
                disk_used as f64 / disk_total as f64 * 100.0
            } else {
                0.0
            },
            load_average: load_average(),
            uptime: System::uptime(),
            process_count: self.processes.len(),
        };
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
        let output = {
            #[cfg(unix)]
            {
                std::process::Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .output()
            }
            #[cfg(windows)]
            {
                std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output()
            }
            #[cfg(not(any(unix, windows)))]
            {
                bail!("Killing processes is not supported on this platform")
            }
        }
        .context("Failed to run the kill command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Could not kill process {}: {}",
                pid,
                stderr.trim().if_empty("permission denied or process not found")
            );
        }
        Ok(())
    }

    pub fn get_top_cpu(&self, limit: usize) -> Vec<&ProcessInfo> {
        let mut processes: Vec<&ProcessInfo> = self.processes.iter().collect();
        processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.into_iter().take(limit).collect()
    }

    pub fn get_top_memory(&self, limit: usize) -> Vec<&ProcessInfo> {
        let mut processes: Vec<&ProcessInfo> = self.processes.iter().collect();
        processes.sort_by(|a, b| {
            b.memory_usage
                .partial_cmp(&a.memory_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        processes.into_iter().take(limit).collect()
    }
}

fn command_of(process: &sysinfo::Process) -> String {
    let cmd = process
        .cmd()
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    if !cmd.is_empty() {
        return cmd;
    }
    match process.exe() {
        Some(exe) => exe.to_string_lossy().into_owned(),
        None => process.name().to_string(),
    }
}

fn format_start_time(timestamp: u64) -> String {
    if timestamp == 0 {
        return "-".to_string();
    }
    chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(unix)]
fn load_average() -> [f64; 3] {
    let load = System::load_average();
    [load.one, load.five, load.fifteen]
}

#[cfg(not(unix))]
fn load_average() -> [f64; 3] {
    [0.0, 0.0, 0.0]
}

trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl IfEmpty for &str {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self.to_string()
        }
    }
}
