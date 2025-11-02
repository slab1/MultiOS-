"""
Educational Robotics Framework Examples

Practical examples demonstrating the framework capabilities
"""

import time
import math
import numpy as np
from typing import List, Tuple, Optional

# Import framework components
from robotics import Robot, create_robot, start_demo
from robotics.core.robot import ControlCommand
from robotics.control.pid_controller import PIDController, PIDConfig
from robotics.control.path_planner import PathPlanner, Point
from robotics.vision.camera import Camera, VisionProcessor
from robotics.vision.sensor_fusion import SensorFusionEngine
from robotics.demos.educational_demos import DemoRunner


class BasicRobotMovement:
    """Example: Basic robot movement and control"""
    
    def __init__(self, robot_type: str = "ev3"):
        self.robot = create_robot(robot_type)
        self.robot.connect()
        print(f"Connected {robot_type} robot")
        
    def move_forward(self, speed: float = 0.5, duration: float = 2.0):
        """Move robot forward"""
        print(f"Moving forward at {speed} speed for {duration} seconds")
        
        start_time = time.time()
        while time.time() - start_time < duration:
            command = ControlCommand(speed, speed)
            self.robot.hardware.send_commands(command)
            time.sleep(0.1)
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Movement complete")
        
    def turn_in_place(self, angle: float, speed: float = 0.3):
        """Turn robot in place by specified angle"""
        print(f"Turning {angle} degrees at speed {speed}")
        
        # Calculate turn duration (simplified)
        duration = abs(angle) / (speed * 90)  # Rough approximation
        
        if angle > 0:
            # Turn left
            command = ControlCommand(-speed, speed)
        else:
            # Turn right
            command = ControlCommand(speed, -speed)
            
        start_time = time.time()
        while time.time() - start_time < duration:
            self.robot.hardware.send_commands(command)
            time.sleep(0.1)
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Turn complete")
        
    def follow_square_path(self, side_length: float = 1.0):
        """Follow a square path"""
        print("Following square path")
        
        # Define square corners
        corners = [
            (side_length, 0),
            (side_length, side_length),
            (0, side_length),
            (0, 0)
        ]
        
        for i, (target_x, target_y) in enumerate(corners):
            print(f"Moving to corner {i+1}: ({target_x}, {target_y})")
            
            # Simplified path following (in real implementation, would use actual path planning)
            dx = target_x
            dy = target_y
            distance = math.sqrt(dx*dx + dy*dy)
            
            if distance > 0:
                # Move forward
                speed = 0.3
                duration = distance / speed
                command = ControlCommand(speed, speed)
                
                start_time = time.time()
                while time.time() - start_time < duration:
                    self.robot.hardware.send_commands(command)
                    time.sleep(0.1)
                    
            # Turn 90 degrees
            self.turn_in_place(-90)  # Turn right for square
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Square path complete")
        
    def cleanup(self):
        """Clean up robot connection"""
        self.robot.disconnect()
        print("Robot disconnected")


class SensorIntegration:
    """Example: Using sensors for robot control"""
    
    def __init__(self, robot_type: str = "ev3"):
        self.robot = create_robot(robot_type)
        self.robot.connect()
        
        # Initialize sensors
        self.sensor_fusion = SensorFusionEngine()
        self.sensor_fusion.register_sensor("ultrasonic_front", "ultrasonic", 1.0)
        self.sensor_fusion.register_sensor("color_sensor", "color", 0.9)
        self.sensor_fusion.initialize_kalman_filter()
        
        print("Sensor integration initialized")
        
    def sensor_reading_demo(self):
        """Demonstrate sensor reading"""
        print("Demonstrating sensor readings...")
        
        for i in range(10):
            sensor_data = self.robot.get_sensors()
            
            print(f"Sensor reading {i+1}:")
            if sensor_data.ultrasonic_distance:
                print(f"  Ultrasonic: {sensor_data.ultrasonic_distance:.2f}m")
            if sensor_data.color_reflectance:
                print(f"  Color: {sensor_data.color_reflectance}")
            if sensor_data.imu_gyroscope:
                print(f"  Gyroscope: {[f'{x:.2f}' for x in sensor_data.imu_gyroscope]}")
                
            time.sleep(1.0)
            
    def distance_based_control(self, stop_distance: float = 0.3):
        """Control robot based on distance sensor"""
        print(f"Starting distance-based control (stop at {stop_distance}m)")
        
        start_time = time.time()
        while time.time() - start_time < 30:  # Run for 30 seconds
            sensor_data = self.robot.get_sensors()
            distance = sensor_data.ultrasonic_distance or 2.0
            
            if distance > stop_distance:
                # Move forward
                speed = min(0.5, distance - stop_distance + 0.1)
                command = ControlCommand(speed, speed)
                print(f"Moving forward at {speed:.2f} (distance: {distance:.2f}m)")
            else:
                # Stop
                command = ControlCommand(0.0, 0.0)
                print("Obstacle detected - stopping")
                
            self.robot.hardware.send_commands(command)
            time.sleep(0.1)
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Distance control demo complete")
        
    def cleanup(self):
        self.robot.disconnect()


