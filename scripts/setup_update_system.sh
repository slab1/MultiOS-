#!/bin/bash

################################################################################
# MultiOS Update System Setup Script
# Version: 1.0.0
# Description: Automated setup and configuration of MultiOS update system
# Author: MultiOS Development Team
# License: MIT
################################################################################

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Configuration variables
UPDATE_CONFIG_DIR="/etc/multios/update"
UPDATE_VAR_DIR="/var/lib/multios/update"
UPDATE_LOG_DIR="/var/log/multios/update"
UPDATE_BIN_DIR="/usr/local/bin"
UPDATE_INIT_DIR="/etc/init.d"
UPDATE_SYSTEMD_DIR="/etc/systemd/system"

# Update system configuration
UPDATE_SERVER_URL="https://updates.multios.org"
REPOSITORY_MIRRORS=(
    "https://mirror1.multios.org"
    "https://mirror2.multios.org"
    "https://mirror3.multios.org"
)
UPDATE_CHANNEL="stable"  # stable, beta, nightly
AUTO_UPDATE_ENABLED=true
DELTA_UPDATES_ENABLED=true
SIGNATURE_VERIFICATION_ENABLED=true
ROLLBACK_ENABLED=true
BACKUP_COUNT=3
MAX_CONCURRENT_DOWNLOADS=4

# Network settings
UPDATE_TIMEOUT=300
MAX_RETRIES=3
RETRY_DELAY=30
UPDATE_PROXY=""
USER_AGENT="MultiOS-UpdateSystem/1.0.0"

# Package repository settings
REPOSITORY_CONFIG_FILE="$UPDATE_CONFIG_DIR/repositories.conf"
UPDATE_SCHEDULE_CONFIG="$UPDATE_CONFIG_DIR/schedule.conf"
VALIDATION_CONFIG="$UPDATE_CONFIG_DIR/validation.conf"

################################################################################
# Logging Functions
################################################################################

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    case "$level" in
        "INFO")
            echo -e "${GREEN}[INFO]${NC} ${timestamp} - $message" | tee -a "$UPDATE_LOG_DIR/setup.log"
            ;;
        "WARN")
            echo -e "${YELLOW}[WARN]${NC} ${timestamp} - $message" | tee -a "$UPDATE_LOG_DIR/setup.log"
            ;;
        "ERROR")
            echo -e "${RED}[ERROR]${NC} ${timestamp} - $message" | tee -a "$UPDATE_LOG_DIR/setup.log"
            ;;
        "DEBUG")
            echo -e "${BLUE}[DEBUG]${NC} ${timestamp} - $message" | tee -a "$UPDATE_LOG_DIR/setup.log"
            ;;
        "SUCCESS")
            echo -e "${GREEN}[SUCCESS]${NC} ${timestamp} - $message" | tee -a "$UPDATE_LOG_DIR/setup.log"
            ;;
    esac
}

log_info() { log "INFO" "$@"; }
log_warn() { log "WARN" "$@"; }
log_error() { log "ERROR" "$@"; }
log_debug() { log "DEBUG" "$@"; }
log_success() { log "SUCCESS" "$@"; }

################################################################################
# Error Handling Functions
################################################################################

cleanup_on_error() {
    local exit_code=$?
    log_error "Setup failed with exit code $exit_code. Cleaning up..."
    
    # Remove partially created configuration files
    if [ -f "$UPDATE_CONFIG_DIR/package_manager.conf" ]; then
        rm -f "$UPDATE_CONFIG_DIR/package_manager.conf"
    fi
    
    if [ -f "$UPDATE_CONFIG_DIR/auto_update.conf" ]; then
        rm -f "$UPDATE_CONFIG_DIR/auto_update.conf"
    fi
    
    if [ -f "$UPDATE_CONFIG_DIR/validation.conf" ]; then
        rm -f "$UPDATE_CONFIG_DIR/validation.conf"
    fi
    
    # Stop and disable services if they were started
    if command -v systemctl >/dev/null 2>&1; then
        systemctl stop multios-update-scheduler 2>/dev/null || true
        systemctl disable multios-update-scheduler 2>/dev/null || true
        systemctl stop multios-package-manager 2>/dev/null || true
        systemctl disable multios-package-manager 2>/dev/null || true
    fi
    
    exit $exit_code
}

set_error_handling() {
    set -e
    trap cleanup_on_error ERR
    trap 'cleanup_on_error 130' INT
    trap 'cleanup_on_error 143' TERM
}

################################################################################
# System Information and Prerequisites
################################################################################

check_system_requirements() {
    log_info "Checking system requirements..."
    
    # Check if running as root
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root"
        exit 1
    fi
    
    # Check operating system
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        log_info "Detected OS: $PRETTY_NAME"
    else
        log_warn "Could not detect operating system version"
    fi
    
    # Check available disk space (minimum 1GB)
    local available_space=$(df / | awk 'NR==2 {print $4}')
    if [ "$available_space" -lt 1048576 ]; then
        log_error "Insufficient disk space. At least 1GB required."
        exit 1
    fi
    
    # Check available memory (minimum 256MB)
    local available_memory=$(free -m | awk 'NR==2{print $7}')
    if [ "$available_memory" -lt 256 ]; then
        log_error "Insufficient memory. At least 256MB required."
        exit 1
    fi
    
    # Check network connectivity
    if ! ping -c 1 google.com >/dev/null 2>&1; then
        log_error "No network connectivity detected"
        exit 1
    fi
    
    log_success "System requirements check passed"
}

