//! Service Daemon Framework
//!
//! Provides a comprehensive framework for managing background services
//! and daemons in the MultiOS kernel.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::{RwLock, Mutex};
use core::sync::atomic::{AtomicU64, AtomicUsize, AtomicBool, AtomicU8, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use core::time::Duration;

/// Daemon service initialization
pub fn init() -> Result<()> {
    info!("Initializing Service Daemon Framework...");
    
    // Initialize daemon registry
    init_daemon_registry()?;
    
    // Initialize daemon scheduler
    init_daemon_scheduler()?;
    
    // Initialize daemon lifecycle management
    init_lifecycle_management()?;
    
    // Start daemon management services
    start_daemon_services()?;
    
    info!("Service Daemon Framework initialized");
    Ok(())
}

/// Daemon service shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down Service Daemon Framework...");
    
    // Stop daemon management services
    stop_daemon_services()?;
    
    // Shutdown lifecycle management
    shutdown_lifecycle_management()?;
    
    // Shutdown daemon scheduler
    shutdown_daemon_scheduler()?;
    
    // Shutdown daemon registry
    shutdown_daemon_registry()?;
    
    info!("Service Daemon Framework shutdown complete");
    Ok(())
}

/// Daemon types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DaemonType {
    SystemService = 0,
    UserService = 1,
    BackgroundTask = 2,
    PeriodicTask = 3,
    EventDriven = 4,
    NetworkService = 5,
    StorageService = 6,
    SecurityService = 7,
}

/// Daemon states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DaemonState {
    Stopped = 0,
    Starting = 1,
    Running = 2,
    Paused = 3,
    Stopping = 4,
    Error = 5,
    Disabled = 6,
}

/// Daemon priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PriorityLevel {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

/// Daemon information
#[derive(Debug, Clone)]
pub struct DaemonInfo {
    pub id: u64,
    pub name: String,
    pub daemon_type: DaemonType,
    pub description: String,
    pub version: String,
    pub priority: PriorityLevel,
    pub dependencies: Vec<u64>, // Other daemon IDs
    pub auto_restart: bool,
    pub max_restart_attempts: u32,
    pub state: DaemonState,
    pub created_at: u64,
    pub last_started_at: u64,
    pub run_count: u32,
}

/// Daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub name: String,
    pub daemon_type: DaemonType,
    pub description: String,
    pub version: String,
    pub priority: PriorityLevel,
    pub dependencies: Vec<String>,
    pub auto_restart: bool,
    pub max_restart_attempts: u32,
    pub initial_state: DaemonState,
    pub resource_limits: ResourceLimits,
}

/// Resource limits for daemons
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u32,
    pub max_disk_io_mb_s: u64,
    pub max_network_io_mb_s: u64,
    pub max_file_descriptors: u32,
}

/// Daemon execution context
#[derive(Debug)]
pub struct DaemonContext {
    pub daemon_id: u64,
    pub config: DaemonConfig,
    pub state: DaemonState,
    pub restart_count: u32,
    pub last_error: Option<String>,
    pub resources: DaemonResources,
}

/// Daemon resources
#[derive(Debug, Clone, Copy)]
pub struct DaemonResources {
    pub memory_used_mb: u64,
    pub cpu_usage_percent: u32,
    pub disk_io_mb_s: u64,
    pub network_io_mb_s: u64,
    pub file_descriptors: u32,
}

/// Daemon statistics
#[derive(Debug, Clone, Copy)]
pub struct DaemonStats {
    pub uptime_seconds: u64,
    pub cpu_time_seconds: u64,
    pub memory_peak_mb: u64,
    pub disk_io_total_mb: u64,
    pub network_io_total_mb: u64,
    pub restart_count: u32,
    pub error_count: u32,
    pub last_error_at: Option<u64>,
}

