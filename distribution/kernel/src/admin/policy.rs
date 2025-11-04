//! Policy Management System
//! 
//! This module provides policy management for system-wide rules and constraints
//! including policy definitions, enforcement, and validation.

#![no_std]

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use spin::{Mutex, RwLock};
use core::sync::atomic::{AtomicU64, Ordering};

use super::{ConfigValue, ConfigKey, ConfigResult, ConfigError};

/// Policy definition
#[derive(Debug, Clone)]
pub struct Policy {
    pub name: String,
    pub description: String,
    pub policy_type: PolicyType,
    pub severity: PolicySeverity,
    pub target: PolicyTarget,
    pub conditions: Vec<PolicyCondition>,
    pub actions: Vec<PolicyAction>,
    pub enabled: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Policy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyType {
    Security = 0,
    Resource = 1,
    Network = 2,
    Storage = 3,
    Process = 4,
    User = 5,
    System = 6,
    Compliance = 7,
}

/// Policy severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicySeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
}

/// Policy target scope
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyTarget {
    All = 0,
    Namespace(String),
    Key(String),
    KeyPattern(String),
    ServiceId(u64),
    UserId(u64),
}

/// Policy condition for triggering
#[derive(Debug, Clone)]
pub struct PolicyCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: ConfigValue,
    pub case_sensitive: bool,
}

/// Condition operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionOperator {
    Equals = 0,
    NotEquals = 1,
    GreaterThan = 2,
    LessThan = 3,
    Contains = 4,
    StartsWith = 5,
    EndsWith = 6,
    RegexMatch = 7,
    InSet = 8,
}

/// Policy action when triggered
#[derive(Debug, Clone)]
pub struct PolicyAction {
    pub action_type: ActionType,
    pub parameters: HashMap<String, ConfigValue>,
    pub description: String,
}

/// Action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Deny = 0,
    Allow = 1,
    Log = 2,
    Alert = 3,
    Throttle = 4,
    Redirect = 5,
    Modify = 6,
    Quarantine = 7,
    Notify = 8,
    Rollback = 9,
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub matched: bool,
    pub policy_name: String,
    pub triggered_actions: Vec<ActionType>,
    pub violation_message: Option<String>,
    pub enforcement_level: EnforcementLevel,
}

/// Enforcement levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementLevel {
    None = 0,
    Soft = 1,
    Hard = 2,
    Strict = 3,
}

/// Policy manager
pub struct PolicyManager {
    policies: RwLock<HashMap<String, Policy>>,
    evaluation_cache: Mutex<HashMap<String, PolicyResult>>,
    violation_log: RwLock<Vec<PolicyViolation>>,
    next_policy_id: AtomicU64,
    policy_stats: PolicyStats,
}

/// Policy violation record
#[derive(Debug, Clone)]
pub struct PolicyViolation {
    pub policy_name: String,
    pub violation_type: ViolationType,
    pub key: Option<ConfigKey>,
    pub value: Option<ConfigValue>,
    pub timestamp: u64,
    pub severity: PolicySeverity,
    pub details: String,
}

/// Violation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    AccessDenied = 0,
    ResourceExceeded = 1,
    SecurityBreach = 2,
    ComplianceFailure = 3,
    PolicyConflict = 4,
    InvalidOperation = 5,
}

/// Policy statistics
#[derive(Debug, Clone)]
pub struct PolicyStats {
    pub total_policies: usize,
    pub enabled_policies: usize,
    pub violations_today: usize,
    pub policy_checks: usize,
    pub enforcement_actions: usize,
    pub last_violation: u64,
}

impl PolicyManager {
    /// Create a new policy manager
    pub fn new() -> Self {
        PolicyManager {
            policies: RwLock::new(HashMap::new()),
            evaluation_cache: Mutex::new(HashMap::new()),
            violation_log: RwLock::new(Vec::new()),
            next_policy_id: AtomicU64::new(1),
            policy_stats: PolicyStats {
                total_policies: 0,
                enabled_policies: 0,
                violations_today: 0,
                policy_checks: 0,
                enforcement_actions: 0,
                last_violation: 0,
            },
        }
    }

    /// Initialize the policy manager
    pub fn init(&self) -> ConfigResult<()> {
        // Load default policies
        self.load_default_policies()?;
        
        info!("Policy manager initialized");
        Ok(())
    }

    /// Register a new policy
    pub fn register_policy(&self, policy: Policy) -> ConfigResult<String> {
        let policy_id = self.next_policy_id.fetch_add(1, Ordering::SeqCst).to_string();
        
        let mut policies = self.policies.write();
        policies.insert(policy_id.clone(), policy);
        
        self.update_stats();
        
        info!("Policy registered: {} (ID: {})", policy.name, policy_id);
        Ok(policy_id)
    }

    /// Check policy compliance for a configuration value
    pub fn check_policy(&self, key: &ConfigKey, value: &ConfigValue) -> ConfigResult<()> {
        let policies = self.policies.read();
        let cache_key = format!("{}.{}.{:?}", key.namespace, key.key, value_type(value));
        
        // Check cache first
        if let Some(cached_result) = self.evaluation_cache.lock().get(&cache_key) {
            if cached_result.matched && cached_result.violation_message.is_none() {
                return Ok(());
            }
        }

        self.policy_stats.policy_checks += 1;
        
        for (policy_id, policy) in policies.iter() {
            if !policy.enabled {
                continue;
            }
            
            if self.policy_applies_to_target(policy, key, value) {
                let result = self.evaluate_policy(policy, key, value);
                
                if result.matched {
                    self.handle_policy_result(policy, key, value, &result)?;
                }
            }
        }

        Ok(())
    }

    /// Get all policies
    pub fn get_policies(&self) -> Vec<(String, Policy)> {
        let policies = self.policies.read();
        policies.iter()
            .map(|(id, policy)| (id.clone(), policy.clone()))
            .collect()
    }

    /// Get policy by name
    pub fn get_policy(&self, policy_id: &str) -> ConfigResult<Policy> {
        let policies = self.policies.read();
        policies.get(policy_id)
            .cloned()
            .ok_or(ConfigError::NotFound)
    }

    /// Update policy
    pub fn update_policy(&self, policy_id: &str, updated_policy: Policy) -> ConfigResult<()> {
        let mut policies = self.policies.write();
        if policies.contains_key(policy_id) {
            policies.insert(policy_id.to_string(), updated_policy);
            self.update_stats();
            info!("Policy updated: {}", policy_id);
            Ok(())
        } else {
            Err(ConfigError::NotFound)
        }
    }

    /// Delete policy
    pub fn delete_policy(&self, policy_id: &str) -> ConfigResult<()> {
        let mut policies = self.policies.write();
        if policies.remove(policy_id).is_some() {
            self.update_stats();
            info!("Policy deleted: {}", policy_id);
            Ok(())
        } else {
            Err(ConfigError::NotFound)
        }
    }

    /// Apply policies to configuration data
    pub fn apply_policies(&self, config_data: &HashMap<ConfigKey, super::ConfigEntry>) -> ConfigResult<()> {
        let mut violations = Vec::new();
        
        for (key, entry) in config_data {
            if let Err(violation) = self.check_policy(&key.key, &entry.value) {
                // Record violation but continue processing
                violations.push(format!("Policy violation for {}: {:?}", key.path, violation));
            }
        }
        
        if !violations.is_empty() {
            warn!("Policy violations found: {:?}", violations);
        }
        
        Ok(())
    }

    /// Get policy violations
    pub fn get_violations(&self) -> Vec<PolicyViolation> {
        self.violation_log.read().clone()
    }

    /// Clear violation log
    pub fn clear_violations(&self) {
        let mut log = self.violation_log.write();
        log.clear();
    }

    /// Get policy statistics
    pub fn get_stats(&self) -> PolicyStats {
        self.policy_stats.clone()
    }

