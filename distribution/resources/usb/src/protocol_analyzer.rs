//! Educational USB Protocol Analyzer
//! 
//! This module provides comprehensive USB protocol analysis and educational tools,
//! including packet capture, descriptor decoding, protocol visualization,
//! and educational explanations of USB concepts.

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use crate::UsbResult;

/// USB transfer types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferType {
    Control,
    Interrupt,
    Isochronous,
    Bulk,
}

impl fmt::Display for TransferType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferType::Control => write!(f, "Control"),
            TransferType::Interrupt => write!(f, "Interrupt"),
            TransferType::Isochronous => write!(f, "Isochronous"),
            TransferType::Bulk => write!(f, "Bulk"),
        }
    }
}

/// USB packet types
#[derive(Debug, Clone)]
pub enum UsbPacket {
    Setup {
        setup: SetupPacket,
    },
    Data {
        data: DataPacket,
    },
    Handshake {
        handshake: HandshakePacket,
    },
    Sof {
        frame_number: u16,
    },
    Token {
        token: TokenPacket,
    },
    Custom {
        packet_type: String,
        data: Vec<u8>,
    },
}

/// USB SETUP packet
#[derive(Debug, Clone)]
pub struct SetupPacket {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

/// USB DATA packet
#[derive(Debug, Clone)]
pub struct DataPacket {
    pub data: Vec<u8>,
    pub pid: u8, // Packet ID
}

/// USB HANDSHAKE packet
#[derive(Debug, Clone)]
pub struct HandshakePacket {
    pub pid: u8, // Packet ID
}

/// USB SOF packet
#[derive(Debug, Clone)]
pub struct SofPacket {
    pub frame_number: u16,
}

/// USB TOKEN packet
#[derive(Debug, Clone)]
pub struct TokenPacket {
    pub pid: u8,
    pub address: u8,
    pub endpoint: u8,
}

/// USB transaction
#[derive(Debug, Clone)]
pub struct UsbTransaction {
    pub timestamp: u64,
    pub device_address: u8,
    pub endpoint_number: u8,
    pub transfer_type: TransferType,
    pub packets: Vec<UsbPacket>,
    pub duration_ns: u64,
}

/// USB transfer (multiple transactions)
#[derive(Debug, Clone)]
pub struct UsbTransfer {
    pub timestamp: u64,
    pub device_address: u8,
    pub endpoint_number: u8,
    pub transfer_type: TransferType,
    pub transactions: Vec<UsbTransaction>,
    pub total_duration_ns: u64,
}

/// USB descriptor types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    Device,
    Configuration,
    String,
    Interface,
    Endpoint,
    DeviceQualifier,
    OtherSpeedConfiguration,
    InterfacePower,
    Otg,
    Debug,
    Bcd,
    Fc,
    SsEndpointCompanion,
    SsIsochEndpointCompanion,
    Unknown(u8),
}

impl DescriptorType {
    /// Parse descriptor type from value
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => DescriptorType::Device,
            0x02 => DescriptorType::Configuration,
            0x03 => DescriptorType::String,
            0x04 => DescriptorType::Interface,
            0x05 => DescriptorType::Endpoint,
            0x06 => DescriptorType::DeviceQualifier,
            0x07 => DescriptorType::OtherSpeedConfiguration,
            0x08 => DescriptorType::InterfacePower,
            0x09 => DescriptorType::Otg,
            0x0A => DescriptorType::Debug,
            0x0B => DescriptorType::Bcd,
            0x0C => DescriptorType::Fc,
            0x0D => DescriptorType::SsEndpointCompanion,
            0x0E => DescriptorType::SsIsochEndpointCompanion,
            other => DescriptorType::Unknown(other),
        }
    }
}

/// USB request types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    Standard,
    Class,
    Vendor,
    Reserved,
}

impl RequestType {
    /// Parse request type from request_type field
    pub fn from_u8(request_type: u8) -> Self {
        match (request_type >> 5) & 0x03 {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            3 => RequestType::Reserved,
            _ => RequestType::Reserved,
        }
    }

