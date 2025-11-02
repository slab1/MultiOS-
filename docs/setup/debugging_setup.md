# MultiOS Development Tools and Debugging Setup

## Overview

This document provides comprehensive setup instructions for debugging tools, IDE configuration, and troubleshooting guides for MultiOS development. The setup covers cross-platform debugging for x86_64, ARM64, and RISC-V targets with a focus on educational value and ease of use.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [GDB Debugging Setup](#gdb-debugging-setup)
3. [QEMU Monitor Configuration](#qemu-monitor-configuration)
4. [Serial Console Setup](#serial-console-setup)
5. [VS Code Configuration](#vs-code-configuration)
6. [Debugging Guides](#debugging-guides)
7. [Troubleshooting](#troubleshooting)
8. [Architecture-Specific Notes](#architecture-specific-notes)

## Prerequisites

### Required Tools

Install the following tools on your development machine:

#### Core Tools
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cross-compilation tool
cargo install cross

# Install QEMU (all architectures)
sudo apt update
sudo apt install -y qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64

# Install GDB with Python support
sudo apt install -y gdb-multiarch

# Install additional debugging tools
sudo apt install -y opensbi u-boot-tools
```

#### Cargo Configuration

Create or update your `.cargo/config.toml` in the project root:

```toml
[alias]
# Convenient aliases for common operations
test-arch = "test --target"
run-arch = "run --target"
debug-arch = "build --target"
build-all = "build --workspace"
test-all = "test --workspace"

[target.x86_64-unknown-none-elf]
runner = "qemu-system-x86_64 -kernel"
rustflags = ["-C", "link-arg=-Tkernel/x86_64/linker.ld"]

[target.aarch64-unknown-none-elf]
runner = "qemu-system-aarch64 -machine virt -cpu cortex-a57"
rustflags = ["-C", "link-arg=-Tkernel/aarch64/linker.ld"]

[target.riscv64gc-unknown-none-elf]
runner = "qemu-system-riscv64 -machine virt -kernel"
rustflags = ["-C", "link-arg=-Tkernel/riscv64/linker.ld"]
```

## GDB Debugging Setup

### Multi-Architecture GDB Configuration

Create a comprehensive GDB setup for all supported architectures:

#### Global GDB Configuration (`~/.gdbinit`)

```gdb
# MultiOS Global GDB Configuration

# Enable Python scripting for better debugging
python
import gdb
import sys
sys.path.insert(0, '/workspace/docs/setup/gdb_scripts')
end

# Set pagination off for large outputs
set pagination off
set confirm off

# Default settings
set print pretty on
set print object on
set print static-members on
set print vtbl on
set print demangle on
set demangle-style gnu-v3

# Better display for Rust types
python
class RustTypePrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        try:
            return str(self.val)
        except:
            return str(self.val)

gdb.type_printer.register('RustPrinter', RustTypePrinter)
end

# Set up architecture-specific configurations
source /workspace/docs/setup/gdb_x86_64.gdb
source /workspace/docs/setup/gdb_aarch64.gdb
source /workspace/docs/setup/gdb_riscv64.gdb
```

#### x86_64 GDB Configuration (`/workspace/docs/setup/gdb_x86_64.gdb`)

```gdb
# x86_64-specific GDB settings
set architecture i386:x86-64
set osabi none

# Useful x86_64 commands
define boot-x86_64
    target remote | qemu-system-x86_64 -gdb stdio -S -kernel $arg0
end
document boot-x86_64
    Boot x86_64 kernel in QEMU with GDB server
    Usage: boot-x86_64 <kernel_binary>
end

define qemu-x86_64
    target remote | qemu-system-x86_64 -gdb stdio -S -kernel $arg0
end
document qemu-x86_64
    Launch QEMU for x86_64 debugging
    Usage: qemu-x86_64 <kernel_binary>
end

# Set architecture-specific breakpoints
define set-kernel-breakpoints-x86_64
    break kernel_main
    break _start
    break panic
    break hal::arch::x86_64::start::boot_main
end
```

#### ARM64 GDB Configuration (`/workspace/docs/setup/gdb_aarch64.gdb`)

```gdb
# ARM64-specific GDB settings
set architecture aarch64
set osabi none

# Useful ARM64 commands
define boot-aarch64
    target remote | qemu-system-aarch64 -gdb stdio -S -machine virt -cpu cortex-a57 -kernel $arg0
end
document boot-aarch64
    Boot ARM64 kernel in QEMU with GDB server
    Usage: boot-aarch64 <kernel_binary>
end

define qemu-aarch64
    target remote | qemu-system-aarch64 -gdb stdio -S -machine virt -cpu cortex-a57 -kernel $arg0
end
document qemu-aarch64
    Launch QEMU for ARM64 debugging
    Usage: qemu-aarch64 <kernel_binary>
end

# Set architecture-specific breakpoints
define set-kernel-breakpoints-aarch64
    break kernel_main
    break _start
    break panic
    break hal::arch::aarch64::start::boot_main
end
```

#### RISC-V GDB Configuration (`/workspace/docs/setup/gdb_riscv64.gdb`)

```gdb
# RISC-V-specific GDB settings
set architecture riscv:rv64gc
set osabi none

# Useful RISC-V commands
define boot-riscv64
    target remote | qemu-system-riscv64 -gdb stdio -S -machine virt -kernel $arg0
end
document boot-riscv64
    Boot RISC-V kernel in QEMU with GDB server
    Usage: boot-riscv64 <kernel_binary>
end

define qemu-riscv64
    target remote | qemu-system-riscv64 -gdb stdio -S -machine virt -kernel $arg0
end
document qemu-riscv64
    Launch QEMU for RISC-V debugging
    Usage: qemu-riscv64 <kernel_binary>
end

# Set architecture-specific breakpoints
define set-kernel-breakpoints-riscv64
    break kernel_main
    break _start
    break panic
    break hal::arch::riscv64::start::boot_main
end
```

### Rust-Specific GDB Scripts

Create GDB Python scripts for better Rust debugging experience:

#### Memory Analysis Script (`/workspace/docs/setup/gdb_scripts/memory.py`)

```python
#!/usr/bin/env python3
"""
Memory analysis utilities for MultiOS debugging
"""

import gdb
import struct

class MultiOSMemoryAnalyzer:
    """Memory analysis utilities for MultiOS kernel"""
    
    def __init__(self):
        self.arch = self.get_architecture()
        
    def get_architecture(self):
        """Determine current architecture"""
        try:
            return gdb.selected_frame().architecture().name()
        except:
            return "unknown"
    
    def print_page_tables(self):
        """Print page table information"""
        if "x86_64" in self.arch:
            self.print_x86_64_page_tables()
        elif "aarch64" in self.arch:
            self.print_aarch64_page_tables()
        elif "riscv" in self.arch:
            self.print_riscv_page_tables()
    
    def print_x86_64_page_tables(self):
        """Print x86_64 page tables"""
        try:
            # Get CR3 register (page table base)
            cr3 = gdb.parse_and_eval("$cr3")
            print(f"CR3 (Page Table Base): 0x{cr3 & 0xFFFFFFFFFFFF:016x}")
            
            # Print first level page table entries
            for i in range(4):  # PML4 entries
                pml4_addr = (cr3 & 0xFFFFFFFFFFFF) + i * 8
                pml4_entry = struct.unpack('<Q', gdb.selected_inferior().read_memory(pml4_addr, 8))[0]
                
                if pml4_entry & 1:  # Present bit
                    print(f"PML4[{i}]: 0x{pml4_entry:016x} (Present)")
                    
        except gdb.error as e:
            print(f"Error reading page tables: {e}")
    
    def print_kernel_heap(self):
        """Print kernel heap information"""
        try:
            # Find static linked_list_allocator::Heap instances
            heaps = gdb.execute("info variables linked_list_allocator::Heap", 
                              to_string=True).split('\n')
            for line in heaps:
                if '0x' in line:
                    addr = line.split('0x')[1].split()[0]
                    print(f"Heap at: 0x{addr}")
                    
        except gdb.error as e:
            print(f"Error reading heap: {e}")
    
    def analyze_memory_leak(self):
        """Analyze potential memory leaks"""
        print("Memory Leak Analysis")
        print("=" * 50)
        self.print_kernel_heap()
        self.print_page_tables()

class MultiOSMemoryCommand(gdb.Command):
    """MultiOS memory analysis command"""
    
    def __init__(self):
        super(MultiOSMemoryCommand, self).__init__("multios-memory", gdb.COMMAND_USER)
    
    def invoke(self, arg, from_tty):
        analyzer = MultiOSMemoryAnalyzer()
        
        if arg == "leak":
            analyzer.analyze_memory_leak()
        elif arg == "heap":
            analyzer.print_kernel_heap()
        elif arg == "pagetables":
            analyzer.print_page_tables()
        else:
            print("Usage: multios-memory [leak|heap|pagetables]")

# Register the command
MultiOSMemoryCommand()

def print_rust_backtrace():
    """Print enhanced Rust backtrace"""
    frame = gdb.newest_frame()
    while frame:
        try:
            func = frame.name()
            if func:
                symtab = frame.find_sal().symtab
                if symtab:
                    filename = symtab.filename
                    line = frame.find_sal().line
                    print(f"#{frame.level()}  {func} at {filename}:{line}")
                else:
                    print(f"#{frame.level()}  {func}")
        except:
            pass
        frame = frame.newer()

gdb.events.stop.connect(lambda event: print_rust_backtrace() if hasattr(event, 'thread') else None)
```

#### Process Analysis Script (`/workspace/docs/setup/gdb_scripts/process.py`)

```python
#!/usr/bin/env python3
"""
Process and scheduler analysis utilities
"""

import gdb

class MultiOSProcessAnalyzer:
    """Process analysis utilities"""
    
    def print_scheduler_state(self):
        """Print current scheduler state"""
        try:
            # Try to access scheduler state
            print("Scheduler State Analysis")
            print("=" * 30)
            
            # Print current process information
            current_task = gdb.parse_and_eval("scheduler::CURRENT_TASK")
            print(f"Current Task: {current_task}")
            
        except gdb.error as e:
            print(f"Error accessing scheduler: {e}")
    
    def print_process_list(self):
        """Print all processes"""
        try:
            print("Process List")
            print("=" * 20)
            
            # This would require access to the process list structure
            # Implementation depends on MultiOS internal structures
            
        except gdb.error as e:
            print(f"Error accessing process list: {e}")

class MultiOSProcessCommand(gdb.Command):
    """MultiOS process analysis command"""
    
    def __init__(self):
        super(MultiOSProcessCommand, self).__init__("multios-process", gdb.COMMAND_USER)
    
    def invoke(self, arg, from_tty):
        analyzer = MultiOSProcessAnalyzer()
        
        if arg == "scheduler":
            analyzer.print_scheduler_state()
        elif arg == "list":
            analyzer.print_process_list()
        else:
            print("Usage: multios-process [scheduler|list]")

MultiOSProcessCommand()
```

## QEMU Monitor Configuration

### QEMU Monitor Setup Script (`/workspace/scripts/qemu_monitor.sh`)

```bash
#!/bin/bash
# MultiOS QEMU Monitor Setup Script

ARCH=${1:-"x86_64"}
KERNEL=${2:-"target/${ARCH}-unknown-none-elf/release/multios"}
SERIAL_PORT=${3:-"/tmp/multios_serial"}
GDB_PORT=${4:-"1234"}

case $ARCH in
    "x86_64"|"amd64")
        QEMU_CMD="qemu-system-x86_64"
        QEMU_ARGS="-kernel $KERNEL -m 512M -boot m -serial unix:$SERIAL_PORT,server,nowait -gdb tcp::$GDB_PORT -nographic -enable-kvm"
        ;;
    "aarch64"|"arm64")
        QEMU_CMD="qemu-system-aarch64"
        QEMU_ARGS="-machine virt -cpu cortex-a57 -kernel $KERNEL -m 512M -serial unix:$SERIAL_PORT,server,nowait -gdb tcp::$GDB_PORT -nographic"
        ;;
    "riscv64"|"riscv")
        QEMU_CMD="qemu-system-riscv64"
        QEMU_ARGS="-machine virt -kernel $KERNEL -m 512M -serial unix:$SERIAL_PORT,server,nowait -gdb tcp::$GDB_PORT -nographic"
        ;;
    *)
        echo "Unknown architecture: $ARCH"
        echo "Supported: x86_64, aarch64, riscv64"
        exit 1
        ;;
esac

echo "Starting QEMU monitor for MultiOS ($ARCH)"
echo "Kernel: $KERNEL"
echo "Serial: $SERIAL_PORT"
echo "GDB: tcp::$GDB_PORT"
echo "Monitor: Ctrl-A then C, then 'quit' to exit"
echo ""

# Create serial socket directory
mkdir -p "$(dirname "$SERIAL_PORT")"

# Run QEMU
exec $QEMU_CMD $QEMU_ARGS
```

### QEMU Monitor Commands Reference

#### Basic Monitor Commands

```bash
# Connect to QEMU monitor (while QEMU is running)
# Press Ctrl-A then C to enter monitor mode

info registers          # Show CPU registers
info memory            # Show memory mapping
info network           # Show network configuration
info usb               # Show USB devices
info pci               # Show PCI devices

# System control
system_powerdown       # Power down the system
system_reset          # Reset the system
quit                  # Quit QEMU

# Debug commands
info registers        # Show all CPU registers
x/10i $pc            # Examine 10 instructions at PC
x/10x $rsp           # Examine 10 words at stack pointer
info breakpoints     # Show active breakpoints
```

#### Architecture-Specific Commands

##### x86_64 Specific
```bash
info registers        # Show x86_64 registers (rax, rbx, rcx, rdx, etc.)
info eflags          # Show CPU flags
info fpu            # Show FPU state
```

##### ARM64 Specific
```bash
info registers        # Show ARM64 registers (x0-x30, sp, pc)
info cpsr            # Show program status register
```

##### RISC-V Specific
```bash
info registers        # Show RISC-V registers (x0-x31, pc)
info csr             # Show control and status registers
```

## Serial Console Setup

### Serial Console Configuration

#### Screen/Tminit Setup

Create a serial console helper script (`/workspace/scripts/serial_console.sh`):

```bash
#!/bin/bash
# MultiOS Serial Console Helper

SERIAL_PORT=${1:-"/tmp/multios_serial"}
BAUD=${2:-"115200"}

# Check if serial socket exists
if [ ! -S "$SERIAL_PORT" ]; then
    echo "Serial socket not found: $SERIAL_PORT"
    echo "Make sure QEMU is running with serial unix:$SERIAL_PORT,server,nowait"
    exit 1
fi

# Use socat if available, otherwise fall back to screen/minicom
if command -v socat &> /dev/null; then
    echo "Using socat for serial console (Ctrl-C to exit)"
    socat -,raw,echo=0 UNIX-CONNECT:$SERIAL_PORT
elif command -v screen &> /dev/null; then
    echo "Using screen for serial console (Ctrl-A then K to exit)"
    screen $SERIAL_PORT $BAUD
else
    echo "Error: Neither socat nor screen found"
    echo "Install with: sudo apt install socat screen"
    exit 1
fi
```

#### Alternative: GTK Terminal Serial Viewer

```bash
#!/bin/bash
# GTK-based serial console for GUI environments

SERIAL_PORT=${1:-"/tmp/multios_serial"}

if ! [ -S "$SERIAL_PORT" ]; then
    echo "Serial socket not found: $SERIAL_PORT"
    exit 1
fi

# Use cutecom or create a simple GTK application
cat > /tmp/serial_gtk.py << 'EOF'
import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, GLib
import subprocess
import os
import sys

class SerialConsole(Gtk.Window):
    def __init__(self, socket_path):
        Gtk.Window.__init__(self, title="MultiOS Serial Console")
        self.set_default_size(800, 600)
        
        vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
        self.add(vbox)
        
        # Text view for serial output
        self.text_view = Gtk.TextView()
        self.text_buffer = self.text_view.get_buffer()
        scrolled = Gtk.ScrolledWindow()
        scrolled.add(self.text_view)
        vbox.pack_start(scrolled, True, True, 0)
        
        # Entry for typing
        self.entry = Gtk.Entry()
        self.entry.connect("activate", self.on_send)
        vbox.pack_start(self.entry, False, False, 0)
        
        self.socket_path = socket_path
        self.setup_socket()
        
        self.show_all()
    
    def setup_socket(self):
        # This is a simplified version - in practice, you'd want to use
        # a proper serial communication library like pyserial
        pass
    
    def on_send(self, widget):
        text = widget.get_text()
        # Send text to serial port
        widget.set_text("")

def main():
    socket_path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/multios_serial"
    win = SerialConsole(socket_path)
    Gtk.main()

if __name__ == "__main__":
    main()
EOF

python3 /tmp/serial_gtk.py "$SERIAL_PORT"
```

### Serial Logging

Create a serial logging script for debugging (`/workspace/scripts/serial_logger.sh`):

```bash
#!/bin/bash
# MultiOS Serial Logger

SERIAL_PORT=${1:-"/tmp/multios_serial"}
LOG_FILE=${2:-"/tmp/multios_serial.log"}

if [ ! -S "$SERIAL_PORT" ]; then
    echo "Serial socket not found: $SERIAL_PORT"
    exit 1
fi

echo "Logging serial output to: $LOG_FILE"
echo "Press Ctrl-C to stop logging"

# Start logging with timestamps
socat -,raw,echo=0 UNIX-CONNECT:$SERIAL_PORT SYSTEM:\
"while IFS= read -r line; do echo \"[\$(date '+%Y-%m-%d %H:%M:%S')] \$line\"; done | tee -a $LOG_FILE"
```

## VS Code Configuration

### Project-Specific Tasks (`/workspace/.vscode/tasks.json`)

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Build x86_64",
            "type": "shell",
            "command": "cargo",
            "args": ["build", "--target", "x86_64-unknown-none-elf"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared",
                "showReuseMessage": true,
                "clear": false
            },
            "problemMatcher": "$rust"
        },
        {
            "label": "Build aarch64",
            "type": "shell",
            "command": "cargo",
            "args": ["build", "--target", "aarch64-unknown-none-elf"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared",
                "showReuseMessage": true,
                "clear": false
            },
            "problemMatcher": "$rust"
        },
        {
            "label": "Build riscv64",
            "type": "shell",
            "command": "cargo",
            "args": ["build", "--target", "riscv64gc-unknown-none-elf"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared",
                "showReuseMessage": true,
                "clear": false
            },
            "problemMatcher": "$rust"
        },
        {
            "label": "Build All",
            "dependsOrder": "parallel",
            "dependsOn": ["Build x86_64", "Build aarch64", "Build riscv64"],
            "group": {
                "kind": "build",
                "isDefault": true
            }
        },
        {
            "label": "Test",
            "type": "shell",
            "command": "cargo",
            "args": ["test"],
            "group": "test",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            },
            "problemMatcher": "$rust"
        },
        {
            "label": "QEMU x86_64",
            "type": "shell",
            "command": "./scripts/qemu_monitor.sh",
            "args": ["x86_64"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "new"
            },
            "isBackground": true
        },
        {
            "label": "QEMU aarch64",
            "type": "shell",
            "command": "./scripts/qemu_monitor.sh",
            "args": ["aarch64"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "new"
            },
            "isBackground": true
        },
        {
            "label": "QEMU riscv64",
            "type": "shell",
            "command": "./scripts/qemu_monitor.sh",
            "args": ["riscv64"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "new"
            },
            "isBackground": true
        },
        {
            "label": "Serial Console",
            "type": "shell",
            "command": "./scripts/serial_console.sh",
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "new"
            },
            "isBackground": true
        }
    ]
}
```

### Debug Configuration (`/workspace/.vscode/launch.json`)

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug x86_64",
            "type": "cppdbg",
            "request": "attach",
            "program": "target/x86_64-unknown-none-elf/release/multios",
            "MIMode": "gdb",
            "miDebuggerPath": "gdb-multiarch",
            "miDebuggerServerAddress": "localhost:1234",
            "setupCommands": [
                {
                    "description": "Load MultiOS x86_64 GDB scripts",
                    "text": "source /workspace/docs/setup/gdb_x86_64.gdb",
                    "ignoreFailures": false
                },
                {
                    "description": "Set architecture to x86_64",
                    "text": "set architecture i386:x86-64",
                    "ignoreFailures": false
                },
                {
                    "description": "Load Rust pretty printers",
                    "text": "source /workspace/docs/setup/gdb_scripts/memory.py",
                    "ignoreFailures": false
                }
            ],
            "preLaunchTask": "Build x86_64"
        },
        {
            "name": "Debug aarch64",
            "type": "cppdbg",
            "request": "attach",
            "program": "target/aarch64-unknown-none-elf/release/multios",
            "MIMode": "gdb",
            "miDebuggerPath": "gdb-multiarch",
            "miDebuggerServerAddress": "localhost:1235",
            "setupCommands": [
                {
                    "description": "Load MultiOS ARM64 GDB scripts",
                    "text": "source /workspace/docs/setup/gdb_aarch64.gdb",
                    "ignoreFailures": false
                },
                {
                    "description": "Set architecture to ARM64",
                    "text": "set architecture aarch64",
                    "ignoreFailures": false
                },
                {
                    "description": "Load Rust pretty printers",
                    "text": "source /workspace/docs/setup/gdb_scripts/memory.py",
                    "ignoreFailures": false
                }
            ],
            "preLaunchTask": "Build aarch64"
        },
        {
            "name": "Debug riscv64",
            "type": "cppdbg",
            "request": "attach",
            "program": "target/riscv64gc-unknown-none-elf/release/multios",
            "MIMode": "gdb",
            "miDebuggerPath": "gdb-multiarch",
            "miDebuggerServerAddress": "localhost:1236",
            "setupCommands": [
                {
                    "description": "Load MultiOS RISC-V GDB scripts",
                    "text": "source /workspace/docs/setup/gdb_riscv64.gdb",
                    "ignoreFailures": false
                },
                {
                    "description": "Set architecture to RISC-V",
                    "text": "set architecture riscv:rv64gc",
                    "ignoreFailures": false
                },
                {
                    "description": "Load Rust pretty printers",
                    "text": "source /workspace/docs/setup/gdb_scripts/memory.py",
                    "ignoreFailures": false
                }
            ],
            "preLaunchTask": "Build riscv64"
        },
        {
            "name": "Debug Current Architecture",
            "type": "cppdbg",
            "request": "attach",
            "MIMode": "gdb",
            "miDebuggerPath": "gdb-multiarch",
            "miDebuggerServerAddress": "localhost:1234",
            "setupCommands": [
                {
                    "description": "Load MultiOS memory analysis",
                    "text": "source /workspace/docs/setup/gdb_scripts/memory.py",
                    "ignoreFailures": false
                },
                {
                    "description": "Load MultiOS process analysis",
                    "text": "source /workspace/docs/setup/gdb_scripts/process.py",
                    "ignoreFailures": false
                }
            ],
            "preLaunchTask": "Build All"
        }
    ]
}
```

### VS Code Extensions Recommendation

Create extensions recommendations file (`/workspace/.vscode/extensions.json`):

```json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "ms-vscode.cpptools",
        "ms-vscode.cpptools-extension-pack",
        "ms-vscode.vscode-json",
        "redhat.vscode-yaml",
        "tamasfe.even-better-toml",
        "serayuzgur.crates",
        "bungcip.better-toml",
        "humao.rest-client",
        "ms-vscode.hexeditor",
        "ms-vscode.remote-containers",
        "ms-azuretools.vscode-docker",
        "eamodio.gitlens",
        "donjayamanne.githistory",
        "bradlc.vscode-tailwindcss"
    ]
}
```

### Rust Analyzer Configuration (`/workspace/.vscode/settings.json`)

```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.inlayHints.enable": true,
    "rust-analyzer.inlayHints.typeHints": true,
    "rust-analyzer.inlayHints.parameterHints": true,
    "rust-analyzer.inlayHints.chainingHints": true,
    "rust-analyzer.inlayHints.discriminantHints": true,
    
    // Files and search
    "files.associations": {
        "*.rs": "rust",
        "Cargo.toml": "toml",
        "Cross.toml": "toml",
        "rust-toolchain.toml": "toml",
        "*.md": "markdown"
    },
    
    // Search exclusions
    "search.exclude": {
        "**/target": true,
        "**/Cargo.lock": true
    },
    
    // Git ignore
    "files.watcherExclude": {
        "**/target/**": true
    },
    
    // Debugging
    "debug.allowBreakpointsEverywhere": true,
    
    // Terminal
    "terminal.integrated.env.linux": {
        "RUST_BACKTRACE": "1"
    },
    "terminal.integrated.env.osx": {
        "RUST_BACKTRACE": "1"
    },
    "terminal.integrated.env.windows": {
        "RUST_BACKTRACE": "1"
    }
}
```

## Debugging Guides

### Quick Start Debugging Guide

#### 1. Setting Up a Debug Session

```bash
# Terminal 1: Build the kernel
cargo build --target x86_64-unknown-none-elf

