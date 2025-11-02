#!/bin/bash

# MultiOS Integration Test Runner
# Runs comprehensive integration tests across all architectures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ARCHITECTURES=("x86_64" "arm64" "riscv64")
QEMU_CONFIGS_DIR="/workspace/qemu/configs"
TEST_DATA_DIR="/workspace/test_data"
REPORTS_DIR="/workspace/reports"
LOG_DIR="/workspace/logs"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Ensure directories exist
mkdir -p "$REPORTS_DIR" "$LOG_DIR" "$TEST_DATA_DIR"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Get QEMU configuration for architecture
get_qemu_config() {
    local arch=$1
    local config_file="$QEMU_CONFIGS_DIR/${arch}.conf"
    
    if [ -f "$config_file" ]; then
        cat "$config_file"
    else
        # Default QEMU configuration
        case "$arch" in
            "x86_64")
                echo "ACCEL=kvm MACHINE_TYPE=pc-q35-6.2 SMP=4 MEMORY=4G KERNEL=target/x86_64-unknown-none/release/multios APPEND=\"console=ttyS0 loglevel=8\" SERIAL=\"-serial stdio\" MONITOR=\"-monitor telnet:127.0.0.1:4444,server,nowait\""
                ;;
            "arm64")
                echo "ACCEL=kvm MACHINE_TYPE=virt SMP=4 MEMORY=4G KERNEL=target/aarch64-unknown-none/release/multios APPEND=\"console=ttyAMA0 loglevel=8\" SERIAL=\"-serial stdio\" MONITOR=\"-monitor telnet:127.0.0.1:4445,server,nowait\""
                ;;
            "riscv64")
                echo "ACCEL=kvm MACHINE_TYPE=spike_virt SMP=4 MEMORY=4G KERNEL=target/riscv64gc-unknown-none-elf/release/multios APPEND=\"console=ttyS0 loglevel=8\" SERIAL=\"-serial stdio\" MONITOR=\"-monitor telnet:127.0.0.1:4446,server,nowait\""
                ;;
        esac
    fi
}

# Parse QEMU configuration
parse_qemu_config() {
    local config_line=$1
    
    export ACCEL=$(echo "$config_line" | grep -o 'ACCEL=[^ ]*' | cut -d= -f2)
    export MACHINE_TYPE=$(echo "$config_line" | grep -o 'MACHINE_TYPE=[^ ]*' | cut -d= -f2)
    export SMP=$(echo "$config_line" | grep -o 'SMP=[0-9]*' | cut -d= -f2)
    export MEMORY=$(echo "$config_line" | grep -o 'MEMORY=[^ ]*' | cut -d= -f2)
    export KERNEL=$(echo "$config_line" | grep -o 'KERNEL=[^ ]*' | cut -d= -f2)
    export APPEND=$(echo "$config_line" | grep -o 'APPEND="[^"]*"' | cut -d= -f2 | sed 's/^"//;s/"$//')
    export SERIAL=$(echo "$config_line" | grep -o 'SERIAL=[^ ]*' | cut -d= -f2)
    export MONITOR=$(echo "$config_line" | grep -o 'MONITOR="[^"]*"' | cut -d= -f2 | sed 's/^"//;s/"$//')
}

