#!/bin/bash
# MultiOS Mobile Device Installation Wizard
# Supports installation on mobile devices and tablets (ARM64, ARMv7)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SUPPORTED_ARCHS=("arm64" "armv7l")
MIN_DISK_SIZE_GB=8
RECOMMENDED_DISK_SIZE_GB=32
MIN_MEMORY_MB=1024
RECOMMENDED_MEMORY_MB=2048

# Installation variables
TARGET_DISK=""
INSTALLATION_PATH="/multios"
USERNAME=""
PASSWORD=""
TIMEZONE="UTC"
KEYBOARD_LAYOUT="us"
CONFIG_PROFILE="mobile"
INTERACTIVE_MODE=true
FAST_MODE=false
ENCRYPTION=false
ROOT_PART_SIZE=""

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
        aarch64|arm64)
            echo "arm64"
            ;;
        armv7l|armv8l)
            echo "armv7l"
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
        print_color $RED "Error: Architecture $arch is not supported for mobile installation"
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
    
    # Check if we're running on ARM hardware
    local cpu_info=$(grep "model name" /proc/cpuinfo || echo "")
    if [[ -z "$cpu_info" ]]; then
        local arm_cpu=$(grep "Processor" /proc/cpuinfo || echo "")
        if [[ -z "$arm_cpu" ]]; then
            print_color $YELLOW "Warning: Could not detect CPU information. Proceeding with caution."
        fi
    fi
}