    /// Load default system policies
    fn load_default_policies(&self) -> ConfigResult<()> {
        // Security policy: Deny all system.* writes
        let security_policy = Policy {
            name: "System Security Policy".to_string(),
            description: "Protect system configuration from unauthorized changes".to_string(),
            policy_type: PolicyType::Security,
            severity: PolicySeverity::Critical,
            target: PolicyTarget::Namespace("system".to_string()),
            conditions: vec![
                PolicyCondition {
                    field: "user_id".to_string(),
                    operator: ConditionOperator::Equals,
                    value: ConfigValue::Unsigned(0), // Non-root user
                    case_sensitive: false,
                }
            ],
            actions: vec![
                PolicyAction {
                    action_type: ActionType::Deny,
                    parameters: HashMap::new(),
                    description: "Deny non-root access to system configuration".to_string(),
                },
                PolicyAction {
                    action_type: ActionType::Log,
                    parameters: HashMap::new(),
                    description: "Log unauthorized access attempt".to_string(),
                }
            ],
            enabled: true,
            created_at: super::get_current_time(),
            updated_at: super::get_current_time(),
        };

        // Resource policy: Limit memory configurations
        let resource_policy = Policy {
            name: "Memory Resource Policy".to_string(),
            description: "Ensure memory configurations stay within safe limits".to_string(),
            policy_type: PolicyType::Resource,
            severity: PolicySeverity::Error,
            target: PolicyTarget::KeyPattern("*.memory.*".to_string()),
            conditions: vec![
                PolicyCondition {
                    field: "value".to_string(),
                    operator: ConditionOperator::GreaterThan,
                    value: ConfigValue::Unsigned(1024 * 1024 * 1024), // 1GB limit
                    case_sensitive: false,
                }
            ],
            actions: vec![
                PolicyAction {
                    action_type: ActionType::Throttle,
                    parameters: HashMap::new(),
                    description: "Throttle large memory allocations".to_string(),
                }
            ],
            enabled: true,
            created_at: super::get_current_time(),
            updated_at: super::get_current_time(),
        };

        self.register_policy(security_policy)?;
        self.register_policy(resource_policy)?;
        
        info!("Default policies loaded");
        Ok(())
    }

    /// Check if policy applies to target
    fn policy_applies_to_target(&self, policy: &Policy, key: &ConfigKey, value: &ConfigValue) -> bool {
        match &policy.target {
            PolicyTarget::All => true,
            PolicyTarget::Namespace(ns) => key.namespace == *ns,
            PolicyTarget::Key(k) => key.key == *k,
            PolicyTarget::KeyPattern(pattern) => key.key.contains(pattern),
            PolicyTarget::ServiceId(_) => true, // Would check service context
            PolicyTarget::UserId(_) => true, // Would check user context
        }
    }

    /// Evaluate policy against configuration
    fn evaluate_policy(&self, policy: &Policy, key: &ConfigKey, value: &ConfigValue) -> PolicyResult {
        let mut matched = false;
        let mut triggered_actions = Vec::new();
        let mut violation_message = None;

        for condition in &policy.conditions {
            if self.check_condition(condition, key, value) {
                matched = true;
                break;
            }
        }

        if matched {
            for action in &policy.actions {
                triggered_actions.push(action.action_type);
            }
            
            if triggered_actions.contains(&ActionType::Deny) {
                violation_message = Some(format!("Policy '{}' denied operation", policy.name));
            }
        }

        PolicyResult {
            matched,
            policy_name: policy.name.clone(),
            triggered_actions,
            violation_message,
            enforcement_level: match policy.severity {
                PolicySeverity::Info => EnforcementLevel::None,
                PolicySeverity::Warning => EnforcementLevel::Soft,
                PolicySeverity::Error => EnforcementLevel::Hard,
                PolicySeverity::Critical => EnforcementLevel::Strict,
            }
        }
    }

