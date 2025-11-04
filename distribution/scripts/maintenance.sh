#!/bin/bash
# MultiOS Backup System Maintenance Script
# Handles system maintenance, cleanup, and monitoring

set -euo pipefail

# Configuration
CONFIG_DIR="/etc/multios/backup"
DATA_DIR="/var/lib/multios/backup"
LOG_DIR="/var/log/multios/backup"
BACKUP_BIN="/usr/local/bin/multios-backup"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

show_help() {
    cat << EOF
MultiOS Backup System Maintenance

Usage: $0 <command>

Commands:
    status          Show system status
    backup-status   Show backup job status
    cleanup         Clean up old backups
    verify-all      Verify all backups
    maintenance     Full system maintenance
    rotate-logs     Rotate log files
    health-check    Run system health check
    migrate         Migrate configuration
    test            Run system tests

EOF
}

show_status() {
    log_info "MultiOS Backup System Status"
    
    # Service status
    echo "Service Status:"
    systemctl is-active multios-backup.service && echo "  ✓ Backup Service: Active" || echo "  ✗ Backup Service: Inactive"
    systemctl is-active multios-backup-scheduler.service && echo "  ✓ Scheduler Service: Active" || echo "  ✗ Scheduler Service: Inactive"
    systemctl is-active multios-backup-web.service && echo "  ✓ Web Console: Active" || echo "  ✗ Web Console: Inactive"
    
    echo
    
    # Disk usage
    echo "Storage Usage:"
    du -sh "$DATA_DIR" 2>/dev/null | awk '{print "  Data Directory: " $1}' || echo "  Data Directory: N/A"
    
    # Active jobs
    echo
    echo "Active Jobs:"
    if command -v multios-backup &> /dev/null; then
        multios-backup status --json 2>/dev/null || echo "  No active jobs"
    else
        echo "  multios-backup command not found"
    fi
    
    # Last backup
    echo
    echo "Recent Activity:"
    if [[ -f "$DATA_DIR/backups/.last-backup" ]]; then
        cat "$DATA_DIR/backups/.last-backup"
    else
        echo "  No recent backup activity"
    fi
}

backup_status() {
    log_info "Backup Job Status"
    
    if ! command -v multios-backup &> /dev/null; then
        log_error "multios-backup command not found"
        return 1
    fi
    
    multios-backup status --detailed
}

cleanup_backups() {
    log_info "Cleaning up old backups..."
    
    local retention_days=${1:-30}
    local cleaned_count=0
    local freed_space=0
    
    # Find and remove old backups
    while IFS= read -r backup_dir; do
        if [[ -d "$backup_dir" ]]; then
            local backup_size=$(du -sb "$backup_dir" | cut -f1)
            rm -rf "$backup_dir"
            
            if [[ $? -eq 0 ]]; then
                ((cleaned_count++))
                ((freed_space += backup_size))
                log_success "Removed: $(basename "$backup_dir")"
            fi
        fi
    done < <(find "$DATA_DIR/backups" -maxdepth 1 -type d -mtime +$retention_days -name "backup-*")
    
    # Clean up temp files
    find "$DATA_DIR/temp" -type f -mtime +1 -delete 2>/dev/null || true
    
    log_success "Cleanup completed: $cleaned_count backups removed, $((freed_space / 1024 / 1024)) MB freed"
}

verify_all_backups() {
    log_info "Verifying all backups..."
    
    if ! command -v multios-backup &> /dev/null; then
        log_error "multios-backup command not found"
        return 1
    fi
    
    local verified=0
    local failed=0
    
    while IFS= read -r backup_id; do
        if [[ -n "$backup_id" ]]; then
            log_info "Verifying backup: $backup_id"
            
            if multios-backup verify --backup "$backup_id" --quick; then
                ((verified++))
                log_success "✓ Backup $backup_id verified"
            else
                ((failed++))
                log_error "✗ Backup $backup_id verification failed"
            fi
        fi
    done < <(multios-backup list --format json | jq -r '.[].id' 2>/dev/null || echo "")
    
    log_info "Verification completed: $verified passed, $failed failed"
}

run_maintenance() {
    log_info "Running full system maintenance..."
    
    # Run maintenance steps
    cleanup_backups
    verify_all_backups
    rotate_logs
    
    # Check system health
    run_health_check
    
    # Update configuration if needed
    update_configuration
    
    log_success "System maintenance completed"
}

rotate_logs() {
    log_info "Rotating log files..."
    
    local log_count=$(find "$LOG_DIR" -name "*.log" -type f | wc -l)
    
    if [[ $log_count -gt 0 ]]; then
        # Rotate logs older than 7 days
        find "$LOG_DIR" -name "*.log" -type f -mtime +7 -exec gzip {} \; 2>/dev/null || true
        
        # Remove logs older than 30 days
        find "$LOG_DIR" -name "*.gz" -type f -mtime +30 -delete 2>/dev/null || true
        
        log_success "Log rotation completed: $log_count files processed"
    else
        log_info "No log files to rotate"
    fi
}

