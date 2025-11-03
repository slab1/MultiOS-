pub mod error;

use crate::hardware::{HardwareInfo, GraphicsDevice, NetworkInterface, StorageDevice};
use crate::core::config::DriverSelectionConfig;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tokio::fs;

/// Driver manager for handling driver installation and management
pub struct DriverManager {
    hardware_info: HardwareInfo,
}

impl DriverManager {
    pub fn new(hardware_info: HardwareInfo) -> Self {
        Self { hardware_info }
    }

    /// Install drivers based on hardware configuration and user selection
    pub async fn install_drivers(&self, selection: DriverSelection) -> Result<()> {
        log::info!("Starting driver installation");
        
        // Install graphics drivers
        if !selection.graphics_drivers.is_empty() {
            self.install_graphics_drivers(&selection.graphics_drivers).await?;
        }
        
        // Install network drivers
        if !selection.network_drivers.is_empty() {
            self.install_network_drivers(&selection.network_drivers).await?;
        }
        
        // Install storage drivers
        if !selection.storage_drivers.is_empty() {
            self.install_storage_drivers(&selection.storage_drivers).await?;
        }
        
        // Install audio drivers
        if let Some(audio_driver) = &selection.audio_driver {
            self.install_audio_driver(audio_driver).await?;
        }
        
        // Install custom drivers
        if !selection.custom_drivers.is_empty() {
            self.install_custom_drivers(&selection.custom_drivers).await?;
        }
        
        // Update module dependencies
        self.update_module_dependencies().await?;
        
        // Configure driver loading order
        self.configure_driver_loading().await?;
        
        log::info!("Driver installation completed");
        Ok(())
    }

    /// Detect compatible graphics drivers for the hardware
    pub async fn detect_graphics_drivers(&self, hardware_info: &HardwareInfo) -> Result<Vec<GraphicsDriver>> {
        log::info!("Detecting compatible graphics drivers");
        
        let mut compatible_drivers = Vec::new();
        let primary_device = &hardware_info.graphics.primary_device;
        
        // NVIDIA drivers
        if primary_device.vendor.contains("NVIDIA") {
            let nvidia_driver = GraphicsDriver {
                name: "NVIDIA".to_string(),
                version: self.detect_nvidia_version().await?,
                description: "NVIDIA proprietary driver".to_string(),
                hardware_support: vec![primary_device.model.clone()],
                recommended: true,
                open_source: false,
            };
            compatible_drivers.push(nvidia_driver);
            
            // Nouveau (open source NVIDIA driver)
            let nouveau_driver = GraphicsDriver {
                name: "Nouveau".to_string(),
                version: "Open Source".to_string(),
                description: "Nouveau open source driver for NVIDIA".to_string(),
                hardware_support: vec![primary_device.model.clone()],
                recommended: false,
                open_source: true,
            };
            compatible_drivers.push(nouveau_driver);
        }
        
        // AMD drivers
        if primary_device.vendor.contains("AMD") {
            let amdgpu_driver = GraphicsDriver {
                name: "AMDGPU".to_string(),
                version: "Open Source".to_string(),
                description: "AMDGPU open source driver".to_string(),
                hardware_support: vec![primary_device.model.clone()],
                recommended: true,
                open_source: true,
            };
            compatible_drivers.push(amdgpu_driver);
            
            // AMD proprietary driver (if available)
            if self.amd_proprietary_available().await? {
                let amd_driver = GraphicsDriver {
                    name: "AMDGPU-PRO".to_string(),
                    version: "Proprietary".to_string(),
                    description: "AMDGPU-PRO proprietary driver".to_string(),
                    hardware_support: vec![primary_device.model.clone()],
                    recommended: false,
                    open_source: false,
                };
                compatible_drivers.push(amd_driver);
            }
        }
        
        // Intel drivers
        if primary_device.vendor.contains("Intel") {
            let intel_driver = GraphicsDriver {
                name: "Intel".to_string(),
                version: "Open Source".to_string(),
                description: "Intel open source graphics driver".to_string(),
                hardware_support: vec![primary_device.model.clone()],
                recommended: true,
                open_source: true,
            };
            compatible_drivers.push(intel_driver);
        }
        
        // Fallback drivers
        let vesa_driver = GraphicsDriver {
            name: "VESA".to_string(),
            version: "Generic".to_string(),
            description: "VESA generic graphics driver".to_string(),
            hardware_support: vec!["Generic".to_string()],
            recommended: false,
            open_source: true,
        };
        compatible_drivers.push(vesa_driver);
        
        log::info!("Found {} compatible graphics drivers", compatible_drivers.len());
        Ok(compatible_drivers)
    }

    /// Detect compatible network drivers for the hardware
    pub async fn detect_network_drivers(&self, hardware_info: &HardwareInfo) -> Result<Vec<NetworkDriver>> {
        log::info!("Detecting compatible network drivers");
        
        let mut compatible_drivers = Vec::new();
        
        for interface in &hardware_info.network.devices {
            let driver_info = self.detect_network_driver_for_interface(interface).await?;
            compatible_drivers.push(driver_info);
        }
        
        log::info!("Found {} compatible network drivers", compatible_drivers.len());
        Ok(compatible_drivers)
    }

    /// Detect compatible storage drivers for the hardware
    pub async fn detect_storage_drivers(&self, hardware_info: &HardwareInfo) -> Result<Vec<StorageDriver>> {
        log::info!("Detecting compatible storage drivers");
        
        let mut compatible_drivers = Vec::new();
        
        for device in &hardware_info.storage.devices {
            let driver_info = self.detect_storage_driver_for_device(device).await?;
            compatible_drivers.push(driver_info);
        }
        
        log::info!("Found {} compatible storage drivers", compatible_drivers.len());
        Ok(compatible_drivers)
    }

    /// Install graphics drivers
    async fn install_graphics_drivers(&self, drivers: &[String]) -> Result<()> {
        log::info!("Installing graphics drivers: {:?}", drivers);
        
        for driver_name in drivers {
            match driver_name.as_str() {
                "nvidia" => self.install_nvidia_driver().await?,
                "nouveau" => self.install_nouveau_driver().await?,
                "amdgpu" => self.install_amdgpu_driver().await?,
                "amd_pro" => self.install_amd_proprietary_driver().await?,
                "intel" => self.install_intel_driver().await?,
                _ => log::warn!("Unknown graphics driver: {}", driver_name),
            }
        }
        
        Ok(())
    }

    /// Install NVIDIA driver
    async fn install_nvidia_driver(&self) -> Result<()> {
        log::info!("Installing NVIDIA driver");
        
        // Check if NVIDIA driver is available
        if !Command::new("which").arg("nvidia-install").output().await?.status.success() {
            log::warn!("NVIDIA installer not found, using package manager");
            // Fall back to package manager
            let output = Command::new("apt")
                .args(&["install", "-y", "nvidia-driver"])
                .output()?;
                
            if !output.status.success() {
                return Err(anyhow!("Failed to install NVIDIA driver via package manager"));
            }
        }
        
        // Configure NVIDIA driver
        self.configure_nvidia_driver().await?;
        
        Ok(())
    }

    /// Install Nouveau driver
    async fn install_nouveau_driver(&self) -> Result<()> {
        log::info!("Installing Nouveau driver");
        
        // Load nouveau module
        let output = Command::new("modprobe")
            .args(&["nouveau"])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to load Nouveau driver: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        // Add nouveau to modules to load at boot
        self.add_module_to_loading("nouveau").await?;
        
        Ok(())
    }

    /// Install AMDGPU driver
    async fn install_amdgpu_driver(&self) -> Result<()> {
        log::info!("Installing AMDGPU driver");
        
        // Load amdgpu module
        let output = Command::new("modprobe")
            .args(&["amdgpu"])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to load AMDGPU driver: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        self.add_module_to_loading("amdgpu").await?;
        
        Ok(())
    }

    /// Install AMD proprietary driver
    async fn install_amd_proprietary_driver(&self) -> Result<()> {
        log::info!("Installing AMD proprietary driver");
        
        // Download and install AMDGPU-PRO driver
        let driver_url = "https://drivers.amd.com/drivers/linux/";
        log::info!("AMD proprietary driver should be downloaded from: {}", driver_url);
        
        // For now, just configure for potential installation
        self.configure_amd_driver().await?;
        
        Ok(())
    }

    /// Install Intel driver
    async fn install_intel_driver(&self) -> Result<()> {
        log::info!("Installing Intel driver");
        
        // Load intel modules
        let modules = ["intel_agp", "intel_graphics"];
        
        for module in &modules {
            let output = Command::new("modprobe").arg(module).output()?;
            if !output.status.success() {
                log::warn!("Failed to load Intel module {}: {}", module,
                    String::from_utf8_lossy(&output.stderr));
            }
            self.add_module_to_loading(module).await?;
        }
        
        Ok(())
    }

    /// Install network drivers
    async fn install_network_drivers(&self, drivers: &[String]) -> Result<()> {
        log::info!("Installing network drivers: {:?}", drivers);
        
        for driver_name in drivers {
            match driver_name.as_str() {
                "e1000" => self.install_e1000_driver().await?,
                "rtl8111" => self.install_rtl8111_driver().await?,
                "iwlwifi" => self.install_iwlwifi_driver().await?,
                "ath10k" => self.install_ath10k_driver().await?,
                _ => log::warn!("Unknown network driver: {}", driver_name),
            }
        }
        
        Ok(())
    }

