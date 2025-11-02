//! Advanced Driver Module Loading System
//!
//! Provides comprehensive driver module loading, unloading, and management
//! with support for dynamic module loading, dependency resolution, and
//! rollback capabilities.

use crate::AdvancedDriverId;
use crate::AdvancedDriverError::{self, *};
use crate::Version;
use alloc::collections::{BTreeMap, VecDeque, HashSet};
use alloc::string::String;
use log::{debug, warn, info, error};

/// Module loading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleLoadState {
    Unloaded,       // Module not loaded
    Loading,        // Module is being loaded
    Loaded,         // Module loaded successfully
    Failed,         // Module load failed
    Unloading,      // Module is being unloaded
    Active,         // Module is active and running
}

/// Driver module information
#[derive(Debug, Clone)]
pub struct DriverModule {
    pub module_id: u32,
    pub name: &'static str,
    pub version: Version,
    pub file_path: String,
    pub size_bytes: usize,
    pub checksum: u64,
    pub dependencies: Vec<ModuleDependency>,
    pub symbols: Vec<ModuleSymbol>,
    pub load_priority: u8,
    pub loaded_at: Option<u64>,
    pub state: ModuleLoadState,
    pub error_count: u32,
    pub last_error: Option<AdvancedDriverError>,
}

/// Module dependency specification
#[derive(Debug, Clone)]
pub struct ModuleDependency {
    pub name: String,
    pub min_version: Version,
    pub max_version: Version,
    pub exact_version: Option<Version>,
}

/// Module symbol export/import
#[derive(Debug, Clone)]
pub struct ModuleSymbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub address: usize,
}

/// Symbol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Function,   // Function symbol
    Variable,   // Variable symbol
    Constant,   // Constant symbol
    Object,     // Object/class symbol
}

/// Module loading context
#[derive(Debug, Clone)]
pub struct LoadingContext {
    pub context_id: u64,
    pub start_time: u64,
    pub timeout_ms: u64,
    pub rollback_on_failure: bool,
    pub preload_dependencies: bool,
}

/// Rollback information for module operations
#[derive(Debug, Clone)]
pub struct RollbackInfo {
    pub rollback_id: u64,
    pub modules_to_rollback: Vec<u32>,
    pub operations_performed: Vec<ModuleOperation>,
    pub timestamp: u64,
}

/// Module operation for rollback tracking
#[derive(Debug, Clone)]
pub struct ModuleOperation {
    pub operation_type: OperationType,
    pub module_id: u32,
    pub timestamp: u64,
    pub success: bool,
}

/// Operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Load,
    Unload,
    Initialize,
    Activate,
    Deactivate,
}

/// Module loading statistics
#[derive(Debug, Clone)]
pub struct ModuleLoadStats {
    pub total_modules: u32,
    pub loaded_modules: u32,
    pub failed_modules: u32,
    pub active_modules: u32,
    pub average_load_time_ms: u64,
    pub total_load_time_ms: u64,
    pub rollback_operations: u32,
}

/// Advanced driver module manager
pub struct DriverModuleManager {
    modules: BTreeMap<u32, DriverModule>,
    module_load_order: VecDeque<u32>,
    symbol_table: BTreeMap<String, usize>,
    loading_contexts: BTreeMap<u64, LoadingContext>,
    rollback_stack: VecDeque<RollbackInfo>,
    load_statistics: ModuleLoadStats,
    module_counter: u32,
    symbol_counter: usize,
}

impl DriverModuleManager {
    /// Create a new driver module manager
    pub fn new() -> Self {
        info!("Initializing Driver Module Manager");
        
        let manager = Self {
            modules: BTreeMap::new(),
            module_load_order: VecDeque::new(),
            symbol_table: BTreeMap::new(),
            loading_contexts: BTreeMap::new(),
            rollback_stack: VecDeque::new(),
            load_statistics: ModuleLoadStats {
                total_modules: 0,
                loaded_modules: 0,
                failed_modules: 0,
                active_modules: 0,
                average_load_time_ms: 0,
                total_load_time_ms: 0,
                rollback_operations: 0,
            },
            module_counter: 0,
            symbol_counter: 0,
        };
        
        info!("Driver Module Manager initialized");
        manager
    }

