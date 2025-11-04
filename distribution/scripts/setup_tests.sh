#!/bin/bash

# Bootloader Test Configuration Script
# Configures and validates the testing environment

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Print functions
print_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $*"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $*"; }
print_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Check system requirements
check_system_requirements() {
    print_info "Checking system requirements..."
    
    # Check OS
    case "$(uname -s)" in
        Linux*)
            OS="Linux"
            ;;
        Darwin*)
            OS="macOS"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            OS="Windows"
            ;;
        *)
            OS="Unknown"
            ;;
    esac
    
    print_info "Operating System: $OS"
    
    # Check architecture
    ARCH=$(uname -m)
    print_info "Architecture: $ARCH"
    
    # Check available memory
    if command -v free &> /dev/null; then
        MEMORY_KB=$(free | grep '^Mem:' | awk '{print $2}')
        MEMORY_GB=$((MEMORY_KB / 1024 / 1024))
        print_info "Available Memory: ${MEMORY_GB}GB"
        
        if [ $MEMORY_GB -lt 4 ]; then
            print_warning "Less than 4GB RAM available. Some tests may be slow."
        fi
    elif command -v vm_stat &> /dev/null; then
        print_info "macOS detected - memory check not implemented"
    fi
    
    # Check disk space
    DISK_AVAILABLE=$(df -BG "$PROJECT_ROOT" | awk 'NR==2 {print $4}' | sed 's/G//')
    print_info "Available Disk Space: ${DISK_AVAILABLE}GB"
    
    if [ $DISK_AVAILABLE -lt 10 ]; then
        print_warning "Less than 10GB disk space available. Tests may fail."
    fi
}

# Install system dependencies
install_dependencies() {
    print_info "Installing system dependencies..."
    
    case "$OS" in
        "Linux")
            if command -v apt-get &> /dev/null; then
                # Ubuntu/Debian
                print_info "Detected Ubuntu/Debian system"
                sudo apt-get update
                sudo apt-get install -y \
                    build-essential \
                    curl \
                    wget \
                    git \
                    pkg-config \
                    libssl-dev \
                    qemu-system-x86 \
                    qemu-system-arm \
                    qemu-system-riscv64 \
                    qemu-utils \
                    ovmf \
                    gcc-aarch64-linux-gnu \
                    gcc-riscv64-linux-gnu \
                    gdb-multiarch \
                    make \
                    python3 \
                    python3-pip
            elif command -v dnf &> /dev/null; then
                # Fedora/RHEL
                print_info "Detected Fedora/RHEL system"
                sudo dnf install -y \
                    gcc \
                    curl \
                    wget \
                    git \
                    pkgconfig \
                    openssl-devel \
                    qemu-system-x86 \
                    qemu-system-aarch64 \
                    qemu-system-riscv64 \
                    qemu-img \
                    edk2-ovmf \
                    gcc-aarch64-linux-gnu \
                    gcc-riscv64-linux-gnu \
                    gdb \
                    make \
                    python3 \
                    python3-pip
            elif command -v pacman &> /dev/null; then
                # Arch Linux
                print_info "Detected Arch Linux system"
                sudo pacman -Sy --needed \
                    base-devel \
                    curl \
                    wget \
                    git \
                    pkg-config \
                    openssl \
                    qemu \
                    qemu-arch-extra \
                    edk2-ovmf \
                    aarch64-linux-gnu-gcc \
                    riscv64-linux-gnu-gcc \
                    gdb \
                    make \
                    python3
            else
                print_error "Unsupported Linux distribution"
                return 1
            fi
            ;;
        "macOS")
            print_info "Detected macOS system"
            
            # Check for Homebrew
            if ! command -v brew &> /dev/null; then
                print_info "Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            
            # Install dependencies via Homebrew
            brew install \
                gcc \
                curl \
                wget \
                git \
                pkg-config \
                openssl \
                qemu \
                gdb
            ;;
        "Windows")
            print_warning "Windows detected. Please install dependencies manually:"
            print_warning "- QEMU: https://www.qemu.org/download/#windows"
            print_warning "- Rust: https://rustup.rs/"
            print_warning "- Git: https://git-scm.com/download/win"
            return 1
            ;;
        *)
            print_error "Unknown operating system"
            return 1
            ;;
    esac
    
    print_success "System dependencies installed"
}

# Install Rust toolchain
install_rust() {
    print_info "Installing Rust toolchain..."
    
    if command -v rustc &> /dev/null; then
        print_info "Rust already installed"
        rustup update
        return 0
    fi
    
    # Download and install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the cargo environment
    source "$HOME/.cargo/env"
    
    # Install additional targets for cross-compilation
    rustup target add x86_64-unknown-none-elf
    rustup target add aarch64-unknown-none-elf
    rustup target add riscv64gc-unknown-none-elf
    
    print_success "Rust toolchain installed"
}

