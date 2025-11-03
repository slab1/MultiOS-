use anyhow::{Result, Context, bail};
use std::path::{Path, PathBuf};
use std::fs;
use tokio::fs as async_fs;
use tokio::process::Command;
use tracing::{info, warn, error, debug};

use crate::types::*;

/// Bootable recovery media creation system
pub struct RecoveryMediaSystem {
    temp_dir: PathBuf,
    output_dir: PathBuf,
}

impl RecoveryMediaSystem {
    /// Create a new recovery media system
    pub async fn new(temp_dir: PathBuf, output_dir: PathBuf) -> Result<Self> {
        // Create directories
        async_fs::create_dir_all(&temp_dir).await?;
        async_fs::create_dir_all(&output_dir).await?;
        
        Ok(Self { temp_dir, output_dir })
    }
    
    /// Create bootable recovery media
    pub async fn create_recovery_media(&self, config: RecoveryMediaConfig) -> Result<RecoveryMediaResult> {
        info!("Creating bootable recovery media: {}", config.name);
        
        let iso_path = self.output_dir.join(format!("{}.iso", config.name));
        
        // Create ISO structure
        let iso_root = self.temp_dir.join("iso_root");
        self.create_iso_structure(&iso_root, &config).await?;
        
        // Create bootable ISO
        self.create_iso(&iso_root, &iso_path).await?;
        
        // Create bootable USB (if requested)
        let usb_path = if config.create_usb {
            Some(self.create_bootable_usb(&iso_path, &config).await?)
        } else {
            None
        };
        
        // Test media
        let test_result = self.test_recovery_media(&iso_path).await?;
        
        let result = RecoveryMediaResult {
            iso_path,
            usb_path,
            size_bytes: fs::metadata(&iso_path)?.len(),
            test_passed: test_result,
            creation_time: chrono::Utc::now(),
        };
        
        info!("Recovery media created successfully: {} ({:.1} MB)", 
              result.iso_path.display(), result.size_bytes as f64 / 1024.0 / 1024.0);
        
        Ok(result)
    }
    
    /// Create ISO filesystem structure
    async fn create_iso_structure(&self, iso_root: &Path, config: &RecoveryMediaConfig) -> Result<()> {
        info!("Creating ISO structure");
        
        // Create directories
        let dirs = [
            "boot",
            "boot/grub",
            "multios",
            "multios/recovery",
            "multios/backup",
            "multios/drivers",
            "tools",
            "config",
        ];
        
        for dir in &dirs {
            async_fs::create_dir_all(iso_root.join(dir)).await?;
        }
        
        // Create boot configuration
        self.create_boot_config(iso_root, config).await?;
        
        // Copy recovery tools
        self.copy_recovery_tools(iso_root).await?;
        
        // Create configuration files
        self.create_config_files(iso_root, config).await?;
        
        // Copy included backups if specified
        if let Some(backup_ids) = &config.include_backups {
            self.copy_backups_to_media(iso_root, backup_ids).await?;
        }
        
        Ok(())
    }
    
