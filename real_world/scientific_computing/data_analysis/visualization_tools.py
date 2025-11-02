"""
Data Analysis and Visualization Tools for Scientific Computing Education
=======================================================================

This module provides comprehensive tools for data analysis and visualization:
- Statistical analysis functions
- Data visualization and plotting
- Time series analysis
- Signal processing tools
- Data preprocessing and cleaning

Author: Scientific Computing Education Team
"""

import numpy as np
import matplotlib.pyplot as plt
from typing import Dict, List, Tuple, Optional, Union, Callable
from scipy import stats
from scipy.signal import find_peaks


class DataAnalysis:
    """Core data analysis functions for scientific data."""
    
    @staticmethod
    def descriptive_statistics(data: np.ndarray, print_output: bool = True) -> Dict:
        """
        Calculate comprehensive descriptive statistics.
        
        Args:
            data: Input data array
            print_output: Whether to print results
            
        Returns:
            Dictionary containing all statistics
        """
        data = np.array(data).flatten()
        n = len(data)
        
        stats_dict = {
            'count': n,
            'mean': np.mean(data),
            'median': np.median(data),
            'mode': None,  # Will calculate if needed
            'std': np.std(data, ddof=1),
            'var': np.var(data, ddof=1),
            'min': np.min(data),
            'max': np.max(data),
            'range': np.max(data) - np.min(data),
            'q1': np.percentile(data, 25),
            'q3': np.percentile(data, 75),
            'iqr': np.percentile(data, 75) - np.percentile(data, 25),
            'skewness': stats.skew(data),
            'kurtosis': stats.kurtosis(data),
            'cv': np.std(data, ddof=1) / np.mean(data) if np.mean(data) != 0 else np.inf
        }
        
        # Calculate mode
        try:
            mode_result = stats.mode(data, keepdims=False)
            stats_dict['mode'] = mode_result.mode
        except:
            stats_dict['mode'] = None
        
        if print_output:
            print("Descriptive Statistics")
            print("=" * 25)
            print(f"Count:     {stats_dict['count']}")
            print(f"Mean:      {stats_dict['mean']:.4f}")
            print(f"Median:    {stats_dict['median']:.4f}")
            print(f"Mode:      {stats_dict['mode']:.4f if stats_dict['mode'] is not None else 'N/A'}")
            print(f"Std Dev:   {stats_dict['std']:.4f}")
            print(f"Variance:  {stats_dict['var']:.4f}")
            print(f"Min:       {stats_dict['min']:.4f}")
            print(f"Max:       {stats_dict['max']:.4f}")
            print(f"Range:     {stats_dict['range']:.4f}")
            print(f"Q1:        {stats_dict['q1']:.4f}")
            print(f"Q3:        {stats_dict['q3']:.4f}")
            print(f"IQR:       {stats_dict['iqr']:.4f}")
            print(f"Skewness:  {stats_dict['skewness']:.4f}")
            print(f"Kurtosis:  {stats_dict['kurtosis']:.4f}")
            print(f"Coeff Var: {stats_dict['cv']:.4f}")
        
        return stats_dict
    
    @staticmethod
    def correlation_analysis(x: np.ndarray, y: np.ndarray, method: str = 'pearson',
                           alpha: float = 0.05) -> Dict:
        """
        Perform correlation analysis between two variables.
        
        Args:
            x: First variable
            y: Second variable
            method: Correlation method ('pearson', 'spearman', 'kendall')
            alpha: Significance level
            
        Returns:
            Dictionary containing correlation results
        """
        if len(x) != len(y):
            raise ValueError("x and y must have same length")
        
        x = np.array(x)
        y = np.array(y)
        
        if method == 'pearson':
            corr_coef, p_value = stats.pearsonr(x, y)
        elif method == 'spearman':
            corr_coef, p_value = stats.spearmanr(x, y)
        elif method == 'kendall':
            corr_coef, p_value = stats.kendalltau(x, y)
        else:
            raise ValueError("Method must be 'pearson', 'spearman', or 'kendall'")
        
        # Calculate confidence interval
        n = len(x)
        z = np.arctanh(corr_coef)  # Fisher's z-transformation
        se = 1 / np.sqrt(n - 3)
        z_crit = stats.norm.ppf(1 - alpha/2)
        
        ci_lower = np.tanh(z - z_crit * se)
        ci_upper = np.tanh(z + z_crit * se)
        
        result = {
            'correlation': corr_coef,
            'p_value': p_value,
            'n': n,
            'method': method,
            'significant': p_value < alpha,
            'ci_lower': ci_lower,
            'ci_upper': ci_upper,
            'interpretation': DataAnalysis._interpret_correlation(corr_coef)
        }
        
        print(f"{method.title()} Correlation Analysis")
        print("=" * 35)
        print(f"Correlation coefficient: {corr_coef:.4f}")
        print(f"P-value: {p_value:.6f}")
        print(f"95% CI: [{ci_lower:.4f}, {ci_upper:.4f}]")
        print(f"Significant: {result['significant']}")
        print(f"Interpretation: {result['interpretation']}")
        
        return result
    
    @staticmethod
    def _interpret_correlation(r: float) -> str:
        """Interpret correlation coefficient magnitude."""
        r = abs(r)
        if r < 0.1:
            return "negligible"
        elif r < 0.3:
            return "weak"
        elif r < 0.5:
            return "moderate"
        elif r < 0.7:
            return "strong"
        else:
            return "very strong"
    
    @staticmethod
    def hypothesis_testing(data: np.ndarray, test_type: str = 'normality',
                          alpha: float = 0.05, **kwargs) -> Dict:
        """
        Perform statistical hypothesis tests.
        
        Args:
            data: Input data
            test_type: Type of test ('normality', 't_test', 'chi_square', 'anova')
            alpha: Significance level
            **kwargs: Additional arguments for specific tests
            
        Returns:
            Dictionary containing test results
        """
        data = np.array(data)
        
        if test_type == 'normality':
            stat, p_value = stats.shapiro(data)
            test_name = "Shapiro-Wilk"
        elif test_type == 't_test':
            # One-sample t-test
            mu = kwargs.get('mu', 0)
            stat, p_value = stats.ttest_1samp(data, mu)
            test_name = f"One-sample t-test (μ={mu})"
        elif test_type == 'chi_square':
            # Chi-square goodness of fit
            expected = kwargs.get('expected')
            if expected is None:
                # Use uniform distribution as default
                n_bins = kwargs.get('n_bins', 10)
                observed, _ = np.histogram(data, bins=n_bins)
                expected = np.full(n_bins, len(data) / n_bins)
            else:
                observed = data
            stat, p_value = stats.chisquare(observed, expected)
            test_name = "Chi-square goodness of fit"
        elif test_type == 'anova':
            # One-way ANOVA
            groups = kwargs.get('groups')
            if groups is None:
                raise ValueError("Groups required for ANOVA")
            stat, p_value = stats.f_oneway(*groups)
            test_name = "One-way ANOVA"
        else:
            raise ValueError("Test type must be 'normality', 't_test', 'chi_square', or 'anova'")
        
        result = {
            'test_name': test_name,
            'statistic': stat,
            'p_value': p_value,
            'alpha': alpha,
            'significant': p_value < alpha,
            'conclusion': 'Reject null hypothesis' if p_value < alpha else 'Fail to reject null hypothesis'
        }
        
        print(f"{test_name}")
        print("=" * len(test_name))
        print(f"Test statistic: {stat:.4f}")
        print(f"P-value: {p_value:.6f}")
        print(f"Alpha: {alpha}")
        print(f"Result: {result['conclusion']}")
        
        return result
    
    @staticmethod
    def outlier_detection(data: np.ndarray, method: str = 'iqr',
                         threshold: float = 1.5) -> Dict:
        """
        Detect outliers in data using various methods.
        
        Args:
            data: Input data array
            method: Detection method ('iqr', 'zscore', 'modified_zscore')
            threshold: Threshold for outlier detection
            
        Returns:
            Dictionary containing outlier information
        """
        data = np.array(data)
        n = len(data)
        
        if method == 'iqr':
            q1, q3 = np.percentile(data, [25, 75])
            iqr = q3 - q1
            lower_bound = q1 - threshold * iqr
            upper_bound = q3 + threshold * iqr
            outliers_mask = (data < lower_bound) | (data > upper_bound)
            
        elif method == 'zscore':
            z_scores = np.abs(stats.zscore(data))
            outliers_mask = z_scores > threshold
            
        elif method == 'modified_zscore':
            median = np.median(data)
            mad = np.median(np.abs(data - median))
            modified_z_scores = 0.6745 * (data - median) / mad
            outliers_mask = np.abs(modified_z_scores) > threshold
            
        else:
            raise ValueError("Method must be 'iqr', 'zscore', or 'modified_zscore'")
        
        outliers = data[outliers_mask]
        outlier_indices = np.where(outliers_mask)[0]
        
        result = {
            'method': method,
            'outliers': outliers,
            'outlier_indices': outlier_indices,
            'n_outliers': len(outliers),
            'outlier_percentage': len(outliers) / n * 100,
            'outliers_mask': outliers_mask
        }
        
        print(f"Outlier Detection ({method.title()})")
        print("=" * 35)
        print(f"Number of outliers: {len(outliers)}")
        print(f"Percentage: {len(outliers)/n*100:.2f}%")
        if len(outliers) > 0:
            print(f"Outlier values: {outliers}")
        
        return result


