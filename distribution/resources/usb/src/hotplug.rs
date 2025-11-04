//! USB Hotplug Detection and Device Enumeration Module
//! 
//! Provides comprehensive USB device hotplug detection and enumeration including:
//! - Real-time device connection/disconnection detection
//! - Device enumeration and configuration
//! - Port polling and interrupt handling
//! - Device state management
//! - Automatic driver binding

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// USB Hotplug Event Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbHotplugEventType {
    DeviceConnected,
    DeviceDisconnected,
    DeviceReset,
    DeviceEnumerated,
    DeviceConfigured,
    DeviceError,
    PortPowerChanged,
    PortOvercurrent,
    PortSuspended,
    PortResumed,
}

/// USB Device Enumeration State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbEnumerationState {
    Uninitialized,
    Reset,
    Addressed,
    Configured,
    Ready,
    Error,
    Removed,
}

/// USB Device Connection Info
#[derive(Debug, Clone)]
pub struct UsbDeviceConnection {
    pub device_address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub speed: UsbSpeed,
    pub configuration: u8,
    pub interfaces: Vec<UsbInterface>,
    pub descriptor: Option<UsbDeviceDescriptor>,
    pub enumeration_state: UsbEnumerationState,
    pub connection_timestamp: u64,
    pub last_activity: u64,
    pub reset_count: u8,
    pub error_count: u8,
    pub active: bool,
}

/// USB Port Monitor
#[derive(Debug)]
pub struct UsbPortMonitor {
    pub port_number: u8,
    pub hub_address: u8,
    pub previous_status: u32,
    pub current_status: u32,
    pub connection_detected: bool,
    pub enable_detected: bool,
    pub suspend_detected: bool,
    pub reset_detected: bool,
    pub overcurrent_detected: bool,
    pub power_changed_detected: bool,
    pub last_change_time: u64,
    pub polling_interval_ms: u32,
    pub devices_connected: Vec<u8>,
}

/// USB Enumeration Timeout
#[derive(Debug, Clone, Copy)]
pub struct UsbEnumerationTimeout {
    pub device_address: u8,
    pub start_time: u64,
    pub timeout_ms: u32,
    pub retries_remaining: u8,
}

/// USB Hotplug Manager
pub struct UsbHotplugManager {
    pub device_connections: BTreeMap<u8, UsbDeviceConnection>,
    pub port_monitors: BTreeMap<u8, UsbPortMonitor>, // Key: (hub_address << 8) | port_number
    pub enumeration_timeouts: BTreeMap<u8, UsbEnumerationTimeout>,
    pub event_callbacks: Vec<fn(UsbHotplugEventType, UsbDeviceConnection)>,
    pub polling_enabled: bool,
    pub interrupt_enabled: bool,
    pub auto_enumeration: bool,
    pub max_retries: u8,
    pub enumeration_timeout_ms: u32,
    pub reset_timeout_ms: u32,
    pub initialized: bool,
}

/// USB Device Enumeration Result
#[derive(Debug, Clone)]
pub enum UsbEnumerationResult {
    Success(UsbDeviceConnection),
    Timeout,
    ResetRequired,
    Stalled,
    ProtocolError(String),
    UnsupportedDevice,
    DeviceRemoved,
}

/// USB Port Status Change
#[derive(Debug, Clone, Copy)]
pub struct UsbPortStatusChange {
    pub hub_address: u8,
    pub port_number: u8,
    pub previous_status: u32,
    pub current_status: u32,
    pub changes: PortChanges,
}

/// Port Change Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortChanges {
    pub connection_status_changed: bool,
    pub port_enabled_changed: bool,
    pub suspend_status_changed: bool,
    pub overcurrent_status_changed: bool,
    pub reset_status_changed: bool,
    pub power_status_changed: bool,
    pub low_speed_status_changed: bool,
    pub high_speed_status_changed: bool,
}

/// USB Device Connection Manager Implementation
impl UsbDeviceConnection {
    /// Create a new device connection
    pub fn new(device_address: u8) -> Self {
        Self {
            device_address,
            vendor_id: 0,
            product_id: 0,
            speed: UsbSpeed::Full,
            configuration: 0,
            interfaces: Vec::new(),
            descriptor: None,
            enumeration_state: UsbEnumerationState::Uninitialized,
            connection_timestamp: 0, // TODO: Add timestamp
            last_activity: 0,
            reset_count: 0,
            error_count: 0,
            active: false,
        }
    }

