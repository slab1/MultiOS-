"""
Anomaly Detection and Pattern Analysis Framework

Advanced algorithms for detecting anomalies and analyzing patterns
in system behavior and performance metrics.
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
from sklearn.preprocessing import StandardScaler, MinMaxScaler
from sklearn.cluster import DBSCAN, KMeans
from sklearn.decomposition import PCA
from sklearn.metrics import silhouette_score
import warnings
warnings.filterwarnings('ignore')

from .config import ResearchConfig


@dataclass
class AnomalyDetectionConfig:
    """Configuration for anomaly detection algorithms."""
    algorithm: str
    parameters: Dict[str, Any]
    sensitivity: float = 0.95
    min_samples: int = 10
    contamination: float = 0.1
    window_size: int = 50
    update_frequency: int = 300  # seconds
    
    def __post_init__(self):
        # Set default parameters based on algorithm
        if self.algorithm == 'isolation_forest':
            if 'contamination' not in self.parameters:
                self.parameters['contamination'] = self.contamination
        elif self.algorithm == 'statistical':
            if 'z_threshold' not in self.parameters:
                self.parameters['z_threshold'] = 3.0
            if 'iqr_multiplier' not in self.parameters:
                self.parameters['iqr_multiplier'] = 1.5


@dataclass
class AnomalyResult:
    """Result of anomaly detection."""
    anomaly_id: str
    timestamp: datetime
    algorithm: str
    score: float
    severity: str  # 'low', 'medium', 'high', 'critical'
    description: str
    affected_metrics: List[str]
    confidence: float
    context_data: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.context_data is None:
            self.context_data = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'anomaly_id': self.anomaly_id,
            'timestamp': self.timestamp.isoformat(),
            'algorithm': self.algorithm,
            'score': self.score,
            'severity': self.severity,
            'description': self.description,
            'affected_metrics': self.affected_metrics,
            'confidence': self.confidence,
            'context_data': self.context_data
        }


@dataclass
class PatternMatch:
    """Represents a detected pattern match."""
    pattern_id: str
    pattern_type: str
    start_time: datetime
    end_time: Optional[datetime]
    similarity_score: float
    pattern_length: int
    description: str
    matched_metrics: List[str]
    pattern_data: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.pattern_data is None:
            self.pattern_data = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'pattern_id': self.pattern_id,
            'pattern_type': self.pattern_type,
            'start_time': self.start_time.isoformat(),
            'end_time': self.end_time.isoformat() if self.end_time else None,
            'similarity_score': self.similarity_score,
            'pattern_length': self.pattern_length,
            'description': self.description,
            'matched_metrics': self.matched_metrics,
            'pattern_data': self.pattern_data
        }


class AnomalyDetector:
    """
    Advanced anomaly detection system supporting multiple algorithms.
    
    Supports:
    - Statistical anomaly detection
    - Machine learning-based detection
    - Time series anomaly detection
    - Multi-dimensional anomaly detection
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize anomaly detector.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Detection algorithms
        self.detectors = {
            'statistical': StatisticalAnomalyDetector(self.config.analysis.anomaly_threshold),
            'isolation_forest': IsolationForestDetector(),
            'dbscan': DBSCANAnomalyDetector(),
            'local_outlier': LocalOutlierDetector(),
            'trend_anomaly': TrendAnomalyDetector(),
            'seasonal_anomaly': SeasonalAnomalyDetector()
        }
        
        # Detection state
        self.detection_active = False
        self.detection_thread = None
        self.detection_interval = self.config.analysis.update_frequency
        
        # Data storage
        self.metric_data = defaultdict(deque)
        self.detected_anomalies = []
        self.anomaly_thresholds = {}
        
        # Model storage for ML-based detectors
        self.trained_models = {}
        self.scalers = {}
        
        # Configuration
        self.sensitivity = 0.95
        self.contamination_rate = 0.1
        
        self.logger.info(f"Initialized anomaly detector with {len(self.detectors)} algorithms")
    
    def add_metric_data(self, 
                       metric_name: str, 
                       value: float, 
                       timestamp: Optional[datetime] = None):
        """
        Add metric data point for anomaly detection.
        
        Args:
            metric_name: Name of the metric
            value: Metric value
            timestamp: Timestamp of measurement
        """
        if timestamp is None:
            timestamp = datetime.now()
        
        # Store data point
        self.metric_data[metric_name].append({
            'timestamp': timestamp,
            'value': value
        })
        
        # Maintain sliding window
        max_window_size = self.config.analysis.pattern_window * 2
        if len(self.metric_data[metric_name]) > max_window_size:
            # Remove oldest points to maintain window
            excess = len(self.metric_data[metric_name]) - max_window_size
            for _ in range(excess):
                self.metric_data[metric_name].popleft()
    
    def detect_anomalies(self, 
                        metrics: Optional[List[str]] = None,
                        algorithms: Optional[List[str]] = None) -> List[AnomalyResult]:
        """
        Detect anomalies in the provided metrics.
        
        Args:
            metrics: List of metrics to analyze
            algorithms: List of algorithms to use
            
        Returns:
            List of detected anomalies
        """
        if algorithms is None:
            algorithms = list(self.detectors.keys())
        
        if metrics is None:
            metrics = list(self.metric_data.keys())
        
        anomalies = []
        
        for algorithm in algorithms:
            if algorithm not in self.detectors:
                self.logger.warning(f"Unknown algorithm: {algorithm}")
                continue
            
            try:
                detector = self.detectors[algorithm]
                algorithm_anomalies = detector.detect(metrics, self.metric_data, self.detection_config)
                anomalies.extend(algorithm_anomalies)
            except Exception as e:
                self.logger.error(f"Algorithm {algorithm} failed: {e}")
        
        # Remove duplicates and sort by timestamp
        unique_anomalies = self._deduplicate_anomalies(anomalies)
        unique_anomalies.sort(key=lambda x: x.timestamp)
        
        # Update stored anomalies
        self.detected_anomalies.extend(unique_anomalies)
        
        # Keep only recent anomalies
        max_anomalies = 1000
        if len(self.detected_anomalies) > max_anomalies:
            self.detected_anomalies = self.detected_anomalies[-max_anomalies:]
        
        return unique_anomalies
    
    def _deduplicate_anomalies(self, anomalies: List[AnomalyResult]) -> List[AnomalyResult]:
        """Remove duplicate anomalies based on time proximity and metrics."""
        if not anomalies:
            return []
        
        # Sort by timestamp
        sorted_anomalies = sorted(anomalies, key=lambda x: x.timestamp)
        unique_anomalies = []
        
        for anomaly in sorted_anomalies:
            is_duplicate = False
            
            # Check if similar anomaly exists within time window
            time_window = timedelta(minutes=5)
            for existing in unique_anomalies:
                time_diff = abs((anomaly.timestamp - existing.timestamp).total_seconds())
                metrics_overlap = set(anomaly.affected_metrics) & set(existing.affected_metrics)
                
                if time_diff < time_window.total_seconds() and metrics_overlap:
                    # Keep the anomaly with higher confidence
                    if anomaly.confidence > existing.confidence:
                        unique_anomalies.remove(existing)
                        unique_anomalies.append(anomaly)
                    is_duplicate = True
                    break
            
            if not is_duplicate:
                unique_anomalies.append(anomaly)
        
        return unique_anomalies
    
    def start_continuous_detection(self, 
                                 metrics: List[str],
                                 algorithms: List[str],
                                 interval: Optional[float] = None):
        """
        Start continuous anomaly detection.
        
        Args:
            metrics: Metrics to monitor
            algorithms: Algorithms to use
            interval: Detection interval in seconds
        """
        if self.detection_active:
            self.logger.warning("Continuous detection already active")
            return
        
        self.detection_metrics = metrics
        self.detection_algorithms = algorithms
        self.detection_interval = interval or self.config.analysis.update_frequency
        
        self.detection_active = True
        self.detection_thread = threading.Thread(target=self._detection_loop)
        self.detection_thread.start()
        
        self.logger.info(f"Started continuous anomaly detection for {len(metrics)} metrics")
    
    def stop_continuous_detection(self) -> Dict[str, Any]:
        """Stop continuous detection and return statistics."""
        if not self.detection_active:
            return {'error': 'No continuous detection active'}
        
        self.detection_active = False
        
        if self.detection_thread:
            self.detection_thread.join(timeout=5)
        
        # Generate detection statistics
        stats = self._generate_detection_statistics()
        
        self.logger.info("Stopped continuous anomaly detection")
        return stats
    
    def _detection_loop(self):
        """Main detection loop for continuous monitoring."""
        while self.detection_active:
            try:
                # Perform anomaly detection
                anomalies = self.detect_anomalies(self.detection_metrics, self.detection_algorithms)
                
                # Log high-severity anomalies
                high_severity = [a for a in anomalies if a.severity in ['high', 'critical']]
                for anomaly in high_severity:
                    self.logger.warning(f"Anomaly detected ({anomaly.severity}): {anomaly.description}")
                
            except Exception as e:
                self.logger.error(f"Detection loop error: {e}")
            
            # Sleep until next detection
            time.sleep(self.detection_interval)
    
    def _generate_detection_statistics(self) -> Dict[str, Any]:
        """Generate detection statistics."""
        if not self.detected_anomalies:
            return {'total_anomalies': 0}
        
        # Count by algorithm
        algorithm_counts = defaultdict(int)
        severity_counts = defaultdict(int)
        metric_counts = defaultdict(int)
        
        for anomaly in self.detected_anomalies:
            algorithm_counts[anomaly.algorithm] += 1
            severity_counts[anomaly.severity] += 1
            
            for metric in anomaly.affected_metrics:
                metric_counts[metric] += 1
        
        # Time distribution
        now = datetime.now()
        recent_anomalies = [a for a in self.detected_anomalies 
                          if (now - a.timestamp).total_seconds() < 3600]  # Last hour
        
        return {
            'total_anomalies': len(self.detected_anomalies),
            'algorithm_distribution': dict(algorithm_counts),
            'severity_distribution': dict(severity_counts),
            'most_affected_metrics': dict(sorted(metric_counts.items(), key=lambda x: x[1], reverse=True)[:5]),
            'recent_anomalies': len(recent_anomalies),
            'detection_span_hours': (self.detected_anomalies[-1].timestamp - self.detected_anomalies[0].timestamp).total_seconds() / 3600
        }
    
    def train_models(self, 
                    training_data: Dict[str, List[float]],
                    algorithms: Optional[List[str]] = None):
        """
        Train machine learning models for anomaly detection.
        
        Args:
            training_data: Training data for each metric
            algorithms: Algorithms to train
        """
        if algorithms is None:
            algorithms = ['isolation_forest', 'dbscan', 'local_outlier']
        
        for algorithm in algorithms:
            if algorithm not in ['isolation_forest', 'dbscan', 'local_outlier']:
                continue
            
            try:
                if algorithm == 'isolation_forest':
                    self._train_isolation_forest(training_data)
                elif algorithm == 'dbscan':
                    self._train_dbscan(training_data)
                elif algorithm == 'local_outlier':
                    self._train_local_outlier(training_data)
                
                self.logger.info(f"Trained {algorithm} model successfully")
                
            except Exception as e:
                self.logger.error(f"Failed to train {algorithm}: {e}")
    
    def _train_isolation_forest(self, training_data: Dict[str, List[float]]):
        """Train Isolation Forest model."""
        # Prepare training data
        data_matrix = []
        for metric, values in training_data.items():
            data_matrix.append(values)
        
        if len(data_matrix) < 2:
            return
        
        # Transpose to get samples x features
        X = np.array(data_matrix).T
        
        # Scale the data
        scaler = StandardScaler()
        X_scaled = scaler.fit_transform(X)
        
        # Train Isolation Forest
        model = IsolationForest(
            contamination=self.contamination_rate,
            random_state=42,
            n_estimators=100
        )
        model.fit(X_scaled)
        
        # Store model and scaler
        self.trained_models['isolation_forest'] = model
        self.scalers['isolation_forest'] = scaler
    
    def _train_dbscan(self, training_data: Dict[str, List[float]]):
        """Train DBSCAN clustering model."""
        # Prepare training data
        data_matrix = []
        for metric, values in training_data.items():
            data_matrix.append(values)
        
        if len(data_matrix) < 2:
            return
        
        # Transpose to get samples x features
        X = np.array(data_matrix).T
        
        # Scale the data
        scaler = StandardScaler()
        X_scaled = scaler.fit_transform(X)
        
        # Train DBSCAN
        model = DBSCAN(eps=0.5, min_samples=5)
        clusters = model.fit_predict(X_scaled)
        
        # Store model and scaler
        self.trained_models['dbscan'] = model
        self.scalers['dbscan'] = scaler
    
    def _train_local_outlier(self, training_data: Dict[str, List[float]]):
        """Train Local Outlier Factor model."""
        # Prepare training data
        data_matrix = []
        for metric, values in training_data.items():
            data_matrix.append(values)
        
        if len(data_matrix) < 2:
            return
        
        # Transpose to get samples x features
        X = np.array(data_matrix).T
        
        # Scale the data
        scaler = StandardScaler()
        X_scaled = scaler.fit_transform(X)
        
        # Train Local Outlier Factor
        from sklearn.neighbors import LocalOutlierFactor
        model = LocalOutlierFactor(
            n_neighbors=20,
            contamination=self.contamination_rate
        )
        model.fit(X_scaled)
        
        # Store model and scaler
        self.trained_models['local_outlier'] = model
        self.scalers['local_outlier'] = scaler
    
    def get_anomaly_statistics(self, 
                              time_window: Optional[timedelta] = None) -> Dict[str, Any]:
        """Get anomaly detection statistics."""
        if not self.detected_anomalies:
            return {'total_anomalies': 0}
        
        # Filter by time window
        if time_window:
            cutoff_time = datetime.now() - time_window
            filtered_anomalies = [a for a in self.detected_anomalies if a.timestamp >= cutoff_time]
        else:
            filtered_anomalies = self.detected_anomalies
        
        if not filtered_anomalies:
            return {'total_anomalies': 0}
        
        # Calculate statistics
        algorithm_counts = defaultdict(int)
        severity_counts = defaultdict(int)
        metric_counts = defaultdict(int)
        
        for anomaly in filtered_anomalies:
            algorithm_counts[anomaly.algorithm] += 1
            severity_counts[anomaly.severity] += 1
            
            for metric in anomaly.affected_metrics:
                metric_counts[metric] += 1
        
        # Calculate rates
        time_span = (filtered_anomalies[-1].timestamp - filtered_anomalies[0].timestamp).total_seconds()
        rate_per_hour = len(filtered_anomalies) / max(time_span / 3600, 1)
        
        return {
            'total_anomalies': len(filtered_anomalies),
            'rate_per_hour': rate_per_hour,
            'algorithm_distribution': dict(algorithm_counts),
            'severity_distribution': dict(severity_counts),
            'most_affected_metrics': dict(sorted(metric_counts.items(), key=lambda x: x[1], reverse=True)),
            'time_span_hours': time_span / 3600,
            'average_confidence': statistics.mean([a.confidence for a in filtered_anomalies]),
            'high_severity_count': len([a for a in filtered_anomalies if a.severity in ['high', 'critical']])
        }
    
    def export_anomalies(self, file_path: str, time_window: Optional[timedelta] = None):
        """Export detected anomalies to file."""
        if time_window:
            cutoff_time = datetime.now() - time_window
            export_anomalies = [a for a in self.detected_anomalies if a.timestamp >= cutoff_time]
        else:
            export_anomalies = self.detected_anomalies
        
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'total_anomalies': len(export_anomalies),
            'time_window_hours': time_window.total_seconds() / 3600 if time_window else None,
            'anomalies': [anomaly.to_dict() for anomaly in export_anomalies]
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Exported {len(export_anomalies)} anomalies to {file_path}")


