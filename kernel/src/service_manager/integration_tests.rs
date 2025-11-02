//! Integration tests for the Service Management Framework
//!
//! These tests verify that the different components work together correctly
//! including service lifecycle management, dependency resolution, and monitoring.

use alloc::vec::Vec;
use alloc::string::String;
use spin::RwLock;

use super::*;
use super::service::{ServiceDescriptor, ServiceType, ServiceState, HealthStatus};
use super::config::{ServiceConfigManager, ServiceConfig};
use super::discovery::{ServiceRegistry, ServiceDiscovery};
use super::monitoring::{ServiceMonitor, HealthCheckType};

/// Integration test: Complete service lifecycle
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_service_lifecycle() {
        // Create service manager
        let manager = ServiceManager::new();
        
        // Create service descriptors
        let database_descriptor = ServiceDescriptor {
            name: "database".to_string(),
            display_name: "Database Service".to_string(),
            description: Some("Database management service".to_string()),
            service_type: ServiceType::SystemService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 1000,
            max_restarts: 3,
            health_check_interval: 30000,
            tags: vec!["database".to_string(), "storage".to_string()],
        };

        let web_descriptor = ServiceDescriptor {
            name: "web-server".to_string(),
            display_name: "Web Server".to_string(),
            description: Some("HTTP web server".to_string()),
            service_type: ServiceType::UserService,
            dependencies: vec![super::super::ServiceDependency {
                service_id: ServiceId(1), // Will be set after database is created
                required: true,
                timeout: 30000,
            }],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 500,
            max_restarts: 5,
            health_check_interval: 10000,
            tags: vec!["web".to_string(), "http".to_string()],
        };

        // Test service registration
        let db_id = manager.register_service(database_descriptor).unwrap();
        assert_eq!(db_id.0, 1);

        // Update web service dependency with correct service ID
        let mut web_descriptor = web_descriptor;
        web_descriptor.dependencies[0].service_id = db_id;

        let web_id = manager.register_service(web_descriptor).unwrap();
        assert_eq!(web_id.0, 2);

        // Test service discovery
        let services = manager.discover_services("database").unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0], db_id);

        let web_services = manager.discover_services("*server*").unwrap();
        assert_eq!(web_services.len(), 1);
        assert_eq!(web_services[0], web_id);

        // Test service starting
        assert_eq!(manager.start_service(db_id), Ok(()));
        
        // Wait a bit for database to start
        crate::hal::sleep_ms(100).unwrap();

        // Start web service (should succeed after database)
        assert_eq!(manager.start_service(web_id), Ok(()));

        // Test service statistics
        let stats = manager.get_stats();
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.running_services, 2);

        // Test service stopping
        assert_eq!(manager.stop_service(web_id), Ok(()));
        assert_eq!(manager.stop_service(db_id), Ok(()));

        let stats = manager.get_stats();
        assert_eq!(stats.running_services, 0);
    }

    #[test]
    fn test_dependency_resolution() {
        let manager = ServiceManager::new();

        // Create service chain: A -> B -> C
        let service_c = ServiceDescriptor {
            name: "service-c".to_string(),
            display_name: "Service C".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let service_b = ServiceDescriptor {
            name: "service-b".to_string(),
            display_name: "Service B".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: vec![super::super::ServiceDependency {
                service_id: ServiceId(0), // Placeholder
                required: true,
                timeout: 10000,
            }],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let service_a = ServiceDescriptor {
            name: "service-a".to_string(),
            display_name: "Service A".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: vec![super::super::ServiceDependency {
                service_id: ServiceId(0), // Placeholder
                required: true,
                timeout: 10000,
            }],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        // Register services
        let c_id = manager.register_service(service_c).unwrap();
        let mut b_descriptor = service_b;
        b_descriptor.dependencies[0].service_id = c_id;
        let b_id = manager.register_service(b_descriptor).unwrap();
        let mut a_descriptor = service_a;
        a_descriptor.dependencies[0].service_id = b_id;
        let a_id = manager.register_service(a_descriptor).unwrap();

        // Test dependency resolution
        let startup_order = manager.resolve_dependencies(&[a_id, b_id, c_id]).unwrap();
        assert_eq!(startup_order.len(), 3);
        assert_eq!(startup_order[0], c_id);
        assert_eq!(startup_order[1], b_id);
        assert_eq!(startup_order[2], a_id);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let manager = ServiceManager::new();

        // Create circular dependency: A -> B -> A
        let service_a = ServiceDescriptor {
            name: "service-a".to_string(),
            display_name: "Service A".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: vec![super::super::ServiceDependency {
                service_id: ServiceId(0), // Placeholder
                required: true,
                timeout: 10000,
            }],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let service_b = ServiceDescriptor {
            name: "service-b".to_string(),
            display_name: "Service B".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: vec![super::super::ServiceDependency {
                service_id: ServiceId(0), // Placeholder
                required: true,
                timeout: 10000,
            }],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        // Register services
        let a_id = manager.register_service(service_a).unwrap();
        let mut b_descriptor = service_b;
        b_descriptor.dependencies[0].service_id = a_id;
        let b_id = manager.register_service(b_descriptor).unwrap();
        
        // Update service A to depend on service B
        {
            let services = manager.services.read();
            if let Some(handle) = services.get(&a_id) {
                let mut service = handle.lock();
                service.descriptor.dependencies[0].service_id = b_id;
            }
        }

        // Test circular dependency detection
        let result = manager.resolve_dependencies(&[a_id, b_id]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ServiceError::CircularDependency);
    }

    #[test]
    fn test_service_monitoring_integration() {
        let manager = ServiceManager::new();
        
        // Create a test service
        let test_descriptor = ServiceDescriptor {
            name: "monitored-service".to_string(),
            display_name: "Monitored Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 1000,
            max_restarts: 3,
            health_check_interval: 5000,
            tags: vec!["monitored".to_string()],
        };

        let service_id = manager.register_service(test_descriptor).unwrap();
        
        // Start the service
        manager.start_service(service_id).unwrap();
        
        // Perform health check
        let health_result = manager.monitor.check_health(service_id);
        assert!(health_result.is_ok());
        
        let report = manager.monitor.get_health_report(service_id).unwrap();
        assert_eq!(report.service_id, service_id);
        
        // Test monitoring statistics
        let stats = manager.monitor.get_stats();
        assert!(stats.total_health_checks > 0);
        
        // Stop the service
        manager.stop_service(service_id).unwrap();
    }

    #[test]
    fn test_load_balancer_integration() {
        let manager = ServiceManager::new();
        
        // Create multiple instances of the same service
        let base_descriptor = ServiceDescriptor {
            name: "load-balanced-service".to_string(),
            display_name: "Load Balanced Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let mut instances = Vec::new();
        for i in 0..3 {
            let mut descriptor = base_descriptor.clone();
            descriptor.name = format!("{}-{}", descriptor.name, i);
            let id = manager.register_service(descriptor).unwrap();
            instances.push(id);
        }

        // Start all instances
        for &id in &instances {
            manager.start_service(id).unwrap();
        }

        // Test load balancing
        for _ in 0..10 {
            let selected = manager.get_service_instance("load-balanced-service");
            assert!(selected.is_ok());
        }

        // Test load balancing statistics
        let lb_stats = manager.load_balancer.get_stats();
        assert!(lb_stats.total_requests > 0);
    }

    #[test]
    fn test_service_configuration_management() {
        let manager = ServiceManager::new();
        
        // Create service
        let descriptor = ServiceDescriptor {
            name: "configurable-service".to_string(),
            display_name: "Configurable Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: Vec::new(),
        };

        let service_id = manager.register_service(descriptor).unwrap();
        
        // Create configuration
        let config = ServiceConfig {
            service_id: Some(service_id),
            name: "configurable-service".to_string(),
            version: "1.0.0".to_string(),
            settings: alloc::collections::BTreeMap::new(),
            environment: alloc::collections::BTreeMap::new(),
            secrets: alloc::collections::BTreeMap::new(),
            network: super::super::config::NetworkConfig {
                bind_address: "127.0.0.1".to_string(),
                bind_port: Some(8080),
                protocol: super::super::config::Protocol::Http,
                ssl_enabled: false,
                ssl_certificate: None,
                ssl_key: None,
                max_connections: 1000,
                connection_timeout: 30000,
                keep_alive: true,
            },
            logging: super::super::config::LoggingConfig {
                level: super::super::config::LogLevel::Info,
                format: super::super::config::LogFormat::Text,
                output: super::super::config::LogOutput::Console,
                file_path: None,
                max_file_size: 10485760,
                max_files: 5,
                rotate_on_size: true,
                timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
            },
            monitoring: super::super::config::MonitoringConfig {
                health_check_enabled: true,
                health_check_interval: 10000,
                health_check_timeout: 5000,
                metrics_enabled: true,
                metrics_endpoint: Some("/metrics".to_string()),
                alert_thresholds: alloc::collections::BTreeMap::new(),
            },
            security: super::super::config::SecurityConfig {
                user: Some("service".to_string()),
                group: Some("service".to_string()),
                capabilities: Vec::new(),
                namespaces: Vec::new(),
                selinux_context: None,
                apparmor_profile: None,
                secure_bits: 0,
            },
            resources: super::super::config::ResourceConfig {
                cpu_limit: Some(1.0),
                memory_limit: Some(134217728), // 128MB
                disk_limit: None,
                network_limit: None,
                file_descriptor_limit: Some(1024),
                thread_limit: Some(64),
                nice_level: Some(0),
                oom_score_adjust: Some(0),
            },
        };

        // Save configuration
        manager.config_manager.save_config(&service_id, config.clone()).unwrap();
        
        // Load configuration back
        let loaded_config = manager.config_manager.load_config(&service_id).unwrap();
        assert_eq!(loaded_config.name, "configurable-service");
        assert_eq!(loaded_config.version, "1.0.0");
        assert_eq!(loaded_config.network.bind_address, "127.0.0.1");
        assert_eq!(loaded_config.network.bind_port, Some(8080));
        
        // Test configuration update
        let mut updates = alloc::collections::BTreeMap::new();
        updates.insert("setting1".to_string(), super::super::config::ConfigValue::String { value: "value1".to_string() });
        manager.config_manager.update_config(&service_id, updates).unwrap();
    }

    #[test]
    fn test_service_registry_operations() {
        let registry = ServiceRegistry::new();
        
        // Create mock service handle
        let descriptor = ServiceDescriptor {
            name: "registry-test-service".to_string(),
            display_name: "Registry Test Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: false,
            restart_delay: 0,
            max_restarts: 0,
            health_check_interval: 0,
            tags: vec!["test".to_string(), "registry".to_string()],
        };
        
        let handle = super::service::ServiceHandle::new(ServiceId(1), descriptor);
        
        // Test service registration
        registry.register_service(handle.clone()).unwrap();
        
        // Test service lookup
        let endpoint = registry.lookup_service(ServiceId(1)).unwrap();
        assert_eq!(endpoint.name, "registry-test-service");
        
        // Test discovery by name
        let services = registry.lookup_by_name("registry-test-service").unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "registry-test-service");
        
        // Test service type filtering
        let services = registry.get_services_by_type(ServiceType::UserService);
        assert!(services.len() > 0);
        
        // Test unregistration
        registry.unregister_service(ServiceId(1)).unwrap();
        
        // Verify service is removed
        let result = registry.lookup_service(ServiceId(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_service_discovery_patterns() {
        let discovery = ServiceDiscovery::new();
        
        // Mock service registry entries would be set up here
        // For now, test pattern matching
        
        assert!(discovery.matches_pattern("my-service", "my-"));
        assert!(discovery.matches_pattern("my-service", "service"));
        assert!(discovery.matches_pattern("my-service", "*"));
        assert!(!discovery.matches_pattern("my-service", "other"));
        
        // Test advanced pattern matching
        assert!(discovery.matches_advanced_pattern("my-service", "regex:my-.*"));
        assert!(discovery.matches_advanced_pattern("my-service", "glob:*service*"));
    }

    #[test]
    fn test_fault_tolerance_mechanisms() {
        let fault_detector = FaultDetector::new();
        let recovery_manager = RecoveryManager::new();
        
        // Simulate a service failure
        let service_id = ServiceId(1);
        
        // Detect fault
        fault_detector.detect_fault(service_id, &ServiceError::HealthCheckFailed).unwrap();
        
        // Test recovery
        let recovery_result = recovery_manager.handle_fault(service_id, &ServiceError::HealthCheckFailed);
        assert!(recovery_result.is_ok());
        
        // Check fault detection statistics
        let stats = fault_detector.get_stats();
        assert!(stats.total_faults_detected > 0);
        
        // Check recovery statistics
        let recovery_stats = recovery_manager.get_stats();
        assert!(recovery_stats.total_recovery_attempts > 0);
    }

    #[test]
    fn test_complete_system_integration() {
        // Test the entire system working together
        
        // Initialize service manager
        ServiceManager::init().unwrap();
        ServiceManager::start().unwrap();
        
        // Create a complex service topology
        let database = ServiceDescriptor {
            name: "database".to_string(),
            display_name: "Database Service".to_string(),
            description: None,
            service_type: ServiceType::SystemService,
            dependencies: Vec::new(),
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 2000,
            max_restarts: 3,
            health_check_interval: 15000,
            tags: vec!["database".to_string(), "storage".to_string()],
        };

        let cache = ServiceDescriptor {
            name: "cache".to_string(),
            display_name: "Cache Service".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: vec![super::super::ServiceDependency {
                service_id: ServiceId(0), // Will be set
                required: true,
                timeout: 10000,
            }],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 1000,
            max_restarts: 5,
            health_check_interval: 10000,
            tags: vec!["cache".to_string(), "memory".to_string()],
        };

        let web_server = ServiceDescriptor {
            name: "web-server".to_string(),
            display_name: "Web Server".to_string(),
            description: None,
            service_type: ServiceType::UserService,
            dependencies: vec![
                super::super::ServiceDependency {
                    service_id: ServiceId(0), // Will be set
                    required: true,
                    timeout: 30000,
                },
                super::super::ServiceDependency {
                    service_id: ServiceId(0), // Will be set
                    required: true,
                    timeout: 15000,
                },
            ],
            resource_limits: None,
            isolation_level: super::super::IsolationLevel::Process,
            auto_restart: true,
            restart_delay: 500,
            max_restarts: 10,
            health_check_interval: 5000,
            tags: vec!["web".to_string(), "http".to_string(), "api".to_string()],
        };

        // Register all services
        let db_id = ServiceManager::register_service(database).unwrap();
        let mut cache_descriptor = cache;
        cache_descriptor.dependencies[0].service_id = db_id;
        let cache_id = ServiceManager::register_service(cache_descriptor).unwrap();
        
        let mut web_descriptor = web_server;
        web_descriptor.dependencies[0].service_id = db_id;
        web_descriptor.dependencies[1].service_id = cache_id;
        let web_id = ServiceManager::register_service(web_descriptor).unwrap();

        // Test dependency resolution
        let startup_order = ServiceManager::resolve_dependencies(&[db_id, cache_id, web_id]).unwrap();
        assert_eq!(startup_order[0], db_id);
        assert_eq!(startup_order[1], cache_id);
        assert_eq!(startup_order[2], web_id);

        // Start all services in order
        ServiceManager::start_services_in_order(&[db_id, cache_id, web_id]).unwrap();

        // Test the entire system
        let stats = ServiceManager::get_stats();
        assert_eq!(stats.total_services, 3);
        assert_eq!(stats.running_services, 3);

        // Perform health checks on all services
        ServiceManager::check_service_health().unwrap();

        // Test load balancing
        for _ in 0..5 {
            let selected = ServiceManager::get_service_instance("web-server");
            assert!(selected.is_ok());
        }

        // Stop services in reverse order
        ServiceManager::stop_services_in_order(&[db_id, cache_id, web_id]).unwrap();

        let final_stats = ServiceManager::get_stats();
        assert_eq!(final_stats.running_services, 0);
    }
}