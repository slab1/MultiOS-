//! USB CDC (Communications Device Class) Driver
//! 
//! Supports CDC devices like USB modems, network adapters, and other communication devices.
//! Implements various CDC subclasses including ACM (Abstract Control Model) for modems.

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// CDC Subclass Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdcSubclass {
    AbstractControlModel = 0x02,    // ACM - Modems
    DirectLineControlModel = 0x03,  // DLCM - Direct modem control
    TelephoneControlModel = 0x04,   // TCM - Telephone
    MultiChannelControlModel = 0x05, // MCCM - Multi-channel control
    CapiControlModel = 0x06,        // CAPI - ISDN
    EthernetNetworkingControlModel = 0x07, // ENCM - Network adapters
    AtmNetworkingControlModel = 0x08, // ANCM - ATM networking
    WirelessHandsetControlModel = 0x09, // WHCM - Wireless
    DeviceManagement = 0x0A,        // MDLM - Device management
    MobileBroadbandInterfaceModel = 0x0B, // MBIM - Mobile broadband
    VendorSpecific = 0xFF,          // VSM - Vendor specific
    Unknown = 0x00,                 // Unknown
}

/// CDC Protocol Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdcProtocol {
    None = 0x00,
    V25ter = 0x01,                 // V.25ter (AT commands)
    ISO14443 = 0x02,               // ISO 14443 Type A/B
    VendorSpecific = 0xFF,         // Vendor specific protocol
}

/// CDC Interface Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdcInterfaceType {
    Communication = 0,
    Data = 1,
    Unknown,
}

/// CDC Control Requests
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdcControlRequest {
    SendEncapsulatedCommand = 0x00,
    GetEncapsulatedCommand = 0x01,
    SetCommunicationInterface = 0x02,
    GetCommunicationInterface = 0x03,
    SetLineCoding = 0x20,
    GetLineCoding = 0x21,
    SetControlLineState = 0x22,
    SendBreak = 0x23,
    SetNetworkConnection = 0x24,
    ResponseAvailable = 0x25,
    NetworkConnection = 0x2A,
    GenericNetworkControl = 0xFF,
}

/// CDC Notification Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdcNotification {
    ResponseAvailable = 0x01,
    NetworkConnection = 0x2A,
    ConnectionSpeedChange = 0x2B,
    Unknown = 0x00,
}

/// CDC Line Coding Structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CdcLineCoding {
    pub dwDTERate: u32,            // Data terminal rate in bits per second
    pub bCharFormat: u8,           // Stop bits (0=1, 1=1.5, 2=2)
    pub bParityType: u8,           // Parity (0=none, 1=odd, 2=even, 3=mark, 4=space)
    pub bDataBits: u8,             // Number of data bits (5-8)
}

/// CDC Line Coding Stop Bits
pub const CDC_STOP_BITS_1: u8 = 0;
pub const CDC_STOP_BITS_1_5: u8 = 1;
pub const CDC_STOP_BITS_2: u8 = 2;

/// CDC Line Coding Parity Types
pub const CDC_PARITY_NONE: u8 = 0;
pub const CDC_PARITY_ODD: u8 = 1;
pub const CDC_PARITY_EVEN: u8 = 2;
pub const CDC_PARITY_MARK: u8 = 3;
pub const CDC_PARITY_SPACE: u8 = 4;

/// CDC Control Line State
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CdcControlLineState {
    pub dtr_present: bool,         // DTR (Data Terminal Ready)
    pub rts_present: bool,         // RTS (Request To Send)
}

/// CDC Notification Structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CdcNotificationData {
    pub bmRequestType: u8,         // Request type
    pub bNotification: u8,         // Notification type
    pub wValue: u16,               // Value
    pub wIndex: u16,               // Interface/Endpoint index
    pub wLength: u16,              // Length of data
}

/// CDC Device Information
#[derive(Debug)]
pub struct CdcDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub subclass: CdcSubclass,
    pub protocol: CdcProtocol,
    pub interface_type: CdcInterfaceType,
    pub communication_interface_number: u8,
    pub data_interface_number: u8,
    pub notification_endpoint: u8,
    pub data_in_endpoint: u8,
    pub data_out_endpoint: u8,
    pub line_coding: CdcLineCoding,
    pub control_line_state: CdcControlLineState,
    pub max_line_coding: CdcLineCoding,
}

