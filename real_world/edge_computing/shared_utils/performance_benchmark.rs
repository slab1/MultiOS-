//! Performance Benchmarking Utilities for Edge Computing
//! MultiOS Edge Computing Demonstrations

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Benchmark metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub power_consumption_watts: f64,
    pub error_rate_percent: f64,
    pub success_rate_percent: f64,
}

/// Latency measurement
#[derive(Debug, Clone)]
pub struct LatencyMeasurer {
    measurements: VecDeque<Duration>,
    max_samples: usize,
}

impl LatencyMeasurer {
    pub fn new(max_samples: usize) -> Self {
        Self {
            measurements: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record_latency(&mut self, duration: Duration) {
        self.measurements.push_back(duration);
        if self.measurements.len() > self.max_samples {
            self.measurements.pop_front();
        }
    }

    pub fn get_statistics(&self) -> Option<BenchmarkMetrics> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut latencies_ms: Vec<f64> = self.measurements
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();

        latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = latencies_ms.len() as f64;
        let sum: f64 = latencies_ms.iter().sum();
        let min = latencies_ms[0];
        let max = latencies_ms[latencies_ms.len() - 1];
        let avg = sum / count;
        let p95 = latencies_ms[(count * 0.95) as usize];
        let p99 = latencies_ms[(count * 0.99) as usize];

        Some(BenchmarkMetrics {
            min_latency_ms: min,
            max_latency_ms: max,
            avg_latency_ms: avg,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            throughput_ops_per_sec: 1000.0 / avg, // operations per second
            cpu_usage_percent: 0.0, // To be filled by system monitoring
            memory_usage_mb: 0.0,   // To be filled by system monitoring
            power_consumption_watts: 0.0, // To be filled by power monitoring
            error_rate_percent: 0.0, // To be filled by error tracking
            success_rate_percent: 100.0, // To be filled by success tracking
        })
    }
}

/// Throughput benchmarker
#[derive(Debug)]
pub struct ThroughputBenchmarker {
    start_time: Option<Instant>,
    operation_count: u64,
    errors: u64,
}

impl ThroughputBenchmarker {
    pub fn new() -> Self {
        Self {
            start_time: None,
            operation_count: 0,
            errors: 0,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.operation_count = 0;
        self.errors = 0;
    }

    pub fn record_operation(&mut self) {
        if self.start_time.is_some() {
            self.operation_count += 1;
        }
    }

    pub fn record_error(&mut self) {
        if self.start_time.is_some() {
            self.errors += 1;
        }
    }

    pub fn get_throughput(&self) -> Option<f64> {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            let throughput = self.operation_count as f64 / elapsed.as_secs_f64();
            Some(throughput)
        } else {
            None
        }
    }

    pub fn get_success_rate(&self) -> f64 {
        let total = self.operation_count + self.errors;
        if total > 0 {
            (self.operation_count as f64 / total as f64) * 100.0
        } else {
            100.0
        }
    }
}

/// Edge device performance profiler
#[derive(Debug)]
pub struct EdgeProfiler {
    cpu_usage: VecDeque<f64>,
    memory_usage: VecDeque<f64>,
    temperature: VecDeque<f64>,
    power_usage: VecDeque<f64>,
    max_samples: usize,
}

impl EdgeProfiler {
    pub fn new(max_samples: usize) -> Self {
        Self {
            cpu_usage: VecDeque::with_capacity(max_samples),
            memory_usage: VecDeque::with_capacity(max_samples),
            temperature: VecDeque::with_capacity(max_samples),
            power_usage: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record_metrics(&mut self, cpu: f64, memory: f64, temp: f64, power: f64) {
        self.cpu_usage.push_back(cpu);
        self.memory_usage.push_back(memory);
        self.temperature.push_back(temp);
        self.power_usage.push_back(power);

        if self.cpu_usage.len() > self.max_samples {
            self.cpu_usage.pop_front();
            self.memory_usage.pop_front();
            self.temperature.pop_front();
            self.power_usage.pop_front();
        }
    }

    pub fn get_average_metrics(&self) -> (f64, f64, f64, f64) {
        let avg_cpu = if !self.cpu_usage.is_empty() {
            self.cpu_usage.iter().sum::<f64>() / self.cpu_usage.len() as f64
        } else {
            0.0
        };

        let avg_memory = if !self.memory_usage.is_empty() {
            self.memory_usage.iter().sum::<f64>() / self.memory_usage.len() as f64
        } else {
            0.0
        };

        let avg_temp = if !self.temperature.is_empty() {
            self.temperature.iter().sum::<f64>() / self.temperature.len() as f64
        } else {
            0.0
        };

        let avg_power = if !self.power_usage.is_empty() {
            self.power_usage.iter().sum::<f64>() / self.power_usage.len() as f64
        } else {
            0.0
        };

        (avg_cpu, avg_memory, avg_temp, avg_power)
    }
}

/// Comprehensive performance benchmark
pub struct PerformanceBenchmark {
    latency_measurer: LatencyMeasurer,
    throughput_benchmarker: ThroughputBenchmarker,
    profiler: EdgeProfiler,
}

impl PerformanceBenchmark {
    pub fn new(max_samples: usize) -> Self {
        Self {
            latency_measurer: LatencyMeasurer::new(max_samples),
            throughput_benchmarker: ThroughputBenchmarker::new(),
            profiler: EdgeProfiler::new(max_samples),
        }
    }

    pub fn start_benchmark(&mut self) {
        self.throughput_benchmarker.start();
    }

    pub fn record_operation_with_latency<F, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.latency_measurer.record_latency(duration);
        self.throughput_benchmarker.record_operation();
        result
    }

    pub fn record_error(&mut self) {
        self.throughput_benchmarker.record_error();
    }

    pub fn record_system_metrics(&mut self, cpu: f64, memory: f64, temp: f64, power: f64) {
        self.profiler.record_metrics(cpu, memory, temp, power);
    }

    pub fn get_comprehensive_metrics(&self) -> Option<BenchmarkMetrics> {
        self.latency_measurer.get_statistics().map(|mut metrics| {
            if let Some(throughput) = self.throughput_benchmarker.get_throughput() {
                metrics.throughput_ops_per_sec = throughput;
            }
            metrics.success_rate_percent = self.throughput_benchmarker.get_success_rate();
            metrics.error_rate_percent = 100.0 - metrics.success_rate_percent;

            let (avg_cpu, avg_memory, _, avg_power) = self.profiler.get_average_metrics();
            metrics.cpu_usage_percent = avg_cpu;
            metrics.memory_usage_mb = avg_memory;
            metrics.power_consumption_watts = avg_power;

            metrics
        })
    }
}

/// Load generator for edge computing workloads
#[derive(Debug)]
pub struct EdgeLoadGenerator {
    target_qps: f64,
    ramp_up_duration: Duration,
    steady_state_duration: Duration,
    cooldown_duration: Duration,
}

impl EdgeLoadGenerator {
    pub fn new(target_qps: f64, ramp_up: Duration, steady_state: Duration, cooldown: Duration) -> Self {
        Self {
            target_qps,
            ramp_up_duration: ramp_up,
            steady_state_duration: steady_state,
            cooldown_duration: cooldown,
        }
    }

    pub async fn generate_load<F, Fut>(&self, operation: F)
    where
        F: Fn() -> Fut + Send + Sync + Clone,
        Fut: std::future::Future<Output: ()> + Send,
    {
        println!("Starting edge workload generation at {} QPS", self.target_qps);
        
        // Ramp-up phase
        println!("Ramp-up phase for {:?}", self.ramp_up_duration);
        for qps in (0..(self.target_qps as u32)).step_by(100).chain(vec![self.target_qps as u32]) {
            let current_qps = qps as f64;
            let interval = Duration::from_secs_f64(1.0 / current_qps);
            
            let mut handles = Vec::new();
            for _ in 0..10 { // Parallel operations
                let operation = operation.clone();
                handles.push(tokio::spawn(async move {
                    operation().await;
                }));
            }
            
            for handle in handles {
                handle.await.unwrap();
            }
            
            tokio::time::sleep(interval).await;
        }

        // Steady-state phase
        println!("Steady-state phase for {:?}", self.steady_state_duration);
        let start_time = Instant::now();
        let interval = Duration::from_secs_f64(1.0 / self.target_qps);
        
        while start_time.elapsed() < self.steady_state_duration {
            let operation = operation.clone();
            tokio::spawn(async move {
                operation().await;
            });
            tokio::time::sleep(interval).await;
        }

        // Cooldown phase
        println!("Cooldown phase for {:?}", self.cooldown_duration);
        tokio::time::sleep(self.cooldown_duration).await;
        
        println!("Load generation completed");
    }
}

/// Memory usage tracker
#[derive(Debug)]
pub struct MemoryTracker {
    baseline_usage: Option<u64>,
    peak_usage: u64,
    current_usage: u64,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            baseline_usage: None,
            peak_usage: 0,
            current_usage: 0,
        }
    }

