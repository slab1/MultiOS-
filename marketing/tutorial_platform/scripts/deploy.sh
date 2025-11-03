#!/bin/bash

# MultiOS Tutorial Platform Deployment Script
# This script sets up the complete tutorial platform infrastructure

set -e  # Exit on any error

# Configuration
PLATFORM_DIR="/workspace/marketing/tutorial_platform"
DEPLOY_DIR="/var/www/multios-tutorial"
BACKUP_DIR="/backup/multios-tutorial"
LOG_FILE="/var/log/multios-tutorial-deploy.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a $LOG_FILE
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a $LOG_FILE
    exit 1
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a $LOG_FILE
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
    fi
}

# Check system requirements
check_requirements() {
    log "Checking system requirements..."
    
    # Check OS
    if ! command -v lsb_release &> /dev/null; then
        error "Unable to determine OS version"
    fi
    
    # Check available disk space (minimum 50GB)
    available_space=$(df / | awk 'NR==2 {print $4}')
    if [ $available_space -lt 52428800 ]; then  # 50GB in KB
        error "Insufficient disk space. At least 50GB required."
    fi
    
    # Check RAM (minimum 8GB)
    total_mem=$(free -m | awk 'NR==2{print $2}')
    if [ $total_mem -lt 8192 ]; then
        error "Insufficient RAM. At least 8GB required."
    fi
    
    log "System requirements check passed"
}

# Update system packages
update_system() {
    log "Updating system packages..."
    
    # Detect package manager and update
    if command -v apt-get &> /dev/null; then
        apt-get update
        apt-get upgrade -y
        apt-get install -y curl wget git nginx postgresql postgresql-contrib redis-server
    elif command -v yum &> /dev/null; then
        yum update -y
        yum install -y curl wget git nginx postgresql-server postgresql-contrib redis
    else
        error "Unsupported package manager"
    fi
    
    log "System packages updated"
}

# Install Node.js and npm
install_nodejs() {
    log "Installing Node.js and npm..."
    
    # Install Node.js 18.x
    curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
    apt-get install -y nodejs
    
    # Verify installation
    node_version=$(node -v)
    npm_version=$(npm -v)
    log "Node.js $node_version and npm $npm_version installed"
}

# Install PM2 process manager
install_pm2() {
    log "Installing PM2 process manager..."
    npm install -g pm2
    pm2 startup systemd -u root --hp /root
    log "PM2 installed and configured"
}

# Setup PostgreSQL database
setup_database() {
    log "Setting up PostgreSQL database..."
    
    # Start PostgreSQL
    if command -v systemctl &> /dev/null; then
        systemctl start postgresql
        systemctl enable postgresql
    else
        service postgresql start
    fi
    
    # Create database and user
    sudo -u postgres createdb multios_tutorial
    sudo -u postgres createuser --interactive tutorial_user
    
    # Set database password
    sudo -u postgres psql -c "ALTER USER tutorial_user PASSWORD 'tutorial_password';"
    sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE multios_tutorial TO tutorial_user;"
    
    log "Database setup completed"
}

# Setup Redis
setup_redis() {
    log "Setting up Redis..."
    
    if command -v systemctl &> /dev/null; then
        systemctl start redis-server
        systemctl enable redis-server
    else
        service redis-server start
    fi
    
    # Configure Redis
    sed -i 's/# maxmemory <bytes>/maxmemory 2gb/' /etc/redis/redis.conf
    sed -i 's/# maxmemory-policy noeviction/maxmemory-policy allkeys-lru/' /etc/redis/redis.conf
    
    if command -v systemctl &> /dev/null; then
        systemctl restart redis-server
    else
        service redis-server restart
    fi
    
    log "Redis configured and started"
}

# Create application directories
create_directories() {
    log "Creating application directories..."
    
    mkdir -p $DEPLOY_DIR/{app,logs,uploads,backups}
    mkdir -p $DEPLOY_DIR/app/{src,public,config}
    
    # Set permissions
    chown -R www-data:www-data $DEPLOY_DIR
    chmod -R 755 $DEPLOY_DIR
    chmod -R 777 $DEPLOY_DIR/logs
    chmod -R 777 $DEPLOY_DIR/uploads
    
    log "Application directories created"
}

# Copy application files
copy_application() {
    log "Copying application files..."
    
    # Copy from source to deploy directory
    cp -r $PLATFORM_DIR/* $DEPLOY_DIR/app/
    
    # Install Node.js dependencies
    cd $DEPLOY_DIR/app
    npm install --production
    
    log "Application files copied and dependencies installed"
}

# Setup environment variables
setup_environment() {
    log "Setting up environment variables..."
    
    cat > $DEPLOY_DIR/app/.env << EOF
# Database Configuration
DB_HOST=localhost
DB_PORT=5432
DB_NAME=multios_tutorial
DB_USER=tutorial_user
DB_PASSWORD=tutorial_password

# Redis Configuration
REDIS_HOST=localhost
REDIS_PORT=6379

# Application Configuration
NODE_ENV=production
PORT=3000
HOST=0.0.0.0

# JWT Configuration
JWT_SECRET=your_jwt_secret_key_here
JWT_EXPIRES_IN=7d

# File Upload Configuration
UPLOAD_PATH=$DEPLOY_DIR/uploads
MAX_FILE_SIZE=100mb

# Email Configuration (configure as needed)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_email_password

# MultiOS SDK Path
MULTIOS_SDK_PATH=/opt/multios

# Video Streaming Configuration
VIDEO_CDN_URL=https://cdn.multios-tutorial.org
HLS_SEGMENT_DURATION=6
HLS_PLAYLIST_LENGTH=10

# Analytics Configuration
ANALYTICS_ENABLED=true
GOOGLE_ANALYTICS_ID=UA-XXXXXXXX-X

# Security Configuration
RATE_LIMIT_WINDOW=15
RATE_LIMIT_MAX=100
SESSION_SECRET=your_session_secret_here

# Community Features
FORUM_ENABLED=true
CHAT_ENABLED=true
VIDEO_CONFERENCING_ENABLED=true

# Certification System
BLOCKCHAIN_ENABLED=true
CERTIFICATE_VALIDITY_YEARS=3
RENEWAL_REMINDER_DAYS=30
EOF

    chmod 600 $DEPLOY_DIR/app/.env
    chown www-data:www-data $DEPLOY_DIR/app/.env
    
    log "Environment variables configured"
}

# Setup Nginx configuration
setup_nginx() {
    log "Setting up Nginx configuration..."
    
    cat > /etc/nginx/sites-available/multios-tutorial << 'EOF'
server {
    listen 80;
    server_name tutorial.multios.org;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name tutorial.multios.org;
    
    # SSL Configuration (configure certificates)
    # ssl_certificate /path/to/certificate.crt;
    # ssl_certificate_key /path/to/private.key;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN";
    add_header X-XSS-Protection "1; mode=block";
    add_header X-Content-Type-Options "nosniff";
    add_header Referrer-Policy "strict-origin-when-cross-origin";
    
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
    
    # API routes
    location /api {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
    
    # WebSocket routes
    location /socket.io/ {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Static file serving
    location /uploads/ {
        alias $DEPLOY_DIR/uploads/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
    
    # Video streaming
    location /videos/ {
        alias $DEPLOY_DIR/videos/;
        expires 1h;
        add_header Cache-Control "public";
    }
    
    # Main application
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
        
        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            proxy_pass http://localhost:3000;
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
EOF

    # Enable site
    ln -sf /etc/nginx/sites-available/multios-tutorial /etc/nginx/sites-enabled/
    
    # Test configuration
    nginx -t
    
    # Reload Nginx
    systemctl reload nginx
    
    log "Nginx configured and started"
}

# Setup SSL certificates (Let's Encrypt)
setup_ssl() {
    log "Setting up SSL certificates..."
    
    # Install Certbot
    apt-get install -y certbot python3-certbot-nginx
    
    # Get certificate (uncomment and configure domain)
    # certbot --nginx -d tutorial.multios.org --non-interactive --agree-tos --email admin@multios.org
    
    log "SSL certificate setup completed"
}

# Setup PM2 process
setup_pm2_process() {
    log "Setting up PM2 process..."
    
    # Create PM2 ecosystem file
    cat > $DEPLOY_DIR/ecosystem.config.js << EOF
module.exports = {
  apps: [{
    name: 'multios-tutorial',
    script: '$DEPLOY_DIR/app/src/server.js',
    instances: 'max',
    exec_mode: 'cluster',
    env: {
      NODE_ENV: 'production',
      PORT: 3000
    },
    error_file: '$DEPLOY_DIR/logs/pm2-error.log',
    out_file: '$DEPLOY_DIR/logs/pm2-out.log',
    log_file: '$DEPLOY_DIR/logs/pm2.log',
    time: true,
    max_restarts: 10,
    min_uptime: '10s',
    max_memory_restart: '1G',
    node_args: '--max-old-space-size=1024'
  }]
};
EOF

    # Start application with PM2
    cd $DEPLOY_DIR
    pm2 start ecosystem.config.js
    pm2 save
    pm2 startup systemd -u root --hp /root
    
    log "PM2 process started and configured"
}

# Setup backup system
setup_backup() {
    log "Setting up backup system..."
    
    # Create backup script
    cat > $DEPLOY_DIR/backup.sh << 'EOF'
#!/bin/bash
# MultiOS Tutorial Platform Backup Script

BACKUP_DIR="/backup/multios-tutorial"
DATE=$(date +%Y%m%d_%H%M%S)
APP_DIR="/var/www/multios-tutorial"

# Create backup directory
mkdir -p $BACKUP_DIR/$DATE

# Backup database
pg_dump multios_tutorial > $BACKUP_DIR/$DATE/database.sql

# Backup application files
tar -czf $BACKUP_DIR/$DATE/application.tar.gz -C $APP_DIR .

# Backup uploads
tar -czf $BACKUP_DIR/$DATE/uploads.tar.gz -C $APP_DIR/uploads .

# Keep only last 7 days of backups
find $BACKUP_DIR -type d -mtime +7 -exec rm -rf {} \;

echo "Backup completed: $DATE"
EOF

    chmod +x $DEPLOY_DIR/backup.sh
    
    # Add to crontab for daily backups
    (crontab -l 2>/dev/null; echo "0 2 * * * $DEPLOY_DIR/backup.sh") | crontab -
    
    log "Backup system configured"
}

# Setup monitoring
setup_monitoring() {
    log "Setting up monitoring..."
    
    # Install monitoring tools
    apt-get install -y htop iotop nethogs
    
    # Create monitoring script
    cat > $DEPLOY_DIR/monitor.sh << 'EOF'
#!/bin/bash
# MultiOS Tutorial Platform Monitoring Script

LOG_FILE="/var/log/multios-tutorial-monitor.log"
DATE=$(date +'%Y-%m-%d %H:%M:%S')

echo "[$DATE] System monitoring report" >> $LOG_FILE

# CPU usage
CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | awk -F'%' '{print $1}')
echo "[$DATE] CPU Usage: $CPU_USAGE%" >> $LOG_FILE

# Memory usage
MEM_USAGE=$(free | grep Mem | awk '{printf("%.2f", $3/$2 * 100.0)}')
echo "[$DATE] Memory Usage: $MEM_USAGE%" >> $LOG_FILE

# Disk usage
DISK_USAGE=$(df / | awk 'NR==2 {print $5}' | awk -F'%' '{print $1}')
echo "[$DATE] Disk Usage: $DISK_USAGE%" >> $LOG_FILE

# Database connections
DB_CONNECTIONS=$(psql -t -c "SELECT count(*) FROM pg_stat_activity;" 2>/dev/null || echo "0")
echo "[$DATE] Database Connections: $DB_CONNECTIONS" >> $LOG_FILE

# PM2 status
PM2_STATUS=$(pm2 jlist | jq -r '.[0].pm2_env.status' 2>/dev/null || echo "unknown")
echo "[$DATE] PM2 Status: $PM2_STATUS" >> $LOG_FILE
EOF

    chmod +x $DEPLOY_DIR/monitor.sh
    
    # Add to crontab for hourly monitoring
    (crontab -l 2>/dev/null; echo "0 * * * * $DEPLOY_DIR/monitor.sh") | crontab -
    
    log "Monitoring system configured"
}

# Setup log rotation
setup_log_rotation() {
    log "Setting up log rotation..."
    
    cat > /etc/logrotate.d/multios-tutorial << EOF
$DEPLOY_DIR/logs/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    copytruncate
}
EOF

    log "Log rotation configured"
}

# Initialize database schema
initialize_database() {
    log "Initializing database schema..."
    
    cd $DEPLOY_DIR/app
    
    # Create database tables (example - implement actual schema)
    psql -U tutorial_user -d multios_tutorial -f config/schema.sql
    
    # Create default admin user
    # password: admin123 (hash this in production)
    psql -U tutorial_user -d multios_tutorial -c "
    INSERT INTO users (username, email, password_hash, role, created_at) 
    VALUES ('admin', 'admin@multios.org', '\$2b\$10\$hashhere', 'admin', NOW());"
    
    log "Database schema initialized"
}

# Setup firewall
setup_firewall() {
    log "Setting up firewall..."
    
    # Install and configure UFW
    apt-get install -y ufw
    
    # Configure firewall rules
    ufw default deny incoming
    ufw default allow outgoing
    ufw allow ssh
    ufw allow 80/tcp
    ufw allow 443/tcp
    
    # Enable firewall
    echo "y" | ufw enable
    
    log "Firewall configured"
}

# Setup email notifications
setup_notifications() {
    log "Setting up email notifications..."
    
    # Install postfix for email
    apt-get install -y postfix
    
    # Configure email relay (configure as needed)
    # Edit /etc/postfix/main.cf
    
    log "Email notifications configured"
}

# Final system hardening
system_hardening() {
    log "Applying system hardening..."
    
    # Disable unused services
    systemctl disable bluetooth
    systemctl disable cups
    systemctl disable avahi-daemon
    
    # Set secure kernel parameters
    cat >> /etc/sysctl.conf << EOF
# MultiOS Tutorial Platform Security
net.ipv4.ip_forward = 0
net.ipv4.conf.all.send_redirects = 0
net.ipv4.conf.default.send_redirects = 0
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.default.accept_redirects = 0
net.ipv6.conf.all.accept_redirects = 0
net.ipv6.conf.default.accept_redirects = 0
net.ipv4.conf.all.accept_source_route = 0
net.ipv4.conf.default.accept_source_route = 0
net.ipv6.conf.all.accept_source_route = 0
net.ipv6.conf.default.accept_source_route = 0
EOF

    sysctl -p
    
    log "System hardening completed"
}

# Create management scripts
create_management_scripts() {
    log "Creating management scripts..."
    
    # Start script
    cat > $DEPLOY_DIR/start.sh << EOF
#!/bin/bash
echo "Starting MultiOS Tutorial Platform..."
pm2 start multios-tutorial
systemctl start nginx
systemctl start postgresql
systemctl start redis-server
echo "Platform started successfully"
EOF

    # Stop script
    cat > $DEPLOY_DIR/stop.sh << EOF
#!/bin/bash
echo "Stopping MultiOS Tutorial Platform..."
pm2 stop multios-tutorial
systemctl stop nginx
echo "Platform stopped successfully"
EOF

    # Restart script
    cat > $DEPLOY_DIR/restart.sh << EOF
#!/bin/bash
echo "Restarting MultiOS Tutorial Platform..."
pm2 restart multios-tutorial
systemctl reload nginx
echo "Platform restarted successfully"
EOF

    # Status script
    cat > $DEPLOY_DIR/status.sh << EOF
#!/bin/bash
echo "MultiOS Tutorial Platform Status:"
echo "================================="
pm2 status
systemctl is-active nginx
systemctl is-active postgresql
systemctl is-active redis-server
EOF

    chmod +x $DEPLOY_DIR/*.sh
    
    log "Management scripts created"
}

# Final verification
final_verification() {
    log "Performing final verification..."
    
    # Check if services are running
    services=("nginx" "postgresql" "redis-server")
    
    for service in "${services[@]}"; do
        if systemctl is-active --quiet $service; then
            log "$service is running"
        else
            warning "$service is not running"
        fi
    done
    
    # Check PM2 processes
    pm2_status=$(pm2 jlist 2>/dev/null | jq -r '.[0].pm2_env.status' || echo "unknown")
    if [ "$pm2_status" = "online" ]; then
        log "Application is running"
    else
        warning "Application status: $pm2_status"
    fi
    
    # Test database connection
    if psql -U tutorial_user -d multios_tutorial -c "SELECT 1;" >/dev/null 2>&1; then
        log "Database connection successful"
    else
        warning "Database connection failed"
    fi
    
    # Test Redis connection
    if redis-cli ping >/dev/null 2>&1; then
        log "Redis connection successful"
    else
        warning "Redis connection failed"
    fi
    
    log "Final verification completed"
}

# Main deployment function
main() {
    log "Starting MultiOS Tutorial Platform deployment..."
    
    check_root
    check_requirements
    
    # Create log file
    touch $LOG_FILE
    
    update_system
    install_nodejs
    install_pm2
    setup_database
    setup_redis
    create_directories
    copy_application
    setup_environment
    setup_nginx
    # setup_ssl  # Uncomment after domain configuration
    setup_pm2_process
    setup_backup
    setup_monitoring
    setup_log_rotation
    initialize_database
    setup_firewall
    setup_notifications
    system_hardening
    create_management_scripts
    final_verification
    
    log "================================================"
    log "MultiOS Tutorial Platform deployment completed!"
    log "================================================"
    log "Platform URL: https://tutorial.multios.org"
    log "Default admin user: admin@multios.org"
    log "Database: multios_tutorial"
    log "Backup directory: $BACKUP_DIR"
    log "Log file: $LOG_FILE"
    log "================================================"
    log "Next steps:"
    log "1. Configure SSL certificates (certbot)"
    log "2. Update DNS records for tutorial.multios.org"
    log "3. Configure email settings in .env"
    log "4. Upload initial content and videos"
    log "5. Set up monitoring and alerts"
    log "================================================"
}

# Run main function
main "$@"
