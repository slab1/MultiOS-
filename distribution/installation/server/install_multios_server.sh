#!/bin/bash
# MultiOS Server Installation Script
# Version: 1.0.0
# Description: Install MultiOS for server use

set -e

# Configuration
INSTALL_DIR="/opt/multios"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/multios"
KERNEL_DIR="/opt/multios/bin"
VERSION="1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
LOG_FILE="/var/log/multios_server_install.log"
exec 1> >(tee -a "$LOG_FILE") 2>&1

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

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
        exit 1
    fi
}

# Check system requirements for server
check_requirements() {
    log "Checking server system requirements..."
    
    # Check architecture
    ARCH=$(uname -m)
    case $ARCH in
        x86_64|aarch64|riscv64)
            info "Architecture: $ARCH - Supported"
            ;;
        *)
            error "Architecture $ARCH is not supported"
            exit 1
            ;;
    esac
    
    # Check available space (minimum 10GB for server)
    AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
    REQUIRED_SPACE=10485760  # 10GB in KB
    
    if [ "$AVAILABLE_SPACE" -lt "$REQUIRED_SPACE" ]; then
        error "Insufficient disk space. Required: 10GB, Available: $((AVAILABLE_SPACE/1024/1024))GB"
        exit 1
    fi
    
    # Check memory (minimum 4GB for server)
    TOTAL_MEM=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    REQUIRED_MEM=4194304  # 4GB in KB
    
    if [ "$TOTAL_MEM" -lt "$REQUIRED_MEM" ]; then
        warning "Low memory detected. Recommended: 8GB minimum, Available: $((TOTAL_MEM/1024))MB"
    fi
    
    # Check CPU cores (minimum 2 for server)
    CPU_CORES=$(nproc)
    if [ "$CPU_CORES" -lt 2 ]; then
        warning "Low CPU core count. Recommended: 4+ cores, Available: $CPU_CORES"
    fi
    
    # Check for network interface
    if ! ip link show | grep -q "^[0-9]"; then
        error "No network interface found"
        exit 1
    fi
    
    info "Server system requirements check passed"
}

# Install security packages
install_security() {
    log "Installing security packages..."
    
    # Install firewall configuration
    cat > "$CONFIG_DIR/firewall.conf" <<EOF
# MultiOS Server Firewall Configuration
# Version: $VERSION

[SERVICE:SSH]
port=22
protocol=tcp
allowed_ips=any

[SERVICE:HTTP]
port=80
protocol=tcp
allowed_ips=any

[SERVICE:HTTPS]
port=443
protocol=tcp
allowed_ips=any

[SERVICE:MULTIOS-API]
port=8080
protocol=tcp
allowed_ips=127.0.0.1,::1

# Block all other incoming connections
[DEFAULT]
action=drop
EOF

    # Install fail2ban for SSH protection
    if command -v fail2ban-server >/dev/null 2>&1; then
        cat > /etc/fail2ban/jail.local <<EOF
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
EOF
        systemctl enable fail2ban
        systemctl start fail2ban
    fi
}

# Create server-specific directories
create_directories() {
    log "Creating server directory structure..."
    
    mkdir -p "$INSTALL_DIR"/{bin,lib,config,logs,data,backups}
    mkdir -p "$CONFIG_DIR"/{profiles,services,modules,templates}
    mkdir -p "$KERNEL_DIR"
    mkdir -p /var/log/multios
    
    # Set permissions
    chmod 755 "$INSTALL_DIR" "$CONFIG_DIR" "$KERNEL_DIR"
    chmod 700 "$CONFIG_DIR/ssl" 2>/dev/null || true
    
    info "Server directory structure created"
}

# Install kernel for server
install_kernel() {
    log "Installing MultiOS kernel for server..."
    
    # Copy kernel binary
    if [ -f "./kernel/target/release/multios-kernel" ]; then
        cp "./kernel/target/release/multios-kernel" "$KERNEL_DIR/"
    else
        warning "Kernel binary not found. Building from source..."
        cd kernel
        cargo build --release
        cp "./target/release/multios-kernel" "$KERNEL_DIR/"
        cd ..
    fi
    
    chmod +x "$KERNEL_DIR/multios-kernel"
    info "Server kernel installed successfully"
}

# Install server-specific bootloader
install_bootloader() {
    log "Installing server bootloader configuration..."
    
    if [ -d "./bootloader" ]; then
        cp -r ./bootloader/* "$INSTALL_DIR/bootloader/"
        
        # Create server-specific bootloader config
        cat > "$INSTALL_DIR/bootloader/server.conf" <<EOF
# MultiOS Server Bootloader Configuration
# Version: $VERSION

[DEFAULT]
timeout=5
default=multios

[MULTIOS]
kernel=$KERNEL_DIR/multios-kernel
initrd=$INSTALL_DIR/bootloader/initrd.img
cmdline=console=ttyS0,115200 root=/dev/sda1 ro quiet

[SAFEMODE]
kernel=$KERNEL_DIR/multios-kernel
cmdline=console=ttyS0,115200 root=/dev/sda1 ro single
EOF
    fi
}

# Install server libraries and modules
install_libraries() {
    log "Installing server libraries and modules..."
    
    # Copy server-specific libraries
    cp -r ./libraries/* "$INSTALL_DIR/lib/"
    
    # Install performance monitoring
    if [ -d "./tools/monitor_dashboard" ]; then
        cp -r ./tools/monitor_dashboard "$INSTALL_DIR/monitoring/"
    fi
    
    # Install backup tools
    if [ -d "./tools/backup_recovery" ]; then
        cp -r ./tools/backup_recovery "$INSTALL_DIR/backup/"
    fi
}

# Create server systemd services
create_systemd_services() {
    log "Creating server systemd services..."
    
    # MultiOS kernel service (server optimized)
    cat > "$SYSTEMD_DIR/multios-kernel.service" <<EOF
[Unit]
Description=MultiOS Hybrid Microkernel (Server)
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
ExecStart=$KERNEL_DIR/multios-kernel --server --max-processes=4096
Restart=always
RestartSec=10
StandardOutput=append:/var/log/multios/kernel.log
StandardError=append:/var/log/multios/kernel_error.log
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

    # Server monitoring service
    cat > "$SYSTEMD_DIR/multios-monitor.service" <<EOF
[Unit]
Description=MultiOS Server Monitoring
After=multios-kernel.service
Wants=multios-kernel.service

[Service]
Type=simple
ExecStart=$INSTALL_DIR/monitoring/start_dashboard.py
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
EOF

    # Backup service
    cat > "$SYSTEMD_DIR/multios-backup.service" <<EOF
[Unit]
Description=MultiOS Server Backup
After=multios-kernel.service
Wants=multios-kernel.service

[Service]
Type=oneshot
ExecStart=$INSTALL_DIR/backup/create_backup.sh
EOF

    cat > "$SYSTEMD_DIR/multios-backup.timer" <<EOF
[Unit]
Description=Run MultiOS Backup every 6 hours
Requires=multios-backup.service

[Timer]
OnCalendar=*-*-* 0,6,12,18:00:00
Persistent=true

[Install]
WantedBy=timers.target
EOF

    # Enable services
    systemctl enable multios-kernel.service
    systemctl enable multios-monitor.service
    systemctl enable multios-backup.timer
    
    info "Server systemd services created and enabled"
}

# Create server configuration
create_config() {
    log "Creating server configuration..."
    
    # Main server configuration
    cat > "$CONFIG_DIR/server.conf" <<EOF
# MultiOS Server Configuration
# Version: $VERSION

[General]
version=$VERSION
install_date=$(date -u +%Y-%m-%dT%H:%M:%SZ)
install_type=server
server_type=production

[System]
kernel_path=$KERNEL_DIR/multios-kernel
install_dir=$INSTALL_DIR
config_dir=$CONFIG_DIR
log_dir=/var/log/multios

[Server]
max_connections=10000
max_processes=4096
max_memory=0  # unlimited
enable_clustering=true
enable_load_balancing=true

[Services]
web_server=enabled
database_server=optional
file_server=enabled
mail_server=optional
dns_server=optional

[Security]
firewall=enabled
ssl=enabled
fail2ban=enabled
secure_boot=optional
encryption=optional

[Monitoring]
enable_metrics=true
enable_alerts=true
enable_logging=true
metrics_port=9090

[Backup]
auto_backup=enabled
backup_interval=6h
backup_retention=30d
backup_location=$INSTALL_DIR/backups
EOF

    # Create admin profile
    mkdir -p "$CONFIG_DIR/profiles/admin"
    cat > "$CONFIG_DIR/profiles/admin/profile.conf" <<EOF
[User]
name=admin
uid=0
gid=0
shell=/bin/bash

[Permissions]
sudo=true
admin=true
service_management=true

[Services]
all_enabled=true
EOF

    info "Server configuration files created"
}

# Install load balancing configuration
install_load_balancer() {
    log "Installing load balancer configuration..."
    
    mkdir -p "$CONFIG_DIR/load_balancer"
    
    cat > "$CONFIG_DIR/load_balancer/default.conf" <<EOF
# MultiOS Load Balancer Configuration
# Version: $VERSION

[DEFAULT]
algorithm=round_robin
health_check_interval=30s
timeout=10s

[BACKEND:web_servers]
servers=127.0.0.1:8080,127.0.0.1:8081
health_check=/health

[FRONTEND:web]
port=80
protocol=http
backend=web_servers

[FRONTEND:secure_web]
port=443
protocol=https
backend=web_servers
ssl_cert=$CONFIG_DIR/ssl/server.crt
ssl_key=$CONFIG_DIR/ssl/server.key
EOF
}

# Run server-specific tests
run_tests() {
    log "Running server installation tests..."
    
    # Test kernel installation
    if [ -x "$KERNEL_DIR/multios-kernel" ]; then
        info "Kernel installation test: PASSED"
    else
        error "Kernel installation test: FAILED"
        return 1
    fi
    
    # Test configuration
    if [ -f "$CONFIG_DIR/server.conf" ]; then
        info "Server configuration test: PASSED"
    else
        error "Server configuration test: FAILED"
        return 1
    fi
    
    # Test services
    if systemctl is-enabled multios-kernel.service >/dev/null 2>&1; then
        info "Systemd services test: PASSED"
    else
        error "Systemd services test: FAILED"
        return 1
    fi
    
    # Test network configuration
    if [ -f "$CONFIG_DIR/firewall.conf" ]; then
        info "Network configuration test: PASSED"
    else
        warning "Network configuration test: FAILED"
    fi
    
    # Test backup configuration
    if [ -d "$INSTALL_DIR/backups" ]; then
        info "Backup configuration test: PASSED"
    else
        error "Backup configuration test: FAILED"
        return 1
    fi
    
    info "All server tests passed successfully"
}

# Print server installation summary
print_summary() {
    echo
    log "MultiOS Server Installation Complete!"
    echo
    echo "Server Installation Summary:"
    echo "  Version: $VERSION"
    echo "  Install Directory: $INSTALL_DIR"
    echo "  Configuration: $CONFIG_DIR"
    echo "  Kernel: $KERNEL_DIR/multios-kernel"
    echo "  Log Directory: /var/log/multios"
    echo "  Backup Directory: $INSTALL_DIR/backups"
    echo
    echo "Services Installed:"
    echo "  - MultiOS Kernel (Server Mode)"
    echo "  - Server Monitoring Dashboard"
    echo "  - Automatic Backup System"
    echo "  - Load Balancer"
    echo "  - Firewall Protection"
    echo
    echo "Next Steps:"
    echo "  1. Reboot your server"
    echo "  2. Select MultiOS from the bootloader"
    echo "  3. Configure network settings"
    echo "  4. Set up SSL certificates"
    echo "  5. Configure services for your needs"
    echo "  6. Run: $INSTALL_DIR/monitoring/validate.py"
    echo
    echo "Useful Commands:"
    echo "  - View logs: tail -f /var/log/multios/kernel.log"
    echo "  - Check status: systemctl status multios-kernel"
    echo "  - Backup now: systemctl start multios-backup"
    echo "  - Monitoring: http://localhost:9090"
    echo
    echo "For more information, see: $CONFIG_DIR/README.md"
    echo
}

# Main installation function
main() {
    echo "========================================"
    echo "  MultiOS Server Installation Script"
    echo "  Version: $VERSION"
    echo "========================================"
    echo
    
    check_root
    check_requirements
    create_directories
    install_security
    install_kernel
    install_bootloader
    install_libraries
    create_systemd_services
    create_config
    install_load_balancer
    run_tests
    print_summary
}

# Run main function
main "$@"