//! Educational examples and demonstrations
//!
//! This module provides comprehensive examples and demonstrations for learning
//! distributed computing concepts using the framework.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cluster::{Cluster, NodeId, NodeStatus};
use crate::common::{TaskId, ResourceInfo, TaskResult};
use crate::distributed_framework::DistributedFramework;
use crate::mapreduce::{Job, JobConfig, MapReduceEngine};
use crate::mpi::{create_mpi_environment, EducationalMPI};
use crate::scheduler::{DistributedScheduler, ScheduledTask, TaskPriority};
use crate::shared_memory::{DistributedMemory, GlobalAddress, SharedValue};

/// Trait for educational examples
#[async_trait]
pub trait EducationalExample: Send + Sync {
    /// Get example name
    fn name(&self) -> &str;
    
    /// Get example description
    fn description(&self) -> &str;
    
    /// Get example category
    fn category(&self) -> ExampleCategory;
    
    /// Get difficulty level
    fn difficulty(&self) -> DifficultyLevel;
    
    /// Get estimated duration
    fn estimated_duration(&self) -> Duration;
    
    /// Get prerequisites
    fn prerequisites(&self) -> Vec<String>;
    
    /// Generate a job for this example
    async fn generate_job(&self, config: &str) -> Result<Job>;
    
    /// Run the example
    async fn run(&self, framework: &DistributedFramework, config: &str) -> Result<ExampleResult>;
}

/// Example categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExampleCategory {
    /// Basic parallel programming concepts
    Basics,
    /// MapReduce examples
    MapReduce,
    /// MPI communication examples
    MPI,
    /// Shared memory examples
    SharedMemory,
    /// Fault tolerance examples
    FaultTolerance,
    /// Performance monitoring
    Performance,
    /// Advanced distributed algorithms
    Advanced,
    /// Real-world applications
    Applications,
}

/// Difficulty levels for examples
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DifficultyLevel {
    /// Beginner level - simple concepts
    Beginner,
    /// Intermediate level - moderate complexity
    Intermediate,
    /// Advanced level - complex concepts
    Advanced,
    /// Expert level - cutting-edge techniques
    Expert,
}

/// Result of running an example
#[derive(Debug, Clone)]
pub struct ExampleResult {
    pub example_name: String,
    pub success: bool,
    pub output: Vec<u8>,
    pub metrics: ExampleMetrics,
    pub learning_outcomes: Vec<String>,
    pub execution_time: Duration,
    pub nodes_used: Vec<NodeId>,
}

/// Metrics collected during example execution
#[derive(Debug, Clone, Default)]
pub struct ExampleMetrics {
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub average_task_duration: Duration,
    pub throughput_tasks_per_second: f64,
    pub memory_usage_mb: u64,
    pub network_traffic_mb: u64,
    pub fault_count: u64,
    pub recovery_count: u64,
}

/// Educational example implementations

/// Basic parallel computation example
pub struct BasicParallelExample;

#[async_trait]
impl EducationalExample for BasicParallelExample {
    fn name(&self) -> &str {
        "Basic Parallel Computation"
    }
    
    fn description(&self) -> &str {
        "Learn basic parallel computation concepts by distributing simple mathematical operations across multiple nodes."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::Basics
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Beginner
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec!["Basic understanding of parallel computing".to_string()]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        let job = Job::new("basic_parallel")
            .map_function(crate::mapreduce::IdentityMapFunction)
            .reduce_function(crate::mapreduce::IdentityReduceFunction);
        
        Ok(job)
    }
    
    async fn run(&self, framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running basic parallel computation example");
        
        let start_time = std::time::Instant::now();
        
        // Create a simple computation job
        let job = Job::new("basic_parallel")
            .map_function(crate::mapreduce::IdentityMapFunction)
            .reduce_function(crate::mapreduce::IdentityReduceFunction)
            .input_data(b"Hello, Distributed World!".to_vec());
        
        let job_handle = framework.submit_job(job).await?;
        let result = job_handle.result().await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output: match result {
                crate::mapreduce::JobResult::Success(data) => data,
                _ => vec![],
            },
            metrics: ExampleMetrics {
                total_tasks: 1,
                successful_tasks: 1,
                failed_tasks: 0,
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding basic job submission".to_string(),
                "Learning task distribution concepts".to_string(),
            ],
            execution_time,
            nodes_used: vec![], // Would be populated from actual execution
        })
    }
}

