//! Multi-Stage Boot Example
//! 
//! This example demonstrates the complete multi-stage boot system
//! including device detection, configuration parsing, boot menu, and
//! different boot modes.

#![no_std]

extern crate bootloader;
extern crate spin;
extern crate log;

use bootloader::boot_config::{BootConfig, BootMode};
use bootloader::boot_menu::{BootMenuEntry, BootMenuConfig, BootMenuSelection};
use bootloader::device_detection::{BootDevice, BootDeviceType, BootArchitecture, BootDeviceContext};
use bootloader::config_parser::{ConfigEntry, ParsedBootConfig, ConfigFormat, ConfigParseError};
use bootloader::multi_stage_boot::{MultiStageBootConfig, MultiStageBootContext, BootStage, BootStageError};
use log::{info, warn, error, debug};

/// Example boot configurations for different scenarios
#[allow(dead_code)]
struct BootExampleScenarios;

impl BootExampleScenarios {
    /// Educational lab boot scenario
    pub fn educational_lab_scenario() -> MultiStageBootConfig {
        info!("Setting up educational lab boot scenario");
        
        let mut config = MultiStageBootConfig::for_educational_lab();
        
        // Enable all boot options for learning
        config.enable_boot_menu = true;
        config.enable_device_detection = true;
        config.enable_config_parsing = true;
        config.default_timeout = 30; // Longer timeout for learning
        
        // Add educational-specific configuration files
        config.config_file_paths = vec![
            "/boot/multios/edu-lab.cfg",
            "/boot/multios/boot.cfg",
            "/boot/grub/grub.cfg",
        ];
        
        info!("Educational lab configuration: timeout={}s, menu enabled", config.default_timeout);
        config
    }
    
    /// Production server boot scenario
    pub fn production_server_scenario() -> MultiStageBootConfig {
        info!("Setting up production server boot scenario");
        
        let mut config = MultiStageBootConfig::default();
        
        // Fast boot with minimal user interaction
        config.enable_boot_menu = true;
        config.enable_device_detection = true;
        config.enable_config_parsing = true;
        config.default_timeout = 5; // Short timeout
        
        // Only essential boot modes
        config.config_file_paths = vec![
            "/boot/multios/production.cfg",
            "/boot/multios/boot.cfg",
        ];
        
        info!("Production server configuration: timeout={}s", config.default_timeout);
        config
    }
    
    /// Embedded system boot scenario
    pub fn embedded_system_scenario() -> MultiStageBootConfig {
        info!("Setting up embedded system boot scenario");
        
        let mut config = MultiStageBootConfig::for_embedded();
        
        // Minimal features for fast, deterministic boot
        config.enable_boot_menu = false;
        config.enable_device_detection = true;
        config.enable_config_parsing = false;
        config.default_timeout = 3;
        
        // Limited device paths for embedded hardware
        config.boot_device_paths = vec!["/dev/mtd0", "/dev/mmcblk0"];
        
        info!("Embedded system configuration: no menu, minimal features");
        config
    }
    
    /// Development/Debug boot scenario
    pub fn development_debug_scenario() -> MultiStageBootConfig {
        info!("Setting up development/debug boot scenario");
        
        let mut config = MultiStageBootConfig::default();
        
        // Enable maximum debugging and features
        config.enable_boot_menu = true;
        config.enable_device_detection = true;
        config.enable_config_parsing = true;
        config.default_timeout = 60; // Very long timeout for debugging
        
        // Debug-specific configuration files
        config.config_file_paths = vec![
            "/boot/multios/debug.cfg",
            "/boot/multios/dev.cfg",
            "/boot/multios/boot.cfg",
        ];
        
        info!("Development debug configuration: timeout={}s, full features", config.default_timeout);
        config
    }
}