/// Daemon callback functions
pub type DaemonStartCallback = fn(&mut DaemonContext) -> Result<()>;
pub type DaemonStopCallback = fn(&mut DaemonContext) -> Result<()>;
pub type DaemonRunCallback = fn(&mut DaemonContext) -> Result<bool>; // Return true to continue
pub type DaemonErrorCallback = fn(&mut DaemonContext, &str) -> Result<()>;

/// Daemon operations
#[derive(Debug, Clone)]
pub struct DaemonOperations {
    pub start: DaemonStartCallback,
    pub stop: DaemonStopCallback,
    pub run: DaemonRunCallback,
    pub error_handler: Option<DaemonErrorCallback>,
}

/// Daemon service statistics
#[derive(Debug, Clone, Copy)]
pub struct DaemonServiceStats {
    pub total_daemons: AtomicUsize,
    pub running_daemons: AtomicUsize,
    pub stopped_daemons: AtomicUsize,
    pub error_daemons: AtomicUsize,
    pub daemon_restarts: AtomicU64,
    pub daemon_failures: AtomicU64,
    pub total_uptime_seconds: AtomicU64,
}

/// Daemon registry
static DAEMON_REGISTRY: RwLock<Vec<DaemonInfo>> = RwLock::new(Vec::new());

/// Daemon contexts
static DAEMON_CONTEXTS: RwLock<Vec<DaemonContext>> = RwLock::new(Vec::new());

/// Daemon operations
static DAEMON_OPERATIONS: RwLock<Vec<DaemonOperations>> = RwLock::new(Vec::new());

/// Next daemon ID
static NEXT_DAEMON_ID: AtomicU64 = AtomicU64::new(1);

/// Daemon service statistics
static DAEMON_SERVICE_STATS: DaemonServiceStats = DaemonServiceStats {
    total_daemons: AtomicUsize::new(0),
    running_daemons: AtomicUsize::new(0),
    stopped_daemons: AtomicUsize::new(0),
    error_daemons: AtomicUsize::new(0),
    daemon_restarts: AtomicU64::new(0),
    daemon_failures: AtomicU64::new(0),
    total_uptime_seconds: AtomicU64::new(0),
};

/// Initialize daemon registry
fn init_daemon_registry() -> Result<()> {
    info!("Initializing daemon registry...");
    
    // Clear registry
    let mut registry = DAEMON_REGISTRY.write();
    registry.clear();
    
    // Register core system daemons
    register_core_daemons()?;
    
    info!("Daemon registry initialized with {} daemons", registry.len());
    
    Ok(())
}

/// Register core system daemons
fn register_core_daemons() -> Result<()> {
    info!("Registering core system daemons...");
    
    // System monitoring daemon
    register_daemon(DaemonConfig {
        name: "system_monitor".to_string(),
        daemon_type: DaemonType::SystemService,
        description: "System monitoring and health checking".to_string(),
        version: "1.0.0".to_string(),
        priority: PriorityLevel::High,
        dependencies: vec![],
        auto_restart: true,
        max_restart_attempts: 3,
        initial_state: DaemonState::Stopped,
        resource_limits: ResourceLimits {
            max_memory_mb: 64,
            max_cpu_percent: 10,
            max_disk_io_mb_s: 10,
            max_network_io_mb_s: 0,
            max_file_descriptors: 64,
        },
    }, DaemonOperations {
        start: system_monitor_start,
        stop: system_monitor_stop,
        run: system_monitor_run,
        error_handler: Some(system_monitor_error),
    })?;
    
    // Power management daemon
    register_daemon(DaemonConfig {
        name: "power_manager".to_string(),
        daemon_type: DaemonType::SystemService,
        description: "Power management and energy optimization".to_string(),
        version: "1.0.0".to_string(),
        priority: PriorityLevel::High,
        dependencies: vec!["system_monitor".to_string()],
        auto_restart: true,
        max_restart_attempts: 3,
        initial_state: DaemonState::Stopped,
        resource_limits: ResourceLimits {
            max_memory_mb: 32,
            max_cpu_percent: 5,
            max_disk_io_mb_s: 5,
            max_network_io_mb_s: 0,
            max_file_descriptors: 32,
        },
    }, DaemonOperations {
        start: power_manager_start,
        stop: power_manager_stop,
        run: power_manager_run,
        error_handler: Some(power_manager_error),
    })?;
    
    // Network service daemon
    register_daemon(DaemonConfig {
        name: "network_service".to_string(),
        daemon_type: DaemonType::NetworkService,
        description: "Network services and connectivity".to_string(),
        version: "1.0.0".to_string(),
        priority: PriorityLevel::Normal,
        dependencies: vec![],
        auto_restart: true,
        max_restart_attempts: 5,
        initial_state: DaemonState::Stopped,
        resource_limits: ResourceLimits {
            max_memory_mb: 128,
            max_cpu_percent: 20,
            max_disk_io_mb_s: 50,
            max_network_io_mb_s: 100,
            max_file_descriptors: 256,
        },
    }, DaemonOperations {
        start: network_service_start,
        stop: network_service_stop,
        run: network_service_run,
        error_handler: Some(network_service_error),
    })?;
    
    info!("Core system daemons registered");
    
    Ok(())
}

