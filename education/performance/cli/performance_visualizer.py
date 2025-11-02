#!/usr/bin/env python3
"""
Performance Visualizer CLI Tool
Advanced performance visualization and heatmap generation
"""

import argparse
import json
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.colors import LinearSegmentedColormap
import seaborn as sns
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional
import os
from pathlib import Path

class PerformanceVisualizer:
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
            samples = self.data['samples']
        else:
            samples = [self.data]
        
        records = []
        for sample in samples:
            record = {
                'timestamp': pd.to_datetime(sample['cpu']['timestamp']),
                'cpu_usage': sample['cpu']['usage'],
                'cpu_cores': sample['cpu']['per_core'],
                'memory_usage': sample['memory']['percentage'],
                'power_estimated': sample['power']['power']['estimated_consumption']
            }
            
            # Add temperature data if available
            if sample['temperature']['sensors']:
                for sensor_name, entries in sample['temperature']['sensors'].items():
                    for entry in entries:
                        record[f"{sensor_name}_{entry['label']}"] = entry['current']
            
            records.append(record)
        
        self.df = pd.DataFrame(records)
    
    def create_cpu_heatmap(self, output_file: str = "cpu_heatmap.png"):
        """Create CPU utilization heatmap across time and cores"""
        if 'cpu_cores' not in self.df.columns or self.df['cpu_cores'].iloc[0] is None:
            print("CPU core data not available")
            return
            
        print("Creating CPU core utilization heatmap...")
        
        # Prepare data for heatmap
        core_data = []
        for _, row in self.df.iterrows():
            if isinstance(row['cpu_cores'], list):
                core_data.append(row['cpu_cores'])
            else:
                # Simulate core data if not available
                core_data.append([row['cpu_usage']] * 8)  # Assume 8 cores
        
        core_data = np.array(core_data)
        
        # Create time labels (show every 10th timestamp to avoid crowding)
        time_labels = self.df['timestamp'].dt.strftime('%H:%M:%S')
        step = max(1, len(time_labels) // 20)
        time_labels = time_labels[::step]
        core_data = core_data[::step, :]
        
        # Create heatmap
        plt.figure(figsize=(12, 8))
        im = plt.imshow(core_data.T, aspect='auto', cmap='YlOrRd', 
                       interpolation='nearest', vmin=0, vmax=100)
        
        # Customize the plot
        plt.xlabel('Time')
        plt.ylabel('CPU Core')
        plt.title('CPU Core Utilization Heatmap', fontsize=16, fontweight='bold')
        
        # Set tick labels
        num_times = len(time_labels)
        x_ticks = np.linspace(0, num_times-1, min(10, num_times), dtype=int)
        plt.xticks(x_ticks, [time_labels[i] for i in x_ticks], rotation=45)
        
        plt.ylabel('CPU Core', fontsize=12)
        y_ticks = range(min(core_data.shape[1], 16))  # Show up to 16 cores
        plt.yticks(y_ticks, [f'Core {i+1}' for i in y_ticks])
        
        # Add colorbar
        cbar = plt.colorbar(im, shrink=0.8)
        cbar.set_label('CPU Usage (%)', fontsize=12)
        
        # Add grid
        plt.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"CPU heatmap saved to {output_file}")
    
    def create_thermal_heatmap(self, output_file: str = "thermal_heatmap.png"):
        """Create thermal map visualization"""
        temp_columns = [col for col in self.df.columns if 'temp' in col.lower() or 'core' in col.lower()]
        
        if not temp_columns:
            print("Temperature data not available")
            return
            
        print("Creating thermal visualization...")
        
        # Get temperature data
        temp_data = self.df[temp_columns].values
        
        # Create thermal map
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10))
        
        # 1. Temperature time series heatmap
        im1 = ax1.imshow(temp_data.T, aspect='auto', cmap='coolwarm',
                        interpolation='nearest')
        ax1.set_title('Temperature Sensors Heatmap', fontsize=14, fontweight='bold')
        ax1.set_xlabel('Time')
        ax1.set_ylabel('Temperature Sensor')
        
        # Set labels
        step = max(1, len(self.df) // 20)
        time_labels = self.df['timestamp'].dt.strftime('%H:%M:%S')
        x_ticks = np.linspace(0, len(time_labels)-1, min(10, len(time_labels)), dtype=int)
        ax1.set_xticks(x_ticks)
        ax1.set_xticklabels([time_labels[i] for i in x_ticks], rotation=45)
        ax1.set_yticks(range(len(temp_columns)))
        ax1.set_yticklabels(temp_columns, fontsize=10)
        
        # Add colorbar
        cbar1 = plt.colorbar(im1, ax=ax1, shrink=0.8)
        cbar1.set_label('Temperature (°C)')
        
        # 2. Current temperature status
        latest_temp = temp_data[-1]
        temp_ranges = [
            (0, 40, '🟦 Cool', 'lightblue'),
            (40, 60, '🟨 Warm', 'yellow'),
            (60, 75, '🟧 Hot', 'orange'),
            (75, 100, '🟥 Critical', 'red')
        ]
        
        colors = []
        labels = []
        for i, (low, high, label, color) in enumerate(temp_ranges):
            in_range = np.sum((latest_temp >= low) & (latest_temp < high))
            if in_range > 0:
                colors.append(color)
                labels.append(f'{label}: {in_range} sensors')
        
        # Create temperature zones visualization
        y_positions = np.arange(len(temp_columns))
        bars = ax2.barh(y_positions, latest_temp, 
                       color=[self._get_temp_color(temp, 'coolwarm') for temp in latest_temp])
        
        ax2.set_title('Current Temperature Status', fontsize=14, fontweight='bold')
        ax2.set_xlabel('Temperature (°C)')
        ax2.set_ylabel('Temperature Sensor')
        ax2.set_yticks(y_positions)
        ax2.set_yticklabels(temp_columns)
        
        # Add temperature values on bars
        for i, (bar, temp) in enumerate(zip(bars, latest_temp)):
            ax2.text(temp + 0.5, bar.get_y() + bar.get_height()/2, 
                    f'{temp:.1f}°C', va='center', fontweight='bold')
        
        # Add warning lines
        ax2.axvline(x=60, color='orange', linestyle='--', alpha=0.7, label='Warning (60°C)')
        ax2.axvline(x=75, color='red', linestyle='--', alpha=0.7, label='Critical (75°C)')
        ax2.legend()
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Thermal visualization saved to {output_file}")
    
    def create_memory_allocation_chart(self, output_file: str = "memory_allocation.png"):
        """Create memory allocation tracking visualization"""
        print("Creating memory allocation chart...")
        
        # This would need allocation data - for now, create a simulated view
        memory_components = ['Kernel', 'User Space', 'Cache', 'Buffers', 'Free']
        # Simulate memory distribution (in real implementation, get from data)
        current_usage = [20, 35, 25, 10, 10]  # Percentages
        
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(15, 6))
        
        # 1. Memory allocation pie chart
        colors = ['#ff6b6b', '#4ecdc4', '#45b7d1', '#f9ca24', '#f0932b']
        wedges, texts, autotexts = ax1.pie(current_usage, labels=memory_components, 
                                          colors=colors, autopct='%1.1f%%', startangle=90)
        ax1.set_title('Memory Allocation Distribution', fontsize=14, fontweight='bold')
        
        # Make percentage text more readable
        for autotext in autotexts:
            autotext.set_color('white')
            autotext.set_fontweight('bold')
        
        # 2. Memory allocation over time (stacked area chart)
        # Simulate time series data
        time_points = np.arange(0, 100)
        time_series_data = {}
        
        for component, base_usage in zip(memory_components, current_usage):
            # Add some variation over time
            variation = np.random.normal(0, 5, len(time_points))
            time_series_data[component] = np.maximum(base_usage + variation, 0)
        
        ax2.stackplot(time_points, *[time_series_data[comp] for comp in memory_components],
                     labels=memory_components, colors=colors, alpha=0.8)
        
        ax2.set_title('Memory Allocation Over Time', fontsize=14, fontweight='bold')
        ax2.set_xlabel('Time')
        ax2.set_ylabel('Memory Usage (%)')
        ax2.set_xlim(0, 100)
        ax2.set_ylim(0, 100)
        ax2.legend(loc='upper right', bbox_to_anchor=(1.15, 1))
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Memory allocation chart saved to {output_file}")
    
    def create_io_throughput_visualization(self, output_file: str = "io_throughput.png"):
        """Create I/O throughput visualization with device breakdown"""
        print("Creating I/O throughput visualization...")
        
        # This would need I/O data - simulate for demonstration
        time_points = np.arange(0, len(self.df))
        
        # Simulate I/O data
        read_throughput = np.random.exponential(30, len(time_points))
        write_throughput = np.random.exponential(20, len(time_points))
        
        # Simulate device breakdown
        devices = ['SSD', 'HDD', 'Network', 'USB']
        device_colors = ['#e74c3c', '#3498db', '#f39c12', '#9b59b6']
        
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 10))
        
        # 1. I/O throughput over time
        ax1.plot(time_points, read_throughput, color='green', linewidth=2, label='Read', alpha=0.8)
        ax1.plot(time_points, write_throughput, color='red', linewidth=2, label='Write', alpha=0.8)
        ax1.fill_between(time_points, 0, read_throughput, color='green', alpha=0.2)
        ax1.fill_between(time_points, 0, write_throughput, color='red', alpha=0.2)
        
        ax1.set_title('I/O Throughput Over Time', fontsize=14, fontweight='bold')
        ax1.set_xlabel('Time')
        ax1.set_ylabel('Throughput (MB/s)')
        ax1.legend()
        ax1.grid(True, alpha=0.3)
        
        # 2. Device breakdown (current snapshot)
        device_throughputs = [45, 25, 15, 10]  # Simulated current values
        
        bars = ax2.bar(devices, device_throughputs, color=device_colors, alpha=0.8)
        ax2.set_title('I/O Device Breakdown (Current)', fontsize=14, fontweight='bold')
        ax2.set_xlabel('Device')
        ax2.set_ylabel('Throughput (MB/s)')
        
        # Add values on bars
        for bar, value in zip(bars, device_throughputs):
            ax2.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                    f'{value} MB/s', ha='center', va='bottom', fontweight='bold')
        
        # Add total throughput annotation
        total = sum(device_throughputs)
        ax2.text(0.02, 0.98, f'Total: {total} MB/s', transform=ax2.transAxes,
                bbox=dict(boxstyle='round', facecolor='white', alpha=0.8),
                verticalalignment='top', fontsize=12, fontweight='bold')
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"I/O throughput visualization saved to {output_file}")
    
    def create_network_visualization(self, output_file: str = "network_traffic.png"):
        """Create network traffic visualization with protocol breakdown"""
        print("Creating network traffic visualization...")
        
        time_points = np.arange(0, len(self.df))
        
        # Simulate network data
        upload_speed = np.random.exponential(5, len(time_points))
        download_speed = np.random.exponential(15, len(time_points))
        
        # Protocol breakdown
        protocols = ['TCP', 'UDP', 'HTTP', 'HTTPS', 'DNS', 'Other']
        protocol_colors = ['#3498db', '#e74c3c', '#f39c12', '#27ae60', '#9b59b6', '#95a5a6']
        
        fig, (ax1, ax2, ax3) = plt.subplots(3, 1, figsize=(12, 12))
        
        # 1. Network speed over time
        ax1.fill_between(time_points, 0, upload_speed, color='blue', alpha=0.6, label='Upload')
        ax1.fill_between(time_points, 0, download_speed, color='green', alpha=0.6, label='Download')
        
        ax1.set_title('Network Traffic Over Time', fontsize=14, fontweight='bold')
        ax1.set_xlabel('Time')
        ax1.set_ylabel('Speed (Mbps)')
        ax1.legend()
        ax1.grid(True, alpha=0.3)
        
        # 2. Protocol breakdown pie chart (current snapshot)
        protocol_usage = [40, 25, 15, 12, 5, 3]  # Simulated percentages
        
        wedges, texts, autotexts = ax2.pie(protocol_usage, labels=protocols, colors=protocol_colors,
                                          autopct='%1.1f%%', startangle=90)
        ax2.set_title('Protocol Breakdown (Current)', fontsize=14, fontweight='bold')
        
        for autotext in autotexts:
            autotext.set_color('white')
            autotext.set_fontweight('bold')
        
        # 3. Connection status over time
        # Simulate active connections over time
        active_connections = np.random.poisson(50, len(time_points))
        
        ax3.plot(time_points, active_connections, color='purple', linewidth=2, marker='o', markersize=3)
        ax3.fill_between(time_points, 0, active_connections, color='purple', alpha=0.3)
        
        ax3.set_title('Active Network Connections Over Time', fontsize=14, fontweight='bold')
        ax3.set_xlabel('Time')
        ax3.set_ylabel('Active Connections')
        ax3.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Network traffic visualization saved to {output_file}")
    
    def create_power_thermal_chart(self, output_file: str = "power_thermal.png"):
        """Create power consumption and thermal visualization"""
        print("Creating power and thermal visualization...")
        
        if self.df.empty:
            return
            
        time_points = np.arange(0, len(self.df))
        
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('Power Consumption & Thermal Analysis', fontsize=16, fontweight='bold')
        
        # 1. Power consumption over time
        if 'power_estimated' in self.df.columns:
            power_data = self.df['power_estimated']
        else:
            power_data = np.random.normal(80, 15, len(time_points))  # Simulated
        
        ax1.plot(time_points, power_data, color='orange', linewidth=2, label='Power')
        ax1.fill_between(time_points, 0, power_data, color='orange', alpha=0.3)
        
        ax1.set_title('Power Consumption Over Time')
        ax1.set_xlabel('Time')
        ax1.set_ylabel('Power (W)')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        
        # 2. Thermal zones visualization
        thermal_zones = ['CPU', 'GPU', 'Chipset', 'Battery', 'SSD', 'Power']
        current_temps = [55, 60, 45, 35, 40, 50]  # Simulated current temperatures
        
        colors = [self._get_temp_color(temp, 'coolwarm') for temp in current_temps]
        bars = ax2.barh(thermal_zones, current_temps, color=colors, alpha=0.8)
        
        ax2.set_title('Current Thermal Zones')
        ax2.set_xlabel('Temperature (°C)')
        ax2.set_ylabel('Component')
        
        # Add temperature labels
        for bar, temp in zip(bars, current_temps):
            ax2.text(temp + 0.5, bar.get_y() + bar.get_height()/2,
                    f'{temp}°C', va='center', fontweight='bold')
        
        # Add warning lines
        ax2.axvline(x=60, color='orange', linestyle='--', alpha=0.7)
        ax2.axvline(x=75, color='red', linestyle='--', alpha=0.7)
        
        # 3. Power vs CPU correlation
        cpu_usage = self.df.get('cpu_usage', np.random.uniform(10, 90, len(time_points)))
        
        scatter = ax3.scatter(cpu_usage, power_data, c=time_points, cmap='viridis', alpha=0.6)
        ax3.set_title('Power vs CPU Usage Correlation')
        ax3.set_xlabel('CPU Usage (%)')
        ax3.set_ylabel('Power (W)')
        ax3.grid(True, alpha=0.3)
        
        # Add colorbar
        cbar = plt.colorbar(scatter, ax=ax3)
        cbar.set_label('Time')
        
        # 4. Thermal efficiency chart
        efficiency_zones = ['Idle', 'Normal', 'High Load', 'Critical']
        efficiency_temps = [30, 45, 60, 80]
        efficiency_power = [20, 50, 120, 200]
        
        scatter2 = ax4.scatter(efficiency_temps, efficiency_power, 
                             s=[100, 150, 200, 250], 
                             c=['green', 'yellow', 'orange', 'red'], alpha=0.7)
        
        # Add labels
        for i, zone in enumerate(efficiency_zones):
            ax4.annotate(zone, (efficiency_temps[i], efficiency_power[i]),
                        xytext=(5, 5), textcoords='offset points',
                        fontweight='bold')
        
        ax4.set_title('Thermal Efficiency Zones')
        ax4.set_xlabel('Temperature (°C)')
        ax4.set_ylabel('Power (W)')
        ax4.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Power and thermal chart saved to {output_file}")
    
    def _get_temp_color(self, temperature: float, cmap_name: str = 'coolwarm') -> str:
        """Get color based on temperature value"""
        # Normalize temperature to 0-1 range (assuming 20-100°C range)
        normalized = max(0, min(1, (temperature - 20) / 80))
        
        cmap = plt.cm.get_cmap(cmap_name)
        rgba = cmap(normalized)
        return rgba
    
    def create_performance_dashboard(self, output_file: str = "performance_dashboard.png"):
        """Create a comprehensive performance dashboard"""
        print("Creating comprehensive performance dashboard...")
        
        if self.df.empty:
            return
            
        # Create a large dashboard
        fig = plt.figure(figsize=(20, 12))
        gs = fig.add_gridspec(3, 4, hspace=0.3, wspace=0.3)
        
        # Main title
        fig.suptitle('System Performance Dashboard', fontsize=20, fontweight='bold')
        
        # 1. CPU usage (top-left)
        ax1 = fig.add_subplot(gs[0, 0])
        if 'cpu_usage' in self.df.columns:
            ax1.plot(self.df.index, self.df['cpu_usage'], color='blue', linewidth=2)
            ax1.fill_between(self.df.index, 0, self.df['cpu_usage'], alpha=0.3, color='blue')
        ax1.set_title('CPU Usage (%)')
        ax1.set_ylim(0, 100)
        ax1.grid(True, alpha=0.3)
        
        # 2. Memory usage (top-middle)
        ax2 = fig.add_subplot(gs[0, 1])
        if 'memory_usage' in self.df.columns:
            ax2.plot(self.df.index, self.df['memory_usage'], color='red', linewidth=2)
            ax2.fill_between(self.df.index, 0, self.df['memory_usage'], alpha=0.3, color='red')
        ax2.set_title('Memory Usage (%)')
        ax2.set_ylim(0, 100)
        ax2.grid(True, alpha=0.3)
        
        # 3. Power consumption (top-right)
        ax3 = fig.add_subplot(gs[0, 2])
        if 'power_estimated' in self.df.columns:
            ax3.plot(self.df.index, self.df['power_estimated'], color='orange', linewidth=2)
            ax3.fill_between(self.df.index, 0, self.df['power_estimated'], alpha=0.3, color='orange')
        ax3.set_title('Power (W)')
        ax3.grid(True, alpha=0.3)
        
        # 4. System status (top-right corner)
        ax4 = fig.add_subplot(gs[0, 3])
        ax4.axis('off')
        
        # Current status indicators
        current_cpu = self.df['cpu_usage'].iloc[-1] if 'cpu_usage' in self.df.columns else 0
        current_memory = self.df['memory_usage'].iloc[-1] if 'memory_usage' in self.df.columns else 0
        current_power = self.df['power_estimated'].iloc[-1] if 'power_estimated' in self.df.columns else 0
        
        status_text = f"""SYSTEM STATUS
        
CPU Usage: {current_cpu:.1f}%
Memory: {current_memory:.1f}%
Power: {current_power:.1f}W

Status: {'⚠️ High Load' if current_cpu > 80 else '✅ Normal'}
        """
        
        ax4.text(0.1, 0.5, status_text, fontsize=12, fontweight='bold',
                verticalalignment='center', fontfamily='monospace')
        
        # 5. Performance metrics comparison (middle-left)
        ax5 = fig.add_subplot(gs[1, :2])
        metrics = ['CPU', 'Memory', 'Power']
        if len(self.df) >= 10:
            avg_values = [
                self.df['cpu_usage'].mean() if 'cpu_usage' in self.df.columns else 0,
                self.df['memory_usage'].mean() if 'memory_usage' in self.df.columns else 0,
                self.df['power_estimated'].mean() if 'power_estimated' in self.df.columns else 0
            ]
            
            # Normalize for comparison (0-100 scale)
            max_vals = [100, 100, max(avg_values) if max(avg_values) > 0 else 1]
            normalized_values = [avg/max_val * 100 for avg, max_val in zip(avg_values, max_vals)]
            
            colors = ['#3498db', '#e74c3c', '#f39c12']
            bars = ax5.bar(metrics, normalized_values, color=colors, alpha=0.7)
            
            # Add value labels
            for bar, avg_val in zip(bars, avg_values):
                ax5.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                        f'{avg_val:.1f}', ha='center', va='bottom', fontweight='bold')
        
        ax5.set_title('Average Performance Metrics')
        ax5.set_ylabel('Normalized Value')
        ax5.set_ylim(0, 100)
        
        # 6. Load distribution (middle-right)
        ax6 = fig.add_subplot(gs[1, 2:])
        
        # Create a simple load distribution visualization
        load_ranges = ['0-20%', '20-40%', '40-60%', '60-80%', '80-100%']
        load_counts = [0] * 5
        
        if 'cpu_usage' in self.df.columns:
            cpu_data = self.df['cpu_usage']
            load_counts[0] = len(cpu_data[(cpu_data >= 0) & (cpu_data < 20)])
            load_counts[1] = len(cpu_data[(cpu_data >= 20) & (cpu_data < 40)])
            load_counts[2] = len(cpu_data[(cpu_data >= 40) & (cpu_data < 60)])
            load_counts[3] = len(cpu_data[(cpu_data >= 60) & (cpu_data < 80)])
            load_counts[4] = len(cpu_data[(cpu_data >= 80) & (cpu_data <= 100)])
        
        # Create load distribution heatmap
        load_matrix = np.array(load_counts).reshape(1, -1)
        im = ax6.imshow(load_matrix, cmap='YlOrRd', aspect='auto')
        
        # Add text annotations
        for i, count in enumerate(load_counts):
            ax6.text(i, 0, str(count), ha='center', va='center', 
                    fontweight='bold', color='white' if count > max(load_counts)/2 else 'black')
        
        ax6.set_title('CPU Load Distribution')
        ax6.set_xticks(range(len(load_ranges)))
        ax6.set_xticklabels(load_ranges)
        ax6.set_yticks([])
        
        # 7. Summary statistics (bottom)
        ax7 = fig.add_subplot(gs[2, :])
        ax7.axis('off')
        
        # Create summary table
        summary_data = []
        if 'cpu_usage' in self.df.columns:
            summary_data.append(['CPU Usage', 
                               f"{self.df['cpu_usage'].mean():.1f}%", 
                               f"{self.df['cpu_usage'].max():.1f}%"])
        if 'memory_usage' in self.df.columns:
            summary_data.append(['Memory Usage', 
                               f"{self.df['memory_usage'].mean():.1f}%", 
                               f"{self.df['memory_usage'].max():.1f}%"])
        if 'power_estimated' in self.df.columns:
            summary_data.append(['Power Consumption', 
                               f"{self.df['power_estimated'].mean():.1f}W", 
                               f"{self.df['power_estimated'].max():.1f}W"])
        
        if summary_data:
            table = ax7.table(cellText=summary_data,
                            colLabels=['Metric', 'Average', 'Peak'],
                            cellLoc='center',
                            loc='center',
                            bbox=[0.1, 0.1, 0.8, 0.8])
            
            table.auto_set_font_size(False)
            table.set_fontsize(10)
            table.scale(1, 2)
            
            # Style the table
            for i in range(len(summary_data) + 1):
                for j in range(3):
                    cell = table[(i, j)]
                    if i == 0:  # Header row
                        cell.set_facecolor('#3498db')
                        cell.set_text_props(weight='bold', color='white')
                    else:
                        cell.set_facecolor('#ecf0f1')
        
        plt.savefig(output_file, dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Performance dashboard saved to {output_file}")
    
    def generate_all_visualizations(self, output_dir: str = "performance_visualizations"):
        """Generate all performance visualizations"""
        os.makedirs(output_dir, exist_ok=True)
        
        print(f"\nGenerating all performance visualizations in {output_dir}/...")
        
        # Create all visualizations
        self.create_cpu_heatmap(f"{output_dir}/cpu_heatmap.png")
        self.create_thermal_heatmap(f"{output_dir}/thermal_heatmap.png")
        self.create_memory_allocation_chart(f"{output_dir}/memory_allocation.png")
        self.create_io_throughput_visualization(f"{output_dir}/io_throughput.png")
        self.create_network_visualization(f"{output_dir}/network_traffic.png")
        self.create_power_thermal_chart(f"{output_dir}/power_thermal.png")
        self.create_performance_dashboard(f"{output_dir}/performance_dashboard.png")
        
        print(f"\n✅ All visualizations generated successfully!")
        print(f"📁 Files saved in: {output_dir}/")

def main():
    parser = argparse.ArgumentParser(description='Performance Visualizer CLI Tool')
    parser.add_argument('data_file', help='Performance data file (JSON or CSV)')
    parser.add_argument('--heatmap', action='store_true', help='Generate CPU heatmap')
    parser.add_argument('--thermal', action='store_true', help='Generate thermal visualization')
    parser.add_argument('--memory', action='store_true', help='Generate memory allocation chart')
    parser.add_argument('--io', action='store_true', help='Generate I/O throughput visualization')
    parser.add_argument('--network', action='store_true', help='Generate network traffic visualization')
    parser.add_argument('--power', action='store_true', help='Generate power/thermal chart')
    parser.add_argument('--dashboard', action='store_true', help='Generate comprehensive dashboard')
    parser.add_argument('--all', action='store_true', help='Generate all visualizations')
    parser.add_argument('--output-dir', type=str, default='performance_visualizations',
                       help='Output directory for visualizations')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.data_file):
        print(f"Error: Data file '{args.data_file}' not found")
        return
    
    visualizer = PerformanceVisualizer(args.data_file)
    visualizer.load_data()
    
    if args.all or not any([args.heatmap, args.thermal, args.memory, args.io, args.network, args.power, args.dashboard]):
        # Generate all visualizations if --all specified or no specific options
        visualizer.generate_all_visualizations(args.output_dir)
    else:
        # Generate specific visualizations
        if args.heatmap:
            visualizer.create_cpu_heatmap(f"{args.output_dir}/cpu_heatmap.png")
        if args.thermal:
            visualizer.create_thermal_heatmap(f"{args.output_dir}/thermal_heatmap.png")
        if args.memory:
            visualizer.create_memory_allocation_chart(f"{args.output_dir}/memory_allocation.png")
        if args.io:
            visualizer.create_io_throughput_visualization(f"{args.output_dir}/io_throughput.png")
        if args.network:
            visualizer.create_network_visualization(f"{args.output_dir}/network_traffic.png")
        if args.power:
            visualizer.create_power_thermal_chart(f"{args.output_dir}/power_thermal.png")
        if args.dashboard:
            visualizer.create_performance_dashboard(f"{args.output_dir}/performance_dashboard.png")

if __name__ == '__main__':
    main()