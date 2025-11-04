//! Configuration Propagation System
//! 
//! This module handles configuration propagation and synchronization
//! across system services, ensuring consistent configuration state
//! and efficient change distribution.

#![no_std]

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use spin::{Mutex, RwLock};
use core::sync::atomic::{AtomicU64, Ordering};

use super::{ConfigKey, ConfigValue, ConfigResult, ConfigError};

/// Configuration propagation target
#[derive(Debug, Clone)]
pub struct PropagationTarget {
    pub service_id: u64,
    pub service_name: String,
    pub target_type: TargetType,
    pub priority: u32,
    pub retry_policy: RetryPolicy,
    pub filters: PropagationFilters,
    pub enabled: bool,
}

/// Target types for configuration propagation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    SystemService = 0,
    UserService = 1,
    ServiceGroup = 2,
    ExternalProcess = 3,
    NetworkService = 4,
    AllServices = 5,
}

/// Retry policy for failed propagations
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub backoff_multiplier: f32,
    pub max_delay_ms: u64,
    pub retry_on_timeout: bool,
    pub retry_on_error: bool,
}

/// Propagation filters
#[derive(Debug, Clone)]
pub struct PropagationFilters {
    pub namespace_filter: Option<String>,
    pub key_patterns: Option<Vec<String>>,
    pub exclude_keys: Option<Vec<String>>,
    pub conditional_filters: Vec<ConditionalFilter>,
}

/// Conditional filter for propagation
#[derive(Debug, Clone)]
pub struct ConditionalFilter {
    pub condition: FilterCondition,
    pub action: FilterAction,
    pub description: String,
}

/// Filter conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterCondition {
    Equals = 0,
    NotEquals = 1,
    GreaterThan = 2,
    LessThan = 3,
    Contains = 4,
    RegexMatch = 5,
}

/// Filter actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    Include = 0,
    Exclude = 1,
    Transform = 2,
    Log = 3,
}

/// Configuration propagation request
#[derive(Debug, Clone)]
pub struct PropagationRequest {
    pub request_id: u64,
    pub config_key: ConfigKey,
    pub config_value: ConfigValue,
    pub operation: PropagationOperation,
    pub source: String,
    pub timestamp: u64,
    pub priority: RequestPriority,
    pub dependencies: Vec<u64>,
}

/// Propagation operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropagationOperation {
    Create = 0,
    Update = 1,
    Delete = 2,
    Batch = 3,
}

/// Request priorities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Propagation result
#[derive(Debug, Clone)]
pub struct PropagationResult {
    pub request_id: u64,
    pub target_id: u64,
    pub success: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub timestamp: u64,
}

/// Propagation statistics
#[derive(Debug, Clone)]
pub struct PropagationStats {
    pub total_requests: usize,
    pub successful_propagations: usize,
    pub failed_propagations: usize,
    pub retries: usize,
    pub average_response_time_ms: f64,
    pub pending_requests: usize,
    pub active_targets: usize,
    pub last_propagation: u64,
    pub propagation_rate_per_second: f64,
}

/// Configuration propagator
pub struct ConfigPropagator {
    targets: RwLock<HashMap<u64, PropagationTarget>>,
    request_queue: Mutex<Vec<PropagationRequest>>,
    pending_requests: RwLock<HashMap<u64, PropagationRequest>>,
    completed_results: RwLock<Vec<PropagationResult>>,
    next_target_id: AtomicU64,
    next_request_id: AtomicU64,
    stats: RwLock<PropagationStats>,
}

/// Propagation configuration
#[derive(Debug, Clone)]
pub struct PropagationConfig {
    pub max_concurrent_requests: usize,
    pub default_retry_policy: RetryPolicy,
    pub batch_size: usize,
    pub timeout_ms: u64,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub enable_prioritization: bool,
    pub enable_dependencies: bool,
}

impl ConfigPropagator {
    /// Create a new configuration propagator
    pub fn new() -> Self {
        ConfigPropagator {
            targets: RwLock::new(HashMap::new()),
            request_queue: Mutex::new(Vec::new()),
            pending_requests: RwLock::new(HashMap::new()),
            completed_results: RwLock::new(Vec::new()),
            next_target_id: AtomicU64::new(1),
            next_request_id: AtomicU64::new(1),
            stats: RwLock::new(PropagationStats {
                total_requests: 0,
                successful_propagations: 0,
                failed_propagations: 0,
                retries: 0,
                average_response_time_ms: 0.0,
                pending_requests: 0,
                active_targets: 0,
                last_propagation: 0,
                propagation_rate_per_second: 0.0,
            }),
        }
    }

