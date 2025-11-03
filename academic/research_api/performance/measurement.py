"""
Performance Measurement System

Advanced performance measurement and profiling utilities for OS research.
Provides detailed metrics collection, profiling, and analysis capabilities.
"""

import os
import time
import json
import psutil
import threading
import asyncio
from typing import Dict, List, Any, Optional, Callable, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import logging
import subprocess
import statistics
import numpy as np
from collections import defaultdict, deque
import cProfile
import pstats
import io
import tracemalloc

from .config import ResearchConfig


@dataclass
class MeasurementPoint:
    """Single measurement data point."""
    timestamp: datetime
    metric_name: str
    value: float
    unit: str
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'timestamp': self.timestamp.isoformat(),
            'metric_name': self.metric_name,
            'value': self.value,
            'unit': self.unit,
            'metadata': self.metadata
        }


@dataclass
class ProfilingSession:
    """Profiling session data."""
    session_id: str
    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None
    profile_data: Dict[str, Any] = None
    memory_samples: List[Dict[str, Any]] = None
    performance_data: List[MeasurementPoint] = None
    status: str = 'active'  # 'active', 'completed', 'stopped'
    
    def __post_init__(self):
        if self.profile_data is None:
            self.profile_data = {}
        if self.memory_samples is None:
            self.memory_samples = []
        if self.performance_data is None:
            self.performance_data = []
    
    def add_measurement(self, measurement: MeasurementPoint):
        """Add a measurement point."""
        self.performance_data.append(measurement)
    
    def add_memory_sample(self, current: int, peak: int, timestamp: Optional[datetime] = None):
        """Add memory profiling sample."""
        sample = {
            'timestamp': (timestamp or datetime.now()).isoformat(),
            'current_bytes': current,
            'peak_bytes': peak,
            'current_mb': current / (1024 * 1024),
            'peak_mb': peak / (1024 * 1024)
        }
        self.memory_samples.append(sample)
    
    def complete(self):
        """Mark session as completed."""
        self.end_time = datetime.now()
        self.duration = (self.end_time - self.start_time).total_seconds()
        self.status = 'completed'
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'session_id': self.session_id,
            'start_time': self.start_time.isoformat(),
            'end_time': self.end_time.isoformat() if self.end_time else None,
            'duration': self.duration,
            'profile_data': self.profile_data,
            'memory_samples': self.memory_samples,
            'performance_data': [m.to_dict() for m in self.performance_data],
            'status': self.status
        }


