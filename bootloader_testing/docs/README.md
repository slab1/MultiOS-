# MultiOS Bootloader Testing Framework

A comprehensive testing framework for MultiOS bootloader supporting unit tests, integration tests, QEMU testing, performance benchmarking, and multi-architecture validation.

## Overview

The MultiOS Bootloader Testing Framework provides:

- **Unit Tests**: Component-level testing of bootloader modules
- **Integration Tests**: End-to-end testing with QEMU across multiple architectures
- **Performance Tests**: Boot time, memory usage, and benchmark testing
- **Validation Tools**: Automated boot validation and reporting
- **Logging System**: Structured logging with performance metrics
- **Multi-Architecture Support**: x86_64, ARM64 (AArch64), and RISC-V
- **CI/CD Integration**: Automated testing for continuous integration

## Architecture

```
bootloader_testing/
├── Cargo.toml              # Workspace configuration
├── unit/                   # Unit tests for bootloader components
├── integration/            # Integration tests with QEMU
├── validation/             # Boot validation tools and CLI
├── logging/                # Comprehensive logging system
├── scripts/                # Testing scripts and automation
├── docs/                   # Documentation
└── configs/                # Test configurations
```

## Quick Start

### 1. Setup Testing Environment

```bash
# Run the setup script
./scripts/setup_tests.sh --all

# Or step by step
./scripts/setup_tests.sh --check          # Check system requirements
./scripts/setup_tests.sh --install        # Install dependencies
./scripts/setup_tests.sh --setup          # Set up test environment
./scripts/setup_tests.sh --validate       # Validate installation
```

### 2. Run Tests

```bash
# Run all tests
./scripts/run_tests.sh --all

# Run specific test types
./scripts/run_tests.sh --unit             # Unit tests only
./scripts/run_tests.sh --qemu             # QEMU integration tests
./scripts/run_tests.sh --memory           # Memory management tests
./scripts/run_tests.sh --performance      # Performance benchmarks

# Run tests for specific architecture
./scripts/run_tests.sh --qemu --arch x86_64

# Run with custom options
./scripts/run_tests.sh --performance --iterations 10 --memory-size 2G
```

### 3. Use Validation Tool

```bash
# Run complete validation suite
cargo run --bin boot_validator -- validate \
    --bootloader-path target/release/multios-bootloader \
    --kernel-path target/release/multios-kernel

# Test specific boot mode
cargo run --bin boot_validator -- test-mode uefi \
    --bootloader-path target/release/multios-bootloader \
    --kernel-path target/release/multios-kernel

# Generate validation report
cargo run --bin boot_validator -- report \
    --results-dir validation_results \
    --output-file validation_report.html
```

## Detailed Documentation

### Unit Tests (`unit/`)

Unit tests provide component-level testing of bootloader functionality:

#### Bootloader Core Tests
- Boot mode detection
- Boot state management
- Memory mapping initialization
- Error handling and recovery
- Configuration validation
- Thread safety testing

#### Memory Management Tests
- Memory map creation and validation
- Memory allocation/deallocation
- Memory test functionality
- Memory boundary checking
- Memory leak detection

#### Kernel Loading Tests
- Kernel file format validation
- Boot parameter passing
- Kernel entry point verification
- Boot information structure validation

#### UEFI Tests
- UEFI system table access
- Boot services usage
- Memory map extraction
- Firmware interface testing

#### Legacy BIOS Tests
- BIOS interrupt handling
- Legacy boot process
- Compatibility testing

#### Performance Tests
- Boot sequence timing
- Memory access performance
- I/O operation benchmarking
- CPU utilization analysis

**Running Unit Tests:**
```bash
cd bootloader_testing/unit
cargo test
cargo test --release
cargo bench  # Run benchmarks
```

### Integration Tests (`integration/`)

Integration tests verify bootloader functionality in a virtualized environment:

