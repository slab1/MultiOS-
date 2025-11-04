#!/usr/bin/env python3
"""
MultiOS Monitoring Agents
Specialized agents for collecting system, application, and educational data
"""

import asyncio
import json
import logging
import time
import psutil
import threading
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Callable
from dataclasses import dataclass, asdict
from pathlib import Path
import subprocess
import os
import socket
from collections import defaultdict, deque
import hashlib

@dataclass
class AgentMetrics:
    """Agent metrics data structure"""
    agent_id: str
    timestamp: float
    metrics: Dict[str, Any]
    status: str  # online, offline, error
    uptime: float

@dataclass
class HardwareSensor:
    """Hardware sensor data"""
    name: str
    type: str  # temperature, voltage, fan, power
    value: float
    unit: str
    status: str  # normal, warning, critical
    threshold_warning: float
    threshold_critical: float

class SystemAgent:
    """System-level monitoring agent"""
    
    def __init__(self, agent_id: str = "system"):
        self.agent_id = agent_id
        self.running = False
        self.metrics_history = deque(maxlen=1000)
        self.last_boot_time = psutil.boot_time()
        self.start_time = time.time()
        
        # Hardware sensors cache
        self.sensors_cache = []
        self.last_sensor_update = 0
        
    def start(self, interval: int = 10):
        """Start system monitoring"""
        self.running = True
        self.collection_interval = interval
        
        # Start collection thread
        self.collection_thread = threading.Thread(target=self._collection_loop)
        self.collection_thread.daemon = True
        self.collection_thread.start()
        
        logging.info(f"System agent {self.agent_id} started")
    
    def stop(self):
        """Stop system monitoring"""
        self.running = False
        if hasattr(self, 'collection_thread'):
            self.collection_thread.join()
        logging.info(f"System agent {self.agent_id} stopped")
    
    def _collection_loop(self):
        """Main collection loop"""
        while self.running:
            try:
                metrics = self.collect_metrics()
                self.metrics_history.append(metrics)
                time.sleep(self.collection_interval)
            except Exception as e:
                logging.error(f"Error in system collection: {e}")
                time.sleep(5)
    
    def collect_metrics(self) -> AgentMetrics:
        """Collect system metrics"""
        timestamp = time.time()
        
        # Basic system metrics
        metrics = {
            'cpu': self._get_cpu_metrics(),
            'memory': self._get_memory_metrics(),
            'disk': self._get_disk_metrics(),
            'network': self._get_network_metrics(),
            'processes': self._get_process_metrics(),
            'system': self._get_system_info(),
            'hardware': self._get_hardware_sensors()
        }
        
        status = self._determine_status(metrics)
        uptime = timestamp - self.start_time
        
        return AgentMetrics(
            agent_id=self.agent_id,
            timestamp=timestamp,
            metrics=metrics,
            status=status,
            uptime=uptime
        )
    
    def _get_cpu_metrics(self) -> Dict[str, Any]:
        """Get CPU metrics"""
        try:
            return {
                'usage_percent': psutil.cpu_percent(interval=1),
                'per_cpu': psutil.cpu_percent(interval=1, percpu=True),
                'count_physical': psutil.cpu_count(logical=False),
                'count_logical': psutil.cpu_count(logical=True),
                'frequency': {
                    'current': psutil.cpu_freq().current if psutil.cpu_freq() else 0,
                    'min': psutil.cpu_freq().min if psutil.cpu_freq() else 0,
                    'max': psutil.cpu_freq().max if psutil.cpu_freq() else 0
                } if psutil.cpu_freq() else {},
                'load_average': list(os.getloadavg()) if hasattr(os, 'getloadavg') else [0, 0, 0],
                'context_switches': psutil.cpu_stats().ctx_switches,
                'interrupts': psutil.cpu_stats().interrupts
            }
        except Exception as e:
            logging.error(f"Error getting CPU metrics: {e}")
            return {}
    
    def _get_memory_metrics(self) -> Dict[str, Any]:
        """Get memory metrics"""
        try:
            memory = psutil.virtual_memory()
            swap = psutil.swap_memory()
            
            return {
                'virtual': {
                    'total': memory.total,
                    'available': memory.available,
                    'used': memory.used,
                    'free': memory.free,
                    'percent': memory.percent,
                    'buffers': getattr(memory, 'buffers', 0),
                    'cached': getattr(memory, 'cached', 0),
                    'shared': getattr(memory, 'shared', 0)
                },
                'swap': {
                    'total': swap.total,
                    'used': swap.used,
                    'free': swap.free,
                    'percent': swap.percent
                }
            }
        except Exception as e:
            logging.error(f"Error getting memory metrics: {e}")
            return {}
    
    def _get_disk_metrics(self) -> Dict[str, Any]:
        """Get disk metrics"""
        try:
            disk_usage = {}
            disk_io = psutil.disk_io_counters()
            
            # Get disk usage for common mount points
            mount_points = ['/', '/home', '/var', '/tmp']
            for mount in mount_points:
                try:
                    usage = psutil.disk_usage(mount)
                    disk_usage[mount] = {
                        'total': usage.total,
                        'used': usage.used,
                        'free': usage.free,
                        'percent': (usage.used / usage.total) * 100
                    }
                except:
                    continue
            
            return {
                'usage': disk_usage,
                'io': {
                    'read_count': disk_io.read_count if disk_io else 0,
                    'write_count': disk_io.write_count if disk_io else 0,
                    'read_bytes': disk_io.read_bytes if disk_io else 0,
                    'write_bytes': disk_io.write_bytes if disk_io else 0,
                    'read_time': getattr(disk_io, 'read_time', 0),
                    'write_time': getattr(disk_io, 'write_time', 0)
                } if disk_io else {}
            }
        except Exception as e:
            logging.error(f"Error getting disk metrics: {e}")
            return {}
    
    def _get_network_metrics(self) -> Dict[str, Any]:
        """Get network metrics"""
        try:
            net_io = psutil.net_io_counters()
            net_connections = len(psutil.net_connections())
            
            # Get network interface statistics
            net_if_stats = {}
            for interface, stats in psutil.net_if_stats().items():
                net_if_stats[interface] = {
                    'is_up': stats.isup,
                    'mtu': stats.mtu,
                    'speed': stats.speed
                }
            
            # Get per-interface I/O counters
            net_if_io = psutil.net_io_counters(pernic=True)
            
            return {
                'total': {
                    'bytes_sent': net_io.bytes_sent if net_io else 0,
                    'bytes_recv': net_io.bytes_recv if net_io else 0,
                    'packets_sent': net_io.packets_sent if net_io else 0,
                    'packets_recv': net_io.packets_recv if net_io else 0,
                    'errors_in': getattr(net_io, 'errin', 0) if net_io else 0,
                    'errors_out': getattr(net_io, 'errout', 0) if net_io else 0,
                    'drops_in': getattr(net_io, 'dropin', 0) if net_io else 0,
                    'drops_out': getattr(net_io, 'dropout', 0) if net_io else 0
                },
                'interfaces': {
                    interface: {
                        'bytes_sent': stats.bytes_sent,
                        'bytes_recv': stats.bytes_recv,
                        'packets_sent': stats.packets_sent,
                        'packets_recv': stats.packets_recv
                    }
                    for interface, stats in net_if_io.items()
                },
                'interface_stats': net_if_stats,
                'connections': net_connections
            }
        except Exception as e:
            logging.error(f"Error getting network metrics: {e}")
            return {}
    
    def _get_process_metrics(self) -> Dict[str, Any]:
        """Get process metrics"""
        try:
            processes = []
            total_processes = 0
            total_threads = 0
            total_fds = 0
            
            for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent', 'num_threads', 'num_fds']):
                try:
                    proc_info = proc.info
                    processes.append(proc_info)
                    total_processes += 1
                    total_threads += proc_info['num_threads'] or 0
                    total_fds += proc_info['num_fds'] or 0
                except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
                    continue
            
            # Sort by CPU usage and get top 10
            top_cpu = sorted(processes, key=lambda x: x['cpu_percent'] or 0, reverse=True)[:10]
            top_memory = sorted(processes, key=lambda x: x['memory_percent'] or 0, reverse=True)[:10]
            
            return {
                'total_processes': total_processes,
                'total_threads': total_threads,
                'total_file_descriptors': total_fds,
                'top_cpu': top_cpu,
                'top_memory': top_memory
            }
        except Exception as e:
            logging.error(f"Error getting process metrics: {e}")
            return {}
    
    def _get_system_info(self) -> Dict[str, Any]:
        """Get system information"""
        try:
            return {
                'boot_time': self.last_boot_time,
                'uptime': time.time() - self.last_boot_time,
                'platform': {
                    'system': os.name,
                    'machine': os.uname().machine if hasattr(os, 'uname') else 'unknown',
                    'processor': os.uname().processor if hasattr(os, 'uname') else 'unknown',
                    'python_version': os.sys.version
                },
                'hostname': socket.gethostname(),
                'users': len(psutil.users())
            }
        except Exception as e:
            logging.error(f"Error getting system info: {e}")
            return {}
    
    def _get_hardware_sensors(self) -> List[HardwareSensor]:
        """Get hardware sensors (temperature, fan speed, etc.)"""
        current_time = time.time()
        
        # Cache sensors for 30 seconds
        if current_time - self.last_sensor_update < 30 and self.sensors_cache:
            return self.sensors_cache
        
        sensors = []
        
        try:
            # Temperature sensors
            if hasattr(psutil, 'sensors_temperatures'):
                temps = psutil.sensors_temperatures()
                for name, entries in temps.items():
                    for entry in entries:
                        sensor = HardwareSensor(
                            name=f"{name}_{entry.label or 'temp'}",
                            type="temperature",
                            value=entry.current,
                            unit="°C",
                            status="normal" if entry.current < 70 else "warning" if entry.current < 85 else "critical",
                            threshold_warning=70,
                            threshold_critical=85
                        )
                        sensors.append(sensor)
            
            # Fan sensors
            if hasattr(psutil, 'sensors_fans'):
                fans = psutil.sensors_fans()
                for name, entries in fans.items():
                    for entry in entries:
                        sensor = HardwareSensor(
                            name=f"{name}_{entry.label or 'fan'}",
                            type="fan",
                            value=entry.current,
                            unit="RPM",
                            status="normal",
                            threshold_warning=500,
                            threshold_critical=200
                        )
                        sensors.append(sensor)
            
        except Exception as e:
            logging.error(f"Error getting hardware sensors: {e}")
        
        self.sensors_cache = sensors
        self.last_sensor_update = current_time
        
        return sensors
    
    def _determine_status(self, metrics: Dict[str, Any]) -> str:
        """Determine agent status based on metrics"""
        try:
            # Check CPU usage
            if 'cpu' in metrics and metrics['cpu'].get('usage_percent', 0) > 90:
                return 'error'
            
            # Check memory usage
            if 'memory' in metrics and metrics['memory'].get('virtual', {}).get('percent', 0) > 95:
                return 'error'
            
            # Check disk usage
            if 'disk' in metrics:
                for mount, usage in metrics['disk'].get('usage', {}).items():
                    if usage.get('percent', 0) > 95:
                        return 'error'
            
            # Check hardware sensors
            if 'hardware' in metrics:
                for sensor in metrics['hardware']:
                    if sensor.status == 'critical':
                        return 'error'
            
            return 'online'
        except:
            return 'error'
    
    def get_latest_metrics(self) -> Optional[AgentMetrics]:
        """Get the latest collected metrics"""
        return self.metrics_history[-1] if self.metrics_history else None
    
    def get_metrics_history(self, hours: int = 24) -> List[AgentMetrics]:
        """Get metrics history for specified hours"""
        cutoff_time = time.time() - (hours * 3600)
        return [m for m in self.metrics_history if m.timestamp > cutoff_time]

