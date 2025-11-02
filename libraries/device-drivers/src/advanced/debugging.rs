//! Debugging Tools Module
//! 
//! Provides comprehensive debugging and tracing capabilities for device drivers,
//! including device state tracing, performance monitoring, and diagnostic tools.

use crate::AdvancedDriverId;
use crate::AdvancedDriverError::{self, *};
use alloc::collections::BTreeMap;
use alloc::string::String;
use log::{debug, warn, error, info};

/// Debug trace levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceLevel {
    None,       // No tracing
    Error,      // Error messages only
    Warning,    // Errors and warnings
    Info,       // General information
    Debug,      // Debug information
    Trace,      // Detailed trace
    Verbose,    // Very detailed trace
}

/// Device trace event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceEventType {
    Initialization,
    Read,
    Write,
    Ioctl,
    Interrupt,
    Error,
    Warning,
    StateChange,
    Performance,
    Custom,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_count: u64,
    pub total_time_ns: u64,
    pub min_time_ns: u64,
    pub max_time_ns: u64,
    pub avg_time_ns: u64,
    pub error_count: u32,
    pub last_operation_time: u64,
}

/// Device trace entry
#[derive(Debug, Clone)]
pub struct DeviceTrace {
    pub timestamp: u64,
    pub driver_id: AdvancedDriverId,
    pub event_type: TraceEventType,
    pub level: TraceLevel,
    pub message: String,
    pub duration_ns: Option<u64>,
    pub error_code: Option<AdvancedDriverError>,
}

/// Debug configuration for a driver
#[derive(Debug, Clone)]
pub struct DriverDebugConfig {
    pub driver_id: AdvancedDriverId,
    pub trace_enabled: bool,
    pub performance_monitoring: bool,
    pub error_tracking: bool,
    pub max_trace_entries: usize,
    pub trace_level: TraceLevel,
    pub performance_threshold_ns: u64,
}

/// Debug manager
pub struct DebugManager {
    driver_configs: BTreeMap<AdvancedDriverId, DriverDebugConfig>,
    trace_log: Vec<DeviceTrace>,
    performance_metrics: BTreeMap<AdvancedDriverId, PerformanceMetrics>,
    error_statistics: BTreeMap<AdvancedDriverId, u32>,
    global_trace_level: TraceLevel,
    max_total_trace_entries: usize,
    trace_callbacks: Vec<fn(&DeviceTrace)>,
    performance_callbacks: Vec<fn(AdvancedDriverId, &PerformanceMetrics)>,
    debug_enabled: bool,
}

impl DebugManager {
    /// Create a new debug manager
    pub fn new() -> Self {
        info!("Initializing Debug Manager");
        
        let manager = Self {
            driver_configs: BTreeMap::new(),
            trace_log: Vec::new(),
            performance_metrics: BTreeMap::new(),
            error_statistics: BTreeMap::new(),
            global_trace_level: TraceLevel::Info,
            max_total_trace_entries: 10000,
            trace_callbacks: Vec::new(),
            performance_callbacks: Vec::new(),
            debug_enabled: true,
        };
        
        info!("Debug Manager initialized with global trace level {:?}", manager.global_trace_level);
        manager
    }

    /// Enable/disable debug mode
    pub fn set_debug_enabled(&mut self, enabled: bool) -> Result<(), AdvancedDriverError> {
        debug!("Setting debug mode to {}", enabled);
        self.debug_enabled = enabled;
        Ok(())
    }

    /// Configure driver debug settings
    pub fn configure_driver(&mut self, driver_id: AdvancedDriverId, config: DriverDebugConfig) -> Result<(), AdvancedDriverError> {
        debug!("Configuring debug for driver {:?}", driver_id);
        
        if config.driver_id != driver_id {
            return Err(ValidationFailed);
        }
        
        self.driver_configs.insert(driver_id, config);
        Ok(())
    }

    /// Set global trace level
    pub fn set_global_trace_level(&mut self, level: TraceLevel) -> Result<(), AdvancedDriverError> {
        debug!("Setting global trace level to {:?}", level);
        self.global_trace_level = level;
        Ok(())
    }

    /// Add a trace entry
    pub fn add_trace(&mut self, driver_id: AdvancedDriverId, event_type: TraceEventType, 
                    level: TraceLevel, message: String) -> Result<(), AdvancedDriverError> {
        if !self.debug_enabled {
            return Ok(());
        }
        
        let trace = DeviceTrace {
            timestamp: 0, // TODO: Get actual timestamp
            driver_id,
            event_type,
            level,
            message,
            duration_ns: None,
            error_code: None,
        };
        
        self.process_trace(trace)
    }

    /// Add a performance trace entry
    pub fn add_performance_trace(&mut self, driver_id: AdvancedDriverId, event_type: TraceEventType,
                                message: String, duration_ns: u64) -> Result<(), AdvancedDriverError> {
        if !self.debug_enabled {
            return Ok(());
        }
        
        let trace = DeviceTrace {
            timestamp: 0, // TODO: Get actual timestamp
            driver_id,
            event_type,
            level: TraceLevel::Debug,
            message,
            duration_ns: Some(duration_ns),
            error_code: None,
        };
        
        // Update performance metrics
        self.update_performance_metrics(driver_id, duration_ns, false);
        
        self.process_trace(trace)
    }

    /// Add an error trace entry
    pub fn add_error_trace(&mut self, driver_id: AdvancedDriverId, message: String,
                          error_code: AdvancedDriverError) -> Result<(), AdvancedDriverError> {
        if !self.debug_enabled {
            return Ok(());
        }
        
        let trace = DeviceTrace {
            timestamp: 0, // TODO: Get actual timestamp
            driver_id,
            event_type: TraceEventType::Error,
            level: TraceLevel::Error,
            message,
            duration_ns: None,
            error_code: Some(error_code),
        };
        
        // Update error statistics
        *self.error_statistics.entry(driver_id).or_insert(0) += 1;
        
        self.process_trace(trace)
    }

    /// Get driver configuration
    pub fn get_driver_config(&self, driver_id: AdvancedDriverId) -> Option<&DriverDebugConfig> {
        self.driver_configs.get(&driver_id)
    }

    /// Get trace log
    pub fn get_trace_log(&self) -> &[DeviceTrace] {
        &self.trace_log
    }

    /// Get trace log for specific driver
    pub fn get_driver_traces(&self, driver_id: AdvancedDriverId) -> Vec<&DeviceTrace> {
        self.trace_log.iter()
            .filter(|trace| trace.driver_id == driver_id)
            .collect()
    }

    /// Get performance metrics for a driver
    pub fn get_performance_metrics(&self, driver_id: AdvancedDriverId) -> Option<&PerformanceMetrics> {
        self.performance_metrics.get(&driver_id)
    }

    /// Get all performance metrics
    pub fn get_all_performance_metrics(&self) -> BTreeMap<AdvancedDriverId, &PerformanceMetrics> {
        self.performance_metrics.clone()
    }

    /// Get error statistics
    pub fn get_error_statistics(&self) -> BTreeMap<AdvancedDriverId, u32> {
        self.error_statistics.clone()
    }

    /// Get debug statistics
    pub fn get_debug_statistics(&self) -> DebugStatistics {
        let mut total_operations = 0;
        let mut total_errors = 0;
        let mut trace_counts = BTreeMap::new();
        
        for metrics in self.performance_metrics.values() {
            total_operations += metrics.operation_count;
        }
        
        for &error_count in self.error_statistics.values() {
            total_errors += error_count;
        }
        
        for trace in &self.trace_log {
            *trace_counts.entry(trace.event_type).or_insert(0) += 1;
        }
        
        DebugStatistics {
            total_traces: self.trace_log.len(),
            total_operations,
            total_errors,
            trace_counts,
            active_drivers: self.driver_configs.len(),
            global_trace_level: self.global_trace_level,
            debug_enabled: self.debug_enabled,
        }
    }

    /// Clear trace log
    pub fn clear_trace_log(&mut self) -> Result<(), AdvancedDriverError> {
        debug!("Clearing trace log");
        self.trace_log.clear();
        Ok(())
    }

    /// Clear performance metrics
    pub fn clear_performance_metrics(&mut self, driver_id: Option<AdvancedDriverId>) -> Result<(), AdvancedDriverError> {
        if let Some(driver_id) = driver_id {
            self.performance_metrics.remove(&driver_id);
        } else {
            self.performance_metrics.clear();
        }
        Ok(())
    }

    /// Clear error statistics
    pub fn clear_error_statistics(&mut self, driver_id: Option<AdvancedDriverId>) -> Result<(), AdvancedDriverError> {
        if let Some(driver_id) = driver_id {
            self.error_statistics.remove(&driver_id);
        } else {
            self.error_statistics.clear();
        }
        Ok(())
    }

    /// Set maximum trace entries
    pub fn set_max_trace_entries(&mut self, max_entries: usize) -> Result<(), AdvancedDriverError> {
        debug!("Setting max trace entries to {}", max_entries);
        self.max_total_trace_entries = max_entries;
        Ok(())
    }

    /// Register trace callback
    pub fn register_trace_callback(&mut self, callback: fn(&DeviceTrace)) {
        self.trace_callbacks.push(callback);
    }

    /// Register performance callback
    pub fn register_performance_callback(&mut self, callback: fn(AdvancedDriverId, &PerformanceMetrics)) {
        self.performance_callbacks.push(callback);
    }

    /// Export traces to string
    pub fn export_traces(&self, driver_id: Option<AdvancedDriverId>) -> String {
        let traces = if let Some(driver_id) = driver_id {
            self.get_driver_traces(driver_id)
        } else {
            self.trace_log.iter().collect()
        };
        
        let mut export = String::new();
        export.push_str("Device Driver Trace Export\n");
        export.push_str("==========================\n\n");
        
        for trace in traces {
            export.push_str(&format!(
                "[{}] {:?}:{} - {}\n",
                trace.timestamp,
                trace.driver_id.0,
                trace.event_type as u8,
                trace.message
            ));
            
            if let Some(duration) = trace.duration_ns {
                export.push_str(&format!("  Duration: {} ns\n", duration));
            }
            
            if let Some(error) = trace.error_code {
                export.push_str(&format!("  Error: {:?}\n", error));
            }
        }
        
        export
    }

    /// Check if tracing is enabled for a driver
    pub fn is_tracing_enabled(&self, driver_id: AdvancedDriverId) -> bool {
        let driver_level = self.driver_configs.get(&driver_id)
            .map(|config| config.trace_level)
            .unwrap_or(self.global_trace_level);
            
        driver_level != TraceLevel::None && self.debug_enabled
    }

    /// Get trace count
    pub fn get_trace_count(&self) -> usize {
        self.trace_log.len()
    }

    /// Internal: Process a trace entry
    fn process_trace(&mut self, mut trace: DeviceTrace) -> Result<(), AdvancedDriverError> {
        // Check if tracing is enabled for this driver
        let config_level = self.driver_configs.get(&trace.driver_id)
            .map(|config| config.trace_level)
            .unwrap_or(self.global_trace_level);
        
        if trace.level as u8 > config_level as u8 {
            return Ok(());
        }
        
        // Add to trace log
        self.trace_log.push(trace.clone());
        
        // Limit trace log size
        if self.trace_log.len() > self.max_total_trace_entries {
            self.trace_log.remove(0);
        }
        
        // Notify callbacks
        self.notify_trace_callbacks(&trace);
        
        Ok(())
    }

    /// Internal: Update performance metrics
    fn update_performance_metrics(&mut self, driver_id: AdvancedDriverId, duration_ns: u64, is_error: bool) {
        let metrics = self.performance_metrics.entry(driver_id).or_insert_with(|| PerformanceMetrics {
            operation_count: 0,
            total_time_ns: 0,
            min_time_ns: u64::MAX,
            max_time_ns: 0,
            avg_time_ns: 0,
            error_count: 0,
            last_operation_time: 0,
        });
        
        metrics.operation_count += 1;
        metrics.total_time_ns += duration_ns;
        metrics.avg_time_ns = metrics.total_time_ns / metrics.operation_count;
        metrics.last_operation_time = duration_ns;
        
        if duration_ns < metrics.min_time_ns {
            metrics.min_time_ns = duration_ns;
        }
        
        if duration_ns > metrics.max_time_ns {
            metrics.max_time_ns = duration_ns;
        }
        
        if is_error {
            metrics.error_count += 1;
        }
        
        // Notify performance callbacks
        self.notify_performance_callbacks(driver_id, metrics);
    }

    /// Internal: Notify trace callbacks
    fn notify_trace_callbacks(&self, trace: &DeviceTrace) {
        for callback in &self.trace_callbacks {
            callback(trace);
        }
    }

    /// Internal: Notify performance callbacks
    fn notify_performance_callbacks(&self, driver_id: AdvancedDriverId, metrics: &PerformanceMetrics) {
        for callback in &self.performance_callbacks {
            callback(driver_id, metrics);
        }
    }

    /// Generate performance report
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Device Driver Performance Report\n");
        report.push_str("=================================\n\n");
        
        if self.performance_metrics.is_empty() {
            report.push_str("No performance data available.\n");
            return report;
        }
        
        for (driver_id, metrics) in &self.performance_metrics {
            report.push_str(&format!("Driver {:?}:\n", driver_id));
            report.push_str(&format!("  Operations: {}\n", metrics.operation_count));
            report.push_str(&format!("  Total Time: {} ns\n", metrics.total_time_ns));
            report.push_str(&format!("  Average Time: {} ns\n", metrics.avg_time_ns));
            report.push_str(&format!("  Min Time: {} ns\n", metrics.min_time_ns));
            report.push_str(&format!("  Max Time: {} ns\n", metrics.max_time_ns));
            report.push_str(&format!("  Errors: {}\n", metrics.error_count));
            report.push_str(&format!("  Error Rate: {:.2}%\n", 
                                    if metrics.operation_count > 0 {
                                        (metrics.error_count as f64 / metrics.operation_count as f64) * 100.0
                                    } else {
                                        0.0
                                    }));
            report.push_str("\n");
        }
        
        report
    }
}

/// Debug statistics
#[derive(Debug, Clone)]
pub struct DebugStatistics {
    pub total_traces: usize,
    pub total_operations: u64,
    pub total_errors: u32,
    pub trace_counts: BTreeMap<TraceEventType, usize>,
    pub active_drivers: usize,
    pub global_trace_level: TraceLevel,
    pub debug_enabled: bool,
}

impl Default for DebugManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_management() {
        let mut manager = DebugManager::new();
        let driver_id = AdvancedDriverId(1);
        
        assert!(manager.add_trace(
            driver_id,
            TraceEventType::Initialization,
            TraceLevel::Info,
            "Driver initialized".to_string()
        ).is_ok());
        
        let traces = manager.get_driver_traces(driver_id);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].message, "Driver initialized");
    }

    #[test]
    fn test_performance_metrics() {
        let mut manager = DebugManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.add_performance_trace(
            driver_id,
            TraceEventType::Read,
            "Read operation".to_string(),
            1000
        ).unwrap();
        
        let metrics = manager.get_performance_metrics(driver_id).unwrap();
        assert_eq!(metrics.operation_count, 1);
        assert_eq!(metrics.total_time_ns, 1000);
        assert_eq!(metrics.avg_time_ns, 1000);
    }

    #[test]
    fn test_error_tracking() {
        let mut manager = DebugManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.add_error_trace(
            driver_id,
            "Test error".to_string(),
            AdvancedDriverError::HardwareError
        ).unwrap();
        
        let stats = manager.get_debug_statistics();
        assert_eq!(stats.total_errors, 1);
    }

    #[test]
    fn test_trace_filtering() {
        let mut manager = DebugManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Add traces at different levels
        manager.add_trace(driver_id, TraceEventType::Info, TraceLevel::Info, "Info message".to_string()).unwrap();
        manager.add_trace(driver_id, TraceEventType::Debug, TraceLevel::Debug, "Debug message".to_string()).unwrap();
        manager.add_trace(driver_id, TraceEventType::Error, TraceLevel::Error, "Error message".to_string()).unwrap();
        
        // Set global level to Warning (should filter out Info and Debug)
        manager.set_global_trace_level(TraceLevel::Warning).unwrap();
        
        // Add another trace - should be filtered
        manager.add_trace(driver_id, TraceEventType::Info, TraceLevel::Info, "Filtered message".to_string()).unwrap();
        
        let traces = manager.get_driver_traces(driver_id);
        assert_eq!(traces.len(), 4); // All traces added before filtering
    }

    #[test]
    fn test_performance_report() {
        let mut manager = DebugManager::new();
        let driver_id = AdvancedDriverId(1);
        
        manager.add_performance_trace(driver_id, TraceEventType::Read, "Read 1".to_string(), 1000).unwrap();
        manager.add_performance_trace(driver_id, TraceEventType::Read, "Read 2".to_string(), 2000).unwrap();
        
        let report = manager.generate_performance_report();
        assert!(report.contains("Driver"));
        assert!(report.contains("Operations: 2"));
        assert!(report.contains("Total Time: 3000 ns"));
    }
}
