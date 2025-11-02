//! USB Host Controller Driver Example
//!
//! This example demonstrates how to use the MultiOS USB host controller
//! framework with xHCI, EHCI, and OHCI controllers.

use crate::host::{XhciHost, EhciHost, OhciHost};
use crate::{UsbError, UsbResult};
use crate::device::UsbDevice;
use crate::descriptor::{UsbDescriptor, UsbDescriptorType};

/// USB Host Controller Manager
/// Manages multiple host controllers and provides unified interface
pub struct UsbHostManager {
    /// xHCI host controller (USB 3.0+)
    xhci_host: Option<XhciHost>,
    /// EHCI host controller (USB 2.0)
    ehci_host: Option<EhciHost>,
    /// OHCI host controller (USB 1.1)
    ohci_host: Option<OhciHost>,
    /// Active devices
    devices: Vec<UsbDevice>,
}

impl UsbHostManager {
    /// Create new USB host manager
    pub fn new() -> Self {
        Self {
            xhci_host: None,
            ehci_host: None,
            ohci_host: None,
            devices: Vec::new(),
        }
    }

    /// Initialize all available host controllers
    pub fn initialize_all(&mut self) -> UsbResult<()> {
        println!("Initializing USB host controllers...");

        // Initialize xHCI host controller (USB 3.0+)
        if let Ok(xhci) = XhciHost::new() {
            println!("✓ xHCI host controller initialized");
            self.xhci_host = Some(xhci);
        } else {
            println!("⚠ xHCI host controller not available");
        }

        // Initialize EHCI host controller (USB 2.0)
        if let Ok(ehci) = EhciHost::new() {
            println!("✓ EHCI host controller initialized");
            self.ehci_host = Some(ehci);
        } else {
            println!("⚠ EHCI host controller not available");
        }

        // Initialize OHCI host controller (USB 1.1)
        if let Ok(ohci) = OhciHost::new() {
            println!("✓ OHCI host controller initialized");
            self.ohci_host = Some(ohci);
        } else {
            println!("⚠ OHCI host controller not available");
        }

        if self.xhci_host.is_none() && self.ehci_host.is_none() && self.ohci_host.is_none() {
            return Err(UsbError::NoController);
        }

        println!("USB host manager ready!");
        Ok(())
    }

    /// Scan all host controllers for connected devices
    pub fn scan_for_devices(&mut self) -> UsbResult<()> {
        println!("Scanning for USB devices...");

        // Scan each host controller
        if let Some(ref mut xhci) = self.xhci_host {
            self.scan_host_controller(xhci, "xHCI")?;
        }

        if let Some(ref mut ehci) = self.ehci_host {
            self.scan_host_controller(ehci, "EHCI")?;
        }

        if let Some(ref mut ohci) = self.ohci_host {
            self.scan_host_controller(ohci, "OHCI")?;
        }

        println!("Found {} USB devices", self.devices.len());
        Ok(())
    }

    /// Scan a specific host controller
    fn scan_host_controller<H: crate::host::HostController>(
        &mut self,
        host: &mut H,
        controller_name: &str,
    ) -> UsbResult<()> {
        println!("Scanning {} controller...", controller_name);

        // Reset host controller
        host.reset()?;

        // Enumerate devices on each port
        let port_count = host.port_count();
        for port in 0..port_count {
            if let Ok(device) = self.enumerate_device_on_port(host, port) {
                println!("✓ Device found on {} port {}: {} (VID:{:04X}, PID:{:04X})",
                    controller_name, port + 1, device.product_name(),
                    device.vendor_id(), device.product_id());
                self.devices.push(device);
            }
        }

        Ok(())
    }

    /// Enumerate a device on a specific port
    fn enumerate_device_on_port<H: crate::host::HostController>(
        &self,
        host: &H,
        port: u8,
    ) -> UsbResult<UsbDevice> {
        // Check if device is connected
        if !host.port_connected(port)? {
            return Err(UsbError::NotConnected);
        }

        // Reset the port
        host.reset_port(port)?;

        // Get device speed
        let speed = host.port_speed(port)?;

        // Create device instance
        let mut device = UsbDevice::new(
            host.device_address(), // Temporary address
            speed,
            crate::device::UsbDeviceClass::Unknown,
        );

        // Enumerate device (simplified)
        device.set_vendor_id(0x1234);
        device.set_product_id(0x5678);
        device.set_product_name("Example USB Device");

        // Set device address
        let address = self.allocate_device_address();
        host.set_device_address(address);
        device.set_address(address);

        Ok(device)
    }

    /// Allocate unique device address
    fn allocate_device_address(&self) -> u8 {
        let mut used_addresses = [false; 128];

        // Mark used addresses
        for device in &self.devices {
            if device.address() < 128 {
                used_addresses[device.address() as usize] = true;
            }
        }

        // Find free address (addresses 1-127 are available)
        for i in 1..128 {
            if !used_addresses[i] {
                return i as u8;
            }
        }

        // Fallback (should not happen in practice)
        127
    }

    /// Get all connected devices
    pub fn get_devices(&self) -> &[UsbDevice] {
        &self.devices
    }

    /// Get device by address
    pub fn get_device_by_address(&self, address: u8) -> Option<&UsbDevice> {
        self.devices.iter().find(|device| device.address() == address)
    }

    /// Remove device by address
    pub fn remove_device(&mut self, address: u8) {
        self.devices.retain(|device| device.address() != address);
    }

