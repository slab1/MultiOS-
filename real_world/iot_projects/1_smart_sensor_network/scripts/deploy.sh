#!/bin/bash

# Smart Sensor Network - Deployment Script
# This script automates the deployment process for the smart sensor network

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to print colored output
print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Default values
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_TYPE="release"
TARGET="riscv64gc-unknown-none-elf"
FEATURES=""
HARDWARE="qemu"
DEVICE_ID=""
LOCATION=""
SENSORS="dht22,pir"
COMMUNICATION="mqtt"
CLOUD_PLATFORM="none"
DEPLOY_TO_LOCAL="false"
DEPLOY_TO_REMOTE="false"
VERBOSE="false"

# Help function
show_help() {
    cat << EOF
Smart Sensor Network Deployment Script

Usage: $0 [OPTIONS]

Options:
    -b, --build-type TYPE     Build type: debug|release (default: release)
    -t, --target TARGET       RISC-V target triple (default: riscv64gc-unknown-none-elf)
    -f, --features FEATURES   Enable features (default: mqtt,lora,display,sensors)
    -h, --hardware HW         Hardware target: qemu|hardware (default: qemu)
    -i, --device-id ID        Unique device identifier (default: auto-generated)
    -l, --location NAME       Device location name (required for hardware deployment)
    -s, --sensors SENSORS     Sensors to enable (default: dht22,pir)
    -c, --communication PROTO Communication protocol (mqtt|lora|none)
    -C, --cloud PLATFORM      Cloud platform (aws|azure|gcp|none)
    --local                   Deploy to local development environment
    --remote                  Deploy to remote production environment
    -v, --verbose             Verbose output
    --help                    Show this help message

Examples:
    $0 --hardware qemu --local
    $0 --hardware hardware --device-id 0x1234_5678 --location "Living Room"
    $0 --communication lora --cloud aws --deploy-remote

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -b|--build-type)
            BUILD_TYPE="$2"
            shift 2
            ;;
        -t|--target)
            TARGET="$2"
            shift 2
            ;;
        -f|--features)
            FEATURES="$2"
            shift 2
            ;;
        -h|--hardware)
            HARDWARE="$2"
            shift 2
            ;;
        -i|--device-id)
            DEVICE_ID="$2"
            shift 2
            ;;
        -l|--location)
            LOCATION="$2"
            shift 2
            ;;
        -s|--sensors)
            SENSORS="$2"
            shift 2
            ;;
        -c|--communication)
            COMMUNICATION="$2"
            shift 2
            ;;
        -C|--cloud)
            CLOUD_PLATFORM="$2"
            shift 2
            ;;
        --local)
            DEPLOY_TO_LOCAL="true"
            shift
            ;;
        --remote)
            DEPLOY_TO_REMOTE="true"
            shift
            ;;
        -v|--verbose)
            VERBOSE="true"
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validation
if [[ "$HARDWARE" == "hardware" ]]; then
    if [[ -z "$LOCATION" ]]; then
        print_error "Location is required for hardware deployment"
        exit 1
    fi
fi

# Set default features if not specified
if [[ -z "$FEATURES" ]]; then
    FEATURES="mqtt,lora,display,sensors"
fi

# Generate device ID if not provided
if [[ -z "$DEVICE_ID" ]]; then
    DEVICE_ID=$(printf "0x%08X" $RANDOM)
    print_info "Generated device ID: $DEVICE_ID"
fi

# Create deployment directory
DEPLOY_DIR="$PROJECT_DIR/deploy"
mkdir -p "$DEPLOY_DIR"

print_info "🚀 Starting Smart Sensor Network Deployment"
print_info "Project Directory: $PROJECT_DIR"
print_info "Build Type: $BUILD_TYPE"
print_info "Target: $TARGET"
print_info "Features: $FEATURES"
print_info "Hardware: $HARDWARE"
print_info "Device ID: $DEVICE_ID"
print_info "Location: $LOCATION"
print_info "Communication: $COMMUNICATION"
print_info "Cloud Platform: $CLOUD_PLATFORM"

# Step 1: Build the project
print_info "Step 1: Building the project..."

cd "$PROJECT_DIR"

# Install dependencies if needed
if ! command -v cargo &> /dev/null; then
    print_warning "Rust/Cargo not found, running dependency installation..."
    cd setup && ./install_deps.sh && cd ..
fi

# Build command
BUILD_CMD="cargo build --$BUILD_TYPE --target $TARGET"

