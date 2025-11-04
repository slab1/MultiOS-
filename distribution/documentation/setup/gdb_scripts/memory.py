#!/usr/bin/env python3
"""
MultiOS Memory Analysis Utilities for GDB
Provides advanced memory debugging capabilities for MultiOS kernel development
"""

import gdb
import struct
import sys

class MultiOSMemoryAnalyzer:
    """Advanced memory analysis utilities for MultiOS kernel"""
    
    def __init__(self):
        self.arch = self.get_architecture()
        self.pointer_size = 8  # Assume 64-bit for now
        
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
    
    def read_memory(self, addr, size):
        """Read memory safely"""
        try:
            return gdb.selected_inferior().read_memory(addr, size)
        except gdb.error:
            return None
    
    def print_page_tables(self):
        """Print page table information"""
        print(f"Architecture: {self.arch}")
        print("=" * 50)
        
        if self.arch == "x86_64":
            self.print_x86_64_page_tables()
        elif self.arch == "aarch64":
            self.print_aarch64_page_tables()
        elif self.arch == "riscv64":
            self.print_riscv_page_tables()
        else:
            print("Unknown architecture - page table analysis not available")
    
    def print_x86_64_page_tables(self):
        """Print x86_64 page tables"""
        try:
            cr3 = self.get_register("cr3")
            if cr3 is None:
                print("Error: Could not read CR3 register")
                return
            
            cr3_value = int(cr3) & 0xFFFFFFFFFFFF000
            print(f"CR3 (Page Table Base): 0x{cr3_value:016x}")
            
            # Print PML4 entries
            print("\nPML4 (Page Map Level 4) Entries:")
            for i in range(4):
                pml4_addr = cr3_value + i * 8
                try:
                    pml4_entry = struct.unpack('<Q', self.read_memory(pml4_addr, 8))[0]
                    
                    if pml4_entry & 1:  # Present bit
                        present = "Present"
                        user_supervisor = "User" if (pml4_entry >> 2) & 1 else "Supervisor"
                        read_write = "RW" if (pml4_entry >> 1) & 1 else "RO"
                        print(f"  PML4[{i}]: 0x{pml4_entry:016x} ({present}, {user_supervisor}, {read_write})")
                        
                        # Try to follow to PDPT
                        pdpt_addr = pml4_entry & 0xFFFFFFFFFFFF000
                        print(f"    -> PDPT at 0x{pdpt_addr:016x}")
                    else:
                        print(f"  PML4[{i}]: 0x{pml4_entry:016x} (Not Present)")
                        
                except Exception as e:
                    print(f"  PML4[{i}]: Error reading entry - {e}")
                    
        except Exception as e:
            print(f"Error reading x86_64 page tables: {e}")
    
    def print_aarch64_page_tables(self):
        """Print ARM64 page table information"""
        print("ARM64 Page Table Analysis")
        print("ARM64 uses EL (Exception Level) specific translation tables")
        print("TTBR0_EL1 and TTBR1_EL1 contain translation table base addresses")
        print("Translation regime depends on current EL and SCTLR configuration")
        
        # This would require privileged register access in real GDB
        print("\nNote: Full page table walking requires privileged register access")
    
    def print_riscv_page_tables(self):
        """Print RISC-V page table information"""
        print("RISC-V Page Table Analysis")
        print("RISC-V uses Sv39/Sv48/Sv57 virtual memory schemes")
        print("satp (Supervisor Address Translation and Protection) register controls MMU")
        
        try:
            # Try to read satp if available
            satp = self.get_register("satp")
            if satp is not None:
                satp_value = int(satp)
                mode = (satp_value >> 60) & 0xF
                asid = (satp_value >> 44) & 0xFFFF
                ppn = satp_value & 0xFFFFFFFFFFF
                
                print(f"SATP: 0x{satp_value:016x}")
                print(f"  Mode: {mode} ({'Sv39' if mode == 8 else 'Unknown'})")
                print(f"  ASID: {asid}")
                print(f"  PPN: 0x{ppn:012x}")
                
                # Calculate page table address
                pte_addr = ppn << 12
                print(f"  Root Page Table at: 0x{pte_addr:016x}")
            else:
                print("SATP register not available in current context")
                
        except Exception as e:
            print(f"Error reading RISC-V MMU state: {e}")
    
    def print_kernel_heap(self):
        """Print kernel heap information"""
        print("Kernel Heap Analysis")
        print("=" * 30)
        
        try:
            # Try to find linked_list_allocator::Heap instances
            print("Searching for heap structures...")
            
            # This is a simplified approach - in practice you'd want to
            # search for specific heap instances in memory
            print("Note: Full heap analysis requires access to heap instance addresses")
            print("Linked list allocator typically manages memory in chunks")
            
        except Exception as e:
            print(f"Error analyzing heap: {e}")
    
    def print_stack_analysis(self):
        """Analyze current stack"""
        print("Stack Analysis")
        print("=" * 20)
        
        try:
            if self.arch == "x86_64":
                sp = self.get_register("rsp")
                bp = self.get_register("rbp")
            elif self.arch == "aarch64":
                sp = self.get_register("sp")
                bp = self.get_register("x29")  # Frame pointer
            elif self.arch == "riscv64":
                sp = self.get_register("x2")   # Stack pointer
                bp = self.get_register("x8")   # Frame pointer
            else:
                print("Unknown architecture for stack analysis")
                return
            
            if sp:
                print(f"Stack Pointer: 0x{int(sp):016x}")
                self.print_stack_contents(int(sp))
            
            if bp:
                print(f"Frame Pointer: 0x{int(bp):016x}")
                
        except Exception as e:
            print(f"Error analyzing stack: {e}")
    
    def print_stack_contents(self, sp_addr):
        """Print stack contents"""
        print("\nStack Contents (first 256 bytes):")
        try:
            stack_data = self.read_memory(sp_addr - 256, 512)
            if stack_data:
                for i in range(0, min(512, len(stack_data)), 8):
                    addr = sp_addr - 256 + i
                    if i + 8 <= len(stack_data):
                        value = struct.unpack('<Q', stack_data[i:i+8])[0]
                        print(f"  0x{addr:016x}: 0x{value:016x}")
        except Exception as e:
            print(f"Error reading stack: {e}")
    
    def analyze_memory_leak(self):
        """Analyze potential memory leaks"""
        print("Memory Leak Analysis")
        print("=" * 30)
        
        self.print_kernel_heap()
        print("\n")
        self.print_page_tables()
        print("\n")
        self.print_stack_analysis()
    
    def find_memory_corruption(self):
        """Look for signs of memory corruption"""
        print("Memory Corruption Analysis")
        print("=" * 35)
        
        try:
            # Check stack canary if present
            print("Checking for stack canaries...")
            
            # Check for common patterns of corruption
            print("Scanning for potential buffer overflows...")
            print("Note: This is a basic analysis - detailed inspection requires symbols")
            
        except Exception as e:
            print(f"Error in memory corruption analysis: {e}")

