# Performance Visualization Tools

A comprehensive performance monitoring and visualization system that provides real-time system behavior analysis, customizable metrics, and advanced performance visualization tools.

## 🚀 Features

### Real-time Performance Dashboard
- **Live Metrics**: Real-time CPU, memory, disk, network, and power consumption monitoring
- **Customizable Refresh**: Configurable update intervals (1s, 2s, 5s, 10s)
- **Interactive Charts**: Dynamic graphs with zoom, pan, and detail views
- **Export Capability**: Export current data and historical trends

### CPU Utilization Heatmaps
- **Core-level Monitoring**: Individual CPU core utilization tracking
- **Scheduling Visualization**: CPU scheduling patterns and load distribution
- **Thermal Mapping**: Temperature visualization for each core
- **Performance Trends**: Historical CPU usage patterns

### Memory Usage Patterns
- **Allocation Tracking**: Memory allocation breakdown by type and usage
- **Swap Usage**: Swap space monitoring and analysis
- **Memory Leaks**: Detection of memory allocation anomalies
- **Cache Analysis**: Buffer and cache utilization statistics

### I/O Throughput Graphs
- **Device-level Breakdown**: Individual disk and storage device monitoring
- **Read/Write Separation**: Separate tracking of read and write operations
- **Queue Depth**: I/O queue depth and latency analysis
- **Storage Health**: Disk health and performance indicators

### Network Traffic Visualization
- **Protocol Breakdown**: Traffic analysis by protocol (TCP, UDP, HTTP, HTTPS, DNS)
- **Bandwidth Monitoring**: Upload/download speed tracking
- **Connection Analysis**: Active connections and network utilization
- **Latency Visualization**: Network latency and performance metrics

### Power Consumption & Thermal Analysis
- **Power Monitoring**: Real-time power consumption tracking
- **Thermal Mapping**: Temperature visualization across system components
- **Efficiency Analysis**: Power efficiency and thermal performance
- **Battery Status**: Battery level and charging status (laptops)

### Interactive Performance Analysis Tools
- **Process Monitor**: Top CPU and memory consuming processes
- **System Events**: Performance-related event logging
- **Anomaly Detection**: Automated detection of performance anomalies
- **Threshold Alerts**: Configurable performance threshold monitoring

### Historical Performance Trending
- **Data Storage**: Long-term performance data storage and retrieval
- **Trend Analysis**: Historical performance trend identification
- **Comparison Tools**: Side-by-side performance comparison
- **Statistical Analysis**: Comprehensive statistical performance metrics

## 📁 Project Structure

```
/workspace/education/performance/
├── dashboard/                 # Web-based performance dashboard
│   ├── index.html           # Main dashboard interface
│   ├── styles.css           # Dashboard styling
│   └── dashboard.js         # Dashboard JavaScript functionality
├── cli/                     # Command-line interface tools
│   ├── performance_cli.py   # Master CLI interface
│   ├── performance_monitor.py  # Data collection tool
│   ├── performance_analyzer.py # Data analysis tool
│   └── performance_visualizer.py # Visualization generator
├── data/                    # Performance data storage
├── output/                  # Analysis and visualization output
├── docs/                    # Documentation
└── requirements.txt         # Python dependencies
```

## 🛠 Installation

### Prerequisites
- Python 3.8 or higher
- Linux/macOS/Windows with system monitoring support

### Setup
1. **Clone or extract the performance tools**
2. **Install Python dependencies**:
   ```bash
   cd /workspace/education/performance
   pip install -r requirements.txt
   ```

3. **Verify system monitoring support**:
   ```bash
   python -c "import psutil; print('System monitoring: OK')"
   ```

### System Requirements
- **CPU**: Multi-core processor recommended for detailed core monitoring
- **Memory**: At least 4GB RAM for comprehensive monitoring
- **Storage**: SSD recommended for I/O performance analysis
- **Permissions**: Elevated privileges may be required for detailed system metrics

## 🎯 Usage

### Command Line Interface

The main CLI tool provides comprehensive performance monitoring capabilities:

```bash
cd /workspace/education/performance/cli

# Monitor system performance
python performance_cli.py monitor --duration 300 --export mydata

# Analyze existing data
python performance_cli.py analyze mydata.json --stats --anomalies iqr

# Generate visualizations
python performance_cli.py visualize mydata.json --all

# Start web dashboard
python performance_cli.py dashboard --port 8080

# Compare datasets
python performance_cli.py compare data1.json data2.json

# List available data files
python performance_cli.py list

# Clean old data
python performance_cli.py clean --older-than 30
```

### Individual Tools

#### Performance Monitor
```bash
# Basic monitoring (2-second intervals)
python performance_monitor.py

# Monitor for specific duration
python performance_monitor.py --duration 60 --interval 1.0

# Export data
python performance_monitor.py --export performance_data

# Generate charts after monitoring
python performance_monitor.py --chart

# Analyze trends
python performance_monitor.py --analyze
```

#### Performance Analyzer
```bash
# Comprehensive statistics
python performance_analyzer.py data.json --stats

# Anomaly detection
python performance_analyzer.py data.json --anomalies iqr --threshold 2.0

# Compare time periods
python performance_analyzer.py data.json --compare "2023-01-01T00:00" "2023-01-01T01:00" "2023-01-01T02:00" "2023-01-01T03:00"

# Generate detailed charts
python performance_analyzer.py data.json --chart --output-dir analysis_output

# Export analysis results
python performance_analyzer.py data.json --export analysis_results
```

#### Performance Visualizer
```bash
# Generate all visualizations
python performance_visualizer.py data.json --all

# Generate specific charts
python performance_visualizer.py data.json --heatmap --thermal --dashboard

# Custom output directory
python performance_visualizer.py data.json --all --output-dir custom_output
```

### Web Dashboard

1. **Start the dashboard**:
   ```bash
   python performance_cli.py dashboard
   ```

2. **Access the dashboard**:
   - Open browser to `http://localhost:8080`
   - The dashboard provides real-time monitoring

3. **Dashboard Features**:
   - Real-time charts updating every 2 seconds
   - Customizable metrics display
   - Export capabilities
   - Historical data visualization
   - Interactive charts with zoom and detail views

## 📊 Data Format

### JSON Data Structure
```json
{
  "monitoring_info": {
    "start_time": "2023-01-01T12:00:00",
    "duration": 300,
    "interval": 2.0
  },
  "samples": [
    {
      "cpu": {
        "usage": 45.2,
        "per_core": [42.1, 48.3, 45.0, 44.8],
        "frequency": {
          "current": 2500.0,
          "min": 1200.0,
          "max": 3200.0
        }
      },
      "memory": {
        "total": 16777216000,
        "used": 8589934592,
        "percentage": 51.2
      },
      "network": {
        "io_counters": {
          "bytes_sent": 1024000,
          "bytes_recv": 2048000
        }
      },
      "timestamp": "2023-01-01T12:00:02"
    }
  ]
}
```

## 🎨 Visualization Types

### CPU Heatmaps
- **Core Utilization**: Color-coded visualization of CPU core usage
- **Load Distribution**: CPU scheduling visualization
- **Thermal Mapping**: Temperature-based coloring

### Memory Allocation Charts
- **Usage Breakdown**: Pie chart of memory allocation types
- **Time Series**: Memory usage trends over time
- **Allocation Tracking**: Memory allocation pattern analysis

### I/O Throughput Graphs
- **Read/Write Separated**: Separate visualization of read and write operations
- **Device Breakdown**: Individual storage device performance
- **Queue Analysis**: I/O queue depth and latency

### Network Traffic Visualization
- **Protocol Analysis**: Traffic breakdown by network protocol
- **Bandwidth Monitoring**: Real-time upload/download speeds
- **Connection Tracking**: Active network connections

### Power & Thermal Charts
- **Power Consumption**: Real-time power usage tracking
- **Thermal Zones**: Temperature visualization across components
- **Efficiency Analysis**: Power efficiency metrics

## 🔧 Configuration

### Refresh Intervals
- Real-time monitoring: 1s, 2s, 5s, 10s
- Historical analysis: 5s, 30s, 60s
- Long-term trends: 5min, 15min, 1hour

### Alert Thresholds
```python
# Example threshold configuration
THRESHOLDS = {
    'cpu': {'warning': 80, 'critical': 95},
    'memory': {'warning': 85, 'critical': 95},
    'temperature': {'warning': 70, 'critical': 85},
    'power': {'warning': 150, 'critical': 200}
}
```

### Data Retention
- Real-time buffer: 3600 samples (1 hour at 1s intervals)
- Long-term storage: Configurable retention periods
- Automatic cleanup: Remove data older than specified days