    /// Start device enumeration
    pub fn start_enumeration(&mut self) -> UsbResult<()> {
        self.enumeration_state = UsbEnumerationState::Reset;
        self.reset_count = 0;
        self.error_count = 0;
        self.active = true;

        log::info!("Starting enumeration for device {} (address {})", 
                  self.device_address, self.device_address);
        Ok(())
    }

    /// Complete device reset
    pub fn complete_reset(&mut self, speed: UsbSpeed) -> UsbResult<()> {
        self.speed = speed;
        self.enumeration_state = UsbEnumerationState::Addressed;
        self.reset_count += 1;
        self.last_activity = 0; // TODO: Add timestamp

        log::info!("Device {} reset complete, speed: {:?}", 
                  self.device_address, speed);
        Ok(())
    }

    /// Set device address
    pub fn set_address(&mut self, address: u8) -> UsbResult<()> {
        self.device_address = address;
        self.enumeration_state = UsbEnumerationState::Addressed;
        self.last_activity = 0; // TODO: Add timestamp

        log::info!("Device {} assigned address {}", self.device_address, address);
        Ok(())
    }

    /// Configure device
    pub fn configure_device(&mut self, config_value: u8) -> UsbResult<()> {
        self.configuration = config_value;
        self.enumeration_state = UsbEnumerationState::Configured;
        self.last_activity = 0; // TODO: Add timestamp

        log::info!("Device {} configured with configuration {}", 
                  self.device_address, config_value);
        Ok(())
    }

    /// Complete enumeration
    pub fn complete_enumeration(&mut self) -> UsbResult<()> {
        self.enumeration_state = UsbEnumerationState::Ready;
        self.last_activity = 0; // TODO: Add timestamp

        log::info!("Device {} enumeration complete", self.device_address);
        Ok(())
    }

    /// Mark device as ready for use
    pub fn mark_ready(&mut self) {
        self.enumeration_state = UsbEnumerationState::Ready;
        self.active = true;
    }

    /// Handle device error
    pub fn handle_error(&mut self, error: &str) -> UsbResult<()> {
        self.error_count += 1;
        self.enumeration_state = UsbEnumerationState::Error;

        log::warn!("Device {} error: {} (error count: {})", 
                  self.device_address, error, self.error_count);
        
        if self.error_count >= 3 {
            return Err(UsbDriverError::ProtocolError);
        }

        Ok(())
    }

    /// Reset error count
    pub fn reset_error_count(&mut self) {
        self.error_count = 0;
    }

    /// Check if device is ready
    pub fn is_ready(&self) -> bool {
        self.enumeration_state == UsbEnumerationState::Ready && self.active
    }

    /// Check if enumeration should be retried
    pub fn should_retry_enumeration(&self, max_retries: u8) -> bool {
        self.error_count < max_retries && 
        (self.enumeration_state == UsbEnumerationState::Reset || 
         self.enumeration_state == UsbEnumerationState::Error)
    }

    /// Get device information summary
    pub fn get_summary(&self) -> String {
        format!("Device {}: VID:PID {:#06X}:{:#06X}, Speed: {:?}, State: {:?}, Errors: {}", 
               self.device_address,
               self.vendor_id, self.product_id,
               self.speed, self.enumeration_state,
               self.error_count)
    }
}

/// USB Port Monitor Implementation
impl UsbPortMonitor {
    /// Create a new port monitor
    pub fn new(hub_address: u8, port_number: u8) -> Self {
        Self {
            port_number,
            hub_address,
            previous_status: 0,
            current_status: 0,
            connection_detected: false,
            enable_detected: false,
            suspend_detected: false,
            reset_detected: false,
            overcurrent_detected: false,
            power_changed_detected: false,
            last_change_time: 0,
            polling_interval_ms: 100, // Default 100ms
            devices_connected: Vec::new(),
        }
    }

    /// Update port status
    pub fn update_status(&mut self, new_status: u32) -> UsbPortStatusChange {
        let change = self.detect_changes(new_status);
        self.previous_status = self.current_status;
        self.current_status = new_status;

        change
    }

