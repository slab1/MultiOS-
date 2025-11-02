#!/usr/bin/env python3
"""
Performance Monitor CLI Tool
Real-time system performance monitoring and analysis
"""

import argparse
import json
import time
import threading
from datetime import datetime, timedelta
from typing import Dict, List, Optional
import psutil
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from collections import deque
import sys
import os

class PerformanceMonitor:
    def __init__(self, interval: float = 1.0):
        self.interval = interval
        self.running = False
        self.data_buffer = deque(maxlen=3600)  # Store 1 hour of data at 1-second intervals
        self.current_data = {}
        
    def collect_cpu_data(self) -> Dict:
        """Collect CPU performance data"""
        cpu_percent = psutil.cpu_percent(interval=None)
        cpu_per_core = psutil.cpu_percent(interval=None, percpu=True)
        
        # CPU frequency
        cpu_freq = psutil.cpu_freq()
        freq_info = {
            'current': cpu_freq.current if cpu_freq else 0,
            'min': cpu_freq.min if cpu_freq else 0,
            'max': cpu_freq.max if cpu_freq else 0
        }
        
        # Load average (Unix-like systems)
        load_avg = None
        if hasattr(os, 'getloadavg'):
            load_avg = os.getloadavg()
        
        # CPU count
        cpu_count = psutil.cpu_count()
        cpu_count_logical = psutil.cpu_count(logical=True)
        
        return {
            'usage': cpu_percent,
            'per_core': cpu_per_core,
            'frequency': freq_info,
            'load_average': load_avg,
            'count': cpu_count,
            'count_logical': cpu_count_logical,
            'timestamp': datetime.now().isoformat()
        }
    
    def collect_memory_data(self) -> Dict:
        """Collect memory performance data"""
        svmem = psutil.virtual_memory()
        swap = psutil.swap_memory()
        
        return {
            'total': svmem.total,
            'available': svmem.available,
            'used': svmem.used,
            'percentage': svmem.percent,
            'free': svmem.free,
            'buffers': getattr(svmem, 'buffers', 0),
            'cached': getattr(svmem, 'cached', 0),
            'shared': getattr(svmem, 'shared', 0),
            'swap': {
                'total': swap.total,
                'used': swap.used,
                'free': swap.free,
                'percentage': swap.percent
            },
            'timestamp': datetime.now().isoformat()
        }
    
    def collect_disk_data(self) -> Dict:
        """Collect disk I/O performance data"""
        disk_io = psutil.disk_io_counters()
        disk_usage = {}
        
        # Get disk usage for all mounted drives
        partitions = psutil.disk_partitions()
        for partition in partitions:
            try:
                usage = psutil.disk_usage(partition.mountpoint)
                disk_usage[partition.device] = {
                    'total': usage.total,
                    'used': usage.used,
                    'free': usage.free,
                    'percentage': (usage.used / usage.total) * 100
                }
            except PermissionError:
                continue
        
        return {
            'io_counters': {
                'read_count': disk_io.read_count if disk_io else 0,
                'write_count': disk_io.write_count if disk_io else 0,
                'read_bytes': disk_io.read_bytes if disk_io else 0,
                'write_bytes': disk_io.write_bytes if disk_io else 0,
                'read_time': disk_io.read_time if disk_io else 0,
                'write_time': disk_io.write_time if disk_io else 0
            } if disk_io else {},
            'usage': disk_usage,
            'timestamp': datetime.now().isoformat()
        }
    
    def collect_network_data(self) -> Dict:
        """Collect network performance data"""
        net_io = psutil.net_io_counters()
        net_connections = len(psutil.net_connections())
        
        # Get network stats per interface
        net_if_stats = {}
        for interface, stats in psutil.net_if_stats().items():
            net_if_stats[interface] = {
                'is_up': stats.isup,
                'mtu': stats.mtu,
                'speed': stats.speed
            }
        
        return {
            'io_counters': {
                'bytes_sent': net_io.bytes_sent if net_io else 0,
                'bytes_recv': net_io.bytes_recv if net_io else 0,
                'packets_sent': net_io.packets_sent if net_io else 0,
                'packets_recv': net_io.packets_recv if net_io else 0,
                'errin': net_io.errin if net_io else 0,
                'errout': net_io.errout if net_io else 0,
                'dropin': net_io.dropin if net_io else 0,
                'dropout': net_io.dropout if net_io else 0
            } if net_io else {},
            'connections': net_connections,
            'interfaces': net_if_stats,
            'timestamp': datetime.now().isoformat()
        }
    
    def collect_process_data(self) -> List[Dict]:
        """Collect top processes data"""
        processes = []
        for proc in psutil.process_iter(['pid', 'name', 'cpu_percent', 'memory_percent', 'memory_info']):
            try:
                proc_info = proc.info
                if proc_info['cpu_percent'] > 0.1:  # Only include active processes
                    processes.append({
                        'pid': proc_info['pid'],
                        'name': proc_info['name'],
                        'cpu_percent': proc_info['cpu_percent'],
                        'memory_percent': proc_info['memory_percent'],
                        'memory_mb': proc_info['memory_info'].rss / 1024 / 1024 if proc_info['memory_info'] else 0
                    })
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        
        return sorted(processes, key=lambda x: x['cpu_percent'], reverse=True)[:20]
    
    def collect_temperature_data(self) -> Dict:
        """Collect temperature data (if available)"""
        temperatures = {}
        
        try:
            # Try to get sensor temperatures
            if hasattr(psutil, 'sensors_temperatures'):
                temps = psutil.sensors_temperatures()
                for name, entries in temps.items():
                    temperatures[name] = []
                    for entry in entries:
                        temperatures[name].append({
                            'label': entry.label,
                            'current': entry.current,
                            'high': entry.high,
                            'critical': entry.critical
                        })
        except:
            pass
        
        return {
            'sensors': temperatures,
            'timestamp': datetime.now().isoformat()
        }
    
    def collect_power_data(self) -> Dict:
        """Collect power/battery data"""
        power_info = {}
        
        try:
            battery = psutil.sensors_battery()
            if battery:
                power_info['battery'] = {
                    'percent': battery.percent,
                    'power_plugged': battery.power_plugged,
                    'time_left': battery.secsleft
                }
        except:
            pass
        
        # Additional power estimation (rough)
        cpu_data = self.collect_cpu_data()
        memory_data = self.collect_memory_data()
        
        # Estimate power consumption based on load
        cpu_power = cpu_data['usage'] * 0.5  # Rough estimate
        memory_power = (memory_data['used'] / memory_data['total']) * 10  # Rough estimate
        
        power_info['estimated_consumption'] = cpu_power + memory_power
        
        return {
            'power': power_info,
            'timestamp': datetime.now().isoformat()
        }
    
    def collect_all_data(self) -> Dict:
        """Collect all performance data"""
        return {
            'cpu': self.collect_cpu_data(),
            'memory': self.collect_memory_data(),
            'disk': self.collect_disk_data(),
            'network': self.collect_network_data(),
            'processes': self.collect_process_data(),
            'temperature': self.collect_temperature_data(),
            'power': self.collect_power_data()
        }
    
    def print_summary(self, data: Dict):
        """Print a formatted summary of performance data"""
        print(f"\n{'='*50}")
        print(f"Performance Summary - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"{'='*50}")
        
        # CPU
        cpu = data['cpu']
        print(f"\n📊 CPU:")
        print(f"   Usage: {cpu['usage']:.1f}%")
        if cpu['frequency']['current'] > 0:
            print(f"   Frequency: {cpu['frequency']['current']:.0f} MHz")
        if cpu['load_average']:
            print(f"   Load Average: {cpu['load_average'][0]:.2f}, {cpu['load_average'][1]:.2f}, {cpu['load_average'][2]:.2f}")
        
        # Memory
        memory = data['memory']
        total_gb = memory['total'] / (1024**3)
        used_gb = memory['used'] / (1024**3)
        print(f"\n💾 Memory:")
        print(f"   Usage: {used_gb:.1f}GB / {total_gb:.1f}GB ({memory['percentage']:.1f}%)")
        if memory['swap']['total'] > 0:
            swap_gb = memory['swap']['total'] / (1024**3)
            print(f"   Swap: {memory['swap']['used'] / (1024**3):.1f}GB / {swap_gb:.1f}GB ({memory['swap']['percentage']:.1f}%)")
        
        # Disk
        disk = data['disk']
        print(f"\n💿 Disk:")
        for device, usage in disk['usage'].items():
            used_gb = usage['used'] / (1024**3)
            total_gb = usage['total'] / (1024**3)
            print(f"   {device}: {used_gb:.1f}GB / {total_gb:.1f}GB ({usage['percentage']:.1f}%)")
        
        # Network
        network = data['network']
        print(f"\n🌐 Network:")
        net_io = network['io_counters']
        if net_io:
            print(f"   Bytes Sent: {net_io['bytes_sent'] / (1024**2):.1f} MB")
            print(f"   Bytes Received: {net_io['bytes_recv'] / (1024**2):.1f} MB")
        print(f"   Connections: {network['connections']}")
        
        # Temperature
        temp = data['temperature']
        print(f"\n🌡️  Temperature:")
        for name, entries in temp['sensors'].items():
            for entry in entries:
                print(f"   {name} - {entry['label']}: {entry['current']:.1f}°C")
        
        # Power
        power = data['power']
        print(f"\n⚡ Power:")
        if 'battery' in power['power']:
            battery = power['power']['battery']
            status = "Charging" if battery['power_plugged'] else "Battery"
            print(f"   {status}: {battery['percent']:.1f}%")
        print(f"   Estimated Consumption: {power['power']['estimated_consumption']:.1f}W")
        
        # Top Processes
        print(f"\n🔥 Top Processes:")
        for i, proc in enumerate(data['processes'][:5], 1):
            print(f"   {i:2d}. {proc['name']:<20} {proc['cpu_percent']:5.1f}% CPU  {proc['memory_mb']:6.1f}MB")
    
    def export_data(self, data: Dict, filename: str):
        """Export performance data to JSON file"""
        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)
        print(f"Data exported to {filename}")
    
    def start_monitoring(self, duration: Optional[int] = None):
        """Start continuous monitoring"""
        self.running = True
        start_time = time.time()
        
        print("Starting performance monitoring...")
        print(f"Collecting data every {self.interval} seconds")
        if duration:
            print(f"Will run for {duration} seconds")
        
        try:
            while self.running:
                data = self.collect_all_data()
                self.current_data = data
                self.data_buffer.append(data)
                
                # Print summary every 10 samples
                if len(self.data_buffer) % 10 == 0:
                    self.print_summary(data)
                
                # Check duration limit
                if duration and (time.time() - start_time) >= duration:
                    break
                
                time.sleep(self.interval)
                
        except KeyboardInterrupt:
            print("\nMonitoring stopped by user")
        finally:
            self.running = False
    
    def create_charts(self, output_dir: str = "performance_charts"):
        """Create performance charts from collected data"""
        if not self.data_buffer:
            print("No data available for charting")
            return
        
        os.makedirs(output_dir, exist_ok=True)
        
        # Extract time series data
        timestamps = [datetime.fromisoformat(d['timestamp']) for d in self.data_buffer]
        cpu_usage = [d['cpu']['usage'] for d in self.data_buffer]
        memory_usage = [d['memory']['percentage'] for d in self.data_buffer]
        
        # Create CPU usage chart
        plt.figure(figsize=(12, 6))
        plt.subplot(2, 1, 1)
        plt.plot(timestamps, cpu_usage, 'b-', linewidth=1)
        plt.title('CPU Usage Over Time')
        plt.ylabel('CPU Usage (%)')
        plt.grid(True)
        
        # Create Memory usage chart
        plt.subplot(2, 1, 2)
        plt.plot(timestamps, memory_usage, 'r-', linewidth=1)
        plt.title('Memory Usage Over Time')
        plt.ylabel('Memory Usage (%)')
        plt.xlabel('Time')
        plt.grid(True)
        
        plt.tight_layout()
        plt.savefig(f"{output_dir}/system_performance.png", dpi=150, bbox_inches='tight')
        plt.close()
        
        print(f"Charts saved to {output_dir}/")
    
    def analyze_trends(self):
        """Analyze performance trends from collected data"""
        if len(self.data_buffer) < 10:
            print("Not enough data for trend analysis")
            return
        
        print("\n" + "="*50)
        print("PERFORMANCE TREND ANALYSIS")
        print("="*50)
        
        # Calculate statistics for different metrics
        cpu_values = [d['cpu']['usage'] for d in self.data_buffer]
        memory_values = [d['memory']['percentage'] for d in self.data_buffer]
        
        # CPU trends
        print(f"\n📊 CPU Analysis:")
        print(f"   Average: {sum(cpu_values)/len(cpu_values):.1f}%")
        print(f"   Peak: {max(cpu_values):.1f}%")
        print(f"   Minimum: {min(cpu_values):.1f}%")
        print(f"   Standard Deviation: {(sum((x - sum(cpu_values)/len(cpu_values))**2 for x in cpu_values)/len(cpu_values))**0.5:.1f}%")
        
        # Memory trends
        print(f"\n💾 Memory Analysis:")
        print(f"   Average: {sum(memory_values)/len(memory_values):.1f}%")
        print(f"   Peak: {max(memory_values):.1f}%")
        print(f"   Minimum: {min(memory_values):.1f}%")
        
        # Detect anomalies
        cpu_mean = sum(cpu_values) / len(cpu_values)
        cpu_std = (sum((x - cpu_mean)**2 for x in cpu_values) / len(cpu_values))**0.5
        
        anomalies = []
        for i, (timestamp, cpu) in enumerate(zip(timestamps, cpu_values)):
            if abs(cpu - cpu_mean) > 2 * cpu_std:
                anomalies.append((timestamp, cpu, "High" if cpu > cpu_mean else "Low"))
        
        if anomalies:
            print(f"\n🚨 Anomalies Detected ({len(anomalies)}):")
            for timestamp, value, direction in anomalies[-5:]:  # Show last 5
                print(f"   {timestamp.strftime('%H:%M:%S')}: {value:.1f}% ({direction})")
        else:
            print(f"\n✅ No significant anomalies detected")

