"""
Data Analysis Module for OS Research API

This module provides comprehensive data analysis capabilities for OS research experiments,
including statistical analysis, time series analysis, regression analysis, clustering,
and specialized OS performance analysis techniques.
"""

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from typing import Dict, List, Any, Optional, Tuple, Union
from scipy import stats
from sklearn.cluster import KMeans
from sklearn.preprocessing import StandardScaler
from sklearn.decomposition import PCA
from sklearn.linear_model import LinearRegression
from sklearn.metrics import r2_score
import warnings
import logging
from pathlib import Path

logger = logging.getLogger(__name__)

class OSDataAnalyzer:
    """
    Comprehensive data analysis engine for OS research experiments.
    
    Provides statistical analysis, time series analysis, regression analysis,
    clustering, anomaly detection, and specialized OS performance metrics.
    """
    
    def __init__(self, config: Optional[Dict] = None):
        """
        Initialize the OS Data Analyzer.
        
        Args:
            config: Configuration dictionary for analysis parameters
        """
        self.config = config or {}
        self.data_cache = {}
        self.analysis_results = {}
        self._setup_logging()
        
    def _setup_logging(self):
        """Setup logging for analysis operations."""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
    
    def load_data(self, data_path: Union[str, Path], 
                  data_type: str = 'auto') -> pd.DataFrame:
        """
        Load research data from various formats.
        
        Args:
            data_path: Path to data file
            data_type: Type of data file ('csv', 'json', 'parquet', 'auto')
            
        Returns:
            Loaded dataframe
        """
        data_path = Path(data_path)
        
        if data_type == 'auto':
            suffix = data_path.suffix.lower()
            if suffix == '.csv':
                data_type = 'csv'
            elif suffix == '.json':
                data_type = 'json'
            elif suffix == '.parquet':
                data_type = 'parquet'
            else:
                raise ValueError(f"Unsupported file type: {suffix}")
        
        try:
            if data_type == 'csv':
                df = pd.read_csv(data_path)
            elif data_type == 'json':
                df = pd.read_json(data_path)
            elif data_type == 'parquet':
                df = pd.read_parquet(data_path)
            else:
                raise ValueError(f"Unsupported data type: {data_type}")
                
            logger.info(f"Successfully loaded data from {data_path}")
            return df
            
        except Exception as e:
            logger.error(f"Failed to load data from {data_path}: {e}")
            raise
    
    def basic_statistics(self, data: Union[pd.DataFrame, pd.Series]) -> Dict[str, Any]:
        """
        Compute comprehensive basic statistics.
        
        Args:
            data: Data to analyze
            
        Returns:
            Dictionary containing comprehensive statistics
        """
        try:
            if isinstance(data, pd.Series):
                stats_dict = {
                    'count': len(data),
                    'mean': data.mean(),
                    'std': data.std(),
                    'min': data.min(),
                    'max': data.max(),
                    'median': data.median(),
                    'q25': data.quantile(0.25),
                    'q75': data.quantile(0.75),
                    'skewness': stats.skew(data),
                    'kurtosis': stats.kurtosis(data),
                    'missing_values': data.isnull().sum()
                }
            else:
                stats_dict = {}
                for column in data.columns:
                    if data[column].dtype in ['int64', 'float64']:
                        stats_dict[column] = self.basic_statistics(data[column])
                    else:
                        stats_dict[column] = {
                            'count': len(data[column]),
                            'unique_values': data[column].nunique(),
                            'most_frequent': data[column].mode().iloc[0] if len(data[column].mode()) > 0 else None,
                            'missing_values': data[column].isnull().sum()
                        }
            
            logger.info("Successfully computed basic statistics")
            return stats_dict
            
        except Exception as e:
            logger.error(f"Failed to compute basic statistics: {e}")
            raise
    
    def correlation_analysis(self, data: pd.DataFrame, 
                           method: str = 'pearson',
                           threshold: float = 0.5) -> Dict[str, Any]:
        """
        Perform correlation analysis between variables.
        
        Args:
            data: Data to analyze
            method: Correlation method ('pearson', 'spearman', 'kendall')
            threshold: Correlation threshold for significance
            
        Returns:
            Dictionary containing correlation results
        """
        try:
            # Select numeric columns only
            numeric_data = data.select_dtypes(include=[np.number])
            
            if numeric_data.empty:
                raise ValueError("No numeric columns found for correlation analysis")
            
            # Compute correlation matrix
            corr_matrix = numeric_data.corr(method=method)
            
            # Find significant correlations
            significant_pairs = []
            for i in range(len(corr_matrix.columns)):
                for j in range(i+1, len(corr_matrix.columns)):
                    corr_value = corr_matrix.iloc[i, j]
                    if abs(corr_value) >= threshold:
                        significant_pairs.append({
                            'var1': corr_matrix.columns[i],
                            'var2': corr_matrix.columns[j],
                            'correlation': corr_value,
                            'strength': self._interpret_correlation(abs(corr_value))
                        })
            
            result = {
                'correlation_matrix': corr_matrix,
                'significant_correlations': significant_pairs,
                'method': method,
                'threshold': threshold
            }
            
            logger.info(f"Correlation analysis completed with {len(significant_pairs)} significant pairs")
            return result
            
        except Exception as e:
            logger.error(f"Correlation analysis failed: {e}")
            raise
    
    def _interpret_correlation(self, correlation: float) -> str:
        """Interpret correlation strength."""
        if correlation >= 0.8:
            return "Very Strong"
        elif correlation >= 0.6:
            return "Strong"
        elif correlation >= 0.4:
            return "Moderate"
        elif correlation >= 0.2:
            return "Weak"
        else:
            return "Very Weak"
    
    def time_series_analysis(self, data: pd.Series, 
                           periods: int = 24,
                           method: str = 'seasonal') -> Dict[str, Any]:
        """
        Perform time series analysis.
        
        Args:
            data: Time series data
            periods: Number of periods for seasonal decomposition
            method: Analysis method ('seasonal', 'trend', 'autocorrelation')
            
        Returns:
            Dictionary containing time series analysis results
        """
        try:
            result = {}
            
            if method == 'seasonal':
                # Seasonal decomposition
                from statsmodels.tsa.seasonal import seasonal_decompose
                
                if len(data) < periods * 2:
                    raise ValueError(f"Insufficient data for seasonal decomposition. Need at least {periods * 2} data points")
                
                decomposition = seasonal_decompose(data.dropna(), model='additive', period=periods)
                
                result = {
                    'trend': decomposition.trend,
                    'seasonal': decomposition.seasonal,
                    'residual': decomposition.resid,
                    'observed': decomposition.observed,
                    'periods': periods
                }
                
            elif method == 'trend':
                # Trend analysis using linear regression
                x = np.arange(len(data))
                valid_data = data.dropna()
                y = valid_data.values
                
                model = LinearRegression()
                model.fit(x[:len(y)].reshape(-1, 1), y)
                
                trend_slope = model.coef_[0]
                trend_r2 = r2_score(y, model.predict(x[:len(y)].reshape(-1, 1)))
                
                result = {
                    'trend_slope': trend_slope,
                    'trend_direction': 'increasing' if trend_slope > 0 else 'decreasing',
                    'trend_r2': trend_r2,
                    'trend_strength': 'strong' if trend_r2 > 0.7 else 'moderate' if trend_r2 > 0.3 else 'weak'
                }
                
            elif method == 'autocorrelation':
                # Autocorrelation analysis
                from statsmodels.stats.diagnostic import acorr_ljungbox
                
                # Compute autocorrelation
                autocorr = [data.autocorr(lag=i) for i in range(1, min(21, len(data)//4))]
                
                # Ljung-Box test for randomness
                lb_test = acorr_ljungbox(data.dropna(), lags=min(20, len(data)//4), return_df=True)
                
                result = {
                    'autocorrelations': autocorr,
                    'ljung_box_test': lb_test,
                    'significant_lags': [i for i, p in enumerate(lb_test['lb_pvalue'], 1) if p < 0.05]
                }
            
            logger.info(f"Time series analysis ({method}) completed successfully")
            return result
            
        except Exception as e:
            logger.error(f"Time series analysis failed: {e}")
            raise
    
    def regression_analysis(self, data: pd.DataFrame, 
                          target_col: str, 
                          feature_cols: Optional[List[str]] = None,
                          method: str = 'linear') -> Dict[str, Any]:
        """
        Perform regression analysis.
        
        Args:
            data: Data for regression analysis
            target_col: Target variable column name
            feature_cols: Feature variable column names (if None, all numeric columns except target)
            method: Regression method ('linear', 'polynomial', 'ridge', 'lasso')
            
        Returns:
            Dictionary containing regression results
        """
        try:
            from sklearn.model_selection import train_test_split
            from sklearn.preprocessing import PolynomialFeatures
            from sklearn.linear_model import Ridge, Lasso
            
            # Prepare data
            if feature_cols is None:
                feature_cols = [col for col in data.select_dtypes(include=[np.number]).columns 
                              if col != target_col]
            
            X = data[feature_cols].dropna()
            y = data[target_col].dropna()
            
            # Align data
            common_indices = X.index.intersection(y.index)
            X = X.loc[common_indices]
            y = y.loc[common_indices]
            
            if len(X) < 10:
                raise ValueError("Insufficient data for regression analysis")
            
            # Split data
            X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
            
            # Initialize model
            if method == 'linear':
                model = LinearRegression()
            elif method == 'polynomial':
                poly_features = PolynomialFeatures(degree=2)
                X_train_poly = poly_features.fit_transform(X_train)
                X_test_poly = poly_features.transform(X_test)
                model = LinearRegression()
            elif method == 'ridge':
                model = Ridge(alpha=1.0)
            elif method == 'lasso':
                model = Lasso(alpha=1.0)
            else:
                raise ValueError(f"Unsupported regression method: {method}")
            
            # Train model
            if method == 'polynomial':
                model.fit(X_train_poly, y_train)
                y_pred = model.predict(X_test_poly)
                r2_train = r2_score(y_train, model.predict(X_train_poly))
                r2_test = r2_score(y_test, y_pred)
                coefficients = dict(zip(poly_features.get_feature_names_out(feature_cols), model.coef_))
            else:
                model.fit(X_train, y_train)
                y_pred = model.predict(X_test)
                r2_train = r2_score(y_train, model.predict(X_train))
                r2_test = r2_score(y_test, y_pred)
                coefficients = dict(zip(feature_cols, model.coef_))
            
            # Calculate metrics
            mae = np.mean(np.abs(y_test - y_pred))
            rmse = np.sqrt(np.mean((y_test - y_pred) ** 2))
            
            result = {
                'model': model,
                'r2_train': r2_train,
                'r2_test': r2_test,
                'mae': mae,
                'rmse': rmse,
                'coefficients': coefficients,
                'feature_importance': dict(sorted(coefficients.items(), key=lambda x: abs(x[1]), reverse=True)),
                'method': method,
                'n_features': len(feature_cols),
                'n_samples': len(X)
            }
            
            logger.info(f"Regression analysis ({method}) completed with R² = {r2_test:.3f}")
            return result
            
        except Exception as e:
            logger.error(f"Regression analysis failed: {e}")
            raise
    
    def clustering_analysis(self, data: pd.DataFrame, 
                          n_clusters: Optional[int] = None,
                          method: str = 'kmeans') -> Dict[str, Any]:
        """
        Perform clustering analysis.
        
        Args:
            data: Data for clustering
            n_clusters: Number of clusters (if None, will be determined)
            method: Clustering method ('kmeans', 'hierarchical', 'dbscan')
            
        Returns:
            Dictionary containing clustering results
        """
        try:
            from sklearn.metrics import silhouette_score
            from sklearn.cluster import DBSCAN, AgglomerativeClustering
            
            # Prepare data
            numeric_data = data.select_dtypes(include=[np.number]).dropna()
            
            if len(numeric_data) < 5:
                raise ValueError("Insufficient data for clustering analysis")
            
            # Standardize data
            scaler = StandardScaler()
            scaled_data = scaler.fit_transform(numeric_data)
            
            # Determine optimal number of clusters if not provided
            if n_clusters is None and method == 'kmeans':
                # Use elbow method and silhouette score
                max_clusters = min(10, len(numeric_data) // 5)
                inertias = []
                silhouette_scores = []
                
                for k in range(2, max_clusters + 1):
                    kmeans = KMeans(n_clusters=k, random_state=42)
                    clusters = kmeans.fit_predict(scaled_data)
                    inertias.append(kmeans.inertia_)
                    silhouette_scores.append(silhouette_score(scaled_data, clusters))
                
                # Choose optimal k (highest silhouette score)
                optimal_k = np.argmax(silhouette_scores) + 2
                n_clusters = optimal_k
                logger.info(f"Optimal number of clusters determined: {n_clusters}")
            
            # Perform clustering
            if method == 'kmeans':
                clusterer = KMeans(n_clusters=n_clusters, random_state=42)
                cluster_labels = clusterer.fit_predict(scaled_data)
                cluster_centers = clusterer.cluster_centers_
                
            elif method == 'hierarchical':
                clusterer = AgglomerativeClustering(n_clusters=n_clusters)
                cluster_labels = clusterer.fit_predict(scaled_data)
                cluster_centers = None
                
            elif method == 'dbscan':
                clusterer = DBSCAN(eps=0.3, min_samples=5)
                cluster_labels = clusterer.fit_predict(scaled_data)
                n_clusters = len(set(cluster_labels)) - (1 if -1 in cluster_labels else 0)
                cluster_centers = None
            else:
                raise ValueError(f"Unsupported clustering method: {method}")
            
            # Calculate metrics
            if len(set(cluster_labels)) > 1:
                silhouette_avg = silhouette_score(scaled_data, cluster_labels)
            else:
                silhouette_avg = -1
            
            # Analyze clusters
            cluster_analysis = {}
            for cluster_id in set(cluster_labels):
                if cluster_id != -1:  # Skip noise points in DBSCAN
                    cluster_mask = cluster_labels == cluster_id
                    cluster_data = numeric_data[cluster_mask]
                    
                    cluster_analysis[f'cluster_{cluster_id}'] = {
                        'size': np.sum(cluster_mask),
                        'percentage': np.sum(cluster_mask) / len(cluster_labels) * 100,
                        'statistics': {
                            col: {
                                'mean': cluster_data[col].mean(),
                                'std': cluster_data[col].std(),
                                'min': cluster_data[col].min(),
                                'max': cluster_data[col].max()
                            }
                            for col in cluster_data.columns
                        }
                    }
            
            result = {
                'cluster_labels': cluster_labels,
                'n_clusters': n_clusters,
                'silhouette_score': silhouette_avg,
                'cluster_analysis': cluster_analysis,
                'method': method,
                'scaler': scaler if hasattr(scaler, 'scale_') else None
            }
            
            if cluster_centers is not None:
                result['cluster_centers'] = cluster_centers
            
            logger.info(f"Clustering analysis ({method}) completed with {n_clusters} clusters")
            return result
            
        except Exception as e:
            logger.error(f"Clustering analysis failed: {e}")
            raise
    
    def os_specific_analysis(self, data: pd.DataFrame, 
                           analysis_type: str = 'performance_profile') -> Dict[str, Any]:
        """
        Perform OS-specific analysis on research data.
        
        Args:
            data: Research data containing OS metrics
            analysis_type: Type of OS analysis ('performance_profile', 'resource_utilization', 'latency_analysis')
            
        Returns:
            Dictionary containing OS-specific analysis results
        """
        try:
            if analysis_type == 'performance_profile':
                return self._performance_profile_analysis(data)
            elif analysis_type == 'resource_utilization':
                return self._resource_utilization_analysis(data)
            elif analysis_type == 'latency_analysis':
                return self._latency_analysis(data)
            else:
                raise ValueError(f"Unsupported OS analysis type: {analysis_type}")
                
        except Exception as e:
            logger.error(f"OS-specific analysis failed: {e}")
            raise
    
    def _performance_profile_analysis(self, data: pd.DataFrame) -> Dict[str, Any]:
        """Analyze performance profile of OS metrics."""
        performance_metrics = []
        
        # Common performance metric columns
        perf_columns = [col for col in data.columns if any(keyword in col.lower() 
                       for keyword in ['cpu', 'memory', 'throughput', 'latency', 'response_time'])]
        
        if not perf_columns:
            # Use all numeric columns if no specific performance columns found
            perf_columns = data.select_dtypes(include=[np.number]).columns.tolist()
        
        analysis = {}
        
        for metric in perf_columns:
            if data[metric].dtype in ['int64', 'float64']:
                analysis[metric] = {
                    'mean': data[metric].mean(),
                    'median': data[metric].median(),
                    'std': data[metric].std(),
                    'cv': data[metric].std() / data[metric].mean() if data[metric].mean() != 0 else float('inf'),
                    'percentile_95': data[metric].quantile(0.95),
                    'percentile_99': data[metric].quantile(0.99),
                    'stability': 'stable' if data[metric].std() / data[metric].mean() < 0.1 else 'volatile'
                }
        
        # Overall performance score
        cv_scores = [analysis[metric]['cv'] for metric in analysis if not np.isinf(analysis[metric]['cv'])]
        avg_cv = np.mean(cv_scores) if cv_scores else 0
        
        overall_profile = {
            'metrics_analyzed': len(analysis),
            'average_variability': avg_cv,
            'overall_stability': 'stable' if avg_cv < 0.2 else 'moderate' if avg_cv < 0.5 else 'volatile',
            'performance_quality': self._assess_performance_quality(analysis)
        }
        
        return {
            'individual_metrics': analysis,
            'overall_profile': overall_profile,
            'recommendations': self._generate_performance_recommendations(analysis)
        }
    
    def _resource_utilization_analysis(self, data: pd.DataFrame) -> Dict[str, Any]:
        """Analyze resource utilization patterns."""
        resource_columns = [col for col in data.columns if any(keyword in col.lower() 
                           for keyword in ['cpu', 'memory', 'disk', 'network', 'utilization'])]
        
        utilization_analysis = {}
        
        for resource in resource_columns:
            if data[resource].dtype in ['int64', 'float64']:
                utilization_analysis[resource] = {
                    'peak_utilization': data[resource].max(),
                    'average_utilization': data[resource].mean(),
                    'utilization_variance': data[resource].var(),
                    'resource_stress_indicators': {
                        'high_utilization_time': (data[resource] > data[resource].quantile(0.9)).sum(),
                        'critical_utilization_time': (data[resource] > data[resource].quantile(0.95)).sum(),
                        'underutilized_time': (data[resource] < data[resource].quantile(0.1)).sum()
                    }
                }
        
        # Resource correlation analysis
        if len(resource_columns) > 1:
            resource_corr = data[resource_columns].corr()
            high_correlations = []
            
            for i in range(len(resource_corr.columns)):
                for j in range(i+1, len(resource_corr.columns)):
                    corr_value = resource_corr.iloc[i, j]
                    if abs(corr_value) > 0.7:
                        high_correlations.append({
                            'resource1': resource_corr.columns[i],
                            'resource2': resource_corr.columns[j],
                            'correlation': corr_value
                        })
        
        return {
            'individual_resources': utilization_analysis,
            'resource_correlations': high_correlations if len(resource_columns) > 1 else [],
            'bottleneck_analysis': self._identify_resource_bottlenecks(utilization_analysis),
            'optimization_suggestions': self._suggest_resource_optimizations(utilization_analysis)
        }
    
    def _latency_analysis(self, data: pd.DataFrame) -> Dict[str, Any]:
        """Analyze latency patterns and distributions."""
        latency_columns = [col for col in data.columns if any(keyword in col.lower() 
                          for keyword in ['latency', 'delay', 'response_time', 'waiting_time'])]
        
        latency_analysis = {}
        
        for latency_metric in latency_columns:
            if data[latency_metric].dtype in ['int64', 'float64']:
                latency_values = data[latency_metric].dropna()
                
                # Statistical analysis
                latency_analysis[latency_metric] = {
                    'distribution_stats': {
                        'mean': latency_values.mean(),
                        'median': latency_values.median(),
                        'std': latency_values.std(),
                        'skewness': stats.skew(latency_values),
                        'kurtosis': stats.kurtosis(latency_values)
                    },
                    'percentile_analysis': {
                        'p50': latency_values.quantile(0.50),
                        'p95': latency_values.quantile(0.95),
                        'p99': latency_values.quantile(0.99),
                        'p99_9': latency_values.quantile(0.999)
                    },
                    'performance_indicators': {
                        'high_latency_incidents': (latency_values > latency_values.quantile(0.95)).sum(),
                        'outliers_detected': self._detect_latency_outliers(latency_values),
                        'service_level_compliance': self._assess_sla_compliance(latency_values)
                    }
                }
        
        return {
            'latency_metrics': latency_analysis,
            'overall_latency_health': self._assess_latency_health(latency_analysis),
            'optimization_priorities': self._prioritize_latency_optimizations(latency_analysis)
        }
    
    def _assess_performance_quality(self, analysis: Dict) -> str:
        """Assess overall performance quality."""
        total_metrics = len(analysis)
        stable_metrics = sum(1 for metric in analysis.values() 
                           if metric['stability'] == 'stable')
        
        stability_ratio = stable_metrics / total_metrics if total_metrics > 0 else 0
        
        if stability_ratio >= 0.8:
            return "Excellent"
        elif stability_ratio >= 0.6:
            return "Good"
        elif stability_ratio >= 0.4:
            return "Fair"
        else:
            return "Poor"
    
    def _generate_performance_recommendations(self, analysis: Dict) -> List[str]:
        """Generate performance optimization recommendations."""
        recommendations = []
        
        for metric, stats in analysis.items():
            if stats['cv'] > 0.5:  # High variability
                recommendations.append(f"High variability detected in {metric}. Consider investigating underlying causes.")
            
            if stats['percentile_99'] > 2 * stats['mean']:  # High tail latency
                recommendations.append(f"Significant tail latency in {metric}. Review for optimization opportunities.")
            
            if stats['stability'] == 'volatile':
                recommendations.append(f"{metric} shows volatile behavior. Consider capacity planning or optimization.")
        
        return recommendations
    
    def _identify_resource_bottlenecks(self, utilization_analysis: Dict) -> List[str]:
        """Identify potential resource bottlenecks."""
        bottlenecks = []
        
        for resource, analysis in utilization_analysis.items():
            peak = analysis['peak_utilization']
            avg = analysis['average_utilization']
            
            if peak > 0.9:  # Peak utilization > 90%
                bottlenecks.append(f"{resource} shows potential bottleneck with {peak:.1%} peak utilization")
            elif peak - avg > 0.3:  # Large gap between peak and average
                bottlenecks.append(f"{resource} shows capacity mismatch with {peak - avg:.1%} gap between peak and average")
        
        return bottlenecks
    
    def _suggest_resource_optimizations(self, utilization_analysis: Dict) -> List[str]:
        """Suggest resource optimization strategies."""
        suggestions = []
        
        for resource, analysis in utilization_analysis.items():
            high_util_time = analysis['resource_stress_indicators']['high_utilization_time']
            underutil_time = analysis['resource_stress_indicators']['underutilized_time']
            
            if high_util_time > len(analysis) * 0.1:  # High utilization > 10% of time
                suggestions.append(f"Consider scaling up {resource} or optimizing workloads")
            elif underutil_time > len(analysis) * 0.5:  # Underutilized > 50% of time
                suggestions.append(f"Consider scaling down {resource} to reduce costs")
        
        return suggestions
    
    def _detect_latency_outliers(self, latency_values: pd.Series) -> int:
        """Detect latency outliers using IQR method."""
        Q1 = latency_values.quantile(0.25)
        Q3 = latency_values.quantile(0.75)
        IQR = Q3 - Q1
        outlier_threshold = Q3 + 1.5 * IQR
        
        outliers = (latency_values > outlier_threshold).sum()
        return int(outliers)
    
    def _assess_sla_compliance(self, latency_values: pd.Series, sla_threshold: Optional[float] = None) -> Dict[str, Any]:
        """Assess SLA compliance for latency metrics."""
        if sla_threshold is None:
            sla_threshold = latency_values.quantile(0.95)  # Use 95th percentile as default SLA
        
        compliant_count = (latency_values <= sla_threshold).sum()
        total_count = len(latency_values)
        compliance_rate = compliant_count / total_count if total_count > 0 else 0
        
        return {
            'sla_threshold': sla_threshold,
            'compliance_rate': compliance_rate,
            'violations': total_count - compliant_count,
            'status': 'compliant' if compliance_rate >= 0.95 else 'non-compliant'
        }
    
    def _assess_latency_health(self, latency_analysis: Dict) -> str:
        """Assess overall latency health."""
        total_metrics = len(latency_analysis)
        healthy_metrics = 0
        
        for metric_analysis in latency_analysis.values():
            # Check if metrics show reasonable skewness and kurtosis
            skewness = abs(metric_analysis['distribution_stats']['skewness'])
            kurtosis = abs(metric_analysis['distribution_stats']['kurtosis'])
            
            # Healthy distribution should not be too skewed or have extreme kurtosis
            if skewness < 2 and kurtosis < 7:
                healthy_metrics += 1
        
        health_ratio = healthy_metrics / total_metrics if total_metrics > 0 else 0
        
        if health_ratio >= 0.8:
            return "Healthy"
        elif health_ratio >= 0.6:
            return "Moderate"
        else:
            return "Concerning"
    
    def _prioritize_latency_optimizations(self, latency_analysis: Dict) -> List[str]:
        """Prioritize latency optimization efforts."""
        priorities = []
        
        for metric, analysis in latency_analysis.items():
            # Prioritize based on skewness and outlier count
            skewness = analysis['distribution_stats']['skewness']
            outlier_count = analysis['performance_indicators']['outliers_detected']
            p99_latency = analysis['percentile_analysis']['p99']
            mean_latency = analysis['distribution_stats']['mean']
            
            priority_score = 0
            
            # High skewness indicates distribution problems
            if abs(skewness) > 2:
                priority_score += 2
            
            # Many outliers indicate instability
            if outlier_count > len(analysis) * 0.05:  # More than 5% outliers
                priority_score += 2
            
            # High tail latency relative to mean
            if p99_latency > 3 * mean_latency:
                priority_score += 1
            
            if priority_score >= 3:
                priorities.append(f"HIGH: {metric} - High skewness, outliers, or tail latency issues")
            elif priority_score >= 2:
                priorities.append(f"MEDIUM: {metric} - Some distribution or latency concerns")
            else:
                priorities.append(f"LOW: {metric} - Relatively healthy latency profile")
        
        return sorted(priorities, key=lambda x: x.startswith('HIGH'), reverse=True)
    
    def save_analysis_results(self, results: Dict[str, Any], 
                            output_path: Union[str, Path],
                            format: str = 'json') -> None:
        """
        Save analysis results to file.
        
        Args:
            results: Analysis results to save
            output_path: Path to save results
            format: Output format ('json', 'pickle', 'csv')
        """
        try:
            output_path = Path(output_path)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            if format == 'json':
                import json
                with open(output_path, 'w') as f:
                    json.dump(results, f, indent=2, default=str)
                    
            elif format == 'pickle':
                import pickle
                with open(output_path, 'wb') as f:
                    pickle.dump(results, f)
                    
            elif format == 'csv':
                # Convert DataFrame results to CSV
                if 'correlation_matrix' in results:
                    results['correlation_matrix'].to_csv(output_path)
                else:
                    # Save as JSON for non-DataFrame results
                    import json
                    with open(output_path.with_suffix('.json'), 'w') as f:
                        json.dump(results, f, indent=2, default=str)
            else:
                raise ValueError(f"Unsupported output format: {format}")
            
            logger.info(f"Analysis results saved to {output_path}")
            
        except Exception as e:
            logger.error(f"Failed to save analysis results: {e}")
            raise

def create_analyzer(config: Optional[Dict] = None) -> OSDataAnalyzer:
    """
    Factory function to create an OSDataAnalyzer instance.
    
    Args:
        config: Configuration dictionary
        
    Returns:
        OSDataAnalyzer instance
    """
    return OSDataAnalyzer(config=config)

def quick_analysis(data: pd.DataFrame, 
                  analysis_types: List[str] = ['basic_statistics', 'correlation_analysis']) -> Dict[str, Any]:
    """
    Perform quick analysis on data using default settings.
    
    Args:
        data: Data to analyze
        analysis_types: List of analysis types to perform
        
    Returns:
        Dictionary containing analysis results
    """
    analyzer = create_analyzer()
    results = {}
    
    for analysis_type in analysis_types:
        try:
            if analysis_type == 'basic_statistics':
                results['basic_statistics'] = analyzer.basic_statistics(data)
            elif analysis_type == 'correlation_analysis':
                results['correlation_analysis'] = analyzer.correlation_analysis(data)
            elif analysis_type == 'performance_profile':
                results['performance_profile'] = analyzer.os_specific_analysis(data, 'performance_profile')
            elif analysis_type == 'resource_utilization':
                results['resource_utilization'] = analyzer.os_specific_analysis(data, 'resource_utilization')
            elif analysis_type == 'latency_analysis':
                results['latency_analysis'] = analyzer.os_specific_analysis(data, 'latency_analysis')
                
        except Exception as e:
            logger.error(f"Failed to perform {analysis_type}: {e}")
            results[analysis_type] = {'error': str(e)}
    
    return results