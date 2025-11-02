# MultiOS CI/CD Pipeline System

A comprehensive CI/CD pipeline system for MultiOS testing across x86_64, ARM64, and RISC-V architectures.

## 🚀 Overview

This CI/CD pipeline system provides automated testing, building, and deployment across multiple architectures with:

- **Matrix builds** for x86_64, ARM64, and RISC-V
- **Automated testing** stages (unit, integration, performance, security)
- **Cross-compilation validation** for all targets
- **Container-based testing** environments (Debian, Ubuntu, Alpine, Fedora)
- **Build artifact generation** and storage
- **Quality gates** with pass/fail criteria
- **Automated reporting** and notifications
- **Performance monitoring** integration

## 📁 Directory Structure

```
testing/ci_cd/
├── github_actions/          # GitHub Actions workflows
│   ├── ci_pipeline.yml      # Main CI pipeline
│   └── release_pipeline.yml # Release pipeline
├── scripts/                 # CI/CD scripts
│   ├── run_container_tests.sh
│   ├── setup_container_environment.sh
│   ├── validate_cross_compilation.sh
│   ├── run_benchmarks.sh
│   ├── generate_quality_report.sh
│   ├── update_monitoring.sh
│   ├── send_notifications.sh
│   ├── deploy_artifacts.sh
│   ├── run_integration_tests.sh
│   └── setup_qemu.sh
├── containers/              # Container configurations
│   ├── debian/Dockerfile
│   ├── ubuntu/Dockerfile
│   ├── alpine/Dockerfile
│   └── fedora/Dockerfile
├── config/                  # Configuration files
│   └── ci_cd_config.toml
├── artifacts/              # Build artifacts storage
├── reports/                # Test reports and logs
├── monitoring/             # Monitoring integration
└── README.md               # This file
```

## 🏗️ Architecture Support

| Architecture | Cross-Compile | Native | QEMU Support | Container |
|-------------|---------------|---------|--------------|-----------|
| x86_64      | ✅ Native     | ✅ Yes  | ✅ Yes       | ✅ Yes    |
| ARM64       | ✅ Yes        | ❌ No   | ✅ Yes       | ✅ Yes    |
| RISC-V64    | ✅ Yes        | ❌ No   | ✅ Yes       | ✅ Yes    |

## 🔧 Setup

### Prerequisites

1. **System Dependencies**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install -y qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
     qemu-system-misc qemu-img curl wget git build-essential

   # Fedora
   sudo dnf install -y qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
     qemu-system-misc qemu-img curl wget git gcc

   # Arch Linux
   sudo pacman -S --noconfirm qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
     qemu-system-misc qemu-arch-extra curl wget git base-devel
   ```

2. **Rust Toolchain**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

   # Install cross-compilation targets
   rustup target add x86_64-unknown-none \
     x86_64-unknown-linux-gnu \
     x86_64-unknown-linux-musl \
     aarch64-unknown-none \
     aarch64-unknown-linux-gnu \
     aarch64-unknown-linux-musl \
     riscv64gc-unknown-none-elf \
     riscv64gc-unknown-linux-gnu \
     riscv64gc-unknown-linux-musl

   # Install cross compiler
   cargo install cross --git https://github.com/cross-rs/cross
   ```

### Environment Setup

1. **Clone and setup**:
   ```bash
   cd /workspace/testing/ci_cd
   chmod +x scripts/*.sh
   ```

2. **Setup QEMU**:
   ```bash
   ./scripts/setup_qemu.sh
   ```

3. **Configure environment** (optional):
   ```bash
   export SLACK_WEBHOOK="https://hooks.slack.com/..."
   export DISCORD_WEBHOOK="https://discord.com/api/webhooks/..."
   export EMAIL_RECIPIENTS="dev@multios.dev,qa@multios.dev"
   export GITHUB_TOKEN="ghp_..."
   export GITHUB_REPOSITORY="multios/multios"
   ```

## 🚀 Usage

### GitHub Actions Workflows

The pipeline runs automatically on:
- Push to `main` or `develop` branches
- Pull requests
- Daily scheduled runs (2 AM UTC)
- Manual workflow dispatch

