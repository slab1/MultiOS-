#!/bin/bash
# MultiOS Development Installation Script
# Version: 1.0.0
# Description: Install MultiOS development environment and build tools

set -e

# Configuration
INSTALL_DIR="$HOME/.local/multios"
DEV_DIR="$HOME/multios-dev"
TOOLS_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/multios"
VERSION="1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Logging
LOG_FILE="$HOME/.local/share/multios/dev_install.log"
mkdir -p "$(dirname "$LOG_FILE")"
exec 1> >(tee -a "$LOG_FILE") 2>&1

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}" >&2
}

warning() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

debug() {
    echo -e "${PURPLE}[DEBUG] $1${NC}"
}

# Check system requirements for development
check_requirements() {
    log "Checking development system requirements..."
    
    # Check architecture
    ARCH=$(uname -m)
    case $ARCH in
        x86_64|aarch64|riscv64)
            info "Architecture: $ARCH - Supported"
            ;;
        *)
            error "Architecture $ARCH may have limited support"
            ;;
    esac
    
    # Check available space (minimum 5GB for development)
    AVAILABLE_SPACE=$(df ~ | awk 'NR==2 {print $4}')
    REQUIRED_SPACE=5242880  # 5GB in KB
    
    if [ "$AVAILABLE_SPACE" -lt "$REQUIRED_SPACE" ]; then
        error "Insufficient disk space. Required: 5GB, Available: $((AVAILABLE_SPACE/1024/1024))GB"
        exit 1
    fi
    
    # Check memory (minimum 8GB for development)
    TOTAL_MEM=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    REQUIRED_MEM=8388608  # 8GB in KB
    
    if [ "$TOTAL_MEM" -lt "$REQUIRED_MEM" ]; then
        warning "Low memory detected. Recommended: 16GB+, Available: $((TOTAL_MEM/1024))MB"
    fi
    
    info "Development system requirements check passed"
}

# Install Rust toolchain
install_rust() {
    log "Installing/Verifying Rust toolchain..."
    
    if command -v cargo >/dev/null 2>&1; then
        RUST_VERSION=$(cargo --version | cut -d' ' -f2)
        info "Rust already installed: $RUST_VERSION"
        
        # Check if version is recent enough
        if rustup show >/dev/null 2>&1; then
            info "Using rustup for version management"
            rustup update
            rustup default stable
        fi
    else
        info "Installing Rust toolchain..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
        
        # Set default toolchain
        rustup default stable
        rustup target add x86_64-unknown-linux-gnu
        rustup target add aarch64-unknown-none
        rustup target add riscv64gc-unknown-none-elf
    fi
    
    # Install additional Rust tools
    info "Installing additional Rust development tools..."
    cargo install cargo-watch      # Auto-rebuild on changes
    cargo install cargo-audit      # Security vulnerability checking
    cargo install cargo-outdated   # Check for outdated dependencies
    cargo install mdbook           # Documentation generation
    cargo install bindgen          # C FFI bindings
    
    info "Rust toolchain installation completed"
}

# Install build dependencies
install_build_deps() {
    log "Installing build dependencies..."
    
    # Detect package manager
    if command -v apt-get >/dev/null 2>&1; then
        PKG_MANAGER="apt"
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            gcc \
            g++ \
            make \
            cmake \
            ninja-build \
            pkg-config \
            libssl-dev \
            libusb-1.0-0-dev \
            qemu-system-x86 \
            qemu-system-arm \
            qemu-system-riscv64 \
            gdb-multiarch \
            lld \
            clang \
            llvm \
            doxygen \
            graphviz \
            git \
            curl \
            wget \
            vim \
            htop \
            tree \
            jq \
            shellcheck
    elif command -v yum >/dev/null 2>&1; then
        PKG_MANAGER="yum"
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y \
            gcc \
            gcc-c++ \
            make \
            cmake \
            ninja-build \
            pkgconfig \
            openssl-devel \
            libusbx-devel \
            qemu-kvm \
            qemu-img \
            gdb \
            lld \
            clang \
            llvm \
            doxygen \
            graphviz \
            git \
            curl \
            wget \
            vim \
            htop \
            tree \
            jq
    elif command -v brew >/dev/null 2>&1; then
        PKG_MANAGER="brew"
        brew install \
            gcc \
            make \
            cmake \
            ninja \
            pkg-config \
            openssl \
            qemu \
            gdb \
            lld \
            clang \
            llvm \
            doxygen \
            graphviz \
            git \
            curl \
            wget \
            vim \
            htop \
            tree \
            jq \
            shellcheck
    else
        warning "Unknown package manager. Please install build dependencies manually:"
        warning "  - GCC/Clang compiler"
        warning "  - Make/CMake/Ninja"
        warning "  - QEMU for testing"
        warning "  - GDB debugger"
        warning "  - Additional development tools"
    fi
    
    info "Build dependencies installation completed"
}