#### QEMU Boot Tests
- Multi-architecture boot testing
- Boot sequence validation
- Console output verification
- Timeout handling
- Error recovery testing

#### Multi-Architecture Tests
- x86_64 testing with PC machine type
- ARM64 testing with virt machine type
- RISC-V testing with virt machine type
- Cross-architecture compatibility

#### Boot Sequence Tests
- UEFI boot sequence testing
- Legacy BIOS boot sequence testing
- Boot order validation
- Boot failure handling

#### Memory Integration Tests
- Real memory allocation testing
- Memory mapping validation
- Memory test execution
- Memory pressure testing

#### Performance Integration Tests
- End-to-end boot timing
- Resource usage monitoring
- System performance profiling
- Benchmark comparisons

**Running Integration Tests:**
```bash
cd bootloader_testing/integration
cargo test --features qemu
cargo test --features multi_arch
```

### Boot Validation Tool (`validation/`)

The validation tool provides comprehensive bootloader testing and reporting:

#### Command Line Interface
```bash
boot_validator [COMMAND] [OPTIONS]

Commands:
  validate       Run complete validation suite
  test-mode      Test specific boot mode
  test-memory    Test memory management
  test-performance Test performance
  report         Generate validation report

Options:
  -v, --verbose     Enable verbose logging
  -o, --output-dir  Output directory (default: ./validation_results)
  -a, --arch        Architecture (default: x86_64)
```

#### Validation Features
- Automated test execution
- Structured result reporting
- Performance metrics collection
- Multi-format output (JSON, HTML)
- Test result archiving
- Error analysis and reporting

#### Usage Examples
```bash
# Complete validation suite
boot_validator validate \
    --bootloader-path /path/to/bootloader \
    --kernel-path /path/to/kernel \
    --output-dir results/validation \
    --verbose

# Test specific boot mode with custom config
boot_validator test-mode uefi \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --config custom_config.json \
    --output-dir results/uefi_test

# Performance testing with multiple iterations
boot_validator test-performance \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --iterations 20 \
    --output-dir results/perf_test

# Generate report from existing results
boot_validator report \
    --results-dir results/validation \
    --output-file comprehensive_report.html
```

### Logging System (`logging/`)

Comprehensive logging infrastructure for test monitoring:

#### Features
- Structured logging with JSON format
- Real-time log streaming
- Performance metrics collection
- Session-based logging
- Thread-aware logging
- Configurable log levels
- Log rotation and archival
- Cross-platform support

#### API Usage
```rust
use bootloader_testing::logging::{BootloaderLogger, LogLevel, SessionStatus};

// Initialize logger
let logger = BootloaderLogger::default()?;

// Start test session
let session_id = logger.start_session(
    "UEFI Boot Test".to_string(),
    "x86_64".to_string()
);

// Log test events
logger.log(session_id, LogLevel::Info, "Boot sequence started", |map| {
    map.insert("boot_mode".to_string(), "uefi".to_string());
    map.insert("memory_size".to_string(), "1G".to_string());
});

// Log performance metrics
let metrics = PerformanceMetrics {
    boot_time_ms: 1500,
    memory_usage_kb: 256000,
    cpu_usage_percent: 45.2,
    disk_io_mb: 0,
    network_io_mb: 0,
};
logger.log_performance(session_id, metrics);

// End session
logger.end_session(session_id, SessionStatus::Completed);

// Export logs
logger.export_logs(session_id, "test_session.log")?;
```

#### Log Formats
- **Console Output**: Human-readable format with colors
- **Structured Logs**: JSON format for machine processing
- **Session Logs**: Per-session log files
- **Performance Logs**: Metrics-focused logging

### Testing Scripts (`scripts/`)

Automated testing and validation scripts:

#### Main Test Script (`run_tests.sh`)
Comprehensive test runner supporting all test types:

```bash
# Available options
--unit              Run unit tests
--qemu              Run QEMU integration tests  
--memory            Run memory management tests
--performance       Run performance tests
--all               Run all tests (default)
--arch ARCH         Target architecture (x86_64, aarch64, riscv64)
--memory-size SIZE  Memory size for tests (e.g., 512M, 1G)
--iterations NUM    Number of iterations for performance tests
--archive           Create archive of test results
--help              Show help message
```

#### Setup Script (`setup_tests.sh`)
Environment configuration and dependency installation:

```bash
# Setup options
--check         Check system requirements only
--install       Install all dependencies
--setup         Set up test environment only
--validate      Validate installation
--all           Run all setup steps (default)
--help          Show help message
```

#### Test Environment Scripts
Quick testing and monitoring utilities:

- `quick_test.sh` - Fast basic boot test
- `benchmark.sh` - Performance benchmarking
- `monitor.sh` - Test execution monitoring

### Configuration (`configs/`)

Test configuration files:

#### Global Configuration (`global.toml`)
```toml
[logging]
level = "INFO"
file = "${MULTIOS_TEST_DIR}/logs/test.log"
structured = true

[testing]
parallel = true
max_concurrent = 4
cleanup_after_tests = true

[reporting]
formats = ["html", "json"]
output_dir = "${MULTIOS_TEST_DIR}/results"
```

#### Architecture-Specific Configs
- `x86_64_test.toml` - x86_64 specific settings
- `aarch64_test.toml` - ARM64 specific settings  
- `riscv64_test.toml` - RISC-V specific settings

#### Custom Configurations
Create custom config files for specific test scenarios:

```toml
# custom_test.toml
[test]
name = "Custom Boot Test"
description = "Custom configuration for specific testing"

[boot]
boot_modes = ["uefi", "legacy"]
timeout = 45

[memory]
test_sizes = ["512M", "1G", "2G", "4G"]

[performance]
iterations = 10
warmup_iterations = 3

[qemu]
extra_args = ["-device", "virtio-rng-pci"]
network_mode = "none"
```

## Test Types

### 1. Unit Tests

**Purpose**: Test individual bootloader components in isolation

**Coverage**:
- Boot mode detection algorithms
- Memory management functions
- Kernel loading procedures
- Boot state management
- Error handling logic
- Configuration validation
- Serial console functionality

**Example**:
```bash
cd bootloader_testing/unit
cargo test bootloader_core_tests
cargo test memory_management_tests
cargo test kernel_loading_tests
```

### 2. Integration Tests

**Purpose**: Test bootloader in realistic environments using QEMU

**Coverage**:
- End-to-end boot sequences
- Multi-architecture compatibility
- Hardware abstraction testing
- Real memory allocation
- Boot parameter passing
- Console output verification
- Timeout and error handling

**Example**:
```bash
cd bootloader_testing/integration
cargo test qemu_boot_integration_tests
cargo test multi_arch_boot_tests
```

### 3. Performance Tests

**Purpose**: Measure and benchmark bootloader performance

**Metrics**:
- Boot time measurements
- Memory usage profiling
- CPU utilization analysis
- I/O performance
- Serial output rate

**Example**:
```bash
cd bootloader_testing
cargo bench boot_performance_benchmarks
./scripts/run_tests.sh --performance --arch x86_64 --iterations 20
```

### 4. Validation Tests

**Purpose**: Automated validation of bootloader compliance

**Checks**:
- Boot sequence correctness
- Memory map validity
- Kernel loading success
- Console output verification
- Error recovery capability
- Performance benchmarks

**Example**:
```bash
cargo run --bin boot_validator -- validate \
    --bootloader-path target/release/multios-bootloader \
    --kernel-path target/release/multios-kernel
```

## Multi-Architecture Support

### Supported Architectures

#### x86_64
- **QEMU Machine**: PC
- **Memory**: 512MB - 4GB
- **Boot Modes**: UEFI, Legacy BIOS
- **Features**: Full feature support
- **Use Case**: Primary development and testing

