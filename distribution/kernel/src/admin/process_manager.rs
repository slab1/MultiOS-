//! MultiOS Process & Service Management
//! 
//! This module provides comprehensive process and service management functionality including:
//! - Process monitoring and control
//! - Service lifecycle management (start, stop, restart, status)
//! - Process resource usage tracking (CPU, memory, I/O)
//! - Process prioritization and scheduling integration
//! - Process termination and signal handling
//! - Service dependency management
//! - Integration with existing scheduler and service manager components

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::{Mutex, RwLock};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::{BTreeMap, HashMap, HashSet};
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicUsize, AtomicBool, Ordering};
use bitflags::bitflags;

/// Process ID type
pub type ProcessId = u32;

/// Thread ID type
pub type ThreadId = u32;

/// Service ID type (re-exported from service manager)
pub use crate::service_manager::ServiceId;

/// Process States
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessState {
    New = 0,           // Process is being created
    Ready = 1,         // Process is ready to run
    Running = 2,       // Process is currently running
    Blocked = 3,       // Process is waiting for I/O or other event
    Suspended = 4,     // Process is suspended
    Terminated = 5,    // Process has terminated
    Zombie = 6,        // Process terminated but waiting for parent to reap
    Defunct = 7,       // Process is defunct/waiting for cleanup
}

/// Process Priority Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ProcessPriority {
    Idle = 0,         // Lowest priority - runs when no other processes
    Low = 1,          // Low priority background tasks
    Normal = 2,       // Normal priority for most processes
    High = 3,         // High priority tasks
    RealTime = 4,     // Real-time priority (highest)
    Critical = 5,     // Critical system processes
}

/// Process Priority Class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessPriorityClass {
    System = 0,       // System processes
    User = 1,         // User processes
    Service = 2,      // Service processes
    Background = 3,   // Background jobs
    Interactive = 4,  // Interactive processes
}

/// Process Access Rights
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProcessAccess: u32 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
        const EXECUTE = 0b00000100;
        const TERMINATE = 0b00001000;
        const SUSPEND = 0b00010000;
        const RESUME = 0b00100000;
        const DEBUG = 0b01000000;
        const ADMIN = 0b10000000;
    }
}

/// Process Flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProcessFlags: u32 {
        const FOREGROUND = 0b00000001;
        const BACKGROUND = 0b00000010;
        const DAEMON = 0b00000100;
        const TRACED = 0b00001000;
        const SETUID = 0b00010000;
        const SETGID = 0b00100000;
        const NO_CPULIMIT = 0b01000000;
        const NO_MEMLIMIT = 0b10000000;
    }
}

/// Process Resource Limits
#[derive(Debug, Clone, Copy)]
pub struct ProcessResourceLimits {
    pub max_memory: u64,           // Maximum memory in bytes
    pub max_stack_size: u64,       // Maximum stack size
    pub max_file_descriptors: u32, // Maximum file descriptors
    pub max_processes: u32,        // Maximum processes (for process groups)
    pub max_cpu_time: u64,         // Maximum CPU time in seconds
    pub max_creation_time: u64,    // Maximum creation time
    pub max_io_read: u64,          // Maximum I/O read in bytes
    pub max_io_write: u64,         // Maximum I/O write in bytes
}

/// Process Resource Usage Statistics
#[derive(Debug, Clone)]
pub struct ProcessResourceUsage {
    pub user_time_ms: u64,         // User CPU time in milliseconds
    pub system_time_ms: u64,       // System CPU time in milliseconds
    pub total_time_ms: u64,        // Total CPU time
    pub memory_usage_bytes: u64,   // Current memory usage
    pub peak_memory_bytes: u64,    // Peak memory usage
    pub stack_usage_bytes: u64,    // Stack usage
    pub heap_usage_bytes: u64,     // Heap usage
    pub file_descriptor_count: u32, // Number of open file descriptors
    pub io_read_bytes: u64,        // Total I/O read
    pub io_write_bytes: u64,       // Total I/O write
    pub io_read_operations: u64,   // Number of read operations
    pub io_write_operations: u64,  // Number of write operations
    pub context_switches: u64,     // Number of context switches
    pub signals_received: u64,     // Number of signals received
    pub page_faults: u64,          // Number of page faults
    pub voluntary_yields: u64,     // Number of voluntary yields
}

