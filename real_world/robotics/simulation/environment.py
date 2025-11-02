"""
Simulation Environment for Educational Robotics

Provides physics simulation and visualization for safe robot experimentation
"""

import numpy as np
import pygame
import math
import time
import threading
from typing import List, Tuple, Optional, Dict, Any, Callable
from dataclasses import dataclass, asdict
from abc import ABC, abstractmethod


@dataclass
class Vector2D:
    """2D Vector class"""
    x: float = 0.0
    y: float = 0.0
    
    def __add__(self, other):
        return Vector2D(self.x + other.x, self.y + other.y)
        
    def __sub__(self, other):
        return Vector2D(self.x - other.x, self.y - other.y)
        
    def __mul__(self, scalar):
        return Vector2D(self.x * scalar, self.y * scalar)
        
    def magnitude(self):
        return math.sqrt(self.x * self.x + self.y * self.y)
        
    def normalize(self):
        mag = self.magnitude()
        if mag > 0:
            return Vector2D(self.x / mag, self.y / mag)
        return Vector2D(0, 0)
        
    def angle(self):
        return math.atan2(self.y, self.x)


@dataclass
class RobotConfig:
    """Configuration for simulated robot"""
    width: float = 0.2        # Robot width in meters
    length: float = 0.3       # Robot length in meters
    max_speed: float = 1.0    # Maximum speed in m/s
    max_acceleration: float = 2.0  # Maximum acceleration in m/s²
    wheel_radius: float = 0.05  # Wheel radius in meters
    wheel_base: float = 0.2     # Distance between wheels in meters
    mass: float = 1.0           # Robot mass in kg
    color: Tuple[int, int, int] = (0, 100, 255)  # RGB color


@dataclass
class SimulationConfig:
    """Configuration for simulation environment"""
    world_width: float = 10.0      # World width in meters
    world_height: float = 8.0      # World height in meters
    grid_size: float = 1.0         # Grid size for visualization
    physics_enabled: bool = True   # Enable physics simulation
    time_step: float = 0.02        # Simulation time step in seconds
    pixel_per_meter: float = 50     # Pixels per meter for rendering
    background_color: Tuple[int, int, int] = (240, 240, 240)
    grid_color: Tuple[int, int, int] = (200, 200, 200)
    obstacle_color: Tuple[int, int, int] = (100, 100, 100)


@dataclass
class Obstacle:
    """Obstacle in simulation environment"""
    position: Vector2D
    width: float
    height: float
    color: Tuple[int, int, int] = (100, 100, 100)
    is_dynamic: bool = False


class PhysicsObject:
    """Base class for physics objects"""
    
    def __init__(self, position: Vector2D, mass: float = 1.0):
        self.position = position
        self.velocity = Vector2D(0, 0)
        self.acceleration = Vector2D(0, 0)
        self.mass = mass
        self.is_static = mass <= 0
        
    def apply_force(self, force: Vector2D):
        """Apply force to object"""
        if not self.is_static and self.mass > 0:
            self.acceleration = force * (1.0 / self.mass)
            
    def update(self, dt: float):
        """Update physics state"""
        if not self.is_static:
            # Update velocity
            self.velocity = self.velocity + self.acceleration * dt
            # Update position
            self.position = self.position + self.velocity * dt
            # Reset acceleration
            self.acceleration = Vector2D(0, 0)


