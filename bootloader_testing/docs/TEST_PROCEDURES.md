# Bootloader Testing Framework - Test Procedures

## Overview

This document provides step-by-step procedures for testing the MultiOS bootloader across multiple architectures and environments.

## Pre-Test Requirements

### System Requirements
- Minimum 4GB RAM (8GB recommended)
- Minimum 10GB disk space
- QEMU with multi-architecture support
- Rust toolchain (latest stable)
- Git for version control

### Prerequisites Installation

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    qemu-system-x86 \
    qemu-system-arm \
    qemu-system-riscv64 \
    qemu-utils \
    ovmf \
    gcc-aarch64-linux-gnu \
    gcc-riscv64-linux-gnu \
    gdb-multiarch

# Fedora/RHEL
sudo dnf install -y \
    gcc \
    qemu-system-x86 \
    qemu-system-aarch64 \
    qemu-system-riscv64 \
    qemu-img \
    edk2-ovmf \
    gcc-aarch64-linux-gnu \
    gcc-riscv64-linux-gnu \
    gdb \
    make \
    python3

# Arch Linux
sudo pacman -Sy --needed \
    base-devel \
    qemu \
    qemu-arch-extra \
    edk2-ovmf \
    aarch64-linux-gnu-gcc \
    riscv64-linux-gnu-gcc \
    gdb \
    make
```

### Environment Setup

```bash
# Clone repository
git clone <repository-url>
cd multios-bootloader

# Run setup script
./scripts/setup_tests.sh --all

# Activate test environment
source test_environment/env.sh

# Verify installation
make validate-install
```

## Unit Testing Procedures

### Basic Unit Test Execution

```bash
# Navigate to unit test directory
cd bootloader_testing/unit

# Run all unit tests
cargo test

# Run specific test categories
cargo test bootloader_core_tests
cargo test memory_management_tests
cargo test kernel_loading_tests
cargo test uefi_boot_tests
cargo test legacy_bios_tests

# Run tests with specific features
cargo test --features uefi
cargo test --features legacy
cargo test --features performance

# Run tests with output capture
cargo test -- --nocapture
cargo test -- --test-threads=1  # Serial execution
```

### Performance Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark categories
cargo bench boot_time_benchmarks
cargo bench memory_allocation_benchmarks
cargo bench bootloader_startup_benchmarks

# Run benchmarks with detailed output
cargo bench -- --verbose

# Generate benchmark reports
cargo bench > benchmark_results.txt
```

### Custom Unit Test Development

1. **Create Test Module**
```rust
// unit/src/custom_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_functionality() {
        // Test implementation
        assert!(true);
    }
}
```

2. **Add Test to Cargo.toml**
```toml
[[test]]
name = "custom_test"
harness = false
```

3. **Register in Unit Test Suite**
```bash
cargo test custom_test
```

## Integration Testing Procedures

### QEMU Integration Tests

```bash
# Navigate to integration directory
cd bootloader_testing/integration

# Run all integration tests
cargo test --features qemu

# Run specific integration tests
cargo test qemu_boot_integration_tests
cargo test multi_arch_boot_tests
cargo test boot_sequence_integration_tests
cargo test memory_integration_tests

# Run with specific features
cargo test --features qemu,multi_arch
cargo test --features uefi
cargo test --features performance
```

### Manual QEMU Testing

#### x86_64 Testing

```bash
# Create test kernel
dd if=/dev/zero of=test_kernel.bin bs=1024 count=100

# Basic x86_64 boot test
qemu-system-x86_64 \
    -m 512M \
    -nographic \
    -kernel test_kernel.bin \
    -append "console=ttyS0 quiet" \
    -serial file:x86_64_console.log

# UEFI x86_64 boot test
qemu-system-x86_64 \
    -m 1G \
    -machine type=pc \
    -cpu qemu64 \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd \
    -kernel test_kernel.bin \
    -append "uefi console=ttyS0" \
    -serial file:uefi_x86_64_console.log
```

#### ARM64 Testing

```bash
# ARM64 boot test
qemu-system-aarch64 \
    -m 1G \
    -cpu cortex-a57 \
    -machine type=virt \
    -kernel test_kernel.bin \
    -append "console=ttyAMA0" \
    -nographic \
    -device pl011,chardev=serial0 \
    -chardev stdio,id=serial0 \
    -serial file:arm64_console.log
```

#### RISC-V Testing

```bash
# RISC-V boot test
qemu-system-riscv64 \
    -m 1G \
    -cpu rv64gc \
    -machine type=virt \
    -kernel test_kernel.bin \
    -append "console=ttyS0" \
    -nographic \
    -serial file:riscv64_console.log
```

### Automated Integration Testing

```bash
# Run all integration tests via script
./scripts/run_tests.sh --integration --all

# Test specific architecture
./scripts/run_tests.sh --integration --arch x86_64
./scripts/run_tests.sh --integration --arch aarch64
./scripts/run_tests.sh --integration --arch riscv64

# Test specific boot modes
./scripts/run_tests.sh --integration --boot-mode uefi
./scripts/run_tests.sh --integration --boot-mode legacy
```

## Performance Testing Procedures

### Boot Time Testing

```bash
# Basic boot time test
time qemu-system-x86_64 \
    -m 512M \
    -nographic \
    -kernel test_kernel.bin \
    -append "console=ttyS0" \
    -serial file:boot_time_test.log

# Automated boot time testing
./scripts/run_tests.sh --performance --iterations 10

# Custom performance test
cargo run --bin boot_validator -- test-performance \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --iterations 20 \
    --output-dir perf_results
```

### Memory Performance Testing

```bash
# Memory allocation test
qemu-system-x86_64 \
    -m 4G \
    -nographic \
    -kernel test_kernel.bin \
    -append "memory_test=true console=ttyS0" \
    -serial file:memory_test.log

# Memory stress test
./scripts/run_tests.sh --memory --memory-size 4G

# Custom memory test
cargo run --bin boot_validator -- test-memory \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --memory-size 2G
```

### CPU Performance Testing

```bash
# CPU utilization test
qemu-system-x86_64 \
    -m 1G \
    -smp 4 \
    -nographic \
    -kernel test_kernel.bin \
    -append "cpu_test=true console=ttyS0" \
    -serial file:cpu_test.log

# Multi-core performance test
./scripts/run_tests.sh --performance --cpus 4 --iterations 5
```

## Validation Testing Procedures

### Complete Validation Suite

```bash
# Run complete validation
cargo run --bin boot_validator -- validate \
    --bootloader-path target/release/multios-bootloader \
    --kernel-path target/release/multios-kernel \
    --output-dir validation_results \
    --verbose

# Validate with custom configuration
cargo run --bin boot_validator -- validate \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --config custom_config.toml \
    --output-dir custom_validation
```

### Specific Mode Validation

```bash
# UEFI validation
cargo run --bin boot_validator -- test-mode uefi \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --output-dir uefi_validation

# Legacy BIOS validation
cargo run --bin boot_validator -- test-mode legacy \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --output-dir legacy_validation

# Memory validation
cargo run --bin boot_validator -- test-memory \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --memory-size 1G
```

### Performance Validation

```bash
# Performance validation with multiple iterations
cargo run --bin boot_validator -- test-performance \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --iterations 50 \
    --output-dir perf_validation

# Compare performance across architectures
for arch in x86_64 aarch64 riscv64; do
    cargo run --bin boot_validator -- test-performance \
        --arch $arch \
        --bootloader-path boot_$arch.bin \
        --kernel-path kernel_$arch.bin \
        --iterations 10 \
        --output-dir perf_$arch
done
```

## Multi-Architecture Testing Procedures

### Sequential Architecture Testing

```bash
# Test all architectures sequentially
./scripts/run_tests.sh --all --parallel false

# Test architectures individually
./scripts/run_tests.sh --arch x86_64 --all
./scripts/run_tests.sh --arch aarch64 --all
./scripts/run_tests.sh --arch riscv64 --all
```

### Parallel Architecture Testing

```bash
# Test architectures in parallel (make)
make test-x86 &
make test-arm &
make test-riscv &
wait

# Test with custom parallel settings
export MULTIOS_MAX_CONCURRENT=3
./scripts/run_tests.sh --all --parallel true
```

### Architecture Comparison Testing

```bash
# Create architecture comparison script
cat > compare_archs.sh << 'EOF'
#!/bin/bash
for arch in x86_64 aarch64 riscv64; do
    echo "Testing $arch..."
    cargo run --bin boot_validator -- test-performance \
        --arch $arch \
        --bootloader-path boot_$arch.bin \
        --kernel-path kernel_$arch.bin \
        --iterations 10 \
        --output-dir results_$arch
done

# Generate comparison report
cargo run --bin boot_validator -- report \
    --results-dir results_x86_64,results_aarch64,results_riscv64 \
    --output-file arch_comparison.html
EOF

chmod +x compare_archs.sh
./compare_archs.sh
```

## Stress Testing Procedures

### Memory Stress Testing

```bash
# Memory stress test with multiple sizes
for size in 512M 1G 2G 4G; do
    echo "Testing with $size memory..."
    cargo run --bin boot_validator -- test-memory \
        --bootloader-path boot.bin \
        --kernel-path kernel.bin \
        --memory-size $size \
        --output-dir memory_stress_$size
done

# Continuous memory testing
./scripts/run_tests.sh --memory --iterations 100 --timeout 60
```

### CPU Stress Testing

```bash
# CPU stress test
for cpus in 1 2 4 8; do
    echo "Testing with $cpus CPUs..."
    qemu-system-x86_64 \
        -m 2G \
        -smp $cpus \
        -nographic \
        -kernel test_kernel.bin \
        -append "cpu_stress=true console=ttyS0" \
        -serial file:cpu_stress_${cpus}.log
done
```

### Long-Running Tests

```bash
# 24-hour stability test
timeout 86400 ./scripts/run_tests.sh --all --loop

# Extended performance testing
cargo run --bin boot_validator -- test-performance \
    --bootloader-path boot.bin \
    --kernel-path kernel.bin \
    --iterations 1000 \
    --output-dir extended_performance
```

## Regression Testing Procedures

### Baseline Creation

```bash
# Create performance baseline
cargo run --bin boot_validator -- test-performance \
    --bootloader-path boot_baseline.bin \
    --kernel-path kernel_baseline.bin \
    --iterations 50 \
    --output-dir baseline_results

# Save baseline configuration
cp baseline_results/test_summary.json baseline_config.json
```

### Regression Testing

```bash
# Compare against baseline
cargo run --bin boot_validator -- test-performance \
    --bootloader-path boot_current.bin \
    --kernel-path kernel_current.bin \
    --iterations 50 \
    --output-dir current_results

# Generate regression report
cargo run --bin boot_validator -- compare \
    --baseline baseline_config.json \
    --current current_results/ \
    --output-file regression_report.html
```

### Automated Regression Testing

```bash
# Create regression test script
cat > regression_test.sh << 'EOF'
#!/bin/bash
BASELINE_COMMIT="HEAD~1"
CURRENT_COMMIT="HEAD"

echo "Testing baseline commit: $BASELINE_COMMIT"
git checkout $BASELINE_COMMIT
make build
cargo run --bin boot_validator -- validate \
    --output-dir baseline_regression

echo "Testing current commit: $CURRENT_COMMIT"
git checkout $CURRENT_COMMIT
make build
cargo run --bin boot_validator -- validate \
    --output-dir current_regression

# Compare results
cargo run --bin boot_validator -- compare \
    --baseline baseline_regression/test_summary.json \
    --current current_regression/test_summary.json \
    --output-file regression_comparison.html

git checkout $CURRENT_COMMIT
EOF

chmod +x regression_test.sh
./regression_test.sh
```

## CI/CD Integration Procedures

### GitHub Actions Setup

```yaml
# .github/workflows/bootloader-tests.yml
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
    
    - name: Setup Environment
      run: ./scripts/setup_tests.sh --all
    
    - name: Build
      run: cargo build --release
    
    - name: Unit Tests
      run: cargo test --release
    
    - name: Integration Tests
      run: |
        cd bootloader_testing
        cargo test --features qemu
    
    - name: Architecture Tests
      run: ./scripts/run_tests.sh --arch ${{ matrix.arch }} --all
    
    - name: Validation
      run: |
        cargo run --bin boot_validator -- validate \
          --bootloader-path target/release/multios-bootloader \
          --kernel-path target/release/multios-kernel
    
    - name: Upload Results
      uses: actions/upload-artifact@v3
      with:
        name: test-results-${{ matrix.arch }}
        path: test_results/
```

### Jenkins Pipeline Setup

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    stages {
        stage('Setup') {
            steps {
                sh './scripts/setup_tests.sh --all'
            }
        }
        
        stage('Build') {
            steps {
                sh 'cargo build --release'
            }
        }
        
        stage('Test') {
            parallel {
                stage('Unit Tests') {
                    steps {
                        sh 'cargo test --release'
                    }
                }
                stage('Integration Tests') {
                    steps {
                        sh 'cd bootloader_testing && cargo test --features qemu'
                    }
                }
                stage('Architecture Tests') {
                    matrix {
                        axes {
                            axis {
                                name 'ARCH'
                                values 'x86_64', 'aarch64', 'riscv64'
                            }
                        }
                        steps {
                            sh "./scripts/run_tests.sh --arch ${ARCH} --all"
                        }
                    }
                }
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

## Troubleshooting Procedures

### Common Issues and Solutions

#### QEMU Not Found
```bash
# Check QEMU installation
which qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64

# Install QEMU (Ubuntu/Debian)
sudo apt-get install qemu-system-x86 qemu-system-arm qemu-system-riscv64

# Verify QEMU installation
qemu-system-x86_64 --version
qemu-system-aarch64 --version
qemu-system-riscv64 --version
```

#### Permission Issues
```bash
# Check file permissions
ls -la scripts/*.sh

# Make scripts executable
chmod +x scripts/*.sh

# Check KVM permissions
ls -la /dev/kvm

# Add user to KVM group
sudo usermod -a -G kvm $USER
# Log out and back in
```

#### Build Failures
```bash
# Clean build artifacts
cargo clean
rm -rf target/debug/deps/*bootloader*

# Update dependencies
cargo update

# Check Rust toolchain
rustup update
rustup toolchain list

# Verify environment
source test_environment/env.sh
echo $MULTIOS_TEST_DIR
```

#### Memory Issues
```bash
# Check available memory
free -h
vm_stat (macOS)

# Increase swap space
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

#### Test Timeouts
```bash
# Increase test timeout
export MULTIOS_TIMEOUT=600
./scripts/run_tests.sh --all

# Run tests in debug mode
export RUST_LOG=debug
./scripts/run_tests.sh --all --verbose
```

### Debug Mode Testing

```bash
# Enable debug logging
export RUST_LOG=debug
export MULTIOS_DEBUG=1

# Run with detailed output
./scripts/run_tests.sh --all --verbose

# Monitor test execution
tail -f test_environment/logs/test.log

# Check QEMU console output
tail -f test_results/console_*.log
```

### Log Analysis

```bash
# View test logs
cat test_results/bootloader_test_*.log

# Analyze structured logs
cat test_environment/logs/structured.log | jq '.'

# Check performance logs
cat test_environment/logs/performance_*.json | jq '.'

# Monitor real-time logs
make watch
```

## Custom Test Development

### Creating Custom Test Scripts

```bash
# Create custom test script
cat > custom_test.sh << 'EOF'
#!/bin/bash
set -euo pipefail

source test_environment/env.sh

echo "Running custom bootloader test..."

# Test parameters
ARCH="${1:-x86_64}"
MEMORY="${2:-1G}"
MODE="${3:-uefi}"

# Run custom QEMU test
qemu-system-$ARCH \
    -m $MEMORY \
    -nographic \
    -kernel test_kernel_$ARCH.bin \
    -append "custom_test=true mode=$MODE console=ttyS0" \
    -serial file:custom_test_$ARCH.log

echo "Custom test completed for $ARCH"
EOF

chmod +x custom_test.sh
./custom_test.sh x86_64 2G uefi
```

### Creating Custom Validation

```rust
// validation/src/custom_validator.rs
use anyhow::Result;

pub fn custom_validation_test(
    bootloader_path: &Path,
    kernel_path: &Path,
) -> Result<TestResult> {
    // Custom validation logic
    Ok(TestResult {
        // Test result implementation
    })
}
```

## Performance Optimization

### Test Optimization

```bash
# Parallel test execution
export MULTIOS_PARALLEL_TESTS=4
./scripts/run_tests.sh --all

# Enable KVM acceleration
export QEMU_ENABLE_KVM=true
./scripts/run_tests.sh --qemu

# Optimize build settings
export RUSTFLAGS="-C target-cpu=native"
cargo build --release
```

### Resource Monitoring

```bash
# Monitor system resources during tests
htop &
PID=$!

# Run tests
./scripts/run_tests.sh --all

# Stop monitoring
kill $PID

# Check resource usage
iostat -x 1 10
```

## Documentation and Reporting

### Generate Test Documentation

```bash
# Generate test documentation
cargo doc --no-deps --all

# Create test report
cargo run --bin boot_validator -- report \
    --results-dir test_results \
    --output-file test_documentation.html

# Export test results
tar -czf test_results.tar.gz test_results/
```

### Test Coverage Analysis

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --all-features --output-dir coverage/

# View coverage report
open coverage/tarpaulin-report.html
```

This comprehensive test procedure guide ensures thorough testing of the MultiOS bootloader across all supported architectures and scenarios.
