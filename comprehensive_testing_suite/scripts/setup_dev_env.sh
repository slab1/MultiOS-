#!/bin/bash

# MultiOS Comprehensive Testing Suite - Development Environment Setup Script

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check system type
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command_exists apt-get; then
            OS="ubuntu"
        elif command_exists dnf; then
            OS="fedora"
        elif command_exists pacman; then
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
    
    if command_exists rustc && command_exists cargo; then
        print_success "Rust toolchain already installed"
        rustc --version
        cargo --version
        return 0
    fi
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    print_success "Rust toolchain installed successfully"
    rustc --version
    cargo --version
}

# Install system dependencies
install_system_deps() {
    print_status "Installing system dependencies..."
    
    case $OS in
        "ubuntu"|"debian")
            print_status "Installing dependencies for Ubuntu/Debian..."
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                pkg-config \
                libssl-dev \
                protobuf-compiler \
                protobuf-c-compiler \
                doxygen \
                graphviz \
                qemu-system-x86 \
                qemu-system-aarch64 \
                qemu-system-riscv64 \
                gcc-aarch64-linux-gnu \
                gcc-riscv64-linux-gnu \
                git \
                curl \
                wget
            ;;
        "fedora"|"rhel")
            print_status "Installing dependencies for Fedora/RHEL..."
            sudo dnf install -y \
                gcc \
                gcc-aarch64-linux-gnu \
                gcc-riscv64-linux-gnu \
                openssl-devel \
                protobuf-compiler \
                protobuf-c-compiler \
                doxygen \
                graphviz \
                qemu-system-x86 \
                qemu-system-aarch64 \
                qemu-system-riscv64 \
                git \
                curl \
                wget
            ;;
        "arch")
            print_status "Installing dependencies for Arch Linux..."
            sudo pacman -S --noconfirm \
                base-devel \
                openssl \
                protobuf \
                doxygen \
                graphviz \
                qemu \
                aarch64-linux-gnu-gcc \
                riscv64-linux-gnu-gcc \
                git \
                curl \
                wget
            ;;
        "macos")
            print_status "Installing dependencies for macOS..."
            if ! command_exists brew; then
                print_status "Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            
            brew install \
                rust \
                qemu \
                openssl \
                protobuf \
                doxygen \
                graphviz \
                gcc
            ;;
        *)
            print_warning "Unknown OS type. Please install dependencies manually."
            return 1
            ;;
    esac
    
    print_success "System dependencies installed successfully"
}

# Install Rust development tools
install_rust_tools() {
    print_status "Installing Rust development tools..."
    
    # Install additional cargo tools
    cargo install cargo-audit
    cargo install cargo-tarpaulin
    cargo install cargo-deny
    cargo install cross
    
    print_success "Rust development tools installed successfully"
}

