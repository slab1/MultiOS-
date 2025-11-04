//! System Monitoring and Health Checking Service
//!
//! Provides comprehensive system monitoring, health checking, alerting,
//! and performance metrics collection.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::{RwLock, Mutex};
use core::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, AtomicU8, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use core::time::Duration;

/// Monitoring service initialization
pub fn init() -> Result<()> {
    info!("Initializing System Monitoring Service...");
    
    // Initialize monitoring framework
    init_monitoring_framework()?;
    
    // Initialize health checks
    init_health_checks()?;
    
    // Initialize performance metrics
    init_performance_metrics()?;
    
    // Initialize alerting system
    init_alerting_system()?;
    
    // Start monitoring services
    start_monitoring_services()?;
    
    info!("System Monitoring Service initialized");
    Ok(())
}

/// Monitoring service shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down System Monitoring Service...");
    
    // Stop monitoring services
    stop_monitoring_services()?;
    
    // Shutdown alerting system
    shutdown_alerting_system()?;
    
    // Shutdown performance metrics
    shutdown_performance_metrics()?;
    
    // Shutdown health checks
    shutdown_health_checks()?;
    
    // Shutdown monitoring framework
    shutdown_monitoring_framework()?;
    
    info!("System Monitoring Service shutdown complete");
    Ok(())
}

/// Monitoring types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MonitorType {
    SystemHealth = 0,
    PerformanceMetrics = 1,
    ResourceUsage = 2,
    ApplicationHealth = 3,
    SecurityMetrics = 4,
    CustomMetrics = 5,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertSeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
    Emergency = 4,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlertStatus {
    Active = 0,
    Acknowledged = 1,
    Resolved = 2,
    Suppressed = 3,
}

/// Metric data
#[derive(Debug, Clone)]
pub struct MetricData {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

/// System metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_io_mb_s: f64,
    pub process_count: usize,
    pub thread_count: usize,
    pub uptime_seconds: u64,
    pub load_average: f64,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub details: String,
    pub timestamp: u64,
    pub response_time_ms: u64,
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HealthStatus {
    Healthy = 0,
    Degraded = 1,
    Unhealthy = 2,
    Unknown = 3,
}

/// Alert information
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: u64,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub component: String,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub resolved_at: Option<u64>,
}

/// Performance threshold
#[derive(Debug, Clone)]
pub struct PerformanceThreshold {
    pub metric_name: String,
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub operator: ThresholdOperator,
}

/// Threshold operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThresholdOperator {
    GreaterThan = 0,
    LessThan = 1,
    EqualTo = 2,
    NotEqualTo = 3,
}

/// Performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub timestamp: u64,
    pub duration_seconds: u64,
    pub metrics: Vec<MetricData>,
    pub alerts: Vec<Alert>,
    pub recommendations: Vec<String>,
}

/// Monitoring service statistics
#[derive(Debug, Clone, Copy)]
pub struct MonitoringServiceStats {
    pub metrics_collected: AtomicU64,
    pub health_checks_performed: AtomicU64,
    pub alerts_generated: AtomicU64,
    pub alerts_resolved: AtomicU64,
    pub performance_violations: AtomicU64,
    pub monitoring_uptime_seconds: AtomicU64,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enable_system_monitoring: bool,
    pub enable_performance_monitoring: bool,
    pub enable_health_checks: bool,
    pub enable_alerting: bool,
    pub collection_interval_ms: u64,
    pub retention_period_days: u64,
    pub max_metrics_per_component: usize,
    pub alert_cooldown_seconds: u64,
}

/// Global monitoring configuration
static MONITORING_CONFIG: RwLock<MonitoringConfig> = RwLock::new(MonitoringConfig {
    enable_system_monitoring: true,
    enable_performance_monitoring: true,
    enable_health_checks: true,
    enable_alerting: true,
    collection_interval_ms: 5000, // 5 seconds
    retention_period_days: 30,
    max_metrics_per_component: 1000,
    alert_cooldown_seconds: 300, // 5 minutes
});

/// System metrics history
static SYSTEM_METRICS_HISTORY: RwLock<Vec<SystemMetrics>> = RwLock::new(Vec::new());

/// Current system load
static CURRENT_SYSTEM_LOAD: AtomicU8 = AtomicU8::new(0);

/// Health check results
static HEALTH_CHECK_RESULTS: RwLock<Vec<HealthCheckResult>> = RwLock::new(Vec::new());

