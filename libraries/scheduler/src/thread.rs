//! Thread Management for MultiOS
//! 
//! This module provides thread control blocks (TCBs), thread creation,
//! termination, and management functionality for the MultiOS kernel.

use alloc::vec::Vec;
use spin::Mutex;
use bitflags::bitflags;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{Priority, ThreadState};
use crate::scheduler_algo::SchedulerError;

/// Thread ID type
pub type ThreadId = usize;

/// Thread handle - Arc wrapper for thread references
pub type ThreadHandle = Arc<Mutex<ThreadControlBlock>>;

/// Thread entry function type
pub type ThreadEntry = fn() -> !;

/// Thread parameters
#[derive(Debug, Clone)]
pub struct ThreadParams {
    pub stack_size: usize,
    pub priority: Priority,
    pub detached: bool,
    pub inherit_priority: bool,
}

/// Thread control block (TCB)
/// 
/// Contains all information about a thread including its execution state,
/// CPU context, and scheduling information.
#[derive(Debug, Clone)]
pub struct ThreadControlBlock {
    /// Unique thread ID
    pub thread_id: ThreadId,
    /// Parent process ID
    pub process_id: usize,
    /// Thread name
    pub name: alloc::vec::Vec<u8>,
    /// Thread priority
    pub priority: Priority,
    /// Thread state
    pub state: ThreadState,
    /// Thread entry point
    pub entry_point: Option<ThreadEntry>,
    /// CPU context (architecture-specific)
    pub context: ThreadContext,
    /// Stack pointer
    pub stack_pointer: usize,
    /// Stack size
    pub stack_size: usize,
    /// Thread creation timestamp
    pub created_at: u64,
    /// Last scheduled timestamp
    pub last_scheduled: u64,
    /// CPU time used (in milliseconds)
    pub cpu_time: u64,
    /// Time slice used in current quantum
    pub time_slice_used: u32,
    /// Thread scheduling parameters
    pub sched_params: ThreadSchedParams,
    /// Thread-local storage pointer
    pub tls_pointer: usize,
    /// Thread flags
    pub flags: ThreadFlags,
}

/// CPU context for thread switching
/// 
/// This structure holds the register state needed for context switching.
/// In a real implementation, this would be architecture-specific.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ThreadContext {
    /// General purpose registers (architecture-dependent number)
    pub registers: [usize; 16],
    /// Program counter (RIP on x86_64)
    pub program_counter: usize,
    /// Stack pointer (RSP on x86_64)
    pub stack_pointer: usize,
    /// Flags register
    pub flags: usize,
    /// Control registers (CR0, CR3, CR4 on x86_64)
    pub control_registers: [usize; 3],
}

/// Thread scheduling parameters
#[derive(Debug, Clone, Copy)]
pub struct ThreadSchedParams {
    /// Time quantum (in ticks)
    pub time_quantum: u32,
    /// Wake-up time for sleeping threads
    pub wake_up_time: Option<u64>,
    /// Wait queue this thread is in (if any)
    pub wait_queue: Option<usize>,
    /// CPU affinity (bitmask of allowed CPUs)
    pub cpu_affinity: u32,
    /// Thread's last CPU
    pub last_cpu: usize,
}

/// Thread flags
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ThreadFlags: u32 {
        const DETACHED = 0b0000_0001;
        const DAEMON = 0b0000_0010;
        const SYSTEM_THREAD = 0b0000_0100;
        const CRITICAL = 0b0000_1000;
        const REALTIME = 0b0001_0000;
        const INTERRUPTIBLE = 0b0010_0000;
        const UNINTERRUPTIBLE = 0b0100_0000;
        const SUSPENDED = 0b1000_0000;
    }
}

/// Thread management result
pub type ThreadResult<T> = Result<T, ThreadError>;

/// Error types for thread operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadError {
    ThreadNotFound,
    InvalidThreadId,
    ThreadAlreadyExists,
    ThreadLimitExceeded,
    ThreadCreationFailed,
    InvalidPriority,
    AccessDenied,
    ThreadInInvalidState,
    ContextSwitchFailed,
    OutOfMemory,
    InvalidStackSize,
}