class PerformanceProfiler:
    """Advanced performance profiling and measurement."""
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize performance profiler.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Active profiling sessions
        self.active_sessions: Dict[str, ProfilingSession] = {}
        
        # Measurement history
        self.measurement_history = deque(maxlen=10000)
        
        # System metrics cache
        self.metrics_cache = {}
        self.last_cache_update = {}
        
        # Performance thresholds for alerting
        self.thresholds = {
            'cpu_usage': 80.0,  # %
            'memory_usage': 85.0,  # %
            'disk_usage': 90.0,  # %
            'response_time': 1.0  # seconds
        }
    
    def start_profiling_session(self, session_name: str, profile_memory: bool = True) -> str:
        """
        Start a new profiling session.
        
        Args:
            session_name: Name for the profiling session
            profile_memory: Whether to profile memory usage
            
        Returns:
            Session ID
        """
        session_id = f"{session_name}_{int(time.time())}"
        
        session = ProfilingSession(
            session_id=session_id,
            start_time=datetime.now()
        )
        
        # Start memory profiling if requested
        if profile_memory:
            tracemalloc.start()
            session.add_memory_sample(0, 0)
        
        self.active_sessions[session_id] = session
        
        self.logger.info(f"Started profiling session: {session_id}")
        return session_id
    
    def stop_profiling_session(self, session_id: str) -> Optional[ProfilingSession]:
        """
        Stop a profiling session and return results.
        
        Args:
            session_id: Session ID to stop
            
        Returns:
            Profiling session data
        """
        if session_id not in self.active_sessions:
            self.logger.warning(f"Profiling session {session_id} not found")
            return None
        
        session = self.active_sessions[session_id]
        
        # Stop memory profiling
        if tracemalloc.is_tracing():
            current, peak = tracemalloc.get_traced_memory()
            session.add_memory_sample(current, peak)
            tracemalloc.stop()
        
        # Complete session
        session.complete()
        
        # Remove from active sessions
        del self.active_sessions[session_id]
        
        self.logger.info(f"Stopped profiling session: {session_id}")
        return session
    
    def profile_function(self, 
                        func: Callable,
                        session_name: str,
                        *args, 
                        **kwargs) -> Tuple[Any, ProfilingSession]:
        """
        Profile a function execution.
        
        Args:
            func: Function to profile
            session_name: Name for profiling session
            *args: Function arguments
            **kwargs: Function keyword arguments
            
        Returns:
            Tuple of (function_result, profiling_session)
        """
        session_id = self.start_profiling_session(session_name, profile_memory=True)
        
        # Setup profiling
        profiler = cProfile.Profile()
        profiler.enable()
        
        start_time = time.time()
        
        try:
            # Execute function
            result = func(*args, **kwargs)
            
            # Get memory usage
            if tracemalloc.is_tracing():
                current, peak = tracemalloc.get_traced_memory()
                session = self.active_sessions[session_id]
                session.add_memory_sample(current, peak)
            
            return result, self.stop_profiling_session(session_id)
            
        finally:
            # Stop profiling
            profiler.disable()
            
            # Get profiling statistics
            s = io.StringIO()
            ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
            ps.print_stats()
            
            profile_output = s.getvalue()
            
            # Add to session data
            session = self.active_sessions.get(session_id)
            if session:
                session.profile_data['function_profile'] = profile_output
                session.profile_data['function_name'] = func.__name__
                session.profile_data['function_args'] = str(args)[:1000]  # Limit size
                session.profile_data['function_kwargs'] = str(kwargs)[:1000]  # Limit size
            
            end_time = time.time()
            self.record_measurement('profiling_duration', end_time - start_time, 'seconds')
    
    def profile_code_block(self, 
                          code_block: Callable,
                          session_name: str,
                          iterations: int = 1) -> ProfilingSession:
        """
        Profile a code block execution.
        
        Args:
            code_block: Code block to profile (function or lambda)
            session_name: Name for profiling session
            iterations: Number of iterations
            
        Returns:
            Profiling session data
        """
        session_id = self.start_profiling_session(session_name, profile_memory=True)
        
        profiler = cProfile.Profile()
        profiler.enable()
        
        execution_times = []
        
        for i in range(iterations):
            start_time = time.time()
            
            if callable(code_block):
                code_block()
            else:
                # Execute code block directly
                exec(code_block)
            
            end_time = time.time()
            execution_times.append(end_time - start_time)
        
        profiler.disable()
        
        # Get profiling statistics
        s = io.StringIO()
        ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
        ps.print_stats()
        
        profile_output = s.getvalue()
        
        # Complete session
        session = self.stop_profiling_session(session_id)
        
        if session:
            session.profile_data['code_block_profile'] = profile_output
            session.profile_data['iterations'] = iterations
            session.profile_data['execution_times'] = execution_times
            session.profile_data['mean_time'] = statistics.mean(execution_times)
            session.profile_data['total_time'] = sum(execution_times)
            
            # Add measurements
            for exec_time in execution_times:
                session.add_measurement(MeasurementPoint(
                    timestamp=datetime.now(),
                    metric_name='code_block_execution_time',
                    value=exec_time,
                    unit='seconds'
                ))
        
        return session
    
    def collect_system_metrics(self, 
                             metrics: Optional[List[str]] = None,
                             include_process_details: bool = False) -> Dict[str, Any]:
        """
        Collect comprehensive system metrics.
        
        Args:
            metrics: Specific metrics to collect
            include_process_details: Include detailed process information
            
        Returns:
            Dictionary with system metrics
        """
        if metrics is None:
            metrics = ['cpu', 'memory', 'disk', 'network', 'processes']
        
        results = {
            'timestamp': datetime.now().isoformat(),
            'metrics': {}
        }
        
        # CPU metrics
        if 'cpu' in metrics:
            results['metrics']['cpu'] = self._collect_cpu_metrics()
        
        # Memory metrics
        if 'memory' in metrics:
            results['metrics']['memory'] = self._collect_memory_metrics()
        
        # Disk metrics
        if 'disk' in metrics:
            results['metrics']['disk'] = self._collect_disk_metrics()
        
        # Network metrics
        if 'network' in metrics:
            results['metrics']['network'] = self._collect_network_metrics()
        
        # Process metrics
        if 'processes' in metrics:
            results['metrics']['processes'] = self._collect_process_metrics(include_process_details)
        
        # System info
        results['system_info'] = self._collect_system_info()
        
        return results
    
    def _collect_cpu_metrics(self) -> Dict[str, Any]:
        """Collect CPU-related metrics."""
        return {
            'cpu_count': psutil.cpu_count(),
            'cpu_count_logical': psutil.cpu_count(logical=True),
            'cpu_usage_percent': psutil.cpu_percent(interval=1),
            'cpu_per_core': psutil.cpu_percent(interval=1, percpu=True),
            'load_average': os.getloadavg() if hasattr(os, 'getloadavg') else None,
            'cpu_freq': psutil.cpu_freq()._asdict() if psutil.cpu_freq() else None
        }
    
    def _collect_memory_metrics(self) -> Dict[str, Any]:
        """Collect memory-related metrics."""
        memory = psutil.virtual_memory()
        swap = psutil.swap_memory()
        
        return {
            'total_gb': memory.total / (1024**3),
            'available_gb': memory.available / (1024**3),
            'used_gb': memory.used / (1024**3),
            'free_gb': memory.free / (1024**3),
            'percent': memory.percent,
            'buffers_gb': getattr(memory, 'buffers', 0) / (1024**3),
            'cached_gb': getattr(memory, 'cached', 0) / (1024**3),
            'swap_total_gb': swap.total / (1024**3),
            'swap_used_gb': swap.used / (1024**3),
            'swap_percent': swap.percent
        }
    
    def _collect_disk_metrics(self) -> Dict[str, Any]:
        """Collect disk-related metrics."""
        disk_usage = psutil.disk_usage('/')
        disk_io = psutil.disk_io_counters()
        
        return {
            'total_gb': disk_usage.total / (1024**3),
            'used_gb': disk_usage.used / (1024**3),
            'free_gb': disk_usage.free / (1024**3),
            'percent': (disk_usage.used / disk_usage.total) * 100,
            'io_counters': disk_io._asdict() if disk_io else None
        }
    
    def _collect_network_metrics(self) -> Dict[str, Any]:
        """Collect network-related metrics."""
        net_io = psutil.net_io_counters()
        net_if_addrs = psutil.net_if_addrs()
        
        return {
            'io_counters': net_io._asdict() if net_io else None,
            'interfaces': list(net_if_addrs.keys()),
            'connections_count': len(psutil.net_connections())
        }
    
    def _collect_process_metrics(self, include_details: bool = False) -> Dict[str, Any]:
        """Collect process-related metrics."""
        process_count = len(psutil.pids())
        
        # Get top processes by CPU and memory
        processes = []
        for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent']):
            try:
                processes.append(proc.info)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        
        # Sort by CPU and memory usage
        processes_by_cpu = sorted(processes, key=lambda x: x.get('cpu_percent', 0), reverse=True)[:10]
        processes_by_memory = sorted(processes, key=lambda x: x.get('memory_percent', 0), reverse=True)[:10]
        
        result = {
            'total_processes': process_count,
            'top_cpu_processes': processes_by_cpu,
            'top_memory_processes': processes_by_memory
        }
        
        if include_details:
            # Get more detailed process information
            detailed_processes = []
            for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent', 
                                            'num_threads', 'create_time', 'status']):
                try:
                    proc_info = proc.info.copy()
                    detailed_processes.append(proc_info)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    pass
            
            result['all_processes'] = detailed_processes
        
        return result
    
    def _collect_system_info(self) -> Dict[str, Any]:
        """Collect general system information."""
        return {
            'platform': {
                'system': os.uname().sysname,
                'release': os.uname().release,
                'version': os.uname().version,
                'machine': os.uname().machine,
                'processor': os.uname().processor
            },
            'boot_time': datetime.fromtimestamp(psutil.boot_time()).isoformat(),
            'uptime_seconds': time.time() - psutil.boot_time()
        }
    
    def record_measurement(self, 
                         metric_name: str, 
                         value: float, 
                         unit: str,
                         metadata: Optional[Dict[str, Any]] = None):
        """
        Record a single measurement.
        
        Args:
            metric_name: Name of the metric
            value: Measurement value
            unit: Unit of measurement
            metadata: Additional metadata
        """
        measurement = MeasurementPoint(
            timestamp=datetime.now(),
            metric_name=metric_name,
            value=value,
            unit=unit,
            metadata=metadata or {}
        )
        
        self.measurement_history.append(measurement)
        self.logger.debug(f"Recorded measurement: {metric_name} = {value} {unit}")
    
    def get_measurement_history(self, 
                               metric_name: Optional[str] = None,
                               time_window: Optional[timedelta] = None) -> List[MeasurementPoint]:
        """
        Get measurement history.
        
        Args:
            metric_name: Specific metric to filter by
            time_window: Time window to filter by
            
        Returns:
            List of measurements
        """
        measurements = list(self.measurement_history)
        
        # Filter by metric name
        if metric_name:
            measurements = [m for m in measurements if m.metric_name == metric_name]
        
        # Filter by time window
        if time_window:
            cutoff_time = datetime.now() - time_window
            measurements = [m for m in measurements if m.timestamp >= cutoff_time]
        
        return measurements
    
    def analyze_measurements(self, metric_name: str) -> Dict[str, Any]:
        """
        Analyze measurements for a specific metric.
        
        Args:
            metric_name: Name of metric to analyze
            
        Returns:
            Analysis results
        """
        measurements = self.get_measurement_history(metric_name)
        
        if not measurements:
            return {'error': f'No measurements found for metric: {metric_name}'}
        
        values = [m.value for m in measurements]
        
        return {
            'metric_name': metric_name,
            'count': len(values),
            'mean': statistics.mean(values),
            'median': statistics.median(values),
            'std_dev': statistics.stdev(values) if len(values) > 1 else 0,
            'min': min(values),
            'max': max(values),
            'latest': values[-1] if values else None,
            'trend': self._calculate_trend(values),
            'time_span_seconds': (measurements[-1].timestamp - measurements[0].timestamp).total_seconds()
        }
    
    def _calculate_trend(self, values: List[float]) -> str:
        """Calculate trend direction for values."""
        if len(values) < 2:
            return 'insufficient_data'
        
        # Simple linear trend calculation
        n = len(values)
        x = list(range(n))
        
        # Calculate slope
        x_mean = statistics.mean(x)
        y_mean = statistics.mean(values)
        
        numerator = sum((x[i] - x_mean) * (values[i] - y_mean) for i in range(n))
        denominator = sum((x[i] - x_mean) ** 2 for i in range(n))
        
        if denominator == 0:
            return 'no_change'
        
        slope = numerator / denominator
        
        if slope > 0.01:
            return 'increasing'
        elif slope < -0.01:
            return 'decreasing'
        else:
            return 'stable'
    
    def monitor_performance(self, 
                          duration: float,
                          interval: float = 1.0,
                          alert_thresholds: Optional[Dict[str, float]] = None) -> Dict[str, Any]:
        """
        Monitor performance continuously.
        
        Args:
            duration: Monitoring duration in seconds
            interval: Sampling interval in seconds
            alert_thresholds: Custom alert thresholds
            
        Returns:
            Monitoring results
        """
        if alert_thresholds:
            thresholds = {**self.thresholds, **alert_thresholds}
        else:
            thresholds = self.thresholds
        
        start_time = time.time()
        samples_collected = 0
        alerts_triggered = []
        
        while time.time() - start_time < duration:
            # Collect metrics
            metrics = self.collect_system_metrics()
            
            # Check thresholds and trigger alerts
            cpu_usage = metrics['metrics']['cpu']['cpu_usage_percent']
            memory_usage = metrics['metrics']['memory']['percent']
            
            if cpu_usage > thresholds.get('cpu_usage', 80):
                alert = {
                    'timestamp': datetime.now().isoformat(),
                    'type': 'high_cpu',
                    'value': cpu_usage,
                    'threshold': thresholds['cpu_usage']
                }
                alerts_triggered.append(alert)
                self.logger.warning(f"High CPU usage alert: {cpu_usage}%")
            
            if memory_usage > thresholds.get('memory_usage', 85):
                alert = {
                    'timestamp': datetime.now().isoformat(),
                    'type': 'high_memory',
                    'value': memory_usage,
                    'threshold': thresholds['memory_usage']
                }
                alerts_triggered.append(alert)
                self.logger.warning(f"High memory usage alert: {memory_usage}%")
            
            # Record measurements
            self.record_measurement('cpu_usage', cpu_usage, 'percent')
            self.record_measurement('memory_usage', memory_usage, 'percent')
            
            samples_collected += 1
            
            # Wait for next sample
            time.sleep(interval)
        
        return {
            'duration': duration,
            'interval': interval,
            'samples_collected': samples_collected,
            'alerts_triggered': alerts_triggered,
            'monitoring_start': datetime.fromtimestamp(start_time).isoformat(),
            'monitoring_end': datetime.now().isoformat()
        }
    
    def generate_performance_report(self, 
                                  session_id: Optional[str] = None,
                                  time_window: Optional[timedelta] = None) -> Dict[str, Any]:
        """
        Generate comprehensive performance report.
        
        Args:
            session_id: Specific session to analyze
            time_window: Time window for analysis
            
        Returns:
            Performance report
        """
        report = {
            'generated_at': datetime.now().isoformat(),
            'report_type': 'performance_analysis'
        }
        
        if session_id and session_id in self.active_sessions:
            # Analyze specific session
            session = self.active_sessions[session_id]
            report['session_analysis'] = self._analyze_session(session)
        else:
            # Analyze all available data
            report['general_analysis'] = self._analyze_all_measurements(time_window)
        
        # System metrics summary
        current_metrics = self.collect_system_metrics()
        report['current_system_state'] = current_metrics
        
        # Performance recommendations
        report['recommendations'] = self._generate_recommendations(current_metrics)
        
        return report
    
    def _analyze_session(self, session: ProfilingSession) -> Dict[str, Any]:
        """Analyze a specific profiling session."""
        analysis = {
            'session_id': session.session_id,
            'duration': session.duration,
            'memory_usage': {
                'samples': len(session.memory_samples),
                'peak_mb': max([s['peak_mb'] for s in session.memory_samples]) if session.memory_samples else 0,
                'average_mb': statistics.mean([s['current_mb'] for s in session.memory_samples]) if session.memory_samples else 0
            },
            'performance_metrics': {}
        }
        
        # Analyze performance measurements
        metric_groups = defaultdict(list)
        for measurement in session.performance_data:
            metric_groups[measurement.metric_name].append(measurement.value)
        
        for metric_name, values in metric_groups.items():
            if values:
                analysis['performance_metrics'][metric_name] = {
                    'mean': statistics.mean(values),
                    'max': max(values),
                    'min': min(values),
                    'samples': len(values)
                }
        
        return analysis
    
    def _analyze_all_measurements(self, time_window: Optional[timedelta]) -> Dict[str, Any]:
        """Analyze all measurements."""
        measurements = self.get_measurement_history(time_window=time_window)
        
        # Group by metric name
        metric_groups = defaultdict(list)
        for measurement in measurements:
            metric_groups[measurement.metric_name].append(measurement)
        
        analysis = {}
        for metric_name, measurements_list in metric_groups.items():
            values = [m.value for m in measurements_list]
            analysis[metric_name] = {
                'count': len(values),
                'mean': statistics.mean(values),
                'latest': values[-1] if values else None,
                'trend': self._calculate_trend(values)
            }
        
        return analysis
    
    def _generate_recommendations(self, current_metrics: Dict[str, Any]) -> List[str]:
        """Generate performance recommendations based on current metrics."""
        recommendations = []
        
        cpu_usage = current_metrics['metrics']['cpu']['cpu_usage_percent']
        memory_usage = current_metrics['metrics']['memory']['percent']
        disk_usage = current_metrics['metrics']['disk']['percent']
        
        if cpu_usage > 80:
            recommendations.append("High CPU usage detected. Consider optimizing CPU-intensive processes or upgrading hardware.")
        
        if memory_usage > 85:
            recommendations.append("High memory usage detected. Consider optimizing memory usage or adding more RAM.")
        
        if disk_usage > 90:
            recommendations.append("High disk usage detected. Consider清理ing up disk space or upgrading storage.")
        
        if not recommendations:
            recommendations.append("System performance appears normal.")
        
        return recommendations
    
    def export_measurements(self, 
                           file_path: str,
                           metric_name: Optional[str] = None,
                           time_window: Optional[timedelta] = None):
        """
        Export measurements to file.
        
        Args:
            file_path: Path to export file
            metric_name: Specific metric to export
            time_window: Time window for export
        """
        measurements = self.get_measurement_history(metric_name, time_window)
        
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'metric_filter': metric_name,
            'time_window': time_window.total_seconds() if time_window else None,
            'measurements': [m.to_dict() for m in measurements]
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2)
        
        self.logger.info(f"Exported {len(measurements)} measurements to {file_path}")
    
    def get_performance_summary(self) -> Dict[str, Any]:
        """Get performance summary for dashboard."""
        recent_measurements = self.get_measurement_history(time_window=timedelta(minutes=10))
        
        # Group by metric
        metric_groups = defaultdict(list)
        for measurement in recent_measurements:
            metric_groups[measurement.metric_name].append(measurement.value)
        
        summary = {
            'timestamp': datetime.now().isoformat(),
            'active_sessions': len(self.active_sessions),
            'total_measurements': len(self.measurement_history),
            'recent_metrics': {}
        }
        
        for metric_name, values in metric_groups.items():
            if values:
                summary['recent_metrics'][metric_name] = {
                    'latest': values[-1],
                    'average': statistics.mean(values),
                    'count': len(values)
                }
        
        # Current system state
        current_metrics = self.collect_system_metrics()
        summary['current_system'] = {
            'cpu_percent': current_metrics['metrics']['cpu']['cpu_usage_percent'],
            'memory_percent': current_metrics['metrics']['memory']['percent'],
            'disk_percent': current_metrics['metrics']['disk']['percent']
        }
        
        return summary


