//! MultiOS System Call Performance Monitoring and Optimization
//! 
//! This module provides comprehensive performance monitoring, analysis, and optimization
//! for the system call interface, including latency measurement, throughput analysis,
//! bottleneck identification, and adaptive optimization strategies.

use crate::log::{info, warn, error, debug};
use crate::arch::interrupts::*;
use crate::memory;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use spin::Mutex;

type PerformanceResult<T> = Result<T, PerformanceError>;

/// Performance monitoring and optimization manager
pub struct SyscallPerformanceManager {
    /// Performance counters
    counters: PerformanceCounters,
    /// Latency histogram
    latency_histogram: LatencyHistogram,
    /// Throughput tracker
    throughput_tracker: ThroughputTracker,
    /// Bottleneck analyzer
    bottleneck_analyzer: BottleneckAnalyzer,
    /// Optimization engine
    optimization_engine: OptimizationEngine,
    /// Real-time performance monitor
    real_time_monitor: RealTimeMonitor,
}

/// Performance counters for system calls
#[derive(Debug, Clone)]
pub struct PerformanceCounters {
    /// Total system calls
    pub total_syscalls: AtomicU64,
    /// Successful system calls
    pub successful_syscalls: AtomicU64,
    /// Failed system calls
    pub failed_syscalls: AtomicU64,
    /// Total latency (nanoseconds)
    pub total_latency_ns: AtomicU64,
    /// Fast path system calls
    pub fast_path_syscalls: AtomicU64,
    /// Standard path system calls
    pub standard_path_syscalls: AtomicU64,
    /// Validation failures
    pub validation_failures: AtomicU64,
    /// Security violations
    pub security_violations: AtomicU64,
    /// Cache hits
    pub cache_hits: AtomicU64,
    /// Cache misses
    pub cache_misses: AtomicU64,
    /// Memory allocation operations
    pub memory_allocations: AtomicU64,
    /// I/O operations
    pub io_operations: AtomicU64,
}

impl PerformanceCounters {
    pub fn new() -> Self {
        Self {
            total_syscalls: AtomicU64::new(0),
            successful_syscalls: AtomicU64::new(0),
            failed_syscalls: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            fast_path_syscalls: AtomicU64::new(0),
            standard_path_syscalls:AtomicU64::new(0),
            validation_failures: AtomicU64::new(0),
            security_violations: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            memory_allocations: AtomicU64::new(0),
            io_operations: AtomicU64::new(0),
        }
    }

    /// Record syscall execution
    pub fn record_syscall(&self, latency_ns: u64, success: bool, fast_path: bool) {
        self.total_syscalls.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
        
        if success {
            self.successful_syscalls.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_syscalls.fetch_add(1, Ordering::Relaxed);
        }
        
        if fast_path {
            self.fast_path_syscalls.fetch_add(1, Ordering::Relaxed);
        } else {
            self.standard_path_syscalls.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record validation failure
    pub fn record_validation_failure(&self) {
        self.validation_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Record security violation
    pub fn record_security_violation(&self) {
        self.security_violations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache operation
    pub fn record_cache_operation(&self, hit: bool) {
        if hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> SyscallPerformanceStats {
        let total = self.total_syscalls.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ns.load(Ordering::Relaxed);
        
        SyscallPerformanceStats {
            total_syscalls: total,
            successful_syscalls: self.successful_syscalls.load(Ordering::Relaxed),
            failed_syscalls: self.failed_syscalls.load(Ordering::Relaxed),
            avg_latency_ns: if total > 0 { total_latency / total } else { 0 },
            fast_path_percentage: if total > 0 {
                (self.fast_path_syscalls.load(Ordering::Relaxed) * 100) / total
            } else { 0 },
            validation_failure_rate: if total > 0 {
                (self.validation_failures.load(Ordering::Relaxed) * 100) / total
            } else { 0 },
            security_violation_rate: if total > 0 {
                (self.security_violations.load(Ordering::Relaxed) * 100) / total
            } else { 0 },
            cache_hit_rate: self.get_cache_hit_rate(),
        }
    }

    /// Calculate cache hit rate
    pub fn get_cache_hit_rate(&self) -> u64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total > 0 {
            (hits * 100) / total
        } else {
            0
        }
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.total_syscalls.store(0, Ordering::Relaxed);
        self.successful_syscalls.store(0, Ordering::Relaxed);
        self.failed_syscalls.store(0, Ordering::Relaxed);
        self.total_latency_ns.store(0, Ordering::Relaxed);
        self.fast_path_syscalls.store(0, Ordering::Relaxed);
        self.standard_path_syscalls.store(0, Ordering::Relaxed);
        self.validation_failures.store(0, Ordering::Relaxed);
        self.security_violations.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.memory_allocations.store(0, Ordering::Relaxed);
        self.io_operations.store(0, Ordering::Relaxed);
    }
}

/// Latency histogram for performance analysis
#[derive(Debug)]
pub struct LatencyHistogram {
    /// Bucket boundaries (in nanoseconds)
    buckets: [u64; 20],
    /// Bucket counts
    counts: [AtomicU64; 20],
    /// Total samples
    total_samples: AtomicU64,
}

impl LatencyHistogram {
    pub fn new() -> Self {
        // Define latency buckets from 10ns to 10ms
        let buckets = [
            10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000,
            20000, 50000, 100000, 200000, 500000, 1000000, 2000000, 5000000, 10000000, u64::MAX
        ];
        
        let counts = core::array::from_fn(|_| AtomicU64::new(0));
        
        Self {
            buckets,
            counts,
            total_samples: AtomicU64::new(0),
        }
    }

    /// Record latency measurement
    pub fn record_latency(&self, latency_ns: u64) {
        if let Some(bucket_index) = self.find_bucket(latency_ns) {
            self.counts[bucket_index].fetch_add(1, Ordering::Relaxed);
            self.total_samples.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Find appropriate bucket for latency
    fn find_bucket(&self, latency_ns: u64) -> Option<usize> {
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if latency_ns <= bucket {
                return Some(i);
            }
        }
        None
    }

    /// Get histogram data
    pub fn get_histogram(&self) -> Vec<(u64, u64)> {
        let mut result = Vec::new();
        let total = self.total_samples.load(Ordering::Relaxed);
        
        for (i, &bucket) in self.buckets.iter().enumerate() {
            let count = self.counts[i].load(Ordering::Relaxed);
            let percentage = if total > 0 {
                (count * 100) / total
            } else { 0 };
            result.push((bucket, percentage));
        }
        
        result
    }

    /// Get percentile latency
    pub fn get_percentile(&self, percentile: f64) -> u64 {
        let total = self.total_samples.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        
        let target_count = ((percentile / 100.0) * total as f64) as u64;
        let mut cumulative = 0u64;
        
        for (i, &count) in self.counts.iter().enumerate() {
            cumulative += count.load(Ordering::Relaxed);
            if cumulative >= target_count {
                return self.buckets[i];
            }
        }
        
        self.buckets[self.buckets.len() - 1]
    }
}

/// Throughput tracker for measuring syscall throughput
#[derive(Debug)]
pub struct ThroughputTracker {
    /// Time windows for throughput calculation
    windows: [AtomicU64; 10],
    /// Window size in milliseconds
    window_size_ms: u64,
    /// Current window index
    current_window: AtomicUsize,
    /// Start time of current window
    window_start_time: AtomicU64,
}

impl ThroughputTracker {
    pub fn new(window_size_ms: u64) -> Self {
        Self {
            windows: core::array::from_fn(|_| AtomicU64::new(0)),
            window_size_ms,
            current_window: AtomicUsize::new(0),
            window_start_time: AtomicU64::new(0),
        }
    }

    /// Record syscall in current window
    pub fn record_syscall(&self) {
        let current_time = self.get_current_time_ms();
        let mut window_idx = self.current_window.load(Ordering::Relaxed);
        let window_start = self.window_start_time.load(Ordering::Relaxed);
        
        // Check if we need to move to next window
        if current_time - window_start >= self.window_size_ms {
            window_idx = (window_idx + 1) % self.windows.len();
            self.current_window.store(window_idx, Ordering::Relaxed);
            self.window_start_time.store(current_time, Ordering::Relaxed);
            self.windows[window_idx].store(0, Ordering::Relaxed);
        }
        
        self.windows[window_idx].fetch_add(1, Ordering::Relaxed);
    }

    /// Get current throughput (syscalls per second)
    pub fn get_throughput(&self) -> u64 {
        let mut total_syscalls = 0u64;
        
        for window in &self.windows {
            total_syscalls += window.load(Ordering::Relaxed);
        }
        
        // Calculate throughput based on time window
        let time_window_sec = (self.windows.len() as u64 * self.window_size_ms) / 1000;
        if time_window_sec > 0 {
            total_syscalls / time_window_sec
        } else {
            0
        }
    }

    /// Get current time in milliseconds
    fn get_current_time_ms(&self) -> u64 {
        // Simplified time function - would use actual timer in real implementation
        1000 // Placeholder
    }
}

/// Bottleneck analyzer for identifying performance bottlenecks
#[derive(Debug)]
pub struct BottleneckAnalyzer {
    /// Syscall frequency analysis
    syscall_frequency: Vec<(usize, u64)>,
    /// Average latency per syscall type
    syscall_latencies: Vec<(usize, u64)>,
    /// Resource utilization tracking
    resource_utilization: ResourceUtilization,
}

impl BottleneckAnalyzer {
    pub fn new() -> Self {
        Self {
            syscall_frequency: Vec::new(),
            syscall_latencies: Vec::new(),
            resource_utilization: ResourceUtilization::new(),
        }
    }

    /// Analyze performance bottlenecks
    pub fn analyze_bottlenecks(&mut self, stats: &SyscallPerformanceStats) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Check for high latency
        if stats.avg_latency_ns > 1000 { // > 1μs
            bottlenecks.push(PerformanceBottleneck::HighLatency {
                description: "Average syscall latency exceeds threshold".to_string(),
                severity: if stats.avg_latency_ns > 10000 { "High" } else { "Medium" },
                recommendation: "Consider optimizing hot path syscalls".to_string(),
            });
        }
        
        // Check for low fast path usage
        if stats.fast_path_percentage < 80 {
            bottlenecks.push(PerformanceBottleneck::LowFastPathUsage {
                description: "Fast path usage below optimal threshold".to_string(),
                current_percentage: stats.fast_path_percentage,
                target_percentage: 90,
                recommendation: "Move more syscalls to fast path".to_string(),
            });
        }
        
        // Check for high validation failure rate
        if stats.validation_failure_rate > 5 {
            bottlenecks.push(PerformanceBottleneck::HighValidationFailures {
                description: "High rate of parameter validation failures".to_string(),
                current_rate: stats.validation_failure_rate,
                recommendation: "Improve parameter validation or add caching".to_string(),
            });
        }
        
        // Check for security violations
        if stats.security_violation_rate > 1 {
            bottlenecks.push(PerformanceBottleneck::SecurityViolations {
                description: "Security violations detected".to_string(),
                current_rate: stats.security_violation_rate,
                recommendation: "Review security policies and access controls".to_string(),
            });
        }
        
        // Check cache hit rate
        if stats.cache_hit_rate < 80 {
            bottlenecks.push(PerformanceBottleneck::LowCacheHitRate {
                description: "Low cache hit rate detected".to_string(),
                current_rate: stats.cache_hit_rate,
                recommendation: "Optimize caching strategy".to_string(),
            });
        }
        
        bottlenecks
    }
}

/// Resource utilization tracking
#[derive(Debug)]
pub struct ResourceUtilization {
    /// CPU utilization percentage
    cpu_utilization: AtomicU64,
    /// Memory utilization in bytes
    memory_utilization: AtomicU64,
    /// I/O utilization percentage
    io_utilization: AtomicU64,
}

impl ResourceUtilization {
    pub fn new() -> Self {
        Self {
            cpu_utilization: AtomicU64::new(0),
            memory_utilization: AtomicU64::new(0),
            io_utilization: AtomicU64::new(0),
        }
    }

    /// Update resource utilization
    pub fn update_utilization(&self, cpu: u64, memory: u64, io: u64) {
        self.cpu_utilization.store(cpu, Ordering::Relaxed);
        self.memory_utilization.store(memory, Ordering::Relaxed);
        self.io_utilization.store(io, Ordering::Relaxed);
    }

    /// Get current utilization
    pub fn get_utilization(&self) -> (u64, u64, u64) {
        (
            self.cpu_utilization.load(Ordering::Relaxed),
            self.memory_utilization.load(Ordering::Relaxed),
            self.io_utilization.load(Ordering::Relaxed),
        )
    }
}

/// Performance bottleneck detection
#[derive(Debug)]
pub enum PerformanceBottleneck {
    HighLatency {
        description: String,
        severity: &'static str,
        recommendation: String,
    },
    LowFastPathUsage {
        description: String,
        current_percentage: u64,
        target_percentage: u64,
        recommendation: String,
    },
    HighValidationFailures {
        description: String,
        current_rate: u64,
        recommendation: String,
    },
    SecurityViolations {
        description: String,
        current_rate: u64,
        recommendation: String,
    },
    LowCacheHitRate {
        description: String,
        current_rate: u64,
        recommendation: String,
    },
}

/// Optimization engine for adaptive performance improvements
#[derive(Debug)]
pub struct OptimizationEngine {
    /// Current optimization settings
    settings: OptimizationSettings,
    /// Optimization history
    history: Vec<OptimizationAction>,
    /// Performance target thresholds
    targets: PerformanceTargets,
}

impl OptimizationEngine {
    pub fn new() -> Self {
        Self {
            settings: OptimizationSettings::default(),
            history: Vec::new(),
            targets: PerformanceTargets::default(),
        }
    }

    /// Apply optimizations based on performance analysis
    pub fn apply_optimizations(&mut self, bottlenecks: &[PerformanceBottleneck]) -> Vec<OptimizationAction> {
        let mut actions = Vec::new();
        
        for bottleneck in bottlenecks {
            let action = match bottleneck {
                PerformanceBottleneck::LowFastPathUsage { current_percentage, .. } => {
                    self.optimize_fast_path_usage(*current_percentage)
                }
                PerformanceBottleneck::LowCacheHitRate { current_rate, .. } => {
                    self.optimize_cache_strategy(*current_rate)
                }
                PerformanceBottleneck::HighLatency { .. } => {
                    self.optimize_latency()
                }
                PerformanceBottleneck::HighValidationFailures { .. } => {
                    self.optimize_validation()
                }
                PerformanceBottleneck::SecurityViolations { .. } => {
                    self.optimize_security()
                }
            };
            
            if let Some(action) = action {
                actions.push(action.clone());
                self.history.push(action);
            }
        }
        
        actions
    }

    /// Optimize fast path usage
    fn optimize_fast_path_usage(&self, current_percentage: u64) -> Option<OptimizationAction> {
        if current_percentage < 90 {
            Some(OptimizationAction::EnableFastPathOptimization {
                reason: "Low fast path usage detected".to_string(),
                expected_improvement: "10-20% latency reduction".to_string(),
            })
        } else {
            None
        }
    }

    /// Optimize cache strategy
    fn optimize_cache_strategy(&self, current_rate: u64) -> Option<OptimizationAction> {
        if current_rate < 80 {
            Some(OptimizationAction::IncreaseCacheSize {
                reason: "Low cache hit rate".to_string(),
                new_cache_size: 4096, // 4KB
                expected_improvement: "5-15% hit rate improvement".to_string(),
            })
        } else {
            None
        }
    }

    /// Optimize latency
    fn optimize_latency(&self) -> Option<OptimizationAction> {
        Some(OptimizationAction::EnableParameterCaching {
            reason: "High latency detected".to_string(),
            expected_improvement: "5-25% latency reduction".to_string(),
        })
    }

    /// Optimize validation
    fn optimize_validation(&self) -> Option<OptimizationAction> {
        Some(OptimizationAction::EnableValidationCaching {
            reason: "High validation failure rate".to_string(),
            cache_ttl_ms: 100,
            expected_improvement: "3-10% overall performance".to_string(),
        })
    }

    /// Optimize security
    fn optimize_security(&self) -> Option<OptimizationAction> {
        Some(OptimizationAction::TightenSecurityPolicies {
            reason: "Security violations detected".to_string(),
            policy_changes: vec!["Reduce wildcard permissions".to_string()],
            expected_improvement: "Better security compliance".to_string(),
        })
    }

    /// Get optimization recommendations
    pub fn get_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Analyze history for patterns
        if self.history.len() > 10 {
            recommendations.push(OptimizationRecommendation::AnalyzeOptimizationHistory {
                history_length: self.history.len(),
                suggestion: "Consider reviewing optimization effectiveness".to_string(),
            });
        }
        
        recommendations
    }
}

/// Real-time performance monitor
#[derive(Debug)]
pub struct RealTimeMonitor {
    /// Performance alert thresholds
    thresholds: PerformanceThresholds,
    /// Current performance state
    current_state: PerformanceState,
    /// Alert callbacks
    alert_callbacks: Vec<AlertCallback>,
}

impl RealTimeMonitor {
    pub fn new() -> Self {
        Self {
            thresholds: PerformanceThresholds::default(),
            current_state: PerformanceState::Normal,
            alert_callbacks: Vec::new(),
        }
    }

    /// Monitor performance in real-time
    pub fn monitor_performance(&mut self, stats: &SyscallPerformanceStats) {
        let mut new_state = PerformanceState::Normal;
        
        // Check latency threshold
        if stats.avg_latency_ns > self.thresholds.latency_warning_ns {
            new_state = PerformanceState::Warning;
        }
        
        if stats.avg_latency_ns > self.thresholds.latency_critical_ns {
            new_state = PerformanceState::Critical;
        }
        
        // Check failure rate threshold
        let failure_rate = if stats.total_syscalls > 0 {
            (stats.failed_syscalls * 100) / stats.total_syscalls
        } else { 0 };
        
        if failure_rate > self.thresholds.failure_rate_warning {
            new_state = PerformanceState::Warning;
        }
        
        if failure_rate > self.thresholds.failure_rate_critical {
            new_state = PerformanceState::Critical;
        }
        
        // Check fast path usage
        if stats.fast_path_percentage < self.thresholds.fast_path_warning {
            new_state = PerformanceState::Warning;
        }
        
        // Update state and trigger alerts if needed
        if new_state != self.current_state {
            self.handle_state_change(new_state, stats);
            self.current_state = new_state;
        }
    }

    /// Handle performance state change
    fn handle_state_change(&mut self, new_state: PerformanceState, stats: &SyscallPerformanceStats) {
        match new_state {
            PerformanceState::Warning => {
                warn!("Performance warning: Avg latency {}ns, Success rate {}%", 
                      stats.avg_latency_ns, 
                      (stats.successful_syscalls * 100) / stats.total_syscalls.max(1));
            }
            PerformanceState::Critical => {
                error!("Performance critical: Avg latency {}ns, Failure rate {}%", 
                       stats.avg_latency_ns,
                       (stats.failed_syscalls * 100) / stats.total_syscalls.max(1));
            }
            PerformanceState::Normal => {
                info!("Performance returned to normal");
            }
        }
        
        // Trigger alert callbacks
        for callback in &self.alert_callbacks {
            callback(new_state, stats);
        }
    }

    /// Add alert callback
    pub fn add_alert_callback(&mut self, callback: AlertCallback) {
        self.alert_callbacks.push(callback);
    }
}

/// Performance state levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceState {
    Normal,
    Warning,
    Critical,
}

/// Performance thresholds for alerts
#[derive(Debug)]
pub struct PerformanceThresholds {
    pub latency_warning_ns: u64,
    pub latency_critical_ns: u64,
    pub failure_rate_warning: u64,
    pub failure_rate_critical: u64,
    pub fast_path_warning: u64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            latency_warning_ns: 1000,      // 1μs
            latency_critical_ns: 10000,    // 10μs
            failure_rate_warning: 5,       // 5%
            failure_rate_critical: 10,     // 10%
            fast_path_warning: 80,         // 80%
        }
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct SyscallPerformanceStats {
    pub total_syscalls: u64,
    pub successful_syscalls: u64,
    pub failed_syscalls: u64,
    pub avg_latency_ns: u64,
    pub fast_path_percentage: u64,
    pub validation_failure_rate: u64,
    pub security_violation_rate: u64,
    pub cache_hit_rate: u64,
}

/// Optimization actions
#[derive(Debug, Clone)]
pub enum OptimizationAction {
    EnableFastPathOptimization {
        reason: String,
        expected_improvement: String,
    },
    IncreaseCacheSize {
        reason: String,
        new_cache_size: usize,
        expected_improvement: String,
    },
    EnableParameterCaching {
        reason: String,
        expected_improvement: String,
    },
    EnableValidationCaching {
        reason: String,
        cache_ttl_ms: u64,
        expected_improvement: String,
    },
    TightenSecurityPolicies {
        reason: String,
        policy_changes: Vec<String>,
        expected_improvement: String,
    },
}

/// Performance optimization recommendations
#[derive(Debug)]
pub enum OptimizationRecommendation {
    AnalyzeOptimizationHistory {
        history_length: usize,
        suggestion: String,
    },
}

/// Optimization settings
#[derive(Debug, Clone)]
pub struct OptimizationSettings {
    pub enable_fast_path: bool,
    pub enable_caching: bool,
    pub enable_adaptive_optimization: bool,
    pub performance_target_ns: u64,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            enable_fast_path: true,
            enable_caching: true,
            enable_adaptive_optimization: true,
            performance_target_ns: 1000, // 1μs target
        }
    }
}