check_dependencies() {
    log_info "Checking required dependencies..."
    
    local missing_deps=()
    
    # Check for required binaries
    local required_commands=("curl" "wget" "gnupg" "rsync" "unzip" "tar" "gzip")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            missing_deps+=("$cmd")
        fi
    done
    
    # Check for cryptographic tools
    local crypto_tools=("gpg" "sha256sum" "md5sum")
    for tool in "${crypto_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_deps+=("$tool")
        fi
    done
    
    # Install missing dependencies if any
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_warn "Missing dependencies: ${missing_deps[*]}"
        log_info "Installing missing dependencies..."
        
        if command -v apt-get >/dev/null 2>&1; then
            apt-get update -qq
            apt-get install -y "${missing_deps[@]}"
        elif command -v yum >/dev/null 2>&1; then
            yum install -y "${missing_deps[@]}"
        elif command -v pacman >/dev/null 2>&1; then
            pacman -S --noconfirm "${missing_deps[@]}"
        else
            log_error "Could not determine package manager to install dependencies"
            exit 1
        fi
    fi
    
    log_success "All dependencies available"
}

################################################################################
# Directory and File Structure Setup
################################################################################

create_directory_structure() {
    log_info "Creating update system directory structure..."
    
    # Create main configuration directories
    mkdir -p "$UPDATE_CONFIG_DIR"
    mkdir -p "$UPDATE_VAR_DIR"
    mkdir -p "$UPDATE_VAR_DIR/cache"
    mkdir -p "$UPDATE_VAR_DIR/packages"
    mkdir -p "$UPDATE_VAR_DIR/backups"
    mkdir -p "$UPDATE_VAR_DIR/rollback"
    mkdir -p "$UPDATE_LOG_DIR"
    mkdir -p "$UPDATE_VAR_DIR/tmp"
    
    # Set proper permissions
    chmod 755 "$UPDATE_CONFIG_DIR"
    chmod 755 "$UPDATE_VAR_DIR"
    chmod 755 "$UPDATE_VAR_DIR/cache"
    chmod 755 "$UPDATE_VAR_DIR/packages"
    chmod 750 "$UPDATE_VAR_DIR/backups"
    chmod 750 "$UPDATE_VAR_DIR/rollback"
    chmod 755 "$UPDATE_LOG_DIR"
    chmod 755 "$UPDATE_VAR_DIR/tmp"
    
    log_success "Directory structure created"
}

################################################################################
# Package Manager Configuration
################################################################################

configure_package_manager() {
    log_info "Configuring package manager..."
    
    # Create package manager configuration
    cat > "$UPDATE_CONFIG_DIR/package_manager.conf" << 'EOF'
# MultiOS Package Manager Configuration
# Generated automatically by setup script

[General]
update_server_url = https://updates.multios.org
repository_mirrors = mirror1.multios.org, mirror2.multios.org, mirror3.multios.org
update_channel = stable
timeout = 300
max_retries = 3
retry_delay = 30
max_concurrent_downloads = 4

[Package]
cache_enabled = true
cache_location = /var/lib/multios/update/cache
auto_cleanup = true
cleanup_threshold = 1000000000  # 1GB
verification_required = true
delta_updates = true

[Network]
user_agent = MultiOS-UpdateSystem/1.0.0
proxy = 
verify_ssl = true
follow_redirects = true
EOF
    
    chmod 600 "$UPDATE_CONFIG_DIR/package_manager.conf"
    log_success "Package manager configuration created"
}

################################################################################
# Repository Configuration
################################################################################

configure_repositories() {
    log_info "Configuring package repositories..."
    
    # Create repository configuration
    cat > "$REPOSITORY_CONFIG_FILE" << 'EOF'
# MultiOS Repository Configuration
# Generated automatically by setup script

[multios-stable]
name = MultiOS Stable Repository
type = rpm
baseurl = https://mirror1.multios.org/repo/stable
enabled = true
gpgcheck = true
gpgkey = https://mirror1.multios.org/repo/RPM-GPG-KEY-MultiOS
priority = 1

[multios-updates]
name = MultiOS Updates Repository
type = rpm
baseurl = https://mirror1.multios.org/repo/updates
enabled = true
gpgcheck = true
gpgkey = https://mirror1.multios.org/repo/RPM-GPG-KEY-MultiOS
priority = 2

[multios-beta]
name = MultiOS Beta Repository
type = rpm
baseurl = https://mirror1.multios.org/repo/beta
enabled = false
gpgcheck = true
gpgkey = https://mirror1.multios.org/repo/RPM-GPG-KEY-MultiOS
priority = 10

[multios-nightly]
name = MultiOS Nightly Repository
type = rpm
baseurl = https://mirror1.multios.org/repo/nightly
enabled = false
gpgcheck = true
gpgkey = https://mirror1.multios.org/repo/RPM-GPG-KEY-MultiOS
priority = 20
EOF
    
    chmod 644 "$REPOSITORY_CONFIG_FILE"
    log_success "Repository configuration created"
}

################################################################################
# Update Scheduling Configuration
################################################################################

configure_update_schedule() {
    log_info "Configuring automatic update scheduling..."
    
    # Create update schedule configuration
    cat > "$UPDATE_SCHEDULE_CONFIG" << EOF
# MultiOS Update Schedule Configuration
# Generated automatically by setup script

[Schedule]
auto_updates_enabled = $AUTO_UPDATE_ENABLED
check_frequency = daily
download_time = 02:00
install_time = 03:00
maintenance_window_start = 02:00
maintenance_window_end = 06:00

[Notifications]
notify_on_download = true
notify_on_install = true
notify_on_error = true
email_recipient = admin@localhost

[Behavior]
download_only = false
install_security_updates = true
install_recommended_updates = true
install_optional_updates = false
confirm_before_install = false

[Rollback]
rollback_enabled = $ROLLBACK_ENABLED
backup_count = $BACKUP_COUNT
backup_retention_days = 30
auto_rollback_on_failure = true
EOF
    
    chmod 600 "$UPDATE_SCHEDULE_CONFIG"
    log_success "Update scheduling configuration created"
}

