use anyhow::{Result, Context};
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};
use ring::hkdf::derive_key;
use ring::hkdf::HKDF_SHA256;
use ring::pbkdf2::{derive, PBKDF2_HMAC_SHA256};
use ring::pbkdf2::PBKDF2_HMAC_SHA256;
use ring::error::Unspecified;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;
use tracing::{info, warn, debug};
use std::collections::HashMap;

use crate::types::*;

/// Encryption engine for secure backups
pub struct EncryptionEngine {
    rng: SystemRandom,
}

impl EncryptionEngine {
    /// Create a new encryption engine
    pub async fn new() -> Result<Self> {
        info!("Initializing encryption engine");
        let rng = SystemRandom::new();
        Ok(Self { rng })
    }
    
    /// Encrypt data using specified settings
    pub async fn encrypt(
        &self,
        data: &[u8],
        settings: &EncryptionSettings,
    ) -> Result<Vec<u8>> {
        if !settings.enabled {
            return Ok(data.to_vec());
        }
        
        // Generate salt if not provided
        let salt = match &settings.salt {
            Some(salt) => salt.clone(),
            None => self.generate_salt()?,
        };
        
        // Derive key from password/passphrase
        let key = self.derive_key("backup_key", &salt, &settings.key_derivation)?;
        
        // Generate IV for AES
        let iv = self.generate_iv()?;
        
        // Encrypt data
        let encrypted_data = self.encrypt_aes_gcm(&data, &key, &iv)?;
        
        // Package encrypted data with metadata
        self.package_encrypted_data(&encrypted_data, &iv, &salt)
    }
    
    /// Decrypt data
    pub async fn decrypt(
        &self,
        encrypted_data: &[u8],
        metadata: &Value,
    ) -> Result<Vec<u8>> {
        // Extract encryption parameters from metadata
        let params = self.extract_encryption_params(metadata)?;
        
        if !params.enabled {
            return Ok(encrypted_data.to_vec());
        }
        
        // Extract encrypted data, IV, and salt
        let (encrypted, iv, salt) = self.unpackage_encrypted_data(encrypted_data, &params)?;
        
        // Derive key
        let key = self.derive_key("backup_key", &salt, &params.key_derivation)?;
        
        // Decrypt data
        let decrypted_data = self.decrypt_aes_gcm(&encrypted, &key, &iv)?;
        
        debug!("Decrypted {} bytes successfully", decrypted_data.len());
        Ok(decrypted_data)
    }
    
    /// Generate random salt
    fn generate_salt(&self) -> Result<Vec<u8>> {
        let mut salt = vec![0u8; 32]; // 256-bit salt
        self.rng.fill(&mut salt).map_err(|_| Unspecified)?;
        Ok(salt)
    }
    
    /// Generate random IV
    fn generate_iv(&self) -> Result<Vec<u8>> {
        let mut iv = vec![0u8; 12]; // 96-bit IV for AES-GCM
        self.rng.fill(&mut iv).map_err(|_| Unspecified)?;
        Ok(iv)
    }
    
    /// Derive encryption key from password
    fn derive_key(
        &self,
        password: &str,
        salt: &[u8],
        key_derivation: &str,
    ) -> Result<Vec<u8>> {
        match key_derivation {
            "PBKDF2" => {
                let mut key = vec![0u8; 32]; // 256-bit key
                let password_bytes = password.as_bytes();
                
                derive(
                    PBKDF2_HMAC_SHA256,
                    password_bytes,
                    salt,
                    100_000, // Iterations
                    &mut key,
                );
                
                Ok(key)
            }
            "HKDF" => {
                // Simple HKDF derivation (would need password stretching in production)
                let salt_info = b"backup_salt";
                let derived = derive_key(
                    HKDF_SHA256,
                    password.as_bytes(),
                    salt,
                    salt_info,
                    32, // Key length
                );
                Ok(derived.as_ref().to_vec())
            }
            _ => {
                warn!("Unknown key derivation method: {}, using PBKDF2", key_derivation);
                self.derive_key(password, salt, "PBKDF2")
            }
        }
    }
    
    /// Encrypt using AES-GCM
    fn encrypt_aes_gcm(&self, data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
        use ring::aead::{Aad, AES_256_GCM, BoundKey, SealingKey, UnboundKey};
        
        // Create key object
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| Unspecified)?;
        
        // Create sealing key
        let mut key = SealingKey::new(unbound_key, iv);
        
        // Create nonce sequence
        let nonce_sequence = ring::aead::NonceSequence::new(iv);
        
        // Encrypt data
        let mut sealed = Vec::new();
        sealed.extend_from_slice(data);
        let aad = Aad::empty();
        
        key.seal_in_place_separate_tag(aad, &mut sealed)
            .map_err(|_| Unspecified)?;
        
        Ok(sealed)
    }
    
    /// Decrypt using AES-GCM
    fn decrypt_aes_gcm(&self, sealed_data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
        use ring::aead::{Aad, AES_256_GCM, BoundKey, OpeningKey, UnboundKey};
        
        // Create key object
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|_| Unspecified)?;
        
        // Create opening key
        let mut key = OpeningKey::new(unbound_key, iv);
        
        // Decrypt data
        let mut decrypted = sealed_data.to_vec();
        let aad = Aad::empty();
        
        let tag_len = 16; // 128-bit tag
        let data_len = decrypted.len() - tag_len;
        
        key.open_in_place(aad, &mut decrypted)
            .map(|_| &mut decrypted[0..data_len])?;
        
        Ok(decrypted[0..data_len].to_vec())
    }
    
    /// Package encrypted data with metadata
    fn package_encrypted_data(&self, encrypted: &[u8], iv: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
        let mut package = Vec::new();
        
        // Add magic number
        package.extend_from_slice(b"MULTIOS_BKUP_V1");
        
        // Add IV length and IV
        package.push(iv.len() as u8);
        package.extend_from_slice(iv);
        
        // Add salt length and salt
        package.push(salt.len() as u8);
        package.extend_from_slice(salt);
        
        // Add encrypted data
        package.extend_from_slice(encrypted);
        
        Ok(package)
    }
    
    /// Unpackage encrypted data
    fn unpackage_encrypted_data(
        &self,
        package: &[u8],
        params: &EncryptionParams,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        if package.len() < 24 { // Minimum package size
            bail!("Invalid package format");
        }
        
        // Check magic number
        if &package[0..15] != b"MULTIOS_BKUP_V1" {
            bail!("Invalid package magic number");
        }
        
        let mut offset = 15;
        
        // Extract IV
        let iv_len = package[offset] as usize;
        offset += 1;
        
        if offset + iv_len > package.len() {
            bail!("Invalid IV length");
        }
        
        let iv = package[offset..offset + iv_len].to_vec();
        offset += iv_len;
        
        // Extract salt
        let salt_len = package[offset] as usize;
        offset += 1;
        
        if offset + salt_len > package.len() {
            bail!("Invalid salt length");
        }
        
        let salt = package[offset..offset + salt_len].to_vec();
        offset += salt_len;
        
        // Extract encrypted data
        let encrypted = package[offset..].to_vec();
        
        Ok((encrypted, iv, salt))
    }
    
    /// Extract encryption parameters from metadata
    fn extract_encryption_params(&self, metadata: &Value) -> Result<EncryptionParams> {
        let encryption_enabled = metadata
            .get("encryption_enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(false);
        
        let algorithm = metadata
            .get("encryption_algorithm")
            .and_then(|a| a.as_str())
            .unwrap_or("AES-256");
        
        let key_derivation = metadata
            .get("key_derivation")
            .and_then(|k| k.as_str())
            .unwrap_or("PBKDF2");
        
        Ok(EncryptionParams {
            enabled: encryption_enabled,
            algorithm: algorithm.to_string(),
            key_derivation: key_derivation.to_string(),
        })
    }
    
    /// Generate backup checksum for integrity
    pub async fn generate_checksum(&self, data: &[u8]) -> Result<String> {
        let digest = digest::digest(&digest::SHA256, data);
        Ok(general_purpose::STANDARD.encode(digest.as_ref()))
    }
    
    /// Verify data integrity using checksum
    pub async fn verify_checksum(&self, data: &[u8], expected_checksum: &str) -> Result<bool> {
        let computed_checksum = self.generate_checksum(data).await?;
        Ok(computed_checksum == expected_checksum)
    }
}

/// Encryption parameters extracted from metadata
#[derive(Debug, Clone)]
struct EncryptionParams {
    enabled: bool,
    algorithm: String,
    key_derivation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_encryption_round_trip() {
        let engine = EncryptionEngine::new().await.unwrap();
        let data = b"Hello, World! This is test data for encryption.";
        let settings = EncryptionSettings {
            enabled: true,
            algorithm: "AES-256".to_string(),
            key_derivation: "PBKDF2".to_string(),
            salt: None,
        };
        
        // Encrypt
        let encrypted = engine.encrypt(data, &settings).await.unwrap();
        
        // Create mock metadata for decryption
        let metadata = serde_json::json!({
            "encryption_enabled": true,
            "encryption_algorithm": "AES-256",
            "key_derivation": "PBKDF2"
        });
        
        // Decrypt
        let decrypted = engine.decrypt(&encrypted, &metadata).await.unwrap();
        
        assert_eq!(data, &decrypted[..]);
    }
    
    #[tokio::test]
    async fn test_checksum_generation() {
        let engine = EncryptionEngine::new().await.unwrap();
        let data = b"Test data for checksum verification";
        
        let checksum = engine.generate_checksum(data).await.unwrap();
        assert!(!checksum.is_empty());
        
        let verified = engine.verify_checksum(data, &checksum).await.unwrap();
        assert!(verified);
    }
}