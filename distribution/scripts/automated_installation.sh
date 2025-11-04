#!/bin/bash
# MultiOS Automated Installation System
# Automated installation and configuration tools

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
AUTOMATION_VERSION="1.0.0"
CONFIG_DIR="/etc/multios/automation"
CONFIG_FILE="$CONFIG_DIR/installation.conf"
LOG_FILE="/var/log/multios-automation.log"
TEMPLATE_DIR="$CONFIG_DIR/templates"
SCRIPT_DIR="/usr/local/bin"

# Installation types
TYPE_DESKTOP="desktop"
TYPE_MOBILE="mobile"
TYPE_IOT="iot"
TYPE_SERVER="server"
TYPE_MINIMAL="minimal"

# Variables
INSTALL_TYPE=""
TARGET_DEVICE=""
CONFIG_PROFILE=""
AUTO_CONFIGURE=true
POST_INSTALL_SCRIPTS=()
PACKAGES_TO_INSTALL=()
NETWORK_CONFIG=()
USER_ACCOUNTS=()
SERVICE_CONFIG=()
DRY_RUN=false
VERBOSE=false
INTERACTIVE=true
LOG_LEVEL="INFO"

# Function to log messages
log() {
    local level="$1"
    shift
    local message="$@"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [$level] $1" "$message" >> "$LOG_FILE"
    
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

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_color $RED "Error: This script must be run as root"
        exit 1
    fi
}

# Function to create directories
create_directories() {
    mkdir -p "$CONFIG_DIR" "$TEMPLATE_DIR" "$(dirname "$LOG_FILE")"
}

# Function to initialize automation system
init_automation() {
    log "INFO" "Initializing MultiOS Automation System v$AUTOMATION_VERSION"
    
    create_directories
    
    # Create default configuration
    if [[ ! -f "$CONFIG_FILE" ]]; then
        create_default_config
    fi
    
    log "SUCCESS" "Automation system initialized"
}

# Function to create default configuration
create_default_config() {
    cat > "$CONFIG_FILE" << EOF
# MultiOS Automated Installation Configuration

[general]
default_type=$TYPE_DESKTOP
log_level=INFO
verbose=false
auto_configure=true

[profiles]
# Installation type profiles
[desktop]
device=/dev/sda
packages=multios-desktop,multios-gui,multios-network
user_config=desktop-user
network=dhcp

[mobile]
device=/dev/mmcblk0
packages=multios-mobile,multios-touch
user_config=mobile-user
network=dhcp

[iot]
device=current
packages=multios-iot,multios-iot-tools
user_config=iot-user
network=static
serial_console=true

[server]
device=/dev/sda
packages=multios-server,multios-services
user_config=server-user
network=static
services=enabled

[minimal]
device=current
packages=multios-core
user_config=minimal-user
network=minimal

[network]
# Network configuration templates
[dhcp]
method=dhcp
interface=auto

[static]
method=static
interface=eth0
ip=192.168.1.100
netmask=255.255.255.0
gateway=192.168.1.1
dns=8.8.8.8

[users]
# User account templates
[desktop-user]
username=desktop
password=changeme
shell=/bin/bash
groups=users,sudo

[mobile-user]
username=mobile
password=changeme
shell=/bin/bash
groups=users

[iot-user]
username=iot
password=changeme
shell=/bin/sh
groups=users

[server-user]
username=admin
password=changeme
shell=/bin/bash
groups=users,sudo,ssh

[minimal-user]
username=user
password=changeme
shell=/bin/sh
groups=users
EOF
}

# Function to load configuration
load_config() {
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log "WARN" "Configuration file not found"
        return 1
    fi
    
    # Source configuration (simplified parsing)
    source "$CONFIG_FILE" 2>/dev/null || {
        log "ERROR" "Failed to load configuration"
        return 1
    }
    
    log "DEBUG" "Configuration loaded successfully"
}

# Function to parse configuration section
parse_config_section() {
    local section="$1"
    local key="$2"
    
    # This is simplified - would use proper config parsing
    grep -A 20 "^\[$section\]" "$CONFIG_FILE" | \
        grep "^$key=" | head -1 | cut -d'=' -f2
}