# Start QEMU instance
start_qemu() {
    local arch=$1
    local log_file="$LOG_DIR/qemu_${arch}_${TIMESTAMP}.log"
    local monitor_port=$(( 4444 + $(echo "$arch" | sed 's/[^0-9]//g' | grep -o '[0-9]*' | head -1) ))
    
    log "Starting QEMU for $arch..."
    
    # Parse QEMU configuration
    local config=$(get_qemu_config "$arch")
    parse_qemu_config "$config"
    
    # Build QEMU command
    local qemu_cmd=""
    case "$arch" in
        "x86_64")
            qemu_cmd="qemu-system-x86_64 -m ${MEMORY:-4G} -smp ${SMP:-4} -machine $MACHINE_TYPE"
            if [ -n "$KERNEL" ] && [ -f "$KERNEL" ]; then
                qemu_cmd="$qemu_cmd -kernel $KERNEL"
            fi
            ;;
        "arm64")
            qemu_cmd="qemu-system-aarch64 -m ${MEMORY:-4G} -smp ${SMP:-4} -machine $MACHINE_TYPE -cpu cortex-a57"
            if [ -n "$KERNEL" ] && [ -f "$KERNEL" ]; then
                qemu_cmd="$qemu_cmd -kernel $KERNEL"
            fi
            ;;
        "riscv64")
            qemu_cmd="qemu-system-riscv64 -m ${MEMORY:-4G} -smp ${SMP:-4} -machine $MACHINE_TYPE"
            if [ -n "$KERNEL" ] && [ -f "$KERNEL" ]; then
                qemu_cmd="$qemu_cmd -kernel $KERNEL"
            fi
            ;;
    esac
    
    # Add standard options
    qemu_cmd="$qemu_cmd -nographic -enable-kvm"
    
    if [ -n "$APPEND" ]; then
        qemu_cmd="$qemu_cmd -append \"$APPEND\""
    fi
    
    if [ -n "$SERIAL" ]; then
        qemu_cmd="$qemu_cmd $SERIAL"
    fi
    
    if [ -n "$MONITOR" ]; then
        qemu_cmd="$qemu_cmd $MONITOR"
    fi
    
    qemu_cmd="$qemu_cmd > \"$log_file\" 2>&1 &"
    
    # Start QEMU
    eval "$qemu_cmd"
    local qemu_pid=$!
    
    # Wait for QEMU to start
    sleep 5
    
    # Check if QEMU is still running
    if kill -0 "$qemu_pid" 2>/dev/null; then
        success "QEMU started for $arch (PID: $qemu_pid)"
        echo "$qemu_pid" > "/tmp/qemu_${arch}_pid.txt"
        return 0
    else
        error "QEMU failed to start for $arch"
        return 1
    fi
}

# Stop QEMU instance
stop_qemu() {
    local arch=$1
    
    if [ -f "/tmp/qemu_${arch}_pid.txt" ]; then
        local qemu_pid=$(cat "/tmp/qemu_${arch}_pid.txt")
        if kill -0 "$qemu_pid" 2>/dev/null; then
            log "Stopping QEMU for $arch (PID: $qemu_pid)"
            kill "$qemu_pid"
            sleep 2
            if kill -0 "$qemu_pid" 2>/dev/null; then
                kill -9 "$qemu_pid"
            fi
        fi
        rm -f "/tmp/qemu_${arch}_pid.txt"
    fi
}

# Run basic system tests
run_basic_tests() {
    local arch=$1
    local test_log="$LOG_DIR/basic_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running basic system tests for $arch..."
    
    # Test basic system operations
    local tests_passed=0
    local tests_total=5
    
    # Test 1: System boot
    log "Test 1: System boot verification"
    if timeout 30 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=boot" -serial stdio 2>/dev/null | grep -q "boot_success"; then
        ((tests_passed++))
        success "Boot test passed"
    else
        error "Boot test failed"
    fi
    
    # Test 2: Memory management
    log "Test 2: Memory management"
    if timeout 30 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=memory" -serial stdio 2>/dev/null | grep -q "memory_ok"; then
        ((tests_passed++))
        success "Memory test passed"
    else
        error "Memory test failed"
    fi
    
    # Test 3: Process creation
    log "Test 3: Process creation"
    if timeout 30 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=process" -serial stdio 2>/dev/null | grep -q "process_ok"; then
        ((tests_passed++))
        success "Process test passed"
    else
        error "Process test failed"
    fi
    
    # Test 4: File operations
    log "Test 4: File operations"
    if timeout 30 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=filesystem" -serial stdio 2>/dev/null | grep -q "filesystem_ok"; then
        ((tests_passed++))
        success "Filesystem test passed"
    else
        error "Filesystem test failed"
    fi
    
    # Test 5: Interrupts
    log "Test 5: Interrupt handling"
    if timeout 30 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=interrupts" -serial stdio 2>/dev/null | grep -q "interrupts_ok"; then
        ((tests_passed++))
        success "Interrupt test passed"
    else
        error "Interrupt test failed"
    fi
    
    # Save test results
    echo "$tests_passed/$tests_total basic tests passed" >> "$test_log"
    
    return $([ "$tests_passed" -eq "$tests_total" ] && echo 0 || echo 1)
}

# Run network integration tests
run_network_tests() {
    local arch=$1
    local test_log="$LOG_DIR/network_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running network integration tests for $arch..."
    
    # Network tests require special setup
    local tests_passed=0
    local tests_total=3
    
    # Test 1: Network initialization
    log "Test 1: Network stack initialization"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=network_init" -serial stdio -netdev user,id=net0 2>/dev/null | grep -q "network_init_ok"; then
        ((tests_passed++))
        success "Network init test passed"
    else
        error "Network init test failed"
    fi
    
    # Test 2: TCP communication
    log "Test 2: TCP communication"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=tcp" -serial stdio -netdev user,id=net0 2>/dev/null | grep -q "tcp_ok"; then
        ((tests_passed++))
        success "TCP test passed"
    else
        error "TCP test failed"
    fi
    
    # Test 3: UDP communication
    log "Test 3: UDP communication"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=udp" -serial stdio -netdev user,id=net0 2>/dev/null | grep -q "udp_ok"; then
        ((tests_passed++))
        success "UDP test passed"
    else
        error "UDP test failed"
    fi
    
    # Save test results
    echo "$tests_passed/$tests_total network tests passed" >> "$test_log"
    
    return $([ "$tests_passed" -eq "$tests_total" ] && echo 0 || echo 1)
}

