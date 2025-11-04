#!/bin/bash

# MultiOS CI/CD Notification System
# Sends notifications about pipeline status, failures, and important events

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPORTS_DIR="/workspace/reports"
LOG_DIR="/workspace/logs"
CONFIG_DIR="/workspace/config"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Notification channels
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"
DISCORD_WEBHOOK="${DISCORD_WEBHOOK:-}"
EMAIL_RECIPIENTS="${EMAIL_RECIPIENTS:-}"
GITHUB_STATUS_URL="${GITHUB_STATUS_URL:-}"

# Ensure directories exist
mkdir -p "$REPORTS_DIR" "$LOG_DIR"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Generate notification content
generate_notification_content() {
    local status=$1
    local pipeline_type=$2
    
    log "Generating notification content for $status $pipeline_type pipeline"
    
    local title=""
    local message=""
    local color=""
    local details=""
    
    case "$status" in
        "SUCCESS")
            title="✅ MultiOS Pipeline Success"
            message="All tests passed and quality gates met!"
            color="good"
            ;;
        "FAILURE")
            title="❌ MultiOS Pipeline Failed"
            message="Pipeline execution failed - immediate attention required"
            color="danger"
            ;;
        "PARTIAL")
            title="⚠️ MultiOS Pipeline Partial Success"
            message="Some components passed, others failed"
            color="warning"
            ;;
        "QUALITY_GATE_FAIL")
            title="🚫 Quality Gate Failure"
            message="Quality gates not met - build rejected"
            color="danger"
            ;;
    esac
    
    # Add pipeline type context
    case "$pipeline_type" in
        "CI")
            title="$title (Continuous Integration)"
            ;;
        "CD")
            title="$title (Continuous Deployment)"
            ;;
        "RELEASE")
            title="$title (Release Pipeline)"
            ;;
    esac
    
    # Collect pipeline details
    local commit_sha=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    local branch_name=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    local commit_message=$(git log -1 --pretty=%B 2>/dev/null | head -1 || echo "unknown")
    local build_number=${GITHUB_RUN_NUMBER:-"local"}
    
    # Get test results summary
    local test_summary=$(get_test_summary)
    local quality_summary=$(get_quality_summary)
    local performance_summary=$(get_performance_summary)
    
    # Generate detailed message
    cat << EOF
**$title**

📋 **Pipeline Information:**
- **Status:** $status
- **Branch:** $branch_name
- **Commit:** ${commit_sha:0:8}
- **Build:** #$build_number
- **Time:** $(date -Iseconds)

