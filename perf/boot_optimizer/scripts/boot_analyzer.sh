#!/bin/bash

# Boot Performance Analyzer Script
# Analyzes system boot performance and provides optimization recommendations

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_DIR="/tmp/boot_reports"
LOG_FILE="/var/log/boot_analysis.log"
BOOT_TIME_TARGET=2000  # Target: 2 seconds

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Boot Performance Analysis Tool${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo
}

print_section() {
    echo -e "\n${YELLOW}>>> $1${NC}"
}

check_privileges() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}Error: This script must be run as root${NC}"
        exit 1
    fi
}

analyze_system_boot_time() {
    print_section "System Boot Time Analysis"
    
    # Get boot time from systemd-analyze
    if command -v systemd-analyze &> /dev/null; then
        local total_time=$(systemd-analyze | grep -oP 'boot time: \K[0-9.]+' || echo "N/A")
        local firmware=$(systemd-analyze blame | grep firmware | head -1 | awk '{print $1}' || echo "0")
        local bootloader=$(systemd-analyze blame | grep bootloader | head -1 | awk '{print $1}' || echo "0")
        local kernel=$(systemd-analyze blame | grep kernel | head -1 | awk '{print $1}' || echo "0")
        
        echo "Total boot time: ${total_time}ms"
        echo "Firmware time: ${firmware}ms"
        echo "Bootloader time: ${bootloader}ms"
        echo "Kernel time: ${kernel}ms"
        
        # Check if target is met
        local total_ms=$(echo "$total_time" | tr -d 'ms' 2>/dev/null || echo "0")
        if [[ $total_ms -gt $BOOT_TIME_TARGET ]]; then
            echo -e "${RED}⚠️  Boot time exceeds target of ${BOOT_TIME_TARGET}ms${NC}"
            return 1
        else
            echo -e "${GREEN}✓ Boot time meets target of ${BOOT_TIME_TARGET}ms${NC}"
            return 0
        fi
    else
        echo "systemd-analyze not available, skipping detailed analysis"
        return 1
    fi
}

analyze_boot_services() {
    print_section "Boot Services Analysis"
    
    if command -v systemd-analyze &> /dev/null; then
        echo "Top 10 slowest boot services:"
        systemd-analyze blame | head -10 | while read line; do
            local time=$(echo "$line" | awk '{print $1}')
            local service=$(echo "$line" | awk '{for(i=2;i<=NF;i++) printf "%s ", $i; print ""}')
            local time_ms=$(echo "$time" | tr -d 's')
            
            if (( $(echo "$time_ms > 500" | bc -l) )); then
                echo -e "${RED}  $time  $service${NC}"
            else
                echo "  $time  $service"
            fi
        done
    fi
}

check_kernel_parameters() {
    print_section "Kernel Parameters Analysis"
    
    local cmdline=$(cat /proc/cmdline)
    echo "Current kernel parameters: $cmdline"
    
    # Check for common boot optimization parameters
    local optimizations=(
        "quiet:Suppress verbose boot messages"
        "systemd.show_status=false:Disable status messages"
        "initcall_debug:Debug init calls (disable for production)"
        "noresume:Skip resume from hibernation"
        "nosmp:Disable SMP (not recommended for modern systems)"
        "maxcpus=1:Limit to single CPU (not recommended)"
    )
    
    for opt in "${optimizations[@]}"; do
        local param="${opt%%:*}"
        local desc="${opt##*:}"
        
        if echo "$cmdline" | grep -q "$param"; then
            echo -e "${GREEN}✓ Found: $param ($desc)${NC}"
        else
            echo -e "${YELLOW}? Not found: $param ($desc)${NC}"
        fi
    done
}

analyze_grub_config() {
    print_section "GRUB Configuration Analysis"
    
    local grub_config="/etc/default/grub"
    
    if [[ -f "$grub_config" ]]; then
        echo "GRUB configuration found at: $grub_config"
        
        # Check for fast boot options
        local grub_cmdline=$(grep "^GRUB_CMDLINE_LINUX_DEFAULT" "$grub_config" | cut -d'"' -f2)
        echo "Current GRUB cmdline: $grub_cmdline"
        
        # Check for timeout
        local timeout=$(grep "^GRUB_TIMEOUT" "$grub_config" | cut -d'=' -f2)
        echo "GRUB timeout: ${timeout}s"
        
        if [[ $timeout -gt 0 ]]; then
            echo -e "${YELLOW}⚠️  Consider setting GRUB_TIMEOUT=0 for faster boot${NC}"
        fi
    else
        echo "GRUB configuration not found at expected location"
    fi
}

check_bootloader_efficiency() {
    print_section "Bootloader Performance Check"
    
    # Check if using systemd-boot
    if [[ -d "/sys/firmware/efi" ]] && command -v bootctl &> /dev/null; then
        echo "Using systemd-boot (EFI)"
        
        # Analyze boot loader entries
        if command -v bootctl &> /dev/null; then
            echo "Boot loader entries:"
            bootctl status 2>/dev/null | grep -E "title|id" | head -10
        fi
    fi
    
    # Check for initramfs optimization
    if [[ -d "/etc/initramfs-tools" ]]; then
        echo "Initramfs configuration found"
        
        # Check for compression type
        local compression=$(update-initramfs -c -k all 2>&1 | grep -o "compress=[a-z]*" | cut -d'=' -f2 || echo "unknown")
        echo "Compression type: $compression"
    fi
}

