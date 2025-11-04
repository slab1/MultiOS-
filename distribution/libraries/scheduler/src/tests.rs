//! Comprehensive tests for the MultiOS Process and Thread Management System
//! 
//! This test suite validates all aspects of the process and thread management
//! implementation including creation, scheduling, synchronization, and error handling.

#![no_std]
#![cfg(test)]

use crate::process::{
    ProcessManager, ProcessCreateParams, ProcessPriority, ProcessState, 
    ProcessFlags, PROCESS_MANAGER, ProcessError, ProcessStats
};
use crate::thread::{
    ThreadManager, ThreadParams, ThreadEntry, THREAD_MANAGER, 
    ThreadError, ThreadFlags, ThreadStats
};
use crate::scheduler_algo::{
    Scheduler, SchedulerConfig, SchedulingAlgorithm, ReadyQueue, 
    SchedulerStatsSnapshot, SchedulerHelpers
};
use crate::{init, init_with_config, get_cpu_count, is_system_ready};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test basic scheduler initialization
    #[test]
    fn test_scheduler_initialization() {
        assert!(!is_system_ready());
        
        let result = init();
        assert!(result.is_ok());
        
        assert!(is_system_ready());
        assert!(get_cpu_count() > 0);
    }

    /// Test custom scheduler configuration
    #[test]
    fn test_custom_scheduler_config() {
        let config = SchedulerConfig {
            algorithm: SchedulingAlgorithm::PriorityBased,
            cpu_count: 2,
            default_time_quantum: 15,
            load_balance_interval: 75,
            enable_cpu_affinity: true,
            enable_load_balancing: false,
        };

        let result = init_with_config(config);
        assert!(result.is_ok());
        assert!(is_system_ready());
    }

    /// Test process creation and basic operations
    #[test]
    fn test_process_creation_and_management() {
        init().unwrap();

        // Create a simple process
        let params = ProcessCreateParams {
            name: b"test_process".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(params);
        assert!(process_id.is_ok());

        let created_id = process_id.unwrap();
        assert_eq!(created_id, 1);

        // Test getting process statistics
        let stats = PROCESS_MANAGER.get_process_stats(created_id);
        assert!(stats.is_ok());

        let process_stats = stats.unwrap();
        assert_eq!(process_stats.process_id, created_id);
        assert_eq!(process_stats.name, "test_process");
        assert_eq!(process_stats.priority, ProcessPriority::Normal);

        // Test process state
        let is_running = PROCESS_MANAGER.is_process_running(created_id);
        assert!(is_running.is_ok());
        assert!(is_running.unwrap());

        // Test setting process priority
        let new_priority = ProcessPriority::High;
        let result = PROCESS_MANAGER.set_process_priority(created_id, new_priority);
        assert!(result.is_ok());

        // Verify priority change
        let updated_stats = PROCESS_MANAGER.get_process_stats(created_id).unwrap();
        assert_eq!(updated_stats.priority, new_priority);

        // Test process termination
        let result = PROCESS_MANAGER.terminate_process(created_id, 0);
        assert!(result.is_ok());

        // Verify process is terminated
        let is_running = PROCESS_MANAGER.is_process_running(created_id);
        assert!(is_running.is_ok());
        assert!(!is_running.unwrap());
    }

    /// Test thread creation and management
    #[test]
    fn test_thread_creation_and_management() {
        init().unwrap();

        // Create a process first
        let process_params = ProcessCreateParams {
            name: b"thread_test_process".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(process_params).unwrap();

        // Create a test thread entry point
        let test_entry: ThreadEntry = || {
            loop {
                crate::yield_cpu();
            }
        };

        // Create a simple thread
        let thread_params = ThreadParams {
            stack_size: 4096,
            priority: crate::Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let thread_handle = THREAD_MANAGER.create_thread(
            process_id,
            b"test_thread".to_vec(),
            Some(test_entry),
            thread_params
        );
        assert!(thread_handle.is_ok());

        let created_handle = thread_handle.unwrap();
        let thread_id = created_handle.lock().thread_id;

        // Test thread statistics
        let stats = THREAD_MANAGER.get_thread_stats(thread_id);
        assert!(stats.is_ok());

        let thread_stats = stats.unwrap();
        assert_eq!(thread_stats.thread_id, thread_id);
        assert_eq!(thread_stats.process_id, process_id);
        assert_eq!(thread_stats.name, "test_thread");
        assert_eq!(thread_stats.priority, crate::Priority::Normal);

        // Test thread priority setting
        let new_priority = crate::Priority::High;
        let result = THREAD_MANAGER.set_thread_priority(thread_id, new_priority);
        assert!(result.is_ok());

        // Test thread sleeping
        let result = THREAD_MANAGER.sleep_thread(thread_id, 1000);
        assert!(result.is_ok());

        // Test thread waking
        let result = THREAD_MANAGER.wake_thread(thread_id);
        assert!(result.is_ok());

        // Test thread termination
        let result = THREAD_MANAGER.terminate_thread(thread_id);
        assert!(result.is_ok());
    }

    /// Test thread queries and filtering
    #[test]
    fn test_thread_queries() {
        init().unwrap();

        // Create a process
        let process_params = ProcessCreateParams {
            name: b"query_test_process".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(process_params).unwrap();

        // Create multiple threads with different priorities
        let test_entry: ThreadEntry = || loop { crate::yield_cpu(); };

        let priorities = [
            crate::Priority::Idle,
            crate::Priority::Low,
            crate::Priority::Normal,
            crate::Priority::High,
            crate::Priority::Critical,
        ];

        let mut created_thread_ids = Vec::new();

        for (i, &priority) in priorities.iter().enumerate() {
            let thread_params = ThreadParams {
                stack_size: 4096,
                priority,
                detached: false,
                inherit_priority: false,
            };

            let thread_name = format!("priority_thread_{}", i);
            
            let thread_handle = THREAD_MANAGER.create_thread(
                process_id,
                thread_name.into_bytes(),
                Some(test_entry),
                thread_params
            );

            if let Ok(handle) = thread_handle {
                created_thread_ids.push(handle.lock().thread_id);
            }
        }

        // Test getting all threads
        let all_threads = THREAD_MANAGER.get_all_threads();
        assert!(!all_threads.is_empty());

        // Test getting threads by process
        let process_threads = THREAD_MANAGER.get_threads_by_process(process_id);
        assert_eq!(process_threads.len(), created_thread_ids.len());

        // Test getting threads by priority
        for &priority in &priorities {
            let priority_threads = THREAD_MANAGER.get_threads_by_priority(priority);
            assert!(!priority_threads.is_empty());
        }

        // Clean up
        for thread_id in created_thread_ids {
            let _ = THREAD_MANAGER.terminate_thread(thread_id);
        }
    }

    /// Test process tree and relationships
    #[test]
    fn test_process_relationships() {
        init().unwrap();

        // Create parent process
        let parent_params = ProcessCreateParams {
            name: b"parent_process".to_vec(),
            priority: ProcessPriority::High,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let parent_id = PROCESS_MANAGER.create_process(parent_params).unwrap();

        // Create child process (would need parent ID tracking in real implementation)
        let child_params = ProcessCreateParams {
            name: b"child_process".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let child_id = PROCESS_MANAGER.create_process(child_params).unwrap();

        // Test getting all processes
        let all_processes = PROCESS_MANAGER.get_all_processes();
        assert!(all_processes.contains(&parent_id));
        assert!(all_processes.contains(&child_id));

        // Test getting processes by priority
        let high_priority_procs = PROCESS_MANAGER.get_processes_by_priority(ProcessPriority::High);
        assert!(high_priority_procs.contains(&parent_id));

        let normal_priority_procs = PROCESS_MANAGER.get_processes_by_priority(ProcessPriority::Normal);
        assert!(normal_priority_procs.contains(&child_id));
    }

    /// Test ready queue operations
    #[test]
    fn test_ready_queue_operations() {
        let mut ready_queue = ReadyQueue::new();

        // Test adding threads
        ready_queue.add_thread(1, crate::Priority::Normal, SchedulingAlgorithm::RoundRobin);
        ready_queue.add_thread(2, crate::Priority::High, SchedulingAlgorithm::RoundRobin);
        ready_queue.add_thread(3, crate::Priority::Low, SchedulingAlgorithm::RoundRobin);

        assert_eq!(ready_queue.len(), 3);
        assert!(!ready_queue.is_empty());

        // Test round-robin scheduling
        let next_thread_rr = ready_queue.get_next_thread(SchedulingAlgorithm::RoundRobin);
        assert!(next_thread_rr.is_some());
        
        let thread_id = next_thread_rr.unwrap();
        assert!(thread_id == 1 || thread_id == 2 || thread_id == 3);

        // Test priority scheduling
        let next_thread_priority = ready_queue.get_next_thread(SchedulingAlgorithm::PriorityBased);
        assert!(next_thread_priority.is_some());
        
        // With priority-based, should get the highest priority thread (thread 2 - High)
        assert_eq!(next_thread_priority.unwrap(), 2);

        // Test thread removal
        let removed = ready_queue.remove_thread(2);
        assert!(removed);
        assert_eq!(ready_queue.len(), 2);

        // Test removal of non-existent thread
        let removed_fake = ready_queue.remove_thread(999);
        assert!(!removed_fake);
    }

    /// Test scheduler statistics
    #[test]
    fn test_scheduler_statistics() {
        init().unwrap();

        // Create some processes and threads
        let process_params = ProcessCreateParams {
            name: b"stats_test_process".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(process_params).unwrap();

        let thread_params = ThreadParams {
            stack_size: 4096,
            priority: crate::Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let thread_entry: ThreadEntry = || loop { crate::yield_cpu(); };

        for i in 0..3 {
            let thread_name = format!("stats_thread_{}", i);
            
            let _ = THREAD_MANAGER.create_thread(
                process_id,
                thread_name.into_bytes(),
                Some(thread_entry),
                thread_params.clone()
            );
        }

        // Get scheduler statistics
        let stats = crate::get_scheduler_stats();
        assert!(stats.cpu_count > 0);
        assert!(stats.algorithm == SchedulingAlgorithm::RoundRobin);

        // Get process statistics
        let process_stats = PROCESS_MANAGER.get_process_stats(process_id);
        assert!(process_stats.is_ok());

        let proc_stats = process_stats.unwrap();
        assert_eq!(proc_stats.thread_count, 3);
        assert!(proc_stats.cpu_time >= 0);
    }

    /// Test error handling
    #[test]
    fn test_error_handling() {
        init().unwrap();

        // Test process not found error
        let result = PROCESS_MANAGER.get_process_stats(99999);
        assert!(matches!(result, Err(ProcessError::ProcessNotFound)));

        let result = PROCESS_MANAGER.is_process_running(99999);
        assert!(matches!(result, Err(ProcessError::ProcessNotFound)));

        let result = PROCESS_MANAGER.set_process_priority(99999, ProcessPriority::High);
        assert!(matches!(result, Err(ProcessError::ProcessNotFound)));

        // Test thread not found error
        let result = THREAD_MANAGER.get_thread_stats(99999);
        assert!(matches!(result, Err(ThreadError::ThreadNotFound)));

        let result = THREAD_MANAGER.set_thread_priority(99999, crate::Priority::High);
        assert!(matches!(result, Err(ThreadError::ThreadNotFound)));

        let result = THREAD_MANAGER.terminate_thread(99999);
        assert!(matches!(result, Err(ThreadError::ThreadNotFound)));
    }

    /// Test process flags and attributes
    #[test]
    fn test_process_flags() {
        let flags = ProcessFlags::PRIVILEGED | ProcessFlags::CRITICAL;
        
        assert!(flags.contains(ProcessFlags::PRIVILEGED));
        assert!(flags.contains(ProcessFlags::CRITICAL));
        assert!(!flags.contains(ProcessFlags::BACKGROUND));

        let inverted_flags = !flags;
        assert!(inverted_flags.contains(ProcessFlags::BACKGROUND));
    }

    /// Test thread flags and attributes
    #[test]
    fn test_thread_flags() {
        let flags = ThreadFlags::DETACHED | ThreadFlags::DAEMON;
        
        assert!(flags.contains(ThreadFlags::DETACHED));
        assert!(flags.contains(ThreadFlags::DAEMON));
        assert!(!flags.contains(ThreadFlags::SYSTEM_THREAD));
    }

    /// Test priority ordering
    #[test]
    fn test_priority_ordering() {
        // Process priorities
        assert!(ProcessPriority::System < ProcessPriority::High);
        assert!(ProcessPriority::High < ProcessPriority::Normal);
        assert!(ProcessPriority::Normal < ProcessPriority::Low);
        assert!(ProcessPriority::Low < ProcessPriority::Idle);

        // Thread priorities
        assert!(crate::Priority::Idle < crate::Priority::Low);
        assert!(crate::Priority::Low < crate::Priority::Normal);
        assert!(crate::Priority::Normal < crate::Priority::High);
        assert!(crate::Priority::High < crate::Priority::Critical);
    }

    /// Test CPU affinity operations
    #[test]
    fn test_cpu_affinity() {
        let cpu_count = get_cpu_count();
        assert!(cpu_count > 0);

        // Create a thread
        init().unwrap();
        
        let process_params = ProcessCreateParams {
            name: b"affinity_test".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(process_params).unwrap();

        let thread_params = ThreadParams {
            stack_size: 4096,
            priority: crate::Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let thread_entry: ThreadEntry = || loop { crate::yield_cpu(); };

        let thread_handle = THREAD_MANAGER.create_thread(
            process_id,
            b"affinity_thread".to_vec(),
            Some(thread_entry),
            thread_params
        );

        if let Ok(handle) = thread_handle {
            let thread_id = handle.lock().thread_id;

            // Test setting CPU affinity
            let cpu_mask: u32 = 0x3; // CPUs 0 and 1
            let result = crate::set_thread_cpu_affinity(thread_id, cpu_mask);
            assert!(result.is_ok());

            // Test checking if thread can run on specific CPU
            for cpu_id in 0..cpu_count {
                let can_run = THREAD_MANAGER.can_run_on_cpu(thread_id, cpu_id);
                assert!(can_run.is_ok());
                
                if cpu_id < 2 {
                    assert!(can_run.unwrap());
                } else {
                    assert!(!can_run.unwrap());
                }
            }
        }
    }

    /// Test process suspension and resumption
    #[test]
    fn test_process_suspension() {
        init().unwrap();

        let params = ProcessCreateParams {
            name: b"suspension_test".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(params).unwrap();

        // Test suspending process
        let result = PROCESS_MANAGER.suspend_process(process_id);
        assert!(result.is_ok());

        // Test resuming process
        let result = PROCESS_MANAGER.resume_process(process_id);
        assert!(result.is_ok());
    }

    /// Test time quantum calculations
    #[test]
    fn test_time_quantum_calculations() {
        // Test round-robin time quantums
        assert_eq!(
            SchedulerHelpers::calculate_time_quantum(crate::Priority::Idle, SchedulingAlgorithm::RoundRobin),
            5
        );
        assert_eq!(
            SchedulerHelpers::calculate_time_quantum(crate::Priority::Critical, SchedulingAlgorithm::RoundRobin),
            40
        );

        // Test priority-based time quantums
        assert_eq!(
            SchedulerHelpers::calculate_time_quantum(crate::Priority::Idle, SchedulingAlgorithm::PriorityBased),
            10
        );
        assert_eq!(
            SchedulerHelpers::calculate_time_quantum(crate::Priority::Critical, SchedulingAlgorithm::PriorityBased),
            30
        );
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    /// Stress test with many processes and threads
    #[test]
    fn test_stress_create_many_processes() {
        init().unwrap();

        let num_processes = 100;
        let mut process_ids = Vec::new();

        // Create many processes
        for i in 0..num_processes {
            let params = ProcessCreateParams {
                name: format!("stress_process_{}", i).into_bytes(),
                priority: ProcessPriority::Normal,
                flags: ProcessFlags::empty(),
                entry_point: None,
                thread_params: None,
            };

            let result = PROCESS_MANAGER.create_process(params);
            if let Ok(process_id) = result {
                process_ids.push(process_id);
            }
        }

        assert!(!process_ids.is_empty());

        // Test getting all processes
        let all_processes = PROCESS_MANAGER.get_all_processes();
        assert!(all_processes.len() >= process_ids.len());

        // Clean up
        for &process_id in &process_ids {
            let _ = PROCESS_MANAGER.terminate_process(process_id, 0);
        }
    }

    /// Stress test with many threads
    #[test]
    fn test_stress_create_many_threads() {
        init().unwrap();

        // Create one process with many threads
        let params = ProcessCreateParams {
            name: b"thread_stress_test".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let process_id = PROCESS_MANAGER.create_process(params).unwrap();

        let num_threads = 200;
        let mut thread_ids = Vec::new();

        let thread_params = ThreadParams {
            stack_size: 4096,
            priority: crate::Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let thread_entry: ThreadEntry = || loop { crate::yield_cpu(); };

        // Create many threads
        for i in 0..num_threads {
            let thread_name = format!("stress_thread_{}", i);
            
            let result = THREAD_MANAGER.create_thread(
                process_id,
                thread_name.into_bytes(),
                Some(thread_entry),
                thread_params.clone()
            );

            if let Ok(handle) = result {
                thread_ids.push(handle.lock().thread_id);
            }
        }

        assert!(!thread_ids.is_empty());

        // Test getting all threads
        let all_threads = THREAD_MANAGER.get_all_threads();
        assert!(all_threads.len() >= thread_ids.len());

        // Test getting threads by process
        let process_threads = THREAD_MANAGER.get_threads_by_process(process_id);
        assert!(process_threads.len() >= thread_ids.len());

        // Clean up
        for &thread_id in &thread_ids {
            let _ = THREAD_MANAGER.terminate_thread(thread_id);
        }
    }
}