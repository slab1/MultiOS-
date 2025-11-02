#!/usr/bin/multios/sh
# MultiOS CLI Shell - Example Scripts Collection
# This file contains various examples of CLI shell usage and scripting

# =============================================================================
# Basic System Information Script
# =============================================================================

function show_system_info() {
    print "=== MultiOS System Information ==="
    print "Kernel: $(uname)"
    print "Uptime: $(uptime)"
    print "Memory: $(free -h)"
    print "Processes: $(ps | wc -l) total"
    print "Current User: $USER"
    print "Working Directory: $PWD"
    print "Shell Version: $SHELL_VERSION"
}

# =============================================================================
# File Backup Script
# =============================================================================

function backup_files() {
    if [ $(array_size $args) -eq 0 ]; then
        print "Usage: backup_files <source_directory> [destination_directory]"
        return 1
    fi
    
    local source_dir = $args[0]
    local dest_dir = ${args[1]:-/backup}
    local timestamp = $(date +%Y%m%d_%H%M%S)
    local backup_name = "backup_${timestamp}.tar.gz"
    
    if !(file_exists $source_dir); then
        print "Error: Source directory '$source_dir' does not exist"
        return 1
    fi
    
    print "Creating backup of '$source_dir' to '$dest_dir/$backup_name'..."
    
    # Create destination directory if it doesn't exist
    execute "mkdir -p $dest_dir"
    
    # Create the backup
    execute "tar -czf $dest_dir/$backup_name $source_dir"
    
    if $(file_exists "$dest_dir/$backup_name"); then
        print "Backup created successfully: $dest_dir/$backup_name"
        print "Backup size: $(length $(read_file "$dest_dir/$backup_name")) bytes"
        return 0
    else
        print "Error: Backup creation failed"
        return 1
    fi
}

# =============================================================================
# Process Monitor Script
# =============================================================================

function monitor_processes() {
    print "=== Process Monitor ==="
    
    # Get top processes by CPU usage
    print "Top 5 processes by CPU usage:"
    ps --sort=-%cpu | head -6
    
    print "\nTop 5 processes by Memory usage:"
    ps --sort=-%mem | head -6
    
    # Check for zombie processes
    local zombie_count = $(ps aux | grep -c "<defunct>")
    if [ $zombie_count -gt 0 ]; then
        print "\nWarning: Found $zombie_count zombie process(es)"
        print "Zombie processes:"
        ps aux | grep "<defunct>"
    else
        print "\nNo zombie processes found"
    fi
    
    # System load average
    print "\nSystem Load:"
    uptime
}

# =============================================================================
# Network Diagnostic Script
# =============================================================================

function network_diagnostics() {
    print "=== Network Diagnostics ==="
    
    # Check network interfaces
    print "Network Interfaces:"
    execute "ip addr show" || print "Could not display network interfaces"
    
    # Check routing table
    print "\nRouting Table:"
    execute "ip route show" || print "Could not display routing table"
    
    # Test connectivity to common hosts
    print "\nConnectivity Tests:"
    for host in "8.8.8.8" "1.1.1.1" "google.com"; do
        print "Testing connection to $host..."
        execute "ping -c 1 $host" || print "Failed to reach $host"
    done
    
    # Check DNS resolution
    print "\nDNS Resolution Test:"
    execute "nslookup google.com" || print "DNS resolution failed"
}

# =============================================================================
# User Management Script
# =============================================================================

function user_management_menu() {
    while true; do
        print "\n=== User Management Menu ==="
        print "1. List all users"
        print "2. Show user information"
        print "3. Add new user"
        print "4. Modify user"
        print "5. Delete user"
        print "6. Exit"
        print -n "Select an option: "
        
        local choice = $(read_input)
        
        case $choice in
            1) list_users ;;
            2) show_user_info ;;
            3) add_user ;;
            4) modify_user ;;
            5) delete_user ;;
            6) break ;;
            *) print "Invalid option. Please try again." ;;
        esac
    done
}

function list_users() {
    print "=== User List ==="
    # This would integrate with actual user management
    print "Current user: $USER"
    print "Home directory: $HOME"
    print "Shell: $SHELL"
}

function show_user_info() {
    print -n "Enter username: "
    local username = $(read_input)
    
    if [ "$username" = "$USER" ]; then
        print "=== Current User Information ==="
        print "Username: $username"
        print "Home: $HOME"
        print "Shell: $SHELL"
        print "Working Directory: $PWD"
        print "Environment Variables: $(array_size $(keys $env))"
    else
        print "User information for '$username' not available"
        print "This is a restricted operation"
    fi
}

function read_input() {
    # Simple input function for scripts
    # In real implementation, this would read from stdin
    return "1"
}

# =============================================================================
# Database Backup Script
# =============================================================================

