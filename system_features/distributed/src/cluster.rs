//! Cluster management and node coordination
//!
//! This module provides cluster formation, node discovery, and coordination
//! services for the distributed computing framework.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::common::{NodeId, ResourceInfo, NodeMetrics};

/// Node identification and status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub address: SocketAddr,
    pub capabilities: NodeCapabilities,
    pub status: NodeStatus,
    pub last_seen: SystemTime,
    pub metrics: Option<NodeMetrics>,
}

/// Node capabilities and resource specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub cpu_cores: usize,
    pub memory_gb: u64,
    pub network_bandwidth_mbps: u64,
    pub storage_gb: u64,
    pub supported_protocols: Vec<String>,
    pub max_concurrent_tasks: usize,
}

/// Current status of a cluster node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is healthy and accepting tasks
    Active,
    /// Node is starting up
    Starting,
    /// Node is processing tasks
    Busy,
    /// Node is overloaded
    Overloaded,
    /// Node is shutting down
    ShuttingDown,
    /// Node has failed
    Failed,
    /// Node is temporarily unavailable
    Unavailable,
}

/// Cluster configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub cluster_id: NodeId,
    pub bind_address: SocketAddr,
    pub discovery_port: u16,
    pub coordinator_port: u16,
    pub heartbeat_interval: Duration,
    pub node_timeout: Duration,
    pub max_nodes: usize,
    pub min_nodes: usize,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        let local_id = Uuid::new_v4();
        let bind_address = "127.0.0.1:0".parse().unwrap();
        
        Self {
            cluster_id: local_id,
            bind_address,
            discovery_port: 8080,
            coordinator_port: 8081,
            heartbeat_interval: Duration::from_secs(5),
            node_timeout: Duration::from_secs(30),
            max_nodes: 100,
            min_nodes: 1,
        }
    }
}

/// Main cluster coordination service
pub struct Cluster {
    config: ClusterConfig,
    nodes: Arc<RwLock<HashMap<NodeId, Node>>>,
    node_sender: broadcast::Sender<NodeUpdate>,
    coordinator_sender: mpsc::Sender<CoordinatorCommand>,
    coordinator_receiver: mpsc::Receiver<CoordinatorCommand>,
    discovery_service: Arc<DiscoveryService>,
    health_monitor: Arc<HealthMonitor>,
}

/// Node update events for cluster members
#[derive(Debug, Clone)]
pub enum NodeUpdate {
    NodeJoined(Node),
    NodeLeft(NodeId),
    NodeStatusChanged(NodeId, NodeStatus),
    NodeMetricsUpdated(NodeId, NodeMetrics),
}

/// Coordinator commands for cluster management
#[derive(Debug)]
pub enum CoordinatorCommand {
    /// Request cluster status
    GetStatus(oneshot::Sender<ClusterStatus>),
    /// Add new node to cluster
    AddNode(Node),
    /// Remove node from cluster
    RemoveNode(NodeId),
    /// Update node capabilities
    UpdateCapabilities(NodeId, NodeCapabilities),
    /// Get optimal nodes for task assignment
    GetOptimalNodes(Vec<NodeId>),
    /// Get cluster health report
    GetHealthReport(oneshot::Sender<HealthReport>),
}

/// Cluster status information
#[derive(Debug, Clone)]
pub struct ClusterStatus {
    pub cluster_id: NodeId,
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub failed_nodes: usize,
    pub total_cpu_cores: usize,
    pub total_memory_gb: u64,
    pub average_load: f64,
    pub network_health: f64,
}

/// Health monitoring and reporting
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub cluster_id: NodeId,
    pub overall_health: f64,
    pub node_health: HashMap<NodeId, f64>,
    pub issues: Vec<HealthIssue>,
}

/// Health issue detected in cluster
#[derive(Debug, Clone)]
pub struct HealthIssue {
    pub severity: HealthSeverity,
    pub node_id: Option<NodeId>,
    pub description: String,
    pub timestamp: SystemTime,
}

/// Severity levels for health issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthSeverity {
    Info,
    Warning,
    Critical,
    Fatal,
}

/// Node discovery service for cluster formation
pub struct DiscoveryService {
    config: ClusterConfig,
    known_nodes: Arc<RwLock<HashMap<NodeId, Node>>>,
    discovery_sender: broadcast::Sender<NodeUpdate>,
    join_requests: Arc<RwLock<Vec<Node>>>,
}

