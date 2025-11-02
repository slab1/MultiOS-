#!/bin/bash

# Quality Gate Report Generator
# Analyzes test results and generates comprehensive quality gate reports

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPORTS_DIR="/workspace/reports"
LOG_DIR="/workspace/logs"
CONFIG_DIR="/workspace/config"
ARTIFACTS_DIR="/workspace/artifacts"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Quality gate thresholds
MIN_COVERAGE=80
MAX_PERFORMANCE_REGRESSION=5
MAX_MEMORY_REGRESSION=10
MIN_TEST_PASS_RATE=95
ALLOWED_SECURITY_ISSUES=0

# Ensure directories exist
mkdir -p "$REPORTS_DIR" "$LOG_DIR"

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

# Analyze test results
analyze_test_results() {
    log "Analyzing test results..."
    
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    local test_results=()
    
    # Process individual architecture reports
    for report_file in "$REPORTS_DIR"/test_report_*.json; do
        if [ -f "$report_file" ]; then
            local arch=$(jq -r '.architecture' "$report_file")
            local overall_status=$(jq -r '.overall_status' "$report_file")
            
            log "Processing test results for $arch: $overall_status"
            
            # Count test results
            local unit_tests_passed=$(jq -r '.tests.unit_tests.passed' "$report_file")
            local integration_tests_passed=$(jq -r '.tests.integration_tests.passed' "$report_file")
            local performance_tests_passed=$(jq -r '.tests.performance_tests.passed' "$report_file")
            local cross_compile_passed=$(jq -r '.tests.cross_compilation.passed' "$report_file")
            local security_tests_passed=$(jq -r '.tests.security_tests.passed' "$report_file")
            
            local arch_passed=0
            local arch_failed=0
            
            if [ "$unit_tests_passed" = "true" ]; then
                ((arch_passed++))
            else
                ((arch_failed++))
            fi
            
            if [ "$integration_tests_passed" = "true" ]; then
                ((arch_passed++))
            else
                ((arch_failed++))
            fi
            
            if [ "$performance_tests_passed" = "true" ]; then
                ((arch_passed++))
            else
                ((arch_failed++))
            fi
            
            if [ "$cross_compile_passed" = "true" ]; then
                ((arch_passed++))
            else
                ((arch_failed++))
            fi
            
            if [ "$security_tests_passed" = "true" ]; then
                ((arch_passed++))
            else
                ((arch_failed++))
            fi
            
            ((total_tests += arch_passed + arch_failed))
            ((passed_tests += arch_passed))
            ((failed_tests += arch_failed))
            
            test_results+=("{\"architecture\":\"$arch\",\"passed\":$arch_passed,\"failed\":$arch_failed,\"status\":\"$overall_status\"}")
        fi
    done
    
    # Calculate pass rate
    local pass_rate=0
    if [ $total_tests -gt 0 ]; then
        pass_rate=$(( passed_tests * 100 / total_tests ))
    fi
    
    # Save analysis results
    cat > "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "summary": {
        "total_tests": $total_tests,
        "passed_tests": $passed_tests,
        "failed_tests": $failed_tests,
        "pass_rate": $pass_rate
    },
    "architecture_results": [
$(IFS=,; printf '%s' "${test_results[*]}")
    ],
    "quality_gate": {
        "test_pass_rate": $pass_rate,
        "threshold": $MIN_TEST_PASS_RATE,
        "passed": $([ "$pass_rate" -ge "$MIN_TEST_PASS_RATE" ] && echo "true" || echo "false")
    }
}
EOF
    
    success "Test analysis complete: $passed_tests/$total_tests passed ($pass_rate%)"
    return $([ "$pass_rate" -ge "$MIN_TEST_PASS_RATE" ] && echo 0 || echo 1)
}

