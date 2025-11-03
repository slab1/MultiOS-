use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn, error, debug};
use serde_json::Value;

use crate::types::*;

/// Storage manager for handling different storage backends
pub struct StorageManager {
    default_location: StorageLocation,
    active_storages: HashMap<String, Box<dyn StorageBackend + Send + Sync>>,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(default_location: StorageLocation) -> Result<Self> {
        let mut manager = Self {
            default_location: default_location.clone(),
            active_storages: HashMap::new(),
        };
        
        // Initialize default storage
        manager.initialize_storage(&default_location).await?;
        
        info!("Storage manager initialized with default location: {}", default_location.path);
        Ok(manager)
    }
    
    /// Initialize a storage backend
    async fn initialize_storage(&mut self, location: &StorageLocation) -> Result<()> {
        let backend: Box<dyn StorageBackend + Send + Sync> = match location.storage_type {
            StorageType::Local => Box::new(LocalStorage::new(location).await?),
            StorageType::Network => Box::new(NetworkStorage::new(location).await?),
            StorageType::AmazonS3 => Box::new(S3Storage::new(location).await?),
            StorageType::GoogleCloud => Box::new(GCSStorage::new(location).await?),
            StorageType::AzureBlob => Box::new(AzureStorage::new(location).await?),
            StorageType::Ftp => Box::new(FTPStorage::new(location).await?),
            StorageType::Sftp => Box::new(SFTPStorage::new(location).await?),
        };
        
        self.active_storages.insert(location.id.clone(), backend);
        
        info!("Initialized storage backend: {} ({:?})", location.id, location.storage_type);
        Ok(())
    }
    
    /// Store a file in backup storage
    pub async fn store_file(&mut self, backup_id: &str, relative_path: &Path, data: &[u8]) -> Result<()> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        // Create backup directory structure
        let backup_path = Path::new(backup_id).join("files").join(relative_path);
        storage.store(&backup_path, data).await?;
        
        debug!("Stored file: {}", backup_path.display());
        Ok(())
    }
    
    /// Load a file from backup storage
    pub async fn load_file(&mut self, backup_id: &str, relative_path: &Path) -> Result<Vec<u8>> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        let backup_path = Path::new(backup_id).join("files").join(relative_path);
        let data = storage.load(&backup_path).await?;
        
        debug!("Loaded file: {}", backup_path.display());
        Ok(data)
    }
    
    /// Store backup metadata
    pub async fn store_metadata(&mut self, backup_id: &str, metadata: &Value) -> Result<()> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        let metadata_path = Path::new(backup_id).join("metadata.json");
        let metadata_str = serde_json::to_string_pretty(metadata)?;
        
        storage.store(&metadata_path, metadata_str.as_bytes()).await?;
        
        debug!("Stored metadata: {}", metadata_path.display());
        Ok(())
    }
    
    /// Load backup metadata
    pub async fn load_metadata(&mut self, backup_id: &str) -> Result<Value> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        let metadata_path = Path::new(backup_id).join("metadata.json");
        let data = storage.load(&metadata_path).await?;
        let metadata_str = String::from_utf8(data)?;
        
        let metadata: Value = serde_json::from_str(&metadata_str)?;
        
        debug!("Loaded metadata: {}", metadata_path.display());
        Ok(metadata)
    }
    
    /// Check if backup exists
    pub async fn backup_exists(&mut self, backup_id: &str) -> bool {
        let storage = match self.get_storage(&self.default_location.id) {
            Ok(s) => s,
            Err(_) => return false,
        };
        
        let metadata_path = Path::new(backup_id).join("metadata.json");
        storage.exists(&metadata_path).await.unwrap_or(false)
    }
    
    /// List available backups
    pub async fn list_backups(&mut self) -> Result<Vec<BackupJob>> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        let mut backups = Vec::new();
        let backup_dirs = storage.list_directories().await?;
        
        for backup_dir in backup_dirs {
            if let Ok(metadata) = self.load_metadata(&backup_dir.to_string_lossy()).await {
                // Parse metadata to create BackupJob
                // This is simplified - in practice you'd have a more robust parser
                if let Some(job_id) = metadata.get("backup_id") {
                    if let Some(job_id_str) = job_id.as_str() {
                        let backup_job = BackupJob {
                            job_id: job_id_str.to_string(),
                            specification: BackupSpecification {
                                job_id: uuid::Uuid::parse_str(job_id_str).unwrap_or(uuid::Uuid::new_v4()),
                                name: metadata.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown").to_string(),
                                backup_type: BackupType::Full,
                                sources: vec![],
                                destination: self.default_location.clone(),
                                compression: CompressionAlgorithm::Zstd,
                                encryption: EncryptionSettings {
                                    enabled: false,
                                    algorithm: "AES-256".to_string(),
                                    key_derivation: "PBKDF2".to_string(),
                                    salt: None,
                                },
                                description: None,
                                tags: HashMap::new(),
                                verify_integrity: false,
                                create_recovery_media: false,
                            },
                            status: BackupJobStatus::Completed,
                            created_at: chrono::Utc::now(),
                            status_changed_at: chrono::Utc::now(),
                            progress: 100,
                            phase: "Completed".to_string(),
                            error_message: None,
                            size_bytes: metadata.get("total_bytes").and_then(|b| b.as_u64()).unwrap_or(0),
                            files_processed: metadata.get("file_count").and_then(|c| c.as_u64()).unwrap_or(0),
                            rate_bytes_per_sec: 0,
                        };
                        backups.push(backup_job);
                    }
                }
            }
        }
        
        Ok(backups)
    }
    
    /// Get storage backend by ID
    fn get_storage(&self, storage_id: &str) -> Result<&Box<dyn StorageBackend + Send + Sync>> {
        self.active_storages.get(storage_id)
            .ok_or_else(|| anyhow::anyhow!("Storage backend not found: {}", storage_id))
    }
    
    /// Delete a backup
    pub async fn delete_backup(&mut self, backup_id: &str) -> Result<()> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        let backup_path = Path::new(backup_id);
        storage.delete(backup_path).await?;
        
        info!("Deleted backup: {}", backup_id);
        Ok(())
    }
    
    /// Get backup size
    pub async fn get_backup_size(&mut self, backup_id: &str) -> Result<u64> {
        let storage = self.get_storage(&self.default_location.id)?;
        
        let backup_path = Path::new(backup_id);
        let size = storage.get_size(backup_path).await?;
        
        Ok(size)
    }
}

