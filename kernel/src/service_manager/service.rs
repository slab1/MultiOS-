//! Service Core Definitions
//! 
//! This module defines the core service structures, states, and types
//! used throughout the service management framework.

use spin::{Mutex, RwLock};
use alloc::string::String;
use alloc::vec::Vec;
use bitflags::bitflags;

/// Unique service identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServiceId(pub u64);

/// Service Handle - Smart pointer to service instance
pub struct ServiceHandle {
    pub service_id: ServiceId,
    service: RwLock<Service>,
}

impl ServiceHandle {
    pub fn new(service_id: ServiceId, descriptor: ServiceDescriptor) -> Self {
        ServiceHandle {
            service_id,
            service: RwLock::new(Service::new(service_id, descriptor)),
        }
    }
}

impl core::ops::Deref for ServiceHandle {
    type Target = RwLock<Service>;
    
    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

/// Service Lifecycle States
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceState {
    Stopped = 0,
    Starting = 1,
    Running = 2,
    Stopping = 3,
    Failed = 4,
    Disabled = 5,
    Paused = 6,
}

/// Service Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceType {
    SystemService = 0,
    UserService = 1,
    ServiceGroup = 2,
    MonitoringService = 3,
    LoadBalancerService = 4,
    DiscoveryService = 5,
}

/// Service Flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ServiceFlags: u32 {
        const AUTO_START = 0b00000001;
        const RESTARTABLE = 0b00000010;
        const CRITICAL = 0b00000100;
        const ISOLATED = 0b00001000;
        const MONITORED = 0b00010000;
        const LOAD_BALANCED = 0b00100000;
        const DISCOVERABLE = 0b01000000;
        const FAULT_TOLERANT = 0b10000000;
    }
}

/// Service Descriptor - Defines service metadata and configuration
#[derive(Debug, Clone)]
pub struct ServiceDescriptor {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub service_type: ServiceType,
    pub dependencies: Vec<ServiceDependency>,
    pub resource_limits: Option<ResourceLimits>,
    pub isolation_level: IsolationLevel,
    pub auto_restart: bool,
    pub restart_delay: u32,
    pub max_restarts: u32,
    pub health_check_interval: u32,
    pub tags: Vec<String>,
}

/// Service Dependency
#[derive(Debug, Clone)]
pub struct ServiceDependency {
    pub service_id: ServiceId,
    pub required: bool,
    pub timeout: u32,
    pub version_constraint: Option<String>,
}

/// Resource Limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_cpu_percent: u8,
    pub max_file_descriptors: u32,
    pub max_threads: u32,
    pub network_bandwidth: Option<u64>, // bytes per second
    pub disk_io: Option<u64>, // IO operations per second
}

/// Isolation Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IsolationLevel {
    None = 0,
    Process = 1,
    Container = 2,
    VirtualMachine = 3,
    Namespace = 4,
}

/// Main Service Structure
pub struct Service {
    pub service_id: ServiceId,
    pub descriptor: ServiceDescriptor,
    pub state: ServiceState,
    pub start_time: Option<u64>,
    pub stop_time: Option<u64>,
    pub restart_count: u32,
    pub last_health_check: Option<u64>,
    pub health_status: HealthStatus,
    pub enabled: bool,
    pub config: Option<super::config::ServiceConfig>,
    pub metrics: ServiceMetrics,
    pub instance_id: String,
    pub process_id: Option<u32>,
    pub pid: Option<u32>,
}

/// Health Status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy = 0,
    Degraded = 1,
    Unhealthy = 2,
    Unknown = 3,
}

/// Service Metrics
#[derive(Debug, Clone)]
pub struct ServiceMetrics {
    pub uptime: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub memory_usage: usize,
    pub cpu_usage: f32,
    pub network_io: NetworkIO,
    pub disk_io: DiskIO,
}

/// Network I/O Statistics
#[derive(Debug, Clone)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub connection_count: u32,
}

/// Disk I/O Statistics
#[derive(Debug, Clone)]
pub struct DiskIO {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_operations: u64,
    pub write_operations: u64,
}

/// Service Instance Information
#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub service_id: ServiceId,
    pub instance_id: String,
    pub host: String,
    pub port: Option<u16>,
    pub endpoint: String,
    pub weight: u32,
    pub status: ServiceState,
    pub last_health_check: u64,
}

/// Service Group Configuration
#[derive(Debug, Clone)]
pub struct ServiceGroup {
    pub group_name: String,
    pub member_services: Vec<ServiceId>,
    pub balancing_strategy: super::load_balancer::BalancingStrategy,
    pub failover_strategy: FailoverStrategy,
    pub minimum_instances: u32,
    pub maximum_instances: u32,
}

/// Failover Strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FailoverStrategy {
    None = 0,
    Manual = 1,
    Automatic = 2,
    CircuitBreaker = 3,
}

impl Service {
    pub fn new(service_id: ServiceId, descriptor: ServiceDescriptor) -> Self {
        Service {
            service_id,
            descriptor,
            state: ServiceState::Stopped,
            start_time: None,
            stop_time: None,
            restart_count: 0,
            last_health_check: None,
            health_status: HealthStatus::Unknown,
            enabled: descriptor.auto_restart,
            config: None,
            metrics: ServiceMetrics {
                uptime: 0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time: 0.0,
                memory_usage: 0,
                cpu_usage: 0.0,
                network_io: NetworkIO {
                    bytes_sent: 0,
                    bytes_received: 0,
                    packets_sent: 0,
                    packets_received: 0,
                    connection_count: 0,
                },
                disk_io: DiskIO {
                    bytes_read: 0,
                    bytes_written: 0,
                    read_operations: 0,
                    write_operations: 0,
                },
            },
            instance_id: format!("{}-{}", descriptor.name, service_id.0),
            process_id: None,
            pid: None,
        }
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        self.state == ServiceState::Running
    }

    /// Check if service is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_status == HealthStatus::Healthy
    }

    /// Check if service can be restarted
    pub fn can_restart(&self) -> bool {
        self.descriptor.auto_restart && 
        self.restart_count < self.descriptor.max_restarts &&
        self.enabled
    }

    /// Update service metrics
    pub fn update_metrics(&mut self, new_metrics: ServiceMetrics) {
        self.metrics = new_metrics;
    }

    /// Get service uptime
    pub fn get_uptime(&self) -> u64 {
        if let Some(start_time) = self.start_time {
            unsafe { crate::hal::get_current_time() - start_time }
        } else {
            0
        }
    }

    /// Get service availability percentage
    pub fn get_availability(&self) -> f32 {
        let total_time = self.metrics.uptime;
        if total_time == 0 {
            return 0.0;
        }
        
        let downtime = self.restart_count as u64 * self.descriptor.restart_delay as u64;
        let uptime = total_time.saturating_sub(downtime);
        
        (uptime as f32 / total_time as f32) * 100.0
    }
}

impl ServiceHandle {
    pub fn get_service(&self) -> Option<spin::RwLockReadGuard<Service>> {
        self.service.read().ok()
    }

    pub fn get_service_mut(&self) -> Option<spin::RwLockWriteGuard<Service>> {
        self.service.write().ok()
    }

    pub fn get_service_id(&self) -> ServiceId {
        self.service_id
    }
}

/// Service Registry Entry
#[derive(Debug, Clone)]
pub struct ServiceRegistryEntry {
    pub service_id: ServiceId,
    pub name: String,
    pub instance: ServiceInstance,
    pub registered_at: u64,
    pub last_updated: u64,
}

/// Service Discovery Query
#[derive(Debug, Clone)]
pub struct ServiceDiscoveryQuery {
    pub name_pattern: Option<String>,
    pub service_type: Option<ServiceType>,
    pub tags: Vec<String>,
    pub healthy_only: bool,
    pub include_metadata: bool,
}

/// Service Health Check Result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub service_id: ServiceId,
    pub healthy: bool,
    pub response_time: u64, // microseconds
    pub error_message: Option<String>,
    pub timestamp: u64,
}

/// Service Event
#[derive(Debug, Clone)]
pub enum ServiceEvent {
    ServiceStarted(ServiceId),
    ServiceStopped(ServiceId),
    ServiceFailed(ServiceId),
    ServiceRestarted(ServiceId),
    ServiceHealthChanged(ServiceId, HealthStatus),
    ServiceDiscovered(ServiceId, String),
    ServiceLoadBalanced(ServiceId, ServiceId),
    ServiceRecovered(ServiceId),
    ConfigurationChanged(ServiceId),
    DependencyChanged(ServiceId, ServiceId),
}

/// Service Event Handler
pub trait ServiceEventHandler: Send + Sync {
    fn handle_event(&self, event: &ServiceEvent);
}

/// Service Lifecycle Hook
pub trait ServiceLifecycleHook: Send + Sync {
    fn on_start(&self, service_id: ServiceId) -> Result<(), super::ServiceError>;
    fn on_stop(&self, service_id: ServiceId) -> Result<(), super::ServiceError>;
    fn on_restart(&self, service_id: ServiceId) -> Result<(), super::ServiceError>;
    fn on_health_change(&self, service_id: ServiceId, status: HealthStatus);
}

/// Default implementation for ServiceLifecycleHook
#[derive(Debug, Clone)]
pub struct DefaultLifecycleHook;

impl ServiceLifecycleHook for DefaultLifecycleHook {
    fn on_start(&self, service_id: ServiceId) -> Result<(), super::ServiceError> {
        info!("Service started: {}", service_id.0);
        Ok(())
    }

    fn on_stop(&self, service_id: ServiceId) -> Result<(), super::ServiceError> {
        info!("Service stopped: {}", service_id.0);
        Ok(())
    }

    fn on_restart(&self, service_id: ServiceId) -> Result<(), super::ServiceError> {
        info!("Service restarted: {}", service_id.0);
        Ok(())
    }

    fn on_health_change(&self, service_id: ServiceId, status: HealthStatus) {
        info!("Service health changed: {} -> {:?}", service_id.0, status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let descriptor = ServiceDescriptor {
            name: "test-service".to_string(),
            display_name: "Test Service".to_string(),
            description: Some("A test service".to_string()),
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 1000,
            max_restarts: 3,
            health_check_interval: 30000,
            tags: vec!["test".to_string()],
        };

        let service = Service::new(ServiceId(1), descriptor);
        assert_eq!(service.state, ServiceState::Stopped);
        assert_eq!(service.health_status, HealthStatus::Unknown);
        assert_eq!(service.enabled, true);
    }

    #[test]
    fn test_service_running_check() {
        let descriptor = ServiceDescriptor {
            name: "test-service".to_string(),
            display_name: "Test Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: IsolationLevel::None,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let mut service = Service::new(ServiceId(1), descriptor);
        assert!(!service.is_running());
        
        service.state = ServiceState::Running;
        assert!(service.is_running());
    }

    #[test]
    fn test_service_restart_capability() {
        let descriptor = ServiceDescriptor {
            name: "test-service".to_string(),
            display_name: "Test Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: IsolationLevel::None,
            auto_restart: true,
            restart_delay: 1000,
            max_restarts: 3,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let mut service = Service::new(ServiceId(1), descriptor);
        assert!(service.can_restart());
        
        service.restart_count = 3;
        assert!(!service.can_restart());
    }

    #[test]
    fn test_service_state_enum() {
        assert_eq!(ServiceState::Stopped as u8, 0);
        assert_eq!(ServiceState::Starting as u8, 1);
        assert_eq!(ServiceState::Running as u8, 2);
        assert_eq!(ServiceState::Failed as u8, 4);
    }

    #[test]
    fn test_service_type_enum() {
        assert_eq!(ServiceType::SystemService as u8, 0);
        assert_eq!(ServiceType::UserService as u8, 1);
        assert_eq!(ServiceType::ServiceGroup as u8, 2);
    }

    #[test]
    fn test_health_status_enum() {
        assert_eq!(HealthStatus::Healthy as u8, 0);
        assert_eq!(HealthStatus::Degraded as u8, 1);
        assert_eq!(HealthStatus::Unhealthy as u8, 2);
        assert_eq!(HealthStatus::Unknown as u8, 3);
    }
}