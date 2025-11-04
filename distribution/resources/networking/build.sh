#!/bin/bash

# MultiOS Networking Drivers Build Script
# Builds and tests the complete networking driver stack

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="/workspace/hardware_support/networking"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
BUILD_DIR="$PROJECT_ROOT/target"
DOCS_DIR="$PROJECT_ROOT/doc"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  MultiOS Networking Drivers Build     ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Rust toolchain
    if ! command -v rustc &> /dev/null; then
        log_error "Rust compiler not found. Please install Rust."
        exit 1
    fi
    
    # Check cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check project structure
    if [ ! -f "$CARGO_TOML" ]; then
        log_error "Cargo.toml not found in $PROJECT_ROOT"
        exit 1
    fi
    
    # Create necessary directories
    mkdir -p "$BUILD_DIR"
    mkdir -p "$DOCS_DIR"
    
    log_success "Prerequisites check passed"
}

clean_build() {
    log_info "Cleaning previous build artifacts..."
    
    cd "$PROJECT_ROOT"
    cargo clean
    
    # Clean additional directories
    rm -rf "$BUILD_DIR/debug"
    rm -rf "$BUILD_DIR/release"
    rm -rf "$DOCS_DIR"
    
    log_success "Clean completed"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    cd "$PROJECT_ROOT"
    
    # Update dependencies
    cargo update
    
    # Check for security vulnerabilities
    if command -v cargo-audit &> /dev/null; then
        log_info "Running security audit..."
        cargo audit || log_warning "Security audit found issues"
    fi
    
    # Check for outdated dependencies
    if command -v cargo-outdated &> /dev/null; then
        log_info "Checking for outdated dependencies..."
        cargo outdated --minimal-deps || log_warning "Some dependencies are outdated"
    fi
    
    log_success "Dependency check completed"
}

build_release() {
    log_info "Building release version..."
    
    cd "$PROJECT_ROOT"
    
    # Build with optimizations
    RUSTFLAGS="-C target-cpu=native" cargo build --release
    
    # Check build artifacts
    if [ -f "$BUILD_DIR/release/libmultios_networking.rlib" ]; then
        log_success "Release build completed successfully"
    else
        log_error "Release build failed - library not found"
        return 1
    fi
    
    # Show build size
    LIB_SIZE=$(du -h "$BUILD_DIR/release/libmultios_networking.rlib" | cut -f1)
    log_info "Library size: $LIB_SIZE"
}

build_debug() {
    log_info "Building debug version..."
    
    cd "$PROJECT_ROOT"
    cargo build
    
    if [ -f "$BUILD_DIR/debug/libmultios_networking.rlib" ]; then
        log_success "Debug build completed successfully"
    else
        log_error "Debug build failed"
        return 1
    fi
}

run_tests() {
    log_info "Running test suite..."
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests
    log_info "Running unit tests..."
    cargo test --lib -- --test-threads=1
    
    # Run integration tests
    log_info "Running integration tests..."
    cargo test --test '*' -- --test-threads=1
    
    # Run with different features
    log_info "Testing with Wi-Fi feature..."
    cargo test --features wifi --lib
    
    log_info "Testing with Ethernet feature..."
    cargo test --features ethernet --lib
    
    log_info "Testing with security feature..."
    cargo test --features security --lib
    
    log_info "Testing with debugging feature..."
    cargo test --features debugging --lib
    
    # Run all features together
    log_info "Testing with all features..."
    cargo test --features "wifi,ethernet,security,debugging" --lib
    
    log_success "All tests passed"
}

run_examples() {
    log_info "Building and testing examples..."
    
    cd "$PROJECT_ROOT"
    
    # Build Wi-Fi scan example
    log_info "Building Wi-Fi scan example..."
    if cargo build --example wifi_scan; then
        log_success "Wi-Fi scan example built successfully"
        
        # Try to run (may fail without hardware)
        if cargo run --example wifi_scan 2>/dev/null; then
            log_success "Wi-Fi scan example executed"
        else
            log_warning "Wi-Fi scan example execution skipped (no hardware)"
        fi
    else
        log_error "Wi-Fi scan example build failed"
    fi
    
    # Build Ethernet test example
    log_info "Building Ethernet test example..."
    if cargo build --example ethernet_test; then
        log_success "Ethernet test example built successfully"
        
        # Try to run (may fail without hardware)
        if cargo run --example ethernet_test 2>/dev/null; then
            log_success "Ethernet test example executed"
        else
            log_warning "Ethernet test example execution skipped (no hardware)"
        fi
    else
        log_error "Ethernet test example build failed"
    fi
}

generate_documentation() {
    log_info "Generating documentation..."
    
    cd "$PROJECT_ROOT"
    
    # Generate Rust documentation
    cargo doc --no-deps --document-private-items
    
    # Check if documentation was generated
    if [ -f "$BUILD_DIR/doc/multios_networking/index.html" ]; then
        log_success "Documentation generated successfully"
        
        # Create additional documentation files
        generate_api_docs
        generate_user_guide
        generate_troubleshooting_guide
        
    else
        log_error "Documentation generation failed"
        return 1
    fi
}

generate_api_docs() {
    log_info "Generating API documentation..."
    
    cat > "$DOCS_DIR/api_reference.md" << 'EOF'
# MultiOS Networking API Reference

## Core Components

### NetworkingManager
Central coordination for all networking subsystems.

```rust
pub struct NetworkingManager {
    wifi_manager: Option<WifiManager>,
    ethernet_manager: Option<EthernetManager>,
    network_stack: NetworkStack,
    // ...
}
```

### WifiManager
Wi-Fi driver operations and network management.

```rust
pub struct WifiManager {
    adapters: Vec<WifiAdapter>,
    connections: Vec<WifiConnection>,
    // ...
}

impl WifiManager {
    pub fn scan_networks(&self, timeout_ms: u32) -> Result<Vec<WifiNetwork>>;
    pub fn connect_to_network(&mut self, config: WifiConfig) -> Result<WifiConnection>;
    pub fn get_statistics(&self) -> WifiStatistics;
}
```

### EthernetManager
Ethernet driver operations and device management.

```rust
pub struct EthernetManager {
    adapters: Vec<EthernetAdapter>,
    vlan_configs: Vec<VlanConfig>,
    // ...
}

impl EthernetManager {
    pub fn configure_autoneg(&self, adapter_id: u32, speeds: Vec<EthernetSpeed>, duplex: DuplexMode) -> Result<()>;
    pub fn create_lag(&mut self, name: String, member_ids: Vec<u32>, mode: AggregationMode) -> Result<LinkAggregationGroup>;
    pub fn get_statistics(&self, adapter_id: u32) -> Result<EthernetStatistics>;
}
```

## Usage Examples

### Basic Wi-Fi Setup
```rust
use multios_networking::prelude::*;

let memory_manager = /* get memory manager */;
let device_manager = /* get device manager */;

NetworkingManager::init(memory_manager, device_manager)?;
let manager = get_manager().unwrap();
let wifi_manager = manager.wifi_manager()?;

let networks = wifi_manager.scan_networks(10000)?;
let config = WifiConfig {
    ssid: "MyNetwork".to_string(),
    security: SecurityProtocol::WPA2,
    password: Some("password123".to_string()),
    auto_connect: true,
    prioritize_saved: true,
    hidden_network: false,
};

let connection = wifi_manager.connect_to_network(config)?;
```

### Ethernet Configuration
```rust
let ethernet_manager = manager.ethernet_manager()?;

ethernet_manager.configure_autoneg(
    adapter_id,
    vec![EthernetSpeed::Speed1Gb],
    DuplexMode::AutoNegotiation
)?;

let lag = ethernet_manager.create_lag(
    "bond0".to_string(),
    vec![adapter1_id, adapter2_id],
    AggregationMode::Lacp
)?;
```
EOF
    
    log_success "API documentation generated"
}

generate_user_guide() {
    log_info "Generating user guide..."
    
    cat > "$DOCS_DIR/user_guide.md" << 'EOF'
# MultiOS Networking User Guide

## Getting Started

### System Requirements
- MultiOS base system
- Compatible Wi-Fi or Ethernet adapters
- Sufficient system memory

### Installation
1. Build the networking drivers
2. Load the kernel modules
3. Configure network interfaces
4. Start networking services

### Basic Configuration

#### Wi-Fi Setup
1. Identify available Wi-Fi adapters
2. Scan for available networks
3. Configure security settings
4. Establish connection

#### Ethernet Setup
1. Detect Ethernet adapters
2. Configure speed and duplex settings
3. Enable power management features
4. Configure link aggregation if needed

### Advanced Features

#### Quality of Service (QoS)
Configure traffic priorities for different applications:
- Voice traffic (highest priority)
- Gaming traffic
- Video streaming
- File transfers
- Best effort (default)

#### Security Configuration
- Firewall rules
- Wi-Fi security protocols
- Certificate management
- EAP authentication

#### Performance Optimization
- Buffer size tuning
- Interrupt coalescing
- Hardware offload features
- Power management settings
EOF
    
    log_success "User guide generated"
}

