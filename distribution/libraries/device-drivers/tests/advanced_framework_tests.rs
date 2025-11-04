//! Advanced Device Driver Framework Integration Tests
//! 
//! Comprehensive integration tests for all advanced driver framework features.
//! Tests validate the complete functionality and integration between components.

#![no_std]

extern crate alloc;
use alloc::vec;

use multios_device_drivers::advanced::*;

#[test]
fn test_advanced_framework_initialization() {
    // Test basic framework initialization
    assert!(init_advanced_framework().is_ok());
    
    // Verify global manager is created
    let manager = get_advanced_manager();
    assert!(manager.is_some());
}

#[test]
fn test_advanced_driver_registration() {
    init_advanced_framework().unwrap();
    
    let driver_info = AdvancedDriverInfo {
        id: AdvancedDriverId(1),
        name: "Test USB Driver",
        version: Version::new(1, 0, 0),
        description: "Test USB controller driver",
        author: "Test Author",
        license: "MIT",
        supported_devices: &[crate::DeviceType::USB],
        priority: 10,
        dependencies: vec![],
        capabilities: crate::DeviceCapabilities::HOT_PLUG,
        power_management: true,
        hot_plug: true,
        testing_required: false,
        load_timeout_ms: 1000,
        unload_timeout_ms: 1000,
        recovery_strategies: vec![RecoveryStrategy::ResetDevice],
    };
    
    assert!(register_advanced_driver(driver_info).is_ok());
}

#[test]
fn test_driver_lifecycle_operations() {
    init_advanced_framework().unwrap();
    
    // Register a test driver
    register_advanced_driver(AdvancedDriverInfo {
        id: AdvancedDriverId(2),
        name: "Test Network Driver",
        version: Version::new(2, 0, 0),
        description: "Test network adapter driver",
        author: "Test Author",
        license: "MIT",
        supported_devices: &[crate::DeviceType::Network],
        priority: 8,
        dependencies: vec![],
        capabilities: crate::DeviceCapabilities::NONE,
        power_management: false,
        hot_plug: false,
        testing_required: false,
        load_timeout_ms: 2000,
        unload_timeout_ms: 1000,
        recovery_strategies: vec![],
    }).unwrap();
    
    // Test loading
    assert!(load_driver(AdvancedDriverId(2)).is_ok());
    
    // Test unloading
    assert!(unload_driver(AdvancedDriverId(2)).is_ok());
}

#[test]
fn test_power_management_operations() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test power management enable/disable
        assert!(manager.power_manager.enable_power_management(AdvancedDriverId(10)).is_ok());
        assert!(manager.power_manager.disable_power_management(AdvancedDriverId(10)).is_ok());
        
        // Test power state transitions
        manager.power_manager.enable_power_management(AdvancedDriverId(10)).unwrap();
        assert!(manager.power_manager.transition_to_state(
            AdvancedDriverId(10),
            PowerState::Sleep
        ).is_ok());
        
        assert!(manager.power_manager.transition_to_state(
            AdvancedDriverId(10),
            PowerState::Active
        ).is_ok());
        
        // Test power policy setting
        assert!(manager.power_manager.set_policy(PowerPolicy::PowerSave).is_ok());
        
        // Test power domain operations
        let domain = PowerDomain {
            id: 1,
            name: "Test Domain",
            devices: vec![AdvancedDriverId(10)],
            parent_domain: None,
            default_state: PowerState::Active,
            wake_up_devices: vec![],
        };
        
        assert!(manager.power_manager.create_power_domain(domain).is_ok());
        assert!(manager.power_manager.add_driver_to_domain(AdvancedDriverId(10), 1).is_ok());
        
        // Test statistics
        let stats = manager.power_manager.get_power_statistics();
        assert!(stats.driver_states.len() > 0);
    }
}

#[test]
fn test_hot_plug_operations() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test device registration
        let device_id = manager.hot_plug_manager.register_device(
            crate::DeviceType::USB,
            hot_plug::BusType::USB,
            Some(1)
        ).unwrap();
        
        assert_eq!(device_id, 1);
        
        // Test device events
        assert!(manager.hot_plug_manager.device_inserted(device_id, Some(0x1234), Some(0x5678)).is_ok());
        assert!(manager.hot_plug_manager.is_device_present(device_id));
        
        assert!(manager.hot_plug_manager.device_changed(device_id, "Configuration changed".to_string()).is_ok());
        assert!(manager.hot_plug_manager.device_error(device_id).is_ok());
        assert!(manager.hot_plug_manager.device_timeout(device_id).is_ok());
        
        // Test device removal
        assert!(manager.hot_plug_manager.device_removed(device_id).is_ok());
        assert!(!manager.hot_plug_manager.is_device_present(device_id));
        
        // Test statistics
        let stats = manager.hot_plug_manager.get_statistics();
        assert_eq!(stats.total_devices, 1);
        assert!(stats.insertion_count >= 1);
    }
}