/// Alert history
static ALERT_HISTORY: RwLock<Vec<Alert>> = RwLock::new(Vec::new());

/// Performance thresholds
static PERFORMANCE_THRESHOLDS: RwLock<Vec<PerformanceThreshold>> = RwLock::new(Vec::new());

/// Metric data storage
static METRIC_STORAGE: RwLock<Vec<MetricData>> = RwLock::new(Vec::new());

/// Monitoring service statistics
static MONITORING_STATS: MonitoringServiceStats = MonitoringServiceStats {
    metrics_collected: AtomicU64::new(0),
    health_checks_performed: AtomicU64::new(0),
    alerts_generated: AtomicU64::new(0),
    alerts_resolved: AtomicU64::new(0),
    performance_violations: AtomicU64::new(0),
    monitoring_uptime_seconds: AtomicU64::new(0),
};

/// Next alert ID
static NEXT_ALERT_ID: AtomicU64 = AtomicU64::new(1);

/// Initialize monitoring framework
fn init_monitoring_framework() -> Result<()> {
    info!("Initializing monitoring framework...");
    
    // Set up metric storage
    init_metric_storage()?;
    
    // Initialize system metrics tracking
    init_system_metrics_tracking()?;
    
    Ok(())
}

/// Initialize health checks
fn init_health_checks() -> Result<()> {
    info!("Initializing health checks...");
    
    // Register default health checks
    register_default_health_checks()?;
    
    Ok(())
}

/// Initialize performance metrics
fn init_performance_metrics() -> Result<()> {
    info!("Initializing performance metrics...");
    
    // Set up performance thresholds
    setup_performance_thresholds()?;
    
    Ok(())
}

/// Initialize alerting system
fn init_alerting_system() -> Result<()> {
    info!("Initializing alerting system...");
    
    // Initialize alert cooldown tracking
    init_alert_cooldowns()?;
    
    Ok(())
}

/// Start monitoring services
fn start_monitoring_services() -> Result<()> {
    info!("Starting monitoring services...");
    
    // Start metrics collection
    start_metrics_collection()?;
    
    // Start health checks
    start_health_checks()?;
    
    // Start performance monitoring
    start_performance_monitoring()?;
    
    Ok(())
}

/// Stop monitoring services
fn stop_monitoring_services() -> Result<()> {
    info!("Stopping monitoring services...");
    
    // Stop performance monitoring
    stop_performance_monitoring()?;
    
    // Stop health checks
    stop_health_checks()?;
    
    // Stop metrics collection
    stop_metrics_collection()?;
    
    Ok(())
}

/// Shutdown alerting system
fn shutdown_alerting_system() -> Result<()> {
    info!("Shutting down alerting system...");
    
    let mut alerts = ALERT_HISTORY.write();
    alerts.clear();
    
    Ok(())
}

/// Shutdown performance metrics
fn shutdown_performance_metrics() -> Result<()> {
    info!("Shutting down performance metrics...");
    
    let mut thresholds = PERFORMANCE_THRESHOLDS.write();
    thresholds.clear();
    
    Ok(())
}

/// Shutdown health checks
fn shutdown_health_checks() -> Result<()> {
    info!("Shutting down health checks...");
    
    let mut health_results = HEALTH_CHECK_RESULTS.write();
    health_results.clear();
    
    Ok(())
}

/// Shutdown monitoring framework
fn shutdown_monitoring_framework() -> Result<()> {
    info!("Shutting down monitoring framework...");
    
    let mut metrics = METRIC_STORAGE.write();
    metrics.clear();
    
    let mut system_metrics = SYSTEM_METRICS_HISTORY.write();
    system_metrics.clear();
    
    Ok(())
}

/// Initialize metric storage
fn init_metric_storage() -> Result<()> {
    info!("Initializing metric storage...");
    
    let mut storage = METRIC_STORAGE.write();
    storage.clear();
    
    Ok(())
}

/// Initialize system metrics tracking
fn init_system_metrics_tracking() -> Result<()> {
    info!("Initializing system metrics tracking...");
    
    let mut history = SYSTEM_METRICS_HISTORY.write();
    history.clear();
    
    Ok(())
}

/// Register default health checks
fn register_default_health_checks() -> Result<()> {
    info!("Registering default health checks...");
    
    // The actual health checks will be performed in the callback functions
    
    Ok(())
}

