#!/bin/bash
# MultiOS Installation and Deployment Tools Main Entry Point
# Centralized interface for all installation, deployment, and recovery tools

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
TOOL_VERSION="1.0.0"
BASE_DIR="$(dirname "$(dirname "$(readlink -f "$0")")")"
LIB_DIR="$BASE_DIR/installation_deployment"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/multios"

# Function to print colored output
print_color() {
    local color=$1
    shift
    echo -e "${color}$@${NC}"
}

# Function to show banner
show_banner() {
    print_color $CYAN "╔══════════════════════════════════════════════════════════════╗"
    print_color $CYAN "║                MultiOS Installation & Deployment            ║"
    print_color $CYAN "║                           Tools Suite                        ║"
    print_color $CYAN "║                         Version $TOOL_VERSION                        ║"
    print_color $CYAN "╚══════════════════════════════════════════════════════════════╝"
    echo
}

# Function to show help
show_help() {
    cat << EOF
MultiOS Installation and Deployment Tools v$TOOL_VERSION

USAGE:
    multios-tools <command> [options]

COMMANDS:

INSTALLATION:
    install-desktop    Install MultiOS on desktop PCs
    install-mobile     Install MultiOS on mobile devices
    install-iot        Install MultiOS on IoT devices
    auto-install       Automated installation

MEDIA CREATION:
    create-iso         Create bootable ISO
    create-usb         Create bootable USB
    create-media       Create installation media (ISO/USB)

PACKAGE MANAGEMENT:
    pkg-install        Install packages
    pkg-search         Search packages
    pkg-update         Update package database
    pkg-list           List installed packages

CONFIGURATION:
    system-config      System configuration tool
    detect-hardware    Hardware detection
    configure-profile  Apply configuration profile

DEPLOYMENT:
    bulk-deploy        Deploy multiple devices
    enterprise-deploy  Enterprise deployment
    network-deploy     Network deployment

RECOVERY & BACKUP:
    system-recovery    System recovery tools
    create-backup      Create system backup
    restore-backup     Restore from backup
    system-check       Check system health

UTILS:
    utils              Show available utilities
    status             Show system status
    help               Show this help

EXAMPLES:
    multios-tools install-desktop --device /dev/sda --username john
    multios-tools create-iso --output multios.iso --type desktop
    multios-tools bulk-deploy --manifest devices.csv
    multios-tools system-config --profile desktop

For detailed help on any command:
    multios-tools <command> --help

For more information:
    man multios-tools
    https://docs.multios.org/installation

EOF
}

# Function to install tools
install_tools() {
    if [[ $EUID -ne 0 ]]; then
        print_color $RED "Error: Installation requires root privileges"
        exit 1
    fi
    
    print_color $GREEN "Installing MultiOS Installation & Deployment Tools..."
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR"
    
    # Install main scripts
    local scripts=(
        "installation/desktop_installer.sh:desktop_installer"
        "installation/mobile_installer.sh:mobile_installer"
        "installation/iot_installer.sh:iot_installer"
        "media_creation/create_bootable_media.sh:create_media"
        "package_manager/multios-pkg.sh:multios-pkg"
        "configuration/system_config.sh:multios-config"
        "deployment/enterprise_deploy.sh:multios-deploy"
        "recovery/system_recovery.sh:multios-recover"
        "automation/automated_installation.sh:multios-automation"
    )
    
    for script_info in "${scripts[@]}"; do
        local source_file="${script_info%%:*}"
        local target_name="${script_info##*:}"
        local source_path="$LIB_DIR/$source_file"
        local target_path="$INSTALL_DIR/$target_name"
        
        if [[ -f "$source_path" ]]; then
            cp "$source_path" "$target_path"
            chmod +x "$target_path"
            print_color $GREEN "✓ Installed: $target_name"
        else
            print_color $RED "✗ Not found: $source_path"
        fi
    done
    
    # Create symlinks for convenience
    ln -sf "$INSTALL_DIR/automated_installation.sh" "$INSTALL_DIR/multios-auto"
    ln -sf "$INSTALL_DIR/create_media.sh" "$INSTALL_DIR/multios-media"
    
    print_color $GREEN "Installation completed!"
    echo "Tools are now available in $INSTALL_DIR"
    echo "Make sure $INSTALL_DIR is in your PATH"
}

