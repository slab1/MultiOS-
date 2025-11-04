# Prometheus integration for MultiOS Monitoring
# Provides Prometheus-compatible metrics endpoints and configuration

import time
import psutil
from prometheus_client import (
    Counter, Gauge, Histogram, Summary, 
    CollectorRegistry, generate_latest, 
    CONTENT_TYPE_LATEST, start_http_server,
    exposition
)
from prometheus_client.core import REGISTRY, CollectorRegistry
import logging

class MultiOSPrometheusCollector:
    """Prometheus collector for MultiOS metrics"""
    
    def __init__(self):
        # System metrics
        self.cpu_usage_gauge = Gauge(
            'multios_cpu_usage_percent', 
            'CPU usage percentage', 
            ['instance', 'cpu']
        )
        self.memory_usage_gauge = Gauge(
            'multios_memory_usage_bytes',
            'Memory usage in bytes',
            ['instance', 'type']
        )
        self.memory_usage_percent_gauge = Gauge(
            'multios_memory_usage_percent',
            'Memory usage percentage',
            ['instance']
        )
        self.disk_usage_gauge = Gauge(
            'multios_disk_usage_bytes',
            'Disk usage in bytes',
            ['instance', 'mountpoint']
        )
        self.disk_usage_percent_gauge = Gauge(
            'multios_disk_usage_percent',
            'Disk usage percentage',
            ['instance', 'mountpoint']
        )
        self.network_io_gauge = Gauge(
            'multios_network_io_bytes_total',
            'Network I/O bytes total',
            ['instance', 'interface', 'direction']
        )
        self.network_packets_gauge = Gauge(
            'multios_network_packets_total',
            'Network packets total',
            ['instance', 'interface', 'direction']
        )
        
        # Process metrics
        self.process_count_gauge = Gauge(
            'multios_process_count',
            'Number of running processes',
            ['instance']
        )
        self.process_cpu_gauge = Gauge(
            'multios_process_cpu_percent',
            'Process CPU usage percentage',
            ['instance', 'pid', 'name']
        )
        self.process_memory_gauge = Gauge(
            'multios_process_memory_bytes',
            'Process memory usage in bytes',
            ['instance', 'pid', 'name']
        )
        
        # System info
        self.boot_time_gauge = Gauge(
            'multios_boot_time_seconds',
            'System boot time in seconds',
            ['instance']
        )
        self.load_average_gauge = Gauge(
            'multios_load_average',
            'System load average',
            ['instance', 'period']
        )
        
        # Educational metrics
        self.lab_sessions_gauge = Gauge(
            'multios_lab_sessions_total',
            'Total lab sessions',
            ['instance', 'status']
        )
        self.student_activity_counter = Counter(
            'multios_student_activity_total',
            'Total student activities',
            ['instance', 'activity_type']
        )
        self.course_resource_gauge = Gauge(
            'multios_course_resource_usage_percent',
            'Course resource usage percentage',
            ['instance', 'course_id', 'resource_type']
        )
        self.engagement_score_gauge = Gauge(
            'multios_student_engagement_score',
            'Student engagement score',
            ['instance', 'student_id']
        )
        
        # Security metrics
        self.security_events_counter = Counter(
            'multios_security_events_total',
            'Total security events',
            ['instance', 'event_type', 'severity']
        )
        self.failed_login_gauge = Gauge(
            'multios_failed_login_attempts_total',
            'Total failed login attempts',
            ['instance', 'source']
        )
        
        # System performance metrics
        self.request_duration_histogram = Histogram(
            'multios_request_duration_seconds',
            'Request duration in seconds',
            ['instance', 'endpoint']
        )
        self.error_counter = Counter(
            'multios_errors_total',
            'Total errors',
            ['instance', 'error_type']
        )
        
        self.instance = 'multios'
    
    def collect(self):
        """Collect all metrics"""
        try:
            self._collect_system_metrics()
            self._collect_process_metrics()
            self._collect_educational_metrics()
            self._collect_security_metrics()
        except Exception as e:
            logging.error(f"Error collecting metrics: {e}")
    
    def _collect_system_metrics(self):
        """Collect system metrics"""
        # CPU metrics
        cpu_percent = psutil.cpu_percent(interval=1)
        self.cpu_usage_gauge.labels(
            instance=self.instance, 
            cpu='total'
        ).set(cpu_percent)
        
        # Per-core CPU usage
        per_cpu = psutil.cpu_percent(interval=1, percpu=True)
        for i, core_percent in enumerate(per_cpu):
            self.cpu_usage_gauge.labels(
                instance=self.instance,
                cpu=f'core_{i}'
            ).set(core_percent)
        
        # Memory metrics
        memory = psutil.virtual_memory()
        self.memory_usage_percent_gauge.labels(
            instance=self.instance
        ).set(memory.percent)
        
        self.memory_usage_gauge.labels(
            instance=self.instance,
            type='total'
        ).set(memory.total)
        
        self.memory_usage_gauge.labels(
            instance=self.instance,
            type='used'
        ).set(memory.used)
        
        self.memory_usage_gauge.labels(
            instance=self.instance,
            type='available'
        ).set(memory.available)
        
        self.memory_usage_gauge.labels(
            instance=self.instance,
            type='free'
        ).set(memory.free)
        
        # Disk metrics
        disk_usage = psutil.disk_usage('/')
        self.disk_usage_percent_gauge.labels(
            instance=self.instance,
            mountpoint='/'
        ).set((disk_usage.used / disk_usage.total) * 100)
        
        self.disk_usage_gauge.labels(
            instance=self.instance,
            mountpoint='/'
        ).set(disk_usage.total)
        
        self.disk_usage_gauge.labels(
            instance=self.instance,
            mountpoint='/used'
        ).set(disk_usage.used)
        
        self.disk_usage_gauge.labels(
            instance=self.instance,
            mountpoint='/free'
        ).set(disk_usage.free)
        
        # Network metrics
        network_io = psutil.net_io_counters()
        self.network_io_gauge.labels(
            instance=self.instance,
            interface='total',
            direction='sent'
        ).set(network_io.bytes_sent)
        
        self.network_io_gauge.labels(
            instance=self.instance,
            interface='total',
            direction='received'
        ).set(network_io.bytes_recv)
        
        self.network_packets_gauge.labels(
            instance=self.instance,
            interface='total',
            direction='sent'
        ).set(network_io.packets_sent)
        
        self.network_packets_gauge.labels(
            instance=self.instance,
            interface='total',
            direction='received'
        ).set(network_io.packets_recv)
        
        # Per-interface network metrics
        per_interface = psutil.net_io_counters(pernic=True)
        for interface, stats in per_interface.items():
            self.network_io_gauge.labels(
                instance=self.instance,
                interface=interface,
                direction='sent'
            ).set(stats.bytes_sent)
            
            self.network_io_gauge.labels(
                instance=self.instance,
                interface=interface,
                direction='received'
            ).set(stats.bytes_recv)
        
        # System info
        boot_time = psutil.boot_time()
        self.boot_time_gauge.labels(
            instance=self.instance
        ).set(boot_time)
        
        # Load average
        load_avg = os.getloadavg() if hasattr(os, 'getloadavg') else (0, 0, 0)
        periods = ['1m', '5m', '15m']
        for period, load in zip(periods, load_avg):
            self.load_average_gauge.labels(
                instance=self.instance,
                period=period
            ).set(load)
    
    def _collect_process_metrics(self):
        """Collect process metrics"""
        # Process count
        process_count = len(psutil.pids())
        self.process_count_gauge.labels(
            instance=self.instance
        ).set(process_count)
        
        # Top processes by CPU
        processes = []
        for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent']):
            try:
                pinfo = proc.info
                if pinfo['cpu_percent'] and pinfo['cpu_percent'] > 1.0:  # Only active processes
                    processes.append(pinfo)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        
        # Sort by CPU usage
        processes.sort(key=lambda x: x['cpu_percent'], reverse=True)
        
        # Report top 20 processes
        for proc in processes[:20]:
            self.process_cpu_gauge.labels(
                instance=self.instance,
                pid=str(proc['pid']),
                name=proc['name'] or 'unknown'
            ).set(proc['cpu_percent'])
            
            self.process_memory_gauge.labels(
                instance=self.instance,
                pid=str(proc['pid']),
                name=proc['name'] or 'unknown'
            ).set(proc['memory_percent'])
    
    def _collect_educational_metrics(self):
        """Collect educational metrics (simulated)"""
        import random
        
        # Lab sessions
        active_sessions = random.randint(20, 45)
        total_sessions_today = random.randint(150, 300)
        completed_sessions = int(total_sessions_today * 0.85)
        
        self.lab_sessions_gauge.labels(
            instance=self.instance,
            status='active'
        ).set(active_sessions)
        
        self.lab_sessions_gauge.labels(
            instance=self.instance,
            status='completed_today'
        ).set(completed_sessions)
        
        self.lab_sessions_gauge.labels(
            instance=self.instance,
            status='total_today'
        ).set(total_sessions_today)
        
        # Student activities
        activity_types = ['login', 'logout', 'file_access', 'application_use', 'network_activity']
        for activity_type in activity_types:
            count = random.randint(50, 200)
            self.student_activity_counter.labels(
                instance=self.instance,
                activity_type=activity_type
            )._value._value += count
        
        # Course resource usage
        courses = ['CS101', 'CS201', 'CS301']
        resource_types = ['cpu', 'memory', 'storage', 'network']
        
        for course in courses:
            for resource_type in resource_types:
                usage = random.uniform(20, 80)
                self.course_resource_gauge.labels(
                    instance=self.instance,
                    course_id=course,
                    resource_type=resource_type
                ).set(usage)
        
        # Student engagement (top 10 students)
        for i in range(10):
            engagement = random.uniform(60, 95)
            self.engagement_score_gauge.labels(
                instance=self.instance,
                student_id=f'student_{i+1}'
            ).set(engagement)
    
    def _collect_security_metrics(self):
        """Collect security metrics (simulated)"""
        import random
        
        # Security events
        event_types = ['login_failure', 'privilege_escalation', 'file_access', 'network_intrusion']
        severities = ['critical', 'warning', 'info']
        
        for event_type in event_types:
            for severity in severities:
                count = random.randint(0, 10)
                if count > 0:
                    self.security_events_counter.labels(
                        instance=self.instance,
                        event_type=event_type,
                        severity=severity
                    )._value._value += count
        
        # Failed login attempts
        failed_logins = random.randint(5, 25)
        self.failed_login_gauge.labels(
            instance=self.instance,
            source='system'
        ).set(failed_logins)

