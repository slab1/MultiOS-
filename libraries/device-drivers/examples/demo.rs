//! MultiOS Device Driver Framework Example
//! 
//! This example demonstrates how to use the MultiOS device driver framework
//! to initialize devices, discover hardware, and interact with devices.

#![allow(dead_code)]

use multios_device_drivers::{
    init, register_driver, find_devices, get_device, get_device_count,
    discover_all_devices, init_console, init_system_timer, init_keyboard,
    get_driver_stats, list_drivers, list_devices, DeviceType, DriverError,
    serial, timer, keyboard,
};

/// Main device driver framework demonstration
pub struct DeviceDriverDemo {
    pub console: Option<serial::SerialConsole>,
    pub keyboard: Option<keyboard::Ps2Keyboard>,
    pub timer: Option<timer::TimerManager>,
}

impl DeviceDriverDemo {
    /// Create new demo instance
    pub fn new() -> Self {
        Self {
            console: None,
            keyboard: None,
            timer: None,
        }
    }

    /// Initialize all devices
    pub fn init_all(&mut self) -> Result<(), DriverError> {
        println!("\n=== MultiOS Device Driver Framework Demo ===\n");
        
        // Initialize the device driver framework
        println!("1. Initializing device driver framework...");
        init()?;
        
        // Initialize console output
        println!("2. Initializing serial console...");
        let console = init_console()?;
        console.println("Console initialized successfully!")?;
        self.console = Some(console);
        
        // Initialize system timer
        println!("3. Initializing system timer...");
        let timer_manager = init_system_timer()?;
        self.timer = Some(timer_manager);
        
        // Initialize keyboard input
        println!("4. Initializing keyboard...");
        init_keyboard()?;
        
        // Discover all devices
        println!("5. Discovering devices...");
        let devices = discover_all_devices()?;
        
        println!("Discovered {} devices:", devices.len());
        for device in &devices {
            println!("  - {}", device);
        }
        
        // Show driver statistics
        println!("\n6. Driver Manager Statistics:");
        let stats = get_driver_stats()?;
        println!("  Total drivers: {}", stats.total_drivers);
        println!("  Active devices: {}", stats.active_devices);
        println!("  Device types: {}", stats.total_device_types);
        println!("  Hot-plug devices: {}", stats.hot_plug_devices);
        
        // List registered drivers
        println!("\n7. Registered drivers:");
        let drivers = list_drivers()?;
        for (i, driver) in drivers.iter().enumerate() {
            println!("  {}. {}", i + 1, driver);
        }
        
        Ok(())
    }

    /// Demonstrate serial console operations
    pub fn demo_console(&mut self) -> Result<(), DriverError> {
        if let Some(ref mut console) = self.console {
            console.println("\n--- Console Demo ---")?;
            
            // Test various print operations
            console.print("This is a test message without newline. ")?;
            console.println("This is a test message with newline.")?;
            
            console.println("Testing numeric output:")?;
            console.println(&format!("Pi = {:.2}", core::f64::consts::PI))?;
            
            console.println("Console demo completed!")?;
        }
        
        Ok(())
    }

    /// Demonstrate keyboard operations
    pub fn demo_keyboard(&mut self) -> Result<(), DriverError> {
        if let Some(ref mut kb) = self.keyboard {
            println!("\n--- Keyboard Demo ---");
            
            // Check keyboard state
            println!("Keyboard has {} events in queue", kb.key_queue.lock().len());
            println!("Keyboard modifiers: {:?}", kb.get_modifiers());
            
            // Simulate some key events for demonstration
            let event1 = keyboard::KeyEvent {
                key_code: keyboard::KeyCode::KeyH,
                is_pressed: true,
                modifiers: keyboard::KeyModifiers::default(),
                timestamp: 0,
            };
            
            let event2 = keyboard::KeyEvent {
                key_code: keyboard::KeyCode::KeyI,
                is_pressed: true,
                modifiers: keyboard::KeyModifiers::default(),
                timestamp: 0,
            };
            
            kb.add_key_event(event1);
            kb.add_key_event(event2);
            
            println!("Added key events to queue");
            println!("Keyboard now has {} events", kb.key_queue.lock().len());
            
            // Test character conversion
            let char = kb.key_code_to_char(keyboard::KeyCode::KeyA, keyboard::KeyModifiers { shift: true, ..Default::default() });
            println!("Key 'A' with shift = '{}'", char.unwrap_or('?'));
            
            let char = kb.key_code_to_char(keyboard::KeyCode::KeyA, keyboard::KeyModifiers::default());
            println!("Key 'a' without shift = '{}'", char.unwrap_or('?'));
            
            println!("Keyboard demo completed!");
        }
        
        Ok(())
    }

    /// Demonstrate timer operations
    pub fn demo_timer(&mut self) -> Result<(), DriverError> {
        if let Some(ref timer_manager) = self.timer {
            println!("\n--- Timer Demo ---");
            
            println!("Timer system initialized");
            
            // Get current timer info
            if let Some(timer) = timer_manager.get_current_timer() {
                println!("Current timer: {}", timer.name());
                println!("Timer frequency: {} Hz", timer.get_frequency());
                println!("Tick count: {}", timer.get_tick_count());
                
                // Show elapsed time
                let elapsed_ns = timer.get_elapsed_ns();
                println!("Elapsed time: {} ns ({:.3} ms)", 
                         elapsed_ns, elapsed_ns as f64 / 1_000_000.0);
            }
            
            println!("Timer demo completed!");
        }
        
        Ok(())
    }

