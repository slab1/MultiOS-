//! Utility functions for the MultiOS Regression Testing System
//!
//! This module provides common utility functions for file handling,
//! data processing, math operations, and other helper functionality.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Generate a unique identifier for test runs
pub fn generate_test_id() -> String {
    format!("test_{}", uuid::Uuid::new_v4())
}

/// Parse duration string (e.g., "30s", "5m", "2h") into Duration
pub fn parse_duration(duration_str: &str) -> Result<Duration> {
    let parts: Vec<&str> = duration_str.split(' ').collect();
    let mut total_duration = Duration::from_secs(0);
    
    for part in parts {
        if part.is_empty() {
            continue;
        }
        
        let (num_str, unit) = part.split_at(part.len() - 1);
        let num: u64 = num_str.parse()
            .map_err(|_| anyhow::anyhow!("Invalid duration number: {}", num_str))?;
        
        match unit {
            "s" => total_duration += Duration::from_secs(num),
            "m" => total_duration += Duration::from_secs(num * 60),
            "h" => total_duration += Duration::from_secs(num * 3600),
            "d" => total_duration += Duration::from_secs(num * 86400),
            _ => return Err(anyhow::anyhow!("Invalid duration unit: {}", unit)),
        }
    }
    
    Ok(total_duration)
}

/// Format duration into human-readable string
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Calculate statistical mean of a slice of numbers
pub fn mean<T: std::ops::Add<Output = T> + std::ops::Div<Output = T> + Copy + Default>(values: &[T]) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    
    let sum = values.iter().fold(T::default(), |acc, &x| acc + x);
    Some(sum / (values.len() as u32).into())
}

/// Calculate standard deviation of a slice of numbers
pub fn std_dev(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    
    let mean = mean(values)?;
    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / (values.len() as f64 - 1.0);
    
    Some(variance.sqrt())
}

/// Calculate percentile of a sorted slice
pub fn percentile(sorted_values: &[f64], p: f64) -> Option<f64> {
    if sorted_values.is_empty() || p <= 0.0 || p >= 100.0 {
        return None;
    }
    
    let index = (p / 100.0) * (sorted_values.len() as f64 - 1.0);
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        Some(sorted_values[lower])
    } else {
        let weight = index.fract();
        Some(sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight)
    }
}

/// Remove outliers from a slice using the IQR method
pub fn remove_outliers(values: &mut Vec<f64>) {
    if values.len() < 4 {
        return;
    }
    
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let q1 = percentile(values, 25.0).unwrap_or(values[0]);
    let q3 = percentile(values, 75.0).unwrap_or(values[values.len() - 1]);
    let iqr = q3 - q1;
    let lower_bound = q1 - 1.5 * iqr;
    let upper_bound = q3 + 1.5 * iqr;
    
    values.retain(|&x| x >= lower_bound && x <= upper_bound);
}

/// Check if file exists and is readable
pub fn is_readable_file(path: &str) -> bool {
    fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}

/// Check if directory exists and is writable
pub fn is_writable_dir(path: &str) -> bool {
    fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false) &&
        fs::metadata(path).map(|m| m.permissions().readonly() == false).unwrap_or(false)
}

/// Create directory recursively if it doesn't exist
pub fn ensure_dir_exists(path: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Read entire file contents into string
pub fn read_file_to_string(path: &str) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

/// Write string to file
pub fn write_string_to_file(path: &str, contents: &str) -> Result<()> {
    ensure_dir_exists(Path::new(path).parent().unwrap_or_else(|| Path::new("")))?;
    fs::write(path, contents)?;
    Ok(())
}

/// Hash a string using SHA256
pub fn hash_string(input: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Convert system time to UTC datetime
pub fn system_time_to_datetime(time: SystemTime) -> DateTime<Utc> {
    DateTime::from(time)
}

/// Convert UNIX timestamp to UTC datetime
pub fn timestamp_to_datetime(timestamp: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now())
}

/// Convert datetime to UNIX timestamp
pub fn datetime_to_timestamp(datetime: &DateTime<Utc>) -> i64 {
    datetime.timestamp()
}

/// Validate email address format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && 
    email.chars().position(|c| c == '@').unwrap_or(0) < email.len() - 1 &&
    email.chars().rev().position(|c| c == '.').unwrap_or(0) > 1
}

/// Sanitize filename by removing invalid characters
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Format file size in human-readable format
pub fn format_file_size(size_bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size_bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size_bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Retry an operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    max_retries: u32,
    initial_delay: Duration,
    operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    
    for attempt in 0..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                if attempt == max_retries {
                    return Err(anyhow::anyhow!("Operation failed after {} attempts: {}", max_retries + 1, err));
                }
                
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
    
    unreachable!()
}

/// Parse key-value pairs from string
pub fn parse_key_value_pairs(input: &str) -> HashMap<String, String> {
    let mut pairs = HashMap::new();
    
    for line in input.lines() {
        if let Some((key, value)) = line.split_once('=') {
            pairs.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    
    pairs
}

/// Merge two HashMaps, with the second map overriding values from the first
pub fn merge_hashmaps<K: Clone + std::hash::Hash + Eq, V: Clone>(
    base: &HashMap<K, V>,
    override_map: &HashMap<K, V>,
) -> HashMap<K, V> {
    let mut result = base.clone();
    result.extend(override_map.clone());
    result
}

/// Convert a result to an option, logging errors
pub fn log_result<T, E: std::fmt::Display>(
    result: Result<T, E>,
    log_level: log::Level,
    message: &str,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            match log_level {
                log::Level::Error => log::error!("{}: {}", message, error),
                log::Level::Warn => log::warn!("{}: {}", message, error),
                log::Level::Info => log::info!("{}: {}", message, error),
                log::Level::Debug => log::debug!("{}: {}", message, error),
                log::Level::Trace => log::trace!("{}: {}", message, error),
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
        assert_eq!(parse_duration("1h 30m 45s").unwrap(), Duration::from_secs(5445));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3723)), "1h 2m");
        assert_eq!(format_duration(Duration::from_secs(90000)), "1d 1h");
    }

    #[test]
    fn test_mean() {
        assert_eq!(mean(&[1.0, 2.0, 3.0]).unwrap(), 2.0);
        assert_eq!(mean(&[5.0]).unwrap(), 5.0);
        assert_eq!(mean(&[0.0, 0.0, 0.0]).unwrap(), 0.0);
        assert_eq!(mean::<i32>(&[]), None);
    }

    #[test]
    fn test_std_dev() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = std_dev(&values).unwrap();
        assert!((result - 2.138).abs() < 0.01);
    }

    #[test]
    fn test_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(percentile(&values, 25.0).unwrap(), 2.5);
        assert_eq!(percentile(&values, 50.0).unwrap(), 5.5);
        assert_eq!(percentile(&values, 75.0).unwrap(), 7.5);
        assert_eq!(percentile(&[], 50.0), None);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test<file>.txt"), "test_file_.txt");
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
        assert_eq!(sanitize_filename("file|with|pipe.txt"), "file_with_pipe.txt");
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.org"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(512), "512 B");
    }

    #[test]
    fn test_parse_key_value_pairs() {
        let input = "key1=value1\nkey2=value2\nkey3=value3";
        let pairs = parse_key_value_pairs(input);
        
        assert_eq!(pairs.get("key1").unwrap(), "value1");
        assert_eq!(pairs.get("key2").unwrap(), "value2");
        assert_eq!(pairs.get("key3").unwrap(), "value3");
    }
}