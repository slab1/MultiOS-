//! Service Monitoring and Health Checking
//! 
//! This module provides comprehensive monitoring and health checking
//! capabilities for services in the MultiOS service management framework.

use spin::{Mutex, RwLock};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicU64, Ordering};

use super::{ServiceId, ServiceResult, ServiceError, service::{HealthStatus, HealthCheckResult, ServiceMetrics}};
use super::config::{MonitoringConfig, HealthCheckResult as ConfigHealthCheck};
use super::get_current_time;

/// Service Monitor - Main monitoring component
pub struct ServiceMonitor {
    health_checkers: RwLock<BTreeMap<ServiceId, HealthChecker>>,
    monitoring_config: RwLock<BTreeMap<ServiceId, MonitoringConfig>>,
    metrics_collector: MetricsCollector,
    alert_manager: AlertManager,
    monitoring_stats: MonitoringStats,
}

/// Health Checker - Performs health checks on services
pub struct HealthChecker {
    service_id: ServiceId,
    check_interval: u32,
    timeout: u32,
    consecutive_failures: u32,
    last_check_time: Option<u64>,
    health_status: HealthStatus,
    custom_health_checks: Vec<CustomHealthCheck>,
}

/// Metrics Collector - Collects service metrics
pub struct MetricsCollector {
    metrics_store: RwLock<BTreeMap<ServiceId, VecDeque<ServiceMetrics>>>,
    collection_interval: u32,
    max_metrics_per_service: usize,
}

/// Alert Manager - Handles alerting for service issues
pub struct AlertManager {
    active_alerts: RwLock<BTreeMap<String, Alert>>,
    alert_rules: RwLock<Vec<AlertRule>>,
    alert_history: RwLock<VecDeque<Alert>>,
}

/// Monitoring Statistics
#[derive(Debug, Clone)]
struct MonitoringStats {
    total_health_checks: u64,
    failed_health_checks: u64,
    total_metrics_collected: u64,
    alerts_triggered: u64,
    average_health_check_time: f64,
    last_check_time: Option<u64>,
}

/// Health Check Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HealthCheckType {
    Http = 0,
    Tcp = 1,
    Process = 2,
    File = 3,
    Command = 4,
    Custom = 5,
}

/// Health Check Configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub check_type: HealthCheckType,
    pub endpoint: Option<String>,
    pub timeout: u32,
    pub interval: u32,
    pub max_retries: u32,
    pub expected_status: Option<i32>,
    pub custom_command: Option<String>,
    pub headers: BTreeMap<String, String>,
    pub payload: Option<String>,
}

/// Custom Health Check
#[derive(Debug, Clone)]
pub struct CustomHealthCheck {
    pub name: String,
    pub check_function: CustomCheckFunction,
    pub interval: u32,
    pub timeout: u32,
}

/// Custom Check Function
pub trait CustomCheckFunction: Send + Sync {
    fn check(&self, service_id: ServiceId) -> HealthCheckResult;
}

/// Default Custom Health Check
#[derive(Debug, Clone)]
pub struct DefaultCustomCheck {
    pub name: String,
}

impl CustomCheckFunction for DefaultCustomCheck {
    fn check(&self, service_id: ServiceId) -> HealthCheckResult {
        HealthCheckResult {
            service_id,
            healthy: true,
            response_time: 0,
            error_message: None,
            timestamp: get_current_time(),
        }
    }
}

/// Alert Rule
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

/// Alert Condition
#[derive(Debug, Clone)]
pub enum AlertCondition {
    HealthCheckFailed,
    HighResponseTime { threshold: u64 },
    HighMemoryUsage { threshold: f32 },
    HighCpuUsage { threshold: f32 },
    ServiceDown,
    Custom { metric: String, operator: ComparisonOperator, value: f64 },
}

/// Alert Severity Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
}

/// Alert State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertState {
    Active = 0,
    Acknowledged = 1,
    Resolved = 2,
}

/// Alert Structure
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub service_id: ServiceId,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub message: String,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub acknowledged_by: Option<String>,
}

/// Alert History Entry
#[derive(Debug, Clone)]
pub struct AlertHistoryEntry {
    pub alert_id: String,
    pub event_type: AlertEventType,
    pub timestamp: u64,
    pub details: String,
}

/// Alert Event Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertEventType {
    Triggered = 0,
    Acknowledged = 1,
    Resolved = 2,
    Escalated = 3,
}

