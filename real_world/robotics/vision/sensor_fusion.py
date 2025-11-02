"""
Sensor Fusion Module

Provides sensor fusion algorithms for combining data from multiple sensors
"""

import numpy as np
import time
import math
from typing import List, Tuple, Optional, Dict, Any
from dataclasses import dataclass
from collections import deque
from dataclasses import dataclass


@dataclass
class SensorReading:
    """Container for sensor reading with timestamp and metadata"""
    value: float
    timestamp: float
    sensor_id: str
    confidence: float = 1.0
    unit: str = ""
    metadata: Optional[Dict] = None


@dataclass
class FusedState:
    """Container for fused sensor state"""
    position: Tuple[float, float, float]  # x, y, z
    velocity: Tuple[float, float, float]  # vx, vy, vz
    orientation: Tuple[float, float, float]  # roll, pitch, yaw
    angular_velocity: Tuple[float, float, float]  # omega_x, omega_y, omega_z
    confidence: float = 1.0
    timestamp: float = 0.0
    
    def __post_init__(self):
        if self.timestamp == 0.0:
            self.timestamp = time.time()


class LowPassFilter:
    """Simple low-pass filter for sensor data smoothing"""
    
    def __init__(self, alpha: float = 0.8):
        self.alpha = alpha
        self.last_value = None
        self.is_initialized = False
        
    def update(self, new_value: float) -> float:
        """Update filter with new value"""
        if not self.is_initialized:
            self.last_value = new_value
            self.is_initialized = True
            return new_value
            
        filtered_value = self.alpha * new_value + (1 - self.alpha) * self.last_value
        self.last_value = filtered_value
        return filtered_value
        
    def reset(self):
        """Reset filter state"""
        self.last_value = None
        self.is_initialized = False


class ComplementaryFilter:
    """Complementary filter for sensor fusion"""
    
    def __init__(self, alpha: float = 0.98):
        self.alpha = alpha  # Weight for gyroscope data
        self.is_initialized = False
        
        # State variables
        self.pitch = 0.0
        self.roll = 0.0
        self.yaw = 0.0
        
        # For tracking changes
        self.last_accel = None
        self.last_gyro = None
        
    def update(self, accel_data: List[float], gyro_data: List[float], 
               dt: float) -> Tuple[float, float, float]:
        """
        Update filter with accelerometer and gyroscope data
        
        Args:
            accel_data: [ax, ay, az] acceleration data
            gyro_data: [gx, gy, gz] gyroscope data
            
        Returns:
            (roll, pitch, yaw) in radians
        """
        if len(accel_data) != 3 or len(gyro_data) != 3:
            return self.roll, self.pitch, self.yaw
            
        ax, ay, az = accel_data
        gx, gy, gz = gyro_data
        
        if not self.is_initialized:
            # Initialize orientation from accelerometer
            self.roll = math.atan2(ay, math.sqrt(ax**2 + az**2))
            self.pitch = math.atan2(-ax, math.sqrt(ay**2 + az**2))
            self.yaw = 0.0  # Initialize yaw to 0
            
            self.last_accel = accel_data.copy()
            self.last_gyro = gyro_data.copy()
            self.is_initialized = True
            
        else:
            # Integrate gyroscope data for short-term changes
            droll = gx * dt
            dpitch = gy * dt
            dyaw = gz * dt
            
            # Update using gyroscope (high-frequency component)
            gyro_roll = self.roll + droll
            gyro_pitch = self.pitch + dpitch
            gyro_yaw = self.yaw + dyaw
            
            # Calculate orientation from accelerometer (low-frequency component)
            if abs(az) > 0.1:  # Avoid division by zero
                accel_roll = math.atan2(ay, math.sqrt(ax**2 + az**2))
                accel_pitch = math.atan2(-ax, math.sqrt(ay**2 + az**2))
            else:
                accel_roll = self.roll
                accel_pitch = self.pitch
                
            # For yaw, use magnetometer if available (not implemented here)
            accel_yaw = self.yaw
            
            # Complementary filter - blend high and low frequency components
            self.roll = self.alpha * gyro_roll + (1 - self.alpha) * accel_roll
            self.pitch = self.alpha * gyro_pitch + (1 - self.alpha) * accel_pitch
            self.yaw = self.alpha * gyro_yaw + (1 - self.alpha) * accel_yaw
            
            # Keep angles in reasonable range
            self.roll = np.clip(self.roll, -np.pi, np.pi)
            self.pitch = np.clip(self.pitch, -np.pi/2, np.pi/2)
            
        return self.roll, self.pitch, self.yaw


class KalmanFilter:
    """Extended Kalman Filter for state estimation"""
    
    def __init__(self, state_dim: int, measurement_dim: int):
        self.state_dim = state_dim
        self.measurement_dim = measurement_dim
        
        # State vector [x, vx, y, vy, theta, omega]
        self.x = np.zeros(state_dim)
        self.P = np.eye(state_dim)  # Covariance matrix
        
        # Process noise covariance
        self.Q = np.eye(state_dim) * 0.01
        
        # Measurement noise covariance
        self.R = np.eye(measurement_dim) * 0.1
        
        self.is_initialized = False
        
    def initialize_state(self, initial_state: np.ndarray):
        """Initialize state vector"""
        if len(initial_state) != self.state_dim:
            raise ValueError(f"Initial state dimension mismatch: expected {self.state_dim}, got {len(initial_state)}")
            
        self.x = initial_state.copy()
        self.is_initialized = True
        
    def predict(self, dt: float, control_input: Optional[np.ndarray] = None):
        """Prediction step of Kalman filter"""
        if not self.is_initialized:
            return
            
        # State transition matrix (simplified for 2D motion with constant velocity)
        F = np.array([
            [1, dt, 0,  0,  0, 0],  # x = x + vx*dt
            [0,  1, 0,  0,  0, 0],  # vx = vx
            [0,  0, 1, dt,  0, 0],  # y = y + vy*dt
            [0,  0, 0,  1,  0, 0],  # vy = vy
            [0,  0, 0,  0,  1, dt], # theta = theta + omega*dt
            [0,  0, 0,  0,  0, 1]   # omega = omega
        ])
        
        # Control input matrix (if provided)
        if control_input is not None:
            B = np.array([
                [0, 0],  # x control
                [1, 0],  # vx control (motor commands)
                [0, 0],  # y control
                [0, 1],  # vy control
                [0, 0],  # theta control
                [0, 0]   # omega control
            ])
            
            # Predict state
            self.x = F @ self.x + B @ control_input
        else:
            # Predict state without control input
            self.x = F @ self.x
            
        # Predict covariance
        self.P = F @ self.P @ F.T + self.Q
        
    def update(self, measurement: np.ndarray, measurement_matrix: Optional[np.ndarray] = None):
        """Update step of Kalman filter"""
        if not self.is_initialized:
            return
            
        if measurement_matrix is None:
            # Default measurement matrix (observing position and orientation)
            H = np.array([
                [1, 0, 0, 0, 0, 0],  # Observe x
                [0, 0, 1, 0, 0, 0],  # Observe y
                [0, 0, 0, 0, 1, 0]   # Observe theta
            ])
        else:
            H = measurement_matrix
            
        # Innovation
        y = measurement - H @ self.x
        
        # Innovation covariance
        S = H @ self.P @ H.T + self.R
        
        # Kalman gain
        K = self.P @ H.T @ np.linalg.inv(S)
        
        # Update state
        self.x = self.x + K @ y
        
        # Update covariance
        I = np.eye(self.state_dim)
        self.P = (I - K @ H) @ self.P
        
    def get_state(self) -> np.ndarray:
        """Get current state estimate"""
        return self.x.copy()
        
    def get_state_certainty(self) -> float:
        """Get state estimation certainty (trace of covariance)"""
        return np.trace(self.P)


