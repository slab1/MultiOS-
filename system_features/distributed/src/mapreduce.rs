//! MapReduce framework for educational parallel computing
//!
//! This module provides a simplified MapReduce implementation designed for
//! educational purposes, demonstrating key concepts of distributed data processing.

use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cluster::{Cluster, NodeId, NodeStatus};
use crate::scheduler::{DistributedScheduler, ScheduledTask, TaskPriority};

/// Unique job identifier
pub type JobId = Uuid;

/// Unique task identifier within a job
pub type TaskId = Uuid;

/// Key-value pairs for MapReduce intermediate data
pub type KeyValuePair<K, V> = (K, V);

/// Results from MapReduce operations
#[derive(Debug, Clone)]
pub enum JobResult {
    Success(Vec<u8>),
    PartialFailure(Vec<u8>, Vec<TaskError>),
    CompleteFailure(Vec<TaskError>),
}

/// Task execution error
#[derive(Debug, Clone)]
pub struct TaskError {
    pub task_id: TaskId,
    pub error_message: String,
    pub node_id: Option<NodeId>,
    pub timestamp: SystemTime,
}

/// Main MapReduce job representation
#[derive(Debug, Clone)]
pub struct Job {
    id: JobId,
    job_name: String,
    map_function: Arc<dyn MapFunction + Send + Sync>,
    reduce_function: Arc<dyn ReduceFunction + Send + Sync>,
    input_data: Vec<u8>,
    configuration: JobConfig,
    tasks: Vec<MapReduceTask>,
    submission_time: SystemTime,
}

/// Job configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    pub map_tasks: usize,
    pub reduce_tasks: usize,
    pub partition_strategy: PartitionStrategy,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub memory_limit_mb: u64,
    pub output_format: OutputFormat,
}

/// Partitioning strategies for data distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitionStrategy {
    /// Hash-based partitioning
    Hash,
    /// Range-based partitioning
    Range,
    /// Custom partitioning function
    Custom,
}

/// Output formats for job results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Raw binary data
    Binary,
    /// JSON formatted data
    Json,
    /// CSV formatted data
    Csv,
    /// Text formatted data
    Text,
}

/// Individual MapReduce task
#[derive(Debug, Clone)]
pub struct MapReduceTask {
    pub task_id: TaskId,
    pub job_id: JobId,
    pub task_type: TaskType,
    pub input_partition: Vec<u8>,
    pub map_output: Option<Vec<(Vec<u8>, Vec<u8>)>>,
    pub status: TaskStatus,
    pub assigned_node: Option<NodeId>,
    pub start_time: Option<SystemTime>,
    pub completion_time: Option<SystemTime>,
    pub retry_count: u32,
}

/// Types of MapReduce tasks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskType {
    Map,
    Reduce,
    Partition,
    Combine,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Retrying,
}

/// Handle for monitoring and controlling job execution
#[derive(Debug)]
pub struct JobHandle {
    job_id: JobId,
    result_tx: mpsc::UnboundedReceiver<JobResult>,
    result: Option<JobResult>,
    completion_notifier: Option<oneshot::Sender<JobResult>>,
}

/// Map function trait for the Map phase
#[async_trait]
pub trait MapFunction: Send + Sync {
    /// Process input data and emit key-value pairs
    /// 
    /// # Arguments
    /// * `key` - Input key (can be empty for line-based input)
    /// * `value` - Input value (e.g., text line, data chunk)
    /// 
    /// # Returns
    /// Iterator of key-value pairs emitted by the map function
    async fn map(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    
    /// Get a name for this map function for debugging
    fn name(&self) -> &str;
}

/// Reduce function trait for the Reduce phase
#[async_trait]
pub trait ReduceFunction: Send + Sync {
    /// Process intermediate key-value pairs and produce final output
    /// 
    /// # Arguments
    /// * `key` - Intermediate key
    /// * `values` - All values associated with this key
    /// 
    /// # Returns
    /// Final result for this key
    async fn reduce(&self, key: Vec<u8>, values: Vec<Vec<u8>>) -> Result<Vec<u8>>;
    
