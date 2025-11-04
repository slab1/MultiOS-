use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
pub struct BootTimestamp {
    pub phase: String,
    pub timestamp: u64,
    pub duration: Option<Duration>,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct BootPhaseMetrics {
    pub name: String,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub duration: Option<Duration>,
    pub cpu_cycles: Option<u64>,
    pub memory_accesses: Option<u64>,
    pub parallel_execution: bool,
    pub dependencies: Vec<String>,
}

#[derive(Clone)]
pub struct BootMeasurementConfig {
    pub enable_detailed_profiling: bool,
    pub enable_cpu_monitoring: bool,
    pub enable_memory_monitoring: bool,
    pub profiling_interval: Duration,
}

impl Default for BootMeasurementConfig {
    fn default() -> Self {
        Self {
            enable_detailed_profiling: true,
            enable_cpu_monitoring: true,
            enable_memory_monitoring: true,
            profiling_interval: Duration::from_micros(100),
        }
    }
}

pub struct BootMeasurement {
    config: BootMeasurementConfig,
    phases: Arc<Mutex<Vec<BootPhaseMetrics>>>,
    measurements: Arc<RwLock<Vec<BootTimestamp>>>,
    global_start: Arc<Mutex<Option<Instant>>>,
    current_phase: Arc<Mutex<Option<String>>>,
}

impl BootMeasurement {
    pub fn new(config: BootMeasurementConfig) -> Self {
        Self {
            config,
            phases: Arc::new(Mutex::new(Vec::new())),
            measurements: Arc::new(RwLock::new(Vec::new())),
            global_start: Arc::new(Mutex::new(None)),
            current_phase: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_boot(&self) {
        let mut start = self.global_start.lock().unwrap();
        *start = Some(Instant::now());
    }

    pub fn start_phase(&self, name: &str, dependencies: Vec<String>, parallel: bool) -> BootPhaseHandle {
        // Create the phase metrics
        let phase = BootPhaseMetrics {
            name: name.to_string(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            cpu_cycles: None,
            memory_accesses: None,
            parallel_execution: parallel,
            dependencies,
        };

        // Add to phases list
        let mut phases = self.phases.lock().unwrap();
        phases.push(phase.clone());

        // Update current phase
        let mut current = self.current_phase.lock().unwrap();
        *current = Some(name.to_string());

        // Add timestamp entry
        let timestamp = BootTimestamp {
            phase: name.to_string(),
            timestamp: self.get_timestamp(),
            duration: None,
            cpu_usage: None,
            memory_usage: None,
        };

        let mut measurements = self.measurements.write().unwrap();
        measurements.push(timestamp);

        BootPhaseHandle {
            name: name.to_string(),
            start_time: phase.start_time,
            phases: self.phases.clone(),
        }
    }

    pub fn end_phase(&self, handle: BootPhaseHandle) {
        let end_time = Instant::now();
        
        // Find and update the phase
        let mut phases = self.phases.lock().unwrap();
        for phase in phases.iter_mut() {
            if phase.name == handle.name {
                phase.end_time = Some(end_time);
                phase.duration = Some(end_time.duration_since(handle.start_time));
                break;
            }
        }

        // Add end timestamp
        let timestamp = BootTimestamp {
            phase: format!("{}_end", handle.name),
            timestamp: self.get_timestamp(),
            duration: None,
            cpu_usage: None,
            memory_usage: None,
        };

        let mut measurements = self.measurements.write().unwrap();
        measurements.push(timestamp);

        // Clear current phase if this was the current one
        let mut current = self.current_phase.lock().unwrap();
        if *current == Some(handle.name) {
            *current = None;
        }
    }

    pub fn get_current_phase(&self) -> Option<String> {
        self.current_phase.lock().unwrap().clone()
    }

    pub fn get_total_boot_time(&self) -> Option<Duration> {
        let start = self.global_start.lock().unwrap();
        if let Some(start_time) = *start {
            Some(start_time.elapsed())
        } else {
            None
        }
    }

    pub fn get_phase_metrics(&self) -> Vec<BootPhaseMetrics> {
        self.phases.lock().unwrap().clone()
    }

    pub fn get_measurements(&self) -> Vec<BootTimestamp> {
        self.measurements.read().unwrap().clone()
    }

    pub fn generate_report(&self) -> BootMeasurementReport {
        let phases = self.get_phase_metrics();
        let measurements = self.get_measurements();
        let total_time = self.get_total_boot_time();

        let mut report = BootMeasurementReport::new();
        
        for phase in phases {
            report.add_phase_data(BootPhaseData {
                name: phase.name,
                duration: phase.duration.unwrap_or_default(),
                parallel: phase.parallel_execution,
                dependencies: phase.dependencies,
            });
        }

        if let Some(total) = total_time {
            report.total_duration = total;
        }

        report.measurements = measurements;
        report
    }

    fn get_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
}

pub struct BootPhaseHandle {
    name: String,
    start_time: Instant,
    phases: Arc<Mutex<Vec<BootPhaseMetrics>>>,
}

impl Drop for BootPhaseHandle {
    fn drop(&mut self) {
        let end_time = Instant::now();
        let mut phases = self.phases.lock().unwrap();
        
        for phase in phases.iter_mut() {
            if phase.name == self.name {
                phase.end_time = Some(end_time);
                phase.duration = Some(end_time.duration_since(self.start_time));
                break;
            }
        }
    }
}

pub struct BootMeasurementReport {
    total_duration: Duration,
    phase_data: Vec<BootPhaseData>,
    measurements: Vec<BootTimestamp>,
}

#[derive(Clone)]
pub struct BootPhaseData {
    pub name: String,
    pub duration: Duration,
    pub parallel: bool,
    pub dependencies: Vec<String>,
}

impl BootMeasurementReport {
    fn new() -> Self {
        Self {
            total_duration: Duration::from_millis(0),
            phase_data: Vec::new(),
            measurements: Vec::new(),
        }
    }

    fn add_phase_data(&mut self, data: BootPhaseData) {
        self.phase_data.push(data);
    }

    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn get_phase_data(&self) -> &[BootPhaseData] {
        &self.phase_data
    }

    pub fn get_slowest_phases(&self, limit: usize) -> Vec<&BootPhaseData> {
        let mut sorted = self.phase_data.clone();
        sorted.sort_by(|a, b| b.duration.cmp(&a.duration));
        sorted.into_iter().take(limit).collect()
    }

    pub fn get_parallel_phases(&self) -> Vec<&BootPhaseData> {
        self.phase_data.iter().filter(|p| p.parallel).collect()
    }

    pub fn get_serial_phases(&self) -> Vec<&BootPhaseData> {
        self.phase_data.iter().filter(|p| !p.parallel).collect()
    }

    pub fn calculate_parallelization_efficiency(&self) -> f64 {
        let total_serial_time: Duration = self.get_serial_phases()
            .iter()
            .map(|p| p.duration)
            .sum();
        
        let total_parallel_time: Duration = self.get_parallel_phases()
            .iter()
            .map(|p| p.duration)
            .sum();
        
        let serial_time_ms = total_serial_time.as_millis() as f64;
        let parallel_time_ms = total_parallel_time.as_millis() as f64;
        
        if serial_time_ms > 0.0 {
            serial_time_ms / (serial_time_ms + parallel_time_ms).max(1.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_measurement() {
        let config = BootMeasurementConfig::default();
        let measurement = BootMeasurement::new(config);
        
        measurement.start_boot();
        
        let handle = measurement.start_phase("test_phase", vec![], false);
        
        std::thread::sleep(Duration::from_millis(10));
        
        drop(handle);
        
        let report = measurement.generate_report();
        assert!(report.get_total_duration() >= Duration::from_millis(10));
        assert_eq!(report.get_phase_data().len(), 1);
    }

    #[test]
    fn test_measurement_config() {
        let config = BootMeasurementConfig {
            enable_detailed_profiling: false,
            enable_cpu_monitoring: false,
            enable_memory_monitoring: false,
            profiling_interval: Duration::from_millis(1),
        };
        
        assert_eq!(config.enable_detailed_profiling, false);
    }
}