#### ARM64 (AArch64)
- **QEMU Machine**: Virt
- **Memory**: 1GB - 8GB
- **Boot Modes**: UEFI
- **Features**: ARM-specific optimizations
- **Use Case**: ARM server/embedded testing

#### RISC-V
- **QEMU Machine**: Virt
- **Memory**: 1GB - 8GB
- **Boot Modes**: UEFI
- **Features**: RISC-V ISA extensions
- **Use Case**: Research and development

### Architecture-Specific Testing

Each architecture has dedicated test configurations and validation procedures:

```bash
# x86_64 testing
./scripts/run_tests.sh --arch x86_64 --all

# ARM64 testing  
./scripts/run_tests.sh --arch aarch64 --all

# RISC-V testing
./scripts/run_tests.sh --arch riscv64 --all

# Multi-architecture testing
./scripts/run_tests.sh --all
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Bootloader Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        arch: [x86_64, aarch64, riscv64]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install dependencies
      run: ./scripts/setup_tests.sh --install
    
    - name: Setup test environment
      run: ./scripts/setup_tests.sh --setup
    
    - name: Build bootloader
      run: cargo build --release
    
    - name: Run unit tests
      run: cargo test --release
    
    - name: Run integration tests
      run: |
        cd bootloader_testing
        cargo test --features qemu
    
    - name: Run architecture-specific tests
      run: ./scripts/run_tests.sh --arch ${{ matrix.arch }} --all
    
    - name: Generate test report
      run: |
        ./scripts/run_tests.sh --all --archive
        mv test_results test_results_${{ matrix.arch }}
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      with:
        name: test-results-${{ matrix.arch }}
        path: test_results_*/
```

### Jenkins Pipeline Example

```groovy
pipeline {
    agent any
    
    options {
        timeout(time: 2, unit: 'HOURS')
        skipStagesAfterUnstable()
    }
    
    stages {
        stage('Setup') {
            steps {
                sh './scripts/setup_tests.sh --all'
                sh 'source test_environment/env.sh'
            }
        }
        
        stage('Build') {
            steps {
                sh 'cargo build --release'
                sh 'cd bootloader_testing && cargo build --release'
            }
        }
        
        stage('Unit Tests') {
            steps {
                sh 'cargo test --release'
                sh 'cd bootloader_testing/unit && cargo test --release'
            }
        }
        
        stage('Integration Tests') {
            parallel {
                stage('x86_64') {
                    steps {
                        sh './scripts/run_tests.sh --arch x86_64 --all'
                    }
                }
                stage('ARM64') {
                    steps {
                        sh './scripts/run_tests.sh --arch aarch64 --all'
                    }
                }
                stage('RISC-V') {
                    steps {
                        sh './scripts/run_tests.sh --arch riscv64 --all'
                    }
                }
            }
        }
        
        stage('Performance Tests') {
            steps {
                sh './scripts/run_tests.sh --performance --iterations 10'
            }
        }
        
        stage('Validation') {
            steps {
                sh '''
                    cargo run --bin boot_validator -- validate \
                        --bootloader-path target/release/multios-bootloader \
                        --kernel-path target/release/multios-kernel
                '''
            }
        }
    }
    
    post {
        always {
            archiveArtifacts artifacts: 'test_results/**/*', fingerprint: true
            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: 'test_results',
                reportFiles: '*.html',
                reportName: 'Bootloader Test Report'
            ])
        }
    }
}
```

## Performance Benchmarking

### Benchmark Categories

#### Boot Performance
- **Cold Boot Time**: Full system initialization time
- **Warm Boot Time**: Cache-warm boot time  
- **Boot Sequence Time**: Individual boot phase timing
- **Memory Initialization Time**: Memory setup duration

