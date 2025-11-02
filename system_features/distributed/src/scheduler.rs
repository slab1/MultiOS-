//! Distributed task scheduling and load balancing
//!
//! This module provides advanced task scheduling algorithms, load balancing
//! strategies, and priority management for distributed computing workloads.

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cluster::{Cluster, NodeId, NodeStatus};
use crate::common::{TaskId, TaskResult, NodeMetrics, DistributionResult, ResourceInfo};

/// Task priorities for scheduling decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task representation in the scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: TaskId,
    pub name: String,
    pub priority: TaskPriority,
    pub estimated_duration: Duration,
    pub resource_requirements: ResourceRequirements,
    pub dependencies: Vec<TaskId>,
    pub submission_time: SystemTime,
    pub timeout: Option<Duration>,
}

/// Resource requirements for task execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_cpu_cores: usize,
    pub min_memory_gb: u64,
    pub network_bandwidth_mbps: u64,
    pub storage_gb: Option<u64>,
}

/// Task execution handle for monitoring and control
#[derive(Debug)]
pub struct TaskHandle {
    task_id: TaskId,
    job_id: Uuid,
    execution_tx: mpsc::UnboundedSender<TaskExecutionEvent>,
    completion_rx: Option<oneshot::Receiver<TaskResult<Vec<u8>>>>,
    result_tx: Option<oneshot::Sender<TaskResult<Vec<u8>>>>,
}

/// Task execution events
#[derive(Debug)]
pub enum TaskExecutionEvent {
    /// Task has started execution
    Started {
        node_id: NodeId,
        start_time: SystemTime,
    },
    /// Task has been completed
    Completed {
        result: TaskResult<Vec<u8>>,
    },
    /// Task execution has failed
    Failed {
        error: String,
        timestamp: SystemTime,
    },
    /// Task has been cancelled
    Cancelled {
        reason: String,
        timestamp: SystemTime,
    },
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: usize,
    pub scheduling_interval: Duration,
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub priority_queues: bool,
    pub task_timeout_default: Duration,
    pub health_check_interval: Duration,
    pub backpressure_threshold: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1000,
            scheduling_interval: Duration::from_millis(100),
            load_balancing_strategy: LoadBalancingStrategy::LeastLoaded,
            priority_queues: true,
            task_timeout_default: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            backpressure_threshold: 0.8,
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin assignment
    RoundRobin,
    /// Assign to least loaded node
    LeastLoaded,
    /// Resource-aware assignment
    ResourceAware,
    /// Random assignment
    Random,
    /// Greedy assignment based on task characteristics
    Greedy,
    /// Dynamic strategy that adapts to performance
    Adaptive,
}

/// Main distributed scheduler implementation
pub struct DistributedScheduler {
    config: SchedulerConfig,
    cluster: Arc<Cluster>,
    
    // Task queues
    priority_queues: Arc<RwLock<HashMap<TaskPriority, VecDeque<ScheduledTask>>>>,
    round_robin_index: Arc<RwLock<usize>>,
    
    // Active tasks
    active_tasks: Arc<RwLock<HashMap<TaskId, TaskHandle>>>,
    task_assignments: Arc<RwLock<HashMap<TaskId, NodeId>>>,
    
    // Load balancing
    load_balancer: Arc<dyn LoadBalancer + Send + Sync>,
    node_load_scores: Arc<RwLock<HashMap<NodeId, f64>>>,
    
    // Statistics and monitoring
    scheduling_stats: Arc<RwLock<SchedulingStatistics>>,
    
    // Control channels
    scheduler_tx: mpsc::UnboundedSender<SchedulerCommand>,
    scheduler_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
}

/// Scheduler control commands
#[derive(Debug)]
pub enum SchedulerCommand {
    /// Submit a new task for scheduling
    SubmitTask {
        task: ScheduledTask,
        response: oneshot::Sender<Result<TaskHandle>>,
    },
    /// Cancel an existing task
    CancelTask {
        task_id: TaskId,
        reason: String,
    },
    /// Get scheduler status
    GetStatus {
        response: oneshot::Sender<SchedulerStatus>,
    },
    /// Get task status
    GetTaskStatus {
        task_id: TaskId,
        response: oneshot::Sender<Option<TaskStatus>>,
    },
    /// Update load balancing strategy
    UpdateStrategy {
        strategy: LoadBalancingStrategy,
    },
    /// Force task reassignment
    ReassignTask {
        task_id: TaskId,
        node_id: NodeId,
    },
}

