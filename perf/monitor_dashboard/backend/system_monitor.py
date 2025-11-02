#!/usr/bin/env python3
"""
System Monitor - Real-time performance monitoring and data collection
Collects system metrics, process information, and performance counters
"""

import psutil
import time
import json
import threading
import sqlite3
import logging
from datetime import datetime, timedelta
from collections import deque, defaultdict
from typing import Dict, List, Any, Optional
import queue
import os

class SystemMonitor:
    def __init__(self, db_path: str = "data/monitor.db", history_size: int = 1000):
        self.db_path = db_path
        self.history_size = history_size
        self.is_monitoring = False
        self.monitor_thread = None
        self.data_queue = queue.Queue()
        
        # Historical data storage
        self.cpu_history = deque(maxlen=history_size)
        self.memory_history = deque(maxlen=history_size)
        self.disk_history = deque(maxlen=history_size)
        self.network_history = deque(maxlen=history_size)
        self.process_history = deque(maxlen=history_size)
        
        # Performance counters
        self.last_cpu_times = psutil.cpu_times()
        self.last_network_io = psutil.net_io_counters()
        self.last_disk_io = psutil.disk_io_counters()
        
        # Process tracking
        self.process_cache = {}
        self.process_alerts = {}
        
        # Setup logging
        self.setup_logging()
        
        # Initialize database
        self.init_database()
        
        # Custom metrics
        self.custom_metrics = {}
        
        # Alert thresholds
        self.thresholds = {
            'cpu': {'warning': 80, 'critical': 95},
            'memory': {'warning': 85, 'critical': 95},
            'disk': {'warning': 85, 'critical': 95},
            'load_avg': {'warning': 2.0, 'critical': 4.0},
            'network': {'warning': 1000000, 'critical': 5000000},  # bytes/sec
        }
    
    def setup_logging(self):
        """Setup logging configuration"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('logs/monitor.log'),
                logging.StreamHandler()
            ]
        )
        self.logger = logging.getLogger('SystemMonitor')
    
    def init_database(self):
        """Initialize SQLite database for historical data"""
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        
        # System metrics table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS system_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME,
                cpu_percent REAL,
                cpu_freq REAL,
                load_avg TEXT,
                memory_percent REAL,
                memory_used REAL,
                memory_total REAL,
                disk_usage TEXT,
                disk_io TEXT,
                network_io TEXT
            )
        ''')
        
        # Process metrics table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS process_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME,
                pid INTEGER,
                name TEXT,
                cpu_percent REAL,
                memory_percent REAL,
                memory_used REAL,
                status TEXT,
                num_threads INTEGER
            )
        ''')
        
        # Custom metrics table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS custom_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME,
                metric_name TEXT,
                metric_value REAL,
                metric_type TEXT,
                tags TEXT
            )
        ''')
        
        # Alerts table
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME,
                alert_type TEXT,
                severity TEXT,
                message TEXT,
                metric_value REAL,
                threshold REAL,
                acknowledged BOOLEAN DEFAULT FALSE
            )
        ''')
        
        conn.commit()
        conn.close()
        
        self.logger.info("Database initialized successfully")
    
    def get_cpu_metrics(self) -> Dict[str, Any]:
        """Collect CPU performance metrics"""
        try:
            cpu_percent = psutil.cpu_percent(interval=1)
            cpu_freq = psutil.cpu_freq()
            cpu_count = psutil.cpu_count()
            cpu_count_logical = psutil.cpu_count(logical=True)
            
            # Calculate per-core CPU usage
            cpu_per_core = psutil.cpu_percent(interval=1, percpu=True)
            
            # Load average (Linux/Unix only)
            try:
                load_avg = os.getloadavg()
            except (OSError, AttributeError):
                load_avg = [0, 0, 0]
            
            # CPU times
            cpu_times = psutil.cpu_times()
            cpu_times_percent = psutil.cpu_times_percent(interval=1)
            
            # CPU usage per process
            processes = []
            for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent']):
                try:
                    proc_info = proc.info
                    if proc_info['cpu_percent'] > 0:  # Only include active processes
                        processes.append(proc_info)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            # Sort by CPU usage
            processes.sort(key=lambda x: x['cpu_percent'], reverse=True)
            top_processes = processes[:10]
            
            return {
                'cpu_percent': cpu_percent,
                'cpu_freq': {
                    'current': cpu_freq.current if cpu_freq else 0,
                    'min': cpu_freq.min if cpu_freq else 0,
                    'max': cpu_freq.max if cpu_freq else 0
                },
                'cpu_count': cpu_count,
                'cpu_count_logical': cpu_count_logical,
                'cpu_per_core': cpu_per_core,
                'load_avg': {
                    '1min': load_avg[0],
                    '5min': load_avg[1],
                    '15min': load_avg[2]
                },
                'cpu_times': {
                    'user': cpu_times.user,
                    'nice': cpu_times.nice,
                    'system': cpu_times.system,
                    'idle': cpu_times.idle,
                    'iowait': getattr(cpu_times, 'iowait', 0),
                    'irq': getattr(cpu_times, 'irq', 0),
                    'softirq': getattr(cpu_times, 'softirq', 0)
                },
                'cpu_times_percent': {
                    'user': cpu_times_percent.user,
                    'nice': cpu_times_percent.nice,
                    'system': cpu_times_percent.system,
                    'idle': cpu_times_percent.idle,
                    'iowait': getattr(cpu_times_percent, 'iowait', 0),
                    'irq': getattr(cpu_times_percent, 'irq', 0),
                    'softirq': getattr(cpu_times_percent, 'softirq', 0)
                },
                'top_processes': top_processes,
                'timestamp': datetime.now().isoformat()
            }
        except Exception as e:
            self.logger.error(f"Error collecting CPU metrics: {e}")
            return {}
    
    def get_memory_metrics(self) -> Dict[str, Any]:
        """Collect memory performance metrics"""
        try:
            memory = psutil.virtual_memory()
            swap = psutil.swap_memory()
            
            # Memory per process
            processes = []
            for proc in psutil.process_iter(['pid', 'name', 'memory_percent', 'memory_info']):
                try:
                    proc_info = proc.info
                    if proc_info['memory_percent'] > 0.1:  # Only include significant processes
                        proc_info['memory_used_mb'] = proc_info['memory_info'].rss / 1024 / 1024
                        processes.append(proc_info)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            # Sort by memory usage
            processes.sort(key=lambda x: x['memory_percent'], reverse=True)
            top_memory_processes = processes[:10]
            
            return {
                'virtual_memory': {
                    'total': memory.total,
                    'available': memory.available,
                    'used': memory.used,
                    'free': memory.free,
                    'percent': memory.percent,
                    'cached': getattr(memory, 'cached', 0),
                    'buffers': getattr(memory, 'buffers', 0),
                    'shared': getattr(memory, 'shared', 0)
                },
                'swap_memory': {
                    'total': swap.total,
                    'used': swap.used,
                    'free': swap.free,
                    'percent': swap.percent,
                    'sin': swap.sin,
                    'sout': swap.sout
                },
                'top_memory_processes': top_memory_processes,
                'timestamp': datetime.now().isoformat()
            }
        except Exception as e:
            self.logger.error(f"Error collecting memory metrics: {e}")
            return {}
    
    def get_disk_metrics(self) -> Dict[str, Any]:
        """Collect disk performance metrics"""
        try:
            # Disk usage
            disk_usage = {}
            partitions = psutil.disk_partitions()
            for partition in partitions:
                try:
                    usage = psutil.disk_usage(partition.mountpoint)
                    disk_usage[partition.mountpoint] = {
                        'device': partition.device,
                        'fstype': partition.fstype,
                        'total': usage.total,
                        'used': usage.used,
                        'free': usage.free,
                        'percent': (usage.used / usage.total) * 100
                    }
                except PermissionError:
                    continue
            
            # Disk I/O
            disk_io = psutil.disk_io_counters(perdisk=True)
            disk_io_data = {}
            if disk_io:
                for device, io_stats in disk_io.items():
                    disk_io_data[device] = {
                        'read_count': io_stats.read_count,
                        'write_count': io_stats.write_count,
                        'read_bytes': io_stats.read_bytes,
                        'write_bytes': io_stats.write_bytes,
                        'read_time': getattr(io_stats, 'read_time', 0),
                        'write_time': getattr(io_stats, 'write_time', 0)
                    }
            
            # Calculate I/O rates
            current_disk_io = psutil.disk_io_counters()
            if current_disk_io and hasattr(self, 'last_disk_io'):
                read_rate = max(0, current_disk_io.read_bytes - self.last_disk_io.read_bytes) / 1024 / 1024  # MB/s
                write_rate = max(0, current_disk_io.write_bytes - self.last_disk_io.write_bytes) / 1024 / 1024  # MB/s
            else:
                read_rate = 0
                write_rate = 0
            
            self.last_disk_io = current_disk_io
            
            # Disk I/O per process
            io_processes = []
            for proc in psutil.process_iter(['pid', 'name', 'num_threads']):
                try:
                    proc_io = proc.io_counters()
                    if proc_io:
                        io_processes.append({
                            'pid': proc.info['pid'],
                            'name': proc.info['name'],
                            'num_threads': proc.info['num_threads'],
                            'read_bytes': proc_io.read_bytes,
                            'write_bytes': proc_io.write_bytes,
                            'total_io': proc_io.read_bytes + proc_io.write_bytes
                        })
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            # Sort by I/O
            io_processes.sort(key=lambda x: x['total_io'], reverse=True)
            top_io_processes = io_processes[:10]
            
            return {
                'disk_usage': disk_usage,
                'disk_io': {
                    'devices': disk_io_data,
                    'read_rate_mbps': read_rate,
                    'write_rate_mbps': write_rate,
                    'total_read_mb': current_disk_io.read_bytes / 1024 / 1024 if current_disk_io else 0,
                    'total_write_mb': current_disk_io.write_bytes / 1024 / 1024 if current_disk_io else 0
                },
                'top_io_processes': top_io_processes,
                'timestamp': datetime.now().isoformat()
            }
        except Exception as e:
            self.logger.error(f"Error collecting disk metrics: {e}")
            return {}
    
    def get_network_metrics(self) -> Dict[str, Any]:
        """Collect network performance metrics"""
        try:
            # Network I/O counters
            network_io = psutil.net_io_counters(pernic=True)
            network_data = {}
            for interface, io_stats in network_io.items():
                network_data[interface] = {
                    'bytes_sent': io_stats.bytes_sent,
                    'bytes_recv': io_stats.bytes_recv,
                    'packets_sent': io_stats.packets_sent,
                    'packets_recv': io_stats.packets_recv,
                    'errin': io_stats.errin,
                    'errout': io_stats.errout,
                    'dropin': io_stats.dropin,
                    'dropout': io_stats.dropout
                }
            
            # Network connections
            connections = psutil.net_connections()
            connection_states = defaultdict(int)
            for conn in connections:
                connection_states[conn.status] += 1
            
            # Network interface stats
            interface_stats = {}
            for interface, stats in psutil.net_if_stats().items():
                interface_stats[interface] = {
                    'is_up': stats.isup,
                    'mtu': stats.mtu,
                    'speed': stats.speed
                }
            
            # Calculate network rates
            current_network_io = psutil.net_io_counters()
            if current_network_io and hasattr(self, 'last_network_io'):
                upload_rate = max(0, current_network_io.bytes_sent - self.last_network_io.bytes_sent) / 1024 / 1024  # MB/s
                download_rate = max(0, current_network_io.bytes_recv - self.last_network_io.bytes_recv) / 1024 / 1024  # MB/s
            else:
                upload_rate = 0
                download_rate = 0
            
            self.last_network_io = current_network_io
            
            return {
                'network_io': {
                    'interfaces': network_data,
                    'upload_rate_mbps': upload_rate,
                    'download_rate_mbps': download_rate,
                    'total_sent_mb': current_network_io.bytes_sent / 1024 / 1024 if current_network_io else 0,
                    'total_recv_mb': current_network_io.bytes_recv / 1024 / 1024 if current_network_io else 0
                },
                'network_connections': {
                    'total': len(connections),
                    'states': dict(connection_states)
                },
                'interface_stats': interface_stats,
                'timestamp': datetime.now().isoformat()
            }
        except Exception as e:
            self.logger.error(f"Error collecting network metrics: {e}")
            return {}
    
    def get_process_metrics(self) -> Dict[str, Any]:
        """Collect detailed process performance metrics"""
        try:
            processes = []
            
            for proc in psutil.process_iter([
                'pid', 'name', 'cpu_percent', 'memory_percent', 'memory_info',
                'status', 'num_threads', 'create_time', 'num_fds', 'num_handles'
            ]):
                try:
                    proc_info = proc.info
                    proc_info['memory_used_mb'] = proc_info['memory_info'].rss / 1024 / 1024 if proc_info['memory_info'] else 0
                    
                    # Calculate CPU time
                    try:
                        cpu_times = proc.cpu_times()
                        proc_info['cpu_time_user'] = cpu_times.user
                        proc_info['cpu_time_system'] = cpu_times.system
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        proc_info['cpu_time_user'] = 0
                        proc_info['cpu_time_system'] = 0
                    
                    # Get I/O stats if available
                    try:
                        io_counters = proc.io_counters()
                        if io_counters:
                            proc_info['io_read_bytes'] = io_counters.read_bytes
                            proc_info['io_write_bytes'] = io_counters.write_bytes
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        proc_info['io_read_bytes'] = 0
                        proc_info['io_write_bytes'] = 0
                    
                    processes.append(proc_info)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            
            # Sort by CPU usage
            cpu_processes = sorted(processes, key=lambda x: x['cpu_percent'], reverse=True)
            
            # Sort by memory usage
            memory_processes = sorted(processes, key=lambda x: x['memory_percent'], reverse=True)
            
            # Process tree
            process_tree = self.build_process_tree(processes)
            
            return {
                'total_processes': len(processes),
                'running_processes': len([p for p in processes if p['status'] == 'running']),
                'sleeping_processes': len([p for p in processes if p['status'] == 'sleeping']),
                'zombie_processes': len([p for p in processes if p['status'] == 'zombie']),
                'top_cpu_processes': cpu_processes[:20],
                'top_memory_processes': memory_processes[:20],
                'process_tree': process_tree,
                'timestamp': datetime.now().isoformat()
            }
        except Exception as e:
            self.logger.error(f"Error collecting process metrics: {e}")
            return {}
    
    def build_process_tree(self, processes: List[Dict]) -> Dict:
        """Build a process tree structure"""
        process_map = {}
        root_processes = []
        
        # Create process map
        for proc in processes:
            process_map[proc['pid']] = {**proc, 'children': []}
        
        # Build tree
        for pid, proc in process_map.items():
            try:
                parent_pid = proc.get('ppid', 0)
                if parent_pid in process_map and parent_pid != pid:
                    process_map[parent_pid]['children'].append(proc)
                else:
                    root_processes.append(proc)
            except Exception:
                root_processes.append(proc)
        
        return {'root': root_processes}
    
    def get_kernel_metrics(self) -> Dict[str, Any]:
        """Collect kernel performance counters"""
        try:
            metrics = {
                'boot_time': psutil.boot_time(),
                'users': len(psutil.users()),
                'system_users': [{'name': user.name, 'terminal': user.terminal, 'host': user.host, 'started': user.started} for user in psutil.users()],
                'timestamp': datetime.now().isoformat()
            }
            
            # System uptime
            uptime = time.time() - psutil.boot_time()
            metrics['uptime_seconds'] = uptime
            metrics['uptime_formatted'] = self.format_uptime(uptime)
            
            return metrics
        except Exception as e:
            self.logger.error(f"Error collecting kernel metrics: {e}")
            return {}
    
    def format_uptime(self, seconds: float) -> str:
        """Format uptime in human readable format"""
        days = int(seconds // 86400)
        hours = int((seconds % 86400) // 3600)
        minutes = int((seconds % 3600) // 60)
        seconds = int(seconds % 60)
        return f"{days}d {hours}h {minutes}m {seconds}s"
    
    def collect_all_metrics(self) -> Dict[str, Any]:
        """Collect all system metrics"""
        self.logger.info("Collecting system metrics...")
        
        metrics = {
            'timestamp': datetime.now().isoformat(),
            'cpu': self.get_cpu_metrics(),
            'memory': self.get_memory_metrics(),
            'disk': self.get_disk_metrics(),
            'network': self.get_network_metrics(),
            'processes': self.get_process_metrics(),
            'kernel': self.get_kernel_metrics()
        }
        
        # Add custom metrics
        for name, metric_func in self.custom_metrics.items():
            try:
                metrics[f'custom_{name}'] = metric_func()
            except Exception as e:
                self.logger.error(f"Error collecting custom metric {name}: {e}")
        
        # Store in history
        self.store_metrics_in_history(metrics)
        
        # Check for alerts
        self.check_alerts(metrics)
        
        return metrics
    
    def store_metrics_in_history(self, metrics: Dict[str, Any]):
        """Store metrics in history buffers"""
        try:
            # CPU history
            if 'cpu' in metrics:
                cpu_data = {
                    'timestamp': metrics['timestamp'],
                    'percent': metrics['cpu'].get('cpu_percent', 0),
                    'load_1min': metrics['cpu'].get('load_avg', {}).get('1min', 0)
                }
                self.cpu_history.append(cpu_data)
            
            # Memory history
            if 'memory' in metrics:
                memory_data = {
                    'timestamp': metrics['timestamp'],
                    'percent': metrics['memory'].get('virtual_memory', {}).get('percent', 0),
                    'used_gb': metrics['memory'].get('virtual_memory', {}).get('used', 0) / (1024**3),
                    'total_gb': metrics['memory'].get('virtual_memory', {}).get('total', 0) / (1024**3)
                }
                self.memory_history.append(memory_data)
            
            # Disk history
            if 'disk' in metrics:
                disk_data = {
                    'timestamp': metrics['timestamp'],
                    'read_rate': metrics['disk'].get('disk_io', {}).get('read_rate_mbps', 0),
                    'write_rate': metrics['disk'].get('disk_io', {}).get('write_rate_mbps', 0)
                }
                self.disk_history.append(disk_data)
            
            # Network history
            if 'network' in metrics:
                network_data = {
                    'timestamp': metrics['timestamp'],
                    'upload_rate': metrics['network'].get('network_io', {}).get('upload_rate_mbps', 0),
                    'download_rate': metrics['network'].get('network_io', {}).get('download_rate_mbps', 0)
                }
                self.network_history.append(network_data)
            
            # Process history
            if 'processes' in metrics:
                process_data = {
                    'timestamp': metrics['timestamp'],
                    'total_processes': metrics['processes'].get('total_processes', 0),
                    'running_processes': metrics['processes'].get('running_processes', 0)
                }
                self.process_history.append(process_data)
            
        except Exception as e:
            self.logger.error(f"Error storing metrics in history: {e}")
    
    def check_alerts(self, metrics: Dict[str, Any]):
        """Check metrics against thresholds and generate alerts"""
        try:
            alerts = []
            
            # CPU alert
            if 'cpu' in metrics:
                cpu_percent = metrics['cpu'].get('cpu_percent', 0)
                if cpu_percent >= self.thresholds['cpu']['critical']:
                    alerts.append(('CPU', 'critical', f'CPU usage critically high: {cpu_percent:.1f}%', cpu_percent, self.thresholds['cpu']['critical']))
                elif cpu_percent >= self.thresholds['cpu']['warning']:
                    alerts.append(('CPU', 'warning', f'CPU usage high: {cpu_percent:.1f}%', cpu_percent, self.thresholds['cpu']['warning']))
            
            # Memory alert
            if 'memory' in metrics:
                memory_percent = metrics['memory'].get('virtual_memory', {}).get('percent', 0)
                if memory_percent >= self.thresholds['memory']['critical']:
                    alerts.append(('Memory', 'critical', f'Memory usage critically high: {memory_percent:.1f}%', memory_percent, self.thresholds['memory']['critical']))
                elif memory_percent >= self.thresholds['memory']['warning']:
                    alerts.append(('Memory', 'warning', f'Memory usage high: {memory_percent:.1f}%', memory_percent, self.thresholds['memory']['warning']))
            
            # Load average alert
            if 'cpu' in metrics:
                load_1min = metrics['cpu'].get('load_avg', {}).get('1min', 0)
                if load_1min >= self.thresholds['load_avg']['critical']:
                    alerts.append(('Load Average', 'critical', f'Load average critically high: {load_1min:.2f}', load_1min, self.thresholds['load_avg']['critical']))
                elif load_1min >= self.thresholds['load_avg']['warning']:
                    alerts.append(('Load Average', 'warning', f'Load average high: {load_1min:.2f}', load_1min, self.thresholds['load_avg']['warning']))
            
            # Process alerts
            if 'processes' in metrics:
                zombie_count = metrics['processes'].get('zombie_processes', 0)
                if zombie_count > 5:
                    alerts.append(('Zombie Processes', 'warning', f'High number of zombie processes: {zombie_count}', zombie_count, 5))
            
            # Store alerts
            for alert_type, severity, message, value, threshold in alerts:
                self.store_alert(alert_type, severity, message, value, threshold)
                self.logger.warning(f"ALERT [{severity.upper()}] {alert_type}: {message}")
        
        except Exception as e:
            self.logger.error(f"Error checking alerts: {e}")
    
    def store_alert(self, alert_type: str, severity: str, message: str, value: float, threshold: float):
        """Store alert in database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            cursor.execute('''
                INSERT INTO alerts (timestamp, alert_type, severity, message, metric_value, threshold)
                VALUES (?, ?, ?, ?, ?, ?)
            ''', (datetime.now().isoformat(), alert_type, severity, message, value, threshold))
            conn.commit()
            conn.close()
        except Exception as e:
            self.logger.error(f"Error storing alert: {e}")
    
    def add_custom_metric(self, name: str, metric_func):
        """Add a custom metric function"""
        self.custom_metrics[name] = metric_func
        self.logger.info(f"Added custom metric: {name}")
    
    def get_historical_data(self, metric_type: str, hours: int = 24) -> List[Dict]:
        """Get historical data for a specific metric"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Calculate start time
            start_time = datetime.now() - timedelta(hours=hours)
            
            if metric_type == 'system':
                cursor.execute('''
                    SELECT * FROM system_metrics 
                    WHERE timestamp > ? 
                    ORDER BY timestamp
                ''', (start_time.isoformat(),))
                
                columns = [description[0] for description in cursor.description]
                return [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            elif metric_type == 'processes':
                cursor.execute('''
                    SELECT * FROM process_metrics 
                    WHERE timestamp > ? 
                    ORDER BY timestamp
                ''', (start_time.isoformat(),))
                
                columns = [description[0] for description in cursor.description]
                return [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            return []
        
        except Exception as e:
            self.logger.error(f"Error getting historical data: {e}")
            return []
    
    def start_monitoring(self, interval: int = 5):
        """Start continuous monitoring"""
        if self.is_monitoring:
            self.logger.warning("Monitoring is already running")
            return
        
        self.is_monitoring = True
        self.monitor_thread = threading.Thread(target=self._monitoring_loop, args=(interval,))
        self.monitor_thread.daemon = True
        self.monitor_thread.start()
        
        self.logger.info(f"Started monitoring with {interval}s interval")
    
    def stop_monitoring(self):
        """Stop monitoring"""
        self.is_monitoring = False
        if self.monitor_thread:
            self.monitor_thread.join()
        
        self.logger.info("Stopped monitoring")
    
    def _monitoring_loop(self, interval: int):
        """Main monitoring loop"""
        while self.is_monitoring:
            try:
                metrics = self.collect_all_metrics()
                self.data_queue.put(metrics)
                
                # Store in database
                self.store_metrics_in_database(metrics)
                
                time.sleep(interval)
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                time.sleep(interval)
    
    def store_metrics_in_database(self, metrics: Dict[str, Any]):
        """Store metrics in SQLite database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            # Store system metrics
            if 'cpu' in metrics:
                cursor.execute('''
                    INSERT INTO system_metrics (
                        timestamp, cpu_percent, cpu_freq, load_avg, memory_percent,
                        memory_used, memory_total, disk_usage, disk_io, network_io
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ''', (
                    metrics['timestamp'],
                    metrics['cpu'].get('cpu_percent', 0),
                    metrics['cpu'].get('cpu_freq', {}).get('current', 0),
                    json.dumps(metrics['cpu'].get('load_avg', {})),
                    metrics.get('memory', {}).get('virtual_memory', {}).get('percent', 0),
                    metrics.get('memory', {}).get('virtual_memory', {}).get('used', 0),
                    metrics.get('memory', {}).get('virtual_memory', {}).get('total', 0),
                    json.dumps(metrics.get('disk', {}).get('disk_usage', {})),
                    json.dumps(metrics.get('disk', {}).get('disk_io', {})),
                    json.dumps(metrics.get('network', {}).get('network_io', {}))
                ))
            
            # Store process metrics
            if 'processes' in metrics and 'top_cpu_processes' in metrics['processes']:
                for proc in metrics['processes']['top_cpu_processes'][:5]:  # Store top 5
                    cursor.execute('''
                        INSERT INTO process_metrics (
                            timestamp, pid, name, cpu_percent, memory_percent,
                            memory_used, status, num_threads
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ''', (
                        metrics['timestamp'],
                        proc.get('pid', 0),
                        proc.get('name', ''),
                        proc.get('cpu_percent', 0),
                        proc.get('memory_percent', 0),
                        proc.get('memory_used_mb', 0),
                        proc.get('status', ''),
                        proc.get('num_threads', 0)
                    ))
            
            conn.commit()
            conn.close()
        
        except Exception as e:
            self.logger.error(f"Error storing metrics in database: {e}")
    
    def get_current_metrics(self) -> Dict[str, Any]:
        """Get current system metrics (blocking call)"""
        return self.collect_all_metrics()
    
    def export_metrics(self, output_file: str, hours: int = 24):
        """Export metrics to JSON file"""
        try:
            data = {
                'export_time': datetime.now().isoformat(),
                'period_hours': hours,
                'system_metrics': self.get_historical_data('system', hours),
                'process_metrics': self.get_historical_data('processes', hours),
                'alerts': self.get_recent_alerts(hours)
            }
            
            with open(output_file, 'w') as f:
                json.dump(data, f, indent=2, default=str)
            
            self.logger.info(f"Metrics exported to {output_file}")
        
        except Exception as e:
            self.logger.error(f"Error exporting metrics: {e}")
    
    def get_recent_alerts(self, hours: int = 24) -> List[Dict]:
        """Get recent alerts from database"""
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()
            
            start_time = datetime.now() - timedelta(hours=hours)
            cursor.execute('''
                SELECT * FROM alerts 
                WHERE timestamp > ? 
                ORDER BY timestamp DESC
            ''', (start_time.isoformat(),))
            
            columns = [description[0] for description in cursor.description]
            alerts = [dict(zip(columns, row)) for row in cursor.fetchall()]
            
            conn.close()
            return alerts
        
        except Exception as e:
            self.logger.error(f"Error getting alerts: {e}")
            return []

if __name__ == "__main__":
    # Example usage
    monitor = SystemMonitor()
    
    # Start monitoring
    monitor.start_monitoring(interval=10)
    
    try:
        # Keep running
        while True:
            time.sleep(60)
    except KeyboardInterrupt:
        print("\nStopping monitor...")
        monitor.stop_monitoring()