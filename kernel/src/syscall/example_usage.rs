//! MultiOS System Call Enhancement Modules - Usage Examples
//! 
//! This module demonstrates how to use the new syscall enhancement modules.
//! It provides practical examples and validation of the performance monitoring,
//! error handling, and assembly interface functionality.

use crate::syscall::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Example demonstrating performance monitoring usage
pub fn performance_monitoring_example() {
    println!("\n=== Performance Monitoring Example ===");
    
    // Initialize performance monitor
    let performance_monitor = Arc::new(Mutex::new(SyscallPerformanceMonitor::new()));
    
    println!("📊 Simulating syscall performance tracking...");
    
    // Simulate various syscalls with different performance characteristics
    let syscall_scenarios = vec![
        (syscall_numbers::FILE_OPEN, "File Open", Duration::from_micros(150)),
        (syscall_numbers::FILE_READ, "File Read", Duration::from_micros(80)),
        (syscall_numbers::PROCESS_GETPID, "Get PID", Duration::from_micros(10)),
        (syscall_numbers::THREAD_YIELD, "Thread Yield", Duration::from_micros(5)),
        (syscall_numbers::MEMORY_INFO, "Memory Info", Duration::from_micros(25)),
    ];
    
    for (i, &(syscall_num, name, duration)) in syscall_scenarios.iter().enumerate() {
        println!("  Processing {} (syscall {})...", name, syscall_num);
        
        // Record syscall start
        {
            let mut monitor = performance_monitor.lock().unwrap();
            monitor.record_syscall_start(syscall_num, 1000 + i as u64);
        }
        
        // Simulate processing time
        std::thread::sleep(duration);
        
        // Simulate occasional errors
        let should_error = i % 4 == 3;
        let error = if should_error {
            Some(SyscallError::ResourceUnavailable)
        } else {
            None
        };
        
        // Record syscall completion
        {
            let mut monitor = performance_monitor.lock().unwrap();
            let stats = monitor.record_syscall_complete(
                syscall_num,
                duration,
                1000 + i as u64,
                error
            );
            
            if let Some(statistics) = stats {
                println!("    ✅ Completed: {} calls, avg latency: {} ns", 
                        statistics.total_calls, statistics.average_latency_ns);
            }
        }
        
        // Log error if occurred
        if let Some(err) = error {
            println!("    ❌ Error occurred: {:?}", err);
        }
    }
    
    // Get and display performance statistics
    let perf_stats = performance_monitor.lock().unwrap().get_performance_statistics();
    println!("\n📈 Performance Summary:");
    println!("  Total syscalls: {}", perf_stats.total_syscalls);
    println!("  Average latency: {} ns", perf_stats.average_latency_ns);
    println!("  Peak latency: {} ns", perf_stats.peak_latency_ns);
    println!("  Cache hit rate: {:.1}%", perf_stats.cache_hit_rate * 100.0);
    
    // Get optimization recommendations
    let recommendations = performance_monitor.lock().unwrap().get_optimization_recommendations();
    println!("\n💡 Optimization Recommendations:");
    for recommendation in recommendations {
        println!("  • {}", recommendation);
    }
}

