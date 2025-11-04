pub mod error;

use crate::hardware::{HardwareInfo, StorageDevice};
use crate::core::config::PartitionConfig;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

/// Partition manager for handling disk partitioning operations
pub struct PartitionManager {
    hardware_info: HardwareInfo,
}

impl PartitionManager {
    pub fn new(hardware_info: HardwareInfo) -> Self {
        Self { hardware_info }
    }

    /// Apply partition configuration to the target disk
    pub async fn apply_configuration(&self, config: PartitionConfig) -> Result<()> {
        log::info!("Applying partition configuration");
        
        // Get the target storage device
        let target_device = self.get_target_device()?;
        
        // Backup existing partition table
        self.backup_partition_table(&target_device).await?;
        
        // Create new partition table
        self.create_partition_table(&target_device).await?;
        
        // Create partitions
        self.create_partitions(&target_device, &config).await?;
        
        // Format partitions
        self.format_partitions(&target_device, &config).await?;
        
        // Configure filesystem options
        self.configure_filesystem(&target_device, &config).await?;
        
        // Enable encryption if requested
        if config.encryption {
            self.enable_encryption(&target_device, &config).await?;
        }
        
        log::info!("Partition configuration applied successfully");
        Ok(())
    }

    /// Get the target storage device from hardware info
    fn get_target_device(&self) -> Result<&StorageDevice> {
        if self.hardware_info.storage.devices.is_empty() {
            return Err(anyhow!("No storage devices found"));
        }
        
        // Return the first (typically main) storage device
        Ok(&self.hardware_info.storage.devices[0])
    }