/// Health monitoring service
pub struct HealthMonitor {
    cluster: Arc<RwLock<HashMap<NodeId, Node>>>,
    health_check_interval: Duration,
    node_timeout: Duration,
    health_scores: Arc<RwLock<HashMap<NodeId, f64>>>,
    issues: Arc<RwLock<Vec<HealthIssue>>>,
}

impl Cluster {
    /// Create a new cluster instance
    pub async fn new(config: &ClusterConfig) -> Result<Self> {
        info!("Creating new cluster with ID: {}", config.cluster_id);
        
        let nodes = Arc::new(RwLock::new(HashMap::new()));
        let (node_sender, _) = broadcast::channel(1000);
        let (coordinator_sender, coordinator_receiver) = mpsc::channel(100);
        
        let discovery_service = Arc::new(DiscoveryService::new(config.clone()).await?);
        let health_monitor = Arc::new(HealthMonitor::new(
            nodes.clone(),
            config.heartbeat_interval,
            config.node_timeout,
        )?);
        
        Ok(Self {
            config: config.clone(),
            nodes,
            node_sender,
            coordinator_sender,
            coordinator_receiver,
            discovery_service,
            health_monitor,
        })
    }
    
    /// Start cluster operations
    pub async fn start(&self) -> Result<()> {
        info!("Starting cluster services");
        
        // Start node discovery
        self.discovery_service.start().await?;
        
        // Start health monitoring
        self.health_monitor.start().await?;
        
        // Start coordinator service
        self.start_coordinator().await?;
        
        info!("Cluster services started successfully");
        Ok(())
    }
    
    /// Stop cluster operations
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping cluster services");
        
        // Stop health monitoring
        self.health_monitor.stop().await?;
        
        // Stop node discovery
        self.discovery_service.stop().await?;
        
