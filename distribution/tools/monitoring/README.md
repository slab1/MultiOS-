# MultiOS Monitoring and Logging Infrastructure

A comprehensive monitoring and logging solution specifically designed for production MultiOS deployments in educational and enterprise environments.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Components](#components)
- [API Reference](#api-reference)
- [Educational Features](#educational-features)
- [Security](#security)
- [Performance](#performance)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

## Overview

The MultiOS Monitoring Infrastructure provides real-time system monitoring, centralized logging, educational analytics, and intelligent alerting for distributed MultiOS deployments. Built specifically for educational institutions and enterprise environments, it offers specialized features for lab management, compliance reporting, and resource optimization.

### Key Capabilities

- **Real-time Monitoring**: Live dashboards with WebSocket streaming
- **Centralized Logging**: Log aggregation from multiple sources
- **Educational Analytics**: Specialized tracking for lab usage and student activity
- **Hardware Health**: Comprehensive system and network monitoring
- **Smart Alerting**: Rule-based alerting with multiple notification channels
- **Compliance Reporting**: Educational institution compliance features
- **Multi-site Support**: Distributed deployment monitoring
- **Integration Ready**: Prometheus, Grafana, and other tool integration

## Architecture

The monitoring infrastructure follows a modular architecture designed for scalability and maintainability:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Dashboard │    │  Mobile/Web     │    │  API Clients    │
│   (React/HTML)  │    │  Applications   │    │  (REST/GraphQL) │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   API Gateway (Flask)   │
                    │  • Authentication       │
                    │  • Rate Limiting        │
                    │  • Request Routing      │
                    └────────────┬────────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
    ┌─────▼─────┐          ┌─────▼─────┐          ┌─────▼─────┐
    │ Monitoring│          │    Log    │          │  Alert   │
    │  Agents   │          │ Aggregator│          │ Manager  │
    └─────┬─────┘          └─────┬─────┘          └─────┬─────┘
          │                      │                      │
    ┌─────▼─────┐          ┌─────▼─────┐          ┌─────▼─────┐
    │Educational│          │ Educational│         │ Compliance│
    │Analytics  │          │ Data Store │         │Reporting  │
    └─────┬─────┘          └─────┬─────┘         └─────┬─────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   Data Storage Layer    │
                    │  • Time Series (TSDB)   │
                    │  • Document Store       │
                    │  • Cache (Redis)        │
                    └────────────────────────┘
```

### Component Descriptions

- **Web Dashboard**: Real-time monitoring interface with customizable widgets
- **API Gateway**: RESTful API with WebSocket support for real-time data
- **Monitoring Agents**: Lightweight collectors for system, network, and application metrics
- **Log Aggregator**: Centralized log collection, parsing, and analysis
- **Alert Manager**: Rule-based alerting with escalation and notification management
- **Educational Analytics**: Specialized analytics for educational environments
- **Data Storage**: Time-series database for metrics, document store for logs

## Features

### System Monitoring
- CPU, Memory, Disk, and Network utilization
- Process monitoring and resource tracking
- Hardware health status and temperature monitoring
- System performance profiling and optimization recommendations

### Educational Features
- Lab usage analytics and reporting
- Student activity tracking and engagement metrics
- Resource utilization by course and instructor
- Compliance reporting for educational standards

### Logging and Analysis
- Centralized log collection from multiple sources
- Real-time log parsing and analysis
- Full-text search with advanced filtering
- Log retention and archival policies

### Alerting and Notifications
- Rule-based alerting with customizable thresholds
- Multiple notification channels (Email, Slack, Webhook, SMS)
- Alert escalation and acknowledgment workflows
- Alert history and performance tracking

### Integration Capabilities
- Prometheus metrics integration
- Grafana dashboard templates
- RESTful API for custom integrations
- Export capabilities (CSV, JSON, PDF)

## Prerequisites

### System Requirements
- **Operating System**: Linux (Ubuntu 18.04+, CentOS 7+, Debian 9+)
- **Python**: 3.8 or higher
- **Memory**: Minimum 4GB RAM (8GB recommended for production)
- **Storage**: Minimum 50GB disk space (SSD recommended)
- **Network**: Stable internet connection for external integrations

### Software Dependencies
- Python 3.8+
- Node.js 14+ (for dashboard assets)
- Nginx (recommended for production deployment)
- PostgreSQL 12+ (for configuration storage)
- Redis 6+ (for caching and session management)

### Network Requirements
- Port 80/443 for web dashboard access
- Port 8080 for API server (configurable)
- Port 9090 for Prometheus metrics (optional)
- Port 3000 for Grafana integration (optional)

## Installation

### Quick Start

1. **Clone and Setup**:
   ```bash
   git clone <repository-url>
   cd deployment/monitoring
   chmod +x deploy.sh
   ./deploy.sh --quick-start
   ```

2. **Access Dashboard**:
   Open browser to `http://localhost:8080`
   Default credentials: `admin` / `admin123`

### Detailed Installation

1. **Prerequisites Installation**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install -y python3.8 python3.8-pip python3.8-venv nodejs npm nginx postgresql redis-server

   # CentOS/RHEL
   sudo yum install -y python38 python38-pip nodejs npm nginx postgresql redis
   ```

2. **Clone Repository**:
   ```bash
   git clone <repository-url> /opt/multios-monitoring
   cd /opt/multios-monitoring/deployment/monitoring
   ```

3. **Run Deployment Script**:
   ```bash
   ./deploy.sh --full-install --production
   ```

4. **Configure Environment**:
   ```bash
   cp config/environments/production.env.template config/production.env
   # Edit configuration file
   nano config/production.env
   ```

5. **Start Services**:
   ```bash
   sudo systemctl enable multios-monitoring
   sudo systemctl start multios-monitoring
   ```

### Docker Installation

```bash
# Build and run with Docker Compose
docker-compose up -d

# Or build custom image
docker build -t multios-monitoring .
docker run -d -p 8080:8080 multios-monitoring
```

## Configuration

### Main Configuration File

Edit `config/monitoring.yaml` for system-wide settings:

```yaml
# General Configuration
general:
  environment: production
  log_level: INFO
  data_retention_days: 90
  timezone: UTC

# Dashboard Configuration
dashboard:
  port: 8080
  ssl_enabled: false
  session_timeout: 3600
  theme: light

# Database Configuration
database:
  type: postgresql
  host: localhost
  port: 5432
  name: multios_monitoring
  username: monitor_user
  password: secure_password
  pool_size: 20

# Redis Configuration
redis:
  host: localhost
  port: 6379
  db: 0
  password: null

# Alert Configuration
alerts:
  enabled: true
  check_interval: 60
  max_alerts_per_rule: 100
  escalation_enabled: true
  channels:
    email:
      enabled: true
      smtp_server: smtp.gmail.com
      smtp_port: 587
      username: alerts@yourdomain.com
      password: app_password
```

### Environment-Specific Configurations

The system supports multiple environment configurations:

- **Development**: `config/environments/development.yaml`
- **Staging**: `config/environments/staging.yaml`
- **Production**: `config/environments/production.yaml`
- **Educational Lab**: `config/environments/educational.yaml`

### Alert Rules Configuration

Customize alert rules in `alerting/rules/`:

```yaml
# High CPU Usage
cpu_high:
  name: "High CPU Usage"
  description: "CPU usage exceeds threshold"
  condition: "cpu_usage > 80"
  severity: "warning"
  duration: 300  # 5 minutes
  actions:
    - email: admin@domain.com
    - slack: "#alerts"
    - webhook: "https://hooks.slack.com/services/..."

# Educational Lab Rules
lab_usage_high:
  name: "High Lab Usage"
  description: "Lab computer usage exceeds capacity"
  condition: "lab_usage > 90"
  severity: "critical"
  duration: 180  # 3 minutes
  actions:
    - email: lab-admin@domain.com
    - notify_lab_coordinator: true
```

## Usage

### Web Dashboard

Access the dashboard at `http://localhost:8080` with your configured credentials.

#### Dashboard Sections

1. **System Overview**
   - Real-time system metrics
   - Performance trends
   - Resource utilization charts

2. **Hardware Monitoring**
   - CPU, memory, disk usage
   - Network traffic analysis
   - Hardware health status

3. **Educational Analytics**
   - Lab usage statistics
   - Student engagement metrics
   - Course resource utilization

4. **Alerts and Logs**
   - Active alerts management
   - Log viewer with filtering
   - Alert history

### API Usage

The system provides RESTful API endpoints for integration:

#### Authentication

```bash
# Login to get JWT token
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}'
```

#### System Metrics

```bash
# Get current system metrics
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/metrics/system

# Get historical CPU data
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/metrics/system/cpu/history?range=24h
```

#### Educational Data

```bash
# Get lab usage statistics
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/analytics/lab/usage

# Get student activity data
curl -H "Authorization: Bearer <token>" \
  http://localhost:8080/api/analytics/students/activity
```

### Command Line Interface

#### Monitoring Agent Management

```bash
# Start specific agent
python backend/agents.py --agent system --start

# Stop monitoring
python backend/agents.py --stop

# Check agent status
python backend/agents.py --status
```

#### Log Management

```bash
# View real-time logs
python logging/log_aggregator.py --tail

# Search logs
python logging/log_aggregator.py --search "error" --last 1h

# Export logs
python logging/log_aggregator.py --export --format csv --range 7d
```

#### Alert Management

```bash
# Test alert rule
python alerting/alert_manager.py --test-rule cpu_high

# List active alerts
python alerting/alert_manager.py --list-active

# Acknowledge alert
python alerting/alert_manager.py --acknowledge <alert_id>
```

## Components

### Monitoring Agents

Located in `backend/agents.py`:

- **SystemAgent**: CPU, memory, disk, network metrics
- **HardwareAgent**: Hardware health and temperature
- **NetworkAgent**: Network traffic and connectivity
- **EducationalAgent**: Lab usage and student activity
- **SecurityAgent**: Security events and intrusion detection

### Dashboard Components

Located in `dashboard/`:

- **index.html**: Main dashboard interface
- **dashboard.js**: Core dashboard functionality
- **charts.js**: Chart rendering and visualization
- **websocket.js**: Real-time data streaming
- **export.js**: Data export capabilities

### Backend Services

Located in `backend/`:

- **server.py**: Flask API server with WebSocket support
- **agents.py**: Monitoring agent management
- Authentication and authorization middleware
- Rate limiting and security features

### Logging System

Located in `logging/`:

- **log_aggregator.py**: Central log collection and analysis
- Log parsers for various formats (syslog, JSON, custom)
- Log analyzers for pattern detection
- Retention and archival policies

### Educational Features

Located in `educational/`:

- **analytics.py**: Educational analytics engine
- Lab usage tracking and reporting
- Student engagement metrics
- Compliance and audit reporting

## API Reference

### Authentication Endpoints

```
POST /api/auth/login
POST /api/auth/logout
GET  /api/auth/profile
POST /api/auth/refresh
```

### Metrics Endpoints

```
GET  /api/metrics/system/{metric_type}
GET  /api/metrics/system/{metric_type}/history
POST /api/metrics/custom
GET  /api/metrics/health
```

### Educational Endpoints

```
GET  /api/analytics/lab/usage
GET  /api/analytics/lab/statistics
GET  /api/analytics/students/activity
GET  /api/analytics/students/engagement
POST /api/analytics/students/checkin
```

### Logging Endpoints

```
GET  /api/logs/search
GET  /api/logs/export
GET  /api/logs/statistics
POST /api/logs/ingest
```

### Alert Endpoints

```
GET  /api/alerts/active
GET  /api/alerts/history
POST /api/alerts/acknowledge
GET  /api/alerts/rules
POST /api/alerts/rules
```

### WebSocket Endpoints

```
WS /ws/realtime/{metric_type}
WS /ws/logs
WS /ws/alerts
WS /ws/educational
```

## Educational Features

### Lab Management

- Real-time lab computer availability tracking
- Reservation system integration
- Usage patterns and optimization recommendations
- Capacity planning based on historical data

### Student Activity Tracking

- Login/logout times and duration
- Application usage and resource consumption
- Academic progress correlation
- Engagement metrics and trends

### Compliance Reporting

- FERPA compliance features
- Audit trail maintenance
- Data retention policies
- Privacy controls and access logging

### Resource Optimization

- Peak usage time analysis
- Resource allocation recommendations
- Cost optimization suggestions
- Performance benchmarking

## Security

### Authentication and Authorization

- JWT-based authentication
- Role-based access control (RBAC)
- Multi-factor authentication support
- Session management and timeout

### Data Protection

- Encrypted data transmission (TLS/SSL)
- At-rest encryption for sensitive data
- Secure credential storage
- Privacy controls for educational data

### Network Security

- API rate limiting
- IP whitelist/blacklist
- Security headers and CSRF protection
- Secure configuration management

### Audit and Compliance

- Comprehensive audit logging
- Change tracking and versioning
- Compliance reporting tools
- Data residency controls

## Performance

### Optimization Features

- Efficient data structures and algorithms
- Connection pooling and caching
- Asynchronous processing
- Database query optimization

### Scalability

- Horizontal scaling support
- Load balancing capabilities
- Microservices architecture
- Resource auto-scaling

### Monitoring Performance

- System performance metrics collection
- Application performance monitoring
- Database query analysis
- Network latency tracking

## Troubleshooting

For detailed troubleshooting information, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

### Common Issues

1. **Dashboard not loading**: Check API server status and network connectivity
2. **Missing metrics**: Verify agent configuration and data collection
3. **Alert notifications not working**: Check notification channel configuration
4. **High memory usage**: Review data retention policies and optimize queries
5. **Slow performance**: Check database indexes and query optimization

### Debugging Commands

```bash
# Check system status
python scripts/health_check.py

# Verify configuration
python scripts/validate_config.py

# Test database connectivity
python scripts/test_db_connection.py

# Debug agent issues
python backend/agents.py --debug --verbose
```

## Contributing

### Development Setup

1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Install development dependencies: `pip install -r requirements-dev.txt`
4. Run tests: `python -m pytest tests/`
5. Submit pull request

### Code Standards

- Follow PEP 8 for Python code
- Use type hints for better code documentation
- Include comprehensive unit tests
- Update documentation for new features

### Testing

```bash
# Run all tests
python -m pytest tests/

# Run specific test suite
python -m pytest tests/unit/
python -m pytest tests/integration/

# Run with coverage
python -m pytest --cov=backend tests/
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:

- **Documentation**: Check this README and TROUBLESHOOTING.md
- **Issues**: Submit through GitHub issues
- **Email**: support@multios-monitoring.org
- **Community**: Join our Discord server

## Changelog

### v1.0.0 (Current)
- Initial release with comprehensive monitoring capabilities
- Educational analytics and compliance features
- Web dashboard with real-time updates
- Multi-site monitoring support
- Prometheus and Grafana integration

---

**MultiOS Monitoring Infrastructure** - Built for educational excellence and enterprise reliability.