class PrometheusIntegration:
    """Prometheus integration handler"""
    
    def __init__(self, config: dict):
        self.config = config
        self.port = config.get('prometheus_port', 9090)
        self.registry = CollectorRegistry()
        self.collector = MultiOSPrometheusCollector()
        self.server = None
        
    def start_server(self):
        """Start Prometheus metrics server"""
        try:
            # Register collector
            self.registry.register(self.collector)
            
            # Start HTTP server
            self.server = start_http_server(
                self.port,
                registry=self.registry
            )
            
            logging.info(f"Prometheus metrics server started on port {self.port}")
            return True
            
        except Exception as e:
            logging.error(f"Failed to start Prometheus server: {e}")
            return False
    
    def stop_server(self):
        """Stop Prometheus metrics server"""
        if self.server:
            self.server.shutdown()
            logging.info("Prometheus metrics server stopped")
    
    def get_metrics_endpoint(self):
        """Get metrics endpoint URL"""
        return f"http://localhost:{self.port}/metrics"
    
    def get_config(self):
        """Get Prometheus scrape configuration"""
        scrape_configs = []
        
        # MultiOS monitoring job
        scrape_configs.append({
            'job_name': 'multios-monitoring',
            'static_configs': [{
                'targets': [f'localhost:{self.port}']
            }],
            'scrape_interval': self.config.get('scrape_interval', '15s'),
            'metrics_path': '/metrics',
            'scrape_timeout': '10s'
        })
        
        return {
            'global': {
                'scrape_interval': self.config.get('global_scrape_interval', '15s'),
                'scrape_timeout': '10s',
                'evaluation_interval': '15s'
            },
            'scrape_configs': scrape_configs
        }
    
    def create_scrape_config_file(self, output_path: str):
        """Create Prometheus scrape configuration file"""
        config = self.get_config()
        
        try:
            import yaml
            with open(output_path, 'w') as f:
                yaml.dump(config, f, default_flow_style=False)
            logging.info(f"Prometheus config written to {output_path}")
            return True
        except Exception as e:
            logging.error(f"Failed to write Prometheus config: {e}")
            return False

