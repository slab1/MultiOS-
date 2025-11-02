#!/usr/bin/env python3
"""
Performance Analyzer CLI Tool
Advanced performance analysis and comparison tools
"""

import argparse
import json
import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional
import glob
import os
from pathlib import Path

class PerformanceAnalyzer:
    def __init__(self, data_file: str):
        self.data_file = data_file
        self.data = None
        self.df = None
        
    def load_data(self):
        """Load performance data from file"""
        try:
            if self.data_file.endswith('.json'):
                with open(self.data_file, 'r') as f:
                    self.data = json.load(f)
                self._process_json_data()
            elif self.data_file.endswith('.csv'):
                self.df = pd.read_csv(self.data_file)
                self.df['timestamp'] = pd.to_datetime(self.df['timestamp'])
            else:
                raise ValueError("Unsupported file format. Use JSON or CSV.")
                
            print(f"Loaded {len(self.df) if self.df is not None else 0} data points")
        except Exception as e:
            print(f"Error loading data: {e}")
            
    def _process_json_data(self):
        """Process JSON data into DataFrame"""
        if 'samples' in self.data:
            # Multiple samples
            samples = self.data['samples']
        else:
            # Single sample
            samples = [self.data]
        
        records = []
        for sample in samples:
            record = {
                'timestamp': pd.to_datetime(sample['cpu']['timestamp']),
                'cpu_usage': sample['cpu']['usage'],
                'memory_usage': sample['memory']['percentage'],
                'memory_used_gb': sample['memory']['used'] / (1024**3),
                'memory_total_gb': sample['memory']['total'] / (1024**3),
                'network_bytes_sent': sample['network']['io_counters']['bytes_sent'],
                'network_bytes_recv': sample['network']['io_counters']['bytes_recv'],
            }
            
            # Add disk data if available
            if sample['disk']['io_counters']:
                record['disk_read_bytes'] = sample['disk']['io_counters']['read_bytes']
                record['disk_write_bytes'] = sample['disk']['io_counters']['write_bytes']
            
            # Add power data if available
            if 'battery' in sample['power']['power']:
                record['battery_percent'] = sample['power']['power']['battery']['percent']
            record['power_estimated'] = sample['power']['power']['estimated_consumption']
            
            records.append(record)
        
        self.df = pd.DataFrame(records)
        
    def generate_statistics(self):
        """Generate comprehensive performance statistics"""
        if self.df is None or self.df.empty:
            print("No data available for analysis")
            return
            
        print("\n" + "="*60)
        print("COMPREHENSIVE PERFORMANCE STATISTICS")
        print("="*60)
        
        # Basic statistics
        stats = self.df.describe()
        
        print("\n📊 BASIC STATISTICS:")
        print(stats[['cpu_usage', 'memory_usage', 'power_estimated']].round(2))
        
        # Time-based analysis
        print(f"\n⏰ TIME ANALYSIS:")
        print(f"   Data Period: {self.df['timestamp'].min()} to {self.df['timestamp'].max()}")
        duration = self.df['timestamp'].max() - self.df['timestamp'].min()
        print(f"   Duration: {duration}")
        print(f"   Sampling Rate: {len(self.df) / duration.total_seconds():.2f} samples/sec")
        
        # Performance insights
        print(f"\n🔍 PERFORMANCE INSIGHTS:")
        
        # CPU insights
        cpu_peak = self.df['cpu_usage'].max()
        cpu_avg = self.df['cpu_usage'].mean()
        cpu_high_threshold = 80
        
        if cpu_peak > cpu_high_threshold:
            print(f"   🚨 CPU: Peak usage of {cpu_peak:.1f}% exceeds {cpu_high_threshold}% threshold")
        else:
            print(f"   ✅ CPU: Peak usage of {cpu_peak:.1f}% within acceptable range")
            
        print(f"   📈 CPU: Average usage {cpu_avg:.1f}%, variation {self.df['cpu_usage'].std():.1f}%")
        
        # Memory insights
        memory_avg = self.df['memory_usage'].mean()
        memory_peak = self.df['memory_usage'].max()
        memory_threshold = 85
        
        if memory_peak > memory_threshold:
            print(f"   🚨 Memory: Peak usage of {memory_peak:.1f}% exceeds {memory_threshold}% threshold")
        else:
            print(f"   ✅ Memory: Peak usage of {memory_peak:.1f}% within acceptable range")
            
        print(f"   📈 Memory: Average usage {memory_avg:.1f}%, variation {self.df['memory_usage'].std():.1f}%")
        
        # Power insights
        power_avg = self.df['power_estimated'].mean()
        power_peak = self.df['power_estimated'].max()
        print(f"   ⚡ Power: Average {power_avg:.1f}W, Peak {power_peak:.1f}W")
        
        # Correlation analysis
        print(f"\n🔗 CORRELATION ANALYSIS:")
        correlations = self.df[['cpu_usage', 'memory_usage', 'power_estimated']].corr()
        print("   CPU-Memory correlation:", correlations.loc['cpu_usage', 'memory_usage'])
        print("   CPU-Power correlation:", correlations.loc['cpu_usage', 'power_estimated'])
        print("   Memory-Power correlation:", correlations.loc['memory_usage', 'power_estimated'])
        
        # Percentile analysis
        print(f"\n📊 PERCENTILE ANALYSIS:")
        for metric in ['cpu_usage', 'memory_usage']:
            p50 = self.df[metric].quantile(0.5)
            p90 = self.df[metric].quantile(0.9)
            p95 = self.df[metric].quantile(0.95)
            p99 = self.df[metric].quantile(0.99)
            print(f"   {metric}: P50={p50:.1f}%, P90={p90:.1f}%, P95={p95:.1f}%, P99={p99:.1f}%")
    
    def detect_anomalies(self, method: str = 'iqr', threshold: float = 2.0):
        """Detect performance anomalies"""
        if self.df is None or self.df.empty:
            print("No data available for anomaly detection")
            return
            
        print(f"\n" + "="*60)
        print(f"ANOMALY DETECTION ({method.upper()})")
        print("="*60)
        
        anomalies = []
        
        if method == 'iqr':
            # Interquartile Range method
            for column in ['cpu_usage', 'memory_usage', 'power_estimated']:
                Q1 = self.df[column].quantile(0.25)
                Q3 = self.df[column].quantile(0.75)
                IQR = Q3 - Q1
                lower_bound = Q1 - 1.5 * IQR
                upper_bound = Q3 + 1.5 * IQR
                
                column_anomalies = self.df[
                    (self.df[column] < lower_bound) | (self.df[column] > upper_bound)
                ]
                
                for _, row in column_anomalies.iterrows():
                    anomalies.append({
                        'timestamp': row['timestamp'],
                        'metric': column,
                        'value': row[column],
                        'type': 'High' if row[column] > upper_bound else 'Low',
                        'severity': abs(row[column] - (upper_bound if row[column] > upper_bound else lower_bound)) / IQR
                    })
        
        elif method == 'zscore':
            # Z-score method
            for column in ['cpu_usage', 'memory_usage', 'power_estimated']:
                z_scores = np.abs((self.df[column] - self.df[column].mean()) / self.df[column].std())
                column_anomalies = self.df[z_scores > threshold]
                
                for _, row in column_anomalies.iterrows():
                    z_score = z_scores.loc[row.name]
                    anomalies.append({
                        'timestamp': row['timestamp'],
                        'metric': column,
                        'value': row[column],
                        'type': 'High' if row[column] > self.df[column].mean() else 'Low',
                        'severity': z_score
                    })
        
        if anomalies:
            print(f"\n🚨 DETECTED {len(anomalies)} ANOMALIES:")
            
            # Group by severity
            severe_anomalies = [a for a in anomalies if a['severity'] > 3]
            moderate_anomalies = [a for a in anomalies if 1.5 <= a['severity'] <= 3]
            mild_anomalies = [a for a in anomalies if a['severity'] < 1.5]
            
            print(f"   Severe: {len(severe_anomalies)}")
            print(f"   Moderate: {len(moderate_anomalies)}")
            print(f"   Mild: {len(mild_anomalies)}")
            
            # Show recent severe anomalies
            if severe_anomalies:
                print(f"\n⚠️  RECENT SEVERE ANOMALIES:")
                recent_severe = sorted(severe_anomalies, key=lambda x: x['timestamp'], reverse=True)[:5]
                for anomaly in recent_severe:
                    print(f"   {anomaly['timestamp'].strftime('%Y-%m-%d %H:%M:%S')} - "
                          f"{anomaly['metric']}: {anomaly['value']:.1f} ({anomaly['type']}, "
                          f"severity: {anomaly['severity']:.1f})")
        else:
            print("✅ No anomalies detected with the specified method")
    
    def compare_periods(self, start1: str, end1: str, start2: str, end2: str):
        """Compare performance between two time periods"""
        if self.df is None or self.df.empty:
            print("No data available for comparison")
            return
            
        print("\n" + "="*60)
        print("PERFORMANCE COMPARISON")
        print("="*60)
        
        try:
            # Parse time periods
            start1_dt = pd.to_datetime(start1)
            end1_dt = pd.to_datetime(end1)
            start2_dt = pd.to_datetime(start2)
            end2_dt = pd.to_datetime(end2)
            
            # Filter data
            period1 = self.df[(self.df['timestamp'] >= start1_dt) & (self.df['timestamp'] <= end1_dt)]
            period2 = self.df[(self.df['timestamp'] >= start2_dt) & (self.df['timestamp'] <= end2_dt)]
            
            if period1.empty or period2.empty:
                print("One or both periods have no data")
                return
            
            print(f"\nPeriod 1: {start1} to {end1} ({len(period1)} samples)")
            print(f"Period 2: {start2} to {end2} ({len(period2)} samples)")
            
            # Compare metrics
            metrics = ['cpu_usage', 'memory_usage', 'power_estimated']
            
            print(f"\n📊 COMPARISON RESULTS:")
            print(f"{'Metric':<20} {'Period 1':<15} {'Period 2':<15} {'Change':<15} {'% Change':<15}")
            print("-" * 80)
            
            for metric in metrics:
                p1_avg = period1[metric].mean()
                p2_avg = period2[metric].mean()
                change = p2_avg - p1_avg
                pct_change = (change / p1_avg) * 100
                
                print(f"{metric:<20} {p1_avg:<15.2f} {p2_avg:<15.2f} {change:<15.2f} {pct_change:<15.1f}%")
        
        except Exception as e:
            print(f"Error in period comparison: {e}")
    
    def create_detailed_charts(self, output_dir: str = "performance_analysis"):
        """Create detailed performance analysis charts"""
        if self.df is None or self.df.empty:
            print("No data available for charting")
            return
            
        os.makedirs(output_dir, exist_ok=True)
        
        # Set style
        plt.style.use('seaborn-v0_8')
        sns.set_palette("husl")
        
        # 1. Time series analysis
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('Performance Time Series Analysis', fontsize=16)
        
        # CPU time series
        axes[0, 0].plot(self.df['timestamp'], self.df['cpu_usage'], color='blue', alpha=0.7)
        axes[0, 0].set_title('CPU Usage Over Time')
        axes[0, 0].set_ylabel('CPU Usage (%)')
        axes[0, 0].grid(True, alpha=0.3)
        
        # Memory time series
        axes[0, 1].plot(self.df['timestamp'], self.df['memory_usage'], color='red', alpha=0.7)
        axes[0, 1].set_title('Memory Usage Over Time')
        axes[0, 1].set_ylabel('Memory Usage (%)')
        axes[0, 1].grid(True, alpha=0.3)
        
        # Power time series
        axes[1, 0].plot(self.df['timestamp'], self.df['power_estimated'], color='orange', alpha=0.7)
        axes[1, 0].set_title('Power Consumption Over Time')
        axes[1, 0].set_ylabel('Power (W)')
        axes[1, 0].set_xlabel('Time')
        axes[1, 0].grid(True, alpha=0.3)
        
        # Combined normalized view
        cpu_norm = (self.df['cpu_usage'] - self.df['cpu_usage'].min()) / (self.df['cpu_usage'].max() - self.df['cpu_usage'].min())
        memory_norm = (self.df['memory_usage'] - self.df['memory_usage'].min()) / (self.df['memory_usage'].max() - self.df['memory_usage'].min())
        
        axes[1, 1].plot(self.df['timestamp'], cpu_norm, label='CPU (normalized)', alpha=0.7)
        axes[1, 1].plot(self.df['timestamp'], memory_norm, label='Memory (normalized)', alpha=0.7)
        axes[1, 1].set_title('Normalized Performance Comparison')
        axes[1, 1].set_ylabel('Normalized Values (0-1)')
        axes[1, 1].set_xlabel('Time')
        axes[1, 1].legend()
        axes[1, 1].grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(f"{output_dir}/time_series_analysis.png", dpi=150, bbox_inches='tight')
        plt.close()
        
        # 2. Distribution analysis
        fig, axes = plt.subplots(1, 3, figsize=(15, 5))
        fig.suptitle('Performance Distribution Analysis', fontsize=16)
        
        # CPU distribution
        axes[0].hist(self.df['cpu_usage'], bins=20, alpha=0.7, color='blue', edgecolor='black')
        axes[0].set_title('CPU Usage Distribution')
        axes[0].set_xlabel('CPU Usage (%)')
        axes[0].set_ylabel('Frequency')
        
        # Memory distribution
        axes[1].hist(self.df['memory_usage'], bins=20, alpha=0.7, color='red', edgecolor='black')
        axes[1].set_title('Memory Usage Distribution')
        axes[1].set_xlabel('Memory Usage (%)')
        axes[1].set_ylabel('Frequency')
        
        # Power distribution
        axes[2].hist(self.df['power_estimated'], bins=20, alpha=0.7, color='orange', edgecolor='black')
        axes[2].set_title('Power Consumption Distribution')
        axes[2].set_xlabel('Power (W)')
        axes[2].set_ylabel('Frequency')
        
        plt.tight_layout()
        plt.savefig(f"{output_dir}/distribution_analysis.png", dpi=150, bbox_inches='tight')
        plt.close()
        
        # 3. Correlation heatmap
        plt.figure(figsize=(10, 8))
        correlation_matrix = self.df[['cpu_usage', 'memory_usage', 'power_estimated']].corr()
        sns.heatmap(correlation_matrix, annot=True, cmap='coolwarm', center=0,
                   square=True, fmt='.3f')
        plt.title('Performance Metrics Correlation Heatmap')
        plt.tight_layout()
        plt.savefig(f"{output_dir}/correlation_heatmap.png", dpi=150, bbox_inches='tight')
        plt.close()
        
        # 4. Box plot comparison
        plt.figure(figsize=(10, 6))
        data_for_boxplot = [self.df['cpu_usage'], self.df['memory_usage'], self.df['power_estimated']]
        labels = ['CPU Usage (%)', 'Memory Usage (%)', 'Power (W)']
        
        box_plot = plt.boxplot(data_for_boxplot, labels=labels, patch_artist=True)
        colors = ['lightblue', 'lightcoral', 'lightgreen']
        for patch, color in zip(box_plot['boxes'], colors):
            patch.set_facecolor(color)
        
        plt.title('Performance Metrics Box Plot')
        plt.ylabel('Value')
        plt.grid(True, alpha=0.3)
        plt.tight_layout()
        plt.savefig(f"{output_dir}/boxplot_analysis.png", dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Detailed analysis charts saved to {output_dir}/")
        
        # Create summary report
        self._create_summary_report(output_dir)
    
    def _create_summary_report(self, output_dir: str):
        """Create a comprehensive summary report"""
        report_file = f"{output_dir}/performance_report.txt"
        
        with open(report_file, 'w') as f:
            f.write("PERFORMANCE ANALYSIS SUMMARY REPORT\n")
            f.write("=" * 50 + "\n\n")
            
            f.write(f"Analysis Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write(f"Data Source: {self.data_file}\n")
            f.write(f"Total Samples: {len(self.df)}\n\n")
            
            # Basic statistics
            f.write("BASIC STATISTICS:\n")
            f.write("-" * 20 + "\n")
            stats = self.df.describe()
            f.write(stats[['cpu_usage', 'memory_usage', 'power_estimated']].round(2).to_string())
            f.write("\n\n")
            
            # Key insights
            f.write("KEY INSIGHTS:\n")
            f.write("-" * 15 + "\n")
            
            cpu_peak = self.df['cpu_usage'].max()
            cpu_avg = self.df['cpu_usage'].mean()
            memory_peak = self.df['memory_usage'].max()
            memory_avg = self.df['memory_usage'].mean()
            
            f.write(f"• CPU: Peak usage {cpu_peak:.1f}%, Average {cpu_avg:.1f}%\n")
            f.write(f"• Memory: Peak usage {memory_peak:.1f}%, Average {memory_avg:.1f}%\n")
            
            if cpu_peak > 80:
                f.write("• CPU performance may need optimization (high peak usage detected)\n")
            if memory_peak > 85:
                f.write("• Memory usage may be concerning (high peak usage detected)\n")
                
            f.write("\nFILES GENERATED:\n")
            f.write("-" * 17 + "\n")
            f.write("• time_series_analysis.png - Time series plots\n")
            f.write("• distribution_analysis.png - Distribution histograms\n")
            f.write("• correlation_heatmap.png - Correlation analysis\n")
            f.write("• boxplot_analysis.png - Statistical box plots\n")
            f.write("• performance_report.txt - This summary report\n")
        
        print(f"Summary report saved to {report_file}")
    
    def export_analysis(self, output_file: str):
        """Export analysis results to JSON"""
        if self.df is None or self.df.empty:
            print("No data available for export")
            return
            
        analysis_results = {
            'metadata': {
                'analysis_date': datetime.now().isoformat(),
                'data_source': self.data_file,
                'total_samples': len(self.df),
                'analysis_period': {
                    'start': self.df['timestamp'].min().isoformat(),
                    'end': self.df['timestamp'].max().isoformat()
                }
            },
            'statistics': {
                'cpu': {
                    'mean': float(self.df['cpu_usage'].mean()),
                    'std': float(self.df['cpu_usage'].std()),
                    'min': float(self.df['cpu_usage'].min()),
                    'max': float(self.df['cpu_usage'].max()),
                    'percentiles': {
                        '50': float(self.df['cpu_usage'].quantile(0.5)),
                        '90': float(self.df['cpu_usage'].quantile(0.9)),
                        '95': float(self.df['cpu_usage'].quantile(0.95)),
                        '99': float(self.df['cpu_usage'].quantile(0.99))
                    }
                },
                'memory': {
                    'mean': float(self.df['memory_usage'].mean()),
                    'std': float(self.df['memory_usage'].std()),
                    'min': float(self.df['memory_usage'].min()),
                    'max': float(self.df['memory_usage'].max())
                },
                'power': {
                    'mean': float(self.df['power_estimated'].mean()),
                    'std': float(self.df['power_estimated'].std()),
                    'min': float(self.df['power_estimated'].min()),
                    'max': float(self.df['power_estimated'].max())
                }
            },
            'correlations': {
                'cpu_memory': float(self.df['cpu_usage'].corr(self.df['memory_usage'])),
                'cpu_power': float(self.df['cpu_usage'].corr(self.df['power_estimated'])),
                'memory_power': float(self.df['memory_usage'].corr(self.df['power_estimated']))
            }
        }
        
        with open(output_file, 'w') as f:
            json.dump(analysis_results, f, indent=2)
        
        print(f"Analysis results exported to {output_file}")

def main():
    parser = argparse.ArgumentParser(description='Performance Analyzer CLI Tool')
    parser.add_argument('data_file', help='Performance data file (JSON or CSV)')
    parser.add_argument('--stats', action='store_true', help='Generate comprehensive statistics')
    parser.add_argument('--anomalies', choices=['iqr', 'zscore'], help='Detect anomalies using specified method')
    parser.add_argument('--threshold', type=float, default=2.0, help='Z-score threshold for anomaly detection')
    parser.add_argument('--compare', nargs=4, metavar=('START1', 'END1', 'START2', 'END2'), 
                       help='Compare two time periods: START1 END1 START2 END2')
    parser.add_argument('--chart', action='store_true', help='Generate detailed analysis charts')
    parser.add_argument('--export', type=str, help='Export analysis results to JSON file')
    parser.add_argument('--output-dir', type=str, default='performance_analysis', 
                       help='Output directory for charts and reports')
    
    args = parser.parse_args()
    
    analyzer = PerformanceAnalyzer(args.data_file)
    analyzer.load_data()
    
    if args.stats:
        analyzer.generate_statistics()
    
    if args.anomalies:
        analyzer.detect_anomalies(method=args.anomalies, threshold=args.threshold)
    
    if args.compare:
        if len(args.compare) != 4:
            print("Error: Compare requires 4 arguments (start1 end1 start2 end2)")
            return
        analyzer.compare_periods(*args.compare)
    
    if args.chart:
        analyzer.create_detailed_charts(args.output_dir)
    
    if args.export:
        analyzer.export_analysis(args.export)

if __name__ == '__main__':
    main()