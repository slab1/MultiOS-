#!/usr/bin/env python3
"""
System utilities and configuration for stress testing
"""

import os
import json
import time
import psutil
import threading
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, asdict
import multiprocessing
import socket


@dataclass
class StressTestConfig:
    """Configuration for stress testing suite"""
    # Test configuration
    test_duration: int = 300  # seconds
    test_dir: str = "/tmp/stress_test"
    output_dir: str = "./stress_test_results"
    parallel_threads: int = 4
    verbose: bool = False
    
    # Memory test configuration
    max_memory_allocation_mb: int = 512
    min_available_memory_mb: int = 1024
    memory_leak_iterations: int = 1000
    fragmentation_test_size_mb: int = 256
    
    # File system test configuration
    min_available_disk_gb: int = 10
    file_io_test_size_mb: int = 100
    concurrent_file_access_threads: int = 10
    max_file_handles: int = 1024
    
    # CPU test configuration
    cpu_stress_duration: int = 60
    cpu_threads_per_core: int = 2
    thermal_test_duration: int = 30
    
    # Resource exhaustion configuration
    max_processes: int = 100
    max_network_connections: int = 1000
    
    # System resource limits
    cpu_usage_warning_threshold: float = 90.0
    memory_usage_warning_threshold: float = 85.0
    disk_usage_warning_threshold: float = 90.0
    
    @classmethod
    def from_file(cls, config_file: str) -> 'StressTestConfig':
        """Load configuration from JSON file"""
        if not Path(config_file).exists():
            raise FileNotFoundError(f"Config file not found: {config_file}")
        
        with open(config_file, 'r') as f:
            config_data = json.load(f)
        
        return cls(**config_data)
    
    def to_file(self, config_file: str):
        """Save configuration to JSON file"""
        with open(config_file, 'w') as f:
            json.dump(asdict(self), f, indent=2)


class SystemMonitor:
    """System resource monitoring for stress testing"""
    
    def __init__(self, config: StressTestConfig):
        self.config = config
        self.monitoring = False
        self.data_points = []
        self.monitoring_thread = None
        
        # System baseline
        self.baseline = self._collect_baseline()
    
    def _collect_baseline(self) -> Dict[str, Any]:
        """Collect system baseline metrics"""
        try:
            memory = psutil.virtual_memory()
            disk = psutil.disk_usage(self.config.test_dir)
            cpu_percent = psutil.cpu_percent(interval=1)
            cpu_count = multiprocessing.cpu_count()
            
            return {
                "timestamp": time.time(),
                "memory": {
                    "total": memory.total,
                    "available": memory.available,
                    "percent": memory.percent,
                    "used": memory.used,
                    "free": memory.free
                },
                "cpu": {
                    "count": cpu_count,
                    "usage_percent": cpu_percent
                },
                "disk": {
                    "total": disk.total,
                    "free": disk.free,
                    "used": disk.used,
                    "percent": (disk.used / disk.total) * 100
                },
                "processes": len(psutil.pids()),
                "network": self._get_network_stats()
            }
        except Exception:
            return {}
    
    def _get_network_stats(self) -> Dict[str, Any]:
        """Get network statistics"""
        try:
            net_io = psutil.net_io_counters()
            return {
                "bytes_sent": net_io.bytes_sent if net_io else 0,
                "bytes_recv": net_io.bytes_recv if net_io else 0,
                "packets_sent": net_io.packets_sent if net_io else 0,
                "packets_recv": net_io.packets_recv if net_io else 0
            }
        except Exception:
            return {}
    
    def start_monitoring(self):
        """Start continuous system monitoring"""
        if self.monitoring:
            return
        
        self.monitoring = True
        self.data_points = []
        self.monitoring_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.monitoring_thread.start()
    
    def stop_monitoring(self) -> Dict[str, Dict[str, Any]]:
        """Stop monitoring and return collected data"""
        self.monitoring = False
        
        if self.monitoring_thread:
            self.monitoring_thread.join(timeout=5)
        
        return self._aggregate_monitoring_data()
    
    def _monitor_loop(self):
        """Main monitoring loop"""
        while self.monitoring:
            try:
                data_point = {
                    "timestamp": time.time(),
                    "memory": dict(psutil.virtual_memory()._asdict()),
                    "cpu": {
                        "percent": psutil.cpu_percent(interval=None),
                        "count": multiprocessing.cpu_count()
                    },
                    "disk": dict(psutil.disk_usage(self.config.test_dir)._asdict()),
                    "processes": len(psutil.pids()),
                    "network": self._get_network_stats()
                }
                
                # Add per-CPU stats
                try:
                    data_point["cpu"]["per_cpu"] = psutil.cpu_percent(interval=None, percpu=True)
                except Exception:
                    data_point["cpu"]["per_cpu"] = []
                
                self.data_points.append(data_point)
                time.sleep(0.5)  # Monitor every 500ms
                
            except Exception as e:
                print(f"Monitoring error: {e}")
                break
    
    def _aggregate_monitoring_data(self) -> Dict[str, Dict[str, Any]]:
        """Aggregate monitoring data into summary statistics"""
        if not self.data_points:
            return {"memory": {}, "cpu": {}, "disk": {}}
        
        try:
            # Memory statistics
            memory_values = [dp["memory"]["percent"] for dp in self.data_points]
            cpu_values = [dp["cpu"]["percent"] for dp in self.data_points]
            disk_values = [dp["disk"].percent for dp in self.data_points]
            
            return {
                "memory": {
                    "avg_percent": sum(memory_values) / len(memory_values),
                    "max_percent": max(memory_values),
                    "min_percent": min(memory_values),
                    "peak_usage_mb": max([dp["memory"]["used"] for dp in self.data_points]) / (1024*1024),
                    "baseline": self.baseline.get("memory", {})
                },
                "cpu": {
                    "avg_percent": sum(cpu_values) / len(cpu_values),
                    "max_percent": max(cpu_values),
                    "min_percent": min(cpu_values),
                    "baseline": self.baseline.get("cpu", {})
                },
                "disk": {
                    "avg_percent": sum(disk_values) / len(disk_values),
                    "max_percent": max(disk_values),
                    "baseline": self.baseline.get("disk", {})
                }
            }
        except Exception:
            return {"memory": {}, "cpu": {}, "disk": {}}
    
    def get_system_info(self) -> Dict[str, Any]:
        """Get comprehensive system information"""
        try:
            # Basic system info
            boot_time = datetime.fromtimestamp(psutil.boot_time())
            uptime = time.time() - psutil.boot_time()
            
            # Memory info
            memory = psutil.virtual_memory()
            
            # CPU info
            cpu_count = multiprocessing.cpu_count()
            cpu_freq = psutil.cpu_freq()
            
            # Disk info
            disk = psutil.disk_usage(self.config.test_dir)
            
            # Network info
            net_io = psutil.net_io_counters()
            
            # Process info
            process_count = len(psutil.pids())
            
            return {
                "system": {
                    "platform": os.uname().sysname,
                    "release": os.uname().release,
                    "version": os.uname().version,
                    "machine": os.uname().machine,
                    "hostname": socket.gethostname()
                },
                "boot_time": boot_time.isoformat(),
                "uptime_seconds": uptime,
                "memory": {
                    "total_gb": memory.total / (1024**3),
                    "available_gb": memory.available / (1024**3),
                    "percent": memory.percent
                },
                "cpu": {
                    "count": cpu_count,
                    "frequency_mhz": cpu_freq.current if cpu_freq else 0,
                    "frequency_max_mhz": cpu_freq.max if cpu_freq else 0,
                    "frequency_min_mhz": cpu_freq.min if cpu_freq else 0
                },
                "disk": {
                    "total_gb": disk.total / (1024**3),
                    "free_gb": disk.free / (1024**3),
                    "used_gb": disk.used / (1024**3),
                    "percent": (disk.used / disk.total) * 100
                },
                "network": {
                    "bytes_sent": net_io.bytes_sent if net_io else 0,
                    "bytes_recv": net_io.bytes_recv if net_io else 0
                },
                "processes": process_count,
                "baseline": self.baseline
            }
        except Exception as e:
            return {"error": str(e)}
    
    def check_resource_limits(self) -> Dict[str, Any]:
        """Check if system resources are within acceptable limits"""
        issues = []
        
        try:
            # Check memory usage
            memory = psutil.virtual_memory()
            if memory.percent > self.config.memory_usage_warning_threshold:
                issues.append(f"High memory usage: {memory.percent:.1f}%")
            
            # Check CPU usage
            cpu_percent = psutil.cpu_percent(interval=1)
            if cpu_percent > self.config.cpu_usage_warning_threshold:
                issues.append(f"High CPU usage: {cpu_percent:.1f}%")
            
            # Check disk usage
            disk = psutil.disk_usage(self.config.test_dir)
            disk_percent = (disk.used / disk.total) * 100
            if disk_percent > self.config.disk_usage_warning_threshold:
                issues.append(f"High disk usage: {disk_percent:.1f}%")
            
            # Check available memory
            if memory.available < self.config.min_available_memory_mb * 1024 * 1024:
                issues.append(f"Low available memory: {memory.available / (1024**3):.1f}GB")
            
            # Check disk space
            if disk.free < self.config.min_available_disk_gb * 1024**3:
                issues.append(f"Low disk space: {disk.free / (1024**3):.1f}GB")
            
            return {
                "status": "WARNING" if issues else "OK",
                "issues": issues,
                "memory_percent": memory.percent,
                "cpu_percent": cpu_percent,
                "disk_percent": disk_percent,
                "memory_available_gb": memory.available / (1024**3),
                "disk_free_gb": disk.free / (1024**3)
            }
            
        except Exception as e:
            return {
                "status": "ERROR",
                "issues": [f"Error checking resources: {str(e)}"]
            }


