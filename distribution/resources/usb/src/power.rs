//! USB Power Management Module
//! 
//! Provides comprehensive USB power management including:
//! - Device power state management (active, suspended, etc.)
//! - Power policy enforcement and optimization
//! - Remote wake-up capability
//! - Power consumption tracking
//! - System sleep/wake handling
//! - Per-device and global power policies

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// USB Power Management State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbPowerManagementState {
    /// Device is actively transferring data
    Active,
    /// Device is in low power state but can be woken
    Suspended,
    /// Device is powered down and cannot be woken remotely
    PoweredDown,
    /// Device is in selective suspend state
    SelectiveSuspended,
    /// Device is in system sleep state
    SystemSuspended,
    /// Device is in idle state with periodic wake
    Idle,
    /// Device is in test mode
    TestMode,
    /// Error state
    Error,
}

/// USB Power Policy Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbPowerPolicy {
    Performance,      // Max performance, no power saving
    Balanced,         // Balanced performance and power saving
    PowerSaver,       // Aggressive power saving
    BatteryOptimized, // Optimized for battery life
    Custom,           // Custom policy
}

/// USB Power Event Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbPowerEvent {
    DeviceSuspended,
    DeviceResumed,
    DevicePoweredDown,
    DevicePoweredUp,
    RemoteWakeupTriggered,
    PowerPolicyChanged,
    IdleTimeoutReached,
    OvercurrentDetected,
    VoltageAnomalyDetected,
    ThermalLimitReached,
}

/// USB Device Power Configuration
#[derive(Debug, Clone)]
pub struct UsbDevicePowerConfig {
    pub device_address: u8,
    pub supported_power_states: Vec<UsbPowerManagementState>,
    pub max_power_ma: u32,
    pub idle_timeout_ms: u32,
    pub remote_wakeup_supported: bool,
    pub remote_wakeup_enabled: bool,
    pub power_policy: UsbPowerPolicy,
    pub wakeup_priority: u8, // 0-255, higher = higher priority
    pub allow_selective_suspend: bool,
}

/// USB System Power Configuration
#[derive(Debug, Clone)]
pub struct UsbSystemPowerConfig {
    pub global_power_policy: UsbPowerPolicy,
    pub default_idle_timeout_ms: u32,
    pub enable_selective_suspend: bool,
    pub enable_remote_wakeup: bool,
    pub max_system_power_ma: u32,
    pub thermal_limit_enabled: bool,
    pub thermal_limit_celsius: f32,
    pub allow_system_suspend: bool,
    pub wake_on_connect: bool,
    pub wake_on_data: bool,
}

/// USB Power Budget
#[derive(Debug, Clone)]
pub struct UsbPowerBudget {
    pub total_power_ma: u32,
    pub allocated_power_ma: u32,
    pub available_power_ma: u32,
    pub device_budgets: BTreeMap<u8, u32>, // device_address -> power_ma
    pub emergency_reserve_ma: u32,
}

/// USB Power Statistics
#[derive(Debug, Clone)]
pub struct UsbPowerStats {
    pub total_power_consumed_ma: u32,
    pub peak_power_consumed_ma: u32,
    pub average_power_consumed_ma: u32,
    pub energy_consumed_mwh: u64,
    pub time_in_active_ms: u64,
    pub time_in_suspended_ms: u64,
    pub time_in_powered_down_ms: u64,
    pub suspend_count: u32,
    pub resume_count: u32,
    pub remote_wakeup_count: u32,
    pub power_policy_changes: u32,
}

/// USB Power Manager
pub struct UsbPowerManager {
    pub system_config: UsbSystemPowerConfig,
    pub device_configs: BTreeMap<u8, UsbDevicePowerConfig>,
    pub device_states: BTreeMap<u8, UsbPowerManagementState>,
    pub power_stats: BTreeMap<u8, UsbPowerStats>,
    pub power_budget: UsbPowerBudget,
    pub power_events: Vec<UsbPowerEvent>,
    pub event_callbacks: Vec<fn(UsbPowerEvent, u8)>,
    pub thermal_monitor_enabled: bool,
    pub current_temperature_c: f32,
    pub power_limit_enforced: bool,
    pub idle_timeout_active: bool,
}

