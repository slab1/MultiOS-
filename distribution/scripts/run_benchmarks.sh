#!/bin/bash

# MultiOS Performance Benchmark Runner
# Runs comprehensive performance benchmarks across all architectures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Benchmark configuration
BENCHMARK_DIR="/workspace/reports"
ARTIFACTS_DIR="/workspace/artifacts"
LOG_DIR="/workspace/logs"
BASELINE_FILE="/workspace/config/performance_baselines/baseline_config.json"
ARCHITECTURES=("x86_64" "arm64" "riscv64")

# Ensure directories exist
mkdir -p "$BENCHMARK_DIR" "$ARTIFACTS_DIR" "$LOG_DIR"

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

# Run boot time benchmark
benchmark_boot_time() {
    local arch=$1
    local target="${arch}-unknown-none"
    local log_file="$LOG_DIR/boot_benchmark_${arch}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Running boot time benchmark for $arch"
    
    # Build the kernel for benchmarking
    case "$arch" in
        "x86_64")
            cargo build --target "$target" --release --features bench 2>&1 | tee "$log_file"
            ;;
        "arm64"|"riscv64")
            cross build --target "$target" --release --features bench 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -ne 0 ]; then
        error "Build failed for $arch boot benchmark"
        return 1
    fi
    
    # Run boot benchmark with QEMU
    local boot_time
    case "$arch" in
        "x86_64")
            boot_time=$(qemu-system-x86_64 -m 512M -smp 1 \
                -kernel "target/$target/release/multios" \
                -nographic \
                -serial stdio \
                -d guest_errors \
                -D "$log_file.qemu" \
                -append "console=ttyS0 loglevel=8 bench=boot" \
                -s 2>&1 | grep -o 'boot_time=[0-9]*' | cut -d= -f2 || echo "0")
            ;;
        "arm64")
            boot_time=$(qemu-system-aarch64 -m 512M -smp 1 \
                -machine virt \
                -cpu cortex-a57 \
                -kernel "target/$target/release/multios" \
                -nographic \
                -serial stdio \
                -d guest_errors \
                -D "$log_file.qemu" \
                -append "console=ttyAMA0 loglevel=8 bench=boot" \
                -s 2>&1 | grep -o 'boot_time=[0-9]*' | cut -d= -f2 || echo "0")
            ;;
        "riscv64")
            boot_time=$(qemu-system-riscv64 -m 512M -smp 1 \
                -machine spike \
                -kernel "target/$target/release/multios" \
                -nographic \
                -serial stdio \
                -d guest_errors \
                -D "$log_file.qemu" \
                -append "console=ttyS0 loglevel=8 bench=boot" \
                -s 2>&1 | grep -o 'boot_time=[0-9]*' | cut -d= -f2 || echo "0")
            ;;
    esac
    
    if [ -n "$boot_time" ] && [ "$boot_time" -gt 0 ]; then
        success "Boot time for $arch: ${boot_time}ms"
        echo "$arch:BOOT_TIME:$boot_time" >> "$ARTIFACTS_DIR/benchmark_results.txt"
        return 0
    else
        warning "Could not measure boot time for $arch"
        return 1
    fi
}

# Run memory benchmark
benchmark_memory() {
    local arch=$1
    local target="${arch}-unknown-none"
    local log_file="$LOG_DIR/memory_benchmark_${arch}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Running memory benchmark for $arch"
    
    # Run memory allocation benchmark
    case "$arch" in
        "x86_64")
            cargo bench --target "$target" --features bench memory_allocation 2>&1 | tee "$log_file"
            ;;
        "arm64"|"riscv64")
            cross bench --target "$target" --features bench memory_allocation 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Memory benchmark completed for $arch"
        
        # Extract memory usage metrics from benchmark output
        local memory_usage
        memory_usage=$(grep -o 'memory_usage=[0-9]*' "$log_file" | cut -d= -f2 | tail -1 || echo "0")
        
        if [ -n "$memory_usage" ] && [ "$memory_usage" -gt 0 ]; then
            echo "$arch:MEMORY_USAGE:${memory_usage}KB" >> "$ARTIFACTS_DIR/benchmark_results.txt"
            success "Memory usage for $arch: ${memory_usage}KB"
        fi
        
        return 0
    else
        error "Memory benchmark failed for $arch"
        return 1
    fi
}

