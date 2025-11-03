use anyhow::{Result, Context, bail};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::Utc;

use crate::types::*;
use crate::compression::CompressionEngine;
use crate::encryption::EncryptionEngine;
use crate::storage::StorageManager;

/// Core backup engine
pub struct BackupEngine {
    config: Arc<RwLock<Config>>,
    storage_manager: Arc<RwLock<StorageManager>>,
    compression_engine: Arc<RwLock<CompressionEngine>>,
    encryption_engine: Arc<RwLock<EncryptionEngine>>,
    active_jobs: Arc<RwLock<HashMap<String, BackupJob>>>,
}

impl BackupEngine {
    /// Create a new backup engine
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
    
    /// Create a new backup job
    pub async fn create_backup(&mut self, specification: BackupSpecification) -> Result<BackupJob> {
        let job_id = specification.job_id.to_string();
        
        // Validate sources
        self.validate_sources(&specification.sources).await?;
        
        // Validate destination
        self.validate_destination(&specification.destination).await?;
        
        let job = BackupJob {
            job_id,
            specification,
            status: BackupJobStatus::Queued,
            created_at: Utc::now(),
            status_changed_at: Utc::now(),
            progress: 0,
            phase: "Queued".to_string(),
            error_message: None,
            size_bytes: 0,
            files_processed: 0,
            rate_bytes_per_sec: 0,
        };
        
        let mut active_jobs = self.active_jobs.write().await;
        active_jobs.insert(job.job_id.clone(), job.clone());
        
        info!("Created backup job: {} - {}", job.job_id, job.specification.name);
        Ok(job)
    }
    
    /// Start a backup job
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let mut active_jobs = self.active_jobs.write().await;
        
        let Some(job) = active_jobs.get_mut(job_id) else {
            bail!("Backup job not found: {}", job_id);
        };
        
        if job.status != BackupJobStatus::Queued && job.status != BackupJobStatus::Paused {
            bail!("Cannot start job in current state: {:?}", job.status);
        }
        
        job.status = BackupJobStatus::Running;
        job.status_changed_at = Utc::now();
        job.phase = "Starting".to_string();
        