# Function to detect hardware
detect_hardware() {
    log "INFO" "Detecting hardware configuration..."
    
    local arch=$(uname -m)
    local memory=$(free -m | awk 'NR==2{print $2}')
    local disks=$(lsblk -d -o name,size,rota | grep -v NAME)
    local network_interfaces=$(ls /sys/class/net/ | grep -v lo)
    
    echo "Hardware Detection:"
    echo "  Architecture: $arch"
    echo "  Memory: ${memory}MB"
    echo "  Storage devices:"
    echo "$disks" | sed 's/^/    /'
    echo "  Network interfaces: $(echo $network_interfaces | tr ' ' ',')"
    
    # Auto-detect installation type based on hardware
    case $arch in
        "x86_64"|"amd64")
            INSTALL_TYPE="${INSTALL_TYPE:-$TYPE_DESKTOP}"
            TARGET_DEVICE="${TARGET_DEVICE:-/dev/sda}"
            ;;
        "aarch64"|"arm64")
            INSTALL_TYPE="${INSTALL_TYPE:-$TYPE_MOBILE}"
            TARGET_DEVICE="${TARGET_DEVICE:-/dev/mmcblk0}"
            ;;
        "riscv64")
            INSTALL_TYPE="${INSTALL_TYPE:-$TYPE_IOT}"
            TARGET_DEVICE="current"
            ;;
    esac
    
    # Auto-detect device if not specified
    if [[ -z "$TARGET_DEVICE" ]] || [[ "$TARGET_DEVICE" == "auto" ]]; then
        local first_disk=$(echo "$disks" | awk 'NR==1 {print $1}')
        if [[ -n "$first_disk" ]]; then
            TARGET_DEVICE="/dev/$first_disk"
            log "INFO" "Auto-detected target device: $TARGET_DEVICE"
        fi
    fi
    
    log "INFO" "Hardware detection completed"
}

# Function to apply network configuration
apply_network_config() {
    local config_type="${1:-dhcp}"
    log "INFO" "Applying network configuration: $config_type"
    
    local config_section="[network.$config_type]"
    
    case $config_type in
        "dhcp")
            if command -v dhclient &> /dev/null; then
                dhclient 2>/dev/null || log "WARN" "DHCP client not available"
            fi
            ;;
        "static")
            local interface=$(parse_config_section "network.static" "interface")
            local ip=$(parse_config_section "network.static" "ip")
            local netmask=$(parse_config_section "network.static" "netmask")
            local gateway=$(parse_config_section "network.static" "gateway")
            local dns=$(parse_config_section "network.static" "dns")
            
            if [[ -n "$interface" ]] && [[ -n "$ip" ]]; then
                ip addr add "$ip/24" dev "$interface" 2>/dev/null || true
                ip route add default via "$gateway" 2>/dev/null || true
                echo "nameserver $dns" > /etc/resolv.conf
                log "SUCCESS" "Static network configured: $interface -> $ip"
            fi
            ;;
    esac
    
    # Test connectivity
    if ping -c 1 8.8.8.8 &>/dev/null; then
        log "SUCCESS" "Network connectivity verified"
    else
        log "WARN" "Network connectivity test failed"
    fi
}

# Function to create user accounts
create_user_accounts() {
    local users="${1:-user}"
    
    log "INFO" "Creating user accounts..."
    
    for user_config in $users; do
        local username=$(parse_config_section "users.$user_config" "username")
        local password=$(parse_config_section "users.$user_config" "password")
        local shell=$(parse_config_section "users.$user_config" "shell" | sed 's/bash/sh/' | sed 's/sh/sh/')
        local groups=$(parse_config_section "users.$user_config" "groups")
        
        if [[ -n "$username" ]]; then
            log "INFO" "Creating user: $username"
            
            if [[ "$DRY_RUN" == "true" ]]; then
                echo "Would create user: $username"
                echo "  Shell: ${shell:-/bin/sh}"
                echo "  Groups: ${groups:-users}"
            else
                # Create user
                useradd -m -s "${shell:-/bin/sh}" "$username" 2>/dev/null || {
                    log "WARN" "User $username may already exist"
                }
                
                # Set password
                if [[ -n "$password" ]]; then
                    echo "$username:$password" | chpasswd
                fi
                
                # Add to groups
                if [[ -n "$groups" ]]; then
                    usermod -aG "$groups" "$username"
                fi
                
                # Create user directories
                mkdir -p "/home/$username/.config"
                mkdir -p "/home/$username/.local/share"
                
                log "SUCCESS" "User $username created"
            fi
        fi
    done
}

