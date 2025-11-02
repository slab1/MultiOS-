#!/bin/bash

# MultiOS Comprehensive Testing Suite - Dependencies Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect operating system
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get >/dev/null 2>&1; then
            OS="ubuntu"
        elif command -v dnf >/dev/null 2>&1; then
            OS="fedora"
        elif command -v pacman >/dev/null 2>&1; then
            OS="arch"
        else
            OS="linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    else
        OS="unknown"
    fi
}

# Install Rust toolchain
install_rust() {
    print_status "Installing Rust toolchain..."
    
    if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        print_success "Rust toolchain already installed"
        rustc --version
        cargo --version
        return 0
    fi
    
    print_status "Downloading and installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    print_success "Rust toolchain installed successfully"
    rustc --version
    cargo --version
}

# Install system dependencies based on OS
install_system_deps() {
    print_status "Installing system dependencies for $OS..."
    
    case $OS in
        "ubuntu"|"debian")
            print_status "Detected Ubuntu/Debian system"
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                pkg-config \
                libssl-dev \
                protobuf-compiler \
                protobuf-c-compiler \
                doxygen \
                graphviz \
                git \
                curl \
                wget
                
            # Install QEMU
            print_status "Installing QEMU..."
            sudo apt-get install -y \
                qemu-system-x86 \
                qemu-system-aarch64 \
                qemu-system-riscv64 \
                qemu-system-ppc64le
                
            # Install cross-compilation toolchains
            print_status "Installing cross-compilation toolchains..."
            sudo apt-get install -y \
                gcc-aarch64-linux-gnu \
                gcc-riscv64-linux-gnu \
                gcc-powerpc64le-linux-gnu
            ;;
        "fedora"|"rhel")
            print_status "Detected Fedora/RHEL system"
            sudo dnf install -y \
                gcc \
                openssl-devel \
                protobuf-compiler \
                protobuf-c-compiler \
                doxygen \
                graphviz \
                git \
                curl \
                wget
                
            # Install QEMU
            print_status "Installing QEMU..."
            sudo dnf install -y \
                qemu-system-x86 \
                qemu-system-aarch64 \
                qemu-system-riscv64 \
                qemu-system-ppc64le
                
            # Install cross-compilation toolchains
            print_status "Installing cross-compilation toolchains..."
            sudo dnf install -y \
                gcc-aarch64-linux-gnu \
                gcc-riscv64-linux-gnu \
                gcc-powerpc64le-linux-gnu
            ;;
        "arch")
            print_status "Detected Arch Linux system"
            sudo pacman -S --noconfirm \
                base-devel \
                openssl \
                protobuf \
                doxygen \
                graphviz \
                git \
                curl \
                wget
                
            # Install QEMU
            print_status "Installing QEMU..."
            sudo pacman -S --noconfirm qemu
                
            # Install cross-compilation toolchains
            print_status "Installing cross-compilation toolchains..."
            sudo pacman -S --noconfirm \
                aarch64-linux-gnu-gcc \
                riscv64-linux-gnu-gcc \
                powerpc64le-linux-gnu-gcc
            ;;
        "macos")
            print_status "Detected macOS system"
            
            # Check if Homebrew is installed
            if ! command -v brew >/dev/null 2>&1; then
                print_status "Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            
            # Install dependencies
            print_status "Installing dependencies via Homebrew..."
            brew install \
                rust \
                openssl \
                protobuf \
                doxygen \
                graphviz \
                qemu \
                gcc
            ;;
        *)
            print_error "Unknown or unsupported operating system: $OSTYPE"
            print_error "Please install dependencies manually:"
            print_error "1. Rust: https://rustup.rs/"
            print_error "2. QEMU: https://www.qemu.org/download/#source"
            print_error "3. System dependencies: OpenSSL, protobuf, doxygen, graphviz"
            exit 1
            ;;
    esac
    
    print_success "System dependencies installed successfully"
}

# Install Rust development tools
install_rust_tools() {
    print_status "Installing Rust development tools..."
    
    local tools=(
        "cargo-audit"
        "cargo-tarpaulin"
        "cargo-deny"
        "cross"
        "cargo-watch"
    )
    
    for tool in "${tools[@]}"; do
        print_status "Installing $tool..."
        if cargo install "$tool"; then
            print_success "$tool installed successfully"
        else
            print_warning "Failed to install $tool"
        fi
    done
}

# Verify installations
verify_installations() {
    print_status "Verifying installations..."
    
    # Check Rust
    if command -v rustc >/dev/null 2>&1; then
        print_success "Rust: $(rustc --version)"
    else
        print_error "Rust not found"
    fi
    
    # Check Cargo
    if command -v cargo >/dev/null 2>&1; then
        print_success "Cargo: $(cargo --version)"
    else
        print_error "Cargo not found"
    fi
    
    # Check QEMU
    local qemu_binaries=(
        "qemu-system-x86_64"
        "qemu-system-aarch64"
        "qemu-system-riscv64"
    )
    
    for binary in "${qemu_binaries[@]}"; do
        if command -v "$binary" >/dev/null 2>&1; then
            print_success "$binary: $($binary --version | head -n1)"
        else
            print_warning "$binary: NOT FOUND"
        fi
    done
    
    # Check development tools
    local tools=(
        "cargo-audit"
        "cargo-tarpaulin"
        "cargo-deny"
    )
    
    for tool in "${tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            print_success "$tool: installed"
        else
            print_warning "$tool: NOT FOUND"
        fi
    done
}

# Main function
main() {
    echo "=================================================="
    echo "  MultiOS Comprehensive Testing Suite"
    echo "  Dependencies Installation"
    echo "=================================================="
    echo
    
    # Detect OS
    detect_os
    print_status "Detected operating system: $OS"
    
    # Install components
    install_rust
    install_system_deps
    install_rust_tools
    
    # Verify installations
    echo
    print_status "Verification Report:"
    verify_installations
    
    echo
    print_success "Dependencies installation completed!"
    
    echo
    echo "Next steps:"
    echo "1. Run the development environment setup:"
    echo "   ./scripts/setup_dev_env.sh"
    echo
    echo "2. Build the project:"
    echo "   cargo build --release"
    echo
    echo "3. Run tests:"
    echo "   cargo test"
    echo
    echo "4. View available make targets:"
    echo "   make help"
}

# Handle arguments
case "${1:-}" in
    "--help"|"-h")
        echo "MultiOS Comprehensive Testing Suite - Dependencies Installation"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --rust-only    Install only Rust toolchain"
        echo "  --system-only  Install only system dependencies"
        echo "  --tools-only   Install only Rust development tools"
        echo "  --verify       Verify existing installations"
        echo
        exit 0
        ;;
    "--rust-only")
        install_rust
        ;;
    "--system-only")
        install_system_deps
        ;;
    "--tools-only")
        install_rust_tools
        ;;
    "--verify")
        verify_installations
        ;;
    *)
        main
        ;;
esac
