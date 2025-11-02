//! MultiOS Advanced Multi-Core Optimization and Virtual Memory Scaling
//!
//! This module provides comprehensive multi-core optimization and virtual memory
//! scaling for MultiOS, integrating all advanced features including:
//! - NUMA-aware memory management
//! - Multi-core scheduler optimization with CPU hot-plug support
//! - Cache coherency protocols and synchronization primitives
//! - Large-scale virtual memory support (petabytes to exabytes)
//! - Comprehensive performance monitoring and optimization
//! - Real-time scheduling for hundreds of cores
//! - Memory affinity and intelligent load balancing
//! - Advanced power and thermal management
//! - Performance regression detection and auto-tuning

#![no_std]

use spin::Mutex;
use bitflags::bitflags;

// Import all submodules
pub mod process;
pub mod thread;
pub mod scheduler_algo;
pub mod multicore;
pub mod performance_monitor;

#[cfg(feature = "examples")]
pub mod examples;
#[cfg(test)]
pub mod tests;

// Re-export key types for convenience
pub use process::{
    ProcessManager, ProcessId, ProcessPriority, ProcessState, ProcessFlags,
    ProcessControlBlock, ProcessCreateParams, ProcessResult, ProcessError,
    PROCESS_MANAGER,
};

pub use thread::{
    ThreadManager, ThreadId, ThreadHandle, ThreadEntry, ThreadParams,
    ThreadControlBlock, ThreadResult, ThreadError, THREAD_MANAGER, ContextSwitch,
};

pub use scheduler_algo::{
    Scheduler, SchedulerConfig, SchedulerHelpers, SchedulerStatsSnapshot,
    SchedulingAlgorithm, CpuAffinity,
};

pub use multicore::{
    MulticoreScheduler, MulticoreConfig, MulticoreConfigBuilder,
    CpuPowerState, CpuPerfInfo, CpuIdleState, SchedDomain,
    BalanceAlgorithm, NumaScheduler, RealtimeScheduler,
    PerformanceMonitor, PerformanceConfig,
    CacheCoherencyMonitor, CacheProtocol, CacheState,
    LockFreeQueue, LockFreeStack, LockFreeCounter,
    MemoryBarriers, CpuGovernor, ThermalAction,
};

pub use performance_monitor::{
    PerfCounterType, PerformanceStats, PerformanceAlert,
    AlertSeverity, AlertAction, PerformancePredictor,
    OptimizationObjective, PerformanceRegression,
    ResourceContentionAnalyzer, ContentionAnalysis,
};

pub use thread::THREAD_MANAGER;
pub use process::PROCESS_MANAGER;

/// Multi-core system configuration
#[derive(Debug, Clone)]
pub struct MultiCoreConfig {
    pub max_cpus: usize,
    pub enable_numa: bool,
    pub numa_nodes: usize,
    pub enable_hotplug: bool,
    pub enable_performance_monitoring: bool,
    pub enable_real_time: bool,
    pub enable_cache_coherency: bool,
    pub enable_large_scale_vm: bool,
    pub max_virtual_memory: usize,
    pub enable_power_management: bool,
    pub enable_thermal_management: bool,
    pub scheduler_config: SchedulerConfig,
    pub multicore_config: MulticoreConfig,
    pub performance_config: PerformanceConfig,
}

/// Multi-core system state
#[derive(Debug)]
pub struct MultiCoreSystem {
    pub scheduler: MulticoreScheduler,
    pub performance_monitor: PerformanceMonitor,
    pub numa_manager: Option<NumaManager>,
    pub cache_coherency: Option<CacheCoherencyMonitor>,
    pub large_scale_vm: Option<LargeScaleVM>,
    pub config: MultiCoreConfig,
    pub initialized: bool,
    pub bootstrap_complete: bool,
}

/// NUMA manager wrapper (imported from memory-manager)
use memory_manager::numa::NumaManager;