        info!("Cluster services stopped");
        Ok(())
    }
    
    /// Add a node to the cluster
    pub async fn add_node(&self, node: Node) -> Result<()> {
        debug!("Adding node {} to cluster", node.id);
        
        {
            let mut nodes = self.nodes.write().await;
            nodes.insert(node.id, node.clone());
        }
        
        // Broadcast node update
        let _ = self.node_sender.send(NodeUpdate::NodeJoined(node));
        
        info!("Node {} added to cluster", node.id);
        Ok(())
    }
    
    /// Remove a node from the cluster
    pub async fn remove_node(&self, node_id: NodeId) -> Result<()> {
        debug!("Removing node {} from cluster", node_id);
        
        {
            let mut nodes = self.nodes.write().await;
            nodes.remove(&node_id);
        }
        
        // Broadcast node update
        let _ = self.node_sender.send(NodeUpdate::NodeLeft(node_id));
        
        info!("Node {} removed from cluster", node_id);
        Ok(())
    }
    
    /// Update node status
    pub async fn update_node_status(&self, node_id: NodeId, status: NodeStatus) -> Result<()> {
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(&node_id) {
                node.status = status.clone();
                node.last_seen = SystemTime::now();
            }
        }
        
        // Broadcast status change
        let _ = self.node_sender.send(NodeUpdate::NodeStatusChanged(node_id, status));
        
        debug!("Node {} status updated", node_id);
        Ok(())
    }
    
    /// Update node metrics
    pub async fn update_node_metrics(&self, node_id: NodeId, metrics: NodeMetrics) -> Result<()> {
        {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(&node_id) {
                node.metrics = Some(metrics.clone());
            }
        }
        
        // Broadcast metrics update
        let _ = self.node_sender.send(NodeUpdate::NodeMetricsUpdated(node_id, metrics));
        
        Ok(())
    }
    
    /// Get current cluster status
    pub async fn get_status(&self) -> Result<ClusterStatus> {
        let nodes = self.nodes.read().await;
        
        let total_nodes = nodes.len();
        let active_nodes = nodes.values().filter(|n| n.status == NodeStatus::Active).count();
        let failed_nodes = nodes.values().filter(|n| n.status == NodeStatus::Failed).count();
        
        let total_cpu_cores: usize = nodes.values().map(|n| n.capabilities.cpu_cores).sum();
        let total_memory_gb: u64 = nodes.values().map(|n| n.capabilities.memory_gb).sum();
        
        let average_load = if total_nodes > 0 {
            nodes.values()
                .filter_map(|n| n.metrics.as_ref().map(|m| m.resource_info.cpu_usage))
                .sum::<f64>() / total_nodes as f64
        } else {
            0.0
        };
        
        Ok(ClusterStatus {
            cluster_id: self.config.cluster_id,
            total_nodes,
            active_nodes,
            failed_nodes,
            total_cpu_cores,
            total_memory_gb,
            average_load,
            network_health: 1.0 - (failed_nodes as f64 / total_nodes as f64),
        })
    }
    
    /// Get nodes by status filter
    pub async fn get_nodes_by_status(&self, status: NodeStatus) -> Result<Vec<Node>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.values()
            .filter(|n| n.status == status)
            .cloned()
            .collect())
    }
    
    /// Find optimal nodes for task assignment
    pub async fn find_optimal_nodes(&self, count: usize, preferences: Vec<NodeId>) -> Result<Vec<NodeId>> {
        let nodes = self.nodes.read().await;
        
        // Create list of candidate nodes based on preferences or all active nodes
        let candidates = if !preferences.is_empty() {
            preferences.into_iter().filter(|id| {
                nodes.get(id)
                    .map(|n| n.status == NodeStatus::Active)
                    .unwrap_or(false)
            }).collect()
        } else {
            nodes.values()
                .filter(|n| n.status == NodeStatus::Active)
                .map(|n| n.id)
                .collect()
        };
        
        if candidates.is_empty() {
            return Ok(vec![]);
        }
        
        // Sort by load (lower is better) and capacity
        let mut scored_nodes: Vec<_> = candidates.into_iter()
            .filter_map(|node_id| {
                nodes.get(&node_id).and_then(|node| {
                    node.metrics.as_ref().map(|metrics| {
                        let load_score = metrics.resource_info.cpu_usage;
                        let capacity_score = node.capabilities.max_concurrent_tasks as f64;
                        let efficiency_score = capacity_score / (load_score + 1.0);
                        (node_id, efficiency_score)
                    })
                })
            })
            .collect();
        
        scored_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(scored_nodes.into_iter().take(count).map(|(id, _)| id).collect())
    }
    
    /// Start coordinator service for cluster management
    async fn start_coordinator(&self) -> Result<()> {
        let coordinator_sender = self.coordinator_sender.clone();
        let cluster = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(tokio::time::Duration::from_secs(1));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Periodic coordinator tasks
                        if let Err(e) = cluster.process_coordinator_commands().await {
                            error!("Error processing coordinator commands: {}", e);
                        }
                    }
                    command = cluster.coordinator_receiver.recv() => {
                        if let Some(cmd) = command {
                            if let Err(e) = cluster.handle_coordinator_command(cmd).await {
                                error!("Error handling coordinator command: {}", e);
                            }
                        } else {
                            // Channel closed, exit
                            break;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Handle individual coordinator commands
    async fn handle_coordinator_command(&self, command: CoordinatorCommand) -> Result<()> {
        match command {
            CoordinatorCommand::GetStatus(sender) => {
                let status = self.get_status().await?;
                let _ = sender.send(status);
            }
            CoordinatorCommand::AddNode(node) => {
                self.add_node(node).await?;
            }
            CoordinatorCommand::RemoveNode(node_id) => {
                self.remove_node(node_id).await?;
            }
            CoordinatorCommand::UpdateCapabilities(node_id, capabilities) => {
                let mut nodes = self.nodes.write().await;
                if let Some(node) = nodes.get_mut(&node_id) {
                    node.capabilities = capabilities;
                }
            }
            CoordinatorCommand::GetOptimalNodes(node_ids) => {
                let optimal_nodes = self.find_optimal_nodes(3, node_ids).await?;
                // Would send back to caller if needed
                debug!("Optimal nodes selected: {:?}", optimal_nodes);
            }
            CoordinatorCommand::GetHealthReport(sender) => {
                let report = self.health_monitor.generate_report().await?;
                let _ = sender.send(report);
            }
        }
        Ok(())
    }
    
    /// Process coordinator commands from the queue
    async fn process_coordinator_commands(&self) -> Result<()> {
        // This would process pending coordinator commands
        // For now, just a placeholder for the periodic tasks
        let _ = self.get_status().await?;
        Ok(())
    }
    
    /// Subscribe to node update events
    pub fn subscribe_updates(&self) -> broadcast::Receiver<NodeUpdate> {
        self.node_sender.subscribe()
    }
}

impl Clone for Cluster {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            nodes: self.nodes.clone(),
            node_sender: self.node_sender.clone(),
            coordinator_sender: self.coordinator_sender.clone(),
            coordinator_receiver: self.coordinator_receiver.clone(),
            discovery_service: self.discovery_service.clone(),
            health_monitor: self.health_monitor.clone(),
        }
    }
}

