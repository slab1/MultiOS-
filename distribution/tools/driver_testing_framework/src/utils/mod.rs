//! Utility Functions Module
//!
//! This module provides common utility functions and helpers used throughout
//! the driver testing framework, including time utilities, math functions,
//! string utilities, and helper macros.

use crate::core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Time utilities for testing framework
pub mod time_utils {
    use super::*;
    
    /// Convert Duration to milliseconds
    pub fn duration_to_ms(duration: Duration) -> u64 {
        duration.as_millis() as u64
    }
    
    /// Convert Duration to microseconds
    pub fn duration_to_us(duration: Duration) -> u64 {
        duration.as_micros() as u64
    }
    
    /// Convert Duration to nanoseconds
    pub fn duration_to_ns(duration: Duration) -> u64 {
        duration.as_nanos() as u64
    }
    
    /// Convert milliseconds to Duration
    pub fn ms_to_duration(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
    
    /// Convert microseconds to Duration
    pub fn us_to_duration(us: u64) -> Duration {
        Duration::from_micros(us)
    }
    
    /// Get current timestamp as milliseconds since epoch
    pub fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// Calculate elapsed time from start timestamp
    pub fn elapsed_since(start: SystemTime) -> Duration {
        SystemTime::now()
            .duration_since(start)
            .unwrap_or_default()
    }
    
    /// Format duration for display
    pub fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let ms_part = duration.subsec_millis();
        
        if total_secs == 0 {
            format!("{}ms", ms_part)
        } else if total_secs < 60 {
            format!("{}.{}s", total_secs, ms_part / 10)
        } else {
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            format!("{}m {}s", minutes, seconds)
        }
    }
    
    /// Check if duration exceeds threshold
    pub fn exceeds_threshold(duration: Duration, threshold: Duration) -> bool {
        duration > threshold
    }
    
    /// Create a timeout duration
    pub fn create_timeout(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }
    
    /// Create a short timeout (milliseconds)
    pub fn create_short_timeout(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
    
    /// Create a long timeout (minutes)
    pub fn create_long_timeout(minutes: u64) -> Duration {
        Duration::from_secs(minutes * 60)
    }
}

/// Math utilities for testing framework
pub mod math_utils {
    use super::*;
    
    /// Calculate percentage
    pub fn calculate_percentage(part: u64, total: u64) -> f32 {
        if total == 0 {
            0.0
        } else {
            (part as f32 / total as f32) * 100.0
        }
    }
    
    /// Calculate average of a slice of u64 values
    pub fn calculate_average_u64(values: &[u64]) -> f32 {
        if values.is_empty() {
            0.0
        } else {
            let sum: u64 = values.iter().sum();
            sum as f32 / values.len() as f32
        }
    }
    
    /// Calculate average of a slice of f32 values
    pub fn calculate_average_f32(values: &[f32]) -> f32 {
        if values.is_empty() {
            0.0
        } else {
            let sum: f32 = values.iter().sum();
            sum / values.len() as f32
        }
    }
    
    /// Calculate median of a slice of u64 values
    pub fn calculate_median_u64(values: &mut [u64]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        
        values.sort();
        let len = values.len();
        
        if len % 2 == 0 {
            let mid_right = len / 2;
            let mid_left = mid_right - 1;
            (values[mid_left] + values[mid_right]) as f32 / 2.0
        } else {
            values[len / 2] as f32
        }
    }
    
    /// Calculate standard deviation
    pub fn calculate_std_dev(values: &[f32]) -> f32 {
        if values.len() <= 1 {
            return 0.0;
        }
        
        let mean = calculate_average_f32(values);
        let variance: f32 = values.iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f32>() / (values.len() as f32 - 1.0);
        
        variance.sqrt()
    }
    
    /// Calculate percentile (e.g., p95 for 95th percentile)
    pub fn calculate_percentile(values: &[f32], percentile: f32) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let index = ((percentile / 100.0) * (sorted_values.len() as f32 - 1.0)).round() as usize;
        
        sorted_values.get(index).copied().unwrap_or(0.0)
    }
    
    /// Clamp value between min and max
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
    
    /// Linear interpolation between two values
    pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
        start + (end - start) * t
    }
    
    /// Check if a value is within range [min, max]
    pub fn is_in_range<T: PartialOrd>(value: T, min: T, max: T) -> bool {
        value >= min && value <= max
    }
}

/// String utilities for testing framework
pub mod string_utils {
    use super::*;
    
    /// Convert test status to string
    pub fn status_to_string(status: TestStatus) -> &'static str {
        match status {
            TestStatus::Passed => "PASSED",
            TestStatus::Failed => "FAILED",
            TestStatus::Skipped => "SKIPPED",
            TestStatus::Timeout => "TIMEOUT",
            TestStatus::Error => "ERROR",
        }
    }
    
    /// Convert test category to string
    pub fn category_to_string(category: TestCategory) -> &'static str {
        match category {
            TestCategory::Unit => "Unit",
            TestCategory::Integration => "Integration",
            TestCategory::Performance => "Performance",
            TestCategory::Stress => "Stress",
            TestCategory::Validation => "Validation",
            TestCategory::Security => "Security",
            TestCategory::Compatibility => "Compatibility",
            TestCategory::Regression => "Regression",
            TestCategory::Debug => "Debug",
            TestCategory::Troubleshooting => "Troubleshooting",
        }
    }
    
    /// Format test result for display
    pub fn format_test_result(result: &TestResult) -> String {
        format!(
            "[{}] {} - {} ({})",
            status_to_string(result.status),
            result.name,
            result.message,
            time_utils::format_duration(result.duration)
        )
    }
    
    /// Truncate string to specified length
    pub fn truncate_string(s: &str, max_length: usize) -> String {
        if s.len() <= max_length {
            s.to_string()
        } else {
            format!("{}...", &s[..max_length.saturating_sub(3)])
        }
    }
    
    /// Pad string to specified length
    pub fn pad_string(s: &str, width: usize, pad_char: char) -> String {
        if s.len() >= width {
            s.to_string()
        } else {
            format!("{}{}", s, pad_char.to_string().repeat(width - s.len()))
        }
    }
    
    /// Convert string to title case
    pub fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Remove whitespace from string
    pub fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }
    
    /// Split string into chunks of specified size
    pub fn split_into_chunks(s: &str, chunk_size: usize) -> Vec<String> {
        s.chars()
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect()
    }
}

/// Data structure utilities
pub mod data_utils {
    use super::*;
    
    /// Create a new HashMap with initial capacity
    pub fn create_hashmap_with_capacity<K, V>(capacity: usize) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
        V: Default,
    {
        HashMap::with_capacity(capacity)
    }
    
    /// Merge two HashMaps, with second map taking precedence
    pub fn merge_hashmaps<K, V>(map1: HashMap<K, V>, mut map2: HashMap<K, V>) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        for (key, value) in map1 {
            map2.entry(key).or_insert(value);
        }
        map2
    }
    
    /// Check if HashMap contains all keys
    pub fn hashmap_contains_all_keys<K, V>(map: &HashMap<K, V>, keys: &[K]) -> bool
    where
        K: std::hash::Hash + Eq,
    {
        keys.iter().all(|key| map.contains_key(key))
    }
    
    /// Get value from HashMap with default
    pub fn hashmap_get_with_default<K, V>(map: &HashMap<K, V>, key: &K, default: V) -> V
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        map.get(key).cloned().unwrap_or(default)
    }
    
    /// Convert slice to HashSet
    pub fn slice_to_hashset<T>(slice: &[T]) -> std::collections::HashSet<&T>
    where
        T: std::hash::Hash + Eq,
    {
        slice.iter().collect()
    }
    
    /// Find common elements between two slices
    pub fn find_common_elements<T>(slice1: &[T], slice2: &[T]) -> Vec<&T>
    where
        T: std::hash::Hash + Eq,
    {
        let set2: std::collections::HashSet<_> = slice2.iter().collect();
        slice1.iter().filter(|item| set2.contains(item)).collect()
    }
    
    /// Remove duplicates from slice
    pub fn remove_duplicates<T>(slice: &[T]) -> Vec<&T>
    where
        T: std::hash::Hash + Eq,
    {
        let mut seen = std::collections::HashSet::new();
        slice.iter().filter(|item| seen.insert(*item)).collect()
    }
}

/// Performance measurement utilities
pub mod perf_utils {
    use super::*;
    
    /// High-resolution timer for performance measurements
    #[derive(Debug)]
    pub struct PerfTimer {
        start_time: std::time::Instant,
        measurements: Vec<Duration>,
    }
    
    impl PerfTimer {
        /// Create a new performance timer
        pub fn new() -> Self {
            Self {
                start_time: std::time::Instant::now(),
                measurements: Vec::new(),
            }
        }
        
        /// Start timing
        pub fn start(&mut self) {
            self.start_time = std::time::Instant::now();
        }
        
        /// Record a measurement
        pub fn record(&mut self) {
            let elapsed = self.start_time.elapsed();
            self.measurements.push(elapsed);
        }
        
        /// Get all measurements
        pub fn get_measurements(&self) -> &[Duration] {
            &self.measurements
        }
        
        /// Get average measurement
        pub fn get_average(&self) -> Option<Duration> {
            if self.measurements.is_empty() {
                None
            } else {
                let total: Duration = self.measurements.iter().sum();
                Some(total / self.measurements.len() as u32)
            }
        }
        
        /// Get minimum measurement
        pub fn get_min(&self) -> Option<Duration> {
            self.measurements.iter().min().copied()
        }
        
        /// Get maximum measurement
        pub fn get_max(&self) -> Option<Duration> {
            self.measurements.iter().max().copied()
        }
        
        /// Get percentile measurement (e.g., 95.0 for 95th percentile)
        pub fn get_percentile(&self, percentile: f32) -> Option<Duration> {
            if self.measurements.is_empty() {
                return None;
            }
            
            let mut sorted_measurements = self.measurements.clone();
            sorted_measurements.sort();
            
            let index = ((percentile / 100.0) * (sorted_measurements.len() as f32 - 1.0)).round() as usize;
            sorted_measurements.get(index).copied()
        }
        
        /// Clear all measurements
        pub fn clear(&mut self) {
            self.measurements.clear();
        }
    }
    
    impl Default for PerfTimer {
        fn default() -> Self {
            Self::new()
        }
    }
    
    /// Measure execution time of a closure
    pub fn measure_execution_time<F, R>(func: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = func();
        let duration = start.elapsed();
        (result, duration)
    }
    
    /// Measure execution time of an async closure
    pub async fn measure_async_execution_time<F, R>(func: F) -> (R, Duration)
    where
        F: std::future::Future<Output = R>,
    {
        let start = std::time::Instant::now();
        let result = func().await;
        let duration = start.elapsed();
        (result, duration)
    }
}

/// Error handling utilities
pub mod error_utils {
    use super::*;
    
    /// Convert different error types to DriverTestError
    pub fn convert_error<E>(error: E) -> DriverTestError
    where
        E: std::fmt::Display,
    {
        DriverTestError::TestExecutionError(error.to_string())
    }
    
    /// Handle optional result with custom error
    pub fn handle_optional_result<T>(result: Option<T>, error_msg: &str) -> Result<T, DriverTestError> {
        result.ok_or_else(|| DriverTestError::TestExecutionError(error_msg.to_string()))
    }
    
    /// Handle Result with custom error mapping
    pub fn handle_result_with_mapping<T, E>(result: Result<T, E>, error_mapper: fn(E) -> DriverTestError) -> Result<T, DriverTestError> {
        result.map_err(error_mapper)
    }
    
