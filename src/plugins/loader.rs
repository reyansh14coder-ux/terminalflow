#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

    /// Directory new plugins are installed into (`~/.terminalflow/plugins`).
    pub fn user_plugin_dir(&self) -> PathBuf {
        self.plugin_dirs
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("plugins"))
    }

    /// Install from a local directory that contains a `plugin.toml`.
    pub fn install_from_local(&self, source: &Path) -> Result<PluginManifest> {
        let manifest_path = source.join("plugin.toml");
        let manifest = self.load_manifest(&manifest_path)?;

        let target = self.user_plugin_dir().join(&manifest.name);
        if target.exists() {
            std::fs::remove_dir_all(&target)
                .context("Failed to replace the existing plugin")?;
        }
        copy_dir(source, &target)
            .context("Failed to copy the plugin into place")?;

        Ok(manifest)
    }

    /// Install from a git URL (cloned into the user plugin directory).
    pub fn install_from_git(&self, url: &str) -> Result<PluginManifest> {
        let name = url
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("plugin")
            .trim_end_matches(".git");

        let target = self.user_plugin_dir().join(name);
        if target.exists() {
            std::fs::remove_dir_all(&target)
                .context("Failed to replace the existing plugin")?;
        }

        let output = std::process::Command::new("git")
            .args(["clone", "--depth", "1", url])
            .arg(&target)
            .output()
            .context("Failed to run git — is git installed?")?;

        if !output.status.success() {
            anyhow::bail!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        let manifest_path = target.join("plugin.toml");
        if !manifest_path.exists() {
            std::fs::remove_dir_all(&target).ok();
            anyhow::bail!("The cloned repository has no plugin.toml, so it is not a TerminalFlow plugin");
        }

        self.load_manifest(&manifest_path)
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
        let content = std::fs::read_to_string(path).context("Failed to read plugin manifest")?;

        let manifest: PluginManifest =
            toml::from_str(&content).context("Failed to parse plugin manifest")?;

        Ok(manifest)
    }

    pub fn load_plugin(&self, manifest: &PluginManifest) -> Result<LoadedPlugin> {
        let plugin_dir = self
            .find_plugin_dir(&manifest.name)
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

fn copy_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let to = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
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

        String::from_utf8(output.stdout).context("Failed to parse plugin output")
    }
}
