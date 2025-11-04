//! MultiOS System Configuration Management Framework
//! 
//! This module provides comprehensive system configuration management including:
//! - Global system settings management
//! - Configuration persistence and loading mechanisms  
//! - Policy management system for system-wide rules
//! - Configuration validation and schema checking
//! - Configuration backup and restore functionality
//! - Configuration change tracking and audit logging
//! - Integration with service manager for configuration propagation

#![no_std]
#![feature(alloc)]
#![feature(hash_map_entry)]

use spin::{Mutex, RwLock};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// Re-export key types and modules
pub mod schema;
pub mod policy;
pub mod persistence;
pub mod validation;
pub mod backup;
pub mod audit;
pub mod propagation;

use schema::ConfigSchema;
use policy::PolicyManager;
use persistence::ConfigStorage;
use validation::ConfigValidator;
use backup::BackupManager;
use audit::AuditLogger;
use propagation::ConfigPropagator;

/// Global configuration manager instance
pub static CONFIG_MANAGER: Mutex<Option<ConfigManager>> = Mutex::new(None);

/// Configuration management result
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Error types for configuration management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConfigError {
    NotFound = 0,
    AlreadyExists,
    InvalidFormat,
    ValidationFailed,
    PersistenceError,
    BackupError,
    PolicyViolation,
    SchemaMismatch,
    AccessDenied,
    VersionConflict,
    CircularDependency,
    Timeout,
    InvalidParameter,
    ResourceExhausted,
    IntegrityCheckFailed,
    AuditLogFull,
    PropagationFailed,
    Unknown,
}

/// System configuration key
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ConfigKey {
    pub namespace: String,
    pub key: String,
    pub path: String, // Full path like "system.network.hostname"
}

/// Configuration value types
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Unsigned(u64),
    Boolean(bool),
    Float(f64),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    None,
}

/// System configuration entry
#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: ConfigKey,
    pub value: ConfigValue,
    pub value_type: ConfigType,
    pub description: Option<String>,
    pub default_value: Option<ConfigValue>,
    pub read_only: bool,
    pub system: bool,
    pub persistent: bool,
    pub last_modified: u64,
    pub version: u64,
    pub checksum: u32,
}

/// Configuration types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    String = 0,
    Integer = 1,
    Unsigned = 2,
    Boolean = 3,
    Float = 4,
    Array = 5,
    Object = 6,
    None = 7,
}

/// Configuration scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
    System = 0,
    Service = 1,
    User = 2,
    Session = 3,
}

/// Configuration change operation
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub operation: ChangeOperation,
    pub key: ConfigKey,
    pub old_value: Option<ConfigValue>,
    pub new_value: Option<ConfigValue>,
    pub timestamp: u64,
    pub user_id: Option<u64>,
    pub source: ChangeSource,
    pub reason: Option<String>,
}

/// Change operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeOperation {
    Create = 0,
    Update = 1,
    Delete = 2,
    Read = 3,
}

/// Change sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeSource {
    System = 0,
    Service = 1,
    User = 2,
    API = 3,
    Policy = 4,
    Backup = 5,
    Migration = 6,
}

/// Configuration statistics
#[derive(Debug, Clone)]
pub struct ConfigStats {
    pub total_entries: usize,
    pub persistent_entries: usize,
    pub read_only_entries: usize,
    pub system_entries: usize,
    pub last_backup: u64,
    pub audit_entries: usize,
    pub policy_violations: usize,
    pub changes_per_minute: f64,
    pub storage_used_bytes: usize,
    pub backup_count: usize,
}