    /// Register a driver module
    pub fn register_module(&mut self, module: DriverModule) -> Result<u32, AdvancedDriverError> {
        debug!("Registering driver module: {} v{}", module.name, module.version);
        
        // Validate module
        if module.name.is_empty() {
            return Err(AdvancedDriverError::DriverNotSupported);
        }
        
        // Check for version conflicts
        if let Some(existing) = self.modules.values()
            .find(|m| m.name == module.name) {
            if existing.version != module.version {
                warn!("Version conflict for module {}: existing v{}, new v{}", 
                      module.name, existing.version, module.version);
                return Err(AdvancedDriverError::VersionConflict);
            }
        }
        
        // Store module
        let module_id = module.module_id;
        self.modules.insert(module_id, module);
        self.update_statistics();
        
        debug!("Driver module registered successfully: {} (ID: {})", module.name, module_id);
        Ok(module_id)
    }

    /// Load a driver module with dependencies
    pub fn load_module(&mut self, module_id: u32, context: LoadingContext) -> Result<(), AdvancedDriverError> {
        debug!("Loading driver module {} with context {}", module_id, context.context_id);
        
        let module = self.modules.get_mut(&module_id)
            .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
        // Check if already loaded
        match module.state {
            ModuleLoadState::Loaded | ModuleLoadState::Active => {
                debug!("Module {} already loaded", module_id);
                return Ok(());
            }
            ModuleLoadState::Loading => {
                warn!("Module {} is already loading", module_id);
                return Err(AdvancedDriverError::LoadFailed);
            }
            ModuleLoadState::Failed => {
                debug!("Retrying failed module {}", module_id);
            }
            _ => {}
        }
        
        // Store loading context
        self.loading_contexts.insert(context.context_id, context.clone());
        
        // Load dependencies first
        if context.preload_dependencies {
            self.load_dependencies(module_id, &context)?;
        }
        
        // Transition to loading state
        module.state = ModuleLoadState::Loading;
        
        // Perform actual module loading
        let load_result = self.perform_module_load(module);
        
        match load_result {
            Ok(()) => {
                module.state = ModuleLoadState::Loaded;
                module.loaded_at = Some(context.start_time);
                self.module_load_order.push_back(module_id);
                self.register_module_symbols(module_id)?;
                
                debug!("Module {} loaded successfully", module_id);
                self.update_statistics();
                Ok(())
            }
            Err(e) => {
                module.state = ModuleLoadState::Failed;
                module.error_count += 1;
                module.last_error = Some(e);
                
                // Rollback if requested
                if context.rollback_on_failure {
                    let _ = self.rollback_module_load(module_id, &context);
                }
                
                error!("Module {} load failed: {:?}", module_id, e);
                self.update_statistics();
                Err(e)
            }
        }
    }

    /// Load module dependencies
    fn load_dependencies(&mut self, module_id: u32, context: &LoadingContext) -> Result<(), AdvancedDriverError> {
        let module = self.modules.get(&module_id)
            .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
        for dependency in &module.dependencies {
            // Find matching module
            let dependent_module_id = self.find_matching_module(&dependency.name, &dependency.min_version, &dependency.max_version)?;
            
            if !self.is_module_loaded(dependent_module_id) {
                self.load_module(dependent_module_id, context.clone())?;
            }
        }
        
        Ok(())
    }

