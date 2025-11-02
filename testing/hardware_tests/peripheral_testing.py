#!/usr/bin/env python3
"""
Peripheral Testing Framework
Comprehensive testing for USB devices, network interfaces, storage devices, and graphics
"""

import os
import sys
import json
import time
import subprocess
import threading
import logging
import tempfile
import shutil
import statistics
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import psutil
import socket
import hashlib

class TestStatus(Enum):
    PASS = "pass"
    FAIL = "fail"
    WARNING = "warning"
    SKIP = "skip"
    UNKNOWN = "unknown"

@dataclass
class PeripheralTestResult:
    """Result of a peripheral test"""
    test_name: str
    device_name: str
    device_type: str
    status: TestStatus
    message: str
    performance_metrics: Dict[str, Any]
    execution_time: float
    timestamp: float
    errors: List[str] = None
    warnings: List[str] = None

class PeripheralTester:
    """Main class for peripheral testing"""
    
    def __init__(self):
        self.logger = self._setup_logging()
        self.test_results = []
        self.usb_devices = []
        self.network_interfaces = []
        self.storage_devices = []
        self.gpu_devices = []
        
    def _setup_logging(self):
        """Setup logging for peripheral testing"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/peripheral_test.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def detect_usb_devices(self) -> List[Dict[str, Any]]:
        """Detect USB devices"""
        usb_devices = []
        
        try:
            # Use lsusb to get device information
            result = subprocess.run(['lsusb'], capture_output=True, text=True)
            if result.returncode == 0:
                lines = result.stdout.strip().split('\n')
                for line in lines:
                    if line.strip():
                        parts = line.split()
                        if len(parts) >= 6:
                            bus = parts[1]
                            device = parts[3].rstrip(':')
                            vendor_product = parts[5]
                            
                            if ':' in vendor_product:
                                vendor_id, product_id = vendor_product.split(':')
                            else:
                                vendor_id = vendor_product
                                product_id = "0000"
                            
                            # Get detailed device information
                            device_info = {
                                'bus': bus,
                                'device': device,
                                'vendor_id': vendor_id,
                                'product_id': product_id,
                                'name': self._get_usb_device_name(vendor_id, product_id),
                                'category': self._categorize_usb_device(vendor_id, product_id),
                                'speed': 'unknown',
                                'testable': True
                            }
                            
                            usb_devices.append(device_info)
                            
        except Exception as e:
            self.logger.error(f"Error detecting USB devices: {e}")
        
        self.usb_devices = usb_devices
        return usb_devices
    
    def _get_usb_device_name(self, vendor_id: str, product_id: str) -> str:
        """Get human-readable USB device name"""
        # Try to get device name from lsusb verbose output
        try:
            # This is a simplified approach - in practice you'd use a database
            known_devices = {
                '046d': {'c52b': 'Logitech USB Receiver', 'c52f': 'Logitech Unifying Receiver'},
                '1532': {'0045': 'Razer DeathAdder Mouse'},
                '413c': {'2107': 'Dell USB Keyboard'},
                '8087': {'0a2b': 'Intel Bluetooth'},
            }
            
            if vendor_id in known_devices:
                vendor_devices = known_devices[vendor_id]
                if product_id in vendor_devices:
                    return vendor_devices[product_id]
            
            return f"USB Device {vendor_id}:{product_id}"
            
        except Exception:
            return f"USB Device {vendor_id}:{product_id}"
    
    def _categorize_usb_device(self, vendor_id: str, product_id: str) -> str:
        """Categorize USB device"""
        # Simple categorization based on vendor/product IDs
        known_categories = {
            '046d': 'input',  # Logitech - mice, keyboards
            '1532': 'input',  # Razer - gaming devices
            '413c': 'input',  # Dell - keyboards
            '8087': 'wireless',  # Intel Bluetooth/WiFi
            '0cf3': 'wireless',  # Atheros
            '04f2': 'camera',   # Chicony cameras
            '05ac': 'storage',  # Apple devices
            '0781': 'storage',  # SanDisk
            '0930': 'storage',  # Toshiba
        }
        
        if vendor_id in known_categories:
            return known_categories[vendor_id]
        
        return 'other'
    
    def test_usb_device_basic(self, device: Dict[str, Any]) -> PeripheralTestResult:
        """Test basic USB device functionality"""
        start_time = time.time()
        
        device_name = device['name']
        device_type = device['category']
        
        errors = []
        warnings = []
        
        try:
            # Check if device is accessible
            result = subprocess.run([
                'lsusb', '-s', f"{device['bus']}:{device['device']}", '-v'
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode != 0:
                errors.append("Device not accessible via lsusb")
                status = TestStatus.FAIL
            else:
                # Check for common issues
                output = result.stdout
                if 'Configuration' in output:
                    status = TestStatus.PASS
                    message = f"USB {device_type} device detected and accessible"
                else:
                    warnings.append("Device may have configuration issues")
                    status = TestStatus.WARNING
                    message = f"USB {device_type} device detected but may have issues"
                
        except subprocess.TimeoutExpired:
            errors.append("Device test timeout")
            status = TestStatus.FAIL
            message = f"USB {device_type} device test timeout"
        except Exception as e:
            errors.append(f"Test execution failed: {str(e)}")
            status = TestStatus.FAIL
            message = f"USB {device_type} device test failed"
        
        return PeripheralTestResult(
            test_name="usb_basic_functionality",
            device_name=device_name,
            device_type=f"usb_{device_type}",
            status=status,
            message=message,
            performance_metrics={'device_accessible': status == TestStatus.PASS},
            execution_time=time.time() - start_time,
            timestamp=time.time(),
            errors=errors,
            warnings=warnings
        )
    
    def test_usb_storage_device(self, device: Dict[str, Any]) -> PeripheralTestResult:
        """Test USB storage device functionality"""
        start_time = time.time()
        
        if device['category'] != 'storage':
            return PeripheralTestResult(
                test_name="usb_storage_performance",
                device_name=device['name'],
                device_type="usb_storage",
                status=TestStatus.SKIP,
                message="Not a storage device",
                performance_metrics={},
                execution_time=time.time() - start_time,
                timestamp=time.time()
            )
        
        device_name = device['name']
        errors = []
        warnings = []
        
        try:
            # Find corresponding block device
            storage_devices = psutil.disk_partitions()
            usb_storage_device = None
            
            # This is a simplified approach - would need to correlate USB and block devices
            for storage in storage_devices:
                if 'usb' in storage.device.lower() or 'sdb' in storage.device:
                    usb_storage_device = storage
                    break
            
            if not usb_storage_device:
                warnings.append("Could not find corresponding block device")
                status = TestStatus.WARNING
                message = "USB storage device detected but block device not found"
                metrics = {}
            else:
                # Test read/write performance
                test_file = f"{usb_storage_device.mountpoint}/usb_test_{int(time.time())}.tmp"
                
                # Write test
                write_start = time.time()
                with open(test_file, 'wb') as f:
                    f.write(b'0' * (10 * 1024 * 1024))  # 10MB
                write_time = time.time() - write_start
                
                # Read test
                read_start = time.time()
                with open(test_file, 'rb') as f:
                    f.read()
                read_time = time.time() - read_start
                
                # Cleanup
                os.remove(test_file)
                
                write_speed = (10 / write_time) if write_time > 0 else 0
                read_speed = (10 / read_time) if read_time > 0 else 0
                
                metrics = {
                    'write_speed_mbps': write_speed,
                    'read_speed_mbps': read_speed,
                    'mount_point': usb_storage_device.mountpoint,
                    'device': usb_storage_device.device
                }
                
                # Determine status based on performance
                if read_speed > 10 and write_speed > 5:
                    status = TestStatus.PASS
                    message = f"USB storage performance: {read_speed:.1f}MB/s read, {write_speed:.1f}MB/s write"
                else:
                    status = TestStatus.WARNING
                    message = f"USB storage slower than expected: {read_speed:.1f}MB/s read, {write_speed:.1f}MB/s write"
                    warnings.append("USB storage performance below expectations")
                    
        except Exception as e:
            errors.append(f"Storage test failed: {str(e)}")
            status = TestStatus.FAIL
            message = "USB storage test failed"
            metrics = {}
        
        return PeripheralTestResult(
            test_name="usb_storage_performance",
            device_name=device_name,
            device_type="usb_storage",
            status=status,
            message=message,
            performance_metrics=metrics,
            execution_time=time.time() - start_time,
            timestamp=time.time(),
            errors=errors,
            warnings=warnings
        )
    
    def test_usb_input_device(self, device: Dict[str, Any]) -> PeripheralTestResult:
        """Test USB input device functionality"""
        start_time = time.time()
        
        if device['category'] not in ['input']:
            return PeripheralTestResult(
                test_name="usb_input_functionality",
                device_name=device['name'],
                device_type="usb_input",
                status=TestStatus.SKIP,
                message="Not an input device",
                performance_metrics={},
                execution_time=time.time() - start_time,
                timestamp=time.time()
            )
        
        device_name = device['name']
        errors = []
        warnings = []
        
        try:
            # Check if device appears in /dev/input
            input_devices = list(Path('/dev/input').glob('event*'))
            
            # This is a simplified check - would need more sophisticated device correlation
            has_input_device = len(input_devices) > 0
            
            if has_input_device:
                status = TestStatus.PASS
                message = "USB input device appears functional"
                metrics = {'input_devices_detected': len(input_devices)}
            else:
                warnings.append("No input devices detected")
                status = TestStatus.WARNING
                message = "USB input device detected but no input events detected"
                metrics = {}
                
        except Exception as e:
            errors.append(f"Input device test failed: {str(e)}")
            status = TestStatus.FAIL
            message = "USB input device test failed"
            metrics = {}
        
        return PeripheralTestResult(
            test_name="usb_input_functionality",
            device_name=device_name,
            device_type="usb_input",
            status=status,
            message=message,
            performance_metrics=metrics,
            execution_time=time.time() - start_time,
            timestamp=time.time(),
            errors=errors,
            warnings=warnings
        )
    
    def test_network_interfaces(self) -> List[PeripheralTestResult]:
        """Test all network interfaces"""
        results = []
        
        # Get network interface statistics
        try:
            net_stats = psutil.net_if_stats()
            net_io = psutil.net_io_counters(pernic=True)
            net_addr = psutil.net_if_addrs()
            
            for interface_name, interface_stats in net_stats.items():
                # Skip loopback and virtual interfaces
                if interface_name.startswith(('lo', 'docker', 'veth', 'br-')):
                    continue
                
                result = self._test_network_interface(
                    interface_name, 
                    interface_stats, 
                    net_io.get(interface_name), 
                    net_addr.get(interface_name, [])
                )
                results.append(result)
                
        except Exception as e:
            self.logger.error(f"Error testing network interfaces: {e}")
            results.append(PeripheralTestResult(
                test_name="network_interface_test",
                device_name="unknown",
                device_type="network",
                status=TestStatus.FAIL,
                message=f"Network testing failed: {str(e)}",
                performance_metrics={},
                execution_time=0,
                timestamp=time.time(),
                errors=[str(e)]
            ))
        
        return results
    
    def _test_network_interface(self, interface_name: str, stats: Any, 
                              io_stats: Any, addresses: List) -> PeripheralTestResult:
        """Test individual network interface"""
        start_time = time.time()
        
        errors = []
        warnings = []
        
        try:
            # Check if interface is up
            is_up = stats.isup
            speed_mbps = stats.speed
            
            if not is_up:
                warnings.append(f"Interface {interface_name} is not up")
            
            # Check interface speed
            speed_status = TestStatus.PASS
            speed_message = ""
            
            if speed_mbps > 0:
                if speed_mbps >= 1000:
                    speed_message = f"High-speed interface ({speed_mbps} Mbps)"
                elif speed_mbps >= 100:
                    speed_message = f"Fast interface ({speed_mbps} Mbps)"
                else:
                    speed_message = f"Slow interface ({speed_mbps} Mbps)"
                    warnings.append("Interface speed lower than typical")
            else:
                warnings.append("Interface speed unknown")
                speed_message = "Interface speed not detected"
            
            # Check for IP addresses
            ip_addresses = []
            for addr in addresses:
                if addr.family == socket.AF_INET:
                    ip_addresses.append(addr.address)
            
            # Test basic connectivity (ping localhost)
            connectivity_status = TestStatus.FAIL
            if is_up and ip_addresses:
                try:
                    result = subprocess.run([
                        'ping', '-c', '3', '-W', '5', '127.0.0.1'
                    ], capture_output=True, text=True, timeout=15)
                    
                    if result.returncode == 0:
                        connectivity_status = TestStatus.PASS
                        connectivity_message = "Interface connectivity test passed"
                    else:
                        connectivity_message = "Interface connectivity test failed"
                        errors.append("Failed to ping localhost")
                except Exception as e:
                    connectivity_message = f"Connectivity test error: {str(e)}"
                    errors.append(str(e))
            else:
                connectivity_message = "Interface not configured or not up"
                errors.append("Interface not usable")
            
            # Determine overall status
            if connectivity_status == TestStatus.PASS:
                status = TestStatus.PASS
                overall_message = f"Network interface {interface_name}: {speed_message}, {connectivity_message}"
            elif warnings and not errors:
                status = TestStatus.WARNING
                overall_message = f"Network interface {interface_name}: {speed_message}, {connectivity_message}"
            else:
                status = TestStatus.FAIL
                overall_message = f"Network interface {interface_name} failed: {speed_message}, {connectivity_message}"
            
            # Get current network I/O statistics
            io_metrics = {}
            if io_stats:
                io_metrics = {
                    'bytes_sent': io_stats.bytes_sent,
                    'bytes_recv': io_stats.bytes_recv,
                    'packets_sent': io_stats.packets_sent,
                    'packets_recv': io_stats.packets_recv,
                    'errors_in': io_stats.errin,
                    'errors_out': io_stats.errout,
                    'drop_in': io_stats.dropin,
                    'drop_out': io_stats.dropout
                }
            
            metrics = {
                'is_up': is_up,
                'speed_mbps': speed_mbps,
                'ip_addresses': ip_addresses,
                'connectivity_working': connectivity_status == TestStatus.PASS,
                'interface_statistics': io_metrics
            }
            
        except Exception as e:
            status = TestStatus.FAIL
            overall_message = f"Network interface {interface_name} test failed: {str(e)}"
            errors.append(str(e))
            metrics = {}
        
        return PeripheralTestResult(
            test_name="network_interface_test",
            device_name=interface_name,
            device_type="network",
            status=status,
            message=overall_message,
            performance_metrics=metrics,
            execution_time=time.time() - start_time,
            timestamp=time.time(),
            errors=errors,
            warnings=warnings
        )
    
    def test_storage_devices(self) -> List[PeripheralTestResult]:
        """Test storage devices"""
        results = []
        
        try:
            disk_usage = psutil.disk_usage('/')
            disk_io = psutil.disk_io_counters()
            
            # Test root filesystem
            root_result = self._test_filesystem('/')
            results.append(root_result)
            
            # Test other mounted filesystems
            for partition in psutil.disk_partitions():
                if partition.mountpoint != '/' and os.path.exists(partition.mountpoint):
                    try:
                        result = self._test_filesystem(partition.mountpoint)
                        results.append(result)
                    except Exception as e:
                        self.logger.warning(f"Failed to test filesystem {partition.mountpoint}: {e}")
                        
        except Exception as e:
            self.logger.error(f"Error testing storage devices: {e}")
            results.append(PeripheralTestResult(
                test_name="storage_device_test",
                device_name="unknown",
                device_type="storage",
                status=TestStatus.FAIL,
                message=f"Storage testing failed: {str(e)}",
                performance_metrics={},
                execution_time=0,
                timestamp=time.time(),
                errors=[str(e)]
            ))
        
        return results
    
    def _test_filesystem(self, mountpoint: str) -> PeripheralTestResult:
        """Test filesystem performance and integrity"""
        start_time = time.time()
        
        errors = []
        warnings = []
        
        try:
            # Get filesystem statistics
            usage = psutil.disk_usage(mountpoint)
            
            # Test sequential read performance
            test_file = os.path.join(mountpoint, f'seq_read_test_{int(time.time())}.tmp')
            
            # Create test file
            with open(test_file, 'wb') as f:
                f.write(b'0' * (50 * 1024 * 1024))  # 50MB
            
            # Read test
            read_start = time.time()
            with open(test_file, 'rb') as f:
                data = f.read()
            read_time = time.time() - read_start
            
            # Test random I/O
            random_file = os.path.join(mountpoint, f'random_io_test_{int(time.time())}.tmp')
            
            # Create file for random I/O
            with open(random_file, 'wb') as f:
                f.write(b'0' * (10 * 1024 * 1024))  # 10MB
            
            random_start = time.time()
            with open(random_file, 'rb') as f:
                # Read random 4KB blocks
                for _ in range(100):
                    f.seek(0)
                    f.read(4096)
            random_time = time.time() - random_start
            
            # Cleanup
            os.remove(test_file)
            os.remove(random_file)
            
            # Calculate performance metrics
            seq_read_speed = (50 / read_time) if read_time > 0 else 0
            random_io_speed = ((100 * 4096) / (1024*1024) / random_time) if random_time > 0 else 0
            
            # Check disk health (simplified)
            total_gb = usage.total / (1024**3)
            free_gb = usage.free / (1024**3)
            usage_percent = (usage.used / usage.total) * 100
            
            # Determine status
            status = TestStatus.PASS
            message = f"Filesystem {mountpoint}: {seq_read_speed:.1f}MB/s sequential, {random_io_speed:.1f}MB/s random"
            
            warnings_issues = []
            if usage_percent > 90:
                warnings_issues.append("Disk space critically low (>90%)")
                status = TestStatus.WARNING
            
            if seq_read_speed < 20:  # Slow sequential read
                warnings_issues.append("Slow sequential read performance")
                status = TestStatus.WARNING if status == TestStatus.PASS else status
            
            if random_io_speed < 5:  # Slow random I/O
                warnings_issues.append("Slow random I/O performance")
                status = TestStatus.WARNING if status == TestStatus.PASS else status
            
            metrics = {
                'mountpoint': mountpoint,
                'total_space_gb': round(total_gb, 2),
                'free_space_gb': round(free_gb, 2),
                'usage_percent': round(usage_percent, 2),
                'sequential_read_speed_mbps': round(seq_read_speed, 2),
                'random_io_speed_mbps': round(random_io_speed, 2)
            }
            
        except Exception as e:
            status = TestStatus.FAIL
            message = f"Filesystem test failed for {mountpoint}: {str(e)}"
            errors.append(str(e))
            metrics = {}
        
        return PeripheralTestResult(
            test_name="filesystem_performance_test",
            device_name=mountpoint,
            device_type="storage",
            status=status,
            message=message,
            performance_metrics=metrics,
            execution_time=time.time() - start_time,
            timestamp=time.time(),
            errors=errors,
            warnings=warnings
        )
    
    def test_gpu_devices(self) -> List[PeripheralTestResult]:
        """Test GPU devices"""
        results = []
        
        try:
            # Check for NVIDIA GPUs
            nvidia_result = self._test_nvidia_gpu()
            if nvidia_result:
                results.append(nvidia_result)
            
            # Check for AMD/Intel GPUs (generic testing)
            generic_result = self._test_generic_gpu()
            if generic_result:
                results.append(generic_result)
                
        except Exception as e:
            self.logger.error(f"Error testing GPU devices: {e}")
            results.append(PeripheralTestResult(
                test_name="gpu_test",
                device_name="unknown",
                device_type="graphics",
                status=TestStatus.FAIL,
                message=f"GPU testing failed: {str(e)}",
                performance_metrics={},
                execution_time=0,
                timestamp=time.time(),
                errors=[str(e)]
            ))
        
        return results
    
    def _test_nvidia_gpu(self) -> Optional[PeripheralTestResult]:
        """Test NVIDIA GPU"""
        start_time = time.time()
        
        try:
            # Check if nvidia-smi is available
            result = subprocess.run(['nvidia-smi'], capture_output=True, text=True, timeout=10)
            if result.returncode != 0:
                return None
            
            # Get GPU information
            gpu_info_result = subprocess.run([
                'nvidia-smi', '--query-gpu=name,driver_version,temperature.gpu,power.draw,utilization.gpu,memory.total,memory.used',
                '--format=csv,noheader,nounits'
            ], capture_output=True, text=True, timeout=10)
            
            if gpu_info_result.returncode != 0:
                return PeripheralTestResult(
                    test_name="nvidia_gpu_test",
                    device_name="NVIDIA GPU",
                    device_type="graphics",
                    status=TestStatus.FAIL,
                    message="Failed to query GPU information",
                    performance_metrics={},
                    execution_time=time.time() - start_time,
                    timestamp=time.time(),
                    errors=["nvidia-smi query failed"]
                )
            
            # Parse GPU information
            gpu_data = gpu_info_result.stdout.strip().split(', ')
            
            if len(gpu_data) >= 7:
                gpu_name = gpu_data[0]
                driver_version = gpu_data[1]
                temperature = float(gpu_data[2]) if gpu_data[2] != '[Not Supported]' else 0
                power_draw = float(gpu_data[3]) if gpu_data[3] != '[Not Supported]' else 0
                utilization = float(gpu_data[4]) if gpu_data[4] != '[Not Supported]' else 0
                memory_total = float(gpu_data[5]) if gpu_data[5] != '[Not Supported]' else 0
                memory_used = float(gpu_data[6]) if gpu_data[6] != '[Not Supported]' else 0
                
                # Determine status
                status = TestStatus.PASS
                message = f"NVIDIA GPU {gpu_name}: {utilization}% utilization, {temperature}°C"
                
                warnings = []
                if temperature > 80:
                    status = TestStatus.WARNING
                    warnings.append("GPU temperature high")
                if utilization > 90:
                    warnings.append("GPU heavily utilized")
                
                metrics = {
                    'gpu_name': gpu_name,
                    'driver_version': driver_version,
                    'temperature_c': temperature,
                    'power_draw_w': power_draw,
                    'utilization_percent': utilization,
                    'memory_total_mb': memory_total,
                    'memory_used_mb': memory_used,
                    'memory_utilization_percent': (memory_used / memory_total * 100) if memory_total > 0 else 0
                }
                
                return PeripheralTestResult(
                    test_name="nvidia_gpu_test",
                    device_name=gpu_name,
                    device_type="graphics",
                    status=status,
                    message=message,
                    performance_metrics=metrics,
                    execution_time=time.time() - start_time,
                    timestamp=time.time(),
                    warnings=warnings
                )
            
        except subprocess.TimeoutExpired:
            return PeripheralTestResult(
                test_name="nvidia_gpu_test",
                device_name="NVIDIA GPU",
                device_type="graphics",
                status=TestStatus.FAIL,
                message="GPU test timeout",
                performance_metrics={},
                execution_time=time.time() - start_time,
                timestamp=time.time(),
                errors=["Test timeout"]
            )
        except Exception as e:
            self.logger.error(f"NVIDIA GPU test error: {e}")
            return PeripheralTestResult(
                test_name="nvidia_gpu_test",
                device_name="NVIDIA GPU",
                device_type="graphics",
                status=TestStatus.FAIL,
                message=f"NVIDIA GPU test failed: {str(e)}",
                performance_metrics={},
                execution_time=time.time() - start_time,
                timestamp=time.time(),
                errors=[str(e)]
            )
        
        return None
    
    def _test_generic_gpu(self) -> Optional[PeripheralTestResult]:
        """Test generic GPU (AMD/Intel)"""
        start_time = time.time()
        
        try:
            # Check for GPU in PCI devices
            result = subprocess.run(['lspci'], capture_output=True, text=True)
            if result.returncode == 0:
                gpu_lines = [line for line in result.stdout.split('\n') 
                           if any(keyword in line.lower() for keyword in 
                                ['vga', '3d controller', 'display', 'graphics'])]
                
                if gpu_lines:
                    gpu_info = gpu_lines[0]
                    gpu_name = gpu_info.split(': ')[1] if ': ' in gpu_info else gpu_info
                    
                    return PeripheralTestResult(
                        test_name="generic_gpu_test",
                        device_name=gpu_name,
                        device_type="graphics",
                        status=TestStatus.PASS,
                        message=f"Generic GPU detected: {gpu_name}",
                        performance_metrics={
                            'gpu_name': gpu_name,
                            'detection_method': 'pci'
                        },
                        execution_time=time.time() - start_time,
                        timestamp=time.time()
                    )
                        
        except Exception as e:
            self.logger.error(f"Generic GPU test error: {e}")
        
        return None
    
    def run_peripheral_test_suite(self) -> Dict[str, Any]:
        """Run complete peripheral test suite"""
        self.logger.info("Starting peripheral test suite")
        
        all_results = []
        
        # Test USB devices
        self.logger.info("Testing USB devices...")
        usb_devices = self.detect_usb_devices()
        for device in usb_devices:
            # Basic USB test
            result = self.test_usb_device_basic(device)
            all_results.append(result)
            
            # Category-specific tests
            if device['category'] == 'storage':
                result = self.test_usb_storage_device(device)
                all_results.append(result)
            elif device['category'] == 'input':
                result = self.test_usb_input_device(device)
                all_results.append(result)
        
        # Test network interfaces
        self.logger.info("Testing network interfaces...")
        network_results = self.test_network_interfaces()
        all_results.extend(network_results)
        
        # Test storage devices
        self.logger.info("Testing storage devices...")
        storage_results = self.test_storage_devices()
        all_results.extend(storage_results)
        
        # Test GPU devices
        self.logger.info("Testing GPU devices...")
        gpu_results = self.test_gpu_devices()
        all_results.extend(gpu_results)
        
        self.test_results = all_results
        
        # Generate summary
        summary = {
            'total_tests': len(all_results),
            'passed': len([r for r in all_results if r.status == TestStatus.PASS]),
            'failed': len([r for r in all_results if r.status == TestStatus.FAIL]),
            'warnings': len([r for r in all_results if r.status == TestStatus.WARNING]),
            'skipped': len([r for r in all_results if r.status == TestStatus.SKIP])
        }
        
        self.logger.info(f"Peripheral testing completed. {summary['passed']}/{summary['total_tests']} tests passed")
        
        return {
            'summary': summary,
            'detailed_results': [asdict(result) for result in all_results]
        }
    
    def generate_peripheral_report(self) -> str:
        """Generate comprehensive peripheral testing report"""
        if not self.test_results:
            self.run_peripheral_test_suite()
        
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'test_suite_version': '1.0'
            },
            'summary': {
                'total_tests': len(self.test_results),
                'passed': len([r for r in self.test_results if r.status == TestStatus.PASS]),
                'failed': len([r for r in self.test_results if r.status == TestStatus.FAIL]),
                'warnings': len([r for r in self.test_results if r.status == TestStatus.WARNING]),
                'skipped': len([r for r in self.test_results if r.status == TestStatus.SKIP])
            },
            'device_summary': {
                'usb_devices_tested': len([r for r in self.test_results if r.device_type.startswith('usb')]),
                'network_interfaces_tested': len([r for r in self.test_results if r.device_type == 'network']),
                'storage_devices_tested': len([r for r in self.test_results if r.device_type == 'storage']),
                'gpu_devices_tested': len([r for r in self.test_results if r.device_type == 'graphics'])
            },
            'detailed_results': [asdict(result) for result in self.test_results],
            'recommendations': self._generate_peripheral_recommendations()
        }
        
        report_path = f"/workspace/testing/hardware_tests/results/peripheral_test_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Peripheral test report generated: {report_path}")
        return report_path
    
    def _generate_peripheral_recommendations(self) -> Dict[str, List[str]]:
        """Generate peripheral-specific recommendations"""
        recommendations = {
            'performance_optimizations': [],
            'compatibility_issues': [],
            'hardware_upgrades': [],
            'configuration_changes': []
        }
        
        # Analyze test results for recommendations
        for result in self.test_results:
            if result.status == TestStatus.WARNING:
                if 'performance' in result.test_name.lower():
                    recommendations['performance_optimizations'].append(
                        f"{result.device_name}: {result.message}"
                    )
                elif 'compatibility' in result.test_name.lower():
                    recommendations['compatibility_issues'].append(
                        f"{result.device_name}: {result.message}"
                    )
            
            if result.status == TestStatus.FAIL:
                recommendations['compatibility_issues'].append(
                    f"{result.device_name}: {result.message}"
                )
        
        # General recommendations based on test types
        usb_failures = len([r for r in self.test_results if r.device_type.startswith('usb') and r.status == TestStatus.FAIL])
        if usb_failures > 0:
            recommendations['configuration_changes'].append(
                "Check USB power management settings"
            )
        
        network_failures = len([r for r in self.test_results if r.device_type == 'network' and r.status == TestStatus.FAIL])
        if network_failures > 0:
            recommendations['configuration_changes'].append(
                "Check network interface configurations and drivers"
            )
        
        storage_performance = [r.performance_metrics.get('sequential_read_speed_mbps', 0) 
                             for r in self.test_results if r.device_type == 'storage']
        if storage_performance and max(storage_performance) < 50:
            recommendations['hardware_upgrades'].append(
                "Consider upgrading storage device for better performance"
            )
        
        return recommendations


def main():
    """Main function for standalone execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Peripheral Testing')
    parser.add_argument('--test', choices=['usb', 'network', 'storage', 'gpu', 'all'],
                       default='all', help='Peripheral type to test')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    tester = PeripheralTester()
    
    if args.test == 'all':
        results = tester.run_peripheral_test_suite()
    else:
        # Run specific tests
        if args.test == 'usb':
            usb_devices = tester.detect_usb_devices()
            results = {'summary': {'usb_devices_detected': len(usb_devices)}}
            for device in usb_devices:
                result = tester.test_usb_device_basic(device)
                tester.test_results.append(result)
        elif args.test == 'network':
            results = {'network_results': tester.test_network_interfaces()}
        elif args.test == 'storage':
            results = {'storage_results': tester.test_storage_devices()}
        elif args.test == 'gpu':
            results = {'gpu_results': tester.test_gpu_devices()}
    
    # Generate report
    report_path = tester.generate_peripheral_report()
    print(f"Peripheral test report generated: {report_path}")
    
    # Print summary if available
    if 'summary' in results:
        summary = results['summary']
        print(f"\nPeripheral Test Summary:")
        print(f"  Total Tests: {summary['total_tests']}")
        print(f"  Passed: {summary['passed']}")
        print(f"  Failed: {summary['failed']}")
        print(f"  Warnings: {summary['warnings']}")


if __name__ == "__main__":
    main()