    /// Retry operation with exponential backoff
    pub async fn retry_with_backoff<F, Fut, T>(
        mut operation: F,
        max_retries: u32,
        base_delay: Duration,
    ) -> Result<T, DriverTestError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, DriverTestError>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    if attempt < max_retries {
                        let delay = Duration::from_millis(
                            base_delay.as_millis() as u64 * (2_u64.pow(attempt))
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            DriverTestError::TestExecutionError("All retry attempts failed".to_string())
        }))
    }
    
    /// Create a timeout error
    pub fn create_timeout_error(timeout: Duration) -> DriverTestError {
        DriverTestError::TimeoutError(timeout)
    }
    
    /// Check if error is retryable
    pub fn is_retryable_error(error: &DriverTestError) -> bool {
        matches!(
            error,
            DriverTestError::TimeoutError(_) |
            DriverTestError::ResourceError(_) |
            DriverTestError::HardwareSimulationError(_)
        )
    }
}

/// Configuration utilities
pub mod config_utils {
    use super::*;
    use serde_yaml;
    use std::fs;
    
    /// Load configuration from YAML file
    pub fn load_config_from_yaml<T>(file_path: &str) -> Result<T, DriverTestError>
    where
        T: serde::de::DeserializeOwned,
    {
        let content = fs::read_to_string(file_path)
            .map_err(|e| DriverTestError::ConfigurationError(format!(
                "Failed to read config file '{}': {}", file_path, e
            )))?;
        
        let config: T = serde_yaml::from_str(&content)
            .map_err(|e| DriverTestError::ConfigurationError(format!(
                "Failed to parse config file '{}': {}", file_path, e
            )))?;
        
        Ok(config)
    }
    
    /// Save configuration to YAML file
    pub fn save_config_to_yaml<T>(config: &T, file_path: &str) -> Result<(), DriverTestError>
    where
        T: serde::Serialize,
    {
        let content = serde_yaml::to_string(config)
            .map_err(|e| DriverTestError::ConfigurationError(format!(
                "Failed to serialize config: {}", e
            )))?;
        
        fs::write(file_path, content)
            .map_err(|e| DriverTestError::ConfigurationError(format!(
                "Failed to write config file '{}': {}", file_path, e
            )))?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate_config<T>(config: &T) -> Result<(), DriverTestError>
    where
        T: ConfigValidator,
    {
        config.validate()
    }
    
    /// Merge two configurations
    pub fn merge_configs<T>(base: &T, override_config: &T) -> Result<T, DriverTestError>
    where
        T: ConfigMerger,
    {
        base.merge_with(override_config)
    }
}

/// Configuration traits
pub trait ConfigValidator {
    fn validate(&self) -> Result<(), DriverTestError>;
}

pub trait ConfigMerger {
    fn merge_with(&self, other: &Self) -> Result<Self, DriverTestError>
    where
        Self: Clone;
}

/// Helper macros for testing framework
#[macro_export]
macro_rules! create_test_result {
    ($name:expr, $status:expr, $message:expr) => {
        TestResult {
            name: $name.to_string(),
            status: $status,
            duration: std::time::Duration::from_millis(0),
            message: $message.to_string(),
            category: TestCategory::Unit,
            metadata: None,
            metrics: None,
        }
    };
}

#[macro_export]
macro_rules! assert_test_result_success {
    ($result:expr) => {
        assert!($result.is_success(), "Test result should be successful but got: {:?}", $result.status);
    };
}

#[macro_export]
macro_rules! assert_test_result_failure {
    ($result:expr) => {
        assert!($result.is_failure(), "Test result should be failed but got: {:?}", $result.status);
    };
}

#[macro_export]
macro_rules! create_performance_timer {
    () => {
        PerfTimer::new()
    };
}

#[macro_export]
macro_rules! measure_performance {
    ($timer:ident, $operation:block) => {
        $timer.start();
        let result = $operation;
        $timer.record();
        result
    };
}

#[macro_export]
macro_rules! log_test_start {
    ($name:expr) => {
        log::info!("Starting test: {}", $name);
    };
}

#[macro_export]
macro_rules! log_test_complete {
    ($name:expr, $status:expr, $duration:expr) => {
        log::info!("Test '{}' completed: {} ({:?})", $name, $status, $duration);
    };
}

/// Benchmark utilities
#[cfg(feature = "performance")]
pub mod benchmark_utils {
    use super::*;
    