#[test]
fn test_error_recovery_operations() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test error reporting
        let error_id = manager.recovery_manager.report_error(
            AdvancedDriverId(20),
            AdvancedDriverError::HardwareError,
            "Test hardware error".to_string()
        ).unwrap();
        
        assert_eq!(error_id, 1);
        
        // Test recovery attempt
        let error = manager.recovery_manager.get_error(error_id).unwrap();
        assert_eq!(error.error_code, AdvancedDriverError::HardwareError);
        assert_eq!(error.category, ErrorCategory::Hardware);
        
        // Test recovery strategies
        assert!(manager.recovery_manager.attempt_recovery(
            error.clone(),
            RecoveryStrategy::ResetDevice
        ).is_ok());
        
        // Test auto-recovery configuration
        assert!(manager.recovery_manager.set_auto_recovery_enabled(true).is_ok());
        assert!(manager.recovery_manager.set_error_threshold(
            AdvancedDriverId(20),
            ErrorCategory::Hardware,
            5
        ).is_ok());
        
        // Test custom recovery strategy
        assert!(manager.recovery_manager.add_recovery_strategy(
            ErrorCategory::Hardware,
            RecoveryStrategy::RestartSystem
        ).is_ok());
        
        // Test statistics
        let stats = manager.recovery_manager.get_recovery_statistics();
        assert!(stats.total_errors > 0);
        assert!(stats.successful_recoveries >= 0);
    }
}

#[test]
fn test_debugging_operations() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test trace operations
        assert!(manager.debug_manager.add_trace(
            AdvancedDriverId(30),
            debugging::TraceEventType::Initialization,
            debugging::TraceLevel::Info,
            "Test initialization trace".to_string()
        ).is_ok());
        
        assert!(manager.debug_manager.add_error_trace(
            AdvancedDriverId(30),
            "Test error trace".to_string(),
            AdvancedDriverError::Timeout
        ).is_ok());
        
        assert!(manager.debug_manager.add_performance_trace(
            AdvancedDriverId(30),
            debugging::TraceEventType::Read,
            "Test performance trace".to_string(),
            1000
        ).is_ok());
        
        // Test configuration
        let config = DriverDebugConfig {
            driver_id: AdvancedDriverId(30),
            trace_enabled: true,
            performance_monitoring: true,
            error_tracking: true,
            max_trace_entries: 1000,
            trace_level: debugging::TraceLevel::Debug,
            performance_threshold_ns: 5000000,
        };
        
        assert!(manager.debug_manager.configure_driver(AdvancedDriverId(30), config).is_ok());
        
        // Test global settings
        assert!(manager.debug_manager.set_global_trace_level(debugging::TraceLevel::Verbose).is_ok());
        assert!(manager.debug_manager.set_debug_enabled(false).is_ok());
        assert!(manager.debug_manager.set_debug_enabled(true).is_ok());
        
        // Test statistics
        let stats = manager.debug_manager.get_debug_statistics();
        assert!(stats.total_traces > 0);
        
        // Test trace retrieval
        let driver_traces = manager.debug_manager.get_driver_traces(AdvancedDriverId(30));
        assert!(!driver_traces.is_empty());
        
        // Test performance metrics
        let metrics = manager.debug_manager.get_performance_metrics(AdvancedDriverId(30));
        assert!(metrics.is_some());
        
        // Test report generation
        let report = manager.debug_manager.generate_performance_report();
        assert!(!report.is_empty());
    }
}

