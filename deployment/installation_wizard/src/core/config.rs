use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Missing required configuration: {0}")]
    MissingValue(String),
    
    #[error("Configuration file error: {0}")]
    FileError(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Installation target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallTarget {
    /// Install to a specific disk
    Disk(String),
    /// Install to a specific partition
    Partition(String),
    /// Network installation
    Network {
        server: String,
        path: String,
    },
    /// USB installation
    Usb {
        device: String,
        persistence: bool,
    },
}

impl InstallTarget {
    pub fn validate(&self) -> Result<()> {
        match self {
            InstallTarget::Disk(path) => {
                if path.is_empty() {
                    return Err(ConfigError::InvalidValue("Disk path cannot be empty".into()).into());
                }
            }
            InstallTarget::Partition(path) => {
                if path.is_empty() {
                    return Err(ConfigError::InvalidValue("Partition path cannot be empty".into()).into());
                }
            }
            InstallTarget::Network { server, path } => {
                if server.is_empty() {
                    return Err(ConfigError::InvalidValue("Network server cannot be empty".into()).into());
                }
                if path.is_empty() {
                    return Err(ConfigError::InvalidValue("Network path cannot be empty".into()).into());
                }
            }
            InstallTarget::Usb { device, .. } => {
                if device.is_empty() {
                    return Err(ConfigError::InvalidValue("USB device cannot be empty".into()).into());
                }
            }
        }
        Ok(())
    }
}

/// Boot type configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootType {
    /// Legacy BIOS boot
    Legacy,
    /// UEFI boot
    Uefi,
    /// Hybrid boot (both legacy and UEFI)
    Hybrid,
    /// Auto-detect boot type
    Auto,
}

impl BootType {
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            // Check for UEFI on x86_64
            if std::fs::read_dir("/sys/firmware/efi").is_ok() {
                BootType::Uefi
            } else {
                BootType::Legacy
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            // Most ARM64 systems use UEFI
            BootType::Uefi
        }
    }

    pub fn validate_hardware(&self, cpu_architecture: &str) -> Result<()> {
        match self {
            BootType::Legacy => {
                if !["x86_64"].contains(&cpu_architecture) {
                    return Err(ConfigError::Validation("Legacy boot not supported on this architecture".into()).into());
                }
            }
            BootType::Uefi | BootType::Hybrid | BootType::Auto => {
                // UEFI is supported on most modern architectures
                if !["x86_64", "ARM64"].contains(&cpu_architecture) {
                    return Err(ConfigError::Validation("UEFI boot may not be fully supported on this architecture".into()).into());
                }
            }
        }
        Ok(())
    }
}

/// User account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub full_name: Option<String>,
    pub password: Option<String>,
    pub is_admin: bool,
    pub auto_login: bool,
}

/// Partition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    pub root_size: u64,           // Root partition size in bytes
    pub home_size: u64,           // Home partition size in bytes (0 for single partition)
    pub swap_size: u64,           // Swap partition size in bytes
    pub boot_size: u64,           // Boot partition size in bytes
    pub use_lvm: bool,            // Use LVM
    pub encryption: bool,         // Encrypt partitions
    pub encryption_password: Option<String>,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub dhcp: bool,
    pub static_ip: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub hostname: String,
}

/// Driver selection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverSelectionConfig {
    pub graphics_driver: String,
    pub network_drivers: Vec<String>,
    pub audio_driver: Option<String>,
    pub custom_drivers: Vec<String>,
    pub auto_install_recommended: bool,
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub enable_recovery: bool,
    pub recovery_points: usize,
    pub max_recovery_size: u64,   // Maximum recovery data size in bytes
    pub enable_rollback: bool,
}

/// Main installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationConfig {
    pub target: InstallTarget,
    pub boot_type: BootType,
    pub locale: String,
    pub timezone: String,
    pub keyboard_layout: String,
    pub username: String,
    pub full_name: Option<String>,
    pub password: Option<String>,
    pub auto_login: bool,
    pub hostname: String,
    pub partition_config: PartitionConfig,
    pub network_config: NetworkConfig,
    pub driver_selection: DriverSelectionConfig,
    pub recovery_config: RecoveryConfig,
    pub dry_run: bool,
    pub skip_warnings: bool,
    pub custom_options: std::collections::HashMap<String, String>,
}

impl Default for InstallationConfig {
    fn default() -> Self {
        let boot_type = BootType::detect();
        let partition_config = PartitionConfig {
            root_size: 20 * 1024 * 1024 * 1024, // 20 GB
            home_size: 0, // Single partition setup
            swap_size: 4 * 1024 * 1024 * 1024,  // 4 GB
            boot_size: 512 * 1024 * 1024,       // 512 MB
            use_lvm: false,
            encryption: false,
            encryption_password: None,
        };

        let network_config = NetworkConfig {
            dhcp: true,
            static_ip: None,
            netmask: None,
            gateway: None,
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            hostname: "multios".to_string(),
        };

        let driver_selection = DriverSelectionConfig {
            graphics_driver: "auto".to_string(),
            network_drivers: vec![],
            audio_driver: None,
            custom_drivers: vec![],
            auto_install_recommended: true,
        };

        let recovery_config = RecoveryConfig {
            enable_recovery: true,
            recovery_points: 5,
            max_recovery_size: 5 * 1024 * 1024 * 1024, // 5 GB
            enable_rollback: true,
        };

        InstallationConfig {
            target: InstallTarget::Disk("/dev/sda".to_string()),
            boot_type,
            locale: "en_US.UTF-8".to_string(),
            timezone: "UTC".to_string(),
            keyboard_layout: "us".to_string(),
            username: "user".to_string(),
            full_name: None,
            password: None,
            auto_login: false,
            hostname: "multios".to_string(),
            partition_config,
            network_config,
            driver_selection,
            recovery_config,
            dry_run: false,
            skip_warnings: false,
            custom_options: std::collections::HashMap::new(),
        }
    }
}

impl InstallationConfig {
    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(ConfigError::FileError(format!(
                "Configuration file does not exist: {:?}",
                path
            )).into());
        }

        let content = std::fs::read_to_string(path)?;
        let config: InstallationConfig = serde_json::from_str(&content)?;
        
        config.validate()?;
        
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate target
        self.target.validate()?;

        // Validate boot type
        self.boot_type.validate_hardware(&std::env::consts::ARCH)?;

        // Validate username
        if self.username.is_empty() {
            return Err(ConfigError::InvalidValue("Username cannot be empty".into()).into());
        }

        if self.username.contains(' ') {
            return Err(ConfigError::InvalidValue("Username cannot contain spaces".into()).into());
        }

        // Validate hostname
        if self.hostname.is_empty() {
            return Err(ConfigError::InvalidValue("Hostname cannot be empty".into()).into());
        }

        // Validate partition sizes
        if self.partition_config.root_size < 5 * 1024 * 1024 * 1024 {
            return Err(ConfigError::InvalidValue("Root partition must be at least 5 GB".into()).into());
        }

        // Validate timezone
        if self.timezone.is_empty() {
            return Err(ConfigError::InvalidValue("Timezone cannot be empty".into()).into());
        }

        // Validate locale
        if self.locale.is_empty() {
            return Err(ConfigError::InvalidValue("Locale cannot be empty".into()).into());
        }

        // Validate recovery configuration
        if self.recovery_config.recovery_points == 0 {
            return Err(ConfigError::InvalidValue("Must have at least one recovery point".into()).into());
        }

        if self.recovery_config.max_recovery_size < 1024 * 1024 * 1024 {
            return Err(ConfigError::InvalidValue("Recovery size must be at least 1 GB".into()).into());
        }

        Ok(())
    }

    /// Get the recommended configuration based on hardware
    pub fn recommended_for_hardware(hardware_info: &crate::hardware::HardwareInfo) -> Self {
        let mut config = InstallationConfig::default();

        // Adjust partition sizes based on available storage
        let total_storage = hardware_info.storage.total_capacity;
        let available_storage = total_storage * 80 / 100; // Use 80% of available storage

        if available_storage >= 100 * 1024 * 1024 * 1024 {
            // 100GB or more - full desktop setup
            config.partition_config.root_size = 50 * 1024 * 1024 * 1024; // 50GB
            config.partition_config.home_size = available_storage - 
                (50 + 4 + 1) * 1024 * 1024 * 1024; // Remaining space for home
        } else if available_storage >= 50 * 1024 * 1024 * 1024 {
            // 50GB or more - standard setup
            config.partition_config.root_size = 30 * 1024 * 1024 * 1024; // 30GB
            config.partition_config.home_size = available_storage - 
                (30 + 4 + 1) * 1024 * 1024 * 1024; // Remaining space for home
        }

        // Adjust swap size based on memory
        let total_memory = hardware_info.memory.total_bytes;
        if total_memory <= 4 * 1024 * 1024 * 1024 {
            config.partition_config.swap_size = total_memory * 2; // 2x RAM
        } else if total_memory <= 16 * 1024 * 1024 * 1024 {
            config.partition_config.swap_size = total_memory; // 1x RAM
        } else {
            config.partition_config.swap_size = 8 * 1024 * 1024 * 1024; // 8GB max
        }

        // Configure recommended drivers based on hardware
        let gpu_vendor = &hardware_info.graphics.gpu_vendor;
        if gpu_vendor.contains("NVIDIA") {
            config.driver_selection.graphics_driver = "nvidia".to_string();
        } else if gpu_vendor.contains("AMD") {
            config.driver_selection.graphics_driver = "amdgpu".to_string();
        } else if gpu_vendor.contains("Intel") {
            config.driver_selection.graphics_driver = "intel".to_string();
        }

        config
    }

    /// Create a minimal configuration for testing
    pub fn minimal() -> Self {
        InstallationConfig {
            target: InstallTarget::Disk("/dev/sda".to_string()),
            boot_type: BootType::Auto,
            locale: "C".to_string(),
            timezone: "UTC".to_string(),
            keyboard_layout: "us".to_string(),
            username: "testuser".to_string(),
            full_name: None,
            password: Some("test".to_string()),
            auto_login: true,
            hostname: "multios-test".to_string(),
            partition_config: PartitionConfig {
                root_size: 10 * 1024 * 1024 * 1024, // 10GB
                home_size: 0, // Single partition
                swap_size: 2 * 1024 * 1024 * 1024,  // 2GB
                boot_size: 512 * 1024 * 1024,       // 512MB
                use_lvm: false,
                encryption: false,
                encryption_password: None,
            },
            network_config: NetworkConfig {
                dhcp: true,
                static_ip: None,
                netmask: None,
                gateway: None,
                dns_servers: vec!["8.8.8.8".to_string()],
                hostname: "multios-test".to_string(),
            },
            driver_selection: DriverSelectionConfig {
                graphics_driver: "auto".to_string(),
                network_drivers: vec![],
                audio_driver: None,
                custom_drivers: vec![],
                auto_install_recommended: false,
            },
            recovery_config: RecoveryConfig {
                enable_recovery: false,
                recovery_points: 0,
                max_recovery_size: 0,
                enable_rollback: false,
            },
            dry_run: true,
            skip_warnings: true,
            custom_options: std::collections::HashMap::new(),
        }
    }
}