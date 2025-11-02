#!/usr/bin/env python3
"""
MultiOS Process and Scheduler Analysis Utilities for GDB
Provides advanced process debugging capabilities for MultiOS kernel development
"""

import gdb
import struct

class MultiOSProcessAnalyzer:
    """Advanced process and scheduler analysis utilities"""
    
    def __init__(self):
        self.arch = self.get_architecture()
    
    def get_architecture(self):
        """Determine current architecture"""
        try:
            arch_str = gdb.selected_frame().architecture().name()
            if "x86_64" in arch_str or "i386:x86-64" in arch_str:
                return "x86_64"
            elif "aarch64" in arch_str:
                return "aarch64"
            elif "riscv" in arch_str:
                return "riscv64"
            else:
                return "unknown"
        except:
            return "unknown"
    
    def get_register(self, reg_name):
        """Get register value safely"""
        try:
            return gdb.parse_and_eval(f"${reg_name}")
        except:
            return None
    
    def print_current_task_info(self):
        """Print information about currently executing task"""
        print("Current Task Information")
        print("=" * 30)
        
        try:
            # Try to get current task from scheduler
            print("Attempting to identify current task...")
            
            # Architecture-specific register analysis
            if self.arch == "x86_64":
                self.print_x86_64_task_info()
            elif self.arch == "aarch64":
                self.print_aarch64_task_info()
            elif self.arch == "riscv64":
                self.print_riscv64_task_info()
            
        except Exception as e:
            print(f"Error analyzing current task: {e}")
    
    def print_x86_64_task_info(self):
        """Print x86_64-specific task information"""
        print("x86_64 Task Context:")
        
        # Print relevant registers
        regs = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "rip"]
        
        for reg in regs:
            value = self.get_register(reg)
            if value is not None:
                print(f"  {reg.upper()}: 0x{int(value):016x}")
        
        # Check for task state segment info if available
        print("\nNote: Full task information requires scheduler data structures")
    
    def print_aarch64_task_info(self):
        """Print ARM64-specific task information"""
        print("ARM64 Task Context:")
        
        # Print relevant registers
        regs = ["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12", "x13", "x14", "x15"]
        
        for i, reg in enumerate(regs):
            value = self.get_register(reg)
            if value is not None:
                print(f"  {reg}: 0x{int(value):016x}", end="")
                if (i + 1) % 4 == 0:
                    print()
                else:
                    print(" ", end="")
        print()
        
        print(f"  sp: 0x{int(self.get_register('sp')):016x}")
        print(f"  pc: 0x{int(self.get_register('pc')):016x}")
    
    def print_riscv64_task_info(self):
        """Print RISC-V-specific task information"""
        print("RISC-V Task Context:")
        
        # Print general purpose registers
        regs = [f"x{i}" for i in range(32)]
        
        for i, reg in enumerate(regs):
            value = self.get_register(reg)
            if value is not None:
                reg_names = {
                    "x0": "zero", "x1": "ra", "x2": "sp", "x3": "gp", "x4": "tp",
                    "x5": "t0", "x6": "t1", "x7": "t2", "x8": "s0", "x9": "s1"
                }
                name = reg_names.get(reg, reg)
                print(f"  {name:5s}({reg:3s}): 0x{int(value):016x}", end="")
                if (i + 1) % 4 == 0:
                    print()
                else:
                    print(" ", end="")
        print()
        
        pc = self.get_register("pc")
        if pc is not None:
            print(f"  pc: 0x{int(pc):016x}")
    
    def print_scheduler_state(self):
        """Print current scheduler state"""
        print("Scheduler State Analysis")
        print("=" * 30)
        
        try:
            print("Current scheduler implementation:")
            print("  Type: Round-robin (initial implementation)")
            print("  Features: Preemptive, priority-based (planned)")
            print("  Time quantum: Configurable")
            
            print("\nScheduler state requires access to:")
            print("  - Ready queue (processes ready to run)")
            print("  - Current running process")
            print("  - Timer interrupt handler")
            print("  - Process control blocks (PCBs)")
            
            print("\nKey scheduler operations to analyze:")
            print("  - Context switching")
            print("  - Process creation/destruction")
            print("  - Interrupt handling")
            print("  - Memory management integration")
            
        except Exception as e:
            print(f"Error analyzing scheduler state: {e}")
    
    def analyze_context_switch(self):
        """Analyze context switching mechanism"""
        print("Context Switch Analysis")
        print("=" * 25)
        
        try:
            print("Context switch components:")
            print("  1. Save current process state")
            print("  2. Load next process state")
            print("  3. Update memory management context")
            print("  4. Jump to next process")
            
            if self.arch == "x86_64":
                print("\nx86_64 context switch considerations:")
                print("  - TSS (Task State Segment) usage")
                print("  - Ring transitions")
                print("  - CR3 switching for memory isolation")
                
            elif self.arch == "aarch64":
                print("\nARM64 context switch considerations:")
                print("  - EL transitions")
                print("  - System register state saving")
                print("  - ASID management")
                
            elif self.arch == "riscv64":
                print("\nRISC-V context switch considerations:")
                print("  - Privilege level transitions")
                print("  - PMP (Physical Memory Protection) context")
                print("  - CSR (Control and Status Register) management")
            
        except Exception as e:
            print(f"Error analyzing context switch: {e}")
    
    def analyze_interrupt_handling(self):
        """Analyze interrupt handling mechanisms"""
        print("Interrupt Handling Analysis")
        print("=" * 30)
        
        try:
            if self.arch == "x86_64":
                print("x86_64 Interrupt Handling:")
                print("  - IDT (Interrupt Descriptor Table)")
                print("  - PIC/APIC (Programmable Interrupt Controller)")
                print("  - Exception handling")
                print("  - Interrupt vectors")
                
            elif self.arch == "aarch64":
                print("ARM64 Interrupt Handling:")
                print("  - Exception levels (EL0-EL3)")
                print("  - Vector tables")
                print("  - GIC (Generic Interrupt Controller)")
                print("  - SError, IRQ, FIQ handling")
                
            elif self.arch == "riscv64":
                print("RISC-V Interrupt Handling:")
                print("  - M-mode/S-mode interrupts")
                print("  - PLIC (Platform-Level Interrupt Controller)")
                print("  - Timer interrupts")
                print("  - Software interrupts")
            
            print("\nScheduler integration points:")
            print("  - Timer interrupt → scheduler invocation")
            print("  - System calls → process management")
            print("  - I/O completion → process wakeup")
            
        except Exception as e:
            print(f"Error analyzing interrupt handling: {e}")
    
    def analyze_process_creation(self):
        """Analyze process creation mechanism"""
        print("Process Creation Analysis")
        print("=" * 25)
        
        try:
            print("Process creation steps:")
            print("  1. Allocate process control block (PCB)")
            print("  2. Create memory context")
            print("  3. Set up file descriptor table")
            print("  4. Initialize CPU context")
            print("  5. Add to scheduler queues")
            print("  6. Return new process ID")
            
            print("\nProcess ID allocation:")
            print("  - Unique identifier for each process")
            print("  - Used for process management")
            print("  - Reused after process termination")
            
            print("\nMemory allocation:")
            print("  - Code segment")
            print("  - Data segment")
            print("  - Stack allocation")
            print("  - Heap initialization")
            
        except Exception as e:
            print(f"Error analyzing process creation: {e}")
    
    def analyze_ipc_mechanisms(self):
        """Analyze Inter-Process Communication"""
        print("IPC (Inter-Process Communication) Analysis")
        print("=" * 40)
        
        try:
            print("IPC mechanisms in MultiOS:")
            print("  1. Message passing (microkernel foundation)")
            print("  2. Shared memory (with proper isolation)")
            print("  3. Signals/notifications")
            print("  4. Synchronization primitives")
            
            print("\nMessage passing design:")
            print("  - Primary IPC mechanism")
            print("  - Based on capability system")
            print("  - Supports both synchronous and asynchronous")
            
            print("\nSecurity considerations:")
            print("  - Capability-based access control")
            print("  - Process isolation")
            print("  - Minimal trusted computing base")
            
        except Exception as e:
            print(f"Error analyzing IPC: {e}")
    
    def analyze_memory_management_integration(self):
        """Analyze how memory management integrates with processes"""
        print("Memory Management Integration")
        print("=" * 30)
        
        try:
            print("Memory management integration points:")
            print("  1. Process address space isolation")
            print("  2. Page table management")
            print("  3. Memory allocation/deallocation")
            print("  4. Memory-mapped I/O")
            print("  5. Copy-on-write mechanisms")
            
            if self.arch == "x86_64":
                print("\nx86_64 Memory Management:")
                print("  - CR3 register for page directory")
                print("  - PAE for extended physical addressing")
                print("  - SMEP/SMAP for security")
                
            elif self.arch == "aarch64":
                print("\nARM64 Memory Management:")
                print("  - TTBR0/TTBR1 for translation")
                print("  - ASID for TLB optimization")
                print("  - Hardware page table walkers")
                
            elif self.arch == "riscv64":
                print("\nRISC-V Memory Management:")
                print("  - SATP for translation control")
                print("  - PMP for physical memory protection")
                print("  - Sv39/Sv48 virtual memory schemes")
            
        except Exception as e:
            print(f"Error analyzing memory integration: {e}")