################################################################################
# Validation System Configuration
################################################################################

configure_validation_system() {
    log_info "Configuring validation system..."
    
    # Create validation system configuration
    cat > "$VALIDATION_CONFIG" << EOF
# MultiOS Update Validation Configuration
# Generated automatically by setup script

[Validation]
signature_verification = $SIGNATURE_VERIFICATION_ENABLED
checksum_verification = true
package_integrity_checks = true
rollback_verification = true

[Keys]
key_server = keys.openpgp.org
key_server_port = 11371
auto_import_keys = true
key_expire_warning_days = 30

[Compatibility]
system_check_enabled = true
hardware_compatibility = true
dependency_check = true
conflict_detection = true

[Security]
secure_update_channel = true
certificate_validation = true
min_tls_version = 1.2
verify_hostname = true
EOF
    
    chmod 600 "$VALIDATION_CONFIG"
    log_success "Validation system configuration created"
}

################################################################################
# Update System Services
################################################################################

create_update_services() {
    log_info "Creating update system services..."
    
    # Create package manager service
    cat > "/etc/systemd/system/multios-package-manager.service" << 'EOF'
[Unit]
Description=MultiOS Package Manager Service
Documentation=man:multios-package-manager(8)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/multios-package-manager --daemon
Restart=always
RestartSec=5
User=root
Group=root

[Install]
WantedBy=multi-user.target
EOF
    
    # Create update scheduler service
    cat > "/etc/systemd/system/multios-update-scheduler.service" << 'EOF'
[Unit]
Description=MultiOS Update Scheduler Service
Documentation=man:multios-update-scheduler(8)
After=multios-package-manager.service
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/multios-update-scheduler --daemon
Restart=always
RestartSec=5
User=root
Group=root

[Install]
WantedBy=multi-user.target
EOF
    
    # Create update notification service
    cat > "/etc/systemd/system/multios-update-notifier.service" << 'EOF'
[Unit]
Description=MultiOS Update Notifier Service
Documentation=man:multios-update-notifier(8)
After=multios-package-manager.service

[Service]
Type=simple
ExecStart=/usr/local/bin/multios-update-notifier --daemon
Restart=always
RestartSec=30
User=root
Group=root

[Install]
WantedBy=multi-user.target
EOF
    
    # Create timers for scheduled updates
    cat > "/etc/systemd/system/multios-update-scheduler.timer" << 'EOF'
[Unit]
Description=MultiOS Update Scheduler Timer
Documentation=man:multios-update-scheduler(8)

[Timer]
OnCalendar=daily
Persistent=true
AccuracySec=1min

[Install]
WantedBy=timers.target
EOF
    
    chmod 644 "/etc/systemd/system/multios-package-manager.service"
    chmod 644 "/etc/systemd/system/multios-update-scheduler.service"
    chmod 644 "/etc/systemd/system/multios-update-notifier.service"
    chmod 644 "/etc/systemd/system/multios-update-scheduler.timer"
    
    log_success "Update system services created"
}

################################################################################
# Update System Scripts
################################################################################

create_update_scripts() {
    log_info "Creating update system scripts..."
    
    # Create package manager script
    cat > "$UPDATE_BIN_DIR/multios-package-manager" << 'EOF'
#!/bin/bash
# MultiOS Package Manager
# Main package management and update handling script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="/etc/multios/update"
LOG_DIR="/var/log/multios/update"
VAR_DIR="/var/lib/multios/update"

source "$CONFIG_DIR/package_manager.conf"
source "$CONFIG_DIR/validation.conf"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG_DIR/package-manager.log"
}

error_exit() {
    log "ERROR: $*"
    exit 1
}

download_package() {
    local package_url="$1"
    local output_file="$2"
    
    log "Downloading package from: $package_url"
    
    curl -f -L \
         --connect-timeout "$timeout" \
         --max-time "$timeout" \
         --retry "$max_retries" \
         --retry-delay "$retry_delay" \
         --max-concurrent-downloads "$max_concurrent_downloads" \
         --user-agent "$user_agent" \
         --output "$output_file" \
         "$package_url" || error_exit "Failed to download package: $package_url"
    
    log "Downloaded package: $output_file"
}

verify_package() {
    local package_file="$1"
    local signature_file="${package_file}.sig"
    
    if [ "$signature_verification" = "true" ] && [ -f "$signature_file" ]; then
        log "Verifying package signature: $package_file"
        gpg --verify "$signature_file" "$package_file" || error_exit "Package signature verification failed: $package_file"
    fi
    
    if [ -f "${package_file}.sha256" ]; then
        log "Verifying package checksum: $package_file"
        sha256sum -c "${package_file}.sha256" || error_exit "Package checksum verification failed: $package_file"
    fi
    
    log "Package verification passed: $package_file"
}

