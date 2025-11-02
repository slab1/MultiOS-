//! MultiOS Advanced Networking Drivers
//!
//! This crate provides comprehensive networking support including:
//! - Wi-Fi drivers with 802.11n/ac/ax support
//! - Wireless security protocols (WPA2, WPA3)
//! - Advanced networking stack integration
//! - Wi-Fi scanning and connection management
//! - Ethernet driver improvements
//! - Network device hotplug support
//! - Wireless debugging and monitoring tools

#![no_std]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]

use core::fmt;
use multios_hal::{Device, InterruptHandler};
use multios_scheduler::{Task, TaskPriority};
use multios_ipc::{Channel, Message};
use spin::RwLock;

// Re-export main components
pub use wifi::*;
pub use ethernet::*;
pub use security::*;
pub use stack::*;
pub use scanning::*;
pub use hotplug::*;
pub use debugging::*;
pub use config::*;

mod wifi;
mod ethernet;
mod security;
mod stack;
mod scanning;
mod hotplug;
mod debugging;
mod config;
mod protocols;

use multios_hal::DeviceManager;
use multios_memory::{MemoryManager, PhysicalAddress, VirtualAddress};

/// Global networking manager instance
static NETWORKING: RwLock<Option<NetworkingManager>> = RwLock::new(None);

/// Initialize the networking subsystem
pub fn init(memory_manager: &MemoryManager, device_manager: &DeviceManager) -> Result<(), NetworkingError> {
    let mut lock = NETWORKING.write();
    if lock.is_some() {
        return Err(NetworkingError::AlreadyInitialized);
    }
    
    let mut manager = NetworkingManager::new(memory_manager, device_manager)?;
    
    // Initialize Wi-Fi subsystem
    #[cfg(feature = "wifi")]
    manager.initialize_wifi()?;
    
    // Initialize Ethernet subsystem
    #[cfg(feature = "ethernet")]
    manager.initialize_ethernet()?;
    
    // Initialize network stack
    manager.initialize_stack()?;
    
    // Start background tasks
    manager.start_background_tasks()?;
    
    *lock = Some(manager);
    Ok(())
}

/// Get the global networking manager
pub fn get_manager() -> Option<multios_scheduler::RwLockReadGuard<'static, Option<NetworkingManager>>> {
    NETWORKING.read().as_ref().map(|_| multios_scheduler::RwLockReadGuard::new(NETWORKING.read()))
}

/// Main networking manager
pub struct NetworkingManager {
    memory_manager: &'static MemoryManager,
    device_manager: &'static DeviceManager,
    wifi_manager: Option<WifiManager>,
    ethernet_manager: Option<EthernetManager>,
    network_stack: NetworkStack,
    device_hotplug: DeviceHotplugHandler,
    scanning: WifiScanner,
    debug_monitor: DebugMonitor,
}

impl NetworkingManager {
    /// Create a new networking manager
    pub fn new(
        memory_manager: &MemoryManager,
        device_manager: &DeviceManager,
    ) -> Result<Self, NetworkingError> {
        Ok(Self {
            memory_manager,
            device_manager,
            wifi_manager: None,
            ethernet_manager: None,
            network_stack: NetworkStack::new(),
            device_hotplug: DeviceHotplugHandler::new(),
            scanning: WifiScanner::new(),
            debug_monitor: DebugMonitor::new(),
        })
    }
    
    /// Initialize Wi-Fi subsystem
    #[cfg(feature = "wifi")]
    fn initialize_wifi(&mut self) -> Result<(), NetworkingError> {
        self.wifi_manager = Some(WifiManager::new()?);
        info!("Wi-Fi subsystem initialized");
        Ok(())
    }
    
    /// Initialize Ethernet subsystem
    #[cfg(feature = "ethernet")]
    fn initialize_ethernet(&mut self) -> Result<(), NetworkingError> {
        self.ethernet_manager = Some(EthernetManager::new()?);
        info!("Ethernet subsystem initialized");
        Ok(())
    }
    
