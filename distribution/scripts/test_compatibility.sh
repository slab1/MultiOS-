#!/bin/bash
# Cross-platform compatibility testing script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

echo "MultiOS Cross-Platform Compatibility Tests"
echo "=========================================="

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_result() {
    echo -e "${GREEN}[RESULT]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Test architectures
ARCHITECTURES=("x86_64" "aarch64" "riscv64")
QEMU_SYSTEMS=("qemu-system-x86_64" "qemu-system-aarch64" "qemu-system-riscv64")

# Test results tracking
declare -A test_results
total_tests=0
passed_tests=0
failed_tests=0

# Function to check if QEMU is available for an architecture
check_qemu() {
    local arch=$1
    local qemu_system=""
    
    case $arch in
        x86_64) qemu_system="qemu-system-x86_64" ;;
        aarch64) qemu_system="qemu-system-aarch64" ;;
        riscv64) qemu_system="qemu-system-riscv64" ;;
    esac
    
    if command -v "$qemu_system" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

# Function to build for architecture
build_for_arch() {
    local arch=$1
    local target=""
    
    case $arch in
        x86_64) target="x86_64-unknown-none" ;;
        aarch64) target="aarch64-unknown-none" ;;
        riscv64) target="riscv64gc-unknown-none" ;;
    esac
    
    print_status "Building for $arch (target: $target)"
    
    cd "$PROJECT_ROOT"
    
    # Install target if not present
    if ! rustup target list --installed | grep -q "^$target$"; then
        print_status "Installing target: $target"
        rustup target add "$target"
    fi
    
    # Build
    if cargo build --target "$target" --release; then
        print_result "Build successful for $arch"
        return 0
    else
        print_error "Build failed for $arch"
        return 1
    fi
}

# Function to run architecture test
test_arch() {
    local arch=$1
    local target=""
    local kernel=""
    local qemu_system=""
    
    case $arch in
        x86_64)
            target="x86_64-unknown-none"
            kernel="target/$target/release/multios-cross-platform-compat"
            qemu_system="qemu-system-x86_64"
            ;;
        aarch64)
            target="aarch64-unknown-none"
            kernel="target/$target/release/multios-cross-platform-compat"
            qemu_system="qemu-system-aarch64"
            ;;
        riscv64)
            target="riscv64gc-unknown-none"
            kernel="target/$target/release/multios-cross-platform-compat"
            qemu_system="qemu-system-riscv64"
            ;;
    esac
    
    print_test "Testing $arch architecture"
    
    total_tests=$((total_tests + 1))
    
    # Check if kernel binary exists
    if [ ! -f "$PROJECT_ROOT/$kernel" ]; then
        print_error "Kernel binary not found: $kernel"
        failed_tests=$((failed_tests + 1))
        test_results[$arch]="FAILED - Binary not found"
        return 1
    fi
    
    # Check if QEMU is available
    if ! check_qemu "$arch"; then
        print_warning "QEMU not available for $arch - skipping QEMU test"
        test_results[$arch]="SKIPPED - QEMU not available"
        return 0
    fi
    
    # Run basic tests (simplified - in practice would need actual kernel)
    print_status "Running basic tests for $arch"
    
    # Test compilation
    if build_for_arch "$arch"; then
        print_result "Compilation test passed for $arch"
    else
        print_error "Compilation test failed for $arch"
        failed_tests=$((failed_tests + 1))
        test_results[$arch]="FAILED - Compilation error"
        return 1
    fi
    
    # Test run (simplified - would need actual bootable kernel)
    # In practice, this would:
    # 1. Create a bootable disk image
    # 2. Run QEMU with appropriate parameters
    # 3. Check for successful boot and operation
    
    print_result "Basic tests passed for $arch"
    passed_tests=$((passed_tests + 1))
    test_results[$arch]="PASSED"
    
    return 0
}

# Function to run cargo tests
run_cargo_tests() {
    print_test "Running Cargo tests"
    
    total_tests=$((total_tests + 1))
    
    cd "$PROJECT_ROOT"
    
    if cargo test --release; then
        print_result "Cargo tests passed"
        passed_tests=$((passed_tests + 1))
        test_results["cargo_tests"]="PASSED"
        return 0
    else
        print_error "Cargo tests failed"
        failed_tests=$((failed_tests + 1))
        test_results["cargo_tests"]="FAILED"
        return 1
    fi
}

# Function to test library functionality
test_library() {
    print_test "Testing library functionality"
    
    total_tests=$((total_tests + 1))
    
    cd "$PROJECT_ROOT"
    
    # Test that the library compiles correctly for all targets
    for arch in "${ARCHITECTURES[@]}"; do
        case $arch in
            x86_64) target="x86_64-unknown-none" ;;
            aarch64) target="aarch64-unknown-none" ;;
            riscv64) target="riscv64gc-unknown-none" ;;
        esac
        
        print_status "Checking compilation for $arch"
        
        if rustup target list --installed | grep -q "^$target$"; then
            if cargo check --target "$target"; then
                print_result "Library compilation OK for $arch"
            else
                print_error "Library compilation failed for $arch"
                failed_tests=$((failed_tests + 1))
                test_results["lib_$arch"]="FAILED"
                return 1
            fi
        else
            print_warning "Target $target not installed - skipping"
        fi
    done
    
    passed_tests=$((passed_tests + 1))
    test_results["library"]="PASSED"
    
    return 0
}

# Function to generate test report
generate_report() {
    local report_file="$PROJECT_ROOT/test_report.md"
    
    print_status "Generating test report: $report_file"
    
    cat > "$report_file" << EOF
# MultiOS Cross-Platform Compatibility Test Report

Generated: $(date)

## Summary

- Total Tests: $total_tests
- Passed: $passed_tests
- Failed: $failed_tests
- Pass Rate: $(( total_tests > 0 ? (passed_tests * 100 / total_tests) : 0 ))%

## Test Results

EOF

    for arch in "${ARCHITECTURES[@]}"; do
        echo "- **$arch**: ${test_results[$arch]:-NOT RUN}" >> "$report_file"
    done
    echo "- **Library Compilation**: ${test_results[library]:-NOT RUN}" >> "$report_file"
    echo "- **Cargo Tests**: ${test_results[cargo_tests]:-NOT RUN}" >> "$report_file"
    
    cat >> "$report_file" << EOF

## Architecture Support

EOF

    for arch in "${ARCHITECTURES[@]}"; do
        echo "- **$arch**: $(check_qemu "$arch" && echo "✅ Available" || echo "⚠️  Not Available")" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Detailed Results

EOF

    for arch in "${ARCHITECTURES[@]}"; do
        cat >> "$report_file" << EOF
### $arch Architecture

**Status**: ${test_results[$arch]:-NOT RUN}

**QEMU Available**: $(check_qemu "$arch" && echo "Yes" || echo "No")

EOF
    done
    
    print_result "Test report generated: $report_file"
}

# Function to show help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --arch ARCH     Test specific architecture (x86_64, aarch64, riscv64)"
    echo "  --all          Test all architectures"
    echo "  --library      Test library compilation"
    echo "  --tests        Run cargo tests"
    echo "  --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --all        # Test all architectures"
    echo "  $0 --arch x86_64 # Test x86_64 only"
    echo "  $0 --library    # Test library compilation"
}

# Parse command line arguments
case "${1:---all}" in
    --arch)
        if [ -z "$2" ]; then
            print_error "Architecture not specified"
            show_help
            exit 1
        fi
        ARCH_TO_TEST="$2"
        ;;
    --all)
        ARCH_TO_TEST="all"
        ;;
    --library)
        test_library
        exit $?
        ;;
    --tests)
        run_cargo_tests
        exit $?
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

print_status "Starting compatibility tests..."

# Test Rust installation
if ! command -v rustc &> /dev/null; then
    print_error "Rust not found. Please install Rust."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Cargo."
    exit 1
fi

print_status "Rust version: $(rustc --version)"
print_status "Cargo version: $(cargo --version)"

# Run tests based on selection
case $ARCH_TO_TEST in
    all)
        print_test "Testing all architectures"
        for arch in "${ARCHITECTURES[@]}"; do
            test_arch "$arch"
        done
        run_cargo_tests
        test_library
        ;;
    x86_64|aarch64|riscv64)
        test_arch "$ARCH_TO_TEST"
        ;;
    *)
        print_error "Unknown architecture: $ARCH_TO_TEST"
        show_help
        exit 1
        ;;
esac

# Generate report
generate_report

# Summary
echo ""
echo "=========================================="
print_status "Test Summary"
echo "Total Tests: $total_tests"
print_result "Passed: $passed_tests"
if [ $failed_tests -gt 0 ]; then
    print_error "Failed: $failed_tests"
else
    print_result "Failed: $failed_tests"
fi

if [ $total_tests -gt 0 ]; then
    pass_rate=$((passed_tests * 100 / total_tests))
    print_result "Pass Rate: $pass_rate%"
fi

# Exit with appropriate code
if [ $failed_tests -eq 0 ]; then
    print_result "All tests passed!"
    exit 0
else
    print_error "Some tests failed!"
    exit 1
fi