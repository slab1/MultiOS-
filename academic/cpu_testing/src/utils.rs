//! Utility Functions Module
//!
//! This module provides common utility functions used throughout
//! the CPU architecture testing framework.

use crate::architecture::Architecture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Convert architecture to string representation
pub fn architecture_to_string(arch: &Architecture) -> String {
    match arch {
        Architecture::X86_64 => "x86_64".to_string(),
        Architecture::ARM64 => "arm64".to_string(),
        Architecture::RISC_V64 => "riscv64".to_string(),
        Architecture::SPARC64 => "sparc64".to_string(),
        Architecture::PowerPC64 => "powerpc64".to_string(),
    }
}

/// Convert string to architecture
pub fn string_to_architecture(s: &str) -> Result<Architecture, String> {
    match s.to_lowercase().as_str() {
        "x86_64" | "x86-64" | "amd64" => Ok(Architecture::X86_64),
        "arm64" | "aarch64" => Ok(Architecture::ARM64),
        "riscv64" | "risc-v64" => Ok(Architecture::RISC_V64),
        "sparc64" | "sparc-v9" => Ok(Architecture::SPARC64),
        "powerpc64" | "ppc64" | "power9" => Ok(Architecture::PowerPC64),
        _ => Err(format!("Unknown architecture: {}", s)),
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}.{}s", seconds, duration.subsec_millis())
    }
}

/// Format large numbers with appropriate units
pub fn format_number_with_units(value: f64, unit: &str) -> String {
    let abs_value = value.abs();
    
    if abs_value >= 1_000_000_000.0 {
        format!("{:.2}G{}", value / 1_000_000_000.0, unit)
    } else if abs_value >= 1_000_000.0 {
        format!("{:.2}M{}", value / 1_000_000.0, unit)
    } else if abs_value >= 1_000.0 {
        format!("{:.2}k{}", value / 1_000.0, unit)
    } else {
        format!("{:.2}{}", value, unit)
    }
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", value, UNITS[unit_index])
}

/// Format frequency in human-readable format
pub fn format_frequency(hz: f64) -> String {
    if hz >= 1_000_000_000.0 {
        format!("{:.2} GHz", hz / 1_000_000_000.0)
    } else if hz >= 1_000_000.0 {
        format!("{:.2} MHz", hz / 1_000_000.0)
    } else if hz >= 1_000.0 {
        format!("{:.2} kHz", hz / 1_000.0)
    } else {
        format!("{:.2} Hz", hz)
    }
}

/// Format power consumption in human-readable format
pub fn format_power(watts: f64) -> String {
    if watts >= 1.0 {
        format!("{:.1} W", watts)
    } else if watts >= 0.001 {
        format!("{:.1} mW", watts * 1000.0)
    } else {
        format!("{:.1} μW", watts * 1_000_000.0)
    }
}

/// Calculate percentage and return formatted string
pub fn format_percentage(value: f64, total: f64) -> String {
    if total == 0.0 {
        "0.0%".to_string()
    } else {
        let percentage = (value / total) * 100.0;
        format!("{:.1}%", percentage)
    }
}

/// Generate timestamp string
pub fn generate_timestamp() -> String {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

/// Create a simple ID from timestamp and random data
pub fn generate_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let timestamp = generate_timestamp();
    let random_val = rand::random::<u64>();
    
    let mut hasher = DefaultHasher::new();
    timestamp.hash(&mut hasher);
    random_val.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Validate architecture string
pub fn is_valid_architecture(arch_str: &str) -> bool {
    string_to_architecture(arch_str).is_ok()
}

/// Get all supported architecture names
pub fn get_supported_architectures() -> Vec<String> {
    vec![
        "x86_64".to_string(),
        "arm64".to_string(),
        "riscv64".to_string(),
        "sparc64".to_string(),
        "powerpc64".to_string(),
    ]
}

/// Calculate statistical measures
pub fn calculate_statistics(values: &[f64]) -> StatisticalSummary {
    if values.is_empty() {
        return StatisticalSummary::empty();
    }

    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    let mean = sum / count;

    let variance: f64 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / count;

    let std_dev = variance.sqrt();

    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let median = if count % 2.0 == 0.0 {
        let mid = count as usize / 2;
        (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
    } else {
        let mid = count as usize / 2;
        sorted_values[mid]
    };

    let min_val = sorted_values.first().copied().unwrap_or(0.0);
    let max_val = sorted_values.last().copied().unwrap_or(0.0);

    StatisticalSummary {
        count: values.len(),
        mean,
        median,
        min: min_val,
        max: max_val,
        std_dev,
        variance,
        sum,
    }
}

/// Statistical summary structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub variance: f64,
    pub sum: f64,
}

impl StatisticalSummary {
    /// Create empty statistical summary
    pub fn empty() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            median: 0.0,
            min: 0.0,
            max: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            sum: 0.0,
        }
    }

    /// Convert to human-readable string
    pub fn to_string(&self) -> String {
        if self.count == 0 {
            "No data".to_string()
        } else {
            format!(
                "count={}, mean={:.2}, median={:.2}, min={:.2}, max={:.2}, std_dev={:.2}",
                self.count, self.mean, self.median, self.min, self.max, self.std_dev
            )
        }
    }
}