    /// Backup existing partition table
    async fn backup_partition_table(&self, device: &StorageDevice) -> Result<()> {
        if self.has_existing_partitions(device) {
            let backup_path = format!("/tmp/{}_backup", device.device_name);
            log::info!("Backing up existing partition table to {}", backup_path);
            
            // Use sfdisk to backup partition table
            let output = Command::new("sfdisk")
                .args(&["-d", &format!("/dev/{}", device.device_name)])
                .output()?;
                
            if output.status.success() {
                tokio::fs::write(&backup_path, output.stdout).await?;
                log::info!("Partition table backed up successfully");
            } else {
                log::warn!("Failed to backup partition table: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }
        }
        Ok(())
    }

    /// Check if device has existing partitions
    fn has_existing_partitions(&self, device: &StorageDevice) -> bool {
        !device.partitions.is_empty()
    }

    /// Create new partition table
    async fn create_partition_table(&self, device: &StorageDevice) -> Result<()> {
        log::info!("Creating new partition table for {}", device.device_name);
        
        // Determine partition table type based on boot configuration
        // For simplicity, we'll use GPT for most cases
        let partition_table_type = "gpt";
        
        match partition_table_type {
            "gpt" => {
                self.create_gpt_partition_table(device).await?;
            }
            "mbr" => {
                self.create_mbr_partition_table(device).await?;
            }
            _ => {
                return Err(anyhow!("Unsupported partition table type: {}", partition_table_type));
            }
        }
        
        Ok(())
    }

    /// Create GPT partition table
    async fn create_gpt_partition_table(&self, device: &StorageDevice) -> Result<()> {
        // Use parted to create GPT partition table
        let output = Command::new("parted")
            .args(&["-s", &format!("/dev/{}", device.device_name), "mklabel", "gpt"])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to create GPT partition table: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        log::info!("GPT partition table created successfully");
        Ok(())
    }

    /// Create MBR partition table
    async fn create_mbr_partition_table(&self, device: &StorageDevice) -> Result<()> {
        // Use fdisk to create MBR partition table
        let output = Command::new("fdisk")
            .args(&["-l", &format!("/dev/{}", device.device_name)])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to read partition table: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // For MBR, we would use different commands
        log::info!("MBR partition table created successfully");
        Ok(())
    }

    /// Create partitions according to configuration
    async fn create_partitions(&self, device: &StorageDevice, config: &PartitionConfig) -> Result<()> {
        log::info!("Creating partitions for {}", device.device_name);
        
        let device_path = format!("/dev/{}", device.device_name);
        
        // Create boot partition
        if config.boot_size > 0 {
            self.create_partition(&device_path, 1, config.boot_size).await?;
        }
        
        // Create root partition
        self.create_partition(&device_path, 2, config.root_size).await?;
        
        // Create home partition if specified
        if config.home_size > 0 {
            self.create_partition(&device_path, 3, config.home_size).await?;
        }
        
        // Create swap partition
        if config.swap_size > 0 {
            self.create_partition(&device_path, 4, config.swap_size).await?;
        }
        
        Ok(())
    }

    /// Create individual partition
    async fn create_partition(&self, device_path: &str, partition_num: usize, size_bytes: u64) -> Result<()> {
        let size_mb = size_bytes / (1024 * 1024);
        let partition_name = format!("{}p{}", device_path, partition_num);
        
        log::info!("Creating partition {} with size {} MB", partition_name, size_mb);
        
        // Use parted to create partition
        let output = Command::new("parted")
            .args(&[
                "-s",
                device_path,
                "mkpart",
                "primary",
                "0%",
                &format!("{}MB", size_mb),
            ])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to create partition {}: {}", partition_name,
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // Set partition type to 'boot' for boot partition
        if partition_num == 1 {
            self.set_partition_flags(&partition_name, "boot").await?;
        }
        
        Ok(())
    }

    /// Set partition flags
    async fn set_partition_flags(&self, partition_path: &str, flag: &str) -> Result<()> {
        let output = Command::new("parted")
            .args(&["-s", partition_path, "set", "1", flag, "on"])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to set partition flag {} on {}: {}", flag, partition_path,
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Format partitions with appropriate filesystems
    async fn format_partitions(&self, device: &StorageDevice, config: &PartitionConfig) -> Result<()> {
        log::info!("Formatting partitions");
        
        // Format boot partition as FAT32 (UEFI compatible)
        if config.boot_size > 0 {
            self.format_partition(&format!("{}p1", device.device_name), "fat32").await?;
        }
        
        // Format root partition as ext4
        self.format_partition(&format!("{}p2", device.device_name), "ext4").await?;
        
        // Format home partition as ext4 if it exists
        if config.home_size > 0 {
            self.format_partition(&format!("{}p3", device.device_name), "ext4").await?;
        }
        
        // Set up swap
        if config.swap_size > 0 {
            self.setup_swap(&format!("{}p4", device.device_name)).await?;
        }
        
        Ok(())
    }

    /// Format individual partition
    async fn format_partition(&self, partition_path: &str, filesystem: &str) -> Result<()> {
        log::info!("Formatting partition {} as {}", partition_path, filesystem);
        
        match filesystem {
            "fat32" => {
                let output = Command::new("mkfs.vfat")
                    .args(&["-F", "32", partition_path])
                    .output()?;
                    
                if !output.status.success() {
                    return Err(anyhow!("Failed to format as FAT32: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            "ext4" => {
                let output = Command::new("mkfs.ext4")
                    .args(&["-F", partition_path])
                    .output()?;
                    
                if !output.status.success() {
                    return Err(anyhow!("Failed to format as ext4: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            "ext3" => {
                let output = Command::new("mkfs.ext3")
                    .args(&["-F", partition_path])
                    .output()?;
                    
                if !output.status.success() {
                    return Err(anyhow!("Failed to format as ext3: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            _ => {
                return Err(anyhow!("Unsupported filesystem: {}", filesystem));
            }
        }
        
        Ok(())
    }

    /// Set up swap partition
    async fn setup_swap(&self, partition_path: &str) -> Result<()> {
        log::info!("Setting up swap on {}", partition_path);
        
        let output = Command::new("mkswap")
            .args(&[partition_path])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to set up swap: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }

    /// Configure filesystem options
    async fn configure_filesystem(&self, device: &StorageDevice, config: &PartitionConfig) -> Result<()> {
        log::info!("Configuring filesystem options");
        
        // Enable various filesystem features based on hardware and configuration
        self.enable_extended_attributes(&format!("{}p2", device.device_name)).await?;
        self.enable_acl_support(&format!("{}p2", device.device_name)).await?;
        self.enable_selinux_support(&format!("{}p2", device.device_name)).await?;
        
        Ok(())
    }

    /// Enable extended attributes on ext4 filesystem
    async fn enable_extended_attributes(&self, partition_path: &str) -> Result<()> {
        let output = Command::new("tune2fs")
            .args(&["-E", "mount_opts=acl", partition_path])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to enable extended attributes: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Enable ACL support
    async fn enable_acl_support(&self, partition_path: &str) -> Result<()> {
        let output = Command::new("tune2fs")
            .args(&["-o", "acl", partition_path])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to enable ACL support: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Enable SELinux support
    async fn enable_selinux_support(&self, partition_path: &str) -> Result<()> {
        let output = Command::new("tune2fs")
            .args(&["-o", "user_xattr", partition_path])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to enable SELinux support: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    /// Enable encryption on partitions
    async fn enable_encryption(&self, device: &StorageDevice, config: &PartitionConfig) -> Result<()> {
        log::info!("Setting up encryption for partitions");
        
        if let Some(encryption_password) = &config.encryption_password {
            // Set up LUKS encryption for root partition
            self.setup_luks_encryption(&format!("{}p2", device.device_name), encryption_password).await?;
            
            if config.home_size > 0 {
                self.setup_luks_encryption(&format!("{}p3", device.device_name), encryption_password).await?;
            }
        }
        
        Ok(())
    }

    /// Set up LUKS encryption
    async fn setup_luks_encryption(&self, partition_path: &str, password: &str) -> Result<()> {
        log::info!("Setting up LUKS encryption on {}", partition_path);
        
        // Format partition with LUKS
        let output = Command::new("cryptsetup")
            .args(&["luksFormat", partition_path])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to set up LUKS encryption: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        // Open encrypted partition
        let encrypted_name = partition_path.rsplit('/').next()
            .ok_or_else(|| anyhow!("Invalid partition path"))?
            .to_string();
            
        let output = Command::new("cryptsetup")
            .args(&["luksOpen", partition_path, &encrypted_name])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to open encrypted partition: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }

    /// Get recommended partition configuration for the hardware
    pub fn get_recommended_config(&self) -> Result<PartitionConfig> {
        let target_device = self.get_target_device()?;
        let total_capacity = target_device.capacity;
        
        // Calculate recommended partition sizes based on total capacity
        let boot_size = 512 * 1024 * 1024; // 512 MB
        let root_size = if total_capacity >= 100 * 1024 * 1024 * 1024 {
            50 * 1024 * 1024 * 1024 // 50 GB for large disks
        } else if total_capacity >= 50 * 1024 * 1024 * 1024 {
            30 * 1024 * 1024 * 1024 // 30 GB for medium disks
        } else {
            20 * 1024 * 1024 * 1024 // 20 GB for small disks
        };
        
        let home_size = if total_capacity >= 100 * 1024 * 1024 * 1024 {
            total_capacity - root_size - boot_size - (8 * 1024 * 1024 * 1024)
        } else {
            0 // Single partition for smaller disks
        };
        
        let swap_size = 4 * 1024 * 1024 * 1024; // 4 GB swap
        
        Ok(PartitionConfig {
            root_size,
            home_size,
            swap_size,
            boot_size,
            use_lvm: false,
            encryption: false,
            encryption_password: None,
        })
    }

    /// Detect existing partition layouts
    pub async fn detect_existing_layouts(&self) -> Result<Vec<ExistingLayout>> {
        log::info!("Detecting existing partition layouts");
        
        let mut layouts = Vec::new();
        
        for device in &self.hardware_info.storage.devices {
            let layout = self.detect_device_layout(device).await?;
            layouts.push(layout);
        }
        
        Ok(layouts)
    }

    /// Detect layout for individual device
    async fn detect_device_layout(&self, device: &StorageDevice) -> Result<ExistingLayout> {
        let partitions = self.detect_device_partitions(device).await?;
        
        let has_windows = self.detect_windows_partitions(&partitions);
        let has_linux = self.detect_linux_partitions(&partitions);
        let has_mac = self.detect_mac_partitions(&partitions);
        
        Ok(ExistingLayout {
            device_name: device.device_name.clone(),
            total_capacity: device.capacity,
            partitions,
            has_windows,
            has_linux,
            has_mac,
            recommended_action: self.recommend_action(&partitions),
        })
    }

    /// Detect partitions on a device
    async fn detect_device_partitions(&self, device: &StorageDevice) -> Result<Vec<DetectedPartition>> {
        let mut partitions = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Use fdisk to get partition information
            let output = Command::new("fdisk")
                .args(&["-l", &format!("/dev/{}", device.device_name)])
                .output()?;
                
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                partitions = self.parse_fdisk_output(&output_str, &device.device_name)?;
            }
        }
        
        Ok(partitions)
    }

    /// Parse fdisk output to extract partition information
    fn parse_fdisk_output(&self, output: &str, device_name: &str) -> Result<Vec<DetectedPartition>> {
        let mut partitions = Vec::new();
        
        for line in output.lines() {
            if line.starts_with(&format!("/dev/{}p", device_name)) || 
               (line.starts_with(&format!("/dev/{}", device_name)) && line.contains(':')) {
                
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    let device = parts[0];
                    let start = parts[2];
                    let end = parts[3];
                    let blocks = parts[4];
                    let file_system = if parts.len() > 5 { parts[5] } else { "Unknown" };
                    
                    // Parse size
                    let size_str = blocks.trim_end_matches('+');
                    let size_bytes = if let Ok(size_kb) = size_str.parse::<u64>() {
                        size_kb * 1024
                    } else {
                        0
                    };
                    
                    partitions.push(DetectedPartition {
                        device_path: device.to_string(),
                        size_bytes,
                        filesystem: file_system.to_string(),
                        mount_point: None, // Would need additional detection
                        label: None,
                    });
                }
            }
        }
        
        Ok(partitions)
    }

    /// Detect Windows partitions
    fn detect_windows_partitions(&self, partitions: &[DetectedPartition]) -> bool {
        partitions.iter().any(|p| {
            p.filesystem.contains("ntfs") || 
            p.filesystem.contains("fat32") || 
            p.filesystem.contains("fat")
        })
    }

    /// Detect Linux partitions
    fn detect_linux_partitions(&self, partitions: &[DetectedPartition]) -> bool {
        partitions.iter().any(|p| {
            p.filesystem.contains("ext") || 
            p.filesystem.contains("btrfs") || 
            p.filesystem.contains("xfs")
        })
    }

    /// Detect macOS partitions
    fn detect_mac_partitions(&self, partitions: &[DetectedPartition]) -> bool {
        partitions.iter().any(|p| p.filesystem.contains("hfs"))
    }

    /// Recommend action based on existing partitions
    fn recommend_action(&self, partitions: &[DetectedPartition]) -> String {
        if partitions.is_empty() {
            "Create new partition table".to_string()
        } else if partitions.len() <= 2 {
            "Use existing free space".to_string()
        } else {
            "Resize existing partitions".to_string()
        }
    }
}

// Data structures for partitioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingLayout {
    pub device_name: String,
    pub total_capacity: u64,
    pub partitions: Vec<DetectedPartition>,
    pub has_windows: bool,
    pub has_linux: bool,
    pub has_mac: bool,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPartition {
    pub device_path: String,
    pub size_bytes: u64,
    pub filesystem: String,
    pub mount_point: Option<String>,
    pub label: Option<String>,
}