/// Service Health Report
#[derive(Debug, Clone)]
pub struct ServiceHealthReport {
    pub service_id: ServiceId,
    pub overall_health: HealthStatus,
    pub last_health_check: Option<HealthCheckResult>,
    pub recent_health_checks: Vec<HealthCheckResult>,
    pub metrics: Option<ServiceMetrics>,
    pub alerts: Vec<Alert>,
    pub uptime: u64,
    pub availability: f32,
}

/// Comparison Operators for Custom Alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ComparisonOperator {
    GreaterThan = 0,
    LessThan = 1,
    Equal = 2,
    NotEqual = 3,
}

impl ServiceMonitor {
    /// Create a new service monitor
    pub fn new() -> Self {
        ServiceMonitor {
            health_checkers: RwLock::new(BTreeMap::new()),
            monitoring_config: RwLock::new(BTreeMap::new()),
            metrics_collector: MetricsCollector::new(),
            alert_manager: AlertManager::new(),
            monitoring_stats: MonitoringStats {
                total_health_checks: 0,
                failed_health_checks: 0,
                total_metrics_collected: 0,
                alerts_triggered: 0,
                average_health_check_time: 0.0,
                last_check_time: None,
            },
        }
    }

    /// Initialize the service monitor
    pub fn init(&self) -> ServiceResult<()> {
        self.metrics_collector.init()?;
        self.alert_manager.init()?;
        
        info!("Service monitor initialized");
        Ok(())
    }

    /// Start monitoring a service
    pub fn start_monitoring(&self, service_handle: super::service::ServiceHandle) -> ServiceResult<()> {
        let service_id = service_handle.get_service_id();
        
        let health_checker = HealthChecker {
            service_id,
            check_interval: 30000, // 30 seconds
            timeout: 5000, // 5 seconds
            consecutive_failures: 0,
            last_check_time: None,
            health_status: HealthStatus::Unknown,
            custom_health_checks: Vec::new(),
        };

        let mut checkers = self.health_checkers.write();
        checkers.insert(service_id, health_checker);

        // Load monitoring configuration if available
        if let Ok(config) = super::super::SERVICE_MANAGER.lock()
            .as_ref()
            .and_then(|manager| manager.config_manager.get_config(service_id).ok())
            .and_then(|config| config.monitoring.clone())
        {
            let mut configs = self.monitoring_config.write();
            configs.insert(service_id, config);
        }

        info!("Started monitoring service: {}", service_id.0);
        Ok(())
    }

    /// Stop monitoring a service
    pub fn stop_monitoring(&self, service_id: ServiceId) -> ServiceResult<()> {
        let mut checkers = self.health_checkers.write();
        checkers.remove(&service_id);

        let mut configs = self.monitoring_config.write();
        configs.remove(&service_id);

        info!("Stopped monitoring service: {}", service_id.0);
        Ok(())
    }

    /// Perform health check on a specific service
    pub fn check_health(&self, service_id: ServiceId) -> ServiceResult<HealthCheckResult> {
        let start_time = get_current_time();

        let checkers = self.health_checkers.read();
        let checker = checkers.get(&service_id).ok_or(ServiceError::ServiceNotFound)?;

        // Perform the health check
        let result = self.perform_health_check(checker)?;

        // Update checker state
        let mut checkers_mut = self.health_checkers.write();
        if let Some(checker) = checkers_mut.get_mut(&service_id) {
            checker.last_check_time = Some(get_current_time());
            checker.health_status = if result.healthy { HealthStatus::Healthy } else { HealthStatus::Unhealthy };
            
            if result.healthy {
                checker.consecutive_failures = 0;
            } else {
                checker.consecutive_failures += 1;
            }
        }

        // Update statistics
        self.update_health_check_stats(start_time, result.healthy);

        // Trigger alerts if necessary
        if !result.healthy {
            self.alert_manager.trigger_alert(service_id, "Health check failed".to_string())?;
        }

        Ok(result)
    }

    /// Get service health report
    pub fn get_health_report(&self, service_id: ServiceId) -> ServiceResult<ServiceHealthReport> {
        let checkers = self.health_checkers.read();
        let checker = checkers.get(&service_id).ok_or(ServiceError::ServiceNotFound)?;

        let recent_checks = self.get_recent_health_checks(service_id, 10)?;
        let metrics = self.metrics_collector.get_latest_metrics(service_id);
        let alerts = self.alert_manager.get_active_alerts_for_service(service_id);

        let overall_health = if checker.consecutive_failures > 3 {
            HealthStatus::Unhealthy
        } else if checker.consecutive_failures > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let uptime = checker.last_check_time.map(|last| get_current_time() - last).unwrap_or(0);
        let availability = self.calculate_availability(&recent_checks);

        Ok(ServiceHealthReport {
            service_id,
            overall_health,
            last_health_check: recent_checks.first().cloned(),
            recent_health_checks: recent_checks,
            metrics,
            alerts,
            uptime,
            availability,
        })
    }