    /// Get request recipient
    pub fn recipient(request_type: u8) -> String {
        match request_type & 0x1F {
            0 => "Device".to_string(),
            1 => "Interface".to_string(),
            2 => "Endpoint".to_string(),
            3 => "Other".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Get request direction
    pub fn direction(request_type: u8) -> String {
        if request_type & 0x80 == 0 {
            "Host to Device".to_string()
        } else {
            "Device to Host".to_string()
        }
    }
}

/// USB standard requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardRequest {
    GetStatus,
    ClearFeature,
    SetFeature,
    SetAddress,
    GetDescriptor,
    SetDescriptor,
    GetConfiguration,
    SetConfiguration,
    GetInterface,
    SetInterface,
    SynchFrame,
    Unknown(u8),
}

impl StandardRequest {
    /// Parse standard request from request field
    pub fn from_u8(request: u8) -> Self {
        match request {
            0x00 => StandardRequest::GetStatus,
            0x01 => StandardRequest::ClearFeature,
            0x03 => StandardRequest::SetFeature,
            0x05 => StandardRequest::SetAddress,
            0x06 => StandardRequest::GetDescriptor,
            0x07 => StandardRequest::SetDescriptor,
            0x08 => StandardRequest::GetConfiguration,
            0x09 => StandardRequest::SetConfiguration,
            0x0A => StandardRequest::GetInterface,
            0x0B => StandardRequest::SetInterface,
            0x0C => StandardRequest::SynchFrame,
            other => StandardRequest::Unknown(other),
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            StandardRequest::GetStatus => "GET_STATUS - Get device status",
            StandardRequest::ClearFeature => "CLEAR_FEATURE - Clear a feature",
            StandardRequest::SetFeature => "SET_FEATURE - Set a feature",
            StandardRequest::SetAddress => "SET_ADDRESS - Set device address",
            StandardRequest::GetDescriptor => "GET_DESCRIPTOR - Get descriptor",
            StandardRequest::SetDescriptor => "SET_DESCRIPTOR - Set descriptor",
            StandardRequest::GetConfiguration => "GET_CONFIGURATION - Get current configuration",
            StandardRequest::SetConfiguration => "SET_CONFIGURATION - Set configuration",
            StandardRequest::GetInterface => "GET_INTERFACE - Get alternate interface",
            StandardRequest::SetInterface => "SET_INTERFACE - Set alternate interface",
            StandardRequest::SynchFrame => "SYNCH_FRAME - Set synchronization frame",
            StandardRequest::Unknown(_) => "UNKNOWN_REQUEST - Unknown request",
        }
    }
}

/// USB feature selectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureSelector {
    DeviceRemoteWakeup,
    EndpointHalt,
    TestMode,
    Unknown(u16),
}

impl FeatureSelector {
    /// Parse feature selector from value field
    pub fn from_u16(value: u16, recipient: &str) -> Self {
        match recipient {
            "Device" => {
                match value {
                    0x01 => FeatureSelector::DeviceRemoteWakeup,
                    0x02 => FeatureSelector::TestMode,
                    other => FeatureSelector::Unknown(other),
                }
            }
            "Endpoint" => {
                match value {
                    0x00 => FeatureSelector::EndpointHalt,
                    other => FeatureSelector::Unknown(other),
                }
            }
            _ => FeatureSelector::Unknown(value),
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            FeatureSelector::DeviceRemoteWakeup => "DEVICE_REMOTE_WAKEUP - Enable/disable remote wakeup",
            FeatureSelector::EndpointHalt => "ENDPOINT_HALT - Set/Clear halt feature",
            FeatureSelector::TestMode => "TEST_MODE - Enter test mode",
            FeatureSelector::Unknown(_) => "UNKNOWN_FEATURE - Unknown feature selector",
        }
    }
}

/// USB descriptor decoder
pub struct DescriptorDecoder {
    /// Enable educational explanations
    enable_education: bool,
}

impl DescriptorDecoder {
    /// Create a new descriptor decoder
    pub fn new() -> Self {
        Self {
            enable_education: true,
        }
    }

    /// Create descriptor decoder with educational mode
    pub fn with_education(enable: bool) -> Self {
        Self {
            enable_education: enable,
        }
    }

    /// Enable/disable educational mode
    pub fn set_educational_mode(&mut self, enable: bool) {
        self.enable_education = enable;
    }

    /// Decode USB descriptor data
    pub fn decode_descriptor(&self, descriptor_data: &[u8]) -> UsbResult<String> {
        if descriptor_data.len() < 2 {
            return Err(UsbResult::InvalidBuffer);
        }

        let descriptor_type = DescriptorType::from_u8(descriptor_data[1]);
        let descriptor_length = descriptor_data[0] as usize;

        if descriptor_data.len() < descriptor_length {
            return Err(UsbResult::InvalidBuffer);
        }

        let mut result = String::new();
        
        match descriptor_type {
            DescriptorType::Device => {
                result.push_str(&self.decode_device_descriptor(&descriptor_data[0..descriptor_length]));
            }
            DescriptorType::Configuration => {
                result.push_str(&self.decode_configuration_descriptor(&descriptor_data[0..descriptor_length]));
            }
            DescriptorType::Interface => {
                result.push_str(&self.decode_interface_descriptor(&descriptor_data[0..descriptor_length]));
            }
            DescriptorType::Endpoint => {
                result.push_str(&self.decode_endpoint_descriptor(&descriptor_data[0..descriptor_length]));
            }
            DescriptorType::String => {
                result.push_str(&self.decode_string_descriptor(&descriptor_data[0..descriptor_length]));
            }
            _ => {
                result.push_str(&format!("Unknown descriptor type: {:?}", descriptor_type));
            }
        }

        if self.enable_education {
            result.push_str("\n\nEducational Note:\n");
            result.push_str(&self.get_educational_info(&descriptor_type));
        }

        Ok(result)
    }

