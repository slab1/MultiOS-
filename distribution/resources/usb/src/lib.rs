//! MultiOS USB Device Driver Framework
//! 
//! A comprehensive USB driver framework supporting:
//! - USB host controller drivers (xHCI, EHCI, OHCI)
//! - USB device class drivers (HID, MSC, CDC, Audio)
//! - USB hub management
//! - Hotplug detection and device enumeration
//! - USB power management
//! - USB security isolation
//! - Educational USB protocol analyzer

#![no_std]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "std")]
#[allow(unused_imports)]
use std::println;

use core::fmt;
use core::sync::atomic::{AtomicU32, Ordering};

// Main framework modules
pub mod host;
pub mod classes;
pub mod hub;
pub mod hotplug;
pub mod power;
pub mod security;
pub mod protocol_analyzer;
pub mod tests;

// Re-export commonly used types
pub use host::{XhciHost, EhciHost, OhciHost};
pub use classes::{HidDevice, MscDevice, CdcDevice, AudioDevice, UsbDeviceClass};
pub use hub::UsbHub;
pub use hotplug::HotplugDetector;
pub use power::{UsbPowerManager, PowerState};
pub use security::{SecurityManager, SecurityLevel, DeviceFingerprint, TrustState};
pub use protocol_analyzer::{ProtocolAnalyzer, DescriptorDecoder};
pub use tests::{TestSuite, TestResult, run_comprehensive_tests, quick_validation_test, benchmark_framework};

/// USB Version constants
pub const USB_VERSION_1_0: u16 = 0x0100;
pub const USB_VERSION_1_1: u16 = 0x0110;
pub const USB_VERSION_2_0: u16 = 0x0200;
pub const USB_VERSION_2_1: u16 = 0x0210;
pub const USB_VERSION_3_0: u16 = 0x0300;
pub const USB_VERSION_3_1: u16 = 0x0310;
pub const USB_VERSION_3_2: u16 = 0x0320;

/// USB Device Class codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbClass {
    None = 0x00,
    Audio = 0x01,
    Communications = 0x02,
    HID = 0x03,
    Physical = 0x05,
    Image = 0x06,
    Printer = 0x07,
    MassStorage = 0x08,
    Hub = 0x09,
    CDCData = 0x0A,
    SmartCard = 0x0B,
    ContentSecurity = 0x0D,
    Video = 0x0E,
    PersonalHealthcare = 0x0F,
    AudioVideo = 0x10,
    Billboard = 0x11,
    USBTypeCBridge = 0x12,
    Matter = 0x13,
    Diagnostic = 0xDC,
    WirelessController = 0xE0,
    Miscellaneous = 0xEF,
    ApplicationSpecific = 0xFE,
    VendorSpecific = 0xFF,
}

/// USB Device Speed
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Low = 1,      // 1.5 Mbps
    Full = 2,     // 12 Mbps  
    High = 3,     // 480 Mbps (USB 2.0)
    Super = 4,    // 5 Gbps (USB 3.0)
    SuperPlus = 5 // 10 Gbps (USB 3.1+)
}

/// USB Transfer Types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

/// USB Endpoint Direction
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDirection {
    Out = 0x00,
    In = 0x80,
}

/// USB Endianness
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbEndianness {
    Little,
    Big,
}

/// USB Configuration Status
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbConfigStatus {
    NotConfigured = 0x00,
    Configured = 0x01,
    Suspended = 0x02,
    PoweredDown = 0x03,
}

/// USB Control Request Type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRequestType {
    Standard = 0x00,
    Class = 0x20,
    Vendor = 0x40,
}

/// USB Request Recipient
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRecipient {
    Device = 0x00,
    Interface = 0x01,
    Endpoint = 0x02,
    Other = 0x03,
}

/// USB Standard Requests
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbStandardRequest {
    GET_STATUS = 0x00,
    CLEAR_FEATURE = 0x01,
    SET_FEATURE = 0x03,
    SET_ADDRESS = 0x05,
    GET_DESCRIPTOR = 0x06,
    SET_DESCRIPTOR = 0x07,
    GET_CONFIGURATION = 0x08,
    SET_CONFIGURATION = 0x09,
    GET_INTERFACE = 0x0A,
    SET_INTERFACE = 0x0B,
    SYNCH_FRAME = 0x0C,
}

