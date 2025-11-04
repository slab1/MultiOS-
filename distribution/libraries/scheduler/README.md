# MultiOS Process and Thread Management System

This document describes the comprehensive process and thread management system implemented for MultiOS, a hybrid microkernel operating system.

## Overview

The scheduler library provides a complete solution for process and thread management in a multi-core environment, featuring:

- **Process Control Blocks (PCBs)** with comprehensive process state tracking
- **Thread Control Blocks (TCBs)** with CPU context management
- **Multiple scheduling algorithms** (Round-Robin, Priority-based, MLFQ, EDF)
- **Multi-core scheduling** with load balancing and CPU affinity
- **Safe Rust abstractions** with proper synchronization primitives
- **Context switching** support for architecture-specific CPU state management

## Architecture

### Core Components

1. **Process Manager** (`process.rs`)
   - Manages all processes in the system
   - Handles process creation, termination, and state management
   - Maintains process tree relationships
   - Provides process statistics and memory tracking

2. **Thread Manager** (`thread.rs`)
   - Manages all threads belonging to processes
   - Handles thread creation, termination, and scheduling
   - Implements CPU context switching
   - Provides thread synchronization and affinity

3. **Scheduler Algorithms** (`scheduler_algo.rs`)
   - Implements multiple scheduling policies
   - Handles multi-core load balancing
   - Provides CPU affinity and real-time scheduling support
   - Maintains scheduler statistics

4. **Global API** (`lib.rs`)
   - High-level system call interface
   - Initialization and configuration
   - Cross-component integration

## Process Management

### Process Control Block (PCB)

```rust
pub struct ProcessControlBlock {
    pub process_id: ProcessId,
    pub parent_id: Option<ProcessId>,
    pub name: Vec<u8>,
    pub priority: ProcessPriority,
    pub state: ProcessState,
    pub flags: ProcessFlags,
    pub threads: Vec<ThreadHandle>,
    pub main_thread: Option<ThreadId>,
    pub created_at: u64,
    pub cpu_time: u64,
    pub memory_stats: ProcessMemoryStats,
    pub exit_status: Option<i32>,
}
```

### Process Priorities

- **System (0)**: Critical system processes (kernel daemons)
- **High (1)**: Important user processes
- **Normal (2)**: Regular user processes
- **Low (3)**: Background tasks
- **Idle (4)**: Low-priority tasks

### Process States

- **Running**: Process is currently executing
- **Waiting**: Process is blocked waiting for I/O or events
- **Stopped**: Process is suspended
- **Terminated**: Process has completed execution

### Process Creation

```rust
let params = ProcessCreateParams {
    name: b"my_process".to_vec(),
    priority: ProcessPriority::Normal,
    flags: ProcessFlags::empty(),
    entry_point: Some(main_function),
    thread_params: None,
};

let process_id = syscall_create_process(params)?;
```

## Thread Management

### Thread Control Block (TCB)

```rust
pub struct ThreadControlBlock {
    pub thread_id: ThreadId,
    pub process_id: usize,
    pub name: Vec<u8>,
    pub priority: Priority,
    pub state: ThreadState,
    pub entry_point: Option<ThreadEntry>,
    pub context: ThreadContext,
    pub stack_pointer: usize,
    pub stack_size: usize,
    pub sched_params: ThreadSchedParams,
    pub flags: ThreadFlags,
}
```

### Thread Context

The `ThreadContext` structure holds CPU register state for context switching:

```rust
#[repr(C)]
pub struct ThreadContext {
    pub registers: [usize; 16],
    pub program_counter: usize,
    pub stack_pointer: usize,
    pub flags: usize,
    pub control_registers: [usize; 3],
}
```

### Thread Creation

```rust
let thread_params = ThreadParams {
    stack_size: 4096,
    priority: Priority::Normal,
    detached: false,
    inherit_priority: false,
};

let thread_handle = syscall_create_thread(
    process_id,
    b"worker_thread".to_vec(),
    Some(worker_function),
    thread_params
)?;
```

### Context Switching

Context switching is implemented in a safe, architecture-specific manner:

```rust
unsafe fn switch_context(
    current_tcb: &mut ThreadControlBlock, 
    next_tcb: &ThreadControlBlock
) -> ! {
    ContextSwitch::save_context(current_tcb);
    ContextSwitch::restore_context(next_tcb);
}
```

## Scheduling Algorithms

### 1. Round-Robin Scheduling

- **Principle**: Each thread gets equal time slices (time quantums)
- **Use case**: General-purpose time-sharing systems
- **Priority handling**: Higher priority threads get longer time quantums

```rust
// Time quantum based on priority
match priority {
    Priority::Idle => 5,
    Priority::Low => 10,
    Priority::Normal => 20,
    Priority::High => 30,
    Priority::Critical => 40,
}
```

### 2. Priority-Based Scheduling

- **Principle**: Always schedule the highest priority ready thread
- **Use case**: Real-time and critical systems
- **Starvation prevention**: Implemented through thread aging

```rust
fn get_next_priority(&mut self) -> Option<ThreadId> {
    for priority_idx in (0..self.priority_queues.len()).rev() {
        let queue = &mut self.priority_queues[priority_idx];
        if !queue.is_empty() {
            return Some(queue.remove(0));
        }
    }
    None
}
```

### 3. Multi-Level Feedback Queue (MLFQ)

- **Principle**: Dynamically adjusts thread priorities based on behavior
- **Features**: 
  - CPU-bound threads get lower priority
  - I/O-bound threads maintain higher priority
  - Aging prevents starvation

### 4. Earliest Deadline First (EDF)

- **Principle**: Schedule thread with earliest deadline
- **Use case**: Real-time systems with timing constraints
- **Implementation**: Requires deadline tracking in TCB

## Multi-Core Scheduling

### Per-CPU Scheduler State

Each CPU maintains its own scheduler state:

```rust
struct CpuScheduler {
    pub cpu_id: CpuId,
    pub state: CpuState,
    pub current_thread: Option<ThreadId>,
    pub ready_queue: ReadyQueue,
    pub load: u32,
    pub last_scheduled: u64,
}
```

### CPU Affinity

Threads can be constrained to specific CPUs:

```rust
pub fn set_thread_cpu_affinity(thread_id: ThreadId, affinity: CpuAffinity) -> Result<()> {
    let cpu_mask: u32 = 0b1111; // Allow CPUs 0-3
    THREAD_MANAGER.get_thread(thread_id).map(|thread_handle| {
        let mut tcb = thread_handle.lock();
        tcb.sched_params.cpu_affinity = cpu_mask;
    })
}
```

### Load Balancing

The scheduler automatically balances load across CPUs:

```rust
pub fn balance_load(&self) -> Result<(), SchedulerError> {
    // Find overloaded and underloaded CPUs
    // Migrate threads from overloaded to underloaded CPUs
    // Update thread CPU affinity
}
```

## Synchronization

### Safe Rust Abstractions

All shared data structures use safe synchronization:

- **Spinlocks**: For short-duration critical sections
- **Arc<Mutex<T>>**: For shared thread/process handles
- **Atomic operations**: For counters and flags

### Thread Safety

```rust
// Safe concurrent access to thread handles
pub type ThreadHandle = Arc<Mutex<ThreadControlBlock>>;

// Safe access to global manager
pub static THREAD_MANAGER: ThreadManager = ThreadManager::new();
```

## System Call Interface

### Process System Calls

```rust
// Create a new process
let process_id = syscall_create_process(params)?;

// Terminate a process
syscall_terminate_process(process_id, exit_status)?;

// Set process priority
PROCESS_MANAGER.set_process_priority(process_id, priority)?;
```

### Thread System Calls

```rust
// Create a new thread
let thread_handle = syscall_create_thread(process_id, name, entry_point, params)?;

// Terminate a thread
syscall_terminate_thread(thread_id)?;

// Set thread priority
syscall_set_thread_priority(thread_id, Priority::High)?;

// Sleep and wake threads
syscall_sleep_thread(thread_id, 1000)?; // Sleep for 1 second
syscall_wake_thread(thread_id)?;
```

## Initialization

### Basic Initialization

```rust
use multios_scheduler::{init, get_cpu_count, is_system_ready};

// Initialize with default configuration
init()?;

// Check if system is ready
if is_system_ready() {
    println!("Scheduler initialized with {} CPUs", get_cpu_count());
}
```

### Custom Configuration

```rust
use multios_scheduler::{init_with_config, SchedulerConfig, SchedulingAlgorithm};

let config = SchedulerConfig {
    algorithm: SchedulingAlgorithm::PriorityBased,
    cpu_count: 8,
    default_time_quantum: 25,
    load_balance_interval: 50,
    enable_cpu_affinity: true,
    enable_load_balancing: true,
};

init_with_config(config)?;
```

## Performance Monitoring

### Scheduler Statistics

