#!/bin/bash

# MultiOS Bootloader Testing Script
# Comprehensive testing script for bootloader validation across multiple architectures

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${PROJECT_ROOT}/test_results"
LOG_FILE="${OUTPUT_DIR}/bootloader_test_$(date +%Y%m%d_%H%M%S).log"
CONFIG_FILE="${PROJECT_ROOT}/configs/testing_config.json"

# Test configurations
declare -A ARCH_CONFIGS=(
    ["x86_64"]="pc:512M:2"
    ["aarch64"]="virt:1G:2" 
    ["riscv64"]="virt:1G:2"
)

declare -A BOOT_MODES=("uefi" "legacy")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
    
    case $level in
        "ERROR")   echo -e "${RED}[ERROR]${NC} $message" >&2 ;;
        "SUCCESS") echo -e "${GREEN}[SUCCESS]${NC} $message" ;;
        "WARNING") echo -e "${YELLOW}[WARNING]${NC} $message" ;;
        "INFO")    echo -e "${BLUE}[INFO]${NC} $message" ;;
    esac
}

# Check dependencies
check_dependencies() {
    log "INFO" "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for required tools
    for tool in qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64; do
        if ! command -v "$tool" &> /dev/null; then
            missing_deps+=("$tool")
        fi
    done
    
    # Check for rust/cargo
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi
    
    # Check for build dependencies
    for tool in make python3; do
        if ! command -v "$tool" &> /dev/null; then
            missing_deps+=("$tool")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log "ERROR" "Missing dependencies: ${missing_deps[*]}"
        log "INFO" "Please install missing dependencies and try again"
        exit 1
    fi
    
    log "SUCCESS" "All dependencies satisfied"
}

# Build bootloader and test binaries
build_test_binaries() {
    log "INFO" "Building test binaries..."
    
    cd "$PROJECT_ROOT"
    
    # Build bootloader
    if [ -f "Cargo.toml" ]; then
        log "INFO" "Building bootloader..."
        cargo build --release || {
            log "ERROR" "Bootloader build failed"
            exit 1
        }
    fi
    
    # Build bootloader testing framework
    if [ -d "bootloader_testing" ]; then
        log "INFO" "Building bootloader testing framework..."
        cd bootloader_testing
        cargo build --release || {
            log "ERROR" "Testing framework build failed"
            exit 1
        }
        cd "$PROJECT_ROOT"
    fi
    
    log "SUCCESS" "Test binaries built successfully"
}

# Run unit tests
run_unit_tests() {
    log "INFO" "Running unit tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests for bootloader
    if cargo test --release 2>&1 | tee -a "$LOG_FILE"; then
        log "SUCCESS" "Unit tests passed"
        return 0
    else
        log "ERROR" "Unit tests failed"
        return 1
    fi
}

# Create test kernel image
create_test_kernel() {
    local arch="$1"
    local output_file="$2"
    
    log "INFO" "Creating test kernel for $arch..."
    
    # Create a minimal test kernel binary
    case $arch in
        "x86_64")
            # Create minimal x86_64 kernel
            echo -e "\x48\x31\xc0\x48\x31\xdb\x48\x31\xc9\x48\x31\xd2\x48\x31\xf6\x48\x31\xff\x48\x31\xed\xeb\x01\x5b\x48\x89\xd8\x48\x31\xc0\x48\x31\xdb\x48\x31\xc9\x48\x31\xd2\x48\x31\xf6\x48\x31\xff\x48\x31\xed\xeb\x01\x5b\x48\x89\xd8\x48\x31\xc0\x48\x31\xdb\x48\x31\xc9\x48\x31\xd2\x48\x31\xf6\x48\x31\xff\x48\x31\xed\xeb\x01\x5b\x48\x89\xd8\x48\x31\xc0\x48\x31\xdb\x48\x31\xc9\x48\x31\xd2\x48\x31\xf6\x48\x31\xff\x48\x31\xed\xeb\x01\x5b\x48\x89\xd8\x48\x31\xc0\x48\x31\xdb\x48\x31\xc9\x48\x31\xd2\x48\x31\xf6\x48\x31\xff\x48\x31\xed\xeb\x01\x5b\x48\x89\xd8" > "$output_file"
            ;;
        "aarch64")
            # Create minimal ARM64 kernel
            echo -e "\xd4\x20\x40\x38\x00\x80\x82\x52\xe0\x03\x1f\xaa\xd4\x20\x80\x38\x08\x00\x00\x14" > "$output_file"
            ;;
        "riscv64")
            # Create minimal RISC-V kernel
            echo -e "\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97\x13\x01\x01\x97" > "$output_file"
            ;;
        *)
            log "ERROR" "Unsupported architecture: $arch"
            return 1
            ;;
    esac
    
    log "SUCCESS" "Test kernel created: $output_file"
}