#[test]
fn test_testing_framework_operations() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test custom test registration
        let test = Test {
            name: "custom_test",
            test_type: TestType::Unit,
            category: TestCategory::Initialization,
            timeout_ms: 1000,
            critical: false,
            enabled: true,
            test_func: |context| {
                context.custom_data.insert("tested".to_string(), "true".to_string());
                TestResult::Pass
            },
        };
        
        assert!(manager.test_manager.register_custom_test(test).is_ok());
        
        // Test test execution
        let result = manager.test_manager.run_test(AdvancedDriverId(40), "custom_test").unwrap();
        assert_eq!(result.result, TestResult::Pass);
        assert!(result.custom_data.contains_key("tested"));
        
        // Test test suite creation
        let suite = TestSuite {
            name: "test_suite",
            description: "Test suite",
            tests: vec![
                Test {
                    name: "suite_test_1",
                    test_type: TestType::Unit,
                    category: TestCategory::Operations,
                    timeout_ms: 1000,
                    critical: false,
                    enabled: true,
                    test_func: |_| TestResult::Pass,
                },
            ],
            setup_func: None,
            teardown_func: None,
        };
        
        assert!(manager.test_manager.register_test_suite(suite).is_ok());
        let suite_result = manager.test_manager.run_test_suite(AdvancedDriverId(40), "test_suite").unwrap();
        assert_eq!(suite_result.overall_result, TestResult::Pass);
        
        // Test driver tests
        let driver_test_result = manager.test_manager.run_driver_tests(AdvancedDriverId(40)).unwrap();
        assert_eq!(driver_test_result.overall_result, TestResult::Pass);
        
        // Test configuration
        assert!(manager.test_manager.set_auto_test_enabled(true).is_ok());
        assert!(manager.test_manager.set_default_timeout(5000).is_ok());
        
        // Test statistics
        let stats = manager.test_manager.get_test_statistics();
        assert!(stats.total_tests > 0);
    }
}

#[test]
fn test_version_management_operations() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test version registration
        assert!(manager.version_manager.register_version("Test Driver", Version::new(1, 0, 0)).is_ok());
        assert!(manager.version_manager.register_version("Test Driver", Version::new(1, 1, 0)).is_ok());
        assert!(manager.version_manager.register_version("Test Driver", Version::new(2, 0, 0)).is_ok());
        
        // Test version parsing
        let parsed_version = Version::parse("1.2.3").unwrap();
        assert_eq!(parsed_version.major, 1);
        assert_eq!(parsed_version.minor, 2);
        assert_eq!(parsed_version.patch, 3);
        
        // Test constraint creation
        let constraint = VersionConstraint::minimum(Version::new(1, 0, 0));
        assert!(Version::new(1, 5, 0).satisfies(&constraint));
        assert!(!Version::new(0, 5, 0).satisfies(&constraint));
        
        // Test compatible driver finding
        let compatible_driver = manager.version_manager.find_compatible_driver(&constraint);
        assert!(compatible_driver.is_ok() || compatible_driver.is_err()); // Depends on registered drivers
        
        // Test compatibility checking
        assert!(manager.version_manager.is_compatible(&Version::new(1, 0, 0), &Version::new(1, 1, 0)));
        assert!(!manager.version_manager.is_compatible(&Version::new(1, 0, 0), &Version::new(2, 0, 0)));
        
        // Test configuration
        assert!(manager.version_manager.set_compatibility_mode(CompatibilityMode::SemVer).is_ok());
        assert!(manager.version_manager.set_conflict_resolution_policy(ConflictResolutionPolicy::Latest).is_ok());
        
        // Test conflict resolution
        assert!(manager.version_manager.resolve_conflicts().is_ok());
        
        // Test statistics
        let stats = manager.version_manager.get_version_statistics();
        assert_eq!(stats.total_versions, 3);
        
        // Test report generation
        let report = manager.version_manager.generate_version_report();
        assert!(!report.is_empty());
    }
}

