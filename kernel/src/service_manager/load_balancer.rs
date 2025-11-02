//! Service Load Balancer
//! 
//! This module provides load balancing functionality for distributing
//! requests across multiple service instances.

use spin::{Mutex, RwLock};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::{ServiceId, ServiceResult, ServiceError, service::ServiceInstance};
use super::discovery::{ServiceEndpoint};

/// Get current system time
fn get_current_time() -> u64 {
    super::get_current_time()
}

/// Load Balancer - Main load balancing component
pub struct LoadBalancer {
    strategies: RwLock<BTreeMap<String, BalancingStrategy>>,
    routing_table: RwLock<BTreeMap<String, Vec<ServiceInstance>>>,
    health_checker: Box<dyn ServiceHealthChecker>,
    load_stats: RwLock<BTreeMap<ServiceId, InstanceLoadStats>>,
    balancing_history: RwLock<VecDeque<BalancingDecision>>,
    current_strategy: AtomicUsize,
}

/// Balancing Strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BalancingStrategy {
    RoundRobin = 0,
    LeastConnections = 1,
    WeightedRoundRobin = 2,
    WeightedLeastConnections = 3,
    Random = 4,
    IpHash = 5,
    ConsistentHash = 6,
    FastestResponse = 7,
    HealthBased = 8,
}

/// Service Health Checker Interface
pub trait ServiceHealthChecker: Send + Sync {
    fn is_healthy(&self, service_id: ServiceId) -> bool;
    fn get_response_time(&self, service_id: ServiceId) -> Option<u64>;
}

/// Default Service Health Checker
#[derive(Debug, Clone)]
pub struct DefaultHealthChecker;

impl ServiceHealthChecker for DefaultHealthChecker {
    fn is_healthy(&self, service_id: ServiceId) -> bool {
        // Simplified implementation - would integrate with actual health monitoring
        true
    }

    fn get_response_time(&self, service_id: ServiceId) -> Option<u64> {
        // Simplified implementation - would get actual response times
        Some(50) // Mock response time
    }
}

/// Instance Load Statistics
#[derive(Debug, Clone)]
struct InstanceLoadStats {
    pub service_id: ServiceId,
    pub current_connections: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub last_request_time: Option<u64>,
    pub last_health_check: Option<u64>,
}

/// Balancing Decision
#[derive(Debug, Clone)]
struct BalancingDecision {
    pub service_name: String,
    pub selected_instance: ServiceId,
    pub strategy_used: BalancingStrategy,
    pub decision_time: u64,
    pub load_metrics: Option<InstanceLoadStats>,
}

/// Request Routing Information
#[derive(Debug, Clone)]
pub struct RoutingRequest {
    pub service_name: String,
    pub client_ip: Option<String>,
    pub request_hash: Option<u64>,
    pub priority: RequestPriority,
}

/// Request Priority Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RequestPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Routing Response
#[derive(Debug, Clone)]
pub struct RoutingResponse {
    pub selected_instance: ServiceId,
    pub endpoint: ServiceEndpoint,
    pub strategy_used: BalancingStrategy,
    pub load_score: f32,
    pub estimated_wait_time: Option<u64>,
}

/// Load Balancer Configuration
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    pub default_strategy: BalancingStrategy,
    pub health_check_interval: u32,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: u32,
    pub max_connections_per_instance: u32,
    pub request_timeout: u32,
    pub enable_failover: bool,
    pub enable_circuit_breaker: bool,
}

/// Circuit Breaker State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CircuitBreakerState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

/// Circuit Breaker
#[derive(Debug, Clone)]
struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<u64>,
    last_success_time: Option<u64>,
    threshold: u32,
    timeout: u32,
}