    /// Initialize network stack
    fn initialize_stack(&mut self) -> Result<(), NetworkingError> {
        self.network_stack.initialize()?;
        info!("Network stack initialized");
        Ok(())
    }
    
    /// Start background networking tasks
    fn start_background_tasks(&mut self) -> Result<(), NetworkingError> {
        // Device monitoring task
        let device_task = Task::new(
            "network_device_monitor",
            TaskPriority::High,
            Box::new(self.device_hotplug.monitor_devices()),
        );
        multios_scheduler::schedule_task(device_task)?;
        
        // Wi-Fi scanning task (if enabled)
        #[cfg(feature = "wifi")]
        if let Some(ref wifi_manager) = self.wifi_manager {
            let scan_task = Task::new(
                "wifi_background_scan",
                TaskPriority::Normal,
                Box::new(self.scanning.periodic_scan(wifi_manager)),
            );
            multios_scheduler::schedule_task(scan_task)?;
        }
        
        // Network debugging task (if enabled)
        #[cfg(feature = "debugging")]
        if let Some(ref monitor) = self.debug_monitor {
            let debug_task = Task::new(
                "network_debug_monitor",
                TaskPriority::Low,
                Box::new(monitor.background_monitoring()),
            );
            multios_scheduler::schedule_task(debug_task)?;
        }
        
        info!("Background networking tasks started");
        Ok(())
    }
    
    /// Get Wi-Fi manager reference
    pub fn wifi_manager(&self) -> Result<&WifiManager, NetworkingError> {
        self.wifi_manager.as_ref().ok_or(NetworkingError::WifiNotEnabled)
    }
    
    /// Get Ethernet manager reference
    pub fn ethernet_manager(&self) -> Result<&EthernetManager, NetworkingError> {
        self.ethernet_manager.as_ref().ok_or(NetworkingError::EthernetNotEnabled)
    }
    
    /// Get network stack reference
    pub fn network_stack(&self) -> &NetworkStack {
        &self.network_stack
    }
    
    /// Get scanning manager reference
    pub fn scanning(&self) -> &WifiScanner {
        &self.scanning
    }
    
    /// Get debugging monitor reference
    pub fn debug_monitor(&self) -> Option<&DebugMonitor> {
        self.debug_monitor.as_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NetworkingError {
    AlreadyInitialized,
    WifiNotEnabled,
    EthernetNotEnabled,
    NotInitialized,
    DeviceNotFound,
    UnsupportedProtocol,
    HardwareError,
    MemoryAllocationFailed,
    InvalidConfiguration,
    SecurityError,
    ConnectionFailed,
}

impl fmt::Display for NetworkingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkingError::AlreadyInitialized => write!(f, "Networking already initialized"),
            NetworkingError::WifiNotEnabled => write!(f, "Wi-Fi not enabled"),
            NetworkingError::EthernetNotEnabled => write!(f, "Ethernet not enabled"),
            NetworkingError::NotInitialized => write!(f, "Networking not initialized"),
            NetworkingError::DeviceNotFound => write!(f, "Network device not found"),
            NetworkingError::UnsupportedProtocol => write!(f, "Unsupported protocol"),
            NetworkingError::HardwareError => write!(f, "Hardware error"),
            NetworkingError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            NetworkingError::InvalidConfiguration => write!(f, "Invalid configuration"),
            NetworkingError::SecurityError => write!(f, "Security error"),
            NetworkingError::ConnectionFailed => write!(f, "Connection failed"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    fn test_runner(tests: &[&dyn Fn()]) {
        for test in tests {
            test();
        }
    }
    
    #[test]
    fn test_networking_manager_creation() {
        // This would need proper test setup with mock HAL
        // For now, just verify compilation
    }
    
    #[test]
    fn test_error_types() {
        assert_eq!(format!("{}", NetworkingError::NotInitialized), "Networking not initialized");
    }
}

/// Logging macros (simplified for no_std)
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => { /* TODO: Implement proper logging */ };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => { /* TODO: Implement proper logging */ };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => { /* TODO: Implement proper logging */ };
}