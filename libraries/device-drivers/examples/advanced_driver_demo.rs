//! Advanced Device Driver Framework Example
//! 
//! This example demonstrates all the advanced features of the MultiOS device driver framework
//! including lifecycle management, dependencies, power management, hot-plug support,
//! error recovery, debugging, testing, and version management.

#![no_std]

extern crate alloc;
use alloc::vec;
use alloc::string::String;
use core::fmt::Write;

use multios_device_drivers::advanced::*;

/// Example driver implementation
struct ExampleDriver {
    id: AdvancedDriverId,
    name: &'static str,
    version: Version,
    loaded: bool,
    active: bool,
}

impl ExampleDriver {
    fn new(id: AdvancedDriverId, name: &'static str, version: Version) -> Self {
        Self {
            id,
            name,
            version,
            loaded: false,
            active: false,
        }
    }

    fn load(&mut self) -> AdvancedResult<()> {
        debug!("Loading example driver: {} v{}", self.name, self.version);
        self.loaded = true;
        Ok(())
    }

    fn unload(&mut self) -> AdvancedResult<()> {
        debug!("Unloading example driver: {}", self.name);
        self.loaded = false;
        self.active = false;
        Ok(())
    }

    fn activate(&mut self) -> AdvancedResult<()> {
        debug!("Activating example driver: {}", self.name);
        self.active = true;
        Ok(())
    }

    fn suspend(&mut self) -> AdvancedResult<()> {
        debug!("Suspending example driver: {}", self.name);
        self.active = false;
        Ok(())
    }
}