class StatisticalAnomalyDetector:
    """Statistical anomaly detection using z-score and IQR methods."""
    
    def __init__(self, threshold: float = 2.0):
        """
        Initialize statistical detector.
        
        Args:
            threshold: Z-score threshold for anomaly detection
        """
        self.threshold = threshold
        self.z_threshold = 2.0
        self.iqr_multiplier = 1.5
    
    def detect(self, 
              metrics: List[str], 
              metric_data: Dict[str, deque],
              config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies using statistical methods."""
        anomalies = []
        
        for metric in metrics:
            if metric not in metric_data or len(metric_data[metric]) < 10:
                continue
            
            # Extract values
            values = [point['value'] for point in metric_data[metric]]
            timestamps = [point['timestamp'] for point in metric_data[metric]]
            
            # Z-score detection
            z_score_anomalies = self._detect_z_score_anomalies(metric, values, timestamps, config)
            anomalies.extend(z_score_anomalies)
            
            # IQR detection
            iqr_anomalies = self._detect_iqr_anomalies(metric, values, timestamps, config)
            anomalies.extend(iqr_anomalies)
        
        return anomalies
    
    def _detect_z_score_anomalies(self, 
                                 metric: str, 
                                 values: List[float], 
                                 timestamps: List[datetime],
                                 config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies using z-score method."""
        anomalies = []
        
        if len(values) < 3:
            return anomalies
        
        mean_val = statistics.mean(values[:-1])  # Exclude last value for training
        std_val = statistics.stdev(values[:-1]) if len(values) > 3 else 0
        
        if std_val == 0:
            return anomalies
        
        # Check last value for anomaly
        last_value = values[-1]
        last_timestamp = timestamps[-1]
        
        z_score = abs((last_value - mean_val) / std_val)
        
        if z_score > self.z_threshold:
            # Determine severity
            if z_score > 3 * self.z_threshold:
                severity = 'critical'
            elif z_score > 2 * self.z_threshold:
                severity = 'high'
            else:
                severity = 'medium'
            
            anomaly = AnomalyResult(
                anomaly_id=f"statistical_zscore_{metric}_{int(last_timestamp.timestamp())}",
                timestamp=last_timestamp,
                algorithm="statistical",
                score=z_score,
                severity=severity,
                description=f"{metric} value {last_value:.2f} is anomalous (z-score: {z_score:.2f})",
                affected_metrics=[metric],
                confidence=min(z_score / (3 * self.z_threshold), 1.0),
                context_data={
                    'z_score': z_score,
                    'mean': mean_val,
                    'std_dev': std_val,
                    'threshold': self.z_threshold
                }
            )
            anomalies.append(anomaly)
        
        return anomalies
    
    def _detect_iqr_anomalies(self, 
                            metric: str, 
                            values: List[float], 
                            timestamps: List[datetime],
                            config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies using Interquartile Range method."""
        anomalies = []
        
        if len(values) < 10:
            return anomalies
        
        # Calculate quartiles using historical data
        historical_values = values[:-1]
        q1 = np.percentile(historical_values, 25)
        q3 = np.percentile(historical_values, 75)
        iqr = q3 - q1
        
        lower_bound = q1 - self.iqr_multiplier * iqr
        upper_bound = q3 + self.iqr_multiplier * iqr
        
        # Check last value
        last_value = values[-1]
        last_timestamp = timestamps[-1]
        
        if last_value < lower_bound or last_value > upper_bound:
            # Determine severity
            if last_value < lower_bound:
                distance = abs(last_value - lower_bound)
                severity = 'high' if distance > iqr else 'medium'
            else:
                distance = abs(last_value - upper_bound)
                severity = 'high' if distance > iqr else 'medium'
            
            anomaly = AnomalyResult(
                anomaly_id=f"statistical_iqr_{metric}_{int(last_timestamp.timestamp())}",
                timestamp=last_timestamp,
                algorithm="statistical",
                score=distance / max(iqr, 0.001),
                severity=severity,
                description=f"{metric} value {last_value:.2f} is outside IQR bounds [{lower_bound:.2f}, {upper_bound:.2f}]",
                affected_metrics=[metric],
                confidence=0.7,
                context_data={
                    'value': last_value,
                    'lower_bound': lower_bound,
                    'upper_bound': upper_bound,
                    'iqr': iqr,
                    'q1': q1,
                    'q3': q3
                }
            )
            anomalies.append(anomaly)
        
        return anomalies


class IsolationForestDetector:
    """Machine learning-based anomaly detection using Isolation Forest."""
    
    def detect(self, 
              metrics: List[str], 
              metric_data: Dict[str, deque],
              config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies using Isolation Forest."""
        anomalies = []
        
        # This would implement Isolation Forest detection
        # Placeholder implementation
        return anomalies


class DBSCANAnomalyDetector:
    """Clustering-based anomaly detection using DBSCAN."""
    
    def detect(self, 
              metrics: List[str], 
              metric_data: Dict[str, deque],
              config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies using DBSCAN clustering."""
        anomalies = []
        
        # This would implement DBSCAN-based detection
        # Placeholder implementation
        return anomalies


class LocalOutlierDetector:
    """Local Outlier Factor-based anomaly detection."""
    
    def detect(self, 
              metrics: List[str], 
              metric_data: Dict[str, deque],
              config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies using Local Outlier Factor."""
        anomalies = []
        
        # This would implement LOF-based detection
        # Placeholder implementation
        return anomalies


class TrendAnomalyDetector:
    """Trend-based anomaly detection."""
    
    def detect(self, 
              metrics: List[str], 
              metric_data: Dict[str, deque],
              config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies based on trend changes."""
        anomalies = []
        
        for metric in metrics:
            if metric not in metric_data or len(metric_data[metric]) < 20:
                continue
            
            values = [point['value'] for point in metric_data[metric]]
            timestamps = [point['timestamp'] for point in metric_data[metric]]
            
            trend_anomalies = self._detect_trend_anomalies(metric, values, timestamps)
            anomalies.extend(trend_anomalies)
        
        return anomalies
    
    def _detect_trend_anomalies(self, 
                              metric: str, 
                              values: List[float], 
                              timestamps: List[datetime]) -> List[AnomalyResult]:
        """Detect trend-based anomalies."""
        anomalies = []
        
        if len(values) < 20:
            return anomalies
        
        # Compare recent trend to historical trend
        recent_window = 10
        historical_window = 10
        
        if len(values) < recent_window + historical_window:
            return anomalies
        
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
            
            # Determine severity
            if slope_change > 1.0:
                severity = 'high'
            elif slope_change > 0.7:
                severity = 'medium'
            else:
                severity = 'low'
            
            anomaly = AnomalyResult(
                anomaly_id=f"trend_{metric}_{int(timestamps[-1].timestamp())}",
                timestamp=timestamps[-1],
                algorithm="trend_anomaly",
                score=slope_change,
                severity=severity,
                description=f"Significant {change_type} detected in {metric} trend",
                affected_metrics=[metric],
                confidence=0.6,
                context_data={
                    'recent_slope': recent_slope,
                    'historical_slope': historical_slope,
                    'change_magnitude': slope_change,
                    'change_type': change_type
                }
            )
            anomalies.append(anomaly)
        
        return anomalies
    
    def _calculate_slope(self, x_values: List[float], y_values: List[float]) -> float:
        """Calculate slope using linear regression."""
        if len(x_values) != len(y_values) or len(x_values) < 2:
            return 0.0
        
        try:
            slope, _, _, _, _ = stats.linregress(x_values, y_values)
            return slope
        except:
            return 0.0


class SeasonalAnomalyDetector:
    """Seasonal pattern-based anomaly detection."""
    
    def detect(self, 
              metrics: List[str], 
              metric_data: Dict[str, deque],
              config: AnomalyDetectionConfig) -> List[AnomalyResult]:
        """Detect anomalies based on seasonal patterns."""
        anomalies = []
        
        for metric in metrics:
            if metric not in metric_data or len(metric_data[metric]) < 50:
                continue
            
            values = [point['value'] for point in metric_data[metric]]
            timestamps = [point['timestamp'] for point in metric_data[metric]]
            
            seasonal_anomalies = self._detect_seasonal_anomalies(metric, values, timestamps)
            anomalies.extend(seasonal_anomalies)
        
        return anomalies
    
    def _detect_seasonal_anomalies(self, 
                                 metric: str, 
                                 values: List[float], 
                                 timestamps: List[datetime]) -> List[AnomalyResult]:
        """Detect seasonal anomalies."""
        anomalies = []
        
        # Simple seasonal detection using autocorrelation
        if len(values) < 24:  # Need at least 24 hours of hourly data
            return anomalies
        
        # Calculate autocorrelation
        autocorr = np.correlate(values - np.mean(values), values - np.mean(values), mode='full')
        autocorr = autocorr[len(autocorr)//2:]
        autocorr = autocorr / autocorr[0]  # Normalize
        
        # Find significant periods (excluding lag 0)
        significant_periods = []
        for lag in range(2, min(len(autocorr), len(values)//2)):
            if autocorr[lag] > 0.5:  # Threshold for significant correlation
                significant_periods.append(lag)
        
        # Check for seasonal anomalies
        if significant_periods:
            # For simplicity, check if current value deviates from expected seasonal pattern
            period = significant_periods[0]  # Use first significant period
            
            if len(values) > period:
                # Get expected value from same position in previous cycle
                expected_idx = len(values) - 1 - period
                if expected_idx >= 0:
                    expected_value = values[expected_idx]
                    actual_value = values[-1]
                    deviation = abs(actual_value - expected_value)
                    
                    if deviation > 2 * np.std(values):  # 2 standard deviations
                        anomaly = AnomalyResult(
                            anomaly_id=f"seasonal_{metric}_{int(timestamps[-1].timestamp())}",
                            timestamp=timestamps[-1],
                            algorithm="seasonal_anomaly",
                            score=deviation / np.std(values),
                            severity='medium',
                            description=f"{metric} value deviates from seasonal pattern (period: {period})",
                            affected_metrics=[metric],
                            confidence=0.5,
                            context_data={
                                'expected_value': expected_value,
                                'actual_value': actual_value,
                                'deviation': deviation,
                                'seasonal_period': period,
                                'autocorrelation': float(autocorr[period])
                            }
                        )
                        anomalies.append(anomaly)
        
        return anomalies


class PatternAnalyzer:
    """
    Advanced pattern analysis for system behavior.
    
    Provides:
    - Pattern recognition and matching
    - Sequence pattern detection
    - Correlation analysis
    - Behavioral clustering
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize pattern analyzer.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Pattern storage
        self.pattern_library = {}
        self.detected_patterns = []
        
        # Pattern detection parameters
        self.min_pattern_length = 5
        self.pattern_similarity_threshold = 0.8
        self.max_patterns_to_store = 1000
        
        # Analysis state
        self.sequence_buffer = defaultdict(deque)
        self.correlation_matrix = {}
        
        self.logger.info("Initialized pattern analyzer")
    
    def analyze_sequences(self, 
                         metric_data: Dict[str, List[float]],
                         min_length: int = 5,
                         similarity_threshold: float = 0.8) -> List[PatternMatch]:
        """
        Analyze sequence patterns in metric data.
        
        Args:
            metric_data: Dictionary of metric names to value sequences
            min_length: Minimum pattern length
            similarity_threshold: Threshold for pattern similarity
            
        Returns:
            List of detected pattern matches
        """
        patterns = []
        
        for metric_name, values in metric_data.items():
            if len(values) < min_length * 2:
                continue
            
            # Detect recurring sequences
            sequence_patterns = self._detect_sequence_patterns(values, metric_name, min_length)
            patterns.extend(sequence_patterns)
            
            # Detect trend patterns
            trend_patterns = self._detect_trend_patterns(values, metric_name)
            patterns.extend(trend_patterns)
            
            # Detect cyclical patterns
            cyclical_patterns = self._detect_cyclical_patterns(values, metric_name)
            patterns.extend(cyclical_patterns)
        
        # Store patterns
        self.detected_patterns.extend(patterns)
        
        # Maintain pattern library size
        if len(self.detected_patterns) > self.max_patterns_to_store:
            self.detected_patterns = self.detected_patterns[-self.max_patterns_to_store:]
        
        return patterns
    
    def _detect_sequence_patterns(self, 
                                values: List[float], 
                                metric_name: str, 
                                min_length: int) -> List[PatternMatch]:
        """Detect recurring sequence patterns."""
        patterns = []
        
        # Convert values to normalized representation
        if len(values) < min_length * 2:
            return patterns
        
        # Simple pattern detection using sliding window
        window_size = min_length
        pattern_count = defaultdict(int)
        
        # Find repeating patterns
        for i in range(len(values) - window_size * 2 + 1):
            pattern1 = values[i:i + window_size]
            pattern2 = values[i + window_size:i + window_size * 2]
            
            # Calculate similarity
            similarity = self._calculate_sequence_similarity(pattern1, pattern2)
            
            if similarity > self.pattern_similarity_threshold:
                pattern_key = tuple(round(v, 2) for v in pattern1)
                pattern_count[pattern_key] += 1
                
                if pattern_count[pattern_key] >= 2:  # Pattern repeats at least twice
                    pattern = PatternMatch(
                        pattern_id=f"sequence_{metric_name}_{len(patterns)}",
                        pattern_type="recurring_sequence",
                        start_time=datetime.now() - timedelta(minutes=len(values)),
                        end_time=datetime.now(),
                        similarity_score=similarity,
                        pattern_length=len(pattern1),
                        description=f"Recurring sequence detected in {metric_name}",
                        matched_metrics=[metric_name],
                        pattern_data={
                            'pattern_values': pattern1,
                            'occurrences': pattern_count[pattern_key],
                            'window_size': window_size
                        }
                    )
                    patterns.append(pattern)
        
        return patterns
    
    def _detect_trend_patterns(self, values: List[float], metric_name: str) -> List[PatternMatch]:
        """Detect trend-based patterns."""
        patterns = []
        
        if len(values) < 10:
            return patterns
        
        # Detect increasing/decreasing sequences
        trend_segments = self._identify_trend_segments(values)
        
        for segment in trend_segments:
            if segment['length'] >= 5:  # Minimum trend length
                pattern = PatternMatch(
                    pattern_id=f"trend_{metric_name}_{len(patterns)}",
                    pattern_type="trend_pattern",
                    start_time=datetime.now() - timedelta(minutes=len(values)) + timedelta(minutes=segment['start_idx']),
                    end_time=datetime.now() - timedelta(minutes=len(values)) + timedelta(minutes=segment['end_idx']),
                    similarity_score=segment['trend_strength'],
                    pattern_length=segment['length'],
                    description=f"{segment['direction']} trend pattern in {metric_name}",
                    matched_metrics=[metric_name],
                    pattern_data={
                        'trend_direction': segment['direction'],
                        'trend_strength': segment['trend_strength'],
                        'start_value': segment['start_value'],
                        'end_value': segment['end_value']
                    }
                )
                patterns.append(pattern)
        
        return patterns
    
    def _detect_cyclical_patterns(self, values: List[float], metric_name: str) -> List[PatternMatch]:
        """Detect cyclical/periodic patterns."""
        patterns = []
        
        if len(values) < 20:
            return patterns
        
        # Use autocorrelation to detect periodicity
        autocorr = np.correlate(values - np.mean(values), values - np.mean(values), mode='full')
        autocorr = autocorr[len(autocorr)//2:]
        autocorr = autocorr / autocorr[0]  # Normalize
        
        # Find significant periods
        significant_periods = []
        for lag in range(2, min(len(autocorr), len(values)//2)):
            if autocorr[lag] > 0.6:  # Strong correlation threshold
                significant_periods.append((lag, autocorr[lag]))
        
        # Create patterns for significant periods
        for period, correlation in significant_periods[:3]:  # Top 3 periods
            pattern = PatternMatch(
                pattern_id=f"cyclical_{metric_name}_{len(patterns)}",
                pattern_type="cyclical_pattern",
                start_time=datetime.now() - timedelta(minutes=len(values)),
                end_time=datetime.now(),
                similarity_score=correlation,
                pattern_length=period,
                description=f"Cyclical pattern in {metric_name} with period {period}",
                matched_metrics=[metric_name],
                pattern_data={
                    'period': period,
                    'autocorrelation': correlation,
                    'cycle_length': period
                }
            )
            patterns.append(pattern)
        
        return patterns
    
    def _identify_trend_segments(self, values: List[float]) -> List[Dict[str, Any]]:
        """Identify trend segments in value sequence."""
        segments = []
        
        if len(values) < 5:
            return segments
        
        # Simple trend identification using slope
        window_size = 5
        
        for i in range(len(values) - window_size + 1):
            window = values[i:i + window_size]
            
            # Calculate trend
            x = list(range(len(window)))
            slope, _, r_value, _, _ = stats.linregress(x, window)
            
            # Determine direction and strength
            if abs(r_value) > 0.7:  # Strong correlation
                direction = "increasing" if slope > 0 else "decreasing"
                strength = abs(r_value)
                
                segments.append({
                    'start_idx': i,
                    'end_idx': i + window_size - 1,
                    'length': window_size,
                    'direction': direction,
                    'trend_strength': strength,
                    'start_value': window[0],
                    'end_value': window[-1]
                })
        
        return segments
    
    def _calculate_sequence_similarity(self, seq1: List[float], seq2: List[float]) -> float:
        """Calculate similarity between two sequences."""
        if len(seq1) != len(seq2):
            return 0.0
        
        # Normalize sequences
        mean1 = statistics.mean(seq1)
        mean2 = statistics.mean(seq2)
        
        norm_seq1 = [v - mean1 for v in seq1]
        norm_seq2 = [v - mean2 for v in seq2]
        
        # Calculate correlation
        correlation, _ = stats.pearsonr(norm_seq1, norm_seq2)
        
        return abs(correlation) if not np.isnan(correlation) else 0.0
    
    def analyze_correlations(self, 
                           metric_data: Dict[str, List[float]],
                           method: str = 'pearson') -> Dict[str, Dict[str, float]]:
        """
        Analyze correlations between metrics.
        
        Args:
            metric_data: Dictionary of metric names to value sequences
            method: Correlation method ('pearson', 'spearman', 'kendall')
            
        Returns:
            Correlation matrix
        """
        if len(metric_data) < 2:
            return {}
        
        metrics = list(metric_data.keys())
        correlations = {}
        
        for i, metric1 in enumerate(metrics):
            correlations[metric1] = {}
            
            for j, metric2 in enumerate(metrics):
                if i == j:
                    correlations[metric1][metric2] = 1.0
                elif i < j:  # Avoid duplicate calculations
                    try:
                        values1 = metric_data[metric1]
                        values2 = metric_data[metric2]
                        
                        # Ensure same length
                        min_length = min(len(values1), len(values2))
                        values1 = values1[-min_length:]
                        values2 = values2[-min_length:]
                        
                        if method == 'pearson':
                            correlation, _ = stats.pearsonr(values1, values2)
                        elif method == 'spearman':
                            correlation, _ = stats.spearmanr(values1, values2)
                        elif method == 'kendall':
                            correlation, _ = stats.kendalltau(values1, values2)
                        else:
                            raise ValueError(f"Unknown correlation method: {method}")
                        
                        correlation = correlation if not np.isnan(correlation) else 0.0
                        
                        correlations[metric1][metric2] = correlation
                        correlations[metric2][metric1] = correlation
                        
                    except Exception as e:
                        self.logger.warning(f"Failed to calculate correlation between {metric1} and {metric2}: {e}")
                        correlations[metric1][metric2] = 0.0
                        correlations[metric2][metric1] = 0.0
                else:
                    # Use already calculated value
                    correlations[metric1][metric2] = correlations[metric2][metric1]
        
        self.correlation_matrix = correlations
        return correlations
    
    def cluster_behavior(self, 
                        behavior_data: Dict[str, List[float]],
                        n_clusters: int = 3) -> Dict[str, Any]:
        """
        Cluster similar behavioral patterns.
        
        Args:
            behavior_data: Dictionary of metric names to value sequences
            n_clusters: Number of clusters to create
            
        Returns:
            Clustering results
        """
        if len(behavior_data) < 2:
            return {'error': 'Insufficient data for clustering'}
        
        try:
            # Prepare data for clustering
            metrics = list(behavior_data.keys())
            data_matrix = []
            
            # Align sequences to same length
            min_length = min(len(values) for values in behavior_data.values())
            for metric in metrics:
                data_matrix.append(behavior_data[metric][-min_length:])
            
            # Transpose to get samples x features
            X = np.array(data_matrix).T
            
            # Normalize data
            scaler = StandardScaler()
            X_scaled = scaler.fit_transform(X)
            
            # Perform clustering
            kmeans = KMeans(n_clusters=n_clusters, random_state=42, n_init=10)
            cluster_labels = kmeans.fit_predict(X_scaled)
            
            # Calculate silhouette score
            if n_clusters > 1:
                silhouette_avg = silhouette_score(X_scaled, cluster_labels)
            else:
                silhouette_avg = 0.0
            
            # Organize results
            clusters = defaultdict(list)
            for i, label in enumerate(cluster_labels):
                clusters[f'cluster_{label}'].append({
                    'metric': metrics[i],
                    'cluster_id': int(label),
                    'distance_to_centroid': float(np.linalg.norm(X_scaled[i] - kmeans.cluster_centers_[label]))
                })
            
            results = {
                'n_clusters': n_clusters,
                'cluster_assignments': dict(clusters),
                'cluster_centroids': kmeans.cluster_centers_.tolist(),
                'silhouette_score': silhouette_avg,
                'inertia': float(kmeans.inertia_)
            }
            
            return results
            
        except Exception as e:
            self.logger.error(f"Clustering failed: {e}")
            return {'error': str(e)}
    
    def find_similar_patterns(self, 
                            target_pattern: List[float],
                            similarity_threshold: float = 0.8) -> List[PatternMatch]:
        """
        Find patterns similar to the target pattern.
        
        Args:
            target_pattern: Pattern to match against
            similarity_threshold: Minimum similarity threshold
            
        Returns:
            List of similar patterns
        """
        similar_patterns = []
        
        for pattern in self.detected_patterns:
            if pattern.pattern_type == "recurring_sequence":
                pattern_values = pattern.pattern_data.get('pattern_values', [])
                
                if len(pattern_values) == len(target_pattern):
                    similarity = self._calculate_sequence_similarity(pattern_values, target_pattern)
                    
                    if similarity >= similarity_threshold:
                        similar_patterns.append(pattern)
        
        # Sort by similarity score
        similar_patterns.sort(key=lambda x: x.similarity_score, reverse=True)
        
        return similar_patterns
    
    def export_patterns(self, file_path: str):
        """Export detected patterns to file."""
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'total_patterns': len(self.detected_patterns),
            'patterns': [pattern.to_dict() for pattern in self.detected_patterns],
            'correlation_matrix': self.correlation_matrix
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Exported {len(self.detected_patterns)} patterns to {file_path}")
    
    def get_pattern_statistics(self) -> Dict[str, Any]:
        """Get pattern analysis statistics."""
        if not self.detected_patterns:
            return {'total_patterns': 0}
        
        # Count by pattern type
        type_counts = defaultdict(int)
        metric_counts = defaultdict(int)
        confidence_scores = []
        
        for pattern in self.detected_patterns:
            type_counts[pattern.pattern_type] += 1
            
            for metric in pattern.matched_metrics:
                metric_counts[metric] += 1
            
            confidence_scores.append(pattern.similarity_score)
        
        return {
            'total_patterns': len(self.detected_patterns),
            'pattern_types': dict(type_counts),
            'most_patterned_metrics': dict(sorted(metric_counts.items(), key=lambda x: x[1], reverse=True)[:5]),
            'average_confidence': statistics.mean(confidence_scores),
            'high_confidence_patterns': len([c for c in confidence_scores if c > 0.9])
        }