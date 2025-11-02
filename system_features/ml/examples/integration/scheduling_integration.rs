// Educational ML Framework - MultiOS Scheduling Integration Example
// Demonstrates how to use the ML framework with MultiOS job scheduling

use multi_os_ml::neural_net::models::SimpleNN;
use multi_os_ml::neural_net::layers::{DenseLayer, Conv2DLayer, MaxPool2DLayer};
use multi_os_ml::neural_net::utils::ActivationFunction;
use multi_os_ml::parallel_training::ParallelTrainer;
use multi_os_ml::data_pipeline::DataPipeline;
use multi_os_ml::runtime::tensor::Tensor;
use std::collections::HashMap;
use std::time::Duration;

/// MultiOS Scheduling Configuration
#[derive(Clone)]
pub struct MultiOSSchedulingConfig {
    pub num_workers: usize,                    // Number of parallel workers
    pub scheduler_type: SchedulerType,         // Type of scheduler to use
    pub resource_requirements: ResourceSpec,   // CPU/Memory requirements
    pub priority_level: PriorityLevel,         // Job priority
    pub max_execution_time: Duration,          // Maximum execution time
    pub auto_scaling: bool,                    // Enable automatic scaling
    pub checkpoint_frequency: Duration,        // How often to save checkpoints
    pub worker_health_check: bool,             // Monitor worker health
}

/// Scheduler types available in MultiOS
#[derive(Clone, Debug)]
pub enum SchedulerType {
    FIFO,                    // First In, First Out
    Priority,               // Priority-based scheduling
    RoundRobin,             // Round-robin distribution
    ResourceAware,          // Resource-aware scheduling
    GPUAware,               // GPU-aware scheduling
    LoadBalanced,           // Dynamic load balancing
}

/// Resource specifications for job scheduling
#[derive(Clone)]
pub struct ResourceSpec {
    pub cpu_cores: usize,              // Number of CPU cores needed
    pub memory_gb: f64,                // Memory requirement in GB
    pub gpu_required: bool,            // Whether GPU is needed
    pub gpu_memory_gb: Option<f64>,    // GPU memory requirement
    pub storage_gb: f64,               // Storage requirement
    pub network_bandwidth_mbps: Option<u64>,  // Network bandwidth needed
}

/// Job priority levels
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Job status tracking
#[derive(Clone, Debug)]
pub enum JobStatus {
    Pending,
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Scheduled ML job
pub struct ScheduledMLJob {
    pub job_id: String,
    pub job_type: MLJobType,
    pub status: JobStatus,
    pub submitted_at: std::time::Instant,
    pub started_at: Option<std::time::Instant>,
    pub estimated_duration: Duration,
    pub worker_assignment: Option<String>,
    pub progress: f64,  // 0.0 to 1.0
    pub resource_usage: ResourceUsage,
}

/// Types of ML jobs that can be scheduled
#[derive(Clone, Debug)]
pub enum MLJobType {
    Training {
        model_config: ModelConfig,
        dataset_config: DatasetConfig,
        epochs: usize,
    },
    Evaluation {
        model_path: String,
        test_dataset: String,
        metrics: Vec<String>,
    },
    HyperparameterSearch {
        search_space: HashMap<String, Vec<String>>,
        objective_metric: String,
        max_trials: usize,
    },
    ModelInference {
        model_path: String,
        input_data: String,
        batch_size: usize,
    },
    DataPreprocessing {
        dataset_path: String,
        preprocessing_steps: Vec<String>,
        output_path: String,
    },
}

/// Model configuration for scheduled jobs
#[derive(Clone)]
pub struct ModelConfig {
    pub architecture: ModelArchitecture,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub hyperparameters: HashMap<String, String>,
}

/// Dataset configuration for scheduled jobs
#[derive(Clone)]
pub struct DatasetConfig {
    pub dataset_type: String,
    pub data_path: String,
    pub validation_split: f64,
    pub batch_size: usize,
    pub preprocessing_steps: Vec<String>,
}

/// Model architecture types
#[derive(Clone, Debug)]
pub enum ModelArchitecture {
    SimpleNN { hidden_layers: Vec<usize> },
    CNN { conv_layers: Vec<ConvLayerConfig> },
    RNN { hidden_size: usize, num_layers: usize },
    Transformer { num_heads: usize, num_layers: usize },
}

/// Convolution layer configuration
#[derive(Clone)]
pub struct ConvLayerConfig {
    pub filters: usize,
    pub kernel_size: (usize, usize),
    pub activation: ActivationFunction,
}

/// Resource usage tracking
#[derive(Clone)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub gpu_usage_percent: Option<f64>,
    pub gpu_memory_mb: Option<f64>,
    pub disk_io_mb_per_sec: f64,
    pub network_io_mb_per_sec: f64,
}

