#!/bin/bash

# MultiOS Monitoring Integration Script
# Integrates CI/CD pipeline with performance monitoring tools and dashboards

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MONITORING_CONFIG="/workspace/testing/ci_cd/monitoring/config"
DASHBOARD_CONFIG="/workspace/perf/monitor_dashboard"
REPORTS_DIR="/workspace/reports"
ARTIFACTS_DIR="/workspace/artifacts"
LOG_DIR="/workspace/logs"

# Ensure directories exist
mkdir -p "$MONITORING_CONFIG" "$REPORTS_DIR" "$ARTIFACTS_DIR" "$LOG_DIR"

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

# Collect build metrics
collect_build_metrics() {
    log "Collecting build metrics..."
    
    local metrics_file="$REPORTS_DIR/build_metrics_$(date +%Y%m%d_%H%M%S).json"
    local build_start=$(date +%s)
    
    # Collect system metrics during build
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | awk -F'%' '{print $1}' || echo "0")
    local memory_info=$(free -m | awk 'NR==2{printf "%d:%d", $3,$2}')
    local disk_usage=$(df /workspace | tail -1 | awk '{print $5}' | sed 's/%//' || echo "0")
    
    # Build completion time
    local build_end=$(date +%s)
    local build_duration=$(( build_end - build_start ))
    
    # Save build metrics
    cat > "$metrics_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "build_duration_seconds": $build_duration,
    "system_metrics": {
        "cpu_usage_percent": $cpu_usage,
        "memory_used_mb": $(echo "$memory_info" | cut -d: -f1),
        "memory_total_mb": $(echo "$memory_info" | cut -d: -f2),
        "disk_usage_percent": $disk_usage
    },
    "build_info": {
        "target_architectures": ["x86_64", "arm64", "riscv64"],
        "cargo_features": "all-features"
    }
}
EOF
    
    success "Build metrics collected: $metrics_file"
    
    # Update dashboard data
    update_dashboard_data "build_metrics" "$metrics_file"
}

# Collect test metrics
collect_test_metrics() {
    log "Collecting test metrics..."
    
    local metrics_file="$REPORTS_DIR/test_metrics_$(date +%Y%m%d_%H%M%S).json"
    local test_start=$(date +%s)
    
    # Parse test results
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    local test_duration=0
    
    for report_file in "$REPORTS_DIR"/test_report_*.json; do
        if [ -f "$report_file" ]; then
            local arch=$(jq -r '.architecture' "$report_file")
            local status=$(jq -r '.overall_status' "$report_file")
            
            # Count test types (5 tests per architecture)
            ((total_tests += 5))
            if [ "$status" = "PASS" ]; then
                ((passed_tests += 5))
            else
                # Count actual passed tests
                local passed_count=0
                if [ "$(jq -r '.tests.unit_tests.passed' "$report_file")" = "true" ]; then ((passed_count++)); fi
                if [ "$(jq -r '.tests.integration_tests.passed' "$report_file")" = "true" ]; then ((passed_count++)); fi
                if [ "$(jq -r '.tests.performance_tests.passed' "$report_file")" = "true" ]; then ((passed_count++)); fi
                if [ "$(jq -r '.tests.cross_compilation.passed' "$report_file")" = "true" ]; then ((passed_count++)); fi
                if [ "$(jq -r '.tests.security_tests.passed' "$report_file")" = "true" ]; then ((passed_count++)); fi
                
                ((passed_tests += passed_count))
                ((failed_tests += 5 - passed_count))
            fi
        fi
    done
    
    local test_end=$(date +%s)
    test_duration=$(( test_end - test_start ))
    
    # Calculate pass rate
    local pass_rate=0
    if [ $total_tests -gt 0 ]; then
        pass_rate=$(( passed_tests * 100 / total_tests ))
    fi
    
    # Save test metrics
    cat > "$metrics_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "test_duration_seconds": $test_duration,
    "test_results": {
        "total_tests": $total_tests,
        "passed_tests": $passed_tests,
        "failed_tests": $failed_tests,
        "pass_rate_percent": $pass_rate
    },
    "architectures_tested": ["x86_64", "arm64", "riscv64"]
}
EOF
    
    success "Test metrics collected: $metrics_file"
    
    # Update dashboard data
    update_dashboard_data "test_metrics" "$metrics_file"
}