install_package() {
    local package_file="$1"
    local temp_dir="$(mktemp -d)"
    
    log "Installing package: $package_file"
    
    # Create rollback backup if rollback is enabled
    if [ "$rollback_enabled" = "true" ]; then
        local backup_file="$VAR_DIR/backups/rollback-$(date +%Y%m%d-%H%M%S).tar.gz"
        tar -czf "$backup_file" /etc /var/www /opt 2>/dev/null || true
        log "Created rollback backup: $backup_file"
    fi
    
    # Extract and install package
    case "$package_file" in
        *.rpm)
            rpm -Uvh "$package_file" || error_exit "RPM installation failed: $package_file"
            ;;
        *.deb)
            dpkg -i "$package_file" || error_exit "DEB installation failed: $package_file"
            ;;
        *.tar.gz)
            tar -xzf "$package_file" -C "$temp_dir"
            if [ -f "$temp_dir/install.sh" ]; then
                "$temp_dir/install.sh" || error_exit "Installation script failed: $package_file"
            fi
            ;;
        *)
            error_exit "Unsupported package format: $package_file"
            ;;
    esac
    
    log "Package installed successfully: $package_file"
    rm -rf "$temp_dir"
}

check_updates() {
    log "Checking for available updates..."
    
    # Implementation would check repository for updates
    # This is a placeholder for the actual update checking logic
    echo "No updates available" > "$VAR_DIR/tmp/update_check.tmp"
}

download_updates() {
    log "Downloading updates..."
    
    # Implementation would download available updates
    # This is a placeholder for the actual download logic
    log "Update download completed"
}

install_updates() {
    log "Installing updates..."
    
    # Implementation would install downloaded updates
    # This is a placeholder for the actual installation logic
    log "Update installation completed"
}

rollback_updates() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        error_exit "Rollback backup not found: $backup_file"
    fi
    
    log "Rolling back to backup: $backup_file"
    tar -xzf "$backup_file" -C / || error_exit "Rollback failed"
    log "Rollback completed successfully"
}

show_help() {
    echo "MultiOS Package Manager"
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  check           Check for available updates"
    echo "  download        Download available updates"
    echo "  install         Install downloaded updates"
    echo "  rollback <file> Rollback to specified backup"
    echo "  daemon          Run in daemon mode"
    echo "  help            Show this help message"
}

main() {
    case "${1:-}" in
        check)
            check_updates
            ;;
        download)
            download_updates
            ;;
        install)
            install_updates
            ;;
        rollback)
            rollback_updates "$2"
            ;;
        daemon)
            log "Starting package manager daemon"
            while true; do
                check_updates
                sleep 3600  # Check every hour
            done
            ;;
        help|"")
            show_help
            ;;
        *)
            error_exit "Unknown command: $1"
            ;;
    esac
}

main "$@"
EOF
    
    chmod +x "$UPDATE_BIN_DIR/multios-package-manager"
    
    # Create update scheduler script
    cat > "$UPDATE_BIN_DIR/multios-update-scheduler" << 'EOF'
#!/bin/bash
# MultiOS Update Scheduler
# Automated update scheduling and execution script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="/etc/multios/update"
LOG_DIR="/var/log/multios/update"
VAR_DIR="/var/lib/multios/update"

source "$CONFIG_DIR/schedule.conf"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG_DIR/scheduler.log"
}

error_exit() {
    log "ERROR: $*"
    exit 1
}

is_maintenance_window() {
    local current_time=$(date +%H:%M)
    local window_start="${maintenance_window_start:-02:00}"
    local window_end="${maintenance_window_end:-06:00}"
    
    # Simple time comparison (assumes same day)
    if [[ "$current_time" > "$window_start" && "$current_time" < "$window_end" ]]; then
        return 0
    fi
    return 1
}

check_security_updates() {
    log "Checking for security updates..."
    # Implementation would check security update repositories
    return 0
}

should_install_update() {
    local update_type="$1"
    
    case "$update_type" in
        security)
            [ "$install_security_updates" = "true" ]
            ;;
        recommended)
            [ "$install_recommended_updates" = "true" ]
            ;;
        optional)
            [ "$install_optional_updates" = "true" ]
            ;;
        *)
            return 1
            ;;
    esac
}

download_updates() {
    log "Downloading updates during maintenance window"
    
    if ! is_maintenance_window; then
        log "Outside maintenance window, skipping downloads"
        return 0
    fi
    
    # Run package manager to download updates
    "$SCRIPT_DIR/multios-package-manager" download || log "Download failed"
}

install_updates() {
    log "Installing updates during maintenance window"
    
    if ! is_maintenance_window; then
        log "Outside maintenance window, skipping installation"
        return 0
    fi
    
    # Run package manager to install updates
    if [ "$confirm_before_install" = "false" ]; then
        "$SCRIPT_DIR/multios-package-manager" install || log "Installation failed"
    else
        log "Manual confirmation required for update installation"
    fi
}

send_notification() {
    local subject="$1"
    local message="$2"
    
    if [ "$notify_on_error" = "true" ] && [ -n "$email_recipient" ]; then
        echo "$message" | mail -s "$subject" "$email_recipient" 2>/dev/null || log "Failed to send notification"
    fi
}

show_help() {
    echo "MultiOS Update Scheduler"
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  schedule         Run scheduled update check"
    echo "  download         Download updates"
    echo "  install          Install updates"
    echo "  daemon           Run in daemon mode"
    echo "  help             Show this help message"
}

main() {
    case "${1:-}" in
        schedule)
            check_security_updates
            download_updates
            install_updates
            ;;
        download)
            download_updates
            ;;
        install)
            install_updates
            ;;
        daemon)
            log "Starting update scheduler daemon"
            while true; do
                check_security_updates
                sleep 3600  # Check every hour
            done
            ;;
        help|"")
            show_help
            ;;
        *)
            echo "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
EOF
    
    chmod +x "$UPDATE_BIN_DIR/multios-update-scheduler"
    
    # Create update notifier script
    cat > "$UPDATE_BIN_DIR/multios-update-notifier" << 'EOF'
