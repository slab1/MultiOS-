# Comprehensive Stress Testing Suite

Advanced stress testing system for memory management and file systems under extreme conditions.

## 🚀 Features

### 1. Memory Pressure Testing
- **Allocation Limits**: Tests maximum memory allocation capacity
- **Memory Leak Detection**: Identifies memory leaks through extended testing
- **Memory Fragmentation**: Analyzes fragmentation patterns and impact
- **Memory Pressure Response**: Tests system behavior under memory constraints
- **Overflow Protection**: Validates memory overflow protection mechanisms

### 2. File System Stress Tests
- **I/O Limits**: Tests file system throughput and latency
- **Concurrent Access**: Tests file access under concurrent load
- **Corruption Scenarios**: Simulates file corruption and recovery
- **Disk Space Exhaustion**: Tests behavior when disk space is exhausted
- **File Handle Limits**: Tests file descriptor management

### 3. CPU Stress Testing
- **CPU Utilization**: Tests CPU performance under sustained load
- **Thermal Testing**: Monitors thermal throttling behavior
- **CPU Scheduling**: Tests process and thread scheduling
- **Cache Performance**: Analyzes CPU cache efficiency

### 4. Concurrent Process and Thread Stress Testing
- **Thread Pool Stress**: Tests thread pool behavior under load
- **Process Pool Stress**: Tests process pool scaling and isolation
- **Synchronization**: Tests locks, semaphores, and barriers
- **IPC Testing**: Tests inter-process communication
- **Resource Contention**: Tests resource contention scenarios

### 5. Resource Exhaustion Scenarios
- **File Handle Exhaustion**: Tests file descriptor limits
- **Memory Exhaustion**: Tests memory exhaustion recovery
- **Process Limits**: Tests process creation limits
- **Network Connection Limits**: Tests network resource limits
- **Resource Recovery**: Tests cleanup and recovery mechanisms

### 6. Recovery and Error Handling
- **Graceful Degradation**: Tests system behavior under stress
- **Error Recovery**: Validates error handling and recovery
- **System Stabilization**: Tests post-stress system recovery
- **Resource Cleanup**: Verifies proper resource cleanup

### 7. Performance Degradation Monitoring
- **Real-time Monitoring**: Continuous resource monitoring during tests
- **Performance Baseline**: Establishes performance baselines
- **Degradation Detection**: Identifies performance degradation patterns
- **Threshold Monitoring**: Alerts on threshold violations

### 8. Automated Reporting and Analysis
- **HTML Reports**: Interactive web-based reports with charts
- **PDF Reports**: Professional PDF reports with analysis
- **JSON Reports**: Machine-readable detailed reports
- **CSV Reports**: Spreadsheet-compatible data export
- **Performance Charts**: Visual analysis of test results

## 📋 Requirements

### System Requirements
- Python 3.7+
- Linux/Unix system (primary target)
- At least 4GB RAM (8GB+ recommended)
- 10GB+ free disk space
- Multiple CPU cores recommended

### Python Dependencies
```bash
pip install -r requirements.txt
```

**Core Dependencies:**
- `psutil` - System and process utilities
- `numpy` - Numerical computing
- `matplotlib` - Plotting and visualization
- `pandas` - Data analysis
- `seaborn` - Statistical data visualization
- `jinja2` - Template engine for reports

## 🛠️ Installation

1. **Clone or download the stress testing suite:**
```bash
# If you have the source code
cd /workspace/testing/stress_tests
```

2. **Install dependencies:**
```bash
pip install -r requirements.txt
```

3. **Create configuration file:**
```bash
python run_stress_tests.py --create-config
```

4. **Customize configuration (optional):**
Edit `stress_test_config.json` to adjust test parameters.

## 🚀 Usage

### Quick Start
```bash
python run_stress_tests.py
```

### Advanced Usage
```bash
# Run with custom configuration
python run_stress_tests.py --config my_config.json

# Run with custom output directory and duration
python run_stress_tests.py --output /tmp/stress_results --duration 600

# Run with more parallel threads
python run_stress_tests.py --parallel 8 --verbose

# Validate environment only
python run_stress_tests.py --validate
```

### Configuration Options

Create a `stress_test_config.json` file:

```json
{
  "test_duration": 300,
  "test_dir": "/tmp/stress_test",
  "output_dir": "./stress_test_results",
  "parallel_threads": 4,
  "verbose": false,
  
  "max_memory_allocation_mb": 512,
  "min_available_memory_mb": 1024,
  "memory_leak_iterations": 1000,
  "fragmentation_test_size_mb": 256,
  
  "min_available_disk_gb": 10,
  "file_io_test_size_mb": 100,
  "concurrent_file_access_threads": 10,
  "max_file_handles": 1024,
  
  "cpu_stress_duration": 60,
  "cpu_threads_per_core": 2,
  "thermal_test_duration": 30,
  
  "max_processes": 100,
  "max_network_connections": 1000,
  
  "cpu_usage_warning_threshold": 90.0,
  "memory_usage_warning_threshold": 85.0,
  "disk_usage_warning_threshold": 90.0
}
```

## 📊 Test Categories

### Memory Tests
- `Memory Allocation Limits` - Tests maximum memory allocation
- `Memory Leak Detection` - Identifies memory leaks
- `Memory Fragmentation` - Analyzes fragmentation patterns
- `Memory Pressure` - Tests behavior under memory pressure
- `Memory Overflow Protection` - Validates protection mechanisms

### File System Tests
- `File I/O Limits` - Tests file system throughput
- `Concurrent File Access` - Tests concurrent access patterns
- `File Corruption Simulation` - Tests corruption handling
- `Disk Space Exhaustion` - Tests behavior with full disk
- `File Handle Limits` - Tests file descriptor management

### CPU Tests
- `CPU Stress` - Tests CPU performance under load
- `CPU Thermal` - Tests thermal throttling
- `CPU Scheduling` - Tests process scheduling
- `CPU Cache Performance` - Analyzes cache efficiency

### Concurrent Tests
- `Thread Pool Stress` - Tests thread pool behavior
- `Process Pool Stress` - Tests process pool scaling
- `Thread Synchronization` - Tests synchronization primitives
- `Process Communication` - Tests IPC mechanisms
- `Resource Contention` - Tests resource contention

### Resource Exhaustion Tests
- `File Handle Exhaustion` - Tests file descriptor limits
- `Memory Exhaustion` - Tests memory exhaustion recovery
- `Process Limit Exhaustion` - Tests process creation limits
- `Network Connection Limits` - Tests network resources
- `Resource Recovery` - Tests cleanup mechanisms

## 📈 Reports

The suite generates comprehensive reports in multiple formats:

### Report Types
- **HTML Report**: Interactive web-based report with charts
- **PDF Report**: Professional report for documentation
- **JSON Report**: Detailed machine-readable data
- **CSV Report**: Spreadsheet-compatible export

### Report Contents
- Executive summary with key metrics
- Detailed test results by category
- Performance charts and analysis
- Resource usage timelines
- System information and recommendations

### Report Locations
- Reports: `./stress_test_results/reports/`
- Charts: `./stress_test_results/charts/`
- Logs: `./stress_test_results/logs/`

## 🔧 Advanced Features

### Custom Test Development
You can extend the suite by adding custom test modules:

```python
from memory.memory_stress import MemoryStressTester

class CustomMemoryTest(MemoryStressTester):
    def test_custom_scenario(self):
        # Your custom test logic here
        return {
            "test_name": "Custom Memory Test",
            "status": "PASS",
            "metrics": {"custom_metric": 100},
            "errors": [],
            "warnings": []
        }
```

### Integration with CI/CD
The suite returns appropriate exit codes:
- `0`: All tests passed (≥90% success rate)
- `1`: Some tests failed (70-89% success rate)
- `2`: Many tests failed (<70% success rate)
- `130`: Interrupted by user

### Monitoring and Alerting
Real-time monitoring during tests:
- CPU usage tracking
- Memory consumption monitoring
- Disk I/O measurement
- Process count tracking

## 🐛 Troubleshooting

### Common Issues

**1. Insufficient Memory**
```bash
# Reduce memory test size in config
"max_memory_allocation_mb": 256,
"min_available_memory_mb": 512
```

**2. Disk Space Issues**
```bash
# Reduce file system test size
"file_io_test_size_mb": 50,
"min_available_disk_gb": 5
```

**3. Permission Errors**
```bash
# Run with appropriate permissions or change test directory
"test_dir": "/tmp/stress_test"
```

**4. Missing Dependencies**
```bash
pip install -r requirements.txt
```

### Performance Tuning

**For Better Performance:**
- Increase `parallel_threads` for more concurrency
- Reduce `test_duration` for shorter runs
- Use SSD storage for better I/O performance
- Ensure adequate system resources

**For Thorough Testing:**
- Increase `test_duration` for longer runs
- Increase `memory_leak_iterations` for better leak detection
- Enable `verbose` logging for detailed output

## 📚 Documentation

### Architecture Overview
- **Main Orchestrator**: `main_stress_test.py` - Coordinates all tests
- **Test Modules**: Individual test categories in separate modules
- **System Monitor**: Real-time resource monitoring
- **Report Generator**: Multi-format report generation
- **Configuration**: Centralized configuration management

### Test Execution Flow
1. **Environment Validation** - Check system requirements
2. **Configuration Loading** - Load test parameters
3. **Baseline Collection** - Establish system baseline
4. **Test Execution** - Run tests by category
5. **Monitoring** - Continuous resource monitoring
6. **Recovery Testing** - Test system recovery
7. **Report Generation** - Create comprehensive reports

## 🤝 Contributing

To contribute to the stress testing suite:

1. **Add New Test Categories**:
   - Create new test module in appropriate directory
   - Inherit from base test class
   - Implement required test methods

2. **Enhance Reporting**:
   - Extend `StressReportGenerator`
   - Add new chart types
   - Improve report templates

3. **Performance Improvements**:
   - Optimize test execution
   - Reduce resource overhead
   - Improve monitoring efficiency

## 📄 License

This stress testing suite is provided as-is for system testing and validation purposes.

## 🆘 Support

### Getting Help
- Check the troubleshooting section above
- Review test logs in `./stress_test_results/logs/`
- Enable verbose mode for detailed output

### Reporting Issues
When reporting issues, please include:
- System information (OS, Python version, hardware)
- Configuration file contents
- Error messages and logs
- Steps to reproduce

---

**Happy Stress Testing! 🎯**

For more information, explore the code documentation and test results.