# Terminal 2: Start QEMU with GDB server
./scripts/qemu_monitor.sh x86_64

# Terminal 3: Connect GDB
gdb-multiarch target/x86_64-unknown-none-elf/release/multios

# In GDB:
(gdb) target remote localhost:1234
(gdb) set-kernel-breakpoints-x86_64
(gdb) continue
```

#### 2. Common Debugging Workflows

##### Kernel Boot Debugging
```bash
# Set breakpoints early in boot process
(gdb) break _start
(gdb) break kernel_main
(gdb) break hal::arch::x86_64::start::boot_main
(gdb) continue

# Use multios-memory commands for memory analysis
(gdb) multios-memory pagetables
(gdb) multios-memory heap
(gdb) multios-memory leak
```

##### Memory Issues
```bash
# Analyze memory allocation
(gdb) multios-memory leak

# Check page tables
(gdb) multios-memory pagetables

# Examine heap
(gdb) multios-memory heap
```

##### Process/Scheduler Issues
```bash
# Check scheduler state
(gdb) multios-process scheduler

# List processes
(gdb) multios-process list
```

### Architecture-Specific Debugging

#### x86_64 Debugging

**Useful GDB Commands:**
```bash
# Set architecture
set architecture i386:x86-64

# Examine registers
info registers
info all-registers