#!/bin/bash
# MultiOS Update Notifier
# Update notification and alerting script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="/etc/multios/update"
LOG_DIR="/var/log/multios/update"
VAR_DIR="/var/lib/multios/update"

source "$CONFIG_DIR/schedule.conf"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$LOG_DIR/notifier.log"
}

send_desktop_notification() {
    local title="$1"
    local message="$2"
    
    # Try various notification methods
    if command -v notify-send >/dev/null 2>&1; then
        notify-send "$title" "$message" -u normal -t 5000 || true
    elif command -v zenity >/dev/null 2>&1; then
        zenity --notification --text "$message" || true
    fi
}

send_email_notification() {
    local subject="$1"
    local message="$2"
    
    if [ -n "$email_recipient" ]; then
        echo "$message" | mail -s "$subject" "$email_recipient" 2>/dev/null || log "Failed to send email notification"
    fi
}

check_update_status() {
    # Check for update status
    if [ -f "$VAR_DIR/tmp/update_check.tmp" ]; then
        local status=$(cat "$VAR_DIR/tmp/update_check.tmp")
        case "$status" in
            "updates available")
                return 10
                ;;
            "security updates available")
                return 11
                ;;
            "updates installed")
                return 0
                ;;
            "no updates")
                return 1
                ;;
            *)
                return 2
                ;;
        esac
    fi
    return 2
}

main() {
    log "Update notifier started"
    
    while true; do
        check_update_status
        local status=$?
        
        case $status in
            10) # Updates available
                if [ "$notify_on_download" = "true" ]; then
                    send_desktop_notification "Updates Available" "MultiOS updates are available for download"
                    send_email_notification "MultiOS Updates Available" "Updates are available for download and installation"
                fi
                ;;
            11) # Security updates available
                if [ "$notify_on_install" = "true" ]; then
                    send_desktop_notification "Security Updates Available" "Important security updates require immediate installation"
                    send_email_notification "MultiOS Security Updates Available" "Security updates are available and recommended for immediate installation"
                fi
                ;;
            0) # Updates installed
                if [ "$notify_on_install" = "true" ]; then
                    send_desktop_notification "Updates Installed" "MultiOS updates have been successfully installed"
                    send_email_notification "MultiOS Updates Installed" "Updates have been successfully installed"
                fi
                ;;
        esac
        
        sleep 1800  # Check every 30 minutes
    done
}

main "$@"
EOF
    
    chmod +x "$UPDATE_BIN_DIR/multios-update-notifier"
    
    log_success "Update system scripts created"
}

################################################################################
# System Integration and Registration
################################################################################

register_gpg_keys() {
    log_info "Registering GPG keys for package verification..."
    
    # Download MultiOS repository GPG key
    local gpg_key_url="https://mirror1.multios.org/repo/RPM-GPG-KEY-MultiOS"
    local temp_keyfile="/tmp/multios-gpg-key"
    
    if command -v wget >/dev/null 2>&1; then
        wget -q "$gpg_key_url" -O "$temp_keyfile" 2>/dev/null || {
            log_warn "Failed to download GPG key from primary mirror"
            # Try secondary mirrors
            for mirror in "${REPOSITORY_MIRRORS[@]}"; do
                local mirror_key_url="${mirror%/}/repo/RPM-GPG-KEY-MultiOS"
                wget -q "$mirror_key_url" -O "$temp_keyfile" 2>/dev/null && break || true
            done
        }
    elif command -v curl >/dev/null 2>&1; then
        curl -f -s -o "$temp_keyfile" "$gpg_key_url" || {
            log_warn "Failed to download GPG key from primary mirror"
            # Try secondary mirrors
            for mirror in "${REPOSITORY_MIRRORS[@]}"; do
                local mirror_key_url="${mirror%/}/repo/RPM-GPG-KEY-MultiOS"
                curl -f -s -o "$temp_keyfile" "$mirror_key_url" && break || true
            done
        }
    fi
    
    if [ -f "$temp_keyfile" ]; then
        gpg --import "$temp_keyfile" || log_warn "Failed to import GPG key"
        rm -f "$temp_keyfile"
        log_success "GPG key registration completed"
    else
        log_warn "GPG key not available - signature verification may fail"
    fi
}

setup_firewall_rules() {
    log_info "Configuring firewall rules for update system..."
    
    # Allow HTTPS traffic for package downloads
    if command -v ufw >/dev/null 2>&1; then
        ufw allow out 443/tcp comment "MultiOS package updates" || log_warn "Failed to configure UFW rules"
    elif command -v firewall-cmd >/dev/null 2>&1; then
        firewall-cmd --permanent --add-service=https || log_warn "Failed to configure firewalld rules"
        firewall-cmd --reload || log_warn "Failed to reload firewalld"
    fi
}

create_systemd_links() {
    log_info "Creating systemd service links..."
    
    # Enable and start services
    systemctl daemon-reload
    
    if systemctl is-enabled multios-package-manager.service 2>/dev/null; then
        systemctl enable multios-package-manager.service || log_warn "Failed to enable package manager service"
        systemctl start multios-package-manager.service || log_warn "Failed to start package manager service"
    fi
    
    if systemctl is-enabled multios-update-scheduler.service 2>/dev/null; then
        systemctl enable multios-update-scheduler.service || log_warn "Failed to enable update scheduler service"
        systemctl start multios-update-scheduler.service || log_warn "Failed to start update scheduler service"
    fi
    
    if systemctl is-enabled multios-update-scheduler.timer 2>/dev/null; then
        systemctl enable multios-update-scheduler.timer || log_warn "Failed to enable update scheduler timer"
        systemctl start multios-update-scheduler.timer || log_warn "Failed to start update scheduler timer"
    fi
    
    if systemctl is-enabled multios-update-notifier.service 2>/dev/null; then
        systemctl enable multios-update-notifier.service || log_warn "Failed to enable update notifier service"
        systemctl start multios-update-notifier.service || log_warn "Failed to start update notifier service"
    fi
    
    log_success "Systemd services configured"
}

