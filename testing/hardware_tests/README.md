# Hardware Testing Framework

A comprehensive testing framework for real hardware beyond QEMU, designed for testing older PCs, Raspberry Pi, embedded boards, and servers.

## Overview

This framework provides automated hardware detection, compatibility testing, performance benchmarking, and optimization recommendations for real-world hardware systems.

## Features

### 1. Hardware Detection and Auto-Configuration
- **Automatic hardware profiling** of CPUs, memory, storage, network interfaces, GPUs, and USB devices
- **Thermal sensor detection** and monitoring
- **BIOS/UEFI information extraction**
- **Hardware compatibility assessment**

### 2. Remote Testing Capabilities
- **SSH-based remote testing** for physical hardware
- **Parallel testing** across multiple devices
- **Real-time monitoring** of remote systems
- **Results transfer and consolidation**

### 3. Automated Hardware Compatibility Testing
- **Comprehensive compatibility checks** for all hardware components
- **Driver compatibility verification**
- **Feature support detection** (SSE4, AVX, hyperthreading, etc.)
- **Performance threshold validation**

### 4. Power Management and Thermal Testing
- **Power consumption monitoring** and analysis
- **Thermal performance testing** under load
- **Thermal cycling tests** for stability
- **Power efficiency scoring**

### 5. Peripheral Testing
- **USB device testing** (storage, input, audio, etc.)
- **Network interface testing** (performance and connectivity)
- **Storage device testing** (read/write performance)
- **GPU testing** (detection, drivers, performance)

### 6. Multi-Core Scaling Tests
- **CPU scaling analysis** across all cores
- **Hyperthreading effectiveness** measurement
- **CPU affinity testing** for optimal performance
- **Thread scaling efficiency** calculations

### 7. Long-Term Stability Testing
- **Extended stress testing** (up to 24+ hours)
- **Memory leak detection**
- **Thermal stability analysis**
- **Performance degradation monitoring**

### 8. Hardware-Specific Optimization Recommendations
- **AI-powered optimization engine**
- **Automatic performance tuning**
- **Hardware-specific configuration**
- **Workload-aware recommendations**

## Installation

### Prerequisites

```bash
# Python dependencies
pip install psutil matplotlib numpy

# System utilities
sudo apt-get install lm-sensors hwinfo stress-ng iperf3

# Optional for NVIDIA GPU testing
# Install NVIDIA drivers and nvidia-ml-py
```

### Framework Setup

```bash
# Clone or download the framework
cd /workspace/testing/hardware_tests

# Make scripts executable
chmod +x scripts/*.sh

# Install Python dependencies
pip install -r requirements.txt  # If requirements.txt exists
```

## Quick Start

### Using the Quick Test Script

```bash
# Run interactive menu
./scripts/quick_test.sh

# Or run specific tests
./scripts/quick_test.sh --quick-detection
./scripts/quick_test.sh --compatibility
./scripts/quick_test.sh --performance
```

### Using the Main Orchestrator

```bash
# Full comprehensive test (24 hours)
python3 hardware_test_orchestrator.py full

# Hardware detection only
python3 hardware_test_orchestrator.py detect

# Compatibility testing
python3 hardware_test_orchestrator.py compatibility

# Quick performance tests (1 hour)
python3 hardware_test_orchestrator.py full --duration 1 --skip-stability
```

## Individual Components

### Hardware Detection
```bash
python3 hardware_detector.py --profile --config
```

**Features:**
- CPU topology detection (cores, threads, features, frequency)
- Memory profiling (capacity, type, speed, ECC support)
- Storage device enumeration (type, capacity, performance)
- Network interface detection (speed, type, configuration)
- GPU identification (vendor, memory, driver)
- USB device cataloging

### Compatibility Testing
```bash
python3 compatibility_testing.py --categories cpu memory storage network
```

**Test Categories:**
- **CPU**: Instruction set compatibility, virtualization support, multi-core scaling
- **Memory**: Size requirements, bandwidth, ECC functionality
- **Storage**: Interface compatibility, performance thresholds, RAID support
- **Network**: Interface detection, performance, connectivity
- **GPU**: Driver compatibility, compute capability, performance
- **USB**: Device enumeration, functionality testing

