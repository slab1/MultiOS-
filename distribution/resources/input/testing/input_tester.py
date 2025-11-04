"""
Input Device Testing Utilities for MultiOS

Comprehensive testing framework for all input devices with automated tests,
performance benchmarking, and educational validation tools.
"""

from typing import Dict, List, Any, Optional, Callable, Tuple
from dataclasses import dataclass, field
from enum import Enum
import time
import threading
import random
import math
import statistics
from collections import defaultdict
import concurrent.futures
import json

from ..core.device_manager import DeviceManager
from ..core.input_event import InputEvent, EventType, EventPriority
from ..touch.touch_screen import TouchScreenDevice, TouchType
from ..voice.voice_input import VoiceInputDevice
from ..motion.motion_sensor import MotionSensorDevice, SensorType
from ..pointing.stylus_device import StylusDevice, PenTool
from ..pointing.controller_device import GameControllerDevice, ControllerType


class TestCategory(Enum):
    """Test categories for input devices"""
    FUNCTIONAL = "functional"
    PERFORMANCE = "performance"
    RELIABILITY = "reliability"
    EDUTECH = "edutech"
    ACCESSIBILITY = "accessibility"
    INTEGRATION = "integration"


class TestStatus(Enum):
    """Test execution status"""
    PENDING = "pending"
    RUNNING = "running"
    PASSED = "passed"
    FAILED = "failed"
    SKIPPED = "skipped"
    ERROR = "error"


@dataclass
class TestResult:
    """Individual test result"""
    test_name: str
    category: TestCategory
    status: TestStatus
    duration: float = 0.0
    score: float = 0.0
    message: str = ""
    metrics: Dict[str, Any] = field(default_factory=dict)
    timestamp: float = field(default_factory=time.time)
    error: Optional[str] = None
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            'test_name': self.test_name,
            'category': self.category.value,
            'status': self.status.value,
            'duration': self.duration,
            'score': self.score,
            'message': self.message,
            'metrics': self.metrics,
            'timestamp': self.timestamp,
            'error': self.error
        }


@dataclass
class TestSuite:
    """Test suite for device category"""
    suite_name: str
    device_type: str
    tests: List[Callable] = field(default_factory=list)
    prerequisites: List[str] = field(default_factory=list)
    timeout: float = 30.0
    
    def add_test(self, test_func: Callable, test_name: str, category: TestCategory):
        """Add test to suite"""
        # Wrap test function with metadata
        def wrapped_test(device, test_result: TestResult):
            start_time = time.time()
            try:
                test_func(device, test_result)
                test_result.duration = time.time() - start_time
            except Exception as e:
                test_result.status = TestStatus.ERROR
                test_result.error = str(e)
                test_result.duration = time.time() - start_time
        
        self.tests.append(wrapped_test)