# Examine memory
x/32x $rsp          # Examine stack
x/32i $pc           # Examine code
x/32x $cr3          # Examine page tables

# Set watchpoints
watch *0x7fff...    # Watch memory address
awatch *0x7fff...   # Access watchpoint
rwatch *0x7fff...   # Read watchpoint
```

**QEMU Monitor Commands:**
```bash
# Register state
info registers
info fpu

# Memory management
info memory

# Breakpoints
info breakpoints
```

#### ARM64 Debugging

**Useful GDB Commands:**
```bash
# Set architecture
set architecture aarch64

# Examine registers
info registers
info all-registers x0-x30

# Memory access
x/32x $sp           # Examine stack
x/32i $pc           # Examine code

# System registers (requires proper setup)
info registers sctlr_el1
info registers tcr_el1
```

#### RISC-V Debugging

**Useful GDB Commands:**
```bash
# Set architecture
set architecture riscv:rv64gc

# Examine registers
info registers
info all-registers x0-x31

# Control and status registers
info csr

# Memory access
x/32x $sp           # Examine stack
x/32i $pc           # Examine code
```

### Serial Console Debugging

#### Reading Boot Messages
```bash
# Start serial console
./scripts/serial_console.sh

# Or log to file
./scripts/serial_logger.sh

# Check recent boot messages
tail -f /tmp/multios_serial.log
```

#### Common Serial Output Patterns

**Boot Sequence:**
```
MultiOS v0.1.0 starting...
[HAL] Initializing x86_64 architecture
[Memory] Free memory: 256MB
[Scheduler] Initializing round-robin scheduler
[Console] Serial console ready
```

**Error Messages:**
```
PANIC: Page fault in kernel
[Error] Failed to allocate memory
[Debug] Stack trace follows
```

### Cross-Architecture Debugging

#### Building for Multiple Architectures
```bash
# Build all targets
cargo build --workspace --target x86_64-unknown-none-elf
cargo build --workspace --target aarch64-unknown-none-elf
cargo build --workspace --target riscv64gc-unknown-none-elf

