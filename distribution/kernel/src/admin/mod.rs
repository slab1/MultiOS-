//! MultiOS Admin Module
//! 
//! This module provides comprehensive system administration functionality including
//! process and service management, system monitoring, and administrative operations.
//! 
//! It includes:
//! - User and group management
//! - Authentication and authorization
//! - Security policy management
//! - Audit and monitoring
//! - Process and service administration
//! - Administrative Shell Interface (admin_shell)
//! - Administrative API System (admin_api)

pub mod process_manager;
pub mod integration_examples;
pub mod user_manager;
pub mod security;
pub mod audit;
pub mod config;
pub mod policy;
pub mod monitoring;
pub mod admin_shell;
pub mod resource_monitor;

// System Configuration Management Framework
pub mod config_manager;
pub mod schema;
pub mod persistence;
pub mod validation;
pub mod backup;
pub mod propagation;

#[cfg(test)]
pub mod admin_shell_tests;
#[cfg(test)]
pub mod config_tests;

//! Administrative API System
//! 
//! This sub-module provides REST-like administrative APIs for system management,
//! including authentication, authorization, request validation, rate limiting,
//! and integration with the existing syscall interface.

pub mod admin_api;

// Re-export admin API types for convenience
pub use admin_api::{
    ApiResult, ApiRequest, ApiResponse, ApiData, ApiError,
    SystemInfo, ProcessInfo, MemoryInfo, ServiceInfo, PerformanceData,
    AdminApiServer, ApiConfig, Permission, AdminModule, AdminUtils,
    DEFAULT_ADMIN_API_CONFIG, ADMIN_API_BASE,
    init_admin_api, shutdown_admin_api, make_api_request,
};

use crate::log::{info, warn, error};
use crate::Result;
use crate::KernelError;

/// Admin module initialization
pub fn init() -> Result<()> {
    info!("Initializing Admin Module...");
    
    // Initialize process and service management
    process_manager::init_process_manager()?;
    
    // Initialize user management
    user_manager::init_user_manager()
        .map_err(|e| {
            error!("Failed to initialize user manager: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize security management
    security::init_security_manager()
        .map_err(|e| {
            error!("Failed to initialize security manager: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize audit system
    audit::init_audit_manager()
        .map_err(|e| {
            error!("Failed to initialize audit manager: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize configuration management
    config::init_config_manager()
        .map_err(|e| {
            error!("Failed to initialize config manager: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize policy management
    policy::init_policy_manager()
        .map_err(|e| {
            error!("Failed to initialize policy manager: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize new configuration management framework
    config_manager::init_config_manager()
        .map_err(|e| {
            error!("Failed to initialize system config manager: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize Administrative Shell
    admin_shell::init()
        .map_err(|e| {
            error!("Failed to initialize admin shell: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize Administrative API System
    let api_config = DEFAULT_ADMIN_API_CONFIG;
    admin_api::init_admin_api(api_config)
        .map_err(|e| {
            error!("Failed to initialize administrative API: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Initialize resource monitoring
    resource_monitor::init()
        .map_err(|e| {
            error!("Failed to initialize resource monitor: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    info!("Admin Module initialized successfully");
    Ok(())
}

/// Run integration examples (for testing and demonstration)
pub fn run_integration_examples() -> Result<()> {
    info!("Running integration examples...");
    integration_examples::run_all_examples()
}

/// Admin module shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down Admin Module...");
    
    // Shutdown Administrative API System first
    admin_api::shutdown_admin_api()
        .map_err(|e| {
            error!("Failed to shutdown administrative API: {:?}", e);
            KernelError::InitializationFailed
        })?;
    
    // Shutdown all components in reverse order
    resource_monitor::shutdown()
        .map_err(|e| {
            error!("Failed to shutdown resource monitor: {:?}", e);
            KernelError::InitializationFailed
        })?;
    admin_shell::shutdown()?;
    policy::shutdown_policy_manager()?;
    config::shutdown_config_manager()?;
    audit::shutdown_audit_manager()?;
    security::shutdown_security_manager()?;
    user_manager::shutdown_user_manager()?;
    process_manager::shutdown_process_manager()?;
    
    // Shutdown system configuration management framework
    // Note: The config_manager module doesn't have a shutdown function in the current implementation
    // This would be added in a complete implementation
    
    info!("Admin Module shutdown complete");
    Ok(())
}