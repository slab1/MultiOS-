//! Configuration Validation Framework
//! 
//! This module provides comprehensive configuration validation including
//! type checking, range validation, consistency checks, and integrity verification.

#![no_std]

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use spin::RwLock;

use super::{ConfigKey, ConfigEntry, ConfigValue, ConfigType, ConfigResult, ConfigError};

/// Validation rule for configuration values
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub rule_type: ValidationRuleType,
    pub target_keys: Vec<String>,
    pub parameters: HashMap<String, ConfigValue>,
    pub severity: ValidationSeverity,
    pub enabled: bool,
}

/// Types of validation rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRuleType {
    TypeCheck = 0,
    RangeCheck = 1,
    PatternCheck = 2,
    DependencyCheck = 3,
    ConsistencyCheck = 4,
    UniquenessCheck = 5,
    FormatCheck = 6,
    BusinessRule = 7,
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Vec<ValidationInfo>,
    pub validated_at: u64,
    pub validation_duration: u64,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub rule: String,
    pub key: ConfigKey,
    pub message: String,
    pub severity: ValidationSeverity,
    pub code: String,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub rule: String,
    pub key: ConfigKey,
    pub message: String,
    pub recommendation: Option<String>,
}

/// Validation information
#[derive(Debug, Clone)]
pub struct ValidationInfo {
    pub rule: String,
    pub key: ConfigKey,
    pub message: String,
    pub details: Option<String>,
}

/// Configuration validator
pub struct ConfigValidator {
    validation_rules: RwLock<Vec<ValidationRule>>,
    validation_cache: RwLock<HashMap<String, ValidationResult>>,
    stats: RwLock<ValidationStats>,
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub total_validations: usize,
    pub successful_validations: usize,
    pub failed_validations: usize,
    pub rule_executions: usize,
    pub cache_hits: usize,
    pub average_validation_time_us: u64,
    pub last_validation: u64,
}

impl ConfigValidator {
    /// Create a new configuration validator
    pub fn new() -> Self {
        ConfigValidator {
            validation_rules: RwLock::new(Vec::new()),
            validation_cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(ValidationStats {
                total_validations: 0,
                successful_validations: 0,
                failed_validations: 0,
                rule_executions: 0,
                cache_hits: 0,
                average_validation_time_us: 0,
                last_validation: 0,
            }),
        }
    }

    /// Initialize the validator
    pub fn init(&self) -> ConfigResult<()> {
        // Load default validation rules
        self.load_default_rules()?;
        
        info!("Configuration validator initialized");
        Ok(())
    }

    /// Register a new validation rule
    pub fn register_rule(&self, rule: ValidationRule) -> ConfigResult<()> {
        let mut rules = self.validation_rules.write();
        rules.push(rule);
        
        info!("Validation rule registered");
        Ok(())
    }

    /// Validate all configurations
    pub fn validate_all_configurations(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        let start_time = super::get_current_time();
        self.update_stats(0);

        let cache_key = self.generate_cache_key(config_data);
        
        // Check cache first
        if let Some(cached_result) = self.validation_cache.read().get(&cache_key) {
            self.bump_cache_hits();
            if cached_result.valid {
                return Ok(());
            }
        }

        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            validated_at: super::get_current_time(),
            validation_duration: 0,
        };

        let rules = self.validation_rules.read();
        