# Collect performance metrics
collect_performance_metrics() {
    log "Collecting performance metrics..."
    
    local metrics_file="$REPORTS_DIR/performance_metrics_$(date +%Y%m%d_%H%M%S).json"
    
    # Parse benchmark results
    local boot_times='{}'
    local memory_usage='{}'
    local cpu_scores='{}'
    
    for comparison_file in "$REPORTS_DIR"/benchmark_comparison_*.json; do
        if [ -f "$comparison_file" ]; then
            local arch=$(jq -r '.architecture' "$comparison_file")
            local boot_time=$(jq -r '.current.boot_time_ms' "$comparison_file")
            local memory_kb=$(jq -r '.current.memory_usage_kb' "$comparison_file")
            local cpu_score=$(jq -r '.current.cpu_score' "$comparison_file")
            
            # Update JSON objects
            boot_times=$(echo "$boot_times" | jq --arg arch "$arch" --arg value "$boot_time" '. + {($arch): ($value | tonumber)}')
            memory_usage=$(echo "$memory_usage" | jq --arg arch "$arch" --arg value "$memory_kb" '. + {($arch): ($value | tonumber)}')
            cpu_scores=$(echo "$cpu_scores" | jq --arg arch "$arch" --arg value "$cpu_score" '. + {($arch): ($value | tonumber)}')
        fi
    done
    
    # Save performance metrics
    cat > "$metrics_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "performance_metrics": {
        "boot_times_ms": $boot_times,
        "memory_usage_kb": $memory_usage,
        "cpu_scores": $cpu_scores
    },
    "monitoring_period": "$(date -Iseconds)"
}
EOF
    
    success "Performance metrics collected: $metrics_file"
    
    # Update dashboard data
    update_dashboard_data "performance_metrics" "$metrics_file"
}

# Update dashboard data
update_dashboard_data() {
    local data_type=$1
    local source_file=$2
    
    log "Updating dashboard data: $data_type"
    
    # Create dashboard data directory if it doesn't exist
    local dashboard_data_dir="$DASHBOARD_CONFIG/data/ci_cd"
    mkdir -p "$dashboard_data_dir"
    
    # Copy data to dashboard
    cp "$source_file" "$dashboard_data_dir/$(basename "$source_file")"
    
    # Update latest data symlinks
    case "$data_type" in
        "build_metrics")
            ln -sf "$source_file" "$dashboard_data_dir/latest_build_metrics.json"
            ;;
        "test_metrics")
            ln -sf "$source_file" "$dashboard_data_dir/latest_test_metrics.json"
            ;;
        "performance_metrics")
            ln -sf "$source_file" "$dashboard_data_dir/latest_performance_metrics.json"
            ;;
    esac
}

