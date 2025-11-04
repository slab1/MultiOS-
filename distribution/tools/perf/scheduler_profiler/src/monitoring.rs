//! Performance Monitoring Module
//! 
//! Real-time performance monitoring for MultiOS scheduler algorithms with focus on:
//! - Context switch overhead measurement
//! - CPU utilization and load balancing analysis
//! - Priority inversion detection
//! - Fairness and responsiveness metrics

use crate::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, Duration};

/// Real-time performance monitor for scheduler metrics
pub struct PerformanceMonitor {
    config: ProfilerConfig,
    /// Hardware performance counters
    hardware_counters: Vec<HardwareCounter>,
    /// Context switch measurements
    context_switch_measurements: VecDeque<ContextSwitchMeasurement>,
    /// Scheduling latency measurements
    scheduling_latencies: VecDeque<SchedulingLatencyMeasurement>,
    /// Priority inversion detector
    priority_inversion_detector: PriorityInversionDetector,
    /// Fairness calculator
    fairness_calculator: FairnessCalculator,
    /// Load balancing analyzer
    load_balancing_analyzer: LoadBalancingAnalyzer,
}

/// Hardware performance counter
#[derive(Debug, Clone)]
pub struct HardwareCounter {
    pub counter_type: CounterType,
    pub cpu_id: usize,
    pub value: AtomicU64,
    pub enabled: bool,
}

/// Types of performance counters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterType {
    InstructionsRetired,
    CpuCycles,
    CacheReferences,
    CacheMisses,
    BranchInstructions,
    BranchMispredictions,
    ContextSwitches,
    CpuMigrations,
    PriorityInversions,
    LoadBalancingEvents,
    SchedulingLatencies,
}

/// Context switch measurement
#[derive(Debug, Clone)]
pub struct ContextSwitchMeasurement {
    pub timestamp: Instant,
    pub from_thread: u64,
    pub to_thread: u64,
    pub from_cpu: usize,
    pub to_cpu: usize,
    pub overhead_cycles: u64,
    pub overhead_nanos: u64,
    pub algorithm: SchedulerAlgorithm,
}

/// Scheduling latency measurement
#[derive(Debug, Clone)]
pub struct SchedulingLatencyMeasurement {
    pub timestamp: Instant,
    pub thread_id: u64,
    pub latency_nanos: u64,
    pub cpu_id: usize,
    pub algorithm: SchedulerAlgorithm,
    pub priority: u8,
}

/// Priority inversion detection
#[derive(Debug, Clone)]
pub struct PriorityInversionMeasurement {
    pub timestamp: Instant,
    pub low_priority_thread: u64,
    pub high_priority_thread: u64,
    pub lock_address: u64,
    pub inversion_duration: Duration,
    pub resolution_method: String,
}

/// Priority inversion detector
pub struct PriorityInversionDetector {
    /// Lock acquisition tracking
    lock_acquisitions: Vec<LockAcquisition>,
    /// Thread priority tracking
    thread_priorities: std::collections::HashMap<u64, u8>,
    /// Detected inversions
    detected_inversions: Vec<PriorityInversionMeasurement>,
}

/// Lock acquisition tracking
#[derive(Debug, Clone)]
pub struct LockAcquisition {
    pub thread_id: u64,
    pub lock_address: u64,
    pub acquisition_time: Instant,
    pub release_time: Option<Instant>,
    pub priority: u8,
}

/// Fairness calculator using Jain's fairness index
pub struct FairnessCalculator {
    /// Thread execution times
    thread_execution_times: std::collections::HashMap<u64, Vec<f32>>,
    /// Fairness history
    fairness_history: VecDeque<FairnessIndex>,
}

/// Fairness index calculation
#[derive(Debug, Clone)]
pub struct FairnessIndex {
    pub timestamp: Instant,
    pub jains_index: f32,
    pub thread_count: usize,
    pub execution_times: Vec<f32>,
}