/// Setup performance thresholds
fn setup_performance_thresholds() -> Result<()> {
    info!("Setting up performance thresholds...");
    
    let mut thresholds = PERFORMANCE_THRESHOLDS.write();
    
    // CPU usage thresholds
    thresholds.push(PerformanceThreshold {
        metric_name: "cpu_usage_percent".to_string(),
        warning_threshold: 70.0,
        critical_threshold: 90.0,
        operator: ThresholdOperator::GreaterThan,
    });
    
    // Memory usage thresholds
    thresholds.push(PerformanceThreshold {
        metric_name: "memory_usage_percent".to_string(),
        warning_threshold: 80.0,
        critical_threshold: 95.0,
        operator: ThresholdOperator::GreaterThan,
    });
    
    // Disk usage thresholds
    thresholds.push(PerformanceThreshold {
        metric_name: "disk_usage_percent".to_string(),
        warning_threshold: 85.0,
        critical_threshold: 95.0,
        operator: ThresholdOperator::GreaterThan,
    });
    
    // Network I/O thresholds
    thresholds.push(PerformanceThreshold {
        metric_name: "network_io_mb_s".to_string(),
        warning_threshold: 100.0,
        critical_threshold: 500.0,
        operator: ThresholdOperator::GreaterThan,
    });
    
    info!("Performance thresholds configured");
    
    Ok(())
}

/// Initialize alert cooldowns
fn init_alert_cooldowns() -> Result<()> {
    info!("Initializing alert cooldowns...");
    
    // Alert cooldown tracking is handled by checking timestamps
    
    Ok(())
}

/// Start metrics collection
fn start_metrics_collection() -> Result<()> {
    info!("Starting metrics collection...");
    
    let config = MONITORING_CONFIG.read();
    
    // Create metrics collection timer
    let _ = crate::services::time_service::create_timer(
        crate::services::time_service::TimerType::Periodic,
        config.collection_interval_ms * 1_000_000, // Convert ms to ns
        metrics_collection_callback
    );
    
    Ok(())
}

/// Start health checks
fn start_health_checks() -> Result<()> {
    info!("Starting health checks...");
    
    // Create health check timer
    let _ = crate::services::time_service::create_timer(
        crate::services::time_service::TimerType::Periodic,
        10_000_000_000, // 10 seconds
        health_check_callback
    );
    
    Ok(())
}

/// Start performance monitoring
fn start_performance_monitoring() -> Result<()> {
    info!("Starting performance monitoring...");
    
    // Create performance monitoring timer
    let _ = crate::services::time_service::create_timer(
        crate::services::time_service::TimerType::Periodic,
        5_000_000_000, // 5 seconds
        performance_monitoring_callback
    );
    
    Ok(())
}

/// Stop metrics collection
fn stop_metrics_collection() -> Result<()> {
    info!("Stopping metrics collection...");
    
    Ok(())
}

/// Stop health checks
fn stop_health_checks() -> Result<()> {
    info!("Stopping health checks...");
    
    Ok(())
}

/// Stop performance monitoring
fn stop_performance_monitoring() -> Result<()> {
    info!("Stopping performance monitoring...");
    
    Ok(())
}

/// Update system metrics
pub fn update_system_metrics() -> Result<()> {
    info!("Updating system metrics...");
    
    let start_time = crate::hal::timers::get_high_res_time();
    
    // Collect system metrics
    let metrics = collect_system_metrics()?;
    
    // Store metrics
    {
        let mut history = SYSTEM_METRICS_HISTORY.write();
        history.push(metrics.clone());
        
        // Limit history size
        let max_history = 1000; // Keep last 1000 samples
        if history.len() > max_history {
            history.remove(0);
        }
    }
    
    // Store individual metrics
    store_metric("cpu_usage_percent", metrics.cpu_usage_percent, "%")?;
    store_metric("memory_usage_percent", metrics.memory_usage_percent, "%")?;
    store_metric("disk_usage_percent", metrics.disk_usage_percent, "%")?;
    store_metric("network_io_mb_s", metrics.network_io_mb_s, "MB/s")?;
    store_metric("process_count", metrics.process_count as f64, "count")?;
    store_metric("thread_count", metrics.thread_count as f64, "count")?;
    store_metric("uptime_seconds", metrics.uptime_seconds as f64, "s")?;
    store_metric("load_average", metrics.load_average, "load")?;
    
    MONITORING_STATS.metrics_collected.fetch_add(8, Ordering::SeqCst); // 8 metrics
    
    let end_time = crate::hal::timers::get_high_res_time();
    let collection_time = end_time - start_time;
    
    info!("System metrics updated in {} ns", collection_time);
    
    Ok(())
}

