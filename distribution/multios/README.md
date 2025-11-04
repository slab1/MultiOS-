# MultiOS Build and Testing System

This repository contains the complete build and testing infrastructure for MultiOS, a universal educational operating system written in Rust.

## Quick Start

### Prerequisites

Install the required dependencies:

```bash
# On Ubuntu/Debian
make install-deps
make install-tools

# Or manually
sudo apt-get install -y build-essential qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu doxygen graphviz
cargo install cargo-audit cargo-tarpaulin cross
```

### Build

```bash
# Build for specific architecture
make build-x86_64
make build-arm64
make build-riscv64

# Build for all architectures
make build-all

# Release build
RELEASE=1 make build-x86_64
```

### Test

```bash
# Run tests for specific architecture
make test-x86_64
make test-arm64
make test-riscv64

# Run all tests
make test-all

# Run QEMU tests
make test-qemu-x86_64
make test-qemu-all

# Generate coverage report
make coverage
```

### Using Scripts Directly

```bash
# Build script
./scripts/build.sh --target x86_64 --release

# Test script
./scripts/test.sh --target x86_64 --suite all --qemu --coverage
```

### Docker Usage

```bash
# Interactive development
make docker-build
docker-compose up -d builder
make dev-x86_64

# Run tests in Docker
make docker-test

# Simulate CI pipeline
make ci-sim
```

## Available Make Targets

### Build Targets
- `make build-x86_64` - Build for x86_64 architecture
- `make build-arm64` - Build for ARM64 architecture  
- `make build-riscv64` - Build for RISC-V64 architecture
- `make build-all` - Build for all architectures

### Test Targets
- `make test-x86_64` - Run tests for x86_64
- `make test-arm64` - Run tests for ARM64
- `make test-riscv64` - Run tests for RISC-V64
- `make test-all` - Run tests for all architectures
- `make test-qemu-all` - Run QEMU tests for all architectures

### Code Quality Targets
- `make fmt` - Format code
- `make lint` - Run clippy linter
- `make audit` - Run security audit
- `make coverage` - Generate coverage report

### Docker Targets
- `make docker-build` - Build Docker image
- `make docker-test` - Run tests in Docker
- `make ci-sim` - Simulate CI pipeline
- `make container-shell` - Open shell in container

### Utility Targets
- `make clean` - Clean build artifacts
- `make setup` - Setup development environment
- `make docs` - Generate documentation
- `make info` - Show build information
- `make help` - Show available targets

## Architecture Support

MultiOS supports three target architectures:

1. **x86_64** - Intel/AMD 64-bit processors
2. **ARM64 (AArch64)** - ARM 64-bit processors
3. **RISC-V64** - RISC-V 64-bit processors

Each architecture has its own build and test pipeline, ensuring compatibility and performance across different hardware platforms.

## Testing Framework

The testing framework includes:

### Unit Tests
- Individual component testing
- Fast execution
- Comprehensive code coverage

### Integration Tests
- System-level interaction testing
- Cross-component validation
- Real-world scenario simulation

### QEMU Testing
- Hardware emulation testing
- Architecture-specific testing
- Automated boot testing

### Performance Benchmarks
- Memory allocation benchmarks
- Scheduler performance tests
- IPC latency measurements
- Context switch overhead analysis

### Code Coverage
- Line coverage analysis
- Branch coverage reporting
- HTML and XML report generation
- Integration with CI/CD

## CI/CD Integration

### GitHub Actions
- Multi-job matrix for different architectures
- Automated testing on pull requests
- Release artifact generation
- Documentation publishing

### GitLab CI
- Stage-based pipeline
- Parallel job execution
- Docker integration
- Environment-specific deployments

### Jenkins
- Declarative pipeline syntax
- Parallel stage execution
- Docker agent support
- Email notifications
- HTML report publishing

### Docker
- Reproducible build environments
- Cross-platform compatibility
- Consistent testing environments
- CI pipeline simulation

## Directory Structure

```
multios/
├── Cargo.toml                 # Workspace configuration
├── Makefile                   # Build system shortcuts
├── scripts/                   # Build and test scripts
│   ├── build.sh              # Main build script (Linux/macOS)
│   ├── build.bat             # Main build script (Windows)
│   └── test.sh               # Testing framework
├── kernel/                    # Kernel crate
├── bootloader/                # Bootloader crate
├── userland/                  # User-space components
├── tests/                     # Test suites
│   ├── unit/                 # Unit tests
│   ├── integration/          # Integration tests
│   └── benchmarks/           # Performance benchmarks
├── ci/                       # CI/CD configuration
│   ├── github/              # GitHub Actions workflows
│   ├── gitlab/              # GitLab CI configuration
│   └── jenkins/             # Jenkins pipeline
├── docs/setup/               # Documentation
│   └── build_automation.md  # Detailed build documentation
├── .github/workflows/        # GitHub Actions
├── .gitlab-ci.yml           # GitLab CI
├── Jenkinsfile              # Jenkins pipeline
├── Dockerfile               # Docker build environment
├── docker-compose.yml       # Docker Compose setup
└── target/                  # Build artifacts (generated)
```

## Development Workflow

### 1. Setup Development Environment
```bash
make setup
```

### 2. Quick Development Cycle
```bash
make quick-test    # Format, lint, and test
```

### 3. Comprehensive Testing
```bash
make full-test     # Complete test suite
```

### 4. Performance Testing
```bash
make bench         # Run benchmarks
```

### 5. Release Preparation
```bash
make release-prepare
make release
```

## Troubleshooting

### Common Issues

**Build failures with missing toolchains:**
```bash
# Install missing cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu
```

**QEMU not found:**
```bash
# Install QEMU
sudo apt-get install qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64
```

**Rust toolchain issues:**
```bash
# Update Rust
rustup update
```

### Debug Mode

Enable verbose output:
```bash
VERBOSE=1 make build-x86_64
VERBOSE=1 make test-x86_64
```

### Log Files

- `build.log` - Build process log
- `test.log` - Test execution log
- `test_results/qemu_test_*.log` - QEMU test output

## Documentation

For detailed documentation, see:
- [Build Automation Guide](docs/setup/build_automation.md)
- [Technical Specifications](../multios_technical_specifications.md)

## Contributing

1. Follow the existing code style and conventions
2. Add tests for new functionality
3. Update documentation as needed
4. Test across all supported architectures
5. Run the full test suite before submitting PRs

## License

MultiOS is licensed under MIT or Apache-2.0. See LICENSE file for details.