# Run driver integration tests
run_driver_tests() {
    local arch=$1
    local test_log="$LOG_DIR/driver_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running driver integration tests for $arch..."
    
    local tests_passed=0
    local tests_total=4
    
    # Test 1: Storage driver
    log "Test 1: Storage driver"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=storage_driver" -serial stdio -drive file=/dev/null,format=raw,if=virtio 2>/dev/null | grep -q "storage_driver_ok"; then
        ((tests_passed++))
        success "Storage driver test passed"
    else
        error "Storage driver test failed"
    fi
    
    # Test 2: Network driver
    log "Test 2: Network driver"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=network_driver" -serial stdio -netdev user,id=net0 -device virtio-net,netdev=net0 2>/dev/null | grep -q "network_driver_ok"; then
        ((tests_passed++))
        success "Network driver test passed"
    else
        error "Network driver test failed"
    fi
    
    # Test 3: Display driver
    log "Test 3: Display driver"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=display_driver" -serial stdio -vga std 2>/dev/null | grep -q "display_driver_ok"; then
        ((tests_passed++))
        success "Display driver test passed"
    else
        error "Display driver test failed"
    fi
    
    # Test 4: Input driver
    log "Test 4: Input driver"
    if timeout 60 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=input_driver" -serial stdio -device qemu-xhci 2>/dev/null | grep -q "input_driver_ok"; then
        ((tests_passed++))
        success "Input driver test passed"
    else
        error "Input driver test failed"
    fi
    
    # Save test results
    echo "$tests_passed/$tests_total driver tests passed" >> "$test_log"
    
    return $([ "$tests_passed" -eq "$tests_total" ] && echo 0 || echo 1)
}

# Run stress tests
run_stress_tests() {
    local arch=$1
    local test_log="$LOG_DIR/stress_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running stress tests for $arch..."
    
    local tests_passed=0
    local tests_total=3
    
    # Test 1: Memory stress
    log "Test 1: Memory stress test"
    if timeout 120 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=memory_stress" -serial stdio -m 1G 2>/dev/null | grep -q "memory_stress_ok"; then
        ((tests_passed++))
        success "Memory stress test passed"
    else
        error "Memory stress test failed"
    fi
    
    # Test 2: CPU stress
    log "Test 2: CPU stress test"
    if timeout 120 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=cpu_stress" -serial stdio -smp 2 2>/dev/null | grep -q "cpu_stress_ok"; then
        ((tests_passed++))
        success "CPU stress test passed"
    else
        error "CPU stress test failed"
    fi
    
    # Test 3: I/O stress
    log "Test 3: I/O stress test"
    if timeout 180 qemu-system-${arch} -kernel "${KERNEL:-}" -append "test=io_stress" -serial stdio -drive file=/tmp/test_disk.img,format=raw,if=virtio 2>/dev/null | grep -q "io_stress_ok"; then
        ((tests_passed++))
        success "I/O stress test passed"
    else
        error "I/O stress test failed"
    fi
    
    # Save test results
    echo "$tests_passed/$tests_total stress tests passed" >> "$test_log"
    
    return $([ "$tests_passed" -eq "$tests_total" ] && echo 0 || echo 1)
}

