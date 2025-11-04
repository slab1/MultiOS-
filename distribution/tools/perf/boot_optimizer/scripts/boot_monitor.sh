#!/bin/bash

# Boot Performance Monitor
# Continuously monitors boot performance and alerts on degradation

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="/var/lib/boot_monitor"
LOG_FILE="/var/log/boot_monitor.log"
ALERT_THRESHOLD=3000  # 3 seconds
WARNING_THRESHOLD=2500  # 2.5 seconds
MAX_HISTORY=30  # Keep last 30 boot measurements

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log() {
    echo -e "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Boot Performance Monitor${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo
}

print_section() {
    echo -e "\n${YELLOW}>>> $1${NC}"
}

init_monitor() {
    mkdir -p "$DATA_DIR"
    mkdir -p "$(dirname "$LOG_FILE")"
    
    # Create history file if it doesn't exist
    local history_file="$DATA_DIR/boot_history.json"
    if [[ ! -f "$history_file" ]]; then
        echo '{"boots": []}' > "$history_file"
    fi
}

get_boot_time() {
    if command -v systemd-analyze &> /dev/null; then
        local total_time=$(systemd-analyze | grep -oP 'boot time: \K[0-9.]+' || echo "0")
        echo "$total_time" | tr -d 'ms' 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

get_boot_type() {
    # Determine boot type based on various factors
    local uptime=$(uptime -p 2>/dev/null | grep -oP 'up \K[0-9]+' || echo "0")
    
    if [[ $uptime -lt 60 ]]; then
        echo "Cold Boot"
    else
        echo "Warm Boot"
    fi
}

record_boot_measurement() {
    local boot_time=$1
    local boot_type=$2
    local timestamp=$(date -Iseconds)
    local kernel=$(uname -r)
    
    # Get current optimizations applied
    local optimizations=()
    
    # Check GRUB timeout
    if [[ -f "/etc/default/grub" ]]; then
        local timeout=$(grep "^GRUB_TIMEOUT=" "/etc/default/grub" | cut -d'=' -f2 || echo "5")
        if [[ $timeout == "0" ]]; then
            optimizations+=("fast_grub")
        fi
    fi
    
    # Check for kernel parameters
    local cmdline=$(cat /proc/cmdline)
    if echo "$cmdline" | grep -q "quiet"; then
        optimizations+=("quiet_kernel")
    fi
    
    # Check service optimizations
    if systemctl is-disabled cups.service &> /dev/null 2>&1; then
        optimizations+=("disabled_services")
    fi
    
    # Record to JSON file
    local history_file="$DATA_DIR/boot_history.json"
    local temp_file=$(mktemp)
    
    # Read existing data
    local existing_data="{}"
    if [[ -f "$history_file" ]]; then
        existing_data=$(cat "$history_file")
    fi
    
    # Create new entry
    cat > "$temp_file" << EOF
{
  "timestamp": "$timestamp",
  "boot_time_ms": $boot_time,
  "boot_type": "$boot_type",
  "kernel": "$kernel",
  "optimizations": $(printf '%s\n' "${optimizations[@]}" | jq -R . | jq -s .)
}
EOF
    
    # Append to history (keep only recent entries)
    jq -c ".boots += [$(cat "$temp_file")]" "$history_file" > "$temp_file.tmp" 2>/dev/null || {
        # Fallback if jq fails
        cp "$history_file" "$temp_file.tmp"
        echo "," >> "$temp_file.tmp"
        cat "$temp_file" >> "$temp_file.tmp"
        echo "}" >> "$temp_file.tmp"
    }
    
    # Trim history to max size
    jq ".boots = (.boots | sort_by(.timestamp) | .[-$MAX_HISTORY:])" "$temp_file.tmp" > "$history_file" 2>/dev/null || cp "$temp_file.tmp" "$history_file"
    
    rm -f "$temp_file" "$temp_file.tmp"
    
    echo "$timestamp,$boot_time,$boot_type,${optimizations[*]}" >> "$DATA_DIR/boot_history.csv"
    
    log "Recorded boot measurement: ${boot_type} - ${boot_time}ms"
}

analyze_boot_trend() {
    local history_file="$DATA_DIR/boot_history.json"
    
    if [[ ! -f "$history_file" ]]; then
        echo "No boot history available"
        return
    fi
    
    print_section "Boot Performance Trend Analysis"
    
    # Get recent boots (last 10)
    local recent_boots=$(jq -r '.boots[-10:] | map(.boot_time_ms) | join(" ")' "$history_file" 2>/dev/null)
    
    if [[ -n "$recent_boots" ]]; then
        echo "Recent boot times (last 10): $recent_boots ms"
        
        # Calculate average
        local avg_boot=$(echo "$recent_boots" | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')
        echo "Average boot time: ${avg_boot}ms"
        
        # Check for trend
        local trend="stable"
        local recent_avg=$(echo "$recent_boots" | tr ' ' '\n' | tail -5 | awk '{sum+=$1} END {print sum/NR}')
        local older_avg=$(echo "$recent_boots" | tr ' ' '\n' | head -5 | awk '{sum+=$1} END {print sum/NR}')
        
        if (( $(echo "$recent_avg > $older_avg * 1.1" | bc -l) )); then
            trend="degrading"
        elif (( $(echo "$recent_avg < $older_avg * 0.9" | bc -l) )); then
            trend="improving"
        fi
        
        case $trend in
            "degrading")
                echo -e "${RED}⚠️  Boot performance is degrading${NC}"
                ;;
            "improving")
                echo -e "${GREEN}✓ Boot performance is improving${NC}"
                ;;
            *)
                echo "Boot performance is stable"
                ;;
        esac
    else
        echo "Insufficient data for trend analysis"
    fi
}