    /// Get a name for this reduce function for debugging
    fn name(&self) -> &str;
}

/// Main MapReduce execution engine
pub struct MapReduceEngine {
    cluster: Arc<Cluster>,
    scheduler: Arc<DistributedScheduler>,
    active_jobs: Arc<RwLock<HashMap<JobId, Job>>>,
    job_results: Arc<RwLock<HashMap<JobId, JobResult>>>,
    task_queue: Arc<RwLock<VecDeque<MapReduceTask>>>,
    completed_tasks: Arc<RwLock<HashMap<TaskId, MapReduceTask>>>,
    statistics: Arc<RwLock<MapReduceStatistics>>,
}

/// Statistics and metrics for MapReduce operations
#[derive(Debug, Clone, Default)]
pub struct MapReduceStatistics {
    pub total_jobs_submitted: u64,
    pub total_jobs_completed: u64,
    pub total_jobs_failed: u64,
    pub total_map_tasks: u64,
    pub total_reduce_tasks: u64,
    pub total_processed_data_bytes: u64,
    pub average_job_completion_time: Duration,
    pub fault_tolerance_effectiveness: f64,
    pub throughput_jobs_per_hour: f64,
}

impl Job {
    /// Create a new MapReduce job
    pub fn new(job_name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_name: job_name.to_string(),
            map_function: Arc::new(IdentityMapFunction),
            reduce_function: Arc::new(IdentityReduceFunction),
            input_data: Vec::new(),
            configuration: JobConfig::default(),
            tasks: Vec::new(),
            submission_time: SystemTime::now(),
        }
    }
    
    /// Set the map function for this job
    pub fn map_function<F>(self, map_fn: F) -> Self
    where
        F: MapFunction + Send + Sync + 'static,
    {
        Self {
            map_function: Arc::new(map_fn),
            ..self
        }
    }
    
    /// Set the reduce function for this job
    pub fn reduce_function<F>(self, reduce_fn: F) -> Self
    where
        F: ReduceFunction + Send + Sync + 'static,
    {
        Self {
            reduce_function: Arc::new(reduce_fn),
            ..self
        }
    }
    
    /// Set input data for the job
    pub fn input_data(self, data: Vec<u8>) -> Self {
        Self {
            input_data: data,
            ..self
        }
    }
    
    /// Set job configuration
    pub fn configuration(self, config: JobConfig) -> Self {
        Self {
            configuration: config,
            ..self
        }
    }
    
    /// Get the job identifier
    pub fn id(&self) -> JobId {
        self.id
    }
    
    /// Get the job name
    pub fn job_name(&self) -> &str {
        &self.job_name
    }
    
    /// Get number of tasks in the job
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    
    /// Get the tasks in the job
    pub fn tasks(&self) -> &[MapReduceTask] {
        &self.tasks
    }
    
    /// Generate tasks for the job based on input data
    pub fn generate_tasks(&mut self) -> Result<()> {
        debug!("Generating tasks for job {}: {} map tasks, {} reduce tasks",
               self.job_name, self.configuration.map_tasks, self.configuration.reduce_tasks);
        
        self.tasks.clear();
        
        // Generate map tasks
        for i in 0..self.configuration.map_tasks {
            let partition = self.partition_input_data(i);
            let task = MapReduceTask {
                task_id: Uuid::new_v4(),
                job_id: self.id,
                task_type: TaskType::Map,
                input_partition: partition,
                map_output: None,
                status: TaskStatus::Pending,
                assigned_node: None,
                start_time: None,
                completion_time: None,
                retry_count: 0,
            };
            self.tasks.push(task);
        }
        
        // Generate reduce tasks
        for i in 0..self.configuration.reduce_tasks {
            let task = MapReduceTask {
                task_id: Uuid::new_v4(),
                job_id: self.id,
                task_type: TaskType::Reduce,
                input_partition: Vec::new(), // Will be filled during shuffle phase
                map_output: None,
                status: TaskStatus::Pending,
                assigned_node: None,
                start_time: None,
                completion_time: None,
                retry_count: 0,
            };
            self.tasks.push(task);
        }
        
        Ok(())
    }
    
    /// Partition input data for map tasks
    fn partition_input_data(&self, partition_index: usize) -> Vec<u8> {
        let total_partitions = self.configuration.map_tasks;
        let data_len = self.input_data.len();
        let partition_size = (data_len + total_partitions - 1) / total_partitions;
        let start = partition_index * partition_size;
        let end = std::cmp::min(start + partition_size, data_len);
        
        self.input_data[start..end].to_vec()
    }
}