/// Example demonstrating error handling usage
pub fn error_handling_example() {
    println!("\n=== Error Handling Example ===");
    
    // Initialize error handler
    let error_handler = Arc::new(Mutex::new(SyscallErrorHandler::new()));
    
    println!("🚨 Simulating comprehensive error handling...");
    
    // Create error contexts for different processes
    for i in 0..3 {
        let context = error_handler.lock().unwrap().create_error_context(2000 + i as u64);
        println!("  Created error context for process {}: {}", 2000 + i, context.is_some());
    }
    
    // Simulate various error scenarios
    let error_scenarios = vec![
        (SyscallError::InvalidArgument, "Invalid parameter passed to syscall"),
        (SyscallError::PermissionDenied, "User lacks permission for operation"),
        (SyscallError::FileNotFound, "Requested file does not exist"),
        (SyscallError::MemoryAllocationFailed, "System out of memory"),
        (SyscallError::ResourceUnavailable, "Resource is currently busy"),
    ];
    
    for (i, &(error_type, description)) in error_scenarios.iter().enumerate() {
        println!("\n  Scenario {}: {}", i + 1, description);
        
        // Log the error
        error_handler.lock().unwrap().log_error(
            error_type,
            3000 + i as u64,
            description
        );
        
        // Get recovery strategy
        let recovery_strategy = error_handler.lock().unwrap().get_recovery_strategy(error_type);
        println!("    Recovery strategy: {:?}", recovery_strategy);
        
        // Execute recovery
        let mut recovery_params = HashMap::new();
        recovery_params.insert("retry_count".to_string(), 3);
        recovery_params.insert("timeout_ms".to_string(), 5000);
        
        let recovery_result = error_handler.lock().unwrap().execute_recovery(
            error_type,
            3000 + i as u64,
            &recovery_params
        );
        
        match recovery_result {
            Ok(action) => println!("    ✅ Recovery successful: {:?}", action),
            Err(err) => println!("    ❌ Recovery failed: {:?}", err),
        }
    }
    
    // Get error statistics
    let error_stats = error_handler.lock().unwrap().get_error_statistics();
    println!("\n📊 Error Statistics:");
    println!("  Total errors: {}", error_stats.total_errors);
    println!("  Unique error types: {}", error_stats.error_counts.len());
    
    for (error_type, count) in &error_stats.error_counts {
        println!("    {:?}: {} occurrences", error_type, count);
    }
    
    // Generate detailed error report
    let detailed_report = error_handler.lock().unwrap().generate_detailed_error_report();
    println!("\n📝 Detailed Error Report Preview:");
    let report_lines: Vec<&str> = detailed_report.lines().collect();
    for line in report_lines.iter().take(5) {
        println!("    {}", line);
    }
    if detailed_report.lines().count() > 5 {
        println!("    ... ({} more lines)", detailed_report.lines().count() - 5);
    }
    
    // Test user-friendly error messages
    println!("\n💬 User-Friendly Error Messages:");
    for (error_type, _) in error_scenarios.iter().take(3) {
        let message = error_handler.lock().unwrap().get_user_friendly_message(*error_type);
        println!("  {:?}: {}", error_type, message);
    }
}

/// Example demonstrating syscall number registry usage
pub fn syscall_registry_example() {
    println!("\n=== Syscall Registry Example ===");
    
    println!("🔍 Exploring available syscalls...");
    
    // Get all syscalls
    let all_syscalls = syscall_numbers::get_all_syscalls();
    println!("  Total syscalls defined: {}", all_syscalls.len());
    
    // Show some example syscalls by category
    let categories = vec![
        ("Process Management", vec![
            syscall_numbers::PROCESS_CREATE,
            syscall_numbers::PROCESS_EXIT,
            syscall_numbers::PROCESS_GETPID,
            syscall_numbers::PROCESS_WAIT,
        ]),
        ("File Operations", vec![
            syscall_numbers::FILE_OPEN,
            syscall_numbers::FILE_CLOSE,
            syscall_numbers::FILE_READ,
            syscall_numbers::FILE_WRITE,
        ]),
        ("Thread Operations", vec![
            syscall_numbers::THREAD_CREATE,
            syscall_numbers::THREAD_EXIT,
            syscall_numbers::THREAD_YIELD,
            syscall_numbers::THREAD_GETTID,
        ]),
        ("Memory Management", vec![
            syscall_numbers::VIRTUAL_ALLOC,
            syscall_numbers::VIRTUAL_FREE,
            syscall_numbers::VIRTUAL_MAP,
            syscall_numbers::MEMORY_INFO,
        ]),
    ];
    
    for (category_name, syscall_list) in categories {
        println!("\n  {}:", category_name);
        for &syscall_num in &syscall_list {
            if let Some(info) = syscall_numbers::get_syscall_info(syscall_num) {
                println!("    {:3}: {} - {}", syscall_num, info.name, info.description);
            } else {
                println!("    {:3}: <unknown syscall>", syscall_num);
            }
        }
    }
    
    // Search syscalls by name
    println!("\n🔎 Searching syscalls containing 'file':");
    let file_syscalls = syscall_numbers::search_syscalls("file");
    for syscall in file_syscalls.iter().take(5) {
        println!("  {:3}: {}", syscall.number, syscall.name);
    }
    
    // Get syscall statistics
    println!("\n📈 Syscall Statistics:");
    let stats = syscall_numbers::get_syscall_statistics();
    println!("  Total defined syscalls: {}", stats.total_syscalls);
    println!("  Categories: {}", stats.categories.len());
    for (category, count) in &stats.categories {
        println!("    {}: {} syscalls", category, count);
    }
}

