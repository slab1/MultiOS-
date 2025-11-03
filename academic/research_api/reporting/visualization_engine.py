"""
Publication-Ready Reporting and Visualization Module

This module provides comprehensive reporting and visualization capabilities for OS research,
including academic-quality plots, statistical reports, publication-ready figures,
and automated report generation for research papers and presentations.
"""

import matplotlib.pyplot as plt
import matplotlib.patches as patches
import seaborn as sns
import pandas as pd
import numpy as np
from typing import Dict, List, Any, Optional, Union, Tuple
from pathlib import Path
import logging
from datetime import datetime
import base64
from io import BytesIO

# Set up plotting style
plt.style.use('seaborn-v0_8')
sns.set_palette("husl")

logger = logging.getLogger(__name__)

class PublicationVisualizer:
    """
    Publication-ready visualization engine for OS research.
    
    Creates academic-quality plots, statistical visualizations, and
    publication-ready figures suitable for research papers and presentations.
    """
    
    def __init__(self, config: Optional[Dict] = None):
        """
        Initialize the Publication Visualizer.
        
        Args:
            config: Configuration for visualization settings
        """
        self.config = config or {}
        self.figure_settings = self._setup_figure_settings()
        self.color_schemes = self._setup_color_schemes()
        self._setup_logging()
        
    def _setup_logging(self):
        """Setup logging for visualization operations."""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
    
    def _setup_figure_settings(self) -> Dict:
        """Setup default figure settings for publication quality."""
        return {
            'figsize': (10, 6),
            'dpi': 300,
            'font_size': 12,
            'title_font_size': 14,
            'label_font_size': 11,
            'tick_font_size': 10,
            'legend_font_size': 10,
            'grid_alpha': 0.3,
            'line_width': 2,
            'marker_size': 6,
            'colors': ['#2E86C1', '#E74C3C', '#58D68D', '#F39C12', '#AF7AC5', '#5DADE2']
        }
    
    def _setup_color_schemes(self) -> Dict:
        """Setup color schemes for different plot types."""
        return {
            'performance': ['#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd', '#8c564b'],
            'comparison': ['#2E86C1', '#E74C3C', '#58D68D', '#F39C12', '#AF7AC5', '#5DADE2'],
            'temperature': ['#313695', '#4575b4', '#74add1', '#abd9e9', '#e0f3f8', '#ffffbf', '#fee090', '#fdae61', '#f46d43', '#d73027', '#a50026'],
            'grayscale': ['#000000', '#333333', '#666666', '#999999', '#CCCCCC', '#FFFFFF']
        }
    
    def create_performance_comparison_plot(self, data: pd.DataFrame, 
                                         metrics: List[str],
                                         title: str = "Performance Comparison",
                                         save_path: Optional[Union[str, Path]] = None) -> plt.Figure:
        """
        Create a performance comparison plot with statistical significance.
        
        Args:
            data: Data containing performance metrics
            metrics: List of metrics to compare
            title: Plot title
            save_path: Path to save the plot
            
        Returns:
            Matplotlib figure object
        """
        try:
            fig, axes = plt.subplots(2, 2, figsize=(15, 12))
            fig.suptitle(title, fontsize=self.figure_settings['title_font_size'], fontweight='bold')
            
            # Performance boxplot
            ax1 = axes[0, 0]
            box_data = [data[metric].dropna() for metric in metrics]
            bp1 = ax1.boxplot(box_data, labels=metrics, patch_artist=True)
            
            for patch, color in zip(bp1['boxes'], self.color_schemes['performance']):
                patch.set_facecolor(color)
                patch.set_alpha(0.7)
            
            ax1.set_title('Performance Distribution', fontweight='bold')
            ax1.set_ylabel('Value')
            ax1.tick_params(axis='x', rotation=45)
            ax1.grid(True, alpha=self.figure_settings['grid_alpha'])
            
            # Performance violin plot
            ax2 = axes[0, 1]
            violin_data = []
            violin_labels = []
            for metric in metrics:
                if not data[metric].dropna().empty:
                    violin_data.append(data[metric].dropna())
                    violin_labels.append(metric)
            
            parts = ax2.violinplot(violin_data, positions=range(len(violin_data)), showmeans=True)
            ax2.set_xticks(range(len(violin_labels)))
            ax2.set_xticklabels(violin_labels, rotation=45)
            ax2.set_title('Performance Distribution (Violin)', fontweight='bold')
            ax2.set_ylabel('Value')
            ax2.grid(True, alpha=self.figure_settings['grid_alpha'])
            
            # Mean performance comparison
            ax3 = axes[1, 0]
            means = [data[metric].mean() for metric in metrics]
            stds = [data[metric].std() for metric in metrics]
            
            bars = ax3.bar(metrics, means, yerr=stds, capsize=5, 
                          color=self.color_schemes['performance'][:len(metrics)],
                          alpha=0.8, edgecolor='black', linewidth=1)
            
            ax3.set_title('Mean Performance with Error Bars', fontweight='bold')
            ax3.set_ylabel('Mean Value')
            ax3.tick_params(axis='x', rotation=45)
            ax3.grid(True, alpha=self.figure_settings['grid_alpha'], axis='y')
            
            # Add value labels on bars
            for bar, mean, std in zip(bars, means, stds):
                height = bar.get_height()
                ax3.text(bar.get_x() + bar.get_width()/2., height + std,
                        f'{mean:.2f}±{std:.2f}', ha='center', va='bottom', fontsize=9)
            
            # Statistical summary table
            ax4 = axes[1, 1]
            ax4.axis('tight')
            ax4.axis('off')
            
            # Create summary statistics
            summary_data = []
            for metric in metrics:
                series = data[metric].dropna()
                if not series.empty:
                    summary_data.append([
                        metric,
                        f'{series.mean():.3f}',
                        f'{series.std():.3f}',
                        f'{series.min():.3f}',
                        f'{series.max():.3f}',
                        f'{series.median():.3f}'
                    ])
            
            table = ax4.table(cellText=summary_data,
                             colLabels=['Metric', 'Mean', 'Std', 'Min', 'Max', 'Median'],
                             cellLoc='center',
                             loc='center')
            table.auto_set_font_size(False)
            table.set_fontsize(9)
            table.scale(1.2, 1.5)
            
            ax4.set_title('Statistical Summary', fontweight='bold', pad=20)
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
                logger.info(f"Performance comparison plot saved to {save_path}")
            
            return fig
            
        except Exception as e:
            logger.error(f"Failed to create performance comparison plot: {e}")
            raise
    
    def create_time_series_visualization(self, data: pd.DataFrame, 
                                       time_column: str,
                                       value_columns: List[str],
                                       title: str = "Time Series Analysis",
                                       save_path: Optional[Union[str, Path]] = None) -> plt.Figure:
        """
        Create comprehensive time series visualization.
        
        Args:
            data: Time series data
            time_column: Name of time column
            value_columns: List of value columns to plot
            title: Plot title
            save_path: Path to save the plot
            
        Returns:
            Matplotlib figure object
        """
        try:
            # Ensure time column is datetime
            if not pd.api.types.is_datetime64_any_dtype(data[time_column]):
                data[time_column] = pd.to_datetime(data[time_column])
            
            fig, axes = plt.subplots(3, 1, figsize=(15, 12))
            fig.suptitle(title, fontsize=self.figure_settings['title_font_size'], fontweight='bold')
            
            # Main time series plot
            ax1 = axes[0]
            for i, col in enumerate(value_columns):
                if col in data.columns:
                    ax1.plot(data[time_column], data[col], 
                            label=col, linewidth=self.figure_settings['line_width'],
                            color=self.color_schemes['comparison'][i % len(self.color_schemes['comparison'])])
            
            ax1.set_title('Time Series Trends', fontweight='bold')
            ax1.set_xlabel('Time')
            ax1.set_ylabel('Value')
            ax1.legend()
            ax1.grid(True, alpha=self.figure_settings['grid_alpha'])
            
            # Moving averages
            ax2 = axes[1]
            window_size = max(1, len(data) // 20)  # Dynamic window size
            
            for i, col in enumerate(value_columns):
                if col in data.columns:
                    moving_avg = data[col].rolling(window=window_size, center=True).mean()
                    ax2.plot(data[time_column], moving_avg,
                            label=f'{col} (MA-{window_size})', linewidth=self.figure_settings['line_width'],
                            linestyle='--',
                            color=self.color_schemes['comparison'][i % len(self.color_schemes['comparison'])])
            
            ax2.set_title('Moving Averages', fontweight='bold')
            ax2.set_xlabel('Time')
            ax2.set_ylabel('Value')
            ax2.legend()
            ax2.grid(True, alpha=self.figure_settings['grid_alpha'])
            
            # Distribution comparison
            ax3 = axes[2]
            for i, col in enumerate(value_columns):
                if col in data.columns and pd.api.types.is_numeric_dtype(data[col]):
                    ax3.hist(data[col].dropna(), bins=30, alpha=0.7,
                            label=f'{col}', color=self.color_schemes['comparison'][i % len(self.color_schemes['comparison'])])
            
            ax3.set_title('Value Distributions', fontweight='bold')
            ax3.set_xlabel('Value')
            ax3.set_ylabel('Frequency')
            ax3.legend()
            ax3.grid(True, alpha=self.figure_settings['grid_alpha'])
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
                logger.info(f"Time series visualization saved to {save_path}")
            
            return fig
            
        except Exception as e:
            logger.error(f"Failed to create time series visualization: {e}")
            raise
    
    def create_correlation_heatmap(self, data: pd.DataFrame,
                                 method: str = 'pearson',
                                 title: str = "Correlation Analysis",
                                 save_path: Optional[Union[str, Path]] = None) -> plt.Figure:
        """
        Create publication-ready correlation heatmap.
        
        Args:
            data: Data for correlation analysis
            method: Correlation method
            title: Plot title
            save_path: Path to save the plot
            
        Returns:
            Matplotlib figure object
        """
        try:
            # Select numeric columns only
            numeric_data = data.select_dtypes(include=[np.number])
            
            if numeric_data.empty:
                raise ValueError("No numeric columns found for correlation analysis")
            
            # Compute correlation matrix
            corr_matrix = numeric_data.corr(method=method)
            
            fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
            
            # Correlation heatmap
            mask = np.triu(np.ones_like(corr_matrix, dtype=bool))
            sns.heatmap(corr_matrix, mask=mask, annot=True, cmap='RdBu_r', center=0,
                       square=True, linewidths=0.5, cbar_kws={"shrink": 0.8}, ax=ax1)
            ax1.set_title(f'Correlation Matrix ({method.title()})', fontweight='bold')
            
            # Clustered correlation heatmap
            from scipy.cluster.hierarchy import dendrogram, linkage
            from scipy.spatial.distance import squareform
            
            # Hierarchical clustering of variables
            distance_matrix = 1 - np.abs(corr_matrix)
            condensed_distances = squareform(distance_matrix)
            linkage_matrix = linkage(condensed_distances, method='average')
            
            # Reorder correlation matrix based on clustering
            dendro = dendrogram(linkage_matrix, no_plot=True)
            cluster_order = dendro['leaves']
            reordered_corr = corr_matrix.iloc[cluster_order, cluster_order]
            
            sns.heatmap(reordered_corr, annot=True, cmap='RdBu_r', center=0,
                       square=True, linewidths=0.5, cbar_kws={"shrink": 0.8}, ax=ax2)
            ax2.set_title('Clustered Correlation Matrix', fontweight='bold')
            
            plt.tight_layout()
            
            if save_path:
                plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
                logger.info(f"Correlation heatmap saved to {save_path}")
            
            return fig
            
        except Exception as e:
            logger.error(f"Failed to create correlation heatmap: {e}")
            raise
    
    def create_performance_dashboard(self, data: pd.DataFrame,
                                   title: str = "OS Performance Dashboard",
                                   save_path: Optional[Union[str, Path]] = None) -> plt.Figure:
        """
        Create comprehensive performance dashboard.
        
        Args:
            data: Performance data
            title: Dashboard title
            save_path: Path to save the dashboard
            
        Returns:
            Matplotlib figure object
        """
        try:
            fig = plt.figure(figsize=(20, 12))
            
            # Create a complex grid layout
            gs = fig.add_gridspec(3, 4, hspace=0.3, wspace=0.3)
            
            # Title
            fig.suptitle(title, fontsize=self.figure_settings['title_font_size'] * 1.5, fontweight='bold')
            
            # Identify numeric columns
            numeric_cols = data.select_dtypes(include=[np.number]).columns.tolist()
            
            if len(numeric_cols) >= 4:
                # 1. Performance Overview (Top Left)
                ax1 = fig.add_subplot(gs[0, :2])
                performance_summary = data[numeric_cols].mean()
                colors = plt.cm.viridis(np.linspace(0, 1, len(performance_summary)))
                
                bars = ax1.bar(range(len(performance_summary)), performance_summary, color=colors)
                ax1.set_title('Performance Overview', fontweight='bold', fontsize=14)
                ax1.set_xticks(range(len(performance_summary)))
                ax1.set_xticklabels(performance_summary.index, rotation=45, ha='right')
                ax1.set_ylabel('Average Value')
                ax1.grid(True, alpha=self.figure_settings['grid_alpha'], axis='y')
                
                # Add value labels
                for bar, value in zip(bars, performance_summary):
                    ax1.text(bar.get_x() + bar.get_width()/2., bar.get_height(),
                            f'{value:.2f}', ha='center', va='bottom', fontsize=10)
                
                # 2. Performance Distribution (Top Right)
                ax2 = fig.add_subplot(gs[0, 2:])
                # Select first few columns for distribution
                dist_cols = numeric_cols[:min(4, len(numeric_cols))]
                data_for_dist = data[dist_cols].melt(var_name='Metric', value_name='Value')
                
                sns.violinplot(data=data_for_dist, x='Metric', y='Value', ax=ax2)
                ax2.set_title('Performance Distributions', fontweight='bold', fontsize=14)
                ax2.tick_params(axis='x', rotation=45)
                
                # 3. Correlation Matrix (Middle Left)
                ax3 = fig.add_subplot(gs[1, :2])
                corr_matrix = data[numeric_cols].corr()
                sns.heatmap(corr_matrix, annot=True, cmap='RdBu_r', center=0,
                           square=True, linewidths=0.5, cbar_kws={"shrink": 0.8}, ax=ax3)
                ax3.set_title('Correlation Matrix', fontweight='bold', fontsize=14)
                
                # 4. Performance Trends (Middle Right)
                ax4 = fig.add_subplot(gs[1, 2:])
                if 'timestamp' in data.columns or 'time' in data.columns:
                    time_col = 'timestamp' if 'timestamp' in data.columns else 'time'
                    for col in numeric_cols[:3]:  # Plot first 3 metrics
                        if pd.api.types.is_datetime64_any_dtype(data[time_col]):
                            ax4.plot(data[time_col], data[col], label=col, linewidth=2)
                    ax4.set_title('Performance Trends', fontweight='bold', fontsize=14)
                    ax4.set_xlabel('Time')
                    ax4.set_ylabel('Value')
                    ax4.legend()
                    ax4.tick_params(axis='x', rotation=45)
                else:
                    # If no time column, show box plots
                    box_data = [data[col].dropna() for col in numeric_cols[:4]]
                    bp = ax4.boxplot(box_data, labels=numeric_cols[:4], patch_artist=True)
                    colors = plt.cm.Set3(np.linspace(0, 1, len(bp['boxes'])))
                    for patch, color in zip(bp['boxes'], colors):
                        patch.set_facecolor(color)
                    ax4.set_title('Performance Variability', fontweight='bold', fontsize=14)
                    ax4.tick_params(axis='x', rotation=45)
                
                ax4.grid(True, alpha=self.figure_settings['grid_alpha'])
                
                # 5. Summary Statistics Table (Bottom)
                ax5 = fig.add_subplot(gs[2, :])
                ax5.axis('tight')
                ax5.axis('off')
                
                # Create comprehensive summary
                summary_data = []
                for col in numeric_cols[:6]:  # Limit to 6 columns for readability
                    series = data[col].dropna()
                    if not series.empty:
                        summary_data.append([
                            col,
                            f'{series.mean():.3f}',
                            f'{series.std():.3f}',
                            f'{series.min():.3f}',
                            f'{series.max():.3f}',
                            f'{series.median():.3f}',
                            f'{(series.std()/series.mean()*100):.1f}%' if series.mean() != 0 else 'N/A'
                        ])
                
                if summary_data:
                    table = ax5.table(cellText=summary_data,
                                     colLabels=['Metric', 'Mean', 'Std', 'Min', 'Max', 'Median', 'CV%'],
                                     cellLoc='center',
                                     loc='center')
                    table.auto_set_font_size(False)
                    table.set_fontsize(10)
                    table.scale(1.2, 2)
                    
                    # Style the table
                    for i in range(len(summary_data) + 1):
                        for j in range(7):
                            cell = table[(i, j)]
                            if i == 0:  # Header row
                                cell.set_facecolor('#4CAF50')
                                cell.set_text_props(weight='bold', color='white')
                            else:
                                cell.set_facecolor('#f0f0f0' if i % 2 == 0 else '#ffffff')
                
                ax5.set_title('Performance Summary Statistics', fontweight='bold', fontsize=14, pad=20)
            
            else:
                # Fallback for insufficient numeric columns
                ax = fig.add_subplot(111)
                ax.text(0.5, 0.5, 'Insufficient numeric data for dashboard creation',
                       ha='center', va='center', transform=ax.transAxes, fontsize=16)
                ax.set_title('Dashboard Creation Failed', fontweight='bold')
            
            if save_path:
                plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
                logger.info(f"Performance dashboard saved to {save_path}")
            
            return fig
            
        except Exception as e:
            logger.error(f"Failed to create performance dashboard: {e}")
            raise
    
    def create_research_publication_figure(self, data: pd.DataFrame,
                                         analysis_results: Dict[str, Any],
                                         figure_type: str = 'comprehensive',
                                         title: str = "Research Results",
                                         save_path: Optional[Union[str, Path]] = None) -> plt.Figure:
        """
        Create publication-ready research figures.
        
        Args:
            data: Research data
            analysis_results: Results from data analysis
            figure_type: Type of figure ('comprehensive', 'performance', 'statistical')
            title: Figure title
            save_path: Path to save the figure
            
        Returns:
            Matplotlib figure object
        """
        try:
            if figure_type == 'comprehensive':
                return self._create_comprehensive_research_figure(data, analysis_results, title, save_path)
            elif figure_type == 'performance':
                return self._create_performance_research_figure(data, analysis_results, title, save_path)
            elif figure_type == 'statistical':
                return self._create_statistical_research_figure(data, analysis_results, title, save_path)
            else:
                raise ValueError(f"Unsupported figure type: {figure_type}")
                
        except Exception as e:
            logger.error(f"Failed to create research publication figure: {e}")
            raise
    
    def _create_comprehensive_research_figure(self, data: pd.DataFrame,
                                            analysis_results: Dict[str, Any],
                                            title: str,
                                            save_path: Optional[Union[str, Path]]) -> plt.Figure:
        """Create comprehensive research figure for publication."""
        fig = plt.figure(figsize=(16, 12))
        gs = fig.add_gridspec(3, 3, hspace=0.4, wspace=0.3)
        
        fig.suptitle(title, fontsize=self.figure_settings['title_font_size'] * 1.2, fontweight='bold')
        
        numeric_cols = data.select_dtypes(include=[np.number]).columns.tolist()
        
        # 1. Performance overview
        ax1 = fig.add_subplot(gs[0, 0])
        if numeric_cols:
            means = data[numeric_cols].mean()
            stds = data[numeric_cols].std()
            ax1.bar(range(len(means)), means, yerr=stds, capsize=3,
                   color=self.color_schemes['performance'][:len(means)])
            ax1.set_title('Performance Metrics', fontweight='bold')
            ax1.set_xticks(range(len(means)))
            ax1.set_xticklabels(means.index, rotation=45, ha='right')
            ax1.set_ylabel('Mean Value')
            ax1.grid(True, alpha=0.3, axis='y')
        
        # 2. Distribution analysis
        ax2 = fig.add_subplot(gs[0, 1])
        if numeric_cols and len(numeric_cols) >= 2:
            ax2.scatter(data[numeric_cols[0]], data[numeric_cols[1]], alpha=0.6)
            ax2.set_xlabel(numeric_cols[0])
            ax2.set_ylabel(numeric_cols[1])
            ax2.set_title('Variable Relationship', fontweight='bold')
            ax2.grid(True, alpha=0.3)
        
        # 3. Time series if available
        ax3 = fig.add_subplot(gs[0, 2])
        time_cols = [col for col in data.columns if 'time' in col.lower() or 'date' in col.lower()]
        if time_cols and numeric_cols:
            time_col = time_cols[0]
            metric_col = numeric_cols[0]
            if pd.api.types.is_datetime64_any_dtype(data[time_col]):
                ax3.plot(data[time_col], data[metric_col], linewidth=2)
                ax3.set_title('Temporal Trend', fontweight='bold')
                ax3.set_xlabel('Time')
                ax3.set_ylabel(metric_col)
                ax3.tick_params(axis='x', rotation=45)
                ax3.grid(True, alpha=0.3)
        
        # 4. Statistical summary
        ax4 = fig.add_subplot(gs[1, :])
        ax4.axis('tight')
        ax4.axis('off')
        
        if 'basic_statistics' in analysis_results:
            stats = analysis_results['basic_statistics']
            if isinstance(stats, dict):
                # Convert stats to table format
                table_data = []
                for key, value in stats.items():
                    if isinstance(value, dict):
                        continue  # Skip nested dictionaries for now
                    table_data.append([key, str(value)[:50]])  # Truncate long values
                
                if table_data:
                    table = ax4.table(cellText=table_data,
                                     colLabels=['Metric', 'Value'],
                                     cellLoc='left',
                                     loc='center')
                    table.auto_set_font_size(False)
                    table.set_fontsize(9)
                    table.scale(1, 1.5)
                    ax4.set_title('Statistical Summary', fontweight='bold', pad=20)
        
        # 5. Additional analysis plots
        if 'correlation_analysis' in analysis_results:
            ax5 = fig.add_subplot(gs[2, 0])
            corr_data = analysis_results['correlation_analysis']
            if 'correlation_matrix' in corr_data:
                sns.heatmap(corr_data['correlation_matrix'], annot=True, cmap='RdBu_r',
                           center=0, square=True, linewidths=0.5, ax=ax5)
                ax5.set_title('Correlations', fontweight='bold')
        
        # 6. Performance profile
        if 'performance_profile' in analysis_results:
            ax6 = fig.add_subplot(gs[2, 1])
            perf_data = analysis_results['performance_profile']
            if 'individual_metrics' in perf_data:
                metrics = list(perf_data['individual_metrics'].keys())[:5]
                values = [perf_data['individual_metrics'][m].get('mean', 0) for m in metrics]
                ax6.barh(range(len(values)), values)
                ax6.set_yticks(range(len(values)))
                ax6.set_yticklabels(metrics)
                ax6.set_title('Performance Profile', fontweight='bold')
                ax6.set_xlabel('Mean Value')
                ax6.grid(True, alpha=0.3, axis='x')
        
        # 7. Key findings
        ax7 = fig.add_subplot(gs[2, 2])
        ax7.axis('off')
        findings_text = self._extract_key_findings(analysis_results)
        ax7.text(0.1, 0.9, 'Key Findings:', fontweight='bold', fontsize=12, transform=ax7.transAxes)
        ax7.text(0.1, 0.7, findings_text, fontsize=10, transform=ax7.transAxes, verticalalignment='top')
        
        if save_path:
            plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
            logger.info(f"Comprehensive research figure saved to {save_path}")
        
        return fig
    
    def _create_performance_research_figure(self, data: pd.DataFrame,
                                          analysis_results: Dict[str, Any],
                                          title: str,
                                          save_path: Optional[Union[str, Path]]) -> plt.Figure:
        """Create performance-focused research figure."""
        fig, axes = plt.subplots(2, 2, figsize=(14, 10))
        fig.suptitle(title, fontsize=self.figure_settings['title_font_size'] * 1.2, fontweight='bold')
        
        numeric_cols = data.select_dtypes(include=[np.number]).columns.tolist()
        
        # Performance comparison
        ax1 = axes[0, 0]
        if numeric_cols:
            performance_data = [data[col].dropna() for col in numeric_cols[:6]]
            bp = ax1.boxplot(performance_data, labels=numeric_cols[:6], patch_artist=True)
            colors = self.color_schemes['performance'][:len(bp['boxes'])]
            for patch, color in zip(bp['boxes'], colors):
                patch.set_facecolor(color)
                patch.set_alpha(0.7)
            ax1.set_title('Performance Distribution', fontweight='bold')
            ax1.set_ylabel('Value')
            ax1.tick_params(axis='x', rotation=45)
            ax1.grid(True, alpha=0.3)
        
        # Performance vs time
        ax2 = axes[0, 1]
        time_cols = [col for col in data.columns if 'time' in col.lower()]
        if time_cols and numeric_cols:
            time_col = time_cols[0]
            for i, col in enumerate(numeric_cols[:3]):
                if pd.api.types.is_datetime64_any_dtype(data[time_col]):
                    ax2.plot(data[time_col], data[col], label=col,
                            linewidth=2, color=self.color_schemes['comparison'][i])
            ax2.set_title('Performance Trends', fontweight='bold')
            ax2.set_xlabel('Time')
            ax2.set_ylabel('Performance')
            ax2.legend()
            ax2.tick_params(axis='x', rotation=45)
            ax2.grid(True, alpha=0.3)
        
        # Resource utilization
        ax3 = axes[1, 0]
        if 'resource_utilization' in analysis_results:
            resource_data = analysis_results['resource_utilization']
            if 'individual_resources' in resource_data:
                resources = list(resource_data['individual_resources'].keys())[:5]
                utilizations = [resource_data['individual_resources'][r].get('average_utilization', 0) 
                              for r in resources]
                ax3.pie(utilizations, labels=resources, autopct='%1.1f%%')
                ax3.set_title('Resource Utilization', fontweight='bold')
        else:
            # Fallback: create simple utilization chart
            if numeric_cols:
                means = data[numeric_cols[:5]].mean()
                ax3.bar(range(len(means)), means)
                ax3.set_title('Resource Utilization', fontweight='bold')
                ax3.set_xticks(range(len(means)))
                ax3.set_xticklabels(means.index, rotation=45, ha='right')
                ax3.set_ylabel('Utilization')
                ax3.grid(True, alpha=0.3, axis='y')
        
        # Performance summary
        ax4 = axes[1, 1]
        if numeric_cols:
            stats_data = []
            for col in numeric_cols[:4]:
                series = data[col].dropna()
                if not series.empty:
                    stats_data.append([col, f'{series.mean():.2f}', f'{series.std():.2f}',
                                     f'{series.min():.2f}', f'{series.max():.2f}'])
            
            if stats_data:
                table = ax4.table(cellText=stats_data,
                                colLabels=['Metric', 'Mean', 'Std', 'Min', 'Max'],
                                cellLoc='center',
                                loc='center')
                table.auto_set_font_size(False)
                table.set_fontsize(9)
                table.scale(1, 1.5)
                ax4.axis('off')
                ax4.set_title('Performance Summary', fontweight='bold', y=0.8)
        
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
            logger.info(f"Performance research figure saved to {save_path}")
        
        return fig
    
    def _create_statistical_research_figure(self, data: pd.DataFrame,
                                          analysis_results: Dict[str, Any],
                                          title: str,
                                          save_path: Optional[Union[str, Path]]) -> plt.Figure:
        """Create statistical analysis research figure."""
        fig, axes = plt.subplots(2, 3, figsize=(18, 12))
        fig.suptitle(title, fontsize=self.figure_settings['title_font_size'] * 1.2, fontweight='bold')
        
        numeric_cols = data.select_dtypes(include=[np.number]).columns.tolist()
        
        # 1. Distribution analysis
        ax1 = axes[0, 0]
        if numeric_cols:
            for i, col in enumerate(numeric_cols[:4]):
                ax1.hist(data[col].dropna(), bins=20, alpha=0.7, label=col,
                        color=self.color_schemes['comparison'][i % len(self.color_schemes['comparison'])])
            ax1.set_title('Distribution Analysis', fontweight='bold')
            ax1.set_xlabel('Value')
            ax1.set_ylabel('Frequency')
            ax1.legend()
            ax1.grid(True, alpha=0.3)
        
        # 2. Q-Q plots for normality assessment
        ax2 = axes[0, 1]
        if numeric_cols:
            from scipy import stats
            stats.probplot(data[numeric_cols[0]].dropna(), dist="norm", plot=ax2)
            ax2.set_title('Q-Q Plot (Normality Test)', fontweight='bold')
            ax2.grid(True, alpha=0.3)
        
        # 3. Correlation analysis
        ax3 = axes[0, 2]
        if len(numeric_cols) >= 2:
            corr_matrix = data[numeric_cols[:6]].corr()
            sns.heatmap(corr_matrix, annot=True, cmap='RdBu_r', center=0,
                       square=True, linewidths=0.5, ax=ax3)
            ax3.set_title('Correlation Analysis', fontweight='bold')
        
        # 4. Statistical summary table
        ax4 = axes[1, 0]
        ax4.axis('tight')
        ax4.axis('off')
        
        if 'basic_statistics' in analysis_results:
            stats_data = []
            for col in numeric_cols[:5]:
                series = data[col].dropna()
                if not series.empty:
                    stats_data.append([
                        col,
                        f'{series.mean():.3f}',
                        f'{series.std():.3f}',
                        f'{series.skew():.3f}',
                        f'{series.kurtosis():.3f}'
                    ])
            
            if stats_data:
                table = ax4.table(cellText=stats_data,
                                colLabels=['Metric', 'Mean', 'Std', 'Skewness', 'Kurtosis'],
                                cellLoc='center',
                                loc='center')
                table.auto_set_font_size(False)
                table.set_fontsize(9)
                table.scale(1, 1.5)
                ax4.set_title('Statistical Moments', fontweight='bold', y=0.8)
        
        # 5. Hypothesis testing results
        ax5 = axes[1, 1]
        ax5.axis('off')
        test_results = self._perform_hypothesis_tests(data, numeric_cols[:4])
        ax5.text(0.1, 0.9, 'Hypothesis Tests:', fontweight='bold', fontsize=12, transform=ax5.transAxes)
        y_pos = 0.7
        for test_name, result in test_results.items():
            ax5.text(0.1, y_pos, f'{test_name}: {result}', fontsize=10, transform=ax5.transAxes)
            y_pos -= 0.15
        
        # 6. Confidence intervals
        ax6 = axes[1, 2]
        if numeric_cols:
            means = []
            conf_intervals = []
            
            for col in numeric_cols[:5]:
                series = data[col].dropna()
                if len(series) > 1:
                    mean = series.mean()
                    sem = stats.sem(series)
                    ci = stats.t.interval(0.95, len(series)-1, loc=mean, scale=sem)
                    means.append(mean)
                    conf_intervals.append(ci)
            
            if means:
                ax6.errorbar(range(len(means)), means, 
                           yerr=[[mean - ci[0] for mean, ci in zip(means, conf_intervals)],
                                [ci[1] - mean for mean, ci in zip(means, conf_intervals)]],
                           fmt='o', capsize=5)
                ax6.set_title('95% Confidence Intervals', fontweight='bold')
                ax6.set_xlabel('Metrics')
                ax6.set_ylabel('Mean Value')
                ax6.set_xticks(range(len(means)))
                ax6.set_xticklabels([f'M{i+1}' for i in range(len(means))])
                ax6.grid(True, alpha=0.3)
        
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=self.figure_settings['dpi'], bbox_inches='tight')
            logger.info(f"Statistical research figure saved to {save_path}")
        
        return fig
    
    def _extract_key_findings(self, analysis_results: Dict[str, Any]) -> str:
        """Extract key findings from analysis results for figure annotation."""
        findings = []
        
        if 'basic_statistics' in analysis_results:
            findings.append("Statistical analysis completed")
        
        if 'correlation_analysis' in analysis_results:
            corr_data = analysis_results['correlation_analysis']
            if 'significant_correlations' in corr_data and corr_data['significant_correlations']:
                findings.append(f"Found {len(corr_data['significant_correlations'])} significant correlations")
        
        if 'performance_profile' in analysis_results:
            perf_data = analysis_results['performance_profile']
            if 'overall_profile' in perf_data:
                stability = perf_data['overall_profile'].get('overall_stability', 'unknown')
                findings.append(f"Overall stability: {stability}")
        
        if not findings:
            return "Comprehensive analysis performed on research data"
        
        return "\\n".join(findings[:3])  # Limit to 3 findings
    
    def _perform_hypothesis_tests(self, data: pd.DataFrame, columns: List[str]) -> Dict[str, str]:
        """Perform basic hypothesis tests on data."""
        from scipy import stats
        
        results = {}
        
        if len(columns) >= 2:
            # T-test between first two columns
            col1_data = data[columns[0]].dropna()
            col2_data = data[columns[1]].dropna()
            
            if len(col1_data) > 1 and len(col2_data) > 1:
                t_stat, p_value = stats.ttest_ind(col1_data, col2_data)
                results['T-test'] = f"p-value: {p_value:.4f} ({'significant' if p_value < 0.05 else 'not significant'})"
            
            # Normality test for first column
            if len(col1_data) > 3:
                shapiro_stat, shapiro_p = stats.shapiro(col1_data)
                results['Normality Test'] = f"p-value: {shapiro_p:.4f} ({'normal' if shapiro_p > 0.05 else 'not normal'})"
        
        return results
    
    def save_plot_as_base64(self, fig: plt.Figure, format: str = 'png') -> str:
        """
        Convert matplotlib figure to base64 string for embedding in reports.
        
        Args:
            fig: Matplotlib figure
            format: Image format ('png', 'svg', 'pdf')
            
        Returns:
            Base64 encoded string
        """
        try:
            buffer = BytesIO()
            fig.savefig(buffer, format=format, dpi=150, bbox_inches='tight')
            buffer.seek(0)
            
            image_base64 = base64.b64encode(buffer.getvalue()).decode()
            buffer.close()
            
            return f"data:image/{format};base64,{image_base64}"
            
        except Exception as e:
            logger.error(f"Failed to convert figure to base64: {e}")
            raise
    
    def create_animated_performance_plot(self, data: pd.DataFrame,
                                       time_column: str,
                                       value_columns: List[str],
                                       title: str = "Performance Animation",
                                       save_path: Optional[Union[str, Path]] = None) -> None:
        """
        Create animated performance plot (requires matplotlib animation).
        
        Args:
            data: Time series data
            time_column: Name of time column
            value_columns: List of value columns to animate
            title: Animation title
            save_path: Path to save the animation
        """
        try:
            import matplotlib.animation as animation
            from matplotlib.animation import FuncAnimation
            
            # Ensure time column is datetime
            if not pd.api.types.is_datetime64_any_dtype(data[time_column]):
                data[time_column] = pd.to_datetime(data[time_column])
            
            fig, ax = plt.subplots(figsize=(12, 8))
            
            def animate(frame):
                ax.clear()
                
                # Plot up to current frame
                current_data = data.iloc[:frame+1]
                
                for i, col in enumerate(value_columns):
                    if col in current_data.columns:
                        ax.plot(current_data[time_column], current_data[col],
                               label=col, linewidth=2,
                               color=self.color_schemes['comparison'][i % len(self.color_schemes['comparison'])])
                
                ax.set_title(f'{title} (Frame {frame+1}/{len(data)})', fontweight='bold')
                ax.set_xlabel('Time')
                ax.set_ylabel('Value')
                ax.legend()
                ax.grid(True, alpha=0.3)
                
                # Rotate x-axis labels for better readability
                plt.setp(ax.xaxis.get_majorticklabels(), rotation=45)
            
            anim = FuncAnimation(fig, animate, frames=len(data), interval=100, repeat=True)
            
            if save_path:
                anim.save(save_path, writer='pillow', fps=10)
                logger.info(f"Animated plot saved to {save_path}")
            
            plt.close(fig)
            
        except Exception as e:
            logger.error(f"Failed to create animated plot: {e}")
            raise

