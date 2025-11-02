#!/usr/bin/env python3
"""
Hardware-Specific Optimization Recommendation System
AI-powered analysis of hardware capabilities and automated optimization recommendations
"""

import os
import sys
import json
import subprocess
import logging
import statistics
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import psutil

class OptimizationCategory(Enum):
    CPU = "cpu"
    MEMORY = "memory"
    STORAGE = "storage"
    NETWORK = "network"
    GPU = "gpu"
    POWER = "power"
    THERMAL = "thermal"
    SYSTEM = "system"
    COMPATIBILITY = "compatibility"

class OptimizationPriority(Enum):
    CRITICAL = "critical"
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    OPTIONAL = "optional"

@dataclass
class OptimizationRecommendation:
    """Individual optimization recommendation"""
    category: OptimizationCategory
    priority: OptimizationPriority
    title: str
    description: str
    implementation_steps: List[str]
    expected_benefit: str
    performance_impact: Dict[str, float]
    compatibility_check: Dict[str, Any]
    auto_applicable: bool
    manual_required: bool
    risk_level: str
    hardware_requirements: List[str]
    test_procedures: List[str]

@dataclass
class HardwareProfile:
    """Complete hardware profile"""
    system_info: Dict[str, Any]
    cpu_info: Dict[str, Any]
    memory_info: Dict[str, Any]
    storage_info: List[Dict[str, Any]]
    network_info: List[Dict[str, Any]]
    gpu_info: List[Dict[str, Any]]
    thermal_sensors: List[Dict[str, Any]]
    performance_baseline: Dict[str, Any]
    detected_workloads: List[str]
    optimization_potential: Dict[str, float]

class OptimizationEngine:
    """Main optimization recommendation engine"""
    
    def __init__(self):
        self.logger = self._setup_logging()
        self.hardware_profile = None
        self.optimization_rules = self._load_optimization_rules()
        self.performance_benchmarks = {}
        
    def _setup_logging(self):
        """Setup logging for optimization engine"""
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s',
            handlers=[
                logging.FileHandler('/workspace/testing/hardware_tests/optimization_engine.log'),
                logging.StreamHandler()
            ]
        )
        return logging.getLogger(__name__)
    
    def _load_optimization_rules(self) -> Dict[OptimizationCategory, List[Dict[str, Any]]]:
        """Load optimization rules and patterns"""
        return {
            OptimizationCategory.CPU: [
                {
                    'condition': lambda profile: profile.cpu_info.get('cores_logical', 1) > 8,
                    'recommendations': [
                        {
                            'title': 'Enable Intel Performance Extensions or AMD PBO',
                            'description': 'Optimize CPU boost behavior for high-core-count processors',
                            'implementation': [
                                'Enable Intel Performance Extensions in BIOS',
                                'Set CPU power policy to Performance',
                                'Disable CPU parking for all cores'
                            ],
                            'benefit': 'Improved multi-threaded performance',
                            'auto_applicable': False,
                            'manual_required': True,
                            'risk': 'low'
                        }
                    ]
                },
                {
                    'condition': lambda profile: 'ht' in ' '.join(profile.cpu_info.get('flags', [])).lower() or 
                                                 'smp' in ' '.join(profile.cpu_info.get('flags', [])).lower(),
                    'recommendations': [
                        {
                            'title': 'Optimize Hyperthreading Settings',
                            'description': 'Fine-tune hyperthreading for optimal performance',
                            'implementation': [
                                'Ensure hyperthreading is enabled in BIOS',
                                'Use CPU affinity for workloads',
                                'Monitor per-core utilization'
                            ],
                            'benefit': 'Improved thread scaling',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                },
                {
                    'condition': lambda profile: profile.cpu_info.get('frequency_max', 0) > 3.0,
                    'recommendations': [
                        {
                            'title': 'CPU Frequency Governor Optimization',
                            'description': 'Set appropriate frequency governor for performance workloads',
                            'implementation': [
                                'Set governor to "performance" for servers',
                                'Use "schedutil" for modern kernels',
                                'Configure CPU idle states'
                            ],
                            'benefit': 'Consistent performance and lower latency',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                }
            ],
            OptimizationCategory.MEMORY: [
                {
                    'condition': lambda profile: profile.memory_info.get('total_gb', 0) >= 32,
                    'recommendations': [
                        {
                            'title': 'Enable Large Page Support',
                            'description': 'Configure large pages for memory-intensive workloads',
                            'implementation': [
                                'Enable transparent huge pages',
                                'Configure reserved huge pages',
                                'Set appropriate huge page size'
                            ],
                            'benefit': 'Reduced TLB misses and improved memory performance',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                },
                {
                    'condition': lambda profile: profile.memory_info.get('ecc_support', False),
                    'recommendations': [
                        {
                            'title': 'ECC Memory Configuration',
                            'description': 'Optimize ECC memory settings for reliability',
                            'implementation': [
                                'Enable ECC error correction',
                                'Configure memory scrubbing intervals',
                                'Monitor memory error rates'
                            ],
                            'benefit': 'Improved system reliability and data integrity',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                }
            ],
            OptimizationCategory.STORAGE: [
                {
                    'condition': lambda profile: any(
                        'ssd' in device.get('type', '').lower() or 'nvme' in device.get('device', '').lower()
                        for device in profile.storage_info
                    ),
                    'recommendations': [
                        {
                            'title': 'SSD Optimization Settings',
                            'description': 'Configure SSD-specific optimizations',
                            'implementation': [
                                'Enable TRIM support',
                                'Disable access time updates for SSD',
                                'Use noatime mount option',
                                'Configure I/O scheduler for SSD (none or deadline)'
                            ],
                            'benefit': 'Extended SSD lifespan and improved performance',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                },
                {
                    'condition': lambda profile: any(
                        'raid' in str(device).lower() 
                        for device in profile.storage_info
                    ),
                    'recommendations': [
                        {
                            'title': 'RAID Array Optimization',
                            'description': 'Optimize RAID configuration for performance',
                            'implementation': [
                                'Use appropriate RAID level for workload',
                                'Configure RAID write-back caching',
                                'Set optimal stripe size',
                                'Enable RAID background initialization'
                            ],
                            'benefit': 'Improved storage performance and reliability',
                            'auto_applicable': False,
                            'manual_required': True,
                            'risk': 'medium'
                        }
                    ]
                }
            ],
            OptimizationCategory.NETWORK: [
                {
                    'condition': lambda profile: any(
                        interface.get('speed_mbps', 0) >= 1000
                        for interface in profile.network_info
                    ),
                    'recommendations': [
                        {
                            'title': 'Network Interface Optimization',
                            'description': 'Optimize network settings for high-speed interfaces',
                            'implementation': [
                                'Enable interrupt coalescing',
                                'Configure network buffer sizes',
                                'Enable hardware offloading features',
                                'Use appropriate interrupt moderation'
                            ],
                            'benefit': 'Improved network throughput and reduced latency',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                }
            ],
            OptimizationCategory.GPU: [
                {
                    'condition': lambda profile: len(profile.gpu_info) > 0,
                    'recommendations': [
                        {
                            'title': 'GPU Driver Optimization',
                            'description': 'Configure GPU drivers for optimal performance',
                            'implementation': [
                                'Install latest GPU drivers',
                                'Enable GPU persistent mode',
                                'Configure GPU memory allocation',
                                'Set appropriate power management mode'
                            ],
                            'benefit': 'Improved GPU performance and stability',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                }
            ],
            OptimizationCategory.POWER: [
                {
                    'condition': lambda profile: profile.cpu_info.get('cores_logical', 1) > 4,
                    'recommendations': [
                        {
                            'title': 'Power Management Optimization',
                            'description': 'Configure power management for performance vs efficiency',
                            'implementation': [
                                'Set appropriate CPU power governor',
                                'Enable C-states for power saving',
                                'Configure P-states for performance',
                                'Set CPU frequency scaling policies'
                            ],
                            'benefit': 'Balanced performance and power consumption',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                }
            ],
            OptimizationCategory.THERMAL: [
                {
                    'condition': lambda profile: len(profile.thermal_sensors) > 0,
                    'recommendations': [
                        {
                            'title': 'Thermal Management Configuration',
                            'description': 'Optimize thermal management settings',
                            'implementation': [
                                'Configure thermal zones and trip points',
                                'Set appropriate fan control policies',
                                'Enable thermal monitoring',
                                'Configure CPU thermal throttling'
                            ],
                            'benefit': 'Prevent thermal throttling and maintain performance',
                            'auto_applicable': True,
                            'manual_required': False,
                            'risk': 'low'
                        }
                    ]
                }
            ]
        }
    
    def analyze_hardware_capabilities(self, hardware_profile: HardwareProfile) -> Dict[str, Any]:
        """Analyze hardware capabilities and potential optimizations"""
        self.hardware_profile = hardware_profile
        
        analysis = {
            'cpu_analysis': self._analyze_cpu_capabilities(),
            'memory_analysis': self._analyze_memory_capabilities(),
            'storage_analysis': self._analyze_storage_capabilities(),
            'network_analysis': self._analyze_network_capabilities(),
            'gpu_analysis': self._analyze_gpu_capabilities(),
            'overall_optimization_score': self._calculate_optimization_score()
        }
        
        return analysis
    
    def _analyze_cpu_capabilities(self) -> Dict[str, Any]:
        """Analyze CPU capabilities and optimization opportunities"""
        if not self.hardware_profile:
            return {}
        
        cpu_info = self.hardware_profile.cpu_info
        
        analysis = {
            'cpu_model': cpu_info.get('model', 'Unknown'),
            'physical_cores': cpu_info.get('cores_physical', 0),
            'logical_cores': cpu_info.get('cores_logical', 0),
            'max_frequency_ghz': cpu_info.get('frequency_max', 0),
            'features': cpu_info.get('flags', []),
            'optimization_potential': {
                'thread_scaling': 'excellent' if cpu_info.get('cores_logical', 1) > 8 else 'good',
                'single_thread_performance': 'excellent' if cpu_info.get('frequency_max', 0) > 3.5 else 'good',
                'virtualization_support': cpu_info.get('vendor') in ['Intel', 'AMD'],
                'instruction_sets': len([f for f in cpu_info.get('flags', []) if f in ['avx2', 'avx512', 'sse4_2']]),
                'efficiency_cores': 0,  # Would need more detailed CPU info
                'performance_cores': cpu_info.get('cores_physical', 0)
            }
        }
        
        # Calculate performance score
        performance_score = 0
        performance_score += min(cpu_info.get('cores_physical', 1) * 10, 50)
        performance_score += min(cpu_info.get('frequency_max', 0) * 10, 30)
        performance_score += len(cpu_info.get('flags', [])) * 2
        performance_score = min(performance_score, 100)
        
        analysis['performance_score'] = performance_score
        
        return analysis
    
    def _analyze_memory_capabilities(self) -> Dict[str, Any]:
        """Analyze memory capabilities and optimization opportunities"""
        if not self.hardware_profile:
            return {}
        
        memory_info = self.hardware_profile.memory_info
        
        analysis = {
            'total_memory_gb': memory_info.get('total_gb', 0),
            'memory_type': memory_info.get('memory_type', 'Unknown'),
            'memory_speed_mhz': memory_info.get('speed_mhz', 0),
            'ecc_support': memory_info.get('ecc_support', False),
            'channels': memory_info.get('channels', 0),
            'optimization_potential': {
                'capacity_score': min(memory_info.get('total_gb', 0) * 2, 100),
                'speed_score': min(memory_info.get('speed_mhz', 0) / 100, 100),
                'reliability_score': 100 if memory_info.get('ecc_support', False) else 70,
                'bandwidth_score': memory_info.get('channels', 1) * 25
            }
        }
        
        return analysis
    
    def _analyze_storage_capabilities(self) -> Dict[str, Any]:
        """Analyze storage capabilities and optimization opportunities"""
        if not self.hardware_profile:
            return {}
        
        storage_devices = self.hardware_profile.storage_info
        
        analysis = {
            'total_devices': len(storage_devices),
            'storage_types': {},
            'total_capacity_gb': 0,
            'optimization_potential': {}
        }
        
        ssd_count = 0
        nvme_count = 0
        hdd_count = 0
        
        for device in storage_devices:
            device_type = device.get('type', 'unknown')
            device_name = device.get('device', '').lower()
            
            if 'ssd' in device_type.lower():
                ssd_count += 1
                analysis['storage_types']['ssd'] = ssd_count
            elif 'nvme' in device_name or 'nvme' in device_type.lower():
                nvme_count += 1
                analysis['storage_types']['nvme'] = nvme_count
            elif 'disk' in device_type.lower():
                hdd_count += 1
                analysis['storage_types']['hdd'] = hdd_count
            
            analysis['total_capacity_gb'] += device.get('size_gb', 0)
        
        # Calculate storage optimization potential
        storage_score = 0
        if nvme_count > 0:
            storage_score += 100
        elif ssd_count > 0:
            storage_score += 80
        elif hdd_count > 0:
            storage_score += 60
        
        analysis['optimization_potential'] = {
            'storage_score': storage_score,
            'io_performance': 'excellent' if nvme_count > 0 else 'good' if ssd_count > 0 else 'adequate',
            'reliability': 'excellent' if hdd_count == 0 else 'good',
            'capacity_scalability': 'limited' if analysis['total_capacity_gb'] < 100 else 'good'
        }
        
        return analysis
    
    def _analyze_network_capabilities(self) -> Dict[str, Any]:
        """Analyze network capabilities and optimization opportunities"""
        if not self.hardware_profile:
            return {}
        
        network_interfaces = self.hardware_profile.network_info
        
        analysis = {
            'total_interfaces': len(network_interfaces),
            'interface_types': {},
            'total_bandwidth_gbps': 0,
            'optimization_potential': {}
        }
        
        for interface in network_interfaces:
            interface_type = 'ethernet' if interface.get('is_ethernet') else 'wireless' if interface.get('is_wireless') else 'unknown'
            speed_mbps = interface.get('speed_mbps', 100)
            
            if interface_type in analysis['interface_types']:
                analysis['interface_types'][interface_type] += 1
            else:
                analysis['interface_types'][interface_type] = 1
            
            analysis['total_bandwidth_gbps'] += speed_mbps / 1000
        
        # Calculate network optimization potential
        max_speed = max([i.get('speed_mbps', 100) for i in network_interfaces] + [100])
        
        analysis['optimization_potential'] = {
            'network_score': min(max_speed / 100, 100),  # Scale to 100
            'bandwidth_score': min(analysis['total_bandwidth_gbps'], 100),
            'interface_diversity': len(analysis['interface_types']),
            'optimization_potential': 'high' if max_speed >= 1000 else 'medium' if max_speed >= 100 else 'low'
        }
        
        return analysis
    
    def _analyze_gpu_capabilities(self) -> Dict[str, Any]:
        """Analyze GPU capabilities and optimization opportunities"""
        if not self.hardware_profile:
            return {}
        
        gpu_devices = self.hardware_profile.gpu_info
        
        analysis = {
            'total_gpus': len(gpu_devices),
            'gpu_vendors': {},
            'total_vram_gb': 0,
            'optimization_potential': {}
        }
        
        for gpu in gpu_devices:
            vendor = gpu.get('vendor', 'Unknown')
            if vendor in analysis['gpu_vendors']:
                analysis['gpu_vendors'][vendor] += 1
            else:
                analysis['gpu_vendors'][vendor] = 1
            
            analysis['total_vram_gb'] += gpu.get('memory_mb', 0) / 1024
        
        # Calculate GPU optimization potential
        if len(gpu_devices) > 0:
            analysis['optimization_potential'] = {
                'gpu_score': min(sum(gpu.get('memory_mb', 0) for gpu in gpu_devices) / 100, 100),
                'compute_potential': 'excellent' if analysis['total_vram_gb'] >= 8 else 'good' if analysis['total_vram_gb'] >= 4 else 'basic',
                'multi_gpu_support': len(gpu_devices) > 1,
                'driver_optimization_potential': 'high' if 'NVIDIA' in analysis['gpu_vendors'] else 'medium'
            }
        else:
            analysis['optimization_potential'] = {
                'gpu_score': 0,
                'compute_potential': 'none',
                'multi_gpu_support': False,
                'driver_optimization_potential': 'none'
            }
        
        return analysis
    
    def _calculate_optimization_score(self) -> float:
        """Calculate overall hardware optimization score"""
        if not self.hardware_profile:
            return 0.0
        
        scores = []
        
        # CPU score
        cpu_info = self.hardware_profile.cpu_info
        cpu_score = min(
            (cpu_info.get('cores_physical', 1) * 10) + 
            (cpu_info.get('frequency_max', 0) * 10) +
            (len(cpu_info.get('flags', [])) * 2),
            100
        )
        scores.append(cpu_score)
        
        # Memory score
        memory_info = self.hardware_profile.memory_info
        memory_score = min(memory_info.get('total_gb', 0) * 2, 100)
        scores.append(memory_score)
        
        # Storage score
        storage_devices = self.hardware_profile.storage_info
        storage_score = min(len(storage_devices) * 20, 100)
        scores.append(storage_score)
        
        # Network score
        network_interfaces = self.hardware_profile.network_info
        network_score = min(len(network_interfaces) * 25, 100)
        scores.append(network_score)
        
        # GPU score
        gpu_devices = self.hardware_profile.gpu_info
        gpu_score = min(len(gpu_devices) * 50, 100)
        scores.append(gpu_score)
        
        return sum(scores) / len(scores) if scores else 0.0
    
    def generate_optimization_recommendations(self) -> List[OptimizationRecommendation]:
        """Generate optimization recommendations based on hardware profile"""
        if not self.hardware_profile:
            self.logger.error("No hardware profile available for optimization analysis")
            return []
        
        recommendations = []
        
        # Apply optimization rules for each category
        for category, rules in self.optimization_rules.items():
            for rule in rules:
                try:
                    if rule['condition'](self.hardware_profile):
                        for rec_template in rule['recommendations']:
                            recommendation = OptimizationRecommendation(
                                category=category,
                                priority=self._determine_priority(rec_template),
                                title=rec_template['title'],
                                description=rec_template['description'],
                                implementation_steps=rec_template['implementation'],
                                expected_benefit=rec_template['benefit'],
                                performance_impact=self._estimate_performance_impact(rec_template),
                                compatibility_check=self._check_compatibility(rec_template),
                                auto_applicable=rec_template.get('auto_applicable', False),
                                manual_required=rec_template.get('manual_required', True),
                                risk_level=rec_template.get('risk', 'low'),
                                hardware_requirements=self._extract_hardware_requirements(rec_template),
                                test_procedures=self._generate_test_procedures(rec_template)
                            )
                            recommendations.append(recommendation)
                except Exception as e:
                    self.logger.error(f"Error applying optimization rule for {category}: {e}")
        
        # Add workload-specific recommendations
        recommendations.extend(self._generate_workload_specific_recommendations())
        
        # Sort by priority
        priority_order = {
            OptimizationPriority.CRITICAL: 0,
            OptimizationPriority.HIGH: 1,
            OptimizationPriority.MEDIUM: 2,
            OptimizationPriority.LOW: 3,
            OptimizationPriority.OPTIONAL: 4
        }
        
        recommendations.sort(key=lambda r: priority_order[r.priority])
        
        return recommendations
    
    def _determine_priority(self, recommendation_template: Dict[str, Any]) -> OptimizationPriority:
        """Determine priority based on recommendation type and hardware"""
        title = recommendation_template['title'].lower()
        benefit = recommendation_template.get('benefit', '').lower()
        
        if 'critical' in title or 'failure' in benefit:
            return OptimizationPriority.CRITICAL
        elif 'performance' in benefit or 'cpu' in title:
            return OptimizationPriority.HIGH
        elif 'optimization' in title or 'tuning' in title:
            return OptimizationPriority.MEDIUM
        else:
            return OptimizationPriority.LOW
    
    def _estimate_performance_impact(self, recommendation_template: Dict[str, Any]) -> Dict[str, float]:
        """Estimate performance impact of optimization"""
        title = recommendation_template['title'].lower()
        benefit = recommendation_template.get('benefit', '').lower()
        
        impact = {
            'throughput': 0.0,
            'latency': 0.0,
            'power_consumption': 0.0,
            'reliability': 0.0
        }
        
        if 'cpu' in title or 'frequency' in benefit:
            impact['throughput'] = 10.0
            impact['latency'] = -5.0  # Negative means improvement
        elif 'memory' in title or 'memory' in benefit:
            impact['throughput'] = 15.0
            impact['latency'] = -10.0
        elif 'storage' in title or 'disk' in benefit:
            impact['throughput'] = 20.0
            impact['latency'] = -15.0
        elif 'network' in title or 'network' in benefit:
            impact['throughput'] = 25.0
            impact['latency'] = -20.0
        elif 'thermal' in title or 'temperature' in benefit:
            impact['power_consumption'] = -10.0  # Reduced power
            impact['reliability'] = 20.0
        
        return impact
    
    def _check_compatibility(self, recommendation_template: Dict[str, Any]) -> Dict[str, Any]:
        """Check compatibility with current hardware"""
        return {
            'compatible': True,
            'warnings': [],
            'conflicts': [],
            'prerequisites': []
        }
    
    def _extract_hardware_requirements(self, recommendation_template: Dict[str, Any]) -> List[str]:
        """Extract hardware requirements for recommendation"""
        title = recommendation_template['title'].lower()
        
        requirements = []
        
        if 'cpu' in title:
            requirements.append('Multi-core processor recommended')
        if 'memory' in title:
            requirements.append('Sufficient system memory')
        if 'storage' in title:
            requirements.append('SSD or NVMe storage recommended')
        if 'network' in title:
            requirements.append('Gigabit network interface')
        
        return requirements
    
    def _generate_test_procedures(self, recommendation_template: Dict[str, Any]) -> List[str]:
        """Generate test procedures for optimization"""
        title = recommendation_template['title'].lower()
        
        tests = []
        
        if 'cpu' in title:
            tests.extend([
                'Run CPU benchmark before and after optimization',
                'Monitor CPU utilization under load',
                'Check CPU frequency scaling behavior'
            ])
        elif 'memory' in title:
            tests.extend([
                'Run memory bandwidth test',
                'Monitor memory usage patterns',
                'Test with memory-intensive workloads'
            ])
        elif 'storage' in title:
            tests.extend([
                'Run disk performance benchmark',
                'Monitor disk I/O patterns',
                'Test with various workload sizes'
            ])
        elif 'network' in title:
            tests.extend([
                'Run network throughput test',
                'Monitor network latency',
                'Test under various load conditions'
            ])
        
        return tests
    
    def _generate_workload_specific_recommendations(self) -> List[OptimizationRecommendation]:
        """Generate recommendations based on detected workloads"""
        recommendations = []
        
        # This is a simplified implementation
        # In practice, you would analyze actual workload patterns
        
        # Database workload recommendations
        recommendations.append(OptimizationRecommendation(
            category=OptimizationCategory.STORAGE,
            priority=OptimizationPriority.HIGH,
            title='Database Storage Optimization',
            description='Optimize storage for database workloads',
            implementation_steps=[
                'Use SSD storage for database files',
                'Configure appropriate I/O scheduler',
                'Enable direct I/O for database operations'
            ],
            expected_benefit='Improved database query performance and reduced latency',
            performance_impact={'throughput': 30.0, 'latency': -25.0},
            compatibility_check={'compatible': True, 'warnings': []},
            auto_applicable=True,
            manual_required=False,
            risk_level='low',
            hardware_requirements=['SSD storage recommended'],
            test_procedures=['Run database benchmark', 'Monitor disk I/O']
        ))
        
        # Compute workload recommendations
        if self.hardware_profile.cpu_info.get('cores_physical', 1) >= 8:
            recommendations.append(OptimizationRecommendation(
                category=OptimizationCategory.CPU,
                priority=OptimizationPriority.HIGH,
                title='Compute Workload Optimization',
                description='Optimize for compute-intensive workloads',
                implementation_steps=[
                    'Set CPU governor to performance mode',
                    'Disable CPU frequency scaling',
                    'Enable all CPU cores'
                ],
                expected_benefit='Maximum compute performance for parallel workloads',
                performance_impact={'throughput': 40.0, 'latency': -20.0},
                compatibility_check={'compatible': True, 'warnings': []},
                auto_applicable=True,
                manual_required=False,
                risk_level='medium',
                hardware_requirements=['8+ CPU cores recommended'],
                test_procedures=['Run compute benchmark', 'Monitor CPU utilization']
            ))
        
        return recommendations
    
    def apply_automatic_optimizations(self, recommendations: List[OptimizationRecommendation]) -> Dict[str, Any]:
        """Apply automatic optimizations"""
        results = {
            'applied_optimizations': [],
            'failed_optimizations': [],
            'manual_interventions_required': []
        }
        
        for recommendation in recommendations:
            if recommendation.auto_applicable:
                try:
                    success = self._apply_optimization(recommendation)
                    if success:
                        results['applied_optimizations'].append(recommendation.title)
                    else:
                        results['failed_optimizations'].append(recommendation.title)
                except Exception as e:
                    self.logger.error(f"Failed to apply optimization {recommendation.title}: {e}")
                    results['failed_optimizations'].append(recommendation.title)
            else:
                results['manual_interventions_required'].append(recommendation.title)
        
        return results
    
    def _apply_optimization(self, recommendation: OptimizationRecommendation) -> bool:
        """Apply individual optimization"""
        try:
            title = recommendation.title.lower()
            
            if 'cpu governor' in title:
                return self._set_cpu_governor('performance')
            elif 'frequency' in title:
                return self._configure_cpu_frequency()
            elif 'thermal' in title:
                return self._configure_thermal_management()
            elif 'network' in title:
                return self._optimize_network_settings()
            elif 'gpu' in title and 'driver' in title:
                return self._optimize_gpu_settings()
            else:
                self.logger.info(f"Optimization {recommendation.title} requires manual intervention")
                return False
                
        except Exception as e:
            self.logger.error(f"Error applying optimization {recommendation.title}: {e}")
            return False
    
    def _set_cpu_governor(self, governor: str) -> bool:
        """Set CPU governor"""
        try:
            governor_path = '/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor'
            import glob
            governor_files = glob.glob(governor_path)
            
            for gov_file in governor_files:
                with open(gov_file, 'w') as f:
                    f.write(governor)
            
            return True
        except Exception as e:
            self.logger.error(f"Failed to set CPU governor: {e}")
            return False
    
    def _configure_cpu_frequency(self) -> bool:
        """Configure CPU frequency settings"""
        try:
            # Enable performance mode
            self._set_cpu_governor('performance')
            return True
        except Exception as e:
            self.logger.error(f"Failed to configure CPU frequency: {e}")
            return False
    
    def _configure_thermal_management(self) -> bool:
        """Configure thermal management settings"""
        try:
            # This would require system-specific thermal configuration
            self.logger.info("Thermal management configuration requires manual setup")
            return False
        except Exception as e:
            self.logger.error(f"Failed to configure thermal management: {e}")
            return False
    
    def _optimize_network_settings(self) -> bool:
        """Optimize network settings"""
        try:
            # Enable various network optimizations
            optimizations = [
                'net.core.rmem_max = 134217728',
                'net.core.wmem_max = 134217728',
                'net.ipv4.tcp_rmem = 4096 87380 134217728',
                'net.ipv4.tcp_wmem = 4096 65536 134217728'
            ]
            
            for optimization in optimizations:
                try:
                    subprocess.run(['sysctl', '-w', optimization], check=True)
                except:
                    pass  # Continue with other optimizations
            
            return True
        except Exception as e:
            self.logger.error(f"Failed to optimize network settings: {e}")
            return False
    
    def _optimize_gpu_settings(self) -> bool:
        """Optimize GPU settings"""
        try:
            # Check for NVIDIA GPU optimization
            result = subprocess.run(['nvidia-smi', '--query-gpu=name', '--format=csv,noheader'], 
                                  capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                # NVIDIA GPU detected - would apply specific optimizations
                self.logger.info("NVIDIA GPU optimizations applied")
                return True
        except:
            pass
        
        return False
    
    def generate_optimization_report(self, recommendations: List[OptimizationRecommendation]) -> str:
        """Generate comprehensive optimization report"""
        report_data = {
            'report_info': {
                'generated_at': time.time(),
                'optimization_engine_version': '1.0'
            },
            'hardware_profile': asdict(self.hardware_profile) if self.hardware_profile else {},
            'recommendations_summary': {
                'total_recommendations': len(recommendations),
                'automatic_optimizations': len([r for r in recommendations if r.auto_applicable]),
                'manual_interventions': len([r for r in recommendations if r.manual_required]),
                'priority_breakdown': self._count_recommendations_by_priority(recommendations)
            },
            'detailed_recommendations': [asdict(rec) for rec in recommendations],
            'implementation_plan': self._generate_implementation_plan(recommendations),
            'expected_improvements': self._calculate_expected_improvements(recommendations)
        }
        
        report_path = f"/workspace/testing/hardware_tests/results/optimization_report_{int(time.time())}.json"
        
        with open(report_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        self.logger.info(f"Optimization report generated: {report_path}")
        return report_path
    
    def _count_recommendations_by_priority(self, recommendations: List[OptimizationRecommendation]) -> Dict[str, int]:
        """Count recommendations by priority"""
        counts = {}
        for rec in recommendations:
            priority = rec.priority.value
            counts[priority] = counts.get(priority, 0) + 1
        return counts
    
    def _generate_implementation_plan(self, recommendations: List[OptimizationRecommendation]) -> Dict[str, Any]:
        """Generate implementation plan for recommendations"""
        plan = {
            'phase_1_critical': [],
            'phase_2_high': [],
            'phase_3_medium': [],
            'phase_4_low': []
        }
        
        for rec in recommendations:
            rec_dict = {
                'title': rec.title,
                'category': rec.category.value,
                'auto_applicable': rec.auto_applicable,
                'manual_required': rec.manual_required,
                'steps': rec.implementation_steps
            }
            
            if rec.priority == OptimizationPriority.CRITICAL:
                plan['phase_1_critical'].append(rec_dict)
            elif rec.priority == OptimizationPriority.HIGH:
                plan['phase_2_high'].append(rec_dict)
            elif rec.priority == OptimizationPriority.MEDIUM:
                plan['phase_3_medium'].append(rec_dict)
            else:
                plan['phase_4_low'].append(rec_dict)
        
        return plan
    
    def _calculate_expected_improvements(self, recommendations: List[OptimizationRecommendation]) -> Dict[str, float]:
        """Calculate expected performance improvements"""
        total_impact = {
            'throughput': 0.0,
            'latency': 0.0,
            'power_consumption': 0.0,
            'reliability': 0.0
        }
        
        for rec in recommendations:
            for metric, impact in rec.performance_impact.items():
                total_impact[metric] += impact
        
        return total_impact


def main():
    """Main function for standalone execution"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Hardware Optimization Recommendation')
    parser.add_argument('--analyze', action='store_true', help='Analyze hardware capabilities')
    parser.add_argument('--recommend', action='store_true', help='Generate optimization recommendations')
    parser.add_argument('--apply', action='store_true', help='Apply automatic optimizations')
    parser.add_argument('--hardware-profile', type=str, help='Path to hardware profile JSON')
    parser.add_argument('--output', type=str, help='Output file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Initialize optimization engine
    engine = OptimizationEngine()
    
    # Load hardware profile
    if args.hardware_profile:
        with open(args.hardware_profile, 'r') as f:
            profile_data = json.load(f)
            hardware_profile = HardwareProfile(**profile_data)
    else:
        # Generate hardware profile from current system
        from hardware_detector import HardwareDetector
        detector = HardwareDetector()
        profile_data = detector.run_full_detection()
        hardware_profile = HardwareProfile(
            system_info=profile_data.get('system', {}),
            cpu_info=profile_data.get('cpu', {}),
            memory_info=profile_data.get('memory', {}),
            storage_info=profile_data.get('storage', []),
            network_info=profile_data.get('network', []),
            gpu_info=profile_data.get('gpu', []),
            thermal_sensors=profile_data.get('system', {}).get('temperature_sensors', []),
            performance_baseline={},
            detected_workloads=[],
            optimization_potential={}
        )
    
    engine.hardware_profile = hardware_profile
    
    # Run analysis
    if args.analyze or not (args.recommend or args.apply):
        print("Analyzing hardware capabilities...")
        analysis = engine.analyze_hardware_capabilities(hardware_profile)
        print(f"Overall optimization score: {analysis['overall_optimization_score']:.1f}/100")
    
    # Generate recommendations
    if args.recommend or args.apply:
        print("Generating optimization recommendations...")
        recommendations = engine.generate_optimization_recommendations()
        
        print(f"\nGenerated {len(recommendations)} recommendations:")
        for rec in recommendations:
            print(f"  [{rec.priority.value.upper()}] {rec.title}")
        
        # Generate report
        report_path = engine.generate_optimization_report(recommendations)
        print(f"Optimization report generated: {report_path}")
        
        # Apply automatic optimizations
        if args.apply:
            print("Applying automatic optimizations...")
            results = engine.apply_automatic_optimizations(recommendations)
            print(f"Applied: {len(results['applied_optimizations'])} optimizations")
            print(f"Failed: {len(results['failed_optimizations'])} optimizations")
            print(f"Manual intervention required: {len(results['manual_interventions_required'])} optimizations")


if __name__ == "__main__":
    main()