#### CI Pipeline Triggers

```yaml
# Automatically triggers on:
- push: [main, develop]
- pull_request: [main, develop]
- schedule: [daily at 2 AM UTC]
- workflow_dispatch: [manual triggers]
```

#### Custom Architecture Selection

```yaml
# In workflow_dispatch inputs:
architectures: "x86_64,arm64,riscv64"  # Comma-separated list
```

### Manual Pipeline Execution

#### Run Full CI Pipeline
```bash
# Run all tests across all architectures
./scripts/run_container_tests.sh

# Run benchmarks
./scripts/run_benchmarks.sh /workspace/artifacts

# Generate quality report
./scripts/generate_quality_report.sh

# Update monitoring dashboard
./scripts/update_monitoring.sh

# Send notifications
./scripts/send_notifications.sh SUCCESS CI
```

#### Test Specific Architecture
```bash
# Test only x86_64
./scripts/run_container_tests.sh --arch x86_64

# Validate cross-compilation for ARM64
./scripts/validate_cross_compilation.sh arm64

# Run integration tests for RISC-V
./scripts/run_integration_tests.sh riscv64
```

#### Deploy Artifacts
```bash
# Deploy to all configured registries
./scripts/deploy_artifacts.sh /workspace/artifacts v1.2.3

# Deploy specific version
DEPLOY_VERSION=v1.2.3 ./scripts/deploy_artifacts.sh
```

### Container Testing

Build and test in different environments:

```bash
# Build Debian container
docker build -f containers/debian/Dockerfile -t multios-test:debian .

# Build Ubuntu container
docker build -f containers/ubuntu/Dockerfile -t multios-test:ubuntu .

# Build Alpine container
docker build -f containers/alpine/Dockerfile -t multios-test:alpine .

# Run containerized tests
docker run --rm -v $(pwd):/workspace multios-test:debian
```

## 🧪 Testing Stages

### 1. Quality Gate
- **Format checking**: `cargo fmt -- --check`
- **Linting**: `cargo clippy -- -D warnings`
- **Security audit**: `cargo audit`

### 2. Build & Test Matrix
For each architecture:
- **Unit tests**: `cargo test --lib`
- **Integration tests**: `cargo test --test '*'`
- **Cross-compilation validation**: Binary creation and format verification

### 3. Container Testing
Testing across different distributions:
- Debian (bookworm-slim)
- Ubuntu (22.04)
- Alpine (3.18)
- Fedora (38)

### 4. Performance Benchmarks
- **Boot time** measurement
- **Memory usage** profiling
- **CPU performance** scoring
- **I/O throughput** testing
- **Network latency** measurement

### 5. Integration Tests
- **Basic system tests**: Boot, memory, processes, filesystem, interrupts
- **Network tests**: TCP/UDP communication, network stack
- **Driver tests**: Storage, network, display, input drivers
- **Stress tests**: Memory, CPU, I/O stress testing

### 6. Security Scanning
- **Vulnerability scanning**: Using Trivy
- **Dependency audit**: `cargo audit`
- **Code quality**: Clippy warnings

## 📊 Quality Gates

| Gate | Threshold | Description |
|------|-----------|-------------|
| Test Pass Rate | ≥95% | Minimum test success rate |
| Code Coverage | ≥80% | Minimum code coverage |
| Security Issues | 0 | No critical/high vulnerabilities |
| Performance Regression | ≤5% | Maximum performance regression |

## 📈 Performance Baselines

### Boot Time Targets
- **x86_64**: 500ms ± 100ms
- **ARM64**: 600ms ± 150ms
- **RISC-V64**: 800ms ± 200ms

### Memory Usage Targets
- **All architectures**: 512MB ± 128MB

### CPU Performance Targets
- **x86_64**: 100 ± 10 points
- **ARM64**: 80 ± 15 points
- **RISC-V64**: 60 ± 20 points

## 🔔 Notifications

### Supported Channels
- **Slack**: Via webhook
- **Discord**: Via webhook
- **Email**: Via sendmail/mail
- **GitHub**: Commit status updates

