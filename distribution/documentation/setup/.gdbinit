# Global GDB Configuration for MultiOS Development
# This file configures GDB for MultiOS kernel debugging across all architectures

# Enable Python scripting for enhanced debugging
python
import sys
import os

# Add MultiOS scripts to Python path
workspace_path = "/workspace"
scripts_path = os.path.join(workspace_path, "docs", "setup", "gdb_scripts")
sys.path.insert(0, scripts_path)

# Load custom GDB modules
try:
    import memory
    import process
    print("MultiOS GDB modules loaded successfully")
except Exception as e:
    print(f"Warning: Could not load MultiOS GDB modules: {e}")
end

# Set up general GDB settings
set pagination off
set confirm off
set print pretty on
set print object on
set print static-members on
set print vtbl on
set print demangle on
set demangle-style gnu-v3
set print address
set print symbol-filename on
set print symbol on
set print source on
set print source-lines on

# Better display for Rust types
python
class MultiOSRustPrinter:
    def __init__(self, val):
        self.val = val

    def to_string(self):
        try:
            # Try to get a nice string representation
            type_name = str(self.val.type)
            if type_name.startswith("&str"):
                return f'"{self.val}"'
            elif type_name.startswith("String"):
                return str(self.val)
            else:
                return str(self.val)
        except:
            return str(self.val)

# Register Rust pretty printer
gdb.type_printer.register('MultiOSRustPrinter', MultiOSRustPrinter())
end

# Architecture-specific auto-configuration
# Try to detect and configure based on binary being debugged
python
def detect_architecture():
    try:
        # This function tries to determine the architecture from the binary
        # It will be called when a binary is loaded
        pass
    except:
        pass

# Auto-setup when binary is loaded
gdb.events.new_objfile.connect(detect_architecture)
end

# Set up custom commands
source /workspace/docs/setup/gdb_x86_64.gdb
source /workspace/docs/setup/gdb_aarch64.gdb  
source /workspace/docs/setup/gdb_riscv64.gdb

# Global utility commands
define multios-help
    echo MultiOS Debugging Help\n
    echo ====================\n
    echo Architecture Commands:\n
    echo   setup-x86_64        - Setup x86_64 debugging\n
    echo   setup-aarch64       - Setup ARM64 debugging\n  
    echo   setup-riscv64       - Setup RISC-V debugging\n
    echo\n
    echo Analysis Commands:\n
    echo   multios-memory      - Memory analysis (leak|heap|pagetables|stack|corruption)\n
    echo   multios-process     - Process analysis (current|scheduler|context|interrupt|creation|ipc|memory)\n
    echo   multios-performance - Performance analysis\n
    echo\n
    echo Architecture-specific commands:\n
    echo   help-x86_64         - Show x86_64 commands\n
    echo   help-aarch64        - Show ARM64 commands\n
    echo   help-riscv64        - Show RISC-V commands\n
end
document multios-help
    Show MultiOS debugging help and commands
end

# Set up auto-completion for custom commands
python
class MultiOSCommandCompleter:
    def __init__(self):
        pass
    
    def complete(self, text, word):
        commands = [
            'multios-memory', 'multios-process', 'multios-performance',
            'setup-x86_64', 'setup-aarch64', 'setup-riscv64',
            'help-x86_64', 'help-aarch64', 'help-riscv64'
        ]
        
        if text.startswith('multios-memory'):
            subcommands = ['leak', 'heap', 'pagetables', 'stack', 'corruption', 'help']
            return [cmd for cmd in subcommands if cmd.startswith(word)]
        elif text.startswith('multios-process'):
            subcommands = ['current', 'scheduler', 'context', 'interrupt', 'creation', 'ipc', 'memory', 'help']
            return [cmd for cmd in subcommands if cmd.startswith(word)]
        elif text.startswith('multios-performance'):
            subcommands = ['timing', 'help']
            return [cmd for cmd in subcommands if cmd.startswith(word)]
        else:
            return [cmd for cmd in commands if cmd.startswith(word)]

# Try to register the completer (this might not work in all GDB versions)
try:
    completer = MultiOSCommandCompleter()
    # This would require access to GDB's completion API
    # gdb.command.complete('multios-memory', completer.complete)
except:
    pass
end

# Auto-backtrace on panic
python
def auto_panic_handler(event):
    try:
        if hasattr(event, 'stop_signal'):
            # Check if this looks like a panic
            frame = gdb.newest_frame()
            if frame and frame.name() and 'panic' in frame.name().lower():
                print("\n*** AUTO-PANIC DETECTED ***")
                print("Stack trace follows...")
                gdb.execute('backtrace')
    except:
        pass

try:
    gdb.events.stop.connect(auto_panic_handler)
except:
    pass
end

# Welcome message
echo \n
echo MultiOS GDB Configuration Loaded\n
echo =================================\n
echo Available commands:\n
echo   multios-help    - Show all available commands\n
echo   help-x86_64     - x86_64 specific help\n
echo   help-aarch64    - ARM64 specific help\n
echo   help-riscv64    - RISC-V specific help\n
echo \n
echo Load a binary with: file <path-to-binary>\n
echo Then use setup commands for your architecture\n
echo \n
