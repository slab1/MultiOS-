#!/bin/bash
# MultiOS Serial Console Helper
# Connect to MultiOS serial console for debugging

set -e

SERIAL_PORT=${1:-"/tmp/multios_serial"}
BAUD=${2:-"115200"}

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_usage() {
    echo "Usage: $0 [SERIAL_PORT] [BAUD]"
    echo ""
    echo "Arguments:"
    echo "  SERIAL_PORT Unix socket for serial connection [default: /tmp/multios_serial]"
    echo "  BAUD        Baud rate [default: 115200]"
    echo ""
    echo "Examples:"
    echo "  $0"
    echo "  $0 /tmp/aarch64_serial 115200"
}

# Check if serial socket exists
if [ ! -S "$SERIAL_PORT" ]; then
    print_message $RED "Error: Serial socket not found: $SERIAL_PORT"
    print_message $YELLOW "Make sure QEMU is running with: -serial unix:$SERIAL_PORT,server,nowait"
    print_message $YELLOW "Or check if QEMU is started with the correct socket path."
    exit 1
fi

print_message $GREEN "MultiOS Serial Console"
print_message $BLUE "======================"
print_message $BLUE "Socket: $SERIAL_PORT"
print_message $BLUE "Baud: $BAUD"
echo ""

print_message $YELLOW "Serial Console Commands:"
print_message $YELLOW "  Ctrl-C to disconnect"
print_message $YELLOW "  Ctrl-A then K to kill screen session (if using screen)"
echo ""

# Check available tools and use the best one
if command -v socat &> /dev/null; then
    print_message $GREEN "Using socat for serial console"
    print_message $GREEN "Press Ctrl-C to exit"
    echo ""
    socat -,raw,echo=0 UNIX-CONNECT:$SERIAL_PORT
elif command -v screen &> /dev/null; then
    print_message $GREEN "Using screen for serial console"
    print_message $YELLOW "Press Ctrl-A then K to exit"
    echo ""
    screen $SERIAL_PORT $BAUD
elif command -v minicom &> /dev/null; then
    print_message $GREEN "Using minicom for serial console"
    print_message $YELLOW "Configure minicom and press Ctrl-A X to exit"
    echo ""
    minicom -D $SERIAL_PORT -b $BAUD
else
    print_message $RED "Error: No serial console tool found"
    print_message $YELLOW "Install socat with: sudo apt install socat"
    print_message $YELLOW "Or install screen with: sudo apt install screen"
    exit 1
fi