class SimulatedRobot(PhysicsObject):
    """Simulated robot with differential drive"""
    
    def __init__(self, config: RobotConfig, position: Vector2D):
        super().__init__(position, config.mass)
        self.config = config
        
        # Robot state
        self.orientation = 0.0  # Radians
        self.angular_velocity = 0.0
        self.left_wheel_speed = 0.0
        self.right_wheel_speed = 0.0
        
        # Sensors (simulated)
        self.sensors = {
            'ultrasonic': {'position': Vector2D(0.1, 0), 'range': 3.0, 'angle': 0.0},
            'imu': {'position': Vector2D(0, 0), 'noise': 0.05},
            'camera': {'position': Vector2D(0.05, 0), 'fov': math.pi/3, 'range': 5.0}
        }
        
        # Control state
        self.target_left_speed = 0.0
        self.target_right_speed = 0.0
        
    def set_wheel_speeds(self, left_speed: float, right_speed: float):
        """Set target wheel speeds"""
        self.target_left_speed = max(-self.config.max_speed, 
                                   min(self.config.max_speed, left_speed))
        self.target_right_speed = max(-self.config.max_speed,
                                    min(self.config.max_speed, right_speed))
        
    def update_drive(self, dt: float):
        """Update differential drive system"""
        # Simple motor model with acceleration limits
        max_accel = self.config.max_acceleration
        
        # Update left wheel
        speed_error = self.target_left_speed - self.left_wheel_speed
        accel = max(-max_accel, min(max_accel, speed_error * 10))  # PD-like controller
        self.left_wheel_speed += accel * dt
        
        # Update right wheel
        speed_error = self.target_right_speed - self.right_wheel_speed
        accel = max(-max_accel, min(max_accel, speed_error * 10))
        self.right_wheel_speed += accel * dt
        
        # Calculate robot motion from wheel speeds
        wheel_radius = self.config.wheel_radius
        wheel_base = self.config.wheel_base
        
        # Linear and angular velocities
        v_linear = wheel_radius * (self.left_wheel_speed + self.right_wheel_speed) / 2
        v_angular = wheel_radius * (self.right_wheel_speed - self.left_wheel_speed) / wheel_base
        
        # Update orientation
        self.angular_velocity = v_angular
        self.orientation += v_angular * dt
        
        # Update position
        vx = v_linear * math.cos(self.orientation)
        vy = v_linear * math.sin(self.orientation)
        
        self.velocity = Vector2D(vx, vy)
        self.position = self.position + self.velocity * dt
        
    def read_ultrasonic_sensor(self, obstacles: List[Obstacle]) -> float:
        """Simulate ultrasonic sensor reading"""
        sensor_config = self.sensors['ultrasonic']
        sensor_pos = sensor_config['position']
        sensor_range = sensor_config['range']
        sensor_angle = sensor_config['angle']
        
        # Transform sensor position to world coordinates
        cos_angle = math.cos(self.orientation + sensor_angle)
        sin_angle = math.sin(self.orientation + sensor_angle)
        world_sensor_x = self.position.x + sensor_pos.x * cos_angle - sensor_pos.y * sin_angle
        world_sensor_y = self.position.y + sensor_pos.x * sin_angle + sensor_pos.y * cos_angle
        
        # Find closest obstacle in sensor direction
        min_distance = sensor_range
        
        for obstacle in obstacles:
            # Calculate distance from sensor to obstacle
            obs_center_x = obstacle.position.x + obstacle.width / 2
            obs_center_y = obstacle.position.y + obstacle.height / 2
            
            # Simple distance calculation (ray-box intersection)
            dx = obs_center_x - world_sensor_x
            dy = obs_center_y - world_sensor_y
            distance = math.sqrt(dx * dx + dy * dy)
            
            if distance < min_distance:
                min_distance = distance
                
        # Add noise
        noise = np.random.normal(0, 0.02)
        return max(0.02, min(sensor_range, min_distance + noise))
        
    def read_imu_sensor(self) -> Dict[str, List[float]]:
        """Simulate IMU sensor reading"""
        sensor_config = self.sensors['imu']
        noise = sensor_config['noise']
        
        # Simulate accelerometer (gravity + motion)
        gravity_x = 9.8 * math.sin(self.orientation)
        gravity_y = 9.8 * math.cos(self.orientation)
        
        acceleration = [
            gravity_x + self.acceleration.x + np.random.normal(0, noise),
            gravity_y + self.acceleration.y + np.random.normal(0, noise),
            9.8 + np.random.normal(0, noise)
        ]
        
        # Simulate gyroscope
        gyroscope = [
            self.angular_velocity + np.random.normal(0, noise),
            0.1 * math.sin(time.time()) + np.random.normal(0, noise),
            0.2 * math.cos(time.time() * 0.5) + np.random.normal(0, noise)
        ]
        
        return {
            'acceleration': acceleration,
            'gyroscope': gyroscope
        }
        
    def get_position(self) -> Vector2D:
        """Get robot position"""
        return self.position
        
    def get_orientation(self) -> float:
        """Get robot orientation"""
        return self.orientation


class PhysicsEngine:
    """Simple physics engine for simulation"""
    
    def __init__(self, config: SimulationConfig):
        self.config = config
        self.objects = []
        self.obstacles = []
        
    def add_object(self, obj):
        """Add physics object"""
        self.objects.append(obj)
        
    def add_obstacle(self, obstacle: Obstacle):
        """Add static obstacle"""
        self.obstacles.append(obstacle)
        
    def update(self, dt: float):
        """Update all physics objects"""
        for obj in self.objects:
            obj.update(dt)
            
        # Handle collisions (simplified)
        self._handle_collisions()
        
    def _handle_collisions(self):
        """Handle simple collision detection and response"""
        for i, obj1 in enumerate(self.objects):
            for j, obj2 in enumerate(self.objects[i+1:], i+1):
                self._check_collision(obj1, obj2)
                
            # Check obstacle collisions
            for obstacle in self.obstacles:
                self._check_obstacle_collision(obj1, obstacle)
                
    def _check_collision(self, obj1, obj2):
        """Check collision between two objects (simplified)"""
        # Simple bounding box collision
        if (abs(obj1.position.x - obj2.position.x) < 0.5 and 
            abs(obj1.position.y - obj2.position.y) < 0.5):
            
            # Simple collision response - separate objects
            dx = obj2.position.x - obj1.position.x
            dy = obj2.position.y - obj1.position.y
            distance = math.sqrt(dx * dx + dy * dy)
            
            if distance > 0:
                overlap = 0.5 - distance / 2
                separation = Vector2D(dx / distance * overlap, dy / distance * overlap)
                
                obj1.position = obj1.position - separation
                obj2.position = obj2.position + separation
                
                # Damp velocities
                obj1.velocity = obj1.velocity * 0.5
                obj2.velocity = obj2.velocity * 0.5
                
    def _check_obstacle_collision(self, obj, obstacle):
        """Check collision with obstacle"""
        # Simple AABB collision
        obj_left = obj.position.x - 0.1
        obj_right = obj.position.x + 0.1
        obj_top = obj.position.y - 0.1
        obj_bottom = obj.position.y + 0.1
        
        obs_left = obstacle.position.x
        obs_right = obstacle.position.x + obstacle.width
        obs_top = obstacle.position.y
        obs_bottom = obstacle.position.y + obstacle.height
        
        if (obj_left < obs_right and obj_right > obs_left and
            obj_top < obs_bottom and obj_bottom > obs_top):
            
            # Simple collision response
            obj.velocity = Vector2D(-obj.velocity.x * 0.5, -obj.velocity.y * 0.5)


