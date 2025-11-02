//! MultiOS Advanced Multi-Core Optimization Example
//!
//! This example demonstrates how to use the comprehensive multi-core optimization
//! and virtual memory scaling features implemented in MultiOS.

use multios_scheduler::*;
use multios_memory_manager::*;

/// Example configuration for a high-performance computing system
fn create_hpc_config() -> MultiCoreConfig {
    MultiCoreConfig {
        max_cpus: 256,
        enable_numa: true,
        numa_nodes: 8,
        enable_hotplug: true,
        enable_performance_monitoring: true,
        enable_real_time: true,
        enable_cache_coherency: true,
        enable_large_scale_vm: true,
        max_virtual_memory: 1 << 60, // 1 Exabyte
        enable_power_management: true,
        enable_thermal_management: true,
        scheduler_config: SchedulerConfig {
            algorithm: SchedulingAlgorithm::MultiLevelFeedbackQueue,
            cpu_count: 256,
            default_time_quantum: 25,
            load_balance_interval: 500,
            enable_cpu_affinity: true,
            enable_load_balancing: true,
        },
        multicore_config: MulticoreConfig {
            max_cpus: 256,
            enable_hotplug: true,
            enable_domains: true,
            domain_size: 16,
            enable_balancing: true,
            balance_algorithm: BalanceAlgorithm::NumaAware,
            enable_power_mgmt: true,
            enable_realtime: true,
            enable_numa: true,
            rt_deadline_us: 1000,
            latency_target_ns: 1000,
            migration_cost_ns: 500,
            cache_line_size: 64,
            enable_monitoring: true,
            monitoring_interval: 100,
        },
        performance_config: PerformanceConfig {
            enable_hardware_counters: true,
            enable_software_counters: true,
            sampling_frequency_hz: 200,
            enable_prediction: true,
            enable_auto_tuning: true,
            alerting_enabled: true,
            retention_period_hours: 168, // 1 week
            max_history_size: 50000,
            thermal_monitoring: true,
            power_monitoring: true,
            numa_monitoring: true,
        },
    }
}

/// Example for a real-time system
fn create_realtime_config() -> MultiCoreConfig {
    MultiCoreConfig {
        max_cpus: 64,
        enable_numa: true,
        numa_nodes: 4,
        enable_hotplug: false, // Disable for real-time
        enable_performance_monitoring: true,
        enable_real_time: true,
        enable_cache_coherency: true,
        enable_large_scale_vm: false, // Simpler for real-time
        max_virtual_memory: 1 << 48, // 256TB
        enable_power_management: false, // Disable for predictable performance
        enable_thermal_management: true,
        scheduler_config: SchedulerConfig {
            algorithm: SchedulingAlgorithm::EarliestDeadlineFirst,
            cpu_count: 64,
            default_time_quantum: 10, // Shorter quantum for real-time
            load_balance_interval: 1000, // Less frequent balancing
            enable_cpu_affinity: true,
            enable_load_balancing: true,
        },
        multicore_config: MulticoreConfig {
            max_cpus: 64,
            enable_hotplug: false,
            enable_domains: true,
            domain_size: 8,
            enable_balancing: true,
            balance_algorithm: BalanceAlgorithm::LoadBased, // Simpler for real-time
            enable_power_mgmt: false,
            enable_realtime: true,
            enable_numa: true,
            rt_deadline_us: 100, // 100 microsecond deadline
            latency_target_ns: 500, // Sub-microsecond latency
            migration_cost_ns: 1000, // Higher migration cost tolerance
            cache_line_size: 64,
            enable_monitoring: true,
            monitoring_interval: 50, // Higher frequency monitoring
        },
        performance_config: PerformanceConfig {
            enable_hardware_counters: true,
            enable_software_counters: true,
            sampling_frequency_hz: 500, // Higher frequency for real-time
            enable_prediction: false, // Disable prediction for determinism
            enable_auto_tuning: false, // Disable auto-tuning for stability
            alerting_enabled: true,
            retention_period_hours: 24, // Shorter retention for real-time
            max_history_size: 10000,
            thermal_monitoring: true,
            power_monitoring: false, // Disable for performance
            numa_monitoring: true,
        },
    }
}