📝 **Latest Commit:**
\`\`\`
$commit_message
\`\`\`

🧪 **Test Results:**
$test_summary

📊 **Quality Gate Status:**
$quality_summary

⚡ **Performance Metrics:**
$performance_summary

🔗 **Artifacts:**
- Build logs: $LOG_DIR/
- Test reports: $REPORTS_DIR/
- Quality reports: $REPORTS_DIR/

EOF

}

# Get test results summary
get_test_summary() {
    local summary="No test data available"
    
    if [ -f "$REPORTS_DIR/test_analysis_latest.json" ]; then
        local total=$(jq -r '.summary.total_tests' "$REPORTS_DIR/test_analysis_latest.json" 2>/dev/null || echo "0")
        local passed=$(jq -r '.summary.passed_tests' "$REPORTS_DIR/test_analysis_latest.json" 2>/dev/null || echo "0")
        local pass_rate=$(jq -r '.summary.pass_rate' "$REPORTS_DIR/test_analysis_latest.json" 2>/dev/null || echo "0")
        
        summary="- **Total Tests:** $total
- **Passed:** $passed
- **Pass Rate:** $pass_rate%"
    fi
    
    echo "$summary"
}

# Get quality gate summary
get_quality_summary() {
    local summary="No quality data available"
    
    if [ -f "$REPORTS_DIR/quality_gate_latest.json" ]; then
        local status=$(jq -r '.overall_status' "$REPORTS_DIR/quality_gate_latest.json" 2>/dev/null || echo "UNKNOWN")
        local passed=$(jq -r '.gates_passed' "$REPORTS_DIR/quality_gate_latest.json" 2>/dev/null || echo "0")
        local total=$(jq -r '.gates_checked' "$REPORTS_DIR/quality_gate_latest.json" 2>/dev/null || echo "0")
        local passed_gates=$(jq -r '.gate_results[] | select(.status == "PASS") | .gate' "$REPORTS_DIR/quality_gate_latest.json" 2>/dev/null | tr '\n' ',' | sed 's/,$//')
        
        summary="- **Overall Status:** $status
- **Gates Passed:** $passed/$total
- **Passed Gates:** ${passed_gates:-"none"}"
        
        # Add failed gates if any
        local failed_gates=$(jq -r '.gate_results[] | select(.status == "FAIL") | .gate' "$REPORTS_DIR/quality_gate_latest.json" 2>/dev/null | tr '\n' ',' | sed 's/,$//')
        if [ -n "$failed_gates" ]; then
            summary="$summary
- **Failed Gates:** $failed_gates"
        fi
    fi
    
    echo "$summary"
}

# Get performance summary
get_performance_summary() {
    local summary="No performance data available"
    
    # Collect performance data for all architectures
    local perf_data=""
    for arch in x86_64 arm64 riscv64; do
        local comparison_file="$REPORTS_DIR/benchmark_comparison_${arch}_latest.json"
        if [ -f "$comparison_file" ]; then
            local boot_time=$(jq -r '.current.boot_time_ms' "$comparison_file" 2>/dev/null || echo "N/A")
            local cpu_score=$(jq -r '.current.cpu_score' "$comparison_file" 2>/dev/null || echo "N/A")
            local boot_regr=$(jq -r '.regression_flags.boot_time_regression' "$comparison_file" 2>/dev/null || echo "false")
            
            local status_icon="✅"
            if [ "$boot_regr" = "true" ]; then
                status_icon="⚠️"
            fi
            
            perf_data="$perf_data
- **$arch:** Boot ${boot_time}ms, CPU ${cpu_score} $status_icon"
        fi
    done
    
    if [ -n "$perf_data" ]; then
        summary="- **Architecture Performance:**$perf_data"
    fi
    
    echo "$summary"
}

# Send Slack notification
send_slack_notification() {
    local content=$1
    
    if [ -z "$SLACK_WEBHOOK" ]; then
        warning "Slack webhook not configured"
        return 1
    fi
    
    log "Sending Slack notification..."
    
    # Create Slack payload
    local payload=$(cat << EOF
{
    "text": "MultiOS CI/CD Pipeline Update",
    "attachments": [
        {
            "color": "good",
            "text": $(echo "$content" | jq -Rs .),
            "mrkdwn_in": ["text"],
            "footer": "MultiOS CI/CD Pipeline",
            "footer_icon": "https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png",
            "ts": $(date +%s)
        }
    ]
}
EOF
)
    
    # Send to Slack
    local response=$(curl -s -X POST -H 'Content-type: application/json' \
        --data "$payload" \
        "$SLACK_WEBHOOK" 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        success "Slack notification sent"
    else
        error "Failed to send Slack notification"
    fi
}

# Send Discord notification
send_discord_notification() {
    local content=$1
    
    if [ -z "$DISCORD_WEBHOOK" ]; then
        warning "Discord webhook not configured"
        return 1
    fi
    
    log "Sending Discord notification..."
    
    # Create Discord payload
    local payload=$(cat << EOF
{
    "embeds": [
        {
            "title": "MultiOS CI/CD Pipeline Update",
            "description": $(echo "$content" | jq -Rs .),
            "color": 3447003,
            "footer": {
                "text": "MultiOS CI/CD Pipeline"
            },
            "timestamp": "$(date -Iseconds)"
        }
    ]
}
EOF
)
    
    # Send to Discord
    local response=$(curl -s -X POST -H 'Content-type: application/json' \
        --data "$payload" \
        "$DISCORD_WEBHOOK" 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        success "Discord notification sent"
    else
        error "Failed to send Discord notification"
    fi
}

# Send email notification
send_email_notification() {
    local content=$1
    local subject=$2
    
    if [ -z "$EMAIL_RECIPIENTS" ]; then
        warning "Email recipients not configured"
        return 1
    fi
    
    log "Sending email notification..."
    
    # Create email content
    local email_content=$(cat << EOF
From: MultiOS CI/CD Pipeline <noreply@multios.dev>
To: $EMAIL_RECIPIENTS
Subject: $subject
Content-Type: text/html; charset=UTF-8

<pre style="font-family: monospace;">$content</pre>

---
This is an automated message from the MultiOS CI/CD Pipeline.
EOF
)
    
    # Send email using sendmail (if available) or other method
    if command -v sendmail >/dev/null 2>&1; then
        echo "$email_content" | sendmail "$EMAIL_RECIPIENTS"
        success "Email notification sent"
    elif command -v mail >/dev/null 2>&1; then
        echo "$content" | mail -s "$subject" "$EMAIL_RECIPIENTS"
        success "Email notification sent"
    else
        warning "Neither sendmail nor mail command available"
        return 1
    fi
}

# Update GitHub commit status
update_github_status() {
    local status=$1
    local description=$2
    local context=$3
    
    if [ -z "$GITHUB_STATUS_URL" ]; then
        warning "GitHub status URL not configured"
        return 1
    fi
    
    log "Updating GitHub commit status..."
    
    # Get commit SHA
    local commit_sha=$(git rev-parse HEAD 2>/dev/null || echo "")
    
    if [ -z "$commit_sha" ]; then
        warning "Could not determine commit SHA"
        return 1
    fi
    
    # Create status payload
    local payload=$(cat << EOF
{
    "state": "$status",
    "description": "$description",
    "context": "$context",
    "target_url": "https://github.com/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"
}
EOF
)
    
    # Update status on GitHub
    local response=$(curl -s -X POST \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.v3+json" \
        --data "$payload" \
        "$GITHUB_STATUS_URL" 2>/dev/null)
    
    if [ $? -eq 0 ]; then
        success "GitHub commit status updated"
    else
        error "Failed to update GitHub commit status"
    fi
}

# Save notification log
save_notification_log() {
    local status=$1
    local content=$2
    local log_file="$LOG_DIR/notifications_$TIMESTAMP.log"
    
    log "Saving notification log..."
    
    cat > "$log_file" << EOF
# MultiOS CI/CD Notification Log
Timestamp: $(date -Iseconds)
Pipeline Status: $status

Notification Content:
$content

Channels Attempted:
EOF
    
    [ -n "$SLACK_WEBHOOK" ] && echo "- Slack: configured" >> "$log_file" || echo "- Slack: not configured" >> "$log_file"
    [ -n "$DISCORD_WEBHOOK" ] && echo "- Discord: configured" >> "$log_file" || echo "- Discord: not configured" >> "$log_file"
    [ -n "$EMAIL_RECIPIENTS" ] && echo "- Email: configured" >> "$log_file" || echo "- Email: not configured" >> "$log_file"
    
    success "Notification log saved: $log_file"
}

# Main notification function
main() {
    local pipeline_status=${1:-"UNKNOWN"}
    local pipeline_type=${2:-"CI"}
    
    log "Starting MultiOS Notification System"
    log "Pipeline Status: $pipeline_status"
    log "Pipeline Type: $pipeline_type"
    
    # Generate notification content
    local notification_content=$(generate_notification_content "$pipeline_status" "$pipeline_type")
    
    # Determine subject line
    local subject=""
    case "$pipeline_status" in
        "SUCCESS")
            subject="MultiOS $pipeline_type Pipeline - SUCCESS ✅"
            ;;
        "FAILURE")
            subject="MultiOS $pipeline_type Pipeline - FAILURE ❌"
            ;;
        "PARTIAL")
            subject="MultiOS $pipeline_type Pipeline - PARTIAL ⚠️"
            ;;
        "QUALITY_GATE_FAIL")
            subject="MultiOS $pipeline_type Pipeline - Quality Gates Failed 🚫"
            ;;
        *)
            subject="MultiOS $pipeline_type Pipeline - $pipeline_status"
            ;;
    esac
    
    # Send notifications to all configured channels
    local notification_sent=false
    
    if [ -n "$SLACK_WEBHOOK" ]; then
        send_slack_notification "$notification_content"
        notification_sent=true
    fi
    
    if [ -n "$DISCORD_WEBHOOK" ]; then
        send_discord_notification "$notification_content"
        notification_sent=true
    fi
    
    if [ -n "$EMAIL_RECIPIENTS" ]; then
        send_email_notification "$notification_content" "$subject"
        notification_sent=true
    fi
    
    # Update GitHub status
    if [ -n "$GITHUB_STATUS_URL" ] && [ -n "$GITHUB_TOKEN" ]; then
        local gh_status="pending"
        local gh_description="Pipeline $pipeline_status"
        local gh_context="MultiOS CI/CD"
        
        case "$pipeline_status" in
            "SUCCESS")
                gh_status="success"
                gh_description="All tests passed - build successful"
                ;;
            "FAILURE"|"QUALITY_GATE_FAIL")
                gh_status="failure"
                gh_description="Pipeline failed - check logs"
                ;;
            "PARTIAL")
                gh_status="failure"
                gh_description="Partial success - some tests failed"
                ;;
        esac
        
        update_github_status "$gh_status" "$gh_description" "$gh_context"
        notification_sent=true
    fi
    
    # Save notification log
    save_notification_log "$pipeline_status" "$notification_content"
    
    if [ "$notification_sent" = true ]; then
        success "Notifications sent successfully"
    else
        warning "No notification channels configured"
    fi
    
    # Print notification content to console
    echo
    echo "Notification Content:"
    echo "===================="
    echo "$notification_content"
    echo "===================="
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [STATUS] [TYPE]

STATUS: SUCCESS|FAILURE|PARTIAL|QUALITY_GATE_FAIL|UNKNOWN (default: UNKNOWN)
TYPE: CI|CD|RELEASE (default: CI)

Environment Variables:
  SLACK_WEBHOOK       - Slack webhook URL for notifications
  DISCORD_WEBHOOK     - Discord webhook URL for notifications
  EMAIL_RECIPIENTS    - Comma-separated list of email addresses
  GITHUB_STATUS_URL   - GitHub status API URL
  GITHUB_TOKEN        - GitHub token for status updates
  GITHUB_RUN_ID       - GitHub Actions run ID
  GITHUB_REPOSITORY   - GitHub repository name

Examples:
  $0 SUCCESS CI
  $0 FAILURE CI
  $0 PARTIAL RELEASE
EOF
}

# Parse command line arguments
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    usage
    exit 0
fi

main "$@"