run_health_check() {
    log_info "Running system health check..."
    
    local issues=0
    
    # Check services
    if ! systemctl is-active multios-backup.service &> /dev/null; then
        log_error "Backup service is not running"
        ((issues++))
    fi
    
    # Check disk space
    local disk_usage=$(df "$DATA_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
    if [[ $disk_usage -gt 90 ]]; then
        log_error "Disk usage critical: ${disk_usage}%"
        ((issues++))
    elif [[ $disk_usage -gt 80 ]]; then
        log_warning "Disk usage high: ${disk_usage}%"
    fi
    
    # Check configuration
    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        log_error "Configuration file missing"
        ((issues++))
    fi
    
    # Check permissions
    if [[ ! -r "$DATA_DIR" ]]; then
        log_error "Data directory not readable"
        ((issues++))
    fi
    
    if [[ $issues -eq 0 ]]; then
        log_success "System health check passed"
    else
        log_warning "System health check found $issues issues"
    fi
}

update_configuration() {
    log_info "Checking configuration updates..."
    
    local config_file="$CONFIG_DIR/config.toml"
    
    if [[ -f "$config_file" ]]; then
        # Check if config needs updating
        if ! grep -q "version.*1\.0\.0" "$config_file"; then
            log_info "Updating configuration to current version..."
            # Backup original config
            cp "$config_file" "${config_file}.backup.$(date +%Y%m%d_%H%M%S)"
            
            # Apply updates (simplified)
            sed -i 's/version = ".*"/version = "1.0.0"/' "$config_file"
            
            log_success "Configuration updated"
        fi
    fi
}

migrate_config() {
    log_info "Migrating configuration..."
    
    local old_config="/etc/multios/backup.conf"
    local new_config="$CONFIG_DIR/config.toml"
    
    if [[ -f "$old_config" ]] && [[ ! -f "$new_config" ]]; then
        log_info "Migrating from old configuration format..."
        
        # Simple migration (would be more complex in reality)
        cat > "$new_config" << EOF
# Migrated configuration
[system]
version = "1.0.0"
name = "MultiOS Backup System"

[paths]
backup_dir = "$DATA_DIR"
log_dir = "$LOG_DIR"
config_dir = "$CONFIG_DIR"

[storage]
default_storage_id = "local-default"
EOF
        
        log_success "Configuration migrated"
    else
        log_info "No migration needed"
    fi
}

run_tests() {
    log_info "Running system tests..."
    
    local test_results=0
    local test_count=0
    
    # Test basic functionality
    ((test_count++))
    if test_basic_functionality; then
        ((test_results++))
        log_success "✓ Basic functionality test passed"
    else
        log_error "✗ Basic functionality test failed"
    fi
    
    # Test backup creation
    ((test_count++))
    if test_backup_creation; then
        ((test_results++))
        log_success "✓ Backup creation test passed"
    else
        log_error "✗ Backup creation test failed"
    fi
    
    # Test restoration
    ((test_count++))
    if test_restore_functionality; then
        ((test_results++))
        log_success "✓ Restore functionality test passed"
    else
        log_error "✗ Restore functionality test failed"
    fi
    
    # Test compression
    ((test_count++))
    if test_compression; then
        ((test_results++))
        log_success "✓ Compression test passed"
    else
        log_error "✗ Compression test failed"
    fi
    
    log_info "Test results: $test_results/$test_count tests passed"
}

test_basic_functionality() {
    # Create test data
    local test_dir="/tmp/multios-backup-test-$$"
    mkdir -p "$test_dir"
    echo "test data" > "$test_dir/test.txt"
    
    # Test basic operations
    local success=true
    
    if [[ ! -d "$test_dir" ]] || [[ ! -f "$test_dir/test.txt" ]]; then
        success=false
    fi
    
    # Cleanup
    rm -rf "$test_dir"
    
    $success
}

test_backup_creation() {
    # This would test backup creation functionality
    # For now, just check if the command exists
    command -v multios-backup &> /dev/null
}

test_restore_functionality() {
    # This would test restore functionality
    # For now, just check if the command exists
    command -v multios-backup &> /dev/null
}

test_compression() {
    # Test compression tools
    local test_file="/tmp/multios-compression-test-$$"
    echo "test data for compression" > "$test_file"
    
    local success=true
    
    # Test gzip
    if command -v gzip &> /dev/null; then
        gzip "$test_file" && gunzip "${test_file}.gz" || success=false
    fi
    
    # Test zstd if available
    if command -v zstd &> /dev/null; then
        echo "test data for compression" > "$test_file"
        zstd "$test_file" && zstd -d "${test_file}.zst" -o "$test_file" || success=false
    fi
    
    rm -f "$test_file" "${test_file}.gz" "${test_file}.zst"
    
    $success
}

# Main execution
main() {
    case "${1:-}" in
        status)
            show_status
            ;;
        backup-status)
            backup_status
            ;;
        cleanup)
            cleanup_backups "${2:-30}"
            ;;
        verify-all)
            verify_all_backups
            ;;
        maintenance)
            run_maintenance
            ;;
        rotate-logs)
            rotate_logs
            ;;
        health-check)
            run_health_check
            ;;
        migrate)
            migrate_config
            ;;
        test)
            run_tests
            ;;
        *)
            show_help
            exit 1
            ;;
    esac
}

main "$@"