# Test in parallel
cargo test --target x86_64-unknown-none-elf &
cargo test --target aarch64-unknown-none-elf &
cargo test --target riscv64gc-unknown-none-elf &
```

## Troubleshooting

### Common Issues and Solutions

#### 1. GDB Connection Issues

**Problem:** Cannot connect to QEMU GDB server
```bash
# Error: Connection refused
# Remote 'g' packet reply is too long
```

**Solutions:**
```bash
# Check if GDB server is running
netstat -an | grep 1234
lsof -i :1234

# Verify architecture match
(gdb) set architecture i386:x86-64  # for x86_64
(gdb) set architecture aarch64     # for ARM64
(gdb) set architecture riscv:rv64gc # for RISC-V

# Try different connection methods
(gdb) target remote :1234
(gdb) target remote tcp::1234
(gdb) target remote | qemu-system-x86_64 -gdb stdio
```

#### 2. QEMU Startup Issues

**Problem:** QEMU fails to start
```bash
# Error: Could not initialize KVM
# Error: Failed to find boot disk
```

**Solutions:**
```bash
# Check kernel file exists
ls -la target/x86_64-unknown-none-elf/release/multios

# Try without KVM
qemu-system-x86_64 -kernel target/.../multios -nographic -enable-kvm=off

# Check architecture support
qemu-system-aarch64 --version
qemu-system-riscv64 --version

