# MultiOS Backup System User Manual

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start Guide](#quick-start-guide)
4. [Backup Operations](#backup-operations)
5. [Restore Operations](#restore-operations)
6. [Scheduling](#scheduling)
7. [Storage Management](#storage-management)
8. [Lab Profiles](#lab-profiles)
9. [Recovery Media](#recovery-media)
10. [Web Console](#web-console)
11. [Command Line Interface](#command-line-interface)
12. [Troubleshooting](#troubleshooting)

## Introduction

The MultiOS Backup System is a comprehensive, enterprise-grade backup and recovery solution designed for the MultiOS operating system. It provides robust data protection, automated scheduling, and multiple recovery options for educational environments and production systems.

### Key Features

- **Complete System Backup**: Full system snapshots with incremental and differential options
- **File-level Backup**: Granular file and directory backup capabilities
- **Partition-level Backup**: Complete partition imaging and restoration
- **Network Backup**: Remote storage integration for off-site backups
- **Automated Scheduling**: Cron-based scheduling with retention policies
- **Compression & Deduplication**: Efficient storage optimization
- **Point-in-time Recovery**: Multiple restore points with timeline navigation
- **Bootable Recovery Media**: Self-contained recovery environments
- **Encrypted Backups**: AES-256 encryption for security
- **Cloud Integration**: Support for AWS S3, Google Cloud, Azure
- **Enterprise Console**: Web-based management interface
- **Educational Profiles**: Lab-specific backup configurations

## Installation

### System Requirements

- MultiOS operating system
- Root access
- At least 1GB free disk space for the system
- Network connectivity (for remote storage features)

### Installation Steps

1. **Run the installation script:**
   ```bash
   sudo ./scripts/install.sh
   ```

2. **Start the services:**
   ```bash
   sudo systemctl start multios-backup
   sudo systemctl start multios-backup-scheduler
   sudo systemctl start multios-backup-web
   ```

3. **Verify installation:**
   ```bash
   sudo systemctl status multios-backup
   ```

4. **Access the web console:**
   ```
   http://localhost:8080
   ```

### Manual Installation

If you prefer to install manually:

1. **Build the Rust components:**
   ```bash
   cd src
   cargo build --release
   sudo cp target/release/multios-backup /usr/local/bin/
   ```

2. **Install Python dependencies:**
   ```bash
   cd python
   pip3 install -r requirements.txt
   ```

3. **Configure the system:**
   ```bash
   sudo mkdir -p /etc/multios/backup
   sudo cp config/config.toml /etc/multios/backup/
   ```

## Quick Start Guide

### Creating Your First Backup

1. **Using the command line:**
   ```bash
   multios-backup create \
     --type full \
     --source /home \
     --name "My First Backup" \
     --compression zstd \
     --verify
   ```

2. **Using the web console:**
   - Navigate to the Backup page
   - Click "Create Backup"
   - Fill in the backup details
   - Click "Start Backup"

### Listing Backups

```bash
# List all backups
multios-backup list

# List with detailed information
multios-backup list --detailed

# List recent backups
multios-backup list --recent 7
```

### Restoring from Backup

```bash
# Restore the latest backup
multios-backup restore --backup latest --target /tmp/restore

# Restore a specific backup
multios-backup restore --backup backup-123 --target /tmp/restore

# Restore specific files
multios-backup restore --backup backup-123 --target /tmp/restore \
  --include /home/user/documents
```

## Backup Operations

### Backup Types

#### Full Backup
Creates a complete copy of all specified data.
```bash
multios-backup create --type full --source /home --name "Full System Backup"
```

#### Incremental Backup
Only backs up files that have changed since the last backup.
```bash
multios-backup create --type incremental --source /home --name "Daily Increment"
```

#### Differential Backup
Backs up files that have changed since the last full backup.
```bash
multios-backup create --type differential --source /home --name "Differential Backup"
```

#### File-level Backup
Backs up specific files or directories.
```bash
multios-backup create --type file --source /etc --name "Config Backup"
```

### Compression Options

Available compression algorithms:
- `none`: No compression (fastest)
- `gzip`: Standard gzip compression
- `lz4`: Fast compression
- `zstd`: Modern, efficient compression (default)

```bash
multios-backup create --compression lz4 --source /home --name "Fast Backup"
```

### Encryption

Enable encryption for sensitive data:
```bash
multios-backup create --encrypt --source /var/lib --name "Encrypted Backup"
```

### Verification

Verify backup integrity after creation:
```bash
multios-backup create --verify --source /home --name "Verified Backup"
```

## Restore Operations

### Basic Restoration

Restore an entire backup:
```bash
multios-backup restore --backup backup-id --target /tmp/restore
```

### Selective Restoration

Restore specific files or directories:
```bash
multios-backup restore --backup backup-id --target /tmp/restore \
  --include /home/user/documents \
  --exclude /home/user/downloads
```

### Point-in-time Recovery

Restore to a specific point in time:
```bash
multios-backup restore --backup backup-id --target /tmp/restore \
  --point-in-time "2024-01-15 14:30:00"
```

### Verification

Verify the restored data:
```bash
multios-backup restore --backup backup-id --target /tmp/restore --verify
```

## Scheduling

### Creating Schedules

#### Daily Backup Schedule
```bash
multios-backup schedule add \
  --name "Daily Home Backup" \
  --cron "0 2 * * *" \
  --backup-type incremental \
  --source /home
```

#### Weekly Full Backup
```bash
multios-backup schedule add \
  --name "Weekly Full Backup" \
  --cron "0 3 * * 0" \
  --backup-type full \
  --source /
```

### Managing Schedules

```bash
# List all schedules
multios-backup schedule list

# Enable a schedule
multios-backup schedule enable daily-backup

# Disable a schedule
multios-backup schedule disable daily-backup

# Remove a schedule
multios-backup schedule remove daily-backup
```

## Storage Management

### Adding Storage Locations

#### Local Storage
```bash
multios-backup storage add --type local --path /backup --name "Local Backup"
```

#### Network Storage
```bash
multios-backup storage add --type network --path 192.168.1.100:/backup --name "NAS Storage"
```

#### Cloud Storage (AWS S3)
```bash
multios-backup storage add --type amazon_s3 --path my-bucket --name "S3 Backup"
```

### Testing Connectivity
```bash
multios-backup storage test local-backup
```

### Setting Default Storage
```bash
multios-backup storage set-default local-backup
```

## Lab Profiles

Lab profiles provide predefined backup configurations for educational environments.

### Creating Lab Profiles

1. **Using the web console:**
   - Navigate to Lab Profiles
   - Click "Create Profile"
   - Fill in the profile details
   - Set default sources and retention

2. **Using configuration files:**
   Create a YAML file in `/etc/multios/backup/labs/`:
   ```yaml
   id: cs101
   name: CS101 Introduction to Programming
   description: Lab environment backup for CS101 students
   default_sources:
     - /home/students
     - /opt/cs101
   default_retention: "30 days"
   schedule_settings:
     cron_expression: "0 3 * * *"
     backup_type: incremental
   ```

### Applying Lab Profiles

```bash
# Apply a lab profile
multios-backup lab-profile apply cs101

# List available profiles
multios-backup lab-profile list
```

### Example Lab Profiles

#### CS101 - Introduction to Programming
```yaml
id: cs101
name: CS101 Introduction to Programming
description: Basic programming course lab environment
default_sources:
  - /home/students
  - /opt/cs101
  - /var/cs101
default_retention: "30 days"
schedule_settings:
  cron_expression: "0 2 * * *"
  backup_type: incremental
custom_config:
  verify_integrity: true
  compression: zstd
```

#### CS301 - Operating Systems
```yaml
id: cs301
name: CS301 Operating Systems
description: OS development and research lab
default_sources:
  - /home/students
  - /var/cs301
  - /opt/os-dev
  - /home/students/os-projects
default_retention: "90 days"
schedule_settings:
  cron_expression: "0 4 * * 0"
  backup_type: full
custom_config:
  verify_integrity: true
  encryption: true
  compression: zstd
```

## Recovery Media

Create bootable recovery media for disaster recovery.

### Creating ISO Image

```bash
multios-backup recovery-media create \
  --name "multios-recovery" \
  --include-backup backup-123 \
  --include-backup backup-456
```

### Creating Bootable USB

```bash
multios-backup recovery-media create \
  --name "multios-recovery-usb" \
  --usb \
  --device /dev/sdb \
  --include-backup backup-123
```

### Listing Recovery Media

```bash
multios-backup recovery-media list
```

## Web Console

Access the web-based management interface at `http://localhost:8080`.

### Dashboard

The dashboard provides:
- System status overview
- Recent backup jobs
- Storage usage
- Quick actions

### Backup Management

- Create new backups
- Monitor running backups
- View backup history
- Verify backups

### Restore Management

- Browse available backups
- Select restore points
- Choose restore targets
- Monitor restore progress

### Schedule Management

- View scheduled backups
- Enable/disable schedules
- Modify cron expressions
- Monitor schedule execution

### Lab Profiles

- Create lab profiles
- Apply profiles to systems
- Monitor lab backup status
- Manage retention policies

## Command Line Interface

### Main Commands

#### `multios-backup create`
Create a new backup.
```bash
multios-backup create [OPTIONS]

Options:
  --type TYPE              Backup type: full, incremental, differential, file, partition
  --source PATH            Source path to backup (can be specified multiple times)
  --name NAME              Backup name
  --destination PATH       Destination path (default: local storage)
  --compression ALGORITHM  Compression algorithm: none, gzip, lz4, zstd
  --encrypt               Enable encryption
  --verify                Verify backup integrity
  --description TEXT      Backup description
```

#### `multios-backup list`
List available backups.
```bash
multios-backup list [OPTIONS]

Options:
  --detailed              Show detailed information
  --backup-type TYPE      Filter by backup type
  --recent DAYS           Show backups from last N days
```

#### `multios-backup restore`
Restore from a backup.
```bash
multios-backup restore [OPTIONS]

Options:
  --backup ID             Backup ID or name to restore
  --target PATH           Target path for restore
  --include PATH          Include specific files (can be specified multiple times)
  --exclude PATH          Exclude specific files (can be specified multiple times)
  --verify               Verify after restore
  --force                Force overwrite existing files
```

#### `multios-backup verify`
Verify backup integrity.
```bash
multios-backup verify [OPTIONS]

Options:
  --backup ID             Backup ID to verify
  --quick                Quick verification only
  --repair               Attempt to repair damaged files
```

#### `multios-backup schedule`
Manage backup schedules.
```bash
multios-backup schedule <subcommand> [OPTIONS]

Subcommands:
  list                    List all schedules
  add                     Add new schedule
  remove                  Remove schedule
  enable                  Enable schedule
  disable                 Disable schedule
```

#### `multios-backup storage`
Manage storage locations.
```bash
multios-backup storage <subcommand> [OPTIONS]

Subcommands:
  list                    List storage locations
  add                     Add storage location
  remove                  Remove storage location
  test                    Test storage connectivity
  set-default            Set default storage location
```

#### `multios-backup status`
Show system status.
```bash
multios-backup status [OPTIONS]

Options:
  --detailed              Show detailed status
  --json                  Output in JSON format
```

### Quick Commands

#### Quick Restore Operations
```bash
# Restore system files
multios-backup quick-restore system-files --target /tmp/restore

# Restore driver files
multios-backup quick-restore drivers --target /tmp/drivers

# Restore user documents
multios-backup quick-restore documents --target /tmp/documents
```

## Troubleshooting

### Common Issues

#### Backup Fails to Start
1. Check available disk space
2. Verify source paths exist
3. Check storage connectivity
4. Review system logs: `journalctl -u multios-backup.service`

#### Restore Fails
1. Verify backup exists and is intact
2. Check target path permissions
3. Ensure sufficient space at target
4. Check for conflicting files

#### Web Console Not Accessible
1. Verify service is running: `systemctl status multios-backup-web`
2. Check firewall settings
3. Verify port 8080 is not blocked

#### Slow Backup Performance
1. Check compression settings (try faster compression)
2. Verify I/O performance on source and destination
3. Reduce concurrent backup jobs
4. Check network connectivity for remote storage

### Log Locations

- System logs: `/var/log/multios/backup/`
- Application logs: `/var/log/multios/backup/application.log`
- Error logs: `/var/log/multios/backup/error.log`

### Getting Help

1. **System diagnostics:**
   ```bash
   sudo ./scripts/maintenance.sh health-check
   ```

2. **View system status:**
   ```bash
   multios-backup status
   ```

3. **Check recent logs:**
   ```bash
   tail -f /var/log/multios/backup/application.log
   ```

4. **Run tests:**
   ```bash
   sudo ./scripts/maintenance.sh test
   ```

### Recovery Procedures

#### System Recovery from Backup

1. **Boot from recovery media**
2. **Select "Restore System"**
3. **Choose backup to restore**
4. **Select target partitions**
5. **Begin restoration process**

#### Quick File Recovery

1. **Identify corrupted files**
2. **Use quick restore:**
   ```bash
   multios-backup quick-restore system-files --target /tmp/recovery --force
   ```
3. **Restore files to original locations**

### Best Practices

1. **Regular Testing**: Test backup and restore procedures regularly
2. **Multiple Locations**: Use multiple storage locations for redundancy
3. **Monitoring**: Monitor backup job status and system health
4. **Retention**: Configure appropriate retention policies
5. **Verification**: Always verify backup integrity
6. **Documentation**: Document your backup strategy and procedures

### Performance Tuning

1. **Compression**: Use appropriate compression for your data type
2. **Parallel Jobs**: Limit concurrent backup jobs for better performance
3. **Network**: Use wired connections for network backups
4. **Storage**: Use fast storage (SSD) for temporary data
5. **Monitoring**: Monitor system resources during backups

For more information, visit the project documentation or contact the development team.