check_performance_alerts() {
    local boot_time=$1
    local history_file="$DATA_DIR/boot_history.json"
    
    print_section "Performance Alerts"
    
    if [[ $boot_time -gt $ALERT_THRESHOLD ]]; then
        echo -e "${RED}🔴 CRITICAL: Boot time ($boot_time ms) exceeds alert threshold ($ALERT_THRESHOLD ms)${NC}"
        send_alert "CRITICAL" "Boot time $boot_time ms exceeds threshold"
    elif [[ $boot_time -gt $WARNING_THRESHOLD ]]; then
        echo -e "${YELLOW}🟡 WARNING: Boot time ($boot_time ms) exceeds warning threshold ($WARNING_THRESHOLD ms)${NC}"
        send_alert "WARNING" "Boot time $boot_time ms exceeds threshold"
    else
        echo -e "${GREEN}✓ Boot time ($boot_time ms) is within acceptable limits${NC}"
    fi
    
    # Check for performance degradation
    if [[ -f "$history_file" ]]; then
        local recent_boots=$(jq -r '.boots[-5:] | map(.boot_time_ms) | join(" ")' "$history_file" 2>/dev/null)
        
        if [[ -n "$recent_boots" ]]; then
            local old_avg=$(echo "$recent_boots" | tr ' ' '\n' | head -3 | awk '{sum+=$1} END {print sum/NR}')
            local new_avg=$(echo "$recent_boots" | tr ' ' '\n' | tail -3 | awk '{sum+=$1} END {print sum/NR}')
            
            if (( $(echo "$new_avg > $old_avg * 1.2" | bc -l) )); then
                echo -e "${YELLOW}⚠️  Performance degradation detected (20%+ slower than recent boots)${NC}"
                send_alert "DEGRADATION" "Boot performance degraded by $(echo "scale=1; ($new_avg - $old_avg) / $old_avg * 100" | bc)%"
            fi
        fi
    fi
}

