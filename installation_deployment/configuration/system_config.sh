#!/bin/bash
# MultiOS System Configuration Tool
# Hardware detection, system setup, and configuration management

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
CONFIG_TOOL_VERSION="1.0.0"
CONFIG_DIR="/etc/multios/config"
CONFIG_FILE="$CONFIG_DIR/system.conf"
LOG_FILE="/var/log/multios-config.log"
PROFILE_DIR="$CONFIG_DIR/profiles"
STATE_FILE="/var/lib/multios/system_state.json"

# Configuration operations
OP_DETECT="detect"
OP_CONFIGURE="configure"
OP_UPDATE="update"
OP_BACKUP="backup"
OP_RESTORE="restore"
OP_STATUS="status"
OP_RESET="reset"

# Variables
OPERATION=""
CONFIG_PROFILE=""
HARDWARE_PROFILE=""
NETWORK_CONFIG=""
USER_CONFIG=""
SERVICE_CONFIG=""
DRY_RUN=false
VERBOSE=false
INTERACTIVE=true
FORCE=false
AUTO_CONFIG=false

# Function to log messages
log() {
    local level="$1"
    shift
    local message="$@"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $message" >> "$LOG_FILE"
    
    case $level in
        "INFO")
            [[ "$VERBOSE" == "true" ]] && print_color $BLUE "[INFO] $message"
            ;;
        "WARN")
            print_color $YELLOW "[WARN] $message"
            ;;
        "ERROR")
            print_color $RED "[ERROR] $message"
            ;;
        "SUCCESS")
            print_color $GREEN "[SUCCESS] $message"
            ;;
        "DEBUG")
            [[ "$VERBOSE" == "true" ]] && print_color $CYAN "[DEBUG] $message"
            ;;
    esac
}

# Function to print colored output
print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Function to create directories
create_directories() {
    mkdir -p "$CONFIG_DIR" "$PROFILE_DIR" "$(dirname "$LOG_FILE")" "$(dirname "$STATE_FILE")"
    chmod 755 "$CONFIG_DIR" "$PROFILE_DIR"
}

# Function to initialize configuration system
init_config_system() {
    log "INFO" "Initializing MultiOS Configuration System v$CONFIG_TOOL_VERSION"
    
    create_directories
    
    # Create default configuration
    if [[ ! -f "$CONFIG_FILE" ]]; then
        create_default_config
    fi
    
    # Create hardware detection script
    create_hardware_detector
    
    # Create configuration profiles
    create_profiles
    
    log "SUCCESS" "Configuration system initialized"
}

# Function to create default configuration
create_default_config() {
    cat > "$CONFIG_FILE" << 'EOF'
# MultiOS System Configuration

[general]
default_profile=desktop
log_level=INFO
verbose=false
auto_detect=true

[hardware]
# Hardware detection settings
detect_graphics=true
detect_audio=true
detect_network=true
detect_storage=true
detect_usb=true

[network]
# Default network configuration
method=dhcp
interface=auto
timeout=30

[services]
# Services to enable by default
essential=systemd-networkd,systemd-resolved
desktop=NetworkManager,ssh
server=ssh,systemd-timesyncd
iot=serial-getty@.service

[security]
# Security settings
firewall=enabled
selinux=disabled
apparmor=disabled
password_policy=enabled

[performance]
# Performance tuning
cpu_governor=balanced
memory_management=default
io_scheduler=default
EOF
}

# Function to create hardware detector
create_hardware_detector() {
    cat > "$CONFIG_DIR/hardware-detector.sh" << 'EOF'
#!/bin/bash
# MultiOS Hardware Detection Script

detect_cpu() {
    echo "CPU Information:"
    grep "model name" /proc/cpuinfo 2>/dev/null | head -1 | cut -d: -f2 | xargs
    echo "  Cores: $(nproc)"
    echo "  Architecture: $(uname -m)"
    echo "  Features: $(grep flags /proc/cpuinfo 2>/dev/null | head -1 | cut -d: -f2 | xargs)"
}

detect_memory() {
    echo "Memory Information:"
    free -h | grep "Mem:" | awk '{print "  Total: " $2}'
    echo "  Type: $(dmidecode -t memory 2>/dev/null | grep "Type:" | head -1 | cut -d: -f2 | xargs || echo "Unknown")"
}

detect_storage() {
    echo "Storage Devices:"
    lsblk -p -o name,size,type,model | grep "disk" | while read line; do
        echo "  $line"
    done
}

detect_network() {
    echo "Network Interfaces:"
    for iface in /sys/class/net/*/; do
        iface=$(basename "$iface")
        if [[ "$iface" != "lo" ]]; then
            echo "  $iface: $(cat "/sys/class/net/$iface/operstate" 2>/dev/null || echo "unknown")"
        fi
    done
}

detect_graphics() {
    echo "Graphics Devices:"
    if command -v lspci &> /dev/null; then
        lspci | grep -i vga | while read line; do
            echo "  $line"
        done
    fi
    echo "  Driver: $(lsmod | grep -E "video|drm|nouveau|radeon|i915" | head -1 | cut -d' ' -f1 || echo "none")"
}

detect_audio() {
    echo "Audio Devices:"
    if command -v lspci &> /dev/null; then
        lspci | grep -i audio | while read line; do
            echo "  $line"
        done
    fi
}

detect_usb() {
    echo "USB Devices:"
    if command -v lsusb &> /dev/null; then
        lsusb | while read line; do
            echo "  $line"
        done
    fi
}

# Run all detections
echo "Hardware Detection Report"
echo "========================"
detect_cpu
echo
detect_memory
echo
detect_storage
echo
detect_network
echo
detect_graphics
echo
detect_audio
echo
detect_usb
EOF
    
    chmod +x "$CONFIG_DIR/hardware-detector.sh"
}

# Function to create configuration profiles
create_profiles() {
    # Desktop profile
    cat > "$PROFILE_DIR/desktop.conf" << 'EOF'
[profile]
name=Desktop
description=Desktop PC configuration

[services]
enabled=NetworkManager,display-manager,ssh
disabled=serial-getty@.service

[performance]
cpu_governor=performance
power_management=true

[graphics]
driver=auto
resolution=auto
compositor=auto

[audio]
driver=auto
pulseaudio=true
EOF

    # Mobile profile
    cat > "$PROFILE_DIR/mobile.conf" << 'EOF'
[profile]
name=Mobile
description=Mobile device configuration

[services]
enabled=systemd-networkd,ssh,bluetooth
disabled=display-manager

[performance]
cpu_governor=powersave
power_management=true

[graphics]
driver=fbdev
compositor=minimal

[audio]
driver=minimal
pulseaudio=false
EOF

    # IoT profile
    cat > "$PROFILE_DIR/iot.conf" << 'EOF'
[profile]
name=IoT
description=IoT device configuration

[services]
enabled=serial-getty@.service,systemd-networkd
disabled=display-manager,NetworkManager

[performance]
cpu_governor=powersave
power_management=true

[security]
firewall=enabled
remote_access=limited

[monitoring]
health_checks=enabled
logging=minimal
EOF

    # Server profile
    cat > "$PROFILE_DIR/server.conf" << 'EOF'
[profile]
name=Server
description=Server configuration

[services]
enabled=ssh,systemd-timesyncd,systemd-networkd
disabled=display-manager,NetworkManager

[performance]
cpu_governor=performance
power_management=false

[security]
firewall=enabled
selinux=enabled
remote_access=ssh_only

[monitoring]
health_checks=enabled
logging=detailed
EOF
}

# Function to detect hardware
detect_hardware() {
    log "INFO" "Detecting hardware configuration..."
    
    # Run hardware detection
    if [[ -f "$CONFIG_DIR/hardware-detector.sh" ]]; then
        echo
        print_color $CYAN "Hardware Detection Report"
        echo "========================"
        bash "$CONFIG_DIR/hardware-detector.sh"
        echo
    fi
    
    # Determine hardware profile
    local arch=$(uname -m)
    local memory_gb=$(( $(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024 ))
    local cpu_cores=$(nproc)
    
    echo "Hardware Summary:"
    echo "  Architecture: $arch"
    echo "  CPU Cores: $cpu_cores"
    echo "  Memory: ${memory_gb}GB"
    
    # Auto-detect profile based on hardware
    case $arch in
        "x86_64"|"amd64")
            if [[ $memory_gb -ge 8 ]] && [[ $cpu_cores -ge 4 ]]; then
                HARDWARE_PROFILE="server"
            else
                HARDWARE_PROFILE="desktop"
            fi
            ;;
        "aarch64"|"arm64")
            HARDWARE_PROFILE="mobile"
            ;;
        "riscv64")
            HARDWARE_PROFILE="iot"
            ;;
        *)
            HARDWARE_PROFILE="desktop"
            ;;
    esac
    
    echo "  Recommended Profile: $HARDWARE_PROFILE"
    
    # Create hardware state file
    create_hardware_state
    
    log "SUCCESS" "Hardware detection completed"
}

# Function to create hardware state
create_hardware_state() {
    cat > "$STATE_FILE" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "hostname": "$(hostname)",
    "architecture": "$(uname -m)",
    "kernel": "$(uname -r)",
    "cpu_cores": $(nproc),
    "memory_gb": $(( $(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024 )),
    "recommended_profile": "$HARDWARE_PROFILE",
    "detected_interfaces": [
$(ls /sys/class/net/ | grep -v lo | while read iface; do
    echo "        \"$iface\""
done | paste -sd, -)
    ],
    "storage_devices": [
$(lsblk -p -d -o name,size,model | grep disk | while read line; do
    echo "        {\"device\": \"$(echo $line | awk '{print $1}')\", \"size\": \"$(echo $line | awk '{print $2}')\", \"model\": \"$(echo $line | awk '{print $3}')\"}"
done | paste -sd, -)
    ]
}
EOF
}

# Function to configure network
configure_network() {
    local profile="${1:-$CONFIG_PROFILE}"
    
    log "INFO" "Configuring network for profile: $profile"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would configure network for profile: $profile"
        return 0
    fi
    
    # Get network configuration from profile
    local method="dhcp"
    local interface="auto"
    
    case $profile in
        "desktop"|"mobile")
            if command -v NetworkManager &> /dev/null; then
                systemctl enable NetworkManager
                log "SUCCESS" "NetworkManager enabled"
            else
                configure_dhcp
            fi
            ;;
        "server"|"iot")
            configure_dhcp
            ;;
    esac
    
    # Configure hostname
    if [[ ! -f /etc/hostname ]]; then
        echo "$(hostname)" > /etc/hostname
        log "SUCCESS" "Hostname configured"
    fi
}

# Function to configure DHCP
configure_dhcp() {
    log "INFO" "Configuring DHCP network"
    
    # Create systemd network configuration
    mkdir -p /etc/systemd/network
    
    cat > /etc/systemd/network/20-dhcp.network << 'EOF'
[Match]
Name=*

[Network]
DHCP=yes
EOF
    
    systemctl enable systemd-networkd
    systemctl enable systemd-resolved
    
    log "SUCCESS" "DHCP network configured"
}

# Function to configure services
configure_services() {
    local profile="${1:-$CONFIG_PROFILE}"
    
    log "INFO" "Configuring services for profile: $profile"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would configure services for profile: $profile"
        return 0
    fi
    
    # Read services configuration
    local services_enabled=()
    local services_disabled=()
    
    case $profile in
        "desktop")
            services_enabled=("NetworkManager" "display-manager" "ssh")
            services_disabled=("serial-getty@.service")
            ;;
        "mobile")
            services_enabled=("systemd-networkd" "ssh" "bluetooth")
            services_disabled=("display-manager")
            ;;
        "iot")
            services_enabled=("serial-getty@.service" "systemd-networkd")
            services_disabled=("display-manager" "NetworkManager")
            ;;
        "server")
            services_enabled=("ssh" "systemd-timesyncd" "systemd-networkd")
            services_disabled=("display-manager" "NetworkManager")
            ;;
    esac
    
    # Enable services
    for service in "${services_enabled[@]}"; do
        if systemctl list-unit-files | grep -q "$service"; then
            systemctl enable "$service" 2>/dev/null || log "WARN" "Failed to enable service: $service"
            log "INFO" "Enabled service: $service"
        fi
    done
    
    # Disable services
    for service in "${services_disabled[@]}"; do
        if systemctl list-unit-files | grep -q "$service"; then
            systemctl disable "$service" 2>/dev/null || log "WARN" "Failed to disable service: $service"
            log "INFO" "Disabled service: $service"
        fi
    done
    
    log "SUCCESS" "Service configuration completed"
}

# Function to configure graphics
configure_graphics() {
    local profile="${1:-$CONFIG_PROFILE}"
    
    log "INFO" "Configuring graphics for profile: $profile"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would configure graphics for profile: $profile"
        return 0
    fi
    
    case $profile in
        "desktop"|"mobile")
            # Configure graphics for desktop/mobile
            mkdir -p /etc/X11
            
            # Auto-detect graphics driver
            local driver="fbdev"
            if lspci | grep -i nvidia &>/dev/null; then
                driver="nouveau"
            elif lspci | grep -i amd &>/dev/null; then
                driver="radeon"
            elif lspci | grep -i intel &>/dev/null; then
                driver="intel"
            fi
            
            echo "Driver: $driver" > /etc/X11/graphics.conf
            log "INFO" "Graphics driver set to: $driver"
            ;;
        "iot"|"server")
            # Disable graphics for server/IoT
            systemctl disable display-manager 2>/dev/null || true
            log "INFO" "Graphics disabled for $profile"
            ;;
    esac
    
    log "SUCCESS" "Graphics configuration completed"
}

# Function to configure audio
configure_audio() {
    local profile="${1:-$CONFIG_PROFILE}"
    
    log "INFO" "Configuring audio for profile: $profile"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would configure audio for profile: $profile"
        return 0
    fi
    
    case $profile in
        "desktop"|"mobile")
            # Enable audio for desktop/mobile
            if command -v pulseaudio &> /dev/null; then
                systemctl enable pulseaudio 2>/dev/null || true
                log "INFO" "PulseAudio enabled"
            fi
            
            # Set default audio driver
            mkdir -p /etc/asound.conf
            cat > /etc/asound.conf << 'EOF'
pcm.!default {
    type pulse
    device default
}
ctl.!default {
    type pulse
    device default
}
EOF
            ;;
        "iot"|"server")
            # Minimal audio for server/IoT
            log "INFO" "Audio minimized for $profile"
            ;;
    esac
    
    log "SUCCESS" "Audio configuration completed"
}

# Function to configure performance
configure_performance() {
    local profile="${1:-$CONFIG_PROFILE}"
    
    log "INFO" "Configuring performance for profile: $profile"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would configure performance for profile: $profile"
        return 0
    fi
    
    case $profile in
        "desktop"|"server")
            # Performance mode
            if command -v cpupower &> /dev/null; then
                cpupower frequency-set -g performance 2>/dev/null || true
                log "INFO" "CPU governor set to performance"
            fi
            ;;
        "mobile"|"iot")
            # Power saving mode
            if command -v cpupower &> /dev/null; then
                cpupower frequency-set -g powersave 2>/dev/null || true
                log "INFO" "CPU governor set to powersave"
            fi
            ;;
    esac
    
    # Configure I/O scheduler
    for device in /sys/block/*/queue/scheduler; do
        local current_scheduler=$(cat "$device")
        local preferred_scheduler="mq-deadline"
        
        case $profile in
            "desktop"|"server")
                preferred_scheduler="noop"
                ;;
            "mobile"|"iot")
                preferred_scheduler="deadline"
                ;;
        esac
        
        if [[ "$current_scheduler" == *"$preferred_scheduler"* ]]; then
            echo "$preferred_scheduler" > "$device" 2>/dev/null || true
        fi
    done
    
    log "SUCCESS" "Performance configuration completed"
}

