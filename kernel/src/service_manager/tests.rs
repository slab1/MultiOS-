//! MultiOS Service Management Framework - Test Suite
//! 
//! This module contains comprehensive tests for the service management framework,
//! demonstrating functionality and validating the implementation.

#![cfg(test)]

use spin::Mutex;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

use super::*;
use super::service::{ServiceDescriptor, ServiceType, ServiceState, HealthStatus, ResourceLimits, IsolationLevel};
use super::config::{ServiceConfigManager, ServiceConfig, ConfigValue, NetworkConfig, Protocol, LoggingConfig, LogLevel, LogFormat, LogOutput, MonitoringConfig, SecurityConfig, ResourceConfig};
use super::discovery::{ServiceRegistry, ServiceDiscovery};
use super::monitoring::{ServiceMonitor, HealthChecker};
use super::load_balancer::{LoadBalancer, BalancingStrategy};
use super::fault_tolerance::{FaultDetector, RecoveryManager, RecoveryPolicy, RecoveryStrategy, BackoffStrategy};

// Test helper functions
fn create_test_service_descriptor(name: &str, service_type: ServiceType) -> ServiceDescriptor {
    ServiceDescriptor {
        name: name.to_string(),
        display_name: format!("{} Service", name),
        description: Some(format!("Test service: {}", name)),
        service_type,
        dependencies: Vec::new(),
        resource_limits: Some(ResourceLimits {
            max_memory: 134217728, // 128MB
            max_cpu_percent: 80,
            max_file_descriptors: 1024,
            max_threads: 64,
            network_bandwidth: None,
            disk_io: None,
        }),
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 1000,
        max_restarts: 3,
        health_check_interval: 30000,
        tags: vec!["test".to_string(), "demo".to_string()],
    }
}

fn create_test_service_config(service_name: &str) -> ServiceConfig {
    ServiceConfig {
        service_id: None,
        name: service_name.to_string(),
        version: "1.0.0".to_string(),
        settings: BTreeMap::new(),
        environment: BTreeMap::new(),
        secrets: BTreeMap::new(),
        network: NetworkConfig {
            bind_address: "0.0.0.0".to_string(),
            bind_port: Some(8080),
            protocol: Protocol::Http,
            ssl_enabled: false,
            ssl_certificate: None,
            ssl_key: None,
            max_connections: 1000,
            connection_timeout: 30000,
            keep_alive: true,
        },
        logging: LoggingConfig {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Console,
            file_path: None,
            max_file_size: 10485760,
            max_files: 5,
            rotate_on_size: true,
            timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
        },
        monitoring: MonitoringConfig {
            health_check_enabled: true,
            health_check_interval: 30000,
            health_check_timeout: 5000,
            metrics_enabled: true,
            metrics_endpoint: Some("/metrics".to_string()),
            alert_thresholds: BTreeMap::new(),
        },
        security: SecurityConfig {
            user: Some("service".to_string()),
            group: Some("service".to_string()),
            capabilities: Vec::new(),
            namespaces: Vec::new(),
            selinux_context: None,
            apparmor_profile: None,
            secure_bits: 0,
        },
        resources: ResourceConfig {
            cpu_limit: Some(1.0),
            memory_limit: Some(134217728),
            disk_limit: None,
            network_limit: None,
            file_descriptor_limit: Some(1024),
            thread_limit: Some(64),
            nice_level: Some(0),
            oom_score_adjust: Some(0),
        },
    }
}

#[test]
fn test_service_manager_initialization() {
    // Test that the service manager can be initialized
    let result = ServiceManager::init();
    assert!(result.is_ok());
}

#[test]
fn test_service_registration() {
    let manager = ServiceManager::new();
    let descriptor = create_test_service_descriptor("test-service", ServiceType::UserService);
    
    let result = manager.register_service(descriptor);
    assert!(result.is_ok());
    
    let service_id = result.unwrap();
    assert!(service_id.0 > 0);
}

#[test]
fn test_service_lifecycle() {
    let manager = ServiceManager::new();
    let descriptor = create_test_service_descriptor("lifecycle-test", ServiceType::UserService);
    
    // Register service
    let service_id = manager.register_service(descriptor).unwrap();
    
    // Start service
    let result = manager.start_service(service_id);
    assert!(result.is_ok());
    
    // Check service state
    let services = manager.services.read();
    let service_handle = services.get(&service_id).unwrap();
    let service = service_handle.lock();
    assert_eq!(service.state, ServiceState::Running);
    
    // Stop service
    let result = manager.stop_service(service_id);
    assert!(result.is_ok());
    
    // Check service state
    let service = service_handle.lock();
    assert_eq!(service.state, ServiceState::Stopped);
}

#[test]
fn test_service_configuration() {
    let manager = ServiceManager::new();
    let descriptor = create_test_service_descriptor("config-test", ServiceType::UserService);
    
    // Create configuration
    let config = create_test_service_config("config-test");
    
    // Register service
    let service_id = manager.register_service(descriptor).unwrap();
    
    // Save configuration
    let result = manager.config_manager.save_config(&service_id, config.clone());
    assert!(result.is_ok());
    
    // Load configuration
    let loaded_config = manager.config_manager.load_config(&service_id);
    assert!(loaded_config.is_ok());
    
    let loaded = loaded_config.unwrap();
    assert_eq!(loaded.name, "config-test");
    assert_eq!(loaded.version, "1.0.0");
}

#[test]
fn test_service_discovery() {
    let discovery = ServiceDiscovery::new();
    
    // Test pattern-based discovery
    let result = discovery.discover_by_pattern("test-*");
    assert!(result.is_ok());
    
    let service_ids = result.unwrap();
    assert!(service_ids.is_empty()); // No services registered yet
    
    // Test filter-based discovery
    let filter = ServiceFilter {
        name_pattern: Some("test".to_string()),
        tags: Vec::new(),
        service_types: vec![ServiceType::UserService],
        healthy_only: false,
        available_only: false,
        max_results: Some(10),
    };
    
    let result = discovery.discover_by_filter(&filter);
    assert!(result.is_ok());
    
    let endpoints = result.unwrap();
    assert!(endpoints.is_empty());
}

#[test]
fn test_service_monitoring() {
    let monitor = ServiceMonitor::new();
    
    // Test health check
    let result = monitor.check_health(ServiceId(1));
    assert!(result.is_err()); // Service doesn't exist
    
    // Test health report
    let result = monitor.get_health_report(ServiceId(1));
    assert!(result.is_err()); // Service doesn't exist
    
    // Test statistics
    let stats = monitor.get_stats();
    assert_eq!(stats.total_health_checks, 0);
    assert_eq!(stats.failed_health_checks, 0);
}

#[test]
fn test_load_balancer() {
    let balancer = LoadBalancer::with_strategy(BalancingStrategy::RoundRobin);
    
    // Test statistics
    let stats = balancer.get_stats();
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.total_connections, 0);
    assert_eq!(stats.decisions_made, 0);
    
    // Test instance selection (should fail with no instances)
    let instances = vec![ServiceId(1)];
    let result = balancer.select_instance("test-service", &instances);
    assert!(result.is_err());
}

#[test]
fn test_fault_tolerance() {
    let detector = FaultDetector::new();
    let recovery = RecoveryManager::new();
    
    // Test fault detection
    let result = detector.detect_fault(ServiceId(1), &ServiceError::ServiceNotFound);
    assert!(result.is_ok());
    
    let fault_type = result.unwrap();
    assert_eq!(fault_type, super::fault_tolerance::FaultType::UnhandledException);
    
    // Test recovery statistics
    let recovery_stats = recovery.get_stats();
    assert_eq!(recovery_stats.total_recovery_attempts, 0);
    assert_eq!(recovery_stats.successful_recoveries, 0);
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
    assert_eq!(ServiceType::MonitoringService as u8, 3);
    assert_eq!(ServiceType::LoadBalancerService as u8, 4);
}

#[test]
fn test_balancing_strategy_enum() {
    assert_eq!(BalancingStrategy::RoundRobin as u8, 0);
    assert_eq!(BalancingStrategy::LeastConnections as u8, 1);
    assert_eq!(BalancingStrategy::WeightedRoundRobin as u8, 2);
    assert_eq!(BalancingStrategy::Random as u8, 4);
}