# Create development directory structure
create_dev_structure() {
    log "Creating development directory structure..."
    
    # Create main directories
    mkdir -p "$INSTALL_DIR"/{bin,lib,docs,examples}
    mkdir -p "$DEV_DIR"/{src,target,docs,tests,tools}
    mkdir -p "$TOOLS_DIR"
    mkdir -p "$CONFIG_DIR"/{profiles,templates}
    mkdir -p ~/.{cargo,rustup,cargo-registry,cargo-git}
    
    # Create Rust configuration
    cat > ~/.cargo/config.toml <<EOF
# MultiOS Development Cargo Configuration

[target.x86_64-unknown-linux-gnu]
runner = "qemu-system-x86_64"

[target.aarch64-unknown-none]
runner = "qemu-system-aarch64 -machine virt -cpu cortex-a57"

[target.riscv64gc-unknown-none-elf]
runner = "qemu-system-riscv64 -machine virt"

[build]
target = "x86_64-unknown-linux-gnu"

[term]
progress-bar = true
color = "always"
EOF
    
    # Create Rust toolchain file
    cat > rust-toolchain.toml <<EOF
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-none",
    "riscv64gc-unknown-none-elf",
]
EOF
    
    info "Development directory structure created"
}

# Install development tools
install_dev_tools() {
    log "Installing development tools..."
    
    # Copy development scripts
    if [ -d "./scripts" ]; then
        cp -r ./scripts/* "$TOOLS_DIR/"
        chmod +x "$TOOLS_DIR"/*.sh 2>/dev/null || true
    fi
    
    # Install testing framework
    if [ -d "./testing" ]; then
        cp -r ./testing/* "$DEV_DIR/tests/"
    fi
    
    # Install example projects
    if [ -d "./examples" ]; then
        cp -r ./examples/* "$DEV_DIR/examples/"
    fi
    
    # Install documentation
    if [ -d "./documentation" ]; then
        cp -r ./documentation/* "$INSTALL_DIR/docs/"
    fi
    
    # Install debugging tools
    mkdir -p "$DEV_DIR/tools"
    
    # Create debug scripts
    cat > "$TOOLS_DIR/multios-debug.sh" <<'EOF'
#!/bin/bash
# MultiOS Debug Helper Script

TARGET=${1:-"x86_64"}
QEMU_ARCH=""

case $TARGET in
    "x86_64"|"x86")
        QEMU_ARCH="qemu-system-x86_64"
        MEMORY="512M"
        ;;
    "aarch64"|"arm")
        QEMU_ARCH="qemu-system-aarch64"
        MACHINE="virt"
        MEMORY="512M"
        ;;
    "riscv64"|"riscv")
        QEMU_ARCH="qemu-system-riscv64"
        MACHINE="virt"
        MEMORY="512M"
        ;;
    *)
        echo "Unknown target: $TARGET"
        echo "Supported targets: x86_64, aarch64, riscv64"
        exit 1
        ;;
esac

# Build the kernel
cd kernel
cargo build --target $TARGET-unknown-none

# Start QEMU with GDB
$QEMU_ARCH \
    -kernel target/$TARGET-unknown-none/release/multios-kernel \
    -m $MEMORY \
    -smp 1 \
    -S -gdb tcp::1234 \
    -nographic \
    $@
EOF
    chmod +x "$TOOLS_DIR/multios-debug.sh"
    
    # Create build script
    cat > "$TOOLS_DIR/multios-build.sh" <<'EOF'
#!/bin/bash
# MultiOS Build Script

set -e

TARGET=${1:-"x86_64"}
FEATURES=${2:-""}

echo "Building MultiOS for target: $TARGET"
echo "Features: $FEATURES"

cd kernel

# Clean previous builds
cargo clean

# Build with optional features
if [ -n "$FEATURES" ]; then
    cargo build --target $TARGET-unknown-none --features "$FEATURES"
else
    cargo build --target $TARGET-unknown-none
fi

# Run tests
echo "Running tests..."
cargo test --target $TARGET-unknown-none

echo "Build completed successfully"
EOF
    chmod +x "$TOOLS_DIR/multios-build.sh"
    
    # Create test script
    cat > "$TOOLS_DIR/multios-test.sh" <<'EOF'
#!/bin/bash
# MultiOS Test Script

set -e

echo "Running MultiOS test suite..."

cd kernel

# Run unit tests
echo "Running unit tests..."
cargo test --lib

# Run integration tests
echo "Running integration tests..."
cargo test --test integration

# Run documentation tests
echo "Running documentation tests..."
cargo test --doc

# Run benchmarks (if available)
if cargo test --bench --no-run 2>/dev/null; then
    echo "Running benchmarks..."
    cargo bench
fi

echo "All tests completed"
EOF
    chmod +x "$TOOLS_DIR/multios-test.sh"
    
    info "Development tools installed"
}

# Create IDE configuration
create_ide_config() {
    log "Creating IDE configuration files..."
    
    # VS Code configuration
    mkdir -p ~/.vscode
    cat > ~/.vscode/settings.json <<EOF
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.lens.enabled": true,
    "rust-analyzer.inlayHints.enabled": true,
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.procMacro.enable": true,
    "files.associations": {
        "*.rs": "rust",
        "*.toml": "toml"
    },
    "editor.formatOnSave": true,
    "editor.rustCodeLens.enabled": true,
    "files.exclude": {
        "**/target": true,
        "**/.cargo": true
    }
}
EOF
    
    # VS Code tasks
    cat > ~/.vscode/tasks.json <<EOF
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Build",
            "type": "shell",
            "command": "multios-build.sh",
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Test",
            "type": "shell",
            "command": "multios-test.sh",
            "group": "test",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Debug",
            "type": "shell",
            "command": "multios-debug.sh",
            "group": "debug",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "shared"
            }
        }
    ]
}
EOF
    
    # GDB configuration
    mkdir -p ~/.gdb
    cat > ~/.gdb/multios-gdb <<EOF