/// Main configuration manager
pub struct ConfigManager {
    config_data: RwLock<HashMap<ConfigKey, ConfigEntry>>,
    schema_manager: ConfigSchema,
    policy_manager: PolicyManager,
    storage: ConfigStorage,
    validator: ConfigValidator,
    backup_manager: BackupManager,
    audit_logger: AuditLogger,
    propagator: ConfigPropagator,
    change_history: RwLock<Vec<ConfigChange>>,
    next_version: AtomicU64,
    stats: ConfigStats,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        ConfigManager {
            config_data: RwLock::new(HashMap::new()),
            schema_manager: ConfigSchema::new(),
            policy_manager: PolicyManager::new(),
            storage: ConfigStorage::new(),
            validator: ConfigValidator::new(),
            backup_manager: BackupManager::new(),
            audit_logger: AuditLogger::new(),
            propagator: ConfigPropagator::new(),
            change_history: RwLock::new(Vec::new()),
            next_version: AtomicU64::new(1),
            stats: ConfigStats {
                total_entries: 0,
                persistent_entries: 0,
                read_only_entries: 0,
                system_entries: 0,
                last_backup: 0,
                audit_entries: 0,
                policy_violations: 0,
                changes_per_minute: 0.0,
                storage_used_bytes: 0,
                backup_count: 0,
            },
        }
    }

    /// Initialize the configuration manager
    pub fn init() -> ConfigResult<()> {
        let mut manager_guard = CONFIG_MANAGER.lock();
        
        if manager_guard.is_some() {
            return Err(ConfigError::AlreadyExists);
        }

        let manager = ConfigManager::new();
        *manager_guard = Some(manager);
        
        info!("Configuration Manager initialized successfully");
        Ok(())
    }

    /// Start the configuration manager
    pub fn start() -> ConfigResult<()> {
        let manager_guard = CONFIG_MANAGER.lock();
        let manager = manager_guard
            .as_ref()
            .ok_or(ConfigError::NotFound)?;

        // Initialize all components
        manager.schema_manager.init()?;
        manager.policy_manager.init()?;
        manager.storage.init()?;
        manager.validator.init()?;
        manager.backup_manager.init()?;
        manager.audit_logger.init()?;
        manager.propagator.init()?;

        // Load persisted configuration
        manager.load_persisted_config()?;

        info!("Configuration Manager started successfully");
        Ok(())
    }

    /// Get a configuration value
    pub fn get_config(key: &ConfigKey) -> ConfigResult<ConfigValue> {
        let manager_guard = CONFIG_MANAGER.lock();
        let manager = manager_guard
            .as_ref()
            .ok_or(ConfigError::NotFound)?;

        let config_data = manager.config_data.read();
        let entry = config_data.get(key)
            .ok_or(ConfigError::NotFound)?;

        // Audit log the read operation
        let change = ConfigChange {
            operation: ChangeOperation::Read,
            key: key.clone(),
            old_value: None,
            new_value: Some(entry.value.clone()),
            timestamp: get_current_time(),
            user_id: None,
            source: ChangeSource::System,
            reason: None,
        };
        manager.audit_logger.log_change(&change)?;

        Ok(entry.value.clone())
    }

    /// Set a configuration value
    pub fn set_config(key: &ConfigKey, value: ConfigValue, user_id: Option<u64>) -> ConfigResult<()> {
        let manager_guard = CONFIG_MANAGER.lock();
        let manager = manager_guard
            .as_ref()
            .ok_or(ConfigError::NotFound)?;

        // Validate the key and value
        manager.validate_key_value(key, &value)?;

        // Check policy constraints
        manager.policy_manager.check_policy(key, &value)?;

        let mut config_data = manager.config_data.write();
        let old_value = config_data.get(key).cloned();

        // Update or create the entry
        let entry = ConfigEntry {
            key: key.clone(),
            value,
            value_type: determine_value_type(&value),
            description: None,
            default_value: None,
            read_only: false,
            system: false,
            persistent: true,
            last_modified: get_current_time(),
            version: manager.next_version.fetch_add(1, Ordering::SeqCst),
            checksum: 0, // Would be calculated
        };

        config_data.insert(key.clone(), entry);

        // Record the change
        let change = ConfigChange {
            operation: if old_value.is_some() { 
                ChangeOperation::Update 
            } else { 
                ChangeOperation::Create 
            },
            key: key.clone(),
            old_value: old_value.map(|e| e.value),
            new_value: Some(value),
            timestamp: get_current_time(),
            user_id,
            source: ChangeSource::User,
            reason: None,
        };

        // Log the change
        manager.audit_logger.log_change(&change)?;
        
        // Add to change history
        let mut history = manager.change_history.write();
        history.push(change);

        // Propagate configuration to services
        manager.propagator.propagate_config(key)?;

        info!("Configuration updated: {} -> {:?}", key.path, value);

        Ok(())
    }

    /// Delete a configuration entry
    pub fn delete_config(key: &ConfigKey, user_id: Option<u64>) -> ConfigResult<()> {
        let manager_guard = CONFIG_MANAGER.lock();
        let manager = manager_guard
            .as_ref()
            .ok_or(ConfigError::NotFound)?;

        let mut config_data = manager.config_data.write();
        let old_value = config_data.remove(key)
            .ok_or(ConfigError::NotFound)?;

        // Check if it's read-only or system
        if old_value.read_only || old_value.system {
            return Err(ConfigError::AccessDenied);
        }

        // Record the change
        let change = ConfigChange {
            operation: ChangeOperation::Delete,
            key: key.clone(),
            old_value: Some(old_value.value),
            new_value: None,
            timestamp: get_current_time(),
            user_id,
            source: ChangeSource::User,
            reason: None,
        };

        manager.audit_logger.log_change(&change)?;

        // Propagate deletion to services
        manager.propagator.propagate_config(key)?;

        info!("Configuration deleted: {}", key.path);

        Ok(())
    }

    /// Create a backup of all configuration
    pub fn create_backup(&self) -> ConfigResult<String> {
        let backup_id = self.backup_manager.create_backup(
            &self.config_data.read(),
            &self.change_history.read()
        )?;

        // Update stats
        self.update_stats_after_backup();

        info!("Configuration backup created: {}", backup_id);
        Ok(backup_id)
    }

    /// Restore configuration from backup
    pub fn restore_backup(&self, backup_id: &str) -> ConfigResult<()> {
        // Validate backup integrity
        self.backup_manager.validate_backup(backup_id)?;

        // Load backup data
        let (config_data, change_history) = self.backup_manager.restore_backup(backup_id)?;

        // Clear current configuration
        let mut current_data = self.config_data.write();
        current_data.clear();
        
        // Restore from backup
        for (key, entry) in config_data {
            current_data.insert(key, entry);
        }

        // Restore change history
        let mut current_history = self.change_history.write();
        current_history.clear();
        current_history.extend(change_history);

        info!("Configuration restored from backup: {}", backup_id);
        Ok(())
    }

    /// Get configuration statistics
    pub fn get_stats(&self) -> ConfigStats {
        self.stats.clone()
    }

    /// Export configuration to a format suitable for persistence
    pub fn export_config(&self) -> ConfigResult<Vec<u8>> {
        let config_data = self.config_data.read();
        self.storage.serialize_config(&*config_data)
    }

    /// Import configuration from persisted data
    pub fn import_config(&self, data: &[u8]) -> ConfigResult<()> {
        let config_data = self.storage.deserialize_config(data)?;
        let mut current_data = self.config_data.write();
        current_data.clear();
        current_data.extend(config_data);
        Ok(())
    }

    /// Validate configuration against schema
    pub fn validate_config(&self) -> ConfigResult<()> {
        let config_data = self.config_data.read();
        self.validator.validate_all_configurations(&*config_data)
    }

    /// Apply policies to current configuration
    pub fn apply_policies(&self) -> ConfigResult<()> {
        let config_data = self.config_data.read();
        self.policy_manager.apply_policies(&*config_data)
    }

    /// Propagate configuration to all relevant services
    pub fn propagate_config(&self) -> ConfigResult<()> {
        let config_data = self.config_data.read();
        self.propagator.propagate_all_configs(&*config_data)
    }

    /// Get change history
    pub fn get_change_history(&self) -> Vec<ConfigChange> {
        self.change_history.read().clone()
    }

    /// Clear old change history entries
    pub fn cleanup_history(&self, max_entries: usize) -> ConfigResult<()> {
        let mut history = self.change_history.write();
        if history.len() > max_entries {
            let to_remove = history.len() - max_entries;
            history.drain(0..to_remove);
        }
        Ok(())
    }

    /// Internal method to validate key and value
    fn validate_key_value(&self, key: &ConfigKey, value: &ConfigValue) -> ConfigResult<()> {
        // Check key format
        if key.namespace.is_empty() || key.key.is_empty() || key.path.is_empty() {
            return Err(ConfigError::InvalidParameter);
        }

        // Validate against schema
        self.schema_manager.validate_value(key, value)?;

        // Check data type compatibility
        if !self.validator.is_value_type_valid(value, &key.namespace) {
            return Err(ConfigError::ValidationFailed);
        }

        Ok(())
    }

    /// Internal method to update statistics after backup
    fn update_stats_after_backup(&self) {
        // This would update internal statistics
        // For now, just a placeholder
    }

    /// Internal method to load persisted configuration
    fn load_persisted_config(&self) -> ConfigResult<()> {
        if let Ok(data) = self.storage.load_config() {
            let config_data = self.storage.deserialize_config(&data)?;
            let mut current_data = self.config_data.write();
            current_data.extend(config_data);
        }
        Ok(())
    }

    /// Internal method to save persisted configuration
    fn save_persisted_config(&self) -> ConfigResult<()> {
        let config_data = self.config_data.read();
        let serialized = self.storage.serialize_config(&*config_data)?;
        self.storage.save_config(&serialized)?;
        Ok(())
    }
}

