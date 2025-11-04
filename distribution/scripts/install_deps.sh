#!/bin/bash

# IoT Projects RISC-V Dependencies Installation Script
# This script installs all necessary dependencies for IoT development on RISC-V

set -e

echo "🚀 Installing RISC-V IoT Development Dependencies..."
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    print_warning "This script is designed for Linux. Some features may not work on other platforms."
fi

# Install system dependencies
print_status "Installing system dependencies..."

# Update package lists
sudo apt-get update

# Install essential build tools
sudo apt-get install -y \
    build-essential \
    cmake \
    git \
    wget \
    curl \
    unzip \
    python3 \
    python3-pip \
    pkg-config \
    libssl-dev \
    libusb-1.0-0-dev \
    libudev-dev \
    libi2c-dev \
    i2c-tools \
    gdb-multiarch

# Install Rust toolchain
print_status "Installing Rust toolchain..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
else
    print_status "Rust is already installed"
fi

# Add RISC-V target
print_status "Adding RISC-V targets..."
rustup target add riscv64gc-unknown-none-elf
rustup target add riscv64imac-unknown-none-elf

# Install RISC-V GCC toolchain
print_status "Installing RISC-V GCC toolchain..."
if ! command -v riscv64-unknown-elf-gcc &> /dev/null; then
    wget https://github.com/riscv/riscv-gnu-toolchain/releases/download/v2023.03.17/riscv64-elf-gcc13.1.0-2023.03.17.tar.xz
    tar -xf riscv64-elf-gcc13.1.0-2023.03.17.tar.xz -C /tmp
    sudo mv /tmp/riscv64-elf-gcc13.1.0-2023.03.17 /opt/riscv64-elf
    export PATH=$PATH:/opt/riscv64-elf/bin
    echo 'export PATH=$PATH:/opt/riscv64-elf/bin' >> ~/.bashrc
else
    print_status "RISC-V GCC is already installed"
fi

# Install QEMU for RISC-V emulation
print_status "Installing QEMU for RISC-V..."
if ! command -v qemu-system-riscv64 &> /dev/null; then
    sudo apt-get install -y qemu-system-riscv
else
    print_status "QEMU is already installed"
fi

# Install OpenOCD for debugging
print_status "Installing OpenOCD..."
if ! command -v openocd &> /dev/null; then
    sudo apt-get install -y openocd
else
    print_status "OpenOCD is already installed"
fi

# Install Python dependencies for IoT projects
print_status "Installing Python dependencies..."
python3 -m pip install --user --upgrade pip
python3 -m pip install --user \
    requests \
    numpy \
    matplotlib \
    pandas \
    seaborn \
    pyserial \
    python-can \
    paho-mqtt \
    flask \
    fastapi \
    uvicorn \
    sqlalchemy \
    alembic \
    pytest \
    jupyter

# Install Docker for containerized deployments
if ! command -v docker &> /dev/null; then
    print_status "Installing Docker..."
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
else
    print_status "Docker is already installed"
fi

# Install Docker Compose
if ! command -v docker-compose &> /dev/null; then
    print_status "Installing Docker Compose..."
    sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
else
    print_status "Docker Compose is already installed"
fi

# Setup udev rules for RISC-V boards (if supported)
print_status "Setting up udev rules for RISC-V boards..."
sudo tee /etc/udev/rules.d/99-riscv-boards.rules > /dev/null <<EOF
# RISC-V Development Board Rules
SUBSYSTEM=="usb", ATTR{idVendor}=="0403", ATTR{idProduct}=="6010", MODE="0666", GROUP="plugdev"
SUBSYSTEM=="usb", ATTR{idVendor}=="0403", ATTR{idProduct}=="6014", MODE="0666", GROUP="plugdev"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Create project workspace
print_status "Creating project workspace..."
mkdir -p ~/iot_workspace/{bin,lib,logs,data,config}
mkdir -p ~/iot_workspace/projects/{build,test,deploy}

# Install additional IoT specific tools
print_status "Installing IoT development tools..."

# Install Mosquitto MQTT broker
sudo apt-get install -y mosquitto mosquitto-clients

# Install Node.js for IoT gateways
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

# Install Node-RED for IoT flows
sudo npm install -g node-red

print_status "✅ Installation complete!"
echo ""
echo "🚀 Next steps:"
echo "1. Source your bashrc: source ~/.bashrc"
echo "2. Run the build script: ./build_riscv.sh"
echo "3. Start with: ./emulate.sh <project-name>"
echo ""
echo "📚 Documentation available in docs/"
echo "🆘 For issues, check the troubleshooting guide"
echo ""
print_status "Installation finished successfully!"
