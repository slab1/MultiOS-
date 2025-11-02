#!/usr/bin/env python3
"""
Performance Tools Demo Script
Demonstrates all features of the performance visualization system
"""

import os
import sys
import json
import time
import subprocess
from pathlib import Path
from datetime import datetime, timedelta
import random

# Add CLI path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'cli'))

try:
    from performance_monitor import PerformanceMonitor
except ImportError:
    print("Error: Cannot import performance_monitor. Make sure you're running from the correct directory.")
    sys.exit(1)

class PerformanceDemo:
    def __init__(self):
        self.base_dir = Path(__file__).parent
        self.data_dir = self.base_dir / "data"
        self.output_dir = self.base_dir / "output"
        
        # Ensure directories exist
        self.data_dir.mkdir(exist_ok=True)
        self.output_dir.mkdir(exist_ok=True)
        
        print("🎯 Performance Visualization Tools Demo")
        print("=" * 50)
    
    def generate_sample_data(self, filename: str, duration_minutes: int = 5):
        """Generate realistic sample performance data"""
        print(f"\n📊 Generating sample data: {filename}")
        print(f"   Duration: {duration_minutes} minutes")
        
        # Create sample data with realistic patterns
        samples = []
        interval_seconds = 10  # 10-second intervals
        total_samples = duration_minutes * 60 // interval_seconds
        
        base_time = datetime.now()
        
        for i in range(total_samples):
            timestamp = base_time + timedelta(seconds=i * interval_seconds)
            
            # Generate realistic CPU usage patterns
            base_cpu = 20 + 30 * (1 + 0.5 * random.random())  # Base 20-65%
            
            # Add some variation
            if i % 10 == 0:  # Periodic spike
                cpu_usage = min(95, base_cpu + random.randint(20, 40))
            else:
                cpu_usage = base_cpu + random.randint(-10, 10)
            
            cpu_usage = max(0, min(100, cpu_usage))
            
            # Generate core utilization (8 cores simulated)
            cpu_cores = []
            for core in range(8):
                core_usage = cpu_usage + random.randint(-20, 20)
                core_usage = max(0, min(100, core_usage))
                cpu_cores.append(core_usage)
            
            # Generate memory usage (base on CPU with some correlation)
            memory_base = 40 + (cpu_usage * 0.3)
            memory_usage = memory_base + random.randint(-15, 15)
            memory_usage = max(10, min(95, memory_usage))
            
            # Total system memory (16GB)
            total_memory = 16 * 1024**3
            used_memory = int((memory_usage / 100) * total_memory)
            
            # Generate network data
            network_upload = random.uniform(0.5, 5.0)  # Mbps
            network_download = random.uniform(1.0, 20.0)  # Mbps
            
            # Generate disk I/O
            disk_read = random.uniform(5, 50)  # MB/s
            disk_write = random.uniform(3, 30)  # MB/s
            
            # Generate power consumption (correlated with CPU)
            base_power = 45 + (cpu_usage * 1.2)
            power_usage = base_power + random.randint(-10, 10)
            power_usage = max(20, power_usage)
            
            # Generate temperatures
            cpu_temp = 35 + (cpu_usage * 0.4) + random.randint(-5, 5)
            gpu_temp = 40 + random.randint(-10, 20)
            
            sample = {
                "timestamp": timestamp.isoformat(),
                "cpu": {
                    "usage": round(cpu_usage, 1),
                    "per_core": [round(x, 1) for x in cpu_cores],
                    "frequency": {
                        "current": 2500 + random.randint(-200, 300),
                        "min": 1200,
                        "max": 3200
                    }
                },
                "memory": {
                    "total": total_memory,
                    "available": total_memory - used_memory,
                    "used": used_memory,
                    "percentage": round(memory_usage, 1),
                    "free": total_memory - used_memory - random.randint(1024**3, 2*1024**3),
                    "buffers": random.randint(512*1024**2, 2*1024**3),
                    "cached": random.randint(1024**3, 4*1024**3),
                    "shared": random.randint(100*1024**2, 500*1024**2),
                    "swap": {
                        "total": 8 * 1024**3,
                        "used": random.randint(0, 1024**3),
                        "free": 7 * 1024**3,
                        "percentage": random.randint(0, 15)
                    }
                },
                "disk": {
                    "io_counters": {
                        "read_count": random.randint(1000, 5000),
                        "write_count": random.randint(800, 4000),
                        "read_bytes": random.randint(100*1024**2, 1000*1024**2),
                        "write_bytes": random.randint(80*1024**2, 800*1024**2),
                        "read_time": random.randint(1000, 10000),
                        "write_time": random.randint(800, 8000)
                    },
                    "usage": {
                        "/dev/sda1": {
                            "total": 500 * 1024**3,
                            "used": random.randint(200*1024**3, 400*1024**3),
                            "free": random.randint(50*1024**3, 200*1024**3),
                            "percentage": random.randint(40, 85)
                        }
                    }
                },
                "network": {
                    "io_counters": {
                        "bytes_sent": random.randint(1024**2, 100*1024**2),
                        "bytes_recv": random.randint(5*1024**2, 500*1024**2),
                        "packets_sent": random.randint(1000, 10000),
                        "packets_recv": random.randint(5000, 50000),
                        "errin": random.randint(0, 10),
                        "errout": random.randint(0, 10),
                        "dropin": random.randint(0, 5),
                        "dropout": random.randint(0, 5)
                    },
                    "connections": random.randint(20, 100),
                    "interfaces": {
                        "eth0": {
                            "is_up": True,
                            "mtu": 1500,
                            "speed": 1000
                        }
                    }
                },
                "processes": [
                    {
                        "pid": 1000 + i,
                        "name": random.choice(["chrome", "firefox", "code", "python", "node", "systemd"]),
                        "cpu_percent": round(random.uniform(0.1, 15.0), 1),
                        "memory_percent": round(random.uniform(0.5, 8.0), 1),
                        "memory_info": {
                            "rss": random.randint(50*1024**2, 500*1024**2),
                            "vms": random.randint(100*1024**2, 1000*1024**2)
                        }
                    }
                ],
                "temperature": {
                    "sensors": {
                        "coretemp": [
                            {
                                "label": "Core 0",
                                "current": round(cpu_temp, 1),
                                "high": 85.0,
                                "critical": 105.0
                            },
                            {
                                "label": "Core 1",
                                "current": round(cpu_temp - 2, 1),
                                "high": 85.0,
                                "critical": 105.0
                            }
                        ],
                        "nvme": [
                            {
                                "label": "Composite",
                                "current": round(gpu_temp, 1),
                                "high": 85.0,
                                "critical": 105.0
                            }
                        ]
                    }
                },
                "power": {
                    "power": {
                        "estimated_consumption": round(power_usage, 1),
                        "battery": {
                            "percent": random.randint(15, 100),
                            "power_plugged": random.choice([True, False]),
                            "secsleft": random.randint(3600, 28800)
                        } if random.random() > 0.5 else {}
                    }
                }
            }
            
            samples.append(sample)
        
        # Save sample data
        sample_data = {
            "monitoring_info": {
                "start_time": base_time.isoformat(),
                "end_time": timestamp.isoformat(),
                "duration": duration_minutes * 60,
                "interval": interval_seconds,
                "sample_count": len(samples),
                "generated": True
            },
            "samples": samples
        }
        
        output_path = self.data_dir / f"{filename}.json"
        with open(output_path, 'w') as f:
            json.dump(sample_data, f, indent=2)
        
        print(f"   ✅ Sample data saved: {output_path}")
        return output_path
    
    def demo_monitoring(self, duration: int = 30):
        """Demonstrate real-time monitoring"""
        print(f"\n🎯 Demo: Real-time Performance Monitoring")
        print(f"   Duration: {duration} seconds")
        print(f"   Interval: 2 seconds")
        
        try:
            monitor = PerformanceMonitor(interval=2.0)
            print("\n⚡ Starting monitoring... (will run for 30 seconds)")
            print("   Press Ctrl+C to stop early\n")
            
            monitor.start_monitoring(duration=duration)
            
            print("\n✅ Monitoring completed!")
            
            # Export the data
            export_path = self.data_dir / f"demo_monitoring_{int(time.time())}.json"
            all_data = {
                "monitoring_info": {
                    "start_time": datetime.now().isoformat(),
                    "duration": duration,
                    "interval": 2.0,
                    "demo_mode": True
                },
                "samples": list(monitor.data_buffer)
            }
            
            with open(export_path, 'w') as f:
                json.dump(all_data, f, indent=2)
            
            print(f"💾 Data exported to: {export_path}")
            return export_path
            
        except KeyboardInterrupt:
            print("\n⚠️ Monitoring stopped by user")
            return None
        except Exception as e:
            print(f"❌ Error during monitoring: {e}")
            return None
    
    def demo_analysis(self, data_file: str):
        """Demonstrate analysis capabilities"""
        print(f"\n📊 Demo: Performance Analysis")
        print(f"   Data file: {data_file}")
        
        try:
            # Run analysis using the CLI tool
            cmd = [
                sys.executable, 
                "cli/performance_analyzer.py", 
                data_file,
                "--stats",
                "--anomalies", "iqr",
                "--chart",
                "--output-dir", "demo_analysis"
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=self.base_dir)
            
            if result.returncode == 0:
                print("   ✅ Analysis completed successfully!")
                print(result.stdout)
            else:
                print("   ⚠️ Analysis completed with warnings:")
                print(result.stderr)
            
            return result.returncode == 0
            
        except Exception as e:
            print(f"❌ Error during analysis: {e}")
            return False
    
    def demo_visualization(self, data_file: str):
        """Demonstrate visualization capabilities"""
        print(f"\n🎨 Demo: Performance Visualization")
        print(f"   Data file: {data_file}")
        
        try:
            # Run visualization using the CLI tool
            cmd = [
                sys.executable,
                "cli/performance_visualizer.py",
                data_file,
                "--all",
                "--output-dir", "demo_visualizations"
            ]
            
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=self.base_dir)
            
            if result.returncode == 0:
                print("   ✅ Visualizations generated successfully!")
                print(result.stdout)
            else:
                print("   ⚠️ Visualization completed with warnings:")
                print(result.stderr)
            
            return result.returncode == 0
            
        except Exception as e:
            print(f"❌ Error during visualization: {e}")
            return False
    
    def demo_cli_interface(self):
        """Demonstrate the master CLI interface"""
        print(f"\n🖥️  Demo: Master CLI Interface")
        
        # Test different CLI commands
        commands = [
            {
                "cmd": ["cli/performance_cli.py", "list"],
                "desc": "List available data files"
            },
            {
                "cmd": ["cli/performance_cli.py", "monitor", "--duration", "10", "--export", "quick_demo"],
                "desc": "Quick monitoring session"
            }
        ]
        
        results = []
        for command in commands:
            print(f"\n   🔧 Testing: {command['desc']}")
            try:
                result = subprocess.run(
                    command["cmd"], 
                    capture_output=True, 
                    text=True, 
                    cwd=self.base_dir,
                    timeout=30
                )
                results.append({
                    "command": command["cmd"],
                    "success": result.returncode == 0,
                    "output": result.stdout,
                    "error": result.stderr
                })
                
                if result.returncode == 0:
                    print(f"      ✅ Success")
                else:
                    print(f"      ⚠️ Issues detected")
                    
            except subprocess.TimeoutExpired:
                print(f"      ⏱️  Timeout (expected for monitoring)")
                results.append({
                    "command": command["cmd"],
                    "success": True,  # Timeout is expected for monitoring
                    "output": "Command timed out (expected)",
                    "error": ""
                })
            except Exception as e:
                print(f"      ❌ Error: {e}")
                results.append({
                    "command": command["cmd"],
                    "success": False,
                    "output": "",
                    "error": str(e)
                })
        
        return results
    
    def show_dashboard_info(self):
        """Show information about the web dashboard"""
        dashboard_path = self.base_dir / "dashboard" / "index.html"
        
        print(f"\n🌐 Web Dashboard Information")
        print(f"   Dashboard location: {dashboard_path}")
        
        if dashboard_path.exists():
            print(f"   ✅ Dashboard files found")
            print(f"\n📋 Dashboard Features:")
            print(f"   • Real-time performance metrics")
            print(f"   • Interactive charts and graphs")
            print(f"   • CPU utilization heatmaps")
            print(f"   • Memory usage patterns")
            print(f"   • I/O throughput visualization")
            print(f"   • Network traffic analysis")
            print(f"   • Power and thermal monitoring")
            print(f"   • Historical data trending")
            print(f"   • Export capabilities")
            
            print(f"\n🚀 To start the dashboard:")
            print(f"   cd /workspace/education/performance")
            print(f"   python cli/performance_cli.py dashboard")
            print(f"   # Then open http://localhost:8080")
        else:
            print(f"   ❌ Dashboard files not found")
    
    def create_comprehensive_demo(self):
        """Run comprehensive demo of all features"""
        print("\n🎯 COMPREHENSIVE DEMO - All Features")
        print("=" * 50)
        
        # Step 1: Generate sample data
        sample_files = []
        sample_files.append(self.generate_sample_data("demo_normal", 3))  # 3 minutes of normal usage
        sample_files.append(self.generate_sample_data("demo_high_load", 2))  # 2 minutes of high load
        
        # Step 2: Run analysis on sample data
        for sample_file in sample_files:
            if sample_file:
                self.demo_analysis(str(sample_file))
                self.demo_visualization(str(sample_file))
        
        # Step 3: Real-time monitoring (shorter for demo)
        print(f"\n⚡ Starting brief real-time monitoring demo...")
        print(f"(This will collect data for 20 seconds)")
        monitoring_data = self.demo_monitoring(20)
        
        # Step 4: Test CLI interface
        self.demo_cli_interface()
        
        # Step 5: Show dashboard info
        self.show_dashboard_info()
        
        # Summary
        print(f"\n🎉 DEMO COMPLETED!")
        print(f"=" * 50)
        print(f"\n📁 Generated Files:")
        print(f"   Sample data: {self.data_dir}")
        print(f"   Analysis output: {self.output_dir}")
        print(f"   Visualizations: demo_visualizations/ (in current directory)")
        
        print(f"\n📊 Files Generated:")
        for sample_file in sample_files:
            if sample_file:
                print(f"   • {sample_file.name}")
        
        if monitoring_data:
            print(f"   • monitoring data (from real-time collection)")
        
        print(f"\n🔍 Next Steps:")
        print(f"   1. Review generated analysis and visualizations")
        print(f"   2. Start the web dashboard: python cli/performance_cli.py dashboard")
        print(f"   3. Try manual monitoring: python cli/performance_monitor.py --duration 60")
        print(f"   4. Analyze your own data: python cli/performance_analyzer.py your_data.json")
        
    def interactive_demo(self):
        """Run interactive demo with user choices"""
        print(f"\n🎯 INTERACTIVE DEMO")
        print("=" * 50)
        
        while True:
            print(f"\n📋 Demo Options:")
            print(f"1. Generate sample data")
            print(f"2. Real-time monitoring demo")
            print(f"3. Analyze existing data")
            print(f"4. Generate visualizations")
            print(f"5. Test CLI interface")
            print(f"6. Show dashboard info")
            print(f"7. Run full demo")
            print(f"8. Exit")
            
            try:
                choice = input(f"\nSelect option (1-8): ").strip()
                
                if choice == '1':
                    duration = input(f"Duration in minutes (default 3): ").strip()
                    duration = int(duration) if duration.isdigit() else 3
                    filename = input(f"Filename (default demo_data): ").strip() or "demo_data"
                    self.generate_sample_data(filename, duration)
                
                elif choice == '2':
                    duration = input(f"Duration in seconds (default 30): ").strip()
                    duration = int(duration) if duration.isdigit() else 30
                    self.demo_monitoring(duration)
                
                elif choice == '3':
                    data_file = input(f"Data file path: ").strip()
                    if data_file:
                        self.demo_analysis(data_file)
                
                elif choice == '4':
                    data_file = input(f"Data file path: ").strip()
                    if data_file:
                        self.demo_visualization(data_file)
                
                elif choice == '5':
                    self.demo_cli_interface()
                
                elif choice == '6':
                    self.show_dashboard_info()
                
                elif choice == '7':
                    self.create_comprehensive_demo()
                    break
                
                elif choice == '8':
                    print(f"\n👋 Demo ended. Thank you!")
                    break
                
                else:
                    print(f"❌ Invalid choice. Please select 1-8.")
            
            except KeyboardInterrupt:
                print(f"\n\n⚠️ Demo interrupted by user")
                break
            except Exception as e:
                print(f"❌ Error: {e}")

def main():
    """Main demo entry point"""
    demo = PerformanceDemo()
    
    if len(sys.argv) > 1:
        option = sys.argv[1].lower()
        
        if option == 'sample':
            filename = sys.argv[2] if len(sys.argv) > 2 else "demo_sample"
            duration = int(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3].isdigit() else 3
            demo.generate_sample_data(filename, duration)
        
        elif option == 'monitor':
            duration = int(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2].isdigit() else 30
            demo.demo_monitoring(duration)
        
        elif option == 'analyze':
            if len(sys.argv) > 2:
                demo.demo_analysis(sys.argv[2])
            else:
                print("Usage: python demo.py analyze <data_file>")
        
        elif option == 'visualize':
            if len(sys.argv) > 2:
                demo.demo_visualization(sys.argv[2])
            else:
                print("Usage: python demo.py visualize <data_file>")
        
        elif option == 'full':
            demo.create_comprehensive_demo()
        
        elif option == 'interactive':
            demo.interactive_demo()
        
        else:
            print("Usage: python demo.py [sample|monitor|analyze|visualize|full|interactive]")
    
    else:
        # Default: run interactive demo
        demo.interactive_demo()

if __name__ == '__main__':
    main()