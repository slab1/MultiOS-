#!/bin/bash
# MultiOS Desktop Installation Wizard
# Supports installation on desktop PCs (x86_64, ARM64)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging
LOG_FILE="/var/log/multios_installation.log"
exec 1> >(tee -a "$LOG_FILE")
exec 2> >(tee -a "$LOG_FILE")

# Configuration
SUPPORTED_ARCHS=("x86_64" "arm64")
MIN_DISK_SIZE_GB=16
RECOMMENDED_DISK_SIZE_GB=50
MIN_MEMORY_MB=2048
RECOMMENDED_MEMORY_MB=4096

# Installation variables
TARGET_DISK=""
INSTALLATION_PATH="/"
USERNAME=""
PASSWORD=""
TIMEZONE=""
KEYBOARD_LAYOUT="us"
PACKAGES=()
CONFIG_PROFILE="desktop"
INTERACTIVE_MODE=true

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to print colored output
print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_color $RED "Error: This script must be run as root"
        exit 1
    fi
}

# Function to detect system architecture
detect_architecture() {
    local arch=$(uname -m)
    case $arch in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            print_color $RED "Error: Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Function to check system requirements
check_system_requirements() {
    log "Checking system requirements..."
    
    # Check architecture
    local arch=$(detect_architecture)
    if [[ ! " ${SUPPORTED_ARCHS[@]} " =~ " ${arch} " ]]; then
        print_color $RED "Error: Architecture $arch is not supported"
        exit 1
    fi
    print_color $GREEN "Architecture: $arch"
    
    # Check memory
    local memory_kb=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    local memory_mb=$((memory_kb / 1024))
    if [[ $memory_mb -lt $MIN_MEMORY_MB ]]; then
        print_color $YELLOW "Warning: Low memory detected (${memory_mb}MB < ${MIN_MEMORY_MB}MB)"
    fi
    print_color $GREEN "Memory: ${memory_mb}MB"
    
    # Check disk space
    local available_space=$(df / | tail -1 | awk '{print $4}')
    local available_gb=$((available_space / 1024 / 1024))
    if [[ $available_gb -lt $MIN_DISK_SIZE_GB ]]; then
        print_color $RED "Error: Insufficient disk space (${available_gb}GB < ${MIN_DISK_SIZE_GB}GB)"
        exit 1
    fi
    print_color $GREEN "Available disk space: ${available_gb}GB"
}

# Function to detect available storage devices
detect_storage_devices() {
    log "Detecting storage devices..."
    
    echo "Available storage devices:"
    local devices=()
    local i=1
    
    while IFS= read -r line; do
        if [[ $line =~ ^/dev/ ]]; then
            local device=$(echo $line | awk '{print $1}')
            local size=$(lsblk -no SIZE "$device" 2>/dev/null || echo "Unknown")
            local type=$(lsblk -no TYPE "$device" 2>/dev/null | head -1 || echo "Unknown")
            devices+=("$device")
            echo "$i. $device ($size, $type)"
            i=$((i + 1))
        fi
    done < <(lsblk -p)
    
    if [[ ${#devices[@]} -eq 0 ]]; then
        print_color $RED "Error: No storage devices detected"
        exit 1
    fi
    
    # If interactive mode, let user select device
    if [[ "$INTERACTIVE_MODE" == "true" ]]; then
        while true; do
            echo -n "Select target device (1-${#devices[@]}): "
            read -r choice
            if [[ $choice =~ ^[0-9]+$ ]] && [[ $choice -ge 1 ]] && [[ $choice -le ${#devices[@]} ]]; then
                TARGET_DISK=${devices[$((choice - 1))]}
                break
            else
                print_color $RED "Invalid choice. Please select a number between 1 and ${#devices[@]}"
            fi
        done
    fi
}

# Function to partition disk
partition_disk() {
    log "Partitioning disk $TARGET_DISK..."
    
    # Create partition table
    parted -s "$TARGET_DISK" mklabel gpt
    
    # Create EFI partition (512MB)
    parted -s "$TARGET_DISK" mkpart EFI fat32 1MiB 513MiB
    parted -s "$TARGET_DISK" set 1 esp on
    
    # Create root partition (remaining space)
    parted -s "$TARGET_DISK" mkpart root ext4 513MiB 100%
    
    # Format partitions
    mkfs.fat -F32 "${TARGET_DISK}1"
    mkfs.ext4 "${TARGET_DISK}2"
    
    log "Disk partitioning completed"
}

# Function to mount file systems
mount_filesystems() {
    log "Mounting file systems..."
    
    local root_part="${TARGET_DISK}2"
    local efi_part="${TARGET_DISK}1"
    
    # Mount root partition
    mount "$root_part" "$INSTALLATION_PATH"
    
    # Mount EFI partition
    mkdir -p "$INSTALLATION_PATH/boot/efi"
    mount "$efi_part" "$INSTALLATION_PATH/boot/efi"
    
    # Bind mount essential directories
    mount --bind /dev "$INSTALLATION_PATH/dev"
    mount --bind /proc "$INSTALLATION_PATH/proc"
    mount --bind /sys "$INSTALLATION_PATH/sys"
    
    log "File systems mounted"
}

# Function to install base system
install_base_system() {
    log "Installing base system..."
    
    local arch=$(detect_architecture)
    
    # Copy kernel and initrd
    if [[ -f /boot/vmlinuz-* ]]; then
        cp /boot/vmlinuz-* "$INSTALLATION_PATH/boot/"
        cp /boot/initrd.img-* "$INSTALLATION_PATH/boot/" 2>/dev/null || true
    fi
    
    # Copy firmware and modules
    mkdir -p "$INSTALLATION_PATH/lib/modules"
    cp -r /lib/modules/* "$INSTALLATION_PATH/lib/modules/" 2>/dev/null || true
    
    # Copy essential binaries
    mkdir -p "$INSTALLATION_PATH/bin" "$INSTALLATION_PATH/sbin" "$INSTALLATION_PATH/usr/bin"
    cp /bin/bash "$INSTALLATION_PATH/bin/" 2>/dev/null || true
    cp /bin/sh "$INSTALLATION_PATH/bin/" 2>/dev/null || true
    cp /sbin/init "$INSTALLATION_PATH/sbin/" 2>/dev/null || true
    
    log "Base system installed"
}

# Function to configure bootloader
configure_bootloader() {
    log "Configuring bootloader..."
    
    local arch=$(detect_architecture)
    
    # Create GRUB configuration
    mkdir -p "$INSTALLATION_PATH/boot/grub"
    
    cat > "$INSTALLATION_PATH/boot/grub/grub.cfg" << EOF
set timeout=5
set default=0

menuentry "MultiOS" {
    linux /boot/vmlinuz-* root=$TARGET_DISK2 ro quiet splash
    initrd /boot/initrd.img-*
}
EOF
    
    # Install GRUB
    if [[ "$arch" == "x86_64" ]]; then
        grub-install --target=x86_64-efi --efi-directory="$INSTALLATION_PATH/boot/efi" --bootloader-id=MultiOS
    elif [[ "$arch" == "arm64" ]]; then
        grub-install --target=arm64-efi --efi-directory="$INSTALLATION_PATH/boot/efi" --bootloader-id=MultiOS
    fi
    
    log "Bootloader configured"
}

# Function to create user account
create_user_account() {
    if [[ -n "$USERNAME" ]] && [[ -n "$PASSWORD" ]]; then
        log "Creating user account: $USERNAME"
        
        # Create user
        useradd -m -s /bin/bash "$USERNAME"
        
        # Set password
        echo "$USERNAME:$PASSWORD" | chpasswd
        
        # Add to sudo group
        usermod -aG sudo "$USERNAME"
        
        log "User account created"
    fi
}

# Function to configure network
configure_network() {
    log "Configuring network..."
    
    # Create basic network configuration
    mkdir -p "$INSTALLATION_PATH/etc/systemd/network"
    
    cat > "$INSTALLATION_PATH/etc/systemd/network/20-wired.network" << EOF
[Match]
Name=*

[Network]
DHCP=yes
EOF
    
    log "Network configured"
}

# Function to enable services
enable_services() {
    log "Enabling services..."
    
    # Enable essential services
    local services=(
        "systemd-networkd"
        "systemd-resolved"
        "ssh"
    )
    
    for service in "${services[@]}"; do
        systemctl --root="$INSTALLATION_PATH" enable "$service" 2>/dev/null || true
    done
    
    log "Services enabled"
}

# Function to unmount and cleanup
cleanup() {
    log "Cleaning up..."
    
    # Unmount file systems
    umount "$INSTALLATION_PATH/dev" 2>/dev/null || true
    umount "$INSTALLATION_PATH/proc" 2>/dev/null || true
    umount "$INSTALLATION_PATH/sys" 2>/dev/null || true
    umount "$INSTALLATION_PATH/boot/efi" 2>/dev/null || true
    umount "$INSTALLATION_PATH"
    
    print_color $GREEN "Installation completed successfully!"
    echo "Please reboot your system to start using MultiOS."
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

MultiOS Desktop Installation Wizard

OPTIONS:
    --target DEVICE          Target disk device (e.g., /dev/sda)
    --username USERNAME      Username for the new user account
    --password PASSWORD      Password for the new user account
    --timezone TIMEZONE      Timezone (e.g., America/New_York)
    --keyboard LAYOUT        Keyboard layout (default: us)
    --profile PROFILE        Configuration profile (desktop, server, minimal)
    --non-interactive        Run in non-interactive mode
    --dry-run               Show what would be done without making changes
    --help                  Show this help message

EXAMPLES:
    $0 --target /dev/sda --username john --password secret123
    $0 --target /dev/sdb --profile minimal --non-interactive
    $0 --dry-run --target /dev/sda

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --target)
                TARGET_DISK="$2"
                shift 2
                ;;
            --username)
                USERNAME="$2"
                shift 2
                ;;
            --password)
                PASSWORD="$2"
                shift 2
                ;;
            --timezone)
                TIMEZONE="$2"
                shift 2
                ;;
            --keyboard)
                KEYBOARD_LAYOUT="$2"
                shift 2
                ;;
            --profile)
                CONFIG_PROFILE="$2"
                shift 2
                ;;
            --non-interactive)
                INTERACTIVE_MODE=false
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                print_color $RED "Error: Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Main installation function
main() {
    print_color $BLUE "MultiOS Desktop Installation Wizard"
    echo "======================================"
    
    # Check if running as root
    check_root
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Check system requirements
    check_system_requirements
    
    # Interactive mode prompts
    if [[ "$INTERACTIVE_MODE" == "true" ]]; then
        echo
        print_color $YELLOW "Interactive Installation Mode"
        echo
        
        # Detect storage devices
        detect_storage_devices
        
        # Get username and password if not provided
        if [[ -z "$USERNAME" ]]; then
            echo -n "Enter username for the new account: "
            read -r USERNAME
        fi
        
        if [[ -z "$PASSWORD" ]]; then
            echo -n "Enter password: "
            read -rs PASSWORD
            echo
        fi
        
        # Confirm installation
        echo
        echo "Installation Summary:"
        echo "  Target Disk: $TARGET_DISK"
        echo "  User: $USERNAME"
        echo "  Profile: $CONFIG_PROFILE"
        echo
        echo -n "Do you want to proceed with the installation? (y/N): "
        read -r confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            print_color $YELLOW "Installation cancelled"
            exit 0
        fi
    fi
    
    # Validate required parameters for non-interactive mode
    if [[ "$INTERACTIVE_MODE" == "false" ]] && [[ -z "$TARGET_DISK" ]]; then
        print_color $RED "Error: --target is required in non-interactive mode"
        show_usage
        exit 1
    fi
    
    # Start installation
    log "Starting MultiOS desktop installation"
    
    if [[ "$DRY_RUN" != "true" ]]; then
        # Partition disk
        partition_disk
        
        # Mount file systems
        mount_filesystems
        
        # Install base system
        install_base_system
        
        # Configure bootloader
        configure_bootloader
        
        # Create user account
        create_user_account
        
        # Configure network
        configure_network
        
        # Enable services
        enable_services
        
        # Cleanup
        cleanup
    else
        log "DRY RUN MODE - No changes will be made"
        echo "The following would be performed:"
        echo "  1. Partition disk: $TARGET_DISK"
        echo "  2. Mount file systems to: $INSTALLATION_PATH"
        echo "  3. Install base system"
        echo "  4. Configure bootloader"
        echo "  5. Create user account: $USERNAME"
        echo "  6. Configure network"
        echo "  7. Enable services"
    fi
    
    log "MultiOS desktop installation completed"
}

# Run main function
main "$@"