    pub fn update(&mut self, usage: u64) {
        if self.baseline_usage.is_none() {
            self.baseline_usage = Some(usage);
        }
        self.current_usage = usage;
        if usage > self.peak_usage {
            self.peak_usage = usage;
        }
    }

    pub fn get_memory_metrics(&self) -> MemoryMetrics {
        MemoryMetrics {
            baseline_mb: self.baseline_usage.unwrap_or(0) / 1024 / 1024,
            current_mb: self.current_usage / 1024 / 1024,
            peak_mb: self.peak_usage / 1024 / 1024,
            usage_increase_mb: (self.current_usage - self.baseline_usage.unwrap_or(0)) / 1024 / 1024,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryMetrics {
    pub baseline_mb: u64,
    pub current_mb: u64,
    pub peak_mb: u64,
    pub usage_increase_mb: u64,
}

/// CPU usage tracker
#[derive(Debug)]
pub struct CpuTracker {
    samples: VecDeque<f64>,
    max_samples: usize,
}

impl CpuTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record_usage(&mut self, usage: f64) {
        self.samples.push_back(usage);
        if self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    pub fn get_average_usage(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        }
    }
}

/// System resource monitor
pub struct SystemMonitor {
    memory_tracker: MemoryTracker,
    cpu_tracker: CpuTracker,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            memory_tracker: MemoryTracker::new(),
            cpu_tracker: CpuTracker::new(1000),
        }
    }

    pub async fn start_monitoring(&mut self, interval: Duration) {
        let mut interval_timer = tokio::time::interval(interval);
        
        loop {
            interval_timer.tick().await;
            
            // Simulate system metrics collection (in real implementation, would read from /proc or system APIs)
            let memory_usage = self.get_current_memory_usage().await;
            let cpu_usage = self.get_current_cpu_usage().await;
            
            self.memory_tracker.update(memory_usage);
            self.cpu_tracker.record_usage(cpu_usage);
        }
    }

    async fn get_current_memory_usage(&self) -> u64 {
        // In real implementation, would read from /proc/meminfo or system APIs
        // For demonstration, return simulated values
        2048000 // 2GB in bytes
    }

    async fn get_current_cpu_usage(&self) -> f64 {
        // In real implementation, would read from /proc/stat or system APIs
        // For demonstration, return simulated values
        rand::random::<f64>() * 100.0
    }

    pub fn get_system_metrics(&self) -> (MemoryMetrics, f64) {
        let memory_metrics = self.memory_tracker.get_memory_metrics();
        let avg_cpu = self.cpu_tracker.get_average_usage();
        (memory_metrics, avg_cpu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_latency_measurer() {
        let mut measurer = LatencyMeasurer::new(10);
        measurer.record_latency(Duration::from_millis(10));
        measurer.record_latency(Duration::from_millis(20));
        measurer.record_latency(Duration::from_millis(30));
        
        let metrics = measurer.get_statistics().unwrap();
        assert_eq!(metrics.min_latency_ms, 10.0);
        assert_eq!(metrics.max_latency_ms, 30.0);
        assert_eq!(metrics.avg_latency_ms, 20.0);
    }

    #[test]
    fn test_throughput_benchmarker() {
        let mut benchmarker = ThroughputBenchmarker::new();
        benchmarker.start();
        
        for _ in 0..100 {
            benchmarker.record_operation();
        }
        
        benchmarker.record_error();
        
        let throughput = benchmarker.get_throughput().unwrap();
        let success_rate = benchmarker.get_success_rate();
        
        assert!(throughput > 0.0);
        assert_eq!(success_rate, 99.0);
    }

    #[test]
    fn test_edge_profiler() {
        let mut profiler = EdgeProfiler::new(5);
        
        profiler.record_metrics(50.0, 1024.0, 65.0, 15.0);
        profiler.record_metrics(60.0, 2048.0, 70.0, 18.0);
        profiler.record_metrics(70.0, 1536.0, 75.0, 20.0);
        
        let (cpu, memory, temp, power) = profiler.get_average_metrics();
        
        assert!((cpu - 60.0).abs() < f64::EPSILON);
        assert!((memory - 1536.0).abs() < f64::EPSILON);
        assert!((temp - 70.0).abs() < f64::EPSILON);
        assert!((power - 17.666666666666668).abs() < 0.001);
    }
}