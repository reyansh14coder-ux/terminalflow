use anyhow::Result;
use colored::*;

pub async fn show() -> Result<()> {
    println!("{}", "📈 System Monitor".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    // Simulated system stats (in real implementation, use sysinfo crate)
    let cpu_usage = 45;
    let memory_usage = 62;
    let disk_usage = 34;

    println!("  {} System Resources", "📊".cyan().bold());
    println!();

    // CPU bar
    let cpu_bar = "█".repeat(cpu_usage as usize / 5) + &"░".repeat(20 - cpu_usage as usize / 5);
    let _cpu_color = if cpu_usage > 80 {
        "red"
    } else if cpu_usage > 60 {
        "yellow"
    } else {
        "green"
    };
    println!("  CPU:     {} {}%", cpu_bar.cyan(), cpu_usage);
    println!();

    // Memory bar
    let mem_bar =
        "█".repeat(memory_usage as usize / 5) + &"░".repeat(20 - memory_usage as usize / 5);
    println!("  Memory:  {} {}%", mem_bar.green(), memory_usage);
    println!();

    // Disk bar
    let disk_bar = "█".repeat(disk_usage as usize / 5) + &"░".repeat(20 - disk_usage as usize / 5);
    println!("  Disk:    {} {}%", disk_bar.blue(), disk_usage);
    println!();

    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!("  {} Press 'q' to exit", "💡 Tip:".dimmed());

    Ok(())
}