### Power and Thermal Testing
```bash
python3 power_thermal_testing.py --test all --duration 60
```

**Tests:**
- Power consumption under various loads
- Thermal performance and stability
- Temperature sensor validation
- Power efficiency metrics
- Thermal cycling tests

### Peripheral Testing
```bash
python3 peripheral_testing.py --test all
```

**Peripherals Tested:**
- USB devices (storage, input, audio, network)
- Network interfaces (ethernet, wireless)
- Storage devices (SSD, HDD, NVMe)
- Graphics cards (NVIDIA, AMD, Intel)

### Multi-Core Scaling
```bash
python3 multicore_scaling_test.py --test scaling --max-threads 16
```

**Analysis:**
- CPU topology detection
- Thread scaling performance
- Hyperthreading effectiveness
- CPU affinity optimization
- Optimal thread count determination

### Long-Term Stability
```bash
python3 stability_testing.py --test comprehensive --duration 24
```

**Stability Tests:**
- Memory leak detection
- Thermal stability monitoring
- Performance degradation analysis
- Extended stress testing
- Anomaly detection

### Optimization Engine
```bash
python3 optimization_engine.py --analyze --recommend --apply
```

**Optimizations:**
- Hardware capability analysis
- Performance tuning recommendations
- Automatic configuration changes
- Workload-specific optimizations

## Configuration

### Hardware Profiles

Hardware profiles are automatically generated and stored in:
```
/workspace/testing/hardware_tests/profiles/
```

Example profile structure:
```json
{
  "detection_timestamp": 1234567890.123,
  "hardware": {
    "cpu": {
      "model": "Intel Core i7-9700K",
      "cores_physical": 8,
      "cores_logical": 8,
      "frequency_max": 4.9,
      "features": ["sse4_1", "sse4_2", "avx2"]
    },
    "memory": {
      "total_gb": 16,
      "memory_type": "DDR4-3200",
      "ecc_support": false
    }
  }
}
```

### Test Configuration

Configuration files are stored in:
```
/workspace/testing/hardware_tests/config/
```

## Test Results

All test results are stored in:
```
/workspace/testing/hardware_tests/results/
```

### Report Types

1. **Hardware Profile** (`hardware_profile_*.json`)
2. **Compatibility Report** (`compatibility_report_*.json`)
3. **Peripheral Test Report** (`peripheral_test_report_*.json`)
4. **Multi-Core Scaling Report** (`multicore_scaling_report_*.json`)
5. **Power/Thermal Report** (`power_thermal_report_*.json`)
6. **Stability Test Report** (`stability_test_report_*.json`)
7. **Optimization Report** (`optimization_report_*.json`)
8. **Final Comprehensive Report** (`final_hardware_test_report_*.json`)

### Result Analysis

Results include:
- **System scores** (0-100 rating)
- **Performance metrics** and benchmarks
- **Compatibility status** for each component
- **Optimization recommendations**
- **Hardware grade** (A+ to F)

## Supported Hardware

### CPUs
- Intel x86/x64 (Core, Xeon, Atom)
- AMD x86/x64 (Ryzen, EPYC, Athlon)
- ARM (Cortex-A, ARMv8, embedded systems)
- RISC-V (experimental support)

### Memory
- DDR3/DDR4/DDR5
- LPDDR3/LPDDR4/LPDDR5
- ECC and non-ECC
- Various capacities (1GB to 1TB+)

### Storage
- SATA SSDs and HDDs
- NVMe SSDs
- USB storage devices
- SD cards (Raspberry Pi, embedded)
- RAID arrays

### Network
- Ethernet (10/100/1000/10G+ Mbps)
- Wireless (802.11a/b/g/n/ac/ax)
- USB network adapters

### Graphics
- NVIDIA (GeForce, Quadro, Tesla)
- AMD (Radeon, FirePro)
- Intel (integrated graphics)
- ARM Mali, VideoCore

