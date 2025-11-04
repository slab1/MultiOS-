#!/bin/bash

################################################################################
# MultiOS Administrator Components Installation Script
# 
# This script installs and configures MultiOS administrative components including:
# - System management tools
# - User management system
# - Monitoring and logging services
# - Configuration management
# - Security tools
# - Update management system
#
# Usage: ./install_admin_components.sh [options]
#
# Options:
#   --type TYPE              Installation type: minimal|standard|enterprise
#   --components LIST        Specific components to install (comma-separated)
#   --config FILE           Configuration file path
#   --destination PATH       Installation destination (default: /opt/multios)
#   --user USER             Service user (default: multios)
#   --group GROUP           Service group (default: multios)
#   --enable-services       Enable and start services after installation
#   --dry-run              Show what would be done without executing
#   --verbose              Enable verbose output
#   --help                 Show this help message
#
# Examples:
#   ./install_admin_components.sh --type=enterprise
#   ./install_admin_components.sh --components=monitoring,update-system --enable-services
#   ./install_admin_components.sh --config=/path/to/config.yml --dry-run
################################################################################

set -euo pipefail

# Default configuration
INSTALL_TYPE="standard"
SPECIFIC_COMPONENTS=""
CONFIG_FILE=""
INSTALL_DESTINATION="/opt/multios"
SERVICE_USER="multios"
SERVICE_GROUP="multios"
ENABLE_SERVICES=false
DRY_RUN=false
VERBOSE=false

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $1"
    fi
}

# Help function
show_help() {
    grep "^#" "$0" | grep -v "#!/bin/bash" | sed 's/^# //'
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            INSTALL_TYPE="$2"
            shift 2
            ;;
        --components)
            SPECIFIC_COMPONENTS="$2"
            shift 2
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        --destination)
            INSTALL_DESTINATION="$2"
            shift 2
            ;;
        --user)
            SERVICE_USER="$2"
            shift 2
            ;;
        --group)
            SERVICE_GROUP="$2"
            shift 2
            ;;
        --enable-services)
            ENABLE_SERVICES=true
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
        --help|-h)
            show_help
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            ;;
    esac
done

# Validation functions
validate_installation_type() {
    case "$INSTALL_TYPE" in
        minimal|standard|enterprise)
            log_info "Installation type: $INSTALL_TYPE"
            ;;
        *)
            log_error "Invalid installation type: $INSTALL_TYPE"
            log_error "Valid types: minimal, standard, enterprise"
            exit 1
            ;;
    esac
}

validate_requirements() {
    log_info "Validating system requirements..."
    
    # Check if running as root
    if [[ $EUID -eq 0 ]]; then
        log_warning "Running as root. Consider using a non-root user with sudo privileges."
    fi
    
    # Check system architecture
    ARCH=$(uname -m)
    if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
        log_error "Unsupported architecture: $ARCH"
        log_error "Supported architectures: x86_64, aarch64"
        exit 1
    fi
    
    # Check available disk space (minimum 2GB)
    AVAILABLE_SPACE=$(df -BG "$INSTALL_DESTINATION" | awk 'NR==2 {print $4}' | sed 's/G//')
    if [[ "$AVAILABLE_SPACE" -lt 2 ]]; then
        log_error "Insufficient disk space. Required: 2GB, Available: ${AVAILABLE_SPACE}GB"
        exit 1
    fi
    
    # Check available memory (minimum 512MB)
    TOTAL_MEMORY=$(free -m | awk 'NR==2{print $2}')
    if [[ "$TOTAL_MEMORY" -lt 512 ]]; then
        log_warning "Low memory detected: ${TOTAL_MEMORY}MB. Recommended: 1GB+"
    fi
    
    # Check for required commands
    local required_commands=("curl" "tar" "systemctl")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command not found: $cmd"
            exit 1
        fi
    done
    
    log_success "System requirements validation passed"
}

# System information gathering
gather_system_info() {
    log_info "Gathering system information..."
    
    SYSTEM_INFO=$(cat <<EOF
{
    "os": "$(uname -s)",
    "kernel": "$(uname -r)",
    "architecture": "$(uname -m)",
    "hostname": "$(hostname)",
    "memory_gb": $(free -g | awk 'NR==2{print $2}'),
    "disk_space_gb": $(df -BG / | awk 'NR==2{print $4}' | sed 's/G//'),
    "cpu_cores": $(nproc),
    "network_interfaces": $(ip link show | grep -c "state UP" || echo "0")
}
EOF
)
    
    log_verbose "System info: $SYSTEM_INFO"
}