/// Collect system metrics
fn collect_system_metrics() -> Result<SystemMetrics> {
    // Collect CPU usage
    let cpu_usage_percent = collect_cpu_usage()?;
    
    // Collect memory usage
    let memory_usage_percent = collect_memory_usage()?;
    
    // Collect disk usage
    let disk_usage_percent = collect_disk_usage()?;
    
    // Collect network I/O
    let network_io_mb_s = collect_network_io()?;
    
    // Collect process/thread counts
    let (process_count, thread_count) = collect_process_info()?;
    
    // Collect uptime
    let uptime_seconds = crate::services::time_service::get_uptime_ns() / 1_000_000_000;
    
    // Calculate load average
    let load_average = calculate_load_average()?;
    
    Ok(SystemMetrics {
        cpu_usage_percent,
        memory_usage_percent,
        disk_usage_percent,
        network_io_mb_s,
        process_count,
        thread_count,
        uptime_seconds,
        load_average,
    })
}

/// Collect CPU usage
fn collect_cpu_usage() -> Result<f64> {
    // Simulate CPU usage collection
    // In real implementation, this would read from /proc/stat or similar
    
    let usage = crate::services::random_service::utils::random_u32_in_range(10, 80) as f64;
    
    CURRENT_SYSTEM_LOAD.store(usage as u8, Ordering::SeqCst);
    
    Ok(usage)
}

/// Collect memory usage
fn collect_memory_usage() -> Result<f64> {
    // Simulate memory usage collection
    let usage = crate::services::random_service::utils::random_u32_in_range(30, 70) as f64;
    
    Ok(usage)
}

/// Collect disk usage
fn collect_disk_usage() -> Result<f64> {
    // Simulate disk usage collection
    let usage = crate::services::random_service::utils::random_u32_in_range(40, 80) as f64;
    
    Ok(usage)
}

/// Collect network I/O
fn collect_network_io() -> Result<f64> {
    // Simulate network I/O collection
    let io_mb_s = crate::services::random_service::utils::random_u32_in_range(0, 200) as f64;
    
    Ok(io_mb_s)
}

/// Collect process information
fn collect_process_info() -> Result<(usize, usize)> {
    // Simulate process/thread count collection
    let process_count = crate::services::random_service::utils::random_u32_in_range(50, 200) as usize;
    let thread_count = process_count * crate::services::random_service::utils::random_u32_in_range(1, 4);
    
    Ok((process_count, thread_count))
}

/// Calculate load average
fn calculate_load_average() -> Result<f64> {
    // Simplified load average calculation
    let cpu_usage = CURRENT_SYSTEM_LOAD.load(Ordering::SeqCst) as f64;
    let load = cpu_usage / 100.0;
    
    Ok(load)
}

/// Store metric
fn store_metric(name: &str, value: f64, unit: &str) -> Result<()> {
    let metric = MetricData {
        name: name.to_string(),
        value,
        unit: unit.to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        tags: Vec::new(),
    };
    
    let mut storage = METRIC_STORAGE.write();
    storage.push(metric);
    
    // Limit storage size
    let config = MONITORING_CONFIG.read();
    if storage.len() > config.max_metrics_per_component {
        storage.remove(0);
    }
    
    Ok(())
}

/// Perform health checks
pub fn perform_health_checks() -> Result<()> {
    info!("Performing health checks...");
    
    let start_time = crate::hal::timers::get_high_res_time();
    
    // Perform various health checks
    let cpu_health = check_cpu_health()?;
    let memory_health = check_memory_health()?;
    let disk_health = check_disk_health()?;
    let network_health = check_network_health()?;
    let power_health = check_power_health()?;
    
    // Store results
    let results = vec![cpu_health, memory_health, disk_health, network_health, power_health];
    
    {
        let mut health_results = HEALTH_CHECK_RESULTS.write();
        health_results.clear();
        health_results.extend(results);
    }
    
    MONITORING_STATS.health_checks_performed.fetch_add(1, Ordering::SeqCst);
    
    let end_time = crate::hal::timers::get_high_res_time();
    let check_time = (end_time - start_time) / 1_000_000; // Convert to ms
    
    info!("Health checks completed in {} ms", check_time);
    
    Ok(())
}

/// Check CPU health
fn check_cpu_health() -> Result<HealthCheckResult> {
    let cpu_usage = collect_cpu_usage()?;
    
    let status = if cpu_usage < 70.0 {
        HealthStatus::Healthy
    } else if cpu_usage < 90.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    let message = format!("CPU usage: {:.1}%", cpu_usage);
    
    Ok(HealthCheckResult {
        component: "cpu".to_string(),
        status,
        message,
        details: "CPU performance monitoring".to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        response_time_ms: 0,
    })
}

/// Check memory health
fn check_memory_health() -> Result<HealthCheckResult> {
    let memory_usage = collect_memory_usage()?;
    
    let status = if memory_usage < 80.0 {
        HealthStatus::Healthy
    } else if memory_usage < 95.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    let message = format!("Memory usage: {:.1}%", memory_usage);
    
    Ok(HealthCheckResult {
        component: "memory".to_string(),
        status,
        message,
        details: "Memory usage monitoring".to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        response_time_ms: 0,
    })
}

/// Check disk health
fn check_disk_health() -> Result<HealthCheckResult> {
    let disk_usage = collect_disk_usage()?;
    
    let status = if disk_usage < 85.0 {
        HealthStatus::Healthy
    } else if disk_usage < 95.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    let message = format!("Disk usage: {:.1}%", disk_usage);
    
    Ok(HealthCheckResult {
        component: "disk".to_string(),
        status,
        message,
        details: "Disk space monitoring".to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        response_time_ms: 0,
    })
}

/// Check network health
fn check_network_health() -> Result<HealthCheckResult> {
    let network_io = collect_network_io()?;
    
    let status = if network_io < 100.0 {
        HealthStatus::Healthy
    } else if network_io < 500.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };
    
    let message = format!("Network I/O: {:.1} MB/s", network_io);
    
    Ok(HealthCheckResult {
        component: "network".to_string(),
        status,
        message,
        details: "Network I/O monitoring".to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        response_time_ms: 0,
    })
}

/// Check power health
fn check_power_health() -> Result<HealthCheckResult> {
    // This would check power management status
    // For now, return healthy status
    
    Ok(HealthCheckResult {
        component: "power".to_string(),
        status: HealthStatus::Healthy,
        message: "Power management healthy".to_string(),
        details: "Power supply and battery monitoring".to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        response_time_ms: 0,
    })
}

/// Monitor performance
pub fn monitor_performance() -> Result<()> {
    info!("Monitoring performance...");
    
    // Check performance thresholds
    check_performance_thresholds()?;
    
    // Generate performance alerts if needed
    check_alert_conditions()?;
    
    MONITORING_STATS.performance_violations.fetch_add(1, Ordering::SeqCst);
    
    Ok(())
}

/// Check performance thresholds
fn check_performance_thresholds() -> Result<()> {
    let thresholds = PERFORMANCE_THRESHOLDS.read();
    let storage = METRIC_STORAGE.read();
    
    for threshold in thresholds.iter() {
        // Find latest metric value
        if let Some(&metric) = storage.iter()
            .filter(|m| m.name == threshold.metric_name)
            .last()
        {
            check_threshold_violation(&threshold, &metric)?;
        }
    }
    
    Ok(())
}

/// Check threshold violation
fn check_threshold_violation(threshold: &PerformanceThreshold, metric: &MetricData) -> Result<()> {
    let mut should_alert = false;
    let mut severity = AlertSeverity::Info;
    
    match threshold.operator {
        ThresholdOperator::GreaterThan => {
            if metric.value > threshold.critical_threshold {
                should_alert = true;
                severity = AlertSeverity::Critical;
            } else if metric.value > threshold.warning_threshold {
                should_alert = true;
                severity = AlertSeverity::Warning;
            }
        }
        ThresholdOperator::LessThan => {
            if metric.value < threshold.critical_threshold {
                should_alert = true;
                severity = AlertSeverity::Critical;
            } else if metric.value < threshold.warning_threshold {
                should_alert = true;
                severity = AlertSeverity::Warning;
            }
        }
        _ => {
            // Other operators not implemented for simplicity
        }
    }
    
    if should_alert {
        generate_alert(severity, &threshold.metric_name, &format!(
            "Metric '{}' value {:.1} {} exceeded threshold", 
            threshold.metric_name, metric.value, metric.unit
        ))?;
    }
    
    Ok(())
}

