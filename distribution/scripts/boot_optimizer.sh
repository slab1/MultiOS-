#!/bin/bash

# Boot Optimization Script
# Applies various optimizations to achieve sub-2-second boot times

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_DIR="/etc/boot_optimizer_backup"
LOG_FILE="/var/log/boot_optimization.log"
DRY_RUN=false
FORCE=false

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
    echo -e "${BLUE}  Boot Optimization Tool${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo
}

print_section() {
    echo -e "\n${YELLOW}>>> $1${NC}"
}

backup_file() {
    local file=$1
    if [[ -f "$file" ]] || [[ -d "$file" ]]; then
        local backup_path="$BACKUP_DIR/$(basename $file)_$(date +%Y%m%d_%H%M%S)"
        mkdir -p "$BACKUP_DIR"
        cp -r "$file" "$backup_path"
        echo "Backed up: $file -> $backup_path"
    fi
}

check_privileges() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}Error: This script must be run as root${NC}"
        exit 1
    fi
}

confirm_action() {
    if [[ "$FORCE" == "true" ]]; then
        return 0
    fi
    
    echo -n "Apply this optimization? (y/N): "
    read -r response
    case "$response" in
        [yY][eE][sS]|[yY]) 
            return 0
            ;;
        *)
            echo "Skipped"
            return 1
            ;;
    esac
}

optimize_grub() {
    print_section "Optimizing GRUB Configuration"
    
    local grub_config="/etc/default/grub"
    
    if [[ ! -f "$grub_config" ]]; then
        echo "GRUB configuration not found, skipping"
        return
    fi
    
    backup_file "$grub_config"
    
    # Set fast boot options
    local current_timeout=$(grep "^GRUB_TIMEOUT=" "$grub_config" | cut -d'=' -f2 || echo "5")
    local current_cmdline=$(grep "^GRUB_CMDLINE_LINUX_DEFAULT=" "$grub_config" | cut -d'"' -f2 || echo "")
    
    echo "Current GRUB timeout: ${current_timeout}s"
    echo "Current cmdline: $current_cmdline"
    
    # Optimize timeout
    if [[ "$current_timeout" != "0" ]]; then
        if confirm_action; then
            sed -i 's/^GRUB_TIMEOUT=.*/GRUB_TIMEOUT=0/' "$grub_config"
            echo -e "${GREEN}✓ Set GRUB timeout to 0${NC}"
        fi
    fi
    
    # Add fast boot parameters
    local desired_params="quiet splash noresume loglevel=0"
    if [[ ! "$current_cmdline" =~ "quiet" ]]; then
        if confirm_action; then
            # Add parameters safely
            local new_cmdline=$(echo "$current_cmdline $desired_params" | xargs)
            sed -i "s|^GRUB_CMDLINE_LINUX_DEFAULT=.*|GRUB_CMDLINE_LINUX_DEFAULT=\"$new_cmdline\"|" "$grub_config"
            echo -e "${GREEN}✓ Added fast boot parameters${NC}"
        fi
    fi
    
    # Update GRUB
    if [[ -d "/sys/firmware/efi" ]]; then
        echo "Updating GRUB for EFI system..."
        confirm_action && update-grub2 || echo "GRUB update skipped"
    else
        echo "Updating GRUB for BIOS system..."
        confirm_action && update-grub || echo "GRUB update skipped"
    fi
}

optimize_kernel_parameters() {
    print_section "Optimizing Kernel Parameters"
    
    local kernel_cmdline="/proc/cmdline"
    local current_cmdline=$(cat "$kernel_cmdline")
    
    echo "Current kernel parameters: $current_cmdline"
    
    # Parameters to add/optimize
    local optimizations=(
        "quiet:Suppress verbose messages"
        "splash:Enable splash screen"
        "noresume:Skip resume from hibernation"
        "systemd.show_status=false:Disable status messages"
        "vt.global_cursor_default=0:Hide cursor during boot"
    )
    
    for opt in "${optimizations[@]}"; do
        local param="${opt%%:*}"
        local desc="${opt##*:}"
        
        if ! echo "$current_cmdline" | grep -q "$param"; then
            echo "Missing: $param ($desc)"
            if confirm_action; then
                echo "To add $param, update GRUB configuration and run update-grub"
                echo "Add to GRUB_CMDLINE_LINUX_DEFAULT in /etc/default/grub"
            fi
        else
            echo -e "${GREEN}✓ Found: $param${NC}"
        fi
    done
}

optimize_systemd_services() {
    print_section "Optimizing Systemd Services"
    
    # Disable unnecessary services for faster boot
    local services_to_disable=(
        " cups.service:Printing service"
        " bluetooth.service:Bluetooth (if not needed)"
        " whoopsie.service:Ubuntu crash reporting"
        " apparmor.service:AppArmor (if not required)"
        " speech-dispatcher.service:Speech synthesis"
    )
    
    echo "Checking services that could be disabled:"
    
    for service_info in "${services_to_disable[@]}"; do
        local service="${service_info%%:*}"
        local desc="${service_info##*:}"
        
        if systemctl is-enabled "$service" &> /dev/null; then
            echo "Service: $service - $desc"
            if confirm_action; then
                systemctl disable "$service"
                echo -e "${GREEN}✓ Disabled $service${NC}"
            fi
        else
            echo -e "${GREEN}✓ $service already disabled${NC}"
        fi
    done
    
    # Check for failed services
    local failed_services=$(systemctl list-units --type=service --state=failed | grep -c "failed" || echo "0")
    if [[ $failed_services -gt 0 ]]; then
        echo -e "${YELLOW}⚠️  Found $failed_services failed services${NC}"
        echo "Failed services:"
        systemctl list-units --type=service --state=failed --no-pager | grep "failed" | awk '{print "  " $1}'
    fi
}

optimize_initramfs() {
    print_section "Optimizing Initramfs"
    
    local initramfs_config="/etc/initramfs-tools/initramfs.conf"
    
    if [[ ! -f "$initramfs_config" ]]; then
        echo "Initramfs configuration not found, skipping"
        return
    fi
    
    backup_file "$initramfs_config"
    
    # Check current compression
    local current_compression=$(grep "^COMPRESS=" "$initramfs_config" | cut -d'=' -f2 || echo "gzip")
    echo "Current compression: $current_compression"
    
    # Recommend compression optimization
    if [[ "$current_compression" == "gzip" ]]; then
        echo -e "${YELLOW}Consider using lz4 compression for faster decompression${NC}"
        echo "To change: edit /etc/initramfs-tools/initramfs.conf"
        echo "Set: COMPRESS=lz4"
        echo "Then run: update-initramfs -c -k all"
    fi
}

optimize_modules() {
    print_section "Optimizing Kernel Modules"
    
    local modules_file="/etc/modules-load.d/boot-optimizer.conf"
    
    # Create optimized modules list
    cat > "$modules_file" << 'EOF'
# Boot optimizer modules
# Load essential modules early for faster boot

# Storage modules
# Uncomment as needed for your hardware
# ahci
# nvme
# ext4

# Network modules
# e1000e
# r8169

# Add your hardware-specific modules here
# Check with: lspci -k
EOF
    
    echo -e "${GREEN}✓ Created optimized modules list at $modules_file${NC}"
    echo "Edit this file to add/remove modules based on your hardware"
    
    # Blacklist unnecessary modules
    local blacklist_dir="/etc/modprobe.d"
    local blacklist_file="$blacklist_dir/boot-optimizer-blacklist.conf"
    
    cat > "$blacklist_file" << 'EOF'
# Blacklist unnecessary modules for faster boot

# Uncomment modules you don't need
# blacklist usbhid
# blacklist btusb
# blacklist snd_hda_intel
# blacklist nouveau

# Add modules to blacklist based on your hardware
EOF
    
    echo -e "${GREEN}✓ Created blacklist file at $blacklist_file${NC}"
}