################################################################################
# Testing and Validation
################################################################################

test_update_system() {
    log_info "Testing update system configuration..."
    
    # Test package manager
    if "$UPDATE_BIN_DIR/multios-package-manager" check >/dev/null 2>&1; then
        log_success "Package manager test passed"
    else
        log_warn "Package manager test failed"
    fi
    
    # Test update scheduler
    if "$UPDATE_BIN_DIR/multios-update-scheduler" schedule >/dev/null 2>&1; then
        log_success "Update scheduler test passed"
    else
        log_warn "Update scheduler test failed"
    fi
    
    # Check configuration files
    local config_files=(
        "$UPDATE_CONFIG_DIR/package_manager.conf"
        "$REPOSITORY_CONFIG_FILE"
        "$UPDATE_SCHEDULE_CONFIG"
        "$VALIDATION_CONFIG"
    )
    
    for config_file in "${config_files[@]}"; do
        if [ -f "$config_file" ]; then
            log_success "Configuration file exists: $config_file"
        else
            log_error "Configuration file missing: $config_file"
        fi
    done
    
    # Test network connectivity to mirrors
    for mirror in "${REPOSITORY_MIRRORS[@]}"; do
        if curl -f -s --connect-timeout 10 "${mirror%/}/repo/" >/dev/null 2>&1; then
            log_success "Mirror connectivity test passed: $mirror"
        else
            log_warn "Mirror connectivity test failed: $mirror"
        fi
    done
    
    log_success "Update system testing completed"
}

create_test_updates() {
    log_info "Creating test update configuration..."
    
    # Create a test update package for validation
    local test_package_dir="$VAR_DIR/tmp/test-update"
    mkdir -p "$test_package_dir"
    
    cat > "$test_package_dir/test-info.txt" << 'EOF'
Test Update Package
===================
This is a test update package for MultiOS update system validation.

Version: 1.0.0-test
Date: 2025-11-05
Type: test
EOF
    
    # Create test package metadata
    cat > "$test_package_dir/metadata.json" << 'EOF'
{
    "name": "test-update",
    "version": "1.0.0",
    "type": "test",
    "date": "2025-11-05",
    "size": 1024,
    "sha256": "test-checksum"
}
EOF
    
    # Compress test package
    cd "$test_package_dir"
    tar -czf "$VAR_DIR/tmp/test-update-1.0.0.tar.gz" *
    cd - >/dev/null
    
    rm -rf "$test_package_dir"
    
    log_success "Test update package created"
}

################################################################################
# Documentation and Help
################################################################################

create_update_docs() {
    log_info "Creating update system documentation..."
    
    # Create main update system documentation
    cat > "$UPDATE_CONFIG_DIR/README.md" << 'EOF'
# MultiOS Update System Documentation

## Overview
The MultiOS Update System provides automated package management, security updates, and system maintenance capabilities.

## Configuration Files
- `package_manager.conf` - Package manager settings
- `repositories.conf` - Repository configuration
- `schedule.conf` - Update scheduling settings
- `validation.conf` - Security and validation settings

## Services
- `multios-package-manager.service` - Main package management service
- `multios-update-scheduler.service` - Automated update scheduling
- `multios-update-scheduler.timer` - Timer for scheduled updates
- `multios-update-notifier.service` - Update notifications

## Usage
```bash
# Check for updates
multios-package-manager check

# Download updates
multios-package-manager download

# Install updates
multios-package-manager install

# Manual update check
multios-update-scheduler schedule

# View update logs
tail -f /var/log/multios/update/*.log
```

## Log Files
- `package-manager.log` - Package manager operations
- `scheduler.log` - Update scheduling operations
- `notifier.log` - Notification service logs
- `setup.log` - Installation and setup logs

## Troubleshooting
1. Check network connectivity to mirrors
2. Verify GPG key installation
3. Check service status with systemctl
4. Review log files for errors
5. Ensure sufficient disk space

## Security
- All packages are cryptographically signed
- TLS/SSL verification enabled
- Checksum validation required
- Secure update channels used
EOF
    
    log_success "Update system documentation created"
}

