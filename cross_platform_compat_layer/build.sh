#!/bin/bash
# Build script for MultiOS Cross-Platform Compatibility Layer

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

echo "MultiOS Cross-Platform Compatibility Layer Build Script"
echo "========================================================"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Please install Rust first."
        exit 1
    fi
    
    RUST_VERSION=$(rustc --version)
    print_status "Found Rust: $RUST_VERSION"
}

# Check if cargo is installed
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Cargo first."
        exit 1
    fi
    
    CARGO_VERSION=$(cargo --version)
    print_status "Found Cargo: $CARGO_VERSION"
}

# Build for native architecture
build_native() {
    print_status "Building for native architecture..."
    cd "$PROJECT_ROOT"
    cargo build --release
    print_status "Native build completed"
}

# Build for specific target
build_target() {
    local target=$1
    print_status "Building for target: $target"
    
    cd "$PROJECT_ROOT"
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "^$target$"; then
        print_status "Installing target: $target"
        rustup target add "$target"
    fi
    
    cargo build --target "$target" --release
    print_status "Build for $target completed"
}

# Build for all supported architectures
build_all() {
    print_status "Building for all supported architectures..."
    
    build_target "x86_64-unknown-none"
    build_target "aarch64-unknown-none"
    build_target "riscv64gc-unknown-none"
    
    print_status "All builds completed successfully"
}

# Run tests
run_tests() {
    print_status "Running tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests
    cargo test --release
    
    # Run integration tests if they exist
    if [ -d "tests" ]; then
        cargo test --release --test integration
    fi
    
    print_status "All tests passed"
}

# Clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    
    cd "$PROJECT_ROOT"
    cargo clean
    rm -rf target/
    
    print_status "Clean completed"
}

# Install target tools
install_tools() {
    print_status "Installing required target tools..."
    
    # Install QEMU for testing (if not already installed)
    if ! command -v qemu-system-x86_64 &> /dev/null; then
        print_warning "QEMU not found. Please install QEMU for cross-platform testing."
        print_warning "Ubuntu/Debian: sudo apt install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64"
        print_warning "macOS: brew install qemu"
    else
        print_status "QEMU found: $(qemu-system-x86_64 --version | head -n1)"
    fi
    
    # Check for target-specific tools
    if command -v clang &> /dev/null; then
        print_status "Clang found: $(clang --version | head -n1)"
    else
        print_warning "Clang not found. Some features may require Clang."
    fi
}

# Show help
show_help() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  native    Build for native architecture only"
    echo "  x86_64    Build for x86_64 architecture"
    echo "  aarpch64  Build for ARM64 architecture"
    echo "  riscv64   Build for RISC-V64 architecture"
    echo "  all       Build for all supported architectures"
    echo "  test      Run all tests"
    echo "  clean     Clean build artifacts"
    echo "  tools     Install required tools"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 native    # Build for current architecture"
    echo "  $0 all       # Build for all architectures"
    echo "  $0 test      # Run tests"
}

# Parse command line arguments
case "${1:-help}" in
    native)
        check_rust
        check_cargo
        build_native
        ;;
    x86_64)
        check_rust
        check_cargo
        build_target "x86_64-unknown-none"
        ;;
    aarch64)
        check_rust
        check_cargo
        build_target "aarch64-unknown-none"
        ;;
    riscv64)
        check_rust
        check_cargo
        build_target "riscv64gc-unknown-none"
        ;;
    all)
        check_rust
        check_cargo
        build_all
        ;;
    test)
        check_rust
        check_cargo
        run_tests
        ;;
    clean)
        clean
        ;;
    tools)
        install_tools
        ;;
    help|*)
        show_help
        ;;
esac

print_status "Build script completed"