/// Example demonstrating advanced driver framework features
fn main() -> AdvancedResult<()> {
    println!("=== MultiOS Advanced Device Driver Framework Demo ===\n");

    // 1. Initialize the advanced framework
    init_advanced_framework()?;
    println!("✓ Advanced Driver Framework initialized\n");

    // 2. Create example drivers with different configurations
    let mut driver1 = ExampleDriver::new(
        AdvancedDriverId(1),
        "USB Controller Driver",
        Version::new(1, 2, 0)
    );

    let mut driver2 = ExampleDriver::new(
        AdvancedDriverId(2),
        "Network Adapter Driver",
        Version::new(2, 1, 1)
    );

    let mut driver3 = ExampleDriver::new(
        AdvancedDriverId(3),
        "Graphics Display Driver",
        Version::new(1, 0, 5)
    );

    // 3. Register drivers with advanced information
    register_driver(AdvancedDriverInfo {
        id: AdvancedDriverId(1),
        name: "USB Controller Driver",
        version: Version::new(1, 2, 0),
        description: "Advanced USB host controller driver",
        author: "MultiOS Team",
        license: "MIT",
        supported_devices: &[crate::DeviceType::USB],
        priority: 10,
        dependencies: vec![
            VersionConstraint::minimum(Version::new(1, 0, 0)),
        ],
        capabilities: crate::DeviceCapabilities::HOT_PLUG | crate::DeviceCapabilities::POWER_MANAGEMENT,
        power_management: true,
        hot_plug: true,
        testing_required: true,
        load_timeout_ms: 5000,
        unload_timeout_ms: 2000,
        recovery_strategies: vec![RecoveryStrategy::ResetDevice, RecoveryStrategy::ReloadDriver],
    })?;

    register_driver(AdvancedDriverInfo {
        id: AdvancedDriverId(2),
        name: "Network Adapter Driver",
        version: Version::new(2, 1, 1),
        description: "High-performance network adapter driver",
        author: "MultiOS Team",
        license: "MIT",
        supported_devices: &[crate::DeviceType::Network],
        priority: 8,
        dependencies: vec![
            VersionConstraint::minimum(Version::new(1, 0, 0)),
            VersionConstraint::maximum(Version::new(3, 0, 0)),
        ],
        capabilities: crate::DeviceCapabilities::POWER_MANAGEMENT,
        power_management: true,
        hot_plug: false,
        testing_required: true,
        load_timeout_ms: 3000,
        unload_timeout_ms: 1000,
        recovery_strategies: vec![RecoveryStrategy::ResetDevice],
    })?;

    register_driver(AdvancedDriverInfo {
        id: AdvancedDriverId(3),
        name: "Graphics Display Driver",
        version: Version::new(1, 0, 5),
        description: "Advanced graphics and display driver",
        author: "MultiOS Team",
        license: "MIT",
        supported_devices: &[crate::DeviceType::Display],
        priority: 12,
        dependencies: vec![],
        capabilities: crate::DeviceCapabilities::NONE,
        power_management: false,
        hot_plug: false,
        testing_required: false,
        load_timeout_ms: 1000,
        unload_timeout_ms: 500,
        recovery_strategies: vec![RecoveryStrategy::ReloadDriver],
    })?;

    println!("✓ Advanced drivers registered\n");

    // 4. Demonstrate lifecycle management
    println!("=== Lifecycle Management ===");
    let manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    if let Some(ref manager) = *manager_guard {
        // Load drivers with dependency resolution
        load_driver(AdvancedDriverId(1))?;
        load_driver(AdvancedDriverId(2))?;
        load_driver(AdvancedDriverId(3))?;
        
        let active_drivers = manager.get_active_drivers();
        println!("✓ Loaded {} drivers: {:?}", active_drivers.len(), active_drivers);
        
        // Show lifecycle states
        let stats = manager.get_statistics();
        println!("✓ Lifecycle states: {:?}", stats.lifecycle_states);
    }
    drop(manager_guard);
    println!();

    // 5. Demonstrate power management
    println!("=== Power Management ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Enable power management for drivers
        manager.power_manager.enable_power_management(AdvancedDriverId(1))?;
        manager.power_manager.enable_power_management(AdvancedDriverId(2))?;
        
        // Set power policy
        manager.power_manager.set_policy(PowerPolicy::Balanced)?;
        
        // Transition drivers to sleep state
        manager.power_manager.transition_to_state(AdvancedDriverId(1), PowerState::Sleep)?;
        manager.power_manager.transition_to_state(AdvancedDriverId(2), PowerState::Idle)?;
        
        // Show power statistics
        let power_stats = manager.power_manager.get_power_statistics();
        println!("✓ Power statistics: {:?}", power_stats);
        
        // Resume driver
        manager.power_manager.transition_to_state(AdvancedDriverId(1), PowerState::Active)?;
        println!("✓ Driver resumed from power management");
    }
    println!();

    // 6. Demonstrate hot-plug support
    println!("=== Hot-Plug Support ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Simulate device insertion
        let usb_device_id = manager.hot_plug_manager.register_device(
            crate::DeviceType::USB,
            hot_plug::BusType::USB,
            Some(1)
        )?;
        
        // Simulate device removal
        manager.hot_plug_manager.device_inserted(usb_device_id, Some(0x1234), Some(0x5678))?;
        manager.hot_plug_manager.device_removed(usb_device_id)?;
        
        // Show hot-plug statistics
        let hot_plug_stats = manager.hot_plug_manager.get_statistics();
        println!("✓ Hot-plug statistics: {:?}", hot_plug_stats);
    }
    println!();

    // 7. Demonstrate error recovery
    println!("=== Error Recovery ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Simulate errors
        let error_id = manager.recovery_manager.report_error(
            AdvancedDriverId(1),
            AdvancedDriverError::HardwareError,
            "USB controller timeout".to_string()
        )?;
        
        let error_id2 = manager.recovery_manager.report_error(
            AdvancedDriverId(2),
            AdvancedDriverError::Timeout,
            "Network adapter slow response".to_string()
        )?;
        
        // Show recovery statistics
        let recovery_stats = manager.recovery_manager.get_recovery_statistics();
        println!("✓ Recovery statistics: {:?}", recovery_stats);
    }
    println!();

    // 8. Demonstrate debugging tools
    println!("=== Debugging Tools ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Add trace entries
        manager.debug_manager.add_trace(
            AdvancedDriverId(1),
            debugging::TraceEventType::Initialization,
            debugging::TraceLevel::Info,
            "Driver initialization completed".to_string()
        )?;
        
        // Add performance trace
        manager.debug_manager.add_performance_trace(
            AdvancedDriverId(2),
            debugging::TraceEventType::Read,
            "Network packet read operation".to_string(),
            1500
        )?;
        
        // Add error trace
        manager.debug_manager.add_error_trace(
            AdvancedDriverId(1),
            "Memory allocation failed".to_string(),
            AdvancedDriverError::ResourceExhaustion
        )?;
        
        // Show debug statistics
        let debug_stats = manager.debug_manager.get_debug_statistics();
        println!("✓ Debug statistics: {:?}", debug_stats);
        
        // Generate performance report
        let perf_report = manager.debug_manager.generate_performance_report();
        println!("✓ Performance report generated");
    }
    println!();

    // 9. Demonstrate testing framework
    println!("=== Testing Framework ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Run driver tests
        let test_result = manager.test_manager.run_driver_tests(AdvancedDriverId(1))?;
        println!("✓ Driver tests result: {:?}", test_result.overall_result);
        
        // Run load tests
        let load_test_result = manager.test_manager.run_load_tests(AdvancedDriverId(2))?;
        println!("✓ Load test result: {:?}", load_test_result.result);
        
        // Show test statistics
        let test_stats = manager.test_manager.get_test_statistics();
        println!("✓ Test statistics: {:?}", test_stats);
    }
    println!();

    // 10. Demonstrate version management
    println!("=== Version Management ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        // Register additional versions
        manager.version_manager.register_version("USB Controller Driver", Version::new(1, 2, 1))?;
        manager.version_manager.register_version("Network Adapter Driver", Version::new(2, 1, 2))?;
        manager.version_manager.register_version("Network Adapter Driver", Version::new(2, 2, 0))?;
        
        // Find compatible drivers
        let constraint = VersionConstraint::minimum(Version::new(2, 0, 0));
        if let Ok(driver_id) = manager.version_manager.find_compatible_driver(&constraint) {
            println!("✓ Found compatible driver: {:?}", driver_id);
        }
        
        // Show version statistics
        let version_stats = manager.version_manager.get_version_statistics();
        println!("✓ Version statistics: {:?}", version_stats);
        
        // Generate version report
        let version_report = manager.version_manager.generate_version_report();
        println!("✓ Version report generated");
    }
    println!();

    // 11. Show comprehensive statistics
    println!("=== Comprehensive Statistics ===");
    if let Some(ref manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        let stats = manager.get_statistics();
        println!("Total registered drivers: {}", stats.total_registered);
        println!("Active drivers: {}", stats.active_drivers);
        println!("Loading queue size: {}", stats.loading_queue_size);
        println!("Dependency graph size: {}", stats.dependency_graph_size);
        println!("Power-managed devices: {}", stats.power_managed_devices);
        println!("Hot-plug events: {}", stats.hot_plug_events);
        println!("Debug traces: {}", stats.debug_traces);
    }
    println!();

    // 12. Cleanup
    println!("=== Cleanup ===");
    if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
        unload_driver(AdvancedDriverId(1))?;
        unload_driver(AdvancedDriverId(2))?;
        unload_driver(AdvancedDriverId(3))?;
        println!("✓ All drivers unloaded successfully");
    }

    println!("\n=== Advanced Device Driver Framework Demo Complete ===");
    Ok(())
}

