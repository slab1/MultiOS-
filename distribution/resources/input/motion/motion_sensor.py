"""
Motion Sensor Driver for MultiOS

Provides support for accelerometer, gyroscope, and magnetometer sensors
with educational motion-based interaction features.
"""

from typing import Dict, List, Optional, Tuple, Callable
from dataclasses import dataclass
from enum import Enum
import time
import math
import threading
import logging
from collections import deque

from ..core.input_device import InputDevice, DeviceCapabilities
from ..core.input_event import InputEvent, EventType, EventPriority


class SensorType(Enum):
    """Motion sensor types"""
    ACCELEROMETER = "accelerometer"
    GYROSCOPE = "gyroscope"
    MAGNETOMETER = "magnetometer"
    COMBINED_9DOF = "9dof_sensor"


class CoordinateSystem(Enum):
    """Coordinate system definitions"""
    DEVICE = "device"      # Device coordinates
    WORLD = "world"        # World coordinates
    SCREEN = "screen"      # Screen coordinates


@dataclass
class MotionVector:
    """3D motion vector"""
    x: float
    y: float
    z: float
    
    def magnitude(self) -> float:
        """Calculate magnitude of vector"""
        return math.sqrt(self.x**2 + self.y**2 + self.z**2)
    
    def normalize(self) -> 'MotionVector':
        """Normalize vector to unit length"""
        mag = self.magnitude()
        if mag == 0:
            return MotionVector(0, 0, 0)
        return MotionVector(self.x/mag, self.y/mag, self.z/mag)
    
    def dot_product(self, other: 'MotionVector') -> float:
        """Calculate dot product with another vector"""
        return self.x * other.x + self.y * other.y + self.z * other.z
    
    def cross_product(self, other: 'MotionVector') -> 'MotionVector':
        """Calculate cross product with another vector"""
        return MotionVector(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x
        )
    
    def to_tuple(self) -> Tuple[float, float, float]:
        """Convert to tuple"""
        return (self.x, self.y, self.z)


@dataclass
class Orientation:
    """Device orientation representation"""
    roll: float    # Rotation around X-axis (degrees)
    pitch: float   # Rotation around Y-axis (degrees)
    yaw: float     # Rotation around Z-axis (degrees)
    
    def to_quaternion(self) -> Tuple[float, float, float, float]:
        """Convert to quaternion (x, y, z, w)"""
        cy = math.cos(math.radians(self.yaw * 0.5))
        sy = math.sin(math.radians(self.yaw * 0.5))
        cp = math.cos(math.radians(self.pitch * 0.5))
        sp = math.sin(math.radians(self.pitch * 0.5))
        cr = math.cos(math.radians(self.roll * 0.5))
        sr = math.sin(math.radians(self.roll * 0.5))
        
        w = cr * cp * cy + sr * sp * sy
        x = sr * cp * cy - cr * sp * sy
        y = cr * sp * cy + sr * cp * sy
        z = cr * cp * sy - sr * sp * cy
        
        return (x, y, z, w)
    
    def to_euler(self) -> Tuple[float, float, float]:
        """Convert back to Euler angles"""
        return (self.roll, self.pitch, self.yaw)


class MotionGesture:
    """Motion gesture detection"""
    
    GESTURE_THRESHOLDS = {
        'shake': {
            'magnitude_threshold': 15.0,
            'frequency_threshold': 3.0,  # shakes per second
            'duration_threshold': 0.5
        },
        'tilt': {
            'angle_threshold': 30.0,  # degrees
            'stability_time': 1.0
        },
        'rotation': {
            'angular_velocity': 90.0,  # degrees per second
            'duration_threshold': 0.2
        },
        'flip': {
            'z_acceleration': -15.0,
            'velocity_threshold': 5.0
        }
    }
    
    def __init__(self):
        self.gesture_history: deque = deque(maxlen=100)
        self.active_gestures: Dict[str, Dict] = {}
    
    def update_motion_data(self, accel: MotionVector, gyro: MotionVector, 
                          orientation: Orientation) -> List[Dict]:
        """Update motion data and detect gestures"""
        gestures = []
        
        # Check for shake gesture
        if self._detect_shake(accel, gyro):
            gestures.append({
                'type': 'shake',
                'confidence': 0.9,
                'timestamp': time.time()
            })
        
        # Check for tilt gesture
        tilt_result = self._detect_tilt(orientation)
        if tilt_result:
            gestures.append({
                'type': 'tilt',
                'direction': tilt_result['direction'],
                'angle': tilt_result['angle'],
                'confidence': 0.8,
                'timestamp': time.time()
            })
        
        # Check for rotation gesture
        rotation_result = self._detect_rotation(gyro)
        if rotation_result:
            gestures.append({
                'type': 'rotation',
                'direction': rotation_result['direction'],
                'angular_velocity': rotation_result['angular_velocity'],
                'confidence': 0.7,
                'timestamp': time.time()
            })
        
        # Check for flip gesture
        if self._detect_flip(accel):
            gestures.append({
                'type': 'flip',
                'direction': 'face_down' if accel.z < -10 else 'face_up',
                'confidence': 0.6,
                'timestamp': time.time()
            })
        
        # Store gestures in history
        self.gesture_history.extend(gestures)
        
        return gestures
    
    def _detect_shake(self, accel: MotionVector, gyro: MotionVector) -> bool:
        """Detect shake gesture"""
        thresholds = self.GESTURE_THRESHOLDS['shake']
        
        accel_mag = accel.magnitude()
        gyro_mag = gyro.magnitude()
        
        if accel_mag > thresholds['magnitude_threshold']:
            # Count shake events
            shake_count = len([g for g in self.gesture_history 
                             if g['type'] == 'shake' and 
                             time.time() - g['timestamp'] < thresholds['duration_threshold']])
            
            return shake_count >= 2
        
        return False
    
    def _detect_tilt(self, orientation: Orientation) -> Optional[Dict]:
        """Detect tilt gesture"""
        thresholds = self.GESTURE_THRESHOLDS['tilt']
        
        # Determine tilt direction and angle
        if abs(orientation.roll) > thresholds['angle_threshold']:
            direction = 'right' if orientation.roll > 0 else 'left'
            angle = abs(orientation.roll)
            return {'direction': direction, 'angle': angle}
        
        elif abs(orientation.pitch) > thresholds['angle_threshold']:
            direction = 'back' if orientation.pitch > 0 else 'forward'
            angle = abs(orientation.pitch)
            return {'direction': direction, 'angle': angle}
        
        return None
    
    def _detect_rotation(self, gyro: MotionVector) -> Optional[Dict]:
        """Detect rotation gesture"""
        thresholds = self.GESTURE_THRESHOLDS['rotation']
        
        angular_velocity = gyro.magnitude()
        
        if angular_velocity > thresholds['angular_velocity']:
            # Determine rotation direction
            max_axis = 'x' if abs(gyro.x) == max(abs(gyro.x), abs(gyro.y), abs(gyro.z)) else \
                      'y' if abs(gyro.y) == max(abs(gyro.y), abs(gyro.z)) else 'z'
            
            direction = f"rotate_{max_axis}"
            
            return {
                'direction': direction,
                'angular_velocity': angular_velocity
            }
        
        return None
    
    def _detect_flip(self, accel: MotionVector) -> bool:
        """Detect flip gesture"""
        thresholds = self.GESTURE_THRESHOLDS['flip']
        
        # Detect significant Z-axis acceleration change (like device flip)
        if accel.z < thresholds['z_acceleration']:
            # Check for flip in recent history
            recent_flips = [g for g in self.gesture_history 
                          if g['type'] == 'flip' and 
                          time.time() - g['timestamp'] < 2.0]
            
            return len(recent_flips) == 0  # Only trigger if no recent flip
        
        return False


