//! Process Management for MultiOS
//! 
//! This module provides process control blocks (PCBs), process creation,
//! termination, and management functionality for the MultiOS kernel.

use alloc::vec::Vec;
use spin::Mutex;
use bitflags::bitflags;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::thread::{ThreadHandle, ThreadId, ThreadState};
use crate::scheduler_algo::SchedulerError;

/// Process ID type
pub type ProcessId = usize;

/// Process priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ProcessPriority {
    System = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Idle = 4,
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Waiting,
    Stopped,
    Terminated,
}

/// Process flags
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ProcessFlags: u32 {
        const PRIVILEGED = 0b0000_0001;
        const SYSTEM_PROCESS = 0b0000_0010;
        const CRITICAL = 0b0000_0100;
        const FOREGROUND = 0b0000_1000;
        const BACKGROUND = 0b0001_0000;
        const SUSPENDED = 0b0010_0000;
        const DETACHED = 0b0100_0000;
    }
}

/// Process Control Block (PCB)
/// 
/// Contains all information about a process including its threads,
/// memory allocation, and state information.
#[derive(Debug, Clone)]
pub struct ProcessControlBlock {
    /// Unique process ID
    pub process_id: ProcessId,
    /// Parent process ID (None for init process)
    pub parent_id: Option<ProcessId>,
    /// Process name
    pub name: alloc::vec::Vec<u8>,
    /// Process priority
    pub priority: ProcessPriority,
    /// Process state
    pub state: ProcessState,
    /// Process flags
    pub flags: ProcessFlags,
    /// List of thread handles belonging to this process
    pub threads: Vec<ThreadHandle>,
    /// Main thread ID (first thread of the process)
    pub main_thread: Option<ThreadId>,
    /// Process creation timestamp
    pub created_at: u64,
    /// CPU time used by this process (in milliseconds)
    pub cpu_time: u64,
    /// Memory usage statistics
    pub memory_stats: ProcessMemoryStats,
    /// Exit status (for terminated processes)
    pub exit_status: Option<i32>,
}

/// Memory statistics for a process
#[derive(Debug, Clone, Copy)]
pub struct ProcessMemoryStats {
    /// Total memory allocated to this process (in bytes)
    pub total_memory: usize,
    /// Shared memory usage (in bytes)
    pub shared_memory: usize,
    /// Code segment size (in bytes)
    pub code_size: usize,
    /// Data segment size (in bytes)
    pub data_size: usize,
    /// Stack size (in bytes)
    pub stack_size: usize,
}

/// Process creation parameters
#[derive(Debug, Clone)]
pub struct ProcessCreateParams {
    /// Process name
    pub name: alloc::vec::Vec<u8>,
    /// Process priority
    pub priority: ProcessPriority,
    /// Process flags
    pub flags: ProcessFlags,
    /// Initial thread entry point
    pub entry_point: Option<fn()>,
    /// Initial thread parameters
    pub thread_params: Option<()>, // Could be extended with actual parameters
}

/// Process management result
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Error types for process operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessError {
    ProcessNotFound,
    InvalidProcessId,
    ProcessAlreadyExists,
    ProcessLimitExceeded,
    ThreadCreationFailed,
    InvalidPriority,
    AccessDenied,
    ProcessInInvalidState,
    OutOfMemory,
}

/// Process Manager
/// 
/// Global manager for all processes in the system.
pub struct ProcessManager {
    /// Next available process ID
    next_process_id: AtomicUsize,
    /// Active processes map (process_id -> PCB)
    processes: spin::Mutex<alloc::vec::Vec<Option<ProcessControlBlock>>>,
    /// Process by parent-child relationships
    process_tree: spin::Mutex<alloc::vec::Vec<Vec<ProcessId>>>,
}

impl ProcessManager {
    /// Create a new process manager
    pub const fn new() -> Self {
        Self {
            next_process_id: AtomicUsize::new(1),
            processes: spin::Mutex::new(alloc::vec::Vec::new()),
            process_tree: spin::Mutex::new(alloc::vec::Vec::new()),
        }
    }

    /// Create a new process
    pub fn create_process(&self, params: ProcessCreateParams) -> ProcessResult<ProcessId> {
        let mut processes = self.processes.lock();
        let mut process_tree = self.process_tree.lock();

        let process_id = self.next_process_id.fetch_add(1, Ordering::SeqCst);
        
        // Ensure we have enough space in the processes vector
        if process_id >= processes.len() {
            processes.resize(process_id + 1, None);
            process_tree.resize(process_id + 1, alloc::vec::Vec::new());
        }

        // Create the PCB
        let pcb = ProcessControlBlock {
            process_id,
            parent_id: None, // Set by caller if needed
            name: params.name,
            priority: params.priority,
            state: ProcessState::Running,
            flags: params.flags,
            threads: Vec::new(),
            main_thread: None,
            created_at: 0, // Would be set from kernel time
            cpu_time: 0,
            memory_stats: ProcessMemoryStats {
                total_memory: 0,
                shared_memory: 0,
                code_size: 0,
                data_size: 0,
                stack_size: 4096, // Default stack size
            },
            exit_status: None,
        };

        // Store the process
        processes[process_id] = Some(pcb);

        // Add to process tree if we have a parent
        // (this would be set by the caller)

        Ok(process_id)
    }

    /// Get a process by ID
    pub fn get_process(&self, process_id: ProcessId) -> ProcessResult<Arc<Mutex<ProcessControlBlock>>> {
        let processes = self.processes.lock();
        
        if process_id >= processes.len() {
            return Err(ProcessError::ProcessNotFound);
        }

        match &processes[process_id] {
            Some(pcb) => Ok(Arc::new(Mutex::new(pcb.clone()))),
            None => Err(ProcessError::ProcessNotFound),
        }
    }

