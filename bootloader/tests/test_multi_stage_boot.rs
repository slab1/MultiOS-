//! Multi-Stage Boot Test Suite
//! 
//! Comprehensive tests for the multi-stage boot system including
//! device detection, configuration parsing, boot menu, and boot stages.

#![cfg(test)]

extern crate bootloader;
extern crate spin;
extern crate log;

use bootloader::boot_config::{BootConfig, BootMode};
use bootloader::boot_menu::{BootMenuEntry, BootMenuConfig, BootMenuSelection};
use bootloader::device_detection::{BootDevice, BootDeviceType, BootArchitecture, BootDeviceContext};
use bootloader::config_parser::{ConfigEntry, ParsedBootConfig, ConfigFormat, ConfigParseError};
use bootloader::multi_stage_boot::{MultiStageBootConfig, MultiStageBootContext, BootStage, BootStageError};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test boot menu entry creation and functionality
    #[test]
    fn test_boot_menu_entry_creation() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: Some("/boot/initrd"),
            command_line: Some("debug loglevel=8"),
            memory_test: false,
            serial_console: true,
        };

        let entry = BootMenuEntry::new(
            1,
            "Test Entry",
            "Test description",
            config,
            true,
            false
        );

        assert_eq!(entry.id, 1);
        assert_eq!(entry.label, "Test Entry");
        assert_eq!(entry.description, "Test description");
        assert!(entry.is_default());
        assert!(!entry.is_recovery_mode());
        assert_eq!(entry.config().kernel_path, "/boot/kernel");
    }

    /// Test boot menu configuration
    #[test]
    fn test_boot_menu_config() {
        let config = BootMenuConfig {
            timeout_seconds: 15,
            enable_recovery_mode: true,
            enable_debug_mode: true,
            enable_normal_mode: true,
            default_boot_mode: BootMenuSelection::Debug,
        };

        assert_eq!(config.timeout_seconds, 15);
        assert!(config.enable_recovery_mode);
        assert!(config.enable_debug_mode);
        assert!(config.enable_normal_mode);
        assert_eq!(config.default_boot_mode, BootMenuSelection::Debug);
    }

    /// Test device detection for different architectures
    #[test]
    fn test_device_detection() {
        let arch = BootArchitecture::current();
        
        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch, BootArchitecture::X86_64);
        
        #[cfg(target_arch = "aarch64")]
        assert_eq!(arch, BootArchitecture::ARM64);
        
        #[cfg(target_arch = "riscv64")]
        assert_eq!(arch, BootArchitecture::RISCV64);

        // Test device type creation
        let device = BootDevice::new(
            BootDeviceType::HardDisk,
            "/dev/sda",
            "Test Hard Drive",
            true,
            false,
            1,
            vec![BootMode::UEFI],
        );

        assert_eq!(device.device_type, BootDeviceType::HardDisk);
        assert_eq!(device.device_path, "/dev/sda");
        assert!(device.is_bootable);
        assert!(!device.is_removable);
        assert_eq!(device.priority, 1);
        assert!(device.supports_mode(BootMode::UEFI));
        assert!(!device.supports_mode(BootMode::LegacyBIOS));
    }

    /// Test configuration parsing for different formats
    #[test]
    fn test_config_parsing() {
        // Test GRUB2 format
        let grub2_config = r#"
timeout=10
default=multios-normal

title MultiOS Normal
  linux /boot/multios/kernel
  options quiet loglevel=3

title MultiOS Debug
  linux /boot/multios/kernel
  options debug loglevel=8
  debug_mode
"#;

        let parsed = config_parser::parse_config_content(grub2_config, ConfigFormat::Grub2).unwrap();
        assert_eq!(parsed.timeout, 10);
        assert_eq!(parsed.default_entry, Some("multios-normal".to_string()));
        assert_eq!(parsed.entries.len(), 2);

        let normal_entry = parsed.find_entry("MultiOS Normal").unwrap();
        assert_eq!(normal_entry.linux, Some("/boot/multios/kernel".to_string()));
        assert_eq!(normal_entry.options, vec!["quiet loglevel=3"]);
        assert!(!normal_entry.debug_mode);

        let debug_entry = parsed.find_entry("MultiOS Debug").unwrap();
        assert_eq!(debug_entry.linux, Some("/boot/multios/kernel".to_string()));
        assert_eq!(debug_entry.options, vec!["debug loglevel=8"]);
        assert!(debug_entry.debug_mode);
    }

    /// Test boot parameter parsing
    #[test]
    fn test_boot_parameter_parsing() {
        let params = "debug console=ttyS0 loglevel=8 quiet single memtest";
        let parsed = config_parser::parse_boot_parameters(params);

        assert!(parsed.contains(&config_parser::BootParameter::Debug));
        assert!(parsed.contains(&config_parser::BootParameter::Console("console=ttyS0".to_string())));
        assert!(parsed.contains(&config_parser::BootParameter::Loglevel(8)));
        assert!(parsed.contains(&config_parser::BootParameter::Quiet));
        assert!(parsed.contains(&config_parser::BootParameter::Single));
        assert!(parsed.contains(&config_parser::BootParameter::Memtest));
    }

    /// Test parameters to string conversion
    #[test]
    fn test_parameters_to_string() {
        let params = vec![
            config_parser::BootParameter::Debug,
            config_parser::BootParameter::Console("console=ttyS0".to_string()),
            config_parser::BootParameter::Loglevel(8),
            config_parser::BootParameter::Quiet,
        ];

        let result = config_parser::parameters_to_string(&params);
        assert!(result.contains("debug"));
        assert!(result.contains("console=ttyS0"));
        assert!(result.contains("loglevel=8"));
        assert!(result.contains("quiet"));
    }

    /// Test multi-stage boot configuration
    #[test]
    fn test_multi_stage_boot_config() {
        let config = MultiStageBootConfig::default();
        
        assert!(config.enable_boot_menu);
        assert!(config.enable_device_detection);
        assert!(config.enable_config_parsing);
        assert_eq!(config.default_timeout, 10);
        assert!(!config.config_file_paths.is_empty());
    }

    /// Test educational lab configuration
    #[test]
    fn test_educational_lab_config() {
        let config = MultiStageBootConfig::for_educational_lab();
        
        assert!(config.enable_boot_menu);
        assert!(config.default_timeout >= 30);
    }

    /// Test embedded system configuration
    #[test]
    fn test_embedded_system_config() {
        let config = MultiStageBootConfig::for_embedded();
        
        assert!(!config.enable_boot_menu);
        assert!(!config.enable_config_parsing);
        assert!(config.default_timeout <= 5);
    }

    /// Test boot stage error handling
    #[test]
    fn test_boot_stage_errors() {
        let error1 = BootStageError::StageFailed(BootStage::Stage2);
        let error2 = BootStageError::ConfigurationError;
        let error3 = BootStageError::DeviceDetectionFailed;

        assert_ne!(error1, error2);
        assert_ne!(error2, error3);
    }

    /// Test configuration entry creation
    #[test]
    fn test_config_entry_creation() {
        let entry = ConfigEntry::new("Test Entry".to_string())
            .linux("/boot/kernel")
            .initrd("/boot/initrd")
            .options(&["quiet", "loglevel=3"])
            .serial_console(true)
            .debug_mode(true)
            .recovery_mode(false);

        assert_eq!(entry.title, "Test Entry");
        assert_eq!(entry.linux, Some("/boot/kernel".to_string()));
        assert_eq!(entry.initrd, Some("/boot/initrd".to_string()));
        assert_eq!(entry.options, vec!["quiet", "loglevel=3"]);
        assert!(entry.serial_console);
        assert!(entry.debug_mode);
        assert!(!entry.recovery_mode);
    }

    /// Test config entry to boot config conversion
    #[test]
    fn test_config_entry_to_boot_config() {
        let entry = ConfigEntry::new("Test".to_string())
            .linux("/boot/kernel")
            .options(&["quiet", "loglevel=3"])
            .serial_console(true);

        let boot_config = entry.to_boot_config().unwrap();
        assert_eq!(boot_config.kernel_path, "/boot/kernel");
        assert!(boot_config.serial_console);
        assert!(!boot_config.memory_test);
    }

    /// Test device type display formatting
    #[test]
    fn test_device_type_display() {
        assert_eq!(format!("{}", BootDeviceType::HardDisk), "Hard Disk");
        assert_eq!(format!("{}", BootDeviceType::USB), "USB Device");
        assert_eq!(format!("{}", BootDeviceType::Network), "Network Boot");
    }

    /// Test boot stage display formatting
    #[test]
    fn test_boot_stage_display() {
        assert_eq!(format!("{}", BootStage::Stage1), "Stage 1: Firmware/BIOS/UEFI");
        assert_eq!(format!("{}", BootStage::Stage6), "Stage 6: Kernel Handoff");
    }

    /// Test architecture capabilities
    #[test]
    fn test_architecture_capabilities() {
        let arch = BootArchitecture::current();
        
        // x86_64 supports both UEFI and Legacy BIOS
        #[cfg(target_arch = "x86_64")]
        {
            assert!(arch.supports_uefi());
            assert!(arch.supports_legacy_bios());
        }
        
        // ARM64 and RISC-V support UEFI but not Legacy BIOS
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            assert!(arch.supports_uefi());
            assert!(!arch.supports_legacy_bios());
        }

        // ARM64 and RISC-V require device tree
        #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
        {
            assert!(arch.requires_device_tree());
        }
    }

    /// Test device priority comparison
    #[test]
    fn test_device_priority() {
        let device1 = BootDevice::new(
            BootDeviceType::HardDisk,
            "/dev/sda",
            "Device 1",
            true,
            false,
            1,
            vec![BootMode::UEFI],
        );

        let device2 = BootDevice::new(
            BootDeviceType::USB,
            "/dev/sdb",
            "Device 2",
            true,
            false,
            3,
            vec![BootMode::UEFI],
        );

        assert!(device1.has_higher_priority_than(&device2));
        assert!(!device2.has_higher_priority_than(&device1));
    }

    /// Test device path type inference
    #[test]
    fn test_device_path_type_inference() {
        assert_eq!(
            bootloader::device_detection::get_device_type_from_path("/dev/sda"),
            Some(BootDeviceType::HardDisk)
        );
        assert_eq!(
            bootloader::device_detection::get_device_type_from_path("/dev/mmcblk0"),
            Some(BootDeviceType::SDCard)
        );
        assert_eq!(
            bootloader::device_detection::get_device_type_from_path("pxe"),
            Some(BootDeviceType::Network)
        );
        assert_eq!(
            bootloader::device_detection::get_device_type_from_path("unknown"),
            None
        );
    }

    /// Test invalid configuration parsing
    #[test]
    fn test_invalid_config_parsing() {
        let invalid_config = "invalid config format";
        let result = config_parser::parse_config_content(invalid_config, ConfigFormat::Grub2);
        
        assert!(matches!(result, Err(ConfigParseError::InvalidFormat)));
    }

    /// Test empty configuration handling
    #[test]
    fn test_empty_config_parsing() {
        let empty_config = "# Just comments";
        let result = config_parser::parse_config_content(empty_config, ConfigFormat::Grub2);
        
        assert!(matches!(result, Err(ConfigParseError::ParseError)));
    }

    /// Test boot menu entry display formatting
    #[test]
    fn test_boot_menu_entry_display() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/kernel",
            initrd_path: None,
            command_line: Some("debug"),
            memory_test: false,
            serial_console: true,
        };

        let entry = BootMenuEntry::new(
            1,
            "Test Entry",
            "Test description",
            config,
            true,
            false
        );

        let display_str = format!("{}", entry);
        assert!(display_str.contains("Test Entry"));
        assert!(display_str.contains("Test description"));
    }

    /// Test multi-stage boot context logging
    #[test]
    fn test_boot_context_logging() {
        let mut context = MultiStageBootContext::new(MultiStageBootConfig::default());
        
        // Log a successful stage
        context.log_stage(BootStage::Stage2, true, "Test stage", None);
        
        // Log a failed stage
        context.log_stage(BootStage::Stage3, false, "Test failure", Some(BootStageError::ConfigurationError));

        assert_eq!(context.boot_log.len(), 2);
        assert!(context.boot_log[0].success);
        assert!(!context.boot_log[1].success);
    }

    /// Test configuration to boot menu conversion
    #[test]
    fn test_config_to_menu_entries() {
        let config = r#"
timeout=10
default=normal

title MultiOS Normal
  linux /boot/kernel
  options quiet

title MultiOS Debug
  linux /boot/kernel
  options debug
  debug_mode
"#;

        let parsed = config_parser::parse_config_content(config, ConfigFormat::Grub2).unwrap();
        let menu_entries = parsed.to_boot_menu_entries().unwrap();
        
        assert_eq!(menu_entries.len(), 2);
        assert_eq!(menu_entries[0].id, 1);
        assert_eq!(menu_entries[1].id, 2);
        assert!(menu_entries[0].is_default());
    }

    /// Test boot mode compatibility
    #[test]
    fn test_boot_mode_compatibility() {
        let uefi_device = BootDevice::new(
            BootDeviceType::HardDisk,
            "/dev/sda",
            "UEFI Device",
            true,
            false,
            1,
            vec![BootMode::UEFI],
        );

        let legacy_device = BootDevice::new(
            BootDeviceType::HardDisk,
            "/dev/sdb",
            "Legacy Device",
            true,
            false,
            2,
            vec![BootMode::LegacyBIOS],
        );

        assert!(uefi_device.supports_mode(BootMode::UEFI));
        assert!(!uefi_device.supports_mode(BootMode::LegacyBIOS));
        assert!(legacy_device.supports_mode(BootMode::LegacyBIOS));
        assert!(!legacy_device.supports_mode(BootMode::UEFI));
    }

    /// Test custom JSON configuration parsing
    #[test]
    fn test_json_config_parsing() {
        let json_config = r#"{
  "timeout": 15,
  "default_entry": "test-entry",
  "entries": [
    {
      "title": "Test Entry",
      "linux": "/boot/kernel",
      "options": ["debug"],
      "serial_console": true
    }
  ]
}"#;

        let result = config_parser::parse_config_content(json_config, ConfigFormat::Json);
        
        // JSON parsing should work (even if specific parsing logic is simplified)
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.timeout, 15);
        assert_eq!(config.entries.len(), 1);
    }
}

/// Integration tests for multi-stage boot system
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete boot flow simulation
    #[test]
    fn test_complete_boot_flow_simulation() {
        // Simulate the complete boot flow without actually booting
        
        // 1. Initialize configuration
        let config = MultiStageBootConfig::default();
        let mut context = MultiStageBootContext::new(config);
        
        // 2. Simulate successful boot stages
        context.log_stage(BootStage::Stage2, true, "Bootloader initialized", None);
        assert_eq!(context.boot_log.len(), 1);
        assert!(context.boot_log[0].success);
        
        // 3. Test device detection simulation
        context.log_stage(BootStage::Stage3, true, "Devices detected and configured", None);
        assert_eq!(context.boot_log.len(), 2);
        
        // 4. Test menu selection simulation
        context.log_stage(BootStage::Stage4, true, "Menu selection completed", None);
        assert_eq!(context.boot_log.len(), 3);
        
        // 5. Test kernel loading simulation
        context.log_stage(BootStage::Stage5, true, "Kernel loaded", None);
        assert_eq!(context.boot_log.len(), 4);
        
        // 6. Verify all stages completed successfully
        for log_entry in &context.boot_log {
            assert!(log_entry.success);
        }
    }

    /// Test error recovery simulation
    #[test]
    fn test_error_recovery_simulation() {
        let config = MultiStageBootConfig::default();
        let mut context = MultiStageBootContext::new(config);
        
        // Simulate a failed stage
        context.log_stage(BootStage::Stage3, false, "Device detection failed", Some(BootStageError::DeviceDetectionFailed));
        assert!(!context.boot_log[0].success);
        
        // Simulate recovery
        context.log_stage(BootStage::Stage3, true, "Recovery successful", None);
        assert!(context.boot_log[1].success);
        
        // Verify recovery worked
        assert_eq!(context.boot_log.len(), 2);
        assert!(context.get_last_error().is_some());
    }

    /// Test configuration fallback handling
    #[test]
    fn test_configuration_fallback() {
        // Test that system gracefully handles missing config files
        let mut config = MultiStageBootConfig::default();
        config.config_file_paths = vec!["/nonexistent/config.cfg"];
        
        // This should not panic, but handle gracefully
        let mut context = MultiStageBootContext::new(config);
        
        // Simulate the configuration loading stage
        context.log_stage(BootStage::Stage3, true, "Using default configuration", None);
        
        assert_eq!(context.boot_log.len(), 1);
        assert!(context.boot_log[0].success);
    }

    /// Test boot menu timeout handling
    #[test]
    fn test_boot_menu_timeout_handling() {
        let menu_config = BootMenuConfig {
            timeout_seconds: 5,
            enable_recovery_mode: true,
            enable_debug_mode: true,
            enable_normal_mode: true,
            default_boot_mode: BootMenuSelection::Normal,
        };

        assert_eq!(menu_config.timeout_seconds, 5);
        
        // Test educational configuration has longer timeout
        let edu_config = BootMenuConfig {
            timeout_seconds: 30,
            enable_recovery_mode: true,
            enable_debug_mode: true,
            enable_normal_mode: true,
            default_boot_mode: BootMenuSelection::Debug,
        };

        assert_eq!(edu_config.timeout_seconds, 30);
    }

    /// Test architecture-specific boot behavior
    #[test]
    fn test_architecture_specific_behavior() {
        let arch = BootArchitecture::current();
        
        match arch {
            BootArchitecture::X86_64 => {
                // x86_64 specific tests
                assert!(arch.supports_uefi());
                assert!(arch.supports_legacy_bios());
            }
            BootArchitecture::ARM64 => {
                // ARM64 specific tests
                assert!(arch.supports_uefi());
                assert!(!arch.supports_legacy_bios());
                assert!(arch.requires_device_tree());
            }
            BootArchitecture::RISCV64 => {
                // RISC-V specific tests
                assert!(arch.supports_uefi());
                assert!(!arch.supports_legacy_bios());
                assert!(arch.requires_device_tree());
            }
            BootArchitecture::Unknown => {
                // Unknown architecture handling
                panic!("Unknown architecture detected");
            }
        }
    }

    /// Test boot parameter validation
    #[test]
    fn test_boot_parameter_validation() {
        // Test various parameter combinations
        let test_cases = vec![
            ("debug", vec![config_parser::BootParameter::Debug]),
            ("quiet", vec![config_parser::BootParameter::Quiet]),
            ("single", vec![config_parser::BootParameter::Single]),
            ("memtest", vec![config_parser::BootParameter::Memtest]),
            ("console=ttyS0", vec![config_parser::BootParameter::Console("console=ttyS0".to_string())]),
            ("loglevel=5", vec![config_parser::BootParameter::Loglevel(5)]),
        ];

        for (input, expected) in test_cases {
            let parsed = config_parser::parse_boot_parameters(input);
            assert_eq!(parsed, expected, "Failed to parse: {}", input);
        }
    }

    /// Test memory-efficient boot configuration
    #[test]
    fn test_memory_efficient_config() {
        // Test that configurations don't use excessive memory
        let config = MultiStageBootConfig::for_embedded();
        
        // Embedded config should have minimal features
        assert!(!config.enable_boot_menu);
        assert!(!config.enable_config_parsing);
        assert_eq!(config.config_file_paths.len(), 0);
        assert!(config.boot_device_paths.len() <= 2);
    }
}

