"""
Research Data Collection and Analysis Framework

Comprehensive data collection, storage, and analysis capabilities
for OS research experiments and performance monitoring.
"""

import os
import time
import json
import sqlite3
import threading
import queue
from typing import Dict, List, Any, Optional, Callable, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import logging
import statistics
import pickle
import gzip
import csv
from collections import defaultdict, deque

from .config import ResearchConfig


@dataclass
class DataPoint:
    """Represents a single data point."""
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
class CollectionSession:
    """Represents a data collection session."""
    session_id: str
    name: str
    start_time: datetime
    end_time: Optional[datetime] = None
    duration: Optional[float] = None
    metrics_collected: List[str] = None
    data_points_count: int = 0
    status: str = 'active'  # 'active', 'completed', 'stopped', 'error'
    configuration: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metrics_collected is None:
            self.metrics_collected = []
        if self.configuration is None:
            self.configuration = {}
    
    def complete(self):
        """Mark session as completed."""
        self.end_time = datetime.now()
        self.duration = (self.end_time - self.start_time).total_seconds()
        self.status = 'completed'
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        data = asdict(self)
        data['start_time'] = self.start_time.isoformat()
        if self.end_time:
            data['end_time'] = self.end_time.isoformat()
        return data


class MetricsCollector:
    """
    Metrics collection engine.
    
    Collects system metrics, performance data, and custom measurements.
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize metrics collector.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Collection state
        self.active_collections = {}
        self.collection_thread = None
        self.collection_active = False
        
        # Data storage
        self.current_data_points = deque(maxlen=10000)
        self.metric_history = defaultdict(deque)
        self.collection_buffers = defaultdict(queue.Queue)
        
        # Collection configuration
        self.sample_rate = config.data_collection.metrics_sample_rate
        self.max_buffer_size = config.data_collection.batch_size
        
        # System metrics definitions
        self.system_metrics = self._define_system_metrics()
        self.custom_metrics = {}
        
        # Threading
        self.collection_lock = threading.Lock()
        
        self.logger.info("Metrics collector initialized")
    
    def _define_system_metrics(self) -> Dict[str, Callable]:
        """Define available system metrics."""
        import psutil
        
        metrics = {
            # CPU metrics
            'cpu_usage': lambda: psutil.cpu_percent(interval=None),
            'cpu_count': lambda: psutil.cpu_count(),
            'cpu_count_logical': lambda: psutil.cpu_count(logical=True),
            'load_average_1min': lambda: os.getloadavg()[0] if hasattr(os, 'getloadavg') else None,
            'load_average_5min': lambda: os.getloadavg()[1] if hasattr(os, 'getloadavg') else None,
            'load_average_15min': lambda: os.getloadavg()[2] if hasattr(os, 'getloadavg') else None,
            
            # Memory metrics
            'memory_usage': lambda: psutil.virtual_memory().percent,
            'memory_total_gb': lambda: psutil.virtual_memory().total / (1024**3),
            'memory_available_gb': lambda: psutil.virtual_memory().available / (1024**3),
            'memory_used_gb': lambda: psutil.virtual_memory().used / (1024**3),
            'memory_free_gb': lambda: psutil.virtual_memory().free / (1024**3),
            'swap_usage': lambda: psutil.swap_memory().percent,
            
            # Disk metrics
            'disk_usage': lambda: psutil.disk_usage('/').percent,
            'disk_total_gb': lambda: psutil.disk_usage('/').total / (1024**3),
            'disk_used_gb': lambda: psutil.disk_usage('/').used / (1024**3),
            'disk_free_gb': lambda: psutil.disk_usage('/').free / (1024**3),
            'disk_read_bytes': lambda: psutil.disk_io_counters().read_bytes if psutil.disk_io_counters() else 0,
            'disk_write_bytes': lambda: psutil.disk_io_counters().write_bytes if psutil.disk_io_counters() else 0,
            
            # Network metrics
            'network_bytes_sent': lambda: psutil.net_io_counters().bytes_sent if psutil.net_io_counters() else 0,
            'network_bytes_recv': lambda: psutil.net_io_counters().bytes_recv if psutil.net_io_counters() else 0,
            'network_packets_sent': lambda: psutil.net_io_counters().packets_sent if psutil.net_io_counters() else 0,
            'network_packets_recv': lambda: psutil.net_io_counters().packets_recv if psutil.net_io_counters() else 0,
            
            # Process metrics
            'process_count': lambda: len(psutil.pids()),
            'thread_count': lambda: sum(p.info['num_threads'] for p in psutil.process_iter(['num_threads']) if p.info['num_threads']),
            
            # System info
            'boot_time': lambda: datetime.fromtimestamp(psutil.boot_time()),
            'uptime_seconds': lambda: time.time() - psutil.boot_time(),
        }
        
        return metrics
    
    def register_custom_metric(self, 
                             metric_name: str,
                             collector_function: Callable,
                             unit: str = "units"):
        """
        Register a custom metric collector.
        
        Args:
            metric_name: Name of the metric
            collector_function: Function that returns metric value
            unit: Unit of measurement
        """
        self.custom_metrics[metric_name] = {
            'function': collector_function,
            'unit': unit
        }
        
        self.logger.info(f"Registered custom metric: {metric_name} ({unit})")
    
    def start_collection(self, 
                        metrics: Optional[List[str]] = None,
                        duration: Optional[float] = None,
                        sample_rate: Optional[float] = None) -> str:
        """
        Start metrics collection.
        
        Args:
            metrics: List of metrics to collect
            duration: Collection duration in seconds
            sample_rate: Sample rate in samples per second
            
        Returns:
            Collection session ID
        """
        # Generate session ID
        session_id = f"collection_{int(time.time())}"
        
        # Set configuration
        metrics_to_collect = metrics or list(self.system_metrics.keys())
        collection_duration = duration or float('inf')  # Infinite if not specified
        collection_rate = sample_rate or self.sample_rate
        
        # Create collection session
        session = CollectionSession(
            session_id=session_id,
            name=f"Collection {len(self.active_collections) + 1}",
            start_time=datetime.now(),
            metrics_collected=metrics_to_collect,
            configuration={
                'duration': collection_duration,
                'sample_rate': collection_rate,
                'buffer_size': self.max_buffer_size
            }
        )
        
        self.active_collections[session_id] = session
        
        # Start collection thread if not already running
        if not self.collection_active:
            self._start_collection_thread()
        
        self.logger.info(f"Started metrics collection: {session_id}")
        return session_id
    
    def stop_collection(self, session_id: str) -> CollectionSession:
        """
        Stop metrics collection.
        
        Args:
            session_id: Session ID to stop
            
        Returns:
            Collection session
        """
        if session_id not in self.active_collections:
            raise ValueError(f"Collection session {session_id} not found")
        
        session = self.active_collections[session_id]
        
        # Remove from active collections
        del self.active_collections[session_id]
        
        # Mark as completed
        session.complete()
        
        # Stop collection thread if no more active sessions
        if not self.active_collections:
            self._stop_collection_thread()
        
        self.logger.info(f"Stopped metrics collection: {session_id}")
        return session
    
    def _start_collection_thread(self):
        """Start the collection thread."""
        self.collection_active = True
        self.collection_thread = threading.Thread(target=self._collection_loop)
        self.collection_thread.daemon = True
        self.collection_thread.start()
    
    def _stop_collection_thread(self):
        """Stop the collection thread."""
        self.collection_active = False
        if self.collection_thread:
            self.collection_thread.join(timeout=5)
    
    def _collection_loop(self):
        """Main collection loop."""
        last_collection_times = defaultdict(float)
        
        while self.collection_active:
            current_time = time.time()
            
            # Collect metrics for active sessions
            for session_id, session in self.active_collections.items():
                collection_rate = session.configuration.get('sample_rate', self.sample_rate)
                collection_interval = 1.0 / collection_rate
                
                # Check if it's time to collect for this session
                last_collection = last_collection_times[session_id]
                if current_time - last_collection >= collection_interval:
                    # Collect metrics for this session
                    self._collect_metrics_for_session(session_id, session)
                    last_collection_times[session_id] = current_time
            
            # Sleep briefly
            time.sleep(0.1)
    
    def _collect_metrics_for_session(self, session_id: str, session: CollectionSession):
        """Collect metrics for a specific session."""
        try:
            for metric_name in session.metrics_collected:
                value = self._collect_metric(metric_name)
                
                if value is not None:
                    # Determine unit
                    unit = "unknown"
                    if metric_name in self.system_metrics:
                        # Map common metrics to units
                        unit_mapping = {
                            'cpu_usage': '%',
                            'memory_usage': '%',
                            'disk_usage': '%',
                            'swap_usage': '%',
                            'cpu_count': 'count',
                            'cpu_count_logical': 'count',
                            'process_count': 'count',
                            'thread_count': 'count',
                            'memory_total_gb': 'GB',
                            'memory_available_gb': 'GB',
                            'memory_used_gb': 'GB',
                            'memory_free_gb': 'GB',
                            'disk_total_gb': 'GB',
                            'disk_used_gb': 'GB',
                            'disk_free_gb': 'GB',
                            'disk_read_bytes': 'bytes',
                            'disk_write_bytes': 'bytes',
                            'network_bytes_sent': 'bytes',
                            'network_bytes_recv': 'bytes',
                            'network_packets_sent': 'packets',
                            'network_packets_recv': 'packets',
                            'uptime_seconds': 'seconds',
                            'load_average_1min': 'load',
                            'load_average_5min': 'load',
                            'load_average_15min': 'load'
                        }
                        unit = unit_mapping.get(metric_name, 'unknown')
                    elif metric_name in self.custom_metrics:
                        unit = self.custom_metrics[metric_name]['unit']
                    
                    # Create data point
                    data_point = DataPoint(
                        timestamp=datetime.now(),
                        metric_name=metric_name,
                        value=float(value),
                        unit=unit
                    )
                    
                    # Store data point
                    with self.collection_lock:
                        self.current_data_points.append(data_point)
                        self.metric_history[metric_name].append(data_point)
                        
                        # Update session data points count
                        session.data_points_count += 1
                    
                    # Add to session buffer for async processing
                    if session_id in self.collection_buffers:
                        try:
                            self.collection_buffers[session_id].put_nowait(data_point)
                        except queue.Full:
                            pass  # Buffer full, skip data point
            
            # Check session duration limit
            session_duration = (datetime.now() - session.start_time).total_seconds()
            max_duration = session.configuration.get('duration', float('inf'))
            
            if session_duration >= max_duration:
                self.stop_collection(session_id)
        
        except Exception as e:
            self.logger.error(f"Error collecting metrics for session {session_id}: {e}")
    
    def _collect_metric(self, metric_name: str) -> Optional[float]:
        """Collect a specific metric."""
        try:
            # Check system metrics
            if metric_name in self.system_metrics:
                value = self.system_metrics[metric_name]()
                return float(value) if value is not None else None
            
            # Check custom metrics
            elif metric_name in self.custom_metrics:
                value = self.custom_metrics[metric_name]['function']()
                return float(value) if value is not None else None
            
            else:
                self.logger.warning(f"Unknown metric: {metric_name}")
                return None
                
        except Exception as e:
            self.logger.error(f"Failed to collect metric {metric_name}: {e}")
            return None
    
    def get_current_data(self, 
                        metrics: Optional[List[str]] = None,
                        time_range: Optional[tuple] = None) -> List[DataPoint]:
        """
        Get current data points.
        
        Args:
            metrics: Filter by metric names
            time_range: Filter by time range (start, end)
            
        Returns:
            List of data points
        """
        with self.collection_lock:
            data_points = list(self.current_data_points)
        
        # Apply filters
        if metrics:
            data_points = [dp for dp in data_points if dp.metric_name in metrics]
        
        if time_range:
            start_time, end_time = time_range
            data_points = [dp for dp in data_points if start_time <= dp.timestamp <= end_time]
        
        return data_points
    
    def get_metric_statistics(self, metric_name: str, time_range: Optional[tuple] = None) -> Dict[str, Any]:
        """
        Get statistics for a specific metric.
        
        Args:
            metric_name: Name of the metric
            time_range: Time range for analysis
            
        Returns:
            Metric statistics
        """
        if metric_name not in self.metric_history:
            return {'error': f'Metric {metric_name} not found'}
        
        # Get data points
        with self.collection_lock:
            data_points = list(self.metric_history[metric_name])
        
        # Apply time filter
        if time_range:
            start_time, end_time = time_range
            data_points = [dp for dp in data_points if start_time <= dp.timestamp <= end_time]
        
        if not data_points:
            return {'error': 'No data points available'}
        
        # Calculate statistics
        values = [dp.value for dp in data_points]
        
        return {
            'metric_name': metric_name,
            'data_points': len(values),
            'min': min(values),
            'max': max(values),
            'mean': statistics.mean(values),
            'median': statistics.median(values),
            'std_dev': statistics.stdev(values) if len(values) > 1 else 0,
            'latest_value': values[-1],
            'first_timestamp': data_points[0].timestamp.isoformat(),
            'last_timestamp': data_points[-1].timestamp.isoformat(),
            'time_span_seconds': (data_points[-1].timestamp - data_points[0].timestamp).total_seconds()
        }
    
    def export_data(self, 
                   file_path: str,
                   format: str = 'json',
                   metrics: Optional[List[str]] = None,
                   time_range: Optional[tuple] = None):
        """
        Export collected data.
        
        Args:
            file_path: Export file path
            format: Export format ('json', 'csv', 'parquet')
            metrics: Metrics to export
            time_range: Time range to export
        """
        data_points = self.get_current_data(metrics, time_range)
        
        if format.lower() == 'json':
            self._export_json(data_points, file_path)
        elif format.lower() == 'csv':
            self._export_csv(data_points, file_path)
        elif format.lower() == 'parquet':
            self._export_parquet(data_points, file_path)
        else:
            raise ValueError(f"Unsupported export format: {format}")
        
        self.logger.info(f"Exported {len(data_points)} data points to {file_path}")
    
    def _export_json(self, data_points: List[DataPoint], file_path: str):
        """Export data to JSON format."""
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'total_data_points': len(data_points),
            'data_points': [dp.to_dict() for dp in data_points]
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
    
    def _export_csv(self, data_points: List[DataPoint], file_path: str):
        """Export data to CSV format."""
        with open(file_path, 'w', newline='') as f:
            writer = csv.writer(f)
            
            # Write header
            writer.writerow(['timestamp', 'metric_name', 'value', 'unit', 'metadata'])
            
            # Write data points
            for dp in data_points:
                writer.writerow([
                    dp.timestamp.isoformat(),
                    dp.metric_name,
                    dp.value,
                    dp.unit,
                    json.dumps(dp.metadata)
                ])
    
    def _export_parquet(self, data_points: List[DataPoint], file_path: str):
        """Export data to Parquet format."""
        try:
            import pandas as pd
            
            # Convert to DataFrame
            df_data = []
            for dp in data_points:
                df_data.append({
                    'timestamp': dp.timestamp,
                    'metric_name': dp.metric_name,
                    'value': dp.value,
                    'unit': dp.unit,
                    'metadata': json.dumps(dp.metadata)
                })
            
            df = pd.DataFrame(df_data)
            df.to_parquet(file_path)
            
        except ImportError:
            self.logger.warning("pandas not available, falling back to CSV")
            csv_path = file_path.replace('.parquet', '.csv')
            self._export_csv(data_points, csv_path)


class DataCollector:
    """
    Comprehensive data collection manager.
    
    Orchestrates multiple collection activities and provides unified interface.
    """
    
    def __init__(self, workspace_dir: Path, config: ResearchConfig):
        """
        Initialize data collector.
        
        Args:
            workspace_dir: Workspace directory for data storage
            config: Research configuration
        """
        self.workspace_dir = workspace_dir
        self.config = config
        
        # Setup logging
        self.logger = logging.getLogger(__name__)
        
        # Storage setup
        self.data_dir = workspace_dir / "data"
        self.data_dir.mkdir(exist_ok=True)
        
        # Initialize collectors
        self.metrics_collector = MetricsCollector(config)
        self.data_store = ResearchDataStore(self.data_dir, config)
        
        # Collection management
        self.collection_sessions = {}
        self.active_collections = {}
        
        # Data processing
        self.processors = []
        self.aggregators = {}
        
        self.logger.info("Data collector initialized")
    
    def initialize_collectors(self):
        """Initialize all data collectors."""
        # Setup basic system metrics
        self._setup_system_metrics()
        
        # Setup data storage
        self.data_store.initialize()
        
        self.logger.info("Data collectors initialized")
    
    def _setup_system_metrics(self):
        """Setup basic system metrics."""
        # Register system metrics
        pass  # Metrics are already defined in MetricsCollector
    
    def start_collection(self, duration: float = 300, metrics: Optional[List[str]] = None) -> str:
        """
        Start comprehensive data collection.
        
        Args:
            duration: Collection duration in seconds
            metrics: List of metrics to collect
            
        Returns:
            Collection session ID
        """
        session_id = self.metrics_collector.start_collection(metrics, duration)
        
        # Track session
        session = self.metrics_collector.active_collections[session_id]
        self.collection_sessions[session_id] = session
        self.active_collections[session_id] = session
        
        self.logger.info(f"Started data collection session: {session_id}")
        return session_id
    
    def stop_collection(self, session_id: str) -> CollectionSession:
        """
        Stop data collection session.
        
        Args:
            session_id: Session ID to stop
            
        Returns:
            Collection session
        """
        session = self.metrics_collector.stop_collection(session_id)
        
        # Remove from active collections
        if session_id in self.active_collections:
            del self.active_collections[session_id]
        
        self.logger.info(f"Stopped data collection session: {session_id}")
        return session
    
    def collect_system_metrics(self, duration: float = 60) -> Dict[str, Any]:
        """
        Collect system metrics for specified duration.
        
        Args:
            duration: Collection duration in seconds
            
        Returns:
            Collected system metrics
        """
        session_id = self.start_collection(duration, list(self.metrics_collector.system_metrics.keys()))
        
        # Wait for collection to complete
        time.sleep(duration + 1)  # Add buffer
        
        # Get collected data
        data_points = self.metrics_collector.get_current_data()
        
        # Organize by metric
        metrics_data = defaultdict(list)
        for dp in data_points:
            metrics_data[dp.metric_name].append(dp)
        
        # Calculate basic statistics
        results = {}
        for metric_name, data_points in metrics_data.items():
            values = [dp.value for dp in data_points]
            results[metric_name] = {
                'count': len(values),
                'mean': statistics.mean(values),
                'min': min(values),
                'max': max(values),
                'std_dev': statistics.stdev(values) if len(values) > 1 else 0,
                'latest': values[-1] if values else None,
                'unit': data_points[0].unit if data_points else 'unknown'
            }
        
        return results
    
    def collect_performance_data(self, duration: float = 120) -> Dict[str, Any]:
        """
        Collect performance-related metrics.
        
        Args:
            duration: Collection duration in seconds
            
        Returns:
            Collected performance data
        """
        performance_metrics = [
            'cpu_usage', 'memory_usage', 'disk_usage', 'disk_read_bytes', 
            'disk_write_bytes', 'network_bytes_sent', 'network_bytes_recv',
            'process_count', 'thread_count', 'load_average_1min'
        ]
        
        session_id = self.start_collection(duration, performance_metrics)
        time.sleep(duration + 1)
        
        # Get and analyze performance data
        data_points = self.metrics_collector.get_current_data(performance_metrics)
        
        # Calculate performance statistics
        performance_stats = {}
        for metric in performance_metrics:
            metric_data = [dp for dp in data_points if dp.metric_name == metric]
            if metric_data:
                values = [dp.value for dp in metric_data]
                performance_stats[metric] = {
                    'samples': len(values),
                    'average': statistics.mean(values),
                    'peak': max(values),
                    'minimum': min(values),
                    'unit': metric_data[0].unit
                }
        
        return performance_stats
    
    def collect_behavior_data(self, duration: float = 300) -> Dict[str, Any]:
        """
        Collect behavioral pattern data.
        
        Args:
            duration: Collection duration in seconds
            
        Returns:
            Collected behavioral data
        """
        behavior_metrics = [
            'cpu_usage', 'memory_usage', 'process_count', 'thread_count',
            'load_average_1min', 'load_average_5min', 'load_average_15min'
        ]
        
        session_id = self.start_collection(duration, behavior_metrics)
        time.sleep(duration + 1)
        
        # Get behavior data
        data_points = self.metrics_collector.get_current_data(behavior_metrics)
        
        # Analyze behavioral patterns
        behavior_analysis = {}
        for metric in behavior_metrics:
            metric_data = [dp for dp in data_points if dp.metric_name == metric]
            if metric_data:
                values = [dp.value for dp in metric_data]
                
                # Simple pattern detection
                trend_direction = "stable"
                if len(values) > 10:
                    # Calculate trend
                    recent_avg = statistics.mean(values[-10:])
                    earlier_avg = statistics.mean(values[:-10])
                    
                    if recent_avg > earlier_avg * 1.1:
                        trend_direction = "increasing"
                    elif recent_avg < earlier_avg * 0.9:
                        trend_direction = "decreasing"
                
                behavior_analysis[metric] = {
                    'values': values,
                    'trend_direction': trend_direction,
                    'volatility': statistics.stdev(values) / statistics.mean(values) if statistics.mean(values) > 0 else 0,
                    'unit': metric_data[0].unit
                }
        
        return behavior_analysis
    
    def store_data(self, 
                  session_id: str,
                  system_data: Dict[str, Any],
                  performance_data: Dict[str, Any],
                  behavior_data: Dict[str, Any],
                  storage_format: str = 'json') -> Dict[str, str]:
        """
        Store collected data.
        
        Args:
            session_id: Collection session ID
            system_data: System metrics data
            performance_data: Performance data
            behavior_data: Behavior data
            storage_format: Storage format
            
        Returns:
            Storage results with file paths
        """
        # Prepare data for storage
        stored_data = {
            'session_id': session_id,
            'timestamp': datetime.now().isoformat(),
            'system_data': system_data,
            'performance_data': performance_data,
            'behavior_data': behavior_data
        }
        
        # Store data using data store
        storage_results = self.data_store.store_collection_data(session_id, stored_data, storage_format)
        
        self.logger.info(f"Stored collection data for session: {session_id}")
        return storage_results
    
    def add_data_processor(self, processor_func: Callable):
        """Add a data processor."""
        self.processors.append(processor_func)
        self.logger.info(f"Added data processor: {processor_func.__name__}")
    
    def get_collection_status(self) -> Dict[str, Any]:
        """Get current collection status."""
        active_sessions = len(self.active_collections)
        total_sessions = len(self.collection_sessions)
        
        # Get metrics collector status
        metrics_status = {
            'active_collections': len(self.metrics_collector.active_collections),
            'available_metrics': len(self.metrics_collector.system_metrics) + len(self.metrics_collector.custom_metrics),
            'custom_metrics': len(self.metrics_collector.custom_metrics)
        }
        
        return {
            'active_sessions': active_sessions,
            'total_sessions': total_sessions,
            'metrics_collection': metrics_status,
            'storage_status': self.data_store.get_status()
        }
    
    def export_session_data(self, session_id: str, export_path: str, format: str = 'json'):
        """Export data for a specific session."""
        # Get session data from store
        session_data = self.data_store.get_session_data(session_id)
        
        if not session_data:
            raise ValueError(f"Session data not found: {session_id}")
        
        # Export based on format
        if format.lower() == 'json':
            with open(export_path, 'w') as f:
                json.dump(session_data, f, indent=2, default=str)
        elif format.lower() == 'csv':
            self._export_session_to_csv(session_data, export_path)
        
        self.logger.info(f"Exported session {session_id} data to {export_path}")
    
    def _export_session_to_csv(self, session_data: Dict[str, Any], file_path: str):
        """Export session data to CSV format."""
        with open(file_path, 'w', newline='') as f:
            writer = csv.writer(f)
            
            # Write header
            writer.writerow(['metric', 'value', 'unit', 'data_type', 'timestamp'])
            
            # Write system data
            for metric, data in session_data.get('system_data', {}).items():
                if isinstance(data, dict):
                    writer.writerow([
                        metric, data.get('latest', ''), data.get('unit', ''),
                        'system', session_data['timestamp']
                    ])
            
            # Write performance data
            for metric, data in session_data.get('performance_data', {}).items():
                if isinstance(data, dict):
                    writer.writerow([
                        metric, data.get('average', ''), data.get('unit', ''),
                        'performance', session_data['timestamp']
                    ])
    
    def get_status(self) -> Dict[str, Any]:
        """Get comprehensive status information."""
        return {
            'data_collector_status': 'active',
            'collection_sessions': len(self.collection_sessions),
            'active_collections': len(self.active_collections),
            'metrics_collector': {
                'active_sessions': len(self.metrics_collector.active_collections),
                'available_metrics': len(self.metrics_collector.system_metrics),
                'custom_metrics': len(self.metrics_collector.custom_metrics)
            },
            'data_storage': self.data_store.get_status()
        }


class ResearchDataStore:
    """
    Research data storage and management system.
    
    Handles storage, retrieval, and management of research data.
    """
    
    def __init__(self, data_dir: Path, config: ResearchConfig):
        """
        Initialize research data store.
        
        Args:
            data_dir: Data storage directory
            config: Research configuration
        """
        self.data_dir = data_dir
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Storage backend
        self.backend = config.data_collection.storage_backend
        
        # Initialize storage
        self.storage_path = self.data_dir / "research_data.db"
        self._initialize_storage()
        
        self.logger.info(f"Research data store initialized with {self.backend} backend")
    
    def _initialize_storage(self):
        """Initialize storage backend."""
        if self.backend == 'sqlite':
            self._init_sqlite()
        elif self.backend == 'file':
            self._init_file_storage()
        else:
            raise ValueError(f"Unsupported storage backend: {self.backend}")
    
    def _init_sqlite(self):
        """Initialize SQLite storage."""
        try:
            conn = sqlite3.connect(self.storage_path)
            cursor = conn.cursor()
            
            # Create tables
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS collection_sessions (
                    session_id TEXT PRIMARY KEY,
                    name TEXT,
                    start_time TEXT,
                    end_time TEXT,
                    duration REAL,
                    metrics_collected TEXT,
                    data_points_count INTEGER,
                    status TEXT,
                    configuration TEXT
                )
            ''')
            
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS data_points (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT,
                    timestamp TEXT,
                    metric_name TEXT,
                    value REAL,
                    unit TEXT,
                    metadata TEXT,
                    FOREIGN KEY (session_id) REFERENCES collection_sessions (session_id)
                )
            ''')
            
            conn.commit()
            conn.close()
            
        except Exception as e:
            self.logger.error(f"Failed to initialize SQLite storage: {e}")
            raise
    
    def _init_file_storage(self):
        """Initialize file-based storage."""
        self.data_dir.mkdir(exist_ok=True)
        
        # Create subdirectories
        (self.data_dir / "sessions").mkdir(exist_ok=True)
        (self.data_dir / "data_points").mkdir(exist_ok=True)
    
    def initialize(self):
        """Initialize the data store."""
        self.logger.info("Research data store initialized")
    
    def store_collection_data(self, 
                             session_id: str,
                             collection_data: Dict[str, Any],
                             storage_format: str = 'json') -> Dict[str, str]:
        """
        Store collection data.
        
        Args:
            session_id: Collection session ID
            collection_data: Data to store
            storage_format: Storage format
            
        Returns:
            Storage results with file paths
        """
        if self.backend == 'sqlite':
            return self._store_sqlite(session_id, collection_data)
        elif self.backend == 'file':
            return self._store_file(session_id, collection_data, storage_format)
        else:
            raise ValueError(f"Unsupported storage backend: {self.backend}")
    
    def _store_sqlite(self, session_id: str, collection_data: Dict[str, Any]) -> Dict[str, str]:
        """Store data in SQLite database."""
        conn = sqlite3.connect(self.storage_path)
        cursor = conn.cursor()
        
        try:
            # Store session metadata
            session_data = collection_data.get('metadata', {})
            cursor.execute('''
                INSERT OR REPLACE INTO collection_sessions 
                (session_id, name, start_time, end_time, duration, metrics_collected, 
                 data_points_count, status, configuration)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                session_id,
                session_data.get('name', f'Session {session_id}'),
                session_data.get('start_time', datetime.now().isoformat()),
                session_data.get('end_time'),
                session_data.get('duration'),
                json.dumps(session_data.get('metrics_collected', [])),
                session_data.get('data_points_count', 0),
                session_data.get('status', 'completed'),
                json.dumps(session_data.get('configuration', {}))
            ))
            
            # Store data points if available
            data_points = collection_data.get('data_points', [])
            for dp_data in data_points:
                cursor.execute('''
                    INSERT INTO data_points 
                    (session_id, timestamp, metric_name, value, unit, metadata)
                    VALUES (?, ?, ?, ?, ?, ?)
                ''', (
                    session_id,
                    dp_data['timestamp'],
                    dp_data['metric_name'],
                    dp_data['value'],
                    dp_data['unit'],
                    json.dumps(dp_data.get('metadata', {}))
                ))
            
            conn.commit()
            
            return {
                'storage_method': 'sqlite',
                'database_path': str(self.storage_path),
                'session_id': session_id,
                'status': 'success'
            }
            
        finally:
            conn.close()
    
    def _store_file(self, session_id: str, collection_data: Dict[str, Any], format: str = 'json') -> Dict[str, str]:
        """Store data in files."""
        session_dir = self.data_dir / "sessions" / session_id
        session_dir.mkdir(parents=True, exist_ok=True)
        
        file_paths = {}
        
        # Store main session data
        session_file = session_dir / f"session_data.{format}"
        if format.lower() == 'json':
            with open(session_file, 'w') as f:
                json.dump(collection_data, f, indent=2, default=str)
        elif format.lower() == 'csv':
            self._export_to_csv(collection_data, session_file)
        
        file_paths['session_data'] = str(session_file)
        
        # Store individual data files
        for data_type, data in collection_data.items():
            if isinstance(data, dict):
                data_file = session_dir / f"{data_type}.{format}"
                if format.lower() == 'json':
                    with open(data_file, 'w') as f:
                        json.dump(data, f, indent=2, default=str)
                elif format.lower() == 'csv':
                    self._export_dict_to_csv(data, data_file)
                file_paths[data_type] = str(data_file)
        
        return {
            'storage_method': 'file',
            'session_directory': str(session_dir),
            'files': file_paths,
            'session_id': session_id,
            'status': 'success'
        }
    
    def _export_to_csv(self, data: Dict[str, Any], file_path: Path):
        """Export data to CSV format."""
        with open(file_path, 'w', newline='') as f:
            writer = csv.writer(f)
            
            # Determine structure and write data
            if 'system_data' in data:
                writer.writerow(['Section', 'Metric', 'Value', 'Unit'])
                
                # Write system data
                for metric, metrics_data in data['system_data'].items():
                    if isinstance(metrics_data, dict):
                        writer.writerow(['System', metric, metrics_data.get('latest', ''), metrics_data.get('unit', '')])
            
            # Write performance data
            if 'performance_data' in data:
                for metric, perf_data in data['performance_data'].items():
                    if isinstance(perf_data, dict):
                        writer.writerow(['Performance', metric, perf_data.get('average', ''), perf_data.get('unit', '')])
    
    def _export_dict_to_csv(self, data: Dict[str, Any], file_path: Path):
        """Export dictionary data to CSV."""
        with open(file_path, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['Metric', 'Value', 'Unit', 'Additional_Data'])
            
            for metric, metric_data in data.items():
                if isinstance(metric_data, dict):
                    writer.writerow([
                        metric,
                        metric_data.get('latest', metric_data.get('average', '')),
                        metric_data.get('unit', ''),
                        json.dumps({k: v for k, v in metric_data.items() if k not in ['latest', 'average', 'unit']})
                    ])
                else:
                    writer.writerow([metric, metric_data, '', ''])
    
    def get_session_data(self, session_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve session data."""
        if self.backend == 'sqlite':
            return self._get_session_sqlite(session_id)
        elif self.backend == 'file':
            return self._get_session_file(session_id)
        else:
            return None
    
    def _get_session_sqlite(self, session_id: str) -> Optional[Dict[str, Any]]:
        """Get session data from SQLite."""
        conn = sqlite3.connect(self.storage_path)
        cursor = conn.cursor()
        
        try:
            # Get session metadata
            cursor.execute('SELECT * FROM collection_sessions WHERE session_id = ?', (session_id,))
            session_row = cursor.fetchone()
            
            if not session_row:
                return None
            
            session_data = {
                'session_id': session_row[0],
                'name': session_row[1],
                'start_time': session_row[2],
                'end_time': session_row[3],
                'duration': session_row[4],
                'metrics_collected': json.loads(session_row[5]) if session_row[5] else [],
                'data_points_count': session_row[6],
                'status': session_row[7],
                'configuration': json.loads(session_row[8]) if session_row[8] else {}
            }
            
            # Get data points
            cursor.execute('SELECT timestamp, metric_name, value, unit, metadata FROM data_points WHERE session_id = ?', (session_id,))
            data_points = []
            for row in cursor.fetchall():
                data_points.append({
                    'timestamp': row[0],
                    'metric_name': row[1],
                    'value': row[2],
                    'unit': row[3],
                    'metadata': json.loads(row[4]) if row[4] else {}
                })
            
            session_data['data_points'] = data_points
            
            return session_data
            
        finally:
            conn.close()
    
    def _get_session_file(self, session_id: str) -> Optional[Dict[str, Any]]:
        """Get session data from files."""
        session_dir = self.data_dir / "sessions" / session_id
        
        if not session_dir.exists():
            return None
        
        # Load session data
        session_file = session_dir / "session_data.json"
        if session_file.exists():
            with open(session_file, 'r') as f:
                session_data = json.load(f)
            return session_data
        
        return None
    
    def list_sessions(self) -> List[str]:
        """List all stored sessions."""
        if self.backend == 'sqlite':
            return self._list_sessions_sqlite()
        elif self.backend == 'file':
            return self._list_sessions_file()
        else:
            return []
    
    def _list_sessions_sqlite(self) -> List[str]:
        """List sessions from SQLite."""
        conn = sqlite3.connect(self.storage_path)
        cursor = conn.cursor()
        
        try:
            cursor.execute('SELECT session_id FROM collection_sessions')
            return [row[0] for row in cursor.fetchall()]
        finally:
            conn.close()
    
    def _list_sessions_file(self) -> List[str]:
        """List sessions from files."""
        sessions_dir = self.data_dir / "sessions"
        if sessions_dir.exists():
            return [d.name for d in sessions_dir.iterdir() if d.is_dir()]
        return []
    
    def get_status(self) -> Dict[str, Any]:
        """Get storage status."""
        sessions = self.list_sessions()
        
        return {
            'backend': self.backend,
            'storage_path': str(self.storage_path) if self.backend == 'sqlite' else str(self.data_dir),
            'total_sessions': len(sessions),
            'storage_size_mb': self._calculate_storage_size(),
            'oldest_session': min(sessions) if sessions else None,
            'newest_session': max(sessions) if sessions else None
        }
    
    def _calculate_storage_size(self) -> float:
        """Calculate total storage size."""
        total_size = 0
        
        if self.backend == 'sqlite' and self.storage_path.exists():
            total_size = self.storage_path.stat().st_size
        elif self.backend == 'file':
            for file_path in self.data_dir.rglob('*'):
                if file_path.is_file():
                    total_size += file_path.stat().st_size
        
        return total_size / (1024 * 1024)  # Convert to MB
    
    def cleanup_old_sessions(self, retention_days: int = 30):
        """Clean up old sessions."""
        cutoff_date = datetime.now() - timedelta(days=retention_days)
        
        sessions = self.list_sessions()
        cleaned_count = 0
        
        for session_id in sessions:
            try:
                session_data = self.get_session_data(session_id)
                if session_data and session_data.get('start_time'):
                    session_date = datetime.fromisoformat(session_data['start_time'])
                    
                    if session_date < cutoff_date:
                        self._delete_session(session_id)
                        cleaned_count += 1
            except Exception as e:
                self.logger.warning(f"Failed to clean session {session_id}: {e}")
        
        self.logger.info(f"Cleaned up {cleaned_count} old sessions")
        return cleaned_count
    
    def _delete_session(self, session_id: str):
        """Delete a session."""
        if self.backend == 'sqlite':
            conn = sqlite3.connect(self.storage_path)
            cursor = conn.cursor()
            
            try:
                cursor.execute('DELETE FROM data_points WHERE session_id = ?', (session_id,))
                cursor.execute('DELETE FROM collection_sessions WHERE session_id = ?', (session_id,))
                conn.commit()
            finally:
                conn.close()
        elif self.backend == 'file':
            import shutil
            session_dir = self.data_dir / "sessions" / session_id
            if session_dir.exists():
                shutil.rmtree(session_dir)
    
    def export_all_data(self, export_path: str, format: str = 'json'):
        """Export all stored data."""
        sessions = self.list_sessions()
        all_data = {
            'export_timestamp': datetime.now().isoformat(),
            'total_sessions': len(sessions),
            'sessions': {}
        }
        
        for session_id in sessions:
            session_data = self.get_session_data(session_id)
            if session_data:
                all_data['sessions'][session_id] = session_data
        
        if format.lower() == 'json':
            with open(export_path, 'w') as f:
                json.dump(all_data, f, indent=2, default=str)
        else:
            raise ValueError(f"Unsupported export format: {format}")
        
        self.logger.info(f"Exported {len(sessions)} sessions to {export_path}")


# Add missing imports
import platform