    /// Detect status changes
    fn detect_changes(&mut self, new_status: u32) -> UsbPortStatusChange {
        let changes = PortChanges {
            connection_status_changed: ((self.current_status ^ new_status) & 0x0001) != 0,
            port_enabled_changed: ((self.current_status ^ new_status) & 0x0002) != 0,
            suspend_status_changed: ((self.current_status ^ new_status) & 0x0040) != 0,
            overcurrent_status_changed: ((self.current_status ^ new_status) & 0x0008) != 0,
            reset_status_changed: ((self.current_status ^ new_status) & 0x0080) != 0,
            power_status_changed: ((self.current_status ^ new_status) & 0x1000) != 0,
            low_speed_status_changed: ((self.current_status ^ new_status) & 0x0100) != 0,
            high_speed_status_changed: ((self.current_status ^ new_status) & 0x0800) != 0,
        };

        // Update detection flags
        self.connection_detected = changes.connection_status_changed;
        self.enable_detected = changes.port_enabled_changed;
        self.suspend_detected = changes.suspend_status_changed;
        self.reset_detected = changes.reset_status_changed;
        self.overcurrent_detected = changes.overcurrent_status_changed;
        self.power_changed_detected = changes.power_status_changed;

        self.last_change_time = 0; // TODO: Add timestamp

        UsbPortStatusChange {
            hub_address: self.hub_address,
            port_number: self.port_number,
            previous_status: self.previous_status,
            current_status: new_status,
            changes,
        }
    }

    /// Check if connection detected
    pub fn is_connected(&self) -> bool {
        (self.current_status & 0x0001) != 0
    }

    /// Check if port is enabled
    pub fn is_enabled(&self) -> bool {
        (self.current_status & 0x0002) != 0
    }

    /// Check if device is suspended
    pub fn is_suspended(&self) -> bool {
        (self.current_status & 0x0040) != 0
    }

    /// Check if overcurrent detected
    pub fn has_overcurrent(&self) -> bool {
        (self.current_status & 0x0008) != 0
    }

    /// Check if reset detected
    pub fn is_reset(&self) -> bool {
        (self.current_status & 0x0080) != 0
    }

    /// Get device speed from port status
    pub fn get_speed(&self) -> UsbSpeed {
        if (self.current_status & 0x0800) != 0 {
            UsbSpeed::Super
        } else if (self.current_status & 0x0400) != 0 {
            UsbSpeed::High
        } else if (self.current_status & 0x0100) != 0 {
            UsbSpeed::Low
        } else {
            UsbSpeed::Full
        }
    }

    /// Check if any significant change detected
    pub fn has_significant_change(&self) -> bool {
        self.connection_detected || 
        self.enable_detected || 
        self.reset_detected || 
        self.overcurrent_detected
    }
}

/// USB Hotplug Manager Implementation
impl UsbHotplugManager {
    /// Create a new hotplug manager
    pub fn new() -> Self {
        Self {
            device_connections: BTreeMap::new(),
            port_monitors: BTreeMap::new(),
            enumeration_timeouts: BTreeMap::new(),
            event_callbacks: Vec::new(),
            polling_enabled: true,
            interrupt_enabled: false,
            auto_enumeration: true,
            max_retries: 3,
            enumeration_timeout_ms: 5000, // 5 seconds
            reset_timeout_ms: 10000, // 10 seconds
            initialized: false,
        }
    }

    /// Initialize hotplug manager
    pub fn initialize(&mut self) -> UsbResult<()> {
        self.initialized = true;
        log::info!("USB Hotplug Manager initialized");
        Ok(())
    }

    /// Add event callback
    pub fn add_event_callback(&mut self, callback: fn(UsbHotplugEventType, UsbDeviceConnection)) {
        self.event_callbacks.push(callback);
    }