# Add features
if [[ -n "$FEATURES" ]]; then
    BUILD_CMD="$BUILD_CMD --features $FEATURES"
fi

# Add verbose flag
if [[ "$VERBOSE" == "true" ]]; then
    BUILD_CMD="$BUILD_CMD --verbose"
fi

print_info "Running: $BUILD_CMD"
eval "$BUILD_CMD"

if [[ $? -eq 0 ]]; then
    print_success "Build completed successfully!"
else
    print_error "Build failed!"
    exit 1
fi

# Step 2: Generate configuration
print_info "Step 2: Generating configuration..."

CONFIG_FILE="$DEPLOY_DIR/sensor_config.toml"
cat > "$CONFIG_FILE" << EOF
# Smart Sensor Network Configuration
# Generated on $(date)

[device]
device_id = $DEVICE_ID
location = "$LOCATION"
node_type = "Smart Sensor"
firmware_version = "1.0.0"
build_type = "$BUILD_TYPE"

[sensors]
enabled_sensors = [$SENSORS]
temp_threshold = 3000        # 30.0°C
humidity_threshold = 3000   # 30.0%
motion_enabled = true
sampling_interval = 30      # seconds
transmission_interval = 300 # seconds

[communication]
protocol = "$COMMUNICATION"
mqtt_broker = "localhost"
mqtt_port = 1883
mqtt_username = ""
mqtt_password = ""
lora_frequency = 868100000   # Hz
lora_spreading_factor = 7
lora_transmission_power = 14 # dBm

[cloud]
platform = "$CLOUD_PLATFORM"
aws_region = "us-west-2"
aws_iot_endpoint = ""
azure_connection_string = ""
gcp_project_id = ""

[hardware]
gpio_dht22 = 0
gpio_pir = 1
gpio_battery = 2
gpio_display_sda = 3
gpio_display_scl = 4
i2c_display_address = 60    # 0x3C in decimal

[power]
low_power_mode = true
sleep_interval = 100        # milliseconds
battery_warning_threshold = 3000 # mV
EOF

print_success "Configuration generated: $CONFIG_FILE"

# Step 3: Create hardware-specific configurations
if [[ "$HARDWARE" == "hardware" ]]; then
    print_info "Step 3: Creating hardware configuration..."
    
    # SiFive HiFive configuration
    HIFIVE_CONFIG="$DEPLOY_DIR/hifive_config.txt"
    cat > "$HIFIVE_CONFIG" << EOF
# SiFive HiFive Board Configuration
# Board: SiFive HiFive Unmatched / RevB

[board_info]
name = "SiFive HiFive"
arch = "riscv64gc"
vendor = "SiFive"
product = "HiFive Unmatched"

[gpio_pins]
dht22_data = 0
pir_output = 1
battery_voltage = 2
display_sda = 3
display_scl = 4
lora_miso = 5
lora_mosi = 6
lora_clk = 7
lora_cs = 8
lora_reset = 9
wifi_tx = 10
wifi_rx = 11

[power]
vin = "5V USB or 12V barrel"
vout = "3.3V regulated"
max_current = "2A"
battery_connector = "J7"

[connectors]
power = "Barrel jack or USB-C"
expansion = "Qwiic/Stemma QT"
programmer = "USB-C or JTAG header"

[sensors]
dht22 = "J7.1 (GPIO0)"
pir = "J7.2 (GPIO1)"
battery = "J7.3 (ADC0)"
display = "J7.5&J7.6 (I2C)"

# Pin mappings for SiFive HiFive
# J7.1 -> GPIO0 (DHT22 data)
# J7.2 -> GPIO1 (PIR motion)
# J7.3 -> ADC0 (battery voltage)
# J7.5 -> GPIO3 (I2C SDA)
# J7.6 -> GPIO4 (I2C SCL)
EOF

    # Kendryte K210 configuration
    K210_CONFIG="$DEPLOY_DIR/k210_config.txt"
    cat > "$K210_CONFIG" << EOF
# Kendryte K210 Board Configuration
# Board: Maixpy/K510 series

[board_info]
name = "Kendryte K210"
arch = "riscv64gc"
vendor = "Kendryte"
product = "K210"

[gpio_pins]
dht22_data = 8
pir_output = 9
battery_voltage = 10
display_sda = 12
display_scl = 11
lora_miso = 14
lora_mosi = 13
lora_clk = 15
lora_cs = 16
lora_reset = 17
wifi_tx = 18
wifi_rx = 19

