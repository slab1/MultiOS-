"""
MultiOS Input Device Support - Main Demonstration

This script demonstrates the complete MultiOS input device support system
with educational interactions and comprehensive testing.
"""

import sys
import os
import time
import logging
import argparse
from typing import Dict, Any

# Add the parent directory to path for imports
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from hardware_support.input.core.device_manager import DeviceManager
from hardware_support.input.calibration.input_calibrator import InputDeviceCalibrator
from hardware_support.input.examples.educational_interactions import (
    EducationalInputManager, LearningActivity, MultiModalInteractionDemo
)
from hardware_support.input.testing.input_tester import (
    InputDeviceTester, TestCategory, StressTestRunner
)

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger("MultiOS_Demo")


class MultiOSInputSystemDemo:
    """Main demonstration class for MultiOS input device support"""
    
    def __init__(self):
        self.device_manager = DeviceManager()
        self.calibrator = InputDeviceCalibrator(self.device_manager)
        self.educational_manager = EducationalInputManager(self.device_manager)
        self.tester = InputDeviceTester(self.device_manager)
        self.stress_tester = StressTestRunner(self.tester)
        self.demo = MultiModalInteractionDemo(self.educational_manager)
        
        # Demo state
        self.setup_complete = False
        self.current_activity = None
    
    def run_full_demonstration(self):
        """Run complete demonstration of all features"""
        logger.info("=" * 60)
        logger.info("MultiOS Input Device Support - Full Demonstration")
        logger.info("=" * 60)
        
        # Step 1: System Setup
        logger.info("\n📱 STEP 1: System Setup and Device Discovery")
        setup_result = self._demonstrate_system_setup()
        if not setup_result['success']:
            logger.error("System setup failed, stopping demonstration")
            return
        
        # Step 2: Device Calibration
        logger.info("\n🔧 STEP 2: Device Calibration")
        calibration_result = self._demonstrate_calibration()
        
        # Step 3: Educational Interactions
        logger.info("\n🎓 STEP 3: Educational Interactions")
        educational_result = self._demonstrate_educational_features()
        
        # Step 4: Multi-Modal Integration
        logger.info("\n🔗 STEP 4: Multi-Modal Integration")
        multimodal_result = self._demonstrate_multimodal_integration()
        
        # Step 5: Comprehensive Testing
        logger.info("\n🧪 STEP 5: Comprehensive Testing")
        testing_result = self._demonstrate_testing()
        
        # Step 6: Performance Analysis
        logger.info("\n⚡ STEP 6: Performance Analysis")
        performance_result = self._demonstrate_performance_analysis()
        
        # Generate final report
        logger.info("\n📊 STEP 7: Final Report")
        final_report = self._generate_final_report({
            'setup': setup_result,
            'calibration': calibration_result,
            'educational': educational_result,
            'multimodal': multimodal_result,
            'testing': testing_result,
            'performance': performance_result
        })
        
        logger.info("\n✅ Demonstration completed successfully!")
        logger.info(f"Final Report: {final_report}")
    
    def _demonstrate_system_setup(self) -> Dict[str, Any]:
        """Demonstrate system setup and device discovery"""
        logger.info("Initializing device manager and discovering devices...")
        
        try:
            # Initialize educational setup
            setup_result = self.educational_manager.initialize_educational_setup()
            
            if setup_result['overall_status'] == 'ready':
                logger.info("✅ System setup completed successfully")
                logger.info(f"  - Discovered {len(setup_result['device_discovery'])} devices")
                logger.info(f"  - Calibrated {len(setup_result['calibration_results'])} devices")
                logger.info(f"  - Set up {len(setup_result['activity_tests'])} educational activities")
                self.setup_complete = True
                return {'success': True, 'setup_result': setup_result}
            else:
                logger.error("❌ System setup failed")
                return {'success': False, 'setup_result': setup_result}
        
        except Exception as e:
            logger.error(f"System setup error: {e}")
            return {'success': False, 'error': str(e)}
    
    def _demonstrate_calibration(self) -> Dict[str, Any]:
        """Demonstrate device calibration"""
        if not self.setup_complete:
            logger.warning("System not set up, skipping calibration demonstration")
            return {'success': False, 'error': 'System not initialized'}
        
        logger.info("Running calibration demonstration...")
        
        try:
            # Get all connected devices
            connected_devices = self.device_manager.get_connected_devices()
            
            calibration_results = {}
            total_accuracy = 0.0
            device_count = 0
            
            for device in connected_devices:
                logger.info(f"Calibrating {device.device_id}...")
                
                # Run diagnostics first
                diagnostics = self.tester.diagnostic_tool.run_diagnostics(device)
                logger.info(f"  Diagnostic score: {diagnostics['overall_score']:.1f}%")
                
                # Perform calibration
                calibration_result = self.calibrator.calibrate_device(device)
                
                calibration_results[device.device_id] = {
                    'status': calibration_result.status.value,
                    'accuracy_score': calibration_result.accuracy_score,
                    'duration': calibration_result.duration,
                    'points': len(calibration_result.points)
                }
                
                total_accuracy += calibration_result.accuracy_score
                device_count += 1
                
                logger.info(f"  ✅ {device.device_id} calibrated ({calibration_result.accuracy_score:.1f}% accuracy)")
            
            average_accuracy = total_accuracy / max(device_count, 1)
            
            logger.info(f"✅ Calibration completed - Average accuracy: {average_accuracy:.1f}%")
            
            return {
                'success': True,
                'calibration_results': calibration_results,
                'average_accuracy': average_accuracy
            }
        
        except Exception as e:
            logger.error(f"Calibration error: {e}")
            return {'success': False, 'error': str(e)}
    
    def _demonstrate_educational_features(self) -> Dict[str, Any]:
        """Demonstrate educational interaction features"""
        if not self.setup_complete:
            logger.warning("System not set up, skipping educational demonstration")
            return {'success': False, 'error': 'System not initialized'}
        
        logger.info("Demonstrating educational interactions...")
        
        try:
            # Test different learning activities
            activities_tested = []
            
            # Math Quiz with multi-modal input
            logger.info("Testing Math Quiz with multi-modal input...")
            result = self.educational_manager.start_learning_activity(LearningActivity.MATH_QUIZ)
            if result['status'] == 'started':
                activities_tested.append('Math Quiz')
                
                # Simulate interactions
                touch_device = self.device_manager.get_device('touch_0')
                voice_device = self.device_manager.get_device('voice_0')
                
                if touch_device:
                    for i in range(3):
                        touch_device.simulate_touch(200 + i*100, 300, 0.8, i)
                        time.sleep(0.3)
                        touch_device.simulate_touch_release(i)
                
                if voice_device:
                    voice_device.simulate_voice_input("the answer is five", 0.9)
                
                # Stop activity
                stop_result = self.educational_manager.stop_learning_activity()
                logger.info(f"  ✅ Math Quiz completed - {stop_result['metrics']['interaction_count']} interactions")
            
            # Science Experiment with motion simulation
            logger.info("Testing Science Experiment with motion simulation...")
            result = self.educational_manager.start_learning_activity(LearningActivity.SCIENCE_EXPERIMENT)
            if result['status'] == 'started':
                activities_tested.append('Science Experiment')
                
                motion_device = self.device_manager.get_device('motion_0')
                stylus_device = self.device_manager.get_device('stylus_0')
                
                if motion_device:
                    motion_device.simulate_motion((9.81, 0, 0), (0, 0, 0))
                    motion_device.simulate_gesture('tilt_forward')
                
                if stylus_device:
                    points = [(100, 200, 0.7), (300, 200, 0.8), (300, 400, 0.9)]
                    stylus_device.simulate_stroke(points)
                
                stop_result = self.educational_manager.stop_learning_activity()
                logger.info(f"  ✅ Science Experiment completed - {stop_result['metrics']['interaction_count']} interactions")
            
            # Art Creation with stylus
            logger.info("Testing Art Creation with stylus...")
            result = self.educational_manager.start_learning_activity(LearningActivity.ART_CREATION)
            if result['status'] == 'started':
                activities_tested.append('Art Creation')
                
                stylus_device = self.device_manager.get_device('stylus_0')
                if stylus_device:
                    # Draw a simple shape
                    points = [
                        (400, 400, 0.8), (600, 400, 0.9), (600, 600, 0.9),
                        (400, 600, 0.9), (400, 400, 0.8)
                    ]
                    stylus_device.simulate_stroke(points)
                
                stop_result = self.educational_manager.stop_learning_activity()
                logger.info(f"  ✅ Art Creation completed - {stop_result['metrics']['interaction_count']} interactions")
            
            logger.info(f"✅ Educational demonstration completed - {len(activities_tested)} activities tested")
            
            return {
                'success': True,
                'activities_tested': activities_tested,
                'total_activities': len(list(LearningActivity))
            }
        
        except Exception as e:
            logger.error(f"Educational demonstration error: {e}")
            return {'success': False, 'error': str(e)}
    
    def _demonstrate_multimodal_integration(self) -> Dict[str, Any]:
        """Demonstrate multi-modal device integration"""
        if not self.setup_complete:
            logger.warning("System not set up, skipping multimodal demonstration")
            return {'success': False, 'error': 'System not initialized'}
        
        logger.info("Demonstrating multi-modal integration...")
        
        try:
            # Run comprehensive demo scenarios
            demo_results = self.demo.run_comprehensive_demo()
            
            successful_scenarios = 0
            total_scenarios = len(demo_results)
            
            for scenario_name, result in demo_results.items():
                if result.get('status') == 'completed':
                    successful_scenarios += 1
                    logger.info(f"  ✅ {scenario_name}: {result.get('interactions_simulated', 0)} interactions")
                else:
                    logger.warning(f"  ⚠️ {scenario_name}: {result.get('status', 'failed')}")
            
            logger.info(f"✅ Multi-modal demonstration completed - {successful_scenarios}/{total_scenarios} scenarios successful")
            
            return {
                'success': True,
                'total_scenarios': total_scenarios,
                'successful_scenarios': successful_scenarios,
                'success_rate': successful_scenarios / total_scenarios if total_scenarios > 0 else 0,
                'demo_results': demo_results
            }
        
        except Exception as e:
            logger.error(f"Multi-modal demonstration error: {e}")
            return {'success': False, 'error': str(e)}
    
    def _demonstrate_testing(self) -> Dict[str, Any]:
        """Demonstrate comprehensive testing capabilities"""
        if not self.setup_complete:
            logger.warning("System not set up, skipping testing demonstration")
            return {'success': False, 'error': 'System not initialized'}
        
        logger.info("Demonstrating comprehensive testing...")
        
        try:
            # Run basic test on all devices
            logger.info("Running functional tests...")
            test_report = self.tester.run_comprehensive_test()
            
            logger.info(f"  📊 Test Results:")
            logger.info(f"    - Devices tested: {test_report['devices_tested']}")
            logger.info(f"    - Total tests: {test_report['total_tests']}")
            logger.info(f"    - Passed tests: {test_report['passed_tests']}")
            logger.info(f"    - Failed tests: {test_report['failed_tests']}")
            logger.info(f"    - Overall score: {test_report['overall_score']:.1f}%")
            
            # Show device-specific results
            for device_id, device_result in test_report['device_results'].items():
                device_passed = len([t for t in device_result['test_results'] if t['status'] == 'passed'])
                device_total = len(device_result['test_results'])
                logger.info(f"    - {device_id}: {device_passed}/{device_total} tests passed")
            
            # Run stress test on one device
            connected_devices = self.device_manager.get_connected_devices()
            if connected_devices:
                logger.info("Running stress test...")
                stress_device = connected_devices[0]
                stress_result = self.stress_tester.run_stress_test(
                    stress_device.device_id, 
                    duration=10.0,  # Short test
                    load_factor=1.0
                )
                
                logger.info(f"  ⚡ Stress Test Results ({stress_device.device_id}):")
                logger.info(f"    - Duration: {stress_result['duration']:.1f}s")
                logger.info(f"    - Events generated: {stress_result['total_events']}")
                logger.info(f"    - Events/second: {stress_result['events_per_second']:.1f}")
                logger.info(f"    - Error rate: {stress_result['error_rate']:.3f}")
                logger.info(f"    - Stability score: {stress_result['stability_score']:.2f}")
            
            # Export test results
            export_path = "/workspace/hardware_support/input/test_results.json"
            if self.tester.export_test_results(export_path):
                logger.info(f"  📄 Test results exported to: {export_path}")
            
            logger.info("✅ Testing demonstration completed")
            
            return {
                'success': True,
                'test_report': test_report,
                'stress_test': stress_result if connected_devices else None,
                'results_exported': os.path.exists(export_path)
            }
        
        except Exception as e:
            logger.error(f"Testing demonstration error: {e}")
            return {'success': False, 'error': str(e)}
    
    def _demonstrate_performance_analysis(self) -> Dict[str, Any]:
        """Demonstrate performance analysis capabilities"""
        logger.info("Analyzing system performance...")
        
        try:
            performance_data = {}
            
            # Analyze device performance
            connected_devices = self.device_manager.get_connected_devices()
            
            for device in connected_devices:
                device_stats = device.get_statistics()
                
                performance_data[device.device_id] = {
                    'event_rate': device_stats.get('event_rate', 0),
                    'uptime': device_stats.get('uptime', 0),
                    'queue_size': device_stats.get('queue_size', 0),
                    'connected': device_stats.get('is_connected', False),
                    'enabled': device_stats.get('is_enabled', False)
                }
                
                logger.info(f"  📈 {device.device_id}:")
                logger.info(f"    - Event rate: {performance_data[device.device_id]['event_rate']:.1f} Hz")
                logger.info(f"    - Uptime: {performance_data[device.device_id]['uptime']:.1f}s")
                logger.info(f"    - Queue size: {performance_data[device.device_id]['queue_size']}")
            
            # Analyze overall system performance
            manager_stats = self.device_manager.get_statistics()
            
            performance_data['system_overall'] = {
                'connected_devices': manager_stats.get('connected_devices', 0),
                'total_events': manager_stats.get('total_events', 0),
                'event_processing_rate': manager_stats.get('event_rate', 0),
                'uptime': manager_stats.get('uptime', 0)
            }
            
            logger.info(f"  📊 System Overall:")
            logger.info(f"    - Connected devices: {performance_data['system_overall']['connected_devices']}")
            logger.info(f"    - Total events processed: {performance_data['system_overall']['total_events']}")
            logger.info(f"    - Event processing rate: {performance_data['system_overall']['event_processing_rate']:.1f} Hz")
            
            # Calculate performance scores
            if performance_data['system_overall']['total_events'] > 0:
                performance_score = min(100, performance_data['system_overall']['event_processing_rate'] / 100 * 100)
            else:
                performance_score = 0
            
            logger.info(f"  🎯 Performance Score: {performance_score:.1f}%")
            
            return {
                'success': True,
                'performance_data': performance_data,
                'performance_score': performance_score
            }
        
        except Exception as e:
            logger.error(f"Performance analysis error: {e}")
            return {'success': False, 'error': str(e)}
    
    def _generate_final_report(self, demo_results: Dict[str, Any]) -> Dict[str, Any]:
        """Generate comprehensive final report"""
        logger.info("Generating final demonstration report...")
        
        # Calculate overall scores
        setup_score = 100 if demo_results['setup']['success'] else 0
        calibration_score = demo_results['calibration'].get('average_accuracy', 0) if demo_results['calibration']['success'] else 0
        educational_score = 100 if demo_results['educational']['success'] else 0
        multimodal_score = demo_results['multimodal'].get('success_rate', 0) * 100 if demo_results['multimodal']['success'] else 0
        testing_score = demo_results['testing']['test_report']['overall_score'] if demo_results['testing']['success'] else 0
        performance_score = demo_results['performance']['performance_score'] if demo_results['performance']['success'] else 0
        
        overall_score = (setup_score + calibration_score + educational_score + 
                        multimodal_score + testing_score + performance_score) / 6
        
        # Generate report
        final_report = {
            'demonstration_summary': {
                'overall_score': overall_score,
                'components_tested': len([r for r in demo_results.values() if r.get('success', False)]),
                'total_components': len(demo_results),
                'success_rate': len([r for r in demo_results.values() if r.get('success', False)]) / len(demo_results)
            },
            'component_scores': {
                'system_setup': setup_score,
                'device_calibration': calibration_score,
                'educational_features': educational_score,
                'multimodal_integration': multimodal_score,
                'testing_capabilities': testing_score,
                'performance_analysis': performance_score
            },
            'key_achievements': [
                "Modern input device support implemented",
                "Educational interaction framework created",
                "Multi-modal integration demonstrated",
                "Comprehensive testing suite provided",
                "Performance analysis capabilities added"
            ],
            'technical_highlights': [
                "Touch screen drivers (capacitive/resistive)",
                "Multi-touch gesture recognition",
                "Voice input processing framework",
                "Accelerometer and gyroscope support",
                "Game controller and VR device integration",
                "Stylus and pen input handling",
                "Input device calibration tools",
                "Educational interaction examples",
                "Automated testing utilities"
            ],
            'next_steps': [
                "Hardware integration testing",
                "Real device validation",
                "Educational content expansion",
                "Performance optimization",
                "Accessibility feature enhancement"
            ]
        }
        
        logger.info("📋 Final Report Summary:")
        logger.info(f"  Overall Score: {overall_score:.1f}%")
        logger.info(f"  Components Tested: {final_report['demonstration_summary']['components_tested']}/{final_report['demonstration_summary']['total_components']}")
        logger.info(f"  Success Rate: {final_report['demonstration_summary']['success_rate']*100:.1f}%")
        
        logger.info("\n🎯 Key Achievements:")
        for achievement in final_report['key_achievements']:
            logger.info(f"  ✅ {achievement}")
        
        logger.info("\n🔧 Technical Features Implemented:")
        for feature in final_report['technical_highlights']:
            logger.info(f"  🔧 {feature}")
        
        return final_report