class VisionBasedControl:
    """Example: Computer vision for robot control"""
    
    def __init__(self, robot_type: str = "raspberry_pi"):
        self.robot = create_robot(robot_type)
        self.robot.connect()
        
        # Initialize camera and vision processing
        self.camera = Camera(width=640, height=480)
        self.camera.initialize()
        self.vision_processor = VisionProcessor(self.camera)
        self.vision_processor.start_processing()
        
        print("Vision-based control initialized")
        
    def line_following_demo(self):
        """Demonstrate line following using vision"""
        print("Starting line following demo...")
        
        # PID controller for line following
        pid_controller = PIDController(
            PIDConfig(kp=2.0, ki=0.1, kd=0.5, output_min=-1.0, output_max=1.0)
        )
        
        base_speed = 0.3
        
        start_time = time.time()
        while time.time() - start_time < 60:  # Run for 60 seconds
            # Get line detection
            line_info = self.vision_processor.detect_line_for_following()
            
            if line_info and line_info['confidence'] > 0.3:
                # Calculate line offset from center
                center_x = line_info['center'][0]
                frame_center_x = 320  # Half of 640
                line_offset = (center_x - frame_center_x) / frame_center_x
                
                # Use PID to calculate steering correction
                steering = pid_controller.compute(0.0, line_offset)
                
                # Calculate motor speeds
                left_speed = base_speed - steering * 0.2
                right_speed = base_speed + steering * 0.2
                
                # Clamp speeds
                left_speed = max(-0.5, min(0.5, left_speed))
                right_speed = max(-0.5, min(0.5, right_speed))
                
                command = ControlCommand(left_speed, right_speed)
                self.robot.hardware.send_commands(command)
                
                if int(time.time()) % 5 == 0:
                    print(f"Following line - Offset: {line_offset:.3f}, "
                          f"Steering: {steering:.3f}")
            else:
                # Line not detected - slow down and search
                command = ControlCommand(0.1, -0.1)  # Turn to search
                self.robot.hardware.send_commands(command)
                print("Searching for line...")
                
            time.sleep(0.05)  # 20 Hz
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Line following demo complete")
        
    def object_detection_demo(self):
        """Demonstrate object detection and avoidance"""
        print("Starting object detection demo...")
        
        pid_controller = PIDController(
            PIDConfig(kp=1.5, ki=0.05, kd=0.3)
        )
        
        start_time = time.time()
        while time.time() - start_time < 60:
            # Process frame for object detection
            result = self.vision_processor.process_frame()
            
            # Check for detected objects
            if result.detected_objects:
                # Find closest object
                closest_object = min(result.detected_objects, 
                                    key=lambda obj: obj.get('area', 0))
                
                # Get object position
                center = closest_object['center']
                frame_center_x = 320
                offset = (center[0] - frame_center_x) / frame_center_x
                
                # Calculate avoidance maneuver
                steering = pid_controller.compute(0.0, offset)
                
                # Adjust speeds to avoid object
                if closest_object.get('confidence', 0) > 0.5:
                    # Object detected - turn away
                    base_speed = 0.2
                    left_speed = base_speed - steering * 0.3
                    right_speed = base_speed + steering * 0.3
                    
                    command = ControlCommand(left_speed, right_speed)
                    self.robot.hardware.send_commands(command)
                    
                    print(f"Avoiding {closest_object.get('color', 'unknown')} object")
                else:
                    # Object not confident - continue
                    command = ControlCommand(0.3, 0.3)
                    self.robot.hardware.send_commands(command)
            else:
                # No objects detected - move forward
                command = ControlCommand(0.3, 0.3)
                self.robot.hardware.send_commands(command)
                
            time.sleep(0.1)
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Object detection demo complete")
        
    def cleanup(self):
        self.vision_processor.stop_processing()
        self.camera.stop_capture()
        self.robot.disconnect()