    /// Initialize the propagator
    pub fn init(&self) -> ConfigResult<()> {
        // Load default propagation targets
        self.load_default_targets()?;
        
        // Set up request processing
        self.start_request_processor()?;

        info!("Configuration propagator initialized");
        Ok(())
    }

    /// Register a new propagation target
    pub fn register_target(&self, target: PropagationTarget) -> ConfigResult<u64> {
        let target_id = self.next_target_id.fetch_add(1, Ordering::SeqCst);
        
        let mut targets = self.targets.write();
        targets.insert(target_id, target);
        
        self.update_stats();
        
        info!("Propagation target registered: {} (ID: {})", target.service_name, target_id);
        Ok(target_id)
    }

    /// Unregister a propagation target
    pub fn unregister_target(&self, target_id: u64) -> ConfigResult<()> {
        let mut targets = self.targets.write();
        
        if targets.remove(&target_id).is_some() {
            self.update_stats();
            info!("Propagation target unregistered: {}", target_id);
            Ok(())
        } else {
            Err(ConfigError::NotFound)
        }
    }

    /// Propagate configuration to a specific key
    pub fn propagate_config(&self, config_key: &ConfigKey) -> ConfigResult<()> {
        // This would be called when a configuration changes
        // For now, just a placeholder implementation
        
        info!("Propagating configuration for key: {}", config_key.path);
        Ok(())
    }

    /// Propagate configuration to all matching targets
    pub fn propagate_all_configs(&self, config_data: &HashMap<ConfigKey, super::ConfigEntry>) -> ConfigResult<()> {
        let targets = self.targets.read();
        let mut propagated_keys = Vec::new();
        
        for (target_id, target) in targets.iter() {
            if !target.enabled {
                continue;
            }
            
            // Check if target should receive this configuration
            let matching_keys = self.find_matching_keys(target, config_data);
            
            for key in matching_keys {
                if let Some(entry) = config_data.get(&key) {
                    self.queue_propagation_request(*target_id, &key, &entry.value, 
                                                  PropagationOperation::Update)?;
                    propagated_keys.push(key.path.clone());
                }
            }
        }
        
        info!("Queued propagation for {} configuration keys", propagated_keys.len());
        Ok(())
    }

    /// Send configuration to specific target
    pub fn send_to_target(&self, target_id: u64, config_key: &ConfigKey, 
                         config_value: &ConfigValue) -> ConfigResult<()> {
        let targets = self.targets.read();
        let target = targets.get(&target_id)
            .ok_or(ConfigError::NotFound)?;
        
        if !target.enabled {
            return Ok(());
        }
        
        // Check filters
        if !self.target_receives_config(target, config_key, config_value) {
            info!("Configuration filtered out for target {}: {}", target_id, config_key.path);
            return Ok(());
        }
        
        // Send configuration to target
        self.send_config_to_service(target, config_key, config_value)?;
        
        info!("Configuration sent to target {}: {} = {:?}", target_id, config_key.path, config_value);
        Ok(())
    }

    /// Batch propagate multiple configurations
    pub fn batch_propagate(&self, configs: &[(ConfigKey, ConfigValue)], 
                          target_ids: Option<Vec<u64>>) -> ConfigResult<Vec<u64>> {
        let mut request_ids = Vec::new();
        
        let targets = if let Some(ids) = target_ids {
            ids
        } else {
            // Send to all enabled targets
            self.targets.read().keys().cloned().collect()
        };
        
        for target_id in targets {
            let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
            let request = PropagationRequest {
                request_id,
                config_key: configs[0].0.clone(), // For batch, we'd have a different structure
                config_value: configs[0].1.clone(),
                operation: PropagationOperation::Batch,
                source: "config_manager".to_string(),
                timestamp: super::get_current_time(),
                priority: RequestPriority::Normal,
                dependencies: Vec::new(),
            };
            
            let mut queue = self.request_queue.lock();
            queue.push(request);
            request_ids.push(request_id);
        }
        
        info!("Batch propagation queued: {} requests", request_ids.len());
        Ok(request_ids)
    }

    /// Get propagation status
    pub fn get_propagation_status(&self) -> PropagationStats {
        self.stats.read().clone()
    }

    /// Get all registered targets
    pub fn get_targets(&self) -> Vec<(u64, PropagationTarget)> {
        let targets = self.targets.read();
        targets.iter()
            .map(|(id, target)| (*id, target.clone()))
            .collect()
    }

