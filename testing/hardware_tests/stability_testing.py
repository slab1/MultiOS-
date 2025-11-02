#!/usr/bin/env python3
"""
Long-Term Stability Testing Framework
Extended testing for system stability, memory leak detection, thermal behavior over time
"""

import os
import sys
import json
import time
import threading
import logging
import subprocess
import statistics
import psutil
import sqlite3
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend
import matplotlib.pyplot as plt
from datetime import datetime, timedelta

class StabilityStatus(Enum):
    STABLE = "stable"
    UNSTABLE = "unstable"
    DEGRADING = "degrading"
    FAILED = "failed"
    CRITICAL = "critical"

@dataclass
class StabilityMetric:
    """Individual stability metric"""
    timestamp: float
    metric_name: str
    value: float
    threshold_warning: float
    threshold_critical: float
    unit: str

@dataclass
class StabilityTestResult:
    """Result of a stability test"""
    test_name: str
    start_time: float
    end_time: float
    duration_hours: float
    status: StabilityStatus
    metrics_collected: List[StabilityMetric]
    anomalies_detected: List[Dict[str, Any]]
    summary: Dict[str, Any]

class LongTermStabilityTester:
    """Main class for long-term stability testing"""
    
    def __init__(self, test_duration_hours: int = 24):
        self.test_duration_hours = test_duration_hours
        self.logger = self._setup_logging()
        self.db_path = "/workspace/testing/hardware_tests/results/stability_metrics.db"
        self.metrics_history = []
        self.is_running = False
        self.stop_event = threading.Event()
        self.test_start_time = None
        self.anomaly_thresholds = self._load_anomaly_thresholds()
        
    def _setup_logging(self):
        """Setup logging for stability testing"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/stability_test.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def _load_anomaly_thresholds(self) -> Dict[str, Dict[str, float]]:
        """Load anomaly detection thresholds"""
        return {
            'cpu_usage': {'warning': 95.0, 'critical': 99.0},
            'memory_usage': {'warning': 90.0, 'critical': 95.0},
            'disk_usage': {'warning': 85.0, 'critical': 95.0},
            'temperature': {'warning': 80.0, 'critical': 90.0},
            'cpu_frequency': {'warning_low': 0.5, 'warning_high': 5.0},  # Relative to max
            'load_average': {'warning': 2.0, 'critical': 4.0},
            'network_errors': {'warning': 10, 'critical': 100},
            'process_count': {'warning': 1000, 'critical': 2000}
        }
    
    def _init_database(self):
        """Initialize SQLite database for metrics storage"""
        os.makedirs(os.path.dirname(self.db_path), exist_ok=True)
        
        with sqlite3.connect(self.db_path) as conn:
            conn.execute('''
                CREATE TABLE IF NOT EXISTS metrics (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp REAL NOT NULL,
                    metric_name TEXT NOT NULL,
                    value REAL NOT NULL,
                    threshold_warning REAL,
                    threshold_critical REAL,
                    unit TEXT,
                    UNIQUE(timestamp, metric_name)
                )
            ''')
            
            conn.execute('''
                CREATE TABLE IF NOT EXISTS anomalies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp REAL NOT NULL,
                    metric_name TEXT NOT NULL,
                    value REAL NOT NULL,
                    anomaly_type TEXT NOT NULL,
                    severity TEXT NOT NULL,
                    description TEXT
                )
            ''')
            
            conn.commit()
    
    def collect_system_metrics(self) -> List[StabilityMetric]:
        """Collect comprehensive system metrics"""
        metrics = []
        current_time = time.time()
        
        try:
            # CPU metrics
            cpu_usage = psutil.cpu_percent(interval=1)
            cpu_freq = psutil.cpu_freq()
            
            metrics.append(StabilityMetric(
                timestamp=current_time,
                metric_name='cpu_usage_percent',
                value=cpu_usage,
                threshold_warning=self.anomaly_thresholds['cpu_usage']['warning'],
                threshold_critical=self.anomaly_thresholds['cpu_usage']['critical'],
                unit='percent'
            ))
            
            if cpu_freq:
                metrics.append(StabilityMetric(
                    timestamp=current_time,
                    metric_name='cpu_frequency_ghz',
                    value=cpu_freq.current / 1000.0,
                    threshold_warning=self.anomaly_thresholds['cpu_frequency']['warning_low'],
                    threshold_critical=self.anomaly_thresholds['cpu_frequency']['warning_high'],
                    unit='ghz'
                ))
            
            # Memory metrics
            memory = psutil.virtual_memory()
            metrics.append(StabilityMetric(
                timestamp=current_time,
                metric_name='memory_usage_percent',
                value=memory.percent,
                threshold_warning=self.anomaly_thresholds['memory_usage']['warning'],
                threshold_critical=self.anomaly_thresholds['memory_usage']['critical'],
                unit='percent'
            ))
            
            metrics.append(StabilityMetric(
                timestamp=current_time,
                metric_name='memory_available_gb',
                value=memory.available / (1024**3),
                threshold_warning=1.0,  # 1GB minimum
                threshold_critical=0.5,  # 500MB minimum
                unit='gb'
            ))
            
            # Disk metrics
            disk_usage = psutil.disk_usage('/')
            metrics.append(StabilityMetric(
                timestamp=current_time,
                metric_name='disk_usage_percent',
                value=(disk_usage.used / disk_usage.total) * 100,
                threshold_warning=self.anomaly_thresholds['disk_usage']['warning'],
                threshold_critical=self.anomaly_thresholds['disk_usage']['critical'],
                unit='percent'
            ))
            
            disk_io = psutil.disk_io_counters()
            if disk_io:
                metrics.append(StabilityMetric(
                    timestamp=current_time,
                    metric_name='disk_read_bytes_per_sec',
                    value=disk_io.read_bytes,
                    threshold_warning=100000000,  # 100MB/s
                    threshold_critical=200000000,  # 200MB/s
                    unit='bytes'
                ))
                
                metrics.append(StabilityMetric(
                    timestamp=current_time,
                    metric_name='disk_write_bytes_per_sec',
                    value=disk_io.write_bytes,
                    threshold_warning=50000000,  # 50MB/s
                    threshold_critical=100000000,  # 100MB/s
                    unit='bytes'
                ))
            
            # Network metrics
            network_io = psutil.net_io_counters()
            if network_io:
                metrics.append(StabilityMetric(
                    timestamp=current_time,
                    metric_name='network_bytes_sent_per_sec',
                    value=network_io.bytes_sent,
                    threshold_warning=10000000,  # 10MB/s
                    threshold_critical=50000000,  # 50MB/s
                    unit='bytes'
                ))
                
                metrics.append(StabilityMetric(
                    timestamp=current_time,
                    metric_name='network_bytes_recv_per_sec',
                    value=network_io.bytes_recv,
                    threshold_warning=10000000,  # 10MB/s
                    threshold_critical=50000000,  # 50MB/s
                    unit='bytes'
                ))
                
                # Error rates
                if network_io.errin > 0 or network_io.errout > 0:
                    metrics.append(StabilityMetric(
                        timestamp=current_time,
                        metric_name='network_errors_total',
                        value=network_io.errin + network_io.errout,
                        threshold_warning=self.anomaly_thresholds['network_errors']['warning'],
                        threshold_critical=self.anomaly_thresholds['network_errors']['critical'],
                        unit='count'
                    ))
            
            # Load average
            if hasattr(os, 'getloadavg'):
                load_avg = os.getloadavg()
                metrics.append(StabilityMetric(
                    timestamp=current_time,
                    metric_name='load_average_1min',
                    value=load_avg[0],
                    threshold_warning=self.anomaly_thresholds['load_average']['warning'],
                    threshold_critical=self.anomaly_thresholds['load_average']['critical'],
                    unit='load'
                ))
            
            # Process count
            process_count = len(psutil.pids())
            metrics.append(StabilityMetric(
                timestamp=current_time,
                metric_name='process_count',
                value=process_count,
                threshold_warning=self.anomaly_thresholds['process_count']['warning'],
                threshold_critical=self.anomaly_thresholds['process_count']['critical'],
                unit='count'
            ))
            
            # Temperature metrics
            temps = self._get_temperatures()
            for temp_name, temp_value in temps.items():
                if temp_value > 0:  # Only record valid temperatures
                    metrics.append(StabilityMetric(
                        timestamp=current_time,
                        metric_name=f'temperature_{temp_name.replace(" ", "_").lower()}',
                        value=temp_value,
                        threshold_warning=self.anomaly_thresholds['temperature']['warning'],
                        threshold_critical=self.anomaly_thresholds['temperature']['critical'],
                        unit='celsius'
                    ))
            
            # Additional metrics
            metrics.extend(self._collect_additional_metrics(current_time))
            
        except Exception as e:
            self.logger.error(f"Error collecting system metrics: {e}")
        
        return metrics
    
    def _get_temperatures(self) -> Dict[str, float]:
        """Get temperature readings"""
        temps = {}
        
        try:
            # Try thermal zones
            thermal_files = glob.glob('/sys/class/thermal/thermal_zone*/temp')
            for i, temp_file in enumerate(thermal_files):
                try:
                    with open(temp_file, 'r') as f:
                        temp_millidegrees = int(f.read().strip())
                        temps[f'thermal_zone_{i}'] = temp_millidegrees / 1000.0
                except:
                    continue
        except:
            pass
        
        # Try lm-sensors
        try:
            result = subprocess.run(['sensors'], capture_output=True, text=True, timeout=5)
            if result.returncode == 0:
                # Parse lm-sensors output
                import re
                
                # Extract CPU temperatures
                core_temps = re.findall(r'Core \d+:\s*\+([\d.]+)°C', result.stdout)
                for i, temp in enumerate(core_temps):
                    temps[f'core_{i}'] = float(temp)
                
                # Extract other temperatures
                other_temps = re.findall(r'(\w+.*?):\s*\+?([\d.]+)°C', result.stdout)
                for name, temp in other_temps:
                    if 'Core' not in name and 'temp' not in name.lower():
                        temps[name.strip().replace(' ', '_').lower()] = float(temp)
        except:
            pass
        
        return temps
    
    def _collect_additional_metrics(self, timestamp: float) -> List[StabilityMetric]:
        """Collect additional stability metrics"""
        additional_metrics = []
        
        try:
            # Process information
            processes = []
            for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent']):
                try:
                    processes.append(proc.info)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            
            # Find top processes by CPU and memory
            if processes:
                top_cpu_process = max(processes, key=lambda p: p['cpu_percent'] or 0)
                top_mem_process = max(processes, key=lambda p: p['memory_percent'] or 0)
                
                if top_cpu_process['cpu_percent']:
                    additional_metrics.append(StabilityMetric(
                        timestamp=timestamp,
                        metric_name='top_process_cpu_percent',
                        value=top_cpu_process['cpu_percent'],
                        threshold_warning=50.0,
                        threshold_critical=80.0,
                        unit='percent'
                    ))
                
                if top_mem_process['memory_percent']:
                    additional_metrics.append(StabilityMetric(
                        timestamp=timestamp,
                        metric_name='top_process_memory_percent',
                        value=top_mem_process['memory_percent'],
                        threshold_warning=20.0,
                        threshold_critical=50.0,
                        unit='percent'
                    ))
            
            # Swap usage
            swap = psutil.swap_memory()
            if swap.total > 0:
                additional_metrics.append(StabilityMetric(
                    timestamp=timestamp,
                    metric_name='swap_usage_percent',
                    value=swap.percent,
                    threshold_warning=20.0,
                    threshold_critical=50.0,
                    unit='percent'
                ))
                
                additional_metrics.append(StabilityMetric(
                    timestamp=timestamp,
                    metric_name='swap_available_gb',
                    value=swap.free / (1024**3),
                    threshold_warning=1.0,
                    threshold_critical=0.5,
                    unit='gb'
                ))
        
        except Exception as e:
            self.logger.error(f"Error collecting additional metrics: {e}")
        
        return additional_metrics
    
    def detect_anomalies(self, metrics: List[StabilityMetric]) -> List[Dict[str, Any]]:
        """Detect anomalies in collected metrics"""
        anomalies = []
        
        for metric in metrics:
            current_time = time.time()
            
            # Check for threshold violations
            if metric.value >= metric.threshold_critical:
                anomaly = {
                    'timestamp': current_time,
                    'metric_name': metric.metric_name,
                    'value': metric.value,
                    'anomaly_type': 'threshold_critical',
                    'severity': 'critical',
                    'description': f'Critical threshold exceeded: {metric.value} {metric.unit} (threshold: {metric.threshold_critical} {metric.unit})'
                }
                anomalies.append(anomaly)
                self.logger.critical(f"CRITICAL: {anomaly['description']}")
            
            elif metric.value >= metric.threshold_warning:
                anomaly = {
                    'timestamp': current_time,
                    'metric_name': metric.metric_name,
                    'value': metric.value,
                    'anomaly_type': 'threshold_warning',
                    'severity': 'warning',
                    'description': f'Warning threshold exceeded: {metric.value} {metric.unit} (threshold: {metric.threshold_warning} {metric.unit})'
                }
                anomalies.append(anomaly)
                self.logger.warning(f"WARNING: {anomaly['description']}")
            
            # Check for trends (if we have history)
            historical_values = [m.value for m in self.metrics_history if m.metric_name == metric.metric_name]
            if len(historical_values) >= 10:  # Need at least 10 previous values
                trend_analysis = self._analyze_trend(historical_values + [metric.value])
                
                if trend_analysis['is_degrading']:
                    anomaly = {
                        'timestamp': current_time,
                        'metric_name': metric.metric_name,
                        'value': metric.value,
                        'anomaly_type': 'trend_degradation',
                        'severity': 'warning',
                        'description': f'Performance degradation detected: {metric.metric_name} trending upward over time'
                    }
                    anomalies.append(anomaly)
                    self.logger.warning(f"DEGRADING: {anomaly['description']}")
        
        return anomalies
    
    def _analyze_trend(self, values: List[float]) -> Dict[str, Any]:
        """Analyze trend in metric values"""
        if len(values) < 5:
            return {'is_degrading': False, 'trend_slope': 0}
        
        # Calculate trend using linear regression
        n = len(values)
        x = list(range(n))
        
        # Simple linear regression
        x_mean = sum(x) / n
        y_mean = sum(values) / n
        
        numerator = sum((x[i] - x_mean) * (values[i] - y_mean) for i in range(n))
        denominator = sum((x[i] - x_mean) ** 2 for i in range(n))
        
        if denominator == 0:
            return {'is_degrading': False, 'trend_slope': 0}
        
        slope = numerator / denominator
        
        # Consider it degrading if the slope is significantly positive
        # and recent values are significantly higher than early values
        early_values = values[:n//3]
        late_values = values[-n//3:]
        
        early_avg = statistics.mean(early_values)
        late_avg = statistics.mean(late_values)
        
        degradation_threshold = early_avg * 0.1  # 10% increase
        
        is_degrading = (slope > 0.01 and 
                       (late_avg - early_avg) > degradation_threshold and
                       n >= 20)
        
        return {
            'is_degrading': is_degrading,
            'trend_slope': slope,
            'early_average': early_avg,
            'late_average': late_avg
        }
    
    def store_metrics(self, metrics: List[StabilityMetric]):
        """Store metrics in database"""
        try:
            with sqlite3.connect(self.db_path) as conn:
                for metric in metrics:
                    conn.execute('''
                        INSERT OR REPLACE INTO metrics 
                        (timestamp, metric_name, value, threshold_warning, threshold_critical, unit)
                        VALUES (?, ?, ?, ?, ?, ?)
                    ''', (
                        metric.timestamp,
                        metric.metric_name,
                        metric.value,
                        metric.threshold_warning,
                        metric.threshold_critical,
                        metric.unit
                    ))
                
                conn.commit()
        except Exception as e:
            self.logger.error(f"Error storing metrics: {e}")
    
    def store_anomalies(self, anomalies: List[Dict[str, Any]]):
        """Store anomalies in database"""
        try:
            with sqlite3.connect(self.db_path) as conn:
                for anomaly in anomalies:
                    conn.execute('''
                        INSERT INTO anomalies 
                        (timestamp, metric_name, value, anomaly_type, severity, description)
                        VALUES (?, ?, ?, ?, ?, ?)
                    ''', (
                        anomaly['timestamp'],
                        anomaly['metric_name'],
                        anomaly['value'],
                        anomaly['anomaly_type'],
                        anomaly['severity'],
                        anomaly['description']
                    ))
                
                conn.commit()
        except Exception as e:
            self.logger.error(f"Error storing anomalies: {e}")
    
    def run_memory_leak_test(self, duration_hours: int = 24) -> Dict[str, Any]:
        """Run memory leak detection test"""
        self.logger.info(f"Starting memory leak test for {duration_hours} hours")
        
        self.test_start_time = time.time()
        end_time = self.test_start_time + (duration_hours * 3600)
        
        memory_measurements = []
        processes_to_monitor = []
        
        # Start by collecting initial process list
        try:
            for proc in psutil.process_iter(['pid', 'name', 'memory_percent']):
                try:
                    processes_to_monitor.append({
                        'pid': proc.info['pid'],
                        'name': proc.info['name'],
                        'initial_memory_percent': proc.info['memory_percent'] or 0
                    })
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
        except:
            pass
        
        # Monitor loop
        while time.time() < end_time:
            try:
                # Collect memory metrics
                memory = psutil.virtual_memory()
                memory_measurements.append({
                    'timestamp': time.time(),
                    'total_gb': memory.total / (1024**3),
                    'available_gb': memory.available / (1024**3),
                    'used_percent': memory.percent,
                    'cached_gb': getattr(memory, 'cached', 0) / (1024**3) if hasattr(memory, 'cached') else 0
                })
                
                # Check for memory leaks in individual processes
                current_processes = []
                process_memory_changes = []
                
                for proc in psutil.process_iter(['pid', 'name', 'memory_percent']):
                    try:
                        current_processes.append({
                            'pid': proc.info['pid'],
                            'name': proc.info['name'],
                            'current_memory_percent': proc.info['memory_percent'] or 0
                        })
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        continue
                
                # Find processes with significant memory increase
                for current_proc in current_processes:
                    initial_proc = next((p for p in processes_to_monitor if p['pid'] == current_proc['pid']), None)
                    if initial_proc:
                        memory_increase = current_proc['current_memory_percent'] - initial_proc['initial_memory_percent']
                        if memory_increase > 5.0:  # 5% increase
                            process_memory_changes.append({
                                'pid': current_proc['pid'],
                                'name': current_proc['name'],
                                'initial_memory': initial_proc['initial_memory_percent'],
                                'current_memory': current_proc['current_memory_percent'],
                                'increase': memory_increase
                            })
                
                if process_memory_changes:
                    self.logger.warning(f"Potential memory leaks detected in {len(process_memory_changes)} processes")
                
                # Check overall system memory
                if memory.percent > 90:
                    self.logger.critical(f"High memory usage: {memory.percent}%")
                
                time.sleep(60)  # Collect every minute
                
            except Exception as e:
                self.logger.error(f"Error in memory leak test: {e}")
                time.sleep(60)
        
        # Analyze results
        if memory_measurements:
            memory_usage_values = [m['used_percent'] for m in memory_measurements]
            
            analysis = {
                'test_duration_hours': duration_hours,
                'measurements_count': len(memory_measurements),
                'memory_statistics': {
                    'min_usage_percent': min(memory_usage_values),
                    'max_usage_percent': max(memory_usage_values),
                    'avg_usage_percent': statistics.mean(memory_usage_values),
                    'std_dev_percent': statistics.stdev(memory_usage_values) if len(memory_usage_values) > 1 else 0
                },
                'potential_memory_leaks': process_memory_changes,
                'memory_trend': self._analyze_trend(memory_usage_values)
            }
            
            # Determine status
            if analysis['memory_statistics']['max_usage_percent'] > 95:
                status = StabilityStatus.CRITICAL
            elif analysis['memory_statistics']['max_usage_percent'] > 85:
                status = StabilityStatus.UNSTABLE
            elif analysis['memory_trend']['is_degrading']:
                status = StabilityStatus.DEGRADING
            else:
                status = StabilityStatus.STABLE
        else:
            analysis = {'error': 'No measurements collected'}
            status = StabilityStatus.FAILED
        
        return {
            'test_name': 'memory_leak_test',
            'status': status,
            'analysis': analysis,
            'measurements': memory_measurements
        }
    
    def run_thermal_stability_test(self, duration_hours: int = 12) -> Dict[str, Any]:
        """Run thermal stability test"""
        self.logger.info(f"Starting thermal stability test for {duration_hours} hours")
        
        self.test_start_time = time.time()
        end_time = self.test_start_time + (duration_hours * 3600)
        
        thermal_measurements = []
        
        # Load generator to stress CPU
        stress_load = threading.Thread(target=self._generate_cpu_load, args=(end_time,))
        stress_load.start()
        
        # Monitor loop
        while time.time() < end_time:
            try:
                # Collect temperature data
                temps = self._get_temperatures()
                
                measurement = {
                    'timestamp': time.time(),
                    'temperatures': temps,
                    'cpu_usage': psutil.cpu_percent(interval=1)
                }
                thermal_measurements.append(measurement)
                
                # Check for thermal throttling
                max_temp = max(temps.values()) if temps else 0
                if max_temp > 85:
                    self.logger.warning(f"High temperature detected: {max_temp}°C")
                
                time.sleep(30)  # Collect every 30 seconds
                
            except Exception as e:
                self.logger.error(f"Error in thermal stability test: {e}")
                time.sleep(30)
        
        # Stop stress load
        self.stop_event.set()
        stress_load.join()
        
        # Analyze results
        if thermal_measurements:
            all_temps = []
            for measurement in thermal_measurements:
                all_temps.extend(measurement['temperatures'].values())
            
            if all_temps:
                analysis = {
                    'test_duration_hours': duration_hours,
                    'measurements_count': len(thermal_measurements),
                    'temperature_statistics': {
                        'min_temp_c': min(all_temps),
                        'max_temp_c': max(all_temps),
                        'avg_temp_c': statistics.mean(all_temps),
                        'std_dev_c': statistics.stdev(all_temps) if len(all_temps) > 1 else 0
                    },
                    'thermal_events': {
                        'high_temp_warnings': len([m for m in thermal_measurements 
                                                 if max(m['temperatures'].values()) > 80 
                                                 if m['temperatures']]),
                        'critical_temps': len([m for m in thermal_measurements 
                                             if max(m['temperatures'].values()) > 90 
                                             if m['temperatures']])
                    }
                }
                
                # Determine status
                if analysis['temperature_statistics']['max_temp_c'] > 95:
                    status = StabilityStatus.CRITICAL
                elif analysis['temperature_statistics']['max_temp_c'] > 85:
                    status = StabilityStatus.UNSTABLE
                elif analysis['temperature_statistics']['max_temp_c'] > 75:
                    status = StabilityStatus.DEGRADING
                else:
                    status = StabilityStatus.STABLE
            else:
                analysis = {'error': 'No temperature data collected'}
                status = StabilityStatus.FAILED
        else:
            analysis = {'error': 'No measurements collected'}
            status = StabilityStatus.FAILED
        
        return {
            'test_name': 'thermal_stability_test',
            'status': status,
            'analysis': analysis,
            'measurements': thermal_measurements
        }
    
    def _generate_cpu_load(self, end_time: float):
        """Generate CPU load for thermal testing"""
        while time.time() < end_time and not self.stop_event.is_set():
            # CPU-intensive calculation
            sum(range(10000))
    
    def run_disk_stress_test(self, duration_hours: int = 8) -> Dict[str, Any]:
        """Run disk stress stability test"""
        self.logger.info(f"Starting disk stress test for {duration_hours} hours")
        
        self.test_start_time = time.time()
        end_time = self.test_start_time + (duration_hours * 3600)
        
        disk_measurements = []
        test_file = '/tmp/stability_test_file.tmp'
        
        # Disk stress workload
        def disk_workload():
            try:
                while time.time() < end_time and not self.stop_event.is_set():
                    # Write stress
                    with open(test_file, 'w') as f:
                        f.write('x' * (1024 * 1024))  # 1MB writes
                    
                    # Read stress
                    with open(test_file, 'r') as f:
                        f.read()
                    
                    time.sleep(0.1)
            except Exception as e:
                self.logger.error(f"Disk workload error: {e}")
            finally:
                try:
                    os.remove(test_file)
                except:
                    pass
        
        # Start disk workload
        disk_thread = threading.Thread(target=disk_workload)
        disk_thread.start()
        
        # Monitor loop
        while time.time() < end_time:
            try:
                # Collect disk metrics
                disk_usage = psutil.disk_usage('/')
                disk_io = psutil.disk_io_counters()
                
                measurement = {
                    'timestamp': time.time(),
                    'disk_usage_percent': (disk_usage.used / disk_usage.total) * 100,
                    'disk_free_gb': disk_usage.free / (1024**3),
                    'disk_read_bytes': disk_io.read_bytes if disk_io else 0,
                    'disk_write_bytes': disk_io.write_bytes if disk_io else 0
                }
                disk_measurements.append(measurement)
                
                # Check for disk issues
                if disk_usage.used / disk_usage.total > 0.95:
                    self.logger.critical("Disk space critically low")
                
                time.sleep(60)  # Collect every minute
                
            except Exception as e:
                self.logger.error(f"Error in disk stress test: {e}")
                time.sleep(60)
        
        # Stop disk workload
        self.stop_event.set()
        disk_thread.join()
        
        # Analyze results
        if disk_measurements:
            usage_values = [m['disk_usage_percent'] for m in disk_measurements]
            
            analysis = {
                'test_duration_hours': duration_hours,
                'measurements_count': len(disk_measurements),
                'disk_statistics': {
                    'min_usage_percent': min(usage_values),
                    'max_usage_percent': max(usage_values),
                    'avg_usage_percent': statistics.mean(usage_values)
                }
            }
            
            # Determine status
            if analysis['disk_statistics']['max_usage_percent'] > 95:
                status = StabilityStatus.CRITICAL
            elif analysis['disk_statistics']['max_usage_percent'] > 85:
                status = StabilityStatus.UNSTABLE
            else:
                status = StabilityStatus.STABLE
        else:
            analysis = {'error': 'No measurements collected'}
            status = StabilityStatus.FAILED
        
        return {
            'test_name': 'disk_stress_test',
            'status': status,
            'analysis': analysis,
            'measurements': disk_measurements
        }
    
    def run_comprehensive_stability_test(self, duration_hours: int = 24) -> StabilityTestResult:
        """Run comprehensive stability test"""
        self.logger.info(f"Starting comprehensive stability test for {duration_hours} hours")
        
        self.test_start_time = time.time()
        end_time = self.test_start_time + (duration_hours * 3600)
        
        self._init_database()
        
        all_metrics = []
        all_anomalies = []
        
        # Main monitoring loop
        while time.time() < end_time:
            try:
                # Collect metrics
                metrics = self.collect_system_metrics()
                all_metrics.extend(metrics)
                self.metrics_history.extend(metrics)
                
                # Detect anomalies
                anomalies = self.detect_anomalies(metrics)
                all_anomalies.extend(anomalies)
                
                # Store in database
                self.store_metrics(metrics)
                self.store_anomalies(anomalies)
                
                self.logger.info(f"Collected {len(metrics)} metrics, detected {len(anomalies)} anomalies")
                
                # Check for critical anomalies
                critical_anomalies = [a for a in anomalies if a['severity'] == 'critical']
                if critical_anomalies:
                    self.logger.critical(f"Critical anomaly detected: {critical_anomalies[0]['description']}")
                
                time.sleep(300)  # Collect every 5 minutes
                
            except Exception as e:
                self.logger.error(f"Error in comprehensive stability test: {e}")
                time.sleep(300)
        
        end_time_actual = time.time()
        duration_hours_actual = (end_time_actual - self.test_start_time) / 3600
        
        # Determine overall status
        if any(a['severity'] == 'critical' for a in all_anomalies):
            status = StabilityStatus.CRITICAL
        elif len([a for a in all_anomalies if a['severity'] == 'warning']) > 50:
            status = StabilityStatus.UNSTABLE
        elif len(all_anomalies) > 20:
            status = StabilityStatus.DEGRADING
        else:
            status = StabilityStatus.STABLE
        
        # Generate summary
        metric_summary = self._generate_metric_summary(all_metrics)
        
        return StabilityTestResult(
            test_name='comprehensive_stability_test',
            start_time=self.test_start_time,
            end_time=end_time_actual,
            duration_hours=duration_hours_actual,
            status=status,
            metrics_collected=all_metrics,
            anomalies_detected=all_anomalies,
            summary={
                'total_metrics_collected': len(all_metrics),
                'total_anomalies_detected': len(all_anomalies),
                'critical_anomalies': len([a for a in all_anomalies if a['severity'] == 'critical']),
                'warning_anomalies': len([a for a in all_anomalies if a['severity'] == 'warning']),
                'metric_summary': metric_summary
            }
        )
    
    def _generate_metric_summary(self, metrics: List[StabilityMetric]) -> Dict[str, Any]:
        """Generate summary of collected metrics"""
        summary = {}
        
        # Group metrics by name
        metric_groups = {}
        for metric in metrics:
            if metric.metric_name not in metric_groups:
                metric_groups[metric.metric_name] = []
            metric_groups[metric.metric_name].append(metric)
        
        # Calculate summary for each metric
        for metric_name, metric_list in metric_groups.items():
            values = [m.value for m in metric_list]
            
            summary[metric_name] = {
                'count': len(values),
                'min': min(values),
                'max': max(values),
                'avg': statistics.mean(values),
                'std_dev': statistics.stdev(values) if len(values) > 1 else 0,
                'unit': metric_list[0].unit if metric_list else ''
            }
        
        return summary
    
    def generate_stability_plots(self, test_result: StabilityTestResult) -> List[str]:
        """Generate stability analysis plots"""
        plot_paths = []
        
        try:
            # Plot CPU usage over time
            cpu_metrics = [m for m in test_result.metrics_collected if m.metric_name == 'cpu_usage_percent']
            if cpu_metrics:
                timestamps = [m.timestamp for m in cpu_metrics]
                values = [m.value for m in cpu_metrics]
                
                plt.figure(figsize=(12, 6))
                plt.plot(timestamps, values)
                plt.title('CPU Usage Over Time')
                plt.xlabel('Time')
                plt.ylabel('CPU Usage (%)')
                plt.grid(True)
                
                plot_path = f"/workspace/testing/hardware_tests/results/cpu_usage_stability_{int(time.time())}.png"
                plt.savefig(plot_path)
                plt.close()
                plot_paths.append(plot_path)
            
            # Plot memory usage over time
            memory_metrics = [m for m in test_result.metrics_collected if m.metric_name == 'memory_usage_percent']
            if memory_metrics:
                timestamps = [m.timestamp for m in memory_metrics]
                values = [m.value for m in memory_metrics]
                
                plt.figure(figsize=(12, 6))
                plt.plot(timestamps, values)
                plt.title('Memory Usage Over Time')
                plt.xlabel('Time')
                plt.ylabel('Memory Usage (%)')
                plt.grid(True)
                
                plot_path = f"/workspace/testing/hardware_tests/results/memory_usage_stability_{int(time.time())}.png"
                plt.savefig(plot_path)
                plt.close()
                plot_paths.append(plot_path)
            
            # Plot temperature over time
            temp_metrics = [m for m in test_result.metrics_collected if m.metric_name.startswith('temperature_')]
            if temp_metrics:
                timestamps = [m.timestamp for m in temp_metrics]
                values = [m.value for m in temp_metrics]
                
                plt.figure(figsize=(12, 6))
                plt.plot(timestamps, values)
                plt.title('Temperature Over Time')
                plt.xlabel('Time')
                plt.ylabel('Temperature (°C)')
                plt.grid(True)
                
                plot_path = f"/workspace/testing/hardware_tests/results/temperature_stability_{int(time.time())}.png"
                plt.savefig(plot_path)
                plt.close()
                plot_paths.append(plot_path)
            
            # Plot anomalies over time
            if test_result.anomalies_detected:
                timestamps = [a['timestamp'] for a in test_result.anomalies_detected]
                critical_times = [t for t, a in zip(timestamps, test_result.anomalies_detected) if a['severity'] == 'critical']
                warning_times = [t for t, a in zip(timestamps, test_result.anomalies_detected) if a['severity'] == 'warning']
                
                plt.figure(figsize=(12, 6))
                if critical_times:
                    plt.scatter(critical_times, [1] * len(critical_times), color='red', label='Critical', s=50)
                if warning_times:
                    plt.scatter(warning_times, [0.5] * len(warning_times), color='orange', label='Warning', s=50)
                plt.title('Detected Anomalies Over Time')
                plt.xlabel('Time')
                plt.ylabel('Severity')
                plt.legend()
                plt.grid(True)
                
                plot_path = f"/workspace/testing/hardware_tests/results/anomalies_timeline_{int(time.time())}.png"
                plt.savefig(plot_path)
                plt.close()
                plot_paths.append(plot_path)
                
        except Exception as e:
            self.logger.error(f"Error generating stability plots: {e}")
        
        return plot_paths
    
    def generate_stability_report(self, test_result: StabilityTestResult) -> str:
        """Generate comprehensive stability test report"""
        
        # Generate plots
        plot_paths = self.generate_stability_plots(test_result)
        
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'test_suite_version': '1.0',
                'stability_status': test_result.status.value
            },
            'test_summary': {
                'test_name': test_result.test_name,
                'duration_hours': test_result.duration_hours,
                'status': test_result.status.value,
                'start_time': test_result.start_time,
                'end_time': test_result.end_time
            },
            'metrics_analysis': test_result.summary['metric_summary'],
            'anomalies_summary': {
                'total_anomalies': test_result.summary['total_anomalies_detected'],
                'critical_anomalies': test_result.summary['critical_anomalies'],
                'warning_anomalies': test_result.summary['warning_anomalies']
            },
            'detailed_anomalies': test_result.anomalies_detected,
            'generated_plots': plot_paths,
            'recommendations': self._generate_stability_recommendations(test_result)
        }
        
        report_path = f"/workspace/testing/hardware_tests/results/stability_test_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Stability test report generated: {report_path}")
        return report_path
    
    def _generate_stability_recommendations(self, test_result: StabilityTestResult) -> Dict[str, List[str]]:
        """Generate stability recommendations"""
        recommendations = {
            'immediate_actions': [],
            'optimizations': [],
            'hardware_upgrades': [],
            'monitoring_suggestions': []
        }
        
        # Analyze anomalies for recommendations
        critical_anomalies = [a for a in test_result.anomalies_detected if a['severity'] == 'critical']
        warning_anomalies = [a for a in test_result.anomalies_detected if a['severity'] == 'warning']
        
        # CPU-related recommendations
        cpu_anomalies = [a for a in critical_anomalies + warning_anomalies if 'cpu' in a['metric_name']]
        if cpu_anomalies:
            recommendations['immediate_actions'].append("Investigate CPU performance issues")
            recommendations['optimizations'].append("Check for CPU-intensive processes")
        
        # Memory-related recommendations
        memory_anomalies = [a for a in critical_anomalies + warning_anomalies if 'memory' in a['metric_name']]
        if memory_anomalies:
            recommendations['immediate_actions'].append("Investigate memory usage and potential leaks")
            recommendations['hardware_upgrades'].append("Consider adding more system memory")
        
        # Temperature-related recommendations
        temp_anomalies = [a for a in critical_anomalies + warning_anomalies if 'temperature' in a['metric_name']]
        if temp_anomalies:
            recommendations['immediate_actions'].append("Check thermal management and cooling")
            recommendations['hardware_upgrades'].append("Consider better cooling solutions")
        
        # Overall status recommendations
        if test_result.status == StabilityStatus.CRITICAL:
            recommendations['immediate_actions'].extend([
                "System shows critical stability issues",
                "Immediate intervention required",
                "Consider reducing workload or hardware inspection"
            ])
        elif test_result.status == StabilityStatus.UNSTABLE:
            recommendations['immediate_actions'].append("System shows instability - investigate further")
            recommendations['monitoring_suggestions'].append("Increase monitoring frequency")
        
        return recommendations


def main():
    """Main function for standalone execution"""
    import argparse
    import glob
    
    parser = argparse.ArgumentParser(description='Long-Term Stability Testing')
    parser.add_argument('--test', choices=['memory_leak', 'thermal', 'disk', 'comprehensive', 'all'],
                       default='all', help='Test type to run')
    parser.add_argument('--duration', type=int, default=24, 
                       help='Test duration in hours')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    tester = LongTermStabilityTester(args.duration)
    
    results = {}
    
    if args.test in ['memory_leak', 'all']:
        print(f"Running memory leak test for {args.duration} hours...")
        results['memory_leak'] = tester.run_memory_leak_test(args.duration)
    
    if args.test in ['thermal', 'all']:
        print(f"Running thermal stability test for {args.duration//2} hours...")
        results['thermal'] = tester.run_thermal_stability_test(args.duration//2)
    
    if args.test in ['disk', 'all']:
        print(f"Running disk stress test for {args.duration//3} hours...")
        results['disk'] = tester.run_disk_stress_test(args.duration//3)
    
    if args.test in ['comprehensive', 'all']:
        print(f"Running comprehensive stability test for {args.duration} hours...")
        comprehensive_result = tester.run_comprehensive_stability_test(args.duration)
        results['comprehensive'] = comprehensive_result
        
        # Generate report
        report_path = tester.generate_stability_report(comprehensive_result)
        print(f"Stability test report generated: {report_path}")
    
    # Print summary
    print("\nStability Test Summary:")
    for test_name, result in results.items():
        if hasattr(result, 'status'):
            print(f"  {test_name}: {result.status.value}")
        else:
            print(f"  {test_name}: {result['status'].value if 'status' in result else 'completed'}")


if __name__ == "__main__":
    main()