    /// Decode device descriptor
    fn decode_device_descriptor(&self, data: &[u8]) -> String {
        if data.len() < 18 {
            return "Invalid device descriptor (too short)".to_string();
        }

        let mut result = String::new();
        result.push_str("USB Device Descriptor\n");
        result.push_str("====================\n\n");

        // bcdUSB
        let bcd_usb = ((data[2] as u16) << 8) | data[3] as u16;
        let usb_version = format!("{}.{:02}", (bcd_usb >> 8) & 0xFF, (bcd_usb >> 4) & 0xF);
        result.push_str(&format!("bcdUSB: {}\n", usb_version));

        // bDeviceClass
        let device_class = data[4];
        result.push_str(&format!("bDeviceClass: 0x{:02X} ", device_class));
        result.push_str(&match device_class {
            0x00 => "(Defined at interface level)",
            0x01 => "(Audio class)",
            0x02 => "(Communications class)",
            0x03 => "(HID class)",
            0x08 => "(Mass storage class)",
            0x09 => "(Hub class)",
            0x0E => "(Video class)",
            0x0D => "(Content security class)",
            0x11 => "(Billboard class)",
            other => &format!("(Reserved/Unknown class: 0x{:02X})", other),
        });
        result.push_str("\n");

        // bDeviceSubClass
        let device_subclass = data[5];
        result.push_str(&format!("bDeviceSubClass: 0x{:02X}\n", device_subclass));

        // bDeviceProtocol
        let device_protocol = data[6];
        result.push_str(&format!("bDeviceProtocol: 0x{:02X}\n", device_protocol));

        // bMaxPacketSize0
        let max_packet_size = data[7];
        result.push_str(&format!("bMaxPacketSize0: {} bytes\n", max_packet_size));

        // idVendor
        let vendor_id = ((data[8] as u16) << 8) | data[9] as u16;
        result.push_str(&format!("idVendor: 0x{:04X}\n", vendor_id));

        // idProduct
        let product_id = ((data[10] as u16) << 8) | data[11] as u16;
        result.push_str(&format!("idProduct: 0x{:04X}\n", product_id));

        // bcdDevice
        let bcd_device = ((data[12] as u16) << 8) | data[13] as u16;
        let device_version = format!("{}.{:02}", (bcd_device >> 8) & 0xFF, (bcd_device >> 4) & 0xF);
        result.push_str(&format!("bcdDevice: {}\n", device_version));

        // iManufacturer
        let manufacturer_index = data[14];
        result.push_str(&format!("iManufacturer: {}\n", manufacturer_index));

        // iProduct
        let product_index = data[15];
        result.push_str(&format!("iProduct: {}\n", product_index));

        // iSerialNumber
        let serial_index = data[16];
        result.push_str(&format!("iSerialNumber: {}\n", serial_index));

        // bNumConfigurations
        let num_configurations = data[17];
        result.push_str(&format!("bNumConfigurations: {}\n", num_configurations));

        result
    }

    /// Decode configuration descriptor
    fn decode_configuration_descriptor(&self, data: &[u8]) -> String {
        if data.len() < 9 {
            return "Invalid configuration descriptor (too short)".to_string();
        }

        let mut result = String::new();
        result.push_str("USB Configuration Descriptor\n");
        result.push_str("============================\n\n");

        // wTotalLength
        let total_length = ((data[2] as u16) << 8) | data[3] as u16;
        result.push_str(&format!("wTotalLength: {} bytes\n", total_length));

        // bNumInterfaces
        let num_interfaces = data[4];
        result.push_str(&format!("bNumInterfaces: {}\n", num_interfaces));

        // bConfigurationValue
        let config_value = data[5];
        result.push_str(&format!("bConfigurationValue: {}\n", config_value));

        // iConfiguration
        let config_index = data[6];
        result.push_str(&format!("iConfiguration: {}\n", config_index));

        // bmAttributes
        let attributes = data[7];
        result.push_str(&format!("bmAttributes: 0x{:02X}\n", attributes));
        if attributes & 0x80 != 0 {
            result.push_str("  - Bus powered\n");
        }
        if attributes & 0x40 != 0 {
            result.push_str("  - Self powered\n");
        }
        if attributes & 0x20 != 0 {
            result.push_str("  - Remote wakeup capable\n");
        }

        // bMaxPower
        let max_power = data[8] * 2; // Value is in 2mA units
        result.push_str(&format!("bMaxPower: {}mA\n", max_power));

        result
    }

