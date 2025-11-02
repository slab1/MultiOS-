//! Scheduling Algorithms for MultiOS
//! 
//! This module implements various scheduling algorithms including
//! priority-based and round-robin scheduling for multi-core systems.

use alloc::vec::Vec;
use spin::Mutex;
use bitflags::bitflags;
use core::sync::atomic::{AtomicUsize, AtomicU32, AtomicU64, Ordering};

use crate::{Priority, ThreadState, SchedulerError};
use crate::thread::{ThreadHandle, ThreadId, ThreadManager, ThreadControlBlock};
use crate::process::{ProcessManager, ProcessId};

/// CPU ID type
pub type CpuId = usize;

/// Number of available CPUs
const MAX_CPUS: usize = 32;

/// CPU affinity mask
pub type CpuAffinity = u32;

/// Scheduling algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingAlgorithm {
    RoundRobin,
    PriorityBased,
    MultiLevelFeedbackQueue,
    EarliestDeadlineFirst,
}

/// CPU state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    Online,
    Offline,
    Suspended,
}

/// Per-CPU scheduler state
#[derive(Debug)]
struct CpuScheduler {
    /// CPU ID
    pub cpu_id: CpuId,
    /// CPU state
    pub state: CpuState,
    /// Currently running thread
    pub current_thread: Option<ThreadId>,
    /// Ready queue for this CPU
    pub ready_queue: ReadyQueue,
    /// CPU load (number of ready threads)
    pub load: u32,
    /// Last scheduling decision time
    pub last_scheduled: u64,
}

/// Ready queue for threads
#[derive(Debug)]
struct ReadyQueue {
    /// FIFO queues for each priority level
    priority_queues: Vec<Vec<ThreadId>>,
    /// Current time quantum counter
    time_quantum_counter: u32,
    /// Current priority being scheduled
    current_priority: Priority,
    /// Round-robin index
    rr_index: usize,
}

impl ReadyQueue {
    fn new() -> Self {
        let mut priority_queues = Vec::new();
        for _ in 0..5 { // 5 priority levels
            priority_queues.push(Vec::new());
        }

        Self {
            priority_queues,
            time_quantum_counter: 0,
            current_priority: Priority::Normal,
            rr_index: 0,
        }
    }

    /// Add a thread to the ready queue
    fn add_thread(&mut self, thread_id: ThreadId, priority: Priority, algorithm: SchedulingAlgorithm) {
        let priority_idx = priority as usize;
        if priority_idx < self.priority_queues.len() {
            self.priority_queues[priority_idx].push(thread_id);
        }
    }

    /// Remove a thread from the ready queue
    fn remove_thread(&mut self, thread_id: ThreadId) -> bool {
        for queue in &mut self.priority_queues {
            if let Some(pos) = queue.iter().position(|&id| id == thread_id) {
                queue.remove(pos);
                return true;
            }
        }
        false
    }

    /// Get the next thread to schedule based on the algorithm
    fn get_next_thread(&mut self, algorithm: SchedulingAlgorithm) -> Option<ThreadId> {
        match algorithm {
            SchedulingAlgorithm::RoundRobin => self.get_next_round_robin(),
            SchedulingAlgorithm::PriorityBased => self.get_next_priority(),
            SchedulingAlgorithm::MultiLevelFeedbackQueue => self.get_next_mlfq(),
            SchedulingAlgorithm::EarliestDeadlineFirst => self.get_next_edf(),
        }
    }

    /// Round-robin scheduling
    fn get_next_round_robin(&mut self) -> Option<ThreadId> {
        // First, try to find a thread at the current priority level
        let start_priority = self.current_priority as usize;
        
        for priority_offset in 0..self.priority_queues.len() {
            let priority_idx = (start_priority + priority_offset) % self.priority_queues.len();
            let queue = &mut self.priority_queues[priority_idx];
            
            if !queue.is_empty() {
                // Round-robin within the priority level
                let thread_id = queue.remove(self.rr_index % queue.len());
                self.rr_index = (self.rr_index + 1) % queue.len();
                self.current_priority = unsafe { core::mem::transmute(priority_idx as u8) };
                return Some(thread_id);
            }
        }
        
        None
    }