/// Example boot menu entries demonstrating all features
fn create_example_boot_menu_entries() -> Vec<BootMenuEntry> {
    info!("Creating example boot menu entries");
    
    let mut entries = Vec::new();
    
    // Normal MultiOS boot
    entries.push(BootMenuEntry::new(
        1,
        "MultiOS Normal",
        "Standard MultiOS installation with default settings",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/kernel",
            initrd_path: Some("/boot/multios/initrd.img"),
            command_line: Some("quiet loglevel=3 console=ttyS0"),
            memory_test: false,
            serial_console: true,
        },
        true, // Default
        false,
    ));
    
    // Debug MultiOS boot
    entries.push(BootMenuEntry::new(
        2,
        "MultiOS Debug",
        "MultiOS with detailed debug output and logging",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/kernel",
            initrd_path: Some("/boot/multios/debug-initrd.img"),
            command_line: Some("debug loglevel=8 console=ttyS0 maxcpus=1"),
            memory_test: false,
            serial_console: true,
        },
        false,
        false,
    ));
    
    // Recovery mode
    entries.push(BootMenuEntry::new(
        3,
        "MultiOS Recovery",
        "Recovery mode for system repair and maintenance",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/recovery-kernel",
            initrd_path: Some("/boot/multios/recovery-initrd.img"),
            command_line: Some("init=/bin/bash single rescue"),
            memory_test: true,
            serial_console: true,
        },
        false,
        true, // Recovery mode
    ));
    
    // Safe mode
    entries.push(BootMenuEntry::new(
        4,
        "MultiOS Safe Mode",
        "Safe mode with minimal drivers and services",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/kernel",
            initrd_path: None,
            command_line: Some("safe_mode no_drivers no_services console=ttyS0"),
            memory_test: false,
            serial_console: true,
        },
        false,
        false,
    ));
    
    // Memory test
    entries.push(BootMenuEntry::new(
        5,
        "Memory Test",
        "Comprehensive memory testing (MemTest86 compatible)",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/memtest",
            initrd_path: None,
            command_line: Some("memtest verbose"),
            memory_test: true,
            serial_console: true,
        },
        false,
        false,
    ));
    
    // Educational demo mode
    entries.push(BootMenuEntry::new(
        6,
        "Educational Demo",
        "MultiOS educational demo with interactive features",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/demo-kernel",
            initrd_path: Some("/boot/multios/demo-initrd.img"),
            command_line: Some("demo_mode interactive loglevel=6 console=ttyAMA0"),
            memory_test: false,
            serial_console: true,
        },
        false,
        false,
    ));
    
    info!("Created {} boot menu entries", entries.len());
    entries
}

/// Example configuration parser demonstration
fn demonstrate_config_parsing() -> Result<(), ConfigParseError> {
    info!("Demonstrating configuration file parsing");
    
    // Example GRUB2-style configuration
    let grub2_config = r#"# MultiOS Boot Configuration
# Educational Lab Configuration

timeout=30
default=multios-normal

title MultiOS Educational Normal
  linux /boot/multios/kernel
  initrd /boot/multios/initrd.img
  options quiet loglevel=3 console=ttyS0
  serial_console

title MultiOS Educational Debug
  linux /boot/multios/kernel
  initrd /boot/multios/debug-initrd.img  
  options debug loglevel=8 console=ttyS0
  serial_console
  debug_mode

title MultiOS Recovery Mode
  linux /boot/multios/recovery-kernel
  initrd /boot/multios/recovery-initrd.img
  options init=/bin/bash single rescue
  recovery_mode
  memory_test

title MultiOS Safe Mode
  linux /boot/multios/kernel
  options safe_mode no_drivers no_services
  timeout 5

title Memory Test Suite
  linux /boot/multios/memtest
  options memtest verbose all_cpus
  timeout 0

title Firmware Setup
  linux /boot/multios/kernel
  options firmware_setup
  timeout 0
"#;
    
    // Parse the configuration
    let parsed_config = config_parser::parse_config_content(grub2_config, ConfigFormat::Grub2)?;
    
    info!("Successfully parsed configuration:");
    info!("  Timeout: {} seconds", parsed_config.timeout);
    info!("  Default entry: {:?}", parsed_config.default_entry);
    info!("  Total entries: {}", parsed_config.entries.len());
    
    // Display parsed entries
    for (index, entry) in parsed_config.entries.iter().enumerate() {
        debug!("Entry {}: {}", index + 1, entry.title);
        if let Some(ref linux) = entry.linux {
            debug!("  Kernel: {}", linux);
        }
        if !entry.options.is_empty() {
            debug!("  Options: {}", entry.options.join(" "));
        }
        if entry.serial_console {
            debug!("  Serial console enabled");
        }
        if entry.debug_mode {
            debug!("  Debug mode enabled");
        }
        if entry.recovery_mode {
            debug!("  Recovery mode enabled");
        }
    }
    
    // Convert to boot menu entries
    let menu_entries = parsed_config.to_boot_menu_entries()?;
    info!("Converted to {} boot menu entries", menu_entries.len());
    
    Ok(())
}

