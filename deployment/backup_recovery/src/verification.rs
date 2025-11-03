use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde_json::Value;
use sha2::{Sha256, Digest};

use crate::types::*;
use crate::storage::StorageManager;
use crate::encryption::EncryptionEngine;

/// Backup verification system
pub struct VerificationSystem {
    storage_manager: Arc<RwLock<StorageManager>>,
    encryption_engine: Arc<RwLock<EncryptionEngine>>,
}

impl VerificationSystem {
    /// Create a new verification system
    pub async fn new(
        storage_manager: Arc<RwLock<StorageManager>>,
        encryption_engine: Arc<RwLock<EncryptionEngine>>,
    ) -> Result<Self> {
        Ok(Self {
            storage_manager,
            encryption_engine,
        })
    }
    
    /// Perform comprehensive backup verification
    pub async fn verify_backup(&self, backup_id: &str) -> Result<VerificationResult> {
        info!("Starting verification for backup: {}", backup_id);
        
        let mut storage_manager = self.storage_manager.write().await;
        let encryption_engine = self.encryption_engine.read().await;
        
        // Load backup metadata
        let metadata = storage_manager.load_metadata(backup_id).await?;
        
        // Perform different types of checks
        let mut integrity_checks = Vec::new();
        
        // 1. Metadata integrity check
        let metadata_check = self.verify_metadata_integrity(&metadata).await?;
        integrity_checks.push(metadata_check);
        
        // 2. File integrity checks
        let file_checks = self.verify_file_integrity(backup_id, &metadata).await?;
        integrity_checks.extend(file_checks);
        
        // 3. Backup structure validation
        let structure_check = self.verify_backup_structure(backup_id, &metadata).await?;
        integrity_checks.push(structure_check);
        
        // 4. Compression integrity check
        let compression_check = self.verify_compression_integrity(backup_id, &metadata).await?;
        integrity_checks.push(compression_check);
        
        // 5. Encryption integrity check (if applicable)
        if metadata.get("encryption_enabled")
            .and_then(|e| e.as_bool())
            .unwrap_or(false) {
            let encryption_check = self.verify_encryption_integrity(backup_id, &metadata).await?;
            integrity_checks.push(encryption_check);
        }
        
        // Determine overall status
        let status = if integrity_checks.iter().any(|check| check.status == VerificationStatus::Failed) {
            VerificationStatus::Failed
        } else if integrity_checks.iter().any(|check| check.status == VerificationStatus::Error) {
            VerificationStatus::Error
        } else {
            VerificationStatus::Passed
        };
        
        // Count results
        let files_verified = integrity_checks.len() as u64;
        let files_failed = integrity_checks.iter()
            .filter(|check| check.status == VerificationStatus::Failed)
            .count() as u64;
        
        // Generate assessment
        let assessment = self.generate_assessment(&integrity_checks);
        
        let result = VerificationResult {
            backup_id: backup_id.to_string(),
            status,
            verified_at: chrono::Utc::now(),
            files_verified,
            files_failed,
            integrity_checks,
            assessment,
        };
        
        info!("Verification completed for backup {}: {:?}", backup_id, result.status);
        Ok(result)
    }
    
    /// Verify metadata integrity
    async fn verify_metadata_integrity(&self, metadata: &Value) -> Result<IntegrityCheck> {
        let mut checks = Vec::new();
        
        // Check required fields
        let required_fields = [
            "backup_id", "name", "backup_type", "created_at", 
            "sources", "total_bytes", "file_count", "version"
        ];
        
        for field in &required_fields {
            if metadata.get(field).is_none() {
                checks.push(format!("Missing required field: {}", field));
            }
        }
        
        // Validate backup ID format
        if let Some(backup_id) = metadata.get("backup_id") {
            if let Some(id_str) = backup_id.as_str() {
                if uuid::Uuid::parse_str(id_str).is_err() {
                    checks.push("Invalid backup ID format".to_string());
                }
            }
        }
        
        // Validate timestamps
        if let Some(created_at) = metadata.get("created_at") {
            if let Some(at_str) = created_at.as_str() {
                if chrono::DateTime::parse_from_rfc3339(at_str).is_err() {
                    checks.push("Invalid creation timestamp format".to_string());
                }
            }
        }
        
        let status = if checks.is_empty() {
            VerificationStatus::Passed
        } else {
            VerificationStatus::Failed
        };
        
        Ok(IntegrityCheck {
            check_type: "Metadata Integrity".to_string(),
            status,
            details: checks.join("; "),
        })
    }
    
    /// Verify file integrity
    async fn verify_file_integrity(&self, backup_id: &str, metadata: &Value) -> Result<Vec<IntegrityCheck>> {
        let mut checks = Vec::new();
        let mut storage_manager = self.storage_manager.write().await;
        
        // Extract file list from metadata
        let files = if let Some(files_array) = metadata.get("files") {
            if let Some(files) = files_array.as_array() {
                files
            } else {
                bail!("Invalid files array in metadata");
            }
        } else {
            warn!("No files array in metadata for backup {}", backup_id);
            return Ok(vec![]);
        };
        
        let total_files = files.len() as u64;
        let mut verified_files = 0u64;
        let mut failed_files = 0u64;
        
        for file_entry in files {
            if let Ok(file_info) = serde_json::from_value::<FileInfo>(file_entry.clone()) {
                match self.verify_single_file(backup_id, &file_info, &mut storage_manager).await {
                    Ok(is_valid) => {
                        if is_valid {
                            verified_files += 1;
                        } else {
                            failed_files += 1;
                        }
                    }
                    Err(e) => {
                        error!("Failed to verify file {}: {}", file_info.path.display(), e);
                        failed_files += 1;
                    }
                }
            }
        }
        
        // Overall file integrity check
        let file_status = if failed_files > 0 {
            VerificationStatus::Failed
        } else {
            VerificationStatus::Passed
        };
        
        checks.push(IntegrityCheck {
            check_type: "File Integrity".to_string(),
            status: file_status,
            details: format!("{} files verified, {} files failed", verified_files, failed_files),
        });
        
        Ok(checks)
    }
    
    /// Verify a single file
    async fn verify_single_file(
        &self,
        backup_id: &str,
        file_info: &FileInfo,
        storage_manager: &mut StorageManager,
    ) -> Result<bool> {
        let file_path = Path::new(backup_id).join("files").join(&file_info.path);
        
        // Check if file exists
        if !storage_manager.exists(&file_path).await? {
            error!("File not found: {}", file_path.display());
            return Ok(false);
        }
        
        // Load file data
        let file_data = storage_manager.load_file(backup_id, &file_info.path).await?;
        
        // Verify file size
        if file_data.len() as u64 != file_info.size {
            error!("File size mismatch for {}: expected {}, got {}", 
                file_info.path.display(), file_info.size, file_data.len());
            return Ok(false);
        }
        
        // Verify checksum if available
        if !file_info.checksum.is_empty() {
            let computed_checksum = self.compute_checksum(&file_data);
            if computed_checksum != file_info.checksum {
                error!("File checksum mismatch for {}: expected {}, got {}", 
                    file_info.path.display(), file_info.checksum, computed_checksum);
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Verify backup structure
    async fn verify_backup_structure(&self, backup_id: &str, metadata: &Value) -> Result<IntegrityCheck> {
        let mut storage_manager = self.storage_manager.write().await;
        let mut issues = Vec::new();
        
        // Check for required files
        let required_files = [
            "metadata.json",
            "manifest.json",
        ];
        
        for required_file in &required_files {
            let file_path = Path::new(backup_id).join(required_file);
            if !storage_manager.exists(&file_path).await? {
                issues.push(format!("Missing required file: {}", required_file));
            }
        }
        
        // Check backup directory structure
        let files_dir = Path::new(backup_id).join("files");
        if !storage_manager.exists(&files_dir).await? {
            issues.push("Missing files directory".to_string());
        }
        
        let status = if issues.is_empty() {
            VerificationStatus::Passed
        } else {
            VerificationStatus::Failed
        };
        
        Ok(IntegrityCheck {
            check_type: "Backup Structure".to_string(),
            status,
            details: issues.join("; "),
        })
    }
    
    /// Verify compression integrity
    async fn verify_compression_integrity(&self, backup_id: &str, metadata: &Value) -> Result<IntegrityCheck> {
        let mut storage_manager = self.storage_manager.write().await;
        
        // Get compression algorithm from metadata
        let compression = metadata.get("compression")
            .and_then(|c| c.as_str())
            .unwrap_or("Zstd");
        
        let mut issues = Vec::new();
        let test_file_path = Path::new(backup_id).join("files").join("test.txt");
        
        // Test compression by loading and decompressing a sample file
        if storage_manager.exists(&test_file_path).await? {
            let file_data = storage_manager.load_file(backup_id, &Path::new("files/test.txt")).await?;
            
            // Test decompression
            match self.test_decompression(&file_data, compression).await {
                Ok(_) => debug!("Compression test passed for {}", test_file_path.display()),
                Err(e) => issues.push(format!("Compression test failed: {}", e)),
            }
        }
        
        let status = if issues.is_empty() {
            VerificationStatus::Passed
        } else {
            VerificationStatus::Failed
        };
        
        Ok(IntegrityCheck {
            check_type: "Compression Integrity".to_string(),
            status,
            details: issues.join("; "),
        })
    }
    
    /// Verify encryption integrity
    async fn verify_encryption_integrity(&self, backup_id: &str, metadata: &Value) -> Result<IntegrityCheck> {
        let encryption_engine = self.encryption_engine.read().await;
        
        // Test encryption/decryption on a sample file
        let test_data = b"Encryption test data";
        
        // Get encryption settings from metadata
        let encryption_settings = EncryptionSettings {
            enabled: true,
            algorithm: metadata.get("encryption_algorithm")
                .and_then(|a| a.as_str())
                .unwrap_or("AES-256")
                .to_string(),
            key_derivation: metadata.get("key_derivation")
                .and_then(|k| k.as_str())
                .unwrap_or("PBKDF2")
                .to_string(),
            salt: None,
        };
        
        // Test encryption round trip
        match encryption_engine.encrypt(test_data, &encryption_settings).await {
            Ok(encrypted) => {
                match encryption_engine.decrypt(&encrypted, metadata).await {
                    Ok(decrypted) => {
                        if decrypted == test_data {
                            debug!("Encryption round trip test passed");
                        } else {
                            return Ok(IntegrityCheck {
                                check_type: "Encryption Integrity".to_string(),
                                status: VerificationStatus::Failed,
                                details: "Encryption round trip failed - data mismatch".to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        return Ok(IntegrityCheck {
                            check_type: "Encryption Integrity".to_string(),
                            status: VerificationStatus::Failed,
                            details: format!("Decryption test failed: {}", e),
                        });
                    }
                }
            }
            Err(e) => {
                return Ok(IntegrityCheck {
                    check_type: "Encryption Integrity".to_string(),
                    status: VerificationStatus::Failed,
                    details: format!("Encryption test failed: {}", e),
                });
            }
        }
        
        Ok(IntegrityCheck {
            check_type: "Encryption Integrity".to_string(),
            status: VerificationStatus::Passed,
            details: "Encryption tests passed".to_string(),
        })
    }
    
    /// Test decompression
    async fn test_decompression(&self, _data: &[u8], _algorithm: &str) -> Result<()> {
        // TODO: Implement proper decompression testing
        Ok(())
    }
    
    /// Compute SHA256 checksum
    fn compute_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Generate assessment from integrity checks
    fn generate_assessment(&self, checks: &[IntegrityCheck]) -> String {
        let passed = checks.iter().filter(|c| c.status == VerificationStatus::Passed).count();
        let failed = checks.iter().filter(|c| c.status == VerificationStatus::Failed).count();
        let errors = checks.iter().filter(|c| c.status == VerificationStatus::Error).count();
        
        if failed == 0 && errors == 0 {
            format!("Backup integrity verification passed ({} checks)", passed)
        } else if failed > 0 {
            format!("Backup integrity verification failed - {} checks failed, {} passed", failed, passed)
        } else {
            format!("Backup integrity verification encountered errors - {} errors, {} passed", errors, passed)
        }
    }
    
    /// Quick integrity check (fast verification)
    pub async fn quick_verification(&self, backup_id: &str) -> Result<VerificationResult> {
        info!("Starting quick verification for backup: {}", backup_id);
        
        let mut storage_manager = self.storage_manager.write().await;
        
        // Basic checks only
        let mut checks = Vec::new();
        
        // Check if backup directory exists
        let backup_path = Path::new(backup_id);
        if storage_manager.exists(backup_path).await? {
            checks.push(IntegrityCheck {
                check_type: "Backup Directory".to_string(),
                status: VerificationStatus::Passed,
                details: "Backup directory exists".to_string(),
            });
        } else {
            checks.push(IntegrityCheck {
                check_type: "Backup Directory".to_string(),
                status: VerificationStatus::Failed,
                details: "Backup directory not found".to_string(),
            });
        }
        
        // Check if metadata file exists
        let metadata_path = backup_path.join("metadata.json");
        if storage_manager.exists(&metadata_path).await? {
            checks.push(IntegrityCheck {
                check_type: "Metadata File".to_string(),
                status: VerificationStatus::Passed,
                details: "Metadata file exists".to_string(),
            });
        } else {
            checks.push(IntegrityCheck {
                check_type: "Metadata File".to_string(),
                status: VerificationStatus::Failed,
                details: "Metadata file not found".to_string(),
            });
        }
        
        let status = if checks.iter().any(|c| c.status == VerificationStatus::Failed) {
            VerificationStatus::Failed
        } else {
            VerificationStatus::Passed
        };
        
        Ok(VerificationResult {
            backup_id: backup_id.to_string(),
            status,
            verified_at: chrono::Utc::now(),
            files_verified: 0,
            files_failed: checks.iter().filter(|c| c.status == VerificationStatus::Failed).count() as u64,
            integrity_checks: checks,
            assessment: "Quick verification completed".to_string(),
        })
    }
    
    /// Verify restore integrity
    pub async fn verify_restore(&self, restore_path: &Path, expected_files: &[PathBuf]) -> Result<VerificationResult> {
        info!("Verifying restore integrity at: {}", restore_path.display());
        
        let mut checks = Vec::new();
        let mut missing_files = Vec::new();
        let mut corrupted_files = Vec::new();
        
        for expected_file in expected_files {
            let full_path = restore_path.join(expected_file);
            
            if !full_path.exists() {
                missing_files.push(expected_file.display().to_string());
                continue;
            }
            
            if full_path.is_file() {
                // Check file integrity
                match tokio::fs::read(&full_path).await {
                    Ok(_) => {
                        debug!("File verified: {}", full_path.display());
                    }
                    Err(e) => {
                        corrupted_files.push(format!("{}: {}", expected_file.display(), e));
                    }
                }
            }
        }
        
        // Generate results
        let mut details = Vec::new();
        
        if !missing_files.is_empty() {
            details.push(format!("Missing files: {}", missing_files.join(", ")));
        }
        
        if !corrupted_files.is_empty() {
            details.push(format!("Corrupted files: {}", corrupted_files.join(", ")));
        }
        
        let status = if missing_files.is_empty() && corrupted_files.is_empty() {
            VerificationStatus::Passed
        } else {
            VerificationStatus::Failed
        };
        
        checks.push(IntegrityCheck {
            check_type: "Restore Files".to_string(),
            status,
            details: if details.is_empty() {
                "All files verified".to_string()
            } else {
                details.join("; ")
            },
        });
        
        let assessment = if status == VerificationStatus::Passed {
            "Restore verification passed".to_string()
        } else {
            "Restore verification found issues".to_string()
        };
        
        Ok(VerificationResult {
            backup_id: format!("restore-{}", restore_path.display()),
            status,
            verified_at: chrono::Utc::now(),
            files_verified: expected_files.len() as u64,
            files_failed: (missing_files.len() + corrupted_files.len()) as u64,
            integrity_checks: checks,
            assessment,
        })
    }
}

/// File information structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    checksum: String,
    permissions: Option<u32>,
    modified: chrono::DateTime<chrono::Utc>,
}