/// Example demonstrating assembly interface usage
pub fn assembly_interface_example() {
    println!("\n=== Assembly Interface Example ===");
    
    // Initialize assembly interface
    let assembly_interface = Arc::new(Mutex::new(AssemblySyscallInterface::new()));
    
    println!("🔧 Demonstrating x86_64 assembly interface...");
    
    // Test x86_64 specific features
    {
        let asm = assembly_interface.lock().unwrap();
        
        // Get syscall entry point
        let entry_point = asm.get_syscall_entry_point(crate::arch::ArchType::X86_64);
        println!("  Syscall entry point: {:?}", entry_point);
        
        // Generate syscall instruction
        let instruction = asm.generate_syscall_instruction(
            crate::arch::ArchType::X86_64,
            syscall_numbers::FILE_OPEN
        );
        println!("  Syscall instruction: {}", instruction);
        
        // Show register mappings
        println!("\n  Register Mappings:");
        let param_regs = asm.get_parameter_registers();
        for (param, reg) in param_regs.iter().take(6) {
            println!("    {} -> {}", param, reg);
        }
        println!("    return -> {}", asm.get_return_register());
        
        // Show optimization settings
        println!("\n  Optimization Settings:");
        let optimizations = asm.get_optimization_settings();
        println!("    Fast path enabled: {}", optimizations.enable_fast_path);
        println!("    Branch prediction: {}", optimizations.enable_branch_prediction);
        println!("    Instruction cache optimization: {}", optimizations.enable_instruction_cache_optimization);
        
        // Show hot paths
        let hot_paths = asm.get_hot_paths();
        println!("\n  Hot Paths (frequently called syscalls):");
        for path in hot_paths.iter().take(5) {
            println!("    {}", path);
        }
    }
    
    // Test fast path caching
    println!("\n  Fast Path Cache Test:");
    {
        let asm = assembly_interface.lock().unwrap();
        for i in 0..5 {
            let fast_path = asm.get_fast_path(syscall_numbers::FILE_OPEN + i);
            println!("    Syscall {} fast path: {}", syscall_numbers::FILE_OPEN + i, fast_path.is_some());
        }
    }
    
    // Test context switching
    println!("\n  Context Switching Test:");
    {
        let asm = assembly_interface.lock().unwrap();
        let context_size = asm.get_context_save_size();
        println!("    Context save size: {} bytes", context_size);
        
        let stack_offset = asm.get_stack_pointer_offset();
        println!("    Stack pointer offset: {}", stack_offset);
        
        let privilege_transition = asm.requires_privilege_transition(
            crate::arch::PrivilegeLevel::Ring3,
            crate::arch::PrivilegeLevel::Ring0
        );
        println!("    Ring3 -> Ring0 transition required: {}", privilege_transition);
    }
}