/// System configuration management functions
pub mod system_config {
    use super::*;

    /// Initialize system configuration with defaults
    pub fn init_system_config() -> ConfigResult<()> {
        let manager_guard = CONFIG_MANAGER.lock();
        let manager = manager_guard
            .as_ref()
            .ok_or(ConfigError::NotFound)?;

        // Set up system configuration keys
        let system_keys = vec![
            ("system.boot.recovery_mode", ConfigValue::Boolean(false)),
            ("system.boot.debug_enabled", ConfigValue::Boolean(true)),
            ("system.network.hostname", ConfigValue::String("multios".to_string())),
            ("system.network.domain", ConfigValue::String("local".to_string())),
            ("system.security.policy_level", ConfigValue::Integer(3)),
            ("system.memory.heap_size", ConfigValue::Unsigned(1024 * 1024)),
            ("system.memory.stack_size", ConfigValue::Unsigned(8192)),
        ];

        for (key_str, default_value) in system_keys {
            let key = ConfigKey {
                namespace: "system".to_string(),
                key: key_str.to_string(),
                path: key_str.to_string(),
            };

            // Only set if not already present
            if manager.get_config(&key).is_err() {
                manager.set_config(&key, default_value, None)?;
            }
        }

        Ok(())
    }

    /// Get system configuration
    pub fn get_system_config(key: &str) -> ConfigResult<ConfigValue> {
        let config_key = ConfigKey {
            namespace: "system".to_string(),
            key: key.to_string(),
            path: format!("system.{}", key),
        };
        ConfigManager::get_config(&config_key)
    }

    /// Set system configuration
    pub fn set_system_config(key: &str, value: ConfigValue) -> ConfigResult<()> {
        let config_key = ConfigKey {
            namespace: "system".to_string(),
            key: key.to_string(),
            path: format!("system.{}", key),
        };
        ConfigManager::set_config(&config_key, value, None)
    }
}

/// Helper function to determine value type
fn determine_value_type(value: &ConfigValue) -> ConfigType {
    match value {
        ConfigValue::String(_) => ConfigType::String,
        ConfigValue::Integer(_) => ConfigType::Integer,
        ConfigValue::Unsigned(_) => ConfigType::Unsigned,
        ConfigValue::Boolean(_) => ConfigType::Boolean,
        ConfigValue::Float(_) => ConfigType::Float,
        ConfigValue::Array(_) => ConfigType::Array,
        ConfigValue::Object(_) => ConfigType::Object,
        ConfigValue::None => ConfigType::None,
    }
}

/// Get current system time
fn get_current_time() -> u64 {
    // This would integrate with the kernel's time subsystem
    // For now, use a placeholder value
    1000000
}

/// Initialize the configuration management system
pub fn init_config_manager() -> ConfigResult<()> {
    ConfigManager::init()?;
    ConfigManager::start()?;
    
    // Initialize system configuration
    system_config::init_system_config()?;
    
    info!("Configuration Management System initialized successfully");
    Ok(())
}

/// Get global configuration manager instance
pub fn get_config_manager() -> Option<&'static Mutex<Option<ConfigManager>>> {
    Some(&CONFIG_MANAGER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert_eq!(manager.stats.total_entries, 0);
    }

    #[test]
    fn test_config_key_creation() {
        let key = ConfigKey {
            namespace: "test".to_string(),
            key: "key1".to_string(),
            path: "test.key1".to_string(),
        };
        assert_eq!(key.namespace, "test");
        assert_eq!(key.key, "key1");
    }

    #[test]
    fn test_config_value_types() {
        let string_val = ConfigValue::String("test".to_string());
        let int_val = ConfigValue::Integer(42);
        let bool_val = ConfigValue::Boolean(true);
        
        assert!(matches!(string_val, ConfigValue::String(_)));
        assert!(matches!(int_val, ConfigValue::Integer(_)));
        assert!(matches!(bool_val, ConfigValue::Boolean(_)));
    }

    #[test]
    fn test_change_operation_variants() {
        assert_eq!(ChangeOperation::Create as u8, 0);
        assert_eq!(ChangeOperation::Update as u8, 1);
        assert_eq!(ChangeOperation::Delete as u8, 2);
    }
}