def main():
    parser = argparse.ArgumentParser(description='Performance Monitor CLI Tool')
    parser.add_argument('--interval', type=float, default=1.0, help='Sampling interval in seconds')
    parser.add_argument('--duration', type=int, help='Monitor for specified duration (seconds)')
    parser.add_argument('--export', type=str, help='Export data to JSON file')
    parser.add_argument('--chart', action='store_true', help='Generate performance charts')
    parser.add_argument('--analyze', action='store_true', help='Analyze performance trends')
    parser.add_argument('--once', action='store_true', help='Collect data once and exit')
    
    args = parser.parse_args()
    
    monitor = PerformanceMonitor(interval=args.interval)
    
    if args.once:
        # Collect data once
        data = monitor.collect_all_data()
        monitor.print_summary(data)
        
        if args.export:
            monitor.export_data(data, args.export)
    else:
        # Start monitoring
        monitor.start_monitoring(duration=args.duration)
        
        # Post-monitoring operations
        if args.chart:
            monitor.create_charts()
        
        if args.analyze and len(monitor.data_buffer) > 0:
            monitor.analyze_trends()
        
        if args.export:
            # Export all collected data
            all_data = {
                'monitoring_info': {
                    'start_time': monitor.data_buffer[0]['cpu']['timestamp'] if monitor.data_buffer else None,
                    'end_time': monitor.data_buffer[-1]['cpu']['timestamp'] if monitor.data_buffer else None,
                    'total_samples': len(monitor.data_buffer),
                    'interval': args.interval
                },
                'samples': list(monitor.data_buffer)
            }
            monitor.export_data(all_data, args.export)

if __name__ == '__main__':
    main()