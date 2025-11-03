use super::config::InstallationConfig;
use super::state::WizardState;
use super::progress::ProgressTracker;
use crate::hardware::HardwareInfo;

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

pub struct InstallationManager {
    config: InstallationConfig,
}

impl InstallationManager {
    pub fn new(config: InstallationConfig) -> Self {
        Self { config }
    }

    /// Copy system files to the target location
    pub async fn copy_system_files(&self, hardware_info: &HardwareInfo) -> Result<()> {
        log::info!("Starting system files copy");
        
        // Create target directories
        self.create_target_directories().await?;
        
        // Copy kernel and initramfs
        self.copy_kernel_files().await?;
        
        // Copy system libraries and binaries
        self.copy_system_libraries().await?;
        
        // Copy device drivers
        self.copy_device_drivers(hardware_info).await?;
        
        // Copy configuration files
        self.copy_configuration_files().await?;
        
        // Create necessary system files
        self.create_system_files().await?;
        
        log::info!("System files copy completed successfully");
        Ok(())
    }

    /// Configure bootloader for the installed system
    pub async fn configure_bootloader(&self, hardware_info: &HardwareInfo) -> Result<()> {
        log::info!("Configuring bootloader");
        
        // Determine boot type
        let boot_type = &self.config.boot_type;
        
        match boot_type {
            super::config::BootType::Legacy => {
                self.configure_grub_legacy(hardware_info).await?;
            }
            super::config::BootType::Uefi => {
                self.configure_grub_uefi(hardware_info).await?;
            }
            super::config::BootType::Hybrid => {
                self.configure_grub_hybrid(hardware_info).await?;
            }
            super::config::BootType::Auto => {
                // Auto-detect and configure appropriate bootloader
                if std::fs::read_dir("/sys/firmware/efi").is_ok() {
                    self.configure_grub_uefi(hardware_info).await?;
                } else {
                    self.configure_grub_legacy(hardware_info).await?;
                }
            }
        }
        
        // Configure boot order and options
        self.setup_boot_order().await?;
        
        // Create boot entries
        self.create_boot_entries(hardware_info).await?;
        
        log::info!("Bootloader configuration completed");
        Ok(())
    }

    /// Get installation media source directory
    fn get_source_directory(&self) -> Result<PathBuf> {
        // Check various possible source locations
        let possible_sources = [
            "/mnt/multios/source",
            "/media/multios/source", 
            "/usr/share/multios/source",
            "./source",
        ];
        
        for source in &possible_sources {
            let path = Path::new(source);
            if path.exists() && path.is_dir() {
                return Ok(path.to_path_buf());
            }
        }
        
        Err(anyhow!("Installation source directory not found"))
    }

    /// Create target directory structure
    async fn create_target_directories(&self) -> Result<()> {
        let target = self.get_target_path()?;
        
        let directories = vec![
            "bin", "sbin", "usr/bin", "usr/sbin", "usr/local/bin",
            "lib", "lib64", "usr/lib", "usr/local/lib",
            "etc", "var/log", "var/tmp", "var/lib",
            "home", "root", "tmp", "opt",
            "boot", "sys", "proc", "dev",
            "run", "media", "mnt", "srv",
        ];
        
        for dir in directories {
            let dir_path = target.join(dir);
            fs::create_dir_all(&dir_path).await?;
        }
        
        Ok(())
    }

    /// Copy kernel and initramfs files
    async fn copy_kernel_files(&self) -> Result<()> {
        let source = self.get_source_directory()?;
        let target = self.get_target_path()?;
        
        let kernel_files = vec![
            ("boot/kernel", "boot/kernel"),
            ("boot/initramfs.img", "boot/initramfs.img"),
        ];
        
        for (src, dst) in kernel_files {
            let src_path = source.join(src);
            let dst_path = target.join(dst);
            
            if src_path.exists() {
                fs::copy(&src_path, &dst_path).await?;
                log::info!("Copied kernel file: {}", src);
            }
        }
        
        Ok(())
    }

