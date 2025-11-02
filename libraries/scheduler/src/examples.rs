//! MultiOS Process and Thread Management Examples
//! 
//! This module demonstrates how to use the process and thread management
//! system in the MultiOS kernel.

#![no_std]

use crate::process::{ProcessCreateParams, ProcessPriority, ProcessFlags, PROCESS_MANAGER};
use crate::thread::{ThreadParams, ThreadEntry, THREAD_MANAGER};
use crate::{init, syscall_create_process, syscall_create_thread, get_cpu_count};

/// Example: Creating a simple process with one thread
pub fn example_create_simple_process() {
    // Define the main process parameters
    let process_params = ProcessCreateParams {
        name: b"example_app".to_vec(),
        priority: ProcessPriority::Normal,
        flags: ProcessFlags::empty(),
        entry_point: None,
        thread_params: None,
    };

    // Create the process
    let process_id = syscall_create_process(process_params)
        .expect("Failed to create process");

    // Define thread parameters
    let thread_params = ThreadParams {
        stack_size: 4096,
        priority: crate::Priority::Normal,
        detached: false,
        inherit_priority: true,
    };

    // Define the thread entry point
    let thread_entry: ThreadEntry = || {
        // Thread main function
        loop {
            // Do some work
            // For demonstration, we'll just yield
            crate::yield_cpu();
        }
    };

    // Create the main thread
    let thread_handle = syscall_create_thread(
        process_id,
        b"main_thread".to_vec(),
        Some(thread_entry),
        thread_params
    ).expect("Failed to create thread");

    println!("Created process {} with main thread", process_id);
}

