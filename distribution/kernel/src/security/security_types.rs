//! Security Types Module
//! 
//! This module defines common types and data structures used throughout
//! the security policy framework.

#![no_std]

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::HashMap;

/// Security policy management result
pub type PolicyResult<T> = Result<T, PolicyError>;

/// Security policy error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PolicyError {
    NotFound = 0,
    InvalidPolicy = 1,
    PolicyConflict = 2,
    EnforcementFailed = 3,
    ViolationDetected = 4,
    VersionMismatch = 5,
    PropagationFailed = 6,
    EvaluationError = 7,
    InvalidScope = 8,
    AccessDenied = 9,
    ResourceExhausted = 10,
    ConflictResolutionFailed = 11,
    ServiceUnavailable = 12,
}

/// Security rule categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RuleCategory {
    Access = 0,
    Process = 1,
    Network = 2,
    Data = 3,
    System = 4,
    User = 5,
    Resource = 6,
    Compliance = 7,
}

/// Policy enforcement modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EnforcementMode {
    Disabled = 0,
    Audit = 1,
    Soft = 2,
    Hard = 3,
    Strict = 4,
    Emergency = 5,
}

/// Policy priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PolicyPriority {
    Lowest = 0,
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
    System = 5,
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub allowed: bool,
    pub policy_matches: Vec<PolicyMatch>,
    pub conflicts: Vec<PolicyConflict>,
    pub violations: Vec<PolicyViolation>,
    pub enforcement_level: EnforcementMode,
    pub audit_required: bool,
    pub quarantine_required: bool,
}

/// Policy match information
#[derive(Debug, Clone)]
pub struct PolicyMatch {
    pub policy_id: String,
    pub priority: PolicyPriority,
    pub rule_matches: Vec<RuleMatch>,
    pub confidence: f32,
}

/// Rule match information
#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub rule_id: String,
    pub category: RuleCategory,
    pub matched: bool,
    pub parameters: HashMap<String, String>,
}

/// Policy conflict information
#[derive(Debug, Clone)]
pub struct PolicyConflict {
    pub policy_a: String,
    pub policy_b: String,
    pub conflict_type: ConflictType,
    pub resolution: ConflictResolution,
}

/// Types of policy conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConflictType {
    AllowDeny = 0,
    Priority = 1,
    Scope = 2,
    Time = 3,
    Resource = 4,
    Capability = 5,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConflictResolution {
    DenyAll = 0,
    AllowAll = 1,
    HighestPriority = 2,
    MostSpecific = 3,
    MostRecent = 4,
    UserChoice = 5,
    SystemDefault = 6,
}

/// Policy violation details
#[derive(Debug, Clone)]
pub struct PolicyViolation {
    pub violation_id: String,
    pub policy_id: String,
    pub rule_category: RuleCategory,
    pub violation_type: ViolationType,
    pub severity: PolicyPriority,
    pub timestamp: u64,
    pub source_context: String,
    pub target_context: String,
    pub details: String,
    pub remediation: ViolationRemediation,
}

/// Types of policy violations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ViolationType {
    UnauthorizedAccess = 0,
    ResourceExceeded = 1,
    CapabilityViolation = 2,
    TimeConstraint = 3,
    NetworkViolation = 4,
    DataClassification = 5,
    ProcessViolation = 6,
    SystemIntegrity = 7,
    ComplianceBreach = 8,
}

/// Violation remediation actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ViolationRemediation {
    None = 0,
    Log = 1,
    Alert = 2,
    Throttle = 3,
    Deny = 4,
    Quarantine = 5,
    Terminate = 6,
    Isolate = 7,
    Notify = 8,
}

/// Policy scope definitions
#[derive(Debug, Clone)]
pub enum PolicyScope {
    System,
    Service(String),
    User(String),
    Role(String),
    Namespace(String),
    Resource(String),
    TimeWindow { start: u64, end: u64 },
    Contextual { conditions: HashMap<String, String> },
}

/// Policy condition for evaluation
#[derive(Debug, Clone)]
pub struct PolicyCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: ConditionValue,
    pub case_sensitive: bool,
}

/// Condition operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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
    NotInSet = 9,
    Exists = 10,
    NotExists = 11,
}

/// Condition value types
#[derive(Debug, Clone)]
pub enum ConditionValue {
    String(String),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    Boolean(bool),
    Set(Vec<String>),
    Range { min: i64, max: i64 },
    TimeRange { start: u64, end: u64 },
}

/// Security policy definition
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub policy_id: String,
    pub name: String,
    pub description: String,
    pub version: PolicyVersion,
    pub category: RuleCategory,
    pub priority: PolicyPriority,
    pub scope: PolicyScope,
    pub conditions: Vec<PolicyCondition>,
    pub rules: Vec<SecurityRule>,
    pub enforcement_mode: EnforcementMode,
    pub enabled: core::sync::atomic::AtomicBool,
    pub created_at: u64,
    pub updated_at: u64,
    pub expires_at: Option<u64>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Policy version information
#[derive(Debug, Clone)]
pub struct PolicyVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
}

