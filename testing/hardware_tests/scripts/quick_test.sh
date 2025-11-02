#!/usr/bin/env bash
# Hardware Testing Framework - Quick Test Scripts
# Automated testing scripts for different hardware configurations

set -e  # Exit on error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root for certain tests
check_root() {
    if [[ $EUID -eq 0 ]]; then
        log "Running as root - full test suite available"
        ROOT_AVAILABLE=true
    else
        warning "Not running as root - some tests may be skipped"
        ROOT_AVAILABLE=false
    fi
}

# Detect hardware capabilities
detect_hardware() {
    log "Detecting hardware capabilities..."
    
    # CPU detection
    if command -v lscpu >/dev/null 2>&1; then
        CPU_CORES=$(lscpu | grep "CPU(s):" | head -1 | awk '{print $2}')
        CPU_MODEL=$(lscpu | grep "Model name:" | sed 's/Model name:\s*//')
        success "CPU: $CPU_MODEL ($CPU_CORES cores)"
    fi
    
    # Memory detection
    if [ -f /proc/meminfo ]; then
        TOTAL_MEM=$(grep MemTotal /proc/meminfo | awk '{print $2}')
        TOTAL_MEM_GB=$((TOTAL_MEM / 1024 / 1024))
        success "Memory: ${TOTAL_MEM_GB}GB"
    fi
    
    # Storage detection
    if command -v lsblk >/dev/null 2>&1; then
        STORAGE_COUNT=$(lsblk -d -n -o NAME | wc -l)
        success "Storage devices: $STORAGE_COUNT"
    fi
    
    # GPU detection
    if command -v nvidia-smi >/dev/null 2>&1; then
        GPU_MODEL=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
        success "GPU: $GPU_MODEL"
    elif command -v lspci >/dev/null 2>&1; then
        GPU_INFO=$(lspci | grep -i vga)
        if [ -n "$GPU_INFO" ]; then
            success "GPU: $GPU_INFO"
        fi
    fi
}

# Run hardware detection test
run_hardware_detection() {
    log "Running hardware detection test..."
    
    if [ -f "/workspace/testing/hardware_tests/hardware_detector.py" ]; then
        python3 /workspace/testing/hardware_tests/hardware_detector.py --profile --verbose
        success "Hardware detection completed"
    else
        warning "Hardware detector not found, running basic detection"
        detect_hardware
    fi
}

# Run compatibility tests
run_compatibility_tests() {
    log "Running hardware compatibility tests..."
    
    if [ -f "/workspace/testing/hardware_tests/compatibility_testing.py" ]; then
        python3 /workspace/testing/hardware_tests/compatibility_testing.py --verbose
        success "Compatibility testing completed"
    else
        warning "Compatibility tester not found"
    fi
}

# Run peripheral tests
run_peripheral_tests() {
    log "Running peripheral tests..."
    
    if [ -f "/workspace/testing/hardware_tests/peripheral_testing.py" ]; then
        python3 /workspace/testing/hardware_tests/peripheral_testing.py --verbose
        success "Peripheral testing completed"
    else
        warning "Peripheral tester not found"
    fi
}

# Run multi-core scaling test
run_multicore_test() {
    log "Running multi-core scaling test..."
    
    if [ -f "/workspace/testing/hardware_tests/multicore_scaling_test.py" ]; then
        python3 /workspace/testing/hardware_tests/multicore_scaling_test.py --test scaling --verbose
        success "Multi-core scaling test completed"
    else
        warning "Multi-core tester not found"
    fi
}

# Run quick power test
run_quick_power_test() {
    log "Running quick power test (5 minutes)..."
    
    if [ -f "/workspace/testing/hardware_tests/power_thermal_testing.py" ]; then
        python3 /workspace/testing/hardware_tests/power_thermal_testing.py --test stress --duration 5 --verbose
        success "Power testing completed"
    else
        warning "Power tester not found"
    fi
}

# Run optimization analysis
run_optimization_analysis() {
    log "Running optimization analysis..."
    
    if [ -f "/workspace/testing/hardware_tests/optimization_engine.py" ]; then
        python3 /workspace/testing/hardware_tests/optimization_engine.py --analyze --recommend --verbose
        success "Optimization analysis completed"
    else
        warning "Optimization engine not found"
    fi
}

# Quick stress test
run_quick_stress_test() {
    log "Running quick stress test (2 minutes)..."
    
    # Simple CPU stress
    stress_start_time=$(date +%s)
    log "Starting CPU stress test..."
    
    # CPU stress using stress if available, otherwise use simple calculations
    if command -v stress >/dev/null 2>&1; then
        stress --cpu 4 --timeout 120s &
        STRESS_PID=$!
    else
        log "stress command not found, using built-in CPU test"
        # Simple CPU test
        for i in {1..4}; do
            {
                end_time=$(($(date +%s) + 120))
                while [ $(date +%s) -lt $end_time ]; do
                    echo "scale=5000; 4*a(1)" | bc -l >/dev/null 2>&1
                done
            } &
        done
    fi
    
    # Monitor during stress
    monitor_duration=120
    for i in $(seq 1 $monitor_duration); do
        if command -v sensors >/dev/null 2>&1; then
            CPU_TEMP=$(sensors | grep "Core 0" | awk '{print $3}' | head -1)
            log "Minute $i: CPU Temp: $CPU_TEMP"
        else
            log "Minute $i: Stress test running..."
        fi
        sleep 60
    done
    
    # Cleanup
    if [ -n "$STRESS_PID" ]; then
        kill $STRESS_PID 2>/dev/null || true
    fi
    pkill -f "scale=5000" 2>/dev/null || true
    
    success "Quick stress test completed"
}

# Network performance test
run_network_test() {
    log "Running network performance test..."
    
    # Simple network test
    if ping -c 1 8.8.8.8 >/dev/null 2>&1; then
        log "Testing network connectivity..."
        
        # Download speed test (simple)
        log "Measuring network performance..."
        
        if command -v iperf3 >/dev/null 2>&1; then
            # Try to run iperf3 to localhost
            if iperf3 -c 127.0.0.1 -t 5 >/dev/null 2>&1; then
                log "Local network loopback test available"
            fi
        fi
        
        success "Network connectivity confirmed"
    else
        error "Network connectivity test failed"
    fi
}

# Storage performance test
run_storage_test() {
    log "Running storage performance test..."
    
    # Create test file
    TEST_FILE="/tmp/hardware_test_$$.tmp"
    TEST_SIZE=100  # MB
    
    log "Testing sequential write performance..."
    START_TIME=$(date +%s.%N)
    dd if=/dev/zero of="$TEST_FILE" bs=1M count=$TEST_SIZE 2>/dev/null
    WRITE_TIME=$(echo "$(date +%s.%N) - $START_TIME" | bc)
    WRITE_SPEED=$(echo "scale=2; $TEST_SIZE / $WRITE_TIME" | bc)
    
    log "Testing sequential read performance..."
    START_TIME=$(date +%s.%N)
    dd if="$TEST_FILE" of=/dev/null bs=1M 2>/dev/null
    READ_TIME=$(echo "$(date +%s.%N) - $START_TIME" | bc)
    READ_SPEED=$(echo "scale=2; $TEST_SIZE / $READ_TIME" | bc)
    
    # Cleanup
    rm -f "$TEST_FILE"
    
    success "Storage test completed: Write ${WRITE_SPEED}MB/s, Read ${READ_SPEED}MB/s"
}

# Temperature monitoring
monitor_temperature() {
    log "Monitoring system temperature for 5 minutes..."
    
    for i in {1..5}; do
        if command -v sensors >/dev/null 2>&1; then
            TEMP_OUTPUT=$(sensors | grep -E "(Core|Package|Tdie|Tctl)" | head -5)
            log "Reading $i/5:\n$TEMP_OUTPUT"
        else
            warning "sensors not available, skipping temperature monitoring"
            break
        fi
        sleep 60
    done
    
    success "Temperature monitoring completed"
}

# Main menu
show_menu() {
    echo -e "\n${BLUE}Hardware Testing Framework - Quick Tests${NC}"
    echo "============================================="
    echo "1)  Quick hardware detection"
    echo "2)  Full hardware detection + compatibility"
    echo "3)  Peripheral testing"
    echo "4)  Multi-core scaling test"
    echo "5)  Quick power/thermal test (5 min)"
    echo "6)  Quick stress test (2 min)"
    echo "7)  Network performance test"
    echo "8)  Storage performance test"
    echo "9)  Temperature monitoring (5 min)"
    echo "10) Optimization analysis"
    echo "11) Run all tests (comprehensive)"
    echo "12) Exit"
    echo
    read -p "Select option [1-12]: " choice
}

# Main execution
main() {
    log "Starting Hardware Testing Framework"
    check_root
    detect_hardware
    
    while true; do
        show_menu
        
        case $choice in
            1)
                run_hardware_detection
                ;;
            2)
                run_hardware_detection
                echo
                run_compatibility_tests
                ;;
            3)
                run_peripheral_tests
                ;;
            4)
                run_multicore_test
                ;;
            5)
                run_quick_power_test
                ;;
            6)
                run_quick_stress_test
                ;;
            7)
                run_network_test
                ;;
            8)
                run_storage_test
                ;;
            9)
                monitor_temperature
                ;;
            10)
                run_optimization_analysis
                ;;
            11)
                log "Running comprehensive test suite..."
                run_hardware_detection
                echo
                run_compatibility_tests
                echo
                run_peripheral_tests
                echo
                run_multicore_test
                echo
                run_quick_power_test
                echo
                run_network_test
                echo
                run_storage_test
                echo
                run_optimization_analysis
                success "Comprehensive test suite completed!"
                ;;
            12)
                log "Exiting Hardware Testing Framework"
                exit 0
                ;;
            *)
                error "Invalid option. Please select 1-12."
                ;;
        esac
        
        echo
        read -p "Press Enter to continue..."
    done
}

# Run main function if script is executed directly
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi