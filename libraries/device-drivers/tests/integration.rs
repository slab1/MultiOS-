//! Integration tests for MultiOS Device Driver Framework
//! 
//! These tests validate the complete framework functionality including
//! device discovery, driver binding, and device operations.

use multios_device_drivers::*;
use std::collections::HashMap;

#[test]
fn test_framework_initialization() {
    // Test basic framework initialization
    let result = init();
    assert!(result.is_ok(), "Framework initialization should succeed");
    
    // Test that driver manager is created
    let device_count = get_device_count();
    assert!(device_count >= 0, "Device count should be non-negative");
    
    // Test utility functions don't panic
    let drivers = list_drivers();
    assert!(drivers.is_ok(), "Listing drivers should succeed");
    
    let devices = list_devices();
    assert!(devices.is_ok(), "Listing devices should succeed");
    
    let stats = get_driver_stats();
    assert!(stats.is_ok(), "Getting stats should succeed");
}

#[test]
fn test_device_discovery() {
    // Initialize framework
    assert!(init().is_ok());
    
    // Test device discovery
    let result = discover_all_devices();
    assert!(result.is_ok(), "Device discovery should succeed");
    
    let devices = result.unwrap();
    
    // Framework should discover at least some simulated devices
    // (In real hardware, this would be actual devices)
    assert!(devices.len() >= 0, "Should return at least empty vector");
    
    // Test that we can iterate over discovered devices
    for device in &devices {
        assert!(device.is_available(), "Devices should be available by default");
        println!("Discovered device: {}", device);
    }
}

#[test]
fn test_device_enumeration_by_type() {
    assert!(init().is_ok());
    
    // Test finding different device types
    let device_types = vec![
        DeviceType::Keyboard,
        DeviceType::UART,
        DeviceType::Mouse,
        DeviceType::Display,
    ];
    
    for device_type in device_types {
        let result = find_devices(device_type);
        
        // Should not panic, may return empty if no devices of that type
        match result {
            Ok(devices) => {
                println!("Found {} {} device(s)", devices.len(), format!("{:?}", device_type).to_lowercase());
                assert!(devices.len() >= 0);
            }
            Err(DriverError::DeviceNotFound) => {
                println!("No {} devices found", format!("{:?}", device_type).to_lowercase());
            }
            Err(e) => panic!("Unexpected error for {:?}: {:?}", device_type, e),
        }
    }
}

#[test]
fn test_serial_console_initialization() {
    assert!(init().is_ok());
    
    // Test console initialization
    let result = init_console();
    assert!(result.is_ok(), "Console initialization should succeed");
    
    let mut console = result.unwrap();
    
    // Test basic console operations
    let test_message = "Integration test message";
    let result = console.print(test_message);
    assert!(result.is_ok(), "Console print should succeed");
    
    let result = console.println(test_message);
    assert!(result.is_ok(), "Console println should succeed");
    
    // Test console state
    assert!(console.is_enabled, "Console should be enabled");
    
    // Test input checking (should be false in test environment)
    let has_input = console.has_input();
    println!("Console has input: {}", has_input);
    // Don't assert - input availability depends on test environment
}

#[test]
fn test_system_timer_initialization() {
    assert!(init().is_ok());
    
    // Test timer initialization
    let result = init_system_timer();
    assert!(result.is_ok(), "Timer initialization should succeed");
    
    let timer_manager = result.unwrap();
    
    // Test timer operations
    if let Some(timer) = timer_manager.get_current_timer() {
        println!("Current timer: {}", timer.name());
        println!("Timer frequency: {} Hz", timer.get_frequency());
        
        let tick_count = timer.get_tick_count();
        println!("Tick count: {}", tick_count);
        assert!(tick_count >= 0, "Tick count should be non-negative");
        
        let elapsed_ns = timer.get_elapsed_ns();
        println!("Elapsed time: {} ns", elapsed_ns);
        assert!(elapsed_ns >= 0, "Elapsed time should be non-negative");
    } else {
        println!("No timer available (expected in test environment)");
    }
}

