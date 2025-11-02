# Performance Monitoring Dashboard

A comprehensive real-time performance monitoring system that provides system metrics, process monitoring, alerting, and performance analytics through both web and CLI interfaces.

## Features

### 🔍 System Monitoring
- **Live System Metrics**: Real-time CPU, memory, disk, and network usage
- **Process Monitoring**: Detailed process performance tracking with CPU/memory usage
- **Kernel Performance Counters**: System-level performance metrics
- **Historical Data**: Long-term performance data storage and analysis

### 📊 Visualization & Analytics
- **Interactive Web Dashboard**: Real-time charts and graphs
- **Performance Reports**: PDF, HTML, and CSV report generation
- **Custom Metrics**: Support for custom performance indicators
- **Data Export**: Multiple export formats for external analysis

### 🚨 Alerting System
- **Threshold-based Alerts**: Configurable warning and critical alerts
- **Multi-channel Notifications**: Email, webhook, and log notifications
- **Alert Aggregation**: Prevents alert spam with intelligent aggregation
- **Alert Management**: Acknowledge, resolve, and track alerts

### 🎛️ Control Interfaces
- **Web Interface**: Full-featured React-based dashboard
- **CLI Tools**: Command-line monitoring and control
- **REST API**: Programmatic access to all features
- **Real-time Updates**: WebSocket-based live data streaming

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React)                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐│
│  │  Dashboard  │ │   Charts    │ │   Alerts    │ │Processes││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘│
└─────────────────────────────────────────────────────────────┘
                            │
                     WebSocket│REST API
                            │
┌─────────────────────────────────────────────────────────────┐
│                  Backend (Flask)                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐│
│  │Web Dashboard│ │  API Server │ │Alert Manager│ │  Report ││
│  │             │ │             │ │             │ │Generator││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘│
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│              System Monitor Engine                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐│
│  │CPU Monitor  │ │Memory Monitor│ │Disk Monitor │ │Network  ││
│  │             │ │             │ │             │ │Monitor  ││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘│
└─────────────────────────────────────────────────────────────┘
                            │
                    System Calls (psutil)
                            │
┌─────────────────────────────────────────────────────────────┐
│                    Database (SQLite)                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐│
│  │System Metrics│ │Process Metrics│ │Custom Metrics│ │ Alerts ││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites
- Python 3.8+
- Node.js 16+ (for frontend)
- Unix-like system (Linux, macOS) recommended

### Quick Setup

1. **Clone and Setup**
   ```bash
   git clone <repository-url>
   cd perf/monitor_dashboard
   ```

2. **Backend Setup**
   ```bash
   cd backend
   pip install -r requirements.txt
   ```

3. **Frontend Setup**
   ```bash
   cd frontend/perf-dashboard
   npm install
   npm run build
   ```

4. **Start the System**
   ```bash
   cd ../../..
   python start_dashboard.py --mode web
   ```

## Usage

### Web Dashboard

Access the web dashboard at `http://localhost:5000`

**Features:**
- **Overview Tab**: Real-time system metrics overview
- **Performance Tab**: Historical charts and trends
- **Processes Tab**: Detailed process monitoring
- **Network Tab**: Network interface and connection stats
- **Alerts Tab**: Alert management and history

### CLI Interface

The CLI provides comprehensive monitoring capabilities:

```bash
# Real-time monitoring
python cli/monitor_cli.py monitor --interval 2

# Show current status
python cli/monitor_cli.py status

# View alerts
python cli/monitor_cli.py alerts --hours 24

# Generate report
python cli/monitor_cli.py report --format pdf --hours 24

# Export data
python cli/monitor_cli.py export --format json --hours 48
```

### API Usage

The REST API provides programmatic access:

```bash
# Get current metrics
curl http://localhost:5000/api/metrics/current

# Get historical data
curl http://localhost:5000/api/metrics/history/system?hours=24

# Start/stop monitoring
curl -X POST http://localhost:5000/api/monitoring/start
curl -X POST http://localhost:5000/api/monitoring/stop

# Get alerts
curl http://localhost:5000/api/alerts?hours=24

# Export report
curl http://localhost:5000/api/export/report?format=pdf
```

## Configuration

Configuration is managed through `config/config.yaml`:

```yaml
# Monitoring settings
monitoring:
  interval: 5  # seconds
  history_size: 1000

# Alert thresholds
thresholds:
  cpu:
    warning: 80
    critical: 95
  memory:
    warning: 85
    critical: 95

# Notifications
notifications:
  email:
    enabled: true
    smtp_server: "smtp.gmail.com"
    to_addresses: ["admin@company.com"]
  
  webhook:
    enabled: true
    url: "https://hooks.slack.com/your-webhook"
```

### Custom Metrics

Add custom metrics in the configuration:

```yaml
custom_metrics:
  database_connections:
    function: "lambda: get_database_connection_count()"
    interval: 60
    thresholds:
      warning: 100
      critical: 200
```

## Components

### SystemMonitor (`backend/system_monitor.py`)
Core monitoring engine that collects:
- CPU usage (per-core and overall)
- Memory usage (physical and virtual)
- Disk usage and I/O statistics
- Network statistics and interfaces
- Process information and performance
- Kernel performance counters