# MultiOS GDB Configuration
# Usage: gdb ./kernel/target/release/multios-kernel

# Set architecture
set architecture i386:x86-64

# Load symbols
file \$1

# Set breakpoints
break main
break panic_handler

# Set display
display/xi \$pc
display \$rax
display \$rbx
display \$rsp

# Set pagination
set pagination off

# Connect to QEMU (if running with -s -S)
# target remote localhost:1234

# Show all settings
show
EOF
    
    # Vim configuration
    cat > ~/.vimrc <<EOF
" MultiOS Development Vim Configuration

" Basic settings
set number
set relativenumber
set tabstop=4
set shiftwidth=4
set expandtab
set smartindent
set hlsearch
set incsearch
set ignorecase
set smartcase

" Rust-specific settings
autocmd BufNewFile,BufRead *.rs setlocal filetype=rust
autocmd FileType rust setlocal tabstop=4 shiftwidth=4 expandtab

" Plugin management
call plug#begin('~/.vim/plugged')

" Rust language support
Plug 'rust-lang/rust.vim'
Plug 'racer-rust/vim-racer'

" General development
Plug 'scrooloose/nerdtree'
Plug 'ctrlpvim/ctrlp.vim'
Plug 'tpope/vim-fugitive'
Plug 'dense-analysis/ale'

call plug#end()

" NERDTree toggle
nnoremap <C-n> :NERDTreeToggle<CR>

" ALE linting
let g:ale_linters = {'rust': ['cargo', 'rls']}
let g:ale_fixers = {'rust': ['rustfmt']}

