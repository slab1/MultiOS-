//! Performance monitoring and metrics collection
//!
//! This module provides comprehensive performance monitoring, metrics collection,
//! and visualization capabilities for distributed computing workloads.

use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cluster::{Cluster, NodeId, NodeStatus};

/// Performance metrics collection interval
pub type CollectionInterval = Duration;

/// Unique metric identifier
pub type MetricId = String;

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: SystemTime,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

/// Performance metric definition
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub metric_id: MetricId,
    pub name: String,
    pub metric_type: MetricType,
    pub unit: String,
    pub description: String,
    pub collection_interval: CollectionInterval,
    pub aggregation: AggregationFunction,
    pub tags: HashMap<String, String>,
}

/// Types of performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter that always increases
    Counter,
    /// Gauge that can go up and down
    Gauge,
    /// Histogram for distribution tracking
    Histogram,
    /// Timer for measuring duration
    Timer,
    /// Rate-based metric
    Rate,
}

/// Aggregation functions for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationFunction {
    Sum,
    Average,
    Min,
    Max,
    Count,
    Percentile(f64), // e.g., 95th percentile
}

/// Cluster-wide performance status
#[derive(Debug, Clone)]
pub struct ClusterStatus {
    pub cluster_id: Uuid,
    pub timestamp: SystemTime,
    pub overall_health: f64,
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_cpu_cores: usize,
    pub total_memory_gb: u64,
    pub average_cpu_usage: f64,
    pub average_memory_usage: f64,
    pub network_throughput_mbps: f64,
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub throughput_tasks_per_second: f64,
    pub latency_p95_ms: f64,
    pub error_rate: f64,
    pub cluster_efficiency: f64,
}

/// Node-specific performance metrics
#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub node_id: NodeId,
    pub timestamp: SystemTime,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkIO,
    pub active_tasks: usize,
    pub task_queue_length: usize,
    pub average_task_duration: Duration,
    pub throughput_tasks_per_second: f64,
    pub error_rate: f64,
    pub resource_efficiency: f64,
}

/// Network I/O statistics
#[derive(Debug, Clone)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u64,
    pub throughput_mbps: f64,
}

/// Task execution metrics
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub task_id: Uuid,
    pub node_id: NodeId,
    pub submission_time: SystemTime,
    pub start_time: Option<SystemTime>,
    pub completion_time: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub status: TaskExecutionStatus,
    pub resource_consumption: ResourceConsumption,
    pub dependencies: Vec<Uuid>,
}

/// Task execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Resource consumption by tasks
#[derive(Debug, Clone)]
pub struct ResourceConsumption {
    pub cpu_time_seconds: f64,
    pub memory_bytes: u64,
    pub network_bytes: u64,
    pub disk_bytes: u64,
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_id: Uuid,
    pub severity: AlertSeverity,
    pub metric_id: MetricId,
    pub node_id: Option<NodeId>,
    pub threshold_value: f64,
    pub current_value: f64,
    pub message: String,
    pub timestamp: SystemTime,
    pub acknowledged: bool,
    pub resolved_at: Option<SystemTime>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub collection_interval: CollectionInterval,
    pub retention_period: Duration,
    pub alert_thresholds: HashMap<MetricId, AlertThreshold>,
    pub enable_real_time_monitoring: bool,
    pub enable_historical_analysis: bool,
    pub max_data_points_per_metric: usize,
    pub enable_cluster_aggregation: bool,
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThreshold {
    pub warning_threshold: f64,
    pub critical_threshold: f64,
    pub comparison: ThresholdComparison,
    pub enabled: bool,
}

/// Threshold comparison types
#[derive(Debug, Clone, Copy)]
pub enum ThresholdComparison {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Main performance monitor service
pub struct PerformanceMonitor {
    cluster: Arc<Cluster>,
    config: MonitoringConfig,
    
    // Metric storage
    metrics: Arc<RwLock<HashMap<MetricId, VecDeque<DataPoint>>>>,
    node_metrics: Arc<RwLock<HashMap<NodeId, NodeMetrics>>>,
    task_metrics: Arc<RwLock<HashMap<Uuid, TaskMetrics>>>,
    
    // Performance definitions
    performance_metrics: Arc<RwLock<HashMap<MetricId, PerformanceMetric>>>,
    
    // Alerts
    active_alerts: Arc<RwLock<HashMap<Uuid, PerformanceAlert>>>,
    alert_history: Arc<RwLock<Vec<PerformanceAlert>>>,
    
    // Monitoring state
    is_monitoring: Arc<RwLock<bool>>,
    last_collection: Arc<RwLock<SystemTime>>,
    
    // Statistics
    monitoring_statistics: Arc<RwLock<MonitoringStatistics>>,
    
    // Event streams
    metrics_event_tx: broadcast::Sender<MetricsEvent>,
    alert_event_tx: broadcast::Sender<AlertEvent>,
}

/// Metrics collection events
#[derive(Debug, Clone)]
pub enum MetricsEvent {
    MetricCollected {
        metric_id: MetricId,
        data_point: DataPoint,
        node_id: Option<NodeId>,
    },
    MetricsAggregated {
        timestamp: SystemTime,
        cluster_metrics: ClusterStatus,
    },
    CollectionCycleCompleted {
        cycle_duration: Duration,
        metrics_collected: usize,
    },
}

/// Alert events
#[derive(Debug, Clone)]
pub enum AlertEvent {
    AlertTriggered {
        alert: PerformanceAlert,
    },
    AlertResolved {
        alert_id: Uuid,
        resolution_time: SystemTime,
    },
    AlertAcknowledged {
        alert_id: Uuid,
        acknowledged_by: String,
    },
}

/// Monitoring service statistics
#[derive(Debug, Clone, Default)]
pub struct MonitoringStatistics {
    pub total_metrics_collected: u64,
    pub total_alerts_generated: u64,
    pub alerts_resolved: u64,
    pub average_collection_time: Duration,
    pub monitoring_uptime: Duration,
    pub data_retention_rate: f64,
    pub alert_response_time: Duration,
    pub false_positive_rate: f64,
}

/// Real-time performance dashboard
#[derive(Debug)]
pub struct PerformanceDashboard {
    monitor: Arc<PerformanceMonitor>,
    update_interval: Duration,
    max_historical_points: usize,
}

/// Cluster analysis and insights
#[derive(Debug, Clone)]
pub struct ClusterAnalysis {
    pub analysis_id: Uuid,
    pub timestamp: SystemTime,
    pub cluster_status: ClusterStatus,
    pub performance_trends: HashMap<MetricId, Vec<DataPoint>>,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub efficiency_metrics: EfficiencyMetrics,
}

/// Performance bottlenecks detected
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    pub bottleneck_type: BottleneckType,
    pub severity: AlertSeverity,
    pub affected_nodes: Vec<NodeId>,
    pub description: String,
    pub impact_on_performance: f64,
    pub suggested_resolution: String,
}

/// Types of performance bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    CPUUtilization,
    MemoryUtilization,
    NetworkBandwidth,
    DiskIO,
    TaskQueueBacklog,
    NodeFailure,
    NetworkLatency,
    LoadImbalance,
    ResourceContention,
}

/// Performance improvement recommendations
#[derive(Debug, Clone)]
pub struct PerformanceRecommendation {
    pub recommendation_id: Uuid,
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_effort: ImplementationEffort,
    pub risk_level: RiskLevel,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    ResourceOptimization,
    LoadBalancing,
    ConfigurationTuning,
    Scaling,
    FaultTolerance,
    NetworkOptimization,
    AlgorithmOptimization,
    Monitoring,
}

/// Implementation effort assessment
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,
    Low,
    Medium,
    High,
    Significant,
}

/// Risk level of implementing recommendation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// System efficiency metrics
#[derive(Debug, Clone)]
pub struct EfficiencyMetrics {
    pub cluster_utilization: f64,
    pub resource_efficiency: f64,
    pub task_throughput_efficiency: f64,
    pub network_efficiency: f64,
    pub fault_recovery_efficiency: f64,
    pub overall_efficiency: f64,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(cluster: Arc<Cluster>, config: Option<MonitoringConfig>) -> Self {
        info!("Initializing performance monitor");
        
        let config = config.unwrap_or_else(|| MonitoringConfig {
            collection_interval: Duration::from_secs(5),
            retention_period: Duration::from_secs(3600), // 1 hour
            alert_thresholds: HashMap::new(),
            enable_real_time_monitoring: true,
            enable_historical_analysis: true,
            max_data_points_per_metric: 1000,
            enable_cluster_aggregation: true,
        });
        
        Self {
            cluster,
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            node_metrics: Arc::new(RwLock::new(HashMap::new())),
            task_metrics: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            is_monitoring: Arc::new(RwLock::new(false)),
            last_collection: Arc::new(RwLock::new(SystemTime::now())),
            monitoring_statistics: Arc::new(RwLock::new(MonitoringStatistics::default())),
            metrics_event_tx: broadcast::channel(1000).0,
            alert_event_tx: broadcast::channel(1000).0,
        }
    }
    
    /// Start performance monitoring
    pub async fn start(&self) -> Result<()> {
        info!("Starting performance monitoring");
        
        // Initialize standard metrics
        self.initialize_standard_metrics().await?;
        
        // Start collection loop
        {
            let mut is_monitoring = self.is_monitoring.write().await;
            *is_monitoring = true;
        }
        
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.collection_loop().await;
        });
        
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.alert_processing_loop().await;
        });
        
        let monitor = self.clone();
        tokio::spawn(async move {
            monitor.cleanup_loop().await;
        });
        
        info!("Performance monitoring started");
        Ok(())
    }
    
    /// Stop performance monitoring
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping performance monitoring");
        
        {
            let mut is_monitoring = self.is_monitoring.write().await;
            *is_monitoring = false;
        }
        
        info!("Performance monitoring stopped");
        Ok(())
    }
    
    /// Initialize standard performance metrics
    async fn initialize_standard_metrics(&self) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        
        // Define cluster-level metrics
        metrics.insert("cluster.cpu.usage".to_string(), PerformanceMetric {
            metric_id: "cluster.cpu.usage".to_string(),
            name: "Cluster CPU Usage".to_string(),
            metric_type: MetricType::Gauge,
            unit: "percentage".to_string(),
            description: "Average CPU usage across all nodes".to_string(),
            collection_interval: self.config.collection_interval,
            aggregation: AggregationFunction::Average,
            tags: HashMap::new(),
        });
        
        metrics.insert("cluster.memory.usage".to_string(), PerformanceMetric {
            metric_id: "cluster.memory.usage".to_string(),
            name: "Cluster Memory Usage".to_string(),
            metric_type: MetricType::Gauge,
            unit: "percentage".to_string(),
            description: "Average memory usage across all nodes".to_string(),
            collection_interval: self.config.collection_interval,
            aggregation: AggregationFunction::Average,
            tags: HashMap::new(),
        });
        
        metrics.insert("cluster.task.throughput".to_string(), PerformanceMetric {
            metric_id: "cluster.task.throughput".to_string(),
            name: "Cluster Task Throughput".to_string(),
            metric_type: MetricType::Rate,
            unit: "tasks/second".to_string(),
            description: "Number of tasks completed per second".to_string(),
            collection_interval: self.config.collection_interval,
            aggregation: AggregationFunction::Sum,
            tags: HashMap::new(),
        });
        
        metrics.insert("cluster.network.throughput".to_string(), PerformanceMetric {
            metric_id: "cluster.network.throughput".to_string(),
            name: "Cluster Network Throughput".to_string(),
            metric_type: MetricType::Gauge,
            unit: "mbps".to_string(),
            description: "Total network throughput across cluster".to_string(),
            collection_interval: self.config.collection_interval,
            aggregation: AggregationFunction::Sum,
            tags: HashMap::new(),
        });
        
        // Define node-level metrics
        let node_metrics = vec![
            "node.cpu.usage",
            "node.memory.usage",
            "node.disk.usage",
            "node.task.queue.length",
            "node.task.duration",
            "node.network.bytes_sent",
            "node.network.bytes_received",
        ];
        
        for metric_name in node_metrics {
            metrics.insert(metric_name.to_string(), PerformanceMetric {
                metric_id: metric_name.to_string(),
                name: metric_name.replace('.', " ").to_string(),
                metric_type: MetricType::Gauge,
                unit: match metric_name {
                    "node.cpu.usage" | "node.memory.usage" | "node.disk.usage" => "percentage".to_string(),
                    "node.task.queue.length" => "count".to_string(),
                    "node.task.duration" => "milliseconds".to_string(),
                    "node.network.bytes_sent" | "node.network.bytes_received" => "bytes".to_string(),
                    _ => "unit".to_string(),
                },
                description: format!("Node-level metric: {}", metric_name),
                collection_interval: self.config.collection_interval,
                aggregation: AggregationFunction::Average,
                tags: HashMap::new(),
            });
        }
        
        Ok(())
    }
    
    /// Main metrics collection loop
    async fn collection_loop(&self) {
        let mut interval = interval(self.config.collection_interval);
        
        loop {
            interval.tick().await;
            
            if !*self.is_monitoring.read().await {
                break;
            }
            
            let start_time = Instant::now();
            
            // Collect cluster metrics
            if let Err(e) = self.collect_cluster_metrics().await {
                error!("Error collecting cluster metrics: {}", e);
            }
            
            // Collect node metrics
            if let Err(e) = self.collect_node_metrics().await {
                error!("Error collecting node metrics: {}", e);
            }
            
            // Collect task metrics
            if let Err(e) = self.collect_task_metrics().await {
                error!("Error collecting task metrics: {}", e);
            }
            
            // Check for alert conditions
            if let Err(e) = self.check_alert_conditions().await {
                error!("Error checking alert conditions: {}", e);
            }
            
            // Update statistics
            let collection_time = start_time.elapsed();
            {
                let mut stats = self.monitoring_statistics.write().await;
                stats.total_metrics_collected += 1;
                stats.average_collection_time = Duration::from_nanos(
                    (stats.average_collection_time.as_nanos() + collection_time.as_nanos()) / 2
                );
                stats.monitoring_uptime += self.config.collection_interval;
            }
            
            // Broadcast collection completion
            let _ = self.metrics_event_tx.send(MetricsEvent::CollectionCycleCompleted {
                cycle_duration: collection_time,
                metrics_collected: 3, // Simplified count
            });
        }
    }
    
    /// Collect cluster-level metrics
    async fn collect_cluster_metrics(&self) -> Result<()> {
        let cluster_status = self.cluster.get_status().await?;
        let timestamp = SystemTime::now();
        
        // Collect CPU usage
        let cpu_usage = cluster_status.average_load * 100.0; // Convert to percentage
        self.record_metric("cluster.cpu.usage".to_string(), cpu_usage, timestamp, None, HashMap::new()).await?;
        
        // Collect memory usage (placeholder)
        let memory_usage = 75.0; // Would be calculated from actual memory metrics
        self.record_metric("cluster.memory.usage".to_string(), memory_usage, timestamp, None, HashMap::new()).await?;
        
        // Collect task throughput
        let throughput = cluster_status.active_nodes as f64 * 0.5; // Simplified calculation
        self.record_metric("cluster.task.throughput".to_string(), throughput, timestamp, None, HashMap::new()).await?;
        
        // Collect network throughput
        let network_throughput = cluster_status.active_nodes as f64 * 10.0; // Simplified calculation
        self.record_metric("cluster.network.throughput".to_string(), network_throughput, timestamp, None, HashMap::new()).await?;
        
        Ok(())
    }
    
    /// Collect node-level metrics
    async fn collect_node_metrics(&self) -> Result<()> {
        let active_nodes = self.cluster.get_nodes_by_status(NodeStatus::Active).await?;
        let timestamp = SystemTime::now();
        
        for node in active_nodes {
            // Simulate node metrics collection
            let node_metrics = NodeMetrics {
                node_id: node.id,
                timestamp,
                cpu_usage: 50.0 + (node.id.as_u128() % 50) as f64, // Simulated usage
                memory_usage: 60.0 + (node.id.as_u128() % 30) as f64,
                disk_usage: 40.0 + (node.id.as_u128() % 40) as f64,
                network_io: NetworkIO {
                    bytes_sent: 1024 * 1024, // 1MB
                    bytes_received: 512 * 1024, // 512KB
                    packets_sent: 1000,
                    packets_received: 800,
                    errors: 0,
                    throughput_mbps: 10.0,
                },
                active_tasks: 5 + (node.id.as_u128() % 10) as usize,
                task_queue_length: (node.id.as_u128() % 5) as usize,
                average_task_duration: Duration::from_millis(500 + (node.id.as_u128() % 1000)),
                throughput_tasks_per_second: 2.0 + (node.id.as_u128() % 5) as f64,
                error_rate: 0.01,
                resource_efficiency: 0.8,
            };
            
            // Store node metrics
            {
                let mut metrics = self.node_metrics.write().await;
                metrics.insert(node.id, node_metrics.clone());
            }
            
            // Record individual metrics
            self.record_metric("node.cpu.usage".to_string(), node_metrics.cpu_usage, timestamp, Some(node.id), HashMap::new()).await?;
            self.record_metric("node.memory.usage".to_string(), node_metrics.memory_usage, timestamp, Some(node.id), HashMap::new()).await?;
            self.record_metric("node.task.queue.length".to_string(), node_metrics.task_queue_length as f64, timestamp, Some(node.id), HashMap::new()).await?;
        }
        
        Ok(())
    }
    
    /// Collect task-level metrics
    async fn collect_task_metrics(&self) -> Result<()> {
        // This would integrate with the scheduler to get task metrics
        // For now, it's a placeholder
        
        Ok(())
    }
    
    /// Record a performance metric
    async fn record_metric(
        &self,
        metric_id: MetricId,
        value: f64,
        timestamp: SystemTime,
        node_id: Option<NodeId>,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        let data_point = DataPoint { timestamp, value, tags };
        
        // Store metric data
        {
            let mut metrics = self.metrics.write().await;
            let metric_data = metrics.entry(metric_id.clone()).or_insert_with(VecDeque::new);
            
            metric_data.push_back(data_point);
            
            // Maintain retention limit
            while metric_data.len() > self.config.max_data_points_per_metric {
                metric_data.pop_front();
            }
        }
        
        // Broadcast metric collection event
        let _ = self.metrics_event_tx.send(MetricsEvent::MetricCollected {
            metric_id,
            data_point: data_point.clone(),
            node_id,
        });
        
        Ok(())
    }
    
    /// Check for alert conditions
    async fn check_alert_conditions(&self) -> Result<()> {
        // This would evaluate thresholds and generate alerts
        // For now, it's a placeholder implementation
        
        // Simulate checking CPU usage alert
        let cpu_usage = self.get_latest_metric_value("cluster.cpu.usage").await.unwrap_or(0.0);
        
        if cpu_usage > 90.0 {
            let alert = PerformanceAlert {
                alert_id: Uuid::new_v4(),
                severity: AlertSeverity::Critical,
                metric_id: "cluster.cpu.usage".to_string(),
                node_id: None,
                threshold_value: 90.0,
                current_value: cpu_usage,
                message: format!("Cluster CPU usage is critically high: {:.1}%", cpu_usage),
                timestamp: SystemTime::now(),
                acknowledged: false,
                resolved_at: None,
            };
            
            self.trigger_alert(alert).await?;
        }
        
        Ok(())
    }
    
    /// Get latest metric value
    async fn get_latest_metric_value(&self, metric_id: &str) -> Option<f64> {
        let metrics = self.metrics.read().await;
        if let Some(data_points) = metrics.get(metric_id) {
            data_points.back().map(|dp| dp.value)
        } else {
            None
        }
    }
    
    /// Trigger a performance alert
    async fn trigger_alert(&self, alert: PerformanceAlert) -> Result<()> {
        debug!("Triggering alert: {}", alert.message);
        
        // Store active alert
        {
            let mut active_alerts = self.active_alerts.write().await;
            active_alerts.insert(alert.alert_id, alert.clone());
        }
        
        // Add to history
        {
            let mut history = self.alert_history.write().await;
            history.push(alert.clone());
        }
        
        // Update statistics
        {
            let mut stats = self.monitoring_statistics.write().await;
            stats.total_alerts_generated += 1;
        }
        
        // Broadcast alert event
        let _ = self.alert_event_tx.send(AlertEvent::AlertTriggered { alert });
        
        Ok(())
    }
    
    /// Alert processing loop
    async fn alert_processing_loop(&self) {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            if !*self.is_monitoring.read().await {
                break;
            }
            
            // Process alerts - check for resolution conditions
            let mut resolved_alerts = Vec::new();
            
            {
                let active_alerts = self.active_alerts.read().await;
                for (alert_id, alert) in active_alerts.iter() {
                    // Check if alert condition has been resolved
                    if self.is_alert_resolved(alert).await {
                        resolved_alerts.push(*alert_id);
                    }
                }
            }
            
            // Resolve alerts
            for alert_id in resolved_alerts {
                self.resolve_alert(alert_id).await?;
            }
        }
    }
    
    /// Check if alert condition has been resolved
    async fn is_alert_resolved(&self, alert: &PerformanceAlert) -> bool {
        // This would check if the metric has returned to normal levels
        // For now, simplified implementation
        
        match alert.severity {
            AlertSeverity::Critical => false, // Critical alerts require manual resolution
            AlertSeverity::Error => {
                let current_value = self.get_latest_metric_value(&alert.metric_id).await.unwrap_or(0.0);
                current_value < alert.threshold_value
            }
            AlertSeverity::Warning => {
                let current_value = self.get_latest_metric_value(&alert.metric_id).await.unwrap_or(0.0);
                current_value < (alert.threshold_value * 0.8)
            }
            AlertSeverity::Info => true, // Info alerts are self-resolving
        }
    }
    
    /// Resolve an alert
    async fn resolve_alert(&self, alert_id: Uuid) -> Result<()> {
        let resolution_time = SystemTime::now();
        
        // Update alert
        {
            let mut active_alerts = self.active_alerts.write().await;
            if let Some(alert) = active_alerts.get_mut(&alert_id) {
                alert.resolved_at = Some(resolution_time);
            }
            
            // Remove from active alerts
            active_alerts.remove(&alert_id);
        }
        
        // Update statistics
        {
            let mut stats = self.monitoring_statistics.write().await;
            stats.alerts_resolved += 1;
        }
        
        // Broadcast resolution event
        let _ = self.alert_event_tx.send(AlertEvent::AlertResolved {
            alert_id,
            resolution_time,
        });
        
        debug!("Alert {} resolved", alert_id);
        Ok(())
    }
    
    /// Cleanup old data loop
    async fn cleanup_loop(&self) {
        let mut interval = interval(Duration::from_secs(300)); // Cleanup every 5 minutes
        
        loop {
            interval.tick().await;
            
            if !*self.is_monitoring.read().await {
                break;
            }
            
            // Clean up old metric data
            self.cleanup_old_metrics().await;
            
            // Clean up old alerts
            self.cleanup_old_alerts().await;
        }
    }
    
    /// Clean up old metric data
    async fn cleanup_old_metrics(&self) {
        let cutoff_time = SystemTime::now() - self.config.retention_period;
        
        let mut metrics = self.metrics.write().await;
        for (_metric_id, data_points) in metrics.iter_mut() {
            // Remove old data points
            while let Some(front) = data_points.front() {
                if front.timestamp < cutoff_time {
                    data_points.pop_front();
                } else {
                    break;
                }
            }
        }
    }
    
    /// Clean up old alerts
    async fn cleanup_old_alerts(&self) {
        let cutoff_time = SystemTime::now() - Duration::from_secs(86400); // 24 hours
        
        let mut history = self.alert_history.write().await;
        
        // Remove old resolved alerts
        history.retain(|alert| {
            if let Some(resolved_at) = alert.resolved_at {
                resolved_at >= cutoff_time
            } else {
                // Keep unresolved alerts
                true
            }
        });
    }
    
    /// Get current cluster status
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        let cluster_info = self.cluster.get_status().await?;
        let timestamp = SystemTime::now();
        
        // Calculate aggregated metrics
        let cpu_usage = self.get_latest_metric_value("cluster.cpu.usage").await.unwrap_or(0.0);
        let memory_usage = self.get_latest_metric_value("cluster.memory.usage").await.unwrap_or(0.0);
        let network_throughput = self.get_latest_metric_value("cluster.network.throughput").await.unwrap_or(0.0);
        let task_throughput = self.get_latest_metric_value("cluster.task.throughput").await.unwrap_or(0.0);
        
        // Calculate overall health score
        let health_score = if cpu_usage < 80.0 && memory_usage < 80.0 && network_throughput > 0.0 {
            1.0 - ((cpu_usage + memory_usage) / 200.0)
        } else {
            0.5
        };
        
        // Calculate cluster efficiency
        let efficiency = task_throughput / (cluster_info.active_nodes as f64 + 1.0);
        
        Ok(ClusterStatus {
            cluster_id: cluster_info.cluster_id,
            timestamp,
            overall_health: health_score,
            total_nodes: cluster_info.total_nodes,
            active_nodes: cluster_info.active_nodes,
            total_cpu_cores: cluster_info.total_cpu_cores,
            total_memory_gb: cluster_info.total_memory_gb,
            average_cpu_usage: cpu_usage,
            average_memory_usage: memory_usage,
            network_throughput_mbps: network_throughput,
            active_tasks: cluster_info.active_nodes * 5, // Simplified
            completed_tasks: cluster_info.active_nodes as u64 * 100,
            failed_tasks: cluster_info.failed_nodes as u64 * 2,
            throughput_tasks_per_second: task_throughput,
            latency_p95_ms: 100.0, // Simplified
            error_rate: cluster_info.failed_nodes as f64 / (cluster_info.total_nodes as f64 + 1.0),
            cluster_efficiency: efficiency,
        })
    }
    
    /// Get node metrics for specific node
    pub async fn get_node_metrics(&self, node_id: NodeId) -> Option<NodeMetrics> {
        let node_metrics = self.node_metrics.read().await;
        node_metrics.get(&node_id).cloned()
    }
    
    /// Get all node metrics
    pub async fn get_all_node_metrics(&self) -> HashMap<NodeId, NodeMetrics> {
        self.node_metrics.read().await.clone()
    }
    
    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
        let active_alerts = self.active_alerts.read().await;
        active_alerts.values().cloned().collect()
    }
    
    /// Get alert history
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<PerformanceAlert> {
        let history = self.alert_history.read().await;
        let mut alerts = history.clone();
        
        if let Some(limit) = limit {
            alerts.truncate(limit);
        }
        
        alerts
    }
    
    /// Get performance metrics for analysis
    pub async fn get_metrics_for_analysis(&self, metric_id: &str, duration: Duration) -> Vec<DataPoint> {
        let cutoff_time = SystemTime::now() - duration;
        let metrics = self.metrics.read().await;
        
        if let Some(data_points) = metrics.get(metric_id) {
            data_points
                .iter()
                .filter(|dp| dp.timestamp >= cutoff_time)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Generate cluster analysis
    pub async fn generate_cluster_analysis(&self) -> Result<ClusterAnalysis> {
        let cluster_status = self.get_cluster_status().await?;
        
        // Collect performance trends for key metrics
        let key_metrics = vec![
            "cluster.cpu.usage",
            "cluster.memory.usage",
            "cluster.task.throughput",
            "cluster.network.throughput",
        ];
        
        let mut performance_trends = HashMap::new();
        for metric_id in key_metrics {
            let trends = self.get_metrics_for_analysis(metric_id, Duration::from_secs(3600)).await;
            performance_trends.insert(metric_id.to_string(), trends);
        }
        
        // Analyze bottlenecks
        let bottlenecks = self.analyze_bottlenecks(&cluster_status).await?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&bottlenecks, &cluster_status).await?;
        
        // Calculate efficiency metrics
        let efficiency_metrics = self.calculate_efficiency_metrics(&cluster_status).await?;
        
        Ok(ClusterAnalysis {
            analysis_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            cluster_status,
            performance_trends,
            bottlenecks,
            recommendations,
            efficiency_metrics,
        })
    }
    
    /// Analyze performance bottlenecks
    async fn analyze_bottlenecks(&self, cluster_status: &ClusterStatus) -> Result<Vec<PerformanceBottleneck>> {
        let mut bottlenecks = Vec::new();
        
        // Check CPU utilization
        if cluster_status.average_cpu_usage > 80.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::CPUUtilization,
                severity: if cluster_status.average_cpu_usage > 95.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                affected_nodes: vec![], // Would be determined from node metrics
                description: format!("High CPU utilization: {:.1}%", cluster_status.average_cpu_usage),
                impact_on_performance: (cluster_status.average_cpu_usage - 80.0) / 20.0,
                suggested_resolution: "Consider scaling up CPU resources or optimizing CPU-intensive tasks".to_string(),
            });
        }
        
        // Check memory utilization
        if cluster_status.average_memory_usage > 80.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::MemoryUtilization,
                severity: if cluster_status.average_memory_usage > 95.0 { AlertSeverity::Critical } else { AlertSeverity::Warning },
                affected_nodes: vec![],
                description: format!("High memory utilization: {:.1}%", cluster_status.average_memory_usage),
                impact_on_performance: (cluster_status.average_memory_usage - 80.0) / 20.0,
                suggested_resolution: "Consider adding more memory or optimizing memory usage patterns".to_string(),
            });
        }
        
        // Check network throughput
        if cluster_status.network_throughput_mbps < 10.0 {
            bottlenecks.push(PerformanceBottleneck {
                bottleneck_type: BottleneckType::NetworkBandwidth,
                severity: AlertSeverity::Warning,
                affected_nodes: vec![],
                description: format!("Low network throughput: {:.1} Mbps", cluster_status.network_throughput_mbps),
                impact_on_performance: 0.3,
                suggested_resolution: "Check network configuration and consider upgrading network infrastructure".to_string(),
            });
        }
        
        Ok(bottlenecks)
    }
    
    /// Generate performance recommendations
    async fn generate_recommendations(&self, bottlenecks: &[PerformanceBottleneck], cluster_status: &ClusterStatus) -> Result<Vec<PerformanceRecommendation>> {
        let mut recommendations = Vec::new();
        
        // Generate recommendations based on bottlenecks
        for bottleneck in bottlenecks {
            match bottleneck.bottleneck_type {
                BottleneckType::CPUUtilization => {
                    recommendations.push(PerformanceRecommendation {
                        recommendation_id: Uuid::new_v4(),
                        priority: if bottleneck.severity == AlertSeverity::Critical { RecommendationPriority::Critical } else { RecommendationPriority::High },
                        category: RecommendationCategory::ResourceOptimization,
                        title: "Scale CPU Resources".to_string(),
                        description: "Consider adding more CPU cores or optimizing CPU-intensive tasks".to_string(),
                        expected_improvement: 25.0,
                        implementation_effort: ImplementationEffort::Medium,
                        risk_level: RiskLevel::Low,
                    });
                }
                BottleneckType::MemoryUtilization => {
                    recommendations.push(PerformanceRecommendation {
                        recommendation_id: Uuid::new_v4(),
                        priority: if bottleneck.severity == AlertSeverity::Critical { RecommendationPriority::Critical } else { RecommendationPriority::High },
                        category: RecommendationCategory::ResourceOptimization,
                        title: "Scale Memory Resources".to_string(),
                        description: "Consider adding more memory or optimizing memory usage patterns".to_string(),
                        expected_improvement: 20.0,
                        implementation_effort: ImplementationEffort::Low,
                        risk_level: RiskLevel::VeryLow,
                    });
                }
                _ => {}
            }
        }
        
        // Add general recommendations
        if cluster_status.error_rate > 0.01 {
            recommendations.push(PerformanceRecommendation {
                recommendation_id: Uuid::new_v4(),
                priority: RecommendationPriority::High,
                category: RecommendationCategory::FaultTolerance,
                title: "Improve Fault Tolerance".to_string(),
                description: "Implement better error handling and recovery mechanisms".to_string(),
                expected_improvement: 15.0,
                implementation_effort: ImplementationEffort::Medium,
                risk_level: RiskLevel::Low,
            });
        }
        
        Ok(recommendations)
    }
    
    /// Calculate efficiency metrics
    async fn calculate_efficiency_metrics(&self, cluster_status: &ClusterStatus) -> Result<EfficiencyMetrics> {
        Ok(EfficiencyMetrics {
            cluster_utilization: (cluster_status.average_cpu_usage + cluster_status.average_memory_usage) / 200.0,
            resource_efficiency: cluster_status.cluster_efficiency,
            task_throughput_efficiency: cluster_status.throughput_tasks_per_second / (cluster_status.active_nodes as f64 + 1.0),
            network_efficiency: cluster_status.network_throughput_mbps / 100.0,
            fault_recovery_efficiency: 1.0 - cluster_status.error_rate,
            overall_efficiency: (cluster_status.cluster_efficiency + (1.0 - cluster_status.error_rate)) / 2.0,
        })
    }
    
    /// Subscribe to metrics events
    pub fn subscribe_metrics_events(&self) -> broadcast::Receiver<MetricsEvent> {
        self.metrics_event_tx.subscribe()
    }
    
    /// Subscribe to alert events
    pub fn subscribe_alert_events(&self) -> broadcast::Receiver<AlertEvent> {
        self.alert_event_tx.subscribe()
    }
    
    /// Get monitoring statistics
    pub async fn get_statistics(&self) -> MonitoringStatistics {
        self.monitoring_statistics.read().await.clone()
    }
    
    /// Create performance dashboard
    pub fn create_dashboard(&self, update_interval: Duration, max_historical_points: usize) -> PerformanceDashboard {
        PerformanceDashboard {
            monitor: Arc::new(self.clone()),
            update_interval,
            max_historical_points,
        }
    }
}

impl Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "Info"),
            AlertSeverity::Warning => write!(f, "Warning"),
            AlertSeverity::Error => write!(f, "Error"),
            AlertSeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl Display for MetricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricType::Counter => write!(f, "Counter"),
            MetricType::Gauge => write!(f, "Gauge"),
            MetricType::Histogram => write!(f, "Histogram"),
            MetricType::Timer => write!(f, "Timer"),
            MetricType::Rate => write!(f, "Rate"),
        }
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            cluster: self.cluster.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            node_metrics: self.node_metrics.clone(),
            task_metrics: self.task_metrics.clone(),
            performance_metrics: self.performance_metrics.clone(),
            active_alerts: self.active_alerts.clone(),
            alert_history: self.alert_history.clone(),
            is_monitoring: self.is_monitoring.clone(),
            last_collection: self.last_collection.clone(),
            monitoring_statistics: self.monitoring_statistics.clone(),
            metrics_event_tx: self.metrics_event_tx.clone(),
            alert_event_tx: self.alert_event_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let monitor = PerformanceMonitor::new(cluster, None);
        
        assert!(!monitor.config.collection_interval.is_zero());
    }
    
    #[tokio::test]
    async fn test_metric_recording() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let monitor = PerformanceMonitor::new(cluster, None);
        
        let metric_id = "test.metric".to_string();
        let value = 42.0;
        let timestamp = SystemTime::now();
        let tags = HashMap::new();
        
        let result = monitor.record_metric(metric_id, value, timestamp, None, tags).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_cluster_status_generation() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let monitor = PerformanceMonitor::new(cluster, None);
        
        let status = monitor.get_cluster_status().await;
        assert!(status.is_ok());
        
        let status = status.unwrap();
        assert!(status.overall_health >= 0.0 && status.overall_health <= 1.0);
        assert!(status.active_nodes >= 0);
    }
    
    #[tokio::test]
    async fn test_alert_creation() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let monitor = PerformanceMonitor::new(cluster, None);
        
        let alert = PerformanceAlert {
            alert_id: Uuid::new_v4(),
            severity: AlertSeverity::Warning,
            metric_id: "test.metric".to_string(),
            node_id: None,
            threshold_value: 80.0,
            current_value: 90.0,
            message: "Test alert message".to_string(),
            timestamp: SystemTime::now(),
            acknowledged: false,
            resolved_at: None,
        };
        
        let result = monitor.trigger_alert(alert).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_performance_analysis() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let monitor = PerformanceMonitor::new(cluster, None);
        
        // Record some test data
        for i in 0..10 {
            monitor.record_metric(
                "cluster.cpu.usage".to_string(),
                50.0 + i as f64 * 5.0,
                SystemTime::now(),
                None,
                HashMap::new(),
            ).await.unwrap();
        }
        
        let analysis = monitor.generate_cluster_analysis().await;
        assert!(analysis.is_ok());
        
        let analysis = analysis.unwrap();
        assert!(!analysis.analysis_id.is_nil());
        assert!(!analysis.performance_trends.is_empty());
    }
    
    #[test]
    fn test_severity_ordering() {
        assert!(AlertSeverity::Critical > AlertSeverity::Error);
        assert!(AlertSeverity::Error > AlertSeverity::Warning);
        assert!(AlertSeverity::Warning > AlertSeverity::Info);
    }
    
    #[test]
    fn test_metric_type_formatting() {
        assert_eq!(format!("{}", MetricType::Gauge), "Gauge");
        assert_eq!(format!("{}", MetricType::Counter), "Counter");
        assert_eq!(format!("{}", MetricType::Timer), "Timer");
    }
    
    #[test]
    fn test_data_point_structure() {
        let data_point = DataPoint {
            timestamp: SystemTime::now(),
            value: 100.5,
            tags: HashMap::new(),
        };
        
        assert!(data_point.value >= 0.0);
    }
    
    #[tokio::test]
    async fn test_monitoring_statistics() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let monitor = PerformanceMonitor::new(cluster, None);
        
        let stats = monitor.get_statistics().await;
        assert_eq!(stats.total_metrics_collected, 0);
        assert_eq!(stats.total_alerts_generated, 0);
    }
}