class MultiOSProcessCommand(gdb.Command):
    """MultiOS process analysis command"""
    
    def __init__(self):
        super(MultiOSProcessCommand, self).__init__("multios-process", gdb.COMMAND_USER)
    
    def invoke(self, arg, from_tty):
        analyzer = MultiOSProcessAnalyzer()
        
        args = arg.split() if arg else []
        
        if not args or "current" in args:
            analyzer.print_current_task_info()
        elif "scheduler" in args:
            analyzer.print_scheduler_state()
        elif "context" in args:
            analyzer.analyze_context_switch()
        elif "interrupt" in args:
            analyzer.analyze_interrupt_handling()
        elif "creation" in args:
            analyzer.analyze_process_creation()
        elif "ipc" in args:
            analyzer.analyze_ipc_mechanisms()
        elif "memory" in args:
            analyzer.analyze_memory_management_integration()
        elif "help" in args:
            self.print_help()
        else:
            print("Usage: multios-process [current|scheduler|context|interrupt|creation|ipc|memory|help]")
            self.print_help()
    
    def print_help(self):
        print("\nMultiOS Process Analysis Commands:")
        print("  multios-process current    - Analyze current task")
        print("  multios-process scheduler  - Show scheduler state")
        print("  multios-process context    - Analyze context switching")
        print("  multios-process interrupt  - Analyze interrupt handling")
        print("  multios-process creation   - Analyze process creation")
        print("  multios-process ipc        - Analyze IPC mechanisms")
        print("  multios-process memory     - Analyze memory integration")
        print("  multios-process help       - Show this help")

