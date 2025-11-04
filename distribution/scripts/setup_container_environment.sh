#!/bin/bash

# MultiOS Container Environment Setup Script
# This script sets up the testing environment for containerized testing

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Create required directories
setup_directories() {
    log "Setting up directory structure..."
    
    mkdir -p /workspace/{logs,reports,artifacts,config,qemu,test_data}
    mkdir -p /workspace/config/{test_scenarios,performance_baselines}
    mkdir -p /workspace/qemu/{configs,templates,snapshots}
    
    success "Directory structure created"
}

# Set up QEMU configurations
setup_qemu_configs() {
    log "Setting up QEMU configurations..."
    
    # Create QEMU configuration files for each architecture
    cat > /workspace/qemu/configs/x86_64.conf << 'EOF'
# QEMU Configuration for x86_64
ACCEL=kvm
MACHINE_TYPE=pc-q35-6.2
SMP=4
MEMORY=4G
KERNEL=/workspace/bootloader/target/x86_64-unknown-none/release/multios
APPEND="console=ttyS0 loglevel=8"
SERIAL="-serial stdio"
MONITOR="-monitor telnet:127.0.0.1:4444,server,nowait"
EOF

    cat > /workspace/qemu/configs/arm64.conf << 'EOF'
# QEMU Configuration for ARM64
ACCEL=kvm
MACHINE_TYPE=virt
SMP=4
MEMORY=4G
KERNEL=/workspace/bootloader/target/aarch64-unknown-none/release/multios
APPEND="console=ttyAMA0 loglevel=8"
SERIAL="-serial stdio"
MONITOR="-monitor telnet:127.0.0.1:4445,server,nowait"
EOF

    cat > /workspace/qemu/configs/riscv64.conf << 'EOF'
# QEMU Configuration for RISC-V64
ACCEL=kvm
MACHINE_TYPE=spike_virt
SMP=4
MEMORY=4G
KERNEL=/workspace/bootloader/target/riscv64gc-unknown-none-elf/release/multios
APPEND="console=ttyS0 loglevel=8"
SERIAL="-serial stdio"
MONITOR="-monitor telnet:127.0.0.1:4446,server,nowait"
EOF

    success "QEMU configurations created"
}

# Create test configuration files
setup_test_configs() {
    log "Setting up test configurations..."
    
    # Main test configuration
    cat > /workspace/config/global_test_config.toml << 'EOF'
# Global Test Configuration for MultiOS CI/CD Pipeline

[general]
test_timeout = 300
retry_attempts = 3
parallel_jobs = 4
log_level = "info"

[architectures]
x86_64 = { enabled = true, qemu_args = "-m 4096 -smp 4" }
arm64 = { enabled = true, qemu_args = "-m 4096 -smp 4" }
riscv64 = { enabled = true, qemu_args = "-m 4096 -smp 4" }

[test_suites]
unit_tests = { enabled = true, timeout = 60 }
integration_tests = { enabled = true, timeout = 180 }
performance_tests = { enabled = true, timeout = 300 }
security_tests = { enabled = true, timeout = 120 }
stress_tests = { enabled = true, timeout = 600 }

[quality_gates]
code_coverage_min = 80
performance_regression_max = 5
security_vulnerabilities = "critical,high"
memory_leak_tolerance = 0

[reporting]
output_formats = ["json", "xml", "html"]
artifact_retention_days = 30
benchmark_baseline_file = "performance_baselines.json"
EOF

    # Performance baseline configuration
    cat > /workspace/config/performance_baselines/baseline_config.json << 'EOF'
{
    "version": "1.0",
    "baselines": {
        "boot_time": {
            "x86_64": { "target_ms": 500, "threshold_ms": 100 },
            "arm64": { "target_ms": 600, "threshold_ms": 150 },
            "riscv64": { "target_ms": 800, "threshold_ms": 200 }
        },
        "memory_usage": {
            "x86_64": { "target_mb": 512, "threshold_mb": 128 },
            "arm64": { "target_mb": 512, "threshold_mb": 128 },
            "riscv64": { "target_mb": 512, "threshold_mb": 128 }
        },
        "cpu_performance": {
            "x86_64": { "target_score": 100, "threshold_score": 10 },
            "arm64": { "target_score": 80, "threshold_score": 15 },
            "riscv64": { "target_score": 60, "threshold_score": 20 }
        }
    }
}
EOF

    # Test scenarios configuration
    cat > /workspace/config/test_scenarios/scenarios.toml << 'EOF'
# Test Scenarios Configuration

[basic_functionality]
name = "Basic System Functionality"
description = "Test basic system operations and core functionality"
tests = [
    "system_booting",
    "memory_management",
    "process_creation",
    "file_operations",
    "interrupts"
]
timeout = 300

[network_testing]
name = "Network Stack Testing"
description = "Test network communication and protocols"
tests = [
    "tcp_communication",
    "udp_communication",
    "icmp_ping",
    "dns_resolution"
]
timeout = 180

[driver_testing]
name = "Device Driver Testing"
description = "Test device driver functionality"
tests = [
    "storage_driver",
    "network_driver",
    "input_driver",
    "display_driver"
]
timeout = 240

[stress_testing]
name = "System Stress Testing"
description = "Test system under heavy load"
tests = [
    "memory_stress",
    "cpu_stress",
    "io_stress",
    "concurrent_processes"
]
timeout = 600
EOF

    success "Test configurations created"
}

