//! Edge Computing Configuration and Utilities
//! MultiOS Edge Computing Demonstrations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Edge computing device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub cpu_cores: usize,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub gpu_available: bool,
    pub tensor_acceleration: bool,
    pub network_bandwidth_mbps: u32,
    pub power_consumption_watts: f32,
    pub battery_level: Option<f32>,
}

/// Edge workload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    AiInference,
    VideoProcessing,
    SensorDataProcessing,
    PredictiveMaintenance,
    SmartCityMonitoring,
    FogGateway,
    LocalOrchestration,
}

/// Edge device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDevice {
    pub device_id: String,
    pub device_type: String,
    pub capabilities: DeviceCapabilities,
    pub current_load: f32,
    pub status: DeviceStatus,
    pub last_heartbeat: std::time::SystemTime,
    pub location: Option<(f32, f32)>, // latitude, longitude
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Busy,
    Sleeping,
    Error(String),
}

/// Edge computing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    pub cluster_id: String,
    pub cloud_endpoint: String,
    pub device_registry: HashMap<String, EdgeDevice>,
    pub workload_distribution: HashMap<WorkloadType, f32>,
    pub performance_threshold: PerformanceThreshold,
    pub security_config: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThreshold {
    pub max_cpu_usage: f32,
    pub max_memory_usage: f32,
    pub max_response_time_ms: u32,
    pub max_latency_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_encryption: bool,
    pub certificate_path: Option<String>,
    pub authentication_method: String,
}

/// Performance metrics for edge devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub network_latency_ms: f32,
    pub inference_time_ms: f32,
    pub throughput_ops_per_sec: f32,
    pub power_efficiency_ops_per_watt: f32,
    pub temperature_celsius: Option<f32>,
}

/// Edge computing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTask {
    pub task_id: String,
    pub workload_type: WorkloadType,
    pub priority: u8,
    pub estimated_duration_ms: u32,
    pub resource_requirements: ResourceRequirements,
    pub data_size_mb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_cpu_cores: usize,
    pub min_memory_mb: u64,
    pub requires_gpu: bool,
    pub requires_tensor_accel: bool,
}

/// Edge computing orchestrator
pub struct EdgeOrchestrator {
    config: Arc<RwLock<EdgeConfig>>,
    device_registry: Arc<RwLock<HashMap<String, EdgeDevice>>>,
    task_queue: Arc<RwLock<Vec<EdgeTask>>>,
    metrics_collector: Arc<RwLock<HashMap<String, PerformanceMetrics>>>,
}