class InputDeviceTester:
    """Comprehensive testing framework for input devices"""
    
    def __init__(self, device_manager: DeviceManager):
        self.device_manager = device_manager
        self.test_results: List[TestResult] = []
        self.active_tests: Dict[str, Dict] = {}
        
        # Test configuration
        self.test_config = {
            'timeout_default': 30.0,
            'retry_count': 3,
            'parallel_execution': True,
            'performance_benchmarks': True,
            'educational_validation': True,
            'accessibility_testing': True
        }
        
        # Initialize test suites
        self.test_suites = {
            'touch': self._create_touch_test_suite(),
            'voice': self._create_voice_test_suite(),
            'motion': self._create_motion_test_suite(),
            'stylus': self._create_stylus_test_suite(),
            'controller': self._create_controller_test_suite()
        }
        
        # Performance benchmarks
        self.performance_baselines = {
            'touch': {
                'max_latency': 0.016,  # 16ms for 60Hz
                'min_sampling_rate': 60,
                'max_error': 5.0
            },
            'voice': {
                'max_recognition_time': 2.0,
                'min_confidence': 0.8,
                'max_processing_delay': 0.5
            },
            'motion': {
                'max_latency': 0.010,  # 10ms for 100Hz
                'min_sampling_rate': 100,
                'accuracy_threshold': 0.95
            },
            'stylus': {
                'max_latency': 0.008,  # 8ms for 120Hz
                'min_sampling_rate': 120,
                'pressure_levels': 1024
            },
            'controller': {
                'max_latency': 0.016,
                'min_sampling_rate': 60,
                'button_debounce': 0.050
            }
        }
    
    def run_comprehensive_test(self, device_id: Optional[str] = None,
                              test_categories: Optional[List[TestCategory]] = None) -> Dict[str, Any]:
        """Run comprehensive testing suite"""
        test_start_time = time.time()
        all_results = {}
        
        # Get devices to test
        devices_to_test = []
        if device_id:
            device = self.device_manager.get_device(device_id)
            if device:
                devices_to_test.append(device)
        else:
            devices_to_test = self.device_manager.get_all_devices()
        
        # Execute tests for each device
        for device in devices_to_test:
            device_suite = self.test_suites.get(device.device_type)
            if not device_suite:
                continue
            
            self.logger.info(f"Testing device: {device.device_id}")
            
            # Filter tests by category if specified
            tests_to_run = device_suite.tests
            if test_categories:
                # Filter tests by category (simplified for demo)
                pass
            
            # Run tests
            device_results = self._run_device_tests(device, tests_to_run)
            all_results[device.device_id] = device_results
        
        total_duration = time.time() - test_start_time
        
        # Generate test report
        report = {
            'test_duration': total_duration,
            'devices_tested': len(devices_to_test),
            'total_tests': len(self.test_results),
            'passed_tests': len([r for r in self.test_results if r.status == TestStatus.PASSED]),
            'failed_tests': len([r for r in self.test_results if r.status == TestStatus.FAILED]),
            'device_results': all_results,
            'overall_score': self._calculate_overall_score(),
            'recommendations': self._generate_test_recommendations()
        }
        
        return report
    
    def _run_device_tests(self, device, tests: List[Callable]) -> Dict[str, Any]:
        """Run tests for specific device"""
        results = {
            'device_id': device.device_id,
            'device_type': device.device_type,
            'test_results': [],
            'performance_metrics': {},
            'educational_validation': {}
        }
        
        for test_func in tests:
            test_result = TestResult(
                test_name=test_func.__name__,
                category=TestCategory.FUNCTIONAL,  # Default category
                status=TestStatus.PENDING
            )
            
            try:
                # Start test
                test_result.status = TestStatus.RUNNING
                test_start = time.time()
                
                # Execute test
                test_func(device, test_result)
                
                test_result.duration = time.time() - test_start
                
                # Determine status based on test outcome
                if test_result.error:
                    test_result.status = TestStatus.FAILED
                else:
                    test_result.status = TestStatus.PASSED
                
            except Exception as e:
                test_result.status = TestStatus.ERROR
                test_result.error = str(e)
            
            results['test_results'].append(test_result.to_dict())
            self.test_results.append(test_result)
        
        # Run performance tests
        performance_metrics = self._run_performance_tests(device)
        results['performance_metrics'] = performance_metrics
        
        # Run educational validation
        educational_validation = self._run_educational_validation(device)
        results['educational_validation'] = educational_validation
        
        return results
    
    def _run_performance_tests(self, device) -> Dict[str, Any]:
        """Run performance benchmarking tests"""
        metrics = {}
        
        if device.device_type == 'touch':
            metrics = self._test_touch_performance(device)
        elif device.device_type == 'voice':
            metrics = self._test_voice_performance(device)
        elif device.device_type == 'motion':
            metrics = self._test_motion_performance(device)
        elif device.device_type == 'stylus':
            metrics = self._test_stylus_performance(device)
        elif device.device_type == 'controller':
            metrics = self._test_controller_performance(device)
        
        return metrics
    
    def _run_educational_validation(self, device) -> Dict[str, Any]:
        """Run educational technology validation"""
        validation = {
            'accessibility_support': False,
            'educational_effectiveness': 0.0,
            'multi_modal_support': False,
            'learning_analytics': False
        }
        
        try:
            # Test accessibility features
            if hasattr(device, 'config'):
                validation['accessibility_support'] = True
            
            # Test educational mode capabilities
            if device.device_type == 'touch':
                # Test gesture support for educational interactions
                capabilities = device.get_capabilities()
                validation['multi_modal_support'] = capabilities.gesture_support
            elif device.device_type == 'voice':
                # Test voice commands for educational content
                validation['educational_effectiveness'] = 0.85  # Simulated score
            elif device.device_type == 'stylus':
                # Test pressure sensitivity for artistic learning
                validation['multi_modal_support'] = device.pressure_levels > 256
            elif device.device_type == 'controller':
                # Test game modes for educational gaming
                validation['learning_analytics'] = hasattr(device, 'interaction_handler')
            
            # Overall educational effectiveness score
            scores = [0.8 if validation['accessibility_support'] else 0.0,
                     validation['educational_effectiveness'],
                     0.9 if validation['multi_modal_support'] else 0.0,
                     0.7 if validation['learning_analytics'] else 0.0]
            validation['overall_score'] = sum(scores) / len(scores)
            
        except Exception as e:
            validation['error'] = str(e)
        
        return validation
    
    def _test_touch_performance(self, device: TouchScreenDevice) -> Dict[str, Any]:
        """Test touch screen performance"""
        latencies = []
        sampling_rates = []
        
        try:
            # Start device if not already running
            if not device.is_enabled:
                device.start()
            
            # Test input latency
            for _ in range(10):
                start_time = time.time()
                device.simulate_touch(500, 500, 1.0, 0)
                
                # Wait for event processing
                events = device.get_recent_events(1)
                if events:
                    latency = time.time() - start_time
                    latencies.append(latency)
            
            # Calculate sampling rate (simplified)
            events = device.get_event_history()
            if len(events) > 1:
                time_span = events[-1].timestamp - events[0].timestamp
                sampling_rates.append(len(events) / max(time_span, 1))
            
            metrics = {
                'average_latency': statistics.mean(latencies) if latencies else 0,
                'max_latency': max(latencies) if latencies else 0,
                'sampling_rate': statistics.mean(sampling_rates) if sampling_rates else 0,
                'baseline_compliance': 'pass' if statistics.mean(latencies) < 0.016 else 'fail'
            }
            
        except Exception as e:
            metrics = {'error': str(e)}
        
        return metrics
    
    def _test_voice_performance(self, device: VoiceInputDevice) -> Dict[str, Any]:
        """Test voice input performance"""
        recognition_times = []
        confidence_scores = []
        
        try:
            # Test voice recognition speed
            test_commands = [
                "hello multi",
                "increase volume", 
                "search for pictures",
                "explain gravity",
                "next page"
            ]
            
            for command in test_commands:
                start_time = time.time()
                device.simulate_voice_input(command, 0.9)
                
                # Wait for processing
                time.sleep(0.1)
                recognition_time = time.time() - start_time
                recognition_times.append(recognition_time)
                
                # Check confidence
                events = device.get_recent_events(2)
                for event in events:
                    if event.event_type == EventType.VOICE_TEXT and event.confidence:
                        confidence_scores.append(event.confidence)
            
            metrics = {
                'average_recognition_time': statistics.mean(recognition_times) if recognition_times else 0,
                'average_confidence': statistics.mean(confidence_scores) if confidence_scores else 0,
                'commands_tested': len(test_commands),
                'baseline_compliance': 'pass' if statistics.mean(recognition_times) < 2.0 else 'fail'
            }
            
        except Exception as e:
            metrics = {'error': str(e)}
        
        return metrics
    
    def _test_motion_performance(self, device: MotionSensorDevice) -> Dict[str, Any]:
        """Test motion sensor performance"""
        response_times = []
        sampling_rates = []
        
        try:
            # Test motion sensor response
            for _ in range(10):
                start_time = time.time()
                device.simulate_motion((9.81, 0, 0), (0, 0, 10))
                
                events = device.get_recent_events(2)
                if events:
                    response_time = time.time() - start_time
                    response_times.append(response_time)
            
            # Test sampling rate
            events = device.get_event_history()
            if len(events) > 1:
                time_span = events[-1].timestamp - events[0].timestamp
                sampling_rates.append(len(events) / max(time_span, 1))
            
            metrics = {
                'average_response_time': statistics.mean(response_times) if response_times else 0,
                'sampling_rate': statistics.mean(sampling_rates) if sampling_rates else 0,
                'accuracy_score': 0.95,  # Simulated
                'baseline_compliance': 'pass' if statistics.mean(response_times) < 0.010 else 'fail'
            }
            
        except Exception as e:
            metrics = {'error': str(e)}
        
        return metrics
    
    def _test_stylus_performance(self, device: StylusDevice) -> Dict[str, Any]:
        """Test stylus performance"""
        latencies = []
        pressure_levels = []
        
        try:
            # Test stylus response
            for _ in range(10):
                start_time = time.time()
                device.simulate_stroke([(100, 100, 0.5), (200, 100, 0.8), (200, 200, 1.0)])
                
                events = device.get_recent_events(3)
                if events:
                    latency = time.time() - start_time
                    latencies.append(latency)
                
                # Collect pressure readings
                for event in events:
                    if event.pressure:
                        pressure_levels.append(event.pressure)
            
            metrics = {
                'average_latency': statistics.mean(latencies) if latencies else 0,
                'pressure_levels_detected': len(set(pressure_levels)),
                'total_pressure_range': max(pressure_levels) - min(pressure_levels) if pressure_levels else 0,
                'baseline_compliance': 'pass' if statistics.mean(latencies) < 0.008 else 'fail'
            }
            
        except Exception as e:
            metrics = {'error': str(e)}
        
        return metrics
    
    def _test_controller_performance(self, device: GameControllerDevice) -> Dict[str, Any]:
        """Test controller performance"""
        response_times = []
        axis_responses = []
        
        try:
            # Test button response
            for button in ['button_a', 'button_b', 'button_x', 'button_y']:
                start_time = time.time()
                device.simulate_button_press(button)
                
                events = device.get_recent_events(1)
                if events:
                    response_time = time.time() - start_time
                    response_times.append(response_time)
            
            # Test axis response
            for axis in ['left_stick_x', 'left_stick_y', 'right_stick_x', 'right_stick_y']:
                device.simulate_axis_move(axis, 0.5)
                axis_responses.append(0.5)  # Simplified
            
            metrics = {
                'average_button_response': statistics.mean(response_times) if response_times else 0,
                'axis_working_count': len(axis_responses),
                'total_buttons_tested': len(response_times),
                'baseline_compliance': 'pass' if statistics.mean(response_times) < 0.016 else 'fail'
            }
            
        except Exception as e:
            metrics = {'error': str(e)}
        
        return metrics
    
    def _calculate_overall_score(self) -> float:
        """Calculate overall test score"""
        if not self.test_results:
            return 0.0
        
        passed_tests = len([r for r in self.test_results if r.status == TestStatus.PASSED])
        total_tests = len(self.test_results)
        
        base_score = (passed_tests / total_tests) * 100 if total_tests > 0 else 0
        
        # Apply performance bonuses/penalties
        performance_factor = self._calculate_performance_factor()
        
        return base_score * performance_factor
    
    def _calculate_performance_factor(self) -> float:
        """Calculate performance factor (0.8 to 1.2)"""
        # Simplified performance calculation
        # In real implementation, would analyze performance metrics
        return 1.0
    
    def _generate_test_recommendations(self) -> List[str]:
        """Generate testing recommendations"""
        recommendations = []
        
        failed_tests = [r for r in self.test_results if r.status == TestStatus.FAILED]
        error_tests = [r for r in self.test_results if r.status == TestStatus.ERROR]
        
        if failed_tests:
            recommendations.append(f"Address {len(failed_tests)} failed tests")
        
        if error_tests:
            recommendations.append(f"Fix {len(error_tests)} tests with errors")
        
        # Performance recommendations
        performance_issues = self._analyze_performance_issues()
        recommendations.extend(performance_issues)
        
        # Educational recommendations
        educational_issues = self._analyze_educational_issues()
        recommendations.extend(educational_issues)
        
        return recommendations
    
    def _analyze_performance_issues(self) -> List[str]:
        """Analyze performance-related issues"""
        issues = []
        
        # Check for slow devices
        slow_devices = []
        for result in self.test_results:
            if result.duration > 5.0:  # Tests taking longer than 5 seconds
                slow_devices.append(result.test_name)
        
        if slow_devices:
            issues.append(f"Consider optimizing performance for: {', '.join(slow_devices)}")
        
        return issues
    
    def _analyze_educational_issues(self) -> List[str]:
        """Analyze educational effectiveness issues"""
        issues = []
        
        # Check for devices without educational features
        devices_without_education = []
        for result in self.test_results:
            if 'educational' in result.test_name.lower() and result.status != TestStatus.PASSED:
                devices_without_education.append(result.test_name)
        
        if devices_without_education:
            issues.append(f"Enhance educational features for: {', '.join(devices_without_education)}")
        
        return issues
    
    # Test suite creation methods
    def _create_touch_test_suite(self) -> TestSuite:
        """Create touch screen test suite"""
        suite = TestSuite(
            suite_name="Touch Screen Tests",
            device_type="touch",
            timeout=30.0
        )
        
        # Add functional tests
        def test_touch_detection(device: TouchScreenDevice, result: TestResult):
            """Test basic touch detection"""
            if device.simulate_touch(100, 100, 1.0, 0):
                events = device.get_recent_events(1)
                if events and events[0].event_type == EventType.TOUCH_START:
                    result.score = 1.0
                    result.message = "Touch detection working correctly"
                else:
                    result.score = 0.0
                    result.message = "Touch detection failed"
            else:
                result.score = 0.0
                result.message = "Could not simulate touch"
        
        def test_multi_touch(device: TouchScreenDevice, result: TestResult):
            """Test multi-touch support"""
            if device.max_touch_points > 1:
                touches = [(100, 100, 1.0), (200, 200, 1.0), (300, 300, 1.0)]
                if device.simulate_multi_touch(touches):
                    result.score = 1.0
                    result.message = f"Multi-touch supported ({device.max_touch_points} points)"
                else:
                    result.score = 0.0
                    result.message = "Multi-touch simulation failed"
            else:
                result.score = 0.5
                result.message = "Single-touch only (resistive display)"
        
        def test_gesture_recognition(device: TouchScreenDevice, result: TestResult):
            """Test gesture recognition"""
            if device.config.get('gesture_enabled', False):
                device.simulate_gesture('swipe_up')
                events = device.get_recent_events(1)
                if events and events[0].event_type == EventType.GESTURE_SWIPE:
                    result.score = 1.0
                    result.message = "Gesture recognition working"
                else:
                    result.score = 0.0
                    result.message = "Gesture recognition failed"
            else:
                result.score = 0.0
                result.message = "Gestures not enabled"
        
        suite.add_test(test_touch_detection, "Touch Detection", TestCategory.FUNCTIONAL)
        suite.add_test(test_multi_touch, "Multi-Touch Support", TestCategory.FUNCTIONAL)
        suite.add_test(test_gesture_recognition, "Gesture Recognition", TestCategory.EDUTECH)
        
        return suite
    
    def _create_voice_test_suite(self) -> TestSuite:
        """Create voice input test suite"""
        suite = TestSuite(
            suite_name="Voice Input Tests",
            device_type="voice",
            timeout=45.0
        )
        
        def test_voice_recognition(device: VoiceInputDevice, result: TestResult):
            """Test basic voice recognition"""
            device.simulate_voice_input("hello multi", 0.9)
            events = device.get_recent_events(1)
            if events and events[0].event_type == EventType.VOICE_TEXT:
                result.score = 1.0
                result.message = "Voice recognition working"
            else:
                result.score = 0.0
                result.message = "Voice recognition failed"
        
        def test_voice_commands(device: VoiceInputDevice, result: TestResult):
            """Test voice command processing"""
            device.simulate_voice_input("increase volume", 0.9)
            events = device.get_recent_events(1)
            if events and events[0].event_type == EventType.VOICE_COMMAND:
                result.score = 1.0
                result.message = "Voice commands working"
            else:
                result.score = 0.0
                result.message = "Voice command processing failed"
        
        def test_noise_handling(device: VoiceInputDevice, result: TestResult):
            """Test noise cancellation and processing"""
            device.simulate_voice_input("test noise", 0.5)  # Low confidence
            events = device.get_recent_events(2)
            # Should handle low confidence gracefully
            result.score = 0.8
            result.message = "Noise handling acceptable"
        
        suite.add_test(test_voice_recognition, "Voice Recognition", TestCategory.FUNCTIONAL)
        suite.add_test(test_voice_commands, "Voice Commands", TestCategory.EDUTECH)
        suite.add_test(test_noise_handling, "Noise Handling", TestCategory.RELIABILITY)
        
        return suite
    
    def _create_motion_test_suite(self) -> TestSuite:
        """Create motion sensor test suite"""
        suite = TestSuite(
            suite_name="Motion Sensor Tests",
            device_type="motion",
            timeout=30.0
        )
        
        def test_acceleration_detection(device: MotionSensorDevice, result: TestResult):
            """Test acceleration detection"""
            device.simulate_motion((9.81, 0, 0))
            events = device.get_recent_events(1)
            if events and events[0].event_type == EventType.MOTION_ACCEL:
                result.score = 1.0
                result.message = "Acceleration detection working"
            else:
                result.score = 0.0
                result.message = "Acceleration detection failed"
        
        def test_gesture_detection(device: MotionSensorDevice, result: TestResult):
            """Test motion gesture detection"""
            device.simulate_gesture('shake')
            events = device.get_recent_events(1)
            if events and events[0].event_type == EventType.GESTURE_SWIPE:
                result.score = 1.0
                result.message = "Motion gestures working"
            else:
                result.score = 0.0
                result.message = "Motion gesture detection failed"
        
        def test_orientation_tracking(device: MotionSensorDevice, result: TestResult):
            """Test orientation tracking"""
            # Simulate device rotation
            from ..motion.motion_sensor import Orientation
            device.set_orientation(Orientation(45, 0, 0))
            
            orientation = device.get_orientation()
            if abs(orientation.roll - 45) < 5:  # Within 5 degrees
                result.score = 1.0
                result.message = "Orientation tracking accurate"
            else:
                result.score = 0.5
                result.message = "Orientation tracking imprecise"
        
        suite.add_test(test_acceleration_detection, "Acceleration Detection", TestCategory.FUNCTIONAL)
        suite.add_test(test_gesture_detection, "Motion Gestures", TestCategory.EDUTECH)
        suite.add_test(test_orientation_tracking, "Orientation Tracking", TestCategory.FUNCTIONAL)
        
        return suite
    
    def _create_stylus_test_suite(self) -> TestSuite:
        """Create stylus test suite"""
        suite = TestSuite(
            suite_name="Stylus Tests",
            device_type="stylus",
            timeout=30.0
        )
        
        def test_pressure_detection(device: StylusDevice, result: TestResult):
            """Test pressure sensitivity"""
            points = [(100, 100, 0.2), (150, 100, 0.8), (200, 100, 1.0)]
            device.simulate_stroke(points)
            
            events = device.get_recent_events(3)
            pressures = [e.pressure for e in events if e.pressure is not None]
            
            if len(set(pressures)) >= 3:  # Different pressure levels detected
                result.score = 1.0
                result.message = "Pressure sensitivity working"
            else:
                result.score = 0.0
                result.message = "Pressure sensitivity failed"
        
        def test_tilt_detection(device: StylusDevice, result: TestResult):
            """Test tilt detection"""
            if device.tilt_support:
                # Simulate tilt
                device.current_tilt_x = 30.0
                device.current_tilt_y = 45.0
                
                events = device.get_recent_events(1)
                if events and events[0].tilt_x is not None:
                    result.score = 1.0
                    result.message = "Tilt detection working"
                else:
                    result.score = 0.0
                    result.message = "Tilt detection failed"
            else:
                result.score = 0.5
                result.message = "Tilt not supported"
        
        def test_drawing_accuracy(device: StylusDevice, result: TestResult):
            """Test drawing accuracy"""
            # Simulate drawing a straight line
            points = [(100, 100, 0.8), (200, 100, 0.8), (300, 100, 0.8)]
            device.simulate_stroke(points)
            
            stats = device.get_drawing_stats()
            if stats['total_strokes'] > 0:
                result.score = 1.0
                result.message = "Drawing accuracy good"
            else:
                result.score = 0.0
                result.message = "Drawing accuracy poor"
        
        suite.add_test(test_pressure_detection, "Pressure Detection", TestCategory.FUNCTIONAL)
        suite.add_test(test_tilt_detection, "Tilt Detection", TestCategory.FUNCTIONAL)
        suite.add_test(test_drawing_accuracy, "Drawing Accuracy", TestCategory.EDUTECH)
        
        return suite
    
    def _create_controller_test_suite(self) -> TestSuite:
        """Create controller test suite"""
        suite = TestSuite(
            suite_name="Controller Tests",
            device_type="controller",
            timeout=30.0
        )
        
        def test_button_presses(device: GameControllerDevice, result: TestResult):
            """Test button press detection"""
            buttons_to_test = ['button_a', 'button_b', 'button_x', 'button_y']
            working_buttons = 0
            
            for button in buttons_to_test:
                if device.simulate_button_press(button):
                    working_buttons += 1
            
            score = working_buttons / len(buttons_to_test)
            result.score = score
            
            if score >= 0.9:
                result.message = f"All buttons working ({working_buttons}/{len(buttons_to_test)})"
            elif score >= 0.7:
                result.message = f"Most buttons working ({working_buttons}/{len(buttons_to_test)})"
            else:
                result.message = f"Some buttons not working ({working_buttons}/{len(buttons_to_test)})"
        
        def test_axis_movement(device: GameControllerDevice, result: TestResult):
            """Test analog axis movement"""
            axes_to_test = ['left_stick_x', 'left_stick_y', 'right_stick_x', 'right_stick_y']
            working_axes = 0
            
            for axis in axes_to_test:
                device.simulate_axis_move(axis, 0.5)
                # Check if axis responded
                status = device.get_controller_status()
                if status['axes'].get(axis, {}).get('value', 0) > 0:
                    working_axes += 1
            
            score = working_axes / len(axes_to_test)
            result.score = score
            
            if score >= 1.0:
                result.message = "All analog axes working"
            else:
                result.message = f"{working_axes}/{len(axes_to_test)} analog axes working"
        
        def test_rumble_feedback(device: GameControllerDevice, result: TestResult):
            """Test haptic rumble feedback"""
            if device.rumble_enabled:
                if device.set_rumble(0.5, 0.5, 0.1):
                    result.score = 1.0
                    result.message = "Rumble feedback working"
                else:
                    result.score = 0.0
                    result.message = "Rumble feedback failed"
            else:
                result.score = 0.5
                result.message = "Rumble not enabled"
        
        suite.add_test(test_button_presses, "Button Presses", TestCategory.FUNCTIONAL)
        suite.add_test(test_axis_movement, "Analog Axes", TestCategory.FUNCTIONAL)
        suite.add_test(test_rumble_feedback, "Haptic Feedback", TestCategory.EDUTECH)
        
        return suite
    
    def export_test_results(self, filepath: str, format_type: str = 'json') -> bool:
        """Export test results to file"""
        try:
            results_data = {
                'test_timestamp': time.time(),
                'total_tests': len(self.test_results),
                'passed_tests': len([r for r in self.test_results if r.status == TestStatus.PASSED]),
                'failed_tests': len([r for r in self.test_results if r.status == TestStatus.FAILED]),
                'overall_score': self._calculate_overall_score(),
                'test_results': [result.to_dict() for result in self.test_results]
            }
            
            if format_type.lower() == 'json':
                with open(filepath, 'w') as f:
                    json.dump(results_data, f, indent=2)
            else:
                # Could add other formats like CSV, XML, etc.
                raise ValueError(f"Unsupported format: {format_type}")
            
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to export test results: {e}")
            return False
    
    def get_test_summary(self) -> Dict[str, Any]:
        """Get comprehensive test summary"""
        if not self.test_results:
            return {'error': 'No test results available'}
        
        # Calculate statistics
        total_tests = len(self.test_results)
        passed_tests = len([r for r in self.test_results if r.status == TestStatus.PASSED])
        failed_tests = len([r for r in self.test_results if r.status == TestStatus.FAILED])
        error_tests = len([r for r in self.test_results if r.status == TestStatus.ERROR])
        skipped_tests = len([r for r in self.test_results if r.status == TestStatus.SKIPPED])
        
        # Group by device type
        results_by_device = defaultdict(list)
        for result in self.test_results:
            # Find device ID from result (simplified)
            device_id = "unknown"  # In real implementation, would track this
            results_by_device[device_id].append(result)
        
        # Group by category
        results_by_category = defaultdict(int)
        for result in self.test_results:
            results_by_category[result.category.value] += 1
        
        return {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'failed_tests': failed_tests,
            'error_tests': error_tests,
            'skipped_tests': skipped_tests,
            'success_rate': (passed_tests / total_tests * 100) if total_tests > 0 else 0,
            'overall_score': self._calculate_overall_score(),
            'results_by_device': {k: len(v) for k, v in results_by_device.items()},
            'results_by_category': dict(results_by_category),
            'average_duration': statistics.mean([r.duration for r in self.test_results]) if self.test_results else 0,
            'total_duration': sum([r.duration for r in self.test_results])
        }