/// Check alert conditions
fn check_alert_conditions() -> Result<()> {
    // Check for system overload
    let system_load = CURRENT_SYSTEM_LOAD.load(Ordering::SeqCst);
    
    if system_load > 90 {
        generate_alert(AlertSeverity::Critical, "system", "System overload detected")?;
    } else if system_load > 70 {
        generate_alert(AlertSeverity::Warning, "system", "High system load")?;
    }
    
    Ok(())
}

/// Generate alert
fn generate_alert(severity: AlertSeverity, component: &str, message: &str) -> Result<()> {
    let alert_id = NEXT_ALERT_ID.fetch_add(1, Ordering::SeqCst);
    
    let alert = Alert {
        id: alert_id,
        severity,
        status: AlertStatus::Active,
        component: component.to_string(),
        title: format!("Alert: {}", message),
        message: message.to_string(),
        timestamp: crate::services::time_service::get_uptime_ns(),
        resolved_at: None,
    };
    
    {
        let mut alerts = ALERT_HISTORY.write();
        alerts.push(alert);
        
        // Limit alert history
        let max_alerts = 1000;
        if alerts.len() > max_alerts {
            alerts.remove(0);
        }
    }
    
    MONITORING_STATS.alerts_generated.fetch_add(1, Ordering::SeqCst);
    
    info!("Alert generated: {} (ID: {})", message, alert_id);
    
    Ok(())
}

/// Metrics collection callback
fn metrics_collection_callback(_interval_ns: u64, _timer_type: crate::services::time_service::TimerType) {
    let _ = update_system_metrics();
}

/// Health check callback
fn health_check_callback(_interval_ns: u64, _timer_type: crate::services::time_service::TimerType) {
    let _ = perform_health_checks();
}

/// Performance monitoring callback
fn performance_monitoring_callback(_interval_ns: u64, _timer_type: crate::services::time_service::TimerType) {
    let _ = monitor_performance();
}

/// Get current system load
pub fn get_system_load() -> f64 {
    CURRENT_SYSTEM_LOAD.load(Ordering::SeqCst) as f64
}

/// Get system metrics history
pub fn get_system_metrics_history(limit: usize) -> Vec<SystemMetrics> {
    let history = SYSTEM_METRICS_HISTORY.read();
    let start_idx = if history.len() > limit {
        history.len() - limit
    } else {
        0
    };
    history[start_idx..].to_vec()
}

/// Get latest system metrics
pub fn get_latest_system_metrics() -> Option<SystemMetrics> {
    let history = SYSTEM_METRICS_HISTORY.read();
    history.last().cloned()
}

/// Get health check results
pub fn get_health_check_results() -> Vec<HealthCheckResult> {
    HEALTH_CHECK_RESULTS.read().clone()
}

/// Get active alerts
pub fn get_active_alerts() -> Vec<Alert> {
    ALERT_HISTORY.read()
        .iter()
        .filter(|alert| alert.status == AlertStatus::Active)
        .cloned()
        .collect()
}

/// Get all alerts
pub fn get_all_alerts() -> Vec<Alert> {
    ALERT_HISTORY.read().clone()
}

/// Get metrics by name
pub fn get_metrics_by_name(name: &str) -> Vec<MetricData> {
    METRIC_STORAGE.read()
        .iter()
        .filter(|metric| metric.name == name)
        .cloned()
        .collect()
}

/// Get all metrics
pub fn get_all_metrics() -> Vec<MetricData> {
    METRIC_STORAGE.read().clone()
}

/// Generate performance report
pub fn generate_performance_report(duration_seconds: u64) -> Result<PerformanceReport> {
    info!("Generating performance report for {} seconds", duration_seconds);
    
    let end_time = crate::services::time_service::get_uptime_ns();
    let start_time = end_time - (duration_seconds * 1_000_000_000);
    
    // Get metrics in time range
    let metrics = METRIC_STORAGE.read()
        .iter()
        .filter(|metric| metric.timestamp >= start_time)
        .cloned()
        .collect();
    
    // Get alerts in time range
    let alerts = ALERT_HISTORY.read()
        .iter()
        .filter(|alert| alert.timestamp >= start_time)
        .cloned()
        .collect();
    
    // Generate recommendations
    let recommendations = generate_performance_recommendations(&metrics);
    
    Ok(PerformanceReport {
        timestamp: end_time,
        duration_seconds,
        metrics,
        alerts,
        recommendations,
    })
}

