"""
Core robotics classes and interfaces for the Educational Robotics Framework

This module provides the foundational classes and interfaces that define
the structure and behavior of robots in the framework.
"""

import time
import threading
import json
import numpy as np
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict
from abc import ABC, abstractmethod
from pathlib import Path


@dataclass
class RobotState:
    """Robot state representation with position, velocity, and sensors"""
    x: float = 0.0          # X position in meters
    y: float = 0.0          # Y position in meters
    theta: float = 0.0      # Orientation in radians
    vx: float = 0.0         # Linear velocity x-component
    vy: float = 0.0         # Linear velocity y-component
    omega: float = 0.0      # Angular velocity
    timestamp: float = 0.0  # Timestamp
    
    def __post_init__(self):
        if self.timestamp == 0.0:
            self.timestamp = time.time()


@dataclass
class SensorData:
    """Container for sensor data from multiple sensors"""
    ultrasonic_distance: Optional[float] = None  # Distance in meters
    color_reflectance: Optional[List[float]] = None  # Color sensor reflectance
    imu_acceleration: Optional[List[float]] = None   # IMU acceleration [ax, ay, az]
    imu_gyroscope: Optional[List[float]] = None      # IMU gyroscope [gx, gy, gz]
    imu_magnetometer: Optional[List[float]] = None   # IMU magnetometer [mx, my, mz]
    camera_image: Optional[np.ndarray] = None        # Camera image
    timestamp: float = 0.0
    
    def __post_init__(self):
        if self.timestamp == 0.0:
            self.timestamp = time.time()


@dataclass
class ControlCommand:
    """Control command for robot actuators"""
    left_motor_speed: float = 0.0    # Left motor speed (-1.0 to 1.0)
    right_motor_speed: float = 0.0   # Right motor speed (-1.0 to 1.0)
    servo_angles: List[float] = None  # Servo angles
    timestamp: float = 0.0
    
    def __post_init__(self):
        if self.servo_angles is None:
            self.servo_angles = []
        if self.timestamp == 0.0:
            self.timestamp = time.time()


class BaseHardware(ABC):
    """Abstract base class for robot hardware interfaces"""
    
    @abstractmethod
    def connect(self) -> bool:
        """Connect to the robot hardware"""
        pass
    
    @abstractmethod
    def disconnect(self) -> bool:
        """Disconnect from the robot hardware"""
        pass
    
    @abstractmethod
    def read_sensors(self) -> SensorData:
        """Read data from all sensors"""
        pass
    
    @abstractmethod
    def send_commands(self, command: ControlCommand) -> bool:
        """Send control commands to actuators"""
        pass
    
    @abstractmethod
    def is_connected(self) -> bool:
        """Check if hardware is connected"""
        pass


class BaseSensor(ABC):
    """Abstract base class for robot sensors"""
    
    def __init__(self, name: str, sampling_rate: float = 10.0):
        self.name = name
        self.sampling_rate = sampling_rate
        self.is_active = False
        self._last_reading = None
        
    @abstractmethod
    def read(self) -> Any:
        """Read sensor data"""
        pass
    
    def start(self):
        """Start sensor reading"""
        self.is_active = True
        
    def stop(self):
        """Stop sensor reading"""
        self.is_active = False


class BaseActuator(ABC):
    """Abstract base class for robot actuators"""
    
    def __init__(self, name: str):
        self.name = name
        self.current_value = 0.0
        
    @abstractmethod
    def set_value(self, value: float) -> bool:
        """Set actuator value"""
        pass
    
    @abstractmethod
    def get_value(self) -> float:
        """Get current actuator value"""
        pass


