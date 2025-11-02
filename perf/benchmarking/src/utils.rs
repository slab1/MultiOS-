//! Benchmark Utilities
//! 
//! This module provides utility functions and helpers for benchmarking,
//! including statistical analysis, timing utilities, and data formatting.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Timing utilities
pub struct Timer;

impl Timer {
    /// Start a high-resolution timer
    pub fn start() -> Instant {
        Instant::now()
    }
    
    /// Calculate elapsed time
    pub fn elapsed(start: &Instant) -> Duration {
        start.elapsed()
    }
    
    /// Format duration as human-readable string
    pub fn format_duration(duration: &Duration) -> String {
        let nanos = duration.subsec_nanos();
        let secs = duration.as_secs();
        
        if secs > 0 {
            format!("{}.{:03}s", secs, nanos / 1_000_000)
        } else if nanos > 1_000_000 {
            format!("{}.{:03}ms", nanos / 1_000_000, (nanos % 1_000_000) / 1_000)
        } else {
            format!("{}ns", nanos)
        }
    }
    
    /// Format duration for performance metrics
    pub fn format_duration_for_metrics(duration: &Duration) -> String {
        let total_nanos = duration.as_nanos();
        
        if total_nanos < 1_000 {
            format!("{}ns", total_nanos)
        } else if total_nanos < 1_000_000 {
            format!("{:.2}µs", total_nanos as f64 / 1_000.0)
        } else if total_nanos < 1_000_000_000 {
            format!("{:.2}ms", total_nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.2}s", total_nanos as f64 / 1_000_000_000.0)
        }
    }
}

/// Statistical analysis utilities
pub struct Stats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub percentile_95: f64,
    pub percentile_99: f64,
}

impl Stats {
    /// Calculate statistical summary of a dataset
    pub fn analyze(values: &[f64]) -> StatisticalSummary {
        if values.is_empty() {
            return StatisticalSummary {
                count: 0,
                mean: 0.0,
                median: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
                percentile_95: 0.0,
                percentile_99: 0.0,
            };
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;
        
        let median = if count % 2 == 0 {
            let mid = count / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count / 2]
        };
        
        let min = sorted_values[0];
        let max = sorted_values[count - 1];
        
        // Calculate standard deviation
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();
        
        // Calculate percentiles
        let p95_index = ((count as f64 * 0.95).floor() as usize).min(count - 1);
        let p99_index = ((count as f64 * 0.99).floor() as usize).min(count - 1);
        
        StatisticalSummary {
            count,
            mean,
            median,
            min,
            max,
            std_dev,
            percentile_95: sorted_values[p95_index],
            percentile_99: sorted_values[p99_index],
        }
    }
    
    /// Calculate coefficient of variation (CV)
    pub fn coefficient_of_variation(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        let summary = Self::analyze(values);
        if summary.mean == 0.0 {
            0.0
        } else {
            (summary.std_dev / summary.mean.abs()) * 100.0
        }
    }
    
    /// Detect outliers using IQR method
    pub fn detect_outliers(values: &[f64]) -> (Vec<usize>, f64, f64) {
        if values.len() < 4 {
            return (Vec::new(), 0.0, 0.0);
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let count = values.len();
        let q1_index = count / 4;
        let q3_index = (3 * count) / 4;
        
        let q1 = sorted_values[q1_index];
        let q3 = sorted_values[q3_index];
        let iqr = q3 - q1;
        
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;
        
        let mut outlier_indices = Vec::new();
        for (i, &value) in values.iter().enumerate() {
            if value < lower_bound || value > upper_bound {
                outlier_indices.push(i);
            }
        }
        
        (outlier_indices, lower_bound, upper_bound)
    }
}

/// Memory size formatting utilities
pub struct Size;

impl Size {
    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
    
    /// Parse human-readable size string
    pub fn parse_size(size_str: &str) -> Option<u64> {
        let parts: Vec<&str> = size_str.trim().split_whitespace().collect();
        if parts.len() != 2 {
            return None;
        }
        
        let value: f64 = parts[0].parse().ok()?;
        let unit = parts[1].to_uppercase();
        
        let multiplier = match unit.as_str() {
            "B" => 1.0,
            "KB" => 1024.0,
            "MB" => 1024.0 * 1024.0,
            "GB" => 1024.0 * 1024.0 * 1024.0,
            "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
            _ => return None,
        };
        
        Some((value * multiplier) as u64)
    }
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    pub measurements: Vec<Duration>,
    pub iteration_count: u64,
    pub start_time: Instant,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
            iteration_count: 0,
            start_time: Instant::now(),
        }
    }
    
    /// Record a measurement
    pub fn record(&mut self, duration: Duration) {
        self.measurements.push(duration);
        self.iteration_count += 1;
    }
    
    /// Calculate throughput metrics
    pub fn calculate_throughput(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        if self.measurements.is_empty() {
            return metrics;
        }
        
        let total_time = self.start_time.elapsed();
        let total_ops = self.iteration_count;
        
        metrics.insert("total_operations".to_string(), total_ops as f64);
        metrics.insert("total_time_sec".to_string(), total_time.as_secs_f64());
        metrics.insert("operations_per_second".to_string(), 
            total_ops as f64 / total_time.as_secs_f64());
        
        if total_ops > 0 {
            let total_ns: u128 = self.measurements.iter()
                .map(|d| d.as_nanos())
                .sum();
            let avg_ns = total_ns / self.measurements.len() as u128;
            metrics.insert("average_operation_ns".to_string(), avg_ns as f64);
        }
        
        // Calculate percentiles
        let summaries = Stats::analyze(&self.measurements_to_seconds());
        metrics.insert("median_operation_sec".to_string(), summaries.median);
        metrics.insert("p95_operation_sec".to_string(), summaries.percentile_95);
        metrics.insert("p99_operation_sec".to_string(), summaries.percentile_99);
        
        metrics
    }
    
    /// Convert measurements to seconds
    fn measurements_to_seconds(&self) -> Vec<f64> {
        self.measurements.iter()
            .map(|d| d.as_secs_f64())
            .collect()
    }
    
    /// Generate performance report
    pub fn generate_report(&self) -> String {
        if self.measurements.is_empty() {
            return "No measurements recorded".to_string();
        }
        
        let metrics = self.calculate_throughput();
        let summaries = Stats::analyze(&self.measurements_to_seconds());
        let cv = Stats::coefficient_of_variation(&self.measurements_to_seconds());
        
        let mut report = String::new();
        report.push_str("=== Performance Metrics Report ===\n\n");
        
        report.push_str(&format!("Total Operations: {}\n", self.iteration_count));
        report.push_str(&format!("Total Time: {}\n", Timer::format_duration(&self.start_time.elapsed())));
        report.push_str(&format!("Operations/Second: {:.2}\n\n", 
            metrics.get("operations_per_second").unwrap_or(&0.0)));
        
        report.push_str("=== Timing Statistics ===\n");
        report.push_str(&format!("Mean: {}\n", Timer::format_duration_for_metrics(
            &Duration::from_secs_f64(summaries.mean))));
        report.push_str(&format!("Median: {}\n", Timer::format_duration_for_metrics(
            &Duration::from_secs_f64(summaries.median))));
        report.push_str(&format!("Min: {}\n", Timer::format_duration_for_metrics(
            &Duration::from_secs_f64(summaries.min))));
        report.push_str(&format!("Max: {}\n", Timer::format_duration_for_metrics(
            &Duration::from_secs_f64(summaries.max))));
        report.push_str(&format!("95th Percentile: {}\n", Timer::format_duration_for_metrics(
            &Duration::from_secs_f64(summaries.percentile_95))));
        report.push_str(&format!("99th Percentile: {}\n", Timer::format_duration_for_metrics(
            &Duration::from_secs_f64(summaries.percentile_99))));
        report.push_str(&format!("Standard Deviation: {}\n", 
            Timer::format_duration_for_metrics(&Duration::from_secs_f64(summaries.std_dev))));
        report.push_str(&format!("Coefficient of Variation: {:.2}%\n\n", cv));
        
        let (outliers, lower_bound, upper_bound) = Stats::detect_outliers(&self.measurements_to_seconds());
        report.push_str(&format!("Outliers Detected: {} ({:.2}%)\n", outliers.len(), 
            (outliers.len() as f64 / self.measurements.len() as f64) * 100.0));
        report.push_str(&format!("Outlier Range: [{}, {}]\n\n", lower_bound, upper_bound));
        
        report
    }
}

/// Benchmark configuration validation
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate benchmark configuration
    pub fn validate_config(config: &crate::BenchmarkConfig) -> Result<(), String> {
        if config.iterations == 0 {
            return Err("Iterations must be greater than 0".to_string());
        }
        
        if config.warmup_iterations >= config.iterations {
            return Err("Warmup iterations must be less than total iterations".to_string());
        }
        
        if let Some(timeout) = config.timeout {
            if timeout.as_secs() == 0 {
                return Err("Timeout must be greater than 0 seconds".to_string());
            }
        }
        
        if config.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }
        
        Ok(())
    }
    
    /// Validate system requirements
    pub fn validate_system_requirements() -> Result<HashMap<String, String>, String> {
        let mut requirements = HashMap::new();
        
        // Check available memory
        let mem_info = match sys_info::mem_info() {
            Ok(info) => info,
            Err(_) => return Err("Failed to get memory information".to_string()),
        };
        
        requirements.insert("available_memory_mb".to_string(), 
            (mem_info.avail / 1024 / 1024).to_string());
        requirements.insert("total_memory_mb".to_string(), 
            (mem_info.total / 1024 / 1024).to_string());
        
        // Check CPU info
        if let Ok(cpu_info) = sys_info::cpu_num() {
            requirements.insert("cpu_cores".to_string(), cpu_info.to_string());
        }
        
        // Check disk space
        if let Ok(path) = std::env::current_dir() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                // Note: This checks the current directory, not necessarily where temp files will be created
                requirements.insert("current_dir_accessible".to_string(), "true".to_string());
            }
        }
        
        Ok(requirements)
    }
}

/// Progress tracking utilities
pub struct ProgressTracker {
    total: u64,
    current: AtomicU64,
    start_time: Instant,
    description: String,
}

use std::sync::atomic::{AtomicU64, Ordering};

impl ProgressTracker {
    pub fn new(total: u64, description: String) -> Self {
        Self {
            total,
            current: AtomicU64::new(0),
            start_time: Instant::now(),
            description,
        }
    }
    
    pub fn increment(&self, amount: u64) {
        self.current.fetch_add(amount, Ordering::SeqCst);
    }
    
    pub fn set_current(&self, value: u64) {
        self.current.store(value, Ordering::SeqCst);
    }
    
    pub fn get_progress(&self) -> (u64, f64, Duration) {
        let current = self.current.load(Ordering::SeqCst);
        let elapsed = self.start_time.elapsed();
        let progress = if self.total > 0 {
            (current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };
        (current, progress, elapsed)
    }
    
    pub fn get_eta(&self) -> Option<Duration> {
        let (current, progress, elapsed) = self.get_progress();
        if progress > 0.0 && current < self.total {
            let total_estimated = elapsed.as_secs_f64() / (progress / 100.0);
            let remaining = total_estimated - elapsed.as_secs_f64();
            if remaining > 0.0 {
                return Some(Duration::from_secs_f64(remaining));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_duration() {
        let duration = Duration::from_millis(1500);
        assert_eq!(Timer::format_duration(&duration), "1.500s");
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(Size::format_bytes(1024), "1.00 KB".to_string());
        assert_eq!(Size::format_bytes(1048576), "1.00 MB".to_string());
    }
    
    #[test]
    fn test_stats_analysis() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = Stats::analyze(&values);
        assert_eq!(summary.count, 5);
        assert_eq!(summary.mean, 3.0);
        assert_eq!(summary.median, 3.0);
    }
}