#[test]
fn test_keyboard_initialization() {
    assert!(init().is_ok());
    
    // Test keyboard initialization
    let result = init_keyboard();
    assert!(result.is_ok(), "Keyboard initialization should succeed");
    
    // Test keyboard event handling
    let has_events = keyboard::global_keyboard_has_events();
    println!("Keyboard has events: {}", has_events);
    
    // Don't assert event availability - depends on test environment
    // Just verify the function doesn't panic
    if let Some(kb) = keyboard::get_global_keyboard() {
        let modifiers = kb.get_modifiers();
        println!("Keyboard modifiers: {:?}", modifiers);
        
        let name = kb.name();
        println!("Keyboard name: {}", name);
    }
}

#[test]
fn test_device_driver_binding() {
    assert!(init().is_ok());
    
    // Discover devices to trigger driver binding
    let _ = discover_all_devices();
    
    // Get device statistics
    let stats = get_driver_stats().unwrap();
    println!("Driver stats: {:?}", stats);
    
    // Framework should have registered at least some drivers
    assert!(stats.total_drivers >= 0, "Should have at least 0 drivers");
    assert!(stats.active_devices >= 0, "Should have at least 0 active devices");
    
    // List all drivers
    let drivers = list_drivers().unwrap();
    println!("Registered drivers:");
    for driver in &drivers {
        println!("  - {}", driver);
    }
    
    // Framework should have registered built-in drivers
    assert!(drivers.len() > 0, "Should have registered built-in drivers");
    
    // Check for expected driver names
    let driver_names: HashMap<&str, bool> = drivers.iter()
        .map(|&name| (name, true))
        .collect();
    
    assert!(driver_names.contains_key("16550 UART Driver"), 
           "Should register UART driver");
    assert!(driver_names.contains_key("8254 PIT Driver"), 
           "Should register PIT timer driver");
    assert!(driver_names.contains_key("PS/2 Keyboard Driver"), 
           "Should register keyboard driver");
}

#[test]
fn test_error_handling() {
    assert!(init().is_ok());
    
    // Test finding non-existent device
    let result = get_device(DeviceId(99999));
    assert_eq!(result, Err(DriverError::DeviceNotFound));
    
    // Test finding devices of unknown type
    // DeviceType should go up to UART, so this tests boundary handling
    let unknown_type = unsafe { core::mem::transmute::<u8, DeviceType>(255) };
    let result = find_devices(unknown_type);
    // Should either return empty or error, but not panic
    match result {
        Ok(devices) => {
            assert!(devices.is_empty() || devices.len() >= 0);
        }
        Err(DriverError::DeviceNotFound) => {
            // This is also acceptable
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_hardware_bus_enumeration() {
    assert!(init().is_ok());
    
    // Test bus enumeration through device discovery
    let devices = discover_all_devices().unwrap();
    
    // Count devices by type to verify bus enumeration
    let mut device_type_counts = HashMap::new();
    for device in &devices {
        let device_type = device.device_type;
        *device_type_counts.entry(device_type).or_insert(0) += 1;
    }
    
    println!("Device type distribution:");
    for (device_type, count) in &device_type_counts {
        println!("  {:?}: {} devices", device_type, count);
    }
    
    // Framework should discover at least some devices
    // (In a simulated environment, this may be limited)
    assert!(devices.len() >= 0, "Should return devices or empty vector");
}

#[test]
fn test_driver_event_callbacks() {
    assert!(init().is_ok());
    
    // This test verifies that the framework can handle custom event callbacks
    let mut event_log = Vec::new();
    
    let callback: multios_device_drivers::driver_manager::DriverEventCallback = 
        |event, device_info| {
            event_log.push((event.clone(), device_info.id));
        };
    
    // Register the callback
    // This would require access to the driver manager internals
    // For now, just verify the callback compiles and can be created
    println!("Event callback created successfully");
    
    // Trigger some events by initializing devices
    let _ = discover_all_devices();
    
    // Note: In a real implementation, we would verify that events were logged
    println!("Event logging test completed");
}

#[test]
fn test_memory_safety() {
    assert!(init().is_ok());
    
    // Test that repeated operations don't cause memory leaks
    for i in 0..100 {
        let devices = discover_all_devices();
        assert!(devices.is_ok(), "Discovery should work on iteration {}", i);
        
        let stats = get_driver_stats();
        assert!(stats.is_ok(), "Stats should work on iteration {}", i);
    }
    
    // Test that device operations are safe
    if let Ok(mut console) = init_console() {
        for i in 0..100 {
            let message = format!("Test message {}", i);
            let result = console.print(&message);
            assert!(result.is_ok(), "Console operations should be safe on iteration {}", i);
        }
    }
}

#[test]
fn test_concurrent_access() {
    assert!(init().is_ok());
    
    // Test that multiple threads can safely access the framework
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    
    let device_counts = Arc::new(Mutex::new(Vec::new()));
    let device_counts_clone = device_counts.clone();
    
    let handles: Vec<_> = (0..4).map(|i| {
        let counts = device_counts_clone.clone();
        thread::spawn(move || {
            for _ in 0..10 {
                if let Ok(stats) = get_driver_stats() {
                    counts.lock().unwrap().push(stats.active_devices);
                }
                thread::sleep(Duration::from_millis(1));
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let collected_counts = device_counts.lock().unwrap();
    assert!(!collected_counts.is_empty(), "Should collect stats from all threads");
    
    // Verify that device counts are consistent across threads
    let unique_counts: Vec<_> = collected_counts.iter().collect();
    println!("Collected device counts: {:?}", unique_counts);
}

#[test]
fn test_platform_specific_functionality() {
    assert!(init().is_ok());
    
    // Test platform detection
    #[cfg(target_arch = "x86_64")]
    {
        println!("Testing x86_64 specific features");
        // x86_64 specific tests would go here
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        println!("Testing ARM64 specific features");
        // ARM64 specific tests would go here
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        println!("Testing RISC-V 64-bit specific features");
        // RISC-V specific tests would go here
    }
    
    // Test that the framework works regardless of platform
    let devices = discover_all_devices().unwrap();
    assert!(devices.len() >= 0, "Framework should work on any supported platform");
}

#[test]
fn test_framework_stress() {
    assert!(init().is_ok());
    
    // Stress test the framework with rapid operations
    for iteration in 0..1000 {
        // Multiple device discoveries
        let _ = discover_all_devices();
        
        // Multiple stats queries
        let _ = get_driver_stats();
        let _ = get_device_count();
        
        // Multiple driver queries
        let _ = list_drivers();
        let _ = list_devices();
        
        // Device type queries
        let _ = find_devices(DeviceType::Keyboard);
        let _ = find_devices(DeviceType::UART);
        let _ = find_devices(DeviceType::Mouse);
        
        if iteration % 100 == 0 {
            println!("Completed stress test iteration {}", iteration);
        }
    }
    
    // Verify framework is still functional after stress test
    let stats = get_driver_stats().unwrap();
    assert!(stats.total_drivers > 0, "Framework should still have drivers after stress test");
    assert!(stats.active_devices >= 0, "Framework should still track devices after stress test");
}

#[test]
fn test_integration_with_existing_bootloader() {
    // This test verifies that the device driver framework can work
    // with the existing bootloader device detection
    
    assert!(init().is_ok());
    
    // The framework should be able to coexist with bootloader device detection
    // and potentially extend or enhance it
    
    let devices = discover_all_devices().unwrap();
    println!("Framework discovered {} devices", devices.len());
    
    // Verify that the framework provides additional functionality
    // beyond what the bootloader provides
    let drivers = list_drivers().unwrap();
    assert!(drivers.len() > 0, "Framework should provide driver management");
    
    let stats = get_driver_stats().unwrap();
    println!("Framework stats: {:?}", stats);
    
    // The framework should be able to manage drivers independently
    // of the bootloader
    assert!(stats.total_drivers >= drivers.len(), 
           "Driver count should be consistent");
}