/// MultiOS Job Scheduler for ML workloads
pub struct MultiOSJobScheduler {
    config: MultiOSSchedulingConfig,
    job_queue: Vec<ScheduledMLJob>,
    active_jobs: HashMap<String, ScheduledMLJob>,
    worker_pool: WorkerPool,
    job_history: Vec<ScheduledMLJob>,
}

/// Worker pool for distributed execution
pub struct WorkerPool {
    workers: Vec<Worker>,
    available_workers: Vec<usize>,
    worker_health_status: HashMap<usize, WorkerHealth>,
}

/// Individual worker in the pool
pub struct Worker {
    pub worker_id: usize,
    pub cpu_cores: usize,
    pub memory_gb: f64,
    pub gpu_available: bool,
    pub gpu_memory_gb: Option<f64>,
    pub current_load: f64,
    pub status: WorkerStatus,
}

/// Worker health monitoring
pub struct WorkerHealth {
    pub last_heartbeat: std::time::Instant,
    pub cpu_temperature: Option<f64>,
    pub memory_pressure: f64,
    pub error_count: usize,
    pub jobs_completed: usize,
}

/// Worker status
#[derive(Clone, Debug)]
pub enum WorkerStatus {
    Available,
    Busy { current_job: String, progress: f64 },
    Offline,
    Maintenance,
}

impl MultiOSSchedulingConfig {
    pub fn new(num_workers: usize) -> Self {
        Self {
            num_workers,
            scheduler_type: SchedulerType::ResourceAware,
            resource_requirements: ResourceSpec::default(),
            priority_level: PriorityLevel::Normal,
            max_execution_time: Duration::from_secs(3600), // 1 hour
            auto_scaling: true,
            checkpoint_frequency: Duration::from_secs(300), // 5 minutes
            worker_health_check: true,
        }
    }
    
    /// Configure scheduler type
    pub fn scheduler_type(mut self, scheduler_type: SchedulerType) -> Self {
        self.scheduler_type = scheduler_type;
        self
    }
    
    /// Set resource requirements
    pub fn resource_requirements(mut self, requirements: ResourceSpec) -> Self {
        self.resource_requirements = requirements;
        self
    }
    
    /// Set job priority
    pub fn priority(mut self, priority: PriorityLevel) -> Self {
        self.priority_level = priority;
        self
    }
    
    /// Set maximum execution time
    pub fn max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = duration;
        self
    }
    
    /// Enable/disable auto-scaling
    pub fn auto_scaling(mut self, enabled: bool) -> Self {
        self.auto_scaling = enabled;
        self
    }
    
    /// Set checkpoint frequency
    pub fn checkpoint_frequency(mut self, frequency: Duration) -> Self {
        self.checkpoint_frequency = frequency;
        self
    }
}

impl ResourceSpec {
    pub fn default() -> Self {
        Self {
            cpu_cores: 4,
            memory_gb: 8.0,
            gpu_required: false,
            gpu_memory_gb: None,
            storage_gb: 10.0,
            network_bandwidth_mbps: None,
        }
    }
    
    /// Create resource spec for CPU-intensive training
    pub fn cpu_intensive() -> Self {
        Self {
            cpu_cores: 16,
            memory_gb: 32.0,
            gpu_required: false,
            gpu_memory_gb: None,
            storage_gb: 50.0,
            network_bandwidth_mbps: Some(1000),
        }
    }
    
    /// Create resource spec for GPU training
    pub fn gpu_training() -> Self {
        Self {
            cpu_cores: 8,
            memory_gb: 16.0,
            gpu_required: true,
            gpu_memory_gb: Some(8.0),
            storage_gb: 100.0,
            network_bandwidth_mbps: Some(1000),
        }
    }
    
    /// Create resource spec for lightweight inference
    pub fn inference_only() -> Self {
        Self {
            cpu_cores: 2,
            memory_gb: 4.0,
            gpu_required: false,
            gpu_memory_gb: None,
            storage_gb: 5.0,
            network_bandwidth_mbps: Some(100),
        }
    }
}

impl MultiOSJobScheduler {
    /// Create new job scheduler
    pub fn new(config: MultiOSSchedulingConfig) -> Self {
        println!("🔧 MultiOS Job Scheduler initialized:");
        println!("   Workers: {}", config.num_workers);
        println!("   Scheduler type: {:?}", config.scheduler_type);
        println!("   Auto-scaling: {}", config.auto_scaling);
        
        let mut worker_pool = WorkerPool::new(config.num_workers);
        
        // Initialize workers based on available resources
        worker_pool.initialize_workers();
        
        Self {
            config,
            job_queue: Vec::new(),
            active_jobs: HashMap::new(),
            worker_pool,
            job_history: Vec::new(),
        }
    }
    
