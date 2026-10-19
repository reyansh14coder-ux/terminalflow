use anyhow::Result;
use colored::*;

use crate::process::ProcessMonitor;

fn bar(percent: f64, width: usize) -> String {
    let filled = ((percent.clamp(0.0, 100.0) / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

fn paint(percent: f64, text: String) -> ColoredString {
    if percent > 80.0 {
        text.red()
    } else if percent > 60.0 {
        text.yellow()
    } else {
        text.green()
    }
}

pub fn format_bytes(value: u64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    let value = value as f64;
    if value >= GB {
        format!("{:.1} GB", value / GB)
    } else {
        format!("{:.1} MB", value / MB)
    }
}

pub async fn show() -> Result<()> {
    let mut monitor = ProcessMonitor::new();
    monitor.refresh()?;
    let stats = monitor.get_stats().clone();

    println!("{}", "📈 System Monitor".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    let cpu = stats.cpu_percent();
    println!(
        "  CPU:      {} {:>5.1}%",
        paint(cpu, bar(cpu, 24)).to_string(),
        cpu
    );

    let mem = stats.memory_percent();
    println!(
        "  Memory:   {} {:>5.1}%  ({} / {})",
        paint(mem, bar(mem, 24)).to_string(),
        mem,
        format_bytes(stats.memory_used),
        format_bytes(stats.memory_total)
    );

    let disk = stats.disk_percent();
    println!(
        "  Disk:     {} {:>5.1}%  ({} / {})",
        paint(disk, bar(disk, 24)).to_string(),
        disk,
        format_bytes(stats.disk_used),
        format_bytes(stats.disk_total)
    );

    println!();
    println!(
        "  {} Uptime: {}  │  Processes: {}  │  Load: {:.2} {:.2} {:.2}",
        "⏱".dimmed(),
        stats.uptime_human(),
        stats.process_count,
        stats.load_average[0],
        stats.load_average[1],
        stats.load_average[2]
    );

    println!();
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "  🔝 Top processes by CPU".cyan().bold());

    for process in monitor.get_top_cpu(5) {
        println!(
            "    {:<24} {:>6.1}%  {:>10}  PID {}",
            truncate(&process.name, 24),
            process.cpu_usage,
            format_bytes(process.memory_usage),
            process.pid
        );
    }

    println!("{}", "─".repeat(60).dimmed());

    Ok(())
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        let mut out: String = value.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
