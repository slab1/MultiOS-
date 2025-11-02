"""
Input Device Calibration and Configuration Tools for MultiOS

Provides comprehensive calibration, configuration, and diagnostic tools
for all input devices with educational guidance and automated testing.
"""

from typing import Dict, List, Optional, Tuple, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
import time
import json
import threading
import logging
from collections import defaultdict
import math

from ..core.input_device import InputDevice
from ..core.device_manager import DeviceManager
from ..core.input_event import InputEvent, EventType, EventPriority


class CalibrationType(Enum):
    """Types of calibration procedures"""
    BASIC = "basic"
    ADVANCED = "advanced"
    FULL = "full"
    AUTOMATED = "automated"
    EDUCATIONAL = "educational"


class CalibrationStatus(Enum):
    """Calibration status states"""
    NOT_STARTED = "not_started"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class CalibrationPoint:
    """Individual calibration point data"""
    expected_x: float
    expected_y: float
    actual_x: Optional[float] = None
    actual_y: Optional[float] = None
    error_x: float = 0.0
    error_y: float = 0.0
    error_distance: float = 0.0
    confidence: float = 1.0
    timestamp: float = field(default_factory=time.time)
    
    def calculate_error(self):
        """Calculate error from expected to actual"""
        if self.actual_x is not None and self.actual_y is not None:
            self.error_x = self.actual_x - self.expected_x
            self.error_y = self.actual_y - self.expected_y
            self.error_distance = math.sqrt(self.error_x**2 + self.error_y**2)
            self.confidence = max(0.0, min(1.0, 1.0 - self.error_distance / 100.0))


@dataclass
class CalibrationResults:
    """Comprehensive calibration results"""
    device_id: str
    device_type: str
    calibration_type: CalibrationType
    status: CalibrationStatus
    start_time: float
    end_time: Optional[float] = None
    duration: float = 0.0
    
    # Calibration data
    points: List[CalibrationPoint] = field(default_factory=list)
    transformation_matrix: Optional[List[List[float]]] = None
    bias_corrections: Optional[Dict[str, float]] = None
    
    # Statistics
    mean_error: float = 0.0
    max_error: float = 0.0
    rms_error: float = 0.0
    success_rate: float = 0.0
    
    # Quality metrics
    accuracy_score: float = 0.0
    precision_score: float = 0.0
    repeatability_score: float = 0.0
    
    def calculate_statistics(self):
        """Calculate calibration statistics"""
        if not self.points:
            return
        
        errors = [point.error_distance for point in self.points if point.error_distance > 0]
        
        if not errors:
            return
        
        self.mean_error = sum(errors) / len(errors)
        self.max_error = max(errors)
        self.rms_error = math.sqrt(sum(e**2 for e in errors) / len(errors))
        self.success_rate = len(errors) / len(self.points)
        
        # Calculate quality scores (0-100)
        self.accuracy_score = max(0, min(100, 100 - self.mean_error))
        self.precision_score = max(0, min(100, 100 - self.rms_error))
        self.repeatability_score = max(0, min(100, 100 - self.max_error))
        
        self.duration = (self.end_time or time.time()) - self.start_time