impl LoadBalancer {
    /// Create a new load balancer with default strategy
    pub fn with_strategy(strategy: BalancingStrategy) -> Self {
        let mut strategies = BTreeMap::new();
        strategies.insert("default".to_string(), strategy);

        LoadBalancer {
            strategies: RwLock::new(strategies),
            routing_table: RwLock::new(BTreeMap::new()),
            health_checker: Box::new(DefaultHealthChecker),
            load_stats: RwLock::new(BTreeMap::new()),
            balancing_history: RwLock::new(VecDeque::new()),
            current_strategy: AtomicUsize::new(strategy as usize),
        }
    }

    /// Initialize the load balancer
    pub fn init(&self) -> ServiceResult<()> {
        // Initialize health checker
        // Clear routing table
        // Set up monitoring
        
        info!("Load balancer initialized");
        Ok(())
    }

    /// Add service instance to routing table
    pub fn add_instance(&self, service_name: String, instance: ServiceInstance) -> ServiceResult<()> {
        let mut routing_table = self.routing_table.write();
        
        let instances = routing_table.entry(service_name).or_insert_with(Vec::new);
        instances.push(instance.clone());

        // Initialize load stats for the instance
        let mut load_stats = self.load_stats.write();
        load_stats.insert(instance.service_id, InstanceLoadStats {
            service_id: instance.service_id,
            current_connections: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0.0,
            last_request_time: None,
            last_health_check: None,
        });

        info!("Service instance added to load balancer: {} ({})", service_name, instance.service_id.0);
        Ok(())
    }

    /// Remove service instance from routing table
    pub fn remove_instance(&self, service_name: String, service_id: ServiceId) -> ServiceResult<()> {
        let mut routing_table = self.routing_table.write();
        
        if let Some(instances) = routing_table.get_mut(&service_name) {
            instances.retain(|instance| instance.service_id != service_id);
            
            if instances.is_empty() {
                routing_table.remove(&service_name);
            }
        }

        // Remove load stats
        let mut load_stats = self.load_stats.write();
        load_stats.remove(&service_id);

        info!("Service instance removed from load balancer: {} ({})", service_name, service_id.0);
        Ok(())
    }

    /// Select service instance for request routing
    pub fn select_instance(&self, service_name: &str, available_instances: &[ServiceId]) -> ServiceResult<ServiceId> {
        let routing_table = self.routing_table.read();
        let instances = routing_table.get(service_name)
            .ok_or(ServiceError::ServiceNotFound)?;

        // Filter instances based on availability and health
        let healthy_instances: Vec<&ServiceInstance> = instances.iter()
            .filter(|instance| {
                available_instances.contains(&instance.service_id) &&
                instance.status == super::service::ServiceState::Running &&
                self.health_checker.is_healthy(instance.service_id)
            })
            .collect();

        if healthy_instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }

        // Get strategy for this service
        let strategies = self.strategies.read();
        let strategy = strategies.get(service_name)
            .or(strategies.get("default"))
            .copied()
            .unwrap_or(BalancingStrategy::RoundRobin);

        // Select instance based on strategy
        let selected_instance = self.select_by_strategy(&healthy_instances, strategy, service_name)?;
        
        // Record the decision
        self.record_balancing_decision(service_name.to_string(), selected_instance.service_id, strategy);