    /// Get all service health reports
    pub fn get_all_health_reports(&self) -> Vec<ServiceHealthReport> {
        let checkers = self.health_checkers.read();
        
        checkers.keys()
            .filter_map(|service_id| self.get_health_report(*service_id).ok())
            .collect()
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> &MonitoringStats {
        &self.monitoring_stats
    }

    /// Get health check count (for overall service manager stats)
    pub fn get_check_count(&self) -> u64 {
        self.monitoring_stats.total_health_checks
    }

    /// Internal methods
    fn perform_health_check(&self, checker: &HealthChecker) -> ServiceResult<HealthCheckResult> {
        let start_time = get_current_time();

        // For now, simulate health check
        // In a real implementation, this would perform actual checks based on service type
        let healthy = true; // Simplified for this implementation
        let response_time = get_current_time() - start_time;

        Ok(HealthCheckResult {
            service_id: checker.service_id,
            healthy,
            response_time,
            error_message: if healthy { None } else Some("Health check failed".to_string()),
            timestamp: get_current_time(),
        })
    }

    fn get_recent_health_checks(&self, service_id: ServiceId, count: usize) -> ServiceResult<Vec<HealthCheckResult>> {
        // In a real implementation, this would retrieve recent health check results from storage
        Ok(Vec::new())
    }

    fn calculate_availability(&self, health_checks: &[HealthCheckResult]) -> f32 {
        if health_checks.is_empty() {
            return 0.0;
        }

        let healthy_count = health_checks.iter().filter(|check| check.healthy).count();
        (healthy_count as f32 / health_checks.len() as f32) * 100.0
    }

    fn update_health_check_stats(&self, start_time: u64, healthy: bool) {
        let response_time = get_current_time() - start_time;
        
        self.monitoring_stats.total_health_checks += 1;
        if !healthy {
            self.monitoring_stats.failed_health_checks += 1;
        }
        
        // Update running average
        let total_checks = self.monitoring_stats.total_health_checks as f64;
        let current_avg = self.monitoring_stats.average_health_check_time;
        let new_avg = (current_avg * (total_checks - 1.0) + response_time as f64) / total_checks;
        self.monitoring_stats.average_health_check_time = new_avg;
        
        self.monitoring_stats.last_check_time = Some(unsafe { crate::hal::get_current_time() });
    }
}

impl HealthChecker {
    /// Add a custom health check
    pub fn add_custom_check(&mut self, check: CustomHealthCheck) -> ServiceResult<()> {
        self.custom_health_checks.push(check);
        Ok(())
    }

    /// Get current health status
    pub fn get_health_status(&self) -> HealthStatus {
        self.health_status
    }

    /// Check if service is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status == HealthStatus::Healthy
    }