def main():
    """Main demonstration entry point"""
    parser = argparse.ArgumentParser(description="MultiOS Input Device Support Demonstration")
    parser.add_argument('--activity', choices=['setup', 'educational', 'testing', 'performance', 'full'], 
                       default='full', help="Demonstration activity to run")
    parser.add_argument('--device', type=str, help="Specific device ID to test")
    parser.add_argument('--duration', type=float, default=60.0, help="Test duration in seconds")
    parser.add_argument('--verbose', action='store_true', help="Enable verbose logging")
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Create and run demo
    demo = MultiOSInputSystemDemo()
    
    try:
        if args.activity == 'setup':
            result = demo._demonstrate_system_setup()
            print(f"Setup result: {result}")
        elif args.activity == 'educational':
            result = demo._demonstrate_educational_features()
            print(f"Educational result: {result}")
        elif args.activity == 'testing':
            result = demo._demonstrate_testing()
            print(f"Testing result: {result}")
        elif args.activity == 'performance':
            result = demo._demonstrate_performance_analysis()
            print(f"Performance result: {result}")
        else:  # full
            demo.run_full_demonstration()
    
    except KeyboardInterrupt:
        logger.info("Demonstration interrupted by user")
    except Exception as e:
        logger.error(f"Demonstration failed: {e}")
        raise


if __name__ == "__main__":
    main()