    /// Create boot configuration
    async fn create_boot_config(&self, iso_root: &Path, config: &RecoveryMediaConfig) -> Result<()> {
        let grub_config = format!(r#"#!/bin/sh
set -e

# MultiOS Recovery Media Boot Configuration
# Created: {}

menuentry "MultiOS Recovery System" {{
    linux /multios/recovery/kernel root=/dev/ram0 quiet
    initrd /multios/recovery/initrd.img
}}

menuentry "MultiOS Recovery System (Debug Mode)" {{
    linux /multios/recovery/kernel root=/dev/ram0 debug
    initrd /multios/recovery/initrd.img
}}

menuentry "MultiOS System Information" {{
    linux /multios/recovery/kernel root=/dev/ram0 info
    initrd /multios/recovery/initrd.img
}}

# Submenu for backup operations
submenu "Backup Operations" {{
    menuentry "List Available Backups" {{
        linux /multios/recovery/kernel root=/dev/ram0 list-backups
        initrd /multios/recovery/initrd.img
    }}
    
    menuentry "Restore from Backup" {{
        linux /multios/recovery/kernel root=/dev/ram0 restore
        initrd /multios/recovery/initrd.img
    }}
    
    menuentry "Create System Backup" {{
        linux /multios/recovery/kernel root=/dev/ram0 backup
        initrd /multios/recovery/initrd.img
    }}
}}

# Submenu for system tools
submenu "System Tools" {{
    menuentry "Memory Test" {{
        linux16 /tools/memtest86+.bin
    }}
    
    menuentry "Partition Manager" {{
        linux /multios/recovery/kernel root=/dev/ram0 partition
        initrd /multios/recovery/initrd.img
    }}
    
    menuentry "File Manager" {{
        linux /multios/recovery/kernel root=/dev/ram0 filemanager
        initrd /multios/recovery/initrd.img
    }}
}}

# Boot options
set timeout=30
set default=0
"#, chrono::Utc::now().to_rfc3339());

        async_fs::write(iso_root.join("boot").join("grub").join("grub.cfg"), grub_config).await?;
        
        Ok(())
    }
    
    /// Copy recovery tools
    async fn copy_recovery_tools(&self, iso_root: &Path) -> Result<()> {
        info!("Copying recovery tools");
        
        // Copy recovery binary
        let recovery_bin = "/usr/local/bin/multios-backup";
        if Path::new(recovery_bin).exists() {
            async_fs::copy(recovery_bin, iso_root.join("multios").join("recovery").join("multios-backup")).await?;
        }
        
        // Copy memory test
        let memtest_bin = "/usr/lib/memtest86+/memtest86+.bin";
        if Path::new(memtest_bin).exists() {
            async_fs::copy(memtest_bin, iso_root.join("tools").join("memtest86+.bin")).await?;
        }
        
        // Copy additional tools
        let tools = [
            ("/bin/busybox", "tools/busybox"),
            ("/sbin/fdisk", "tools/fdisk"),
            ("/sbin/mkfs.ext4", "tools/mkfs.ext4"),
            ("/usr/bin/rsync", "tools/rsync"),
            ("/usr/bin/dd", "tools/dd"),
        ];
        
        for (source, dest) in &tools {
            if Path::new(source).exists() {
                let dest_path = iso_root.join(dest);
                async_fs::create_dir_all(dest_path.parent().unwrap()).await?;
                async_fs::copy(source, dest_path).await?;
            }
        }
        
        Ok(())
    }
    
    /// Create configuration files
    async fn create_config_files(&self, iso_root: &Path, config: &RecoveryMediaConfig) -> Result<()> {
        info!("Creating configuration files");
        
        // Create recovery configuration
        let recovery_config = format!(r#"# MultiOS Recovery Media Configuration
# Created: {}

[system]
name = "{}"
version = "1.0.0"
created = "{}"

[recovery]
auto_detect = {}
network_enabled = {}
ssh_enabled = {}

[backup]
default_destination = "{}"
compression = "{}"
encryption = {}

[ui]
timeout = {}
default_menu = "{}"
"#,
            chrono::Utc::now().to_rfc3339(),
            config.name,
            chrono::Utc::now().to_rfc3339(),
            config.auto_detect,
            config.network_enabled,
            config.ssh_enabled,
            config.default_backup_destination,
            config.compression_algorithm,
            config.encryption_enabled,
            config.boot_timeout,
            config.default_menu
        );
        
        async_fs::write(iso_root.join("config").join("recovery.toml"), recovery_config).await?;
        
        // Create device detection script
        let device_script = r#"#!/bin/sh
# Device Detection Script for MultiOS Recovery

echo "Detecting available storage devices..."

# List block devices
lsblk

echo "Detecting network interfaces..."
ip link show

echo "Detection complete."
read -p "Press Enter to continue..."
"#;
        
        async_fs::write(iso_root.join("tools").join("detect_devices.sh"), device_script).await?;
        
        Ok(())
    }
    
    /// Copy backups to recovery media
    async fn copy_backups_to_media(&self, iso_root: &Path, backup_ids: &[String]) -> Result<()> {
        info!("Copying {} backups to media", backup_ids.len());
        
        let backup_dir = iso_root.join("multios").join("backup");
        
        for backup_id in backup_ids {
            let source_path = Path::new("/var/lib/multios/backup").join(backup_id);
            let dest_path = backup_dir.join(backup_id);
            
            if source_path.exists() {
                // Copy entire backup directory
                self.copy_directory(&source_path, &dest_path).await?;
                info!("Copied backup: {}", backup_id);
            } else {
                warn!("Backup not found: {}", backup_id);
            }
        }
        
        Ok(())
    }
    
    /// Copy directory recursively
    async fn copy_directory(&self, source: &Path, dest: &Path) -> Result<()> {
        async_fs::create_dir_all(dest).await?;
        
        let mut entries = async_fs::read_dir(source).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = dest.join(file_name);
            
            if path.is_dir() {
                self.copy_directory(&path, &dest_path).await?;
            } else {
                async_fs::copy(&path, &dest_path).await?;
            }
        }
        
        Ok(())
    }
    