    /// Get consecutive failure count
    pub fn get_failure_count(&self) -> u32 {
        self.consecutive_failures
    }
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        MetricsCollector {
            metrics_store: RwLock::new(BTreeMap::new()),
            collection_interval: 60000, // 1 minute
            max_metrics_per_service: 1000,
        }
    }

    /// Initialize metrics collector
    pub fn init(&self) -> ServiceResult<()> {
        info!("Metrics collector initialized");
        Ok(())
    }

    /// Collect metrics for a service
    pub fn collect_metrics(&self, service_id: ServiceId, metrics: ServiceMetrics) -> ServiceResult<()> {
        let mut store = self.metrics_store.write();
        
        let metrics_queue = store.entry(service_id).or_insert_with(VecDeque::new);
        metrics_queue.push_back(metrics);
        
        // Maintain max size
        while metrics_queue.len() > self.max_metrics_per_service {
            metrics_queue.pop_front();
        }
        
        Ok(())
    }

    /// Get latest metrics for a service
    pub fn get_latest_metrics(&self, service_id: ServiceId) -> Option<ServiceMetrics> {
        let store = self.metrics_store.read();
        store.get(&service_id).and_then(|queue| queue.back().cloned())
    }

    /// Get metrics history for a service
    pub fn get_metrics_history(&self, service_id: ServiceId, count: usize) -> Vec<ServiceMetrics> {
        let store = self.metrics_store.read();
        if let Some(queue) = store.get(&service_id) {
            queue.iter().rev().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        AlertManager {
            active_alerts: RwLock::new(BTreeMap::new()),
            alert_rules: RwLock::new(Vec::new()),
            alert_history: RwLock::new(VecDeque::new()),
        }
    }

    /// Initialize alert manager
    pub fn init(&self) -> ServiceResult<()> {
        info!("Alert manager initialized");
        Ok(())
    }

    /// Trigger an alert
    pub fn trigger_alert(&self, service_id: ServiceId, message: String) -> ServiceResult<()> {
        let alert_id = format!("alert-{}-{}-{}", service_id.0, crate::service_manager::syscall::get_current_time(), 
                              crate::service_manager::syscall::generate_random_id());
        
        let alert = Alert {
            id: alert_id.clone(),
            service_id,
            rule_name: "Manual Alert".to_string(),
            severity: AlertSeverity::Warning,
            state: AlertState::Active,
            message,
            created_at: unsafe { crate::hal::get_current_time() },
            resolved_at: None,
            acknowledged_by: None,
        };

        let mut active_alerts = self.active_alerts.write();
        active_alerts.insert(alert_id, alert.clone());

        let mut history = self.alert_history.write();
        history.push_back(AlertHistoryEntry {
            alert_id: alert_id.clone(),
            event_type: AlertEventType::Triggered,
            timestamp: get_current_time(),
            details: format!("Alert triggered for service {}", service_id.0),
        });

        // Maintain history size
        while history.len() > 1000 {
            history.pop_front();
        }

        info!("Alert triggered: {} for service {}", alert_id, service_id.0);
        Ok(())
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: &str) -> ServiceResult<()> {
        let mut active_alerts = self.active_alerts.write();
        
        if let Some(mut alert) = active_alerts.remove(alert_id) {
            alert.state = AlertState::Resolved;
            alert.resolved_at = Some(unsafe { crate::hal::get_current_time() });

            let mut history = self.alert_history.write();
            history.push_back(AlertHistoryEntry {
                alert_id: alert_id.to_string(),
                event_type: AlertEventType::Resolved,
                timestamp: get_current_time(),
                details: format!("Alert {} resolved", alert_id),
            });

            info!("Alert resolved: {}", alert_id);
            Ok(())
        } else {
            Err(ServiceError::ServiceNotFound)
        }
    }

    /// Get active alerts for a service
    pub fn get_active_alerts_for_service(&self, service_id: ServiceId) -> Vec<Alert> {
        let active_alerts = self.active_alerts.read();
        active_alerts.values()
            .filter(|alert| alert.service_id == service_id && alert.state == AlertState::Active)
            .cloned()
            .collect()
    }

    /// Add alert rule
    pub fn add_alert_rule(&self, rule: AlertRule) -> ServiceResult<()> {
        let mut rules = self.alert_rules.write();
        rules.push(rule);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = ServiceMonitor::new();
        assert_eq!(monitor.get_stats().total_health_checks, 0);
        assert_eq!(monitor.get_stats().failed_health_checks, 0);
    }

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker {
            service_id: ServiceId(1),
            check_interval: 30000,
            timeout: 5000,
            consecutive_failures: 0,
            last_check_time: None,
            health_status: HealthStatus::Unknown,
            custom_health_checks: Vec::new(),
        };

        assert_eq!(checker.get_health_status(), HealthStatus::Unknown);
        assert_eq!(checker.get_failure_count(), 0);
        assert!(!checker.is_healthy());
    }

    #[test]
    fn test_health_check_types() {
        assert_eq!(HealthCheckType::Http as u8, 0);
        assert_eq!(HealthCheckType::Tcp as u8, 1);
        assert_eq!(HealthCheckType::Custom as u8, 5);
    }

    #[test]
    fn test_alert_severity_levels() {
        assert_eq!(AlertSeverity::Info as u8, 0);
        assert_eq!(AlertSeverity::Warning as u8, 1);
        assert_eq!(AlertSeverity::Critical as u8, 3);
    }

    #[test]
    fn test_availability_calculation() {
        let monitor = ServiceMonitor::new();
        
        let checks = vec![
            HealthCheckResult {
                service_id: ServiceId(1),
                healthy: true,
                response_time: 100,
                error_message: None,
                timestamp: 1000,
            },
            HealthCheckResult {
                service_id: ServiceId(1),
                healthy: true,
                response_time: 100,
                error_message: None,
                timestamp: 2000,
            },
            HealthCheckResult {
                service_id: ServiceId(1),
                healthy: false,
                response_time: 100,
                error_message: Some("Failed".to_string()),
                timestamp: 3000,
            },
        ];
        
        let availability = monitor.calculate_availability(&checks);
        assert_eq!(availability, 66.666664); // 2/3 = 66.67%
    }
}