# Run CPU benchmark
benchmark_cpu() {
    local arch=$1
    local target="${arch}-unknown-none"
    local log_file="$LOG_DIR/cpu_benchmark_${arch}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Running CPU benchmark for $arch"
    
    # Run CPU-intensive operations benchmark
    case "$arch" in
        "x86_64")
            cargo bench --target "$target" --features bench cpu_intensive 2>&1 | tee "$log_file"
            ;;
        "arm64"|"riscv64")
            cross bench --target "$target" --features bench cpu_intensive 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "CPU benchmark completed for $arch"
        
        # Extract CPU performance metrics
        local cpu_score
        cpu_score=$(grep -o 'cpu_score=[0-9]*' "$log_file" | cut -d= -f2 | tail -1 || echo "0")
        
        if [ -n "$cpu_score" ] && [ "$cpu_score" -gt 0 ]; then
            echo "$arch:CPU_SCORE:$cpu_score" >> "$ARTIFACTS_DIR/benchmark_results.txt"
            success "CPU score for $arch: $cpu_score"
        fi
        
        return 0
    else
        error "CPU benchmark failed for $arch"
        return 1
    fi
}

# Run I/O benchmark
benchmark_io() {
    local arch=$1
    local target="${arch}-unknown-none"
    local log_file="$LOG_DIR/io_benchmark_${arch}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Running I/O benchmark for $arch"
    
    # Run filesystem I/O benchmark
    case "$arch" in
        "x86_64")
            cargo bench --target "$target" --features bench filesystem_io 2>&1 | tee "$log_file"
            ;;
        "arm64"|"riscv64")
            cross bench --target "$target" --features bench filesystem_io 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "I/O benchmark completed for $arch"
        
        # Extract I/O performance metrics
        local io_throughput
        io_throughput=$(grep -o 'io_throughput=[0-9]*' "$log_file" | cut -d= -f2 | tail -1 || echo "0")
        
        if [ -n "$io_throughput" ] && [ "$io_throughput" -gt 0 ]; then
            echo "$arch:IO_THROUGHPUT:${io_throughput}MB/s" >> "$ARTIFACTS_DIR/benchmark_results.txt"
            success "I/O throughput for $arch: ${io_throughput}MB/s"
        fi
        
        return 0
    else
        error "I/O benchmark failed for $arch"
        return 1
    fi
}

# Run network benchmark
benchmark_network() {
    local arch=$1
    local target="${arch}-unknown-none"
    local log_file="$LOG_DIR/network_benchmark_${arch}_$(date +%Y%m%d_%H%M%S).log"
    
    log "Running network benchmark for $arch"
    
    # Run network stack benchmark
    case "$arch" in
        "x86_64")
            cargo bench --target "$target" --features bench network_stack 2>&1 | tee "$log_file"
            ;;
        "arm64"|"riscv64")
            cross bench --target "$target" --features bench network_stack 2>&1 | tee "$log_file"
            ;;
    esac
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        success "Network benchmark completed for $arch"
        
        # Extract network performance metrics
        local network_latency
        network_latency=$(grep -o 'network_latency=[0-9]*' "$log_file" | cut -d= -f2 | tail -1 || echo "0")
        
        if [ -n "$network_latency" ] && [ "$network_latency" -gt 0 ]; then
            echo "$arch:NETWORK_LATENCY:${network_latency}us" >> "$ARTIFACTS_DIR/benchmark_results.txt"
            success "Network latency for $arch: ${network_latency}us"
        fi
        
        return 0
    else
        error "Network benchmark failed for $arch"
        return 1
    fi
}

