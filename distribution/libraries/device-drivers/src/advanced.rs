//! Advanced Device Driver Framework
//! 
//! This module provides comprehensive driver lifecycle management, dependencies,
//! power management, hot-plug support, debugging tools, and testing framework.

#![no_std]

use core::fmt;
use alloc::string::String;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use log::{info, warn, error, debug};

/// Version structure for driver versioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<u32>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }

    /// Create a new prerelease version
    pub fn new_prerelease(major: u32, minor: u32, patch: u32, prerelease: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: Some(prerelease),
        }
    }

    /// Format version as string
    pub fn to_string(&self) -> String {
        if let Some(prerelease) = self.prerelease {
            format!("{}.{}.{}-{}", self.major, self.minor, self.patch, prerelease)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Version constraint for driver dependencies
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionConstraint {
    pub min_version: Option<Version>,
    pub max_version: Option<Version>,
    pub exact_version: Option<Version>,
    pub allowed_prereleases: bool,
}

pub mod lifecycle;
pub mod dependencies;
pub mod power_management;
pub mod hot_plug;
pub mod recovery;
pub mod debugging;
pub mod testing;
pub mod versioning;
pub mod resource_cleanup;
pub mod driver_modules;

pub use lifecycle::{DriverLifecycleManager, LifecycleState, LifecycleEvent};
pub use dependencies::{DriverDependency, DependencyManager, DependencyGraph};
pub use power_management::{PowerManager, PowerState, PowerTransition};
pub use hot_plug::{EnhancedHotPlugManager, HotPlugEvent, DeviceNotification, ScanResult, BusScanResult};
pub use recovery::{EnhancedRecoveryManager, RecoveryStrategy, ErrorInfo, EnhancedRecoveryStatistics};
pub use debugging::{DebugManager, TraceLevel, DeviceTrace};
pub use testing::{TestManager, TestSuite, TestResult, ValidationFramework};
pub use versioning::{VersionManager};
pub use resource_cleanup::{ResourceCleanupManager, ResourceInfo, ResourceType, CleanupStatus, ResourceStats};
pub use driver_modules::{DriverModuleManager, DriverModule, ModuleLoadState, LoadingContext, ModuleLoadStats};

/// Advanced driver error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancedDriverError {
    // Basic errors
    DeviceNotFound,
    DriverNotSupported,
    InitializationFailed,
    DeviceBusy,
    PermissionDenied,
    HardwareError,
    
    // Advanced errors
    LifecycleTransitionFailed,
    DependencyResolutionFailed,
    CircularDependency,
    VersionConflict,
    LoadFailed,
    UnloadFailed,
    PowerTransitionFailed,
    HotPlugTimeout,
    RecoveryFailed,
    TestFailed,
    VersionMismatch,
    DependencyUnsatisfied,
    ResourceExhaustion,
    Timeout,
    ValidationFailed,
}

impl fmt::Display for AdvancedDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdvancedDriverError::DeviceNotFound => write!(f, "Device not found"),
            AdvancedDriverError::DriverNotSupported => write!(f, "Driver not supported"),
            AdvancedDriverError::InitializationFailed => write!(f, "Initialization failed"),
            AdvancedDriverError::DeviceBusy => write!(f, "Device is busy"),
            AdvancedDriverError::PermissionDenied => write!(f, "Permission denied"),
            AdvancedDriverError::HardwareError => write!(f, "Hardware error"),
            AdvancedDriverError::LifecycleTransitionFailed => write!(f, "Lifecycle transition failed"),
            AdvancedDriverError::DependencyResolutionFailed => write!(f, "Dependency resolution failed"),
            AdvancedDriverError::CircularDependency => write!(f, "Circular dependency detected"),
            AdvancedDriverError::VersionConflict => write!(f, "Version conflict detected"),
            AdvancedDriverError::LoadFailed => write!(f, "Driver load failed"),
            AdvancedDriverError::UnloadFailed => write!(f, "Driver unload failed"),
            AdvancedDriverError::PowerTransitionFailed => write!(f, "Power transition failed"),
            AdvancedDriverError::HotPlugTimeout => write!(f, "Hot-plug operation timeout"),
            AdvancedDriverError::RecoveryFailed => write!(f, "Recovery operation failed"),
            AdvancedDriverError::TestFailed => write!(f, "Driver test failed"),
            AdvancedDriverError::VersionMismatch => write!(f, "Version mismatch"),
            AdvancedDriverError::DependencyUnsatisfied => write!(f, "Dependency unsatisfied"),
            AdvancedDriverError::ResourceExhaustion => write!(f, "Resource exhaustion"),
            AdvancedDriverError::Timeout => write!(f, "Operation timeout"),
            AdvancedDriverError::ValidationFailed => write!(f, "Validation failed"),
        }
    }
}