    /// Simple benchmark runner
    pub struct BenchmarkRunner {
        iterations: u32,
        warmup_iterations: u32,
    }
    
    impl BenchmarkRunner {
        pub fn new(iterations: u32) -> Self {
            Self {
                iterations,
                warmup_iterations: 3,
            }
        }
        
        pub fn run_benchmark<F, T>(&self, name: &str, mut operation: F) -> Result<BenchmarkResult, DriverTestError>
        where
            F: FnMut() -> T,
        {
            // Warmup
            for _ in 0..self.warmup_iterations {
                let _ = operation();
            }
            
            // Actual benchmark
            let start_time = std::time::Instant::now();
            for _ in 0..self.iterations {
                let _ = operation();
            }
            let total_duration = start_time.elapsed();
            
            let avg_duration = total_duration / self.iterations;
            
            Ok(BenchmarkResult {
                name: name.to_string(),
                total_duration,
                average_duration: avg_duration,
                iterations: self.iterations,
            })
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct BenchmarkResult {
        pub name: String,
        pub total_duration: Duration,
        pub average_duration: Duration,
        pub iterations: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_utils() {
        let duration = Duration::from_millis(1500);
        assert_eq!(time_utils::duration_to_ms(duration), 1500);
        assert_eq!(time_utils::duration_to_us(duration), 1500000);
        assert_eq!(time_utils::format_duration(duration), "1.500s");
    }
    
    #[test]
    fn test_math_utils() {
        assert_eq!(math_utils::calculate_percentage(25, 100), 25.0);
        assert_eq!(math_utils::clamp(5, 0, 10), 5);
        assert_eq!(math_utils::clamp(15, 0, 10), 10);
        assert_eq!(math_utils::clamp(-5, 0, 10), 0);
    }
    
    #[test]
    fn test_string_utils() {
        assert_eq!(string_utils::status_to_string(TestStatus::Passed), "PASSED");
        assert_eq!(string_utils::category_to_string(TestCategory::Unit), "Unit");
        
        let test_result = TestResult {
            name: "test_example".to_string(),
            status: TestStatus::Passed,
            duration: Duration::from_millis(100),
            message: "Test message".to_string(),
            category: TestCategory::Unit,
            metadata: None,
            metrics: None,
        };
        
        let formatted = string_utils::format_test_result(&test_result);
        assert!(formatted.contains("[PASSED]"));
        assert!(formatted.contains("test_example"));
    }
    
    #[test]
    fn test_perf_timer() {
        let mut timer = PerfTimer::new();
        
        for i in 0..5 {
            timer.start();
            std::thread::sleep(std::time::Duration::from_millis(i * 10));
            timer.record();
        }
        
        assert_eq!(timer.get_measurements().len(), 5);
        assert!(timer.get_average().is_some());
        assert!(timer.get_min() <= timer.get_max().unwrap());
    }
    
    #[test]
    fn test_macro_functions() {
        let result = create_test_result!("test_name", TestStatus::Passed, "test passed");
        assert_eq!(result.name, "test_name");
        assert_eq!(result.status, TestStatus::Passed);
        
        assert_test_result_success!(result);
        
        let failure_result = create_test_result!("failure_test", TestStatus::Failed, "test failed");
        assert_test_result_failure!(failure_result);
    }
}
