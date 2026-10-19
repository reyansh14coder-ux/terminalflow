use anyhow::Result;
use colored::*;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub status: String,
    pub running: bool,
}

/// Structured container list — used by `tf docker` and the dashboard.
pub fn list_containers() -> Result<Vec<Container>> {
    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.Names}}\t{{.Image}}\t{{.Status}}",
        ])
        .output()
        .map_err(|_| anyhow::anyhow!("Docker is not installed or not on PATH"))?;

    if !output.status.success() {
        anyhow::bail!(
            "Docker daemon is not running: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let containers = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut parts = line.splitn(3, '\t');
            let name = parts.next().unwrap_or("").to_string();
            let image = parts.next().unwrap_or("").to_string();
            let status = parts.next().unwrap_or("").to_string();
            let running = status.starts_with("Up");
            Container {
                name,
                image,
                status,
                running,
            }
        })
        .collect();

    Ok(containers)
}

pub async fn manage() -> Result<()> {
    println!("{}", "🐳 Docker Manager".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    match list_containers() {
        Ok(containers) => {
            println!("{}", "Containers:".green().bold());
            println!();

            if containers.is_empty() {
                println!("  {}", "No containers yet".dimmed());
            }

            for container in containers {
                if container.running {
                    println!(
                        "  {} {} {}",
                        "✅".green(),
                        container.name.green(),
                        format!("— {}", container.status).dimmed()
                    );
                } else {
                    println!(
                        "  {} {} {}",
                        "⏹️".red(),
                        container.name.red(),
                        format!("— {}", container.status).dimmed()
                    );
                }
            }
        }
        Err(err) => {
            println!("  {} {}", "⚠️".yellow(), err.to_string().yellow());
        }
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!();
    println!(
        "  {} Start  {} Stop   {} Restart   {} Logs",
        "[s]".green(),
        "[x]".red(),
        "[r]".yellow(),
        "[l]".cyan()
    );

    Ok(())
}
