#!/bin/bash
# MultiOS Embedded/IoT Installation Script
# Version: 1.0.0
# Description: Install MultiOS for embedded systems and IoT devices

set -e

# Configuration
INSTALL_DIR="/opt/multios"
SYSTEMD_DIR="/etc/systemd/system"
CONFIG_DIR="/etc/multios"
KERNEL_DIR="/opt/multios/bin"
VERSION="1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
LOG_FILE="/var/log/multios_embed_install.log"
exec 1> >(tee -a "$LOG_FILE") 2>&1

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}" >&2
}

warning() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        error "This script must be run as root"
        exit 1
    fi
}

# Detect embedded platform
detect_platform() {
    log "Detecting embedded platform..."
    
    # Detect architecture
    ARCH=$(uname -m)
    case $ARCH in
        armv7l|aarch64)
            PLATFORM="arm"
            info "ARM platform detected"
            ;;
        riscv32|riscv64)
            PLATFORM="riscv"
            info "RISC-V platform detected"
            ;;
        x86_64|i386)
            PLATFORM="x86"
            info "x86 platform detected"
            ;;
        mips|mipsel)
            PLATFORM="mips"
            info "MIPS platform detected"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
    
    # Detect device type
    if [ -f "/sys/firmware/devicetree/base/model" ]; then
        DEVICE_MODEL=$(cat /sys/firmware/devicetree/base/model 2>/dev/null | tr ' ' '_')
        info "Device model: $DEVICE_MODEL"
    fi
    
    # Detect available hardware
    check_hardware
}

# Check hardware capabilities
check_hardware() {
    log "Checking hardware capabilities..."
    
    # Check for specific hardware
    if [ -d "/sys/class/leds" ]; then
        LED_COUNT=$(ls /sys/class/leds | wc -l)
        info "LEDs detected: $LED_COUNT"
    fi
    
    if [ -d "/sys/class/thermal" ]; then
        THERMAL_ZONES=$(ls /sys/class/thermal | grep -c thermal_zone || echo "0")
        info "Thermal zones: $THERMAL_ZONES"
    fi
    
    # Check for network interfaces
    NET_INTERFACES=$(ip link show | grep -c "^[0-9]" || echo "0")
    info "Network interfaces: $NET_INTERFACES"
    
    # Check for GPIO
    if [ -d "/sys/class/gpio" ]; then
        info "GPIO support detected"
    fi
    
    # Check for I2C/SPI
    if [ -d "/sys/bus/i2c" ]; then
        info "I2C bus detected"
    fi
    
    if [ -d "/sys/bus/spi" ]; then
        info "SPI bus detected"
    fi
}

# Check system requirements for embedded systems
check_requirements() {
    log "Checking embedded system requirements..."
    
    # Check memory (minimum 512MB for embedded)
    TOTAL_MEM=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    MIN_MEM=524288  # 512MB in KB
    
    if [ "$TOTAL_MEM" -lt "$MIN_MEM" ]; then
        error "Insufficient memory. Required: 512MB, Available: $((TOTAL_MEM/1024))MB"
        exit 1
    fi
    
    # Check storage (minimum 1GB)
    AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
    MIN_SPACE=1048576  # 1GB in KB
    
    if [ "$AVAILABLE_SPACE" -lt "$MIN_SPACE" ]; then
        error "Insufficient storage. Required: 1GB, Available: $((AVAILABLE_SPACE/1024/1024))GB"
        exit 1
    fi
    
    # Check for essential services
    if ! command -v systemctl >/dev/null 2>&1; then
        warning "systemd not available, using simple init system"
        USE_SIMPLE_INIT=true
    else
        USE_SIMPLE_INIT=false
    fi
    
    info "Embedded system requirements check passed"
}

# Create embedded-specific directories
create_directories() {
    log "Creating embedded directory structure..."
    
    mkdir -p "$INSTALL_DIR"/{bin,lib,config,logs,data,firmware,sensors}
    mkdir -p "$CONFIG_DIR"/{profiles,services,modules,gpio,i2c,spi}
    mkdir -p "$KERNEL_DIR"
    mkdir -p "/var/log/multios"
    mkdir -p "/tmp/multios"
    
    # Set permissions
    chmod 755 "$INSTALL_DIR" "$CONFIG_DIR" "$KERNEL_DIR"
    chmod 1777 "/tmp/multios"
    
    info "Embedded directory structure created"
}