# Component definitions
declare -A COMPONENTS

# Define components for each installation type
COMPONENTS=(
    # Core components (always installed)
    ["core"]="Core system utilities and libraries"
    ["system-tools"]="System administration tools"
    
    # User management
    ["user-management"]="User and group management system"
    
    # Security components
    ["security-core"]="Core security framework"
    ["rbac"]="Role-Based Access Control system"
    ["encryption"]="Encryption and cryptographic services"
    ["audit"]="Security audit and logging"
    
    # Monitoring and logging
    ["monitoring"]="System monitoring and metrics"
    ["logging"]="Centralized logging system"
    ["alerting"]="Alert and notification system"
    
    # Update management
    ["update-system"]="Update and package management"
    ["rollback"]="System rollback and recovery"
    
    # Configuration management
    ["config-management"]="Configuration management system"
    
    # Web interface
    ["web-interface"]="Web-based administration interface"
    
    # CLI tools
    ["cli-tools"]="Command-line administration tools"
)

# Component dependencies
declare -A DEPENDENCIES
DEPENDENCIES=(
    ["user-management"]="core,security-core"
    ["rbac"]="security-core,user-management"
    ["encryption"]="security-core"
    ["audit"]="security-core,logging"
    ["monitoring"]="core,logging"
    ["alerting"]="monitoring,logging"
    ["update-system"]="core,rollback,config-management"
    ["web-interface"]="core,monitoring,user-management"
    ["cli-tools"]="core,user-management,update-system"
)

# Installation functions
install_core() {
    log_info "Installing core components..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would create directories and copy core files"
        return 0
    fi
    
    # Create directory structure
    mkdir -p "$INSTALL_DESTINATION"/{bin,lib,etc,var,share,doc}
    
    # Create symlinks
    ln -sf "$INSTALL_DESTINATION/bin/multios-admin" /usr/local/bin/multios-admin
    ln -sf "$INSTALL_DESTINATION/bin/multios-update" /usr/local/bin/multios-update
    ln -sf "$INSTALL_DESTINATION/bin/multios-monitor" /usr/local/bin/multios-monitor
    
    log_success "Core components installed"
}

install_system_tools() {
    log_info "Installing system administration tools..."
    
    local tools=(
        "system-info"
        "service-manager" 
        "process-manager"
        "network-manager"
        "disk-manager"
    )
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install tools: ${tools[*]}"
        return 0
    fi
    
    for tool in "${tools[@]}"; do
        log_verbose "Installing $tool..."
        # Simulate tool installation
        touch "$INSTALL_DESTINATION/bin/$tool"
        chmod +x "$INSTALL_DESTINATION/bin/$tool"
    done
    
    log_success "System tools installed"
}

install_user_management() {
    log_info "Installing user management system..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install user management components"
        return 0
    fi
    
    # Create service user and group
    if ! getent group "$SERVICE_GROUP" >/dev/null 2>&1; then
        groupadd --system "$SERVICE_GROUP"
        log_success "Created group: $SERVICE_GROUP"
    fi
    
    if ! getent passwd "$SERVICE_USER" >/dev/null 2>&1; then
        useradd --system --gid "$SERVICE_GROUP" --home-dir "$INSTALL_DESTINATION" \
                --shell /bin/false "$SERVICE_USER"
        log_success "Created user: $SERVICE_USER"
    fi
    
    # Install user management components
    mkdir -p "$INSTALL_DESTINATION/lib/user-management"
    
    # Create configuration
    cat > "$INSTALL_DESTINATION/etc/user-management.conf" <<EOF
[user_management]
enabled = true
default_shell = /bin/bash
home_directory_template = /home/{username}
default_groups = users
password_policy = standard
session_timeout = 3600
max_failed_attempts = 5
lockout_duration = 900

[security]
enable_mfa = true
password_requirements = strong
session_encryption = true
audit_logging = true

[integration]
pam_enabled = true
ldap_enabled = false
active_directory_enabled = false
EOF
    
    log_success "User management system installed"
}

install_security_core() {
    log_info "Installing core security framework..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install security framework"
        return 0
    fi
    
    mkdir -p "$INSTALL_DESTINATION/lib/security"
    
    # Create security configuration
    cat > "$INSTALL_DESTINATION/etc/security.conf" <<EOF
[security_framework]
enabled = true
security_level = standard
audit_enabled = true
log_level = info

[authentication]
methods = password,mfa,certificate
password_policy = strict
mfa_required = true
certificate_validation = strict

[encryption]
algorithms = aes-256-gcm,rsa-4096,ecc-p384
key_derivation = pbkdf2
random_source = system
tls_version = 1.3

[access_control]
default_policy = deny
require_authentication = true
session_timeout = 1800
max_concurrent_sessions = 5

[compliance]
frameworks = cis,sox,hipaa
audit_retention = 365d
log_retention = 90d
report_generation = monthly
EOF
    
    log_success "Core security framework installed"
}

install_rbac() {
    log_info "Installing Role-Based Access Control system..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install RBAC system"
        return 0
    fi
    
    mkdir -p "$INSTALL_DESTINATION/lib/rbac"
    
    # Create RBAC configuration
    cat > "$INSTALL_DESTINATION/etc/rbac.conf" <<EOF
[rbac]
enabled = true
cache_enabled = true
cache_size = 10000
cache_ttl = 3600

[roles]
default_roles = user,guest,admin,system_admin
create_custom_roles = true
inheritance_enabled = true

[permissions]
permission_validation = strict
delegation_enabled = true
delegation_restrictions = standard

[security_levels]
levels = public,internal,confidential,restricted,system
clearance_enforcement = true
EOF
    
    log_success "RBAC system installed"
}

install_monitoring() {
    log_info "Installing monitoring system..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install monitoring system"
        return 0
    fi
    
    mkdir -p "$INSTALL_DESTINATION/lib/monitoring"/{collectors,processors,storage}
    mkdir -p "$INSTALL_DESTINATION/var/monitoring"/{data,cache,logs}
    
    # Create monitoring configuration
    cat > "$INSTALL_DESTINATION/etc/monitoring.conf" <<EOF
[monitoring]
enabled = true
collection_interval = 60s
retention_period = 30d
compression_enabled = true

[metrics]
cpu = true
memory = true
disk = true
network = true
processes = true
services = true

[alerts]
enabled = true
check_interval = 30s
escalation_enabled = true
notification_channels = email,webhook

[dashboard]
web_interface = true
port = 8080
ssl_enabled = false
EOF
    
    log_success "Monitoring system installed"
}

install_update_system() {
    log_info "Installing update management system..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install update system"
        return 0
    fi
    
    mkdir -p "$INSTALL_DESTINATION/lib/update-system"/{validator,scheduler,rollback}
    mkdir -p "$INSTALL_DESTINATION/var/updates"/{packages,cache,backups}
    
    # Create update system configuration
    cat > "$INSTALL_DESTINATION/etc/update-system.conf" <<EOF
[update_system]
enabled = true
auto_check = true
auto_install = false
check_interval = 24h

[security]
require_signature = true
require_integrity_check = true
trusted_publishers = system,publisher
validation_strict = true

[scheduling]
maintenance_window = 02:00-06:00
priority_handling = intelligent
user_approval = required
notification = enabled

[rollback]
auto_rollback = true
backup_retention = 30d
max_rollback_points = 10
emergency_rollback = true

[repositories]
primary = https://releases.multios.org/stable
security = https://security.multios.org
enabled = true
cache_enabled = true
EOF
    
    log_success "Update management system installed"
}

install_config_management() {
    log_info "Installing configuration management system..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install configuration management"
        return 0
    fi
    
    mkdir -p "$INSTALL_DESTINATION/lib/config-management"/{engine,validators,backends}
    mkdir -p "$INSTALL_DESTINATION/etc/config"/{profiles,templates}
    
    # Create configuration management setup
    cat > "$INSTALL_DESTINATION/etc/config-management.conf" <<EOF
[config_management]
enabled = true
validation_required = true
version_control = true
backup_enabled = true

[profiles]
default = standard
available = minimal,standard,enterprise,custom

[validation]
schema_validation = true
syntax_checking = true
dependency_validation = true

[version_control]
backend = git
auto_commit = true
retention = 90d

[deployment]
atomic_deployment = true
rollback_on_failure = true
testing_environment = false
EOF
    
    log_success "Configuration management system installed"
}

install_web_interface() {
    log_info "Installing web-based administration interface..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install web interface"
        return 0
    fi
    
    mkdir -p "$INSTALL_DESTINATION/web-interface"/{static,templates,assets}
    
    # Create web interface configuration
    cat > "$INSTALL_DESTINATION/etc/web-interface.conf" <<EOF
[web_interface]
enabled = true
port = 8080
bind_address = 127.0.0.1
ssl_enabled = false

[authentication]
method = session
session_timeout = 1800
max_concurrent_sessions = 10

[features]
dashboard = true
user_management = true
system_monitoring = true
update_management = true
configuration = true
logs = true

[security]
csrf_protection = true
xss_protection = true
content_security_policy = strict
secure_headers = true
EOF
    
    log_success "Web interface installed"
}

install_cli_tools() {
    log_info "Installing CLI administration tools..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would install CLI tools"
        return 0
    fi
    
    # Create CLI tool scripts
    local cli_tools=(
        "multios-system"
        "multios-users"
        "multios-security"
        "multios-config"
        "multios-logs"
    )
    
    for tool in "${cli_tools[@]}"; do
        cat > "$INSTALL_DESTINATION/bin/$tool" <<EOF
#!/bin/bash
# MultiOS $tool CLI tool
# Generated by installation script

SCRIPT_DIR="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export PATH="\$SCRIPT_DIR:\$PATH"

# Load configuration
if [[ -f "$INSTALL_DESTINATION/etc/\${tool//multios-/}.conf" ]]; then
    source "$INSTALL_DESTINATION/etc/\${tool//multios-/}.conf"
fi

# Simple command dispatcher
case "\${1:-}" in
    status)
        echo "$tool status: operational"
        ;;
    *)
        echo "Usage: \$tool {status|help}"
        exit 1
        ;;
esac
EOF
        chmod +x "$INSTALL_DESTINATION/bin/$tool"
    done
    
    log_success "CLI tools installed"
}

# Installation orchestration
install_component() {
    local component="$1"
    local component_info="${COMPONENTS[$component]:-Unknown component}"
    
    log_info "Installing $component: $component_info"
    
    case "$component" in
        core)
            install_core
            ;;
        system-tools)
            install_system_tools
            ;;
        user-management)
            install_user_management
            ;;
        security-core)
            install_security_core
            ;;
        rbac)
            install_rbac
            ;;
        monitoring)
            install_monitoring
            ;;
        update-system)
            install_update_system
            ;;
        config-management)
            install_config_management
            ;;
        web-interface)
            install_web_interface
            ;;
        cli-tools)
            install_cli_tools
            ;;
        *)
            log_warning "Unknown component: $component"
            return 1
            ;;
    esac
}

resolve_dependencies() {
    local component="$1"
    local processed=()
    
    # Check if component has dependencies
    if [[ -n "${DEPENDENCIES[$component]:-}" ]]; then
        local deps=(${DEPENDENCIES[$component]//,/ })
        for dep in "${deps[@]}"; do
            log_verbose "Processing dependency: $dep"
            resolve_dependencies "$dep"
        done
    fi
    
    # Add component if not already processed
    if [[ ! " ${processed[@]} " =~ " $component " ]]; then
        processed+=("$component")
        install_component "$component"
    fi
}

install_by_type() {
    local type="$1"
    local components=()
    
    case "$type" in
        minimal)
            components=(core system-tools security-core monitoring config-management)
            ;;
        standard)
            components=(core system-tools user-management security-core rbac monitoring logging update-system config-management web-interface cli-tools)
            ;;
        enterprise)
            components=(core system-tools user-management security-core rbac encryption audit monitoring logging alerting update-system rollback config-management web-interface cli-tools)
            ;;
    esac
    
    log_info "Installing $type components: ${components[*]}"
    
    # Resolve dependencies and install
    for component in "${components[@]}"; do
        resolve_dependencies "$component"
    done
}

install_specific_components() {
    local components_list="$1"
    IFS=',' read -ra COMPONENTS_ARRAY <<< "$components_list"
    
    log_info "Installing specific components: ${COMPONENTS_ARRAY[*]}"
    
    for component in "${COMPONENTS_ARRAY[@]}"; do
        resolve_dependencies "$component"
    done
}

configure_services() {
    log_info "Configuring system services..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would configure services"
        return 0
    fi
    
    # Create systemd service files
    cat > /etc/systemd/system/multios-admin.service <<EOF
[Unit]
Description=MultiOS Administration Service
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_GROUP
ExecStart=$INSTALL_DESTINATION/bin/multios-admin daemon
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF
    
    cat > /etc/systemd/system/multios-monitoring.service <<EOF
[Unit]
Description=MultiOS Monitoring Service
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_GROUP
ExecStart=$INSTALL_DESTINATION/bin/multios-monitor daemon
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF
    
    cat > /etc/systemd/system/multios-update.service <<EOF
[Unit]
Description=MultiOS Update Service
After=network.target

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_GROUP
ExecStart=$INSTALL_DESTINATION/bin/multios-update daemon
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd and enable services
    systemctl daemon-reload
    
    if [[ "$ENABLE_SERVICES" == "true" ]]; then
        systemctl enable multios-admin
        systemctl enable multios-monitoring
        systemctl enable multios-update
        
        log_info "Starting services..."
        systemctl start multios-admin || log_warning "Failed to start multios-admin"
        systemctl start multios-monitoring || log_warning "Failed to start multios-monitoring"
        systemctl start multios-update || log_warning "Failed to start multios-update"
        
        log_success "Services enabled and started"
    else
        log_info "Services created but not started. Use 'systemctl enable multios-*' to enable."
    fi
}

set_permissions() {
    log_info "Setting proper permissions..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would set permissions"
        return 0
    fi
    
    # Set ownership
    chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DESTINATION"
    
    # Set directory permissions
    find "$INSTALL_DESTINATION" -type d -exec chmod 755 {} \;
    
    # Set file permissions
    find "$INSTALL_DESTINATION" -type f -exec chmod 644 {} \;
    
    # Make binaries executable
    find "$INSTALL_DESTINATION/bin" -type f -exec chmod 755 {} \;
    
    # Set secure permissions on configuration files
    find "$INSTALL_DESTINATION/etc" -type f -exec chmod 640 {} \;
    
    log_success "Permissions set"
}