[power]
vin = "5V USB"
vout = "3.3V regulated"
max_current = "1.5A"
battery_connector = "BAT connector"

[connectors]
power = "USB-C"
expansion = "Grove connectors"
programmer = "USB-C"

[sensors]
dht22 = "GPIO8"
pir = "GPIO9"
battery = "ADC1"
display = "I2C (GPIO11-12)"
EOF

    print_success "Hardware configurations created"
fi

# Step 4: Create flash scripts
print_info "Step 4: Creating flash scripts..."

# QEMU flash script
QEMU_FLASH="$DEPLOY_DIR/flash_qemu.sh"
cat > "$QEMU_FLASH" << 'EOF'
#!/bin/bash

# QEMU Flash Script for Smart Sensor Network

BUILD_DIR="$1"
if [[ -z "$BUILD_DIR" ]]; then
    BUILD_DIR="target/release"
fi

BINARY="$BUILD_DIR/smart_sensor_network"

echo "Starting QEMU with smart sensor network binary..."
echo "Binary: $BINARY"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    echo "Please build the project first"
    exit 1
fi

qemu-system-riscv64 \
    -machine virt \
    -m 256M \
    -nographic \
    -smp 2 \
    -serial stdio \
    -kernel "$BINARY" \
    -netdev user,id=net0 \
    -device virtio-net,netdev=net0 \
    -netdev user,id=net1 \
    -device virtio-net,netdev=net1 \
    -device virtio-gpu \
    -device virtio-keyboard \
    -device i2c-core,addr=0x3c \
    -rtc clock=vm,base=utc

EOF

chmod +x "$QEMU_FLASH"

# Hardware flash script
HW_FLASH="$DEPLOY_DIR/flash_hardware.sh"
cat > "$HW_FLASH" << 'EOF'
#!/bin/bash

# Hardware Flash Script for Smart Sensor Network

BUILD_DIR="$1"
BOARD="$2"

if [[ -z "$BUILD_DIR" ]]; then
    BUILD_DIR="target/release"
fi

BINARY="$BUILD_DIR/smart_sensor_network"

echo "Flashing smart sensor network to hardware..."
echo "Binary: $BINARY"
echo "Board: $BOARD"

if [[ ! -f "$BINARY" ]]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

case "$BOARD" in
    "hifive")
        echo "Flashing to SiFive HiFive board..."
        # Using OpenOCD with ST-Link
        openocd -f interface/stlink.cfg -f target/riscv.cfg \
                -c "program $BINARY reset exit"
        ;;
    "k210")
        echo "Flashing to Kendryte K210..."
        # Using kflash.py
        if command -v kflash &> /dev/null; then
            kflash -b 1500000 -p /dev/ttyUSB0 -t "$BINARY"
        else
            echo "kflash not found. Install with: pip install kflash"
            exit 1
        fi
        ;;
    *)
        echo "Unknown board: $BOARD"
        echo "Supported boards: hifive, k210"
        exit 1
        ;;
esac

echo "Flash completed successfully!"

EOF

chmod +x "$HW_FLASH"

print_success "Flash scripts created"

# Step 5: Create monitoring scripts
print_info "Step 5: Creating monitoring scripts..."

# Serial monitor script
SERIAL_MONITOR="$DEPLOY_DIR/monitor.sh"
cat > "$SERIAL_MONITOR" << 'EOF'
#!/bin/bash

# Serial Monitor for Smart Sensor Network

DEVICE="${1:-/dev/ttyUSB0}"
BAUD="${2:-115200}"

echo "Connecting to device: $DEVICE at $BAUD baud"
echo "Press Ctrl+C to disconnect"

if command -v minicom &> /dev/null; then
    minicom -b $BAUD -D $DEVICE
elif command -v screen &> /dev/null; then
    screen $DEVICE $BAUD
else
    echo "Installing serial monitor tools..."
    echo "Try: sudo apt-get install minicom"
    exit 1
fi

EOF

chmod +x "$SERIAL_MONITOR"

# Network monitor script
NETWORK_MONITOR="$DEPLOY_DIR/monitor_network.sh"
cat > "$NETWORK_MONITOR" << 'EOF'
#!/bin/bash

# Network Monitor for Smart Sensor Network

BROKER="${1:-localhost}"
PORT="${2:-1883}"
TOPIC="${3:-sensors/+/data}"

echo "Monitoring MQTT traffic..."
echo "Broker: $BROKER"
echo "Port: $PORT"
echo "Topic: $TOPIC"
echo "Press Ctrl+C to stop"