    /// Update target configuration
    pub fn update_target(&self, target_id: u64, updated_target: PropagationTarget) -> ConfigResult<()> {
        let mut targets = self.targets.write();
        
        if targets.contains_key(&target_id) {
            targets.insert(target_id, updated_target);
            info!("Target updated: {}", target_id);
            Ok(())
        } else {
            Err(ConfigError::NotFound)
        }
    }

    /// Enable/disable a target
    pub fn set_target_enabled(&self, target_id: u64, enabled: bool) -> ConfigResult<()> {
        let mut targets = self.targets.write();
        
        if let Some(target) = targets.get_mut(&target_id) {
            target.enabled = enabled;
            self.update_stats();
            info!("Target {} {}", target_id, if enabled { "enabled" } else { "disabled" });
            Ok(())
        } else {
            Err(ConfigError::NotFound)
        }
    }

    /// Get pending propagation requests
    pub fn get_pending_requests(&self) -> Vec<PropagationRequest> {
        let pending = self.pending_requests.read();
        pending.values().cloned().collect()
    }

    /// Clear completed results
    pub fn clear_completed_results(&self) -> ConfigResult<()> {
        let mut results = self.completed_results.write();
        results.clear();
        Ok(())
    }

    /// Load default propagation targets
    fn load_default_targets(&self) -> ConfigResult<()> {
        // System service target
        let system_target = PropagationTarget {
            service_id: 0, // Special ID for system
            service_name: "system".to_string(),
            target_type: TargetType::SystemService,
            priority: 1,
            retry_policy: RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 1000,
                backoff_multiplier: 2.0,
                max_delay_ms: 30000,
                retry_on_timeout: true,
                retry_on_error: true,
            },
            filters: PropagationFilters {
                namespace_filter: Some("system".to_string()),
                key_patterns: Some(vec!["*".to_string()]),
                exclude_keys: None,
                conditional_filters: Vec::new(),
            },
            enabled: true,
        };

        // Service manager target
        let service_manager_target = PropagationTarget {
            service_id: 1, // Service manager ID
            service_name: "service_manager".to_string(),
            target_type: TargetType::SystemService,
            priority: 2,
            retry_policy: RetryPolicy {
                max_retries: 5,
                initial_delay_ms: 500,
                backoff_multiplier: 1.5,
                max_delay_ms: 10000,
                retry_on_timeout: true,
                retry_on_error: true,
            },
            filters: PropagationFilters {
                namespace_filter: Some("service".to_string()),
                key_patterns: Some(vec!["*".to_string()]),
                exclude_keys: None,
                conditional_filters: Vec::new(),
            },
            enabled: true,
        };

        self.register_target(system_target)?;
        self.register_target(service_manager_target)?;

