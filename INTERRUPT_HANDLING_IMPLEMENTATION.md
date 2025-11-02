# MultiOS Interrupt Handling and System Calls Implementation

## Overview

This document describes the comprehensive interrupt handling and system call infrastructure implemented for the MultiOS kernel, supporting multiple architectures (x86_64, ARM64, RISC-V).

## Architecture Support

### x86_64
- **IDT (Interrupt Descriptor Table)**: 256-entry interrupt descriptor table with gate descriptors
- **Interrupt Controllers**: 
  - Legacy PIC (8259A) for compatibility
  - Modern APIC (Advanced PIC) for multi-core systems
- **System Calls**: Using both `syscall/sysret` and legacy `int 0x80` mechanisms
- **Hardware Interrupts**: Timer, keyboard, serial ports, and other device interrupts

### ARM64 (AArch64)
- **Exception Vectors**: Full exception vector table for all exception levels (EL0-EL3)
- **Interrupt Controller**: GIC (Generic Interrupt Controller) v2/v3 support
- **System Calls**: Using `svc #0` (Supervisor Call) instruction
- **Privilege Levels**: EL0 (User), EL1 (Kernel), EL2 (Hypervisor), EL3 (Secure Monitor)

### RISC-V64
- **Trap Vectors**: Unified trap handling for interrupts and exceptions
- **Interrupt Controllers**: CLINT (Core Local Interruptor) and PLIC (Platform Level Interrupt Controller)
- **System Calls**: Using `ecall` (Environment Call) instruction
- **Privilege Levels**: User, Supervisor, Machine modes

## Interrupt Handling Infrastructure

### Core Components

1. **Interrupt Descriptor Tables (IDT)**
   - Architecture-specific interrupt table setup
   - Proper gate types (interrupt gates, trap gates, task gates)
   - Privilege level protection

2. **Interrupt Handlers**
   - Exception handlers (CPU exceptions like page faults, divide by zero)
   - Hardware interrupt handlers (timer, keyboard, devices)
   - System call handlers
   - Error handling and recovery

3. **Interrupt Controllers**
   - PIC/8259A for legacy systems
   - APIC for modern x86_64 multi-core systems
   - GIC for ARM64 systems
   - CLINT/PLIC for RISC-V systems

### Interrupt Types