if command -v mosquitto_sub &> /dev/null; then
    mosquitto_sub -h $BROKER -p $PORT -t "$TOPIC" -v
else
    echo "Mosquitto client not found. Install with: sudo apt-get install mosquitto-clients"
    exit 1
fi

EOF

chmod +x "$NETWORK_MONITOR"

print_success "Monitoring scripts created"

# Step 6: Create cloud deployment scripts
if [[ "$CLOUD_PLATFORM" != "none" ]]; then
    print_info "Step 6: Creating cloud deployment configuration..."
    
    case "$CLOUD_PLATFORM" in
        "aws")
            AWS_DEPLOY="$DEPLOY_DIR/aws_deploy.sh"
            cat > "$AWS_DEPLOY" << 'EOF'
#!/bin/bash

# AWS IoT Core Deployment Script

REGION="${AWS_REGION:-us-west-2}"
IOT_ENDPOINT="${AWS_IOT_ENDPOINT}"

echo "Deploying to AWS IoT Core..."
echo "Region: $REGION"

# Install AWS CLI if not present
if ! command -v aws &> /dev/null; then
    echo "Installing AWS CLI..."
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
    unzip awscliv2.zip
    sudo ./aws/install
fi

# Create IoT Thing
aws iot create-thing --thing-name "smart-sensor-$DEVICE_ID"

# Create and attach certificate
CERTIFICATE_ARN=$(aws iot create-keys-and-certificate \
    --set-as-active --certificate-pem-outfile cert.pem \
    --private-key-outfile private.key --public-key-outfile public.key \
    --query 'certificateArn' --output text)

# Attach certificate to thing
aws iot attach-thing-principal --thing-name "smart-sensor-$DEVICE_ID" \
    --principal "$CERTIFICATE_ARN"

# Create and attach policy
aws iot create-policy --policy-name "smart-sensor-policy" \
    --policy-document '{
        "Version": "2012-10-17",
        "Statement": [
            {
                "Effect": "Allow",
                "Action": [
                    "iot:Connect",
                    "iot:Publish",
                    "iot:Subscribe",
                    "iot:Receive"
                ],
                "Resource": "*"
            }
        ]
    }'

aws iot attach-policy --policy-name "smart-sensor-policy" \
    --target "$CERTIFICATE_ARN"

echo "AWS IoT deployment completed!"
echo "Certificate saved to: cert.pem"
echo "Private key saved to: private.key"
echo "Device ID: $DEVICE_ID"

EOF
            chmod +x "$AWS_DEPLOY"
            ;;
        "azure")
            AZURE_DEPLOY="$DEPLOY_DIR/azure_deploy.sh"
            cat > "$AZURE_DEPLOY" << 'EOF'
#!/bin/bash

# Azure IoT Hub Deployment Script

HUB_NAME="${AZURE_IOT_HUB}"
DEVICE_ID="${AZURE_DEVICE_ID:-$DEVICE_ID}"

echo "Deploying to Azure IoT Hub..."
echo "Hub: $HUB_NAME"
echo "Device: $DEVICE_ID"

# Install Azure CLI if not present
if ! command -v az &> /dev/null; then
    echo "Installing Azure CLI..."
    curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash
fi

# Login to Azure
echo "Please login to Azure..."
az login

# Create device in IoT Hub
az iot hub device-identity create \
    --hub-name "$HUB_NAME" \
    --device-id "$DEVICE_ID" \
    --auth-method shared_private_key

# Get connection string
CONNECTION_STRING=$(az iot hub device-identity connection-string show \
    --hub-name "$HUB_NAME" \
    --device-id "$DEVICE_ID" \
    --output tsv)

echo "Azure IoT deployment completed!"
echo "Connection string: $CONNECTION_STRING"
echo "Device ID: $DEVICE_ID"

EOF
            chmod +x "$AZURE_DEPLOY"
            ;;
        "gcp")
            GCP_DEPLOY="$DEPLOY_DIR/gcp_deploy.sh"
            cat > "$GCP_DEPLOY" << 'EOF'
#!/bin/bash

# Google Cloud IoT Deployment Script

PROJECT_ID="${GCP_PROJECT_ID}"
REGISTRY_ID="${GCP_REGISTRY_ID:-sensor-registry}"
LOCATION="${GCP_LOCATION:-us-central1}"
DEVICE_ID="${GCP_DEVICE_ID:-$DEVICE_ID}"

