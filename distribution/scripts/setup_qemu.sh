#!/bin/bash

# MultiOS QEMU Setup Script
# Sets up QEMU for MultiOS testing across all architectures

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Detect package manager and install QEMU
install_qemu() {
    log "Installing QEMU for MultiOS testing..."
    
    if command -v apt-get >/dev/null 2>&1; then
        # Debian/Ubuntu
        log "Detected apt package manager"
        sudo apt-get update
        sudo apt-get install -y qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
            qemu-system-misc qemu-utils libcapstone-dev libslirp-dev device-tree-compiler
    elif command -v dnf >/dev/null 2>&1; then
        # Fedora
        log "Detected dnf package manager"
        sudo dnf install -y qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
            qemu-system-misc qemu-img capstone-devel libslirp-devel device-tree-compiler
    elif command -v pacman >/dev/null 2>&1; then
        # Arch Linux
        log "Detected pacman package manager"
        sudo pacman -S --noconfirm qemu-system-x86 qemu-system-aarch64 qemu-system-riscv64 \
            qemu-system-misc qemu-arch-extra capstone libslirp
    elif command -v zypper >/dev/null 2>&1; then
        # openSUSE
        log "Detected zypper package manager"
        sudo zypper install -y qemu-x86 qemu-aarch64 qemu-riscv64 \
            qemu-system-misc qemu-utils libcapstone5 libslirp0 device-tree-compiler
    elif command -v apk >/dev/null 2>&1; then
        # Alpine
        log "Detected apk package manager"
        sudo apk add qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64 \
            qemu-system-misc qemu-img capstone-dev libslirp device-tree-compiler
    else
        error "No supported package manager found"
        return 1
    fi
    
    success "QEMU installation completed"
}

