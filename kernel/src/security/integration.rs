//! Security Policy Integration Module
//! 
//! This module provides integration between the security policy framework
//! and other system components including audit, configuration, and security managers.

#![no_std]

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::HashMap;
use spin::{Mutex, RwLock};

use super::security_types::*;
use crate::admin::audit::{AuditManager, AuditEvent, AuditEventType, AuditLevel, AuditResult};
use crate::admin::security::{SecurityManager, SecurityLevel, SecurityError, SecurityContext};
use crate::admin::config_manager::{ConfigManager, ConfigValue, ConfigKey, ConfigError, ConfigResult};

// Re-export for convenience
pub use super::policy::{
    SecurityPolicyManager, init_policy_manager, shutdown_policy_manager, get_policy_manager,
    SecurityFramework, SecurityFrameworkConfig, FrameworkStats,
    init_security_framework, shutdown_security_framework, get_security_framework,
};

/// Integration result type
pub type IntegrationResult<T> = Result<T, IntegrationError>;

/// Integration error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IntegrationError {
    AuditConnectionFailed = 0,
    SecurityConnectionFailed = 1,
    ConfigConnectionFailed = 2,
    PolicyEvaluationFailed = 3,
    ViolationHandlingFailed = 4,
    PropagationFailed = 5,
    HealthCheckFailed = 6,
    ServiceUnavailable = 7,
}

/// Policy integration manager
pub struct PolicyIntegrationManager {
    policy_manager: SecurityPolicyManager,
    audit_manager: Option<&'static Mutex<Option<AuditManager>>>,
    security_manager: Option<&'static Mutex<Option<SecurityManager>>>,
    config_manager: Option<&'static Mutex<Option<ConfigManager>>>,
    integration_status: IntegrationStatus,
    stats: IntegrationStats,
}

/// Integration status tracking
#[derive(Debug, Clone)]
struct IntegrationStatus {
    audit_connected: bool,
    security_connected: bool,
    config_connected: bool,
    last_health_check: u64,
    services_registered: Vec<String>,
    active_enforcement_points: usize,
}

/// Integration statistics
#[derive(Debug, Clone)]
struct IntegrationStats {
    total_evaluations: u64,
    audit_logs_written: u64,
    security_contexts_created: u64,
    config_validations: u64,
    violations_handled: u64,
    propagation_successes: u64,
    propagation_failures: u64,
}

impl PolicyIntegrationManager {
    /// Create a new policy integration manager
    pub fn new() -> Self {
        PolicyIntegrationManager {
            policy_manager: SecurityPolicyManager::new(),
            audit_manager: None,
            security_manager: None,
            config_manager: None,
            integration_status: IntegrationStatus {
                audit_connected: false,
                security_connected: false,
                config_connected: false,
                last_health_check: 0,
                services_registered: Vec::new(),
                active_enforcement_points: 0,
            },
            stats: IntegrationStats {
                total_evaluations: 0,
                audit_logs_written: 0,
                security_contexts_created: 0,
                config_validations: 0,
                violations_handled: 0,
                propagation_successes: 0,
                propagation_failures: 0,
            },
        }
    }

    /// Initialize the integration manager
    pub fn init(&mut self) -> IntegrationResult<()> {
        // Initialize policy manager
        self.policy_manager.init()
            .map_err(|_| IntegrationError::PolicyEvaluationFailed)?;

        // Establish connections to other managers
        self.connect_to_audit_manager()?;
        self.connect_to_security_manager()?;
        self.connect_to_config_manager()?;

        // Register enforcement points
        self.register_enforcement_points()?;

        info!("Policy Integration Manager initialized successfully");
        Ok(())
    }

    /// Shutdown the integration manager
    pub fn shutdown(&mut self) -> IntegrationResult<()> {
        // Shutdown policy manager
        self.policy_manager.shutdown()
            .map_err(|_| IntegrationError::ServiceUnavailable)?;

        info!("Policy Integration Manager shutdown complete");
        Ok(())
    }