# Function to install packages
install_packages() {
    local packages="$1"
    
    log "INFO" "Installing packages: $packages"
    
    if [[ -z "$packages" ]]; then
        log "INFO" "No packages specified for installation"
        return 0
    fi
    
    # Convert comma-separated list to array
    IFS=',' read -ra PACKAGE_ARRAY <<< "$packages"
    
    for package in "${PACKAGE_ARRAY[@]}"; do
        package=$(echo "$package" | xargs)  # Trim whitespace
        
        if [[ "$DRY_RUN" == "true" ]]; then
            echo "Would install package: $package"
        else
            log "INFO" "Installing package: $package"
            
            # Try using multios-pkg if available
            if command -v multios-pkg &> /dev/null; then
                multios-pkg install "$package" 2>/dev/null || log "WARN" "Failed to install $package"
            else
                # Alternative installation methods
                if [[ "$package" =~ ^http ]]; then
                    # Download and install
                    wget "$package" -O "/tmp/$package" 2>/dev/null || log "WARN" "Failed to download $package"
                else
                    log "WARN" "Package manager not available for: $package"
                fi
            fi
        fi
    done
    
    log "SUCCESS" "Package installation completed"
}

# Function to configure services
configure_services() {
    local services="${1:-}"
    
    log "INFO" "Configuring services..."
    
    if [[ "$services" == "enabled" ]] || [[ "$services" == "minimal" ]]; then
        # Enable essential services
        local essential_services=("systemd-networkd" "systemd-resolved")
        
        if [[ "$services" == "enabled" ]]; then
            essential_services+=("ssh" "systemd-timesyncd")
        fi
        
        for service in "${essential_services[@]}"; do
            if [[ "$DRY_RUN" == "true" ]]; then
                echo "Would enable service: $service"
            else
                systemctl enable "$service" 2>/dev/null || log "WARN" "Failed to enable service: $service"
            fi
        done
    fi
    
    log "SUCCESS" "Service configuration completed"
}

# Function to run post-install scripts
run_post_install_scripts() {
    local scripts="$1"
    
    if [[ -z "$scripts" ]]; then
        return 0
    fi
    
    log "INFO" "Running post-installation scripts: $scripts"
    
    IFS=',' read -ra SCRIPT_ARRAY <<< "$scripts"
    
    for script in "${SCRIPT_ARRAY[@]}"; do
        script=$(echo "$script" | xargs)
        script_path="$SCRIPT_DIR/$script"
        
        if [[ -f "$script_path" ]]; then
            log "INFO" "Running script: $script"
            
            if [[ "$DRY_RUN" == "true" ]]; then
                echo "Would run script: $script"
            else
                chmod +x "$script_path"
                "$script_path" || log "WARN" "Script $script failed"
            fi
        else
            log "WARN" "Post-install script not found: $script"
        fi
    done
}

