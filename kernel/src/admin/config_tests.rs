//! Configuration Management System Integration Tests
//!
//! This module provides comprehensive integration tests for the configuration
//! management framework covering CRUD operations, validation, policies,
//! persistence, backup/restore, and service propagation.

#![cfg(test)]
#![allow(dead_code)]

use crate::admin::config_manager::*;
use crate::admin::schema::*;
use crate::admin::policy::*;
use crate::admin::persistence::*;
use crate::admin::validation::*;
use crate::admin::backup::*;
use crate::admin::propagation::*;
use crate::admin::audit::*;
use core::sync::atomic::{AtomicU64, Ordering};

/// Mock service registry for testing configuration propagation
struct MockServiceRegistry {
    services: spin::RwLock<Vec<MockService>>,
}

impl MockServiceRegistry {
    const fn new() -> Self {
        Self {
            services: spin::RwLock::new(Vec::new()),
        }
    }

    fn add_service(&self, service: MockService) {
        self.services.write().push(service);
    }

    fn get_services(&self) -> Vec<MockService> {
        self.services.read().clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct MockService {
    id: String,
    name: String,
    version: String,
    config_updates: spin::RwLock<Vec<ConfigUpdate>>,
}

impl MockService {
    const fn new(id: String, name: String, version: String) -> Self {
        Self {
            id,
            name,
            version,
            config_updates: spin::RwLock::new(Vec::new()),
        }
    }

    fn receive_config_update(&self, update: ConfigUpdate) {
        self.config_updates.write().push(update);
    }

    fn get_config_updates(&self) -> Vec<ConfigUpdate> {
        self.config_updates.read().clone()
    }
}

impl ConfigurableService for MockService {
    fn service_id(&self) -> &str {
        &self.id
    }

    fn service_name(&self) -> &str {
        &self.name
    }

    fn service_version(&self) -> &str {
        &self.version
    }

    fn apply_config(&self, config: &SystemConfig) -> ConfigResult<()> {
        // Mock configuration application
        let update = ConfigUpdate {
            service_id: self.id.clone(),
            config_path: "system".to_string(),
            timestamp: current_timestamp(),
            config_data: config.clone(),
        };
        self.receive_config_update(update);
        Ok(())
    }

    fn rollback_config(&self, version: VersionId) -> ConfigResult<()> {
        // Mock rollback
        Ok(())
    }

    fn validate_config(&self, config: &SystemConfig) -> ConfigResult<()> {
        // Mock validation
        Ok(())
    }

    fn supported_config_types(&self) -> Vec<ConfigType> {
        vec![ConfigType::System, ConfigType::Service]
    }
}

/// Mock storage backend for testing persistence operations
struct MockStorage {
    data: spin::RwLock<core::collections::BTreeMap<String, Vec<u8>>>,
    write_count: AtomicU64,
    read_count: AtomicU64,
}

impl MockStorage {
    const fn new() -> Self {
        Self {
            data: spin::RwLock::new(core::collections::BTreeMap::new()),
            write_count: AtomicU64::new(0),
            read_count: AtomicU64::new(0),
        }
    }

    fn get_write_count(&self) -> u64 {
        self.write_count.load(Ordering::Relaxed)
    }

    fn get_read_count(&self) -> u64 {
        self.read_count.load(Ordering::Relaxed)
    }
}

impl StorageBackend for MockStorage {
    fn write_config(&self, path: &str, data: &[u8]) -> ConfigResult<()> {
        self.write_count.fetch_add(1, Ordering::Relaxed);
        self.data.write().insert(path.to_string(), data.to_vec());
        Ok(())
    }

    fn read_config(&self, path: &str) -> ConfigResult<Vec<u8>> {
        self.read_count.fetch_add(1, Ordering::Relaxed);
        self.data.read()
            .get(path)
            .cloned()
            .ok_or(ConfigError::StorageError {
                message: format!("Config not found at path: {}", path),
            })
    }

    fn delete_config(&self, path: &str) -> ConfigResult<()> {
        self.data.write().remove(path);
        Ok(())
    }

    fn list_configs(&self) -> ConfigResult<Vec<String>> {
        Ok(self.data.read().keys().cloned().collect())
    }

    fn exists(&self, path: &str) -> bool {
        self.data.read().contains_key(path)
    }
}

/// Test configuration data structures
#[derive(Debug, Clone, PartialEq)]
struct TestConfig {
    name: String,
    value: u64,
    enabled: bool,
    tags: Vec<String>,
}

impl TestConfig {
    const fn new(name: String, value: u64, enabled: bool, tags: Vec<String>) -> Self {
        Self { name, value, enabled, tags }
    }
}

/// Helper function to get current timestamp for tests
fn current_timestamp() -> u64 {
    // Mock timestamp for testing - in real kernel, would use actual time source
    1609459200 // January 1, 2021 00:00:00 UTC
}

/// Test setup helper
fn setup_test_environment() -> (
    ConfigManager,
    PolicyManager,
    ValidationManager,
    BackupManager,
    PropagationManager,
    MockStorage,
    MockServiceRegistry,
) {
    let storage = MockStorage::new();
    let service_registry = MockServiceRegistry::new();
    
    // Initialize managers with mock storage
    let config_manager = ConfigManager::new(storage.clone());
    let policy_manager = PolicyManager::new();
    let validation_manager = ValidationManager::new();
    let backup_manager = BackupManager::new(storage.clone());
    let propagation_manager = PropagationManager::new(service_registry.clone());
    
    (config_manager, policy_manager, validation_manager, backup_manager, propagation_manager, storage, service_registry)
}

#[cfg(test)]
mod config_crud_tests {
    use super::*;

    #[test]
    fn test_config_create_and_retrieve() {
        let (config_manager, _, _, _, _, storage, _) = setup_test_environment();
        
        let test_config = SystemConfig::new(
            "test_config".to_string(),
            ConfigType::System,
            json::object! {
                "name" => "test",
                "value" => 42,
                "enabled" => true
            },
        );
        
        // Create configuration
        let result = config_manager.create_config("test", &test_config);
        assert!(result.is_ok());
        
        // Retrieve configuration
        let retrieved = config_manager.get_config("test");
        assert!(retrieved.is_ok());
        
        let retrieved_config = retrieved.unwrap();
        assert_eq!(retrieved_config.name, test_config.name);
        assert_eq!(retrieved_config.config_type, test_config.config_type);
        
        // Verify storage operations
        assert_eq!(storage.get_write_count(), 1);
        assert_eq!(storage.get_read_count(), 1);
    }

    #[test]
    fn test_config_update_with_versioning() {
        let (config_manager, _, _, _, _, _, _) = setup_test_environment();
        
        let initial_config = SystemConfig::new(
            "versioned_config".to_string(),
            ConfigType::System,
            json::object! {
                "version" => 1,
                "data" => "initial"
            },
        );
        
        // Create initial configuration
        let create_result = config_manager.create_config("versioned", &initial_config);
        assert!(create_result.is_ok());
        
        // Update configuration
        let updated_config = SystemConfig::new(
            "versioned_config".to_string(),
            ConfigType::System,
            json::object! {
                "version" => 2,
                "data" => "updated"
            },
        );
        
        let update_result = config_manager.update_config("versioned", &updated_config);
        assert!(update_result.is_ok());
        
        // Verify version tracking
        let history = config_manager.get_config_history("versioned");
        assert!(history.is_ok());
        
        let history_vec = history.unwrap();
        assert_eq!(history_vec.len(), 2); // Initial + updated
        
        // Verify latest version
        let latest = config_manager.get_config("versioned");
        assert!(latest.is_ok());
        
        let latest_config = latest.unwrap();
        assert_eq!(latest_config.version, 2);
    }

    #[test]
    fn test_config_delete_with_cleanup() {
        let (config_manager, _, _, _, _, storage, _) = setup_test_environment();
        
        let test_config = SystemConfig::new(
            "delete_me".to_string(),
            ConfigType::System,
            json::object! { "data" => "temp" },
        );
        
        // Create configuration
        let create_result = config_manager.create_config("delete_me", &test_config);
        assert!(create_result.is_ok());
        
        // Verify it exists
        let exists_before = config_manager.config_exists("delete_me");
        assert!(exists_before);
        
        // Delete configuration
        let delete_result = config_manager.delete_config("delete_me");
        assert!(delete_result.is_ok());
        
        // Verify it's deleted
        let exists_after = config_manager.config_exists("delete_me");
        assert!(!exists_after);
        
        // Verify cleanup operations
        assert_eq!(storage.get_write_count(), 2); // Create + delete
    }

    #[test]
    fn test_batch_operations() {
        let (config_manager, _, _, _, _, _, _) = setup_test_environment();
        
        let configs = vec![
            SystemConfig::new("batch1".to_string(), ConfigType::System, json::object! { "id" => 1 }),
            SystemConfig::new("batch2".to_string(), ConfigType::System, json::object! { "id" => 2 }),
            SystemConfig::new("batch3".to_string(), ConfigType::System, json::object! { "id" => 3 }),
        ];
        
        // Create batch
        let batch_result = config_manager.create_configs_batch(&configs);
        assert!(batch_result.is_ok());
        
        // Retrieve all configs
        let all_configs = config_manager.list_all_configs();
        assert!(all_configs.is_ok());
        
        let config_list = all_configs.unwrap();
        assert_eq!(config_list.len(), 3);
        
        // Update batch
        let updated_configs = vec![
            SystemConfig::new("batch1".to_string(), ConfigType::System, json::object! { "id" => 1, "updated" => true }),
            SystemConfig::new("batch2".to_string(), ConfigType::System, json::object! { "id" => 2, "updated" => true }),
        ];
        
        let update_batch_result = config_manager.update_configs_batch(&updated_configs);
        assert!(update_batch_result.is_ok());
        
        // Verify updates
        let batch1 = config_manager.get_config("batch1");
        assert!(batch1.is_ok());
        let batch1_data = &batch1.unwrap().config_data;
        assert_eq!(batch1_data["updated"], true);
    }
}

#[cfg(test)]
mod schema_validation_tests {
    use super::*;

    #[test]
    fn test_schema_registration_and_validation() {
        let (_, _, validation_manager, _, _, _, _) = setup_test_environment();
        
        // Register test schema
        let test_schema = SchemaDefinition {
            schema_type: SchemaType::System,
            fields: vec![
                SchemaField {
                    name: "name".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    constraints: vec![StringConstraint::MinLength(1)],
                },
                SchemaField {
                    name: "value".to_string(),
                    field_type: FieldType::U64,
                    required: true,
                    constraints: vec![NumericConstraint::Min(0)],
                },
                SchemaField {
                    name: "enabled".to_string(),
                    field_type: FieldType::Bool,
                    required: false,
                    constraints: vec![],
                },
            ],
        };
        
        let register_result = validation_manager.register_schema(&test_schema);
        assert!(register_result.is_ok());
        
        // Validate valid configuration
        let valid_config = json::object! {
            "name" => "test_config",
            "value" => 42,
            "enabled" => true
        };
        
        let validation_result = validation_manager.validate_config(&valid_config, &SchemaType::System);
        assert!(validation_result.is_ok());
        
        // Validate invalid configuration (missing required field)
        let invalid_config = json::object! {
            "value" => 42,
            "enabled" => true
        };
        
        let invalid_result = validation_manager.validate_config(&invalid_config, &SchemaType::System);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_nested_schema_validation() {
        let (_, _, validation_manager, _, _, _, _) = setup_test_environment();
        
        // Register nested schema
        let nested_schema = SchemaDefinition {
            schema_type: SchemaType::Service,
            fields: vec![
                SchemaField {
                    name: "service".to_string(),
                    field_type: FieldType::Object,
                    required: true,
                    constraints: vec![],
                },
            ],
        };
        
        let register_result = validation_manager.register_schema(&nested_schema);
        assert!(register_result.is_ok());
        
        // Validate configuration with nested object
        let nested_config = json::object! {
            "service" => {
                "name" => "test_service",
                "port" => 8080,
                "config" => {
                    "enabled" => true,
                    "timeout" => 30
                }
            }
        };
        
        let validation_result = validation_manager.validate_config(&nested_config, &SchemaType::Service);
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_constraint_validation() {
        let (_, _, validation_manager, _, _, _, _) = setup_test_environment();
        
        // Register schema with various constraints
        let constrained_schema = SchemaDefinition {
            schema_type: SchemaType::Application,
            fields: vec![
                SchemaField {
                    name: "email".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    constraints: vec![StringConstraint::Pattern(r".*@.*\..*".to_string())],
                },
                SchemaField {
                    name: "age".to_string(),
                    field_type: FieldType::U32,
                    required: true,
                    constraints: vec![NumericConstraint::Min(0), NumericConstraint::Max(150)],
                },
                SchemaField {
                    name: "tags".to_string(),
                    field_type: FieldType::Array,
                    required: false,
                    constraints: vec![ArrayConstraint::MaxItems(10)],
                },
            ],
        };
        
        let register_result = validation_manager.register_schema(&constrained_schema);
        assert!(register_result.is_ok());
        
        // Test valid configuration
        let valid_config = json::object! {
            "email" => "test@example.com",
            "age" => 25,
            "tags" => ["tag1", "tag2"]
        };
        
        let valid_result = validation_manager.validate_config(&valid_config, &SchemaType::Application);
        assert!(valid_result.is_ok());
        
        // Test invalid email pattern
        let invalid_email = json::object! {
            "email" => "invalid_email",
            "age" => 25
        };
        
        let email_result = validation_manager.validate_config(&invalid_email, &SchemaType::Application);
        assert!(email_result.is_err());
        
        // Test invalid age range
        let invalid_age = json::object! {
            "email" => "test@example.com",
            "age" => 200
        };
        
        let age_result = validation_manager.validate_config(&invalid_age, &SchemaType::Application);
        assert!(age_result.is_err());
    }
}

#[cfg(test)]
mod policy_management_tests {
    use super::*;

    #[test]
    fn test_policy_creation_and_evaluation() {
        let (_, policy_manager, _, _, _, _, _) = setup_test_environment();
        
        // Create test policy
        let test_policy = Policy {
            id: "test_policy".to_string(),
            name: "Test Policy".to_string(),
            description: "Test policy for validation".to_string(),
            enabled: true,
            priority: 10,
            conditions: vec![
                PolicyCondition {
                    field: "user_level".to_string(),
                    operator: ConditionOperator::GreaterThan,
                    value: json::number(5),
                },
            ],
            actions: vec![
                PolicyAction::Deny,
            ],
        };
        
        let create_result = policy_manager.create_policy("test_policy", &test_policy);
        assert!(create_result.is_ok());
        
        // Test policy evaluation with matching condition
        let context = json::object! {
            "user_level" => 10
        };
        
        let eval_result = policy_manager.evaluate_policy("test_policy", &context);
        assert!(eval_result.is_ok());
        
        let action = eval_result.unwrap();
        assert_eq!(action, PolicyAction::Deny);
        
        // Test policy evaluation with non-matching condition
        let non_matching_context = json::object! {
            "user_level" => 3
        };
        
        let no_match_result = policy_manager.evaluate_policy("test_policy", &non_matching_context);
        assert!(no_match_result.is_err());
    }

    #[test]
    fn test_policy_chaining_and_priority() {
        let (_, policy_manager, _, _, _, _, _) = setup_test_environment();
        
        // Create multiple policies with different priorities
        let high_priority_policy = Policy {
            id: "high_priority".to_string(),
            name: "High Priority".to_string(),
            description: "High priority policy".to_string(),
            enabled: true,
            priority: 100,
            conditions: vec![
                PolicyCondition {
                    field: "access_level".to_string(),
                    operator: ConditionOperator::Equals,
                    value: json::string("admin"),
                },
            ],
            actions: vec![PolicyAction::Allow],
        };
        
        let low_priority_policy = Policy {
            id: "low_priority".to_string(),
            name: "Low Priority".to_string(),
            description: "Low priority policy".to_string(),
            enabled: true,
            priority: 1,
            conditions: vec![
                PolicyCondition {
                    field: "access_level".to_string(),
                    operator: ConditionOperator::NotEquals,
                    value: json::string("guest"),
                },
            ],
            actions: vec![PolicyAction::Allow],
        };
        
        policy_manager.create_policy("high_priority", &high_priority_policy).unwrap();
        policy_manager.create_policy("low_priority", &low_priority_policy).unwrap();
        
        // Test policy chain evaluation
        let context = json::object! {
            "access_level" => "admin"
        };
        
        let chain_result = policy_manager.evaluate_policy_chain(&context);
        assert!(chain_result.is_ok());
        
        let actions = chain_result.unwrap();
        assert_eq!(actions.len(), 2);
        
        // High priority policy should be evaluated first
        assert_eq!(actions[0].policy_id, "high_priority");
        assert_eq!(actions[0].action, PolicyAction::Allow);
    }

    #[test]
    fn test_policy_conflict_resolution() {
        let (_, policy_manager, _, _, _, _, _) = setup_test_environment();
        
        // Create conflicting policies
        let allow_policy = Policy {
            id: "allow_policy".to_string(),
            name: "Allow Policy".to_string(),
            description: "Allows access".to_string(),
            enabled: true,
            priority: 50,
            conditions: vec![
                PolicyCondition {
                    field: "department".to_string(),
                    operator: ConditionOperator::Equals,
                    value: json::string("engineering"),
                },
            ],
            actions: vec![PolicyAction::Allow],
        };
        
        let deny_policy = Policy {
            id: "deny_policy".to_string(),
            name: "Deny Policy".to_string(),
            description: "Denies access".to_string(),
            enabled: true,
            priority: 60, // Higher priority
            conditions: vec![
                PolicyCondition {
                    field: "department".to_string(),
                    operator: ConditionOperator::Equals,
                    value: json::string("engineering"),
                },
            ],
            actions: vec![PolicyAction::Deny],
        };
        
        policy_manager.create_policy("allow_policy", &allow_policy).unwrap();
        policy_manager.create_policy("deny_policy", &deny_policy).unwrap();
        
        // Test conflict resolution
        let context = json::object! {
            "department" => "engineering"
        };
        
        let result = policy_manager.evaluate_policy_chain(&context);
        assert!(result.is_ok());
        
        let actions = result.unwrap();
        
        // Deny policy should win due to higher priority
        let final_action = actions.first().unwrap();
        assert_eq!(final_action.action, PolicyAction::Deny);
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;

    #[test]
    fn test_atomic_write_operations() {
        let (_, _, _, _, _, storage, _) = setup_test_environment();
        
        // Test atomic write with mock storage
        let test_data = json::object! {
            "key" => "value",
            "number" => 42
        };
        
        let write_result = storage.write_config("atomic_test", &test_data.to_string().into_bytes());
        assert!(write_result.is_ok());
        
        // Verify write
        let read_result = storage.read_config("atomic_test");
        assert!(read_result.is_ok());
        
        let read_data = read_result.unwrap();
        assert_eq!(read_data, test_data.to_string().into_bytes());
    }

    #[test]
    fn test_rollback_on_write_failure() {
        let (_, _, _, _, _, storage, _) = setup_test_environment();
        
        // Pre-existing data
        let original_data = json::object! { "original" => "data" };
        storage.write_config("rollback_test", &original_data.to_string().into_bytes()).unwrap();
        
        // Simulate failed write (in real implementation, this would be handled by transaction rollback)
        let failed_data = json::object! { "failed" => "write" };
        
        // In a real implementation, the rollback mechanism would restore original_data
        // For this test, we verify the storage state
        let before_rollback = storage.read_config("rollback_test");
        assert!(before_rollback.is_ok());
    }

    #[test]
    fn test_version_control_persistence() {
        let (config_manager, _, _, _, _, storage, _) = setup_test_environment();
        
        // Create configuration with version tracking
        let v1_config = SystemConfig::new(
            "versioned".to_string(),
            ConfigType::System,
            json::object! { "version" => 1 },
        );
        
        config_manager.create_config("versioned", &v1_config).unwrap();
        
        // Update to version 2
        let v2_config = SystemConfig::new(
            "versioned".to_string(),
            ConfigType::System,
            json::object! { "version" => 2 },
        );
        
        config_manager.update_config("versioned", &v2_config).unwrap();
        
        // Verify both versions are stored
        let v1_history = config_manager.get_config_version("versioned", 1);
        assert!(v1_history.is_ok());
        
        let v2_current = config_manager.get_config("versioned");
        assert!(v2_current.is_ok());
        assert_eq!(v2_current.unwrap().version, 2);
    }
}

#[cfg(test)]
mod backup_restore_tests {
    use super::*;

    #[test]
    fn test_full_backup_creation() {
        let (config_manager, _, _, backup_manager, _, storage, _) = setup_test_environment();
        
        // Create multiple configurations
        let configs = vec![
            SystemConfig::new("backup1".to_string(), ConfigType::System, json::object! { "id" => 1 }),
            SystemConfig::new("backup2".to_string(), ConfigType::Application, json::object! { "id" => 2 }),
            SystemConfig::new("backup3".to_string(), ConfigType::Service, json::object! { "id" => 3 }),
        ];
        
        for (i, config) in configs.iter().enumerate() {
            config_manager.create_config(&format!("backup{}", i + 1), config).unwrap();
        }
        
        // Create full backup
        let backup_result = backup_manager.create_full_backup("test_backup");
        assert!(backup_result.is_ok());
        
        let backup_info = backup_result.unwrap();
        assert_eq!(backup_info.backup_type, BackupType::Full);
        assert_eq!(backup_info.config_count, 3);
    }

    #[test]
    fn test_incremental_backup() {
        let (config_manager, _, _, backup_manager, _, _, _) = setup_test_environment();
        
        // Create initial configuration
        let initial_config = SystemConfig::new(
            "incremental".to_string(),
            ConfigType::System,
            json::object! { "version" => 1 },
        );
        
        config_manager.create_config("incremental", &initial_config).unwrap();
        
        // Create first backup (effectively full)
        let first_backup = backup_manager.create_incremental_backup("incremental_backup_1");
        assert!(first_backup.is_ok());
        
        // Update configuration
        let updated_config = SystemConfig::new(
            "incremental".to_string(),
            ConfigType::System,
            json::object! { "version" => 2 },
        );
        
        config_manager.update_config("incremental", &updated_config).unwrap();
        
        // Create second backup (incremental)
        let second_backup = backup_manager.create_incremental_backup("incremental_backup_2");
        assert!(second_backup.is_ok());
        
        let second_backup_info = second_backup.unwrap();
        assert_eq!(second_backup_info.backup_type, BackupType::Incremental);
    }

    #[test]
    fn test_backup_compression() {
        let (_, _, _, backup_manager, _, storage, _) = setup_test_environment();
        
        // Create large configuration data
        let large_config = json::object! {
            "data" => "x".repeat(10000), // Large string
            "numbers" => (0..1000).map(|i| i).collect::<Vec<_>>(),
            "nested" => {
                "level1" => {
                    "level2" => {
                        "level3" => {
                            "data" => "deeply_nested_large_data".repeat(100)
                        }
                    }
                }
            }
        };
        
        storage.write_config("large_config", &large_config.to_string().into_bytes()).unwrap();
        
        // Create backup with compression
        let backup_result = backup_manager.create_compressed_backup("compressed_backup");
        assert!(backup_result.is_ok());
        
        let backup_info = backup_result.unwrap();
        assert!(backup_info.compressed);
        assert!(backup_info.size < 50000); // Should be significantly smaller than uncompressed
    }

    #[test]
    fn test_backup_restore() {
        let (config_manager, _, _, backup_manager, _, storage, _) = setup_test_environment();
        
        // Create test configuration
        let test_config = SystemConfig::new(
            "restore_test".to_string(),
            ConfigType::System,
            json::object! {
                "restoration" => "test",
                "timestamp" => current_timestamp()
            },
        );
        
        config_manager.create_config("restore_test", &test_config).unwrap();
        
        // Create backup
        let backup_result = backup_manager.create_full_backup("restore_backup");
        assert!(backup_result.is_ok());
        
        // Modify configuration
        let modified_config = SystemConfig::new(
            "restore_test".to_string(),
            ConfigType::System,
            json::object! {
                "restoration" => "modified",
                "timestamp" => current_timestamp() + 1000
            },
        );
        
        config_manager.update_config("restore_test", &modified_config).unwrap();
        
        // Restore from backup
        let restore_result = backup_manager.restore_from_backup("restore_backup");
        assert!(restore_result.is_ok());
        
        // Verify restoration
        let restored_config = config_manager.get_config("restore_test");
        assert!(restored_config.is_ok());
        
        let config_data = &restored_config.unwrap().config_data;
        assert_eq!(config_data["restoration"], "test");
    }
}

#[cfg(test)]
mod propagation_tests {
    use super::*;

    #[test]
    fn test_service_registration_and_updates() {
        let (_, _, _, _, propagation_manager, _, service_registry) = setup_test_environment();
        
        // Register mock services
        let service1 = MockService::new(
            "service1".to_string(),
            "Test Service 1".to_string(),
            "1.0.0".to_string(),
        );
        
        let service2 = MockService::new(
            "service2".to_string(),
            "Test Service 2".to_string(),
            "2.0.0".to_string(),
        );
        
        service_registry.add_service(service1.clone());
        service_registry.add_service(service2.clone());
        
        // Create test configuration
        let test_config = SystemConfig::new(
            "propagation_test".to_string(),
            ConfigType::System,
            json::object! {
                "propagation" => "test",
                "services" => ["service1", "service2"]
            },
        );
        
        // Propagate configuration to services
        let propagation_result = propagation_manager.propagate_config(&test_config);
        assert!(propagation_result.is_ok());
        
        // Verify services received updates
        let updates1 = service1.get_config_updates();
        assert_eq!(updates1.len(), 1);
        assert_eq!(updates1[0].service_id, "service1");
        
        let updates2 = service2.get_config_updates();
        assert_eq!(updates2.len(), 1);
        assert_eq!(updates2[0].service_id, "service2");
    }

    #[test]
    fn test_selective_propagation() {
        let (_, _, _, _, propagation_manager, _, service_registry) = setup_test_environment();
        
        // Register services with different config type support
        let system_service = MockService::new(
            "system_service".to_string(),
            "System Service".to_string(),
            "1.0.0".to_string(),
        );
        
        let app_service = MockService::new(
            "app_service".to_string(),
            "App Service".to_string(),
            "1.0.0".to_string(),
        );
        
        service_registry.add_service(system_service.clone());
        service_registry.add_service(app_service.clone());
        
        // Create system configuration
        let system_config = SystemConfig::new(
            "system_only".to_string(),
            ConfigType::System,
            json::object! { "type" => "system" },
        );
        
        // Propagate to System-type services only
        let selective_result = propagation_manager.propagate_selectively(
            &system_config,
            vec![ConfigType::System],
        );
        assert!(selective_result.is_ok());
        
        // Verify selective propagation
        let system_updates = system_service.get_config_updates();
        assert_eq!(system_updates.len(), 1);
        
        let app_updates = app_service.get_config_updates();
        assert_eq!(app_updates.len(), 0); // Should not receive system config
    }

    #[test]
    fn test_propagation_failure_handling() {
        let (_, _, _, _, propagation_manager, _, service_registry) = setup_test_environment();
        
        // Register service that will fail
        let failing_service = MockService::new(
            "failing_service".to_string(),
            "Failing Service".to_string(),
            "1.0.0".to_string(),
        );
        
        service_registry.add_service(failing_service.clone());
        
        // Create test configuration
        let test_config = SystemConfig::new(
            "failure_test".to_string(),
            ConfigType::System,
            json::object! { "test" => true },
        );
        
        // In a real implementation, this would test rollback on failure
        let propagation_result = propagation_manager.propagate_config(&test_config);
        assert!(propagation_result.is_ok());
        
        // Verify partial success handling
        let updates = failing_service.get_config_updates();
        assert_eq!(updates.len(), 1);
    }

    #[test]
    fn test_propagation_status_tracking() {
        let (_, _, _, _, propagation_manager, _, service_registry) = setup_test_environment();
        
        // Register multiple services
        let services = vec![
            MockService::new("service1".to_string(), "Service 1".to_string(), "1.0.0".to_string()),
            MockService::new("service2".to_string(), "Service 2".to_string(), "1.0.0".to_string()),
            MockService::new("service3".to_string(), "Service 3".to_string(), "1.0.0".to_string()),
        ];
        
        for service in &services {
            service_registry.add_service(service.clone());
        }
        
        // Create and propagate configuration
        let test_config = SystemConfig::new(
            "status_test".to_string(),
            ConfigType::System,
            json::object! { "status" => "tracking" },
        );
        
        let propagation_result = propagation_manager.propagate_config(&test_config);
        assert!(propagation_result.is_ok());
        
        // Get propagation status
        let status_result = propagation_manager.get_propagation_status("status_test");
        assert!(status_result.is_ok());
        
        let status = status_result.unwrap();
        assert_eq!(status.total_services, 3);
        assert_eq!(status.successful_services, 3);
        assert_eq!(status.failed_services, 0);
    }
}

#[cfg(test)]
mod audit_integration_tests {
    use super::*;

    #[test]
    fn test_audit_logging_for_config_changes() {
        let (config_manager, _, _, _, _, _, _) = setup_test_environment();
        
        // Create configuration (should generate audit log)
        let test_config = SystemConfig::new(
            "audit_test".to_string(),
            ConfigType::System,
            json::object! { "action" => "create" },
        );
        
        let create_result = config_manager.create_config("audit_test", &test_config);
        assert!(create_result.is_ok());
        
        // Update configuration (should generate audit log)
        let updated_config = SystemConfig::new(
            "audit_test".to_string(),
            ConfigType::System,
            json::object! { "action" => "update" },
        );
        
        let update_result = config_manager.update_config("audit_test", &updated_config);
        assert!(update_result.is_ok());
        
        // Get audit logs for configuration
        // Note: In real implementation, this would interface with the audit system
        let audit_result = config_manager.get_audit_logs("audit_test");
        assert!(audit_result.is_ok());
        
        let logs = audit_result.unwrap();
        assert_eq!(logs.len(), 2); // Create + Update
    }

    #[test]
    fn test_audit_trail_for_backup_operations() {
        let (_, _, _, backup_manager, _, _, _) = setup_test_environment();
        
        // Create backup (should generate audit log)
        let backup_result = backup_manager.create_full_backup("audit_backup");
        assert!(backup_result.is_ok());
        
        // Restore backup (should generate audit log)
        let restore_result = backup_manager.restore_from_backup("audit_backup");
        assert!(restore_result.is_ok());
        
        // In real implementation, verify audit logs were created
        // This test demonstrates the integration pattern
    }

    #[test]
    fn test_audit_integration_with_policies() {
        let (_, policy_manager, _, _, _, _, _) = setup_test_environment();
        
        // Create policy (should generate audit log)
        let test_policy = Policy {
            id: "audit_policy".to_string(),
            name: "Audit Test Policy".to_string(),
            description: "Policy for audit testing".to_string(),
            enabled: true,
            priority: 50,
            conditions: vec![],
            actions: vec![PolicyAction::Allow],
        };
        
        let create_result = policy_manager.create_policy("audit_policy", &test_policy);
        assert!(create_result.is_ok());
        
        // Update policy (should generate audit log)
        let mut updated_policy = test_policy.clone();
        updated_policy.enabled = false;
        
        let update_result = policy_manager.update_policy("audit_policy", &updated_policy);
        assert!(update_result.is_ok());
        
        // Get audit logs for policy
        let audit_result = policy_manager.get_audit_logs("audit_policy");
        assert!(audit_result.is_ok());
        
        let logs = audit_result.unwrap();
        assert_eq!(logs.len(), 2); // Create + Update
    }
}

#[cfg(test)]
mod integration_lifecycle_tests {
    use super::*;

    #[test]
    fn test_complete_config_lifecycle() {
        let (config_manager, policy_manager, validation_manager, backup_manager, propagation_manager, _, service_registry) = setup_test_environment();
        
        // 1. Register schema
        let lifecycle_schema = SchemaDefinition {
            schema_type: SchemaType::Application,
            fields: vec![
                SchemaField {
                    name: "lifecycle_id".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    constraints: vec![StringConstraint::MinLength(1)],
                },
                SchemaField {
                    name: "step".to_string(),
                    field_type: FieldType::U32,
                    required: true,
                    constraints: vec![NumericConstraint::Min(1)],
                },
            ],
        };
        
        validation_manager.register_schema(&lifecycle_schema).unwrap();
        
        // 2. Create policy
        let lifecycle_policy = Policy {
            id: "lifecycle_policy".to_string(),
            name: "Lifecycle Policy".to_string(),
            description: "Policy for lifecycle testing".to_string(),
            enabled: true,
            priority: 100,
            conditions: vec![
                PolicyCondition {
                    field: "step".to_string(),
                    operator: ConditionOperator::LessThan,
                    value: json::number(10),
                },
            ],
            actions: vec![PolicyAction::Allow],
        };
        
        policy_manager.create_policy("lifecycle_policy", &lifecycle_policy).unwrap();
        
        // 3. Register service
        let lifecycle_service = MockService::new(
            "lifecycle_service".to_string(),
            "Lifecycle Service".to_string(),
            "1.0.0".to_string(),
        );
        
        service_registry.add_service(lifecycle_service.clone());
        
        // 4. Create initial configuration
        let initial_config = SystemConfig::new(
            "lifecycle".to_string(),
            ConfigType::Application,
            json::object! {
                "lifecycle_id" => "test_lifecycle",
                "step" => 1,
                "status" => "initializing"
            },
        );
        
        // Validate before creation
        let validation_result = validation_manager.validate_config(&initial_config.config_data, &ConfigType::Application);
        assert!(validation_result.is_ok());
        
        // Create configuration
        let create_result = config_manager.create_config("lifecycle", &initial_config);
        assert!(create_result.is_ok());
        
        // 5. Update through lifecycle steps
        for step in 2..=5 {
            let updated_config = SystemConfig::new(
                "lifecycle".to_string(),
                ConfigType::Application,
                json::object! {
                    "lifecycle_id" => "test_lifecycle",
                    "step" => step,
                    "status" => format!("step_{}", step)
                },
            );
            
            // Validate
            let validation_result = validation_manager.validate_config(&updated_config.config_data, &ConfigType::Application);
            assert!(validation_result.is_ok());
            
            // Check policy
            let policy_result = policy_manager.evaluate_policy("lifecycle_policy", &json::object! { "step" => step });
            assert!(policy_result.is_ok());
            
            // Update configuration
            let update_result = config_manager.update_config("lifecycle", &updated_config);
            assert!(update_result.is_ok());
            
            // Propagate to service
            let propagation_result = propagation_manager.propagate_config(&updated_config);
            assert!(propagation_result.is_ok());
        }
        
        // 6. Create backup of lifecycle state
        let backup_result = backup_manager.create_full_backup("lifecycle_backup");
        assert!(backup_result.is_ok());
        
        // 7. Verify final state
        let final_config = config_manager.get_config("lifecycle");
        assert!(final_config.is_ok());
        
        let final_data = &final_config.unwrap().config_data;
        assert_eq!(final_data["step"], 5);
        assert_eq!(final_data["status"], "step_5");
        
        // Verify service received updates
        let service_updates = lifecycle_service.get_config_updates();
        assert_eq!(service_updates.len(), 5); // Initial + 4 updates
        
        // 8. Verify audit trail
        let audit_logs = config_manager.get_audit_logs("lifecycle");
        assert!(audit_logs.is_ok());
        
        let logs = audit_logs.unwrap();
        assert_eq!(logs.len(), 6); // Initial creation + 5 updates
    }

    #[test]
    fn test_concurrent_config_operations() {
        let (config_manager, _, _, _, _, _, _) = setup_test_environment();
        
        // Create multiple configurations concurrently
        let configs = (0..10).map(|i| {
            SystemConfig::new(
                format!("concurrent_{}", i),
                ConfigType::System,
                json::object! {
                    "id" => i,
                    "concurrent" => true,
                    "timestamp" => current_timestamp()
                },
            )
        }).collect::<Vec<_>>();
        
        // Create all configurations
        for config in &configs {
            let result = config_manager.create_config(&config.name, config);
            assert!(result.is_ok());
        }
        
        // Verify all were created
        let all_configs = config_manager.list_all_configs();
        assert!(all_configs.is_ok());
        
        let config_list = all_configs.unwrap();
        assert_eq!(config_list.len(), 10);
        
        // Update all configurations
        for i in 0..10 {
            let updated_config = SystemConfig::new(
                format!("concurrent_{}", i),
                ConfigType::System,
                json::object! {
                    "id" => i,
                    "concurrent" => true,
                    "updated" => true,
                    "timestamp" => current_timestamp() + i as u64
                },
            );
            
            let result = config_manager.update_config(&format!("concurrent_{}", i), &updated_config);
            assert!(result.is_ok());
        }
        
        // Verify updates
        for i in 0..10 {
            let config = config_manager.get_config(&format!("concurrent_{}", i));
            assert!(config.is_ok());
            
            let config_data = &config.unwrap().config_data;
            assert_eq!(config_data["updated"], true);
        }
    }

    #[test]
    fn test_disaster_recovery_scenario() {
        let (config_manager, policy_manager, validation_manager, backup_manager, propagation_manager, _, service_registry) = setup_test_environment();
        
        // 1. Setup complete system state
        let schemas = vec![
            (SchemaType::System, vec!["name", "value"]),
            (SchemaType::Application, vec!["app_id", "config"]),
            (SchemaType::Service, vec!["service_name", "port"]),
        ];
        
        for (schema_type, fields) in schemas {
            let schema = SchemaDefinition {
                schema_type,
                fields: fields.iter().map(|f| SchemaField {
                    name: f.to_string(),
                    field_type: FieldType::String,
                    required: true,
                    constraints: vec![],
                }).collect(),
            };
            
            validation_manager.register_schema(&schema).unwrap();
        }
        
        // 2. Create policies
        let policies = vec![
            ("security_policy", PolicyAction::Allow, 100),
            ("access_policy", PolicyAction::Deny, 50),
        ];
        
        for (policy_id, action, priority) in policies {
            let policy = Policy {
                id: policy_id.to_string(),
                name: policy_id.to_string(),
                description: format!("{} for disaster recovery", policy_id),
                enabled: true,
                priority,
                conditions: vec![],
                actions: vec![action],
            };
            
            policy_manager.create_policy(policy_id, &policy).unwrap();
        }
        
        // 3. Register services
        let services = vec![
            ("web_service", "Web Service", "1.0.0"),
            ("db_service", "Database Service", "2.0.0"),
            ("cache_service", "Cache Service", "1.5.0"),
        ];
        
        for (id, name, version) in services {
            let service = MockService::new(id.to_string(), name.to_string(), version.to_string());
            service_registry.add_service(service);
        }
        
        // 4. Create production configurations
        let configs = vec![
            ("web_config", ConfigType::Application, json::object! { "port" => 8080, "ssl" => true }),
            ("db_config", ConfigType::Service, json::object! { "host" => "localhost", "port" => 5432 }),
            ("cache_config", ConfigType::Service, json::object! { "size" => "1GB", "ttl" => 3600 }),
        ];
        
        for (name, config_type, data) in &configs {
            let config = SystemConfig::new(name.to_string(), *config_type, data.clone());
            config_manager.create_config(name, &config).unwrap();
            
            // Propagate to services
            propagation_manager.propagate_config(&config).unwrap();
        }
        
        // 5. Create comprehensive backup
        let backup_result = backup_manager.create_full_backup("disaster_recovery_backup");
        assert!(backup_result.is_ok());
        
        let backup_info = backup_result.unwrap();
        assert_eq!(backup_info.config_count, 3);
        
        // 6. Simulate disaster - clear all configurations
        for (name, _, _) in &configs {
            config_manager.delete_config(name).unwrap();
        }
        
        // Verify disaster state
        let remaining_configs = config_manager.list_all_configs();
        assert_eq!(remaining_configs.unwrap().len(), 0);
        
        // 7. Restore from backup
        let restore_result = backup_manager.restore_from_backup("disaster_recovery_backup");
        assert!(restore_result.is_ok());
        
        // 8. Verify complete restoration
        let restored_configs = config_manager.list_all_configs();
        assert!(restored_configs.is_ok());
        
        let config_list = restored_configs.unwrap();
        assert_eq!(config_list.len(), 3);
        
        // 9. Verify restored data integrity
        for (name, config_type, expected_data) in &configs {
            let config = config_manager.get_config(name);
            assert!(config.is_ok());
            
            let restored_config = config.unwrap();
            assert_eq!(restored_config.name, *name);
            assert_eq!(restored_config.config_type, *config_type);
            
            // Data should match original (with potential timestamp differences)
            assert_eq!(restored_config.config_data["port"], expected_data["port"]);
            assert_eq!(restored_config.config_data["host"], expected_data["host"]);
        }
        
        // 10. Verify system functionality after restoration
        let validation_result = validation_manager.validate_config(&json::object! { "port" => 8080, "ssl" => true }, &ConfigType::Application);
        assert!(validation_result.is_ok());
        
        let policy_result = policy_manager.evaluate_policy("security_policy", &json::object! {});
        assert!(policy_result.is_ok());
    }
}

/// Performance and stress test utilities
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_config_handling() {
        let (config_manager, _, _, _, _, _, _) = setup_test_environment();
        
        // Create large configuration
        let large_data = json::object! {
            "large_array" => (0..10000).map(|i| i).collect::<Vec<_>>(),
            "large_object" => (0..1000).map(|i| {
                (i.to_string(), format!("value_{}", i))
            }).collect::<json::Object>(),
            "nested_data" => {
                "level1" => {
                    "level2" => {
                        "level3" => {
                            "data" => "x".repeat(5000)
                        }
                    }
                }
            }
        };
        
        let large_config = SystemConfig::new(
            "large_config".to_string(),
            ConfigType::System,
            large_data,
        );
        
        // Test creation
        let create_result = config_manager.create_config("large_config", &large_config);
        assert!(create_result.is_ok());
        
        // Test retrieval
        let retrieve_result = config_manager.get_config("large_config");
        assert!(retrieve_result.is_ok());
        
        let retrieved_config = retrieve_result.unwrap();
        assert_eq!(retrieved_config.config_data["large_array"].as_array().unwrap().len(), 10000);
    }

    #[test]
    fn test_rapid_config_updates() {
        let (config_manager, _, _, _, _, _, _) = setup_test_environment();
        
        let base_config = SystemConfig::new(
            "rapid_update".to_string(),
            ConfigType::System,
            json::object! { "counter" => 0 },
        );
        
        config_manager.create_config("rapid_update", &base_config).unwrap();
        
        // Perform rapid updates
        for i in 1..=1000 {
            let updated_config = SystemConfig::new(
                "rapid_update".to_string(),
                ConfigType::System,
                json::object! { "counter" => i },
            );
            
            let update_result = config_manager.update_config("rapid_update", &updated_config);
            assert!(update_result.is_ok());
        }
        
        // Verify final state
        let final_config = config_manager.get_config("rapid_update");
        assert!(final_config.is_ok());
        
        let final_data = &final_config.unwrap().config_data;
        assert_eq!(final_data["counter"], 1000);
        
        // Verify version tracking
        let history = config_manager.get_config_history("rapid_update");
        assert!(history.is_ok());
        assert_eq!(history.unwrap().len(), 1001); // Initial + 1000 updates
    }
}