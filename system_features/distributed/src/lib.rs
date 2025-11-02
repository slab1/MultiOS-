//! Distributed Computing Framework for Parallel Processing Education
//!
//! This module provides a comprehensive framework for learning and experimenting
//! with distributed computing concepts, including task scheduling, load balancing,
//! MPI-like communication, MapReduce processing, and fault tolerance mechanisms.

#![allow(clippy::module_inception)]
#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use crossbeam::channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub mod prelude {
    pub use crate::cluster::{Cluster, ClusterConfig, Node, NodeId, NodeStatus};
    pub use crate::communication::{
        Message, MessageType, NetworkClient, NetworkServer, ProtocolVersion,
    };
    pub use crate::error::{DistributedError, Result};
    pub use crate::examples::*;
    pub use crate::fault_tolerance::{FaultDetector, RecoveryManager};
    pub use crate::mapreduce::{Job, JobConfig, MapReduceEngine, Task};
    pub use crate::mpi::{Barrier, CollectiveOp, Communicator, MPIError, Rank, Size};
    pub use crate::monitoring::{PerformanceMonitor, MetricsCollector};
    pub use crate::prelude_base::*;
    pub use crate::scheduler::{
        DistributedScheduler, LoadBalancer, LoadBalancingStrategy, SchedulerConfig, TaskHandle,
        TaskPriority,
    };
    pub use crate::shared_memory::{DistributedMemory, MemoryRegion, SharedValue};
}

pub mod prelude_base {
    pub use crate::common::types::*;
    pub use crate::common::{DistributionResult, NodeMetrics, ResourceInfo, TaskResult};
}

#[macro_use]
mod macros {
    /// Macro for creating distributed tasks with automatic error handling
    #[macro_export]
    macro_rules! distributed_task {
        ($name:expr, $func:expr, $($args:expr),* $(,)?) => {
            crate::Task::new($name, $func)
                $(.add_argument($args))*
        };
    }

    /// Macro for logging with distributed context
    #[macro_export]
    macro_rules! dlog {
        ($level:ident, $msg:literal $(, $key:expr => $value:expr)*) => {
            tracing::$level!(target: "distributed", $msg $(, node_id = $key = $value)*);
        };
    }

    /// Macro for creating simple distributed operations
    #[macro_export]
    macro_rules! distributed_op {
        ($name:literal => $map:expr, $reduce:expr) => {
            crate::mapreduce::Job::new($name)
                .map_function($map)
                .reduce_function($reduce)
        };
    }
}

/// Common types and data structures used throughout the framework
pub mod common {
    use super::*;
    
    /// Unique identifier for distributed nodes
    pub type NodeId = Uuid;
    
    /// Task identifier
    pub type TaskId = Uuid;
    
    /// Job identifier
    pub type JobId = Uuid;
    
    /// Time duration for operations
    pub type Duration = std::time::Duration;
    
    /// System time representation
    pub type SystemTime = std::time::SystemTime;
    
    /// Resource utilization metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResourceInfo {
        pub cpu_usage: f64,
        pub memory_usage: f64,
        pub network_bandwidth: f64,
        pub disk_io: f64,
        pub timestamp: SystemTime,
    }
    
    /// Performance metrics for tasks and jobs
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TaskResult<T> {
        pub task_id: TaskId,
        pub node_id: NodeId,
        pub result: Option<T>,
        pub execution_time: Duration,
        pub success: bool,
        pub error_message: Option<String>,
    }
    
    /// Metrics for cluster nodes
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NodeMetrics {
        pub node_id: NodeId,
        pub resource_info: ResourceInfo,
        pub task_queue_length: usize,
        pub active_connections: usize,
        pub last_heartbeat: SystemTime,
    }
    
    /// Result of load balancing operations
    #[derive(Debug, Clone)]
    pub struct DistributionResult {
        pub assigned_node: NodeId,
        pub estimated_load: f64,
        pub priority_adjustment: f64,
    }
}

/// Error types for the distributed framework
pub mod error {
    use super::*;
    