# Function to execute installation workflow
execute_installation() {
    log "INFO" "Starting automated installation: $INSTALL_TYPE"
    
    local profile_section="[$INSTALL_TYPE]"
    
    # Get configuration from profile
    local device=$(parse_config_section "$profile_section" "device")
    local packages=$(parse_config_section "$profile_section" "packages")
    local user_config=$(parse_config_section "$profile_section" "user_config")
    local network=$(parse_config_section "$profile_section" "network")
    local services=$(parse_config_section "$profile_section" "services")
    local post_scripts=$(parse_config_section "$profile_section" "post_scripts")
    
    # Override with command line arguments
    device="${TARGET_DEVICE:-$device}"
    
    log "INFO" "Installation configuration:"
    echo "  Type: $INSTALL_TYPE"
    echo "  Device: ${device:-current}"
    echo "  Packages: ${packages:-none}"
    echo "  Users: ${user_config:-none}"
    echo "  Network: ${network:-dhcp}"
    echo "  Services: ${services:-none}"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo
        echo "DRY RUN MODE - No changes will be made"
        echo "====================================="
    else
        # Confirm installation
        if [[ "$INTERACTIVE" == "true" ]] && [[ "$AUTO_CONFIGURE" != "true" ]]; then
            echo
            echo -n "Proceed with automated installation? (y/N): "
            read -r confirm
            if [[ ! $confirm =~ ^[Yy]$ ]]; then
                log "INFO" "Installation cancelled by user"
                return 0
            fi
        fi
        
        echo
        print_color $GREEN "Starting installation..."
    fi
    
    # Execute installation steps
    log "INFO" "Executing installation workflow"
    
    if [[ "$device" != "current" ]] && [[ -n "$device" ]]; then
        log "INFO" "Installing to device: $device"
        # Would run actual installation here
    else
        log "INFO" "Installing to current system"
    fi
    
    # Configure network
    apply_network_config "$network"
    
    # Create user accounts
    create_user_accounts "$user_config"
    
    # Install packages
    install_packages "$packages"
    
    # Configure services
    configure_services "$services"
    
    # Run post-install scripts
    run_post_install_scripts "$post_scripts"
    
    log "SUCCESS" "Installation workflow completed"
}

# Function to create installation template
create_template() {
    local template_name="$1"
    local template_type="$2"
    
    log "INFO" "Creating installation template: $template_name"
    
    local template_file="$TEMPLATE_DIR/$template_name.conf"
    
    cat > "$template_file" << EOF
# MultiOS Installation Template: $template_name
# Type: $template_type

[general]
name=$template_name
type=$template_type

[installation]
device=auto
network=dhcp
packages=multios-core
user_config=default-user
services=enabled

[network.dhcp]
method=dhcp

[network.static]
method=static
interface=eth0
ip=192.168.1.100
netmask=255.255.255.0
gateway=192.168.1.1
dns=8.8.8.8

[users.default-user]
username=user
password=changeme
shell=/bin/bash
groups=users

# Custom sections for this template
EOF
    
    log "SUCCESS" "Template created: $template_file"
    echo "$template_file"
}

# Function to apply template
apply_template() {
    local template_file="$1"
    
    if [[ ! -f "$template_file" ]]; then
        log "ERROR" "Template file not found: $template_file"
        return 1
    fi
    
    log "INFO" "Applying template: $template_file"
    
    # Source template configuration
    source "$template_file" 2>/dev/null || {
        log "ERROR" "Failed to load template: $template_file"
        return 1
    }
    
    # Set variables from template
    INSTALL_TYPE="${type:-$TYPE_DESKTOP}"
    TARGET_DEVICE="${device:-auto}"
    PACKAGES_TO_INSTALL=($(echo "$packages" | tr ',' ' '))
    
    # Run installation with template settings
    execute_installation
    
    log "SUCCESS" "Template applied successfully"
}

# Function to validate configuration
validate_config() {
    log "INFO" "Validating configuration..."
    
    local errors=0
    
    # Check configuration file
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log "ERROR" "Configuration file not found: $CONFIG_FILE"
        errors=$((errors + 1))
    fi
    
    # Check installation type
    local valid_types=("$TYPE_DESKTOP" "$TYPE_MOBILE" "$TYPE_IOT" "$TYPE_SERVER" "$TYPE_MINIMAL")
    if [[ ! " ${valid_types[@]} " =~ " ${INSTALL_TYPE} " ]]; then
        log "ERROR" "Invalid installation type: $INSTALL_TYPE"
        errors=$((errors + 1))
    fi
    
    # Check target device
    if [[ "$TARGET_DEVICE" != "auto" ]] && [[ -n "$TARGET_DEVICE" ]]; then
        if [[ ! -b "$TARGET_DEVICE" ]] && [[ "$TARGET_DEVICE" != "current" ]]; then
            log "WARN" "Target device may not exist: $TARGET_DEVICE"
        fi
    fi
    
    if [[ $errors -eq 0 ]]; then
        log "SUCCESS" "Configuration validation passed"
    else
        log "ERROR" "Configuration validation failed with $errors errors"
    fi
    
    return $errors
}

# Function to show configuration
show_config() {
    echo "MultiOS Automation Configuration"
    echo "================================="
    echo
    echo "Configuration File: $CONFIG_FILE"
    echo "Template Directory: $TEMPLATE_DIR"
    echo "Current Settings:"
    echo "  Installation Type: ${INSTALL_TYPE:-not set}"
    echo "  Target Device: ${TARGET_DEVICE:-auto}"
    echo "  Auto Configure: $AUTO_CONFIGURE"
    echo
    echo "Available Templates:"
    ls -1 "$TEMPLATE_DIR"/*.conf 2>/dev/null | while read template; do
        echo "  $(basename "$template" .conf)"
    done || echo "  No templates found"
    echo
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: multios-automation <command> [options]

MultiOS Automated Installation System v$AUTOMATION_VERSION

COMMANDS:
    auto-install         Start automated installation
    validate-config      Validate current configuration
    create-template <name> <type>  Create installation template
    apply-template <template>      Apply installation template
    show-config          Show current configuration
    hardware-detect      Detect hardware and suggest configuration

INSTALLATION TYPES:
    desktop              Desktop PC installation
    mobile               Mobile device installation
    iot                  IoT device installation
    server               Server installation
    minimal              Minimal installation

OPTIONS:
    --type TYPE          Installation type
    --device DEVICE      Target device (e.g., /dev/sda)
    --config PROFILE     Configuration profile
    --template FILE      Template file to apply
    --dry-run            Show what would be done
    --verbose            Verbose output
    --auto               Run without prompts
    --help               Show this help message

EXAMPLES:
    multios-automation auto-install --type desktop --device /dev/sda
    multios-automation create-template server-installation server
    multios-automation apply-template server-installation.conf
    multios-automation validate-config

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            auto-install)
                COMMAND="auto-install"
                shift
                ;;
            validate-config)
                COMMAND="validate-config"
                shift
                ;;
            create-template)
                COMMAND="create-template"
                TEMPLATE_NAME="$2"
                TEMPLATE_TYPE="$3"
                shift 3
                ;;
            apply-template)
                COMMAND="apply-template"
                TEMPLATE_FILE="$2"
                shift 2
                ;;
            show-config)
                COMMAND="show-config"
                shift
                ;;
            hardware-detect)
                COMMAND="hardware-detect"
                shift
                ;;
            --type)
                INSTALL_TYPE="$2"
                shift 2
                ;;
            --device)
                TARGET_DEVICE="$2"
                shift 2
                ;;
            --config)
                CONFIG_PROFILE="$2"
                shift 2
                ;;
            --template)
                TEMPLATE_FILE="$2"
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
            --auto)
                AUTO_CONFIGURE=true
                INTERACTIVE=false
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
    # Initialize automation system
    init_automation
    load_config
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Execute command
    case "${COMMAND:-hardware-detect}" in
        "auto-install")
            check_root
            detect_hardware
            
            if [[ -z "$INSTALL_TYPE" ]]; then
                INSTALL_TYPE="$TYPE_DESKTOP"
                log "WARN" "No installation type specified, using default: $INSTALL_TYPE"
            fi
            
            validate_config || exit 1
            execute_installation
            ;;
        "validate-config")
            validate_config
            ;;
        "create-template")
            if [[ -z "$TEMPLATE_NAME" ]] || [[ -z "$TEMPLATE_TYPE" ]]; then
                print_color $RED "Error: create-template requires name and type"
                show_usage
                exit 1
            fi
            create_template "$TEMPLATE_NAME" "$TEMPLATE_TYPE"
            ;;
        "apply-template")
            if [[ -z "$TEMPLATE_FILE" ]]; then
                print_color $RED "Error: apply-template requires template file"
                show_usage
                exit 1
            fi
            apply_template "$TEMPLATE_FILE"
            ;;
        "show-config")
            show_config
            ;;
        "hardware-detect")
            detect_hardware
            ;;
        *)
            print_color $RED "Error: Unknown command"
            show_usage
            exit 1
            ;;
    esac
    
    log "SUCCESS" "Operation completed successfully"
}

# Run main function
main "$@"