/// Load balancing analyzer
pub struct LoadBalancingAnalyzer {
    /// CPU loads
    cpu_loads: Vec<f32>,
    /// Load balancing events
    load_balance_events: Vec<LoadBalanceEvent>,
    /// Migration tracking
    migrations: Vec<ThreadMigration>,
}

/// Load balancing event
#[derive(Debug, Clone)]
pub struct LoadBalanceEvent {
    pub timestamp: Instant,
    pub source_cpu: usize,
    pub target_cpu: usize,
    pub thread_id: u64,
    pub migration_reason: String,
    pub load_before: f32,
    pub load_after: f32,
    pub performance_impact: f32,
}

/// Thread migration tracking
#[derive(Debug, Clone)]
pub struct ThreadMigration {
    pub timestamp: Instant,
    pub thread_id: u64,
    pub from_cpu: usize,
    pub to_cpu: usize,
    pub migration_cost_cycles: u64,
    pub performance_benefit: f32,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(config: ProfilerConfig) -> Self {
        let hardware_counters = if config.enable_counters {
            Self::initialize_hardware_counters(config.core_count)
        } else {
            Vec::new()
        };

        Self {
            config,
            hardware_counters,
            context_switch_measurements: VecDeque::with_capacity(10000),
            scheduling_latencies: VecDeque::with_capacity(10000),
            priority_inversion_detector: PriorityInversionDetector::new(),
            fairness_calculator: FairnessCalculator::new(),
            load_balancing_analyzer: LoadBalancingAnalyzer::new(),
        }
    }

    /// Initialize hardware performance counters
    fn initialize_hardware_counters(core_count: usize) -> Vec<HardwareCounter> {
        let mut counters = Vec::new();
        
        let counter_types = vec![
            CounterType::InstructionsRetired,
            CounterType::CpuCycles,
            CounterType::CacheReferences,
            CounterType::CacheMisses,
            CounterType::BranchInstructions,
            CounterType::BranchMispredictions,
            CounterType::ContextSwitches,
            CounterType::CpuMigrations,
        ];

        for cpu_id in 0..core_count {
            for counter_type in &counter_types {
                counters.push(HardwareCounter {
                    counter_type: counter_type.clone(),
                    cpu_id,
                    value: AtomicU64::new(0),
                    enabled: true,
                });
            }
        }

        counters
    }

    /// Collect a comprehensive performance sample
    pub async fn collect_sample(&self) -> Result<PerformanceSample, Box<dyn std::error::Error>> {
        let timestamp = Utc::now();
        
        // Collect CPU utilization
        let cpu_utilization = self.collect_cpu_utilization().await?;
        
        // Collect scheduling latency metrics
        let scheduling_latency = self.collect_scheduling_latency().await?;
        
        // Collect context switch overhead
        let context_switch_overhead = self.collect_context_switch_overhead().await?;
        
        // Calculate load balancing efficiency
        let load_balancing_efficiency = self.calculate_load_balancing_efficiency().await?;
        
        // Calculate fairness index
        let fairness_index = self.calculate_fairness_index().await?;
        
        // Calculate responsiveness score
        let responsiveness_score = self.calculate_responsiveness_score().await?;
        
        // Calculate throughput
        let throughput = self.calculate_throughput().await?;

        Ok(PerformanceSample {
            timestamp,
            cpu_utilization,
            scheduling_latency,
            context_switch_overhead,
            load_balancing_efficiency,
            fairness_index,
            responsiveness_score,
            throughput,
            algorithm: self.config.algorithm,
            core_count: self.config.core_count,
        })
    }

    /// Collect CPU utilization per core
    async fn collect_cpu_utilization(&self) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let mut utilization = Vec::with_capacity(self.config.core_count);
        
        for cpu_id in 0..self.config.core_count {
            // Simulate CPU utilization measurement
            // In real implementation, this would read from /proc/stat or similar
            let cpu_util = self.measure_cpu_utilization(cpu_id).await?;
            utilization.push(cpu_util);
        }
        
        Ok(utilization)
    }

    /// Measure CPU utilization for a specific core
    async fn measure_cpu_utilization(&self, cpu_id: usize) -> Result<f32, Box<dyn std::error::Error>> {
        // Simulated measurement - in real implementation would use:
        // - perf_event_open() system call
        // - Reading /proc/stat for CPU times
        // - Hardware performance counters
        
        // Generate realistic utilization based on current load
        let base_util = 0.6; // 60% base utilization
        let variation = (cpu_id as f32 * 0.1) % 0.4; // Add some variation per CPU
        let utilization = (base_util + variation).min(1.0).max(0.0);
        
        Ok(utilization)
    }

    /// Collect scheduling latency metrics
    async fn collect_scheduling_latency(&self) -> Result<SchedulingLatency, Box<dyn std::error::Error>> {
        let recent_latencies: Vec<u64> = self.scheduling_latencies
            .iter()
            .rev()
            .take(1000)
            .map(|m| m.latency_nanos)
            .collect();

        if recent_latencies.is_empty() {
            return Ok(SchedulingLatency {
                min_ns: 0,
                max_ns: 0,
                avg_ns: 0.0,
                p95_ns: 0,
                p99_ns: 0,
            });
        }

        let min_ns = recent_latencies.iter().min().copied().unwrap_or(0);
        let max_ns = recent_latencies.iter().max().copied().unwrap_or(0);
        let avg_ns = recent_latencies.iter().sum::<u64>() as f64 / recent_latencies.len() as f64;
        
        let mut sorted_latencies = recent_latencies;
        sorted_latencies.sort();
        
        let p95_idx = (sorted_latencies.len() as f64 * 0.95) as usize;
        let p99_idx = (sorted_latencies.len() as f32 * 0.99) as usize;
        
        let p95_ns = sorted_latencies.get(p95_idx).copied().unwrap_or(max_ns);
        let p99_ns = sorted_latencies.get(p99_idx).copied().unwrap_or(max_ns);

        Ok(SchedulingLatency {
            min_ns,
            max_ns,
            avg_ns,
            p95_ns,
            p99_ns,
        })
    }

    /// Collect context switch overhead metrics
    async fn collect_context_switch_overhead(&self) -> Result<ContextSwitchOverhead, Box<dyn std::error::Error>> {
        let recent_switches: Vec<&ContextSwitchMeasurement> = self.context_switch_measurements
            .iter()
            .rev()
            .take(5000)
            .collect();

        if recent_switches.is_empty() {
            return Ok(ContextSwitchOverhead {
                min_cycles: 0,
                max_cycles: 0,
                avg_cycles: 0.0,
                min_microseconds: 0,
                max_microseconds: 0,
                avg_microseconds: 0.0,
                total_switches: 0,
            });
        }

        let cycles: Vec<u64> = recent_switches.iter().map(|m| m.overhead_cycles).collect();
        let micros: Vec<u64> = recent_switches.iter().map(|m| m.overhead_nanos / 1000).collect();

        let min_cycles = cycles.iter().min().copied().unwrap_or(0);
        let max_cycles = cycles.iter().max().copied().unwrap_or(0);
        let avg_cycles = cycles.iter().sum::<u64>() as f64 / cycles.len() as f64;
        
        let min_micros = micros.iter().min().copied().unwrap_or(0);
        let max_micros = micros.iter().max().copied().unwrap_or(0);
        let avg_micros = micros.iter().sum::<u64>() as f64 / micros.len() as f64;

        Ok(ContextSwitchOverhead {
            min_cycles,
            max_cycles,
            avg_cycles,
            min_microseconds: min_micros,
            max_microseconds: max_micros,
            avg_microseconds: avg_micros,
            total_switches: recent_switches.len() as u64,
        })
    }

    /// Calculate load balancing efficiency
    async fn calculate_load_balancing_efficiency(&self) -> Result<f32, Box<dyn std::error::Error>> {
        // Load balancing efficiency is measured as:
        // 1.0 - (load_variance / max_load)
        // Higher values indicate better load distribution
        
        let loads = self.load_balancing_analyzer.get_current_loads();
        
        if loads.is_empty() {
            return Ok(1.0);
        }

        let mean_load: f32 = loads.iter().sum::<f32>() / loads.len() as f32;
        let variance: f32 = loads.iter()
            .map(|&load| (load - mean_load).powi(2))
            .sum::<f32>() / loads.len() as f32;
        
        let max_load = loads.iter().fold(0.0f32, |a, &b| a.max(b));
        
        if max_load > 0.0 {
            let efficiency = 1.0 - (variance.sqrt() / max_load);
            Ok(efficiency.max(0.0).min(1.0))
        } else {
            Ok(1.0)
        }
    }

    /// Calculate fairness index using Jain's fairness index
    async fn calculate_fairness_index(&self) -> Result<f32, Box<dyn std::error::Error>> {
        // Jain's fairness index: (sum(xi)^2) / (n * sum(xi^2))
        // Where xi are the throughput/allocated resources for each thread
        
        let execution_times = self.fairness_calculator.get_current_execution_times();
        
        if execution_times.is_empty() {
            return Ok(1.0);
        }

        let sum: f32 = execution_times.iter().sum();
        let sum_squares: f32 = execution_times.iter().map(|&x| x * x).sum();
        let n = execution_times.len() as f32;
        
        if sum_squares > 0.0 {
            let jains_index = (sum * sum) / (n * sum_squares);
            Ok(jains_index.max(0.0).min(1.0))
        } else {
            Ok(1.0)
        }
    }

    /// Calculate responsiveness score
    async fn calculate_responsiveness_score(&self) -> Result<f32, Box<dyn std::error::Error>> {
        // Responsiveness is measured as:
        // - Average response time for interactive threads
        // - Variance in response times
        // - Ratio of threads meeting response time targets
        
        let interactive_threads = self.get_interactive_thread_count().await;
        let avg_response_time = self.get_average_response_time().await;
        
        // Normalize response time (assume 100ms is good, 1s is bad)
        let normalized_response_time = (avg_response_time.as_secs_f32() / 0.1).min(1.0);
        
        // Base score on normalized response time and interactive thread count
        let base_score = 1.0 - normalized_response_time;
        let thread_factor = (interactive_threads as f32 / 10.0).min(1.0); // Assume 10 interactive threads is good
        
        Ok((base_score * 0.7 + thread_factor * 0.3).max(0.0).min(1.0))
    }

    /// Calculate system throughput
    async fn calculate_throughput(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // Throughput is measured as completed tasks per second
        // This would be calculated based on actual completed work
        
        // Simulated throughput calculation
        let base_throughput = 1000.0; // Base 1000 tasks/second
        let utilization_factor = self.calculate_average_cpu_utilization();
        
        Ok(base_throughput * utilization_factor as f64)
    }

    /// Calculate average CPU utilization
    fn calculate_average_cpu_utilization(&self) -> f32 {
        // Simplified calculation - would use actual measurement data
        0.75 // 75% average utilization
    }

    /// Record context switch event
    pub fn record_context_switch(&mut self, from_thread: u64, to_thread: u64, from_cpu: usize, to_cpu: usize) {
        // Simulate context switch overhead measurement
        let overhead_cycles = self.simulate_context_switch_cost(from_cpu, to_cpu);
        let overhead_nanos = self.cycles_to_nanos(overhead_cycles);
        
        let measurement = ContextSwitchMeasurement {
            timestamp: Instant::now(),
            from_thread,
            to_thread,
            from_cpu,
            to_cpu,
            overhead_cycles,
            overhead_nanos,
            algorithm: self.config.algorithm,
        };

        self.context_switch_measurements.push_back(measurement);
        
        // Keep only recent measurements to limit memory usage
        if self.context_switch_measurements.len() > 10000 {
            self.context_switch_measurements.pop_front();
        }
    }

    /// Record scheduling latency
    pub fn record_scheduling_latency(&mut self, thread_id: u64, latency_nanos: u64, cpu_id: usize, priority: u8) {
        let measurement = SchedulingLatencyMeasurement {
            timestamp: Instant::now(),
            thread_id,
            latency_nanos,
            cpu_id,
            algorithm: self.config.algorithm,
            priority,
        };

        self.scheduling_latencies.push_back(measurement);
        
        // Keep only recent measurements
        if self.scheduling_latencies.len() > 10000 {
            self.scheduling_latencies.pop_front();
        }
    }

    /// Check for performance anomalies
    pub async fn check_anomalies(&self) {
        // Check for high scheduling latency
        if let Ok(latency) = self.collect_scheduling_latency().await {
            if latency.avg_ns > 1000000 { // 1ms average is concerning
                println!("Warning: High scheduling latency detected: {} ns", latency.avg_ns);
            }
        }

        // Check for high context switch overhead
        if let Ok(overhead) = self.collect_context_switch_overhead().await {
            if overhead.avg_microseconds > 10.0 { // 10µs average is concerning
                println!("Warning: High context switch overhead detected: {} µs", overhead.avg_microseconds);
            }
        }
    }

    /// Simulate context switch cost
    fn simulate_context_switch_cost(&self, from_cpu: usize, to_cpu: usize) -> u64 {
        // Base cost for context switch
        let base_cost = 2000; // cycles
        
        // Additional cost for cross-CPU migration
        let migration_cost = if from_cpu != to_cpu { 1000 } else { 0 };
        
        // Some randomness to simulate real behavior
        let random_factor = (rand::random::<u32>() % 500) as u64;
        
        base_cost + migration_cost + random_factor
    }

    /// Convert cycles to nanoseconds (assuming 3GHz CPU)
    fn cycles_to_nanos(&self, cycles: u64) -> u64 {
        let cpu_frequency_hz = 3_000_000_000; // 3GHz
        let nanos_per_cycle = 1_000_000_000 / cpu_frequency_hz;
        cycles * nanos_per_cycle
    }

    /// Get interactive thread count (simulated)
    async fn get_interactive_thread_count(&self) -> usize {
        // In real implementation, this would identify threads with I/O or interactive characteristics
        rand::random::<usize>() % 20 // Random between 0-19
    }

    /// Get average response time (simulated)
    async fn get_average_response_time(&self) -> Duration {
        let micros = (rand::random::<u32>() % 500000) + 10000; // 10ms to 500ms
        Duration::from_micros(micros as u64)
    }
}

impl PriorityInversionDetector {
    fn new() -> Self {
        Self {
            lock_acquisitions: Vec::new(),
            thread_priorities: std::collections::HashMap::new(),
            detected_inversions: Vec::new(),
        }
    }

    /// Update thread priority
    pub fn update_thread_priority(&mut self, thread_id: u64, priority: u8) {
        self.thread_priorities.insert(thread_id, priority);
    }

    /// Record lock acquisition
    pub fn record_lock_acquisition(&mut self, thread_id: u64, lock_address: u64) {
        let priority = self.thread_priorities.get(&thread_id).copied().unwrap_or(0);
        
        self.lock_acquisitions.push(LockAcquisition {
            thread_id,
            lock_address,
            acquisition_time: Instant::now(),
            release_time: None,
            priority,
        });
    }

    /// Record lock release
    pub fn record_lock_release(&mut self, thread_id: u64, lock_address: u64) {
        if let Some(acquisition) = self.lock_acquisitions.iter_mut().find(|a| 
            a.thread_id == thread_id && 
            a.lock_address == lock_address && 
            a.release_time.is_none()
        ) {
            acquisition.release_time = Some(Instant::now());
            
            // Check for priority inversion
            self.check_priority_inversion(acquisition);
        }
    }

    /// Check for priority inversion
    fn check_priority_inversion(&mut self, acquisition: &LockAcquisition) {
        // Find if there are higher priority threads waiting for this lock
        let mut waiting_high_priority = false;
        
        for (thread_id, &priority) in &self.thread_priorities {
            if *thread_id != acquisition.thread_id && priority > acquisition.priority {
                // Check if this thread is waiting (simplified check)
                if self.is_thread_waiting(*thread_id) {
                    waiting_high_priority = true;
                    break;
                }
            }
        }

        if waiting_high_priority && acquisition.priority < 5 { // Lower priority threshold
            let inversion = PriorityInversionMeasurement {
                timestamp: Instant::now(),
                low_priority_thread: acquisition.thread_id,
                high_priority_thread: 0, // Would be determined from waiting threads
                lock_address: acquisition.lock_address,
                inversion_duration: acquisition.release_time.unwrap() - acquisition.acquisition_time,
                resolution_method: "priority_inheritance".to_string(),
            };
            
            self.detected_inversions.push(inversion);
        }
    }