#[test]
fn test_health_status_enum() {
    assert_eq!(HealthStatus::Healthy as u8, 0);
    assert_eq!(HealthStatus::Degraded as u8, 1);
    assert_eq!(HealthStatus::Unhealthy as u8, 2);
    assert_eq!(HealthStatus::Unknown as u8, 3);
}

#[test]
fn test_config_value_types() {
    let string_val = ConfigValue::String { value: "test".to_string() };
    let int_val = ConfigValue::Integer { value: 42 };
    let bool_val = ConfigValue::Boolean { value: true };
    let float_val = ConfigValue::Float { value: 3.14 };
    
    assert_eq!(string_val.as_string(), Some("test"));
    assert_eq!(int_val.as_i64(), Some(42));
    assert_eq!(bool_val.as_bool(), Some(true));
    assert_eq!(float_val.as_f64(), Some(3.14));
}

#[test]
fn test_network_protocol_enum() {
    assert_eq!(Protocol::Http as u8, 0);
    assert_eq!(Protocol::Https as u8, 1);
    assert_eq!(Protocol::Tcp as u8, 2);
    assert_eq!(Protocol::Udp as u8, 3);
    assert_eq!(Protocol::UnixSocket as u8, 4);
}

#[test]
fn test_log_level_enum() {
    assert_eq!(LogLevel::Debug as u8, 0);
    assert_eq!(LogLevel::Info as u8, 1);
    assert_eq!(LogLevel::Warning as u8, 2);
    assert_eq!(LogLevel::Error as u8, 3);
    assert_eq!(LogLevel::Critical as u8, 4);
}

#[test]
fn test_recovery_strategy_enum() {
    assert_eq!(RecoveryStrategy::None as u8, 0);
    assert_eq!(RecoveryStrategy::Restart as u8, 1);
    assert_eq!(RecoveryStrategy::RestartWithDelay as u8, 2);
    assert_eq!(RecoveryStrategy::Failover as u8, 5);
}

#[test]
fn test_backoff_strategy_enum() {
    assert_eq!(BackoffStrategy::None as u8, 0);
    assert_eq!(BackoffStrategy::Linear as u8, 1);
    assert_eq!(BackoffStrategy::Exponential as u8, 2);
    assert_eq!(BackoffStrategy::Fixed as u8, 3);
}

#[test]
fn test_isolation_level_enum() {
    assert_eq!(IsolationLevel::None as u8, 0);
    assert_eq!(IsolationLevel::Process as u8, 1);
    assert_eq!(IsolationLevel::Container as u8, 2);
    assert_eq!(IsolationLevel::VirtualMachine as u8, 3);
}

#[test]
fn test_service_statistics() {
    let manager = ServiceManager::new();
    
    // Initially no services
    let stats = manager.get_stats();
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.running_services, 0);
    assert_eq!(stats.stopped_services, 0);
    assert_eq!(stats.failed_services, 0);
}

#[test]
fn test_circular_dependency_detection() {
    let manager = ServiceManager::new();
    
    // This test would need to be expanded to actually create services with dependencies
    // For now, just test that the error type exists
    assert_eq!(ServiceError::CircularDependency as u8, 17);
}

#[test]
fn test_service_manager_state_enum() {
    assert_eq!(super::ServiceManagerState::Initialized as u8, 0);
    assert_eq!(super::ServiceManagerState::Running as u8, 1);
    assert_eq!(super::ServiceManagerState::Paused as u8, 2);
    assert_eq!(super::ServiceManagerState::Stopped as u8, 3);
    assert_eq!(super::ServiceManagerState::Error as u8, 4);
}

#[test]
fn test_request_priority_enum() {
    assert_eq!(super::load_balancer::RequestPriority::Low as u8, 0);
    assert_eq!(super::load_balancer::RequestPriority::Normal as u8, 1);
    assert_eq!(super::load_balancer::RequestPriority::High as u8, 2);
    assert_eq!(super::load_balancer::RequestPriority::Critical as u8, 3);
}

#[test]
fn test_fault_severity_enum() {
    assert_eq!(super::fault_tolerance::FaultSeverity::Info as u8, 0);
    assert_eq!(super::fault_tolerance::FaultSeverity::Warning as u8, 1);
    assert_eq!(super::fault_tolerance::FaultSeverity::Error as u8, 2);
    assert_eq!(super::fault_tolerance::FaultSeverity::Critical as u8, 3);
    assert_eq!(super::fault_tolerance::FaultSeverity::Fatal as u8, 4);
}

