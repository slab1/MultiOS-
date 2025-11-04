#!/bin/bash

# MultiOS Monitoring Infrastructure Deployment Script
# This script automates the installation and configuration of the monitoring system

set -e  # Exit on any error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
INSTALL_DIR="/opt/multios-monitoring"
SERVICE_USER="multios"
LOG_DIR="/var/log/multios-monitoring"
DATA_DIR="/var/lib/multios-monitoring"
CONFIG_DIR="/etc/multios-monitoring"
ENVIRONMENT="production"
PYTHON_VERSION="3.8"
NODE_VERSION="14"
QUICK_START=false
FULL_INSTALL=false
FORCE_INSTALL=false
VERBOSE=false
DRY_RUN=false

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --service-user)
                SERVICE_USER="$2"
                shift 2
                ;;
            --environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            --python-version)
                PYTHON_VERSION="$2"
                shift 2
                ;;
            --quick-start)
                QUICK_START=true
                shift
                ;;
            --full-install)
                FULL_INSTALL=true
                shift
                ;;
            --force)
                FORCE_INSTALL=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Show help information
show_help() {
    cat << EOF
MultiOS Monitoring Infrastructure Deployment Script

Usage: $0 [OPTIONS]

Options:
    --install-dir DIR         Installation directory (default: /opt/multios-monitoring)
    --service-user USER       System user for the service (default: multios)
    --environment ENV         Environment type: development, staging, production, educational
    --python-version VERSION  Python version to use (default: 3.8)
    --quick-start            Quick installation with defaults
    --full-install           Full installation with all dependencies
    --force                  Force installation even if already installed
    --verbose                Verbose output
    --dry-run                Show what would be done without executing
    --help                   Show this help message

Examples:
    $0 --quick-start
    $0 --full-install --environment production
    $0 --install-dir /opt/monitoring --service-user monitor
    $0 --dry-run --verbose

EOF
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_warning "Running as root. This is not recommended for security reasons."
        if [[ "$FORCE_INSTALL" != true ]]; then
            read -p "Continue anyway? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
        fi
    fi
}

# Detect operating system
detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$NAME
        VERSION=$VERSION_ID
        DISTRO=$(echo $ID | tr '[:upper:]' '[:lower:]')
    else
        print_error "Cannot detect operating system"
        exit 1
    fi
    
    print_info "Detected OS: $OS $VERSION"
    
    # Check if supported
    case $DISTRO in
        ubuntu|debian|centos|rhel|fedora|opensuse)
            print_success "Supported distribution detected: $DISTRO"
            ;;
        *)
            print_warning "Unsupported distribution: $DISTRO. Proceeding anyway..."
            ;;
    esac
}

# Install system dependencies
install_system_deps() {
    print_info "Installing system dependencies..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would install system dependencies"
        return
    fi
    
    case $DISTRO in
        ubuntu|debian)
            apt-get update
            apt-get install -y \
                python3.$PYTHON_VERSION python3.$PYTHON_VERSION-pip \
                python3.$PYTHON_VERSION-venv python3.$PYTHON_VERSION-dev \
                nodejs npm nginx postgresql redis-server \
                build-essential libpq-dev libffi-dev libssl-dev \
                curl wget git unzip htop iftop nethogs \
                python3-psutil python3-numpy python3-pandas \
                supervisor rsyslog logrotate \
                fail2ban ufw
            ;;
        centos|rhel|fedora)
            yum groupinstall -y "Development Tools"
            yum install -y \
                python$PYTHON_VERSION python$PYTHON_VERSION-pip \
                python$PYTHON_VERSION-devel nodejs npm \
                nginx postgresql redis \
                postgresql-devel libffi-devel openssl-devel \
                curl wget git unzip htop iftop nethogs \
                python3-psutil python3-numpy python3-pandas \
                supervisor rsyslog logrotate \
                fail2ban firewalld
            ;;
        opensuse)
            zypper install -y \
                python3 python3-pip python3-devel nodejs npm \
                nginx postgresql redis \
                postgresql-devel libffi-devel openssl-devel \
                curl wget git unzip htop iftop nethogs \
                python3-psutil python3-numpy python3-pandas \
                supervisor rsyslog logrotate \
                fail2ban firewalld
            ;;
    esac
    
    print_success "System dependencies installed"
}

# Install Python dependencies
install_python_deps() {
    print_info "Installing Python dependencies..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would install Python dependencies"
        return
    fi
    
    # Create virtual environment
    python3.$PYTHON_VERSION -m venv $INSTALL_DIR/venv
    
    # Activate virtual environment
    source $INSTALL_DIR/venv/bin/activate
    
    # Upgrade pip
    pip install --upgrade pip wheel setuptools
    
    # Install dependencies
    pip install -r $PWD/requirements.txt
    
    # Deactivate virtual environment
    deactivate
    
    print_success "Python dependencies installed"
}