/// Progress tracking utility
pub struct ProgressTracker {
    total: u64,
    current: u64,
    start_time: SystemTime,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: 0,
            start_time: SystemTime::now(),
        }
    }

    /// Update progress
    pub fn update(&mut self, increment: u64) {
        self.current = (self.current + increment).min(self.total);
    }

    /// Set current progress
    pub fn set_current(&mut self, current: u64) {
        self.current = current.min(self.total);
    }

    /// Get current progress percentage
    pub fn get_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    /// Get estimated time remaining
    pub fn get_eta(&self) -> Option<Duration> {
        if self.current == 0 || self.current >= self.total {
            None
        } else {
            let elapsed = self.start_time.elapsed().unwrap_or_default();
            let rate = self.current as f64 / elapsed.as_secs_f64();
            let remaining_work = (self.total - self.current) as f64;
            let estimated_seconds = remaining_work / rate;
            Some(Duration::from_secs_f64(estimated_seconds))
        }
    }

    /// Get formatted progress string
    pub fn get_progress_string(&self) -> String {
        let percentage = self.get_percentage();
        let eta = self.get_eta();
        
        let eta_str = if let Some(eta_duration) = eta {
            format!(" ETA: {}", format_duration(eta_duration))
        } else {
            "".to_string()
        };
        
        format!("[{:.1}%] ({}/{}){}", percentage, self.current, self.total, eta_str)
    }
}

/// Simple color formatting for terminal output
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset,
}

impl Color {
    /// Get ANSI color code
    pub fn code(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Format text with color (only if terminal supports it)
pub fn colorize(text: &str, color: Color) -> String {
    if atty::is(atty::Stream::Stdout) {
        format!("{}{}{}", color.code(), text, Color::Reset.code())
    } else {
        text.to_string()
    }
}

/// Check if output is a terminal
pub fn is_terminal() -> bool {
    atty::is(atty::Stream::Stdout)
}

/// Format success/error messages with colors
pub fn format_success(message: &str) -> String {
    colorize(&format!("✓ {}", message), Color::Green)
}

pub fn format_error(message: &str) -> String {
    colorize(&format!("✗ {}", message), Color::Red)
}

pub fn format_warning(message: &str) -> String {
    colorize(&format!("⚠ {}", message), Color::Yellow)
}

pub fn format_info(message: &str) -> String {
    colorize(&format!("ℹ {}", message), Color::Blue)
}

/// Parse command line arguments for architectures
pub fn parse_architecture_list(arg: &str) -> Result<Vec<Architecture>, String> {
    let parts: Vec<&str> = arg.split(',').map(|s| s.trim()).collect();
    let mut architectures = Vec::new();

    for part in parts {
        match string_to_architecture(part) {
            Ok(arch) => architectures.push(arch),
            Err(e) => return Err(format!("Invalid architecture '{}': {}", part, e)),
        }
    }

    Ok(architectures)
}

/// Create a simple CSV row
pub fn create_csv_row(values: &[String]) -> String {
    values.iter()
        .map(|val| {
            if val.contains(',') || val.contains('"') {
                format!("\"{}\"", val.replace('"', "\"\""))
            } else {
                val.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Parse CSV row
pub fn parse_csv_row(row: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in row.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                values.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    values.push(current.trim().to_string());
    values
}

/// Validate file path
pub fn validate_file_path(path: &str) -> Result<(), String> {
    let path_obj = std::path::Path::new(path);
    
    if let Some(parent) = path_obj.parent() {
        if !parent.exists() {
            return Err(format!("Parent directory does not exist: {}", parent.display()));
        }
    }

    Ok(())
}

/// Ensure directory exists
pub fn ensure_directory_exists(path: &str) -> Result<(), String> {
    let path_obj = std::path::Path::new(path);
    
    if !path_obj.exists() {
        std::fs::create_dir_all(path_obj)
            .map_err(|e| format!("Failed to create directory {}: {}", path, e))?;
    }

    Ok(())
}

/// Simple file utility functions
pub mod file_utils {
    use std::fs;
    use std::io::Write;
    use super::*;

    /// Read entire file content
    pub fn read_file_to_string(path: &str) -> Result<String, String> {
        fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path, e))
    }

    /// Write string to file
    pub fn write_string_to_file(path: &str, content: &str) -> Result<(), String> {
        let mut file = fs::File::create(path)
            .map_err(|e| format!("Failed to create file {}: {}", path, e))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to file {}: {}", path, e))
    }

    /// Check if file exists
    pub fn file_exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }

    /// Get file size
    pub fn get_file_size(path: &str) -> Result<u64, String> {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to get metadata for {}: {}", path, e))?;
        
        Ok(metadata.len())
    }
}

/// Thread-safe counter
use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicCounter {
    counter: AtomicU64,
}

impl AtomicCounter {
    /// Create new atomic counter
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    /// Increment and return new value
    pub fn increment(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Get current value
    pub fn get(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }

    /// Reset counter
    pub fn reset(&self) {
        self.counter.store(0, Ordering::SeqCst);
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_conversion() {
        assert_eq!(architecture_to_string(&Architecture::X86_64), "x86_64");
        assert_eq!(string_to_architecture("arm64").unwrap(), Architecture::ARM64);
    }

    #[test]
    fn test_number_formatting() {
        assert_eq!(format_number_with_units(1024.0, "B"), "1.00kB");
        assert_eq!(format_number_with_units(1048576.0, "B"), "1.00MB");
    }

    #[test]
    fn test_byte_formatting() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = calculate_statistics(&values);
        
        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 0.001);
        assert!((stats.std_dev - 1.58).abs() < 0.01);
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new(100);
        tracker.update(25);
        
        assert_eq!(tracker.get_percentage(), 25.0);
        assert!(!tracker.get_progress_string().is_empty());
    }

    #[test]
    fn test_csv_parsing() {
        let row = create_csv_row(&["value1", "value2", "value3"]);
        let parsed = parse_csv_row(&row);
        
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], "value1");
    }
}