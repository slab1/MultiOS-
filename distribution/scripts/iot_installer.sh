#!/bin/bash
# MultiOS IoT Device Installation Wizard
# Supports installation on IoT devices (RISC-V, ARMv6, x86_64)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SUPPORTED_ARCHS=("riscv64" "armv6l" "armv7l" "x86_64")
MIN_DISK_SIZE_MB=512
RECOMMENDED_DISK_SIZE_MB=2048
MIN_MEMORY_MB=256
MAX_MEMORY_MB=4096
MIN_FLASH_SIZE_MB=128

# Installation variables
TARGET_DEVICE=""
INSTALLATION_PATH="/multios"
USERNAME="iotuser"
PASSWORD=""
TIMEZONE="UTC"
KEYBOARD_LAYOUT="us"
CONFIG_PROFILE="minimal"
INTERACTIVE_MODE=true
FAST_MODE=false
MINIMAL_MODE=false
REMOTE_MGMT=true
SERIAL_CONSOLE=true
CONFIG_SERVER=""
PROFILE=""

# Device type
DEVICE_TYPE=""
BOARD_NAME=""
SERIAL_PORT="/dev/ttyS0"
BAUD_RATE="115200"

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
        riscv64)
            echo "riscv64"
            ;;
        armv6l|armv7l|aarch64)
            echo "$(uname -m)"
            ;;
        *)
            print_color $RED "Error: Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Function to detect IoT device type
detect_device_type() {
    log "Detecting IoT device type..."
    
    local arch=$(detect_architecture)
    local board_info=""
    
    # Try to detect board information
    if [[ -f /proc/device-tree/model ]]; then
        board_info=$(cat /proc/device-tree/model 2>/dev/null || echo "")
    elif [[ -f /sys/class/dmi/id/product_name ]]; then
        board_info=$(cat /sys/class/dmi/id/product_name 2>/dev/null || echo "")
    fi
    
    # Determine device type based on architecture and board info
    case $arch in
        "x86_64")
            if [[ "$board_info" =~ (Intel|AMD) ]]; then
                DEVICE_TYPE="embedded_pc"
                echo "Detected: Embedded PC"
            else
                DEVICE_TYPE="generic_x86"
                echo "Detected: Generic x86 IoT Device"
            fi
            ;;
        "riscv64")
            DEVICE_TYPE="riscv_board"
            echo "Detected: RISC-V IoT Device"
            ;;
        "armv6l"|"armv7l"|"aarch64")
            if [[ "$board_info" =~ (Raspberry|Beaglebone|Jetson) ]]; then
                DEVICE_TYPE="single_board_computer"
                BOARD_NAME=$(echo "$board_info" | head -1)
                echo "Detected: Single Board Computer - $BOARD_NAME"
            elif [[ "$board_info" =~ (ESP|STM) ]]; then
                DEVICE_TYPE="microcontroller"
                BOARD_NAME=$(echo "$board_info" | head -1)
                echo "Detected: Microcontroller - $BOARD_NAME"
            else
                DEVICE_TYPE="arm_iot"
                echo "Detected: ARM IoT Device"
            fi
            ;;
    esac
}

# Function to check system requirements for IoT
check_system_requirements() {
    log "Checking IoT system requirements..."
    
    # Check architecture
    local arch=$(detect_architecture)
    if [[ ! " ${SUPPORTED_ARCHS[@]} " =~ " ${arch} " ]]; then
        print_color $RED "Error: Architecture $arch is not supported for IoT installation"
        exit 1
    fi
    print_color $GREEN "Architecture: $arch"
    
    # Check memory
    local memory_kb=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    local memory_mb=$((memory_kb / 1024))
    
    if [[ $memory_mb -lt $MIN_MEMORY_MB ]]; then
        print_color $YELLOW "Warning: Low memory detected (${memory_mb}MB < ${MIN_MEMORY_MB}MB)"
        CONFIG_PROFILE="ultra-minimal"
    elif [[ $memory_mb -gt $MAX_MEMORY_MB ]]; then
        print_color $GREEN "High memory available: ${memory_mb}MB"
        CONFIG_PROFILE="standard"
    fi
    print_color $GREEN "Memory: ${memory_mb}MB"
    
    # Check storage
    local available_space=$(df / | tail -1 | awk '{print $4}')
    local available_mb=$((available_space / 1024))
    if [[ $available_mb -lt $MIN_DISK_SIZE_MB ]]; then
        print_color $RED "Error: Insufficient storage (${available_mb}MB < ${MIN_DISK_SIZE_MB}MB)"
        exit 1
    fi
    print_color $GREEN "Available storage: ${available_mb}MB"
    
    # Detect device type
    detect_device_type
}