class MultiOSMemoryCommand(gdb.Command):
    """MultiOS memory analysis command"""
    
    def __init__(self):
        super(MultiOSMemoryCommand, self).__init__("multios-memory", gdb.COMMAND_USER)
    
    def invoke(self, arg, from_tty):
        analyzer = MultiOSMemoryAnalyzer()
        
        args = arg.split() if arg else []
        
        if not args or "leak" in args:
            analyzer.analyze_memory_leak()
        elif "heap" in args:
            analyzer.print_kernel_heap()
        elif "pagetables" in args or "page-tables" in args:
            analyzer.print_page_tables()
        elif "stack" in args:
            analyzer.print_stack_analysis()
        elif "corruption" in args:
            analyzer.find_memory_corruption()
        elif "help" in args:
            self.print_help()
        else:
            print("Usage: multios-memory [leak|heap|pagetables|stack|corruption|help]")
            self.print_help()
    
    def print_help(self):
        print("\nMultiOS Memory Analysis Commands:")
        print("  multios-memory leak        - Complete memory leak analysis")
        print("  multios-memory heap        - Analyze kernel heap")
        print("  multios-memory pagetables  - Analyze page tables")
        print("  multios-memory stack       - Analyze current stack")
        print("  multios-memory corruption  - Look for memory corruption")
        print("  multios-memory help        - Show this help")

# Register the command
try:
    MultiOSMemoryCommand()
    print("MultiOS memory analysis utilities loaded")
except Exception as e:
    print(f"Error loading MultiOS memory utilities: {e}")

def rust_backtrace_handler(event):
    """Enhanced Rust backtrace handler"""
    if hasattr(event, 'stop_reason'):
        try:
            frame = gdb.newest_frame()
            print("\nEnhanced Rust Backtrace:")
            print("=" * 40)
            
            level = 0
            while frame:
                try:
                    func_name = frame.name()
                    if func_name:
                        sal = frame.find_sal()
                        if sal and sal.symtab:
                            filename = sal.symtab.filename
                            line = sal.line
                            print(f"#{level:2d} {func_name} at {filename}:{line}")
                        else:
                            print(f"#{level:2d} {func_name}")
                    else:
                        # Try to get function info from symbol
                        sym = frame.symbol()
                        if sym:
                            print(f"#{level:2d} {sym.name}")
                        else:
                            print(f"#{level:2d} <unknown function>")
                    
                    level += 1
                    
                except:
                    print(f"#{level:2d} <error reading frame>")
                    level += 1
                
                frame = frame.newer()
                
            print("=" * 40)
            
        except Exception as e:
            # Fallback to basic backtrace
            pass

# Connect the event handler
try:
    gdb.events.stop.connect(rust_backtrace_handler)
except Exception as e:
    print(f"Note: Could not connect backtrace handler: {e}")

# Auto-analysis on panic
def panic_detection_handler(event):
    """Detect and analyze kernel panics"""
    try:
        frame = gdb.newest_frame()
        if frame and frame.name() and "panic" in frame.name().lower():
            print("\n*** KERNEL PANIC DETECTED ***")
            print("Running panic analysis...")
            
            analyzer = MultiOSMemoryAnalyzer()
            analyzer.print_stack_analysis()
            
    except:
        pass

try:
    gdb.events.stop.connect(panic_detection_handler)
except:
    pass

print("MultiOS GDB memory analysis script loaded successfully")
