//! USB Host Controller Management Module
//! 
//! Provides unified management of different USB host controllers:
//! - xHCI (USB 3.0+)
//! - EHCI (USB 2.0)
//! - OHCI (USB 1.0/1.1)
//!
//! This module handles controller discovery, initialization, and routing.

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// Host Controller Type identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbHostControllerType {
    XHCI,
    EHCI,
    OHCI,
    UHCI,
    Unknown,
}

/// USB Host Controller Manager
pub struct UsbHostControllerManager {
    pub controllers: BTreeMap<u8, UsbHostController>,
    pub controller_types: BTreeMap<u8, UsbHostControllerType>,
    pub active_controllers: Vec<u8>,
    pub port_assignments: BTreeMap<u8, PortAssignment>,
    pub max_slots: u8,
    pub initialized: bool,
}

/// Port assignment information
#[derive(Debug, Clone)]
pub struct PortAssignment {
    pub port_number: u8,
    pub controller_id: u8,
    pub speed: UsbSpeed,
    pub device_connected: bool,
    pub device_address: Option<u8>,
}

/// USB Host Controller Discovery Result
#[derive(Debug)]
pub struct UsbControllerDiscovery {
    pub controller_id: u8,
    pub base_address: u64,
    pub controller_type: UsbHostControllerType,
    pub max_ports: u8,
    pub max_slots: u8,
    pub capabilities: Vec<String>,
}

/// USB Host Controller Manager Implementation
impl UsbHostControllerManager {
    /// Create a new host controller manager
    pub fn new() -> Self {
        Self {
            controllers: BTreeMap::new(),
            controller_types: BTreeMap::new(),
            active_controllers: Vec::new(),
            port_assignments: BTreeMap::new(),
            max_slots: 0,
            initialized: false,
        }
    }

    /// Initialize the manager
    pub fn initialize(&mut self) -> UsbResult<()> {
        self.initialized = true;
        log::info!("USB Host Controller Manager initialized");
        Ok(())
    }

    /// Discover all host controllers in the system
    pub fn discover_controllers(&mut self) -> UsbResult<Vec<UsbControllerDiscovery>> {
        let mut discoveries = Vec::new();

        // Common USB controller addresses for x86_64 systems
        let common_addresses = [
            0xFED90000, // xHCI #1
            0xFED80000, // xHCI #2
            0xFEC00000, // EHCI #1
            0xFEC80000, // EHCI #2
            0xFEC10000, // OHCI #1
            0xFEC20000, // OHCI #2
            0xFEC40000, // UHCI #1
            0xFEC50000, // UHCI #2
        ];

        let mut controller_id = 0;

        for &base_address in &common_addresses {
            match self.discover_controller_at_address(base_address)? {
                Some(discovery) => {
                    discoveries.push(discovery);
                    controller_id += 1;
                }
                None => continue,
            }
        }

        log::info!("Discovered {} USB host controllers", discoveries.len());
        for discovery in &discoveries {
            log::info!("  Controller {}: {} at {:#x}", 
                      discovery.controller_id,
                      match discovery.controller_type {
                          UsbHostControllerType::XHCI => "xHCI",
                          UsbHostControllerType::EHCI => "EHCI",
                          UsbHostControllerType::OHCI => "OHCI",
                          UsbHostControllerType::UHCI => "UHCI",
                          UsbHostControllerType::Unknown => "Unknown",
                      },
                      discovery.base_address);
        }

        Ok(discoveries)
    }

    /// Discover a controller at a specific PCI/MMIO address
    pub fn discover_controller_at_address(&self, base_address: u64) -> UsbResult<Option<UsbControllerDiscovery>> {
        unsafe {
            let cap_reg = base_address as *const u8;
            let cap_length = *cap_reg;

            if cap_length == 0 || cap_length == 0xFF {
                return Ok(None); // Invalid capability register
            }

            // Read capability register to determine controller type
            let cap_header = core::ptr::read_unaligned(base_address as *const u32);

            // Check for xHCI (eXtensible Host Controller Interface)
            if cap_header == 0x46434441 { // "ACDF" - xHCI signature
                return Ok(Some(UsbControllerDiscovery {
                    controller_id: 0,
                    base_address,
                    controller_type: UsbHostControllerType::XHCI,
                    max_ports: 15,
                    max_slots: 255,
                    capabilities: vec![
                        "SuperSpeed".to_string(),
                        "Multiple Transfer Rings".to_string(),
                        "Stream Mode".to_string(),
                    ],
                }));
            }

            // Check for EHCI (Enhanced Host Controller Interface)
            let hci_version = (cap_header >> 16) & 0xFFFF;
            if hci_version >= 0x0100 && hci_version < 0x0200 {
                // Additional EHCI identification can be done here
                return Ok(Some(UsbControllerDiscovery {
                    controller_id: 0,
                    base_address,
                    controller_type: UsbHostControllerType::EHCI,
                    max_ports: 8,
                    max_slots: 128,
                    capabilities: vec![
                        "High Speed".to_string(),
                        "Asynchronous Scheduling".to_string(),
                        "Isochronous Scheduling".to_string(),
                    ],
                }));
            }

            // Check for OHCI (Open Host Controller Interface)
            let ohci_revision = *cap_reg;
            if ohci_revision == 0x10 { // OHCI revision 1.0
                return Ok(Some(UsbControllerDiscovery {
                    controller_id: 0,
                    base_address,
                    controller_type: UsbHostControllerType::OHCI,
                    max_ports: 15,
                    max_slots: 64,
                    capabilities: vec![
                        "Full/Low Speed".to_string(),
                        "Interrupt-driven".to_string(),
                        "Legacy Support".to_string(),
                    ],
                }));
            }

            Ok(None)
        }
    }