    /// Perform the actual module loading
    fn perform_module_load(&mut self, module: &mut DriverModule) -> Result<(), AdvancedDriverError> {
        debug!("Performing load operation for module: {}", module.name);
        
        // Simulate module loading time based on size
        let load_time_ms = (module.size_bytes / 1024).min(1000) as u64;
        
        for _ in 0..load_time_ms / 10 {
            // Busy wait simulation
        }
        
        // Validate module integrity
        if module.checksum == 0 {
            warn!("Module {} has invalid checksum", module.name);
            return Err(AdvancedDriverError::ValidationFailed);
        }
        
        // Load symbols from module
        for symbol in &module.symbols {
            if symbol.symbol_type == SymbolType::Function {
                let symbol_name = format!("{}_{}", module.name, symbol.name);
                self.symbol_table.insert(symbol_name, symbol.address);
            }
        }
        
        Ok(())
    }

    /// Register module symbols in global symbol table
    fn register_module_symbols(&mut self, module_id: u32) -> Result<(), AdvancedDriverError> {
        let module = self.modules.get(&module_id)
            .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
        for symbol in &module.symbols {
            let full_name = format!("{}::{}", module.name, symbol.name);
            self.symbol_table.insert(full_name, symbol.address);
            self.symbol_counter += 1;
        }
        
        debug!("Registered {} symbols for module {}", module.symbols.len(), module.name);
        Ok(())
    }

    /// Unload a driver module
    pub fn unload_module(&mut self, module_id: u32) -> Result<(), AdvancedDriverError> {
        debug!("Unloading driver module {}", module_id);
        
        let module = self.modules.get_mut(&module_id)
            .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
        // Check if any loaded modules depend on this module
        let dependents = self.find_module_dependents(module_id);
        if !dependents.is_empty() {
            warn!("Cannot unload module {}: {} modules depend on it", module_id, dependents.len());
            return Err(AdvancedDriverError::DependencyUnsatisfied);
        }
        
        // Transition to unloading state
        module.state = ModuleLoadState::Unloading;
        
        // Remove symbols from global symbol table
        self.unregister_module_symbols(module_id)?;
        
        // Remove from load order
        self.module_load_order.retain(|&id| id != module_id);
        
        // Transition to unloaded state
        module.state = ModuleLoadState::Unloaded;
        module.loaded_at = None;
        
        debug!("Module {} unloaded successfully", module_id);
        self.update_statistics();
        Ok(())
    }

    /// Unregister module symbols from global symbol table
    fn unregister_module_symbols(&mut self, module_id: u32) -> Result<(), AdvancedDriverError> {
        let module = self.modules.get(&module_id)
            .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
        for symbol in &module.symbols {
            let full_name = format!("{}::{}", module.name, symbol.name);
            self.symbol_table.remove(&full_name);
        }
        
        debug!("Unregistered symbols for module {}", module.name);
        Ok(())
    }

    /// Activate a loaded module
    pub fn activate_module(&mut self, module_id: u32) -> Result<(), AdvancedDriverError> {
        debug!("Activating driver module {}", module_id);
        
        let module = self.modules.get_mut(&module_id)
            .ok_or(AdvancedDriverError::DeviceNotFound)?;
        
        if module.state != ModuleLoadState::Loaded {
            return Err(AdvancedDriverError::LoadFailed);
        }
        
        // Perform module initialization
        let init_result = self.initialize_module(module);
        
        match init_result {
            Ok(()) => {
                module.state = ModuleLoadState::Active;
                debug!("Module {} activated successfully", module_id);
                self.update_statistics();
                Ok(())
            }
            Err(e) => {
                module.state = ModuleLoadState::Failed;
                error!("Module {} activation failed: {:?}", module_id, e);
                Err(e)
            }
        }
    }

    /// Initialize a module
    fn initialize_module(&self, module: &mut DriverModule) -> Result<(), AdvancedDriverError> {
        debug!("Initializing module: {}", module.name);
        
        // Simulate module initialization
        for _ in 0..module.load_priority * 10 {
            // Busy wait simulation
        }
        
        Ok(())
    }