/// Security rule definition
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub rule_id: String,
    pub name: String,
    pub category: RuleCategory,
    pub description: String,
    pub parameters: RuleParameters,
    pub actions: Vec<RuleAction>,
    pub conditions: Vec<PolicyCondition>,
    pub enabled: bool,
    pub priority: PolicyPriority,
    pub rate_limits: Option<RateLimit>,
    pub resource_limits: Option<ResourceLimit>,
}

/// Rule action definition
#[derive(Debug, Clone)]
pub struct RuleAction {
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub target: ActionTarget,
    pub delay: u64,
    pub retry_count: u32,
    pub on_failure: FailureAction,
}

/// Action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActionType {
    Allow = 0,
    Deny = 1,
    Log = 2,
    Audit = 3,
    Alert = 4,
    Throttle = 5,
    Quarantine = 6,
    Terminate = 7,
    Redirect = 8,
    Modify = 9,
    Isolate = 10,
    Notify = 11,
    Rollback = 12,
    Failover = 13,
    Scale = 14,
    LoadBalance = 15,
}

/// Action target types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActionTarget {
    SelfTarget = 0,
    Source = 1,
    Target = 2,
    Service(String),
    User(String),
    System = 4,
    Network = 5,
    Storage = 6,
}

/// Failure action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FailureAction {
    Continue = 0,
    Retry = 1,
    Fallback = 2,
    Abort = 3,
    Escalate = 4,
}

/// Rule parameters
#[derive(Debug, Clone)]
pub struct RuleParameters {
    pub timeout: u64,
    pub retry_count: u32,
    pub cache_ttl: u64,
    pub enable_audit: bool,
    pub quarantine_on_violation: bool,
    pub isolation_level: u32,
    pub resource_pool: Option<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
    pub window_size: u64,
}

/// Resource limiting configuration
#[derive(Debug, Clone)]
pub struct ResourceLimit {
    pub memory_mb: u64,
    pub cpu_percent: u32,
    pub disk_io_mb: u64,
    pub network_bandwidth_mbps: u32,
    pub file_descriptors: u32,
    pub processes: u32,
}

/// Policy history and versioning
#[derive(Debug, Clone)]
pub struct PolicyVersionHistory {
    pub history_id: String,
    pub policy_id: String,
    pub version: PolicyVersion,
    pub snapshot: Vec<u8>,
    pub changes: PolicyChangeSet,
    pub created_by: String,
    pub created_at: u64,
    pub is_rollback_point: bool,
}

/// Policy change tracking
#[derive(Debug, Clone)]
pub struct PolicyChangeSet {
    pub added_rules: Vec<String>,
    pub removed_rules: Vec<String>,
    pub modified_rules: Vec<String>,
    pub scope_changes: Vec<String>,
    pub enforcement_changes: Vec<String>,
}

/// Security policy statistics
#[derive(Debug, Clone)]
pub struct PolicyStats {
    pub total_policies: usize,
    pub enabled_policies: usize,
    pub total_rules: usize,
    pub active_rules: usize,
    pub evaluations_performed: u64,
    pub policy_violations: u64,
    pub conflict_resolutions: u64,
    pub propagation_success: u64,
    pub propagation_failures: u64,
    pub rollbacks_performed: u64,
}

/// Service policy binding for propagation
#[derive(Debug, Clone)]
pub struct ServicePolicyBinding {
    pub service_id: String,
    pub policy_ids: Vec<String>,
    pub last_update: u64,
    pub status: PropagationStatus,
    pub error_count: u32,
}

/// Policy propagation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PropagationStatus {
    Pending = 0,
    InProgress = 1,
    Success = 2,
    Failed = 3,
    Timeout = 4,
    Disabled = 5,
}

/// Evaluation context for policy evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub user_id: u64,
    pub session_id: u64,
    pub service_id: String,
    pub resource_id: String,
    pub resource_type: String,
    pub operation: String,
    pub security_level: crate::admin::security::SecurityLevel,
    pub timestamp: u64,
    pub namespace: String,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Security framework configuration
#[derive(Debug, Clone)]
pub struct SecurityFrameworkConfig {
    pub enable_policy_enforcement: bool,
    pub enable_violation_logging: bool,
    pub enable_propagation: bool,
    pub enable_versioning: bool,
    pub max_policy_cache_size: usize,
    pub violation_retention_days: u32,
    pub propagation_timeout_ms: u64,
    pub evaluation_cache_ttl: u64,
    pub enable_real_time_monitoring: bool,
    pub audit_integration_enabled: bool,
    pub config_integration_enabled: bool,
    pub conflict_resolution_strategy: String,
}

/// Security framework statistics
#[derive(Debug, Clone)]
pub struct FrameworkStats {
    pub policy_evaluations: u64,
    pub violations_detected: u64,
    pub policies_propagated: u64,
    pub rollbacks_performed: u64,
    pub audit_integrations: u64,
    pub config_integrations: u64,
    pub active_enforcement_points: usize,
    pub services_monitored: usize,
    pub last_violation: u64,
    pub system_security_score: f32,
}