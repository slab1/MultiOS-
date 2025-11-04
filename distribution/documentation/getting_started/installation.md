# MultiOS Installation Guide

This guide provides detailed installation instructions for MultiOS on various platforms and environments.

## Installation Methods

MultiOS can be installed and used in several ways:

1. **Development Installation** - Build from source for development
2. **Pre-built Binary** - Download and run pre-compiled binaries
3. **Docker Installation** - Use containerized development environment
4. **Virtual Machine** - Install in QEMU or other virtual machines
5. **Bare Metal** - Install directly on hardware

## Development Installation (Recommended)

For developers who want to build and modify MultiOS.

### Prerequisites

#### Required Software

**Rust Toolchain (1.70+):**
```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install additional targets
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf  
rustup target add riscv64gc-unknown-none-elf

# Install additional tools
cargo install cargo-audit cargo-tarpaulin cross
```

**Build Tools:**

*Ubuntu/Debian:*
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
    gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu \
    doxygen graphviz \
    git cmake ninja-build
```

*Fedora/RHEL/CentOS:*
```bash
sudo dnf install -y \
    @development-tools \
    qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
    gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu \
    doxygen graphviz \
    git cmake ninja-build
```

*Arch Linux:*
```bash
sudo pacman -S --needed \
    base-devel \
    qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
    aarch64-linux-gnu-gcc riscv64-linux-gnu-gcc \
    doxygen graphviz \
    git cmake ninja
```

*macOS:*
```bash
# Install Homebrew if needed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install qemu cmake ninja git

# Install cross-compilers (via Homebrew)
brew install aarch64-elf-gcc riscv64-elf-gcc
```

*Windows:*
```powershell
# Install Chocolatey if needed
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install dependencies
choco install -y visualstudio2019buildtools visualstudio2019-workload-vctools
choco install -y qemu git cmake ninja

# For WSL2 (recommended)
wsl --install
```

### Download and Build

```bash
# Clone the repository
git clone https://github.com/multios/multios.git
cd multios

# Verify prerequisites
make check-prereqs

# Build for x86_64 (default)
make build-x86_64

# Or build for all architectures
make build-all