/// Advanced driver result type
pub type AdvancedResult<T> = Result<T, AdvancedDriverError>;

/// Driver identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancedDriverId(pub u32);

/// Advanced driver information
#[derive(Debug, Clone)]
pub struct AdvancedDriverInfo {
    pub id: AdvancedDriverId,
    pub name: &'static str,
    pub version: Version,
    pub description: &'static str,
    pub author: &'static str,
    pub license: &'static str,
    pub supported_devices: &'static [crate::DeviceType],
    pub priority: u8,
    pub dependencies: Vec<VersionConstraint>,
    pub capabilities: crate::DeviceCapabilities,
    pub power_management: bool,
    pub hot_plug: bool,
    pub testing_required: bool,
    pub load_timeout_ms: u64,
    pub unload_timeout_ms: u64,
    pub recovery_strategies: Vec<RecoveryStrategy>,
}

/// Global advanced driver manager
pub static ADVANCED_DRIVER_MANAGER: Mutex<Option<AdvancedDriverManager>> = Mutex::new(None);

/// Advanced driver manager
pub struct AdvancedDriverManager {
    pub lifecycle_manager: DriverLifecycleManager,
    pub dependency_manager: DependencyManager,
    pub power_manager: PowerManager,
    pub hot_plug_manager: EnhancedHotPlugManager,
    pub recovery_manager: EnhancedRecoveryManager,
    pub module_manager: DriverModuleManager,
    pub debug_manager: DebugManager,
    pub test_manager: TestManager,
    pub version_manager: VersionManager,
    pub resource_cleanup_manager: ResourceCleanupManager,
    driver_registry: BTreeMap<AdvancedDriverId, AdvancedDriverInfo>,
    loading_queue: Vec<AdvancedDriverId>,
    active_drivers: BTreeMap<AdvancedDriverId, LifecycleState>,
}

impl AdvancedDriverManager {
    /// Create a new advanced driver manager
    pub fn new() -> Self {
        info!("Initializing Advanced Driver Manager");
        
        let manager = Self {
            lifecycle_manager: DriverLifecycleManager::new(),
            dependency_manager: DependencyManager::new(),
            power_manager: PowerManager::new(),
            hot_plug_manager: EnhancedHotPlugManager::new(),
            recovery_manager: EnhancedRecoveryManager::new(),
            module_manager: DriverModuleManager::new(),
            debug_manager: DebugManager::new(),
            test_manager: TestManager::new(),
            version_manager: VersionManager::new(),
            resource_cleanup_manager: ResourceCleanupManager::new(),
            driver_registry: BTreeMap::new(),
            loading_queue: Vec::new(),
            active_drivers: BTreeMap::new(),
        };
        
        info!("Advanced Driver Manager initialized");
        manager
    }

    /// Register an advanced driver
    pub fn register_driver(&mut self, driver_info: AdvancedDriverInfo) -> AdvancedResult<()> {
        info!("Registering advanced driver: {} v{}", driver_info.name, driver_info.version);
        
        // Validate driver information
        if driver_info.name.is_empty() {
            return Err(AdvancedDriverError::DriverNotSupported);
        }
        
        // Check for version conflicts
        if let Some(existing) = self.driver_registry.values()
            .find(|d| d.name == driver_info.name) {
            if existing.version != driver_info.version {
                warn!("Version conflict for driver {}: existing v{}, new v{}", 
                      driver_info.name, existing.version, driver_info.version);
                return Err(AdvancedDriverError::VersionConflict);
            }
        }
        
        // Register with dependency manager
        self.dependency_manager.register_driver(&driver_info)?;
        
        // Register with version manager
        self.version_manager.register_version(&driver_info.name, driver_info.version)?;
        
        // Store driver info
        self.driver_registry.insert(driver_info.id, driver_info);
        
        info!("Advanced driver registered successfully");
        Ok(())
    }

    /// Load a driver with all dependencies
    pub fn load_driver(&mut self, driver_id: AdvancedDriverId) -> AdvancedResult<()> {
        debug!("Loading driver with ID: {:?}", driver_id);
        
        let driver_info = self.driver_registry.get(&driver_id)
            .ok_or(AdvancedDriverError::DriverNotSupported)?;
        
        // Check if driver is already loaded
        if let Some(state) = self.active_drivers.get(&driver_id) {
            if *state == LifecycleState::Active {
                return Ok(());
            }
        }
        
        // Load dependencies first
        self.load_dependencies(driver_id)?;
        
        // Transition to loading state
        self.lifecycle_manager.transition_to_loading(driver_id)?;
        
        // Perform actual driver loading (would integrate with existing driver system)
        self.load_driver_implementation(driver_info)?;
        
        // Mark as active
        self.active_drivers.insert(driver_id, LifecycleState::Active);
        
        // Start power management
        if driver_info.power_management {
            self.power_manager.enable_power_management(driver_id)?;
        }
        
        // Run tests if required
        if driver_info.testing_required {
            self.test_manager.run_driver_tests(driver_id)?;
        }
        
        info!("Driver loaded successfully: {}", driver_info.name);
        Ok(())
    }

    /// Unload a driver and its dependents
    pub fn unload_driver(&mut self, driver_id: AdvancedDriverId) -> AdvancedResult<()> {
        debug!("Unloading driver with ID: {:?}", driver_id);
        
        let driver_info = self.driver_registry.get(&driver_id)
            .ok_or(AdvancedDriverError::DriverNotSupported)?;
        
        // Check if any loaded drivers depend on this driver
        let dependents = self.dependency_manager.get_dependents(driver_id);
        if !dependents.is_empty() {
            let dependent_names: Vec<_> = dependents.iter()
                .filter_map(|id| self.driver_registry.get(id).map(|d| d.name))
                .collect();
            return Err(AdvancedDriverError::DependencyUnsatisfied);
        }
        
        // Disable power management
        if driver_info.power_management {
            self.power_manager.disable_power_management(driver_id)?;
        }
        
        // Run driver tests
        self.test_manager.run_unload_tests(driver_id)?;
        
        // Transition to unloading state
        self.lifecycle_manager.transition_to_unloading(driver_id)?;
        
        // Perform actual driver unloading
        self.unload_driver_implementation(driver_info)?;
        
        // Remove from active drivers
        self.active_drivers.remove(&driver_id);
        
        info!("Driver unloaded successfully: {}", driver_info.name);
        Ok(())
    }

    /// Load all dependencies for a driver
    fn load_dependencies(&mut self, driver_id: AdvancedDriverId) -> AdvancedResult<()> {
        let driver_info = self.driver_registry.get(&driver_id)
            .ok_or(AdvancedDriverError::DriverNotSupported)?;
        
        for dependency in &driver_info.dependencies {
            let dependent_driver_id = self.version_manager.find_compatible_driver(dependency)?;
            
            if !self.active_drivers.contains_key(&dependent_driver_id) {
                self.load_driver(dependent_driver_id)?;
            }
        }
        
        Ok(())
    }

    /// Get all registered drivers
    pub fn get_registered_drivers(&self) -> Vec<&AdvancedDriverInfo> {
        self.driver_registry.values().collect()
    }

    /// Get active drivers
    pub fn get_active_drivers(&self) -> Vec<AdvancedDriverId> {
        self.active_drivers.keys().copied().collect()
    }

    /// Get driver statistics
    pub fn get_statistics(&self) -> AdvancedDriverStatistics {
        AdvancedDriverStatistics {
            total_registered: self.driver_registry.len(),
            active_drivers: self.active_drivers.len(),
            loading_queue_size: self.loading_queue.len(),
            lifecycle_states: self.lifecycle_manager.get_state_counts(),
            dependency_graph_size: self.dependency_manager.get_graph_size(),
            power_managed_devices: self.power_manager.get_power_managed_count(),
            hot_plug_events: self.hot_plug_manager.get_event_count(),
            debug_traces: self.debug_manager.get_trace_count(),
        }
    }

    /// Simulate driver loading (placeholder for actual implementation)
    fn load_driver_implementation(&self, driver_info: &AdvancedDriverInfo) -> AdvancedResult<()> {
        // This would integrate with the existing driver system
        debug!("Loading driver implementation: {}", driver_info.name);
        
        // Simulate some processing time
        for _ in 0..driver_info.load_timeout_ms.min(100) / 10 {
            // Busy wait simulation
        }
        
        Ok(())
    }