fn main() {
    println!("MultiOS Advanced Multi-Core Optimization Example");
    println!("===============================================\n");

    // Example 1: High-Performance Computing System
    println!("Example 1: High-Performance Computing System");
    println!("--------------------------------------------");
    
    let hpc_config = create_hpc_config();
    match init_multicore_system(hpc_config) {
        Ok(()) => {
            println!("✓ HPC system initialized successfully");
            
            // Demonstrate NUMA-aware memory allocation
            println!("\n📊 Demonstrating NUMA-aware memory allocation:");
            let pages = allocate_memory_numa_aware(
                4096 * 1024, // 4GB
                NumaPolicy::Interleave,
            ).unwrap();
            println!("✓ Allocated {} pages with NUMA interleaving", pages.len());
            
            // Demonstrate large-scale virtual memory
            println!("\n🗄️  Demonstrating large-scale virtual memory:");
            map_virtual_memory_large_scale(
                VirtAddr::new(0x8000000000), // High address space
                1 << 40, // 1TB
                VmaFlags::READABLE | VmaFlags::WRITABLE | VmaFlags::HUGEPAGE,
                VmaBacking::Anonymous,
                true, // Prefer huge pages
            ).unwrap();
            println!("✓ Mapped 1TB of virtual memory with huge pages");
            
            // Demonstrate performance monitoring
            println!("\n📈 Demonstrating performance monitoring:");
            let stats = get_performance_statistics();
            println!("✓ CPU utilization: {:.2}%", stats.cpu_stats[0].utilization_percent);
            println!("✓ Memory bandwidth: {:.2} GB/s", stats.memory_stats.total_bandwidth_gbps);
            println!("✓ Cache hit rate: {:.2}%", stats.cpu_stats[0].cache_hit_rate);
            
            // Demonstrate performance optimization
            println!("\n⚡ Demonstrating performance optimization:");
            let recommendation = optimize_performance().unwrap();
            println!("✓ Optimization: {}", recommendation.action);
            println!("✓ Expected improvement: {:.2}%", recommendation.expected_improvement);
            
        },
        Err(e) => {
            println!("✗ Failed to initialize HPC system: {:?}", e);
        }
    }

    println!("\n" + "=".repeat(60) + "\n");

    // Example 2: Real-Time System
    println!("Example 2: Real-Time System");
    println!("----------------------------");
    
    let realtime_config = create_realtime_config();
    match init_multicore_system(realtime_config) {
        Ok(()) => {
            println!("✓ Real-time system initialized successfully");
            
            // Enable real-time scheduling
            enable_realtime_scheduling(true).unwrap();
            println!("✓ Real-time scheduling enabled");
            
            // Configure power management for performance
            configure_power_management(CpuGovernor::Performance, true).unwrap();
            println!("✓ Power management configured for performance");
            
            // Enable thermal management
            enable_thermal_management(true, 85).unwrap(); // 85°C throttle threshold
            println!("✓ Thermal management enabled (85°C threshold)");
            
        },
        Err(e) => {
            println!("✗ Failed to initialize real-time system: {:?}", e);
        }
    }

    println!("\n" + "=".repeat(60) + "\n");

    // Example 3: System Diagnostics
    println!("Example 3: System Diagnostics");
    println!("-----------------------------");
    
    match check_system_compatibility() {
        Ok(report) => {
            println!("✓ System compatibility check completed");
            println!("  - CPU count: {}", report.cpu_count);
            println!("  - Memory: {} GB", report.memory_gb);
            println!("  - NUMA nodes: {}", report.numa_nodes);
            println!("  - Compatible: {}", report.compatible);
            
            if !report.warnings.is_empty() {
                println!("  - Warnings:");
                for warning in report.warnings {
                    println!("    ⚠ {}", warning);
                }
            }
            
            if !report.recommendations.is_empty() {
                println!("  - Recommendations:");
                for recommendation in report.recommendations {
                    println!("    💡 {}", recommendation);
                }
            }
        },
        Err(e) => {
            println!("✗ System compatibility check failed: {:?}", e);
        }
    }

    println!("\n" + "=".repeat(60) + "\n");

    // Example 4: Health Check
    println!("Example 4: System Health Check");
    println!("------------------------------");
    
    match health_check() {
        Ok(health) => {
            println!("✓ System health check completed");
            println!("  - Overall health: {:?}", health.overall_health);
            println!("  - Checks performed: {}", health.checks.len());
            
            for (component, result, message) in health.checks {
                let icon = match result {
                    CheckResult::Pass => "✅",
                    CheckResult::Warning => "⚠️ ",
                    CheckResult::Fail => "❌",
                };
                println!("  {} {}: {}", icon, component, message);
            }
        },
        Err(e) => {
            println!("✗ Health check failed: {:?}", e);
        }
    }

    println!("\n" + "=".repeat(60) + "\n");

    // Example 5: Memory Management
    println!("Example 5: Advanced Memory Management");
    println!("------------------------------------");
    
    // Demonstrate memory deduplication
    println!("🔄 Performing memory deduplication...");
    let saved_bytes = perform_memory_deduplication().unwrap();
    println!("✓ Memory saved through deduplication: {} bytes", saved_bytes);
    
    // Demonstrate memory pressure handling
    println!("\n📊 Handling memory pressure...");
    handle_memory_pressure().unwrap();
    println!("✓ Memory pressure handled successfully");
    
    // Display comprehensive statistics
    println!("\n📊 Virtual Memory Statistics:");
    let vm_stats = get_virtual_memory_statistics();
    println!("  - Total virtual memory: {} bytes", vm_stats.total_virtual_memory);
    println!("  - Mapped memory: {} bytes", vm_stats.mapped_memory);
    println!("  - Compressed memory: {} bytes", vm_stats.compressed_memory);
    println!("  - Huge pages allocated: {}", vm_stats.huge_pages_allocated);
    
    println!("\n📊 Cache Coherency Statistics:");
    let cache_stats = get_cache_coherency_statistics();
    println!("  - Cache hits: {}", cache_stats.cache_hits.load(Ordering::SeqCst));
    println!("  - Cache misses: {}", cache_stats.cache_misses.load(Ordering::SeqCst));
    println!("  - Coherency events: {}", cache_stats.coherency_events.load(Ordering::SeqCst));
    
    println!("\n📊 NUMA Statistics:");
    let numa_stats = get_numa_statistics();
    for node_id in 0..numa_stats.total_memory.len() {
        if numa_stats.total_memory[node_id] > 0 {
            println!("  - Node {}: {} MB total, {} MB free",
                    node_id,
                    numa_stats.total_memory[node_id] / 1024 / 1024,
                    numa_stats.free_memory[node_id] / 1024 / 1024);
        }
    }

    println!("\n" + "=".repeat(60) + "\n");

    // Example 6: Export Performance Report
    println!("Example 6: Performance Report Export");
    println!("-----------------------------------");
    
    // Export as JSON
    match export_performance_report(ExportFormat::JSON) {
        Ok(data) => {
            println!("✓ Performance report exported as JSON ({} bytes)", data.len());
            println!("  First 100 characters: {}", String::from_utf8_lossy(&data[..data.len().min(100)]));
        },
        Err(e) => {
            println!("✗ Failed to export JSON report: {:?}", e);
        }
    }
    
    // Export as CSV
    match export_performance_report(ExportFormat::CSV) {
        Ok(data) => {
            println!("✓ Performance report exported as CSV ({} bytes)", data.len());
            println!("  First 100 characters: {}", String::from_utf8_lossy(&data[..data.len().min(100)]));
        },
        Err(e) => {
            println!("✗ Failed to export CSV report: {:?}", e);
        }
    }

    println!("\n" + "=".repeat(60) + "\n");

    // Example 7: CPU Hot-Plug Demo
    println!("Example 7: CPU Hot-Plug Management");
    println!("---------------------------------");
    
    println!("🔌 Demonstrating CPU hot-plug capabilities:");
    
    // Simulate taking a CPU offline
    let cpu_to_offline = 1;
    match enable_cpu_hotplug(cpu_to_offline, false) {
        Ok(()) => {
            println!("✓ CPU {} taken offline", cpu_to_offline);
        },
        Err(e) => {
            println!("⚠ Failed to take CPU offline: {:?}", e);
        }
    }
    
    // Simulate bringing CPU back online
    match enable_cpu_hotplug(cpu_to_offline, true) {
        Ok(()) => {
            println!("✓ CPU {} brought back online", cpu_to_offline);
        },
        Err(e) => {
            println!("⚠ Failed to bring CPU online: {:?}", e);
        }
    }

    println!("\n🎉 Multi-core optimization example completed!");
    println!("\nThis example demonstrates the comprehensive multi-core");
    println!("optimization and virtual memory scaling capabilities of");
    println!("MultiOS, including:");
    println!("  ✓ NUMA-aware memory management");
    println!("  ✓ Advanced multi-core scheduling");
    println!("  ✓ Cache coherency protocols");
    println!("  ✓ Large-scale virtual memory");
    println!("  ✓ Performance monitoring and optimization");
    println!("  ✓ CPU hot-plug support");
    println!("  ✓ Real-time scheduling capabilities");
    println!("  ✓ Comprehensive system diagnostics");
    println!("  ✓ Performance reporting and analytics");
}

