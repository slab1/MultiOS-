//! Power Management Module
//! 
//! Provides comprehensive power management capabilities for device drivers,
//! including power state transitions, power domain management, and wake-up
//! event handling.

use crate::AdvancedDriverId;
use crate::AdvancedDriverError::{self, *};
use alloc::collections::BTreeMap;
use log::{debug, warn, info, error};

/// Power states for devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Off,           // Device is completely off
    Standby,       // Device is in low-power standby
    Sleep,         // Device is in sleep state
    Hibernate,     // Device is hibernating
    Active,        // Device is fully active
    Idle,          // Device is active but idle
}

/// Power transition information
#[derive(Debug, Clone)]
pub struct PowerTransition {
    pub from_state: PowerState,
    pub to_state: PowerState,
    pub duration_ms: u64,
    pub power_consumed_mw: u32,
    pub power_saved_mw: u32,
}

/// Power domain configuration
#[derive(Debug, Clone)]
pub struct PowerDomain {
    pub id: u32,
    pub name: &'static str,
    pub devices: Vec<AdvancedDriverId>,
    pub parent_domain: Option<u32>,
    pub default_state: PowerState,
    pub wake_up_devices: Vec<AdvancedDriverId>,
}

/// Power management policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPolicy {
    Performance,   // Maximize performance, ignore power
    Balanced,      // Balance performance and power
    PowerSave,     // Maximize power saving
    Custom,        // Custom policy
}

/// Power management event
#[derive(Debug, Clone)]
pub enum PowerManagementEvent {
    StateChanged(AdvancedDriverId, PowerState, PowerState),
    TransitionStarted(AdvancedDriverId, PowerTransition),
    TransitionCompleted(AdvancedDriverId, PowerTransition),
    TransitionFailed(AdvancedDriverId, PowerTransition, AdvancedDriverError),
    WakeUpEvent(AdvancedDriverId, PowerState),
    LowPowerWarning(AdvancedDriverId),
    HighPowerConsumption(AdvancedDriverId),
    PolicyChanged(PowerPolicy),
    DomainStateChanged(u32, PowerState),
}

/// Power manager
pub struct PowerManager {
    driver_states: BTreeMap<AdvancedDriverId, PowerState>,
    power_domains: BTreeMap<u32, PowerDomain>,
    transition_history: Vec<PowerTransition>,
    current_policy: PowerPolicy,
    idle_threshold_ms: u64,
    auto_sleep_enabled: bool,
    event_callbacks: Vec<fn(PowerManagementEvent)>,
    total_power_saved_mw: u64,
    total_power_consumed_mwh: u64,
}

impl PowerManager {
    /// Create a new power manager
    pub fn new() -> Self {
        info!("Initializing Power Manager");
        
        let manager = Self {
            driver_states: BTreeMap::new(),
            power_domains: BTreeMap::new(),
            transition_history: Vec::new(),
            current_policy: PowerPolicy::Balanced,
            idle_threshold_ms: 5000, // 5 seconds default
            auto_sleep_enabled: true,
            event_callbacks: Vec::new(),
            total_power_saved_mw: 0,
            total_power_consumed_mwh: 0,
        };
        
        info!("Power Manager initialized with {:?} policy", manager.current_policy);
        manager
    }

    /// Enable power management for a driver
    pub fn enable_power_management(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        debug!("Enabling power management for driver {:?}", driver_id);
        
        self.driver_states.insert(driver_id, PowerState::Active);
        info!("Power management enabled for driver {:?}", driver_id);
        Ok(())
    }

    /// Disable power management for a driver
    pub fn disable_power_management(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        debug!("Disabling power management for driver {:?}", driver_id);
        
        self.driver_states.remove(&driver_id);
        info!("Power management disabled for driver {:?}", driver_id);
        Ok(())
    }

    /// Transition driver to a specific power state
    pub fn transition_to_state(&mut self, driver_id: AdvancedDriverId, target_state: PowerState) -> Result<PowerTransition, AdvancedDriverError> {
        debug!("Transitioning driver {:?} to power state {:?}", driver_id, target_state);
        
        let current_state = self.driver_states.get(&driver_id)
            .copied()
            .unwrap_or(PowerState::Off);
        
        // Validate transition
        if !self.can_transition(current_state, target_state) {
            warn!("Invalid power state transition for driver {:?}: {:?} -> {:?}", 
                  driver_id, current_state, target_state);
            return Err(PowerTransitionFailed);
        }
        
        // Create transition record
        let transition = PowerTransition {
            from_state: current_state,
            to_state: target_state,
            duration_ms: self.calculate_transition_duration(current_state, target_state),
            power_consumed_mw: self.calculate_power_consumption(target_state),
            power_saved_mw: if current_state == PowerState::Active {
                self.calculate_power_consumption(current_state) - self.calculate_power_consumption(target_state)
            } else {
                0
            },
        };
        
        // Perform actual state transition
        self.perform_state_transition(driver_id, &transition)?;
        
        // Update state
        self.driver_states.insert(driver_id, target_state);
        
        // Record transition
        self.transition_history.push(transition.clone());
        
        // Update statistics
        self.total_power_saved_mw += transition.power_saved_mw as u64;
        
        // Notify callbacks
        self.notify_event(PowerManagementEvent::TransitionCompleted(driver_id, transition.clone()));
        
        info!("Driver {:?} transitioned to power state {:?}", driver_id, target_state);
        
        Ok(transition)
    }