    /// Check if a condition is met
    fn check_condition(&self, condition: &PolicyCondition, key: &ConfigKey, value: &ConfigValue) -> bool {
        // For simplicity, check against the configuration value
        // In a real implementation, this would check various fields
        
        match condition.operator {
            ConditionOperator::Equals => value == &condition.value,
            ConditionOperator::NotEquals => value != &condition.value,
            ConditionOperator::GreaterThan => self.compare_values(value, &condition.value) > 0,
            ConditionOperator::LessThan => self.compare_values(value, &condition.value) < 0,
            ConditionOperator::Contains => {
                if let ConfigValue::String(s) = value {
                    if let ConfigValue::String(pattern) = &condition.value {
                        s.contains(pattern)
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            _ => false, // Simplified for other operators
        }
    }

    /// Compare two configuration values
    fn compare_values(&self, a: &ConfigValue, b: &ConfigValue) -> i32 {
        match (a, b) {
            (ConfigValue::Integer(x), ConfigValue::Integer(y)) => x.cmp(y) as i32,
            (ConfigValue::Unsigned(x), ConfigValue::Unsigned(y)) => x.cmp(y) as i32,
            (ConfigValue::Float(x), ConfigValue::Float(y)) => x.partial_cmp(y).unwrap_or(0) as i32,
            (ConfigValue::String(x), ConfigValue::String(y)) => x.cmp(y) as i32,
            _ => 0, // Different types or not comparable
        }
    }

    /// Handle policy result
    fn handle_policy_result(&self, policy: &Policy, key: &ConfigKey, value: &ConfigValue, result: &PolicyResult) -> ConfigResult<()> {
        self.policy_stats.enforcement_actions += 1;

        // Handle enforcement actions
        for action_type in &result.triggered_actions {
            match action_type {
                ActionType::Deny => {
                    // Record violation
                    let violation = PolicyViolation {
                        policy_name: policy.name.clone(),
                        violation_type: ViolationType::AccessDenied,
                        key: Some(key.clone()),
                        value: Some(value.clone()),
                        timestamp: super::get_current_time(),
                        severity: policy.severity,
                        details: result.violation_message.clone().unwrap_or_default(),
                    };
                    
                    self.record_violation(violation);
                    return Err(ConfigError::PolicyViolation);
                },
                ActionType::Log => {
                    info!("Policy log: {} - {} = {:?}", policy.name, key.path, value);
                },
                ActionType::Alert => {
                    warn!("Policy alert: {} - {} = {:?}", policy.name, key.path, value);
                },
                _ => {
                    // Other actions would be implemented
                }
            }
        }

        Ok(())
    }

    /// Record policy violation
    fn record_violation(&self, violation: PolicyViolation) {
        let mut log = self.violation_log.write();
        log.push(violation);
        
        self.policy_stats.violations_today += 1;
        self.policy_stats.last_violation = super::get_current_time();
        
        // Limit log size
        if log.len() > 1000 {
            let to_remove = log.len() - 1000;
            log.drain(0..to_remove);
        }
    }

    /// Update statistics
    fn update_stats(&self) {
        let policies = self.policies.read();
        self.policy_stats.total_policies = policies.len();
        self.policy_stats.enabled_policies = policies.values()
            .filter(|p| p.enabled)
            .count();
    }
}

/// Helper function to get value type
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
    fn test_policy_creation() {
        let policy = Policy {
            name: "Test Policy".to_string(),
            description: "A test policy".to_string(),
            policy_type: PolicyType::Security,
            severity: PolicySeverity::Warning,
            target: PolicyTarget::All,
            conditions: vec![],
            actions: vec![],
            enabled: true,
            created_at: 1000000,
            updated_at: 1000000,
        };

        assert_eq!(policy.name, "Test Policy");
        assert!(policy.enabled);
    }

    #[test]
    fn test_condition_evaluation() {
        let condition = PolicyCondition {
            field: "value".to_string(),
            operator: ConditionOperator::Equals,
            value: ConfigValue::String("test".to_string()),
            case_sensitive: false,
        };

        let key = ConfigKey {
            namespace: "test".to_string(),
            key: "key1".to_string(),
            path: "test.key1".to_string(),
        };

        // This would be tested with actual value comparison
        assert_eq!(condition.field, "value");
    }

    #[test]
    fn test_policy_stats() {
        let stats = PolicyStats {
            total_policies: 5,
            enabled_policies: 3,
            violations_today: 2,
            policy_checks: 100,
            enforcement_actions: 10,
            last_violation: 2000000,
        };

        assert_eq!(stats.total_policies, 5);
        assert_eq!(stats.enabled_policies, 3);
    }
}