### Notification Triggers
- **Success**: All tests pass
- **Failure**: Any test fails
- **Partial**: Some tests fail
- **Quality Gate Fail**: Quality gates not met

### Environment Variables
```bash
export SLACK_WEBHOOK="https://hooks.slack.com/..."
export DISCORD_WEBHOOK="https://discord.com/api/webhooks/..."
export EMAIL_RECIPIENTS="dev@multios.dev,qa@multios.dev"
export GITHUB_TOKEN="ghp_..."
export GITHUB_REPOSITORY="multios/multios"
```

## 📦 Artifact Management

### Build Artifacts
- **Binaries**: For each target architecture
- **Packages**: Compressed archives with checksums
- **Documentation**: READMEs, changelogs, installation guides

### Storage Locations
- **GitHub Actions Artifacts**: 30-day retention
- **Local Storage**: `/workspace/artifacts/`
- **Registry Deployment**: Docker Hub, GitHub Releases, crates.io
- **Object Storage**: AWS S3 (if configured)

## 📊 Monitoring Integration

### Dashboard Integration
- **Performance Dashboard**: `/workspace/perf/monitor_dashboard/`
- **Metrics Collection**: Build, test, performance metrics
- **Alert System**: Automated alerts for failures/regressions
- **Prometheus Metrics**: Available in `/workspace/testing/ci_cd/monitoring/prometheus/`

### Metrics Collected
- Build duration and resource usage
- Test pass rates and coverage
- Performance benchmarks and trends
- Quality gate status
- Security vulnerability counts

## 🔧 Configuration

### Main Configuration
Edit `config/ci_cd_config.toml` to customize:
- Architecture settings
- Test timeouts and thresholds
- Quality gate requirements
- Container base images
- Performance baselines
- Notification settings

### QEMU Configuration
Edit `qemu/configs/{arch}.conf` to customize:
- Machine types and CPU settings
- Memory allocation
- Network configuration
- Serial and monitor settings

## 🐛 Troubleshooting

### Common Issues

1. **QEMU not found**:
   ```bash
   # Check installation
   which qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64
   
   # Reinstall if needed
   ./scripts/setup_qemu.sh
   ```

2. **KVM access denied**:
   ```bash
   # Add user to kvm group
   sudo usermod -a -G kvm $USER
   
   # Log out and back in, or:
   newgrp kvm
   ```

3. **Cross-compilation fails**:
   ```bash
   # Verify targets are installed
   rustup target list --installed
   
   # Reinstall cross tool
   cargo install cross --git https://github.com/cross-rs/cross
   ```

4. **Container builds fail**:
   ```bash
   # Check Docker daemon
   docker --version
   docker ps
   
   # Verify base images
   docker pull debian:bookworm-slim
   docker pull ubuntu:22.04
   ```

### Debug Mode

Enable debug logging:
```bash
export RUST_BACKTRACE=1
export CARGO_LOG=debug
./scripts/run_container_tests.sh
```

## 📝 Contributing

### Adding New Tests

1. Create test in appropriate directory:
   - Unit tests: `tests/unit/`
   - Integration tests: `tests/integration/`
   - Performance tests: `tests/benchmarks/`

2. Update test runner scripts:
   - `scripts/run_container_tests.sh`
   - `scripts/run_integration_tests.sh`

3. Add to CI pipeline:
   - Update `github_actions/ci_pipeline.yml`

### Adding New Architectures

1. Update configuration:
   - `config/ci_cd_config.toml`
   - `scripts/run_container_tests.sh`

2. Add GitHub Actions matrix:
   - `github_actions/ci_pipeline.yml`

3. Create QEMU configuration:
   - `qemu/configs/{arch}.conf`

## 📄 License

This CI/CD pipeline system is part of the MultiOS project.

## 🤝 Support

- **Issues**: Create an issue in the GitHub repository
- **Documentation**: Check `/workspace/docs/` directory
- **Community**: Join our Discord server
- **Email**: dev@multios.dev

---

*Generated by MultiOS CI/CD Pipeline System*