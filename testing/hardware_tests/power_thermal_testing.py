#!/usr/bin/env python3
"""
Power Management and Thermal Testing Framework
Advanced power consumption analysis, thermal monitoring, and energy efficiency testing
"""

import os
import sys
import json
import time
import threading
import logging
import subprocess
import psutil
import statistics
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple, Callable
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import glob
import re

@dataclass
class PowerMeasurement:
    """Power consumption measurement"""
    timestamp: float
    cpu_power_w: float
    gpu_power_w: Optional[float]
    total_power_w: float
    voltage_v: Optional[float]
    current_a: Optional[float]
    energy_consumed_j: float
    cpu_temperature_c: float
    gpu_temperature_c: Optional[float]
    ambient_temperature_c: Optional[float]

@dataclass
class ThermalZone:
    """Thermal zone information"""
    name: str
    type: str
    temperature_c: float
    critical_temperature_c: Optional[float]
    passive_temperature_c: Optional[float]
    active_temperature_c: Optional[float]

@dataclass
class PowerTestConfig:
    """Configuration for power testing"""
    duration_minutes: int = 60
    sampling_interval_seconds: int = 5
    stress_cpu: bool = True
    stress_memory: bool = True
    stress_gpu: bool = True
    stress_storage: bool = True
    thermal_warning_threshold: int = 80
    thermal_critical_threshold: int = 90
    power_monitoring_available: bool = True
    gpu_monitoring_available: bool = True