# Verify linker script exists
ls kernel/x86_64/linker.ld
```

#### 3. Serial Console Issues

**Problem:** Serial console not working
```bash
# Error: No such file or directory
# Error: Permission denied
```

**Solutions:**
```bash
# Check socket exists
ls -la /tmp/multios_serial

# Create socket directory
mkdir -p /tmp

# Check permissions
chmod 777 /tmp/multios_serial

# Use alternative tools
minicom -D /tmp/multios_serial
cu -l /tmp/multios_serial
```

#### 4. Cross-Compilation Issues

**Problem:** Wrong architecture binary
```bash
# Error: cannot execute binary file: Exec format error
```

**Solutions:**
```bash
# Verify target triple
file target/x86_64-unknown-none-elf/release/multios

# Check Cargo configuration
cat .cargo/config.toml

# Install missing targets
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf

# Use cross for compilation
cross build --target x86_64-unknown-none-elf
```

#### 5. Memory Debugging Issues

**Problem:** Memory analysis shows no data
```bash
# Solution: Check symbols are loaded
(gdb) info symbol 0xaddress
(gdb) info variables

# Reload symbols
(gdb) file target/x86_64-unknown-none-elf/release/multios

# Check debug info level
show debug debug
```

### Debug Script Issues

#### GDB Python Scripts Not Loading

**Problem:** Custom GDB scripts not working
```bash
# Solution: Check Python support
(gdb) python print("Python works")

