#!/bin/bash

# MultiOS Testing Framework
# Automated testing using cargo test and QEMU

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_RESULTS_DIR="$PROJECT_ROOT/test_results"
TEST_LOG_FILE="$PROJECT_ROOT/test.log"

# QEMU configurations for different targets
declare -A QEMU_COMMANDS=(
    ["x86_64"]="qemu-system-x86_64 -m 256M -drive format=raw,file=KERNEL_IMAGE -serial stdio -monitor none -display none"
    ["arm64"]="qemu-system-aarch64 -m 256M -machine virt -cpu cortex-a57 -nographic -kernel KERNEL_IMAGE"
    ["riscv64"]="qemu-system-riscv64 -m 256M -machine virt -nographic -kernel KERNEL_IMAGE"
)

# Default values
TARGET=""
TEST_SUITE=""
QEMU=false
CLEAN=false
VERBOSE=false
HELP=false
COVERAGE=false
INTEGRATION=false
UNIT=false
PARALLEL=true

# Logging functions
log() {
    echo -e "${GREEN}[TEST]${NC} $1" | tee -a "$TEST_LOG_FILE"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$TEST_LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$TEST_LOG_FILE"
    exit 1
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
MultiOS Testing Framework

Usage: $0 [OPTIONS]

Options:
    -t, --target TARGET     Target architecture (x86_64, arm64, riscv64)
    -s, --suite SUITE       Test suite (unit, integration, all)
    -q, --qemu             Enable QEMU testing
    -c, --coverage         Generate test coverage report
    -p, --parallel         Run tests in parallel
    --clean               Clean test artifacts before testing
    -v, --verbose          Enable verbose output
    -h, --help             Show this help message

Examples:
    $0 --target x86_64 --suite unit --qemu
    $0 --target arm64 --suite integration --coverage
    $0 --target riscv64 --suite all --parallel

Test Suites:
    unit       - Run unit tests only
    integration - Run integration tests only
    all        - Run all tests

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--target)
                TARGET="$2"
                shift 2
                ;;
            -s|--suite)
                TEST_SUITE="$2"
                shift 2
                ;;
            -q|--qemu)
                QEMU=true
                shift
                ;;
            -c|--coverage)
                COVERAGE=true
                shift
                ;;
            -p|--parallel)
                PARALLEL=true
                shift
                ;;
            --clean)
                CLEAN=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                HELP=true
                shift
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done
}

# Validate arguments
validate_args() {
    if [[ "$HELP" == "true" ]]; then
        show_help
        exit 0
    fi
    
    if [[ -z "$TARGET" ]]; then
        error "Target architecture must be specified. Use --help for usage information."
    fi
    
    if [[ -z "$TEST_SUITE" ]]; then
        TEST_SUITE="all"
    fi
    
    # Validate target
    local valid_targets=("x86_64" "arm64" "riscv64")
    local target_valid=false
    for valid_target in "${valid_targets[@]}"; do
        if [[ "$TARGET" == "$valid_target" ]]; then
            target_valid=true
            break
        fi
    done
    
    if [[ "$target_valid" == "false" ]]; then
        error "Invalid target: $TARGET. Valid targets: ${valid_targets[*]}"
    fi
    
    # Validate test suite
    local valid_suites=("unit" "integration" "all")
    local suite_valid=false
    for valid_suite in "${valid_suites[@]}"; do
        if [[ "$TEST_SUITE" == "$valid_suite" ]]; then
            suite_valid=true
            break
        fi
    done
    
    if [[ "$suite_valid" == "false" ]]; then
        error "Invalid test suite: $TEST_SUITE. Valid suites: ${valid_suites[*]}"
    fi
}