    /// Decode interface descriptor
    fn decode_interface_descriptor(&self, data: &[u8]) -> String {
        if data.len() < 9 {
            return "Invalid interface descriptor (too short)".to_string();
        }

        let mut result = String::new();
        result.push_str("USB Interface Descriptor\n");
        result.push_str("========================\n\n");

        // bInterfaceNumber
        let interface_number = data[2];
        result.push_str(&format!("bInterfaceNumber: {}\n", interface_number));

        // bAlternateSetting
        let alt_setting = data[3];
        result.push_str(&format!("bAlternateSetting: {}\n", alt_setting));

        // bNumEndpoints
        let num_endpoints = data[4];
        result.push_str(&format!("bNumEndpoints: {}\n", num_endpoints));

        // bInterfaceClass
        let interface_class = data[5];
        result.push_str(&format!("bInterfaceClass: 0x{:02X} ", interface_class));
        result.push_str(&match interface_class {
            0x00 => "(Device class)",
            0x01 => "(Audio class)",
            0x02 => "(Communications class)",
            0x03 => "(HID class)",
            0x05 => "(Physical class)",
            0x06 => "(Image class)",
            0x07 => "(Printer class)",
            0x08 => "(Mass storage class)",
            0x09 => "(Hub class)",
            0x0A => "(CDC data class)",
            0x0B => "(Smart card class)",
            0x0D => "(Content security class)",
            0x0E => "(Video class)",
            0x0F => "(Personal healthcare class)",
            0x10 => "(Audio/Video devices)",
            0x11 => "(Billboard class)",
            0x12 => "(Diagnostic class)",
            0x13 => "(Wireless controller class)",
            0x14 => "(Miscellaneous class)",
            0x16 => "(Application specific)",
            0x1B => "(USB Type-C Bridge class)",
            other => &format!("(Vendor specific class: 0x{:02X})", other),
        });
        result.push_str("\n");

        // bInterfaceSubClass
        let interface_subclass = data[6];
        result.push_str(&format!("bInterfaceSubClass: 0x{:02X}\n", interface_subclass));

        // bInterfaceProtocol
        let interface_protocol = data[7];
        result.push_str(&format!("bInterfaceProtocol: 0x{:02X}\n", interface_protocol));

        // iInterface
        let interface_index = data[8];
        result.push_str(&format!("iInterface: {}\n", interface_index));

        result
    }

    /// Decode endpoint descriptor
    fn decode_endpoint_descriptor(&self, data: &[u8]) -> String {
        if data.len() < 7 {
            return "Invalid endpoint descriptor (too short)".to_string();
        }

        let mut result = String::new();
        result.push_str("USB Endpoint Descriptor\n");
        result.push_str("=======================\n\n");

        // bEndpointAddress
        let endpoint_address = data[2];
        let endpoint_num = endpoint_address & 0x0F;
        let direction = if endpoint_address & 0x80 != 0 { "IN" } else { "OUT" };
        result.push_str(&format!("bEndpointAddress: 0x{:02X} ({}/EP{})\n", 
            endpoint_address, direction, endpoint_num));

        // bmAttributes
        let attributes = data[3];
        let transfer_type = attributes & 0x03;
        result.push_str(&format!("bmAttributes: 0x{:02X} ", attributes));
        result.push_str(&match transfer_type {
            0x00 => "(Control transfer)",
            0x01 => "(Isochronous transfer)",
            0x02 => "(Bulk transfer)",
            0x03 => "(Interrupt transfer)",
            _ => "(Invalid transfer type)",
        });
        result.push_str("\n");

        // wMaxPacketSize
        let max_packet_size = ((data[4] as u16) << 8) | data[5] as u16;
        let max_packet_value = max_packet_size & 0x07FF;
        let transactions = ((max_packet_size >> 11) & 0x03) + 1;
        result.push_str(&format!("wMaxPacketSize: {} bytes ({} transaction(s) per frame)\n", 
            max_packet_value, transactions));

        // bInterval
        let interval = data[6];
        result.push_str(&format!("bInterval: {} frames", interval));
        result.push_str(&match transfer_type {
            0x01 => " (Isochronous interval)",
            0x03 => " (Interrupt interval)",
            _ => "",
        });
        result.push_str("\n");

        result
    }

    /// Decode string descriptor
    fn decode_string_descriptor(&self, data: &[u8]) -> String {
        if data.len() < 2 {
            return "Invalid string descriptor (too short)".to_string();
        }

        let mut result = String::new();
        result.push_str("USB String Descriptor\n");
        result.push_str("=====================\n\n");

        // Extract Unicode string data
        let mut string_data = Vec::new();
        for i in (2..data.len()).step_by(2) {
            if i + 1 < data.len() {
                let char = (data[i + 1] as u16) << 8 | data[i] as u16;
                string_data.push(char as char);
            }
        }

        let string_value: String = string_data.iter().collect();
        result.push_str(&format!("String Value: \"{}\"\n", string_value));
        result.push_str(&format!("Length: {} characters\n", string_value.len()));

        result
    }