    /// Check if a power state transition is valid
    fn can_transition(&self, from_state: PowerState, to_state: PowerState) -> bool {
        match (from_state, to_state) {
            // Valid transitions
            (PowerState::Off, PowerState::Standby) => true,
            (PowerState::Off, PowerState::Active) => true,
            (PowerState::Standby, PowerState::Off) => true,
            (PowerState::Standby, PowerState::Active) => true,
            (PowerState::Standby, PowerState::Sleep) => true,
            (PowerState::Active, PowerState::Idle) => true,
            (PowerState::Active, PowerState::Sleep) => true,
            (PowerState::Active, PowerState::Standby) => true,
            (PowerState::Active, PowerState::Off) => true,
            (PowerState::Idle, PowerState::Active) => true,
            (PowerState::Idle, PowerState::Sleep) => true,
            (PowerState::Sleep, PowerState::Active) => true,
            (PowerState::Sleep, PowerState::Standby) => true,
            (PowerState::Hibernate, PowerState::Active) => true,
            
            // Self-transitions
            (state, state) => true,
            
            // Invalid transitions
            _ => false,
        }
    }

    /// Calculate transition duration
    fn calculate_transition_duration(&self, from_state: PowerState, to_state: PowerState) -> u64 {
        // Simplified duration calculation
        match (from_state, to_state) {
            (PowerState::Off, PowerState::Active) => 100,
            (PowerState::Active, PowerState::Off) => 50,
            (PowerState::Active, PowerState::Sleep) => 200,
            (PowerState::Sleep, PowerState::Active) => 500,
            (PowerState::Hibernate, PowerState::Active) => 2000,
            _ => 20,
        }
    }

    /// Calculate power consumption for a state
    fn calculate_power_consumption(&self, state: PowerState) -> u32 {
        match state {
            PowerState::Off => 0,
            PowerState::Standby => 10,
            PowerState::Sleep => 50,
            PowerState::Hibernate => 5,
            PowerState::Idle => 200,
            PowerState::Active => 500,
        }
    }

    /// Perform the actual state transition
    fn perform_state_transition(&self, driver_id: AdvancedDriverId, transition: &PowerTransition) -> Result<(), AdvancedDriverError> {
        debug!("Performing state transition for driver {:?}: {:?} -> {:?}", 
               driver_id, transition.from_state, transition.to_state);
        
        // Notify transition start
        self.notify_event(PowerManagementEvent::TransitionStarted(driver_id, transition.clone()));
        
        // Simulate transition time
        for _ in 0..(transition.duration_ms / 10) {
            // Busy wait simulation
        }
        
        Ok(())
    }

    /// Get current power state of a driver
    pub fn get_driver_state(&self, driver_id: AdvancedDriverId) -> Option<PowerState> {
        self.driver_states.get(&driver_id).copied()
    }

    /// Get power statistics
    pub fn get_power_statistics(&self) -> PowerStatistics {
        let mut state_counts = BTreeMap::new();
        for &state in self.driver_states.values() {
            *state_counts.entry(state).or_insert(0) += 1;
        }
        
        PowerStatistics {
            driver_states: state_counts,
            total_transitions: self.transition_history.len(),
            total_power_saved_mw: self.total_power_saved_mw,
            total_power_consumed_mwh: self.total_power_consumed_mwh,
            current_policy: self.current_policy,
            auto_sleep_enabled: self.auto_sleep_enabled,
        }
    }

    /// Set power management policy
    pub fn set_policy(&mut self, policy: PowerPolicy) -> Result<(), AdvancedDriverError> {
        debug!("Setting power policy to {:?}", policy);
        
        let old_policy = self.current_policy;
        self.current_policy = policy;
        
        self.notify_event(PowerManagementEvent::PolicyChanged(policy));
        
        info!("Power policy changed from {:?} to {:?}", old_policy, policy);
        Ok(())
    }

    /// Create a power domain
    pub fn create_power_domain(&mut self, domain: PowerDomain) -> Result<(), AdvancedDriverError> {
        debug!("Creating power domain {}: {}", domain.id, domain.name);
        
        self.power_domains.insert(domain.id, domain);
        Ok(())
    }

    /// Move driver to a power domain
    pub fn add_driver_to_domain(&mut self, driver_id: AdvancedDriverId, domain_id: u32) -> Result<(), AdvancedDriverError> {
        debug!("Adding driver {:?} to power domain {}", driver_id, domain_id);
        
        let domain = self.power_domains.get_mut(&domain_id)
            .ok_or(DeviceNotFound)?;
        
        if !domain.devices.contains(&driver_id) {
            domain.devices.push(driver_id);
        }
        
        Ok(())
    }