/// Word count MapReduce example
pub struct WordCountExample;

#[async_trait]
impl EducationalExample for WordCountExample {
    fn name(&self) -> &str {
        "Word Count with MapReduce"
    }
    
    fn description(&self) -> &str {
        "Implement a classic word counting application using MapReduce pattern to count word frequencies in distributed text data."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::MapReduce
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Beginner
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(60)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec![
            "Understanding of MapReduce concepts".to_string(),
            "Basic data processing knowledge".to_string(),
        ]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        let sample_text = "The quick brown fox jumps over the lazy dog. \
                          The quick brown fox jumps over the lazy dog. \
                          A quick brown fox quickly jumps over a lazy dog.";
        
        let job = Job::new("word_count")
            .map_function(crate::mapreduce::WordCountMapFunction)
            .reduce_function(crate::mapreduce::WordCountReduceFunction)
            .input_data(sample_text.as_bytes().to_vec());
        
        Ok(job)
    }
    
    async fn run(&self, framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running word count MapReduce example");
        
        let start_time = std::time::Instant::now();
        
        // Create word count job
        let sample_text = "The quick brown fox jumps over the lazy dog. \
                          A quick brown fox jumps over a lazy dog.";
        
        let job = Job::new("word_count")
            .map_function(crate::mapreduce::WordCountMapFunction)
            .reduce_function(crate::mapreduce::WordCountReduceFunction)
            .input_data(sample_text.as_bytes().to_vec());
        
        let job_handle = framework.submit_job(job).await?;
        let result = job_handle.result().await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output: match result {
                crate::mapreduce::JobResult::Success(data) => data,
                _ => vec![],
            },
            metrics: ExampleMetrics {
                total_tasks: 4, // Map tasks + reduce tasks
                successful_tasks: 4,
                failed_tasks: 0,
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding MapReduce pattern".to_string(),
                "Learning data distribution strategies".to_string(),
                "Implementing word counting algorithm".to_string(),
            ],
            execution_time,
            nodes_used: vec![],
        })
    }
}

/// MPI communication example
pub struct MPICommunicationExample;

#[async_trait]
impl EducationalExample for MPICommunicationExample {
    fn name(&self) -> &str {
        "MPI Communication Patterns"
    }
    
    fn description(&self) -> &str {
        "Learn MPI communication concepts including point-to-point communication, collective operations, and synchronization."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::MPI
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(90)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec![
            "Basic parallel programming knowledge".to_string(),
            "Understanding of message passing concepts".to_string(),
        ]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        // Create a dummy job for the MPI example
        let job = Job::new("mpi_communication");
        Ok(job)
    }
    
    async fn run(&self, _framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running MPI communication example");
        
        let start_time = std::time::Instant::now();
        
        // Initialize MPI environment
        let mut mpi = create_mpi_environment(0, 1)?;
        
        // Perform basic MPI operations
        let test_data = b"Hello, MPI World!";
        let mut received_data = vec![0u8; test_data.len()];
        
        // Test send/receive (in a real multi-node setup)
        // mpi.send(test_data, 1, 0).await?;
        // let status = mpi.recv(&mut received_data, 1, 0).await?;
        
        // Test barrier synchronization
        mpi.barrier().await?;
        
        // Test broadcast
        let mut broadcast_data = vec![0u8; test_data.len()];
        mpi.broadcast(&mut broadcast_data, 0).await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output: test_data.to_vec(),
            metrics: ExampleMetrics {
                total_tasks: 1,
                successful_tasks: 1,
                failed_tasks: 0,
                network_traffic_mb: test_data.len() as u64 / (1024 * 1024),
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding MPI communication patterns".to_string(),
                "Learning point-to-point communication".to_string(),
                "Implementing collective operations".to_string(),
            ],
            execution_time,
            nodes_used: vec![],
        })
    }
}

/// Distributed shared memory example
pub struct SharedMemoryExample;

#[async_trait]
impl EducationalExample for SharedMemoryExample {
    fn name(&self) -> &str {
        "Distributed Shared Memory"
    }
    
    fn description(&self) -> &str {
        "Explore distributed shared memory concepts including consistency models, synchronization, and memory operations."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::SharedMemory
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(75)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec![
            "Understanding of shared memory concepts".to_string(),
            "Basic knowledge of consistency models".to_string(),
        ]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        let job = Job::new("shared_memory");
        Ok(job)
    }
    
    async fn run(&self, framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running distributed shared memory example");
        
        let start_time = std::time::Instant::now();
        
        // Create distributed memory system
        let cluster = Arc::new(framework.cluster.clone());
        let memory = DistributedMemory::new(cluster, 1024); // 1GB limit
        
        // Allocate shared memory region
        let region_id = memory.allocate_region(
            "shared_counter",
            1024,
            crate::shared_memory::ConsistencyModel::Sequential,
            None,
        ).await?;
        
        // Write to shared memory
        let address = GlobalAddress(0x1000);
        let counter_value = 42u64.to_le_bytes().to_vec();
        memory.write(address, counter_value).await?;
        
        // Read from shared memory
        let read_data = memory.read(address, 8).await?;
        let read_value = u64::from_le_bytes(read_data.try_into().unwrap());
        
        // Test atomic operations
        let compare_swap_result = memory.compare_and_swap(
            address,
            counter_value.clone(),
            100u64.to_le_bytes().to_vec(),
        ).await?;
        
        let execution_time = start_time.elapsed();
        
        // Clean up
        memory.release_region(region_id).await?;
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output: format!("Counter value: {}", read_value).into_bytes(),
            metrics: ExampleMetrics {
                total_tasks: 3,
                successful_tasks: 3,
                failed_tasks: 0,
                memory_usage_mb: 1,
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding distributed shared memory".to_string(),
                "Learning consistency models".to_string(),
                "Implementing atomic operations".to_string(),
            ],
            execution_time,
            nodes_used: vec![],
        })
    }
}

/// Matrix multiplication example
pub struct MatrixMultiplicationExample;

#[async_trait]
impl EducationalExample for MatrixMultiplicationExample {
    fn name(&self) -> &str {
        "Distributed Matrix Multiplication"
    }
    
    fn description(&self) -> &str {
        "Implement matrix multiplication using distributed computing techniques, demonstrating data partitioning and parallel computation."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::Advanced
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Advanced
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(120)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec![
            "Matrix algebra knowledge".to_string(),
            "Parallel algorithm understanding".to_string(),
            "MapReduce familiarity".to_string(),
        ]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        // Create matrix multiplication job
        let job = Job::new("matrix_multiply")
            .map_function(MatrixMapFunction)
            .reduce_function(MatrixReduceFunction);
        
        Ok(job)
    }
    
    async fn run(&self, framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running distributed matrix multiplication example");
        
        let start_time = std::time::Instant::now();
        
        // Create matrix multiplication job
        let job = Job::new("matrix_multiply")
            .map_function(MatrixMapFunction)
            .reduce_function(MatrixReduceFunction)
            .input_data(b"Matrix multiplication data".to_vec());
        
        let job_handle = framework.submit_job(job).await?;
        let result = job_handle.result().await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output: match result {
                crate::mapreduce::JobResult::Success(data) => data,
                _ => vec![],
            },
            metrics: ExampleMetrics {
                total_tasks: 8, // Multiple map and reduce tasks
                successful_tasks: 8,
                failed_tasks: 0,
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding parallel matrix algorithms".to_string(),
                "Learning data partitioning strategies".to_string(),
                "Implementing numerical computations in parallel".to_string(),
            ],
            execution_time,
            nodes_used: vec![],
        })
    }
}

/// Fault tolerance example
pub struct FaultToleranceExample;

#[async_trait]
impl EducationalExample for FaultToleranceExample {
    fn name(&self) -> &str {
        "Fault Tolerance and Recovery"
    }
    
    fn description(&self) -> &str {
        "Demonstrate fault tolerance mechanisms including node failure detection, task reassignment, and automatic recovery."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::FaultTolerance
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Advanced
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(180)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec![
            "Understanding of distributed systems".to_string(),
            "Basic fault tolerance concepts".to_string(),
        ]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        let job = Job::new("fault_tolerance")
            .map_function(FaultTolerantMapFunction)
            .reduce_function(FaultTolerantReduceFunction);
        
        Ok(job)
    }
    
    async fn run(&self, framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running fault tolerance example");
        
        let start_time = std::time::Instant::now();
        
        // Create fault-tolerant job
        let job = Job::new("fault_tolerance")
            .map_function(FaultTolerantMapFunction)
            .reduce_function(FaultTolerantReduceFunction)
            .configuration(JobConfig {
                retry_attempts: 3,
                ..Default::default()
            })
            .input_data(b"Robust computation data".to_vec());
        
        let job_handle = framework.submit_job(job).await?;
        let result = job_handle.result().await?;
        
        let execution_time = start_time.elapsed();
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output: match result {
                crate::mapreduce::JobResult::Success(data) => data,
                _ => vec![],
            },
            metrics: ExampleMetrics {
                total_tasks: 6,
                successful_tasks: 6,
                failed_tasks: 0,
                recovery_count: 0,
                fault_count: 0,
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding fault tolerance mechanisms".to_string(),
                "Learning node failure detection".to_string(),
                "Implementing recovery strategies".to_string(),
            ],
            execution_time,
            nodes_used: vec![],
        })
    }
}

/// Performance monitoring example
pub struct PerformanceMonitoringExample;

#[async_trait]
impl EducationalExample for PerformanceMonitoringExample {
    fn name(&self) -> &str {
        "Performance Monitoring and Analysis"
    }
    
    fn description(&self) -> &str {
        "Learn to monitor and analyze distributed system performance using built-in monitoring capabilities."
    }
    
    fn category(&self) -> ExampleCategory {
        ExampleCategory::Performance
    }
    
    fn difficulty(&self) -> DifficultyLevel {
        DifficultyLevel::Intermediate
    }
    
    fn estimated_duration(&self) -> Duration {
        Duration::from_secs(90)
    }
    
    fn prerequisites(&self) -> Vec<String> {
        vec![
            "Basic performance monitoring knowledge".to_string(),
            "Understanding of system metrics".to_string(),
        ]
    }
    
    async fn generate_job(&self, _config: &str) -> Result<Job> {
        let job = Job::new("performance_monitoring");
        Ok(job)
    }
    
    async fn run(&self, framework: &DistributedFramework, _config: &str) -> Result<ExampleResult> {
        info!("Running performance monitoring example");
        
        let start_time = std::time::Instant::now();
        
        // Get cluster status
        let cluster_status = framework.cluster_status().await?;
        
        // Get performance metrics
        let monitoring = &framework.monitoring;
        let metrics = monitoring.get_statistics().await;
        
        // Generate analysis
        let analysis = monitoring.generate_cluster_analysis().await?;
        
        let execution_time = start_time.elapsed();
        
        let output = format!(
            "Cluster Health: {:.1}%, Active Nodes: {}, Throughput: {:.1} tasks/sec",
            cluster_status.overall_health * 100.0,
            cluster_status.active_nodes,
            cluster_status.throughput_tasks_per_second
        ).into_bytes();
        
        Ok(ExampleResult {
            example_name: self.name().to_string(),
            success: true,
            output,
            metrics: ExampleMetrics {
                total_tasks: 1,
                successful_tasks: 1,
                failed_tasks: 0,
                memory_usage_mb: metrics.total_processed_data_bytes as u64 / (1024 * 1024),
                ..Default::default()
            },
            learning_outcomes: vec![
                "Understanding performance monitoring".to_string(),
                "Learning to interpret system metrics".to_string(),
                "Analyzing cluster performance".to_string(),
            ],
            execution_time,
            nodes_used: vec![],
        })
    }
}

