#!/bin/bash

# QEMU Emulation Script for RISC-V IoT Projects
# This script emulates IoT projects using QEMU for testing and development

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Function to print colored output
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Default values
PROJECT_NAME=""
MACHINE_TYPE="virt"  # virt machine for RISC-V
MEMORY_SIZE="256M"
CPU_COUNT="2"
SERIAL_OUTPUT="true"
NETWORK="true"
SENSORS="true"
DEBUG="false"
GDB_PORT="1234"
MONITOR_PORT="4444"

# Help function
show_help() {
    cat << EOF
QEMU Emulation Script for RISC-V IoT Projects

Usage: $0 [OPTIONS] <project-name>

Options:
    -m, --machine TYPE        QEMU machine type (default: virt)
    -s, --memory SIZE         Memory size (default: 256M)
    -c, --cpus NUMBER         Number of CPUs (default: 2)
    -g, --gdb                 Enable GDB debugging on port 1234
    -d, --debug               Enable debug output
    -n, --no-serial           Disable serial output
    --no-network              Disable network emulation
    --no-sensors              Disable sensor simulation
    -h, --help                Show this help message

Examples:
    $0 smart_sensor_network
    $0 --gdb --debug industrial_iot_monitoring
    $0 --memory 512M --cpus 4 agricultural_iot
    $0 --no-serial home_automation

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--machine)
            MACHINE_TYPE="$2"
            shift 2
            ;;
        -s|--memory)
            MEMORY_SIZE="$2"
            shift 2
            ;;
        -c|--cpus)
            CPU_COUNT="$2"
            shift 2
            ;;
        -g|--gdb)
            DEBUG="true"
            GDB="true"
            shift
            ;;
        -d|--debug)
            DEBUG="true"
            shift
            ;;
        -n|--no-serial)
            SERIAL_OUTPUT="false"
            shift
            ;;
        --no-network)
            NETWORK="false"
            shift
            ;;
        --no-sensors)
            SENSORS="false"
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            PROJECT_NAME="$1"
            shift
            ;;
    esac
done

# Validate project name
if [[ -z "$PROJECT_NAME" ]]; then
    print_error "Project name is required"
    show_help
    exit 1
fi

print_info "🚀 Starting QEMU emulation for: $PROJECT_NAME"
print_info "Machine Type: $MACHINE_TYPE"
print_info "Memory: $MEMORY_SIZE"
print_info "CPUs: $CPU_COUNT"

# Find binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/../$PROJECT_NAME"

# Check for binary in different locations
BINARY_PATHS=(
    "$PROJECT_DIR/target/release/$PROJECT_NAME"
    "$PROJECT_DIR/target/debug/$PROJECT_NAME"
    "$PROJECT_DIR/target/riscv_build/$PROJECT_NAME"
    "$PROJECT_DIR/bin/$PROJECT_NAME"
)

BINARY=""
for path in "${BINARY_PATHS[@]}"; do
    if [[ -f "$path" ]]; then
        BINARY="$path"
        break
    fi
done

if [[ -z "$BINARY" ]]; then
    print_error "No binary found for project '$PROJECT_NAME'"
    print_info "Available binaries:"
    find "$PROJECT_DIR" -type f -executable 2>/dev/null || true
    print_info "Please build the project first: ./build_riscv.sh $PROJECT_NAME"
    exit 1
fi

print_success "Using binary: $BINARY"

# Check QEMU availability
if ! command -v qemu-system-riscv64 &> /dev/null; then
    print_error "QEMU for RISC-V is not installed"
    print_info "Install with: sudo apt-get install qemu-system-riscv"
    exit 1
fi

# Create sensor simulation (if enabled)
if [[ "$SENSORS" == "true" ]]; then
    print_info "Setting up sensor simulation..."
    
    # Create a simple sensor simulator script
    SENSOR_SCRIPT="/tmp/sensor_sim_$$.sh"
    cat > "$SENSOR_SCRIPT" << 'EOF'
#!/bin/bash
# IoT Sensor Simulation Script

while true; do
    # Generate random sensor data
    TEMP=$((2000 + RANDOM % 3000))  # Temperature in deci-celsius (20.00-50.00°C)
    HUMIDITY=$((3000 + RANDOM % 4000))  # Humidity in deci-percent (30.00-70.00%)
    MOTION=$(($RANDOM % 2))  # Motion detection (0 or 1)
    
    echo "SENSOR_DATA: temp=$TEMP,humidity=$HUMIDITY,motion=$MOTION"
    
    # Read commands from stdin and simulate sensor responses
    while read -t 0.1 line; do
        case "$line" in
            "GET_TEMP")
                echo "TEMP_RESPONSE:$TEMP"
                ;;
            "GET_HUMIDITY")
                echo "HUMIDITY_RESPONSE:$HUMIDITY"
                ;;
            "GET_MOTION")
                echo "MOTION_RESPONSE:$MOTION"
                ;;
        esac
    done
    
    sleep 1  # Update every second
done
EOF
    chmod +x "$SENSOR_SCRIPT"
    
    # Start sensor simulator in background
    "$SENSOR_SCRIPT" > /tmp/sensor_data_$$.fifo &
    SENSOR_PID=$!
    
    print_info "Sensor simulator running (PID: $SENSOR_PID)"
fi

# Build QEMU command
QEMU_CMD="qemu-system-riscv64"