# Update dashboard alerts
update_dashboard_alerts() {
    log "Updating dashboard alerts..."
    
    local alerts_file="$REPORTS_DIR/dashboard_alerts_$(date +%Y%m%d_%H%M%S).json"
    local active_alerts='[]'
    
    # Check for quality gate failures
    if [ -f "$REPORTS_DIR/quality_gate_status.txt" ] && [ "$(cat "$REPORTS_DIR/quality_gate_status.txt")" = "FAIL" ]; then
        active_alerts=$(echo "$active_alerts" | jq '. + [{
            "severity": "high",
            "message": "Quality gates failed - build rejected",
            "timestamp": "'$(date -Iseconds)'",
            "type": "quality_gate"
        }]')
    fi
    
    # Check for test failures
    local failed_tests=0
    for report_file in "$REPORTS_DIR"/test_report_*.json; do
        if [ -f "$report_file" ] && [ "$(jq -r '.overall_status' "$report_file")" = "FAIL" ]; then
            ((failed_tests++))
        fi
    done
    
    if [ $failed_tests -gt 0 ]; then
        active_alerts=$(echo "$active_alerts" | jq '. + [{
            "severity": "medium",
            "message": "'$failed_tests' architecture(s) have test failures",
            "timestamp": "'$(date -Iseconds)'",
            "type": "test_failure"
        }]')
    fi
    
    # Check for performance regressions
    local performance_regressions=0
    for comparison_file in "$REPORTS_DIR"/benchmark_comparison_*.json; do
        if [ -f "$comparison_file" ]; then
            local boot_regr=$(jq -r '.regression_flags.boot_time_regression' "$comparison_file" 2>/dev/null || echo "false")
            local mem_regr=$(jq -r '.regression_flags.memory_regression' "$comparison_file" 2>/dev/null || echo "false")
            local cpu_regr=$(jq -r '.regression_flags.cpu_regression' "$comparison_file" 2>/dev/null || echo "false")
            
            if [ "$boot_regr" = "true" ] || [ "$mem_regr" = "true" ] || [ "$cpu_regr" = "true" ]; then
                ((performance_regressions++))
            fi
        fi
    done
    
    if [ $performance_regressions -gt 0 ]; then
        active_alerts=$(echo "$active_alerts" | jq '. + [{
            "severity": "medium",
            "message": "Performance regressions detected in '$performance_regressions' architecture(s)",
            "timestamp": "'$(date -Iseconds)'",
            "type": "performance_regression"
        }]')
    fi
    
    # Save alerts
    cat > "$alerts_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "total_alerts": $(echo "$active_alerts" | jq 'length'),
    "alerts": $active_alerts
}
EOF
    
    # Update dashboard alerts
    local dashboard_data_dir="$DASHBOARD_CONFIG/data/ci_cd"
    mkdir -p "$dashboard_data_dir"
    cp "$alerts_file" "$dashboard_data_dir/"
    ln -sf "$alerts_file" "$dashboard_data_dir/latest_alerts.json"
    
    success "Dashboard alerts updated"
}

# Generate monitoring report
generate_monitoring_report() {
    log "Generating monitoring report..."
    
    local report_file="$REPORTS_DIR/monitoring_report_$(date +%Y%m%d_%H%M%S).json"
    
    # Collect latest metrics
    local build_metrics=$(find "$REPORTS_DIR" -name "build_metrics_*.json" -type f -exec ls -t {} + | head -1 | xargs cat 2>/dev/null || echo '{}')
    local test_metrics=$(find "$REPORTS_DIR" -name "test_metrics_*.json" -type f -exec ls -t {} + | head -1 | xargs cat 2>/dev/null || echo '{}')
    local perf_metrics=$(find "$REPORTS_DIR" -name "performance_metrics_*.json" -type f -exec ls -t {} + | head -1 | xargs cat 2>/dev/null || echo '{}')
    local alerts=$(find "$REPORTS_DIR" -name "dashboard_alerts_*.json" -type f -exec ls -t {} + | head -1 | xargs cat 2>/dev/null || echo '{}')
    
    # Generate comprehensive monitoring report
    cat > "$report_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "report_version": "1.0",
    "pipeline_status": "$(cat "$REPORTS_DIR/quality_gate_status.txt" 2>/dev/null || echo 'UNKNOWN')",
    "metrics": {
        "build": $build_metrics,
        "testing": $test_metrics,
        "performance": $perf_metrics
    },
    "alerts": $alerts,
    "monitoring_summary": {
        "data_sources": ["build_metrics", "test_metrics", "performance_metrics", "quality_gates"],
        "alert_count": $(echo "$alerts" | jq -r '.total_alerts' 2>/dev/null || echo '0'),
        "last_updated": "$(date -Iseconds)"
    }
}
EOF
    
    # Update dashboard with summary
    cp "$report_file" "$DASHBOARD_CONFIG/data/ci_cd/latest_monitoring_report.json"
    
    success "Monitoring report generated: $report_file"
}

