#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub identity_file: Option<String>,
    pub forward_agent: bool,
    pub forward_x11: bool,
    pub compression: bool,
    pub keep_alive: u32,
    pub server_alive_interval: u32,
    pub strict_host_key_checking: String,
    pub log_level: String,
}

impl Default for SSHConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            user: "root".to_string(),
            identity_file: None,
            forward_agent: false,
            forward_x11: false,
            compression: true,
            keep_alive: 60,
            server_alive_interval: 30,
            strict_host_key_checking: "ask".to_string(),
            log_level: "INFO".to_string(),
        }
    }
}

impl SSHConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read SSH config")?;
        
        let config: Self = toml::from_str(&content)
            .context("Failed to parse SSH config")?;
        
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "-p".to_string(),
            self.port.to_string(),
        ];
        
        if let Some(key) = &self.identity_file {
            args.push("-i".to_string());
            args.push(key.clone());
        }
        
        if self.forward_agent {
            args.push("-A".to_string());
        }
        
        if self.forward_x11 {
            args.push("-X".to_string());
        }
        
        if self.compression {
            args.push("-C".to_string());
        }
        
        args.push("-o".to_string());
        args.push(format!("StrictHostKeyChecking={}", self.strict_host_key_checking));
        
        args.push("-o".to_string());
        args.push(format!("ServerAliveInterval={}", self.server_alive_interval));
        
        args
    }

    pub fn parse_ssh_config_file(path: &PathBuf) -> Result<Vec<Self>> {
        let mut configs = Vec::new();
        
        if !path.exists() {
            return Ok(configs);
        }
        
        let content = std::fs::read_to_string(path)?;
        let mut current_host: Option<String> = None;
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(host) = line.strip_prefix("Host ") {
                if let Some(h) = current_host.take() {
                    config.host = h;
                    configs.push(config);
                    config = Self::default();
                }
                current_host = Some(host.to_string());
            } else if let Some(value) = line.strip_prefix("HostName ") {
                config.host = value.to_string();
            } else if let Some(value) = line.strip_prefix("Port ") {
                if let Ok(port) = value.parse() {
                    config.port = port;
                }
            } else if let Some(value) = line.strip_prefix("User ") {
                config.user = value.to_string();
            } else if let Some(value) = line.strip_prefix("IdentityFile ") {
                config.identity_file = Some(value.to_string());
            } else if line == "ForwardAgent yes" {
                config.forward_agent = true;
            } else if line == "ForwardX11 yes" {
                config.forward_x11 = true;
            } else if line == "Compression yes" {
                config.compression = true;
            }
        }
        
        if let Some(h) = current_host {
            config.host = h;
            configs.push(config);
        }
        
        Ok(configs)
    }
}
