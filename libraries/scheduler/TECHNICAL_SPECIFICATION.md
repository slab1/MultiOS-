# MultiOS Process and Thread Management - Technical Specification

## API Contract and Guarantees

This document defines the exact API contracts, guarantees, and behavior expectations for the MultiOS process and thread management system.

## 1. Process Management API

### 1.1 Process Creation
```rust
pub fn syscall_create_process(params: ProcessCreateParams) -> ProcessResult<ProcessId>
```

**Contract:**
- Returns a unique ProcessId >= 1 for successful creation
- Process ID is never reused during system lifetime
- Creates a process in `ProcessState::Running` state
- Allocates initial memory according to `ProcessMemoryStats`
- Returns `ProcessError::ProcessLimitExceeded` if system limit reached

**Guarantees:**
- Process ID uniqueness is atomic and thread-safe
- Memory allocation is failure-atomic
- Process is ready for immediate thread creation
- No resource leaks on creation failure

**Error Cases:**
- `ProcessError::ProcessLimitExceeded`: System has reached maximum process count
- `ProcessError::InvalidPriority`: Priority value is invalid
- `ProcessError::OutOfMemory`: Insufficient memory for process structures

### 1.2 Process Termination
```rust
pub fn syscall_terminate_process(process_id: ProcessId, exit_status: i32) -> ProcessResult<()>
```

**Contract:**
- Transitions process to `ProcessState::Terminated`
- Sets `exit_status` in PCB for parent process retrieval
- Terminates all threads belonging to the process
- Cleans up process resources and memory
- Returns success even if process was already terminated

**Guarantees:**
- All child threads are terminated before process cleanup
- Memory is properly freed
- Parent-child relationships are updated
- No zombie processes remain

**Error Cases:**
- `ProcessError::ProcessNotFound`: Process ID does not exist
- `ProcessError::AccessDenied`: Insufficient privileges to terminate process

### 1.3 Process State Management
```rust
pub fn suspend_process(process_id: ProcessId) -> ProcessResult<()>
pub fn resume_process(process_id: ProcessId) -> ProcessResult<()>
pub fn set_process_priority(process_id: ProcessId, priority: ProcessPriority) -> ProcessResult<()>
```

**Contract:**
- `suspend_process`: Adds `ProcessFlags::SUSPENDED` flag, pauses all threads
- `resume_process`: Removes `ProcessFlags::SUSPENDED` flag, resumes threads
- `set_process_priority`: Changes process scheduling priority

**Guarantees:**
- Suspended processes do not consume CPU time
- Resumed processes resume from exact suspension point
- Priority changes take effect immediately for scheduling decisions

## 2. Thread Management API

### 2.1 Thread Creation
```rust
pub fn syscall_create_thread(
    process_id: ProcessId,
    name: Vec<u8>,
    entry_point: Option<ThreadEntry>,
    params: ThreadParams
) -> ThreadResult<ThreadHandle>
```

**Contract:**
- Creates thread within specified process
- Thread starts in `ThreadState::Ready` state
- Allocates stack of specified size (must be >= 4096 bytes)
- Sets up initial CPU context for entry point execution
- Returns Arc<Mutex<ThreadControlBlock>> for thread reference

**Guarantees:**
- Thread ID is unique and never reused
- Stack is properly aligned and protected
- Entry point function is called with correct calling convention
- Thread is immediately schedulable

**Error Cases:**
- `ProcessError::ProcessNotFound`: Target process does not exist
- `ThreadError::InvalidStackSize`: Stack size is too small or too large
- `ThreadError::OutOfMemory`: Insufficient memory for thread structures
- `ThreadError::ThreadLimitExceeded`: Per-process or system thread limit reached

### 2.2 Thread Control
```rust
pub fn syscall_terminate_thread(thread_id: ThreadId) -> ThreadResult<()>
pub fn syscall_set_thread_priority(thread_id: ThreadId, priority: Priority) -> ThreadResult<()>
pub fn syscall_sleep_thread(thread_id: ThreadId, duration_ms: u64) -> ThreadResult<()>
pub fn syscall_wake_thread(thread_id: ThreadId) -> ThreadResult<()>
```

**Contract:**
- `terminate_thread`: Transitions thread to `ThreadState::Terminated`
- `set_thread_priority`: Changes thread scheduling priority immediately
- `sleep_thread`: Transitions thread to `ThreadState::Waiting` with wake-up timer
- `wake_thread`: Transitions sleeping thread to `ThreadState::Ready`

