//! Desktop Features Support
//! 
//! Provides support for USB, graphics, multiple monitors, and other
//! desktop-specific features

use crate::log::{info, warn, error};
use crate::KernelError;

use super::{PciManager, UsbControllerInfo, UsbControllerType, UsbVersion, 
           DisplayDeviceInfo, DisplayType, ConnectionType};

/// USB endpoint types
const USB_ENDPOINT_TYPE_CONTROL: u8 = 0;
const USB_ENDPOINT_TYPE_ISOCHRONOUS: u8 = 1;
const USB_ENDPOINT_TYPE_BULK: u8 = 2;
const USB_ENDPOINT_TYPE_INTERRUPT: u8 = 3;

/// USB request types
const USB_REQ_TYPE_STANDARD: u8 = 0 << 5;
const USB_REQ_TYPE_CLASS: u8 = 1 << 5;
const USB_REQ_TYPE_VENDOR: u8 = 2 << 5;

/// USB request directions
const USB_DIR_OUT: u8 = 0;
const USB_DIR_IN: u8 = 1 << 7;

/// USB request recipients
const USB_RECIPIENT_DEVICE: u8 = 0;
const USB_RECIPIENT_INTERFACE: u8 = 1;
const USB_RECIPIENT_ENDPOINT: u8 = 2;
const USB_RECIPIENT_OTHER: u8 = 3;

/// USB device classes
const USB_CLASS_DEVICE: u8 = 0x00;
const USB_CLASS_AUDIO: u8 = 0x01;
const USB_CLASS_COMMUNICATION: u8 = 0x02;
const USB_CLASS_HID: u8 = 0x03;
const USB_CLASS_PHYSICAL: u8 = 0x05;
const USB_CLASS_IMAGE: u8 = 0x06;
const USB_CLASS_PRINTER: u8 = 0x07;
const USB_CLASS_MASS_STORAGE: u8 = 0x08;
const USB_CLASS_HUB: u8 = 0x09;
const USB_CLASS_CDC_DATA: u8 = 0x0A;
const USB_CLASS_SMART_CARD: u8 = 0x0B;
const USB_CLASS_CONTENT_SECURITY: u8 = 0x0D;
const USB_CLASS_VIDEO: u8 = 0x0E;
const USB_CLASS_WIRELESS_CONTROLLER: u8 = 0xE0;
const USB_CLASS_MISCELLANEOUS: u8 = 0xEF;
const USB_CLASS_APPLICATION_SPECIFIC: u8 = 0xFE;
const USB_CLASS_VENDOR_SPECIFIC: u8 = 0xFF;

/// USB device speeds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
}

/// USB hub information
#[derive(Debug, Clone)]
pub struct UsbHubInfo {
    pub hub_address: u8,
    pub port_count: u8,
    pub speed_support: Vec<UsbSpeed>,
    pub power_good: u8, // Power good delay (ms)
    pub power_control: bool,
    pub over_current_protection: bool,
}

/// USB device information
#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub address: u8,
    pub parent_hub: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub speed: UsbSpeed,
    pub max_power: u8, // In 2mA units
    pub manufacturer: String,
    pub product: String,
    pub serial_number: String,
    pub configuration_count: u8,
    pub interface_count: u8,
}

/// USB configuration information
#[derive(Debug, Clone)]
pub struct UsbConfiguration {
    pub configuration_value: u8,
    pub attributes: u8,
    pub max_power: u8,
    pub interface_count: u8,
    pub interfaces: Vec<UsbInterface>,
}

/// USB interface information
#[derive(Debug, Clone)]
pub struct UsbInterface {
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub endpoint_count: u8,
    pub endpoints: Vec<UsbEndpoint>,
}