        Ok(selected_instance.service_id)
    }

    /// Route a request to appropriate service instance
    pub fn route_request(&self, request: RoutingRequest) -> ServiceResult<RoutingResponse> {
        let service_name = &request.service_name;
        
        // Get available instances
        let routing_table = self.routing_table.read();
        let instances = routing_table.get(service_name)
            .ok_or(ServiceError::ServiceNotFound)?;

        let available_instances: Vec<ServiceId> = instances.iter()
            .filter(|instance| {
                instance.status == super::service::ServiceState::Running &&
                self.health_checker.is_healthy(instance.service_id)
            })
            .map(|instance| instance.service_id)
            .collect();

        if available_instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }

        // Select instance
        let selected_id = self.select_instance(service_name, &available_instances)?;
        
        // Get the selected instance details
        let selected_instance = instances.iter()
            .find(|instance| instance.service_id == selected_id)
            .ok_or(ServiceError::ServiceNotFound)?;

        // Update load statistics
        self.update_load_stats(selected_id);

        // Calculate load score
        let load_score = self.calculate_load_score(selected_id);

        // Estimate wait time
        let estimated_wait_time = self.estimate_wait_time(selected_id);

        let endpoint = ServiceEndpoint {
            service_id: selected_id,
            name: service_name.to_string(),
            address: selected_instance.host.clone(),
            port: selected_instance.port,
            protocol: super::config::Protocol::Http,
            health_status: super::service::HealthStatus::Healthy,
            metadata: BTreeMap::new(),
        };

        let strategies = self.strategies.read();
        let strategy = strategies.get(service_name)
            .or(strategies.get("default"))
            .copied()
            .unwrap_or(BalancingStrategy::RoundRobin);

        Ok(RoutingResponse {
            selected_instance: selected_id,
            endpoint,
            strategy_used: strategy,
            load_score,
            estimated_wait_time,
        })
    }

    /// Set balancing strategy for a service
    pub fn set_strategy(&self, service_name: String, strategy: BalancingStrategy) -> ServiceResult<()> {
        let mut strategies = self.strategies.write();
        strategies.insert(service_name, strategy);
        
        info!("Balancing strategy set for service: {:?}", strategy);
        Ok(())
    }

    /// Get current balancing statistics
    pub fn get_stats(&self) -> BalancingStats {
        let load_stats = self.load_stats.read();
        let history = self.balancing_history.read();

        let total_requests: u64 = load_stats.values()
            .map(|stats| stats.total_requests)
            .sum();

        let total_connections: u32 = load_stats.values()
            .map(|stats| stats.current_connections)
            .sum();

        let avg_response_time = if !load_stats.is_empty() {
            let total_response_time: f64 = load_stats.values()
                .map(|stats| stats.average_response_time)
                .sum();
            total_response_time / load_stats.len() as f64
        } else {
            0.0
        };

        BalancingStats {
            total_requests,
            total_connections,
            average_response_time: avg_response_time,
            decisions_made: history.len() as u64,
            active_circuit_breakers: 0, // Would be calculated from circuit breaker states
            load_distribution: load_stats.values().map(|s| s.total_requests).collect(),
        }
    }

    /// Get decision count (for overall service manager stats)
    pub fn get_decision_count(&self) -> u64 {
        let history = self.balancing_history.read();
        history.len() as u64
    }

    /// Internal methods
    fn select_by_strategy(&self, instances: &[&ServiceInstance], strategy: BalancingStrategy, service_name: &str) -> ServiceResult<&ServiceInstance> {
        match strategy {
            BalancingStrategy::RoundRobin => self.round_robin_select(instances),
            BalancingStrategy::LeastConnections => self.least_connections_select(instances),
            BalancingStrategy::WeightedRoundRobin => self.weighted_round_robin_select(instances),
            BalancingStrategy::WeightedLeastConnections => self.weighted_least_connections_select(instances),
            BalancingStrategy::Random => self.random_select(instances),
            BalancingStrategy::IpHash => self.ip_hash_select(instances, service_name),
            BalancingStrategy::ConsistentHash => self.consistent_hash_select(instances, service_name),
            BalancingStrategy::FastestResponse => self.fastest_response_select(instances),
            BalancingStrategy::HealthBased => self.health_based_select(instances),
        }
    }

    fn round_robin_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        if instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }
        
        let index = self.current_strategy.fetch_add(1, Ordering::SeqCst) % instances.len();
        Ok(instances[index])
    }

    fn least_connections_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        let mut min_connections = u32::MAX;
        let mut selected = instances[0];
        let mut min_response_time = u64::MAX;

        for instance in instances {
            let load_stats = self.load_stats.read();
            if let Some(stats) = load_stats.get(&instance.service_id) {
                // Prefer instances with fewer connections, then faster response times
                if stats.current_connections < min_connections || 
                   (stats.current_connections == min_connections && stats.average_response_time < min_response_time as f64) {
                    min_connections = stats.current_connections;
                    min_response_time = stats.average_response_time as u64;
                    selected = instance;
                }
            } else {
                // If no stats available, this is a new instance
                return Ok(instance);
            }
        }
        
        Ok(selected)
    }

    fn weighted_round_robin_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        if instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }
        
        // Select based on weights
        let total_weight: u32 = instances.iter().map(|i| i.weight).sum();
        if total_weight == 0 {
            return self.round_robin_select(instances);
        }
        
        let target_weight = (self.current_strategy.fetch_add(1, Ordering::SeqCst) % total_weight) + 1;
        let mut current_weight = 0;
        
        for instance in instances {
            current_weight += instance.weight;
            if current_weight >= target_weight {
                return Ok(instance);
            }
        }
        
        // Fallback to last instance
        Ok(instances[instances.len() - 1])
    }

    fn weighted_least_connections_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        let mut best_instance = instances[0];
        let mut best_score = f64::MAX;
        
        for instance in instances {
            let load_stats = self.load_stats.read();
            if let Some(stats) = load_stats.get(&instance.service_id) {
                // Calculate weighted score: (connections / weight)
                let weighted_score = if instance.weight > 0 {
                    stats.current_connections as f64 / instance.weight as f64
                } else {
                    stats.current_connections as f64
                };
                
                if weighted_score < best_score {
                    best_score = weighted_score;
                    best_instance = instance;
                }
            } else {
                // New instance with no connections gets highest priority
                return Ok(instance);
            }
        }
        
        Ok(best_instance)
    }

    fn random_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        if instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }
        
        let index = (crate::hal::get_random_u32() as usize) % instances.len();
        Ok(instances[index])
    }

    fn ip_hash_select(&self, instances: &[&ServiceInstance], client_ip: &str) -> ServiceResult<&ServiceInstance> {
        if instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }
        
        // Simple hash based on client IP
        let hash: usize = client_ip.chars().map(|c| c as usize).sum();
        let index = hash % instances.len();
        Ok(instances[index])
    }

    fn consistent_hash_select(&self, instances: &[&ServiceInstance], request_hash: &str) -> ServiceResult<&ServiceInstance> {
        if instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }
        
        // Simple consistent hash implementation
        let hash: usize = request_hash.chars().map(|c| c as usize).sum();
        let index = hash % instances.len();
        Ok(instances[index])
    }

    fn fastest_response_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        let mut fastest = instances[0];
        let mut fastest_time = u64::MAX;
        
        for instance in instances {
            let load_stats = self.load_stats.read();
            if let Some(stats) = load_stats.get(&instance.service_id) {
                if stats.average_response_time < fastest_time as f64 {
                    fastest_time = stats.average_response_time as u64;
                    fastest = instance;
                }
            } else {
                // New instance assumed to be fast
                return Ok(instance);
            }
        }
        
        Ok(fastest)
    }

    fn health_based_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        // Filter only healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|instance| self.health_checker.is_healthy(instance.service_id))
            .collect();
        
        if healthy_instances.is_empty() {
            // If no healthy instances, fall back to any instance
            return Err(ServiceError::ServiceNotFound);
        }
        
        // Use least connections among healthy instances
        self.least_connections_select(&healthy_instances)
    }
                    min_connections = stats.current_connections;
                    selected = instance;
                }
            }
        }

        Ok(selected)
    }

    fn weighted_round_robin_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        // Simplified weighted selection based on instance weight
        let total_weight: u32 = instances.iter().map(|i| i.weight).sum();
        
        if total_weight == 0 {
            return self.round_robin_select(instances);
        }

        let index = self.current_strategy.fetch_add(1, Ordering::SeqCst) % total_weight as usize;
        let mut current_weight = 0;

        for instance in instances {
            current_weight += instance.weight;
            if index < current_weight as usize {
                return Ok(instance);
            }
        }

        Ok(instances[0])
    }

    fn weighted_least_connections_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        let mut best_ratio = f32::MAX;
        let mut selected = instances[0];

        for instance in instances {
            let load_stats = self.load_stats.read();
            if let Some(stats) = load_stats.get(&instance.service_id) {
                if instance.weight > 0 {
                    let ratio = stats.current_connections as f32 / instance.weight as f32;
                    if ratio < best_ratio {
                        best_ratio = ratio;
                        selected = instance;
                    }
                }
            }
        }

        Ok(selected)
    }

    fn random_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        use core::num::Wrapping;
        
        if instances.is_empty() {
            return Err(ServiceError::ServiceNotFound);
        }
        
        let index = (Wrapping(self.current_strategy.fetch_add(1, Ordering::SeqCst)) * 1103515245 + 12345).0 as usize % instances.len();
        Ok(instances[index])
    }

    fn ip_hash_select(&self, instances: &[&ServiceInstance], _service_name: &str) -> ServiceResult<&ServiceInstance> {
        // Simplified IP hash - would use actual client IP
        let index = self.current_strategy.fetch_add(1, Ordering::SeqCst) % instances.len();
        Ok(instances[index])
    }

    fn consistent_hash_select(&self, instances: &[&ServiceInstance], _service_name: &str) -> ServiceResult<&ServiceInstance> {
        // Simplified consistent hash - would implement proper consistent hashing
        let index = self.current_strategy.fetch_add(1, Ordering::SeqCst) % instances.len();
        Ok(instances[index])
    }

    fn fastest_response_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        let mut fastest_response_time = u64::MAX;
        let mut selected = instances[0];

        for instance in instances {
            if let Some(response_time) = self.health_checker.get_response_time(instance.service_id) {
                if response_time < fastest_response_time {
                    fastest_response_time = response_time;
                    selected = instance;
                }
            }
        }

        Ok(selected)
    }

    fn health_based_select(&self, instances: &[&ServiceInstance]) -> ServiceResult<&ServiceInstance> {
        // Prioritize instances with better health scores
        let mut best_health_score = f32::MIN;
        let mut selected = instances[0];

        for instance in instances {
            let load_stats = self.load_stats.read();
            let health_score = if let Some(stats) = load_stats.get(&instance.service_id) {
                // Calculate health score based on various metrics
                let success_rate = if stats.total_requests > 0 {
                    stats.successful_requests as f32 / stats.total_requests as f32
                } else {
                    1.0
                };
                
                let response_time_score = if stats.average_response_time > 0.0 {
                    1.0 / (1.0 + stats.average_response_time as f32 / 1000.0)
                } else {
                    1.0
                };

                success_rate * 0.7 + response_time_score * 0.3
            } else {
                0.5 // Default score for new instances
            };

            if health_score > best_health_score {
                best_health_score = health_score;
                selected = instance;
            }
        }

        Ok(selected)
    }

    fn record_balancing_decision(&self, service_name: String, service_id: ServiceId, strategy: BalancingStrategy) {
        let load_stats = self.load_stats.read();
        let stats = load_stats.get(&service_id).cloned();

        let mut history = self.balancing_history.write();
        history.push_back(BalancingDecision {
            service_name,
            selected_instance: service_id,
            strategy_used: strategy,
            decision_time: get_current_time(),
            load_metrics: stats,
        });

        // Maintain history size
        while history.len() > 1000 {
            history.pop_front();
        }
    }

    fn update_load_stats(&self, service_id: ServiceId) {
        let mut load_stats = self.load_stats.write();
        if let Some(stats) = load_stats.get_mut(&service_id) {
            stats.total_requests += 1;
            stats.last_request_time = Some(get_current_time());
        }
    }

    fn calculate_load_score(&self, service_id: ServiceId) -> f32 {
        let load_stats = self.load_stats.read();
        if let Some(stats) = load_stats.get(&service_id) {
            // Calculate load score based on connections and response time
            let connection_score = 1.0 / (1.0 + stats.current_connections as f32 / 100.0);
            let response_time_score = 1.0 / (1.0 + stats.average_response_time as f32 / 1000.0);
            
            (connection_score + response_time_score) / 2.0
        } else {
            1.0 // Default score for new instances
        }
    }

    fn estimate_wait_time(&self, service_id: ServiceId) -> Option<u64> {
        let load_stats = self.load_stats.read();
        if let Some(stats) = load_stats.get(&service_id) {
            if stats.current_connections > 0 {
                // Simple estimation based on current load
                Some(stats.current_connections as u64 * 10) // 10ms per connection
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Load Balancing Statistics
#[derive(Debug, Clone)]
pub struct BalancingStats {
    pub total_requests: u64,
    pub total_connections: u32,
    pub average_response_time: f64,
    pub decisions_made: u64,
    pub active_circuit_breakers: u32,
    pub load_distribution: Vec<u64>,
}

impl LoadBalancer {
    /// Create load balancer with custom health checker
    pub fn with_health_checker(strategy: BalancingStrategy, health_checker: Box<dyn ServiceHealthChecker>) -> Self {
        let mut strategies = BTreeMap::new();
        strategies.insert("default".to_string(), strategy);

        LoadBalancer {
            strategies: RwLock::new(strategies),
            routing_table: RwLock::new(BTreeMap::new()),
            health_checker,
            load_stats: RwLock::new(BTreeMap::new()),
            balancing_history: RwLock::new(VecDeque::new()),
            current_strategy: AtomicUsize::new(strategy as usize),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancer_creation() {
        let balancer = LoadBalancer::with_strategy(BalancingStrategy::RoundRobin);
        assert_eq!(balancer.get_decision_count(), 0);
    }

    #[test]
    fn test_balancing_strategy_enum() {
        assert_eq!(BalancingStrategy::RoundRobin as u8, 0);
        assert_eq!(BalancingStrategy::LeastConnections as u8, 1);
        assert_eq!(BalancingStrategy::Random as u8, 4);
        assert_eq!(BalancingStrategy::HealthBased as u8, 8);
    }

    #[test]
    fn test_instance_load_stats() {
        let stats = InstanceLoadStats {
            service_id: ServiceId(1),
            current_connections: 10,
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            average_response_time: 25.5,
            last_request_time: Some(1000),
            last_health_check: Some(900),
        };

        assert_eq!(stats.current_connections, 10);
        assert_eq!(stats.total_requests, 100);
        assert_eq!(stats.successful_requests, 95);
    }

    #[test]
    fn test_request_priority() {
        assert_eq!(RequestPriority::Low as u8, 0);
        assert_eq!(RequestPriority::Normal as u8, 1);
        assert_eq!(RequestPriority::Critical as u8, 3);
    }

    #[test]
    fn test_circuit_breaker_state() {
        assert_eq!(CircuitBreakerState::Closed as u8, 0);
        assert_eq!(CircuitBreakerState::Open as u8, 1);
        assert_eq!(CircuitBreakerState::HalfOpen as u8, 2);
    }

    #[test]
    fn test_balancing_decision_structure() {
        let decision = BalancingDecision {
            service_name: "test-service".to_string(),
            selected_instance: ServiceId(1),
            strategy_used: BalancingStrategy::RoundRobin,
            decision_time: 1000,
            load_metrics: None,
        };

        assert_eq!(decision.service_name, "test-service");
        assert_eq!(decision.selected_instance, ServiceId(1));
        assert_eq!(decision.strategy_used, BalancingStrategy::RoundRobin);
    }
}