# Compare with baseline
compare_with_baseline() {
    local arch=$1
    local timestamp=$(date +%Y%m%d_%H%M%S)
    
    log "Comparing results with baseline for $arch"
    
    # Create benchmark result for this run
    local current_file="$BENCHMARK_DIR/benchmark_current_${arch}_${timestamp}.json"
    
    # Parse benchmark results
    local boot_time="0"
    local memory_usage="0"
    local cpu_score="0"
    local io_throughput="0"
    local network_latency="0"
    
    while IFS=: read -r metric_arch metric_name metric_value; do
        if [ "$metric_arch" = "$arch" ]; then
            case "$metric_name" in
                "BOOT_TIME") boot_time="${metric_value%ms}" ;;
                "MEMORY_USAGE") memory_usage="${metric_value%KB}" ;;
                "CPU_SCORE") cpu_score="$metric_value" ;;
                "IO_THROUGHPUT") io_throughput="${metric_value%MB/s}" ;;
                "NETWORK_LATENCY") network_latency="${metric_value%us}" ;;
            esac
        fi
    done < "$ARTIFACTS_DIR/benchmark_results.txt"
    
    # Generate current results JSON
    cat > "$current_file" << EOF
{
    "architecture": "$arch",
    "timestamp": "$(date -Iseconds)",
    "metrics": {
        "boot_time_ms": $boot_time,
        "memory_usage_kb": $memory_usage,
        "cpu_score": $cpu_score,
        "io_throughput_mbps": $io_throughput,
        "network_latency_us": $network_latency
    }
}
EOF
    
    # Compare with baseline if available
    if [ -f "$BASELINE_FILE" ]; then
        local baseline_file="$BENCHMARK_DIR/benchmark_baseline_${arch}.json"
        
        # Extract baseline values (simplified - in practice, you'd parse the JSON properly)
        local baseline_boot_time=$(grep -o "\"boot_time\":{\"$arch\":{\"target_ms\":\([0-9]*\)" "$BASELINE_FILE" | grep -o '[0-9]*' || echo "500")
        local baseline_memory=$(grep -o "\"memory_usage\":{\"$arch\":{\"target_mb\":\([0-9]*\)" "$BASELINE_FILE" | grep -o '[0-9]*' || echo "512")
        local baseline_cpu=$(grep -o "\"cpu_performance\":{\"$arch\":{\"target_score\":\([0-9]*\)" "$BASELINE_FILE" | grep -o '[0-9]*' || echo "100")
        
        # Calculate performance differences
        local boot_diff=0
        local memory_diff=0
        local cpu_diff=0
        
        if [ "$boot_time" -gt 0 ] && [ "$baseline_boot_time" -gt 0 ]; then
            boot_diff=$(( (boot_time - baseline_boot_time) * 100 / baseline_boot_time ))
        fi
        
        if [ "$memory_usage" -gt 0 ] && [ "$baseline_memory" -gt 0 ]; then
            memory_diff=$(( (memory_usage - baseline_memory) * 100 / baseline_memory ))
        fi
        
        if [ "$cpu_score" -gt 0 ] && [ "$baseline_cpu" -gt 0 ]; then
            cpu_diff=$(( (cpu_score - baseline_cpu) * 100 / baseline_cpu ))
        fi
        
        # Generate comparison report
        cat > "$BENCHMARK_DIR/benchmark_comparison_${arch}_${timestamp}.json" << EOF
{
    "architecture": "$arch",
    "timestamp": "$(date -Iseconds)",
    "current": {
        "boot_time_ms": $boot_time,
        "memory_usage_kb": $memory_usage,
        "cpu_score": $cpu_score,
        "io_throughput_mbps": $io_throughput,
        "network_latency_us": $network_latency
    },
    "baseline": {
        "boot_time_ms": $baseline_boot_time,
        "memory_usage_kb": $(( baseline_memory * 1024 )),
        "cpu_score": $baseline_cpu
    },
    "changes": {
        "boot_time_change_percent": $boot_diff,
        "memory_usage_change_percent": $memory_diff,
        "cpu_performance_change_percent": $cpu_diff
    },
    "regression_flags": {
        "boot_time_regression": $([ "$boot_diff" -gt 5 ] && echo "true" || echo "false"),
        "memory_regression": $([ "$memory_diff" -gt 10 ] && echo "true" || echo "false"),
        "cpu_regression": $([ "$cpu_diff" -lt -5 ] && echo "true" || echo "false")
    }
}
EOF
        
        success "Performance comparison generated for $arch"
    else
        warning "Baseline file not found - comparison skipped"
    fi
}

