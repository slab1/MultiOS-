# MultiOS Package Manager - Comprehensive Documentation

## Overview

The MultiOS Package Manager is a comprehensive, secure, and efficient package management system designed for the MultiOS ecosystem. It provides advanced features for package distribution, dependency resolution, security validation, and automated system maintenance.

## Table of Contents

1. [Architecture](#architecture)
2. [Installation](#installation)
3. [Core Features](#core-features)
4. [Usage Guide](#usage-guide)
5. [Development Guide](#development-guide)
6. [API Reference](#api-reference)
7. [Security](#security)
8. [Troubleshooting](#troubleshooting)
9. [Contributing](#contributing)

## Architecture

### System Components

The MultiOS Package Manager consists of several interconnected components:

```
┌─────────────────────────────────────────────────────────────┐
│                    MultiOS Package Manager                  │
├─────────────────────────────────────────────────────────────┤
│  Python CLI & API  │  Rust Core Engine  │  Repository Mgmt  │
├─────────────────────────────────────────────────────────────┤
│  Security Module   │  Package Storage   │  Scheduler        │
├─────────────────────────────────────────────────────────────┤
│  Package Builder   │  Repository Builder│  Testing Suite    │
└─────────────────────────────────────────────────────────────┘
```

#### Core Components

1. **Rust Core Engine** (`src/`)
   - High-performance package operations
   - Cryptographic security features
   - Dependency resolution algorithms
   - Repository management
   - Storage and caching

2. **Python Interface** (`python/`)
   - Command-line interface
   - Python API for applications
   - Interactive tools
   - Package building utilities

3. **Repository Management** (`tools/`)
   - Repository builder
   - Package signing and verification
   - Bulk operations

4. **Testing Framework** (`tests/`)
   - Unit tests
   - Integration tests
   - Performance benchmarks
   - Security validation

### Key Features

- **Multi-Architecture Support**: x86_64, ARM64, RISC-V, Universal
- **Security-First Design**: Digital signatures, checksums, vulnerability scanning
- **Delta Updates**: Efficient bandwidth usage with incremental updates
- **Dependency Resolution**: Automatic dependency checking and conflict detection
- **Rollback Capability**: Safe package rollbacks with backup support
- **Automated Scheduling**: Configurable update scheduling and notifications
- **Enterprise Features**: Bulk operations, policy management, audit trails

## Installation

### System Requirements

- Rust 1.70+ (for core components)
- Python 3.8+ (for CLI and API)
- Linux/macOS/Windows (cross-platform support)
- Minimum 100MB disk space
- Network connectivity for package downloads

### Quick Installation

```bash
# Clone the repository
git clone https://github.com/multios/package-manager.git
cd package-manager

# Build and install the core package manager
cd src
cargo build --release
sudo cp target/release/multios-pm /usr/local/bin/

# Install Python CLI tools
cd ../python
pip3 install -e .

# Verify installation
multios-pm --version
```

### Advanced Installation

#### From Source

1. **Install Rust Toolchain**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Build Core Components**
   ```bash
   cd /workspace/deployment/package_manager/src
   cargo build --release --features=crypto,delta-updates,security
   ```

3. **Install Python Dependencies**
   ```bash
   cd /workspace/deployment/package_manager/python
   pip3 install -r requirements.txt
   ```

4. **Create System Directories**
   ```bash
   sudo mkdir -p /var/lib/multios-package-manager
   sudo mkdir -p /etc/multios-package-manager
   sudo mkdir -p /var/log/multios-package-manager
   sudo chown -R $USER:$USER /var/lib/multios-package-manager
   ```

#### Docker Installation

```bash
# Build container
docker build -t multios-package-manager .

# Run container
docker run -it --rm \
  -v /var/lib/multios-package-manager:/var/lib/multios-package-manager \
  multios-package-manager
```

### Configuration

Create `/etc/multios-package-manager/config.toml`:

```toml
[general]
data_directory = "/var/lib/multios-package-manager"
log_level = "info"
max_concurrent_downloads = 10
download_timeout = 300

[security]
require_signature = false
allow_unsigned = true
trusted_publishers_only = false
checksum_algorithm = "sha256"

[updates]
auto_check = true
auto_install = false
check_interval = 86400  # 24 hours
security_updates_only = false
require_confirmation = true

[scheduler]
maintenance_day = "Sunday"
maintenance_time = "02:00"
timezone = "UTC"
notify_on_updates = true
```

## Core Features

### Package Operations

#### Installation
```bash
# Install a single package
multios-pm install firefox

# Install multiple packages
multios-pm install firefox git vscode

# Install specific version
multios-pm install firefox --version 91.0

# Install from specific repository
multios-pm install package --repository community
```

#### Removal
```bash
# Uninstall package
multios-pm uninstall firefox

# Remove package and configuration
multios-pm uninstall firefox --purge

# Force removal even with dependencies
multios-pm uninstall package --force
```

#### Updates
```bash
# Check for updates
multios-pm check-updates

# Update all packages
multios-pm update

# Update specific packages
multios-pm update firefox git

# Update security fixes only
multios-pm update --security-only
```

### Repository Management

#### Adding Repositories
```bash
# Add official repository
multios-pm repository add main https://repo.multios.org/main

# Add community repository
multios-pm repository add community https://repo.multios.org/community \
  --description "Community packages" \
  --priority 10

# List repositories
multios-pm repository list
```

#### Synchronization
```bash
# Sync all repositories
multios-pm sync

# Sync specific repository
multios-pm sync --repository main

# Force refresh
multios-pm sync --force
```

### Security Features

#### Package Verification
```bash
# Verify installed packages
multios-pm verify

# Verify specific packages
multios-pm verify firefox git

# Fix detected issues
multios-pm verify --fix
```

#### Package Signing
```bash
# Generate signing key
multios-pm sign generate-key my-key --algorithm ed25519

# Sign package
multios-pm sign sign package.tar.xz my-key

# Verify signature
multios-pm sign verify package.tar.xz my-key

# Trust public key
multios-pm sign trust key-id
```

### Search and Discovery

```bash
# Search packages
multios-pm search "web browser"

# Search with filters
multios-pm search browser --description --tags --limit 20

# List installed packages
multios-pm list

# Show package information
multios-pm info firefox --dependencies --security
```

## Usage Guide

### Basic Package Management

#### Finding Packages
```bash
# Search for packages by name
multios-pm search firefox

# Search in descriptions
multios-pm search "web browser" --description

# Search in tags
multios-pm search development --tags

# List all available packages
multios-pm list --available
```

#### Installing Software
```bash
# Install development tools
multios-pm install gcc make cmake rust

# Install GUI applications
multios-pm install gedit code spotify

# Install system utilities
multios-pm install htop vim curl wget
```

#### Managing Dependencies
The package manager automatically handles dependencies:
- Resolves required packages
- Checks for version conflicts
- Installs missing dependencies
- Warns about potential issues

### Advanced Operations

#### Bulk Operations
```bash
# Install development environment
multios-pm install gcc make cmake rust python3 nodejs git

# Install media packages
multios-pm install vlc gimp inkscape shotwell

# System maintenance
multios-pm update && multios-pm cleanup
```

#### Selective Updates
```bash
# Update only security packages
multios-pm update --security-only

# Update development packages
multios-pm update --development

# Update specific categories
multios-pm update --category "development,utilities"
```

#### Package Information
```bash
# Get detailed package info
multios-pm info package-name

# Show dependencies
multios-pm info package-name --dependencies

# Show security information
multios-pm info package-name --security

# Show files included
multios-pm info package-name --files
```

### Automated Updates

#### Configuration
```bash
# Enable automatic update checking
multios-pm configure --auto-check

# Enable automatic security updates
multios-pm configure --auto-install --security-only

# Set update schedule
multios-pm configure --check-interval 12 --maintenance-time 03:00
```

#### Manual Control
```bash
# Check for updates
multios-pm check-updates

# Install all updates
multios-pm update

# Install security updates only
multios-pm update --security-only

# Preview changes
multios-pm check-updates --dry-run
```

### Python API Usage

```python
import asyncio
from multios_pm import MultiOSPackageManager

async def main():
    pm = MultiOSPackageManager()
    
    # Install packages
    await pm.install_packages(['firefox', 'git'])
    
    # Search for packages
    packages = await pm.search_packages('web browser')
    for pkg in packages:
        print(f"{pkg.name} {pkg.version}")
    
    # Check for updates
    updates = await pm.check_for_updates()
    if updates:
        print(f"Found {len(updates)} updates")
    
    # Get installed packages
    installed = await pm.get_installed_packages()
    print(f"Installed {len(installed)} packages")

asyncio.run(main())
```

### Interactive Mode

```bash
# Launch interactive package manager
multios-pm --interactive

# Python interactive mode
python3 -c "from multios_pm import *; pm = MultiOSPackageManager(); pm.interactive_search()"
```

## Development Guide

### Creating Packages

#### Package Structure
```
my-package/
├── package.yaml          # Package configuration
├── src/                  # Source files
│   ├── bin/             # Executables
│   ├── lib/             # Libraries
│   └── share/           # Data files
├── docs/                # Documentation
├── tests/               # Test files
└── README.md            # Package documentation
```

#### Package Configuration (package.yaml)
```yaml
name: my-application
version: 1.0.0
description: My awesome application
architecture: universal
maintainer: Developer <dev@example.com>
license: MIT

dependencies:
  - libssl3 >= 1.1.0
  - glibc >= 2.31

provides:
  - myapp

conflicts:
  - old-application >= 1.0.0

categories:
  - development
  - productivity

tags:
  - application
  - development-tool

homepage: https://example.com/my-app
source_url: https://github.com/user/my-app

install_scripts:
  pre_install: |
    # Pre-installation script
    echo "Installing my-application"
  post_install: |
    # Post-installation script
    echo "Configuration complete"

files:
  - bin/
  - lib/
  - share/my-app/
  - etc/my-app/

exclude_patterns:
  - "*.o"
  - "*.a"
  - ".git"
  - "tests/"
```

#### Building Packages
```bash
# Create package skeleton
python3 -m multios_pm.builder skeleton my-package

# Build package
python3 -m multios_pm.builder build

# Sign package
python3 -m multios_pm.builder build --sign --key-id my-key

# Verify package
python3 -m multios_pm.builder verify package.tar.xz
```

### Repository Management

#### Creating Repositories
```bash
# Create new repository
multios-repo-builder create my-repo /path/to/repo

# Add packages
multios-repo-builder add package1.tar.xz --repo-name my-repo --repo-path /path/to/repo
multios-repo-builder add package2.tar.xz --repo-name my-repo --repo-path /path/to/repo

# Build repository index
multios-repo-builder build --repo-name my-repo --repo-path /path/to/repo

# Sync from directory
multios-repo-builder sync /path/to/packages --repo-name my-repo --repo-path /path/to/repo
```

#### Repository Structure
```
my-repository/
├── packages/            # Package files
│   ├── package1-1.0.0-universal.tar.xz
│   ├── package2-2.1.0-x86_64.tar.xz
│   └── package1-1.0.0-universal.tar.xz.sha256
└── metadata/            # Repository metadata
    ├── index.json       # Main index
    ├── index.json.gz    # Compressed index
    ├── category-development.json
    └── packages.txt     # Package list
```

### Development Workflow

1. **Package Development**
   - Create package skeleton
   - Implement application
   - Write tests
   - Create build scripts
   - Build and test package

2. **Repository Integration**
   - Add to development repository
   - Test with package manager
   - Create release package
   - Publish to production repository

3. **Quality Assurance**
   - Run test suite
   - Security scanning
   - Performance testing
   - Documentation review

## API Reference

### Python API

#### MultiOSPackageManager

```python
class MultiOSPackageManager:
    def __init__(self, data_dir: str = "/var/lib/multios-package-manager")
    
    # Package Operations
    async def install_packages(self, packages: List[str], versions: Optional[List[str]] = None) -> bool
    async def uninstall_packages(self, packages: List[str], purge: bool = False) -> bool
    async def update_packages(self, packages: Optional[List[str]] = None) -> List[Dict[str, Any]]
    async def search_packages(self, query: str, limit: int = 50) -> List[Package]
    async def get_installed_packages(self) -> List[Package]
    async def get_package_info(self, package_name: str) -> Optional[Package]
    
    # Repository Operations
    async def get_repositories(self) -> List[Repository]
    async def add_repository(self, name: str, url: str, description: str = "", priority: int = 0) -> bool
    async def remove_repository(self, name: str) -> bool
    async def sync_repositories(self, force: bool = False) -> bool
    
    # System Operations
    async def check_for_updates(self) -> List[UpdateInfo]
    async def verify_packages(self, packages: Optional[List[str]] = None) -> List[VerificationResult]
    async def rollback_package(self, package_name: str, version: str, backup: bool = True) -> bool
    async def get_status(self) -> Dict[str, Any]
    async def cleanup(self, old_versions: bool = True, clear_cache: bool = False, orphaned: bool = True, dry_run: bool = False) -> Dict[str, Any]
```

#### Data Classes

```python
@dataclass
class Package:
    name: str
    version: Version
    description: str
    architecture: str
    size: int
    install_date: Optional[str] = None
    repository: Optional[str] = None
    status: PackageStatus = PackageStatus.NOT_INSTALLED
    dependencies: List[str] = None
    tags: List[str] = None
    security_info: Optional[Dict[str, Any]] = None

@dataclass
class UpdateInfo:
    package_name: str
    current_version: Version
    available_version: Version
    update_type: UpdateType
    security_update: bool
    delta_available: bool
    repository: str

@dataclass
class Repository:
    name: str
    url: str
    description: str = ""
    enabled: bool = True
    priority: int = 0
    last_sync: Optional[str] = None
```

### CLI Reference

#### Global Options
```
--data-dir DIR          Package manager data directory
--config FILE           Configuration file path
--verbose, -v           Enable verbose output
--help, -h              Show help message
```

#### Commands

**install**
```
multios-pm install PACKAGE... [options]

Options:
  --version VERSION      Package version
  --force               Force installation
  --repository REPO     Specific repository
  --help               Show help
```

**search**
```
multios-pm search QUERY [options]

Options:
  --description         Search in descriptions
  --tags               Search in tags
  --limit N            Result limit
  --help               Show help
```

**update**
```
multios-pm update [PACKAGE...] [options]

Options:
  --development        Include development packages
  --security-only     Update only security fixes
  --help             Show help
```

**repository**
```
multios-pm repository <command> [options]

Commands:
  add NAME URL         Add repository
  remove NAME          Remove repository
  list                List repositories
  help               Show help
```

## Security

### Security Model

The MultiOS Package Manager implements a comprehensive security model:

1. **Package Signing**: All packages are digitally signed
2. **Checksum Verification**: SHA-256 checksums for integrity
3. **Repository Authentication**: HTTPS and GPG verification
4. **Vulnerability Scanning**: Automated security checks
5. **Trusted Publishers**: Whitelist of verified developers

### Security Features

#### Digital Signatures
- Ed25519 public key cryptography
- RSA-2048/4096 support
- ECDSA P-256/P-384 support
- Automatic key rotation

#### Vulnerability Management
- CVE database integration
- Automated security scanning
- Risk-based update prioritization
- Security advisory notifications

#### Access Control
- Role-based permissions
- Repository access policies
- Package installation restrictions
- Audit trail logging

### Best Practices

1. **Always verify signatures**
   ```bash
   multios-pm verify --strict
   ```

2. **Use trusted repositories only**
   ```bash
   multios-pm configure --trusted-publishers-only
   ```

3. **Enable automatic security updates**
   ```bash
   multios-pm configure --auto-install --security-only
   ```

4. **Regular security audits**
   ```bash
   multios-pm verify --security-audit
   ```

## Troubleshooting

### Common Issues

#### Installation Problems
```bash
# Check disk space
df -h /var/lib/multios-package-manager

# Verify permissions
ls -la /var/lib/multios-package-manager

# Check network connectivity
curl -I https://repo.multios.org

# View logs
tail -f /var/log/multios-package-manager.log
```

#### Repository Issues
```bash
# Test repository access
multios-pm sync --verbose

# Check repository configuration
cat /etc/multios-package-manager/repositories.toml

# Reset repository cache
multios-pm cleanup --clear-cache
```

#### Package Conflicts
```bash
# Check for conflicts
multios-pm verify --check-conflicts

# Resolve dependency issues
multios-pm install --force-deps package-name

# Manual dependency resolution
multios-pm depsolve package-name
```

### Debug Mode

Enable verbose logging:
```bash
export RUST_LOG=debug
multios-pm --verbose install package-name
```

Or use system logging:
```bash
journalctl -u multios-package-manager -f
```

### Performance Issues

#### Slow Package Operations
1. Check network connectivity
2. Increase concurrent download limit
3. Use mirrors closer to your location
4. Clear package cache

#### High Memory Usage
1. Limit concurrent operations
2. Adjust cache size limits
3. Enable delta updates
4. Regular cleanup

### Getting Help

#### Documentation
- User Guide: `/usr/share/doc/multios-package-manager/`
- API Reference: Available online
- Examples: `/usr/share/examples/multios-package-manager/`

#### Support Channels
- GitHub Issues: https://github.com/multios/package-manager/issues
- Documentation: https://docs.multios.org/package-manager
- Community Forum: https://community.multios.org

#### Log Collection
```bash
# Collect system information
multios-pm status --debug > debug-info.txt

# Collect logs
journalctl -u multios-package-manager > logs.txt

# Package information
multios-pm list --json > installed-packages.json
```

## Contributing

### Development Setup

1. **Fork the repository**
   ```bash
   git clone https://github.com/yourusername/package-manager.git
   cd package-manager
   ```

2. **Set up development environment**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Python dependencies
   pip3 install -r requirements-dev.txt
   
   # Set up git hooks
   pre-commit install
   ```

3. **Build and test**
   ```bash
   # Build all components
   ./scripts/build.sh
   
   # Run tests
   ./scripts/test.sh
   
   # Run integration tests
   ./scripts/test-integration.sh
   ```

### Code Standards

#### Rust Code
- Follow Rust standard formatting (`rustfmt`)
- Use `clippy` for linting
- Write comprehensive unit tests
- Document public APIs

#### Python Code
- Follow PEP 8 style guidelines
- Use type hints
- Write docstrings for public APIs
- Include unit tests

#### Documentation
- Update relevant documentation
- Include code examples
- Keep CHANGELOG.md updated
- Write clear commit messages

### Submitting Changes

1. **Create feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and test**
   ```bash
   # Make your changes
   # Add tests
   ./scripts/test.sh
   ```

3. **Commit changes**
   ```bash
   git add .
   git commit -m "feat: add new package manager feature"
   ```

4. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   # Create pull request on GitHub
   ```

### Release Process

1. **Version bumping**
   ```bash
   # Update version numbers
   ./scripts/bump-version.sh 1.2.3
   ```

2. **Testing**
   ```bash
   # Full test suite
   ./scripts/test-all.sh
   
   # Integration testing
   ./scripts/test-integration.sh
   ```

3. **Documentation**
   ```bash
   # Generate documentation
   ./scripts/generate-docs.sh
   ```

4. **Release**
   ```bash
   # Create release
   ./scripts/release.sh 1.2.3
   ```

### Community Guidelines

- Be respectful and welcoming
- Help review and test contributions
- Share knowledge and provide feedback
- Follow the code of conduct
- Report issues and security concerns responsibly

---

## License

The MultiOS Package Manager is licensed under the MIT License. See LICENSE file for details.

## Acknowledgments

- Rust community for excellent tooling
- Python packaging ecosystem
- Contributors and testers
- MultiOS development team

---

For more information, visit https://multios.org/package-manager