use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use aes_gcm::AeadCore;
use base64::{Engine as _, engine::general_purpose};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

pub struct Encryption {
    cipher: Aes256Gcm,
}

impl Encryption {
    pub fn new(key: &[u8]) -> Result<Self> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }
    
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(general_purpose::STANDARD.encode(result))
    }
    
    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let data = general_purpose::STANDARD.decode(encrypted)?;
        
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        Ok(String::from_utf8(plaintext)?)
    }
    
    pub fn generate_key() -> Vec<u8> {
        use rand::Rng;
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill(&mut key[..]);
        key
    }

    pub fn load_or_init_key() -> anyhow::Result<Vec<u8>> {
        let proj_dirs = ProjectDirs::from("com", "productivityapp", "app")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        let key_path: PathBuf = proj_dirs.config_dir().join("key.bin");
        if key_path.exists() {
            let bytes = fs::read(&key_path)?;
            if bytes.len() == 32 { Ok(bytes) } else { Err(anyhow::anyhow!("Invalid key size")) }
        } else {
            let key = Self::generate_key();
            fs::create_dir_all(proj_dirs.config_dir())?;
            fs::write(&key_path, &key)?;
            Ok(key)
        }
    }
}