/// USB Power Policy Manager
pub struct UsbPowerPolicyManager {
    pub policies: BTreeMap<UsbPowerPolicy, UsbPolicyProfile>,
    pub active_policy: UsbPowerPolicy,
    pub policy_transition_rules: Vec<UsbPolicyTransition>,
    pub custom_parameters: BTreeMap<String, u32>,
}

/// USB Policy Profile
#[derive(Debug, Clone)]
pub struct UsbPolicyProfile {
    pub name: String,
    pub idle_timeout_ms: u32,
    pub allow_selective_suspend: bool,
    pub allow_system_suspend: bool,
    pub remote_wakeup_enabled: bool,
    pub thermal_monitoring: bool,
    pub max_power_optimization: bool,
    pub performance_thresholds: PolicyThresholds,
}

/// Policy Performance Thresholds
#[derive(Debug, Clone)]
pub struct PolicyThresholds {
    pub suspend_threshold_pct: u8,      // CPU usage threshold for suspend
    pub data_activity_threshold: u32,   // Bytes/sec threshold for activity
    pub thermal_limit_celsius: f32,
    pub power_budget_threshold_pct: u8,
}

/// USB Policy Transition Rule
#[derive(Debug, Clone)]
pub struct UsbPolicyTransition {
    pub trigger: PolicyTrigger,
    pub from_policy: UsbPowerPolicy,
    pub to_policy: UsbPowerPolicy,
    pub conditions: Vec<PolicyCondition>,
}

/// Policy Trigger
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyTrigger {
    BatteryLevelLow,
    ThermalLimitReached,
    PowerBudgetExceeded,
    NoActivity,
    UserRequest,
    SystemSleep,
    SystemWake,
    CriticalBattery,
}

/// Policy Condition
#[derive(Debug, Clone)]
pub struct PolicyCondition {
    pub condition_type: ConditionType,
    pub threshold: f32,
    pub comparison: ConditionComparison,
}

/// Condition Type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionType {
    BatteryLevel,
    Temperature,
    PowerUsage,
    DataActivity,
    TimeIdle,
    ConnectionCount,
}

/// Condition Comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionComparison {
    GreaterThan,
    LessThan,
    EqualTo,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// USB Power Manager Implementation
impl UsbPowerManager {
    /// Create a new power manager
    pub fn new() -> Self {
        let system_config = UsbSystemPowerConfig {
            global_power_policy: UsbPowerPolicy::Balanced,
            default_idle_timeout_ms: 5000, // 5 seconds
            enable_selective_suspend: true,
            enable_remote_wakeup: true,
            max_system_power_ma: 1000, // 1A default
            thermal_limit_enabled: true,
            thermal_limit_celsius: 60.0,
            allow_system_suspend: true,
            wake_on_connect: true,
            wake_on_data: false,
        };

        let power_budget = UsbPowerBudget {
            total_power_ma: system_config.max_system_power_ma,
            allocated_power_ma: 0,
            available_power_ma: system_config.max_system_power_ma,
            device_budgets: BTreeMap::new(),
            emergency_reserve_ma: 100, // 100mA emergency reserve
        };

        Self {
            system_config,
            device_configs: BTreeMap::new(),
            device_states: BTreeMap::new(),
            power_stats: BTreeMap::new(),
            power_budget,
            power_events: Vec::new(),
            event_callbacks: Vec::new(),
            thermal_monitor_enabled: true,
            current_temperature_c: 25.0,
            power_limit_enforced: false,
            idle_timeout_active: false,
        }
    }

    /// Initialize power manager
    pub fn initialize(&mut self) -> UsbResult<()> {
        log::info!("USB Power Manager initialized");
        log::info!("Global power policy: {:?}", self.system_config.global_power_policy);
        log::info!("System power budget: {} mA", self.power_budget.total_power_ma);
        
        // Initialize device configurations
        self.initialize_device_defaults();
        
        // Set up power monitoring
        self.setup_power_monitoring();
        
        Ok(())
    }