# Basic parameters
QEMU_ARGS=(
    "-machine" "$MACHINE_TYPE"
    "-m" "$MEMORY_SIZE"
    "-smp" "cpu=$CPU_COUNT"
    "-nographic"  # No GUI, text only
    "-kernel" "$BINARY"
)

# Serial output
if [[ "$SERIAL_OUTPUT" == "true" ]]; then
    QEMU_ARGS+=("-serial" "stdio")
else
    QEMU_ARGS+=("-serial" "null")
fi

# GDB debugging
if [[ "$DEBUG" == "true" ]] || [[ "$GDB" == "true" ]]; then
    QEMU_ARGS+=("-s" "-S")  # Wait for GDB connection
    print_info "GDB debugging enabled on port $GDB_PORT"
    print_info "Connect with: riscv64-unknown-elf-gdb -tui $BINARY"
fi

# Network configuration
if [[ "$NETWORK" == "true" ]]; then
    print_info "Setting up network emulation..."
    
    # Create tap interface for network connectivity
    TAP_INTERFACE="tap0"
    if ! ip link show "$TAP_INTERFACE" &> /dev/null; then
        print_info "Creating tap interface $TAP_INTERFACE..."
        sudo ip tuntap add dev "$TAP_INTERFACE" mode tap
        sudo ip addr add 192.168.100.1/24 dev "$TAP_INTERFACE"
        sudo ip link set "$TAP_INTERFACE" up
    fi
    
    QEMU_ARGS+=(
        "-netdev" "tap,id=net0,ifname=$TAP_INTERFACE"
        "-device" "virtio-net,netdev=net0"
    )
    
    # Start simple HTTP server for testing
    HTTP_SERVER_PID=""
    if command -v python3 &> /dev/null; then
        cd "$PROJECT_DIR" && python3 -m http.server 8080 > /tmp/http_server.log 2>&1 &
        HTTP_SERVER_PID=$!
        print_info "HTTP server started on port 8080 (PID: $HTTP_SERVER_PID)"
    fi
fi

# Add device models for IoT peripherals
QEMU_ARGS+=(
    "-device" "virtio-gpu"
    "-device" "virtio-keyboard"
    "-device" "virtio-mouse"
)

# I2C/SPI device simulation
QEMU_ARGS+=("-device" "i2c-core,addr=0x42")

# Add real-time clock
QEMU_ARGS+=("-rtc" "clock=vm,base=utc")

# Debug output
if [[ "$DEBUG" == "true" ]]; then
    QEMU_ARGS+=("-d" "guest_errors")
fi

# Create monitor socket
MONITOR_SOCKET="/tmp/qemu_monitor_$$.sock"
QEMU_ARGS+=("-monitor" "unix:$MONITOR_SOCKET,server,nowait")

# Cleanup function
cleanup() {
    print_info "Cleaning up..."
    
    # Kill sensor simulator
    if [[ -n "$SENSOR_PID" ]] && kill -0 "$SENSOR_PID" 2>/dev/null; then
        kill "$SENSOR_PID" 2>/dev/null || true
    fi
    
    # Kill HTTP server
    if [[ -n "$HTTP_SERVER_PID" ]] && kill -0 "$HTTP_SERVER_PID" 2>/dev/null; then
        kill "$HTTP_SERVER_PID" 2>/dev/null || true
    fi
    
    # Remove temporary files
    rm -f "$SENSOR_SCRIPT" "$MONITOR_SOCKET"
    
    # Stop tap interface if we created it
    if [[ "$NETWORK" == "true" ]] && [[ "$TAP_INTERFACE" == "tap0" ]]; then
        sudo ip link set "$TAP_INTERFACE" down 2>/dev/null || true
        sudo ip link del "$TAP_INTERFACE" 2>/dev/null || true
    fi
}

# Set trap for cleanup
trap cleanup EXIT

# Print final command
print_info "QEMU Command:"
echo "  $QEMU_CMD ${QEMU_ARGS[*]}"
echo ""

# Save QEMU configuration
CONFIG_FILE="/tmp/qemu_config_$$.txt"
cat > "$CONFIG_FILE" << EOF
# QEMU Configuration for $PROJECT_NAME
QEMU Machine: $MACHINE_TYPE
Memory: $MEMORY_SIZE
CPUs: $CPU_COUNT
Network: $NETWORK
Sensors: $SENSORS
Debug: $DEBUG

Command: $QEMU_CMD ${QEMU_ARGS[*]}
Binary: $BINARY
EOF

print_success "🎬 Starting QEMU emulation..."
print_info "Configuration saved to: $CONFIG_FILE"
echo ""

# Display help
print_info "📺 Emulation Controls:"
print_info "  Ctrl+C: Exit emulation"
print_info "  Ctrl+A then X: Exit QEMU"
print_info "  Ctrl+A then C: Switch to QEMU monitor"
echo ""

# Start QEMU
if [[ "$DEBUG" == "true" ]] || [[ "$GDB" == "true" ]]; then
    print_info "QEMU waiting for GDB connection..."
    print_info "Connect with: riscv64-unknown-elf-gdb -tui $BINARY"
    print_info "Then run: target remote localhost:$GDB_PORT"
    echo ""
fi

# Run QEMU
"$QEMU_CMD" "${QEMU_ARGS[@]}"

print_success "✅ Emulation completed"
