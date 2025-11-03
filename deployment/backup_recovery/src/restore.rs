use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use chrono::Utc;

use crate::types::*;
use crate::compression::CompressionEngine;
use crate::encryption::EncryptionEngine;
use crate::storage::StorageManager;

/// Core restore engine
pub struct RestoreEngine {
    config: Arc<RwLock<Config>>,
    storage_manager: Arc<RwLock<StorageManager>>,
    compression_engine: Arc<RwLock<CompressionEngine>>,
    encryption_engine: Arc<RwLock<EncryptionEngine>>,
    active_jobs: Arc<RwLock<HashMap<String, RestoreJob>>>,
}

impl RestoreEngine {
    /// Create a new restore engine
    pub async fn new(
        config: Arc<RwLock<Config>>,
        storage_manager: Arc<RwLock<StorageManager>>,
    ) -> Result<Self> {
        let compression_engine = Arc::new(RwLock::new(
            CompressionEngine::new().await?
        ));
        
        let encryption_engine = Arc::new(RwLock::new(
            EncryptionEngine::new().await?
        ));
        
        Ok(Self {
            config,
            storage_manager,
            compression_engine,
            encryption_engine,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Create a new restore job
    pub async fn create_restore(&mut self, specification: RestoreSpecification) -> Result<RestoreJob> {
        let job_id = specification.job_id.to_string();
        
        // Validate backup exists
        self.validate_backup(&specification.backup_id).await?;
        
        // Validate target path
        self.validate_target_path(&specification.target_path).await?;
        
        let job = RestoreJob {
            job_id,
            specification,
            status: RestoreJobStatus::Queued,
            created_at: Utc::now(),
            status_changed_at: Utc::now(),
            progress: 0,
            phase: "Queued".to_string(),
            error_message: None,
            files_restored: 0,
            bytes_restored: 0,
        };
        
        let mut active_jobs = self.active_jobs.write().await;
        active_jobs.insert(job.job_id.clone(), job.clone());
        
        info!("Created restore job: {}", job.job_id);
        Ok(job)
    }
    
    /// Start a restore job
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let mut active_jobs = self.active_jobs.write().await;
        
        let Some(job) = active_jobs.get_mut(job_id) else {
            bail!("Restore job not found: {}", job_id);
        };
        
        if job.status != RestoreJobStatus::Queued && job.status != RestoreJobStatus::Paused {
            bail!("Cannot start job in current state: {:?}", job.status);
        }
        
        job.status = RestoreJobStatus::Running;
        job.status_changed_at = Utc::now();
        job.phase = "Starting".to_string();
        
        // Spawn restore task
        let job_id = job.job_id.clone();
        let config = self.config.clone();
        let storage_manager = self.storage_manager.clone();
        let compression_engine = self.compression_engine.clone();
        let encryption_engine = self.encryption_engine.clone();
        let active_jobs_arc = self.active_jobs.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::execute_restore_job(
                &job_id,
                config,
                storage_manager,
                compression_engine,
                encryption_engine,
                active_jobs_arc,
            ).await {
                error!("Restore job {} failed: {}", job_id, e);
                
                let mut jobs = active_jobs_arc.write().await;
                if let Some(failed_job) = jobs.get_mut(&job_id) {
                    failed_job.status = RestoreJobStatus::Failed;
                    failed_job.status_changed_at = Utc::now();
                    failed_job.phase = "Failed".to_string();
                    failed_job.error_message = Some(e.to_string());
                }
            }
        });
        
        info!("Started restore job: {}", job_id);
        Ok(())
    }
    
    /// Execute the restore job
    async fn execute_restore_job(
        job_id: &str,
        config: Arc<RwLock<Config>>,
        storage_manager: Arc<RwLock<StorageManager>>,
        compression_engine: Arc<RwLock<CompressionEngine>>,
        encryption_engine: Arc<RwLock<EncryptionEngine>>,
        active_jobs: Arc<RwLock<HashMap<String, RestoreJob>>>,
    ) -> Result<()> {
        let mut jobs = active_jobs.write().await;
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Job not found during execution");
        };
        
        job.phase = "Loading metadata".to_string();
        job.progress = 5;
        