function database_backup() {
    if [ $(array_size $args) -lt 2 ]; then
        print "Usage: database_backup <database_name> <backup_directory>"
        return 1
    fi
    
    local db_name = $args[0]
    local backup_dir = $args[1]
    local timestamp = $(date +%Y%m%d_%H%M%S)
    local backup_file = "$backup_dir/${db_name}_${timestamp}.sql"
    
    print "Backing up database '$db_name' to '$backup_file'..."
    
    # Create backup directory
    execute "mkdir -p $backup_dir"
    
    # This would use actual database backup commands
    # For demonstration, we'll create a mock backup
    local mock_data = "-- Database backup for $db_name\n-- Timestamp: $timestamp\nCREATE DATABASE backup_test;"
    write_file $backup_file $mock_data
    
    if $(file_exists $backup_file); then
        print "Database backup completed: $backup_file"
        print "Backup size: $(length $(read_file $backup_file)) bytes"
        return 0
    else
        print "Database backup failed"
        return 1
    fi
}

# =============================================================================
# Log Analysis Script
# =============================================================================

function analyze_logs() {
    if [ $(array_size $args) -eq 0 ]; then
        print "Usage: analyze_logs <log_file> [pattern]"
        return 1
    fi
    
    local log_file = $args[0]
    local pattern = ${args[1]:-""}
    
    if !(file_exists $log_file); then
        print "Error: Log file '$log_file' does not exist"
        return 1
    fi
    
    print "=== Log Analysis for $log_file ==="
    
    # Get basic statistics
    local total_lines = $(wc -l $log_file)
    local file_size = $(stat -f%z $log_file)
    
    print "Total lines: $total_lines"
    print "File size: $file_size bytes"
    
    # Search for errors if no pattern provided
    if [ -z "$pattern" ]; then
        print "\nSearching for common error patterns..."
        local error_count = 0
        local warning_count = 0
        
        # This would process the actual log file
        print "Error patterns found: $error_count"
        print "Warning patterns found: $warning_count"
    else
        print "\nSearching for pattern: $pattern"
        # This would search for the specific pattern
        print "Pattern matches would be displayed here"
    fi
}

# =============================================================================
# System Maintenance Script
# =============================================================================

function system_maintenance() {
    print "=== MultiOS System Maintenance ==="
    
    # Update system time
    print "1. Synchronizing system time..."
    execute "ntpdate -s time.nist.gov" || print "Time synchronization failed (ntpdate not available)"
    
    # Clean temporary files
    print "2. Cleaning temporary files..."
    local temp_dirs = ["/tmp", "/var/tmp", "/home/user/.cache"]
    for dir in $temp_dirs; do
        if $(file_exists $dir); then
            execute "find $dir -type f -atime +7 -delete" || print "Could not clean $dir"
            print "Cleaned $dir"
        fi
    done
    
    # Check disk space
    print "3. Checking disk space..."
    execute "df -h"
    
    # Check system logs
    print "4. Checking system logs for errors..."
    local log_files = ["/var/log/syslog", "/var/log/messages"]
    for log in $log_files; do
        if $(file_exists $log); then
            local error_count = $(grep -c "ERROR\|CRITICAL" $log 2>/dev/null || echo "0")
            if [ $error_count -gt 0 ]; then
                print "Found $error_count errors in $log"
            else
                print "No errors found in $log"
            fi
        fi
    done
    
    # Update package database (if applicable)
    print "5. Updating package database..."
    execute "apt update" || execute "yum check-update" || print "Package manager not available"
    
    print "\nSystem maintenance completed"
}

# =============================================================================
# Application Deployment Script
# =============================================================================

function deploy_application() {
    if [ $(array_size $args) -lt 2 ]; then
        print "Usage: deploy_application <app_name> <version> [environment]"
        return 1
    fi
    
    local app_name = $args[0]
    local version = $args[1]
    local environment = ${args[2]:-"production"}
    local deploy_dir = "/opt/$app_name"
    
    print "=== Deploying $app_name v$version to $environment ==="
    
    # Create deployment directory
    execute "mkdir -p $deploy_dir"
    
    # Download application (mock)
    print "1. Downloading application..."
    local app_package = "$app_name-$version.tar.gz"
    print "Would download: $app_package"
    
    # Extract application
    print "2. Extracting application..."
    execute "tar -xzf $app_package -C $deploy_dir" || print "Extraction failed (mock)"
    
    # Set permissions
    print "3. Setting permissions..."
    execute "chmod 755 $deploy_dir"
    execute "chown -R app:app $deploy_dir" || print "User 'app' not found"
    
    # Create systemd service (if applicable)
    print "4. Creating service..."
    local service_file = "/etc/systemd/system/$app_name.service"
    local service_config = "[Unit]
Description=$app_name application
After=network.target

[Service]
Type=simple
User=app
ExecStart=$deploy_dir/$app_name
Restart=always

[Install]
WantedBy=multi-user.target"
    
    write_file $service_file $service_config
    execute "systemctl daemon-reload" || print "systemd not available"
    
    # Start the application
    print "5. Starting application..."
    execute "systemctl start $app_name" || print "Failed to start service"
    execute "systemctl enable $app_name" || print "Failed to enable service"
    
    print "Deployment completed successfully"
    print "Application is running at: $deploy_dir"
    print "Service status: systemctl status $app_name"
}

# =============================================================================
# Performance Monitoring Script
# =============================================================================

function performance_monitor() {
    local duration = ${args[0]:-60}  # Default 60 seconds
    local interval = ${args[1]:-5}   # Default 5 second intervals
    
    print "=== Performance Monitor ==="
    print "Duration: $duration seconds"
    print "Sampling interval: $interval seconds"
    print "Starting monitoring..."
    
    local start_time = $(date +%s)
    local end_time = $start_time + $duration
    
    while [ $(date +%s) -lt $end_time ]; do
        local current_time = $(date)
        print "\n--- Sample at $current_time ---"
        
        # CPU usage
        print "CPU Usage:"
        execute "top -bn1 | grep 'Cpu(s)'" || print "CPU info not available"
        
        # Memory usage
        print "Memory Usage:"
        execute "free -h"
        
        # Disk I/O
        print "Disk I/O:"
        execute "iostat 1 1" || print "iostat not available"
        
        # Network statistics
        print "Network Statistics:"
        execute "cat /proc/net/dev" | grep -v "lo:" || print "Network stats not available"
        
        # Load average
        print "Load Average:"
        execute "uptime"
        
        # Sleep before next sample
        if [ $(date +%s) -lt $end_time ]; then
            sleep $interval
        fi
    done
    
    print "\nMonitoring completed"
}

# =============================================================================
# Security Audit Script
# =============================================================================

function security_audit() {
    print "=== MultiOS Security Audit ==="
    
    # Check user accounts
    print "1. User Account Security"
    print "Current user: $USER"
    print "User privileges: $(groups)"
    
    # Check file permissions
    print "\n2. File Permissions Check"
    local critical_files = ["/etc/passwd", "/etc/shadow", "/etc/sudoers"]
    for file in $critical_files; do
        if $(file_exists $file); then
            local perms = $(stat -f%Mp%Lp $file)
            print "$file: $perms"
            # Check for world-writable files
            if [[ $perms == *"w"* ]] && [[ $perms == *"other"* ]]; then
                print "  WARNING: World-writable file detected!"
            fi
        fi
    done
    
    # Check running services
    print "\n3. Running Services"
    execute "systemctl list-units --type=service --state=running" || print "systemd not available"
    
    # Check network ports
    print "\n4. Open Network Ports"
    execute "netstat -tlnp" || execute "ss -tlnp" || print "Network tools not available"
    
    # Check authentication
    print "\n5. Authentication Security"
    if $(file_exists "/etc/pam.d/common-password"); then
        print "PAM configuration found"
    else
        print "PAM not configured"
    fi
    
    print "\nSecurity audit completed"
}

# =============================================================================
# Main Script Execution
# =============================================================================

# Parse command line arguments
if [ $(array_size $args) -eq 0 ]; then
    print "MultiOS CLI Shell - Example Scripts"
    print "Available functions:"
    print "  show_system_info        - Display system information"
    print "  backup_files <src> [dst] - Backup files or directories"
    print "  monitor_processes        - Monitor system processes"
    print "  network_diagnostics      - Network diagnostic tools"
    print "  user_management_menu     - User management interface"
    print "  database_backup <db> <dir> - Database backup utility"
    print "  analyze_logs <file> [pat] - Analyze log files"
    print "  system_maintenance       - System maintenance tasks"
    print "  deploy_application <name> <ver> [env] - Deploy applications"
    print "  performance_monitor [dur] [int] - Performance monitoring"
    print "  security_audit           - Security audit tools"
    print ""
    print "Usage: multios_shell_examples.sh <function_name> [arguments]"
    
    # Run a default set of functions
    print "Running default system overview..."
    show_system_info
    monitor_processes
else
    # Execute the requested function
    local function_name = $args[0]
    local function_args = ${args[1..]}  # Extract arguments starting from index 1
    
    if $(type -t $function_name 2>/dev/null); then
        print "Executing: $function_name"
        $function_name $function_args
    else
        print "Error: Function '$function_name' not found"
        print "Available functions:"
        print "  show_system_info, backup_files, monitor_processes, network_diagnostics"
        print "  user_management_menu, database_backup, analyze_logs, system_maintenance"
        print "  deploy_application, performance_monitor, security_audit"
        exit 1
    fi
fi

print "\nMultiOS CLI Shell examples completed successfully"
exit 0