/// Scheduler status information
#[derive(Debug, Clone)]
pub struct SchedulerStatus {
    pub total_queued_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_scheduling_time: Duration,
    pub load_distribution: HashMap<NodeId, usize>,
    pub current_strategy: LoadBalancingStrategy,
    pub cluster_utilization: f64,
}

/// Task status information
#[derive(Debug, Clone)]
pub struct TaskStatus {
    pub task_id: TaskId,
    pub status: TaskExecutionStatus,
    pub assigned_node: Option<NodeId>,
    pub submission_time: SystemTime,
    pub start_time: Option<SystemTime>,
    pub completion_time: Option<SystemTime>,
    pub progress: f64,
    pub current_stage: String,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExecutionStatus {
    Queued,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Scheduling statistics
#[derive(Debug, Clone)]
pub struct SchedulingStatistics {
    pub total_scheduled_tasks: u64,
    pub total_scheduled_time: Duration,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub cancelled_tasks: u64,
    pub average_task_duration: Duration,
    pub scheduling_overhead: Duration,
    pub load_balance_variance: f64,
}

/// Load balancing trait for different strategies
#[async_trait]
pub trait LoadBalancer {
    /// Select the optimal node for a given task
    fn select_node(
        &self,
        task: &ScheduledTask,
        cluster: &Cluster,
        node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId>;
    
    /// Update load information after task assignment
    fn update_load(
        &self,
        node_id: NodeId,
        task: &ScheduledTask,
        load_scores: &mut HashMap<NodeId, f64>,
    );
    
    /// Get strategy name for reporting
    fn name(&self) -> &str;
}

impl DistributedScheduler {
    /// Create a new distributed scheduler
    pub async fn new(cluster: Arc<Cluster>) -> Result<Self> {
        let config = SchedulerConfig::default();
        Self::new_with_config(cluster, config).await
    }
    
    /// Create a new scheduler with custom configuration
    pub async fn new_with_config(cluster: Arc<Cluster>, config: SchedulerConfig) -> Result<Self> {
        info!("Creating distributed scheduler with strategy: {:?}", config.load_balancing_strategy);
        
        let (scheduler_tx, scheduler_rx) = mpsc::unbounded_channel();
        
        let priority_queues = Arc::new(RwLock::new(HashMap::new()));
        for priority in [TaskPriority::Critical, TaskPriority::High, TaskPriority::Normal, TaskPriority::Low] {
            priority_queues.write().await.insert(priority, VecDeque::new());
        }
        
        let load_balancer: Arc<dyn LoadBalancer + Send + Sync> = match config.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => Arc::new(RoundRobinBalancer::new()),
            LoadBalancingStrategy::LeastLoaded => Arc::new(LeastLoadedBalancer::new()),
            LoadBalancingStrategy::ResourceAware => Arc::new(ResourceAwareBalancer::new()),
            LoadBalancingStrategy::Random => Arc::new(RandomBalancer::new()),
            LoadBalancingStrategy::Greedy => Arc::new(GreedyBalancer::new()),
            LoadBalancingStrategy::Adaptive => Arc::new(AdaptiveBalancer::new()),
        };
        
        Ok(Self {
            config: config.clone(),
            cluster,
            priority_queues,
            round_robin_index: Arc::new(RwLock::new(0)),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_assignments: Arc::new(RwLock::new(HashMap::new())),
            load_balancer,
            node_load_scores: Arc::new(RwLock::new(HashMap::new())),
            scheduling_stats: Arc::new(RwLock::new(SchedulingStatistics::default())),
            scheduler_tx,
            scheduler_rx,
        })
    }
    
    /// Start the scheduler service
    pub async fn start(&self) -> Result<()> {
        info!("Starting distributed scheduler");
        
        // Start scheduling loop
        let scheduler = self.clone();
        tokio::spawn(async move {
            scheduler.scheduling_loop().await;
        });
        
        // Start health monitoring
        let scheduler = self.clone();
        tokio::spawn(async move {
            scheduler.health_monitoring_loop().await;
        });
        
        info!("Distributed scheduler started");
        Ok(())
    }
    
    /// Submit a task for scheduling
    pub async fn submit_task(&self, task: ScheduledTask) -> Result<TaskHandle> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.scheduler_tx.send(SchedulerCommand::SubmitTask {
            task,
            response: response_tx,
        })?;
        
        let handle = response_rx.await??;
        Ok(handle)
    }
    
    /// Cancel a task
    pub async fn cancel_task(&self, task_id: TaskId, reason: String) -> Result<()> {
        self.scheduler_tx.send(SchedulerCommand::CancelTask {
            task_id,
            reason,
        })?;
        
        Ok(())
    }
    
    /// Submit a job to the scheduler
    pub async fn submit_job(&self, job: crate::mapreduce::Job) -> Result<crate::mapreduce::JobHandle> {
        debug!("Submitting job {} with {} tasks", job.id(), job.task_count());
        
        let job_id = job.id();
        let mut task_handles = Vec::new();
        
        // Convert job tasks to scheduled tasks and submit them
        for (i, task) in job.tasks().into_iter().enumerate() {
            let scheduled_task = ScheduledTask {
                id: Uuid::new_v4(),
                name: format!("{}_task_{}", job.job_name(), i),
                priority: TaskPriority::Normal,
                estimated_duration: Duration::from_millis(100),
                resource_requirements: ResourceRequirements {
                    min_cpu_cores: 1,
                    min_memory_gb: 1,
                    network_bandwidth_mbps: 10,
                    storage_gb: None,
                },
                dependencies: vec![],
                submission_time: SystemTime::now(),
                timeout: Some(Duration::from_secs(60)),
            };
            
            let handle = self.submit_task(scheduled_task).await?;
            task_handles.push(handle);
        }
        
        Ok(crate::mapreduce::JobHandle::new(job_id, task_handles))
    }
    
    /// Get current scheduler status
    pub async fn get_status(&self) -> Result<SchedulerStatus> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.scheduler_tx.send(SchedulerCommand::GetStatus {
            response: response_tx,
        })?;
        
        let status = response_rx.await?;
        Ok(status)
    }
    