    /// Comprehensive error type for distributed operations
    #[derive(thiserror::Error, Debug)]
    pub enum DistributedError {
        #[error("Network communication error: {0}")]
        Network(#[from] std::io::Error),
        
        #[error("Serialization/Deserialization error: {0}")]
        Serialization(#[from] bincode::Error),
        
        #[error("JSON processing error: {0}")]
        Json(#[from] serde_json::Error),
        
        #[error("MPI operation error: {0}")]
        MPI(String),
        
        #[error("Task scheduling error: {0}")]
        Scheduling(String),
        
        #[error("Node communication error: {0}")]
        NodeCommunication(String),
        
        #[error("Load balancing error: {0}")]
        LoadBalancing(String),
        
        #[error("Fault detection error: {0}")]
        FaultDetection(String),
        
        #[error("Recovery operation error: {0}")]
        Recovery(String),
        
        #[error("Timeout occurred after {0:?}")]
        Timeout(Duration),
        
        #[error("Invalid configuration: {0}")]
        Configuration(String),
        
        #[error("Resource unavailable: {0}")]
        ResourceUnavailable(String),
        
        #[error("Task execution failed: {0}")]
        TaskExecution(String),
        
        #[error("Cluster operation failed: {0}")]
        ClusterOperation(String),
    }
    
    /// Result type alias for distributed operations
    pub type Result<T> = std::result::Result<T, DistributedError>;
}

/// Cluster management and node coordination
pub mod cluster;
/// Distributed task scheduling and load balancing
pub mod scheduler;
/// MPI compatibility layer for educational use
pub mod mpi;
/// MapReduce implementation for parallel processing
pub mod mapreduce;
/// Distributed shared memory system
pub mod shared_memory;
/// Fault tolerance and recovery mechanisms
pub mod fault_tolerance;
/// Network communication and message passing
pub mod communication;
/// Performance monitoring and metrics collection
pub mod monitoring;
/// Educational examples and demonstrations
pub mod examples;

/// Main framework entry point and orchestration
pub struct DistributedFramework {
    cluster: Arc<cluster::Cluster>,
    scheduler: Arc<scheduler::DistributedScheduler>,
    network: Arc<communication::NetworkServer>,
    monitoring: Arc<monitoring::PerformanceMonitor>,
    fault_tolerance: Arc<fault_tolerance::RecoveryManager>,
}

impl DistributedFramework {
    /// Initialize a new distributed computing framework
    pub async fn new(config: &crate::cluster::ClusterConfig) -> Result<Self> {
        info!("Initializing distributed computing framework");
        
        // Initialize cluster
        let cluster = Arc::new(cluster::Cluster::new(config).await?);
        
        // Initialize scheduler with cluster integration
        let scheduler = Arc::new(scheduler::DistributedScheduler::new(cluster.clone()).await?);
        
        // Initialize network communication layer
        let network = Arc::new(communication::NetworkServer::new(config).await?);
        
        // Initialize performance monitoring
        let monitoring = Arc::new(monitoring::PerformanceMonitor::new(cluster.clone()).await?);
        
        // Initialize fault tolerance system
        let fault_tolerance = Arc::new(fault_tolerance::RecoveryManager::new(
            cluster.clone(),
            scheduler.clone(),
            network.clone(),
        )?);
        
        info!("Distributed framework initialized successfully");
        
        Ok(Self {
            cluster,
            scheduler,
            network,
            monitoring,
            fault_tolerance,
        })
    }
    
    /// Submit a job to the distributed framework
    pub async fn submit_job(&self, job: mapreduce::Job) -> Result<mapreduce::JobHandle> {
        info!("Submitting job: {}", job.id());
        
        // Submit to scheduler
        let job_handle = self.scheduler.submit_job(job).await?;
        
        // Start monitoring
        self.monitoring.track_job(job_handle.id()).await?;
        
        Ok(job_handle)
    }
    
    /// Get cluster status and metrics
    pub async fn cluster_status(&self) -> Result<monitoring::ClusterStatus> {
        self.monitoring.get_cluster_status().await
    }
    
    /// Run a distributed computation example
    pub async fn run_example(
        &self,
        example_name: &str,
        config: &str,
    ) -> Result<mapreduce::JobResult> {
        info!("Running distributed example: {}", example_name);
        
        let example = examples::get_example(example_name)
            .ok_or_else(|| crate::error::DistributedError::Configuration(
                format!("Example '{}' not found", example_name)
            ))?;
        
        let job = example.generate_job(config).await?;
        let job_handle = self.submit_job(job).await?;
        
        // Wait for completion
        job_handle.result().await.map_err(Into::into)
    }
    
    /// Start the framework with full initialization
    pub async fn start(&self) -> Result<()> {
        info!("Starting distributed framework services");
        
        // Start cluster discovery and coordination
        self.cluster.start().await?;
        
        // Start network communication
        self.network.start().await?;
        
        // Start performance monitoring
        self.monitoring.start().await?;
        
        // Start fault tolerance monitoring
        self.fault_tolerance.start().await?;
        
        info!("Distributed framework started successfully");
        
        Ok(())
    }
    
    /// Gracefully shutdown the framework
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down distributed framework");
        
        // Stop fault tolerance monitoring
        self.fault_tolerance.stop().await?;
        
        // Stop performance monitoring
        self.monitoring.stop().await?;
        
        // Stop network communication
        self.network.stop().await?;
        
        // Stop cluster coordination
        self.cluster.stop().await?;
        
        info!("Distributed framework shutdown complete");
        
        Ok(())
    }
}

impl Drop for DistributedFramework {
    fn drop(&mut self) {
        // Graceful shutdown on drop
        let _ = tokio::runtime::Handle::current().block_on(self.shutdown());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::ClusterConfig;
    use crate::examples::basic_parallel::BasicParallelExample;
    
    #[tokio::test]
    async fn test_framework_initialization() {
        let config = ClusterConfig::default();
        let framework = DistributedFramework::new(&config).await;
        assert!(framework.is_ok());
    }
    
    #[tokio::test]
    async fn test_basic_job_submission() {
        let config = ClusterConfig::default();
        let framework = DistributedFramework::new(&config).await.unwrap();
        
        // Create a simple example job
        let example = BasicParallelExample::new();
        let job = example.generate_job("test").await.unwrap();
        
        let job_handle = framework.submit_job(job).await;
        assert!(job_handle.is_ok());
    }
}