    /// Register port for monitoring
    pub fn register_port(&mut self, hub_address: u8, port_number: u8) -> UsbResult<()> {
        let key = (hub_address << 8) | port_number;
        
        if self.port_monitors.contains_key(&key) {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        let monitor = UsbPortMonitor::new(hub_address, port_number);
        self.port_monitors.insert(key, monitor);

        log::info!("Port {} on hub {} registered for monitoring", port_number, hub_address);
        Ok(())
    }

    /// Start monitoring port
    pub fn start_port_monitoring(&mut self, hub_address: u8, port_number: u8) -> UsbResult<()> {
        let key = (hub_address << 8) | port_number;
        
        let monitor = self.port_monitors.get_mut(&key)
            .ok_or(UsbDriverError::DeviceNotFound { address: port_number })?;

        // Initialize port status
        let initial_status = 0x0000; // No device connected
        monitor.update_status(initial_status);

        log::info!("Started monitoring port {} on hub {}", port_number, hub_address);
        Ok(())
    }

    /// Update port status and handle changes
    pub fn update_port_status(&mut self, hub_address: u8, port_number: u8, status: u32) -> UsbResult<UsbPortStatusChange> {
        let key = (hub_address << 8) | port_number;
        
        let monitor = self.port_monitors.get_mut(&key)
            .ok_or(UsbDriverError::DeviceNotFound { address: port_number })?;

        let change = monitor.update_status(status);

        // Handle changes
        self.handle_port_changes(&change)?;

        Ok(change)
    }

    /// Handle port status changes
    fn handle_port_changes(&mut self, change: &UsbPortStatusChange) -> UsbResult<()> {
        let key = (change.hub_address << 8) | change.port_number;
        let monitor = self.port_monitors.get(&key)
            .ok_or(UsbDriverError::DeviceNotFound { address: change.port_number })?;

        // Handle connection
        if change.changes.connection_status_changed {
            if change.current_status & 0x0001 != 0 {
                // Device connected
                self.handle_device_connected(change.hub_address, change.port_number, monitor.get_speed())?;
            } else {
                // Device disconnected
                self.handle_device_disconnected(change.hub_address, change.port_number)?;
            }
        }

        // Handle reset
        if change.changes.reset_status_changed && change.current_status & 0x0080 != 0 {
            self.handle_port_reset(change.hub_address, change.port_number)?;
        }

        // Handle enable changes
        if change.changes.port_enabled_changed {
            if change.current_status & 0x0002 != 0 {
                self.handle_port_enabled(change.hub_address, change.port_number)?;
            } else {
                self.handle_port_disabled(change.hub_address, change.port_number)?;
            }
        }

        // Handle suspend/resume
        if change.changes.suspend_status_changed {
            if change.current_status & 0x0040 != 0 {
                self.handle_port_suspended(change.hub_address, change.port_number)?;
            } else {
                self.handle_port_resumed(change.hub_address, change.port_number)?;
            }
        }

        Ok(())
    }

    /// Handle device connection
    fn handle_device_connected(&mut self, hub_address: u8, port_number: u8, speed: UsbSpeed) -> UsbResult<()> {
        // Create new device connection
        let mut device = UsbDeviceConnection::new(0); // Address will be assigned during enumeration
        
        // Start enumeration
        device.start_enumeration()?;
        
        let device_address = self.device_connections.len() as u8 + 1; // Simple address assignment
        device.device_address = device_address;
        device.speed = speed;
        device.connection_timestamp = 0; // TODO: Add timestamp

        // Store device connection
        self.device_connections.insert(device_address, device);

        // Update port monitor
        let key = (hub_address << 8) | port_number;
        if let Some(monitor) = self.port_monitors.get_mut(&key) {
            monitor.devices_connected.push(device_address);
        }

        // Start enumeration timeout
        let timeout = UsbEnumerationTimeout {
            device_address,
            start_time: 0, // TODO: Add timestamp
            timeout_ms: self.enumeration_timeout_ms,
            retries_remaining: self.max_retries,
        };
        self.enumeration_timeouts.insert(device_address, timeout);

        log::info!("Device connected on hub {} port {} (address {})", 
                  hub_address, port_number, device_address);

        // Trigger callback
        if let Some(device) = self.device_connections.get(&device_address) {
            self.trigger_event_callback(UsbHotplugEventType::DeviceConnected, device.clone());
        }

        Ok(())
    }

    /// Handle device disconnection
    fn handle_device_disconnected(&mut self, hub_address: u8, port_number: u8) -> UsbResult<()> {
        // Find device on this port
        let mut disconnected_device = None;
        
        for (device_address, device) in &self.device_connections {
            // Check if device was on this port (this is simplified)
            if device.active {
                disconnected_device = Some(*device_address);
                break;
            }
        }

        if let Some(device_address) = disconnected_device {
            if let Some(mut device) = self.device_connections.remove(&device_address) {
                device.enumeration_state = UsbEnumerationState::Removed;
                device.active = false;

                // Remove from enumeration timeouts
                self.enumeration_timeouts.remove(&device_address);

                log::info!("Device {} disconnected", device_address);

                // Trigger callback
                self.trigger_event_callback(UsbHotplugEventType::DeviceDisconnected, device);
            }
        }

        // Update port monitor
        let key = (hub_address << 8) | port_number;
        if let Some(monitor) = self.port_monitors.get_mut(&key) {
            monitor.devices_connected.clear();
        }

        Ok(())
    }

    /// Handle port reset
    fn handle_port_reset(&mut self, hub_address: u8, port_number: u8) -> UsbResult<()> {
        log::info!("Port reset detected on hub {} port {}", hub_address, port_number);

        // Find device on this port and reset it
        let mut reset_device_addresses = Vec::new();
        
        for (device_address, device) in &self.device_connections {
            if device.active && device.enumeration_state == UsbEnumerationState::Ready {
                reset_device_addresses.push(*device_address);
            }
        }

        for device_address in reset_device_addresses {
            if let Some(device) = self.device_connections.get_mut(&device_address) {
                device.enumeration_state = UsbEnumerationState::Reset;
                self.trigger_event_callback(UsbHotplugEventType::DeviceReset, device.clone());
            }
        }

        Ok(())
    }

    /// Handle port enabled
    fn handle_port_enabled(&mut self, hub_address: u8, port_number: u8) -> UsbResult<()> {
        log::info!("Port enabled on hub {} port {}", hub_address, port_number);

        // Update device enumeration state
        let key = (hub_address << 8) | port_number;
        if let Some(monitor) = self.port_monitors.get(&key) {
            for &device_address in &monitor.devices_connected {
                if let Some(device) = self.device_connections.get_mut(&device_address) {
                    if device.enumeration_state == UsbEnumerationState::Reset {
                        device.complete_reset(monitor.get_speed())?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle port disabled
    fn handle_port_disabled(&mut self, _hub_address: u8, _port_number: u8) -> UsbResult<()> {
        log::info!("Port disabled on hub {} port {}", _hub_address, _port_number);
        // Device should handle this through disconnection detection
        Ok(())
    }

    /// Handle port suspended
    fn handle_port_suspended(&mut self, _hub_address: u8, _port_number: u8) -> UsbResult<()> {
        log::info!("Port suspended on hub {} port {}", _hub_address, _port_number);
        // Implementation would handle suspend state for devices
        Ok(())
    }

    /// Handle port resumed
    fn handle_port_resumed(&mut self, _hub_address: u8, _port_number: u8) -> UsbResult<()> {
        log::info!("Port resumed on hub {} port {}", _hub_address, _port_number);
        // Implementation would handle resume for suspended devices
        Ok(())
    }

    /// Enumerate a device
    pub fn enumerate_device(&mut self, device_address: u8) -> UsbResult<UsbEnumerationResult> {
        let device = self.device_connections.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        match device.enumeration_state {
            UsbEnumerationState::Reset => {
                // Complete reset and start address assignment
                // This would involve actual USB transactions
                device.enumeration_state = UsbEnumerationState::Addressed;
                Ok(UsbEnumerationResult::ResetRequired)
            }
            UsbEnumerationState::Addressed => {
                // Assign address (simplified)
                device.set_address(device_address)?;
                Ok(UsbEnumerationResult::Success(device.clone()))
            }
            UsbEnumerationState::Configured => {
                // Device is configured, ready for use
                device.mark_ready();
                Ok(UsbEnumerationResult::Success(device.clone()))
            }
            _ => {
                Err(UsbDriverError::InvalidConfiguration)
            }
        }
    }

    /// Update enumeration timeouts
    pub fn update_timeouts(&mut self) -> UsbResult<()> {
        let current_time = 0; // TODO: Add timestamp

        let mut expired_devices = Vec::new();

        for (&device_address, timeout) in &self.enumeration_timeouts {
            let elapsed = current_time - timeout.start_time;
            if elapsed > timeout.timeout_ms {
                expired_devices.push(device_address);
            }
        }

        for device_address in expired_devices {
            self.handle_enumeration_timeout(device_address)?;
        }

        Ok(())
    }

    /// Handle enumeration timeout
    fn handle_enumeration_timeout(&mut self, device_address: u8) -> UsbResult<()> {
        let device = self.device_connections.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        if device.should_retry_enumeration(self.max_retries) {
            log::warn!("Enumeration timeout for device {}, retrying", device_address);
            device.start_enumeration()?;
        } else {
            log::error!("Enumeration failed for device {} after {} retries", 
                       device_address, self.max_retries);
            device.enumeration_state = UsbEnumerationState::Error;
            return Ok(());
        }

        Ok(())
    }

    /// Trigger event callbacks
    fn trigger_event_callback(&self, event_type: UsbHotplugEventType, device: UsbDeviceConnection) {
        for callback in &self.event_callbacks {
            callback(event_type, device.clone());
        }
    }

    /// Get all connected devices
    pub fn get_connected_devices(&self) -> Vec<&UsbDeviceConnection> {
        self.device_connections.values()
            .filter(|device| device.active)
            .collect()
    }

    /// Get device by address
    pub fn get_device(&self, device_address: u8) -> UsbResult<&UsbDeviceConnection> {
        self.device_connections.get(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })
    }

    /// Get port monitor
    pub fn get_port_monitor(&self, hub_address: u8, port_number: u8) -> UsbResult<&UsbPortMonitor> {
        let key = (hub_address << 8) | port_number;
        self.port_monitors.get(&key)
            .ok_or(UsbDriverError::DeviceNotFound { address: port_number })
    }

    /// Get all port monitors
    pub fn get_port_monitors(&self) -> Vec<&UsbPortMonitor> {
        self.port_monitors.values().collect()
    }

    /// Check if manager is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> UsbHotplugStats {
        let mut connected_devices = 0;
        let mut enumerated_devices = 0;
        let mut error_devices = 0;

        for device in self.device_connections.values() {
            if device.active {
                connected_devices += 1;
            }
            if device.enumeration_state == UsbEnumerationState::Ready {
                enumerated_devices += 1;
            }
            if device.enumeration_state == UsbEnumerationState::Error {
                error_devices += 1;
            }
        }

        UsbHotplugStats {
            total_devices: self.device_connections.len() as u8,
            connected_devices,
            enumerated_devices,
            error_devices,
            active_ports: self.port_monitors.len() as u8,
            active_timeouts: self.enumeration_timeouts.len() as u8,
        }
    }

    /// Enable/disable polling
    pub fn set_polling_enabled(&mut self, enabled: bool) {
        self.polling_enabled = enabled;
        log::info!("USB polling {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Enable/disable auto enumeration
    pub fn set_auto_enumeration(&mut self, enabled: bool) {
        self.auto_enumeration = enabled;
        log::info!("Auto enumeration {}", if enabled { "enabled" } else { "disabled" });
    }
}

/// USB Hotplug Statistics
#[derive(Debug, Clone)]
pub struct UsbHotplugStats {
    pub total_devices: u8,
    pub connected_devices: u8,
    pub enumerated_devices: u8,
    pub error_devices: u8,
    pub active_ports: u8,
    pub active_timeouts: u8,
}

impl Default for UsbHotplugManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_connection_creation() {
        let device = UsbDeviceConnection::new(5);
        assert_eq!(device.device_address, 5);
        assert_eq!(device.enumeration_state, UsbEnumerationState::Uninitialized);
        assert!(!device.is_ready());
    }

    #[test]
    fn test_port_monitor_creation() {
        let monitor = UsbPortMonitor::new(1, 2);
        assert_eq!(monitor.hub_address, 1);
        assert_eq!(monitor.port_number, 2);
        assert!(!monitor.is_connected());
    }

    #[test]
    fn test_port_status_update() {
        let mut monitor = UsbPortMonitor::new(1, 2);
        
        let change1 = monitor.update_status(0x0001); // Device connected
        assert!(change1.changes.connection_status_changed);
        assert!(monitor.is_connected());

        let change2 = monitor.update_status(0x0003); // Device still connected + enabled
        assert!(change2.changes.port_enabled_changed);
        assert!(monitor.is_enabled());
    }

    #[test]
    fn test_hotplug_manager_creation() {
        let manager = UsbHotplugManager::new();
        assert!(!manager.is_initialized());
        assert_eq!(manager.get_connected_devices().len(), 0);
    }

    #[test]
    fn test_port_registration() {
        let mut manager = UsbHotplugManager::new();
        
        let result = manager.register_port(1, 2);
        assert!(result.is_ok());
        assert_eq!(manager.get_port_monitors().len(), 1);
    }

    #[test]
    fn test_enumeration_timeout() {
        let mut manager = UsbHotplugManager::new();
        let mut device = UsbDeviceConnection::new(1);
        
        device.enumeration_state = UsbEnumerationState::Reset;
        assert!(device.should_retry_enumeration(3));
        
        device.error_count = 3;
        assert!(!device.should_retry_enumeration(3));
    }
}