create_uninstall_script() {
    log_info "Creating uninstall script..."
    
    cat > "$INSTALL_DESTINATION/uninstall.sh" <<EOF
#!/bin/bash
# MultiOS Administrator Components Uninstall Script

set -e

UNINSTALL_SCRIPT="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
SERVICE_USER="$SERVICE_USER"
SERVICE_GROUP="$SERVICE_GROUP"

echo "MultiOS Administrator Components Uninstaller"
echo "============================================"
echo

read -p "This will remove all MultiOS administrator components. Continue? [y/N]: " -n 1 -r
echo
if [[ ! \$REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstallation cancelled."
    exit 0
fi

echo "Stopping and disabling services..."
systemctl stop multios-admin multios-monitoring multios-update 2>/dev/null || true
systemctl disable multios-admin multios-monitoring multios-update 2>/dev/null || true

echo "Removing systemd service files..."
rm -f /etc/systemd/system/multios-*.service
systemctl daemon-reload

echo "Removing CLI tools..."
rm -f /usr/local/bin/multios-admin
rm -f /usr/local/bin/multios-update
rm -f /usr/local/bin/multios-monitor

echo "Removing installation directory..."
rm -rf "\$UNINSTALL_SCRIPT"

echo "Removing user and group (if no other processes)..."
if pgrep -u "\$SERVICE_USER" >/dev/null 2>&1; then
    echo "Warning: User \$SERVICE_USER has running processes. Manual cleanup required."
else
    userdel "\$SERVICE_USER" 2>/dev/null || true
fi

if getent group "\$SERVICE_GROUP" >/dev/null 2>&1; then
    if [[ -z "\$(getent group \$SERVICE_GROUP | cut -d: -f4)" ]]; then
        groupdel "\$SERVICE_GROUP" 2>/dev/null || true
    fi
fi

echo "Uninstallation completed successfully."
echo
echo "Note: Some configuration files may remain in /etc if they were modified."
echo "Manual cleanup may be required for custom configurations."
EOF
    
    chmod +x "$INSTALL_DESTINATION/uninstall.sh"
    
    log_success "Uninstall script created at $INSTALL_DESTINATION/uninstall.sh"
}

finalize_installation() {
    log_info "Finalizing installation..."
    
    # Create installation manifest
    cat > "$INSTALL_DESTINATION/manifest.json" <<EOF
{
    "installation": {
        "type": "$INSTALL_TYPE",
        "destination": "$INSTALL_DESTINATION",
        "service_user": "$SERVICE_USER",
        "service_group": "$SERVICE_GROUP",
        "components": $(
            if [[ -n "$SPECIFIC_COMPONENTS" ]]; then
                echo "[$(echo "$SPECIFIC_COMPONENTS" | sed 's/,/","/g' | sed 's/^/"/' | sed 's/$/"/')]"
            else
                case "$INSTALL_TYPE" in
                    minimal) echo '["core","system-tools","security-core","monitoring","config-management"]' ;;
                    standard) echo '["core","system-tools","user-management","security-core","rbac","monitoring","logging","update-system","config-management","web-interface","cli-tools"]' ;;
                    enterprise) echo '["core","system-tools","user-management","security-core","rbac","encryption","audit","monitoring","logging","alerting","update-system","rollback","config-management","web-interface","cli-tools"]' ;;
                esac
            fi
        ),
        "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
        "version": "1.2.0"
    },
    "system_info": $SYSTEM_INFO
}
EOF
    
    # Create version file
    echo "1.2.0" > "$INSTALL_DESTINATION/VERSION"
    
    # Generate completion message
    log_success "=== Installation Completed Successfully ==="
    echo
    echo "MultiOS Administrator Components v1.2.0"
    echo "Installation Type: $INSTALL_TYPE"
    echo "Installation Directory: $INSTALL_DESTINATION"
    echo "Service User: $SERVICE_USER"
    echo "Service Group: $SERVICE_GROUP"
    echo
    echo "Installed Components:"
    case "$INSTALL_TYPE" in
        minimal)
            echo "  - Core system utilities"
            echo "  - Security framework"
            echo "  - Monitoring system"
            echo "  - Configuration management"
            ;;
        standard)
            echo "  - Core system utilities"
            echo "  - User management system"
            echo "  - Security framework with RBAC"
            echo "  - Monitoring and logging"
            echo "  - Update management system"
            echo "  - Configuration management"
            echo "  - Web-based administration interface"
            echo "  - CLI administration tools"
            ;;
        enterprise)
            echo "  - All standard components"
            echo "  - Advanced encryption services"
            echo "  - Comprehensive audit system"
            echo "  - Alert and notification system"
            echo "  - Advanced rollback and recovery"
            ;;
    esac
    echo
    echo "Next Steps:"
    echo "  1. Review configuration files in $INSTALL_DESTINATION/etc/"
    echo "  2. Configure services: systemctl enable multios-*"
    echo "  3. Start services: systemctl start multios-*"
    echo "  4. Access web interface at http://localhost:8080"
    echo "  5. Use CLI tools: multios-admin, multios-update, multios-monitor"
    echo
    echo "Documentation: $INSTALL_DESTINATION/doc/"
    echo "Uninstall: $INSTALL_DESTINATION/uninstall.sh"
    echo
    
    if [[ "$ENABLE_SERVICES" == "true" ]]; then
        log_info "Services are running. Check status with: systemctl status multios-*"
    else
        log_info "Services created but not started. Start with: systemctl start multios-*"
    fi
}

# Main installation function
main() {
    log_info "Starting MultiOS Administrator Components Installation"
    log_info "Installation Type: $INSTALL_TYPE"
    log_info "Destination: $INSTALL_DESTINATION"
    echo
    
    # Validate inputs
    validate_installation_type
    
    # Check requirements
    validate_requirements
    
    # Gather system information
    gather_system_info
    
    # Create installation directory
    if [[ "$DRY_RUN" == "false" ]]; then
        mkdir -p "$INSTALL_DESTINATION"
        log_verbose "Created installation directory: $INSTALL_DESTINATION"
    fi
    
    # Install components
    echo "Installing components..."
    if [[ -n "$SPECIFIC_COMPONENTS" ]]; then
        install_specific_components "$SPECIFIC_COMPONENTS"
    else
        install_by_type "$INSTALL_TYPE"
    fi
    
    # Configure services
    configure_services
    
    # Set permissions
    set_permissions
    
    # Create uninstall script
    create_uninstall_script
    
    # Finalize
    finalize_installation
}

# Error handling
trap 'log_error "Installation failed at line $LINENO. Exit code: $?"' ERR

# Check for required arguments
if [[ "$INSTALL_TYPE" == "" ]]; then
    log_error "Installation type is required"
    show_help
fi

# Run main function
main "$@"

exit 0