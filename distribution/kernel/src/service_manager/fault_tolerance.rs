//! Fault Tolerance and Recovery
//! 
//! This module provides fault detection, tolerance, and automatic recovery
//! mechanisms for services in the MultiOS service management framework.

use spin::{Mutex, RwLock};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque, HashSet};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::{ServiceId, ServiceResult, ServiceError, service::{HealthStatus, ServiceState}};
use super::service_manager::ServiceError as SMError;

/// Get current system time
fn get_current_time() -> u64 {
    super::get_current_time()
}

/// Fault Detector - Monitors services for faults and failures
pub struct FaultDetector {
    failure_tracker: RwLock<BTreeMap<ServiceId, FailureTracker>>,
    fault_patterns: RwLock<Vec<FaultPattern>>,
    detection_rules: RwLock<Vec<DetectionRule>>,
    fault_history: RwLock<VecDeque<FaultEvent>>,
    detection_stats: DetectionStats,
}

/// Recovery Manager - Manages automatic recovery and remediation
pub struct RecoveryManager {
    recovery_policies: RwLock<BTreeMap<ServiceId, RecoveryPolicy>>,
    recovery_actions: RwLock<Vec<RecoveryAction>>,
    recovery_history: RwLock<VecDeque<RecoveryEvent>>,
    recovery_stats: RecoveryStats,
    active_recoveries: RwLock<BTreeMap<ServiceId, ActiveRecovery>>,
}

/// Failure Tracker - Tracks failure patterns for services
#[derive(Debug, Clone)]
struct FailureTracker {
    service_id: ServiceId,
    failure_count: u32,
    consecutive_failures: u32,
    last_failure_time: Option<u64>,
    failure_pattern: FailurePattern,
    severity: FaultSeverity,
    detection_threshold: u32,
    recovery_threshold: u32,
}

/// Fault Patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FailurePattern {
    None = 0,
    Transient = 1,
    Intermittent = 2,
    Persistent = 3,
    Cascading = 4,
    GracefulDegradation = 5,
}

/// Fault Severity Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FaultSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
    Fatal = 4,
}

/// Fault Event
#[derive(Debug, Clone)]
struct FaultEvent {
    service_id: ServiceId,
    fault_type: FaultType,
    severity: FaultSeverity,
    detection_time: u64,
    resolved: bool,
    resolution_time: Option<u64>,
    error_details: Option<String>,
}

/// Fault Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FaultType {
    HealthCheckFailure = 0,
    ResourceExhaustion = 1,
    DependencyFailure = 2,
    ConfigurationError = 3,
    NetworkIssue = 4,
    SecurityViolation = 5,
    PerformanceDegradation = 6,
    UnhandledException = 7,
    Timeout = 8,
    CircuitBreakerOpen = 9,
}

/// Detection Rule
#[derive(Debug, Clone)]
struct DetectionRule {
    name: String,
    fault_type: FaultType,
    condition: DetectionCondition,
    threshold: u32,
    window_size: u32,
    enabled: bool,
}

/// Detection Condition
#[derive(Debug, Clone)]
pub enum DetectionCondition {
    FailureCountExceeds { threshold: u32 },
    ResponseTimeExceeds { threshold: u64 },
    MemoryUsageExceeds { threshold: f32 },
    CpuUsageExceeds { threshold: f32 },
    ConnectionCountExceeds { threshold: u32 },
    ErrorRateExceeds { threshold: f32 },
    Custom { expression: String },
}

/// Recovery Policy - Defines how to recover from different faults
#[derive(Debug, Clone)]
struct RecoveryPolicy {
    service_id: ServiceId,
    max_recovery_attempts: u32,
    recovery_strategy: RecoveryStrategy,
    backoff_strategy: BackoffStrategy,
    escalation_policy: EscalationPolicy,
    recovery_timeout: u32,
}

/// Recovery Strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecoveryStrategy {
    None = 0,
    Restart = 1,
    RestartWithDelay = 2,
    ScaleUp = 3,
    ScaleDown = 4,
    Failover = 5,
    CircuitBreakerReset = 6,
    ConfigurationReload = 7,
    ResourceCleanup = 8,
    DependencyRestart = 9,
    GracefulShutdown = 10,
}

/// Backoff Strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BackoffStrategy {
    None = 0,
    Linear { delay_ms: u32, increment_ms: u32 },
    Exponential { initial_delay_ms: u32, multiplier: f32, max_delay_ms: u32 },
    Fixed { delay_ms: u32 },
    Adaptive { initial_delay_ms: u32, max_delay_ms: u32 },
}

/// Escalation Policy
#[derive(Debug, Clone)]
struct EscalationPolicy {
    enabled: bool,
    escalation_levels: Vec<EscalationLevel>,
}

/// Escalation Level
#[derive(Debug, Clone)]
struct EscalationLevel {
    level: u32,
    threshold: u32,
    action: EscalationAction,
    delay: u32,
}

/// Escalation Actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EscalationAction {
    Log = 0,
    Notify = 1,
    Restart = 2,
    Stop = 3,
    Failover = 4,
    PageOnCall = 5,
}

/// Recovery Action - Executable recovery step
#[derive(Debug, Clone)]
struct RecoveryAction {
    id: String,
    service_id: ServiceId,
    action_type: RecoveryActionType,
    parameters: BTreeMap<String, String>,
    timeout: u32,
    retry_count: u32,
    max_retries: u32,
}

/// Recovery Action Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecoveryActionType {
    RestartService = 0,
    ScaleInstances = 1,
    ResetCircuitBreaker = 2,
    ReloadConfiguration = 3,
    CleanupResources = 4,
    RestartDependencies = 5,
    NetworkReconnect = 6,
    ResourceQuotaReset = 7,
    HealthCheckOverride = 8,
}

/// Recovery Event
#[derive(Debug, Clone)]
struct RecoveryEvent {
    service_id: ServiceId,
    action_id: String,
    action_type: RecoveryActionType,
    start_time: u64,
    end_time: Option<u64>,
    success: bool,
    error_message: Option<String>,
}

/// Active Recovery - Currently executing recovery
#[derive(Debug, Clone)]
struct ActiveRecovery {
    service_id: ServiceId,
    recovery_policy: RecoveryPolicy,
    current_attempt: u32,
    start_time: u64,
    active_actions: Vec<String>,
    backoff_timer: Option<u64>,
}

/// Detection Statistics
#[derive(Debug, Clone)]
struct DetectionStats {
    total_detections: u64,
    false_positives: u64,
    average_detection_time: f64,
    most_common_fault: Option<FaultType>,
    last_detection_time: Option<u64>,
}

/// Recovery Statistics
#[derive(Debug, Clone)]
struct RecoveryStats {
    total_recovery_attempts: u64,
    successful_recoveries: u64,
    failed_recoveries: u64,
    average_recovery_time: f64,
    escalations_triggered: u64,
    last_recovery_time: Option<u64>,
}

/// Fault Tolerance Configuration
#[derive(Debug, Clone)]
pub struct FaultToleranceConfig {
    pub auto_recovery_enabled: bool,
    pub max_recovery_attempts: u32,
    pub detection_sensitivity: DetectionSensitivity,
    pub recovery_timeout: u32,
    pub enable_escalation: bool,
    pub enable_circuit_breaker: bool,
}

/// Detection Sensitivity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DetectionSensitivity {
    Low = 0,
    Normal = 1,
    High = 2,
    Paranoid = 3,
}

/// Circuit Breaker - Prevents cascade failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    threshold: u32,
    timeout: u32,
    last_failure_time: Option<u64>,
    last_success_time: Option<u64>,
}

impl FaultDetector {
    /// Create a new fault detector
    pub fn new() -> Self {
        FaultDetector {
            failure_tracker: RwLock::new(BTreeMap::new()),
            fault_patterns: RwLock::new(Vec::new()),
            detection_rules: RwLock::new(Vec::new()),
            fault_history: RwLock::new(VecDeque::new()),
            detection_stats: DetectionStats {
                total_detections: 0,
                false_positives: 0,
                average_detection_time: 0.0,
                most_common_fault: None,
                last_detection_time: None,
            },
        }
    }