/// Example demonstrating integrated system usage
pub fn integrated_system_example() {
    println!("\n=== Integrated System Example ===");
    
    // Initialize all components
    let performance_monitor = Arc::new(Mutex::new(SyscallPerformanceMonitor::new()));
    let error_handler = Arc::new(Mutex::new(SyscallErrorHandler::new()));
    let assembly_interface = Arc::new(Mutex::new(AssemblySyscallInterface::new()));
    
    println!("🔄 Simulating complete syscall workflow...");
    
    let start_time = Instant::now();
    let mut total_syscalls = 0;
    let mut total_errors = 0;
    
    // Simulate various syscall operations
    for i in 0..20 {
        let syscall_num = match i % 6 {
            0 => syscall_numbers::FILE_OPEN,
            1 => syscall_numbers::FILE_READ,
            2 => syscall_numbers::PROCESS_GETPID,
            3 => syscall_numbers::THREAD_YIELD,
            4 => syscall_numbers::MEMORY_INFO,
            _ => syscall_numbers::SYSTEM_INFO,
        };
        
        // Get syscall info
        let syscall_info = syscall_numbers::get_syscall_info(syscall_num);
        println!("  Processing {} ({})...", 
                syscall_info.map(|s| s.name).unwrap_or("unknown"),
                syscall_num);
        
        // Record performance start
        {
            let mut monitor = performance_monitor.lock().unwrap();
            monitor.record_syscall_start(syscall_num, 4000 + i as u64);
        }
        
        // Check if we should use assembly fast path
        let use_fast_path = {
            let asm = assembly_interface.lock().unwrap();
            asm.get_fast_path(syscall_num).is_some()
        };
        
        if use_fast_path {
            println!("    ⚡ Using fast path");
        }
        
        // Simulate processing
        let processing_time = Duration::from_micros(20 + (i as u64 % 100));
        std::thread::sleep(processing_time);
        
        // Determine if error should occur
        let should_error = i % 7 == 0;
        let error = if should_error {
            total_errors += 1;
            Some(match i % 4 {
                0 => SyscallError::InvalidArgument,
                1 => SyscallError::PermissionDenied,
                2 => SyscallError::ResourceUnavailable,
                _ => SyscallError::MemoryAllocationFailed,
            })
        } else {
            None
        };
        
        // Record completion
        {
            let mut monitor = performance_monitor.lock().unwrap();
            let stats = monitor.record_syscall_complete(
                syscall_num,
                processing_time,
                4000 + i as u64,
                error
            );
            
            if let Some(statistics) = stats {
                println!("    📊 Stats: {} calls, {} ns avg", 
                        statistics.total_calls, statistics.average_latency_ns);
            }
        }
        
        // Handle error if occurred
        if let Some(err) = error {
            println!("    ❌ Error: {:?}", err);
            
            // Log error
            error_handler.lock().unwrap().log_error(
                err,
                4000 + i as u64,
                "Integrated workflow test"
            );
            
            // Attempt recovery
            let recovery_result = error_handler.lock().unwrap().execute_recovery(
                err,
                4000 + i as u64,
                &HashMap::new()
            );
            
            match recovery_result {
                Ok(action) => println!("    🔄 Recovery: {:?}", action),
                Err(rec_err) => println!("    💥 Recovery failed: {:?}", rec_err),
            }
        } else {
            println!("    ✅ Success");
        }
        
        total_syscalls += 1;
    }
    
    let total_time = start_time.elapsed();
    let throughput = total_syscalls as f64 / total_time.as_secs_f64();
    
    // Final statistics
    println!("\n📊 Integrated System Results:");
    println!("  Total syscalls: {}", total_syscalls);
    println!("  Total errors: {}", total_errors);
    println!("  Success rate: {:.1}%", ((total_syscalls - total_errors) as f64 / total_syscalls as f64) * 100.0);
    println!("  Throughput: {:.2} syscalls/second", throughput);
    println!("  Total time: {:?}", total_time);
    
    // Get final statistics
    let perf_stats = performance_monitor.lock().unwrap().get_performance_statistics();
    let error_stats = error_handler.lock().unwrap().get_error_statistics();
    
    println!("\n🎯 Final System State:");
    println!("  Performance monitor active: {}", performance_monitor.lock().unwrap().is_active());
    println!("  Error handler operational: {}", error_handler.lock().unwrap().is_operational());
    println!("  Assembly interface initialized: {}", assembly_interface.lock().unwrap().is_initialized());
    
    println!("  Performance stats: {} total calls, {} ns avg latency", 
            perf_stats.total_syscalls, perf_stats.average_latency_ns);
    println!("  Error stats: {} total errors", error_stats.total_errors);
}

/// Main example function that runs all examples
pub fn run_all_examples() {
    println!("🚀 MultiOS System Call Enhancement Modules - Examples");
    println!("=====================================================");
    
    performance_monitoring_example();
    error_handling_example();
    syscall_registry_example();
    assembly_interface_example();
    integrated_system_example();
    
    println!("\n🎉 All examples completed successfully!");
    println!("   The syscall enhancement modules are ready for production use.");
}