#### Memory Performance
- **Memory Allocation Time**: Allocation/deallocation speed
- **Memory Access Time**: Read/write performance
- **Memory Test Duration**: Memory validation time
- **Memory Usage**: Peak and average memory consumption

#### I/O Performance
- **Disk Read Time**: Kernel and config file loading
- **Console Output Rate**: Serial console throughput
- **Network I/O**: Network boot performance (if applicable)

### Benchmark Execution

```bash
# Run all benchmarks
cargo bench --all

# Run specific benchmarks
cargo bench boot_time_benchmarks
cargo bench memory_allocation_benchmarks
cargo bench bootloader_startup_benchmarks

# Performance testing with detailed metrics
./scripts/run_tests.sh --performance --iterations 20 --verbose

# Custom performance test
cargo run --bin boot_validator -- test-performance \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --iterations 50 \
    --output-dir perf_results
```

### Benchmark Results

Results are automatically saved in multiple formats:

- **Console Output**: Real-time benchmark results
- **JSON Files**: Machine-readable benchmark data
- **HTML Reports**: Visual benchmark comparisons
- **CSV Files**: Spreadsheet-compatible data
- **Log Files**: Detailed execution logs

## Troubleshooting

### Common Issues

#### QEMU Not Found
```bash
# Check QEMU installation
which qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64

# Install QEMU (Ubuntu/Debian)
sudo apt-get install qemu-system-x86 qemu-system-arm qemu-system-riscv64

# Install QEMU (Fedora/RHEL)
sudo dnf install qemu-system-x86 qemu-system-arm qemu-system-riscv64

# Install QEMU (Arch Linux)
sudo pacman -S qemu qemu-arch-extra
```

#### Permission Issues
```bash
# Make scripts executable
chmod +x scripts/*.sh

# Add user to kvm group (for KVM acceleration)
sudo usermod -a -G kvm $USER
# Log out and back in for group changes to take effect

# Check KVM availability
ls -la /dev/kvm
```

#### Rust Toolchain Issues
```bash
# Update Rust toolchain
rustup update

# Install missing targets
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf

# Install testing tools
cargo install cargo-watch cargo-audit
```

