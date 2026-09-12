use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub commands: Vec<PluginCommand>,
    pub hooks: Vec<PluginHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    pub event: HookEvent,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookEvent {
    PreCommit,
    PostCommit,
    PrePush,
    PostPush,
    PreTest,
    PostTest,
    FileChanged,
    Error,
    Startup,
    Shutdown,
}

pub struct PluginLoader {
    plugin_dirs: Vec<PathBuf>,
}

impl PluginLoader {
    pub fn new() -> Self {
        let mut dirs = Vec::new();
        
        // User plugins directory
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".terminalflow").join("plugins"));
        }
        
        // System plugins directory
        dirs.push(PathBuf::from("/usr/local/share/terminalflow/plugins"));
        dirs.push(PathBuf::from("/usr/share/terminalflow/plugins"));
        
        Self { plugin_dirs: dirs }
    }

    pub fn discover_plugins(&self) -> Result<Vec<PluginManifest>> {
        let mut plugins = Vec::new();
        
        for dir in &self.plugin_dirs {
            if dir.exists() {
                self.scan_directory(dir, &mut plugins)?;
            }
        }
        
        Ok(plugins)
    }

    fn scan_directory(&self, dir: &Path, plugins: &mut Vec<PluginManifest>) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("plugin.toml");
                    if manifest_path.exists() {
                        if let Ok(manifest) = self.load_manifest(&manifest_path) {
                            plugins.push(manifest);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn load_manifest(&self, path: &Path) -> Result<PluginManifest> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read plugin manifest")?;
        
        let manifest: PluginManifest = toml::from_str(&content)
            .context("Failed to parse plugin manifest")?;
        
        Ok(manifest)
    }

    pub fn load_plugin(&self, manifest: &PluginManifest) -> Result<LoadedPlugin> {
        let plugin_dir = self.find_plugin_dir(&manifest.name)
            .context("Plugin directory not found")?;
        
        let entry_point = plugin_dir.join(&manifest.entry_point);
        
        Ok(LoadedPlugin {
            manifest: manifest.clone(),
            directory: plugin_dir,
            entry_point,
        })
    }

    fn find_plugin_dir(&self, name: &str) -> Option<PathBuf> {
        for dir in &self.plugin_dirs {
            let plugin_dir = dir.join(name);
            if plugin_dir.exists() {
                return Some(plugin_dir);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub directory: PathBuf,
    pub entry_point: PathBuf,
}

impl LoadedPlugin {
    pub fn execute_command(&self, command: &str, args: &[String]) -> Result<String> {
        let output = std::process::Command::new(&self.entry_point)
            .arg(command)
            .args(args)
            .output()
            .context("Failed to execute plugin command")?;
        
        String::from_utf8(output.stdout)
            .context("Failed to parse plugin output")
    }
}
