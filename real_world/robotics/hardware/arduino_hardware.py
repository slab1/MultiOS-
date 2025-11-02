"""
Arduino Hardware Abstraction Layer

Implements hardware interface for Arduino-based robots
"""

import time
import threading
import serial
import numpy as np
from typing import Optional, List
from .robot import BaseHardware, SensorData, ControlCommand


class ArduinoHardware(BaseHardware):
    """Arduino hardware interface implementation"""
    
    def __init__(self, config: Optional[dict] = None):
        self.config = config or self._default_config()
        self.is_connected_flag = False
        self.serial_connection = None
        self.read_buffer = []
        self.write_buffer = []
        self._read_thread = None
        self._running = False
        
        # Motor pins configuration
        self.left_motor_pins = self.config['motors']['left_motor']  # [direction_pin, pwm_pin]
        self.right_motor_pins = self.config['motors']['right_motor']  # [direction_pin, pwm_pin]
        
        # Sensor pins configuration
        self.ultrasonic_pins = self.config['sensors']['ultrasonic']
        self.line_sensor_pins = self.config['sensors']['line_sensor']
        
        # Robot dimensions for kinematics
        self.wheel_radius = 0.03  # 30mm typical wheel radius
        self.wheel_base = 0.10    # 100mm distance between wheels
        
    def _default_config(self) -> dict:
        """Return default Arduino configuration"""
        return {
            'motors': {
                'left_motor': [5, 6],      # direction, pwm pins
                'right_motor': [9, 10]     # direction, pwm pins
            },
            'sensors': {
                'ultrasonic': [7, 8],      # trigger, echo pins
                'line_sensor': [0, 1, 2, 3, 4]  # analog pins for line sensors
            },
            'communication': {
                'port': '/dev/ttyUSB0',    # Arduino serial port
                'baudrate': 115200,
                'timeout': 1.0
            }
        }
        
    def connect(self) -> bool:
        """Connect to Arduino hardware"""
        try:
            print("Connecting to Arduino...")
            
            # In simulation mode, we'll simulate the connection
            if self.config.get('simulation', False):
                print("Arduino simulation mode enabled")
                self.is_connected_flag = True
                self._running = True
                self._start_simulation()
                return True
                
            # Real Arduino connection (commented out for simulation)
            """
            port = self.config['communication']['port']
            baudrate = self.config['communication']['baudrate']
            timeout = self.config['communication']['timeout']
            
            self.serial_connection = serial.Serial(port, baudrate, timeout=timeout)
            time.sleep(2)  # Wait for Arduino to reset
            
            # Send initialization command
            self._send_command("INIT", {})
            response = self._read_response()
            
            if response and response.get('status') == 'ok':
                self.is_connected_flag = True
                self._running = True
                self._start_threads()
                return True
            else:
                print("Arduino initialization failed")
                return False
            """
            
            # Simulation fallback
            self.is_connected_flag = True
            self._running = True
            self._start_simulation()
            return True
            
        except Exception as e:
            print(f"Failed to connect to Arduino: {e}")
            # Fallback to simulation
            self.is_connected_flag = True
            self._running = True
            self._start_simulation()
            return True
            
    def _start_simulation(self):
        """Start simulation mode"""
        self._sim_thread = threading.Thread(target=self._simulation_loop, daemon=True)
        self._sim_thread.start()
        
    def _simulation_loop(self):
        """Simulation loop for Arduino operation"""
        while self._running:
            time.sleep(0.05)  # 20 Hz simulation rate
            # Simulate sensor readings and motor responses
            
    def _start_threads(self):
        """Start communication threads"""
        self._read_thread = threading.Thread(target=self._read_loop, daemon=True)
        self._read_thread.start()
        
    def _read_loop(self):
        """Serial read loop"""
        while self._running and self.serial_connection:
            try:
                if self.serial_connection.in_waiting:
                    line = self.serial_connection.readline().decode().strip()
                    self._process_message(line)
                time.sleep(0.01)
            except Exception as e:
                print(f"Read error: {e}")
                
    def _process_message(self, message: str):
        """Process incoming serial message"""
        try:
            # Parse Arduino message format: "TYPE:value1,value2,..."
            if ':' not in message:
                return
                
            msg_type, values = message.split(':', 1)
            parsed_values = [float(v) for v in values.split(',')]
            
            if msg_type == 'SENSORS':
                # Update sensor data
                self._last_sensor_data = parsed_values
            elif msg_type == 'STATUS':
                # Handle status messages
                pass
                
        except Exception as e:
            print(f"Error processing message: {e}")
            
    def _send_command(self, command: str, params: dict):
        """Send command to Arduino"""
        if self.serial_connection:
            message = f"{command}:{','.join(str(v) for v in params.values())}\n"
            self.serial_connection.write(message.encode())
            
    def disconnect(self) -> bool:
        """Disconnect from Arduino hardware"""
        try:
            self._running = False
            
            if self.serial_connection:
                self._send_command("STOP", {})
                self.serial_connection.close()
                
            print("Arduino disconnected")
            return True
            
        except Exception as e:
            print(f"Error disconnecting from Arduino: {e}")
            return False
            
    def read_sensors(self) -> SensorData:
        """Read data from Arduino sensors"""
        try:
            if hasattr(self, '_last_sensor_data'):
                data = self._last_sensor_data
            else:
                # Generate simulated sensor data
                data = self._generate_simulated_sensors()
                
            return SensorData(
                ultrasonic_distance=data[0] if len(data) > 0 else None,
                color_reflectance=data[1:6] if len(data) > 5 else None,  # Line sensor array
                timestamp=time.time()
            )
            
        except Exception as e:
            print(f"Error reading Arduino sensors: {e}")
            return SensorData(timestamp=time.time())
            
    def _generate_simulated_sensors(self) -> List[float]:
        """Generate simulated sensor data"""
        # Ultrasonic distance (0.1 to 2.0 meters)
        distance = 0.5 + 0.3 * np.sin(time.time() * 2)
        
        # Line sensor array (5 sensors, 0-1023 for Arduino ADC)
        line_sensors = []
        for i in range(5):
            # Simulate line detection with varying values
            base_value = 200 + 100 * np.sin(time.time() + i)
            noise = np.random.normal(0, 20)
            line_sensors.append(max(0, min(1023, base_value + noise)))
            
        return [distance] + line_sensors
        
    def send_commands(self, command: ControlCommand) -> bool:
        """Send control commands to Arduino motors"""
        try:
            # Convert motor speeds from -1.0 to 1.0 to PWM values (0-255)
            left_pwm = int((command.left_motor_speed + 1.0) * 127.5)
            right_pwm = int((command.right_motor_speed + 1.0) * 127.5)
            
            # Clamp PWM values
            left_pwm = max(0, min(255, left_pwm))
            right_pwm = max(0, min(255, right_pwm))
            
            # In real implementation, send to Arduino
            if self.config.get('simulation', False):
                print(f"Arduino motors - Left: {left_pwm}, Right: {right_pwm}")
                
            # Real Arduino command (commented out for simulation)
            """
            self._send_command("MOTORS", {
                'left': left_pwm,
                'right': right_pwm
            })
            """
            
            return True
            
        except Exception as e:
            print(f"Error sending commands to Arduino: {e}")
            return False
            
    def is_connected(self) -> bool:
        """Check if Arduino is connected"""
        return self.is_connected_flag and self._running
        
    def calibrate_line_sensors(self, samples: int = 100):
        """Calibrate line sensors"""
        print("Starting line sensor calibration...")
        print("Place sensors over different surfaces...")
        
        min_values = [1023] * 5  # Maximum ADC value
        max_values = [0] * 5     # Minimum ADC value
        
        for i in range(samples):
            sensor_data = self.read_sensors()
            if sensor_data.color_reflectance:
                for j, value in enumerate(sensor_data.color_reflectance):
                    min_values[j] = min(min_values[j], value)
                    max_values[j] = max(max_values[j], value)
                    
            time.sleep(0.01)
            
        calibration_data = {
            'min_values': min_values,
            'max_values': max_values
        }
        
        print(f"Calibration complete: Min={min_values}, Max={max_values}")
        return calibration_data
        
    def read_line_position(self, calibration_data: dict) -> float:
        """Read line position using calibrated line sensors"""
        sensor_data = self.read_sensors()
        if not sensor_data.color_reflectance:
            return 0.0
            
        reflectance = sensor_data.color_reflectance
        min_vals = calibration_data['min_values']
        max_vals = calibration_data['max_values']
        
        # Normalize sensor values to 0-1 range
        normalized = []
        for i, value in enumerate(reflectance):
            if max_vals[i] > min_vals[i]:
                norm = (value - min_vals[i]) / (max_vals[i] - min_vals[i])
                normalized.append(norm)
            else:
                normalized.append(0.5)
                
        # Calculate weighted average for line position
        weights = [-2, -1, 0, 1, 2]  # Position weights for 5 sensors
        line_position = sum(w * r for w, r in zip(weights, normalized))
        
        return line_position / 2.0  # Normalize to -1 to 1 range
        
    def emergency_stop(self):
        """Emergency stop - stop all motors"""
        print("Arduino emergency stop")
        self.send_commands(ControlCommand(0.0, 0.0))


class ArduinoMotorController:
    """Arduino Motor Controller abstraction"""
    
    def __init__(self, pin_config: List[int]):
        self.direction_pin = pin_config[0]
        self.pwm_pin = pin_config[1]
        self.current_speed = 0
        self.current_direction = 1  # 1 for forward, -1 for backward
        
    def set_speed(self, speed: float):
        """Set motor speed (-1.0 to 1.0)"""
        self.current_speed = speed
        self.current_direction = 1 if speed >= 0 else -1
        
        # Convert to PWM value (0-255)
        pwm_value = int(abs(speed) * 255)
        
        print(f"Motor Dir:{self.direction_pin}, PWM:{self.pwm_pin} -> Speed: {speed} (PWM: {pwm_value})")
        
    def get_speed(self) -> float:
        """Get current motor speed"""
        return self.current_speed * self.current_direction
        
    def stop(self):
        """Stop motor"""
        self.current_speed = 0
        print(f"Motor {self.direction_pin} stopped")


class ArduinoUltrasonicSensor:
    """Arduino Ultrasonic Sensor abstraction"""
    
    def __init__(self, pin_config: List[int]):
        self.trigger_pin = pin_config[0]
        self.echo_pin = pin_config[1]
        
    def read_distance(self) -> float:
        """Read distance in meters using ultrasonic sensor"""
        # Simulate ultrasonic reading
        base_distance = 0.5 + 0.2 * np.sin(time.time() * 1.5)
        noise = np.random.normal(0, 0.02)
        distance = max(0.02, min(3.0, base_distance + noise))
        
        print(f"Ultrasonic (Trig:{self.trigger_pin}, Echo:{self.echo_pin}) -> Distance: {distance:.2f}m")
        return distance