# Analyze code coverage
analyze_code_coverage() {
    log "Analyzing code coverage..."
    
    local coverage_files=("$REPORTS_DIR"/coverage_*.json "$REPORTS_DIR"/lcov.info* 2>/dev/null || true)
    local overall_coverage=0
    local coverage_files_found=0
    
    for coverage_file in "${coverage_files[@]}"; do
        if [ -f "$coverage_file" ]; then
            ((coverage_files_found++))
            
            if [[ "$coverage_file" == *.json ]]; then
                local coverage=$(jq -r '.coverage' "$coverage_file" 2>/dev/null || echo "0")
            else
                local coverage=$(grep -o '[0-9]*\.[0-9]*%' "$coverage_file" | head -1 | sed 's/%//' 2>/dev/null || echo "0")
            fi
            
            if [ -n "$coverage" ] && [ "$coverage" != "null" ]; then
                overall_coverage=$(( overall_coverage + coverage ))
                log "Coverage from $coverage_file: ${coverage}%"
            fi
        fi
    done
    
    if [ $coverage_files_found -gt 0 ]; then
        overall_coverage=$(( overall_coverage / coverage_files_found ))
    fi
    
    # Save coverage analysis
    cat > "$REPORTS_DIR/coverage_analysis_${TIMESTAMP}.json" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "overall_coverage": $overall_coverage,
    "threshold": $MIN_COVERAGE,
    "passed": $([ "$overall_coverage" -ge "$MIN_COVERAGE" ] && echo "true" || echo "false"),
    "coverage_files_found": $coverage_files_found
}
EOF
    
    if [ "$overall_coverage" -ge "$MIN_COVERAGE" ]; then
        success "Code coverage: ${overall_coverage}% (threshold: ${MIN_COVERAGE}%)"
        return 0
    else
        warning "Code coverage: ${overall_coverage}% (threshold: ${MIN_COVERAGE}%)"
        return 1
    fi
}

# Analyze security results
analyze_security() {
    log "Analyzing security results..."
    
    local security_issues=0
    local critical_issues=0
    local high_issues=0
    
    # Check for security audit results
    for audit_file in "$LOG_DIR"/security_audit* "$REPORTS_DIR"/trivy*; do
        if [ -f "$audit_file" ]; then
            local critical=$(grep -c "critical\|CRITICAL" "$audit_file" || echo "0")
            local high=$(grep -c "high\|HIGH" "$audit_file" || echo "0")
            
            ((critical_issues += critical))
            ((high_issues += high))
            ((security_issues += critical + high))
            
            log "Security issues in $audit_file: critical=$critical, high=$high"
        fi
    done
    
    # Check for clippy warnings
    local clippy_warnings=$(grep -c "warning\|WARNING" "$LOG_DIR"/clippy* 2>/dev/null || echo "0")
    ((security_issues += clippy_warnings))
    
    # Save security analysis
    cat > "$REPORTS_DIR/security_analysis_${TIMESTAMP}.json" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "total_issues": $security_issues,
    "critical_issues": $critical_issues,
    "high_issues": $high_issues,
    "clippy_warnings": $clippy_warnings,
    "threshold": $ALLOWED_SECURITY_ISSUES,
    "passed": $([ "$security_issues" -le "$ALLOWED_SECURITY_ISSUES" ] && echo "true" || echo "false")
}
EOF
    
    if [ "$security_issues" -le "$ALLOWED_SECURITY_ISSUES" ]; then
        success "Security analysis: $security_issues issues found (threshold: $ALLOWED_SECURITY_ISSUES)"
        return 0
    else
        error "Security analysis: $security_issues issues found (threshold: $ALLOWED_SECURITY_ISSUES)"
        return 1
    fi
}