    /// Submit a new ML job for scheduling
    pub fn submit_job(&mut self, job: MLJobType) -> String {
        let job_id = format!("ml_job_{}", self.job_queue.len() + self.active_jobs.len());
        
        let estimated_duration = self.estimate_job_duration(&job);
        let resource_requirements = self.determine_resource_requirements(&job);
        
        let scheduled_job = ScheduledMLJob {
            job_id: job_id.clone(),
            job_type: job,
            status: JobStatus::Pending,
            submitted_at: std::time::Instant::now(),
            started_at: None,
            estimated_duration,
            worker_assignment: None,
            progress: 0.0,
            resource_usage: ResourceUsage::default(),
        };
        
        self.job_queue.push(scheduled_job);
        
        println!("📋 Job submitted: {} ({} jobs in queue)", job_id, self.job_queue.len());
        
        // Try to schedule immediately if workers are available
        self.schedule_jobs();
        
        job_id
    }
    
    /// Check job status
    pub fn get_job_status(&self, job_id: &str) -> Option<&JobStatus> {
        // Check active jobs first
        if let Some(job) = self.active_jobs.get(job_id) {
            return Some(&job.status);
        }
        
        // Check queued jobs
        for job in &self.job_queue {
            if job.job_id == job_id {
                return Some(&job.status);
            }
        }
        
        // Check job history
        for job in &self.job_history {
            if job.job_id == job_id {
                return Some(&job.status);
            }
        }
        
        None
    }
    
    /// Cancel a pending or running job
    pub fn cancel_job(&mut self, job_id: &str) -> bool {
        // Cancel from queue
        if let Some(pos) = self.job_queue.iter().position(|job| job.job_id == job_id) {
            let job = self.job_queue.remove(pos);
            println!("❌ Job cancelled from queue: {}", job_id);
            return true;
        }
        
        // Cancel from active jobs
        if let Some(mut job) = self.active_jobs.remove(job_id) {
            job.status = JobStatus::Cancelled;
            self.job_history.push(job);
            println!("❌ Job cancelled during execution: {}", job_id);
            return true;
        }
        
        false
    }
    
    /// Get scheduler statistics
    pub fn get_statistics(&self) -> SchedulerStats {
        let queued_count = self.job_queue.len();
        let active_count = self.active_jobs.len();
        let completed_count = self.job_history.iter()
            .filter(|job| matches!(job.status, JobStatus::Completed))
            .count();
        
        let total_memory_used = self.active_jobs.values()
            .map(|job| job.resource_usage.memory_usage_mb)
            .sum::<f64>();
        
        let avg_cpu_usage = if !self.active_jobs.is_empty() {
            self.active_jobs.values()
                .map(|job| job.resource_usage.cpu_usage_percent)
                .sum::<f64>() / self.active_jobs.len() as f64
        } else {
            0.0
        };
        
        SchedulerStats {
            queued_jobs: queued_count,
            active_jobs: active_count,
            completed_jobs: completed_count,
            total_memory_used_mb: total_memory_used,
            average_cpu_usage_percent: avg_cpu_usage,
            available_workers: self.worker_pool.get_available_worker_count(),
            scheduler_utilization_percent: (active_count as f64 / self.config.num_workers as f64) * 100.0,
        }
    }
    
    /// Update job progress (simulated)
    pub fn update_job_progress(&mut self, job_id: &str, progress: f64) {
        if let Some(job) = self.active_jobs.get_mut(job_id) {
            job.progress = progress;
            
            // Update resource usage (simulated)
            job.resource_usage.cpu_usage_percent = 50.0 + progress * 40.0;
            job.resource_usage.memory_usage_mb = 100.0 + progress * 50.0;
            
            if progress >= 1.0 {
                job.status = JobStatus::Completed;
                let completed_job = self.active_jobs.remove(job_id).unwrap();
                self.job_history.push(completed_job);
                println!("✅ Job completed: {}", job_id);
            }
        }
    }
    
    /// Schedule jobs based on scheduler type and available resources
    fn schedule_jobs(&mut self) {
        if self.job_queue.is_empty() {
            return;
        }
        
        match self.config.scheduler_type {
            SchedulerType::FIFO => self.schedule_fifo(),
            SchedulerType::Priority => self.schedule_priority(),
            SchedulerType::RoundRobin => self.schedule_round_robin(),
            SchedulerType::ResourceAware => self.schedule_resource_aware(),
            SchedulerType::GPUAware => self.schedule_gpu_aware(),
            SchedulerType::LoadBalanced => self.schedule_load_balanced(),
        }
    }
    