# Verify script syntax
python3 /workspace/docs/setup/gdb_scripts/memory.py

# Load manually first
(gdb) source /workspace/docs/setup/gdb_scripts/memory.py

# Check paths
(gdb) python import sys; print(sys.path)
```

#### Missing Rust Pretty Printers

**Problem:** Rust types not displaying properly
```bash
# Solution: Load Rust GDB pretty printers
curl -L https://github.com/rust-lang/rust/raw/master/src/etc/gdb_providers.py -o ~/.gdb_rust_pretty_printing.py

# Add to .gdbinit
source ~/.gdb_rust_pretty_printing.py
```

### Performance Issues

#### Slow Debugging

**Problem:** GDB sessions are very slow
```solutions:
```bash
# Disable symbol loading for unused files
(gdb) skip file*/std/

# Reduce symbol table size
(gdb) set build-id-verbose 0
(gdb) set build-id-cache-size 0

# Limit symbol search
(gdb) set max-symbol-cache 1000

# Use separate GDB instances per architecture
```

### VS Code Integration Issues

#### Debug Configuration Not Working

**Problem:** VS Code debug not starting
```json
// Solution: Check tasks.json and launch.json syntax
{
    "version": "2.0.0",  // Must be 2.0.0
    "configurations": [
        {
            "name": "Debug x86_64",
            "type": "cppdbg",  // Must be cppdbg for GDB
            "request": "attach"  // or "launch"
        }
    ]
}
```