impl JobHandle {
    /// Create a new job handle
    pub fn new(job_id: JobId, tasks: Vec<crate::scheduler::TaskHandle>) -> Self {
        let (result_tx, result_rx) = mpsc::unbounded_channel();
        
        // Store result transmitter for later use
        if let Ok(_) = result_tx.send(JobResult::Success(vec![])) {
            // Initial placeholder
        }
        
        Self {
            job_id,
            result_rx,
            result: None,
            completion_notifier: None,
        }
    }
    
    /// Get the job identifier
    pub fn id(&self) -> JobId {
        self.job_id
    }
    
    /// Wait for job completion and return the result
    pub async fn result(&mut self) -> Result<JobResult> {
        if let Some(result) = self.result.take() {
            return Ok(result);
        }
        
        // Wait for result from the channel
        match self.result_rx.recv().await {
            Some(result) => Ok(result),
            None => Err(anyhow::Error::msg("Job result channel closed")),
        }
    }
    
    /// Check if job is completed
    pub async fn is_completed(&self) -> bool {
        if self.result.is_some() {
            return true;
        }
        
        // Check channel for messages
        self.result_rx.is_closed() || !self.result_rx.is_empty()
    }
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            map_tasks: 4,
            reduce_tasks: 2,
            partition_strategy: PartitionStrategy::Hash,
            timeout: Duration::from_secs(300),
            retry_attempts: 3,
            memory_limit_mb: 512,
            output_format: OutputFormat::Json,
        }
    }
}

impl MapReduceEngine {
    /// Create a new MapReduce engine
    pub fn new(cluster: Arc<Cluster>, scheduler: Arc<DistributedScheduler>) -> Self {
        info!("Creating MapReduce engine");
        
        Self {
            cluster,
            scheduler,
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            job_results: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(VecDeque::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(MapReduceStatistics::default())),
        }
    }
    
    /// Submit a job for execution
    pub async fn submit_job(&self, mut job: Job) -> Result<JobHandle> {
        info!("Submitting MapReduce job: {} (ID: {})", job.job_name, job.id);
        
        // Generate tasks for the job
        job.generate_tasks()?;
        
        // Store job in active jobs
        {
            let mut active_jobs = self.active_jobs.write().await;
            active_jobs.insert(job.id, job.clone());
        }
        
        // Create job handle
        let job_handle = JobHandle::new(job.id, vec![]);
        
        // Start job execution
        self.start_job_execution(job).await?;
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_jobs_submitted += 1;
        }
        
        Ok(job_handle)
    }
    
    /// Start executing a job
    async fn start_job_execution(&self, job: Job) -> Result<()> {
        debug!("Starting execution of job: {}", job.job_name);
        
        // Phase 1: Execute map tasks
        self.execute_map_phase(&job).await?;
        
        // Phase 2: Shuffle intermediate data
        self.execute_shuffle_phase(&job).await?;
        
        // Phase 3: Execute reduce tasks
        self.execute_reduce_phase(&job).await?;
        
        // Phase 4: Finalize and collect results
        self.finalize_job(&job).await?;
        
        Ok(())
    }
    
    /// Execute the map phase of a job
    async fn execute_map_phase(&self, job: &Job) -> Result<()> {
        info!("Executing map phase for job: {}", job.job_name);
        
        let map_tasks: Vec<_> = job.tasks
            .iter()
            .filter(|task| task.task_type == TaskType::Map)
            .cloned()
            .collect();
        
        debug!("Starting {} map tasks", map_tasks.len());
        
        // Execute map tasks concurrently
        let mut handles = Vec::new();
        for task in map_tasks {
            let handle = self.execute_map_task(job, task).await;
            if let Ok(h) = handle {
                handles.push(h);
            }
        }
        
        // Wait for all map tasks to complete
        for handle in handles {
            handle.await.map_err(|e| {
                error!("Map task failed: {}", e);
                anyhow::Error::msg("Map task execution failed")
            })?;
        }
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_map_tasks += job.configuration.map_tasks as u64;
        }
        
        Ok(())
    }
    
    /// Execute a single map task
    async fn execute_map_task(&self, job: &Job, task: MapReduceTask) -> tokio::task::JoinHandle<Result<()>> {
        let map_function = job.map_function.clone();
        let input_data = task.input_partition.clone();
        let task_id = task.task_id;
        
        tokio::spawn(async move {
            debug!("Executing map task: {}", task_id);
            
            // Parse input data (simplified - assume line-based input)
            let lines: Vec<Vec<u8>> = input_data
                .split(|&b| b == b'\n')
                .map(|line| line.to_vec())
                .collect();
            
            let mut map_outputs = Vec::new();
            
            // Apply map function to each input
            for (i, line) in lines.into_iter().enumerate() {
                let key = format!("line_{}", i).into_bytes();
                let map_result = map_function.map(key, line).await?;
                map_outputs.extend(map_result);
            }
            
            // Store intermediate results
            let mut completed_tasks = self.completed_tasks.write().await;
            let mut updated_task = task.clone();
            updated_task.map_output = Some(map_outputs);
            updated_task.status = TaskStatus::Completed;
            updated_task.completion_time = Some(SystemTime::now());
            completed_tasks.insert(task_id, updated_task);
            
            debug!("Map task {} completed with {} outputs", task_id, map_outputs.len());
            Ok(())
        })
    }
    
    /// Execute the shuffle phase (data redistribution between map and reduce tasks)
    async fn execute_shuffle_phase(&self, job: &Job) -> Result<()> {
        info!("Executing shuffle phase for job: {}", job.job_name);
        
        // Collect all map outputs
        let mut all_map_outputs = HashMap::new();
        {
            let completed_tasks = self.completed_tasks.read().await;
            for task in completed_tasks.values() {
                if task.task_type == TaskType::Map && task.status == TaskStatus::Completed {
                    if let Some(map_output) = &task.map_output {
                        for (key, value) in map_output {
                            all_map_outputs
                                .entry(key.clone())
                                .or_insert_with(Vec::new)
                                .push(value.clone());
                        }
                    }
                }
            }
        }
        
        debug!("Shuffle phase: {} unique keys, {} total key-value pairs",
               all_map_outputs.len(),
               all_map_outputs.values().map(|v| v.len()).sum::<usize>());
        
        // Partition data for reduce tasks
        let reduce_partitions = self.partition_for_reduce(&all_map_outputs, job.configuration.reduce_tasks);
        
        // Distribute partitions to reduce tasks
        let mut reduce_tasks = job.tasks
            .iter()
            .filter(|task| task.task_type == TaskType::Reduce)
            .cloned()
            .collect::<Vec<_>>();
        
        for (i, task) in reduce_tasks.iter_mut().enumerate() {
            if let Some(partition) = reduce_partitions.get(&i) {
                task.input_partition = bincode::serialize(partition)?;
            }
        }
        
        Ok(())
    }
    
    /// Partition intermediate data for reduce tasks
    fn partition_for_reduce(
        &self,
        all_outputs: &HashMap<Vec<u8>, Vec<Vec<u8>>>,
        reduce_tasks: usize,
    ) -> HashMap<usize, HashMap<Vec<u8>, Vec<Vec<u8>>>> {
        let mut partitions = HashMap::new();
        
        for (key, values) in all_outputs {
            let partition_index = self.calculate_partition_index(key, reduce_tasks);
            partitions
                .entry(partition_index)
                .or_insert_with(HashMap::new)
                .insert(key.clone(), values.clone());
        }
        
        partitions
    }
    
    /// Calculate partition index for a key
    fn calculate_partition_index(&self, key: &[u8], partitions: usize) -> usize {
        match partitions {
            0 => 0,
            _ => {
                // Simple hash-based partitioning
                let hash: usize = key.iter().map(|&b| b as usize).sum();
                hash % partitions
            }
        }
    }
    
    /// Execute the reduce phase of a job
    async fn execute_reduce_phase(&self, job: &Job) -> Result<()> {
        info!("Executing reduce phase for job: {}", job.job_name);
        
        let reduce_tasks: Vec<_> = job.tasks
            .iter()
            .filter(|task| task.task_type == TaskType::Reduce)
            .cloned()
            .collect();
        
        debug!("Starting {} reduce tasks", reduce_tasks.len());
        
        // Execute reduce tasks concurrently
        let mut handles = Vec::new();
        for task in reduce_tasks {
            let handle = self.execute_reduce_task(job, task).await;
            if let Ok(h) = handle {
                handles.push(h);
            }
        }
        
        // Wait for all reduce tasks to complete
        for handle in handles {
            handle.await.map_err(|e| {
                error!("Reduce task failed: {}", e);
                anyhow::Error::msg("Reduce task execution failed")
            })?;
        }
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_reduce_tasks += job.configuration.reduce_tasks as u64;
        }
        
        Ok(())
    }
    
    /// Execute a single reduce task
    async fn execute_reduce_task(&self, job: &Job, task: MapReduceTask) -> tokio::task::JoinHandle<Result<()>> {
        let reduce_function = job.reduce_function.clone();
        let input_partition = task.input_partition.clone();
        let task_id = task.task_id;
        
        tokio::spawn(async move {
            debug!("Executing reduce task: {}", task_id);
            
            // Deserialize intermediate data
            let intermediate_data: HashMap<Vec<u8>, Vec<Vec<u8>>> = 
                bincode::deserialize(&input_partition)?;
            
            let mut reduce_results = Vec::new();
            
            // Apply reduce function to each key-value group
            for (key, values) in intermediate_data {
                let result = reduce_function.reduce(key, values).await?;
                reduce_results.push(result);
            }
            
            // Store results
            let mut completed_tasks = self.completed_tasks.write().await;
            let mut updated_task = task.clone();
            updated_task.map_output = Some(reduce_results
                .iter()
                .flat_map(|result| vec![(Vec::new(), result.clone())])
                .collect());
            updated_task.status = TaskStatus::Completed;
            updated_task.completion_time = Some(SystemTime::now());
            completed_tasks.insert(task_id, updated_task);
            
            debug!("Reduce task {} completed", task_id);
            Ok(())
        })
    }
    
    /// Finalize a job and collect results
    async fn finalize_job(&self, job: &Job) -> Result<()> {
        info!("Finalizing job: {}", job.job_name);
        
        // Collect all reduce outputs
        let mut final_results = Vec::new();
        {
            let completed_tasks = self.completed_tasks.read().await;
            for task in completed_tasks.values() {
                if task.task_type == TaskType::Reduce && task.status == TaskStatus::Completed {
                    if let Some(outputs) = &task.map_output {
                        for (_, result) in outputs {
                            final_results.push(result.clone());
                        }
                    }
                }
            }
        }
        
        // Combine results based on output format
        let combined_result = match job.configuration.output_format {
            OutputFormat::Binary => final_results.into_iter().flatten().collect(),
            OutputFormat::Json => format!("[\"{}\"]", 
                final_results.iter()
                    .map(|r| String::from_utf8_lossy(r))
                    .collect::<Vec<_>>()
                    .join("\", \"")
            ).into_bytes(),
            OutputFormat::Csv => final_results.iter()
                .map(|r| String::from_utf8_lossy(r).to_string())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
            OutputFormat::Text => final_results.iter()
                .map(|r| String::from_utf8_lossy(r))
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
        };
        
        // Store final result
        {
            let mut job_results = self.job_results.write().await;
            job_results.insert(job.id, JobResult::Success(combined_result));
        }
        
        // Remove from active jobs
        {
            let mut active_jobs = self.active_jobs.write().await;
            active_jobs.remove(&job.id);
        }
        
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_jobs_completed += 1;
            stats.total_processed_data_bytes += job.input_data.len() as u64;
        }
        
        Ok(())
    }
    
    /// Get job status and results
    pub async fn get_job_status(&self, job_id: JobId) -> Result<Option<JobResult>> {
        let job_results = self.job_results.read().await;
        Ok(job_results.get(&job_id).cloned())
    }
    
    /// Get engine statistics
    pub async fn get_statistics(&self) -> MapReduceStatistics {
        self.statistics.read().await.clone()
    }
}