    /// First-In-First-Out scheduling
    fn schedule_fifo(&mut self) {
        while !self.job_queue.is_empty() {
            let available_worker = self.worker_pool.get_available_worker();
            if let Some(worker_id) = available_worker {
                let job = self.job_queue.remove(0);
                self.start_job_on_worker(job, worker_id);
            } else {
                break;
            }
        }
    }
    
    /// Priority-based scheduling
    fn schedule_priority(&mut self) {
        self.job_queue.sort_by(|a, b| b.priority_level.cmp(&a.priority_level));
        self.schedule_fifo();
    }
    
    /// Round-robin scheduling
    fn schedule_round_robin(&mut self) {
        let mut worker_index = 0;
        let mut job_index = 0;
        
        while job_index < self.job_queue.len() {
            if let Some(worker_id) = self.worker_pool.get_available_worker_by_index(worker_index % self.config.num_workers) {
                let job = self.job_queue.remove(job_index);
                self.start_job_on_worker(job, worker_id);
                worker_index = (worker_index + 1) % self.config.num_workers;
            } else {
                job_index += 1;
            }
        }
    }
    
    /// Resource-aware scheduling
    fn schedule_resource_aware(&mut self) {
        // Sort jobs by resource efficiency (estimated duration / resources needed)
        self.job_queue.sort_by(|a, b| {
            let efficiency_a = a.estimated_duration.as_secs_f64() / a.resource_usage.memory_usage_mb;
            let efficiency_b = b.estimated_duration.as_secs_f64() / b.resource_usage.memory_usage_mb;
            efficiency_b.partial_cmp(&efficiency_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        self.schedule_fifo();
    }
    
    /// GPU-aware scheduling
    fn schedule_gpu_aware(&mut self) {
        // Separate GPU and CPU jobs
        let mut gpu_jobs = Vec::new();
        let mut cpu_jobs = Vec::new();
        
        for job in self.job_queue.drain(..) {
            if matches!(job.job_type, MLJobType::Training { .. }) {
                // Assume training jobs need GPU
                gpu_jobs.push(job);
            } else {
                cpu_jobs.push(job);
            }
        }
        
        // Schedule GPU jobs first if GPU workers available
        if self.worker_pool.has_gpu_workers() {
            self.job_queue.extend(gpu_jobs);
            self.schedule_fifo();
        }
        
        // Then schedule CPU jobs
        self.job_queue.extend(cpu_jobs);
        self.schedule_fifo();
    }
    
    /// Load-balanced scheduling
    fn schedule_load_balanced(&mut self) {
        // Find the worker with lowest current load
        while !self.job_queue.is_empty() {
            let best_worker = self.worker_pool.get_least_loaded_worker();
            if let Some(worker_id) = best_worker {
                let job = self.job_queue.remove(0);
                self.start_job_on_worker(job, worker_id);
            } else {
                break;
            }
        }
    }
    
    /// Start a job on a specific worker
    fn start_job_on_worker(&mut self, mut job: ScheduledMLJob, worker_id: usize) {
        job.status = JobStatus::Running;
        job.started_at = Some(std::time::Instant::now());
        job.worker_assignment = Some(format!("worker_{}", worker_id));
        
        self.active_jobs.insert(job.job_id.clone(), job.clone());
        self.worker_pool.assign_job_to_worker(worker_id, job.job_id.clone());
        
        println!("🚀 Started job {} on worker {}", job.job_id, worker_id);
        
        // Simulate job execution
        self.simulate_job_execution(job.job_id);
    }
    
    /// Simulate job execution progress
    fn simulate_job_execution(&mut self, job_id: String) {
        // In a real implementation, this would be handled by the actual worker
        // For demonstration, we'll simulate progress updates
        
        let total_duration = match &self.active_jobs.get(&job_id).unwrap().job_type {
            MLJobType::Training { epochs, .. } => Duration::from_secs(*epochs as u64 * 10), // 10s per epoch
            MLJobType::Evaluation { .. } => Duration::from_secs(30),
            MLJobType::HyperparameterSearch { max_trials, .. } => Duration::from_secs(*max_trials as u64 * 5),
            _ => Duration::from_secs(60),
        };
        
        // Simulate progress updates
        let progress_steps = 10;
        let step_duration = total_duration / progress_steps as u32;
        
        std::thread::spawn(move || {
            std::thread::sleep(step_duration);
            // Progress would be updated in real implementation
        });
    }
    
    /// Estimate job duration based on job type
    fn estimate_job_duration(&self, job: &MLJobType) -> Duration {
        match job {
            MLJobType::Training { epochs, .. } => Duration::from_secs(*epochs as u64 * 30),
            MLJobType::Evaluation { .. } => Duration::from_secs(60),
            MLJobType::HyperparameterSearch { max_trials, .. } => Duration::from_secs(*max_trials as u64 * 45),
            MLJobType::ModelInference { .. } => Duration::from_secs(30),
            MLJobType::DataPreprocessing { .. } => Duration::from_secs(120),
        }
    }
    
    /// Determine resource requirements for a job
    fn determine_resource_requirements(&self, job: &MLJobType) -> ResourceUsage {
        match job {
            MLJobType::Training { .. } => ResourceUsage {
                cpu_usage_percent: 80.0,
                memory_usage_mb: 2048.0,
                gpu_usage_percent: Some(90.0),
                gpu_memory_mb: Some(4096.0),
                disk_io_mb_per_sec: 100.0,
                network_io_mb_per_sec: 50.0,
            },
            MLJobType::Evaluation { .. } => ResourceUsage {
                cpu_usage_percent: 60.0,
                memory_usage_mb: 1024.0,
                gpu_usage_percent: None,
                gpu_memory_mb: None,
                disk_io_mb_per_sec: 200.0,
                network_io_mb_per_sec: 30.0,
            },
            _ => ResourceUsage::default(),
        }
    }
}

impl WorkerPool {
    pub fn new(num_workers: usize) -> Self {
        Self {
            workers: Vec::new(),
            available_workers: (0..num_workers).collect(),
            worker_health_status: HashMap::new(),
        }
    }
    
    pub fn initialize_workers(&mut self) {
        for i in 0..self.workers.len() {
            self.worker_health_status.insert(i, WorkerHealth {
                last_heartbeat: std::time::Instant::now(),
                cpu_temperature: Some(45.0),
                memory_pressure: 0.1,
                error_count: 0,
                jobs_completed: 0,
            });
        }
    }
    
    pub fn get_available_worker(&self) -> Option<usize> {
        self.available_workers.first().copied()
    }
    
    pub fn get_available_worker_by_index(&self, index: usize) -> Option<usize> {
        if index < self.available_workers.len() {
            Some(self.available_workers[index])
        } else {
            None
        }
    }
    
    pub fn get_least_loaded_worker(&self) -> Option<usize> {
        self.workers.iter()
            .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap_or(std::cmp::Ordering::Equal))
            .map(|worker| worker.worker_id)
    }
    
    pub fn has_gpu_workers(&self) -> bool {
        self.workers.iter().any(|worker| worker.gpu_available)
    }
    
    pub fn assign_job_to_worker(&mut self, worker_id: usize, job_id: String) {
        // Mark worker as busy
        if let Some(worker) = self.workers.iter_mut().find(|w| w.worker_id == worker_id) {
            worker.status = WorkerStatus::Busy { current_job: job_id, progress: 0.0 };
        }
        
        // Remove from available workers
        self.available_workers.retain(|&id| id != worker_id);
    }
    
    pub fn get_available_worker_count(&self) -> usize {
        self.available_workers.len()
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 50.0,
            memory_usage_mb: 512.0,
            gpu_usage_percent: None,
            gpu_memory_mb: None,
            disk_io_mb_per_sec: 10.0,
            network_io_mb_per_sec: 5.0,
        }
    }
}

/// Scheduler statistics
pub struct SchedulerStats {
    pub queued_jobs: usize,
    pub active_jobs: usize,
    pub completed_jobs: usize,
    pub total_memory_used_mb: f64,
    pub average_cpu_usage_percent: f64,
    pub available_workers: usize,
    pub scheduler_utilization_percent: f64,
}

/// Educational example 1: Basic job scheduling
pub fn example_basic_job_scheduling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Basic Job Scheduling ===");
    