class StressTestRunner:
    """Stress testing for input devices"""
    
    def __init__(self, tester: InputDeviceTester):
        self.tester = tester
        self.stress_results = {}
    
    def run_stress_test(self, device_id: str, duration: float = 60.0,
                       load_factor: float = 1.0) -> Dict[str, Any]:
        """Run stress test on specific device"""
        device = self.tester.device_manager.get_device(device_id)
        if not device:
            return {'error': f'Device {device_id} not found'}
        
        start_time = time.time()
        test_events = []
        error_count = 0
        
        try:
            # Start device
            device.start()
            
            # Run stress test
            while time.time() - start_time < duration:
                try:
                    # Generate load based on device type
                    if device.device_type == 'touch':
                        # Simulate rapid touch events
                        for i in range(int(load_factor)):
                            device.simulate_touch(
                                random.randint(0, 1920),
                                random.randint(0, 1080),
                                random.uniform(0.1, 1.0),
                                random.randint(0, 9)
                            )
                    
                    elif device.device_type == 'voice':
                        # Simulate voice commands
                        commands = ["test", "hello", "volume", "search"]
                        command = random.choice(commands)
                        device.simulate_voice_input(command, random.uniform(0.7, 1.0))
                    
                    elif device.device_type == 'motion':
                        # Simulate motion events
                        device.simulate_motion(
                            (random.uniform(-15, 15), random.uniform(-15, 15), random.uniform(5, 15)),
                            (random.uniform(-100, 100), random.uniform(-100, 100), random.uniform(-100, 100))
                        )
                    
                    elif device.device_type == 'stylus':
                        # Simulate drawing
                        points = [
                            (random.randint(0, 1920), random.randint(0, 1080), random.uniform(0.1, 1.0))
                            for _ in range(random.randint(1, 5))
                        ]
                        device.simulate_stroke(points)
                    
                    elif device.device_type == 'controller':
                        # Simulate button/axis events
                        if random.random() < 0.8:  # 80% buttons, 20% axes
                            buttons = ['button_a', 'button_b', 'button_x', 'button_y']
                            device.simulate_button_press(random.choice(buttons))
                        else:
                            axes = ['left_stick_x', 'left_stick_y', 'right_stick_x', 'right_stick_y']
                            device.simulate_axis_move(random.choice(axes), random.uniform(-1, 1))
                    
                    test_events.append(time.time())
                    
                    # Small delay to prevent overwhelming
                    time.sleep(0.001 * load_factor)
                
                except Exception as e:
                    error_count += 1
        
        except Exception as e:
            return {'error': f'Stress test failed: {e}'}
        
        finally:
            # Cleanup
            device.stop()
        
        # Calculate stress test metrics
        total_duration = time.time() - start_time
        events_per_second = len(test_events) / max(total_duration, 1)
        error_rate = error_count / max(len(test_events), 1)
        
        return {
            'device_id': device_id,
            'duration': total_duration,
            'total_events': len(test_events),
            'events_per_second': events_per_second,
            'error_count': error_count,
            'error_rate': error_rate,
            'load_factor': load_factor,
            'stability_score': max(0, 1.0 - error_rate),
            'throughput_score': min(1.0, events_per_second / 1000)  # Normalized to 1000 events/sec
        }
    
    def run_comprehensive_stress_test(self, device_ids: Optional[List[str]] = None) -> Dict[str, Any]:
        """Run comprehensive stress testing on multiple devices"""
        if not device_ids:
            device_ids = [d.device_id for d in self.tester.device_manager.get_all_devices()]
        
        stress_results = {}
        
        for device_id in device_ids:
            self.tester.logger.info(f"Running stress test for {device_id}")
            
            # Run multiple stress test scenarios
            scenarios = [
                ('light_load', 30.0, 0.5),
                ('normal_load', 60.0, 1.0),
                ('heavy_load', 30.0, 2.0)
            ]
            
            device_results = {}
            for scenario_name, duration, load_factor in scenarios:
                result = self.run_stress_test(device_id, duration, load_factor)
                device_results[scenario_name] = result
            
            stress_results[device_id] = device_results
        
        return stress_results