# Function to verify installation
verify_installation() {
    print_color $BLUE "Verifying MultiOS Tools Installation..."
    
    local tools=(
        "desktop_installer.sh"
        "mobile_installer.sh"
        "iot_installer.sh"
        "create_media.sh"
        "multios-pkg"
        "multios-config"
        "multios-deploy"
        "multios-recover"
        "multios-automation"
    )
    
    local missing=()
    
    for tool in "${tools[@]}"; do
        if [[ -x "$INSTALL_DIR/$tool" ]]; then
            print_color $GREEN "✓ $tool"
        else
            missing+=("$tool")
            print_color $RED "✗ $tool"
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        echo
        print_color $YELLOW "Missing tools: ${missing[*]}"
        echo "Run: multios-tools install"
        return 1
    else
        echo
        print_color $GREEN "All tools installed successfully!"
        return 0
    fi
}

# Function to show available utilities
show_utils() {
    echo "MultiOS Tools - Available Utilities"
    echo "==================================="
    echo
    
    # Find all scripts and tools
    echo "Installation Tools:"
    for script in "$LIB_DIR/installation/"*.sh; do
        if [[ -f "$script" ]]; then
            local name=$(basename "$script" .sh)
            echo "  • $(basename "$script")"
        fi
    done
    
    echo
    echo "Media Creation:"
    for script in "$LIB_DIR/media_creation/"*.sh; do
        if [[ -f "$script" ]]; then
            echo "  • $(basename "$script")"
        fi
    done
    
    echo
    echo "Configuration Tools:"
    for script in "$LIB_DIR/configuration/"*.sh; do
        if [[ -f "$script" ]]; then
            echo "  • $(basename "$script")"
        fi
    done
    
    echo
    echo "Deployment Tools:"
    for script in "$LIB_DIR/deployment/"*.sh; do
        if [[ -f "$script" ]]; then
            echo "  • $(basename "$script")"
        fi
    done
    
    echo
    echo "Recovery Tools:"
    for script in "$LIB_DIR/recovery/"*.sh; do
        if [[ -f "$script" ]]; then
            echo "  • $(basename "$script")"
        fi
    done
    
    echo
    echo "Package Manager:"
    for script in "$LIB_DIR/package_manager/"*.sh; do
        if [[ -f "$script" ]]; then
            echo "  • $(basename "$script")"
        fi
    done
    
    echo
    echo "Automation Tools:"
    for script in "$LIB_DIR/automation/"*.sh; do
        if [[ -f "$script" ]]; then
            echo "  • $(basename "$script")"
        fi
    done
}