# Install minimal kernel for embedded systems
install_kernel() {
    log "Installing MultiOS kernel for embedded systems..."
    
    # Use optimized build for embedded
    if [ -f "./kernel/target/release/multios-kernel" ]; then
        cp "./kernel/target/release/multios-kernel" "$KERNEL_DIR/"
    else
        warning "Kernel binary not found. Building from source..."
        cd kernel
        # Use minimal build configuration
        cargo build --release --features="minimal"
        cp "./target/release/multios-kernel" "$KERNEL_DIR/"
        cd ..
    fi
    
    chmod +x "$KERNEL_DIR/multios-kernel"
    
    # Create minimal configuration
    cat > "$KERNEL_DIR/kernel.conf" <<EOF
# MultiOS Embedded Kernel Configuration
# Version: $VERSION
# Platform: $PLATFORM

[Memory]
max_heap=128M
stack_size=8K
enable_paging=true

[Processes]
max_processes=256
enable_fork=false

[Features]
enable_networking=true
enable_filesystem=true
enable_gpio=true
enable_i2c=true
enable_spi=true
enable_uart=true

[Hardware]
platform=$PLATFORM
EOF
    
    info "Embedded kernel installed successfully"
}

# Install embedded bootloader
install_bootloader() {
    log "Installing embedded bootloader..."
    
    if [ -d "./bootloader" ]; then
        cp -r ./bootloader/* "$INSTALL_DIR/bootloader/"
    fi
    
    # Create embedded bootloader config
    cat > "$INSTALL_DIR/bootloader/embedded.conf" <<EOF
# MultiOS Embedded Bootloader Configuration
# Version: $VERSION

[DEFAULT]
timeout=3
default=multios
enable_watchdog=true

[MULTIOS]
kernel=$KERNEL_DIR/multios-kernel
cmdline=console=ttyS0,115200 root=/dev/mmcblk0p1 ro quiet

[SAFEMODE]
kernel=$KERNEL_DIR/multios-kernel
cmdline=console=ttyS0,115200 root=/dev/mmcblk0p1 ro single

[RECOVERY]
kernel=$KERNEL_DIR/multios-kernel
cmdline=console=ttyS0,115200 root=/dev/mmcblk0p1 ro recovery
EOF
}

# Install embedded libraries and drivers
install_embedded_libraries() {
    log "Installing embedded libraries and drivers..."
    
    # Copy core libraries
    cp -r ./libraries/* "$INSTALL_DIR/lib/"
    
    # Install hardware support
    if [ -d "./hardware_support" ]; then
        cp -r ./hardware_support/* "$INSTALL_DIR/lib/drivers/"
    fi
    
    # Install IoT frameworks
    if [ -d "./real_world/iot_projects" ]; then
        cp -r ./real_world/iot_projects/* "$INSTALL_DIR/sensors/"
    fi
}

# Create embedded systemd services (or simple init)
create_services() {
    log "Creating embedded system services..."
    
    if [ "$USE_SIMPLE_INIT" = true ]; then
        # Create simple init scripts
        cat > "$INSTALL_DIR/init/multios-init.sh" <<EOF
#!/bin/bash
# MultiOS Embedded Init Script

# Start core services
$KERNEL_DIR/multios-kernel --daemon

# Start IoT services
$INSTALL_DIR/bin/iot-gateway &
$INSTALL_DIR/bin/sensor-manager &

# Start web interface (if enabled)
if [ "\$ENABLE_WEB_UI" = "true" ]; then
    $INSTALL_DIR/bin/web-server --port=8080 &
fi

echo "MultiOS Embedded system started"
EOF
        chmod +x "$INSTALL_DIR/init/multios-init.sh"
        
        # Create service scripts
        mkdir -p "$INSTALL_DIR/services"
        
        # IoT Gateway service
        cat > "$INSTALL_DIR/services/iot-gateway.sh" <<EOF
#!/bin/bash
# IoT Gateway Service

while true; do
    # Collect sensor data
    \$INSTALL_DIR/bin/collect-sensor-data
    
    # Process data
    \$INSTALL_DIR/bin/process-iot-data
    
    # Send to cloud (if configured)
    if [ "\$CLOUD_ENDPOINT" ]; then
        \$INSTALL_DIR/bin/send-to-cloud \$CLOUD_ENDPOINT
    fi
    
    sleep 60  # Collect data every minute
done
EOF
        chmod +x "$INSTALL_DIR/services/*.sh"
        
    else
        # Use systemd for systems that support it
        cat > "$SYSTEMD_DIR/multios-kernel.service" <<EOF
[Unit]
Description=MultiOS Hybrid Microkernel (Embedded)
After=network.target

[Service]
Type=simple
User=root
ExecStart=$KERNEL_DIR/multios-kernel --embedded --daemon
Restart=always
RestartSec=10
StandardOutput=append:/var/log/multios/kernel.log
StandardError=append:/var/log/multios/kernel_error.log

[Install]
WantedBy=multi-user.target
EOF
        
        # IoT Gateway service
        cat > "$SYSTEMD_DIR/multios-iot-gateway.service" <<EOF
[Unit]
Description=MultiOS IoT Gateway
After=multios-kernel.service
Wants=multios-kernel.service

[Service]
Type=simple
ExecStart=/bin/bash -c 'while true; do \$INSTALL_DIR/bin/iot-gateway; sleep 60; done'
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
EOF
        
        systemctl enable multios-kernel.service
        systemctl enable multios-iot-gateway.service
    fi
    
    info "Embedded services configured"
}

# Create embedded configuration
create_config() {
    log "Creating embedded configuration..."
    
    # Main embedded configuration
    cat > "$CONFIG_DIR/embedded.conf" <<EOF
# MultiOS Embedded Configuration
# Version: $VERSION

[General]
version=$VERSION
install_date=$(date -u +%Y-%m-%dT%H:%M:%SZ)
install_type=embedded
platform=$PLATFORM
device_model=${DEVICE_MODEL:-unknown}

[System]
kernel_path=$KERNEL_DIR/multios-kernel
install_dir=$INSTALL_DIR
config_dir=$CONFIG_DIR
log_dir=/var/log/multios

[Hardware]
platform=$PLATFORM
architecture=$ARCH
memory_limit=512M
storage_limit=1G

[IoT]
enable_sensors=true
enable_actuators=true
enable_cloud_sync=true
data_collection_interval=60s
local_buffer_size=10M

[Sensors]
temperature=enabled
humidity=optional
pressure=optional
light=optional
motion=optional

[Actuators]
led_control=enabled
gpio_control=enabled
pwm_control=optional
servo_control=optional

[Network]
wifi=enabled
bluetooth=optional
ethernet=enabled
cellular=optional

[Security]
device_authentication=enabled
data_encryption=optional
secure_boot=optional
firmware_signature=enabled

[Power]
power_management=enabled
sleep_mode=enabled
wake_on_network=enabled
battery_monitoring=optional
EOF

    # Create device profile
    mkdir -p "$CONFIG_DIR/profiles/device"
    cat > "$CONFIG_DIR/profiles/device/profile.conf" <<EOF
[Device]
name=multios-embedded
type=iot_gateway
location=unknown

[Services]
sensor_collection=enabled
data_processing=enabled
cloud_sync=enabled
web_interface=optional

[Sensors]
enabled_sensors=temperature,humidity
sampling_rate=60s
data_precision=2

[Network]
wifi_ssid=
wifi_password=
cloud_endpoint=
api_key=
EOF
    
    # GPIO configuration
    mkdir -p "$CONFIG_DIR/gpio"
    cat > "$CONFIG_DIR/gpio/gpio.conf" <<EOF
# GPIO Configuration for MultiOS Embedded
# Version: $VERSION

[GPIO:17]
direction=output
initial_value=0
name=LED_STATUS

[GPIO:18]
direction=output
pwm=true
frequency=1000
name=PWM_OUTPUT

[GPIO:22]
direction=input
pull=up
name=BUTTON_INPUT

[GPIO:24]
direction=output
initial_value=1
name=RELAY_CONTROL
EOF
    
    info "Embedded configuration files created"
}

# Configure IoT sensors
configure_sensors() {
    log "Configuring IoT sensors..."
    
    mkdir -p "$CONFIG_DIR/sensors"
    
    # Temperature sensor configuration
    cat > "$CONFIG_DIR/sensors/temperature.conf" <<EOF
[SENSOR:temperature]
type=i2c
address=0x48
name=DS1621
enabled=true
sampling_rate=60s
precision=1
unit=celsius
EOF
    
    # Humidity sensor configuration
    cat > "$CONFIG_DIR/sensors/humidity.conf" <<EOF
[SENSOR:humidity]
type=i2c
address=0x40
name=HDC1080
enabled=true
sampling_rate=60s
precision=2
unit=percent
EOF
    
    # Motion sensor configuration
    cat > "$CONFIG_DIR/sensors/motion.conf" <<EOF
[SENSOR:motion]
type=gpio
pin=22
name=PIR_Motion
enabled=true
sensitivity=medium
debounce=2s
EOF
    
    info "IoT sensor configuration completed"
}

# Install web interface for monitoring
install_web_interface() {
    log "Installing embedded web interface..."
    
    mkdir -p "$INSTALL_DIR/web"
    
    # Create simple web interface
    cat > "$INSTALL_DIR/web/index.html" <<'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>MultiOS Embedded Device</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .sensor-data { background: #f0f0f0; padding: 10px; margin: 10px 0; }
        .status { color: green; font-weight: bold; }
        .warning { color: orange; font-weight: bold; }
        .error { color: red; font-weight: bold; }
    </style>
    <script>
        function updateData() {
            fetch('/api/sensors')
                .then(response => response.json())
                .then(data => {
                    document.getElementById('temperature').textContent = data.temperature + '°C';
                    document.getElementById('humidity').textContent = data.humidity + '%';
                    document.getElementById('uptime').textContent = data.uptime;
                });
        }
        setInterval(updateData, 5000);
        window.onload = updateData;
    </script>
</head>
<body>
    <h1>MultiOS Embedded Device Monitor</h1>
    <div class="sensor-data">
        <h3>Sensor Data</h3>
        <p>Temperature: <span id="temperature">--</span></p>
        <p>Humidity: <span id="humidity">--</span></p>
        <p>Uptime: <span id="uptime">--</span></p>
    </div>
    <div class="sensor-data">
        <h3>System Status</h3>
        <p class="status">System Online</p>
        <p>Platform: <span id="platform">Loading...</span></p>
    </div>
</body>
</html>
EOF
    
    info "Web interface installed"
}

# Run embedded-specific tests
run_tests() {
    log "Running embedded installation tests..."
    
    # Test kernel installation
    if [ -x "$KERNEL_DIR/multios-kernel" ]; then
        info "Kernel installation test: PASSED"
    else
        error "Kernel installation test: FAILED"
        return 1
    fi
    
    # Test configuration
    if [ -f "$CONFIG_DIR/embedded.conf" ]; then
        info "Embedded configuration test: PASSED"
    else
        error "Embedded configuration test: FAILED"
        return 1
    fi
    
    # Test GPIO configuration
    if [ -f "$CONFIG_DIR/gpio/gpio.conf" ]; then
        info "GPIO configuration test: PASSED"
    else
        error "GPIO configuration test: FAILED"
        return 1
    fi
    
    # Test sensor configuration
    if [ -f "$CONFIG_DIR/sensors/temperature.conf" ]; then
        info "Sensor configuration test: PASSED"
    else
        warning "Sensor configuration test: WARNING"
    fi
    
    # Test web interface
    if [ -f "$INSTALL_DIR/web/index.html" ]; then
        info "Web interface test: PASSED"
    else
        warning "Web interface test: WARNING"
    fi
    
    info "All embedded tests completed"
}

# Print embedded installation summary
print_summary() {
    echo
    log "MultiOS Embedded Installation Complete!"
    echo
    echo "Embedded Installation Summary:"
    echo "  Version: $VERSION"
    echo "  Platform: $PLATFORM"
    echo "  Architecture: $ARCH"
    echo "  Install Directory: $INSTALL_DIR"
    echo "  Configuration: $CONFIG_DIR"
    echo "  Kernel: $KERNEL_DIR/multios-kernel"
    echo "  Log Directory: /var/log/multios"
    echo
    echo "Embedded Features Installed:"
    echo "  - MultiOS Kernel (Embedded Mode)"
    echo "  - IoT Gateway Service"
    echo "  - Sensor Management"
    echo "  - GPIO Control"
    echo "  - Web Interface (if enabled)"
    echo "  - Cloud Sync Capability"
    echo
    echo "Next Steps:"
    echo "  1. Configure sensors and actuators"
    echo "  2. Set up network connectivity"
    echo "  3. Configure cloud endpoints (optional)"
    echo "  4. Test GPIO functionality"
    echo "  5. Access web interface at http://device:8080"
    echo
    echo "Useful Commands:"
    echo "  - View sensor data: cat $CONFIG_DIR/sensors/data.log"
    echo "  - Control GPIO: $INSTALL_DIR/bin/gpio-control 17 on"
    echo "  - Check status: $INSTALL_DIR/bin/device-status"
    echo "  - Update firmware: $INSTALL_DIR/bin/firmware-update"
    echo
    echo "For more information, see: $CONFIG_DIR/README_EMBEDDED.md"
    echo
}

# Main installation function
main() {
    echo "========================================"
    echo "  MultiOS Embedded/IoT Installation"
    echo "  Version: $VERSION"
    echo "========================================"
    echo
    
    check_root
    detect_platform
    check_requirements
    create_directories
    install_kernel
    install_bootloader
    install_embedded_libraries
    create_services
    create_config
    configure_sensors
    install_web_interface
    run_tests
    print_summary
}

# Run main function
main "$@"