class SimulationRenderer:
    """Pygame-based renderer for simulation"""
    
    def __init__(self, config: SimulationConfig):
        self.config = config
        
        # Initialize pygame
        pygame.init()
        
        # Calculate screen dimensions
        screen_width = int(config.world_width * config.pixel_per_meter)
        screen_height = int(config.world_height * config.pixel_per_meter)
        
        # Create screen
        self.screen = pygame.display.set_mode((screen_width, screen_height))
        pygame.display.set_caption("Educational Robotics Simulation")
        
        # Create font for text rendering
        self.font = pygame.font.Font(None, 24)
        
        # Clock for timing
        self.clock = pygame.time.Clock()
        
    def world_to_screen(self, position: Vector2D) -> Tuple[int, int]:
        """Convert world coordinates to screen coordinates"""
        x = int(position.x * self.config.pixel_per_meter)
        y = int(self.config.world_height * self.config.pixel_per_meter - position.y * self.config.pixel_per_meter)
        return (x, y)
        
    def draw_grid(self):
        """Draw grid on screen"""
        grid_color = self.config.grid_color
        
        # Vertical lines
        for x in np.arange(0, self.config.world_width + self.config.grid_size, self.config.grid_size):
            start = self.world_to_screen(Vector2D(x, 0))
            end = self.world_to_screen(Vector2D(x, self.config.world_height))
            pygame.draw.line(self.screen, grid_color, start, end)
            
        # Horizontal lines
        for y in np.arange(0, self.config.world_height + self.config.grid_size, self.config.grid_size):
            start = self.world_to_screen(Vector2D(0, y))
            end = self.world_to_screen(Vector2D(self.config.world_width, y))
            pygame.draw.line(self.screen, grid_color, start, end)
            
    def draw_obstacles(self, obstacles: List[Obstacle]):
        """Draw obstacles"""
        for obstacle in obstacles:
            screen_pos = self.world_to_screen(obstacle.position)
            width_px = int(obstacle.width * self.config.pixel_per_meter)
            height_px = int(obstacle.height * self.config.pixel_per_meter)
            
            pygame.draw.rect(self.screen, obstacle.color, 
                           (screen_pos[0], screen_pos[1], width_px, height_px))
            
    def draw_robot(self, robot: SimulatedRobot):
        """Draw robot"""
        # Robot body
        pos = self.world_to_screen(robot.position)
        robot_width = int(robot.config.width * self.config.pixel_per_meter)
        robot_height = int(robot.config.length * self.config.pixel_per_meter)
        
        rect = pygame.Rect(pos[0] - robot_width//2, pos[1] - robot_height//2, 
                          robot_width, robot_height)
        pygame.draw.rect(self.screen, robot.config.color, rect)
        
        # Robot orientation indicator
        end_pos = self.world_to_screen(Vector2D(
            robot.position.x + math.cos(robot.orientation) * 0.15,
            robot.position.y + math.sin(robot.orientation) * 0.15
        ))
        
        pygame.draw.line(self.screen, (255, 0, 0), pos, end_pos, 3)
        
        # Wheel visualization
        wheel_radius = int(robot.config.wheel_radius * self.config.pixel_per_meter)
        wheel_offset = int(robot.config.wheel_base * self.config.pixel_per_meter) // 2
        
        # Left wheel
        left_wheel_pos = self.world_to_screen(Vector2D(
            robot.position.x - math.sin(robot.orientation) * wheel_offset * 0.1,
            robot.position.y + math.cos(robot.orientation) * wheel_offset * 0.1
        ))
        pygame.draw.circle(self.screen, (50, 50, 50), left_wheel_pos, wheel_radius)
        
        # Right wheel
        right_wheel_pos = self.world_to_screen(Vector2D(
            robot.position.x + math.sin(robot.orientation) * wheel_offset * 0.1,
            robot.position.y - math.cos(robot.orientation) * wheel_offset * 0.1
        ))
        pygame.draw.circle(self.screen, (50, 50, 50), right_wheel_pos, wheel_radius)
        
    def draw_sensors(self, robot: SimulatedRobot):
        """Draw sensor ranges"""
        # Ultrasonic sensor range
        ultra_config = robot.sensors['ultrasonic']
        sensor_pos = self.world_to_screen(Vector2D(
            robot.position.x + ultra_config['position'].x * math.cos(robot.orientation) - 
                    ultra_config['position'].y * math.sin(robot.orientation),
            robot.position.y + ultra_config['position'].x * math.sin(robot.orientation) + 
                    ultra_config['position'].y * math.cos(robot.orientation)
        ))
        
        range_px = int(ultra_config['range'] * self.config.pixel_per_meter)
        pygame.draw.circle(self.screen, (255, 255, 0), sensor_pos, range_px, 1)
        
    def draw_text(self, text: str, position: Tuple[int, int], color: Tuple[int, int, int] = (0, 0, 0)):
        """Draw text on screen"""
        text_surface = self.font.render(text, True, color)
        self.screen.blit(text_surface, position)
        
    def render_frame(self, robot: SimulatedRobot, obstacles: List[Obstacle]):
        """Render complete frame"""
        # Clear screen
        self.screen.fill(self.config.background_color)
        
        # Draw grid
        self.draw_grid()
        
        # Draw obstacles
        self.draw_obstacles(obstacles)
        
        # Draw robot and sensors
        self.draw_robot(robot)
        self.draw_sensors(robot)
        
        # Draw status information
        status_y = 10
        self.draw_text(f"Position: ({robot.position.x:.2f}, {robot.position.y:.2f})", 
                      (10, status_y))
        self.draw_text(f"Orientation: {math.degrees(robot.orientation):.1f}°", 
                      (10, status_y + 25))
        self.draw_text(f"Speed: {robot.left_wheel_speed:.2f}, {robot.right_wheel_speed:.2f}", 
                      (10, status_y + 50))
                      
        # Update display
        pygame.display.flip()


