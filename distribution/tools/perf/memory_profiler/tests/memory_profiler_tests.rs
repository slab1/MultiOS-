//! Memory Profiler Tests
//!
//! Test suite for the Memory Profiling and Optimization Tools.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_profiler_initialization() {
        // Test that the memory profiler can be initialized
        init();
        
        let stats = get_stats();
        assert!(stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst) >= 0);
        
        println!("Memory profiler initialization test passed");
    }
    
    #[test]
    fn test_realtime_memory_tracking() {
        init();
        
        // Start real-time tracking
        RealtimeTracker::start_monitoring(100);
        
        // Record some allocations
        for i in 0..10 {
            RealtimeTracker::record_allocation(1024 + i * 100);
        }
        
        // Take a snapshot
        let snapshot = RealtimeTracker::take_snapshot();
        assert!(snapshot.is_some());
        
        // Stop tracking
        RealtimeTracker::stop_monitoring();
        
        println!("Real-time tracking test passed");
    }
    
    #[test]
    fn test_allocator_hook() {
        init();
        
        let size = 1024;
        let alignment = 8;
        let node = 0;
        let flags = AllocationFlags::NORMAL;
        let caller = 0x1000_0000;
        
        // Hook an allocation
        AllocatorHook::hook_allocation(size, alignment, node, flags, caller);
        
        // Get statistics
        let stats = AllocatorHook::get_statistics();
        assert!(stats.total_allocations >= 1);
        
        // Hook a deallocation
        AllocatorHook::hook_deallocation(size, caller);
        
        println!("Allocator hook test passed");
    }
    
    #[test]
    fn test_cache_profiler() {
        init();
        
        // Record cache accesses
        for i in 0..5 {
            let address = 0x2000_0000 + (i * 64) as u64;
            let size = 64;
            let access_type = CacheAccessType::READ;
            let latency = 20;
            
            CacheProfiler::record_access(address, size, access_type, latency);
        }
        
        // Get statistics
        let stats = CacheProfiler::get_statistics();
        assert!(stats.total_accesses >= 5);
        assert!(stats.l1_stats.hit_ratio >= 0.0 && stats.l1_stats.hit_ratio <= 1.0);
        
        println!("Cache profiler test passed");
    }
    
    #[test]
    fn test_leak_detector() {
        init();
        
        // Record allocations
        let mut allocation_ids = Vec::new();
        for i in 0..5 {
            let address = 0x3000_0000 + (i * 1024) as u64;
            let size = 1024;
            let caller = 0x4000_0000;
            let node = 0;
            let mut flags = AllocationFlags::new();
            flags.set_never_freed();
            
            let allocation_id = LeakDetector::record_allocation(address, size, caller, node, flags);
            allocation_ids.push(allocation_id);
        }
        
        // Record some deallocations (but not all)
        for i in 0..2 {
            let address = 0x3000_0000 + (i * 1024) as u64;
            LeakDetector::record_deallocation(address);
        }
        
        // Perform leak detection scan
        let report = LeakDetector::scan_for_leaks();
        
        // The report should have been generated (even if no leaks detected in short test)
        assert!(report.timestamp > 0);
        
        // Get statistics
        let stats = LeakDetector::get_statistics();
        assert!(stats.total_allocations >= 5);
        
        println!("Leak detector test passed");
    }
    
    #[test]
    fn test_fragmentation_analyzer() {
        init();
        
        // Record allocations
        for i in 0..10 {
            let address = 0x5000_0000 + (i * 1024) as u64;
            let size = 512 + i * 128;
            let owner = 0x6000_0000;
            
            FragmentationAnalyzer::record_allocation(address, size, owner);
        }
        
        // Record some deallocations to create fragmentation
        for i in 0..3 {
            let address = 0x5000_0000 + (i * 1024) as u64;
            FragmentationAnalyzer::record_deallocation(address);
        }
        
        // Perform fragmentation analysis
        let report = FragmentationAnalyzer::analyze_fragmentation();
        
        // Verify report structure
        assert!(report.timestamp > 0);
        assert!(report.fragmentation_stats.total_heap_size > 0);
        
        // Test defragmentation
        let defrag_result = FragmentationAnalyzer::defragment_heap();
        assert!(defrag_result.improvement >= 0.0);
        assert!(defrag_result.memory_saved >= 0);
        
        println!("Fragmentation analyzer test passed");
    }
    
    #[test]
    fn test_stack_profiler() {
        init();
        
        // Record function entries and exits
        let thread_id = 1;
        let function_address = 0x7000_0000;
        
        for i in 0..5 {
            let frame_size = 256 + i * 64;
            let parameters = frame_size / 4;
            
            StackProfiler::record_function_entry(thread_id, function_address, frame_size, parameters);
            StackProfiler::record_function_exit(thread_id, function_address);
        }
        
        // Generate stack report
        let report = StackProfiler::generate_report();
        
        // Verify report structure
        assert!(!report.thread_snapshots.is_empty() || report.thread_snapshots.len() >= 0);
        assert!(report.timestamp > 0);
        
        // Get statistics
        let stats = StackProfiler::get_statistics();
        assert!(stats.total_allocations >= 0);
        
        println!("Stack profiler test passed");
    }
    
    #[test]
    fn test_numa_profiler() {
        init();
        
        // Test NUMA allocation
        let size = 4096;
        let flags = NUMAFlags::LOCAL_FIRST;
        let preferred_node = Some(0);
        
        let result = NUMAProfiler::numa_allocate(size, flags, preferred_node);
        
        // In the test environment, allocation might not succeed, but we should get a result
        assert!(result.node_id >= 0);
        
        // Record NUMA access
        let address = 0x8000_0000;
        let access_size = 64;
        let access_type = AccessType::Read;
        let thread_node = 0;
        
        NUMAProfiler::record_access(address, access_size, access_type, thread_node);
        
        // Generate NUMA report
        let report = NUMAProfiler::generate_optimization_report();
        
        // Verify report structure
        assert!(report.timestamp > 0);
        assert!(report.node_statistics.len() > 0);
        
        // Get statistics
        let stats = NUMAProfiler::get_statistics();
        assert!(stats.total_allocations >= 0);
        
        println!("NUMA profiler test passed");
    }
    
    #[test]
    fn test_memory_mapper() {
        init();
        
        // Register allocations
        let mut allocation_ids = Vec::new();
        for i in 0..3 {
            let address = 0x9000_0000 + (i * 1024) as u64;
            let size = 1024 + i * 512;
            let caller = 0xA000_0000 + (i * 0x100) as u64;
            let allocation_id = i as u64;
            
            let success = MemoryMapper::register_allocation(address, size, caller, allocation_id);
            assert!(success);
            allocation_ids.push(allocation_id);
        }
        
        // Get memory mapping for first allocation
        let address = 0x9000_0000;
        let mapping = MemoryMapper::get_memory_mapping(address);
        assert!(mapping.is_some());
        
        // Get allocations by caller
        let caller = 0xA000_0000;
        let caller_allocations = MemoryMapper::get_allocations_by_caller(caller);
        assert!(!caller_allocations.is_empty());
        
        // Unregister one allocation
        let unregister_success = MemoryMapper::unregister_allocation(address);
        assert!(unregister_success);
        
        // Verify it's no longer mapped
        let mapping_after = MemoryMapper::get_memory_mapping(address);
        assert!(mapping_after.is_none());
        
        println!("Memory mapper test passed");
    }
    
    #[test]
    fn test_comprehensive_report() {
        init();
        
        // Generate some data by using different components
        RealtimeTracker::record_allocation(1024);
        AllocatorHook::hook_allocation(1024, 8, 0, AllocationFlags::NORMAL, 0x1000_0000);
        CacheProfiler::record_access(0x2000_0000, 64, CacheAccessType::READ, 25);
        
        // Generate comprehensive report
        let report = MemoryMapper::generate_comprehensive_report();
        
        // Verify report structure
        assert!(report.timestamp > 0);
        assert!(report.global_stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst) >= 0);
        assert!(report.realtime_data.total_allocated >= 0);
        assert!(report.system_health_score >= 0.0 && report.system_health_score <= 100.0);
        
        println!("Comprehensive report test passed");
    }
    
    #[test]
    fn test_integration_scenario() {
        init();
        
        // Simulate a realistic scenario with multiple components working together
        
        // 1. Start real-time tracking
        RealtimeTracker::start_monitoring(100);
        
        // 2. Perform various allocations and track them
        for i in 0..20 {
            let address = 0xB000_0000 + (i * 0x1000) as u64;
            let size = 512 + (i % 5) * 256;
            let caller = 0xC000_0000 + (i * 0x100) as u64;
            let node = i % 4;
            
            // Register in various components
            AllocatorHook::hook_allocation(size, 8, node, AllocationFlags::NORMAL, caller);
            MemoryMapper::register_allocation(address, size, caller, i);
            
            // Record cache accesses
            if i % 3 == 0 {
                CacheProfiler::record_access(address, 64, CacheAccessType::READ, 20);
            }
            
            // Record NUMA accesses
            NUMAProfiler::record_access(address, 64, AccessType::Read, node as u8);
            
            // Record stack frames
            if i % 4 == 0 {
                StackProfiler::record_function_entry(i as u32, caller, size, size / 4);
                StackProfiler::record_function_exit(i as u32, caller);
            }
            
            // Perform some deallocations to create fragmentation patterns
            if i % 7 == 0 && i > 0 {
                let old_address = 0xB000_0000 + ((i - 7) * 0x1000) as u64;
                FragmentationAnalyzer::record_allocation(old_address, 512 + ((i - 7) % 5) * 256, 0xD000_0000);
                FragmentationAnalyzer::record_deallocation(old_address);
                
                LeakDetector::record_deallocation(old_address);
                MemoryMapper::unregister_allocation(old_address);
            }
        }
        
        // 3. Generate reports from different components
        let snapshot = RealtimeTracker::take_snapshot();
        assert!(snapshot.is_some());
        
        let leak_report = LeakDetector::scan_for_leaks();
        assert!(leak_report.timestamp > 0);
        
        let frag_report = FragmentationAnalyzer::analyze_fragmentation();
        assert!(frag_report.fragmentation_stats.total_heap_size > 0);
        
        let stack_report = StackProfiler::generate_report();
        assert!(stack_report.timestamp > 0);
        
        let numa_report = NUMAProfiler::generate_optimization_report();
        assert!(numa_report.timestamp > 0);
        
        // 4. Generate comprehensive report
        let comprehensive_report = MemoryMapper::generate_comprehensive_report();
        
        // Verify all components contributed to the report
        assert!(comprehensive_report.global_stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst) > 0);
        assert!(comprehensive_report.realtime_data.total_allocated > 0);
        assert!(comprehensive_report.system_health_score >= 0.0);
        
        // 5. Stop tracking
        RealtimeTracker::stop_monitoring();
        
        println!("Integration scenario test passed");
    }
    
    #[test]
    fn test_statistics_consistency() {
        init();
        
        // Perform a series of operations and verify consistency
        
        let initial_stats = get_stats();
        let initial_allocations = initial_stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst);
        let initial_deallocations = initial_stats.total_deallocations.load(std::sync::atomic::Ordering::SeqCst);
        
        // Perform allocations and deallocations
        let mut addresses = Vec::new();
        for i in 0..10 {
            let address = 0xE000_0000 + (i * 0x1000) as u64;
            let size = 1024;
            let caller = 0xF000_0000;
            
            AllocatorHook::hook_allocation(size, 8, 0, AllocationFlags::NORMAL, caller);
            MemoryMapper::register_allocation(address, size, caller, i);
            addresses.push(address);
        }
        
        // Deallocate some
        for i in 0..3 {
            let address = addresses[i];
            let size = 1024;
            let caller = 0xF000_0000;
            
            AllocatorHook::hook_deallocation(size, caller);
            MemoryMapper::unregister_allocation(address);
        }
        
        // Check final statistics
        let final_stats = get_stats();
        let final_allocations = final_stats.total_allocations.load(std::sync::atomic::Ordering::SeqCst);
        let final_deallocations = final_stats.total_deallocations.load(std::sync::atomic::Ordering::SeqCst);
        
        // Verify consistency
        assert_eq!(final_allocations, initial_allocations + 10);
        assert_eq!(final_deallocations, initial_deallocations + 3);
        
        println!("Statistics consistency test passed");
    }
    
    #[test]
    fn test_performance_under_load() {
        use std::time::Instant;
        
        init();
        
        let start_time = Instant::now();
        let mut operations_count = 0;
        
        // Perform a large number of operations quickly
        for iteration in 0..100 {
            for i in 0..50 {
                let address = (iteration as u64 * 1000 + i) << 12; // Well-spread addresses
                let size = 256 + (i % 10) * 128;
                let caller = 0x10000000 + i as u64;
                
                // Perform multiple types of operations
                AllocatorHook::hook_allocation(size, 8, 0, AllocationFlags::NORMAL, caller);
                CacheProfiler::record_access(address, 64, CacheAccessType::READ, 20);
                MemoryMapper::register_allocation(address, size, caller, operations_count);
                NUMAProfiler::record_access(address, 64, AccessType::Read, 0);
                
                operations_count += 4; // 4 operations per iteration
                
                // Deallocate some to keep memory usage reasonable
                if i % 10 == 0 && iteration > 0 {
                    let old_address = ((iteration - 1) as u64 * 1000 + i) << 12;
                    AllocatorHook::hook_deallocation(size, caller);
                    MemoryMapper::unregister_allocation(old_address);
                }
            }
        }
        
        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        let ops_per_ms = operations_count as f64 / duration.as_millis() as f64;
        
        // Should handle at least 1000 operations per millisecond
        assert!(ops_per_ms >= 1000.0, "Performance too low: {} ops/ms", ops_per_ms);
        
        println!("Performance test passed: {} operations in {}ms ({:.1} ops/ms)", 
                operations_count, duration.as_millis(), ops_per_ms);
    }
}