    /// Rollback module load operation
    pub fn rollback_module_load(&mut self, module_id: u32, context: &LoadingContext) -> Result<(), AdvancedDriverError> {
        debug!("Rolling back load operation for module {}", module_id);
        
        let rollback_id = self.generate_rollback_id();
        
        let rollback_info = RollbackInfo {
            rollback_id,
            modules_to_rollback: vec![module_id],
            operations_performed: vec![
                ModuleOperation {
                    operation_type: OperationType::Load,
                    module_id,
                    timestamp: context.start_time,
                    success: false,
                }
            ],
            timestamp: context.start_time,
        };
        
        self.rollback_stack.push_back(rollback_info);
        self.load_statistics.rollback_operations += 1;
        
        // Actually rollback the module
        if let Some(module) = self.modules.get_mut(&module_id) {
            module.state = ModuleLoadState::Unloaded;
            module.loaded_at = None;
            self.unregister_module_symbols(module_id)?;
        }
        
        debug!("Rollback completed for module {}", module_id);
        Ok(())
    }

    /// Find matching module by name and version constraints
    fn find_matching_module(&self, name: &str, min_version: &Version, max_version: &Version) -> Result<u32, AdvancedDriverError> {
        let mut best_match: Option<u32> = None;
        let mut best_version: Option<Version> = None;
        
        for (&module_id, module) in &self.modules {
            if module.name == name {
                // Check version constraints
                if module.version >= *min_version && module.version <= *max_version {
                    // Choose the newest version that fits constraints
                    if best_match.is_none() || 
                       best_version.map_or(true, |v| module.version > v) {
                        best_match = Some(module_id);
                        best_version = Some(module.version);
                    }
                }
            }
        }
        
        best_match.ok_or(AdvancedDriverError::DriverNotSupported)
    }

    /// Check if module is loaded
    fn is_module_loaded(&self, module_id: u32) -> bool {
        self.modules.get(&module_id)
            .map(|module| matches!(module.state, ModuleLoadState::Loaded | ModuleLoadState::Active))
            .unwrap_or(false)
    }

    /// Find modules that depend on the given module
    fn find_module_dependents(&self, module_id: u32) -> Vec<u32> {
        let mut dependents = Vec::new();
        
        for (&id, module) in &self.modules {
            for dependency in &module.dependencies {
                if let Ok(dep_module_id) = self.find_matching_module(
                    &dependency.name, 
                    &dependency.min_version, 
                    &dependency.max_version
                ) {
                    if dep_module_id == module_id && self.is_module_loaded(id) {
                        dependents.push(id);
                        break;
                    }
                }
            }
        }
        
        dependents
    }

    /// Get module information
    pub fn get_module(&self, module_id: u32) -> Option<&DriverModule> {
        self.modules.get(&module_id)
    }

    /// Get all loaded modules
    pub fn get_loaded_modules(&self) -> Vec<&DriverModule> {
        self.modules.values()
            .filter(|module| matches!(module.state, ModuleLoadState::Loaded | ModuleLoadState::Active))
            .collect()
    }

    /// Get active modules
    pub fn get_active_modules(&self) -> Vec<&DriverModule> {
        self.modules.values()
            .filter(|module| module.state == ModuleLoadState::Active)
            .collect()
    }

    /// Resolve symbol address
    pub fn resolve_symbol(&self, symbol_name: &str) -> Option<usize> {
        self.symbol_table.get(symbol_name).copied()
    }

    /// Get loading statistics
    pub fn get_statistics(&self) -> &ModuleLoadStats {
        &self.load_statistics
    }

    /// Internal: Update statistics
    fn update_statistics(&mut self) {
        let mut loaded = 0;
        let mut failed = 0;
        let mut active = 0;
        
        for module in self.modules.values() {
            self.load_statistics.total_modules += 1;
            
            match module.state {
                ModuleLoadState::Loaded => loaded += 1,
                ModuleLoadState::Active => {
                    loaded += 1;
                    active += 1;
                }
                ModuleLoadState::Failed => failed += 1,
                _ => {}
            }
        }
        
        self.load_statistics.loaded_modules = loaded;
        self.load_statistics.failed_modules = failed;
        self.load_statistics.active_modules = active;
    }

    /// Internal: Generate rollback ID
    fn generate_rollback_id(&mut self) -> u64 {
        static mut ROLLBACK_ID: u64 = 0;
        unsafe {
            ROLLBACK_ID += 1;
            ROLLBACK_ID
        }
    }

    /// Create a loading context
    pub fn create_loading_context(&mut self, timeout_ms: u64, rollback_on_failure: bool) -> u64 {
        let context_id = self.generate_rollback_id(); // Reuse ID generator
        
        let context = LoadingContext {
            context_id,
            start_time: 0, // TODO: Get actual timestamp
            timeout_ms,
            rollback_on_failure,
            preload_dependencies: true,
        };
        
        self.loading_contexts.insert(context_id, context);
        context_id
    }

    /// Get all modules
    pub fn get_all_modules(&self) -> Vec<&DriverModule> {
        self.modules.values().collect()
    }

    /// Find module by name
    pub fn find_module_by_name(&self, name: &str) -> Option<&DriverModule> {
        self.modules.values().find(|module| module.name == name)
    }
}

impl Default for DriverModuleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registration() {
        let mut manager = DriverModuleManager::new();
        
        let module = DriverModule {
            module_id: 1,
            name: "Test Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/test.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 10,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        let result = manager.register_module(module);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_module_loading() {
        let mut manager = DriverModuleManager::new();
        
        // Register a simple module
        let module = DriverModule {
            module_id: 1,
            name: "Simple Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/simple.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: vec![
                ModuleSymbol {
                    name: "init".to_string(),
                    symbol_type: SymbolType::Function,
                    address: 0x1000,
                }
            ],
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(module).unwrap();
        
        // Create loading context
        let context = LoadingContext {
            context_id: 1,
            start_time: 0,
            timeout_ms: 5000,
            rollback_on_failure: false,
            preload_dependencies: false,
        };
        
        // Load the module
        assert!(manager.load_module(1, context).is_ok());
        
        let loaded_module = manager.get_module(1).unwrap();
        assert_eq!(loaded_module.state, ModuleLoadState::Loaded);
    }

    #[test]
    fn test_module_dependencies() {
        let mut manager = DriverModuleManager::new();
        
        // Register dependency module
        let dep_module = DriverModule {
            module_id: 1,
            name: "Base Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/base.ko".to_string(),
            size_bytes: 512,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 1,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        // Register dependent module
        let dependent_module = DriverModule {
            module_id: 2,
            name: "Derived Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/derived.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: vec![
                ModuleDependency {
                    name: "Base Module".to_string(),
                    min_version: Version::new(1, 0, 0),
                    max_version: Version::new(2, 0, 0),
                    exact_version: None,
                }
            ],
            symbols: Vec::new(),
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(dep_module).unwrap();
        manager.register_module(dependent_module).unwrap();
        
        // Create loading context with dependency preloading
        let context = LoadingContext {
            context_id: 1,
            start_time: 0,
            timeout_ms: 10000,
            rollback_on_failure: false,
            preload_dependencies: true,
        };
        
        // Load the dependent module (should auto-load dependency)
        assert!(manager.load_module(2, context).is_ok());
        
        // Both modules should be loaded
        assert!(manager.get_module(1).unwrap().state == ModuleLoadState::Loaded);
        assert!(manager.get_module(2).unwrap().state == ModuleLoadState::Loaded);
    }

    #[test]
    fn test_symbol_resolution() {
        let mut manager = DriverModuleManager::new();
        
        let module = DriverModule {
            module_id: 1,
            name: "TestModule",
            version: Version::new(1, 0, 0),
            file_path: "/modules/test.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: vec![
                ModuleSymbol {
                    name: "init".to_string(),
                    symbol_type: SymbolType::Function,
                    address: 0x1000,
                },
                ModuleSymbol {
                    name: "version".to_string(),
                    symbol_type: SymbolType::Variable,
                    address: 0x2000,
                }
            ],
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(module).unwrap();
        
        let context = LoadingContext {
            context_id: 1,
            start_time: 0,
            timeout_ms: 5000,
            rollback_on_failure: false,
            preload_dependencies: false,
        };
        
        manager.load_module(1, context).unwrap();
        
        // Test symbol resolution
        assert_eq!(manager.resolve_symbol("TestModule::init"), Some(0x1000));
        assert_eq!(manager.resolve_symbol("TestModule::version"), Some(0x2000));
        assert_eq!(manager.resolve_symbol("NonExistent"), None);
    }

    #[test]
    fn test_module_unloading() {
        let mut manager = DriverModuleManager::new();
        
        let module = DriverModule {
            module_id: 1,
            name: "Simple Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/simple.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(module).unwrap();
        
        let context = LoadingContext {
            context_id: 1,
            start_time: 0,
            timeout_ms: 5000,
            rollback_on_failure: false,
            preload_dependencies: false,
        };
        
        // Load and then unload
        manager.load_module(1, context).unwrap();
        assert!(manager.unload_module(1).is_ok());
        
        let unloaded_module = manager.get_module(1).unwrap();
        assert_eq!(unloaded_module.state, ModuleLoadState::Unloaded);
    }

    #[test]
    fn test_module_activation() {
        let mut manager = DriverModuleManager::new();
        
        let module = DriverModule {
            module_id: 1,
            name: "Active Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/active.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(module).unwrap();
        
        let context = LoadingContext {
            context_id: 1,
            start_time: 0,
            timeout_ms: 5000,
            rollback_on_failure: false,
            preload_dependencies: false,
        };
        
        manager.load_module(1, context).unwrap();
        assert!(manager.activate_module(1).is_ok());
        
        let active_module = manager.get_module(1).unwrap();
        assert_eq!(active_module.state, ModuleLoadState::Active);
    }

    #[test]
    fn test_statistics() {
        let mut manager = DriverModuleManager::new();
        
        let module1 = DriverModule {
            module_id: 1,
            name: "Module 1",
            version: Version::new(1, 0, 0),
            file_path: "/modules/module1.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        let module2 = DriverModule {
            module_id: 2,
            name: "Module 2",
            version: Version::new(2, 0, 0),
            file_path: "/modules/module2.ko".to_string(),
            size_bytes: 2048,
            checksum: 54321,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 10,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(module1).unwrap();
        manager.register_module(module2).unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.loaded_modules, 0);
        assert_eq!(stats.failed_modules, 0);
        assert_eq!(stats.active_modules, 0);
    }

    #[test]
    fn test_loading_context_creation() {
        let mut manager = DriverModuleManager::new();
        
        let context_id = manager.create_loading_context(5000, true);
        assert_eq!(context_id, 1);
        
        assert!(manager.loading_contexts.contains_key(&context_id));
    }

    #[test]
    fn test_module_search() {
        let mut manager = DriverModuleManager::new();
        
        let module = DriverModule {
            module_id: 1,
            name: "Findable Module",
            version: Version::new(1, 0, 0),
            file_path: "/modules/findable.ko".to_string(),
            size_bytes: 1024,
            checksum: 12345,
            dependencies: Vec::new(),
            symbols: Vec::new(),
            load_priority: 5,
            loaded_at: None,
            state: ModuleLoadState::Unloaded,
            error_count: 0,
            last_error: None,
        };
        
        manager.register_module(module).unwrap();
        
        assert_eq!(manager.find_module_by_name("Findable Module").unwrap().module_id, 1);
        assert_eq!(manager.find_module_by_name("Non-existent Module"), None);
    }
}