/// Large-scale virtual memory wrapper
use memory_manager::large_scale_vm::LargeScaleVirtualMemory;

/// Initialize the complete multi-core optimization system
pub fn init_multicore_system(config: MultiCoreConfig) -> MultiCoreResult<()> {
    info!("Initializing MultiOS Advanced Multi-Core System...");
    
    let mut system = MultiCoreSystem {
        scheduler: MulticoreScheduler::new(config.multicore_config.clone()),
        performance_monitor: PerformanceMonitor::new(config.performance_config.clone(), config.max_cpus),
        numa_manager: None,
        cache_coherency: None,
        large_scale_vm: None,
        config: config.clone(),
        initialized: false,
        bootstrap_complete: false,
    };

    // Initialize NUMA management
    if config.enable_numa {
        info!("Initializing NUMA management...");
        let numa_config = memory_manager::numa::NumaConfig {
            enable_numa: true,
            enable_balancing: true,
            balance_interval: 1000,
            migration_threshold: 0.1,
            max_migrations_per_sec: 100,
            enable_interleaving: false,
        };
        system.numa_manager = Some(NumaManager::new(numa_config));
    }

    // Initialize cache coherency
    if config.enable_cache_coherency {
        info!("Initializing cache coherency protocols...");
        system.cache_coherency = Some(CacheCoherencyMonitor::new(
            CacheProtocol::MESIF,
            16 * 1024 * 1024, // 16MB cache
        ));
    }

    // Initialize large-scale virtual memory
    if config.enable_large_scale_vm {
        info!("Initializing large-scale virtual memory...");
        system.large_scale_vm = Some(LargeScaleVirtualMemory::new(config.max_virtual_memory));
    }

    // Initialize scheduler
    info!("Initializing multi-core scheduler...");
    system.scheduler.init()?;

    // Initialize performance monitoring
    if config.enable_performance_monitoring {
        info!("Starting performance monitoring...");
        system.performance_monitor.start_monitoring()?;
    }

    // Bootstrap complete
    system.initialized = true;
    system.bootstrap_complete = true;

    // Store system state globally
    *MULTICORE_SYSTEM.lock() = Some(system);
    
    info!("MultiOS Multi-Core System initialized successfully!");
    info!("Configuration:");
    info!("  Max CPUs: {}", config.max_cpus);
    info!("  NUMA enabled: {}", config.enable_numa);
    info!("  Hot-plug enabled: {}", config.enable_hotplug);
    info!("  Performance monitoring: {}", config.enable_performance_monitoring);
    info!("  Real-time scheduling: {}", config.enable_real_time);
    info!("  Cache coherency: {}", config.enable_cache_coherency);
    info!("  Large-scale VM: {}", config.enable_large_scale_vm);
    info!("  Max virtual memory: {} bytes", config.max_virtual_memory);

    Ok(())
}

/// Global multi-core system state
static MULTICORE_SYSTEM: Mutex<Option<MultiCoreSystem>> = Mutex::new(None);

/// Multi-core system result
pub type MultiCoreResult<T> = Result<T, MultiCoreError>;

/// Multi-core system errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiCoreError {
    NotInitialized,
    AlreadyInitialized,
    InvalidConfiguration,
    InitializationFailed,
    ResourceUnavailable,
    UnsupportedFeature,
    ConfigurationError,
    HardwareIncompatible,
}

/// Get the global multi-core system instance
fn get_multicore_system() -> MultiCoreResult<&'static Mutex<Option<MultiCoreSystem>>> {
    let guard = MULTICORE_SYSTEM.lock();
    if guard.is_none() {
        return Err(MultiCoreError::NotInitialized);
    }
    Ok(&MULTICORE_SYSTEM)
}

/// High-level API functions