# Trigger dashboard refresh
trigger_dashboard_refresh() {
    log "Triggering dashboard refresh..."
    
    # Check if dashboard service is running
    if [ -f "$DASHBOARD_CONFIG/start_dashboard.py" ]; then
        # Send refresh signal to dashboard
        if [ -f "$DASHBOARD_CONFIG/data/ci_cd/dashboard_refresh.fifo" ]; then
            echo "refresh" > "$DASHBOARD_CONFIG/data/ci_cd/dashboard_refresh.fifo"
        else
            # Create refresh mechanism
            mkdir -p "$DASHBOARD_CONFIG/data/ci_cd"
            mkfifo "$DASHBOARD_CONFIG/data/ci_cd/dashboard_refresh.fifo" 2>/dev/null || true
            echo "refresh" > "$DASHBOARD_CONFIG/data/ci_cd/dashboard_refresh.fifo" &
        fi
        
        success "Dashboard refresh triggered"
    else
        warning "Dashboard service not found - refresh skipped"
    fi
}

# Update Prometheus metrics (if available)
update_prometheus_metrics() {
    log "Updating Prometheus metrics..."
    
    local prometheus_metrics_dir="$MONITORING_CONFIG/prometheus"
    mkdir -p "$prometheus_metrics_dir"
    
    local metrics_file="$prometheus_metrics_dir/ci_cd_metrics_$(date +%Y%m%d_%H%M%S).prom"
    
    # Generate Prometheus-style metrics
    cat > "$metrics_file" << EOF
# MultiOS CI/CD Pipeline Metrics
# Generated at $(date -Iseconds)

# Build metrics
multios_build_duration_seconds $(date +%s)
multios_build_cpu_usage_percent $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | awk -F'%' '{print $1}' || echo "0")
multios_build_memory_usage_percent $(free | awk 'FNR == 2{printf "%.1f", $3*100/$2}')

# Test metrics
multios_test_pass_rate_percent $([ -f "$REPORTS_DIR/test_metrics_latest.json" ] && jq -r '.test_results.pass_rate_percent' "$REPORTS_DIR/test_metrics_latest.json" 2>/dev/null || echo "0")
multios_test_total_count $([ -f "$REPORTS_DIR/test_metrics_latest.json" ] && jq -r '.test_results.total_tests' "$REPORTS_DIR/test_metrics_latest.json" 2>/dev/null || echo "0")
multios_test_passed_count $([ -f "$REPORTS_DIR/test_metrics_latest.json" ] && jq -r '.test_results.passed_tests' "$REPORTS_DIR/test_metrics_latest.json" 2>/dev/null || echo "0")
multios_test_failed_count $([ -f "$REPORTS_DIR/test_metrics_latest.json" ] && jq -r '.test_results.failed_tests' "$REPORTS_DIR/test_metrics_latest.json" 2>/dev/null || echo "0")

# Quality gate metrics
multios_quality_gate_status $([ "$(cat "$REPORTS_DIR/quality_gate_status.txt" 2>/dev/null || echo 'UNKNOWN')" = "PASS" ] && echo "1" || echo "0")

# Architecture-specific metrics
multios_arch_x86_64_boot_time_ms $([ -f "$REPORTS_DIR/benchmark_comparison_x86_64_latest.json" ] && jq -r '.current.boot_time_ms' "$REPORTS_DIR/benchmark_comparison_x86_64_latest.json" 2>/dev/null || echo "0")
multios_arch_arm64_boot_time_ms $([ -f "$REPORTS_DIR/benchmark_comparison_arm64_latest.json" ] && jq -r '.current.boot_time_ms' "$REPORTS_DIR/benchmark_comparison_arm64_latest.json" 2>/dev/null || echo "0")
multios_arch_riscv64_boot_time_ms $([ -f "$REPORTS_DIR/benchmark_comparison_riscv64_latest.json" ] && jq -r '.current.boot_time_ms' "$REPORTS_DIR/benchmark_comparison_riscv64_latest.json" 2>/dev/null || echo "0")
EOF
    
    # Update latest metrics
    ln -sf "$metrics_file" "$prometheus_metrics_dir/latest_metrics.prom"
    
    success "Prometheus metrics updated: $metrics_file"
}

# Main monitoring integration
main() {
    log "Starting MultiOS Monitoring Integration"
    
    # Collect all metrics
    collect_build_metrics
    collect_test_metrics
    collect_performance_metrics
    
    # Update dashboard
    update_dashboard_alerts
    generate_monitoring_report
    trigger_dashboard_refresh
    
    # Update external monitoring systems
    update_prometheus_metrics
    
    success "Monitoring integration complete"
}

main "$@"