# Verify QEMU installation
verify_qemu() {
    log "Verifying QEMU installation..."
    
    local missing_bins=()
    
    # Check for required QEMU binaries
    for binary in qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64 qemu-img; do
        if command -v "$binary" >/dev/null 2>&1; then
            local version=$($binary --version | head -1)
            success "$binary: $version"
        else
            missing_bins+=("$binary")
            error "$binary: not found"
        fi
    done
    
    if [ ${#missing_bins[@]} -eq 0 ]; then
        success "All required QEMU binaries are installed"
        return 0
    else
        warning "Some QEMU binaries are missing: ${missing_bins[*]}"
        return 1
    fi
}

# Setup KVM support
setup_kvm() {
    log "Setting up KVM support..."
    
    # Check if KVM is available
    if [ -e /dev/kvm ]; then
        success "KVM device found"
        
        # Check if user has access to KVM
        if [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
            success "KVM access granted"
        else
            warning "KVM access denied - adding user to kvm group"
            sudo usermod -a -G kvm "$USER"
            warning "You may need to log out and back in for KVM access"
        fi
        
        # Enable KVM in modprobe if not already enabled
        if ! lsmod | grep -q kvm; then
            log "Loading KVM modules..."
            sudo modprobe kvm
            sudo modprobe kvm_intel 2>/dev/null || sudo modprobe kvm_amd 2>/dev/null || true
        fi
        
        # Make KVM modules load on boot
        echo "kvm" | sudo tee -a /etc/modules-load.d/multios.conf >/dev/null
        echo "kvm_intel" | sudo tee -a /etc/modules-load.d/multios.conf >/dev/null 2>/dev/null || true
        echo "kvm_amd" | sudo tee -a /etc/modules-load.d/multios.conf >/dev/null 2>/dev/null || true
        
    else
        warning "KVM device not found - virtualization may not be available"
        warning "Please ensure hardware virtualization is enabled in BIOS/UEFI"
    fi
}

# Create QEMU configuration files
create_qemu_configs() {
    log "Creating QEMU configuration files..."
    
    local config_dir="/workspace/qemu/configs"
    mkdir -p "$config_dir"
    
    # x86_64 configuration
    cat > "$config_dir/x86_64.conf" << 'EOF'
# QEMU Configuration for x86_64 MultiOS Testing
ACCEL=kvm
MACHINE_TYPE=pc-q35-6.2
SMP=4
MEMORY=4G
KERNEL=target/x86_64-unknown-none/release/multios
APPEND="console=ttyS0 loglevel=8 quiet"
SERIAL="-serial stdio"
MONITOR="-monitor telnet:127.0.0.1:4444,server,nowait"
NETDEV="-netdev user,id=net0,hostfwd=tcp::5555-:22"
DEVICE="-device virtio-net,netdev=net0"
DRIVE="-drive file=/dev/null,format=raw,if=virtio"
DISPLAY="-display none"
EOF

    # ARM64 configuration
    cat > "$config_dir/arm64.conf" << 'EOF'
# QEMU Configuration for ARM64 MultiOS Testing
ACCEL=kvm
MACHINE_TYPE=virt
SMP=4
MEMORY=4G
CPU=cortex-a57
KERNEL=target/aarch64-unknown-none/release/multios
APPEND="console=ttyAMA0 loglevel=8 quiet"
SERIAL="-serial stdio"
MONITOR="-monitor telnet:127.0.0.1:4445,server,nowait"
NETDEV="-netdev user,id=net0,hostfwd=tcp::5556-:22"
DEVICE="-device virtio-net,netdev=net0"
DRIVE="-drive file=/dev/null,format=raw,if=virtio"
DISPLAY="-display none"
EOF

    # RISC-V64 configuration
    cat > "$config_dir/riscv64.conf" << 'EOF'
# QEMU Configuration for RISC-V64 MultiOS Testing
ACCEL=kvm
MACHINE_TYPE=spike_virt
SMP=4
MEMORY=4G
KERNEL=target/riscv64gc-unknown-none-elf/release/multios
APPEND="console=ttyS0 loglevel=8 quiet"
SERIAL="-serial stdio"
MONITOR="-monitor telnet:127.0.0.1:4446,server,nowait"
NETDEV="-netdev user,id=net0,hostfwd=tcp::5557-:22"
DEVICE="-device virtio-net,netdev=net0"
DRIVE="-drive file=/dev/null,format=raw,if=virtio"
DISPLAY="-display none"
EOF

    success "QEMU configuration files created in $config_dir"
}

# Setup networking support
setup_networking() {
    log "Setting up networking support..."
    
    # Check for TAP device support
    if [ -e /dev/net/tun ]; then
        success "TAP device support available"
    else
        warning "TAP device not available"
    fi
    
    # Setup iptables rules for user-mode networking (if needed)
    if command -v iptables >/dev/null 2>&1; then
        # Check if iptables rules exist
        if ! iptables -L FORWARD | grep -q "MASQUERADE.*qemu"; then
            log "Setting up iptables for QEMU networking..."
            sudo iptables -A FORWARD -m state --state RELATED,ESTABLISHED -j ACCEPT
            sudo iptables -A FORWARD -i qemu -o eth0 -j MASQUERADE
            sudo iptables -A FORWARD -i qemu -o qemu -j DROP
        else
            success "iptables rules already configured"
        fi
    else
        warning "iptables not available - networking may be limited"
    fi
}

# Setup test disk images
setup_test_disks() {
    log "Setting up test disk images..."
    
    local disk_dir="/workspace/qemu/disks"
    mkdir -p "$disk_dir"
    
    # Create small test disk images
    for size in 100M 1G; do
        local disk_file="$disk_dir/test_disk_${size}.img"
        if [ ! -f "$disk_file" ]; then
            log "Creating test disk image: $disk_file"
            qemu-img create -f qcow2 "$disk_file" "$size"
        fi
    done
    
    # Create swap disk
    local swap_disk="$disk_dir/swap_disk.img"
    if [ ! -f "$swap_disk" ]; then
        log "Creating swap disk image: $swap_disk"
        qemu-img create -f qcow2 "$swap_disk" 512M
    fi
    
    success "Test disk images created in $disk_dir"
}

# Setup monitoring scripts
setup_monitoring() {
    log "Setting up QEMU monitoring scripts..."
    
    local script_dir="/workspace/qemu/scripts"
    mkdir -p "$script_dir"
    
    # QEMU monitoring script
    cat > "$script_dir/monitor.sh" << 'EOF'
#!/bin/bash
# QEMU Monitoring Script

ARCH=$1
PID_FILE="/tmp/qemu_${ARCH}_pid.txt"

if [ -f "$PID_FILE" ]; then
    PID=$(cat "$PID_FILE")
    if kill -0 "$PID" 2>/dev/null; then
        echo "QEMU $ARCH is running (PID: $PID)"
        echo "CPU: $(ps -p $PID -o %cpu= 2>/dev/null || echo 'N/A')%"
        echo "Memory: $(ps -p $PID -o %mem= 2>/dev/null || echo 'N/A')%"
    else
        echo "QEMU $ARCH is not running"
        rm -f "$PID_FILE"
    fi
else
    echo "QEMU $ARCH is not running"
fi
EOF

    # QEMU management script
    cat > "$script_dir/manage.sh" << 'EOF'
#!/bin/bash
# QEMU Management Script

ACTION=$1
ARCH=$2

case "$ACTION" in
    "start")
        ./run_${ARCH}.sh
        ;;
    "stop")
        if [ -f "/tmp/qemu_${ARCH}_pid.txt" ]; then
            kill $(cat "/tmp/qemu_${ARCH}_pid.txt")
            rm -f "/tmp/qemu_${ARCH}_pid.txt"
            echo "QEMU $ARCH stopped"
        else
            echo "QEMU $ARCH is not running"
        fi
        ;;
    "status")
        ./monitor.sh "$ARCH"
        ;;
    "restart")
        $0 stop "$ARCH"
        sleep 2
        $0 start "$ARCH"
        ;;
    *)
        echo "Usage: $0 {start|stop|status|restart} {arch}"
        exit 1
        ;;