#[test]
fn test_detection_sensitivity_enum() {
    assert_eq!(super::fault_tolerance::DetectionSensitivity::Low as u8, 0);
    assert_eq!(super::fault_tolerance::DetectionSensitivity::Normal as u8, 1);
    assert_eq!(super::fault_tolerance::DetectionSensitivity::High as u8, 2);
    assert_eq!(super::fault_tolerance::DetectionSensitivity::Paranoid as u8, 3);
}

#[test]
fn test_comparison_operator_enum() {
    assert_eq!(super::config::ComparisonOperator::GreaterThan as u8, 0);
    assert_eq!(super::config::ComparisonOperator::LessThan as u8, 1);
    assert_eq!(super::config::ComparisonOperator::Equal as u8, 2);
    assert_eq!(super::config::ComparisonOperator::NotEqual as u8, 3);
    assert_eq!(super::config::ComparisonOperator::GreaterEqual as u8, 4);
    assert_eq!(super::config::ComparisonOperator::LessEqual as u8, 5);
}

#[test]
fn test_service_error_enum() {
    assert_eq!(ServiceError::ServiceNotFound as u8, 0);
    assert_eq!(ServiceError::ServiceAlreadyExists as u8, 1);
    assert_eq!(ServiceError::ServiceNotRunning as u8, 2);
    assert_eq!(ServiceError::ServiceNotStopped as u8, 3);
    assert_eq!(ServiceError::ConfigurationError as u8, 4);
    assert_eq!(ServiceError::DependencyError as u8, 5);
    assert_eq!(ServiceError::PermissionDenied as u8, 6);
    assert_eq!(ServiceError::ResourceExhausted as u8, 7);
    assert_eq!(ServiceError::InvalidConfiguration as u8, 8);
    assert_eq!(ServiceError::ServiceFailed as u8, 9);
    assert_eq!(ServiceError::DiscoveryError as u8, 10);
    assert_eq!(ServiceError::LoadBalancerError as u8, 11);
    assert_eq!(ServiceError::FaultToleranceError as u8, 12);
    assert_eq!(ServiceError::InvalidServiceHandle as u8, 13);
    assert_eq!(ServiceError::ServiceTimeout as u8, 14);
    assert_eq!(ServiceError::HealthCheckFailed as u8, 15);
    assert_eq!(ServiceError::CircularDependency as u8, 16);
}

// Integration test - Full service lifecycle
#[test]
fn test_full_service_lifecycle() {
    let mut manager = ServiceManager::new();
    
    // Initialize manager
    let result = ServiceManager::init();
    assert!(result.is_ok());
    
    let result = ServiceManager::start();
    assert!(result.is_ok());
    
    // Create a test service
    let descriptor = create_test_service_descriptor("integration-test", ServiceType::UserService);
    let service_id = manager.register_service(descriptor).unwrap();
    
    // Start the service
    let result = manager.start_service(service_id);
    assert!(result.is_ok());
    
    // Check that it's running
    let services = manager.services.read();
    let service_handle = services.get(&service_id).unwrap();
    let service = service_handle.lock();
    assert_eq!(service.state, ServiceState::Running);
    assert!(service.is_running());
    assert_eq!(service.health_status, HealthStatus::Healthy); // Would be updated by monitoring
    
    // Stop the service
    let result = manager.stop_service(service_id);
    assert!(result.is_ok());
    
    // Check that it's stopped
    let service = service_handle.lock();
    assert_eq!(service.state, ServiceState::Stopped);
    assert!(!service.is_running());
    
    // Enable the service
    let result = manager.enable_service(service_id);
    assert!(result.is_ok());
    
    // Check that it's enabled
    let service = service_handle.lock();
    assert!(service.enabled);
    
    // Disable the service
    let result = manager.disable_service(service_id);
    assert!(result.is_ok());
    
    // Check that it's disabled
    let service = service_handle.lock();
    assert!(!service.enabled);
    
    // Unregister the service
    let result = manager.unregister_service(service_id);
    assert!(result.is_ok());
    
    // Verify service is gone
    let services = manager.services.read();
    assert!(!services.contains_key(&service_id));
}