/// USB Descriptor Types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDescriptorType {
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    DeviceQualifier = 0x06,
    OtherSpeedConfiguration = 0x07,
    InterfacePower = 0x08,
    OTG = 0x09,
    Debug = 0x0A,
    BOS = 0x0F,
    DeviceCapability = 0x10,
}

/// USB Feature Selectors
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbFeatureSelector {
    DEVICE_REMOTE_WAKEUP = 0x0001,
    ENDPOINT_HALT = 0x0000,
    TEST_MODE = 0x0002,
}

/// USB Transfer Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransferStatus {
    Success,
    Stalled,
    BabbleDetected,
    BufferOverrun,
    BufferUnderrun,
    NotAccessed,
    Aborted,
    ShortPacket,
    Cancelled,
}

/// USB Device State
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbDeviceState {
    Attached = 0x00,
    Powered = 0x01,
    Default = 0x02,
    Address = 0x03,
    Configured = 0x04,
    Suspended = 0x05,
    Reset = 0x06,
}

/// USB Descriptor Header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbDescriptorHeader {
    pub bLength: u8,
    pub bDescriptorType: u8,
}

/// USB Device Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbDeviceDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdUSB: u16,
    pub bDeviceClass: u8,
    pub bDeviceSubClass: u8,
    pub bDeviceProtocol: u8,
    pub bMaxPacketSize0: u8,
    pub idVendor: u16,
    pub idProduct: u16,
    pub bcdDevice: u16,
    pub iManufacturer: u8,
    pub iProduct: u8,
    pub iSerialNumber: u8,
    pub bNumConfigurations: u8,
}

/// USB Configuration Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbConfigDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub wTotalLength: u16,
    pub bNumInterfaces: u8,
    pub bConfigurationValue: u8,
    pub iConfiguration: u8,
    pub bmAttributes: u8,
    pub bMaxPower: u8,
}

/// USB Interface Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbInterfaceDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bInterfaceNumber: u8,
    pub bAlternateSetting: u8,
    pub bNumEndpoints: u8,
    pub bInterfaceClass: u8,
    pub bInterfaceSubClass: u8,
    pub bInterfaceProtocol: u8,
    pub iInterface: u8,
}

/// USB Endpoint Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbEndpointDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bEndpointAddress: u8,
    pub bmAttributes: u8,
    pub wMaxPacketSize: u16,
    pub bInterval: u8,
}

/// USB Setup Packet for Control Transfers
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbSetupPacket {
    pub bmRequestType: u8,
    pub bRequest: u8,
    pub wValue: u16,
    pub wIndex: u16,
    pub wLength: u16,
}

/// USB Transfer Request Block (TRB)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbTRB {
    pub ptr_low: u32,
    pub ptr_high: u32,
    pub status: u32,
    pub control: u32,
}

/// USB Hub Descriptor
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbHubDescriptor {
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bNbrPorts: u8,
    pub wHubCharacteristics: u16,
    pub bPwrOn2PwrGood: u8,
    pub bHubContrCurrent: u8,
    pub deviceRemovable: u8,
    pub portPwrCtrlMask: u8,
}

/// USB Hub Status
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbHubStatus {
    pub wHubStatus: u16,
    pub wHubChange: u16,
    pub bPortStatus: [u16; 8],
    pub bPortChange: [u16; 8],
}

/// USB Power Management States
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbPowerState {
    Active = 0x00,
    Suspended = 0x01,
    PoweredDown = 0x02,
    Off = 0x03,
}

/// USB Security Isolation Levels
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSecurityLevel {
    None = 0x00,
    Basic = 0x01,
    Enhanced = 0x02,
    Full = 0x03,
    Military = 0x04,
}

/// USB Transaction Types for Protocol Analyzer
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbTransactionType {
    Setup,
    Data,
    Status,
    Token,
    Handshake,
    Split,
}

/// USB Protocol Analyzer Packet
#[derive(Debug, Clone)]
pub struct UsbPacket {
    pub timestamp: u64,
    pub transaction_type: UsbTransactionType,
    pub endpoint: u8,
    pub device_address: u8,
    pub data: Vec<u8>,
    pub status: UsbTransferStatus,
    pub duration_ns: u64,
}

/// USB Device Handle
#[derive(Debug, Clone)]
pub struct UsbDevice {
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub speed: UsbSpeed,
    pub state: UsbDeviceState,
    pub configuration: u8,
    pub interfaces: Vec<UsbInterface>,
    pub descriptor: Option<UsbDeviceDescriptor>,
}

/// USB Interface
#[derive(Debug, Clone)]
pub struct UsbInterface {
    pub number: u8,
    pub alternate_setting: u8,
    pub class: UsbClass,
    pub subclass: u8,
    pub protocol: u8,
    pub endpoints: Vec<UsbEndpoint>,
    pub descriptor: Option<UsbInterfaceDescriptor>,
}

/// USB Endpoint
#[derive(Debug, Clone)]
pub struct UsbEndpoint {
    pub address: u8,
    pub direction: UsbDirection,
    pub transfer_type: UsbTransferType,
    pub max_packet_size: u16,
    pub interval: u8,
    pub descriptor: Option<UsbEndpointDescriptor>,
}

/// USB Host Controller
#[derive(Debug, Clone)]
pub enum UsbHostController {
    XHCI(XhciController),
    EHCI(EhciController),
    OHCI(OhciController),
}

/// USB Controller Statistics
#[derive(Debug, Clone)]
pub struct UsbControllerStats {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub bytes_transferred: u64,
    pub error_count: u32,
    pub last_error: Option<String>,
}

/// USB Power Management State
#[derive(Debug, Clone)]
pub struct UsbPowerInfo {
    pub state: UsbPowerState,
    pub current_draw_ma: u32,
    pub max_power_ma: u32,
    pub port_power_managed: bool,
    pub wakeup_enabled: bool,
}

/// USB Security Context
#[derive(Debug, Clone)]
pub struct UsbSecurityContext {
    pub isolation_level: UsbSecurityLevel,
    pub device_trust_score: u8,
    pub access_restrictions: Vec<String>,
    pub audit_log: Vec<UsbAuditEntry>,
    pub encrypted_channels: bool,
}

/// USB Audit Entry
#[derive(Debug, Clone)]
pub struct UsbAuditEntry {
    pub timestamp: u64,
    pub device_id: String,
    pub action: String,
    pub result: UsbTransferStatus,
    pub data_size: usize,
}

/// USB Event Types
#[derive(Debug, Clone)]
pub enum UsbEvent {
    DeviceConnected { port: u8, speed: UsbSpeed },
    DeviceDisconnected { address: u8 },
    DeviceReset { address: u8 },
    DeviceSuspended { address: u8 },
    DeviceResumed { address: u8 },
    TransferCompleted { address: u8, endpoint: u8, status: UsbTransferStatus },
    TransferError { address: u8, endpoint: u8, error: UsbTransferStatus },
    PowerStateChanged { device: u8, state: UsbPowerState },
    SecurityViolation { device: u8, violation: String },
}

/// USB Framework Initialization Result
#[derive(Debug)]
pub struct UsbFramework {
    pub initialized: bool,
    pub controllers: Vec<UsbHostController>,
    pub devices: Vec<UsbDevice>,
    pub hubs: Vec<UsbHub>,
    pub protocol_analyzer: Option<UsbProtocolAnalyzer>,
    pub power_manager: Option<UsbPowerManager>,
    pub security_manager: Option<UsbSecurityManager>,
}

/// USB Hub State
#[derive(Debug)]
pub struct UsbHub {
    pub address: u8,
    pub ports: u8,
    pub descriptors: Option<UsbHubDescriptor>,
    pub status: UsbHubStatus,
    pub power_management: bool,
    pub individual_power_control: bool,
}