### Systems
- Desktop workstations
- Enterprise servers
- Single board computers (Raspberry Pi, etc.)
- Embedded systems
- Virtual machines (limited)

## Example Usage Scenarios

### 1. New System Validation
```bash
# Complete system validation
python3 hardware_test_orchestrator.py full --duration 4

# Quick validation (30 minutes)
python3 hardware_test_orchestrator.py full --duration 0.5 --skip-stability
```

### 2. Hardware Compatibility Check
```bash
# Test compatibility before deployment
python3 hardware_test_orchestrator.py compatibility

# Test specific components
python3 compatibility_testing.py --categories cpu memory storage
```

### 3. Performance Benchmarking
```bash
# Multi-core performance analysis
python3 multicore_scaling_test.py --test scaling

# Storage performance testing
python3 peripheral_testing.py --test storage

# Power/thermal analysis
python3 power_thermal_testing.py --test all --duration 30
```

### 4. Remote Hardware Testing
```bash
# Test remote server
python3 remote_testing.py --hostname server1 --username admin --test stress

# Parallel testing of multiple devices
# (See remote_testing.py for configuration)
```

### 5. Optimization Analysis
```bash
# Generate optimization recommendations
python3 optimization_engine.py --analyze --recommend

# Apply automatic optimizations
python3 optimization_engine.py --apply
```

## Advanced Features

### Custom Test Scripts

Create custom test scripts using the framework components:

```python
from hardware_detector import HardwareDetector
from compatibility_testing import HardwareCompatibilityTester

# Detect hardware
detector = HardwareDetector()
hardware_data = detector.run_full_detection()

# Run compatibility tests
tester = HardwareCompatibilityTester()
tester.current_hardware_profile = hardware_data
results = tester.run_compatibility_suite()
```

### Integration with CI/CD

```yaml
# Example GitHub Actions workflow
- name: Hardware Compatibility Test
  run: |
    python3 hardware_test_orchestrator.py compatibility
    # Check if all tests passed
    if [ $? -eq 0 ]; then
      echo "Hardware compatibility verified"
    else
      echo "Hardware compatibility issues detected"
      exit 1
    fi
```

### Automated Reporting

```bash
# Generate reports and send notifications
python3 hardware_test_orchestrator.py full --duration 2 --output /tmp/results/

# Email report (requires mail setup)
cat results/final_hardware_test_report_*.json | \
  mail -s "Hardware Test Report" admin@company.com
```

## Troubleshooting

### Common Issues

1. **Permission Errors**
   ```bash
   # Some tests require root access
   sudo python3 hardware_test_orchestrator.py full
   ```

2. **Missing Dependencies**
   ```bash
   # Install required system packages
   sudo apt-get install lm-sensors hwinfo stress-ng
   pip install psutil matplotlib numpy
   ```

3. **GPU Testing Issues**
   ```bash
   # NVIDIA GPU testing requires drivers
   nvidia-smi  # Should work
   ```

4. **Thermal Monitoring**
   ```bash
   # Configure lm-sensors
   sudo sensors-detect
   sudo modprobe coretemp
   ```

### Debug Mode

```bash
# Enable verbose logging
python3 hardware_test_orchestrator.py --verbose
# Or set environment variable
export HARDWARE_TEST_DEBUG=1
```

### Log Files

Check log files for detailed information:
```
/workspace/testing/hardware_tests/logs/orchestrator.log
/workspace/testing/hardware_tests/hardware_detection.log
/workspace/testing/hardware_tests/compatibility_test.log
```

## Contributing

To extend the framework:

1. **Add new hardware detection** in `hardware_detector.py`
2. **Implement new compatibility tests** in `compatibility_testing.py`
3. **Add optimization rules** in `optimization_engine.py`
4. **Create custom test modules** following the existing patterns

## License

This hardware testing framework is provided as-is for educational and testing purposes.

## Support

For issues, questions, or contributions, please refer to the framework documentation or create detailed issue reports with:
- Hardware specification
- Operating system details
- Error messages/logs
- Steps to reproduce issues