    /// Copy system libraries and binaries
    async fn copy_system_libraries(&self) -> Result<()> {
        let source = self.get_source_directory()?;
        let target = self.get_target_path()?;
        
        let lib_dirs = vec![
            ("usr/lib", "usr/lib"),
            ("usr/local/lib", "usr/local/lib"),
            ("lib", "lib"),
            ("lib64", "lib64"),
        ];
        
        for (src_dir, dst_dir) in lib_dirs {
            let src_path = source.join(src_dir);
            let dst_path = target.join(dst_dir);
            
            if src_path.exists() {
                self.copy_directory_recursive(&src_path, &dst_path).await?;
                log::info!("Copied library directory: {}", src_dir);
            }
        }
        
        Ok(())
    }

    /// Copy device drivers based on hardware info
    async fn copy_device_drivers(&self, hardware_info: &HardwareInfo) -> Result<()> {
        let source = self.get_source_directory()?;
        let target = self.get_target_path()?;
        
        // Copy graphics drivers
        let gpu_vendor = &hardware_info.graphics.gpu_vendor;
        let graphics_driver_dir = if gpu_vendor.contains("NVIDIA") {
            "drivers/nvidia"
        } else if gpu_vendor.contains("AMD") {
            "drivers/amdgpu"
        } else if gpu_vendor.contains("Intel") {
            "drivers/intel"
        } else {
            "drivers/fallback"
        };
        
        let graphics_src = source.join(graphics_driver_dir);
        let graphics_dst = target.join("usr/lib/modules/extra");
        
        if graphics_src.exists() {
            self.copy_directory_recursive(&graphics_src, &graphics_dst).await?;
            log::info!("Copied graphics drivers for {}", gpu_vendor);
        }
        
        // Copy network drivers
        let network_src = source.join("drivers/network");
        let network_dst = target.join("usr/lib/modules/extra/network");
        
        if network_src.exists() {
            self.copy_directory_recursive(&network_src, &network_dst).await?;
            log::info!("Copied network drivers");
        }
        
        Ok(())
    }

    /// Copy configuration files
    async fn copy_configuration_files(&self) -> Result<()> {
        let source = self.get_source_directory()?;
        let target = self.get_target_path()?;
        
        let config_files = vec![
            ("etc/hostname", "etc/hostname"),
            ("etc/hosts", "etc/hosts"),
            ("etc/fstab", "etc/fstab"),
            ("etc/passwd", "etc/passwd"),
            ("etc/group", "etc/group"),
            ("etc/shadow", "etc/shadow"),
        ];
        
        for (src, dst) in config_files {
            let src_path = source.join(src);
            let dst_path = target.join(dst);
            
            if src_path.exists() {
                fs::copy(&src_path, &dst_path).await?;
                log::info!("Copied config file: {}", src);
            }
        }
        
        Ok(())
    }

    /// Create necessary system files
    async fn create_system_files(&self) -> Result<()> {
        let target = self.get_target_path()?;
        
        // Create empty system files
        let system_files = vec![
            "etc/mtab",
            "etc/adjtime",
            "etc/lvm/cache",
            "etc/lvm/backup",
            "etc/lvm/archive",
        ];
        
        for file in system_files {
            let file_path = target.join(file);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&file_path, "").await?;
        }
        
        // Set proper permissions
        self.set_permissions(&target).await?;
        
        Ok(())
    }

    /// Configure GRUB for legacy BIOS
    async fn configure_grub_legacy(&self, hardware_info: &HardwareInfo) -> Result<()> {
        let target = self.get_target_path()?;
        let grub_dir = target.join("boot/grub");
        
        // Create GRUB configuration
        let grub_config = self.generate_grub_config(hardware_info);
        fs::write(grub_dir.join("grub.cfg"), grub_config).await?;
        
        // Install GRUB to boot sector
        let disk_path = self.get_disk_path()?;
        self.install_grub_legacy(&disk_path, &target).await?;
        
        Ok(())
    }

    /// Configure GRUB for UEFI
    async fn configure_grub_uefi(&self, hardware_info: &HardwareInfo) -> Result<()> {
        let target = self.get_target_path()?;
        let efi_dir = target.join("boot/efi");
        
        // Create EFI GRUB configuration
        let grub_config = self.generate_efi_grub_config(hardware_info);
        fs::write(efi_dir.join("grub/grub.cfg"), grub_config).await?;
        
        // Copy GRUB EFI binary
        let grub_efi_binary = efi_dir.join("EFI/multios/grubx64.efi");
        self.install_grub_uefi(&grub_efi_binary, &target).await?;
        
        Ok(())
    }

    /// Configure GRUB for hybrid boot (both legacy and UEFI)
    async fn configure_grub_hybrid(&self, hardware_info: &HardwareInfo) -> Result<()> {
        self.configure_grub_legacy(hardware_info).await?;
        self.configure_grub_uefi(hardware_info).await?;
        Ok(())
    }

    /// Set up boot order and boot options
    async fn setup_boot_order(&self) -> Result<()> {
        // This would typically involve updating firmware boot order
        // Implementation depends on the target platform
        
        log::info!("Setting up boot order");
        Ok(())
    }

    /// Create boot entries
    async fn create_boot_entries(&self, hardware_info: &HardwareInfo) -> Result<()> {
        let target = self.get_target_path()?;
        
        // Create boot menu entries
        let boot_entries = vec![
            ("default", "MultiOS"),
            ("recovery", "MultiOS (Recovery Mode)"),
            ("safe", "MultiOS (Safe Mode)"),
        ];
        
        for (name, description) in boot_entries {
            self.create_boot_entry(name, description, hardware_info).await?;
        }
        
        Ok(())
    }

    /// Create individual boot entry
    async fn create_boot_entry(&self, name: &str, description: &str, hardware_info: &HardwareInfo) -> Result<()> {
        let target = self.get_target_path()?;
        let boot_dir = target.join("boot");
        
        let entry_config = match name {
            "recovery" => self.generate_recovery_config(hardware_info),
            "safe" => self.generate_safe_config(hardware_info),
            _ => self.generate_default_config(hardware_info),
        };
        
        let entry_path = boot_dir.join(format!("entries/{}.conf", name));
        if let Some(parent) = entry_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&entry_path, entry_config).await?;
        
        log::info!("Created boot entry: {}", name);
        Ok(())
    }

    /// Generate GRUB configuration for legacy BIOS
    fn generate_grub_config(&self, hardware_info: &HardwareInfo) -> String {
        format!(r#"
# GRUB configuration for MultiOS
set timeout=5
set default=0

# Set root filesystem
set root=(hd0,1)

# MultiOS menu entries
menuentry "MultiOS" {{
    linux /boot/kernel root=/dev/sda1 quiet splash
    initrd /boot/initramfs.img
}}

menuentry "MultiOS (Recovery Mode)" {{
    linux /boot/kernel root=/dev/sda1 recovery single
    initrd /boot/initramfs.img
}}

menuentry "MultiOS (Safe Mode)" {{
    linux /boot/kernel root=/dev/sda1 safe_mode no_acpi
    initrd /boot/initramfs.img
}}

# Hardware-specific configurations
# CPU: {cpu_vendor} {cpu_model}
# Memory: {memory_gb} GB
# Graphics: {gpu_vendor} {gpu_model}
# Storage: {storage_type} ({storage_size} GB)
"#,
            cpu_vendor = hardware_info.cpu.vendor,
            cpu_model = hardware_info.cpu.model,
            memory_gb = hardware_info.memory.total_bytes / (1024 * 1024 * 1024),
            gpu_vendor = hardware_info.graphics.gpu_vendor,
            gpu_model = hardware_info.graphics.gpu_model,
            storage_type = hardware_info.storage.devices[0].device_type,
            storage_size = hardware_info.storage.devices[0].capacity / (1024 * 1024 * 1024),
        )
    }

    /// Generate GRUB configuration for UEFI
    fn generate_efi_grub_config(&self, hardware_info: &HardwareInfo) -> String {
        // Similar to legacy but with UEFI-specific paths
        self.generate_grub_config(hardware_info)
    }

    /// Generate default boot entry configuration
    fn generate_default_config(&self, hardware_info: &HardwareInfo) -> String {
        format!(r#"
title MultiOS
linux /boot/kernel
initrd /boot/initramfs.img
options root=/dev/sda1 quiet splash
"#)
    }

    /// Generate recovery boot entry configuration
    fn generate_recovery_config(&self, hardware_info: &HardwareInfo) -> String {
        format!(r#"
title MultiOS (Recovery Mode)
linux /boot/kernel
initrd /boot/initramfs.img
options root=/dev/sda1 recovery single
"#)
    }

    /// Generate safe mode boot entry configuration
    fn generate_safe_config(&self, hardware_info: &HardwareInfo) -> String {
        format!(r#"
title MultiOS (Safe Mode)
linux /boot/kernel
initrd /boot/initramfs.img
options root=/dev/sda1 safe_mode no_acpi
"#)
    }

    /// Get the target installation path
    fn get_target_path(&self) -> Result<PathBuf> {
        match &self.config.target {
            super::config::InstallTarget::Disk(disk) => Ok(PathBuf::from(disk)),
            super::config::InstallTarget::Partition(partition) => Ok(PathBuf::from(partition)),
            _ => Err(anyhow!("Unsupported target type for installation")),
        }
    }

    /// Get disk path for bootloader installation
    fn get_disk_path(&self) -> Result<String> {
        match &self.config.target {
            super::config::InstallTarget::Disk(disk) => Ok(disk.clone()),
            super::config::InstallTarget::Partition(partition) => {
                // Extract disk from partition (e.g., /dev/sda1 -> /dev/sda)
                let disk = partition.rsplit('/').next()
                    .ok_or_else(|| anyhow!("Invalid partition path"))?
                    .chars()
                    .take_while(|c| c.is_alphabetic())
                    .collect::<String>();
                Ok(format!("/dev/{}", disk))
            }
            _ => Err(anyhow!("Cannot determine disk path for target type")),
        }
    }

    /// Install GRUB to legacy BIOS boot sector
    async fn install_grub_legacy(&self, disk_path: &str, target_path: &Path) -> Result<()> {
        let output = Command::new("grub-install")
            .args(&["--boot-directory", &target_path.join("boot").to_string_lossy(), disk_path])
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to install GRUB: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        log::info!("GRUB installed successfully to {}", disk_path);
        Ok(())
    }

    /// Install GRUB to UEFI
    async fn install_grub_uefi(&self, efi_binary: &Path, target_path: &Path) -> Result<()> {
        // Create EFI directory structure
        let efi_dir = target_path.join("boot/efi/EFI/multios");
        fs::create_dir_all(&efi_dir).await?;
        
        // Copy GRUB EFI binary
        let grub_binary_source = "/usr/lib/grub/x86_64-efi/grub.efi";
        if Path::new(grub_binary_source).exists() {
            fs::copy(grub_binary_source, efi_binary).await?;
        }
        
        log::info!("UEFI GRUB installed successfully");
        Ok(())
    }

    /// Copy directory recursively
    async fn copy_directory_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        if src.is_dir() {
            fs::create_dir_all(dst).await?;
            
            let mut entries = fs::read_dir(src).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name()
                    .ok_or_else(|| anyhow!("Invalid file name"))?;
                let dst_path = dst.join(entry_name);
                
                if entry_path.is_dir() {
                    self.copy_directory_recursive(&entry_path, &dst_path).await?;
                } else {
                    fs::copy(&entry_path, &dst_path).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Set proper permissions on system files
    async fn set_permissions(&self, target_path: &Path) -> Result<()> {
        let permissions = vec![
            (target_path.join("etc/shadow"), 0o640),
            (target_path.join("etc/passwd"), 0o644),
            (target_path.join("etc/group"), 0o644),
            (target_path.join("boot"), 0o755),
        ];
        
        for (path, mode) in permissions {
            if path.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).await?;
                }
            }
        }
        
        Ok(())
    }
}