# Analyze performance results
analyze_performance() {
    log "Analyzing performance results..."
    
    local performance_regressions=0
    local performance_files=("$REPORTS_DIR"/benchmark_comparison_*.json)
    
    for perf_file in "${performance_files[@]}"; do
        if [ -f "$perf_file" ]; then
            # Check for performance regressions
            local boot_regression=$(jq -r '.regression_flags.boot_time_regression' "$perf_file" 2>/dev/null || echo "false")
            local memory_regression=$(jq -r '.regression_flags.memory_regression' "$perf_file" 2>/dev/null || echo "false")
            local cpu_regression=$(jq -r '.regression_flags.cpu_regression' "$perf_file" 2>/dev/null || echo "false")
            
            if [ "$boot_regression" = "true" ] || [ "$memory_regression" = "true" ] || [ "$cpu_regression" = "true" ]; then
                ((performance_regressions++))
                warning "Performance regression detected in $(basename "$perf_file")"
            fi
        fi
    done
    
    # Check performance thresholds
    for comparison_file in "$REPORTS_DIR"/benchmark_comparison_*.json; do
        if [ -f "$comparison_file" ]; then
            local boot_change=$(jq -r '.changes.boot_time_change_percent' "$comparison_file" 2>/dev/null || echo "0")
            local memory_change=$(jq -r '.changes.memory_usage_change_percent' "$comparison_file" 2>/dev/null || echo "0")
            local cpu_change=$(jq -r '.changes.cpu_performance_change_percent' "$comparison_file" 2>/dev/null || echo "0")
            
            if [ "$boot_change" -gt "$MAX_PERFORMANCE_REGRESSION" ]; then
                warning "Boot time regression: ${boot_change}% (threshold: ${MAX_PERFORMANCE_REGRESSION}%)"
                ((performance_regressions++))
            fi
            
            if [ "$memory_change" -gt "$MAX_MEMORY_REGRESSION" ]; then
                warning "Memory usage regression: ${memory_change}% (threshold: ${MAX_MEMORY_REGRESSION}%)"
                ((performance_regressions++))
            fi
            
            if [ "$cpu_change" -lt $((-MAX_PERFORMANCE_REGRESSION)) ]; then
                warning "CPU performance regression: ${cpu_change}% (threshold: -${MAX_PERFORMANCE_REGRESSION}%)"
                ((performance_regressions++))
            fi
        fi
    done
    
    # Save performance analysis
    cat > "$REPORTS_DIR/performance_analysis_${TIMESTAMP}.json" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "regressions_detected": $performance_regressions,
    "thresholds": {
        "max_boot_time_regression": $MAX_PERFORMANCE_REGRESSION,
        "max_memory_regression": $MAX_MEMORY_REGRESSION,
        "max_cpu_performance_regression": -$MAX_PERFORMANCE_REGRESSION
    },
    "passed": $([ "$performance_regressions" -eq 0 ] && echo "true" || echo "false")
}
EOF
    
    if [ "$performance_regressions" -eq 0 ]; then
        success "Performance analysis: No regressions detected"
        return 0
    else
        warning "Performance analysis: $performance_regressions regressions detected"
        return 1
    fi
}

# Generate quality gate status
generate_quality_gate_status() {
    log "Generating quality gate status..."
    
    # Run gate checks
    local gate_results=()
    local overall_status="PASS"
    
    # Test pass rate gate
    if analyze_test_results; then
        gate_results+=("{\"gate\":\"test_pass_rate\",\"status\":\"PASS\"}")
    else
        gate_results+=("{\"gate\":\"test_pass_rate\",\"status\":\"FAIL\"}")
        overall_status="FAIL"
    fi
    
    # Code coverage gate
    if analyze_code_coverage; then
        gate_results+=("{\"gate\":\"code_coverage\",\"status\":\"PASS\"}")
    else
        gate_results+=("{\"gate\":\"code_coverage\",\"status\":\"FAIL\"}")
        overall_status="FAIL"
    fi
    
    # Security gate
    if analyze_security; then
        gate_results+=("{\"gate\":\"security\",\"status\":\"PASS\"}")
    else
        gate_results+=("{\"gate\":\"security\",\"status\":\"FAIL\"}")
        overall_status="FAIL"
    fi
    
    # Performance gate
    if analyze_performance; then
        gate_results+=("{\"gate\":\"performance\",\"status\":\"PASS\"}")
    else
        gate_results+=("{\"gate\":\"performance\",\"status\":\"FAIL\"}")
        overall_status="FAIL"
    fi
    
    # Count gates
    local passed_gates=$(echo "${gate_results[@]}" | jq -r '.[] | select(.status == "PASS") | .gate' | wc -l)
    local total_gates=4
    
    # Generate quality gate report
    cat > "$REPORTS_DIR/quality_gate_${TIMESTAMP}.json" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "overall_status": "$overall_status",
    "gates_checked": $total_gates,
    "gates_passed": $passed_gates,
    "gates_failed": $(( total_gates - passed_gates )),
    "gate_results": [
$(IFS=,; printf '%s' "${gate_results[*]}")
    ],
    "metrics": {
        "min_test_pass_rate": $MIN_TEST_PASS_RATE,
        "min_code_coverage": $MIN_COVERAGE,
        "max_security_issues": $ALLOWED_SECURITY_ISSUES,
        "max_performance_regression": $MAX_PERFORMANCE_REGRESSION
    }
}
EOF
    
    # Save overall status
    echo "$overall_status" > "$REPORTS_DIR/quality_gate_status.txt"
    
    # Generate markdown report
    generate_markdown_report "$overall_status" "$passed_gates" "$total_gates"
    
    if [ "$overall_status" = "PASS" ]; then
        success "All quality gates passed! ($passed_gates/$total_gates)"
        return 0
    else
        error "Quality gates failed! ($passed_gates/$total_gates passed)"
        return 1
    fi
}

