#!/bin/bash
# MultiOS Master Installation Script
# Version: 1.0.0
# Description: Auto-detect system type and run appropriate installation

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

# Detect system type
detect_system_type() {
    header "Detecting System Type..."
    
    # Check if running in container
    if [ -f /.dockerenv ] || grep -q docker /proc/1/cgroup 2>/dev/null; then
        SYSTEM_TYPE="container"
        info "Container environment detected"
    fi
    
    # Check if running in VM
    if [ -f /sys/class/dmi/id/product_name ] || [ -f /sys/class/dmi/id/sys_vendor ]; then
        if grep -qi "virtual\|vmware\|qemu\|kvm\|parallels" /sys/class/dmi/id/* 2>/dev/null; then
            SYSTEM_TYPE="vm"
            info "Virtual machine environment detected"
        fi
    fi
    
    # Check for embedded/IoT indicators
    if [ -f /proc/device-tree/model ] || [ -d /sys/firmware/devicetree ]; then
        if grep -qi "raspberry\|banana\|odroid\|pine" /proc/device-tree/model 2>/dev/null; then
            SYSTEM_TYPE="embedded"
            info "Embedded system detected (Raspberry Pi, etc.)"
            return
        fi
    fi
    
    # Check for server indicators
    if [ -f /etc/redhat-release ] || [ -f /etc/debian_version ]; then
        if systemctl list-units --type=service | grep -qE "ssh|apache|nginx|mysql|postgresql"; then
            SYSTEM_TYPE="server"
            info "Server environment detected"
            return
        fi
    fi
    
    # Default to desktop if no specific type detected
    if [ -z "$SYSTEM_TYPE" ]; then
        SYSTEM_TYPE="desktop"
        info "Desktop/Laptop environment detected"
    fi
}

# Display installation menu
show_menu() {
    header "MultiOS Installation Menu"
    echo
    echo "Please select installation type:"
    echo
    echo "1) Desktop/Laptop (Default)"
    echo "   - GUI support enabled"
    echo "   - Full desktop environment"
    echo "   - User-friendly interface"
    echo
    echo "2) Server"
    echo "   - Optimized for server workloads"
    echo "   - Enhanced security"
    echo "   - Monitoring and backup tools"
    echo
    echo "3) Embedded/IoT"
    echo "   - Minimal footprint"
    echo "   - IoT sensor support"
    echo "   - Edge computing features"
    echo
    echo "4) Development Environment"
    echo "   - Full development tools"
    echo "   - IDE integration"
    echo "   - Build and test automation"
    echo
    echo "5) Interactive Detection (Auto-detect and ask)"
    echo "6) Cancel Installation"
    echo
    
    if [ -n "$1" ]; then
        echo -e "${YELLOW}Auto-selected: $1${NC}"
        echo
        echo "Press Enter to continue with this selection, or type a number (1-6) to change:"
        read -r choice
        
        if [ -z "$choice" ]; then
            choice=$1
        fi
    else
        echo -n "Enter your choice (1-6): "
        read -r choice
    fi
    
    case $choice in
        1) SELECTED_TYPE="desktop" ;;
        2) SELECTED_TYPE="server" ;;
        3) SELECTED_TYPE="embedded" ;;
        4) SELECTED_TYPE="development" ;;
        5) SELECTED_TYPE="interactive" ;;
        6) 
            echo "Installation cancelled"
            exit 0
            ;;
        *) 
            warning "Invalid choice, using auto-detected type: $SYSTEM_TYPE"
            SELECTED_TYPE="$SYSTEM_TYPE"
            ;;
    esac
}

# Verify installation files
verify_installation() {
    header "Verifying Installation Files..."
    
    local errors=0
    
    # Check if installation scripts exist
    local scripts=(
        "installation/desktop/install_multios_desktop.sh"
        "installation/server/install_multios_server.sh"
        "installation/embedded/install_multios_embedded.sh"
        "installation/development/install_multios_dev.sh"
    )
    
    for script in "${scripts[@]}"; do
        if [ -f "$script" ]; then
            info "✓ $script exists"
        else
            error "✗ $script not found"
            errors=$((errors + 1))
        fi
    done
    
    # Check core components
    local components=(
        "kernel"
        "bootloader"
        "libraries"
        "documentation"
    )
    
    for component in "${components[@]}"; do
        if [ -d "$component" ]; then
            info "✓ $component directory exists"
        else
            error "✗ $component directory not found"
            errors=$((errors + 1))
        fi
    done
    
    # Check checksums if available
    if [ -f "checksums.sha256" ]; then
        info "Verifying checksums..."
        if sha256sum -c checksums.sha256 >/dev/null 2>&1; then
            info "✓ Checksums verified"
        else
            warning "⚠ Checksum verification failed (may be normal during development)"
        fi
    else
        warning "⚠ No checksums file found"
    fi
    
    if [ $errors -gt 0 ]; then
        error "Installation verification failed with $errors errors"
        error "Please ensure all files are present and try again"
        exit 1
    fi
    
    info "Installation verification completed successfully"
}

# Run selected installation
run_installation() {
    local install_script=""
    
    case $SELECTED_TYPE in
        "desktop")
            install_script="installation/desktop/install_multios_desktop.sh"
            header "Starting Desktop Installation..."
            ;;
        "server")
            install_script="installation/server/install_multios_server.sh"
            header "Starting Server Installation..."
            ;;
        "embedded")
            install_script="installation/embedded/install_multios_embedded.sh"
            header "Starting Embedded/IoT Installation..."
            ;;
        "development")
            install_script="installation/development/install_multios_dev.sh"
            header "Starting Development Environment Installation..."
            ;;
        "interactive")
            # Re-detect and ask
            detect_system_type
            show_menu
            run_installation
            return
            ;;
    esac
    
    if [ ! -f "$install_script" ]; then
        error "Installation script not found: $install_script"
        exit 1
    fi
    
    # Make script executable
    chmod +x "$install_script"
    
    # Run installation
    info "Running installation script: $install_script"
    echo
    bash "$install_script"
}

# Show system information
show_system_info() {
    header "System Information"
    echo
    echo "Architecture: $(uname -m)"
    echo "Kernel: $(uname -r)"
    echo "OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2 2>/dev/null || echo "Unknown")"
    echo "Memory: $(free -h | grep Mem | awk '{print $2}')"
    echo "Storage: $(df -h ~ | tail -1 | awk '{print $4}') available"
    echo "CPU Cores: $(nproc)"
    echo
    
    # Check for virtualization
    if [ -f /sys/class/dmi/id/product_name ]; then
        echo "Hardware: $(cat /sys/class/dmi/id/product_name 2>/dev/null)"
    fi
    
    if command -v systemctl >/dev/null 2>&1; then
        echo "Init System: systemd"
    else
        echo "Init System: Unknown"
    fi
    
    echo
}

# Create installation report
create_report() {
    local report_file="/tmp/multios_install_report_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "MultiOS Installation Report"
        echo "==========================="
        echo
        echo "Date: $(date)"
        echo "User: $(whoami)"
        echo "System: $(uname -a)"
        echo
        echo "Installation Type: $SELECTED_TYPE"
        echo "Detected System Type: $SYSTEM_TYPE"
        echo
        echo "System Information:"
        echo "  Architecture: $(uname -m)"
        echo "  Memory: $(free -h | grep Mem | awk '{print $2}')"
        echo "  Storage: $(df -h ~ | tail -1 | awk '{print $4}') available"
        echo "  CPU Cores: $(nproc)"
        echo
        echo "Installation completed successfully"
        
    } > "$report_file"
    
    info "Installation report saved to: $report_file"
}

# Main function
main() {
    header "========================================"
    header "    MultiOS Installation Wizard"
    header "    Version: 1.0.0"
    header "========================================"
    echo
    
    # Check if running from correct directory
    if [ ! -d "kernel" ] || [ ! -d "installation" ]; then
        error "This script must be run from the MultiOS distribution directory"
        error "Please extract the distribution and run this script from there"
        exit 1
    fi
    
    # Show system information
    show_system_info
    
    # Detect system type
    detect_system_type
    
    # Show menu or auto-select
    if [ "$1" == "--auto" ] || [ "$1" == "-y" ]; then
        SELECTED_TYPE="$SYSTEM_TYPE"
        info "Auto-selected installation type: $SELECTED_TYPE"
    else
        show_menu "$SYSTEM_TYPE"
    fi
    
    echo
    header "Starting MultiOS Installation"
    echo "Installation Type: $SELECTED_TYPE"
    echo
    
    # Verify installation
    verify_installation
    
    # Run installation
    run_installation
    
    # Create report
    create_report
    
    header "Installation Complete!"
    echo
    echo "MultiOS has been installed successfully on your system."
    echo
    echo "Next steps:"
    echo "  1. Read the installation summary above"
    echo "  2. Reboot your system if required"
    echo "  3. Check the documentation for next steps"
    echo
    echo "For support, visit: https://github.com/multios/multios"
    echo
}

# Check for help flag
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "MultiOS Master Installation Script"
    echo
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --auto, -y    Auto-select detected system type"
    echo "  --help, -h    Show this help message"
    echo
    echo "This script will detect your system type and guide you through"
    echo "the appropriate MultiOS installation process."
    exit 0
fi

# Run main function
main "$@"