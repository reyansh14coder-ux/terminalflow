#![allow(dead_code)]

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use super::encrypt::{hex_decode, hex_encode, Encryption};

const KDF_ITERATIONS: u32 = 60_000;
const META_FILE: &str = "meta.json";

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultMeta {
    salt: String,
    iterations: u32,
}

/// Password-protected vault. Unlocking writes a session file so follow-up
/// commands (`store`, `get`, `list`, `backup`) work non-interactively until
/// `tf vault lock` is run.
pub struct SecretVault {
    path: PathBuf,
    key: Option<[u8; 32]>,
}

impl SecretVault {
    pub fn new(path: PathBuf) -> Result<Self> {
        let vault_path = path.join("vault");
        std::fs::create_dir_all(&vault_path)?;

        let mut vault = Self {
            path: vault_path,
            key: None,
        };

        // Resume a previous unlock when the session file is still around.
        if let Some(key) = vault.read_session()? {
            vault.key = Some(key);
        }

        Ok(vault)
    }

    fn meta_path(&self) -> PathBuf {
        self.path.join(META_FILE)
    }

    fn session_path(&self) -> PathBuf {
        self.path.join(".session")
    }

    fn meta(&self) -> Result<VaultMeta> {
        match std::fs::read_to_string(self.meta_path()) {
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(_) => Ok(VaultMeta {
                salt: hex_encode(&Encryption::random_bytes(16)),
                iterations: KDF_ITERATIONS,
            }),
        }
    }

    fn write_meta(&self, meta: &VaultMeta) -> Result<()> {
        std::fs::write(self.meta_path(), serde_json::to_string_pretty(meta)?)?;
        Ok(())
    }

    fn read_session(&self) -> Result<Option<[u8; 32]>> {
        let raw = match std::fs::read_to_string(self.session_path()) {
            Ok(raw) => raw,
            Err(_) => return Ok(None),
        };
        match hex_decode(raw.trim()) {
            Ok(bytes) => match bytes.try_into() {
                Ok(key) => Ok(Some(key)),
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    fn write_session(&self) -> Result<()> {
        if let Some(key) = self.key {
            std::fs::write(self.session_path(), hex_encode(&key))?;
        }
        Ok(())
    }

    pub fn unlock(&mut self, password: &str) -> Result<()> {
        if password.trim().is_empty() {
            bail!("Password must not be empty");
        }

        let meta = self.meta()?;
        if !self.meta_path().exists() {
            self.write_meta(&meta)?;
        }

        let salt = hex_decode(&meta.salt).context("Vault salt is corrupted")?;
        let key = Encryption::derive_key(password, &salt, meta.iterations);

        self.key = Some(key);
        self.write_session()?;
        println!("🔓 Vault unlocked");
        Ok(())
    }

    pub fn lock(&mut self) -> Result<()> {
        self.key = None;
        let session = self.session_path();
        if session.exists() {
            std::fs::remove_file(session)?;
        }
        println!("🔒 Vault locked");
        Ok(())
    }

    pub fn is_unlocked(&self) -> bool {
        self.key.is_some()
    }

    fn require_key(&self) -> Result<&[u8; 32]> {
        self.key
            .as_ref()
            .context("Vault is locked — run `tf vault unlock` first")
    }

    pub fn store(&self, name: &str, data: &str) -> Result<()> {
        let key = self.require_key()?;
        let encryption = Encryption::from_key(*key);
        let payload = encryption.encrypt_str(data)?;
        std::fs::write(self.entry_path(name), payload)?;
        Ok(())
    }

    pub fn retrieve(&self, name: &str) -> Result<String> {
        let key = self.require_key()?;
        let path = self.entry_path(name);
        let payload = std::fs::read_to_string(&path)
            .with_context(|| format!("Secret '{}' not found in the vault", name))?;
        let encryption = Encryption::from_key(*key);
        encryption.decrypt_str(payload.trim())
    }

    fn entry_path(&self, name: &str) -> PathBuf {
        self.path.join(format!("{}.enc", name))
    }

    pub fn list_secrets(&self) -> Result<Vec<String>> {
        let mut secrets = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".enc") {
                        secrets.push(name.trim_end_matches(".enc").to_string());
                    }
                }
            }
        }

        secrets.sort();
        Ok(secrets)
    }

    /// Write a single-file backup (entries stay encrypted).
    pub fn backup(&self, backup_path: &Path) -> Result<()> {
        let mut entries = serde_json::Map::new();

        for name in self.list_secrets()? {
            let payload = std::fs::read_to_string(self.entry_path(&name))?;
            entries.insert(name, serde_json::Value::String(payload));
        }

        let backup = serde_json::json!({
            "version": 1,
            "meta": std::fs::read_to_string(self.meta_path()).ok().and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok()),
            "entries": serde_json::Value::Object(entries),
        });

        if let Some(parent) = backup_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        std::fs::write(backup_path, serde_json::to_string_pretty(&backup)?)?;
        println!("✅ Vault backed up to {}", backup_path.display());
        Ok(())
    }

    /// Restore entries from a backup written by `backup`.
    pub fn restore(&mut self, backup_path: &Path) -> Result<()> {
        let raw = std::fs::read_to_string(backup_path)
            .with_context(|| format!("Backup {} not found", backup_path.display()))?;
        let backup: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("{} is not a TerminalFlow vault backup", backup_path.display()))?;

        let entries = backup
            .get("entries")
            .and_then(|e| e.as_object())
            .context("Backup has no entries")?;

        for (name, value) in entries {
            if let Some(payload) = value.as_str() {
                std::fs::write(self.entry_path(name), payload)?;
            }
        }

        if let Some(meta_value) = backup.get("meta") {
            if !self.meta_path().exists() {
                std::fs::write(self.meta_path(), serde_json::to_string_pretty(meta_value)?)?;
            }
        }

        println!("✅ Restored {} secret(s)", entries.len());
        Ok(())
    }
}