/// Performance targets
#[derive(Debug)]
pub struct PerformanceTargets {
    pub max_avg_latency_ns: u64,
    pub min_fast_path_percentage: u64,
    pub max_failure_rate: u64,
    pub min_cache_hit_rate: u64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            max_avg_latency_ns: 5000,   // 5μs
            min_fast_path_percentage: 90,
            max_failure_rate: 2,
            min_cache_hit_rate: 85,
        }
    }
}

/// Performance error types
#[derive(Debug)]
pub enum PerformanceError {
    InvalidMetrics,
    OptimizationFailed(String),
    ThresholdExceeded(String),
}

/// Alert callback type
pub type AlertCallback = fn(PerformanceState, &SyscallPerformanceStats);

// Global performance manager
use spin::Mutex;
static PERFORMANCE_MANAGER: Mutex<Option<SyscallPerformanceManager>> = Mutex::new(None);

/// Initialize performance manager
pub fn init_performance_manager() -> Result<(), PerformanceError> {
    let mut manager_guard = PERFORMANCE_MANAGER.lock();
    
    if manager_guard.is_some() {
        return Err(PerformanceError::OptimizationFailed("Manager already initialized".to_string()));
    }
    
    let manager = SyscallPerformanceManager {
        counters: PerformanceCounters::new(),
        latency_histogram: LatencyHistogram::new(),
        throughput_tracker: ThroughputTracker::new(1000), // 1 second windows
        bottleneck_analyzer: BottleneckAnalyzer::new(),
        optimization_engine: OptimizationEngine::new(),
        real_time_monitor: RealTimeMonitor::new(),
    };
    
    *manager_guard = Some(manager);
    
    info!("Performance manager initialized");
    Ok(())
}

/// Get global performance manager
pub fn get_performance_manager() -> Option<Mutex<SyscallPerformanceManager>> {
    PERFORMANCE_MANAGER.lock().as_ref().map(|_| PERFORMANCE_MANAGER.clone())
}

/// Record system call performance
pub fn record_syscall_performance(latency_ns: u64, success: bool, fast_path: bool) {
    let mut manager_guard = PERFORMANCE_MANAGER.lock();
    
    if let Some(manager) = manager_guard.as_mut() {
        manager.counters.record_syscall(latency_ns, success, fast_path);
        manager.latency_histogram.record_latency(latency_ns);
        manager.throughput_tracker.record_syscall();
        
        let stats = manager.counters.get_stats();
        manager.real_time_monitor.monitor_performance(&stats);
    }
}

/// Get performance report
pub fn get_performance_report() -> Option<SyscallPerformanceStats> {
    let manager_guard = PERFORMANCE_MANAGER.lock();
    manager_guard.as_ref().map(|manager| manager.counters.get_stats())
}

/// Analyze and optimize performance
pub fn analyze_and_optimize_performance() -> Vec<OptimizationAction> {
    let mut manager_guard = PERFORMANCE_MANAGER.lock();
    
    if let Some(manager) = manager_guard.as_mut() {
        let stats = manager.counters.get_stats();
        let bottlenecks = manager.bottleneck_analyzer.analyze_bottlenecks(&stats);
        let actions = manager.optimization_engine.apply_optimizations(&bottlenecks);
        actions
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_counters() {
        let counters = PerformanceCounters::new();
        
        counters.record_syscall(1000, true, true);
        counters.record_syscall(2000, false, false);
        
        let stats = counters.get_stats();
        assert_eq!(stats.total_syscalls, 2);
        assert_eq!(stats.successful_syscalls, 1);
        assert_eq!(stats.failed_syscalls, 1);
        assert_eq!(stats.avg_latency_ns, 1500);
    }

    #[test]
    fn test_latency_histogram() {
        let histogram = LatencyHistogram::new();
        
        histogram.record_latency(500);
        histogram.record_latency(1500);
        histogram.record_latency(5000);
        
        let histogram_data = histogram.get_histogram();
        assert!(histogram_data.len() > 0);
        
        let p99 = histogram.get_percentile(99.0);
        assert!(p99 >= 5000);
    }

    #[test]
    fn test_throughput_tracker() {
        let tracker = ThroughputTracker::new(1000); // 1 second windows
        
        // Record some syscalls
        for _ in 0..100 {
            tracker.record_syscall();
        }
        
        let throughput = tracker.get_throughput();
        assert!(throughput > 0);
    }

    #[test]
    fn test_bottleneck_analyzer() {
        let mut analyzer = BottleneckAnalyzer::new();
        
        let stats = SyscallPerformanceStats {
            total_syscalls: 1000,
            successful_syscalls: 900,
            failed_syscalls: 100,
            avg_latency_ns: 2000,
            fast_path_percentage: 70,
            validation_failure_rate: 10,
            security_violation_rate: 2,
            cache_hit_rate: 60,
        };
        
        let bottlenecks = analyzer.analyze_bottlenecks(&stats);
        assert!(!bottlenecks.is_empty());
    }
}