class MotionSensorDevice(InputDevice):
    """Motion sensor device driver"""
    
    def __init__(self, device_id: str, sensor_type: SensorType = SensorType.COMBINED_9DOF):
        super().__init__(device_id, 'motion')
        
        self.sensor_type = sensor_type
        
        # Sensor properties
        self.sample_rate = 100  # 100Hz
        self.range_accelerometer = 16.0  # g
        self.range_gyroscope = 2000.0    # degrees per second
        self.resolution = 0.001
        
        # Current sensor readings
        self.current_acceleration = MotionVector(0, 0, 9.81)  # Gravity
        self.current_gyroscope = MotionVector(0, 0, 0)
        self.current_magnetometer = MotionVector(0, 0, 0)
        self.current_orientation = Orientation(0, 0, 0)
        
        # Sensor filtering and processing
        self.motion_gesture = MotionGesture()
        self.noise_filter_enabled = True
        self.calibration_matrix = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
        self.bias_correction = MotionVector(0, 0, 0)
        
        # Motion history for gesture detection
        self.acceleration_history: deque = deque(maxlen=50)
        self.gyroscope_history: deque = deque(maxlen=50)
        self.orientation_history: deque = deque(maxlen=50)
        
        # Educational motion interactions
        self.educational_gestures = {
            'shake': 'Clear screen or undo action',
            'tilt_left': 'Navigate left',
            'tilt_right': 'Navigate right',
            'tilt_forward': 'Scroll down or zoom out',
            'tilt_back': 'Scroll up or zoom in',
            'rotate_cw': 'Rotate clockwise',
            'rotate_ccw': 'Rotate counter-clockwise',
            'flip': 'Flip orientation or reset view'
        }
        
        # Configuration
        self.config.update({
            'gesture_detection': True,
            'noise_filtering': True,
            'auto_calibration': False,
            'educational_mode': True,
            'coordinate_system': CoordinateSystem.DEVICE.value,
            'sensitivity_multiplier': 1.0
        })
        
        self.logger = logging.getLogger(f"motion.{device_id}")
    
    def get_capabilities(self) -> DeviceCapabilities:
        """Get device capabilities"""
        capabilities = DeviceCapabilities()
        capabilities.supported_events = {
            EventType.MOTION_ACCEL,
            EventType.MOTION_GYRO
        }
        capabilities.sampling_rate = self.sample_rate
        capabilities.accuracy = 0.95
        
        if self.sensor_type in [SensorType.ACCELEROMETER, SensorType.COMBINED_9DOF]:
            capabilities.accelerometer_range = self.range_accelerometer
        
        if self.sensor_type in [SensorType.GYROSCOPE, SensorType.COMBINED_9DOF]:
            capabilities.gyroscope_range = self.range_gyroscope
        
        return capabilities
    
    def connect(self) -> bool:
        """Connect to motion sensor"""
        try:
            self._init_sensor_hardware()
            
            # Initial sensor reading
            self._read_initial_values()
            
            self.is_connected = True
            self.logger.info(f"Connected {self.sensor_type.value} sensor")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to connect motion sensor: {e}")
            return False
    
    def disconnect(self) -> bool:
        """Disconnect from motion sensor"""
        try:
            self._cleanup_sensor_hardware()
            self.is_connected = False
            self.logger.info("Disconnected motion sensor")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to disconnect: {e}")
            return False
    
    def _init_sensor_hardware(self):
        """Initialize sensor hardware"""
        # In real implementation, this would initialize I2C/SPI interface
        # to communicate with motion sensor (MPU-6050, MPU-9250, etc.)
        pass
    
    def _cleanup_sensor_hardware(self):
        """Cleanup sensor hardware"""
        # In real implementation, this would cleanup hardware resources
        pass
    
    def _read_initial_values(self):
        """Read initial sensor values for calibration"""
        # In real implementation, this would read multiple samples
        # and calculate bias corrections
        pass
    
    def start_polling(self):
        """Start motion sensor polling"""
        pass
    
    def stop_polling(self):
        """Stop motion sensor polling"""
        pass
    
    def calibrate(self) -> bool:
        """Calibrate motion sensor"""
        try:
            self.logger.info("Starting motion sensor calibration")
            
            # Calibrate accelerometer
            self._calibrate_accelerometer()
            
            # Calibrate gyroscope
            self._calibrate_gyroscope()
            
            # Calibrate magnetometer if available
            if self.sensor_type in [SensorType.COMBINED_9DOF]:
                self._calibrate_magnetometer()
            
            # Calculate calibration matrix
            self._calculate_calibration_matrix()
            
            self.is_calibrated = True
            self.logger.info("Motion sensor calibration completed")
            return True
        
        except Exception as e:
            self.logger.error(f"Motion calibration failed: {e}")
            return False
    
    def _calibrate_accelerometer(self):
        """Calibrate accelerometer"""
        self.logger.info("Calibrating accelerometer...")
        # Simulate calibration process
        time.sleep(2)
    
    def _calibrate_gyroscope(self):
        """Calibrate gyroscope"""
        self.logger.info("Calibrating gyroscope...")
        # Simulate calibration process
        time.sleep(2)
    
    def _calibrate_magnetometer(self):
        """Calibrate magnetometer"""
        self.logger.info("Calibrating magnetometer...")
        # Simulate calibration process
        time.sleep(1)
    
    def _calculate_calibration_matrix(self):
        """Calculate calibration matrix"""
        # Simple identity matrix for simulation
        self.calibration_matrix = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0]
        ]
    
    def _poll_device(self):
        """Poll motion sensor for new data"""
        try:
            if not self.is_enabled or not self.is_connected:
                return
            
            # Read raw sensor data
            raw_accel = self._read_raw_acceleration()
            raw_gyro = self._read_raw_gyroscope()
            
            if raw_accel:
                self.current_acceleration = raw_accel
                
                # Generate acceleration event
                event = InputEvent(
                    event_type=EventType.MOTION_ACCEL,
                    timestamp=time.time(),
                    priority=EventPriority.NORMAL,
                    device_id=self.device_id,
                    device_type='motion',
                    x=raw_accel.x,
                    y=raw_accel.y,
                    z=raw_accel.z,
                    acceleration={
                        'x': raw_accel.x,
                        'y': raw_accel.y,
                        'z': raw_accel.z,
                        'magnitude': raw_accel.magnitude()
                    }
                )
                self._send_event(event)
            
            if raw_gyro:
                self.current_gyroscope = raw_gyro
                
                # Calculate orientation from gyroscope integration
                self._update_orientation(raw_gyro)
                
                # Generate gyroscope event
                event = InputEvent(
                    event_type=EventType.MOTION_GYRO,
                    timestamp=time.time(),
                    priority=EventPriority.NORMAL,
                    device_id=self.device_id,
                    device_type='motion',
                    x=raw_gyro.x,
                    y=raw_gyro.y,
                    z=raw_gyro.z,
                    rotation={
                        'roll': self.current_orientation.roll,
                        'pitch': self.current_orientation.pitch,
                        'yaw': self.current_orientation.yaw,
                        'angular_velocity_x': raw_gyro.x,
                        'angular_velocity_y': raw_gyro.y,
                        'angular_velocity_z': raw_gyro.z
                    }
                )
                self._send_event(event)
            
            # Detect motion gestures if enabled
            if self.config.get('gesture_detection', True):
                self._detect_motion_gestures()
            
        except Exception as e:
            self.logger.error(f"Motion polling error: {e}")
    
    def _read_raw_acceleration(self) -> Optional[MotionVector]:
        """Read raw acceleration data from sensor"""
        # Simulate sensor reading with noise
        import random
        
        if random.random() < 0.7:  # 70% chance of valid data
            # Simulate realistic acceleration with gravity and device motion
            base_gravity = MotionVector(0, 0, 9.81)
            noise = MotionVector(
                random.gauss(0, 0.1),
                random.gauss(0, 0.1),
                random.gauss(0, 0.1)
            )
            
            # Apply device orientation to gravity
            orientation_rad = math.radians(self.current_orientation.roll)
            gravity_x = 9.81 * math.sin(orientation_rad)
            gravity_y = 0
            gravity_z = 9.81 * math.cos(orientation_rad)
            
            accel = MotionVector(
                gravity_x + noise.x,
                gravity_y + noise.y,
                gravity_z + noise.z
            )
            
            # Apply calibration
            if self.is_calibrated:
                accel = self._apply_calibration(accel)
            
            return accel
        
        return None
    
    def _read_raw_gyroscope(self) -> Optional[MotionVector]:
        """Read raw gyroscope data from sensor"""
        import random
        
        if random.random() < 0.7:  # 70% chance of valid data
            # Simulate realistic angular velocities
            noise = MotionVector(
                random.gauss(0, 2.0),  # ±2 deg/s noise
                random.gauss(0, 2.0),
                random.gauss(0, 2.0)
            )
            
            gyro = MotionVector(
                random.gauss(0, 10) + noise.x,
                random.gauss(0, 10) + noise.y,
                random.gauss(0, 10) + noise.z
            )
            
            return gyro
        
        return None
    
    def _apply_calibration(self, accel: MotionVector) -> MotionVector:
        """Apply calibration matrix and bias correction"""
        # Apply calibration matrix
        calibrated_x = (self.calibration_matrix[0][0] * accel.x + 
                       self.calibration_matrix[0][1] * accel.y + 
                       self.calibration_matrix[0][2] * accel.z)
        
        calibrated_y = (self.calibration_matrix[1][0] * accel.x + 
                       self.calibration_matrix[1][1] * accel.y + 
                       self.calibration_matrix[1][2] * accel.z)
        
        calibrated_z = (self.calibration_matrix[2][0] * accel.x + 
                       self.calibration_matrix[2][1] * accel.y + 
                       self.calibration_matrix[2][2] * accel.z)
        
        # Apply bias correction
        calibrated_x -= self.bias_correction.x
        calibrated_y -= self.bias_correction.y
        calibrated_z -= self.bias_correction.z
        
        return MotionVector(calibrated_x, calibrated_y, calibrated_z)
    
    def _update_orientation(self, angular_velocity: MotionVector):
        """Update device orientation from gyroscope data"""
        # Integrate angular velocity to get orientation changes
        dt = 0.01  # Assuming 100Hz sampling
        
        # Update Euler angles
        self.current_orientation.roll += angular_velocity.x * dt
        self.current_orientation.pitch += angular_velocity.y * dt
        self.current_orientation.yaw += angular_velocity.z * dt
        
        # Keep angles in reasonable ranges
        self.current_orientation.roll = max(-180, min(180, self.current_orientation.roll))
        self.current_orientation.pitch = max(-90, min(90, self.current_orientation.pitch))
        self.current_orientation.yaw = self.current_orientation.yaw % 360
        
        # Store in history
        self.orientation_history.append(self.current_orientation)
    
    def _detect_motion_gestures(self):
        """Detect motion-based gestures"""
        if not self.current_acceleration or not self.current_gyroscope:
            return
        
        gestures = self.motion_gesture.update_motion_data(
            self.current_acceleration,
            self.current_gyroscope,
            self.current_orientation
        )
        
        for gesture in gestures:
            self.logger.info(f"Detected gesture: {gesture}")
            
            # Send gesture event
            event = InputEvent(
                event_type=EventType.GESTURE_SWIPE,  # Using SWIPE for gestures
                timestamp=gesture['timestamp'],
                priority=EventPriority.HIGH,
                device_id=self.device_id,
                device_type='motion',
                gesture_data=gesture
            )
            self._send_event(event)
    
    def get_motion_status(self) -> Dict[str, any]:
        """Get current motion sensor status"""
        return {
            'sensor_type': self.sensor_type.value,
            'sample_rate': self.sample_rate,
            'is_calibrated': self.is_calibrated,
            'current_acceleration': self.current_acceleration.to_tuple(),
            'current_gyroscope': self.current_gyroscope.to_tuple(),
            'current_orientation': {
                'roll': self.current_orientation.roll,
                'pitch': self.current_orientation.pitch,
                'yaw': self.current_orientation.yaw
            },
            'acceleration_magnitude': self.current_acceleration.magnitude(),
            'gesture_detection_enabled': self.config.get('gesture_detection', True),
            'educational_mode': self.config.get('educational_mode', True)
        }
    
    def get_orientation(self) -> Orientation:
        """Get current device orientation"""
        return self.current_orientation
    
    def set_orientation(self, orientation: Orientation):
        """Set device orientation (for simulation/testing)"""
        self.current_orientation = orientation
    
    def get_acceleration(self) -> MotionVector:
        """Get current acceleration"""
        return self.current_acceleration
    
    def get_gyroscope(self) -> MotionVector:
        """Get current gyroscope reading"""
        return self.current_gyroscope
    
    def simulate_motion(self, accel: Tuple[float, float, float],
                       gyro: Tuple[float, float, float] = (0, 0, 0)) -> bool:
        """Simulate motion data for testing"""
        if not self.is_enabled or not self.is_connected:
            return False
        
        # Set current values
        self.current_acceleration = MotionVector(*accel)
        self.current_gyroscope = MotionVector(*gyro)
        
        # Update orientation
        if self.is_calibrated:
            self._update_orientation(self.current_gyroscope)
        
        # Send events
        accel_event = InputEvent(
            event_type=EventType.MOTION_ACCEL,
            timestamp=time.time(),
            priority=EventPriority.NORMAL,
            device_id=self.device_id,
            device_type='motion',
            x=accel[0],
            y=accel[1],
            z=accel[2],
            acceleration={
                'x': accel[0], 'y': accel[1], 'z': accel[2],
                'magnitude': math.sqrt(sum(x**2 for x in accel))
            }
        )
        self._send_event(accel_event)
        
        if any(gyro):
            gyro_event = InputEvent(
                event_type=EventType.MOTION_GYRO,
                timestamp=time.time(),
                priority=EventPriority.NORMAL,
                device_id=self.device_id,
                device_type='motion',
                x=gyro[0],
                y=gyro[1],
                z=gyro[2],
                rotation={
                    'roll': self.current_orientation.roll,
                    'pitch': self.current_orientation.pitch,
                    'yaw': self.current_orientation.yaw,
                    'angular_velocity_x': gyro[0],
                    'angular_velocity_y': gyro[1],
                    'angular_velocity_z': gyro[2]
                }
            )
            self._send_event(gyro_event)
        
        return True
    
    def simulate_gesture(self, gesture_type: str) -> bool:
        """Simulate motion gesture for testing"""
        if not self.is_enabled or not self.is_connected:
            return False
        
        gesture_data = {
            'type': gesture_type,
            'confidence': 0.9,
            'timestamp': time.time()
        }
        
        event = InputEvent(
            event_type=EventType.GESTURE_SWIPE,
            timestamp=time.time(),
            priority=EventPriority.HIGH,
            device_id=self.device_id,
            device_type='motion',
            gesture_data=gesture_data
        )
        self._send_event(event)
        
        return True
    
    def get_educational_gestures(self) -> Dict[str, str]:
        """Get available educational motion gestures"""
        return self.educational_gestures.copy()
    
    def enable_educational_mode(self, enabled: bool = True):
        """Enable or disable educational interaction mode"""
        self.config['educational_mode'] = enabled