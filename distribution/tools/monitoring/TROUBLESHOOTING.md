# MultiOS Monitoring Infrastructure - Troubleshooting Guide

This guide provides solutions for common issues encountered when deploying, configuring, and operating the MultiOS Monitoring Infrastructure.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Service Startup Problems](#service-startup-problems)
- [Dashboard Access Issues](#dashboard-access-issues)
- [Monitoring Data Issues](#monitoring-data-issues)
- [Alert System Problems](#alert-system-problems)
- [Database Issues](#database-issues)
- [Performance Problems](#performance-problems)
- [Network and Connectivity](#network-and-connectivity)
- [Security and Authentication](#security-and-authentication)
- [Educational Features Issues](#educational-features-issues)
- [Log Analysis](#log-analysis)
- [System Maintenance](#system-maintenance)
- [Emergency Procedures](#emergency-procedures)
- [Getting Help](#getting-help)

## Installation Issues

### Python Version Conflicts

**Problem**: Multiple Python versions installed, wrong version being used.

**Symptoms**:
```
python3: command not found
ModuleNotFoundError: No module named 'flask'
```

**Solutions**:
```bash
# Check available Python versions
ls /usr/bin/python*

# Install specific Python version (Ubuntu/Debian)
sudo apt-get install python3.8 python3.8-pip python3.8-venv

# Create virtual environment with specific version
python3.8 -m venv /opt/multios-monitoring/venv

# Activate virtual environment
source /opt/multios-monitoring/venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

**Prevention**:
```bash
# Set default Python version
sudo update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.8 1
```

### Permission Denied Errors

**Problem**: Permission errors during installation.

**Symptoms**:
```
Permission denied: '/etc/systemd/system/multios-monitoring.service'
```

**Solutions**:
```bash
# Fix script permissions
chmod +x deploy.sh

# Run with appropriate permissions
sudo ./deploy.sh --full-install

# Create required directories with proper permissions
sudo mkdir -p /opt/multios-monitoring
sudo chown $USER:$USER /opt/multios-monitoring
```

### Missing System Dependencies

**Problem**: Required system packages missing.

**Symptoms**:
```
E: Unable to locate package python3-psutil
```

**Solutions**:
```bash
# Update package lists
sudo apt-get update

# Install missing dependencies (Ubuntu/Debian)
sudo apt-get install -y \
    python3-pip python3-dev python3-venv \
    python3-psutil python3-numpy python3-pandas \
    postgresql postgresql-dev redis-server \
    nginx nodejs npm

# For CentOS/RHEL
sudo yum install -y \
    python3-pip python3-devel \
    python3-psutil python3-numpy python3-pandas \
    postgresql postgresql-devel redis \
    nginx nodejs npm
```

### Network Installation Failures

**Problem**: Network timeouts or connection issues during installation.

**Solutions**:
```bash
# Test network connectivity
ping google.com

# Use alternative package mirrors
sudo sed -i 's|http://archive.ubuntu.com|https://mirrors.kernel.org|g' /etc/apt/sources.list

# Install from offline packages
./deploy.sh --offline-install --local-packages /path/to/packages/

# Configure proxy if needed
export http_proxy=http://proxy.company.com:8080
export https_proxy=http://proxy.company.com:8080
```

## Service Startup Problems

### Service Won't Start

**Problem**: Systemd service fails to start.

**Diagnosis**:
```bash
# Check service status
sudo systemctl status multios-monitoring

# View detailed logs
sudo journalctl -u multios-monitoring -n 50 --no-pager

# Check configuration files
sudo /opt/multios-monitoring/venv/bin/python backend/server.py --check-config
```

**Common Solutions**:

1. **Port Already in Use**:
```bash
# Find process using port 8080
sudo netstat -tulpn | grep :8080

# Kill conflicting process
sudo kill -9 <PID>

# Or change port in configuration
# Edit /etc/multios-monitoring/monitoring.yaml
```

2. **Missing Dependencies**:
```bash
# Activate virtual environment
source /opt/multios-monitoring/venv/bin/activate

# Reinstall dependencies
pip install -r requirements.txt

# Check for missing packages
pip check
```

3. **Database Connection Issues**:
```bash
# Test database connection
sudo -u postgres psql -c "SELECT version();"

# Reset database password
sudo -u postgres psql
ALTER USER monitor_user WITH PASSWORD 'new_secure_password';
\q

# Update configuration
sudo nano /etc/multios-monitoring/production.env
```

4. **File Permissions**:
```bash
# Fix ownership
sudo chown -R multios:multios /opt/multios-monitoring/
sudo chown -R multios:multios /var/lib/multios-monitoring/
sudo chown -R multios:multios /var/log/multios-monitoring/

# Fix permissions
sudo chmod +x /opt/multios-monitoring/backend/server.py
sudo chmod 600 /etc/multios-monitoring/*.env
```

### Service Starts but Crashes

**Problem**: Service starts but immediately stops.

**Diagnosis**:
```bash
# Run in foreground to see errors
sudo -u multios /opt/multios-monitoring/venv/bin/python backend/server.py

# Check system resources
free -h
df -h
```

**Solutions**:
1. **Insufficient Memory**: Increase system RAM or swap space
2. **Disk Space**: Clean up logs and temporary files
3. **Configuration Errors**: Validate YAML configuration syntax

### High CPU/Memory Usage

**Problem**: Service consuming excessive resources.

**Solutions**:
```bash
# Monitor resource usage
top -p $(pgrep -f "multios-monitoring")

# Adjust worker processes in configuration
# Edit config/environments/production.yaml
workers: 2  # Reduce from default
worker_connections: 1000
```

## Dashboard Access Issues

### Dashboard Not Loading

**Problem**: Web interface not accessible.

**Diagnosis**:
```bash
# Test API endpoint
curl http://localhost:8080/api/health

# Check if service is running
sudo systemctl status multios-monitoring

# Check firewall rules
sudo ufw status
sudo firewall-cmd --list-all
```

**Solutions**:
1. **Nginx Configuration Issues**:
```bash
# Test nginx configuration
sudo nginx -t

# Restart nginx
sudo systemctl restart nginx

# Check nginx logs
sudo tail -f /var/log/nginx/error.log
```

2. **Firewall Blocking Access**:
```bash
# Allow HTTP/HTTPS traffic
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# For firewalld
sudo firewall-cmd --permanent --add-service=http
sudo firewall-cmd --permanent --add-service=https
sudo firewall-cmd --reload
```

3. **Service Not Listening**:
```bash
# Check listening ports
sudo netstat -tulpn | grep :8080

# Verify binding address
# Edit config/monitoring.yaml
dashboard:
  bind_address: "0.0.0.0"  # Allow external connections
  port: 8080
```

### Authentication Problems

**Problem**: Cannot log in to dashboard.

**Solutions**:
1. **Reset Default Password**:
```bash
# Generate new password hash
python3 -c "
import bcrypt
password = 'new_secure_password'
hashed = bcrypt.hashpw(password.encode('utf-8'), bcrypt.gensalt())
print(hashed.decode('utf-8'))
"

# Update database
sudo -u postgres psql multios_monitoring
UPDATE users SET password_hash='<new_hash>' WHERE username='admin';
```

2. **Check JWT Configuration**:
```bash
# Verify JWT secret
grep JWT_SECRET /etc/multios-monitoring/production.env

# Regenerate if compromised
python3 -c "import secrets; print(secrets.token_hex(32))"
```

3. **Clear Browser Cache**:
   - Clear browser cookies and local storage
   - Try incognito/private browsing mode

### SSL/TLS Certificate Issues

**Problem**: SSL certificate errors or warnings.

**Solutions**:
1. **Self-Signed Certificate**:
```bash
# Generate new self-signed certificate
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/multios-monitoring.key \
  -out /etc/ssl/certs/multios-monitoring.crt

# Update nginx configuration
sudo nano /etc/nginx/sites-available/multios-monitoring
```

2. **Let's Encrypt Certificate**:
```bash
# Install certbot
sudo apt-get install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d your-domain.com

# Auto-renewal
sudo crontab -e
0 12 * * * /usr/bin/certbot renew --quiet
```

## Monitoring Data Issues

### No Metrics Data

**Problem**: Dashboard shows no system metrics.

**Diagnosis**:
```bash
# Check monitoring agents
sudo systemctl status multios-monitoring

# Test agent directly
sudo -u multios /opt/multios-monitoring/venv/bin/python backend/agents.py --test

# Check metrics API
curl http://localhost:8080/api/metrics/system/cpu
```

**Solutions**:
1. **Agent Configuration**:
```bash
# Verify agent configuration
# Edit config/monitoring.yaml
agents:
  enabled: true
  interval: 60  # seconds
  system_agent:
    enabled: true
    metrics:
      - cpu
      - memory
      - disk
      - network
```

2. **Database Connection**:
```bash
# Test database connectivity
sudo -u multios /opt/multios-monitoring/venv/bin/python -c "
import psycopg2
conn = psycopg2.connect('postgresql://monitor_user:password@localhost/multios_monitoring')
print('Database connection successful')
conn.close()
"
```

3. **Permission Issues**:
```bash
# Check agent permissions
sudo setcap cap_sys_ptrace=eip /opt/multios-monitoring/venv/bin/python

# Add user to required groups
sudo usermod -a -G disk,netdev multios
```

### Inaccurate Metrics

**Problem**: System metrics seem incorrect or delayed.

**Solutions**:
1. **Update Collection Interval**:
```yaml
# In config/monitoring.yaml
agents:
  collection_interval: 30  # Reduce for more frequent updates
  cpu_agent:
    sample_rate: 5  # 5-second intervals
```

2. **Check System Clock**:
```bash
# Sync system time
sudo ntpdate -s time.nist.gov

# Verify timezone
timedatectl status
```

3. **Resource Monitoring**:
```bash
# Monitor agent resource usage
htop -p $(pgrep -f agents.py)

# Adjust agent priority
sudo nice -n -10 /opt/multios-monitoring/venv/bin/python backend/agents.py
```

### Network Monitoring Not Working

**Problem**: Network metrics not collected.

**Solutions**:
1. **Install Network Tools**:
```bash
# Install required packages
sudo apt-get install -y ethtool net-tools iftop nethogs

# Check network interfaces
ip link show
ifconfig
```

2. **Network Permissions**:
```bash
# Add user to network group
sudo usermod -a -G netdev multios

# Set capabilities for raw socket access
sudo setcap cap_net_raw,cap_net_admin=eip /opt/multios-monitoring/venv/bin/python
```

## Alert System Problems

### Alerts Not Triggering

**Problem**: Expected alerts not being sent.

**Diagnosis**:
```bash
# Check alert manager status
sudo systemctl status multios-monitoring

# Test alert rule manually
sudo -u multios /opt/multios-monitoring/venv/bin/python alerting/alert_manager.py --test-rule cpu_high

# Check alert logs
sudo tail -f /var/log/multios-monitoring/alerts.log
```

**Solutions**:
1. **Rule Configuration**:
```yaml
# In config/monitoring.yaml
alerts:
  rules:
    cpu_high:
      condition: "cpu_usage > 80"
      duration: 300  # 5 minutes
      severity: "warning"
```

2. **Notification Channels**:
```bash
# Test email notification
sudo -u multios /opt/multios-monitoring/venv/bin/python -c "
from alerting.notification_channels import EmailChannel
channel = EmailChannel()
channel.send('Test Alert', 'This is a test alert')
"

# Test webhook
curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  -H 'Content-type: application/json' \
  --data '{"text":"Test alert from MultiOS Monitoring"}'
```

3. **Check Alert History**:
```bash
# View alert history
sudo -u postgres psql multios_monitoring
SELECT * FROM alerts ORDER BY created_at DESC LIMIT 10;
```

### Alert Spam

**Problem**: Receiving too many alert notifications.

**Solutions**:
1. **Increase Thresholds**:
```yaml
# Adjust alert thresholds
alerts:
  rules:
    high_memory:
      condition: "memory_usage > 95"
      duration: 600  # 10 minutes
```

2. **Add Suppression**:
```yaml
# Suppress alerts during maintenance
maintenance_mode: true
suppress_alerts: true
```

3. **Rate Limiting**:
```yaml
# Limit alert frequency
alerts:
  rate_limit: 10  # Max 10 alerts per hour per rule
```

## Database Issues

### Database Connection Errors

**Problem**: Cannot connect to PostgreSQL database.

**Diagnosis**:
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection
psql -h localhost -U monitor_user -d multios_monitoring

# Check logs
sudo tail -f /var/log/postgresql/postgresql-*.log
```

**Solutions**:
1. **Restart PostgreSQL**:
```bash
sudo systemctl restart postgresql
sudo systemctl enable postgresql
```

2. **Fix Connection String**:
```bash
# Check connection parameters
cat /etc/multios-monitoring/production.env

# Update password if needed
sudo -u postgres psql
ALTER USER monitor_user WITH PASSWORD 'new_password';
\q
```

3. **Increase Connections**:
```bash
# Edit PostgreSQL configuration
sudo nano /etc/postgresql/*/main/postgresql.conf
max_connections = 200

# Restart PostgreSQL
sudo systemctl restart postgresql
```

### Database Performance Issues

**Problem**: Slow database queries or high load.

**Solutions**:
1. **Add Indexes**:
```sql
-- Common indexes for monitoring data
CREATE INDEX idx_metrics_timestamp ON metrics(timestamp);
CREATE INDEX idx_alerts_created_at ON alerts(created_at);
CREATE INDEX idx_logs_timestamp ON logs(timestamp);
```

2. **Optimize Queries**:
```sql
-- Analyze slow queries
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;
```

3. **Connection Pooling**:
```yaml
# In config/monitoring.yaml
database:
  pool_size: 20
  max_overflow: 30
  pool_timeout: 30
```

### Database Disk Space Issues

**Problem**: Database disk space running low.

**Solutions**:
```bash
# Check database size
sudo -u postgres psql
\l+ multios_monitoring

# Clean old data
# In config/monitoring.yaml
data_retention:
  metrics: 30  # days
  logs: 7      # days
  alerts: 90   # days

# Manual cleanup
sudo -u postgres psql multios_monitoring
DELETE FROM metrics WHERE timestamp < NOW() - INTERVAL '30 days';
VACUUM FULL;
```

## Performance Problems

### High CPU Usage

**Problem**: System CPU usage consistently high.

**Solutions**:
1. **Optimize Collection Intervals**:
```yaml
# Reduce sampling frequency
agents:
  collection_interval: 120  # 2 minutes
  cpu_agent:
    sample_rate: 30  # 30 seconds
```

2. **Reduce Data Retention**:
```yaml
# Keep less historical data
data_retention:
  metrics: 7   # days
  logs: 3      # days
```

3. **Enable Caching**:
```yaml
# Enable Redis caching
cache:
  enabled: true
  ttl: 300  # 5 minutes
```

### High Memory Usage

**Problem**: System running out of memory.

**Solutions**:
1. **Reduce Worker Processes**:
```yaml
# In production config
workers: 1
worker_connections: 500
```

2. **Enable Swap**:
```bash
# Create swap file
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Make permanent
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

3. **Memory Monitoring**:
```bash
# Monitor memory usage
free -h
ps aux --sort=-%mem | head -10
```

### Slow Database Queries

**Problem**: Dashboard loading slowly.

**Solutions**:
1. **Add Database Indexes**:
```sql
-- Performance indexes
CREATE INDEX CONCURRENTLY idx_metrics_timestamp 
ON metrics(timestamp DESC);

CREATE INDEX CONCURRENTLY idx_logs_level_timestamp 
ON logs(level, timestamp DESC);
```

2. **Query Optimization**:
```sql
-- Analyze query plans
EXPLAIN ANALYZE SELECT * FROM metrics 
WHERE timestamp > NOW() - INTERVAL '1 hour' 
ORDER BY timestamp DESC;
```

3. **Database Configuration**:
```bash
# Adjust PostgreSQL settings
sudo nano /etc/postgresql/*/main/postgresql.conf
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
```

## Network and Connectivity

### WebSocket Connection Issues

**Problem**: Real-time updates not working.

**Solutions**:
1. **Proxy Configuration**:
```nginx
# In nginx configuration
location /ws/ {
    proxy_pass http://127.0.0.1:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
}
```

2. **Firewall Rules**:
```bash
# Allow WebSocket traffic
sudo ufw allow from 192.168.1.0/24 to any port 8080
```

3. **Client-Side Issues**:
```javascript
// Check browser console for errors
// Ensure WebSocket URL is correct
const ws = new WebSocket('ws://localhost:8080/ws/realtime');
```

### Remote Agent Connection Issues

**Problem**: Cannot connect remote monitoring agents.

**Solutions**:
1. **Network Connectivity**:
```bash
# Test network connectivity
telnet monitoring-server 8080
nc -zv monitoring-server 8080

# Check firewall
sudo ufw allow from <agent-ip> to any port 8080
```

2. **Authentication**:
```yaml
# In agent configuration
agent:
  api_key: "your-secret-api-key"
  server_url: "https://monitoring.company.com"
  verify_ssl: true
```

3. **SSL/TLS Issues**:
```bash
# Test SSL connection
openssl s_client -connect monitoring-server:8080

# Install certificates
sudo cp ca-certificates.crt /etc/ssl/certs/
sudo update-ca-certificates
```

## Security and Authentication

### SSL Certificate Problems

**Problem**: SSL certificate errors.

**Solutions**:
1. **Check Certificate Expiry**:
```bash
# Check certificate
openssl x509 -in /etc/ssl/certs/multios-monitoring.crt -text -noout

# Renew Let's Encrypt certificate
sudo certbot renew
```

2. **Generate New Self-Signed Certificate**:
```bash
# Generate new certificate
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/multios-monitoring.key \
  -out /etc/ssl/certs/multios-monitoring.crt
```

3. **Certificate Permissions**:
```bash
# Set proper permissions
sudo chmod 600 /etc/ssl/private/multios-monitoring.key
sudo chmod 644 /etc/ssl/certs/multios-monitoring.crt
sudo chown root:root /etc/ssl/private/multios-monitoring.key
```

### Failed Login Attempts

**Problem**: Too many failed login attempts.

**Solutions**:
1. **Check Fail2ban Status**:
```bash
# View banned IPs
sudo fail2ban-client status multios-monitoring

# Unban IP
sudo fail2ban-client set multios-monitoring unbanip <IP>
```

2. **Reset User Password**:
```bash
# Reset password in database
sudo -u postgres psql multios_monitoring
UPDATE users SET failed_attempts = 0, locked_until = NULL WHERE username = 'admin';
```

3. **Disable Account Lockout (Temporary)**:
```yaml
# In config/monitoring.yaml
security:
  max_failed_attempts: 10
  lockout_duration: 1800  # 30 minutes
```

## Educational Features Issues

### Lab Usage Data Not Updating

**Problem**: Educational analytics not showing current data.

**Solutions**:
1. **Check Educational Agent**:
```bash
# Test educational agent
sudo -u multios /opt/multios-monitoring/venv/bin/python educational/analytics.py --test

# Check logs
sudo tail -f /var/log/multios-monitoring/educational.log
```

2. **Database Permissions**:
```bash
# Grant permissions to educational tables
sudo -u postgres psql multios_monitoring
GRANT ALL PRIVILEGES ON TABLE lab_usage TO monitor_user;
GRANT ALL PRIVILEGES ON TABLE student_activity TO monitor_user;
```

3. **Configuration Issues**:
```yaml
# In config/monitoring.yaml
educational:
  agents:
    lab_usage:
      enabled: true
      update_interval: 300  # 5 minutes
    student_activity:
      enabled: true
      tracking_enabled: true
```

### Compliance Reporting Errors

**Problem**: Compliance reports not generating.

**Solutions**:
1. **Check Report Generator**:
```bash
# Test report generation
sudo -u multios /opt/multios-monitoring/venv/bin/python educational/compliance.py --test-report

# Check permissions
sudo chmod +x educational/compliance.py
```

2. **Missing Data**:
```sql
-- Check if required data exists
SELECT COUNT(*) FROM lab_usage;
SELECT COUNT(*) FROM student_activity;
```

3. **Report Configuration**:
```yaml
# In config/environments/production.yaml
compliance:
  reports:
    ferpa:
      enabled: true
      retention_period: 2555  # 7 years
      audit_trail: true
```

## Log Analysis

### Analyzing Application Logs

**Problem**: Need to troubleshoot specific issues using logs.

**Solutions**:
```bash
# View real-time logs
sudo tail -f /var/log/multios-monitoring/application.log

# Search for errors
sudo grep -i error /var/log/multios-monitoring/application.log

# Search specific time range
sudo grep "2023-10-01 10:00" /var/log/multios-monitoring/application.log

# Analyze log patterns
sudo grep -c "ERROR" /var/log/multios-monitoring/application.log
```

### Log Rotation Issues

**Problem**: Logs not being rotated or disk space running out.

**Solutions**:
```bash
# Check logrotate configuration
sudo logrotate -d /etc/logrotate.d/multios-monitoring

# Force log rotation
sudo logrotate -f /etc/logrotate.d/multios-monitoring

# Check disk usage
du -sh /var/log/multios-monitoring/
```

### Centralized Logging Setup

**Problem**: Want to send logs to external system.

**Solutions**:
```yaml
# In config/monitoring.yaml
logging:
  destinations:
    - type: syslog
      server: log-server.company.com
      port: 514
    - type: elasticsearch
      host: elasticsearch.company.com
      index: multios-monitoring
```

## System Maintenance

### Backup and Restore

**Problem**: Need to backup monitoring data.

**Solutions**:
```bash
# Backup database
sudo -u postgres pg_dump multios_monitoring > backup_$(date +%Y%m%d).sql

# Backup configuration
sudo tar -czf config_backup_$(date +%Y%m%d).tar.gz /etc/multios-monitoring/

# Restore from backup
sudo -u postgres psql multios_monitoring < backup_20231001.sql
sudo tar -xzf config_backup_20231001.tar.gz -C /
```

### System Updates

**Problem**: Apply security updates to monitoring system.

**Solutions**:
```bash
# Update system packages
sudo apt-get update && sudo apt-get upgrade

# Update Python dependencies
source /opt/multios-monitoring/venv/bin/activate
pip install -r requirements.txt --upgrade

# Restart services
sudo systemctl restart multios-monitoring
```

### Performance Monitoring

**Problem**: Monitor system performance for optimization.

**Solutions**:
```bash
# System resource monitoring
htop
iotop
iftop

# Monitor specific process
pidstat -p $(pgrep -f multios-monitoring) 1

# Database performance
sudo -u postgres psql multios_monitoring
SELECT * FROM pg_stat_activity;
```

## Emergency Procedures

### Service Outage Recovery

**Problem**: Complete service outage.

**Emergency Steps**:
1. **Check System Status**:
```bash
sudo systemctl status postgresql redis nginx multios-monitoring
sudo systemctl list-failed
```

2. **Emergency Restart**:
```bash
sudo systemctl restart postgresql
sudo systemctl restart redis
sudo systemctl restart nginx
sudo systemctl restart multios-monitoring
```

3. **Emergency Access**:
```bash
# Direct API access
curl http://localhost:8080/api/health

# Direct database access
sudo -u postgres psql multios_monitoring
```

### Data Recovery

**Problem**: Data corruption or loss.

**Recovery Steps**:
1. **Stop Services**:
```bash
sudo systemctl stop multios-monitoring
```

2. **Restore from Backup**:
```bash
# Restore database
sudo -u postgres dropdb multios_monitoring
sudo -u postgres createdb multios_monitoring
sudo -u postgres psql multios_monitoring < backup_20231001.sql
```

3. **Verify and Restart**:
```bash
# Verify data integrity
sudo -u postgres psql multios_monitoring -c "SELECT COUNT(*) FROM metrics;"

# Start services
sudo systemctl start multios-monitoring
```

### Security Incident Response

**Problem**: Security breach or compromised system.

**Response Steps**:
1. **Isolate System**:
```bash
# Block external access
sudo ufw deny incoming
```

2. **Preserve Evidence**:
```bash
# Backup logs
sudo cp -r /var/log/multios-monitoring/ /backup/security-incident-$(date +%Y%m%d)/
```

3. **Change Credentials**:
```bash
# Regenerate all secrets
python3 -c "import secrets; print(secrets.token_hex(32))"

# Update all passwords
sudo -u postgres psql multios_monitoring
UPDATE users SET password_hash = '<new_hash>';
```

## Getting Help

### Collect Diagnostic Information

Before contacting support, gather this information:

```bash
# System information
uname -a
cat /etc/os-release
free -h
df -h

# Service status
sudo systemctl status postgresql redis nginx multios-monitoring

# Recent logs
sudo journalctl -u multios-monitoring --since "1 hour ago" > diagnostics.log

# Configuration
tar -czf configuration.tar.gz /etc/multios-monitoring/

# Database status
sudo -u postgres psql -c "SELECT version();"
```

### Log Files Location

Key log files to check:
- Application logs: `/var/log/multios-monitoring/application.log`
- Agent logs: `/var/log/multios-monitoring/agents.log`
- Alert logs: `/var/log/multios-monitoring/alerts.log`
- Web server logs: `/var/log/nginx/access.log` and `/var/log/nginx/error.log`
- System logs: `sudo journalctl -u multios-monitoring`

### Useful Commands Reference

```bash
# Service management
sudo systemctl status multios-monitoring
sudo systemctl restart multios-monitoring
sudo journalctl -u multios-monitoring -f

# Database management
sudo -u postgres psql multios_monitoring
sudo pg_dump multios_monitoring > backup.sql

# Configuration validation
sudo /opt/multios-monitoring/venv/bin/python backend/server.py --validate-config

# Health checks
curl http://localhost:8080/api/health
curl http://localhost:8080/api/metrics/system

# Log analysis
sudo tail -f /var/log/multios-monitoring/application.log
sudo grep ERROR /var/log/multios-monitoring/application.log

# Performance monitoring
top -p $(pgrep -f multios-monitoring)
htop
```

### Professional Support

For enterprise support and consulting:
- Email: support@multios-monitoring.org
- Phone: +1-XXX-XXX-XXXX
- Hours: 24/7 for critical issues
- SLA: 99.9% uptime guarantee

### Community Support

- GitHub Issues: [Project Repository]
- Documentation: [Official Documentation Site]
- Forum: [Community Forum]
- Discord: [Community Discord Server]

---

**Remember**: Always backup your data before making changes, and test changes in a development environment first when possible.