/// Thread Manager
/// 
/// Global manager for all threads in the system.
pub struct ThreadManager {
    /// Next available thread ID
    next_thread_id: AtomicUsize,
    /// Active threads map (thread_id -> TCB)
    threads: spin::Mutex<alloc::vec::Vec<Option<ThreadControlBlock>>>,
    /// Thread pool for reusable TCBs
    thread_pool: spin::Mutex<Vec<ThreadControlBlock>>,
    /// Global thread count
    thread_count: AtomicUsize,
}

impl ThreadManager {
    /// Create a new thread manager
    pub const fn new() -> Self {
        Self {
            next_thread_id: AtomicUsize::new(1),
            threads: spin::Mutex::new(alloc::vec::Vec::new()),
            thread_pool: spin::Mutex::new(Vec::new()),
            thread_count: AtomicUsize::new(0),
        }
    }

    /// Create a new thread
    pub fn create_thread(
        &self,
        process_id: usize,
        name: Vec<u8>,
        entry_point: Option<ThreadEntry>,
        params: ThreadParams,
    ) -> ThreadResult<ThreadHandle> {
        let mut threads = self.threads.lock();

        let thread_id = self.next_thread_id.fetch_add(1, Ordering::SeqCst);
        
        // Ensure we have enough space in the threads vector
        if thread_id >= threads.len() {
            threads.resize(thread_id + 1, None);
        }

        // Create the TCB
        let mut tcb = ThreadControlBlock {
            thread_id,
            process_id,
            name,
            priority: if params.inherit_priority {
                // Inherit from process - would need process lookup
                Priority::Normal
            } else {
                params.priority
            },
            state: ThreadState::Ready,
            entry_point,
            context: ThreadContext {
                registers: [0; 16],
                program_counter: 0,
                stack_pointer: 0,
                flags: 0,
                control_registers: [0; 3],
            },
            stack_pointer: 0,
            stack_size: params.stack_size,
            created_at: 0, // Would be set from kernel time
            last_scheduled: 0,
            cpu_time: 0,
            time_slice_used: 0,
            sched_params: ThreadSchedParams {
                time_quantum: match params.priority {
                    Priority::Idle => 5,
                    Priority::Low => 10,
                    Priority::Normal => 20,
                    Priority::High => 30,
                    Priority::Critical => 40,
                },
                wake_up_time: None,
                wait_queue: None,
                cpu_affinity: 0xFFFFFFFF, // All CPUs by default
                last_cpu: 0,
            },
            tls_pointer: 0,
            flags: ThreadFlags::empty(),
        };

        if params.detached {
            tcb.flags.insert(ThreadFlags::DETACHED);
        }

        // Store the thread
        let thread_handle = Arc::new(Mutex::new(tcb.clone()));
        threads[thread_id] = Some(tcb);
        
        // Update thread count
        self.thread_count.fetch_add(1, Ordering::SeqCst);

        Ok(thread_handle)
    }

    /// Get a thread by ID
    pub fn get_thread(&self, thread_id: ThreadId) -> ThreadResult<ThreadHandle> {
        let threads = self.threads.lock();
        
        if thread_id >= threads.len() {
            return Err(ThreadError::ThreadNotFound);
        }

        match &threads[thread_id] {
            Some(tcb) => Ok(Arc::new(Mutex::new(tcb.clone()))),
            None => Err(ThreadError::ThreadNotFound),
        }
    }

    /// Terminate a thread
    pub fn terminate_thread(&self, thread_id: ThreadId) -> ThreadResult<()> {
        let mut threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        if let Some(ref mut tcb) = threads[thread_id] {
            tcb.state = ThreadState::Terminated;
        }

        Ok(())
    }

    /// Get all thread IDs
    pub fn get_all_threads(&self) -> Vec<ThreadId> {
        let threads = self.threads.lock();
        let mut result = Vec::new();

        for (i, thread) in threads.iter().enumerate() {
            if thread.is_some() {
                result.push(i);
            }
        }

        result
    }

