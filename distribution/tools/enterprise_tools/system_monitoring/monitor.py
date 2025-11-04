"""
System Monitoring and Health Check Management System
"""

import os
import json
import logging
import subprocess
import threading
import time
import psutil
import socket
from pathlib import Path
from typing import Dict, List, Optional, Any, Callable
from datetime import datetime, timedelta
from concurrent.futures import ThreadPoolExecutor
import smtplib
from email.mime.text import MimeText
from email.mime.multipart import MimeMultipart

from ..core.models import SystemInfo, SystemStatus, HealthCheck
from ..core.utils import execute_command, ping_host, is_port_open

class SystemMonitor:
    """System monitoring and health check manager"""
    
    def __init__(self, config_path: str = "/etc/multios-enterprise/monitoring.yaml"):
        self.config_path = config_path
        self.sites = {}
        self.monitored_systems = {}
        self.health_checks = {}
        self.alert_handlers = {}
        self.monitoring_active = False
        self.monitor_thread = None
        self.logger = logging.getLogger(__name__)
        
        self._load_configuration()
        self._setup_directories()
        self._setup_alert_handlers()
    
    def _load_configuration(self) -> None:
        """Load monitoring configuration"""
        # Default configuration
        self.config = {
            'monitoring': {
                'interval': 300,  # 5 minutes
                'timeout': 30,
                'max_workers': 50
            },
            'thresholds': {
                'cpu_usage': 80.0,
                'memory_usage': 85.0,
                'disk_usage': 90.0,
                'network_latency': 100.0,  # milliseconds
                'process_count': 1000
            },
            'health_checks': {
                'enabled': True,
                'check_interval': 600,  # 10 minutes
                'services_to_check': ['ssh', 'http', 'https', 'dns']
            },
            'alerts': {
                'enabled': True,
                'email_notifications': {
                    'enabled': False,
                    'smtp_server': '',
                    'smtp_port': 587,
                    'username': '',
                    'password': '',
                    'recipients': []
                },
                'slack_notifications': {
                    'enabled': False,
                    'webhook_url': '',
                    'channel': '#alerts'
                }
            },
            'retention': {
                'health_check_history_days': 30,
                'performance_data_hours': 168  # 7 days
            }
        }
        
        # Load configuration file if exists
        if os.path.exists(self.config_path):
            try:
                import yaml
                with open(self.config_path, 'r') as f:
                    loaded_config = yaml.safe_load(f)
                self.config.update(loaded_config)
            except Exception as e:
                self.logger.warning(f"Failed to load monitoring config: {e}")
    
    def _setup_directories(self) -> None:
        """Create monitoring directories"""
        directories = [
            "/var/lib/multios-enterprise/monitoring",
            "/var/lib/multios-enterprise/health_checks",
            "/var/lib/multios-enterprise/performance_data",
            "/var/log/multios-enterprise/monitoring",
            "/var/cache/multios-enterprise/alerts"
        ]
        
        for directory in directories:
            Path(directory).mkdir(parents=True, exist_ok=True)
    
    def _setup_alert_handlers(self) -> None:
        """Setup alert notification handlers"""
        # Email alert handler
        if self.config['alerts']['email_notifications']['enabled']:
            self.alert_handlers['email'] = self._send_email_alert
        
        # Slack alert handler
        if self.config['alerts']['slack_notifications']['enabled']:
            self.alert_handlers['slack'] = self._send_slack_alert
        
        # Log alert handler (always available)
        self.alert_handlers['log'] = self._log_alert
    
    def add_site(self, site_config: 'SiteConfig') -> None:
        """Add a site to monitoring"""
        self.sites[site_config.site_id] = site_config
        self.logger.info(f"Added site {site_config.name} to monitoring")
    
    def enable_monitoring(self, system_id: str) -> bool:
        """Enable monitoring for a system"""
        try:
            if system_id in self.monitored_systems:
                self.logger.warning(f"System {system_id} already being monitored")
                return True
            
            # Create monitoring configuration for system
            monitor_config = {
                'system_id': system_id,
                'enabled': True,
                'last_check': None,
                'consecutive_failures': 0,
                'alert_sent': False
            }
            
            self.monitored_systems[system_id] = monitor_config
            
            self.logger.info(f"Enabled monitoring for system {system_id}")
            return True
        except Exception as e:
            self.logger.error(f"Failed to enable monitoring for {system_id}: {e}")
            return False
    
    def disable_monitoring(self, system_id: str) -> bool:
        """Disable monitoring for a system"""
        try:
            if system_id in self.monitored_systems:
                del self.monitored_systems[system_id]
                self.logger.info(f"Disabled monitoring for system {system_id}")
                return True
            return False
        except Exception as e:
            self.logger.error(f"Failed to disable monitoring for {system_id}: {e}")
            return False
    
    def start_monitoring(self) -> None:
        """Start system monitoring"""
        if self.monitoring_active:
            self.logger.warning("Monitoring is already active")
            return
        
        self.monitoring_active = True
        self.monitor_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
        self.monitor_thread.start()
        
        self.logger.info("Started system monitoring")
    
    def stop_monitoring(self) -> None:
        """Stop system monitoring"""
        self.monitoring_active = False
        
        if self.monitor_thread:
            self.monitor_thread.join(timeout=5)
        
        self.logger.info("Stopped system monitoring")
    
    def _monitoring_loop(self) -> None:
        """Main monitoring loop"""
        while self.monitoring_active:
            try:
                # Perform health checks on all monitored systems
                self._perform_health_checks()
                
                # Run performance monitoring
                self._collect_performance_data()
                
                # Check for alerts
                self._check_alerts()
                
                # Sleep until next monitoring cycle
                time.sleep(self.config['monitoring']['interval'])
                
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                time.sleep(60)  # Wait 1 minute before retrying
    
    def _perform_health_checks(self) -> None:
        """Perform health checks on all monitored systems"""
        max_workers = self.config['monitoring']['max_workers']
        
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            futures = {}
            
            for system_id in self.monitored_systems.keys():
                future = executor.submit(self._check_system_health, system_id)
                futures[system_id] = future
            
            # Collect results
            for system_id, future in futures.items():
                try:
                    health_check = future.result(timeout=self.config['monitoring']['timeout'])
                    if health_check:
                        self.health_checks[system_id] = health_check
                        self._save_health_check(system_id, health_check)
                except Exception as e:
                    self.logger.error(f"Health check failed for {system_id}: {e}")
    
    def _check_system_health(self, system_id: str) -> Optional[HealthCheck]:
        """Check health of a specific system"""
        try:
            # Get system information (this would normally come from registry)
            # For now, create a mock system info
            system_info = self._get_system_info(system_id)
            if not system_info:
                return None
            
            timestamp = datetime.now()
            issues = []
            recommendations = []
            checks = {}
            performance_metrics = {}
            
            # Basic connectivity check
            is_reachable = ping_host(system_info.ip_address)
            checks['network_connectivity'] = is_reachable
            
            if not is_reachable:
                issues.append(f"System {system_info.hostname} is not reachable")
                recommendations.append("Check network connectivity and firewall settings")
            else:
                # Perform additional checks if reachable
                checks.update(self._check_system_services(system_info))
                checks.update(self._check_system_performance(system_info))
                checks.update(self._check_system_resources(system_info))
            
            # Determine overall status
            if checks.get('network_connectivity', False):
                if all(checks.values()):
                    overall_status = SystemStatus.ONLINE
                elif any(checks.values()):
                    overall_status = SystemStatus.DEGRADED
                else:
                    overall_status = SystemStatus.OFFLINE
            else:
                overall_status = SystemStatus.OFFLINE
            
            # Collect performance metrics
            performance_metrics = self._get_performance_metrics(system_info)
            
            health_check = HealthCheck(
                system_id=system_id,
                timestamp=timestamp,
                overall_status=overall_status,
                checks=checks,
                performance_metrics=performance_metrics,
                issues=issues,
                recommendations=recommendations
            )
            
            # Update monitoring status
            monitor_config = self.monitored_systems[system_id]
            monitor_config['last_check'] = timestamp
            
            if overall_status in [SystemStatus.OFFLINE, SystemStatus.DEGRADED]:
                monitor_config['consecutive_failures'] += 1
            else:
                monitor_config['consecutive_failures'] = 0
                monitor_config['alert_sent'] = False  # Reset alert flag
            
            return health_check
            
        except Exception as e:
            self.logger.error(f"Failed to check health for system {system_id}: {e}")
            return None
    
    def _get_system_info(self, system_id: str) -> Optional[SystemInfo]:
        """Get system information (this would integrate with the main registry)"""
        # This is a placeholder - in real implementation, this would query the system registry
        return SystemInfo(
            system_id=system_id,
            hostname=f"system-{system_id[:8]}",
            ip_address=f"192.168.1.{int(system_id[:2], 16) % 254 + 1}",
            mac_address=f"{':'.join([f'{int(system_id[i:i+2], 16):02x}' for i in range(0, 12, 2)])}",
            system_type='desktop',
            cpu_model='Unknown',
            memory_gb=8,
            storage_gb=500,
            network_interface='eth0',
            site_id='default',
            location='Unknown'
        )
    
    def _check_system_services(self, system_info: SystemInfo) -> Dict[str, bool]:
        """Check status of system services"""
        services = self.config['health_checks']['services_to_check']
        service_checks = {}
        
        for service in services:
            if service == 'ssh':
                service_checks[f'service_{service}'] = is_port_open(system_info.ip_address, 22)
            elif service == 'http':
                service_checks[f'service_{service}'] = is_port_open(system_info.ip_address, 80)
            elif service == 'https':
                service_checks[f'service_{service}'] = is_port_open(system_info.ip_address, 443)
            elif service == 'dns':
                service_checks[f'service_{service}'] = self._check_dns_resolution(system_info.ip_address)
            else:
                # Generic port check
                port = int(service.split('_')[-1]) if '_' in service else 80
                service_checks[f'service_{service}'] = is_port_open(system_info.ip_address, port)
        
        return service_checks
    
    def _check_system_performance(self, system_info: SystemInfo) -> Dict[str, bool]:
        """Check system performance metrics"""
        performance_checks = {}
        
        # Get remote system performance (simplified - would use SSH/remote monitoring)
        # For now, simulate performance checks
        performance_checks['cpu_usage_ok'] = True  # Would check against threshold
        performance_checks['memory_usage_ok'] = True  # Would check against threshold
        performance_checks['disk_usage_ok'] = True  # Would check against threshold
        
        return performance_checks
    
    def _check_system_resources(self, system_info: SystemInfo) -> Dict[str, bool]:
        """Check system resource availability"""
        resource_checks = {
            'disk_space_available': True,  # Would check available disk space
            'memory_available': True,      # Would check available memory
            'processes_running': True      # Would check process count
        }
        
        return resource_checks
    
    def _check_dns_resolution(self, ip_address: str) -> bool:
        """Check DNS resolution capability"""
        try:
            # Simple DNS check - try to resolve a common domain
            result = subprocess.run(
                ['nslookup', 'google.com', ip_address],
                capture_output=True,
                text=True,
                timeout=10
            )
            return result.returncode == 0
        except Exception:
            return False
    
    def _get_performance_metrics(self, system_info: SystemInfo) -> Dict[str, float]:
        """Get performance metrics from system"""
        # This would normally collect metrics via SSH or remote monitoring
        # For now, return mock metrics
        return {
            'cpu_usage': 45.0,
            'memory_usage': 62.0,
            'disk_usage': 34.0,
            'network_latency': 2.5,
            'load_average': 0.8,
            'uptime_hours': 720.5
        }
    
    def _collect_performance_data(self) -> None:
        """Collect performance data from monitored systems"""
        timestamp = datetime.now()
        
        for system_id in self.monitored_systems.keys():
            try:
                system_info = self._get_system_info(system_id)
                if system_info and self.health_checks.get(system_id):
                    metrics = self.health_checks[system_id].performance_metrics
                    metrics['timestamp'] = timestamp
                    
                    # Save performance data
                    self._save_performance_data(system_id, metrics)
            except Exception as e:
                self.logger.error(f"Failed to collect performance data for {system_id}: {e}")
    
    def _check_alerts(self) -> None:
        """Check for alerts and send notifications"""
        for system_id, health_check in self.health_checks.items():
            monitor_config = self.monitored_systems[system_id]
            
            # Check if alert should be sent
            if self._should_send_alert(system_id, health_check, monitor_config):
                self._send_alert(system_id, health_check)
                monitor_config['alert_sent'] = True
    
    def _should_send_alert(self, system_id: str, health_check: HealthCheck, 
                          monitor_config: Dict[str, Any]) -> bool:
        """Determine if alert should be sent"""
        # Don't send alerts if already sent recently
        if monitor_config.get('alert_sent', False):
            return False
        
        # Send alerts for critical issues
        if health_check.overall_status == SystemStatus.OFFLINE:
            return True
        
        # Send alerts for degraded performance
        if health_check.overall_status == SystemStatus.DEGRADED:
            # Check if critical thresholds are exceeded
            for check_name, check_result in health_check.checks.items():
                if not check_result and 'critical' in check_name:
                    return True
        
        # Send alerts for consecutive failures
        if monitor_config.get('consecutive_failures', 0) >= 3:
            return True
        
        return False
    
    def _send_alert(self, system_id: str, health_check: HealthCheck) -> None:
        """Send alert notification"""
        try:
            alert_data = {
                'system_id': system_id,
                'timestamp': health_check.timestamp.isoformat(),
                'status': health_check.overall_status.value,
                'issues': health_check.issues,
                'recommendations': health_check.recommendations
            }
            
            # Send via all configured alert handlers
            for handler_name, handler_func in self.alert_handlers.items():
                try:
                    handler_func(alert_data)
                except Exception as e:
                    self.logger.error(f"Alert handler {handler_name} failed: {e}")
            
            self.logger.info(f"Sent alert for system {system_id}")
            
        except Exception as e:
            self.logger.error(f"Failed to send alert for system {system_id}: {e}")
    
    def _send_email_alert(self, alert_data: Dict[str, Any]) -> None:
        """Send email alert"""
        email_config = self.config['alerts']['email_notifications']
        
        if not email_config['enabled'] or not email_config['recipients']:
            return
        
        try:
            # Create email message
            msg = MimeMultipart()
            msg['From'] = email_config['username']
            msg['To'] = ', '.join(email_config['recipients'])
            msg['Subject'] = f"MultiOS Alert: System {alert_data['system_id']} - {alert_data['status']}"
            
            # Create email body
            body = f"""MultiOS System Alert

System ID: {alert_data['system_id']}
Status: {alert_data['status']}
Timestamp: {alert_data['timestamp']}

Issues:
{chr(10).join(f'- {issue}' for issue in alert_data['issues'])}

Recommendations:
{chr(10).join(f'- {rec}' for rec in alert_data['recommendations'])}

This is an automated alert from the MultiOS Enterprise Deployment System.
"""
            
            msg.attach(MimeText(body, 'plain'))
            
            # Send email
            server = smtplib.SMTP(email_config['smtp_server'], email_config['smtp_port'])
            server.starttls()
            server.login(email_config['username'], email_config['password'])
            server.send_message(msg)
            server.quit()
            
        except Exception as e:
            self.logger.error(f"Failed to send email alert: {e}")
    
    def _send_slack_alert(self, alert_data: Dict[str, Any]) -> None:
        """Send Slack alert"""
        slack_config = self.config['alerts']['slack_notifications']
        
        if not slack_config['enabled'] or not slack_config['webhook_url']:
            return
        
        try:
            import requests
            
            # Create Slack message
            slack_message = {
                'channel': slack_config['channel'],
                'username': 'MultiOS Monitor',
                'icon_emoji': ':warning:',
                'attachments': [{
                    'color': 'danger' if alert_data['status'] == 'offline' else 'warning',
                    'title': f'System Alert: {alert_data["system_id"]}',
                    'fields': [
                        {'title': 'Status', 'value': alert_data['status'], 'short': True},
                        {'title': 'Timestamp', 'value': alert_data['timestamp'], 'short': True},
                        {'title': 'Issues', 'value': '\n'.join(alert_data['issues']), 'short': False}
                    ]
                }]
            }
            
            response = requests.post(slack_config['webhook_url'], json=slack_message)
            response.raise_for_status()
            
        except Exception as e:
            self.logger.error(f"Failed to send Slack alert: {e}")
    
    def _log_alert(self, alert_data: Dict[str, Any]) -> None:
        """Log alert to file"""
        try:
            log_file = Path("/var/log/multios-enterprise/monitoring/alerts.log")
            
            log_entry = {
                'timestamp': datetime.now().isoformat(),
                'system_id': alert_data['system_id'],
                'status': alert_data['status'],
                'issues': alert_data['issues'],
                'recommendations': alert_data['recommendations']
            }
            
            with open(log_file, 'a') as f:
                f.write(json.dumps(log_entry) + '\n')
                
        except Exception as e:
            self.logger.error(f"Failed to log alert: {e}")
    
    def _save_health_check(self, system_id: str, health_check: HealthCheck) -> None:
        """Save health check result"""
        try:
            health_file = Path("/var/lib/multios-enterprise/health_checks") / f"{system_id}.json"
            
            health_data = {
                'system_id': health_check.system_id,
                'timestamp': health_check.timestamp.isoformat(),
                'overall_status': health_check.overall_status.value,
                'checks': health_check.checks,
                'performance_metrics': health_check.performance_metrics,
                'issues': health_check.issues,
                'recommendations': health_check.recommendations
            }
            
            with open(health_file, 'w') as f:
                json.dump(health_data, f, indent=2)
                
        except Exception as e:
            self.logger.error(f"Failed to save health check for {system_id}: {e}")
    
    def _save_performance_data(self, system_id: str, metrics: Dict[str, Any]) -> None:
        """Save performance data"""
        try:
            data_file = Path("/var/lib/multios-enterprise/performance_data") / f"{system_id}.jsonl"
            
            with open(data_file, 'a') as f:
                f.write(json.dumps(metrics) + '\n')
                
        except Exception as e:
            self.logger.error(f"Failed to save performance data for {system_id}: {e}")
    
    def get_health_check(self, system_id: str) -> Optional[HealthCheck]:
        """Get latest health check for a system"""
        return self.health_checks.get(system_id)
    
    def get_all_health_checks(self) -> Dict[str, HealthCheck]:
        """Get health checks for all monitored systems"""
        return self.health_checks.copy()
    
    def get_monitoring_status(self) -> Dict[str, Any]:
        """Get current monitoring status"""
        return {
            'active': self.monitoring_active,
            'monitored_systems': len(self.monitored_systems),
            'healthy_systems': sum(1 for hc in self.health_checks.values() 
                                 if hc.overall_status == SystemStatus.ONLINE),
            'degraded_systems': sum(1 for hc in self.health_checks.values() 
                                  if hc.overall_status == SystemStatus.DEGRADED),
            'offline_systems': sum(1 for hc in self.health_checks.values() 
                                 if hc.overall_status == SystemStatus.OFFLINE)
        }
    
    def trigger_manual_health_check(self, system_id: str) -> Optional[HealthCheck]:
        """Trigger a manual health check for a system"""
        if system_id not in self.monitored_systems:
            return None
        
        health_check = self._check_system_health(system_id)
        if health_check:
            self.health_checks[system_id] = health_check
            self._save_health_check(system_id, health_check)
        
        return health_check
    
    def update_system_status(self, system_id: str, status: SystemStatus) -> bool:
        """Update system status directly (for deployment processes)"""
        try:
            timestamp = datetime.now()
            
            # Create a basic health check with provided status
            health_check = HealthCheck(
                system_id=system_id,
                timestamp=timestamp,
                overall_status=status,
                checks={'manual_update': True},
                performance_metrics={},
                issues=[],
                recommendations=[]
            )
            
            self.health_checks[system_id] = health_check
            self._save_health_check(system_id, health_check)
            
            return True
        except Exception as e:
            self.logger.error(f"Failed to update system status for {system_id}: {e}")
            return False
