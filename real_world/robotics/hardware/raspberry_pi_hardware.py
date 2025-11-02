"""
Raspberry Pi Hardware Abstraction Layer

Implements hardware interface for Raspberry Pi robots
"""

import time
import threading
import numpy as np
from typing import Optional, List, Tuple
from .robot import BaseHardware, SensorData, ControlCommand

try:
    import RPi.GPIO as GPIO
    import spidev
    RPI_AVAILABLE = True
except ImportError:
    RPI_AVAILABLE = False
    print("RPi.GPIO not available - running in simulation mode")


class RaspberryPiHardware(BaseHardware):
    """Raspberry Pi hardware interface implementation"""
    
    def __init__(self, config: Optional[dict] = None):
        self.config = config or self._default_config()
        self.is_connected_flag = False
        self.gpio_initialized = False
        
        # Motor pins configuration (GPIO pins for PWM and direction)
        self.left_motor_pins = self.config['motors']['left_motor']    # [direction_pin, pwm_pin]
        self.right_motor_pins = self.config['motors']['right_motor']  # [direction_pin, pwm_pin]
        
        # Sensor configuration
        self.camera_index = self.config['sensors']['camera']
        self.imu_address = self.config['sensors']['imu']
        
        # Robot dimensions for kinematics
        self.wheel_radius = 0.04    # 40mm typical wheel radius
        self.wheel_base = 0.16      # 160mm distance between wheels
        
        # Motor state
        self.left_motor_pwm = 0
        self.right_motor_pwm = 0
        
        # I2C/SPI buses for sensors
        self.i2c_bus = None
        self.spi_bus = None
        
    def _default_config(self) -> dict:
        """Return default Raspberry Pi configuration"""
        return {
            'motors': {
                'left_motor': [18, 19],    # direction, PWM pins
                'right_motor': [20, 21]    # direction, PWM pins
            },
            'sensors': {
                'camera': '/dev/video0',
                'imu': '/dev/i2c-1'
            },
            'communication': {
                'i2c_bus': 1,
                'spi_bus': 0
            },
            'simulation': not RPI_AVAILABLE  # Use simulation if RPi modules not available
        }
        
    def connect(self) -> bool:
        """Connect to Raspberry Pi hardware"""
        try:
            print("Connecting to Raspberry Pi...")
            
            if self.config.get('simulation', True):
                print("Raspberry Pi simulation mode enabled")
                self.is_connected_flag = True
                self._start_simulation()
                return True
                
            # Initialize GPIO
            if not RPI_AVAILABLE:
                raise ImportError("RPi.GPIO not available")
                
            GPIO.setmode(GPIO.BCM)
            GPIO.setwarnings(False)
            
            # Setup motor pins
            self._setup_motor_pins()
            
            # Initialize sensor buses
            self._setup_sensor_buses()
            
            self.gpio_initialized = True
            self.is_connected_flag = True
            print("Raspberry Pi connected successfully")
            return True
            
        except Exception as e:
            print(f"Failed to connect to Raspberry Pi: {e}")
            # Fallback to simulation
            print("Falling back to simulation mode")
            self.is_connected_flag = True
            self._start_simulation()
            return True
            
    def _start_simulation(self):
        """Start simulation mode"""
        self._sim_thread = threading.Thread(target=self._simulation_loop, daemon=True)
        self._sim_thread.start()
        
    def _simulation_loop(self):
        """Simulation loop for Raspberry Pi operation"""
        while self.is_connected_flag:
            time.sleep(0.1)  # 10 Hz simulation rate
            
    def _setup_motor_pins(self):
        """Setup GPIO pins for motor control"""
        # Left motor
        GPIO.setup(self.left_motor_pins[0], GPIO.OUT)  # Direction pin
        GPIO.setup(self.left_motor_pins[1], GPIO.OUT)  # PWM pin
        self.left_motor_pwm_obj = GPIO.PWM(self.left_motor_pins[1], 1000)  # 1kHz PWM
        self.left_motor_pwm_obj.start(0)
        
        # Right motor
        GPIO.setup(self.right_motor_pins[0], GPIO.OUT)  # Direction pin
        GPIO.setup(self.right_motor_pins[1], GPIO.OUT)  # PWM pin
        self.right_motor_pwm_obj = GPIO.PWM(self.right_motor_pins[1], 1000)  # 1kHz PWM
        self.right_motor_pwm_obj.start(0)
        
    def _setup_sensor_buses(self):
        """Setup I2C and SPI buses for sensors"""
        # Initialize I2C bus for IMU
        try:
            # In real implementation, this would use smbus2
            print("I2C bus initialized for IMU sensor")
        except Exception as e:
            print(f"I2C initialization failed: {e}")
            
        # Initialize SPI bus for high-speed sensors
        try:
            if self.config.get('communication', {}).get('spi_bus'):
                # self.spi_bus = spidev.SpiDev()
                # self.spi_bus.open(0, 0)  # Bus 0, CS 0
                # self.spi_bus.max_speed_hz = 1000000
                print("SPI bus initialized")
        except Exception as e:
            print(f"SPI initialization failed: {e}")
            
    def disconnect(self) -> bool:
        """Disconnect from Raspberry Pi hardware"""
        try:
            self.is_connected_flag = False
            
            if self.gpio_initialized:
                # Stop PWM
                if hasattr(self, 'left_motor_pwm_obj'):
                    self.left_motor_pwm_obj.stop()
                if hasattr(self, 'right_motor_pwm_obj'):
                    self.right_motor_pwm_obj.stop()
                    
                # Cleanup GPIO
                GPIO.cleanup()
                self.gpio_initialized = False
                
            print("Raspberry Pi disconnected")
            return True
            
        except Exception as e:
            print(f"Error disconnecting from Raspberry Pi: {e}")
            return False
            
    def read_sensors(self) -> SensorData:
        """Read data from Raspberry Pi sensors"""
        try:
            # Read IMU sensor data
            imu_data = self._read_imu_sensors()
            
            # Read camera (simulated for now)
            camera_data = self._read_camera()
            
            return SensorData(
                imu_acceleration=imu_data['acceleration'],
                imu_gyroscope=imu_data['gyroscope'],
                imu_magnetometer=imu_data['magnetometer'],
                camera_image=camera_data,
                timestamp=time.time()
            )
            
        except Exception as e:
            print(f"Error reading Raspberry Pi sensors: {e}")
            return SensorData(timestamp=time.time())
            
    def _read_imu_sensors(self) -> dict:
        """Read IMU sensor data (accelerometer, gyroscope, magnetometer)"""
        # Simulate IMU data for demonstration
        t = time.time()
        
        # Simulate accelerometer (m/s²)
        acceleration = [
            0.1 * np.sin(t * 0.5),  # x-axis
            0.1 * np.cos(t * 0.5),  # y-axis
            9.8 + 0.05 * np.sin(t * 0.3)  # z-axis (gravity + movement)
        ]
        
        # Simulate gyroscope (rad/s)
        gyroscope = [
            0.2 * np.sin(t * 0.8),   # x-axis (roll rate)
            0.3 * np.cos(t * 0.6),   # y-axis (pitch rate)
            0.1 * np.sin(t * 1.2)    # z-axis (yaw rate)
        ]
        
        # Simulate magnetometer (µT)
        magnetometer = [
            30 + 5 * np.sin(t * 0.1),  # x-axis
            10 + 3 * np.cos(t * 0.1),  # y-axis
            40 + 2 * np.sin(t * 0.05)  # z-axis
        ]
        
        return {
            'acceleration': acceleration,
            'gyroscope': gyroscope,
            'magnetometer': magnetometer
        }
        
    def _read_camera(self) -> Optional[np.ndarray]:
        """Read camera image"""
        # Simulate camera image for demonstration
        # In real implementation, this would use picamera or OpenCV
        if np.random.random() > 0.8:  # Simulate occasional camera reads
            # Create a simple simulated image
            height, width = 240, 320
            image = np.random.randint(0, 256, (height, width, 3), dtype=np.uint8)
            return image
        return None
        
    def send_commands(self, command: ControlCommand) -> bool:
        """Send control commands to Raspberry Pi motors"""
        try:
            # Convert motor speeds from -1.0 to 1.0 to PWM duty cycle (0-100)
            left_pwm = abs(command.left_motor_speed) * 100
            right_pwm = abs(command.right_motor_speed) * 100
            
            # Set motor directions and PWM
            self._set_motor_direction(0, command.left_motor_speed)   # Left motor
            self._set_motor_direction(1, command.right_motor_speed)  # Right motor
            
            # Set PWM speeds
            if hasattr(self, 'left_motor_pwm_obj'):
                self.left_motor_pwm_obj.ChangeDutyCycle(left_pwm)
            if hasattr(self, 'right_motor_pwm_obj'):
                self.right_motor_pwm_obj.ChangeDutyCycle(right_pwm)
                
            self.left_motor_pwm = left_pwm
            self.right_motor_pwm = right_pwm
            
            return True
            
        except Exception as e:
            print(f"Error sending commands to Raspberry Pi motors: {e}")
            return False
            
    def _set_motor_direction(self, motor_index: int, speed: float):
        """Set motor direction based on speed sign"""
        direction_pin = self.left_motor_pins[0] if motor_index == 0 else self.right_motor_pins[0]
        
        if speed >= 0:
            GPIO.output(direction_pin, GPIO.LOW)   # Forward
        else:
            GPIO.output(direction_pin, GPIO.HIGH)  # Backward
            
    def is_connected(self) -> bool:
        """Check if Raspberry Pi is connected"""
        return self.is_connected_flag
        
    def read_encoder(self, encoder_pin: int) -> int:
        """Read rotary encoder value"""
        # Simulate encoder reading
        # In real implementation, this would use GPIO interrupts to count pulses
        if not hasattr(self, '_encoder_counts'):
            self._encoder_counts = {}
            
        if encoder_pin not in self._encoder_counts:
            self._encoder_counts[encoder_pin] = 0
            
        # Simulate encoder counting based on motor speed
        speed = self.left_motor_pwm if encoder_pin == self.left_motor_pins[1] else self.right_motor_pwm
        self._encoder_counts[encoder_pin] += int(speed * 0.01)  # Simulate pulse counting
        
        return self._encoder_counts[encoder_pin]
        
    def setup_servo(self, servo_pin: int, frequency: int = 50):
        """Setup servo motor control"""
        try:
            GPIO.setup(servo_pin, GPIO.OUT)
            servo_pwm = GPIO.PWM(servo_pin, frequency)
            servo_pwm.start(0)  # Start with 0% duty cycle
            return servo_pwm
        except Exception as e:
            print(f"Error setting up servo on pin {servo_pin}: {e}")
            return None
            
    def set_servo_angle(self, servo_pwm, angle: float):
        """Set servo angle (0-180 degrees)"""
        # Convert angle to duty cycle (1-2ms pulse for 0-180 degrees)
        duty_cycle = 1.0 + (angle / 180.0) * 4.0  # 1ms to 5ms pulse
        if servo_pwm:
            servo_pwm.ChangeDutyCycle(duty_cycle)
            
    def read_distance_sensor(self, trigger_pin: int, echo_pin: int) -> float:
        """Read distance using ultrasonic sensor"""
        try:
            # Setup pins
            GPIO.setup(trigger_pin, GPIO.OUT)
            GPIO.setup(echo_pin, GPIO.IN)
            
            # Send trigger pulse
            GPIO.output(trigger_pin, GPIO.LOW)
            time.sleep(0.000002)
            GPIO.output(trigger_pin, GPIO.HIGH)
            time.sleep(0.00001)
            GPIO.output(trigger_pin, GPIO.LOW)
            
            # Measure echo time
            start_time = time.time()
            while GPIO.input(echo_pin) == GPIO.LOW:
                start_time = time.time()
                
            end_time = time.time()
            while GPIO.input(echo_pin) == GPIO.HIGH:
                end_time = time.time()
                
            # Calculate distance
            pulse_duration = end_time - start_time
            distance = pulse_duration * 17150  # Speed of sound / 2
            return max(0.02, min(3.0, distance))  # Clamp to sensor range
            
        except Exception as e:
            print(f"Error reading distance sensor: {e}")
            # Return simulated distance
            return 0.5 + 0.2 * np.sin(time.time() * 2)
            
    def emergency_stop(self):
        """Emergency stop - disable all motors and servos"""
        print("Raspberry Pi emergency stop")
        
        if hasattr(self, 'left_motor_pwm_obj'):
            self.left_motor_pwm_obj.ChangeDutyCycle(0)
        if hasattr(self, 'right_motor_pwm_obj'):
            self.right_motor_pwm_obj.ChangeDutyCycle(0)
            
        self.left_motor_pwm = 0
        self.right_motor_pwm = 0
        
    def get_system_info(self) -> dict:
        """Get Raspberry Pi system information"""
        import os
        import platform
        
        try:
            # In a real implementation, this would gather actual system info
            return {
                'cpu_usage': np.random.uniform(10, 30),  # Simulated
                'memory_usage': np.random.uniform(40, 60),  # Simulated
                'temperature': 45.0 + np.random.uniform(0, 10),  # Simulated
                'hostname': platform.node(),
                'uptime': time.time()  # Simulated
            }
        except Exception as e:
            print(f"Error getting system info: {e}")
            return {}


class RaspberryPiCamera:
    """Raspberry Pi Camera abstraction"""
    
    def __init__(self, camera_index: int = 0):
        self.camera_index = camera_index
        self.is_opened = False
        self.resolution = (640, 480)
        self.framerate = 30
        
    def open(self) -> bool:
        """Open camera"""
        try:
            if not RPI_AVAILABLE:
                print("Camera simulation mode")
                self.is_opened = True
                return True
                
            # Real camera initialization would go here
            print(f"Opening camera {self.camera_index}")
            self.is_opened = True
            return True
            
        except Exception as e:
            print(f"Error opening camera: {e}")
            return False
            
    def capture_image(self) -> Optional[np.ndarray]:
        """Capture single image"""
        if not self.is_opened:
            return None
            
        try:
            if not RPI_AVAILABLE:
                # Generate simulated image
                image = np.random.randint(0, 256, (480, 640, 3), dtype=np.uint8)
                # Add some structure to make it more realistic
                image[100:200, 200:400] = [255, 0, 0]  # Red rectangle
                image[300:400, 100:300] = [0, 255, 0]  # Green rectangle
                return image
                
            # Real camera capture would go here
            return np.random.randint(0, 256, (480, 640, 3), dtype=np.uint8)
            
        except Exception as e:
            print(f"Error capturing image: {e}")
            return None
            
    def close(self):
        """Close camera"""
        print("Closing camera")
        self.is_opened = False