**Guarantees:**
- Terminated threads are immediately removed from scheduling
- Priority changes affect next scheduling decision
- Sleeping threads wake at exact specified time (within timer resolution)
- Waking non-sleeping thread has no effect

**Error Cases:**
- `ThreadError::ThreadNotFound`: Thread ID does not exist
- `ThreadError::ThreadInInvalidState`: Operation not valid for current state

### 2.3 Context Switching
```rust
pub unsafe fn context_switch(current_thread: &mut ThreadControlBlock, next_thread: &ThreadControlBlock) -> !
```

**Contract:**
- Saves complete CPU state of current thread
- Restores complete CPU state of next thread
- Switches stack pointers and program counters
- Transfers execution to next thread immediately

**Guarantees:**
- All general-purpose registers are preserved
- Flags register is properly restored
- Stack pointer is correctly switched
- No instruction ordering issues across switch
- Function never returns (execution transfers permanently)

**Architecture Requirements:**
- x86_64: All 16 general-purpose registers + RFLAGS + RSP + RIP
- Future: Architecture-specific register sets

## 3. Scheduler API

### 3.1 Initialization
```rust
pub fn init() -> SchedulerResult<()>
pub fn init_with_config(config: SchedulerConfig) -> SchedulerResult<()>
```

**Contract:**
- Initializes global scheduler state
- Sets up per-CPU ready queues
- Configures scheduling algorithm
- Enables interrupt-driven scheduling

**Guarantees:**
- Scheduler is ready for immediate thread creation
- All CPUs are online and schedulable
- Load balancing is active (if enabled)
- No data races in scheduler state

**Error Cases:**
- `SchedulerError::SchedulerAlreadyInitialized`: Scheduler already initialized
- `SchedulerError::InvalidConfiguration`: Configuration parameters are invalid

### 3.2 Scheduling Operations
```rust
pub fn add_thread(thread_handle: ThreadHandle) -> SchedulerResult<()>
pub fn remove_thread(thread_id: ThreadId) -> SchedulerResult<()>
pub fn schedule_next() -> SchedulerResult<ThreadHandle>
```

**Contract:**
- `add_thread`: Adds thread to appropriate ready queue based on CPU affinity
- `remove_thread`: Removes thread from ready queues and cancels current execution
- `schedule_next`: Selects and returns next thread to execute

**Guarantees:**
- Threads are scheduled according to configured algorithm
- CPU affinity is respected
- Load balancing maintains work distribution
- No starvation for lower-priority threads (with aging)

**Error Cases:**
- `SchedulerError::NoRunnableThreads`: No ready threads available
- `SchedulerError::ThreadNotFound`: Thread not found in scheduler queues

### 3.3 Multi-Core Operations
```rust
pub fn get_cpu_count() -> usize
pub fn set_thread_cpu_affinity(thread_id: ThreadId, affinity: CpuAffinity) -> ThreadResult<()>
pub fn balance_load() -> Result<(), SchedulerError>
```

**Contract:**
- `get_cpu_count`: Returns number of online CPUs
- `set_thread_cpu_affinity`: Restricts thread to specified CPU set
- `balance_load`: Migrates threads to equalize CPU load

**Guarantees:**
- CPU count reflects actual online processors
- Affinity changes affect next scheduling decision
- Load balancing respects affinity constraints
- Migration is transparent to running threads

## 4. Priority and State Contracts

### 4.1 Priority Ordering
```rust
ProcessPriority: System < High < Normal < Low < Idle
ThreadPriority: Idle < Low < Normal < High < Critical
```

**Contract:**
- Lower numerical values = higher priority
- Higher priority threads always preferred by scheduler
- Time quantums are proportional to priority (higher = more CPU time)
- Priority inheritance is implemented for synchronization

**Guarantees:**
- Priority ordering is total and consistent
- No priority inversion without mitigation
- Starvation is prevented through aging

### 4.2 State Transitions
```rust
// Process States
Running -> Waiting  (via I/O wait)
Running -> Stopped  (via suspend)
Running -> Terminated (via termination)
Waiting -> Running  (via I/O completion)
Stopped -> Running  (via resume)
Terminated -> [terminal state]

// Thread States  
Ready -> Running    (via scheduler selection)
Running -> Ready    (via time slice expiration)
Running -> Waiting  (via sleep/wait)
Running -> Terminated (via termination)
Waiting -> Ready    (via wake/timer expiration)
```

**Contract:**
- All state transitions are atomic
- Only valid transitions are permitted
- State changes are immediately visible to scheduler
- Terminal states cannot be exited

## 5. Memory and Resource Guarantees

### 5.1 Memory Management
- **PCB Size**: ~200 bytes per process
- **TCB Size**: ~150 bytes per thread  
- **Stack Allocation**: Aligned to 16-byte boundaries
- **Stack Protection**: Guard pages prevent overflow
- **Memory Limits**: Configurable per-process and system-wide

### 5.2 Resource Limits
- **Maximum Processes**: System configuration (default: 1024)
- **Maximum Threads**: Per-process (default: 256) and system-wide (default: 4096)
- **Stack Size Limits**: Minimum 4KB, maximum 16MB
- **CPU Affinity**: Up to 32 CPUs supported

### 5.3 Locking and Synchronization
- **Global State**: Protected by spin::Mutex (non-blocking)
- **Per-CPU State**: Lock-free operations where possible
- **Thread Handles**: Arc<Mutex<T>> for safe sharing
- **No Deadlocks**: Hierarchical locking order maintained

## 6. Performance Guarantees

### 6.1 Time Complexity
- **Process Creation**: O(1) - constant time allocation
- **Thread Creation**: O(1) - constant time setup
- **Scheduling Decision**: O(1) average - ready queue lookup
- **Context Switch**: O(1) - fixed register save/restore
- **Load Balancing**: O(n) - n = CPU count

### 6.2 Latency Guarantees
- **Thread Wake-up**: <= 1 scheduler tick (configurable)
- **Context Switch**: <= 100 CPU cycles (architecture dependent)
- **Process Creation**: <= 1ms (assuming sufficient memory)
- **Priority Change**: Immediate effect on next scheduling decision

### 6.3 Scalability
- Linear scaling with CPU count
- Minimal cross-CPU communication
- NUMA-friendly memory layout
- Lock contention minimized

## 7. Error Handling and Recovery

### 7.1 Error Classification
- **Fatal Errors**: System state corruption, immediate kernel panic
- **Resource Errors**: Out of memory, limit exceeded - return error to caller
- **State Errors**: Invalid operations - return error, no state change
- **Not Found Errors**: Invalid IDs - return appropriate error

### 7.2 Recovery Mechanisms
- **Memory Allocation Failure**: Clean rollback of partial allocations
- **Thread Creation Failure**: Process state remains consistent
- **Scheduler Failure**: System remains stable, graceful degradation
- **CPU Failure**: Automatic load rebalancing to remaining CPUs

## 8. Security Considerations

### 8.1 Process Isolation
- Each process has separate address space
- Process ID provides unique identification
- No process can directly access another process's memory
- System processes have elevated privileges

### 8.2 Thread Safety
- Thread handles are reference-counted
- No direct thread structure manipulation
- Safe memory access through handles only
- Context switching preserves security boundaries

### 8.3 Privilege Separation
- System processes: Can manage other processes
- User processes: Limited to own process management
- Privilege escalation requires explicit system call
- Security checks at all privilege boundaries

## 9. Architecture-Specific Contracts

### 9.1 x86_64 Requirements
```rust
#[repr(C)]
pub struct ThreadContext {
    pub registers: [usize; 16],      // RAX, RBX, RCX, RDX, RSI, RDI, RBP, R8-R15
    pub program_counter: usize,      // RIP
    pub stack_pointer: usize,        // RSP  
    pub flags: usize,                // RFLAGS
    pub control_registers: [usize; 3], // CR0, CR3, CR4
}
```

### 9.2 Calling Convention
- **Thread Entry**: System V AMD64 ABI
- **Context Switch**: Hardware-level register save/restore
- **System Calls**: Optimized kernel entry/exit

## 10. Testing and Validation

### 10.1 API Contract Tests
- All public functions test both success and error cases
- State transition validation
- Resource leak detection
- Concurrency stress testing

### 10.2 Performance Benchmarks
- Context switch timing
- Thread creation throughput
- Scheduling latency measurement
- Multi-core scalability testing

### 10.3 Integration Tests
- End-to-end process lifecycle
- Multi-threaded application scenarios
- Cross-CPU thread migration
- System call interface validation

This specification defines the exact behavior and guarantees provided by the MultiOS process and thread management system, ensuring reliable and predictable operation across all supported platforms and configurations.