/// CDC Network Statistics
#[derive(Debug)]
pub struct CdcNetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u32,
    pub tx_errors: u32,
}

/// CDC AT Command Parser
#[derive(Debug)]
pub struct CdcAtCommandParser {
    pub buffer: Vec<u8>,
    pub echo_enabled: bool,
    pub verbose_mode: bool,
    pub verbose_level: u8,
    pub response_format: String,
}

/// CDC Response Buffer
#[derive(Debug)]
pub struct CdcResponse {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub is_ok: bool,
    pub error_code: Option<u32>,
}

/// CDC Driver
pub struct CdcDriver {
    pub device_info: CdcDeviceInfo,
    pub at_command_parser: CdcAtCommandParser,
    pub response_buffer: Vec<CdcResponse>,
    pub network_stats: CdcNetworkStats,
    pub line_coding_set: bool,
    pub data_interface_active: bool,
    pub communication_interface_active: bool,
    pub blocking_mode: bool,
    pub current_interface: CdcInterfaceType,
}

/// CDC ACM Driver
pub struct CdcAcmDriver {
    pub base_driver: CdcDriver,
    pub echo_enabled: bool,
    pub at_command_mode: bool,
    pub modem_status: u8,
    pub ring_indicator: bool,
    pub call_progress_indicator: u8,
}

/// CDC NCM (Network Control Model) Driver  
pub struct CdcNcmDriver {
    pub base_driver: CdcDriver,
    pub ncm_version: u16,
    pub max_datagram_size: u16,
    pub max_segment_size: u32,
    pub ndp_sign: u32,
    pub datagram_aggregation: bool,
}

/// CDC Driver Implementation
impl CdcDriver {
    /// Create a new CDC driver instance
    pub fn new(device_address: u8) -> Self {
        Self {
            device_info: CdcDeviceInfo {
                vendor_id: 0,
                product_id: 0,
                subclass: CdcSubclass::Unknown,
                protocol: CdcProtocol::None,
                interface_type: CdcInterfaceType::Unknown,
                communication_interface_number: 0,
                data_interface_number: 0,
                notification_endpoint: 0,
                data_in_endpoint: 0,
                data_out_endpoint: 0,
                line_coding: CdcLineCoding {
                    dwDTERate: 9600,
                    bCharFormat: CDC_STOP_BITS_1,
                    bParityType: CDC_PARITY_NONE,
                    bDataBits: 8,
                },
                control_line_state: CdcControlLineState {
                    dtr_present: false,
                    rts_present: false,
                },
                max_line_coding: CdcLineCoding {
                    dwDTERate: 115200,
                    bCharFormat: CDC_STOP_BITS_2,
                    bParityType: CDC_PARITY_SPACE,
                    bDataBits: 8,
                },
            },
            at_command_parser: CdcAtCommandParser {
                buffer: Vec::new(),
                echo_enabled: false,
                verbose_mode: false,
                verbose_level: 1,
                response_format: "numeric".to_string(),
            },
            response_buffer: Vec::new(),
            network_stats: CdcNetworkStats {
                rx_bytes: 0,
                tx_bytes: 0,
                rx_packets: 0,
                tx_packets: 0,
                rx_errors: 0,
                tx_errors: 0,
            },
            line_coding_set: false,
            data_interface_active: false,
            communication_interface_active: false,
            blocking_mode: false,
            current_interface: CdcInterfaceType::Unknown,
        }
    }

    /// Initialize CDC device
    pub fn initialize(&mut self) -> UsbResult<()> {
        log::info!("Initializing CDC device");
        
        // Set default line coding
        self.set_default_line_coding()?;
        
        // Initialize communication interface
        self.communication_interface_active = true;
        self.current_interface = CdcInterfaceType::Communication;
        
        log::info!("CDC device initialized successfully");
        Ok(())
    }

    /// Set default line coding parameters
    fn set_default_line_coding(&mut self) -> UsbResult<()> {
        self.device_info.line_coding = CdcLineCoding {
            dwDTERate: 9600,
            bCharFormat: CDC_STOP_BITS_1,
            bParityType: CDC_PARITY_NONE,
            bDataBits: 8,
        };
        
        self.line_coding_set = true;
        log::info!("Set default line coding: {} baud, 8N1", 
                  self.device_info.line_coding.dwDTERate);
        Ok(())
    }