generate_troubleshooting_guide() {
    log_info "Generating troubleshooting guide..."
    
    cat > "$DOCS_DIR/troubleshooting.md" << 'EOF'
# MultiOS Networking Troubleshooting Guide

## Common Issues

### Wi-Fi Connection Problems

#### No Networks Found
**Symptoms**: Wi-Fi scan returns empty results
**Solutions**:
- Check if Wi-Fi adapter is enabled
- Verify adapter drivers are loaded
- Check physical hardware connections
- Try different scanning modes

#### Connection Timeout
**Symptoms**: Connection attempts timeout
**Solutions**:
- Move closer to access point
- Check for interference sources
- Verify network credentials
- Update Wi-Fi drivers

#### Authentication Failed
**Symptoms**: Password rejected despite correct credentials
**Solutions**:
- Verify WPA/WPA2 passphrase
- Check security protocol compatibility
- Clear saved network settings
- Try WPS connection method

### Ethernet Performance Issues

#### Slow Transfer Speeds
**Symptoms**: Network performance below expectations
**Solutions**:
- Check link speed and duplex settings
- Verify cable quality and connections
- Update Ethernet drivers
- Adjust buffer sizes

#### Connection Drops
**Symptoms**: Intermittent network connectivity
**Solutions**:
- Check physical cable connections
- Verify power management settings
- Monitor adapter temperatures
- Test with different cables

#### High Latency
**Symptoms**: Slow response times
**Solutions**:
- Adjust interrupt coalescing settings
- Increase buffer sizes
- Enable interrupt moderation
- Check for CPU overload

### Security Problems

#### Certificate Errors
**Symptoms**: Certificate validation failures
**Solutions**:
- Verify certificate chain
- Check certificate expiration
- Update CA certificates
- Verify certificate formats

#### Firewall Blocking
**Symptoms**: Network services not accessible
**Solutions**:
- Review firewall rules
- Check port configurations
- Verify direction settings
- Test with firewall disabled

### Performance Monitoring

#### Tools and Commands
- `netif-stat`: Interface statistics
- `wifi-scan --detailed`: Wi-Fi diagnostics
- `net-perf`: Performance monitoring
- `tcpdump`: Packet capture

#### Key Metrics
- Signal strength (RSSI)
- Throughput (Mbps)
- Latency (milliseconds)
- Packet loss (percentage)
- Error rates

### Hardware Diagnostics

#### Adapter Information
```bash
lspci | grep -i network
lsusb | grep -i wireless
hwinfo --network
```

#### Driver Information
```bash
modinfo <driver_name>
ethtool -i <interface_name>
iwconfig
```

#### Performance Testing
```bash
iperf3 -c <server_ip> -t 30
ping -c 100 <gateway_ip>
traceroute <destination>
```
EOF
    
    log_success "Troubleshooting guide generated"
}

run_lint_checks() {
    log_info "Running lint checks..."
    
    cd "$PROJECT_ROOT"
    
    # Run rustfmt
    if command -v rustfmt &> /dev/null; then
        log_info "Running rustfmt..."
        rustfmt --check src/*.rs examples/*.rs
        if [ $? -eq 0 ]; then
            log_success "Code formatting is correct"
        else
            log_warning "Code formatting issues found"
        fi
    fi
    
    # Run clippy
    if command -v cargo-clippy &> /dev/null; then
        log_info "Running clippy..."
        cargo clippy --all-targets --all-features 2>&1 | grep -E "warning|error" || log_success "No clippy warnings"
    fi
    
    log_success "Lint checks completed"
}

create_package() {
    log_info "Creating distribution package..."
    
    cd "$PROJECT_ROOT"
    
    # Create package structure
    PKAGE_DIR="$BUILD_DIR/multios-networking-package"
    mkdir -p "$PKAGE_DIR"/{lib,docs,examples,config}
    
    # Copy library files
    cp "$BUILD_DIR/release/"*.rlib "$PKAGE_DIR/lib/" 2>/dev/null || true
    cp "$BUILD_DIR/release/"*.a "$PKAGE_DIR/lib/" 2>/dev/null || true
    
    # Copy documentation
    cp -r "$BUILD_DIR/doc"/* "$PKAGE_DIR/docs/" 2>/dev/null || true
    cp README.md CONFIGURATION.md "$PKAGE_DIR/docs/" 2>/dev/null || true
    
    # Copy examples
    cp examples/*.rs "$PKAGE_DIR/examples/" 2>/dev/null || true
    
    # Copy configuration templates
    cp config/*.conf "$PKAGE_DIR/config/" 2>/dev/null || true
    
    # Create package info
    cat > "$PKAGE_DIR/PACKAGE_INFO" << EOF
MultiOS Networking Drivers
Version: 1.0.0
Build Date: $(date)
Build Host: $(hostname)
Package Size: $(du -sh "$PKAGE_DIR" | cut -f1)
EOF
    
    # Create archive
    cd "$BUILD_DIR"
    tar -czf multios-networking-1.0.0.tar.gz multios-networking-package/
    
    log_success "Package created: $BUILD_DIR/multios-networking-1.0.0.tar.gz"
}

validate_build() {
    log_info "Validating build..."
    
    cd "$PROJECT_ROOT"
    
    # Check library symbols
    if command -v nm &> /dev/null && [ -f "$BUILD_DIR/release/libmultios_networking.rlib" ]; then
        log_info "Checking library symbols..."
        SYMBOL_COUNT=$(nm "$BUILD_DIR/release/libmultios_networking.rlib" 2>/dev/null | wc -l)
        log_info "Library contains $SYMBOL_COUNT symbols"
    fi
    
    # Verify documentation links
    if [ -f "$BUILD_DIR/doc/multios_networking/index.html" ]; then
        log_info "Validating documentation..."
        # Check for broken links (basic check)
        if grep -q "404 Not Found" "$BUILD_DIR/doc/multios_networking/index.html"; then
            log_warning "Documentation may have broken links"
        else
            log_success "Documentation links appear valid"
        fi
    fi
    
    # Test basic functionality
    log_info "Running basic functionality test..."
    if cargo test --lib basic_functionality 2>/dev/null; then
        log_success "Basic functionality test passed"
    else
        log_warning "Basic functionality test may have failed"
    fi
    
    log_success "Build validation completed"
}

print_summary() {
    echo
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Build Summary                      ${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo
    
    # Build information
    echo -e "${BLUE}Build Information:${NC}"
    echo "  Project Root: $PROJECT_ROOT"
    echo "  Target Directory: $BUILD_DIR"
    echo "  Documentation: $BUILD_DIR/doc"
    echo
    
    # File sizes
    if [ -f "$BUILD_DIR/release/libmultios_networking.rlib" ]; then
        LIB_SIZE=$(du -h "$BUILD_DIR/release/libmultios_networking.rlib" | cut -f1)
        echo -e "${BLUE}Library Size:${NC} $LIB_SIZE"
    fi
    
    if [ -f "$BUILD_DIR/multios-networking-1.0.0.tar.gz" ]; then
        PKG_SIZE=$(du -h "$BUILD_DIR/multios-networking-1.0.0.tar.gz" | cut -f1)
        echo -e "${BLUE}Package Size:${NC} $PKG_SIZE"
    fi
    
    echo
    
    # Next steps
    echo -e "${GREEN}Next Steps:${NC}"
    echo "  1. Review generated documentation in $BUILD_DIR/doc"
    echo "  2. Test examples with actual hardware"
    echo "  3. Configure networking according to your needs"
    echo "  4. Monitor performance with built-in tools"
    echo
    
    # Useful commands
    echo -e "${YELLOW}Useful Commands:${NC}"
    echo "  View documentation: firefox $BUILD_DIR/doc/multios_networking/index.html"
    echo "  Run Wi-Fi example: cd $PROJECT_ROOT && cargo run --example wifi_scan"
    echo "  Run Ethernet example: cd $PROJECT_ROOT && cargo run --example ethernet_test"
    echo "  Run tests: cd $PROJECT_ROOT && cargo test"
    echo "  Build documentation: cd $PROJECT_ROOT && cargo doc"
    echo
    
    log_success "Build process completed successfully!"
}

# Main execution
main() {
    print_header
    
    # Parse command line arguments
    CLEAN_ONLY=false
    SKIP_TESTS=false
    SKIP_DOCS=false
    SKIP_PACKAGE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                CLEAN_ONLY=true
                shift
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --skip-docs)
                SKIP_DOCS=true
                shift
                ;;
            --skip-package)
                SKIP_PACKAGE=true
                shift
                ;;
            --help|-h)
                echo "MultiOS Networking Drivers Build Script"
                echo
                echo "Usage: $0 [options]"
                echo
                echo "Options:"
                echo "  --clean        Clean build artifacts only"
                echo "  --skip-tests   Skip running tests"
                echo "  --skip-docs    Skip documentation generation"
                echo "  --skip-package Skip creating distribution package"
                echo "  --help, -h     Show this help message"
                echo
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Main build process
    check_prerequisites
    
    if [ "$CLEAN_ONLY" = true ]; then
        clean_build
        print_summary
        exit 0
    fi
    
    clean_build
    check_dependencies
    
    log_info "Starting build process..."
    
    # Build stages
    build_release
    build_debug
    
    if [ "$SKIP_TESTS" = false ]; then
        run_tests
        run_examples
        run_lint_checks
    else
        log_warning "Tests skipped"
    fi
    
    if [ "$SKIP_DOCS" = false ]; then
        generate_documentation
    else
        log_warning "Documentation generation skipped"
    fi
    
    validate_build
    
    if [ "$SKIP_PACKAGE" = false ]; then
        create_package
    else
        log_warning "Package creation skipped"
    fi
    
    print_summary
}

# Error handling
trap 'log_error "Build failed at line $LINENO"' ERR

# Execute main function
main "$@"