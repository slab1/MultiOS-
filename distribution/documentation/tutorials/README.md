# MultiOS Tutorials

Welcome to the MultiOS tutorial series! These hands-on guides will teach you how to develop applications, drivers, and kernel modules for MultiOS.

## Tutorial Series

### Beginner Level

1. [Writing Your First MultiOS Application](#tutorial-1-your-first-application)
2. [Understanding System Calls](#tutorial-2-system-calls)
3. [File System Operations](#tutorial-3-file-system-operations)
4. [Basic Network Programming](#tutorial-4-network-programming)
5. [Creating a Simple GUI Application](#tutorial-5-gui-applications)

### Intermediate Level

6. [Building a Device Driver](#tutorial-6-device-drivers)
7. [Custom System Services](#tutorial-7-system-services)
8. [Cross-Platform Development](#tutorial-8-cross-platform)
9. [Memory Management](#tutorial-9-memory-management)
10. [Process and Thread Management](#tutorial-10-threads-and-processes)

### Advanced Level

11. [Kernel Module Development](#tutorial-11-kernel-modules)
12. [Custom File Systems](#tutorial-12-file-systems)
13. [Interrupt Handling](#tutorial-13-interrupt-handling)
14. [Performance Optimization](#tutorial-14-performance)
15. [Security Implementation](#tutorial-15-security)

## Tutorial 1: Your First Application

Let's start with a simple "Hello World" application to understand the MultiOS development environment.

### Prerequisites

- Rust toolchain installed
- MultiOS development environment set up
- QEMU for testing

### Step 1: Create Project Structure

```bash
# Create a new directory for our tutorial
mkdir multios-tutorials
cd multios-tutorials
mkdir hello-world
cd hello-world
```

### Step 2: Create Cargo.toml

```toml
[package]
name = "hello-world"
version = "0.1.0"
edition = "2021"

[dependencies]
# MultiOS system libraries
multios-syscalls = { path = "../kernel/syscalls" }
multios-memory = { path = "../libraries/memory-manager" }
multios-log = { path = "../kernel/log" }

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

### Step 3: Write the Application

Create `src/main.rs`:

```rust
//! Hello World Tutorial for MultiOS
//! 
//! This tutorial demonstrates:
//! - Basic application structure
//! - Using system calls
//! - Logging and debugging
//! - Error handling

use multios_syscalls::*;
use multios_log::*;

/// Main entry point
fn main() -> Result<(), MultiOSError> {
    // Initialize logging
    init_log(LogLevel::Info)?;
    
    info!("Starting Hello World application");
    
    // Get system information
    print_system_info()?;
    
    // Demonstrate various operations
    demonstrate_operations()?;
    
    info!("Hello World application completed successfully");
    
    Ok(())
}

/// Print basic system information
fn print_system_info() -> Result<(), MultiOSError> {
    info!("=== System Information ===");
    
    // Get system info via system call
    let system_info = get_system_info()?;
    println!("Architecture: {:?}", system_info.architecture);
    println!("Memory: {} MB", system_info.total_memory / (1024 * 1024));
    println!("CPU Cores: {}", system_info.cpu_count);
    println!("Kernel Version: {}", system_info.kernel_version);
    
    Ok(())
}

/// Demonstrate various MultiOS operations
fn demonstrate_operations() -> Result<(), MultiOSError> {
    info!("=== Demonstrating Operations ===");
    
    // 1. File operations
    demonstrate_file_operations()?;
    
    // 2. Memory operations
    demonstrate_memory_operations()?;
    
    // 3. Time operations
    demonstrate_time_operations()?;
    
    // 4. Process operations
    demonstrate_process_operations()?;
    
    Ok(())
}

/// Demonstrate file system operations
fn demonstrate_file_operations() -> Result<(), MultiOSError> {
    info!("File operations demo");
    
    // Create a file
    let file_path = "/tmp/hello_world.txt";
    let file_id = file_open(file_path, FileOpenFlags::CREATE | FileOpenFlags::WRITE)?;
    
    // Write data
    let data = b"Hello from MultiOS!\nThis is a tutorial file.\n";
    let bytes_written = file_write(file_id, data)?;
    info!("Wrote {} bytes to file", bytes_written);
    
    // Read data back
    let mut buffer = vec![0u8; data.len()];
    let bytes_read = file_read(file_id, &mut buffer)?;
    info!("Read {} bytes from file", bytes_read);
    info!("File contents: {}", String::from_utf8_lossy(&buffer));
    
    // Close file
    file_close(file_id)?;
    
    // Clean up
    file_unlink(file_path)?;
    
    Ok(())
}

/// Demonstrate memory operations
fn demonstrate_memory_operations() -> Result<(), MultiOSError> {
    info!("Memory operations demo");
    
    // Allocate memory
    let size = 1024;
    let memory_ptr = virtual_alloc(size, MemoryFlags::READ | MemoryFlags::WRITE)?;
    info!("Allocated {} bytes at address: {:?}", size, memory_ptr);
    
    // Write data to memory
    unsafe {
        for (i, byte) in b"Hello Memory!".iter().enumerate() {
            if i < size {
                memory_ptr.add(i).write_volatile(*byte);
            }
        }
    }
    
    // Read data from memory
    let mut read_buffer = vec![0u8; 16];
    unsafe {
        for i in 0..read_buffer.len() {
            read_buffer[i] = memory_ptr.add(i).read_volatile();
        }
    }
    
    info!("Memory contents: {}", String::from_utf8_lossy(&read_buffer));
    
    // Free memory
    virtual_free(memory_ptr, size)?;
    info!("Memory freed");
    
    Ok(())
}

/// Demonstrate time operations
fn demonstrate_time_operations() -> Result<(), MultiOSError> {
    info!("Time operations demo");
    
    // Get current time
    let current_time = get_current_time()?;
    info!("Current time: {:?}", current_time);
    
    // Get uptime
    let uptime = get_uptime()?;
    info!("System uptime: {:?}", uptime);
    
    // Sleep for a bit
    info!("Sleeping for 1 second...");
    sleep(1000)?; // milliseconds
    
    Ok(())
}

/// Demonstrate process operations
fn demonstrate_process_operations() -> Result<(), MultiOSError> {
    info!("Process operations demo");
    
    // Get current process ID
    let current_pid = get_current_pid()?;
    info!("Current process ID: {}", current_pid);
    
    // Get process information
    let process_info = get_process_info(current_pid)?;
    info!("Process info: {:?}", process_info);
    
    // List all processes
    let processes = list_processes()?;
    info!("Total processes: {}", processes.len());
    
    Ok(())
}
```

### Step 4: Add System Call Wrappers

Create `src/syscall_wrappers.rs`:

```rust
//! System call wrapper functions
//! 
//! These wrappers provide a more convenient interface to MultiOS system calls.
//! They handle error checking and provide type-safe interfaces.

use super::*;

/// System information structure
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub architecture: Architecture,
    pub total_memory: u64,
    pub cpu_count: u32,
    pub kernel_version: String,
}

/// Process information structure
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_id: ProcessId,
    pub name: String,
    pub state: ProcessState,
    pub memory_usage: u64,
    pub cpu_time: u64,
}

/// File open flags
bitflags! {
    pub struct FileOpenFlags: u32 {
        const READ = 0x00000001;
        const WRITE = 0x00000002;
        const CREATE = 0x00000004;
        const TRUNCATE = 0x00000008;
        const APPEND = 0x00000010;
        const EXCLUSIVE = 0x00000020;
        const NON_BLOCKING = 0x00000040;
    }
}

/// Memory allocation flags
bitflags! {
    pub struct MemoryFlags: u32 {
        const READ = 0x00000001;
        const WRITE = 0x00000002;
        const EXECUTE = 0x00000004;
        const USER = 0x00000008;
        const KERNEL = 0x00000010;
    }
}

/// Initialize logging system
pub fn init_log(level: LogLevel) -> Result<(), MultiOSError> {
    // Initialize the kernel logging system
    // This is a simplified version - actual implementation would
    // interface with the kernel logging subsystem
    Ok(())
}

/// Get system information
pub fn get_system_info() -> Result<SystemInfo, MultiOSError> {
    // This would be implemented as a system call
    // For tutorial purposes, we'll return mock data
    Ok(SystemInfo {
        architecture: Architecture::X86_64,
        total_memory: 1024 * 1024 * 1024, // 1 GB
        cpu_count: 4,
        kernel_version: "MultiOS v1.0.0".to_string(),
    })
}

/// Open a file
pub fn file_open(path: &str, flags: FileOpenFlags) -> Result<FileId, MultiOSError> {
    let path_ptr = path.as_ptr();
    let result = syscall(
        SyscallNumber::FileOpen,
        &[path_ptr as u64, flags.bits() as u64],
    )?;
    Ok(FileId(result as u64))
}

/// Write to a file
pub fn file_write(file_id: FileId, data: &[u8]) -> Result<usize, MultiOSError> {
    let data_ptr = data.as_ptr();
    let result = syscall(
        SyscallNumber::FileWrite,
        &[file_id.0, data_ptr as u64, data.len() as u64],
    )?;
    Ok(result as usize)
}

/// Read from a file
pub fn file_read(file_id: FileId, buffer: &mut [u8]) -> Result<usize, MultiOSError> {
    let buffer_ptr = buffer.as_mut_ptr();
    let result = syscall(
        SyscallNumber::FileRead,
        &[file_id.0, buffer_ptr as u64, buffer.len() as u64],
    )?;
    Ok(result as usize)
}

/// Close a file
pub fn file_close(file_id: FileId) -> Result<(), MultiOSError> {
    syscall(SyscallNumber::FileClose, &[file_id.0])?;
    Ok(())
}

/// Delete a file
pub fn file_unlink(path: &str) -> Result<(), MultiOSError> {
    let path_ptr = path.as_ptr();
    syscall(SyscallNumber::FileUnlink, &[path_ptr as u64])?;
    Ok(())
}

/// Allocate virtual memory
pub fn virtual_alloc(size: usize, flags: MemoryFlags) -> Result<*mut u8, MultiOSError> {
    let result = syscall(
        SyscallNumber::VirtualAlloc,
        &[size as u64, flags.bits() as u64],
    )?;
    Ok(result as *mut u8)
}

/// Free virtual memory
pub fn virtual_free(ptr: *mut u8, size: usize) -> Result<(), MultiOSError> {
    syscall(SyscallNumber::VirtualFree, &[ptr as u64, size as u64])?;
    Ok(())
}

/// Get current time
pub fn get_current_time() -> Result<SystemTime, MultiOSError> {
    let result = syscall(SyscallNumber::GetCurrentTime, &[])?;
    Ok(SystemTime::from_raw(result))
}

/// Sleep for specified milliseconds
pub fn sleep(milliseconds: u32) -> Result<(), MultiOSError> {
    syscall(SyscallNumber::Sleep, &[milliseconds as u64])?;
    Ok(())
}

/// Get system uptime
pub fn get_uptime() -> Result<Duration, MultiOSError> {
    // This would be a system call
    Ok(Duration::from_secs(123))
}

/// Get current process ID
pub fn get_current_pid() -> Result<ProcessId, MultiOSError> {
    let result = syscall(SyscallNumber::GetPid, &[])?;
    Ok(ProcessId(result))
}

/// Get process information
pub fn get_process_info(process_id: ProcessId) -> Result<ProcessInfo, MultiOSError> {
    let result = syscall(SyscallNumber::ProcessGetInfo, &[process_id.0])?;
    // Parse result into ProcessInfo structure
    // For tutorial purposes, return mock data
    Ok(ProcessInfo {
        process_id,
        name: "hello-world".to_string(),
        state: ProcessState::Running,
        memory_usage: 1024 * 1024,
        cpu_time: 0,
    })
}

/// List all processes
pub fn list_processes() -> Result<Vec<ProcessInfo>, MultiOSError> {
    let result = syscall(SyscallNumber::GetProcessList, &[])?;
    // Parse result into vector of ProcessInfo
    // For tutorial purposes, return mock data
    Ok(vec![ProcessInfo {
        process_id: ProcessId(1),
        name: "kernel".to_string(),
        state: ProcessState::Running,
        memory_usage: 1024 * 1024,
        cpu_time: 0,
    }])
}
```

### Step 5: Build and Run

```bash
# Build the application
cargo build

# Run in MultiOS QEMU environment
cd ../..
make run-x86_64
```

### Expected Output

```
[INFO] Starting Hello World application
[INFO] === System Information ===
Architecture: X86_64
Memory: 1024 MB
CPU Cores: 4
Kernel Version: MultiOS v1.0.0
[INFO] File operations demo
[INFO] Wrote 49 bytes to file
[INFO] Read 49 bytes from file
[INFO] File contents: Hello from MultiOS!
This is a tutorial file.
[INFO] Memory operations demo
[INFO] Allocated 1024 bytes at address: 0x80001000
[INFO] Memory contents: Hello Memory!
[INFO] Memory freed
[INFO] Time operations demo
[INFO] Current time: SystemTime { seconds: 1234567890, nanoseconds: 123456789 }
[INFO] System uptime: Duration { secs: 123, nanos: 0 }
[INFO] Sleeping for 1 second...
[INFO] Process operations demo
[INFO] Current process ID: 42
[INFO] Process info: ProcessInfo { process_id: ProcessId(42), ... }
[INFO] Total processes: 1
[INFO] Hello World application completed successfully
```

### Tutorial 1 Summary

You have successfully created your first MultiOS application! Key concepts learned:

- ✅ Application structure and entry point
- ✅ System call interface
- ✅ File system operations
- ✅ Memory allocation and management
- ✅ Time and process management
- ✅ Logging and debugging

## Tutorial 2: Understanding System Calls

This tutorial dives deeper into the system call interface and teaches you how to create custom system calls.

### What You'll Learn

- How system calls work internally
- Creating custom system call handlers
- Passing data between user and kernel space
- Error handling in system calls

### Step 1: Add Custom System Call

First, let's add a custom system call to our kernel. Add to `kernel/src/syscall/numbers.rs`:

```rust
/// Custom tutorial system calls
pub enum TutorialSyscall {
    HelloWorld = 1000,
    GetTutorialInfo = 1001,
    TutorialCalculation = 1002,
}
```

### Step 2: Implement System Call Handler

Create `kernel/src/syscall/tutorial.rs`:

```rust
//! Tutorial system call implementations
//! 
//! This module demonstrates how to implement custom system calls
//! and handle data passing between user and kernel space.

use crate::syscall::*;
use crate::error::*;

/// Tutorial system call handler
pub fn handle_tutorial_syscall(
    syscall_number: TutorialSyscall,
    arg1: u64,
    arg2: u64,
    arg3: u64,
) -> Result<u64, KernelError> {
    match syscall_number {
        TutorialSyscall::HelloWorld => handle_hello_world_syscall(),
        TutorialSyscall::GetTutorialInfo => handle_get_tutorial_info_syscall(),
        TutorialSyscall::TutorialCalculation => {
            handle_tutorial_calculation_syscall(arg1, arg2, arg3)
        }
    }
}

/// Handle hello world system call
fn handle_hello_world_syscall() -> Result<u64, KernelError> {
    info!("Tutorial: Hello World system call invoked from user space!");
    
    // Return a greeting message length as the result
    let message = "Hello from MultiOS Tutorial!";
    Ok(message.len() as u64)
}

/// Handle get tutorial info system call
fn handle_get_tutorial_info_syscall() -> Result<u64, KernelError> {
    info!("Tutorial: Get tutorial info system call invoked");
    
    // Allocate a tutorial info structure in kernel space
    let tutorial_info = TutorialInfo {
        version: "1.0.0",
        tutorial_number: 2,
        description: "System Call Tutorial",
        features: vec![
            "Custom system calls".to_string(),
            "User/kernel communication".to_string(),
            "Data passing".to_string(),
        ],
    };
    
    // Write the structure to user space
    let user_ptr = arg1 as *mut TutorialInfo;
    unsafe {
        user_ptr.write(tutorial_info);
    }
    
    Ok(0) // Success
}

/// Handle tutorial calculation system call
fn handle_tutorial_calculation_syscall(
    arg1: u64,
    arg2: u64,
    arg3: u64,
) -> Result<u64, KernelError> {
    info!("Tutorial: Calculation system call - operands: {}, {}, {}", arg1, arg2, arg3);
    
    // Perform a calculation: (arg1 + arg2) * arg3
    let result = (arg1 + arg2) * arg3;
    
    info!("Tutorial: Calculation result: {}", result);
    
    Ok(result)
}

/// Tutorial information structure
#[derive(Debug, Clone)]
struct TutorialInfo {
    version: &'static str,
    tutorial_number: u32,
    description: &'static str,
    features: Vec<String>,
}
```

### Step 3: Integrate with Main System Call Handler

Modify `kernel/src/syscall/handler.rs`:

```rust
// Add the tutorial handler
mod tutorial;

pub fn handle_syscall(
    syscall_number: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) -> Result<u64, KernelError> {
    match syscall_number {
        // ... existing system calls ...
        
        // Tutorial system calls
        1000 => tutorial::handle_tutorial_syscall(
            tutorial::TutorialSyscall::HelloWorld,
            arg1, arg2, arg3,
        ),
        1001 => tutorial::handle_tutorial_syscall(
            tutorial::TutorialSyscall::GetTutorialInfo,
            arg1, arg2, arg3,
        ),
        1002 => tutorial::handle_tutorial_syscall(
            tutorial::TutorialSyscall::TutorialCalculation,
            arg1, arg2, arg3,
        ),
        
        // ... rest of system calls ...
        
        _ => Err(KernelError::InvalidSyscall),
    }
}
```

### Step 4: Create Tutorial Application

Create `tutorials/tutorial-02-syscalls/src/main.rs`:

```rust
//! Tutorial 2: System Calls
//! 
//! This tutorial demonstrates:
//! - How to invoke custom system calls
//! - Passing data to and from the kernel
//! - Error handling in system calls

use std::mem::size_of;

/// Tutorial system call numbers (must match kernel)
const TUTORIAL_HELLO_WORLD: u64 = 1000;
const TUTORIAL_GET_INFO: u64 = 1001;
const TUTORIAL_CALCULATION: u64 = 1002;

/// Tutorial information structure (must match kernel)
#[derive(Debug, Clone)]
struct TutorialInfo {
    version: [u8; 16],  // Fixed-size version string
    tutorial_number: u32,
    description: [u8; 32],  // Fixed-size description
    feature_count: u32,
    features: [u32; 16],  // Offsets to feature strings
}

/// System call interface
fn syscall(syscall_num: u64, arg1: u64, arg2: u64, arg3: u64) -> Result<u64, TutorialError> {
    unsafe {
        let result = syscall4(syscall_num, arg1, arg2, arg3, 0);
        if result < 0 {
            Err(TutorialError::from_raw(-result as i32))
        } else {
            Ok(result as u64)
        }
    }
}

extern "C" {
    fn syscall4(num: u64, a1: u64, a2: u64, a3: u64, a4: u64) -> i64;
}

/// Tutorial error types
#[derive(Debug)]
enum TutorialError {
    SyscallFailed,
    InvalidParameter,
    OutOfMemory,
}

impl TutorialError {
    fn from_raw(error_code: i32) -> Self {
        match error_code {
            1 => TutorialError::InvalidParameter,
            2 => TutorialError::OutOfMemory,
            _ => TutorialError::SyscallFailed,
        }
    }
}

fn main() -> Result<(), TutorialError> {
    println!("=== Tutorial 2: System Calls ===\n");
    
    // 1. Call hello world system call
    println!("1. Calling Hello World system call:");
    let message_length = syscall(TUTORIAL_HELLO_WORLD, 0, 0, 0)
        .map_err(|_| TutorialError::SyscallFailed)?;
    println!("   Message length: {}", message_length);
    println!("   ✓ System call executed successfully\n");
    
    // 2. Call get tutorial info system call
    println!("2. Getting tutorial information:");
    
    // Allocate buffer for tutorial info
    let info_size = size_of::<TutorialInfo>();
    let info_ptr = allocate_buffer(info_size) as *mut TutorialInfo;
    
    if info_ptr.is_null() {
        return Err(TutorialError::OutOfMemory);
    }
    
    let result = syscall(TUTORIAL_GET_INFO, info_ptr as u64, 0, 0)
        .map_err(|_| TutorialError::SyscallFailed)?;
    
    if result == 0 {
        // Read the tutorial info from kernel space
        let tutorial_info = unsafe { info_ptr.read() };
        
        println!("   Tutorial Version: {}", 
                 String::from_utf8_lossy(&tutorial_info.version));
        println!("   Tutorial Number: {}", tutorial_info.tutorial_number);
        println!("   Description: {}", 
                 String::from_utf8_lossy(&tutorial_info.description));
        println!("   Number of Features: {}", tutorial_info.feature_count);
        println!("   ✓ Tutorial info retrieved successfully\n");
    }
    
    // 3. Call calculation system call
    println!("3. Performing calculation via system call:");
    let operand1 = 10;
    let operand2 = 5;
    let multiplier = 3;
    
    println!("   Calculating: ({} + {}) * {}", operand1, operand2, multiplier);
    
    let calculation_result = syscall(
        TUTORIAL_CALCULATION,
        operand1 as u64,
        operand2 as u64,
        multiplier as u64,
    ).map_err(|_| TutorialError::SyscallFailed)?;
    
    let expected_result = (operand1 + operand2) * multiplier;
    
    if calculation_result == expected_result as u64 {
        println!("   Result: {}", calculation_result);
        println!("   ✓ Calculation completed correctly\n");
    } else {
        println!("   ✗ Calculation error: expected {}, got {}", 
                 expected_result, calculation_result);
        return Err(TutorialError::SyscallFailed);
    }
    
    // Clean up
    deallocate_buffer(info_ptr as *mut u8, info_size);
    
    println!("=== Tutorial 2 Completed Successfully! ===");
    println!("\nKey concepts learned:");
    println!("- How system calls transfer data between user and kernel space");
    println!("- Creating custom system call handlers");
    println!("- Error handling in system calls");
    println!("- Memory management for system call data");
    
    Ok(())
}

/// Allocate buffer for system call data
fn allocate_buffer(size: usize) -> *mut u8 {
    // This would use the kernel's memory allocation
    // For tutorial purposes, we'll use a simple allocation
    let mut buffer = Vec::<u8>::with_capacity(size);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

/// Deallocate buffer
fn deallocate_buffer(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, size, size);
    }
}
```

### Step 5: Build and Test

```bash
# Build the tutorial
cd tutorials/tutorial-02-syscalls
cargo build

# Run in MultiOS
# (This would require setting up a proper testing environment)
```

### Tutorial 2 Summary

You've learned how to:

- ✅ Create custom system calls
- ✅ Implement system call handlers in the kernel
- ✅ Pass data between user and kernel space
- ✅ Handle errors in system calls
- ✅ Manage memory for system call parameters

## Tutorial 3: File System Operations

This tutorial teaches you how to work with files and directories in MultiOS.

### What You'll Learn

- File operations (create, read, write, delete)
- Directory operations
- File attributes and permissions
- Creating a simple file-based database

### Tutorial Application

Create `tutorials/tutorial-03-filesystem/src/main.rs`:

```rust
//! Tutorial 3: File System Operations
//! 
//! This tutorial demonstrates:
//! - File and directory operations
//! - File attributes and permissions
//! - Building a simple file-based database

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};

/// Simple file-based database
struct FileDatabase {
    database_path: String,
}

/// Database record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    id: u32,
    name: String,
    email: String,
    data: Vec<u8>,
}

impl FileDatabase {
    pub fn new(database_path: &str) -> Self {
        FileDatabase {
            database_path: database_path.to_string(),
        }
    }
    
    /// Initialize database (create if doesn't exist)
    pub fn init(&self) -> Result<(), FileSystemError> {
        // Check if database file exists
        if !path_exists(&self.database_path)? {
            // Create database file
            let mut file = create_file(&self.database_path, FileFlags::CREATE | FileFlags::WRITE)?;
            
            // Write empty database header
            let header = DatabaseHeader {
                version: 1,
                record_count: 0,
                next_id: 1,
            };
            
            let header_bytes = bincode::serialize(&header)?;
            file.write_all(&header_bytes)?;
            
            file_close(file)?;
        }
        
        Ok(())
    }
    
    /// Add a new record
    pub fn add_record(&self, name: &str, email: &str, data: &[u8]) -> Result<u32, FileSystemError> {
        let mut file = open_file(&self.database_path, FileFlags::READ | FileFlags::WRITE)?;
        
        // Read header
        let mut header_bytes = vec![0u8; size_of::<DatabaseHeader>()];
        file.read_exact(&mut header_bytes)?;
        let mut header: DatabaseHeader = bincode::deserialize(&header_bytes)?;
        
        // Create new record
        let record = Record {
            id: header.next_id,
            name: name.to_string(),
            email: email.to_string(),
            data: data.to_vec(),
        };
        
        // Serialize record
        let record_bytes = bincode::serialize(&record)?;
        let record_size = record_bytes.len() as u32;
        
        // Seek to end of file
        file.seek(SeekFrom::End(0))?;
        
        // Write record size and data
        file.write_all(&record_size.to_le_bytes())?;
        file.write_all(&record_bytes)?;
        
        // Update header
        file.seek(SeekFrom::Start(0))?;
        header.record_count += 1;
        header.next_id += 1;
        
        let header_bytes = bincode::serialize(&header)?;
        file.write_all(&header_bytes)?;
        
        file_close(file)?;
        
        println!("Added record ID {}: {} ({})", record.id, name, email);
        Ok(record.id)
    }
    
    /// Retrieve a record by ID
    pub fn get_record(&self, id: u32) -> Result<Option<Record>, FileSystemError> {
        let mut file = open_file(&self.database_path, FileFlags::READ)?;
        
        // Read header
        let mut header_bytes = vec![0u8; size_of::<DatabaseHeader>()];
        file.read_exact(&mut header_bytes)?;
        let header: DatabaseHeader = bincode::deserialize(&header_bytes)?;
        
        // Search for record
        let mut position = size_of::<DatabaseHeader>() as u64;
        
        for _ in 0..header.record_count {
            // Read record size
            file.seek(SeekFrom::Start(position))?;
            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            let record_size = u32::from_le_bytes(size_bytes) as usize;
            
            // Read record data
            let mut record_bytes = vec![0u8; record_size];
            file.read_exact(&mut record_bytes)?;
            let record: Record = bincode::deserialize(&record_bytes)?;
            
            if record.id == id {
                file_close(file)?;
                return Ok(Some(record));
            }
            
            position += 4 + record_size as u64;
        }
        
        file_close(file)?;
        Ok(None)
    }
    
    /// List all records
    pub fn list_records(&self) -> Result<Vec<Record>, FileSystemError> {
        let mut file = open_file(&self.database_path, FileFlags::READ)?;
        
        // Read header
        let mut header_bytes = vec![0u8; size_of::<DatabaseHeader>()];
        file.read_exact(&mut header_bytes)?;
        let header: DatabaseHeader = bincode::deserialize(&header_bytes)?;
        
        let mut records = Vec::new();
        let mut position = size_of::<DatabaseHeader>() as u64;
        
        for _ in 0..header.record_count {
            // Read record size
            file.seek(SeekFrom::Start(position))?;
            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            let record_size = u32::from_le_bytes(size_bytes) as usize;
            
            // Read record data
            let mut record_bytes = vec![0u8; record_size];
            file.read_exact(&mut record_bytes)?;
            let record: Record = bincode::deserialize(&record_bytes)?;
            records.push(record);
            
            position += 4 + record_size as u64;
        }
        
        file_close(file)?;
        Ok(records)
    }
    
    /// Delete a record by ID
    pub fn delete_record(&self, id: u32) -> Result<bool, FileSystemError> {
        // For simplicity, we'll mark records as deleted
        // In a real implementation, you might want to compact the file
        
        let mut file = open_file(&self.database_path, FileFlags::READ | FileFlags::WRITE)?;
        
        // Read header
        let mut header_bytes = vec![0u8; size_of::<DatabaseHeader>()];
        file.read_exact(&mut header_bytes)?;
        let header: DatabaseHeader = bincode::deserialize(&header_bytes)?;
        
        // Search for record and mark as deleted
        let mut position = size_of::<DatabaseHeader>() as u64;
        
        for _ in 0..header.record_count {
            // Read record size
            file.seek(SeekFrom::Start(position))?;
            let mut size_bytes = [0u8; 4];
            file.read_exact(&mut size_bytes)?;
            let record_size = u32::from_le_bytes(size_bytes) as usize;
            
            // Read first few bytes to check ID
            let mut record_bytes = vec![0u8; std::mem::size_of::<u32>()];
            file.read_exact(&mut record_bytes)?;
            let record_id = u32::from_le_bytes(record_bytes.try_into().unwrap());
            
            if record_id == id {
                // Mark as deleted (set ID to 0)
                file.seek(SeekFrom::Start(position + 4))?;
                let deleted_record = bincode::serialize(&Record {
                    id: 0, // Deleted marker
                    name: String::new(),
                    email: String::new(),
                    data: Vec::new(),
                })?;
                file.write_all(&deleted_record)?;
                
                file_close(file)?;
                println!("Deleted record ID {}", id);
                return Ok(true);
            }
            
            position += 4 + record_size as u64;
        }
        
        file_close(file)?;
        Ok(false)
    }
}

/// Database file header
#[derive(Debug, Serialize, Deserialize)]
struct DatabaseHeader {
    version: u32,
    record_count: u32,
    next_id: u32,
}

/// File system operations (simplified wrappers)
fn path_exists(path: &str) -> Result<bool, FileSystemError> {
    // This would check if a path exists using system calls
    Ok(true) // Simplified for tutorial
}

fn create_file(path: &str, flags: FileFlags) -> Result<File, FileSystemError> {
    // This would create a file using system calls
    File::create(path).map_err(|e| FileSystemError::CreateFailed(e.to_string()))
}

fn open_file(path: &str, flags: FileFlags) -> Result<File, FileSystemError> {
    // This would open a file using system calls
    File::open(path).map_err(|e| FileSystemError::OpenFailed(e.to_string()))
}

fn file_close(file: File) -> Result<(), FileSystemError> {
    // This would close a file using system calls
    drop(file);
    Ok(())
}

#[derive(Debug)]
enum FileSystemError {
    CreateFailed(String),
    OpenFailed(String),
    ReadFailed(String),
    WriteFailed(String),
    NotFound,
}

fn main() -> Result<(), FileSystemError> {
    println!("=== Tutorial 3: File System Operations ===\n");
    
    // Create database
    let database = FileDatabase::new("/tmp/tutorial.db");
    
    // Initialize database
    database.init()?;
    println!("✓ Database initialized\n");
    
    // Add some records
    println!("Adding records:");
    let id1 = database.add_record("Alice", "alice@example.com", b"Hello Alice!")?;
    let id2 = database.add_record("Bob", "bob@example.com", b"Hello Bob!")?;
    let id3 = database.add_record("Charlie", "charlie@example.com", b"Hello Charlie!")?;
    println!();
    
    // List all records
    println!("Listing all records:");
    let records = database.list_records()?;
    for record in &records {
        println!("  ID: {}, Name: {}, Email: {}, Data: {}", 
                 record.id, record.name, record.email, 
                 String::from_utf8_lossy(&record.data));
    }
    println!();
    
    // Retrieve specific record
    println!("Retrieving record ID {}:", id2);
    if let Some(record) = database.get_record(id2)? {
        println!("  Found: ID: {}, Name: {}, Email: {}, Data: {}", 
                 record.id, record.name, record.email, 
                 String::from_utf8_lossy(&record.data));
    }
    println!();
    
    // Delete a record
    println!("Deleting record ID {}", id3);
    if database.delete_record(id3)? {
        println!("✓ Record deleted\n");
    }
    
    // List records again
    println!("Records after deletion:");
    let records = database.list_records()?;
    for record in &records {
        if record.id > 0 { // Skip deleted records
            println!("  ID: {}, Name: {}, Email: {}", 
                     record.id, record.name, record.email);
        }
    }
    
    println!("\n=== File System Tutorial Completed! ===");
    println!("\nKey concepts learned:");
    println!("- File and directory operations");
    println!("- File attributes and permissions");
    println!("- Building file-based data structures");
    println!("- Error handling in file operations");
    
    Ok(())
}
```

### Tutorial 3 Summary

You've learned how to:

- ✅ Create, read, write, and delete files
- ✅ Work with file attributes and permissions
- ✅ Build file-based data structures
- ✅ Handle file system errors
- ✅ Implement a simple database

## Tutorial 4: Network Programming

This tutorial introduces network programming in MultiOS.

### What You'll Learn

- Creating network sockets
- TCP client and server implementation
- UDP communication
- Building a simple chat application

### Tutorial Application

Create `tutorials/tutorial-04-network/src/main.rs`:

```rust
//! Tutorial 4: Network Programming
//! 
//! This tutorial demonstrates:
//! - Creating network sockets
//! - TCP client and server communication
//! - UDP communication
//! - Building a simple chat application

use std::net::{TcpListener, TcpStream, UdpSocket};
use std::io::{Read, Write, BufRead, BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Simple chat message
#[derive(Debug, Clone)]
struct ChatMessage {
    username: String,
    message: String,
    timestamp: u64,
}

/// Chat server
struct ChatServer {
    tcp_listener: TcpListener,
    udp_socket: Arc<UdpSocket>,
    clients: Arc<Mutex<Vec<TcpStream>>>,
    messages: Arc<Mutex<Vec<ChatMessage>>>,
}

impl ChatServer {
    pub fn new(tcp_port: u16, udp_port: u16) -> Result<Self, NetworkError> {
        // Create TCP listener
        let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", tcp_port))
            .map_err(|e| NetworkError::BindFailed(e.to_string()))?;
        
        // Create UDP socket
        let udp_socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", udp_port))
            .map_err(|e| NetworkError::BindFailed(e.to_string()))?);
        
        Ok(ChatServer {
            tcp_listener,
            udp_socket,
            clients: Arc::new(Mutex::new(Vec::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub fn run(&self) -> Result<(), NetworkError> {
        println!("Chat server starting on TCP:{} and UDP:{}", 
                 self.tcp_listener.local_addr()?.port(),
                 self.udp_socket.local_addr()?.port());
        
        // Start UDP message receiver
        let udp_socket = Arc::clone(&self.udp_socket);
        let messages = Arc::clone(&self.messages);
        thread::spawn(move || {
            Self::receive_udp_messages(&udp_socket, &messages);
        });
        
        // Accept TCP connections
        for stream in self.tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&self.clients);
                    let messages = Arc::clone(&self.messages);
                    thread::spawn(move || {
                        Self::handle_tcp_client(stream, &clients, &messages);
                    });
                }
                Err(e) => {
                    println!("Connection error: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_tcp_client(
        stream: TcpStream,
        clients: &Arc<Mutex<Vec<TcpStream>>>,
        messages: &Arc<Mutex<Vec<ChatMessage>>>,
    ) {
        let addr = stream.peer_addr().unwrap();
        println!("New TCP connection from {}", addr);
        
        // Add client to list
        {
            let mut client_list = clients.lock().unwrap();
            client_list.push(stream.try_clone().unwrap());
        }
        
        // Send chat history
        Self::send_chat_history(&stream, messages);
        
        // Handle client messages
        let reader = BufReader::new(&stream);
        for line in reader.lines() {
            match line {
                Ok(msg) => {
                    if msg.starts_with("/quit") {
                        println!("Client {} disconnected", addr);
                        break;
                    }
                    
                    // Parse message
                    let parts: Vec<&str> = msg.splitn(2, ' ').collect();
                    if parts.len() == 2 && parts[0] == "/username" {
                        // Handle username change
                        let username = parts[1].to_string();
                        println!("Client {} set username to: {}", addr, username);
                    } else {
                        // Broadcast message
                        let chat_msg = ChatMessage {
                            username: format!("Client[{}]", addr.port()),
                            message: msg,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };
                        
                        Self::broadcast_message(&chat_msg, clients, messages);
                    }
                }
                Err(e) => {
                    println!("Error reading from client {}: {}", addr, e);
                    break;
                }
            }
        }
        
        // Remove client from list
        {
            let mut client_list = clients.lock().unwrap();
            client_list.retain(|client| {
                client.peer_addr().unwrap() != addr
            });
        }
    }
    
    fn receive_udp_messages(
        udp_socket: &UdpSocket,
        messages: &Arc<Mutex<Vec<ChatMessage>>>,
    ) {
        let mut buffer = [0; 1024];
        loop {
            match udp_socket.recv_from(&mut buffer) {
                Ok((len, addr)) => {
                    let msg = String::from_utf8_lossy(&buffer[..len]);
                    println!("UDP message from {}: {}", addr, msg);
                    
                    let chat_msg = ChatMessage {
                        username: format!("UDP[{}:{}]", addr.ip(), addr.port()),
                        message: msg.to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    
                    messages.lock().unwrap().push(chat_msg);
                }
                Err(e) => {
                    println!("UDP receive error: {}", e);
                }
            }
        }
    }
    
    fn send_chat_history(
        stream: &TcpStream,
        messages: &Arc<Mutex<Vec<ChatMessage>>>,
    ) {
        let writer = BufWriter::new(stream);
        let messages_list = messages.lock().unwrap();
        
        for msg in &*messages_list {
            let line = format!("[{}] {}: {}\n", 
                             msg.timestamp, msg.username, msg.message);
            let _ = writer.write_all(line.as_bytes());
        }
        let _ = writer.flush();
    }
    
    fn broadcast_message(
        message: &ChatMessage,
        clients: &Arc<Mutex<Vec<TcpStream>>>,
        messages: &Arc<Mutex<Vec<ChatMessage>>>,
    ) {
        // Add to message history
        messages.lock().unwrap().push(message.clone());
        
        // Send to all connected clients
        let line = format!("[{}] {}: {}\n", 
                         message.timestamp, message.username, message.message);
        
        let client_list = clients.lock().unwrap();
        for client in &*client_list {
            let _ = client.write_all(line.as_bytes());
        }
    }
}

/// TCP client
struct TcpChatClient {
    stream: TcpStream,
    username: String,
}

impl TcpChatClient {
    pub fn connect(address: &str, username: &str) -> Result<Self, NetworkError> {
        let stream = TcpStream::connect(address)
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        
        // Set non-blocking for read timeout
        stream.set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        
        Ok(TcpChatClient {
            stream,
            username: username.to_string(),
        })
    }
    
    pub fn run(&mut self) -> Result<(), NetworkError> {
        println!("Connected to chat server as {}", self.username);
        println!("Type messages to chat. Use '/quit' to exit.");
        
        let (reader, writer) = self.stream.split();
        let mut reader = BufReader::new(reader);
        let mut writer = BufWriter::new(writer);
        
        // Send username
        let username_msg = format!("/username {}\n", self.username);
        writer.write_all(username_msg.as_bytes())?;
        writer.flush()?;
        
        // Thread for sending messages
        let sender_stream = self.stream.try_clone().unwrap();
        thread::spawn(move || {
            Self::handle_user_input(sender_stream);
        });
        
        // Thread for receiving messages
        let receiver_stream = self.stream.try_clone().unwrap();
        thread::spawn(move || {
            Self::receive_messages(receiver_stream);
        });
        
        // Wait for threads to finish
        loop {
            thread::sleep(Duration::from_millis(100));
        }
    }
    
    fn handle_user_input(mut stream: TcpStream) {
        let stdin = std::io::stdin();
        let mut input = String::new();
        
        loop {
            input.clear();
            match stdin.read_line(&mut input) {
                Ok(_) => {
                    if input.trim() == "/quit" {
                        println!("Disconnecting...");
                        break;
                    }
                    
                    if let Err(e) = stream.write_all(input.as_bytes()) {
                        println!("Send error: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Input error: {}", e);
                    break;
                }
            }
        }
    }
    
    fn receive_messages(mut stream: TcpStream) {
        let reader = BufReader::new(&mut stream);
        for line in reader.lines() {
            match line {
                Ok(msg) => {
                    println!("{}", msg);
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
}

/// Network error types
#[derive(Debug)]
enum NetworkError {
    BindFailed(String),
    ConnectionFailed(String),
    SendFailed(String),
    ReceiveFailed(String),
}

fn main() -> Result<(), NetworkError> {
    println!("=== Tutorial 4: Network Programming ===\n");
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage:");
        println!("  Server: {} server [tcp_port] [udp_port]", args[0]);
        println!("  Client: {} client <server_address> <tcp_port> <username>", args[0]);
        return Ok(());
    }
    
    match args[1].as_str() {
        "server" => {
            let tcp_port = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8080);
            let udp_port = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(8081);
            
            let server = ChatServer::new(tcp_port, udp_port)?;
            server.run()?;
        }
        "client" => {
            if args.len() < 5 {
                println!("Usage: {} client <server> <tcp_port> <username>", args[0]);
                return Ok(());
            }
            
            let server = &args[2];
            let tcp_port: u16 = args[3].parse()
                .map_err(|_| NetworkError::ConnectionFailed("Invalid port".to_string()))?;
            let username = &args[4];
            
            let address = format!("{}:{}", server, tcp_port);
            let mut client = TcpChatClient::connect(&address, username)?;
            client.run()?;
        }
        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }
    
    Ok(())
}
```

### Tutorial 4 Summary

You've learned how to:

- ✅ Create TCP and UDP sockets
- ✅ Build a TCP server
- ✅ Implement TCP client communication
- ✅ Handle UDP message passing
- ✅ Build a multi-threaded chat application

## Tutorial 5: GUI Applications

This tutorial introduces GUI development in MultiOS.

### What You'll Learn

- Creating windows and controls
- Drawing graphics and text
- Handling user input events
- Building a simple calculator application

### Tutorial Application

Create `tutorials/tutorial-05-gui/src/main.rs`:

```rust
//! Tutorial 5: GUI Programming
//! 
//! This tutorial demonstrates:
//! - Creating windows and controls
//! - Drawing graphics and text
//! - Handling user input events
//! - Building a simple calculator application

use std::collections::HashMap;

/// Calculator application
struct Calculator {
    window: Window,
    display: TextBox,
    buttons: HashMap<String, Button>,
    current_value: String,
    operator: Option<char>,
    first_operand: f64,
}

/// GUI control types
trait Control {
    fn handle_event(&mut self, event: &Event) -> bool;
    fn draw(&self, graphics: &Graphics);
    fn get_bounds(&self) -> Rectangle;
    fn set_bounds(&mut self, bounds: Rectangle);
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
}

/// Window
struct Window {
    handle: WindowHandle,
    controls: Vec<Box<dyn Control>>,
    title: String,
    is_visible: bool,
}

impl Window {
    pub fn new(title: &str, x: i32, y: i32, width: i32, height: i32) -> Result<Self, GuiError> {
        // Create window using system calls
        let handle = create_window(x, y, width, height, title)?;
        
        Ok(Window {
            handle,
            controls: Vec::new(),
            title: title.to_string(),
            is_visible: true,
        })
    }
    
    pub fn add_control(&mut self, control: Box<dyn Control>) {
        self.controls.push(control);
    }
    
    pub fn show(&mut self) -> Result<(), GuiError> {
        show_window(self.handle)?;
        self.is_visible = true;
        Ok(())
    }
    
    pub fn hide(&mut self) -> Result<(), GuiError> {
        hide_window(self.handle)?;
        self.is_visible = false;
        Ok(())
    }
    
    pub fn run(&mut self) -> Result<(), GuiError> {
        let mut running = true;
        
        while running {
            // Process window messages
            let event = get_next_event()?;
            
            match event.event_type {
                EventType::Quit => {
                    running = false;
                }
                EventType::MouseClick => {
                    self.handle_mouse_click(event.x, event.y);
                }
                EventType::KeyPress => {
                    self.handle_key_press(event.key_code);
                }
                EventType::Paint => {
                    self.draw();
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn handle_mouse_click(&mut self, x: i32, y: i32) {
        let click_point = Point::new(x, y);
        
        // Find which control was clicked
        for control in &mut self.controls {
            if control.get_bounds().contains(click_point) {
                // Forward event to control
                let event = Event {
                    event_type: EventType::MouseClick,
                    x,
                    y,
                    key_code: 0,
                };
                control.handle_event(&event);
                break;
            }
        }
    }
    
    fn handle_key_press(&mut self, key_code: u32) {
        // Forward key events to controls that need them
        for control in &mut self.controls {
            let event = Event {
                event_type: EventType::KeyPress,
                x: 0,
                y: 0,
                key_code,
            };
            control.handle_event(&event);
        }
    }
    
    fn draw(&self) {
        let graphics = get_window_graphics(self.handle).unwrap();
        
        // Clear window
        graphics.clear(Color::WHITE).unwrap();
        
        // Draw all controls
        for control in &self.controls {
            control.draw(&graphics);
        }
        
        // Refresh window
        refresh_window(self.handle).unwrap();
    }
}

/// Text box control
struct TextBox {
    bounds: Rectangle,
    text: String,
    font: Font,
    is_focused: bool,
}

impl TextBox {
    pub fn new(bounds: Rectangle) -> Self {
        TextBox {
            bounds,
            text: String::new(),
            font: Font::default(),
            is_focused: false,
        }
    }
}

impl Control for TextBox {
    fn handle_event(&mut self, event: &Event) -> bool {
        match event.event_type {
            EventType::MouseClick => {
                self.is_focused = true;
                true
            }
            EventType::KeyPress => {
                if self.is_focused {
                    match event.key_code {
                        0x08 => { // Backspace
                            self.text.pop();
                            true
                        }
                        0x0D => { // Enter
                            // Handle enter key
                            true
                        }
                        _ => {
                            // Add character to text
                            if let Some(character) = key_code_to_char(event.key_code) {
                                self.text.push(character);
                            }
                            true
                        }
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    fn draw(&self, graphics: &Graphics) {
        // Draw text box background
        graphics.draw_rect(
            self.bounds,
            Some(Color::WHITE),
            Some(if self.is_focused { Color::BLUE } else { Color::GRAY })
        ).unwrap();
        
        // Draw text
        let text_color = if self.is_focused { Color::BLACK } else { Color::DARK_GRAY };
        let _ = graphics.draw_text(
            &self.text,
            Point::new(self.bounds.x + 5, self.bounds.y + 5),
            &self.font,
            text_color
        );
    }
    
    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    
    fn set_bounds(&mut self, bounds: Rectangle) {
        self.bounds = bounds;
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn set_visible(&mut self, _visible: bool) {
        // Not implemented in this simple example
    }
}

/// Button control
struct Button {
    bounds: Rectangle,
    text: String,
    font: Font,
    is_pressed: bool,
    on_click: Option<Box<dyn Fn()>>,
}

impl Button {
    pub fn new(bounds: Rectangle, text: &str) -> Self {
        Button {
            bounds,
            text: text.to_string(),
            font: Font::default(),
            is_pressed: false,
            on_click: None,
        }
    }
    
    pub fn set_on_click<F>(&mut self, callback: F)
    where
        F: 'static + Fn(),
    {
        self.on_click = Some(Box::new(callback));
    }
}

impl Control for Button {
    fn handle_event(&mut self, event: &Event) -> bool {
        match event.event_type {
            EventType::MouseClick => {
                self.is_pressed = true;
                
                // Call click handler
                if let Some(ref callback) = self.on_click {
                    callback();
                }
                
                true
            }
            _ => false,
        }
    }
    
    fn draw(&self, graphics: &Graphics) {
        let bg_color = if self.is_pressed { Color::GRAY } else { Color::LIGHT_GRAY };
        let border_color = Color::DARK_GRAY;
        
        // Draw button background
        graphics.draw_rect(self.bounds, Some(bg_color), Some(border_color)).unwrap();
        
        // Draw button text
        graphics.draw_text(
            &self.text,
            Point::new(
                self.bounds.x + self.bounds.width / 2 - (self.text.len() * 6) / 2,
                self.bounds.y + self.bounds.height / 2 - 8,
            ),
            &self.font,
            Color::BLACK
        ).unwrap();
    }
    
    fn get_bounds(&self) -> Rectangle {
        self.bounds
    }
    
    fn set_bounds(&mut self, bounds: Rectangle) {
        self.bounds = bounds;
    }
    
    fn is_visible(&self) -> bool {
        true
    }
    
    fn set_visible(&mut self, _visible: bool) {
        // Not implemented in this simple example
    }
}

/// Calculator implementation
impl Calculator {
    pub fn new() -> Result<Self, GuiError> {
        // Create main window
        let mut window = Window::new("MultiOS Calculator", 100, 100, 300, 400)?;
        
        // Create display
        let display_rect = Rectangle {
            x: 20,
            y: 20,
            width: 260,
            height: 60,
        };
        let mut display = TextBox::new(display_rect);
        window.add_control(Box::new(display.clone()));
        
        // Create buttons
        let mut buttons = HashMap::new();
        
        // Number buttons (0-9)
        for i in 0..=9 {
            let row = 3 - i / 3;
            let col = i % 3;
            let x = 20 + col * 70;
            let y = 100 + row * 70;
            
            let button_rect = Rectangle {
                x,
                y,
                width: 60,
                height: 60,
            };
            
            let mut button = Button::new(button_rect, &i.to_string());
            
            // Add number button handler
            let current_value = std::rc::Rc::new(std::cell::RefCell::new("".to_string()));
            let display_clone = display.clone();
            
            button.set_on_click(move || {
                let mut value = current_value.borrow_mut();
                value.push_str(&i.to_string());
                display_clone.text = value.clone();
            });
            
            window.add_control(Box::new(button.clone()));
            buttons.insert(i.to_string(), button);
        }
        
        // Operator buttons
        let operators = vec![('+', 310, 100), ('-', 310, 170), ('*', 310, 240), ('/', 310, 310)];
        
        for &(op, x, y) in &operators {
            let button_rect = Rectangle { x, y, width: 60, height: 60 };
            let mut button = Button::new(button_rect, &op.to_string());
            
            // Add operator handler
            let current_value = std::rc::Rc::new(std::cell::RefCell::new("".to_string()));
            let operator = std::rc::Rc::new(std::cell::RefCell::new(None));
            let display_clone = display.clone();
            
            button.set_on_click(move || {
                let value = current_value.borrow().clone();
                if let Ok(num) = value.parse::<f64>() {
                    *operator.borrow_mut() = Some(op);
                    display_clone.text = value;
                }
            });
            
            window.add_control(Box::new(button.clone()));
            buttons.insert(op.to_string(), button);
        }
        
        // Equals button
        let equals_rect = Rectangle { x: 310, y: 380, width: 60, height: 60 };
        let mut equals_button = Button::new(equals_rect, "=");
        
        let current_value = std::rc::Rc::new(std::cell::RefCell::new("".to_string()));
        let operator = std::rc::Rc::new(std::cell::RefCell::new(None));
        let display_clone = display.clone();
        
        equals_button.set_on_click(move || {
            let value = current_value.borrow().clone();
            if let Ok(num) = value.parse::<f64>() {
                if let Some(op) = *operator.borrow() {
                    let result = match op {
                        '+' => display_clone.text.parse::<f64>().unwrap_or(0.0) + num,
                        '-' => display_clone.text.parse::<f64>().unwrap_or(0.0) - num,
                        '*' => display_clone.text.parse::<f64>().unwrap_or(0.0) * num,
                        '/' => {
                            let divisor = display_clone.text.parse::<f64>().unwrap_or(1.0);
                            if divisor != 0.0 { num / divisor } else { 0.0 }
                        },
                        _ => 0.0,
                    };
                    display_clone.text = result.to_string();
                    *operator.borrow_mut() = None;
                }
            }
        });
        
        window.add_control(Box::new(equals_button));
        
        // Clear button
        let clear_rect = Rectangle { x: 20, y: 380, width: 130, height: 60 };
        let mut clear_button = Button::new(clear_rect, "Clear");
        
        let display_clone = display.clone();
        clear_button.set_on_click(move || {
            display_clone.text.clear();
        });
        
        window.add_control(Box::new(clear_button));
        
        Ok(Calculator {
            window,
            display,
            buttons,
            current_value: String::new(),
            operator: None,
            first_operand: 0.0,
        })
    }
    
    pub fn run(&mut self) -> Result<(), GuiError> {
        self.window.show()?;
        self.window.run()
    }
}

/// Event system
#[derive(Debug, Clone, Copy)]
struct Event {
    event_type: EventType,
    x: i32,
    y: i32,
    key_code: u32,
}

#[derive(Debug, Clone, Copy)]
enum EventType {
    Quit,
    MouseClick,
    MouseMove,
    KeyPress,
    Paint,
    Timer,
}

/// Geometry types
#[derive(Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy)]
struct Rectangle {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rectangle {
    fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }
}

/// Color definitions
#[derive(Debug, Clone, Copy)]
enum Color {
    WHITE,
    BLACK,
    RED,
    GREEN,
    BLUE,
    GRAY,
    LIGHT_GRAY,
    DARK_GRAY,
}

/// Font
struct Font {
    name: String,
    size: u32,
}

/// Window handle
struct WindowHandle {
    id: u32,
}

/// Graphics context
struct Graphics {
    handle: GraphicsHandle,
}

struct GraphicsHandle {
    id: u32,
}

/// Error types
#[derive(Debug)]
enum GuiError {
    WindowCreationFailed,
    InvalidBounds,
    GraphicsError,
}

/// GUI system calls (simplified)
fn create_window(x: i32, y: i32, width: i32, height: i32, title: &str) -> Result<WindowHandle, GuiError> {
    // This would use actual GUI system calls
    Ok(WindowHandle { id: 1 })
}

fn show_window(handle: WindowHandle) -> Result<(), GuiError> {
    // This would use actual GUI system calls
    Ok(())
}

fn hide_window(handle: WindowHandle) -> Result<(), GuiError> {
    // This would use actual GUI system calls
    Ok(())
}

fn get_next_event() -> Result<Event, GuiError> {
    // This would get the next event from the GUI system
    Ok(Event {
        event_type: EventType::Quit,
        x: 0,
        y: 0,
        key_code: 0,
    })
}

fn get_window_graphics(handle: WindowHandle) -> Result<Graphics, GuiError> {
    // This would get the graphics context for a window
    Ok(Graphics {
        handle: GraphicsHandle { id: handle.id }
    })
}

fn refresh_window(handle: WindowHandle) -> Result<(), GuiError> {
    // This would refresh a window
    Ok(())
}

impl Graphics {
    fn clear(&self, color: Color) -> Result<(), GuiError> {
        // This would clear the graphics context
        Ok(())
    }
    
    fn draw_rect(&self, rect: Rectangle, fill: Option<Color>, border: Option<Color>) -> Result<(), GuiError> {
        // This would draw a rectangle
        Ok(())
    }
    
    fn draw_text(&self, text: &str, position: Point, font: &Font, color: Color) -> Result<TextMetrics, GuiError> {
        // This would draw text
        Ok(TextMetrics { width: text.len() as i32, height: font.size as i32 })
    }
}

struct TextMetrics {
    width: i32,
    height: i32,
}

impl Font {
    fn default() -> Self {
        Font {
            name: "Default".to_string(),
            size: 12,
        }
    }
}

fn key_code_to_char(key_code: u32) -> Option<char> {
    // Convert key code to character
    // This would be a real implementation
    match key_code {
        0x41..=0x5A => Some(key_code as u8 as char), // A-Z
        0x30..=0x39 => Some(key_code as u8 as char), // 0-9
        _ => None,
    }
}

fn main() -> Result<(), GuiError> {
    println!("=== Tutorial 5: GUI Programming ===\n");
    
    // Initialize GUI system
    init_gui_system()?;
    
    // Create and run calculator
    let mut calculator = Calculator::new()?;
    calculator.run()?;
    
    Ok(())
}

fn init_gui_system() -> Result<(), GuiError> {
    // Initialize the GUI system
    // This would initialize the window manager, graphics system, etc.
    Ok(())
}
```

### Tutorial 5 Summary

You've learned how to:

- ✅ Create windows and controls
- ✅ Handle user input events (mouse, keyboard)
- ✅ Draw graphics and text
- ✅ Build a functional GUI application
- ✅ Event-driven programming model

## Summary

This tutorial series has covered the fundamentals of MultiOS development:

### Beginner Level ✅
1. **Your First Application** - Basic application structure and system calls
2. **System Calls** - Creating custom system calls and kernel communication
3. **File System Operations** - Working with files and directories
4. **Network Programming** - TCP/UDP communication and chat application
5. **GUI Applications** - Window management and event handling

### Next Steps

Continue with intermediate and advanced tutorials:

- [Tutorial 6: Device Drivers](#tutorial-6-device-drivers)
- [Tutorial 7: System Services](#tutorial-7-system-services)
- [Tutorial 8: Cross-Platform Development](#tutorial-8-cross-platform)
- [Tutorial 9: Memory Management](#tutorial-9-memory-management)
- [Tutorial 10: Thread and Process Management](#tutorial-10-threads-and-processes)

### Resources

- [API Reference](../api/README.md) - Complete API documentation
- [Developer Guide](../developer/README.md) - Development environment setup
- [Architecture Guide](../architecture/README.md) - System architecture details
- [Examples](../examples/) - Complete working examples

---

**Up**: [Documentation Index](../README.md)  
**Related**: [Developer Guide](../developer/README.md) | [API Reference](../api/README.md)