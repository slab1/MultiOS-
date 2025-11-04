# MultiOS Backup System Installation Guide

## Table of Contents

1. [Overview](#overview)
2. [System Requirements](#system-requirements)
3. [Installation Methods](#installation-methods)
4. [Pre-Installation Checklist](#pre-installation-checklist)
5. [Installation Process](#installation-process)
6. [Post-Installation Configuration](#post-installation-configuration)
7. [Verification](#verification)
8. [Troubleshooting](#troubleshooting)
9. [Uninstallation](#uninstallation)

## Overview

The MultiOS Backup System provides comprehensive backup and recovery capabilities for MultiOS environments. This guide covers the complete installation process, from system requirements to post-installation configuration.

### What Gets Installed

- **Rust Backup Engine**: Core backup and restore functionality
- **Python Management Tools**: Advanced scripting and automation
- **Web Console**: Browser-based management interface
- **System Services**: Background services for automated backups
- **Configuration Files**: Default configurations and templates
- **Documentation**: User manuals and guides

## System Requirements

### Hardware Requirements

#### Minimum Requirements
- **CPU**: Dual-core 1.5 GHz processor
- **Memory**: 2 GB RAM
- **Storage**: 1 GB available disk space
- **Network**: Ethernet connection (for remote storage features)

#### Recommended Requirements
- **CPU**: Quad-core 2.0+ GHz processor
- **Memory**: 4+ GB RAM
- **Storage**: 10+ GB available disk space (for backup data)
- **Network**: Gigabit Ethernet connection

### Software Requirements

#### Operating System
- MultiOS (any supported version)
- Linux kernel 4.0 or higher
- systemd init system

#### Dependencies
- **Rust**: Rust compiler and Cargo package manager
- **Python**: Python 3.8 or higher
- **System Tools**: Standard Unix utilities (dd, gzip, tar, etc.)
- **Build Tools**: gcc, make (for compilation)

### Network Requirements

#### Ports
- **8080**: Web console access
- **443**: HTTPS access (if using web server)
- **22**: SSH access (for remote management)

#### Connectivity
- Internet access for cloud storage features
- Network access to remote storage systems
- DNS resolution capability

## Installation Methods

### Method 1: Automated Installation (Recommended)

The automated installation script handles all dependencies and configuration:

```bash
# Download and run the installer
curl -sSL https://github.com/multios/backup-recovery/raw/main/scripts/install.sh | sudo bash
```

### Method 2: Manual Installation

For advanced users who prefer manual control:

```bash
# Clone the repository
git clone https://github.com/multios/backup-recovery.git
cd backup-recovery

# Run manual installation
sudo ./scripts/install.sh --manual
```

### Method 3: Package Installation

If available in your package manager:

```bash
# Debian/Ubuntu
sudo apt-get install multios-backup

# Red Hat/CentOS
sudo yum install multios-backup

# Arch Linux
sudo pacman -S multios-backup
```

## Pre-Installation Checklist

### System Preparation

#### 1. Update System
```bash
# Update system packages
sudo apt update && sudo apt upgrade -y  # Debian/Ubuntu
sudo yum update -y                      # Red Hat/CentOS
```

#### 2. Install Basic Dependencies
```bash
# Debian/Ubuntu
sudo apt install -y build-essential python3 python3-pip git curl wget

# Red Hat/CentOS
sudo yum groupinstall -y "Development Tools"
sudo yum install -y python3 python3-pip git curl wget
```

#### 3. Install Rust Toolchain
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### 4. Create System User (Optional)
```bash
# Create dedicated backup user
sudo useradd -r -s /bin/bash -d /var/lib/multios/backup multios-backup
sudo usermod -aG disk multios-backup  # Allow disk access
```

#### 5. Prepare Directories
```bash
# Create installation directories
sudo mkdir -p /etc/multios/backup
sudo mkdir -p /var/lib/multios/backup
sudo mkdir -p /var/log/multios/backup

# Set ownership
sudo chown -R root:root /etc/multios/backup
sudo chown -R root:root /var/lib/multios/backup
sudo chown -R root:root /var/log/multios/backup

# Set permissions
sudo chmod 755 /var/lib/multios/backup
sudo chmod 755 /var/log/multios/backup
sudo chmod 600 /etc/multios/backup  # Secure config directory
```

### Network Configuration

#### 1. Configure Firewall
```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 8080/tcp comment "MultiOS Backup Web Console"

# Firewalld (Red Hat/CentOS)
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --reload

# iptables
sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT
```

#### 2. Configure SELinux (if applicable)
```bash
# Check SELinux status
getenforce

# Allow backup operations
setsebool -P backup_journal 1
setsebool -P logrotate_read_generic_config 1
```

## Installation Process

### Automated Installation

#### Step 1: Download Installation Package
```bash
# Option 1: Direct download
wget https://github.com/multios/backup-recovery/archive/main.tar.gz
tar -xzf main.tar.gz
cd backup-recovery-main

# Option 2: Git clone
git clone https://github.com/multios/backup-recovery.git
cd backup-recovery
```

#### Step 2: Run Installation Script
```bash
# Make script executable
chmod +x scripts/install.sh

# Run installation
sudo ./scripts/install.sh
```

The installation script will:
1. Check system requirements
2. Install Rust dependencies
3. Build the backup system
4. Install Python tools
5. Create directory structure
6. Set up configuration files
7. Install systemd services
8. Configure cron jobs
9. Set up firewall rules
10. Create test configurations

#### Step 3: Monitor Installation Progress
The script provides real-time feedback:
```
==========================================
  MultiOS Backup System Installer
==========================================

[INFO] Checking system dependencies...
[SUCCESS] All dependencies found
[INFO] Installing Rust dependencies...
[SUCCESS] Rust backup system built successfully
[INFO] Installing Python dependencies...
[SUCCESS] Python dependencies installed successfully
[INFO] Setting up directory structure...
[SUCCESS] Directory structure created
[INFO] Setting up configuration...
[SUCCESS] Configuration setup complete
[INFO] Setting up systemd services...
[SUCCESS] Systemd services created and enabled
...
```

### Manual Installation

#### Step 1: Build Rust Components
```bash
cd src

# Update Cargo.toml with system paths if needed
cargo build --release

# Install binary
sudo cp target/release/multios-backup /usr/local/bin/
sudo chmod +x /usr/local/bin/multios-backup
```

#### Step 2: Install Python Components
```bash
cd python

# Install system-wide or in virtual environment
sudo pip3 install -r requirements.txt

# Or create virtual environment
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

#### Step 3: Configure System
```bash
# Copy configuration files
sudo cp config/config.toml /etc/multios/backup/
sudo chmod 600 /etc/multios/backup/config.toml

# Set up directories
sudo mkdir -p /var/lib/multios/backup/{backups,temp,media}
sudo mkdir -p /etc/multios/backup/labs
sudo mkdir -p /var/log/multios/backup
```

#### Step 4: Install Systemd Services
```bash
# Copy service files
sudo cp scripts/multios-backup.service /etc/systemd/system/
sudo cp scripts/multios-backup-scheduler.service /etc/systemd/system/
sudo cp scripts/multios-backup-web.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable services
sudo systemctl enable multios-backup.service
sudo systemctl enable multios-backup-scheduler.service
sudo systemctl enable multios-backup-web.service
```

## Post-Installation Configuration

### Basic Configuration

#### 1. Edit Configuration File
```bash
sudo nano /etc/multios/backup/config.toml
```

Key configuration options:
```toml
[paths]
backup_dir = "/var/lib/multios/backup"
temp_dir = "/tmp/multios-backup"
log_dir = "/var/log/multios/backup"

[storage]
default_storage_id = "local-default"
max_concurrent_backups = 4

[compression]
default_algorithm = "zstd"

[encryption]
default_enabled = false
```

#### 2. Configure Storage Locations
```bash
# Add local storage location
multios-backup storage add --type local --path /backup --name "Main Backup" --default

# Add network storage location
multios-backup storage add --type network --path 192.168.1.100:/backup --name "NAS Storage"

# Add cloud storage location
multios-backup storage add --type amazon_s3 --path my-backup-bucket --name "S3 Backup"
```

#### 3. Test Configuration
```bash
# Validate configuration
multios-backup config validate

# Test system status
multios-backup status
```

### Advanced Configuration

#### 1. Set Up Scheduling
```bash
# Create daily incremental backup schedule
multios-backup schedule add \
  --name "Daily Backup" \
  --cron "0 2 * * *" \
  --backup-type incremental \
  --source /home

# Create weekly full backup schedule
multios-backup schedule add \
  --name "Weekly Full Backup" \
  --cron "0 3 * * 0" \
  --backup-type full \
  --source /
```

#### 2. Configure Lab Profiles
```bash
# Create lab profile
cat > /etc/multios/backup/labs/cs101.yaml << EOF
id: cs101
name: CS101 Introduction to Programming
description: Lab environment backup
default_sources:
  - /home/students
  - /opt/cs101
default_retention: "30 days"
schedule_settings:
  cron_expression: "0 3 * * *"
  backup_type: incremental
EOF

# Apply lab profile
multios-backup lab-profile apply cs101
```

#### 3. Set Up Monitoring
```bash
# Enable monitoring
echo "monitoring_enabled = true" >> /etc/multios/backup/config.toml

# Configure log rotation
cat > /etc/logrotate.d/multios-backup << EOF
/var/log/multios/backup/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 root root
}
EOF
```

### Security Configuration

#### 1. Set Up Encryption
```bash
# Generate encryption key
openssl rand -hex 32 > /etc/multios/backup/encryption.key
sudo chmod 600 /etc/multios/backup/encryption.key

# Configure encryption in config
echo "encryption_key_path = \"/etc/multios/backup/encryption.key\"" >> /etc/multios/backup/config.toml
```

#### 2. Configure Access Control
```bash
# Set up user groups
sudo groupadd multios-backup-admin
sudo groupadd multios-backup-user

# Add users to groups
sudo usermod -aG multios-backup-admin adminuser
sudo usermod -aG multios-backup-user student

# Set permissions
sudo chown -R :multios-backup-admin /etc/multios/backup
sudo chmod -R 750 /etc/multios/backup
sudo chmod 640 /etc/multios/backup/config.toml
```

## Verification

### Service Status

#### 1. Check Service Status
```bash
# Check all services
sudo systemctl status multios-backup
sudo systemctl status multios-backup-scheduler
sudo systemctl status multios-backup-web

# Check if services are running
systemctl is-active multios-backup
systemctl is-active multios-backup-scheduler
systemctl is-active multios-backup-web
```

#### 2. Verify Network Access
```bash
# Check web console port
sudo netstat -tlnp | grep 8080

# Test web console
curl -I http://localhost:8080

# Check firewall
sudo ufw status | grep 8080
```

### Functional Testing

#### 1. Test Backup Creation
```bash
# Create test backup
multios-backup create \
  --type full \
  --source /tmp/test-data \
  --name "Test Backup" \
  --verify

# List backups
multios-backup list
```

#### 2. Test Restore Functionality
```bash
# Restore test backup
mkdir -p /tmp/restore-test
multios-backup restore \
  --backup latest \
  --target /tmp/restore-test \
  --verify

# Verify restored data
ls -la /tmp/restore-test/
```

#### 3. Test Quick Operations
```bash
# Test system status
multios-backup status --detailed

# Test verification
multios-backup verify --backup latest --quick

# Test scheduling
multios-backup schedule list
```

### Integration Testing

#### 1. Test Web Console
```bash
# Access web console in browser
# URL: http://localhost:8080

# Or test via command line
curl -s http://localhost:8080/api/status | jq .
```

#### 2. Test Python Tools
```bash
# Test Python backup manager
cd python
python3 backup_manager.py lab-profile list

# Test web console
python3 web_console.py --help
```

## Troubleshooting

### Common Installation Issues

#### Issue 1: Rust Installation Fails
```bash
# Error: rustc not found
# Solution: Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustc --version
```

#### Issue 2: Permission Errors
```bash
# Error: Permission denied
# Solution: Check file permissions
sudo chown -R root:root /usr/local/bin/multios-backup
sudo chmod +x /usr/local/bin/multios-backup

# Check SELinux context (if applicable)
sudo restorecon /usr/local/bin/multios-backup
```

#### Issue 3: Service Won't Start
```bash
# Check service logs
sudo journalctl -u multios-backup.service -f

# Check configuration
multios-backup config validate

# Restart service
sudo systemctl restart multios-backup
```

#### Issue 4: Web Console Not Accessible
```bash
# Check if port is in use
sudo netstat -tlnp | grep 8080

# Check firewall
sudo ufw status

# Check service status
sudo systemctl status multios-backup-web
sudo journalctl -u multios-backup-web.service
```

### Debug Mode

#### Enable Debug Logging
```bash
# Edit configuration
sudo nano /etc/multios/backup/config.toml

# Add debug settings
[monitoring]
log_level = "DEBUG"

# Restart services
sudo systemctl restart multios-backup
```

#### Run Manual Tests
```bash
# Test backup creation manually
multios-backup create --type full --source /tmp --name "Debug Test"

# Test with verbose output
RUST_LOG=debug multios-backup create --type full --source /tmp --name "Debug Test"
```

### Getting Help

#### 1. Check Documentation
```bash
# View installed documentation
ls /usr/share/doc/multios-backup/
cat /etc/multios/backup/README.md
```

#### 2. Generate Support Information
```bash
# Create support bundle
./scripts/maintenance.sh test > support-info.txt
systemctl status >> support-info.txt
journalctl -u multios-backup >> support-info.txt
```

#### 3. Contact Support
- **Documentation**: Check `/usr/share/doc/multios-backup/`
- **Community**: GitHub issues and discussions
- **Commercial Support**: Contact MultiOS team

## Uninstallation

### Complete Removal

#### 1. Stop Services
```bash
# Stop all services
sudo systemctl stop multios-backup-web
sudo systemctl stop multios-backup-scheduler
sudo systemctl stop multios-backup

# Disable services
sudo systemctl disable multios-backup-web
sudo systemctl disable multios-backup-scheduler
sudo systemctl disable multios-backup
```

#### 2. Remove Services
```bash
# Remove systemd service files
sudo rm /etc/systemd/system/multios-backup.service
sudo rm /etc/systemd/system/multios-backup-scheduler.service
sudo rm /etc/systemd/system/multios-backup-web.service

# Reload systemd
sudo systemctl daemon-reload
```

#### 3. Remove Application Files
```bash
# Remove binaries
sudo rm /usr/local/bin/multios-backup

# Remove Python tools
sudo rm -rf /usr/local/lib/python*/site-packages/multios_backup/

# Remove cron jobs
sudo crontab -l | grep -v multios-backup | sudo crontab -
```

#### 4. Remove Configuration and Data
```bash
# Backup configuration (optional)
sudo cp -r /etc/multios/backup /etc/multios/backup.backup.$(date +%Y%m%d)

# Remove configuration
sudo rm -rf /etc/multios/backup

# Remove data (CAUTION: This deletes all backups)
sudo rm -rf /var/lib/multios/backup
sudo rm -rf /var/log/multios/backup
```

#### 5. Remove Dependencies (Optional)
```bash
# Only remove if not used by other applications
sudo apt remove --purge -y build-essential  # If only used for backup
pip3 uninstall multios-backup  # Python packages
```

### Selective Removal

#### Keep Configuration and Data
```bash
# Only stop and disable services
sudo systemctl stop multios-backup
sudo systemctl disable multios-backup

# Keep everything else intact
```

#### Keep Data Only
```bash
# Remove application but keep data
sudo rm /usr/local/bin/multios-backup
sudo rm -rf /etc/multios/backup
sudo rm /etc/systemd/system/multios-backup.service
sudo systemctl daemon-reload

# Data remains in /var/lib/multios/backup
```

### Verification of Removal

```bash
# Check services
systemctl list-unit-files | grep multios-backup

# Check binaries
which multios-backup

# Check processes
ps aux | grep multios-backup

# Check ports
sudo netstat -tlnp | grep 8080
```

---

**Installation Guide Version**: 1.0  
**MultiOS Backup System Version**: 1.0.0  
**Last Updated**: [Current Date]

For additional support, please refer to the [User Manual](USER_MANUAL.md) or contact the MultiOS Backup System team.