    /// Get educational information about descriptor type
    fn get_educational_info(&self, descriptor_type: &DescriptorType) -> String {
        match descriptor_type {
            DescriptorType::Device => {
                String::from(
                    "Device descriptors contain information about the USB device itself, \
                     including USB version, device class, vendor/product IDs, and \
                     manufacturer information. This is the first descriptor a host \
                     requests when a device is connected.\n\n\
                     Key concepts:\n\
                     • USB version indicates supported features\n\
                     • Device class determines device category\n\
                     • Vendor/Product IDs identify the manufacturer and product\n\
                     • bMaxPacketSize0 sets control endpoint packet size"
                )
            }
            DescriptorType::Configuration => {
                String::from(
                    "Configuration descriptors define how a device is configured, \
                     including power requirements, supported interfaces, and endpoint \
                     layouts. A device can have multiple configurations, but only \
                     one can be active at a time.\n\n\
                     Key concepts:\n\
                     • Configuration value selects which configuration to use\n\
                     • bmAttributes indicate power characteristics\n\
                     • bMaxPower sets maximum current consumption\n\
                     • Multiple interfaces allow device functionality grouping"
                )
            }
            DescriptorType::Interface => {
                String::from(
                    "Interface descriptors define logical functions within a device, \
                     grouping related endpoints into interfaces. Each interface \
                     represents a different aspect of device functionality.\n\n\
                     Key concepts:\n\
                     • Interface numbers are device-specific\n\
                     • Alternate settings provide different configurations\n\
                     • Interface class defines functionality type\n\
                     • Endpoints within an interface work together"
                )
            }
            DescriptorType::Endpoint => {
                String::from(
                    "Endpoint descriptors define the communication channels used by \
                     the device. Endpoints are unidirectional pipes for data transfer, \
                     with endpoint 0 reserved for control transfers.\n\n\
                     Key concepts:\n\
                     • IN endpoints transfer data from device to host\n\
                     • OUT endpoints transfer data from host to device\n\
                     • Transfer types determine bandwidth allocation\n\
                     • wMaxPacketSize sets data chunk size"
                )
            }
            DescriptorType::String => {
                String::from(
                    "String descriptors contain human-readable text information \
                     in Unicode format. They provide manufacturer names, product \
                     names, serial numbers, and other descriptive text.\n\n\
                     Key concepts:\n\
                     • Strings are referenced by index in other descriptors\n\
                     • Unicode encoding allows international character sets\n\
                     • Optional strings can be empty\n\
                     • Multiple language IDs supported"
                )
            }
            _ => {
                "This descriptor type provides additional device functionality \
                 or compatibility information. Refer to USB specification for \
                 details about specific descriptor purposes.".to_string()
            }
        }
    }
}

/// USB protocol analyzer for capturing and analyzing USB traffic
pub struct ProtocolAnalyzer {
    /// Enable capture mode
    capture_enabled: bool,
    /// Captured transactions
    transactions: Vec<UsbTransaction>,
    /// Max transactions to keep
    max_transactions: usize,
    /// Enable educational mode
    educational_mode: bool,
}

impl ProtocolAnalyzer {
    /// Create a new protocol analyzer
    pub fn new() -> Self {
        Self {
            capture_enabled: false,
            transactions: Vec::new(),
            max_transactions: 1000,
            educational_mode: true,
        }
    }

    /// Create protocol analyzer with custom settings
    pub fn new_with_config(
        enable_capture: bool,
        max_transactions: usize,
        educational_mode: bool,
    ) -> Self {
        Self {
            capture_enabled: enable_capture,
            transactions: Vec::with_capacity(max_transactions),
            max_transactions,
            educational_mode,
        }
    }

    /// Start/stop capture mode
    pub fn set_capture_enabled(&mut self, enabled: bool) {
        self.capture_enabled = enabled;
    }

    /// Enable/disable educational mode
    pub fn set_educational_mode(&mut self, enabled: bool) {
        self.educational_mode = enabled;
    }

    /// Add a transaction to the analyzer
    pub fn add_transaction(&mut self, transaction: UsbTransaction) {
        if !self.capture_enabled {
            return;
        }

        self.transactions.push(transaction);

        // Limit transaction count
        if self.transactions.len() > self.max_transactions {
            self.transactions.remove(0);
        }
    }

    /// Get captured transactions
    pub fn get_transactions(&self) -> &[UsbTransaction] {
        &self.transactions
    }

    /// Clear captured transactions
    pub fn clear_transactions(&mut self) {
        self.transactions.clear();
    }

    /// Analyze captured transactions and generate report
    pub fn generate_analysis_report(&self) -> String {
        let mut report = String::new();
        report.push_str("USB Protocol Analysis Report\n");
        report.push_str("============================\n\n");

        report.push_str(&format!("Total transactions captured: {}\n", self.transactions.len()));
        report.push_str(&format!("Capture enabled: {}\n", self.capture_enabled));
        report.push_str(&format!("Educational mode: {}\n\n", self.educational_mode));

        if self.transactions.is_empty() {
            report.push_str("No transactions captured.\n");
            return report;
        }

        // Analyze transaction statistics
        let mut device_stats: alloc::collections::BTreeMap<u8, usize> = alloc::collections::BTreeMap::new();
        let mut transfer_type_stats: alloc::collections::BTreeMap<TransferType, usize> = alloc::collections::BTreeMap::new();

        for transaction in &self.transactions {
            *device_stats.entry(transaction.device_address).or_insert(0) += 1;
            *transfer_type_stats.entry(transaction.transfer_type).or_insert(0) += 1;
        }

        report.push_str("Device Statistics:\n");
        report.push_str("------------------\n");
        for (device_addr, count) in device_stats {
            report.push_str(&format!("Device {}: {} transactions\n", device_addr, count));
        }

        report.push_str("\nTransfer Type Statistics:\n");
        report.push_str("--------------------------\n");
        for (transfer_type, count) in transfer_type_stats {
            report.push_str(&format!("{:?}: {} transactions\n", transfer_type, count));
        }

        // Generate timeline
        report.push_str("\nTransaction Timeline:\n");
        report.push_str("--------------------\n");
        for (i, transaction) in self.transactions.iter().enumerate() {
            report.push_str(&format!("{:4}: Device {} EP{} ({:?}) - {} packets\n", 
                i, transaction.device_address, transaction.endpoint_number,
                transaction.transfer_type, transaction.packets.len()));
        }

        if self.educational_mode {
            report.push_str("\nEducational Notes:\n");
            report.push_str("==================\n");
            report.push_str(&self.get_educational_notes());
        }

        report
    }