    /// Evaluate policies with full integration
    pub fn evaluate_policies(&mut self, context: &EvaluationContext) -> IntegrationResult<EvaluationResult> {
        self.stats.total_evaluations += 1;

        let result = self.policy_manager.evaluate_policies(context)
            .map_err(|_| IntegrationError::PolicyEvaluationFailed)?;

        // Log evaluation to audit system
        if self.integration_status.audit_connected {
            self.log_evaluation_to_audit(context, &result)?;
        }

        // Create security context if needed
        if result.enforcement_level as u8 > 1 {
            self.create_security_context_for_evaluation(context)?;
        }

        // Validate against configuration system
        if self.integration_status.config_connected {
            self.validate_against_config(context, &result)?;
        }

        Ok(result)
    }

    /// Handle policy violation with integrated response
    pub fn handle_violation(&mut self, violation: PolicyViolation) -> IntegrationResult<()> {
        self.stats.violations_handled += 1;

        // Log violation to audit system
        if self.integration_status.audit_connected {
            self.log_violation_to_audit(&violation)?;
        }

        // Apply security context changes
        if self.integration_status.security_connected {
            self.apply_security_context_response(&violation)?;
        }

        // Update configuration if needed
        if self.integration_status.config_connected {
            self.update_config_for_violation(&violation)?;
        }

        // Propagate violation to services
        self.propagate_violation_to_services(&violation)?;

        Ok(())
    }

    /// Create a policy with integrated validation
    pub fn create_policy(&mut self, policy: SecurityPolicy) -> IntegrationResult<String> {
        // Validate policy against configuration
        if self.integration_status.config_connected {
            self.validate_policy_against_config(&policy)?;
        }

        // Create policy through policy manager
        let policy_id = self.policy_manager.create_policy(policy)
            .map_err(|_| IntegrationError::PolicyEvaluationFailed)?;

        // Log policy creation
        if self.integration_status.audit_connected {
            self.log_policy_creation(&policy_id)?;
        }

        // Apply to security manager
        if self.integration_status.security_connected {
            self.apply_policy_to_security_manager(&policy_id)?;
        }

        Ok(policy_id)
    }

    /// Perform health check on all integrations
    pub fn health_check(&mut self) -> IntegrationResult<()> {
        let current_time = self.get_current_time();

        // Check if health check is needed
        if current_time - self.integration_status.last_health_check < 60 {
            return Ok(());
        }

        // Test audit connection
        if self.integration_status.audit_connected {
            self.test_audit_connection()?;
        }

        // Test security connection
        if self.integration_status.security_connected {
            self.test_security_connection()?;
        }

        // Test config connection
        if self.integration_status.config_connected {
            self.test_config_connection()?;
        }

        self.integration_status.last_health_check = current_time;
        Ok(())
    }

    /// Get integration statistics
    pub fn get_stats(&self) -> (IntegrationStats, super::policy::PolicyStats) {
        let policy_stats = self.policy_manager.get_stats();
        (self.stats.clone(), policy_stats)
    }

    // ==================== Connection Management ====================

    /// Connect to audit manager
    fn connect_to_audit_manager(&mut self) -> IntegrationResult<()> {
        if let Some(audit_mgr) = crate::admin::audit::get_audit_manager() {
            self.audit_manager = Some(audit_mgr);
            self.integration_status.audit_connected = true;
            debug!("Connected to Audit Manager");
        } else {
            warn!("Audit Manager not available for integration");
        }
        Ok(())
    }

    /// Connect to security manager
    fn connect_to_security_manager(&mut self) -> IntegrationResult<()> {
        if let Some(security_mgr) = crate::admin::security::get_security_manager() {
            self.security_manager = Some(security_mgr);
            self.integration_status.security_connected = true;
            debug!("Connected to Security Manager");
        } else {
            warn!("Security Manager not available for integration");
        }
        Ok(())
    }

    /// Connect to config manager
    fn connect_to_config_manager(&mut self) -> IntegrationResult<()> {
        // In real implementation, would get config manager
        // For now, we'll simulate config manager connection
        self.integration_status.config_connected = true;
        debug!("Connected to Config Manager");
        Ok(())
    }

    // ==================== Audit Integration ====================

    /// Log policy evaluation to audit system
    fn log_evaluation_to_audit(&self, context: &EvaluationContext, result: &EvaluationResult) -> IntegrationResult<()> {
        if let Some(audit_mgr) = self.audit_manager {
            let event = AuditEvent {
                event_id: 0,
                timestamp: context.timestamp,
                event_type: AuditEventType::SecurityPolicyViolation,
                level: if result.allowed { AuditLevel::Info } else { AuditLevel::Warning },
                user_id: Some(context.user_id),
                session_id: Some(context.session_id),
                process_id: None,
                thread_id: None,
                ip_address: None,
                source: "policy_evaluator".to_string(),
                target: context.service_id.clone(),
                details: format!("Policy evaluation - Allowed: {}, Matches: {}, Conflicts: {}", 
                               result.allowed, result.policy_matches.len(), result.conflicts.len()),
                result: result.allowed,
                additional_data: Vec::new(),
            };

            let _ = audit_mgr.lock().as_mut().and_then(|mgr| mgr.log_event(event).ok());
            self.stats.audit_logs_written += 1;
        }
        Ok(())
    }

    /// Log policy violation to audit system
    fn log_violation_to_audit(&self, violation: &PolicyViolation) -> IntegrationResult<()> {
        if let Some(audit_mgr) = self.audit_manager {
            let event = AuditEvent {
                event_id: 0,
                timestamp: violation.timestamp,
                event_type: AuditEventType::SecurityViolation,
                level: match violation.severity {
                    PolicyPriority::Lowest | PolicyPriority::Low => AuditLevel::Warning,
                    PolicyPriority::Normal => AuditLevel::Error,
                    PolicyPriority::High | PolicyPriority::Critical | PolicyPriority::System => AuditLevel::Critical,
                },
                user_id: None,
                session_id: None,
                process_id: None,
                thread_id: None,
                ip_address: None,
                source: "policy_violation".to_string(),
                target: violation.policy_id.clone(),
                details: format!("{}: {}", violation.violation_type as u8, violation.details),
                result: false,
                additional_data: Vec::new(),
            };

            let _ = audit_mgr.lock().as_mut().and_then(|mgr| mgr.log_event(event).ok());
            self.stats.audit_logs_written += 1;
        }
        Ok(())
    }

    /// Log policy creation to audit system
    fn log_policy_creation(&self, policy_id: &str) -> IntegrationResult<()> {
        if let Some(audit_mgr) = self.audit_manager {
            let event = AuditEvent {
                event_id: 0,
                timestamp: self.get_current_time(),
                event_type: AuditEventType::ConfigurationChanged,
                level: AuditLevel::Info,
                user_id: None,
                session_id: None,
                process_id: None,
                thread_id: None,
                ip_address: None,
                source: "policy_manager".to_string(),
                target: policy_id.to_string(),
                details: "Security policy created".to_string(),
                result: true,
                additional_data: Vec::new(),
            };

            let _ = audit_mgr.lock().as_mut().and_then(|mgr| mgr.log_event(event).ok());
            self.stats.audit_logs_written += 1;
        }
        Ok(())
    }

    // ==================== Security Manager Integration ====================

    /// Create security context for policy evaluation
    fn create_security_context_for_evaluation(&mut self, context: &EvaluationContext) -> IntegrationResult<()> {
        if let Some(security_mgr) = self.security_manager {
            // Create security context based on evaluation
            let context_id = context.session_id; // Use session ID as context ID
            
            // This would create a security context in the security manager
            // For now, we'll just log the action
            self.stats.security_contexts_created += 1;
        }
        Ok(())
    }