class PathPlanningDemo:
    """Example: Path planning and navigation"""
    
    def __init__(self, robot_type: str = "ev3"):
        self.robot = create_robot(robot_type)
        self.robot.connect()
        
        # Initialize path planner
        self.path_planner = PathPlanner("astar")
        
        print("Path planning demo initialized")
        
    def navigate_to_target(self, start_pos: Tuple[float, float], 
                          target_pos: Tuple[float, float]):
        """Navigate robot to target position"""
        print(f"Navigating from {start_pos} to {target_pos}")
        
        # Define obstacles
        obstacles = [
            (2.0, 1.0),  # Box obstacle
            (3.0, 2.0),  # Cylinder obstacle  
            (1.5, 3.0),  # Wall obstacle
            (4.0, 1.5)   # Complex obstacle
        ]
        
        # Plan path
        trajectory = self.path_planner.navigate_to(
            start_pos, target_pos, 
            obstacles=obstacles, 
            total_time=20.0
        )
        
        if not trajectory:
            print("No path found!")
            return
            
        print(f"Generated path with {len(trajectory)} waypoints")
        
        # Execute path following
        for i, point in enumerate(trajectory[::5]):  # Follow every 5th point
            # Calculate desired velocity
            if i < len(trajectory) - 5:
                next_point = trajectory[i + 5]
                dx = next_point.x - point.x
                dy = next_point.y - point.y
                distance = math.sqrt(dx*dx + dy*dy)
                
                if distance > 0:
                    speed = min(0.3, distance * 2)  # Scale speed by distance
                    
                    # Calculate direction
                    if abs(dx) > abs(dy):
                        # More horizontal movement
                        left_speed = speed if dx >= 0 else -speed
                        right_speed = speed if dx >= 0 else -speed
                    else:
                        # More vertical movement
                        if dy >= 0:
                            left_speed = 0.2
                            right_speed = 0.2
                        else:
                            left_speed = -0.2
                            right_speed = -0.2
                            
                    command = ControlCommand(left_speed, right_speed)
                    self.robot.hardware.send_commands(command)
                    
                    print(f"Moving to waypoint {i}: ({point.x:.2f}, {point.y:.2f}) "
                          f"at speed {speed:.2f}")
                    
            time.sleep(1.0)  # Wait between waypoints
            
        # Stop
        command = ControlCommand(0.0, 0.0)
        self.robot.hardware.send_commands(command)
        print("Navigation complete")
        
    def maze_solving(self):
        """Solve a simple maze using path planning"""
        print("Starting maze solving...")
        
        # Define maze walls as obstacles
        walls = [
            (1, 0, 8, 0.2),   # Bottom wall
            (1, 4, 8, 0.2),   # Top wall
            (1, 0, 0.2, 4),   # Left wall
            (9, 0, 0.2, 4),   # Right wall
            (3, 1, 0.2, 2),   # Internal wall
            (5, 2, 0.2, 2),   # Internal wall
            (7, 1, 0.2, 2),   # Internal wall
        ]
        
        # Convert wall format for path planning
        obstacles = [(wall[0] + wall[2]/2, wall[1] + wall[3]/2) for wall in walls]
        
        start_pos = (1.5, 2.0)
        goal_pos = (8.5, 2.0)
        
        # Plan path through maze
        trajectory = self.path_planner.navigate_to(
            start_pos, goal_pos,
            obstacles=obstacles,
            total_time=30.0
        )
        
        if trajectory:
            print(f"Maze solution found with {len(trajectory)} waypoints")
            # Execute path following (simplified for demo)
            print("Executing maze solution...")
            time.sleep(2)  # Simulate execution
            print("Maze solved!")
        else:
            print("No maze solution found!")
            
    def cleanup(self):
        self.robot.disconnect()