/// Example: Creating a multi-threaded application
pub fn example_create_multithreaded_process() {
    // Create the process
    let process_params = ProcessCreateParams {
        name: b"web_server".to_vec(),
        priority: ProcessPriority::High,
        flags: ProcessFlags::FOREGROUND,
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(process_params)
        .expect("Failed to create process");

    // Create worker threads
    let cpu_count = get_cpu_count();
    
    for i in 0..cpu_count {
        let thread_params = ThreadParams {
            stack_size: 8192,
            priority: crate::Priority::High,
            detached: false,
            inherit_priority: false,
        };

        let thread_entry: ThreadEntry = move || {
            println!("Worker thread {} starting", i);
            
            loop {
                // Simulate work processing
                crate::yield_cpu();
            }
        };

        let thread_name = format!("worker_{}", i).into_bytes();
        
        syscall_create_thread(
            process_id,
            thread_name,
            Some(thread_entry),
            thread_params
        ).expect("Failed to create worker thread");
    }

    // Create a listener thread
    let listener_params = ThreadParams {
        stack_size: 4096,
        priority: crate::Priority::Critical,
        detached: false,
        inherit_priority: false,
    };

    let listener_entry: ThreadEntry = || {
        println!("Listener thread starting");
        
        loop {
            // Listen for incoming connections
            crate::yield_cpu();
        }
    };

    syscall_create_thread(
        process_id,
        b"listener".to_vec(),
        Some(listener_entry),
        listener_params
    ).expect("Failed to create listener thread");

    println!("Created multi-threaded web server with {} worker threads", cpu_count);
}

/// Example: Creating a real-time process
pub fn example_create_realtime_process() {
    // Create a high-priority real-time process
    let realtime_params = ProcessCreateParams {
        name: b"realtime_controller".to_vec(),
        priority: ProcessPriority::High,
        flags: ProcessFlags::CRITICAL | ProcessFlags::SYSTEM_PROCESS,
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(realtime_params)
        .expect("Failed to create real-time process");

    // Create critical control thread
    let control_params = ThreadParams {
        stack_size: 8192,
        priority: crate::Priority::Critical,
        detached: false,
        inherit_priority: false,
    };

    let control_entry: ThreadEntry = || {
        println!("Real-time control loop starting");
        
        loop {
            // Critical real-time control logic
            // Must complete within deadline
            
            crate::yield_cpu();
        }
    };

    syscall_create_thread(
        process_id,
        b"control_loop".to_vec(),
        Some(control_entry),
        control_params
    ).expect("Failed to create control thread");

    println!("Created real-time control process");
}

/// Example: Creating daemon processes
pub fn example_create_daemon_processes() {
    // Create a system daemon
    let daemon_params = ProcessCreateParams {
        name: b"system_monitor".to_vec(),
        priority: ProcessPriority::Low,
        flags: ProcessFlags::BACKGROUND | ProcessFlags::DETACHED,
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(daemon_params)
        .expect("Failed to create daemon process");

    // Create monitoring thread
    let monitor_params = ThreadParams {
        stack_size: 4096,
        priority: crate::Priority::Low,
        detached: true, // Detached thread
        inherit_priority: false,
    };

    let monitor_entry: ThreadEntry = || {
        println!("System monitor starting");
        
        loop {
            // Monitor system health
            // Low priority background work
            
            crate::yield_cpu();
        }
    };

    syscall_create_thread(
        process_id,
        b"monitor".to_vec(),
        Some(monitor_entry),
        monitor_params
    ).expect("Failed to create monitor thread");

    println!("Created system monitor daemon");
}

/// Example: Priority-based thread management
pub fn example_priority_management() {
    let process_params = ProcessCreateParams {
        name: b"priority_demo".to_vec(),
        priority: ProcessPriority::Normal,
        flags: ProcessFlags::empty(),
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(process_params)
        .expect("Failed to create process");

    // Create threads with different priorities
    let priorities = [
        (crate::Priority::Idle, "idle_worker"),
        (crate::Priority::Low, "background_task"),
        (crate::Priority::Normal, "normal_worker"),
        (crate::Priority::High, "important_task"),
        (crate::Priority::Critical, "critical_task"),
    ];

    for (priority, name) in priorities {
        let params = ThreadParams {
            stack_size: 4096,
            priority,
            detached: false,
            inherit_priority: false,
        };

        let entry: ThreadEntry = move || {
            println!("{} thread starting with priority {:?}", name, priority);
            
            loop {
                // Do work based on priority
                crate::yield_cpu();
            }
        };

        let thread_name = format!("{}_thread", name).into_bytes();
        
        syscall_create_thread(
            process_id,
            thread_name,
            Some(entry),
            params
        ).expect("Failed to create priority thread");
    }

    println!("Created process with 5 threads of different priorities");
}

/// Example: CPU affinity management
pub fn example_cpu_affinity() {
    let cpu_count = get_cpu_count();
    
    let process_params = ProcessCreateParams {
        name: b"affinity_demo".to_vec(),
        priority: ProcessPriority::Normal,
        flags: ProcessFlags::empty(),
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(process_params)
        .expect("Failed to create process");

    // Create threads bound to specific CPUs
    for cpu_id in 0..cpu_count {
        let params = ThreadParams {
            stack_size: 4096,
            priority: crate::Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let entry: ThreadEntry = move || {
            println!("Thread bound to CPU {}", cpu_id);
            
            // Set CPU affinity (bitmask for this CPU)
            let cpu_mask: u32 = 1 << cpu_id;
            let thread_id = get_current_thread_id(); // Would need to implement this
            
            crate::set_thread_cpu_affinity(thread_id, cpu_mask)
                .expect("Failed to set CPU affinity");
            
            loop {
                // Work on specific CPU
                crate::yield_cpu();
            }
        };

        let thread_name = format!("cpu_{}_thread", cpu_id).into_bytes();
        
        syscall_create_thread(
            process_id,
            thread_name,
            Some(entry),
            params
        ).expect("Failed to create CPU-bound thread");
    }

    println!("Created {} threads with CPU affinity", cpu_count);
}

/// Example: Thread synchronization
pub fn example_thread_sync() {
    let process_params = ProcessCreateParams {
        name: b"sync_demo".to_vec(),
        priority: ProcessPriority::Normal,
        flags: ProcessFlags::empty(),
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(process_params)
        .expect("Failed to create process");

    // Create producer thread
    let producer_params = ThreadParams {
        stack_size: 4096,
        priority: crate::Priority::Normal,
        detached: false,
        inherit_priority: false,
    };

    let producer_entry: ThreadEntry = || {
        println!("Producer thread starting");
        
        loop {
            // Produce data
            
            // Wake up consumer threads
            // This would use synchronization primitives
            crate::yield_cpu();
        }
    };

    // Create consumer thread
    let consumer_params = ThreadParams {
        stack_size: 4096,
        priority: crate::Priority::High,
        detached: false,
        inherit_priority: false,
    };

    let consumer_entry: ThreadEntry = || {
        println!("Consumer thread starting");
        
        loop {
            // Wait for data from producer
            
            // Process data
            crate::yield_cpu();
        }
    };

    syscall_create_thread(
        process_id,
        b"producer".to_vec(),
        Some(producer_entry),
        producer_params
    ).expect("Failed to create producer thread");

    syscall_create_thread(
        process_id,
        b"consumer".to_vec(),
        Some(consumer_entry),
        consumer_params
    ).expect("Failed to create consumer thread");

    println!("Created producer-consumer thread pair");
}

/// Example: Process monitoring and statistics
pub fn example_process_monitoring() {
    // Create a monitored process
    let process_params = ProcessCreateParams {
        name: b"monitored_app".to_vec(),
        priority: ProcessPriority::Normal,
        flags: ProcessFlags::empty(),
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(process_params)
        .expect("Failed to create process");

    // Create monitored threads
    for i in 0..3 {
        let params = ThreadParams {
            stack_size: 4096,
            priority: crate::Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let entry: ThreadEntry = move || {
            println!("Monitored thread {} starting", i);
            
            loop {
                // Do work
                crate::yield_cpu();
            }
        };

        let thread_name = format!("worker_{}", i).into_bytes();
        
        syscall_create_thread(
            process_id,
            thread_name,
            Some(entry),
            params
        ).expect("Failed to create monitored thread");
    }

    // Monitor the process
    loop {
        // Get process statistics
        if let Ok(stats) = PROCESS_MANAGER.get_process_stats(process_id) {
            println!("Process '{}': {} threads, {}ms CPU time", 
                stats.name, 
                stats.thread_count, 
                stats.cpu_time
            );
        }

        // Sleep before next check
        // In real code, this would be a timer-based check
        break;
    }

    println!("Process monitoring example complete");
}

/// Example: Custom scheduler configuration
pub fn example_custom_scheduler() {
    use crate::{init_with_config, SchedulerConfig, SchedulingAlgorithm};
    
    // Create custom scheduler configuration
    let config = SchedulerConfig {
        algorithm: SchedulingAlgorithm::PriorityBased,
        cpu_count: 8,
        default_time_quantum: 30,
        load_balance_interval: 50,
        enable_cpu_affinity: true,
        enable_load_balancing: true,
    };

    // Initialize with custom config
    init_with_config(config)
        .expect("Failed to initialize custom scheduler");

    // Create high-priority process
    let process_params = ProcessCreateParams {
        name: b"realtime_app".to_vec(),
        priority: ProcessPriority::High,
        flags: ProcessFlags::CRITICAL,
        entry_point: None,
        thread_params: None,
    };

    let process_id = syscall_create_process(process_params)
        .expect("Failed to create real-time process");

    // Create critical threads
    for i in 0..4 {
        let params = ThreadParams {
            stack_size: 8192,
            priority: crate::Priority::Critical,
            detached: false,
            inherit_priority: false,
        };

        let entry: ThreadEntry = move || {
            println!("Critical thread {} started", i);
            
            loop {
                // Real-time work
                crate::yield_cpu();
            }
        };

        let thread_name = format!("critical_{}", i).into_bytes();
        
        syscall_create_thread(
            process_id,
            thread_name,
            Some(entry),
            params
        ).expect("Failed to create critical thread");
    }

    println!("Created real-time application with custom scheduler configuration");
}

/// Helper function to get current thread ID
/// In a real implementation, this would be available through TLS
fn get_current_thread_id() -> crate::thread::ThreadId {
    0 // Placeholder
}

/// Example: Initialize scheduler and run demonstrations
pub fn run_all_examples() {
    println!("MultiOS Process and Thread Management Examples");
    println!("=============================================");

    // Initialize the scheduler
    init().expect("Failed to initialize scheduler");

    println!("\n1. Simple Process Creation:");
    example_create_simple_process();

    println!("\n2. Multi-threaded Application:");
    example_create_multithreaded_process();

    println!("\n3. Real-time Process:");
    example_create_realtime_process();

    println!("\n4. Daemon Processes:");
    example_create_daemon_processes();

    println!("\n5. Priority Management:");
    example_priority_management();

    println!("\n6. CPU Affinity:");
    example_cpu_affinity();

    println!("\n7. Thread Synchronization:");
    example_thread_sync();

    println!("\n8. Process Monitoring:");
    example_process_monitoring();

    println!("\n9. Custom Scheduler:");
    example_custom_scheduler();

    println!("\nAll examples completed successfully!");
}