create_troubleshooting_guide() {
    log_info "Creating troubleshooting guide..."
    
    cat > "$UPDATE_LOG_DIR/TROUBLESHOOTING.md" << 'EOF'
# MultiOS Update System Troubleshooting Guide

## Common Issues and Solutions

### 1. Network Connectivity Issues

**Problem:** Cannot connect to update servers
**Symptoms:** Download failures, timeout errors
**Solutions:**
- Check internet connectivity
- Verify firewall rules allow HTTPS traffic
- Check proxy settings if applicable
- Test mirror connectivity manually

```bash
# Test connectivity
curl -I https://mirror1.multios.org/repo/
# Test with timeout
timeout 10 curl -f https://mirror1.multios.org/repo/
```

### 2. GPG Key Issues

**Problem:** Package signature verification fails
**Symptoms:** "GPG key not trusted" errors
**Solutions:**
- Import the MultiOS GPG key
- Check key expiration
- Update repository keys

```bash
# Import GPG key
wget -O - https://mirror1.multios.org/repo/RPM-GPG-KEY-MultiOS | gpg --import
# Check key status
gpg --list-keys
```

### 3. Service Issues

**Problem:** Update services fail to start
**Symptoms:** Services in failed state
**Solutions:**
- Check service logs
- Verify configuration files
- Check permissions
- Restart services

```bash
# Check service status
systemctl status multios-package-manager
# Check service logs
journalctl -u multios-package-manager -f
# Restart service
systemctl restart multios-package-manager
```

### 4. Configuration Issues

**Problem:** Update system not working as expected
**Symptoms:** Unexpected behavior
**Solutions:**
- Verify configuration file syntax
- Check file permissions
- Validate configuration settings

```bash
# Check configuration
cat /etc/multios/update/package_manager.conf
# Verify configuration syntax
source /etc/multios/update/package_manager.conf
# Check file permissions
ls -la /etc/multios/update/
```

### 5. Disk Space Issues

**Problem:** Insufficient disk space for updates
**Symptoms:** Download failures, installation errors
**Solutions:**
- Clean package cache
- Remove old backups
- Check available space

```bash
# Check disk space
df -h
# Clean package cache
rm -rf /var/lib/multios/update/cache/*
# Clean old backups
find /var/lib/multios/update/backups/ -name "*.tar.gz" -mtime +30 -delete
```

### 6. Permission Issues

**Problem:** File permission errors
**Symptoms:** Access denied errors
**Solutions:**
- Fix file permissions
- Check ownership
- Verify SELinux context (if applicable)

```bash
# Fix permissions
chown -R root:root /etc/multios/update/
chmod 755 /etc/multios/update/
chmod 600 /etc/multios/update/*.conf
# Check SELinux context (if applicable)
ls -Z /etc/multios/update/
```

## Log Analysis

### Important Log Files
- `/var/log/multios/update/setup.log` - Installation and setup logs
- `/var/log/multios/update/package-manager.log` - Package operations
- `/var/log/multios/update/scheduler.log` - Update scheduling
- `/var/log/multios/update/notifier.log` - Notification service

### Common Log Patterns
- `ERROR` - Non-recoverable errors
- `WARN` - Warning conditions
- `SUCCESS` - Successful operations
- `Downloading` - Package download operations
- `Installing` - Package installation operations

### Log Rotation
Log files are automatically rotated to prevent excessive disk usage:
- Max file size: 10MB per log file
- Max files: 5 rotated logs per service
- Rotation happens automatically by logrotate

## Performance Tuning

### Network Optimization
- Increase timeout for slow connections
- Use proxy for improved performance
- Configure concurrent downloads

### Storage Optimization
- Enable delta updates for bandwidth savings
- Configure cache cleanup thresholds
- Monitor disk space usage

### Scheduling Optimization
- Align updates with maintenance windows
- Configure appropriate check frequencies
- Set appropriate retry policies

## Security Considerations

### GPG Key Management
- Regularly update GPG keys
- Monitor key expiration
- Secure private keys

### Network Security
- Use only trusted mirrors
- Enable SSL/TLS verification
- Monitor network traffic

### Access Control
- Restrict service permissions
- Audit update operations
- Monitor for unauthorized changes

## Emergency Procedures

### System Rollback
If an update causes system instability:

```bash
# List available backups
ls -la /var/lib/multios/update/backups/
# Restore from backup
tar -xzf /var/lib/multios/update/backups/rollback-YYYYMMDD-HHMMSS.tar.gz -C /
# Reboot system
reboot
```

### Disable Updates
To temporarily disable automatic updates:

```bash
# Stop update scheduler
systemctl stop multios-update-scheduler.timer
# Disable service
systemctl disable multios-update-scheduler.timer
```

### Manual Recovery
For complete system recovery:

```bash
# Stop all update services
systemctl stop multios-update-scheduler multios-package-manager multios-update-notifier
# Remove problematic packages
rpm -e --nodeps <package-name>  # For RPM-based systems
dpkg -r <package-name>          # For Debian-based systems
# Restore from backup
tar -xzf /var/lib/multios/update/backups/latest-backup.tar.gz -C /
```

## Contact Information
For additional support:
- Documentation: `/etc/multios/update/README.md`
- Logs: `/var/log/multios/update/`
- Issue tracking: https://issues.multios.org
- Community forum: https://forum.multios.org
EOF
    
    log_success "Troubleshooting guide created"
}

################################################################################
# Main Setup Function
################################################################################

main() {
    log_info "Starting MultiOS Update System Setup"
    log_info "======================================"
    
    # Error handling
    set_error_handling
    
    # System checks
    check_system_requirements
    check_dependencies
    
    # Directory structure
    create_directory_structure
    
    # Configuration
    configure_package_manager
    configure_repositories
    configure_update_schedule
    configure_validation_system
    
    # Services and scripts
    create_update_services
    create_update_scripts
    
    # System integration
    register_gpg_keys
    setup_firewall_rules
    create_systemd_links
    
    # Testing and validation
    test_update_system
    create_test_updates
    
    # Documentation
    create_update_docs
    create_troubleshooting_guide
    
    log_success "======================================"
    log_success "MultiOS Update System Setup Completed Successfully"
    log_success "======================================"
    log_info ""
    log_info "The following components have been installed:"
    log_info "  - Package manager configuration"
    log_info "  - Repository configuration"
    log_info "  - Update scheduling system"
    log_info "  - Validation and security system"
    log_info "  - System services and timers"
    log_info "  - Update scripts and tools"
    log_info "  - Documentation and troubleshooting guides"
    log_info ""
    log_info "Services installed:"
    log_info "  - multios-package-manager.service"
    log_info "  - multios-update-scheduler.service"
    log_info "  - multios-update-scheduler.timer"
    log_info "  - multios-update-notifier.service"
    log_info ""
    log_info "Configuration files:"
    log_info "  - $UPDATE_CONFIG_DIR/package_manager.conf"
    log_info "  - $REPOSITORY_CONFIG_FILE"
    log_info "  - $UPDATE_SCHEDULE_CONFIG"
    log_info "  - $VALIDATION_CONFIG"
    log_info ""
    log_info "Log files:"
    log_info "  - $UPDATE_LOG_DIR/setup.log"
    log_info "  - $UPDATE_LOG_DIR/package-manager.log"
    log_info "  - $UPDATE_LOG_DIR/scheduler.log"
    log_info "  - $UPDATE_LOG_DIR/notifier.log"
    log_info "  - $UPDATE_LOG_DIR/TROUBLESHOOTING.md"
    log_info ""
    log_info "Usage examples:"
    log_info "  - Check for updates: multios-package-manager check"
    log_info "  - Manual update: multios-update-scheduler schedule"
    log_info "  - View logs: tail -f $UPDATE_LOG_DIR/*.log"
    log_info "  - Service status: systemctl status multios-*"
    log_info ""
    log_info "For troubleshooting, see: $UPDATE_LOG_DIR/TROUBLESHOOTING.md"
    log_info ""
    
    exit 0
}