class GrafanaIntegration:
    """Grafana integration for dashboard provisioning"""
    
    def __init__(self, config: dict):
        self.config = config
        self.grafana_url = config.get('grafana_url', 'http://localhost:3000')
        self.api_key = config.get('grafana_api_key')
        self.dashboards_dir = config.get('dashboards_dir', 'grafana_dashboards')
        
    def create_dashboard_configs(self):
        """Create Grafana dashboard configurations"""
        dashboards = [
            {
                'name': 'MultiOS System Overview',
                'uid': 'multios-system-overview',
                'panels': [
                    {
                        'title': 'CPU Usage',
                        'type': 'stat',
                        'targets': [
                            {
                                'expr': 'multios_cpu_usage_percent{instance="multios"}',
                                'legendFormat': 'CPU Usage'
                            }
                        ]
                    },
                    {
                        'title': 'Memory Usage',
                        'type': 'stat',
                        'targets': [
                            {
                                'expr': 'multios_memory_usage_percent{instance="multios"}',
                                'legendFormat': 'Memory Usage'
                            }
                        ]
                    },
                    {
                        'title': 'System Performance',
                        'type': 'graph',
                        'targets': [
                            {
                                'expr': 'multios_cpu_usage_percent{instance="multios"}',
                                'legendFormat': 'CPU %'
                            },
                            {
                                'expr': 'multios_memory_usage_percent{instance="multios"}',
                                'legendFormat': 'Memory %'
                            }
                        ]
                    },
                    {
                        'title': 'Network I/O',
                        'type': 'graph',
                        'targets': [
                            {
                                'expr': 'multios_network_io_bytes_total{instance="multios",direction="received"}',
                                'legendFormat': 'Received'
                            },
                            {
                                'expr': 'multios_network_io_bytes_total{instance="multios",direction="sent"}',
                                'legendFormat': 'Sent'
                            }
                        ]
                    }
                ]
            },
            {
                'name': 'MultiOS Educational Analytics',
                'uid': 'multios-educational-analytics',
                'panels': [
                    {
                        'title': 'Active Lab Sessions',
                        'type': 'stat',
                        'targets': [
                            {
                                'expr': 'multios_lab_sessions_total{instance="multios",status="active"}',
                                'legendFormat': 'Active Sessions'
                            }
                        ]
                    },
                    {
                        'title': 'Lab Sessions Trend',
                        'type': 'graph',
                        'targets': [
                            {
                                'expr': 'multios_lab_sessions_total{instance="multios"}',
                                'legendFormat': '{{status}}'
                            }
                        ]
                    },
                    {
                        'title': 'Student Engagement Score',
                        'type': 'stat',
                        'targets': [
                            {
                                'expr': 'avg(multios_student_engagement_score{instance="multios"})',
                                'legendFormat': 'Average Engagement'
                            }
                        ]
                    },
                    {
                        'title': 'Course Resource Usage',
                        'type': 'graph',
                        'targets': [
                            {
                                'expr': 'multios_course_resource_usage_percent{instance="multios",resource_type="cpu"}',
                                'legendFormat': '{{course_id}} CPU'
                            },
                            {
                                'expr': 'multios_course_resource_usage_percent{instance="multios",resource_type="memory"}',
                                'legendFormat': '{{course_id}} Memory'
                            }
                        ]
                    }
                ]
            },
            {
                'name': 'MultiOS Security Monitoring',
                'uid': 'multios-security-monitoring',
                'panels': [
                    {
                        'title': 'Security Events',
                        'type': 'table',
                        'targets': [
                            {
                                'expr': 'multios_security_events_total{instance="multios"}',
                                'legendFormat': '{{event_type}}'
                            }
                        ]
                    },
                    {
                        'title': 'Failed Login Attempts',
                        'type': 'stat',
                        'targets': [
                            {
                                'expr': 'multios_failed_login_attempts_total{instance="multios"}',
                                'legendFormat': 'Failed Logins'
                            }
                        ]
                    },
                    {
                        'title': 'Security Events Trend',
                        'type': 'graph',
                        'targets': [
                            {
                                'expr': 'multios_security_events_total{instance="multios"}',
                                'legendFormat': '{{event_type}}'
                            }
                        ]
                    }
                ]
            }
        ]
        
        return dashboards
    
    def provision_dashboards(self):
        """Provision Grafana dashboards"""
        if not self.api_key:
            logging.warning("No Grafana API key provided, skipping dashboard provisioning")
            return False
        
        dashboards = self.create_dashboard_configs()
        success_count = 0
        
        for dashboard_config in dashboards:
            try:
                success = self._create_dashboard(dashboard_config)
                if success:
                    success_count += 1
            except Exception as e:
                logging.error(f"Failed to create dashboard {dashboard_config['name']}: {e}")
        
        logging.info(f"Successfully provisioned {success_count}/{len(dashboards)} dashboards")
        return success_count > 0
    
    def _create_dashboard(self, dashboard_config: dict):
        """Create a single Grafana dashboard"""
        headers = {
            'Authorization': f'Bearer {self.api_key}',
            'Content-Type': 'application/json'
        }
        
        # Create dashboard JSON
        dashboard_json = {
            'dashboard': {
                'id': None,
                'uid': dashboard_config['uid'],
                'title': dashboard_config['name'],
                'tags': ['multios', 'monitoring'],
                'timezone': 'browser',
                'refresh': '30s',
                'time': {
                    'from': 'now-1h',
                    'to': 'now'
                },
                'panels': dashboard_config['panels'],
                'annotations': {
                    'list': []
                },
                'editable': True,
                'gnetId': None,
                'graphTooltip': 0,
                'links': [],
                'panels': dashboard_config['panels'],
                'schemaVersion': 26,
                'version': 1
            },
            'overwrite': True,
            'inputs': [],
            'requiresAuth': False
        }
        
        # Create dashboard via API
        response = requests.post(
            f"{self.grafana_url}/api/dashboards/db",
            headers=headers,
            json=dashboard_json
        )
        
        if response.status_code in [200, 201]:
            logging.info(f"Created dashboard: {dashboard_config['name']}")
            return True
        else:
            logging.error(f"Failed to create dashboard: {response.status_code} - {response.text}")
            return False

# Export for external use
__all__ = ['MultiOSPrometheusCollector', 'PrometheusIntegration', 'GrafanaIntegration']

if __name__ == '__main__':
    import os
    import requests
    
    # Example usage
    logging.basicConfig(level=logging.INFO)
    
    # Prometheus configuration
    prometheus_config = {
        'prometheus_port': 9090,
        'scrape_interval': '15s'
    }
    
    # Start Prometheus integration
    prometheus = PrometheusIntegration(prometheus_config)
    prometheus.start_server()
    
    # Create Grafana dashboards (if API key is available)
    grafana_config = {
        'grafana_url': 'http://localhost:3000',
        'grafana_api_key': os.getenv('GRAFANA_API_KEY')
    }
    
    grafana = GrafanaIntegration(grafana_config)
    grafana.provision_dashboards()
    
    # Keep server running
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        prometheus.stop_server()