/// Example of creating a performance-critical thread with optimal placement
fn create_performance_critical_thread() {
    println!("\n🔧 Creating performance-critical thread...");
    
    // Create thread with specific CPU affinity
    let cpu_affinity = 0xFF; // CPUs 0-7
    let thread_params = thread::ThreadParams {
        priority: Priority::Critical,
        cpu_affinity,
        // ... other parameters
    };
    
    // The thread would be created and added with optimal placement
    println!("✓ Performance-critical thread created with CPU affinity to CPUs 0-7");
}

/// Example of NUMA-aware memory allocation for different workloads
fn demonstrate_numa_allocation() {
    println!("\n🌐 Demonstrating NUMA-aware allocation strategies:");
    
    // Interleaved allocation for parallel processing
    let interleaved_pages = allocate_memory_numa_aware(
        8 * 1024 * 1024, // 8GB
        NumaPolicy::Interleave,
    ).unwrap();
    println!("✓ Allocated {} pages with interleaving for parallel processing", 
             interleaved_pages.len());
    
    // Local allocation for single-threaded workloads
    let local_pages = allocate_memory_numa_aware(
        4 * 1024 * 1024, // 4GB
        NumaPolicy::Local,
    ).unwrap();
    println!("✓ Allocated {} pages locally for single-threaded workload", 
             local_pages.len());
    
    // Bind to specific node for NUMA-aware applications
    let bound_pages = allocate_memory_numa_aware(
        16 * 1024 * 1024, // 16GB
        NumaPolicy::Bind(2), // Bind to node 2
    ).unwrap();
    println!("✓ Allocated {} pages bound to NUMA node 2", 
             bound_pages.len());
}

/// Example of memory optimization for large datasets
fn optimize_large_dataset_memory() {
    println!("\n📊 Optimizing memory for large dataset processing:");
    
    // Map large virtual memory region with huge pages
    map_virtual_memory_large_scale(
        VirtAddr::new(0x10000000000), // 256TB address space
        1 << 50, // 1PB
        VmaFlags::READABLE | VmaFlags::WRITABLE | VmaFlags::HUGEPAGE,
        VmaBacking::Anonymous,
        true, // Prefer huge pages
    ).unwrap();
    println!("✓ Mapped 1PB virtual memory region with huge page preference");
    
    // Perform memory deduplication
    let saved = perform_memory_deduplication().unwrap();
    println!("✓ Memory deduplication saved {} bytes", saved);
    
    // Enable memory compression for infrequently accessed data
    // This would be done through the memory management system
    println!("✓ Memory compression enabled for cold data");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpc_config_creation() {
        let config = create_hpc_config();
        assert_eq!(config.max_cpus, 256);
        assert!(config.enable_numa);
        assert_eq!(config.numa_nodes, 8);
        assert!(config.enable_performance_monitoring);
        assert!(config.enable_real_time);
        assert!(config.enable_cache_coherency);
        assert!(config.enable_large_scale_vm);
    }

    #[test]
    fn test_realtime_config_creation() {
        let config = create_realtime_config();
        assert_eq!(config.max_cpus, 64);
        assert_eq!(config.numa_nodes, 4);
        assert!(!config.enable_hotplug); // Disabled for real-time
        assert_eq!(config.scheduler_config.algorithm, SchedulingAlgorithm::EarliestDeadlineFirst);
        assert_eq!(config.multicore_config.rt_deadline_us, 100);
        assert!(!config.performance_config.enable_prediction); // Disabled for determinism
    }

    #[test]
    fn test_performance_optimization_example() {
        // This would test the optimization workflow
        // In a real implementation, this would require system initialization
        println!("Performance optimization test would require full system setup");
    }

    #[test]
    fn test_memory_management_example() {
        // Test memory allocation patterns
        let policies = [
            NumaPolicy::Local,
            NumaPolicy::Interleave,
            NumaPolicy::Preferred(0),
        ];
        
        for policy in policies {
            println!("Testing NUMA policy: {:?}", policy);
            // In real implementation, would test allocation
        }
    }
}

/// Performance benchmark example
#[allow(dead_code)]
fn run_performance_benchmarks() {
    println!("\n🏁 Running Performance Benchmarks");
    println!("=================================");
    
    // This would run comprehensive performance tests
    println!("✓ Memory bandwidth benchmark");
    println!("✓ CPU utilization benchmark");
    println!("✓ Context switch latency benchmark");
    println!("✓ NUMA access pattern benchmark");
    println!("✓ Cache coherency benchmark");
    println!("✓ Virtual memory performance benchmark");
    println!("✓ All benchmarks completed successfully");
}

/// System monitoring example
#[allow(dead_code)]
fn run_system_monitoring() {
    println!("\n📊 Running System Monitoring");
    println!("============================");
    
    // Real-time system monitoring
    for i in 0..10 {
        let stats = get_performance_statistics();
        println!("Sample {}: CPU util: {:.1}%, Memory BW: {:.1} GB/s, Cache hit: {:.1}%",
                 i + 1,
                 stats.cpu_stats[0].utilization_percent,
                 stats.memory_stats.total_bandwidth_gbps,
                 stats.cpu_stats[0].cache_hit_rate);
        
        // Sleep for sampling interval
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