/// Initialize daemon scheduler
fn init_daemon_scheduler() -> Result<()> {
    info!("Initializing daemon scheduler...");
    
    // Start daemon scheduler timer
    let _ = crate::services::time_service::create_timer(
        crate::services::time_service::TimerType::Periodic,
        100_000_000, // 100ms
        daemon_scheduler_callback
    );
    
    Ok(())
}

/// Initialize lifecycle management
fn init_lifecycle_management() -> Result<()> {
    info!("Initializing daemon lifecycle management...");
    
    // Start lifecycle management timer
    let _ = crate::services::time_service::create_timer(
        crate::services::time_service::TimerType::Periodic,
        1_000_000_000, // 1s
        daemon_lifecycle_callback
    );
    
    Ok(())
}

/// Start daemon services
fn start_daemon_services() -> Result<()> {
    info!("Starting daemon management services...");
    
    // Start all daemons that are configured to auto-start
    start_auto_start_daemons()?;
    
    Ok(())
}

/// Stop daemon services
fn stop_daemon_services() -> Result<()> {
    info!("Stopping daemon management services...");
    
    // Stop all running daemons
    stop_all_daemons()?;
    
    Ok(())
}

/// Shutdown lifecycle management
fn shutdown_lifecycle_management() -> Result<()> {
    info!("Shutting down daemon lifecycle management...");
    
    Ok(())
}

/// Shutdown daemon scheduler
fn shutdown_daemon_scheduler() -> Result<()> {
    info!("Shutting down daemon scheduler...");
    
    Ok(())
}

/// Shutdown daemon registry
fn shutdown_daemon_registry() -> Result<()> {
    info!("Shutting down daemon registry...");
    
    let mut registry = DAEMON_REGISTRY.write();
    registry.clear();
    
    let mut contexts = DAEMON_CONTEXTS.write();
    contexts.clear();
    
    let mut operations = DAEMON_OPERATIONS.write();
    operations.clear();
    
    Ok(())
}

/// Register a new daemon
pub fn register_daemon(config: DaemonConfig, operations: DaemonOperations) -> Result<u64> {
    let daemon_id = NEXT_DAEMON_ID.fetch_add(1, Ordering::SeqCst);
    
    let daemon_info = DaemonInfo {
        id: daemon_id,
        name: config.name.clone(),
        daemon_type: config.daemon_type,
        description: config.description.clone(),
        version: config.version.clone(),
        priority: config.priority,
        dependencies: Vec::new(), // Will be populated after dependency resolution
        auto_restart: config.auto_restart,
        max_restart_attempts: config.max_restart_attempts,
        state: config.initial_state,
        created_at: crate::services::time_service::get_uptime_ns(),
        last_started_at: 0,
        run_count: 0,
    };
    
    let daemon_context = DaemonContext {
        daemon_id,
        config: config.clone(),
        state: config.initial_state,
        restart_count: 0,
        last_error: None,
        resources: DaemonResources {
            memory_used_mb: 0,
            cpu_usage_percent: 0,
            disk_io_mb_s: 0,
            network_io_mb_s: 0,
            file_descriptors: 0,
        },
    };
    
    // Add to registry
    {
        let mut registry = DAEMON_REGISTRY.write();
        registry.push(daemon_info);
    }
    
    // Add to contexts
    {
        let mut contexts = DAEMON_CONTEXTS.write();
        contexts.push(daemon_context);
    }
    
    // Add operations
    {
        let mut operations_registry = DAEMON_OPERATIONS.write();
        operations_registry.push(operations);
    }
    
    DAEMON_SERVICE_STATS.total_daemons.fetch_add(1, Ordering::SeqCst);
    
    info!("Daemon registered: {} (ID: {})", config.name, daemon_id);
    
    Ok(daemon_id)
}

