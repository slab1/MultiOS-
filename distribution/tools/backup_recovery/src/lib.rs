use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub mod backup;
pub mod restore;
pub mod compression;
pub mod encryption;
pub mod scheduling;
pub mod network;
pub mod verification;
pub mod recovery_media;
pub mod storage;
pub mod types;

use crate::backup::BackupEngine;
use crate::restore::RestoreEngine;
use crate::types::*;

/// Main backup and recovery system
pub struct BackupRecoverySystem {
    config: Arc<RwLock<Config>>,
    backup_engine: Arc<RwLock<BackupEngine>>,
    restore_engine: Arc<RwLock<RestoreEngine>>,
    storage_manager: Arc<RwLock<storage::StorageManager>>,
}

impl BackupRecoverySystem {
    /// Create a new backup recovery system
    pub async fn new(config_path: impl AsRef<Path>) -> Result<Self> {
        let config = Arc::new(RwLock::new(
            Self::load_config(config_path).await?
        ));
        
        let storage_manager = Arc::new(RwLock::new(
            storage::StorageManager::new(config.read().await.default_storage.clone()).await?
        ));
        
        let backup_engine = Arc::new(RwLock::new(
            BackupEngine::new(
                config.clone(),
                storage_manager.clone()
            ).await?
        ));
        
        let restore_engine = Arc::new(RwLock::new(
            RestoreEngine::new(
                config.clone(),
                storage_manager.clone()
            ).await?
        ));
        
        info!("Backup Recovery System initialized");
        Ok(Self {
            config,
            backup_engine,
            restore_engine,
            storage_manager,
        })
    }
    
    /// Load configuration from file
    async fn load_config(config_path: impl AsRef<Path>) -> Result<Config> {
        let path = config_path.as_ref();
        if path.exists() {
            let content = tokio::fs::read_to_string(path)
                .await
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let default_config = Config::default();
            let config_dir = path.parent().unwrap_or(Path::new("."));
            tokio::fs::create_dir_all(config_dir).await?;
            tokio::fs::write(path, toml::to_string_pretty(&default_config)?).await?;
            Ok(default_config)
        }
    }
    
    /// Create a backup job
    pub async fn create_backup(&self, backup_spec: BackupSpecification) -> Result<BackupJob> {
        let mut backup_engine = self.backup_engine.write().await;
        backup_engine.create_backup(backup_spec).await
    }
    
    /// Start a backup job
    pub async fn start_backup(&self, job_id: &str) -> Result<()> {
        let backup_engine = self.backup_engine.read().await;
        backup_engine.start_job(job_id).await
    }
    
    /// List available backup jobs
    pub async fn list_backups(&self) -> Result<Vec<BackupJob>> {
        let storage_manager = self.storage_manager.read().await;
        storage_manager.list_backups().await
    }
    
    /// Restore from a backup
    pub async fn restore_backup(&self, restore_spec: RestoreSpecification) -> Result<RestoreJob> {
        let mut restore_engine = self.restore_engine.write().await;
        restore_engine.create_restore(restore_spec).await
    }
    
    /// Start a restore job
    pub async fn start_restore(&self, job_id: &str) -> Result<()> {
        let restore_engine = self.restore_engine.read().await;
        restore_engine.start_job(job_id).await
    }
    
    /// Verify backup integrity
    pub async fn verify_backup(&self, backup_id: &str) -> Result<VerificationResult> {
        let backup_engine = self.backup_engine.read().await;
        backup_engine.verify_backup(backup_id).await
    }
    
    /// Get system status
    pub async fn get_status(&self) -> Result<SystemStatus> {
        let config = self.config.read().await;
        let backup_engine = self.backup_engine.read().await;
        let restore_engine = self.restore_engine.read().await;
        
        Ok(SystemStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: std::time::SystemTime::now(),
            config_version: config.version,
            active_backups: backup_engine.active_jobs().await,
            active_restores: restore_engine.active_jobs().await,
        })
    }
}