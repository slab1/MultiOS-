# MultiOS Educational Labs

Comprehensive hands-on laboratory exercises designed to reinforce operating systems concepts through practical experimentation and discovery.

## 🎯 Laboratory Learning Philosophy

### Learning by Experimentation
Our educational labs emphasize active learning through experimentation, where students explore OS concepts by:

- **Measuring Real Performance**: Collect actual performance data from MultiOS
- **Debugging Live Systems**: Practice debugging skills on running systems
- **Comparing Implementations**: Analyze different approaches and their trade-offs
- **Discovering Principles**: Learn fundamental principles through guided experimentation
- **Building Intuition**: Develop deep understanding through hands-on exploration

### Laboratory Structure
Each lab includes:
- **Pre-lab Preparation**: Reading materials and conceptual foundation
- **Guided Experiments**: Step-by-step activities with expected outcomes
- **Exploration Activities**: Open-ended investigations
- **Analysis and Reflection**: Data interpretation and concept synthesis
- **Assessment Activities**: Quizzes and practical evaluations

## 📚 Lab Sequence and Organization

### 🏁 Foundation Labs (Labs 1-10)
**Target Audience:** Beginning OS students
**Prerequisites:** Basic programming knowledge
**Duration:** 2-3 hours per lab

### 🚀 Intermediate Labs (Labs 11-20)
**Target Audience:** Students with OS course background
**Prerequisites:** Foundation labs completion
**Duration:** 3-4 hours per lab

### ⚡ Advanced Labs (Labs 21-30)
**Target Audience:** Advanced systems programming students
**Prerequisites:** Intermediate labs completion
**Duration:** 4-5 hours per lab

### 🔬 Research Labs (Labs 31-40)
**Target Audience:** Graduate students and researchers
**Prerequisites:** Advanced labs completion
**Duration:** 6-8 hours per lab

## 🏁 Foundation Laboratory Series

### Lab 1: Exploring MultiOS Architecture
**Duration:** 2.5 hours
**Difficulty:** ⭐
**Prerequisites:** Basic command line skills

#### Learning Objectives
- Understand MultiOS project structure
- Explore system architecture through hands-on investigation
- Learn to navigate large codebases effectively
- Practice using development tools

#### Pre-lab Preparation
- Read: [MultiOS Architecture Overview](preparation/multios_architecture.md)
- Watch: [Codebase Navigation Video](videos/codebase_navigation.md)
- Install: Complete development environment setup

#### Lab Activities

##### Activity 1.1: Project Structure Exploration (30 minutes)
**Objective:** Understand the organization of the MultiOS codebase

**Tasks:**
1. Clone MultiOS repository and explore top-level directory structure
2. Identify major components (kernel, bootloader, userland, etc.)
3. Analyze build system configuration (Cargo.toml, Makefile)
4. Explore architecture-specific directories

**Guided Questions:**
- How is the codebase organized? What are the main components?
- What build systems are used? How are they configured?
- Which architectures are supported? How is this organized?

**Expected Outcome:** Understanding of project organization and build system

##### Activity 1.2: Architecture Investigation (45 minutes)
**Objective:** Explore how MultiOS supports multiple architectures

**Tasks:**
1. Examine architecture-specific code in `kernel/src/arch/`
2. Compare implementation differences between x86_64, ARM64, and RISC-V
3. Build MultiOS for each supported architecture
4. Run each architecture in QEMU and observe differences

**Guided Questions:**
- What are the common abstractions across architectures?
- Which components are architecture-specific vs. shared?
- How does MultiOS handle architecture differences at runtime?

**Expected Outcome:** Understanding of cross-architecture design

##### Activity 1.3: Kernel Component Exploration (60 minutes)
**Objective:** Explore kernel architecture and major subsystems

**Tasks:**
1. Examine kernel main entry points
2. Investigate memory management subsystem
3. Explore process management components
4. Look at device driver framework
5. Trace system call handling

**Guided Questions:**
- How does the kernel initialize?
- What are the major subsystems and how do they interact?
- How are system calls implemented and handled?

**Expected Outcome:** Understanding of kernel architecture and subsystem organization

##### Activity 1.4: Development Tools Practice (45 minutes)
**Objective:** Learn essential development and debugging tools

**Tasks:**
1. Set up IDE with Rust and MultiOS support
2. Use GDB to debug running MultiOS instance
3. Practice with cargo and build system commands
4. Explore documentation generation and browsing