#### Memory Issues
```bash
# Check available memory
free -h
vm_stat (macOS)

# Increase swap space if needed
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

#### Build Failures
```bash
# Clean build artifacts
cargo clean
rm -rf target/debug/deps/*bootloader*

# Update dependencies
cargo update

# Check for missing system dependencies
./scripts/setup_tests.sh --check

# Verify environment setup
source test_environment/env.sh
echo $MULTIOS_TEST_DIR
```

### Debug Mode

Enable verbose debugging for detailed test information:

```bash
# Enable verbose logging
export RUST_LOG=debug
export MULTIOS_DEBUG=1

# Run tests with debug output
./scripts/run_tests.sh --all --verbose

# Validate with debug info
cargo run --bin boot_validator -- validate \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --verbose
```

### Log Analysis

Detailed logs are automatically generated:

```bash
# Check main test log
cat test_results/bootloader_test_*.log

# View structured logs
cat test_environment/logs/structured.log | jq '.'

# Monitor test execution
tail -f test_environment/logs/test.log

# Analyze performance logs
cat test_environment/logs/performance_*.json | jq '.'
```

### Test Failure Analysis

When tests fail, check:

1. **Log Files**: Detailed error information in test logs
2. **Console Output**: QEMU console logs in `test_results/console_*.log`
3. **System Resources**: Available memory and disk space
4. **Dependencies**: Verify all required tools are installed
5. **Permissions**: Check file and KVM permissions
6. **Architecture**: Confirm target architecture support

## Development Guide

### Adding New Tests

#### Unit Tests
1. Create test file in `unit/src/`
2. Add test configuration to `unit/Cargo.toml`
3. Implement test functions with appropriate attributes
4. Add test to CI/CD pipeline

#### Integration Tests  
1. Create test file in `integration/src/`
2. Add QEMU configuration for required architecture
3. Implement test with async/await patterns
4. Add test to integration test suite

#### Validation Tests
1. Extend validation tool in `validation/src/main.rs`
2. Add new command options and parameters
3. Implement test logic with proper error handling
4. Update documentation

### Test Configuration

#### Custom Test Configurations
Create test-specific configuration files:

```toml
# stress_test.toml
[test]
name = "Stress Test"
timeout = 300

[memory]
test_sizes = ["1G", "2G", "4G", "8G"]

[performance]
iterations = 100
warmup_iterations = 10

[qemu]
kvm = true
debug = true
```

#### Environment Variables
```bash
export MULTIOS_TEST_DIR=/custom/test/path
export MULTIOS_LOG_LEVEL=DEBUG
export MULTIOS_TIMEOUT=60
export MULTIOS_PARALLEL_TESTS=8
export MULTIOS_ARCH=x86_64
```

### Best Practices

#### Test Design
- **Isolated Tests**: Each test should be independent
- **Descriptive Names**: Use clear, descriptive test names
- **Proper Assertions**: Use specific assertions with meaningful messages
- **Resource Cleanup**: Ensure proper cleanup of resources
- **Error Handling**: Handle errors gracefully with informative messages

#### Performance Testing
- **Warmup Iterations**: Run warmup iterations before measurement
- **Statistical Analysis**: Use mean, median, min, max for results
- **Consistent Environment**: Ensure consistent testing environment
- **Multiple Runs**: Run tests multiple times for statistical significance

#### Integration Testing
- **Timeout Management**: Set appropriate timeouts for all tests
- **Architecture Coverage**: Test on all supported architectures
- **Error Scenarios**: Test both success and failure cases
- **Resource Monitoring**: Monitor system resources during testing

## API Reference

### Logger API

```rust
// Initialize logger
let logger = BootloaderLogger::new(config)?;

// Start test session
let session_id = logger.start_session(name, architecture);

// Log events
logger.log(session_id, LogLevel::Info, "message", |map| {
    map.insert("key".to_string(), "value".to_string());
});

// Log performance metrics
logger.log_performance(session_id, metrics);

// End session
logger.end_session(session_id, SessionStatus::Completed);

// Export logs
logger.export_logs(session_id, path)?;
```

### Validation Tool API

```rust
// Command line interface
use bootloader_testing::validation::{Cli, Commands};

// CLI commands
Commands::Validate { bootloader_path, kernel_path, config }
Commands::TestMode { mode, bootloader_path, kernel_path }
Commands::TestMemory { bootloader_path, kernel_path, memory_size }
Commands::TestPerformance { bootloader_path, kernel_path, iterations }
Commands::Report { results_dir, output_file }
```

## Contributing

### Development Setup

1. Clone the repository
2. Run `./scripts/setup_tests.sh --all`
3. Activate test environment: `source test_environment/env.sh`
4. Run tests to verify setup: `./scripts/run_tests.sh --all`

### Test Development Workflow

1. Create feature branch
2. Write tests for new functionality
3. Run tests to verify functionality
4. Update documentation
5. Submit pull request

### Code Quality

- Follow Rust coding standards
- Add tests for all new functionality
- Maintain test coverage above 80%
- Document all public APIs
- Run `cargo fmt` and `cargo clippy` before commits

## License

This testing framework is provided under the MIT OR Apache-2.0 License.

## Support

For questions, issues, or contributions:

- **GitHub Issues**: https://github.com/multios/bootloader/issues
- **Documentation**: https://github.com/multios/bootloader/wiki
- **Discussions**: https://github.com/multios/bootloader/discussions
- **Email**: multios-dev@example.com

## Changelog

### Version 0.1.0
- Initial release
- Unit test framework
- Integration test suite
- QEMU multi-architecture support
- Validation tool
- Comprehensive logging system
- Performance benchmarking
- CI/CD integration examples
- Complete documentation