1. **Exceptions (CPU Generated)**
   - Divide by zero (#DE)
   - Debug (#DB) 
   - Breakpoint (#BP)
   - Invalid opcode (#UD)
   - Page fault (#PF)
   - General protection fault (#GP)
   - Floating point exception (#MF)
   - Machine check (#MC)

2. **Hardware Interrupts (IRQ)**
   - Timer interrupts (IRQ 0)
   - Keyboard interrupts (IRQ 1)
   - Serial port interrupts
   - Disk controller interrupts
   - Network controller interrupts

3. **Software Interrupts**
   - System calls (syscall number in registers)
   - Debug breakpoints
   - Performance monitoring interrupts

### Interrupt Handling Process

1. **Hardware Interrupt Occurs**
   - CPU receives interrupt signal
   - Current context is saved
   - Interrupt vector is looked up in IDT
   - Handler is called

2. **Handler Execution**
   - Interrupt controller is acknowledged (EOI)
   - Hardware is serviced
   - Context switching if needed
   - Return from interrupt

3. **Context Switching**
   - Save current thread state
   - Select next thread to run
   - Restore next thread state
   - Return to new context

## System Call Interface

### System Call Numbers

The system call interface provides a comprehensive set of operations:

#### Process Management
- `PROCESS_CREATE` - Create new process
- `PROCESS_EXIT` - Exit current process
- `PROCESS_WAIT` - Wait for process termination
- `PROCESS_GETPID` - Get process ID
- `PROCESS_GETPPID` - Get parent process ID

#### Thread Management
- `THREAD_CREATE` - Create new thread
- `THREAD_EXIT` - Exit current thread
- `THREAD_JOIN` - Wait for thread to complete
- `THREAD_YIELD` - Yield CPU to other threads
- `THREAD_GETTID` - Get thread ID
- `THREAD_SET_PRIORITY` - Set thread priority
- `THREAD_GET_PRIORITY` - Get thread priority

#### Memory Management
- `VIRTUAL_ALLOC` - Allocate virtual memory
- `VIRTUAL_FREE` - Free virtual memory
- `VIRTUAL_MAP` - Map memory region
- `VIRTUAL_UNMAP` - Unmap memory region
- `PHYSICAL_ALLOC` - Allocate physical memory
- `PHYSICAL_FREE` - Free physical memory

#### File and I/O Operations
- `FILE_OPEN` - Open file
- `FILE_CLOSE` - Close file
- `FILE_READ` - Read from file
- `FILE_WRITE` - Write to file
- `FILE_SEEK` - Seek in file
- `FILE_STAT` - Get file statistics

#### Inter-Process Communication
- `IPC_SEND` - Send IPC message
- `IPC_RECEIVE` - Receive IPC message
- `IPC_POLL` - Poll for IPC events
- `MESSAGE_QUEUE_CREATE` - Create message queue
- `MESSAGE_QUEUE_SEND` - Send to message queue
- `MESSAGE_QUEUE_RECEIVE` - Receive from message queue

#### Synchronization
- `MUTEX_CREATE` - Create mutex
- `MUTEX_LOCK` - Lock mutex
- `MUTEX_UNLOCK` - Unlock mutex
- `CONDITION_CREATE` - Create condition variable
- `CONDITION_WAIT` - Wait on condition
- `CONDITION_SIGNAL` - Signal condition
- `SEMAPHORE_CREATE` - Create semaphore
- `SEMAPHORE_WAIT` - Wait on semaphore
- `SEMAPHORE_POST` - Post to semaphore

#### Device I/O
- `DEVICE_OPEN` - Open device
- `DEVICE_CLOSE` - Close device
- `DEVICE_READ` - Read from device
- `DEVICE_WRITE` - Write to device
- `DEVICE_IOCTL` - Device I/O control
- `INTERRUPT_REGISTER` - Register interrupt handler
- `INTERRUPT_UNREGISTER` - Unregister interrupt handler

#### System Information
- `SYSTEM_INFO` - Get system information
- `MEMORY_INFO` - Get memory information
- `CPU_INFO` - Get CPU information
- `TIME_GET` - Get system time
- `TIME_SET` - Set system time
- `CLOCK_GETTIME` - Get clock time

#### Debug and Monitoring
- `DEBUG_SET_BREAKPOINT` - Set debug breakpoint
- `DEBUG_REMOVE_BREAKPOINT` - Remove debug breakpoint
- `PROFILING_START` - Start performance profiling
- `PROFILING_STOP` - Stop performance profiling
- `TRACE_MARKER` - Add trace marker

### Parameter Validation

The system call interface includes comprehensive parameter validation:

1. **Pointer Validation**
   - Check for null pointers
   - Validate memory region access
   - Ensure proper alignment
   - Verify privilege level permissions

2. **Range Validation**
   - Check integer ranges
   - Validate buffer sizes
   - Ensure within acceptable limits

3. **Privilege Checking**
   - Verify caller has required permissions
   - Check user/kernel space boundaries
   - Validate resource access rights

### Thread Safety

The interrupt handling system is designed to be thread-safe:

1. **Interrupt Context Protection**
   - Critical sections protected by interrupt disabling
   - Atomic operations for shared data
   - Lock-free data structures where possible

2. **System Call Protection**
   - Parameter validation in all system calls
   - Proper error handling and reporting
   - Prevention of deadlock situations

3. **Resource Management**
   - Proper cleanup of resources
   - Prevention of resource leaks
   - Graceful handling of error conditions

## Implementation Details

### x86_64 Specific

#### IDT Structure
```rust
#[repr(C)]
struct IdtEntry {
    offset_low: u16,     // Offset bits 0..15
    selector: u16,       // Code segment selector
    ist_index: u8,       // Interrupt Stack Table index
    type_attr: u8,       // Type and attributes
    offset_mid: u16,     // Offset bits 16..31
    offset_high: u32,    // Offset bits 32..63
    zero: u16,           // Reserved
}
```

#### System Call Convention
- **syscall/sysret**: Fast system calls using CPU instructions
- **int 0x80**: Legacy system calls for compatibility
- **Register usage**: 
  - RAX: system call number
  - RDI, RSI, RDX, R10, R8, R9: arguments
  - RAX: return value
  - Carry flag: error indication

#### APIC Configuration
- Local APIC for each CPU core
- I/O APIC for interrupt routing
- Timer configuration for scheduling
- Interrupt priority handling

### ARM64 Specific

#### Exception Vectors
- **EL0 with SP0**: User mode exceptions
- **EL1 with SP1**: Kernel mode exceptions
- **Lower EL AArch64**: Exceptions from lower privilege levels
- **Lower EL AArch32**: Legacy 32-bit mode exceptions

#### System Call Convention
- **svc #0**: Supervisor call instruction
- **Register usage**:
  - x0-x5: arguments
  - x8: system call number
  - x0: return value
  - x18: thread ID (if needed)

#### GIC Configuration
- GICv3 redistributor setup
- Interrupt routing configuration
- Priority handling
- CPU interface setup

### RISC-V Specific

#### Trap Vector
- **mtvec**: Machine trap vector register
- Handles both interrupts and exceptions
- Exception codes in mcause register
- Trap handler address in mtvec

#### System Call Convention
- **ecall**: Environment call instruction
- **Register usage**:
  - a0-a5: arguments
  - a7: system call number
  - a0: return value

#### CLINT/PLIC Configuration
- CLINT for timer and software interrupts
- PLIC for external interrupt routing
- Interrupt priority configuration
- CPU-specific interrupt handling

## Error Handling

### Exception Handling
1. **Page Faults**
   - Invalid memory access detection
   - Page table walk for page allocation
   - Kill process on invalid access
   
2. **Protection Faults**
   - Privilege level violations
   - Invalid memory access
   - Resource access violations
   
3. **Machine Check**
   - Hardware errors
   - Memory parity errors
   - CPU cache errors

### System Call Error Handling
1. **Parameter Validation**
   - Null pointer checks
   - Range validation
   - Privilege verification
   
2. **Resource Management**
   - Memory allocation failures
   - File handle exhaustion
   - Process/thread limits

3. **Error Codes**
   - Standardized error codes
   - Detailed error information
   - Debug information for development

## Performance Considerations

### Interrupt Latency
1. **Fast Interrupt Handling**
   - Minimal handler execution time
   - Immediate acknowledgment
   - Efficient context switching

2. **Interrupt Coalescing**
   - Combine multiple interrupts
   - Reduce interrupt overhead
   - Improve throughput

### System Call Performance
1. **Fast System Calls**
   - Use of architecture-specific instructions
   - Minimal parameter copying
   - Efficient error handling

2. **Parameter Marshalling**
   - Direct register usage
   - Minimal copying overhead
   - Efficient data validation

## Security Features

### Privilege Separation
1. **User/Kernel Boundary**
   - Strict privilege level enforcement
   - Parameter validation
   - Memory protection

2. **System Call Protection**
   - Parameter bounds checking
   - Resource access validation
   - Permission verification

### Interrupt Protection
1. **Interrupt Masking**
   - Critical section protection
   - Nested interrupt handling
   - Priority-based masking

2. **Context Protection**
   - Stack overflow protection
   - Register preservation
   - Memory access validation

## Testing and Debugging

### Interrupt Testing
1. **Hardware Interrupt Simulation**
   - Timer interrupt testing
   - Device interrupt simulation
   - Exception simulation

2. **System Call Testing**
   - Parameter validation testing
   - Error condition testing
   - Performance testing

### Debug Features
1. **Interrupt Debugging**
   - Interrupt trace logging
   - Handler timing measurement
   - Exception debugging

2. **System Call Debugging**
   - System call tracing
   - Parameter logging
   - Error debugging

## Future Enhancements

### Interrupt Handling
1. **Advanced Interrupt Controllers**
   - MSI (Message Signaled Interrupts)
   - Interrupt remapping
   - Virtual interrupt support

2. **Interrupt Optimization**
   - Adaptive interrupt coalescing
   - Load-based interrupt balancing
   - Dynamic interrupt priorities

### System Call Interface
1. **Extended System Calls**
   - Asynchronous I/O system calls
   - Advanced synchronization primitives
   - Virtual memory operations

2. **Performance Optimization**
   - System call batching
   - Zero-copy parameter passing
   - Cache-friendly data structures

This comprehensive interrupt handling and system call implementation provides a robust foundation for the MultiOS kernel across multiple architectures while maintaining security, performance, and reliability.