        // Load backup metadata
        let storage_manager = storage_manager.read().await;
        let metadata = storage_manager.load_metadata(&job.specification.backup_id).await?;
        
        job.phase = "Preparing restore".to_string();
        job.progress = 10;
        
        // Create target directory
        if !job.specification.target_path.exists() {
            tokio::fs::create_dir_all(&job.specification.target_path).await?;
        }
        
        let files_list = Self::extract_file_list(&metadata)?;
        let total_files = files_list.len() as u64;
        let mut processed_files = 0u64;
        let mut total_bytes = 0u64;
        let start_time = std::time::Instant::now();
        
        job.phase = "Restoring files".to_string();
        job.progress = 15;
        
        // Restore files
        for file_info in files_list {
            // Check if file should be included
            if !Self::should_include_file(&file_info.path, &job.specification.include_paths, &job.specification.exclude_paths) {
                processed_files += 1;
                continue;
            }
            
            // Load file from storage
            let mut storage_manager = storage_manager.write().await;
            let file_data = storage_manager.load_file(&job.specification.backup_id, &file_info.path).await?;
            
            total_bytes += file_data.len() as u64;
            
            // Decrypt if needed
            let decrypted_data = {
                let encryption_engine = encryption_engine.read().await;
                encryption_engine.decrypt(&file_data, &metadata).await?
            };
            
            // Decompress if needed
            let final_data = {
                let compression_engine = compression_engine.read().await;
                compression_engine.decompress(&decrypted_data, &metadata).await?
            };
            
            // Write to target location
            let target_path = job.specification.target_path.join(&file_info.path);
            if let Some(parent) = target_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&target_path, &final_data).await?;
            
            // Restore permissions if requested
            if job.specification.restore_permissions && file_info.permissions.is_some() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let permissions = file_info.permissions.unwrap();
                    tokio::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(permissions)).await?;
                }
            }
            
            processed_files += 1;
            
            // Update progress
            if total_files > 0 {
                job.progress = 15 + (processed_files * 80 / total_files) as u8;
            }
            job.files_restored = processed_files;
            job.bytes_restored = total_bytes;
            
            // Calculate rate
            let elapsed = start_time.elapsed().as_secs();
            if elapsed > 0 {
                let _rate = total_bytes / elapsed;
            }
            
            // Allow cancellation check
            if processed_files % 100 == 0 {
                let current_jobs = active_jobs.read().await;
                if let Some(current_job) = current_jobs.get(job_id) {
                    if current_job.status == RestoreJobStatus::Cancelled {
                        return Ok(());
                    }
                }
            }
        }
        
        // Verify restore if requested
        if job.specification.verify_restore {
            job.phase = "Verifying restore".to_string();
            job.progress = 95;
            
            Self::verify_restore(&job.specification.target_path, &job.specification.include_paths).await?;
        }
        
        job.status = RestoreJobStatus::Completed;
        job.status_changed_at = Utc::now();
        job.phase = "Completed".to_string();
        job.progress = 100;
        
        info!("Restore job {} completed: {} files, {} bytes", job_id, processed_files, total_bytes);
        Ok(())
    }
    
    /// Validate that backup exists
    async fn validate_backup(&self, backup_id: &str) -> Result<()> {
        let storage_manager = self.storage_manager.read().await;
        
        if !storage_manager.backup_exists(backup_id).await {
            bail!("Backup not found: {}", backup_id);
        }
        
        Ok(())
    }
    
    /// Validate target path
    async fn validate_target_path(&self, target_path: &Path) -> Result<()> {
        let parent = target_path.parent().ok_or_else(|| anyhow::anyhow!("Invalid target path"))?;
        
        if !parent.exists() {
            bail!("Parent directory does not exist: {:?}", parent);
        }
        
        if !parent.is_dir() {
            bail!("Parent path is not a directory: {:?}", parent);
        }
        
        Ok(())
    }
    
    /// Extract file list from metadata
    fn extract_file_list(metadata: &serde_json::Value) -> Result<Vec<FileInfo>> {
        // This is a simplified implementation
        // In reality, you'd parse the metadata structure to get file list
        let files = metadata.get("files")
            .and_then(|f| f.as_array())
            .cloned()
            .unwrap_or_default();
        
        let mut file_list = Vec::new();
        for file in files {
            if let Ok(file_info) = serde_json::from_value::<FileInfo>(file) {
                file_list.push(file_info);
            }
        }
        
        Ok(file_list)
    }
    
    /// Check if file should be included in restore
    fn should_include_file(
        file_path: &Path,
        include_paths: &[PathBuf],
        exclude_paths: &[PathBuf],
    ) -> bool {
        // Check exclusions first
        for exclude in exclude_paths {
            if file_path.starts_with(exclude) {
                return false;
            }
        }
        
        // If no includes specified, include all files
        if include_paths.is_empty() {
            return true;
        }
        
        // Check inclusions
        for include in include_paths {
            if file_path.starts_with(include) {
                return true;
            }
        }
        
        false
    }
    
    /// Verify restore integrity
    async fn verify_restore(target_path: &Path, include_paths: &[PathBuf]) -> Result<()> {
        // TODO: Implement verification logic
        // This would typically involve checking file checksums, permissions, etc.
        Ok(())
    }
    
    /// Get number of active jobs
    pub async fn active_jobs(&self) -> u32 {
        let jobs = self.active_jobs.read().await;
        jobs.values()
            .filter(|job| matches!(job.status, RestoreJobStatus::Running | RestoreJobStatus::Queued))
            .count() as u32
    }
    
    /// Cancel a restore job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Restore job not found: {}", job_id);
        };
        
        job.status = RestoreJobStatus::Cancelled;
        job.status_changed_at = Utc::now();
        job.phase = "Cancelled".to_string();
        
        info!("Cancelled restore job: {}", job_id);
        Ok(())
    }
    
    /// Pause a restore job
    pub async fn pause_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Restore job not found: {}", job_id);
        };
        
        if job.status == RestoreJobStatus::Running {
            job.status = RestoreJobStatus::Paused;
            job.status_changed_at = Utc::now();
            job.phase = "Paused".to_string();
            
            info!("Paused restore job: {}", job_id);
        }
        
        Ok(())
    }
    
    /// Resume a restore job
    pub async fn resume_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Restore job not found: {}", job_id);
        };
        
        if job.status == RestoreJobStatus::Paused {
            job.status = RestoreJobStatus::Running;
            job.status_changed_at = Utc::now();
            job.phase = "Resuming".to_string();
            
            info!("Resumed restore job: {}", job_id);
        }
        
        Ok(())
    }
    
    /// Quick restore for common issues
    pub async fn quick_restore(&self, options: QuickRestoreOptions) -> Result<RestoreJob> {
        let job_id = Uuid::new_v4();
        
        // Determine files to restore based on restore type
        let include_paths = match options.restore_type {
            QuickRestoreType::SystemFiles => {
                vec![
                    PathBuf::from("/etc"),
                    PathBuf::from("/bin"),
                    PathBuf::from("/usr/bin"),
                ]
            }
            QuickRestoreType::Drivers => {
                vec![
                    PathBuf::from("/lib/modules"),
                    PathBuf::from("/usr/lib"),
                ]
            }
            QuickRestoreType::UserDocuments => {
                vec![
                    PathBuf::from("/home"),
                ]
            }
            QuickRestoreType::ApplicationData => {
                vec![
                    PathBuf::from("/var/lib"),
                ]
            }
            QuickRestoreType::ConfigurationFiles => {
                vec![
                    PathBuf::from("/etc"),
                ]
            }
        };
        
        // Create restore specification
        let specification = RestoreSpecification {
            job_id,
            backup_id: "latest".to_string(), // Would need to be determined
            target_path: options.target_path,
            include_paths,
            exclude_paths: vec![],
            point_in_time: None,
            verify_restore: options.verify_after,
            restore_permissions: true,
            restore_ownership: true,
        };
        
        self.create_restore(specification).await
    }
}

/// File information from backup metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FileInfo {
    path: PathBuf,
    size: u64,
    checksum: String,
    permissions: Option<u32>,
    modified: chrono::DateTime<chrono::Utc>,
}

use std::collections::HashMap;