# Function to detect storage devices for IoT
detect_storage_devices() {
    log "Detecting storage devices..."
    
    echo "Available storage devices:"
    local devices=()
    local device_info=()
    local i=1
    
    # Try multiple detection methods
    for method in "lsblk" "find /dev" "blockdev"; do
        case $method in
            "lsblk")
                while IFS= read -r line; do
                    if [[ $line =~ ^/dev/ ]]; then
                        local device=$(echo $line | awk '{print $1}')
                        local size=$(lsblk -no SIZE "$device" 2>/dev/null || echo "Unknown")
                        local type=$(lsblk -no TYPE "$device" 2>/dev/null | head -1 || echo "disk")
                        
                        if [[ "$type" == "disk" ]] && [[ -b "$device" ]]; then
                            devices+=("$device")
                            device_info+=("$size")
                            echo "$i. $device ($size)"
                            i=$((i + 1))
                        fi
                    fi
                done < <(lsblk -p 2>/dev/null)
                ;;
            "find")
                for dev in /dev/mmcblk* /dev/sd* /dev/vd* /dev/nvme* /dev/ubi* /dev/mtd*; do
                    if [[ -b "$dev" ]]; then
                        local size=$(blockdev --getsize64 "$dev" 2>/dev/null || echo "0")
                        local size_mb=$((size / 1024 / 1024))
                        devices+=("$dev")
                        device_info+=("${size_mb}MB")
                        echo "$i. $dev (${size_mb}MB)"
                        i=$((i + 1))
                    fi
                done
                ;;
        esac
        
        [[ ${#devices[@]} -gt 0 ]] && break
    done
    
    if [[ ${#devices[@]} -eq 0 ]]; then
        print_color $YELLOW "No standard storage devices found. This is normal for IoT devices."
        print_color $YELLOW "The system will be installed to the current filesystem."
        TARGET_DEVICE="current"
        return 0
    fi
    
    # If interactive mode, let user select device
    if [[ "$INTERACTIVE_MODE" == "true" ]]; then
        while true; do
            echo -n "Select target device (1-${#devices[@]}) or press Enter for current filesystem: "
            read -r choice
            if [[ -z "$choice" ]]; then
                TARGET_DEVICE="current"
                break
            elif [[ $choice =~ ^[0-9]+$ ]] && [[ $choice -ge 1 ]] && [[ $choice -le ${#devices[@]} ]]; then
                TARGET_DEVICE=${devices[$((choice - 1))]}
                break
            else
                print_color $RED "Invalid choice. Please select a number between 1 and ${#devices[@]} or press Enter"
            fi
        done
    fi
}

# Function to configure serial console
configure_serial_console() {
    log "Configuring serial console..."
    
    # Determine serial port based on device type
    case $DEVICE_TYPE in
        "riscv_board")
            SERIAL_PORT="/dev/ttyS0"
            BAUD_RATE="115200"
            ;;
        "single_board_computer")
            if [[ "$BOARD_NAME" =~ "Raspberry" ]]; then
                SERIAL_PORT="/dev/ttyAMA0"
            else
                SERIAL_PORT="/dev/ttyS0"
            fi
            ;;
        *)
            SERIAL_PORT="/dev/ttyS0"
            ;;
    esac
    
    print_color $GREEN "Serial console: $SERIAL_PORT at ${BAUD_RATE} baud"
}

# Function to partition disk for IoT
partition_disk() {
    if [[ "$TARGET_DEVICE" == "current" ]]; then
        log "Using current filesystem for installation"
        return 0
    fi
    
    log "Partitioning disk $TARGET_DEVICE for IoT device..."
    
    # Get device size
    local device_size=$(blockdev --getsize64 "$TARGET_DEVICE")
    local device_size_mb=$((device_size / 1024 / 1024))
    
    # Create partition table based on device type and size
    if [[ "$device_size_mb" -lt 1024 ]]; then
        # Small device - single partition
        parted -s "$TARGET_DEVICE" mklabel msdos
        parted -s "$TARGET_DEVICE" mkpart primary 0% 100%
        ROOT_PART="${TARGET_DEVICE}1"
        mkfs.ext4 -L "multios-root" "$ROOT_PART"
    else
        # Larger device - boot + root
        parted -s "$TARGET_DEVICE" mklabel msdos
        parted -s "$TARGET_DEVICE" mkpart primary 1MiB 100MiB
        parted -s "$TARGET_DEVICE" mkpart primary 100MiB 100%
        
        ROOT_PART="${TARGET_DEVICE}2"
        
        # Format partitions
        mkfs.vfat -F32 "${TARGET_DEVICE}1"
        mkfs.ext4 -L "multios-root" "$ROOT_PART"
    fi
    
    log "IoT disk partitioning completed"
}

# Function to mount filesystems for IoT
mount_filesystems() {
    if [[ "$TARGET_DEVICE" == "current" ]]; then
        mkdir -p "$INSTALLATION_PATH"
        return 0
    fi
    
    log "Mounting IoT filesystems..."
    
    # Create mount point
    mkdir -p "$INSTALLATION_PATH"
    
    # Mount root partition
    mount "$ROOT_PART" "$INSTALLATION_PATH"
    
    # Bind mount for installation
    mount --bind /dev "$INSTALLATION_PATH/dev"
    mount --bind /proc "$INSTALLATION_PATH/proc"
    mount --bind /sys "$INSTALLATION_PATH/sys"
    
    # Mount boot partition if it exists
    if [[ -b "${TARGET_DEVICE}1" ]]; then
        mkdir -p "$INSTALLATION_PATH/boot"
        mount "${TARGET_DEVICE}1" "$INSTALLATION_PATH/boot"
    fi
    
    log "IoT filesystems mounted"
}

# Function to install minimal IoT base system
install_iot_base() {
    log "Installing minimal IoT base system..."
    
    local arch=$(detect_architecture)
    
    # Create minimal directory structure
    mkdir -p "$INSTALLATION_PATH"/{bin,sbin,etc,var,home,root,tmp,usr/{bin,sbin,lib},lib,boot,mnt,media,opt,srv}
    
    # Copy essential binaries
    local essential_bins=("sh" "init" "mount" "umount" "cat" "ls" "mkdir" "echo" "sleep")
    for bin in "${essential_bins[@]}"; do
        if [[ -f "/bin/$bin" ]]; then
            cp "/bin/$bin" "$INSTALLATION_PATH/bin/" 2>/dev/null || true
            cp "/sbin/$bin" "$INSTALLATION_PATH/sbin/" 2>/dev/null || true
        fi
    done
    
    # Create IoT-specific init
    cat > "$INSTALLATION_PATH/sbin/init" << EOF
#!/bin/sh
# MultiOS IoT Init Script

echo "MultiOS IoT System Starting..."

# Initialize console
echo "Serial console: $SERIAL_PORT at $BAUD_RATE"
stty -F $SERIAL_PORT $BAUD_RATE raw

# Mount filesystems
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t tmpfs tmpfs /tmp
mount -t tmpfs tmpfs /var/run

# Initialize hardware
case "$DEVICE_TYPE" in
    "riscv_board")
        echo "Initializing RISC-V board..."
        ;;
    "single_board_computer")
        echo "Initializing single board computer..."
        # Enable interfaces based on board type
        if [[ "$BOARD_NAME" =~ "Raspberry" ]]; then
            # Raspberry Pi specific initialization
            for iface in eth0 wlan0; do
                if [[ -d "/sys/class/net/\$iface" ]]; then
                    echo "Network interface \$iface detected"
                fi
            done
        fi
        ;;
    *)
        echo "Initializing IoT device..."
        ;;
esac

# Start services based on profile
case "$CONFIG_PROFILE" in
    "ultra-minimal")
        echo "Ultra-minimal configuration"
        ;;
    "minimal")
        echo "Minimal configuration"
        # Start networking
        ifconfig lo 127.0.0.1 up
        ;;
    "standard")
        echo "Standard configuration"
        # Start basic networking
        ifconfig lo 127.0.0.1 up
        # Auto-detect and configure interfaces
        for iface in /sys/class/net/*/; do
            iface=\$(basename "\$iface")
            if [[ "\$iface" != "lo" ]]; then
                echo "Configuring \$iface"
                ifconfig "\$iface" up 2>/dev/null || true
            fi
        done
        ;;
esac

# Start IoT services
echo "Starting IoT services..."

# Create a simple console shell
exec /bin/sh
EOF
    
    chmod +x "$INSTALLATION_PATH/sbin/init"
    
    log "IoT base system installed"
}

# Function to configure IoT bootloader
configure_iot_bootloader() {
    if [[ "$TARGET_DEVICE" == "current" ]]; then
        log "Skipping bootloader configuration for current filesystem"
        return 0
    fi
    
    log "Configuring IoT bootloader..."
    
    local arch=$(detect_architecture)
    
    # Create boot directory
    mkdir -p "$INSTALLATION_PATH/boot"
    
    # Create U-Boot environment for different architectures
    case $arch in
        "riscv64")
            cat > "$INSTALLATION_PATH/boot/uEnv.txt" << EOF
kernel_file=/boot/kernel
initrd_file=/boot/initrd
bootargs=root=$ROOT_PART rw console=$SERIAL_PORT,$BAUD_RATE
EOF
            ;;
        "armv6l"|"armv7l"|"aarch64")
            cat > "$INSTALLATION_PATH/boot/config.txt" << EOF
# MultiOS IoT Configuration
kernel=/boot/kernel
initramfs=/boot/initrd
console=$SERIAL_PORT,$BAUD_RATE
EOF
            ;;
        *)
            cat > "$INSTALLATION_PATH/boot/grub.cfg" << EOF
set timeout=5
set default=0

menuentry "MultiOS IoT" {
    linux /boot/kernel root=$ROOT_PART rw console=$SERIAL_PORT,$BAUD_RATE
    initrd /boot/initrd
}
EOF
            ;;
    esac
    
    log "IoT bootloader configured"
}

# Function to create IoT services
create_iot_services() {
    log "Creating IoT services..."
    
    # Create systemd service for IoT management
    mkdir -p "$INSTALLATION_PATH/etc/systemd/system"
    
    cat > "$INSTALLATION_PATH/etc/systemd/system/multios-iot.service" << EOF
[Unit]
Description=MultiOS IoT System
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/iot-service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
    
    # Create IoT service script
    mkdir -p "$INSTALLATION_PATH/usr/bin"
    cat > "$INSTALLATION_PATH/usr/bin/iot-service" << 'EOF'
#!/bin/sh
# IoT Service Manager
echo "MultiOS IoT Service Started"

# Monitor system health
while true; do
    # Check system status
    uptime_info=\$(uptime)
    mem_info=\$(free -h | grep Mem:)
    
    echo "System Status: \$uptime_info | Memory: \$mem_info"
    
    # Send status to remote management if configured
    if [[ -n "$CONFIG_SERVER" ]]; then
        echo "Sending status to \$CONFIG_SERVER"
        # Status reporting would go here
    fi
    
    sleep 300  # Check every 5 minutes
done
EOF
    
    chmod +x "$INSTALLATION_PATH/usr/bin/iot-service"
    
    log "IoT services created"
}

# Function to create remote management configuration
create_remote_management() {
    if [[ "$REMOTE_MGMT" != "true" ]]; then
        return 0
    fi
    
    log "Configuring remote management..."
    
    # Create SSH configuration
    mkdir -p "$INSTALLATION_PATH/etc/ssh"
    
    # Enable SSH for IoT management
    cat > "$INSTALLATION_PATH/etc/ssh/sshd_config" << EOF
# MultiOS IoT SSH Configuration
Port 22
PermitRootLogin yes
PasswordAuthentication yes
PubkeyAuthentication yes
Subsystem sftp /usr/lib/openssh/sftp-server
EOF
    
    # Create management scripts
    mkdir -p "$INSTALLATION_PATH/usr/local/bin"
    cat > "$INSTALLATION_PATH/usr/local/bin/iot-mgmt" << 'EOF'
#!/bin/sh
# IoT Management Interface
echo "MultiOS IoT Management Interface"
echo "================================="
echo "Commands:"
echo "  status - Show system status"
echo "  restart - Restart IoT service"
echo "  update - Check for updates"
echo "  config - Edit configuration"
echo
read -p "Enter command: " cmd

case "$cmd" in
    status)
        echo "System uptime: \$(uptime)"
        echo "Memory usage: \$(free -h)"
        echo "Disk usage: \$(df -h)"
        ;;
    restart)
        echo "Restarting IoT service..."
        # Service restart would go here
        ;;
    update)
        echo "Checking for updates..."
        # Update check would go here
        ;;
    config)
        echo "Opening configuration editor..."
        # Config editor would go here
        ;;
    *)
        echo "Unknown command"
        ;;
esac
EOF
    
    chmod +x "$INSTALLATION_PATH/usr/local/bin/iot-mgmt"
    
    log "Remote management configured"
}

# Function to create minimal configuration profiles
create_configuration_profiles() {
    log "Creating configuration profiles..."
    
    case $CONFIG_PROFILE in
        "ultra-minimal")
            # Ultra-minimal - no networking, serial console only
            cat > "$INSTALLATION_PATH/etc/iot_profile" << EOF
PROFILE=ultra-minimal
SERVICES=basic
NETWORKING=false
SERIAL_CONSOLE=true
EOF
            ;;
        "minimal")
            # Minimal - basic networking, serial + SSH
            cat > "$INSTALLATION_PATH/etc/iot_profile" << EOF
PROFILE=minimal
SERVICES=basic,networking,ssh
NETWORKING=true
SERIAL_CONSOLE=true
SSH=true
EOF
            ;;
        "standard"|*)
            # Standard - full networking, management tools
            cat > "$INSTALLATION_PATH/etc/iot_profile" << EOF
PROFILE=standard
SERVICES=basic,networking,ssh,monitoring,remote-mgmt
NETWORKING=true
SERIAL_CONSOLE=true
SSH=true
REMOTE_MGMT=true
EOF
            ;;
    esac
    
    log "Configuration profiles created"
}

# Function to cleanup and finalize
cleanup() {
    log "Cleaning up IoT installation..."
    
    if [[ "$TARGET_DEVICE" != "current" ]]; then
        # Unmount file systems
        umount "$INSTALLATION_PATH/boot" 2>/dev/null || true
        umount "$INSTALLATION_PATH/dev" 2>/dev/null || true
        umount "$INSTALLATION_PATH/proc" 2>/dev/null || true
        umount "$INSTALLATION_PATH/sys" 2>/dev/null || true
        umount "$INSTALLATION_PATH"
    fi
    
    print_color $GREEN "IoT installation completed successfully!"
    echo
    print_color $CYAN "Device Configuration:"
    echo "  Architecture: $(detect_architecture)"
    echo "  Device Type: $DEVICE_TYPE"
    echo "  Profile: $CONFIG_PROFILE"
    echo "  Serial Console: $SERIAL_PORT at $BAUD_RATE"
    echo "  Installation Path: $INSTALLATION_PATH"
    echo
    echo "Next steps:"
    echo "1. Reboot your IoT device"
    echo "2. Connect via serial console or SSH"
    echo "3. Run 'iot-mgmt' for management interface"
    echo
    if [[ "$REMOTE_MGMT" == "true" ]]; then
        echo "4. Configure remote management server: $CONFIG_SERVER"
    fi
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

MultiOS IoT Device Installation Wizard

OPTIONS:
    --device DEVICE          Target device (e.g., /dev/mmcblk0, or "current")
    --profile PROFILE        Configuration profile (ultra-minimal, minimal, standard)
    --serial-port PORT       Serial console port (default: /dev/ttyS0)
    --baud-rate RATE         Serial baud rate (default: 115200)
    --remote-mgmt            Enable remote management
    --config-server SERVER   Remote management server URL
    --non-interactive        Run in non-interactive mode
    --help                   Show this help message

PROFILES:
    ultra-minimal    - Serial console only, no networking (256MB+ RAM)
    minimal          - Basic networking and SSH (512MB+ RAM)
    standard         - Full IoT features with monitoring (1GB+ RAM)

EXAMPLES:
    $0 --device /dev/mmcblk0 --profile minimal
    $0 --device current --profile ultra-minimal
    $0 --device /dev/sda --remote-mgmt --config-server https://iot.company.com/api

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --device)
                TARGET_DEVICE="$2"
                shift 2
                ;;
            --profile)
                PROFILE="$2"
                shift 2
                ;;
            --serial-port)
                SERIAL_PORT="$2"
                shift 2
                ;;
            --baud-rate)
                BAUD_RATE="$2"
                shift 2
                ;;
            --remote-mgmt)
                REMOTE_MGMT=true
                shift
                ;;
            --config-server)
                CONFIG_SERVER="$2"
                REMOTE_MGMT=true
                shift 2
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
    print_color $MAGENTA "MultiOS IoT Device Installation Wizard"
    echo "========================================="
    
    # Check if running as root
    check_root
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Check system requirements
    check_system_requirements
    
    # Configure serial console
    configure_serial_console
    
    # Set profile
    if [[ -n "$PROFILE" ]]; then
        CONFIG_PROFILE="$PROFILE"
    fi
    
    # Interactive mode prompts
    if [[ "$INTERACTIVE_MODE" == "true" ]]; then
        echo
        print_color $YELLOW "Interactive IoT Installation Mode"
        echo
        
        # Detect storage devices
        detect_storage_devices
        
        # Show device type and profile
        echo
        echo "Detected Configuration:"
        echo "  Device Type: $DEVICE_TYPE"
        echo "  Architecture: $(detect_architecture)"
        echo "  Profile: $CONFIG_PROFILE"
        echo "  Serial Console: $SERIAL_PORT at $BAUD_RATE"
        echo
        
        # Prompt for profile if not set
        if [[ -z "$PROFILE" ]]; then
            echo "Configuration Profiles:"
            echo "1. ultra-minimal - Serial console only (256MB+ RAM)"
            echo "2. minimal - Basic networking and SSH (512MB+ RAM)"
            echo "3. standard - Full IoT features (1GB+ RAM)"
            echo -n "Select profile (1-3) or press Enter for $CONFIG_PROFILE: "
            read -r choice
            case $choice in
                1) CONFIG_PROFILE="ultra-minimal" ;;
                2) CONFIG_PROFILE="minimal" ;;
                3) CONFIG_PROFILE="standard" ;;
            esac
        fi
        
        # Prompt for remote management
        echo -n "Enable remote management? (y/N): "
        read -r rm_choice
        if [[ $rm_choice =~ ^[Yy]$ ]]; then
            REMOTE_MGMT=true
            echo -n "Management server URL (optional): "
            read -r CONFIG_SERVER
        fi
        
        # Confirm installation
        echo
        echo "Installation Summary:"
        echo "  Target Device: ${TARGET_DEVICE:-current filesystem}"
        echo "  Profile: $CONFIG_PROFILE"
        echo "  Remote Management: $REMOTE_MGMT"
        if [[ -n "$CONFIG_SERVER" ]]; then
            echo "  Management Server: $CONFIG_SERVER"
        fi
        echo
        echo -n "Do you want to proceed with the IoT installation? (y/N): "
        read -r confirm
        if [[ ! $confirm =~ ^[Yy]$ ]]; then
            print_color $YELLOW "Installation cancelled"
            exit 0
        fi
    fi
    
    # Start installation
    log "Starting MultiOS IoT installation"
    
    # Partition disk
    partition_disk
    
    # Mount filesystems
    mount_filesystems
    
    # Install base system
    install_iot_base
    
    # Configure bootloader
    configure_iot_bootloader
    
    # Create services
    create_iot_services
    
    # Configure remote management
    create_remote_management
    
    # Create configuration profiles
    create_configuration_profiles
    
    # Cleanup
    cleanup
    
    log "MultiOS IoT installation completed"
}

# Run main function
main "$@"