#### Rust Analyzer Not Working

**Problem:** Rust code not highlighting or showing errors
```json
// Solution: Check rust-analyzer settings
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

### Building and Dependencies

#### Missing Dependencies

**Problem:** Build fails with missing dependencies
```bash
# Solution: Install system dependencies
sudo apt update
sudo apt install -y build-essential cmake ninja-build pkg-config

# For cross-compilation
sudo apt install -y gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu

# For QEMU
sudo apt install -y qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64

# Install Rust target
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf
```

## Architecture-Specific Notes

### x86_64 Specific

**Linker Script Location:** `kernel/x86_64/linker.ld`
**Boot Protocol:** UEFI + BIOS support
**Debug Features:**
- IA-32e paging support
- SSE/SSE2 debugging
- System management mode (SMM) hooks

**QEMU Command:**
```bash
qemu-system-x86_64 \
    -kernel target/x86_64-unknown-none-elf/release/multios \
    -m 512M \
    -serial unix:/tmp/multios_serial,server,nowait \
    -gdb tcp::1234 \
    -nographic \
    -enable-kvm
```

### ARM64 Specific

**Linker Script Location:** `kernel/aarch64/linker.ld`
**Boot Protocol:** UEFI + Device Tree
**Debug Features:**
- AArch64 exception handling
- SVE debugging support
- System register access

**QEMU Command:**
```bash
qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a57 \
    -kernel target/aarch64-unknown-none-elf/release/multios \
    -m 512M \
    -serial unix:/tmp/multios_serial,server,nowait \
    -gdb tcp::1235 \
    -nographic
```

### RISC-V Specific

**Linker Script Location:** `kernel/riscv64/linker.ld`
**Boot Protocol:** OpenSBI + Device Tree
**Debug Features:**
- RV64GC support (G=General, C=Compressed)
- Exception debugging
- PMP (Physical Memory Protection)

**QEMU Command:**
```bash
qemu-system-riscv64 \
    -machine virt \
    -kernel target/riscv64gc-unknown-none-elf/release/multios \
    -m 512M \
    -serial unix:/tmp/multios_serial,server,nowait \
    -gdb tcp::1236 \
    -nographic
```

## Best Practices

### Debugging Workflow

1. **Start with Serial Console:** Always start with serial output to see boot messages
2. **Use GDB for Low-Level Issues:** For memory corruption, page faults, etc.
3. **Leverage QEMU Monitor:** For system state inspection
4. **Architecture-Specific Tools:** Use appropriate tools for each architecture

### Documentation

1. **Log Everything:** Keep detailed logs of debug sessions
2. **Save Breakpoint Configurations:** Maintain GDB scripts for reproducible debugging
3. **Document Solutions:** Add solutions to this troubleshooting guide

### Performance

1. **Optimize Symbol Loading:** Only load symbols for relevant code
2. **Use Multiple GDB Instances:** One per architecture for parallel debugging
3. **Leverage VS Code Tasks:** Automate common build and debug operations

### Educational Value

1. **Clear Error Messages:** Ensure the OS provides helpful error messages
2. **Step-by-Step Tutorials:** Use the debugging guides to teach OS concepts
3. **Interactive Debugging:** Use GDB to demonstrate memory management, scheduling, etc.

## Conclusion

This debugging setup provides comprehensive tools for MultiOS development across all supported architectures. The combination of GDB, QEMU, serial console, and VS Code integration creates a powerful development environment optimized for educational use.

For questions or issues not covered in this guide, refer to:
- [MultiOS Technical Specifications](../multios_technical_specifications.md)
- [Cross-Compilation Guide](../cross_compilation/cross_compilation_guide.md)
- [Architecture Analysis](../os_architectures/os_architectures_analysis.md)

---

*This document is part of the MultiOS educational operating system project.*
*Last updated: 2025-11-02*