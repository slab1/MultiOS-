//! USB Hub Management Module
//! 
//! Provides comprehensive USB hub functionality including:
//! - Hub descriptor parsing
//! - Port status management
//! - Power management
//! - Hotplug detection
//! - Device enumeration and routing

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// USB Hub States
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbHubState {
    Uninitialized,
    Initialized,
    Configured,
    Error,
    Suspended,
}

/// USB Hub Features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbHubFeature {
    PortPower = 0x0008,
    PortReset = 0x0004,
    PortEnable = 0x0004,
    PortSuspend = 0x0005,
    PortOverCurrent = 0x000A,
    PortConnection = 0x0008,
    PortPowerGood = 0x0002,
}

/// USB Hub Status
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbHubStatus {
    pub wHubStatus: u16,
    pub wHubChange: u16,
    pub hub_local_power_good: bool,
    pub hub_overcurrent: bool,
    pub hub_local_power_changed: bool,
    pub hub_overcurrent_changed: bool,
    pub power_switching_mode: u16,
    pub compound_device: bool,
    pub overcurrent_protection: u16,
}

/// USB Port Status and Control
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbPortStatus {
    pub wPortStatus: u16,
    pub wPortChange: u16,
    pub connection: bool,
    pub enable: bool,
    pub suspend: bool,
    pub over_current: bool,
    pub reset: bool,
    pub low_speed: bool,
    pub high_speed: bool,
    pub test_mode: bool,
    pub indicator_control: u8,
    pub power_on_to_good_time_ms: u8,
}

/// USB Hub Characteristics
#[derive(Debug, Clone, Copy)]
pub struct UsbHubCharacteristics {
    pub power_switching: u16,
    pub compound_device: bool,
    pub overcurrent_protection: u16,
    pub tt_think_time: u8,
    pub port_indicators_support: bool,
    pub power_good_time: u8,
}

/// USB Hub Power Management
#[derive(Debug, Clone, Copy)]
pub struct UsbHubPower {
    pub per_port_power: bool,
    pub ganged_power: bool,
    pub power_on_to_good_time_ms: u8,
    pub max_power_ma: u16,
    pub overcurrent_current_ma: u8,
}

/// USB Hub Device
#[derive(Debug)]
pub struct UsbHubDevice {
    pub address: u8,
    pub hub_descriptors: UsbHubDescriptor,
    pub characteristics: UsbHubCharacteristics,
    pub power_management: UsbHubPower,
    pub port_status: Vec<UsbPortStatus>,
    pub hub_status: UsbHubStatus,
    pub device_connected: bool,
    pub self_powered: bool,
    pub hub_state: UsbHubState,
    pub port_reset_busy: bool,
    pub port_power_enabled: bool,
    pub power_management_enabled: bool,
}

/// USB Port Device Entry
#[derive(Debug, Clone)]
pub struct UsbPortDevice {
    pub port_number: u8,
    pub hub_address: u8,
    pub device_address: Option<u8>,
    pub speed: UsbSpeed,
    pub connection_detected: bool,
    pub reset_complete: bool,
    pub device_enumerated: bool,
    pub power_state: UsbPowerState,
    pub last_activity: u64,
}

/// USB Hub Manager
pub struct UsbHubManager {
    pub hubs: BTreeMap<u8, UsbHubDevice>,
    pub port_devices: BTreeMap<u8, UsbPortDevice>,
    pub hub_descriptors_cache: BTreeMap<u8, UsbHubDescriptor>,
    pub discovery_callback: Option<fn(UsbEvent)>,
    pub power_management_enabled: bool,
    pub port_reset_timeout_ms: u32,
    pub enumeration_timeout_ms: u32,
}

/// USB Hub Configuration
#[derive(Debug, Clone)]
pub struct UsbHubConfig {
    pub enable_power_management: bool,
    pub port_reset_timeout_ms: u32,
    pub enumeration_timeout_ms: u32,
    pub auto_power_on: bool,
    pub power_cycling_enabled: bool,
    pub overcurrent_handling: bool,
}