/// Process Control Block (PCB)
#[derive(Debug)]
pub struct ProcessControlBlock {
    pub process_id: ProcessId,
    pub parent_id: Option<ProcessId>,
    pub thread_group_id: ThreadId,
    pub process_group_id: ProcessId,
    pub session_id: ProcessId,
    pub state: ProcessState,
    pub priority: ProcessPriority,
    pub priority_class: ProcessPriorityClass,
    pub flags: ProcessFlags,
    pub access_rights: ProcessAccess,
    pub resource_limits: ProcessResourceLimits,
    pub creation_time_ms: u64,
    pub start_time_ms: u64,
    pub last_scheduled_ms: u64,
    pub user_id: u32,
    pub group_id: u32,
    pub effective_user_id: u32,
    pub effective_group_id: u32,
    pub current_working_directory: String,
    pub root_directory: String,
    pub environment_variables: HashMap<String, String>,
    pub command_line: Vec<String>,
    pub resource_usage: ProcessResourceUsage,
    pub service_id: Option<ServiceId>,
    pub signal_handlers: HashMap<Signal, SignalHandler>,
    pub children: HashSet<ProcessId>,
    pub threads: HashSet<ThreadId>,
}

/// Signal Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Signal {
    SIGHUP = 1,    // Hang up
    SIGINT = 2,    // Interrupt
    SIGQUIT = 3,   // Quit
    SIGILL = 4,    // Illegal instruction
    SIGABRT = 6,   // Abort
    SIGFPE = 8,    // Floating point exception
    SIGKILL = 9,   // Kill (cannot be caught)
    SIGSEGV = 11,  // Segmentation fault
    SIGPIPE = 13,  // Broken pipe
    SIGALRM = 14,  // Alarm clock
    SIGTERM = 15,  // Termination
    SIGUSR1 = 16,  // User-defined signal 1
    SIGUSR2 = 17,  // User-defined signal 2
    SIGCHLD = 18,  // Child status changed
    SIGCONT = 19,  // Continue
    SIGSTOP = 20,  // Stop (cannot be caught)
    SIGTSTP = 21,  // Terminal stop
    SIGTTIN = 22,  // Terminal input for background process
    SIGTTOU = 23,  // Terminal output for background process
}

/// Signal Actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignalAction {
    Default = 0,    // Default action
    Ignore = 1,     // Ignore signal
    Catch = 2,      // Catch signal with handler
    Stop = 3,       // Stop process
    Terminate = 4,  // Terminate process
}

/// Signal Handler
#[derive(Debug)]
pub struct SignalHandler {
    pub action: SignalAction,
    pub handler_address: Option<usize>,
    pub mask: HashSet<Signal>,
}

/// Service Process Manager
#[derive(Debug)]
pub struct ServiceProcess {
    pub service_id: ServiceId,
    pub process_id: ProcessId,
    pub name: String,
    pub enabled: bool,
    pub auto_restart: bool,
    pub restart_count: u32,
    pub max_restarts: u32,
    pub last_start_time_ms: u64,
    pub dependencies: HashSet<ServiceId>,
    pub resource_quota: Option<ProcessResourceLimits>,
}

/// Process Manager Statistics
#[derive(Debug, Clone)]
pub struct ProcessManagerStats {
    pub total_processes: u32,
    pub running_processes: u32,
    pub blocked_processes: u32,
    pub suspended_processes: u32,
    pub terminated_processes: u32,
    pub zombie_processes: u32,
    pub service_processes: u32,
    pub system_processes: u32,
    pub user_processes: u32,
    pub signals_sent: u64,
    pub signals_handled: u64,
    pub context_switches: u64,
    pub process_creations: u64,
    pub process_terminations: u64,
    pub service_restarts: u64,
}

/// Process Manager Configuration
#[derive(Debug, Clone)]
pub struct ProcessManagerConfig {
    pub max_processes: u32,
    pub max_service_processes: u32,
    pub default_stack_size: u64,
    pub default_memory_limit: u64,
    pub default_file_descriptor_limit: u32,
    pub enable_process_accounting: bool,
    pub enable_resource_monitoring: bool,
    pub service_timeout_ms: u64,
    pub grace_period_ms: u64,
    pub emergency_termination_timeout_ms: u64,
}

/// Global Process Manager
pub struct ProcessManager {
    next_process_id: AtomicU32,
    next_thread_id: AtomicU32,
    processes: RwLock<BTreeMap<ProcessId, ProcessControlBlock>>,
    service_processes: RwLock<HashMap<ServiceId, ServiceProcess>>,
    signal_handlers: RwLock<HashMap<Signal, HashMap<ProcessId, SignalHandler>>>,
    process_tree: RwLock<HashMap<ProcessId, HashSet<ProcessId>>>,
    config: ProcessManagerConfig,
    stats: RwLock<ProcessManagerStats>,
    initialized: AtomicBool,
}

/// Global process manager instance
static PROCESS_MANAGER: Mutex<Option<ProcessManager>> = Mutex::new(None);

/// Process operation result
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Process management errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessError {
    ProcessNotFound = 0,
    InvalidProcessId,
    ProcessAlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    InvalidState,
    InvalidPriority,
    DependencyError,
    ServiceNotFound,
    SignalNotSupported,
    ResourceLimitExceeded,
    Timeout,
    SystemError,
    ServiceError,
    InvalidResourceLimits,
    CircularDependency,
}

impl ProcessManager {
    /// Create a new process manager instance
    pub fn new() -> Self {
        ProcessManager {
            next_process_id: AtomicU32::new(1),
            next_thread_id: AtomicU32::new(1),
            processes: RwLock::new(BTreeMap::new()),
            service_processes: RwLock::new(HashMap::new()),
            signal_handlers: RwLock::new(HashMap::new()),
            process_tree: RwLock::new(HashMap::new()),
            config: ProcessManagerConfig::default(),
            stats: RwLock::new(ProcessManagerStats::default()),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize the process manager
    pub fn init() -> ProcessResult<()> {
        let mut guard = PROCESS_MANAGER.lock();
        
        if guard.is_some() {
            return Err(ProcessError::ProcessAlreadyExists);
        }

        let manager = ProcessManager::new();
        guard.replace(manager);
        
        info!("Process Manager initialized successfully");
        Ok(())
    }

    /// Get process manager instance
    pub fn get() -> Option<&'static Mutex<Option<ProcessManager>>> {
        Some(&PROCESS_MANAGER)
    }

    /// Create a new process
    pub fn create_process(
        &self,
        parent_id: Option<ProcessId>,
        priority: ProcessPriority,
        priority_class: ProcessPriorityClass,
        flags: ProcessFlags,
        command: Vec<String>,
        working_directory: String,
        environment: HashMap<String, String>,
    ) -> ProcessResult<ProcessId> {
        // Check resource limits
        let current_stats = self.stats.read();
        if current_stats.total_processes >= self.config.max_processes {
            return Err(ProcessError::ResourceExhausted);
        }
        drop(current_stats);

        let process_id = self.allocate_process_id();
        let thread_id = self.allocate_thread_id();

        // Create process control block
        let pcb = ProcessControlBlock {
            process_id,
            parent_id,
            thread_group_id: thread_id,
            process_group_id: process_id,
            session_id: process_id,
            state: ProcessState::New,
            priority,
            priority_class,
            flags,
            access_rights: ProcessAccess::READ | ProcessAccess::WRITE | ProcessAccess::EXECUTE,
            resource_limits: self.create_default_limits(),
            creation_time_ms: crate::hal::get_current_time(),
            start_time_ms: 0,
            last_scheduled_ms: 0,
            user_id: 0, // Would be set from current user context
            group_id: 0,
            effective_user_id: 0,
            effective_group_id: 0,
            current_working_directory: working_directory,
            root_directory: "/".to_string(),
            environment_variables: environment,
            command_line: command,
            resource_usage: ProcessResourceUsage::default(),
            service_id: None,
            signal_handlers: self.create_default_signal_handlers(),
            children: HashSet::new(),
            threads: HashSet::new(),
        };

        // Add to process tree if parent exists
        if let Some(parent_pid) = parent_id {
            let mut tree = self.process_tree.write();
            let parent_children = tree.entry(parent_pid).or_insert_with(HashSet::new);
            parent_children.insert(process_id);
        }

        // Store process control block
        let mut processes = self.processes.write();
        processes.insert(process_id, pcb);

        // Update statistics
        self.update_stats(|stats| {
            stats.total_processes += 1;
            stats.process_creations += 1;
        });

        info!("Created process {} with priority {:?}", process_id, priority);
        Ok(process_id)
    }

    /// Terminate a process
    pub fn terminate_process(&self, process_id: ProcessId, force: bool) -> ProcessResult<()> {
        let mut processes = self.processes.write();
        let pcb = processes.get_mut(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        // Check permissions
        if !pcb.access_rights.contains(ProcessAccess::TERMINATE) {
            return Err(ProcessError::PermissionDenied);
        }

        match pcb.state {
            ProcessState::Terminated | ProcessState::Zombie | ProcessState::Defunct => {
                return Ok(()); // Already terminated
            }
            _ => {
                if force {
                    // Immediate termination
                    pcb.state = ProcessState::Terminated;
                    self.force_cleanup_process(process_id)?;
                } else {
                    // Graceful termination - send SIGTERM
                    pcb.state = ProcessState::Terminated;
                    self.send_signal_internal(process_id, Signal::SIGTERM)?;
                }
            }
        }

        // Update statistics
        self.update_stats(|stats| {
            stats.total_processes = stats.total_processes.saturating_sub(1);
            stats.process_terminations += 1;
        });

        info!("Terminated process {} (force: {})", process_id, force);
        Ok(())
    }

    /// Suspend a process
    pub fn suspend_process(&self, process_id: ProcessId) -> ProcessResult<()> {
        let mut processes = self.processes.write();
        let pcb = processes.get_mut(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        if !pcb.access_rights.contains(ProcessAccess::SUSPEND) {
            return Err(ProcessError::PermissionDenied);
        }

        match pcb.state {
            ProcessState::Running | ProcessState::Ready => {
                pcb.state = ProcessState::Suspended;
                self.update_stats(|stats| stats.suspended_processes += 1);
                info!("Suspended process {}", process_id);
                Ok(())
            }
            _ => Err(ProcessError::InvalidState),
        }
    }

    /// Resume a process
    pub fn resume_process(&self, process_id: ProcessId) -> ProcessResult<()> {
        let mut processes = self.processes.write();
        let pcb = processes.get_mut(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        if !pcb.access_rights.contains(ProcessAccess::RESUME) {
            return Err(ProcessError::PermissionDenied);
        }

        if pcb.state == ProcessState::Suspended {
            pcb.state = ProcessState::Ready;
            self.update_stats(|stats| stats.suspended_processes = stats.suspended_processes.saturating_sub(1));
            info!("Resumed process {}", process_id);
        }

        Ok(())
    }

    /// Get process information
    pub fn get_process_info(&self, process_id: ProcessId) -> ProcessResult<ProcessControlBlock> {
        let processes = self.processes.read();
        let pcb = processes.get(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        Ok(pcb.clone())
    }

    /// List all processes
    pub fn list_processes(&self) -> ProcessResult<Vec<ProcessId>> {
        let processes = self.processes.read();
        Ok(processes.keys().cloned().collect())
    }

    /// Get process statistics
    pub fn get_process_stats(&self, process_id: ProcessId) -> ProcessResult<ProcessResourceUsage> {
        let processes = self.processes.read();
        let pcb = processes.get(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        Ok(pcb.resource_usage.clone())
    }

    /// Update process priority
    pub fn set_process_priority(&self, process_id: ProcessId, priority: ProcessPriority) -> ProcessResult<()> {
        let mut processes = self.processes.write();
        let pcb = processes.get_mut(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        pcb.priority = priority;

        // Update scheduler priority
        self.update_scheduler_priority(process_id, priority)?;

        info!("Updated process {} priority to {:?}", process_id, priority);
        Ok(())
    }

    /// Send signal to a process
    pub fn send_signal(&self, process_id: ProcessId, signal: Signal) -> ProcessResult<()> {
        self.send_signal_internal(process_id, signal)?;

        self.update_stats(|stats| {
            stats.signals_sent += 1;
        });

        info!("Sent signal {:?} to process {}", signal, process_id);
        Ok(())
    }

    /// Create a service process
    pub fn create_service_process(
        &self,
        service_id: ServiceId,
        name: String,
        command: Vec<String>,
        auto_restart: bool,
        max_restarts: u32,
    ) -> ProcessResult<ProcessId> {
        // Check service process limits
        let current_service_count = self.service_processes.read().len() as u32;
        if current_service_count >= self.config.max_service_processes {
            return Err(ProcessError::ResourceExhausted);
        }

        let process_id = self.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::Service,
            ProcessFlags::DAEMON,
            command,
            "/".to_string(),
            HashMap::new(),
        )?;

        // Create service process record
        let service_process = ServiceProcess {
            service_id,
            process_id,
            name,
            enabled: true,
            auto_restart,
            restart_count: 0,
            max_restarts,
            last_start_time_ms: crate::hal::get_current_time(),
            dependencies: HashSet::new(),
            resource_quota: None,
        };

        // Store service process
        let mut service_processes = self.service_processes.write();
        service_processes.insert(service_id, service_process);

        // Link process to service
        let mut processes = self.processes.write();
        if let Some(pcb) = processes.get_mut(&process_id) {
            pcb.service_id = Some(service_id);
            pcb.state = ProcessState::Ready;
        }

        info!("Created service process {} for service {}", process_id, service_id.0);
        Ok(process_id)
    }

    /// Start a service process
    pub fn start_service_process(&self, service_id: ServiceId) -> ProcessResult<()> {
        let service_processes = self.service_processes.read();
        let service_process = service_processes.get(&service_id)
            .ok_or(ProcessError::ServiceNotFound)?;

        if !service_process.enabled {
            return Err(ProcessError::InvalidState);
        }

        // Check dependencies
        self.check_service_dependencies(service_id)?;

        let mut processes = self.processes.write();
        let pcb = processes.get_mut(&service_process.process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        pcb.state = ProcessState::Running;
        pcb.start_time_ms = crate::hal::get_current_time();

        info!("Started service process {} for service {}", service_process.process_id, service_id.0);
        Ok(())
    }

    /// Stop a service process
    pub fn stop_service_process(&self, service_id: ServiceId) -> ProcessResult<()> {
        let service_processes = self.service_processes.read();
        let service_process = service_processes.get(&service_id)
            .ok_or(ProcessError::ServiceNotFound)?;

        let process_id = service_process.process_id;
        self.terminate_process(process_id, false)?;

        info!("Stopped service process {} for service {}", process_id, service_id.0);
        Ok(())
    }

    /// Restart a service process
    pub fn restart_service_process(&self, service_id: ServiceId) -> ProcessResult<()> {
        // Stop the service
        self.stop_service_process(service_id)?;

        // Wait for grace period
        crate::hal::sleep_ms(self.config.grace_period_ms);

        // Start the service
        self.start_service_process(service_id)?;

        self.update_stats(|stats| {
            stats.service_restarts += 1;
        });

        info!("Restarted service process for service {}", service_id.0);
        Ok(())
    }

    /// Get service process status
    pub fn get_service_status(&self, service_id: ServiceId) -> ProcessResult<ServiceProcess> {
        let service_processes = self.service_processes.read();
        let service_process = service_processes.get(&service_id)
            .ok_or(ProcessError::ServiceNotFound)?;

        Ok(service_process.clone())
    }

    /// Monitor process resource usage
    pub fn monitor_process_resources(&self) -> ProcessResult<()> {
        if !self.config.enable_resource_monitoring {
            return Ok(());
        }

        let current_time = crate::hal::get_current_time();
        let mut processes = self.processes.write();

        for pcb in processes.values_mut() {
            if pcb.state == ProcessState::Running {
                // Update resource usage
                self.update_process_resource_usage(pcb, current_time)?;

                // Check resource limits
                if self.check_resource_limits(pcb)? {
                    warn!("Process {} exceeded resource limits", pcb.process_id);
                    // Take action (e.g., terminate or throttle)
                    self.handle_resource_limit_exceeded(pcb.process_id)?;
                }
            }
        }

        Ok(())
    }

    /// Get process manager statistics
    pub fn get_stats(&self) -> ProcessManagerStats {
        self.stats.read().clone()
    }

    /// Internal helper methods
    fn allocate_process_id(&self) -> ProcessId {
        self.next_process_id.fetch_add(1, Ordering::SeqCst)
    }

    fn allocate_thread_id(&self) -> ThreadId {
        self.next_thread_id.fetch_add(1, Ordering::SeqCst)
    }

    fn create_default_limits(&self) -> ProcessResourceLimits {
        ProcessResourceLimits {
            max_memory: self.config.default_memory_limit,
            max_stack_size: self.config.default_stack_size,
            max_file_descriptors: self.config.default_file_descriptor_limit,
            max_processes: 1,
            max_cpu_time: 3600, // 1 hour
            max_creation_time: 10000, // 10 seconds
            max_io_read: 1024 * 1024 * 1024, // 1 GB
            max_io_write: 1024 * 1024 * 1024, // 1 GB
        }
    }

    fn create_default_signal_handlers(&self) -> HashMap<Signal, SignalHandler> {
        let mut handlers = HashMap::new();
        
        // Set default handlers for common signals
        handlers.insert(Signal::SIGTERM, SignalHandler {
            action: SignalAction::Default,
            handler_address: None,
            mask: HashSet::new(),
        });
        
        handlers.insert(Signal::SIGKILL, SignalHandler {
            action: SignalAction::Terminate,
            handler_address: None,
            mask: HashSet::new(),
        });
        
        handlers.insert(Signal::SIGINT, SignalHandler {
            action: SignalAction::Default,
            handler_address: None,
            mask: HashSet::new(),
        });
        
        handlers
    }

    fn send_signal_internal(&self, process_id: ProcessId, signal: Signal) -> ProcessResult<()> {
        let mut processes = self.processes.write();
        let pcb = processes.get_mut(&process_id)
            .ok_or(ProcessError::ProcessNotFound)?;

        let handler = pcb.signal_handlers.get(&signal)
            .ok_or(ProcessError::SignalNotSupported)?;

        // Update usage statistics
        pcb.resource_usage.signals_received += 1;

        // Handle signal based on action
        match handler.action {
            SignalAction::Default => self.handle_signal_default(process_id, signal)?,
            SignalAction::Ignore => {/* Do nothing */},
            SignalAction::Catch => self.handle_signal_catch(process_id, signal, handler)?,
            SignalAction::Stop => pcb.state = ProcessState::Suspended,
            SignalAction::Terminate => pcb.state = ProcessState::Terminated,
        }

        self.update_stats(|stats| {
            stats.signals_handled += 1;
        });

        Ok(())
    }

    fn handle_signal_default(&self, process_id: ProcessId, signal: Signal) -> ProcessResult<()> {
        // Handle default signal behavior
        match signal {
            Signal::SIGTERM | Signal::SIGKILL => {
                self.terminate_process(process_id, signal == Signal::SIGKILL)?;
            }
            Signal::SIGSTOP => {
                self.suspend_process(process_id)?;
            }
            Signal::SIGCONT => {
                self.resume_process(process_id)?;
            }
            _ => {
                // Default behavior for other signals
                warn!("Unhandled signal {:?} for process {}", signal, process_id);
            }
        }
        Ok(())
    }

    fn handle_signal_catch(&self, process_id: ProcessId, signal: Signal, handler: &SignalHandler) -> ProcessResult<()> {
        // Call user-defined signal handler
        info!("Calling signal handler for signal {:?} in process {}", signal, process_id);
        Ok(())
    }

    fn update_scheduler_priority(&self, process_id: ProcessId, priority: ProcessPriority) -> ProcessResult<()> {
        // Integrate with scheduler
        // This would update the process priority in the scheduler
        info!("Updated scheduler priority for process {} to {:?}", process_id, priority);
        Ok(())
    }

    fn check_service_dependencies(&self, service_id: ServiceId) -> ProcessResult<()> {
        let service_processes = self.service_processes.read();
        let service_process = service_processes.get(&service_id)
            .ok_or(ProcessError::ServiceNotFound)?;

        for &dep_service_id in &service_process.dependencies {
            let dep_service = service_processes.get(&dep_service_id)
                .ok_or(ProcessError::DependencyError)?;

            let mut processes = self.processes.write();
            let dep_pcb = processes.get_mut(&dep_service.process_id)
                .ok_or(ProcessError::DependencyError)?;

            if dep_pcb.state != ProcessState::Running {
                return Err(ProcessError::DependencyError);
            }
        }

        Ok(())
    }

    fn update_process_resource_usage(&self, pcb: &mut ProcessControlBlock, current_time: u64) -> ProcessResult<()> {
        // Simulate resource usage updates
        // In a real implementation, this would read actual system stats
        
        pcb.resource_usage.total_time_ms = current_time.saturating_sub(pcb.start_time_ms);
        pcb.resource_usage.context_switches += 1; // Simulate context switches
        
        // Simulate memory usage
        if pcb.resource_usage.memory_usage_bytes == 0 {
            pcb.resource_usage.memory_usage_bytes = 1024 * 1024; // 1MB baseline
        }
        
        Ok(())
    }

    fn check_resource_limits(&self, pcb: &ProcessControlBlock) -> ProcessResult<bool> {
        let limits = &pcb.resource_limits;
        let usage = &pcb.resource_usage;

        if usage.memory_usage_bytes > limits.max_memory {
            return Ok(true);
        }

        if usage.total_time_ms / 1000 > limits.max_cpu_time {
            return Ok(true);
        }

        if usage.file_descriptor_count > limits.max_file_descriptors {
            return Ok(true);
        }

        Ok(false)
    }

    fn handle_resource_limit_exceeded(&self, process_id: ProcessId) -> ProcessResult<()> {
        // Take action when resource limits are exceeded
        warn!("Taking action for process {} resource limit exceeded", process_id);
        
        // Options: terminate, throttle, or send notification
        // For now, send SIGTERM
        self.send_signal_internal(process_id, Signal::SIGTERM)?;
        
        Ok(())
    }

    fn force_cleanup_process(&self, process_id: ProcessId) -> ProcessResult<()> {
        // Force cleanup of process resources
        let mut processes = self.processes.write();
        processes.remove(&process_id);

        // Clean up from process tree
        let mut tree = self.process_tree.write();
        tree.remove(&process_id);

        Ok(())
    }

    fn update_stats<F>(&self, f: F)
    where
        F: FnOnce(&mut ProcessManagerStats),
    {
        let mut stats = self.stats.write();
        f(&mut stats);
    }
}

impl Default for ProcessManagerConfig {
    fn default() -> Self {
        Self {
            max_processes: 65536,
            max_service_processes: 1024,
            default_stack_size: 8 * 1024 * 1024, // 8MB
            default_memory_limit: 256 * 1024 * 1024, // 256MB
            default_file_descriptor_limit: 1024,
            enable_process_accounting: true,
            enable_resource_monitoring: true,
            service_timeout_ms: 30000, // 30 seconds
            grace_period_ms: 5000, // 5 seconds
            emergency_termination_timeout_ms: 10000, // 10 seconds
        }
    }
}

impl Default for ProcessResourceUsage {
    fn default() -> Self {
        Self {
            user_time_ms: 0,
            system_time_ms: 0,
            total_time_ms: 0,
            memory_usage_bytes: 0,
            peak_memory_bytes: 0,
            stack_usage_bytes: 0,
            heap_usage_bytes: 0,
            file_descriptor_count: 0,
            io_read_bytes: 0,
            io_write_bytes: 0,
            io_read_operations: 0,
            io_write_operations: 0,
            context_switches: 0,
            signals_received: 0,
            page_faults: 0,
            voluntary_yields: 0,
        }
    }
}

impl Default for ProcessManagerStats {
    fn default() -> Self {
        Self {
            total_processes: 0,
            running_processes: 0,
            blocked_processes: 0,
            suspended_processes: 0,
            terminated_processes: 0,
            zombie_processes: 0,
            service_processes: 0,
            system_processes: 0,
            user_processes: 0,
            signals_sent: 0,
            signals_handled: 0,
            context_switches: 0,
            process_creations: 0,
            process_terminations: 0,
            service_restarts: 0,
        }
    }
}

/// Initialize process manager
pub fn init_process_manager() -> Result<()> {
    info!("Initializing Process Manager...");
    
    ProcessManager::init()?;
    
    let manager_guard = PROCESS_MANAGER.lock();
    if let Some(manager) = manager_guard.as_ref() {
        manager.initialized.store(true, Ordering::SeqCst);
    }
    
    info!("Process Manager initialized successfully");
    Ok(())
}

/// Shutdown process manager
pub fn shutdown_process_manager() -> Result<()> {
    info!("Shutting down Process Manager...");
    
    let manager_guard = PROCESS_MANAGER.lock();
    if let Some(manager) = manager_guard.as_ref() {
        // Terminate all processes gracefully
        let processes = manager.list_processes()?;
        for process_id in processes {
            if let Err(e) = manager.terminate_process(process_id, false) {
                warn!("Failed to terminate process {}: {:?}", process_id, e);
            }
        }
        
        info!("All processes terminated");
    }
    
    info!("Process Manager shutdown complete");
    Ok(())
}

/// System call interface for process management
pub mod syscall {
    use super::*;

    /// Create a new process
    pub fn create_process(
        parent_id: Option<ProcessId>,
        priority: ProcessPriority,
        command: Vec<String>,
    ) -> ProcessResult<ProcessId> {
        let manager_guard = PROCESS_MANAGER.lock();
        let manager = manager_guard.as_ref().ok_or(ProcessError::ProcessNotFound)?;
        
        manager.create_process(
            parent_id,
            priority,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            command,
            "/".to_string(),
            HashMap::new(),
        )
    }

    /// Terminate a process
    pub fn terminate_process(process_id: ProcessId) -> ProcessResult<()> {
        let manager_guard = PROCESS_MANAGER.lock();
        let manager = manager_guard.as_ref().ok_or(ProcessError::ProcessNotFound)?;
        manager.terminate_process(process_id, false)
    }

    /// Get process information
    pub fn get_process_info(process_id: ProcessId) -> ProcessResult<ProcessControlBlock> {
        let manager_guard = PROCESS_MANAGER.lock();
        let manager = manager_guard.as_ref().ok_or(ProcessError::ProcessNotFound)?;
        manager.get_process_info(process_id)
    }

    /// Send signal to process
    pub fn send_signal(process_id: ProcessId, signal: Signal) -> ProcessResult<()> {
        let manager_guard = PROCESS_MANAGER.lock();
        let manager = manager_guard.as_ref().ok_or(ProcessError::ProcessNotFound)?;
        manager.send_signal(process_id, signal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_creation() {
        let manager = ProcessManager::new();
        let process_id = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        assert!(process_id > 0);
        
        let info = manager.get_process_info(process_id).unwrap();
        assert_eq!(info.process_id, process_id);
        assert_eq!(info.state, ProcessState::New);
        assert_eq!(info.priority, ProcessPriority::Normal);
    }

    #[test]
    fn test_process_termination() {
        let manager = ProcessManager::new();
        let process_id = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        manager.terminate_process(process_id, true).unwrap();
        
        let info = manager.get_process_info(process_id).unwrap();
        assert_eq!(info.state, ProcessState::Terminated);
    }

    #[test]
    fn test_signal_handling() {
        let manager = ProcessManager::new();
        let process_id = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        manager.send_signal(process_id, Signal::SIGTERM).unwrap();
        
        let info = manager.get_process_info(process_id).unwrap();
        assert_eq!(info.resource_usage.signals_received, 1);
    }

    #[test]
    fn test_service_process_management() {
        let manager = ProcessManager::new();
        let service_id = ServiceId(1);
        
        let process_id = manager.create_service_process(
            service_id,
            "test-service".to_string(),
            vec!["service".to_string()],
            true,
            3,
        ).unwrap();
        
        assert!(process_id > 0);
        
        let status = manager.get_service_status(service_id).unwrap();
        assert_eq!(status.service_id, service_id);
        assert_eq!(status.auto_restart, true);
        assert_eq!(status.max_restarts, 3);
    }

    #[test]
    fn test_resource_monitoring() {
        let manager = ProcessManager::new();
        let process_id = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        // Simulate running process
        let mut processes = manager.processes.write();
        if let Some(pcb) = processes.get_mut(&process_id) {
            pcb.state = ProcessState::Running;
            pcb.start_time_ms = crate::hal::get_current_time();
        }
        drop(processes);
        
        manager.monitor_process_resources().unwrap();
        
        let usage = manager.get_process_stats(process_id).unwrap();
        assert!(usage.total_time_ms > 0);
    }

    #[test]
    fn test_process_priority_change() {
        let manager = ProcessManager::new();
        let process_id = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        manager.set_process_priority(process_id, ProcessPriority::High).unwrap();
        
        let info = manager.get_process_info(process_id).unwrap();
        assert_eq!(info.priority, ProcessPriority::High);
    }

    #[test]
    fn test_process_manager_statistics() {
        let manager = ProcessManager::new();
        manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_processes, 1);
        assert_eq!(stats.process_creations, 1);
    }

    #[test]
    fn test_process_list() {
        let manager = ProcessManager::new();
        let process1 = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test1".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        let process2 = manager.create_process(
            None,
            ProcessPriority::Normal,
            ProcessPriorityClass::User,
            ProcessFlags::empty(),
            vec!["test2".to_string()],
            "/".to_string(),
            HashMap::new(),
        ).unwrap();
        
        let processes = manager.list_processes().unwrap();
        assert_eq!(processes.len(), 2);
        assert!(processes.contains(&process1));
        assert!(processes.contains(&process2));
    }

    #[test]
    fn test_error_handling() {
        let manager = ProcessManager::new();
        
        // Test non-existent process
        assert_eq!(manager.terminate_process(9999, false), Err(ProcessError::ProcessNotFound));
        assert_eq!(manager.get_process_info(9999), Err(ProcessError::ProcessNotFound));
        assert_eq!(manager.get_process_stats(9999), Err(ProcessError::ProcessNotFound));
        assert_eq!(manager.set_process_priority(9999, ProcessPriority::High), Err(ProcessError::ProcessNotFound));
        assert_eq!(manager.send_signal(9999, Signal::SIGTERM), Err(ProcessError::ProcessNotFound));
        
        // Test invalid service
        assert_eq!(manager.get_service_status(ServiceId(9999)), Err(ProcessError::ServiceNotFound));
    }
}