class InputDeviceDiagnostic:
    """Comprehensive diagnostic system for input devices"""
    
    def __init__(self):
        self.diagnostic_history: List[Dict] = []
        self.known_issues: Dict[str, List[str]] = defaultdict(list)
        self.performance_baselines: Dict[str, Dict] = {}
        
        # Diagnostic patterns
        self.issue_patterns = {
            'drift': {
                'pattern': lambda x: abs(x['position_drift']) > 10,
                'description': 'Position drift detected',
                'severity': 'medium'
            },
            'noise': {
                'pattern': lambda x: x['signal_noise_ratio'] < 10,
                'description': 'High signal noise detected',
                'severity': 'high'
            },
            'dead_zone': {
                'pattern': lambda x: x['dead_zone_size'] > 20,
                'description': 'Excessive dead zone detected',
                'severity': 'low'
            },
            'calibration_lost': {
                'pattern': lambda x: x['calibration_error'] > 50,
                'description': 'Calibration accuracy degraded',
                'severity': 'high'
            },
            'sampling_rate_drop': {
                'pattern': lambda x: x['actual_sampling_rate'] < x['expected_sampling_rate'] * 0.8,
                'description': 'Sampling rate below expected',
                'severity': 'medium'
            }
        }
    
    def run_diagnostics(self, device: InputDevice) -> Dict[str, Any]:
        """Run comprehensive diagnostics on device"""
        diagnostic_start = time.time()
        
        diagnostics = {
            'device_id': device.device_id,
            'device_type': device.device_type,
            'timestamp': diagnostic_start,
            'tests': {}
        }
        
        # Test connectivity
        diagnostics['tests']['connectivity'] = self._test_connectivity(device)
        
        # Test responsiveness
        diagnostics['tests']['responsiveness'] = self._test_responsiveness(device)
        
        # Test accuracy
        diagnostics['tests']['accuracy'] = self._test_accuracy(device)
        
        # Test repeatability
        diagnostics['tests']['repeatability'] = self._test_repeatability(device)
        
        # Test signal quality
        diagnostics['tests']['signal_quality'] = self._test_signal_quality(device)
        
        # Test performance metrics
        diagnostics['tests']['performance'] = self._test_performance(device)
        
        # Overall assessment
        diagnostics['overall_score'] = self._calculate_overall_score(diagnostics['tests'])
        diagnostics['issues_found'] = self._identify_issues(diagnostics['tests'])
        diagnostics['recommendations'] = self._generate_recommendations(diagnostics['tests'])
        
        self.diagnostic_history.append(diagnostics)
        return diagnostics
    
    def _test_connectivity(self, device: InputDevice) -> Dict[str, Any]:
        """Test device connectivity"""
        return {
            'connected': device.is_connected,
            'enabled': device.is_enabled,
            'signal_strength': 95 if device.is_connected else 0,
            'response_time': 0.001,
            'status': 'pass' if device.is_connected else 'fail'
        }
    
    def _test_responsiveness(self, device: InputDevice) -> Dict[str, Any]:
        """Test device responsiveness"""
        # Simulate responsiveness testing
        import random
        return {
            'input_latency': random.uniform(1, 10),  # milliseconds
            'event_rate': random.uniform(50, 120),
            'response_consistency': random.uniform(0.8, 1.0),
            'status': 'pass' if random.random() > 0.1 else 'warning'
        }
    
    def _test_accuracy(self, device: InputDevice) -> Dict[str, Any]:
        """Test device accuracy"""
        # Simulate accuracy testing
        import random
        return {
            'position_accuracy': random.uniform(1, 5),  # pixels
            'pressure_accuracy': random.uniform(0.05, 0.2),  # percentage
            'angle_accuracy': random.uniform(1, 10),  # degrees
            'accuracy_score': random.uniform(0.8, 0.99),
            'status': 'pass' if random.random() > 0.15 else 'fail'
        }
    
    def _test_repeatability(self, device: InputDevice) -> Dict[str, Any]:
        """Test device repeatability"""
        # Simulate repeatability testing
        import random
        return {
            'standard_deviation': random.uniform(0.5, 3.0),
            'variance': random.uniform(0.1, 1.0),
            'repeatability_score': random.uniform(0.75, 0.95),
            'status': 'pass' if random.random() > 0.2 else 'warning'
        }
    
    def _test_signal_quality(self, device: InputDevice) -> Dict[str, Any]:
        """Test signal quality"""
        # Simulate signal quality testing
        import random
        return {
            'signal_noise_ratio': random.uniform(10, 30),
            'signal_strength': random.uniform(80, 100),
            'interference_level': random.uniform(0, 10),
            'signal_quality_score': random.uniform(0.7, 0.95),
            'status': 'pass' if random.random() > 0.1 else 'warning'
        }
    
    def _test_performance(self, device: InputDevice) -> Dict[str, Any]:
        """Test device performance metrics"""
        # Simulate performance testing
        import random
        expected_sampling = getattr(device.get_capabilities(), 'sampling_rate', 60)
        actual_sampling = random.uniform(expected_sampling * 0.7, expected_sampling * 1.1)
        
        return {
            'expected_sampling_rate': expected_sampling,
            'actual_sampling_rate': actual_sampling,
            'cpu_usage': random.uniform(5, 15),  # percentage
            'memory_usage': random.uniform(10, 30),  # MB
            'throughput': random.uniform(0.8, 1.0),  # relative
            'status': 'pass' if actual_sampling >= expected_sampling * 0.8 else 'warning'
        }
    
    def _calculate_overall_score(self, tests: Dict[str, Dict]) -> float:
        """Calculate overall diagnostic score"""
        scores = []
        weights = {
            'connectivity': 0.25,
            'responsiveness': 0.20,
            'accuracy': 0.25,
            'repeatability': 0.15,
            'signal_quality': 0.10,
            'performance': 0.05
        }
        
        for test_name, result in tests.items():
            if test_name in weights:
                if result['status'] == 'pass':
                    score = 100
                elif result['status'] == 'warning':
                    score = 70
                else:  # fail
                    score = 30
                scores.append(score * weights[test_name])
        
        return sum(scores) if scores else 0
    
    def _identify_issues(self, tests: Dict[str, Dict]) -> List[Dict[str, Any]]:
        """Identify issues from test results"""
        issues = []
        
        for test_name, result in tests.items():
            if result['status'] == 'fail':
                issues.append({
                    'test': test_name,
                    'severity': 'high',
                    'description': f'{test_name.replace("_", " ").title()} test failed',
                    'recommendation': f'Replace or repair {test_name.replace("_", " ")} components'
                })
            elif result['status'] == 'warning':
                issues.append({
                    'test': test_name,
                    'severity': 'medium',
                    'description': f'{test_name.replace("_", " ").title()} test shows warning',
                    'recommendation': f'Calibrate or adjust {test_name.replace("_", " ")} settings'
                })
        
        return issues
    
    def _generate_recommendations(self, tests: Dict[str, Dict]) -> List[str]:
        """Generate actionable recommendations"""
        recommendations = []
        
        for test_name, result in tests.items():
            if result['status'] == 'fail':
                recommendations.append(f"URGENT: Fix {test_name.replace('_', ' ')} issues")
            elif result['status'] == 'warning':
                recommendations.append(f"Consider calibrating {test_name.replace('_', ' ')}")
        
        if not recommendations:
            recommendations.append("Device is functioning optimally")
        
        return recommendations


