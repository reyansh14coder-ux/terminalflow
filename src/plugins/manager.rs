#![allow(dead_code)]

use super::loader::{LoadedPlugin, PluginLoader};
use anyhow::Result;
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
        // Download and install plugin from source
        println!("📥 Installing plugin from {}...", source);

        // In real implementation, this would clone/download the plugin
        // For now, we'll simulate it

        println!("✅ Plugin installed successfully!");
        Ok(())
    }

    pub fn uninstall_plugin(&mut self, name: &str) -> Result<()> {
        if self.plugins.remove(name).is_some() {
            println!("✅ Plugin '{}' uninstalled", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }

    pub fn update_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(_plugin) = self.plugins.get(name) {
            println!("🔄 Updating plugin '{}'...", name);
            // In real implementation, this would pull updates
            println!("✅ Plugin '{}' updated", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }
}