class Robot:
    """Main robot class that integrates hardware, sensors, and control"""
    
    def __init__(self, hardware: BaseHardware, config: Optional[Dict] = None):
        self.hardware = hardware
        self.config = config or {}
        self.state = RobotState()
        self.is_running = False
        self._control_thread = None
        self._sensor_thread = None
        self._callbacks = {
            'state_update': [],
            'sensor_update': [],
            'error': []
        }
        
    def connect(self) -> bool:
        """Connect to robot hardware"""
        try:
            if self.hardware.connect():
                self.is_running = True
                self._start_threads()
                return True
            return False
        except Exception as e:
            self._trigger_callback('error', f"Connection failed: {e}")
            return False
            
    def disconnect(self):
        """Disconnect from robot hardware"""
        self.is_running = False
        self._stop_threads()
        self.hardware.disconnect()
        
    def _start_threads(self):
        """Start sensor reading and control threads"""
        self._control_thread = threading.Thread(target=self._control_loop, daemon=True)
        self._sensor_thread = threading.Thread(target=self._sensor_loop, daemon=True)
        
        self._control_thread.start()
        self._sensor_thread.start()
        
    def _stop_threads(self):
        """Stop all threads"""
        # Threads will terminate when is_running becomes False
        
    def _control_loop(self):
        """Main control loop - runs continuously when robot is active"""
        while self.is_running:
            try:
                # Read current state
                sensor_data = self.hardware.read_sensors()
                
                # Update robot state based on sensor data
                self._update_state(sensor_data)
                
                # Execute control logic (to be implemented by subclasses)
                command = self._compute_control_command()
                
                # Send commands to hardware
                self.hardware.send_commands(command)
                
                # Trigger callbacks
                self._trigger_callback('state_update', self.state)
                
                time.sleep(0.02)  # 50 Hz control frequency
                
            except Exception as e:
                self._trigger_callback('error', f"Control loop error: {e}")
                time.sleep(0.1)
                
    def _sensor_loop(self):
        """Sensor reading loop"""
        while self.is_running:
            try:
                sensor_data = self.hardware.read_sensors()
                self._trigger_callback('sensor_update', sensor_data)
                time.sleep(0.1)  # 10 Hz sensor frequency
            except Exception as e:
                self._trigger_callback('error', f"Sensor loop error: {e}")
                time.sleep(0.1)
                
    def _update_state(self, sensor_data: SensorData):
        """Update robot state based on sensor readings"""
        # This is a basic implementation - subclasses should override with more sophisticated state estimation
        self.state.timestamp = sensor_data.timestamp
        
        # Update position based on motor encoders (simplified)
        # In a real implementation, this would use odometry calculations
        # For now, we'll keep position constant for demonstration
        
    def _compute_control_command(self) -> ControlCommand:
        """Compute control commands based on current state and goals"""
        # This should be overridden by subclasses to implement specific control strategies
        return ControlCommand(0.0, 0.0)
        
    def register_callback(self, event: str, callback):
        """Register callback function for events"""
        if event in self._callbacks:
            self._callbacks[event].append(callback)
            
    def _trigger_callback(self, event: str, data: Any):
        """Trigger registered callbacks for an event"""
        if event in self._callbacks:
            for callback in self._callbacks[event]:
                try:
                    callback(data)
                except Exception as e:
                    print(f"Callback error: {e}")
                    
    def get_state(self) -> RobotState:
        """Get current robot state"""
        return self.state
        
    def get_sensors(self) -> SensorData:
        """Get latest sensor readings"""
        return self.hardware.read_sensors()
        
    def save_log(self, filename: str):
        """Save robot operation log"""
        log_data = {
            'state_history': [asdict(state) for state in getattr(self, '_state_history', [])],
            'config': self.config,
            'start_time': getattr(self, '_start_time', time.time())
        }
        
        log_path = Path(filename)
        log_path.parent.mkdir(exist_ok=True)
        
        with open(log_path, 'w') as f:
            json.dump(log_data, f, indent=2)
            
    def stop(self):
        """Stop robot operation"""
        self.disconnect()
        
    def __enter__(self):
        return self
        
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.disconnect()