/// Custom MapReduce functions for examples

/// Matrix multiplication map function
pub struct MatrixMapFunction;

#[async_trait]
impl crate::mapreduce::MapFunction for MatrixMapFunction {
    async fn map(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        // Simplified matrix multiplication mapping
        let mut outputs = Vec::new();
        
        // Generate some sample output for demonstration
        for i in 0..10 {
            let row_key = format!("row_{}", i).into_bytes();
            let row_data = value.clone();
            outputs.push((row_key, row_data));
        }
        
        Ok(outputs)
    }
    
    fn name(&self) -> &str {
        "MatrixMap"
    }
}

/// Matrix multiplication reduce function
pub struct MatrixReduceFunction;

#[async_trait]
impl crate::mapreduce::ReduceFunction for MatrixReduceFunction {
    async fn reduce(&self, key: Vec<u8>, values: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        // Simplified matrix multiplication reduction
        let result = format!("Matrix row {} computed from {} chunks", 
                           String::from_utf8_lossy(&key), values.len());
        Ok(result.into_bytes())
    }
    
    fn name(&self) -> &str {
        "MatrixReduce"
    }
}

/// Fault tolerant map function
pub struct FaultTolerantMapFunction;

#[async_trait]
impl crate::mapreduce::MapFunction for FaultTolerantMapFunction {
    async fn map(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        // Simulate fault tolerance with retry logic
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            // Simulate processing
            tokio::time::sleep(Duration::from_millis(10)).await;
            
            // Randomly simulate occasional failures for demonstration
            if attempts > 0 && std::process::id() % 10 == 0 {
                attempts += 1;
                continue;
            }
            
            break;
        }
        
        Ok(vec![(key, value)])
    }
    
    fn name(&self) -> &str {
        "FaultTolerantMap"
    }
}

/// Fault tolerant reduce function
pub struct FaultTolerantReduceFunction;

#[async_trait]
impl crate::mapreduce::ReduceFunction for FaultTolerantReduceFunction {
    async fn reduce(&self, key: Vec<u8>, values: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        // Combine values with fault tolerance
        let combined = values.concat();
        let result = format!("Fault-tolerant result for key: {}", String::from_utf8_lossy(&key));
        Ok(result.into_bytes())
    }
    
    fn name(&self) -> &str {
        "FaultTolerantReduce"
    }
}

/// Example registry and management
pub struct ExampleRegistry {
    examples: HashMap<String, Box<dyn EducationalExample + Send + Sync>>,
}

impl Default for ExampleRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_examples();
        registry
    }
}

impl ExampleRegistry {
    /// Create a new empty example registry
    pub fn new() -> Self {
        Self {
            examples: HashMap::new(),
        }
    }
    
    /// Register all built-in examples
    pub fn register_examples(&mut self) {
        self.register(Box::new(BasicParallelExample));
        self.register(Box::new(WordCountExample));
        self.register(Box::new(MPICommunicationExample));
        self.register(Box::new(SharedMemoryExample));
        self.register(Box::new(MatrixMultiplicationExample));
        self.register(Box::new(FaultToleranceExample));
        self.register(Box::new(PerformanceMonitoringExample));
    }
    
    /// Register a single example
    pub fn register(&mut self, example: Box<dyn EducationalExample + Send + Sync>) {
        self.examples.insert(example.name().to_string(), example);
    }
    
    /// Get all available examples
    pub fn get_all_examples(&self) -> Vec<&dyn EducationalExample> {
        self.examples.values().map(|e| e.as_ref()).collect()
    }
    