// Test service discovery with actual services
#[test]
fn test_service_discovery_integration() {
    let discovery = ServiceDiscovery::new();
    let registry = ServiceRegistry::new();
    
    // Initialize components
    let result = registry.init();
    assert!(result.is_ok());
    
    let result = discovery.init();
    assert!(result.is_ok());
    
    // Create test services
    let mut test_services = Vec::new();
    
    for i in 0..5 {
        let descriptor = create_test_service_descriptor(
            &format!("test-service-{}", i), 
            ServiceType::UserService
        );
        
        // Create service handle (simplified for testing)
        let service_handle = super::service::ServiceHandle::new(
            ServiceId(i as u64 + 100), 
            descriptor
        );
        
        // Register service
        let result = registry.register_service(service_handle);
        if result.is_ok() {
            test_services.push(ServiceId(i as u64 + 100));
        }
    }
    
    // Test discovery by pattern
    let result = discovery.discover_by_pattern("test-service-*");
    assert!(result.is_ok());
    
    let discovered_services = result.unwrap();
    // Should find some services (implementation dependent)
    
    // Test discovery by name
    let result = discovery.lookup_by_name("test-service-1");
    assert!(result.is_ok());
    
    let endpoints = result.unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].name, "test-service-1");
}

// Test load balancing with service instances
#[test]
fn test_load_balancing_integration() {
    let balancer = LoadBalancer::with_strategy(BalancingStrategy::RoundRobin);
    
    // Initialize load balancer
    let result = balancer.init();
    assert!(result.is_ok());
    
    // Add test service instances
    for i in 0..3 {
        let instance = super::service::ServiceInstance {
            service_id: ServiceId(i as u64 + 200),
            instance_id: format!("instance-{}", i),
            host: "localhost".to_string(),
            port: Some(8080 + i as u16),
            endpoint: format!("localhost:{}", 8080 + i as u16),
            weight: 100,
            status: ServiceState::Running,
            last_health_check: None,
        };
        
        let result = balancer.add_instance("test-service".to_string(), instance);
        assert!(result.is_ok());
    }
    
    // Test routing requests
    let request = super::load_balancer::RoutingRequest {
        service_name: "test-service".to_string(),
        client_ip: Some("192.168.1.100".to_string()),
        request_hash: None,
        priority: super::load_balancer::RequestPriority::Normal,
    };
    
    let result = balancer.route_request(request);
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(response.selected_instance.0 >= 200);
    assert!(response.selected_instance.0 < 203);
    
    // Test different balancing strategies
    let strategies = vec![
        BalancingStrategy::RoundRobin,
        BalancingStrategy::LeastConnections,
        BalancingStrategy::Random,
    ];
    
    for strategy in strategies {
        let result = balancer.set_strategy("test-service".to_string(), strategy);
        assert!(result.is_ok());
    }
    
    // Get statistics
    let stats = balancer.get_stats();
    assert!(stats.total_requests >= 1);
}

// Test fault tolerance with simulated failures
#[test]
fn test_fault_tolerance_integration() {
    let detector = FaultDetector::new();
    let recovery = RecoveryManager::new();
    
    // Initialize components
    let result = detector.init();
    assert!(result.is_ok());
    
    let result = recovery.init();
    assert!(result.is_ok());
    
    // Create recovery policy
    let policy = RecoveryPolicy {
        service_id: ServiceId(300),
        max_recovery_attempts: 3,
        recovery_strategy: RecoveryStrategy::RestartWithDelay,
        backoff_strategy: BackoffStrategy::Exponential {
            initial_delay_ms: 1000,
            multiplier: 2.0,
            max_delay_ms: 30000,
        },
        escalation_policy: super::fault_tolerance::EscalationPolicy {
            enabled: false,
            escalation_levels: Vec::new(),
        },
        recovery_timeout: 30000,
    };
    
    // Set recovery policy
    let result = recovery.set_recovery_policy(ServiceId(300), policy);
    assert!(result.is_ok());
    
    // Simulate service failure
    let result = detector.detect_fault(ServiceId(300), &ServiceError::ServiceFailed);
    assert!(result.is_ok());
    
    // Trigger recovery
    let result = recovery.handle_fault(ServiceId(300), &super::fault_tolerance::FaultType::UnhandledException);
    assert!(result.is_ok());
    
    // Check statistics
    let detection_stats = detector.get_stats();
    assert_eq!(detection_stats.total_detections, 1);
    
    let recovery_stats = recovery.get_stats();
    assert_eq!(recovery_stats.total_recovery_attempts, 1);
}