" Rust-specific shortcuts
nnoremap <Leader>rf :RustFmt<CR>
nnoremap <Leader>rc :RustRun<CR>
nnoremap <Leader>rt :RustTest<CR>
EOF
    
    info "IDE configuration files created"
}

# Create development documentation
create_dev_docs() {
    log "Creating development documentation..."
    
    mkdir -p "$INSTALL_DIR/docs/development"
    
    # Create development guide
    cat > "$INSTALL_DIR/docs/development/README.md" <<EOF
# MultiOS Development Guide

## Quick Start

### Building MultiOS
\`\`\`bash
# Build for x86_64
multios-build.sh x86_64

# Build for ARM64
multios-build.sh aarch64

# Build with specific features
multios-build.sh x86_64 "networking graphics"
\`\`\`

### Running Tests
\`\`\`bash
multios-test.sh
\`\`\`

### Debugging
\`\`\`bash
# Start debug session
multios-debug.sh x86_64

# In another terminal, connect GDB
gdb -x ~/.gdb/multios-gdb ./kernel/target/release/multios-kernel
\`\`\`

## Development Workflow

### 1. Setting up the Environment
- Install Rust toolchain
- Install build dependencies
- Configure IDE/editor

### 2. Building and Testing
- Use provided build scripts
- Run full test suite
- Check code coverage

### 3. Debugging
- Use QEMU for emulation
- Configure GDB for debugging
- Use logging for tracing

### 4. Contributing
- Follow coding standards
- Write tests for new features
- Update documentation

## Architecture Overview

MultiOS is built with:
- Rust for safety and performance
- Modular design for portability
- HAL for hardware abstraction
- Service-based architecture

## Target Platforms

### Supported Platforms
- x86_64 (Desktop/Server)
- ARM64 (Mobile/Embedded)
- RISC-V (Educational)

### Adding New Platforms
1. Implement HAL for target architecture
2. Add target configuration
3. Update build scripts
4. Add testing support

## Testing Strategy

### Test Types
- Unit tests (individual components)
- Integration tests (system components)
- Integration tests (cross-platform)
- Benchmarks (performance)

### Testing Tools
- Rust testing framework
- QEMU for emulation
- Custom testing utilities

## Debugging Guide

### Common Issues
- Stack overflow
- Memory leaks
- Race conditions
- Hardware interaction

### Debug Tools
- GDB debugger
- QEMU monitor
- Logging framework
- Memory analysis

## Resources

### Documentation
- [API Documentation](api/)
- [Architecture Guide](architecture/)
- [Contributing Guide](contributing/)

### Tools
- [Rust Book](https://doc.rust-lang.org/book/)
- [OSDev Wiki](https://wiki.osdev.org/)
- [QEMU Documentation](https://www.qemu.org/docs/)

### Community
- GitHub Issues
- Discord Server
- Mailing List
EOF
    
    # Create contribution guidelines
    cat > "$INSTALL_DIR/docs/development/CONTRIBUTING.md" <<EOF
# Contributing to MultiOS

## Code Style

### Rust Guidelines
- Follow [Rust Style Guide](https://github.com/rust-lang/style-team/blob/master/guide/guide.md)
- Use \`rustfmt\` for formatting
- Use \`clippy\` for linting
- Write idiomatic Rust code

### Documentation
- Document all public APIs
- Include examples in doc comments
- Update README and guides
- Add inline comments for complex logic

### Testing
- Write tests for all public functions
- Include integration tests
- Test edge cases
- Maintain test coverage >80%

## Development Process

### 1. Fork and Clone
\`\`\`bash
git clone https://github.com/your-username/multios.git
cd multios
git remote add upstream https://github.com/multios/multios.git
\`\`\`

### 2. Create Feature Branch
\`\`\`bash
git checkout -b feature/your-feature-name
\`\`\`

### 3. Make Changes
- Write code following style guidelines
- Add tests for new functionality
- Update documentation
- Run full test suite

### 4. Submit PR
- Push branch to your fork
- Create pull request
- Fill out PR template
- Address review comments

## Pull Request Guidelines

### PR Title
Use clear, descriptive titles:
- \`feat:\` for new features
- \`fix:\` for bug fixes
- \`docs:\` for documentation
- \`refactor:\` for refactoring

### PR Description
- Describe what and why
- Include screenshots (if applicable)
- Link related issues
- List breaking changes

### Review Process
- At least one reviewer required
- All CI checks must pass
- Address all review comments
- Update tests if needed

## Issue Guidelines

### Bug Reports
Use the bug report template and include:
- Environment information
- Reproduction steps
- Expected vs actual behavior
- Relevant logs or screenshots

### Feature Requests
Use the feature request template and include:
- Problem statement
- Proposed solution
- Alternative solutions
- Additional context

## Communication

### Channels
- GitHub Issues (bug reports)
- GitHub Discussions (questions)
- Discord (real-time chat)
- Mailing List (announcements)

### Guidelines
- Be respectful and professional
- Search before posting
- Provide complete information
- Follow community guidelines
EOF
    
    info "Development documentation created"
}

# Run development environment tests
run_dev_tests() {
    log "Running development environment tests..."
    
    # Test Rust installation
    if command -v cargo >/dev/null 2>&1; then
        info "Rust/Cargo test: PASSED"
    else
        error "Rust/Cargo test: FAILED"
        return 1
    fi
    
    # Test build tools
    if command -v gcc >/dev/null 2>&1 && command -v make >/dev/null 2>&1; then
        info "Build tools test: PASSED"
    else
        warning "Build tools test: WARNING"
    fi
    
    # Test QEMU installation
    if command -v qemu-system-x86_64 >/dev/null 2>&1; then
        info "QEMU test: PASSED"
    else
        warning "QEMU test: WARNING"
    fi
    
    # Test development scripts
    if [ -x "$TOOLS_DIR/multios-build.sh" ]; then
        info "Build scripts test: PASSED"
    else
        error "Build scripts test: FAILED"
        return 1
    fi
    
    # Test configuration
    if [ -f ~/.cargo/config.toml ]; then
        info "Cargo configuration test: PASSED"
    else
        error "Cargo configuration test: FAILED"
        return 1
    fi
    
    info "Development environment tests completed"
}

# Print development installation summary
print_summary() {
    echo
    log "MultiOS Development Environment Installation Complete!"
    echo
    echo "Development Environment Summary:"
    echo "  Version: $VERSION"
    echo "  Install Directory: $INSTALL_DIR"
    echo "  Development Directory: $DEV_DIR"
    echo "  Tools Directory: $TOOLS_DIR"
    echo "  Configuration: $CONFIG_DIR"
    echo "  Log File: $LOG_FILE"
    echo
    echo "Development Tools Installed:"
    echo "  - Rust toolchain and cargo"
    echo "  - Build dependencies"
    echo "  - QEMU for testing"
    echo "  - GDB debugger"
    echo "  - IDE configurations (VS Code, Vim)"
    echo "  - Development scripts"
    echo "  - Documentation"
    echo
    echo "Available Commands:"
    echo "  multios-build.sh [target] [features] - Build MultiOS"
    echo "  multios-test.sh                     - Run tests"
    echo "  multios-debug.sh [target]           - Debug MultiOS"
    echo
    echo "Quick Start:"
    echo "  1. cd $DEV_DIR"
    echo "  2. multios-build.sh x86_64"
    echo "  3. multios-debug.sh x86_64"
    echo
    echo "IDE Setup:"
    echo "  - VS Code: Open $DEV_DIR in VS Code"
    echo "  - Vim: Open files in Vim (configs pre-loaded)"
    echo
    echo "Documentation:"
    echo "  - Development Guide: $INSTALL_DIR/docs/development/README.md"
    echo "  - Contributing: $INSTALL_DIR/docs/development/CONTRIBUTING.md"
    echo "  - API Docs: $INSTALL_DIR/docs/api/"
    echo
}

# Main installation function
main() {
    echo "========================================"
    echo "  MultiOS Development Environment"
    echo "  Installation Script"
    echo "  Version: $VERSION"
    echo "========================================"
    echo
    
    check_requirements
    install_rust
    install_build_deps
    create_dev_structure
    install_dev_tools
    create_ide_config
    create_dev_docs
    run_dev_tests
    print_summary
}

# Run main function
main "$@"