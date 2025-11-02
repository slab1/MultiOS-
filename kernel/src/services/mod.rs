//! MultiOS System Services
//!
//! This module provides essential system services for MultiOS including:
//! - Time management (system time, time zones, timers)
//! - Random number generation (hardware and software RNG)
//! - I/O services (stdio, networking)
//! - Power management
//! - Service daemon framework for background services
//! - System monitoring and health checking services

pub mod time_service;
pub mod random_service;
pub mod io_service;
pub mod power_service;
pub mod daemon_service;
pub mod monitoring_service;
pub mod cli_service;
pub mod cli_script_interpreter;
pub mod cli_application;

use crate::{KernelError, Result};
use crate::log::{info, warn, error};

/// System services initialization
pub fn init() -> Result<()> {
    info!("Initializing System Services...");
    
    // Initialize services in dependency order
    time_service::init()?;
    random_service::init()?;
    io_service::init()?;
    power_service::init()?;
    daemon_service::init()?;
    monitoring_service::init()?;
    cli_service::init()?;
    
    info!("System Services initialization complete");
    Ok(())
}

/// System services shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down System Services...");
    
    // Shutdown services in reverse dependency order
    cli_service::shutdown()?;
    monitoring_service::shutdown()?;
    daemon_service::shutdown()?;
    power_service::shutdown()?;
    io_service::shutdown()?;
    random_service::shutdown()?;
    time_service::shutdown()?;
    
    info!("System Services shutdown complete");
    Ok(())
}

/// System services configuration
#[derive(Debug, Clone)]
pub struct SystemServicesConfig {
    pub enable_time_service: bool,
    pub enable_random_service: bool,
    pub enable_io_service: bool,
    pub enable_power_service: bool,
    pub enable_daemon_service: bool,
    pub enable_monitoring_service: bool,
    pub enable_cli_service: bool,
    pub max_daemons: usize,
    pub monitoring_interval_ms: u64,
    pub cli_history_size: usize,
    pub cli_completion_enabled: bool,
}

impl Default for SystemServicesConfig {
    fn default() -> Self {
        Self {
            enable_time_service: true,
            enable_random_service: true,
            enable_io_service: true,
            enable_power_service: true,
            enable_daemon_service: true,
            enable_monitoring_service: true,
            enable_cli_service: true,
            max_daemons: 64,
            monitoring_interval_ms: 1000,
            cli_history_size: 1000,
            cli_completion_enabled: true,
        }
    }
}

/// System services information
#[derive(Debug, Clone)]
pub struct SystemServicesInfo {
    pub initialized_services: Vec<String>,
    pub service_versions: Vec<(String, String)>,
    pub total_daemons: usize,
    pub uptime_ns: u64,
    pub system_load: f64,
}

/// Get system services information
pub fn get_services_info() -> Result<SystemServicesInfo> {
    Ok(SystemServicesInfo {
        initialized_services: vec![
            "time_service".to_string(),
            "random_service".to_string(),
            "io_service".to_string(),
            "power_service".to_string(),
            "daemon_service".to_string(),
            "monitoring_service".to_string(),
            "cli_service".to_string(),
        ],
        service_versions: vec![
            ("time_service".to_string(), "1.0.0".to_string()),
            ("random_service".to_string(), "1.0.0".to_string()),
            ("io_service".to_string(), "1.0.0".to_string()),
            ("power_service".to_string(), "1.0.0".to_string()),
            ("daemon_service".to_string(), "1.0.0".to_string()),
            ("monitoring_service".to_string(), "1.0.0".to_string()),
            ("cli_service".to_string(), "1.0.0".to_string()),
        ],
        total_daemons: daemon_service::get_daemon_count(),
        uptime_ns: time_service::get_uptime_ns(),
        system_load: monitoring_service::get_system_load(),
    })
}

/// System services statistics
#[derive(Debug, Clone)]
pub struct SystemServicesStats {
    pub time_stats: time_service::TimeServiceStats,
    pub random_stats: random_service::RandomServiceStats,
    pub io_stats: io_service::IoServiceStats,
    pub power_stats: power_service::PowerServiceStats,
    pub daemon_stats: daemon_service::DaemonServiceStats,
    pub monitoring_stats: monitoring_service::MonitoringServiceStats,
    pub cli_stats: cli_service::CliServiceStats,
}

/// Get system services statistics
pub fn get_services_stats() -> SystemServicesStats {
    SystemServicesStats {
        time_stats: time_service::get_stats(),
        random_stats: random_service::get_stats(),
        io_stats: io_service::get_stats(),
        power_stats: power_service::get_stats(),
        daemon_stats: daemon_service::get_stats(),
        monitoring_stats: monitoring_service::get_stats(),
        cli_stats: cli_service::get_stats(),
    }
}

/// System services error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemServiceError {
    ServiceNotInitialized,
    ServiceAlreadyInitialized,
    InvalidConfiguration,
    ResourceExhausted,
    PermissionDenied,
    OperationNotSupported,
    Timeout,
    HardwareNotAvailable,
    ServiceUnavailable,
}

impl From<SystemServiceError> for KernelError {
    fn from(_error: SystemServiceError) -> Self {
        KernelError::FeatureNotSupported
    }
}