    /// Initialize the fault detector
    pub fn init(&self) -> ServiceResult<()> {
        // Set up default detection rules
        self.setup_default_rules()?;
        
        info!("Fault detector initialized");
        Ok(())
    }

    /// Detect faults for a service
    pub fn detect_fault(&self, service_id: ServiceId, error: &ServiceError) -> ServiceResult<FaultType> {
        let start_time = get_current_time();
        
        // Update failure tracker
        self.update_failure_tracker(service_id, error)?;
        
        // Analyze fault pattern
        let fault_type = self.analyze_fault_type(service_id, error)?;
        
        // Check detection rules
        let detected = self.check_detection_rules(service_id, &fault_type)?;
        
        if detected {
            // Record fault event
            self.record_fault_event(service_id, fault_type, FaultSeverity::Error)?;
            
            // Update statistics
            self.update_detection_stats(start_time, fault_type);
            
            info!("Fault detected for service {}: {:?}", service_id.0, fault_type);
        }
        
        Ok(fault_type)
    }

    /// Get failure status for a service
    pub fn get_failure_status(&self, service_id: ServiceId) -> Option<&FailureTracker> {
        let tracker = self.failure_tracker.read();
        tracker.get(&service_id)
    }

    /// Reset failure tracking for a service
    pub fn reset_failure_tracking(&self, service_id: ServiceId) -> ServiceResult<()> {
        let mut tracker = self.failure_tracker.write();
        tracker.remove(&service_id);
        
        info!("Failure tracking reset for service: {}", service_id.0);
        Ok(())
    }

    /// Get fault detection statistics
    pub fn get_stats(&self) -> &DetectionStats {
        &self.detection_stats
    }

    /// Add detection rule
    pub fn add_detection_rule(&self, rule: DetectionRule) -> ServiceResult<()> {
        let mut rules = self.detection_rules.write();
        rules.push(rule);
        
        info!("Detection rule added: {}", rule.name);
        Ok(())
    }

    /// Internal methods
    fn update_failure_tracker(&self, service_id: ServiceId, error: &ServiceError) -> ServiceResult<()> {
        let mut tracker = self.failure_tracker.write();
        
        let current_time = get_current_time();
        
        if let Some(existing_tracker) = tracker.get_mut(&service_id) {
            existing_tracker.failure_count += 1;
            existing_tracker.consecutive_failures += 1;
            existing_tracker.last_failure_time = Some(current_time);
            
            // Update failure pattern based on timing
            if let Some(last_failure) = existing_tracker.last_failure_time {
                let time_diff = current_time - last_failure;
                existing_tracker.failure_pattern = if time_diff < 5000 {
                    FailurePattern::Transient
                } else if time_diff < 30000 {
                    FailurePattern::Intermittent
                } else {
                    FailurePattern::Persistent
                };
            }
        } else {
            // Create new failure tracker
            tracker.insert(service_id, FailureTracker {
                service_id,
                failure_count: 1,
                consecutive_failures: 1,
                last_failure_time: Some(current_time),
                failure_pattern: FailurePattern::Transient,
                severity: FaultSeverity::Warning,
                detection_threshold: 3,
                recovery_threshold: 1,
            });
        }
        
        Ok(())
    }

    fn analyze_fault_type(&self, service_id: ServiceId, error: &ServiceError) -> ServiceResult<FaultType> {
        // Map service errors to fault types
        match error {
            ServiceError::HealthCheckFailed => Ok(FaultType::HealthCheckFailure),
            ServiceError::ServiceFailed => Ok(FaultType::UnhandledException),
            ServiceError::ConfigurationError => Ok(FaultType::ConfigurationError),
            ServiceError::ResourceExhausted => Ok(FaultType::ResourceExhaustion),
            ServiceError::ServiceTimeout => Ok(FaultType::Timeout),
            _ => Ok(FaultType::UnhandledException),
        }
    }