# Run tests to verify installation
make test-x86_64
```

### Verify Installation

```bash
# Check that binaries were created
ls -la target/*/unknown-none-elf/release/multios

# Run a quick test
make run-x86_64-test
```

## Pre-built Binary Installation

For users who want to run MultiOS without building from source.

### Download Binaries

Download pre-compiled MultiOS images:

```bash
# Create directories
mkdir -p ~/multios/binaries
cd ~/multios/binaries

# Download latest release (replace with actual URLs)
wget https://github.com/multios/multios/releases/latest/download/multios-x86_64.bin
wget https://github.com/multios/multios/releases/latest/download/multios-aarch64.bin
wget https://github.com/multios/multios/releases/latest/download/multios-riscv64.bin

# Make executable
chmod +x multios-*.bin
```

### Running Pre-built Binaries

```bash
# Run x86_64 version
qemu-system-x86_64 -kernel multios-x86_64.bin -m 256M -nographic

# Run ARM64 version
qemu-system-aarch64 -machine virt -cpu cortex-a57 \
    -kernel multios-aarch64.bin -m 256M -nographic

# Run RISC-V version  
qemu-system-riscv64 -machine virt \
    -kernel multios-riscv64.bin -m 256M -nographic
```

## Docker Installation

Use Docker for a consistent development environment.

### Install Docker

*Ubuntu/Debian:*
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

*macOS:*
```bash
# Install Docker Desktop
brew install --cask docker
```

*Windows:*
```bash
# Install Docker Desktop for Windows
# Download from: https://docs.docker.com/desktop/windows/install/
```

### Build Docker Image

```bash
# Clone repository
git clone https://github.com/multios/multios.git
cd multios

# Build Docker image
docker build -t multios-dev:latest .

# Or use Docker Compose
docker-compose build
```

### Run in Docker

```bash
# Interactive development
docker run -it --rm -v $(pwd):/workspace multios-dev:latest bash

# Run specific build
docker run --rm -v $(pwd):/workspace multios-dev:latest make build-x86_64

# Run tests
docker run --rm -v $(pwd):/workspace multios-dev:latest make test-x86_64
```

## Virtual Machine Installation

Install MultiOS in virtual machines for safe testing.

### QEMU Installation (Recommended)

```bash
# Create disk image
qemu-img create -f qcow2 multios.qcow2 2G

# Install MultiOS (replace with actual installation process)
# This is a simplified example - actual installation may vary

# Run with disk
qemu-system-x86_64 \
    -kernel multios-x86_64.bin \
    -m 512M \
    -drive file=multios.qcow2,format=qcow2 \
    -nographic
```

### VMware Installation

1. Create new virtual machine
2. Select "Other" as guest OS type
3. Configure:
   - Memory: 512MB minimum
   - Disk: 2GB
   - Network: NAT
4. Boot from MultiOS image
5. Follow installation prompts

### VirtualBox Installation

```bash
# Create VM
VBoxManage createvm --name "MultiOS" --ostype "Other" --register

# Configure VM
VBoxManage modifyvm "MultiOS" --memory 512 --vram 16
VBoxManage modifyvm "MultiOS" --boot1 disk --boot2 dvd --boot3 none

# Add disk controller
VBoxManage storagectl "MultiOS" --name "SATA Controller" --add sata
VBoxManage createhd --filename multios.vdi --size 2048 --format VDI

# Attach disk
VBoxManage storageattach "MultiOS" --storagectl "SATA Controller" --port 0 --device 0 --type hdd --medium multios.vdi

# Install MultiOS
VBoxManage storageattach "MultiOS" --storagectl "SATA Controller" --port 1 --device 0 --type dvddrive --medium /path/to/multios.iso

# Start VM
VBoxManage startvm "MultiOS" --type gui
```

## Bare Metal Installation

Install MultiOS directly on hardware (advanced users only).

### System Requirements

- **CPU**: x86_64, ARM64, or RISC-V processor with 64-bit support
- **Memory**: 512MB minimum, 1GB recommended
- **Storage**: 1GB available space
- **Boot**: UEFI or Legacy BIOS support

### Installation Process

#### 1. Prepare Installation Media

**USB Installation:**
```bash
# Create bootable USB (Linux/macOS)
sudo dd if=multios-x86_64.bin of=/dev/sdX bs=1M status=progress
sync

# Windows (use Rufus or similar tool)
# 1. Download Rufus from https://rufus.ie/
# 2. Select MultiOS image
# 3. Write to USB device
```

**PXE Network Boot:**
```bash
# Configure TFTP server
sudo cp multios-x86_64.bin /var/lib/tftpboot/
sudo systemctl restart tftpd-hpa

# Configure DHCP (dnsmasq example)
echo "dhcp-boot=multios-x86_64.bin" | sudo tee -a /etc/dnsmasq.conf
sudo systemctl restart dnsmasq
```

#### 2. Boot from Installation Media

1. **Boot computer** from USB or network
2. **Select MultiOS** from boot menu
3. **Run installer** (if provided)
4. **Partition disk** as needed
5. **Install bootloader** (GRUB)
6. **Reboot** into installed system

#### 3. Post-Installation

```bash
# Verify installation
systemctl status multios
multios-info

# Update system (if package manager available)
multios-update

# Install additional packages
multios-install <package-name>
```

## Multi-Platform Development

For development across multiple architectures.

### Cross-Compilation Setup

```bash
# Install Rust targets
rustup target add x86_64-unknown-none-elf
rustup target add aarch64-unknown-none-elf
rustup target add riscv64gc-unknown-none-elf

# Install cross-compilers
sudo apt-get install gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu

# Test cross-compilation
make build-all
make test-all
```

### Multi-Architecture Testing

```bash
# Build for all architectures
make build-all

# Test on QEMU emulators
make test-qemu-all

# Run automated multi-arch tests
make test-cross-platform
```

## IDE Integration

Set up your preferred IDE for MultiOS development.

### Visual Studio Code

```bash
# Install VS Code
sudo snap install code --classic

# Install Rust extension
code --install-extension rust-lang.rust-analyzer

# Open project
cd multios
code .

# Accept recommended extensions
# Configure tasks and launch.json as needed
```

### Vim/Neovim

```rust
# Add to ~/.vimrc or ~/.config/nvim/init.vim
call plug#begin('~/.vim/plugged')

Plug 'rust-lang/rust.vim'
Plug 'fannheyward/coc-rust-analyzer'
call plug#end()

" Rust specific settings
autocmd FileType rust setlocal tabstop=4 shiftwidth=4 expandtab
```

### Emacs

```elisp
;; Add to ~/.emacs.d/init.el
(use-package rust-mode
  :ensure t
  :config
  (define-key rust-mode-map (kbd "C-c C-c") 'rust-compile)
  (define-key rust-mode-map (kbd "C-c C-t") 'rust-test))
```

## Configuration

### Environment Variables

```bash
# Add to ~/.bashrc or ~/.zshrc
export MULTIOS_HOME="$HOME/multios"
export PATH="$MULTIOS_HOME/scripts:$PATH"
export QEMU_AUDIO_DRV=none  # Disable audio in QEMU
```

### Git Configuration

```bash
# Configure Git for MultiOS development
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"

# Set up useful aliases
git config --global alias.st status
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.ci commit
```

### Make Configuration

Create `~/.multios.mk`:

```makefile
# Personal MultiOS configuration
ARCH ?= x86_64
JOBS ?= $(shell nproc)
VERBOSE ?= 0
PROFILE ?= release

# QEMU settings
QEMU_MEMORY ?= 256M
QEMU_CORES ?= 1

# Development settings
RUST_LOG ?= info
RUST_BACKTRACE ?= 1
```

## Troubleshooting Installation

### Common Issues

#### Rust Not Found
```bash
# Check Rust installation
which rustc
rustc --version

# Reinstall if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Missing Dependencies
```bash
# Check system dependencies
make check-deps

# Install missing dependencies
make install-deps
```

#### QEMU Issues
```bash
# Verify QEMU installation
qemu-system-x86_64 --version
qemu-system-aarch64 --version
qemu-system-riscv64 --version

# Test QEMU basic functionality
qemu-system-x86_64 --help
```

#### Permission Issues
```bash
# Add user to necessary groups
sudo usermod -a -G kvm,disk,dialout $USER

# Log out and back in for group changes to take effect
# Or use: newgrp kvm
```

#### Build Failures
```bash
# Clean build artifacts
make clean
cargo clean

# Clear Rust cache
rustup self uninstall
rustup install stable

# Reinstall with latest version
rustup update stable
```

### Getting Help

1. **Check the logs**: `make log-build` for detailed build logs
2. **Run diagnostics**: `make doctor` to check system health
3. **Search issues**: [GitHub Issues](https://github.com/multios/multios/issues)
4. **Ask for help**: [GitHub Discussions](https://github.com/multios/multios/discussions)

## Advanced Installation

### Automated Installation Scripts

```bash
# Quick setup script
curl -fsSL https://raw.githubusercontent.com/multios/multios/main/scripts/install.sh | bash

# Development setup script
curl -fsSL https://raw.githubusercontent.com/multios/multios/main/scripts/setup-dev.sh | bash
```

### Custom Installation

```bash
# Install to custom directory
PREFIX=/opt/multios make install

# Install specific architecture only
make install-x86_64

# Install with custom configuration
make install CONFIG=custom.toml
```

### Package Manager Integration

**APT (Debian/Ubuntu):**
```bash
# Add MultiOS repository
echo "deb [signed-by=/etc/apt/keyrings/multios.gpg] https://apt.multios.org stable main" | sudo tee /etc/apt/sources.list.d/multios.list

# Install
sudo apt update
sudo apt install multios
```

**Homebrew (macOS):**
```bash
# Add tap (when available)
brew tap multios/multios
brew install multios
```

## Verification

After installation, verify everything works:

```bash
# Check MultiOS version
multios --version

# Run system test
make test-system

# Check all architectures
make test-all-archs

# Verify development environment
make doctor
```

## Next Steps

After successful installation:

1. **Read the Quick Start Guide** for basic usage
2. **Explore the tutorials** to learn more
3. **Join the community** for support and discussions
4. **Contribute back** to help improve MultiOS

---

**Up**: [Getting Started README](README.md)  
**Next**: [System Requirements](requirements.md)  
**Related**: [Quick Start Guide](README.md) | [Development Guide](../developer/README.md)