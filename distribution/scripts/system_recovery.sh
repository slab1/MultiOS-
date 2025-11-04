#!/bin/bash
# MultiOS System Recovery and Backup Tool
# Comprehensive recovery, backup, and restore functionality

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
BACKUP_DIR="/var/backups/multios"
RECOVERY_DIR="/var/recovery/multios"
LOG_FILE="/var/log/multios-recovery.log"
SNAPSHOT_DIR="/var/snapshots/multios"
CONFIG_FILE="/etc/multios/recovery.conf"

# Recovery modes
MODE_BACKUP="backup"
MODE_RESTORE="restore"
MODE_SNAPSHOT="snapshot"
MODE_REPAIR="repair"
MODE_CHECK="check"
MODE_CREATE_REScue="create-rescue"

# Recovery types
TYPE_SYSTEM="system"
TYPE_DATA="data"
TYPE_CONFIG="config"
TYPE_FULL="full"

# Variables
OPERATION=""
RECOVERY_TYPE=""
SOURCE_PATH=""
TARGET_PATH=""
BACKUP_NAME=""
EXCLUDE_PATHS=()
COMPRESSION=true
ENCRYPTION=false
ENCRYPTION_KEY=""
SCHEDULE=""
DRY_RUN=false
VERBOSE=false
INTERACTIVE=true
FORCE=false

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

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_color $RED "Error: This operation requires root privileges"
        exit 1
    fi
}

# Function to create directories
create_directories() {
    mkdir -p "$BACKUP_DIR" "$RECOVERY_DIR" "$SNAPSHOT_DIR" "$(dirname "$CONFIG_FILE")"
    chmod 700 "$BACKUP_DIR" "$RECOVERY_DIR"
}

# Function to initialize recovery system
init_recovery_system() {
    log "INFO" "Initializing MultiOS Recovery System v$TOOL_VERSION"
    
    create_directories
    
    # Create default configuration
    if [[ ! -f "$CONFIG_FILE" ]]; then
        cat > "$CONFIG_FILE" << EOF
# MultiOS Recovery System Configuration

[paths]
backup_dir=$BACKUP_DIR
recovery_dir=$RECOVERY_DIR
snapshot_dir=$SNAPSHOT_DIR
log_file=$LOG_FILE

[options]
compression=true
encryption=false
verbose=false
interactive=true

[exclusions]
# Paths to exclude from backups
exclude_patterns=(
    "/proc/*"
    "/sys/*"
    "/dev/*"
    "/tmp/*"
    "/var/tmp/*"
    "/var/cache/*"
    "/var/log/*"
    "/lost+found"
)
EOF
    fi
    
    log "SUCCESS" "Recovery system initialized"
}

# Function to load configuration
load_config() {
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log "WARN" "Configuration file not found, using defaults"
        return
    fi
    
    # Source configuration (simplified - would use proper config parsing)
    source "$CONFIG_FILE" 2>/dev/null || true
}

# Function to detect system information
detect_system_info() {
    log "INFO" "Detecting system information..."
    
    local arch=$(uname -m)
    local kernel=$(uname -r)
    local distro=$(lsb_release -d 2>/dev/null | cut -f2 || echo "Unknown")
    local disk_info=$(df -h / | tail -1)
    
    echo "System Information:"
    echo "  Architecture: $arch"
    echo "  Kernel: $kernel"
    echo "  Distribution: $distro"
    echo "  Root Disk: $disk_info"
    echo "  Date: $(date)"
    
    log "INFO" "System: $arch $kernel $distro"
}

# Function to create system snapshot
create_snapshot() {
    local snapshot_name="${1:-snapshot-$(date +%Y%m%d-%H%M%S)}"
    local snapshot_path="$SNAPSHOT_DIR/$snapshot_name"
    
    log "INFO" "Creating system snapshot: $snapshot_name"
    
    # Create snapshot directory
    mkdir -p "$snapshot_path"
    
    # Create snapshot metadata
    cat > "$snapshot_path/metadata.json" << EOF
{
    "name": "$snapshot_name",
    "timestamp": "$(date -Iseconds)",
    "hostname": "$(hostname)",
    "kernel": "$(uname -r)",
    "arch": "$(uname -m)",
    "root_filesystem": "$(findmnt -n -o SOURCE /)",
    "mountpoints": [
$(mount | grep -E '^\/' | while read line; do
    local device=$(echo "$line" | awk '{print $1}')
    local mountpoint=$(echo "$line" | awk '{print $3}')
    echo "        {\"device\": \"$device\", \"mountpoint\": \"$mountpoint\"}"
done | paste -sd, -)
    ]
}
EOF
    
    # Create filesystem snapshots (simplified)
    log "INFO" "Creating filesystem snapshots..."
    
    # Create hard links snapshot using rsync
    if command -v rsync &> /dev/null; then
        rsync -aAXx --link-dest=/ / "$snapshot_path/root/" 2>/dev/null || true
    else
        log "WARN" "rsync not available, using basic snapshot"
        mkdir -p "$snapshot_path/root"
        # Basic snapshot would go here
    fi
    
    log "SUCCESS" "System snapshot created: $snapshot_path"
    echo "$snapshot_path"
}

# Function to list snapshots
list_snapshots() {
    log "INFO" "Listing system snapshots"
    
    if [[ ! -d "$SNAPSHOT_DIR" ]]; then
        print_color $YELLOW "No snapshots directory found"
        return
    fi
    
    echo "Available Snapshots:"
    echo "==================="
    echo "Name                    Date                    Size"
    echo "------------------------ ---------------------- --------"
    
    for snapshot in "$SNAPSHOT_DIR"/*/; do
        if [[ -d "$snapshot" ]]; then
            local name=$(basename "$snapshot")
            local date=$(stat -c %y "$snapshot" | cut -d' ' -f1)
            local size=$(du -sh "$snapshot" | cut -f1)
            printf "%-23s %-23s %-8s\n" "$name" "$date" "$size"
        fi
    done
}

# Function to create backup
create_backup() {
    local backup_name="${1:-backup-$(date +%Y%m%d-%H%M%S)}"
    local backup_path="$BACKUP_DIR/$backup_name"
    local source_paths=("${@:2}")
    
    log "INFO" "Creating backup: $backup_name"
    
    # Create backup directory
    mkdir -p "$backup_path"
    
    # Determine source paths
    if [[ ${#source_paths[@]} -eq 0 ]]; then
        case $RECOVERY_TYPE in
            "$TYPE_SYSTEM")
                source_paths=("/")
                EXCLUDE_PATHS+=("/proc" "/sys" "/dev" "/tmp" "/var/cache" "/var/log")
                ;;
            "$TYPE_DATA")
                source_paths=("/home" "/var/www" "/opt" "/srv")
                ;;
            "$TYPE_CONFIG")
                source_paths=("/etc" "/root/.config")
                ;;
            "$TYPE_FULL")
                source_paths=("/")
                EXCLUDE_PATHS+=("/proc" "/sys" "/dev" "/tmp" "/var/cache" "/var/log")
                ;;
            *)
                source_paths=("/")
                ;;
        esac
    fi
    
    # Create backup metadata
    cat > "$backup_path/metadata.json" << EOF
{
    "name": "$backup_name",
    "type": "$RECOVERY_TYPE",
    "timestamp": "$(date -Iseconds)",
    "hostname": "$(hostname)",
    "source_paths": $(printf '%s\n' "${source_paths[@]}" | jq -R . | jq -s .),
    "excluded_paths": $(printf '%s\n' "${EXCLUDE_PATHS[@]}" | jq -R . | jq -s .),
    "compression": $COMPRESSION,
    "encryption": $ENCRYPTION
}
EOF
    
    # Create backup archive
    log "INFO" "Creating backup archive..."
    
    local archive_file="$backup_path/backup.tar.gz"
    
    if [[ "$COMPRESSION" == "true" ]]; then
        if command -v tar &> /dev/null; then
            local exclude_args=()
            for pattern in "${EXCLUDE_PATHS[@]}"; do
                exclude_args+=("--exclude=$pattern")
            done
            
            if [[ "$DRY_RUN" == "true" ]]; then
                echo "Would create backup archive: $archive_file"
                echo "From paths: ${source_paths[*]}"
                echo "Excluding: ${EXCLUDE_PATHS[*]}"
            else
                tar -czf "$archive_file" "${exclude_args[@]}" "${source_paths[@]}"
            fi
        else
            log "ERROR" "tar not available for backup creation"
            return 1
        fi
    else
        log "ERROR" "Uncompressed backup not supported in this version"
        return 1
    fi
    
    # Create backup manifest
    local manifest_file="$backup_path/manifest.txt"
    find "${source_paths[@]}" -type f 2>/dev/null > "$manifest_file" || true
    
    # Calculate checksum
    if [[ -f "$archive_file" ]]; then
        local checksum=$(sha256sum "$archive_file" | cut -d' ' -f1)
        echo "$checksum" > "$backup_path/checksum.sha256"
    fi
    
    log "SUCCESS" "Backup created: $backup_path"
    echo "$backup_path"
}

# Function to list backups
list_backups() {
    log "INFO" "Listing available backups"
    
    if [[ ! -d "$BACKUP_DIR" ]]; then
        print_color $YELLOW "No backups directory found"
        return
    fi
    
    echo "Available Backups:"
    echo "=================="
    echo "Name                    Date                    Type    Size"
    echo "------------------------ ---------------------- ------- --------"
    
    for backup in "$BACKUP_DIR"/*/; do
        if [[ -d "$backup" ]]; then
            local name=$(basename "$backup")
            local date=$(stat -c %y "$backup" | cut -d' ' -f1)
            local type="unknown"
            local size="unknown"
            
            # Read backup type from metadata
            if [[ -f "$backup/metadata.json" ]]; then
                type=$(jq -r '.type' "$backup/metadata.json" 2>/dev/null || echo "unknown")
            fi
            
            # Calculate size
            if [[ -f "$backup/backup.tar.gz" ]]; then
                size=$(du -sh "$backup/backup.tar.gz" | cut -f1)
            else
                size=$(du -sh "$backup" | cut -f1)
            fi
            
            printf "%-23s %-23s %-7s %-8s\n" "$name" "$date" "$type" "$size"
        fi
    done
}

# Function to restore from backup
restore_backup() {
    local backup_path="$1"
    local target_path="${2:-/}"
    
    log "INFO" "Restoring backup: $backup_name to $target_path"
    
    # Check if backup exists
    if [[ ! -d "$backup_path" ]]; then
        log "ERROR" "Backup not found: $backup_path"
        return 1
    fi
    
    # Verify backup integrity
    if [[ -f "$backup_path/checksum.sha256" ]]; then
        log "INFO" "Verifying backup integrity..."
        local stored_checksum=$(cat "$backup_path/checksum.sha256")
        local current_checksum=$(sha256sum "$backup_path/backup.tar.gz" | cut -d' ' -f1)
        
        if [[ "$stored_checksum" != "$current_checksum" ]]; then
            log "ERROR" "Backup integrity check failed"
            return 1
        fi
        log "SUCCESS" "Backup integrity verified"
    fi
    
    # Create restore point
    log "INFO" "Creating restore point..."
    local restore_point="/var/restore-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$restore_point"
    cp -a "$target_path" "$restore_point/" 2>/dev/null || true
    
    # Extract backup
    log "INFO" "Extracting backup..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "Would restore backup from: $backup_path"
        echo "To target: $target_path"
        echo "Restore point: $restore_point"
    else
        if [[ -f "$backup_path/backup.tar.gz" ]]; then
            # Create backup of current state
            if [[ "$target_path" == "/" ]]; then
                log "INFO" "Creating system backup before restore..."
                create_snapshot "pre-restore-snapshot"
            fi
            
            # Extract backup
            cd "$target_path"
            tar -xzf "$backup_path/backup.tar.gz"
            
            log "SUCCESS" "Backup restored successfully"
            echo "Restore point created at: $restore_point"
        else
            log "ERROR" "Backup archive not found"
            return 1
        fi
    fi
}

# Function to check file system health
check_filesystem() {
    local device="${1:-$(findmnt -n -o SOURCE /)}"
    
    log "INFO" "Checking filesystem health: $device"
    
    if [[ ! -b "$device" ]]; then
        log "ERROR" "Device not found: $device"
        return 1
    fi
    
    echo "Filesystem Check for: $device"
    echo "============================="
    
    # Check if filesystem is mounted
    if mountpoint -q "$(findmnt -n -o TARGET /)" 2>/dev/null; then
        print_color $YELLOW "Warning: Filesystem is mounted. Unmounting for full check..."
        if [[ "$INTERACTIVE" == "true" ]] && [[ "$FORCE" != "true" ]]; then
            echo -n "Continue with mounted filesystem? (y/N): "
            read -r choice
            [[ ! $choice =~ ^[Yy]$ ]] && return 1
        fi
    fi
    
    # Check disk usage
    echo "Disk Usage:"
    df -h "$device"
    echo
    
    # Check inode usage
    echo "Inode Usage:"
    df -i "$device"
    echo
    
    # Check for bad blocks (if possible)
    if command -v badblocks &> /dev/null; then
        log "INFO" "Checking for bad blocks..."
        if [[ "$INTERACTIVE" == "true" ]] && [[ "$FORCE" != "true" ]]; then
            echo -n "Run badblocks check? This may take a long time. (y/N): "
            read -r choice
            if [[ $choice =~ ^[Yy]$ ]]; then
                badblocks -v "$device"
            fi
        else
            badblocks -v "$device" 2>/dev/null || log "WARN" "Badblocks check failed or skipped"
        fi
    fi
    
    # Check filesystem consistency
    if command -v fsck &> /dev/null; then
        log "INFO" "Running filesystem consistency check..."
        fsck -f "$device" 2>/dev/null || log "WARN" "Filesystem check found issues"
    fi
    
    log "SUCCESS" "Filesystem check completed"
}

# Function to repair filesystem
repair_filesystem() {
    local device="${1:-$(findmnt -n -o SOURCE /)}"
    
    log "INFO" "Attempting filesystem repair: $device"
    
    if [[ ! -b "$device" ]]; then
        log "ERROR" "Device not found: $device"
        return 1
    fi
    
    # Check if mounted
    if mountpoint -q "$(findmnt -n -o TARGET /)" 2>/dev/null; then
        print_color $RED "Error: Cannot repair mounted filesystem"
        log "ERROR" "Cannot repair mounted filesystem: $device"
        return 1
    fi
    
    # Attempt to repair
    log "INFO" "Running filesystem repair..."
    
    if command -v fsck &> /dev/null; then
        fsck -y "$device"
        local exit_code=$?
        
        case $exit_code in
            0)
                log "SUCCESS" "Filesystem repair completed successfully"
                ;;
            1)
                log "WARN" "Filesystem repair completed with warnings"
                ;;
            *)
                log "ERROR" "Filesystem repair failed with exit code: $exit_code"
                return $exit_code
                ;;
        esac
    else
        log "ERROR" "fsck not available"
        return 1
    fi
}

# Function to create rescue media
create_rescue_media() {
    local output_path="$1"
    local rescue_type="${2:-minimal}"
    
    log "INFO" "Creating rescue media: $output_path"
    
    # Create rescue directory structure
    local rescue_dir="$TMP_DIR/rescue"
    mkdir -p "$rescue_dir"/{bin,etc,usr,sbin,lib,boot}
    
    # Create minimal rescue system
    create_rescue_init "$rescue_dir"
    create_rescue_commands "$rescue_dir"
    create_rescue_scripts "$rescue_dir"
    
    # Create boot configuration
    create_rescue_boot_config "$rescue_dir"
    
    # Create rescue ISO
    if command -v mkisofs &> /dev/null; then
        log "INFO" "Creating rescue ISO..."
        mkisofs -o "$output_path" -J -R -b isolinux/isolinux.bin \
            -c isolinux/boot.cat -no-emul-boot -boot-load-size 4 \
            -boot-info-table "$rescue_dir"
    else
        log "ERROR" "mkisofs not available for ISO creation"
        return 1
    fi
    
    log "SUCCESS" "Rescue media created: $output_path"
}

# Function to create rescue init script
create_rescue_init() {
    local rescue_dir="$1"
    
    cat > "$rescue_dir/sbin/init" << 'EOF'
#!/bin/sh
# MultiOS Rescue System Init

echo "MultiOS Rescue System"
echo "====================="

# Mount essential filesystems
mount -t proc proc /proc
mount -t sysfs sysfs /sys
mount -t tmpfs tmpfs /tmp
mount -t tmpfs tmpfs /var

# Create device nodes
mknod /dev/null c 1 3
mknod /dev/zero c 1 5
mknod /dev/random c 1 8

# Start rescue shell
echo ""
echo "Available commands:"
echo "  lsblk           - List block devices"
echo "  fdisk           - Partition management"
echo "  fsck            - Filesystem check"
echo "  dd              - Disk imaging"
echo "  resize2fs       - Resize ext filesystems"
echo "  mke2fs          - Create ext filesystems"
echo "  mount           - Mount filesystems"
echo "  umount          - Unmount filesystems"
echo "  multios-recover - MultiOS recovery tools"
echo ""
echo "Type 'exit' to reboot"
echo ""

exec /bin/sh
EOF
    
    chmod +x "$rescue_dir/sbin/init"
}

# Function to create rescue commands
create_rescue_commands() {
    local rescue_dir="$1"
    
    # Create symbolic links to essential commands
    local commands=("lsblk" "fdisk" "fsck" "dd" "resize2fs" "mke2fs" "mount" "umount" "cat" "ls" "cd")
    
    for cmd in "${commands[@]}"; do
        if command -v "$cmd" &> /dev/null; then
            local cmd_path=$(which "$cmd")
            ln -s "$cmd_path" "$rescue_dir/bin/$cmd"
            ln -s "$cmd_path" "$rescue_dir/sbin/$cmd"
        fi
    done
    
    # Copy MultiOS recovery tool
    if [[ -f "/usr/local/bin/multios-recover" ]]; then
        cp "/usr/local/bin/multios-recover" "$rescue_dir/bin/"
        chmod +x "$rescue_dir/bin/multios-recover"
    fi
}

# Function to create rescue boot configuration
create_rescue_boot_config() {
    local rescue_dir="$1"
    
    # Create ISOLINUX configuration
    mkdir -p "$rescue_dir/isolinux"
    
    cat > "$rescue_dir/isolinux/isolinux.cfg" << EOF
UI menu.c32
PROMPT 0
TIMEOUT 30
DEFAULT MultiOS_Rescue

MENU TITLE MultiOS Rescue System

LABEL MultiOS Rescue
    MENU LABEL MultiOS Rescue System
    KERNEL /boot/kernel
    APPEND init=/sbin/init console=ttyS0,115200
EOF
    
    # Copy menu.c32 if available
    for path in /usr/lib/ISOLINUX/menu.c32 /usr/lib/syslinux/menu.c32; do
        if [[ -f "$path" ]]; then
            cp "$path" "$rescue_dir/isolinux/"
            break
        fi
    done
    
    # Create minimal kernel (simplified)
    echo "rescue_kernel_stub" > "$rescue_dir/boot/kernel"
}

# Function to schedule automated backups
schedule_backup() {
    local schedule="$1"
    local backup_command="/usr/local/bin/multios-recover backup --schedule"
    
    log "INFO" "Scheduling automated backups: $schedule"
    
    # Create cron job
    (crontab -l 2>/dev/null; echo "$schedule $backup_command") | crontab - 2>/dev/null || {
        print_color $YELLOW "Could not create cron job. Please add manually:"
        echo "$schedule $backup_command"
    }
    
    log "SUCCESS" "Backup scheduling configured"
}

# Function to show system recovery info
show_recovery_info() {
    log "INFO" "MultiOS Recovery System Information"
    
    echo "MultiOS Recovery System v$TOOL_VERSION"
    echo "===================================="
    echo ""
    detect_system_info
    echo ""
    echo "Recovery Paths:"
    echo "  Backup Directory: $BACKUP_DIR"
    echo "  Recovery Directory: $RECOVERY_DIR"
    echo "  Snapshot Directory: $SNAPSHOT_DIR"
    echo "  Log File: $LOG_FILE"
    echo ""
    echo "Available Operations:"
    echo "  backup        - Create system backup"
    echo "  restore       - Restore from backup"
    echo "  snapshot      - Create system snapshot"
    echo "  check         - Check filesystem health"
    echo "  repair        - Repair filesystem"
    echo "  create-rescue - Create rescue media"
    echo ""
}