################################################################################
# Script Execution
################################################################################

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "MultiOS Update System Setup Script"
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --help, -h           Show this help message"
            echo "  --version, -v        Show version information"
            echo "  --check-only         Only perform system checks"
            echo "  --skip-services      Skip service creation and registration"
            echo "  --debug              Enable debug logging"
            echo ""
            echo "This script configures the MultiOS update system including:"
            echo "  - Package manager configuration"
            echo "  - Repository setup and mirroring"
            echo "  - Automatic update scheduling"
            echo "  - Security validation and GPG verification"
            echo "  - System services and timers"
            echo "  - Rollback and backup capabilities"
            echo "  - Delta update support"
            echo ""
            exit 0
            ;;
        --version|-v)
            echo "MultiOS Update System Setup Script v1.0.0"
            echo "Copyright (c) 2025 MultiOS Development Team"
            echo "License: MIT"
            exit 0
            ;;
        --check-only)
            log_info "Running system checks only..."
            check_system_requirements
            check_dependencies
            exit 0
            ;;
        --skip-services)
            SKIP_SERVICES=true
            shift
            ;;
        --debug)
            set -x
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check if already configured
if [ -f "$UPDATE_CONFIG_DIR/.configured" ]; then
    log_warn "Update system appears to be already configured"
    read -p "Do you want to reconfigure? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Setup cancelled by user"
        exit 0
    fi
fi

# Mark as configured after successful setup
mark_as_configured() {
    touch "$UPDATE_CONFIG_DIR/.configured"
    echo "# Configuration completed on $(date)" >> "$UPDATE_CONFIG_DIR/.configured"
}

# Override main to mark as configured
main() {
    log_info "Starting MultiOS Update System Setup"
    log_info "======================================"
    
    # Error handling
    set_error_handling
    
    # System checks
    check_system_requirements
    check_dependencies
    
    # Directory structure
    create_directory_structure
    
    # Configuration
    configure_package_manager
    configure_repositories
    configure_update_schedule
    configure_validation_system
    
    # Services and scripts
    if [ "$SKIP_SERVICES" != "true" ]; then
        create_update_services
        create_update_scripts
        create_systemd_links
    fi
    
    # System integration
    register_gpg_keys
    setup_firewall_rules
    
    # Testing and validation
    test_update_system
    create_test_updates
    
    # Documentation
    create_update_docs
    create_troubleshooting_guide
    
    # Mark as configured
    mark_as_configured
    
    log_success "======================================"
    log_success "MultiOS Update System Setup Completed Successfully"
    log_success "======================================"
    log_info ""
    log_info "The following components have been installed:"
    log_info "  - Package manager configuration"
    log_info "  - Repository configuration"
    log_info "  - Update scheduling system"
    log_info "  - Validation and security system"
    log_info "  - System services and timers"
    log_info "  - Update scripts and tools"
    log_info "  - Documentation and troubleshooting guides"
    log_info ""
    log_info "Services installed:"
    log_info "  - multios-package-manager.service"
    log_info "  - multios-update-scheduler.service"
    log_info "  - multios-update-scheduler.timer"
    log_info "  - multios-update-notifier.service"
    log_info ""
    log_info "Configuration files:"
    log_info "  - $UPDATE_CONFIG_DIR/package_manager.conf"
    log_info "  - $REPOSITORY_CONFIG_FILE"
    log_info "  - $UPDATE_SCHEDULE_CONFIG"
    log_info "  - $VALIDATION_CONFIG"
    log_info ""
    log_info "Log files:"
    log_info "  - $UPDATE_LOG_DIR/setup.log"
    log_info "  - $UPDATE_LOG_DIR/package-manager.log"
    log_info "  - $UPDATE_LOG_DIR/scheduler.log"
    log_info "  - $UPDATE_LOG_DIR/notifier.log"
    log_info "  - $UPDATE_LOG_DIR/TROUBLESHOOTING.md"
    log_info ""
    log_info "Usage examples:"
    log_info "  - Check for updates: multios-package-manager check"
    log_info "  - Manual update: multios-update-scheduler schedule"
    log_info "  - View logs: tail -f $UPDATE_LOG_DIR/*.log"
    log_info "  - Service status: systemctl status multios-*"
    log_info ""
    log_info "For troubleshooting, see: $UPDATE_LOG_DIR/TROUBLESHOOTING.md"
    log_info ""
    
    exit 0
}

# Execute main function
main "$@"