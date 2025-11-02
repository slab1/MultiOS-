#!/usr/bin/env python3
"""
Hardware Compatibility Testing Framework
Automated testing for hardware compatibility, drivers, and system integration
"""

import os
import sys
import json
import subprocess
import time
import logging
import hashlib
import tempfile
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import psutil
import threading

class CompatibilityStatus(Enum):
    UNKNOWN = "unknown"
    COMPATIBLE = "compatible"
    INCOMPATIBLE = "incompatible"
    PARTIAL = "partial"
    NEEDS_DRIVER = "needs_driver"
    WARNING = "warning"

@dataclass
class CompatibilityTest:
    """Individual compatibility test definition"""
    test_id: str
    name: str
    description: str
    category: str  # cpu, memory, storage, network, gpu, usb, etc.
    priority: int  # 1-5, higher is more important
    dependencies: List[str]
    test_function: str
    expected_result: str
    timeout_seconds: int = 300

@dataclass
class CompatibilityResult:
    """Result of a compatibility test"""
    test_id: str
    status: CompatibilityStatus
    message: str
    details: Dict[str, Any]
    execution_time: float
    timestamp: float
    performance_metrics: Optional[Dict[str, Any]] = None
    warnings: List[str] = None

class HardwareCompatibilityTester:
    """Main class for hardware compatibility testing"""
    
    def __init__(self, config_path: str = None):
        self.logger = self._setup_logging()
        self.config_path = config_path or "/workspace/testing/hardware_tests/config"
        self.test_database = self._load_test_database()
        self.results = []
        self.current_hardware_profile = {}
        
    def _setup_logging(self):
        """Setup logging for compatibility testing"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/compatibility_test.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def _load_test_database(self) -> List[CompatibilityTest]:
        """Load test database from file or create default"""
        db_file = Path(self.config_path) / "compatibility_tests.json"
        
        if db_file.exists():
            with open(db_file, 'r') as f:
                data = json.load(f)
                return [CompatibilityTest(**test) for test in data]
        
        # Create default test database
        default_tests = [
            # CPU Tests
            CompatibilityTest(
                test_id="cpu_instruction_set",
                name="CPU Instruction Set Compatibility",
                description="Verify CPU supports required instruction sets",
                category="cpu",
                priority=5,
                dependencies=[],
                test_function="test_cpu_instruction_set",
                expected_result="all_required_instructions_available"
            ),
            CompatibilityTest(
                test_id="cpu_multicore_scaling",
                name="CPU Multi-core Scaling",
                description="Test performance scaling across CPU cores",
                category="cpu",
                priority=4,
                dependencies=["cpu_instruction_set"],
                test_function="test_cpu_multicore_scaling",
                expected_result="linear_scaling_to_available_cores"
            ),
            CompatibilityTest(
                test_id="cpu_virtualization",
                name="CPU Virtualization Support",
                description="Check for hardware virtualization support",
                category="cpu",
                priority=3,
                dependencies=["cpu_instruction_set"],
                test_function="test_cpu_virtualization",
                expected_result="virtualization_support_detected"
            ),
            
            # Memory Tests
            CompatibilityTest(
                test_id="memory_size",
                name="Memory Size Compatibility",
                description="Verify sufficient memory for workloads",
                category="memory",
                priority=5,
                dependencies=[],
                test_function="test_memory_size",
                expected_result="minimum_memory_requirements_met"
            ),
            CompatibilityTest(
                test_id="memory_bandwidth",
                name="Memory Bandwidth",
                description="Test memory bandwidth performance",
                category="memory",
                priority=3,
                dependencies=["memory_size"],
                test_function="test_memory_bandwidth",
                expected_result="acceptable_bandwidth_measured"
            ),
            CompatibilityTest(
                test_id="memory_ecc",
                name="ECC Memory Support",
                description="Check ECC memory detection and functionality",
                category="memory",
                priority=2,
                dependencies=["memory_size"],
                test_function="test_memory_ecc",
                expected_result="ecc_memory_detected"
            ),
            
            # Storage Tests
            CompatibilityTest(
                test_id="storage_interface",
                name="Storage Interface Compatibility",
                description="Test storage device interfaces",
                category="storage",
                priority=4,
                dependencies=[],
                test_function="test_storage_interface",
                expected_result="storage_interfaces_functional"
            ),
            CompatibilityTest(
                test_id="storage_performance",
                name="Storage Performance",
                description="Measure storage read/write performance",
                category="storage",
                priority=3,
                dependencies=["storage_interface"],
                test_function="test_storage_performance",
                expected_result="acceptable_performance_measured"
            ),
            CompatibilityTest(
                test_id="storage_raid",
                name="RAID Configuration",
                description="Test RAID array functionality",
                category="storage",
                priority=2,
                dependencies=["storage_interface"],
                test_function="test_storage_raid",
                expected_result="raid_arrays_detected"
            ),
            
            # Network Tests
            CompatibilityTest(
                test_id="network_interfaces",
                name="Network Interface Detection",
                description="Verify all network interfaces are detected",
                category="network",
                priority=4,
                dependencies=[],
                test_function="test_network_interfaces",
                expected_result="all_interfaces_detected"
            ),
            CompatibilityTest(
                test_id="network_performance",
                name="Network Performance",
                description="Test network throughput and latency",
                category="network",
                priority=3,
                dependencies=["network_interfaces"],
                test_function="test_network_performance",
                expected_result="acceptable_performance"
            ),
            
            # GPU Tests
            CompatibilityTest(
                test_id="gpu_detection",
                name="GPU Detection",
                description="Detect and identify GPU devices",
                category="gpu",
                priority=4,
                dependencies=[],
                test_function="test_gpu_detection",
                expected_result="gpus_detected"
            ),
            CompatibilityTest(
                test_id="gpu_drivers",
                name="GPU Driver Compatibility",
                description="Check GPU driver installation and functionality",
                category="gpu",
                priority=5,
                dependencies=["gpu_detection"],
                test_function="test_gpu_drivers",
                expected_result="drivers_functional"
            ),
            CompatibilityTest(
                test_id="gpu_performance",
                name="GPU Performance",
                description="Test GPU compute performance",
                category="gpu",
                priority=3,
                dependencies=["gpu_drivers"],
                test_function="test_gpu_performance",
                expected_result="acceptable_gpu_performance"
            ),
            
            # USB Tests
            CompatibilityTest(
                test_id="usb_detection",
                name="USB Device Detection",
                description="Test USB device enumeration",
                category="usb",
                priority=3,
                dependencies=[],
                test_function="test_usb_detection",
                expected_result="usb_devices_enumerated"
            ),
            CompatibilityTest(
                test_id="usb_performance",
                name="USB Performance",
                description="Test USB transfer speeds",
                category="usb",
                priority=2,
                dependencies=["usb_detection"],
                test_function="test_usb_performance",
                expected_result="acceptable_usb_performance"
            ),
            
            # System Tests
            CompatibilityTest(
                test_id="system_power",
                name="Power Management",
                description="Test power management features",
                category="system",
                priority=3,
                dependencies=[],
                test_function="test_system_power",
                expected_result="power_management_functional"
            ),
            CompatibilityTest(
                test_id="system_thermal",
                name="Thermal Management",
                description="Test thermal monitoring and management",
                category="system",
                priority=4,
                dependencies=[],
                test_function="test_system_thermal",
                expected_result="thermal_management_operational"
            )
        ]
        
        # Save default database
        os.makedirs(os.path.dirname(db_file), exist_ok=True)
        with open(db_file, 'w') as f:
            json.dump([asdict(test) for test in default_tests], f, indent=2)
        
        return default_tests
    
    def detect_current_hardware(self) -> Dict[str, Any]:
        """Detect current hardware configuration"""
        from hardware_detector import HardwareDetector
        
        detector = HardwareDetector()
        self.current_hardware_profile = detector.run_full_detection()
        return self.current_hardware_profile
    
    def run_compatibility_test(self, test: CompatibilityTest) -> CompatibilityResult:
        """Run a single compatibility test"""
        start_time = time.time()
        test_id = test.test_id
        
        self.logger.info(f"Running compatibility test: {test.name}")
        
        try:
            # Get test function from this class
            test_func = getattr(self, test.test_function, None)
            if not test_func:
                return CompatibilityResult(
                    test_id=test_id,
                    status=CompatibilityStatus.UNKNOWN,
                    message=f"Test function {test.test_function} not found",
                    details={},
                    execution_time=time.time() - start_time,
                    timestamp=time.time(),
                    warnings=[f"Test function {test.test_function} missing"]
                )
            
            # Run the test
            result = test_func(test)
            
            return CompatibilityResult(
                test_id=test_id,
                status=result.get('status', CompatibilityStatus.UNKNOWN),
                message=result.get('message', ''),
                details=result.get('details', {}),
                execution_time=time.time() - start_time,
                timestamp=time.time(),
                performance_metrics=result.get('performance_metrics'),
                warnings=result.get('warnings', [])
            )
            
        except Exception as e:
            self.logger.error(f"Error running test {test_id}: {e}")
            return CompatibilityResult(
                test_id=test_id,
                status=CompatibilityStatus.UNKNOWN,
                message=f"Test execution failed: {str(e)}",
                details={'error': str(e)},
                execution_time=time.time() - start_time,
                timestamp=time.time(),
                warnings=[f"Test execution error: {str(e)}"]
            )
    
    # Test Implementation Methods
    
    def test_cpu_instruction_set(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test CPU instruction set compatibility"""
        cpu_info = self.current_hardware_profile.get('cpu', {})
        flags = cpu_info.get('flags', [])
        
        required_instructions = {
            'sse4_1': 'SSE 4.1 support',
            'sse4_2': 'SSE 4.2 support',
            'avx': 'AVX support',
            'avx2': 'AVX2 support',
            'fma': 'FMA support',
            'bmi1': 'BMI1 support',
            'bmi2': 'BMI2 support'
        }
        
        missing_instructions = []
        available_instructions = []
        
        for flag, description in required_instructions.items():
            if flag in flags:
                available_instructions.append(description)
            else:
                missing_instructions.append(description)
        
        status = CompatibilityStatus.COMPATIBLE
        message = f"CPU supports {len(available_instructions)}/{len(required_instructions)} instruction sets"
        
        if missing_instructions:
            status = CompatibilityStatus.PARTIAL
            message += f", missing: {', '.join(missing_instructions)}"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'available_instructions': available_instructions,
                'missing_instructions': missing_instructions,
                'cpu_model': cpu_info.get('model', 'Unknown')
            },
            'performance_metrics': {
                'instruction_sets_supported': len(available_instructions),
                'total_instruction_sets_checked': len(required_instructions)
            }
        }
    
    def test_cpu_multicore_scaling(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test CPU multi-core scaling performance"""
        cpu_info = self.current_hardware_profile.get('cpu', {})
        logical_cores = cpu_info.get('cores_logical', 1)
        physical_cores = cpu_info.get('cores_physical', 1)
        
        # Run multi-threaded calculation test
        def cpu_intensive_task(duration=5):
            end_time = time.time() + duration
            count = 0
            while time.time() < end_time:
                count += sum(range(1000))
            return count
        
        # Test single thread
        start_time = time.time()
        single_thread_result = cpu_intensive_task()
        single_thread_time = time.time() - start_time
        
        # Test multi thread
        start_time = time.time()
        threads = []
        for _ in range(min(logical_cores, 8)):  # Limit to 8 threads for testing
            thread = threading.Thread(target=cpu_intensive_task)
            thread.start()
            threads.append(thread)
        
        for thread in threads:
            thread.join()
        
        multi_thread_time = time.time() - start_time
        
        # Calculate scaling factor
        expected_speedup = min(logical_cores, 8)
        actual_speedup = single_thread_time / multi_thread_time if multi_thread_time > 0 else 0
        scaling_efficiency = (actual_speedup / expected_speedup) * 100 if expected_speedup > 0 else 0
        
        status = CompatibilityStatus.COMPATIBLE
        message = f"Multi-core scaling: {actual_speedup:.2f}x speedup on {expected_speedup} cores"
        
        if scaling_efficiency < 70:
            status = CompatibilityStatus.WARNING
            message += f" (efficiency: {scaling_efficiency:.1f}%)"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'logical_cores': logical_cores,
                'physical_cores': physical_cores,
                'expected_speedup': expected_speedup,
                'actual_speedup': actual_speedup,
                'scaling_efficiency_percent': scaling_efficiency
            },
            'performance_metrics': {
                'single_thread_time': single_thread_time,
                'multi_thread_time': multi_thread_time,
                'speedup_factor': actual_speedup
            }
        }
    
    def test_cpu_virtualization(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test CPU virtualization support"""
        cpu_info = self.current_hardware_profile.get('cpu', {})
        flags = cpu_info.get('flags', [])
        
        virtualization_flags = ['vmx', 'svm']  # Intel VT-x, AMD-V
        
        has_virtualization = any(flag in flags for flag in virtualization_flags)
        
        status = CompatibilityStatus.COMPATIBLE if has_virtualization else CompatibilityStatus.NEEDS_DRIVER
        message = "Virtualization supported" if has_virtualization else "Virtualization not detected"
        
        if not has_virtualization:
            # Check if virtualization is disabled in BIOS
            try:
                result = subprocess.run(['systemd-detect-virt'], capture_output=True, text=True)
                if result.returncode == 0 and result.stdout.strip() != 'none':
                    message += f" (running in virtualized environment: {result.stdout.strip()})"
            except:
                pass
        
        return {
            'status': status,
            'message': message,
            'details': {
                'virtualization_flags': [flag for flag in virtualization_flags if flag in flags],
                'cpu_vendor': cpu_info.get('vendor', 'Unknown')
            }
        }
    
    def test_memory_size(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test memory size compatibility"""
        memory_info = self.current_hardware_profile.get('memory', {})
        total_memory_gb = memory_info.get('total_gb', 0)
        
        # Define minimum requirements for different use cases
        requirements = {
            'minimal': 4,      # GB
            'standard': 8,     # GB
            'performance': 16, # GB
            'workstation': 32  # GB
        }
        
        compatible_configs = []
        warnings = []
        
        for config, min_memory in requirements.items():
            if total_memory_gb >= min_memory:
                compatible_configs.append(config)
            else:
                warnings.append(f"Insufficient memory for {config} configuration ({min_memory}GB required)")
        
        status = CompatibilityStatus.WARNING if not compatible_configs else CompatibilityStatus.COMPATIBLE
        message = f"Memory: {total_memory_gb}GB (compatible with: {', '.join(compatible_configs)})"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'total_memory_gb': total_memory_gb,
                'compatible_configurations': compatible_configs,
                'requirements_met': len(compatible_configs),
                'warnings': warnings
            }
        }
    
    def test_memory_bandwidth(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test memory bandwidth performance"""
        memory_info = self.current_hardware_profile.get('memory', {})
        
        try:
            # Simple bandwidth test using dd
            test_file = '/tmp/memory_bandwidth_test.dat'
            test_size = 100 * 1024 * 1024  # 100MB
            
            # Write test
            start_time = time.time()
            subprocess.run(['dd', 'if=/dev/zero', f'of={test_file}', 'bs=1M', 
                          f'count={test_size//(1024*1024)}'], 
                          capture_output=True, timeout=60)
            write_time = time.time() - start_time
            
            # Read test
            start_time = time.time()
            subprocess.run(['dd', f'if={test_file}', 'of=/dev/null', 'bs=1M'], 
                          capture_output=True, timeout=60)
            read_time = time.time() - start_time
            
            # Cleanup
            os.remove(test_file)
            
            # Calculate bandwidth in MB/s
            write_bandwidth = (test_size / (1024*1024)) / write_time if write_time > 0 else 0
            read_bandwidth = (test_size / (1024*1024)) / read_time if read_time > 0 else 0
            
            # Expected bandwidth based on memory type
            memory_type = memory_info.get('memory_type', 'Unknown')
            memory_speed = memory_info.get('speed_mhz', 0)
            
            status = CompatibilityStatus.COMPATIBLE
            message = f"Memory bandwidth: {read_bandwidth:.1f}MB/s read, {write_bandwidth:.1f}MB/s write"
            
            # Performance warnings
            warnings = []
            if read_bandwidth < 5000:  # Less than 5GB/s is concerning for modern systems
                status = CompatibilityStatus.WARNING
                warnings.append("Memory bandwidth lower than expected")
            
            return {
                'status': status,
                'message': message,
                'details': {
                    'memory_type': memory_type,
                    'memory_speed_mhz': memory_speed,
                    'read_bandwidth_mbps': read_bandwidth,
                    'write_bandwidth_mbps': write_bandwidth,
                    'test_size_mb': test_size // (1024*1024),
                    'warnings': warnings
                },
                'performance_metrics': {
                    'read_bandwidth_mbps': read_bandwidth,
                    'write_bandwidth_mbps': write_bandwidth
                }
            }
            
        except Exception as e:
            return {
                'status': CompatibilityStatus.UNKNOWN,
                'message': f"Memory bandwidth test failed: {str(e)}",
                'details': {'error': str(e)}
            }
    
    def test_memory_ecc(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test ECC memory support"""
        memory_info = self.current_hardware_profile.get('memory', {})
        
        ecc_supported = memory_info.get('ecc_support', False)
        
        status = CompatibilityStatus.COMPATIBLE if not ecc_supported else CompatibilityStatus.COMPATIBLE
        message = f"ECC memory: {'detected' if ecc_supported else 'not detected'}"
        
        # ECC functionality test (if ECC is supported)
        if ecc_supported:
            try:
                # Check kernel messages for ECC errors
                result = subprocess.run(['dmesg'], capture_output=True, text=True)
                ecc_errors = 'ECC' in result.stdout and 'error' in result.stdout.lower()
                
                if ecc_errors:
                    status = CompatibilityStatus.WARNING
                    message += " (ECC errors detected in system logs)"
                
            except Exception as e:
                self.logger.warning(f"Could not check ECC status: {e}")
        
        return {
            'status': status,
            'message': message,
            'details': {
                'ecc_supported': ecc_supported,
                'memory_type': memory_info.get('memory_type', 'Unknown'),
                'memory_speed_mhz': memory_info.get('speed_mhz', 0)
            }
        }
    
    def test_storage_interface(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test storage interface compatibility"""
        storage_devices = self.current_hardware_profile.get('storage', [])
        
        interface_types = {
            'SATA': ['sata', 'ahci'],
            'NVMe': ['nvme'],
            'USB': ['usb'],
            'Network': ['network', 'iscsi'],
            'Virtual': ['loop', 'dm', 'lvm']
        }
        
        detected_interfaces = []
        supported_interfaces = []
        
        for device in storage_devices:
            device_name = device.get('device', '').lower()
            device_type = device.get('type', '').lower()
            
            # Detect interface type
            for interface, keywords in interface_types.items():
                if any(keyword in device_name or keyword in device_type for keyword in keywords):
                    if interface not in detected_interfaces:
                        detected_interfaces.append(interface)
                        supported_interfaces.append(interface)
        
        status = CompatibilityStatus.COMPATIBLE if supported_interfaces else CompatibilityStatus.WARNING
        message = f"Storage interfaces: {', '.join(supported_interfaces) if supported_interfaces else 'None detected'}"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'detected_interfaces': detected_interfaces,
                'supported_interfaces': supported_interfaces,
                'total_devices': len(storage_devices),
                'devices': storage_devices
            }
        }
    
    def test_storage_performance(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test storage performance"""
        storage_devices = self.current_hardware_profile.get('storage', [])
        
        # Test the root filesystem
        test_results = []
        
        try:
            # Test sequential read/write
            test_file = '/tmp/storage_performance_test.dat'
            test_size = 50 * 1024 * 1024  # 50MB
            
            # Write test
            start_time = time.time()
            subprocess.run(['dd', 'if=/dev/zero', f'of={test_file}', 'bs=1M', 
                          f'count={test_size//(1024*1024)}'], 
                          capture_output=True, timeout=120)
            write_time = time.time() - start_time
            
            # Read test
            start_time = time.time()
            subprocess.run(['dd', f'if={test_file}', 'of=/dev/null', 'bs=1M'], 
                          capture_output=True, timeout=120)
            read_time = time.time() - start_time
            
            # Random I/O test
            start_time = time.time()
            subprocess.run(['dd', f'if={test_file}', 'of=/tmp/test_random.dat', 'bs=4K', 
                          'count=1000'], capture_output=True, timeout=60)
            random_time = time.time() - start_time
            
            # Calculate performance
            write_bandwidth = (test_size / (1024*1024)) / write_time if write_time > 0 else 0
            read_bandwidth = (test_size / (1024*1024)) / read_time if read_time > 0 else 0
            random_bandwidth = (1000 * 4096) / (1024*1024) / random_time if random_time > 0 else 0
            
            # Determine performance level
            status = CompatibilityStatus.COMPATIBLE
            message = f"Storage performance: {read_bandwidth:.1f}MB/s read, {write_bandwidth:.1f}MB/s write"
            
            # Performance classification
            if read_bandwidth < 50:
                status = CompatibilityStatus.WARNING
                message += " (slow)"
            elif read_bandwidth > 500:
                message += " (fast)"
            
            test_results = [{
                'device': 'root_filesystem',
                'read_bandwidth_mbps': read_bandwidth,
                'write_bandwidth_mbps': write_bandwidth,
                'random_bandwidth_mbps': random_bandwidth,
                'test_size_mb': test_size // (1024*1024)
            }]
            
            # Cleanup
            os.remove(test_file)
            try:
                os.remove('/tmp/test_random.dat')
            except:
                pass
                
        except Exception as e:
            return {
                'status': CompatibilityStatus.UNKNOWN,
                'message': f"Storage performance test failed: {str(e)}",
                'details': {'error': str(e)}
            }
        
        return {
            'status': status,
            'message': message,
            'details': {
                'test_results': test_results
            },
            'performance_metrics': test_results[0] if test_results else {}
        }
    
    def test_storage_raid(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test RAID configuration"""
        try:
            # Check for software RAID
            result = subprocess.run(['mdadm', '--detail', '--scan'], capture_output=True, text=True)
            software_raid = result.returncode == 0 and result.stdout.strip()
            
            # Check for hardware RAID
            result = subprocess.run(['lspci'], capture_output=True, text=True)
            hardware_raid = 'RAID' in result.stdout
            
            raid_arrays = []
            
            if software_raid:
                for line in result.stdout.strip().split('\n'):
                    raid_arrays.append({
                        'type': 'software_raid',
                        'configuration': line.strip()
                    })
            
            status = CompatibilityStatus.COMPATIBLE
            message = f"RAID arrays: {len(raid_arrays)} detected"
            
            if not raid_arrays:
                message = "No RAID arrays detected"
            
            return {
                'status': status,
                'message': message,
                'details': {
                    'raid_arrays': raid_arrays,
                    'software_raid_supported': bool(software_raid),
                    'hardware_raid_present': bool(hardware_raid)
                }
            }
            
        except Exception as e:
            return {
                'status': CompatibilityStatus.UNKNOWN,
                'message': f"RAID test failed: {str(e)}",
                'details': {'error': str(e)}
            }
    
    def test_network_interfaces(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test network interface detection"""
        network_interfaces = self.current_hardware_profile.get('network', [])
        
        interface_summary = []
        total_interfaces = len(network_interfaces)
        active_interfaces = 0
        
        for interface in network_interfaces:
            interface_info = {
                'name': interface.get('name'),
                'type': 'Ethernet' if interface.get('is_ethernet') else 'Wireless' if interface.get('is_wireless') else 'Unknown',
                'speed_mbps': interface.get('speed_mbps', 0),
                'status': interface.get('status', 'unknown'),
                'ip_addresses': interface.get('ip_addresses', [])
            }
            interface_summary.append(interface_info)
            
            if interface.get('status') == 'up':
                active_interfaces += 1
        
        status = CompatibilityStatus.COMPATIBLE
        message = f"Network interfaces: {total_interfaces} detected, {active_interfaces} active"
        
        if active_interfaces == 0:
            status = CompatibilityStatus.WARNING
            message += " (no active interfaces)"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'total_interfaces': total_interfaces,
                'active_interfaces': active_interfaces,
                'interfaces': interface_summary
            }
        }
    
    def test_network_performance(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test network performance"""
        network_interfaces = self.current_hardware_profile.get('network', [])
        
        if not network_interfaces:
            return {
                'status': CompatibilityStatus.WARNING,
                'message': "No network interfaces available for testing",
                'details': {}
            }
        
        # Test loopback performance first
        try:
            # Simple network performance test using ping and localhost
            start_time = time.time()
            
            # Test local network stack
            for _ in range(100):
                result = subprocess.run(['ping', '-c', '1', '127.0.0.1'], 
                                      capture_output=True, timeout=5)
                if result.returncode != 0:
                    break
            
            network_test_time = time.time() - start_time
            
            status = CompatibilityStatus.COMPATIBLE
            message = f"Network stack test: {network_test_time:.2f}s for 100 iterations"
            
            return {
                'status': status,
                'message': message,
                'details': {
                    'test_time_seconds': network_test_time,
                    'iterations': 100,
                    'avg_ping_time_ms': (network_test_time * 1000) / 100
                },
                'performance_metrics': {
                    'network_test_time': network_test_time
                }
            }
            
        except Exception as e:
            return {
                'status': CompatibilityStatus.UNKNOWN,
                'message': f"Network performance test failed: {str(e)}",
                'details': {'error': str(e)}
            }
    
    def test_gpu_detection(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test GPU detection"""
        gpu_devices = self.current_hardware_profile.get('gpu', [])
        
        gpu_summary = []
        total_gpus = len(gpu_devices)
        
        for gpu in gpu_devices:
            gpu_info = {
                'name': gpu.get('name', 'Unknown'),
                'vendor': gpu.get('vendor', 'Unknown'),
                'memory_mb': gpu.get('memory_mb', 0),
                'driver_version': gpu.get('driver_version', 'Unknown')
            }
            gpu_summary.append(gpu_info)
        
        status = CompatibilityStatus.COMPATIBLE if total_gpus > 0 else CompatibilityStatus.WARNING
        message = f"GPU devices: {total_gpus} detected"
        
        if total_gpus == 0:
            message += " (no discrete GPU detected)"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'total_gpus': total_gpus,
                'gpu_devices': gpu_summary
            }
        }
    
    def test_gpu_drivers(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test GPU driver compatibility"""
        gpu_devices = self.current_hardware_profile.get('gpu', [])
        driver_status = {}
        
        for gpu in gpu_devices:
            gpu_name = gpu.get('name', '')
            vendor = gpu.get('vendor', '')
            
            driver_info = {
                'installed': False,
                'working': False,
                'version': 'Unknown',
                'issues': []
            }
            
            try:
                if 'NVIDIA' in vendor:
                    # Test NVIDIA drivers
                    result = subprocess.run(['nvidia-smi'], capture_output=True, text=True, timeout=10)
                    if result.returncode == 0:
                        driver_info['installed'] = True
                        driver_info['working'] = True
                        
                        # Extract driver version
                        for line in result.stdout.split('\n'):
                            if 'Driver Version' in line:
                                driver_info['version'] = line.split(':')[1].strip()
                                break
                    else:
                        driver_info['installed'] = False
                        driver_info['issues'].append('NVIDIA driver not functional')
                        
                elif vendor in ['AMD', 'Intel']:
                    # Test generic GPU drivers
                    driver_info['installed'] = True  # Assume basic drivers are present
                    driver_info['working'] = True
                    driver_info['version'] = 'kernel_builtin'
                    
            except Exception as e:
                driver_info['installed'] = False
                driver_info['issues'].append(f'Driver test failed: {str(e)}')
            
            driver_status[gpu_name] = driver_info
        
        # Overall status
        working_drivers = sum(1 for info in driver_status.values() if info['working'])
        status = CompatibilityStatus.COMPATIBLE if working_drivers > 0 else CompatibilityStatus.NEEDS_DRIVER
        message = f"GPU drivers: {working_drivers}/{len(gpu_devices)} functional"
        
        if working_drivers == 0:
            message += " (GPU drivers need installation)"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'driver_status': driver_status
            }
        }
    
    def test_gpu_performance(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test GPU performance"""
        gpu_devices = self.current_hardware_profile.get('gpu', [])
        
        if not gpu_devices:
            return {
                'status': CompatibilityStatus.WARNING,
                'message': "No GPU devices available for performance testing",
                'details': {}
            }
        
        performance_results = []
        
        for gpu in gpu_devices:
            gpu_name = gpu.get('name', '')
            vendor = gpu.get('vendor', '')
            
            try:
                if 'NVIDIA' in vendor:
                    # Test NVIDIA GPU
                    result = subprocess.run([
                        'nvidia-smi', '--query-gpu=name,temperature.gpu,power.draw,utilization.gpu',
                        '--format=csv,noheader,nounits'
                    ], capture_output=True, text=True, timeout=30)
                    
                    if result.returncode == 0:
                        parts = result.stdout.strip().split(', ')
                        if len(parts) >= 4:
                            performance_results.append({
                                'gpu': gpu_name,
                                'temperature_c': parts[1] if parts[1] != '[Not Supported]' else 0,
                                'power_draw_w': parts[2] if parts[2] != '[Not Supported]' else 0.0,
                                'utilization_percent': parts[3] if parts[3] != '[Not Supported]' else 0
                            })
                    
                    # Simple compute test if CUDA is available
                    try:
                        result = subprocess.run([
                            'nvidia-smi', '--query-compute-apps=pid,process_name,used_memory',
                            '--format=csv,noheader,nounits'
                        ], capture_output=True, text=True, timeout=10)
                        
                        if result.returncode == 0 and result.stdout.strip():
                            performance_results[0]['compute_apps_running'] = len(result.stdout.strip().split('\n'))
                    except:
                        pass
                        
            except Exception as e:
                self.logger.warning(f"GPU performance test failed for {gpu_name}: {e}")
        
        status = CompatibilityStatus.COMPATIBLE
        message = f"GPU performance tested for {len(performance_results)} device(s)"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'performance_results': performance_results
            },
            'performance_metrics': performance_results[0] if performance_results else {}
        }
    
    def test_usb_detection(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test USB device detection"""
        usb_devices = self.current_hardware_profile.get('usb_devices', [])
        
        usb_categories = {
            'storage': ['flash', 'disk', 'drive'],
            'input': ['keyboard', 'mouse', 'joystick'],
            'audio': ['audio', 'sound', 'microphone'],
            'network': ['network', 'ethernet'],
            'other': []
        }
        
        categorized_devices = {category: [] for category in usb_categories}
        
        for device in usb_devices:
            device_name = device.get('name', '').lower()
            categorized = False
            
            for category, keywords in usb_categories.items():
                if any(keyword in device_name for keyword in keywords):
                    categorized_devices[category].append(device)
                    categorized = True
                    break
            
            if not categorized:
                categorized_devices['other'].append(device)
        
        total_usb_devices = len(usb_devices)
        status = CompatibilityStatus.COMPATIBLE
        message = f"USB devices: {total_usb_devices} detected"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'total_usb_devices': total_usb_devices,
                'categorized_devices': {k: len(v) for k, v in categorized_devices.items()},
                'devices': usb_devices
            }
        }
    
    def test_usb_performance(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test USB performance"""
        # This would require USB device testing which may not be safe
        # Return a placeholder test result
        return {
            'status': CompatibilityStatus.WARNING,
            'message': "USB performance testing requires manual verification for safety",
            'details': {
                'note': 'USB performance testing disabled for safety'
            }
        }
    
    def test_system_power(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test system power management"""
        power_features = {
            'acpi': False,
            'battery': False,
            'sleep_states': [],
            'cpu_governors': [],
            'thermal_zones': []
        }
        
        try:
            # Check ACPI
            result = subprocess.run(['which', 'acpi'], capture_output=True)
            power_features['acpi'] = result.returncode == 0
            
            # Check battery
            if os.path.exists('/sys/class/power_supply/BAT0'):
                power_features['battery'] = True
            
            # Check CPU governors
            governor_files = glob.glob('/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor')
            governors = set()
            for gov_file in governor_files:
                try:
                    with open(gov_file, 'r') as f:
                        governors.add(f.read().strip())
                except:
                    pass
            power_features['cpu_governors'] = list(governors)
            
            # Check thermal zones
            thermal_zones = glob.glob('/sys/class/thermal/thermal_zone*')
            power_features['thermal_zones'] = len(thermal_zones)
            
        except Exception as e:
            self.logger.warning(f"Power management test error: {e}")
        
        status = CompatibilityStatus.COMPATIBLE
        features_count = sum(1 for v in power_features.values() if v and (isinstance(v, list) and v or isinstance(v, (bool, int)) and v))
        message = f"Power management: {features_count}/5 features detected"
        
        if features_count < 3:
            status = CompatibilityStatus.WARNING
            message += " (limited power management)"
        
        return {
            'status': status,
            'message': message,
            'details': power_features
        }
    
    def test_system_thermal(self, test: CompatibilityTest) -> Dict[str, Any]:
        """Test thermal management"""
        thermal_sensors = self.current_hardware_profile.get('system', {}).get('temperature_sensors', [])
        
        if not thermal_sensors:
            return {
                'status': CompatibilityStatus.WARNING,
                'message": "No thermal sensors detected',
                'details': {}
            }
        
        temperature_readings = []
        high_temperatures = []
        
        for sensor in thermal_sensors:
            temp = sensor.get('temperature_c', 0)
            sensor_name = sensor.get('name', 'Unknown')
            
            temperature_readings.append({
                'name': sensor_name,
                'temperature_c': temp
            })
            
            if temp > 80:
                high_temperatures.append({
                    'sensor': sensor_name,
                    'temperature_c': temp
                })
        
        status = CompatibilityStatus.COMPATIBLE
        message = f"Thermal sensors: {len(thermal_sensors)} detected"
        
        if high_temperatures:
            status = CompatibilityStatus.WARNING
            message += f" ({len(high_temperatures)} sensors reporting high temperature)"
        
        return {
            'status': status,
            'message': message,
            'details': {
                'total_sensors': len(thermal_sensors),
                'temperature_readings': temperature_readings,
                'high_temperature_sensors': high_temperatures
            },
            'performance_metrics': {
                'max_temperature': max(t['temperature_c'] for t in temperature_readings) if temperature_readings else 0
            }
        }
    
    def run_compatibility_suite(self, categories: List[str] = None) -> List[CompatibilityResult]:
        """Run complete compatibility test suite"""
        self.logger.info("Starting hardware compatibility test suite")
        
        # Detect hardware if not already done
        if not self.current_hardware_profile:
            self.detect_current_hardware()
        
        # Filter tests by categories if specified
        tests_to_run = self.test_database
        if categories:
            tests_to_run = [test for test in self.test_database if test.category in categories]
        
        # Run tests in dependency order
        results = []
        completed_tests = set()
        
        for test in tests_to_run:
            # Check dependencies
            if not all(dep in completed_tests for dep in test.dependencies):
                self.logger.warning(f"Skipping test {test.test_id} due to unmet dependencies")
                continue
            
            try:
                result = self.run_compatibility_test(test)
                results.append(result)
                completed_tests.add(test.test_id)
                
                # Log result
                status_emoji = {
                    CompatibilityStatus.COMPATIBLE: "✓",
                    CompatibilityStatus.INCOMPATIBLE: "✗",
                    CompatibilityStatus.PARTIAL: "⚠",
                    CompatibilityStatus.NEEDS_DRIVER: "⊘",
                    CompatibilityStatus.WARNING: "⚠",
                    CompatibilityStatus.UNKNOWN: "?"
                }
                
                self.logger.info(f"{status_emoji.get(result.status, '?')} {result.message}")
                
            except Exception as e:
                self.logger.error(f"Failed to run test {test.test_id}: {e}")
                results.append(CompatibilityResult(
                    test_id=test.test_id,
                    status=CompatibilityStatus.UNKNOWN,
                    message=f"Test execution failed: {str(e)}",
                    details={},
                    execution_time=0,
                    timestamp=time.time()
                ))
        
        self.results = results
        self.logger.info(f"Compatibility testing completed. {len(results)} tests executed.")
        
        return results
    
    def generate_compatibility_report(self) -> str:
        """Generate comprehensive compatibility report"""
        if not self.results:
            self.run_compatibility_suite()
        
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'test_suite_version': '1.0',
                'hardware_profile': self.current_hardware_profile
            },
            'summary': {
                'total_tests': len(self.results),
                'passed': len([r for r in self.results if r.status == CompatibilityStatus.COMPATIBLE]),
                'failed': len([r for r in self.results if r.status == CompatibilityStatus.INCOMPATIBLE]),
                'warnings': len([r for r in self.results if r.status in [CompatibilityStatus.WARNING, CompatibilityStatus.PARTIAL]]),
                'needs_driver': len([r for r in self.results if r.status == CompatibilityStatus.NEEDS_DRIVER]),
                'unknown': len([r for r in self.results if r.status == CompatibilityStatus.UNKNOWN])
            },
            'detailed_results': [asdict(result) for result in self.results],
            'recommendations': self._generate_compatibility_recommendations()
        }
        
        report_path = f"/workspace/testing/hardware_tests/results/compatibility_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Compatibility report generated: {report_path}")
        return report_path
    
    def _generate_compatibility_recommendations(self) -> Dict[str, List[str]]:
        """Generate recommendations based on compatibility results"""
        recommendations = {
            'critical_issues': [],
            'warnings': [],
            'optimizations': [],
            'driver_recommendations': [],
            'hardware_upgrades': []
        }
        
        for result in self.results:
            if result.status == CompatibilityStatus.INCOMPATIBLE:
                recommendations['critical_issues'].append(f"{result.test_id}: {result.message}")
            elif result.status == CompatibilityStatus.WARNING:
                recommendations['warnings'].append(f"{result.test_id}: {result.message}")
            elif result.status == CompatibilityStatus.NEEDS_DRIVER:
                recommendations['driver_recommendations'].append(f"{result.test_id}: {result.message}")
        
        # Add optimization recommendations based on hardware
        cpu_info = self.current_hardware_profile.get('cpu', {})
        if cpu_info.get('cores_logical', 1) > 8:
            recommendations['optimizations'].append("Consider enabling CPU multi-threading optimizations")
        
        memory_info = self.current_hardware_profile.get('memory', {})
        if memory_info.get('total_gb', 0) > 16:
            recommendations['optimizations'].append("System has sufficient memory for memory-intensive workloads")
        
        return recommendations


def main():
    """Main function for standalone execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Hardware Compatibility Testing')
    parser.add_argument('--categories', nargs='+', 
                       choices=['cpu', 'memory', 'storage', 'network', 'gpu', 'usb', 'system'],
                       help='Test categories to run')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    tester = HardwareCompatibilityTester()
    
    # Run tests
    results = tester.run_compatibility_suite(args.categories)
    
    # Generate report
    report_path = tester.generate_compatibility_report()
    print(f"Compatibility report generated: {report_path}")
    
    # Print summary
    summary = {
        'Total Tests': len(results),
        'Compatible': len([r for r in results if r.status == CompatibilityStatus.COMPATIBLE]),
        'Incompatible': len([r for r in results if r.status == CompatibilityStatus.INCOMPATIBLE]),
        'Warnings': len([r for r in results if r.status in [CompatibilityStatus.WARNING, CompatibilityStatus.PARTIAL]]),
        'Needs Driver': len([r for r in results if r.status == CompatibilityStatus.NEEDS_DRIVER])
    }
    
    print("\nCompatibility Test Summary:")
    for key, value in summary.items():
        print(f"  {key}: {value}")


if __name__ == "__main__":
    main()