pub mod error;

use crate::core::config::RecoveryConfig;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs;

/// Recovery manager for handling installation rollback and recovery
pub struct RecoveryManager {
    recovery_points: Vec<RecoveryPoint>,
    max_points: usize,
    recovery_dir: PathBuf,
}

impl RecoveryManager {
    pub fn new() -> Self {
        let recovery_dir = PathBuf::from("/var/lib/multios/recovery");
        
        Self {
            recovery_points: Vec::new(),
            max_points: 10, // Default maximum
            recovery_dir,
        }
    }

    /// Create a new recovery point
    pub async fn create_point(&mut self, recovery_point: RecoveryPoint) -> Result<()> {
        log::info!("Creating recovery point: {}", recovery_point.name);
        
        // Create recovery directory if it doesn't exist
        fs::create_dir_all(&self.recovery_dir).await?;
        
        // Create point-specific directory
        let point_dir = self.recovery_dir.join(&recovery_point.name);
        fs::create_dir_all(&point_dir).await?;
        
        // Save recovery point metadata
        let metadata_file = point_dir.join("metadata.json");
        let metadata = serde_json::to_string_pretty(&recovery_point)?;
        fs::write(&metadata_file, metadata).await?;
        
        // Create backup of critical system files
        self.backup_system_files(&point_dir).await?;
        
        // Create backup of partition table
        self.backup_partition_table(&point_dir).await?;
        
        // Create backup of bootloader configuration
        self.backup_bootloader_config(&point_dir).await?;
        
        // Store recovery point
        self.recovery_points.push(recovery_point.clone());
        
        // Clean up old recovery points if necessary
        self.cleanup_old_points().await?;
        
        log::info!("Recovery point '{}' created successfully", recovery_point.name);
        Ok(())
    }