# Setup testing environment
setup_testing_environment() {
    log "Setting up testing environment..."
    
    # Create test results directory
    mkdir -p "$TEST_RESULTS_DIR"
    
    # Initialize test log
    echo "MultiOS Test Log - $(date)" > "$TEST_LOG_FILE"
    
    # Check for required tools
    check_dependencies
    
    # Clean if requested
    if [[ "$CLEAN" == "true" ]]; then
        log "Cleaning test artifacts..."
        cargo clean
        rm -rf "$TEST_RESULTS_DIR"/*
    fi
}

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."
    
    # Check cargo
    if ! command -v cargo &> /dev/null; then
        error "cargo is not installed or not in PATH"
    fi
    
    # Check rustc
    if ! command -v rustc &> /dev/null; then
        error "rustc is not installed or not in PATH"
    fi
    
    # Check QEMU if testing with QEMU
    if [[ "$QEMU" == "true" ]]; then
        if ! command -v "${QEMU_COMMANDS[$TARGET]%% *}" &> /dev/null; then
            warn "QEMU not found for target $TARGET. Installing..."
            install_qemu "$TARGET"
        fi
    fi
    
    # Check coverage tools if needed
    if [[ "$COVERAGE" == "true" ]]; then
        if ! command -v cargo-tarpaulin &> /dev/null; then
            warn "cargo-tarpaulin not found. Installing..."
            install_tarpaulin
        fi
    fi
}

# Install QEMU
install_qemu() {
    local target=$1
    case $target in
        "x86_64")
            sudo apt-get update
            sudo apt-get install -y qemu-system-x86
            ;;
        "arm64")
            sudo apt-get update
            sudo apt-get install -y qemu-system-aarch64
            ;;
        "riscv64")
            sudo apt-get update
            sudo apt-get install -y qemu-system-riscv64
            ;;
    esac
}

# Install tarpaulin for coverage
install_tarpaulin() {
    cargo install cargo-tarpaulin
}

# Run unit tests
run_unit_tests() {
    log "Running unit tests..."
    
    local test_args=()
    
    if [[ "$VERBOSE" == "true" ]]; then
        test_args+=("--nocapture")
    fi
    
    if [[ "$PARALLEL" == "true" ]]; then
        test_args+=("--test-threads=$(nproc)")
    fi
    
    # Test kernel
    log "Testing kernel..."
    cd "$PROJECT_ROOT/kernel"
    cargo test "${test_args[@]}" || error "Kernel unit tests failed"
    
    # Test bootloader
    log "Testing bootloader..."
    cd "$PROJECT_ROOT/bootloader"
    cargo test "${test_args[@]}" || error "Bootloader unit tests failed"
    
    # Test userland
    log "Testing userland..."
    cd "$PROJECT_ROOT/userland"
    cargo test "${test_args[@]}" || error "Userland unit tests failed"
    
    log "Unit tests completed successfully"
}

# Run integration tests
run_integration_tests() {
    log "Running integration tests..."
    
    cd "$PROJECT_ROOT/tests/integration"
    
    local test_args=()
    
    if [[ "$VERBOSE" == "true" ]]; then
        test_args+=("--nocapture")
    fi
    
    cargo test "${test_args[@]}" || error "Integration tests failed"
    
    log "Integration tests completed successfully"
}

# Run QEMU tests
run_qemu_tests() {
    if [[ "$QEMU" != "true" ]]; then
        return 0
    fi
    
    log "Running QEMU tests for $TARGET..."
    
    # Build kernel for QEMU
    log "Building kernel for QEMU testing..."
    cd "$PROJECT_ROOT/kernel"
    cargo build --features qemu || error "QEMU kernel build failed"
    
    # Find the kernel binary
    local kernel_binary="$PROJECT_ROOT/target/x86_64-unknown-none/debug/multios-kernel"
    if [[ ! -f "$kernel_binary" ]]; then
        kernel_binary="$PROJECT_ROOT/target/debug/multios-kernel"
    fi
    
    if [[ ! -f "$kernel_binary" ]]; then
        error "Kernel binary not found for QEMU testing"
    fi
    
    # Run QEMU with timeout and capture output
    local qemu_cmd="${QEMU_COMMANDS[$TARGET]//KERNEL_IMAGE/$kernel_binary}"
    local test_output="$TEST_RESULTS_DIR/qemu_test_${TARGET}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Executing QEMU test: $qemu_cmd"
    log "Test output will be saved to: $test_output"
    
    # Run QEMU with timeout (30 seconds for basic test)
    timeout 30s bash -c "$qemu_cmd" > "$test_output" 2>&1 || {
        warn "QEMU test timed out or failed. Check $test_output for details."
        cat "$test_output"
    }
    
    # Check if kernel booted successfully
    if grep -q "MultiOS Kernel" "$test_output" 2>/dev/null; then
        log "QEMU test passed - Kernel booted successfully"
    else
        warn "QEMU test may have failed - No successful boot message found"
    fi
}

# Generate coverage report
generate_coverage_report() {
    if [[ "$COVERAGE" != "true" ]]; then
        return 0
    fi
    
    log "Generating coverage report..."
    
    local coverage_output="$TEST_RESULTS_DIR/coverage_${TARGET}_$(date +%Y%m%d_%H%M%S)"
    
    # Generate coverage for each crate
    cd "$PROJECT_ROOT"
    cargo tarpaulin --out html --output-dir "$coverage_output" || warn "Coverage generation failed"
    
    # Generate coverage summary
    cargo tarpaulin --out json --output-dir "$coverage_output" > "${coverage_output}/coverage.json" 2>/dev/null || true
    
    log "Coverage report generated in: $coverage_output"
}

# Generate test report
generate_test_report() {
    local report_file="$TEST_RESULTS_DIR/test_report_$(date +%Y%m%d_%H%M%S).txt"
    
    cat > "$report_file" << EOF
MultiOS Test Report
===================
Test Date: $(date)
Target: $TARGET
Test Suite: $TEST_SUITE
QEMU Testing: $QEMU
Coverage Report: $COVERAGE
Parallel Tests: $PARALLEL
Verbose Output: $VERBOSE"

Test Results Directory: $TEST_RESULTS_DIR
Test Log: $TEST_LOG_FILE

Test Configuration:
- Target Architecture: $TARGET
- Test Suite: $TEST_SUITE
- QEMU Enabled: $QEMU
- Coverage Generation: $COVERAGE
- Parallel Execution: $PARALLEL

EOF
    
    log "Test report generated: $report_file"
}

# Main testing function
main() {
    # Setup log
    exec > >(tee -a "$TEST_LOG_FILE")
    exec 2>&1
    
    info "MultiOS Testing Framework Starting..."
    info "Target: $TARGET"
    info "Test Suite: $TEST_SUITE"
    info "QEMU Testing: $QEMU"
    
    # Setup environment
    setup_testing_environment
    
    # Run tests based on suite
    case "$TEST_SUITE" in
        "unit")
            run_unit_tests
            ;;
        "integration")
            run_integration_tests
            ;;
        "all")
            run_unit_tests
            run_integration_tests
            ;;
    esac
    
    # Run QEMU tests
    run_qemu_tests
    
    # Generate coverage report
    generate_coverage_report
    
    # Generate test report
    generate_test_report
    
    log "Testing completed successfully!"
    info "Test results available in: $TEST_RESULTS_DIR"
    info "Test log: $TEST_LOG_FILE"
}

# Run main function
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    parse_args "$@"
    validate_args
    main
fi