    /// Transition an entire power domain
    pub fn transition_domain(&mut self, domain_id: u32, target_state: PowerState) -> Result<(), AdvancedDriverError> {
        debug!("Transitioning power domain {} to state {:?}", domain_id, target_state);
        
        let domain = self.power_domains.get(&domain_id)
            .ok_or(DeviceNotFound)?;
        
        let mut transitions = Vec::new();
        
        for &driver_id in &domain.devices {
            let transition = self.transition_to_state(driver_id, target_state)?;
            transitions.push(transition);
        }
        
        self.notify_event(PowerManagementEvent::DomainStateChanged(domain_id, target_state));
        
        info!("Power domain {} transitioned to state {:?}", domain_id, target_state);
        Ok(())
    }

    /// Get power-managed devices count
    pub fn get_power_managed_count(&self) -> usize {
        self.driver_states.len()
    }

    /// Register power management event callback
    pub fn register_event_callback(&mut self, callback: fn(PowerManagementEvent)) {
        self.event_callbacks.push(callback);
    }

    /// Notify all event callbacks
    fn notify_event(&self, event: PowerManagementEvent) {
        for callback in &self.event_callbacks {
            callback(event.clone());
        }
    }

    /// Enable/disable automatic sleep
    pub fn set_auto_sleep_enabled(&mut self, enabled: bool) -> Result<(), AdvancedDriverError> {
        debug!("Setting auto sleep to {}", enabled);
        self.auto_sleep_enabled = enabled;
        Ok(())
    }

    /// Check if device should enter sleep state based on policy
    pub fn should_enter_sleep(&self, driver_id: AdvancedDriverId, idle_time_ms: u64) -> bool {
        if !self.auto_sleep_enabled {
            return false;
        }
        
        if idle_time_ms < self.idle_threshold_ms {
            return false;
        }
        
        match self.current_policy {
            PowerPolicy::Performance => false,
            PowerPolicy::Balanced => idle_time_ms > 10000,
            PowerPolicy::PowerSave => idle_time_ms > 2000,
            PowerPolicy::Custom => {
                // Custom logic could be implemented here
                idle_time_ms > 5000
            }
        }
    }
}

/// Power statistics
#[derive(Debug, Clone)]
pub struct PowerStatistics {
    pub driver_states: BTreeMap<PowerState, usize>,
    pub total_transitions: usize,
    pub total_power_saved_mw: u64,
    pub total_power_consumed_mwh: u64,
    pub current_policy: PowerPolicy,
    pub auto_sleep_enabled: bool,
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_state_transitions() {
        let mut manager = PowerManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Enable power management
        assert!(manager.enable_power_management(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(PowerState::Active));
        
        // Transition to sleep
        let transition = manager.transition_to_state(driver_id, PowerState::Sleep).unwrap();
        assert_eq!(transition.from_state, PowerState::Active);
        assert_eq!(transition.to_state, PowerState::Sleep);
        assert!(transition.power_saved_mw > 0);
        
        // Transition back to active
        assert!(manager.transition_to_state(driver_id, PowerState::Active).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(PowerState::Active));
    }

    #[test]
    fn test_invalid_transitions() {
        let mut manager = PowerManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.enable_power_management(driver_id).unwrap();
        
        // Invalid: Off to Sleep (missing intermediate step)
        assert!(manager.transition_to_state(driver_id, PowerState::Sleep).is_ok());
        
        // Invalid: Sleep directly to Off
        assert!(manager.transition_to_state(driver_id, PowerState::Off).is_err());
    }

    #[test]
    fn test_power_policy() {
        let mut manager = PowerManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.enable_power_management(driver_id).unwrap();
        
        // Test different policies
        manager.set_policy(PowerPolicy::Performance).unwrap();
        assert!(!manager.should_enter_sleep(driver_id, 10000));
        
        manager.set_policy(PowerPolicy::PowerSave).unwrap();
        assert!(manager.should_enter_sleep(driver_id, 5000));
        
        manager.set_policy(PowerPolicy::Balanced).unwrap();
        assert!(!manager.should_enter_sleep(driver_id, 2000));
        assert!(manager.should_enter_sleep(driver_id, 15000));
    }

    #[test]
    fn test_power_statistics() {
        let mut manager = PowerManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.enable_power_management(driver_id).unwrap();
        
        // Make some transitions
        manager.transition_to_state(driver_id, PowerState::Sleep).unwrap();
        manager.transition_to_state(driver_id, PowerState::Active).unwrap();
        
        let stats = manager.get_power_statistics();
        assert_eq!(stats.total_transitions, 2);
        assert!(stats.total_power_saved_mw > 0);
        assert!(stats.driver_states.contains_key(&PowerState::Active));
    }
}