class CommunicationDemo:
    """Example: Robot communication and networking"""
    
    def __init__(self, robot_type: str = "ev3"):
        self.robot = create_robot(robot_type)
        self.robot.connect()
        
        # Import communication modules
        from robotics.communication.protocols import SerialCommunication, WiFiCommunication
        
        # Initialize communication channels
        self.serial_comm = SerialCommunication({'simulation': True})
        self.wifi_comm = WiFiCommunication({'role': 'client', 'simulation': True})
        
        print("Communication demo initialized")
        
    def serial_communication_demo(self):
        """Demonstrate serial communication"""
        print("Starting serial communication demo...")
        
        # Connect serial communication
        if self.serial_comm.connect():
            # Send test message
            test_command = {
                'command': 'move_forward',
                'speed': 0.5,
                'duration': 2.0
            }
            
            self.serial_comm.send_robot_command('move_forward', test_command)
            print("Sent forward command via serial")
            
            # Listen for responses
            for i in range(20):
                message = self.serial_comm.receive_message()
                if message:
                    print(f"Received: {message.message_type} from {message.sender_id}")
                time.sleep(0.5)
                
            self.serial_comm.disconnect()
        else:
            print("Serial communication failed")
            
    def wifi_networking_demo(self):
        """Demonstrate WiFi networking"""
        print("Starting WiFi networking demo...")
        
        # This would typically connect to a server
        print("WiFi communication is in simulation mode")
        print("In real implementation, this would:")
        print("1. Connect to WiFi network")
        print("2. Send/receive messages with other robots")
        print("3. Participate in robot swarm")
        print("4. Update central control system")
        
        # Simulate some networking activities
        time.sleep(2)
        print("Simulated networking operations complete")
        
    def robot_swarm_demo(self):
        """Demonstrate multi-robot coordination"""
        print("Starting robot swarm coordination demo...")
        
        # Simulate robot swarm behavior
        print("Simulating robot swarm with 3 robots:")
        print("- Robot 1: Leader, moving forward")
        print("- Robot 2: Following at offset (0.5m left, 1m behind)")
        print("- Robot 3: Following at offset (0.5m right, 1m behind)")
        
        # In real implementation, this would involve:
        # 1. Leader robot broadcasts its position
        # 2. Follower robots receive position data
        # 3. Followers calculate their target positions
        # 4. All robots move in formation
        
        time.sleep(3)
        print("Swarm coordination simulation complete")
        
    def cleanup(self):
        self.robot.disconnect()


def run_basic_examples():
    """Run basic robot movement examples"""
    print("="*60)
    print("RUNNING BASIC ROBOT MOVEMENT EXAMPLES")
    print("="*60)
    
    try:
        # Create movement controller
        movement = BasicRobotMovement("ev3")
        
        print("\n1. Basic Forward Movement")
        movement.move_forward(speed=0.3, duration=2.0)
        
        print("\n2. Turning in Place")
        movement.turn_in_place(90, speed=0.3)
        
        print("\n3. Square Path Following")
        movement.follow_square_path(side_length=0.5)
        
        movement.cleanup()
        
    except Exception as e:
        print(f"Error in basic examples: {e}")


def run_sensor_examples():
    """Run sensor integration examples"""
    print("="*60)
    print("RUNNING SENSOR INTEGRATION EXAMPLES")
    print("="*60)
    
    try:
        sensors = SensorIntegration("ev3")
        
        print("\n1. Sensor Reading Demonstration")
        sensors.sensor_reading_demo()
        
        print("\n2. Distance-Based Control")
        sensors.distance_based_control(stop_distance=0.5)
        
        sensors.cleanup()
        
    except Exception as e:
        print(f"Error in sensor examples: {e}")


def run_vision_examples():
    """Run computer vision examples"""
    print("="*60)
    print("RUNNING COMPUTER VISION EXAMPLES")
    print("="*60)
    
    try:
        vision = VisionBasedControl("raspberry_pi")
        
        print("\n1. Line Following with Vision")
        vision.line_following_demo()
        
        print("\n2. Object Detection and Avoidance")
        vision.object_detection_demo()
        
        vision.cleanup()
        
    except Exception as e:
        print(f"Error in vision examples: {e}")