class SensorFusionEngine:
    """Main sensor fusion engine that combines multiple sensors"""
    
    def __init__(self):
        # Initialize filters
        self.complementary_filter = ComplementaryFilter()
        self.kalman_filter = KalmanFilter(state_dim=6, measurement_dim=3)
        
        # Initialize sensors
        self.sensors = {}
        self.sensor_filters = {}
        self.sensor_confidence = {}
        
        # State tracking
        self.current_state = FusedState(
            position=(0.0, 0.0, 0.0),
            velocity=(0.0, 0.0, 0.0),
            orientation=(0.0, 0.0, 0.0),
            angular_velocity=(0.0, 0.0, 0.0)
        )
        
        # History for analysis
        self.state_history = deque(maxlen=1000)
        
        # Configuration
        self.use_kalman_filter = True
        self.use_complementary_filter = True
        self.position_noise_threshold = 0.1  # meters
        self.velocity_noise_threshold = 0.05  # m/s
        
    def register_sensor(self, sensor_id: str, sensor_type: str, 
                       initial_confidence: float = 1.0):
        """Register a new sensor"""
        self.sensors[sensor_id] = sensor_type
        self.sensor_confidence[sensor_id] = initial_confidence
        
        # Initialize appropriate filter for sensor type
        if sensor_type in ['ultrasonic', 'distance']:
            self.sensor_filters[sensor_id] = LowPassFilter(alpha=0.9)
        elif sensor_type in ['imu', 'accelerometer', 'gyroscope']:
            self.sensor_filters[sensor_id] = LowPassFilter(alpha=0.95)
        elif sensor_type in ['camera', 'vision']:
            self.sensor_filters[sensor_id] = LowPassFilter(alpha=0.8)
        else:
            self.sensor_filters[sensor_id] = LowPassFilter(alpha=0.85)
            
        print(f"Registered sensor: {sensor_id} ({sensor_type})")
        
    def update_sensor_reading(self, sensor_id: str, reading: SensorReading):
        """Update sensor reading and apply filtering"""
        if sensor_id not in self.sensors:
            self.register_sensor(sensor_id, "unknown")
            
        # Apply sensor-specific filtering
        if sensor_id in self.sensor_filters:
            filtered_value = self.sensor_filters[sensor_id].update(reading.value)
        else:
            filtered_value = reading.value
            
        # Store filtered reading
        self.sensor_filters[sensor_id + "_filtered"] = filtered_value
        
        # Update sensor confidence based on reading quality
        self._update_sensor_confidence(sensor_id, reading)
        
    def _update_sensor_confidence(self, sensor_id: str, reading: SensorReading):
        """Update sensor confidence based on reading quality"""
        # Adjust confidence based on timestamp (stale data gets lower confidence)
        age = time.time() - reading.timestamp
        time_confidence = max(0.1, 1.0 - age / 10.0)  # Decay over 10 seconds
        
        # Combine with reading confidence
        reading_confidence = reading.confidence
        
        # Update overall confidence
        alpha = 0.1  # Smoothing factor
        current_conf = self.sensor_confidence.get(sensor_id, 1.0)
        new_conf = current_conf * (1 - alpha) + alpha * (time_confidence * reading_confidence)
        
        self.sensor_confidence[sensor_id] = new_conf
        
    def fuse_sensors(self, dt: float) -> FusedState:
        """Fuse all sensor data to produce unified state estimate"""
        # Collect available sensor data
        sensor_data = self._collect_sensor_data()
        
        # Update complementary filter with IMU data
        if self.use_complementary_filter and 'imu' in sensor_data:
            imu_data = sensor_data['imu']
            roll, pitch, yaw = self.complementary_filter.update(
                imu_data.get('acceleration', [0, 0, 9.8]),
                imu_data.get('gyroscope', [0, 0, 0]),
                dt
            )
            
            # Update current state
            self.current_state.orientation = (roll, pitch, yaw)
            self.current_state.angular_velocity = imu_data.get('gyroscope', [0, 0, 0])
            
        # Update Kalman filter with position measurements
        if self.use_kalman_filter and self.kalman_filter.is_initialized:
            self.kalman_filter.predict(dt)
            
            # Create measurement vector from available sensors
            measurement = self._create_measurement_vector(sensor_data)
            if measurement is not None:
                self.kalman_filter.update(measurement)
                
                # Extract state from Kalman filter
                kf_state = self.kalman_filter.get_state()
                self.current_state.position = (kf_state[0], kf_state[2], 0.0)
                self.current_state.velocity = (kf_state[1], kf_state[3], 0.0)
                if len(kf_state) >= 6:
                    self.current_state.angular_velocity = (0.0, 0.0, kf_state[5])
                    
        # Perform sensor weighting and consensus
        self._perform_sensor_consensus(sensor_data)
        
        # Update timestamp
        self.current_state.timestamp = time.time()
        
        # Calculate overall confidence
        self.current_state.confidence = self._calculate_overall_confidence()
        
        # Store in history
        self.state_history.append(self.current_state)
        
        return self.current_state
        
    def _collect_sensor_data(self) -> Dict[str, Dict[str, float]]:
        """Collect and organize sensor data"""
        sensor_data = {}
        
        # Process each registered sensor
        for sensor_id, sensor_type in self.sensors.items():
            filtered_key = sensor_id + "_filtered"
            if filtered_key in self.sensor_filters:
                value = self.sensor_filters[filtered_key]
                confidence = self.sensor_confidence.get(sensor_id, 0.5)
                
                if sensor_type not in sensor_data:
                    sensor_data[sensor_type] = {}
                    
                # Organize data by sensor type
                if sensor_type in ['imu', 'accelerometer', 'gyroscope']:
                    if sensor_type == 'imu':
                        sensor_data[sensor_type][sensor_id] = value
                    elif sensor_type == 'accelerometer':
                        sensor_data[sensor_type][sensor_id] = value
                    elif sensor_type == 'gyroscope':
                        sensor_data[sensor_type][sensor_id] = value
                else:
                    sensor_data[sensor_type][sensor_id] = {
                        'value': value,
                        'confidence': confidence
                    }
                    
        return sensor_data
        
    def _create_measurement_vector(self, sensor_data: Dict[str, Dict]) -> Optional[np.ndarray]:
        """Create measurement vector for Kalman filter"""
        measurements = []
        
        # Position measurements from various sensors
        if 'ultrasonic' in sensor_data:
            # Use ultrasonic for distance measurements
            # This is simplified - real implementation would transform to global coordinates
            if 'ultrasonic_front' in sensor_data['ultrasonic']:
                front_distance = sensor_data['ultrasonic']['ultrasonic_front']['value']
                measurements.extend([front_distance, 0.0])  # x, y (simplified)
                
        if 'camera' in sensor_data:
            # Use camera for position estimates
            if 'camera_position' in sensor_data['camera']:
                pos_data = sensor_data['camera']['camera_position']['value']
                if len(pos_data) >= 2:
                    measurements.extend([pos_data[0], pos_data[1]])
                    
        if len(measurements) >= 3:
            return np.array(measurements[:3])  # x, y, theta
            
        return None
        
    def _perform_sensor_consensus(self, sensor_data: Dict[str, Dict]):
        """Perform sensor consensus and outlier detection"""
        # This is a simplified consensus algorithm
        # In practice, you would implement more sophisticated consensus algorithms
        
        position_estimates = []
        
        # Collect position estimates from multiple sensors
        for sensor_type, sensors in sensor_data.items():
            if sensor_type in ['ultrasonic', 'camera', 'gps']:  # Position sensors
                for sensor_id, data in sensors.items():
                    if isinstance(data, dict) and 'value' in data:
                        if len(data['value']) >= 2:
                            position_estimates.append((data['value'][:2], data.get('confidence', 1.0)))
                            
        # Perform weighted average if multiple estimates available
        if len(position_estimates) > 1:
            weights = [conf for _, conf in position_estimates]
            positions = [pos for pos, _ in position_estimates]
            
            if len(positions[0]) >= 2:
                weighted_x = sum(pos[0] * w for pos, w in zip(positions, weights))
                weighted_y = sum(pos[1] * w for pos, w in zip(positions, weights))
                total_weight = sum(weights)
                
                if total_weight > 0:
                    consensus_x = weighted_x / total_weight
                    consensus_y = weighted_y / total_weight
                    
                    # Update position estimate
                    current_pos = self.current_state.position
                    self.current_state.position = (consensus_x, consensus_y, current_pos[2])
                    
    def _calculate_overall_confidence(self) -> float:
        """Calculate overall state estimation confidence"""
        confidences = list(self.sensor_confidence.values())
        if not confidences:
            return 0.5
            
        # Weighted average of sensor confidences
        return sum(confidences) / len(confidences)
        
    def initialize_kalman_filter(self, initial_position: Tuple[float, float] = (0.0, 0.0),
                                initial_orientation: float = 0.0,
                                initial_velocity: Tuple[float, float] = (0.0, 0.0)):
        """Initialize Kalman filter with known initial state"""
        initial_state = np.array([
            initial_position[0],  # x
            initial_velocity[0],  # vx
            initial_position[1],  # y
            initial_velocity[1],  # vy
            initial_orientation,  # theta
            0.0                   # omega
        ])
        
        self.kalman_filter.initialize_state(initial_state)
        print(f"Kalman filter initialized with state: {initial_state}")
        
    def get_sensor_status(self) -> Dict[str, Any]:
        """Get status of all sensors"""
        status = {}
        
        for sensor_id, sensor_type in self.sensors.items():
            confidence = self.sensor_confidence.get(sensor_id, 0.0)
            has_filter = sensor_id in self.sensor_filters
            
            status[sensor_id] = {
                'type': sensor_type,
                'confidence': confidence,
                'has_filter': has_filter,
                'status': 'active' if confidence > 0.5 else 'inactive'
            }
            
        return status
        
    def reset_filters(self):
        """Reset all filters to initial state"""
        # Reset complementary filter
        self.complementary_filter = ComplementaryFilter(alpha=self.complementary_filter.alpha)
        
        # Reset Kalman filter
        self.kalman_filter = KalmanFilter(self.kalman_filter.state_dim, 
                                         self.kalman_filter.measurement_dim)
        self.kalman_filter.is_initialized = False
        
        # Reset sensor filters
        for sensor_id in self.sensor_filters:
            if not sensor_id.endswith("_filtered"):
                self.sensor_filters[sensor_id].reset()
                
        # Reset state history
        self.state_history.clear()
        
        print("All filters reset")
        
    def export_state_history(self, filename: str):
        """Export state history to file"""
        if not self.state_history:
            print("No state history to export")
            return
            
        history_data = []
        for state in self.state_history:
            history_data.append({
                'timestamp': state.timestamp,
                'position': state.position,
                'velocity': state.velocity,
                'orientation': state.orientation,
                'angular_velocity': state.angular_velocity,
                'confidence': state.confidence
            })
            
        import json
        with open(filename, 'w') as f:
            json.dump(history_data, f, indent=2)
            
        print(f"State history exported to {filename}")


# Example usage and testing
if __name__ == "__main__":
    print("Testing Sensor Fusion Engine...")
    
    # Initialize sensor fusion engine
    fusion_engine = SensorFusionEngine()
    
    # Register sensors
    fusion_engine.register_sensor("imu_main", "imu", 1.0)
    fusion_engine.register_sensor("ultrasonic_front", "ultrasonic", 0.9)
    fusion_engine.register_sensor("camera_position", "camera", 0.8)
    
    # Initialize Kalman filter
    fusion_engine.initialize_kalman_filter(
        initial_position=(0.0, 0.0),
        initial_orientation=0.0
    )
    
    # Simulate sensor data
    print("\nSimulating sensor fusion...")
    
    for i in range(100):
        dt = 0.05  # 20 Hz
        
        # Generate simulated sensor readings
        current_time = time.time()
        
        # IMU data (accelerometer + gyroscope)
        accel_data = [0.1 * np.sin(i * dt), 0.05 * np.cos(i * dt), 9.8 + 0.1 * np.sin(i * dt * 0.5)]
        gyro_data = [0.2 * np.sin(i * dt * 0.5), 0.1 * np.cos(i * dt * 0.3), 0.05 * np.sin(i * dt * 0.1)]
        
        imu_reading = SensorReading(
            value={'acceleration': accel_data, 'gyroscope': gyro_data},
            timestamp=current_time,
            sensor_id="imu_main",
            confidence=0.95
        )
        fusion_engine.update_sensor_reading("imu_main", imu_reading)
        
        # Ultrasonic distance
        distance = 1.0 + 0.2 * np.sin(i * dt)
        ultra_reading = SensorReading(
            value=distance,
            timestamp=current_time,
            sensor_id="ultrasonic_front", 
            confidence=0.8
        )
        fusion_engine.update_sensor_reading("ultrasonic_front", ultra_reading)
        
        # Camera position estimate
        cam_x = 0.5 * i * dt
        cam_y = 0.1 * i * dt
        cam_reading = SensorReading(
            value=[cam_x, cam_y],
            timestamp=current_time,
            sensor_id="camera_position",
            confidence=0.7
        )
        fusion_engine.update_sensor_reading("camera_position", cam_reading)
        
        # Perform sensor fusion
        fused_state = fusion_engine.fuse_sensors(dt)
        
        # Print results periodically
        if i % 20 == 0:
            print(f"Step {i}:")
            print(f"  Position: {fused_state.position}")
            print(f"  Velocity: {fused_state.velocity}")
            print(f"  Orientation: {np.degrees(fused_state.orientation)}")
            print(f"  Confidence: {fused_state.confidence:.3f}")
            
    # Show final sensor status
    print(f"\nSensor Status:")
    sensor_status = fusion_engine.get_sensor_status()
    for sensor_id, status in sensor_status.items():
        print(f"  {sensor_id}: {status['status']} (confidence: {status['confidence']:.3f})")
        
    # Export state history
    fusion_engine.export_state_history("/workspace/real_world/robotics/data/fusion_history.json")
    
    print("\nSensor fusion testing complete!")