    /// Set line coding parameters
    pub fn set_line_coding(&mut self, line_coding: CdcLineCoding) -> UsbResult<()> {
        // Validate line coding parameters
        if line_coding.bDataBits < 5 || line_coding.bDataBits > 8 {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        if line_coding.bCharFormat > CDC_STOP_BITS_2 {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        if line_coding.bParityType > CDC_PARITY_SPACE {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        // Send SET_LINE_CODING request
        let request = CdcControlRequest::SetLineCoding;
        let interface = self.device_info.communication_interface_number;
        
        // Implementation would send control transfer to set line coding
        log::info!("Setting line coding: {} baud, {} data bits, {} parity, {} stop bits",
                  line_coding.dwDTERate, line_coding.bDataBits,
                  line_coding.bParityType, line_coding.bCharFormat + 1);

        self.device_info.line_coding = line_coding;
        self.line_coding_set = true;

        Ok(())
    }

    /// Get line coding parameters
    pub fn get_line_coding(&mut self) -> UsbResult<CdcLineCoding> {
        if !self.communication_interface_active {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        // Send GET_LINE_CODING request
        let request = CdcControlRequest::GetLineCoding;
        let interface = self.device_info.communication_interface_number;
        
        // Implementation would receive line coding from device
        Ok(self.device_info.line_coding)
    }

    /// Set control line state
    pub fn set_control_line_state(&mut self, control_line_state: CdcControlLineState) -> UsbResult<()> {
        if !self.communication_interface_active {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        // Send SET_CONTROL_LINE_STATE request
        let request = CdcControlRequest::SetControlLineState;
        let interface = self.device_info.communication_interface_number;
        
        let value = (control_line_state.dtr_present as u16) |
                   ((control_line_state.rts_present as u16) << 1);

        log::info!("Setting control lines: DTR={}, RTS={}",
                  control_line_state.dtr_present, control_line_state.rts_present);

        self.device_info.control_line_state = control_line_state;

        Ok(())
    }

    /// Send AT command
    pub fn send_at_command(&mut self, command: &str) -> UsbResult<Vec<u8>> {
        if !self.communication_interface_active || !self.line_coding_set {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        let mut cmd_buffer = command.as_bytes().to_vec();
        cmd_buffer.extend_from_slice(b"\r\n");

        // Add to parser buffer
        self.at_command_parser.buffer.extend_from_slice(&cmd_buffer);

        // Send through data endpoint
        self.send_data(&cmd_buffer)?;

        // Wait for response
        self.wait_for_response()?;

        // Extract and parse response
        self.parse_at_response()

        // Note: Actual implementation would handle asynchronous responses
    }

    /// Send data through CDC data interface
    pub fn send_data(&mut self, data: &[u8]) -> UsbResult<()> {
        if !self.data_interface_active {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        // Implementation would send data through data-out endpoint
        self.network_stats.tx_bytes += data.len() as u64;
        self.network_stats.tx_packets += 1;

        log::debug!("Sending {} bytes of CDC data", data.len());
        Ok(())
    }

    /// Receive data through CDC data interface
    pub fn receive_data(&mut self, buffer: &mut [u8]) -> UsbResult<usize> {
        if !self.data_interface_active {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        // Implementation would receive data through data-in endpoint
        // For now, return 0 bytes
        log::debug!("Receiving CDC data into buffer of size {}", buffer.len());
        Ok(0)
    }

    /// Wait for AT command response
    fn wait_for_response(&mut self) -> UsbResult<()> {
        // Implementation would wait for interrupt notification
        // or poll for response available notification
        Ok(())
    }

    /// Parse AT command response
    fn parse_at_response(&mut self) -> UsbResult<Vec<u8>> {
        // Parse response from buffer
        if self.response_buffer.is_empty() {
            return Ok(Vec::new());
        }

        let response = self.response_buffer.remove(0);
        let mut response_data = response.data;

        // Convert to string for parsing
        if let Ok(response_str) = String::from_utf8(response_data.clone()) {
            if response_str.starts_with("OK") {
                log::debug!("AT command completed successfully");
            } else if response_str.starts_with("ERROR") {
                log::warn!("AT command failed: {}", response_str);
            }
        }

        Ok(response_data)
    }

    /// Send break signal
    pub fn send_break(&mut self, break_duration_ms: u16) -> UsbResult<()> {
        if !self.communication_interface_active {
            return Err(UsbDriverError::InvalidConfiguration);
        }

        // Send SEND_BREAK request
        let request = CdcControlRequest::SendBreak;
        let interface = self.device_info.communication_interface_number;
        
        log::info!("Sending break signal for {} ms", break_duration_ms);

        Ok(())
    }

    /// Process CDC notification
    pub fn process_notification(&mut self, notification: &[u8]) -> UsbResult<()> {
        if notification.len() < 8 {
            return Err(UsbDriverError::ProtocolError);
        }

        let bmRequestType = notification[0];
        let bNotification = notification[1];
        let wValue = ((notification[3] as u16) << 8) | (notification[2] as u16);
        let wIndex = ((notification[5] as u16) << 8) | (notification[4] as u16);
        let wLength = ((notification[7] as u16) << 8) | (notification[6] as u16);

        match bNotification {
            0x01 => { // Response Available
                self.process_response_available_notification()?;
            }
            0x2A => { // Network Connection
                log::info!("Network connection notification: {}", wValue);
            }
            0x2B => { // Connection Speed Change
                self.process_connection_speed_change_notification(wValue)?;
            }
            _ => {
                log::warn!("Unknown CDC notification: {:#x}", bNotification);
            }
        }

        Ok(())
    }

    /// Process response available notification
    fn process_response_available_notification(&mut self) -> UsbResult<()> {
        // Response is available - get the response data
        log::debug!("Processing response available notification");
        
        // Implementation would read response data from device
        Ok(())
    }

    /// Process connection speed change notification
    fn process_connection_speed_change_notification(&mut self, new_speed: u16) -> UsbResult<()> {
        log::info!("Connection speed changed to {} baud", new_speed);
        
        // Update line coding if needed
        self.device_info.line_coding.dwDTERate = new_speed as u32;
        
        Ok(())
    }

    /// Set communication interface
    pub fn set_communication_interface(&mut self, interface_number: u8) -> UsbResult<()> {
        // Send SET_COMMUNICATION_INTERFACE request
        let request = CdcControlRequest::SetCommunicationInterface;
        
        self.device_info.communication_interface_number = interface_number;
        self.communication_interface_active = true;
        self.current_interface = CdcInterfaceType::Communication;

        log::info!("Set communication interface to {}", interface_number);
        Ok(())
    }

    /// Set data interface
    pub fn set_data_interface(&mut self, interface_number: u8) -> UsbResult<()> {
        // Send SET_DATA_INTERFACE request if supported
        self.device_info.data_interface_number = interface_number;
        self.data_interface_active = true;
        self.current_interface = CdcInterfaceType::Data;

        log::info!("Set data interface to {}", interface_number);
        Ok(())
    }

    /// Get device information
    pub fn get_device_info(&self) -> &CdcDeviceInfo {
        &self.device_info
    }

    /// Check if line coding is configured
    pub fn is_line_coding_set(&self) -> bool {
        self.line_coding_set
    }

    /// Check if data interface is active
    pub fn is_data_interface_active(&self) -> bool {
        self.data_interface_active
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> &CdcNetworkStats {
        &self.network_stats
    }

    /// Reset network statistics
    pub fn reset_network_stats(&mut self) {
        self.network_stats = CdcNetworkStats {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
            rx_errors: 0,
            tx_errors: 0,
        };
    }

    /// Set blocking mode
    pub fn set_blocking_mode(&mut self, blocking: bool) {
        self.blocking_mode = blocking;
    }

    /// Check if blocking mode is enabled
    pub fn is_blocking_mode(&self) -> bool {
        self.blocking_mode
    }
}

/// CDC ACM Driver Implementation
impl CdcAcmDriver {
    /// Create a new CDC ACM driver
    pub fn new(device_address: u8) -> Self {
        Self {
            base_driver: CdcDriver::new(device_address),
            echo_enabled: false,
            at_command_mode: false,
            modem_status: 0,
            ring_indicator: false,
            call_progress_indicator: 0,
        }
    }

    /// Initialize ACM device
    pub fn initialize(&mut self) -> UsbResult<()> {
        self.base_driver.initialize()?;
        
        // Set ACM-specific defaults
        self.echo_enabled = true;
        self.at_command_mode = true;
        self.base_driver.at_command_parser.echo_enabled = true;
        
        log::info!("CDC ACM device initialized");
        Ok(())
    }

    /// Enable/disable DTR
    pub fn set_dtr(&mut self, enabled: bool) -> UsbResult<()> {
        let mut control_state = self.base_driver.device_info.control_line_state;
        control_state.dtr_present = enabled;
        
        self.base_driver.set_control_line_state(control_state)?;
        Ok(())
    }

    /// Enable/disable RTS
    pub fn set_rts(&mut self, enabled: bool) -> UsbResult<()> {
        let mut control_state = self.base_driver.device_info.control_line_state;
        control_state.rts_present = enabled;
        
        self.base_driver.set_control_line_state(control_state)?;
        Ok(())
    }

    /// Check if DTR is set
    pub fn is_dtr_set(&self) -> bool {
        self.base_driver.device_info.control_line_state.dtr_present
    }

    /// Check if RTS is set
    pub fn is_rts_set(&self) -> bool {
        self.base_driver.device_info.control_line_state.rts_present
    }

    /// Send AT command and get response
    pub fn send_at_command_with_response(&mut self, command: &str) -> UsbResult<String> {
        let response_data = self.base_driver.send_at_command(command)?;
        
        if let Ok(response_str) = String::from_utf8(response_data) {
            Ok(response_str.trim().to_string())
        } else {
            Ok("Invalid UTF-8".to_string())
        }
    }

    /// Test device connection
    pub fn test_connection(&mut self) -> UsbResult<bool> {
        let response = self.send_at_command_with_response("AT")?;
        Ok(response.contains("OK"))
    }

    /// Get modem status
    pub fn get_modem_status(&self) -> u8 {
        self.modem_status
    }

    /// Check for ring indicator
    pub fn has_ring_indicator(&self) -> bool {
        self.ring_indicator
    }
}

/// CDC NCM Driver Implementation
impl CdcNcmDriver {
    /// Create a new CDC NCM driver
    pub fn new(device_address: u8) -> Self {
        Self {
            base_driver: CdcDriver::new(device_address),
            ncm_version: 0x0110, // Default to version 1.1
            max_datagram_size: 8192,
            max_segment_size: 65535,
            ndp_sign: 0x001E16F4,
            datagram_aggregation: true,
        }
    }

    /// Initialize NCM device
    pub fn initialize(&mut self) -> UsbResult<()> {
        self.base_driver.initialize()?;
        
        // Set NCM-specific defaults
        self.datagram_aggregation = true;
        
        log::info!("CDC NCM device initialized");
        Ok(())
    }

    /// Get NCM version
    pub fn get_ncm_version(&self) -> u16 {
        self.ncm_version
    }

    /// Get maximum datagram size
    pub fn get_max_datagram_size(&self) -> u16 {
        self.max_datagram_size
    }

    /// Check if datagram aggregation is enabled
    pub fn is_datagram_aggregation_enabled(&self) -> bool {
        self.datagram_aggregation
    }

    /// Enable/disable datagram aggregation
    pub fn set_datagram_aggregation(&mut self, enabled: bool) {
        self.datagram_aggregation = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdc_driver_creation() {
        let driver = CdcDriver::new(1);
        assert_eq!(driver.is_line_coding_set(), false);
        assert_eq!(driver.is_data_interface_active(), false);
    }

    #[test]
    fn test_cdc_line_coding_creation() {
        let line_coding = CdcLineCoding {
            dwDTERate: 115200,
            bCharFormat: CDC_STOP_BITS_1,
            bParityType: CDC_PARITY_EVEN,
            bDataBits: 7,
        };

        assert_eq!(line_coding.dwDTERate, 115200);
        assert_eq!(line_coding.bDataBits, 7);
    }

    #[test]
    fn test_cdc_control_line_state_creation() {
        let control_state = CdcControlLineState {
            dtr_present: true,
            rts_present: false,
        };

        assert_eq!(control_state.dtr_present, true);
        assert_eq!(control_state.rts_present, false);
    }

    #[test]
    fn test_cdc_acm_driver_creation() {
        let acm_driver = CdcAcmDriver::new(1);
        assert_eq!(acm_driver.echo_enabled, false);
        assert_eq!(acm_driver.at_command_mode, false);
    }

    #[test]
    fn test_cdc_ncm_driver_creation() {
        let ncm_driver = CdcNcmDriver::new(1);
        assert_eq!(ncm_driver.ncm_version, 0x0110);
        assert_eq!(ncm_driver.datagram_aggregation, true);
    }
}