## 🐛 Troubleshooting

### Common Issues

1. **Permission Errors**:
   ```bash
   # Linux/macOS - Run with appropriate permissions
   sudo python performance_cli.py monitor
   ```

2. **Missing Dependencies**:
   ```bash
   pip install --upgrade -r requirements.txt
   ```

3. **Dashboard Not Loading**:
   - Check browser console for JavaScript errors
   - Ensure port is not in use: `lsof -i :8080`
   - Try different port: `python performance_cli.py dashboard --port 8081`

4. **No Data Collected**:
   - Verify system monitoring support
   - Check firewall settings
   - Ensure sufficient disk space

### Performance Considerations

- **Memory Usage**: Dashboard and analysis tools require sufficient RAM
- **CPU Impact**: Monitoring adds minimal CPU overhead (~1-2%)
- **Storage**: Data collection generates JSON files, monitor disk space
- **Network**: Dashboard uses minimal bandwidth for real-time updates

### System-Specific Notes

#### Linux
- Install `lm-sensors` for temperature monitoring: `sudo apt-get install lm-sensors`
- Enable ACPI for power monitoring: `sudo apt-get install acpi`
- Some metrics require root access

#### macOS
- Temperature monitoring may be limited
- Power metrics available on laptops
- Some system calls require appropriate permissions

#### Windows
- Requires Python 3.8+ with appropriate permissions
- Some advanced metrics may not be available
- Power monitoring available on laptops

## 📈 Performance Analysis

### Metrics Collected
- **CPU**: Usage percentage, core utilization, frequency, load average
- **Memory**: Total, used, available, cached, buffers, swap
- **Disk**: Read/write operations, throughput, usage by device
- **Network**: Bytes sent/received, packets, errors, connections
- **Temperature**: CPU, GPU, chipset, battery temperatures
- **Power**: Consumption estimation, battery status

### Anomaly Detection Methods
- **IQR (Interquartile Range)**: Statistical outlier detection
- **Z-Score**: Standard deviation-based anomaly detection
- **Threshold-based**: Fixed threshold violations
- **Trend Analysis**: Performance degradation detection

### Statistical Analysis
- **Descriptive Statistics**: Mean, median, standard deviation, percentiles
- **Correlation Analysis**: Relationship between different metrics
- **Trend Analysis**: Performance changes over time
- **Comparative Analysis**: Performance comparison between periods

## 🔄 Advanced Usage

### Custom Dashboards
Modify `dashboard/dashboard.js` to add custom metrics or visualizations:

```javascript
// Add custom metric to dashboard
function addCustomMetric(data) {
    const customValue = calculateCustomMetric(data);
    updateMetricDisplay('customMetric', customValue);
}
```

### Extending Analysis
Add custom analysis functions to `performance_analyzer.py`:

```python
def custom_analysis(self, data):
    # Implement custom analysis logic
    custom_results = perform_custom_analysis(data)
    return custom_results
```

### Integration with Other Tools
- **Prometheus**: Export metrics for Prometheus monitoring
- **Grafana**: Import data for Grafana dashboards
- **Log Analysis**: Integrate with system log analysis tools

## 📝 Logging and Monitoring

### Log Files
- Performance data: `/workspace/education/performance/data/`
- Analysis results: `/workspace/education/performance/output/`
- Generated charts: `performance_charts/`, `performance_visualizations/`

### Monitoring Best Practices
1. **Regular Data Collection**: Establish consistent monitoring intervals
2. **Threshold Configuration**: Set appropriate alert thresholds
3. **Historical Analysis**: Regularly review performance trends
4. **Capacity Planning**: Use data for resource planning
5. **Incident Response**: Use real-time alerts for performance issues

## 🤝 Contributing

To extend the performance visualization tools:

1. **Add New Metrics**: Extend data collection in `performance_monitor.py`
2. **New Visualizations**: Add to `performance_visualizer.py`
3. **Custom Analysis**: Implement in `performance_analyzer.py`
4. **Dashboard Enhancements**: Modify `dashboard.js` and `dashboard.html`

## 📄 License

This performance visualization system is provided as-is for educational and monitoring purposes.

## 🆘 Support

For issues and questions:
1. Check the troubleshooting section above
2. Verify system requirements and dependencies
3. Review log files for error messages
4. Ensure proper permissions for system monitoring

---

**Performance Visualization Tools v1.0** - Comprehensive system performance monitoring and analysis suite.