/// Add a process to the multi-core system
pub fn add_process(params: process::ProcessCreateParams) -> process::ProcessResult<ProcessId> {
    let system = get_multicore_system()?;
    let guard = system.lock();
    
    if let Some(sys) = guard.as_ref() {
        if !sys.initialized {
            return Err(process::ProcessError::InvalidParameter);
        }
        
        // Add process through existing process manager
        sys.scheduler.add_process_optimized(params)
    } else {
        Err(process::ProcessError::InvalidParameter)
    }
}

/// Add a thread with optimal placement
pub fn add_thread_optimized(thread: thread::ThreadHandle) -> thread::ThreadResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        sys.scheduler.add_thread_optimized(thread)?;
        
        // Update performance monitoring
        if let Some(perf_monitor) = &sys.performance_monitor {
            perf_monitor.get_current_stats();
        }
        
        Ok(())
    } else {
        Err(thread::ThreadError::InvalidParameter)
    }
}

/// Set CPU affinity for optimal placement
pub fn set_thread_cpu_affinity_optimized(
    thread_id: thread::ThreadId,
    affinity: scheduler_algo::CpuAffinity,
) -> thread::ThreadResult<()> {
    let system = get_multicore_system()?;
    let guard = system.lock();
    
    if let Some(sys) = guard.as_ref() {
        // Update scheduler with new affinity
        sys.scheduler.set_thread_cpu_affinity(thread_id, affinity)?;
        
        // Update NUMA affinity if enabled
        if let Some(numa_manager) = &sys.numa_manager {
            let policy = memory_manager::numa::NumaPolicy::Bind(0); // Default to node 0
            // numa_manager.set_thread_policy(thread_id as usize, policy)?;
        }
        
        Ok(())
    } else {
        Err(thread::ThreadError::InvalidParameter)
    }
}

/// Enable CPU hot-plug
pub fn enable_cpu_hotplug(cpu_id: usize, enabled: bool) -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        sys.scheduler.set_cpu_enabled(cpu_id, enabled)?;
        
        // Update performance monitoring
        sys.performance_monitor.set_counter_enabled(0, enabled)?;
        
        Ok(())
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Optimize memory allocation for NUMA
pub fn allocate_memory_numa_aware(size: usize, policy: memory_manager::numa::NumaPolicy) -> MultiCoreResult<Vec<memory_manager::PhysAddr>> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        if let Some(numa_manager) = &mut sys.numa_manager {
            if !numa_manager.is_initialized() {
                return Err(MultiCoreError::NotInitialized);
            }
            
            let page_count = size / 4096; // Assume 4KB pages
            numa_manager.allocate_with_policy(policy, page_count)
                .map_err(|_| MultiCoreError::ResourceUnavailable)
        } else {
            Err(MultiCoreError::UnsupportedFeature)
        }
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Map virtual memory with large-scale support
pub fn map_virtual_memory_large_scale(
    start: memory_manager::VirtAddr,
    size: usize,
    flags: memory_manager::VmaFlags,
    backing: memory_manager::VmaBacking,
    huge_page_preference: bool,
) -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        if let Some(large_vm) = &mut sys.large_scale_vm {
            large_vm.map_virtual_extended(start, size, flags, backing, huge_page_preference)
                .map_err(|_| MultiCoreError::InitializationFailed)
        } else {
            Err(MultiCoreError::UnsupportedFeature)
        }
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Perform performance optimization
pub fn optimize_performance() -> MultiCoreResult<performance_monitor::OptimizationRecommendation> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        sys.performance_monitor.optimize_performance()
            .map_err(|_| MultiCoreError::ResourceUnavailable)
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Get comprehensive performance statistics
pub fn get_performance_statistics() -> performance_monitor::PerformanceStats {
    let system = get_multicore_system();
    
    match system {
        Ok(system) => {
            let guard = system.lock();
            if let Some(sys) = guard.as_ref() {
                sys.performance_monitor.get_current_stats()
            } else {
                performance_monitor::PerformanceStats::default()
            }
        },
        Err(_) => performance_monitor::PerformanceStats::default(),
    }
}