    /// Priority-based scheduling
    fn get_next_priority(&mut self) -> Option<ThreadId> {
        // Always prefer higher priority threads
        for priority_idx in (0..self.priority_queues.len()).rev() {
            let queue = &mut self.priority_queues[priority_idx];
            
            if !queue.is_empty() {
                // FIFO within priority level
                let thread_id = queue.remove(0);
                self.current_priority = unsafe { core::mem::transmute(priority_idx as u8) };
                return Some(thread_id);
            }
        }
        
        None
    }

    /// Multi-level feedback queue scheduling
    fn get_next_mlfq(&mut self) -> Option<ThreadId> {
        self.time_quantum_counter += 1;
        
        // For MLFQ, we implement aging and priority adjustment
        // For now, use a simplified version similar to priority-based
        self.get_next_priority()
    }

    /// Earliest deadline first scheduling
    fn get_next_edf(&mut self) -> Option<ThreadId> {
        // EDF would require deadline information in TCB
        // For now, fall back to priority-based
        self.get_next_priority()
    }

    /// Check if the ready queue is empty
    fn is_empty(&self) -> bool {
        self.priority_queues.iter().all(|queue| queue.is_empty())
    }

    /// Get total number of threads in ready queue
    fn len(&self) -> usize {
        self.priority_queues.iter().map(|queue| queue.len()).sum()
    }
}

/// Multi-core scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Scheduling algorithm to use
    pub algorithm: SchedulingAlgorithm,
    /// Number of CPUs available
    pub cpu_count: usize,
    /// Default time quantum for threads
    pub default_time_quantum: u32,
    /// CPU load balancing interval (in milliseconds)
    pub load_balance_interval: u64,
    /// Enable CPU affinity
    pub enable_cpu_affinity: bool,
    /// Enable automatic load balancing
    pub enable_load_balancing: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            algorithm: SchedulingAlgorithm::RoundRobin,
            cpu_count: 4, // Assume 4 CPUs by default
            default_time_quantum: 20,
            load_balance_interval: 100,
            enable_cpu_affinity: true,
            enable_load_balancing: true,
        }
    }
}

/// Main scheduler structure
pub struct Scheduler {
    /// Scheduler configuration
    config: SchedulerConfig,
    /// Thread manager reference
    thread_manager: &'static ThreadManager,
    /// Process manager reference  
    process_manager: &'static ProcessManager,
    /// Per-CPU scheduler state
    cpu_schedulers: Vec<Mutex<CpuScheduler>>,
    /// Global ready queue for load balancing
    global_ready_queue: Mutex<ReadyQueue>,
    /// Scheduler statistics
    stats: SchedulerStats,
}

/// Scheduler statistics
#[derive(Debug, Default)]
pub struct SchedulerStats {
    /// Total context switches
    pub context_switches: AtomicU64,
    /// Total threads scheduled
    pub threads_scheduled: AtomicU64,
    /// CPU utilization percentages
    pub cpu_utilization: [AtomicU32; MAX_CPUS],
    /// Scheduling latency (nanoseconds)
    pub scheduling_latency: AtomicU64,
    /// Load balancing operations
    pub load_balances: AtomicU64,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        let thread_manager = &crate::thread::THREAD_MANAGER;
        let process_manager = &crate::process::PROCESS_MANAGER;
        
        let cpu_count = core::cmp::min(MAX_CPUS, 4); // Assume 4 CPUs for now
        let mut cpu_schedulers = Vec::with_capacity(cpu_count);
        
        for cpu_id in 0..cpu_count {
            cpu_schedulers.push(Mutex::new(CpuScheduler {
                cpu_id,
                state: CpuState::Online,
                current_thread: None,
                ready_queue: ReadyQueue::new(),
                load: 0,
                last_scheduled: 0,
            }));
        }

