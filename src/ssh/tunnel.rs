use anyhow::{Context, Result};
use std::process::{Command, Child, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SSHTunnel {
    local_port: u16,
    remote_host: String,
    remote_port: u16,
    ssh_host: String,
    ssh_user: String,
    process: Option<Child>,
}

impl SSHTunnel {
    pub fn new(
        local_port: u16,
        remote_host: &str,
        remote_port: u16,
        ssh_host: &str,
        ssh_user: &str,
    ) -> Self {
        Self {
            local_port,
            remote_host: remote_host.to_string(),
            remote_port,
            ssh_host: ssh_host.to_string(),
            ssh_user: ssh_user.to_string(),
            process: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let output = Command::new("ssh")
            .args([
                "-L",
                &format!("{}:{}:{}", self.local_port, self.remote_host, self.remote_port),
                "-N",
                "-f",
                &format!("{}@{}", self.ssh_user, self.ssh_host),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start SSH tunnel")?;
        
        self.process = Some(output);
        
        println!(
            "✅ SSH tunnel established: localhost:{} -> {}:{} via {}@{}",
            self.local_port, self.remote_host, self.remote_port, self.ssh_user, self.ssh_host
        );
        
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill().context("Failed to stop SSH tunnel")?;
            println!("🛑 SSH tunnel stopped");
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    pub fn status(&self) -> TunnelStatus {
        if self.is_running() {
            TunnelStatus::Running {
                local_port: self.local_port,
                remote_host: self.remote_host.clone(),
                remote_port: self.remote_port,
            }
        } else {
            TunnelStatus::Stopped
        }
    }
}

#[derive(Debug, Clone)]
pub enum TunnelStatus {
    Running {
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    },
    Stopped,
}

impl std::fmt::Display for TunnelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelStatus::Running { local_port, remote_host, remote_port } => {
                write!(
                    f,
                    "✅ Running: localhost:{} -> {}:{}",
                    local_port, remote_host, remote_port
                )
            }
            TunnelStatus::Stopped => write!(f, "🛑 Stopped"),
        }
    }
}

pub struct TunnelManager {
    tunnels: Vec<Arc<Mutex<SSHTunnel>>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: Vec::new(),
        }
    }

    pub fn add_tunnel(&mut self, tunnel: SSHTunnel) -> Arc<Mutex<SSHTunnel>> {
        let tunnel = Arc::new(Mutex::new(tunnel));
        self.tunnels.push(tunnel.clone());
        tunnel
    }

    pub async fn start_all(&self) -> Result<()> {
        for tunnel in &self.tunnels {
            let mut t = tunnel.lock().await;
            t.start()?;
        }
        Ok(())
    }

    pub async fn stop_all(&self) -> Result<()> {
        for tunnel in &self.tunnels {
            let mut t = tunnel.lock().await;
            t.stop()?;
        }
        Ok(())
    }

    pub async fn status_all(&self) -> Vec<TunnelStatus> {
        let mut statuses = Vec::new();
        for tunnel in &self.tunnels {
            let t = tunnel.lock().await;
            statuses.push(t.status());
        }
        statuses
    }
}