    /// Register a discovered controller
    pub fn register_controller(&mut self, discovery: UsbControllerDiscovery) -> UsbResult<()> {
        let controller_id = self.controllers.len() as u8;

        match discovery.controller_type {
            UsbHostControllerType::XHCI => {
                let controller = UsbHostController::XHCI(XhciController::new(discovery.base_address));
                self.controllers.insert(controller_id, controller);
            }
            UsbHostControllerType::EHCI => {
                let controller = UsbHostController::EHCI(EhciController::new(discovery.base_address));
                self.controllers.insert(controller_id, controller);
            }
            UsbHostControllerType::OHCI => {
                let controller = UsbHostController::OHCI(OhciController::new(discovery.base_address));
                self.controllers.insert(controller_id, controller);
            }
            _ => {
                log::warn!("Unsupported controller type: {:?}", discovery.controller_type);
                return Err(UsbDriverError::UnsupportedFeature);
            }
        }

        self.controller_types.insert(controller_id, discovery.controller_type);
        self.max_slots = self.max_slots.max(discovery.max_slots);

        log::info!("Registered controller {}: {:?}", controller_id, discovery.controller_type);
        Ok(())
    }

    /// Initialize all registered controllers
    pub fn initialize_controllers(&mut self) -> UsbResult<()> {
        for (&controller_id, controller) in &mut self.controllers {
            match controller {
                UsbHostController::XHCI(xhci) => {
                    log::info!("Initializing xHCI controller {}", controller_id);
                    xhci.initialize()?;
                }
                UsbHostController::EHCI(ehci) => {
                    log::info!("Initializing EHCI controller {}", controller_id);
                    ehci.initialize()?;
                }
                UsbHostController::OHCI(ohci) => {
                    log::info!("Initializing OHCI controller {}", controller_id);
                    ohci.initialize()?;
                }
            }
            
            self.active_controllers.push(controller_id);
        }

        self.assign_ports();
        log::info!("All controllers initialized successfully");
        Ok(())
    }

    /// Assign ports to controllers
    fn assign_ports(&mut self) {
        self.port_assignments.clear();

        for (&controller_id, controller) in &self.controllers {
            let max_ports = match controller {
                UsbHostController::XHCI(xhci) => xhci.max_ports,
                UsbHostController::EHCI(ehci) => ehci.max_ports,
                UsbHostController::OHCI(ohci) => ohci.max_ports,
            };

            for port_number in 1..=max_ports {
                let assignment = PortAssignment {
                    port_number,
                    controller_id,
                    speed: match controller {
                        UsbHostController::XHCI(_) => UsbSpeed::Super,
                        UsbHostController::EHCI(_) => UsbSpeed::High,
                        UsbHostController::OHCI(_) => UsbSpeed::Full,
                    },
                    device_connected: false,
                    device_address: None,
                };

                self.port_assignments.insert(port_number, assignment);
            }
        }
    }

    /// Get controller by ID
    pub fn get_controller(&self, controller_id: u8) -> UsbResult<&UsbHostController> {
        self.controllers.get(&controller_id)
            .ok_or(UsbDriverError::DeviceNotFound { address: controller_id })
    }

    /// Get mutable controller by ID
    pub fn get_controller_mut(&mut self, controller_id: u8) -> UsbResult<&mut UsbHostController> {
        self.controllers.get_mut(&controller_id)
            .ok_or(UsbDriverError::DeviceNotFound { address: controller_id })
    }

    /// Get all active controllers
    pub fn get_active_controllers(&self) -> &[u8] {
        &self.active_controllers
    }