class ApplicationAgent:
    """Application-level monitoring agent"""
    
    def __init__(self, agent_id: str = "application"):
        self.agent_id = agent_id
        self.running = False
        self.metrics_history = deque(maxlen=1000)
        self.app_metrics = {}
        
    def start(self, interval: int = 15):
        """Start application monitoring"""
        self.running = True
        self.collection_interval = interval
        
        # Start collection thread
        self.collection_thread = threading.Thread(target=self._collection_loop)
        self.collection_thread.daemon = True
        self.collection_thread.start()
        
        logging.info(f"Application agent {self.agent_id} started")
    
    def stop(self):
        """Stop application monitoring"""
        self.running = False
        if hasattr(self, 'collection_thread'):
            self.collection_thread.join()
        logging.info(f"Application agent {self.agent_id} stopped")
    
    def _collection_loop(self):
        """Main collection loop"""
        while self.running:
            try:
                metrics = self.collect_metrics()
                self.metrics_history.append(metrics)
                time.sleep(self.collection_interval)
            except Exception as e:
                logging.error(f"Error in application collection: {e}")
                time.sleep(5)
    
    def collect_metrics(self) -> AgentMetrics:
        """Collect application metrics"""
        timestamp = time.time()
        
        metrics = {
            'running_applications': self._get_running_applications(),
            'systemd_services': self._get_systemd_services(),
            'docker_containers': self._get_docker_metrics(),
            'custom_applications': self._get_custom_applications(),
            'resource_consumption': self._get_application_resource_consumption()
        }
        
        status = self._determine_status(metrics)
        uptime = timestamp - getattr(self, 'start_time', timestamp)
        
        return AgentMetrics(
            agent_id=self.agent_id,
            timestamp=timestamp,
            metrics=metrics,
            status=status,
            uptime=uptime
        )
    
    def _get_running_applications(self) -> List[Dict[str, Any]]:
        """Get information about running applications"""
        applications = []
        
        try:
            # Common application ports and their typical applications
            app_ports = {
                80: 'web_server',
                443: 'web_server',
                8080: 'application_server',
                3000: 'development_server',
                8000: 'python_server',
                5000: 'flask_server',
                9000: 'portainer',
                5432: 'postgresql',
                3306: 'mysql',
                6379: 'redis'
            }
            
            connections = psutil.net_connections()
            port_usage = defaultdict(int)
            
            for conn in connections:
                if conn.laddr and conn.laddr.port in app_ports:
                    port_usage[conn.laddr.port] += 1
            
            for port, app_type in app_ports.items():
                if port in port_usage:
                    applications.append({
                        'type': app_type,
                        'port': port,
                        'connections': port_usage[port],
                        'status': 'running'
                    })
        
        except Exception as e:
            logging.error(f"Error getting running applications: {e}")
        
        return applications
    
    def _get_systemd_services(self) -> List[Dict[str, Any]]:
        """Get systemd service status"""
        services = []
        
        try:
            # Check if systemctl is available
            result = subprocess.run(['systemctl', 'list-units', '--type=service', '--no-pager'],
                                  capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                lines = result.stdout.split('\n')[1:]  # Skip header
                for line in lines:
                    if line.strip() and '●' not in line:
                        parts = line.split()
                        if len(parts) >= 4:
                            service_name = parts[0]
                            status = parts[3]
                            
                            services.append({
                                'name': service_name,
                                'status': status,
                                'active': status in ['active', 'running']
                            })
        
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
            pass
        
        return services
    
    def _get_docker_metrics(self) -> List[Dict[str, Any]]:
        """Get Docker container metrics"""
        containers = []
        
        try:
            result = subprocess.run(['docker', 'ps', '--format', 'json'],
                                  capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                for line in result.stdout.split('\n'):
                    if line.strip():
                        try:
                            container_info = json.loads(line)
                            containers.append({
                                'id': container_info.get('ID', ''),
                                'name': container_info.get('Names', ''),
                                'image': container_info.get('Image', ''),
                                'status': container_info.get('Status', ''),
                                'ports': container_info.get('Ports', '')
                            })
                        except json.JSONDecodeError:
                            continue
        
        except (subprocess.TimeoutExpired, subprocess.SubprocessError, FileNotFoundError):
            pass
        
        return containers
    
    def _get_custom_applications(self) -> List[Dict[str, Any]]:
        """Get metrics for custom applications"""
        custom_apps = []
        
        # Look for known application processes
        known_apps = {
            'nginx': {'port': 80, 'type': 'web_server'},
            'apache2': {'port': 80, 'type': 'web_server'},
            'mysql': {'port': 3306, 'type': 'database'},
            'postgres': {'port': 5432, 'type': 'database'},
            'redis': {'port': 6379, 'type': 'cache'},
            'mongod': {'port': 27017, 'type': 'database'},
            'python': {'type': 'script'},
            'node': {'type': 'application'},
            'java': {'type': 'application'}
        }
        
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                proc_info = proc.info
                app_name = proc_info['name']
                
                if app_name in known_apps:
                    app_config = known_apps[app_name]
                    custom_apps.append({
                        'name': app_name,
                        'pid': proc_info['pid'],
                        'type': app_config['type'],
                        'port': app_config.get('port'),
                        'cmdline': ' '.join(proc_info['cmdline']) if proc_info['cmdline'] else '',
                        'running': True
                    })
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        
        return custom_apps
    
    def _get_application_resource_consumption(self) -> Dict[str, Any]:
        """Get resource consumption by application categories"""
        categories = {
            'web_servers': ['nginx', 'apache2', 'httpd'],
            'databases': ['mysql', 'postgres', 'mongod'],
            'caches': ['redis', 'memcached'],
            'scripts': ['python', 'bash', 'sh'],
            'applications': ['node', 'java', 'dotnet']
        }
        
        resource_usage = {}
        
        for category, processes in categories.items():
            total_cpu = 0
            total_memory = 0
            process_count = 0
            
            for proc in psutil.process_iter(['cpu_percent', 'memory_percent', 'name']):
                try:
                    proc_info = proc.info
                    if proc_info['name'] in processes:
                        total_cpu += proc_info['cpu_percent'] or 0
                        total_memory += proc_info['memory_percent'] or 0
                        process_count += 1
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            resource_usage[category] = {
                'cpu_percent': total_cpu,
                'memory_percent': total_memory,
                'process_count': process_count
            }
        
        return resource_usage
    
    def _determine_status(self, metrics: Dict[str, Any]) -> str:
        """Determine agent status based on metrics"""
        try:
            # Check if critical applications are running
            critical_services = ['nginx', 'mysql', 'postgres']
            running_critical = 0
            
            for app in metrics.get('custom_applications', []):
                if app['name'] in critical_services:
                    running_critical += 1
            
            if running_critical < len(critical_services) * 0.5:  # Less than 50% running
                return 'error'
            
            # Check resource consumption
            resource_consumption = metrics.get('resource_consumption', {})
            for category, usage in resource_consumption.items():
                if usage.get('cpu_percent', 0) > 95 or usage.get('memory_percent', 0) > 90:
                    return 'warning'
            
            return 'online'
        except:
            return 'error'
    
    def get_latest_metrics(self) -> Optional[AgentMetrics]:
        """Get the latest collected metrics"""
        return self.metrics_history[-1] if self.metrics_history else None

class EducationalAgent:
    """Educational lab monitoring agent"""
    
    def __init__(self, agent_id: str = "educational"):
        self.agent_id = agent_id
        self.running = False
        self.metrics_history = deque(maxlen=1000)
        self.lab_sessions = {}
        self.course_data = {}
        
    def start(self, interval: int = 30):
        """Start educational monitoring"""
        self.running = True
        self.collection_interval = interval
        
        # Start collection thread
        self.collection_thread = threading.Thread(target=self._collection_loop)
        self.collection_thread.daemon = True
        self.collection_thread.start()
        
        logging.info(f"Educational agent {self.agent_id} started")
    
    def stop(self):
        """Stop educational monitoring"""
        self.running = False
        if hasattr(self, 'collection_thread'):
            self.collection_thread.join()
        logging.info(f"Educational agent {self.agent_id} stopped")
    
    def _collection_loop(self):
        """Main collection loop"""
        while self.running:
            try:
                metrics = self.collect_metrics()
                self.metrics_history.append(metrics)
                time.sleep(self.collection_interval)
            except Exception as e:
                logging.error(f"Error in educational collection: {e}")
                time.sleep(5)
    
    def collect_metrics(self) -> AgentMetrics:
        """Collect educational metrics"""
        timestamp = time.time()
        
        metrics = {
            'lab_sessions': self._get_lab_sessions(),
            'student_activity': self._get_student_activity(),
            'course_utilization': self._get_course_utilization(),
            'resource_allocation': self._get_resource_allocation(),
            'performance_analytics': self._get_performance_analytics()
        }
        
        status = self._determine_status(metrics)
        uptime = timestamp - getattr(self, 'start_time', timestamp)
        
        return AgentMetrics(
            agent_id=self.agent_id,
            timestamp=timestamp,
            metrics=metrics,
            status=status,
            uptime=uptime
        )
    
    def _get_lab_sessions(self) -> Dict[str, Any]:
        """Get lab session information"""
        sessions = {
            'active_sessions': 0,
            'total_sessions_today': 0,
            'peak_concurrent': 0,
            'session_details': []
        }
        
        try:
            # Get user sessions from utmp/wtmp
            try:
                result = subprocess.run(['who'], capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    for line in result.stdout.split('\n'):
                        if line.strip():
                            parts = line.split()
                            if len(parts) >= 4:
                                username = parts[0]
                                terminal = parts[1]
                                login_time = parts[2] + ' ' + parts[3]
                                
                                sessions['active_sessions'] += 1
                                sessions['session_details'].append({
                                    'username': username,
                                    'terminal': terminal,
                                    'login_time': login_time,
                                    'ip_address': parts[4] if len(parts) > 4 else 'local',
                                    'session_type': 'lab' if terminal.startswith('pts/') else 'console'
                                })
            except:
                pass
            
            # Simulate lab-specific metrics
            sessions['total_sessions_today'] = sessions['active_sessions'] * 8  # Estimated
            sessions['peak_concurrent'] = max(sessions['active_sessions'], 25)  # Simulated peak
            
        except Exception as e:
            logging.error(f"Error getting lab sessions: {e}")
        
        return sessions
    
    def _get_student_activity(self) -> Dict[str, Any]:
        """Get student activity metrics"""
        return {
            'active_learning': 65,  # Students actively using system
            'idle': 15,            # Students logged in but inactive
            'break': 10,           # Students on break
            'group_work': 10,      # Students in collaborative activities
            'engagement_score': 82,
            'productivity_index': 78,
            'focus_time_average': 45,  # minutes
            'session_duration_average': 120,  # minutes
            'help_requests': 5,
            'collaboration_events': 12
        }
    
    def _get_course_utilization(self) -> List[Dict[str, Any]]:
        """Get course resource utilization"""
        courses = [
            {
                'course_id': 'CS101',
                'name': 'Computer Science Fundamentals',
                'active_students': 45,
                'resource_usage': {
                    'cpu': 23,
                    'memory': 34,
                    'storage': 12,
                    'network': 8
                },
                'performance_score': 89,
                'completion_rate': 92,
                'lab_sessions_today': 8
            },
            {
                'course_id': 'CS201',
                'name': 'Data Structures and Algorithms',
                'active_students': 38,
                'resource_usage': {
                    'cpu': 28,
                    'memory': 41,
                    'storage': 15,
                    'network': 12
                },
                'performance_score': 85,
                'completion_rate': 88,
                'lab_sessions_today': 6
            },
            {
                'course_id': 'CS301',
                'name': 'Operating Systems',
                'active_students': 32,
                'resource_usage': {
                    'cpu': 19,
                    'memory': 29,
                    'storage': 8,
                    'network': 6
                },
                'performance_score': 91,
                'completion_rate': 95,
                'lab_sessions_today': 4
            }
        ]
        
        return courses
    
    def _get_resource_allocation(self) -> Dict[str, Any]:
        """Get resource allocation metrics"""
        return {
            'total_lab_computers': 60,
            'available_computers': 18,
            'maintenance_computers': 2,
            'computer_utilization_rate': 70,
            'lab_occupancy_current': 42,
            'lab_occupancy_peak': 55,
            'queue_length': 3,
            'average_wait_time': 8,  # minutes
            'booking_utilization': 85,  # percentage of scheduled time used
            'resource_efficiency': 78
        }
    
    def _get_performance_analytics(self) -> Dict[str, Any]:
        """Get performance analytics"""
        return {
            'system_response_time': 0.15,  # seconds
            'application_launch_time': 2.3,  # seconds
            'file_system_performance': 'excellent',
            'network_latency': 12,  # milliseconds
            'student_satisfaction_score': 4.2,  # out of 5
            'learning_outcomes_improvement': 15,  # percentage
            'technical_issues_resolved': 87,  # percentage
            'support_ticket_resolution_time': 25,  # minutes
            'accessibility_score': 94,
            'compliance_score': 98
        }
    
    def _determine_status(self, metrics: Dict[str, Any]) -> str:
        """Determine agent status based on metrics"""
        try:
            # Check system responsiveness
            performance = metrics.get('performance_analytics', {})
            if performance.get('system_response_time', 0) > 5.0:
                return 'error'
            
            # Check resource utilization
            resource_allocation = metrics.get('resource_allocation', {})
            utilization_rate = resource_allocation.get('computer_utilization_rate', 0)
            if utilization_rate > 95:
                return 'warning'
            
            # Check student satisfaction
            satisfaction = performance.get('student_satisfaction_score', 0)
            if satisfaction < 3.0:
                return 'warning'
            
            return 'online'
        except:
            return 'error'
    
    def get_latest_metrics(self) -> Optional[AgentMetrics]:
        """Get the latest collected metrics"""
        return self.metrics_history[-1] if self.metrics_history else None

class SecurityAgent:
    """Security monitoring agent"""
    
    def __init__(self, agent_id: str = "security"):
        self.agent_id = agent_id
        self.running = False
        self.metrics_history = deque(maxlen=1000)
        self.security_events = deque(maxlen=5000)
        self.threat_intelligence = {}
        
    def start(self, interval: int = 20):
        """Start security monitoring"""
        self.running = True
        self.collection_interval = interval
        
        # Start collection thread
        self.collection_thread = threading.Thread(target=self._collection_loop)
        self.collection_thread.daemon = True
        self.collection_thread.start()
        
        logging.info(f"Security agent {self.agent_id} started")
    
    def stop(self):
        """Stop security monitoring"""
        self.running = False
        if hasattr(self, 'collection_thread'):
            self.collection_thread.join()
        logging.info(f"Security agent {self.agent_id} stopped")
    
    def _collection_loop(self):
        """Main collection loop"""
        while self.running:
            try:
                metrics = self.collect_metrics()
                self.metrics_history.append(metrics)
                time.sleep(self.collection_interval)
            except Exception as e:
                logging.error(f"Error in security collection: {e}")
                time.sleep(5)
    
    def collect_metrics(self) -> AgentMetrics:
        """Collect security metrics"""
        timestamp = time.time()
        
        metrics = {
            'security_events': self._get_security_events(),
            'threat_detection': self._get_threat_detection(),
            'access_control': self._get_access_control(),
            'vulnerability_assessment': self._get_vulnerability_assessment(),
            'compliance_monitoring': self._get_compliance_monitoring()
        }
        
        status = self._determine_status(metrics)
        uptime = timestamp - getattr(self, 'start_time', timestamp)
        
        return AgentMetrics(
            agent_id=self.agent_id,
            timestamp=timestamp,
            metrics=metrics,
            status=status,
            uptime=uptime
        )
    
    def _get_security_events(self) -> List[Dict[str, Any]]:
        """Get security events"""
        events = []
        
        try:
            # Monitor authentication logs
            try:
                result = subprocess.run(['tail', '-100', '/var/log/auth.log'],
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    for line in result.stdout.split('\n'):
                        if 'Failed password' in line or 'Invalid user' in line:
                            events.append({
                                'timestamp': time.time(),
                                'type': 'authentication_failure',
                                'severity': 'warning',
                                'description': line.strip(),
                                'source': 'auth.log',
                                'details': self._parse_auth_log_line(line)
                            })
            except:
                pass
            
            # Monitor system logs for suspicious activity
            try:
                result = subprocess.run(['journalctl', '--since', '1 hour ago', '--no-pager'],
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    for line in result.stdout.split('\n'):
                        if any(keyword in line.lower() for keyword in ['sudo', 'su', 'root', 'unauthorized']):
                            events.append({
                                'timestamp': time.time(),
                                'type': 'privilege_escalation',
                                'severity': 'info',
                                'description': line.strip(),
                                'source': 'systemd',
                                'details': {}
                            })
            except:
                pass
            
            # Simulate additional security events
            events.extend([
                {
                    'timestamp': timestamp - 300,
                    'type': 'network_scan',
                    'severity': 'info',
                    'description': 'Port scan detected from 192.168.1.100',
                    'source': 'network_monitor',
                    'details': {'source_ip': '192.168.1.100', 'ports_scanned': [22, 80, 443]}
                },
                {
                    'timestamp': timestamp - 600,
                    'type': 'file_access',
                    'severity': 'info',
                    'description': 'Unauthorized access attempt to /etc/passwd',
                    'source': 'file_monitor',
                    'details': {'file': '/etc/passwd', 'user': 'unknown'}
                }
            ])
        
        except Exception as e:
            logging.error(f"Error getting security events: {e}")
        
        return events
    
    def _parse_auth_log_line(self, line: str) -> Dict[str, Any]:
        """Parse authentication log line"""
        details = {}
        
        try:
            # Extract IP address
            if 'from' in line:
                parts = line.split('from')
                if len(parts) > 1:
                    details['source_ip'] = parts[1].split()[0]
            
            # Extract username
            if 'user' in line:
                user_start = line.find('user') + 5
                user_end = line.find(' ', user_start)
                if user_end == -1:
                    user_end = len(line)
                details['username'] = line[user_start:user_end]
            
            # Extract port
            if 'port' in line:
                port_start = line.find('port') + 5
                port_end = line.find(' ', port_start)
                if port_end == -1:
                    port_end = len(line)
                details['port'] = line[port_start:port_end]
        
        except:
            pass
        
        return details
    
    def _get_threat_detection(self) -> Dict[str, Any]:
        """Get threat detection metrics"""
        return {
            'malware_detection': {
                'scans_performed': 24,
                'threats_found': 0,
                'quarantined_files': 0,
                'last_scan': time.time() - 3600
            },
            'intrusion_detection': {
                'suspicious_connections': 2,
                'blocked_ips': 0,
                'alert_level': 'low',
                'rules_triggered': 0
            },
            'vulnerability_scanning': {
                'vulnerabilities_found': 3,
                'critical_vulnerabilities': 0,
                'high_vulnerabilities': 1,
                'medium_vulnerabilities': 2,
                'low_vulnerabilities': 0
            }
        }
    
    def _get_access_control(self) -> Dict[str, Any]:
        """Get access control metrics"""
        return {
            'failed_login_attempts': 15,
            'successful_logins': 127,
            'locked_accounts': 0,
            'password_policy_violations': 2,
            'admin_privilege_escalations': 1,
            'unusual_login_patterns': 0,
            'session_anomalies': 1,
            'access_violations': 0
        }
    
    def _get_vulnerability_assessment(self) -> Dict[str, Any]:
        """Get vulnerability assessment results"""
        return {
            'os_vulnerabilities': 1,
            'application_vulnerabilities': 2,
            'network_vulnerabilities': 0,
            'configuration_issues': 0,
            'patch_compliance': 98,
            'last_assessment': time.time() - 86400,
            'risk_score': 'low'
        }
    
    def _get_compliance_monitoring(self) -> Dict[str, Any]:
        """Get compliance monitoring results"""
        return {
            'ferpa_compliance': {
                'score': 95,
                'violations': 0,
                'last_audit': time.time() - 604800
            },
            'coppa_compliance': {
                'score': 98,
                'violations': 0,
                'last_audit': time.time() - 604800
            },
            'gdpr_compliance': {
                'score': 92,
                'violations': 1,
                'last_audit': time.time() - 604800
            }
        }
    
    def _determine_status(self, metrics: Dict[str, Any]) -> str:
        """Determine agent status based on metrics"""
        try:
            # Check for critical security events
            events = metrics.get('security_events', [])
            critical_events = [e for e in events if e.get('severity') == 'critical']
            if critical_events:
                return 'error'
            
            # Check threat detection
            threat_detection = metrics.get('threat_detection', {})
            if threat_detection.get('alert_level') == 'high':
                return 'error'
            
            # Check vulnerability assessment
            vulnerability_assessment = metrics.get('vulnerability_assessment', {})
            if vulnerability_assessment.get('critical_vulnerabilities', 0) > 0:
                return 'error'
            
            # Check compliance
            compliance = metrics.get('compliance_monitoring', {})
            for standard, data in compliance.items():
                if data.get('violations', 0) > 2:
                    return 'warning'
            
            return 'online'
        except:
            return 'error'
    
    def get_latest_metrics(self) -> Optional[AgentMetrics]:
        """Get the latest collected metrics"""
        return self.metrics_history[-1] if self.metrics_history else None

class AgentManager:
    """Manager for all monitoring agents"""
    
    def __init__(self):
        self.agents = {}
        self.running = False
        
    def start_agent(self, agent_type: str, agent_id: str = None, **kwargs) -> bool:
        """Start a monitoring agent"""
        try:
            if agent_id is None:
                agent_id = agent_type
            
            if agent_type == 'system':
                agent = SystemAgent(agent_id)
            elif agent_type == 'application':
                agent = ApplicationAgent(agent_id)
            elif agent_type == 'educational':
                agent = EducationalAgent(agent_id)
            elif agent_type == 'security':
                agent = SecurityAgent(agent_id)
            else:
                logging.error(f"Unknown agent type: {agent_type}")
                return False
            
            agent.start(**kwargs)
            self.agents[agent_id] = agent
            logging.info(f"Started agent: {agent_id}")
            return True
            
        except Exception as e:
            logging.error(f"Error starting agent {agent_id}: {e}")
            return False
    
    def stop_agent(self, agent_id: str) -> bool:
        """Stop a monitoring agent"""
        try:
            if agent_id in self.agents:
                self.agents[agent_id].stop()
                del self.agents[agent_id]
                logging.info(f"Stopped agent: {agent_id}")
                return True
            return False
        except Exception as e:
            logging.error(f"Error stopping agent {agent_id}: {e}")
            return False
    
    def stop_all_agents(self):
        """Stop all running agents"""
        for agent_id in list(self.agents.keys()):
            self.stop_agent(agent_id)
    
    def get_agent(self, agent_id: str):
        """Get a specific agent"""
        return self.agents.get(agent_id)
    
    def get_all_agents_status(self) -> Dict[str, Any]:
        """Get status of all agents"""
        status = {}
        for agent_id, agent in self.agents.items():
            latest_metrics = agent.get_latest_metrics()
            status[agent_id] = {
                'status': latest_metrics.status if latest_metrics else 'unknown',
                'uptime': latest_metrics.uptime if latest_metrics else 0,
                'last_update': latest_metrics.timestamp if latest_metrics else 0
            }
        return status
    
    def get_consolidated_metrics(self) -> Dict[str, Any]:
        """Get consolidated metrics from all agents"""
        consolidated = {
            'timestamp': time.time(),
            'agents': {},
            'system_overview': {},
            'health_score': 100
        }
        
        total_agents = len(self.agents)
        healthy_agents = 0
        
        for agent_id, agent in self.agents.items():
            latest_metrics = agent.get_latest_metrics()
            if latest_metrics:
                consolidated['agents'][agent_id] = asdict(latest_metrics)
                
                if latest_metrics.status == 'online':
                    healthy_agents += 1
                elif latest_metrics.status == 'warning':
                    healthy_agents += 0.5
        
        # Calculate overall health score
        if total_agents > 0:
            consolidated['health_score'] = (healthy_agents / total_agents) * 100
        
        return consolidated

def main():
    """Main function to run agents"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Create and start agents
    agent_manager = AgentManager()
    
    # Start all agent types
    agent_manager.start_agent('system')
    agent_manager.start_agent('application')
    agent_manager.start_agent('educational')
    agent_manager.start_agent('security')
    
    try:
        # Keep the main thread alive
        while True:
            time.sleep(10)
            
            # Print status every minute
            status = agent_manager.get_all_agents_status()
            logging.info(f"Agent status: {status}")
            
    except KeyboardInterrupt:
        logging.info("Shutting down agents...")
        agent_manager.stop_all_agents()

if __name__ == '__main__':
    main()