analyze_device_initialization() {
    print_section "Device Initialization Analysis"
    
    # Check for parallel device initialization
    local dmesg_file="/var/log/dmesg"
    if [[ -f "$dmesg_file" ]]; then
        echo "Analyzing device initialization from dmesg..."
        
        # Look for USB initialization times
        local usb_init_time=$(grep "usb.*1-.*:.*new.*USB" "$dmesg_file" | head -1 | awk '{print $1}' | tr -d '[]')
        if [[ -n "$usb_init_time" ]]; then
            echo "USB device initialization started: $usb_init_time"
        fi
        
        # Check for PCI device enumeration
        local pci_init_time=$(grep "PCI.*enumeration" "$dmesg_file" | head -1 | awk '{print $1}' | tr -d '[]')
        if [[ -n "$pci_init_time" ]]; then
            echo "PCI enumeration: $pci_init_time"
        fi
    else
        echo "dmesg log not available"
    fi
    
    # Check for modular kernel configuration
    if [[ -f "/proc/modules" ]]; then
        local loaded_modules=$(wc -l < /proc/modules)
        echo "Loaded kernel modules: $loaded_modules"
        
        if [[ $loaded_modules -gt 100 ]]; then
            echo -e "${YELLOW}⚠️  Many modules loaded - consider optimizing${NC}"
        fi
    fi
}

generate_optimization_recommendations() {
    print_section "Optimization Recommendations"
    
    local recommendations=()
    
    # Check boot time
    local total_time=$(systemd-analyze 2>/dev/null | grep -oP 'boot time: \K[0-9.]+' || echo "0")
    local total_ms=$(echo "$total_time" | tr -d 'ms' 2>/dev/null || echo "0")
    
    if (( $(echo "$total_ms > $BOOT_TIME_TARGET" | bc -l) )); then
        recommendations+=("Boot time exceeds target - consider parallel initialization")
    fi
    
    # Check GRUB timeout
    if [[ -f "/etc/default/grub" ]]; then
        local timeout=$(grep "^GRUB_TIMEOUT" "/etc/default/grub" | cut -d'=' -f2 || echo "5")
        if [[ $timeout -gt 0 ]]; then
            recommendations+=("Set GRUB_TIMEOUT=0 to skip bootloader menu")
        fi
    fi
    
    # Check for unnecessary services
    local slow_services=$(systemd-analyze blame 2>/dev/null | awk '$1 ~ /^[0-9]+\.[0-9]+s$/ && $1 > 0.5 {print $2}' | wc -l)
    if [[ $slow_services -gt 3 ]]; then
        recommendations+=("Found $slow_services slow services - review and optimize")
    fi
    
    # Output recommendations
    if [[ ${#recommendations[@]} -eq 0 ]]; then
        echo -e "${GREEN}✓ No critical optimization recommendations found${NC}"
    else
        echo "Optimization opportunities:"
        for rec in "${recommendations[@]}"; do
            echo -e "${YELLOW}• $rec${NC}"
        done
    fi
}

save_report() {
    local report_file="$REPORT_DIR/boot_analysis_$(date +%Y%m%d_%H%M%S).txt"
    
    print_section "Saving Report"
    
    # Create report directory
    mkdir -p "$REPORT_DIR"
    
    # Save detailed report
    {
        echo "Boot Performance Analysis Report"
        echo "Generated: $(date)"
        echo "System: $(uname -a)"
        echo
        echo "=== SYSTEM INFORMATION ==="
        uname -a
        echo
        echo "=== BOOT TIME ANALYSIS ==="
        systemd-analyze 2>/dev/null || echo "systemd-analyze not available"
        echo
        echo "=== BOOT SERVICE ANALYSIS ==="
        systemd-analyze blame 2>/dev/null || echo "systemd-analyze not available"
        echo
        echo "=== KERNEL PARAMETERS ==="
        cat /proc/cmdline
        echo
        echo "=== LOADED MODULES ==="
        lsmod | head -20
    } > "$report_file"
    
    echo "Report saved to: $report_file"
    
    # Also save JSON report if possible
    local json_file="$REPORT_DIR/boot_analysis_$(date +%Y%m%d_%H%M%S).json"
    {
        echo "{"
        echo "  \"timestamp\": \"$(date -Iseconds)\","
        echo "  \"system\": \"$(uname -a | sed 's/\"/\\\"/g')\","
        echo "  \"boot_time\": \"$(systemd-analyze 2>/dev/null | grep -oP 'boot time: \K[0-9.]+' || echo 'N/A')\","
        echo "  \"kernel_cmdline\": \"$(cat /proc/cmdline | sed 's/\"/\\\"/g')\","
        echo "  \"loaded_modules\": $(lsmod | wc -l)"
        echo "}"
    } > "$json_file"
    
    echo "JSON report saved to: $json_file"
}

main() {
    print_header
    
    # Check if running as root
    check_privileges
    
    # Create log directory
    mkdir -p "$(dirname "$LOG_FILE")"
    
    log "Starting boot performance analysis"
    
    # Run all analysis functions
    analyze_system_boot_time
    analyze_boot_services
    check_kernel_parameters
    analyze_grub_config
    check_bootloader_efficiency
    analyze_device_initialization
    generate_optimization_recommendations
    save_report
    
    echo
    echo -e "${GREEN}Boot analysis complete!${NC}"
    log "Boot performance analysis completed"
}

# Parse command line arguments
case "${1:-}" in
    --help|-h)
        echo "Boot Performance Analysis Tool"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --quick        Run quick analysis only"
        echo "  --detailed     Run detailed analysis (default)"
        echo
        echo "This tool analyzes system boot performance and provides optimization recommendations."
        exit 0
        ;;
    --quick)
        # Run only essential checks
        analyze_system_boot_time
        generate_optimization_recommendations
        ;;
    --detailed|"")
        main
        ;;
    *)
        echo "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac
