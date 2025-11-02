#!/usr/bin/env python3
"""
Performance CLI - Master Interface
Comprehensive CLI interface for performance monitoring and analysis
"""

import argparse
import sys
import os
import subprocess
from pathlib import Path
from typing import Optional, List
import json
from datetime import datetime

# Add the CLI directory to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    from performance_monitor import PerformanceMonitor
    from performance_analyzer import PerformanceAnalyzer
    from performance_visualizer import PerformanceVisualizer
except ImportError:
    print("Error: Could not import required modules. Make sure all CLI tools are available.")
    sys.exit(1)

class PerformanceCLI:
    def __init__(self):
        self.data_dir = Path("/workspace/education/performance/data")
        self.output_dir = Path("/workspace/education/performance/output")
        self.data_dir.mkdir(exist_ok=True)
        self.output_dir.mkdir(exist_ok=True)
    
    def monitor(self, duration: int = None, interval: float = 2.0, export: str = None):
        """Start performance monitoring"""
        print(f"🎯 Starting Performance Monitor")
        print(f"   Interval: {interval}s")
        if duration:
            print(f"   Duration: {duration}s")
        print()
        
        try:
            monitor = PerformanceMonitor(interval=interval)
            
            if export:
                export_path = self.data_dir / f"{export}.json"
                if duration:
                    # Monitor for specific duration
                    monitor.start_monitoring(duration=duration)
                    all_data = {
                        'monitoring_info': {
                            'start_time': datetime.now().isoformat(),
                            'duration': duration,
                            'interval': interval
                        },
                        'samples': list(monitor.data_buffer)
                    }
                    with open(export_path, 'w') as f:
                        json.dump(all_data, f, indent=2)
                else:
                    # Manual monitoring with keyboard interrupt
                    data = monitor.collect_all_data()
                    monitor.print_summary(data)
                    with open(export_path, 'w') as f:
                        json.dump(data, f, indent=2)
                
                print(f"💾 Data exported to: {export_path}")
            else:
                monitor.start_monitoring(duration=duration)
        
        except KeyboardInterrupt:
            print("\n⚠️ Monitoring stopped by user")
        except Exception as e:
            print(f"❌ Error during monitoring: {e}")
    
    def analyze(self, data_file: str, stats: bool = True, anomalies: str = None, 
                threshold: float = 2.0, compare: List[str] = None, 
                export: str = None):
        """Analyze performance data"""
        data_path = self._resolve_data_file(data_file)
        if not data_path:
            return
        
        print(f"📊 Analyzing Performance Data: {data_path.name}")
        print()
        
        try:
            analyzer = PerformanceAnalyzer(str(data_path))
            analyzer.load_data()
            
            if stats:
                analyzer.generate_statistics()
            
            if anomalies:
                analyzer.detect_anomalies(method=anomalies, threshold=threshold)
            
            if compare:
                if len(compare) != 4:
                    print("❌ Error: --compare requires 4 datetime arguments (start1 end1 start2 end2)")
                    print("   Example: 2023-01-01T00:00 2023-01-01T01:00 2023-01-01T02:00 2023-01-01T03:00")
                    return
                analyzer.compare_periods(*compare)
            
            if export:
                export_path = self.output_dir / f"{export}_analysis.json"
                analyzer.export_analysis(str(export_path))
        
        except Exception as e:
            print(f"❌ Error during analysis: {e}")
    
    def visualize(self, data_file: str, charts: List[str] = None, output_dir: str = None):
        """Generate performance visualizations"""
        data_path = self._resolve_data_file(data_file)
        if not data_path:
            return
        
        if output_dir:
            output_path = Path(output_dir)
        else:
            output_path = self.output_dir / data_path.stem
        
        print(f"🎨 Generating Performance Visualizations")
        print(f"   Data: {data_path.name}")
        print(f"   Output: {output_path}")
        print()
        
        try:
            visualizer = PerformanceVisualizer(str(data_path))
            visualizer.load_data()
            
            if not charts:
                # Generate all visualizations
                visualizer.generate_all_visualizations(str(output_path))
            else:
                # Generate specific charts
                for chart in charts:
                    if chart == 'heatmap':
                        visualizer.create_cpu_heatmap(str(output_path / "cpu_heatmap.png"))
                    elif chart == 'thermal':
                        visualizer.create_thermal_heatmap(str(output_path / "thermal_heatmap.png"))
                    elif chart == 'memory':
                        visualizer.create_memory_allocation_chart(str(output_path / "memory_allocation.png"))
                    elif chart == 'io':
                        visualizer.create_io_throughput_visualization(str(output_path / "io_throughput.png"))
                    elif chart == 'network':
                        visualizer.create_network_visualization(str(output_path / "network_traffic.png"))
                    elif chart == 'power':
                        visualizer.create_power_thermal_chart(str(output_path / "power_thermal.png"))
                    elif chart == 'dashboard':
                        visualizer.create_performance_dashboard(str(output_path / "performance_dashboard.png"))
                    else:
                        print(f"⚠️ Unknown chart type: {chart}")
            
            print(f"✅ Visualizations completed!")
        
        except Exception as e:
            print(f"❌ Error during visualization: {e}")
    
    def dashboard(self, port: int = 8080, host: str = 'localhost'):
        """Start the web dashboard"""
        dashboard_path = Path(__file__).parent.parent / 'dashboard' / 'index.html'
        
        if not dashboard_path.exists():
            print("❌ Dashboard files not found. Please ensure dashboard is properly installed.")
            return
        
        print(f"🌐 Starting Performance Dashboard")
        print(f"   URL: http://{host}:{port}")
        print(f"   Dashboard: {dashboard_path}")
        print()
        print("Press Ctrl+C to stop the dashboard")
        
        try:
            # For a real deployment, you would use a web server like Python's http.server
            # For now, we'll just provide instructions
            print(f"""
📋 Dashboard Instructions:

1. The web dashboard is located at: {dashboard_path}
2. Open the dashboard in your web browser
3. The dashboard shows real-time performance metrics
4. It includes:
   - CPU utilization and core heatmap
   - Memory usage and allocation breakdown
   - I/O throughput with device breakdown
   - Network traffic with protocol analysis
   - Power consumption and thermal monitoring
   - Performance analysis tools
   - Historical trending and comparison

5. To run the dashboard with a local server:
   cd {dashboard_path.parent}
   python -m http.server {port}

6. Then open: http://localhost:{port}/dashboard/index.html
            """)
            
            # Try to start a simple HTTP server
            import webbrowser
            import threading
            import time
            from http.server import HTTPServer, SimpleHTTPRequestHandler
            
            os.chdir(dashboard_path.parent)
            
            def start_server():
                try:
                    server = HTTPServer((host, port), SimpleHTTPRequestHandler)
                    print(f"🚀 Dashboard server started on http://{host}:{port}")
                    server.serve_forever()
                except Exception as e:
                    print(f"⚠️ Could not start server: {e}")
                    print(f"Please open the dashboard manually: file://{dashboard_path}")
            
            # Start server in background
            server_thread = threading.Thread(target=start_server, daemon=True)
            server_thread.start()
            
            # Wait a moment for server to start
            time.sleep(1)
            
            # Try to open in browser
            try:
                webbrowser.open(f'http://{host}:{port}/dashboard/index.html')
            except:
                pass
            
            try:
                input("\nPress Enter to stop the server...")
            except KeyboardInterrupt:
                pass
        
        except Exception as e:
            print(f"❌ Error starting dashboard: {e}")
    
    def compare(self, file1: str, file2: str, output: str = None):
        """Compare two performance datasets"""
        path1 = self._resolve_data_file(file1)
        path2 = self._resolve_data_file(file2)
        
        if not path1 or not path2:
            return
        
        print(f"🔍 Comparing Performance Data")
        print(f"   File 1: {path1.name}")
        print(f"   File 2: {path2.name}")
        print()
        
        try:
            # Load both datasets
            analyzer1 = PerformanceAnalyzer(str(path1))
            analyzer1.load_data()
            
            analyzer2 = PerformanceAnalyzer(str(path2))
            analyzer2.load_data()
            
            # Compare statistics
            print("COMPARISON RESULTS:")
            print("=" * 50)
            
            if analyzer1.df is not None and analyzer2.df is not None:
                metrics = ['cpu_usage', 'memory_usage', 'power_estimated']
                
                for metric in metrics:
                    if metric in analyzer1.df.columns and metric in analyzer2.df.columns:
                        avg1 = analyzer1.df[metric].mean()
                        avg2 = analyzer2.df[metric].mean()
                        change = ((avg2 - avg1) / avg1) * 100
                        
                        print(f"\n{metric.upper()}:")
                        print(f"  {path1.name}: {avg1:.2f}")
                        print(f"  {path2.name}: {avg2:.2f}")
                        print(f"  Change: {change:+.1f}%")
                
                if output:
                    comparison_data = {
                        'comparison_date': datetime.now().isoformat(),
                        'file1': str(path1),
                        'file2': str(path2),
                        'file1_stats': {
                            metric: float(analyzer1.df[metric].mean()) 
                            for metric in metrics 
                            if metric in analyzer1.df.columns
                        },
                        'file2_stats': {
                            metric: float(analyzer2.df[metric].mean()) 
                            for metric in metrics 
                            if metric in analyzer2.df.columns
                        }
                    }
                    
                    output_path = self.output_dir / f"{output}_comparison.json"
                    with open(output_path, 'w') as f:
                        json.dump(comparison_data, f, indent=2)
                    
                    print(f"\n💾 Comparison saved to: {output_path}")
        
        except Exception as e:
            print(f"❌ Error during comparison: {e}")
    
    def list_data(self):
        """List available performance data files"""
        print("📁 Available Performance Data Files:")
        print("=" * 40)
        
        if not self.data_dir.exists():
            print("No data directory found.")
            return
        
        data_files = list(self.data_dir.glob("*.json"))
        
        if not data_files:
            print("No data files found.")
            return
        
        for file_path in sorted(data_files):
            try:
                # Get file stats
                stat = file_path.stat()
                size = stat.st_size
                modified = datetime.fromtimestamp(stat.st_m_time)
                
                # Try to get sample count
                sample_count = "?"
                with open(file_path, 'r') as f:
                    data = json.load(f)
                    if 'samples' in data:
                        sample_count = len(data['samples'])
                    elif 'cpu' in data:
                        sample_count = 1
                
                print(f"  📄 {file_path.name:<30} {sample_count:>4} samples  {size:>8} bytes  {modified.strftime('%Y-%m-%d %H:%M')}")
            
            except Exception as e:
                print(f"  ❌ {file_path.name:<30} Error reading file")
    
    def clean(self, older_than_days: int = 30):
        """Clean old performance data files"""
        print(f"🧹 Cleaning performance data older than {older_than_days} days...")
        
        if not self.data_dir.exists():
            print("No data directory found.")
            return
        
        cutoff_date = datetime.now().timestamp() - (older_than_days * 24 * 3600)
        
        removed_files = 0
        total_size = 0
        
        for file_path in self.data_dir.glob("*.json"):
            if file_path.stat().st_mtime < cutoff_date:
                size = file_path.stat().st_size
                file_path.unlink()
                removed_files += 1
                total_size += size
                print(f"  🗑️  Removed: {file_path.name}")
        
        print(f"\n✅ Cleanup completed:")
        print(f"   Files removed: {removed_files}")
        print(f"   Space freed: {total_size / (1024*1024):.1f} MB")
    
    def _resolve_data_file(self, filename: str) -> Optional[Path]:
        """Resolve data file path"""
        if not filename:
            print("❌ Error: No data file specified")
            return None
        
        # Check if absolute path
        path = Path(filename)
        if path.exists():
            return path
        
        # Check in data directory
        path = self.data_dir / filename
        if path.exists():
            return path
        
        # Check with .json extension
        path = self.data_dir / f"{filename}.json"
        if path.exists():
            return path
        
        print(f"❌ Error: Data file not found: {filename}")
        print(f"   Searched in:")
        print(f"   - {filename}")
        print(f"   - {self.data_dir / filename}")
        print(f"   - {self.data_dir / f'{filename}.json'}")
        
        return None