    /// List all available recovery points
    pub async fn list_points(&self) -> Result<Vec<RecoveryPoint>> {
        log::info!("Listing recovery points");
        
        let mut points = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.recovery_dir).await {
            for entry in entries {
                if let Ok(entry) = entry {
                    let point_dir = entry.path();
                    let metadata_file = point_dir.join("metadata.json");
                    
                    if metadata_file.exists() {
                        if let Ok(metadata_str) = fs::read_to_string(&metadata_file).await {
                            if let Ok(point) = serde_json::from_str::<RecoveryPoint>(&metadata_str) {
                                points.push(point);
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by creation time (newest first)
        points.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(points)
    }

    /// Restore from a recovery point
    pub async fn restore_point(&self, point_name: &str) -> Result<()> {
        log::info!("Restoring from recovery point: {}", point_name);
        
        let point_dir = self.recovery_dir.join(point_name);
        if !point_dir.exists() {
            return Err(anyhow!("Recovery point '{}' not found", point_name));
        }
        
        // Restore system files
        self.restore_system_files(&point_dir).await?;
        
        // Restore partition table
        self.restore_partition_table(&point_dir).await?;
        
        // Restore bootloader configuration
        self.restore_bootloader_config(&point_dir).await?;
        
        log::info!("Recovery point '{}' restored successfully", point_name);
        Ok(())
    }

    /// Remove a recovery point
    pub async fn remove_point(&mut self, point_name: &str) -> Result<()> {
        log::info!("Removing recovery point: {}", point_name);
        
        let point_dir = self.recovery_dir.join(point_name);
        if point_dir.exists() {
            fs::remove_dir_all(&point_dir).await?;
        }
        
        // Remove from memory
        self.recovery_points.retain(|p| p.name != point_name);
        
        log::info!("Recovery point '{}' removed", point_name);
        Ok(())
    }

    /// Clean up old recovery points
    async fn cleanup_old_points(&mut self) -> Result<()> {
        if self.recovery_points.len() > self.max_points {
            // Sort by creation time (oldest first)
            self.recovery_points.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            
            // Remove oldest points
            while self.recovery_points.len() > self.max_points {
                if let Some(oldest_point) = self.recovery_points.first() {
                    let point_dir = self.recovery_dir.join(&oldest_point.name);
                    if point_dir.exists() {
                        fs::remove_dir_all(&point_dir).await?;
                    }
                    self.recovery_points.remove(0);
                }
            }
        }
        
        Ok(())
    }

    /// Backup critical system files
    async fn backup_system_files(&self, point_dir: &Path) -> Result<()> {
        log::info!("Backing up system files");
        
        let backup_dir = point_dir.join("system_files");
        fs::create_dir_all(&backup_dir).await?;
        
        let critical_files = vec![
            ("/etc/fstab", "fstab"),
            ("/etc/passwd", "passwd"),
            ("/etc/group", "group"),
            ("/etc/shadow", "shadow"),
            ("/etc/hostname", "hostname"),
            ("/etc/hosts", "hosts"),
            ("/etc/lightdm/lightdm.conf", "lightdm.conf"),
        ];
        
        for (source, dest) in critical_files {
            let source_path = Path::new(source);
            if source_path.exists() {
                let dest_path = backup_dir.join(dest);
                fs::copy(source_path, dest_path).await?;
            }
        }
        
        Ok(())
    }

    /// Restore system files from backup
    async fn restore_system_files(&self, point_dir: &Path) -> Result<()> {
        log::info!("Restoring system files");
        
        let backup_dir = point_dir.join("system_files");
        if !backup_dir.exists() {
            log::warn!("System files backup not found in recovery point");
            return Ok(());
        }
        
        let critical_files = vec![
            ("/etc/fstab", "fstab"),
            ("/etc/passwd", "passwd"),
            ("/etc/group", "group"),
            ("/etc/shadow", "shadow"),
            ("/etc/hostname", "hostname"),
            ("/etc/hosts", "hosts"),
            ("/etc/lightdm/lightdm.conf", "lightdm.conf"),
        ];
        
        for (dest, source) in critical_files {
            let backup_path = backup_dir.join(source);
            let dest_path = Path::new(dest);
            
            if backup_path.exists() {
                // Create backup of current file first
                if dest_path.exists() {
                    let backup_current = format!("{}.backup", dest);
                    fs::copy(dest_path, Path::new(&backup_current)).await?;
                }
                
                // Restore from recovery point
                fs::copy(&backup_path, dest_path).await?;
            }
        }
        
        Ok(())
    }

    /// Backup partition table
    async fn backup_partition_table(&self, point_dir: &Path) -> Result<()> {
        log::info!("Backing up partition table");
        
        let backup_file = point_dir.join("partition_table");
        
        // Use sfdisk to backup partition table for all detected disks
        let disks = ["sda", "sdb", "sdc"]; // Common disk names
        
        for disk in &disks {
            let device_path = format!("/dev/{}", disk);
            if Path::new(&device_path).exists() {
                let output = tokio::process::Command::new("sfdisk")
                    .args(&["-d", &device_path])
                    .output()
                    .await?;
                    
                if output.status.success() {
                    let backup_content = String::from_utf8_lossy(&output.stdout);
                    let disk_backup_file = format!("{}_{}", backup_file.display(), disk);
                    fs::write(&disk_backup_file, backup_content).await?;
                    break; // Only backup the first disk found
                }
            }
        }
        
        Ok(())
    }

    /// Restore partition table from backup
    async fn restore_partition_table(&self, point_dir: &Path) -> Result<()> {
        log::info!("Restoring partition table");
        
        let backup_file = point_dir.join("partition_table");
        
        // Look for any partition table backup file
        let disk_backups = vec!["sda", "sdb", "sdc"];
        
        for disk in &disk_backups {
            let backup_file_path = format!("{}_{}", backup_file.display(), disk);
            if Path::new(&backup_file_path).exists() {
                let backup_content = fs::read_to_string(&backup_file_path).await?;
                let device_path = format!("/dev/{}", disk);
                
                // WARNING: This is a dangerous operation
                log::warn!("Restoring partition table to {}", device_path);
                
                let output = tokio::process::Command::new("sfdisk")
                    .args(&["--force", &device_path])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()?;
                    
                // Write the partition table backup to stdin
                let mut stdin = output.stdin.take().unwrap();
                stdin.write_all(backup_content.as_bytes()).await?;
                
                let result = output.wait_with_output().await?;
                
                if !result.status.success() {
                    log::warn!("Failed to restore partition table: {}", 
                        String::from_utf8_lossy(&result.stderr));
                }
                
                break; // Only restore for the first disk found
            }
        }
        
        Ok(())
    }

    /// Backup bootloader configuration
    async fn backup_bootloader_config(&self, point_dir: &Path) -> Result<()> {
        log::info!("Backing up bootloader configuration");
        
        let backup_dir = point_dir.join("bootloader");
        fs::create_dir_all(&backup_dir).await?;
        
        // Backup GRUB configuration
        let grub_configs = vec![
            "/boot/grub/grub.cfg",
            "/etc/default/grub",
        ];
        
        for config in &grub_configs {
            let config_path = Path::new(config);
            if config_path.exists() {
                let dest_file = backup_dir.join(config_path.file_name().unwrap());
                fs::copy(config_path, dest_file).await?;
            }
        }
        
        // Backup UEFI boot entries
        let efi_backup_dir = backup_dir.join("efi");
        if Path::new("/boot/efi").exists() {
            fs::create_dir_all(&efi_backup_dir).await?;
            self.copy_dir_recursive(Path::new("/boot/efi"), &efi_backup_dir).await?;
        }
        
        Ok(())
    }

    /// Restore bootloader configuration
    async fn restore_bootloader_config(&self, point_dir: &Path) -> Result<()> {
        log::info!("Restoring bootloader configuration");
        
        let backup_dir = point_dir.join("bootloader");
        if !backup_dir.exists() {
            log::warn!("Bootloader backup not found in recovery point");
            return Ok(());
        }
        
        // Restore GRUB configuration
        let grub_configs = vec![
            "/boot/grub/grub.cfg",
            "/etc/default/grub",
        ];
        
        for config in &grub_configs {
            let backup_file = backup_dir.join(Path::new(config).file_name().unwrap());
            if backup_file.exists() {
                let config_path = Path::new(config);
                
                // Create backup of current config first
                if config_path.exists() {
                    let backup_current = format!("{}.recovery_backup", config);
                    fs::copy(config_path, Path::new(&backup_current)).await?;
                }
                
                // Restore from recovery point
                fs::copy(&backup_file, config_path).await?;
            }
        }
        
        // Restore UEFI boot entries
        let efi_backup_dir = backup_dir.join("efi");
        if efi_backup_dir.exists() && Path::new("/boot/efi").exists() {
            self.copy_dir_recursive(&efi_backup_dir, Path::new("/boot/efi")).await?;
        }
        
        Ok(())
    }

    /// Copy directory recursively
    async fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        if src.is_dir() {
            fs::create_dir_all(dst).await?;
            
            let mut entries = fs::read_dir(src).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name()
                    .ok_or_else(|| anyhow!("Invalid file name"))?;
                let dst_path = dst.join(entry_name);
                
                if entry_path.is_dir() {
                    self.copy_dir_recursive(&entry_path, &dst_path).await?;
                } else {
                    fs::copy(&entry_path, &dst_path).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Get recovery statistics
    pub async fn get_statistics(&self) -> Result<RecoveryStatistics> {
        let points = self.list_points().await?;
        let total_points = points.len();
        let total_size = self.calculate_total_size().await?;
        
        let oldest_point = points.last().cloned();
        let newest_point = points.first().cloned();
        
        Ok(RecoveryStatistics {
            total_points,
            total_size_bytes: total_size,
            oldest_point,
            newest_point,
            max_points: self.max_points,
        })
    }

    /// Calculate total size of all recovery points
    async fn calculate_total_size(&self) -> Result<u64> {
        let mut total_size = 0;
        
        if let Ok(entries) = fs::read_dir(&self.recovery_dir).await {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        total_size += self.calculate_dir_size(&path).await?;
                    }
                }
            }
        }
        
        Ok(total_size)
    }

    /// Calculate size of a directory recursively
    async fn calculate_dir_size(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0;
        
        let mut entries = fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let metadata = fs::metadata(&entry_path).await?;
            
            if entry_path.is_dir() {
                total_size += self.calculate_dir_size(&entry_path).await?;
            } else {
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }

    /// Validate recovery point integrity
    pub async fn validate_point(&self, point_name: &str) -> Result<RecoveryValidationResult> {
        let point_dir = self.recovery_dir.join(point_name);
        
        if !point_dir.exists() {
            return Ok(RecoveryValidationResult {
                valid: false,
                errors: vec!["Recovery point directory not found".to_string()],
                warnings: Vec::new(),
            });
        }
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check metadata file
        let metadata_file = point_dir.join("metadata.json");
        if !metadata_file.exists() {
            errors.push("Metadata file missing".to_string());
        } else {
            if let Ok(metadata_str) = fs::read_to_string(&metadata_file).await {
                if let Err(e) = serde_json::from_str::<RecoveryPoint>(&metadata_str) {
                    errors.push(format!("Invalid metadata: {}", e));
                }
            }
        }
        
        // Check backup files
        let backup_checks = vec![
            ("system_files", "System files backup"),
            ("partition_table", "Partition table backup"),
            ("bootloader", "Bootloader backup"),
        ];
        
        for (backup_dir, description) in backup_checks {
            let backup_path = point_dir.join(backup_dir);
            if !backup_path.exists() {
                warnings.push(format!("{} directory missing", description));
            }
        }
        
        Ok(RecoveryValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
}

/// Recovery point information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub name: String,
    pub description: String,
    pub created_at: SystemTime,
    pub created_by: String,
    pub point_type: RecoveryPointType,
    pub size_bytes: u64,
    pub files_backed_up: Vec<String>,
    pub installation_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryPointType {
    /// Initial system state
    Initial,
    /// Before installation steps
    PreInstallation,
    /// Before partitioning
    PrePartitioning,
    /// Before file system changes
    PreFileSystem,
    /// Before bootloader changes
    PreBootLoader,
    /// Final backup
    Final,
    /// Manual recovery point
    Manual,
}

impl RecoveryPoint {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: "Installation recovery point".to_string(),
            created_at: SystemTime::now(),
            created_by: "multios-installer".to_string(),
            point_type: RecoveryPointType::Manual,
            size_bytes: 0,
            files_backed_up: Vec::new(),
            installation_step: None,
        }
    }
}

/// Recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatistics {
    pub total_points: usize,
    pub total_size_bytes: u64,
    pub oldest_point: Option<RecoveryPoint>,
    pub newest_point: Option<RecoveryPoint>,
    pub max_points: usize,
}

/// Recovery validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}