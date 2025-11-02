"""
Educational Robotics Demos

Real-world robotics demonstrations for educational purposes
"""

import time
import math
import numpy as np
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass
from core.robot import Robot, RobotState, ControlCommand
from control.pid_controller import PIDController, PIDConfig
from vision.camera import Camera, VisionProcessor, LineDetector
from vision.sensor_fusion import SensorFusionEngine


@dataclass
class DemoResult:
    """Result of demo execution"""
    success: bool
    duration: float
    distance_traveled: float
    final_position: tuple
    errors_encountered: List[str]
    performance_metrics: Dict[str, Any]


class BaseDemo:
    """Base class for all robotics demos"""
    
    def __init__(self, robot: Robot, name: str = "base_demo"):
        self.robot = robot
        self.name = name
        self.is_running = False
        self.start_time = 0.0
        self.result = DemoResult(
            success=False,
            duration=0.0,
            distance_traveled=0.0,
            final_position=(0.0, 0.0),
            errors_encountered=[],
            performance_metrics={}
        )
        
        # Control parameters
        self.max_speed = 0.5
        self.max_time = 60.0  # Maximum demo duration in seconds
        
        # Statistics tracking
        self.total_distance = 0.0
        self.last_position = None
        self.performance_data = []
        
    def start(self) -> DemoResult:
        """Start the demo"""
        print(f"Starting {self.name} demo...")
        self.is_running = True
        self.start_time = time.time()
        self.result = DemoResult(
            success=False,
            duration=0.0,
            distance_traveled=0.0,
            final_position=(0.0, 0.0),
            errors_encountered=[],
            performance_metrics={}
        )
        
        # Reset robot state
        initial_state = self.robot.get_state()
        self.last_position = (initial_state.x, initial_state.y)
        
        try:
            self._setup()
            self._run_demo()
            self._cleanup()
            self.result.success = True
        except Exception as e:
            error_msg = f"Demo failed with error: {str(e)}"
            print(error_msg)
            self.result.errors_encountered.append(error_msg)
            
        # Calculate final metrics
        self.result.duration = time.time() - self.start_time
        self.result.distance_traveled = self.total_distance
        self.result.final_position = self.last_position
        self.result.performance_metrics = self._calculate_performance_metrics()
        
        print(f"{self.name} demo completed. Success: {self.result.success}")
        return self.result
        
    def stop(self):
        """Stop the demo"""
        self.is_running = False
        self._cleanup()
        
    def _setup(self):
        """Setup demo-specific components"""
        pass
        
    def _run_demo(self):
        """Main demo execution loop"""
        while self.is_running and (time.time() - self.start_time) < self.max_time:
            try:
                # Update robot state
                current_time = time.time()
                dt = current_time - getattr(self, '_last_update_time', current_time)
                self._last_update_time = current_time
                
                # Execute demo logic
                self._execute_step(dt)
                
                # Update statistics
                self._update_statistics()
                
                time.sleep(0.02)  # 50 Hz update rate
                
            except Exception as e:
                self.result.errors_encountered.append(f"Runtime error: {str(e)}")
                break
                
    def _execute_step(self, dt: float):
        """Execute one step of the demo"""
        # To be implemented by subclasses
        pass
        
    def _cleanup(self):
        """Cleanup demo-specific resources"""
        pass
        
    def _update_statistics(self):
        """Update demo statistics"""
        current_state = self.robot.get_state()
        current_position = (current_state.x, current_state.y)
        
        if self.last_position:
            distance = math.sqrt(
                (current_position[0] - self.last_position[0])**2 + 
                (current_position[1] - self.last_position[1])**2
            )
            self.total_distance += distance
            
        self.last_position = current_position
        
        # Store performance data
        self.performance_data.append({
            'time': time.time() - self.start_time,
            'position': current_position,
            'orientation': current_state.theta,
            'velocity': (current_state.vx, current_state.vy)
        })
        
    def _calculate_performance_metrics(self) -> Dict[str, Any]:
        """Calculate performance metrics"""
        if not self.performance_data:
            return {}
            
        metrics = {
            'total_distance': self.total_distance,
            'average_speed': self.total_distance / max(self.result.duration, 0.1),
            'completion_rate': 1.0 if self.result.success else 0.0,
            'data_points': len(self.performance_data)
        }
        
        # Calculate smoothness metrics
        if len(self.performance_data) > 1:
            velocities = []
            for i in range(1, len(self.performance_data)):
                dt = (self.performance_data[i]['time'] - 
                      self.performance_data[i-1]['time'])
                if dt > 0:
                    dx = (self.performance_data[i]['position'][0] - 
                          self.performance_data[i-1]['position'][0])
                    dy = (self.performance_data[i]['position'][1] - 
                          self.performance_data[i-1]['position'][1])
                    speed = math.sqrt(dx*dx + dy*dy) / dt
                    velocities.append(speed)
                    
            if velocities:
                metrics['speed_variance'] = np.var(velocities)
                metrics['max_speed'] = max(velocities)
                metrics['min_speed'] = min(velocities)
                
        return metrics