# Function to apply configuration
apply_configuration() {
    local profile="${1:-$CONFIG_PROFILE}"
    
    if [[ -z "$profile" ]]; then
        profile="$HARDWARE_PROFILE"
    fi
    
    log "INFO" "Applying configuration profile: $profile"
    
    # Save current configuration state
    if [[ -f "$STATE_FILE" ]]; then
        cp "$STATE_FILE" "$STATE_FILE.backup"
    fi
    
    # Apply configuration steps
    configure_network "$profile"
    configure_services "$profile"
    configure_graphics "$profile"
    configure_audio "$profile"
    configure_performance "$profile"
    
    # Update state file
    update_system_state "$profile"
    
    log "SUCCESS" "Configuration applied successfully"
}

# Function to update system state
update_system_state() {
    local profile="$1"
    
    # Add current configuration to state
    local current_state=$(cat "$STATE_FILE")
    echo "$current_state" | jq --arg profile "$profile" '. + {current_profile: $profile, configured_at: now | todatetime}' > "$STATE_FILE.tmp"
    mv "$STATE_FILE.tmp" "$STATE_FILE"
}

# Function to backup configuration
backup_configuration() {
    local backup_dir="/var/backups/multios/config-$(date +%Y%m%d-%H%M%S)"
    
    log "INFO" "Backing up system configuration..."
    
    mkdir -p "$backup_dir"
    
    # Backup configuration files
    cp -r "$CONFIG_DIR" "$backup_dir/config/"
    cp "$STATE_FILE" "$backup_dir/state.json" 2>/dev/null || true
    cp /etc/hostname "$backup_dir/hostname" 2>/dev/null || true
    cp -r /etc/systemd/network "$backup_dir/network/" 2>/dev/null || true
    
    # Create backup manifest
    cat > "$backup_dir/manifest.txt" << EOF
Configuration Backup
Generated: $(date)
Hostname: $(hostname)
Profile: $CONFIG_PROFILE

Files included:
EOF
    
    find "$backup_dir" -type f >> "$backup_dir/manifest.txt"
    
    log "SUCCESS" "Configuration backed up to: $backup_dir"
    echo "$backup_dir"
}

