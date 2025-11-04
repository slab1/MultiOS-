//! Driver Lifecycle Management
//! 
//! Manages the complete lifecycle of device drivers from registration through
//! unloading, including state tracking and transition validation.

use crate::AdvancedDriverId;
use crate::AdvancedDriverError::{self, *};
use alloc::collections::BTreeMap;
use log::{debug, warn, error, info};

/// Driver lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Unregistered,    // Driver not registered
    Registered,      // Driver registered but not loaded
    Loading,         // Driver is being loaded
    Loaded,          // Driver loaded but not active
    Active,          // Driver is fully active
    Suspending,      // Driver is being suspended
    Suspended,       // Driver is suspended
    Resuming,        // Driver is being resumed
    Unloading,       // Driver is being unloaded
    Error,           // Driver is in error state
    Recovering,      // Driver is being recovered
}

/// Lifecycle events
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    DriverRegistered(AdvancedDriverId),
    DriverUnregistered(AdvancedDriverId),
    LoadRequested(AdvancedDriverId),
    LoadCompleted(AdvancedDriverId),
    LoadFailed(AdvancedDriverId),
    ActivationRequested(AdvancedDriverId),
    ActivationCompleted(AdvancedDriverId),
    ActivationFailed(AdvancedDriverId),
    SuspendRequested(AdvancedDriverId),
    SuspendCompleted(AdvancedDriverId),
    SuspendFailed(AdvancedDriverId),
    ResumeRequested(AdvancedDriverId),
    ResumeCompleted(AdvancedDriverId),
    ResumeFailed(AdvancedDriverId),
    UnloadRequested(AdvancedDriverId),
    UnloadCompleted(AdvancedDriverId),
    UnloadFailed(AdvancedDriverId),
    ErrorDetected(AdvancedDriverId),
    RecoveryStarted(AdvancedDriverId),
    RecoveryCompleted(AdvancedDriverId),
    RecoveryFailed(AdvancedDriverId),
}

/// State transition
struct StateTransition {
    from_state: LifecycleState,
    to_state: LifecycleState,
    event: LifecycleEvent,
    timestamp: u64,
}

/// Driver lifecycle manager
pub struct DriverLifecycleManager {
    driver_states: BTreeMap<AdvancedDriverId, LifecycleState>,
    state_history: BTreeMap<AdvancedDriverId, Vec<StateTransition>>,
    transition_counters: BTreeMap<(LifecycleState, LifecycleState), u32>,
    event_callbacks: Vec<fn(LifecycleEvent)>,
}

impl DriverLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new() -> Self {
        Self {
            driver_states: BTreeMap::new(),
            state_history: BTreeMap::new(),
            transition_counters: BTreeMap::new(),
            event_callbacks: Vec::new(),
        }
    }

    /// Register a driver
    pub fn register_driver(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        debug!("Registering driver {:?} in lifecycle manager", driver_id);
        
        self.driver_states.insert(driver_id, LifecycleState::Registered);
        self.state_history.insert(driver_id, Vec::new());
        
        self.notify_event(LifecycleEvent::DriverRegistered(driver_id));
        Ok(())
    }

    /// Unregister a driver
    pub fn unregister_driver(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        debug!("Unregistering driver {:?} from lifecycle manager", driver_id);
        
        let current_state = self.driver_states.get(&driver_id)
            .copied()
            .unwrap_or(LifecycleState::Unregistered);
        
        // Cannot unregister active or loading drivers
        if current_state == LifecycleState::Active || current_state == LifecycleState::Loading {
            return Err(LifecycleTransitionFailed);
        }
        
        self.driver_states.insert(driver_id, LifecycleState::Unregistered);
        self.notify_event(LifecycleEvent::DriverUnregistered(driver_id));
        Ok(())
    }

    /// Transition to loading state
    pub fn transition_to_loading(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Loading, LifecycleEvent::LoadRequested(driver_id))
    }

    /// Transition to loaded state
    pub fn transition_to_loaded(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Loaded, LifecycleEvent::LoadCompleted(driver_id))
    }

    /// Transition to active state
    pub fn transition_to_active(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Active, LifecycleEvent::ActivationCompleted(driver_id))
    }

    /// Transition to suspending state
    pub fn transition_to_suspending(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Suspending, LifecycleEvent::SuspendRequested(driver_id))
    }

    /// Transition to suspended state
    pub fn transition_to_suspended(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Suspended, LifecycleEvent::SuspendCompleted(driver_id))
    }

    /// Transition to resuming state
    pub fn transition_to_resuming(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Resuming, LifecycleEvent::ResumeRequested(driver_id))
    }

    /// Transition to unloading state
    pub fn transition_to_unloading(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Unloading, LifecycleEvent::UnloadRequested(driver_id))
    }

    /// Transition to error state
    pub fn transition_to_error(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Error, LifecycleEvent::ErrorDetected(driver_id))
    }

    /// Transition to recovering state
    pub fn transition_to_recovering(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Recovering, LifecycleEvent::RecoveryStarted(driver_id))
    }

    /// Complete recovery
    pub fn complete_recovery(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_state(driver_id, LifecycleState::Active, LifecycleEvent::RecoveryCompleted(driver_id))
    }

    /// Transition to error and attempt recovery
    pub fn handle_error(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        self.transition_to_error(driver_id)?;
        
        let state_history = self.state_history.get(&driver_id)
            .map(|v| v.len())
            .unwrap_or(0);
        
        // Only attempt recovery if we have previous successful states
        if state_history > 1 {
            self.transition_to_recovering(driver_id)?;
        }
        
        Ok(())
    }

    /// Get current state of a driver
    pub fn get_driver_state(&self, driver_id: AdvancedDriverId) -> Option<LifecycleState> {
        self.driver_states.get(&driver_id).copied()
    }

    /// Check if driver can transition to target state
    pub fn can_transition(&self, driver_id: AdvancedDriverId, target_state: LifecycleState) -> bool {
        let current_state = match self.driver_states.get(&driver_id) {
            Some(state) => *state,
            None => return target_state == LifecycleState::Unregistered,
        };
        
        match (current_state, target_state) {
            // Valid transitions
            (LifecycleState::Registered, LifecycleState::Loading) => true,
            (LifecycleState::Loading, LifecycleState::Loaded) => true,
            (LifecycleState::Loaded, LifecycleState::Active) => true,
            (LifecycleState::Active, LifecycleState::Suspending) => true,
            (LifecycleState::Suspending, LifecycleState::Suspended) => true,
            (LifecycleState::Suspended, LifecycleState::Resuming) => true,
            (LifecycleState::Resuming, LifecycleState::Active) => true,
            (LifecycleState::Active, LifecycleState::Unloading) => true,
            (LifecycleState::Loaded, LifecycleState::Unloading) => true,
            (LifecycleState::Registered, LifecycleState::Unregistered) => true,
            (LifecycleState::Error, LifecycleState::Recovering) => true,
            (LifecycleState::Recovering, LifecycleState::Active) => true,
            
            // Error state transitions
            (LifecycleState::Error, _) => false,
            
            // Self-transitions (generally not useful but sometimes needed)
            (state, state) => true,
            
            // Invalid transitions
            _ => false,
        }
    }

    /// Get state transition history for a driver
    pub fn get_state_history(&self, driver_id: AdvancedDriverId) -> Option<&Vec<StateTransition>> {
        self.state_history.get(&driver_id)
    }

    /// Get state counts
    pub fn get_state_counts(&self) -> BTreeMap<LifecycleState, usize> {
        let mut counts = BTreeMap::new();
        
        for &state in self.driver_states.values() {
            *counts.entry(state).or_insert(0) += 1;
        }
        
        counts
    }

    /// Register event callback
    pub fn register_event_callback(&mut self, callback: fn(LifecycleEvent)) {
        self.event_callbacks.push(callback);
    }

    /// Internal state transition method
    fn transition_state(&mut self, driver_id: AdvancedDriverId, target_state: LifecycleState, event: LifecycleEvent) -> Result<(), AdvancedDriverError> {
        let current_state = self.driver_states.get(&driver_id)
            .copied()
            .ok_or(DeviceNotFound)?;
        
        // Validate transition
        if !self.can_transition(driver_id, target_state) {
            warn!("Invalid state transition for driver {:?}: {:?} -> {:?}", 
                  driver_id, current_state, target_state);
            return Err(LifecycleTransitionFailed);
        }
        
        // Record transition
        let transition = StateTransition {
            from_state: current_state,
            to_state: target_state,
            event: event.clone(),
            timestamp: 0, // TODO: Get actual timestamp
        };
        
        if let Some(history) = self.state_history.get_mut(&driver_id) {
            history.push(transition);
        }
        
        // Update driver state
        self.driver_states.insert(driver_id, target_state);
        
        // Update transition counters
        let transition_key = (current_state, target_state);
        *self.transition_counters.entry(transition_key).or_insert(0) += 1;
        
        debug!("Driver {:?} transitioned: {:?} -> {:?}", driver_id, current_state, target_state);
        
        // Notify callbacks
        self.notify_event(event);
        
        Ok(())
    }

    /// Notify all event callbacks
    fn notify_event(&self, event: LifecycleEvent) {
        for callback in &self.event_callbacks {
            callback(event.clone());
        }
    }

    /// Get transition statistics
    pub fn get_transition_stats(&self) -> BTreeMap<(LifecycleState, LifecycleState), u32> {
        self.transition_counters.clone()
    }

    /// Reset driver state (emergency operation)
    pub fn force_reset(&mut self, driver_id: AdvancedDriverId) -> Result<(), AdvancedDriverError> {
        debug!("Force resetting driver {:?} to Registered state", driver_id);
        
        self.driver_states.insert(driver_id, LifecycleState::Registered);
        self.notify_event(LifecycleEvent::DriverRegistered(driver_id));
        Ok(())
    }

    /// Get all drivers in a specific state
    pub fn get_drivers_in_state(&self, state: LifecycleState) -> Vec<AdvancedDriverId> {
        self.driver_states.iter()
            .filter_map(|(id, &driver_state)| {
                if driver_state == state {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for DriverLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_state_transitions() {
        let mut manager = DriverLifecycleManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Register driver
        assert!(manager.register_driver(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Registered));
        
        // Load driver
        assert!(manager.transition_to_loading(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Loading));
        
        assert!(manager.transition_to_loaded(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Loaded));
        
        // Activate driver
        assert!(manager.transition_to_active(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Active));
        
        // Unload driver
        assert!(manager.transition_to_unloading(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Unloading));
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut manager = DriverLifecycleManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Register driver
        manager.register_driver(driver_id).unwrap();
        
        // Invalid: Active without loading
        assert!(!manager.can_transition(driver_id, LifecycleState::Active));
        assert!(manager.transition_to_active(driver_id).is_err());
        
        // Invalid: Loading twice
        manager.transition_to_loading(driver_id).unwrap();
        assert!(manager.transition_to_loading(driver_id).is_err());
    }

    #[test]
    fn test_error_handling() {
        let mut manager = DriverLifecycleManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Register and activate driver
        manager.register_driver(driver_id).unwrap();
        manager.transition_to_loading(driver_id).unwrap();
        manager.transition_to_loaded(driver_id).unwrap();
        manager.transition_to_active(driver_id).unwrap();
        
        // Trigger error
        assert!(manager.handle_error(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Recovering));
        
        // Complete recovery
        assert!(manager.complete_recovery(driver_id).is_ok());
        assert_eq!(manager.get_driver_state(driver_id), Some(LifecycleState::Active));
    }

    #[test]
    fn test_state_history() {
        let mut manager = DriverLifecycleManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.register_driver(driver_id).unwrap();
        manager.transition_to_loading(driver_id).unwrap();
        manager.transition_to_loaded(driver_id).unwrap();
        
        let history = manager.get_state_history(driver_id).unwrap();
        assert_eq!(history.len(), 3); // Registered, Loading, Loaded
        
        assert_eq!(history[0].from_state, LifecycleState::Unregistered);
        assert_eq!(history[0].to_state, LifecycleState::Registered);
        
        assert_eq!(history[1].from_state, LifecycleState::Registered);
        assert_eq!(history[1].to_state, LifecycleState::Loading);
    }
}