# Generate overall benchmark report
generate_overall_report() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local report_file="$BENCHMARK_DIR/benchmark_overall_${timestamp}.json"
    
    log "Generating overall benchmark report"
    
    # Collect all results
    local architectures=()
    local summary_stats=$(jq -n '{
        total_architectures: 0,
        successful_benchmarks: 0,
        failed_benchmarks: 0,
        avg_boot_time: 0,
        avg_memory_usage: 0,
        avg_cpu_score: 0
    }')
    
    local arch_results=()
    
    for arch in "${ARCHITECTURES[@]}"; do
        if [ -f "$BENCHMARK_DIR/benchmark_current_${arch}_${timestamp}.json" ]; then
            architectures+=("\"$arch\"")
            arch_results+=("$(cat "$BENCHMARK_DIR/benchmark_current_${arch}_${timestamp}.json")")
        fi
    done
    
    # Generate final report
    cat > "$report_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "report_version": "1.0",
    "architectures": [$(IFS=,; echo "${architectures[*]}")],
    "results": [
$(printf '        %s' "${arch_results[@]}" | paste -sd, -)
    ],
    "summary": {
        "total_architectures": ${#architectures[@]},
        "benchmark_status": "completed",
        "next_steps": "review individual architecture reports for details"
    }
}
EOF
    
    success "Overall benchmark report generated: $report_file"
    
    # Copy to latest for easy access
    cp "$report_file" "$BENCHMARK_DIR/benchmark_latest.json"
}

# Main benchmark execution
main() {
    local artifacts_dir=${1:-/workspace/artifacts}
    
    log "Starting MultiOS Performance Benchmark Suite"
    log "Artifacts directory: $artifacts_dir"
    
    # Initialize benchmark results file
    > "$ARTIFACTS_DIR/benchmark_results.txt"
    
    # Run benchmarks for each architecture
    local failed_benchmarks=()
    
    for arch in "${ARCHITECTURES[@]}"; do
        log "=== Benchmarking $arch ==="
        
        local arch_failed=false
        
        # Run all benchmark types
        if ! benchmark_boot_time "$arch"; then
            arch_failed=true
            failed_benchmarks+=("boot_$arch")
        fi
        
        if ! benchmark_memory "$arch"; then
            arch_failed=true
            failed_benchmarks+=("memory_$arch")
        fi
        
        if ! benchmark_cpu "$arch"; then
            arch_failed=true
            failed_benchmarks+=("cpu_$arch")
        fi
        
        if ! benchmark_io "$arch"; then
            arch_failed=true
            failed_benchmarks+=("io_$arch")
        fi
        
        if ! benchmark_network "$arch"; then
            arch_failed=true
            failed_benchmarks+=("network_$arch")
        fi
        
        # Compare with baseline
        if ! compare_with_baseline "$arch"; then
            warning "Baseline comparison failed for $arch"
        fi
        
        if [ "$arch_failed" = true ]; then
            error "Some benchmarks failed for $arch"
        else
            success "All benchmarks completed for $arch"
        fi
    done
    
    # Generate overall report
    generate_overall_report
    
    # Final status
    if [ ${#failed_benchmarks[@]} -eq 0 ]; then
        success "All performance benchmarks completed successfully!"
        exit 0
    else
        error "Some benchmarks failed: ${failed_benchmarks[*]}"
        exit 1
    fi
}

main "$@"