    /// Get all port assignments
    pub fn get_port_assignments(&self) -> &BTreeMap<u8, PortAssignment> {
        &self.port_assignments
    }

    /// Get port assignment for a specific port
    pub fn get_port_assignment(&self, port_number: u8) -> UsbResult<&PortAssignment> {
        self.port_assignments.get(&port_number)
            .ok_or(UsbDriverError::DeviceNotFound { address: port_number })
    }

    /// Find controller by port number
    pub fn find_controller_by_port(&self, port_number: u8) -> UsbResult<u8> {
        let assignment = self.get_port_assignment(port_number)?;
        Ok(assignment.controller_id)
    }

    /// Check if controller supports port
    pub fn controller_supports_port(&self, controller_id: u8, port_number: u8) -> bool {
        if let Some(assignment) = self.port_assignments.get(&port_number) {
            assignment.controller_id == controller_id
        } else {
            false
        }
    }

    /// Route USB transaction to appropriate controller
    pub fn route_transaction(&self, port_number: u8, transaction_data: &[u8]) -> UsbResult<(&UsbHostController, &[u8])> {
        let controller_id = self.find_controller_by_port(port_number)?;
        let controller = self.get_controller(controller_id)?;

        Ok((controller, transaction_data))
    }

    /// Enable all schedules (periodic, control, bulk)
    pub fn enable_all_schedules(&mut self) -> UsbResult<()> {
        for controller in self.controllers.values_mut() {
            match controller {
                UsbHostController::XHCI(xhci) => {
                    xhci.enable_periodic_schedule()?;
                    xhci.enable_async_schedule()?;
                }
                UsbHostController::EHCI(ehci) => {
                    ehci.enable_periodic_schedule()?;
                    ehci.enable_async_schedule()?;
                }
                UsbHostController::OHCI(ohci) => {
                    ohci.enable_periodic_schedule()?;
                    ohci.enable_control_schedule()?;
                    ohci.enable_bulk_schedule()?;
                }
            }
        }

        log::info!("All USB schedules enabled");
        Ok(())
    }

    /// Get system-wide statistics
    pub fn get_system_stats(&self) -> UsbControllerStats {
        let mut total_stats = UsbControllerStats {
            total_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            bytes_transferred: 0,
            error_count: 0,
            last_error: None,
        };

        for controller in self.controllers.values() {
            let stats = match controller {
                UsbHostController::XHCI(xhci) => xhci.get_stats(),
                UsbHostController::EHCI(ehci) => ehci.get_stats(),
                UsbHostController::OHCI(ohci) => ohci.get_stats(),
            };

            total_stats.total_transactions += stats.total_transactions;
            total_stats.successful_transactions += stats.successful_transactions;
            total_stats.failed_transactions += stats.failed_transactions;
            total_stats.bytes_transferred += stats.bytes_transferred;
            total_stats.error_count += stats.error_count;
        }

        total_stats
    }

    /// Perform system-wide controller reset
    pub fn reset_all_controllers(&mut self) -> UsbResult<()> {
        for controller in self.controllers.values_mut() {
            match controller {
                UsbHostController::XHCI(xhci) => {
                    xhci.reset()?;
                }
                UsbHostController::EHCI(ehci) => {
                    ehci.reset()?;
                }
                UsbHostController::OHCI(ohci) => {
                    ohci.reset()?;
                }
            }
        }

        log::info!("All controllers reset");
        Ok(())
    }

    /// Get controller type for a given controller ID
    pub fn get_controller_type(&self, controller_id: u8) -> UsbResult<UsbHostControllerType> {
        self.controller_types.get(&controller_id)
            .copied()
            .ok_or(UsbDriverError::DeviceNotFound { address: controller_id })
    }

    /// Check if manager is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get total number of controllers
    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }

    /// Get maximum number of slots across all controllers
    pub fn get_max_slots(&self) -> u8 {
        self.max_slots
    }
}

impl Default for UsbHostControllerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_controller_manager_creation() {
        let manager = UsbHostControllerManager::new();
        assert!(!manager.is_initialized());
        assert_eq!(manager.controller_count(), 0);
    }

    #[test]
    fn test_port_assignment() {
        let manager = UsbHostControllerManager::new();
        
        let assignment = PortAssignment {
            port_number: 1,
            controller_id: 0,
            speed: UsbSpeed::High,
            device_connected: false,
            device_address: None,
        };

        assert_eq!(assignment.port_number, 1);
        assert_eq!(assignment.controller_id, 0);
        assert_eq!(assignment.speed, UsbSpeed::High);
    }

    #[test]
    fn test_controller_discovery() {
        let mut manager = UsbHostControllerManager::new();
        
        // Test discovery at non-existent address
        let result = manager.discover_controller_at_address(0x00000000);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}