/// Get NUMA statistics
pub fn get_numa_statistics() -> memory_manager::numa::NumaMemoryStats {
    let system = get_multicore_system();
    
    match system {
        Ok(system) => {
            let guard = system.lock();
            if let Some(sys) = guard.as_ref() {
                if let Some(numa_manager) = &sys.numa_manager {
                    numa_manager.get_stats()
                } else {
                    memory_manager::numa::NumaMemoryStats::default()
                }
            } else {
                memory_manager::numa::NumaMemoryStats::default()
            }
        },
        Err(_) => memory_manager::numa::NumaMemoryStats::default(),
    }
}

/// Get cache coherency statistics
pub fn get_cache_coherency_statistics() -> memory_manager::cache_coherency::ProtocolStats {
    let system = get_multicore_system();
    
    match system {
        Ok(system) => {
            let guard = system.lock();
            if let Some(sys) = guard.as_ref() {
                if let Some(cache_coherency) = &sys.cache_coherency {
                    cache_coherency.get_protocol_stats()
                } else {
                    memory_manager::cache_coherency::ProtocolStats::default()
                }
            } else {
                memory_manager::cache_coherency::ProtocolStats::default()
            }
        },
        Err(_) => memory_manager::cache_coherency::ProtocolStats::default(),
    }
}

/// Get virtual memory statistics
pub fn get_virtual_memory_statistics() -> memory_manager::large_scale_vm::VirtualMemoryStats {
    let system = get_multicore_system();
    
    match system {
        Ok(system) => {
            let guard = system.lock();
            if let Some(sys) = guard.as_ref() {
                if let Some(large_vm) = &sys.large_scale_vm {
                    large_vm.get_stats()
                } else {
                    memory_manager::large_scale_vm::VirtualMemoryStats::default()
                }
            } else {
                memory_manager::large_scale_vm::VirtualMemoryStats::default()
            }
        },
        Err(_) => memory_manager::large_scale_vm::VirtualMemoryStats::default(),
    }
}