    /// Get host controller information
    pub fn get_host_info(&self) -> String {
        let mut info = String::new();
        info.push_str("USB Host Controller Status:\n");
        info.push_str("===========================\n\n");

        if let Some(_) = &self.xhci_host {
            info.push_str("xHCI (USB 3.0+): ✓ Available\n");
        } else {
            info.push_str("xHCI (USB 3.0+): ✗ Not available\n");
        }

        if let Some(_) = &self.ehci_host {
            info.push_str("EHCI (USB 2.0): ✓ Available\n");
        } else {
            info.push_str("EHCI (USB 2.0): ✗ Not available\n");
        }

        if let Some(_) = &self.ohci_host {
            info.push_str("OHCI (USB 1.1): ✓ Available\n");
        } else {
            info.push_str("OHCI (USB 1.1): ✗ Not available\n");
        }

        info.push_str(&format!("\nConnected devices: {}\n", self.devices.len()));
        info
    }
}

/// Example: USB Host Controller Usage
pub fn example_host_controller() -> UsbResult<()> {
    println!("USB Host Controller Example");
    println!("============================\n");

    // Create host manager
    let mut host_manager = UsbHostManager::new();

    // Initialize all host controllers
    host_manager.initialize_all()?;

    // Display host information
    println!("\n{}", host_manager.get_host_info());

    // Scan for devices
    host_manager.scan_for_devices()?;

    // Display found devices
    println!("\nConnected Devices:");
    println!("-----------------");
    for (i, device) in host_manager.get_devices().iter().enumerate() {
        println!("{}: {} (VID:{:04X}, PID:{:04X})",
            i + 1, device.product_name(), device.vendor_id(), device.product_id());
    }

    // Example: Control transfer to configure device
    if !host_manager.get_devices().is_empty() {
        let device = &host_manager.get_devices()[0];
        println!("\nExample: Configuring device at address {}", device.address());

        // This would be a real control transfer in actual implementation
        println!("• Sending GET_DESCRIPTOR request");
        println!("• Configuring device settings");
        println!("• Setting device as ready for use");
    }

    println!("\nHost controller example completed successfully!");
    Ok(())
}

/// Example: Multiple Host Controller Detection
pub fn example_detect_controllers() -> UsbResult<()> {
    println!("USB Controller Detection Example");
    println!("=================================\n");

    let mut detected_controllers = Vec::new();

    // Try to detect xHCI controller
    match XhciHost::new() {
        Ok(_) => detected_controllers.push("xHCI (USB 3.0+)"),
        Err(_) => println!("xHCI controller not found or not supported"),
    }

    // Try to detect EHCI controller
    match EhciHost::new() {
        Ok(_) => detected_controllers.push("EHCI (USB 2.0)"),
        Err(_) => println!("EHCI controller not found or not supported"),
    }

    // Try to detect OHCI controller
    match OhciHost::new() {
        Ok(_) => detected_controllers.push("OHCI (USB 1.1)"),
        Err(_) => println!("OHCI controller not found or not supported"),
    }

    println!("Detected USB Host Controllers:");
    if detected_controllers.is_empty() {
        println!("No USB host controllers found!");
    } else {
        for (i, controller) in detected_controllers.iter().enumerate() {
            println!("{}: {}", i + 1, controller);
        }
    }

    println!("\nController capabilities by USB version:");
    println!("• xHCI: SuperSpeed (5+ Gbps), Enhanced power management");
    println!("• EHCI: High-Speed (480 Mbps), Better power management than OHCI");
    println!("• OHCI: Full-Speed (12 Mbps) and Low-Speed (1.5 Mbps)");

    Ok(())
}

/// Example: USB Device Enumeration Process
pub fn example_device_enumeration() {
    println!("USB Device Enumeration Process");
    println!("===============================\n");

    println!("Step-by-step USB device enumeration:");
    println!("====================================\n");

    println!("1. Device Detection");
    println!("   • Device connects to USB port");
    println!("   • Pull-up resistors on D+/D- lines signal presence");
    println!("   • Host detects through line state changes");
    println!();

    println!("2. Reset and Address Assignment");
    println!("   • Host sends USB reset signal");
    println!("   • Device enters default state (address 0)");
    println!("   • Host assigns unique address to device");
    println!();

    println!("3. Descriptor Retrieval");
    println!("   • GET_DESCRIPTOR (Device) - Device capabilities");
    println!("   • GET_DESCRIPTOR (Configuration) - Interface/endpoint layout");
    println!("   • GET_DESCRIPTOR (String) - Human-readable text");
    println!();

    println!("4. Configuration Selection");
    println!("   • Host selects appropriate configuration");
    println!("   • Device activates interfaces and endpoints");
    println!("   • Device becomes operational");
    println!();

    println!("5. Device Ready");
    println!("   • Host can now communicate with device");
    println!("   • Applications can access device functions");
    println!("   • Device ready for normal operation");
    println!();

    println!("Example descriptor retrieval sequence:");
    println!("====================================");
    println!("SETUP[GET_DESCRIPTOR] -> DATA[Device Descriptor] -> HANDSHAKE[ACK]");
    println!("SETUP[SET_ADDRESS]    -> HANDSHAKE[ACK]");
    println!("SETUP[GET_DESCRIPTOR] -> DATA[Configuration Descriptor] -> HANDSHAKE[ACK]");
    println!("SETUP[GET_DESCRIPTOR] -> DATA[String Descriptor] -> HANDSHAKE[ACK]");
    println!("SETUP[SET_CONFIGURATION] -> HANDSHAKE[ACK]");
    println!("Device is now ready for use!");
}