# Install Rust tools and testing utilities
install_rust_tools() {
    print_info "Installing Rust testing tools..."
    
    cargo install \
        cargo-watch \
        cargo-audit \
        cargo-outdated \
        cargo-semver-checks
    
    # Install additional testing tools
    if [ "$OS" = "Linux" ]; then
        # Install addr2line for better panic messages
        sudo apt-get install -y binutils-dev 2>/dev/null || \
        sudo dnf install -y binutils-devel 2>/dev/null || \
        sudo pacman -S --needed binutils 2>/dev/null || true
    fi
    
    print_success "Rust tools installed"
}

# Create test environment setup
setup_test_environment() {
    print_info "Setting up test environment..."
    
    local test_dir="$PROJECT_ROOT/test_environment"
    mkdir -p "$test_dir"
    
    # Create directories
    mkdir -p "$test_dir"/{kernels,disks,logs,results,configs}
    
    # Create default test configurations
    create_test_configs "$test_dir/configs"
    
    # Set up environment variables
    cat > "$test_dir/env.sh" << EOF
#!/bin/bash
# MultiOS Bootloader Testing Environment

export MULTIOS_TEST_DIR="$test_dir"
export MULTIOS_KERNEL_DIR="\$MULTIOS_TEST_DIR/kernels"
export MULTIOS_DISK_DIR="\$MULTIOS_TEST_DIR/disks"
export MULTIOS_LOG_DIR="\$MULTIOS_TEST_DIR/logs"
export MULTIOS_RESULT_DIR="\$MULTIOS_TEST_DIR/results"
export MULTIOS_CONFIG_DIR="\$MULTIOS_TEST_DIR/configs"

# QEMU settings
export QEMU_AUDIO_DRV=none
export SDL_VIDEODRIVER=dummy

# Add to PATH
export PATH="\$PATH:\$MULTIOS_TEST_DIR/scripts"

echo "MultiOS test environment activated"
echo "Test directory: \$MULTIOS_TEST_DIR"
EOF
    
    chmod +x "$test_dir/env.sh"
    
    # Create test scripts
    create_test_scripts "$test_dir/scripts"
    
    print_success "Test environment set up at: $test_dir"
    print_info "To activate environment: source $test_dir/env.sh"
}

# Create test configurations
create_test_configs() {
    local config_dir="$1"
    
    # x86_64 configuration
    cat > "$config_dir/x86_64_test.toml" << EOF
[architecture]
name = "x86_64"
qemu_binary = "qemu-system-x86_64"
machine = "pc"
memory = "512M"
cpus = 2

[boot]
boot_modes = ["uefi", "legacy"]
timeout = 30

[memory]
test_sizes = ["512M", "1G", "2G"]

[performance]
iterations = 5
timeout = 15
EOF
    
    # ARM64 configuration
    cat > "$config_dir/aarch64_test.toml" << EOF
[architecture]
name = "aarch64"
qemu_binary = "qemu-system-aarch64"
machine = "virt"
memory = "1G"
cpus = 2

[boot]
boot_modes = ["uefi"]
timeout = 30

[memory]
test_sizes = ["1G", "2G"]

[performance]
iterations = 3
timeout = 15
EOF
    
    # RISC-V configuration
    cat > "$config_dir/riscv64_test.toml" << EOF
[architecture]
name = "riscv64"
qemu_binary = "qemu-system-riscv64"
machine = "virt"
memory = "1G"
cpus = 2

[boot]
boot_modes = ["uefi"]
timeout = 30

[memory]
test_sizes = ["1G", "2G"]

[performance]
iterations = 3
timeout = 15
EOF
    
    # Global test configuration
    cat > "$config_dir/global.toml" << EOF
[logging]
level = "INFO"
file = "\${MULTIOS_TEST_DIR}/logs/test.log"
structured = true

[testing]
parallel = true
max_concurrent = 4
cleanup_after_tests = true

[reporting]
formats = ["html", "json"]
output_dir = "\${MULTIOS_TEST_DIR}/results"
EOF
}

# Create test helper scripts
create_test_scripts() {
    local script_dir="$1"
    
    mkdir -p "$script_dir"
    
    # Quick test script
    cat > "$script_dir/quick_test.sh" << 'EOF'
#!/bin/bash
# Quick bootloader test

source "$(dirname "$0")/../env.sh"

echo "Running quick bootloader test..."

# Test basic boot on x86_64
timeout 30 qemu-system-x86_64 \
    -m 512M \
    -nographic \
    -kernel "$MULTIOS_KERNEL_DIR/test_kernel_x86_64.bin" \
    -append "console=ttyS0 quiet"

echo "Quick test completed"
EOF
    chmod +x "$script_dir/quick_test.sh"
    
    # Benchmark script
    cat > "$script_dir/benchmark.sh" << 'EOF'
#!/bin/bash
# Bootloader benchmark

source "$(dirname "$0")/../env.sh"

echo "Running bootloader benchmark..."

for arch in x86_64 aarch64 riscv64; do
    echo "Benchmarking $arch..."
    
    for i in {1..5}; do
        start_time=$(date +%s%N)
        
        timeout 20 qemu-system-$arch \
            -m 1G \
            -nographic \
            -kernel "$MULTIOS_KERNEL_DIR/test_kernel_${arch}.bin" \
            -append "console=ttyS0" &
        
        qemu_pid=$!
        wait $qemu_pid
        
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 ))
        
        echo "$arch iteration $i: ${duration}ms"
    done