    /// Main scheduling loop
    async fn scheduling_loop(&self) {
        let mut interval = interval(self.config.scheduling_interval);
        
        loop {
            interval.tick().await;
            
            // Process pending commands
            if let Ok(command) = self.scheduler_rx.try_recv() {
                if let Err(e) = self.handle_command(command).await {
                    error!("Error handling scheduler command: {}", e);
                }
            }
            
            // Schedule ready tasks
            if let Err(e) = self.schedule_ready_tasks().await {
                error!("Error scheduling tasks: {}", e);
            }
        }
    }
    
    /// Health monitoring loop
    async fn health_monitoring_loop(&self) {
        let mut interval = interval(self.config.health_check_interval);
        
        loop {
            interval.tick().await;
            
            // Check for failed tasks and reassign if necessary
            if let Err(e) = self.check_task_health().await {
                error!("Error in task health check: {}", e);
            }
            
            // Update load scores
            if let Err(e) = self.update_load_scores().await {
                error!("Error updating load scores: {}", e);
            }
        }
    }
    
    /// Handle scheduler commands
    async fn handle_command(&self, command: SchedulerCommand) -> Result<()> {
        match command {
            SchedulerCommand::SubmitTask { task, response } => {
                let handle = self.submit_task_internal(task).await?;
                let _ = response.send(Ok(handle));
            }
            SchedulerCommand::CancelTask { task_id, reason } => {
                self.cancel_task_internal(task_id, reason).await?;
            }
            SchedulerCommand::GetStatus { response } => {
                let status = self.get_status_internal().await?;
                let _ = response.send(status);
            }
            SchedulerCommand::GetTaskStatus { task_id, response } => {
                let status = self.get_task_status_internal(task_id).await?;
                let _ = response.send(status);
            }
            SchedulerCommand::UpdateStrategy { strategy } => {
                self.update_strategy(strategy).await?;
            }
            SchedulerCommand::ReassignTask { task_id, node_id } => {
                self.reassign_task_internal(task_id, node_id).await?;
            }
        }
        Ok(())
    }
    