optimize_cpu_governor() {
    print_section "Optimizing CPU Governor"
    
    # Set CPU governor to performance during boot
    local governor_script="/etc/init.d/set-performance-governor"
    
    cat > "$governor_script" << 'EOF'
#!/bin/bash
### BEGIN INIT INFO
# Provides:          set-performance-governor
# Required-Start:    $local_fs
# Required-Stop:
# Default-Start:     S
# Default-Stop:
# Short-Description: Set CPU governor to performance
# Description:       Set CPU governor to performance for faster boot
### END INIT INFO

case "$1" in
    start)
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            if [ -f "$cpu" ]; then
                echo performance > "$cpu"
            fi
        done
        ;;
    stop)
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            if [ -f "$cpu" ]; then
                echo ondemand > "$cpu" 2>/dev/null || echo powersave > "$cpu" 2>/dev/null
            fi
        done
        ;;
    *)
        echo "Usage: $0 {start|stop}"
        exit 1
        ;;
esac

exit 0
EOF
    
    chmod +x "$governor_script"
    echo -e "${GREEN}✓ Created CPU governor optimization script${NC}"
}

optimize_network() {
    print_section "Optimizing Network Configuration"
    
    # Disable network wait if not needed
    local networkd_config="/etc/systemd/system/network-online.target.wants/systemd-networkd-wait-online.service"
    
    if [[ -L "$networkd_config" ]]; then
        echo "Found network wait service"
        echo -e "${YELLOW}Consider disabling if network is not required during boot${NC}"
        echo "To disable: systemctl disable systemd-networkd-wait-online.service"
    fi
    
    # Optimize DNS resolution
    local resolv_conf="/etc/resolv.conf"
    if [[ -f "$resolv_conf" ]]; then
        echo "DNS configuration:"
        grep -E "nameserver|options" "$resolv_conf" | head -5
    fi
}

optimize_filesystem() {
    print_section "Optimizing Filesystem Configuration"
    
    # Check fstab for optimization opportunities
    local fstab="/etc/fstab"
    
    if [[ -f "$fstab" ]]; then
        echo "Analyzing /etc/fstab for optimization opportunities..."
        
        # Look for ext4 with optimization options
        if grep -q "ext4.*noatime" "$fstab"; then
            echo -e "${GREEN}✓ Found noatime optimization${NC}"
        else
            echo -e "${YELLOW}? Consider adding noatime to ext4 mounts${NC}"
        fi
        
        if grep -q "ext4.*commit=" "$fstab"; then
            echo -e "${GREEN}✓ Found commit optimization${NC}"
        else
            echo -e "${YELLOW}? Consider adding commit=1 to ext4 mounts for faster sync${NC}"
        fi
    fi
}

measure_current_boot_time() {
    print_section "Measuring Current Boot Time"
    
    if command -v systemd-analyze &> /dev/null; then
        echo "Current boot performance:"
        systemd-analyze
        echo
        echo "Slowest services:"
        systemd-analyze blame | head -5
    else
        echo "systemd-analyze not available"
    fi
}

apply_all_optimizations() {
    print_section "Applying All Optimizations"
    
    echo -e "${YELLOW}This will apply the following optimizations:${NC}"
    echo "1. Optimize GRUB configuration (set timeout=0, add fast boot parameters)"
    echo "2. Add kernel boot parameters for faster boot"
    echo "3. Disable unnecessary systemd services"
    echo "4. Optimize initramfs compression"
    echo "5. Configure kernel modules for faster loading"
    echo "6. Set CPU governor to performance during boot"
    echo "7. Optimize network configuration"
    echo "8. Analyze filesystem configuration"
    echo
    echo -e "${RED}Note: Some changes require a reboot to take effect${NC}"
    
    if ! confirm_action; then
        return
    fi
    
    optimize_grub
    optimize_kernel_parameters
    optimize_systemd_services
    optimize_initramfs
    optimize_modules
    optimize_cpu_governor
    optimize_network
    optimize_filesystem
}