```rust
let stats = get_scheduler_stats();
println!("Context switches: {}", stats.context_switches);
println!("Threads scheduled: {}", stats.threads_scheduled);
println!("Load balancing operations: {}", stats.load_balances);
```

### Thread Statistics

```rust
let thread_stats = THREAD_MANAGER.get_thread_stats(thread_id)?;
println!("Thread '{}' on CPU {}: {}ms CPU time", 
    thread_stats.name, 
    thread_stats.process_id, 
    thread_stats.cpu_time
);
```

## Memory Management

### Process Memory Statistics

```rust
pub struct ProcessMemoryStats {
    pub total_memory: usize,
    pub shared_memory: usize,
    pub code_size: usize,
    pub data_size: usize,
    pub stack_size: usize,
}
```

The memory statistics are tracked per process to enable:
- Memory usage monitoring
- Resource accounting
- Memory leak detection
- Process memory limits

## Error Handling

### Comprehensive Error Types

```rust
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
```

### Safe Error Propagation

All errors use Rust's `Result<T, E>` type for safe error handling:

```rust
pub type ProcessResult<T> = Result<T, ProcessError>;
pub type ThreadResult<T> = Result<T, ThreadError>;
pub type SchedulerResult<T> = Result<T, SchedulerError>;
```

## Testing

The scheduler includes comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_process_creation() {
        let manager = ProcessManager::new();
        let params = ProcessCreateParams { /* ... */ };
        let result = manager.create_process(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_thread_priority_ordering() {
        assert!(Priority::Idle < Priority::Low);
        assert!(Priority::Normal < Priority::High);
    }
}
```

## Architecture-Specific Implementation

### x86_64 Context Switching

The context switching implementation is optimized for x86_64:

```rust
unsafe fn save_context(tcb: &mut ThreadControlBlock) {
    core::arch::asm!(
        "push rax; push rbx; push rcx; push rdx",
        "push rsi; push rdi; push rbp; push r8",
        // ... more registers
        "mov {}, rsp",
        "=r"(tcb.context.stack_pointer)
    );
}
```

### Future Architecture Support

The design supports easy extension to other architectures:
- **ARM64**: Update register list and calling convention
- **RISC-V**: Modify register names and ABI
- **Custom ISAs**: Add architecture-specific context structure

## Integration with Kernel

### Kernel Integration Points

```rust
// In kernel initialization
use multios_scheduler::{init, schedule_next};

pub fn kernel_main(arch: ArchType, boot_info: &BootInfo) -> KernelResult<()> {
    // ... other initialization ...
    
    // Initialize scheduler
    init()?;
    
    // Start main loop
    loop {
        let next_thread = schedule_next()?;
        // Context switch to next thread
    }
}
```

### Interrupt Handling

```rust
#[no_mangle]
pub extern "C" fn timer_interrupt_handler() {
    // Trigger scheduler
    schedule_next();
}
```

## Best Practices

### Thread Creation

1. **Always specify stack size**: Prevent stack overflow
2. **Use appropriate priorities**: Match thread importance
3. **Set CPU affinity when needed**: For cache locality
4. **Clean up terminated threads**: Prevent resource leaks

### Process Management

1. **Use process limits**: Prevent fork bombs
2. **Monitor resource usage**: Track memory and CPU
3. **Handle process termination**: Clean up child processes
4. **Use appropriate flags**: Set process permissions

### Performance Optimization

1. **Enable load balancing**: For multi-core systems
2. **Use CPU affinity**: For cache-sensitive workloads
3. **Monitor scheduler statistics**: Identify bottlenecks
4. **Choose appropriate algorithm**: Match workload characteristics

## Future Enhancements

### Planned Features

1. **Real-time scheduling**: EDF and rate-monotonic scheduling
2. **Energy-aware scheduling**: CPU frequency scaling integration
3. **NUMA support**: Non-uniform memory access optimization
4. **Security hardening**: Process isolation and capability-based security
5. **Distributed scheduling**: Across multiple systems

### Architecture Extensions

1. **ARM64 support**: Optimized context switching
2. **RISC-V support**: RISC-V specific optimizations
3. **GPU scheduling**: CUDA and OpenCL integration
4. **FPGA offloading**: Heterogeneous computing support

## Conclusion

This process and thread management system provides a robust, scalable, and secure foundation for the MultiOS operating system. The modular design allows for easy extension and customization while maintaining type safety and memory safety through Rust's guarantees.

The implementation balances performance with safety, providing both high-performance operations for critical system components and safe abstractions for user-level threads and processes.