# Register the command
try:
    MultiOSProcessCommand()
    print("MultiOS process analysis utilities loaded")
except Exception as e:
    print(f"Error loading MultiOS process utilities: {e}")

# Performance analysis helpers
class MultiOSPerformanceAnalyzer:
    """Performance analysis utilities"""
    
    def analyze_timing(self):
        """Analyze timing and performance"""
        print("Performance Analysis")
        print("=" * 25)
        
        try:
            # This would require access to performance counters
            print("Key performance metrics to monitor:")
            print("  - Context switch overhead")
            print("  - Memory allocation latency")
            print("  - Interrupt response time")
            print("  - IPC message passing overhead")
            
            print("\nArchitecture-specific performance considerations:")
            arch = gdb.selected_frame().architecture().name()
            if "x86_64" in arch:
                print("  - TSC (Time Stamp Counter) usage")
                print("  - Cache effects")
                print("  - Branch prediction")
            elif "aarch64" in arch:
                print("  - Performance monitoring units")
                print("  - Cache behavior")
                print("  - Exception overhead")
            elif "riscv" in arch:
                print("  - Performance counters")
                print("  - CSR access overhead")
                
        except Exception as e:
            print(f"Error in performance analysis: {e}")

class MultiOSPerformanceCommand(gdb.Command):
    """MultiOS performance analysis command"""
    
    def __init__(self):
        super(MultiOSPerformanceCommand, self).__init__("multios-performance", gdb.COMMAND_USER)
    
    def invoke(self, arg, from_tty):
        analyzer = MultiOSPerformanceAnalyzer()
        
        if arg == "timing":
            analyzer.analyze_timing()
        else:
            print("Usage: multios-performance timing")

try:
    MultiOSPerformanceCommand()
    print("MultiOS performance analysis utilities loaded")
except Exception as e:
    print(f"Error loading MultiOS performance utilities: {e}")

print("MultiOS GDB process analysis script loaded successfully")