class ArduinoLineSensor:
    """Arduino Line Sensor Array"""
    
    def __init__(self, pin_config: List[int]):
        self.sensor_pins = pin_config
        self.calibration_data = None
        
    def read_reflectance(self) -> List[int]:
        """Read reflectance from all line sensors"""
        # Simulate line sensor readings
        readings = []
        for i, pin in enumerate(self.sensor_pins):
            # Generate simulated readings with some variation
            base_value = 400 + 200 * np.sin(time.time() + i * 0.5)
            noise = np.random.normal(0, 30)
            value = int(max(0, min(1023, base_value + noise)))
            readings.append(value)
            
        print(f"Line sensors {self.sensor_pins} -> {readings}")
        return readings
        
    def set_calibration(self, min_values: List[int], max_values: List[int]):
        """Set sensor calibration data"""
        self.calibration_data = {
            'min_values': min_values,
            'max_values': max_values
        }
        
    def read_normalized(self) -> List[float]:
        """Read normalized reflectance values (0.0 to 1.0)"""
        if not self.calibration_data:
            return [0.5] * len(self.sensor_pins)
            
        raw_readings = self.read_reflectance()
        min_vals = self.calibration_data['min_values']
        max_vals = self.calibration_data['max_values']
        
        normalized = []
        for i, reading in enumerate(raw_readings):
            if max_vals[i] > min_vals[i]:
                norm = (reading - min_vals[i]) / (max_vals[i] - min_vals[i])
            else:
                norm = 0.5
            normalized.append(norm)
            
        return normalized