    fn check_detection_rules(&self, service_id: ServiceId, fault_type: &FaultType) -> ServiceResult<bool> {
        let rules = self.detection_rules.read();
        let tracker = self.failure_tracker.read();
        
        if let Some(service_tracker) = tracker.get(&service_id) {
            for rule in rules.iter() {
                if rule.fault_type == *fault_type && rule.enabled {
                    if self.matches_condition(service_tracker, &rule.condition)? {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn matches_condition(&self, tracker: &FailureTracker, condition: &DetectionCondition) -> ServiceResult<bool> {
        match condition {
            DetectionCondition::FailureCountExceeds { threshold } => {
                Ok(tracker.failure_count >= *threshold)
            }
            DetectionCondition::ResponseTimeExceeds { threshold: _ } => {
                // Would check actual response time metrics
                Ok(false)
            }
            DetectionCondition::MemoryUsageExceeds { threshold: _ } => {
                // Would check actual memory usage
                Ok(false)
            }
            DetectionCondition::CpuUsageExceeds { threshold: _ } => {
                // Would check actual CPU usage
                Ok(false)
            }
            DetectionCondition::ConnectionCountExceeds { threshold: _ } => {
                // Would check actual connection count
                Ok(false)
            }
            DetectionCondition::ErrorRateExceeds { threshold: _ } => {
                // Would check actual error rate
                Ok(false)
            }
            DetectionCondition::Custom { expression: _ } => {
                // Would evaluate custom expression
                Ok(false)
            }
        }
    }

    fn record_fault_event(&self, service_id: ServiceId, fault_type: FaultType, severity: FaultSeverity) -> ServiceResult<()> {
        let event = FaultEvent {
            service_id,
            fault_type,
            severity,
            detection_time: get_current_time(),
            resolved: false,
            resolution_time: None,
            error_details: None,
        };

        let mut history = self.fault_history.write();
        history.push_back(event);

        // Maintain history size
        while history.len() > 1000 {
            history.pop_front();
        }

        Ok(())
    }

    fn update_detection_stats(&self, start_time: u64, fault_type: FaultType) {
        let detection_time = get_current_time() - start_time;
        
        self.detection_stats.total_detections += 1;
        self.detection_stats.last_detection_time = Some(get_current_time());
        
        // Update running average
        let total_detections = self.detection_stats.total_detections as f64;
        let current_avg = self.detection_stats.average_detection_time;
        let new_avg = (current_avg * (total_detections - 1.0) + detection_time as f64) / total_detections;
        self.detection_stats.average_detection_time = new_avg;
        
        // Update most common fault (simplified)
        self.detection_stats.most_common_fault = Some(fault_type);
    }

    fn setup_default_rules(&self) -> ServiceResult<()> {
        let default_rules = vec![
            DetectionRule {
                name: "High Failure Count".to_string(),
                fault_type: FaultType::HealthCheckFailure,
                condition: DetectionCondition::FailureCountExceeds { threshold: 3 },
                threshold: 3,
                window_size: 300, // 5 minutes
                enabled: true,
            },
            DetectionRule {
                name: "Persistent Failure Pattern".to_string(),
                fault_type: FaultType::Persistent,
                condition: DetectionCondition::FailureCountExceeds { threshold: 5 },
                threshold: 5,
                window_size: 600, // 10 minutes
                enabled: true,
            },
        ];

        let mut rules = self.detection_rules.write();
        rules.extend(default_rules);

        Ok(())
    }
}

impl RecoveryManager {
    /// Create a new recovery manager
    pub fn new() -> Self {
        RecoveryManager {
            recovery_policies: RwLock::new(BTreeMap::new()),
            recovery_actions: RwLock::new(Vec::new()),
            recovery_history: RwLock::new(VecDeque::new()),
            recovery_stats: RecoveryStats {
                total_recovery_attempts: 0,
                successful_recoveries: 0,
                failed_recoveries: 0,
                average_recovery_time: 0.0,
                escalations_triggered: 0,
                last_recovery_time: None,
            },
            active_recoveries: RwLock::new(BTreeMap::new()),
        }
    }

    /// Initialize the recovery manager
    pub fn init(&self) -> ServiceResult<()> {
        info!("Recovery manager initialized");
        Ok(())
    }

    /// Handle a fault and trigger recovery if appropriate
    pub fn handle_fault(&self, service_id: ServiceId, fault_type: &FaultType) -> ServiceResult<()> {
        let start_time = get_current_time();
        
        // Check if we have a recovery policy for this service
        let policies = self.recovery_policies.read();
        if let Some(policy) = policies.get(&service_id) {
            // Check if recovery should be triggered
            if self.should_trigger_recovery(service_id, policy)? {
                // Start recovery process
                self.start_recovery(service_id, policy.clone())?;
            }
        } else {
            // No policy defined, use default recovery
            self.start_default_recovery(service_id)?;
        }
        
        info!("Fault handling initiated for service {}: {:?}", service_id.0, fault_type);
        Ok(())
    }

    /// Execute recovery action
    pub fn execute_recovery_action(&self, service_id: ServiceId, action_type: RecoveryActionType) -> ServiceResult<bool> {
        let start_time = get_current_time();
        
        // Execute the recovery action based on type
        let success = match action_type {
            RecoveryActionType::RestartService => self.restart_service(service_id)?,
            RecoveryActionType::ResetCircuitBreaker => self.reset_circuit_breaker(service_id)?,
            RecoveryActionType::ReloadConfiguration => self.reload_configuration(service_id)?,
            RecoveryActionType::CleanupResources => self.cleanup_resources(service_id)?,
            RecoveryActionType::HealthCheckOverride => self.override_health_check(service_id)?,
            _ => {
                warn!("Recovery action not implemented: {:?}", action_type);
                false
            }
        };
        
        // Record recovery event
        let event = RecoveryEvent {
            service_id,
            action_id: format!("action-{}-{}-{}", service_id.0, action_type as u8, start_time),
            action_type,
            start_time,
            end_time: Some(get_current_time()),
            success,
            error_message: if success { None } else Some("Recovery action failed".to_string()),
        };

        let mut history = self.recovery_history.write();
        history.push_back(event);

        // Update statistics
        self.update_recovery_stats(start_time, success);

        info!("Recovery action executed for service {}: {:?} = {}", service_id.0, action_type, success);
        Ok(success)
    }

    /// Get recovery statistics
    pub fn get_stats(&self) -> &RecoveryStats {
        &self.recovery_stats
    }

    /// Get action count (for overall service manager stats)
    pub fn get_action_count(&self) -> u64 {
        self.recovery_stats.total_recovery_attempts
    }

    /// Set recovery policy for a service
    pub fn set_recovery_policy(&self, service_id: ServiceId, policy: RecoveryPolicy) -> ServiceResult<()> {
        let mut policies = self.recovery_policies.write();
        policies.insert(service_id, policy);
        
        info!("Recovery policy set for service: {}", service_id.0);
        Ok(())
    }

    /// Get active recoveries
    pub fn get_active_recoveries(&self) -> Vec<ActiveRecovery> {
        let active = self.active_recoveries.read();
        active.values().cloned().collect()
    }

    /// Internal methods
    fn should_trigger_recovery(&self, service_id: ServiceId, policy: &RecoveryPolicy) -> ServiceResult<bool> {
        let active_recoveries = self.active_recoveries.read();
        
        // Check if recovery is already in progress
        if active_recoveries.contains_key(&service_id) {
            return Ok(false);
        }
        
        // Check if we've exceeded max attempts
        // This would require accessing fault detector's failure tracker
        // Simplified implementation for now
        
        Ok(true)
    }

    fn start_recovery(&self, service_id: ServiceId, policy: RecoveryPolicy) -> ServiceResult<()> {
        let active_recovery = ActiveRecovery {
            service_id,
            recovery_policy: policy.clone(),
            current_attempt: 1,
            start_time: get_current_time(),
            active_actions: Vec::new(),
            backoff_timer: None,
        };

        let mut active_recoveries = self.active_recoveries.write();
        active_recoveries.insert(service_id, active_recovery);
        
        // Execute recovery actions based on strategy
        self.execute_recovery_strategy(service_id, &policy)?;
        
        Ok(())
    }

    fn start_default_recovery(&self, service_id: ServiceId) -> ServiceResult<()> {
        let default_policy = RecoveryPolicy {
            service_id,
            max_recovery_attempts: 3,
            recovery_strategy: RecoveryStrategy::RestartWithDelay,
            backoff_strategy: BackoffStrategy::Linear { delay_ms: 1000, increment_ms: 1000 },
            escalation_policy: EscalationPolicy {
                enabled: false,
                escalation_levels: Vec::new(),
            },
            recovery_timeout: 30000,
        };
        
        self.start_recovery(service_id, default_policy)
    }

    fn execute_recovery_strategy(&self, service_id: ServiceId, policy: &RecoveryPolicy) -> ServiceResult<()> {
        match policy.recovery_strategy {
            RecoveryStrategy::Restart => {
                self.execute_recovery_action(service_id, RecoveryActionType::RestartService)?;
            }
            RecoveryStrategy::RestartWithDelay => {
                // Would implement delay logic here
                self.execute_recovery_action(service_id, RecoveryActionType::RestartService)?;
            }
            RecoveryStrategy::CircuitBreakerReset => {
                self.execute_recovery_action(service_id, RecoveryActionType::ResetCircuitBreaker)?;
            }
            RecoveryStrategy::ConfigurationReload => {
                self.execute_recovery_action(service_id, RecoveryActionType::ReloadConfiguration)?;
            }
            _ => {
                warn!("Recovery strategy not implemented: {:?}", policy.recovery_strategy);
            }
        }
        
        Ok(())
    }

    fn restart_service(&self, service_id: ServiceId) -> ServiceResult<bool> {
        // This would interact with the service manager to restart the service
        info!("Restarting service: {}", service_id.0);
        Ok(true)
    }

    fn reset_circuit_breaker(&self, service_id: ServiceId) -> ServiceResult<bool> {
        info!("Resetting circuit breaker for service: {}", service_id.0);
        Ok(true)
    }

    fn reload_configuration(&self, service_id: ServiceId) -> ServiceResult<bool> {
        info!("Reloading configuration for service: {}", service_id.0);
        Ok(true)
    }

    fn cleanup_resources(&self, service_id: ServiceId) -> ServiceResult<bool> {
        info!("Cleaning up resources for service: {}", service_id.0);
        Ok(true)
    }

    fn override_health_check(&self, service_id: ServiceId) -> ServiceResult<bool> {
        info!("Overriding health check for service: {}", service_id.0);
        Ok(true)
    }

    fn update_recovery_stats(&self, start_time: u64, success: bool) {
        let recovery_time = get_current_time() - start_time;
        
        self.recovery_stats.total_recovery_attempts += 1;
        if success {
            self.recovery_stats.successful_recoveries += 1;
        } else {
            self.recovery_stats.failed_recoveries += 1;
        }
        
        self.recovery_stats.last_recovery_time = Some(get_current_time());
        
        // Update running average
        let total_attempts = self.recovery_stats.total_recovery_attempts as f64;
        let current_avg = self.recovery_stats.average_recovery_time;
        let new_avg = (current_avg * (total_attempts - 1.0) + recovery_time as f64) / total_attempts;
        self.recovery_stats.average_recovery_time = new_avg;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_detector_creation() {
        let detector = FaultDetector::new();
        assert_eq!(detector.get_stats().total_detections, 0);
        assert_eq!(detector.get_stats().false_positives, 0);
    }

    #[test]
    fn test_recovery_manager_creation() {
        let manager = RecoveryManager::new();
        assert_eq!(manager.get_stats().total_recovery_attempts, 0);
        assert_eq!(manager.get_stats().successful_recoveries, 0);
    }

    #[test]
    fn test_failure_pattern_enum() {
        assert_eq!(FailurePattern::None as u8, 0);
        assert_eq!(FailurePattern::Transient as u8, 1);
        assert_eq!(FailurePattern::Persistent as u8, 3);
    }

    #[test]
    fn test_fault_severity_levels() {
        assert_eq!(FaultSeverity::Info as u8, 0);
        assert_eq!(FaultSeverity::Warning as u8, 1);
        assert_eq!(FaultSeverity::Critical as u8, 3);
        assert_eq!(FaultSeverity::Fatal as u8, 4);
    }

    #[test]
    fn test_fault_type_enum() {
        assert_eq!(FaultType::HealthCheckFailure as u8, 0);
        assert_eq!(FaultType::ResourceExhaustion as u8, 1);
        assert_eq!(FaultType::NetworkIssue as u8, 4);
    }

    #[test]
    fn test_recovery_strategy_enum() {
        assert_eq!(RecoveryStrategy::None as u8, 0);
        assert_eq!(RecoveryStrategy::Restart as u8, 1);
        assert_eq!(RecoveryStrategy::Failover as u8, 4);
    }

    #[test]
    fn test_backoff_strategy_enum() {
        assert_eq!(BackoffStrategy::None as u8, 0);
        assert_eq!(BackoffStrategy::Linear as u8, 1);
        assert_eq!(BackoffStrategy::Exponential as u8, 2);
    }

    #[test]
    fn test_detection_sensitivity() {
        assert_eq!(DetectionSensitivity::Low as u8, 0);
        assert_eq!(DetectionSensitivity::Normal as u8, 1);
        assert_eq!(DetectionSensitivity::Paranoid as u8, 3);
    }

    #[test]
    fn test_circuit_breaker_state() {
        assert_eq!(super::super::load_balancer::CircuitBreakerState::Closed as u8, 0);
        assert_eq!(super::super::load_balancer::CircuitBreakerState::Open as u8, 1);
        assert_eq!(super::super::load_balancer::CircuitBreakerState::HalfOpen as u8, 2);
    }

    #[test]
    fn test_failure_tracker_creation() {
        let tracker = FailureTracker {
            service_id: ServiceId(1),
            failure_count: 5,
            consecutive_failures: 3,
            last_failure_time: Some(1000),
            failure_pattern: FailurePattern::Intermittent,
            severity: FaultSeverity::Warning,
            detection_threshold: 3,
            recovery_threshold: 1,
        };

        assert_eq!(tracker.failure_count, 5);
        assert_eq!(tracker.consecutive_failures, 3);
        assert_eq!(tracker.failure_pattern, FailurePattern::Intermittent);
    }

    #[test]
    fn test_recovery_policy_creation() {
        let policy = RecoveryPolicy {
            service_id: ServiceId(1),
            max_recovery_attempts: 3,
            recovery_strategy: RecoveryStrategy::RestartWithDelay,
            backoff_strategy: BackoffStrategy::Exponential { 
                initial_delay_ms: 1000, 
                multiplier: 2.0, 
                max_delay_ms: 30000 
            },
            escalation_policy: EscalationPolicy {
                enabled: false,
                escalation_levels: Vec::new(),
            },
            recovery_timeout: 30000,
        };

        assert_eq!(policy.max_recovery_attempts, 3);
        assert_eq!(policy.recovery_strategy, RecoveryStrategy::RestartWithDelay);
    }

    #[test]
    fn test_recovery_action_creation() {
        let action = RecoveryAction {
            id: "restart-1".to_string(),
            service_id: ServiceId(1),
            action_type: RecoveryActionType::RestartService,
            parameters: BTreeMap::new(),
            timeout: 10000,
            retry_count: 0,
            max_retries: 3,
        };

        assert_eq!(action.action_type, RecoveryActionType::RestartService);
        assert_eq!(action.timeout, 10000);
        assert_eq!(action.max_retries, 3);
    }
}