class ResourceManager:
    """Manage resource cleanup and cleanup verification"""
    
    def __init__(self):
        self.tracked_resources = []
        self.cleanup_handlers = []
    
    def track_resource(self, resource, cleanup_func=None):
        """Track a resource for cleanup"""
        self.tracked_resources.append({
            "resource": resource,
            "cleanup_func": cleanup_func,
            "created_time": time.time()
        })
    
    def add_cleanup_handler(self, handler_func):
        """Add a custom cleanup handler"""
        self.cleanup_handlers.append(handler_func)
    
    def cleanup_all(self):
        """Cleanup all tracked resources"""
        cleaned_resources = []
        failed_cleanups = []
        
        # Cleanup tracked resources
        for tracked in self.tracked_resources:
            try:
                if tracked["cleanup_func"]:
                    tracked["cleanup_func"](tracked["resource"])
                cleaned_resources.append(tracked)
            except Exception as e:
                failed_cleanups.append((tracked, str(e)))
        
        # Run cleanup handlers
        for handler in self.cleanup_handlers:
            try:
                handler()
            except Exception as e:
                failed_cleanups.append((handler, str(e)))
        
        self.tracked_resources = []
        self.cleanup_handlers = []
        
        return {
            "cleaned_resources": len(cleaned_resources),
            "failed_cleanups": len(failed_cleanups),
            "failures": failed_cleanups
        }
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.cleanup_all()


def get_system_limits() -> Dict[str, int]:
    """Get system resource limits"""
    limits = {}
    
    try:
        # Get process limits
        import resource
        limits["max_file_descriptors"] = resource.getrlimit(resource.RLIMIT_NOFILE)[0]
        limits["max_processes"] = resource.getrlimit(resource.RLIMIT_NPROC)[0]
        limits["max_memory_bytes"] = resource.getrlimit(resource.RLIMIT_AS)[0]
    except Exception:
        pass
    
    try:
        # Get file descriptor limits
        soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
        limits["fd_soft_limit"] = soft
        limits["fd_hard_limit"] = hard
    except Exception:
        pass
    
    return limits


def check_ulimits() -> Dict[str, Any]:
    """Check user resource limits"""
    try:
        import subprocess
        result = subprocess.run(['ulimit', '-a'], capture_output=True, text=True)
        return {
            "status": "SUCCESS" if result.returncode == 0 else "ERROR",
            "output": result.stdout if result.returncode == 0 else result.stderr
        }
    except Exception as e:
        return {"status": "ERROR", "error": str(e)}


def validate_test_environment(config: StressTestConfig) -> Dict[str, Any]:
    """Validate that the test environment is suitable for stress testing"""
    issues = []
    warnings = []
    
    # Check test directory
    try:
        test_dir = Path(config.test_dir)
        if not test_dir.exists():
            test_dir.mkdir(parents=True, exist_ok=True)
        else:
            # Check write permissions
            test_file = test_dir / ".stress_test_write_check"
            with open(test_file, 'w') as f:
                f.write("test")
            test_file.unlink()
    except Exception as e:
        issues.append(f"Cannot write to test directory {config.test_dir}: {str(e)}")
    
    # Check available memory
    memory = psutil.virtual_memory()
    available_mb = memory.available / (1024 * 1024)
    required_mb = config.max_memory_allocation_mb + config.min_available_memory_mb
    
    if available_mb < required_mb:
        issues.append(f"Insufficient memory: {available_mb:.0f}MB available, {required_mb:.0f}MB required")
    
    # Check available disk space
    disk = psutil.disk_usage(config.test_dir)
    available_gb = disk.free / (1024**3)
    required_gb = config.min_available_disk_gb
    
    if available_gb < required_gb:
        issues.append(f"Insufficient disk space: {available_gb:.1f}GB available, {required_gb:.1f}GB required")
    
    # Check CPU count
    cpu_count = multiprocessing.cpu_count()
    if cpu_count < 2:
        warnings.append(f"Low CPU count: {cpu_count} (recommended: 2+)")
    
    # Check for running stress tests
    try:
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            if 'stress' in proc.info['name'].lower():
                warnings.append(f"Another stress test may be running (PID: {proc.info['pid']})")
                break
    except Exception:
        pass
    
    return {
        "valid": len(issues) == 0,
        "issues": issues,
        "warnings": warnings,
        "system_info": {
            "memory_available_gb": available_mb / 1024,
            "disk_available_gb": available_gb,
            "cpu_count": cpu_count
        }
    }