/// USB endpoint information
#[derive(Debug, Clone)]
pub struct UsbEndpoint {
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

/// USB Manager
pub struct UsbManager {
    pub initialized: bool,
    pub controllers: Vec<UsbControllerInfo>,
    pub devices: Vec<UsbDevice>,
    pub hubs: Vec<UsbHubInfo>,
    pub configurations: Vec<UsbConfiguration>,
    pub is_enumerating: bool,
}

/// USB device type detection
#[derive(Debug, Clone)]
pub enum UsbDeviceType {
    Keyboard,
    Mouse,
    Storage,
    Printer,
    Camera,
    Audio,
    Network,
    Hub,
    Unknown,
}

/// Graphics mode information
#[derive(Debug, Clone)]
pub struct GraphicsMode {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u8,
    pub refresh_rate: u32,
    pub format: PixelFormat,
}

/// Pixel formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
    Rgb332,
    Rgb444,
    Rgb555,
    Rgb565,
    Rgb888,
    Rgba8888,
    Bgra8888,
    Yuv422,
    Yuv444,
}

/// Graphics information
#[derive(Debug, Clone)]
pub struct GraphicsInfo {
    pub device_id: u16,
    pub vendor_id: u16,
    pub memory_size: u32, // In bytes
    pub modes: Vec<GraphicsMode>,
    pub current_mode: Option<GraphicsMode>,
    pub supports_3d: bool,
    pub supports_acceleration: bool,
}

/// Multiple monitor configuration
#[derive(Debug, Clone)]
pub struct MultiMonitorConfig {
    pub primary_display: u8,
    pub display_count: u8,
    pub displays: Vec<MonitorLayout>,
    pub virtual_width: u32,
    pub virtual_height: u32,
    pub bezel_correction: bool,
}

/// Monitor layout information
#[derive(Debug, Clone)]
pub struct MonitorLayout {
    pub display_id: u8,
    pub x_position: i32,
    pub y_position: i32,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub is_primary: bool,
    pub is_enabled: bool,
}

/// Graphics Manager
pub struct GraphicsManager {
    pub initialized: bool,
    pub graphics_cards: Vec<GraphicsInfo>,
    pub displays: Vec<DisplayDeviceInfo>,
    pub multi_monitor_config: Option<MultiMonitorConfig>,
    pub current_mode: Option<GraphicsMode>,
}

/// Power button event
#[derive(Debug, Clone, Copy)]
pub enum PowerButtonEvent {
    Pressed,
    Released,
    Held,
}

/// Thermal event
#[derive(Debug, Clone, Copy)]
pub enum ThermalEvent {
    Critical,
    Hot,
    Normal,
}

/// Desktop Manager
pub struct DesktopManager {
    pub usb_manager: UsbManager,
    pub graphics_manager: GraphicsManager,
    pub is_initialized: bool,
}

impl DesktopManager {
    /// Create new desktop manager
    pub fn new() -> Self {
        Self {
            usb_manager: UsbManager {
                initialized: false,
                controllers: Vec::new(),
                devices: Vec::new(),
                hubs: Vec::new(),
                configurations: Vec::new(),
                is_enumerating: false,
            },
            graphics_manager: GraphicsManager {
                initialized: false,
                graphics_cards: Vec::new(),
                displays: Vec::new(),
                multi_monitor_config: None,
                current_mode: None,
            },
            is_initialized: false,
        }
    }
}