**Guided Questions:**
- What development tools are essential for OS development?
- How do you effectively debug kernel code?
- Where can you find documentation and help resources?

**Expected Outcome:** Proficiency with essential development tools

#### Lab Assessment
- **Concept Check Quiz** (15 minutes): 10 multiple choice questions about lab concepts
- **Practical Exercise** (30 minutes): Navigate codebase to find specific information
- **Reflection Assignment** (20 minutes): Write about discoveries and insights

#### Lab Resources
- [Lab Guide PDF](labs/foundation/lab1/lab-guide.pdf)
- [Solutions Manual](labs/foundation/lab1/solutions.md)
- [Code Reference](labs/foundation/lab1/reference/)
- [Video Tutorials](labs/foundation/lab1/videos/)

---

### Lab 2: Memory Management Fundamentals
**Duration:** 3 hours
**Difficulty:** ⭐⭐
**Prerequisites:** Lab 1 completion

#### Learning Objectives
- Understand memory allocation strategies
- Learn about memory fragmentation and coalescing
- Practice memory debugging and analysis
- Explore memory performance characteristics

#### Pre-lab Preparation
- Read: [Memory Management Concepts](preparation/memory_management.md)
- Review: [Rust Ownership and Borrowing](preparation/rust_ownership.md)
- Prepare: Rust development environment

#### Lab Activities

##### Activity 2.1: Simple Allocator Implementation (60 minutes)
**Objective:** Implement a basic memory allocator to understand allocation principles

**Tasks:**
1. Analyze existing allocator implementation in MultiOS
2. Design a simple first-fit allocator
3. Implement allocator with block tracking
4. Test allocator with various allocation patterns

**Implementation Framework:**
```rust
pub struct SimpleAllocator {
    memory_pool: Vec<u8>,
    free_list: LinkedList<FreeBlock>,
    allocated_blocks: BTreeMap<*mut u8, BlockHeader>,
}

impl SimpleAllocator {
    pub fn allocate(&mut self, size: usize) -> Result<*mut u8, AllocError> {
        // TODO: Implement first-fit allocation
    }
    
    pub fn deallocate(&mut self, ptr: *mut u8) -> Result<(), DeallocError> {
        // TODO: Implement deallocation with coalescing
    }
}
```

**Guided Questions:**
- How do you track allocated and free memory blocks?
- What are the trade-offs of different allocation strategies?
- How do you handle alignment requirements?

##### Activity 2.2: Memory Fragmentation Analysis (45 minutes)
**Objective:** Understand memory fragmentation and its impact

**Tasks:**
1. Create workloads with different allocation patterns
2. Measure memory fragmentation over time
3. Implement different allocation strategies (first-fit, best-fit)
4. Compare fragmentation characteristics

**Analysis Framework:**
```rust
pub struct FragmentationAnalyzer {
    allocator: Box<dyn Allocator>,
    measurement_points: Vec<MeasurementPoint>,
}

impl FragmentationAnalyzer {
    pub fn measure_fragmentation(&mut self, workload: &AllocationWorkload) -> FragmentationMetrics {
        // TODO: Implement fragmentation measurement
    }
}
```

**Guided Questions:**
- What causes memory fragmentation?
- How do different allocation strategies affect fragmentation?
- When is fragmentation a problem and when is it acceptable?

##### Activity 2.3: Performance Benchmarking (60 minutes)
**Objective:** Measure and compare allocator performance

**Tasks:**
1. Design benchmark suite for allocator performance
2. Measure allocation/deallocation speed
3. Analyze performance under different workloads
4. Compare with system allocator

**Benchmark Framework:**
```rust
pub struct AllocatorBenchmark {
    test_workloads: Vec<Workload>,
    metrics_collector: MetricsCollector,
}

impl AllocatorBenchmark {
    pub fn run_comprehensive_benchmark(&mut self) -> BenchmarkResults {
        // TODO: Implement benchmark execution
    }
}
```

**Guided Questions:**
- What factors affect allocator performance?
- How do you design fair benchmarks for different allocators?
- What metrics are most important for allocator evaluation?

##### Activity 2.4: Memory Debugging (45 minutes)
**Objective:** Learn to debug memory-related issues

**Tasks:**
1. Introduce memory leaks into allocator
2. Use debugging tools to identify leaks
3. Fix memory management bugs
4. Verify fix effectiveness

**Debugging Tools:**
- Memory profiler
- Leak detector
- Address sanitizer
- Valgrind integration

