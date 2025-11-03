#!/bin/bash

# MultiOS CPU Architecture Testing Framework - Example Usage Scripts
# This script demonstrates various testing scenarios and configurations

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored messages
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
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

# Check if binary exists
check_binary() {
    if [ ! -f "target/release/cpu_testing" ]; then
        error "Binary not found. Please run 'cargo build --release' first"
        exit 1
    fi
}

# Create results directory
mkdir -p examples/results

# Example 1: Basic Simulation
run_basic_simulation() {
    info "Running basic simulation on x86_64 and ARM64"
    
    ./target/release/cpu_testing simulate \
        --archs x86_64,arm64 \
        --output examples/results/basic_sim \
        --config configs/default.toml
    
    success "Basic simulation completed"
}

# Example 2: Comprehensive ISA Testing
run_isa_testing() {
    info "Running comprehensive ISA testing"
    
    ./target/release/cpu_testing test-isa \
        --archs x86_64,arm64,riscv64 \
        --suite comprehensive
    
    success "ISA testing completed"
}

# Example 3: Performance Benchmarking
run_performance_benchmarks() {
    info "Running performance benchmarks"
    
    ./target/release/cpu_testing benchmark \
        --archs x86_64,arm64,powerpc64 \
        --type arithmetic
    
    sleep 2
    ./target/release/cpu_testing benchmark \
        --archs x86_64,arm64,powerpc64 \
        --type memory
    
    sleep 2
    ./target/release/cpu_testing benchmark \
        --archs x86_64,arm64,powerpc64 \
        --type floating_point
    
    success "Performance benchmarking completed"
}

# Example 4: Memory Hierarchy Analysis
run_memory_tests() {
    info "Running memory hierarchy tests"
    
    ./target/release/cpu_testing memory \
        --archs x86_64,arm64,riscv64 \
        --type cache
    
    success "Memory hierarchy testing completed"
}

# Example 5: Pipeline Analysis
run_pipeline_analysis() {
    info "Running pipeline analysis"
    
    ./target/release/cpu_testing pipeline \
        --archs x86_64,arm64 \
        --type branch_prediction
    
    success "Pipeline analysis completed"
}

# Example 6: Architecture Comparison
run_architecture_comparison() {
    info "Running comprehensive architecture comparison"
    
    ./target/release/cpu_testing compare \
        --archs x86_64,arm64,riscv64,powerpc64 \
        --type comprehensive
    
    success "Architecture comparison completed"
}

# Example 7: Custom Configuration Generation
run_configuration_generation() {
    info "Generating custom processor configurations"
    
    ./target/release/cpu_testing configure --type standard
    ./target/release/cpu_testing configure --type high-performance
    ./target/release/cpu_testing configure --type energy-efficient
    ./target/release/cpu_testing configure --type ai-optimized
    
    success "Configuration generation completed"
}

# Example 8: Complete Test Suite
run_complete_test_suite() {
    info "Running complete test suite on all architectures"
    
    ./target/release/cpu_testing all \
        --archs x86_64,arm64,riscv64 \
        --output examples/results/complete_suite
    
    success "Complete test suite finished"
}

# Example 9: Single Architecture Deep Dive
run_deep_dive() {
    local arch=$1
    
    if [ -z "$arch" ]; then
        arch="x86_64"
    fi
    
    info "Running deep dive analysis on $arch"
    
    # ISA testing
    ./target/release/cpu_testing test-isa --archs $arch
    
    # Performance testing
    ./target/release/cpu_testing benchmark --archs $arch --type all
    
    # Memory testing
    ./target/release/cpu_testing memory --archs $arch --type all
    
    # Pipeline testing
    ./target/release/cpu_testing pipeline --archs $arch --type all
    
    success "Deep dive analysis of $arch completed"
}

# Example 10: Research Workflow
run_research_workflow() {
    info "Running research workflow for architecture comparison"
    
    # Step 1: Generate configurations
    ./target/release/cpu_testing configure --type standard
    ./target/release/cpu_testing configure --type high-performance
    ./target/release/cpu_testing configure --type energy-efficient
    
    # Step 2: Run comprehensive testing
    ./target/release/cpu_testing all \
        --archs x86_64,arm64 \
        --output examples/results/research
    
    # Step 3: Generate comparison report
    ./target/release/cpu_testing compare \
        --archs x86_64,arm64 \
        --type comprehensive
    
    success "Research workflow completed"
}

# Example 11: Educational Demo
run_educational_demo() {
    info "Running educational demonstration"
    
    info "Demonstrating ISA differences..."
    ./target/release/cpu_testing test-isa --archs x86_64,arm64,riscv64 --suite basic
    
    info "Demonstrating performance characteristics..."
    ./target/release/cpu_testing benchmark --archs x86_64,arm64 --type arithmetic
    
    info "Demonstrating memory hierarchy behavior..."
    ./target/release/cpu_testing memory --archs x86_64 --type cache
    
    info "Demonstrating pipeline characteristics..."
    ./target/release/cpu_testing pipeline --archs x86_64 --type pipeline
    
    success "Educational demonstration completed"
}

# Example 12: Performance Regression Testing
run_regression_testing() {
    info "Running performance regression testing"
    
    # Test baseline configuration
    ./target/release/cpu_testing benchmark --archs x86_64 --type arithmetic
    
    # Save results
    cp results/benchmarks.json examples/results/baseline_benchmarks.json
    
    warning "Baseline benchmarks saved. Modify source code and re-run to test for regressions."
}

# Main menu
show_menu() {
    echo ""
    echo "MultiOS CPU Architecture Testing Framework - Examples"
    echo "====================================================="
    echo "1.  Basic Simulation (x86_64, ARM64)"
    echo "2.  Comprehensive ISA Testing"
    echo "3.  Performance Benchmarking"
    echo "4.  Memory Hierarchy Analysis"
    echo "5.  Pipeline Analysis"
    echo "6.  Architecture Comparison"
    echo "7.  Configuration Generation"
    echo "8.  Complete Test Suite"
    echo "9.  Deep Dive Analysis (default: x86_64)"
    echo "10. Research Workflow"
    echo "11. Educational Demo"
    echo "12. Performance Regression Testing"
    echo "13. Run All Examples"
    echo "0.  Exit"
    echo ""
}

# Run selected example
run_example() {
    case $1 in
        1) run_basic_simulation ;;
        2) run_isa_testing ;;
        3) run_performance_benchmarks ;;
        4) run_memory_tests ;;
        5) run_pipeline_analysis ;;
        6) run_architecture_comparison ;;
        7) run_configuration_generation ;;
        8) run_complete_test_suite ;;
        9) run_deep_dive $2 ;;
        10) run_research_workflow ;;
        11) run_educational_demo ;;
        12) run_regression_testing ;;
        13)
            info "Running all examples..."
            run_basic_simulation
            run_isa_testing
            run_performance_benchmarks
            run_memory_tests
            run_pipeline_analysis
            run_architecture_comparison
            run_configuration_generation
            success "All examples completed!"
            ;;
        *) error "Invalid option" ;;
    esac
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --arch)
            ARCH="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [option] [--arch architecture]"
            echo ""
            echo "Options:"
            echo "  1-13    Run specific example"
            echo "  --arch  Specify architecture for deep dive (default: x86_64)"
            echo "  --help  Show this help message"
            exit 0
            ;;
        *)
            OPTION="$1"
            shift
            ;;
    esac
done

# Check binary and run
check_binary

if [ -n "$OPTION" ]; then
    if [ "$OPTION" = "9" ]; then
        run_example 9 "$ARCH"
    else
        run_example "$OPTION"
    fi
else
    # Interactive mode
    while true; do
        show_menu
        read -p "Select an option: " choice
        case $choice in
            0)
                info "Goodbye!"
                break
                ;;
            9)
                read -p "Architecture for deep dive (default: x86_64): " arch
                run_example 9 "$arch"
                ;;
            *)
                run_example "$choice"
                ;;
        esac
        echo ""
        read -p "Press Enter to continue..."
    done
fi