# Install Node.js dependencies
install_nodejs_deps() {
    print_info "Installing Node.js dependencies..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would install Node.js dependencies"
        return
    fi
    
    cd dashboard
    
    # Install dependencies
    npm install
    
    # Build assets
    npm run build
    
    cd ..
    
    print_success "Node.js dependencies installed"
}

# Create system user
create_service_user() {
    print_info "Creating service user: $SERVICE_USER"
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would create service user: $SERVICE_USER"
        return
    fi
    
    # Create user if it doesn't exist
    if ! id "$SERVICE_USER" &>/dev/null; then
        useradd --system --shell /bin/false --home $INSTALL_DIR \
                --create-home --user-group $SERVICE_USER
        print_success "Created user: $SERVICE_USER"
    else
        print_info "User $SERVICE_USER already exists"
    fi
}

# Create directory structure
create_directories() {
    print_info "Creating directory structure..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would create directories:"
        print_info "  - $INSTALL_DIR"
        print_info "  - $LOG_DIR"
        print_info "  - $DATA_DIR"
        print_info "  - $CONFIG_DIR"
        return
    fi
    
    # Create directories
    mkdir -p $INSTALL_DIR
    mkdir -p $LOG_DIR
    mkdir -p $DATA_DIR
    mkdir -p $CONFIG_DIR
    mkdir -p $DATA_DIR/{logs,metrics,backups}
    mkdir -p $CONFIG_DIR/{environments,rules,templates}
    
    # Set ownership
    chown -R $SERVICE_USER:$SERVICE_USER $INSTALL_DIR
    chown -R $SERVICE_USER:$SERVICE_USER $LOG_DIR
    chown -R $SERVICE_USER:$SERVICE_USER $DATA_DIR
    chown -R $SERVICE_USER:$SERVICE_USER $CONFIG_DIR
    
    print_success "Directory structure created"
}

# Copy application files
copy_application_files() {
    print_info "Copying application files..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would copy application files to $INSTALL_DIR"
        return
    fi
    
    # Copy all files except deployment scripts
    rsync -av --exclude='deploy.sh' --exclude='*.md' --exclude='*.txt' \
          --exclude='.git' . $INSTALL_DIR/
    
    # Set ownership
    chown -R $SERVICE_USER:$SERVICE_USER $INSTALL_DIR
    
    print_success "Application files copied"
}

# Setup configuration
setup_configuration() {
    print_info "Setting up configuration..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup configuration for environment: $ENVIRONMENT"
        return
    fi
    
    # Copy environment template
    if [[ -f config/environments/$ENVIRONMENT.env.template ]]; then
        cp config/environments/$ENVIRONMENT.env.template \
           $CONFIG_DIR/$ENVIRONMENT.env
    else
        cp config/environments/production.env.template \
           $CONFIG_DIR/$ENVIRONMENT.env
    fi
    
    # Generate random keys
    SECRET_KEY=$(python3 -c "import secrets; print(secrets.token_hex(32))")
    JWT_SECRET=$(python3 -c "import secrets; print(secrets.token_hex(32))")
    
    # Update configuration with generated values
    sed -i "s/your_secret_key_here/$SECRET_KEY/g" $CONFIG_DIR/$ENVIRONMENT.env
    sed -i "s/your_jwt_secret_here/$JWT_SECRET/g" $CONFIG_DIR/$ENVIRONMENT.env
    
    # Update paths in configuration
    sed -i "s|/opt/multios-monitoring|$INSTALL_DIR|g" $CONFIG_DIR/$ENVIRONMENT.env
    sed -i "s|/var/log/multios-monitoring|$LOG_DIR|g" $CONFIG_DIR/$ENVIRONMENT.env
    sed -i "s|/var/lib/multios-monitoring|$DATA_DIR|g" $CONFIG_DIR/$ENVIRONMENT.env
    
    # Set ownership
    chown $SERVICE_USER:$SERVICE_USER $CONFIG_DIR/$ENVIRONMENT.env
    chmod 600 $CONFIG_DIR/$ENVIRONMENT.env
    
    print_success "Configuration setup completed"
}

# Setup database
setup_database() {
    print_info "Setting up database..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup database"
        return
    fi
    
    # Start PostgreSQL service
    systemctl start postgresql
    systemctl enable postgresql
    
    # Create database and user
    sudo -u postgres psql << EOF
CREATE DATABASE multios_monitoring;
CREATE USER monitor_user WITH ENCRYPTED PASSWORD 'monitor_secure_password';
GRANT ALL PRIVILEGES ON DATABASE multios_monitoring TO monitor_user;
\q
EOF
    
    # Run database migrations
    source $INSTALL_DIR/venv/bin/activate
    cd $INSTALL_DIR
    python -c "
import os
os.environ['DATABASE_URL'] = 'postgresql://monitor_user:monitor_secure_password@localhost/multios_monitoring'
from backend.server import db, create_app
app = create_app()
with app.app_context():
    db.create_all()
"
    deactivate
    
    print_success "Database setup completed"
}

# Setup Redis
setup_redis() {
    print_info "Setting up Redis..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup Redis"
        return
    fi
    
    # Start Redis service
    systemctl start redis
    systemctl enable redis
    
    # Configure Redis
    if [[ ! -f /etc/redis/redis.conf.backup ]]; then
        cp /etc/redis/redis.conf /etc/redis/redis.conf.backup
    fi
    
    # Configure for monitoring
    echo "maxmemory 256mb" >> /etc/redis/redis.conf
    echo "maxmemory-policy allkeys-lru" >> /etc/redis/redis.conf
    echo "save 900 1" >> /etc/redis/redis.conf
    echo "save 300 10" >> /etc/redis/redis.conf
    echo "save 60 10000" >> /etc/redis/redis.conf
    
    systemctl restart redis
    
    print_success "Redis setup completed"
}

# Setup systemd service
setup_systemd_service() {
    print_info "Setting up systemd service..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup systemd service"
        return
    fi
    
    # Create systemd service file
    cat > /etc/systemd/system/multios-monitoring.service << EOF
[Unit]
Description=MultiOS Monitoring Infrastructure
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=forking
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$INSTALL_DIR
Environment=PATH=$INSTALL_DIR/venv/bin
ExecStart=$INSTALL_DIR/venv/bin/python backend/server.py --daemon
ExecStop=/bin/kill -TERM \$MAINPID
ExecReload=/bin/kill -HUP \$MAINPID
Restart=on-failure
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=multios-monitoring

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd
    systemctl daemon-reload
    systemctl enable multios-monitoring
    
    print_success "Systemd service setup completed"
}

# Setup Nginx configuration
setup_nginx() {
    print_info "Setting up Nginx configuration..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup Nginx configuration"
        return
    fi
    
    # Create Nginx configuration
    cat > /etc/nginx/sites-available/multios-monitoring << EOF
server {
    listen 80;
    server_name _;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    
    # Static files
    location /static/ {
        alias $INSTALL_DIR/dashboard/assets/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
    
    # Dashboard
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Timeout settings
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # API endpoints
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        
        # Rate limiting
        limit_req zone=api burst=20 nodelay;
    }
}
EOF
    
    # Enable site
    ln -sf /etc/nginx/sites-available/multios-monitoring /etc/nginx/sites-enabled/
    
    # Remove default site
    rm -f /etc/nginx/sites-enabled/default
    
    # Test configuration
    nginx -t
    
    # Restart Nginx
    systemctl restart nginx
    systemctl enable nginx
    
    print_success "Nginx configuration completed"
}

# Setup log rotation
setup_log_rotation() {
    print_info "Setting up log rotation..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup log rotation"
        return
    fi
    
    # Create logrotate configuration
    cat > /etc/logrotate.d/multios-monitoring << EOF
$LOG_DIR/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 $SERVICE_USER $SERVICE_USER
    postrotate
        systemctl reload multios-monitoring
    endscript
}
EOF
    
    print_success "Log rotation setup completed"
}

# Setup fail2ban
setup_fail2ban() {
    print_info "Setting up fail2ban..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup fail2ban"
        return
    fi
    
    # Create fail2ban jail
    cat > /etc/fail2ban/jail.d/multios-monitoring.conf << EOF
[multios-monitoring]
enabled = true
port = 80,443,8080
filter = multios-monitoring
logpath = $LOG_DIR/access.log
maxretry = 5
bantime = 3600
findtime = 600
EOF
    
    # Create filter
    cat > /etc/fail2ban/filter.d/multios-monitoring.conf << EOF
[Definition]
failregex = ^<HOST> .* "(GET|POST) .* HTTP/.*" (4|5)\d\d .*$
ignoreregex =
EOF
    
    # Restart fail2ban
    systemctl restart fail2ban
    
    print_success "Fail2ban setup completed"
}

# Setup firewall
setup_firewall() {
    print_info "Setting up firewall..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would setup firewall"
        return
    fi
    
    case $DISTRO in
        ubuntu|debian)
            # UFW
            ufw --force reset
            ufw default deny incoming
            ufw default allow outgoing
            ufw allow ssh
            ufw allow 80/tcp
            ufw allow 443/tcp
            ufw allow 8080/tcp
            ufw enable
            ;;
        centos|rhel|fedora)
            # Firewalld
            systemctl start firewalld
            systemctl enable firewalld
            firewall-cmd --permanent --add-service=ssh
            firewall-cmd --permanent --add-service=http
            firewall-cmd --permanent --add-service=https
            firewall-cmd --permanent --add-port=8080/tcp
            firewall-cmd --reload
            ;;
    esac
    
    print_success "Firewall setup completed"
}