# Generate markdown report
generate_markdown_report() {
    local status=$1
    local passed=$2
    local total=$3
    local report_file="$REPORTS_DIR/quality_gate_report_${TIMESTAMP}.md"
    
    # Calculate icon based on status
    local icon="✅"
    if [ "$status" = "FAIL" ]; then
        icon="❌"
    fi
    
    cat > "$report_file" << EOF
# MultiOS Quality Gate Report

## Summary
$icon **Overall Status: $status**

**Gates Checked:** $passed/$total passed

**Generated:** $(date -Iseconds)

---

## Quality Gates

### 🧪 Test Pass Rate
- **Threshold:** $MIN_TEST_PASS_RATE%
- **Status:** $([ -f "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" ] && jq -r '.quality_gate.passed' "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" && echo "✅ PASS" || echo "❌ FAIL")
- **Details:** $([ -f "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" ] && jq -r '.summary.pass_rate' "$REPORTS_DIR/test_analysis_${TIMESTAMP}.json" || echo "0")% pass rate

### 📊 Code Coverage
- **Threshold:** $MIN_COVERAGE%
- **Status:** $([ -f "$REPORTS_DIR/coverage_analysis_${TIMESTAMP}.json" ] && jq -r '.passed' "$REPORTS_DIR/coverage_analysis_${TIMESTAMP}.json" && echo "✅ PASS" || echo "❌ FAIL")
- **Details:** $([ -f "$REPORTS_DIR/coverage_analysis_${TIMESTAMP}.json" ] && jq -r '.overall_coverage' "$REPORTS_DIR/coverage_analysis_${TIMESTAMP}.json" || echo "0")% coverage

### 🔒 Security
- **Threshold:** ≤ $ALLOWED_SECURITY_ISSUES issues
- **Status:** $([ -f "$REPORTS_DIR/security_analysis_${TIMESTAMP}.json" ] && jq -r '.passed' "$REPORTS_DIR/security_analysis_${TIMESTAMP}.json" && echo "✅ PASS" || echo "❌ FAIL")
- **Details:** $([ -f "$REPORTS_DIR/security_analysis_${TIMESTAMP}.json" ] && jq -r '.total_issues' "$REPORTS_DIR/security_analysis_${TIMESTAMP}.json" || echo "0") issues found

### ⚡ Performance
- **Threshold:** No regressions
- **Status:** $([ -f "$REPORTS_DIR/performance_analysis_${TIMESTAMP}.json" ] && jq -r '.passed' "$REPORTS_DIR/performance_analysis_${TIMESTAMP}.json" && echo "✅ PASS" || echo "❌ FAIL")
- **Details:** $([ -f "$REPORTS_DIR/performance_analysis_${TIMESTAMP}.json" ] && jq -r '.regressions_detected' "$REPORTS_DIR/performance_analysis_${TIMESTAMP}.json" || echo "0") regressions detected

---

## Recommendations

$([ "$status" = "FAIL" ] && echo "⚠️ **Action Required:** Review failed gates and address issues before proceeding." || echo "✅ **Status Acceptable:** All quality gates have been met.")

## Next Steps

1. Review individual test reports in $REPORTS_DIR/
2. Check build logs in $LOG_DIR/
3. Address any failing quality gates
4. Re-run quality gate validation

---

*Report generated automatically by MultiOS CI/CD Pipeline*
EOF

    success "Markdown report generated: $report_file"
}

# Main execution
main() {
    log "Starting Quality Gate Analysis"
    
    # Generate quality gate status
    if generate_quality_gate_status; then
        success "Quality gate analysis completed successfully"
        exit 0
    else
        error "Quality gate analysis found issues"
        exit 1
    fi
}

main "$@"