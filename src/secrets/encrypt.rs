#![allow(dead_code)]

use anyhow::{Context, Result};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use rand::RngCore;

const ENC_PREFIX: &str = "xchacha20poly1305:v1";
const KEY_FILE_VERSION: &str = "key:v1";

/// XChaCha20-Poly1305 helper used by the secrets vault.
pub struct Encryption {
    key: [u8; 32],
}

impl Encryption {
    /// Random key, used for a newly created local key file.
    pub fn random_key() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self { key }
    }

    pub fn from_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Load the key at `path`, creating it (with 0600 permissions) on first use.
    pub fn load_or_create_key(path: &std::path::Path) -> Result<Self> {
        if path.exists() {
            let raw = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read key file {}", path.display()))?;
            let raw = raw.trim();
            let raw = raw.strip_prefix(KEY_FILE_VERSION).unwrap_or(raw);
            let key = hex_decode(raw).context("Key file is corrupted — delete it to re-key")?;
            let key: [u8; 32] = key
                .try_into()
                .map_err(|_| anyhow::anyhow!("Key file must contain a 32-byte key"))?;
            return Ok(Self::from_key(key));
        }

        let encryption = Self::random_key();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }

        let content = format!("{}\n{}\n", KEY_FILE_VERSION, hex_encode(&encryption.key));
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write key file {}", path.display()))?;
        restrict_permissions(path);

        Ok(encryption)
    }

    /// Encrypt UTF-8 text. Output is self-describing: `prefix:nonce:ct` in hex.
    pub fn encrypt_str(&self, plaintext: &str) -> Result<String> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .context("Invalid encryption key")?;

        let mut nonce_bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| anyhow::anyhow!("Encryption failed"))?;

        Ok(format!(
            "{}:{}:{}",
            ENC_PREFIX,
            hex_encode(&nonce_bytes),
            hex_encode(&ciphertext)
        ))
    }

    pub fn decrypt_str(&self, payload: &str) -> Result<String> {
        let bytes = self.decrypt_bytes(payload)?;
        String::from_utf8(bytes).context("Decrypted value is not valid UTF-8")
    }

    pub fn decrypt_bytes(&self, payload: &str) -> Result<Vec<u8>> {
        // Legacy plaintext values (created before encryption existed) pass
        // through untouched so existing installs keep working.
        if !payload.starts_with(ENC_PREFIX) {
            return Ok(payload.as_bytes().to_vec());
        }

        let parts: Vec<&str> = payload.split(':').collect();
        if parts.len() != 4 || parts[1] != "v1" {
            anyhow::bail!("Unsupported encrypted value format");
        }

        let nonce = hex_decode(parts[2]).context("Corrupt nonce")?;
        let nonce: [u8; 24] = nonce
            .try_into()
            .map_err(|_| anyhow::anyhow!("Corrupt nonce length"))?;
        let ciphertext = hex_decode(parts[3]).context("Corrupt ciphertext")?;

        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .context("Invalid encryption key")?;

        cipher
            .decrypt(XNonce::from_slice(&nonce), ciphertext.as_slice())
            .map_err(|_| anyhow::anyhow!("Decryption failed — wrong key or tampered data"))
    }

    pub fn is_encrypted(payload: &str) -> bool {
        payload.starts_with(ENC_PREFIX)
    }

    /// PBKDF2-HMAC-SHA256 key derivation for password-based encryption.
    pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> [u8; 32] {
        pbkdf2::pbkdf2_hmac_array::<sha2::Sha256, 32>(password.as_bytes(), salt, iterations)
    }

    pub fn random_bytes(len: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut buffer);
        buffer
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &std::path::Path) {}

pub fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn hex_decode(value: &str) -> Result<Vec<u8>> {
    let value = value.trim();
    if value.len() % 2 != 0 {
        anyhow::bail!("Hex string must have an even length");
    }
    (0..value.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&value[i..i + 2], 16)
                .context("Invalid hex character")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_plaintext() {
        let enc = Encryption::random_key();
        let payload = enc.encrypt_str("postgres://user:pass@db").unwrap();
        assert!(Encryption::is_encrypted(&payload));
        assert_eq!(enc.decrypt_str(&payload).unwrap(), "postgres://user:pass@db");
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let payload = Encryption::random_key().encrypt_str("secret").unwrap();
        let result = Encryption::random_key().decrypt_str(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn legacy_plaintext_still_readable() {
        let enc = Encryption::random_key();
        assert_eq!(enc.decrypt_str("plain-old-value").unwrap(), "plain-old-value");
    }

    #[test]
    fn hex_round_trip() {
        let data = vec![0u8, 1, 0x7f, 0x80, 0xff];
        assert_eq!(hex_decode(&hex_encode(&data)).unwrap(), data);
    }

    #[test]
    fn derived_keys_are_deterministic() {
        let salt = b"0123456789abcdef";
        let a = Encryption::derive_key("hunter2", salt, 1000);
        let b = Encryption::derive_key("hunter2", salt, 1000);
        let c = Encryption::derive_key("hunter3", salt, 1000);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