esac
EOF

    chmod +x "$script_dir"/*.sh
    
    success "QEMU monitoring scripts created in $script_dir"
}

# Test QEMU setup
test_qemu_setup() {
    log "Testing QEMU setup..."
    
    local test_passed=true
    
    # Test basic QEMU functionality
    for arch in x86_64 aarch64 riscv64; do
        log "Testing QEMU for $arch..."
        
        case "$arch" in
            "x86_64")
                if timeout 10 qemu-system-x86_64 -M pc -m 128M -nographic -device sga 2>/dev/null; then
                    success "x86_64 QEMU test passed"
                else
                    error "x86_64 QEMU test failed"
                    test_passed=false
                fi
                ;;
            "aarch64")
                if timeout 10 qemu-system-aarch64 -M virt -m 128M -nographic -device sga 2>/dev/null; then
                    success "ARM64 QEMU test passed"
                else
                    error "ARM64 QEMU test failed"
                    test_passed=false
                fi
                ;;
            "riscv64")
                if timeout 10 qemu-system-riscv64 -M spike_virt -m 128M -nographic -device sga 2>/dev/null; then
                    success "RISC-V64 QEMU test passed"
                else
                    error "RISC-V64 QEMU test failed"
                    test_passed=false
                fi
                ;;
        esac
    done
    
    if [ "$test_passed" = true ]; then
        success "All QEMU tests passed"
        return 0
    else
        error "Some QEMU tests failed"
        return 1
    fi
}

# Generate setup report
generate_setup_report() {
    local report_file="/workspace/qemu/setup_report_$(date +%Y%m%d_%H%M%S).txt"
    
    log "Generating QEMU setup report..."
    
    cat > "$report_file" << EOF
# MultiOS QEMU Setup Report

Setup Date: $(date -Iseconds)
System: $(uname -a)

## QEMU Installation Status
EOF

    # Check each QEMU binary
    for binary in qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64 qemu-img; do
        if command -v "$binary" >/dev/null 2>&1; then
            echo "- $binary: $(command -v $binary)" >> "$report_file"
        else
            echo "- $binary: NOT FOUND" >> "$report_file"
        fi
    done

    cat >> "$report_file" << EOF

## KVM Support
- KVM Device: $([ -e /dev/kvm ] && echo "Available" || echo "Not Available")
- User Access: $([ -r /dev/kvm ] && [ -w /dev/kvm ] && echo "Granted" || echo "Denied")

## Configuration Files
- Location: /workspace/qemu/configs/
- Files: $(ls -1 /workspace/qemu/configs/ 2>/dev/null | tr '\n' ' ' || echo "None")

## Test Disks
- Location: /workspace/qemu/disks/
- Files: $(ls -1 /workspace/qemu/disks/*.img 2>/dev/null | tr '\n' ' ' || echo "None")

## Monitoring Scripts
- Location: /workspace/qemu/scripts/
- Scripts: $(ls -1 /workspace/qemu/scripts/*.sh 2>/dev/null | xargs -I {} basename {} | tr '\n' ' ' || echo "None")

---
Setup completed successfully.
EOF

    success "Setup report generated: $report_file"
}

# Main setup function
main() {
    log "Starting MultiOS QEMU Setup"
    
    # Install QEMU
    if ! install_qemu; then
        error "QEMU installation failed"
        exit 1
    fi
    
    # Verify installation
    if ! verify_qemu; then
        warning "QEMU verification failed - some features may not work"
    fi
    
    # Setup components
    setup_kvm
    create_qemu_configs
    setup_networking
    setup_test_disks
    setup_monitoring
    
    # Test setup
    if test_qemu_setup; then
        success "QEMU setup completed successfully"
    else
        warning "QEMU setup completed with some issues"
    fi
    
    # Generate report
    generate_setup_report
    
    echo
    echo "QEMU Setup Summary:"
    echo "==================="
    echo "QEMU binaries: $(command -v qemu-system-x86_64 qemu-system-aarch64 qemu-system-riscv64 qemu-img 2>/dev/null | wc -l)/4 found"
    echo "KVM support: $([ -e /dev/kvm ] && echo "Available" || echo "Not Available")"
    echo "Config files: $(ls -1 /workspace/qemu/configs/ 2>/dev/null | wc -l) created"
    echo "Test scripts: $(ls -1 /workspace/qemu/scripts/*.sh 2>/dev/null | wc -l) created"
    echo "==================="
}

main "$@"