impl EdgeOrchestrator {
    pub fn new(config: EdgeConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            device_registry: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            metrics_collector: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new edge device
    pub async fn register_device(&self, device: EdgeDevice) -> Result<(), Box<dyn std::error::Error>> {
        let mut registry = self.device_registry.write().await;
        registry.insert(device.device_id.clone(), device);
        println!("Registered edge device: {}", device.device_id);
        Ok(())
    }

    /// Submit a task for processing
    pub async fn submit_task(&self, task: EdgeTask) -> Result<(), Box<dyn std::error::Error>> {
        let mut queue = self.task_queue.write().await;
        queue.push(task);
        queue.sort_by(|a, b| b.priority.cmp(&a.priority)); // Sort by priority descending
        println!("Submitted task: {} with priority {}", task.task_id, task.priority);
        Ok(())
    }

    /// Get the best available device for a task
    pub async fn select_best_device(&self, task: &EdgeTask) -> Option<String> {
        let registry = self.device_registry.read().await;
        let metrics = self.metrics_collector.read().await;
        
        let mut best_device = None;
        let mut best_score = f32::MIN;

        for (device_id, device) in registry.iter() {
            if device.status != DeviceStatus::Online {
                continue;
            }

            // Calculate suitability score
            let score = self.calculate_device_suitability(device, &metrics.get(device_id), task);
            
            if score > best_score {
                best_score = score;
                best_device = Some(device_id.clone());
            }
        }

        best_device
    }

    /// Calculate device suitability score for a task
    fn calculate_device_suitability(
        &self,
        device: &EdgeDevice,
        metrics: Option<&PerformanceMetrics>,
        task: &EdgeTask,
    ) -> f32 {
        let mut score = 100.0;

        // CPU availability
        let available_cores = device.capabilities.cpu_cores as f32 * (1.0 - device.current_load);
        if available_cores < task.resource_requirements.min_cpu_cores as f32 {
            return 0.0; // Not suitable
        }
        score += (available_cores / task.resource_requirements.min_cpu_cores as f32) * 10.0;

        // Memory availability
        let available_memory = device.capabilities.memory_mb as f32 * 0.8; // Reserve 20%
        if available_memory < task.resource_requirements.min_memory_mb as f32 {
            return 0.0; // Not suitable
        }
        score += (available_memory / task.resource_requirements.min_memory_mb as f32) * 5.0;

        // GPU availability for AI tasks
        if task.workload_type == WorkloadType::AiInference && task.resource_requirements.requires_gpu {
            if device.capabilities.gpu_available {
                score += 20.0;
            } else {
                score -= 30.0;
            }
        }

        // Tensor acceleration bonus
        if task.resource_requirements.requires_tensor_accel && device.capabilities.tensor_acceleration {
            score += 15.0;
        }

        // Performance penalty for busy devices
        if let Some(metrics) = metrics {
            score -= metrics.cpu_usage * 50.0; // Penalty for high CPU usage
            score -= metrics.memory_usage * 30.0; // Penalty for high memory usage
        }

        score
    }

    /// Update device metrics
    pub async fn update_metrics(&self, device_id: String, metrics: PerformanceMetrics) {
        let mut metrics_collector = self.metrics_collector.write().await;
        metrics_collector.insert(device_id, metrics);
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> HashMap<String, PerformanceMetrics> {
        let metrics = self.metrics_collector.read().await;
        metrics.clone()
    }
}

/// Create default edge computing configuration
pub fn create_default_config() -> EdgeConfig {
    let mut workload_distribution = HashMap::new();
    workload_distribution.insert(WorkloadType::AiInference, 0.3);
    workload_distribution.insert(WorkloadType::VideoProcessing, 0.25);
    workload_distribution.insert(WorkloadType::SensorDataProcessing, 0.2);
    workload_distribution.insert(WorkloadType::PredictiveMaintenance, 0.1);
    workload_distribution.insert(WorkloadType::SmartCityMonitoring, 0.1);
    workload_distribution.insert(WorkloadType::FogGateway, 0.05);

    EdgeConfig {
        cluster_id: "multios-edge-cluster".to_string(),
        cloud_endpoint: "https://cloud.multios.com/api/v1".to_string(),
        device_registry: HashMap::new(),
        workload_distribution,
        performance_threshold: PerformanceThreshold {
            max_cpu_usage: 80.0,
            max_memory_usage: 85.0,
            max_response_time_ms: 100,
            max_latency_ms: 50,
        },
        security_config: SecurityConfig {
            enable_encryption: true,
            certificate_path: None,
            authentication_method: "jwt".to_string(),
        },
    }
}

/// Create a sample edge device
pub fn create_sample_device(device_id: &str, device_type: &str) -> EdgeDevice {
    EdgeDevice {
        device_id: device_id.to_string(),
        device_type: device_type.to_string(),
        capabilities: DeviceCapabilities {
            cpu_cores: 4,
            memory_mb: 4096,
            storage_gb: 64,
            gpu_available: true,
            tensor_acceleration: true,
            network_bandwidth_mbps: 100,
            power_consumption_watts: 15.0,
            battery_level: Some(85.0),
        },
        current_load: 0.25,
        status: DeviceStatus::Online,
        last_heartbeat: std::time::SystemTime::now(),
        location: Some((37.7749, -122.4194)), // San Francisco
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_edge_orchestrator_creation() {
        let config = create_default_config();
        let orchestrator = EdgeOrchestrator::new(config);
        assert!(orchestrator.config.read().await.cluster_id.contains("multios-edge-cluster"));
    }

    #[tokio::test]
    async fn test_device_registration() {
        let config = create_default_config();
        let orchestrator = EdgeOrchestrator::new(config);
        
        let device = create_sample_device("device-001", "edge-gateway");
        let result = orchestrator.register_device(device).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = create_default_config();
        let orchestrator = EdgeOrchestrator::new(config);
        
        let task = EdgeTask {
            task_id: "task-001".to_string(),
            workload_type: WorkloadType::AiInference,
            priority: 10,
            estimated_duration_ms: 5000,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 2,
                min_memory_mb: 1024,
                requires_gpu: true,
                requires_tensor_accel: true,
            },
            data_size_mb: 100.0,
        };
        
        let result = orchestrator.submit_task(task).await;
        assert!(result.is_ok());
    }
}