# Run QEMU boot test
run_qemu_test() {
    local arch="$1"
    local boot_mode="$2"
    local test_name="${3:-${arch}_${boot_mode}}"
    local timeout="${4:-30}"
    
    log "INFO" "Running QEMU boot test: $test_name"
    
    local config="${ARCH_CONFIGS[$arch]}"
    IFS=':' read -r machine memory cpus <<< "$config"
    
    # Create temporary files
    local kernel_file="/tmp/test_kernel_$arch.bin"
    local console_file="/tmp/console_${test_name}.log"
    
    create_test_kernel "$arch" "$kernel_file" || return 1
    
    # Build QEMU command
    local qemu_cmd="qemu-system-${arch}"
    local qemu_args=(
        "-m" "$memory"
        "-smp" "$cpus"
        "-M" "$machine"
        "-nographic"
        "-kernel" "$kernel_file"
        "-append" "boot_mode=$boot_mode console=ttyS0"
        "-serial" "file:$console_file"
        "-monitor" "stdio"
    )
    
    # Architecture-specific adjustments
    case $arch in
        "aarch64")
            qemu_args+=("-cpu" "cortex-a57")
            qemu_args+=("-machine" "type=virt")
            qemu_args+=("-device" "virtio-blk-device,scsi=off")
            qemu_args+=("-device" "virtio-gpu-device")
            qemu_args+=("-device" "virtio-net-device,vectors=6")
            qemu_args+=("-device" "pl011,chardev=serial0")
            qemu_args+=("-chardev" "stdio,id=serial0")
            ;;
        "riscv64")
            qemu_args+=("-cpu" "rv64gc")
            qemu_args+=("-object" "memory-backend-ram,size=1G,id=mem")
            qemu_args+=("-machine" "type=virt,memory-backend=mem")
            qemu_args+=("-device" "virtio-blk-device,scsi=off")
            qemu_args+=("-device" "virtio-net-device,vectors=6")
            qemu_args+=("-device" "pl011,chardev=serial0")
            qemu_args+=("-chardev" "stdio,id=serial0")
            ;;
    esac
    
    # Run QEMU with timeout
    log "INFO" "Starting QEMU: ${qemu_args[*]}"
    
    if timeout "$timeout" "${qemu_cmd}" "${qemu_args[@]}" 2>&1 | tee -a "$LOG_FILE"; then
        # Check console output for success indicators
        if grep -q -i "boot\|multios\|success\|kernel" "$console_file" 2>/dev/null || \
           grep -q -E "Boot|UEFI|EFI|BIOS|Booting" "$console_file" 2>/dev/null; then
            log "SUCCESS" "QEMU test passed: $test_name"
            
            # Save console output
            local output_file="${OUTPUT_DIR}/console_${test_name}.log"
            cp "$console_file" "$output_file"
            
            # Cleanup
            rm -f "$console_file"
            
            return 0
        else
            log "WARNING" "QEMU test completed but no boot indicators found: $test_name"
            local output_file="${OUTPUT_DIR}/console_${test_name}.log"
            cp "$console_file" "$output_file"
            rm -f "$console_file"
            return 1
        fi
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log "WARNING" "QEMU test timed out: $test_name"
        else
            log "ERROR" "QEMU test failed with exit code $exit_code: $test_name"
        fi
        
        # Save console output even on failure
        if [ -f "$console_file" ]; then
            local output_file="${OUTPUT_DIR}/console_${test_name}.log"
            cp "$console_file" "$output_file"
            rm -f "$console_file"
        fi
        
        rm -f "$kernel_file"
        return 1
    fi
}

# Run memory test
run_memory_test() {
    local arch="$1"
    local test_name="${2:-${arch}_memory_test}"
    local memory_size="${3:-1G}"
    
    log "INFO" "Running memory test: $test_name with $memory_size"
    
    local config="virt:${memory_size}:2"
    IFS=':' read -r machine mem cpus <<< "$config"
    
    local kernel_file="/tmp/test_kernel_${arch}_memory.bin"
    local console_file="/tmp/console_${test_name}.log"
    
    create_test_kernel "$arch" "$kernel_file" || return 1
    
    local qemu_cmd="qemu-system-${arch}"
    local qemu_args=(
        "-m" "$memory_size"
        "-smp" "2"
        "-M" "$machine"
        "-nographic"
        "-kernel" "$kernel_file"
        "-append" "memory_test=true console=ttyS0"
        "-serial" "file:$console_file"
    )
    
    # Run memory test with longer timeout
    if timeout 60 "$qemu_cmd" "${qemu_args[@]}" 2>&1 | tee -a "$LOG_FILE"; then
        log "SUCCESS" "Memory test passed: $test_name"
        
        # Save console output
        local output_file="${OUTPUT_DIR}/console_${test_name}.log"
        cp "$console_file" "$output_file"
        
        rm -f "$console_file" "$kernel_file"
        return 0
    else
        log "ERROR" "Memory test failed: $test_name"
        
        # Save console output
        if [ -f "$console_file" ]; then
            local output_file="${OUTPUT_DIR}/console_${test_name}.log"
            cp "$console_file" "$output_file"
            rm -f "$console_file"
        fi
        
        rm -f "$kernel_file"
        return 1
    fi
}

# Run performance test
run_performance_test() {
    local arch="$1"
    local iterations="${2:-5}"
    local test_name="${3:-${arch}_performance_test}"
    
    log "INFO" "Running performance test: $test_name with $iterations iterations"
    
    local total_time=0
    local successful_runs=0
    
    for i in $(seq 1 "$iterations"); do
        log "INFO" "Performance test iteration $i/$iterations"
        
        local start_time=$(date +%s%N)
        
        if run_qemu_test "$arch" "uefi" "${test_name}_iter_${i}" "15"; then
            local end_time=$(date +%s%N)
            local iteration_time=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
            total_time=$((total_time + iteration_time))
            successful_runs=$((successful_runs + 1))
            
            log "INFO" "Iteration $i completed in ${iteration_time}ms"
        else
            log "WARNING" "Performance test iteration $i failed"
        fi
    done
    
    if [ $successful_runs -gt 0 ]; then
        local avg_time=$((total_time / successful_runs))
        log "SUCCESS" "Performance test completed: $test_name"
        log "INFO" "Successful runs: $successful_runs/$iterations"
        log "INFO" "Average boot time: ${avg_time}ms"
        
        # Save performance report
        local perf_file="${OUTPUT_DIR}/performance_${test_name}.txt"
        cat > "$perf_file" << EOF
Performance Test Report
=======================
Architecture: $arch
Total iterations: $iterations
Successful runs: $successful_runs
Average boot time: ${avg_time}ms
Total test time: $((total_time / 1000))s
EOF
        return 0
    else
        log "ERROR" "Performance test failed: no successful runs"
        return 1
    fi
}