/// Performance tests for boot system
#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Test configuration parsing performance
    #[test]
    fn test_config_parsing_performance() {
        let large_config = generate_large_config();
        
        let start = std::time::Instant::now();
        let result = config_parser::parse_config_content(&large_config, ConfigFormat::Grub2);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        // Parsing should complete within reasonable time
        assert!(duration.as_millis() < 100);
    }

    /// Test device detection performance
    #[test]
    fn test_device_detection_performance() {
        let start = std::time::Instant::now();
        
        // This is a simplified test - in real implementation
        // device detection would scan actual hardware
        let context = MultiStageBootContext::new(MultiStageBootConfig::default());
        
        let duration = start.elapsed();
        
        // Basic initialization should be fast
        assert!(duration.as_micros() < 1000);
    }

    /// Generate a large configuration for performance testing
    fn generate_large_config() -> String {
        let mut config = String::new();
        config.push_str("timeout=10\n");
        config.push_str("default=entry1\n\n");
        
        for i in 1..=100 {
            config.push_str(&format!(
                "title Entry {}\n  linux /boot/kernel{}\n  options quiet loglevel=3\n\n",
                i, i
            ));
        }
        
        config
    }
}

/// Run all multi-stage boot tests
#[test]
fn run_all_multistage_boot_tests() {
    println!("Running Multi-Stage Boot Test Suite...");
    
    // Run all test modules
    tests::test_boot_menu_entry_creation();
    tests::test_boot_menu_config();
    tests::test_device_detection();
    tests::test_config_parsing();
    tests::test_boot_parameter_parsing();
    tests::test_parameters_to_string();
    tests::test_multi_stage_boot_config();
    tests::test_educational_lab_config();
    tests::test_embedded_system_config();
    tests::test_boot_stage_errors();
    tests::test_config_entry_creation();
    tests::test_config_entry_to_boot_config();
    tests::test_device_type_display();
    tests::test_boot_stage_display();
    tests::test_architecture_capabilities();
    tests::test_device_priority();
    tests::test_device_path_type_inference();
    tests::test_invalid_config_parsing();
    tests::test_empty_config_parsing();
    tests::test_boot_menu_entry_display();
    tests::test_boot_context_logging();
    tests::test_config_to_menu_entries();
    tests::test_boot_mode_compatibility();
    tests::test_json_config_parsing();
    
    println!("All multi-stage boot tests passed!");
}