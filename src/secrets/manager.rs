#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub value: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct SecretManager {
    secrets: HashMap<String, Secret>,
    vault_path: std::path::PathBuf,
}

impl SecretManager {
    pub fn new() -> Result<Self> {
        let vault_path = dirs::home_dir()
            .context("Cannot find home directory")?
            .join(".terminalflow")
            .join("secrets");

        std::fs::create_dir_all(&vault_path)?;

        let mut manager = Self {
            secrets: HashMap::new(),
            vault_path,
        };

        manager.load_secrets()?;
        Ok(manager)
    }

    fn load_secrets(&mut self) -> Result<()> {
        let secrets_file = self.vault_path.join("secrets.json");
        if secrets_file.exists() {
            let content = std::fs::read_to_string(&secrets_file)?;
            self.secrets = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    fn save_secrets(&self) -> Result<()> {
        let secrets_file = self.vault_path.join("secrets.json");
        let content = serde_json::to_string_pretty(&self.secrets)?;
        std::fs::write(secrets_file, content)?;
        Ok(())
    }

    pub fn set(&mut self, name: &str, value: &str, description: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now();

        let secret = Secret {
            name: name.to_string(),
            value: value.to_string(),
            description: description.map(|s| s.to_string()),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        self.secrets.insert(name.to_string(), secret);
        self.save_secrets()?;

        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Option<String>> {
        Ok(self.secrets.get(name).map(|s| s.value.clone()))
    }

    pub fn delete(&mut self, name: &str) -> Result<bool> {
        let removed = self.secrets.remove(name).is_some();
        if removed {
            self.save_secrets()?;
        }
        Ok(removed)
    }

    pub fn list(&self) -> Vec<&Secret> {
        self.secrets.values().collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Secret> {
        self.secrets
            .values()
            .filter(|s| {
                s.name.contains(query)
                    || s.description.as_deref().unwrap_or("").contains(query)
                    || s.tags.iter().any(|t| t.contains(query))
            })
            .collect()
    }

    pub fn set_tag(&mut self, name: &str, tag: &str) -> Result<()> {
        if let Some(secret) = self.secrets.get_mut(name) {
            if !secret.tags.contains(&tag.to_string()) {
                secret.tags.push(tag.to_string());
                secret.updated_at = chrono::Utc::now();
                self.save_secrets()?;
            }
        }
        Ok(())
    }

    pub fn import_env(&mut self, prefix: &str) -> Result<u32> {
        let mut count = 0;

        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let name = key.strip_prefix(prefix).unwrap_or(&key);
                self.set(name, &value, Some("Imported from environment"))?;
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn export_env(&self, prefix: &str) -> Result<HashMap<String, String>> {
        let mut env_vars = HashMap::new();

        for secret in self.secrets.values() {
            let key = format!("{}{}", prefix, secret.name);
            env_vars.insert(key, secret.value.clone());
        }

        Ok(env_vars)
    }
}