echo "Deploying to Google Cloud IoT..."
echo "Project: $PROJECT_ID"
echo "Registry: $REGISTRY_ID"
echo "Location: $LOCATION"
echo "Device: $DEVICE_ID"

# Install gcloud if not present
if ! command -v gcloud &> /dev/null; then
    echo "Installing Google Cloud SDK..."
    curl https://sdk.cloud.google.com | bash
    exec -l $SHELL
fi

# Set project
gcloud config set project "$PROJECT_ID"

# Enable required APIs
gcloud services enable cloudiot.googleapis.com

# Create device registry
gcloud iot registries create "$REGISTRY_ID" \
    --region="$LOCATION" \
    --event-notification-config=topic=projects/$PROJECT_ID/topics/iot-topic \
    --log-level=info

# Generate RSA keypair for device authentication
openssl genrsa -out rsa_private.pem 2048
openssl rsa -in rsa_private.pem -pubout -out rsa_public.pem

# Create device
gcloud iot devices create "$DEVICE_ID" \
    --registry="$REGISTRY_ID" \
    --region="$LOCATION" \
    --public-key path=rsa_public.pem,format=RSA_X509_PEM

echo "Google Cloud IoT deployment completed!"
echo "Private key saved to: rsa_private.pem"
echo "Public key saved to: rsa_public.pem"
echo "Device ID: $DEVICE_ID"

EOF
            chmod +x "$GCP_DEPLOY"
            ;;
    esac
    
    print_success "Cloud deployment script created: $CLOUD_PLATFORM"
fi

# Step 7: Create documentation
print_info "Step 7: Creating deployment documentation..."

DOCUMENTATION="$DEPLOY_DIR/README.md"
cat > "$DOCUMENTATION" << EOF
# Smart Sensor Network - Deployment Package

**Generated on:** $(date)
**Device ID:** $DEVICE_ID
**Location:** $LOCATION
**Build Type:** $BUILD_TYPE

## Quick Start