    /// Get threads by process ID
    pub fn get_threads_by_process(&self, process_id: usize) -> Vec<ThreadId> {
        let threads = self.threads.lock();
        let mut result = Vec::new();

        for thread in threads.iter() {
            if let Some(tcb) = thread {
                if tcb.process_id == process_id && matches!(tcb.state, ThreadState::Ready | ThreadState::Running) {
                    result.push(tcb.thread_id);
                }
            }
        }

        result
    }

    /// Get threads by priority
    pub fn get_threads_by_priority(&self, priority: Priority) -> Vec<ThreadId> {
        let threads = self.threads.lock();
        let mut result = Vec::new();

        for thread in threads.iter() {
            if let Some(tcb) = thread {
                if tcb.priority == priority && matches!(tcb.state, ThreadState::Ready | ThreadState::Running) {
                    result.push(tcb.thread_id);
                }
            }
        }

        result
    }

    /// Set thread priority
    pub fn set_thread_priority(&self, thread_id: ThreadId, priority: Priority) -> ThreadResult<()> {
        let mut threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        if let Some(ref mut tcb) = threads[thread_id] {
            tcb.priority = priority;
            
            // Update time quantum based on new priority
            tcb.sched_params.time_quantum = match priority {
                Priority::Idle => 5,
                Priority::Low => 10,
                Priority::Normal => 20,
                Priority::High => 30,
                Priority::Critical => 40,
            };
        }

        Ok(())
    }

    /// Wake up a sleeping thread
    pub fn wake_thread(&self, thread_id: ThreadId) -> ThreadResult<()> {
        let mut threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        if let Some(ref mut tcb) = threads[thread_id] {
            if matches!(tcb.state, ThreadState::Waiting) {
                tcb.state = ThreadState::Ready;
                tcb.sched_params.wake_up_time = None;
            }
        }

        Ok(())
    }

    /// Put a thread to sleep
    pub fn sleep_thread(&self, thread_id: ThreadId, duration_ms: u64) -> ThreadResult<()> {
        let mut threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        if let Some(ref mut tcb) = threads[thread_id] {
            tcb.state = ThreadState::Waiting;
            // Wake up time would be calculated from current time + duration
            // tcb.sched_params.wake_up_time = Some(current_time() + duration_ms);
        }

        Ok(())
    }

    /// Check if a thread is running
    pub fn is_thread_running(&self, thread_id: ThreadId) -> ThreadResult<bool> {
        let threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        let tcb = threads[thread_id].as_ref().unwrap();
        Ok(matches!(tcb.state, ThreadState::Ready | ThreadState::Running))
    }

    /// Get thread statistics
    pub fn get_thread_stats(&self, thread_id: ThreadId) -> ThreadResult<ThreadStats> {
        let threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        let tcb = threads[thread_id].as_ref().unwrap();
        
        Ok(ThreadStats {
            thread_id: tcb.thread_id,
            process_id: tcb.process_id,
            name: String::from_utf8_lossy(&tcb.name).to_string(),
            priority: tcb.priority,
            state: tcb.state,
            cpu_time: tcb.cpu_time,
            stack_size: tcb.stack_size,
            time_slice_used: tcb.time_slice_used,
            cpu_affinity: tcb.sched_params.cpu_affinity,
            uptime: 0, // Would calculate from creation time
        })
    }

    /// Get current thread count
    pub fn get_thread_count(&self) -> usize {
        self.thread_count.load(Ordering::SeqCst)
    }

    /// Check if thread can run on specified CPU
    pub fn can_run_on_cpu(&self, thread_id: ThreadId, cpu_id: usize) -> ThreadResult<bool> {
        let threads = self.threads.lock();
        
        if thread_id >= threads.len() || threads[thread_id].is_none() {
            return Err(ThreadError::ThreadNotFound);
        }

        let tcb = threads[thread_id].as_ref().unwrap();
        let cpu_mask: u32 = 1 << cpu_id;
        Ok(tcb.sched_params.cpu_affinity & cpu_mask != 0)
    }
}