    /// Install e1000 driver (Intel Ethernet)
    async fn install_e1000_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["e1000"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load e1000 driver"));
        }
        self.add_module_to_loading("e1000").await?;
        Ok(())
    }

    /// Install RTL8111 driver (Realtek Ethernet)
    async fn install_rtl8111_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["r8169"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load RTL8111 driver"));
        }
        self.add_module_to_loading("r8169").await?;
        Ok(())
    }

    /// Install iwlwifi driver (Intel WiFi)
    async fn install_iwlwifi_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["iwlwifi"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load iwlwifi driver"));
        }
        self.add_module_to_loading("iwlwifi").await?;
        Ok(())
    }

    /// Install ath10k driver (Atheros WiFi)
    async fn install_ath10k_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["ath10k_core"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load ath10k driver"));
        }
        self.add_module_to_loading("ath10k_core").await?;
        Ok(())
    }

    /// Install storage drivers
    async fn install_storage_drivers(&self, drivers: &[String]) -> Result<()> {
        log::info!("Installing storage drivers: {:?}", drivers);
        
        for driver_name in drivers {
            match driver_name.as_str() {
                "ahci" => self.install_ahci_driver().await?,
                "nvme" => self.install_nvme_driver().await?,
                "mpt3sas" => self.install_mpt3sas_driver().await?,
                _ => log::warn!("Unknown storage driver: {}", driver_name),
            }
        }
        
        Ok(())
    }

    /// Install AHCI driver
    async fn install_ahci_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["ahci"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load AHCI driver"));
        }
        self.add_module_to_loading("ahci").await?;
        Ok(())
    }

    /// Install NVMe driver
    async fn install_nvme_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["nvme"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load NVMe driver"));
        }
        self.add_module_to_loading("nvme").await?;
        Ok(())
    }

    /// Install MPT3SAS driver
    async fn install_mpt3sas_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["mpt3sas"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load MPT3SAS driver"));
        }
        self.add_module_to_loading("mpt3sas").await?;
        Ok(())
    }

    /// Install audio driver
    async fn install_audio_driver(&self, driver_name: &str) -> Result<()> {
        log::info!("Installing audio driver: {}", driver_name);
        
        match driver_name {
            "hda_intel" => self.install_hda_intel_driver().await?,
            "snd_hda_intel" => self.install_snd_hda_intel_driver().await?,
            _ => log::warn!("Unknown audio driver: {}", driver_name),
        }
        
        Ok(())
    }

    /// Install HDA Intel audio driver
    async fn install_hda_intel_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["snd_hda_intel"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load HDA Intel audio driver"));
        }
        self.add_module_to_loading("snd_hda_intel").await?;
        Ok(())
    }

    /// Install snd_hda_intel driver
    async fn install_snd_hda_intel_driver(&self) -> Result<()> {
        let output = Command::new("modprobe").args(&["snd_hda_intel"]).output()?;
        if !output.status.success() {
            return Err(anyhow!("Failed to load snd_hda_intel driver"));
        }
        self.add_module_to_loading("snd_hda_intel").await?;
        Ok(())
    }

    /// Install custom drivers
    async fn install_custom_drivers(&self, drivers: &[String]) -> Result<()> {
        log::info!("Installing custom drivers: {:?}", drivers);
        
        for driver_path in drivers {
            if Path::new(driver_path).exists() {
                self.install_driver_from_path(driver_path).await?;
            } else {
                log::warn!("Custom driver not found: {}", driver_path);
            }
        }
        
        Ok(())
    }

    /// Install driver from file path
    async fn install_driver_from_path(&self, driver_path: &str) -> Result<()> {
        log::info!("Installing custom driver from: {}", driver_path);
        
        // Copy driver to modules directory
        let target_path = format!("/lib/modules/$(uname -r)/kernel/drivers/extra/{}", 
            Path::new(driver_path).file_name().unwrap().to_string_lossy());
        
        // Create target directory
        if let Some(parent) = Path::new(&target_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Copy driver file
        tokio::fs::copy(driver_path, &target_path).await?;
        
        // Update module dependencies
        let output = Command::new("depmod").output()?;
        if !output.status.success() {
            log::warn!("Failed to update module dependencies");
        }
        
        Ok(())
    }

    /// Update module dependencies
    async fn update_module_dependencies(&self) -> Result<()> {
        log::info!("Updating module dependencies");
        
        let output = Command::new("depmod")
            .arg("-a")
            .output()?;
            
        if !output.status.success() {
            return Err(anyhow!("Failed to update module dependencies"));
        }
        
        Ok(())
    }

    /// Configure driver loading order
    async fn configure_driver_loading(&self) -> Result<()> {
        log::info!("Configuring driver loading order");
        
        // Update initramfs to include new drivers
        let output = Command::new("update-initramfs")
            .args(&["-u"])
            .output()?;
            
        if !output.status.success() {
            log::warn!("Failed to update initramfs: {}", 
                String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }

    // Helper methods for driver detection
    fn detect_nvidia_version(&self) -> Result<String> {
        // Check for NVIDIA driver version
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("nvidia-settings")
                .arg("--query")
                .arg("DriverVersion")
                .output() {
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
                }
            }
        }
        Ok("Unknown".to_string())
    }

    async fn amd_proprietary_available(&self) -> Result<bool> {
        // Check if AMDGPU-PRO package is available
        let output = Command::new("apt")
            .args(&["list", "amdgpu-pro", "--installed"])
            .output()?;
            
        Ok(output.status.success())
    }

    async fn detect_network_driver_for_interface(&self, interface: &NetworkInterface) -> Result<NetworkDriver> {
        let driver_info = NetworkDriver {
            name: interface.driver.clone(),
            description: format!("Driver for {} interface", interface.interface_name),
            supported_devices: vec![interface.interface_name.clone()],
            recommended: self.is_recommended_network_driver(&interface.driver),
        };
        
        Ok(driver_info)
    }

    async fn detect_storage_driver_for_device(&self, device: &StorageDevice) -> Result<StorageDriver> {
        let driver_info = StorageDriver {
            name: self.infer_storage_driver(&device.device_type),
            description: format!("Driver for {} device", device.model),
            supported_devices: vec![device.device_name.clone()],
            recommended: true,
        };
        
        Ok(driver_info)
    }

    fn is_recommended_network_driver(&self, driver_name: &str) -> bool {
        let recommended_drivers = ["e1000", "r8169", "iwlwifi", "ath10k"];
        recommended_drivers.contains(&driver_name)
    }

    fn infer_storage_driver(&self, device_type: &str) -> String {
        match device_type.to_lowercase().as_str() {
            s if s.contains("nvme") => "nvme".to_string(),
            s if s.contains("sata") => "ahci".to_string(),
            s if s.contains("scsi") => "mpt3sas".to_string(),
            _ => "ahci".to_string(), // Default
        }
    }

    async fn configure_nvidia_driver(&self) -> Result<()> {
        // Configure NVIDIA driver settings
        // This would involve creating Xorg configuration files, etc.
        log::info!("Configuring NVIDIA driver");
        Ok(())
    }

    async fn configure_amd_driver(&self) -> Result<()> {
        // Configure AMD driver settings
        log::info!("Configuring AMD driver");
        Ok(())
    }

    async fn add_module_to_loading(&self, module_name: &str) -> Result<()> {
        // Add module to /etc/modules for automatic loading at boot
        let modules_file = "/etc/modules";
        
        // Read existing content
        let existing_content = if Path::new(modules_file).exists() {
            tokio::fs::read_to_string(modules_file).await?
        } else {
            String::new()
        };
        
        // Add module if not already present
        if !existing_content.contains(module_name) {
            let updated_content = if existing_content.is_empty() {
                format!("{}\n", module_name)
            } else {
                format!("{}\n{}\n", existing_content.trim(), module_name)
            };
            
            tokio::fs::write(modules_file, updated_content).await?;
            log::info!("Added {} to module loading list", module_name);
        }
        
        Ok(())
    }
}

// Driver selection structure that contains user's choices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverSelection {
    pub graphics_drivers: Vec<String>,
    pub network_drivers: Vec<String>,
    pub storage_drivers: Vec<String>,
    pub audio_driver: Option<String>,
    pub custom_drivers: Vec<String>,
}

impl Default for DriverSelection {
    fn default() -> Self {
        Self {
            graphics_drivers: Vec::new(),
            network_drivers: Vec::new(),
            storage_drivers: Vec::new(),
            audio_driver: None,
            custom_drivers: Vec::new(),
        }
    }
}

// Driver information structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsDriver {
    pub name: String,
    pub version: String,
    pub description: String,
    pub hardware_support: Vec<String>,
    pub recommended: bool,
    pub open_source: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDriver {
    pub name: String,
    pub description: String,
    pub supported_devices: Vec<String>,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDriver {
    pub name: String,
    pub description: String,
    pub supported_devices: Vec<String>,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDriver {
    pub name: String,
    pub description: String,
    pub supported_devices: Vec<String>,
    pub recommended: bool,
}