    /// Terminate a process
    pub fn terminate_process(&self, process_id: ProcessId, exit_status: i32) -> ProcessResult<()> {
        let mut processes = self.processes.lock();
        
        if process_id >= processes.len() || processes[process_id].is_none() {
            return Err(ProcessError::ProcessNotFound);
        }

        if let Some(ref mut pcb) = processes[process_id] {
            pcb.state = ProcessState::Terminated;
            pcb.exit_status = Some(exit_status);

            // Terminate all threads in this process
            for thread_handle in &pcb.threads {
                // This would notify the thread manager to terminate threads
                // thread::terminate_thread(*thread_handle)?;
            }
        }

        Ok(())
    }

    /// Get all process IDs
    pub fn get_all_processes(&self) -> Vec<ProcessId> {
        let processes = self.processes.lock();
        let mut result = Vec::new();

        for (i, process) in processes.iter().enumerate() {
            if process.is_some() {
                result.push(i);
            }
        }

        result
    }

    /// Get processes by priority
    pub fn get_processes_by_priority(&self, priority: ProcessPriority) -> Vec<ProcessId> {
        let processes = self.processes.lock();
        let mut result = Vec::new();

        for process in processes.iter() {
            if let Some(pcb) = process {
                if pcb.priority == priority {
                    result.push(pcb.process_id);
                }
            }
        }

        result
    }

    /// Get process statistics
    pub fn get_process_stats(&self, process_id: ProcessId) -> ProcessResult<ProcessStats> {
        let processes = self.processes.lock();
        
        if process_id >= processes.len() || processes[process_id].is_none() {
            return Err(ProcessError::ProcessNotFound);
        }

        let pcb = processes[process_id].as_ref().unwrap();
        
        Ok(ProcessStats {
            process_id: pcb.process_id,
            name: String::from_utf8_lossy(&pcb.name).to_string(),
            priority: pcb.priority,
            state: pcb.state,
            thread_count: pcb.threads.len(),
            cpu_time: pcb.cpu_time,
            memory_stats: pcb.memory_stats,
            uptime: 0, // Would calculate from creation time
        })
    }

    /// Check if a process is running
    pub fn is_process_running(&self, process_id: ProcessId) -> ProcessResult<bool> {
        let processes = self.processes.lock();
        
        if process_id >= processes.len() || processes[process_id].is_none() {
            return Err(ProcessError::ProcessNotFound);
        }

        let pcb = processes[process_id].as_ref().unwrap();
        Ok(matches!(pcb.state, ProcessState::Running))
    }

    /// Set process priority
    pub fn set_process_priority(&self, process_id: ProcessId, priority: ProcessPriority) -> ProcessResult<()> {
        let mut processes = self.processes.lock();
        
        if process_id >= processes.len() || processes[process_id].is_none() {
            return Err(ProcessError::ProcessNotFound);
        }

        if let Some(ref mut pcb) = processes[process_id] {
            pcb.priority = priority;
        }

        Ok(())
    }

    /// Suspend a process
    pub fn suspend_process(&self, process_id: ProcessId) -> ProcessResult<()> {
        let mut processes = self.processes.lock();
        
        if process_id >= processes.len() || processes[process_id].is_none() {
            return Err(ProcessError::ProcessNotFound);
        }

        if let Some(ref mut pcb) = processes[process_id] {
            pcb.flags.insert(ProcessFlags::SUSPENDED);
        }

        Ok(())
    }

    /// Resume a process
    pub fn resume_process(&self, process_id: ProcessId) -> ProcessResult<()> {
        let mut processes = self.processes.lock();
        
        if process_id >= processes.len() || processes[process_id].is_none() {
            return Err(ProcessError::ProcessNotFound);
        }

        if let Some(ref mut pcb) = processes[process_id] {
            pcb.flags.remove(ProcessFlags::SUSPENDED);
        }

        Ok(())
    }
}

/// Process statistics structure
#[derive(Debug, Clone)]
pub struct ProcessStats {
    pub process_id: ProcessId,
    pub name: String,
    pub priority: ProcessPriority,
    pub state: ProcessState,
    pub thread_count: usize,
    pub cpu_time: u64,
    pub memory_stats: ProcessMemoryStats,
    pub uptime: u64,
}

/// Global process manager instance
pub static PROCESS_MANAGER: ProcessManager = ProcessManager::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let manager = ProcessManager::new();
        
        let params = ProcessCreateParams {
            name: b"test_process".to_vec(),
            priority: ProcessPriority::Normal,
            flags: ProcessFlags::empty(),
            entry_point: None,
            thread_params: None,
        };

        let result = manager.create_process(params);
        assert!(result.is_ok());
        
        let process_id = result.unwrap();
        assert_eq!(process_id, 1);
    }

    #[test]
    fn test_process_priority_ordering() {
        assert!(ProcessPriority::System < ProcessPriority::High);
        assert!(ProcessPriority::High < ProcessPriority::Normal);
        assert!(ProcessPriority::Normal < ProcessPriority::Low);
        assert!(ProcessPriority::Low < ProcessPriority::Idle);
    }

    #[test]
    fn test_process_flags() {
        let flags = ProcessFlags::PRIVILEGED | ProcessFlags::CRITICAL;
        assert!(flags.contains(ProcessFlags::PRIVILEGED));
        assert!(flags.contains(ProcessFlags::CRITICAL));
        assert!(!flags.contains(ProcessFlags::BACKGROUND));
    }

    #[test]
    fn test_process_state() {
        let state = ProcessState::Running;
        assert_ne!(state, ProcessState::Terminated);
    }
}