// USB Manager Implementation
impl UsbManager {
    /// Create new USB manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            controllers: Vec::new(),
            devices: Vec::new(),
            hubs: Vec::new(),
            configurations: Vec::new(),
            is_enumerating: false,
        }
    }
    
    /// Initialize USB subsystem
    pub fn initialize(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Initializing USB subsystem...");
        
        // Step 1: Detect USB controllers
        self.detect_usb_controllers(pci_manager)?;
        
        // Step 2: Initialize USB controllers
        self.init_usb_controllers()?;
        
        // Step 3: Reset and configure controllers
        self.reset_usb_controllers()?;
        
        // Step 4: Enumerate USB devices
        self.enumerate_usb_devices()?;
        
        self.initialized = true;
        
        info!("USB subsystem initialized: {} controllers, {} devices", 
              self.controllers.len(), self.devices.len());
        
        Ok(())
    }
    
    /// Detect USB controllers
    fn detect_usb_controllers(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Detecting USB controllers...");
        
        // Find USB controller PCI devices (class 0x0C, subclass 0x03)
        let usb_devices = pci_manager.devices.iter()
            .filter(|d| d.class_code == 0x0C && d.subclass == 0x03)
            .collect::<Vec<_>>();
        
        for device in usb_devices {
            let controller_info = self.detect_usb_controller_type(device.bus, device.device, device.function)?;
            
            if controller_info.is_some() {
                self.controllers.push(controller_info.unwrap());
                info!("Found USB controller at {}:{}:{} ({}:{})", 
                      device.bus, device.device, device.function,
                      device.vendor_id, device.device_id);
            }
        }
        
        info!("Detected {} USB controllers", self.controllers.len());
        Ok(())
    }
    
    /// Detect USB controller type
    fn detect_usb_controller_type(&self, bus: u8, device: u8, function: u8) 
        -> Result<Option<UsbControllerInfo>, KernelError> {
        // Read PCI configuration to determine USB controller type
        // Based on programming interface (prog_if) field:
        // 0x00 = UHCI (USB 1.1)
        // 0x10 = OHCI (USB 1.1)
        // 0x20 = EHCI (USB 2.0)
        // 0x30 = XHCI (USB 3.0+)
        
        // This would read from PCI configuration space
        let prog_if = 0x30; // Simulate XHCI
        
        let controller_info = match prog_if {
            0x00 | 0x10 => UsbControllerInfo {
                controller_type: UsbControllerType::Uhci,
                pci_location: Some((bus, device, function)),
                usb_version: UsbVersion::Usb2_0,
                ports: 2,
                driver_attached: false,
            },
            0x20 => UsbControllerInfo {
                controller_type: UsbControllerType::Ehci,
                pci_location: Some((bus, device, function)),
                usb_version: UsbVersion::Usb2_0,
                ports: 6,
                driver_attached: false,
            },
            0x30 => UsbControllerInfo {
                controller_type: UsbControllerType::Xhci,
                pci_location: Some((bus, device, function)),
                usb_version: UsbVersion::Usb3_0,
                ports: 15,
                driver_attached: false,
            },
            _ => UsbControllerInfo {
                controller_type: UsbControllerType::Uhci,
                pci_location: Some((bus, device, function)),
                usb_version: UsbVersion::Usb1_1,
                ports: 2,
                driver_attached: false,
            },
        };
        
        Ok(Some(controller_info))
    }
    
    /// Initialize USB controllers
    fn init_usb_controllers(&mut self) -> Result<(), KernelError> {
        info!("Initializing USB controllers...");
        
        for controller in &mut self.controllers {
            match controller.controller_type {
                UsbControllerType::Xhci => self.init_xhci_controller(controller)?,
                UsbControllerType::Ehci => self.init_ehci_controller(controller)?,
                UsbControllerType::Ohci => self.init_ohci_controller(controller)?,
                UsbControllerType::Uhci => self.init_uhci_controller(controller)?,
            }
        }
        
        Ok(())
    }
    
    /// Initialize XHCI controller
    fn init_xhci_controller(&self, controller: &mut UsbControllerInfo) -> Result<(), KernelError> {
        info!("Initializing XHCI (USB 3.0+) controller with {} ports", controller.ports);
        
        // Configure XHCI memory-mapped I/O
        // Setup command and event rings
        // Configure interrupts
        // Start controller
        
        Ok(())
    }
    
    /// Initialize EHCI controller
    fn init_ehci_controller(&self, controller: &mut UsbControllerInfo) -> Result<(), KernelError> {
        info!("Initializing EHCI (USB 2.0) controller with {} ports", controller.ports);
        
        // Configure EHCI registers
        // Setup periodic and asynchronous schedules
        // Configure interrupts
        
        Ok(())
    }
    
    /// Initialize OHCI controller
    fn init_ohci_controller(&self, controller: &mut UsbControllerInfo) -> Result<(), KernelError> {
        info!("Initializing OHCI (USB 1.1) controller with {} ports", controller.ports);
        
        // Configure OHCI registers
        // Setup control and bulk lists
        
        Ok(())
    }
    
    /// Initialize UHCI controller
    fn init_uhci_controller(&self, controller: &mut UsbControllerInfo) -> Result<(), KernelError> {
        info!("Initializing UHCI (USB 1.1) controller with {} ports", controller.ports);
        
        // Configure UHCI frame list
        // Setup transfer descriptors
        
        Ok(())
    }
    
    /// Reset USB controllers
    fn reset_usb_controllers(&mut self) -> Result<(), KernelError> {
        info!("Resetting USB controllers...");
        
        for controller in &mut self.controllers {
            // Reset controller hardware
            // Wait for reset completion
            // Configure for enumeration
            
            controller.driver_attached = true;
        }
        
        Ok(())
    }
    
    /// Enumerate USB devices
    fn enumerate_usb_devices(&mut self) -> Result<(), KernelError> {
        info!("Enumerating USB devices...");
        
        self.is_enumerating = true;
        
        // Start with root hub enumeration
        self.enumerate_root_hubs()?;
        
        // This would recursively enumerate all USB devices
        // including hubs and their downstream devices
        
        // Add some example USB devices for demonstration
        self.add_example_usb_devices()?;
        
        self.is_enumerating = false;
        
        info!("USB device enumeration complete");
        Ok(())
    }
    
    /// Enumerate root hubs
    fn enumerate_root_hubs(&mut self) -> Result<(), KernelError> {
        // Root hubs are built into each USB controller
        // Enumerate the ports on each root hub
        
        for controller in &self.controllers {
            let hub_info = UsbHubInfo {
                hub_address: 1, // Root hub address
                port_count: controller.ports,
                speed_support: vec![UsbSpeed::Full, UsbSpeed::High],
                power_good: 100,
                power_control: true,
                over_current_protection: true,
            };
            
            self.hubs.push(hub_info);
        }
        
        Ok(())
    }
    
    /// Add example USB devices for demonstration
    fn add_example_usb_devices(&mut self) -> Result<(), KernelError> {
        // USB Keyboard
        self.devices.push(UsbDevice {
            address: 2,
            parent_hub: 1,
            vendor_id: 0x046A,
            product_id: 0x0001,
            device_class: USB_CLASS_HID,
            device_subclass: 0x01,
            device_protocol: 0x01,
            speed: UsbSpeed::Full,
            max_power: 100,
            manufacturer: "Logitech".to_string(),
            product: "USB Keyboard".to_string(),
            serial_number: "KEYBOARD001".to_string(),
            configuration_count: 1,
            interface_count: 1,
        });
        
        // USB Mouse
        self.devices.push(UsbDevice {
            address: 3,
            parent_hub: 1,
            vendor_id: 0x046A,
            product_id: 0x0002,
            device_class: USB_CLASS_HID,
            device_subclass: 0x01,
            device_protocol: 0x02,
            speed: UsbSpeed::Full,
            max_power: 100,
            manufacturer: "Logitech".to_string(),
            product: "USB Mouse".to_string(),
            serial_number: "MOUSE001".to_string(),
            configuration_count: 1,
            interface_count: 1,
        });
        
        // USB Storage Device
        self.devices.push(UsbDevice {
            address: 4,
            parent_hub: 1,
            vendor_id: 0x0781,
            product_id: 0x5567,
            device_class: USB_CLASS_MASS_STORAGE,
            device_subclass: 0x06,
            device_protocol: 0x50,
            speed: UsbSpeed::High,
            max_power: 200,
            manufacturer: "SanDisk".to_string(),
            product: "Cruzer Blade".to_string(),
            serial_number: "SANDISK001".to_string(),
            configuration_count: 1,
            interface_count: 1,
        });
        
        Ok(())
    }
    
    /// Get USB device type
    pub fn get_device_type(&self, device: &UsbDevice) -> UsbDeviceType {
        match device.device_class {
            USB_CLASS_HID => {
                if device.device_protocol == 0x01 {
                    UsbDeviceType::Keyboard
                } else {
                    UsbDeviceType::Mouse
                }
            },
            USB_CLASS_MASS_STORAGE => UsbDeviceType::Storage,
            USB_CLASS_PRINTER => UsbDeviceType::Printer,
            USB_CLASS_IMAGE => UsbDeviceType::Camera,
            USB_CLASS_AUDIO => UsbDeviceType::Audio,
            USB_CLASS_VENDOR_SPECIFIC => {
                // Check vendor/product IDs for specific devices
                match (device.vendor_id, device.product_id) {
                    (0x046A, _) => UsbDeviceType::Keyboard, // Logitech
                    (0x0781, _) => UsbDeviceType::Storage,   // SanDisk
                    _ => UsbDeviceType::Unknown,
                }
            },
            _ => UsbDeviceType::Unknown,
        }
    }
    
    /// Get all USB controllers
    pub fn get_controllers(&self) -> &[UsbControllerInfo] {
        &self.controllers
    }
    
    /// Get all USB devices
    pub fn get_devices(&self) -> &[UsbDevice] {
        &self.devices
    }
    
    /// Get USB hubs
    pub fn get_hubs(&self) -> &[UsbHubInfo] {
        &self.hubs
    }
    
    /// Find USB device by address
    pub fn find_device_by_address(&self, address: u8) -> Option<&UsbDevice> {
        self.devices.iter().find(|&d| d.address == address)
    }
    
    /// Find devices by class
    pub fn find_devices_by_class(&self, device_class: u8) -> Vec<&UsbDevice> {
        self.devices.iter().filter(|&d| d.device_class == device_class).collect()
    }
    
    /// Reset USB device
    pub fn reset_device(&self, address: u8) -> Result<(), KernelError> {
        if let Some(device) = self.find_device_by_address(address) {
            info!("Resetting USB device {} ({})", address, device.product);
            
            // Send USB reset command to device
            // Wait for reset completion
            // Re-enumerate device
            
            Ok(())
        } else {
            Err(KernelError::NotFound)
        }
    }
}

// Graphics Manager Implementation
impl GraphicsManager {
    /// Create new graphics manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            graphics_cards: Vec::new(),
            displays: Vec::new(),
            multi_monitor_config: None,
            current_mode: None,
        }
    }
    
    /// Initialize graphics subsystem
    pub fn initialize(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Initializing graphics subsystem...");
        
        // Step 1: Detect graphics cards
        self.detect_graphics_cards(pci_manager)?;
        
        // Step 2: Initialize graphics cards
        self.init_graphics_cards()?;
        
        // Step 3: Detect displays
        self.detect_displays()?;
        
        // Step 4: Setup multiple monitor support
        self.setup_multi_monitor()?;
        
        // Step 5: Set default display mode
        self.set_default_mode()?;
        
        self.initialized = true;
        
        info!("Graphics subsystem initialized: {} graphics cards, {} displays", 
              self.graphics_cards.len(), self.displays.len());
        
        Ok(())
    }
    
    /// Detect graphics cards
    fn detect_graphics_cards(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Detecting graphics cards...");
        
        // Find display controller PCI devices (class 0x03)
        let graphics_devices = pci_manager.find_devices_by_class(0x03); // Display controller
        
        for device in graphics_devices {
            let graphics_info = self.detect_graphics_card(
                device.bus, device.device, device.function,
                device.vendor_id, device.device_id
            )?;
            
            if graphics_info.is_some() {
                self.graphics_cards.push(graphics_info.unwrap());
                info!("Found graphics card at {}:{}:{} ({}:{})", 
                      device.bus, device.device, device.function,
                      device.vendor_id, device.device_id);
            }
        }
        
        info!("Detected {} graphics cards", self.graphics_cards.len());
        Ok(())
    }
    
    /// Detect graphics card
    fn detect_graphics_card(&self, bus: u8, device: u8, function: u8, 
                          vendor_id: u16, device_id: u16) -> Result<Option<GraphicsInfo>, KernelError> {
        // Detect supported display modes from graphics card
        let modes = vec![
            GraphicsMode {
                width: 1920,
                height: 1080,
                bits_per_pixel: 32,
                refresh_rate: 60,
                format: PixelFormat::Rgba8888,
            },
            GraphicsMode {
                width: 1366,
                height: 768,
                bits_per_pixel: 32,
                refresh_rate: 60,
                format: PixelFormat::Rgba8888,
            },
            GraphicsMode {
                width: 2560,
                height: 1440,
                bits_per_pixel: 32,
                refresh_rate: 60,
                format: PixelFormat::Rgba8888,
            },
        ];
        
        let graphics_info = GraphicsInfo {
            device_id,
            vendor_id,
            memory_size: 268435456, // 256MB
            modes,
            current_mode: None,
            supports_3d: true,
            supports_acceleration: true,
        };
        
        Ok(Some(graphics_info))
    }
    
    /// Initialize graphics cards
    fn init_graphics_cards(&mut self) -> Result<(), KernelError> {
        info!("Initializing graphics cards...");
        
        for graphics_card in &mut self.graphics_cards {
            // Initialize graphics card hardware
            // Setup video memory
            // Configure interrupts
            
            info!("Initialized graphics card ({}:{})", 
                  graphics_card.vendor_id, graphics_card.device_id);
        }
        
        Ok(())
    }
    
    /// Detect displays
    fn detect_displays(&mut self) -> Result<(), KernelError> {
        info!("Detecting displays...");
        
        // Detect displays through EDID (Extended Display Identification Data)
        // This would query displays connected to graphics cards
        
        // Add example displays
        self.displays.push(DisplayDeviceInfo {
            display_type: DisplayType::Monitor,
            resolution: (1920, 1080),
            refresh_rate_hz: 60,
            color_depth: 32,
            connection_type: ConnectionType::Hdmi,
            edid_info: Some(vec![0; 128]), // Example EDID data
        });
        
        self.displays.push(DisplayDeviceInfo {
            display_type: DisplayType::Monitor,
            resolution: (1366, 768),
            refresh_rate_hz: 60,
            color_depth: 32,
            connection_type: ConnectionType::Vga,
            edid_info: Some(vec![0; 128]), // Example EDID data
        });
        
        info!("Detected {} displays", self.displays.len());
        Ok(())
    }
    
    /// Setup multiple monitor support
    fn setup_multi_monitor(&mut self) -> Result<(), KernelError> {
        info!("Setting up multiple monitor support...");
        
        if self.displays.len() >= 2 {
            let virtual_width = self.displays.iter().map(|d| d.resolution.0).sum();
            let virtual_height = self.displays.iter().map(|d| d.resolution.1).max().unwrap_or(0);
            
            let displays = self.displays.iter().enumerate().map(|(i, display)| {
                MonitorLayout {
                    display_id: i as u8,
                    x_position: if i == 0 { 0 } else { self.displays[0].resolution.0 as i32 },
                    y_position: 0,
                    width: display.resolution.0,
                    height: display.resolution.1,
                    refresh_rate: display.refresh_rate_hz,
                    is_primary: i == 0,
                    is_enabled: true,
                }
            }).collect();
            
            self.multi_monitor_config = Some(MultiMonitorConfig {
                primary_display: 0,
                display_count: self.displays.len() as u8,
                displays,
                virtual_width,
                virtual_height,
                bezel_correction: true,
            });
            
            info!("Multi-monitor setup complete: {} displays, virtual resolution {}x{}", 
                  self.displays.len(), virtual_width, virtual_height);
        }
        
        Ok(())
    }
    
    /// Set default display mode
    fn set_default_mode(&mut self) -> Result<(), KernelError> {
        if !self.graphics_cards.is_empty() && !self.graphics_cards[0].modes.is_empty() {
            let default_mode = self.graphics_cards[0].modes[0].clone();
            self.current_mode = Some(default_mode.clone());
            
            // Set mode on primary graphics card
            // This would involve programming graphics card registers
            
            info!("Set default graphics mode: {}x{}x{} @ {}Hz", 
                  default_mode.width, default_mode.height, 
                  default_mode.bits_per_pixel, default_mode.refresh_rate);
        }
        
        Ok(())
    }
    
    /// Get all graphics cards
    pub fn get_graphics_cards(&self) -> &[GraphicsInfo] {
        &self.graphics_cards
    }
    
    /// Get all displays
    pub fn get_displays(&self) -> &[DisplayDeviceInfo] {
        &self.displays
    }
    
    /// Get current display mode
    pub fn get_current_mode(&self) -> Option<&GraphicsMode> {
        self.current_mode.as_ref()
    }
    
    /// Get multi-monitor configuration
    pub fn get_multi_monitor_config(&self) -> Option<&MultiMonitorConfig> {
        self.multi_monitor_config.as_ref()
    }
    
    /// Set display mode
    pub fn set_display_mode(&mut self, display_id: u8, mode: &GraphicsMode) -> Result<(), KernelError> {
        if display_id >= self.displays.len() as u8 {
            return Err(KernelError::NotFound);
        }
        
        info!("Setting display {} to mode {}x{}x{} @ {}Hz", 
              display_id, mode.width, mode.height, mode.bits_per_pixel, mode.refresh_rate);
        
        // Program graphics card to set the new mode
        Ok(())
    }
    
    /// Enable/disable display
    pub fn set_display_enabled(&mut self, display_id: u8, enabled: bool) -> Result<(), KernelError> {
        if let Some(ref mut config) = self.multi_monitor_config {
            if display_id < config.displays.len() as u8 {
                config.displays[display_id as usize].is_enabled = enabled;
                
                info!("{} display {}", if enabled { "Enabled" } else { "Disabled" }, display_id);
                
                // Update virtual resolution if needed
                if config.display_count >= 2 {
                    config.virtual_width = config.displays.iter()
                        .filter(|d| d.is_enabled)
                        .map(|d| d.width)
                        .sum();
                }
                
                Ok(())
            } else {
                Err(KernelError::NotFound)
            }
        } else {
            Err(KernelError::NotSupported)
        }
    }
    
    /// Get display layout
    pub fn get_display_layout(&self, display_id: u8) -> Result<MonitorLayout, KernelError> {
        if let Some(ref config) = self.multi_monitor_config {
            if display_id < config.displays.len() as u8 {
                Ok(config.displays[display_id as usize].clone())
            } else {
                Err(KernelError::NotFound)
            }
        } else {
            Err(KernelError::NotSupported)
        }
    }
    
    /// Set display position
    pub fn set_display_position(&mut self, display_id: u8, x: i32, y: i32) -> Result<(), KernelError> {
        if let Some(ref mut config) = self.multi_monitor_config {
            if display_id < config.displays.len() as u8 {
                let display = &mut config.displays[display_id as usize];
                display.x_position = x;
                display.y_position = y;
                
                info!("Moved display {} to position ({}, {})", display_id, x, y);
                
                Ok(())
            } else {
                Err(KernelError::NotFound)
            }
        } else {
            Err(KernelError::NotSupported)
        }
    }
}

// Desktop Manager Implementation
impl DesktopManager {
    /// Create new desktop manager
    pub fn new() -> Self {
        Self {
            usb_manager: UsbManager::new(),
            graphics_manager: GraphicsManager::new(),
            is_initialized: false,
        }
    }
    
    /// Initialize desktop managers
    pub fn initialize(&mut self, pci_manager: &PciManager) -> Result<(), KernelError> {
        info!("Initializing desktop subsystem...");
        
        // Initialize USB subsystem
        self.usb_manager.initialize(pci_manager)?;
        
        // Initialize graphics subsystem
        self.graphics_manager.initialize(pci_manager)?;
        
        self.is_initialized = true;
        
        info!("Desktop subsystem initialization complete");
        Ok(())
    }
    
    /// Get USB manager
    pub fn get_usb_manager(&self) -> &UsbManager {
        &self.usb_manager
    }
    
    /// Get USB manager (mutable)
    pub fn get_usb_manager_mut(&mut self) -> &mut UsbManager {
        &mut self.usb_manager
    }
    
    /// Get graphics manager
    pub fn get_graphics_manager(&self) -> &GraphicsManager {
        &self.graphics_manager
    }
    
    /// Get graphics manager (mutable)
    pub fn get_graphics_manager_mut(&mut self) -> &mut GraphicsManager {
        &mut self.graphics_manager
    }
}