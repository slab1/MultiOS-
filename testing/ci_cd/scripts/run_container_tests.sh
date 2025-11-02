#!/bin/bash

# MultiOS Container Test Runner
# This script runs comprehensive tests in containerized environments

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_LOG_DIR="/workspace/logs"
TEST_REPORT_DIR="/workspace/reports"
ARCHITECTURES=("x86_64" "arm64" "riscv64")
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create directories
mkdir -p "$TEST_LOG_DIR" "$TEST_REPORT_DIR"

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

# Function to run unit tests
run_unit_tests() {
    local arch=$1
    local log_file="$TEST_LOG_DIR/unit_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running unit tests for $arch..."
    
    cargo test --target $arch-unknown-none --all-features --lib 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Unit tests passed for $arch"
        echo "UNIT_TESTS_PASSED=true" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
    else
        error "Unit tests failed for $arch"
        echo "UNIT_TESTS_PASSED=false" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
        return 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    local arch=$1
    local log_file="$TEST_LOG_DIR/integration_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running integration tests for $arch..."
    
    # Run integration tests with QEMU
    ./scripts/run_qemu_tests.sh "$arch" 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Integration tests passed for $arch"
        echo "INTEGRATION_TESTS_PASSED=true" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
    else
        error "Integration tests failed for $arch"
        echo "INTEGRATION_TESTS_PASSED=false" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
        return 1
    fi
}

# Function to run performance benchmarks
run_performance_tests() {
    local arch=$1
    local log_file="$TEST_LOG_DIR/performance_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running performance benchmarks for $arch..."
    
    cargo bench --target $arch-unknown-none --all-features 2>&1 | tee "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Performance tests passed for $arch"
        echo "PERFORMANCE_TESTS_PASSED=true" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
    else
        error "Performance tests failed for $arch"
        echo "PERFORMANCE_TESTS_PASSED=false" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
        return 1
    fi
}

# Function to validate cross-compilation
validate_cross_compilation() {
    local arch=$1
    local log_file="$TEST_LOG_DIR/cross_compile_${arch}_${TIMESTAMP}.log"
    
    log "Validating cross-compilation for $arch..."
    
    # Check if binary can be created
    if [ "$arch" != "x86_64" ]; then
        cross build --target $arch-unknown-none --release 2>&1 | tee "$log_file"
    else
        cargo build --target $arch-unknown-none --release 2>&1 | tee "$log_file"
    fi
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Cross-compilation validation passed for $arch"
        echo "CROSS_COMPILE_PASSED=true" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
    else
        error "Cross-compilation validation failed for $arch"
        echo "CROSS_COMPILE_PASSED=false" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
        return 1
    fi
}

# Function to run security tests
run_security_tests() {
    local arch=$1
    local log_file="$TEST_LOG_DIR/security_tests_${arch}_${TIMESTAMP}.log"
    
    log "Running security tests for $arch..."
    
    # Run cargo audit
    cargo audit 2>&1 | tee "$log_file"
    
    # Run clippy
    cargo clippy --target $arch-unknown-none --all-features -- -D warnings 2>&1 | tee -a "$log_file"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Security tests passed for $arch"
        echo "SECURITY_TESTS_PASSED=true" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
    else
        error "Security tests failed for $arch"
        echo "SECURITY_TESTS_PASSED=false" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
        return 1
    fi
}

# Function to generate test report
generate_test_report() {
    local arch=$1
    local report_file="$TEST_REPORT_DIR/test_report_${arch}_${TIMESTAMP}.json"
    
    # Parse test results
    local unit_passed=false
    local integration_passed=false
    local performance_passed=false
    local cross_compile_passed=false
    local security_passed=false
    
    if [ -f "$TEST_REPORT_DIR/test_status_${arch}.txt" ]; then
        source "$TEST_REPORT_DIR/test_status_${arch}.txt"
    fi
    
    # Generate JSON report
    cat > "$report_file" << EOF
{
    "architecture": "$arch",
    "timestamp": "$TIMESTAMP",
    "tests": {
        "unit_tests": {
            "status": $([ "$unit_passed" = true ] && echo "true" || echo "false"),
            "passed": $unit_passed
        },
        "integration_tests": {
            "status": $([ "$integration_passed" = true ] && echo "true" || echo "false"),
            "passed": $integration_passed
        },
        "performance_tests": {
            "status": $([ "$performance_passed" = true ] && echo "true" || echo "false"),
            "passed": $performance_passed
        },
        "cross_compilation": {
            "status": $([ "$cross_compile_passed" = true ] && echo "true" || echo "false"),
            "passed": $cross_compile_passed
        },
        "security_tests": {
            "status": $([ "$security_passed" = true ] && echo "true" || echo "false"),
            "passed": $security_passed
        }
    },
    "overall_status": $([ "$unit_passed" = true ] && [ "$integration_passed" = true ] && [ "$performance_passed" = true ] && [ "$cross_compile_passed" = true ] && [ "$security_passed" = true ] && echo '"PASS"' || echo '"FAIL"')
}
EOF
    
    success "Generated test report: $report_file"
}

# Main test execution
main() {
    log "Starting MultiOS container test suite..."
    
    local failed_tests=()
    
    # Run tests for each architecture
    for arch in "${ARCHITECTURES[@]}"; do
        log "Testing architecture: $arch"
        
        # Initialize test status file
        echo "ARCHITECTURE=$arch" > "$TEST_REPORT_DIR/test_status_${arch}.txt"
        echo "TIMESTAMP=$TIMESTAMP" >> "$TEST_REPORT_DIR/test_status_${arch}.txt"
        
        # Run all test suites
        if ! run_unit_tests "$arch"; then
            failed_tests+=("unit_tests_$arch")
        fi
        
        if ! run_integration_tests "$arch"; then
            failed_tests+=("integration_tests_$arch")
        fi
        
        if ! run_performance_tests "$arch"; then
            failed_tests+=("performance_tests_$arch")
        fi
        
        if ! validate_cross_compilation "$arch"; then
            failed_tests+=("cross_compile_$arch")
        fi
        
        if ! run_security_tests "$arch"; then
            failed_tests+=("security_tests_$arch")
        fi
        
        # Generate individual report
        generate_test_report "$arch"
    done
    
    # Generate overall report
    local overall_status="PASS"
    if [ ${#failed_tests[@]} -gt 0 ]; then
        overall_status="FAIL"
    fi
    
    cat > "$TEST_REPORT_DIR/overall_report_${TIMESTAMP}.json" << EOF
{
    "timestamp": "$TIMESTAMP",
    "overall_status": "$overall_status",
    "tested_architectures": [$(printf '"%s",' "${ARCHITECTURES[@]}" | sed 's/,$//')],
    "failed_tests": [$(printf '"%s",' "${failed_tests[@]}" | sed 's/,$//')],
    "total_tests": $(( ${#ARCHITECTURES[@]} * 5 )),
    "passed_tests": $(( ${#ARCHITECTURES[@]} * 5 - ${#failed_tests[@]} ))
}
EOF
    
    # Final status
    if [ "$overall_status" = "PASS" ]; then
        success "All tests passed successfully!"
        exit 0
    else
        error "Some tests failed: ${failed_tests[*]}"
        exit 1
    fi
}

# Run main function
main "$@"