//! Configuration Schema Management
//! 
//! This module provides schema validation and management for system configurations
//! including schema definitions, validation rules, and type checking.

#![no_std]

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{ConfigValue, ConfigKey, ConfigType, ConfigResult, ConfigError};

/// Schema definition for a configuration value
#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    pub namespace: String,
    pub key: String,
    pub value_type: ConfigType,
    pub required: bool,
    pub validation_rules: ValidationRules,
    pub default_value: Option<ConfigValue>,
    pub description: Option<String>,
    pub min_value: Option<ConfigValue>,
    pub max_value: Option<ConfigValue>,
    pub allowed_values: Option<Vec<ConfigValue>>,
    pub pattern: Option<String>,
    pub enum_values: Option<Vec<String>>,
    pub dependencies: Vec<SchemaDependency>,
}

/// Validation rules for configuration values
#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub regex_pattern: Option<String>,
    pub custom_validator: Option<String>,
    pub range_validation: bool,
    pub type_enforcement: bool,
}

/// Schema dependency definition
#[derive(Debug, Clone)]
pub struct SchemaDependency {
    pub key: String,
    pub condition: DependencyCondition,
    pub required_for: Vec<String>,
}

/// Dependency conditions
#[derive(Debug, Clone)]
pub enum DependencyCondition {
    Equals(ConfigValue),
    NotEquals(ConfigValue),
    GreaterThan(ConfigValue),
    LessThan(ConfigValue),
    InRange(ConfigValue, ConfigValue),
    Contains(String),
}

/// Schema registry
pub struct ConfigSchema {
    schemas: RwLock<HashMap<String, SchemaDefinition>>,
    next_schema_id: AtomicU64,
    validation_cache: RwLock<HashMap<String, ValidationResult>>,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub validation_time: u64,
}