**Guided Questions:**
- How do you detect memory leaks in kernel code?
- What are common memory-related bugs and how do you fix them?
- How do you verify memory management correctness?

#### Lab Assessment
- **Practical Implementation** (45 minutes): Complete allocator implementation
- **Performance Analysis Report** (30 minutes): Analyze benchmark results
- **Debugging Exercise** (30 minutes): Fix intentionally broken code
- **Concept Quiz** (15 minutes): 8 questions on memory management principles

---

### Lab 3: Process Management and Scheduling
**Duration:** 3.5 hours
**Difficulty:** ⭐⭐
**Prerequisites:** Labs 1-2 completion

#### Learning Objectives
- Understand process lifecycle management
- Learn about scheduling algorithms and their trade-offs
- Practice with concurrent programming concepts
- Explore process creation and termination

#### Pre-lab Preparation
- Read: [Process Management Concepts](preparation/process_management.md)
- Review: [Scheduling Algorithms](preparation/scheduling_algorithms.md)
- Study: MultiOS process management implementation

#### Lab Activities

##### Activity 3.1: Process Control Block Analysis (45 minutes)
**Objective:** Understand process representation and management

**Tasks:**
1. Examine MultiOS Process Control Block (PCB) structure
2. Analyze process state transitions
3. Trace process creation from user space to kernel
4. Implement process statistics collection

**Analysis Framework:**
```rust
pub struct ProcessAnalyzer {
    process_db: ProcessDatabase,
    state_tracker: StateTracker,
}

impl ProcessAnalyzer {
    pub fn analyze_process_lifecycle(&self, pid: ProcessId) -> LifecycleAnalysis {
        // TODO: Implement process lifecycle analysis
    }
}
```

**Guided Questions:**
- What information is stored in a process control block?
- How are processes tracked and managed by the OS?
- What are the different process states and when do transitions occur?

##### Activity 3.2: Scheduling Algorithm Implementation (90 minutes)
**Objective:** Implement and test different scheduling algorithms

**Tasks:**
1. Implement round-robin scheduling
2. Add priority-based scheduling
3. Implement first-come-first-served (FCFS)
4. Compare algorithm performance

**Implementation Framework:**
```rust
pub trait SchedulingAlgorithm {
    fn schedule(&mut self, run_queue: &mut Vec<Process>) -> Option<Process>;
    fn preempt(&mut self, current: &Process, incoming: &Process) -> bool;
    fn update_priorities(&mut self, processes: &[Process]);
}

pub struct RoundRobinScheduler {
    time_slice: Duration,
    current_index: usize,
}

pub struct PriorityScheduler {
    priority_levels: BTreeMap<Priority, Vec<Process>>,
}
```

**Guided Questions:**
- How do different scheduling algorithms affect system responsiveness?
- What are the trade-offs between fairness and performance?
- How do you handle priority inversion problems?

##### Activity 3.3: Process Communication (60 minutes)
**Objective:** Explore inter-process communication mechanisms

**Tasks:**
1. Implement simple message passing between processes
2. Create shared memory communication
3. Add synchronization primitives (semaphores, mutexes)
4. Test communication under load

**Communication Framework:**
```rust
pub struct ProcessCommunication {
    message_queues: HashMap<QueueId, MessageQueue>,
    shared_memory: SharedMemoryManager,
    synchronization: SyncPrimitiveManager,
}

pub struct MessageQueue {
    messages: VecDeque<Message>,
    max_size: usize,
    waiting_processes: Vec<ProcessId>,
}
```

**Guided Questions:**
- What are the trade-offs between different IPC mechanisms?
- How do you ensure thread safety in IPC?
- What are common synchronization problems and how do you avoid them?

##### Activity 3.4: Performance Monitoring (45 minutes)
**Objective:** Monitor and analyze process performance

**Tasks:**
1. Implement process performance metrics collection
2. Create real-time process monitoring
3. Analyze scheduling performance under different workloads
4. Generate performance reports

**Monitoring Framework:**
```rust
pub struct ProcessMonitor {
    metrics_collector: MetricsCollector,
    real_time_view: RealTimeView,
    performance_analyzer: PerformanceAnalyzer,
}

impl ProcessMonitor {
    pub fn start_monitoring(&mut self) {
        // TODO: Start real-time monitoring
    }
    
    pub fn generate_performance_report(&self) -> PerformanceReport {
        // TODO: Generate comprehensive performance report
    }
}
```