// Built-in map and reduce functions for educational purposes

/// Identity map function - passes through input data
pub struct IdentityMapFunction;

#[async_trait]
impl MapFunction for IdentityMapFunction {
    async fn map(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        Ok(vec![(key, value)])
    }
    
    fn name(&self) -> &str {
        "Identity"
    }
}

/// Identity reduce function - combines values
pub struct IdentityReduceFunction;

#[async_trait]
impl ReduceFunction for IdentityReduceFunction {
    async fn reduce(&self, key: Vec<u8>, values: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        let combined = values.concat();
        Ok(key.into_iter().chain(combined).collect())
    }
    
    fn name(&self) -> &str {
        "Identity"
    }
}

/// Word count map function
pub struct WordCountMapFunction;

#[async_trait]
impl MapFunction for WordCountMapFunction {
    async fn map(&self, _key: Vec<u8>, value: Vec<u8>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let text = String::from_utf8_lossy(&value);
        let words: Vec<&str> = text.split_whitespace().collect();
        
        let mut outputs = Vec::new();
        for word in words {
            if !word.is_empty() {
                let word_bytes = word.to_lowercase().into_bytes();
                let count_bytes = b"1".to_vec();
                outputs.push((word_bytes, count_bytes));
            }
        }
        
        Ok(outputs)
    }
    