/// Storage backend trait
#[async_trait::async_trait]
pub trait StorageBackend {
    /// Store data at path
    async fn store(&self, path: &Path, data: &[u8]) -> Result<()>;
    
    /// Load data from path
    async fn load(&self, path: &Path) -> Result<Vec<u8>>;
    
    /// Check if path exists
    async fn exists(&self, path: &Path) -> Result<bool>;
    
    /// Delete path
    async fn delete(&self, path: &Path) -> Result<()>;
    
    /// Get size of path
    async fn get_size(&self, path: &Path) -> Result<u64>;
    
    /// List directories
    async fn list_directories(&self) -> Result<Vec<PathBuf>>;
}

/// Local filesystem storage
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub async fn new(location: &StorageLocation) -> Result<Self> {
        let base_path = PathBuf::from(&location.path);
        
        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_path).await?;
        
        Ok(Self { base_path })
    }
    
    fn full_path(&self, path: &Path) -> PathBuf {
        self.base_path.join(path)
    }
}

#[async_trait::async_trait]
impl StorageBackend for LocalStorage {
    async fn store(&self, path: &Path, data: &[u8]) -> Result<()> {
        let full_path = self.full_path(path);
        
        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let mut file = fs::File::create(&full_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;
        
        Ok(())
    }
    
    async fn load(&self, path: &Path) -> Result<Vec<u8>> {
        let full_path = self.full_path(path);
        let data = fs::read(&full_path).await?;
        Ok(data)
    }
    
    async fn exists(&self, path: &Path) -> Result<bool> {
        let full_path = self.full_path(path);
        Ok(full_path.exists())
    }
    
    async fn delete(&self, path: &Path) -> Result<()> {
        let full_path = self.full_path(path);
        
        if full_path.is_dir() {
            fs::remove_dir_all(&full_path).await?;
        } else if full_path.is_file() {
            fs::remove_file(&full_path).await?;
        }
        
        Ok(())
    }
    
    async fn get_size(&self, path: &Path) -> Result<u64> {
        let full_path = self.full_path(path);
        
        if full_path.is_dir() {
            self.calculate_dir_size(&full_path).await
        } else {
            let metadata = fs::metadata(&full_path).await?;
            Ok(metadata.len())
        }
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            }
        }
        
        Ok(dirs)
    }
    
    async fn calculate_dir_size(&self, dir: &Path) -> Result<u64> {
        let mut total_size = 0u64;
        let mut entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if metadata.is_dir() {
                total_size += self.calculate_dir_size(&path).await?;
            } else {
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }
}

/// Network attached storage (simplified)
pub struct NetworkStorage {
    // Network storage implementation
}

