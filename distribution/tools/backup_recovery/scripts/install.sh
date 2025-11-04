#!/bin/bash
# MultiOS Backup System Installation Script
# Complete system setup and configuration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/usr/local"
CONFIG_DIR="/etc/multios/backup"
DATA_DIR="/var/lib/multios/backup"
LOG_DIR="/var/log/multios/backup"
BIN_DIR="/usr/local/bin"
SYSTEMD_DIR="/etc/systemd/system"

# Functions
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

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root"
        exit 1
    fi
}

check_dependencies() {
    log_info "Checking system dependencies..."
    
    local missing_deps=()
    
    # Check for required commands
    for cmd in rustc cargo python3 pip3 systemd systemctl; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Please install missing dependencies and run this script again"
        exit 1
    fi
    
    log_success "All dependencies found"
}

install_rust_dependencies() {
    log_info "Installing Rust dependencies..."
    
    cd src
    
    # Update Cargo.toml with system paths if needed
    log_info "Building backup system..."
    cargo build --release
    
    if [[ $? -eq 0 ]]; then
        log_success "Rust backup system built successfully"
        
        # Install binary
        cp target/release/multios-backup $BIN_DIR/
        chmod +x $BIN_DIR/multios-backup
        log_success "Installed multios-backup to $BIN_DIR/"
    else
        log_error "Failed to build backup system"
        exit 1
    fi
    
    cd ..
}

install_python_dependencies() {
    log_info "Installing Python dependencies..."
    
    cd python
    
    # Create virtual environment if requested
    if [[ "${VIRTUAL_ENV:-}" == "" ]]; then
        log_info "Creating virtual environment..."
        python3 -m venv venv
        source venv/bin/activate
        log_info "Virtual environment activated"
    fi
    
    # Install requirements
    pip3 install -r requirements.txt
    
    if [[ $? -eq 0 ]]; then
        log_success "Python dependencies installed successfully"
    else
        log_error "Failed to install Python dependencies"
        exit 1
    fi
    
    cd ..
}

setup_directories() {
    log_info "Setting up directory structure..."
    
    local directories=(
        "$CONFIG_DIR"
        "$DATA_DIR"
        "$LOG_DIR"
        "$DATA_DIR/backups"
        "$DATA_DIR/temp"
        "$DATA_DIR/media"
        "$CONFIG_DIR/labs"
    )
    
    for dir in "${directories[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            log_info "Created directory: $dir"
        fi
    done
    
    # Set permissions
    chown -R root:root "$DATA_DIR"
    chmod 755 "$DATA_DIR"
    chmod 755 "$LOG_DIR"
    
    log_success "Directory structure created"
}

setup_config() {
    log_info "Setting up configuration..."
    
    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        cp config/config.toml "$CONFIG_DIR/"
        log_info "Copied default configuration"
    else
        log_warning "Configuration file already exists, skipping"
    fi
    
    # Set secure permissions on config
    chmod 600 "$CONFIG_DIR/config.toml"
    
    log_success "Configuration setup complete"
}

setup_systemd_services() {
    log_info "Setting up systemd services..."
    
    # Backup service
    cat > "$SYSTEMD_DIR/multios-backup.service" << EOF
[Unit]
Description=MultiOS Backup System
After=network.target
Requires=network.target

[Service]
Type=forking
ExecStart=$BIN_DIR/multios-backup daemon start
ExecStop=$BIN_DIR/multios-backup daemon stop
ExecReload=$BIN_DIR/multios-backup daemon reload
Restart=always
RestartSec=10
User=root
WorkingDirectory=$DATA_DIR

[Install]
WantedBy=multi-user.target
EOF

    # Scheduler service
    cat > "$SYSTEMD_DIR/multios-backup-scheduler.service" << EOF
[Unit]
Description=MultiOS Backup Scheduler
After=multios-backup.service
Requires=multios-backup.service

[Service]
Type=simple
ExecStart=$BIN_DIR/multios-backup scheduler start
Restart=always
RestartSec=10
User=root

[Install]
WantedBy=multi-user.target
EOF

    # Web console service
    cat > "$SYSTEMD_DIR/multios-backup-web.service" << EOF
[Unit]
Description=MultiOS Backup Web Console
After=network.target
Requires=network.target

[Service]
Type=simple
ExecStart=/usr/bin/python3 $BIN_DIR/web_console.py
Restart=always
RestartSec=10
User=root
Environment=PYTHONPATH=$BIN_DIR
WorkingDirectory=$DATA_DIR

[Install]
WantedBy=multi-user.target
EOF

    # Enable services
    systemctl daemon-reload
    systemctl enable multios-backup.service
    systemctl enable multios-backup-scheduler.service
    systemctl enable multios-backup-web.service
    
    log_success "Systemd services created and enabled"
}