/// Start a specific daemon
pub fn start_daemon(daemon_id: u64) -> Result<()> {
    info!("Starting daemon {}", daemon_id);
    
    // Check if daemon exists
    let daemon_index = find_daemon_index(daemon_id)?;
    
    // Check dependencies
    check_dependencies(daemon_id)?;
    
    // Update state to starting
    {
        let mut contexts = DAEMON_CONTEXTS.write();
        let context = &mut contexts[daemon_index];
        context.state = DaemonState::Starting;
    }
    
    // Call start operation
    let operations = DAEMON_OPERATIONS.read();
    if daemon_index < operations.len() {
        let mut contexts = DAEMON_CONTEXTS.write();
        let context = &mut contexts[daemon_index];
        
        if let Err(e) = (operations[daemon_index].start)(context) {
            // Start failed
            context.state = DaemonState::Error;
            context.last_error = Some(format!("Start failed: {:?}", e));
            
            warn!("Daemon {} start failed: {:?}", daemon_id, e);
            DAEMON_SERVICE_STATS.daemon_failures.fetch_add(1, Ordering::SeqCst);
            
            return Err(e);
        }
    }
    
    // Update state to running
    {
        let mut contexts = DAEMON_CONTEXTS.write();
        let context = &mut contexts[daemon_index];
        context.state = DaemonState::Running;
        context.last_started_at = crate::services::time_service::get_uptime_ns();
        context.run_count += 1;
    }
    
    // Update registry
    {
        let mut registry = DAEMON_REGISTRY.write();
        registry[daemon_index].state = DaemonState::Running;
        registry[daemon_index].last_started_at = crate::services::time_service::get_uptime_ns();
        registry[daemon_index].run_count += 1;
    }
    
    DAEMON_SERVICE_STATS.running_daemons.fetch_add(1, Ordering::SeqCst);
    
    info!("Daemon {} started successfully", daemon_id);
    
    Ok(())
}

/// Stop a specific daemon
pub fn stop_daemon(daemon_id: u64) -> Result<()> {
    info!("Stopping daemon {}", daemon_id);
    
    let daemon_index = find_daemon_index(daemon_id)?;
    
    // Update state to stopping
    {
        let mut contexts = DAEMON_CONTEXTS.write();
        let context = &mut contexts[daemon_index];
        context.state = DaemonState::Stopping;
    }
    
    // Call stop operation
    let operations = DAEMON_OPERATIONS.read();
    if daemon_index < operations.len() {
        let mut contexts = DAEMON_CONTEXTS.write();
        let context = &mut contexts[daemon_index];
        
        if let Err(e) = (operations[daemon_index].stop)(context) {
            warn!("Daemon {} stop failed: {:?}", daemon_id, e);
        }
    }
    
    // Update state to stopped
    {
        let mut contexts = DAEMON_CONTEXTS.write();
        let context = &mut contexts[daemon_index];
        context.state = DaemonState::Stopped;
    }
    
    // Update registry
    {
        let mut registry = DAEMON_REGISTRY.write();
        registry[daemon_index].state = DaemonState::Stopped;
    }
    
    DAEMON_SERVICE_STATS.running_daemons.fetch_sub(1, Ordering::SeqCst);
    DAEMON_SERVICE_STATS.stopped_daemons.fetch_add(1, Ordering::SeqCst);
    
    info!("Daemon {} stopped", daemon_id);
    
    Ok(())
}

/// Restart a specific daemon
pub fn restart_daemon(daemon_id: u64) -> Result<()> {
    info!("Restarting daemon {}", daemon_id);
    
    // Stop the daemon
    stop_daemon(daemon_id)?;
    
    // Wait a bit
    crate::hal::timers::sleep_ns(100_000_000); // 100ms
    
    // Start the daemon
    start_daemon(daemon_id)?;
    
    Ok(())
}

/// Get all daemons
pub fn get_all_daemons() -> Vec<DaemonInfo> {
    DAEMON_REGISTRY.read().clone()
}

/// Get running daemons
pub fn get_running_daemons() -> Vec<DaemonInfo> {
    DAEMON_REGISTRY.read()
        .iter()
        .filter(|d| d.state == DaemonState::Running)
        .cloned()
        .collect()
}

/// Get daemon by name
pub fn get_daemon_by_name(name: &str) -> Result<DaemonInfo> {
    let registry = DAEMON_REGISTRY.read();
    
    for daemon in registry.iter() {
        if daemon.name == name {
            return Ok(daemon.clone());
        }
    }
    
    Err(KernelError::InvalidParameter)
}

/// Find daemon index by ID
fn find_daemon_index(daemon_id: u64) -> Result<usize> {
    let registry = DAEMON_REGISTRY.read();
    
    for (i, daemon) in registry.iter().enumerate() {
        if daemon.id == daemon_id {
            return Ok(i);
        }
    }
    
    Err(KernelError::InvalidParameter)
}

/// Check daemon dependencies
fn check_dependencies(daemon_id: u64) -> Result<()> {
    let registry = DAEMON_REGISTRY.read();
    let daemon_index = find_daemon_index(daemon_id)?;
    
    let daemon = &registry[daemon_index];
    
    // Check if all dependencies are running
    for &dep_id in &daemon.dependencies {
        let dep_index = find_daemon_index(dep_id)?;
        if registry[dep_index].state != DaemonState::Running {
            return Err(KernelError::InvalidParameter);
        }
    }
    
    Ok(())
}

/// Start all auto-start daemons
fn start_auto_start_daemons() -> Result<()> {
    info!("Starting auto-start daemons...");
    
    let registry = DAEMON_REGISTRY.read();
    
    // Start daemons sorted by priority
    let mut daemons_to_start: Vec<_> = registry.iter()
        .filter(|d| d.state == DaemonState::Stopped && d.auto_restart)
        .cloned()
        .collect();
    
    // Sort by priority (higher priority first)
    daemons_to_start.sort_by(|a, b| a.priority.cmp(&b.priority));
    
    for daemon in daemons_to_start {
        if let Err(e) = start_daemon(daemon.id) {
            warn!("Failed to start auto-start daemon {}: {:?}", daemon.name, e);
        }
    }
    
    Ok(())
}

/// Stop all daemons
fn stop_all_daemons() -> Result<()> {
    info!("Stopping all daemons...");
    
    let registry = DAEMON_REGISTRY.read();
    
    for daemon in registry.iter() {
        if daemon.state == DaemonState::Running {
            let _ = stop_daemon(daemon.id);
        }
    }
    
    Ok(())
}

/// Daemon scheduler callback
fn daemon_scheduler_callback(_interval_ns: u64, _timer_type: crate::services::time_service::TimerType) {
    schedule_daemon_execution();
}

/// Daemon lifecycle callback
fn daemon_lifecycle_callback(_interval_ns: u64, _timer_type: crate::services::time_service::TimerType) {
    manage_daemon_lifecycle();
}