# Function to restore configuration
restore_configuration() {
    local backup_dir="$1"
    
    if [[ ! -d "$backup_dir" ]]; then
        log "ERROR" "Backup directory not found: $backup_dir"
        return 1
    fi
    
    log "INFO" "Restoring configuration from: $backup_dir"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would restore configuration from: $backup_dir"
        return 0
    fi
    
    # Restore configuration files
    cp -r "$backup_dir/config"/* "$CONFIG_DIR/"
    
    if [[ -f "$backup_dir/state.json" ]]; then
        cp "$backup_dir/state.json" "$STATE_FILE"
    fi
    
    if [[ -f "$backup_dir/hostname" ]]; then
        cp "$backup_dir/hostname" /etc/hostname
    fi
    
    if [[ -d "$backup_dir/network" ]]; then
        cp -r "$backup_dir/network"/* /etc/systemd/network/
    fi
    
    log "SUCCESS" "Configuration restored successfully"
}

# Function to show configuration status
show_status() {
    echo "MultiOS Configuration Status"
    echo "============================"
    echo
    
    # Show current state
    if [[ -f "$STATE_FILE" ]]; then
        echo "Current Configuration:"
        cat "$STATE_FILE" | jq -r 'to_entries | map("\(.key): \(.value)") | .[]' 2>/dev/null || echo "State file exists but cannot be parsed"
    else
        echo "No configuration state found"
    fi
    
    echo
    echo "Configuration Profile: ${CONFIG_PROFILE:-none}"
    echo "Hardware Profile: ${HARDWARE_PROFILE:-none}"
    echo
    
    # Show system services status
    echo "Essential Services Status:"
    for service in systemd-networkd systemd-resolved ssh; do
        if systemctl is-enabled "$service" &>/dev/null; then
            echo "  ✓ $service: enabled"
        else
            echo "  ✗ $service: disabled"
        fi
    done
    
    echo
    echo "Network Configuration:"
    if systemctl is-enabled systemd-networkd &>/dev/null; then
        echo "  Method: DHCP"
    else
        echo "  Method: NetworkManager (if enabled)"
    fi
    
    echo
    echo "Performance Settings:"
    if command -v cpupower &> /dev/null; then
        local governor=$(cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null | head -1)
        echo "  CPU Governor: ${governor:-unknown}"
    fi
    
    echo
}

# Function to reset configuration
reset_configuration() {
    log "INFO" "Resetting system configuration..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would reset configuration to defaults"
        return 0
    fi
    
    # Reset to hardware-detected profile
    detect_hardware
    apply_configuration "$HARDWARE_PROFILE"
    
    log "SUCCESS" "Configuration reset completed"
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: multios-config <operation> [options]

MultiOS System Configuration Tool v$CONFIG_TOOL_VERSION

OPERATIONS:
    detect              Detect hardware and suggest configuration
    configure           Apply configuration for a profile
    backup              Backup current configuration
    restore <dir>       Restore configuration from backup
    status              Show current configuration status
    reset               Reset configuration to defaults

PROFILES:
    desktop             Desktop PC configuration
    mobile              Mobile device configuration
    iot                 IoT device configuration
    server              Server configuration

OPTIONS:
    --profile PROFILE   Configuration profile to use
    --auto              Auto-detect and apply configuration
    --dry-run           Show what would be done
    --verbose           Verbose output
    --force             Force operation
    --help              Show this help message

EXAMPLES:
    multios-config detect
    multios-config configure --profile desktop
    multios-config auto
    multios-config backup
    multios-config restore /var/backups/multios/config-20241201-120000
    multios-config status

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            detect)
                OPERATION="$OP_DETECT"
                shift
                ;;
            configure)
                OPERATION="$OP_CONFIGURE"
                shift
                ;;
            backup)
                OPERATION="$OP_BACKUP"
                shift
                ;;
            restore)
                OPERATION="$OP_RESTORE"
                BACKUP_DIR="$2"
                shift 2
                ;;
            status)
                OPERATION="$OP_STATUS"
                shift
                ;;
            reset)
                OPERATION="$OP_RESET"
                shift
                ;;
            auto)
                OPERATION="$OP_DETECT"
                AUTO_CONFIG=true
                shift
                ;;
            --profile)
                CONFIG_PROFILE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --force)
                FORCE=true
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

# Main function
main() {
    # Initialize configuration system
    init_config_system
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Execute operation
    case "${OPERATION:-$OP_DETECT}" in
        "$OP_DETECT")
            detect_hardware
            
            if [[ "$AUTO_CONFIG" == "true" ]]; then
                CONFIG_PROFILE="$HARDWARE_PROFILE"
                apply_configuration "$CONFIG_PROFILE"
            fi
            ;;
        "$OP_CONFIGURE")
            detect_hardware
            
            if [[ -z "$CONFIG_PROFILE" ]]; then
                CONFIG_PROFILE="$HARDWARE_PROFILE"
                log "WARN" "No profile specified, using hardware-detected: $CONFIG_PROFILE"
            fi
            
            apply_configuration "$CONFIG_PROFILE"
            ;;
        "$OP_BACKUP")
            check_root
            backup_configuration
            ;;
        "$OP_RESTORE")
            check_root
            if [[ -z "$BACKUP_DIR" ]]; then
                print_color $RED "Error: restore requires backup directory"
                show_usage
                exit 1
            fi
            restore_configuration "$BACKUP_DIR"
            ;;
        "$OP_STATUS")
            detect_hardware
            show_status
            ;;
        "$OP_RESET")
            check_root
            reset_configuration
            ;;
        *)
            print_color $RED "Error: Unknown operation"
            show_usage
            exit 1
            ;;
    esac
    
    log "SUCCESS" "Operation completed successfully"
}

# Run main function
main "$@"