setup_cron_jobs() {
    log_info "Setting up cron jobs..."
    
    # Add cron job for maintenance
    (crontab -l 2>/dev/null; echo "0 2 * * * $BIN_DIR/multios-backup maintenance") | crontab -
    
    # Add cron job for log rotation
    (crontab -l 2>/dev/null; echo "0 3 * * 0 $BIN_DIR/multios-backup rotate-logs") | crontab -
    
    log_success "Cron jobs configured"
}

setup_firewall() {
    log_info "Configuring firewall rules..."
    
    if command -v ufw &> /dev/null; then
        # Allow web console port
        ufw allow 8080/tcp comment "MultiOS Backup Web Console"
        log_info "UFW firewall rules added"
    elif command -v firewall-cmd &> /dev/null; then
        # Allow web console port
        firewall-cmd --permanent --add-port=8080/tcp
        firewall-cmd --reload
        log_info "Firewalld firewall rules added"
    else
        log_warning "No firewall detected or configured"
    fi
}

setup_selinux() {
    if [[ -f /etc/selinux/config ]] && command -v setsebool &> /dev/null; then
        log_info "Configuring SELinux policies..."
        
        # Allow backup operations
        setsebool -P backup_journal 1
        setsebool -P logrotate_read_generic_config 1
        
        log_info "SELinux policies configured"
    fi
}

create_test_backups() {
    log_info "Creating test backup configuration..."
    
    # Create a test lab profile
    cat > "$CONFIG_DIR/labs/test-lab.yaml" << EOF
id: test-lab
name: Test Lab Environment
description: Test environment for demonstration purposes
default_sources:
  - /tmp/test-data
  - /var/log/test
default_retention: "7 days"
schedule_settings:
  cron_expression: "0 2 * * *"
  backup_type: "incremental"
  enabled: true
custom_config:
  compression: "zstd"
  encryption: false
  verify_integrity: true
EOF

    # Create test data directory
    mkdir -p /tmp/test-data
    echo "Test backup data" > /tmp/test-data/test-file.txt
    
    log_success "Test backup configuration created"
}

display_next_steps() {
    echo
    log_success "=== Installation Complete ==="
    echo
    echo "MultiOS Backup System has been installed successfully!"
    echo
    echo "Next steps:"
    echo "1. Start the services:"
    echo "   systemctl start multios-backup"
    echo "   systemctl start multios-backup-scheduler"
    echo "   systemctl start multios-backup-web"
    echo
    echo "2. Check status:"
    echo "   systemctl status multios-backup"
    echo
    echo "3. Access web console:"
    echo "   http://localhost:8080"
    echo
    echo "4. Test backup system:"
    echo "   multios-backup create --type full --source /tmp/test-data --name 'Test Backup'"
    echo
    echo "Configuration files:"
    echo "   - Main config: $CONFIG_DIR/config.toml"
    echo "   - Lab profiles: $CONFIG_DIR/labs/"
    echo "   - Backup data: $DATA_DIR/"
    echo "   - Logs: $LOG_DIR/"
    echo
    echo "Documentation:"
    echo "   - User manual: $CONFIG_DIR/README.md"
    echo "   - API docs: $CONFIG_DIR/API.md"
    echo "   - Lab guide: $CONFIG_DIR/LAB_GUIDE.md"
    echo
}

main() {
    echo "=========================================="
    echo "  MultiOS Backup System Installer"
    echo "=========================================="
    echo
    
    check_root
    check_dependencies
    
    install_rust_dependencies
    install_python_dependencies
    setup_directories
    setup_config
    setup_systemd_services
    setup_cron_jobs
    setup_firewall
    setup_selinux
    create_test_backups
    
    display_next_steps
}

# Handle command line arguments
case "${1:-install}" in
    --help|-h)
        echo "MultiOS Backup System Installer"
        echo
        echo "Usage: $0 [install]"
        echo
        echo "This script installs and configures the MultiOS Backup System"
        echo
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac