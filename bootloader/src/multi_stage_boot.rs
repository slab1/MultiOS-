//! Multi-Stage Boot Manager
//! 
//! Orchestrates the complete multi-stage boot process including device detection,
//! configuration parsing, boot menu display, and kernel loading.

use crate::{
    BootConfig, BootMode, BootError, BootResult,
    boot_menu::{self, BootMenuEntry, BootMenuSelection},
    device_detection::{self, BootDeviceContext, BootArchitecture},
    config_parser::{self, ConfigFormat, ParsedBootConfig, ConfigParseError},
};
use spin::Mutex;

/// Boot stages in order of execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStage {
    Stage1, // Firmware/BIOS/UEFI
    Stage2, // Bootloader initialization
    Stage3, // Device detection and configuration parsing
    Stage4, // Boot menu display and selection
    Stage5, // Kernel loading
    Stage6, // Handoff to kernel
}

/// Boot stage result
pub type BootStageResult<T> = Result<T, BootStageError>;

/// Boot stage errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootStageError {
    StageFailed(BootStage),
    ConfigurationError,
    DeviceDetectionFailed,
    KernelNotFound,
    BootMenuFailed,
    InvalidBootConfig,
}

/// Multi-stage boot configuration
#[derive(Debug, Clone)]
pub struct MultiStageBootConfig {
    pub enable_boot_menu: bool,
    pub enable_device_detection: bool,
    pub enable_config_parsing: bool,
    pub default_timeout: u8,
    pub architecture: BootArchitecture,
    pub boot_modes_enabled: Vec<BootMode>,
    pub config_file_paths: Vec<&'static str>,
    pub boot_device_paths: Vec<&'static str>,
}

/// Multi-stage boot context
#[derive(Debug)]
pub struct MultiStageBootContext {
    pub current_stage: BootStage,
    pub config: MultiStageBootConfig,
    pub device_context: Option<BootDeviceContext>,
    pub parsed_config: Option<ParsedBootConfig>,
    pub selected_boot_entry: Option<BootMenuEntry>,
    pub boot_log: Vec<BootStageLog>,
}

/// Boot stage execution log entry
#[derive(Debug, Clone)]
pub struct BootStageLog {
    pub stage: BootStage,
    pub timestamp: u64,
    pub success: bool,
    pub message: String,
    pub error: Option<BootStageError>,
}

/// Global multi-stage boot context
static MULTI_STAGE_BOOT_CONTEXT: Mutex<Option<MultiStageBootContext>> = Mutex::new(None);

impl MultiStageBootConfig {
    /// Create default configuration
    pub fn default() -> Self {
        Self {
            enable_boot_menu: true,
            enable_device_detection: true,
            enable_config_parsing: true,
            default_timeout: 10,
            architecture: BootArchitecture::current(),
            boot_modes_enabled: vec![BootMode::UEFI, BootMode::LegacyBIOS],
            config_file_paths: vec![
                "/boot/multios/boot.cfg",
                "/boot/grub/grub.cfg",
                "/boot/loader/entries/multios.conf",
            ],
            boot_device_paths: vec![
                "/dev/sda",
                "/dev/sdb", 
                "/dev/mmcblk0",
            ],
        }
    }

    /// Create educational lab configuration
    pub fn for_educational_lab() -> Self {
        let mut config = Self::default();
        config.enable_boot_menu = true;
        config.default_timeout = 30;
        config.config_file_paths = vec![
            "/boot/multios/edu-lab.cfg",
        ];
        config
    }

    /// Create embedded system configuration
    pub fn for_embedded() -> Self {
        let mut config = Self::default();
        config.enable_boot_menu = false;
        config.enable_config_parsing = false;
        config.default_timeout = 3;
        config.boot_device_paths = vec!["/dev/mtd0", "/dev/mmcblk0"];
        config
    }
}

impl MultiStageBootContext {
    /// Create a new multi-stage boot context
    pub fn new(config: MultiStageBootConfig) -> Self {
        Self {
            current_stage: BootStage::Stage1,
            config,
            device_context: None,
            parsed_config: None,
            selected_boot_entry: None,
            boot_log: Vec::new(),
        }
    }

    /// Log a boot stage execution
    pub fn log_stage(&mut self, stage: BootStage, success: bool, message: &str, error: Option<BootStageError>) {
        self.boot_log.push(BootStageLog {
            stage,
            timestamp: get_current_timestamp(),
            success,
            message: message.to_string(),
            error,
        });
    }

    /// Get the last error from the boot log
    pub fn get_last_error(&self) -> Option<&BootStageError> {
        self.boot_log.iter()
            .rev()
            .find(|log| !log.success)
            .and_then(|log| log.error.as_ref())
    }
}

