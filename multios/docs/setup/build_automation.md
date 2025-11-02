# MultiOS Build Automation Documentation

## Table of Contents

1. [Overview](#overview)
2. [Build System Architecture](#build-system-architecture)
3. [Getting Started](#getting-started)
4. [Build Scripts](#build-scripts)
5. [Testing Framework](#testing-framework)
6. [CI/CD Configuration](#cicd-configuration)
7. [Docker Setup](#docker-setup)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Usage](#advanced-usage)

## Overview

MultiOS uses a comprehensive build automation system designed to support multi-architecture compilation, automated testing, and continuous integration. The system is built around Rust's Cargo build system with custom scripts for cross-compilation, QEMU testing, and CI/CD integration.

### Key Features

- **Multi-Architecture Support**: x86_64, ARM64 (AArch64), and RISC-V64
- **Automated Testing**: Unit tests, integration tests, and QEMU emulation
- **CI/CD Integration**: GitHub Actions, GitLab CI, and Jenkins pipelines
- **Docker Support**: Reproducible build environments
- **Coverage Analysis**: Code coverage reporting with tarpaulin
- **Security Auditing**: Automated dependency vulnerability scanning

## Build System Architecture

The build system consists of several layers:

```
┌─────────────────────────────────────────────┐
│              CI/CD Platforms                │
├─────────────────────────────────────────────┤
│           Build & Test Scripts             │
├─────────────────────────────────────────────┤
│              Docker Containers             │
├─────────────────────────────────────────────┤
│           Cargo Build System               │
├─────────────────────────────────────────────┤
│          Cross-Compilation Tools           │
├─────────────────────────────────────────────┤
│            Target Architectures            │
└─────────────────────────────────────────────┘
```

### Directory Structure

```
multios/
├── Cargo.toml                 # Workspace configuration
├── scripts/
│   ├── build.sh              # Main build script (Linux/macOS)
│   ├── build.bat             # Main build script (Windows)
│   └── test.sh               # Testing framework
├── kernel/                   # Kernel crate
│   └── Cargo.toml
├── bootloader/               # Bootloader crate
│   └── Cargo.toml
├── userland/                 # User-space components
│   └── Cargo.toml
├── tests/                    # Test suites
│   ├── unit/                 # Unit tests
│   ├── integration/          # Integration tests
│   └── benchmarks/           # Performance benchmarks
├── .github/workflows/        # GitHub Actions
├── .gitlab-ci.yml           # GitLab CI
├── Jenkinsfile              # Jenkins pipeline
├── Dockerfile               # Docker build environment
├── docker-compose.yml       # Docker Compose setup
└── target/                  # Build artifacts (generated)
```

## Getting Started

### Prerequisites

Install the following tools:

#### On Ubuntu/Debian:

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    qemu-system-x86 \
    qemu-system-aarch64 \
    qemu-system-riscv64 \
    gcc-aarch64-linux-gnu \
    gcc-riscv64-linux-gnu \
    doxygen \
    graphviz

# Rust tools
cargo install cargo-audit cargo-tarpaulin cross
```

#### On macOS:

```bash
# Using Homebrew
brew install rust qemu aarch64-linux-gnu-gcc riscv64-linux-gnu-gcc

# Rust tools
cargo install cargo-audit cargo-tarpaulin cross
```

#### On Windows:

```bash
# Install Rust from https://rustup.rs
# Install QEMU from https://www.qemu.org/download/#windows
# Install cross-compilation toolchains as needed
```

### Quick Start

1. **Clone the repository**:
   ```bash
   git clone <multios-repo-url>
   cd multios
   ```

2. **Build for x86_64**:
   ```bash
   ./scripts/build.sh --target x86_64 --release
   ```

3. **Run tests**:
   ```bash
   ./scripts/test.sh --target x86_64 --suite all --qemu
   ```

4. **Using Docker**:
   ```bash
   docker-compose up builder
   docker-compose exec builder bash
   ```

## Build Scripts

### build.sh (Linux/macOS)

The main build script for compiling MultiOS across different architectures.

#### Usage

```bash
./scripts/build.sh [OPTIONS]
```

#### Options

- `-t, --target TARGET`: Target architecture (x86_64, arm64, riscv64)
- `-r, --release`: Build in release mode
- `-c, --clean`: Clean build artifacts before building
- `-j, --parallel N`: Enable parallel builds with N jobs
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Show help message

#### Examples

```bash
# Basic build
./scripts/build.sh --target x86_64

# Release build with cleaning
./scripts/build.sh --target x86_64 --release --clean

# Parallel build with verbose output
./scripts/build.sh --target arm64 --parallel 4 --verbose
```

### build.bat (Windows)

Windows version of the build script with equivalent functionality.

#### Usage

```batch
scripts\build.bat [OPTIONS]
```

### test.sh

The testing framework that handles unit tests, integration tests, and QEMU testing.

#### Usage

```bash
./scripts/test.sh [OPTIONS]
```

#### Options

- `-t, --target TARGET`: Target architecture
- `-s, --suite SUITE`: Test suite (unit, integration, all)
- `-q, --qemu`: Enable QEMU testing
- `-c, --coverage`: Generate test coverage report
- `-p, --parallel`: Run tests in parallel
- `--clean`: Clean test artifacts
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Show help message

#### Examples

```bash
# Run unit tests
./scripts/test.sh --target x86_64 --suite unit

# Run all tests with QEMU and coverage
./scripts/test.sh --target x86_64 --suite all --qemu --coverage

# Run integration tests only
./scripts/test.sh --target arm64 --suite integration --qemu
```

## Testing Framework

The testing framework provides comprehensive testing capabilities across multiple architectures.

### Test Types

1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Cross-component interaction testing
3. **QEMU Tests**: Emulated hardware testing
4. **Coverage Analysis**: Code coverage reporting
5. **Performance Benchmarks**: Performance regression testing

### QEMU Configurations

#### x86_64
```bash
qemu-system-x86_64 -m 256M -drive format=raw,file=KERNEL_IMAGE -serial stdio -monitor none -display none
```

#### ARM64
```bash
qemu-system-aarch64 -m 256M -machine virt -cpu cortex-a57 -nographic -kernel KERNEL_IMAGE
```

#### RISC-V64
```bash
qemu-system-riscv64 -m 256M -machine virt -nographic -kernel KERNEL_IMAGE
```

### Test Results

Test results are stored in `test_results/` with the following structure:

```
test_results/
├── junit.xml                    # JUnit test report
├── coverage_*.html              # HTML coverage report
├── qemu_test_*.log             # QEMU test logs
└── test_report_*.txt           # Human-readable test report
```

## CI/CD Configuration

MultiOS supports multiple CI/CD platforms for automated building and testing.

### GitHub Actions

Location: `.github/workflows/ci.yml`

Features:
- Multi-job matrix for different architectures
- Caching for faster builds
- Artifact publishing
- Deployment to staging and production
- Security auditing
- Coverage reporting

#### Workflow Triggers

- Push to `main`, `develop`, `staging` branches
- Pull requests to `main`
- Manual workflow dispatch

### GitLab CI

Location: `.gitlab-ci.yml`

Features:
- Stage-based pipeline
- Parallel job execution
- Docker integration
- Coverage collection
- Environment-specific deployments

#### Pipeline Stages

1. **validate**: Linting and security auditing
2. **test**: Unit and integration testing
3. **build**: Release builds
4. **deploy**: Staging and production deployment

### Jenkins

Location: `Jenkinsfile`

Features:
- Declarative pipeline syntax
- Parallel stage execution
- Docker agent support
- Email notifications
- Artifact archiving
- HTML report publishing

#### Pipeline Stages

1. **Prepare**: Environment setup
2. **Lint and Format**: Code quality checks
3. **Security Audit**: Dependency scanning
4. **Unit Tests Matrix**: Multi-architecture testing
5. **Integration Tests**: System-level testing
6. **QEMU Tests Matrix**: Hardware emulation testing
7. **Code Coverage**: Coverage analysis
8. **Build Matrix**: Release builds
9. **Documentation**: API documentation generation
10. **Performance Benchmarks**: Performance regression testing
11. **Deploy**: Environment-specific deployment

## Docker Setup

Docker provides a consistent build and test environment across different platforms.

### Dockerfile

The Dockerfile creates a comprehensive build environment with:
- Rust toolchain
- Cross-compilation toolchains
- QEMU for testing
- Documentation tools
- Testing utilities

### Docker Compose

Docker Compose configuration provides:

1. **builder**: Interactive development environment
2. **test-x86_64/arm64/riscv64**: Architecture-specific testing
3. **docs**: Documentation generation
4. **ci-sim**: CI pipeline simulation

### Usage Examples

#### Interactive Development
```bash
docker-compose up -d builder
docker-compose exec builder bash
cd kernel
cargo test
```

#### Run All Tests
```bash
docker-compose up test-x86_64 test-arm64 test-riscv64
```

#### CI Simulation
```bash
docker-compose up ci-sim
docker-compose logs ci-sim
```

#### Generate Documentation
```bash
docker-compose up docs
ls docs/generated/
```

## Troubleshooting

### Common Issues

#### Build Failures

**Issue**: Cross-compilation toolchain not found
```bash
# Solution: Install missing toolchain
sudo apt-get install gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu
```

**Issue**: QEMU not available
```bash
# Solution: Install QEMU
sudo apt-get install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64
```

**Issue**: Rust toolchain outdated
```bash
# Solution: Update Rust
rustup update
```

#### Test Failures

**Issue**: Timeout in QEMU tests
```bash
# Solution: Check kernel boot messages and adjust timeout
# Review test logs in test_results/qemu_test_*.log
```

**Issue**: Coverage report generation fails
```bash
# Solution: Install tarpaulin
cargo install cargo-tarpaulin
```

#### CI/CD Issues

**Issue**: GitHub Actions cache misses
```bash
# Solution: Check cache key patterns in .github/workflows/ci.yml
# Ensure Cargo.lock is committed
```

**Issue**: Docker build fails
```bash
# Solution: Clear Docker cache
docker system prune -a
```

### Debug Mode

Enable verbose output for detailed debugging:

```bash
./scripts/build.sh --target x86_64 --verbose
./scripts/test.sh --target x86_64 --suite all --verbose
```

### Log Files

Build and test logs are stored in:
- `build.log`: Build process log
- `test.log`: Test execution log
- `test_results/qemu_test_*.log`: QEMU test output

## Advanced Usage

### Custom Build Profiles

Create custom build profiles in `Cargo.toml`:

```toml
[profile.custom-debug]
inherits = "dev"
debug = true
opt-level = 0

[profile.custom-release]
inherits = "release"
lto = "fat"
opt-level = 3
```

### Cross-Compilation with cross

Use `cross` for easier cross-compilation:

```bash
cross test --target aarch64-unknown-none
cross build --target riscv64gc-unknown-none --release
```

### Performance Benchmarking

Run performance benchmarks:

```bash
cd tests/benchmarks
cargo bench
```

### Custom QEMU Testing

Create custom QEMU test scenarios:

```bash
# Custom QEMU command
qemu-system-x86_64 -m 512M \
    -drive format=raw,file=kernel.bin \
    -serial stdio \
    -monitor telnet:127.0.0.1:4444,server,nowait \
    -gdb tcp::1234
```

### Automated Documentation

Generate API documentation:

```bash
cargo doc --no-deps --all-features --open
```

### Security Scanning

Run security audits:

```bash
cargo audit
cargo deny check
```

### Release Process

1. **Version bump**:
   ```bash
   # Update version in Cargo.toml files
   ```

2. **Tag release**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **CI/CD will automatically**:
   - Build for all targets
   - Run comprehensive tests
   - Generate documentation
   - Create release artifacts
   - Deploy to staging/production

### Contributing

When contributing to the build system:

1. **Test thoroughly** across all architectures
2. **Update documentation** for any changes
3. **Follow existing patterns** and conventions
4. **Add appropriate tests** for new functionality
5. **Update CI/CD configurations** as needed

---

For more information, see the [Technical Specifications](../multios_technical_specifications.md) and individual component documentation.