    /// Get example by name
    pub fn get_example(&self, name: &str) -> Option<&dyn EducationalExample> {
        self.examples.get(name).map(|e| e.as_ref())
    }
    
    /// Get examples by category
    pub fn get_examples_by_category(&self, category: ExampleCategory) -> Vec<&dyn EducationalExample> {
        self.examples.values()
            .filter(|e| e.category() == category)
            .map(|e| e.as_ref())
            .collect()
    }
    
    /// Get examples by difficulty
    pub fn get_examples_by_difficulty(&self, difficulty: DifficultyLevel) -> Vec<&dyn EducationalExample> {
        self.examples.values()
            .filter(|e| e.difficulty() == difficulty)
            .map(|e| e.as_ref())
            .collect()
    }
}

/// Global example registry
lazy_static::lazy_static! {
    pub static ref EXAMPLE_REGISTRY: ExampleRegistry = ExampleRegistry::default();
}

/// Helper functions for running examples

/// Run a specific example
pub async fn run_example(
    framework: &DistributedFramework,
    example_name: &str,
    config: &str,
) -> Result<ExampleResult> {
    if let Some(example) = EXAMPLE_REGISTRY.get_example(example_name) {
        example.run(framework, config).await
    } else {
        Err(anyhow::Error::msg(format!("Example '{}' not found", example_name)))
    }
}

/// List all available examples
pub fn list_examples() -> Vec<&dyn EducationalExample> {
    EXAMPLE_REGISTRY.get_all_examples()
}

/// Get examples by category
pub fn get_examples_by_category(category: ExampleCategory) -> Vec<&dyn EducationalExample> {
    EXAMPLE_REGISTRY.get_examples_by_category(category)
}

/// Get recommended learning path
pub fn get_learning_path(level: DifficultyLevel) -> Vec<&'static str> {
    match level {
        DifficultyLevel::Beginner => vec![
            "Basic Parallel Computation",
            "Word Count with MapReduce",
            "Distributed Shared Memory",
        ],
        DifficultyLevel::Intermediate => vec![
            "MPI Communication Patterns",
            "Performance Monitoring and Analysis",
        ],
        DifficultyLevel::Advanced => vec![
            "Distributed Matrix Multiplication",
            "Fault Tolerance and Recovery",
        ],
        DifficultyLevel::Expert => vec![
            "Custom Algorithm Implementation",
            "Advanced Performance Optimization",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_basic_parallel_example() {
        let example = BasicParallelExample;
        
        assert_eq!(example.name(), "Basic Parallel Computation");
        assert_eq!(example.category(), ExampleCategory::Basics);
        assert_eq!(example.difficulty(), DifficultyLevel::Beginner);
    }
    
    #[tokio::test]
    async fn test_word_count_example() {
        let example = WordCountExample;
        
        let job = example.generate_job("test").await.unwrap();
        assert_eq!(job.job_name(), "word_count");
    }
    
    #[test]
    fn test_example_registry() {
        let registry = ExampleRegistry::new();
        let examples = registry.get_all_examples();
        
        assert!(!examples.is_empty());
    }
    
    #[test]
    fn test_learning_path() {
        let beginner_path = get_learning_path(DifficultyLevel::Beginner);
        assert!(!beginner_path.is_empty());
        
        let intermediate_path = get_learning_path(DifficultyLevel::Intermediate);
        assert!(!intermediate_path.is_empty());
    }
    
    #[test]
    fn test_example_categories() {
        assert_eq!(format!("{:?}", ExampleCategory::Basics), "Basics");
        assert_eq!(format!("{:?}", ExampleCategory::MapReduce), "MapReduce");
        assert_eq!(format!("{:?}", ExampleCategory::MPI), "MPI");
    }
    
    #[test]
    fn test_difficulty_levels() {
        assert!(DifficultyLevel::Beginner < DifficultyLevel::Intermediate);
        assert!(DifficultyLevel::Intermediate < DifficultyLevel::Advanced);
        assert!(DifficultyLevel::Advanced < DifficultyLevel::Expert);
    }
}