# Verify QEMU installation
verify_qemu() {
    print_status "Verifying QEMU installation..."
    
    local qemu_binaries=(
        "qemu-system-x86_64"
        "qemu-system-aarch64"
        "qemu-system-riscv64"
    )
    
    local missing_bins=()
    
    for binary in "${qemu_binaries[@]}"; do
        if command_exists "$binary"; then
            local version=$($binary --version | head -n1)
            print_success "$binary: $version"
        else
            missing_bins+=("$binary")
            print_error "$binary: NOT FOUND"
        fi
    done
    
    if [ ${#missing_bins[@]} -gt 0 ]; then
        print_warning "Some QEMU binaries are missing. Install with:"
        print_warning "  Ubuntu/Debian: sudo apt-get install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64"
        print_warning "  Fedora/RHEL: sudo dnf install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64"
        return 1
    else
        print_success "All QEMU binaries found successfully"
    fi
}

# Create project structure
create_project_structure() {
    print_status "Creating project structure..."
    
    local dirs=(
        "test_results"
        "performance_reports"
        "stress_test_results"
        "coverage_reports"
        "qemu_environments"
        "qemu_environments/x86_64"
        "qemu_environments/arm64"
        "qemu_environments/riscv"
        "logs"
        "artifacts"
    )
    
    for dir in "${dirs[@]}"; do
        mkdir -p "$dir"
        print_success "Created directory: $dir"
    done
}

# Setup environment variables
setup_environment() {
    print_status "Setting up environment variables..."
    
    local env_file=".env"
    
    cat > "$env_file" << EOF
# MultiOS Comprehensive Testing Suite Environment Configuration
export RUST_LOG=info
export MULTIOS_TEST_TIMEOUT=1800
export MULTIOS_MAX_MEMORY_MB=2048
export MULTIOS_QEMU_X86_64=qemu-system-x86_64
export MULTIOS_QEMU_AARCH64=qemu-system-aarch64
export MULTIOS_QEMU_RISCV=qemu-system-riscv64
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

# Custom configuration paths
export MULTIOS_TEST_RESULTS_DIR=test_results
export MULTIOS_PERFORMANCE_REPORTS_DIR=performance_reports
export MULTIOS_STRESS_TEST_RESULTS_DIR=stress_test_results
export MULTIOS_COVERAGE_REPORTS_DIR=coverage_reports
EOF
    
    print_success "Environment file created: $env_file"
    print_status "Source the environment file with: source $env_file"
}

# Build the project
build_project() {
    print_status "Building MultiOS Comprehensive Testing Suite..."
    
    if cargo build --release; then
        print_success "Project built successfully"
    else
        print_error "Project build failed"
        return 1
    fi
}

# Run initial tests
run_initial_tests() {
    print_status "Running initial tests..."
    
    if cargo test --lib; then
        print_success "Initial tests passed"
    else
        print_error "Initial tests failed"
        return 1
    fi
}

# Show next steps
show_next_steps() {
    echo
    echo "=================================================="
    echo "  MultiOS Comprehensive Testing Suite Setup Complete!"
    echo "=================================================="
    echo
    echo "Next steps:"
    echo "1. Source the environment file:"
    echo "   source .env"
    echo
    echo "2. Run the comprehensive test suite:"
    echo "   cargo run --bin multios_test_runner all"
    echo
    echo "3. Run specific test categories:"
    echo "   cargo run --bin multios_test_runner category --category Unit"
    echo "   cargo run --bin multios_test_runner category --category Integration"
    echo
    echo "4. Run performance monitoring:"
    echo "   cargo run --bin multios_performance_monitor --duration 60"
    echo
    echo "5. Run stress testing:"
    echo "   cargo run --bin multios_stress_tester --profile balanced"
    echo
    echo "6. Generate coverage reports:"
    echo "   cargo run --bin multios_coverage_analyzer --format html"
    echo
    echo "Available make targets:"
    echo "   make help              - Show all available targets"
    echo "   make build             - Build the project"
    echo "   make test              - Run all tests"
    echo "   make validate          - Validate environment"
    echo "   make coverage          - Generate coverage reports"
    echo "   make comprehensive-test - Run comprehensive test suite"
    echo
    echo "For more information, see README.md"
    echo
}

# Main setup function
main() {
    echo "=================================================="
    echo "  MultiOS Comprehensive Testing Suite"
    echo "  Development Environment Setup"
    echo "=================================================="
    echo
    
    # Detect operating system
    detect_os
    print_status "Detected OS: $OS"
    echo
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the comprehensive_testing_suite directory."
        exit 1
    fi
    
    # Install components
    install_rust
    install_system_deps
    install_rust_tools
    
    # Verify installations
    verify_qemu
    
    # Create project structure
    create_project_structure
    
    # Setup environment
    setup_environment
    
    # Build project
    build_project
    
    # Run initial tests
    run_initial_tests
    
    # Show next steps
    show_next_steps
}

# Handle script arguments
case "${1:-}" in
    "--help"|"-h")
        echo "MultiOS Comprehensive Testing Suite - Development Environment Setup"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --verify-only  Only verify existing installation"
        echo "  --minimal      Minimal setup without optional components"
        echo
        exit 0
        ;;
    "--verify-only")
        print_status "Verifying existing installation..."
        verify_qemu
        cargo --version
        rustc --version
        print_success "Verification complete"
        exit 0
        ;;
    "--minimal")
        print_status "Performing minimal setup..."
        install_rust
        install_rust_tools
        build_project
        run_initial_tests
        show_next_steps
        ;;
    *)
        main
        ;;
esac