/// USB Hub Implementation
impl UsbHubDevice {
    /// Create a new USB hub device
    pub fn new(address: u8, descriptors: UsbHubDescriptor) -> Self {
        Self {
            address,
            hub_descriptors: descriptors,
            characteristics: UsbHubCharacteristics {
                power_switching: 0,
                compound_device: false,
                overcurrent_protection: 0,
                tt_think_time: 0,
                port_indicators_support: false,
                power_good_time: descriptors.bPwrOn2PwrGood,
            },
            power_management: UsbHubPower {
                per_port_power: false,
                ganged_power: false,
                power_on_to_good_time_ms: descriptors.bPwrOn2PwrGood as u16 * 2,
                max_power_ma: descriptors.bHubContrCurrent as u16 * 100,
                overcurrent_current_ma: 100,
            },
            port_status: vec![UsbPortStatus {
                wPortStatus: 0,
                wPortChange: 0,
                connection: false,
                enable: false,
                suspend: false,
                over_current: false,
                reset: false,
                low_speed: false,
                high_speed: false,
                test_mode: false,
                indicator_control: 0,
                power_on_to_good_time_ms: descriptors.bPwrOn2PwrGood,
            }; descriptors.bNbrPorts as usize],
            hub_status: UsbHubStatus {
                wHubStatus: 0,
                wHubChange: 0,
                hub_local_power_good: false,
                hub_overcurrent: false,
                hub_local_power_changed: false,
                hub_overcurrent_changed: false,
                power_switching_mode: 0,
                compound_device: false,
                overcurrent_protection: 0,
            },
            device_connected: true,
            self_powered: false,
            hub_state: UsbHubState::Uninitialized,
            port_reset_busy: false,
            port_power_enabled: false,
            power_management_enabled: false,
        }
    }

    /// Initialize hub device
    pub fn initialize(&mut self) -> UsbResult<()> {
        if self.hub_state == UsbHubState::Initialized {
            return Ok(());
        }

        // Parse hub characteristics from descriptors
        self.parse_characteristics();

        // Initialize port status array
        for port_status in &mut self.port_status {
            port_status.power_on_to_good_time_ms = self.hub_descriptors.bPwrOn2PwrGood;
        }

        self.hub_state = UsbHubState::Initialized;
        log::info!("Hub {} initialized with {} ports", self.address, self.hub_descriptors.bNbrPorts);
        Ok(())
    }

    /// Configure hub device
    pub fn configure(&mut self) -> UsbResult<()> {
        if self.hub_state != UsbHubState::Initialized {
            return Err(UsbDriverError::ControllerNotInitialized);
        }

        // Enable port power if supported
        if self.characteristics.power_switching == 0x0001 {
            self.enable_port_power()?;
            self.port_power_enabled = true;
        }

        // Enable power management if supported
        if self.characteristics.power_switching != 0 {
            self.power_management_enabled = true;
        }

        self.hub_state = UsbHubState::Configured;
        log::info!("Hub {} configured", self.address);
        Ok(())
    }

    /// Parse hub characteristics from descriptors
    fn parse_characteristics(&mut self) {
        let characteristics = self.hub_descriptors.wHubCharacteristics;

        // Extract power switching mode
        self.characteristics.power_switching = characteristics & 0x0003;
        match self.characteristics.power_switching {
            0x0000 => {
                self.power_management.per_port_power = true;
                self.power_management.ganged_power = false;
            }
            0x0001 => {
                self.power_management.per_port_power = false;
                self.power_management.ganged_power = true;
            }
            0x0002 => {
                self.power_management.per_port_power = false;
                self.power_management.ganged_power = false;
            }
            0x0003 => {
                self.power_management.per_port_power = true;
                self.power_management.ganged_power = true;
            }
            _ => {}
        }

        // Extract compound device flag
        self.characteristics.compound_device = (characteristics & 0x0004) != 0;

        // Extract overcurrent protection
        self.characteristics.overcurrent_protection = (characteristics & 0x0018) >> 3;
        match self.characteristics.overcurrent_protection {
            0x00 => {
                self.power_management.overcurrent_current_ma = 0;
            }
            0x01 => {
                self.power_management.overcurrent_current_ma = 100;
            }
            0x02 => {
                self.power_management.overcurrent_current_ma = 200;
            }
            0x03 => {
                self.power_management.overcurrent_current_ma = 500;
            }
            _ => {}
        }

        // Extract TT think time
        self.characteristics.tt_think_time = (characteristics & 0x0060) >> 5;

        // Extract port indicators support
        self.characteristics.port_indicators_support = (characteristics & 0x0080) != 0;
    }

    /// Get port status
    pub fn get_port_status(&self, port_number: u8) -> UsbResult<UsbPortStatus> {
        if port_number == 0 || port_number > self.hub_descriptors.bNbrPorts {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        Ok(self.port_status[port_number as usize - 1])
    }

    /// Set port feature
    pub fn set_port_feature(&mut self, port_number: u8, feature: UsbHubFeature) -> UsbResult<()> {
        if port_number == 0 || port_number > self.hub_descriptors.bNbrPorts {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        let port_status = &mut self.port_status[port_number as usize - 1];

        match feature {
            UsbHubFeature::PortPower => {
                port_status.enable = true;
                log::info!("Hub {} port {} power enabled", self.address, port_number);
            }
            UsbHubFeature::PortReset => {
                port_status.reset = true;
                port_status.connection = true;
                log::info!("Hub {} port {} reset started", self.address, port_number);
            }
            UsbHubFeature::PortEnable => {
                port_status.enable = true;
                log::info!("Hub {} port {} enabled", self.address, port_number);
            }
            UsbHubFeature::PortSuspend => {
                port_status.suspend = true;
                log::info!("Hub {} port {} suspended", self.address, port_number);
            }
            _ => {
                log::warn!("Unsupported hub feature: {:?}", feature);
                return Err(UsbDriverError::UnsupportedFeature);
            }
        }

        Ok(())
    }

    /// Clear port feature
    pub fn clear_port_feature(&mut self, port_number: u8, feature: UsbHubFeature) -> UsbResult<()> {
        if port_number == 0 || port_number > self.hub_descriptors.bNbrPorts {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        let port_status = &mut self.port_status[port_number as usize - 1];

        match feature {
            UsbHubFeature::PortEnable => {
                port_status.enable = false;
                log::info!("Hub {} port {} disabled", self.address, port_number);
            }
            UsbHubFeature::PortReset => {
                port_status.reset = false;
                log::info!("Hub {} port {} reset cleared", self.address, port_number);
            }
            UsbHubFeature::PortSuspend => {
                port_status.suspend = false;
                log::info!("Hub {} port {} resumed", self.address, port_number);
            }
            _ => {
                log::warn!("Unsupported hub clear feature: {:?}", feature);
                return Err(UsbDriverError::UnsupportedFeature);
            }
        }

        Ok(())
    }

    /// Enable port power
    pub fn enable_port_power(&mut self) -> UsbResult<()> {
        if self.characteristics.power_switching == 0x0002 {
            // No power switching available
            return Ok(());
        }

        if self.characteristics.power_switching == 0x0000 {
            // Per-port power switching
            for i in 1..=self.hub_descriptors.bNbrPorts {
                self.set_port_feature(i, UsbHubFeature::PortPower)?;
            }
        } else {
            // Ganged power switching - power on all ports
            // Implementation would send power-on command to hub
        }

        log::info!("Hub {} port power enabled", self.address);
        Ok(())
    }

    /// Disable port power
    pub fn disable_port_power(&mut self) -> UsbResult<()> {
        if self.characteristics.power_switching == 0x0002 {
            // No power switching available
            return Ok(());
        }

        if self.characteristics.power_switching == 0x0000 {
            // Per-port power switching
            for i in 1..=self.hub_descriptors.bNbrPorts {
                // Implementation would send power-off command
            }
        } else {
            // Ganged power switching - power off all ports
            // Implementation would send power-off command to hub
        }

        log::info!("Hub {} port power disabled", self.address);
        Ok(())
    }

    /// Detect port connections
    pub fn detect_connections(&mut self) -> UsbResult<Vec<u8>> {
        let mut connected_ports = Vec::new();

        for port_number in 1..=self.hub_descriptors.bNbrPorts {
            // Implementation would read actual port status from hub
            // For now, simulate detection based on current state

            let port_status = &mut self.port_status[port_number as usize - 1];
            
            // Simulate connection detection
            if !port_status.connection {
                port_status.connection = true;
                connected_ports.push(port_number);
                log::info!("Hub {} port {} device connected", self.address, port_number);
            }
        }

        Ok(connected_ports)
    }

    /// Detect port disconnections
    pub fn detect_disconnections(&mut self) -> UsbResult<Vec<u8>> {
        let mut disconnected_ports = Vec::new();

        for port_number in 1..=self.hub_descriptors.bNbrPorts {
            let port_status = &mut self.port_status[port_number as usize - 1];
            
            // Simulate disconnection detection
            if port_status.connection {
                port_status.connection = false;
                port_status.enable = false;
                disconnected_ports.push(port_number);
                log::info!("Hub {} port {} device disconnected", self.address, port_number);
            }
        }

        Ok(disconnected_ports)
    }

    /// Reset port
    pub fn reset_port(&mut self, port_number: u8) -> UsbResult<()> {
        if port_number == 0 || port_number > self.hub_descriptors.bNbrPorts {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        if self.port_reset_busy {
            return Err(UsbDriverError::Timeout);
        }

        self.port_reset_busy = true;

        // Clear reset and enable flags first
        self.clear_port_feature(port_number, UsbHubFeature::PortReset)?;
        self.clear_port_feature(port_number, UsbHubFeature::PortEnable)?;

        // Set reset
        self.set_port_feature(port_number, UsbHubFeature::PortReset)?;

        log::info!("Hub {} port {} reset initiated", self.address, port_number);
        Ok(())
    }

    /// Complete port reset
    pub fn complete_port_reset(&mut self, port_number: u8) -> UsbResult<UsbSpeed> {
        if port_number == 0 || port_number > self.hub_descriptors.bNbrPorts {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        self.port_reset_busy = false;

        let port_status = &mut self.port_status[port_number as usize - 1];

        // Clear reset flag
        self.clear_port_feature(port_number, UsbHubFeature::PortReset)?;

        // Enable port
        self.set_port_feature(port_number, UsbHubFeature::PortEnable)?;

        // Determine device speed based on port status
        let speed = if port_status.high_speed {
            UsbSpeed::High
        } else if port_status.low_speed {
            UsbSpeed::Low
        } else {
            UsbSpeed::Full
        };

        port_status.reset = false;
        port_status.connection = true;

        log::info!("Hub {} port {} reset complete, speed: {:?}", 
                  self.address, port_number, speed);
        Ok(speed)
    }

    /// Get hub statistics
    pub fn get_stats(&self) -> UsbHubStats {
        let mut connected_ports = 0;
        let mut enabled_ports = 0;

        for port_status in &self.port_status {
            if port_status.connection {
                connected_ports += 1;
            }
            if port_status.enable {
                enabled_ports += 1;
            }
        }

        UsbHubStats {
            total_ports: self.hub_descriptors.bNbrPorts,
            connected_ports,
            enabled_ports,
            powered_ports: if self.port_power_enabled { 
                self.hub_descriptors.bNbrPorts 
            } else { 
                0 
            },
            self_powered: self.self_powered,
            power_management_enabled: self.power_management_enabled,
        }
    }

    /// Suspend hub
    pub fn suspend(&mut self) -> UsbResult<()> {
        self.hub_state = UsbHubState::Suspended;
        log::info!("Hub {} suspended", self.address);
        Ok(())
    }

    /// Resume hub
    pub fn resume(&mut self) -> UsbResult<()> {
        self.hub_state = UsbHubState::Configured;
        log::info!("Hub {} resumed", self.address);
        Ok(())
    }
}

/// USB Hub Statistics
#[derive(Debug, Clone)]
pub struct UsbHubStats {
    pub total_ports: u8,
    pub connected_ports: u8,
    pub enabled_ports: u8,
    pub powered_ports: u8,
    pub self_powered: bool,
    pub power_management_enabled: bool,
}

/// USB Hub Manager Implementation
impl UsbHubManager {
    /// Create a new hub manager
    pub fn new() -> Self {
        Self {
            hubs: BTreeMap::new(),
            port_devices: BTreeMap::new(),
            hub_descriptors_cache: BTreeMap::new(),
            discovery_callback: None,
            power_management_enabled: true,
            port_reset_timeout_ms: 10000, // 10 seconds
            enumeration_timeout_ms: 5000, // 5 seconds
        }
    }

    /// Initialize hub manager
    pub fn initialize(&mut self) -> UsbResult<()> {
        log::info!("USB Hub Manager initialized");
        Ok(())
    }

    /// Register a hub device
    pub fn register_hub(&mut self, address: u8, descriptors: UsbHubDescriptor) -> UsbResult<()> {
        if self.hubs.contains_key(&address) {
            return Err(UsbDriverError::DeviceNotFound { address });
        }

        let hub = UsbHubDevice::new(address, descriptors);
        self.hubs.insert(address, hub);

        log::info!("Hub {} registered with {} ports", address, descriptors.bNbrPorts);
        Ok(())
    }

    /// Initialize all hubs
    pub fn initialize_hubs(&mut self) -> UsbResult<()> {
        for (address, hub) in &mut self.hubs {
            hub.initialize()?;
            hub.configure()?;
            log::info!("Hub {} initialized and configured", address);
        }

        Ok(())
    }

    /// Discover all connected devices
    pub fn discover_devices(&mut self) -> UsbResult<Vec<UsbEvent>> {
        let mut events = Vec::new();

        for (hub_address, hub) in &mut self.hubs {
            // Detect new connections
            match hub.detect_connections() {
                Ok(new_connections) => {
                    for port_number in new_connections {
                        let event = UsbEvent::DeviceConnected { 
                            port: port_number, 
                            speed: UsbSpeed::Full // Default, will be determined during reset
                        };
                        events.push(event);

                        // Create port device entry
                        let port_device = UsbPortDevice {
                            port_number,
                            hub_address: *hub_address,
                            device_address: None,
                            speed: UsbSpeed::Full,
                            connection_detected: true,
                            reset_complete: false,
                            device_enumerated: false,
                            power_state: UsbPowerState::Active,
                            last_activity: 0,
                        };

                        let key = (*hub_address << 8) | port_number;
                        self.port_devices.insert(key, port_device);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to detect connections on hub {}: {:?}", hub_address, e);
                }
            }

            // Detect disconnections
            match hub.detect_disconnections() {
                Ok(disconnections) => {
                    for port_number in disconnections {
                        let key = (*hub_address << 8) | port_number;
                        if let Some(port_device) = self.port_devices.remove(&key) {
                            if let Some(device_address) = port_device.device_address {
                                let event = UsbEvent::DeviceDisconnected { address: device_address };
                                events.push(event);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to detect disconnections on hub {}: {:?}", hub_address, e);
                }
            }
        }

        Ok(events)
    }

    /// Reset a port
    pub fn reset_port(&mut self, hub_address: u8, port_number: u8) -> UsbResult<UsbSpeed> {
        let hub = self.hubs.get_mut(&hub_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: hub_address })?;

        hub.reset_port(port_number)?;

        // Simulate reset completion after timeout
        // In real implementation, this would be handled by interrupt/notification
        let speed = hub.complete_port_reset(port_number)?;

        // Update port device
        let key = (hub_address << 8) | port_number;
        if let Some(port_device) = self.port_devices.get_mut(&key) {
            port_device.reset_complete = true;
            port_device.speed = speed;
            port_device.last_activity = 0; // TODO: Add timestamp
        }

        Ok(speed)
    }

    /// Enumerate device on port
    pub fn enumerate_device(&mut self, hub_address: u8, port_number: u8) -> UsbResult<u8> {
        let hub = self.hubs.get(&hub_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: hub_address })?;

        if port_number == 0 || port_number > hub.hub_descriptors.bNbrPorts {
            return Err(UsbDriverError::DeviceNotFound { address: port_number });
        }

        // Assign device address (simple allocation, in reality would be more sophisticated)
        let mut device_address = 2; // Start from address 2 (0 is reserved, 1 is host)
        while self.port_devices.values().any(|pd| pd.device_address == Some(device_address)) {
            device_address += 1;
        }

        // Update port device
        let key = (hub_address << 8) | port_number;
        if let Some(port_device) = self.port_devices.get_mut(&key) {
            port_device.device_address = Some(device_address);
            port_device.device_enumerated = true;
        }

        log::info!("Device enumerated on hub {} port {} with address {}", 
                  hub_address, port_number, device_address);
        Ok(device_address)
    }

    /// Get hub by address
    pub fn get_hub(&self, address: u8) -> UsbResult<&UsbHubDevice> {
        self.hubs.get(&address)
            .ok_or(UsbDriverError::DeviceNotFound { address })
    }

    /// Get port device
    pub fn get_port_device(&self, hub_address: u8, port_number: u8) -> UsbResult<&UsbPortDevice> {
        let key = (hub_address << 8) | port_number;
        self.port_devices.get(&key)
            .ok_or(UsbDriverError::DeviceNotFound { address: port_number })
    }

    /// Get all hubs
    pub fn get_hubs(&self) -> Vec<u8> {
        self.hubs.keys().cloned().collect()
    }

    /// Get all port devices
    pub fn get_port_devices(&self) -> Vec<&UsbPortDevice> {
        self.port_devices.values().collect()
    }

    /// Get system hub statistics
    pub fn get_system_stats(&self) -> UsbHubSystemStats {
        let mut total_hubs = 0;
        let mut total_ports = 0;
        let mut connected_ports = 0;
        let mut powered_hubs = 0;
        let mut powered_ports = 0;

        for hub in self.hubs.values() {
            total_hubs += 1;
            total_ports += hub.hub_descriptors.bNbrPorts;

            let stats = hub.get_stats();
            connected_ports += stats.connected_ports;
            powered_ports += stats.powered_ports;

            if stats.self_powered {
                powered_hubs += 1;
            }
        }

        UsbHubSystemStats {
            total_hubs,
            total_ports,
            connected_ports,
            powered_hubs,
            powered_ports,
            power_management_enabled: self.power_management_enabled,
        }
    }

    /// Set discovery callback
    pub fn set_discovery_callback(&mut self, callback: fn(UsbEvent)) {
        self.discovery_callback = Some(callback);
    }

    /// Check if hub manager is initialized
    pub fn is_initialized(&self) -> bool {
        !self.hubs.is_empty()
    }

    /// Remove hub
    pub fn remove_hub(&mut self, address: u8) -> UsbResult<()> {
        let hub = self.hubs.remove(&address)
            .ok_or(UsbDriverError::DeviceNotFound { address })?;

        // Remove associated port devices
        for port_number in 1..=hub.hub_descriptors.bNbrPorts {
            let key = (address << 8) | port_number;
            self.port_devices.remove(&key);
        }

        log::info!("Hub {} removed", address);
        Ok(())
    }
}

/// USB Hub System Statistics
#[derive(Debug, Clone)]
pub struct UsbHubSystemStats {
    pub total_hubs: u8,
    pub total_ports: u8,
    pub connected_ports: u8,
    pub powered_hubs: u8,
    pub powered_ports: u8,
    pub power_management_enabled: bool,
}

impl Default for UsbHubManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UsbHubConfig {
    fn default() -> Self {
        Self {
            enable_power_management: true,
            port_reset_timeout_ms: 10000,
            enumeration_timeout_ms: 5000,
            auto_power_on: true,
            power_cycling_enabled: false,
            overcurrent_handling: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hub_device_creation() {
        let descriptors = UsbHubDescriptor {
            bLength: 9,
            bDescriptorType: 0x29,
            bNbrPorts: 4,
            wHubCharacteristics: 0x0001,
            bPwrOn2PwrGood: 100,
            bHubContrCurrent: 100,
            deviceRemovable: 0x00,
            portPwrCtrlMask: 0xFF,
        };

        let hub = UsbHubDevice::new(1, descriptors);
        assert_eq!(hub.address, 1);
        assert_eq!(hub.hub_descriptors.bNbrPorts, 4);
        assert_eq!(hub.hub_state, UsbHubState::Uninitialized);
    }

    #[test]
    fn test_hub_manager_creation() {
        let manager = UsbHubManager::new();
        assert!(!manager.is_initialized());
        assert_eq!(manager.get_hubs().len(), 0);
    }

    #[test]
    fn test_port_device_creation() {
        let port_device = UsbPortDevice {
            port_number: 1,
            hub_address: 1,
            device_address: None,
            speed: UsbSpeed::Full,
            connection_detected: false,
            reset_complete: false,
            device_enumerated: false,
            power_state: UsbPowerState::Active,
            last_activity: 0,
        };

        assert_eq!(port_device.port_number, 1);
        assert_eq!(port_device.hub_address, 1);
        assert_eq!(port_device.connection_detected, false);
    }

    #[test]
    fn test_hub_registration() {
        let mut manager = UsbHubManager::new();
        let descriptors = UsbHubDescriptor {
            bLength: 9,
            bDescriptorType: 0x29,
            bNbrPorts: 2,
            wHubCharacteristics: 0x0001,
            bPwrOn2PwrGood: 50,
            bHubContrCurrent: 50,
            deviceRemovable: 0x00,
            portPwrCtrlMask: 0xFF,
        };

        let result = manager.register_hub(1, descriptors);
        assert!(result.is_ok());
        assert_eq!(manager.get_hubs().len(), 1);
    }
}