#!/usr/bin/env python3
"""
Remote Testing System for Physical Hardware
Provides remote execution, monitoring, and data collection for real hardware
"""

import os
import sys
import json
import time
import socket
import threading
import subprocess
import paramiko
import asyncio
import websockets
import logging
from pathlib import Path
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, asdict
from datetime import datetime
import psutil
import hashlib
import base64
import tempfile
import shutil

@dataclass
class RemoteTestConfig:
    """Configuration for remote testing"""
    hostname: str
    username: str
    key_file: Optional[str] = None
    password: Optional[str] = None
    port: int = 22
    timeout: int = 30
    test_directory: str = "/tmp/hardware_tests"
    results_directory: str = "/workspace/testing/hardware_tests/results"
    monitoring_interval: int = 5
    transfer_timeout: int = 300

@dataclass
class HardwareMetrics:
    """Hardware performance metrics"""
    timestamp: float
    cpu_usage: float
    memory_usage: float
    disk_usage: float
    network_io: Dict[str, int]
    temperature: List[Dict[str, Any]]
    power_consumption: Optional[float] = None
    fan_speeds: List[Dict[str, Any]] = None

class RemoteHardwareTester:
    """Main class for remote hardware testing"""
    
    def __init__(self, config: RemoteTestConfig):
        self.config = config
        self.ssh_client = None
        self.sftp_client = None
        self.logger = self._setup_logging()
        self.test_results = []
        self.metrics_history = []
        self.is_monitoring = False
        self.monitoring_thread = None
        
    def _setup_logging(self):
        """Setup logging for remote testing"""
        log_path = Path("/workspace/testing/hardware_tests/remote_testing.log")
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler(log_path),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def connect(self) -> bool:
        """Establish SSH connection to remote hardware"""
        try:
            self.ssh_client = paramiko.SSHClient()
            self.ssh_client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
            
            # Setup authentication
            if self.config.key_file:
                private_key = paramiko.RSAKey.from_private_key_file(self.config.key_file)
                self.ssh_client.connect(
                    hostname=self.config.hostname,
                    username=self.config.username,
                    pkey=private_key,
                    port=self.config.port,
                    timeout=self.config.timeout
                )
            else:
                self.ssh_client.connect(
                    hostname=self.config.hostname,
                    username=self.config.username,
                    password=self.config.password,
                    port=self.config.port,
                    timeout=self.config.timeout
                )
            
            self.sftp_client = self.ssh_client.open_sftp()
            self.logger.info(f"Successfully connected to {self.config.hostname}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to connect to {self.config.hostname}: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from remote hardware"""
        if self.sftp_client:
            self.sftp_client.close()
        if self.ssh_client:
            self.ssh_client.close()
        self.logger.info("Disconnected from remote hardware")
    
    def execute_remote_command(self, command: str, timeout: int = 300) -> Dict[str, Any]:
        """Execute command on remote hardware"""
        if not self.ssh_client:
            raise RuntimeError("Not connected to remote hardware")
        
        try:
            self.logger.debug(f"Executing remote command: {command}")
            
            stdin, stdout, stderr = self.ssh_client.exec_command(command, timeout=timeout)
            
            exit_status = stdout.channel.recv_exit_status()
            stdout_data = stdout.read().decode('utf-8')
            stderr_data = stderr.read().decode('utf-8')
            
            result = {
                'command': command,
                'exit_status': exit_status,
                'stdout': stdout_data,
                'stderr': stderr_data,
                'timestamp': time.time()
            }
            
            self.logger.debug(f"Command executed with exit status: {exit_status}")
            return result
            
        except Exception as e:
            self.logger.error(f"Error executing remote command: {e}")
            return {
                'command': command,
                'error': str(e),
                'timestamp': time.time()
            }
    
    def upload_file(self, local_path: str, remote_path: str) -> bool:
        """Upload file to remote hardware"""
        if not self.sftp_client:
            raise RuntimeError("Not connected to remote hardware")
        
        try:
            self.logger.debug(f"Uploading {local_path} to {remote_path}")
            
            # Create remote directory if needed
            remote_dir = os.path.dirname(remote_path)
            self.execute_remote_command(f"mkdir -p {remote_dir}")
            
            self.sftp_client.put(local_path, remote_path)
            self.logger.info(f"Successfully uploaded {local_path} to {remote_path}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to upload file: {e}")
            return False
    
    def download_file(self, remote_path: str, local_path: str) -> bool:
        """Download file from remote hardware"""
        if not self.sftp_client:
            raise RuntimeError("Not connected to remote hardware")
        
        try:
            self.logger.debug(f"Downloading {remote_path} to {local_path}")
            
            # Create local directory if needed
            os.makedirs(os.path.dirname(local_path), exist_ok=True)
            
            self.sftp_client.get(remote_path, local_path)
            self.logger.info(f"Successfully downloaded {remote_path} to {local_path}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to download file: {e}")
            return False
    
    def run_remote_script(self, script_content: str, script_name: str = None) -> str:
        """Run script content on remote hardware"""
        if not script_name:
            script_name = f"remote_script_{int(time.time())}.sh"
        
        remote_script_path = f"{self.config.test_directory}/{script_name}"
        
        try:
            # Create remote directory
            self.execute_remote_command(f"mkdir -p {self.config.test_directory}")
            
            # Write script to remote
            script_command = f"cat > {remote_script_path} << 'EOF'\n{script_content}\nEOF"
            result = self.execute_remote_command(script_command)
            
            if result['exit_status'] != 0:
                raise RuntimeError(f"Failed to write script: {result['stderr']}")
            
            # Make script executable
            self.execute_remote_command(f"chmod +x {remote_script_path}")
            
            # Execute script
            result = self.execute_remote_command(f"{remote_script_path}")
            
            return result['stdout']
            
        except Exception as e:
            self.logger.error(f"Error running remote script: {e}")
            raise
    
    def get_hardware_metrics(self) -> HardwareMetrics:
        """Get current hardware metrics from remote system"""
        try:
            # Get system metrics
            metrics_script = '''
import psutil
import time
import json

data = {
    'timestamp': time.time(),
    'cpu_percent': psutil.cpu_percent(interval=1),
    'memory_percent': psutil.virtual_memory().percent,
    'disk_usage': psutil.disk_usage('/').percent,
    'network_io': {k: v._asdict() for k, v in psutil.net_io_counters(pernic=True).items()},
    'boot_time': psutil.boot_time()
}

try:
    import sensors
    sensors.init()
    data['temperature'] = []
    data['fan_speeds'] = []
    for chip in sensors.ChipIterator():
        for feature in sensors.FeatureIterator(chip):
            feature_name = sensors.FeatureName(feature)
            for subfeature in sensors.SubFeatureIterator(feature):
                subfeature_name = sensors.SubFeatureName(subfeature)
                value = sensors.get_value(chip, subfeature.number)
                
                feature_label = str(feature_name)
                if 'temp' in feature_label.lower():
                    data['temperature'].append({
                        'name': feature_label,
                        'value': value,
                        'unit': '°C'
                    })
                elif 'fan' in feature_label.lower():
                    data['fan_speeds'].append({
                        'name': feature_label,
                        'value': value,
                        'unit': 'RPM'
                    })
except ImportError:
    data['temperature'] = []
    data['fan_speeds'] = []

print(json.dumps(data))
'''
            
            result = self.execute_remote_command("python3 -c '" + metrics_script.replace("'", "'\\''") + "'")
            
            if result['exit_status'] == 0:
                metrics_data = json.loads(result['stdout'])
                return HardwareMetrics(
                    timestamp=metrics_data['timestamp'],
                    cpu_usage=metrics_data['cpu_percent'],
                    memory_usage=metrics_data['memory_percent'],
                    disk_usage=metrics_data['disk_usage'],
                    network_io=metrics_data['network_io'],
                    temperature=metrics_data.get('temperature', []),
                    fan_speeds=metrics_data.get('fan_speeds', [])
                )
            else:
                raise RuntimeError(f"Failed to get metrics: {result['stderr']}")
                
        except Exception as e:
            self.logger.error(f"Error getting hardware metrics: {e}")
            return HardwareMetrics(
                timestamp=time.time(),
                cpu_usage=0,
                memory_usage=0,
                disk_usage=0,
                network_io={},
                temperature=[]
            )
    
    def start_monitoring(self):
        """Start continuous hardware monitoring"""
        if self.is_monitoring:
            return
        
        self.is_monitoring = True
        self.monitoring_thread = threading.Thread(target=self._monitoring_loop)
        self.monitoring_thread.daemon = True
        self.monitoring_thread.start()
        self.logger.info("Started hardware monitoring")
    
    def stop_monitoring(self):
        """Stop hardware monitoring"""
        self.is_monitoring = False
        if self.monitoring_thread:
            self.monitoring_thread.join(timeout=10)
        self.logger.info("Stopped hardware monitoring")
    
    def _monitoring_loop(self):
        """Main monitoring loop"""
        while self.is_monitoring:
            try:
                metrics = self.get_hardware_metrics()
                self.metrics_history.append(metrics)
                
                # Keep only last 1000 metrics to prevent memory issues
                if len(self.metrics_history) > 1000:
                    self.metrics_history = self.metrics_history[-1000:]
                
                # Check for critical conditions
                if metrics.cpu_usage > 95:
                    self.logger.warning(f"High CPU usage: {metrics.cpu_usage}%")
                
                if metrics.memory_usage > 95:
                    self.logger.warning(f"High memory usage: {metrics.memory_usage}%")
                
                for temp_sensor in metrics.temperature:
                    if temp_sensor.get('value', 0) > 80:
                        self.logger.warning(f"High temperature {temp_sensor['name']}: {temp_sensor['value']}°C")
                
                time.sleep(self.config.monitoring_interval)
                
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                time.sleep(self.config.monitoring_interval)
    
    def run_stress_test(self, duration_minutes: int = 30) -> Dict[str, Any]:
        """Run stress test on remote hardware"""
        self.logger.info(f"Starting stress test for {duration_minutes} minutes")
        
        stress_script = f'''
import subprocess
import threading
import time
import sys
import psutil

def cpu_stress(duration):
    """CPU stress function"""
    end_time = time.time() + duration
    while time.time() < end_time:
        # CPU-intensive work
        sum(range(100000))

def memory_stress(duration):
    """Memory stress function"""
    end_time = time.time() + duration
    data = []
    try:
        while time.time() < end_time:
            # Allocate memory
            data.append([0] * 1000)
            time.sleep(0.1)
    except MemoryError:
        pass

def disk_stress(duration):
    """Disk stress function"""
    end_time = time.time() + duration
    try:
        with open('/tmp/stress_test.dat', 'w') as f:
            while time.time() < end_time:
                f.write('x' * 1024)
    except:
        pass
    finally:
        try:
            os.remove('/tmp/stress_test.dat')
        except:
            pass

# Start stress threads
threads = []
threads.append(threading.Thread(target=cpu_stress, args=(duration*60,)))
threads.append(threading.Thread(target=memory_stress, args=(duration*60,)))
threads.append(threading.Thread(target=disk_stress, args=(duration*60,)))

for thread in threads:
    thread.start()

for thread in threads:
    thread.join()

print("Stress test completed")
'''
        
        try:
            result = self.run_remote_script(stress_script)
            
            # Collect test results
            test_results = {
                'test_type': 'stress_test',
                'duration_minutes': duration_minutes,
                'status': 'completed',
                'timestamp': time.time(),
                'metrics_collected': len(self.metrics_history)
            }
            
            return test_results
            
        except Exception as e:
            self.logger.error(f"Stress test failed: {e}")
            return {
                'test_type': 'stress_test',
                'status': 'failed',
                'error': str(e),
                'timestamp': time.time()
            }
    
    def run_performance_benchmark(self) -> Dict[str, Any]:
        """Run performance benchmarks on remote hardware"""
        self.logger.info("Starting performance benchmark")
        
        benchmark_script = '''
import subprocess
import time
import json
import os

def run_command(cmd):
    """Run command and capture timing"""
    start_time = time.time()
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60)
        end_time = time.time()
        return {
            'command': cmd,
            'exit_code': result.returncode,
            'duration': end_time - start_time,
            'stdout': result.stdout,
            'stderr': result.stderr
        }
    except subprocess.TimeoutExpired:
        return {
            'command': cmd,
            'exit_code': -1,
            'duration': 60,
            'stdout': '',
            'stderr': 'Timeout'
        }

# CPU benchmark
print("Running CPU benchmark...")
cpu_benchmark = run_command("dd if=/dev/zero bs=1M count=1000 2>/dev/null | dd of=/dev/null bs=1M 2>/dev/null")

# Memory benchmark
print("Running memory benchmark...")
memory_benchmark = run_command("dd if=/dev/zero of=/dev/null bs=1M count=1000 2>/dev/null")

# Disk write benchmark
print("Running disk write benchmark...")
disk_write_benchmark = run_command("dd if=/dev/zero of=/tmp/benchmark.tmp bs=1M count=100 2>/dev/null")

# Disk read benchmark
print("Running disk read benchmark...")
disk_read_benchmark = run_command("dd if=/tmp/benchmark.tmp of=/dev/null bs=1M 2>/dev/null")

# Cleanup
try:
    os.remove('/tmp/benchmark.tmp')
except:
    pass

results = {
    'cpu_benchmark': cpu_benchmark,
    'memory_benchmark': memory_benchmark,
    'disk_write_benchmark': disk_write_benchmark,
    'disk_read_benchmark': disk_read_benchmark,
    'timestamp': time.time()
}

print(json.dumps(results))
'''
        
        try:
            result = self.run_remote_script(benchmark_script)
            benchmark_data = json.loads(result)
            
            # Process results
            processed_results = {
                'test_type': 'performance_benchmark',
                'status': 'completed',
                'timestamp': time.time(),
                'results': benchmark_data
            }
            
            return processed_results
            
        except Exception as e:
            self.logger.error(f"Performance benchmark failed: {e}")
            return {
                'test_type': 'performance_benchmark',
                'status': 'failed',
                'error': str(e),
                'timestamp': time.time()
            }
    
    def transfer_results(self) -> bool:
        """Transfer test results to local machine"""
        try:
            remote_results_dir = f"{self.config.test_directory}/results"
            
            # Create local results directory
            os.makedirs(self.config.results_directory, exist_ok=True)
            
            # List remote results
            result = self.execute_remote_command(f"ls -la {remote_results_dir} 2>/dev/null || echo 'No results found'")
            
            if 'No results found' in result['stdout']:
                self.logger.warning("No remote results found")
                return False
            
            # Get file list
            files_result = self.execute_remote_command(f"find {remote_results_dir} -type f -name '*.json' -o -name '*.log'")
            
            if files_result['exit_status'] == 0:
                remote_files = [f.strip() for f in files_result['stdout'].split('\n') if f.strip()]
                
                for remote_file in remote_files:
                    local_file = os.path.join(
                        self.config.results_directory,
                        os.path.basename(remote_file)
                    )
                    self.download_file(remote_file, local_file)
            
            self.logger.info("Test results transferred successfully")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to transfer results: {e}")
            return False
    
    def generate_test_report(self) -> str:
        """Generate comprehensive test report"""
        report_data = {
            'test_session': {
                'hostname': self.config.hostname,
                'start_time': self.test_results[0]['timestamp'] if self.test_results else time.time(),
                'end_time': time.time(),
                'tests_run': len(self.test_results)
            },
            'test_results': [asdict(result) if hasattr(result, '__dict__') else result 
                           for result in self.test_results],
            'metrics_history': [asdict(metric) if hasattr(metric, '__dict__') else metric 
                              for metric in self.metrics_history]
        }
        
        # Calculate summary statistics
        if self.metrics_history:
            cpu_values = [m.cpu_usage for m in self.metrics_history]
            memory_values = [m.memory_usage for m in self.metrics_history]
            
            report_data['summary'] = {
                'avg_cpu_usage': sum(cpu_values) / len(cpu_values),
                'max_cpu_usage': max(cpu_values),
                'avg_memory_usage': sum(memory_values) / len(memory_values),
                'max_memory_usage': max(memory_values),
                'metrics_collected': len(self.metrics_history)
            }
        
        report_path = os.path.join(
            self.config.results_directory,
            f"remote_test_report_{int(time.time())}.json"
        )
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Test report generated: {report_path}")
        return report_path
    
    def cleanup(self):
        """Cleanup remote test directory"""
        try:
            self.execute_remote_command(f"rm -rf {self.config.test_directory}")
            self.logger.info("Remote cleanup completed")
        except Exception as e:
            self.logger.error(f"Cleanup failed: {e}")


class RemoteTestOrchestrator:
    """Orchestrates remote testing across multiple hardware devices"""
    
    def __init__(self):
        self.logger = self._setup_logging()
        self.testers = {}
        
    def _setup_logging(self):
        """Setup logging for orchestrator"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s'
        )
        return logging.getLogger(__name__)
    
    def add_hardware_device(self, device_id: str, config: RemoteTestConfig):
        """Add hardware device for testing"""
        self.testers[device_id] = RemoteHardwareTester(config)
        self.logger.info(f"Added hardware device: {device_id}")
    
    def run_parallel_tests(self, test_configs: List[Dict[str, Any]]) -> Dict[str, str]:
        """Run tests in parallel on multiple devices"""
        results = {}
        threads = []
        
        def run_test_on_device(device_id: str, test_config: Dict[str, Any]):
            tester = self.testers.get(device_id)
            if not tester:
                results[device_id] = {'error': 'Device not found'}
                return
            
            try:
                if not tester.connect():
                    results[device_id] = {'error': 'Failed to connect'}
                    return
                
                # Run specified tests
                device_results = {}
                for test_name in test_config.get('tests', []):
                    if test_name == 'stress_test':
                        duration = test_config.get('duration_minutes', 30)
                        result = tester.run_stress_test(duration)
                        device_results['stress_test'] = result
                        
                    elif test_name == 'benchmark':
                        result = tester.run_performance_benchmark()
                        device_results['benchmark'] = result
                
                results[device_id] = device_results
                
                # Transfer results
                tester.transfer_results()
                
            except Exception as e:
                results[device_id] = {'error': str(e)}
                
            finally:
                tester.disconnect()
                tester.cleanup()
        
        # Start test threads
        for device_id, test_config in test_configs.items():
            thread = threading.Thread(target=run_test_on_device, args=(device_id, test_config))
            thread.start()
            threads.append(thread)
        
        # Wait for all tests to complete
        for thread in threads:
            thread.join()
        
        self.logger.info(f"Completed parallel tests on {len(results)} devices")
        return results


def main():
    """Main function for standalone execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Remote Hardware Testing')
    parser.add_argument('--hostname', required=True, help='Remote hostname/IP')
    parser.add_argument('--username', required=True, help='Remote username')
    parser.add_argument('--key-file', help='SSH key file path')
    parser.add_argument('--password', help='SSH password')
    parser.add_argument('--port', type=int, default=22, help='SSH port')
    parser.add_argument('--test', choices=['stress', 'benchmark', 'both'], 
                       default='both', help='Test type to run')
    parser.add_argument('--duration', type=int, default=30, 
                       help='Test duration in minutes')
    
    args = parser.parse_args()
    
    config = RemoteTestConfig(
        hostname=args.hostname,
        username=args.username,
        key_file=args.key_file,
        password=args.password,
        port=args.port
    )
    
    tester = RemoteHardwareTester(config)
    
    if tester.connect():
        try:
            if args.test in ['stress', 'both']:
                tester.start_monitoring()
                result = tester.run_stress_test(args.duration)
                tester.stop_monitoring()
                print(f"Stress test result: {result}")
            
            if args.test in ['benchmark', 'both']:
                result = tester.run_performance_benchmark()
                print(f"Benchmark result: {result}")
            
            tester.transfer_results()
            tester.generate_test_report()
            
        finally:
            tester.disconnect()
    else:
        print("Failed to connect to remote hardware")


if __name__ == "__main__":
    main()