/// Setup function for examples
#[cfg(test)]
fn setup() -> AdvancedResult<()> {
    init_advanced_framework()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_framework_initialization() {
        assert!(setup().is_ok());
    }

    #[test]
    fn test_driver_lifecycle() {
        setup().unwrap();
        
        // Test basic lifecycle operations
        assert!(register_driver(AdvancedDriverInfo {
            id: AdvancedDriverId(100),
            name: "Test Driver",
            version: Version::new(1, 0, 0),
            description: "Test driver",
            author: "Test",
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
        }).is_ok());
        
        assert!(load_driver(AdvancedDriverId(100)).is_ok());
        assert!(unload_driver(AdvancedDriverId(100)).is_ok());
    }

    #[test]
    fn test_power_management() {
        setup().unwrap();
        
        if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
            // Test power state transitions
            assert!(manager.power_manager.enable_power_management(AdvancedDriverId(200)).is_ok());
            assert!(manager.power_manager.transition_to_state(
                AdvancedDriverId(200),
                PowerState::Sleep
            ).is_ok());
            
            let stats = manager.power_manager.get_power_statistics();
            assert!(stats.total_transitions >= 1);
        }
    }

    #[test]
    fn test_hot_plug_operations() {
        setup().unwrap();
        
        if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
            // Test device registration
            assert!(manager.hot_plug_manager.register_device(
                crate::DeviceType::USB,
                hot_plug::BusType::USB,
                Some(1)
            ).is_ok());
            
            let stats = manager.hot_plug_manager.get_statistics();
            assert_eq!(stats.total_devices, 1);
        }
    }

    #[test]
    fn test_error_recovery() {
        setup().unwrap();
        
        if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
            // Test error reporting
            let error_id = manager.recovery_manager.report_error(
                AdvancedDriverId(300),
                AdvancedDriverError::HardwareError,
                "Test error".to_string()
            );
            
            assert!(error_id.is_ok());
            
            let stats = manager.recovery_manager.get_recovery_statistics();
            assert_eq!(stats.total_errors, 1);
        }
    }

    #[test]
    fn test_debugging_capabilities() {
        setup().unwrap();
        
        if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
            // Test trace operations
            assert!(manager.debug_manager.add_trace(
                AdvancedDriverId(400),
                debugging::TraceEventType::Initialization,
                debugging::TraceLevel::Info,
                "Test trace".to_string()
            ).is_ok());
            
            let stats = manager.debug_manager.get_debug_statistics();
            assert!(stats.total_traces >= 1);
        }
    }

    #[test]
    fn test_version_management() {
        setup().unwrap();
        
        if let Some(ref mut manager) = *ADVANCED_DRIVER_MANAGER.lock() {
            // Test version registration
            assert!(manager.version_manager.register_version(
                "Test Driver",
                Version::new(1, 0, 0)
            ).is_ok());
            
            let stats = manager.version_manager.get_version_statistics();
            assert_eq!(stats.total_drivers, 1);
        }
    }
}