    let config = MultiOSSchedulingConfig::new(4)
        .scheduler_type(SchedulerType::FIFO)
        .priority(PriorityLevel::Normal);
    
    let mut scheduler = MultiOSJobScheduler::new(config);
    
    // Submit training jobs
    let job1_id = scheduler.submit_job(MLJobType::Training {
        model_config: ModelConfig {
            architecture: ModelArchitecture::SimpleNN { hidden_layers: vec![64, 32] },
            input_shape: vec![4],
            output_shape: vec![3],
            hyperparameters: HashMap::new(),
        },
        dataset_config: DatasetConfig {
            dataset_type: "iris".to_string(),
            data_path: "datasets/iris.csv".to_string(),
            validation_split: 0.2,
            batch_size: 32,
            preprocessing_steps: vec!["normalize".to_string()],
        },
        epochs: 10,
    });
    
    let job2_id = scheduler.submit_job(MLJobType::Evaluation {
        model_path: "models/iris_model.bin".to_string(),
        test_dataset: "datasets/iris_test.csv".to_string(),
        metrics: vec!["accuracy".to_string(), "precision".to_string()],
    });
    
    let job3_id = scheduler.submit_job(MLJobType::Training {
        model_config: ModelConfig {
            architecture: ModelArchitecture::CNN { 
                conv_layers: vec![ConvLayerConfig { filters: 32, kernel_size: (3, 3), activation: ActivationFunction::ReLU }]
            },
            input_shape: vec![28, 28, 1],
            output_shape: vec![10],
            hyperparameters: HashMap::new(),
        },
        dataset_config: DatasetConfig {
            dataset_type: "mnist".to_string(),
            data_path: "datasets/mnist.csv".to_string(),
            validation_split: 0.1,
            batch_size: 64,
            preprocessing_steps: vec!["normalize".to_string(), "augment".to_string()],
        },
        epochs: 20,
    });
    