    /// Internal task submission
    async fn submit_task_internal(&self, task: ScheduledTask) -> Result<TaskHandle> {
        debug!("Submitting task {} with priority {:?}", task.id, task.priority);
        
        // Add to appropriate priority queue
        {
            let mut queues = self.priority_queues.write().await;
            if let Some(queue) = queues.get_mut(&task.priority) {
                queue.push_back(task);
            }
        }
        
        // Create task handle
        let (execution_tx, _) = mpsc::unbounded_channel();
        let task_handle = TaskHandle {
            task_id: task.id,
            job_id: Uuid::new_v4(), // Would be derived from job
            execution_tx,
            completion_rx: None,
            result_tx: None,
        };
        
        // Track active task
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task.id, task_handle.clone());
        }
        
        Ok(task_handle)
    }
    
    /// Internal task cancellation
    async fn cancel_task_internal(&self, task_id: TaskId, reason: String) -> Result<()> {
        debug!("Cancelling task {}: {}", task_id, reason);
        
        // Remove from active tasks
        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.remove(&task_id);
        }
        
        // Remove from queue if still queued
        {
            let mut queues = self.priority_queues.write().await;
            for queue in queues.values_mut() {
                if let Some(pos) = queue.iter().position(|t| t.id == task_id) {
                    queue.remove(pos);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Internal status retrieval
    async fn get_status_internal(&self) -> Result<SchedulerStatus> {
        let queued_tasks = {
            let queues = self.priority_queues.read().await;
            queues.values().map(|q| q.len()).sum()
        };
        
        let active_tasks = {
            let active_tasks = self.active_tasks.read().await;
            active_tasks.len()
        };
        
        let load_distribution = {
            let assignments = self.task_assignments.read().await;
            let mut distribution = HashMap::new();
            for (_, node_id) in assignments.iter() {
                *distribution.entry(*node_id).or_insert(0) += 1;
            }
            distribution
        };
        
        let stats = self.scheduling_stats.read().await;
        
        Ok(SchedulerStatus {
            total_queued_tasks: queued_tasks,
            active_tasks,
            completed_tasks: stats.successful_tasks,
            failed_tasks: stats.failed_tasks,
            average_scheduling_time: stats.average_task_duration,
            load_distribution,
            current_strategy: self.config.load_balancing_strategy,
            cluster_utilization: self.calculate_cluster_utilization().await,
        })
    }
    
    /// Schedule ready tasks to available nodes
    async fn schedule_ready_tasks(&self) -> Result<()> {
        let available_nodes = self.get_available_nodes().await?;
        
        if available_nodes.is_empty() {
            return Ok(());
        }
        
        let mut scheduled_count = 0;
        let max_concurrent = self.config.max_concurrent_tasks;
        
        // Process each priority level
        for priority in [TaskPriority::Critical, TaskPriority::High, TaskPriority::Normal, TaskPriority::Low] {
            if scheduled_count >= max_concurrent {
                break;
            }
            
            let mut tasks_to_schedule = Vec::new();
            
            // Get ready tasks from this priority level
            {
                let mut queues = self.priority_queues.write().await;
                if let Some(queue) = queues.get_mut(&priority) {
                    // Take tasks that can be scheduled
                    while scheduled_count < max_concurrent && !queue.is_empty() {
                        if let Some(task) = queue.pop_front() {
                            tasks_to_schedule.push(task);
                            scheduled_count += 1;
                        }
                    }
                }
            }
            
            // Schedule the tasks
            for task in tasks_to_schedule {
                if let Err(e) = self.schedule_single_task(task, &available_nodes).await {
                    error!("Failed to schedule task: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Schedule a single task to an optimal node
    async fn schedule_single_task(&self, task: ScheduledTask, available_nodes: &[NodeId]) -> Result<()> {
        if available_nodes.is_empty() {
            return Ok(());
        }
        
        let start_time = Instant::now();
        
        // Get current load scores
        let node_loads = self.node_load_scores.read().await.clone();
        
        // Use load balancer to select optimal node
        let selected_node = self.load_balancer.select_node(
            &task,
            &self.cluster,
            &node_loads,
            available_nodes,
        )?;
        
        // Update load score
        self.load_balancer.update_load(
            selected_node,
            &task,
            &mut self.node_load_scores.write().await,
        );
        
        // Track assignment
        {
            let mut assignments = self.task_assignments.write().await;
            assignments.insert(task.id, selected_node);
        }
        
        // Update statistics
        {
            let mut stats = self.scheduling_stats.write().await;
            stats.total_scheduled_tasks += 1;
            stats.total_scheduled_time += start_time.elapsed();
            stats.scheduling_overhead += start_time.elapsed();
        }
        
        info!("Task {} assigned to node {} (strategy: {})", 
              task.id, selected_node, self.load_balancer.name());
        
        Ok(())
    }
    
    /// Get list of available nodes for task assignment
    async fn get_available_nodes(&self) -> Result<Vec<NodeId>> {
        let cluster_status = self.cluster.get_status().await?;
        let available_nodes = self.cluster.get_nodes_by_status(NodeStatus::Active).await?;
        
        Ok(available_nodes.into_iter().map(|n| n.id).collect())
    }
    
    /// Calculate overall cluster utilization
    async fn calculate_cluster_utilization(&self) -> f64 {
        let assignments = self.task_assignments.read().await;
        let active_tasks = assignments.len();
        let max_tasks = self.config.max_concurrent_tasks;
        
        active_tasks as f64 / max_tasks as f64
    }
    
    /// Check task health and handle failures
    async fn check_task_health(&self) -> Result<()> {
        let active_tasks = self.active_tasks.read().await.clone();
        
        for (task_id, handle) in active_tasks.iter() {
            // Check for timeout
            // Implementation would check task execution status
            // and handle timeouts or failures
            
            // This is a placeholder for timeout/failure detection
            let _ = handle;
        }
        
        Ok(())
    }
    
    /// Update load scores for all nodes
    async fn update_load_scores(&self) -> Result<()> {
        let cluster_nodes = self.cluster.get_nodes_by_status(NodeStatus::Active).await?;
        let assignments = self.task_assignments.read().await;
        
        let mut scores = self.node_load_scores.write().await;
        scores.clear();
        
        for node in cluster_nodes {
            let task_count = assignments.values().filter(|&&id| id == node.id).count();
            scores.insert(node.id, task_count as f64);
        }
        
        Ok(())
    }
    
    /// Update load balancing strategy
    async fn update_strategy(&self, strategy: LoadBalancingStrategy) -> Result<()> {
        self.config.load_balancing_strategy = strategy.clone();
        info!("Load balancing strategy updated to: {:?}", strategy);
        Ok(())
    }
    
    /// Get task status
    async fn get_task_status_internal(&self, task_id: TaskId) -> Result<Option<TaskStatus>> {
        let active_tasks = self.active_tasks.read().await;
        
        if let Some(handle) = active_tasks.get(&task_id) {
            let assignment = self.task_assignments.read().await.get(&task_id).copied();
            
            Ok(Some(TaskStatus {
                task_id,
                status: TaskExecutionStatus::Assigned,
                assigned_node: assignment,
                submission_time: SystemTime::now(),
                start_time: None,
                completion_time: None,
                progress: 0.0,
                current_stage: "Scheduled".to_string(),
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Internal task reassignment
    async fn reassign_task_internal(&self, task_id: TaskId, node_id: NodeId) -> Result<()> {
        {
            let mut assignments = self.task_assignments.write().await;
            assignments.insert(task_id, node_id);
        }
        
        info!("Task {} reassigned to node {}", task_id, node_id);
        Ok(())
    }
}

impl Clone for DistributedScheduler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cluster: self.cluster.clone(),
            priority_queues: self.priority_queues.clone(),
            round_robin_index: self.round_robin_index.clone(),
            active_tasks: self.active_tasks.clone(),
            task_assignments: self.task_assignments.clone(),
            load_balancer: self.load_balancer.clone(),
            node_load_scores: self.node_load_scores.clone(),
            scheduling_stats: self.scheduling_stats.clone(),
            scheduler_tx: self.scheduler_tx.clone(),
            scheduler_rx: self.scheduler_rx.clone(),
        }
    }
}

// Load balancing implementations

/// Round-robin load balancer
pub struct RoundRobinBalancer {
    index: usize,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        Self { index: 0 }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinBalancer {
    fn select_node(
        &self,
        _task: &ScheduledTask,
        _cluster: &Cluster,
        _node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId> {
        if available_nodes.is_empty() {
            return Err(crate::error::DistributedError::ResourceUnavailable(
                "No available nodes".to_string()
            ));
        }
        
        let node_id = available_nodes[self.index % available_nodes.len()];
        Ok(node_id)
    }
    
    fn update_load(
        &self,
        _node_id: NodeId,
        _task: &ScheduledTask,
        _load_scores: &mut HashMap<NodeId, f64>,
    ) {
        // Round-robin doesn't update loads
    }
    
    fn name(&self) -> &str {
        "Round Robin"
    }
}

/// Least loaded load balancer
pub struct LeastLoadedBalancer;

impl LeastLoadedBalancer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancer for LeastLoadedBalancer {
    fn select_node(
        &self,
        _task: &ScheduledTask,
        _cluster: &Cluster,
        node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId> {
        if available_nodes.is_empty() {
            return Err(crate::error::DistributedError::ResourceUnavailable(
                "No available nodes".to_string()
            ));
        }
        
        let mut best_node = available_nodes[0];
        let mut best_load = f64::INFINITY;
        
        for &node_id in available_nodes {
            let load = node_loads.get(&node_id).unwrap_or(&0.0);
            if *load < best_load {
                best_load = *load;
                best_node = node_id;
            }
        }
        
        Ok(best_node)
    }
    
    fn update_load(
        &self,
        node_id: NodeId,
        task: &ScheduledTask,
        load_scores: &mut HashMap<NodeId, f64>,
    ) {
        let current_load = load_scores.get(&node_id).unwrap_or(&0.0);
        let task_load = task.estimated_duration.as_secs_f64() / 3600.0; // Hour-based load
        load_scores.insert(node_id, current_load + task_load);
    }
    
    fn name(&self) -> &str {
        "Least Loaded"
    }
}

/// Resource-aware load balancer
pub struct ResourceAwareBalancer;

impl ResourceAwareBalancer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancer for ResourceAwareBalancer {
    fn select_node(
        &self,
        task: &ScheduledTask,
        _cluster: &Cluster,
        _node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId> {
        if available_nodes.is_empty() {
            return Err(crate::error::DistributedError::ResourceUnavailable(
                "No available nodes".to_string()
            ));
        }
        
        // This is a simplified resource-aware selection
        // In practice, would query cluster for node capabilities
        let node_id = available_nodes[0]; // Simplified
        
        Ok(node_id)
    }
    
    fn update_load(
        &self,
        node_id: NodeId,
        task: &ScheduledTask,
        load_scores: &mut HashMap<NodeId, f64>,
    ) {
        let resource_cost = (task.resource_requirements.min_cpu_cores as f64 * 0.6 +
                           task.resource_requirements.min_memory_gb as f64 * 0.4);
        
        let current_load = load_scores.get(&node_id).unwrap_or(&0.0);
        load_scores.insert(node_id, current_load + resource_cost);
    }
    
    fn name(&self) -> &str {
        "Resource Aware"
    }
}

/// Random load balancer
pub struct RandomBalancer;

impl RandomBalancer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancer for RandomBalancer {
    fn select_node(
        &self,
        _task: &ScheduledTask,
        _cluster: &Cluster,
        _node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId> {
        use rand::Rng;
        
        if available_nodes.is_empty() {
            return Err(crate::error::DistributedError::ResourceUnavailable(
                "No available nodes".to_string()
            ));
        }
        
        let mut rng = rand::thread_rng();
        let node_id = available_nodes[rng.gen_range(0..available_nodes.len())];
        Ok(*node_id)
    }
    
    fn update_load(
        &self,
        _node_id: NodeId,
        _task: &ScheduledTask,
        _load_scores: &mut HashMap<NodeId, f64>,
    ) {
        // Random doesn't maintain load scores
    }
    
    fn name(&self) -> &str {
        "Random"
    }
}

/// Greedy load balancer
pub struct GreedyBalancer;

impl GreedyBalancer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancer for GreedyBalancer {
    fn select_node(
        &self,
        task: &ScheduledTask,
        _cluster: &Cluster,
        node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId> {
        if available_nodes.is_empty() {
            return Err(crate::error::DistributedError::ResourceUnavailable(
                "No available nodes".to_string()
            ));
        }
        
        let mut best_node = available_nodes[0];
        let mut best_score = f64::INFINITY;
        
        for &node_id in available_nodes {
            let load = node_loads.get(&node_id).unwrap_or(&0.0);
            let task_cost = task.estimated_duration.as_secs_f64();
            let total_cost = load + task_cost;
            
            if total_cost < best_score {
                best_score = total_cost;
                best_node = node_id;
            }
        }
        
        Ok(best_node)
    }
    
    fn update_load(
        &self,
        node_id: NodeId,
        task: &ScheduledTask,
        load_scores: &mut HashMap<NodeId, f64>,
    ) {
        let task_cost = task.estimated_duration.as_secs_f64();
        let current_load = load_scores.get(&node_id).unwrap_or(&0.0);
        load_scores.insert(node_id, current_load + task_cost);
    }
    
    fn name(&self) -> &str {
        "Greedy"
    }
}

/// Adaptive load balancer (simplified)
pub struct AdaptiveBalancer;

impl AdaptiveBalancer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoadBalancer for AdaptiveBalancer {
    fn select_node(
        &self,
        task: &ScheduledTask,
        _cluster: &Cluster,
        node_loads: &HashMap<NodeId, f64>,
        available_nodes: &[NodeId],
    ) -> Result<NodeId> {
        if available_nodes.is_empty() {
            return Err(crate::error::DistributedError::ResourceUnavailable(
                "No available nodes".to_string()
            ));
        }
        
        // Adaptive strategy - choose based on task characteristics
        let task_complexity = task.estimated_duration.as_secs_f64();
        
        let mut best_node = available_nodes[0];
        let mut best_score = f64::INFINITY;
        
        for &node_id in available_nodes {
            let load = node_loads.get(&node_id).unwrap_or(&0.0);
            
            // Adaptive scoring based on task characteristics
            let adaptive_score = if task_complexity > 30.0 {
                // For complex tasks, prefer less loaded nodes
                load * 2.0 + task_complexity
            } else {
                // For simple tasks, balance load
                load + task_complexity * 0.5
            };
            
            if adaptive_score < best_score {
                best_score = adaptive_score;
                best_node = node_id;
            }
        }
        
        Ok(best_node)
    }
    
    fn update_load(
        &self,
        node_id: NodeId,
        task: &ScheduledTask,
        load_scores: &mut HashMap<NodeId, f64>,
    ) {
        let task_complexity = task.estimated_duration.as_secs_f64();
        let current_load = load_scores.get(&node_id).unwrap_or(&0.0);
        
        // Adaptive load update
        let load_increment = if task_complexity > 30.0 {
            task_complexity * 1.5
        } else {
            task_complexity
        };
        
        load_scores.insert(node_id, current_load + load_increment);
    }
    
    fn name(&self) -> &str {
        "Adaptive"
    }
}

impl Default for SchedulingStatistics {
    fn default() -> Self {
        Self {
            total_scheduled_tasks: 0,
            total_scheduled_time: Duration::from_secs(0),
            successful_tasks: 0,
            failed_tasks: 0,
            cancelled_tasks: 0,
            average_task_duration: Duration::from_secs(0),
            scheduling_overhead: Duration::from_secs(0),
            load_balance_variance: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = crate::cluster::ClusterConfig::default();
        let cluster = Arc::new(crate::cluster::Cluster::new(&config).await.unwrap());
        let scheduler = DistributedScheduler::new(cluster).await;
        assert!(scheduler.is_ok());
    }
    
    #[tokio::test]
    async fn test_task_submission() {
        let config = crate::cluster::ClusterConfig::default();
        let cluster = Arc::new(crate::cluster::Cluster::new(&config).await.unwrap());
        let scheduler = Arc::new(DistributedScheduler::new(cluster).await.unwrap());
        
        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: "test_task".to_string(),
            priority: TaskPriority::Normal,
            estimated_duration: Duration::from_secs(1),
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 1,
                min_memory_gb: 1,
                network_bandwidth_mbps: 10,
                storage_gb: None,
            },
            dependencies: vec![],
            submission_time: SystemTime::now(),
            timeout: Some(Duration::from_secs(5)),
        };
        
        let handle = scheduler.submit_task(task).await;
        assert!(handle.is_ok());
    }
    
    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }
}