def create_parser():
    """Create command line argument parser"""
    parser = argparse.ArgumentParser(
        description="Performance CLI - Comprehensive Performance Monitoring and Analysis",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Monitor system for 60 seconds
  %(prog)s monitor --duration 60 --export mydata
  
  # Analyze existing data
  %(prog)s analyze mydata.json --stats --anomalies iqr
  
  # Generate visualizations
  %(prog)s visualize mydata.json --all
  
  # Start web dashboard
  %(prog)s dashboard
  
  # Compare two datasets
  %(prog)s compare data1.json data2.json --output comparison
  
Data Files:
  Data files are stored in: /workspace/education/performance/data/
  Output files are saved in: /workspace/education/performance/output/
        """
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Monitor command
    monitor_parser = subparsers.add_parser('monitor', help='Start performance monitoring')
    monitor_parser.add_argument('--duration', type=int, help='Monitor duration in seconds')
    monitor_parser.add_argument('--interval', type=float, default=2.0, help='Sampling interval in seconds')
    monitor_parser.add_argument('--export', type=str, help='Export data to file (without extension)')
    
    # Analyze command
    analyze_parser = subparsers.add_parser('analyze', help='Analyze performance data')
    analyze_parser.add_argument('data_file', help='Performance data file')
    analyze_parser.add_argument('--stats', action='store_true', help='Generate comprehensive statistics')
    analyze_parser.add_argument('--anomalies', choices=['iqr', 'zscore'], help='Detect anomalies')
    analyze_parser.add_argument('--threshold', type=float, default=2.0, help='Z-score threshold')
    analyze_parser.add_argument('--compare', nargs=4, metavar=('START1', 'END1', 'START2', 'END2'), 
                               help='Compare time periods')
    analyze_parser.add_argument('--export', type=str, help='Export analysis results')
    
    # Visualize command
    visualize_parser = subparsers.add_parser('visualize', help='Generate performance visualizations')
    visualize_parser.add_argument('data_file', help='Performance data file')
    visualize_parser.add_argument('--charts', nargs='+', 
                                 choices=['heatmap', 'thermal', 'memory', 'io', 'network', 'power', 'dashboard'],
                                 help='Specific charts to generate')
    visualize_parser.add_argument('--output-dir', type=str, help='Output directory')
    
    # Dashboard command
    dashboard_parser = subparsers.add_parser('dashboard', help='Start web dashboard')
    dashboard_parser.add_argument('--port', type=int, default=8080, help='Port number')
    dashboard_parser.add_argument('--host', type=str, default='localhost', help='Host address')
    
    # Compare command
    compare_parser = subparsers.add_parser('compare', help='Compare performance datasets')
    compare_parser.add_argument('file1', help='First data file')
    compare_parser.add_argument('file2', help='Second data file')
    compare_parser.add_argument('--output', type=str, help='Output file for comparison results')
    
    # List command
    list_parser = subparsers.add_parser('list', help='List available performance data files')
    
    # Clean command
    clean_parser = subparsers.add_parser('clean', help='Clean old performance data files')
    clean_parser.add_argument('--older-than', type=int, default=30, help='Clean files older than N days')
    
    return parser

def main():
    """Main entry point"""
    parser = create_parser()
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    cli = PerformanceCLI()
    
    try:
        if args.command == 'monitor':
            cli.monitor(duration=args.duration, interval=args.interval, export=args.export)
        
        elif args.command == 'analyze':
            cli.analyze(args.data_file, stats=args.stats, anomalies=args.anomalies,
                       threshold=args.threshold, compare=args.compare, export=args.export)
        
        elif args.command == 'visualize':
            cli.visualize(args.data_file, charts=args.charts, output_dir=args.output_dir)
        
        elif args.command == 'dashboard':
            cli.dashboard(port=args.port, host=args.host)
        
        elif args.command == 'compare':
            cli.compare(args.file1, args.file2, output=args.output)
        
        elif args.command == 'list':
            cli.list_data()
        
        elif args.command == 'clean':
            cli.clean(older_than_days=args.older_than)
    
    except KeyboardInterrupt:
        print("\n⚠️ Operation cancelled by user")
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == '__main__':
    main()