rollback_optimizations() {
    print_section "Rollback Optimizations"
    
    if [[ ! -d "$BACKUP_DIR" ]]; then
        echo "No backup directory found"
        return
    fi
    
    echo "Available backups:"
    ls -la "$BACKUP_DIR"
    echo
    
    echo -n "Restore from backup? (y/N): "
    read -r response
    case "$response" in
        [yY][eE][sS]|[yY])
            # Restore GRUB backup
            local grub_backup=$(ls -t "$BACKUP_DIR"/grub_* 2>/dev/null | head -1)
            if [[ -n "$grub_backup" ]]; then
                cp "$grub_backup" /etc/default/grub
                echo "Restored GRUB configuration from: $grub_backup"
                
                # Update GRUB
                if [[ -d "/sys/firmware/efi" ]]; then
                    update-grub2
                else
                    update-grub
                fi
            fi
            
            echo "Rollback completed. Reboot required for changes to take effect."
            ;;
        *)
            echo "Rollback cancelled"
            ;;
    esac
}

show_status() {
    print_section "Optimization Status"
    
    echo "Current boot time:"
    measure_current_boot_time
    
    echo
    echo "Recent changes:"
    if [[ -d "$BACKUP_DIR" ]]; then
        ls -lt "$BACKUP_DIR" | head -10
    else
        echo "No backups found"
    fi
    
    echo
    echo "Optimization status:"
    [[ -f "/etc/default/grub" ]] && grep -q "GRUB_TIMEOUT=0" "/etc/default/grub" && echo "✓ GRUB timeout optimized" || echo "✗ GRUB timeout not optimized"
    
    if systemctl is-disabled cups.service &> /dev/null; then
        echo "✓ Unnecessary services disabled"
    else
        echo "? Some services may still be enabled"
    fi
}

main() {
    print_header
    
    # Check if running as root
    check_privileges
    
    # Create log directory
    mkdir -p "$(dirname "$LOG_FILE")"
    
    log "Starting boot optimization"
    
    # Show current status
    show_status
    
    # Apply optimizations
    apply_all_optimizations
    
    echo
    echo -e "${GREEN}Boot optimization complete!${NC}"
    echo -e "${YELLOW}Please reboot your system for changes to take effect.${NC}"
    echo -e "${BLUE}Run 'systemd-analyze' after reboot to check boot time.${NC}"
    
    log "Boot optimization completed"
}

# Parse command line arguments
case "${1:-}" in
    --help|-h)
        echo "Boot Optimization Tool"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h        Show this help message"
        echo "  --dry-run         Show what would be done without making changes"
        echo "  --force           Apply optimizations without confirmation"
        echo "  --rollback        Rollback previous optimizations"
        echo "  --status          Show current optimization status"
        echo "  --grub            Optimize GRUB only"
        echo "  --services        Optimize systemd services only"
        echo "  --modules         Optimize kernel modules only"
        echo
        echo "This tool applies various optimizations to achieve faster boot times."
        echo "Always create backups before applying optimizations."
        exit 0
        ;;
    --dry-run)
        DRY_RUN=true
        echo "Dry run mode - no changes will be made"
        ;;
    --force)
        FORCE=true
        echo "Force mode - optimizations will be applied without confirmation"
        ;;
    --rollback)
        rollback_optimizations
        exit 0
        ;;
    --status)
        show_status
        exit 0
        ;;
    --grub)
        optimize_grub
        exit 0
        ;;
    --services)
        optimize_systemd_services
        exit 0
        ;;
    --modules)
        optimize_modules
        exit 0
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