    /// Demonstrate device enumeration
    pub fn demo_device_enumeration(&mut self) -> Result<(), DriverError> {
        println!("\n--- Device Enumeration Demo ---");
        
        // Find devices by type
        let device_types = [DeviceType::Keyboard, DeviceType::UART, DeviceType::Mouse];
        
        for device_type in device_types {
            match find_devices(device_type) {
                Ok(devices) => {
                    println!("Found {} {} device(s):", devices.len(), 
                             match device_type {
                                 DeviceType::Keyboard => "keyboard",
                                 DeviceType::UART => "UART/serial",
                                 DeviceType::Mouse => "mouse",
                                 _ => "unknown",
                             });
                    
                    for device in &devices {
                        println!("  - {}", device);
                    }
                }
                Err(_) => {
                    println!("No {} devices found", match device_type {
                        DeviceType::Keyboard => "keyboard",
                        DeviceType::UART => "UART/serial",
                        DeviceType::Mouse => "mouse",
                        _ => "unknown",
                    });
                }
            }
        }
        
        println!("Device enumeration demo completed!");
        Ok(())
    }

    /// Demonstrate driver management
    pub fn demo_driver_management(&mut self) -> Result<(), DriverError> {
        println!("\n--- Driver Management Demo ---");
        
        // List all drivers
        println!("All registered drivers:");
        let drivers = list_drivers()?;
        for (i, driver) in drivers.iter().enumerate() {
            println!("  {}. {}", i + 1, driver);
        }
        
        // Show statistics
        let stats = get_driver_stats()?;
        println!("\nStatistics:");
        println!("  Total drivers: {}", stats.total_drivers);
        println!("  Active devices: {}", stats.active_devices);
        println!("  Device types: {}", stats.total_device_types);
        println!("  Hot-plug devices: {}", stats.hot_plug_devices);
        
        println!("Driver management demo completed!");
        Ok(())
    }

    /// Run complete demo
    pub fn run_demo(&mut self) -> Result<(), DriverError> {
        self.init_all()?;
        
        self.demo_console()?;
        self.demo_keyboard()?;
        self.demo_timer()?;
        self.demo_device_enumeration()?;
        self.demo_driver_management()?;
        
        println!("\n=== Demo Completed Successfully! ===\n");
        Ok(())
    }
}

/// Platform-specific initialization example
#[cfg(target_arch = "x86_64")]
pub fn initialize_x86_64_devices() -> Result<DeviceDriverDemo, DriverError> {
    let mut demo = DeviceDriverDemo::new();
    
    println!("Detected x86_64 architecture, initializing platform-specific devices");
    
    // x86_64 specific initialization
    demo.init_all()?;
    
    Ok(demo)
}

/// Platform-specific initialization example
#[cfg(target_arch = "aarch64")]
pub fn initialize_arm64_devices() -> Result<DeviceDriverDemo, DriverError> {
    let mut demo = DeviceDriverDemo::new();
    
    println!("Detected ARM64 architecture, initializing platform-specific devices");
    
    // ARM64 specific initialization
    demo.init_all()?;
    
    Ok(demo)
}

/// Platform-specific initialization example
#[cfg(target_arch = "riscv64")]
pub fn initialize_riscv64_devices() -> Result<DeviceDriverDemo, DriverError> {
    let mut demo = DeviceDriverDemo::new();
    
    println!("Detected RISC-V 64-bit architecture, initializing platform-specific devices");
    
    // RISC-V specific initialization
    demo.init_all()?;
    
    Ok(demo)
}

/// Initialize drivers for current platform
pub fn initialize_platform_devices() -> Result<DeviceDriverDemo, DriverError> {
    #[cfg(target_arch = "x86_64")]
    {
        initialize_x86_64_devices()
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        initialize_arm64_devices()
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        initialize_riscv64_devices()
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    {
        Err(DriverError::DriverNotSupported)
    }
}

/// Simple console demo for testing
pub fn simple_console_demo() -> Result<(), DriverError> {
    println!("Starting simple console demo...");
    
    // Initialize framework
    init()?;
    
    // Create and initialize console
    let mut console = init_console()?;
    console.println("Hello from MultiOS Device Driver Framework!")?;
    
    println!("Console demo completed!");
    Ok(())
}

/// Test device framework functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_creation() {
        let demo = DeviceDriverDemo::new();
        assert!(demo.console.is_none());
        assert!(demo.keyboard.is_none());
        assert!(demo.timer.is_none());
    }

    #[test]
    fn test_platform_detection() {
        #[cfg(target_arch = "x86_64")]
        {
            let result = initialize_x86_64_devices();
            assert!(result.is_ok());
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            let result = initialize_arm64_devices();
            assert!(result.is_ok());
        }
        
        #[cfg(target_arch = "riscv64")]
        {
            let result = initialize_riscv64_devices();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_simple_demo() {
        // This test just verifies the function compiles and runs
        // In a real system, we would test actual hardware interaction
        let result = simple_console_demo();
        // We don't assert success because we might not have actual hardware
        // but at least the code should compile and run without panic
        let _ = result;
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_framework_initialization() {
        let result = multios_device_drivers::init();
        // Framework initialization should work even without hardware
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_enumeration() {
        multios_device_drivers::init().unwrap();
        
        // This should return an empty result if no devices are detected
        // but should not panic
        let _ = multios_device_drivers::discover_all_devices();
        
        // Check device count
        let count = multios_device_drivers::get_device_count();
        assert!(count >= 0); // Should be at least 0
    }

    #[test]
    fn test_utility_functions() {
        multios_device_drivers::init().unwrap();
        
        // Test utility functions don't panic
        let _ = multios_device_drivers::list_drivers();
        let _ = multios_device_drivers::list_devices();
        let _ = multios_device_drivers::get_driver_stats();
    }
}