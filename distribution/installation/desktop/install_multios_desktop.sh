#!/bin/bash
# MultiOS Desktop Installation Script
# Version: 1.0.0
# Description: Install MultiOS for desktop/laptop use

set -e

# Configuration
INSTALL_DIR="/usr/local/multios"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/multios"
KERNEL_DIR="/usr/local/bin"
VERSION="1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging
LOG_FILE="/var/log/multios_install.log"
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

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        error "This script should not be run as root for security reasons"
        error "Please run as a regular user with sudo privileges"
        exit 1
    fi
    
    if ! sudo -n true 2>/dev/null; then
        error "This script requires sudo privileges"
        echo "Please run: sudo visudo"
        echo "Add: $USER ALL=(ALL) NOPASSWD: ALL"
        exit 1
    fi
}

# Check system requirements
check_requirements() {
    log "Checking system requirements..."
    
    # Check architecture
    ARCH=$(uname -m)
    case $ARCH in
        x86_64|aarch64|riscv64)
            info "Architecture: $ARCH - Supported"
            ;;
        *)
            error "Architecture $ARCH is not supported"
            exit 1
            ;;
    esac
    
    # Check available space (minimum 2GB)
    AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
    REQUIRED_SPACE=2097152  # 2GB in KB
    
    if [ "$AVAILABLE_SPACE" -lt "$REQUIRED_SPACE" ]; then
        error "Insufficient disk space. Required: 2GB, Available: $((AVAILABLE_SPACE/1024/1024))GB"
        exit 1
    fi
    
    # Check memory (minimum 2GB)
    TOTAL_MEM=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    REQUIRED_MEM=2097152  # 2GB in KB
    
    if [ "$TOTAL_MEM" -lt "$REQUIRED_MEM" ]; then
        warning "Low memory detected. Recommended: 4GB minimum, Available: $((TOTAL_MEM/1024))MB"
    fi
    
    info "System requirements check passed"
}

# Create necessary directories
create_directories() {
    log "Creating directory structure..."
    
    sudo mkdir -p "$INSTALL_DIR"/{bin,lib,config,logs,data}
    sudo mkdir -p "$CONFIG_DIR"/{profiles,modules,services}
    sudo mkdir -p "$KERNEL_DIR"
    
    # Set permissions
    sudo chown -R $USER:$USER "$INSTALL_DIR" "$CONFIG_DIR"
    sudo chmod 755 "$INSTALL_DIR" "$CONFIG_DIR" "$KERNEL_DIR"
    
    info "Directory structure created"
}

# Install kernel
install_kernel() {
    log "Installing MultiOS kernel..."
    
    # Copy kernel binary
    if [ -f "./kernel/target/release/multios-kernel" ]; then
        sudo cp "./kernel/target/release/multios-kernel" "$KERNEL_DIR/"
    elif [ -f "./kernel/target/debug/multios-kernel" ]; then
        sudo cp "./kernel/target/debug/multios-kernel" "$KERNEL_DIR/"
    else
        warning "Kernel binary not found. Building from source..."
        cd kernel
        cargo build --release
        sudo cp "./target/release/multios-kernel" "$KERNEL_DIR/"
        cd ..
    fi
    
    sudo chmod +x "$KERNEL_DIR/multios-kernel"
    info "Kernel installed successfully"
}

# Install bootloader
install_bootloader() {
    log "Installing MultiOS bootloader..."
    
    if [ -d "./bootloader" ]; then
        sudo cp -r ./bootloader/* "$INSTALL_DIR/bootloader/"
        
        # Install systemd service for bootloader updates
        sudo tee "$SYSTEMD_DIR/multios-bootloader.service" > /dev/null <<EOF
[Unit]
Description=MultiOS Bootloader
After=network.target

[Service]
Type=oneshot
ExecStart=$INSTALL_DIR/bootloader/update-bootloader.sh
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF
        sudo systemctl enable multios-bootloader.service
    fi
}

# Install libraries and modules
install_libraries() {
    log "Installing MultiOS libraries and modules..."
    
    # Copy kernel library
    sudo cp -r ./libraries/* "$INSTALL_DIR/lib/"
    
    # Install Rust dependencies
    if command -v cargo >/dev/null 2>&1; then
        info "Installing Rust dependencies..."
        cd libraries
        for crate in */Cargo.toml; do
            dir=$(dirname "$crate")
            cd "$dir"
            cargo build --release
            sudo cp "./target/release/"*.so "$INSTALL_DIR/lib/" 2>/dev/null || true
            cd ..
        done
        cd ../..
    fi
}