class TimeSeriesAnalysis:
    """Time series analysis and forecasting tools."""
    
    @staticmethod
    def moving_average(data: np.ndarray, window: int) -> np.ndarray:
        """
        Calculate moving average.
        
        Args:
            data: Time series data
            window: Window size
            
        Returns:
            Moving average array
        """
        if window > len(data):
            raise ValueError("Window size cannot be larger than data length")
        
        moving_avg = np.convolve(data, np.ones(window)/window, mode='valid')
        return moving_avg
    
    @staticmethod
    def exponential_smoothing(data: np.ndarray, alpha: float) -> np.ndarray:
        """
        Perform exponential smoothing.
        
        Args:
            data: Time series data
            alpha: Smoothing parameter (0 < alpha <= 1)
            
        Returns:
            Smoothed data array
        """
        if not 0 < alpha <= 1:
            raise ValueError("Alpha must be between 0 and 1")
        
        smoothed = np.zeros(len(data))
        smoothed[0] = data[0]
        
        for i in range(1, len(data)):
            smoothed[i] = alpha * data[i] + (1 - alpha) * smoothed[i-1]
        
        return smoothed
    
    @staticmethod
    def autocorrelation(data: np.ndarray, max_lags: Optional[int] = None) -> np.ndarray:
        """
        Calculate autocorrelation function.
        
        Args:
            data: Time series data
            max_lags: Maximum number of lags to calculate
            
        Returns:
            Autocorrelation array
        """
        data = np.array(data)
        n = len(data)
        
        if max_lags is None:
            max_lags = min(n//2, 50)
        
        # Center the data
        data_centered = data - np.mean(data)
        
        # Calculate autocorrelation
        autocorr = np.correlate(data_centered, data_centered, mode='full')
        autocorr = autocorr[n-1:n-1+max_lags+1]
        autocorr = autocorr / autocorr[0]  # Normalize
        
        return autocorr
    
    @staticmethod
    def trend_analysis(data: np.ndarray, time_points: Optional[np.ndarray] = None) -> Dict:
        """
        Analyze trend in time series data.
        
        Args:
            data: Time series data
            time_points: Time points (if None, uses indices)
            
        Returns:
            Dictionary containing trend analysis results
        """
        if time_points is None:
            time_points = np.arange(len(data))
        
        # Linear regression for trend
        slope, intercept, r_value, p_value, std_err = stats.linregress(time_points, data)
        
        # Calculate trend line
        trend_line = slope * time_points + intercept
        
        # Determine trend direction
        if p_value < 0.05:
            if slope > 0:
                direction = "increasing"
            else:
                direction = "decreasing"
            significance = "significant"
        else:
            direction = "no clear trend"
            significance = "not significant"
        
        result = {
            'slope': slope,
            'intercept': intercept,
            'r_squared': r_value**2,
            'p_value': p_value,
            'trend_line': trend_line,
            'direction': direction,
            'significance': significance
        }
        
        print("Trend Analysis")
        print("=" * 15)
        print(f"Slope: {slope:.6f}")
        print(f"R-squared: {r_value**2:.4f}")
        print(f"P-value: {p_value:.6f}")
        print(f"Direction: {direction} ({significance})")
        
        return result
    
    @staticmethod
    def seasonal_decomposition(data: np.ndarray, period: int = 12) -> Dict:
        """
        Simple seasonal decomposition using moving averages.
        
        Args:
            data: Time series data
            period: Seasonal period
            
        Returns:
            Dictionary containing decomposed components
        """
        n = len(data)
        
        # Trend using centered moving average
        if period % 2 == 0:
            trend = DataAnalysis._centered_moving_average(data, period)
        else:
            trend = DataAnalysis._moving_average(data, period)
        
        # Detrended data
        detrended = data - trend
        
        # Seasonal component
        seasonal = np.zeros(period)
        for i in range(period):
            seasonal_values = detrended[i::period]
            seasonal[i] = np.mean(seasonal_values)
        
        # Expand seasonal to full length
        seasonal_full = np.tile(seasonal, n // period + 1)[:n]
        
        # Residual
        residual = data - trend - seasonal_full
        
        return {
            'trend': trend,
            'seasonal': seasonal,
            'seasonal_full': seasonal_full,
            'residual': residual,
            'original': data,
            'period': period
        }
    
    @staticmethod
    def _moving_average(data: np.ndarray, window: int) -> np.ndarray:
        """Helper function for moving average."""
        padded = np.pad(data, (window//2, window//2), mode='edge')
        return np.convolve(padded, np.ones(window)/window, mode='valid')
    
    @staticmethod
    def _centered_moving_average(data: np.ndarray, window: int) -> np.ndarray:
        """Helper function for centered moving average."""
        if window % 2 == 0:
            # For even window, use two moving averages
            ma1 = TimeSeriesAnalysis._moving_average(data, window)
            ma2 = TimeSeriesAnalysis._moving_average(data, window-1)
            return (ma1[window//2:-window//2+1] + ma2[window//2-1:-window//2+1]) / 2
        else:
            return TimeSeriesAnalysis._moving_average(data, window)


class SignalProcessing:
    """Signal processing utilities for scientific computing."""
    
    @staticmethod
    def filter_lowpass(data: np.ndarray, cutoff_freq: float, sampling_rate: float,
                      order: int = 5) -> np.ndarray:
        """
        Apply lowpass filter using Butterworth approximation.
        
        Args:
            data: Input signal
            cutoff_freq: Cutoff frequency in Hz
            sampling_rate: Sampling rate in Hz
            order: Filter order
            
        Returns:
            Filtered signal
        """
        # Normalize frequency
        nyquist = sampling_rate / 2
        normalized_freq = cutoff_freq / nyquist
        
        if normalized_freq >= 1:
            return data  # No filtering needed
        
        # Simple Butterworth approximation using polynomial filtering
        # This is a simplified implementation for educational purposes
        
        # Create filter coefficients
        if order == 1:
            # First-order lowpass: y[n] = α*x[n] + (1-α)*y[n-1]
            alpha = 1 - np.exp(-2 * np.pi * normalized_freq)
        else:
            # Higher-order approximation
            alpha = 1 - np.exp(-2 * np.pi * normalized_freq / order)
        
        # Apply filter
        filtered = np.zeros_like(data)
        filtered[0] = alpha * data[0]
        
        for i in range(1, len(data)):
            filtered[i] = alpha * data[i] + (1 - alpha) * filtered[i-1]
        
        return filtered
    
    @staticmethod
    def filter_highpass(data: np.ndarray, cutoff_freq: float, sampling_rate: float,
                       order: int = 5) -> np.ndarray:
        """
        Apply highpass filter.
        
        Args:
            data: Input signal
            cutoff_freq: Cutoff frequency in Hz
            sampling_rate: Sampling rate in Hz
            order: Filter order
            
        Returns:
            Filtered signal
        """
        # Highpass = original - lowpass(original)
        lowpass = SignalProcessing.filter_lowpass(data, cutoff_freq, sampling_rate, order)
        return data - lowpass
    
    @staticmethod
    def detect_peaks(data: np.ndarray, height: Optional[float] = None,
                    distance: int = 1) -> Dict:
        """
        Detect peaks in signal data.
        
        Args:
            data: Input signal
            height: Minimum peak height
            distance: Minimum distance between peaks
            
        Returns:
            Dictionary containing peak information
        """
        peaks, properties = find_peaks(data, height=height, distance=distance)
        
        result = {
            'peaks': peaks,
            'peak_values': data[peaks],
            'n_peaks': len(peaks),
            'properties': properties
        }
        
        print(f"Peak Detection")
        print("=" * 15)
        print(f"Number of peaks: {len(peaks)}")
        if len(peaks) > 0:
            print(f"Peak locations: {peaks}")
            print(f"Peak values: {data[peaks]}")
        
        return result
    
    @staticmethod
    def power_spectral_density(data: np.ndarray, sampling_rate: float = 1.0) -> Dict:
        """
        Calculate power spectral density using Welch's method.
        
        Args:
            data: Input signal
            sampling_rate: Sampling rate in Hz
            
        Returns:
            Dictionary containing frequency and power data
        """
        # Simple periodogram (Welch's method approximation)
        n = len(data)
        window_size = min(256, n // 4)
        
        # Apply window and compute FFT
        window = np.hanning(window_size)
        
        # Zero-pad if necessary
        if n < window_size:
            padded_data = np.pad(data, (0, window_size - n), mode='constant')
            padded_window = np.pad(window, (0, n), mode='constant')
        else:
            padded_data = data[:window_size] * window
        
        # Compute FFT
        fft_data = np.fft.fft(padded_data)
        power_spectrum = np.abs(fft_data)**2 / (np.sum(window**2) * sampling_rate)
        
        # Create frequency axis
        frequencies = np.fft.fftfreq(len(padded_data), 1/sampling_rate)
        
        # Take positive frequencies only
        positive_freq_mask = frequencies >= 0
        frequencies = frequencies[positive_freq_mask]
        power_spectrum = power_spectrum[positive_freq_mask]
        
        # Double the power for positive frequencies (except DC)
        power_spectrum[1:] *= 2
        
        return {
            'frequencies': frequencies,
            'power': power_spectrum,
            'sampling_rate': sampling_rate
        }


class DataVisualization:
    """Data visualization tools for scientific data."""
    
    @staticmethod
    def plot_distribution(data: np.ndarray, bins: int = 30, 
                         figsize: Tuple[int, int] = (12, 8)) -> None:
        """
        Create comprehensive distribution plots.
        
        Args:
            data: Input data
            bins: Number of histogram bins
            figsize: Figure size
        """
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=figsize)
        
        # Histogram
        ax1.hist(data, bins=bins, alpha=0.7, density=True, color='skyblue', edgecolor='black')
        ax1.set_title('Histogram')
        ax1.set_xlabel('Value')
        ax1.set_ylabel('Density')
        
        # Box plot
        ax2.boxplot(data)
        ax2.set_title('Box Plot')
        ax2.set_ylabel('Value')
        
        # Q-Q plot
        stats.probplot(data, dist="norm", plot=ax3)
        ax3.set_title('Q-Q Plot')
        
        # Violin plot
        ax4.violinplot(data)
        ax4.set_title('Violin Plot')
        ax4.set_ylabel('Value')
        
        plt.tight_layout()
        plt.show()
    
    @staticmethod
    def plot_time_series(time: np.ndarray, data: np.ndarray,
                        trend: Optional[np.ndarray] = None,
                        seasonal: Optional[np.ndarray] = None,
                        figsize: Tuple[int, int] = (12, 6)) -> None:
        """
        Plot time series data with optional components.
        
        Args:
            time: Time points
            data: Time series data
            trend: Trend component
            seasonal: Seasonal component
            figsize: Figure size
        """
        fig, axes = plt.subplots(len([d for d in [data, trend, seasonal] if d is not None]),
                               1, figsize=figsize, sharex=True)
        
        plot_idx = 0
        
        if data is not None:
            axes[plot_idx].plot(time, data, label='Original Data', color='blue')
            axes[plot_idx].set_title('Time Series')
            axes[plot_idx].set_ylabel('Value')
            axes[plot_idx].legend()
            axes[plot_idx].grid(True, alpha=0.3)
            plot_idx += 1
        
        if trend is not None:
            axes[plot_idx].plot(time, trend, label='Trend', color='red')
            axes[plot_idx].set_title('Trend Component')
            axes[plot_idx].set_ylabel('Value')
            axes[plot_idx].legend()
            axes[plot_idx].grid(True, alpha=0.3)
            plot_idx += 1
        
        if seasonal is not None:
            axes[plot_idx].plot(time, seasonal, label='Seasonal', color='green')
            axes[plot_idx].set_title('Seasonal Component')
            axes[plot_idx].set_ylabel('Value')
            axes[plot_idx].set_xlabel('Time')
            axes[plot_idx].legend()
            axes[plot_idx].grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.show()
    
    @staticmethod
    def plot_correlation_matrix(data: np.ndarray, 
                               feature_names: Optional[List[str]] = None,
                               figsize: Tuple[int, int] = (10, 8)) -> None:
        """
        Plot correlation matrix heatmap.
        
        Args:
            data: Data matrix
            feature_names: Names of features
            figsize: Figure size
        """
        if data.ndim == 1:
            raise ValueError("Data must be 2D for correlation matrix")
        
        # Calculate correlation matrix
        corr_matrix = np.corrcoef(data.T)
        
        # Create figure
        fig, ax = plt.subplots(figsize=figsize)
        
        # Create heatmap
        im = ax.imshow(corr_matrix, cmap='RdBu_r', vmin=-1, vmax=1)
        
        # Add colorbar
        plt.colorbar(im, ax=ax)
        
        # Set ticks and labels
        if feature_names is None:
            feature_names = [f'Feature {i}' for i in range(data.shape[1])]
        
        ax.set_xticks(range(len(feature_names)))
        ax.set_yticks(range(len(feature_names)))
        ax.set_xticklabels(feature_names, rotation=45)
        ax.set_yticklabels(feature_names)
        
        # Add correlation values as text
        for i in range(len(feature_names)):
            for j in range(len(feature_names)):
                text = ax.text(j, i, f'{corr_matrix[i, j]:.2f}',
                             ha="center", va="center", color="black" if abs(corr_matrix[i, j]) < 0.5 else "white")
        
        ax.set_title('Correlation Matrix')
        plt.tight_layout()
        plt.show()
    
    @staticmethod
    def plot_3d_scatter(data: np.ndarray, labels: Optional[np.ndarray] = None,
                       feature_indices: Tuple[int, int, int] = (0, 1, 2),
                       figsize: Tuple[int, int] = (10, 8)) -> None:
        """
        Create 3D scatter plot.
        
        Args:
            data: Data matrix (samples x features)
            labels: Optional labels for coloring points
            feature_indices: Indices of features to plot
            figsize: Figure size
        """
        fig = plt.figure(figsize=figsize)
        ax = fig.add_subplot(111, projection='3d')
        
        x, y, z = data[:, feature_indices[0]], data[:, feature_indices[1]], data[:, feature_indices[2]]
        
        if labels is not None:
            scatter = ax.scatter(x, y, z, c=labels, cmap='viridis')
            plt.colorbar(scatter, ax=ax, shrink=0.5, aspect=5)
        else:
            ax.scatter(x, y, z)
        
        ax.set_xlabel(f'Feature {feature_indices[0]}')
        ax.set_ylabel(f'Feature {feature_indices[1]}')
        ax.set_zlabel(f'Feature {feature_indices[2]}')
        ax.set_title('3D Scatter Plot')
        
        plt.tight_layout()
        plt.show()


def demo_data_analysis():
    """Demonstration of data analysis and visualization capabilities."""
    print("Data Analysis and Visualization - Educational Examples")
    print("=" * 55)
    
    # Generate sample data
    np.random.seed(42)
    normal_data = np.random.normal(100, 15, 1000)
    skewed_data = np.random.exponential(2, 1000) + 10
    
    print("\n1. Descriptive Statistics (Normal Distribution):")
    DataAnalysis.descriptive_statistics(normal_data)
    
    print("\n\n2. Outlier Detection:")
    outlier_result = DataAnalysis.outlier_detection(normal_data, method='iqr')
    
    print("\n\n3. Time Series Analysis:")
    # Generate synthetic time series
    time = np.linspace(0, 100, 500)
    trend = 0.1 * time + np.random.normal(0, 1, 500)
    seasonal = 5 * np.sin(2 * np.pi * time / 20)
    noise = np.random.normal(0, 2, 500)
    time_series = trend + seasonal + noise
    
    trend_result = TimeSeriesAnalysis.trend_analysis(time_series, time)
    decomp_result = TimeSeriesAnalysis.seasonal_decomposition(time_series, period=20)
    
    print("\n\n4. Signal Processing:")
    # Generate synthetic signal with noise
    t = np.linspace(0, 10, 1000)
    clean_signal = np.sin(2 * np.pi * 5 * t) + 0.5 * np.sin(2 * np.pi * 20 * t)
    noisy_signal = clean_signal + 0.3 * np.random.normal(0, 1, 1000)
    
    filtered_signal = SignalProcessing.filter_lowpass(noisy_signal, 10, 100)
    psd_result = SignalProcessing.power_spectral_density(clean_signal, 100)
    
    peak_result = SignalProcessing.detect_peaks(clean_signal)
    print(f"Number of peaks detected: {peak_result['n_peaks']}")
    
    print("\n\n5. Correlation Analysis:")
    x = np.random.normal(0, 1, 200)
    y = 0.5 * x + np.random.normal(0, 0.5, 200)
    corr_result = DataAnalysis.correlation_analysis(x, y)
    
    print("\n\n6. Hypothesis Testing:")
    DataAnalysis.hypothesis_testing(normal_data, test_type='normality')
    
    print("\n\nNote: Visualization plots require matplotlib backend to display.")
    print("Use DataVisualization class methods to create plots when running interactively.")


if __name__ == "__main__":
    demo_data_analysis()