// Test configuration management
#[test]
fn test_configuration_management_integration() {
    let config_manager = ServiceConfigManager::new();
    
    // Initialize config manager
    let result = config_manager.init();
    assert!(result.is_ok());
    
    // Create test configuration
    let mut config = create_test_service_config("config-integration-test");
    
    // Add some custom settings
    config.settings.insert("custom_setting".to_string(), ConfigValue::String { 
        value: "custom_value".to_string() 
    });
    config.settings.insert("timeout".to_string(), ConfigValue::Integer { 
        value: 5000 
    });
    config.environment.insert("ENV_VAR".to_string(), "test_value".to_string());
    
    // Save configuration
    let service_id = ServiceId(400);
    let result = config_manager.save_config(&service_id, config.clone());
    assert!(result.is_ok());
    
    // Load configuration
    let result = config_manager.load_config(&service_id);
    assert!(result.is_ok());
    
    let loaded_config = result.unwrap();
    assert_eq!(loaded_config.name, "config-integration-test");
    assert_eq!(loaded_config.version, "1.0.0");
    
    // Verify custom settings
    assert_eq!(
        loaded_config.settings.get("custom_setting").unwrap().as_string(),
        Some("custom_value")
    );
    assert_eq!(
        loaded_config.settings.get("timeout").unwrap().as_i64(),
        Some(5000)
    );
    
    // Verify environment variables
    assert_eq!(
        loaded_config.environment.get("ENV_VAR").unwrap(),
        &"test_value".to_string()
    );
    
    // Test configuration update
    let mut updates = BTreeMap::new();
    updates.insert("timeout".to_string(), ConfigValue::Integer { value: 10000 });
    updates.insert("new_setting".to_string(), ConfigValue::Boolean { value: true });
    
    let result = config_manager.update_config(&service_id, updates);
    assert!(result.is_ok());
    
    // Verify updates
    let result = config_manager.load_config(&service_id);
    assert!(result.is_ok());
    
    let updated_config = result.unwrap();
    assert_eq!(
        updated_config.settings.get("timeout").unwrap().as_i64(),
        Some(10000)
    );
    assert_eq!(
        updated_config.settings.get("new_setting").unwrap().as_bool(),
        Some(true)
    );
    
    // Test validation
    let invalid_config = ServiceConfig {
        service_id: None,
        name: "".to_string(), // Invalid empty name
        version: "1.0.0".to_string(),
        settings: BTreeMap::new(),
        environment: BTreeMap::new(),
        secrets: BTreeMap::new(),
        network: NetworkConfig {
            bind_address: "0.0.0.0".to_string(),
            bind_port: None,
            protocol: Protocol::Http,
            ssl_enabled: false,
            ssl_certificate: None,
            ssl_key: None,
            max_connections: 1000,
            connection_timeout: 30000,
            keep_alive: true,
        },
        logging: LoggingConfig {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Console,
            file_path: None,
            max_file_size: 10485760,
            max_files: 5,
            rotate_on_size: true,
            timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
        },
        monitoring: MonitoringConfig {
            health_check_enabled: true,
            health_check_interval: 30000,
            health_check_timeout: 5000,
            metrics_enabled: true,
            metrics_endpoint: Some("/metrics".to_string()),
            alert_thresholds: BTreeMap::new(),
        },
        security: SecurityConfig {
            user: Some("service".to_string()),
            group: Some("service".to_string()),
            capabilities: Vec::new(),
            namespaces: Vec::new(),
            selinux_context: None,
            apparmor_profile: None,
            secure_bits: 0,
        },
        resources: ResourceConfig {
            cpu_limit: Some(1.0),
            memory_limit: Some(134217728),
            disk_limit: None,
            network_limit: None,
            file_descriptor_limit: Some(1024),
            thread_limit: Some(64),
            nice_level: Some(0),
            oom_score_adjust: Some(0),
        },
    };
    
    let result = config_manager.validate_config(&invalid_config);
    assert!(result.is_err()); // Should fail validation
}