# Function to detect storage devices
detect_storage_devices() {
    log "Detecting storage devices..."
    
    echo "Available storage devices:"
    local devices=()
    local i=1
    
    # Try multiple methods to detect storage devices
    while IFS= read -r line; do
        if [[ $line =~ ^/dev/ ]]; then
            local device=$(echo $line | awk '{print $1}')
            local size=$(lsblk -no SIZE "$device" 2>/dev/null || echo "Unknown")
            local type=$(lsblk -no TYPE "$device" 2>/dev/null | head -1 || echo "disk")
            local name=$(basename "$device")
            
            # Skip loop devices and RAM disks
            if [[ ! "$name" =~ ^(loop|ram) ]] && [[ "$type" == "disk" ]]; then
                devices+=("$device")
                echo "$i. $device ($size)"
                i=$((i + 1))
            fi
        fi
    done < <(lsblk -p 2>/dev/null)
    
    # If lsblk failed, try alternative methods
    if [[ ${#devices[@]} -eq 0 ]]; then
        log "lsblk failed, trying alternative detection methods..."
        
        # Check for common mobile device nodes
        for dev in /dev/mmcblk* /dev/sd* /dev/vd* /dev/nvme*; do
            if [[ -b "$dev" ]]; then
                local size=$(blockdev --getsize64 "$dev" 2>/dev/null || echo "0")
                local size_gb=$((size / 1024 / 1024 / 1024))
                devices+=("$dev")
                echo "$i. $dev (${size_gb}GB)"
                i=$((i + 1))
            fi
        done
    fi
    
    if [[ ${#devices[@]} -eq 0 ]]; then
        print_color $RED "Error: No storage devices detected"
        print_color $YELLOW "Common mobile device nodes to try:"
        echo "  /dev/mmcblk0 (eMMC)"
        echo "  /dev/mmcblk1 (SD card)"
        echo "  /dev/sda (USB storage)"
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

# Function to partition disk for mobile device
partition_disk() {
    log "Partitioning disk $TARGET_DISK for mobile device..."
    
    # Get disk size
    local disk_size=$(blockdev --getsize64 "$TARGET_DISK")
    local disk_size_gb=$((disk_size / 1024 / 1024 / 1024))
    
    # Create partition table
    if [[ "$disk_size_gb" -gt 32 ]]; then
        # For larger disks, create standard layout
        parted -s "$TARGET_DISK" mklabel gpt
        parted -s "$TARGET_DISK" mkpart primary 1MiB 512MiB
        parted -s "$TARGET_DISK" mkpart primary ext4 512MiB 100%
        ROOT_PART_SIZE="100%"
    else
        # For smaller disks, create compact layout
        local root_start="200MiB"
        local root_end=$((disk_size_gb * 1024 - 50))MiB
        parted -s "$TARGET_DISK" mklabel gpt
        parted -s "$TARGET_DISK" mkpart primary 1MiB 200MiB  # Boot partition
        parted -s "$TARGET_DISK" mkpart primary ext4 "$root_start" "$root_end"  # Root partition
        parted -s "$TARGET_DISK" mkpart primary linux-swap "$root_end" 100%  # Swap partition
        ROOT_PART_SIZE="$root_start:$root_end"
    fi
    
    # Get partition numbers
    ROOT_PART=$(parted -s "$TARGET_DISK" print | grep "primary" | tail -1 | awk '{print $1}')
    
    # Format partitions
    if [[ "$ROOT_PART_SIZE" == "100%" ]]; then
        mkfs.ext4 -L "multios-root" "${TARGET_DISK}${ROOT_PART}"
    else
        mkfs.ext4 -L "multios-root" "${TARGET_DISK}${ROOT_PART}"
    fi
    
    # Enable resize for later expansion
    tune2fs -c 0 -i 0 "${TARGET_DISK}${ROOT_PART}" 2>/dev/null || true
    
    log "Disk partitioning completed"
}

# Function to mount file systems
mount_filesystems() {
    log "Mounting file systems..."
    
    # Create mount point
    mkdir -p "$INSTALLATION_PATH"
    
    # Mount root partition
    mount "${TARGET_DISK}${ROOT_PART}" "$INSTALLATION_PATH"
    
    # Create essential directories
    mkdir -p "$INSTALLATION_PATH"/{bin,sbin,etc,var,home,root,tmp,usr,boot,proc,sys,dev,run,mnt,media,opt,srv}
    
    # Bind mount for installation
    mount --bind /dev "$INSTALLATION_PATH/dev"
    mount --bind /proc "$INSTALLATION_PATH/proc"
    mount --bind /sys "$INSTALLATION_PATH/sys"
    
    log "File systems mounted"
}

# Function to install mobile-optimized base system
install_mobile_base() {
    log "Installing mobile-optimized base system..."
    
    local arch=$(detect_architecture)
    
    # Create minimal directory structure
    mkdir -p "$INSTALLATION_PATH"/{bin,sbin,usr/{bin,sbin,lib},lib,lib64}
    
    # Copy essential binaries for ARM
    local binaries=("bash" "sh" "init" "mount" "umount" "cat" "ls" "mkdir" "touch" "echo")
    for bin in "${binaries[@]}"; do
        if [[ -f "/bin/$bin" ]]; then
            cp "/bin/$bin" "$INSTALLATION_PATH/bin/" 2>/dev/null || true
        fi
    done
    
    # Create init script for mobile devices
    cat > "$INSTALLATION_PATH/sbin/init" << 'EOF'
#!/bin/sh
# Mobile device init script

echo "MultiOS Mobile Initializing..."

# Mount essential filesystems
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t tmpfs tmpfs /tmp
mount -t tmpfs tmpfs /var/run

# Initialize display
if [[ -f /dev/fb0 ]]; then
    echo "Display detected"
    # Framebuffer initialization would go here
fi

# Initialize networking
if [[ -f /sys/class/net/wlan0/uevent ]]; then
    echo "WiFi detected"
    # WiFi initialization would go here
fi

# Start mobile services
echo "Starting mobile services..."
# Service management would go here

# Start GUI
if [[ -f /usr/bin/wayland ]]; then
    echo "Starting Wayland compositor..."
    # Wayland compositor would start here
elif [[ -f /usr/bin/X ]]; then
    echo "Starting X11..."
    # X11 initialization would go here
fi

# Welcome message
echo "Welcome to MultiOS Mobile!"
EOF
    
    chmod +x "$INSTALLATION_PATH/sbin/init"
    
    log "Mobile base system installed"
}

# Function to configure mobile bootloader
configure_mobile_bootloader() {
    log "Configuring mobile bootloader..."
    
    local arch=$(detect_architecture)
    
    # Create U-Boot configuration for ARM
    mkdir -p "$INSTALLATION_PATH/boot"
    
    cat > "$INSTALLATION_PATH/boot/uEnv.txt" << EOF
kernel_file=/boot/kernel
initrd_file=/boot/initrd
bootargs=root=${TARGET_DISK}${ROOT_PART} rw quiet splash console=ttyS0,115200
EOF
    
    # Create boot script
    cat > "$INSTALLATION_PATH/boot/boot.scr" << EOF
# U-Boot boot script for MultiOS Mobile
echo "MultiOS Mobile Boot..."
if fatload \${mmc_dev} 1:1 0x42000000 kernel; then
    if fatload \${mmc_dev} 1:1 0x43000000 initrd; then
        booti 0x42000000 0x43000000 0x41000000
    else
        booti 0x42000000 - 0x41000000
    fi
fi
EOF
    
    log "Mobile bootloader configured"
}

# Function to create mobile user setup
setup_mobile_user() {
    if [[ -n "$USERNAME" ]]; then
        log "Setting up mobile user: $USERNAME"
        
        # Create mobile user directory
        mkdir -p "$INSTALLATION_PATH/home/$USERNAME"
        chown 1000:1000 "$INSTALLATION_PATH/home/$USERNAME"
        
        # Create mobile-specific configuration
        mkdir -p "$INSTALLATION_PATH/home/$USERNAME/.config"
        mkdir -p "$INSTALLATION_PATH/home/$USERNAME/.local/share"
        
        # Create basic mobile configuration
        cat > "$INSTALLATION_PATH/home/$USERNAME/.bashrc" << EOF
# MultiOS Mobile Configuration
export PATH=/usr/local/bin:/usr/bin:/bin
export PS1="[\\u@\\h \\w]\\$ "
alias ll='ls -alF'
alias la='ls -A'
alias l='ls -CF'
EOF
        
        log "Mobile user configured"
    fi
}

# Function to install mobile applications
install_mobile_apps() {
    log "Installing mobile applications..."
    
    # Create apps directory
    mkdir -p "$INSTALLATION_PATH/usr/share/applications"
    
    # Create mobile launcher
    cat > "$INSTALLATION_PATH/usr/share/applications/multios-mobile.desktop" << EOF
[Desktop Entry]
Name=MultiOS Mobile
Comment=MultiOS Mobile Operating System
Exec=/usr/bin/start_multios_mobile
Icon=multios
Terminal=false
Type=Application
Categories=System;
EOF
    
    # Create start script
    cat > "$INSTALLATION_PATH/usr/bin/start_multios_mobile" << 'EOF'
#!/bin/sh
# MultiOS Mobile launcher
echo "Starting MultiOS Mobile..."
# GUI launcher would go here
EOF
    
    chmod +x "$INSTALLATION_PATH/usr/bin/start_multios_mobile"
    
    log "Mobile applications configured"
}

# Function to enable mobile services
enable_mobile_services() {
    log "Enabling mobile services..."
    
    # Create systemd service for mobile
    cat > "$INSTALLATION_PATH/etc/systemd/system/multios-mobile.service" << EOF
[Unit]
Description=MultiOS Mobile
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/bin/start_multios_mobile
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF
    
    log "Mobile services configured"
}

# Function to create mobile recovery partition
create_recovery_partition() {
    log "Creating recovery partition..."
    
    # Create small recovery partition at the end
    local recovery_start=$(parted -s "$TARGET_DISK" print | grep "primary" | tail -1 | awk '{print $3}')
    local disk_end=$(parted -s "$TARGET_DISK" print | grep "Disk" | awk '{print $3}')
    
    if [[ -n "$recovery_start" ]] && [[ -n "$disk_end" ]]; then
        # Add recovery partition (last 100MB)
        local recovery_start_mb=$(echo "$disk_end" | sed 's/MB//' | awk '{print int($1 - 100)}')
        local recovery_end=$(echo "$disk_end" | sed 's/MB//')
        
        parted -s "$TARGET_DISK" mkpart primary ext4 "${recovery_start_mb}MB" "${recovery_end}MB"
        local recovery_part=$(parted -s "$TARGET_DISK" print | grep "primary" | tail -1 | awk '{print $1}')
        
        mkfs.ext4 -L "multios-recovery" "${TARGET_DISK}${recovery_part}"
        
        log "Recovery partition created: ${TARGET_DISK}${recovery_part}"
    fi
}

# Function to unmount and cleanup
cleanup() {
    log "Cleaning up..."
    
    # Unmount file systems
    umount "$INSTALLATION_PATH/dev" 2>/dev/null || true
    umount "$INSTALLATION_PATH/proc" 2>/dev/null || true
    umount "$INSTALLATION_PATH/sys" 2>/dev/null || true
    umount "$INSTALLATION_PATH"
    
    print_color $GREEN "Mobile installation completed successfully!"
    echo "Please reboot your device to start using MultiOS Mobile."
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

MultiOS Mobile Device Installation Wizard

OPTIONS:
    --target DEVICE          Target disk device (e.g., /dev/mmcblk0)
    --username USERNAME      Username for the new user account
    --password PASSWORD      Password for the new user account
    --timezone TIMEZONE      Timezone (default: UTC)
    --keyboard LAYOUT        Keyboard layout (default: us)
    --encryption             Enable disk encryption
    --fast                   Fast installation mode
    --non-interactive        Run in non-interactive mode
    --help                   Show this help message

EXAMPLES:
    $0 --target /dev/mmcblk0 --username john
    $0 --target /dev/mmcblk1 --encryption --non-interactive
    $0 --target /dev/mmcblk0 --fast

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
            --encryption)
                ENCRYPTION=true
                shift
                ;;
            --fast)
                FAST_MODE=true
                shift
                ;;
            --non-interactive)
                INTERACTIVE_MODE=false
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
    print_color $BLUE "MultiOS Mobile Device Installation Wizard"
    echo "=============================================="
    
    # Check if running as root
    check_root
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Check system requirements
    check_system_requirements
    
    # Interactive mode prompts
    if [[ "$INTERACTIVE_MODE" == "true" ]]; then
        echo
        print_color $YELLOW "Interactive Mobile Installation Mode"
        echo
        
        # Detect storage devices
        detect_storage_devices
        
        # Get username
        if [[ -z "$USERNAME" ]]; then
            echo -n "Enter username (optional, default: mobile): "
            read -r USERNAME
            USERNAME=${USERNAME:-mobile}
        fi
        
        # Confirm installation
        echo
        echo "Installation Summary:"
        echo "  Target Device: $TARGET_DISK"
        echo "  User: $USERNAME"
        echo "  Fast Mode: $FAST_MODE"
        echo "  Encryption: $ENCRYPTION"
        echo
        echo -n "Do you want to proceed with the mobile installation? (y/N): "
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
    log "Starting MultiOS mobile installation"
    
    # Partition disk
    partition_disk
    
    # Create recovery partition if not in fast mode
    if [[ "$FAST_MODE" != "true" ]]; then
        create_recovery_partition
    fi
    
    # Mount file systems
    mount_filesystems
    
    # Install base system
    install_mobile_base
    
    # Configure bootloader
    configure_mobile_bootloader
    
    # Setup user
    setup_mobile_user
    
    # Install mobile applications
    install_mobile_apps
    
    # Enable services
    enable_mobile_services
    
    # Cleanup
    cleanup
    
    log "MultiOS mobile installation completed"
}

# Run main function
main "$@"