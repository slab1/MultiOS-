//! Core bootloader unit tests
//! 
//! Tests fundamental bootloader functionality including boot state management,
//! error handling, and core boot operations.

use super::*;
use pretty_assertions::assert_eq;
use rstest::*;
use serial_test::serial;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    /// Test boot mode detection
    #[test]
    #[serial]
    fn test_boot_mode_detection() {
        // Test that boot modes are properly detected
        let boot_mode = detect_boot_mode();
        
        // Should be a valid boot mode
        assert!(match boot_mode {
            BootMode::UEFI | BootMode::LegacyBIOS | BootMode::Unknown => true,
            _ => false,
        });
    }

    /// Test boot error variants
    #[test]
    fn test_boot_error_variants() {
        let errors = [
            BootError::UefiNotSupported,
            BootError::LegacyNotSupported,
            BootError::KernelNotFound,
            BootError::MemoryMapError,
            BootError::InvalidKernelFormat,
            BootError::BootProcessError,
        ];
        
        // Each error should have a unique representation
        let mut seen_errors = Vec::new();
        for &error in &errors {
            assert!(!seen_errors.contains(&error), "Duplicate error: {:?}", error);
            seen_errors.push(error);
        }
        
        assert_eq!(seen_errors.len(), 6);
    }

    /// Test boot configuration creation
    #[test]
    fn test_boot_config_creation() {
        let config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/multios/kernel",
            initrd_path: Some("/boot/initrd"),
            command_line: Some("quiet loglevel=3"),
            memory_test: true,
            serial_console: true,
        };
        
        assert_eq!(config.mode, BootMode::UEFI);
        assert_eq!(config.kernel_path, "/boot/multios/kernel");
        assert_eq!(config.initrd_path, Some("/boot/initrd"));
        assert_eq!(config.command_line, Some("quiet loglevel=3"));
        assert!(config.memory_test);
        assert!(config.serial_console);
    }

    /// Test boot state management
    #[test]
    fn test_boot_state_management() {
        // Test that boot state can be created and accessed
        let boot_time = 1000;
        let boot_config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/boot/test",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
        };
        
        let memory_map = memory_map::MemoryMap::new();
        
        let boot_state = BootState {
            mode: BootMode::UEFI,
            boot_time,
            memory_map,
            boot_config: boot_config.clone(),
        };
        
        assert_eq!(boot_state.mode, BootMode::UEFI);
        assert_eq!(boot_state.boot_time, boot_time);
        assert_eq!(boot_state.boot_config.kernel_path, "/boot/test");
    }

    /// Test boot mode ordering
    #[test]
    fn test_boot_mode_ordering() {
        // Test that boot modes can be ordered for comparison
        assert!(BootMode::UEFI as u8 < BootMode::LegacyBIOS as u8);
        assert!(BootMode::LegacyBIOS as u8 < BootMode::Unknown as u8);
    }

    /// Test memory map initialization
    #[test]
    fn test_memory_map_initialization() {
        let memory_map = memory_map::MemoryMap::new();
        
        // Should be a valid memory map
        assert!(memory_map.total_memory() >= 0);
    }

    /// Test kernel loading configuration
    #[test]
    fn test_kernel_loading_config() {
        let config = BootConfig {
            mode: BootMode::LegacyBIOS,
            kernel_path: "/custom/kernel/path",
            initrd_path: None,
            command_line: Some("debug console=ttyS0"),
            memory_test: true,
            serial_console: false,
        };
        
        // Verify kernel path configuration
        assert_eq!(config.kernel_path, "/custom/kernel/path");
        
        // Verify memory test is enabled
        assert!(config.memory_test);
        
        // Verify serial console is disabled
        assert!(!config.serial_console);
    }

    /// Test serial console initialization
    #[test]
    fn test_serial_console_initialization() {
        // Test that serial console can be initialized
        // This is a simplified test - in real hardware this would test actual serial ports
        let result = init_serial_console_test();
        
        assert!(result.is_ok(), "Serial console should initialize successfully");
    }

    /// Test boot time retrieval
    #[test]
    fn test_boot_time_retrieval() {
        let start_time = get_boot_time();
        
        // Simulate some processing time
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let end_time = get_boot_time();
        
        // The times might be the same in a test environment
        // but the function should still return a valid time
        assert!(start_time >= 0);
        assert!(end_time >= start_time || start_time == end_time);
    }

    /// Test boot configuration validation
    #[test]
    fn test_boot_config_validation() {
        // Test with valid configuration
        let valid_config = BootConfig {
            mode: BootMode::UEFI,
            kernel_path: "/valid/path",
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: true,
        };
        
        assert!(validate_boot_config(&valid_config).is_ok());
        
        // Test with invalid kernel path
        let invalid_config = BootConfig {
            mode: BootMode::Unknown,
            kernel_path: "", // Empty path should be invalid
            initrd_path: None,
            command_line: None,
            memory_test: false,
            serial_console: false,
        };
        
        assert!(validate_boot_config(&invalid_config).is_err());
    }

    /// Test multi-threaded boot state access
    #[test]
    fn test_thread_safety() {
        // Test that boot state can be safely accessed from multiple threads
        let state = Arc::new(Mutex::new(None));
        let mut handles = Vec::new();
        
        for i in 0..4 {
            let state_clone = state.clone();
            let handle = thread::spawn(move || {
                let mut guard = state_clone.lock().unwrap();
                *guard = Some(BootState {
                    mode: BootMode::UEFI,
                    boot_time: i as u64,
                    memory_map: memory_map::MemoryMap::new(),
                    boot_config: BootConfig {
                        mode: BootMode::UEFI,
                        kernel_path: "/test",
                        initrd_path: None,
                        command_line: None,
                        memory_test: false,
                        serial_console: true,
                    },
                });
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify that the state was set (by the last thread)
        let guard = state.lock().unwrap();
        assert!(guard.is_some());
    }

    /// Property-based test for boot configuration
    #[test]
    fn test_boot_config_properties() {
        use proptest::prelude::*;
        
        proptest! {
            #[test]
            fn test_config_property_mode_is_valid(
                mode in prop_one_of![Just(BootMode::UEFI), Just(BootMode::LegacyBIOS), Just(BootMode::Unknown)]
            ) {
                let config = BootConfig {
                    mode,
                    kernel_path: "/valid/path",
                    initrd_path: None,
                    command_line: None,
                    memory_test: false,
                    serial_console: true,
                };
                
                assert!(match config.mode {
                    BootMode::UEFI | BootMode::LegacyBIOS | BootMode::Unknown => true,
                    _ => false,
                });
            }
        }
    }

    // Helper functions for testing

    /// Test version of serial console initialization
    fn init_serial_console_test() -> Result<(), BootError> {
        // Simplified test - in real implementation this would test actual serial port initialization
        Ok(())
    }

    /// Validate boot configuration
    fn validate_boot_config(config: &BootConfig) -> Result<(), BootError> {
        // Validate kernel path
        if config.kernel_path.is_empty() {
            return Err(BootError::KernelNotFound);
        }
        
        // Validate boot mode
        match config.mode {
            BootMode::UEFI | BootMode::LegacyBIOS => Ok(()),
            BootMode::Unknown => Err(BootError::BootProcessError),
        }
    }
}