class SimulationEnvironment:
    """Main simulation environment class"""
    
    def __init__(self, robot_config: Optional[RobotConfig] = None, 
                 sim_config: Optional[SimulationConfig] = None):
        self.robot_config = robot_config or RobotConfig()
        self.sim_config = sim_config or SimulationConfig()
        
        # Initialize components
        self.physics_engine = PhysicsEngine(self.sim_config)
        self.renderer = SimulationRenderer(self.sim_config)
        
        # Simulation state
        self.robot = None
        self.obstacles = []
        self.is_running = False
        self.simulation_time = 0.0
        
        # User callbacks
        self.update_callbacks = []
        self.control_callbacks = {}
        
    def create_robot(self, position: Tuple[float, float] = (1.0, 1.0)) -> SimulatedRobot:
        """Create simulated robot"""
        robot = SimulatedRobot(self.robot_config, Vector2D(position[0], position[1]))
        self.robot = robot
        self.physics_engine.add_object(robot)
        return robot
        
    def add_obstacle(self, x: float, y: float, width: float, height: float,
                    color: Tuple[int, int, int] = (100, 100, 100)):
        """Add obstacle to simulation"""
        obstacle = Obstacle(Vector2D(x, y), width, height, color)
        self.obstacles.append(obstacle)
        self.physics_engine.add_obstacle(obstacle)
        
    def add_control_handler(self, key: int, callback: Callable[[SimulatedRobot], None]):
        """Add keyboard control handler"""
        self.control_callbacks[key] = callback
        
    def add_update_callback(self, callback: Callable[[SimulatedRobot, float], None]):
        """Add update callback"""
        self.update_callbacks.append(callback)
        
    def start(self):
        """Start simulation"""
        if self.robot is None:
            self.create_robot()
            
        self.is_running = True
        self._simulation_loop()
        
    def _simulation_loop(self):
        """Main simulation loop"""
        running = True
        
        while running and self.is_running:
            start_time = time.time()
            
            # Handle pygame events
            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    running = False
                elif event.type == pygame.KEYDOWN:
                    if event.key in self.control_callbacks:
                        self.control_callbacks[event.key](self.robot)
                        
            # Update physics
            dt = self.sim_config.time_step
            
            if self.sim_config.physics_enabled:
                # Update robot drive system
                self.robot.update_drive(dt)
                # Update physics
                self.physics_engine.update(dt)
                
            # Run user callbacks
            for callback in self.update_callbacks:
                callback(self.robot, dt)
                
            # Render frame
            self.renderer.render_frame(self.robot, self.obstacles)
            
            # Control timing
            elapsed = time.time() - start_time
            sleep_time = max(0, self.sim_config.time_step - elapsed)
            if sleep_time > 0:
                time.sleep(sleep_time)
                
            self.simulation_time += dt
            
        self.is_running = False
        pygame.quit()
        
    def stop(self):
        """Stop simulation"""
        self.is_running = False
        
    def reset(self):
        """Reset simulation"""
        self.simulation_time = 0.0
        
        # Reset robot
        if self.robot:
            self.robot.position = Vector2D(1.0, 1.0)
            self.robot.velocity = Vector2D(0, 0)
            self.robot.orientation = 0.0
            self.robot.left_wheel_speed = 0.0
            self.robot.right_wheel_speed = 0.0
            self.robot.target_left_speed = 0.0
            self.robot.target_right_speed = 0.0
            
    def get_state(self) -> Dict[str, Any]:
        """Get current simulation state"""
        if not self.robot:
            return {}
            
        return {
            'robot': {
                'position': (self.robot.position.x, self.robot.position.y),
                'orientation': self.robot.orientation,
                'velocity': (self.robot.velocity.x, self.robot.velocity.y),
                'left_wheel_speed': self.robot.left_wheel_speed,
                'right_wheel_speed': self.robot.right_wheel_speed
            },
            'simulation_time': self.simulation_time,
            'obstacles': len(self.obstacles)
        }