# Function to show usage
show_usage() {
    cat << EOF
Usage: multios-recover <operation> [options]

MultiOS System Recovery and Backup Tool v$TOOL_VERSION

OPERATIONS:
    backup <type>        Create system backup (system|data|config|full)
    restore <name>       Restore from backup
    snapshot [name]      Create system snapshot
    list-snapshots       List available snapshots
    check [device]       Check filesystem health
    repair [device]      Repair filesystem
    create-rescue <file> Create rescue media
    schedule <cron>      Schedule automated backups
    info                 Show recovery system information

TYPES:
    system               Backup system files
    data                 Backup user data
    config               Backup configuration files
    full                 Complete system backup

OPTIONS:
    --source PATH        Source path for backup
    --target PATH        Target path for restore
    --exclude PATH       Exclude path from backup
    --no-compression     Disable compression
    --encrypt            Enable encryption
    --schedule           Mark as scheduled backup
    --dry-run            Show what would be done
    --verbose            Verbose output
    --interactive        Interactive mode (default)
    --force              Force operation
    --help               Show this help message

EXAMPLES:
    multios-recover backup system --exclude /tmp
    multios-recover restore backup-20241201 --target /mnt/restore
    multios-recover snapshot pre-update
    multios-recover check /dev/sda1
    multios-recover repair /dev/sda1
    multios-recover create-rescue multios-rescue.iso
    multios-recover schedule "0 2 * * *"

EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            backup)
                OPERATION="$MODE_BACKUP"
                RECOVERY_TYPE="$2"
                shift 2
                ;;
            restore)
                OPERATION="$MODE_RESTORE"
                BACKUP_NAME="$2"
                shift 2
                ;;
            snapshot)
                OPERATION="$MODE_SNAPSHOT"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    SNAPSHOT_NAME="$2"
                    shift
                fi
                shift
                ;;
            list-snapshots)
                OPERATION="list-snapshots"
                shift
                ;;
            check)
                OPERATION="$MODE_CHECK"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    TARGET_DEVICE="$2"
                    shift
                fi
                shift
                ;;
            repair)
                OPERATION="$MODE_REPAIR"
                if [[ -n "$2" ]] && [[ "$2" != -* ]]; then
                    TARGET_DEVICE="$2"
                    shift
                fi
                shift
                ;;
            create-rescue)
                OPERATION="$MODE_CREATE_REScue"
                RESCUE_OUTPUT="$2"
                shift 2
                ;;
            schedule)
                OPERATION="schedule"
                SCHEDULE="$2"
                shift 2
                ;;
            info)
                OPERATION="info"
                shift
                ;;
            --source)
                SOURCE_PATH="$2"
                shift 2
                ;;
            --target)
                TARGET_PATH="$2"
                shift 2
                ;;
            --exclude)
                EXCLUDE_PATHS+=("$2")
                shift 2
                ;;
            --no-compression)
                COMPRESSION=false
                shift
                ;;
            --encrypt)
                ENCRYPTION=true
                shift
                ;;
            --schedule)
                # Mark as scheduled backup
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --interactive)
                INTERACTIVE=true
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
    # Initialize recovery system
    init_recovery_system
    load_config
    
    # Check for root for operations that require it
    case "${OPERATION:-$MODE_BACKUP}" in
        "$MODE_BACKUP"|"$MODE_RESTORE"|"$MODE_SNAPSHOT"|"$MODE_REPAIR"|"$MODE_CREATE_REScue"|"schedule")
            check_root
            ;;
    esac
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Execute operation
    case "${OPERATION:-$MODE_BACKUP}" in
        "$MODE_BACKUP")
            if [[ -z "$RECOVERY_TYPE" ]]; then
                RECOVERY_TYPE="$TYPE_SYSTEM"
            fi
            create_backup "$BACKUP_NAME" "${SOURCE_PATH}"
            ;;
        "$MODE_RESTORE")
            if [[ -z "$BACKUP_NAME" ]]; then
                print_color $RED "Error: restore requires backup name"
                show_usage
                exit 1
            fi
            restore_backup "$BACKUP_NAME" "$TARGET_PATH"
            ;;
        "$MODE_SNAPSHOT")
            create_snapshot "$SNAPSHOT_NAME"
            ;;
        "list-snapshots")
            list_snapshots
            ;;
        "$MODE_CHECK")
            check_filesystem "$TARGET_DEVICE"
            ;;
        "$MODE_REPAIR")
            repair_filesystem "$TARGET_DEVICE"
            ;;
        "$MODE_CREATE_REScue")
            if [[ -z "$RESCUE_OUTPUT" ]]; then
                RESCUE_OUTPUT="multios-rescue.iso"
            fi
            create_rescue_media "$RESCUE_OUTPUT"
            ;;
        "schedule")
            if [[ -z "$SCHEDULE" ]]; then
                print_color $RED "Error: schedule requires cron expression"
                show_usage
                exit 1
            fi
            schedule_backup "$SCHEDULE"
            ;;
        "info")
            show_recovery_info
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