impl fmt::Display for BootStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootStage::Stage1 => write!(f, "Stage 1: Firmware/BIOS/UEFI"),
            BootStage::Stage2 => write!(f, "Stage 2: Bootloader Initialization"),
            BootStage::Stage3 => write!(f, "Stage 3: Device Detection & Configuration"),
            BootStage::Stage4 => write!(f, "Stage 4: Boot Menu & Selection"),
            BootStage::Stage5 => write!(f, "Stage 5: Kernel Loading"),
            BootStage::Stage6 => write!(f, "Stage 6: Kernel Handoff"),
        }
    }
}

impl fmt::Display for BootStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootStageError::StageFailed(stage) => write!(f, "Stage {} failed", stage as u8),
            BootStageError::ConfigurationError => write!(f, "Configuration error"),
            BootStageError::DeviceDetectionFailed => write!(f, "Device detection failed"),
            BootStageError::KernelNotFound => write!(f, "Kernel not found"),
            BootStageError::BootMenuFailed => write!(f, "Boot menu operation failed"),
            BootStageError::InvalidBootConfig => write!(f, "Invalid boot configuration"),
        }
    }
}

impl fmt::Display for BootStageLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.success { "SUCCESS" } else { "FAILED" };
        write!(
            f,
            "[{}] {}: {} - {}",
            status, self.stage, self.message, 
            self.error.map(|e| e.to_string()).unwrap_or_else(|| "No error".to_string())
        )
    }
}

/// Initialize multi-stage boot process
pub fn init_multi_stage_boot(config: MultiStageBootConfig) -> BootResult<MultiStageBootContext> {
    println!("\n=== Multi-Stage Boot Manager v1.0 ===");
    println!("Initializing Multi-Stage Boot Process...\n");
    
    let mut context = MultiStageBootContext::new(config);
    context.log_stage(BootStage::Stage1, true, "Multi-stage boot initialization started", None);
    
    // Store the context globally
    let mut context_guard = MULTI_STAGE_BOOT_CONTEXT.lock();
    *context_guard = Some(context);
    
    Ok(MultiStageBootContext::new(config))
}

/// Execute the complete multi-stage boot process
pub fn execute_multi_stage_boot(config: MultiStageBootConfig) -> BootResult<!> {
    let mut context = MultiStageBootContext::new(config);
    context.log_stage(BootStage::Stage1, true, "Starting multi-stage boot process", None);
    
    println!("\n=== Multi-Stage Boot Process Starting ===");
    
    // Stage 1: Firmware/BIOS/UEFI (already handled by firmware)
    context.current_stage = BootStage::Stage2;
    context.log_stage(BootStage::Stage2, true, "Bootloader initialized", None);
    
    // Stage 2: Bootloader initialization
    println!("\n--- {} ---", BootStage::Stage2);
    initialize_bootloader(&mut context)?;
    
    // Stage 3: Device detection and configuration parsing
    context.current_stage = BootStage::Stage3;
    println!("\n--- {} ---", BootStage::Stage3);
    detect_devices_and_parse_config(&mut context)?;
    
    // Stage 4: Boot menu display and selection
    context.current_stage = BootStage::Stage4;
    println!("\n--- {} ---", BootStage::Stage4);
    display_boot_menu_and_select(&mut context)?;
    
    // Stage 5: Kernel loading
    context.current_stage = BootStage::Stage5;
    println!("\n--- {} ---", BootStage::Stage5);
    load_kernel(&mut context)?;
    
    // Stage 6: Handoff to kernel
    context.current_stage = BootStage::Stage6;
    println!("\n--- {} ---", BootStage::Stage6);
    hand_off_to_kernel(&mut context)?;
    
    // This should never be reached
    Err(BootError::BootProcessError)
}

/// Stage 2: Initialize bootloader
fn initialize_bootloader(context: &mut MultiStageBootContext) -> BootStageResult<()> {
    println!("Initializing bootloader components...");
    
    // Initialize serial console if enabled
    if let Some(ref device_context) = context.device_context {
        let bootable_devices = device_context.bootable_devices();
        if !bootable_devices.is_empty() {
            println!("Found {} bootable devices", bootable_devices.len());
            for device in &bootable_devices {
                println!("  - {}", device);
            }
        }
    }
    
    context.log_stage(BootStage::Stage2, true, "Bootloader initialization completed", None);
    Ok(())
}

/// Stage 3: Device detection and configuration parsing
fn detect_devices_and_parse_config(context: &mut MultiStageBootContext) -> BootStageResult<()> {
    println!("Detecting boot devices and parsing configuration...");
    
    // Detect devices if enabled
    if context.config.enable_device_detection {
        match device_detection::init_device_detection(BootMode::UEFI) {
            Ok(device_context) => {
                context.device_context = Some(device_context);
                println!("✓ Device detection completed");
            }
            Err(_) => {
                println!("⚠ Device detection failed, using default configuration");
                context.log_stage(BootStage::Stage3, false, "Device detection failed", Some(BootStageError::DeviceDetectionFailed));
            }
        }
    }
    
    // Parse configuration files if enabled
    if context.config.enable_config_parsing {
        for config_path in &context.config.config_file_paths {
            match config_parser::parse_config_file(config_path, ConfigFormat::Grub2) {
                Ok(parsed_config) => {
                    context.parsed_config = Some(parsed_config);
                    println!("✓ Configuration loaded from: {}", config_path);
                    break;
                }
                Err(_) => {
                    println!("⚠ Failed to load configuration from: {}", config_path);
                }
            }
        }
        
        // Fallback to default configuration if none loaded
        if context.parsed_config.is_none() {
            println!("Using default configuration");
            let default_config = ParsedBootConfig::new(ConfigFormat::Custom);
            context.parsed_config = Some(default_config);
        }
    }
    
    context.log_stage(BootStage::Stage3, true, "Device detection and configuration parsing completed", None);
    Ok(())
}

/// Stage 4: Display boot menu and handle selection
fn display_boot_menu_and_select(context: &mut MultiStageBootContext) -> BootStageResult<()> {
    println!("Displaying boot menu and handling selection...");
    
    // Initialize boot menu
    let menu_config = boot_menu::BootMenuConfig {
        timeout_seconds: context.config.default_timeout,
        enable_recovery_mode: true,
        enable_debug_mode: true,
        enable_normal_mode: true,
        default_boot_mode: BootMenuSelection::Normal,
    };
    
    if let Err(_) = boot_menu::init_boot_menu(menu_config) {
        context.log_stage(BootStage::Stage4, false, "Boot menu initialization failed", Some(BootStageError::BootMenuFailed));
        return Err(BootStageError::BootMenuFailed.into());
    }
    
    // Display boot menu if enabled
    if context.config.enable_boot_menu {
        match boot_menu::display_boot_menu() {
            Ok(selected_entry) => {
                context.selected_boot_entry = Some(selected_entry);
                println!("✓ Boot menu selection completed: {}", selected_entry.label);
            }
            Err(_) => {
                // Fallback to default boot entry
                match boot_menu::get_default_boot_entry() {
                    Ok(default_entry) => {
                        context.selected_boot_entry = Some(default_entry);
                        println!("✓ Using default boot entry");
                    }
                    Err(_) => {
                        context.log_stage(BootStage::Stage4, false, "No boot entry available", Some(BootStageError::BootMenuFailed));
                        return Err(BootStageError::BootMenuFailed.into());
                    }
                }
            }
        }
    } else {
        // Use default boot entry without menu
        match boot_menu::get_default_boot_entry() {
            Ok(default_entry) => {
                context.selected_boot_entry = Some(default_entry);
                println!("✓ Using default boot entry (menu disabled)");
            }
            Err(_) => {
                context.log_stage(BootStage::Stage4, false, "No default boot entry available", Some(BootStageError::BootMenuFailed));
                return Err(BootStageError::BootMenuFailed.into());
            }
        }
    }
    
    context.log_stage(BootStage::Stage4, true, "Boot menu and selection completed", None);
    Ok(())
}

/// Stage 5: Load kernel
fn load_kernel(context: &mut MultiStageBootContext) -> BootStageResult<()> {
    println!("Loading kernel...");
    
    if let Some(ref selected_entry) = context.selected_boot_entry {
        let boot_config = selected_entry.config();
        println!("Loading kernel from: {}", boot_config.kernel_path);
        
        if let Some(command_line) = boot_config.command_line {
            println!("Kernel command line: {}", command_line);
        }
        
        if let Some(ref initrd) = boot_config.initrd_path {
            println!("Initrd: {}", initrd);
        }
        
        if boot_config.memory_test {
            println!("Memory test enabled");
        }
        
        if boot_config.serial_console {
            println!("Serial console enabled");
        }
        
        context.log_stage(BootStage::Stage5, true, "Kernel loading initiated", None);
        Ok(())
    } else {
        context.log_stage(BootStage::Stage5, false, "No boot entry selected", Some(BootStageError::KernelNotFound));
        Err(BootStageError::KernelNotFound.into())
    }
}

/// Stage 6: Handoff to kernel
fn hand_off_to_kernel(context: &mut MultiStageBootContext) -> BootStageResult<!> {
    println!("Handing off control to kernel...");
    
    if let Some(ref selected_entry) = context.selected_boot_entry {
        let boot_config = selected_entry.config();
        
        println!("Final boot configuration:");
        println!("  Kernel: {}", boot_config.kernel_path);
        if let Some(command_line) = boot_config.command_line {
            println!("  Command Line: {}", command_line);
        }
        if let Some(initrd) = boot_config.initrd_path {
            println!("  Initrd: {}", initrd);
        }
        println!("  Serial Console: {}", boot_config.serial_console);
        println!("  Memory Test: {}", boot_config.memory_test);
        
        context.log_stage(BootStage::Stage6, true, "Kernel handoff initiated", None);
        
        // Create boot information for the kernel
        let boot_info = crate::kernel_loader::create_kernel_boot_info(
            boot_config.kernel_path,
            boot_config.command_line,
        );
        
        // Actually hand off to kernel (this will not return)
        crate::kernel_loader::enter_kernel(boot_info);
        
        // If we get here, something went wrong
        context.log_stage(BootStage::Stage6, false, "Kernel failed to start", Some(BootStageError::StageFailed(BootStage::Stage6)));
        Err(BootStageError::StageFailed(BootStage::Stage6).into())
    } else {
        context.log_stage(BootStage::Stage6, false, "No boot configuration available for handoff", Some(BootStageError::InvalidBootConfig));
        Err(BootStageError::InvalidBootConfig.into())
    }
}

/// Get current timestamp (simplified)
fn get_current_timestamp() -> u64 {
    // In a real implementation, this would read from system timer
    0
}

/// Get boot configuration from context
pub fn get_boot_config_from_context() -> Result<BootConfig, BootStageError> {
    let context_guard = MULTI_STAGE_BOOT_CONTEXT.lock();
    let context = context_guard.as_ref()
        .ok_or(BootStageError::ConfigurationError)?;
        
    if let Some(ref selected_entry) = context.selected_boot_entry {
        Ok(selected_entry.config().clone())
    } else {
        Err(BootStageError::InvalidBootConfig)
    }
}

/// Get boot stage log
pub fn get_boot_stage_log() -> Result<Vec<BootStageLog>, BootStageError> {
    let context_guard = MULTI_STAGE_BOOT_CONTEXT.lock();
    let context = context_guard.as_ref()
        .ok_or(BootStageError::ConfigurationError)?;
        
    Ok(context.boot_log.clone())
}

/// Display boot stage log
pub fn display_boot_stage_log() -> Result<(), BootStageError> {
    let log = get_boot_stage_log()?;
    
    println!("\n=== Boot Stage Execution Log ===");
    for entry in &log {
        println!("{}", entry);
    }
    
    Ok(())
}

/// Resume boot process from a specific stage
pub fn resume_boot_from_stage(stage: BootStage, config: MultiStageBootConfig) -> BootResult<!> {
    println!("Resuming boot process from: {}", stage);
    
    let mut context = MultiStageBootContext::new(config);
    context.current_stage = stage;
    
    match stage {
        BootStage::Stage2 => initialize_bootloader(&mut context)?,
        BootStage::Stage3 => {
            detect_devices_and_parse_config(&mut context)?;
            display_boot_menu_and_select(&mut context)?;
        }
        BootStage::Stage4 => {
            display_boot_menu_and_select(&mut context)?;
            load_kernel(&mut context)?;
        }
        BootStage::Stage5 => {
            load_kernel(&mut context)?;
            hand_off_to_kernel(&mut context)?;
        }
        BootStage::Stage6 => {
            hand_off_to_kernel(&mut context)?;
        }
        BootStage::Stage1 => {
            return execute_multi_stage_boot(config);
        }
    }
    
    Err(BootError::BootProcessError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_stage_boot_config_creation() {
        let config = MultiStageBootConfig::default();
        
        assert!(config.enable_boot_menu);
        assert!(config.enable_device_detection);
        assert!(config.enable_config_parsing);
        assert_eq!(config.default_timeout, 10);
        assert!(!config.config_file_paths.is_empty());
    }

    #[test]
    fn test_stage_display() {
        assert_eq!(format!("{}", BootStage::Stage1), "Stage 1: Firmware/BIOS/UEFI");
        assert_eq!(format!("{}", BootStage::Stage6), "Stage 6: Kernel Handoff");
    }

    #[test]
    fn test_boot_stage_log() {
        let log = BootStageLog {
            stage: BootStage::Stage2,
            timestamp: 12345,
            success: true,
            message: "Test message".to_string(),
            error: None,
        };
        
        assert!(format!("{}", log).contains("Stage 2"));
        assert!(format!("{}", log).contains("Test message"));
        assert!(format!("{}", log).contains("SUCCESS"));
    }

    #[test]
    fn test_context_logging() {
        let mut context = MultiStageBootContext::new(MultiStageBootConfig::default());
        
        context.log_stage(BootStage::Stage2, true, "Test stage", None);
        
        assert_eq!(context.boot_log.len(), 1);
        assert_eq!(context.boot_log[0].stage, BootStage::Stage2);
        assert!(context.boot_log[0].success);
    }
}