# Run complete test suite
run_test_suite() {
    log "INFO" "Running complete bootloader test suite"
    
    local failed_tests=()
    local passed_tests=()
    
    # Test all architectures and boot modes
    for arch in "${!ARCH_CONFIGS[@]}"; do
        log "INFO" "Testing architecture: $arch"
        
        for boot_mode in "${BOOT_MODES[@]}"; do
            local test_name="${arch}_${boot_mode}_test"
            
            if run_qemu_test "$arch" "$boot_mode" "$test_name" "30"; then
                passed_tests+=("$test_name")
            else
                failed_tests+=("$test_name")
            fi
        done
        
        # Run memory test
        local memory_test_name="${arch}_memory_test"
        if run_memory_test "$arch" "$memory_test_name" "1G"; then
            passed_tests+=("$memory_test_name")
        else
            failed_tests+=("$memory_test_name")
        fi
        
        # Run performance test
        local perf_test_name="${arch}_performance_test"
        if run_performance_test "$arch" "3" "$perf_test_name"; then
            passed_tests+=("$perf_test_name")
        else
            failed_tests+=("$perf_test_name")
        fi
    done
    
    # Generate test report
    generate_test_report "${passed_tests[@]}" "${failed_tests[@]}"
    
    # Summary
    log "INFO" "Test suite completed"
    log "INFO" "Passed tests: ${#passed_tests[@]}"
    log "INFO" "Failed tests: ${#failed_tests[@]}"
    
    if [ ${#failed_tests[@]} -eq 0 ]; then
        log "SUCCESS" "All tests passed!"
        return 0
    else
        log "ERROR" "Some tests failed:"
        printf '%s\n' "${failed_tests[@]}" >&2
        return 1
    fi
}

# Generate test report
generate_test_report() {
    local passed_tests=("$@")
    local failed_tests=("${@:$(($# - ${#passed_tests[@]}))}")
    
    local report_file="${OUTPUT_DIR}/test_report.html"
    
    log "INFO" "Generating test report: $report_file"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>MultiOS Bootloader Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .passed { color: green; }
        .failed { color: red; }
        .summary { margin: 20px 0; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .status { font-weight: bold; }
    </style>
</head>
<body>
    <div class="header">
        <h1>MultiOS Bootloader Test Report</h1>
        <p>Generated on: $(date '+%Y-%m-%d %H:%M:%S')</p>
        <p>Log file: $LOG_FILE</p>
    </div>
    
    <div class="summary">
        <h2>Test Summary</h2>
        <div class="passed">Passed: ${#passed_tests[@]}</div>
        <div class="failed">Failed: ${#failed_tests[@]}</div>
        <div>Total: $((${#passed_tests[@]} + ${#failed_tests[@]}))</div>
    </div>
    
    <h2>Passed Tests</h2>
    <table>
        <tr><th>Test Name</th><th>Status</th></tr>
EOF
    
    for test in "${passed_tests[@]}"; do
        echo "        <tr><td>$test</td><td class="passed">PASSED</td></tr>" >> "$report_file"
    done
    
    echo "    </table>" >> "$report_file"
    
    if [ ${#failed_tests[@]} -gt 0 ]; then
        echo "    <h2>Failed Tests</h2>" >> "$report_file"
        echo "    <table>" >> "$report_file"
        echo "        <tr><th>Test Name</th><th>Status</th></tr>" >> "$report_file"
        
        for test in "${failed_tests[@]}"; do
            echo "        <tr><td>$test</td><td class="failed">FAILED</td></tr>" >> "$report_file"
        done
        
        echo "    </table>" >> "$report_file"
    fi
    
    echo "</body></html>" >> "$report_file"
    
    log "SUCCESS" "Test report generated: $report_file"
}

# Cleanup function
cleanup() {
    log "INFO" "Cleaning up test files..."
    
    # Remove temporary files
    rm -f /tmp/test_kernel_*.bin
    rm -f /tmp/console_*.log
    rm -f /tmp/qemu_*.pid
    
    # Create archive of results if requested
    if [ "${ARCHIVE_RESULTS:-}" = "true" ]; then
        local archive_file="${OUTPUT_DIR}/bootloader_tests_$(date +%Y%m%d_%H%M%S).tar.gz"
        log "INFO" "Creating results archive: $archive_file"
        tar -czf "$archive_file" -C "$OUTPUT_DIR" .
    fi
}

# Main function
main() {
    # Setup
    mkdir -p "$OUTPUT_DIR"
    
    log "INFO" "Starting MultiOS Bootloader Testing"
    log "INFO" "Project root: $PROJECT_ROOT"
    log "INFO" "Output directory: $OUTPUT_DIR"
    log "INFO" "Log file: $LOG_FILE"
    
    # Set up cleanup trap
    trap cleanup EXIT
    
    # Parse command line arguments
    local run_unit=false
    local run_qemu=false
    local run_memory=false
    local run_performance=false
    local run_suite=false
    local architectures=("${!ARCH_CONFIGS[@]}")
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit)
                run_unit=true
                shift
                ;;
            --qemu)
                run_qemu=true
                shift
                ;;
            --memory)
                run_memory=true
                shift
                ;;
            --performance)
                run_performance=true
                shift
                ;;
            --all)
                run_unit=true
                run_qemu=true
                run_memory=true
                run_performance=true
                run_suite=true
                shift
                ;;
            --arch)
                architectures=("$2")
                shift 2
                ;;
            --memory-size)
                MEMORY_SIZE="$2"
                shift 2
                ;;
            --iterations)
                ITERATIONS="$2"
                shift 2
                ;;
            --archive)
                ARCHIVE_RESULTS="true"
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log "ERROR" "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    # Check dependencies and build
    check_dependencies
    build_test_binaries
    
    local exit_code=0
    
    # Run requested tests
    if [ "$run_unit" = true ]; then
        if ! run_unit_tests; then
            exit_code=1
        fi
    fi
    
    if [ "$run_qemu" = true ]; then
        for arch in "${architectures[@]}"; do
            for boot_mode in "${BOOT_MODES[@]}"; do
                if ! run_qemu_test "$arch" "$boot_mode"; then
                    exit_code=1
                fi
            done
        done
    fi
    
    if [ "$run_memory" = true ]; then
        for arch in "${architectures[@]}"; do
            if ! run_memory_test "$arch"; then
                exit_code=1
            fi
        done
    fi
    
    if [ "$run_performance" = true ]; then
        for arch in "${architectures[@]}"; do
            if ! run_performance_test "$arch" "${ITERATIONS:-5}"; then
                exit_code=1
            fi
        done
    fi
    
    if [ "$run_suite" = true ]; then
        if ! run_test_suite; then
            exit_code=1
        fi
    fi
    
    if [ $exit_code -eq 0 ]; then
        log "SUCCESS" "All requested tests completed successfully"
    else
        log "ERROR" "Some tests failed. Check the log file for details."
    fi
    
    return $exit_code
}

# Show help message
show_help() {
    cat << EOF
MultiOS Bootloader Testing Script

Usage: $0 [OPTIONS]

Options:
    --unit              Run unit tests
    --qemu              Run QEMU integration tests
    --memory            Run memory management tests
    --performance       Run performance tests
    --all               Run all tests (default)
    --arch ARCH         Target architecture (x86_64, aarch64, riscv64)
    --memory-size SIZE  Memory size for tests (e.g., 512M, 1G)
    --iterations NUM    Number of iterations for performance tests
    --archive           Create archive of test results
    --help              Show this help message

Examples:
    $0 --all                           # Run all tests
    $0 --unit                          # Run only unit tests
    $0 --qemu --arch x86_64            # Run QEMU tests for x86_64
    $0 --performance --iterations 10   # Run performance tests with 10 iterations
    $0 --memory --memory-size 2G       # Run memory tests with 2GB

Report bugs to: https://github.com/multios/bootloader/issues
EOF
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