# Generate integration test report
generate_integration_report() {
    local arch=$1
    local report_file="$REPORTS_DIR/integration_tests_${arch}_${TIMESTAMP}.xml"
    
    log "Generating integration test report for $arch..."
    
    # Parse test results from log files
    local basic_result=$(grep -o "[0-9]*/[0-9]* basic tests passed" "$LOG_DIR/basic_tests_${arch}_${TIMESTAMP}.log" 2>/dev/null || echo "0/0")
    local network_result=$(grep -o "[0-9]*/[0-9]* network tests passed" "$LOG_DIR/network_tests_${arch}_${TIMESTAMP}.log" 2>/dev/null || echo "0/0")
    local driver_result=$(grep -o "[0-9]*/[0-9]* driver tests passed" "$LOG_DIR/driver_tests_${arch}_${TIMESTAMP}.log" 2>/dev/null || echo "0/0")
    local stress_result=$(grep -o "[0-9]*/[0-9]* stress tests passed" "$LOG_DIR/stress_tests_${arch}_${TIMESTAMP}.log" 2>/dev/null || echo "0/0")
    
    # Calculate totals
    local total_passed=$(echo "$basic_result $network_result $driver_result $stress_result" | awk '{split($1,a,"/"); split($3,b,"/"); split($5,c,"/"); split($7,d,"/"); print a[1]+b[1]+c[1]+d[1]}')
    local total_tests=$(echo "$basic_result $network_result $driver_result $stress_result" | awk '{split($1,a,"/"); split($3,b,"/"); split($5,c,"/"); split($7,d,"/"); print a[2]+b[2]+c[2]+d[2]}')
    
    # Generate JUnit XML report
    cat > "$report_file" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="MultiOS Integration Tests" tests="$total_tests" failures="$(( total_tests - total_passed ))" time="0">
    <testsuite name="$arch Integration Tests" tests="$total_tests" failures="$(( total_tests - total_passed ))" time="0">
        <testcase name="Basic System Tests" classname="$arch.Basic" time="0">
            <system-out>$basic_result passed</system-out>
        </testcase>
        <testcase name="Network Integration Tests" classname="$arch.Network" time="0">
            <system-out>$network_result passed</system-out>
        </testcase>
        <testcase name="Driver Integration Tests" classname="$arch.Drivers" time="0">
            <system-out>$driver_result passed</system-out>
        </testcase>
        <testcase name="Stress Tests" classname="$arch.Stress" time="0">
            <system-out>$stress_result passed</system-out>
        </testcase>
    </testsuite>
</testsuites>
EOF
    
    success "Integration test report generated: $report_file"
}

# Main integration test execution
main() {
    local arch=${1:-}
    
    log "Starting MultiOS Integration Test Suite"
    
    if [ -z "$arch" ]; then
        # Test all architectures
        log "Testing all architectures"
        
        local failed_archs=()
        
        for arch in "${ARCHITECTURES[@]}"; do
            log "=== Integration Testing $arch ==="
            
            # Start QEMU
            if ! start_qemu "$arch"; then
                error "Failed to start QEMU for $arch"
                failed_archs+=("$arch")
                continue
            fi
            
            # Run integration tests
            local arch_failed=false
            
            if ! run_basic_tests "$arch"; then
                arch_failed=true
            fi
            
            if ! run_network_tests "$arch"; then
                arch_failed=true
            fi
            
            if ! run_driver_tests "$arch"; then
                arch_failed=true
            fi
            
            if ! run_stress_tests "$arch"; then
                arch_failed=true
            fi
            
            # Stop QEMU
            stop_qemu "$arch"
            
            # Generate report
            generate_integration_report "$arch"
            
            if [ "$arch_failed" = true ]; then
                error "Integration tests failed for $arch"
                failed_archs+=("$arch")
            else
                success "Integration tests passed for $arch"
            fi
        done
        
        # Overall status
        if [ ${#failed_archs[@]} -eq 0 ]; then
            success "All integration tests passed!"
            exit 0
        else
            error "Integration tests failed for: ${failed_archs[*]}"
            exit 1
        fi
    else
        # Test specific architecture
        if [[ " ${ARCHITECTURES[@]} " =~ " ${arch} " ]]; then
            log "Testing specific architecture: $arch"
            
            # Start QEMU
            if start_qemu "$arch"; then
                # Run all tests
                run_basic_tests "$arch" || true
                run_network_tests "$arch" || true
                run_driver_tests "$arch" || true
                run_stress_tests "$arch" || true
                
                # Stop QEMU
                stop_qemu "$arch"
                
                # Generate report
                generate_integration_report "$arch"
            else
                error "Failed to start QEMU for $arch"
                exit 1
            fi
        else
            error "Unknown architecture: $arch"
            echo "Supported architectures: ${ARCHITECTURES[*]}"
            exit 1
        fi
    fi
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [ARCHITECTURE]

ARCHITECTURE: x86_64, arm64, riscv64, or empty for all architectures

This script runs comprehensive integration tests across all architectures
using QEMU emulation.

Examples:
  $0                 # Test all architectures
  $0 x86_64          # Test only x86_64
  $0 arm64           # Test only ARM64
  $0 riscv64         # Test only RISC-V64
EOF
}

# Parse command line arguments
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    usage
    exit 0
fi

main "$@"