# Function to check system requirements
check_requirements() {
    echo "System Requirements Check"
    echo "========================"
    echo
    
    # Check architecture
    local arch=$(uname -m)
    echo "Architecture: $arch"
    
    local supported_archs=("x86_64" "aarch64" "arm64" "riscv64")
    if [[ " ${supported_archs[@]} " =~ " $arch " ]]; then
        print_color $GREEN "✓ Supported architecture"
    else
        print_color $RED "✗ Unsupported architecture"
    fi
    
    echo
    
    # Check available disk space
    local available_gb=$(( $(df / | tail -1 | awk '{print $4}') / 1024 / 1024 ))
    echo "Available disk space: ${available_gb}GB"
    
    if [[ $available_gb -gt 10 ]]; then
        print_color $GREEN "✓ Sufficient disk space"
    elif [[ $available_gb -gt 2 ]]; then
        print_color $YELLOW "⚠ Limited disk space"
    else
        print_color $RED "✗ Insufficient disk space"
    fi
    
    echo
    
    # Check memory
    local memory_mb=$(( $(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 ))
    echo "Memory: ${memory_mb}MB"
    
    if [[ $memory_mb -gt 2048 ]]; then
        print_color $GREEN "✓ Sufficient memory"
    elif [[ $memory_mb -gt 1024 ]]; then
        print_color $YELLOW "⚠ Limited memory"
    else
        print_color $RED "✗ Insufficient memory"
    fi
    
    echo
    
    # Check for required tools
    echo "Required Tools:"
    local required_tools=("tar" "wget" "curl" "parted" "mkfs.vfat" "mkfs.ext4")
    
    for tool in "${required_tools[@]}"; do
        if command -v "$tool" &> /dev/null; then
            print_color $GREEN "✓ $tool"
        else
            print_color $RED "✗ $tool"
        fi
    done
    
    echo
    
    # Check permissions
    if [[ $EUID -eq 0 ]]; then
        print_color $GREEN "✓ Running as root (required for installation)"
    else
        print_color $YELLOW "⚠ Not running as root (some operations may be limited)"
    fi
}

# Function to show system status
show_status() {
    echo "MultiOS Tools System Status"
    echo "==========================="
    echo
    
    # Check if tools are installed
    verify_installation
    local install_status=$?
    
    echo
    echo "Tool Usage Examples:"
    echo "-------------------"
    echo
    echo "1. Desktop Installation:"
    echo "   multios-tools install-desktop --device /dev/sda --username john"
    echo
    echo "2. Mobile Installation:"
    echo "   multios-tools install-mobile --device /dev/mmcblk0"
    echo
    echo "3. IoT Installation:"
    echo "   multios-tools install-iot --device /dev/mtd0 --profile minimal"
    echo
    echo "4. Create Installation Media:"
    echo "   multios-tools create-iso --type iso --output multios.iso"
    echo
    echo "5. Package Management:"
    echo "   multios-pkg install multios-desktop"
    echo "   multios-pkg search editor"
    echo
    echo "6. System Configuration:"
    echo "   multios-config detect"
    echo "   multios-config configure --profile desktop"
    echo
    echo "7. Enterprise Deployment:"
    echo "   multios-deploy bulk-deploy --manifest devices.csv"
    echo
    echo "8. System Recovery:"
    echo "   multios-recover backup system"
    echo "   multios-recover restore backup-20241201"
    echo
    
    return $install_status
}

# Function to create example configurations
create_examples() {
    local examples_dir="$HOME/.config/multios"
    
    print_color $GREEN "Creating example configurations in $examples_dir..."
    
    mkdir -p "$examples_dir"/{installation,deployment,profiles}
    
    # Installation example
    cat > "$examples_dir/installation/desktop-example.conf" << 'EOF'
# Desktop Installation Example
DEVICE=/dev/sda
USERNAME=desktop
PASSWORD=changeme
TIMEZONE=America/New_York
KEYBOARD=us
PROFILE=desktop
EOF

    # Deployment example
    cat > "$examples_dir/deployment/enterprise-example.csv" << 'EOF'
# Enterprise Deployment Example
# hostname,ip_address,mac_address,device_type,profile,target_device
desktop-01,192.168.1.101,00:11:22:33:44:55,desktop,desktop,/dev/sda
server-01,192.168.1.201,00:11:22:33:44:60,server,server,/dev/sda
iot-01,192.168.1.151,00:11:22:33:44:70,iot,iot,/dev/mmcblk0
EOF

    # Profile example
    cat > "$examples_dir/profiles/minimal.conf" << 'EOF'
# Minimal Configuration Profile
[profile]
name=Minimal
description=Minimal system configuration

[network]
method=dhcp

[services]
enabled=systemd-networkd,systemd-resolved
disabled=display-manager,NetworkManager

[packages]
base=multios-core
minimal=true
EOF

    print_color $GREEN "Example configurations created:"
    echo "  • $examples_dir/installation/desktop-example.conf"
    echo "  • $examples_dir/deployment/enterprise-example.csv"
    echo "  • $examples_dir/profiles/minimal.conf"
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            install-desktop)
                shift
                if [[ -x "$INSTALL_DIR/desktop_installer.sh" ]]; then
                    exec "$INSTALL_DIR/desktop_installer.sh" "$@"
                else
                    print_color $RED "Error: desktop_installer.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            install-mobile)
                shift
                if [[ -x "$INSTALL_DIR/mobile_installer.sh" ]]; then
                    exec "$INSTALL_DIR/mobile_installer.sh" "$@"
                else
                    print_color $RED "Error: mobile_installer.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            install-iot)
                shift
                if [[ -x "$INSTALL_DIR/iot_installer.sh" ]]; then
                    exec "$INSTALL_DIR/iot_installer.sh" "$@"
                else
                    print_color $RED "Error: iot_installer.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            auto-install)
                shift
                if [[ -x "$INSTALL_DIR/automated_installation.sh" ]]; then
                    exec "$INSTALL_DIR/automated_installation.sh" "$@"
                else
                    print_color $RED "Error: automated_installation.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            create-iso)
                shift
                if [[ -x "$INSTALL_DIR/create_media.sh" ]]; then
                    exec "$INSTALL_DIR/create_media.sh" "$@"
                else
                    print_color $RED "Error: create_media.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            create-usb)
                shift
                if [[ -x "$INSTALL_DIR/create_media.sh" ]]; then
                    exec "$INSTALL_DIR/create_media.sh" --type usb "$@"
                else
                    print_color $RED "Error: create_media.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            create-media)
                shift
                if [[ -x "$INSTALL_DIR/create_media.sh" ]]; then
                    exec "$INSTALL_DIR/create_media.sh" "$@"
                else
                    print_color $RED "Error: create_media.sh not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            pkg-install|pkg-search|pkg-update|pkg-list)
                local pkg_cmd="${1#pkg-}"
                shift
                if [[ -x "$INSTALL_DIR/multios-pkg" ]]; then
                    exec "$INSTALL_DIR/multios-pkg" "$pkg_cmd" "$@"
                else
                    print_color $RED "Error: multios-pkg not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            system-config)
                shift
                if [[ -x "$INSTALL_DIR/multios-config" ]]; then
                    exec "$INSTALL_DIR/multios-config" "$@"
                else
                    print_color $RED "Error: multios-config not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            detect-hardware)
                shift
                if [[ -x "$INSTALL_DIR/multios-config" ]]; then
                    exec "$INSTALL_DIR/multios-config" detect "$@"
                else
                    print_color $RED "Error: multios-config not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            configure-profile)
                shift
                if [[ -x "$INSTALL_DIR/multios-config" ]]; then
                    exec "$INSTALL_DIR/multios-config" configure "$@"
                else
                    print_color $RED "Error: multios-config not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            bulk-deploy)
                shift
                if [[ -x "$INSTALL_DIR/multios-deploy" ]]; then
                    exec "$INSTALL_DIR/multios-deploy" bulk-deploy "$@"
                else
                    print_color $RED "Error: multios-deploy not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            enterprise-deploy)
                shift
                if [[ -x "$INSTALL_DIR/multios-deploy" ]]; then
                    exec "$INSTALL_DIR/multios-deploy" enterprise-deploy "$@"
                else
                    print_color $RED "Error: multios-deploy not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            network-deploy)
                shift
                if [[ -x "$INSTALL_DIR/multios-deploy" ]]; then
                    exec "$INSTALL_DIR/multios-deploy" network-deploy "$@"
                else
                    print_color $RED "Error: multios-deploy not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            system-recovery)
                shift
                if [[ -x "$INSTALL_DIR/multios-recover" ]]; then
                    exec "$INSTALL_DIR/multios-recover" "$@"
                else
                    print_color $RED "Error: multios-recover not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            create-backup)
                shift
                if [[ -x "$INSTALL_DIR/multios-recover" ]]; then
                    exec "$INSTALL_DIR/multios-recover" backup "$@"
                else
                    print_color $RED "Error: multios-recover not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            restore-backup)
                shift
                if [[ -x "$INSTALL_DIR/multios-recover" ]]; then
                    exec "$INSTALL_DIR/multios-recover" restore "$@"
                else
                    print_color $RED "Error: multios-recover not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            system-check)
                shift
                if [[ -x "$INSTALL_DIR/multios-recover" ]]; then
                    exec "$INSTALL_DIR/multios-recover" check "$@"
                else
                    print_color $RED "Error: multios-recover not found. Run: multios-tools install"
                    exit 1
                fi
                ;;
            utils)
                show_utils
                ;;
            status)
                show_status
                exit $?
                ;;
            install)
                install_tools
                ;;
            verify)
                verify_installation
                exit $?
                ;;
            check-reqs)
                check_requirements
                ;;
            examples)
                create_examples
                ;;
            help|--help|-h)
                show_help
                exit 0
                ;;
            *)
                print_color $RED "Error: Unknown command: $1"
                echo
                show_help
                exit 1
                ;;
        esac
    done
}

# Main function
main() {
    # Show banner if no arguments or if help is requested
    if [[ $# -eq 0 ]] || [[ "$1" == "help" ]] || [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
        show_banner
        show_help
        exit 0
    fi
    
    # Parse and execute command
    parse_arguments "$@"
}

# Run main function
main "$@"