        Self {
            config: SchedulerConfig {
                algorithm: SchedulingAlgorithm::RoundRobin,
                cpu_count,
                default_time_quantum: 20,
                load_balance_interval: 100,
                enable_cpu_affinity: true,
                enable_load_balancing: true,
            },
            thread_manager,
            process_manager,
            cpu_schedulers,
            global_ready_queue: Mutex::new(ReadyQueue::new()),
            stats: SchedulerStats::default(),
        }
    }

    /// Initialize scheduler with configuration
    pub fn with_config(config: SchedulerConfig) -> Self {
        let mut scheduler = Self::new();
        scheduler.config = config;
        scheduler
    }

    /// Add a thread to the scheduler
    pub fn add_thread(&self, thread_handle: ThreadHandle) -> Result<(), SchedulerError> {
        let tcb = thread_handle.lock();
        let thread_id = tcb.thread_id;
        let priority = tcb.priority;
        drop(tcb);

        // Determine which CPU to add this thread to
        let cpu_id = self.select_cpu_for_thread(thread_id, priority);
        
        // Add to the appropriate CPU's ready queue
        {
            let mut cpu_scheduler = self.cpu_schedulers[cpu_id].lock();
            cpu_scheduler.ready_queue.add_thread(thread_id, priority, self.config.algorithm);
            cpu_scheduler.load += 1;
        }

        self.stats.threads_scheduled.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Remove a thread from the scheduler
    pub fn remove_thread(&self, thread_id: ThreadId, cpu_id: Option<CpuId>) -> Result<(), SchedulerError> {
        let target_cpu = if let Some(cpu) = cpu_id {
            cpu
        } else {
            // Search all CPUs for the thread
            self.find_thread_cpu(thread_id)?
        };

        {
            let mut cpu_scheduler = self.cpu_schedulers[target_cpu].lock();
            if cpu_scheduler.ready_queue.remove_thread(thread_id) {
                cpu_scheduler.load = cpu_scheduler.load.saturating_sub(1);
            }
            
            // If this was the current thread, clear it
            if cpu_scheduler.current_thread == Some(thread_id) {
                cpu_scheduler.current_thread = None;
            }
        }

        Ok(())
    }

    /// Schedule the next thread for a specific CPU
    pub fn schedule_next(&self, cpu_id: CpuId) -> Result<ThreadHandle, SchedulerError> {
        let mut cpu_scheduler = self.cpu_schedulers[cpu_id].lock();
        
        // If there's already a current thread, put it back in the ready queue
        if let Some(current_thread_id) = cpu_scheduler.current_thread {
            if let Ok(thread_handle) = self.thread_manager.get_thread(current_thread_id) {
                let mut tcb = thread_handle.lock();
                tcb.state = ThreadState::Ready;
                // Reset time slice
                tcb.time_slice_used = 0;
                // Add back to ready queue
                cpu_scheduler.ready_queue.add_thread(tcb.thread_id, tcb.priority, self.config.algorithm);
            }
        }

        // Get next thread from ready queue
        let next_thread_id = if let Some(thread_id) = cpu_scheduler.ready_queue.get_next_thread(self.config.algorithm) {
            thread_id
        } else {
            // No ready threads, return idle thread
            return Err(SchedulerError::NoRunnableThreads);
        };

        // Set as current thread
        cpu_scheduler.current_thread = Some(next_thread_id);
        cpu_scheduler.last_scheduled = 0; // Would be set from current time

        // Mark thread as running
        if let Ok(thread_handle) = self.thread_manager.get_thread(next_thread_id) {
            let mut tcb = thread_handle.lock();
            tcb.state = ThreadState::Running;
            tcb.last_scheduled = cpu_scheduler.last_scheduled;
        }

        self.stats.context_switches.fetch_add(1, Ordering::SeqCst);
        self.thread_manager.get_thread(next_thread_id)
            .map_err(|_| SchedulerError::NoRunnableThreads)
    }

    /// Get the current thread running on a CPU
    pub fn get_current_thread(&self, cpu_id: CpuId) -> Option<ThreadId> {
        let cpu_scheduler = self.cpu_schedulers[cpu_id].lock();
        cpu_scheduler.current_thread
    }

    /// Perform load balancing between CPUs
    pub fn balance_load(&self) -> Result<(), SchedulerError> {
        if !self.config.enable_load_balancing {
            return Ok(());
        }

        // Find CPUs with high and low loads
        let mut overloaded_cpus = Vec::new();
        let mut underloaded_cpus = Vec::new();

        for cpu_scheduler in &self.cpu_schedulers {
            let cpu = cpu_scheduler.lock();
            if cpu.state != CpuState::Online {
                continue;
            }

            if cpu.load > 10 { // Threshold for overload
                overloaded_cpus.push((cpu.cpu_id, cpu.load));
            } else if cpu.load < 3 && !cpu.ready_queue.is_empty() { // Threshold for underload
                underloaded_cpus.push((cpu.cpu_id, cpu.load));
            }
        }

        // Move threads from overloaded to underloaded CPUs
        for (overloaded_cpu, _) in overloaded_cpus {
            for (underloaded_cpu, _) in &underloaded_cpus {
                if overloaded_cpu == *underloaded_cpu {
                    continue;
                }

                // Transfer one thread
                let thread_id = {
                    let mut overloaded = self.cpu_schedulers[overloaded_cpu].lock();
                    if let Some(thread_id) = overloaded.ready_queue.priority_queues[0].pop() {
                        overloaded.load = overloaded.load.saturating_sub(1);
                        Some(thread_id)
                    } else {
                        None
                    }
                };

                if let Some(thread_id) = thread_id {
                    {
                        let mut underloaded = self.cpu_schedulers[*underloaded_cpu].lock();
                        underloaded.ready_queue.add_thread(thread_id, Priority::Normal, self.config.algorithm);
                        underloaded.load += 1;
                    }

                    // Update thread's CPU affinity if needed
                    // thread_manager.update_thread_cpu_affinity(thread_id, *underloaded_cpu)?;
                    break;
                }
            }
        }

        self.stats.load_balances.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    /// Select the best CPU for a thread
    fn select_cpu_for_thread(&self, thread_id: ThreadId, priority: Priority) -> CpuId {
        if !self.config.enable_cpu_affinity {
            // Simple load balancing
            let mut min_load = u32::MAX;
            let mut best_cpu = 0;

            for cpu_scheduler in &self.cpu_schedulers {
                let cpu = cpu_scheduler.lock();
                if cpu.state != CpuState::Online {
                    continue;
                }

                if cpu.load < min_load {
                    min_load = cpu.load;
                    best_cpu = cpu.cpu_id;
                }
            }

            return best_cpu;
        }

        // Check thread's CPU affinity
        if let Ok(thread_handle) = self.thread_manager.get_thread(thread_id) {
            let tcb = thread_handle.lock();
            let affinity = tcb.sched_params.cpu_affinity;
            drop(tcb);

            // Find the least loaded CPU within affinity
            let mut min_load = u32::MAX;
            let mut best_cpu = 0;

            for cpu_id in 0..self.config.cpu_count {
                let cpu_mask: u32 = 1 << cpu_id;
                if affinity & cpu_mask != 0 {
                    let cpu = self.cpu_schedulers[cpu_id].lock();
                    if cpu.state == CpuState::Online && cpu.load < min_load {
                        min_load = cpu.load;
                        best_cpu = cpu_id;
                    }
                }
            }

            return best_cpu;
        }

        0 // Fallback
    }

    /// Find which CPU a thread is currently on
    fn find_thread_cpu(&self, thread_id: ThreadId) -> Result<CpuId, SchedulerError> {
        for cpu_scheduler in &self.cpu_schedulers {
            let cpu = cpu_scheduler.lock();
            if cpu.current_thread == Some(thread_id) {
                return Ok(cpu.cpu_id);
            }
            
            // Check if thread is in ready queue
            for queue in &cpu.ready_queue.priority_queues {
                if queue.contains(&thread_id) {
                    return Ok(cpu.cpu_id);
                }
            }
        }

        Err(SchedulerError::ThreadNotFound)
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStatsSnapshot {
        let mut cpu_utilization = [0u32; MAX_CPUS];
        for (i, cpu_util) in self.stats.cpu_utilization.iter().enumerate().take(self.config.cpu_count) {
            cpu_utilization[i] = cpu_util.load(Ordering::SeqCst);
        }

        SchedulerStatsSnapshot {
            context_switches: self.stats.context_switches.load(Ordering::SeqCst),
            threads_scheduled: self.stats.threads_scheduled.load(Ordering::SeqCst),
            cpu_utilization,
            scheduling_latency: self.stats.scheduling_latency.load(Ordering::SeqCst),
            load_balances: self.stats.load_balances.load(Ordering::SeqCst),
            algorithm: self.config.algorithm,
            cpu_count: self.config.cpu_count,
        }
    }

    /// Get the number of CPUs
    pub fn get_cpu_count(&self) -> usize {
        self.config.cpu_count
    }

    /// Check if a CPU is online
    pub fn is_cpu_online(&self, cpu_id: CpuId) -> bool {
        if cpu_id >= self.config.cpu_count {
            return false;
        }
        
        let cpu_scheduler = self.cpu_schedulers[cpu_id].lock();
        cpu_scheduler.state == CpuState::Online
    }

    /// Take a CPU offline
    pub fn take_cpu_offline(&self, cpu_id: CpuId) -> Result<(), SchedulerError> {
        if cpu_id >= self.config.cpu_count {
            return Err(SchedulerError::InvalidThreadId);
        }

        let mut cpu_scheduler = self.cpu_schedulers[cpu_id].lock();
        cpu_scheduler.state = CpuState::Offline;
        
        // Migrate threads from this CPU to others
        if let Some(current_thread) = cpu_scheduler.current_thread.take() {
            // Migrate current thread
            let target_cpu = self.select_cpu_for_thread(current_thread, Priority::Normal);
            {
                let target_scheduler = self.cpu_schedulers[target_cpu].lock();
                // target_scheduler.ready_queue.add_thread(current_thread, Priority::Normal, self.config.algorithm);
            }
        }

        // Migrate ready queue threads
        while let Some(thread_id) = cpu_scheduler.ready_queue.get_next_thread(self.config.algorithm) {
            let target_cpu = self.select_cpu_for_thread(thread_id, Priority::Normal);
            {
                let target_scheduler = self.cpu_schedulers[target_cpu].lock();
                // target_scheduler.ready_queue.add_thread(thread_id, Priority::Normal, self.config.algorithm);
            }
        }

        Ok(())
    }

    /// Bring a CPU back online
    pub fn bring_cpu_online(&self, cpu_id: CpuId) -> Result<(), SchedulerError> {
        if cpu_id >= self.config.cpu_count {
            return Err(SchedulerError::InvalidThreadId);
        }

        let mut cpu_scheduler = self.cpu_schedulers[cpu_id].lock();
        cpu_scheduler.state = CpuState::Online;
        Ok(())
    }
}

/// Scheduler statistics snapshot
#[derive(Debug, Clone)]
pub struct SchedulerStatsSnapshot {
    pub context_switches: u64,
    pub threads_scheduled: u64,
    pub cpu_utilization: [u32; MAX_CPUS],
    pub scheduling_latency: u64,
    pub load_balances: u64,
    pub algorithm: SchedulingAlgorithm,
    pub cpu_count: usize,
}

/// Helper functions for scheduler operations
pub struct SchedulerHelpers;

impl SchedulerHelpers {
    /// Calculate time quantum based on priority
    pub fn calculate_time_quantum(priority: Priority, algorithm: SchedulingAlgorithm) -> u32 {
        match algorithm {
            SchedulingAlgorithm::RoundRobin => match priority {
                Priority::Idle => 5,
                Priority::Low => 10,
                Priority::Normal => 20,
                Priority::High => 30,
                Priority::Critical => 40,
            },
            SchedulingAlgorithm::PriorityBased => match priority {
                Priority::Idle => 10,
                Priority::Low => 15,
                Priority::Normal => 20,
                Priority::High => 25,
                Priority::Critical => 30,
            },
            _ => 20, // Default
        }
    }

    /// Check if thread should be preempted
    pub fn should_preempt(current_thread: &ThreadControlBlock, new_thread: &ThreadControlBlock, algorithm: SchedulingAlgorithm) -> bool {
        match algorithm {
            SchedulingAlgorithm::PriorityBased => {
                // Preempt if new thread has higher priority
                new_thread.priority > current_thread.priority
            },
            SchedulingAlgorithm::RoundRobin => {
                // Preempt if time quantum exhausted
                current_thread.time_slice_used >= current_thread.sched_params.time_quantum
            },
            SchedulingAlgorithm::MultiLevelFeedbackQueue => {
                // Complex MLFQ logic would go here
                new_thread.priority > current_thread.priority
            },
            SchedulingAlgorithm::EarliestDeadlineFirst => {
                // Would compare deadlines here
                false
            },
        }
    }

    /// Age threads to prevent starvation
    pub fn age_threads(threads: &[ThreadId], thread_manager: &ThreadManager) {
        for &thread_id in threads {
            if let Ok(thread_handle) = thread_manager.get_thread(thread_id) {
                let mut tcb = thread_handle.lock();
                // Increase priority slightly to prevent starvation
                if tcb.priority > Priority::Idle {
                    let new_priority = match tcb.priority {
                        Priority::High => Priority::Critical,
                        Priority::Normal => Priority::High,
                        Priority::Low => Priority::Normal,
                        Priority::Idle => Priority::Low,
                        Priority::Critical => Priority::Critical,
                    };
                    tcb.priority = new_priority;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ready_queue_operations() {
        let mut ready_queue = ReadyQueue::new();
        
        // Add threads
        ready_queue.add_thread(1, Priority::Normal, SchedulingAlgorithm::RoundRobin);
        ready_queue.add_thread(2, Priority::High, SchedulingAlgorithm::RoundRobin);
        ready_queue.add_thread(3, Priority::Low, SchedulingAlgorithm::RoundRobin);
        
        assert_eq!(ready_queue.len(), 3);
        
        // Get next thread (should be highest priority)
        let next_thread = ready_queue.get_next_thread(SchedulingAlgorithm::RoundRobin);
        assert_eq!(next_thread, Some(2)); // High priority thread
        
        // Remove thread
        let removed = ready_queue.remove_thread(2);
        assert!(removed);
        assert_eq!(ready_queue.len(), 2);
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        assert_eq!(scheduler.get_cpu_count(), 4);
    }

    #[test]
    fn test_time_quantum_calculation() {
        assert_eq!(SchedulerHelpers::calculate_time_quantum(Priority::Idle, SchedulingAlgorithm::RoundRobin), 5);
        assert_eq!(SchedulerHelpers::calculate_time_quantum(Priority::Critical, SchedulingAlgorithm::RoundRobin), 40);
    }

    #[test]
    fn test_priority_preemption() {
        let current = ThreadControlBlock {
            thread_id: 1,
            process_id: 1,
            name: b"current".to_vec(),
            priority: Priority::Normal,
            state: ThreadState::Running,
            entry_point: None,
            context: super::thread::ThreadContext {
                registers: [0; 16],
                program_counter: 0,
                stack_pointer: 0,
                flags: 0,
                control_registers: [0; 3],
            },
            stack_pointer: 0,
            stack_size: 4096,
            created_at: 0,
            last_scheduled: 0,
            cpu_time: 0,
            time_slice_used: 0,
            sched_params: super::thread::ThreadSchedParams {
                time_quantum: 20,
                wake_up_time: None,
                wait_queue: None,
                cpu_affinity: 0xFFFFFFFF,
                last_cpu: 0,
            },
            tls_pointer: 0,
            flags: super::thread::ThreadFlags::empty(),
        };

        let new_thread = ThreadControlBlock {
            thread_id: 2,
            process_id: 1,
            name: b"new".to_vec(),
            priority: Priority::High,
            state: ThreadState::Ready,
            entry_point: None,
            context: current.context,
            stack_pointer: 0,
            stack_size: 4096,
            created_at: 0,
            last_scheduled: 0,
            cpu_time: 0,
            time_slice_used: 0,
            sched_params: current.sched_params,
            tls_pointer: 0,
            flags: super::thread::ThreadFlags::empty(),
        };

        assert!(SchedulerHelpers::should_preempt(&current, &new_thread, SchedulingAlgorithm::PriorityBased));
    }
}