class LineFollowingDemo(BaseDemo):
    """Line Following Robot Demo"""
    
    def __init__(self, robot: Robot):
        super().__init__(robot, "Line Following")
        
        # Line following specific configuration
        self.line_detector = LineDetector()
        self.pid_controller = PIDController(
            PIDConfig(kp=2.0, ki=0.1, kd=0.5, output_min=-1.0, output_max=1.0)
        )
        
        # Control parameters
        self.base_speed = 0.3
        self.line_confidence_threshold = 0.3
        
        # State tracking
        self.line_lost_time = 0.0
        self.max_line_lost_duration = 3.0
        
    def _setup(self):
        """Setup line following demo"""
        print("Setting up line following demo...")
        
        # Initialize PID controller with line following parameters
        self.pid_controller.config.kp = 2.5
        self.pid_controller.config.ki = 0.05
        self.pid_controller.config.kd = 1.0
        
        # Set base speed for line following
        self.base_speed = 0.4
        
    def _execute_step(self, dt: float):
        """Execute line following step"""
        # Get camera frame for line detection
        if hasattr(self.robot, 'camera') and self.robot.camera:
            frame = self.robot.camera.capture_frame()
        else:
            # Simulate camera frame for testing
            frame = np.zeros((480, 640, 3), dtype=np.uint8)
            
        if frame is not None:
            # Detect line
            line_info = self.line_detector.detect_line_following_line(frame)
            
            if line_info and line_info['confidence'] > self.line_confidence_threshold:
                # Line detected - follow it
                line_center_x = line_info['center'][0]
                frame_center_x = frame.shape[1] // 2
                
                # Calculate line offset from center (normalized to -1 to 1)
                line_offset = (line_center_x - frame_center_x) / (frame.shape[1] // 2)
                
                # Use PID controller to calculate steering correction
                steering_correction = self.pid_controller.compute(0.0, line_offset, dt)
                
                # Calculate motor speeds
                left_speed = self.base_speed - steering_correction * 0.3
                right_speed = self.base_speed + steering_correction * 0.3
                
                # Clamp speeds
                left_speed = max(-self.max_speed, min(self.max_speed, left_speed))
                right_speed = max(-self.max_speed, min(self.max_speed, right_speed))
                
                # Send command to robot
                command = ControlCommand(left_speed, right_speed)
                self.robot.hardware.send_commands(command)
                
                # Reset line lost timer
                self.line_lost_time = 0.0
                
                # Print progress occasionally
                if int(time.time() * 2) % 2 == 0:
                    print(f"Following line - Offset: {line_offset:.3f}, "
                          f"Correction: {steering_correction:.3f}")
                    
            else:
                # Line lost - search or stop
                self.line_lost_time += dt
                
                if self.line_lost_time < self.max_line_lost_duration:
                    # Slow down and turn to search for line
                    search_speed = 0.1
                    turn_speed = 0.3
                    
                    if int(self.line_lost_time * 2) % 2 == 0:
                        left_speed = -turn_speed
                        right_speed = search_speed
                    else:
                        left_speed = search_speed
                        right_speed = -turn_speed
                        
                    command = ControlCommand(left_speed, right_speed)
                    self.robot.hardware.send_commands(command)
                    
                    print(f"Line lost - searching... ({self.line_lost_time:.1f}s)")
                    
                else:
                    # Line lost too long - stop
                    command = ControlCommand(0.0, 0.0)
                    self.robot.hardware.send_commands(command)
                    print("Line lost for too long - stopping")
                    self.is_running = False


class ObstacleAvoidanceDemo(BaseDemo):
    """Obstacle Avoidance Robot Demo"""
    
    def __init__(self, robot: Robot):
        super().__init__(robot, "Obstacle Avoidance")
        
        # Obstacle avoidance configuration
        self.sensor_fusion = SensorFusionEngine()
        self.pid_controller = PIDController(
            PIDConfig(kp=1.5, ki=0.05, kd=0.3)
        )
        
        # Control parameters
        self.forward_speed = 0.4
        self.avoidance_distance = 0.3  # Distance to start avoidance (meters)
        self.stop_distance = 0.15      # Distance to stop (meters)
        
        # Sensor configuration
        self.sensor_fusion.register_sensor("ultrasonic_front", "ultrasonic", 1.0)
        self.sensor_fusion.register_sensor("ultrasonic_left", "ultrasonic", 0.8)
        self.sensor_fusion.register_sensor("ultrasonic_right", "ultrasonic", 0.8)
        
    def _setup(self):
        """Setup obstacle avoidance demo"""
        print("Setting up obstacle avoidance demo...")
        
        # Initialize sensor fusion
        self.sensor_fusion.initialize_kalman_filter()
        
        # Configure PID controller for smooth avoidance
        self.pid_controller.config.kp = 2.0
        self.pid_controller.config.ki = 0.1
        self.pid_controller.config.kd = 0.5
        
    def _execute_step(self, dt: float):
        """Execute obstacle avoidance step"""
        # Get sensor readings
        sensor_data = self.robot.get_sensors()
        
        # Update sensor fusion with latest readings
        if sensor_data.ultrasonic_distance:
            # Update front sensor
            front_reading = self.sensor_fusion.SensorReading(
                value=sensor_data.ultrasonic_distance,
                timestamp=sensor_data.timestamp,
                sensor_id="ultrasonic_front",
                confidence=0.9
            )
            self.sensor_fusion.update_sensor_reading("ultrasonic_front", front_reading)
            
        # Get front distance from fusion engine
        front_distance = sensor_data.ultrasonic_distance or 2.0
        
        # Obstacle avoidance logic
        if front_distance > self.avoidance_distance:
            # Clear path - move forward
            command = ControlCommand(self.forward_speed, self.forward_speed)
            self.robot.hardware.send_commands(command)
            
            if int(time.time() * 2) % 2 == 0:
                print(f"Moving forward - Distance: {front_distance:.2f}m")
                
        elif front_distance > self.stop_distance:
            # Obstacle detected - turn to avoid
            # Use side sensors to determine turn direction
            left_distance = self._get_side_distance("left", sensor_data)
            right_distance = self._get_side_distance("right", sensor_data)
            
            if left_distance > right_distance:
                # Turn left
                turn_speed = 0.3
                command = ControlCommand(-turn_speed, turn_speed)
                direction = "left"
            else:
                # Turn right
                turn_speed = 0.3
                command = ControlCommand(turn_speed, -turn_speed)
                direction = "right"
                
            self.robot.hardware.send_commands(command)
            
            print(f"Avoiding obstacle - turning {direction}, "
                  f"Distance: {front_distance:.2f}m")
                  
        else:
            # Too close - stop
            command = ControlCommand(0.0, 0.0)
            self.robot.hardware.send_commands(command)
            print(f"Too close to obstacle - stopping at {front_distance:.2f}m")
            
    def _get_side_distance(self, side: str, sensor_data) -> float:
        """Get distance from side sensor"""
        # This would need actual side sensor implementation
        # For now, simulate based on robot position or use other sensors
        return sensor_data.ultrasonic_distance or 1.0


class MazeSolvingDemo(BaseDemo):
    """Maze Solving Robot Demo using right-hand rule"""
    
    def __init__(self, robot: Robot):
        super().__init__(robot, "Maze Solving")
        
        # Maze solving configuration
        self.current_direction = 0  # 0=North, 1=East, 2=South, 3=West
        self.maze_state = {
            'walls': set(),  # Set of walls (x, y, direction)
            'visited': set(),  # Set of visited positions
            'current_cell': (0, 0),  # Current grid cell
            'start_cell': (0, 0),
            'goal_cell': None
        }
        
        # Movement parameters
        self.cell_size = 0.5  # Size of each maze cell in meters
        self.movement_speed = 0.3
        self.turn_speed = 0.4
        
        # State tracking
        self.move_in_progress = False
        self.turn_in_progress = False
        self.target_cell = None
        
        # Maze representation
        self.maze_grid = []  # Will be populated with maze layout
        
    def _setup(self):
        """Setup maze solving demo"""
        print("Setting up maze solving demo...")
        
        # Define maze layout (0=free, 1=wall, 2=start, 3=goal)
        self.maze_grid = [
            [1, 1, 1, 1, 1, 1, 1],
            [1, 2, 0, 0, 0, 0, 1],
            [1, 1, 1, 0, 1, 0, 1],
            [1, 0, 0, 0, 1, 0, 1],
            [1, 0, 1, 1, 1, 3, 1],
            [1, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1]
        ]
        
        # Find start and goal positions
        for y, row in enumerate(self.maze_grid):
            for x, cell in enumerate(row):
                if cell == 2:  # Start
                    self.maze_state['start_cell'] = (x, y)
                    self.maze_state['current_cell'] = (x, y)
                elif cell == 3:  # Goal
                    self.maze_state['goal_cell'] = (x, y)
                    
        # Mark start cell as visited
        self.maze_state['visited'].add(self.maze_state['current_cell'])
        
        print(f"Maze solving from {self.maze_state['start_cell']} "
              f"to {self.maze_state['goal_cell']}")
              
    def _execute_step(self, dt: float):
        """Execute maze solving step"""
        if not self.move_in_progress and not self.turn_in_progress:
            # Plan next move using right-hand rule
            next_move = self._plan_next_move()
            
            if next_move:
                direction, target_cell = next_move
                self._execute_move(direction, target_cell)
            else:
                # No valid moves - try to find alternative or stop
                print("No valid moves available")
                self.is_running = False
                
        # Check if we've reached the goal
        if self.maze_state['current_cell'] == self.maze_state['goal_cell']:
            print("Goal reached!")
            self.is_running = False
            
    def _plan_next_move(self) -> Optional[tuple]:
        """Plan next move using right-hand rule"""
        current_x, current_y = self.maze_state['current_cell']
        
        # Right-hand rule: Try right, straight, left, back
        directions = [
            (self.current_direction + 1) % 4,  # Right
            self.current_direction,            # Straight
            (self.current_direction + 3) % 4, # Left
            (self.current_direction + 2) % 4  # Back
        ]
        
        for direction in directions:
            target_x, target_y = self._move_in_direction(current_x, current_y, direction)
            
            # Check if target cell is valid and not a wall
            if (0 <= target_x < len(self.maze_grid[0]) and 
                0 <= target_y < len(self.maze_grid) and
                self.maze_grid[target_y][target_x] != 1 and
                (target_x, target_y) not in self.maze_state['visited']):
                
                return direction, (target_x, target_y)
                
        # If no unvisited cells available, choose any valid move
        for direction in directions:
            target_x, target_y = self._move_in_direction(current_x, current_y, direction)
            
            if (0 <= target_x < len(self.maze_grid[0]) and 
                0 <= target_y < len(self.maze_grid) and
                self.maze_grid[target_y][target_x] != 1):
                
                return direction, (target_x, target_y)
                
        return None
        
    def _move_in_direction(self, x: int, y: int, direction: int) -> tuple:
        """Calculate target cell coordinates for given direction"""
        dx, dy = self._direction_to_delta(direction)
        return x + dx, y + dy
        
    def _direction_to_delta(self, direction: int) -> tuple:
        """Convert direction to delta coordinates"""
        if direction == 0:  # North
            return 0, -1
        elif direction == 1:  # East
            return 1, 0
        elif direction == 2:  # South
            return 0, 1
        elif direction == 3:  # West
            return -1, 0
        return 0, 0
        
    def _execute_move(self, direction: int, target_cell: tuple):
        """Execute movement to target cell"""
        current_state = self.robot.get_state()
        
        # Calculate target world coordinates
        target_world_x = target_cell[0] * self.cell_size
        target_world_y = target_cell[1] * self.cell_size
        
        # Calculate required turn
        turn_angle = self._calculate_turn_angle(current_state.theta, direction)
        
        if abs(turn_angle) > 0.1:  # Need to turn
            self._execute_turn(turn_angle)
        else:
            self._execute_forward_movement(target_world_x, target_world_y)
            
    def _calculate_turn_angle(self, current_theta: float, target_direction: int) -> float:
        """Calculate required turn angle"""
        target_theta = self._direction_to_angle(target_direction)
        angle_diff = target_theta - current_theta
        
        # Normalize to [-pi, pi]
        while angle_diff > math.pi:
            angle_diff -= 2 * math.pi
        while angle_diff < -math.pi:
            angle_diff += 2 * math.pi
            
        return angle_diff
        
    def _direction_to_angle(self, direction: int) -> float:
        """Convert direction to angle in radians"""
        return direction * (math.pi / 2)
        
    def _execute_turn(self, turn_angle: float):
        """Execute turning motion"""
        self.turn_in_progress = True
        turn_speed = 0.4
        
        if turn_angle > 0:
            # Turn left
            command = ControlCommand(-turn_speed, turn_speed)
        else:
            # Turn right
            command = ControlCommand(turn_speed, -turn_speed)
            
        self.robot.hardware.send_commands(command)
        self.turn_start_time = time.time()
        self.turn_angle = turn_angle
        
        print(f"Turning {math.degrees(turn_angle):.1f} degrees")
        
    def _execute_forward_movement(self, target_x: float, target_y: float):
        """Execute forward movement to target"""
        self.move_in_progress = True
        
        command = ControlCommand(self.movement_speed, self.movement_speed)
        self.robot.hardware.send_commands(command)
        
        self.target_position = (target_x, target_y)
        
        print(f"Moving to cell {self.target_position}")
        
    def _update_movement_state(self, dt: float):
        """Update movement state"""
        current_state = self.robot.get_state()
        
        # Update turn completion
        if self.turn_in_progress:
            # Simplified turn completion check
            # In reality, would use gyroscope or encoder feedback
            elapsed = time.time() - self.turn_start_time
            if elapsed > abs(self.turn_angle) / 0.4:  # Approximate turn time
                self.turn_in_progress = False
                self.current_direction = (self.current_direction + 1) % 4
                print(f"Turn completed, new direction: {self.current_direction}")
                
        # Update movement completion
        if self.move_in_progress:
            distance = math.sqrt(
                (current_state.x - self.target_position[0])**2 + 
                (current_state.y - self.target_position[1])**2
            )
            
            if distance < 0.1:  # Close enough to target
                self.move_in_progress = False
                
                # Update maze state
                self.maze_state['current_cell'] = self.target_position
                self.maze_state['visited'].add(self.target_position)
                
                print(f"Reached cell {self.target_position}")


class RobotFormationDemo(BaseDemo):
    """Multi-robot formation demo"""
    
    def __init__(self, robot: Robot, robots: List[Robot] = None):
        super().__init__(robot, "Robot Formation")
        
        # Formation parameters
        self.robots = robots or [robot]
        self.formation_type = "line"  # line, triangle, square
        self.leader_robot = robot
        self.follower_robots = [r for r in self.robots if r != robot]
        
        # Formation control
        self.formation_spacing = 0.5  # Spacing between robots in meters
        self.follow_speed = 0.3
        
        # Formation positions
        self.formation_positions = []
        
    def _setup(self):
        """Setup formation demo"""
        print(f"Setting up {self.formation_type} formation demo with {len(self.robots)} robots")
        
        # Calculate formation positions
        self._calculate_formation_positions()
        
        # Assign robots to positions
        self.robot_assignments = {}
        for i, robot in enumerate(self.robots):
            if i < len(self.formation_positions):
                self.robot_assignments[robot] = self.formation_positions[i]
            else:
                self.robot_assignments[robot] = (0, 0)  # Default position
                
    def _calculate_formation_positions(self):
        """Calculate formation positions for robots"""
        if self.formation_type == "line":
            # Line formation
            self.formation_positions = [
                (i * self.formation_spacing, 0) 
                for i in range(len(self.robots))
            ]
        elif self.formation_type == "triangle":
            # Triangle formation
            self.formation_positions = [
                (0, 0),  # Leader
                (-self.formation_spacing, -self.formation_spacing),  # Left
                (self.formation_spacing, -self.formation_spacing),   # Right
            ]
            # Add more positions if needed
            while len(self.formation_positions) < len(self.robots):
                self.formation_positions.append((0, 0))
        elif self.formation_type == "square":
            # Square formation
            self.formation_positions = [
                (0, 0),          # Front left
                (self.formation_spacing, 0),  # Front right
                (0, -self.formation_spacing),  # Back left
                (self.formation_spacing, -self.formation_spacing),  # Back right
            ]
            # Add more positions if needed
            while len(self.formation_positions) < len(self.robots):
                self.formation_positions.append((0, 0))
                
    def _execute_step(self, dt: float):
        """Execute formation control step"""
        # Leader robot moves forward
        leader_command = ControlCommand(0.4, 0.4)
        self.leader_robot.hardware.send_commands(leader_command)
        
        # Follower robots maintain formation
        for robot in self.follower_robots:
            target_position = self.robot_assignments[robot]
            current_position = robot.get_state()
            
            # Simple proportional controller for formation keeping
            dx = target_position[0] - current_position.x
            dy = target_position[1] - current_position.y
            
            # Calculate movement command
            kp = 2.0  # Proportional gain
            left_speed = kp * (dy - dx * 0.5)
            right_speed = kp * (dy + dx * 0.5)
            
            # Clamp speeds
            left_speed = max(-self.follow_speed, min(self.follow_speed, left_speed))
            right_speed = max(-self.follow_speed, min(self.follow_speed, right_speed))
            
            command = ControlCommand(left_speed, right_speed)
            robot.hardware.send_commands(command)
            
        if int(time.time() * 2) % 2 == 0:
            print("Maintaining formation...")


# Main demo runner and testing
class DemoRunner:
    """Main demo runner for educational robotics"""
    
    def __init__(self, robot: Robot):
        self.robot = robot
        self.demo_results = {}
        
    def run_demo(self, demo_name: str) -> DemoResult:
        """Run specified demo"""
        if demo_name.lower() == "line_following":
            demo = LineFollowingDemo(self.robot)
        elif demo_name.lower() == "obstacle_avoidance":
            demo = ObstacleAvoidanceDemo(self.robot)
        elif demo_name.lower() == "maze_solving":
            demo = MazeSolvingDemo(self.robot)
        elif demo_name.lower() == "formation":
            demo = RobotFormationDemo(self.robot)
        else:
            raise ValueError(f"Unknown demo: {demo_name}")
            
        result = demo.start()
        self.demo_results[demo_name] = result
        return result
        
    def run_all_demos(self) -> Dict[str, DemoResult]:
        """Run all available demos"""
        demo_names = ["line_following", "obstacle_avoidance", "maze_solving"]
        results = {}
        
        for demo_name in demo_names:
            print(f"\n{'='*50}")
            print(f"Running {demo_name} demo")
            print(f"{'='*50}")
            
            try:
                result = self.run_demo(demo_name)
                results[demo_name] = result
                print(f"Demo {demo_name} completed successfully: {result.success}")
            except Exception as e:
                print(f"Demo {demo_name} failed: {e}")
                results[demo_name] = DemoResult(
                    success=False,
                    duration=0.0,
                    distance_traveled=0.0,
                    final_position=(0.0, 0.0),
                    errors_encountered=[str(e)],
                    performance_metrics={}
                )
                
        return results
        
    def print_summary(self):
        """Print summary of all demo results"""
        print(f"\n{'='*60}")
        print("DEMO EXECUTION SUMMARY")
        print(f"{'='*60}")
        
        for demo_name, result in self.demo_results.items():
            print(f"\n{demo_name.upper()}:")
            print(f"  Success: {result.success}")
            print(f"  Duration: {result.duration:.2f} seconds")
            print(f"  Distance: {result.distance_traveled:.2f} meters")
            print(f"  Final Position: {result.final_position}")
            print(f"  Errors: {len(result.errors_encountered)}")
            
            if result.performance_metrics:
                print("  Performance Metrics:")
                for metric, value in result.performance_metrics.items():
                    if isinstance(value, float):
                        print(f"    {metric}: {value:.3f}")
                    else:
                        print(f"    {metric}: {value}")


if __name__ == "__main__":
    print("Testing Educational Robotics Demos...")
    
    # This would typically use a real robot, but for testing we'll create a simulated one
    from hardware.ev3_hardware import EV3Hardware
    
    # Create robot (simulated EV3)
    hardware = EV3Hardware()
    robot = Robot(hardware)
    
    # Connect robot
    if robot.connect():
        print("Robot connected successfully")
        
        # Create demo runner
        demo_runner = DemoRunner(robot)
        
        try:
            # Run individual demo
            print("\nRunning line following demo...")
            result = demo_runner.run_demo("line_following")
            print(f"Line following result: {result.success}")
            
            # Run all demos
            print("\nRunning all demos...")
            all_results = demo_runner.run_all_demos()
            
            # Print summary
            demo_runner.print_summary()
            
        except KeyboardInterrupt:
            print("\nDemo execution interrupted")
        finally:
            # Cleanup
            robot.disconnect()
            print("Robot disconnected")
    else:
        print("Failed to connect robot")
        
    print("\nDemo testing complete!")