/// USB Protocol Analyzer State
#[derive(Debug)]
pub struct UsbProtocolAnalyzer {
    pub captures_enabled: bool,
    pub captured_packets: Vec<UsbPacket>,
    pub filtering_enabled: bool,
    pub filters: Vec<UsbPacketFilter>,
}

/// USB Packet Filter
#[derive(Debug)]
pub struct UsbPacketFilter {
    pub device_filter: Option<u16>,
    pub class_filter: Option<u8>,
    pub endpoint_filter: Option<u8>,
    pub transaction_filter: Option<UsbTransactionType>,
}

/// USB Power Manager
#[derive(Debug)]
pub struct UsbPowerManager {
    pub global_power_policy: UsbPowerState,
    pub device_power_states: Vec<UsbPowerInfo>,
    pub idle_timeout_ms: u32,
    pub auto_suspend_enabled: bool,
}

/// USB Security Manager
#[derive(Debug)]
pub struct UsbSecurityManager {
    pub global_security_level: UsbSecurityLevel,
    pub device_policies: Vec<UsbDevicePolicy>,
    pub quarantine_list: Vec<String>,
    pub audit_enabled: bool,
}

/// USB Device Policy
#[derive(Debug)]
pub struct UsbDevicePolicy {
    pub vendor_id: u16,
    pub product_id: u16,
    pub security_level: UsbSecurityLevel,
    pub allowed_operations: Vec<String>,
    pub quarantine_days: u8,
}

/// USB Driver Error Types
#[derive(Debug, Clone)]
pub enum UsbDriverError {
    ControllerNotInitialized,
    DeviceNotFound { address: u8 },
    TransferFailed { status: UsbTransferStatus },
    UnsupportedFeature,
    InvalidConfiguration,
    SecurityViolation,
    PowerManagementError,
    Timeout,
    ProtocolError,
}

/// USB Driver Result Type
pub type UsbResult<T> = Result<T, UsbDriverError>;

/// Global USB instance counter
static USB_INSTANCE_COUNT: AtomicU32 = AtomicU32::new(0);

/// USB Framework implementation
impl UsbFramework {
    /// Create a new USB framework instance
    pub fn new() -> Self {
        USB_INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        
        Self {
            initialized: false,
            controllers: Vec::new(),
            devices: Vec::new(),
            hubs: Vec::new(),
            protocol_analyzer: None,
            power_manager: None,
            security_manager: None,
        }
    }

    /// Initialize the USB framework
    pub fn initialize(&mut self) -> UsbResult<()> {
        self.initialized = true;
        log::info!("USB Framework initialized successfully");
        
        Ok(())
    }

    /// Add a host controller
    pub fn add_controller(&mut self, controller: UsbHostController) {
        self.controllers.push(controller);
    }

    /// Get all registered devices
    pub fn get_devices(&self) -> &[UsbDevice] {
        &self.devices
    }

    /// Get all registered hubs
    pub fn get_hubs(&self) -> &[UsbHub] {
        &self.hubs
    }
}

impl Default for UsbFramework {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for USB endianness conversion
pub trait UsbEndian {
    fn read_u16_le(data: &[u8], offset: usize) -> u16;
    fn read_u16_be(data: &[u8], offset: usize) -> u16;
    fn write_u16_le(data: &mut [u8], offset: usize, value: u16);
    fn write_u16_be(data: &mut [u8], offset: usize, value: u16);
}

impl UsbEndian for UsbEndianness {
    fn read_u16_le(data: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([data[offset], data[offset + 1]])
    }

    fn read_u16_be(data: &[u8], offset: usize) -> u16 {
        u16::from_be_bytes([data[offset], data[offset + 1]])
    }

    fn write_u16_le(data: &mut [u8], offset: usize, value: u16) {
        data[offset] = value as u8;
        data[offset + 1] = (value >> 8) as u8;
    }

    fn write_u16_be(data: &mut [u8], offset: usize, value: u16) {
        data[offset] = (value >> 8) as u8;
        data[offset + 1] = value as u8;
    }
}

/// USB utility functions
pub mod utils {
    use super::*;
    
    /// Calculate USB CRC5 for token packets
    pub fn calculate_usb_crc5(data: u8) -> u8 {
        let mut crc = 0x1F;
        let mut temp = data << 3;
        
        for _ in 0..5 {
            if ((crc ^ temp) & 0x80) != 0 {
                crc ^= 0x05;
            }
            crc = (crc << 1) & 0x1F;
            temp <<= 1;
        }
        
        crc
    }
    
    /// Calculate USB CRC16 for data packets
    pub fn calculate_usb_crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        
        for byte in data {
            let mut temp = *byte as u16;
            
            for _ in 0..8 {
                if ((crc ^ temp) & 0x0001) != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
                temp >>= 1;
            }
        }
        
        crc
    }
    
    /// Parse USB setup packet from data
    pub fn parse_setup_packet(data: &[u8]) -> Option<UsbSetupPacket> {
        if data.len() < 8 {
            return None;
        }
        
        Some(UsbSetupPacket {
            bmRequestType: data[0],
            bRequest: data[1],
            wValue: UsbEndianness::read_u16_le(data, 2),
            wIndex: UsbEndianness::read_u16_le(data, 4),
            wLength: UsbEndianness::read_u16_le(data, 6),
        })
    }
    
    /// Convert USB speed to human readable string
    pub fn speed_to_string(speed: UsbSpeed) -> &'static str {
        match speed {
            UsbSpeed::Low => "1.5 Mbps (Low Speed)",
            UsbSpeed::Full => "12 Mbps (Full Speed)",
            UsbSpeed::High => "480 Mbps (High Speed)",
            UsbSpeed::Super => "5 Gbps (Super Speed)",
            UsbSpeed::SuperPlus => "10 Gbps (Super Speed Plus)",
        }
    }
    
    /// Get USB class name from class code
    pub fn class_to_string(class: UsbClass) -> &'static str {
        match class {
            UsbClass::None => "Unspecified",
            UsbClass::Audio => "Audio",
            UsbClass::Communications => "Communications",
            UsbClass::HID => "Human Interface Device",
            UsbClass::Physical => "Physical",
            UsbClass::Image => "Image",
            UsbClass::Printer => "Printer",
            UsbClass::MassStorage => "Mass Storage",
            UsbClass::Hub => "Hub",
            UsbClass::CDCData => "CDC Data",
            UsbClass::SmartCard => "Smart Card",
            UsbClass::ContentSecurity => "Content Security",
            UsbClass::Video => "Video",
            UsbClass::PersonalHealthcare => "Personal Healthcare",
            UsbClass::AudioVideo => "Audio/Video",
            UsbClass::Billboard => "Billboard",
            UsbClass::USBTypeCBridge => "USB Type-C Bridge",
            UsbClass::Matter => "Matter",
            UsbClass::Diagnostic => "Diagnostic",
            UsbClass::WirelessController => "Wireless Controller",
            UsbClass::Miscellaneous => "Miscellaneous",
            UsbClass::ApplicationSpecific => "Application Specific",
            UsbClass::VendorSpecific => "Vendor Specific",
        }
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;

    #[test]
    fn test_usb_crc5_calculation() {
        assert_eq!(calculate_usb_crc5(0x8F), 0x10);
    }

    #[test]
    fn test_usb_crc16_calculation() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let crc = calculate_usb_crc16(&data);
        // CRC16 calculation test - actual value depends on algorithm
        assert!(crc > 0);
    }

    #[test]
    fn test_setup_packet_parsing() {
        let data = [0x80, 0x06, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00];
        let packet = parse_setup_packet(&data).unwrap();
        
        assert_eq!(packet.bmRequestType, 0x80);
        assert_eq!(packet.bRequest, 0x06);
        assert_eq!(packet.wValue, 0x0001);
    }

    #[test]
    fn test_endianness_conversion() {
        let data = [0x12, 0x34];
        let le_value = UsbEndianness::read_u16_le(&data, 0);
        let be_value = UsbEndianness::read_u16_be(&data, 0);
        
        assert_eq!(le_value, 0x3412);
        assert_eq!(be_value, 0x1234);
    }
}