use std::time::{Duration, Instant};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Device {
    pub name: String,
    pub device_type: DeviceType,
    pub initialization_time: Duration,
    pub dependencies: Vec<String>,
    pub priority: DevicePriority,
    pub critical: bool,
    pub parallel_safe: bool,
    pub power_management: PowerManagementType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeviceType {
    CPU,
    Memory,
    Storage,
    Network,
    Graphics,
    USB,
    PCI,
    Audio,
    Input,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevicePriority {
    Critical,
    High,
    Medium,
    Low,
    Optional,
}

#[derive(Clone, Debug)]
pub enum PowerManagementType {
    ACPI,
    Legacy,
    None,
}

#[derive(Clone, Debug)]
pub struct DeviceInitializationPlan {
    pub devices: Vec<Device>,
    pub initialization_order: Vec<String>,
    pub parallel_groups: Vec<Vec<String>>,
    pub estimated_time: Duration,
    pub power_optimization: bool,
}

#[derive(Clone, Debug)]
pub struct DeviceInitMetrics {
    pub devices_initialized: usize,
    pub total_init_time: Duration,
    pub parallel_init_efficiency: f64,
    pub critical_devices_time: Duration,
    pub power_savings: Duration,
}

pub struct DeviceInitializer {
    devices: Arc<Mutex<HashMap<String, Device>>>,
    dependency_graph: Arc<Mutex<DeviceDependencyGraph>>,
    config: DeviceInitConfig,
    boot_profile: Arc<Mutex<BootProfile>>,
}

#[derive(Clone)]
pub struct DeviceInitConfig {
    pub enable_parallel_init: bool,
    pub parallel_groups: usize,
    pub prioritize_critical: bool,
    pub aggressive_power_management: bool,
    pub detect_slow_devices: bool,
    pub skip_optional_devices: bool,
}

impl Default for DeviceInitConfig {
    fn default() -> Self {
        Self {
            enable_parallel_init: true,
            parallel_groups: 6,
            prioritize_critical: true,
            aggressive_power_management: false,
            detect_slow_devices: true,
            skip_optional_devices: false,
        }
    }
}

struct DeviceDependencyGraph {
    devices: HashMap<String, Device>,
    edges: HashMap<String, HashSet<String>>, // device -> dependencies
    device_types: HashMap<String, DeviceType>,
}

impl DeviceDependencyGraph {
    fn new() -> Self {
        Self {
            devices: HashMap::new(),
            edges: HashMap::new(),
            device_types: HashMap::new(),
        }
    }

    fn add_device(&mut self, device: Device) {
        self.devices.insert(device.name.clone(), device.clone());
        self.edges.insert(device.name.clone(), device.dependencies.iter().cloned().collect());
        self.device_types.insert(device.name.clone(), device.device_type.clone());
    }

    fn get_initialization_order(&self) -> Result<Vec<String>, String> {
        // Topological sort with priority consideration
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut priority_queue: BTreeMap<DevicePriority, VecDeque<String>> = BTreeMap::new();
        let mut result = Vec::new();

        // Calculate in-degrees and categorize by priority
        for (device_name, deps) in &self.edges {
            in_degree.entry(device_name.clone()).or_insert(0);
            
            for dep in deps {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }

            // Add to priority queue if no dependencies
            if deps.is_empty() {
                if let Some(device) = self.devices.get(device_name) {
                    priority_queue
                        .entry(device.priority.clone())
                        .or_insert_with(VecDeque::new)
                        .push_back(device_name.clone());
                }
            }
        }

        // Process by priority
        while !priority_queue.is_empty() {
            let mut processed_any = false;
            
            // Process from highest to lowest priority
            for (priority, queue) in priority_queue.clone().iter_mut() {
                while let Some(device_name) = queue.pop_front() {
                    if in_degree.get(&device_name) == Some(&0) {
                        result.push(device_name.clone());
                        processed_any = true;

                        // Remove edges from this device
                        if let Some(deps) = self.edges.get(&device_name) {
                            for dep in deps {
                                if let Some(&mut ref mut degree) = in_degree.get_mut(dep) {
                                    *degree -= 1;
                                    if *degree == 0 {
                                        // Add dependent device to queue if all dependencies are met
                                        if let Some(device) = self.devices.get(dep) {
                                            priority_queue
                                                .entry(device.priority.clone())
                                                .or_insert_with(VecDeque::new)
                                                .push_back(dep.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !processed_any {
                return Err("Circular dependency detected or invalid priority ordering".to_string());
            }
        }

        if result.len() != self.devices.len() {
            return Err("Some devices could not be initialized due to circular dependencies".to_string());
        }

        Ok(result)
    }

    fn find_parallel_groups(&self, init_order: &[String]) -> Vec<Vec<String>> {
        if !init_order.is_empty() {
            let mut groups = Vec::new();
            let mut current_group = Vec::new();
            let mut processed = HashSet::new();

            for device_name in init_order {
                if processed.contains(device_name) {
                    continue;
                }

                let device = match self.devices.get(device_name) {
                    Some(d) => d,
                    None => continue,
                };

                // Check if this device can be initialized in parallel with current group
                let can_add_to_group = current_group.iter().all(|other| {
                    self.can_init_parallel(device, other)
                });

                if can_add_to_group && current_group.len() < 6 {
                    current_group.push(device_name.clone());
                    processed.insert(device_name.clone());
                } else {
                    if !current_group.is_empty() {
                        groups.push(current_group.clone());
                    }
                    current_group.clear();
                    current_group.push(device_name.clone());
                    processed.insert(device_name.clone());
                }
            }

            if !current_group.is_empty() {
                groups.push(current_group);
            }

            groups
        } else {
            Vec::new()
        }
    }

    fn can_init_parallel(&self, device1: &Device, device2: &Device) -> bool {
        // Devices can be initialized in parallel if they don't depend on each other
        let mut device1_deps = HashSet::new();
        device1_deps.extend(&device1.dependencies);
        
        let mut device2_deps = HashSet::new();
        device2_deps.extend(&device2.dependencies);

        // Check if device1 depends on device2
        if device1_deps.contains(&device2.name) {
            return false;
        }

        // Check if device2 depends on device1
        if device2_deps.contains(&device1.name) {
            return false;
        }

        // Critical devices should not be parallelized
        if device1.critical || device2.critical {
            return false;
        }

        // Some device types should not be parallelized (like CPU)
        if device1.device_type == DeviceType::CPU || device2.device_type == DeviceType::CPU {
            return false;
        }

        device1.parallel_safe && device2.parallel_safe
    }
}

struct BootProfile {
    cold_boot_times: Vec<Duration>,
    warm_boot_times: Vec<Duration>,
    device_init_patterns: HashMap<String, Vec<Duration>>,
}

impl BootProfile {
    fn new() -> Self {
        Self {
            cold_boot_times: Vec::new(),
            warm_boot_times: Vec::new(),
            device_init_patterns: HashMap::new(),
        }
    }

    fn add_cold_boot_time(&mut self, time: Duration) {
        self.cold_boot_times.push(time);
        if self.cold_boot_times.len() > 100 {
            self.cold_boot_times.remove(0);
        }
    }

    fn add_warm_boot_time(&mut self, time: Duration) {
        self.warm_boot_times.push(time);
        if self.warm_boot_times.len() > 100 {
            self.warm_boot_times.remove(0);
        }
    }

    fn add_device_init_time(&mut self, device_name: String, time: Duration) {
        self.device_init_patterns
            .entry(device_name)
            .or_insert_with(Vec::new)
            .push(time);
        
        // Keep only recent measurements
        if let Some(times) = self.device_init_patterns.get_mut(&device_name) {
            if times.len() > 50 {
                times.remove(0);
            }
        }
    }

    fn get_average_cold_boot_time(&self) -> Option<Duration> {
        if self.cold_boot_times.is_empty() {
            None
        } else {
            Some(self.cold_boot_times.iter().sum::<Duration>() / self.cold_boot_times.len() as u32)
        }
    }

    fn get_average_warm_boot_time(&self) -> Option<Duration> {
        if self.warm_boot_times.is_empty() {
            None
        } else {
            Some(self.warm_boot_times.iter().sum::<Duration>() / self.warm_boot_times.len() as u32)
        }
    }

    fn get_device_init_average(&self, device_name: &str) -> Option<Duration> {
        if let Some(times) = self.device_init_patterns.get(device_name) {
            if times.is_empty() {
                None
            } else {
                Some(times.iter().sum::<Duration>() / times.len() as u32)
            }
        } else {
            None
        }
    }
}

impl DeviceInitializer {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            dependency_graph: Arc::new(Mutex::new(DeviceDependencyGraph::new())),
            config: DeviceInitConfig::default(),
            boot_profile: Arc::new(Mutex::new(BootProfile::new())),
        }
    }

    pub fn configure(&mut self, config: DeviceInitConfig) {
        self.config = config;
    }

    pub fn add_device(&self, device: Device) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.name.clone(), device.clone());
        
        let mut graph = self.dependency_graph.lock().unwrap();
        graph.add_device(device);
    }

    pub fn create_initialization_plan(&self) -> Result<DeviceInitializationPlan, String> {
        let graph = self.dependency_graph.lock().unwrap();
        let init_order = graph.get_initialization_order()?;
        
        let mut parallel_groups = Vec::new();
        if self.config.enable_parallel_init {
            parallel_groups = graph.find_parallel_groups(&init_order);
        }

        // Calculate estimated time
        let estimated_time = self.calculate_estimated_time(&init_order, &parallel_groups);

        let devices: Vec<Device> = graph.devices.values().cloned().collect();

        Ok(DeviceInitializationPlan {
            devices,
            initialization_order: init_order,
            parallel_groups,
            estimated_time,
            power_optimization: self.config.aggressive_power_management,
        })
    }

    pub fn simulate_initialization(&self, plan: &DeviceInitializationPlan) -> DeviceInitMetrics {
        let start_time = Instant::now();
        let mut devices_initialized = 0;
        let mut critical_devices_time = Duration::from_millis(0);
        let mut power_savings = Duration::from_millis(0);

        // Simulate initialization
        if self.config.enable_parallel_init && !plan.parallel_groups.is_empty() {
            for group in &plan.parallel_groups {
                let group_start = Instant::now();
                
                // Simulate parallel device initialization
                for device_name in group {
                    if let Some(device) = self.devices.lock().unwrap().get(device_name) {
                        // Simulate device initialization time
                        let init_time = self.get_realistic_init_time(device);
                        std::thread::sleep(init_time.min(Duration::from_millis(5))); // Reduced for simulation
                        
                        devices_initialized += 1;
                        
                        if device.critical {
                            critical_devices_time += init_time;
                        }
                        
                        if device.power_management != PowerManagementType::None {
                            power_savings += Duration::from_millis(2);
                        }
                        
                        // Add to boot profile
                        let mut profile = self.boot_profile.lock().unwrap();
                        profile.add_device_init_time(device_name.clone(), init_time);
                    }
                }
                
                let group_time = group_start.elapsed();
                println!("Device group initialized in {:?}", group_time);
            }
        } else {
            // Simulate sequential initialization
            for device_name in &plan.initialization_order {
                if let Some(device) = self.devices.lock().unwrap().get(device_name) {
                    let init_time = self.get_realistic_init_time(device);
                    std::thread::sleep(init_time.min(Duration::from_millis(5))); // Reduced for simulation
                    
                    devices_initialized += 1;
                    
                    if device.critical {
                        critical_devices_time += init_time;
                    }
                    
                    if device.power_management != PowerManagementType::None {
                        power_savings += Duration::from_millis(2);
                    }
                    
                    // Add to boot profile
                    let mut profile = self.boot_profile.lock().unwrap();
                    profile.add_device_init_time(device_name.clone(), init_time);
                }
            }
        }

        let total_time = start_time.elapsed();
        let parallel_efficiency = if self.config.enable_parallel_init && !plan.parallel_groups.is_empty() {
            // Calculate efficiency based on theoretical serial time
            let serial_time: Duration = plan.initialization_order.iter()
                .filter_map(|name| self.devices.lock().unwrap().get(name))
                .map(|d| self.get_realistic_init_time(d))
                .sum();
            
            if serial_time > Duration::from_millis(0) {
                serial_time.as_millis() as f64 / total_time.as_millis() as f64
            } else {
                0.0
            }
        } else {
            1.0
        };

        DeviceInitMetrics {
            devices_initialized,
            total_init_time: total_time,
            parallel_init_efficiency: parallel_efficiency,
            critical_devices_time,
            power_savings,
        }
    }

    fn get_realistic_init_time(&self, device: &Device) -> Duration {
        // Use boot profile data if available
        let profile = self.boot_profile.lock().unwrap();
        if let Some(avg_time) = profile.get_device_init_average(&device.name) {
            avg_time
        } else {
            device.initialization_time
        }
    }

    fn calculate_estimated_time(&self, init_order: &[String], parallel_groups: &[Vec<String>]) -> Duration {
        let devices = self.devices.lock().unwrap();
        
        if self.config.enable_parallel_init && !parallel_groups.is_empty() {
            // Calculate time with parallelization
            let mut total_time = Duration::from_millis(0);
            
            for group in parallel_groups {
                let group_time: Duration = group.iter()
                    .filter_map(|name| devices.get(name))
                    .map(|d| self.get_realistic_init_time(d))
                    .max()
                    .unwrap_or_default();
                
                total_time += group_time;
            }
            
            total_time
        } else {
            // Calculate time without parallelization
            init_order.iter()
                .filter_map(|name| devices.get(name))
                .map(|d| self.get_realistic_init_time(d))
                .sum()
        }
    }

    pub fn record_cold_boot(&self, time: Duration) {
        let mut profile = self.boot_profile.lock().unwrap();
        profile.add_cold_boot_time(time);
    }

    pub fn record_warm_boot(&self, time: Duration) {
        let mut profile = self.boot_profile.lock().unwrap();
        profile.add_warm_boot_time(time);
    }

    pub fn get_boot_analysis(&self) -> BootAnalysisReport {
        let profile = self.boot_profile.lock().unwrap();
        
        let mut report = BootAnalysisReport::new();
        
        if let Some(cold_time) = profile.get_average_cold_boot_time() {
            report.average_cold_boot_time = cold_time;
        }
        
        if let Some(warm_time) = profile.get_average_warm_boot_time() {
            report.average_warm_boot_time = warm_time;
        }
        
        if let (Some(cold), Some(warm)) = (profile.get_average_cold_boot_time(), profile.get_average_warm_boot_time()) {
            report.warm_boot_improvement = cold - warm;
            report.warm_boot_improvement_percentage = if cold > Duration::from_millis(0) {
                (warm.as_millis() as f64 / cold.as_millis() as f64) * 100.0
            } else {
                0.0
            };
        }
        
        // Analyze device initialization patterns
        for (device_name, times) in &profile.device_init_patterns {
            if times.len() >= 5 {
                let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
                let variance = self.calculate_variance(times);
                
                if avg_time > Duration::from_millis(100) || variance > Duration::from_millis(50) {
                    report.slow_or_inconsistent_devices.insert(device_name.clone(), DeviceAnalysis {
                        average_time: avg_time,
                        variance,
                        measurements_count: times.len(),
                    });
                }
            }
        }
        
        report
    }

    fn calculate_variance(&self, times: &[Duration]) -> Duration {
        if times.len() < 2 {
            return Duration::from_millis(0);
        }
        
        let mean = times.iter().sum::<Duration>() / times.len() as u32;
        let variance_sum: Duration = times.iter()
            .map(|&t| {
                let diff = if t > mean { t - mean } else { mean - t };
                diff * diff
            })
            .sum();
        
        variance_sum / (times.len() - 1) as u32
    }

    pub fn get_optimization_suggestions(&self) -> Vec<DeviceOptimizationSuggestion> {
        let mut suggestions = Vec::new();
        
        let devices = self.devices.lock().unwrap();
        let graph = self.dependency_graph.lock().unwrap();
        
        // Check for slow critical devices
        for device in devices.values().filter(|d| d.critical) {
            if device.initialization_time > Duration::from_millis(200) {
                suggestions.push(DeviceOptimizationSuggestion {
                    device_name: device.name.clone(),
                    suggestion: format!("Critical device '{}' has slow initialization. Consider optimization or parallel initialization", device.name),
                    expected_improvement: Duration::from_millis(50),
                    optimization_type: "Speed".to_string(),
                });
            }
        }
        
        // Check for devices that could be parallelized
        let parallel_safe_count = devices.values().filter(|d| d.parallel_safe && !d.critical).count();
        if parallel_safe_count > 0 && !self.config.enable_parallel_init {
            suggestions.push(DeviceOptimizationSuggestion {
                device_name: "parallel_init".to_string(),
                suggestion: format!("Enable parallel initialization for {} safe devices", parallel_safe_count),
                expected_improvement: Duration::from_millis(200),
                optimization_type: "Parallelization".to_string(),
            });
        }
        
        // Check for power management opportunities
        let power_management_count = devices.values()
            .filter(|d| d.power_management == PowerManagementType::None)
            .count();
        if power_management_count > 0 && self.config.aggressive_power_management {
            suggestions.push(DeviceOptimizationSuggestion {
                device_name: "power_management".to_string(),
                suggestion: format!("Enable power management for {} devices", power_management_count),
                expected_improvement: Duration::from_millis(30),
                optimization_type: "Power".to_string(),
            });
        }
        
        suggestions
    }
}

pub struct BootAnalysisReport {
    pub average_cold_boot_time: Duration,
    pub average_warm_boot_time: Duration,
    pub warm_boot_improvement: Duration,
    pub warm_boot_improvement_percentage: f64,
    pub slow_or_inconsistent_devices: HashMap<String, DeviceAnalysis>,
}

#[derive(Clone, Debug)]
pub struct DeviceAnalysis {
    pub average_time: Duration,
    pub variance: Duration,
    pub measurements_count: usize,
}

impl BootAnalysisReport {
    fn new() -> Self {
        Self {
            average_cold_boot_time: Duration::from_millis(0),
            average_warm_boot_time: Duration::from_millis(0),
            warm_boot_improvement: Duration::from_millis(0),
            warm_boot_improvement_percentage: 0.0,
            slow_or_inconsistent_devices: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceOptimizationSuggestion {
    pub device_name: String,
    pub suggestion: String,
    pub expected_improvement: Duration,
    pub optimization_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_initializer_creation() {
        let initializer = DeviceInitializer::new();
        let plan = initializer.create_initialization_plan().unwrap();
        assert_eq!(plan.devices.len(), 0);
    }

    #[test]
    fn test_device_addition() {
        let initializer = DeviceInitializer::new();
        
        let device = Device {
            name: "cpu".to_string(),
            device_type: DeviceType::CPU,
            initialization_time: Duration::from_millis(100),
            dependencies: vec![],
            priority: DevicePriority::Critical,
            critical: true,
            parallel_safe: false,
            power_management: PowerManagementType::ACPI,
        };
        
        initializer.add_device(device);
        
        let plan = initializer.create_initialization_plan().unwrap();
        assert_eq!(plan.devices.len(), 1);
        assert_eq!(plan.initialization_order[0], "cpu");
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DeviceDependencyGraph::new();
        
        let cpu = Device {
            name: "cpu".to_string(),
            device_type: DeviceType::CPU,
            initialization_time: Duration::from_millis(100),
            dependencies: vec![],
            priority: DevicePriority::Critical,
            critical: true,
            parallel_safe: false,
            power_management: PowerManagementType::ACPI,
        };
        
        let memory = Device {
            name: "memory".to_string(),
            device_type: DeviceType::Memory,
            initialization_time: Duration::from_millis(50),
            dependencies: vec!["cpu".to_string()],
            priority: DevicePriority::Critical,
            critical: true,
            parallel_safe: false,
            power_management: PowerManagementType::ACPI,
        };
        
        graph.add_device(cpu);
        graph.add_device(memory);
        
        let init_order = graph.get_initialization_order().unwrap();
        assert_eq!(init_order[0], "cpu");
        assert_eq!(init_order[1], "memory");
    }
}