done
EOF
    chmod +x "$script_dir/benchmark.sh"
    
    # Test monitoring script
    cat > "$script_dir/monitor.sh" << 'EOF'
#!/bin/bash
# Test monitoring script

source "$(dirname "$0")/../env.sh"

echo "MultiOS Test Monitor"
echo "==================="

# Check for running QEMU instances
qemu_count=$(pgrep -f "qemu-system" | wc -l)
echo "Running QEMU instances: $qemu_count"

# Check disk usage
disk_usage=$(df -h "$MULTIOS_TEST_DIR" | tail -1 | awk '{print $5}' | sed 's/%//')
echo "Disk usage: ${disk_usage}%"

# Check log size
if [ -d "$MULTIOS_LOG_DIR" ]; then
    log_size=$(du -sh "$MULTIOS_LOG_DIR" 2>/dev/null | cut -f1)
    echo "Log directory size: $log_size"
fi

# Show recent test results
if [ -d "$MULTIOS_RESULT_DIR" ]; then
    echo "Recent test results:"
    ls -lt "$MULTIOS_RESULT_DIR"/*.html 2>/dev/null | head -5 | awk '{print $9}' | xargs -I {} basename {}
fi
EOF
    chmod +x "$script_dir/monitor.sh"
}

# Validate installation
validate_installation() {
    print_info "Validating installation..."
    
    local errors=0
    
    # Check QEMU installation
    for qemu in qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64; do
        if command -v "$qemu" &> /dev/null; then
            version=$($qemu --version | head -1)
            print_success "$qemu: $version"
        else
            print_error "$qemu: Not found"
            errors=$((errors + 1))
        fi
    done
    
    # Check Rust installation
    if command -v rustc &> /dev/null && command -v cargo &> /dev/null; then
        rust_version=$(rustc --version)
        cargo_version=$(cargo --version)
        print_success "Rust: $rust_version"
        print_success "Cargo: $cargo_version"
    else
        print_error "Rust toolchain: Not found"
        errors=$((errors + 1))
    fi
    
    # Check cross-compilation targets
    if command -v rustup &> /dev/null; then
        for target in x86_64-unknown-none-elf aarch64-unknown-none-elf riscv64gc-unknown-none-elf; do
            if rustup target list --installed 2>/dev/null | grep -q "$target"; then
                print_success "Target $target: Installed"
            else
                print_warning "Target $target: Not installed"
            fi
        done
    fi
    
    # Check test environment
    if [ -d "$PROJECT_ROOT/test_environment" ]; then
        print_success "Test environment: Created"
    else
        print_error "Test environment: Not found"
        errors=$((errors + 1))
    fi
    
    if [ $errors -eq 0 ]; then
        print_success "All validation checks passed"
        return 0
    else
        print_error "$errors validation checks failed"
        return 1
    fi
}

# Show help
show_help() {
    cat << EOF
MultiOS Bootloader Test Configuration

Usage: $0 [OPTIONS]

Options:
    --check         Check system requirements only
    --install       Install all dependencies
    --setup         Set up test environment only
    --validate      Validate installation
    --all           Run all setup steps (default)
    --help          Show this help message

Examples:
    $0 --all                # Complete setup
    $0 --check              # Check requirements only
    $0 --install --setup    # Install and setup
    $0 --validate           # Validate installation

After setup, activate the test environment:
    source test_environment/env.sh
EOF
}

# Main function
main() {
    local check_only=false
    local install_only=false
    local setup_only=false
    local validate_only=false
    local run_all=true
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --check)
                check_only=true
                run_all=false
                shift
                ;;
            --install)
                install_only=true
                run_all=false
                shift
                ;;
            --setup)
                setup_only=true
                run_all=false
                shift
                ;;
            --validate)
                validate_only=true
                run_all=false
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    print_info "MultiOS Bootloader Test Configuration"
    echo
    
    if [ "$check_only" = true ]; then
        check_system_requirements
        return 0
    fi
    
    if [ "$install_only" = true ]; then
        install_dependencies
        install_rust
        install_rust_tools
        return 0
    fi
    
    if [ "$setup_only" = true ]; then
        setup_test_environment
        return 0
    fi
    
    if [ "$validate_only" = true ]; then
        validate_installation
        return 0
    fi
    
    if [ "$run_all" = true ]; then
        check_system_requirements
        echo
        
        install_dependencies
        echo
        
        install_rust
        echo
        
        install_rust_tools
        echo
        
        setup_test_environment
        echo
        
        validate_installation
        
        print_success "Setup completed successfully!"
        print_info "To activate test environment: source test_environment/env.sh"
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