    /// Decode SETUP packet
    pub fn decode_setup_packet(&self, setup_data: &[u8]) -> UsbResult<String> {
        if setup_data.len() != 8 {
            return Err(UsbResult::InvalidBuffer);
        }

        let mut result = String::new();
        result.push_str("SETUP Packet Analysis\n");
        result.push_str("=====================\n\n");

        let request_type = setup_data[0];
        let request = setup_data[1];
        let value = ((setup_data[3] as u16) << 8) | setup_data[2] as u16;
        let index = ((setup_data[5] as u16) << 8) | setup_data[4] as u16;
        let length = ((setup_data[7] as u16) << 8) | setup_data[6] as u16;

        // Parse request type
        let req_type = RequestType::from_u8(request_type);
        let recipient = RequestType::recipient(request_type);
        let direction = RequestType::direction(request_type);

        result.push_str(&format!("Request Type: 0x{:02X}\n", request_type));
        result.push_str(&format!("  Type: {:?}\n", req_type));
        result.push_str(&format!("  Recipient: {}\n", recipient));
        result.push_str(&format!("  Direction: {}\n\n", direction));

        // Parse request
        let standard_request = StandardRequest::from_u8(request);
        result.push_str(&format!("Request: 0x{:02X} - {}\n", request, standard_request.description()));

        // Parse value and index based on request type
        match standard_request {
            StandardRequest::GetDescriptor => {
                let descriptor_type = DescriptorType::from_u8((value >> 8) as u8);
                let descriptor_index = value as u8;
                result.push_str(&format!("Value: 0x{:04X} (Type: {:?}, Index: {})\n", 
                    value, descriptor_type, descriptor_index));
                result.push_str("  • Descriptor type determines what descriptor is requested\n");
                result.push_str("  • Descriptor index selects specific descriptor\n\n");
            }
            StandardRequest::SetAddress => {
                result.push_str(&format!("Value: 0x{:04X} (Device Address: {})\n", value, value as u8));
                result.push_str("  • This request assigns an address to the device\n");
                result.push_str("  • Device will use this address for subsequent requests\n\n");
            }
            StandardRequest::SetConfiguration => {
                result.push_str(&format!("Value: 0x{:04X} (Configuration: {})\n", value, value as u8));
                result.push_str("  • This request activates a specific configuration\n");
                result.push_str("  • All interfaces become available after this\n\n");
            }
            StandardRequest::GetStatus | StandardRequest::ClearFeature | StandardRequest::SetFeature => {
                let feature = FeatureSelector::from_u16(value, &recipient);
                result.push_str(&format!("Value: 0x{:04X} - {}\n", value, feature.description()));
                result.push_str("  • Feature selectors modify device behavior\n");
                result.push_str("  • Device/endpoint states can be controlled\n\n");
            }
            _ => {
                result.push_str(&format!("Value: 0x{:04X}\n", value));
            }
        }

        result.push_str(&format!("Index: 0x{:04X}\n", index));
        result.push_str(&format!("Length: {} bytes\n", length));

        if self.educational_mode {
            result.push_str("\nEducational Note:\n");
            result.push_str(&self.get_setup_packet_education(&req_type, &standard_request));
        }

        Ok(result)
    }

    /// Get educational notes for SETUP packet
    fn get_setup_packet_education(&self, req_type: &RequestType, request: &StandardRequest) -> String {
        let mut education = String::new();

        match request {
            StandardRequest::GetDescriptor => {
                education.push_str(
                    "GET_DESCRIPTOR is fundamental to USB enumeration. The host requests \
                     device, configuration, interface, endpoint, and string descriptors \
                     to understand device capabilities. Device descriptors are always \
                     requested first using default address 0.\n\n\
                     Process:\n\
                     1. Host requests device descriptor\n\
                     2. Host sets device address\n\
                     3. Host requests full configuration descriptor\n\
                     4. Host selects configuration\n\
                     5. Device becomes operational"
                );
            }
            StandardRequest::SetAddress => {
                education.push_str(
                    "SET_ADDRESS is a critical step in USB enumeration. Before this, \
                     the device uses address 0 for control transfers. Each device \
                     gets a unique address on the USB bus to distinguish between \
                     multiple connected devices. This address persists until the \
                     device is disconnected."
                );
            }
            StandardRequest::SetConfiguration => {
                education.push_str(
                    "SET_CONFIGURATION activates the device's interfaces and endpoints. \
                     Only after this can the device transfer data. Different \
                     configurations can provide different power consumption levels \
                     or interface combinations."
                );
            }
            StandardRequest::GetStatus => {
                education.push_str(
                    "GET_STATUS returns device or endpoint status information. For \
                     devices, this includes power status and self-powered/bus-powered \
                     state. For endpoints, this indicates halt status."
                );
            }
            _ => {
                education.push_str(
                    "This request is part of USB device control. Standard requests \
                     follow the USB specification and work across all device classes, \
                     while class requests are specific to particular device types."
                );
            }
        }

        education
    }

