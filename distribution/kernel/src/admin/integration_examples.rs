//! MultiOS Process & Service Management Integration Examples
//! 
//! This module provides examples demonstrating how the comprehensive process and
//! service management system integrates with existing MultiOS components:
//! - Integration with scheduler for process prioritization and scheduling
//! - Integration with service manager for service lifecycle management
//! - Integration with HAL for resource monitoring and hardware interactions
//! - Cross-component communication and coordination

use crate::log::{info, warn, error};
use crate::{Result, KernelError};
use crate::admin::process_manager::*;
use crate::service_manager::*;
use crate::scheduler::*;
use crate::hal::*;

/// Integration Example: Starting a Service Process
/// 
/// This demonstrates how to create and manage a service process that integrates
/// with both the process manager and service manager.
pub fn example_start_service_process() -> Result<ServiceId> {
    info!("=== Example: Starting Service Process ===");
    
    // Get service manager instance
    let service_manager_guard = SERVICE_MANAGER.lock();
    let service_manager = service_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create service descriptor for a background service
    let service_descriptor = ServiceDescriptor {
        name: "background-service".to_string(),
        display_name: "Background Service".to_string(),
        description: Some("Example background service".to_string()),
        service_type: ServiceType::SystemService,
        dependencies: Vec::new(),
        resource_limits: Some(ProcessResourceLimits {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_stack_size: 8 * 1024 * 1024, // 8MB
            max_file_descriptors: 256,
            max_processes: 1,
            max_cpu_time: 3600, // 1 hour
            max_creation_time: 5000, // 5 seconds
            max_io_read: 1024 * 1024 * 1024, // 1GB
            max_io_write: 1024 * 1024 * 1024, // 1GB
        }),
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 5000, // 5 seconds
        max_restarts: 3,
        health_check_interval: 30000, // 30 seconds
        tags: vec!["background".to_string(), "example".to_string()],
    };
    
    // Register the service
    let service_id = service_manager.register_service(service_descriptor)?;
    info!("Registered service with ID: {}", service_id.0);
    
    // Create service process
    let process_manager_guard = ProcessManager::get().unwrap().lock();
    let process_manager = process_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    let process_id = process_manager.create_service_process(
        service_id,
        "background-service".to_string(),
        vec!["background-service".to_string(), "--daemon".to_string()],
        true, // auto_restart
        3,   // max_restarts
    )?;
    
    info!("Created service process {} for service {}", process_id, service_id.0);
    
    // Start the service
    service_manager.start_service(service_id)?;
    process_manager.start_service_process(service_id)?;
    
    info!("Service process started successfully");
    Ok(service_id)
}

/// Integration Example: Process Monitoring and Resource Management
/// 
/// This demonstrates how to monitor process resources and handle limit violations.
pub fn example_process_monitoring() -> Result<ProcessId> {
    info!("=== Example: Process Monitoring ===");
    
    let process_manager_guard = ProcessManager::get().unwrap().lock();
    let process_manager = process_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create a process with custom resource limits
    let process_id = process_manager.create_process(
        None, // no parent
        ProcessPriority::High,
        ProcessPriorityClass::User,
        ProcessFlags::BACKGROUND,
        vec!["monitored-process".to_string()],
        "/tmp".to_string(),
        HashMap::new(),
    )?;
    
    info!("Created monitored process {}", process_id);
    
    // Get process information
    let process_info = process_manager.get_process_info(process_id)?;
    info!("Process info: priority {:?}, state {:?}", process_info.priority, process_info.state);
    
    // Simulate some resource usage
    // In a real system, this would be updated by the system
    process_manager.monitor_process_resources()?;
    
    // Get resource usage statistics
    let resource_usage = process_manager.get_process_stats(process_id)?;
    info!("Resource usage: memory={} bytes, CPU time={}ms", 
          resource_usage.memory_usage_bytes, resource_usage.total_time_ms);
    
    Ok(process_id)
}