### AlertManager (`backend/alert_manager.py`)
Handles alert generation and notification:
- Threshold-based alert generation
- Alert aggregation to prevent spam
- Email, webhook, and log notifications
- Alert acknowledgment and resolution
- Alert statistics and trends

### ReportGenerator (`backend/report_generator.py`)
Generates comprehensive reports:
- PDF reports with charts and analysis
- HTML reports for web viewing
- CSV exports for data analysis
- Performance recommendations
- Statistical analysis

### WebDashboard (`backend/web_dashboard.py`)
Flask-based web server providing:
- REST API for all dashboard features
- WebSocket support for real-time updates
- Interactive charts and visualizations
- Alert management interface
- Configuration management

## Database Schema

The system uses SQLite with the following main tables:

### system_metrics
- timestamp, cpu_percent, cpu_freq, load_avg
- memory_percent, memory_used, memory_total
- disk_usage, disk_io, network_io

### process_metrics
- timestamp, pid, name, cpu_percent, memory_percent
- memory_used, status, num_threads

### custom_metrics
- timestamp, metric_name, metric_value, metric_type, tags

### alerts
- timestamp, alert_type, severity, message
- metric_value, threshold, acknowledged, resolved

## Performance Optimization

### Memory Management
- Configurable history size limits
- Automatic cleanup of old data
- Efficient data structures for real-time processing

### Database Optimization
- Indexed queries for fast lookups
- Batch inserts for better performance
- Automatic database maintenance

### Real-time Updates
- WebSocket for efficient real-time data
- Client-side data caching
- Optimized chart rendering

## Alert System

### Threshold Types
- **Percentage-based**: CPU, memory, disk usage
- **Rate-based**: Network throughput
- **Count-based**: Zombie processes
- **Custom**: Any numeric metric

### Severity Levels
- **Info**: Informational notifications
- **Warning**: Elevated attention required
- **Critical**: Immediate action required

### Aggregation
Alerts are aggregated within a 5-minute window to prevent notification spam while preserving critical information.

## Monitoring Capabilities

### System Metrics
- CPU usage (overall and per-core)
- Memory usage (physical and swap)
- Disk usage (all mounted filesystems)
- Network traffic (per-interface)
- System load average
- Uptime and user information

### Process Monitoring
- Top CPU-consuming processes
- Top memory-consuming processes
- Process tree visualization
- Process I/O statistics
- Thread count tracking
- Zombie process detection

### Network Monitoring
- Interface statistics
- Connection state tracking
- Bandwidth utilization
- Error and drop statistics
- Network protocol breakdown

### Custom Monitoring
- Plugin architecture for custom metrics
- Configurable collection intervals
- Custom threshold definitions
- Custom alert notifications

## Export and Reporting

### Report Formats
- **PDF**: Comprehensive reports with charts
- **HTML**: Interactive web-based reports
- **CSV**: Raw data for external analysis
- **JSON**: Programmatic access format

### Report Content
- System performance summary
- Trend analysis and projections
- Top resource consumers
- Alert history and statistics
- Performance recommendations
- Detailed process analysis

## Troubleshooting

### Common Issues

1. **Permission Errors**
   ```bash
   # Ensure proper permissions
   chmod +x start_dashboard.py
   chmod +x cli/monitor_cli.py
   ```

2. **Database Locked**
   ```bash
   # Stop all processes and restart
   python start_dashboard.py --mode web
   ```

3. **High CPU Usage**
   ```bash
   # Increase monitoring interval
   # Edit config/config.yaml
   monitoring:
     interval: 10  # Increase from 5 to 10 seconds
   ```

4. **Missing Dependencies**
   ```bash
   # Reinstall requirements
   pip install -r requirements.txt
   npm install
   ```

### Logs
Check logs for detailed error information:
- `logs/monitor.log`: General system logs
- `logs/alerts.log`: Alert-specific logs

### Health Checks
```bash
# Check system health
python cli/monitor_cli.py status

# Test web interface
curl http://localhost:5000/api/health
```

## Development

### Adding Custom Metrics

1. Define metric function in `config/config.yaml`:
   ```yaml
   custom_metrics:
     my_custom_metric:
       function: "lambda: get_my_metric()"
       interval: 30
   ```

2. Implement the metric function:
   ```python
   def get_my_metric():
       # Your custom logic here
       return metric_value
   ```

3. Register the metric:
   ```python
   monitor.add_custom_metric('my_custom_metric', get_my_metric)
   ```

### Extending the Web Interface

1. Add new API endpoints in `web_dashboard.py`
2. Create React components in `frontend/src/components/`
3. Update the main Dashboard component
4. Add routing and navigation

### Custom Alert Handlers

```python
def custom_alert_handler(alert):
    # Custom alert processing
    if alert['severity'] == 'critical':
        send_to_emergency_channel(alert)

alert_manager.add_alert_handler(custom_alert_handler)
```

## Security Considerations

- API rate limiting enabled by default
- CORS configuration for cross-origin requests
- Input validation for all API endpoints
- Secure configuration file permissions
- No default authentication (add if needed)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Submit a pull request

### Development Setup
```bash
# Backend development
cd backend
pip install -r requirements-dev.txt

# Frontend development
cd frontend/perf-dashboard
npm install
npm run dev
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review the logs
3. Search existing issues
4. Create a new issue with detailed information

---

**Performance Monitoring Dashboard** - Real-time system performance monitoring with comprehensive analytics and alerting.