# Set up monitoring and logging
setup_monitoring() {
    log "Setting up monitoring and logging..."
    
    # Create logging configuration
    cat > /workspace/config/logging_config.yaml << 'EOF'
version: 1
formatters:
  detailed:
    format: '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
handlers:
  console:
    class: logging.StreamHandler
    level: INFO
    formatter: detailed
  file:
    class: logging.FileHandler
    level: DEBUG
    formatter: detailed
    filename: /workspace/logs/test_execution.log
loggers:
  multios.test:
    level: DEBUG
    handlers: [console, file]
root:
  level: INFO
  handlers: [console, file]
EOF

    # Create performance monitoring script
    cat > /workspace/scripts/monitor_performance.sh << 'EOF'
#!/bin/bash
# Performance monitoring script for CI/CD pipeline

set -euo pipefail

LOG_FILE="/workspace/logs/performance_monitor.log"

log_metric() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Monitor CPU usage
cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | awk -F'%' '{print $1}')
log_metric "CPU_USAGE:${cpu_usage}"

# Monitor memory usage
mem_info=$(free -m | awk 'NR==2{printf "MEMORY:%d:%d:%d", $3,$2,$7}')
log_metric "$mem_info"

# Monitor disk I/O
io_stats=$(iostat -x 1 1 | tail -n +4 | head -1 | awk '{print "DISK_IO:read=%s,write=%s", $4, $5}')
log_metric "$io_stats"

# Monitor network (if available)
if command -v ifstat >/dev/null 2>&1; then
    net_stats=$(ifstat 1 1 | tail -n +3 | head -1 | awk '{print "NETWORK:rx=%s,tx=%s", $2, $3}')
    log_metric "$net_stats"
fi

echo "Performance metrics logged to $LOG_FILE"
EOF

    chmod +x /workspace/scripts/monitor_performance.sh

    success "Monitoring and logging setup complete"
}

# Set up quality gates
setup_quality_gates() {
    log "Setting up quality gates..."
    
    cat > /workspace/scripts/quality_gate_check.sh << 'EOF'
#!/bin/bash
# Quality gate validation script

set -euo pipefail

QUALITY_GATE_STATUS="PASS"
FAILED_GATES=()

# Check code coverage
check_code_coverage() {
    local coverage_file="$1"
    if [ -f "$coverage_file" ]; then
        local coverage=$(grep -o '[0-9.]*%' "$coverage_file" | head -1 | sed 's/%//')
        if [ "$(echo "$coverage < 80" | bc -l)" -eq 1 ]; then
            QUALITY_GATE_STATUS="FAIL"
            FAILED_GATES+=("code_coverage:$coverage")
        fi
    fi
}

# Check security vulnerabilities
check_security_vulnerabilities() {
    local audit_file="$1"
    if [ -f "$audit_file" ]; then
        local critical=$(grep -c "critical\|high" "$audit_file" || true)
        if [ "$critical" -gt 0 ]; then
            QUALITY_GATE_STATUS="FAIL"
            FAILED_GATES+=("security_vulnerabilities:$critical")
        fi
    fi
}

# Check performance regression
check_performance_regression() {
    local current_file="$1"
    local baseline_file="$2"
    
    if [ -f "$current_file" ] && [ -f "$baseline_file" ]; then
        # Compare current performance with baseline
        # This is a simplified check - in practice, you'd parse JSON and compare metrics
        local regression_detected=false
        
        # Example: Check boot time regression
        if [ -f "$current_file" ] && [ -f "$baseline_file" ]; then
            # Parse JSON and compare values (simplified)
            if grep -q "boot_time_regression" "$current_file"; then
                regression_detected=true
            fi
        fi
        
        if [ "$regression_detected" = true ]; then
            QUALITY_GATE_STATUS="FAIL"
            FAILED_GATES+=("performance_regression")
        fi
    fi
}

# Check test pass rate
check_test_pass_rate() {
    local test_results_file="$1"
    local min_pass_rate=95
    
    if [ -f "$test_results_file" ]; then
        local total_tests=$(grep -o 'total_tests":[0-9]*' "$test_results_file" | grep -o '[0-9]*')
        local passed_tests=$(grep -o 'passed_tests":[0-9]*' "$test_results_file" | grep -o '[0-9]*')
        
        if [ -n "$total_tests" ] && [ -n "$passed_tests" ]; then
            local pass_rate=$(( passed_tests * 100 / total_tests ))
            if [ "$pass_rate" -lt "$min_pass_rate" ]; then
                QUALITY_GATE_STATUS="FAIL"
                FAILED_GATES+=("test_pass_rate:$pass_rate")
            fi
        fi
    fi
}

# Main execution
main() {
    local coverage_file="${1:-/workspace/reports/coverage.json}"
    local audit_file="${2:-/workspace/logs/security_audit.log}"
    local current_perf_file="${3:-/workspace/reports/current_performance.json}"
    local baseline_perf_file="${4:-/workspace/config/performance_baselines/baseline_config.json}"
    local test_results_file="${5:-/workspace/reports/overall_report.json}"
    
    check_code_coverage "$coverage_file"
    check_security_vulnerabilities "$audit_file"
    check_performance_regression "$current_perf_file" "$baseline_perf_file"
    check_test_pass_rate "$test_results_file"
    
    # Save quality gate status
    echo "QUALITY_GATE_STATUS=$QUALITY_GATE_STATUS" > /workspace/reports/quality_gate_status.txt
    echo "FAILED_GATES=$(IFS=,; echo "${FAILED_GATES[*]}")" >> /workspace/reports/quality_gate_status.txt
    
    # Generate quality gate report
    cat > /workspace/reports/quality_gate_report.json << EOF
{
    "timestamp": "$(date -Iseconds)",
    "status": "$QUALITY_GATE_STATUS",
    "failed_gates": [$(IFS=,; echo "\"${FAILED_GATES[*]}\"")],
    "summary": {
        "total_gates_checked": 4,
        "passed_gates": $(( 4 - ${#FAILED_GATES[@]} )),
        "failed_gates": ${#FAILED_GATES[@]}
    }
}
EOF
    
    if [ "$QUALITY_GATE_STATUS" = "PASS" ]; then
        echo "All quality gates passed!"
        exit 0
    else
        echo "Quality gate failure: ${FAILED_GATES[*]}"
        exit 1
    fi
}

main "$@"
EOF

    chmod +x /workspace/scripts/quality_gate_check.sh

    success "Quality gates setup complete"
}

# Main execution
main() {
    log "Setting up MultiOS container environment..."
    
    setup_directories
    setup_qemu_configs
    setup_test_configs
    setup_monitoring
    setup_quality_gates
    
    success "Container environment setup complete!"
    
    # Show final status
    log "Environment summary:"
    log "- Test configuration: /workspace/config/global_test_config.toml"
    log "- QEMU configs: /workspace/qemu/configs/"
    log "- Scripts: /workspace/scripts/"
    log "- Logs: /workspace/logs/"
    log "- Reports: /workspace/reports/"
    log "- Artifacts: /workspace/artifacts/"
}

main "$@"