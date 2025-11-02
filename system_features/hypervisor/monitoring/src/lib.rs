//! Performance Monitoring and Debugging Tools
//! 
//! Provides comprehensive performance monitoring, debugging tools, and analysis
//! for virtualized environments and educational purposes.

use crate::{VmId, VcpuId, HypervisorError};
use crate::core::{VmState, VmStats, CpuStats, HypervisorStats, MemoryStats};
use crate::cpu::{VmExitReason, VmcsRegion, VmcbRegion};
use crate::memory::{MemoryManager, PerformanceCounters};

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::RwLock;
use core::time::Duration;

/// Performance metric types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricType {
    CPUUtilization,
    MemoryUtilization,
    VMExitRate,
    InstructionRate,
    IORate,
    NetworkThroughput,
    ContextSwitchRate,
    PageFaultRate,
    HypervisorOverhead,
}

/// Performance sample structure
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    pub timestamp_ms: u64,
    pub vm_id: Option<VmId>,
    pub vcpu_id: Option<VcpuId>,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub sample_interval_ms: u32,
    pub retention_period_hours: u32,
    pub metrics_to_monitor: Vec<MetricType>,
    pub alert_thresholds: BTreeMap<MetricType, f64>,
    pub enable_debugging: bool,
    pub enable_tracing: bool,
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub metric_type: MetricType,
    pub current_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub timestamp_ms: u64,
    pub vm_id: Option<VmId>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Debug trace entry
#[derive(Debug, Clone)]
pub struct DebugTraceEntry {
    pub timestamp_ns: u64,
    pub trace_type: TraceType,
    pub vm_id: Option<VmId>,
    pub vcpu_id: Option<VcpuId>,
    pub data: TraceData,
}

/// Trace types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraceType {
    VMEntry,
    VMExit,
    InstructionExecution,
    MemoryAccess,
    DeviceAccess,
    Interrupt,
    Exception,
    SystemCall,
}

/// Trace data
#[derive(Debug, Clone)]
pub enum TraceData {
    VMExitReason(VmExitReason),
    InstructionPointer(u64),
    RegisterState([u64; 16]),
    MemoryAddress(u64, u64), // (address, size)
    DeviceInfo(String, u64), // (device, address)
    InterruptInfo(u8, u64),  // (vector, error_code)
    ExceptionInfo(u8, u64),  // (vector, error_code)
}

/// Performance profiling data
#[derive(Debug, Clone)]
pub struct ProfilingData {
    pub vm_id: VmId,
    pub profile_type: ProfileType,
    pub duration_ms: u64,
    pub samples: Vec<PerformanceSample>,
    pub summary: ProfileSummary,
}

/// Profile types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProfileType {
    CPU,
    Memory,
    I/O,
    Network,
    All,
}

/// Profile summary statistics
#[derive(Debug, Clone)]
pub struct ProfileSummary {
    pub total_samples: usize,
    pub average_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub standard_deviation: f64,
    pub percentiles: BTreeMap<f32, f64>, // e.g., 50.0 -> median, 95.0 -> 95th percentile
}

/// Performance Monitor and Debugger
pub struct PerformanceMonitor {
    /// Monitoring configuration
    config: MonitoringConfig,
    /// Performance samples storage
    samples: Vec<PerformanceSample>,
    /// Real-time metrics
    realtime_metrics: BTreeMap<VmId, BTreeMap<MetricType, f64>>,
    /// Active alerts
    alerts: Vec<PerformanceAlert>,
    /// Debug traces
    traces: Vec<DebugTraceEntry>,
    /// Profiling sessions
    profiling_sessions: BTreeMap<String, ProfilingData>,
    /// Monitoring start time
    start_time_ms: u64,
    /// Total samples collected
    total_samples_collected: u64,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: MonitoringConfig) -> Self {
        PerformanceMonitor {
            config,
            samples: Vec::new(),
            realtime_metrics: BTreeMap::new(),
            alerts: Vec::new(),
            traces: Vec::new(),
            profiling_sessions: BTreeMap::new(),
            start_time_ms: 0, // Would use actual timestamp
            total_samples_collected: 0,
        }
    }
    
    /// Start monitoring
    pub fn start_monitoring(&mut self) -> Result<(), HypervisorError> {
        if self.config.enabled {
            return Err(HypervisorError::ConfigurationError(String::from("Monitoring already enabled")));
        }
        
        self.config.enabled = true;
        self.start_time_ms = self.get_current_time_ms();
        
        info!("Started performance monitoring with {} metrics", self.config.metrics_to_monitor.len());
        Ok(())
    }
    
    /// Stop monitoring
    pub fn stop_monitoring(&mut self) -> Result<(), HypervisorError> {
        if !self.config.enabled {
            return Err(HypervisorError::ConfigurationError(String::from("Monitoring already disabled")));
        }
        
        self.config.enabled = false;
        
        info!("Stopped performance monitoring. Collected {} samples", self.total_samples_collected);
        Ok(())
    }
    
    /// Collect performance sample
    pub fn collect_sample(&mut self, sample: PerformanceSample) -> Result<(), HypervisorError> {
        if !self.config.enabled {
            return Err(HypervisorError::ConfigurationError(String::from("Monitoring not enabled")));
        }
        
        // Store sample if retention period allows
        if self.should_retain_sample(sample.timestamp_ms) {
            self.samples.push(sample.clone());
            self.total_samples_collected += 1;
        }
        
        // Update real-time metrics
        if let Some(vm_id) = sample.vm_id {
            self.realtime_metrics.entry(vm_id)
                .or_insert_with(BTreeMap::new)
                .insert(sample.metric_type, sample.value);
        }
        
        // Check for alerts
        self.check_alerts(&sample)?;
        
        // Add trace if enabled
        if self.config.enable_tracing {
            self.add_trace_entry(sample)?;
        }
        
        Ok(())
    }
    
    /// Collect VM performance metrics
    pub fn collect_vm_metrics(&mut self, vm_id: VmId, vm_stats: &VmStats, hypervisor_stats: &HypervisorStats) -> Result<(), HypervisorError> {
        let timestamp = self.get_current_time_ms();
        
        // Collect CPU metrics
        for (i, cpu_stat) in vm_stats.vcpu_stats.iter().enumerate() {
            let cpu_util = self.calculate_cpu_utilization(cpu_stat, timestamp);
            self.collect_sample(PerformanceSample {
                timestamp_ms: timestamp,
                vm_id: Some(vm_id),
                vcpu_id: Some(VcpuId(i as u32)),
                metric_type: MetricType::CPUUtilization,
                value: cpu_util,
                unit: String::from("percent"),
            })?;
            
            // VM exit rate
            let exit_rate = self.calculate_vm_exit_rate(cpu_stat, timestamp);
            self.collect_sample(PerformanceSample {
                timestamp_ms: timestamp,
                vm_id: Some(vm_id),
                vcpu_id: Some(VcpuId(i as u32)),
                metric_type: MetricType::VMExitRate,
                value: exit_rate,
                unit: String::from("exits/second"),
            })?;
            
            // Instruction rate
            let instr_rate = self.calculate_instruction_rate(cpu_stat, timestamp);
            self.collect_sample(PerformanceSample {
                timestamp_ms: timestamp,
                vm_id: Some(vm_id),
                vcpu_id: Some(VcpuId(i as u32)),
                metric_type: MetricType::InstructionRate,
                value: instr_rate,
                unit: String::from("instructions/second"),
            })?;
        }
        
        // Collect memory metrics
        let mem_util = self.calculate_memory_utilization(&vm_stats.memory_stats);
        self.collect_sample(PerformanceSample {
            timestamp_ms: timestamp,
            vm_id: Some(vm_id),
            vcpu_id: None,
            metric_type: MetricType::MemoryUtilization,
            value: mem_util,
            unit: String::from("percent"),
        })?;
        
        // Collect hypervisor overhead
        let overhead = self.calculate_hypervisor_overhead(hypervisor_stats);
        self.collect_sample(PerformanceSample {
            timestamp_ms: timestamp,
            vm_id: Some(vm_id),
            vcpu_id: None,
            metric_type: MetricType::HypervisorOverhead,
            value: overhead,
            unit: String::from("percent"),
        })?;
        
        Ok(())
    }
    
    /// Add debug trace entry
    pub fn add_trace_entry(&mut self, sample: PerformanceSample) -> Result<(), HypervisorError> {
        if !self.config.enable_tracing {
            return Ok(());
        }
        
        let trace_entry = DebugTraceEntry {
            timestamp_ns: sample.timestamp_ms * 1_000_000,
            trace_type: match sample.metric_type {
                MetricType::CPUUtilization => TraceType::InstructionExecution,
                MetricType::VMExitRate => TraceType::VMExit,
                MetricType::MemoryUtilization => TraceType::MemoryAccess,
                _ => TraceType::VMEntry,
            },
            vm_id: sample.vm_id,
            vcpu_id: sample.vcpu_id,
            data: match sample.metric_type {
                MetricType::VMExitRate => TraceData::VMExitReason(VmExitReason::Unknown),
                MetricType::MemoryUtilization => TraceData::MemoryAddress(0, 0),
                _ => TraceData::InstructionPointer(sample.value as u64),
            },
        };
        
        self.traces.push(trace_entry);
        Ok(())
    }
    
    /// Check for performance alerts
    fn check_alerts(&mut self, sample: &PerformanceSample) -> Result<(), HypervisorError> {
        if let Some(&threshold) = self.config.alert_thresholds.get(&sample.metric_type) {
            if sample.value > threshold {
                let alert = PerformanceAlert {
                    id: format!("alert_{}_{}", sample.metric_type as u32, self.get_current_time_ms()),
                    severity: self.determine_alert_severity(sample.value, threshold),
                    metric_type: sample.metric_type,
                    current_value: sample.value,
                    threshold_value: threshold,
                    message: format!("{} exceeded threshold: {} > {}", 
                                   self.metric_type_name(sample.metric_type), sample.value, threshold),
                    timestamp_ms: sample.timestamp_ms,
                    vm_id: sample.vm_id,
                };
                
                self.alerts.push(alert);
                warn!("Performance alert: {}", alert.message);
            }
        }
        
        Ok(())
    }
    
    /// Calculate CPU utilization
    fn calculate_cpu_utilization(&self, cpu_stat: &CpuStats, timestamp: u64) -> f64 {
        let time_diff = if cpu_stat.total_time_ms > 0 {
            timestamp - cpu_stat.total_time_ms
        } else {
            1
        };
        
        let utilization = (cpu_stat.instruction_count as f64 / time_diff as f64) * 100.0;
        utilization.min(100.0).max(0.0)
    }
    
    /// Calculate VM exit rate
    fn calculate_vm_exit_rate(&self, cpu_stat: &CpuStats, timestamp: u64) -> f64 {
        let time_diff = if cpu_stat.total_time_ms > 0 {
            timestamp - cpu_stat.total_time_ms
        } else {
            1
        };
        
        (cpu_stat.vm_exit_count as f64 / time_diff as f64) * 1000.0
    }
    
    /// Calculate instruction rate
    fn calculate_instruction_rate(&self, cpu_stat: &CpuStats, timestamp: u64) -> f64 {
        let time_diff = if cpu_stat.total_time_ms > 0 {
            timestamp - cpu_stat.total_time_ms
        } else {
            1
        };
        
        (cpu_stat.instruction_count as f64 / time_diff as f64) * 1000.0
    }
    
    /// Calculate memory utilization
    fn calculate_memory_utilization(&self, memory_stats: &MemoryStats) -> f64 {
        if memory_stats.allocated_mb > 0 {
            (memory_stats.used_mb as f64 / memory_stats.allocated_mb as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Calculate hypervisor overhead
    fn calculate_hypervisor_overhead(&self, hypervisor_stats: &HypervisorStats) -> f64 {
        // Simplified calculation - in real implementation would be more complex
        (hypervisor_stats.total_vm_exits as f64 / hypervisor_stats.vm_exit_count as f64) * 100.0
    }
    
    /// Determine alert severity
    fn determine_alert_severity(&self, value: f64, threshold: f64) -> AlertSeverity {
        let ratio = value / threshold;
        
        if ratio > 2.0 {
            AlertSeverity::Critical
        } else if ratio > 1.5 {
            AlertSeverity::Error
        } else if ratio > 1.1 {
            AlertSeverity::Warning
        } else {
            AlertSeverity::Info
        }
    }
    
    /// Get metric type name
    fn metric_type_name(&self, metric_type: MetricType) -> &'static str {
        match metric_type {
            MetricType::CPUUtilization => "CPU Utilization",
            MetricType::MemoryUtilization => "Memory Utilization",
            MetricType::VMExitRate => "VM Exit Rate",
            MetricType::InstructionRate => "Instruction Rate",
            MetricType::IORate => "I/O Rate",
            MetricType::NetworkThroughput => "Network Throughput",
            MetricType::ContextSwitchRate => "Context Switch Rate",
            MetricType::PageFaultRate => "Page Fault Rate",
            MetricType::HypervisorOverhead => "Hypervisor Overhead",
        }
    }
    
    /// Should retain sample based on retention policy
    fn should_retain_sample(&self, timestamp_ms: u64) -> bool {
        let retention_ms = (self.config.retention_period_hours as u64) * 60 * 60 * 1000;
        let current_time = self.get_current_time_ms();
        
        current_time - timestamp_ms <= retention_ms
    }
    
    /// Start profiling session
    pub fn start_profiling(&mut self, session_id: String, vm_id: VmId, profile_type: ProfileType) -> Result<(), HypervisorError> {
        if self.profiling_sessions.contains_key(&session_id) {
            return Err(HypervisorError::ConfigurationError(String::from("Profiling session already exists")));
        }
        
        let profiling_data = ProfilingData {
            vm_id,
            profile_type,
            duration_ms: 0,
            samples: Vec::new(),
            summary: ProfileSummary {
                total_samples: 0,
                average_value: 0.0,
                min_value: f64::INFINITY,
                max_value: f64::NEG_INFINITY,
                standard_deviation: 0.0,
                percentiles: BTreeMap::new(),
            },
        };
        
        self.profiling_sessions.insert(session_id, profiling_data);
        
        info!("Started profiling session '{}' for VM {} (type: {:?})", session_id, vm_id.0, profile_type);
        Ok(())
    }
    
    /// Stop profiling session
    pub fn stop_profiling(&mut self, session_id: String) -> Result<ProfilingData, HypervisorError> {
        let mut profiling_data = self.profiling_sessions.remove(&session_id)
            .ok_or(HypervisorError::ConfigurationError(format!("Profiling session '{}' not found", session_id)))?;
        
        // Calculate summary statistics
        profiling_data.summary = self.calculate_profile_summary(&profiling_data.samples);
        profiling_data.duration_ms = self.get_current_time_ms() - self.start_time_ms;
        
        info!("Stopped profiling session '{}' (duration: {} ms, samples: {})", 
              session_id, profiling_data.duration_ms, profiling_data.samples.len());
        
        Ok(profiling_data)
    }
    
    /// Calculate profile summary
    fn calculate_profile_summary(&self, samples: &[PerformanceSample]) -> ProfileSummary {
        if samples.is_empty() {
            return ProfileSummary {
                total_samples: 0,
                average_value: 0.0,
                min_value: 0.0,
                max_value: 0.0,
                standard_deviation: 0.0,
                percentiles: BTreeMap::new(),
            };
        }
        
        let total_samples = samples.len();
        let sum: f64 = samples.iter().map(|s| s.value).sum();
        let average_value = sum / total_samples as f64;
        
        let min_value = samples.iter().map(|s| s.value).fold(f64::INFINITY, f64::min);
        let max_value = samples.iter().map(|s| s.value).fold(f64::NEG_INFINITY, f64::max);
        
        let variance: f64 = samples.iter()
            .map(|s| {
                let diff = s.value - average_value;
                diff * diff
            })
            .sum::<f64>() / total_samples as f64;
        
        let standard_deviation = variance.sqrt();
        
        // Calculate percentiles
        let mut values: Vec<f64> = samples.iter().map(|s| s.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mut percentiles = BTreeMap::new();
        for &p in &[50.0, 75.0, 90.0, 95.0, 99.0] {
            let index = ((p / 100.0) * (total_samples as f64 - 1.0)).round() as usize;
            percentiles.insert(p, values[index.min(values.len() - 1)]);
        }
        
        ProfileSummary {
            total_samples,
            average_value,
            min_value,
            max_value,
            standard_deviation,
            percentiles,
        }
    }
    
    /// Get current time in milliseconds (simplified)
    fn get_current_time_ms(&self) -> u64 {
        0 // Would use actual timestamp
    }
    
    /// Get performance samples for a VM
    pub fn get_vm_samples(&self, vm_id: VmId) -> Vec<&PerformanceSample> {
        self.samples.iter()
            .filter(|s| s.vm_id == Some(vm_id))
            .collect()
    }
    
    /// Get samples by metric type
    pub fn get_samples_by_metric(&self, metric_type: MetricType) -> Vec<&PerformanceSample> {
        self.samples.iter()
            .filter(|s| s.metric_type == metric_type)
            .collect()
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&PerformanceAlert> {
        self.alerts.iter().collect()
    }
    
    /// Get recent traces
    pub fn get_recent_traces(&self, limit: usize) -> Vec<&DebugTraceEntry> {
        self.traces.iter()
            .rev()
            .take(limit)
            .collect()
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            total_samples: self.samples.len(),
            active_alerts: self.alerts.len(),
            total_traces: self.traces.len(),
            active_profiling_sessions: self.profiling_sessions.len(),
            uptime_ms: self.get_current_time_ms() - self.start_time_ms,
            sample_rate: if self.samples.len() > 0 {
                self.samples.len() as f64 / ((self.get_current_time_ms() - self.start_time_ms) as f64 / 1000.0)
            } else {
                0.0
            },
        }
    }
    
    /// Generate performance report
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Performance Monitoring Report\n");
        report.push_str("============================\n\n");
        
        let stats = self.get_performance_stats();
        report.push_str(&format!("Total samples collected: {}\n", stats.total_samples));
        report.push_str(&format!("Active alerts: {}\n", stats.active_alerts));
        report.push_str(&format!("Total traces: {}\n", stats.total_traces));
        report.push_str(&format!("Active profiling sessions: {}\n", stats.active_profiling_sessions));
        report.push_str(&format!("Monitoring uptime: {} ms\n", stats.uptime_ms));
        report.push_str(&format!("Average sample rate: {:.2} samples/second\n\n", stats.sample_rate));
        
        // Active alerts
        if !self.alerts.is_empty() {
            report.push_str("Active Alerts:\n");
            for alert in &self.alerts {
                report.push_str(&format!("  [{}] {}: {} (threshold: {})\n", 
                                      format!("{:?}", alert.severity), 
                                      alert.message, 
                                      alert.current_value, 
                                      alert.threshold_value));
            }
            report.push_str("\n");
        }
        
        // Recent performance trends
        report.push_str("Performance Trends (Last 10 samples):\n");
        let recent_samples: Vec<_> = self.samples.iter()
            .rev()
            .take(10)
            .collect();
        
        for sample in recent_samples {
            let vm_str = if let Some(vm_id) = sample.vm_id {
                format!("VM{}", vm_id.0)
            } else {
                "System".to_string()
            };
            
            report.push_str(&format!("  {} [{}]: {:.2} {}\n", 
                                  vm_str, 
                                  self.metric_type_name(sample.metric_type), 
                                  sample.value, 
                                  sample.unit));
        }
        
        // Profiling sessions
        if !self.profiling_sessions.is_empty() {
            report.push_str("\nActive Profiling Sessions:\n");
            for (session_id, profiling_data) in &self.profiling_sessions {
                report.push_str(&format!("  Session '{}': {:?} for VM {} ({} samples)\n", 
                                      session_id, profiling_data.profile_type, 
                                      profiling_data.vm_id.0, profiling_data.samples.len()));
            }
        }
        
        report
    }
    
    /// Clear old data
    pub fn clear_old_data(&mut self) -> Result<(), HypervisorError> {
        let current_time = self.get_current_time_ms();
        let retention_ms = (self.config.retention_period_hours as u64) * 60 * 60 * 1000;
        
        // Clear old samples
        self.samples.retain(|s| current_time - s.timestamp_ms <= retention_ms);
        
        // Clear old traces
        self.traces.retain(|t| current_time - (t.timestamp_ns / 1_000_000) <= retention_ms);
        
        // Clear resolved alerts
        self.alerts.retain(|a| current_time - a.timestamp_ms <= retention_ms);
        
        info!("Cleared old monitoring data");
        Ok(())
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_samples: usize,
    pub active_alerts: usize,
    pub total_traces: usize,
    pub active_profiling_sessions: usize,
    pub uptime_ms: u64,
    pub sample_rate: f64,
}