    println!("\n📊 Submitted 3 jobs:");
    println!("   Job 1: {} (Training)", job1_id);
    println!("   Job 2: {} (Evaluation)", job2_id);
    println!("   Job 3: {} (CNN Training)", job3_id);
    
    // Check scheduler statistics
    let stats = scheduler.get_statistics();
    println!("\n📈 Scheduler Statistics:");
    println!("   Queued jobs: {}", stats.queued_jobs);
    println!("   Active jobs: {}", stats.active_jobs);
    println!("   Available workers: {}", stats.available_workers);
    
    // Simulate some job progression
    for progress in [0.2, 0.5, 0.8, 1.0].iter() {
        scheduler.update_job_progress(&job1_id, *progress);
        println!("   Job {} progress: {}%", job1_id, progress * 100.0);
        std::thread::sleep(Duration::from_millis(100));
    }
    
    println!("✅ Basic scheduling example completed");
    Ok(())
}

/// Educational example 2: Priority scheduling
pub fn example_priority_scheduling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 2: Priority Scheduling ===");
    
    let config = MultiOSSchedulingConfig::new(3)
        .scheduler_type(SchedulerType::Priority);
    
    let mut scheduler = MultiOSJobScheduler::new(config);
    
    // Submit jobs with different priorities
    scheduler.submit_job(MLJobType::Training {
        model_config: ModelConfig {
            architecture: ModelArchitecture::SimpleNN { hidden_layers: vec![32] },
            input_shape: vec![10],
            output_shape: vec![2],
            hyperparameters: HashMap::new(),
        },
        dataset_config: DatasetConfig {
            dataset_type: "synthetic".to_string(),
            data_path: "datasets/synthetic.csv".to_string(),
            validation_split: 0.2,
            batch_size: 16,
            preprocessing_steps: vec![],
        },
        epochs: 5,
    });
    
    // Critical job (should be scheduled first)
    let critical_job_id = scheduler.submit_job(MLJobType::ModelInference {
        model_path: "models/production_model.bin".to_string(),
        input_data: "realtime_data.json".to_string(),
        batch_size: 1000,
    });
    
    // Update job priority (simulated)
    println!("🔴 Critical job submitted: {} (will be prioritized)", critical_job_id);
    
    let stats = scheduler.get_statistics();
    println!("\n📈 Priority scheduling statistics:");
    println!("   Queued jobs: {}", stats.queued_jobs);
    println!("   Available workers: {}", stats.available_workers);
    
    println!("✅ Priority scheduling example completed");
    Ok(())
}

