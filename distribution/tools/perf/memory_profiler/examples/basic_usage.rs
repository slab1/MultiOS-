//! Memory Profiler Examples
//!
//! This module contains example usage scenarios for the Memory Profiling
//! and Optimization Tools for MultiOS.

use memory_profiler_kernel::*;
use memory_profiler_userspace::*;

/// Example 1: Basic Real-time Memory Monitoring
pub fn example_realtime_monitoring() {
    // Initialize the memory profiling system
    init();
    
    println!("Starting real-time memory monitoring...");
    
    // Start monitoring with 1-second interval
    RealtimeTracker::start_monitoring(1000);
    
    // Simulate some allocations for demonstration
    for i in 0..100 {
        let size = (i % 10 + 1) * 1024; // 1KB to 10KB
        let address = 0x1000_0000 + (i * 0x1000) as u64;
        let caller = 0x2000_0000 + (i * 0x100) as u64;
        
        // Record allocation
        AllocatorHook::hook_allocation(size, 8, 0, AllocationFlags::NORMAL, caller);
        MemoryMapper::register_allocation(address, size, caller, i);
        
        // Record some cache accesses
        if i % 3 == 0 {
            CacheProfiler::record_access(address, size, CacheAccessType::READ, 25);
        }
        
        // Take periodic snapshots
        if i % 20 == 0 {
            if let Some(snapshot) = RealtimeTracker::take_snapshot() {
                println!("Memory snapshot: {} MB allocated, {}% pressure", 
                        snapshot.total_allocated / (1024 * 1024),
                        snapshot.memory_pressure * 100.0);
            }
        }
        
        // Simulate some deallocations
        if i % 7 == 0 {
            AllocatorHook::hook_deallocation(size, caller);
            MemoryMapper::unregister_allocation(address);
        }
    }
    
    // Get final statistics
    let stats = get_stats();
    println!("Final memory statistics:");
    println!("  Total allocations: {}", stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst));
    println!("  Current allocated: {} MB", stats.current_allocated.load(std::sync::atomic::Ordering::SeqCst) / (1024 * 1024));
    
    // Generate comprehensive report
    let report = MemoryMapper::generate_comprehensive_report();
    println!("System health score: {:.1}/100.0", report.system_health_score);
    
    // Stop monitoring
    RealtimeTracker::stop_monitoring();
    println!("Real-time monitoring completed.");
}

