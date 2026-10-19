#![allow(dead_code)]

use super::loader::{LoadedPlugin, PluginLoader};
use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
    loader: PluginLoader,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            loader: PluginLoader::new(),
        }
    }

    pub fn discover_and_load(&mut self) -> Result<()> {
        let manifests = self.loader.discover_plugins()?;

        for manifest in manifests {
            match self.loader.load_plugin(&manifest) {
                Ok(plugin) => {
                    println!("✅ Loaded plugin: {} v{}", manifest.name, manifest.version);
                    self.plugins.insert(manifest.name.clone(), plugin);
                }
                Err(e) => {
                    eprintln!("❌ Failed to load plugin {}: {}", manifest.name, e);
                }
            }
        }

        Ok(())
    }

    pub fn execute_command(
        &self,
        plugin_name: &str,
        command: &str,
        args: &[String],
    ) -> Result<String> {
        let plugin = self
            .plugins
            .get(plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", plugin_name))?;

        plugin.execute_command(command, args)
    }

    pub fn list_plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }

    pub fn get_plugin(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.get(name)
    }

    pub fn install_plugin(&mut self, source: &str) -> Result<()> {
        let path = std::path::Path::new(source);

        let manifest = if path.join("plugin.toml").exists() {
            println!("📦 Installing local plugin from {}…", source);
            self.loader.install_from_local(path)?
        } else if source.contains("://") || source.ends_with(".git") {
            println!("📦 Cloning plugin from {}…", source);
            self.loader.install_from_git(source)?
        } else {
            anyhow::bail!(
                "'{}' is neither a local plugin folder (containing plugin.toml) nor a git URL.\n  Example: tf plugin install https://github.com/user/tf-plugin-git.git",
                source
            );
        };

        self.plugins.clear();
        self.discover_and_load()?;
        println!(
            "✅ Installed {} v{} — run `tf plugin list` to see it",
            manifest.name, manifest.version
        );
        Ok(())
    }

    pub fn uninstall_plugin(&mut self, name: &str) -> Result<()> {
        let removed_from_memory = self.plugins.remove(name).is_some();

        let on_disk = self.loader.user_plugin_dir().join(name);
        let removed_from_disk = if on_disk.exists() {
            std::fs::remove_dir_all(&on_disk)?;
            true
        } else {
            false
        };

        if removed_from_memory || removed_from_disk {
            println!("✅ Plugin '{}' uninstalled", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Plugin '{}' not found. Install one with `tf plugin install <source>`",
                name
            ))
        }
    }

    pub fn update_plugin(&mut self, name: &str) -> Result<()> {
        let plugin_dir = self.loader.user_plugin_dir().join(name);
        if !plugin_dir.exists() {
            return Err(anyhow::anyhow!("Plugin '{}' not found", name));
        }

        let output = std::process::Command::new("git")
            .args(["-C"])
            .arg(&plugin_dir)
            .args(["pull", "--ff-only"])
            .output()
            .context("Failed to run git — is git installed?")?;

        if !output.status.success() {
            anyhow::bail!(
                "Update failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        println!("✅ Plugin '{}' updated", name);
        self.plugins.clear();
        self.discover_and_load()?;
        Ok(())
    }
}
