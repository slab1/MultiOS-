#!/usr/bin/env python3
"""
Hardware Detection and Auto-Configuration System
Detects hardware components and automatically configures test environments
"""

import os
import sys
import subprocess
import json
import re
import logging
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import platform
import socket
import threading
import time
import psutil
import glob

class HardwareDetector:
    """Main class for hardware detection and auto-configuration"""
    
    def __init__(self, config_path: str = None):
        self.logger = self._setup_logging()
        self.config_path = config_path or "/workspace/testing/hardware_tests/config"
        self.detected_hardware = {}
        self.test_config = {}
        
    def _setup_logging(self):
        """Setup logging configuration"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/hardware_detection.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def detect_cpu_info(self) -> Dict:
        """Detect CPU information"""
        cpu_info = {
            'model': 'Unknown',
            'architecture': platform.machine(),
            'cores_physical': psutil.cpu_count(logical=False),
            'cores_logical': psutil.cpu_count(logical=True),
            'frequency_current': 0,
            'frequency_max': 0,
            'cache_l1': 0,
            'cache_l2': 0,
            'cache_l3': 0,
            'features': [],
            'vendor': 'Unknown',
            'stepping': 'Unknown',
            'model_number': 'Unknown',
            'flags': []
        }
        
        try:
            # Try /proc/cpuinfo first (Linux)
            if os.path.exists('/proc/cpuinfo'):
                with open('/proc/cpuinfo', 'r') as f:
                    cpuinfo_content = f.read()
                    
                # Extract CPU model
                model_match = re.search(r'model name\s*:\s*(.+)', cpuinfo_content)
                if model_match:
                    cpu_info['model'] = model_match.group(1).strip()
                
                # Extract vendor (Intel/AMD detection)
                if 'GenuineIntel' in cpuinfo_content:
                    cpu_info['vendor'] = 'Intel'
                elif 'AuthenticAMD' in cpuinfo_content:
                    cpu_info['vendor'] = 'AMD'
                    
                # Extract cache sizes
                l1_match = re.search(r'cache size\s*:\s*(\d+)\s*KB', cpuinfo_content)
                if l1_match:
                    cpu_info['cache_l1'] = int(l1_match.group(1))
                
                # Extract flags/features
                flags_match = re.search(r'flags\s*:\s*(.+)', cpuinfo_content)
                if flags_match:
                    cpu_info['flags'] = flags_match.group(1).split()
                    
            # Get frequency information
            try:
                cpu_freq = psutil.cpu_freq()
                if cpu_freq:
                    cpu_info['frequency_current'] = cpu_freq.current
                    cpu_info['frequency_max'] = cpu_freq.max
            except:
                pass
                
            # Get CPU features via lscpu if available
            try:
                result = subprocess.run(['lscpu'], capture_output=True, text=True)
                if result.returncode == 0:
                    lines = result.stdout.split('\n')
                    for line in lines:
                        if 'Model name' in line:
                            cpu_info['model'] = line.split(':')[1].strip()
                        elif 'CPU(s):' in line and 'Thread' not in line:
                            cpu_info['cores_logical'] = int(line.split(':')[1].strip())
                        elif 'Core(s) per socket:' in line:
                            cpu_info['cores_physical'] = int(line.split(':')[1].strip())
                        elif 'CPU family:' in line:
                            cpu_info['family'] = int(line.split(':')[1].strip())
                        elif 'Model:' in line:
                            cpu_info['model_number'] = line.split(':')[1].strip()
                        elif 'Stepping:' in line:
                            cpu_info['stepping'] = line.split(':')[1].strip()
            except:
                pass
                
        except Exception as e:
            self.logger.error(f"Error detecting CPU info: {e}")
            
        return cpu_info
    
    def detect_memory_info(self) -> Dict:
        """Detect memory information"""
        memory_info = {
            'total_gb': round(psutil.virtual_memory().total / (1024**3), 2),
            'available_gb': round(psutil.virtual_memory().available / (1024**3), 2),
            'memory_type': 'Unknown',
            'speed_mhz': 0,
            'channels': 0,
            'slots_used': 0,
            'slots_total': 0,
            'ecc_support': False,
            'manufacturer': 'Unknown'
        }
        
        try:
            # Try dmidecode for detailed memory info
            result = subprocess.run(['dmidecode', '-t', 'memory'], capture_output=True, text=True)
            if result.returncode == 0:
                memory_details = result.stdout
                
                # Extract memory type
                type_match = re.search(r'Type:\s*(.+)', memory_details)
                if type_match:
                    memory_info['memory_type'] = type_match.group(1).strip()
                
                # Extract speed
                speed_match = re.search(r'Speed:\s*(\d+)\s*MHz', memory_details)
                if speed_match:
                    memory_info['speed_mhz'] = int(speed_match.group(1))
                
                # Extract ECC support
                if 'ECC' in memory_details:
                    memory_info['ecc_support'] = True
                    
                # Count memory slots
                memory_info['slots_used'] = len(re.findall(r'Size:\s*\d+', memory_details))
                memory_info['slots_total'] = len(re.findall(r'Locator:\s*', memory_details))
                
        except Exception as e:
            self.logger.warning(f"Could not get detailed memory info: {e}")
            
        return memory_info
    
    def detect_storage_info(self) -> List[Dict]:
        """Detect storage devices"""
        storage_devices = []
        
        try:
            # Get disk information using psutil
            disk_usage = psutil.disk_usage('/')
            disk_info = {
                'device': 'root',
                'mountpoint': '/',
                'total_gb': round(disk_usage.total / (1024**3), 2),
                'free_gb': round(disk_usage.free / (1024**3), 2),
                'filesystem': 'unknown',
                'device_type': 'unknown'
            }
            storage_devices.append(disk_info)
            
            # Try lsblk for more detailed information
            try:
                result = subprocess.run(['lsblk', '-J', '-o', 'NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT,MODEL'], 
                                      capture_output=True, text=True)
                if result.returncode == 0:
                    data = json.loads(result.stdout)
                    for device in data.get('blockdevices', []):
                        device_info = {
                            'device': device.get('name', 'unknown'),
                            'size_gb': self._parse_size(device.get('size', '0')),
                            'type': device.get('type', 'unknown'),
                            'filesystem': device.get('fstype', 'unknown'),
                            'mountpoint': device.get('mountpoint', 'none'),
                            'model': device.get('model', 'unknown')
                        }
                        
                        # Determine device type
                        if device_info['type'] == 'disk':
                            device_info['device_type'] = 'HDD/SSD'
                        elif device_info['type'] == 'part':
                            device_info['device_type'] = 'Partition'
                        elif device_info['type'] == 'loop':
                            device_info['device_type'] = 'Loop Device'
                            
                        storage_devices.append(device_info)
                        
            except Exception as e:
                self.logger.warning(f"Could not get detailed disk info: {e}")
                
        except Exception as e:
            self.logger.error(f"Error detecting storage: {e}")
            
        return storage_devices
    
    def _parse_size(self, size_str):
        """Parse size string to GB"""
        try:
            # Remove any non-numeric characters except decimal point
            size_str = re.sub(r'[^\d.]', '', size_str)
            if not size_str:
                return 0
            size_bytes = int(float(size_str) * (1024**3))
            return round(size_bytes / (1024**3), 2)
        except:
            return 0
    
    def detect_network_info(self) -> List[Dict]:
        """Detect network interfaces"""
        network_interfaces = []
        
        try:
            for interface, addrs in psutil.net_if_addrs().items():
                if interface.startswith(('lo', 'docker', 'veth')):
                    continue
                    
                interface_info = {
                    'name': interface,
                    'mac_address': 'unknown',
                    'ip_addresses': [],
                    'ipv6_addresses': [],
                    'is_ethernet': False,
                    'is_wireless': False,
                    'speed_mbps': 0,
                    'status': 'unknown'
                }
                
                # Check if it's ethernet or wireless
                interface_lower = interface.lower()
                if any(x in interface_lower for x in ['eth', 'enp', 'enx', 'en']):
                    interface_info['is_ethernet'] = True
                elif any(x in interface_lower for x in ['wlan', 'wlp', 'wlan']):
                    interface_info['is_wireless'] = True
                
                # Get interface statistics
                try:
                    stats = psutil.net_if_stats()[interface]
                    interface_info['status'] = 'up' if stats.isup else 'down'
                    interface_info['speed_mbps'] = stats.speed if stats.speed > 0 else 1000
                except:
                    pass
                
                # Get addresses
                for addr in addrs:
                    if addr.family == socket.AF_INET:
                        interface_info['ip_addresses'].append(addr.address)
                    elif addr.family == socket.AF_INET6:
                        interface_info['ipv6_addresses'].append(addr.address)
                    elif hasattr(socket, 'AF_LINK'):
                        if addr.family == socket.AF_LINK:
                            interface_info['mac_address'] = addr.address
                
                network_interfaces.append(interface_info)
                
        except Exception as e:
            self.logger.error(f"Error detecting network interfaces: {e}")
            
        return network_interfaces
    
    def detect_gpu_info(self) -> List[Dict]:
        """Detect GPU information"""
        gpu_devices = []
        
        # Try nvidia-smi for NVIDIA GPUs
        try:
            result = subprocess.run(['nvidia-smi', '--query-gpu=name,memory.total,temperature.gpu,power.draw', 
                                   '--format=csv,noheader,nounits'], 
                                  capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                lines = result.stdout.strip().split('\n')
                for line in lines:
                    parts = line.split(', ')
                    if len(parts) >= 2:
                        gpu_info = {
                            'name': parts[0],
                            'memory_mb': int(parts[1]) if parts[1] != '[Not Supported]' else 0,
                            'temperature': int(parts[2]) if len(parts) > 2 and parts[2] != '[Not Supported]' else 0,
                            'power_draw_w': float(parts[3]) if len(parts) > 3 and parts[3] != '[Not Supported]' else 0.0,
                            'vendor': 'NVIDIA',
                            'driver_version': 'unknown'
                        }
                        gpu_devices.append(gpu_info)
        except:
            pass
        
        # Try lspci for other GPUs
        try:
            result = subprocess.run(['lspci'], capture_output=True, text=True)
            if result.returncode == 0:
                lines = result.stdout.split('\n')
                for line in lines:
                    if 'VGA' in line or '3D controller' in line or 'Display' in line:
                        gpu_info = {
                            'name': line.split(': ')[1].strip() if ': ' in line else line.strip(),
                            'vendor': 'Unknown',
                            'driver_version': 'unknown',
                            'memory_mb': 0
                        }
                        
                        # Identify vendor
                        if 'NVIDIA' in line:
                            gpu_info['vendor'] = 'NVIDIA'
                        elif 'AMD' in line or 'ATI' in line:
                            gpu_info['vendor'] = 'AMD'
                        elif 'Intel' in line:
                            gpu_info['vendor'] = 'Intel'
                            
                        gpu_devices.append(gpu_info)
        except:
            pass
            
        return gpu_devices
    
    def detect_usb_devices(self) -> List[Dict]:
        """Detect USB devices"""
        usb_devices = []
        
        try:
            # Try lsusb
            result = subprocess.run(['lsusb'], capture_output=True, text=True)
            if result.returncode == 0:
                lines = result.stdout.strip().split('\n')
                for line in lines:
                    if line.strip():
                        parts = line.split()
                        if len(parts) >= 6:
                            bus = parts[1]
                            device = parts[3].rstrip(':')
                            vendor_id = parts[5].split(':')[0]
                            product_id = parts[5].split(':')[1]
                            
                            # Try to get device name from lsusb -s option
                            try:
                                detail_result = subprocess.run(['lsusb', '-s', f'{bus}:{device}', '-v'], 
                                                             capture_output=True, text=True, timeout=5)
                                product_name = "Unknown USB Device"
                                if detail_result.returncode == 0:
                                    # Extract product name from verbose output
                                    name_match = re.search(r'iProduct\s+\d+\s+(.+)', detail_result.stdout)
                                    if name_match:
                                        product_name = name_match.group(1).strip()
                            except:
                                pass
                            
                            usb_device = {
                                'bus': bus,
                                'device': device,
                                'vendor_id': vendor_id,
                                'product_id': product_id,
                                'name': product_name,
                                'speed': 'unknown',
                                'power_requirement': 'unknown'
                            }
                            usb_devices.append(usb_device)
        except:
            pass
            
        return usb_devices
    
    def detect_system_info(self) -> Dict:
        """Detect general system information"""
        system_info = {
            'hostname': socket.gethostname(),
            'os_name': platform.system(),
            'os_release': platform.release(),
            'os_version': platform.version(),
            'architecture': platform.machine(),
            'python_version': platform.python_version(),
            'uptime_seconds': time.time() - psutil.boot_time(),
            'load_average': list(os.getloadavg()) if hasattr(os, 'getloadavg') else [0, 0, 0],
            'temperature_sensors': [],
            'fan_sensors': [],
            'voltage_sensors': []
        }
        
        # Try to get hardware sensors
        try:
            # Try lm-sensors
            result = subprocess.run(['sensors'], capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                output = result.stdout
                
                # Extract temperature sensors
                temp_matches = re.findall(r'(\w+.*?):\s*\+?([\d.]+)°C', output)
                for name, temp in temp_matches:
                    system_info['temperature_sensors'].append({
                        'name': name.strip(),
                        'temperature_c': float(temp)
                    })
                
                # Extract fan sensors
                fan_matches = re.findall(r'(\w+.*?):\s*([\d.]+)\s*RPM', output)
                for name, rpm in fan_matches:
                    system_info['fan_sensors'].append({
                        'name': name.strip(),
                        'speed_rpm': float(rpm)
                    })
                    
                # Extract voltage sensors
                voltage_matches = re.findall(r'(\w+.*?):\s*\+?([\d.]+) V', output)
                for name, voltage in voltage_matches:
                    system_info['voltage_sensors'].append({
                        'name': name.strip(),
                        'voltage_v': float(voltage)
                    })
                    
        except:
            pass
            
        return system_info
    
    def run_full_detection(self) -> Dict:
        """Run complete hardware detection"""
        self.logger.info("Starting hardware detection...")
        
        detected_hardware = {
            'system': self.detect_system_info(),
            'cpu': self.detect_cpu_info(),
            'memory': self.detect_memory_info(),
            'storage': self.detect_storage_info(),
            'network': self.detect_network_info(),
            'gpu': self.detect_gpu_info(),
            'usb_devices': self.detect_usb_devices()
        }
        
        self.detected_hardware = detected_hardware
        self.logger.info("Hardware detection completed")
        
        return detected_hardware
    
    def generate_hardware_profile(self, save_path: str = None) -> str:
        """Generate hardware profile file"""
        if not self.detected_hardware:
            self.run_full_detection()
            
        profile = {
            'detection_timestamp': time.time(),
            'profile_version': '1.0',
            'hardware': self.detected_hardware,
            'test_recommendations': self._generate_test_recommendations()
        }
        
        profile_path = save_path or f"/workspace/testing/hardware_tests/profiles/hardware_profile_{int(time.time())}.json"
        
        with open(profile_path, 'w') as f:
            json.dump(profile, f, indent=2, default=str)
            
        self.logger.info(f"Hardware profile saved to {profile_path}")
        return profile_path
    
    def _generate_test_recommendations(self) -> Dict:
        """Generate test recommendations based on detected hardware"""
        recommendations = {
            'cpu_tests': [],
            'memory_tests': [],
            'storage_tests': [],
            'network_tests': [],
            'gpu_tests': [],
            'thermal_tests': [],
            'power_tests': [],
            'peripheral_tests': []
        }
        
        cpu = self.detected_hardware.get('cpu', {})
        memory = self.detected_hardware.get('memory', {})
        storage = self.detected_hardware.get('storage', [])
        network = self.detected_hardware.get('network', [])
        gpu = self.detected_hardware.get('gpu', [])
        system = self.detected_hardware.get('system', {})
        
        # CPU test recommendations
        if cpu.get('cores_physical', 1) > 1:
            recommendations['cpu_tests'].extend([
                'multi_core_stress_test',
                'thread_scaling_test',
                'cpu_affinity_test'
            ])
            
        if cpu.get('cores_logical', 1) > cpu.get('cores_physical', 1):
            recommendations['cpu_tests'].append('hyperthreading_validation')
            
        # Memory test recommendations
        if memory.get('total_gb', 0) > 16:
            recommendations['memory_tests'].extend([
                'large_memory_stress_test',
                'memory_bandwidth_test'
            ])
            
        if memory.get('ecc_support'):
            recommendations['memory_tests'].append('ecc_memory_test')
            
        # Storage test recommendations
        for device in storage:
            if device.get('type') == 'disk':
                if device.get('size_gb', 0) > 500:
                    recommendations['storage_tests'].append('large_disk_test')
                    
        # Network test recommendations
        for interface in network:
            if interface.get('speed_mbps', 0) >= 1000:
                recommendations['network_tests'].append('gigabit_performance_test')
                
        # GPU test recommendations
        if gpu:
            recommendations['gpu_tests'].extend([
                'gpu_memory_test',
                'gpu_compute_test'
            ])
            
        # Thermal test recommendations
        if system.get('temperature_sensors'):
            recommendations['thermal_tests'].append('thermal_stress_test')
            
        # Power test recommendations
        if gpu or len(cpu.get('cores_logical', [])) > 4:
            recommendations['power_tests'].append('power_consumption_test')
            
        return recommendations
    
    def export_config(self, output_path: str = None):
        """Export hardware configuration for test framework"""
        if not self.detected_hardware:
            self.run_full_detection()
            
        config = {
            'hardware_profile': self.detected_hardware,
            'test_config': self._generate_test_config(),
            'optimization_settings': self._generate_optimization_settings()
        }
        
        config_path = output_path or "/workspace/testing/hardware_tests/config/hardware_config.json"
        
        with open(config_path, 'w') as f:
            json.dump(config, f, indent=2, default=str)
            
        self.logger.info(f"Hardware configuration exported to {config_path}")
        return config_path
    
    def _generate_test_config(self) -> Dict:
        """Generate test configuration based on hardware"""
        cpu = self.detected_hardware.get('cpu', {})
        memory = self.detected_hardware.get('memory', {})
        
        config = {
            'parallel_workers': min(cpu.get('cores_logical', 4), 16),
            'memory_test_size_gb': min(memory.get('total_gb', 8) * 0.8, 32),
            'stress_test_duration_minutes': 30,
            'temperature_warning_threshold': 80,
            'temperature_critical_threshold': 90,
            'power_test_enabled': True,
            'gpu_tests_enabled': len(self.detected_hardware.get('gpu', [])) > 0,
            'network_tests_enabled': len(self.detected_hardware.get('network', [])) > 0
        }
        
        return config
    
    def _generate_optimization_settings(self) -> Dict:
        """Generate hardware-specific optimization settings"""
        optimizations = {
            'cpu_scaling_governor': 'performance',
            'memory_frequency': 'auto',
            'gpu_power_limit': 'default',
            'thermal_policy': 'balanced'
        }
        
        cpu = self.detected_hardware.get('cpu', {})
        
        # Adjust based on hardware
        if cpu.get('cores_physical', 1) >= 8:
            optimizations['parallel_test_workers'] = min(cpu.get('cores_logical', 16), 16)
        else:
            optimizations['parallel_test_workers'] = min(cpu.get('cores_logical', 4), 8)
            
        return optimizations


def main():
    """Main function for standalone execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Hardware Detection and Configuration')
    parser.add_argument('--profile', action='store_true', help='Generate hardware profile')
    parser.add_argument('--config', action='store_true', help='Export configuration')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    detector = HardwareDetector()
    
    if args.profile:
        detector.run_full_detection()
        detector.generate_hardware_profile(args.output)
    
    if args.config:
        detector.run_full_detection()
        detector.export_config(args.output)
    
    if not args.profile and not args.config:
        detector.run_full_detection()
        print(json.dumps(detector.detected_hardware, indent=2, default=str))


if __name__ == "__main__":
    main()