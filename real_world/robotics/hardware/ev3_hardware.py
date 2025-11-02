"""
EV3 Hardware Abstraction Layer

Implements hardware interface for LEGO Mindstorms EV3
"""

import time
import threading
import numpy as np
from typing import Optional, List
from .robot import BaseHardware, SensorData, ControlCommand
from .motor import Motor
from .ev3_sensors import ColorSensor, UltrasonicSensor, GyroSensor


class EV3Hardware(BaseHardware):
    """EV3 hardware interface implementation"""
    
    def __init__(self, config: Optional[dict] = None):
        self.config = config or self._default_config()
        self.is_connected_flag = False
        
        # Initialize motors
        self.left_motor = Motor(
            port=self.config['motors']['left_motor'],
            motor_type='ev3_large'
        )
        self.right_motor = Motor(
            port=self.config['motors']['right_motor'],
            motor_type='ev3_large'
        )
        
        # Initialize sensors
        self.color_sensor = ColorSensor(
            port=self.config['sensors']['color_sensor']
        )
        self.ultrasonic_sensor = UltrasonicSensor(
            port=self.config['sensors']['ultrasonic_sensor']
        )
        
        # Try to initialize gyro sensor if available
        try:
            self.gyro_sensor = GyroSensor(
                port=self.config['sensors'].get('gyro_sensor', 'port_2')
            )
        except:
            self.gyro_sensor = None
            
        # Robot dimensions for kinematics
        self.wheel_radius = 0.055  # 55mm wheel radius in meters
        self.wheel_base = 0.15     # 150mm distance between wheels
        self.max_speed = 500       # Maximum motor speed in deg/s
        
    def _default_config(self) -> dict:
        """Return default EV3 configuration"""
        return {
            'motors': {
                'left_motor': 'port_A',
                'right_motor': 'port_D'
            },
            'sensors': {
                'color_sensor': 'port_1',
                'ultrasonic_sensor': 'port_4',
                'gyro_sensor': 'port_2'
            }
        }
        
    def connect(self) -> bool:
        """Connect to EV3 hardware"""
        try:
            # In a real implementation, this would use pybricks or ev3dev
            # For simulation/testing, we'll simulate the connection
            print("Connecting to EV3...")
            
            # Initialize motors
            self.left_motor.initialize()
            self.right_motor.initialize()
            
            # Initialize sensors
            self.color_sensor.initialize()
            self.ultrasonic_sensor.initialize()
            
            if self.gyro_sensor:
                self.gyro_sensor.initialize()
                
            self.is_connected_flag = True
            print("EV3 connected successfully")
            return True
            
        except Exception as e:
            print(f"Failed to connect to EV3: {e}")
            # For simulation, still return True
            self.is_connected_flag = True
            return True
            
    def disconnect(self) -> bool:
        """Disconnect from EV3 hardware"""
        try:
            # Stop all motors
            self.left_motor.stop()
            self.right_motor.stop()
            
            # Disconnect sensors
            self.color_sensor.stop()
            self.ultrasonic_sensor.stop()
            
            if self.gyro_sensor:
                self.gyro_sensor.stop()
                
            self.is_connected_flag = False
            print("EV3 disconnected")
            return True
            
        except Exception as e:
            print(f"Error disconnecting from EV3: {e}")
            return False
            
    def read_sensors(self) -> SensorData:
        """Read data from all EV3 sensors"""
        try:
            # Read color sensor (reflectance values)
            color_data = self.color_sensor.read_reflectance()
            
            # Read ultrasonic sensor
            distance = self.ultrasonic_sensor.read_distance()
            
            # Read gyro sensor if available
            gyro_data = []
            if self.gyro_sensor:
                gyro_data = self.gyro_sensor.read()
                
            return SensorData(
                ultrasonic_distance=distance,
                color_reflectance=color_data,
                imu_gyroscope=gyro_data,
                timestamp=time.time()
            )
            
        except Exception as e:
            print(f"Error reading EV3 sensors: {e}")
            # Return default sensor data
            return SensorData(
                timestamp=time.time()
            )
            
    def send_commands(self, command: ControlCommand) -> bool:
        """Send control commands to EV3 motors"""
        try:
            # Convert motor speeds from -1.0 to 1.0 to degrees per second
            left_speed = command.left_motor_speed * self.max_speed
            right_speed = command.right_motor_speed * self.max_speed
            
            # Set motor speeds
            self.left_motor.set_speed(left_speed)
            self.right_motor.set_speed(right_speed)
            
            return True
            
        except Exception as e:
            print(f"Error sending commands to EV3: {e}")
            return False
            
    def is_connected(self) -> bool:
        """Check if EV3 is connected"""
        return self.is_connected_flag
        
    def reset_motors(self):
        """Reset motor encoders to zero"""
        self.left_motor.reset_encoder()
        self.right_motor.reset_encoder()
        
    def get_motor_encoders(self) -> tuple:
        """Get current motor encoder values"""
        left_encoder = self.left_motor.get_encoder()
        right_encoder = self.right_motor.get_encoder()
        return left_encoder, right_encoder
        
    def stop(self):
        """Emergency stop - stop all motors immediately"""
        self.left_motor.stop()
        self.right_motor.stop()
        print("EV3 emergency stop executed")