impl ConfigSchema {
    /// Create a new schema manager
    pub fn new() -> Self {
        ConfigSchema {
            schemas: RwLock::new(HashMap::new()),
            next_schema_id: AtomicU64::new(1),
            validation_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize the schema manager
    pub fn init(&self) -> ConfigResult<()> {
        // Load default schemas
        self.load_default_schemas()?;
        
        info!("Schema manager initialized");
        Ok(())
    }

    /// Register a new schema
    pub fn register_schema(&self, schema: SchemaDefinition) -> ConfigResult<()> {
        let key = format!("{}.{}", schema.namespace, schema.key);
        let mut schemas = self.schemas.write();
        schemas.insert(key, schema);
        
        info!("Schema registered: {}", key);
        Ok(())
    }

    /// Validate a configuration value against its schema
    pub fn validate_value(&self, key: &ConfigKey, value: &ConfigValue) -> ConfigResult<()> {
        let schema_key = format!("{}.{}", key.namespace, key.key);
        let schemas = self.schemas.read();
        
        if let Some(schema) = schemas.get(&schema_key) {
            drop(schemas);
            
            let result = self.perform_validation(schema, value);
            if !result.valid {
                return Err(ConfigError::ValidationFailed);
            }
        } else {
            // No specific schema, apply general validation
            self.general_validation(value)?;
        }
        
        Ok(())
    }

    /// Get schema for a key
    pub fn get_schema(&self, namespace: &str, key: &str) -> ConfigResult<SchemaDefinition> {
        let schema_key = format!("{}.{}", namespace, key);
        let schemas = self.schemas.read();
        
        schemas.get(&schema_key)
            .cloned()
            .ok_or(ConfigError::NotFound)
    }

    /// Update schema validation cache
    pub fn update_cache(&self, key: &str, result: ValidationResult) {
        let mut cache = self.validation_cache.write();
        cache.insert(key.to_string(), result);
    }

    /// Clear validation cache
    pub fn clear_cache(&self) {
        let mut cache = self.validation_cache.write();
        cache.clear();
    }

    /// Load default system schemas
    fn load_default_schemas(&self) -> ConfigResult<()> {
        // System boot configuration schema
        let boot_schema = SchemaDefinition {
            namespace: "system".to_string(),
            key: "boot".to_string(),
            value_type: ConfigType::Object,
            required: true,
            validation_rules: ValidationRules {
                min_length: None,
                max_length: None,
                regex_pattern: None,
                custom_validator: None,
                range_validation: false,
                type_enforcement: true,
            },
            default_value: Some(ConfigValue::Object(HashMap::new())),
            description: Some("System boot configuration".to_string()),
            min_value: None,
            max_value: None,
            allowed_values: None,
            pattern: None,
            enum_values: None,
            dependencies: vec![],
        };

        // Network configuration schema
        let network_schema = SchemaDefinition {
            namespace: "system".to_string(),
            key: "network".to_string(),
            value_type: ConfigType::Object,
            required: true,
            validation_rules: ValidationRules {
                min_length: None,
                max_length: None,
                regex_pattern: None,
                custom_validator: None,
                range_validation: false,
                type_enforcement: true,
            },
            default_value: Some(ConfigValue::Object(HashMap::new())),
            description: Some("Network configuration".to_string()),
            min_value: None,
            max_value: None,
            allowed_values: None,
            pattern: None,
            enum_values: None,
            dependencies: vec![],
        };

        // Security policy schema
        let security_schema = SchemaDefinition {
            namespace: "system".to_string(),
            key: "security".to_string(),
            value_type: ConfigType::Object,
            required: true,
            validation_rules: ValidationRules {
                min_length: None,
                max_length: None,
                regex_pattern: None,
                custom_validator: None,
                range_validation: true,
                type_enforcement: true,
            },
            default_value: Some(ConfigValue::Object(HashMap::new())),
            description: Some("Security policy configuration".to_string()),
            min_value: Some(ConfigValue::Integer(1)),
            max_value: Some(ConfigValue::Integer(5)),
            allowed_values: None,
            pattern: None,
            enum_values: None,
            dependencies: vec![],
        };

        self.register_schema(boot_schema)?;
        self.register_schema(network_schema)?;
        self.register_schema(security_schema)?;

        info!("Default schemas loaded");
        Ok(())
    }

    /// Perform detailed validation against a schema
    fn perform_validation(&self, schema: &SchemaDefinition, value: &ConfigValue) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check value type
        if !self.type_matches(value, schema.value_type) {
            errors.push(format!("Value type mismatch: expected {:?}, got {:?}", 
                             schema.value_type, value_type(value)));
        }

        // Check range validation
        if let (Some(min), Some(max)) = (&schema.min_value, &schema.max_value) {
            if !self.is_in_range(value, min, max) {
                errors.push("Value out of allowed range".to_string());
            }
        }

        // Check allowed values
        if let Some(allowed) = &schema.allowed_values {
            if !self.is_allowed_value(value, allowed) {
                errors.push("Value not in allowed set".to_string());
            }
        }

        // Check string-specific validations
        if let ConfigValue::String(s) = value {
            if let Some(max_len) = schema.validation_rules.max_length {
                if s.len() > max_len {
                    errors.push(format!("String too long: max {}", max_len));
                }
            }
            
            if let Some(min_len) = schema.validation_rules.min_length {
                if s.len() < min_len {
                    warnings.push(format!("String too short: min {}", min_len));
                }
            }
        }

        // Check pattern validation
        if let (Some(pattern), ConfigValue::String(s)) = (&schema.pattern, value) {
            // Simple pattern matching (would use regex in real implementation)
            if !s.contains(pattern) {
                warnings.push(format!("String doesn't match pattern: {}", pattern));
            }
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            validation_time: super::get_current_time(),
        }
    }

    /// General validation for values without specific schemas
    fn general_validation(&self, value: &ConfigValue) -> ConfigResult<()> {
        match value {
            ConfigValue::String(s) if s.is_empty() => {
                Err(ConfigError::ValidationFailed)
            },
            ConfigValue::Array(arr) if arr.is_empty() => {
                Err(ConfigError::ValidationFailed)
            },
            ConfigValue::Object(obj) if obj.is_empty() => {
                Err(ConfigError::ValidationFailed)
            },
            _ => Ok(()),
        }
    }

    /// Check if value type matches schema type
    fn type_matches(&self, value: &ConfigValue, expected_type: ConfigType) -> bool {
        match (value, expected_type) {
            (ConfigValue::String(_), ConfigType::String) => true,
            (ConfigValue::Integer(_), ConfigType::Integer) => true,
            (ConfigValue::Unsigned(_), ConfigType::Unsigned) => true,
            (ConfigValue::Boolean(_), ConfigType::Boolean) => true,
            (ConfigValue::Float(_), ConfigType::Float) => true,
            (ConfigValue::Array(_), ConfigType::Array) => true,
            (ConfigValue::Object(_), ConfigType::Object) => true,
            (ConfigValue::None, ConfigType::None) => true,
            // Allow type coercion in some cases
            (ConfigValue::Integer(i), ConfigType::Unsigned) if *i >= 0 => true,
            (ConfigValue::Unsigned(u), ConfigType::Integer) => true,
            (ConfigValue::Integer(i), ConfigType::Float) => true,
            (ConfigValue::Unsigned(u), ConfigType::Float) => true,
            (ConfigValue::Float(_), ConfigType::Integer) => true,
            (ConfigValue::Float(_), ConfigType::Unsigned) => true,
            _ => false,
        }
    }

    /// Check if value is in range
    fn is_in_range(&self, value: &ConfigValue, min: &ConfigValue, max: &ConfigValue) -> bool {
        match (value, min, max) {
            (ConfigValue::Integer(v), ConfigValue::Integer(min), ConfigValue::Integer(max)) => 
                *v >= *min && *v <= *max,
            (ConfigValue::Unsigned(v), ConfigValue::Unsigned(min), ConfigValue::Unsigned(max)) => 
                *v >= *min && *v <= *max,
            (ConfigValue::Float(v), ConfigValue::Float(min), ConfigValue::Float(max)) => 
                *v >= *min && *v <= *max,
            _ => false,
        }
    }

    /// Check if value is in allowed set
    fn is_allowed_value(&self, value: &ConfigValue, allowed: &[ConfigValue]) -> bool {
        allowed.iter().any(|allowed_val| value == allowed_val)
    }
}

/// Helper function to get value type name
fn value_type(value: &ConfigValue) -> &'static str {
    match value {
        ConfigValue::String(_) => "String",
        ConfigValue::Integer(_) => "Integer",
        ConfigValue::Unsigned(_) => "Unsigned",
        ConfigValue::Boolean(_) => "Boolean",
        ConfigValue::Float(_) => "Float",
        ConfigValue::Array(_) => "Array",
        ConfigValue::Object(_) => "Object",
        ConfigValue::None => "None",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = SchemaDefinition {
            namespace: "test".to_string(),
            key: "key1".to_string(),
            value_type: ConfigType::String,
            required: true,
            validation_rules: ValidationRules {
                min_length: Some(1),
                max_length: Some(100),
                regex_pattern: None,
                custom_validator: None,
                range_validation: false,
                type_enforcement: true,
            },
            default_value: Some(ConfigValue::String("default".to_string())),
            description: Some("Test schema".to_string()),
            min_value: None,
            max_value: None,
            allowed_values: None,
            pattern: None,
            enum_values: None,
            dependencies: vec![],
        };

        assert_eq!(schema.namespace, "test");
        assert_eq!(schema.value_type, ConfigType::String);
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec!["test warning".to_string()],
            validation_time: 1000000,
        };

        assert!(result.valid);
        assert_eq!(result.warnings.len(), 1);
    }
}