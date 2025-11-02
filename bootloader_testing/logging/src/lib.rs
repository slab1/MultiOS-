//! Bootloader Testing Logging System
//! 
//! Comprehensive logging infrastructure for bootloader testing including
//! structured logging, performance metrics, and real-time monitoring.

use chrono::{DateTime, Utc, Local};
use log::{Log, Metadata, Record, Level, logger};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub module: String,
    pub test_id: Option<Uuid>,
    pub message: String,
    pub metadata: HashMap<String, String>,
    pub call_site: Option<String>,
    pub thread_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub id: Uuid,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub architecture: String,
    pub test_count: usize,
    pub pass_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub timeout_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub boot_time_ms: u64,
    pub memory_usage_kb: u64,
    pub cpu_usage_percent: f64,
    pub disk_io_mb: u64,
    pub network_io_mb: u64,
}

pub struct BootloaderLogger {
    inner: Arc<Mutex<BootloaderLoggerInner>>,
    config: LoggerConfig,
}

#[derive(Clone)]
struct LoggerConfig {
    output_dir: PathBuf,
    max_file_size_mb: u64,
    max_files: u32,
    enable_performance_metrics: bool,
    enable_structured_logging: bool,
    log_level: Level,
}

struct BootloaderLoggerInner {
    current_session: Option<TestSession>,
    log_entries: Vec<LogEntry>,
    active_tests: HashMap<Uuid, TestSession>,
    performance_data: HashMap<Uuid, PerformanceMetrics>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./test_logs"),
            max_file_size_mb: 100,
            max_files: 10,
            enable_performance_metrics: true,
            enable_structured_logging: true,
            log_level: Level::Info,
        }
    }
}

impl BootloaderLogger {
    /// Create a new logger with configuration
    pub fn new(config: LoggerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Create output directory
        fs::create_dir_all(&config.output_dir)?;
        
        let logger = Self {
            inner: Arc::new(Mutex::new(BootloaderLoggerInner {
                current_session: None,
                log_entries: Vec::new(),
                active_tests: HashMap::new(),
                performance_data: HashMap::new(),
            })),
            config,
        };

        // Initialize logging
        log::set_max_level(config.log_level);
        log::set_boxed_logger(Box::new(logger.clone()))?;

        Ok(logger)
    }