/// Example 2: Memory Leak Detection
pub fn example_leak_detection() {
    init();
    
    println!("Starting memory leak detection test...");
    
    // Simulate a memory leak scenario
    let mut leaked_addresses = Vec::new();
    
    for i in 0..50 {
        let size = 1024 * (i % 100 + 1); // Variable size allocations
        let address = 0x3000_0000 + (i * 0x2000) as u64;
        let caller = 0x4000_0000;
        
        // Mark as never freed (simulating a potential leak)
        let mut flags = AllocationFlags::new();
        flags.set_never_freed();
        
        let allocation_id = LeakDetector::record_allocation(address, size, caller, 0, flags);
        
        if i % 10 == 0 {
            // Simulate some deallocations (but not all)
            if let Some(addr) = leaked_addresses.pop() {
                LeakDetector::record_deallocation(addr);
                println!("Deallocated memory at 0x{:x}", addr);
            }
        } else {
            leaked_addresses.push(address);
        }
        
        // Simulate stack frames for leak context
        if i % 5 == 0 {
            StackProfiler::record_function_entry(i as u32, caller + (i * 0x100) as u64, size, 32);
        }
        
        // Wait a bit to let allocations "age"
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // Perform leak detection scan
    let leak_report = LeakDetector::scan_for_leaks();
    
    println!("Leak detection results:");
    println!("  Detected leaks: {}", leak_report.detected_leaks.len());
    println!("  Memory waste estimate: {} KB", leak_report.memory_waste_estimate / 1024);
    
    if !leak_report.recommendations.is_empty() {
        println!("  Top recommendations:");
        for (i, rec) in leak_report.recommendations.iter().take(3).enumerate() {
            println!("    {}. {} (Priority: {:?})", i + 1, rec.description, rec.priority);
        }
    }
    
    // Get leak statistics
    let leak_stats = LeakDetector::get_statistics();
    println!("  Detection rate: {:.2}%", leak_stats.detection_rate * 100.0);
    println!("  False positive rate: {:.2}%", leak_stats.false_positives as f32 / leak_stats.detected_leaks as f32 * 100.0);
    
    println!("Memory leak detection completed.");
}

/// Example 3: Cache Performance Analysis
pub fn example_cache_analysis() {
    init();
    
    println!("Starting cache performance analysis...");
    
    // Simulate different memory access patterns
    let patterns = [
        ("Sequential Access", CacheAccessType::READ),
        ("Random Access", CacheAccessType::WRITE),
        ("Mixed Access", CacheAccessType::READ | CacheAccessType::WRITE),
    ];
    
    for (pattern_name, access_type) in &patterns {
        println!("Testing {} pattern...", pattern_name);
        
        for i in 0..1000 {
            let address = match pattern_name {
                "Sequential Access" => 0x5000_0000 + (i * 64) as u64, // Cache-line aligned
                "Random Access" => 0x6000_0000 + ((i * 4096) % 0x100000) as u64, // Page-sized random
                "Mixed Access" => 0x7000_0000 + ((i * 256) % 0x10000) as u64, // Mixed pattern
                _ => 0,
            };
            
            let size = 64; // Cache line size
            let latency = match access_type {
                CacheAccessType::READ => 20 + (i % 10) as u32,
                CacheAccessType::WRITE => 30 + (i % 15) as u32,
                _ => 25 + (i % 12) as u32,
            };
            
            CacheProfiler::record_access(address, size, *access_type, latency);
            
            // Also track memory access for NUMA analysis
            NUMAProfiler::record_access(address, size, AccessType::Read, (i % 4) as u8);
        }
        
        // Generate performance analysis for this pattern
        let recommendations = CacheProfiler::analyze_performance();
        println!("  Generated {} recommendations for this pattern", recommendations.len());
    }
    
    // Get overall cache statistics
    let cache_stats = CacheProfiler::get_statistics();
    println!("Cache performance summary:");
    println!("  L1 hit ratio: {:.2}%", cache_stats.l1_stats.hit_ratio * 100.0);
    println!("  L2 hit ratio: {:.2}%", cache_stats.l2_stats.hit_ratio * 100.0);
    println!("  L3 hit ratio: {:.2}%", cache_stats.l3_stats.hit_ratio * 100.0);
    println!("  Average L1 latency: {:.1} cycles", cache_stats.l1_stats.average_latency);
    println!("  Overall hit ratio: {:.2}%", cache_stats.overall_hit_ratio * 100.0);
    
    // Generate visualization data
    let viz_data = CacheProfiler::generate_visualization_data();
    println!("  Visualization data generated with {} access points", viz_data.recent_accesses);
    
    println!("Cache performance analysis completed.");
}

/// Example 4: Heap Fragmentation Analysis
pub fn example_fragmentation_analysis() {
    init();
    
    println!("Starting heap fragmentation analysis...");
    
    // Simulate various allocation patterns that cause fragmentation
    let mut allocated_blocks = Vec::new();
    
    // Pattern 1: Small allocations scattered throughout heap
    println!("Simulating small scattered allocations...");
    for i in 0..100 {
        let size = 64 + (i % 16) * 16; // 64-256 bytes
        let address = 0x8000_0000 + (i * 1024) as u64;
        
        FragmentationAnalyzer::record_allocation(address, size, 0x9000_0000);
        allocated_blocks.push((address, size));
        
        if i % 15 == 0 {
            // Deallocate some blocks to create holes
            if let Some((addr, _)) = allocated_blocks.pop() {
                FragmentationAnalyzer::record_deallocation(addr);
            }
        }
    }
    
    // Pattern 2: Large allocations mixed with small ones
    println!("Simulating mixed allocation sizes...");
    for i in 0..20 {
        let size = if i % 3 == 0 {
            1024 * 1024 // 1MB allocations
        } else {
            256 + (i % 4) * 128 // Small allocations
        };
        let address = 0xA000_0000 + (i * 0x10000) as u64;
        
        FragmentationAnalyzer::record_allocation(address, size, 0xB000_0000);
        allocated_blocks.push((address, size));
        
        if i % 4 == 0 {
            if let Some((addr, _)) = allocated_blocks.pop() {
                FragmentationAnalyzer::record_deallocation(addr);
            }
        }
    }
    
    // Perform fragmentation analysis
    let frag_analysis = FragmentationAnalyzer::analyze_fragmentation();
    
    println!("Fragmentation analysis results:");
    println!("  External fragmentation: {:.2}%", frag_analysis.fragmentation_stats.external_fragmentation * 100.0);
    println!("  Internal fragmentation: {:.2}%", frag_analysis.fragmentation_stats.internal_fragmentation * 100.0);
    println!("  Effective fragmentation: {:.2}%", frag_analysis.fragmentation_stats.effective_fragmentation * 100.0);
    println!("  Largest free block: {} KB", frag_analysis.fragmentation_stats.largest_free_block / 1024);
    println!("  Total free blocks: {}", frag_analysis.fragmentation_stats.total_free_blocks);
    
    if !frag_analysis.recommended_actions.is_empty() {
        println!("  Recommended actions:");
        for action in &frag_analysis.recommended_actions {
            println!("    - {} (Risk: {:?})", action.description, action.risk_assessment);
        }
    }
    
    // Test defragmentation
    let defrag_result = FragmentationAnalyzer::defragment_heap();
    println!("Defragmentation simulation:");
    println!("  Improvement: {:.2}%", defrag_result.improvement * 100.0);
    println!("  Memory saved: {} KB", defrag_result.memory_saved / 1024);
    println!("  Duration: {} ms", defrag_result.duration_ms);
    
    // Get fragmentation statistics
    let frag_stats = FragmentationAnalyzer::get_statistics();
    println!("Fragmentation statistics:");
    println!("  Total analyses: {}", frag_stats.total_analyses);
    println!("  Successful defragmentations: {}", frag_stats.successful_defragmentations);
    println!("  Average fragmentation: {:.2}%", frag_stats.average_fragmentation * 100.0);
    
    println!("Heap fragmentation analysis completed.");
}

/// Example 5: NUMA-Aware Memory Allocation
pub fn example_numa_optimization() {
    init();
    
    println!("Starting NUMA-aware memory allocation test...");
    
    // Get NUMA topology information
    let numa_stats = NUMAProfiler::get_statistics();
    println!("NUMA topology: {} nodes detected", numa_stats.node_count);
    
    // Test different allocation policies
    let policies = [
        (PolicyType::LOCAL_FIRST, "Local First"),
        (PolicyType::INTERLEAVE, "Interleaved"),
        (PolicyType::BANDWIDTH_AWARE, "Bandwidth Aware"),
        (PolicyType::THERMAL_AWARE, "Thermal Aware"),
    ];
    
    for (policy, policy_name) in &policies {
        println!("\nTesting {} allocation policy...", policy_name);
        
        NUMAProfiler::set_allocation_policy(*policy);
        
        // Perform allocations with different preferences
        for i in 0..10 {
            let size = 4096 * (i % 8 + 1); // 4KB to 32KB
            let preferred_node = Some((i % numa_stats.node_count as usize) as u8);
            
            let result = NUMAProfiler::numa_allocate(size, NUMAFlags::LOCAL_FIRST, preferred_node);
            
            if result.success {
                println!("  Allocation {}: 0x{:x} on node {} (latency: {} ns)", 
                        i + 1, result.address, result.node_id, result.latency_estimate);
                
                // Record access patterns
                for j in 0..100 {
                    let access_addr = result.address + (j * 64) as u64;
                    let access_type = if j % 3 == 0 { AccessType::Read } else { AccessType::Write };
                    NUMAProfiler::record_access(access_addr, 64, access_type, result.node_id);
                }
            } else {
                println!("  Allocation {} failed: {}", i + 1, result.reason);
            }
        }
    }
    
    // Generate NUMA optimization report
    let numa_report = NUMAProfiler::generate_optimization_report();
    
    println!("\nNUMA optimization analysis:");
    println!("  Optimization opportunities found: {}", numa_report.optimization_opportunities.len());
    
    if !numa_report.migration_recommendations.is_empty() {
        println!("  Top migration recommendations:");
        for (i, rec) in numa_report.migration_recommendations.iter().take(3).enumerate() {
            println!("    {}. Node {} -> Node {} (benefit: {:.2})", 
                    i + 1, rec.current_node, rec.target_node, rec.expected_benefit);
        }
    }
    
    if !numa_report.load_balancing_suggestions.is_empty() {
        println!("  Load balancing suggestions:");
        for suggestion in &numa_report.load_balancing_suggestions {
            println!("    Node {} -> Node {}: {} MB", 
                    suggestion.source_node, suggestion.target_node, 
                    suggestion.suggested_migration_amount / (1024 * 1024));
        }
    }
    
    // Test memory migration simulation
    NUMAProfiler::record_access(0xC000_0000, 4096, AccessType::ReadWrite, 0);
    // Simulate migration...
    
    println!("NUMA-aware memory allocation test completed.");
}

/// Example 6: Stack Usage Profiling
pub fn example_stack_profiling() {
    init();
    
    println!("Starting stack usage profiling...");
    
    // Simulate deep call chains
    for thread_id in 0..4 {
        println!("Profiling thread {}...", thread_id);
        
        // Simulate function call stack
        let functions = [
            (0xD000_0000, 1024), // main
            (0xD100_0000, 512),  // init_system
            (0xD200_0000, 256),  // load_modules
            (0xD300_0000, 128),  // load_memory_manager
            (0xD400_0000, 64),   // initialize_allocator
        ];
        
        for (func_addr, frame_size) in &functions {
            StackProfiler::record_function_entry(thread_id, *func_addr, *frame_size, *frame_size / 4);
            
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            StackProfiler::record_function_exit(thread_id, *func_addr);
        }
    }
    
    // Generate stack profiling report
    let stack_report = StackProfiler::generate_report();
    
    println!("Stack profiling results:");
    println!("  Thread snapshots: {}", stack_report.thread_snapshots.len());
    
    for snapshot in &stack_report.thread_snapshots {
        println!("    Thread {}: depth {}, peak {}, usage {} KB", 
                snapshot.thread_id, snapshot.current_depth, snapshot.peak_depth,
                snapshot.used_space / 1024);
    }
    
    if !stack_report.optimization_opportunities.is_empty() {
        println!("  Stack optimization opportunities:");
        for opt in &stack_report.optimization_opportunities {
            println!("    - {} (savings: {} bytes)", opt.description, opt.estimated_savings);
        }
    }
    
    if !stack_report.overflow_analysis.is_empty() {
        println!("  Stack overflow events: {}", stack_report.overflow_analysis.len());
        for overflow in &stack_report.overflow_analysis {
            println!("    Thread {}: {} bytes overflow", overflow.thread_id, overflow.overflow_amount);
        }
    }
    
    // Get stack statistics
    let stack_stats = StackProfiler::get_statistics();
    println!("Stack statistics:");
    println!("  Max depth reached: {}", stack_stats.max_depth_reached);
    println!("  Stack overflows: {}", stack_stats.stack_overflows);
    println!("  Active threads: {}", stack_stats.active_threads);
    
    println!("Stack usage profiling completed.");
}

/// Example 7: Comprehensive Analysis Report
pub fn example_comprehensive_analysis() {
    init();
    
    println!("Generating comprehensive memory analysis...");
    
    // Run all the previous examples to generate data
    example_realtime_monitoring();
    example_leak_detection();
    example_cache_analysis();
    example_fragmentation_analysis();
    example_numa_optimization();
    example_stack_profiling();
    
    // Generate comprehensive report
    let report = MemoryMapper::generate_comprehensive_report();
    
    println!("\n=== COMPREHENSIVE MEMORY ANALYSIS REPORT ===");
    println!("Generated at: {}", report.timestamp);
    
    println!("\n--- Memory Summary ---");
    println!("Total allocations: {}", report.global_stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst));
    println!("Current allocated: {} MB", 
            report.global_stats.current_allocated.load(std::sync::atomic::Ordering::SeqCst) / (1024 * 1024));
    println!("Peak allocated: {} MB", 
            report.global_stats.peak_allocated.load(std::sync::atomic::Ordering::SeqCst) / (1024 * 1024));
    
    println!("\n--- Real-time Performance ---");
    println!("Allocation rate: {} MB/s", report.realtime_data.allocation_rate / (1024 * 1024));
    println!("Memory pressure: {:.1}%", report.realtime_data.memory_pressure * 100.0);
    println!("Cache hit ratio: {:.1}%", report.realtime_data.cache_hit_ratio * 100.0);
    println!("NUMA efficiency: {:.1}%", report.realtime_data.numa_efficiency * 100.0);
    
    println!("\n--- Cache Performance ---");
    println!("L1 hit ratio: {:.1}%", report.cache_performance.l1_hit_ratio * 100.0);
    println!("L2 hit ratio: {:.1}%", report.cache_performance.l2_hit_ratio * 100.0);
    println!("L3 hit ratio: {:.1}%", report.cache_performance.l3_hit_ratio * 100.0);
    println!("Average latency: {} cycles", report.cache_performance.average_latency);
    
    println!("\n--- Leak Detection ---");
    println!("Detected leaks: {}", report.leak_analysis.detected_leaks);
    println!("Memory waste: {} MB", report.leak_analysis.memory_waste / (1024 * 1024));
    println!("Leak rate: {:.2}%", report.leak_analysis.leak_rate * 100.0);
    
    println!("\n--- Fragmentation ---");
    println!("External fragmentation: {:.1}%", report.fragmentation_analysis.external_fragmentation * 100.0);
    println!("Internal fragmentation: {:.1}%", report.fragmentation_analysis.internal_fragmentation * 100.0);
    println!("Heap health score: {:.1}/10", report.fragmentation_analysis.heap_health_score * 10.0);
    
    println!("\n--- Stack Analysis ---");
    println!("Max stack depth: {}", report.stack_analysis.max_stack_depth);
    println!("Stack overflows: {}", report.stack_analysis.stack_overflows);
    println!("Stack efficiency: {:.1}%", report.stack_analysis.stack_efficiency * 100.0);
    
    println!("\n--- NUMA Analysis ---");
    println!("Local access ratio: {:.1}%", report.numa_analysis.local_access_ratio * 100.0);
    println!("Load balance score: {:.1}/10", report.numa_analysis.load_balance_score * 10.0);
    println!("NUMA efficiency: {:.1}%", report.numa_analysis.numa_efficiency * 100.0);
    
    if !report.optimization_recommendations.is_empty() {
        println!("\n--- Optimization Recommendations ---");
        for (i, rec) in report.optimization_recommendations.iter().enumerate() {
            println!("{}. {} (Impact: {:.1}%)", i + 1, rec.description, rec.expected_impact * 100.0);
        }
    }
    
    println!("\n--- System Health Score ---");
    println!("Overall system health: {:.1}/100.0", report.system_health_score);
    
    if report.system_health_score >= 80.0 {
        println!("✓ System is performing well");
    } else if report.system_health_score >= 60.0 {
        println!("⚠ System has some performance issues");
    } else {
        println!("✗ System requires attention");
    }
    
    println!("\nComprehensive analysis completed!");
}

/// Run all examples
pub fn run_all_examples() {
    println!("Running Memory Profiler Examples...\n");
    
    example_realtime_monitoring();
    println!("\n" + &"=".repeat(60) + "\n");
    
    example_leak_detection();
    println!("\n" + &"=".repeat(60) + "\n");
    
    example_cache_analysis();
    println!("\n" + &"=".repeat(60) + "\n");
    
    example_fragmentation_analysis();
    println!("\n" + &"=".repeat(60) + "\n");
    
    example_numa_optimization();
    println!("\n" + &"=".repeat(60) + "\n");
    
    example_stack_profiling();
    println!("\n" + &"=".repeat(60) + "\n");
    
    example_comprehensive_analysis();
    
    println!("\nAll examples completed successfully!");
}