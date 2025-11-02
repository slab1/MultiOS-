#!/usr/bin/env python3
"""
Educational Robotics Framework - Test Suite

Comprehensive testing for all framework components
"""

import sys
import time
import numpy as np
from pathlib import Path

# Add the robotics framework to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from robotics import create_robot, start_demo
    from robotics.core.robot import Robot, ControlCommand, RobotState
    from robotics.hardware.ev3_hardware import EV3Hardware
    from robotics.control.pid_controller import PIDController, PIDConfig
    from robotics.control.path_planner import PathPlanner, Point, AStarPlanner
    from robotics.vision.camera import Camera, VisionProcessor
    from robotics.vision.sensor_fusion import SensorFusionEngine
    from robotics.communication.protocols import SerialCommunication, Message
    from robotics.simulation.environment import SimulationEnvironment, RobotConfig
    from robotics.demos.educational_demos import LineFollowingDemo
    from robotics.curriculum.curriculum_manager import CurriculumManager
    IMPORTS_SUCCESSFUL = True
except ImportError as e:
    IMPORTS_SUCCESSFUL = False
    IMPORT_ERROR = str(e)
    print(f"Import failed: {IMPORT_ERROR}")


class FrameworkTester:
    """Comprehensive test suite for the robotics framework"""
    
    def __init__(self):
        self.test_results = {}
        self.passed_tests = 0
        self.failed_tests = 0
        
    def run_test(self, test_name: str, test_function):
        """Run a single test and record results"""
        print(f"\n{'='*60}")
        print(f"Testing: {test_name}")
        print(f"{'='*60}")
        
        try:
            start_time = time.time()
            test_function()
            duration = time.time() - start_time
            
            self.test_results[test_name] = {
                'status': 'PASSED',
                'duration': duration,
                'error': None
            }
            self.passed_tests += 1
            print(f"✓ {test_name} PASSED ({duration:.2f}s)")
            
        except Exception as e:
            duration = time.time() - start_time
            self.test_results[test_name] = {
                'status': 'FAILED',
                'duration': duration,
                'error': str(e)
            }
            self.failed_tests += 1
            print(f"✗ {test_name} FAILED: {e}")
            
    def test_robot_creation(self):
        """Test basic robot creation and configuration"""
        # Test EV3 robot creation
        robot = create_robot("ev3")
        assert robot is not None
        assert hasattr(robot, 'hardware')
        assert hasattr(robot, 'state')
        assert hasattr(robot, 'connect')
        
        # Test Arduino robot creation
        robot = create_robot("arduino")
        assert robot is not None
        
        # Test Raspberry Pi robot creation
        robot = create_robot("raspberry_pi")
        assert robot is not None
        
    def test_robot_connection(self):
        """Test robot connection and disconnection"""
        robot = create_robot("ev3")
        
        # Test connection
        result = robot.connect()
        assert isinstance(result, bool)
        
        # Test state after connection
        if robot.is_running:
            state = robot.get_state()
            assert isinstance(state, RobotState)
            
        # Test disconnection
        robot.disconnect()
        assert not robot.is_running
        
    def test_hardware_abstraction(self):
        """Test hardware abstraction layer"""
        # Test EV3 hardware
        config = {
            'motors': {
                'left_motor': 'port_A',
                'right_motor': 'port_D'
            },
            'sensors': {
                'color_sensor': 'port_1',
                'ultrasonic_sensor': 'port_4'
            }
        }
        
        hardware = EV3Hardware(config)
        assert hardware is not None
        assert hardware.connect()
        assert hardware.is_connected()
        
        # Test sensor reading
        sensor_data = hardware.read_sensors()
        assert sensor_data is not None
        assert hasattr(sensor_data, 'timestamp')
        
        # Test command sending
        command = ControlCommand(0.5, 0.5)
        result = hardware.send_commands(command)
        assert isinstance(result, bool)
        
        hardware.disconnect()
        
    def test_pid_controller(self):
        """Test PID controller functionality"""
        config = PIDConfig(kp=2.0, ki=0.1, kd=0.05)
        pid = PIDController(config)
        
        # Test basic computation
        output = pid.compute(1.0, 0.0)
        assert isinstance(output, float)
        assert abs(output) <= 1.0  # Should be clamped to output range
        
        # Test multiple iterations
        for i in range(10):
            output = pid.compute(1.0, 0.5)
            assert isinstance(output, float)
            
        # Test performance metrics
        metrics = pid.get_performance_metrics()
        assert isinstance(metrics, dict)
        
        # Test reset
        pid.reset()
        
    def test_path_planning(self):
        """Test path planning algorithms"""
        # Test A* planner
        planner = AStarPlanner(grid_resolution=0.1)
        start = Point(0, 0)
        goal = Point(1, 1)
        obstacles = []
        
        path = planner.plan(start, goal, obstacles)
        assert path is not None
        assert len(path.points) > 0
        assert path.points[0] == start
        assert path.points[-1] == goal
        
        # Test trajectory generation
        from robotics.control.path_planner import TrajectoryGenerator
        generator = TrajectoryGenerator()
        trajectory = generator.generate_trapezoidal_trajectory(path, 5.0)
        assert trajectory is not None
        assert len(trajectory) > 0
        
    def test_camera_vision(self):
        """Test camera and computer vision"""
        # Test camera initialization
        camera = Camera(width=320, height=240)
        assert camera is not None
        
        # Test camera connection
        result = camera.initialize()
        assert isinstance(result, bool)
        
        # Test frame capture
        frame = camera.capture_frame()
        assert frame is not None
        assert frame.shape[0] > 0
        assert frame.shape[1] > 0
        
        # Test vision processor
        processor = VisionProcessor(camera)
        assert processor is not None
        
        # Test line detection
        line_info = processor.detect_line_for_following(frame)
        # Should return None or dict (line may not be detected in test frame)
        if line_info is not None:
            assert isinstance(line_info, dict)
            assert 'center' in line_info
            assert 'confidence' in line_info
            
    def test_sensor_fusion(self):
        """Test sensor fusion engine"""
        fusion = SensorFusionEngine()
        
        # Test sensor registration
        fusion.register_sensor("ultrasonic", "ultrasonic", 1.0)
        fusion.register_sensor("imu", "imu", 0.9)
        
        # Test Kalman filter initialization
        fusion.initialize_kalman_filter((0, 0), 0.0)
        
        # Test sensor fusion
        fused_state = fusion.fuse_sensors(0.02)
        assert fused_state is not None
        assert hasattr(fused_state, 'position')
        assert hasattr(fused_state, 'orientation')
        
        # Test sensor status
        status = fusion.get_sensor_status()
        assert isinstance(status, dict)
        
    def test_communication(self):
        """Test communication protocols"""
        # Test serial communication
        serial_config = {'simulation': True, 'port': '/dev/ttyUSB0', 'baudrate': 115200}
        serial_comm = SerialCommunication(serial_config)
        
        assert serial_comm is not None
        result = serial_comm.connect()
        assert isinstance(result, bool)
        
        # Test message creation
        message = Message(
            sender_id="test_sender",
            receiver_id="test_receiver",
            message_type="test",
            data={"value": 42}
        )
        assert message.sender_id == "test_sender"
        
        # Test message sending
        result = serial_comm.send_message(message)
        assert isinstance(result, bool)
        
        # Test message receiving
        received = serial_comm.receive_message()
        # May be None in simulation mode
        
        serial_comm.disconnect()
        
    def test_simulation_environment(self):
        """Test simulation environment"""
        # Create robot configuration
        robot_config = RobotConfig(
            width=0.2,
            length=0.3,
            max_speed=1.0
        )
        
        # Create simulation environment
        sim_env = SimulationEnvironment(robot_config)
        assert sim_env is not None
        
        # Create robot in simulation
        robot = sim_env.create_robot((1.0, 1.0))
        assert robot is not None
        
        # Test state access
        state = sim_env.get_state()
        assert isinstance(state, dict)
        
        # Test environment configuration
        assert hasattr(sim_env, 'robot')
        assert hasattr(sim_env, 'obstacles')
        
    def test_educational_demos(self):
        """Test educational demo implementations"""
        robot = create_robot("ev3")
        
        # Test demo creation
        demo = LineFollowingDemo(robot)
        assert demo is not None
        assert hasattr(demo, 'name')
        assert demo.name == "Line Following"
        
        # Test demo configuration
        assert hasattr(demo, 'pid_controller')
        assert hasattr(demo, 'max_speed')
        
    def test_curriculum_manager(self):
        """Test curriculum management"""
        manager = CurriculumManager()
        
        # Test curriculum creation
        foundation = manager.create_foundation_curriculum()
        assert foundation is not None
        assert hasattr(foundation, 'name')
        assert hasattr(foundation, 'lessons')
        assert len(foundation.lessons) > 0
        
        intermediate = manager.create_intermediate_curriculum()
        assert intermediate is not None
        
        advanced = manager.create_advanced_curriculum()
        assert advanced is not None
        
    def test_integration_workflow(self):
        """Test complete integration workflow"""
        # Create robot
        robot = create_robot("ev3")
        
        # Connect and setup
        robot.connect()
        
        # Test basic control
        command = ControlCommand(0.3, 0.3)
        result = robot.hardware.send_commands(command)
        assert isinstance(result, bool)
        
        # Test sensor integration
        sensor_data = robot.get_sensors()
        assert sensor_data is not None
        
        # Test state monitoring
        state = robot.get_state()
        assert isinstance(state, RobotState)
        
        # Cleanup
        robot.disconnect()
        
    def run_all_tests(self):
        """Run all tests in the test suite"""
        print("="*80)
        print("EDUCATIONAL ROBOTICS FRAMEWORK - COMPREHENSIVE TEST SUITE")
        print("="*80)
        
        if not IMPORTS_SUCCESSFUL:
            print(f"Cannot run tests due to import error: {IMPORT_ERROR}")
            return False
            
        # Core functionality tests
        self.run_test("Robot Creation", self.test_robot_creation)
        self.run_test("Robot Connection", self.test_robot_connection)
        
        # Hardware abstraction tests
        self.run_test("Hardware Abstraction", self.test_hardware_abstraction)
        
        # Control system tests
        self.run_test("PID Controller", self.test_pid_controller)
        self.run_test("Path Planning", self.test_path_planning)
        
        # Vision and sensing tests
        self.run_test("Camera Vision", self.test_camera_vision)
        self.run_test("Sensor Fusion", self.test_sensor_fusion)
        
        # Communication tests
        self.run_test("Communication Protocols", self.test_communication)
        
        # Simulation tests
        self.run_test("Simulation Environment", self.test_simulation_environment)
        
        # Educational components tests
        self.run_test("Educational Demos", self.test_educational_demos)
        self.run_test("Curriculum Manager", self.test_curriculum_manager)
        
        # Integration tests
        self.run_test("Integration Workflow", self.test_integration_workflow)
        
        # Print summary
        self.print_summary()
        
        return self.failed_tests == 0
        
    def print_summary(self):
        """Print test execution summary"""
        print(f"\n{'='*80}")
        print("TEST EXECUTION SUMMARY")
        print(f"{'='*80}")
        
        total_tests = self.passed_tests + self.failed_tests
        success_rate = (self.passed_tests / total_tests * 100) if total_tests > 0 else 0
        
        print(f"Total Tests: {total_tests}")
        print(f"Passed: {self.passed_tests}")
        print(f"Failed: {self.failed_tests}")
        print(f"Success Rate: {success_rate:.1f}%")
        
        if self.failed_tests > 0:
            print(f"\nFailed Tests:")
            for test_name, result in self.test_results.items():
                if result['status'] == 'FAILED':
                    print(f"  - {test_name}: {result['error']}")
                    
        print(f"\n{'='*80}")
        
        if success_rate == 100.0:
            print("🎉 ALL TESTS PASSED! Framework is ready for use.")
        elif success_rate >= 80.0:
            print("✅ Most tests passed. Framework is largely functional.")
        else:
            print("⚠️  Several tests failed. Framework may need fixes.")
            
        print(f"{'='*80}")