class PowerThermalTester:
    """Main class for power and thermal testing"""
    
    def __init__(self, config: PowerTestConfig = None):
        self.config = config or PowerTestConfig()
        self.logger = self._setup_logging()
        self.power_measurements = []
        self.thermal_zones = []
        self.is_monitoring = False
        self.monitoring_thread = None
        self.energy_baseline_j = 0
        self.test_start_time = None
        
    def _setup_logging(self):
        """Setup logging for power/thermal testing"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/power_thermal_test.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def detect_thermal_sensors(self) -> List[ThermalZone]:
        """Detect available thermal sensors"""
        thermal_zones = []
        
        try:
            # Read thermal zones from sysfs
            thermal_files = glob.glob('/sys/class/thermal/thermal_zone*')
            
            for thermal_file in thermal_files:
                zone_path = Path(thermal_file)
                zone_name = zone_path.name
                
                try:
                    # Get zone type
                    type_file = zone_path / 'type'
                    with open(type_file, 'r') as f:
                        zone_type = f.read().strip()
                    
                    # Get temperature
                    temp_file = zone_path / 'temp'
                    with open(temp_file, 'r') as f:
                        temp_millidegrees = int(f.read().strip())
                        temperature_c = temp_millidegrees / 1000.0
                    
                    # Get trip points
                    critical_temp = None
                    passive_temp = None
                    active_temps = []
                    
                    trip_files = glob.glob(f'{thermal_file}/trip_point_*_temp')
                    for trip_file in trip_files:
                        trip_type_file = trip_file.replace('_temp', '_type')
                        
                        with open(trip_file, 'r') as f:
                            trip_temp_millidegrees = int(f.read().strip())
                            trip_temp_c = trip_temp_millidegrees / 1000.0
                        
                        with open(trip_type_file, 'r') as f:
                            trip_type = f.read().strip()
                        
                        if trip_type == 'critical':
                            critical_temp = trip_temp_c
                        elif trip_type == 'passive':
                            passive_temp = trip_temp_c
                        elif trip_type == 'active':
                            active_temps.append(trip_temp_c)
                    
                    thermal_zone = ThermalZone(
                        name=zone_name,
                        type=zone_type,
                        temperature_c=temperature_c,
                        critical_temperature_c=critical_temp,
                        passive_temperature_c=passive_temp,
                        active_temperature_c=active_temps[0] if active_temps else None
                    )
                    
                    thermal_zones.append(thermal_zone)
                    
                except Exception as e:
                    self.logger.warning(f"Error reading thermal zone {zone_name}: {e}")
                    
        except Exception as e:
            self.logger.error(f"Error detecting thermal sensors: {e}")
        
        # Also try lm-sensors if available
        try:
            result = subprocess.run(['sensors'], capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                self.logger.info("lm-sensors data available")
                # This could be parsed for additional sensor information
        except:
            pass
        
        self.thermal_zones = thermal_zones
        return thermal_zones
    
    def get_power_consumption(self) -> Dict[str, float]:
        """Get current power consumption"""
        power_data = {
            'cpu_power_w': 0.0,
            'gpu_power_w': None,
            'total_power_w': 0.0,
            'voltage_v': None,
            'current_a': None
        }
        
        # Try to get GPU power consumption
        try:
            result = subprocess.run(['nvidia-smi', '--query-gpu=power.draw', 
                                   '--format=csv,noheader,nounits'], 
                                  capture_output=True, text=True, timeout=5)
            if result.returncode == 0 and result.stdout.strip():
                power_data['gpu_power_w'] = float(result.stdout.strip())
        except:
            pass
        
        # Try to get system power consumption from various sources
        try:
            # Check for power supply sensors
            power_supply_files = glob.glob('/sys/class/power_supply/BAT*/power_now')
            if power_supply_files:
                with open(power_supply_files[0], 'r') as f:
                    power_now_uw = int(f.read().strip())
                    power_data['total_power_w'] = power_now_uw / 1000000.0
            else:
                # Estimate based on CPU usage and system info
                cpu_power = self._estimate_cpu_power()
                total_power = cpu_power + (power_data.get('gpu_power_w', 0) or 0)
                power_data['total_power_w'] = total_power
                
        except Exception as e:
            self.logger.warning(f"Could not get power consumption: {e}")
            # Fallback to estimation
            cpu_power = self._estimate_cpu_power()
            power_data['total_power_w'] = cpu_power
        
        return power_data
    
    def _estimate_cpu_power(self) -> float:
        """Estimate CPU power consumption based on usage and frequency"""
        try:
            cpu_usage = psutil.cpu_percent(interval=1)
            cpu_freq = psutil.cpu_freq()
            
            if not cpu_freq:
                return 15.0  # Default estimate
            
            # Base power consumption (watts)
            base_power = 10.0
            
            # Add power based on usage (10-15W at 100% usage)
            usage_power = (cpu_usage / 100.0) * 15.0
            
            # Add power based on frequency (modern CPUs ~1-2W per GHz)
            if cpu_freq.max > 0:
                freq_power = (cpu_freq.current / cpu_freq.max) * 5.0
            else:
                freq_power = 0
            
            return base_power + usage_power + freq_power
            
        except Exception:
            return 15.0  # Default estimate
    
    def get_temperatures(self) -> Dict[str, float]:
        """Get current temperature readings"""
        temps = {
            'cpu_temperature_c': 0.0,
            'gpu_temperature_c': None,
            'max_thermal_zone_c': 0.0
        }
        
        try:
            # Get CPU temperature
            temps['cpu_temperature_c'] = self._get_cpu_temperature()
            
            # Get GPU temperature
            try:
                result = subprocess.run(['nvidia-smi', '--query-gpu=temperature.gpu', 
                                       '--format=csv,noheader,nounits'], 
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0 and result.stdout.strip():
                    temps['gpu_temperature_c'] = float(result.stdout.strip())
            except:
                pass
            
            # Get max temperature from thermal zones
            if self.thermal_zones:
                temps['max_thermal_zone_c'] = max(tz.temperature_c for tz in self.thermal_zones)
            
        except Exception as e:
            self.logger.warning(f"Could not get temperature readings: {e}")
        
        return temps
    
    def _get_cpu_temperature(self) -> float:
        """Get CPU temperature"""
        try:
            # Try reading from /sys/class/thermal
            thermal_files = glob.glob('/sys/class/thermal/thermal_zone*/temp')
            if thermal_files:
                temps = []
                for temp_file in thermal_files:
                    try:
                        with open(temp_file, 'r') as f:
                            temp_millidegrees = int(f.read().strip())
                            temps.append(temp_millidegrees / 1000.0)
                    except:
                        continue
                if temps:
                    return max(temps)
            
            # Try psutil (requires lm-sensors on some systems)
            try:
                temps = psutil.sensors_temperatures()
                if temps:
                    # Get max temperature from all sensors
                    all_temps = []
                    for sensor_name, sensor_list in temps.items():
                        for sensor in sensor_list:
                            all_temps.append(sensor.current)
                    if all_temps:
                        return max(all_temps)
            except:
                pass
            
            return 0.0
            
        except Exception:
            return 0.0
    
    def collect_power_measurement(self) -> PowerMeasurement:
        """Collect a single power measurement"""
        timestamp = time.time()
        power_data = self.get_power_consumption()
        temp_data = self.get_temperatures()
        
        # Calculate cumulative energy
        if self.power_measurements:
            last_measurement = self.power_measurements[-1]
            time_delta = timestamp - last_measurement.timestamp
            avg_power = (last_measurement.total_power_w + power_data['total_power_w']) / 2
            energy_increment = avg_power * time_delta
            energy_consumed_j = last_measurement.energy_consumed_j + energy_increment
        else:
            energy_consumed_j = 0
        
        measurement = PowerMeasurement(
            timestamp=timestamp,
            cpu_power_w=power_data['cpu_power_w'],
            gpu_power_w=power_data.get('gpu_power_w'),
            total_power_w=power_data['total_power_w'],
            voltage_v=power_data.get('voltage_v'),
            current_a=power_data.get('current_a'),
            energy_consumed_j=energy_consumed_j,
            cpu_temperature_c=temp_data['cpu_temperature_c'],
            gpu_temperature_c=temp_data.get('gpu_temperature_c'),
            ambient_temperature_c=None  # Would need external sensor
        )
        
        return measurement
    
    def start_power_monitoring(self):
        """Start continuous power monitoring"""
        if self.is_monitoring:
            return
        
        self.is_monitoring = True
        self.monitoring_thread = threading.Thread(target=self._monitoring_loop)
        self.monitoring_thread.daemon = True
        self.monitoring_thread.start()
        self.logger.info("Started power monitoring")
    
    def stop_power_monitoring(self):
        """Stop power monitoring"""
        self.is_monitoring = False
        if self.monitoring_thread:
            self.monitoring_thread.join(timeout=10)
        self.logger.info("Stopped power monitoring")
    
    def _monitoring_loop(self):
        """Main monitoring loop"""
        self.test_start_time = time.time()
        
        while self.is_monitoring:
            try:
                measurement = self.collect_power_measurement()
                self.power_measurements.append(measurement)
                
                # Check for thermal warnings
                if measurement.cpu_temperature_c > self.config.thermal_critical_threshold:
                    self.logger.error(f"Critical CPU temperature: {measurement.cpu_temperature_c:.1f}°C")
                elif measurement.cpu_temperature_c > self.config.thermal_warning_threshold:
                    self.logger.warning(f"High CPU temperature: {measurement.cpu_temperature_c:.1f}°C")
                
                if measurement.gpu_temperature_c and measurement.gpu_temperature_c > self.config.thermal_critical_threshold:
                    self.logger.error(f"Critical GPU temperature: {measurement.gpu_temperature_c:.1f}°C")
                elif measurement.gpu_temperature_c and measurement.gpu_temperature_c > self.config.thermal_warning_threshold:
                    self.logger.warning(f"High GPU temperature: {measurement.gpu_temperature_c:.1f}°C")
                
                # Keep measurements from last 1000 samples
                if len(self.power_measurements) > 1000:
                    self.power_measurements = self.power_measurements[-1000:]
                
                time.sleep(self.config.sampling_interval_seconds)
                
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                time.sleep(self.config.sampling_interval_seconds)
    
    def run_power_stress_test(self, duration_minutes: int = None) -> Dict[str, Any]:
        """Run power stress test"""
        if not duration_minutes:
            duration_minutes = self.config.duration_minutes
        
        self.logger.info(f"Starting power stress test for {duration_minutes} minutes")
        
        # Start monitoring
        self.start_power_monitoring()
        
        # Run stress tests
        stress_threads = []
        
        if self.config.stress_cpu:
            cpu_thread = threading.Thread(target=self._cpu_stress_workload, 
                                         args=(duration_minutes * 60,))
            cpu_thread.start()
            stress_threads.append(cpu_thread)
        
        if self.config.stress_memory:
            mem_thread = threading.Thread(target=self._memory_stress_workload, 
                                         args=(duration_minutes * 60,))
            mem_thread.start()
            stress_threads.append(mem_thread)
        
        if self.config.stress_gpu and self.config.gpu_monitoring_available:
            gpu_thread = threading.Thread(target=self._gpu_stress_workload, 
                                         args=(duration_minutes * 60,))
            gpu_thread.start()
            stress_threads.append(gpu_thread)
        
        if self.config.stress_storage:
            storage_thread = threading.Thread(target=self._storage_stress_workload, 
                                             args=(duration_minutes * 60,))
            storage_thread.start()
            stress_threads.append(storage_thread)
        
        # Wait for stress tests to complete
        for thread in stress_threads:
            thread.join()
        
        # Stop monitoring
        self.stop_power_monitoring()
        
        # Calculate results
        results = self._calculate_power_test_results()
        results['test_config'] = asdict(self.config)
        results['test_duration_minutes'] = duration_minutes
        
        return results
    
    def _cpu_stress_workload(self, duration_seconds: int):
        """CPU stress workload"""
        end_time = time.time() + duration_seconds
        while time.time() < end_time:
            # CPU-intensive calculation
            result = sum(range(100000))
            # Brief pause to prevent overheating
            time.sleep(0.1)
    
    def _memory_stress_workload(self, duration_seconds: int):
        """Memory stress workload"""
        end_time = time.time() + duration_seconds
        memory_hog = []
        try:
            while time.time() < end_time:
                # Allocate memory
                memory_hog.append(bytearray(1024 * 1024))  # 1MB chunks
                time.sleep(0.1)
                
                # Limit memory usage
                if len(memory_hog) > 100:  # 100MB limit
                    memory_hog = memory_hog[-50:]  # Keep only last 50MB
        except MemoryError:
            pass
        finally:
            # Clean up
            memory_hog.clear()
    
    def _gpu_stress_workload(self, duration_seconds: int):
        """GPU stress workload"""
        try:
            # Simple GPU compute stress using nvidia-smi
            end_time = time.time() + duration_seconds
            while time.time() < end_time:
                # Check GPU status periodically
                subprocess.run(['nvidia-smi', '--query-gpu=name', '--format=csv,noheader'], 
                             capture_output=True, timeout=5)
                time.sleep(5)  # Check every 5 seconds
        except:
            pass
    
    def _storage_stress_workload(self, duration_seconds: int):
        """Storage stress workload"""
        test_file = '/tmp/power_stress_test.tmp'
        try:
            end_time = time.time() + duration_seconds
            while time.time() < end_time:
                # Write stress
                with open(test_file, 'w') as f:
                    f.write('x' * (1024 * 1024))  # 1MB writes
                
                # Read stress
                with open(test_file, 'r') as f:
                    f.read()
                
                time.sleep(0.5)
        except Exception as e:
            self.logger.warning(f"Storage stress test error: {e}")
        finally:
            try:
                os.remove(test_file)
            except:
                pass
    
    def _calculate_power_test_results(self) -> Dict[str, Any]:
        """Calculate power test results"""
        if not self.power_measurements:
            return {'error': 'No power measurements collected'}
        
        # Extract data
        total_power = [m.total_power_w for m in self.power_measurements]
        cpu_power = [m.cpu_power_w for m in self.power_measurements]
        gpu_power = [m.gpu_power_w for m in self.power_measurements if m.gpu_power_w is not None]
        cpu_temps = [m.cpu_temperature_c for m in self.power_measurements]
        gpu_temps = [m.gpu_temperature_c for m in self.power_measurements if m.gpu_temperature_c is not None]
        energy_consumed = [m.energy_consumed_j for m in self.power_measurements]
        
        # Calculate statistics
        results = {
            'test_info': {
                'test_duration_seconds': self.power_measurements[-1].timestamp - self.power_measurements[0].timestamp,
                'measurements_count': len(self.power_measurements),
                'sampling_interval_seconds': self.config.sampling_interval_seconds
            },
            'power_consumption': {
                'avg_total_power_w': statistics.mean(total_power),
                'max_total_power_w': max(total_power),
                'min_total_power_w': min(total_power),
                'std_total_power_w': statistics.stdev(total_power) if len(total_power) > 1 else 0
            },
            'cpu_power': {
                'avg_cpu_power_w': statistics.mean(cpu_power),
                'max_cpu_power_w': max(cpu_power),
                'min_cpu_power_w': min(cpu_power)
            },
            'temperatures': {
                'avg_cpu_temp_c': statistics.mean(cpu_temps),
                'max_cpu_temp_c': max(cpu_temps),
                'min_cpu_temp_c': min(cpu_temps)
            },
            'energy': {
                'total_energy_consumed_j': energy_consumed[-1] if energy_consumed else 0,
                'total_energy_consumed_kwh': (energy_consumed[-1] / 3600000) if energy_consumed else 0
            }
        }
        
        # GPU specific results
        if gpu_power:
            results['gpu_power'] = {
                'avg_gpu_power_w': statistics.mean(gpu_power),
                'max_gpu_power_w': max(gpu_power),
                'min_gpu_power_w': min(gpu_power)
            }
        
        if gpu_temps:
            results['gpu_temperatures'] = {
                'avg_gpu_temp_c': statistics.mean(gpu_temps),
                'max_gpu_temp_c': max(gpu_temps),
                'min_gpu_temp_c': min(gpu_temps)
            }
        
        # Thermal analysis
        results['thermal_analysis'] = self._analyze_thermal_data()
        
        # Power efficiency metrics
        results['efficiency_metrics'] = self._calculate_efficiency_metrics()
        
        return results
    
    def _analyze_thermal_data(self) -> Dict[str, Any]:
        """Analyze thermal performance data"""
        if not self.power_measurements:
            return {}
        
        cpu_temps = [m.cpu_temperature_c for m in self.power_measurements]
        max_temp = max(cpu_temps)
        
        # Thermal performance classification
        if max_temp >= self.config.thermal_critical_threshold:
            thermal_state = "critical"
        elif max_temp >= self.config.thermal_warning_threshold:
            thermal_state = "warning"
        elif max_temp >= 70:
            thermal_state = "elevated"
        else:
            thermal_state = "normal"
        
        return {
            'thermal_state': thermal_state,
            'max_temperature_c': max_temp,
            'thermal_margin_c': 100 - max_temp,  # Assume 100°C is absolute limit
            'thermal_throttling_detected': any(m.cpu_temperature_c >= self.config.thermal_warning_threshold 
                                              for m in self.power_measurements),
            'temperature_stability': statistics.stdev(cpu_temps) if len(cpu_temps) > 1 else 0
        }
    
    def _calculate_efficiency_metrics(self) -> Dict[str, Any]:
        """Calculate power efficiency metrics"""
        if not self.power_measurements:
            return {}
        
        # Calculate performance per watt
        total_energy_j = self.power_measurements[-1].energy_consumed_j
        test_duration_s = self.power_measurements[-1].timestamp - self.power_measurements[0].timestamp
        
        # Get average CPU usage during test
        avg_cpu_usage = sum(psutil.cpu_percent(interval=None) for _ in range(10)) / 10
        
        metrics = {
            'average_power_w': statistics.mean([m.total_power_w for m in self.power_measurements]),
            'energy_per_second_j': total_energy_j / test_duration_s if test_duration_s > 0 else 0,
            'efficiency_score': 100 / (statistics.mean([m.total_power_w for m in self.power_measurements]) + 1)
        }
        
        # Power quality metrics
        power_values = [m.total_power_w for m in self.power_measurements]
        if len(power_values) > 1:
            power_stability = 1 - (statistics.stdev(power_values) / statistics.mean(power_values))
            metrics['power_stability'] = max(0, power_stability)
        
        return metrics
    
    def run_thermal_cycling_test(self, cycles: int = 5, cooldown_minutes: int = 5) -> Dict[str, Any]:
        """Run thermal cycling test"""
        self.logger.info(f"Starting thermal cycling test: {cycles} cycles")
        
        cycle_results = []
        
        for cycle in range(cycles):
            self.logger.info(f"Thermal cycle {cycle + 1}/{cycles}")
            
            # Heat phase
            heat_start = time.time()
            self._cpu_stress_workload(10 * 60)  # 10 minutes heat
            heat_duration = time.time() - heat_start
            
            # Measure peak temperature
            peak_temp = 0
            for _ in range(30):  # Monitor for 2.5 minutes
                measurement = self.collect_power_measurement()
                peak_temp = max(peak_temp, measurement.cpu_temperature_c)
                time.sleep(5)
            
            # Cool down phase
            self.logger.info("Cooling down...")
            time.sleep(cooldown_minutes * 60)
            
            # Measure minimum temperature
            min_temp = 100
            for _ in range(30):  # Monitor for 2.5 minutes
                measurement = self.collect_power_measurement()
                min_temp = min(min_temp, measurement.cpu_temperature_c)
                time.sleep(5)
            
            cycle_result = {
                'cycle_number': cycle + 1,
                'peak_temperature_c': peak_temp,
                'min_temperature_c': min_temp,
                'temperature_swing_c': peak_temp - min_temp,
                'heat_duration_s': heat_duration
            }
            cycle_results.append(cycle_result)
            
            self.logger.info(f"Cycle {cycle + 1}: Peak {peak_temp:.1f}°C, Min {min_temp:.1f}°C, Swing {peak_temp - min_temp:.1f}°C")
        
        # Analyze thermal cycling performance
        avg_swing = statistics.mean([r['temperature_swing_c'] for r in cycle_results])
        max_peak = max([r['peak_temperature_c'] for r in cycle_results])
        
        analysis = {
            'total_cycles': cycles,
            'average_temperature_swing_c': avg_swing,
            'maximum_peak_temperature_c': max_peak,
            'thermal_stability': 1 - (statistics.stdev([r['temperature_swing_c'] for r in cycle_results]) / avg_swing) if avg_swing > 0 else 0,
            'thermal_cycling_results': cycle_results
        }
        
        return analysis
    
    def run_idle_power_test(self, duration_minutes: int = 30) -> Dict[str, Any]:
        """Run idle power consumption test"""
        self.logger.info(f"Starting idle power test for {duration_minutes} minutes")
        
        # Stop any existing monitoring
        self.stop_power_monitoring()
        
        # Clear previous measurements
        self.power_measurements = []
        
        # Start monitoring
        self.start_power_monitoring()
        
        # Wait for test duration
        time.sleep(duration_minutes * 60)
        
        # Stop monitoring
        self.stop_power_monitoring()
        
        # Calculate idle power results
        if not self.power_measurements:
            return {'error': 'No idle power measurements collected'}
        
        # Filter out initial measurements to get steady state
        steady_state_measurements = self.power_measurements[len(self.power_measurements)//4:]
        
        idle_power_values = [m.total_power_w for m in steady_state_measurements]
        
        return {
            'test_duration_minutes': duration_minutes,
            'measurements_count': len(self.power_measurements),
            'steady_state_measurements': len(steady_state_measurements),
            'idle_power_consumption': {
                'average_w': statistics.mean(idle_power_values),
                'minimum_w': min(idle_power_values),
                'maximum_w': max(idle_power_values),
                'standard_deviation_w': statistics.stdev(idle_power_values) if len(idle_power_values) > 1 else 0
            }
        }
    
    def generate_power_thermal_report(self) -> str:
        """Generate comprehensive power and thermal report"""
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'test_configuration': asdict(self.config),
                'thermal_zones_detected': len(self.thermal_zones)
            },
            'thermal_sensors': [asdict(tz) for tz in self.thermal_zones],
            'power_measurements': [asdict(m) for m in self.power_measurements],
            'analysis': self._calculate_power_test_results()
        }
        
        report_path = f"/workspace/testing/hardware_tests/results/power_thermal_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Power/thermal report generated: {report_path}")
        return report_path
    
    def optimize_power_settings(self) -> Dict[str, Any]:
        """Provide power optimization recommendations"""
        recommendations = {
            'current_power_state': {},
            'optimization_suggestions': [],
            'configuration_changes': []
        }
        
        # Analyze current power settings
        try:
            # Check CPU governor
            governors = set()
            governor_files = glob.glob('/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor')
            for gov_file in governor_files:
                try:
                    with open(gov_file, 'r') as f:
                        governors.add(f.read().strip())
                except:
                    pass
            
            recommendations['current_power_state']['cpu_governors'] = list(governors)
            
            # Check power management features
            features = {}
            
            # Check if Intel P-State is available
            if os.path.exists('/sys/devices/system/cpu/intel_pstate'):
                features['intel_pstate'] = True
            else:
                features['intel_pstate'] = False
            
            # Check ACPI features
            features['acpi_available'] = os.path.exists('/proc/acpi')
            
            recommendations['current_power_state']['power_features'] = features
            
        except Exception as e:
            self.logger.warning(f"Could not analyze current power settings: {e}")
        
        # Generate recommendations based on measurements
        if self.power_measurements:
            avg_power = statistics.mean([m.total_power_w for m in self.power_measurements])
            max_temp = max([m.cpu_temperature_c for m in self.power_measurements])
            
            if max_temp > self.config.thermal_warning_threshold:
                recommendations['optimization_suggestions'].append(
                    "Consider improving thermal management (cleaning, better cooling)"
                )
                recommendations['configuration_changes'].extend([
                    "Enable aggressive CPU throttling",
                    "Reduce maximum CPU frequency",
                    "Improve case ventilation"
                ])
            
            if avg_power > 100:  # High power consumption
                recommendations['optimization_suggestions'].append(
                    "High power consumption detected - consider power-saving optimizations"
                )
                recommendations['configuration_changes'].extend([
                    "Switch to performance governor when needed",
                    "Enable CPU C-states",
                    "Use GPU power limiting if available"
                ])
            
            # Performance recommendations
            if max_temp < 70 and avg_power < 50:
                recommendations['optimization_suggestions'].append(
                    "System has good thermal and power characteristics"
                )
                recommendations['configuration_changes'].append(
                    "System is suitable for sustained workloads"
                )
        
        return recommendations


def main():
    """Main function for standalone execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Power and Thermal Testing')
    parser.add_argument('--test', choices=['stress', 'thermal_cycle', 'idle', 'all'],
                       default='all', help='Test type to run')
    parser.add_argument('--duration', type=int, default=60, 
                       help='Test duration in minutes')
    parser.add_argument('--cycles', type=int, default=5, 
                       help='Number of thermal cycles')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    config = PowerTestConfig(duration_minutes=args.duration)
    tester = PowerThermalTester(config)
    
    # Detect thermal sensors
    thermal_zones = tester.detect_thermal_sensors()
    print(f"Detected {len(thermal_zones)} thermal zones")
    
    results = {}
    
    if args.test in ['stress', 'all']:
        print("Running power stress test...")
        results['stress_test'] = tester.run_power_stress_test()
    
    if args.test in ['thermal_cycle', 'all']:
        print("Running thermal cycling test...")
        results['thermal_cycling'] = tester.run_thermal_cycling_test(args.cycles)
    
    if args.test in ['idle', 'all']:
        print("Running idle power test...")
        results['idle_test'] = tester.run_idle_power_test(30)  # 30 minutes idle
    
    # Generate report
    if results:
        report_path = tester.generate_power_thermal_report()
        print(f"Power/thermal report generated: {report_path}")
        
        # Print summary
        if 'stress_test' in results:
            stress_results = results['stress_test']
            print(f"\nPower Test Summary:")
            print(f"  Average Power: {stress_results['power_consumption']['avg_total_power_w']:.1f}W")
            print(f"  Peak Power: {stress_results['power_consumption']['max_total_power_w']:.1f}W")
            print(f"  Max CPU Temp: {stress_results['temperatures']['max_cpu_temp_c']:.1f}°C")
            print(f"  Total Energy: {stress_results['energy']['total_energy_consumed_kwh']:.4f}kWh")


if __name__ == "__main__":
    main()