        info!("Default propagation targets loaded");
        Ok(())
    }

    /// Start the request processor
    fn start_request_processor(&self) -> ConfigResult<()> {
        // Would start background processing thread
        // For now, just a placeholder
        info!("Request processor started");
        Ok(())
    }

    /// Find keys that match a target's filters
    fn find_matching_keys(&self, target: &PropagationTarget, 
                         config_data: &HashMap<ConfigKey, super::ConfigEntry>) -> Vec<ConfigKey> {
        let mut matching_keys = Vec::new();
        
        for key in config_data.keys() {
            if self.key_matches_filters(target, key) {
                matching_keys.push(key.clone());
            }
        }
        
        matching_keys
    }

    /// Check if a key matches a target's filters
    fn key_matches_filters(&self, target: &PropagationTarget, key: &ConfigKey) -> bool {
        // Check namespace filter
        if let Some(ref namespace_filter) = target.filters.namespace_filter {
            if key.namespace != *namespace_filter {
                return false;
            }
        }
        
        // Check key patterns
        if let Some(ref patterns) = target.filters.key_patterns {
            let mut matches = false;
            for pattern in patterns {
                if pattern == "*" || pattern == &key.path || 
                   (pattern.ends_with("*") && key.path.starts_with(&pattern[..pattern.len()-1])) {
                    matches = true;
                    break;
                }
            }
            if !matches {
                return false;
            }
        }
        
        // Check exclude keys
        if let Some(ref exclude_keys) = target.filters.exclude_keys {
            for exclude_pattern in exclude_keys {
                if exclude_pattern == &key.path || 
                   (exclude_pattern.ends_with("*") && key.path.starts_with(&exclude_pattern[..exclude_pattern.len()-1])) {
                    return false;
                }
            }
        }
        
        // Check conditional filters
        for filter in &target.filters.conditional_filters {
            if !self.evaluate_conditional_filter(filter, key) {
                return false;
            }
        }
        
        true
    }

    /// Check if target should receive a specific configuration
    fn target_receives_config(&self, target: &PropagationTarget, 
                             key: &ConfigKey, value: &ConfigValue) -> bool {
        self.key_matches_filters(target, key)
    }

    /// Evaluate conditional filter
    fn evaluate_conditional_filter(&self, filter: &ConditionalFilter, key: &ConfigKey) -> bool {
        // Simplified conditional filter evaluation
        // In a real implementation, would evaluate the condition against the configuration
        match filter.condition {
            FilterCondition::Contains => {
                // Check if key contains certain string
                key.path.contains("critical")
            },
            _ => true, // Simplified for other conditions
        }
    }

    /// Queue a propagation request
    fn queue_propagation_request(&self, target_id: u64, key: &ConfigKey, 
                                value: &ConfigValue, operation: PropagationOperation) -> ConfigResult<()> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::SeqCst);
        
        let request = PropagationRequest {
            request_id,
            config_key: key.clone(),
            config_value: value.clone(),
            operation,
            source: "config_manager".to_string(),
            timestamp: super::get_current_time(),
            priority: RequestPriority::Normal,
            dependencies: Vec::new(),
        };
        
        let mut queue = self.request_queue.lock();
        queue.push(request);
        
        let mut pending = self.pending_requests.write();
        pending.insert(request_id, request);
        
        self.update_stats();
        
        info!("Propagation request queued: {} (ID: {}) for target {}", key.path, request_id, target_id);
        Ok(())
    }

    /// Send configuration to a service
    fn send_config_to_service(&self, target: &PropagationTarget, key: &ConfigKey, 
                             value: &ConfigValue) -> ConfigResult<()> {
        // In a real implementation, would send configuration to the service
        // This could involve IPC, network calls, or direct function calls
        
        match target.target_type {
            TargetType::SystemService => {
                // Send to system service via internal mechanisms
                info!("Sending config to system service: {} = {:?}", key.path, value);
            },
            TargetType::UserService => {
                // Send to user service via IPC
                info!("Sending config to user service: {} = {:?}", key.path, value);
            },
            TargetType::NetworkService => {
                // Send to network service
                info!("Sending config to network service: {} = {:?}", key.path, value);
            },
            _ => {
                // Handle other target types
                info!("Sending config to target {}: {} = {:?}", target.service_id, key.path, value);
            }
        }
        
        Ok(())
    }

    /// Update statistics
    fn update_stats(&self) {
        let mut stats = self.stats.write();
        let targets = self.targets.read();
        
        stats.active_targets = targets.values().filter(|t| t.enabled).count();
        stats.pending_requests = self.pending_requests.read().len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_target_creation() {
        let target = PropagationTarget {
            service_id: 123,
            service_name: "test_service".to_string(),
            target_type: TargetType::SystemService,
            priority: 1,
            retry_policy: RetryPolicy {
                max_retries: 3,
                initial_delay_ms: 1000,
                backoff_multiplier: 2.0,
                max_delay_ms: 30000,
                retry_on_timeout: true,
                retry_on_error: true,
            },
            filters: PropagationFilters {
                namespace_filter: Some("test".to_string()),
                key_patterns: Some(vec!["*".to_string()]),
                exclude_keys: None,
                conditional_filters: Vec::new(),
            },
            enabled: true,
        };

        assert_eq!(target.service_id, 123);
        assert_eq!(target.target_type, TargetType::SystemService);
        assert!(target.enabled);
    }

    #[test]
    fn test_propagation_request_creation() {
        let key = ConfigKey {
            namespace: "test".to_string(),
            key: "key1".to_string(),
            path: "test.key1".to_string(),
        };

        let request = PropagationRequest {
            request_id: 1,
            config_key: key,
            config_value: ConfigValue::String("value1".to_string()),
            operation: PropagationOperation::Update,
            source: "test".to_string(),
            timestamp: 1000000,
            priority: RequestPriority::Normal,
            dependencies: Vec::new(),
        };

        assert_eq!(request.request_id, 1);
        assert_eq!(request.operation, PropagationOperation::Update);
    }

    #[test]
    fn test_retry_policy_creation() {
        let policy = RetryPolicy {
            max_retries: 5,
            initial_delay_ms: 500,
            backoff_multiplier: 2.0,
            max_delay_ms: 10000,
            retry_on_timeout: true,
            retry_on_error: true,
        };

        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.initial_delay_ms, 500);
        assert!(policy.retry_on_timeout);
    }
}