/// Thread statistics structure
#[derive(Debug, Clone)]
pub struct ThreadStats {
    pub thread_id: ThreadId,
    pub process_id: usize,
    pub name: String,
    pub priority: Priority,
    pub state: ThreadState,
    pub cpu_time: u64,
    pub stack_size: usize,
    pub time_slice_used: u32,
    pub cpu_affinity: u32,
    pub uptime: u64,
}

/// Global thread manager instance
pub static THREAD_MANAGER: ThreadManager = ThreadManager::new();

/// Context switching utilities
pub struct ContextSwitch;

impl ContextSwitch {
    /// Initialize thread context for first execution
    pub fn init_context(entry_point: ThreadEntry, stack_pointer: usize) -> ThreadContext {
        ThreadContext {
            registers: [0; 16],
            program_counter: entry_point as usize,
            stack_pointer,
            flags: 0x200, // Enable interrupts flag on x86_64
            control_registers: [0; 3],
        }
    }

    /// Save current thread context
    /// 
    /// # Safety
    /// This function should only be called in interrupt context
    pub unsafe fn save_context(current_tcb: &mut ThreadControlBlock) {
        core::arch::asm!(
            // Save general purpose registers
            "push rax",
            "push rbx", 
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13", 
            "push r14",
            "push r15",
            // Save RSP
            "mov {}, rsp",
            // Save RFLAGS
            "pushfq",
            "pop rax",
            "mov {}, rax",
            :
            "=r"(current_tcb.context.stack_pointer),
            "=r"(current_tcb.context.flags)
            :
            :
            "volatile"
        );
    }

    /// Restore next thread context
    /// 
    /// # Safety
    /// This function should only be called in interrupt context
    pub unsafe fn restore_context(next_tcb: &ThreadControlBlock) -> ! {
        // Set RSP first
        core::arch::asm!("mov rsp, {}", in(reg) next_tcb.context.stack_pointer);
        
        // Restore RFLAGS
        core::arch::asm!(
            "mov rax, {}",
            "push rax",
            "popfq",
            :
            :
            "r"(next_tcb.context.flags)
        );

        // Restore general purpose registers
        core::arch::asm!(
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12", 
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            // Jump to entry point
            "jmp {}",
            :
            "r"(next_tcb.context.program_counter)
        );
    }

    /// Perform context switch between two threads
    /// 
    /// # Safety
    /// This function should only be called in interrupt context
    pub unsafe fn switch_context(current_tcb: &mut ThreadControlBlock, next_tcb: &ThreadControlBlock) -> ! {
        // Save current context
        Self::save_context(current_tcb);
        
        // Clear time slice for current thread
        current_tcb.time_slice_used = 0;
        
        // Set last scheduled time for next thread
        // next_tcb.last_scheduled = get_current_time();
        
        // Restore next context
        Self::restore_context(next_tcb);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_creation() {
        let manager = ThreadManager::new();
        
        let params = ThreadParams {
            stack_size: 4096,
            priority: Priority::Normal,
            detached: false,
            inherit_priority: false,
        };

        let result = manager.create_thread(1, b"test_thread".to_vec(), None, params);
        assert!(result.is_ok());
        
        let thread_handle = result.unwrap();
        let tcb = thread_handle.lock();
        assert_eq!(tcb.thread_id, 1);
        assert_eq!(tcb.priority, Priority::Normal);
    }

    #[test]
    fn test_thread_priority_ordering() {
        assert!(Priority::Idle < Priority::Low);
        assert!(Priority::Low < Priority::Normal);
        assert!(Priority::Normal < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn test_thread_flags() {
        let flags = ThreadFlags::DETACHED | ThreadFlags::DAEMON;
        assert!(flags.contains(ThreadFlags::DETACHED));
        assert!(flags.contains(ThreadFlags::DAEMON));
        assert!(!flags.contains(ThreadFlags::SYSTEM_THREAD));
    }

    #[test]
    fn test_thread_state() {
        let state = ThreadState::Ready;
        assert_ne!(state, ThreadState::Terminated);
    }

    #[test]
    fn test_thread_time_quantum() {
        let params_idle = ThreadParams {
            stack_size: 4096,
            priority: Priority::Idle,
            detached: false,
            inherit_priority: false,
        };
        
        assert_eq!(params_idle.priority as u32, 0);
    }
}