# Create systemd services
create_systemd_services() {
    log "Creating systemd services..."
    
    # MultiOS kernel service
    sudo tee "$SYSTEMD_DIR/multios-kernel.service" > /dev/null <<EOF
[Unit]
Description=MultiOS Hybrid Microkernel
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
ExecStart=$KERNEL_DIR/multios-kernel --daemon
Restart=always
RestartSec=10
StandardOutput=append:$INSTALL_DIR/logs/kernel.log
StandardError=append:$INSTALL_DIR/logs/kernel_error.log

[Install]
WantedBy=multi-user.target
EOF

    # Service manager
    sudo tee "$SYSTEMD_DIR/multios-service-manager.service" > /dev/null <<EOF
[Unit]
Description=MultiOS Service Manager
After=multios-kernel.service
Wants=multios-kernel.service

[Service]
Type=simple
ExecStart=$INSTALL_DIR/bin/service-manager
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

    # Enable services
    sudo systemctl enable multios-kernel.service
    sudo systemctl enable multios-service-manager.service
    
    info "Systemd services created and enabled"
}

# Create configuration files
create_config() {
    log "Creating configuration files..."
    
    # Main configuration
    sudo tee "$CONFIG_DIR/multios.conf" > /dev/null <<EOF
# MultiOS Desktop Configuration
# Version: $VERSION

[General]
version=$VERSION
install_date=$(date -u +%Y-%m-%dT%H:%M:%SZ)
install_type=desktop

[System]
kernel_path=$KERNEL_DIR/multios-kernel
install_dir=$INSTALL_DIR
config_dir=$CONFIG_DIR
log_dir=$INSTALL_DIR/logs

[Features]
graphics=enabled
networking=enabled
security=enabled
update_system=enabled

[Desktop]
gui_support=enabled
window_manager=multios-wm
file_manager=multios-filemanager

[Security]
secure_boot=enabled
encryption=optional
audit_log=enabled
EOF

    # Create default user profile
    mkdir -p "$CONFIG_DIR/profiles/default"
    sudo tee "$CONFIG_DIR/profiles/default/profile.conf" > /dev/null <<EOF
[User]
name=default
uid=1000
gid=1000
shell=/bin/multios-sh

[Preferences]
theme=default
language=en
keyboard=us
timezone=UTC

[Services]
gui=enabled
networking=enabled
sound=enabled
bluetooth=enabled
EOF

    info "Configuration files created"
}

# Install documentation
install_documentation() {
    log "Installing documentation..."
    
    sudo mkdir -p /usr/local/share/doc/multios
    sudo cp -r ./documentation/* /usr/local/share/doc/multios/
    
    # Install man pages
    if [ -d "./docs/man" ]; then
        sudo mkdir -p /usr/local/man/man1
        sudo cp ./docs/man/*.1 /usr/local/man/man1/ 2>/dev/null || true
        sudo mandb -q 2>/dev/null || true
    fi
}

# Run post-installation tests
run_tests() {
    log "Running post-installation tests..."
    
    # Test kernel installation
    if [ -x "$KERNEL_DIR/multios-kernel" ]; then
        info "Kernel binary test: PASSED"
    else
        error "Kernel binary test: FAILED"
        return 1
    fi
    
    # Test configuration
    if [ -f "$CONFIG_DIR/multios.conf" ]; then
        info "Configuration test: PASSED"
    else
        error "Configuration test: FAILED"
        return 1
    fi
    
    # Test services
    if systemctl is-enabled multios-kernel.service >/dev/null 2>&1; then
        info "Systemd services test: PASSED"
    else
        error "Systemd services test: FAILED"
        return 1
    fi
    
    info "All tests passed successfully"
}

# Print installation summary
print_summary() {
    echo
    log "MultiOS Desktop Installation Complete!"
    echo
    echo "Installation Summary:"
    echo "  Version: $VERSION"
    echo "  Install Directory: $INSTALL_DIR"
    echo "  Configuration: $CONFIG_DIR"
    echo "  Kernel: $KERNEL_DIR/multios-kernel"
    echo "  Log File: $LOG_FILE"
    echo
    echo "Next Steps:"
    echo "  1. Reboot your system"
    echo "  2. Select MultiOS from the bootloader"
    echo "  3. Configure your user profile"
    echo "  4. Install additional packages using multios-pkg"
    echo
    echo "For more information, see: /usr/local/share/doc/multios/README.md"
    echo
}

# Main installation function
main() {
    echo "========================================"
    echo "  MultiOS Desktop Installation Script"
    echo "  Version: $VERSION"
    echo "========================================"
    echo
    
    check_root
    check_requirements
    create_directories
    install_kernel
    install_bootloader
    install_libraries
    create_systemd_services
    create_config
    install_documentation
    run_tests
    print_summary
}

# Run main function
main "$@"