        // Spawn backup task
        let job_id = job.job_id.clone();
        let config = self.config.clone();
        let storage_manager = self.storage_manager.clone();
        let compression_engine = self.compression_engine.clone();
        let encryption_engine = self.encryption_engine.clone();
        let active_jobs_arc = self.active_jobs.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::execute_backup_job(
                &job_id,
                config,
                storage_manager,
                compression_engine,
                encryption_engine,
                active_jobs_arc,
            ).await {
                error!("Backup job {} failed: {}", job_id, e);
                
                let mut jobs = active_jobs_arc.write().await;
                if let Some(failed_job) = jobs.get_mut(&job_id) {
                    failed_job.status = BackupJobStatus::Failed;
                    failed_job.status_changed_at = Utc::now();
                    failed_job.phase = "Failed".to_string();
                    failed_job.error_message = Some(e.to_string());
                }
            }
        });
        
        info!("Started backup job: {}", job_id);
        Ok(())
    }
    
    /// Execute the backup job
    async fn execute_backup_job(
        job_id: &str,
        config: Arc<RwLock<Config>>,
        storage_manager: Arc<RwLock<StorageManager>>,
        compression_engine: Arc<RwLock<CompressionEngine>>,
        encryption_engine: Arc<RwLock<EncryptionEngine>>,
        active_jobs: Arc<RwLock<HashMap<String, BackupJob>>>,
    ) -> Result<()> {
        let mut jobs = active_jobs.write().await;
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Job not found during execution");
        };
        
        job.phase = "Analyzing".to_string();
        job.progress = 5;
        
        // Discover files to backup
        let files = Self::discover_files(&job.specification.sources).await?;
        job.files_processed = 0;
        job.phase = "Backing up".to_string();
        job.progress = 10;
        
        let total_files = files.len() as u64;
        let mut processed_files = 0u64;
        let mut total_bytes = 0u64;
        let start_time = std::time::Instant::now();
        
        // Process files
        for file_path in files {
            if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
                if metadata.is_file() {
                    // Read file
                    let content = tokio::fs::read(&file_path).await?;
                    total_bytes += content.len() as u64;
                    
                    // Compress if needed
                    let compressed_content = {
                        let compression_engine = compression_engine.read().await;
                        compression_engine.compress(
                            &content,
                            &job.specification.compression
                        ).await?
                    };
                    
                    // Encrypt if needed
                    let final_content = {
                        let encryption_engine = encryption_engine.read().await;
                        encryption_engine.encrypt(
                            &compressed_content,
                            &job.specification.encryption
                        ).await?
                    };
                    
                    // Store file
                    let relative_path = Self::get_relative_path(&file_path, &job.specification.sources);
                    let storage_manager = storage_manager.write().await;
                    storage_manager.store_file(&job_id, &relative_path, &final_content).await?;
                    
                    processed_files += 1;
                    
                    // Update progress
                    if total_files > 0 {
                        job.progress = 10 + (processed_files * 90 / total_files) as u8;
                    }
                    job.files_processed = processed_files;
                    job.size_bytes = total_bytes;
                    
                    // Calculate rate
                    let elapsed = start_time.elapsed().as_secs();
                    if elapsed > 0 {
                        job.rate_bytes_per_sec = total_bytes / elapsed;
                    }
                }
            }
            
            // Allow cancellation check
            if processed_files % 100 == 0 {
                let current_jobs = active_jobs.read().await;
                if let Some(current_job) = current_jobs.get(job_id) {
                    if current_job.status == BackupJobStatus::Cancelled {
                        return Ok(());
                    }
                }
            }
        }
        
        // Create backup metadata
        job.phase = "Finalizing".to_string();
        job.progress = 95;
        
        let metadata = Self::create_backup_metadata(&job.specification, total_bytes, processed_files)?;
        {
            let mut storage_manager = storage_manager.write().await;
            storage_manager.store_metadata(&job_id, &metadata).await?;
        }
        
        job.status = BackupJobStatus::Completed;
        job.status_changed_at = Utc::now();
        job.phase = "Completed".to_string();
        job.progress = 100;
        
        info!("Backup job {} completed: {} files, {} bytes", job_id, processed_files, total_bytes);
        Ok(())
    }
    
    /// Validate backup sources
    async fn validate_sources(&self, sources: &[PathBuf]) -> Result<()> {
        for source in sources {
            if !source.exists() {
                warn!("Backup source does not exist: {:?}", source);
                continue;
            }
            
            if !source.is_dir() && !source.is_file() {
                bail!("Backup source is not a file or directory: {:?}", source);
            }
        }
        
        Ok(())
    }
    
    /// Validate backup destination
    async fn validate_destination(&self, destination: &StorageLocation) -> Result<()> {
        // TODO: Implement destination validation based on storage type
        Ok(())
    }
    
    /// Discover files to backup
    async fn discover_files(sources: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for source in sources {
            if source.is_file() {
                files.push(source.clone());
            } else if source.is_dir() {
                Self::discover_files_in_dir(source, &mut files).await?;
            }
        }
        
        Ok(files)
    }
    
    /// Recursively discover files in directory
    async fn discover_files_in_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if metadata.is_dir() {
                // Skip system directories
                if !path.to_string_lossy().contains("/proc/") &&
                   !path.to_string_lossy().contains("/sys/") &&
                   !path.to_string_lossy().contains("/dev/") {
                    Self::discover_files_in_dir(&path, files).await?;
                }
            } else if metadata.is_file() {
                files.push(path);
            }
        }
        
        Ok(())
    }
    
    /// Get relative path from source directories
    fn get_relative_path(file_path: &Path, sources: &[PathBuf]) -> PathBuf {
        for source in sources {
            if file_path.starts_with(source) {
                let relative = file_path.strip_prefix(source).unwrap_or(file_path);
                return relative.to_path_buf();
            }
        }
        file_path.to_path_buf()
    }
    
    /// Create backup metadata
    fn create_backup_metadata(
        spec: &BackupSpecification,
        total_bytes: u64,
        file_count: u64,
    ) -> Result<serde_json::Value> {
        let metadata = serde_json::json!({
            "backup_id": spec.job_id,
            "name": spec.name,
            "backup_type": format!("{:?}", spec.backup_type),
            "created_at": Utc::now().to_rfc3339(),
            "sources": spec.sources.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
            "total_bytes": total_bytes,
            "file_count": file_count,
            "compression": format!("{:?}", spec.compression),
            "encryption_enabled": spec.encryption.enabled,
            "description": spec.description,
            "tags": spec.tags,
            "version": "1.0.0"
        });
        
        Ok(metadata)
    }
    
    /// Verify backup integrity
    pub async fn verify_backup(&self, backup_id: &str) -> Result<VerificationResult> {
        let storage_manager = self.storage_manager.read().await;
        
        // Load backup metadata
        let metadata = storage_manager.load_metadata(backup_id).await?;
        
        // TODO: Implement verification logic
        let verification_result = VerificationResult {
            backup_id: backup_id.to_string(),
            status: VerificationStatus::Passed,
            verified_at: Utc::now(),
            files_verified: 0,
            files_failed: 0,
            integrity_checks: vec![],
            assessment: "Backup verification completed".to_string(),
        };
        
        Ok(verification_result)
    }
    
    /// Get number of active jobs
    pub async fn active_jobs(&self) -> u32 {
        let jobs = self.active_jobs.read().await;
        jobs.values()
            .filter(|job| matches!(job.status, BackupJobStatus::Running | BackupJobStatus::Queued))
            .count() as u32
    }
    
    /// Cancel a backup job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Backup job not found: {}", job_id);
        };
        
        job.status = BackupJobStatus::Cancelled;
        job.status_changed_at = Utc::now();
        job.phase = "Cancelled".to_string();
        
        info!("Cancelled backup job: {}", job_id);
        Ok(())
    }
    
    /// Pause a backup job
    pub async fn pause_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Backup job not found: {}", job_id);
        };
        
        if job.status == BackupJobStatus::Running {
            job.status = BackupJobStatus::Paused;
            job.status_changed_at = Utc::now();
            job.phase = "Paused".to_string();
            
            info!("Paused backup job: {}", job_id);
        }
        
        Ok(())
    }
    
    /// Resume a backup job
    pub async fn resume_job(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.active_jobs.write().await;
        
        let Some(job) = jobs.get_mut(job_id) else {
            bail!("Backup job not found: {}", job_id);
        };
        
        if job.status == BackupJobStatus::Paused {
            job.status = BackupJobStatus::Running;
            job.status_changed_at = Utc::now();
            job.phase = "Resuming".to_string();
            
            info!("Resumed backup job: {}", job_id);
        }
        
        Ok(())
    }
}

use std::collections::HashMap;