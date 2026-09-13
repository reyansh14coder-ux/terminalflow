use anyhow::Result;

pub struct Encryption {
    key: Vec<u8>,
}

impl Encryption {
    pub fn new(password: &str) -> Result<Self> {
        let key = password.as_bytes().to_vec();
        
        Ok(Self { key })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encrypted = data.to_vec();
        
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= self.key[i % self.key.len()];
        }
        
        Ok(encrypted)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decrypted = data.to_vec();
        
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= self.key[i % self.key.len()];
        }
        
        Ok(decrypted)
    }

    pub fn hash_password(password: &str) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }

    pub fn generate_salt() -> Vec<u8> {
        use rand::Rng;
        
        let mut rng = rand::thread_rng();
        (0..16).map(|_| rng.gen::<u8>()).collect()
    }

    pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32) -> Vec<u8> {
        let mut key = Vec::new();
        
        for i in 0..iterations {
            let mut hash = Vec::new();
            hash.extend_from_slice(password);
            hash.extend_from_slice(salt);
            hash.extend_from_slice(&i.to_le_bytes());
            
            key.extend_from_slice(&hash);
        }
        
        key.truncate(32);
        key
    }
}
