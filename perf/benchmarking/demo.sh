#!/bin/bash

# MultiOS Comprehensive Benchmarking Framework Demo Script
# This script demonstrates the usage and capabilities of the benchmarking framework

echo "======================================================"
echo "MultiOS Comprehensive Benchmarking Framework Demo"
echo "======================================================"
echo ""

# Function to display usage information
show_usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Available commands:"
    echo "  build       - Build the benchmarking framework"
    echo "  run         - Run all benchmarks"
    echo "  run-cpu     - Run CPU benchmarks only"
    echo "  run-memory  - Run memory benchmarks only"
    echo "  test        - Run unit tests"
    echo "  examples    - Run example benchmarks"
    echo "  help        - Show this help message"
    echo "  setup       - Setup development environment"
    echo ""
}

# Function to build the framework
build_framework() {
    echo "Building MultiOS Benchmarking Framework..."
    echo "=========================================="
    
    # Check if Rust is installed
    if ! command -v rustc &> /dev/null; then
        echo "❌ Rust not found. Please install Rust first:"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "   source ~/.cargo/env"
        return 1
    fi
    
    echo "✅ Rust found: $(rustc --version)"
    echo "✅ Cargo found: $(cargo --version)"
    echo ""
    
    # Build release version
    echo "Building release version..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        echo "✅ Build successful!"
        echo "📦 Binary location: ./target/release/multios-benchmark"
    else
        echo "❌ Build failed!"
        return 1
    fi
    echo ""
}

# Function to run benchmarks
run_benchmarks() {
    echo "Running Comprehensive Benchmarks..."
    echo "==================================="
    
    if [ ! -f "./target/release/multios-benchmark" ]; then
        echo "❌ Binary not found. Run 'build' first."
        return 1
    fi
    
    echo "Running with 1000 iterations for demo..."
    echo ""
    
    # Run all benchmarks
    ./target/release/multios-benchmark run --category all --iterations 1000 --verbose --output demo_results.json --format json
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ Benchmarks completed successfully!"
        echo "📊 Results saved to: demo_results.json"
    else
        echo "❌ Benchmark execution failed!"
        return 1
    fi
}

# Function to run CPU benchmarks only
run_cpu_benchmarks() {
    echo "Running CPU Benchmarks..."
    echo "========================"
    
    if [ ! -f "./target/release/multios-benchmark" ]; then
        echo "❌ Binary not found. Run 'build' first."
        return 1
    fi
    
    echo "Running CPU benchmarks with 5000 iterations..."
    echo ""
    
    ./target/release/multios-benchmark run --category cpu --iterations 5000 --verbose
    
    if [ $? -eq 0 ]; then
        echo "✅ CPU benchmarks completed!"
    else
        echo "❌ CPU benchmark execution failed!"
        return 1
    fi
}

# Function to run memory benchmarks
run_memory_benchmarks() {
    echo "Running Memory Benchmarks..."
    echo "============================"
    
    if [ ! -f "./target/release/multios-benchmark" ]; then
        echo "❌ Binary not found. Run 'build' first."
        return 1
    fi
    
    echo "Running memory benchmarks with 2000 iterations..."
    echo ""
    
    ./target/release/multios-benchmark run --category memory --iterations 2000 --verbose
    
    if [ $? -eq 0 ]; then
        echo "✅ Memory benchmarks completed!"
    else
        echo "❌ Memory benchmark execution failed!"
        return 1
    fi
}

# Function to run unit tests
run_tests() {
    echo "Running Unit Tests..."
    echo "===================="
    
    if ! command -v cargo &> /dev/null; then
        echo "❌ Cargo not found. Please install Rust first."
        return 1
    fi
    
    echo "Running cargo test..."
    echo ""
    
    cargo test --verbose
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ All tests passed!"
    else
        echo ""
        echo "❌ Some tests failed!"
        return 1
    fi
}

# Function to run examples
run_examples() {
    echo "Running Example Benchmarks..."
    echo "============================="
    
    if [ ! -f "./target/release/multios-benchmark" ]; then
        echo "❌ Binary not found. Run 'build' first."
        return 1
    fi
    
    echo "Running custom benchmark examples..."
    echo ""
    
    # List available benchmarks
    echo "Available benchmarks:"
    ./target/release/multios-benchmark list
    echo ""
    
    echo "Examples available:"
    echo "  - CPU benchmark example: examples/cpu_benchmark_example.rs"
    echo "  - Memory benchmark example: examples/memory_benchmark_example.rs"
    echo ""
    echo "To run examples:"
    echo "  cargo run --example cpu_benchmark_example"
    echo "  cargo run --example memory_benchmark_example"
}

# Function to setup development environment
setup_environment() {
    echo "Setting up Development Environment..."
    echo "====================================="
    
    # Check if Rust is installed
    if ! command -v rustc &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
    
    echo "✅ Rust installation verified"
    
    # Install additional tools
    echo "Installing additional development tools..."
    
    if command -v cargo &> /dev/null; then
        cargo install cargo-watch
        cargo install cargo-expand
        cargo install cargo-tarpaulin
        
        if [ $? -eq 0 ]; then
            echo "✅ Development tools installed"
        else
            echo "⚠️  Some tools failed to install"
        fi
    fi
    
    # Install system dependencies
    echo ""
    echo "Installing system dependencies..."
    
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y build-essential perf-tools-unstable
        echo "✅ System dependencies installed (Debian/Ubuntu)"
    elif command -v yum &> /dev/null; then
        sudo yum install -y perf
        echo "✅ System dependencies installed (RedHat/CentOS)"
    elif command -v brew &> /dev/null; then
        brew install binutils
        echo "✅ System dependencies installed (macOS)"
    else
        echo "⚠️  Please install build tools and perf manually"
    fi
    
    echo ""
    echo "✅ Development environment setup complete!"
}

# Function to show framework capabilities
show_capabilities() {
    echo "MultiOS Benchmarking Framework Capabilities"
    echo "==========================================="
    echo ""
    echo "📊 Benchmark Categories:"
    echo "   • CPU Performance (5 benchmarks)"
    echo "     - Integer operations, floating-point ops"
    echo "     - Matrix multiplication, cryptography"
    echo "     - SIMD operations"
    echo ""
    echo "   • Memory Performance (6 benchmarks)"
    echo "     - Sequential/random access patterns"
    echo "     - Cache performance analysis"
    echo "     - Allocation speed and bandwidth"
    echo ""
    echo "   • File System I/O (6 benchmarks)"
    echo "     - Sequential/random file operations"
    echo "     - Metadata and directory operations"
    echo "     - Small file workload simulation"
    echo ""
    echo "   • Network Performance (5 benchmarks)"
    echo "     - TCP/UDP throughput and latency"
    echo "     - Socket creation overhead"
    echo "     - Protocol analysis"
    echo ""
    echo "   • Boot Time Analysis (3 benchmarks)"
    echo "     - Boot sequence timing"
    echo "     - Component initialization"
    echo "     - Optimization opportunities"
    echo ""
    echo "   • System Call Performance (5 benchmarks)"
    echo "     - Basic syscalls overhead"
    echo "     - Process/thread creation"
    echo "     - IPC operations"
    echo ""
    echo "📈 Reporting Features:"
    echo "   • Multiple output formats (JSON, CSV, HTML, Human)"
    echo "   • Statistical analysis (mean, median, percentiles)"
    echo "   • Performance comparison with baselines"
    echo "   • Interactive HTML reports"
    echo "   • System information collection"
    echo ""
    echo "🛠️  Advanced Features:"
    echo "   • Cross-platform compatibility"
    echo "   • Extensible architecture"
    echo "   • Real-time progress tracking"
    echo "   • Automated CI/CD integration"
    echo "   • Comprehensive documentation"
    echo ""
}

# Function to show project structure
show_structure() {
    echo "Project Structure"
    echo "================="
    echo ""
    echo "perf/benchmarking/"
    echo "├── src/"
    echo "│   ├── lib.rs              # Core framework"
    echo "│   ├── main.rs             # CLI interface"
    echo "│   ├── cpu.rs              # CPU benchmarks"
    echo "│   ├── memory.rs           # Memory tests"
    echo "│   ├── filesystem.rs       # File system tests"
    echo "│   ├── network.rs          # Network tests"
    echo "│   ├── boot_time.rs        # Boot analysis"
    echo "│   ├── syscalls.rs         # Syscall tests"
    echo "│   ├── utils.rs            # Utilities"
    echo "│   └── reporter.rs         # Report generation"
    echo "├── examples/"
    echo "│   ├── cpu_benchmark_example.rs"
    echo "│   └── memory_benchmark_example.rs"
    echo "├── Cargo.toml"
    echo "├── Makefile"
    echo "├── README.md"
    echo "└── IMPLEMENTATION_REPORT.md"
    echo ""
    echo "📁 Key Files:"
    echo "   • README.md - Comprehensive documentation"
    echo "   • Makefile - Build and run automation"
    echo "   • Cargo.toml - Dependencies and features"
    echo "   • IMPLEMENTATION_REPORT.md - Technical details"
}

# Main script logic
case "$1" in
    "build")
        build_framework
        ;;
    "run")
        run_benchmarks
        ;;
    "run-cpu")
        run_cpu_benchmarks
        ;;
    "run-memory")
        run_memory_benchmarks
        ;;
    "test")
        run_tests
        ;;
    "examples")
        run_examples
        ;;
    "setup")
        setup_environment
        ;;
    "capabilities")
        show_capabilities
        ;;
    "structure")
        show_structure
        ;;
    "help"|"--help"|"-h"|"")
        show_usage
        echo "Additional commands:"
        echo "  capabilities - Show framework capabilities"
        echo "  structure    - Show project structure"
        echo ""
        echo "Quick Start:"
        echo "  1. $0 setup          # Setup development environment"
        echo "  2. $0 build          # Build the framework"
        echo "  3. $0 run            # Run all benchmarks"
        echo ""
        echo "For more information, see README.md"
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac

echo ""
echo "======================================================"
echo "MultiOS Benchmarking Framework Demo Complete"
echo "======================================================"