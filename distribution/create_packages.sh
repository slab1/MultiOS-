#!/bin/bash
# MultiOS Distribution Package Creator
# Version: 1.0.0
# Description: Create portable distribution packages for different platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

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

header() {
    echo -e "${PURPLE}$1${NC}"
}

# Configuration
VERSION="1.0.0"
RELEASE_DATE=$(date -u +%Y%m%d)
PACKAGE_NAME="multios"
DIST_DIR="./distributions"
TEMP_DIR="./temp_package"

# Create distributions directory
mkdir -p "$DIST_DIR"

# Clean previous builds
clean_build() {
    log "Cleaning previous builds..."
    rm -rf "$TEMP_DIR"
    mkdir -p "$TEMP_DIR"
}

# Create tarball distribution
create_tarball() {
    local arch=${1:-"universal"}
    local platform=${2:-"all"}
    
    header "Creating tarball distribution..."
    
    local package_name="$PACKAGE_NAME-v$VERSION-$arch-$platform-$RELEASE_DATE"
    local tarball_name="$package_name.tar.gz"
    local tarball_path="$DIST_DIR/$tarball_name"
    
    log "Package name: $package_name"
    log "Output: $tarball_path"
    
    # Copy distribution files
    rsync -av --exclude='.git' --exclude='checksums.sha256' --exclude='verify.sh' . "$TEMP_DIR/"
    
    # Create package-specific files
    cat > "$TEMP_DIR/PACKAGE_INFO.txt" <<EOF
MultiOS Distribution Package
============================

Package: $package_name
Version: $VERSION
Release Date: $RELEASE_DATE
Architecture: $arch
Platform: $platform

Contents:
- MultiOS Kernel (Hybrid Microkernel)
- Bootloader (Multi-stage)
- System Libraries
- Installation Scripts
- Documentation
- Development Tools
- Testing Framework

Quick Start:
1. Extract the archive: tar -xzf $tarball_name
2. Enter directory: cd $package_name
3. Run installer: ./install.sh

For more information, see README.md

MultiOS Team
https://multios.org
EOF
    
    # Create version info
    cat > "$TEMP_DIR/VERSION.txt" <<EOF
MultiOS Version Information
===========================

Distribution Version: $VERSION
Release Date: $RELEASE_DATE
Build Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Git Commit: $(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
Rust Version: $(rustc --version 2>/dev/null || echo "unknown")
Build Architecture: $(uname -m)

Supported Targets:
- x86_64-unknown-linux-gnu
- aarch64-unknown-none
- riscv64gc-unknown-none-elf

Installation Types:
- Desktop (GUI, multimedia, user-friendly)
- Server (optimized, secure, monitored)
- Embedded (IoT, minimal, edge computing)
- Development (tools, IDEs, debugging)

Features:
- Hybrid microkernel architecture
- Memory-safe Rust implementation
- Multi-architecture support
- Hardware abstraction layer
- Service-based architecture
- Educational focus
EOF
    
    # Add checksums for this specific package
    find "$TEMP_DIR" -type f -not -name "checksums.sha256" | xargs sha256sum > "$TEMP_DIR/package_checksums.sha256"
    
    # Create tarball
    cd "$TEMP_DIR"
    tar -czf "../$tarball_path" .
    cd ..
    
    # Generate checksums for the tarball
    cd "$DIST_DIR"
    sha256sum "$tarball_name" > "$tarball_name.sha256"
    cd ..
    
    log "Created: $tarball_path"
    
    # Show size
    local size=$(du -h "$tarball_path" | cut -f1)
    log "Package size: $size"
    
    return 0
}

# Create minimal development package
create_dev_package() {
    header "Creating development-only package..."
    
    local package_name="$PACKAGE_NAME-dev-v$VERSION-$RELEASE_DATE"
    local tarball_name="$package_name.tar.gz"
    local tarball_path="$DIST_DIR/$tarball_name"
    
    log "Package name: $package_name"
    log "Output: $tarball_path"
    
    # Create minimal structure
    mkdir -p "$TEMP_DIR"
    
    # Copy only development-related files
    cp -r kernel "$TEMP_DIR/"
    cp -r bootloader "$TEMP_DIR/"
    cp -r libraries "$TEMP_DIR/"
    cp -r documentation "$TEMP_DIR/"
    cp -r examples "$TEMP_DIR/"
    cp -r testing "$TEMP_DIR/"
    cp -r scripts "$TEMP_DIR/"
    cp install.sh "$TEMP_DIR/"
    cp verify.sh "$TEMP_DIR/"
    cp README.md "$TEMP_DIR/"
    cp LICENSE* "$TEMP_DIR/"
    cp Cargo.toml "$TEMP_DIR/"
    
    # Add development-specific info
    cat > "$TEMP_DIR/README_DEV.md" <<EOF
# MultiOS Development Package

This is a minimal package containing only the components needed for development:

- Kernel source code
- Bootloader source
- Libraries and crates
- Documentation
- Examples
- Testing framework
- Development scripts

Installation:
1. Install Rust toolchain
2. Install build dependencies
3. Run: ./install.sh --dev

For full system installation, use the complete distribution package.
EOF
    
    # Create tarball
    cd "$TEMP_DIR"
    tar -czf "../$tarball_path" .
    cd ..
    
    # Generate checksums
    cd "$DIST_DIR"
    sha256sum "$tarball_name" > "$tarball_name.sha256"
    cd ..
    
    log "Created: $tarball_path"
    local size=$(du -h "$tarball_path" | cut -f1)
    log "Package size: $size"
}

# Create ISO image for desktop (bootable)
create_iso_package() {
    header "Creating bootable ISO package (experimental)..."
    
    local package_name="$PACKAGE_NAME-desktop-v$VERSION-$RELEASE_DATE"
    local iso_name="$package_name.iso"
    local iso_path="$DIST_DIR/$iso_name"
    
    log "Package name: $package_name"
    log "Output: $iso_path"
    
    # This is experimental - creating a basic ISO structure
    # In a real implementation, you would use tools like genisoimage or xorriso
    
    mkdir -p "$TEMP_DIR/iso_root/boot"
    
    # Copy kernel and bootloader
    if [ -f "kernel/target/release/multios-kernel" ]; then
        cp "kernel/target/release/multios-kernel" "$TEMP_DIR/iso_root/boot/"
    fi
    
    # Copy installation scripts
    cp -r installation "$TEMP_DIR/iso_root/"
    cp README.md "$TEMP_DIR/iso_root/README.txt"
    
    # Create basic bootloader config
    cat > "$TEMP_DIR/iso_root/boot/grub.cfg" <<EOF
# MultiOS Boot Menu
set timeout=5
set default=0

menuentry "MultiOS Desktop" {
    linux /boot/multios-kernel
    initrd /boot/initrd.img
}

menuentry "MultiOS (Safe Mode)" {
    linux /boot/multios-kernel single
}

menuentry "Install MultiOS" {
    linux /boot/multios-kernel installer
}
EOF
    
    warning "ISO creation is experimental and requires additional setup"
    warning "Use tarball packages for reliable installation"
}

# Create Docker container package
create_docker_package() {
    header "Creating Docker container package..."
    
    local package_name="$PACKAGE_NAME-docker-v$VERSION-$RELEASE_DATE"
    local tarball_name="$package_name.tar"
    local tarball_path="$DIST_DIR/$tarball_name"
    
    log "Package name: $package_name"
    log "Output: $tarball_path"
    
    # Create Dockerfile
    cat > "$TEMP_DIR/Dockerfile" <<EOF
FROM ubuntu:22.04

LABEL description="MultiOS Development Environment"
LABEL version="$VERSION"
LABEL maintainer="MultiOS Team <team@multios.org>"

# Install dependencies
RUN apt-get update && apt-get install -y \\
    build-essential \\
    rustc \\
    cargo \\
    qemu-system-x86 \\
    git \\
    curl \\
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . /src
WORKDIR /src

# Build kernel
RUN cargo build --release

# Set up development environment
RUN echo 'alias multios-build="cargo build --release"' >> ~/.bashrc
RUN echo 'alias multios-test="cargo test"' >> ~/.bashrc

CMD ["/bin/bash"]
EOF
    
    # Create docker-compose.yml
    cat > "$TEMP_DIR/docker-compose.yml" <<EOF
version: '3.8'

services:
  multios-dev:
    build: .
    volumes:
      - ./:/workspace
    working_dir: /workspace
    stdin_open: true
    tty: true
    
  multios-qemu:
    build: .
    privileged: true
    volumes:
      - ./:/workspace
    working_dir: /workspace
    command: qemu-system-x86_64 -kernel kernel/target/release/multios-kernel -nographic
EOF
    
    # Create build script
    cat > "$TEMP_DIR/build-docker.sh" <<'EOF'
#!/bin/bash
docker build -t multios-dev .
docker run -it -v $(pwd):/workspace multios-dev
EOF
    chmod +x "$TEMP_DIR/build-docker.sh"
    
    # Create tarball
    cd "$TEMP_DIR"
    tar -cf "../$tarball_path" .
    cd ..
    
    # Generate checksums
    cd "$DIST_DIR"
    sha256sum "$tarball_name" > "$tarball_name.sha256"
    cd ..
    
    log "Created: $tarball_path"
    local size=$(du -h "$tarball_path" | cut -f1)
    log "Package size: $size"
}

# Generate package index
generate_package_index() {
    header "Generating package index..."
    
    local index_file="$DIST_DIR/PACKAGES.md"
    
    {
        echo "# MultiOS Distribution Packages"
        echo
        echo "**Release:** $VERSION ($RELEASE_DATE)"
        echo
        echo "## Available Packages"
        echo
        
        for tarball in "$DIST_DIR"/*.tar.gz; do
            if [ -f "$tarball" ]; then
                local basename=$(basename "$tarball")
                local checksum_file="$tarball.sha256"
                local checksum=""
                
                if [ -f "$checksum_file" ]; then
                    checksum=$(head -1 "$checksum_file" | awk '{print $1}')
                fi
                
                echo "### $basename"
                echo
                echo "**Size:** $(du -h "$tarball" | cut -f1)"
                echo "**SHA256:** \`$checksum\`"
                echo
                
                # Parse package name for description
                if [[ $basename == *"desktop"* ]]; then
                    echo "**Desktop Installation** - Full desktop environment with GUI support"
                elif [[ $basename == *"server"* ]]; then
                    echo "**Server Installation** - Optimized for server workloads"
                elif [[ $basename == *"embedded"* ]]; then
                    echo "**Embedded/IoT Installation** - Minimal footprint for IoT devices"
                elif [[ $basename == *"dev"* ]]; then
                    echo "**Development Package** - Minimal development environment"
                elif [[ $basename == *"docker"* ]]; then
                    echo "**Docker Package** - Containerized development environment"
                else
                    echo "**Universal Package** - Contains all installation types"
                fi
                echo
            fi
        done
        
        echo "## Installation"
        echo
        echo "Choose the package that best fits your needs and extract it:"
        echo
        echo "\`\`\`bash"
        echo "tar -xzf package-name.tar.gz"
        echo "cd package-directory"
        echo "./install.sh"
        echo "\`\`\`"
        echo
        echo "## Verification"
        echo
        echo "Verify package integrity:"
        echo
        echo "\`\`\`bash"
        echo "sha256sum -c package-name.tar.gz.sha256"
        echo "\`\`\`"
        echo
        echo "## Support"
        echo
        echo "- Documentation: https://docs.multios.org"
        echo "- Issues: https://github.com/multios/multios/issues"
        echo "- Community: https://discord.gg/multios"
        
    } > "$index_file"
    
    log "Package index created: $index_file"
}

# Show usage information
show_usage() {
    cat <<EOF
MultiOS Distribution Package Creator
====================================

Usage: $0 [OPTIONS]

Options:
    --all           Create all package types (default)
    --tarball       Create tarball distribution
    --dev           Create development-only package
    --iso           Create bootable ISO (experimental)
    --docker        Create Docker container package
    --clean         Clean build directory
    --index         Generate package index only
    --help          Show this help message

Examples:
    $0 --all                    # Create all packages
    $0 --tarball                # Create tarball only
    $0 --dev --docker          # Create dev and docker packages
    $0 --clean --all           # Clean and rebuild all packages

Package Types:
    tarball    - Universal tarball with all installation types
    dev        - Minimal development environment
    iso        - Bootable ISO (experimental)
    docker     - Docker container setup

EOF
}

# Main function
main() {
    echo "========================================"
    echo "  MultiOS Distribution Package Creator"
    echo "  Version: $VERSION"
    echo "========================================"
    echo
    
    # Parse arguments
    local create_all=true
    local create_tarball=false
    local create_dev=false
    local create_iso=false
    local create_docker=false
    local clean_only=false
    local index_only=false
    
    for arg in "$@"; do
        case $arg in
            --all)
                create_all=true
                create_tarball=true
                create_dev=true
                create_docker=true
                ;;
            --tarball)
                create_all=false
                create_tarball=true
                ;;
            --dev)
                create_all=false
                create_dev=true
                ;;
            --iso)
                create_all=false
                create_iso=true
                ;;
            --docker)
                create_all=false
                create_docker=true
                ;;
            --clean)
                clean_build
                clean_only=true
                ;;
            --index)
                generate_package_index
                index_only=true
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            *)
                warning "Unknown option: $arg"
                show_usage
                exit 1
                ;;
        esac
    done
    
    if [ "$clean_only" = true ]; then
        log "Clean completed. Exiting."
        exit 0
    fi
    
    if [ "$index_only" = true ]; then
        exit 0
    fi
    
    # Clean build directory
    clean_build
    
    # Get architecture info
    local arch=$(uname -m)
    case $arch in
        x86_64)
            ARCH_LABEL="x86_64"
            ;;
        aarch64|arm64)
            ARCH_LABEL="aarch64"
            ;;
        riscv64)
            ARCH_LABEL="riscv64"
            ;;
        *)
            ARCH_LABEL="unknown"
            ;;
    esac
    
    info "Build architecture: $ARCH_LABEL"
    
    # Create packages
    if [ "$create_tarball" = true ]; then
        create_tarball "$ARCH_LABEL" "universal"
    fi
    
    if [ "$create_dev" = true ]; then
        create_dev_package
    fi
    
    if [ "$create_iso" = true ]; then
        create_iso_package
    fi
    
    if [ "$create_docker" = true ]; then
        create_docker_package
    fi
    
    # Generate package index
    generate_package_index
    
    # Show summary
    header "Package Creation Complete!"
    echo
    echo "Generated packages:"
    ls -lh "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.tar "$DIST_DIR"/*.iso 2>/dev/null || true
    echo
    echo "Package index: $DIST_DIR/PACKAGES.md"
    echo
    
    # Cleanup
    rm -rf "$TEMP_DIR"
}

# Run main function
main "$@"