    fn name(&self) -> &str {
        "WordCount"
    }
}

/// Word count reduce function
pub struct WordCountReduceFunction;

#[async_trait]
impl ReduceFunction for WordCountReduceFunction {
    async fn reduce(&self, key: Vec<u8>, values: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        let count: u32 = values
            .iter()
            .map(|v| String::from_utf8_lossy(v).parse::<u32>().unwrap_or(0))
            .sum();
        
        let result = format!("{}: {}", String::from_utf8_lossy(&key), count);
        Ok(result.into_bytes())
    }
    
    fn name(&self) -> &str {
        "WordCount"
    }
}

/// Utility functions for creating common MapReduce patterns
pub mod patterns {
    use super::*;
    
    /// Create a word counting job
    pub fn word_count_job(text: &str) -> Job {
        Job::new("word_count")
            .map_function(WordCountMapFunction)
            .reduce_function(WordCountReduceFunction)
            .input_data(text.as_bytes().to_vec())
    }
    
    /// Create a simple aggregation job
    pub fn sum_job(numbers: Vec<u64>) -> Job {
        Job::new("sum")
            .map_function(SumMapFunction)
            .reduce_function(SumReduceFunction)
            .input_data(bincode::serialize(&numbers).unwrap())
    }
}

/// Sum map function
pub struct SumMapFunction;

#[async_trait]
impl MapFunction for SumMapFunction {
    async fn map(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        // For sum operation, we don't need keys, just pass through the values
        Ok(vec![(key, value)])
    }
    
    fn name(&self) -> &str {
        "Sum"
    }
}

/// Sum reduce function
pub struct SumReduceFunction;

#[async_trait]
impl ReduceFunction for SumReduceFunction {
    async fn reduce(&self, _key: Vec<u8>, values: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        let sum: u64 = values
            .iter()
            .flat_map(|v| bincode::deserialize::<Vec<u64>>(v))
            .flatten()
            .sum();
        
        Ok(bincode::serialize(&sum).unwrap())
    }
    
    fn name(&self) -> &str {
        "Sum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_job_creation() {
        let job = Job::new("test_job");
        assert_eq!(job.job_name(), "test_job");
        assert!(!job.id().is_nil());
    }
    
    #[tokio::test]
    async fn test_job_task_generation() {
        let mut job = Job::new("test_job")
            .input_data(b"line1\nline2\nline3\nline4".to_vec());
        
        job.generate_tasks().unwrap();
        assert_eq!(job.task_count(), 6); // 4 map + 2 reduce (default config)
    }
    
    #[tokio::test]
    async fn test_word_count_map() {
        let mapper = WordCountMapFunction;
        let result = mapper.map(vec![], b"hello world hello".to_vec()).await.unwrap();
        
        assert_eq!(result.len(), 3);
        // Count occurrences
        let hello_count = result.iter()
            .filter(|(k, _)| k == b"hello")
            .count();
        assert_eq!(hello_count, 2);
    }
    
    #[tokio::test]
    async fn test_word_count_reduce() {
        let reducer = WordCountReduceFunction;
        let values = vec![b"1".to_vec(), b"1".to_vec(), b"1".to_vec()];
        let result = reducer.reduce(b"hello".to_vec(), values).await.unwrap();
        
        let result_str = String::from_utf8_lossy(&result);
        assert!(result_str.contains("hello: 3"));
    }
    
    #[test]
    fn test_partitioning() {
        let engine = MapReduceEngine::new(
            Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap()),
            Arc::new(DistributedScheduler::new(
                Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap())
            ).await.unwrap()),
        );
        
        let key1 = b"apple".to_vec();
        let key2 = b"banana".to_vec();
        let key3 = b"cherry".to_vec();
        
        let partition1 = engine.calculate_partition_index(&key1, 3);
        let partition2 = engine.calculate_partition_index(&key2, 3);
        let partition3 = engine.calculate_partition_index(&key3, 3);
        
        assert!(partition1 < 3);
        assert!(partition2 < 3);
        assert!(partition3 < 3);
    }
}