def run_planning_examples():
    """Run path planning examples"""
    print("="*60)
    print("RUNNING PATH PLANNING EXAMPLES")
    print("="*60)
    
    try:
        planning = PathPlanningDemo("ev3")
        
        print("\n1. Target Navigation")
        planning.navigate_to_target((0, 0), (3, 2))
        
        print("\n2. Maze Solving")
        planning.maze_solving()
        
        planning.cleanup()
        
    except Exception as e:
        print(f"Error in planning examples: {e}")


def run_communication_examples():
    """Run communication examples"""
    print("="*60)
    print("RUNNING COMMUNICATION EXAMPLES")
    print("="*60)
    
    try:
        comm = CommunicationDemo("ev3")
        
        print("\n1. Serial Communication")
        comm.serial_communication_demo()
        
        print("\n2. WiFi Networking")
        comm.wifi_networking_demo()
        
        print("\n3. Robot Swarm")
        comm.robot_swarm_demo()
        
        comm.cleanup()
        
    except Exception as e:
        print(f"Error in communication examples: {e}")


def run_demo_suite():
    """Run complete demo suite"""
    print("="*80)
    print("EDUCATIONAL ROBOTICS FRAMEWORK - COMPLETE DEMO SUITE")
    print("="*80)
    
    demos = [
        ("Basic Movement", run_basic_examples),
        ("Sensor Integration", run_sensor_examples),
        ("Computer Vision", run_vision_examples),
        ("Path Planning", run_planning_examples),
        ("Communication", run_communication_examples)
    ]
    
    for demo_name, demo_function in demos:
        print(f"\n{'='*20} {demo_name} {'='*20}")
        try:
            demo_function()
            print(f"✓ {demo_name} completed successfully")
        except Exception as e:
            print(f"✗ {demo_name} failed: {e}")
            
    print(f"\n{'='*80}")
    print("DEMO SUITE COMPLETE")
    print(f"{'='*80}")


def run_interactive_demo():
    """Run interactive demo where user can control robot"""
    print("="*60)
    print("INTERACTIVE ROBOT CONTROL DEMO")
    print("="*60)
    print("Commands:")
    print("  w - Move forward")
    print("  s - Move backward")  
    print("  a - Turn left")
    print("  d - Turn right")
    print("  space - Stop")
    print("  q - Quit")
    print()
    
    try:
        robot_controller = BasicRobotMovement("ev3")
        
        print("Use keyboard commands to control robot:")
        
        while True:
            command = input("Enter command (w/s/a/d/space/q): ").lower().strip()
            
            if command == 'q':
                break
            elif command == 'w':
                print("Moving forward...")
                robot_controller.move_forward(speed=0.3, duration=1.0)
            elif command == 's':
                print("Moving backward...")
                robot_controller.move_forward(speed=-0.3, duration=1.0)
            elif command == 'a':
                print("Turning left...")
                robot_controller.turn_in_place(45, speed=0.3)
            elif command == 'd':
                print("Turning right...")
                robot_controller.turn_in_place(-45, speed=0.3)
            elif command == ' ':
                print("Stopping...")
                command = ControlCommand(0.0, 0.0)
                robot_controller.robot.hardware.send_commands(command)
            else:
                print("Unknown command. Use w/s/a/d/space/q")
                
        robot_controller.cleanup()
        print("Interactive demo ended")
        
    except KeyboardInterrupt:
        print("\nDemo interrupted by user")
    except Exception as e:
        print(f"Error in interactive demo: {e}")


if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1:
        demo_type = sys.argv[1].lower()
        
        if demo_type == "basic":
            run_basic_examples()
        elif demo_type == "sensors":
            run_sensor_examples()
        elif demo_type == "vision":
            run_vision_examples()
        elif demo_type == "planning":
            run_planning_examples()
        elif demo_type == "communication":
            run_communication_examples()
        elif demo_type == "interactive":
            run_interactive_demo()
        elif demo_type == "all":
            run_demo_suite()
        else:
            print(f"Unknown demo type: {demo_type}")
            print("Available demos: basic, sensors, vision, planning, communication, interactive, all")
    else:
        # Run a simple demo by default
        print("Running basic movement demo...")
        run_basic_examples()