    /// Simulate driver unloading (placeholder for actual implementation)
    fn unload_driver_implementation(&self, driver_info: &AdvancedDriverInfo) -> AdvancedResult<()> {
        // This would integrate with the existing driver system
        debug!("Unloading driver implementation: {}", driver_info.name);
        
        // Simulate some processing time
        for _ in 0..driver_info.unload_timeout_ms.min(100) / 10 {
            // Busy wait simulation
        }
        
        Ok(())
    }
}

/// Advanced driver statistics
#[derive(Debug, Clone)]
pub struct AdvancedDriverStatistics {
    pub total_registered: usize,
    pub active_drivers: usize,
    pub loading_queue_size: usize,
    pub lifecycle_states: BTreeMap<LifecycleState, usize>,
    pub dependency_graph_size: usize,
    pub power_managed_devices: usize,
    pub hot_plug_events: usize,
    pub debug_traces: usize,
}

/// Initialize the advanced driver framework
pub fn init_advanced_framework() -> AdvancedResult<()> {
    let mut manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    
    let manager = AdvancedDriverManager::new();
    *manager_guard = Some(manager);
    
    info!("Advanced Driver Framework initialized");
    Ok(())
}

/// Get the advanced driver manager
pub fn get_advanced_manager() -> Option<AdvancedDriverManager> {
    let manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    manager_guard.clone()
}

/// Register an advanced driver
pub fn register_advanced_driver(driver_info: AdvancedDriverInfo) -> AdvancedResult<()> {
    let mut manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
    manager.register_driver(driver_info)
}

/// Load a driver with all dependencies
pub fn load_driver(driver_id: AdvancedDriverId) -> AdvancedResult<()> {
    let mut manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
    manager.load_driver(driver_id)
}

/// Unload a driver
pub fn unload_driver(driver_id: AdvancedDriverId) -> AdvancedResult<()> {
    let mut manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
    manager.unload_driver(driver_id)
}

/// Get driver statistics
pub fn get_advanced_statistics() -> AdvancedDriverStatistics {
    let manager_guard = ADVANCED_DRIVER_MANAGER.lock();
    
    if let Some(manager) = manager_guard.as_ref() {
        manager.get_statistics()
    } else {
        AdvancedDriverStatistics {
            total_registered: 0,
            active_drivers: 0,
            loading_queue_size: 0,
            lifecycle_states: BTreeMap::new(),
            dependency_graph_size: 0,
            power_managed_devices: 0,
            hot_plug_events: 0,
            debug_traces: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_driver_error_variants() {
        let errors = [
            AdvancedDriverError::DeviceNotFound,
            AdvancedDriverError::DriverNotSupported,
            AdvancedDriverError::LifecycleTransitionFailed,
            AdvancedDriverError::DependencyResolutionFailed,
            AdvancedDriverError::VersionConflict,
            AdvancedDriverError::LoadFailed,
            AdvancedDriverError::UnloadFailed,
            AdvancedDriverError::PowerTransitionFailed,
            AdvancedDriverError::HotPlugTimeout,
            AdvancedDriverError::RecoveryFailed,
            AdvancedDriverError::TestFailed,
            AdvancedDriverError::Timeout,
            AdvancedDriverError::ValidationFailed,
        ];
        
        // Test that all errors can be formatted
        for error in &errors {
            let formatted = format!("{}", error);
            assert!(!formatted.is_empty());
        }
    }

    #[test]
    fn test_advanced_driver_info_validation() {
        let valid_driver = AdvancedDriverInfo {
            id: AdvancedDriverId(1),
            name: "Test Driver",
            version: Version::new(1, 0, 0),
            description: "Test driver",
            author: "Test Author",
            license: "MIT",
            supported_devices: &[],
            priority: 10,
            dependencies: Vec::new(),
            capabilities: crate::DeviceCapabilities::NONE,
            power_management: false,
            hot_plug: false,
            testing_required: false,
            load_timeout_ms: 1000,
            unload_timeout_ms: 1000,
            recovery_strategies: Vec::new(),
        };
        
        // Test that valid driver info doesn't cause errors during creation
        assert_eq!(valid_driver.name, "Test Driver");
        assert_eq!(valid_driver.version.major, 1);
    }
}