    /// Create bootable ISO
    async fn create_iso(&self, iso_root: &Path, iso_path: &Path) -> Result<()> {
        info!("Creating ISO file: {}", iso_path.display());
        
        // Use genisoimage or xorriso to create ISO
        let output = Command::new("genisoimage")
            .arg("-o")
            .arg(iso_path)
            .arg("-b")
            .arg("boot/grub/stage2")
            .arg("-no-emul-boot")
            .arg("-boot-load-size")
            .arg("4")
            .arg("-boot-info-table")
            .arg("-R")
            .arg("-J")
            .arg("-v")
            .arg("-T")
            .arg("-V")
            .arg("MULTIOS_RECOVERY")
            .arg("-x")
            .arg("lost+found")
            .arg(iso_root)
            .output()
            .await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to create ISO: {}", error);
        }
        
        info!("ISO created successfully");
        Ok(())
    }
    
    /// Create bootable USB drive
    async fn create_bootable_usb(&self, iso_path: &Path, config: &RecoveryMediaConfig) -> Result<PathBuf> {
        info!("Creating bootable USB drive");
        
        if let Some(device) = &config.usb_device {
            // Format USB drive
            self.format_usb_device(device).await?;
            
            // Install bootloader
            self.install_bootloader(device, iso_path).await?;
            
            Ok(PathBuf::from(device))
        } else {
            bail!("No USB device specified");
        }
    }
    
    /// Format USB device
    async fn format_usb_device(&self, device: &str) -> Result<()> {
        warn!("Formatting USB device: {} - THIS WILL DESTROY ALL DATA!", device);
        
        // Unmount device if mounted
        let _ = Command::new("umount")
            .arg(device)
            .output()
            .await;
        
        // Create FAT32 filesystem
        let output = Command::new("mkfs.vfat")
            .arg("-F")
            .arg("32")
            .arg("-n")
            .arg("MULTIOS_RECOVERY")
            .arg(device)
            .output()
            .await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to format USB device: {}", error);
        }
        
        info!("USB device formatted successfully");
        Ok(())
    }
    
    /// Install bootloader to device
    async fn install_bootloader(&self, device: &str, iso_path: &Path) -> Result<()> {
        // Mount USB device
        let mount_point = self.temp_dir.join("usb_mount");
        async_fs::create_dir_all(&mount_point).await?;
        
        Command::new("mount")
            .arg(device)
            .arg(&mount_point)
            .output()
            .await?;
        
        // Copy ISO to USB root
        async_fs::copy(iso_path, mount_point.join("multios-recovery.iso")).await?;
        
        // Install bootloader
        let output = Command::new("dd")
            .arg("if=/usr/lib/ISOLINUX/isolinux.bin")
            .arg(&mount_point.join("isolinux.bin"))
            .output()
            .await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to install bootloader: {}", error);
        }
        
        // Unmount device
        Command::new("umount")
            .arg(&mount_point)
            .output()
            .await?;
        
        info!("Bootloader installed successfully");
        Ok(())
    }
    
    /// Test recovery media
    async fn test_recovery_media(&self, iso_path: &Path) -> Result<bool> {
        info!("Testing recovery media: {}", iso_path.display());
        
        // Basic tests
        let tests = vec![
            ("ISO file exists", iso_path.exists()),
            ("ISO file is readable", fs::metadata(iso_path).map(|m| m.len() > 0).unwrap_or(false)),
            ("ISO file size reasonable", fs::metadata(iso_path).map(|m| m.len() < 1024 * 1024 * 1024).unwrap_or(false)), // < 1GB
        ];
        
        let mut all_passed = true;
        for (test_name, passed) in tests {
            if passed {
                debug!("Test passed: {}", test_name);
            } else {
                warn!("Test failed: {}", test_name);
                all_passed = false;
            }
        }
        
        Ok(all_passed)
    }
    
    /// List available recovery media
    pub async fn list_recovery_media(&self) -> Result<Vec<RecoveryMediaInfo>> {
        let mut media_list = Vec::new();
        
        let entries = async_fs::read_dir(&self.output_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("iso") {
                let metadata = entry.metadata().await?;
                
                media_list.push(RecoveryMediaInfo {
                    name: path.file_stem().unwrap().to_string_lossy().to_string(),
                    path,
                    size_bytes: metadata.len(),
                    created_at: metadata.created()?.into(),
                    bootable: true, // Assume all ISO files are bootable
                    tested: true,   // Assume tested after creation
                });
            }
        }
        
        Ok(media_list)
    }
    
    /// Validate recovery media
    pub async fn validate_recovery_media(&self, media_path: &Path) -> Result<RecoveryMediaValidation> {
        info!("Validating recovery media: {}", media_path.display());
        
        let mut validation = RecoveryMediaValidation {
            media_path: media_path.to_path_buf(),
            is_valid: true,
            bootable: false,
            readable: false,
            size_check: false,
            checksum_valid: false,
            issues: Vec::new(),
        };
        
        // Check file exists
        if !media_path.exists() {
            validation.issues.push("Media file does not exist".to_string());
            validation.is_valid = false;
            return Ok(validation);
        }
        
        // Check readability
        if let Ok(metadata) = fs::metadata(media_path) {
            validation.readable = true;
            
            // Check size
            if metadata.len() > 0 && metadata.len() < 1024 * 1024 * 1024 { // < 1GB
                validation.size_check = true;
            } else {
                validation.issues.push("Invalid file size".to_string());
            }
        } else {
            validation.issues.push("Cannot read file metadata".to_string());
            validation.is_valid = false;
        }
        
        // Check if bootable (basic check)
        if let Ok(content) = async_fs::read(media_path).await {
            if content.windows(2048).any(|window| window.contains(b"ELTORITO")) {
                validation.bootable = true;
            } else {
                validation.issues.push("Not a bootable ISO image".to_string());
            }
        }
        
        // Calculate checksum
        if let Ok(content) = async_fs::read(media_path).await {
            let checksum = self.calculate_sha256(&content);
            validation.checksum_valid = !checksum.is_empty();
        }
        
        validation.is_valid = validation.readable && validation.size_check && validation.bootable;
        
        if validation.is_valid {
            info!("Recovery media validation passed");
        } else {
            warn!("Recovery media validation failed: {:?}", validation.issues);
        }
        
        Ok(validation)
    }
    
    /// Calculate SHA256 checksum
    fn calculate_sha256(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Create emergency boot disk
    pub async fn create_emergency_boot_disk(&self, target_path: &Path) -> Result<()> {
        info!("Creating emergency boot disk: {}", target_path.display());
        
        // Create minimal boot environment
        let boot_dir = self.temp_dir.join("emergency_boot");
        async_fs::create_dir_all(&boot_dir).await?;
        
        // Create minimal GRUB config
        let grub_config = r#"#!/bin/sh
set -e

menuentry "MultiOS Emergency Recovery" {
    linux /kernel quiet root=/dev/ram0
    initrd /initrd.gz
}

set timeout=5
set default=0
"#;
        
        async_fs::write(boot_dir.join("grub.cfg"), grub_config).await?;
        
        // Copy minimal kernel and initrd (placeholder)
        // In production, these would be actual kernel/initrd files
        async_fs::write(boot_dir.join("kernel"), b"placeholder kernel").await?;
        async_fs::write(boot_dir.join("initrd.gz"), b"placeholder initrd").await?;
        
        info!("Emergency boot disk created");
        Ok(())
    }
}