/// Integration Example: Cross-Component Process Management
/// 
/// This demonstrates how the process manager, scheduler, and service manager
//! work together to manage system processes.
pub fn example_cross_component_management() -> Result<()> {
    info!("=== Example: Cross-Component Management ===");
    
    // Get all component instances
    let service_manager_guard = SERVICE_MANAGER.lock();
    let service_manager = service_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    let process_manager_guard = ProcessManager::get().unwrap().lock();
    let process_manager = process_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create a high-priority system service
    let system_service_descriptor = ServiceDescriptor {
        name: "high-priority-service".to_string(),
        display_name: "High Priority Service".to_string(),
        description: Some("System service with high priority".to_string()),
        service_type: ServiceType::SystemService,
        dependencies: Vec::new(),
        resource_limits: Some(ProcessResourceLimits {
            max_memory: 128 * 1024 * 1024, // 128MB
            max_stack_size: 16 * 1024 * 1024, // 16MB
            max_file_descriptors: 512,
            max_processes: 1,
            max_cpu_time: 7200, // 2 hours
            max_creation_time: 10000, // 10 seconds
            max_io_read: 2 * 1024 * 1024 * 1024, // 2GB
            max_io_write: 2 * 1024 * 1024 * 1024, // 2GB
        }),
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 2000, // 2 seconds
        max_restarts: 5,
        health_check_interval: 10000, // 10 seconds
        tags: vec!["system".to_string(), "high-priority".to_string()],
    };
    
    let system_service_id = service_manager.register_service(system_service_descriptor)?;
    info!("Registered system service: {}", system_service_id.0);
    
    // Create the service process with high priority
    let system_process_id = process_manager.create_service_process(
        system_service_id,
        "high-priority-service".to_string(),
        vec!["high-priority-service".to_string(), "--priority=high".to_string()],
        true,
        5,
    )?;
    
    // Set high priority for the process
    process_manager.set_process_priority(system_process_id, ProcessPriority::High)?;
    info!("Set high priority for process {}", system_process_id);
    
    // Start the service
    service_manager.start_service(system_service_id)?;
    process_manager.start_service_process(system_service_id)?;
    
    // Get statistics from all components
    let service_stats = service_manager.get_stats();
    let process_stats = process_manager.get_stats();
    let scheduler_stats = get_scheduler_stats();
    
    info!("Service Manager Stats: {} services running", service_stats.running_services);
    info!("Process Manager Stats: {} processes total", process_stats.total_processes);
    info!("Scheduler Stats: {} ready threads", scheduler_stats.ready_threads);
    
    // Demonstrate signal handling
    process_manager.send_signal(system_process_id, Signal::SIGUSR1)?;
    info!("Sent SIGUSR1 to high-priority service process");
    
    Ok(())
}

/// Integration Example: Service Dependency Management
/// 
/// This demonstrates how to manage services with dependencies.
pub fn example_service_dependencies() -> Result<Vec<ServiceId>> {
    info!("=== Example: Service Dependencies ===");
    
    let service_manager_guard = SERVICE_MANAGER.lock();
    let service_manager = service_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create database service (dependency)
    let db_descriptor = ServiceDescriptor {
        name: "database-service".to_string(),
        display_name: "Database Service".to_string(),
        description: Some("Database service for applications".to_string()),
        service_type: ServiceType::SystemService,
        dependencies: Vec::new(),
        resource_limits: Some(ProcessResourceLimits {
            max_memory: 256 * 1024 * 1024, // 256MB
            max_stack_size: 8 * 1024 * 1024, // 8MB
            max_file_descriptors: 256,
            max_processes: 1,
            max_cpu_time: 86400, // 24 hours
            max_creation_time: 10000, // 10 seconds
            max_io_read: 10 * 1024 * 1024 * 1024, // 10GB
            max_io_write: 10 * 1024 * 1024 * 1024, // 10GB
        }),
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 3000, // 3 seconds
        max_restarts: 5,
        health_check_interval: 15000, // 15 seconds
        tags: vec!["database".to_string(), "dependency".to_string()],
    };
    
    let db_service_id = service_manager.register_service(db_descriptor)?;
    
    // Create application service (depends on database)
    let app_descriptor = ServiceDescriptor {
        name: "application-service".to_string(),
        display_name: "Application Service".to_string(),
        description: Some("Application service depending on database".to_string()),
        service_type: ServiceType::UserService,
        dependencies: vec![ServiceDependency {
            service_id: db_service_id,
            required: true,
            timeout: 30000, // 30 seconds
            version_constraint: None,
        }],
        resource_limits: Some(ProcessResourceLimits {
            max_memory: 128 * 1024 * 1024, // 128MB
            max_stack_size: 8 * 1024 * 1024, // 8MB
            max_file_descriptors: 128,
            max_processes: 1,
            max_cpu_time: 43200, // 12 hours
            max_creation_time: 10000, // 10 seconds
            max_io_read: 5 * 1024 * 1024 * 1024, // 5GB
            max_io_write: 5 * 1024 * 1024 * 1024, // 5GB
        }),
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 5000, // 5 seconds
        max_restarts: 3,
        health_check_interval: 20000, // 20 seconds
        tags: vec!["application".to_string(), "user".to_string()],
    };
    
    let app_service_id = service_manager.register_service(app_descriptor)?;
    
    info!("Created service dependency chain: app -> database");
    info!("Database service ID: {}", db_service_id.0);
    info!("Application service ID: {}", app_service_id.0);
    
    // Start services in dependency order
    service_manager.start_service(db_service_id)?;
    info!("Started database service");
    
    service_manager.start_service(app_service_id)?;
    info!("Started application service");
    
    Ok(vec![db_service_id, app_service_id])
}

/// Integration Example: Resource-Constrained Process Management
/// 
/// This demonstrates how to manage processes with strict resource constraints.
pub fn example_resource_constrained_processes() -> Result<ProcessId> {
    info!("=== Example: Resource-Constrained Processes ===");
    
    let process_manager_guard = ProcessManager::get().unwrap().lock();
    let process_manager = process_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create a process with very restrictive resource limits
    let process_id = process_manager.create_process(
        None,
        ProcessPriority::Low,
        ProcessPriorityClass::Background,
        ProcessFlags::BACKGROUND,
        vec!["constrained-process".to_string()],
        "/tmp".to_string(),
        HashMap::new(),
    )?;
    
    info!("Created resource-constrained process {}", process_id);
    
    // Get process info and modify resource limits
    let mut process_info = process_manager.get_process_info(process_id)?;
    
    // Apply strict resource limits
    process_info.resource_limits = ProcessResourceLimits {
        max_memory: 32 * 1024 * 1024, // 32MB
        max_stack_size: 4 * 1024 * 1024, // 4MB
        max_file_descriptors: 64,
        max_processes: 1,
        max_cpu_time: 600, // 10 minutes
        max_creation_time: 2000, // 2 seconds
        max_io_read: 100 * 1024 * 1024, // 100MB
        max_io_write: 100 * 1024 * 1024, // 100MB
    };
    
    // Start the process
    process_info.state = ProcessState::Running;
    process_info.start_time_ms = get_current_time();
    
    info!("Started resource-constrained process with limits:");
    info!("  Memory: {} bytes", process_info.resource_limits.max_memory);
    info!("  CPU Time: {} seconds", process_info.resource_limits.max_cpu_time);
    info!("  File Descriptors: {}", process_info.resource_limits.max_file_descriptors);
    
    // Monitor resource usage
    process_manager.monitor_process_resources()?;
    
    Ok(process_id)
}

/// Integration Example: Emergency Process Management
/// 
//! This demonstrates emergency response procedures for system management.
pub fn example_emergency_management() -> Result<()> {
    info!("=== Example: Emergency Process Management ===");
    
    let process_manager_guard = ProcessManager::get().unwrap().lock();
    let process_manager = process_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create several test processes
    let mut test_processes = Vec::new();
    
    for i in 0..5 {
        let process_id = process_manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec![format!("test-process-{}", i)],
            "/tmp".to_string(),
            HashMap::new(),
        )?;
        test_processes.push(process_id);
    }
    
    info!("Created {} test processes", test_processes.len());
    
    // Start all processes
    for &process_id in &test_processes {
        let mut process_info = process_manager.get_process_info(process_id)?;
        process_info.state = ProcessState::Running;
        process_info.start_time_ms = get_current_time();
    }
    
    // Simulate emergency condition - terminate all user processes
    info!("EMERGENCY: Terminating all user processes");
    
    for &process_id in &test_processes {
        process_manager.terminate_process(process_id, true)?;
        info!("Terminated process {}", process_id);
    }
    
    // Get updated statistics
    let process_stats = process_manager.get_stats();
    info!("After emergency termination:");
    info!("  Total processes: {}", process_stats.total_processes);
    info!("  Terminated processes: {}", process_stats.terminated_processes);
    
    Ok(())
}

/// Integration Example: Performance Monitoring and Optimization
//! 
//! This demonstrates how to monitor and optimize system performance
//! using the integrated management components.
pub fn example_performance_monitoring() -> Result<()> {
    info!("=== Example: Performance Monitoring ===");
    
    // Get all component instances
    let service_manager_guard = SERVICE_MANAGER.lock();
    let service_manager = service_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    let process_manager_guard = ProcessManager::get().unwrap().lock();
    let process_manager = process_manager_guard.as_ref()
        .ok_or(KernelError::NotInitialized)?;
    
    // Create performance monitoring service
    let perf_service_descriptor = ServiceDescriptor {
        name: "performance-monitor".to_string(),
        display_name: "Performance Monitor".to_string(),
        description: Some("System performance monitoring service".to_string()),
        service_type: ServiceType::SystemService,
        dependencies: Vec::new(),
        resource_limits: Some(ProcessResourceLimits {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_stack_size: 4 * 1024 * 1024, // 4MB
            max_file_descriptors: 128,
            max_processes: 1,
            max_cpu_time: 3600, // 1 hour
            max_creation_time: 5000, // 5 seconds
            max_io_read: 512 * 1024 * 1024, // 512MB
            max_io_write: 512 * 1024 * 1024, // 512MB
        }),
        isolation_level: IsolationLevel::Process,
        auto_restart: true,
        restart_delay: 2000, // 2 seconds
        max_restarts: 3,
        health_check_interval: 10000, // 10 seconds
        tags: vec!["monitoring".to_string(), "performance".to_string()],
    };
    
    let perf_service_id = service_manager.register_service(perf_service_descriptor)?;
    
    // Create and start the performance monitor
    let perf_process_id = process_manager.create_service_process(
        perf_service_id,
        "performance-monitor".to_string(),
        vec!["performance-monitor".to_string(), "--detailed".to_string()],
        true,
        3,
    )?;
    
    service_manager.start_service(perf_service_id)?;
    process_manager.start_service_process(perf_service_id)?;
    
    info!("Performance monitoring service started");
    
    // Collect comprehensive system statistics
    let service_stats = service_manager.get_stats();
    let process_stats = process_manager.get_stats();
    let scheduler_stats = get_scheduler_stats();
    let hal_stats = get_stats();
    
    info!("=== System Performance Report ===");
    info!("Services: {} total, {} running", 
          service_stats.total_services, service_stats.running_services);
    info!("Processes: {} total, {} running, {} blocked", 
          process_stats.total_processes, process_stats.running_processes, 
          process_stats.blocked_processes);
    info!("Scheduler: {} ready threads, {} context switches", 
          scheduler_stats.ready_threads, scheduler_stats.context_switches);
    info!("HAL: CPU {}, Memory {} pages", 
          hal_stats.cpu_stats.total_cores, hal_stats.memory_stats.total_pages);
    
    // Monitor and analyze resource usage
    process_manager.monitor_process_resources()?;
    
    // Check service health
    service_manager.check_service_health()?;
    
    info!("Performance monitoring completed");
    
    Ok(())
}

/// Run all integration examples
pub fn run_all_examples() -> Result<()> {
    info!("=======================================");
    info!("MultiOS Process & Service Management Integration Examples");
    info!("=======================================");
    
    // Example 1: Service Process Management
    let _service_id = example_start_service_process()?;
    sleep_ms(1000)?;
    
    // Example 2: Process Monitoring
    let _process_id = example_process_monitoring()?;
    sleep_ms(1000)?;
    
    // Example 3: Cross-Component Management
    example_cross_component_management()?;
    sleep_ms(1000)?;
    
    // Example 4: Service Dependencies
    let _service_ids = example_service_dependencies()?;
    sleep_ms(1000)?;
    
    // Example 5: Resource-Constrained Processes
    let _constrained_process_id = example_resource_constrained_processes()?;
    sleep_ms(1000)?;
    
    // Example 6: Emergency Management
    example_emergency_management()?;
    sleep_ms(1000)?;
    
    // Example 7: Performance Monitoring
    example_performance_monitoring()?;
    
    info!("=======================================");
    info!("All integration examples completed successfully");
    info!("=======================================");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_process_integration() {
        // This test validates that service and process management work together
        assert!(example_start_service_process().is_ok());
    }

    #[test]
    fn test_cross_component_integration() {
        // This test validates cross-component coordination
        assert!(example_cross_component_management().is_ok());
    }

    #[test]
    fn test_emergency_procedures() {
        // This test validates emergency management procedures
        assert!(example_emergency_management().is_ok());
    }
}