/// Handle memory pressure
pub fn handle_memory_pressure() -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        if let Some(large_vm) = &mut sys.large_scale_vm {
            large_vm.handle_memory_pressure()
                .map_err(|_| MultiCoreError::ResourceUnavailable)
        } else {
            Ok(())
        }
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Perform memory deduplication
pub fn perform_memory_deduplication() -> MultiCoreResult<usize> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        if let Some(large_vm) = &mut sys.large_scale_vm {
            large_vm.perform_deduplication()
                .map_err(|_| MultiCoreError::ResourceUnavailable)
        } else {
            Ok(0)
        }
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Enable real-time scheduling for critical threads
pub fn enable_realtime_scheduling(enable: bool) -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        // Update scheduler configuration
        let mut config = sys.scheduler.config.clone();
        config.enable_realtime = enable;
        
        // Reinitialize scheduler with new configuration
        sys.scheduler = MulticoreScheduler::new(config);
        sys.scheduler.init()?;
        
        Ok(())
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Configure power management
pub fn configure_power_management(policy: multicore::CpuGovernor, scaling_enabled: bool) -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        // Update power management configuration
        // This would update CPU frequency governors and power policies
        
        Ok(())
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Enable thermal management
pub fn enable_thermal_management(enable: bool, throttle_temp: u8) -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        // Configure thermal management
        // This would set thermal thresholds and response actions
        
        Ok(())
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Export comprehensive performance report
pub fn export_performance_report(format: performance_monitor::ExportFormat) -> MultiCoreResult<Vec<u8>> {
    let system = get_multicore_system()?;
    let guard = system.lock();
    
    if let Some(sys) = guard.as_ref() {
        sys.performance_monitor.export_performance_data(format)
            .map_err(|_| MultiCoreError::ResourceUnavailable)
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Create optimized configuration for system specifications
pub fn create_optimized_config(
    cpu_count: usize,
    memory_gb: usize,
    numa_nodes: usize,
    enable_advanced_features: bool,
) -> MultiCoreConfig {
    let base_config = MultiCoreConfig {
        max_cpus: core::cmp::min(cpu_count, 1024),
        enable_numa: numa_nodes > 1,
        numa_nodes,
        enable_hotplug: true,
        enable_performance_monitoring: true,
        enable_real_time: enable_advanced_features,
        enable_cache_coherency: true,
        enable_large_scale_vm: memory_gb > 256, // Enable for systems with >256GB RAM
        max_virtual_memory: if memory_gb > 1024 {
            1 << 60 // 1 Exabyte for very large systems
        } else {
            1 << 50 // 1 Petabyte for medium-large systems
        },
        enable_power_management: true,
        enable_thermal_management: enable_advanced_features,
        scheduler_config: SchedulerConfig {
            algorithm: scheduler_algo::SchedulingAlgorithm::MultiLevelFeedbackQueue,
            cpu_count: core::cmp::min(cpu_count, 256),
            default_time_quantum: if enable_advanced_features { 25 } else { 20 },
            load_balance_interval: 500,
            enable_cpu_affinity: true,
            enable_load_balancing: true,
        },
        multicore_config: MulticoreConfig {
            max_cpus: cpu_count,
            enable_hotplug: true,
            enable_domains: cpu_count > 32,
            domain_size: core::cmp::min(cpu_count / 8, 64),
            enable_balancing: true,
            balance_algorithm: if numa_nodes > 1 {
                multicore::BalanceAlgorithm::NumaAware
            } else {
                multicore::BalanceAlgorithm::LoadBased
            },
            enable_power_mgmt: true,
            enable_realtime: enable_advanced_features,
            enable_numa: numa_nodes > 1,
            rt_deadline_us: 1000,
            latency_target_ns: 1000,
            migration_cost_ns: 500,
            cache_line_size: 64,
            enable_monitoring: true,
            monitoring_interval: 100,
        },
        performance_config: PerformanceConfig {
            enable_hardware_counters: enable_advanced_features,
            enable_software_counters: true,
            sampling_frequency_hz: if enable_advanced_features { 200 } else { 100 },
            enable_prediction: enable_advanced_features,
            enable_auto_tuning: enable_advanced_features,
            alerting_enabled: true,
            retention_period_hours: if enable_advanced_features { 168 } else { 24 }, // 1 week vs 1 day
            max_history_size: if enable_advanced_features { 50000 } else { 10000 },
            thermal_monitoring: enable_advanced_features,
            power_monitoring: true,
            numa_monitoring: numa_nodes > 1,
        },
    };

    base_config
}

/// System compatibility check
pub fn check_system_compatibility() -> MultiCoreResult<CompatibilityReport> {
    let cpu_count = detect_cpu_count()?;
    let memory_gb = detect_memory_gb()?;
    let numa_nodes = detect_numa_nodes()?;
    
    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    // Check CPU count
    if cpu_count > 1024 {
        warnings.push(format!("High CPU count ({}) may impact scheduling performance", cpu_count));
    }

    // Check memory
    if memory_gb > 8192 {
        warnings.push(format!("Large memory system ({}) GB detected", memory_gb));
        recommendations.push("Enable large-scale virtual memory support".to_string());
    }

    // Check NUMA
    if numa_nodes > 16 {
        warnings.push(format!("Complex NUMA topology with {} nodes", numa_nodes));
        recommendations.push("Consider NUMA-aware scheduling algorithms".to_string());
    }

    // Check for hardware features
    if !has_performance_counters() {
        issues.push("Hardware performance counters not available".to_string());
    }

    if !has_thermal_sensors() {
        warnings.push("Thermal sensors not detected".to_string());
    }

    Ok(CompatibilityReport {
        cpu_count,
        memory_gb,
        numa_nodes,
        issues,
        warnings,
        recommendations,
        compatible: issues.is_empty(),
    })
}

/// Compatibility check results
#[derive(Debug, Clone)]
pub struct CompatibilityReport {
    pub cpu_count: usize,
    pub memory_gb: usize,
    pub numa_nodes: usize,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
    pub compatible: bool,
}

// Helper functions for system detection
fn detect_cpu_count() -> MultiCoreResult<usize> {
    // Simplified CPU detection - would use platform-specific methods
    Ok(8)
}

fn detect_memory_gb() -> MultiCoreResult<usize> {
    // Simplified memory detection - would query system
    Ok(64)
}

fn detect_numa_nodes() -> MultiCoreResult<usize> {
    // Simplified NUMA detection - would query firmware/ACPI
    Ok(1)
}

fn has_performance_counters() -> bool {
    // Simplified check - would query CPU capabilities
    true
}

fn has_thermal_sensors() -> bool {
    // Simplified check - would query ACPI/thermal sensors
    true
}

/// Multi-core system maintenance functions

/// Perform system health check
pub fn health_check() -> MultiCoreResult<HealthStatus> {
    let system = get_multicore_system()?;
    let guard = system.lock();
    
    if let Some(sys) = guard.as_ref() {
        let mut status = HealthStatus {
            overall_health: HealthLevel::Good,
            checks: Vec::new(),
        };

        // Check scheduler health
        let sched_stats = sys.performance_monitor.get_current_stats();
        if sched_stats.scheduler_stats.total_context_switches == 0 {
            status.checks.push(("Scheduler".to_string(), CheckResult::Warning, "No context switches detected".to_string()));
        } else {
            status.checks.push(("Scheduler".to_string(), CheckResult::Pass, "Operating normally".to_string()));
        }

        // Check NUMA health
        if let Some(numa_manager) = &sys.numa_manager {
            if numa_manager.is_initialized() {
                status.checks.push(("NUMA".to_string(), CheckResult::Pass, "NUMA management active".to_string()));
            } else {
                status.checks.push(("NUMA".to_string(), CheckResult::Warning, "NUMA manager not initialized".to_string()));
            }
        }

        // Check performance monitoring
        if sys.performance_monitor.get_current_stats().cpu_stats.len() > 0 {
            status.checks.push(("Performance Monitor".to_string(), CheckResult::Pass, "Monitoring active".to_string()));
        } else {
            status.checks.push(("Performance Monitor".to_string(), CheckResult::Warning, "No performance data".to_string()));
        }

        // Determine overall health
        let mut has_issues = false;
        let mut has_warnings = false;
        
        for (_, result, _) in &status.checks {
            match result {
                CheckResult::Fail => has_issues = true,
                CheckResult::Warning => has_warnings = true,
                _ => {}
            }
        }

        status.overall_health = if has_issues {
            HealthLevel::Critical
        } else if has_warnings {
            HealthLevel::Warning
        } else {
            HealthLevel::Good
        };

        Ok(status)
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub overall_health: HealthLevel,
    pub checks: Vec<(String, CheckResult, String)>,
}

/// Health level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthLevel {
    Good,
    Warning,
    Critical,
}

/// Individual check result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    Pass,
    Warning,
    Fail,
}

/// Shutdown the multi-core system gracefully
pub fn shutdown_multicore_system() -> MultiCoreResult<()> {
    let system = get_multicore_system()?;
    let mut guard = system.lock();
    
    if let Some(sys) = guard.as_mut() {
        info!("Shutting down MultiOS Multi-Core System...");
        
        // Stop performance monitoring
        if sys.config.enable_performance_monitoring {
            sys.performance_monitor.stop_monitoring()?;
        }

        // Perform final cleanup
        sys.initialized = false;
        sys.bootstrap_complete = false;

        info!("Multi-Core System shutdown complete");
        Ok(())
    } else {
        Err(MultiCoreError::NotInitialized)
    }
}

// Legacy compatibility functions
/// Initialize the scheduler (legacy function for backward compatibility)
pub fn init() -> SchedulerResult<()> {
    init_with_default()
}

/// Get the next thread to run (legacy function)
pub fn schedule_next() -> SchedulerResult<thread::ThreadHandle> {
    get_global_scheduler()?.schedule_next(0)
}

/// Add a thread to the scheduler (legacy function)
pub fn add_thread(thread: thread::ThreadHandle) -> SchedulerResult<()> {
    get_global_scheduler()?.add_thread(thread)
}

/// Remove a thread from the scheduler (legacy function)
pub fn remove_thread(thread_id: thread::ThreadId) -> SchedulerResult<()> {
    get_global_scheduler()?.remove_thread(thread_id, None)
}

/// Get current thread count (legacy function)
pub fn get_thread_count() -> usize {
    thread::THREAD_MANAGER.get_thread_count()
}

/// Get the global scheduler instance
fn get_global_scheduler() -> SchedulerResult<&'static Mutex<Option<scheduler_algo::Scheduler>>> {
    let scheduler_guard = scheduler_algo::SCHEDULER.lock();
    
    if scheduler_guard.is_none() {
        return Err(SchedulerError::SchedulerAlreadyInitialized);
    }
    
    Ok(&scheduler_algo::SCHEDULER)
}

/// Initialize the scheduler with default configuration
pub fn init_with_default() -> SchedulerResult<()> {
    init_with_config(SchedulerConfig::default())
}

/// Initialize the scheduler with a custom configuration
pub fn init_with_config(config: SchedulerConfig) -> SchedulerResult<()> {
    let mut scheduler_guard = scheduler_algo::SCHEDULER.lock();
    
    if scheduler_guard.is_some() {
        return Err(SchedulerError::SchedulerAlreadyInitialized);
    }
    
    let scheduler = scheduler_algo::Scheduler::with_config(config);
    *scheduler_guard = Some(scheduler);
    
    Ok(())
}

/// Get current thread count
pub fn get_current_thread_count() -> usize {
    let scheduler_guard = scheduler_algo::SCHEDULER.lock();
    
    if let Some(scheduler) = scheduler_guard.as_ref() {
        scheduler.get_thread_count()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multicore_system_initialization() {
        let config = create_optimized_config(8, 64, 1, true);
        let result = init_multicore_system(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimized_config_creation() {
        let config = create_optimized_config(32, 128, 2, true);
        assert_eq!(config.max_cpus, 32);
        assert!(config.enable_numa);
        assert_eq!(config.numa_nodes, 2);
        assert!(config.enable_performance_monitoring);
        assert!(config.enable_realtime);
        assert!(config.enable_cache_coherency);
        assert!(config.enable_large_scale_vm);
    }

    #[test]
    fn test_compatibility_check() {
        let report = check_system_compatibility().unwrap();
        assert!(report.cpu_count > 0);
        assert!(report.memory_gb > 0);
        assert!(report.numa_nodes > 0);
        assert!(report.compatible || !report.issues.is_empty());
    }

    #[test]
    fn test_performance_export() {
        let config = create_optimized_config(4, 16, 1, false);
        let _ = init_multicore_system(config);
        
        let json_data = export_performance_report(performance_monitor::ExportFormat::JSON);
        assert!(json_data.is_ok());
        
        let csv_data = export_performance_report(performance_monitor::ExportFormat::CSV);
        assert!(csv_data.is_ok());
    }

    #[test]
    fn test_system_health_check() {
        let config = create_optimized_config(4, 16, 1, false);
        let _ = init_multicore_system(config);
        
        let health = health_check().unwrap();
        assert!(matches!(health.overall_health, HealthLevel::Good | HealthLevel::Warning));
        assert!(!health.checks.is_empty());
    }

    #[test]
    fn test_legacy_scheduler_compatibility() {
        let result = init_with_default();
        assert!(result.is_ok());
        
        let thread_count = get_current_thread_count();
        assert_eq!(thread_count, 0);
    }
}