send_alert() {
    local severity=$1
    local message=$2
    
    # Log alert
    log "ALERT [$severity]: $message"
    
    # Send to system log
    logger -t boot_monitor -p daemon.warn "Boot Monitor Alert [$severity]: $message"
    
    # Additional alert mechanisms could be added here
    # - Email notifications
    # - Slack/Discord webhooks
    # - SNMP traps
    # - etc.
}

generate_performance_report() {
    local output_file="$DATA_DIR/performance_report_$(date +%Y%m%d_%H%M%S).txt"
    
    print_section "Generating Performance Report"
    
    {
        echo "Boot Performance Report"
        echo "Generated: $(date)"
        echo "System: $(uname -a)"
        echo
        
        echo "=== CURRENT STATUS ==="
        echo "Current boot time: $(get_boot_time)ms"
        echo "Boot type: $(get_boot_type)"
        echo
        
        echo "=== RECENT BOOT HISTORY ==="
        if [[ -f "$DATA_DIR/boot_history.json" ]]; then
            jq -r '.boots[-10:] | .[] | "\(.timestamp) - \(.boot_type) - \(.boot_time_ms)ms"' "$DATA_DIR/boot_history.json" 2>/dev/null || echo "Could not parse history"
        fi
        echo
        
        echo "=== PERFORMANCE ANALYSIS ==="
        analyze_boot_trend
        echo
        
        echo "=== SYSTEM CONFIGURATION ==="
        echo "Kernel parameters:"
        cat /proc/cmdline
        echo
        echo "GRUB timeout:"
        [[ -f "/etc/default/grub" ]] && grep "^GRUB_TIMEOUT=" "/etc/default/grub" || echo "Not found"
        echo
        
        echo "=== RECOMMENDATIONS ==="
        generate_recommendations
        
    } > "$output_file"
    
    echo "Report saved to: $output_file"
}

generate_recommendations() {
    local current_boot=$(get_boot_time)
    
    if [[ $current_boot -gt 2000 ]]; then
        echo "• Boot time exceeds 2-second target - run boot_optimizer.sh"
    fi
    
    if [[ -f "/etc/default/grub" ]]; then
        local timeout=$(grep "^GRUB_TIMEOUT=" "/etc/default/grub" | cut -d'=' -f2 || echo "5")
        if [[ $timeout -gt 0 ]]; then
            echo "• Set GRUB_TIMEOUT=0 in /etc/default/grub"
        fi
    fi
    
    local slow_services=$(systemd-analyze blame 2>/dev/null | awk '$1 ~ /^[0-9]+\.[0-9]+s$/ && $1 > 0.5 {print $2}' | wc -l)
    if [[ $slow_services -gt 3 ]]; then
        echo "• Review and optimize $slow_services slow services"
    fi
    
    if ! systemctl is-disabled cups.service &> /dev/null 2>&1; then
        echo "• Consider disabling unnecessary services (cups, bluetooth, etc.)"
    fi
    
    echo "• Run boot_analyzer.sh for detailed analysis"
}

continuous_monitor() {
    print_section "Starting Continuous Monitor"
    
    log "Starting continuous boot monitoring"
    
    local last_boot_file="$DATA_DIR/last_boot_measurement"
    
    while true; do
        local current_boot=$(get_boot_time)
        local boot_type=$(get_boot_type)
        
        # Check if this is a new boot
        if [[ ! -f "$last_boot_file" ]] || [[ "$current_boot" != "$(cat "$last_boot_file")" ]]; then
            echo "New boot detected: $boot_type - ${current_boot}ms"
            
            # Record measurement
            record_boot_measurement "$current_boot" "$boot_type"
            
            # Check for alerts
            check_performance_alerts "$current_boot"
            
            # Save current boot time
            echo "$current_boot" > "$last_boot_file"
        fi
        
        # Sleep for 1 minute before checking again
        sleep 60
    done
}

show_current_status() {
    print_section "Current Boot Status"
    
    local current_boot=$(get_boot_time)
    local boot_type=$(get_boot_type)
    
    echo "Current boot time: ${current_boot}ms"
    echo "Boot type: $boot_type"
    
    # Performance indicator
    if [[ $current_boot -le 1500 ]]; then
        echo -e "Performance: ${GREEN}Excellent${NC} (< 1.5s)"
    elif [[ $current_boot -le 2000 ]]; then
        echo -e "Performance: ${GREEN}Good${NC} (< 2s)"
    elif [[ $current_boot -le 2500 ]]; then
        echo -e "Performance: ${YELLOW}Fair${NC} (2-2.5s)"
    else
        echo -e "Performance: ${RED}Poor${NC} (> 2.5s)"
    fi
    
    echo
    
    # Show recent boots
    if [[ -f "$DATA_DIR/boot_history.json" ]]; then
        echo "Recent boot times:"
        jq -r '.boots[-5:] | .[] | "  \(.timestamp) - \(.boot_type) - \(.boot_time_ms)ms"' "$DATA_DIR/boot_history.json" 2>/dev/null | tail -5 || echo "  Could not parse history"
    fi
    
    echo
    
    # Performance trend
    analyze_boot_trend
}

export_boot_data() {
    local output_file="$DATA_DIR/boot_data_export_$(date +%Y%m%d_%H%M%S).json"
    
    print_section "Exporting Boot Data"
    
    {
        echo "{"
        echo "  \"export_timestamp\": \"$(date -Iseconds)\","
        echo "  \"system_info\": {"
        echo "    \"kernel\": \"$(uname -r)\","
        echo "    \"hostname\": \"$(hostname)\","
        echo "    \"uptime\": \"$(uptime -p 2>/dev/null || uptime)\""
        echo "  },"
        echo "  \"current_status\": {"
        echo "    \"boot_time_ms\": $(get_boot_time),"
        echo "    \"boot_type\": \"$(get_boot_type)\""
        echo "  },"
        echo "  \"boot_history\": "
        
        if [[ -f "$DATA_DIR/boot_history.json" ]]; then
            jq '.' "$DATA_DIR/boot_history.json" 2>/dev/null || echo '"Could not parse history"'
        else
            echo '"No history available"'
        fi
        echo "}"
    } > "$output_file"
    
    echo "Data exported to: $output_file"
}

main() {
    print_header
    
    init_monitor
    
    # Check boot time on startup
    local current_boot=$(get_boot_time)
    local boot_type=$(get_boot_type)
    
    echo "Current boot analysis:"
    echo "Boot time: ${current_boot}ms"
    echo "Boot type: $boot_type"
    
    # Record current boot
    record_boot_measurement "$current_boot" "$boot_type"
    
    # Check for alerts
    check_performance_alerts "$current_boot"
    
    # Show current status
    show_current_status
    
    log "Boot monitoring setup complete"
}

# Parse command line arguments
case "${1:-}" in
    --help|-h)
        echo "Boot Performance Monitor"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "Options:"
        echo "  --help, -h       Show this help message"
        echo "  --monitor        Start continuous monitoring (daemon mode)"
        echo "  --status         Show current boot status"
        echo "  --report         Generate performance report"
        echo "  --export         Export boot data to JSON"
        echo "  --analyze        Analyze boot trend"
        echo
        echo "This tool monitors boot performance and alerts on issues."
        echo "Requires root privileges for full functionality."
        exit 0
        ;;
    --monitor)
        continuous_monitor
        ;;
    --status)
        show_current_status
        ;;
    --report)
        generate_performance_report
        ;;
    --export)
        export_boot_data
        ;;
    --analyze)
        analyze_boot_trend
        ;;
    --record)
        local boot_time=${2:-$(get_boot_time)}
        record_boot_measurement "$boot_time" "$(get_boot_type)"
        ;;
    *)
        main
        ;;
esac