    /// Check if thread is waiting (simplified)
    fn is_thread_waiting(&self, _thread_id: u64) -> bool {
        // In real implementation, this would check thread state
        rand::random::<bool>()
    }
}

impl FairnessCalculator {
    fn new() -> Self {
        Self {
            thread_execution_times: std::collections::HashMap::new(),
            fairness_history: VecDeque::with_capacity(1000),
        }
    }

    /// Update thread execution time
    pub fn update_execution_time(&mut self, thread_id: u64, execution_time: f32) {
        self.thread_execution_times
            .entry(thread_id)
            .or_insert_with(Vec::new)
            .push(execution_time);
        
        // Keep only recent execution times
        if let Some(times) = self.thread_execution_times.get_mut(&thread_id) {
            if times.len() > 100 {
                times.drain(0..10);
            }
        }
    }

    /// Get current execution times for fairness calculation
    fn get_current_execution_times(&self) -> Vec<f32> {
        self.thread_execution_times
            .values()
            .filter_map(|times| times.last().copied())
            .collect()
    }
}

impl LoadBalancingAnalyzer {
    fn new() -> Self {
        Self {
            cpu_loads: Vec::new(),
            load_balance_events: Vec::new(),
            migrations: Vec::new(),
        }
    }

    /// Update CPU load
    pub fn update_cpu_load(&mut self, cpu_id: usize, load: f32) {
        if self.cpu_loads.len() <= cpu_id {
            self.cpu_loads.resize(cpu_id + 1, 0.0);
        }
        self.cpu_loads[cpu_id] = load;
    }

    /// Record load balancing event
    pub fn record_load_balance(&mut self, source_cpu: usize, target_cpu: usize, thread_id: u64, reason: String) {
        let load_before = self.cpu_loads.get(source_cpu).copied().unwrap_or(0.0);
        let load_after = self.cpu_loads.get(target_cpu).copied().unwrap_or(0.0);
        
        let event = LoadBalanceEvent {
            timestamp: Instant::now(),
            source_cpu,
            target_cpu,
            thread_id,
            migration_reason: reason,
            load_before,
            load_after,
            performance_impact: self.calculate_migration_impact(load_before, load_after),
        };
        
        self.load_balance_events.push(event);
    }

    /// Calculate migration impact
    fn calculate_migration_impact(&self, load_before: f32, load_after: f32) -> f32 {
        // Positive impact if load becomes more balanced
        let imbalance_before = (load_before - 0.5).abs();
        let imbalance_after = ((load_before - 1.0) + load_after - 0.5).abs();
        (imbalance_before - imbalance_after).max(0.0)
    }

    /// Get current CPU loads
    fn get_current_loads(&self) -> Vec<f32> {
        self.cpu_loads.clone()
    }
}