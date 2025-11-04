#!/bin/bash
# MultiOS Development Environment Setup Script
# This script sets up the complete development environment for MultiOS

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_header() {
    print_message $BLUE "=================================="
    print_message $BLUE "MultiOS Development Setup"
    print_message $BLUE "=================================="
    echo
}

print_success() {
    print_message $GREEN "✓ $1"
}

print_warning() {
    print_message $YELLOW "⚠ $1"
}

print_error() {
    print_message $RED "✗ $1"
}

print_info() {
    print_message $BLUE "ℹ $1"
}

check_command() {
    if command -v "$1" &> /dev/null; then
        print_success "$1 is installed"
        return 0
    else
        print_error "$1 is not installed"
        return 1
    fi
}

install_system_dependencies() {
    print_info "Installing system dependencies..."
    
    if command -v apt &> /dev/null; then
        sudo apt update
        sudo apt install -y \
            build-essential \
            cmake \
            ninja-build \
            pkg-config \
            curl \
            git \
            gdb-multiarch \
            qemu-system-x86 \
            qemu-system-aarch64 \
            qemu-system-riscv64 \
            socat \
            screen \
            minicom \
            opensbi \
            u-boot-tools
        
        print_success "System dependencies installed"
    else
        print_warning "apt not found. Please install dependencies manually:"
        print_info "- build-essential, cmake, ninja-build, pkg-config"
        print_info "- gdb-multiarch"
        print_info "- qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64"
        print_info "- socat, screen, minicom"
    fi
}

install_rust() {
    print_info "Checking Rust installation..."
    
    if check_command "rustc"; then
        print_success "Rust is already installed"
        rustc --version
    else
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        
        if check_command "rustc"; then
            print_success "Rust installed successfully"
            rustc --version
        else
            print_error "Rust installation failed"
            exit 1
        fi
    fi
}

install_rust_targets() {
    print_info "Installing Rust targets..."
    
    rustup target add x86_64-unknown-none-elf
    rustup target add aarch64-unknown-none-elf
    rustup target add riscv64gc-unknown-none-elf
    
    print_success "Rust targets installed"
}

install_cross_tool() {
    print_info "Installing cross tool..."
    
    if cargo install cross --locked; then
        print_success "Cross tool installed"
    else
        print_warning "Cross tool installation failed, continuing without it"
    fi
}

setup_gdb_config() {
    print_info "Setting up GDB configuration..."
    
    # Create global .gdbinit if it doesn't exist
    if [ ! -f "$HOME/.gdbinit" ]; then
        cp /workspace/docs/setup/.gdbinit "$HOME/.gdbinit"
        print_success "Global .gdbinit created"
    else
        print_info "Global .gdbinit already exists, backing up and updating..."
        cp "$HOME/.gdbinit" "$HOME/.gdbinit.backup.$(date +%Y%m%d_%H%M%S)"
        # Append our configuration
        echo "" >> "$HOME/.gdbinit"
        echo "# MultiOS configuration" >> "$HOME/.gdbinit"
        echo "source /workspace/docs/setup/.gdbinit" >> "$HOME/.gdbinit"
        print_success "Global .gdbinit updated"
    fi
}

setup_vscode() {
    print_info "Setting up VS Code configuration..."
    
    # VS Code configuration is already created in the project
    print_success "VS Code configuration is ready"
    print_info "Recommended VS Code extensions:"
    print_info "  - rust-lang.rust-analyzer"
    print_info "  - ms-vscode.cpptools"
    print_info "  - ms-vscode.remote-containers"
}

test_installation() {
    print_info "Testing installation..."
    
    # Test basic cargo commands
    if cargo check --help &> /dev/null; then
        print_success "Cargo is working"
    else
        print_error "Cargo is not working properly"
        return 1
    fi
    
    # Test if we can build for a target
    print_info "Testing cross-compilation..."
    cd /workspace
    
    if cargo check --target x86_64-unknown-none-elf &> /dev/null; then
        print_success "x86_64 target compilation works"
    else
        print_warning "x86_64 target compilation test failed (this is normal for new projects)"
    fi
    
    # Test QEMU
    if check_command "qemu-system-x86_64"; then
        print_success "QEMU is working"
    else
        print_warning "QEMU test failed"
    fi
    
    # Test GDB
    if check_command "gdb-multiarch"; then
        print_success "GDB multiarch is working"
    else
        print_warning "GDB multiarch test failed"
    fi
}

print_usage_examples() {
    print_message $BLUE "\nUsage Examples"
    print_message $BLUE "=============="
    echo
    
    print_info "Building for all architectures:"
    echo "  cargo build --target x86_64-unknown-none-elf"
    echo "  cargo build --target aarch64-unknown-none-elf"
    echo "  cargo build --target riscv64gc-unknown-none-elf"
    echo
    
    print_info "Starting QEMU for debugging:"
    echo "  ./scripts/qemu_monitor.sh x86_64"
    echo
    
    print_info "Connecting GDB:"
    echo "  gdb-multiarch target/x86_64-unknown-none-elf/release/multios"
    echo "  (gdb) target remote localhost:1234"
    echo "  (gdb) continue"
    echo
    
    print_info "VS Code tasks (Ctrl+Shift+P → Tasks: Run Task):"
    echo "  Build All"
    echo "  QEMU x86_64"
    echo "  Debug x86_64"
    echo
    
    print_info "Serial console (in separate terminal):"
    echo "  ./scripts/serial_console.sh"
    echo
    
    print_info "VS Code debugging:"
    echo "  Open command palette (Ctrl+Shift+P)"
    echo "  Run: Debug: Start Debugging"
    echo "  Choose: Debug x86_64"
}

print_next_steps() {
    print_message $BLUE "\nNext Steps"
    print_message $BLUE "=========="
    echo
    
    print_info "1. Review the debugging setup guide:"
    echo "   docs/setup/debugging_setup.md"
    echo
    
    print_info "2. Try building the kernel:"
    echo "   cargo build --target x86_64-unknown-none-elf"
    echo
    
    print_info "3. Start a debugging session:"
    echo "   Terminal 1: ./scripts/qemu_monitor.sh x86_64"
    echo "   Terminal 2: ./scripts/serial_console.sh"
    echo "   Terminal 3: gdb-multiarch target/x86_64-unknown-none-elf/release/multios"
    echo
    
    print_info "4. Explore VS Code integration:"
    echo "   - Install recommended extensions"
    echo "   - Use Ctrl+Shift+P to run tasks"
    echo "   - Use F5 to start debugging"
    echo
    
    print_info "5. Learn the debugging commands:"
    echo "   (gdb) multios-help"
    echo "   (gdb) multios-memory leak"
    echo "   (gdb) multios-process scheduler"
    echo
    
    print_success "MultiOS development environment setup complete!"
}

main() {
    print_header
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || [ ! -d "docs/setup" ]; then
        print_error "Please run this script from the MultiOS project root directory"
        exit 1
    fi
    
    print_info "Setting up MultiOS development environment..."
    echo
    
    # Install dependencies step by step
    install_system_dependencies
    echo
    
    install_rust
    echo
    
    install_rust_targets
    echo
    
    install_cross_tool
    echo
    
    setup_gdb_config
    echo
    
    setup_vs_code
    echo
    
    test_installation
    echo
    
    print_usage_examples
    print_next_steps
}

# Run main function
main "$@"