# Example usage and testing
if __name__ == "__main__":
    print("Testing Simulation Environment...")
    
    # Create robot configuration
    robot_config = RobotConfig(
        width=0.2,
        length=0.3,
        max_speed=1.0,
        color=(0, 100, 255)
    )
    
    # Create simulation environment
    sim_env = SimulationEnvironment(robot_config)
    
    # Create robot
    robot = sim_env.create_robot((2.0, 2.0))
    
    # Add obstacles
    sim_env.add_obstacle(5.0, 1.0, 1.0, 0.5, (200, 100, 100))
    sim_env.add_obstacle(3.0, 4.0, 0.5, 1.0, (100, 200, 100))
    sim_env.add_obstacle(7.0, 6.0, 1.5, 0.3, (100, 100, 200))
    
    # Add keyboard controls
    def move_forward(robot):
        robot.set_wheel_speeds(0.5, 0.5)
        print("Moving forward")
        
    def turn_left(robot):
        robot.set_wheel_speeds(-0.3, 0.3)
        print("Turning left")
        
    def turn_right(robot):
        robot.set_wheel_speeds(0.3, -0.3)
        print("Turning right")
        
    def stop_robot(robot):
        robot.set_wheel_speeds(0.0, 0.0)
        print("Stopping")
        
    sim_env.add_control_handler(pygame.K_w, move_forward)
    sim_env.add_control_handler(pygame.K_a, turn_left)
    sim_env.add_control_handler(pygame.K_d, turn_right)
    sim_env.add_control_handler(pygame.K_s, stop_robot)
    
    # Add update callback
    def print_status(robot, dt):
        if int(sim_env.simulation_time * 2) % 2 == 0:  # Print every 0.5 seconds
            print(f"Time: {sim_env.simulation_time:.1f}s, "
                  f"Position: ({robot.position.x:.2f}, {robot.position.y:.2f})")
                  
    sim_env.add_update_callback(print_status)
    
    print("Simulation controls:")
    print("  W - Move forward")
    print("  A - Turn left")
    print("  D - Turn right")
    print("  S - Stop")
    print("  Close window to exit")
    
    try:
        # Start simulation
        sim_env.start()
    except KeyboardInterrupt:
        print("\nSimulation interrupted")
    finally:
        sim_env.stop()
        
    print("Simulation testing complete!")