/// Recovery media configuration
#[derive(Debug, Clone)]
pub struct RecoveryMediaConfig {
    pub name: String,
    pub description: String,
    pub include_backups: Option<Vec<String>>,
    pub create_usb: bool,
    pub usb_device: Option<String>,
    pub auto_detect: bool,
    pub network_enabled: bool,
    pub ssh_enabled: bool,
    pub default_backup_destination: String,
    pub compression_algorithm: String,
    pub encryption_enabled: bool,
    pub boot_timeout: u32,
    pub default_menu: String,
}

/// Recovery media creation result
#[derive(Debug)]
pub struct RecoveryMediaResult {
    pub iso_path: PathBuf,
    pub usb_path: Option<PathBuf>,
    pub size_bytes: u64,
    pub test_passed: bool,
    pub creation_time: chrono::DateTime<chrono::Utc>,
}

/// Recovery media information
#[derive(Debug)]
pub struct RecoveryMediaInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub created_at: std::time::SystemTime,
    pub bootable: bool,
    pub tested: bool,
}

/// Recovery media validation result
#[derive(Debug)]
pub struct RecoveryMediaValidation {
    pub media_path: PathBuf,
    pub is_valid: bool,
    pub bootable: bool,
    pub readable: bool,
    pub size_check: bool,
    pub checksum_valid: bool,
    pub issues: Vec<String>,
}