/// Generate performance recommendations
fn generate_performance_recommendations(metrics: &[MetricData]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    // Analyze CPU usage
    if let Some(cpu_metrics) = metrics.iter().filter(|m| m.name == "cpu_usage_percent") {
        let avg_cpu: f64 = cpu_metrics.map(|m| m.value).sum::<f64>() / cpu_metrics.count() as f64;
        
        if avg_cpu > 80.0 {
            recommendations.push("Consider optimizing CPU-intensive processes or upgrading hardware".to_string());
        }
    }
    
    // Analyze memory usage
    if let Some(mem_metrics) = metrics.iter().filter(|m| m.name == "memory_usage_percent") {
        let avg_mem: f64 = mem_metrics.map(|m| m.value).sum::<f64>() / mem_metrics.count() as f64;
        
        if avg_mem > 85.0 {
            recommendations.push("Consider adding more RAM or optimizing memory usage".to_string());
        }
    }
    
    recommendations
}

/// Get monitoring service statistics
pub fn get_stats() -> MonitoringServiceStats {
    MONITORING_STATS
}

/// Benchmark monitoring service
pub fn benchmark_monitoring_service() -> Result<(u64, u64, u64)> {
    info!("Benchmarking monitoring service...");
    
    let mut metrics_collection_time = 0;
    let mut health_check_time = 0;
    let mut performance_monitoring_time = 0;
    
    // Benchmark metrics collection
    let start = crate::hal::timers::get_high_res_time();
    let _ = update_system_metrics();
    metrics_collection_time = crate::hal::timers::get_high_res_time() - start;
    
    // Benchmark health checks
    let start = crate::hal::timers::get_high_res_time();
    let _ = perform_health_checks();
    health_check_time = crate::hal::timers::get_high_res_time() - start;
    
    // Benchmark performance monitoring
    let start = crate::hal::timers::get_high_res_time();
    let _ = monitor_performance();
    performance_monitoring_time = crate::hal::timers::get_high_res_time() - start;
    
    Ok((metrics_collection_time, health_check_time, performance_monitoring_time))
}

/// Monitoring utility functions
pub mod utils {
    use super::*;
    
    /// Format health status as string
    pub fn format_health_status(status: HealthStatus) -> &'static str {
        match status {
            HealthStatus::Healthy => "Healthy",
            HealthStatus::Degraded => "Degraded",
            HealthStatus::Unhealthy => "Unhealthy",
            HealthStatus::Unknown => "Unknown",
        }
    }
    
    /// Format alert severity as string
    pub fn format_alert_severity(severity: AlertSeverity) -> &'static str {
        match severity {
            AlertSeverity::Info => "Info",
            AlertSeverity::Warning => "Warning",
            AlertSeverity::Error => "Error",
            AlertSeverity::Critical => "Critical",
            AlertSeverity::Emergency => "Emergency",
        }
    }
    
    /// Format metric value with unit
    pub fn format_metric_value(value: f64, unit: &str) -> String {
        format!("{:.2} {}", value, unit)
    }
    
    /// Calculate metric trend
    pub fn calculate_trend(metrics: &[MetricData]) -> TrendDirection {
        if metrics.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let first = metrics.first().unwrap().value;
        let last = metrics.last().unwrap().value;
        
        let change_percent = ((last - first) / first) * 100.0;
        
        if change_percent > 5.0 {
            TrendDirection::Increasing
        } else if change_percent < -5.0 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
    
    /// Calculate metric statistics
    pub fn calculate_metric_stats(metrics: &[MetricData]) -> MetricStatistics {
        if metrics.is_empty() {
            return MetricStatistics {
                count: 0,
                min: 0.0,
                max: 0.0,
                avg: 0.0,
                std_dev: 0.0,
            };
        }
        
        let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
        let count = values.len();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg: f64 = values.iter().sum::<f64>() / count as f64;
        
        let variance = values.iter()
            .map(|&x| (x - avg) * (x - avg))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();
        
        MetricStatistics {
            count,
            min,
            max,
            avg,
            std_dev,
        }
    }
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Metric statistics
#[derive(Debug, Clone, Copy)]
pub struct MetricStatistics {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub std_dev: f64,
}