/// Example boot device detection demonstration
fn demonstrate_device_detection() -> Result<(), BootStageError> {
    info!("Demonstrating boot device detection");
    
    // Initialize device detection for current architecture
    let architecture = BootArchitecture::current();
    info!("Current architecture: {:?}", architecture);
    
    let boot_mode = BootMode::UEFI;
    let mut context = device_detection::init_device_detection(boot_mode)?;
    
    info!("Device detection completed successfully");
    info!("Total devices detected: {}", context.devices().len());
    
    // Display all detected devices
    for device in context.devices() {
        info!("Device: {}", device);
        info!("  Type: {:?}", device.device_type);
        info!("  Path: {}", device.device_path);
        info!("  Bootable: {}", device.is_bootable);
        info!("  Removable: {}", device.is_removable);
        info!("  Priority: {}", device.priority);
    }
    
    // Display bootable devices sorted by priority
    let bootable_devices = context.bootable_devices();
    info!("\nBootable devices (by priority):");
    for device in bootable_devices {
        info!("  [P{}] {} - {}", device.priority, device.device_type, device.description);
    }
    
    // Test device availability checking
    for device in &bootable_devices {
        match device_detection::check_device_availability(device.device_path) {
            Ok(available) => {
                info!("Device {} availability: {}", device.device_path, if available { "Available" } else { "Unavailable" });
            }
            Err(_) => {
                warn!("Failed to check availability for device: {}", device.device_path);
            }
        }
    }
    
    Ok(())
}

/// Example multi-stage boot execution
fn demonstrate_multi_stage_boot() -> Result<(), BootStageError> {
    info!("Demonstrating multi-stage boot process");
    
    // Set up multi-stage boot configuration
    let config = BootExampleScenarios::educational_lab_scenario();
    
    // Initialize multi-stage boot context
    let mut context = MultiStageBootContext::new(config);
    
    info!("Starting multi-stage boot demonstration");
    
    // Stage 2: Bootloader initialization
    info!("\n--- Stage 2: Bootloader Initialization ---");
    multi_stage_boot::initialize_bootloader(&mut context)?;
    
    // Stage 3: Device detection and configuration parsing
    info!("\n--- Stage 3: Device Detection & Configuration ---");
    multi_stage_boot::detect_devices_and_parse_config(&mut context)?;
    
    // Stage 4: Boot menu display and selection
    info!("\n--- Stage 4: Boot Menu & Selection ---");
    // For demo purposes, we'll skip the actual menu display
    info!("Boot menu demonstration skipped in example");
    
    // Stage 5: Kernel loading
    info!("\n--- Stage 5: Kernel Loading ---");
    // Create a dummy boot entry for demonstration
    let demo_entry = BootMenuEntry::new(
        1,
        "Demo Kernel",
        "Demonstration kernel entry",
        BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/demo-kernel",
            initrd_path: Some("/boot/multios/demo-initrd.img"),
            command_line: Some("demo_mode debug"),
            memory_test: false,
            serial_console: true,
        },
        true,
        false,
    );
    context.selected_boot_entry = Some(demo_entry);
    
    multi_stage_boot::load_kernel(&mut context)?;
    
    // Display boot stage execution log
    info!("\n=== Boot Stage Execution Log ===");
    for log_entry in &context.boot_log {
        info!("{}", log_entry);
    }
    
    // Note: We don't actually perform Stage 6 (kernel handoff) in the example
    // as it would require a real kernel to jump to
    
    info!("Multi-stage boot demonstration completed successfully");
    Ok(())
}

/// Example error handling and recovery
fn demonstrate_error_handling() -> Result<(), BootStageError> {
    info!("Demonstrating error handling and recovery mechanisms");
    
    // Test configuration parsing errors
    info!("\n--- Testing Configuration Parsing Errors ---");
    let invalid_config = "invalid config format";
    match config_parser::parse_config_content(invalid_config, ConfigFormat::Grub2) {
        Ok(_) => warn!("Unexpected success parsing invalid configuration"),
        Err(error) => info!("Correctly caught configuration parsing error: {:?}", error),
    }
    
    // Test device detection errors
    info!("\n--- Testing Device Detection with Unknown Architecture ---");
    let mut context = MultiStageBootContext::new(MultiStageBootConfig::default());
    
    // Simulate an error condition
    context.log_stage(BootStage::Stage3, false, "Simulated device detection failure", Some(BootStageError::DeviceDetectionFailed));
    
    // Test recovery mechanisms
    info!("\n--- Testing Recovery Mechanisms ---");
    if let Some(error) = context.get_last_error() {
        info!("Last error detected: {:?}", error);
        
        // Attempt recovery
        match multi_stage_boot::detect_devices_and_parse_config(&mut context) {
            Ok(_) => info!("Recovery successful"),
            Err(recovery_error) => {
                warn!("Recovery failed: {:?}", recovery_error);
                // Continue with default configuration
                info!("Falling back to default configuration");
                context.log_stage(BootStage::Stage3, true, "Using default configuration", None);
            }
        }
    }
    
    Ok(())
}

/// Main demonstration function
#[no_mangle]
pub extern "C" fn demo_multistage_boot() {
    info!("=== Multi-Stage Boot System Demonstration ===");
    info!("Starting comprehensive demonstration of all features");
    
    // Run all demonstrations
    if let Err(error) = demonstrate_config_parsing() {
        error!("Configuration parsing demonstration failed: {:?}", error);
    }
    
    if let Err(error) = demonstrate_device_detection() {
        error!("Device detection demonstration failed: {:?}", error);
    }
    
    if let Err(error) = demonstrate_multi_stage_boot() {
        error!("Multi-stage boot demonstration failed: {:?}", error);
    }
    
    if let Err(error) = demonstrate_error_handling() {
        error!("Error handling demonstration failed: {:?}", error);
    }
    
    // Demonstrate different boot scenarios
    info!("\n=== Boot Scenario Configurations ===");
    
    let scenarios = [
        ("Educational Lab", BootExampleScenarios::educational_lab_scenario()),
        ("Production Server", BootExampleScenarios::production_server_scenario()),
        ("Embedded System", BootExampleScenarios::embedded_system_scenario()),
        ("Development Debug", BootExampleScenarios::development_debug_scenario()),
    ];
    
    for (name, config) in scenarios {
        info!("\nScenario: {}", name);
        info!("  Boot menu enabled: {}", config.enable_boot_menu);
        info!("  Device detection enabled: {}", config.enable_device_detection);
        info!("  Config parsing enabled: {}", config.enable_config_parsing);
        info!("  Timeout: {} seconds", config.default_timeout);
        info!("  Config file paths: {:?}", config.config_file_paths);
        info!("  Boot device paths: {:?}", config.boot_device_paths);
    }
    
    info!("\n=== Multi-Stage Boot Demonstration Complete ===");
    info!("All features demonstrated successfully");
}

/// Boot menu selection example
fn demonstrate_boot_menu() {
    info!("=== Boot Menu Demonstration ===");
    
    // Create example boot menu entries
    let entries = create_example_boot_menu_entries();
    
    // Display the menu
    info!("Available boot options:");
    for entry in &entries {
        info!("{}", entry);
    }
    
    // Demonstrate selection
    info!("\nSimulating user selection...");
    let selected_entry = &entries[0]; // Select first entry
    info!("User selected: {}", selected_entry.label);
    info!("Kernel path: {}", selected_entry.config().kernel_path);
    if let Some(command_line) = selected_entry.config().command_line {
        info!("Command line: {}", command_line);
    }
    
    info!("Boot menu demonstration complete");
}

/// Entry point for the example
#[no_mangle]
pub extern "C" fn run_boot_example() -> ! {
    // Initialize logging
    log::set_max_level(log::LevelFilter::Debug);
    
    // Run the demonstration
    demo_multistage_boot();
    
    // Demonstrate boot menu
    demonstrate_boot_menu();
    
    // Halt the system (this is just an example)
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}