    /// Create logger with default configuration
    pub fn default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new(LoggerConfig::default())
    }

    /// Start a new test session
    pub fn start_session(&self, name: String, architecture: String) -> Uuid {
        let session_id = Uuid::new_v4();
        let session = TestSession {
            id: session_id,
            name,
            start_time: Utc::now(),
            end_time: None,
            status: SessionStatus::Running,
            architecture,
            test_count: 0,
            pass_count: 0,
            fail_count: 0,
            skip_count: 0,
            timeout_count: 0,
        };

        let mut inner = self.inner.lock().unwrap();
        inner.current_session = Some(session.clone());
        inner.active_tests.insert(session_id, session);

        self.log(session_id, LogLevel::Info, "Session Started", |map| {
            map.insert("session_name".to_string(), name);
            map.insert("architecture".to_string(), architecture);
        });

        session_id
    }

    /// End a test session
    pub fn end_session(&self, session_id: Uuid, status: SessionStatus) {
        let mut inner = self.inner.lock().unwrap();
        
        if let Some(session) = inner.active_tests.get_mut(&session_id) {
            session.end_time = Some(Utc::now());
            session.status = status.clone();
            
            // Update current session if it matches
            if let Some(current) = &mut inner.current_session {
                if current.id == session_id {
                    *current = session.clone();
                }
            }
        }

        self.log(session_id, LogLevel::Info, "Session Ended", |map| {
            map.insert("status".to_string(), format!("{:?}", status));
        });

        // Flush logs to file
        self.flush_logs(&session_id);
    }

    /// Log a message with optional metadata
    pub fn log<F>(&self, test_id: Uuid, level: LogLevel, message: &str, metadata_fn: F)
    where
        F: FnOnce(&mut HashMap<String, String>),
    {
        let entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            module: self.get_caller_module(),
            test_id: Some(test_id),
            message: message.to_string(),
            metadata: {
                let mut map = HashMap::new();
                metadata_fn(&mut map);
                map
            },
            call_site: Some(self.get_caller_location()),
            thread_id: std::thread::current().id().as_u64(),
        };

        let mut inner = self.inner.lock().unwrap();
        inner.log_entries.push(entry);

        // If structured logging is enabled, also write to file
        if self.config.enable_structured_logging {
            self.write_structured_log(&entry);
        }
    }

    /// Log performance metrics
    pub fn log_performance(&self, test_id: Uuid, metrics: PerformanceMetrics) {
        let mut inner = self.inner.lock().unwrap();
        inner.performance_data.insert(test_id, metrics);

        self.log(test_id, LogLevel::Debug, "Performance Metrics", |map| {
            map.insert("boot_time_ms".to_string(), metrics.boot_time_ms.to_string());
            map.insert("memory_usage_kb".to_string(), metrics.memory_usage_kb.to_string());
            map.insert("cpu_usage_percent".to_string(), metrics.cpu_usage_percent.to_string());
            map.insert("disk_io_mb".to_string(), metrics.disk_io_mb.to_string());
            map.insert("network_io_mb".to_string(), metrics.network_io_mb.to_string());
        });
    }

    /// Get session statistics
    pub fn get_session_stats(&self, session_id: Uuid) -> Option<TestSession> {
        let inner = self.inner.lock().unwrap();
        inner.active_tests.get(&session_id).cloned()
    }

    /// Get all log entries for a session
    pub fn get_session_logs(&self, session_id: Uuid) -> Vec<LogEntry> {
        let inner = self.inner.lock().unwrap();
        inner.log_entries
            .iter()
            .filter(|entry| entry.test_id == Some(session_id))
            .cloned()
            .collect()
    }

    /// Export logs to JSON file
    pub fn export_logs(&self, session_id: Uuid, output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let logs = self.get_session_logs(session_id);
        let logs_json = serde_json::to_string_pretty(&logs)?;
        fs::write(output_path, logs_json)?;
        Ok(())
    }

    /// Get performance data for a session
    pub fn get_performance_data(&self, session_id: Uuid) -> Option<PerformanceMetrics> {
        let inner = self.inner.lock().unwrap();
        inner.performance_data.get(&session_id).cloned()
    }

    /// Clear all logs
    pub fn clear_logs(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.log_entries.clear();
        inner.performance_data.clear();
    }

    /// Flush logs to disk
    fn flush_logs(&self, session_id: &Uuid) {
        if let Some(session) = self.get_session_stats(*session_id) {
            let log_file = self.config.output_dir.join(format!("session_{}.log", session.id));
            
            let logs = self.get_session_logs(*session_id);
            if !logs.is_empty() {
                // Write in structured format
                let log_content: Vec<String> = logs.iter()
                    .map(|entry| serde_json::to_string(entry).unwrap_or_default())
                    .collect();
                
                let content = log_content.join("\n");
                let _ = fs::write(&log_file, content);
            }
        }
    }

    /// Write structured log entry to file
    fn write_structured_log(&self, entry: &LogEntry) {
        let log_file = self.config.output_dir.join("structured.log");
        let log_line = serde_json::to_string(entry).unwrap_or_default();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(&log_file)
            .and_then(|mut file| file.write_all(format!("{}\n", log_line).as_bytes()));
    }

    /// Get caller module name
    fn get_caller_module(&self) -> String {
        // Simplified - in real implementation, this would use backtrace or caller_info
        "bootloader_test".to_string()
    }

    /// Get caller location
    fn get_caller_location(&self) -> String {
        // Simplified - in real implementation, this would get actual file:line
        format!("{}:{}", file!(), line!())
    }
}

impl Clone for BootloaderLogger {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            config: self.config.clone(),
        }
    }
}

impl Log for BootloaderLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true // Logger is always enabled
    }

    fn log(&self, record: &Record) {
        let level = match record.level() {
            Level::Trace => LogLevel::Trace,
            Level::Debug => LogLevel::Debug,
            Level::Info => LogLevel::Info,
            Level::Warn => LogLevel::Warn,
            Level::Error => LogLevel::Error,
        };

        let mut metadata = HashMap::new();
        metadata.insert("file".to_string(), record.file().unwrap_or("unknown").to_string());
        metadata.insert("line".to_string(), record.line().unwrap_or(0).to_string());
        metadata.insert("target".to_string(), record.target().to_string());

        let entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            module: "bootloader".to_string(),
            test_id: None,
            message: record.args().to_string(),
            metadata,
            call_site: Some(format!("{}:{}", 
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0)
            )),
            thread_id: std::thread::current().id().as_u64(),
        };

        let mut inner = self.inner.lock().unwrap();
        inner.log_entries.push(entry);

        // Write to structured log if enabled
        if self.config.enable_structured_logging {
            self.write_structured_log(&entry);
        }
    }

    fn flush(&self) {
        // Ensure all logs are written
        if let Some(ref session) = self.inner.lock().unwrap().current_session {
            self.flush_logs(&session.id);
        }
    }
}

/// Performance monitor for tracking system metrics during testing
pub struct PerformanceMonitor {
    start_time: SystemTime,
    start_cpu_time: std::time::Instant,
    memory_start: Option<usize>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            start_cpu_time: std::time::Instant::now(),
            memory_start: Self::get_memory_usage(),
        }
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        let current_time = SystemTime::now();
        let duration = current_time.duration_since(self.start_time).unwrap();
        let boot_time_ms = duration.as_millis() as u64;

        let current_memory = Self::get_memory_usage();
        let memory_usage_kb = current_memory.unwrap_or(0) as u64;

        let cpu_duration = self.start_cpu_time.elapsed();
        let cpu_usage_percent = (cpu_duration.as_millis() as f64) / (duration.as_millis() as f64) * 100.0;

        PerformanceMetrics {
            boot_time_ms,
            memory_usage_kb,
            cpu_usage_percent: cpu_usage_percent.min(100.0),
            disk_io_mb: 0, // Would be implemented with actual I/O monitoring
            network_io_mb: 0, // Would be implemented with actual network monitoring
        }
    }

    fn get_memory_usage() -> Option<usize> {
        // Simplified memory usage - in real implementation this would use platform-specific APIs
        // For Linux: read /proc/self/status
        // For Windows: use GetProcessMemoryInfo
        // For macOS: use getrusage
        Some(0) // Placeholder
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Test logging macro
#[macro_export]
macro_rules! test_log {
    ($logger:expr, $level:expr, $test_id:expr, $message:expr) => {
        $logger.log($test_id, $level, $message, |_| {});
    };
    
    ($logger:expr, $level:expr, $test_id:expr, $message:expr, $($key:literal => $value:expr),* ) => {
        $logger.log($test_id, $level, $message, |map| {
            $(
                map.insert($key.to_string(), $value.to_string());
            )*
        });
    };
}

/// Performance logging macro
#[macro_export]
macro_rules! log_performance {
    ($logger:expr, $test_id:expr, $monitor:expr) => {
        let metrics = $monitor.get_metrics();
        $logger.log_performance($test_id, metrics);
    };
}

/// Structured logging examples

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = LoggerConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = BootloaderLogger::new(config).unwrap();
        assert!(logger.inner.lock().unwrap().log_entries.is_empty());
    }

    #[test]
    fn test_session_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = LoggerConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let logger = BootloaderLogger::new(config).unwrap();
        let session_id = logger.start_session("Test Session".to_string(), "x86_64".to_string());

        // Log some messages
        logger.log(session_id, LogLevel::Info, "Test message", |map| {
            map.insert("test_param".to_string(), "value".to_string());
        });

        // End session
        logger.end_session(session_id, SessionStatus::Completed);

        // Check session stats
        let stats = logger.get_session_stats(session_id);
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.name, "Test Session");
        assert_eq!(stats.architecture, "x86_64");
        assert!(stats.end_time.is_some());
        assert!(matches!(stats.status, SessionStatus::Completed));
    }

    #[test]
    fn test_performance_monitoring() {
        let monitor = PerformanceMonitor::new();
        
        // Simulate some work
        std::thread::sleep(Duration::from_millis(10));
        
        let metrics = monitor.get_metrics();
        assert!(metrics.boot_time_ms >= 10);
        assert!(metrics.memory_usage_kb >= 0);
        assert!(metrics.cpu_usage_percent >= 0.0);
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            module: "test".to_string(),
            test_id: Some(Uuid::new_v4()),
            message: "Test message".to_string(),
            metadata: HashMap::new(),
            call_site: Some("test.rs:10".to_string()),
            thread_id: 1,
        };

        let serialized = serde_json::to_string(&entry).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.message, deserialized.message);
    }
}