    /// Get general educational notes
    fn get_educational_notes(&self) -> String {
        String::from(
            "USB Protocol Fundamentals:\n\n\
             Transfer Types:\n\
             • Control transfers: Used for configuration and status\n\
             • Bulk transfers: Large data transfers with error recovery\n\
             • Interrupt transfers: Periodic small data transfers\n\
             • Isochronous transfers: Time-sensitive data with guaranteed bandwidth\n\n\
             USB Transfer Cycle:\n\
             1. Token packet (defines transfer type and target)\n\
             2. Data packet (contains actual data)\n\
             3. Handshake packet (confirms success/failure)\n\n\
             Device Enumeration:\n\
             1. Device reset and detection\n\
             2. Address assignment\n\
             3. Descriptor retrieval\n\
             4. Configuration selection\n\
             5. Device ready for use"
        )
    }

    /// Export captured data to a structured format
    pub fn export_data(&self) -> String {
        let mut export = String::new();
        export.push_str("USB Protocol Capture Export\n");
        export.push_str("============================\n\n");

        export.push_str(&format!("Timestamp: {}\n", "N/A")); // Would use real timestamp in production
        export.push_str(&format!("Total Transactions: {}\n\n", self.transactions.len()));

        for (i, transaction) in self.transactions.iter().enumerate() {
            export.push_str(&format!("Transaction {}\n", i));
            export.push_str(&format!("Device: {}\n", transaction.device_address));
            export.push_str(&format!("Endpoint: {}\n", transaction.endpoint_number));
            export.push_str(&format!("Transfer Type: {:?}\n", transaction.transfer_type));
            export.push_str(&format!("Packets: {}\n", transaction.packets.len()));
            export.push_str(&format!("Duration: {} ns\n\n", transaction.duration_ns));
        }

        export
    }

    /// Generate educational tutorial content
    pub fn generate_tutorial(&self, topic: &str) -> String {
        match topic.to_lowercase().as_str() {
            "enumeration" => self.generate_enumeration_tutorial(),
            "transfer_types" => self.generate_transfer_types_tutorial(),
            "descriptors" => self.generate_descriptors_tutorial(),
            "protocol" => self.generate_protocol_tutorial(),
            _ => self.generate_basic_tutorial(),
        }
    }

    fn generate_enumeration_tutorial(&self) -> String {
        String::from(
            "USB Device Enumeration Tutorial\n\
             ================================\n\n\
             USB enumeration is the process by which a host discovers and \
             configures USB devices. Here's the step-by-step process:\n\n\
             1. Device Detection\n\
             • Device connects to USB port\n\
             • Pull-up resistors on D+ and D- lines indicate presence\n\
             • Host detects device through line state changes\n\n\
             2. Reset and Address Assignment\n\
             • Host sends reset signal to device\n\
             • Device enters default state (address 0)\n\
             • Host assigns unique address to device\n\n\
             3. Descriptor Retrieval\n\
             • Host requests device descriptor\n\
             • Device responds with capabilities\n\
             • Host may request additional descriptors\n\n\
             4. Configuration Selection\n\
             • Host requests configuration descriptor\n\
             • Host selects appropriate configuration\n\
             • Device activates interfaces and endpoints\n\n\
             5. Device Ready\n\
             • Device fully operational\n\
             • Applications can access device functions\n\n\
             Educational Exercise:\n\
             Use a USB analyzer to capture this process and identify each step \
             in the USB traffic. Look for SETUP packets with specific requests \
             like GET_DESCRIPTOR, SET_ADDRESS, and SET_CONFIGURATION."
        )
    }

    fn generate_transfer_types_tutorial(&self) -> String {
        String::from(
            "USB Transfer Types Tutorial\n\
             ===========================\n\n\
             USB supports four different transfer types, each optimized for \
             different purposes:\n\n\
             Control Transfers:\n\
             • Purpose: Device configuration and status\n\
             • Reliability: High (retry on error)\n\
             • Latency: Unspecified\n\
             • Bandwidth: Guaranteed during enumeration\n\
             • Usage: Setup packets, configuration\n\
             • Example: SET_ADDRESS, GET_DESCRIPTOR\n\n\
             Bulk Transfers:\n\
             • Purpose: Large data transfers\n\
             • Reliability: High (retry on error)\n\
             • Latency: No guarantee\n\
             • Bandwidth: Shared, unused bandwidth\n\
             • Usage: File transfers, printers\n\
             • Example: USB mass storage\n\n\
             Interrupt Transfers:\n\
             • Purpose: Periodic small transfers\n\
             • Reliability: High (retry on error)\n\
             • Latency: Maximum service interval guaranteed\n\
             • Bandwidth: Reserved for device\n\
             • Usage: Input devices, status updates\n\
             • Example: USB keyboard, mouse\n\n\
             Isochronous Transfers:\n\
             • Purpose: Time-sensitive data\n\
             • Reliability: No retry (data might be lost)\n\
             • Latency: Guaranteed service rate\n\
             • Bandwidth: Reserved bandwidth\n\
             • Usage: Audio, video streaming\n\
             • Example: USB audio device\n\n\
             Educational Exercise:\n\
             Analyze USB traffic and identify transfer types by examining \
             endpoint characteristics and transfer patterns."
        )
    }