class ResourceMonitor:
    """Resource monitoring with advanced features."""
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize resource monitor.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.profiler = PerformanceProfiler(config)
        self.logger = logging.getLogger(__name__)
        
        # Monitoring state
        self.monitoring_active = False
        self.monitor_thread = None
        self.monitor_interval = config.performance.measurement_interval
        self.monitored_resources = []
        
        # Data storage
        self.resource_history = defaultdict(deque)
        self.alert_callbacks = []
    
    def start_monitoring(self, 
                        resources: List[str],
                        interval: float = 1.0,
                        duration: Optional[float] = None,
                        alerts: bool = True):
        """
        Start resource monitoring.
        
        Args:
            resources: List of resources to monitor
            interval: Monitoring interval in seconds
            duration: Monitoring duration in seconds
            alerts: Enable alert system
        """
        if self.monitoring_active:
            self.logger.warning("Monitoring already active")
            return
        
        self.monitored_resources = resources
        self.monitor_interval = interval
        self.monitoring_duration = duration
        self.enable_alerts = alerts
        
        self.monitoring_active = True
        self.monitor_thread = threading.Thread(target=self._monitoring_loop)
        self.monitor_thread.start()
        
        self.logger.info(f"Started monitoring {len(resources)} resources")
    
    def stop_monitoring(self) -> Dict[str, Any]:
        """Stop resource monitoring and return collected data."""
        if not self.monitoring_active:
            return {'error': 'No monitoring active'}
        
        self.monitoring_active = False
        
        if self.monitor_thread:
            self.monitor_thread.join(timeout=5)
        
        # Compile results
        results = {}
        for resource, history in self.resource_history.items():
            results[resource] = list(history)
        
        self.logger.info("Stopped resource monitoring")
        return results
    
    def _monitoring_loop(self):
        """Main monitoring loop."""
        start_time = time.time()
        
        while self.monitoring_active:
            loop_start = time.time()
            
            # Collect resource data
            current_time = datetime.now()
            
            for resource in self.monitored_resources:
                try:
                    value = self._collect_resource_metric(resource)
                    if value is not None:
                        self.resource_history[resource].append({
                            'timestamp': current_time,
                            'value': value
                        })
                        
                        # Check alerts if enabled
                        if self.enable_alerts:
                            self._check_alerts(resource, value)
                
                except Exception as e:
                    self.logger.warning(f"Failed to collect {resource}: {e}")
            
            # Check duration limit
            if self.monitoring_duration:
                elapsed = time.time() - start_time
                if elapsed >= self.monitoring_duration:
                    break
            
            # Sleep until next interval
            loop_duration = time.time() - loop_start
            sleep_time = max(0, self.monitor_interval - loop_duration)
            
            if sleep_time > 0:
                time.sleep(sleep_time)
    
    def _collect_resource_metric(self, resource: str) -> Optional[float]:
        """Collect a specific resource metric."""
        if resource == 'cpu':
            return psutil.cpu_percent()
        elif resource == 'memory':
            return psutil.virtual_memory().percent
        elif resource == 'disk':
            disk = psutil.disk_usage('/')
            return (disk.used / disk.total) * 100
        elif resource == 'network_rx':
            return psutil.net_io_counters().bytes_recv
        elif resource == 'network_tx':
            return psutil.net_io_counters().bytes_sent
        elif resource == 'processes':
            return len(psutil.pids())
        else:
            return None
    
    def _check_alerts(self, resource: str, value: float):
        """Check if resource value triggers alerts."""
        alert_thresholds = {
            'cpu': 80.0,
            'memory': 85.0,
            'disk': 90.0
        }
        
        if resource in alert_thresholds and value > alert_thresholds[resource]:
            alert = {
                'timestamp': datetime.now().isoformat(),
                'resource': resource,
                'value': value,
                'threshold': alert_thresholds[resource],
                'severity': 'warning' if value < alert_thresholds[resource] + 10 else 'critical'
            }
            
            # Trigger callbacks
            for callback in self.alert_callbacks:
                try:
                    callback(alert)
                except Exception as e:
                    self.logger.warning(f"Alert callback failed: {e}")
    
    def add_alert_callback(self, callback: Callable):
        """Add alert callback function."""
        self.alert_callbacks.append(callback)
    
    def get_resource_statistics(self, resource: str) -> Dict[str, Any]:
        """Get statistics for a monitored resource."""
        if resource not in self.resource_history:
            return {'error': f'Resource {resource} not monitored'}
        
        history = list(self.resource_history[resource])
        if not history:
            return {'error': f'No data for resource {resource}'}
        
        values = [h['value'] for h in history]
        
        return {
            'resource': resource,
            'samples': len(values),
            'mean': statistics.mean(values),
            'median': statistics.median(values),
            'min': min(values),
            'max': max(values),
            'std_dev': statistics.stdev(values) if len(values) > 1 else 0,
            'latest': values[-1],
            'time_span': (history[-1]['timestamp'] - history[0]['timestamp']).total_seconds()
        }
    
    def export_monitoring_data(self, file_path: str):
        """Export monitoring data to file."""
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'monitored_resources': self.monitored_resources,
            'monitoring_data': {resource: [{'timestamp': h['timestamp'].isoformat(), 'value': h['value']} 
                                         for h in history] 
                              for resource, history in self.resource_history.items()}
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Exported monitoring data to {file_path}")