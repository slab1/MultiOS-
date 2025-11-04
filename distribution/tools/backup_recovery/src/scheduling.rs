use anyhow::{Result, Context, bail};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};

use crate::types::*;
use crate::BackupRecoverySystem;

/// Backup scheduling system
pub struct SchedulingSystem {
    config: Arc<RwLock<Config>>,
    active_schedules: Arc<RwLock<HashMap<String, ScheduleJob>>>,
    backup_system: Arc<RwLock<BackupRecoverySystem>>,
}

impl SchedulingSystem {
    /// Create a new scheduling system
    pub async fn new(
        config: Arc<RwLock<Config>>,
        backup_system: Arc<RwLock<BackupRecoverySystem>>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            active_schedules: Arc::new(RwLock::new(HashMap::new())),
            backup_system,
        })
    }
    
    /// Start the scheduling system
    pub async fn start(&self) -> Result<()> {
        info!("Starting backup scheduling system");
        
        // Load and start existing schedules
        let config = self.config.read().await;
        for schedule_config in &config.schedules {
            if schedule_config.enabled {
                self.start_schedule(schedule_config).await?;
            }
        }
        
        // Start the scheduler loop
        let active_schedules = self.active_schedules.clone();
        let backup_system = self.backup_system.clone();
        let config_clone = self.config.clone();
        
        tokio::spawn(async move {
            Self::scheduler_loop(active_schedules, backup_system, config_clone).await;
        });
        
        info!("Backup scheduling system started");
        Ok(())
    }
    
    /// Add a new schedule
    pub async fn add_schedule(&self, schedule: ScheduleConfig) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Validate cron expression
        Self::validate_cron_expression(&schedule.cron_expression)?;
        
        // Add to configuration
        config.schedules.push(schedule.clone());
        
        // Start the schedule if enabled
        if schedule.enabled {
            self.start_schedule(&schedule).await?;
        }
        
        // Save configuration
        self.save_config(&config).await?;
        
        info!("Added schedule: {}", schedule.name);
        Ok(())
    }
    
    /// Remove a schedule
    pub async fn remove_schedule(&self, schedule_name: &str) -> Result<()> {
        let mut config = self.config.write().await;
        
        // Find and remove the schedule
        let original_len = config.schedules.len();
        config.schedules.retain(|s| s.name != schedule_name);
        
        if config.schedules.len() == original_len {
            bail!("Schedule not found: {}", schedule_name);
        }
        
        // Stop the schedule if running
        self.stop_schedule(schedule_name).await?;
        
        // Save configuration
        self.save_config(&config).await?;
        
        info!("Removed schedule: {}", schedule_name);
        Ok(())
    }
    
    /// Enable a schedule
    pub async fn enable_schedule(&self, schedule_name: &str) -> Result<()> {
        let mut config = self.config.write().await;
        
        if let Some(schedule) = config.schedules.iter_mut().find(|s| s.name == schedule_name) {
            schedule.enabled = true;
            self.save_config(&config).await?;
            
            // Start the schedule
            self.start_schedule(schedule).await?;
            
            info!("Enabled schedule: {}", schedule_name);
        } else {
            bail!("Schedule not found: {}", schedule_name);
        }
        
        Ok(())
    }
    
    /// Disable a schedule
    pub async fn disable_schedule(&self, schedule_name: &str) -> Result<()> {
        let mut config = self.config.write().await;
        
        if let Some(schedule) = config.schedules.iter_mut().find(|s| s.name == schedule_name) {
            schedule.enabled = false;
            self.save_config(&config).await?;
            
            // Stop the schedule
            self.stop_schedule(schedule_name).await?;
            
            info!("Disabled schedule: {}", schedule_name);
        } else {
            bail!("Schedule not found: {}", schedule_name);
        }
        
        Ok(())
    }
    
    /// List all schedules
    pub async fn list_schedules(&self) -> Result<Vec<ScheduleConfig>> {
        let config = self.config.read().await;
        Ok(config.schedules.clone())
    }
    
    /// Start a specific schedule
    async fn start_schedule(&self, schedule: &ScheduleConfig) -> Result<()> {
        let active_schedules = self.active_schedules.clone();
        let schedule_name = schedule.name.clone();
        
        // Create schedule job
        let job = ScheduleJob {
            name: schedule.name.clone(),
            cron_expression: schedule.cron_expression.clone(),
            backup_spec_id: schedule.backup_spec_id.clone(),
            next_run: self.calculate_next_run(&schedule.cron_expression)?,
            last_run: None,
            is_running: false,
        };
        
        // Add to active schedules
        let mut active_schedules = active_schedules.write().await;
        active_schedules.insert(schedule_name, job);
        
        info!("Started schedule: {}", schedule.name);
        Ok(())
    }
    
    /// Stop a specific schedule
    async fn stop_schedule(&self, schedule_name: &str) -> Result<()> {
        let mut active_schedules = self.active_schedules.write().await;
        active_schedules.remove(schedule_name);
        
        info!("Stopped schedule: {}", schedule_name);
        Ok(())
    }
    
    /// Main scheduler loop
    async fn scheduler_loop(
        active_schedules: Arc<RwLock<HashMap<String, ScheduleJob>>>,
        backup_system: Arc<RwLock<BackupRecoverySystem>>,
        config: Arc<RwLock<Config>>,
    ) {
        let mut interval = interval(Duration::from_secs(60)); // Check every minute
        
        loop {
            interval.tick().await;
            
            let now = Utc::now();
            let active_schedules = active_schedules.read().await;
            
            for (name, job) in active_schedules.iter() {
                if job.next_run <= now && !job.is_running {
                    // Trigger backup
                    drop(active_schedules); // Release lock
                    
                    let name = name.clone();
                    let job = job.clone();
                    let backup_system = backup_system.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::execute_scheduled_backup(&name, &job, &backup_system).await {
                            error!("Scheduled backup {} failed: {}", name, e);
                        }
                    });
                    
                    break; // Avoid holding lock during spawn
                }
            }
        }
    }
    
    /// Execute a scheduled backup
    async fn execute_scheduled_backup(
        schedule_name: &str,
        job: &ScheduleJob,
        backup_system: &Arc<RwLock<BackupRecoverySystem>>,
    ) -> Result<()> {
        debug!("Executing scheduled backup: {}", schedule_name);
        
        // Mark as running
        {
            let mut backup_system = backup_system.write().await;
            let backup_engine = backup_system.backup_engine.read().await;
            // Update job status would go here
        }
        
        // Create backup specification (simplified)
        let backup_spec = BackupSpecification {
            job_id: Uuid::new_v4(),
            name: format!("Scheduled backup: {}", schedule_name),
            backup_type: BackupType::Incremental,
            sources: vec![PathBuf::from("/home"), PathBuf::from("/etc")],
            destination: StorageLocation {
                id: "local-default".to_string(),
                storage_type: StorageType::Local,
                path: "/var/lib/multios/backup".to_string(),
                config: HashMap::new(),
                is_default: true,
            },
            compression: CompressionAlgorithm::Zstd,
            encryption: EncryptionSettings {
                enabled: false,
                algorithm: "AES-256".to_string(),
                key_derivation: "PBKDF2".to_string(),
                salt: None,
            },
            description: Some(format!("Auto-generated from schedule: {}", schedule_name)),
            tags: HashMap::from([
                ("scheduled".to_string(), "true".to_string()),
                ("schedule_name".to_string(), schedule_name.to_string()),
            ]),
            verify_integrity: true,
            create_recovery_media: false,
        };
        
        // Create and start backup
        let mut backup_system = backup_system.write().await;
        let backup_job = backup_system.create_backup(backup_spec).await?;
        backup_system.start_backup(&backup_job.job_id).await?;
        
        info!("Started scheduled backup: {} (job: {})", schedule_name, backup_job.job_id);
        
        // Update schedule next run time
        let next_run = Self::calculate_next_run(&job.cron_expression)?;
        
        let mut backup_system = backup_system.write().await;
        let mut config = backup_system.config.write().await;
        
        if let Some(schedule) = config.schedules.iter_mut().find(|s| s.name == schedule_name) {
            schedule.last_run = Some(Utc::now());
            schedule.next_run = Some(next_run);
            
            backup_system.save_config(&config).await?;
        }
        
        Ok(())
    }
    
    /// Calculate next run time from cron expression
    fn calculate_next_run(cron_expression: &str) -> Result<DateTime<Utc>> {
        // Simplified cron parsing - in production, use a proper cron parser
        // This is a basic implementation for demonstration
        
        let now = Utc::now();
        
        // Parse basic cron: "minute hour day month day_of_week"
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        
        if parts.len() != 5 {
            bail!("Invalid cron expression: {}", cron_expression);
        }
        
        let (minute, hour, day, month, dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
        
        // Simple implementation - check each minute for next hour
        for minute_offset in 0..60 {
            let candidate = now + chrono::Duration::minutes(minute_offset);
            
            // Check if candidate matches cron expression
            if Self::matches_cron_part(candidate.minute(), minute) &&
               Self::matches_cron_part(candidate.hour(), hour) &&
               Self::matches_cron_part(candidate.day(), day) &&
               Self::matches_cron_part(candidate.month(), month) &&
               Self::matches_cron_part(candidate.weekday().number_from_monday(), dow) {
                return Ok(candidate);
            }
        }
        
        // Default to next hour if no match found
        Ok(now + chrono::Duration::hours(1))
    }
    
    /// Check if a value matches a cron part
    fn matches_cron_part(value: u32, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if let Ok(num) = pattern.parse::<u32>() {
            return value == num;
        }
        
        // Handle ranges like "1-5"
        if let Some((start, end)) = pattern.split_once('-') {
            if let (Ok(start), Ok(end)) = (start.parse::<u32>(), end.parse::<u32>()) {
                return value >= start && value <= end;
            }
        }
        
        // Handle lists like "1,3,5"
        let values: Vec<u32> = pattern.split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        values.contains(&value)
    }
    
    /// Validate cron expression
    fn validate_cron_expression(cron_expression: &str) -> Result<()> {
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        
        if parts.len() != 5 {
            bail!("Cron expression must have 5 parts: minute hour day month day_of_week");
        }
        
        for (i, part) in parts.iter().enumerate() {
            if !Self::is_valid_cron_part(part) {
                bail!("Invalid cron part at position {}: {}", i, part);
            }
        }
        
        Ok(())
    }
    
    /// Check if a cron part is valid
    fn is_valid_cron_part(part: &str) -> bool {
        if part == "*" {
            return true;
        }
        
        // Number
        if part.parse::<u32>().is_ok() {
            return true;
        }
        
        // Range
        if part.contains('-') {
            if let Some((start, end)) = part.split_once('-') {
                return start.parse::<u32>().is_ok() && end.parse::<u32>().is_ok();
            }
        }
        
        // List
        if part.contains(',') {
            return part.split(',').all(|s| s.parse::<u32>().is_ok() || s == "*");
        }
        
        false
    }
    
    /// Save configuration to file
    async fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = "/etc/multios/backup/config.toml";
        let config_str = toml::to_string_pretty(config)?;
        
        tokio::fs::write(config_path, config_str).await?;
        
        Ok(())
    }
    
    /// Apply retention policies
    pub async fn apply_retention_policies(&self) -> Result<()> {
        info!("Applying backup retention policies");
        
        let mut backup_system = self.backup_system.write().await;
        let mut storage_manager = backup_system.storage_manager.write().await;
        
        // Get list of all backups
        let backups = storage_manager.list_backups().await?;
        
        // Group backups by retention policy
        let mut policy_backups: HashMap<String, Vec<BackupJob>> = HashMap::new();
        
        for backup in backups {
            let policy_name = backup.specification.tags
                .get("retention_policy")
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            
            policy_backups.entry(policy_name).or_insert_with(Vec::new).push(backup);
        }
        
        // Apply retention for each policy
        let config = self.config.read().await;
        for (policy_name, policy_backups) in policy_backups {
            if let Some(policy) = config.retention_policies.iter().find(|p| p.name == policy_name) {
                self.apply_retention_to_policy(&policy, &policy_backups).await?;
            }
        }
        
        info!("Retention policies applied");
        Ok(())
    }
    
    /// Apply retention policy to backups
    async fn apply_retention_to_policy(
        &self,
        policy: &RetentionPolicy,
        backups: &[BackupJob],
    ) -> Result<()> {
        let mut backup_system = self.backup_system.write().await;
        let mut storage_manager = backup_system.storage_manager.write().await;
        
        let mut sorted_backups = backups.to_vec();
        sorted_backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let to_delete = self.determine_backups_to_delete(policy, &sorted_backups)?;
        
        for backup in to_delete {
            storage_manager.delete_backup(&backup.job_id).await?;
            info!("Deleted backup due to retention policy: {}", backup.job_id);
        }
        
        Ok(())
    }
    
    /// Determine which backups to delete based on retention policy
    fn determine_backups_to_delete(
        &self,
        policy: &RetentionPolicy,
        backups: &[BackupJob],
    ) -> Result<Vec<BackupJob>> {
        let now = Utc::now();
        let mut to_delete = Vec::new();
        
        // Group backups by type and time periods
        let mut daily = Vec::new();
        let mut weekly = Vec::new();
        let mut monthly = Vec::new();
        let mut yearly = Vec::new();
        
        for backup in backups {
            let age_days = (now - backup.created_at).num_days() as u32;
            
            if age_days < policy.keep_daily {
                daily.push(backup);
            } else if age_days < policy.keep_weekly * 7 {
                weekly.push(backup);
            } else if age_days < policy.keep_monthly * 30 {
                monthly.push(backup);
            } else {
                yearly.push(backup);
            }
        }
        
        // Delete old backups keeping only the specified retention counts
        let mut all_to_delete = Vec::new();
        
        // Keep only last N backups regardless of age if specified
        if let Some(keep_last_n) = policy.keep_last_n {
            if backups.len() > keep_last_n as usize {
                all_to_delete.extend_from_slice(&backups[keep_last_n as usize..]);
            }
        }
        
        // Additional logic for specific retention periods would go here
        
        Ok(all_to_delete)
    }
}

/// Schedule job information
#[derive(Debug, Clone)]
struct ScheduleJob {
    name: String,
    cron_expression: String,
    backup_spec_id: String,
    next_run: DateTime<Utc>,
    last_run: Option<DateTime<Utc>>,
    is_running: bool,
}