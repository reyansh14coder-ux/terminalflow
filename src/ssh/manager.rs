#![allow(dead_code)]

use std::collections::HashMap;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_path: Option<String>,
    pub tags: Vec<String>,
}

pub struct SSHManager {
    connections: HashMap<String, SSHConnection>,
    config_path: std::path::PathBuf,
}

impl SSHManager {
    pub fn new() -> Result<Self> {
        let config_path = dirs::home_dir()
            .context("Cannot find home directory")?
            .join(".terminalflow")
            .join("ssh.json");
        
        let mut manager = Self {
            connections: HashMap::new(),
            config_path,
        };
        
        manager.load()?;
        Ok(manager)
    }

    fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path)?;
            self.connections = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.connections)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add(&mut self, connection: SSHConnection) -> Result<()> {
        self.connections.insert(connection.name.clone(), connection);
        self.save()?;
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> Result<bool> {
        let removed = self.connections.remove(name).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    pub fn get(&self, name: &str) -> Option<&SSHConnection> {
        self.connections.get(name)
    }

    pub fn list(&self) -> Vec<&SSHConnection> {
        self.connections.values().collect()
    }

    pub fn connect(&self, name: &str) -> Result<()> {
        let conn = self.connections.get(name)
            .context("Connection not found")?;
        
        let mut args = vec![
            "-p".to_string(),
            conn.port.to_string(),
        ];
        
        if let Some(key) = &conn.key_path {
            args.push("-i".to_string());
            args.push(key.clone());
        }
        
        args.push(format!("{}@{}", conn.user, conn.host));
        
        std::process::Command::new("ssh")
            .args(&args)
            .status()
            .context("Failed to connect")?;
        
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<&SSHConnection> {
        self.connections.values()
            .filter(|c| {
                c.name.contains(query)
                    || c.host.contains(query)
                    || c.user.contains(query)
                    || c.tags.iter().any(|t| t.contains(query))
            })
            .collect()
    }
}