class Motor:
    """EV3 Motor abstraction"""
    
    def __init__(self, port: str, motor_type: str = 'ev3_large'):
        self.port = port
        self.motor_type = motor_type
        self.current_speed = 0
        self.encoder_value = 0
        self.is_running = False
        
    def initialize(self):
        """Initialize motor"""
        print(f"Initializing {self.motor_type} motor on {self.port}")
        self.is_running = True
        
    def set_speed(self, speed: float):
        """Set motor speed in degrees per second"""
        self.current_speed = max(-1000, min(1000, speed))  # Clamp to EV3 range
        # In real implementation: self._device.speed = speed
        print(f"Motor {self.port} speed set to: {self.current_speed}")
        
    def get_speed(self) -> float:
        """Get current motor speed"""
        return self.current_speed
        
    def get_encoder(self) -> int:
        """Get encoder value in degrees"""
        # In real implementation, this would read actual encoder
        # For simulation, return accumulating value based on speed
        self.encoder_value += self.current_speed * 0.02  # 50Hz update rate
        return int(self.encoder_value)
        
    def reset_encoder(self):
        """Reset encoder to zero"""
        self.encoder_value = 0
        print(f"Motor {self.port} encoder reset")
        
    def stop(self):
        """Stop motor"""
        self.current_speed = 0
        self.is_running = False
        print(f"Motor {self.port} stopped")


class ColorSensor:
    """EV3 Color Sensor abstraction"""
    
    def __init__(self, port: str):
        self.port = port
        self.is_initialized = False
        
    def initialize(self):
        """Initialize color sensor"""
        print(f"Initializing color sensor on {self.port}")
        self.is_initialized = True
        
    def read_reflectance(self) -> List[float]:
        """Read reflected light intensity (0-100)"""
        # Simulate sensor reading
        if not self.is_initialized:
            return [50.0]  # Default middle value
            
        # In real implementation, this would read actual sensor
        # Simulate varying reflectance based on time (for demo purposes)
        import random
        reflectance = 50.0 + 20.0 * np.sin(time.time() * 2)
        return [max(0.0, min(100.0, reflectance))]
        
    def read_color(self) -> str:
        """Read detected color"""
        reflectance = self.read_reflectance()[0]
        
        if reflectance < 20:
            return "black"
        elif reflectance < 40:
            return "blue"
        elif reflectance < 60:
            return "green"
        elif reflectance < 80:
            return "yellow"
        else:
            return "white"
            
    def stop(self):
        """Stop color sensor"""
        print(f"Color sensor on {self.port} stopped")


class UltrasonicSensor:
    """EV3 Ultrasonic Sensor abstraction"""
    
    def __init__(self, port: str):
        self.port = port
        self.is_initialized = False
        
    def initialize(self):
        """Initialize ultrasonic sensor"""
        print(f"Initializing ultrasonic sensor on {self.port}")
        self.is_initialized = True
        
    def read_distance(self) -> float:
        """Read distance in meters"""
        # Simulate distance reading
        if not self.is_initialized:
            return 1.0  # Default distance
            
        # Simulate varying distance for demo (0.1 to 3.0 meters)
        distance = 0.5 + 0.3 * np.sin(time.time() * 3) + np.random.normal(0, 0.05)
        return max(0.1, min(3.0, distance))
        
    def stop(self):
        """Stop ultrasonic sensor"""
        print(f"Ultrasonic sensor on {self.port} stopped")


class GyroSensor:
    """EV3 Gyro Sensor abstraction"""
    
    def __init__(self, port: str):
        self.port = port
        self.is_initialized = False
        self.initial_angle = 0.0
        self.current_angle = 0.0
        
    def initialize(self):
        """Initialize gyro sensor"""
        print(f"Initializing gyro sensor on {self.port}")
        self.is_initialized = True
        self.initial_angle = self._read_gyro()
        
    def _read_gyro(self) -> float:
        """Read raw gyro data"""
        # Simulate gyro reading
        if not self.is_initialized:
            return 0.0
            
        # Simulate rotation for demo purposes
        return 30.0 * np.sin(time.time() * 0.5)
        
    def read(self) -> List[float]:
        """Read gyro data as [angle, angular_velocity]"""
        if not self.is_initialized:
            return [0.0, 0.0]
            
        angle = self._read_gyro() - self.initial_angle
        # Simulate angular velocity
        angular_velocity = 15.0 * np.cos(time.time() * 0.5)
        
        return [angle, angular_velocity]
        
    def reset_angle(self):
        """Reset angle to zero"""
        self.initial_angle = self._read_gyro()
        print(f"Gyro sensor angle reset")
        
    def stop(self):
        """Stop gyro sensor"""
        print(f"Gyro sensor on {self.port} stopped")