impl NetworkStorage {
    pub async fn new(_location: &StorageLocation) -> Result<Self> {
        // TODO: Implement network storage
        warn!("Network storage not yet implemented");
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl StorageBackend for NetworkStorage {
    async fn store(&self, _path: &Path, _data: &[u8]) -> Result<()> {
        bail!("Network storage not implemented")
    }
    
    async fn load(&self, _path: &Path) -> Result<Vec<u8>> {
        bail!("Network storage not implemented")
    }
    
    async fn exists(&self, _path: &Path) -> Result<bool> {
        bail!("Network storage not implemented")
    }
    
    async fn delete(&self, _path: &Path) -> Result<()> {
        bail!("Network storage not implemented")
    }
    
    async fn get_size(&self, _path: &Path) -> Result<u64> {
        bail!("Network storage not implemented")
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        bail!("Network storage not implemented")
    }
}

/// Amazon S3 storage (simplified)
pub struct S3Storage {
    // S3 storage implementation
}

impl S3Storage {
    pub async fn new(_location: &StorageLocation) -> Result<Self> {
        // TODO: Implement S3 storage
        warn!("S3 storage not yet implemented");
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl StorageBackend for S3Storage {
    async fn store(&self, _path: &Path, _data: &[u8]) -> Result<()> {
        bail!("S3 storage not implemented")
    }
    
    async fn load(&self, _path: &Path) -> Result<Vec<u8>> {
        bail!("S3 storage not implemented")
    }
    
    async fn exists(&self, _path: &Path) -> Result<bool> {
        bail!("S3 storage not implemented")
    }
    
    async fn delete(&self, _path: &Path) -> Result<()> {
        bail!("S3 storage not implemented")
    }
    
    async fn get_size(&self, _path: &Path) -> Result<u64> {
        bail!("S3 storage not implemented")
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        bail!("S3 storage not implemented")
    }
}

/// Google Cloud Storage (simplified)
pub struct GCSStorage {
    // GCS storage implementation
}

impl GCSStorage {
    pub async fn new(_location: &StorageLocation) -> Result<Self> {
        // TODO: Implement GCS storage
        warn!("GCS storage not yet implemented");
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl StorageBackend for GCSStorage {
    async fn store(&self, _path: &Path, _data: &[u8]) -> Result<()> {
        bail!("GCS storage not implemented")
    }
    
    async fn load(&self, _path: &Path) -> Result<Vec<u8>> {
        bail!("GCS storage not implemented")
    }
    
    async fn exists(&self, _path: &Path) -> Result<bool> {
        bail!("GCS storage not implemented")
    }
    
    async fn delete(&self, _path: &Path) -> Result<()> {
        bail!("GCS storage not implemented")
    }
    
    async fn get_size(&self, _path: &Path) -> Result<u64> {
        bail!("GCS storage not implemented")
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        bail!("GCS storage not implemented")
    }
}

/// Azure Blob Storage (simplified)
pub struct AzureStorage {
    // Azure storage implementation
}

impl AzureStorage {
    pub async fn new(_location: &StorageLocation) -> Result<Self> {
        // TODO: Implement Azure storage
        warn!("Azure storage not yet implemented");
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl StorageBackend for AzureStorage {
    async fn store(&self, _path: &Path, _data: &[u8]) -> Result<()> {
        bail!("Azure storage not implemented")
    }
    
    async fn load(&self, _path: &Path) -> Result<Vec<u8>> {
        bail!("Azure storage not implemented")
    }
    
    async fn exists(&self, _path: &Path) -> Result<bool> {
        bail!("Azure storage not implemented")
    }
    
    async fn delete(&self, _path: &Path) -> Result<()> {
        bail!("Azure storage not implemented")
    }
    
    async fn get_size(&self, _path: &Path) -> Result<u64> {
        bail!("Azure storage not implemented")
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        bail!("Azure storage not implemented")
    }
}

/// FTP storage (simplified)
pub struct FTPStorage {
    // FTP storage implementation
}

impl FTPStorage {
    pub async fn new(_location: &StorageLocation) -> Result<Self> {
        // TODO: Implement FTP storage
        warn!("FTP storage not yet implemented");
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl StorageBackend for FTPStorage {
    async fn store(&self, _path: &Path, _data: &[u8]) -> Result<()> {
        bail!("FTP storage not implemented")
    }
    
    async fn load(&self, _path: &Path) -> Result<Vec<u8>> {
        bail!("FTP storage not implemented")
    }
    
    async fn exists(&self, _path: &Path) -> Result<bool> {
        bail!("FTP storage not implemented")
    }
    
    async fn delete(&self, _path: &Path) -> Result<()> {
        bail!("FTP storage not implemented")
    }
    
    async fn get_size(&self, _path: &Path) -> Result<u64> {
        bail!("FTP storage not implemented")
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        bail!("FTP storage not implemented")
    }
}

/// SFTP storage (simplified)
pub struct SFTPStorage {
    // SFTP storage implementation
}

impl SFTPStorage {
    pub async fn new(_location: &StorageLocation) -> Result<Self> {
        // TODO: Implement SFTP storage
        warn!("SFTP storage not yet implemented");
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl StorageBackend for SFTPStorage {
    async fn store(&self, _path: &Path, _data: &[u8]) -> Result<()> {
        bail!("SFTP storage not implemented")
    }
    
    async fn load(&self, _path: &Path) -> Result<Vec<u8>> {
        bail!("SFTP storage not implemented")
    }
    
    async fn exists(&self, _path: &Path) -> Result<bool> {
        bail!("SFTP storage not implemented")
    }
    
    async fn delete(&self, _path: &Path) -> Result<()> {
        bail!("SFTP storage not implemented")
    }
    
    async fn get_size(&self, _path: &Path) -> Result<u64> {
        bail!("SFTP storage not implemented")
    }
    
    async fn list_directories(&self) -> Result<Vec<PathBuf>> {
        bail!("SFTP storage not implemented")
    }
}