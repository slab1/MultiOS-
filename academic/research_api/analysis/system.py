"""
System Behavior Analysis Framework

Advanced system behavior analysis tools for OS research.
Provides pattern detection, anomaly identification, and behavioral modeling.
"""

import os
import time
import json
import numpy as np
import pandas as pd
from typing import Dict, List, Any, Optional, Tuple, Union
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import logging
import statistics
from collections import defaultdict, deque
import threading
from scipy import stats
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
from sklearn.cluster import DBSCAN
import warnings
warnings.filterwarnings('ignore')

from .config import ResearchConfig


@dataclass
class BehaviorPattern:
    """Represents a detected behavior pattern."""
    pattern_id: str
    pattern_type: str
    confidence: float
    start_time: datetime
    end_time: Optional[datetime] = None
    description: str = ""
    frequency: float = 0.0
    parameters: Dict[str, Any] = None
    affected_metrics: List[str] = None
    
    def __post_init__(self):
        if self.parameters is None:
            self.parameters = {}
        if self.affected_metrics is None:
            self.affected_metrics = []
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'pattern_id': self.pattern_id,
            'pattern_type': self.pattern_type,
            'confidence': self.confidence,
            'start_time': self.start_time.isoformat(),
            'end_time': self.end_time.isoformat() if self.end_time else None,
            'description': self.description,
            'frequency': self.frequency,
            'parameters': self.parameters,
            'affected_metrics': self.affected_metrics
        }


@dataclass
class AnomalyAlert:
    """Represents an anomaly detection alert."""
    alert_id: str
    anomaly_type: str
    severity: str  # 'low', 'medium', 'high', 'critical'
    timestamp: datetime
    description: str
    affected_metrics: List[str]
    confidence: float
    additional_data: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.additional_data is None:
            self.additional_data = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'alert_id': self.alert_id,
            'anomaly_type': self.anomaly_type,
            'severity': self.severity,
            'timestamp': self.timestamp.isoformat(),
            'description': self.description,
            'affected_metrics': self.affected_metrics,
            'confidence': self.confidence,
            'additional_data': self.additional_data
        }


@dataclass
class BehaviorModel:
    """Behavioral model of system components."""
    model_id: str
    model_type: str
    creation_time: datetime
    metrics: List[str]
    parameters: Dict[str, Any]
    performance_metrics: Dict[str, float]
    last_update: Optional[datetime] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'model_id': self.model_id,
            'model_type': self.model_type,
            'creation_time': self.creation_time.isoformat(),
            'last_update': self.last_update.isoformat() if self.last_update else None,
            'metrics': self.metrics,
            'parameters': self.parameters,
            'performance_metrics': self.performance_metrics
        }