    fn generate_descriptors_tutorial(&self) -> String {
        String::from(
            "USB Descriptors Tutorial\n\
             ========================\n\n\
             USB descriptors are data structures that describe device capabilities \
             and configuration to the host. They form a hierarchical structure:\n\n\
             Device Descriptor (Root Level):\n\
             • Describes device capabilities globally\n\
             • Contains USB version, class, vendor/product IDs\n\
             • Fixed length (18 bytes)\n\n\
             Configuration Descriptors:\n\
             • Describes how device can be configured\n\
             • Contains power requirements and interface count\n\
             • Variable length based on interfaces\n\n\
             Interface Descriptors:\n\
             • Describes logical device function\n\
             • Groups related endpoints\n\
             • Can have multiple alternate settings\n\n\
             Endpoint Descriptors:\n\
             • Describes communication channels\n\
             • Defines transfer type and bandwidth requirements\n\
             • IN/OUT directions and endpoint numbers\n\n\
             String Descriptors:\n\
             • Human-readable text information\n\
             • Unicode encoded\n\
             • Referenced by index from other descriptors\n\n\
             Descriptor Retrieval Process:\n\
             1. GET_DESCRIPTOR (Device) - First request\n\
             2. SET_ADDRESS - Assign unique address\n\
             3. GET_DESCRIPTOR (Configuration) - Get full config\n\
             4. GET_DESCRIPTOR (String) - Get descriptive text\n\n\
             Educational Exercise:\n\
             Parse USB descriptors and create a device capability map. \
             Identify device class, supported interfaces, and power requirements."
        )
    }

    fn generate_protocol_tutorial(&self) -> String {
        String::from(
            "USB Protocol Fundamentals Tutorial\n\
             ==================================\n\n\
             Understanding USB protocol structure and packet types:\n\n\
             USB Packet Types:\n\
             • Token packets: Define transfer type and target\n\
             • Data packets: Carry actual data\n\
             • Handshake packets: Confirm success/failure\n\
             • SOF packets: Start of frame timing\n\n\
             USB Transfer Cycle:\n\
             1. Host sends token packet\n\
             2. Data transfer (if applicable)\n\
             3. Handshake response\n\n\
             USB Frame Structure:\n\
             • Frame starts with SOF packet\n\
             • Frame duration: 1ms (full-speed) or 125µs (high-speed)\n\
             • Multiple transactions per frame\n\n\
             Error Detection and Recovery:\n\
             • CRC on data packets\n\
             • Automatic retry on error\n\
             • Transaction abort on repeated failures\n\n\
             USB Speed Levels:\n\
             • Low-speed: 1.5 Mbps (keyboards, mice)\n\
             • Full-speed: 12 Mbps (most devices)\n\
             • High-speed: 480 Mbps (USB 2.0)\n\
             • SuperSpeed: 5+ Gbps (USB 3.0+)\n\n\
             Educational Exercise:\n\
             Use protocol analyzer to examine real USB traffic. Identify \
             packet types, timing patterns, and error recovery mechanisms."
        )
    }

    fn generate_basic_tutorial(&self) -> String {
        String::from(
            "USB Protocol Analyzer Tutorial\n\
             ==============================\n\n\
             Welcome to USB protocol analysis! This tool helps you understand \
             how USB devices communicate with the host system.\n\n\
             Key Concepts:\n\
             • USB is a serial communication protocol\n\
             • Multiple devices share the same bus\n\
             • Each device gets a unique address\n\
             • Different transfer types for different purposes\n\n\
             Getting Started:\n\
             1. Enable capture mode\n\
             2. Connect and analyze USB devices\n\
             3. Examine protocol sequences\n\
             4. Study device descriptors\n\
             5. Understand transfer patterns\n\n\
             Learning Path:\n\
             • Start with enumeration process\n\
             • Learn transfer types\n\
             • Understand descriptors\n\
             • Study protocol details\n\n\
             Use the generate_tutorial() method with topics:\n\
             • 'enumeration' - Device discovery process\n\
             • 'transfer_types' - USB data transfer methods\n\
             • 'descriptors' - Device capability descriptions\n\
             • 'protocol' - Low-level communication details\n\n\
             Educational Value:\n\
             Understanding USB protocol is essential for:\n\
             • Device driver development\n\
             • Hardware debugging\n\
             • Security analysis\n\
             • Performance optimization"
        )
    }
}

impl Default for ProtocolAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}