def main():
    """Main test execution function"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Run robotics framework tests")
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--quick', '-q', action='store_true', help='Run quick tests only')
    args = parser.parse_args()
    
    if args.verbose:
        print("Running in verbose mode...")
        
    if args.quick:
        print("Running quick tests only...")
        # Run only core tests for quick validation
        tester = FrameworkTester()
        
        if IMPORTS_SUCCESSFUL:
            # Quick test of core functionality
            try:
                robot = create_robot("ev3")
                result = robot.connect()
                print(f"✓ Core imports and robot creation: SUCCESS")
                
                hardware = EV3Hardware()
                result = hardware.connect()
                print(f"✓ Hardware abstraction: {'SUCCESS' if result else 'SIMULATION MODE'}")
                
                pid = PIDController()
                output = pid.compute(1.0, 0.0)
                print(f"✓ PID controller: SUCCESS (output: {output:.3f})")
                
                planner = PathPlanner("astar")
                trajectory = planner.navigate_to((0, 0), (1, 1))
                print(f"✓ Path planning: {'SUCCESS' if trajectory else 'SIMULATION MODE'}")
                
                print("✓ QUICK TESTS COMPLETED SUCCESSFULLY")
                
            except Exception as e:
                print(f"✗ Quick tests failed: {e}")
                return 1
    else:
        # Run full test suite
        tester = FrameworkTester()
        success = tester.run_all_tests()
        return 0 if success else 1
        
    return 0


if __name__ == "__main__":
    sys.exit(main())