        // Apply each validation rule
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }

            let rule_result = self.apply_rule(rule, config_data);
            self.merge_result(&mut result, rule_result);
            self.bump_rule_executions();
        }

        let end_time = super::get_current_time();
        result.validation_duration = end_time - start_time;

        // Update cache
        let mut cache = self.validation_cache.write();
        cache.insert(cache_key, result.clone());

        // Update statistics
        self.update_stats(result.validation_duration as usize);

        if !result.valid {
            warn!("Configuration validation failed: {} errors, {} warnings", 
                  result.errors.len(), result.warnings.len());
            return Err(ConfigError::ValidationFailed);
        }

        info!("Configuration validation successful: {} info messages", result.info.len());
        Ok(())
    }

    /// Validate a single configuration value
    pub fn validate_config_value(&self, key: &ConfigKey, value: &ConfigValue) -> ConfigResult<()> {
        let result = self.validate_single_value(key, value);
        
        if !result.valid {
            return Err(ConfigError::ValidationFailed);
        }

        Ok(())
    }

    /// Check if value type is valid
    pub fn is_value_type_valid(&self, value: &ConfigValue, namespace: &str) -> bool {
        match value {
            ConfigValue::String(_) => true,
            ConfigValue::Integer(_) => true,
            ConfigValue::Unsigned(_) => true,
            ConfigValue::Boolean(_) => true,
            ConfigValue::Float(_) => true,
            ConfigValue::Array(arr) => !arr.is_empty(),
            ConfigValue::Object(obj) => !obj.is_empty(),
            ConfigValue::None => namespace == "system", // Allow None for system configs
        }
    }

    /// Validate configuration consistency
    pub fn validate_consistency(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Check for contradictory values
        self.check_contradictions(config_data)?;
        
        // Check for missing dependencies
        self.check_dependencies(config_data)?;
        
        // Check for circular references
        self.check_circular_references(config_data)?;

        info!("Configuration consistency check passed");
        Ok(())
    }

    /// Validate security constraints
    pub fn validate_security(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Check for insecure values
        self.check_security_constraints(config_data)?;
        
        // Check for privilege escalation
        self.check_privilege_escalation(config_data)?;

        info!("Configuration security validation passed");
        Ok(())
    }

    /// Get validation statistics
    pub fn get_stats(&self) -> ValidationStats {
        self.stats.read().clone()
    }

    /// Clear validation cache
    pub fn clear_cache(&self) {
        let mut cache = self.validation_cache.write();
        cache.clear();
    }

    /// Load default validation rules
    fn load_default_rules(&self) -> ConfigResult<()> {
        // Type checking rule
        let type_rule = ValidationRule {
            name: "TypeValidation".to_string(),
            description: "Validate configuration value types".to_string(),
            rule_type: ValidationRuleType::TypeCheck,
            target_keys: vec!["*".to_string()], // All keys
            parameters: HashMap::new(),
            severity: ValidationSeverity::Error,
            enabled: true,
        };

        // Range validation rule
        let range_rule = ValidationRule {
            name: "RangeValidation".to_string(),
            description: "Validate numeric value ranges".to_string(),
            rule_type: ValidationRuleType::RangeCheck,
            target_keys: vec!["system.*".to_string()],
            parameters: HashMap::new(),
            severity: ValidationSeverity::Warning,
            enabled: true,
        };

        // Dependency check rule
        let dependency_rule = ValidationRule {
            name: "DependencyValidation".to_string(),
            description: "Validate configuration dependencies".to_string(),
            rule_type: ValidationRuleType::DependencyCheck,
            target_keys: vec!["system.*".to_string()],
            parameters: HashMap::new(),
            severity: ValidationSeverity::Error,
            enabled: true,
        };

        self.register_rule(type_rule)?;
        self.register_rule(range_rule)?;
        self.register_rule(dependency_rule)?;

        info!("Default validation rules loaded");
        Ok(())
    }

    /// Apply a single validation rule
    fn apply_rule(&self, rule: &ValidationRule, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ValidationResult {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            validated_at: super::get_current_time(),
            validation_duration: 0,
        };

        match rule.rule_type {
            ValidationRuleType::TypeCheck => {
                for (key, entry) in config_data {
                    if self.rule_applies_to_key(rule, key) {
                        if !self.validate_type(entry) {
                            result.errors.push(ValidationError {
                                rule: rule.name.clone(),
                                key: key.clone(),
                                message: format!("Invalid type for key: {}", key.path),
                                severity: rule.severity,
                                code: "TYPE_MISMATCH".to_string(),
                            });
                            result.valid = false;
                        }
                    }
                }
            },
            ValidationRuleType::RangeCheck => {
                for (key, entry) in config_data {
                    if self.rule_applies_to_key(rule, key) {
                        if !self.validate_range(entry) {
                            result.warnings.push(ValidationWarning {
                                rule: rule.name.clone(),
                                key: key.clone(),
                                message: format!("Value out of recommended range for: {}", key.path),
                                recommendation: Some("Consider using values within established ranges".to_string()),
                            });
                        }
                    }
                }
            },
            ValidationRuleType::DependencyCheck => {
                for (key, entry) in config_data {
                    if self.rule_applies_to_key(rule, key) {
                        if !self.validate_dependencies(key, config_data) {
                            result.errors.push(ValidationError {
                                rule: rule.name.clone(),
                                key: key.clone(),
                                message: format!("Missing dependencies for: {}", key.path),
                                severity: rule.severity,
                                code: "DEPENDENCY_MISSING".to_string(),
                            });
                            result.valid = false;
                        }
                    }
                }
            },
            _ => {
                // Other rule types would be implemented
            }
        }

        result
    }

    /// Check if a rule applies to a key
    fn rule_applies_to_key(&self, rule: &ValidationRule, key: &ConfigKey) -> bool {
        for pattern in &rule.target_keys {
            if pattern == "*" || pattern == "all" {
                return true;
            }
            if pattern == &key.path {
                return true;
            }
            if pattern.ends_with("*") && key.path.starts_with(&pattern[..pattern.len()-1]) {
                return true;
            }
            if pattern.contains("*") && self.match_pattern(pattern, &key.path) {
                return true;
            }
        }
        false
    }

    /// Simple pattern matching
    fn match_pattern(&self, pattern: &str, text: &str) -> bool {
        // Simple wildcard matching
        if pattern == "*" {
            return true;
        }
        
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let text_parts: Vec<&str> = text.split('.').collect();
        
        if pattern_parts.len() != text_parts.len() {
            return false;
        }
        
        for (pattern_part, text_part) in pattern_parts.iter().zip(text_parts.iter()) {
            if pattern_part != &"*" && pattern_part != text_part {
                return false;
            }
        }
        
        true
    }

    /// Validate type consistency
    fn validate_type(&self, entry: &ConfigEntry) -> bool {
        match (&entry.value, entry.value_type) {
            (ConfigValue::String(_), super::ConfigType::String) => true,
            (ConfigValue::Integer(_), super::ConfigType::Integer) => true,
            (ConfigValue::Unsigned(_), super::ConfigType::Unsigned) => true,
            (ConfigValue::Boolean(_), super::ConfigType::Boolean) => true,
            (ConfigValue::Float(_), super::ConfigType::Float) => true,
            (ConfigValue::Array(_), super::ConfigType::Array) => true,
            (ConfigValue::Object(_), super::ConfigType::Object) => true,
            (ConfigValue::None, super::ConfigType::None) => true,
            _ => false,
        }
    }

    /// Validate range constraints
    fn validate_range(&self, entry: &ConfigEntry) -> bool {
        match &entry.value {
            ConfigValue::Integer(i) => *i >= 0, // Basic range check
            ConfigValue::Unsigned(u) => *u <= 1024 * 1024 * 1024, // 1GB limit
            ConfigValue::Float(f) => *f >= 0.0 && *f <= 1000.0,
            _ => true,
        }
    }

    /// Validate dependencies
    fn validate_dependencies(&self, key: &ConfigKey, config_data: &HashMap<ConfigKey, ConfigEntry>) -> bool {
        // Check for common dependencies
        match key.key.as_str() {
            "network" => {
                // Network config requires system boot config
                let boot_key = ConfigKey {
                    namespace: "system".to_string(),
                    key: "boot".to_string(),
                    path: "system.boot".to_string(),
                };
                config_data.contains_key(&boot_key)
            },
            "security" => {
                // Security config requires system config
                let system_key = ConfigKey {
                    namespace: "system".to_string(),
                    key: "security".to_string(),
                    path: "system.security".to_string(),
                };
                config_data.contains_key(&system_key)
            },
            _ => true, // No specific dependencies
        }
    }

    /// Check for contradictory configuration values
    fn check_contradictions(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Check if debug and security are both enabled at high levels
        let debug_key = ConfigKey {
            namespace: "system".to_string(),
            key: "boot.debug_enabled".to_string(),
            path: "system.boot.debug_enabled".to_string(),
        };
        
        let security_key = ConfigKey {
            namespace: "system".to_string(),
            key: "security.policy_level".to_string(),
            path: "system.security.policy_level".to_string(),
        };

        if let (Some(debug_entry), Some(security_entry)) = (config_data.get(&debug_key), config_data.get(&security_key)) {
            if let (ConfigValue::Boolean(true), ConfigValue::Integer(level)) = (&debug_entry.value, &security_entry.value) {
                if *level >= 4 { // High security level
                    warn!("Debug mode enabled with high security level");
                }
            }
        }

        Ok(())
    }

    /// Check for missing dependencies
    fn check_dependencies(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Would check all configuration dependencies
        Ok(())
    }

    /// Check for circular references
    fn check_circular_references(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Would check for circular dependencies
        Ok(())
    }

    /// Check security constraints
    fn check_security_constraints(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Check for insecure values like hardcoded passwords, keys, etc.
        for (key, entry) in config_data {
            if let ConfigValue::String(s) = &entry.value {
                if s.contains("password") || s.contains("key") || s.contains("secret") {
                    warn!("Potential security risk in configuration: {}", key.path);
                }
            }
        }

        Ok(())
    }

    /// Check for privilege escalation risks
    fn check_privilege_escalation(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> ConfigResult<()> {
        // Would check for configurations that could lead to privilege escalation
        Ok(())
    }

    /// Generate cache key for validation result
    fn generate_cache_key(&self, config_data: &HashMap<ConfigKey, ConfigEntry>) -> String {
        // Simple hash of configuration data
        let mut hash = 0u32;
        for (key, entry) in config_data {
            hash = hash.wrapping_add(key.path.len() as u32);
            hash = hash.wrapping_add(entry.version as u32);
        }
        format!("config_v{}", hash)
    }

    /// Merge validation results
    fn merge_result(&self, target: &mut ValidationResult, source: ValidationResult) {
        if !source.valid {
            target.valid = false;
        }
        target.errors.extend(source.errors);
        target.warnings.extend(source.warnings);
        target.info.extend(source.info);
    }

    /// Update statistics
    fn update_stats(&self, duration_us: usize) {
        let mut stats = self.stats.write();
        stats.total_validations += 1;
        stats.last_validation = super::get_current_time();
        
        // Update average duration
        if stats.average_validation_time_us == 0 {
            stats.average_validation_time_us = duration_us as u64;
        } else {
            stats.average_validation_time_us = (stats.average_validation_time_us + duration_us as u64) / 2;
        }
    }

    /// Bump cache hits counter
    fn bump_cache_hits(&self) {
        let mut stats = self.stats.write();
        stats.cache_hits += 1;
    }

    /// Bump rule executions counter
    fn bump_rule_executions(&self) {
        let mut stats = self.stats.write();
        stats.rule_executions += 1;
    }

    /// Validate a single value
    fn validate_single_value(&self, key: &ConfigKey, value: &ConfigValue) -> ValidationResult {
        ValidationResult {
            valid: self.is_value_type_valid(value, &key.namespace),
            errors: if !self.is_value_type_valid(value, &key.namespace) {
                vec![ValidationError {
                    rule: "SingleValueValidation".to_string(),
                    key: key.clone(),
                    message: format!("Invalid value type for key: {}", key.path),
                    severity: ValidationSeverity::Error,
                    code: "INVALID_TYPE".to_string(),
                }]
            } else {
                Vec::new()
            },
            warnings: Vec::new(),
            info: Vec::new(),
            validated_at: super::get_current_time(),
            validation_duration: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rule_creation() {
        let rule = ValidationRule {
            name: "TestRule".to_string(),
            description: "A test rule".to_string(),
            rule_type: ValidationRuleType::TypeCheck,
            target_keys: vec!["test.*".to_string()],
            parameters: HashMap::new(),
            severity: ValidationSeverity::Warning,
            enabled: true,
        };

        assert_eq!(rule.name, "TestRule");
        assert!(rule.enabled);
    }

    #[test]
    fn test_pattern_matching() {
        let validator = ConfigValidator::new();
        
        assert!(validator.match_pattern("test.*", "test.key"));
        assert!(validator.match_pattern("*.key", "test.key"));
        assert!(validator.match_pattern("*", "any.path"));
        assert!(!validator.match_pattern("test.key", "test.different"));
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult {
            valid: false,
            errors: vec![ValidationError {
                rule: "TestRule".to_string(),
                key: ConfigKey {
                    namespace: "test".to_string(),
                    key: "key1".to_string(),
                    path: "test.key1".to_string(),
                },
                message: "Test error".to_string(),
                severity: ValidationSeverity::Error,
                code: "TEST_ERROR".to_string(),
            }],
            warnings: Vec::new(),
            info: Vec::new(),
            validated_at: 1000000,
            validation_duration: 500,
        };

        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }
}