class ResearchReportGenerator:
    """
    Automated research report generation for OS experiments.
    
    Generates comprehensive, publication-ready reports including
    statistical analysis, visualizations, and research conclusions.
    """
    
    def __init__(self, config: Optional[Dict] = None):
        """
        Initialize the Research Report Generator.
        
        Args:
            config: Configuration for report generation
        """
        self.config = config or {}
        self.visualizer = PublicationVisualizer(config)
        self.report_sections = []
        self._setup_logging()
    
    def _setup_logging(self):
        """Setup logging for report generation."""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
    
    def generate_comprehensive_report(self, data: pd.DataFrame,
                                    analysis_results: Dict[str, Any],
                                    output_path: Union[str, Path],
                                    title: str = "OS Research Experiment Report",
                                    include_visualizations: bool = True,
                                    format: str = 'html') -> str:
        """
        Generate a comprehensive research report.
        
        Args:
            data: Research data
            analysis_results: Results from data analysis
            output_path: Path to save the report
            title: Report title
            include_visualizations: Whether to include visualizations
            format: Output format ('html', 'pdf', 'latex')
            
        Returns:
            Path to generated report
        """
        try:
            output_path = Path(output_path)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            if format == 'html':
                return self._generate_html_report(data, analysis_results, output_path, title, include_visualizations)
            elif format == 'pdf':
                return self._generate_pdf_report(data, analysis_results, output_path, title, include_visualizations)
            elif format == 'latex':
                return self._generate_latex_report(data, analysis_results, output_path, title, include_visualizations)
            else:
                raise ValueError(f"Unsupported format: {format}")
                
        except Exception as e:
            logger.error(f"Failed to generate comprehensive report: {e}")
            raise
    
    def _generate_html_report(self, data: pd.DataFrame,
                            analysis_results: Dict[str, Any],
                            output_path: Path,
                            title: str,
                            include_visualizations: bool) -> str:
        """Generate HTML report with embedded visualizations."""
        try:
            html_content = self._create_html_template(title)
            
            # Add executive summary
            html_content += self._create_executive_summary(analysis_results)
            
            # Add data overview
            html_content += self._create_data_overview(data)
            
            # Add analysis results
            html_content += self._create_analysis_section(analysis_results)
            
            # Add visualizations if requested
            if include_visualizations:
                html_content += self._create_visualizations_section(data, analysis_results)
            
            # Add conclusions and recommendations
            html_content += self._create_conclusions_section(analysis_results)
            
            # Add methodology
            html_content += self._create_methodology_section()
            
            html_content += self._close_html_template()
            
            # Write HTML file
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(html_content)
            
            logger.info(f"HTML report generated: {output_path}")
            return str(output_path)
            
        except Exception as e:
            logger.error(f"Failed to generate HTML report: {e}")
            raise
    
    def _create_html_template(self, title: str) -> str:
        """Create HTML template structure."""
        return f"""
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>{title}</title>
            <style>
                body {{
                    font-family: 'Arial', sans-serif;
                    line-height: 1.6;
                    margin: 0;
                    padding: 20px;
                    background-color: #f5f5f5;
                }}
                .container {{
                    max-width: 1200px;
                    margin: 0 auto;
                    background-color: white;
                    padding: 30px;
                    border-radius: 10px;
                    box-shadow: 0 0 20px rgba(0,0,0,0.1);
                }}
                h1 {{
                    color: #2c3e50;
                    text-align: center;
                    border-bottom: 3px solid #3498db;
                    padding-bottom: 20px;
                }}
                h2 {{
                    color: #34495e;
                    border-left: 4px solid #3498db;
                    padding-left: 20px;
                    margin-top: 40px;
                }}
                h3 {{
                    color: #2c3e50;
                    margin-top: 30px;
                }}
                .summary-box {{
                    background-color: #ecf0f1;
                    padding: 20px;
                    border-radius: 8px;
                    margin: 20px 0;
                    border-left: 5px solid #3498db;
                }}
                table {{
                    width: 100%;
                    border-collapse: collapse;
                    margin: 20px 0;
                }}
                th, td {{
                    border: 1px solid #bdc3c7;
                    padding: 12px;
                    text-align: left;
                }}
                th {{
                    background-color: #3498db;
                    color: white;
                    font-weight: bold;
                }}
                tr:nth-child(even) {{
                    background-color: #f8f9fa;
                }}
                .metric-card {{
                    background-color: #ffffff;
                    border: 1px solid #e0e0e0;
                    border-radius: 8px;
                    padding: 15px;
                    margin: 10px 0;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }}
                .chart-container {{
                    text-align: center;
                    margin: 30px 0;
                    padding: 20px;
                    background-color: #fafafa;
                    border-radius: 8px;
                }}
                .recommendation {{
                    background-color: #d5f4e6;
                    border-left: 5px solid #27ae60;
                    padding: 15px;
                    margin: 15px 0;
                    border-radius: 0 8px 8px 0;
                }}
                .warning {{
                    background-color: #fdeaea;
                    border-left: 5px solid #e74c3c;
                    padding: 15px;
                    margin: 15px 0;
                    border-radius: 0 8px 8px 0;
                }}
            </style>
        </head>
        <body>
            <div class="container">
        """
    
    def _create_executive_summary(self, analysis_results: Dict[str, Any]) -> str:
        """Create executive summary section."""
        summary = """
        <div class="summary-box">
            <h2>Executive Summary</h2>
        """
        
        # Extract key findings
        key_findings = []
        
        if 'basic_statistics' in analysis_results:
            key_findings.append("Statistical analysis completed on experimental data")
        
        if 'correlation_analysis' in analysis_results:
            corr_data = analysis_results['correlation_analysis']
            if 'significant_correlations' in corr_data:
                key_findings.append(f"Identified {len(corr_data['significant_correlations'])} significant correlations")
        
        if 'performance_profile' in analysis_results:
            perf_data = analysis_results['performance_profile']
            if 'overall_profile' in perf_data:
                stability = perf_data['overall_profile'].get('overall_stability', 'unknown')
                key_findings.append(f"System performance stability: {stability}")
        
        if 'resource_utilization' in analysis_results:
            resource_data = analysis_results['resource_utilization']
            if 'bottleneck_analysis' in resource_data:
                bottlenecks = resource_data['bottleneck_analysis']
                key_findings.append(f"Identified {len(bottlenecks)} potential resource bottlenecks")
        
        summary += "<ul>"
        for finding in key_findings:
            summary += f"<li>{finding}</li>"
        summary += "</ul>"
        summary += "</div>"
        
        return summary
    
    def _create_data_overview(self, data: pd.DataFrame) -> str:
        """Create data overview section."""
        return f"""
        <h2>Data Overview</h2>
        <div class="metric-card">
            <h3>Dataset Information</h3>
            <p><strong>Total Records:</strong> {len(data):,}</p>
            <p><strong>Total Variables:</strong> {len(data.columns)}</p>
            <p><strong>Numeric Variables:</strong> {len(data.select_dtypes(include=[np.number]).columns)}</p>
            <p><strong>Categorical Variables:</strong> {len(data.select_dtypes(include=['object']).columns)}</p>
            <p><strong>Missing Values:</strong> {data.isnull().sum().sum():,} ({(data.isnull().sum().sum() / (len(data) * len(data.columns)) * 100):.2f}%)</p>
        </div>
        
        <h3>Variable Summary</h3>
        <table>
            <tr>
                <th>Variable Name</th>
                <th>Data Type</th>
                <th>Non-Null Count</th>
                <th>Missing %</th>
                <th>Unique Values</th>
            </tr>
        """
    
    def _create_analysis_section(self, analysis_results: Dict[str, Any]) -> str:
        """Create analysis results section."""
        section = "<h2>Analysis Results</h2>"
        
        if 'basic_statistics' in analysis_results:
            section += """
            <h3>Descriptive Statistics</h3>
            <div class="metric-card">
                <p>Comprehensive descriptive statistics computed for all numeric variables including measures of central tendency, dispersion, and distribution shape.</p>
            </div>
            """
        
        if 'correlation_analysis' in analysis_results:
            section += """
            <h3>Correlation Analysis</h3>
            <div class="metric-card">
                <p>Analysis of relationships between variables using correlation coefficients to identify significant associations.</p>
            </div>
            """
        
        if 'performance_profile' in analysis_results:
            section += """
            <h3>Performance Profile</h3>
            <div class="metric-card">
                <p>OS-specific performance analysis including stability assessment and optimization recommendations.</p>
            </div>
            """
        
        if 'resource_utilization' in analysis_results:
            section += """
            <h3>Resource Utilization Analysis</h3>
            <div class="metric-card">
                <p>Analysis of system resource utilization patterns and identification of potential bottlenecks.</p>
            </div>
            """
        
        return section
    
    def _create_visualizations_section(self, data: pd.DataFrame, analysis_results: Dict[str, Any]) -> str:
        """Create visualizations section."""
        section = "<h2>Visualizations</h2>"
        
        try:
            # Create and embed performance dashboard
            dashboard_fig = self.visualizer.create_performance_dashboard(data)
            dashboard_base64 = self.visualizer.save_plot_as_base64(dashboard_fig)
            
            section += f"""
            <div class="chart-container">
                <h3>Performance Dashboard</h3>
                <img src="{dashboard_base64}" alt="Performance Dashboard" style="max-width: 100%; height: auto;">
            </div>
            """
            
            plt.close(dashboard_fig)  # Clean up
            
        except Exception as e:
            logger.warning(f"Failed to create dashboard visualization: {e}")
        
        try:
            # Create correlation heatmap
            numeric_data = data.select_dtypes(include=[np.number])
            if not numeric_data.empty and len(numeric_data.columns) > 1:
                corr_fig = self.visualizer.create_correlation_heatmap(data)
                corr_base64 = self.visualizer.save_plot_as_base64(corr_fig)
                
                section += f"""
                <div class="chart-container">
                    <h3>Correlation Analysis</h3>
                    <img src="{corr_base64}" alt="Correlation Heatmap" style="max-width: 100%; height: auto;">
                </div>
                """
                
                plt.close(corr_fig)  # Clean up
                
        except Exception as e:
            logger.warning(f"Failed to create correlation visualization: {e}")
        
        return section
    
    def _create_conclusions_section(self, analysis_results: Dict[str, Any]) -> str:
        """Create conclusions and recommendations section."""
        section = "<h2>Conclusions and Recommendations</h2>"
        
        recommendations = []
        
        if 'performance_profile' in analysis_results:
            perf_data = analysis_results['performance_profile']
            if 'recommendations' in perf_data:
                recommendations.extend(perf_data['recommendations'])
        
        if 'resource_utilization' in analysis_results:
            resource_data = analysis_results['resource_utilization']
            if 'optimization_suggestions' in resource_data:
                recommendations.extend(resource_data['optimization_suggestions'])
        
        if recommendations:
            section += "<h3>Recommendations</h3>"
            for rec in recommendations[:5]:  # Limit to top 5 recommendations
                section += f'<div class="recommendation">{rec}</div>'
        else:
            section += """
            <div class="recommendation">
                Analysis completed successfully. Review detailed results for specific insights and optimization opportunities.
            </div>
            """
        
        return section
    
    def _create_methodology_section(self) -> str:
        """Create methodology section."""
        return """
        <h2>Methodology</h2>
        <div class="metric-card">
            <h3>Analysis Approach</h3>
            <p>This research utilized comprehensive statistical analysis methods including:</p>
            <ul>
                <li><strong>Descriptive Statistics:</strong> Measures of central tendency, dispersion, and distribution shape</li>
                <li><strong>Correlation Analysis:</strong> Pearson and Spearman correlation coefficients</li>
                <li><strong>Performance Analysis:</strong> OS-specific performance profiling and stability assessment</li>
                <li><strong>Resource Analysis:</strong> Utilization patterns and bottleneck identification</li>
                <li><strong>Visualization:</strong> Publication-quality charts and dashboards for data interpretation</li>
            </ul>
            
            <h3>Tools and Technologies</h3>
            <p>Analysis performed using Python-based research infrastructure including pandas, numpy, scipy, matplotlib, and seaborn for statistical computing and visualization.</p>
        </div>
        """
    
    def _close_html_template(self) -> str:
        """Close HTML template."""
        return """
            </div>
            <footer style="text-align: center; margin-top: 50px; padding: 20px; border-top: 1px solid #bdc3c7; color: #7f8c8d;">
                <p>Generated by OS Research API | Report Date: """ + datetime.now().strftime("%Y-%m-%d %H:%M:%S") + """</p>
            </footer>
        </body>
        </html>
        """
    
    def _generate_pdf_report(self, data: pd.DataFrame,
                           analysis_results: Dict[str, Any],
                           output_path: Path,
                           title: str,
                           include_visualizations: bool) -> str:
        """Generate PDF report (requires additional dependencies)."""
        # This would require additional libraries like reportlab or weasyprint
        # For now, return HTML path and suggest conversion
        html_path = output_path.with_suffix('.html')
        self._generate_html_report(data, analysis_results, html_path, title, include_visualizations)
        
        logger.warning(f"PDF generation not fully implemented. HTML report generated at: {html_path}")
        return str(html_path)
    
    def _generate_latex_report(self, data: pd.DataFrame,
                             analysis_results: Dict[str, Any],
                             output_path: Path,
                             title: str,
                             include_visualizations: bool) -> str:
        """Generate LaTeX report for academic publications."""
        # This would require LaTeX-specific formatting
        # For now, return HTML path
        html_path = output_path.with_suffix('.html')
        self._generate_html_report(data, analysis_results, html_path, title, include_visualizations)
        
        logger.warning(f"LaTeX generation not fully implemented. HTML report generated at: {html_path}")
        return str(html_path)

def create_publication_visualizer(config: Optional[Dict] = None) -> PublicationVisualizer:
    """
    Factory function to create a PublicationVisualizer instance.
    
    Args:
        config: Configuration dictionary
        
    Returns:
        PublicationVisualizer instance
    """
    return PublicationVisualizer(config=config)

def create_report_generator(config: Optional[Dict] = None) -> ResearchReportGenerator:
    """
    Factory function to create a ResearchReportGenerator instance.
    
    Args:
        config: Configuration dictionary
        
    Returns:
        ResearchReportGenerator instance
    """
    return ResearchReportGenerator(config=config)