impl DiscoveryService {
    /// Create a new discovery service
    pub async fn new(config: ClusterConfig) -> Result<Self> {
        let (discovery_sender, _) = broadcast::channel(1000);
        let known_nodes = Arc::new(RwLock::new(HashMap::new()));
        let join_requests = Arc::new(RwLock::new(Vec::new()));
        
        Ok(Self {
            config,
            known_nodes,
            discovery_sender,
            join_requests,
        })
    }
    
    /// Start the discovery service
    pub async fn start(&self) -> Result<()> {
        info!("Starting node discovery service");
        
        // Start UDP multicast for node discovery
        let discovery_sender = self.discovery_sender.clone();
        let known_nodes = self.known_nodes.clone();
        
        tokio::spawn(async move {
            // Discovery implementation would go here
            // This is a simplified placeholder
            let mut interval = interval(tokio::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                // Periodic discovery tasks
            }
        });
        
        Ok(())
    }
    
    /// Stop the discovery service
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping node discovery service");
        Ok(())
    }
}

impl HealthMonitor {
    /// Create a new health monitoring service
    pub fn new(
        cluster: Arc<RwLock<HashMap<NodeId, Node>>>,
        check_interval: Duration,
        node_timeout: Duration,
    ) -> Result<Self> {
        Ok(Self {
            cluster,
            health_check_interval: check_interval,
            node_timeout,
            health_scores: Arc::new(RwLock::new(HashMap::new())),
            issues: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Start health monitoring
    pub async fn start(&self) -> Result<()> {
        info!("Starting health monitoring service");
        
        let health_check_interval = self.health_check_interval;
        let cluster = self.cluster.clone();
        let health_scores = self.health_scores.clone();
        let issues = self.issues.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(health_check_interval);
            
            loop {
                interval.tick().await;
                
                // Check all nodes
                let mut scores = health_scores.write().await;
                let mut issues_list = issues.write().await;
                
                let nodes = cluster.read().await;
                for (node_id, node) in nodes.iter() {
                    let health_score = calculate_node_health(node);
                    scores.insert(*node_id, health_score);
                    
                    if health_score < 0.5 {
                        issues_list.push(HealthIssue {
                            severity: HealthSeverity::Warning,
                            node_id: Some(*node_id),
                            description: format!("Node {} health score low: {:.2}", node_id, health_score),
                            timestamp: SystemTime::now(),
                        });
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop health monitoring
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping health monitoring service");
        Ok(())
    }
    
    /// Generate health report
    pub async fn generate_report(&self) -> Result<HealthReport> {
        let scores = self.health_scores.read().await;
        let issues = self.issues.read().await;
        
        let overall_health = if !scores.is_empty() {
            scores.values().sum::<f64>() / scores.len() as f64
        } else {
            1.0
        };
        
        Ok(HealthReport {
            cluster_id: Uuid::new_v4(), // Would get from cluster config
            overall_health,
            node_health: scores.clone(),
            issues: issues.clone(),
        })
    }
}

/// Calculate health score for a node based on its metrics
fn calculate_node_health(node: &Node) -> f64 {
    if let Some(metrics) = &node.metrics {
        let cpu_health = 1.0 - (metrics.resource_info.cpu_usage / 100.0);
        let memory_health = 1.0 - (metrics.resource_info.memory_usage / 100.0);
        let network_health = 1.0 - (metrics.resource_info.network_bandwidth / 1000.0);
        
        // Weighted average of health metrics
        (cpu_health * 0.4 + memory_health * 0.3 + network_health * 0.3)
            .clamp(0.0, 1.0)
    } else {
        0.5 // Default health when no metrics available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cluster_creation() {
        let config = ClusterConfig::default();
        let cluster = Cluster::new(&config).await;
        assert!(cluster.is_ok());
    }
    
    #[tokio::test]
    async fn test_node_addition() {
        let config = ClusterConfig::default();
        let cluster = Cluster::new(&config).await.unwrap();
        
        let node = Node {
            id: Uuid::new_v4(),
            address: "127.0.0.1:8080".parse().unwrap(),
            capabilities: NodeCapabilities {
                cpu_cores: 4,
                memory_gb: 8,
                network_bandwidth_mbps: 100,
                storage_gb: 500,
                supported_protocols: vec!["tcp".to_string()],
                max_concurrent_tasks: 8,
            },
            status: NodeStatus::Active,
            last_seen: SystemTime::now(),
            metrics: None,
        };
        
        let result = cluster.add_node(node).await;
        assert!(result.is_ok());
        
        let status = cluster.get_status().await.unwrap();
        assert_eq!(status.total_nodes, 1);
        assert_eq!(status.active_nodes, 1);
    }
}