/// Schedule daemon execution
fn schedule_daemon_execution() {
    let operations = DAEMON_OPERATIONS.read();
    let mut contexts = DAEMON_CONTEXTS.write();
    
    for (i, context) in contexts.iter_mut().enumerate() {
        if context.state == DaemonState::Running && i < operations.len() {
            // Call run operation
            if let Err(e) = (operations[i].run)(context) {
                // Handle daemon run error
                context.last_error = Some(format!("Run failed: {:?}", e));
                
                if context.config.auto_restart && context.restart_count < context.config.max_restart_attempts {
                    context.restart_count += 1;
                    DAEMON_SERVICE_STATS.daemon_restarts.fetch_add(1, Ordering::SeqCst);
                    
                    info!("Restarting daemon {} (attempt {})", context.daemon_id, context.restart_count);
                    
                    // Restart the daemon
                    let _ = stop_daemon(context.daemon_id);
                    let _ = start_daemon(context.daemon_id);
                } else {
                    context.state = DaemonState::Error;
                    DAEMON_SERVICE_STATS.daemon_failures.fetch_add(1, Ordering::SeqCst);
                    
                    warn!("Daemon {} failed and will not be restarted", context.daemon_id);
                    
                    // Call error handler if available
                    if let Some(error_handler) = operations[i].error_handler {
                        let _ = error_handler(context, &format!("Run failed: {:?}", e));
                    }
                }
            }
        }
    }
}

/// Manage daemon lifecycle
fn manage_daemon_lifecycle() {
    // Monitor daemon health
    // Check for hung daemons
    // Perform periodic maintenance
    
    let mut contexts = DAEMON_CONTEXTS.write();
    let registry = DAEMON_REGISTRY.read();
    
    for (i, context) in contexts.iter_mut().enumerate() {
        // Check if running daemons need attention
        if context.state == DaemonState::Running {
            // Update resource usage (simplified)
            context.resources.memory_used_mb = 10; // Simulated
            context.resources.cpu_usage_percent = 5; // Simulated
        }
    }
}

/// System monitor daemon callbacks
fn system_monitor_start(_context: &mut DaemonContext) -> Result<()> {
    info!("Starting system monitor daemon");
    Ok(())
}

fn system_monitor_stop(_context: &mut DaemonContext) -> Result<()> {
    info!("Stopping system monitor daemon");
    Ok(())
}

fn system_monitor_run(context: &mut DaemonContext) -> Result<bool> {
    // Update system monitoring
    crate::services::monitoring_service::update_system_metrics()?;
    
    info!("System monitor daemon running (memory: {}MB, CPU: {}%)", 
          context.resources.memory_used_mb, context.resources.cpu_usage_percent);
    
    Ok(true) // Continue running
}

fn system_monitor_error(context: &mut DaemonContext, error_msg: &str) -> Result<()> {
    warn!("System monitor daemon error: {}", error_msg);
    context.last_error = Some(error_msg.to_string());
    Ok(())
}

/// Power manager daemon callbacks
fn power_manager_start(_context: &mut DaemonContext) -> Result<()> {
    info!("Starting power manager daemon");
    Ok(())
}

fn power_manager_stop(_context: &mut DaemonContext) -> Result<()> {
    info!("Stopping power manager daemon");
    Ok(())
}

fn power_manager_run(_context: &mut DaemonContext) -> Result<bool> {
    // Update power management
    let _ = crate::services::power_service::update_thermal_info();
    let _ = crate::services::power_service::update_battery_info();
    
    Ok(true)
}

fn power_manager_error(context: &mut DaemonContext, error_msg: &str) -> Result<()> {
    warn!("Power manager daemon error: {}", error_msg);
    context.last_error = Some(error_msg.to_string());
    Ok(())
}

/// Network service daemon callbacks
fn network_service_start(_context: &mut DaemonContext) -> Result<()> {
    info!("Starting network service daemon");
    Ok(())
}

fn network_service_stop(_context: &mut DaemonContext) -> Result<()> {
    info!("Stopping network service daemon");
    Ok(())
}

fn network_service_run(_context: &mut DaemonContext) -> Result<bool> {
    // Process network operations
    
    Ok(true)
}

fn network_service_error(context: &mut DaemonContext, error_msg: &str) -> Result<()> {
    warn!("Network service daemon error: {}", error_msg);
    context.last_error = Some(error_msg.to_string());
    Ok(())
}

/// Get daemon service statistics
pub fn get_stats() -> DaemonServiceStats {
    DAEMON_SERVICE_STATS
}

/// Get daemon count
pub fn get_daemon_count() -> usize {
    DAEMON_SERVICE_STATS.total_daemons.load(Ordering::SeqCst)
}

/// Benchmark daemon service
pub fn benchmark_daemon_service() -> Result<(u64, u64, u64)> {
    info!("Benchmarking daemon service...");
    
    let mut registration_time = 0;
    let mut startup_time = 0;
    let mut execution_time = 0;
    
    // Benchmark daemon registration
    let start = crate::hal::timers::get_high_res_time();
    let _ = register_daemon(DaemonConfig {
        name: "benchmark_daemon".to_string(),
        daemon_type: DaemonType::BackgroundTask,
        description: "Benchmark test daemon".to_string(),
        version: "1.0.0".to_string(),
        priority: PriorityLevel::Background,
        dependencies: vec![],
        auto_restart: false,
        max_restart_attempts: 0,
        initial_state: DaemonState::Stopped,
        resource_limits: ResourceLimits {
            max_memory_mb: 10,
            max_cpu_percent: 1,
            max_disk_io_mb_s: 1,
            max_network_io_mb_s: 0,
            max_file_descriptors: 10,
        },
    }, DaemonOperations {
        start: |_| { Ok(()) },
        stop: |_| { Ok(()) },
        run: |_| { Ok(true) },
        error_handler: None,
    });
    registration_time = crate::hal::timers::get_high_res_time() - start;
    
    Ok((registration_time, startup_time, execution_time))
}

/// Daemon utility functions
pub mod utils {
    use super::*;
    
    /// Format daemon state as string
    pub fn format_state(state: DaemonState) -> &'static str {
        match state {
            DaemonState::Stopped => "Stopped",
            DaemonState::Starting => "Starting",
            DaemonState::Running => "Running",
            DaemonState::Paused => "Paused",
            DaemonState::Stopping => "Stopping",
            DaemonState::Error => "Error",
            DaemonState::Disabled => "Disabled",
        }
    }
    
    /// Format daemon type as string
    pub fn format_daemon_type(daemon_type: DaemonType) -> &'static str {
        match daemon_type {
            DaemonType::SystemService => "System Service",
            DaemonType::UserService => "User Service",
            DaemonType::BackgroundTask => "Background Task",
            DaemonType::PeriodicTask => "Periodic Task",
            DaemonType::EventDriven => "Event Driven",
            DaemonType::NetworkService => "Network Service",
            DaemonType::StorageService => "Storage Service",
            DaemonType::SecurityService => "Security Service",
        }
    }
    
    /// Format priority as string
    pub fn format_priority(priority: PriorityLevel) -> &'static str {
        match priority {
            PriorityLevel::Critical => "Critical",
            PriorityLevel::High => "High",
            PriorityLevel::Normal => "Normal",
            PriorityLevel::Low => "Low",
            PriorityLevel::Background => "Background",
        }
    }
    
    /// Calculate daemon uptime
    pub fn calculate_uptime(daemon: &DaemonInfo) -> u64 {
        if daemon.state == DaemonState::Running && daemon.last_started_at > 0 {
            crate::services::time_service::get_uptime_ns() - daemon.last_started_at
        } else {
            0
        }
    }
    
    /// Check if daemon is healthy
    pub fn is_healthy(daemon: &DaemonInfo) -> bool {
        match daemon.state {
            DaemonState::Running => true,
            DaemonState::Stopped => false,
            DaemonState::Starting => true,
            DaemonState::Paused => true,
            DaemonState::Stopping => false,
            DaemonState::Error => false,
            DaemonState::Disabled => false,
        }
    }
}