class SystemAnalyzer:
    """
    Comprehensive system behavior analysis engine.
    
    Provides:
    - Pattern detection and recognition
    - Anomaly detection and alerting
    - Behavior modeling and prediction
    - Trend analysis and forecasting
    - System health assessment
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize system analyzer.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Analysis state
        self.analysis_active = False
        self.analysis_thread = None
        self.analysis_interval = config.analysis.update_frequency
        
        # Data storage
        self.metric_history = defaultdict(deque)
        self.max_history_size = config.analysis.pattern_window * 10
        self.detected_patterns = []
        self.anomaly_alerts = []
        
        # Analysis models
        self.behavior_models = {}
        self.baseline_models = {}
        self.anomaly_detectors = {}
        
        # Analysis configuration
        self.detection_algorithms = config.analysis.detection_algorithms
        self.anomaly_threshold = config.analysis.anomaly_threshold
        
        # Initialize analysis components
        self._initialize_analysis_components()
    
    def _initialize_analysis_components(self):
        """Initialize analysis algorithms and models."""
        # Initialize anomaly detectors
        for algorithm in self.detection_algorithms:
            if algorithm == 'statistical_outlier':
                self.anomaly_detectors['statistical'] = StatisticalAnomalyDetector(self.anomaly_threshold)
            elif algorithm == 'pattern_matching':
                self.anomaly_detectors['pattern'] = PatternMatchingDetector()
            elif algorithm == 'trend_analysis':
                self.anomaly_detectors['trend'] = TrendAnomalyDetector()
        
        self.logger.info(f"Initialized {len(self.anomaly_detectors)} anomaly detectors")
    
    def start_continuous_analysis(self, 
                                metrics: List[str],
                                duration: Optional[float] = None,
                                analysis_interval: Optional[float] = None):
        """
        Start continuous system behavior analysis.
        
        Args:
            metrics: List of metrics to analyze
            duration: Analysis duration in seconds
            analysis_interval: Analysis update interval in seconds
        """
        if self.analysis_active:
            self.logger.warning("Continuous analysis already active")
            return
        
        self.metrics_to_analyze = metrics
        self.analysis_duration = duration
        self.analysis_interval = analysis_interval or self.config.analysis.update_frequency
        
        # Initialize metric history for new metrics
        for metric in metrics:
            if metric not in self.metric_history:
                self.metric_history[metric] = deque(maxlen=self.max_history_size)
        
        self.analysis_active = True
        self.analysis_thread = threading.Thread(target=self._analysis_loop)
        self.analysis_thread.start()
        
        self.logger.info(f"Started continuous analysis for {len(metrics)} metrics")
    
    def stop_continuous_analysis(self) -> Dict[str, Any]:
        """Stop continuous analysis and return summary."""
        if not self.analysis_active:
            return {'error': 'No continuous analysis active'}
        
        self.analysis_active = False
        
        if self.analysis_thread:
            self.analysis_thread.join(timeout=5)
        
        # Generate analysis summary
        summary = self.generate_analysis_summary()
        
        self.logger.info("Stopped continuous analysis")
        return summary
    
    def _analysis_loop(self):
        """Main analysis loop."""
        start_time = time.time()
        
        while self.analysis_active:
            loop_start = time.time()
            
            try:
                # Perform analysis
                analysis_results = self.perform_analysis()
                
                # Process results
                self._process_analysis_results(analysis_results)
                
            except Exception as e:
                self.logger.error(f"Analysis loop error: {e}")
            
            # Check duration limit
            if self.analysis_duration:
                elapsed = time.time() - start_time
                if elapsed >= self.analysis_duration:
                    break
            
            # Sleep until next analysis
            loop_duration = time.time() - loop_start
            sleep_time = max(0, self.analysis_interval - loop_duration)
            
            if sleep_time > 0:
                time.sleep(sleep_time)
    
    def perform_analysis(self) -> Dict[str, Any]:
        """Perform comprehensive system analysis."""
        analysis_results = {
            'timestamp': datetime.now(),
            'patterns_detected': [],
            'anomalies_detected': [],
            'trends_identified': [],
            'health_assessment': {}
        }
        
        # Pattern detection
        patterns = self.detect_patterns()
        analysis_results['patterns_detected'] = patterns
        
        # Anomaly detection
        anomalies = self.detect_anomalies()
        analysis_results['anomalies_detected'] = anomalies
        
        # Trend analysis
        trends = self.analyze_trends()
        analysis_results['trends_identified'] = trends
        
        # Health assessment
        health = self.assess_system_health()
        analysis_results['health_assessment'] = health
        
        return analysis_results
    
    def _process_analysis_results(self, results: Dict[str, Any]):
        """Process analysis results and update state."""
        # Update detected patterns
        for pattern in results['patterns_detected']:
            self.detected_patterns.append(pattern)
        
        # Update anomaly alerts
        for alert in results['anomalies_detected']:
            self.anomaly_alerts.append(alert)
        
        # Keep only recent alerts (last 1000)
        if len(self.anomaly_alerts) > 1000:
            self.anomaly_alerts = self.anomaly_alerts[-1000:]
    
    def collect_behavior_data(self, 
                            duration: int = 60,
                            metrics: Optional[List[str]] = None) -> Dict[str, Any]:
        """
        Collect system behavior data for analysis.
        
        Args:
            duration: Data collection duration in seconds
            metrics: List of metrics to collect
            
        Returns:
            Collected behavior data
        """
        import psutil
        
        collection_start = datetime.now()
        data_points = []
        
        metrics_to_collect = metrics or [
            'cpu_usage', 'memory_usage', 'disk_usage', 'process_count',
            'network_io', 'load_average', 'context_switches'
        ]
        
        start_time = time.time()
        point_interval = max(1, duration // 50)  # Collect up to 50 points
        
        while time.time() - start_time < duration:
            point_start = time.time()
            
            # Collect system metrics
            data_point = {
                'timestamp': datetime.now(),
                'metrics': {}
            }
            
            try:
                # CPU metrics
                if 'cpu_usage' in metrics_to_collect:
                    data_point['metrics']['cpu_usage'] = psutil.cpu_percent(interval=0.1)
                
                # Memory metrics
                if 'memory_usage' in metrics_to_collect:
                    memory = psutil.virtual_memory()
                    data_point['metrics']['memory_usage'] = memory.percent
                
                # Disk metrics
                if 'disk_usage' in metrics_to_collect:
                    disk = psutil.disk_usage('/')
                    data_point['metrics']['disk_usage'] = (disk.used / disk.total) * 100
                
                # Process metrics
                if 'process_count' in metrics_to_collect:
                    data_point['metrics']['process_count'] = len(psutil.pids())
                
                # Network metrics
                if 'network_io' in metrics_to_collect:
                    net_io = psutil.net_io_counters()
                    data_point['metrics']['network_bytes_sent'] = net_io.bytes_sent
                    data_point['metrics']['network_bytes_recv'] = net_io.bytes_recv
                
                # Load average
                if 'load_average' in metrics_to_collect:
                    if hasattr(os, 'getloadavg'):
                        load_avg = os.getloadavg()
                        data_point['metrics']['load_avg_1min'] = load_avg[0]
                        data_point['metrics']['load_avg_5min'] = load_avg[1]
                        data_point['metrics']['load_avg_15min'] = load_avg[2]
                
                data_points.append(data_point)
                
            except Exception as e:
                self.logger.warning(f"Failed to collect metrics: {e}")
            
            # Control collection rate
            elapsed = time.time() - point_start
            sleep_time = max(0, point_interval - elapsed)
            if sleep_time > 0:
                time.sleep(sleep_time)
        
        # Update metric history
        for data_point in data_points:
            for metric_name, value in data_point['metrics'].items():
                self.metric_history[metric_name].append({
                    'timestamp': data_point['timestamp'],
                    'value': value
                })
        
        return {
            'collection_duration': duration,
            'metrics_collected': metrics_to_collect,
            'data_points': len(data_points),
            'collection_start': collection_start.isoformat(),
            'collection_end': datetime.now().isoformat(),
            'raw_data': data_points
        }
    
    def detect_patterns(self) -> List[BehaviorPattern]:
        """Detect behavioral patterns in system data."""
        patterns = []
        
        # Pattern detection for each metric
        for metric_name, history in self.metric_history.items():
            if len(history) < 10:  # Need sufficient data
                continue
            
            # Extract values for analysis
            values = [point['value'] for point in history]
            timestamps = [point['timestamp'] for point in history]
            
            # Detect cyclical patterns
            cyclical_patterns = self._detect_cyclical_patterns(values, timestamps, metric_name)
            patterns.extend(cyclical_patterns)
            
            # Detect step patterns
            step_patterns = self._detect_step_patterns(values, timestamps, metric_name)
            patterns.extend(step_patterns)
            
            # Detect trend patterns
            trend_patterns = self._detect_trend_patterns(values, timestamps, metric_name)
            patterns.extend(trend_patterns)
        
        return patterns
    
    def _detect_cyclical_patterns(self, values: List[float], timestamps: List[datetime], metric_name: str) -> List[BehaviorPattern]:
        """Detect cyclical/periodic patterns."""
        patterns = []
        
        if len(values) < 20:
            return patterns
        
        # Simple period detection using autocorrelation
        try:
            # Calculate autocorrelation
            autocorr = np.correlate(values - np.mean(values), values - np.mean(values), mode='full')
            autocorr = autocorr[len(autocorr)//2:]
            autocorr = autocorr / autocorr[0]  # Normalize
            
            # Find peaks in autocorrelation (excluding lag 0)
            peak_indices = []
            for i in range(2, min(len(autocorr), len(values)//2)):
                if (autocorr[i] > autocorr[i-1] and autocorr[i] > autocorr[i+1] and 
                    autocorr[i] > 0.5):  # Threshold for significant correlation
                    peak_indices.append(i)
            
            if peak_indices:
                # Create pattern for each significant peak
                for lag in peak_indices:
                    pattern = BehaviorPattern(
                        pattern_id=f"cyclical_{metric_name}_{lag}",
                        pattern_type="cyclical",
                        confidence=float(autocorr[lag]),
                        start_time=timestamps[0],
                        description=f"Cyclical pattern detected in {metric_name} with period of {lag} samples",
                        parameters={
                            'period': lag,
                            'correlation': float(autocorr[lag]),
                            'cycle_length_seconds': (timestamps[-1] - timestamps[0]).total_seconds() / (len(timestamps) - lag)
                        },
                        affected_metrics=[metric_name]
                    )
                    patterns.append(pattern)
        
        except Exception as e:
            self.logger.warning(f"Failed to detect cyclical patterns for {metric_name}: {e}")
        
        return patterns
    
    def _detect_step_patterns(self, values: List[float], timestamps: List[datetime], metric_name: str) -> List[BehaviorPattern]:
        """Detect step/jump patterns."""
        patterns = []
        
        if len(values) < 10:
            return patterns
        
        # Detect significant value changes (steps)
        for i in range(1, len(values)):
            change = abs(values[i] - values[i-1])
            relative_change = change / max(abs(values[i-1]), 0.001)  # Avoid division by zero
            
            # Consider it a step if change is significant
            if relative_change > 0.2:  # 20% change threshold
                pattern = BehaviorPattern(
                    pattern_id=f"step_{metric_name}_{i}",
                    pattern_type="step",
                    confidence=min(relative_change, 1.0),  # Cap at 1.0
                    start_time=timestamps[i-1],
                    end_time=timestamps[i],
                    description=f"Step change detected in {metric_name}",
                    parameters={
                        'change_magnitude': change,
                        'relative_change': relative_change,
                        'from_value': values[i-1],
                        'to_value': values[i]
                    },
                    affected_metrics=[metric_name]
                )
                patterns.append(pattern)
        
        return patterns
    
    def _detect_trend_patterns(self, values: List[float], timestamps: List[datetime], metric_name: str) -> List[BehaviorPattern]:
        """Detect trend patterns."""
        patterns = []
        
        if len(values) < 10:
            return patterns
        
        # Linear regression to detect trends
        try:
            x = np.arange(len(values))
            slope, intercept, r_value, p_value, std_err = stats.linregress(x, values)
            
            # Determine if trend is significant
            if abs(r_value) > 0.7 and p_value < 0.05:
                trend_type = "increasing" if slope > 0 else "decreasing"
                
                pattern = BehaviorPattern(
                    pattern_id=f"trend_{metric_name}_{trend_type}",
                    pattern_type="trend",
                    confidence=abs(r_value),
                    start_time=timestamps[0],
                    end_time=timestamps[-1],
                    description=f"{trend_type.title()} trend detected in {metric_name}",
                    parameters={
                        'slope': float(slope),
                        'r_squared': float(r_value**2),
                        'p_value': float(p_value),
                        'trend_type': trend_type
                    },
                    affected_metrics=[metric_name]
                )
                patterns.append(pattern)
        
        except Exception as e:
            self.logger.warning(f"Failed to detect trend patterns for {metric_name}: {e}")
        
        return patterns
    
    def detect_anomalies(self) -> List[AnomalyAlert]:
        """Detect system anomalies using multiple algorithms."""
        alerts = []
        
        # Apply each anomaly detection algorithm
        for detector_name, detector in self.anomaly_detectors.items():
            try:
                detector_alerts = detector.detect(self.metric_history)
                alerts.extend(detector_alerts)
            except Exception as e:
                self.logger.warning(f"Anomaly detector {detector_name} failed: {e}")
        
        return alerts
    
    def analyze_trends(self) -> List[Dict[str, Any]]:
        """Analyze trends across all monitored metrics."""
        trends = []
        
        for metric_name, history in self.metric_history.items():
            if len(history) < 10:
                continue
            
            values = [point['value'] for point in history]
            timestamps = [point['timestamp'] for point in history]
            
            # Calculate trend statistics
            trend_analysis = self._calculate_trend_statistics(values, timestamps)
            trends.append({
                'metric': metric_name,
                'trend_analysis': trend_analysis,
                'data_points': len(values),
                'time_span': (timestamps[-1] - timestamps[0]).total_seconds()
            })
        
        return trends
    
    def _calculate_trend_statistics(self, values: List[float], timestamps: List[datetime]) -> Dict[str, Any]:
        """Calculate detailed trend statistics."""
        if len(values) < 2:
            return {'error': 'Insufficient data'}
        
        # Linear regression
        x = np.arange(len(values))
        slope, intercept, r_value, p_value, std_err = stats.linregress(x, values)
        
        # Change rate calculation
        total_change = values[-1] - values[0]
        relative_change = total_change / max(abs(values[0]), 0.001)
        
        # Volatility (coefficient of variation)
        volatility = np.std(values) / np.mean(values) if np.mean(values) != 0 else 0
        
        # Trend direction and strength
        if abs(r_value) > 0.7 and p_value < 0.05:
            if slope > 0:
                trend_direction = 'increasing'
                trend_strength = 'strong' if abs(r_value) > 0.9 else 'moderate'
            else:
                trend_direction = 'decreasing'
                trend_strength = 'strong' if abs(r_value) > 0.9 else 'moderate'
        else:
            trend_direction = 'stable'
            trend_strength = 'weak'
        
        return {
            'slope': float(slope),
            'r_squared': float(r_value**2),
            'p_value': float(p_value),
            'total_change': float(total_change),
            'relative_change': float(relative_change),
            'volatility': float(volatility),
            'trend_direction': trend_direction,
            'trend_strength': trend_strength,
            'mean': float(np.mean(values)),
            'median': float(np.median(values)),
            'std_dev': float(np.std(values))
        }
    
    def assess_system_health(self) -> Dict[str, Any]:
        """Assess overall system health based on recent behavior."""
        health_metrics = {}
        health_score = 100.0  # Start with perfect health
        
        for metric_name, history in self.metric_history.items():
            if len(history) < 5:
                continue
            
            recent_values = [point['value'] for point in list(history)[-10:]]
            
            # Calculate health indicators
            metric_health = self._assess_metric_health(metric_name, recent_values)
            health_metrics[metric_name] = metric_health
            
            # Adjust overall health score
            health_score -= metric_health['health_penalty']
        
        # Ensure health score is within bounds
        health_score = max(0.0, min(100.0, health_score))
        
        return {
            'overall_health_score': health_score,
            'health_status': self._get_health_status(health_score),
            'metric_health': health_metrics,
            'assessment_timestamp': datetime.now().isoformat()
        }
    
    def _assess_metric_health(self, metric_name: str, values: List[float]) -> Dict[str, Any]:
        """Assess health of a specific metric."""
        if not values:
            return {'health_penalty': 0, 'status': 'unknown'}
        
        # Define health thresholds for common metrics
        health_thresholds = {
            'cpu_usage': {'good': 70, 'warning': 85, 'critical': 95},
            'memory_usage': {'good': 75, 'warning': 85, 'critical': 95},
            'disk_usage': {'good': 80, 'warning': 90, 'critical': 95},
            'process_count': {'good': 500, 'warning': 1000, 'critical': 2000}
        }
        
        current_value = values[-1]
        recent_mean = statistics.mean(values[-5:]) if len(values) >= 5 else current_value
        
        # Determine health status
        if metric_name in health_thresholds:
            thresholds = health_thresholds[metric_name]
            
            if current_value < thresholds['good']:
                status = 'healthy'
                penalty = 0
            elif current_value < thresholds['warning']:
                status = 'acceptable'
                penalty = 5
            elif current_value < thresholds['critical']:
                status = 'warning'
                penalty = 15
            else:
                status = 'critical'
                penalty = 30
        else:
            # For unknown metrics, use statistical analysis
            mean_val = statistics.mean(values)
            std_val = statistics.stdev(values) if len(values) > 1 else 0
            
            if abs(current_value - mean_val) <= std_val:
                status = 'healthy'
                penalty = 0
            elif abs(current_value - mean_val) <= 2 * std_val:
                status = 'acceptable'
                penalty = 5
            else:
                status = 'warning'
                penalty = 10
        
        return {
            'current_value': current_value,
            'recent_mean': recent_mean,
            'status': status,
            'health_penalty': penalty,
            'deviation_from_mean': abs(current_value - statistics.mean(values))
        }
    
    def _get_health_status(self, score: float) -> str:
        """Convert health score to status string."""
        if score >= 90:
            return 'excellent'
        elif score >= 75:
            return 'good'
        elif score >= 60:
            return 'acceptable'
        elif score >= 40:
            return 'warning'
        else:
            return 'critical'
    
    def generate_insights(self, behavior_data: Dict[str, Any]) -> List[str]:
        """Generate insights from behavior analysis."""
        insights = []
        
        # Analyze detected patterns
        patterns = behavior_data.get('patterns', [])
        for pattern in patterns:
            if pattern.pattern_type == 'trend':
                trend_dir = pattern.parameters.get('trend_type', 'unknown')
                confidence = pattern.confidence
                insights.append(f"Detected {trend_dir} trend in {pattern.affected_metrics[0]} with {confidence:.1%} confidence")
            elif pattern.pattern_type == 'cyclical':
                period = pattern.parameters.get('period', 'unknown')
                insights.append(f"Found cyclical pattern in {pattern.affected_metrics[0]} with period of {period} samples")
            elif pattern.pattern_type == 'step':
                change_magnitude = pattern.parameters.get('change_magnitude', 0)
                insights.append(f"Detected significant change in {pattern.affected_metrics[0]} (magnitude: {change_magnitude:.2f})")
        
        # Analyze anomalies
        anomalies = behavior_data.get('anomalies', [])
        for anomaly in anomalies:
            severity = anomaly.severity
            insights.append(f"{severity.upper()} anomaly detected: {anomaly.description}")
        
        # Analyze trends
        trends = behavior_data.get('trends', [])
        for trend in trends:
            trend_dir = trend.get('trend_analysis', {}).get('trend_direction', 'stable')
            trend_strength = trend.get('trend_analysis', {}).get('trend_strength', 'weak')
            metric = trend.get('metric', 'unknown')
            
            if trend_dir != 'stable':
                insights.append(f"{metric} shows {trend_strength} {trend_dir} trend")
        
        # System health insights
        health_assessment = behavior_data.get('health_assessment', {})
        health_score = health_assessment.get('overall_health_score', 100)
        
        if health_score >= 90:
            insights.append(f"System health is excellent ({health_score:.1f}/100)")
        elif health_score >= 75:
            insights.append(f"System health is good ({health_score:.1f}/100)")
        elif health_score >= 60:
            insights.append(f"System health needs attention ({health_score:.1f}/100)")
        else:
            insights.append(f"System health is poor ({health_score:.1f}/100) - immediate attention recommended")
        
        return insights
    
    def get_system_status(self) -> Dict[str, Any]:
        """Get current system analysis status."""
        return {
            'analysis_active': self.analysis_active,
            'metrics_monitored': list(self.metric_history.keys()),
            'patterns_detected': len(self.detected_patterns),
            'anomaly_alerts': len(self.anomaly_alerts),
            'behavior_models': len(self.behavior_models),
            'analysis_algorithms': list(self.anomaly_detectors.keys()),
            'last_analysis': datetime.now().isoformat()
        }
    
    def export_analysis_data(self, file_path: str):
        """Export analysis data to file."""
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'metric_history': {
                metric: [{'timestamp': point['timestamp'].isoformat(), 'value': point['value']}
                        for point in history]
                for metric, history in self.metric_history.items()
            },
            'detected_patterns': [pattern.to_dict() for pattern in self.detected_patterns],
            'anomaly_alerts': [alert.to_dict() for alert in self.anomaly_alerts],
            'system_status': self.get_system_status()
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Exported analysis data to {file_path}")
    
    def generate_analysis_summary(self) -> Dict[str, Any]:
        """Generate comprehensive analysis summary."""
        return {
            'summary_timestamp': datetime.now().isoformat(),
            'analysis_duration': self._get_analysis_duration(),
            'metrics_summary': self._get_metrics_summary(),
            'patterns_summary': self._get_patterns_summary(),
            'anomalies_summary': self._get_anomalies_summary(),
            'health_summary': self._get_health_summary()
        }
    
    def _get_analysis_duration(self) -> float:
        """Get total analysis duration."""
        if not self.metric_history:
            return 0.0
        
        # Calculate from earliest metric data
        earliest_time = None
        latest_time = None
        
        for history in self.metric_history.values():
            if history:
                first_timestamp = history[0]['timestamp']
                last_timestamp = history[-1]['timestamp']
                
                if earliest_time is None or first_timestamp < earliest_time:
                    earliest_time = first_timestamp
                if latest_time is None or last_timestamp > latest_time:
                    latest_time = last_timestamp
        
        if earliest_time and latest_time:
            return (latest_time - earliest_time).total_seconds()
        
        return 0.0
    
    def _get_metrics_summary(self) -> Dict[str, Any]:
        """Get summary of monitored metrics."""
        summary = {}
        
        for metric_name, history in self.metric_history.items():
            if not history:
                continue
            
            values = [point['value'] for point in history]
            summary[metric_name] = {
                'data_points': len(values),
                'time_span_seconds': (history[-1]['timestamp'] - history[0]['timestamp']).total_seconds(),
                'mean': statistics.mean(values),
                'latest_value': values[-1],
                'min_value': min(values),
                'max_value': max(values)
            }
        
        return summary
    
    def _get_patterns_summary(self) -> Dict[str, Any]:
        """Get summary of detected patterns."""
        if not self.detected_patterns:
            return {'total_patterns': 0}
        
        pattern_types = defaultdict(int)
        confidence_scores = []
        
        for pattern in self.detected_patterns:
            pattern_types[pattern.pattern_type] += 1
            confidence_scores.append(pattern.confidence)
        
        return {
            'total_patterns': len(self.detected_patterns),
            'pattern_types': dict(pattern_types),
            'average_confidence': statistics.mean(confidence_scores),
            'high_confidence_patterns': len([c for c in confidence_scores if c > 0.8])
        }
    
    def _get_anomalies_summary(self) -> Dict[str, Any]:
        """Get summary of detected anomalies."""
        if not self.anomaly_alerts:
            return {'total_alerts': 0}
        
        severity_counts = defaultdict(int)
        anomaly_types = defaultdict(int)
        
        for alert in self.anomaly_alerts:
            severity_counts[alert.severity] += 1
            anomaly_types[alert.anomaly_type] += 1
        
        return {
            'total_alerts': len(self.anomaly_alerts),
            'severity_distribution': dict(severity_counts),
            'anomaly_types': dict(anomaly_types),
            'recent_alerts': len([a for a in self.anomaly_alerts if 
                               (datetime.now() - a.timestamp).total_seconds() < 3600])  # Last hour
        }
    
    def _get_health_summary(self) -> Dict[str, Any]:
        """Get system health summary."""
        health_assessment = self.assess_system_health()
        
        return {
            'overall_score': health_assessment['overall_health_score'],
            'health_status': health_assessment['health_status'],
            'metrics_health': {k: v['status'] for k, v in health_assessment['metric_health'].items()},
            'critical_metrics': [k for k, v in health_assessment['metric_health'].items() 
                               if v['status'] == 'critical']
        }


class StatisticalAnomalyDetector:
    """Statistical anomaly detection using z-score and IQR methods."""
    
    def __init__(self, threshold: float = 2.0):
        """
        Initialize statistical anomaly detector.
        
        Args:
            threshold: Z-score threshold for anomaly detection
        """
        self.threshold = threshold
    
    def detect(self, metric_history: Dict[str, deque]) -> List[AnomalyAlert]:
        """Detect anomalies using statistical methods."""
        alerts = []
        
        for metric_name, history in metric_history.items():
            if len(history) < 10:
                continue
            
            values = [point['value'] for point in history]
            timestamps = [point['timestamp'] for point in history]
            
            # Z-score detection
            z_score_alerts = self._detect_z_score_anomalies(values, timestamps, metric_name)
            alerts.extend(z_score_alerts)
            
            # IQR detection
            iqr_alerts = self._detect_iqr_anomalies(values, timestamps, metric_name)
            alerts.extend(iqr_alerts)
        
        return alerts
    
    def _detect_z_score_anomalies(self, values: List[float], timestamps: List[datetime], metric_name: str) -> List[AnomalyAlert]:
        """Detect anomalies using z-score method."""
        alerts = []
        
        if len(values) < 3:
            return alerts
        
        mean_val = statistics.mean(values)
        std_val = statistics.stdev(values)
        
        if std_val == 0:
            return alerts
        
        for i, (value, timestamp) in enumerate(zip(values, timestamps)):
            z_score = abs((value - mean_val) / std_val)
            
            if z_score > self.threshold:
                # Determine severity based on z-score
                if z_score > 3 * self.threshold:
                    severity = 'critical'
                elif z_score > 2 * self.threshold:
                    severity = 'high'
                else:
                    severity = 'medium'
                
                alert = AnomalyAlert(
                    alert_id=f"zscore_{metric_name}_{i}_{int(timestamp.timestamp())}",
                    anomaly_type="statistical_outlier",
                    severity=severity,
                    timestamp=timestamp,
                    description=f"{metric_name} value {value} is anomalous (z-score: {z_score:.2f})",
                    affected_metrics=[metric_name],
                    confidence=min(z_score / (3 * self.threshold), 1.0),
                    additional_data={
                        'z_score': z_score,
                        'mean': mean_val,
                        'std_dev': std_dev,
                        'threshold': self.threshold
                    }
                )
                alerts.append(alert)
        
        return alerts
    
    def _detect_iqr_anomalies(self, values: List[float], timestamps: List[datetime], metric_name: str) -> List[AnomalyAlert]:
        """Detect anomalies using Interquartile Range method."""
        alerts = []
        
        if len(values) < 10:
            return alerts
        
        sorted_values = sorted(values)
        q1 = np.percentile(sorted_values, 25)
        q3 = np.percentile(sorted_values, 75)
        iqr = q3 - q1
        
        lower_bound = q1 - 1.5 * iqr
        upper_bound = q3 + 1.5 * iqr
        
        for i, (value, timestamp) in enumerate(zip(values, timestamps)):
            if value < lower_bound or value > upper_bound:
                # Determine severity based on distance from bounds
                if value < lower_bound:
                    distance = abs(value - lower_bound)
                    severity = 'high' if distance > iqr else 'medium'
                else:
                    distance = abs(value - upper_bound)
                    severity = 'high' if distance > iqr else 'medium'
                
                alert = AnomalyAlert(
                    alert_id=f"iqr_{metric_name}_{i}_{int(timestamp.timestamp())}",
                    anomaly_type="iqr_outlier",
                    severity=severity,
                    timestamp=timestamp,
                    description=f"{metric_name} value {value} is outside IQR bounds [{lower_bound:.2f}, {upper_bound:.2f}]",
                    affected_metrics=[metric_name],
                    confidence=0.7,
                    additional_data={
                        'value': value,
                        'lower_bound': lower_bound,
                        'upper_bound': upper_bound,
                        'iqr': iqr
                    }
                )
                alerts.append(alert)
        
        return alerts


class PatternMatchingDetector:
    """Pattern matching anomaly detection."""
    
    def detect(self, metric_history: Dict[str, deque]) -> List[AnomalyAlert]:
        """Detect anomalies using pattern matching."""
        alerts = []
        
        # This would implement more sophisticated pattern matching
        # For now, return empty list as it's a placeholder implementation
        return alerts


class TrendAnomalyDetector:
    """Trend-based anomaly detection."""
    
    def detect(self, metric_history: Dict[str, deque]) -> List[AnomalyAlert]:
        """Detect anomalies based on trend changes."""
        alerts = []
        
        for metric_name, history in metric_history.items():
            if len(history) < 20:
                continue
            
            values = [point['value'] for point in history]
            timestamps = [point['timestamp'] for point in history]
            
            # Detect sudden trend changes
            trend_alerts = self._detect_trend_changes(values, timestamps, metric_name)
            alerts.extend(trend_alerts)
        
        return alerts
    
    def _detect_trend_changes(self, values: List[float], timestamps: List[datetime], metric_name: str) -> List[AnomalyAlert]:
        """Detect sudden changes in trend."""
        alerts = []
        
        if len(values) < 20:
            return alerts
        
        # Compare recent trend to historical trend
        recent_window = 10
        historical_window = 10
        
        recent_values = values[-recent_window:]
        historical_values = values[-recent_window-historical_window:-recent_window]
        
        # Calculate slopes
        recent_slope = self._calculate_slope(list(range(len(recent_values))), recent_values)
        historical_slope = self._calculate_slope(list(range(len(historical_values))), historical_values)
        
        # Detect significant slope change
        slope_change = abs(recent_slope - historical_slope)
        if slope_change > 0.5:  # Threshold for significant change
            change_type = "acceleration" if (recent_slope > 0 and historical_slope > 0) else "deceleration"
            if recent_slope * historical_slope < 0:
                change_type = "reversal"
            
            alert = AnomalyAlert(
                alert_id=f"trend_change_{metric_name}_{int(timestamps[-1].timestamp())}",
                anomaly_type="trend_change",
                severity='medium',
                timestamp=timestamps[-1],
                description=f"Significant {change_type} detected in {metric_name} trend",
                affected_metrics=[metric_name],
                confidence=0.6,
                additional_data={
                    'recent_slope': recent_slope,
                    'historical_slope': historical_slope,
                    'change_magnitude': slope_change,
                    'change_type': change_type
                }
            )
            alerts.append(alert)
        
        return alerts
    
    def _calculate_slope(self, x_values: List[float], y_values: List[float]) -> float:
        """Calculate slope using linear regression."""
        if len(x_values) != len(y_values) or len(x_values) < 2:
            return 0.0
        
        try:
            slope, _, _, _, _ = stats.linregress(x_values, y_values)
            return slope
        except:
            return 0.0


class BehaviorTracker:
    """Tracks system behavior patterns and changes."""
    
    def __init__(self, config: ResearchConfig):
        """Initialize behavior tracker."""
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Behavior tracking
        self.behavior_baseline = {}
        self.behavior_profiles = {}
        self.behavior_changes = []
        
        # Tracking configuration
        self.tracking_window = config.analysis.pattern_window
        self.baseline_duration = config.analysis.baseline_duration
    
    def create_behavior_baseline(self, 
                               metrics: List[str],
                               duration: int = 300) -> Dict[str, Any]:
        """
        Create behavior baseline for metrics.
        
        Args:
            metrics: List of metrics to baseline
            duration: Baseline collection duration in seconds
            
        Returns:
            Baseline data
        """
        self.logger.info(f"Creating behavior baseline for {len(metrics)} metrics")
        
        # Collect baseline data
        import psutil
        
        baseline_data = {}
        start_time = time.time()
        data_collection_interval = 1.0
        
        for metric in metrics:
            baseline_data[metric] = []
        
        while time.time() - start_time < duration:
            collection_start = time.time()
            
            try:
                # Collect metric values
                for metric in metrics:
                    if metric == 'cpu_usage':
                        value = psutil.cpu_percent(interval=0.1)
                    elif metric == 'memory_usage':
                        value = psutil.virtual_memory().percent
                    elif metric == 'disk_usage':
                        disk = psutil.disk_usage('/')
                        value = (disk.used / disk.total) * 100
                    elif metric == 'process_count':
                        value = len(psutil.pids())
                    else:
                        value = 0  # Placeholder for other metrics
                    
                    baseline_data[metric].append(value)
            
            except Exception as e:
                self.logger.warning(f"Failed to collect baseline metric {metric}: {e}")
            
            # Control collection rate
            elapsed = time.time() - collection_start
            sleep_time = max(0, data_collection_interval - elapsed)
            if sleep_time > 0:
                time.sleep(sleep_time)
        
        # Calculate baseline statistics
        baseline_stats = {}
        for metric, values in baseline_data.items():
            if values:
                baseline_stats[metric] = {
                    'mean': statistics.mean(values),
                    'median': statistics.median(values),
                    'std_dev': statistics.stdev(values) if len(values) > 1 else 0,
                    'min': min(values),
                    'max': max(values),
                    'percentile_5': np.percentile(values, 5),
                    'percentile_95': np.percentile(values, 95),
                    'data_points': len(values)
                }
        
        # Store baseline
        self.behavior_baseline = baseline_stats
        
        self.logger.info("Behavior baseline creation completed")
        return baseline_stats
    
    def compare_with_baseline(self, current_values: Dict[str, float]) -> List[Dict[str, Any]]:
        """
        Compare current values with baseline and identify deviations.
        
        Args:
            current_values: Current metric values
            
        Returns:
            List of deviation reports
        """
        deviations = []
        
        for metric, current_value in current_values.items():
            if metric not in self.behavior_baseline:
                continue
            
            baseline = self.behavior_baseline[metric]
            baseline_mean = baseline['mean']
            baseline_std = baseline['std_dev']
            
            # Calculate deviation
            deviation = abs(current_value - baseline_mean)
            relative_deviation = deviation / max(baseline_mean, 0.001)
            
            # Determine if significant
            if baseline_std > 0:
                z_score = deviation / baseline_std
                is_significant = z_score > 2.0  # 2 standard deviations
            else:
                is_significant = relative_deviation > 0.2  # 20% change
            
            if is_significant:
                deviation_report = {
                    'metric': metric,
                    'current_value': current_value,
                    'baseline_mean': baseline_mean,
                    'deviation': deviation,
                    'relative_deviation': relative_deviation,
                    'z_score': deviation / max(baseline_std, 0.001),
                    'significance': 'high' if deviation > 2 * baseline_std else 'medium',
                    'timestamp': datetime.now().isoformat()
                }
                deviations.append(deviation_report)
        
        return deviations
    
    def track_behavior_changes(self, 
                             current_values: Dict[str, float],
                             change_threshold: float = 0.2) -> List[Dict[str, Any]]:
        """
        Track significant behavior changes.
        
        Args:
            current_values: Current metric values
            change_threshold: Threshold for significant change
            
        Returns:
            List of detected changes
        """
        changes = []
        current_time = datetime.now()
        
        # Compare with previous profile if exists
        if hasattr(self, '_previous_profile') and self._previous_profile:
            for metric, current_value in current_values.items():
                if metric in self._previous_profile:
                    previous_value = self._previous_profile[metric]
                    change = abs(current_value - previous_value) / max(previous_value, 0.001)
                    
                    if change > change_threshold:
                        change_direction = "increase" if current_value > previous_value else "decrease"
                        
                        change_report = {
                            'metric': metric,
                            'change_type': change_direction,
                            'previous_value': previous_value,
                            'current_value': current_value,
                            'change_magnitude': change,
                            'change_percentage': change * 100,
                            'timestamp': current_time.isoformat()
                        }
                        changes.append(change_report)
        
        # Update previous profile
        self._previous_profile = current_values.copy()
        
        return changes