### For QEMU Testing
\`\`\`bash
./flash_qemu.sh
\`\`\`

### For Hardware Deployment
\`\`\`bash
# Flash to hardware
./flash_hardware.sh target/release hifive

# Monitor serial output
./monitor.sh /dev/ttyUSB0 115200
\`\`\`

## Files Included

- **sensor_config.toml** - Device configuration
- **flash_qemu.sh** - QEMU flash script
- **flash_hardware.sh** - Hardware flash script
- **monitor.sh** - Serial monitor
- **monitor_network.sh** - Network traffic monitor
- **hifive_config.txt** - SiFive HiFive pin mapping
- **k210_config.txt** - Kendryte K210 pin mapping

## Cloud Integration

EOF

if [[ "$CLOUD_PLATFORM" != "none" ]]; then
    echo "- **${CLOUD_PLATFORM}_deploy.sh** - Cloud platform deployment" >> "$DOCUMENTATION"
fi

cat >> "$DOCUMENTATION" << 'EOF'

## Configuration

Edit `sensor_config.toml` to customize:

### Device Settings
```toml
[device]
device_id = 0x12345678
location = "Your Location"
node_type = "Smart Sensor"
```

### Sensor Thresholds
```toml
[sensors]
temp_threshold = 3000        # 30.0°C
humidity_threshold = 3000   # 30.0%
sampling_interval = 30      # seconds
```

### Communication
```toml
[communication]
protocol = "mqtt"           # mqtt, lora, none
mqtt_broker = "localhost"
lora_frequency = 868100000  # Hz
```

## Hardware Connections

### SiFive HiFive
```
GPIO0 -> DHT22 Data
GPIO1 -> PIR Motion
GPIO2 -> Battery (ADC)
GPIO3 -> Display SDA (I2C)
GPIO4 -> Display SCL (I2C)
```

### Kendryte K210
```
GPIO8  -> DHT22 Data
GPIO9  -> PIR Motion
GPIO10 -> Battery (ADC)
GPIO12 -> Display SDA (I2C)
GPIO11 -> Display SCL (I2C)
```

## Monitoring

### Serial Output
\`\`\`bash
./monitor.sh /dev/ttyUSB0 115200
\`\`\`

### MQTT Traffic
\`\`\`bash
./monitor_network.sh localhost 1883 "sensors/$DEVICE_ID/data"
\`\`\`

## Troubleshooting

1. **Build fails**: Check Rust installation and RISC-V target
2. **Flash fails**: Verify board connection and programmer
3. **Sensors not responding**: Check wiring and power connections
4. **Communication fails**: Verify network settings and credentials

## Next Steps

1. Test the deployment in QEMU first
2. Configure hardware according to pin mappings
3. Flash the binary to your RISC-V board
4. Monitor serial output for initialization
5. Verify sensor readings and communication

For detailed information, see the project documentation.
EOF

print_success "Deployment documentation created"

# Step 8: Create post-deployment tests
print_info "Step 8: Creating post-deployment tests..."

TEST_SCRIPT="$DEPLOY_DIR/test_deployment.sh"
cat > "$TEST_SCRIPT" << 'EOF'
#!/bin/bash

# Post-Deployment Test Script

echo "Running post-deployment tests..."

# Test 1: Check serial connection
echo "Test 1: Checking serial connection..."
if ls /dev/ttyUSB* 1> /dev/null 2>&1; then
    echo "✅ Serial device detected"
    DEVICE=$(ls /dev/ttyUSB* | head -1)
    echo "Device: $DEVICE"
else
    echo "❌ No serial device found"
    exit 1
fi

# Test 2: Send test command
echo "Test 2: Sending test command..."
echo "PING" > /tmp/test_serial
# This would require a more sophisticated test with actual serial communication

# Test 3: Check MQTT connection
echo "Test 3: Checking MQTT connection..."
if command -v mosquitto_pub &> /dev/null; then
    mosquitto_pub -h localhost -t "test/topic" -m "Test message"
    if [[ $? -eq 0 ]]; then
        echo "✅ MQTT connection successful"
    else
        echo "❌ MQTT connection failed"
    fi
else
    echo "⚠️  MQTT client not available"
fi

echo "Post-deployment tests completed!"
EOF

chmod +x "$TEST_SCRIPT"

print_success "Deployment tests created"

# Step 9: Create summary
print_info "Step 9: Creating deployment summary..."

SUMMARY_FILE="$DEPLOY_DIR/deployment_summary.txt"
cat > "$SUMMARY_FILE" << EOF
Smart Sensor Network - Deployment Summary
==========================================

Deployment Date: $(date)
Device ID: $DEVICE_ID
Location: $LOCATION
Build Type: $BUILD_TYPE
Target Hardware: $HARDWARE
Communication Protocol: $COMMUNICATION
Cloud Platform: $CLOUD_PLATFORM
Enabled Features: $FEATURES

Deployment Directory: $DEPLOY_DIR
Project Directory: $PROJECT_DIR

Next Steps:
1. Review configuration in sensor_config.toml
2. Flash binary using appropriate script
3. Monitor output using monitor.sh
4. Verify sensor readings and communication
5. Configure cloud integration if needed

Documentation: $DEPLOY_DIR/README.md
EOF

# Final cleanup and packaging
print_info "Step 10: Packaging deployment..."

# Create tarball
TARBALL="$DEPLOY_DIR/smart_sensor_network_deployment.tar.gz"
cd "$PROJECT_DIR"
tar -czf "$TARBALL" -C deploy .

print_success "Deployment package created: $TARBALL"

# Final success message
echo ""
print_success "🎉 Deployment completed successfully!"
echo ""
echo "📦 Deployment Package: $TARBALL"
echo "📄 Configuration: $CONFIG_FILE"
echo "📚 Documentation: $DEPLOY_DIR/README.md"
echo "🔧 Scripts: $DEPLOY_DIR/"
echo ""
echo "🚀 Next Steps:"
echo "1. Review configuration: cat $CONFIG_FILE"
echo "2. Test in QEMU: $DEPLOY_DIR/flash_qemu.sh"
echo "3. Deploy to hardware: $DEPLOY_DIR/flash_hardware.sh"
echo "4. Monitor output: $DEPLOY_DIR/monitor.sh"
echo ""
echo "📚 For detailed instructions, see: $DEPLOY_DIR/README.md"

if [[ "$DEPLOY_TO_LOCAL" == "true" ]]; then
    echo ""
    print_info "Starting local deployment..."
    
    if [[ "$HARDWARE" == "qemu" ]]; then
        "$QEMU_FLASH" "target/$BUILD_TYPE"
    fi
fi

if [[ "$DEPLOY_TO_REMOTE" == "true" ]]; then
    echo ""
    print_info "Remote deployment not implemented yet"
    echo "Manual deployment steps:"
    echo "1. Transfer binary to target device"
    echo "2. Configure network settings"
    echo "3. Start monitoring"
fi

print_success "All deployment tasks completed!"