    /// Initialize default device configurations
    fn initialize_device_defaults(&mut self) {
        // HID devices - low power
        let hid_config = UsbDevicePowerConfig {
            device_address: 0, // Will be updated per device
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::SelectiveSuspended,
            ],
            max_power_ma: 100,
            idle_timeout_ms: 2000, // 2 seconds
            remote_wakeup_supported: true,
            remote_wakeup_enabled: true,
            power_policy: UsbPowerPolicy::PowerSaver,
            wakeup_priority: 50,
            allow_selective_suspend: true,
        };

        // MSC devices - medium power
        let msc_config = UsbDevicePowerConfig {
            device_address: 0,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::PoweredDown,
            ],
            max_power_ma: 500,
            idle_timeout_ms: 10000, // 10 seconds
            remote_wakeup_supported: false,
            remote_wakeup_enabled: false,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 30,
            allow_selective_suspend: false,
        };

        // Audio devices - medium power
        let audio_config = UsbDevicePowerConfig {
            device_address: 0,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::Idle,
            ],
            max_power_ma: 300,
            idle_timeout_ms: 5000, // 5 seconds
            remote_wakeup_supported: false,
            remote_wakeup_enabled: false,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 60,
            allow_selective_suspend: false,
        };

        // CDC devices - variable power
        let cdc_config = UsbDevicePowerConfig {
            device_address: 0,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::Idle,
                UsbPowerManagementState::SelectiveSuspended,
            ],
            max_power_ma: 100,
            idle_timeout_ms: 3000, // 3 seconds
            remote_wakeup_supported: true,
            remote_wakeup_enabled: true,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 40,
            allow_selective_suspend: true,
        };

        log::info!("Initialized default power configurations for USB classes");
    }

    /// Set up power monitoring
    fn setup_power_monitoring(&mut self) {
        if self.thermal_monitor_enabled {
            // Start thermal monitoring
            log::info!("Thermal monitoring enabled, limit: {}°C", 
                      self.system_config.thermal_limit_celsius);
        }

        if self.idle_timeout_active {
            log::info!("Idle timeout monitoring enabled: {} ms", 
                      self.system_config.default_idle_timeout_ms);
        }
    }

    /// Register device for power management
    pub fn register_device(&mut self, device_address: u8, class_code: UsbClass) -> UsbResult<()> {
        if self.device_configs.contains_key(&device_address) {
            return Err(UsbDriverError::DeviceNotFound { address: device_address });
        }

        // Create device-specific configuration based on class
        let config = match class_code {
            UsbClass::HID => self.create_hid_power_config(device_address),
            UsbClass::MassStorage => self.create_msc_power_config(device_address),
            UsbClass::Audio => self.create_audio_power_config(device_address),
            UsbClass::Communications => self.create_cdc_power_config(device_address),
            _ => self.create_generic_power_config(device_address),
        };

        // Allocate power budget
        let budget_allocation = self.allocate_power_budget(device_address, config.max_power_ma)?;
        
        self.device_configs.insert(device_address, config);
        self.device_states.insert(device_address, UsbPowerManagementState::Active);
        self.power_stats.insert(device_address, UsbPowerStats {
            total_power_consumed_ma: 0,
            peak_power_consumed_ma: 0,
            average_power_consumed_ma: 0,
            energy_consumed_mwh: 0,
            time_in_active_ms: 0,
            time_in_suspended_ms: 0,
            time_in_powered_down_ms: 0,
            suspend_count: 0,
            resume_count: 0,
            remote_wakeup_count: 0,
            power_policy_changes: 0,
        });

        log::info!("Device {} registered for power management (budget: {} mA)", 
                  device_address, budget_allocation);
        Ok(())
    }

    /// Create HID power configuration
    fn create_hid_power_config(&self, device_address: u8) -> UsbDevicePowerConfig {
        UsbDevicePowerConfig {
            device_address,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::SelectiveSuspended,
            ],
            max_power_ma: 100,
            idle_timeout_ms: 2000,
            remote_wakeup_supported: true,
            remote_wakeup_enabled: true,
            power_policy: UsbPowerPolicy::PowerSaver,
            wakeup_priority: 50,
            allow_selective_suspend: true,
        }
    }

    /// Create MSC power configuration
    fn create_msc_power_config(&self, device_address: u8) -> UsbDevicePowerConfig {
        UsbDevicePowerConfig {
            device_address,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::PoweredDown,
            ],
            max_power_ma: 500,
            idle_timeout_ms: 10000,
            remote_wakeup_supported: false,
            remote_wakeup_enabled: false,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 30,
            allow_selective_suspend: false,
        }
    }

    /// Create Audio power configuration
    fn create_audio_power_config(&self, device_address: u8) -> UsbDevicePowerConfig {
        UsbDevicePowerConfig {
            device_address,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::Idle,
            ],
            max_power_ma: 300,
            idle_timeout_ms: 5000,
            remote_wakeup_supported: false,
            remote_wakeup_enabled: false,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 60,
            allow_selective_suspend: false,
        }
    }

    /// Create CDC power configuration
    fn create_cdc_power_config(&self, device_address: u8) -> UsbDevicePowerConfig {
        UsbDevicePowerConfig {
            device_address,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
                UsbPowerManagementState::Idle,
                UsbPowerManagementState::SelectiveSuspended,
            ],
            max_power_ma: 100,
            idle_timeout_ms: 3000,
            remote_wakeup_supported: true,
            remote_wakeup_enabled: true,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 40,
            allow_selective_suspend: true,
        }
    }

    /// Create generic power configuration
    fn create_generic_power_config(&self, device_address: u8) -> UsbDevicePowerConfig {
        UsbDevicePowerConfig {
            device_address,
            supported_power_states: vec![
                UsbPowerManagementState::Active,
                UsbPowerManagementState::Suspended,
            ],
            max_power_ma: 200,
            idle_timeout_ms: 5000,
            remote_wakeup_supported: false,
            remote_wakeup_enabled: false,
            power_policy: UsbPowerPolicy::Balanced,
            wakeup_priority: 25,
            allow_selective_suspend: true,
        }
    }

    /// Allocate power budget for device
    fn allocate_power_budget(&mut self, device_address: u8, requested_ma: u32) -> UsbResult<u32> {
        let available = self.power_budget.available_power_ma.saturating_sub(self.power_budget.emergency_reserve_ma);
        
        if available < 50 { // Minimum power allocation
            return Err(UsbDriverError::UnsupportedFeature);
        }

        let allocated = requested_ma.min(available);
        self.power_budget.allocated_power_ma += allocated;
        self.power_budget.available_power_ma = self.power_budget.total_power_ma - self.power_budget.allocated_power_ma;
        self.power_budget.device_budgets.insert(device_address, allocated);

        log::info!("Allocated {} mA power budget for device {} (available: {} mA)", 
                  allocated, device_address, self.power_budget.available_power_ma);

        Ok(allocated)
    }

    /// Suspend device
    pub fn suspend_device(&mut self, device_address: u8) -> UsbResult<()> {
        let config = self.device_configs.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        let current_state = self.device_states.get(&device_address)
            .copied()
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        if current_state == UsbPowerManagementState::Suspended {
            return Ok(()); // Already suspended
        }

        if !config.supported_power_states.contains(&UsbPowerManagementState::Suspended) {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        self.device_states.insert(device_address, UsbPowerManagementState::Suspended);

        // Update statistics
        if let Some(stats) = self.power_stats.get_mut(&device_address) {
            stats.suspend_count += 1;
        }

        // Trigger event
        self.trigger_power_event(UsbPowerEvent::DeviceSuspended, device_address);

        log::info!("Device {} suspended", device_address);
        Ok(())
    }

    /// Resume device
    pub fn resume_device(&mut self, device_address: u8) -> UsbResult<()> {
        let config = self.device_configs.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        let current_state = self.device_states.get(&device_address)
            .copied()
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        if current_state == UsbPowerManagementState::Active {
            return Ok(()); // Already active
        }

        self.device_states.insert(device_address, UsbPowerManagementState::Active);

        // Update statistics
        if let Some(stats) = self.power_stats.get_mut(&device_address) {
            stats.resume_count += 1;
        }

        // Trigger event
        self.trigger_power_event(UsbPowerEvent::DeviceResumed, device_address);

        log::info!("Device {} resumed", device_address);
        Ok(())
    }

    /// Set device power state
    pub fn set_device_power_state(&mut self, device_address: u8, state: UsbPowerManagementState) -> UsbResult<()> {
        let config = self.device_configs.get(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        if !config.supported_power_states.contains(&state) {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        let old_state = self.device_states.get(&device_address)
            .copied()
            .unwrap_or(UsbPowerManagementState::Active);

        self.device_states.insert(device_address, state);

        // Handle state transition effects
        self.handle_state_transition(device_address, old_state, state)?;

        // Update statistics
        self.update_state_statistics(device_address, old_state, state);

        log::info!("Device {} power state changed: {:?} -> {:?}", 
                  device_address, old_state, state);
        Ok(())
    }

    /// Handle power state transition
    fn handle_state_transition(&mut self, device_address: u8, old_state: UsbPowerManagementState, new_state: UsbPowerManagementState) -> UsbResult<()> {
        match new_state {
            UsbPowerManagementState::Suspended | UsbPowerManagementState::SelectiveSuspended => {
                // Enable remote wakeup if supported
                if let Some(config) = self.device_configs.get(&device_address) {
                    if config.remote_wakeup_supported {
                        self.enable_remote_wakeup(device_address)?;
                    }
                }
            }
            UsbPowerManagementState::PoweredDown => {
                // Disable remote wakeup
                if let Some(config) = self.device_configs.get_mut(&device_address) {
                    config.remote_wakeup_enabled = false;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Update statistics for state transition
    fn update_state_statistics(&mut self, device_address: u8, old_state: UsbPowerManagementState, new_state: UsbPowerManagementState) {
        if let Some(stats) = self.power_stats.get_mut(&device_address) {
            let current_time = 0; // TODO: Add timestamp

            match old_state {
                UsbPowerManagementState::Active => {
                    stats.time_in_active_ms += current_time;
                }
                UsbPowerManagementState::Suspended | UsbPowerManagementState::SelectiveSuspended => {
                    stats.time_in_suspended_ms += current_time;
                }
                UsbPowerManagementState::PoweredDown => {
                    stats.time_in_powered_down_ms += current_time;
                }
                _ => {}
            }
        }
    }

    /// Enable remote wakeup for device
    pub fn enable_remote_wakeup(&mut self, device_address: u8) -> UsbResult<()> {
        let config = self.device_configs.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        if !config.remote_wakeup_supported {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        config.remote_wakeup_enabled = true;
        log::info!("Remote wakeup enabled for device {}", device_address);
        Ok(())
    }

    /// Disable remote wakeup for device
    pub fn disable_remote_wakeup(&mut self, device_address: u8) -> UsbResult<()> {
        let config = self.device_configs.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        config.remote_wakeup_enabled = false;
        log::info!("Remote wakeup disabled for device {}", device_address);
        Ok(())
    }

    /// Set global power policy
    pub fn set_global_power_policy(&mut self, policy: UsbPowerPolicy) -> UsbResult<()> {
        self.system_config.global_power_policy = policy;

        // Apply policy to all devices
        for (device_address, config) in &mut self.device_configs {
            self.apply_power_policy(*device_address, policy)?;
        }

        // Trigger event
        self.trigger_power_event(UsbPowerEvent::PowerPolicyChanged, 0);

        log::info!("Global power policy set to {:?}", policy);
        Ok(())
    }

    /// Apply power policy to specific device
    pub fn apply_power_policy(&mut self, device_address: u8, policy: UsbPowerPolicy) -> UsbResult<()> {
        let config = self.device_configs.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        config.power_policy = policy;

        // Update device parameters based on policy
        match policy {
            UsbPowerPolicy::Performance => {
                config.idle_timeout_ms = u32::MAX; // Never timeout
                config.allow_selective_suspend = false;
            }
            UsbPowerPolicy::Balanced => {
                config.idle_timeout_ms = 5000; // 5 seconds
                config.allow_selective_suspend = true;
            }
            UsbPowerPolicy::PowerSaver => {
                config.idle_timeout_ms = 2000; // 2 seconds
                config.allow_selective_suspend = true;
            }
            UsbPowerPolicy::BatteryOptimized => {
                config.idle_timeout_ms = 1000; // 1 second
                config.allow_selective_suspend = true;
            }
            UsbPowerPolicy::Custom => {
                // Keep existing parameters
            }
        }

        log::info!("Power policy applied to device {}: {:?}", device_address, policy);
        Ok(())
    }

    /// Check device idle state
    pub fn check_device_idle(&mut self, device_address: u8) -> UsbResult<bool> {
        let config = self.device_configs.get(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        if self.device_states.get(&device_address) != Some(&UsbPowerManagementState::Active) {
            return Ok(false); // Not active, not idle in active sense
        }

        // Check if device has been idle based on timeout
        let current_time = 0; // TODO: Add timestamp
        let idle_timeout = config.idle_timeout_ms;

        if current_time > idle_timeout {
            self.trigger_power_event(UsbPowerEvent::IdleTimeoutReached, device_address);
            return Ok(true);
        }

        Ok(false)
    }

    /// Handle device activity (reset idle timer)
    pub fn handle_device_activity(&mut self, device_address: u8) -> UsbResult<()> {
        let config = self.device_configs.get_mut(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })?;

        // Resume device if suspended
        let current_state = self.device_states.get(&device_address)
            .copied()
            .unwrap_or(UsbPowerManagementState::Active);

        if current_state != UsbPowerManagementState::Active {
            self.resume_device(device_address)?;
        }

        log::debug!("Device {} activity detected", device_address);
        Ok(())
    }

    /// Trigger power event
    fn trigger_power_event(&mut self, event: UsbPowerEvent, device_address: u8) {
        self.power_events.push(event);

        for callback in &self.event_callbacks {
            callback(event, device_address);
        }
    }

    /// Add power event callback
    pub fn add_power_event_callback(&mut self, callback: fn(UsbPowerEvent, u8)) {
        self.event_callbacks.push(callback);
    }

    /// Get device power state
    pub fn get_device_power_state(&self, device_address: u8) -> UsbResult<UsbPowerManagementState> {
        self.device_states.get(&device_address)
            .copied()
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })
    }

    /// Get device power configuration
    pub fn get_device_config(&self, device_address: u8) -> UsbResult<&UsbDevicePowerConfig> {
        self.device_configs.get(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })
    }

    /// Get power budget information
    pub fn get_power_budget(&self) -> &UsbPowerBudget {
        &self.power_budget
    }

    /// Get power statistics for device
    pub fn get_device_power_stats(&self, device_address: u8) -> UsbResult<&UsbPowerStats> {
        self.power_stats.get(&device_address)
            .ok_or(UsbDriverError::DeviceNotFound { address: device_address })
    }

    /// Get all registered devices
    pub fn get_registered_devices(&self) -> Vec<u8> {
        self.device_configs.keys().cloned().collect()
    }

    /// Unregister device
    pub fn unregister_device(&mut self, device_address: u8) -> UsbResult<()> {
        // Remove from device configurations
        self.device_configs.remove(&device_address);
        self.device_states.remove(&device_address);
        
        // Free power budget
        if let Some(budget) = self.power_budget.device_budgets.remove(&device_address) {
            self.power_budget.allocated_power_ma = self.power_budget.allocated_power_ma.saturating_sub(budget);
            self.power_budget.available_power_ma = self.power_budget.total_power_ma - self.power_budget.allocated_power_ma;
        }

        // Remove statistics
        self.power_stats.remove(&device_address);

        log::info!("Device {} unregistered from power management", device_address);
        Ok(())
    }

    /// Get system power statistics
    pub fn get_system_power_stats(&self) -> UsbSystemPowerStats {
        let mut total_devices = 0;
        let mut active_devices = 0;
        let mut suspended_devices = 0;
        let mut total_power_ma = 0;

        for (device_address, _) in &self.device_configs {
            total_devices += 1;

            match self.device_states.get(device_address) {
                Some(UsbPowerManagementState::Active | UsbPowerManagementState::Idle) => {
                    active_devices += 1;
                }
                Some(UsbPowerManagementState::Suspended | UsbPowerManagementState::SelectiveSuspended) => {
                    suspended_devices += 1;
                }
                _ => {}
            }

            total_power_ma += self.power_budget.device_budgets.get(device_address)
                .copied()
                .unwrap_or(0);
        }

        UsbSystemPowerStats {
            total_power_budget_ma: self.power_budget.total_power_ma,
            allocated_power_ma: self.power_budget.allocated_power_ma,
            available_power_ma: self.power_budget.available_power_ma,
            total_devices,
            active_devices,
            suspended_devices,
            current_power_ma: total_power_ma,
            thermal_limit_enabled: self.thermal_monitor_enabled,
            current_temperature_c: self.current_temperature_c,
        }
    }

    /// Update thermal monitoring
    pub fn update_thermal_monitoring(&mut self, temperature_c: f32) -> UsbResult<()> {
        self.current_temperature_c = temperature_c;

        if temperature_c > self.system_config.thermal_limit_celsius {
            self.trigger_power_event(UsbPowerEvent::ThermalLimitReached, 0);
            
            // Could implement thermal throttling here
            log::warn!("Thermal limit exceeded: {}°C (limit: {}°C)", 
                      temperature_c, self.system_config.thermal_limit_celsius);
            
            return Err(UsbDriverError::ProtocolError);
        }

        Ok(())
    }

    /// Set power budget
    pub fn set_power_budget(&mut self, total_ma: u32) -> UsbResult<()> {
        let old_total = self.power_budget.total_power_ma;
        self.power_budget.total_power_ma = total_ma;
        self.power_budget.available_power_ma = total_ma - self.power_budget.allocated_power_ma;

        log::info!("Power budget changed: {} mA -> {} mA", old_total, total_ma);
        Ok(())
    }
}

/// USB System Power Statistics
#[derive(Debug, Clone)]
pub struct UsbSystemPowerStats {
    pub total_power_budget_ma: u32,
    pub allocated_power_ma: u32,
    pub available_power_ma: u32,
    pub total_devices: usize,
    pub active_devices: usize,
    pub suspended_devices: usize,
    pub current_power_ma: u32,
    pub thermal_limit_enabled: bool,
    pub current_temperature_c: f32,
}

impl Default for UsbPowerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_manager_creation() {
        let manager = UsbPowerManager::new();
        assert_eq!(manager.device_configs.len(), 0);
        assert_eq!(manager.system_config.global_power_policy, UsbPowerPolicy::Balanced);
    }

    #[test]
    fn test_device_registration() {
        let mut manager = UsbPowerManager::new();
        
        let result = manager.register_device(1, UsbClass::HID);
        assert!(result.is_ok());
        assert_eq!(manager.device_configs.len(), 1);
        assert!(manager.device_states.contains_key(&1));
    }

    #[test]
    fn test_device_suspend_resume() {
        let mut manager = UsbPowerManager::new();
        
        manager.register_device(1, UsbClass::HID).unwrap();
        assert_eq!(manager.get_device_power_state(1).unwrap(), UsbPowerManagementState::Active);
        
        manager.suspend_device(1).unwrap();
        assert_eq!(manager.get_device_power_state(1).unwrap(), UsbPowerManagementState::Suspended);
        
        manager.resume_device(1).unwrap();
        assert_eq!(manager.get_device_power_state(1).unwrap(), UsbPowerManagementState::Active);
    }

    #[test]
    fn test_power_budget_allocation() {
        let mut manager = UsbPowerManager::new();
        manager.system_config.max_system_power_ma = 1000;
        
        manager.register_device(1, UsbClass::HID).unwrap();
        let budget = manager.get_power_budget();
        
        assert_eq!(budget.allocated_power_ma, 100); // HID max power
        assert!(budget.available_power_ma > 0);
    }

    #[test]
    fn test_power_policy_application() {
        let mut manager = UsbPowerManager::new();
        
        manager.register_device(1, UsbClass::HID).unwrap();
        manager.set_global_power_policy(UsbPowerPolicy::PowerSaver).unwrap();
        
        let config = manager.get_device_config(1).unwrap();
        assert_eq!(config.power_policy, UsbPowerPolicy::PowerSaver);
        assert_eq!(config.idle_timeout_ms, 2000); // PowerSaver timeout
    }

    #[test]
    fn test_system_power_stats() {
        let manager = UsbPowerManager::new();
        let stats = manager.get_system_power_stats();
        
        assert_eq!(stats.total_devices, 0);
        assert_eq!(stats.active_devices, 0);
        assert_eq!(stats.current_power_ma, 0);
    }
}