# Run health checks
run_health_checks() {
    print_info "Running health checks..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would run health checks"
        return
    fi
    
    # Check services
    services=("postgresql" "redis" "nginx" "multios-monitoring")
    for service in "${services[@]}"; do
        if systemctl is-active --quiet $service; then
            print_success "$service is running"
        else
            print_error "$service is not running"
        fi
    done
    
    # Check ports
    ports=(80 443 8080)
    for port in "${ports[@]}"; do
        if netstat -tuln | grep -q ":$port "; then
            print_success "Port $port is listening"
        else
            print_warning "Port $port is not listening"
        fi
    done
    
    # Test API endpoint
    if curl -s http://localhost:8080/api/health >/dev/null; then
        print_success "API endpoint is responding"
    else
        print_warning "API endpoint is not responding"
    fi
}

# Start services
start_services() {
    print_info "Starting services..."
    
    if [[ $DRY_RUN == true ]]; then
        print_info "[DRY RUN] Would start services"
        return
    fi
    
    # Start the monitoring service
    systemctl start multios-monitoring
    
    # Check status
    if systemctl is-active --quiet multios-monitoring; then
        print_success "MultiOS Monitoring Infrastructure is running"
    else
        print_error "Failed to start MultiOS Monitoring Infrastructure"
        return 1
    fi
}

# Display post-installation information
display_post_install_info() {
    print_success "=== MultiOS Monitoring Infrastructure Installation Complete ==="
    echo
    print_info "Installation Directory: $INSTALL_DIR"
    print_info "Configuration Directory: $CONFIG_DIR"
    print_info "Log Directory: $LOG_DIR"
    print_info "Data Directory: $DATA_DIR"
    echo
    print_info "Access the dashboard at:"
    print_info "  Local: http://localhost:8080"
    print_info "  Network: http://$(hostname -I | awk '{print $1}'):8080"
    echo
    print_info "Default credentials:"
    print_info "  Username: admin"
    print_info "  Password: admin123"
    print_warning "  Please change the default password after first login!"
    echo
    print_info "Service management commands:"
    print_info "  Start:   sudo systemctl start multios-monitoring"
    print_info "  Stop:    sudo systemctl stop multios-monitoring"
    print_info "  Restart: sudo systemctl restart multios-monitoring"
    print_info "  Status:  sudo systemctl status multios-monitoring"
    print_info "  Logs:    sudo journalctl -u multios-monitoring -f"
    echo
    print_info "Configuration files:"
    print_info "  Main Config: $CONFIG_DIR/monitoring.yaml"
    print_info "  Environment: $CONFIG_DIR/$ENVIRONMENT.env"
    print_info "  Alert Rules: $CONFIG_DIR/rules/"
    echo
    if [[ $QUICK_START == false ]]; then
        print_info "Next steps:"
        print_info "  1. Review and update configuration files"
        print_info "  2. Customize alert rules and notification settings"
        print_info "  3. Configure SSL/TLS certificates for production"
        print_info "  4. Set up external integrations (Grafana, Prometheus)"
        print_info "  5. Review security settings and firewall rules"
    fi
    echo
    print_info "For troubleshooting, see: ./TROUBLESHOOTING.md"
}

# Main installation function
main() {
    print_info "Starting MultiOS Monitoring Infrastructure deployment..."
    
    # Parse arguments
    parse_args "$@"
    
    # Check requirements
    check_root
    detect_os
    
    # Set up depending on installation type
    if [[ $QUICK_START == true ]]; then
        print_info "Running quick start installation..."
        install_system_deps
        install_python_deps
        create_service_user
        create_directories
        copy_application_files
        setup_configuration
        setup_database
        setup_redis
        setup_systemd_service
        setup_nginx
        start_services
    elif [[ $FULL_INSTALL == true ]]; then
        print_info "Running full installation..."
        install_system_deps
        install_nodejs_deps
        install_python_deps
        create_service_user
        create_directories
        copy_application_files
        setup_configuration
        setup_database
        setup_redis
        setup_systemd_service
        setup_nginx
        setup_log_rotation
        setup_fail2ban
        setup_firewall
        start_services
        run_health_checks
    else
        print_info "Running standard installation..."
        install_system_deps
        install_python_deps
        create_service_user
        create_directories
        copy_application_files
        setup_configuration
        setup_database
        setup_redis
        setup_systemd_service
        setup_nginx
        setup_log_rotation
        start_services
        run_health_checks
    fi
    
    # Display post-installation information
    display_post_install_info
}

# Run main function with all arguments
main "$@"