/// Educational example 3: Resource-aware scheduling
pub fn example_resource_aware_scheduling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 3: Resource-Aware Scheduling ===");
    
    let config = MultiOSSchedulingConfig::new(6)
        .scheduler_type(SchedulerType::ResourceAware)
        .resource_requirements(ResourceSpec::cpu_intensive());
    
    let mut scheduler = MultiOSJobScheduler::new(config);
    
    // Submit jobs with different resource requirements
    let cpu_job_id = scheduler.submit_job(MLJobType::Training {
        model_config: ModelConfig {
            architecture: ModelArchitecture::CNN { 
                conv_layers: vec![
                    ConvLayerConfig { filters: 64, kernel_size: (3, 3), activation: ActivationFunction::ReLU },
                    ConvLayerConfig { filters: 128, kernel_size: (3, 3), activation: ActivationFunction::ReLU },
                ]
            },
            input_shape: vec![32, 32, 3],
            output_shape: vec![10],
            hyperparameters: HashMap::new(),
        },
        dataset_config: DatasetConfig {
            dataset_type: "cifar10".to_string(),
            data_path: "datasets/cifar10.csv".to_string(),
            validation_split: 0.1,
            batch_size: 128,
            preprocessing_steps: vec!["normalize".to_string(), "augment".to_string()],
        },
        epochs: 50,
    });
    
    let inference_job_id = scheduler.submit_job(MLJobType::ModelInference {
        model_path: "models/fast_model.bin".to_string(),
        input_data: "batch_input.json".to_string(),
        batch_size: 500,
    });
    
    let hyperparam_job_id = scheduler.submit_job(MLJobType::HyperparameterSearch {
        search_space: {
            let mut space = HashMap::new();
            space.insert("learning_rate".to_string(), vec!["0.001".to_string(), "0.01".to_string(), "0.1".to_string()]);
            space.insert("batch_size".to_string(), vec!["16".to_string(), "32".to_string(), "64".to_string()]);
            space
        },
        objective_metric: "validation_accuracy".to_string(),
        max_trials: 20,
    });
    
    println!("📋 Submitted 3 jobs with different resource requirements:");
    println!("   CPU-intensive: {} (CNN training)", cpu_job_id);
    println!("   Lightweight: {} (inference)", inference_job_id);
    println!("   Medium: {} (hyperparameter search)", hyperparam_job_id);
    
    let stats = scheduler.get_statistics();
    println!("\n📈 Resource-aware scheduling results:");
    println!("   Total memory used: {:.1} MB", stats.total_memory_used_mb);
    println!("   Average CPU usage: {:.1}%", stats.average_cpu_usage_percent);
    println!("   Scheduler utilization: {:.1}%", stats.scheduler_utilization_percent);
    
    println!("✅ Resource-aware scheduling example completed");
    Ok(())
}

/// Educational example 4: Complete scheduling workflow
pub fn example_complete_scheduling_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 4: Complete Scheduling Workflow ===");
    
    let config = MultiOSSchedulingConfig::new(4)
        .scheduler_type(SchedulerType::LoadBalanced)
        .auto_scaling(true)
        .checkpoint_frequency(Duration::from_secs(60));
    
    let mut scheduler = MultiOSJobScheduler::new(config);
    
    // Simulate a complete ML pipeline
    println!("🔄 Simulating complete ML pipeline:");
    
    // 1. Data preprocessing
    let prep_job_id = scheduler.submit_job(MLJobType::DataPreprocessing {
        dataset_path: "raw_data/large_dataset.csv".to_string(),
        preprocessing_steps: vec![
            "clean".to_string(),
            "normalize".to_string(),
            "encode_categorical".to_string(),
            "split_train_test".to_string(),
        ],
        output_path: "processed_data/".to_string(),
    });
    println!("1. 📊 Submitted preprocessing job: {}", prep_job_id);
    
    // 2. Model training (multiple experiments)
    let experiment_jobs: Vec<String> = (0..3).map(|i| {
        scheduler.submit_job(MLJobType::Training {
            model_config: ModelConfig {
                architecture: ModelArchitecture::SimpleNN { 
                    hidden_layers: vec![64, 32, 16 + i * 8] 
                },
                input_shape: vec![20],
                output_shape: vec![3],
                hyperparameters: {
                    let mut hyperparams = HashMap::new();
                    hyperparams.insert("learning_rate".to_string(), format!("0.00{}", 1 + i));
                    hyperparams
                },
            },
            dataset_config: DatasetConfig {
                dataset_type: "processed".to_string(),
                data_path: "processed_data/train.csv".to_string(),
                validation_split: 0.2,
                batch_size: 32,
                preprocessing_steps: vec![],
            },
            epochs: 20,
        })
    }).collect();
    
    println!("2. 🧠 Submitted {} training experiments:", experiment_jobs.len());
    for (i, job_id) in experiment_jobs.iter().enumerate() {
        println!("   Experiment {}: {}", i + 1, job_id);
    }
    
    // 3. Model evaluation
    let eval_job_ids: Vec<String> = experiment_jobs.iter().map(|training_job_id| {
        scheduler.submit_job(MLJobType::Evaluation {
            model_path: format!("models/{}.bin", training_job_id),
            test_dataset: "processed_data/test.csv".to_string(),
            metrics: vec!["accuracy".to_string(), "precision".to_string(), "recall".to_string()],
        })
    }).collect();
    
    println!("3. 📈 Submitted {} evaluation jobs", eval_job_ids.len());
    
    // 4. Best model inference
    let inference_job_id = scheduler.submit_job(MLJobType::ModelInference {
        model_path: "models/best_model.bin".to_string(),
        input_data: "production_data.json".to_string(),
        batch_size: 10000,
    });
    
    println!("4. 🚀 Submitted production inference job: {}", inference_job_id);
    
    // Monitor workflow progress
    println!("\n📊 Workflow Monitoring:");
    for step in 0..5 {
        let stats = scheduler.get_statistics();
        println!("   Step {}: {} queued, {} active, {} completed", 
                 step + 1, stats.queued_jobs, stats.active_jobs, stats.completed_jobs);
        
        // Simulate job completion
        if !scheduler.job_queue.is_empty() {
            let completed_job = scheduler.job_queue.first().unwrap();
            println!("     ▶️  Completing: {}", completed_job.job_id);
        }
        
        std::thread::sleep(Duration::from_millis(200));
    }
    
    let final_stats = scheduler.get_statistics();
    println!("\n🏁 Final Workflow Statistics:");
    println!("   Total jobs completed: {}", final_stats.completed_jobs);
    println!("   Average scheduler utilization: {:.1}%", final_stats.scheduler_utilization_percent);
    println!("   Peak memory usage: {:.1} MB", final_stats.total_memory_used_mb);
    
    println!("✅ Complete scheduling workflow example finished");
    Ok(())
}

/// Helper function to demonstrate scheduling concepts
pub fn demonstrate_scheduling_concepts() {
    println!("\n🎯 MULTIOS SCHEDULING CONCEPTS:\n");
    
    println!("1. JOB QUEUE MANAGEMENT:");
    println!("   • FIFO: Simple first-come-first-served");
    println!("   • Priority: Important jobs run first");
    println!("   • Round-robin: Fair distribution across workers");
    
    println!("\n2. RESOURCE AWARENESS:");
    println!("   • CPU-intensive vs I/O-intensive jobs");
    println!("   • Memory requirements matching");
    println!("   • GPU resource allocation");
    
    println!("\n3. LOAD BALANCING:");
    println!("   • Dynamic worker assignment");
    println!("   • Workload distribution");
    println!("   • Automatic scaling based on demand");
    
    println!("\n4. JOB LIFECYCLE:");
    println!("   • Submission → Queued → Running → Completed");
    println!("   • Progress tracking and reporting");
    println!("   • Error handling and recovery");
    
    println!("\n5. EDUCATIONAL BENEFITS:");
    println!("   • Visualize job scheduling concepts");
    println!("   • Understand resource allocation");
    println!("   • Learn distributed computing principles");
    println!("   • Practice with real scheduling scenarios");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scheduling_config() {
        let config = MultiOSSchedulingConfig::new(8);
        assert_eq!(config.num_workers, 8);
        assert!(config.auto_scaling);
        assert_eq!(config.scheduler_type, SchedulerType::ResourceAware);
    }
    
    #[test]
    fn test_resource_spec() {
        let cpu_spec = ResourceSpec::cpu_intensive();
        assert_eq!(cpu_spec.cpu_cores, 16);
        assert!(!cpu_spec.gpu_required);
        
        let gpu_spec = ResourceSpec::gpu_training();
        assert!(gpu_spec.gpu_required);
        assert_eq!(gpu_spec.gpu_memory_gb, Some(8.0));
    }
    
    #[test]
    fn test_job_scheduling() {
        let mut scheduler = MultiOSJobScheduler::new(MultiOSSchedulingConfig::new(2));
        
        let job_id = scheduler.submit_job(MLJobType::Training {
            model_config: ModelConfig {
                architecture: ModelArchitecture::SimpleNN { hidden_layers: vec![32] },
                input_shape: vec![4],
                output_shape: vec![2],
                hyperparameters: HashMap::new(),
            },
            dataset_config: DatasetConfig {
                dataset_type: "test".to_string(),
                data_path: "test.csv".to_string(),
                validation_split: 0.2,
                batch_size: 16,
                preprocessing_steps: vec![],
            },
            epochs: 5,
        });
        
        assert!(!job_id.is_empty());
        assert_eq!(scheduler.get_statistics().queued_jobs, 1);
    }
    
    #[test]
    fn test_priority_levels() {
        assert!(PriorityLevel::Critical > PriorityLevel::High);
        assert!(PriorityLevel::High > PriorityLevel::Normal);
        assert!(PriorityLevel::Normal > PriorityLevel::Low);
    }
}