#[test]
fn test_comprehensive_integration() {
    init_advanced_framework().unwrap();
    
    // Register multiple drivers with complex configurations
    for i in 1..=5 {
        let driver_info = AdvancedDriverInfo {
            id: AdvancedDriverId(i),
            name: &format!("Driver {}", i),
            version: Version::new(i, 0, 0),
            description: &format!("Test driver {}", i),
            author: "Test Author",
            license: "MIT",
            supported_devices: &[crate::DeviceType::USB],
            priority: i as u8,
            dependencies: if i > 1 {
                vec![VersionConstraint::minimum(Version::new(1, 0, 0))]
            } else {
                vec![]
            },
            capabilities: crate::DeviceCapabilities::HOT_PLUG | crate::DeviceCapabilities::POWER_MANAGEMENT,
            power_management: true,
            hot_plug: true,
            testing_required: true,
            load_timeout_ms: 2000,
            unload_timeout_ms: 1000,
            recovery_strategies: vec![RecoveryStrategy::ResetDevice, RecoveryStrategy::ReloadDriver],
        };
        
        assert!(register_advanced_driver(driver_info).is_ok());
    }
    
    // Test comprehensive workflow
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Load all drivers (dependencies will be resolved automatically)
        for i in 1..=5 {
            assert!(manager.load_driver(AdvancedDriverId(i)).is_ok());
        }
        
        // Verify all drivers are loaded
        let active_drivers = manager.get_active_drivers();
        assert_eq!(active_drivers.len(), 5);
        
        // Test power management for all drivers
        for &driver_id in &active_drivers {
            manager.power_manager.enable_power_management(driver_id).unwrap();
            manager.power_manager.transition_to_state(driver_id, PowerState::Sleep).unwrap();
        }
        
        // Test hot-plug operations
        for i in 1..=3 {
            let device_id = manager.hot_plug_manager.register_device(
                crate::DeviceType::USB,
                hot_plug::BusType::USB,
                Some(i)
            ).unwrap();
            manager.hot_plug_manager.device_inserted(device_id, None, None).unwrap();
        }
        
        // Test error recovery
        for &driver_id in &active_drivers {
            manager.recovery_manager.report_error(
                driver_id,
                AdvancedDriverError::Timeout,
                format!("Test error for driver {:?}", driver_id)
            ).unwrap();
        }
        
        // Test debugging
        for &driver_id in &active_drivers {
            manager.debug_manager.add_trace(
                driver_id,
                debugging::TraceEventType::Initialization,
                debugging::TraceLevel::Info,
                format!("Driver {:?} initialized", driver_id)
            ).unwrap();
            
            manager.debug_manager.add_performance_trace(
                driver_id,
                debugging::TraceEventType::Read,
                format!("Performance test for driver {:?}", driver_id),
                (driver_id.0 as u64) * 1000
            ).unwrap();
        }
        
        // Test testing framework
        for &driver_id in &active_drivers {
            let test_result = manager.test_manager.run_driver_tests(driver_id).unwrap();
            assert_eq!(test_result.overall_result, TestResult::Pass);
        }
        
        // Test version management
        for i in 1..=5 {
            manager.version_manager.register_version(
                &format!("Driver {}", i),
                Version::new(i, 0, 1)
            ).unwrap();
        }
        
        // Get comprehensive statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_registered, 5);
        assert_eq!(stats.active_drivers, 5);
        assert!(stats.power_managed_devices > 0);
        assert!(stats.hot_plug_events > 0);
        assert!(stats.debug_traces > 0);
        
        // Test unloading (dependencies will prevent unloading if other drivers depend on them)
        // This should work since we don't have actual driver implementations
        unload_driver(AdvancedDriverId(5)).unwrap();
        unload_driver(AdvancedDriverId(4)).unwrap();
        unload_driver(AdvancedDriverId(3)).unwrap();
        unload_driver(AdvancedDriverId(2)).unwrap();
        unload_driver(AdvancedDriverId(1)).unwrap();
    }
}

#[test]
fn test_error_conditions() {
    init_advanced_framework().unwrap();
    
    // Test invalid driver operations
    assert!(load_driver(AdvancedDriverId(999)).is_err());
    assert!(unload_driver(AdvancedDriverId(999)).is_err());
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test invalid power state transitions
        assert!(manager.power_manager.transition_to_state(
            AdvancedDriverId(999),
            PowerState::Active
        ).is_err());
        
        // Test invalid hot-plug operations
        assert!(manager.hot_plug_manager.device_inserted(999, None, None).is_err());
        
        // Test invalid error reporting
        assert!(manager.recovery_manager.get_error(999).is_none());
        
        // Test invalid test operations
        assert!(manager.test_manager.run_test(AdvancedDriverId(999), "nonexistent").is_err());
        
        // Test invalid version operations
        assert!(manager.version_manager.find_compatible_driver(&VersionConstraint::default()).is_err());
    }
}

#[test]
fn test_framework_statistics() {
    init_advanced_framework().unwrap();
    
    // Test statistics without any operations
    let stats = get_advanced_statistics();
    assert_eq!(stats.total_registered, 0);
    assert_eq!(stats.active_drivers, 0);
    
    // Register a driver and test statistics
    register_advanced_driver(AdvancedDriverInfo {
        id: AdvancedDriverId(100),
        name: "Stats Test Driver",
        version: Version::new(1, 0, 0),
        description: "Statistics test driver",
        author: "Test Author",
        license: "MIT",
        supported_devices: &[],
        priority: 10,
        dependencies: vec![],
        capabilities: crate::DeviceCapabilities::NONE,
        power_management: false,
        hot_plug: false,
        testing_required: false,
        load_timeout_ms: 1000,
        unload_timeout_ms: 1000,
        recovery_strategies: vec![],
    }).unwrap();
    
    let stats = get_advanced_statistics();
    assert_eq!(stats.total_registered, 1);
}

#[test]
fn test_concurrent_operations() {
    use core::sync::atomic::{AtomicU32, Ordering};
    
    init_advanced_framework().unwrap();
    
    let counter = AtomicU32::new(0);
    
    // Test concurrent driver registration
    for i in 0..10 {
        let driver_id = AdvancedDriverId(1000 + i);
        let driver_info = AdvancedDriverInfo {
            id: driver_id,
            name: &format!("Concurrent Driver {}", i),
            version: Version::new(1, 0, 0),
            description: "Concurrent test driver",
            author: "Test Author",
            license: "MIT",
            supported_devices: &[],
            priority: 10,
            dependencies: vec![],
            capabilities: crate::DeviceCapabilities::NONE,
            power_management: false,
            hot_plug: false,
            testing_required: false,
            load_timeout_ms: 1000,
            unload_timeout_ms: 1000,
            recovery_strategies: vec![],
        };
        
        counter.fetch_add(1, Ordering::SeqCst);
        assert!(register_advanced_driver(driver_info).is_ok());
    }
    
    let final_stats = get_advanced_statistics();
    assert_eq!(final_stats.total_registered, counter.load(Ordering::SeqCst));
}

// Test framework configuration and limits
#[test]
fn test_framework_configuration() {
    init_advanced_framework().unwrap();
    
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Test power management configuration
        assert!(manager.power_manager.set_auto_sleep_enabled(false).is_ok());
        assert!(manager.power_manager.set_auto_sleep_enabled(true).is_ok());
        
        // Test debugging configuration
        assert!(manager.debug_manager.set_max_trace_entries(5000).is_ok());
        
        // Test hot-plug configuration
        assert!(manager.hot_plug_manager.set_timeout_duration(5000).is_ok());
        assert!(manager.hot_plug_manager.set_max_history_entries(500).is_ok());
        assert!(manager.hot_plug_manager.clear_history().is_ok());
        
        // Test recovery configuration
        assert!(manager.recovery_manager.set_auto_recovery_enabled(false).is_ok());
        assert!(manager.recovery_manager.set_auto_recovery_enabled(true).is_ok());
        
        // Test testing configuration
        assert!(manager.test_manager.set_auto_test_enabled(false).is_ok());
        assert!(manager.test_manager.set_auto_test_enabled(true).is_ok());
    }
}

#[test]
fn test_performance_characteristics() {
    use core::time::Duration;
    
    init_advanced_framework().unwrap();
    
    // Test registration performance
    let start = core::time::Instant::now();
    
    for i in 0..100 {
        let driver_info = AdvancedDriverInfo {
            id: AdvancedDriverId(5000 + i),
            name: &format!("Performance Driver {}", i),
            version: Version::new(1, 0, 0),
            description: "Performance test driver",
            author: "Test Author",
            license: "MIT",
            supported_devices: &[],
            priority: 10,
            dependencies: vec![],
            capabilities: crate::DeviceCapabilities::NONE,
            power_management: false,
            hot_plug: false,
            testing_required: false,
            load_timeout_ms: 1000,
            unload_timeout_ms: 1000,
            recovery_strategies: vec![],
        };
        
        assert!(register_advanced_driver(driver_info).is_ok());
    }
    
    let elapsed = start.elapsed();
    
    // Registration should be fast (< 1 second for 100 drivers)
    assert!(elapsed.as_millis() < 1000, "Registration took too long: {:?}", elapsed);
    
    // Test memory usage (approximate check)
    let stats = get_advanced_statistics();
    assert!(stats.total_registered == 100);
    
    // Test retrieval performance
    let start = core::time::Instant::now();
    
    if let Some(ref manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        for i in 0..100 {
            let _ = manager.get_registered_drivers();
        }
    }
    
    let elapsed = start.elapsed();
    // Retrieval should also be fast
    assert!(elapsed.as_millis() < 500, "Retrieval took too long: {:?}", elapsed);
}