class InputDeviceCalibrator:
    """Comprehensive calibration system for input devices"""
    
    def __init__(self, device_manager: DeviceManager):
        self.device_manager = device_manager
        self.diagnostic_tool = InputDeviceDiagnostic()
        
        # Calibration templates
        self.calibration_templates = {
            'touch_screen': {
                'points': [
                    (100, 100), (1820, 100), (1820, 980), (100, 980), (960, 540)
                ],
                'instructions': [
                    "Touch the top-left corner",
                    "Touch the top-right corner", 
                    "Touch the bottom-right corner",
                    "Touch the bottom-left corner",
                    "Touch the center of the screen"
                ]
            },
            'stylus': {
                'points': [
                    (200, 200), (1720, 200), (1720, 880), (200, 880),
                    (960, 200), (960, 540), (960, 880), (200, 540)
                ],
                'instructions': [
                    "Draw a dot in the top-left",
                    "Draw a dot in the top-right",
                    "Draw a dot in the bottom-right", 
                    "Draw a dot in the bottom-left",
                    "Draw a dot in the top-center",
                    "Draw a dot in the center",
                    "Draw a dot in the bottom-center",
                    "Draw a dot in the left-center"
                ]
            },
            'motion_sensor': {
                'points': [(0, 0, 9.81), (0, 9.81, 0), (9.81, 0, 0)],
                'instructions': [
                    "Place device flat on table",
                    "Hold device vertically",
                    "Hold device horizontally"
                ]
            }
        }
        
        self.logger = logging.getLogger("input.calibration")
    
    def calibrate_device(self, device: InputDevice, 
                        calibration_type: CalibrationType = CalibrationType.BASIC) -> CalibrationResults:
        """Calibrate a specific device"""
        device_type = device.device_type
        template = self.calibration_templates.get(device_type)
        
        if not template:
            raise ValueError(f"No calibration template found for device type: {device_type}")
        
        # Create calibration results
        results = CalibrationResults(
            device_id=device.device_id,
            device_type=device_type,
            calibration_type=calibration_type,
            status=CalibrationStatus.IN_PROGRESS,
            start_time=time.time()
        )
        
        self.logger.info(f"Starting {calibration_type.value} calibration for {device.device_id}")
        
        try:
            if calibration_type == CalibrationType.BASIC:
                results = self._basic_calibration(device, template, results)
            elif calibration_type == CalibrationType.ADVANCED:
                results = self._advanced_calibration(device, template, results)
            elif calibration_type == CalibrationType.FULL:
                results = self._full_calibration(device, template, results)
            elif calibration_type == CalibrationType.EDUCATIONAL:
                results = self._educational_calibration(device, template, results)
            
            # Calculate statistics
            results.calculate_statistics()
            
            # Apply calibration if successful
            if results.status == CalibrationStatus.COMPLETED:
                self._apply_calibration_results(device, results)
            
            self.logger.info(f"Calibration completed for {device.device_id}")
            
        except Exception as e:
            self.logger.error(f"Calibration failed for {device.device_id}: {e}")
            results.status = CalibrationStatus.FAILED
        
        results.end_time = time.time()
        results.calculate_statistics()
        
        return results
    
    def _basic_calibration(self, device: InputDevice, template: Dict, 
                          results: CalibrationResults) -> CalibrationResults:
        """Perform basic calibration"""
        # Run basic calibration procedure
        for i, (x, y) in enumerate(template['points'][:5]):  # Use first 5 points
            point = CalibrationPoint(expected_x=x, expected_y=y)
            
            # Simulate user input collection
            # In real implementation, this would wait for user interaction
            time.sleep(2)  # Simulate user touching point
            
            # Simulate reading actual position
            import random
            point.actual_x = x + random.gauss(0, 5)  # Add some noise
            point.actual_y = y + random.gauss(0, 5)
            point.calculate_error()
            
            results.points.append(point)
        
        results.status = CalibrationStatus.COMPLETED
        return results
    
    def _advanced_calibration(self, device: InputDevice, template: Dict,
                            results: CalibrationResults) -> CalibrationResults:
        """Perform advanced calibration"""
        # Use all calibration points
        for i, (x, y) in enumerate(template['points']):
            point = CalibrationPoint(expected_x=x, expected_y=y)
            
            # Simulate more precise calibration
            time.sleep(1.5)
            
            import random
            point.actual_x = x + random.gauss(0, 3)  # Less noise
            point.actual_y = y + random.gauss(0, 3)
            point.calculate_error()
            
            results.points.append(point)
        
        results.status = CalibrationStatus.COMPLETED
        return results
    
    def _full_calibration(self, device: InputDevice, template: Dict,
                         results: CalibrationResults) -> CalibrationResults:
        """Perform full calibration including bias correction"""
        # Use all points and multiple measurements
        for i, (x, y) in enumerate(template['points']):
            for j in range(3):  # Multiple measurements
                point = CalibrationPoint(expected_x=x, expected_y=y)
                time.sleep(1)
                
                import random
                point.actual_x = x + random.gauss(0, 2)  # Even less noise
                point.actual_y = y + random.gauss(0, 2)
                point.calculate_error()
                
                results.points.append(point)
        
        results.status = CalibrationStatus.COMPLETED
        return results
    
    def _educational_calibration(self, device: InputDevice, template: Dict,
                               results: CalibrationResults) -> CalibrationResults:
        """Perform educational calibration with guided instructions"""
        self.logger.info("Starting educational calibration with guided instructions")
        
        # Provide educational context for each point
        for i, ((x, y), instruction) in enumerate(zip(template['points'], template['instructions'])):
            point = CalibrationPoint(expected_x=x, expected_y=y)
            
            # Educational instruction
            self.logger.info(f"Point {i+1}: {instruction}")
            
            # Simulate guided interaction
            time.sleep(3)  # Give time for learning
            
            import random
            point.actual_x = x + random.gauss(0, 4)
            point.actual_y = y + random.gauss(0, 4)
            point.calculate_error()
            
            # Provide feedback
            if point.error_distance < 10:
                self.logger.info("Excellent! Very precise touch.")
            elif point.error_distance < 25:
                self.logger.info("Good touch! Calibration improving.")
            else:
                self.logger.info("Try to touch more precisely for better calibration.")
            
            results.points.append(point)
        
        results.status = CalibrationStatus.COMPLETED
        return results
    
    def _apply_calibration_results(self, device: InputDevice, results: CalibrationResults):
        """Apply calibration results to device"""
        if results.status != CalibrationStatus.COMPLETED:
            return
        
        # Generate transformation matrix
        results.transformation_matrix = self._generate_transformation_matrix(results.points)
        
        # Store calibration data in device
        device.calibration_data.update({
            'calibration_timestamp': results.start_time,
            'transformation_matrix': results.transformation_matrix,
            'mean_error': results.mean_error,
            'max_error': results.max_error,
            'success_rate': results.success_rate,
            'accuracy_score': results.accuracy_score
        })
        
        # Mark device as calibrated
        device.is_calibrated = True
        
        self.logger.info(f"Applied calibration to {device.device_id} (Accuracy: {results.accuracy_score:.1f}%)")
    
    def _generate_transformation_matrix(self, points: List[CalibrationPoint]) -> List[List[float]]:
        """Generate transformation matrix from calibration points"""
        # Simplified matrix generation - in real implementation would use proper math
        # to map screen coordinates to device coordinates
        
        if not points:
            return [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
        
        # Calculate average scaling factors
        x_errors = [point.error_x for point in points if point.actual_x is not None]
        y_errors = [point.error_y for point in points if point.actual_y is not None]
        
        x_scale = 1.0 - (sum(x_errors) / len(x_errors) / 1000) if x_errors else 1.0
        y_scale = 1.0 - (sum(y_errors) / len(y_errors) / 1000) if y_errors else 1.0
        
        x_offset = sum(x_errors) / len(x_errors) if x_errors else 0
        y_offset = sum(y_errors) / len(y_errors) if y_errors else 0
        
        return [
            [x_scale, 0, -x_offset],
            [0, y_scale, -y_offset],
            [0, 0, 1]
        ]
    
    def batch_calibrate(self, device_ids: Optional[List[str]] = None,
                       calibration_type: CalibrationType = CalibrationType.BASIC) -> Dict[str, CalibrationResults]:
        """Calibrate multiple devices"""
        devices = []
        
        if device_ids:
            for device_id in device_ids:
                device = self.device_manager.get_device(device_id)
                if device:
                    devices.append(device)
        else:
            devices = self.device_manager.get_all_devices()
        
        results = {}
        
        for device in devices:
            try:
                self.logger.info(f"Calibrating {device.device_id}")
                result = self.calibrate_device(device, calibration_type)
                results[device.device_id] = result
            except Exception as e:
                self.logger.error(f"Failed to calibrate {device.device_id}: {e}")
                results[device.device_id] = CalibrationResults(
                    device_id=device.device_id,
                    device_type=device.device_type,
                    calibration_type=calibration_type,
                    status=CalibrationStatus.FAILED,
                    start_time=time.time(),
                    end_time=time.time()
                )
        
        return results
    
    def auto_calibrate_all(self) -> Dict[str, Any]:
        """Automatically calibrate all connected devices"""
        connected_devices = self.device_manager.get_connected_devices()
        results = {}
        
        for device in connected_devices:
            try:
                # Run diagnostics first
                diagnostics = self.diagnostic_tool.run_diagnostics(device)
                
                # Determine calibration type based on diagnostics
                if diagnostics['overall_score'] < 70:
                    calibration_type = CalibrationType.FULL
                elif diagnostics['overall_score'] < 85:
                    calibration_type = CalibrationType.ADVANCED
                else:
                    calibration_type = CalibrationType.BASIC
                
                # Perform calibration
                result = self.calibrate_device(device, calibration_type)
                results[device.device_id] = {
                    'diagnostics': diagnostics,
                    'calibration': result,
                    'status': 'success' if result.status == CalibrationStatus.COMPLETED else 'failed'
                }
                
            except Exception as e:
                self.logger.error(f"Auto calibration failed for {device.device_id}: {e}")
                results[device.device_id] = {
                    'diagnostics': None,
                    'calibration': None,
                    'status': 'error',
                    'error': str(e)
                }
        
        return results
    
    def get_calibration_report(self, device_id: str) -> Dict[str, Any]:
        """Get detailed calibration report for device"""
        device = self.device_manager.get_device(device_id)
        if not device:
            return {'error': f'Device {device_id} not found'}
        
        # Run diagnostics
        diagnostics = self.diagnostic_tool.run_diagnostics(device)
        
        # Get current calibration status
        calibration_status = {
            'is_calibrated': device.is_calibrated,
            'calibration_data': device.calibration_data,
            'last_calibration': device.calibration_data.get('calibration_timestamp'),
            'accuracy_score': device.calibration_data.get('accuracy_score', 0)
        }
        
        return {
            'device_info': {
                'device_id': device_id,
                'device_type': device.device_type,
                'capabilities': {
                    'sampling_rate': device.get_capabilities().sampling_rate,
                    'accuracy': device.get_capabilities().accuracy,
                    'pressure_sensitive': device.get_capabilities().pressure_sensitive,
                    'gesture_support': device.get_capabilities().gesture_support
                }
            },
            'diagnostics': diagnostics,
            'calibration_status': calibration_status,
            'recommendations': self._generate_device_recommendations(device, diagnostics)
        }
    
    def _generate_device_recommendations(self, device: InputDevice, diagnostics: Dict) -> List[str]:
        """Generate device-specific recommendations"""
        recommendations = []
        
        score = diagnostics['overall_score']
        
        if score < 50:
            recommendations.append("URGENT: Device requires immediate calibration and repair")
        elif score < 70:
            recommendations.append("Device calibration recommended")
        elif score < 85:
            recommendations.append("Device performing well, minor calibration suggested")
        else:
            recommendations.append("Device performing optimally")
        
        # Check for specific issues
        for issue in diagnostics.get('issues_found', []):
            recommendations.append(f"Fix {issue['test']}: {issue['recommendation']}")
        
        return recommendations
    
    def validate_calibration(self, device: InputDevice) -> Dict[str, Any]:
        """Validate current calibration quality"""
        if not device.is_calibrated:
            return {'status': 'not_calibrated', 'error': 'Device has not been calibrated'}
        
        validation_results = {
            'status': 'valid',
            'checks': {},
            'overall_quality': 'good'
        }
        
        calibration_data = device.calibration_data
        validation_results['checks'] = {
            'accuracy_score': calibration_data.get('accuracy_score', 0),
            'max_error': calibration_data.get('max_error', 0),
            'success_rate': calibration_data.get('success_rate', 0),
            'recent_calibration': time.time() - calibration_data.get('calibration_timestamp', 0) < 86400  # 24 hours
        }
        
        # Determine overall quality
        accuracy = validation_results['checks']['accuracy_score']
        max_error = validation_results['checks']['max_error']
        
        if accuracy > 90 and max_error < 20:
            validation_results['overall_quality'] = 'excellent'
        elif accuracy > 80 and max_error < 30:
            validation_results['overall_quality'] = 'good'
        elif accuracy > 70 and max_error < 50:
            validation_results['overall_quality'] = 'acceptable'
        else:
            validation_results['overall_quality'] = 'poor'
            validation_results['status'] = 'invalid'
        
        return validation_results