**Guided Questions:**
- How do you measure process performance accurately?
- What metrics are most important for scheduling evaluation?
- How do you identify performance bottlenecks?

#### Lab Assessment
- **Algorithm Implementation** (60 minutes): Complete scheduling algorithm
- **Performance Analysis** (45 minutes): Compare algorithm performance
- **Communication Testing** (45 minutes): Test IPC mechanisms
- **System Design** (30 minutes): Design improved scheduling system

---

## 🚀 Intermediate Laboratory Series

### Lab 11: Advanced Memory Management
**Duration:** 4 hours
**Difficulty:** ⭐⭐⭐
**Prerequisites:** Foundation labs completion

#### Learning Objectives
- Master virtual memory systems implementation
- Understand page table management and optimization
- Learn about demand paging and page replacement
- Practice with memory protection mechanisms

#### Lab Activities

##### Activity 11.1: Virtual Memory Implementation (90 minutes)
**Objective:** Implement virtual address translation and page table management

**Implementation Tasks:**
1. Design page table structure for MultiOS
2. Implement virtual-to-physical address translation
3. Add page table management (allocation, mapping, unmapping)
4. Test virtual memory with various access patterns

**Virtual Memory Framework:**
```rust
pub struct VirtualMemoryManager {
    page_tables: BTreeMap<ProcessId, PageTable>,
    frame_allocator: FrameAllocator,
    page_replacement: PageReplacementAlgorithm,
    memory_protection: ProtectionManager,
}

pub struct PageTable {
    entries: Vec<PageTableEntry>,
    level_count: usize,
    page_size: usize,
}

impl VirtualMemoryManager {
    pub fn translate_address(&self, pid: ProcessId, vaddr: VirtualAddress) -> Result<PhysicalAddress, TranslationError> {
        // TODO: Implement multi-level page table walk
    }
    
    pub fn map_pages(&mut self, pid: ProcessId, vaddr: VirtualAddress, pages: usize, permissions: PagePermissions) -> Result<(), MappingError> {
        // TODO: Implement page mapping with allocation
    }
}
```

##### Activity 11.2: Page Replacement Algorithms (75 minutes)
**Objective:** Implement and compare page replacement strategies

**Implementation Tasks:**
1. Implement Least Recently Used (LRU) algorithm
2. Add Clock/Second Chance algorithm
3. Implement Working Set model
4. Compare algorithm performance

**Page Replacement Framework:**
```rust
pub trait PageReplacementAlgorithm {
    fn select_victim(&mut self, candidate_pages: &[PageFrame]) -> PageFrame;
    fn access_page(&mut self, page: PageFrame);
    fn reset(&mut self);
}

pub struct LRUReplacer {
    access_history: LinkedHashMap<PageFrame, Instant>,
    capacity: usize,
}

pub struct ClockReplacer {
    use_bits: Vec<bool>,
    clock_hand: usize,
    frame_count: usize,
}
```

##### Activity 11.3: Demand Paging System (60 minutes)
**Objective:** Implement demand paging with page fault handling

**Implementation Tasks:**
1. Design page fault handler
2. Implement page allocation on fault
3. Add page preload and prefetching
4. Test demand paging with various workloads

**Demand Paging Framework:**
```rust
pub struct DemandPager {
    fault_handler: PageFaultHandler,
    page_cache: PageCache,
    io_manager: IoManager,
}

impl DemandPager {
    pub fn handle_page_fault(&mut self, fault_addr: VirtualAddress, fault_type: FaultType) -> Result<(), FaultError> {
        // TODO: Implement page fault handling logic
    }
}
```

##### Activity 11.4: Memory Protection (45 minutes)
**Objective:** Implement memory protection mechanisms

**Implementation Tasks:**
1. Add memory permission checking
2. Implement write protection and copy-on-write
3. Add stack and heap protection
4. Test protection mechanisms

#### Lab Assessment
- **Virtual Memory Implementation** (90 minutes): Complete virtual memory system
- **Page Replacement Comparison** (75 minutes): Compare algorithm performance
- **Demand Paging Test** (60 minutes): Test paging system under load
- **Protection Testing** (45 minutes): Verify memory protection works

---

### Lab 12: Device Driver Development
**Duration:** 4 hours
**Difficulty:** ⭐⭐⭐
**Prerequisites:** Intermediate labs completion

#### Learning Objectives
- Master device driver architecture and development
- Understand interrupt handling and device communication
- Learn about driver frameworks and registration
- Practice with hardware abstraction

#### Lab Activities

##### Activity 12.1: Character Device Driver (90 minutes)
**Objective:** Implement a complete character device driver

**Implementation Tasks:**
1. Design character device interface
2. Implement driver registration and initialization
3. Add read/write operations
4. Handle device-specific commands

**Driver Framework:**
```rust
pub trait CharacterDevice {
    fn open(&mut self) -> Result<(), DeviceError>;
    fn close(&mut self) -> Result<(), DeviceError>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, DeviceError>;
    fn write(&mut self, buffer: &[u8]) -> Result<usize, DeviceError>;
    fn ioctl(&mut self, command: u32, data: &mut [u8]) -> Result<(), DeviceError>;
}

pub struct VirtualSerialDevice {
    buffer: VecDeque<u8>,
    is_open: bool,
    settings: DeviceSettings,
}
```

##### Activity 12.2: Interrupt Handling (75 minutes)
**Objective:** Implement interrupt handling for device events

**Implementation Tasks:**
1. Design interrupt service routine framework
2. Implement device interrupt handler
3. Add interrupt masking and prioritization
4. Test interrupt handling under various conditions

**Interrupt Framework:**
```rust
pub struct InterruptController {
    interrupt_handlers: BTreeMap<InterruptNumber, Box<dyn InterruptHandler>>,
    mask_register: InterruptMask,
    priority_levels: PriorityManager,
}

pub trait InterruptHandler {
    fn handle_interrupt(&mut self, interrupt_num: InterruptNumber) -> Result<(), InterruptError>;
    fn enable(&mut self);
    fn disable(&mut self);
}
```

##### Activity 12.3: Block Device Driver (60 minutes)
**Objective:** Implement block device interface and I/O handling

**Implementation Tasks:**
1. Design block device interface
2. Implement block I/O operations
3. Add buffer management
4. Optimize I/O performance

##### Activity 12.4: Driver Testing Framework (45 minutes)
**Objective:** Create comprehensive testing for device drivers

**Implementation Tasks:**
1. Design driver testing framework
2. Create unit tests for driver functionality
3. Add integration tests with device simulation
4. Implement performance testing

---

## ⚡ Advanced Laboratory Series

### Lab 21: Real-Time Systems Implementation
**Duration:** 5 hours
**Difficulty:** ⭐⭐⭐⭐
**Prerequisites:** Advanced labs completion

#### Learning Objectives
- Implement real-time scheduling algorithms
- Understand timing analysis and validation
- Master interrupt latency optimization
- Practice with real-time system design

#### Lab Activities

##### Activity 21.1: Real-Time Scheduler Implementation (120 minutes)
**Objective:** Build complete real-time scheduling system

**Implementation Tasks:**
1. Implement Rate Monotonic Scheduling (RMS)
2. Add Earliest Deadline First (EDF) algorithm
3. Create priority inheritance protocol
4. Implement resource reservation system

**Real-Time Scheduler Framework:**
```rust
pub struct RealTimeScheduler {
    algorithms: HashMap<AlgorithmType, Box<dyn SchedulingAlgorithm>>,
    task_set: RealTimeTaskSet,
    schedulability_analyzer: SchedulabilityAnalyzer,
    timing_validator: TimingValidator,
}

pub struct RealTimeTask {
    pub id: TaskId,
    pub period: Duration,
    pub execution_time: Duration,
    pub deadline: Duration,
    pub wcet: Duration, // Worst-case execution time
    pub utilization: f64,
}
```

##### Activity 21.2: Timing Analysis and Validation (90 minutes)
**Objective:** Implement schedulability analysis and timing validation

**Implementation Tasks:**
1. Create schedulability test implementation
2. Add worst-case execution time analysis
3. Implement response time analysis
4. Add timing validation framework

##### Activity 21.3: Interrupt Latency Optimization (90 minutes)
**Objective:** Minimize and measure interrupt handling latency

**Implementation Tasks:**
1. Measure baseline interrupt latency
2. Optimize interrupt handler code
3. Implement interrupt nesting and prioritization
4. Validate timing guarantees

##### Activity 21.4: Real-Time System Integration (60 minutes)
**Objective:** Integrate real-time components into MultiOS

**Implementation Tasks:**
1. Integrate scheduler with MultiOS kernel
2. Add real-time process management
3. Implement real-time I/O handling
4. Test system with real-time workloads

---

## 🔬 Research Laboratory Series

### Lab 31: Performance Analysis and Optimization
**Duration:** 6 hours
**Difficulty:** ⭐⭐⭐⭐⭐
**Prerequisites:** Research labs enrollment

#### Learning Objectives
- Master advanced performance analysis techniques
- Understand microarchitectural optimization
- Practice with large-scale system profiling
- Conduct performance research studies

#### Lab Activities

##### Activity 31.1: Advanced Profiling Framework (120 minutes)
**Objective:** Build comprehensive performance analysis framework

**Implementation Tasks:**
1. Design multi-level performance monitoring
2. Implement CPU performance counter analysis
3. Add memory hierarchy profiling
4. Create automated performance regression detection

**Performance Analysis Framework:**
```rust
pub struct PerformanceAnalysisFramework {
    cpu_profiler: CPUProfiler,
    memory_profiler: MemoryProfiler,
    io_profiler: IOProfiler,
    statistical_analyzer: StatisticalAnalyzer,
    visualization_engine: PerformanceVisualization,
}

pub struct PerformanceStudy {
    hypothesis: PerformanceHypothesis,
    experimental_design: ExperimentalDesign,
    data_collection: DataCollectionProtocol,
    analysis_methods: Vec<AnalysisMethod>,
}
```

##### Activity 31.2: Microarchitectural Optimization (120 minutes)
**Objective:** Implement and analyze microarchitectural optimizations

**Implementation Tasks:**
1. Analyze cache behavior and optimize data layouts
2. Implement branch prediction optimization
3. Add SIMD instruction usage where beneficial
4. Optimize memory access patterns

##### Activity 31.3: Large-Scale System Profiling (120 minutes)
**Objective:** Profile and optimize large MultiOS components

**Implementation Tasks:**
1. Profile entire kernel subsystems
2. Identify performance bottlenecks
3. Implement optimizations based on profiling data
4. Validate performance improvements

##### Activity 31.4: Performance Research Study (120 minutes)
**Objective:** Conduct rigorous performance research

**Implementation Tasks:**
1. Design performance research experiment
2. Implement automated data collection
3. Perform statistical analysis of results
4. Document findings and conclusions

---

## 📊 Lab Assessment and Evaluation

### Assessment Framework

#### Lab Participation (40%)
- **Pre-lab Preparation** (10%): Completed reading and preparation
- **Active Engagement** (20%): Participation in lab activities
- **Collaboration** (10%): Effective teamwork and helping others

#### Lab Work Quality (35%)
- **Implementation Quality** (20%): Correctness and efficiency of code
- **Problem-Solving** (10%): Ability to debug and fix issues
- **Innovation** (5%): Creative approaches and optimizations

#### Post-lab Assessment (25%)
- **Concept Understanding** (15%): Quiz and discussion assessment
- **Report Quality** (10%): Clear documentation and analysis

### Lab Success Metrics

#### Individual Learning Outcomes
- **Conceptual Understanding**: Ability to explain OS concepts clearly
- **Practical Skills**: Competence in implementing OS components
- **Problem-Solving**: Systematic approach to debugging and optimization
- **Research Skills**: Ability to design and conduct experiments

#### Community Contributions
- **Code Quality**: Contributions suitable for MultiOS inclusion
- **Documentation**: Clear lab guides and tutorials
- **Peer Teaching**: Helping other students understand concepts
- **Innovation**: Novel approaches and optimizations

### Lab Resources and Support

#### Technical Resources
- **Lab Environment**: Pre-configured MultiOS development environment
- **Hardware Access**: Specialized hardware for advanced labs
- **Software Tools**: Complete suite of development and analysis tools
- **Documentation**: Comprehensive lab guides and reference materials

#### Human Support
- **Lab Instructors**: Expert guidance during lab sessions
- **Teaching Assistants**: One-on-one help with technical issues
- **Peer Tutors**: Student mentors for collaborative learning
- **Office Hours**: Additional time for questions and guidance

#### Community Resources
- **Online Forums**: Discussion platforms for lab-related questions
- **Video Resources**: Recorded lab sessions and tutorials
- **Code Repository**: Shared code examples and solutions
- **Feedback System**: Continuous improvement based on student input

---

**Ready to get hands-on with operating systems?** Check our [lab schedule](labs/schedule/) and register for the next available session!

*Remember: Labs are designed to be challenging but supportive environments where you can explore OS concepts through direct experimentation and discovery.*