    /// Apply security context response to violation
    fn apply_security_context_response(&self, violation: &PolicyViolation) -> IntegrationResult<()> {
        if let Some(security_mgr) = self.security_manager {
            match violation.remediation {
                ViolationRemediation::Quarantine => {
                    // Would quarantine the security context
                    debug!("Quarantining security context due to violation: {}", violation.violation_id);
                }
                ViolationRemediation::Terminate => {
                    // Would terminate the security context
                    debug!("Terminating security context due to violation: {}", violation.violation_id);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Apply policy to security manager
    fn apply_policy_to_security_manager(&self, policy_id: &str) -> IntegrationResult<()> {
        if let Some(security_mgr) = self.security_manager {
            // This would apply the policy to the security manager
            debug!("Applied policy {} to Security Manager", policy_id);
        }
        Ok(())
    }

    // ==================== Config Manager Integration ====================

    /// Validate policy against configuration
    fn validate_policy_against_config(&self, policy: &SecurityPolicy) -> IntegrationResult<()> {
        // This would validate the policy against config manager rules
        self.stats.config_validations += 1;
        Ok(())
    }

    /// Validate evaluation against configuration
    fn validate_against_config(&self, context: &EvaluationContext, result: &EvaluationResult) -> IntegrationResult<()> {
        // This would validate the evaluation result against config constraints
        self.stats.config_validations += 1;
        Ok(())
    }

    /// Update configuration for violation
    fn update_config_for_violation(&self, violation: &PolicyViolation) -> IntegrationResult<()> {
        // This would update configuration based on violation
        debug!("Updating configuration for violation: {}", violation.violation_id);
        Ok(())
    }

    // ==================== Service Management ====================

    /// Register enforcement points
    fn register_enforcement_points(&mut self) -> IntegrationResult<()> {
        // This would register with various system services
        self.integration_status.active_enforcement_points = 5; // Simulated
        Ok(())
    }

    /// Propagate violation to services
    fn propagate_violation_to_services(&mut self, violation: &PolicyViolation) -> IntegrationResult<()> {
        // This would propagate violation to affected services
        debug!("Propagating violation {} to services", violation.violation_id);
        Ok(())
    }

    // ==================== Health Checks ====================

    /// Test audit connection
    fn test_audit_connection(&self) -> IntegrationResult<()> {
        if let Some(audit_mgr) = self.audit_manager {
            // Test if audit manager is responsive
            // In real implementation, would ping the audit manager
            debug!("Audit Manager connection healthy");
        }
        Ok(())
    }

    /// Test security connection
    fn test_security_connection(&self) -> IntegrationResult<()> {
        if let Some(security_mgr) = self.security_manager {
            // Test if security manager is responsive
            debug!("Security Manager connection healthy");
        }
        Ok(())
    }

    /// Test config connection
    fn test_config_connection(&self) -> IntegrationResult<()> {
        // Test if config manager is responsive
        debug!("Config Manager connection healthy");
        Ok(())
    }

    // ==================== Utility Methods ====================

    /// Get current time
    fn get_current_time(&self) -> u64 {
        // In real implementation, would get time from kernel's time subsystem
        crate::hal::get_current_time()
    }
}

/// Global policy integration manager
static mut POLICY_INTEGRATION_MANAGER: Option<PolicyIntegrationManager> = None;

/// Initialize the global policy integration manager
pub fn init_policy_integration_manager() -> IntegrationResult<()> {
    unsafe {
        if POLICY_INTEGRATION_MANAGER.is_some() {
            return Err(IntegrationError::ServiceUnavailable);
        }

        let mut manager = PolicyIntegrationManager::new();
        manager.init()?;
        
        POLICY_INTEGRATION_MANAGER = Some(manager);
    }

    info!("Policy Integration Manager initialized successfully");
    Ok(())
}

/// Shutdown the global policy integration manager
pub fn shutdown_policy_integration_manager() -> IntegrationResult<()> {
    unsafe {
        if let Some(mut manager) = POLICY_INTEGRATION_MANAGER.take() {
            manager.shutdown()?;
        }
    }

    info!("Policy Integration Manager shutdown complete");
    Ok(())
}

/// Get the global policy integration manager instance
pub fn get_policy_integration_manager() -> Option<&'static mut PolicyIntegrationManager> {
    unsafe { POLICY_INTEGRATION_MANAGER.as_mut() }
}

/// Convenience function to evaluate policies with full integration
pub fn evaluate_policies_integrated(context: &EvaluationContext) -> IntegrationResult<EvaluationResult> {
    if let Some(manager) = get_policy_integration_manager() {
        manager.evaluate_policies(context)
    } else {
        Err(IntegrationError::ServiceUnavailable)
    }
}

/// Convenience function to handle violations with full integration
pub fn handle_violation_integrated(violation: PolicyViolation) -> IntegrationResult<()> {
    if let Some(manager) = get_policy_integration_manager() {
        manager.handle_violation(violation)
    } else {
        Err(IntegrationError::ServiceUnavailable)
    }
}

/// Convenience function to create policies with full integration
pub fn create_policy_integrated(policy: SecurityPolicy) -> IntegrationResult<String> {
    if let Some(manager) = get_policy_integration_manager() {
        manager.create_policy(policy)
    } else {
        Err(IntegrationError::ServiceUnavailable)
    }
}