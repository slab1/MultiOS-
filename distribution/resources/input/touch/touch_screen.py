"""
Touch Screen Driver for MultiOS

Provides support for both capacitive and resistive touch screens
with multi-touch gesture recognition.
"""

from typing import Dict, List, Tuple, Optional, Callable
from dataclasses import dataclass
from enum import Enum
import time
import math
import threading
import logging

from ..core.input_device import InputDevice, DeviceCapabilities
from ..core.input_event import InputEvent, EventType, EventPriority


class TouchType(Enum):
    """Touch screen types"""
    CAPACITIVE = "capacitive"
    RESISTIVE = "resistive"


class TouchState(Enum):
    """Touch point states"""
    NONE = "none"
    PRESSED = "pressed"
    MOVING = "moving"
    RELEASED = "released"


@dataclass
class TouchPoint:
    """Individual touch point data"""
    id: int
    x: float
    y: float
    pressure: float
    area: float
    state: TouchState
    timestamp: float
    
    # Additional data
    velocity_x: float = 0.0
    velocity_y: float = 0.0
    tilt_x: float = 0.0
    tilt_y: float = 0.0
    
    def distance_to(self, other: 'TouchPoint') -> float:
        """Calculate distance to another touch point"""
        return math.sqrt((self.x - other.x)**2 + (self.y - other.y)**2)
    
    def move_distance_to(self, other: 'TouchPoint') -> float:
        """Calculate movement distance from another touch point"""
        return math.sqrt((self.x - other.x)**2 + (self.y - other.y)**2)


class GestureState(Enum):
    """Gesture recognition states"""
    IDLE = "idle"
    DETECTING = "detecting"
    ACTIVE = "active"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class GestureRecognizer:
    """Multi-touch gesture recognition engine"""
    
    def __init__(self):
        self.active_gestures: Dict[int, Dict] = {}
        self.gesture_threshold = 10.0  # pixels
        self.pinch_threshold = 30.0    # pixels
        self.rotate_threshold = 15.0   # degrees
        self.swipe_threshold = 50.0    # pixels
        
        # Gesture patterns
        self.tap_history: List[Tuple[int, float, float]] = []
        self.gesture_patterns = {
            'tap': self._detect_tap,
            'double_tap': self._detect_double_tap,
            'swipe': self._detect_swipe,
            'pinch': self._detect_pinch,
            'rotate': self._detect_rotate,
            'long_press': self._detect_long_press
        }
        
        self.logger = logging.getLogger("touch.gesture")
    
    def update_touches(self, touches: List[TouchPoint]):
        """Update gesture recognition with new touch data"""
        self._update_gesture_states(touches)
        self._analyze_gestures(touches)
        return self._generate_gesture_events(touches)
    
    def _update_gesture_states(self, touches: List[TouchPoint]):
        """Update gesture tracking states"""
        # Update touch point velocities and movement
        for touch in touches:
            if touch.id in self.active_gestures:
                prev_touch = self.active_gestures[touch.id].get('last_touch')
                if prev_touch:
                    dt = touch.timestamp - prev_touch.timestamp
                    if dt > 0:
                        touch.velocity_x = (touch.x - prev_touch.x) / dt
                        touch.velocity_y = (touch.y - prev_touch.y) / dt
            
            self.active_gestures[touch.id] = {
                'touch': touch,
                'last_touch': touch,
                'start_time': time.time(),
                'start_position': (touch.x, touch.y),
                'total_distance': 0.0
            }
        
        # Clean up released touches
        active_ids = {t.id for t in touches}
        removed_ids = []
        for gesture_id in self.active_gestures:
            if gesture_id not in active_ids:
                self.active_gestures[gesture_id]['touch'].state = TouchState.RELEASED
                removed_ids.append(gesture_id)
        
        for gesture_id in removed_ids:
            del self.active_gestures[gesture_id]
    
    def _analyze_gestures(self, touches: List[TouchPoint]):
        """Analyze current touches for gestures"""
        if len(touches) == 1:
            self._analyze_single_touch(touches[0])
        elif len(touches) == 2:
            self._analyze_multi_touch(touches)
    
    def _analyze_single_touch(self, touch: TouchPoint):
        """Analyze single touch gestures"""
        gesture = self.active_gestures.get(touch.id, {})
        
        if touch.state == TouchState.PRESSED:
            gesture['gesture_type'] = 'touch_down'
            gesture['start_time'] = time.time()
        
        elif touch.state == TouchState.MOVING:
            # Detect swipe
            distance = math.sqrt((touch.x - gesture['start_position'][0])**2 + 
                               (touch.y - gesture['start_position'][1])**2)
            
            if distance > self.swipe_threshold:
                direction = self._get_swipe_direction(touch, gesture['start_position'])
                gesture['gesture_type'] = f'swipe_{direction}'
        
        elif touch.state == TouchState.RELEASED:
            # Detect tap
            total_time = time.time() - gesture.get('start_time', time.time())
            distance = math.sqrt((touch.x - gesture['start_position'][0])**2 + 
                               (touch.y - gesture['start_position'][1])**2)
            
            if total_time < 0.5 and distance < 5.0:
                self.tap_history.append((touch.id, touch.x, touch.y))
    
    def _analyze_multi_touch(self, touches: List[TouchPoint]):
        """Analyze multi-touch gestures"""
        if len(touches) != 2:
            return
        
        touch1, touch2 = touches
        
        # Calculate distance between touches
        distance = touch1.distance_to(touch2)
        
        # Get previous distance for comparison
        prev_gesture1 = self.active_gestures.get(touch1.id, {})
        prev_gesture2 = self.active_gestures.get(touch2.id, {})
        
        if 'last_distance' in prev_gesture1:
            prev_distance = prev_gesture1['last_distance']
            distance_change = distance - prev_distance
            
            # Detect pinch gesture
            if abs(distance_change) > self.pinch_threshold:
                if distance_change > 0:
                    # Pinch out (zoom in)
                    self.active_gestures[touch1.id]['gesture_type'] = 'pinch_out'
                    self.active_gestures[touch2.id]['gesture_type'] = 'pinch_out'
                else:
                    # Pinch in (zoom out)
                    self.active_gestures[touch1.id]['gesture_type'] = 'pinch_in'
                    self.active_gestures[touch2.id]['gesture_type'] = 'pinch_in'
        
        # Detect rotation gesture
        prev_angle = prev_gesture1.get('angle', 0)
        current_angle = math.atan2(touch2.y - touch1.y, touch2.x - touch1.x)
        angle_change = math.degrees(current_angle - prev_angle)
        
        if abs(angle_change) > self.rotate_threshold:
            if angle_change > 0:
                self.active_gestures[touch1.id]['gesture_type'] = 'rotate_cw'
                self.active_gestures[touch2.id]['gesture_type'] = 'rotate_cw'
            else:
                self.active_gestures[touch1.id]['gesture_type'] = 'rotate_ccw'
                self.active_gestures[touch2.id]['gesture_type'] = 'rotate_ccw'
        
        # Store current state
        self.active_gestures[touch1.id]['last_distance'] = distance
        self.active_gestures[touch1.id]['angle'] = current_angle
    
    def _get_swipe_direction(self, touch: TouchPoint, start_pos: Tuple[float, float]) -> str:
        """Get swipe direction from movement"""
        dx = touch.x - start_pos[0]
        dy = touch.y - start_pos[1]
        
        if abs(dx) > abs(dy):
            return 'right' if dx > 0 else 'left'
        else:
            return 'down' if dy > 0 else 'up'
    
    def _detect_tap(self, touches: List[TouchPoint]) -> Optional[Dict]:
        """Detect tap gesture"""
        if len(touches) == 1 and touches[0].state == TouchState.RELEASED:
            return {'type': 'tap', 'x': touches[0].x, 'y': touches[0].y}
        return None
    
    def _detect_double_tap(self, touches: List[TouchPoint]) -> Optional[Dict]:
        """Detect double tap gesture"""
        if (len(self.tap_history) >= 2 and 
            touches and touches[0].state == TouchState.RELEASED):
            
            tap1 = self.tap_history[-2]
            tap2 = self.tap_history[-1]
            
            # Check if taps are close in time and space
            time_diff = tap2[1] - tap1[1] if len(tap2) > 1 else float('inf')
            distance = math.sqrt((tap1[1] - tap2[1])**2 + (tap1[2] - tap2[2])**2)
            
            if time_diff < 0.5 and distance < 20.0:
                return {'type': 'double_tap', 'x': tap2[1], 'y': tap2[2]}
        return None
    
    def _detect_swipe(self, touches: List[TouchPoint]) -> Optional[Dict]:
        """Detect swipe gesture"""
        # Implemented in _analyze_single_touch
        pass
    
    def _detect_pinch(self, touches: List[TouchPoint]) -> Optional[Dict]:
        """Detect pinch gesture"""
        # Implemented in _analyze_multi_touch
        pass
    
    def _detect_rotate(self, touches: List[TouchPoint]) -> Optional[Dict]:
        """Detect rotate gesture"""
        # Implemented in _analyze_multi_touch
        pass
    
    def _detect_long_press(self, touches: List[TouchPoint]) -> Optional[Dict]:
        """Detect long press gesture"""
        if len(touches) == 1:
            touch = touches[0]
            press_time = time.time() - self.active_gestures.get(touch.id, {}).get('start_time', time.time())
            
            if (touch.state == TouchState.PRESSED and press_time > 1.0):
                return {'type': 'long_press', 'x': touch.x, 'y': touch.y}
        return None
    
    def _generate_gesture_events(self, touches: List[TouchPoint]) -> List[Dict]:
        """Generate gesture events from analysis"""
        events = []
        
        for touch in touches:
            gesture = self.active_gestures.get(touch.id, {})
            gesture_type = gesture.get('gesture_type')
            
            if gesture_type and touch.state == TouchState.RELEASED:
                # Create gesture event
                event = {
                    'type': gesture_type,
                    'x': touch.x,
                    'y': touch.y,
                    'duration': time.time() - gesture.get('start_time', time.time()),
                    'touch_count': len(touches),
                    'timestamp': touch.timestamp
                }
                events.append(event)
        
        # Check for double tap
        double_tap = self._detect_double_tap(touches)
        if double_tap:
            events.append(double_tap)
        
        return events


class TouchScreenDevice(InputDevice):
    """Touch screen device driver"""
    
    def __init__(self, device_id: str, touch_type: TouchType = TouchType.CAPACITIVE):
        super().__init__(device_id, 'touch')
        
        self.touch_type = touch_type
        self.screen_width = 1920
        self.screen_height = 1080
        self.max_touch_points = 10 if touch_type == TouchType.CAPACITIVE else 1
        
        # Touch data
        self.current_touches: Dict[int, TouchPoint] = {}
        self.gesture_recognizer = GestureRecognizer()
        
        # Calibration data
        self.calibration_matrix: List[List[float]] = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0]
        ]
        
        # Configuration
        self.config.update({
            'sensitivity': 0.8,
            'dead_zone': 5.0,
            'gesture_enabled': True,
            'palm_rejection': True,
            'noise_filtering': True,
            'calibration_active': False
        })
        
        self.logger = logging.getLogger(f"touch.{device_id}")
    
    def get_capabilities(self) -> DeviceCapabilities:
        """Get device capabilities"""
        capabilities = DeviceCapabilities()
        capabilities.supported_events = {
            EventType.TOUCH_START,
            EventType.TOUCH_MOVE,
            EventType.TOUCH_END,
            EventType.GESTURE_SWIPE,
            EventType.GESTURE_PINCH,
            EventType.GESTURE_ROTATE,
            EventType.GESTURE_TAP
        }
        capabilities.max_touch_points = self.max_touch_points
        capabilities.pressure_sensitive = True
        capabilities.gesture_support = True
        capabilities.sampling_rate = 60  # 60Hz
        capabilities.resolution_x = self.screen_width
        capabilities.resolution_y = self.screen_height
        capabilities.physical_size = (500, 300)  # 50cm x 30cm
        return capabilities
    
    def connect(self) -> bool:
        """Connect to touch screen"""
        try:
            # Initialize hardware interface
            self._init_hardware()
            
            self.is_connected = True
            self.logger.info(f"Connected {self.touch_type.value} touch screen")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to connect touch screen: {e}")
            return False
    
    def disconnect(self) -> bool:
        """Disconnect from touch screen"""
        try:
            self._cleanup_hardware()
            self.is_connected = False
            self.logger.info("Disconnected touch screen")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to disconnect: {e}")
            return False
    
    def _init_hardware(self):
        """Initialize hardware interface"""
        # Simulate hardware initialization
        # In real implementation, this would initialize I2C/SPI/USB interface
        pass
    
    def _cleanup_hardware(self):
        """Cleanup hardware interface"""
        # Simulate hardware cleanup
        pass
    
    def start_polling(self):
        """Start touch screen polling"""
        # Polling is handled by the base class worker loop
        pass
    
    def stop_polling(self):
        """Stop touch screen polling"""
        # Stopping is handled by the base class
        pass
    
    def calibrate(self) -> bool:
        """Calibrate touch screen"""
        try:
            self.logger.info("Starting touch screen calibration")
            
            # Simulate calibration process
            calibration_points = [
                (50, 50), (self.screen_width - 50, 50),
                (self.screen_width - 50, self.screen_height - 50),
                (50, self.screen_height - 50),
                (self.screen_width // 2, self.screen_height // 2)
            ]
            
            # In real implementation, this would prompt user to touch points
            # and read actual hardware coordinates
            for i, (expected_x, expected_y) in enumerate(calibration_points):
                self.logger.info(f"Calibration point {i+1}/{len(calibration_points)}")
                time.sleep(2)  # Wait for user input
            
            # Generate calibration matrix
            self._generate_calibration_matrix(calibration_points)
            
            self.is_calibrated = True
            self.logger.info("Touch screen calibration completed")
            return True
        
        except Exception as e:
            self.logger.error(f"Calibration failed: {e}")
            return False
    
    def _generate_calibration_matrix(self, points: List[Tuple[float, float]]):
        """Generate calibration matrix from calibration points"""
        # Simplified calibration matrix generation
        # In real implementation, this would use actual hardware coordinates
        self.calibration_matrix = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0]
        ]
    
    def _poll_device(self):
        """Poll touch screen for new data"""
        try:
            # Simulate reading touch data from hardware
            raw_touches = self._read_hardware_touches()
            
            # Process raw touches
            processed_touches = self._process_touch_data(raw_touches)
            
            # Update touch state
            self.current_touches = processed_touches
            
            # Generate events for touches
            self._generate_touch_events(processed_touches)
            
            # Generate gesture events if enabled
            if self.config.get('gesture_enabled', True):
                self._generate_gesture_events(processed_touches)
        
        except Exception as e:
            self.logger.error(f"Touch polling error: {e}")
    
    def _read_hardware_touches(self) -> List[Dict]:
        """Read raw touch data from hardware"""
        # Simulate hardware read
        # In real implementation, this would read from hardware registers
        touches = []
        
        # Simulate random touches for demonstration
        import random
        if random.random() < 0.1:  # 10% chance of touch
            touch_count = random.randint(1, min(3, self.max_touch_points))
            for i in range(touch_count):
                touches.append({
                    'id': i,
                    'x': random.randint(0, self.screen_width),
                    'y': random.randint(0, self.screen_height),
                    'pressure': random.uniform(0.1, 1.0),
                    'area': random.uniform(10, 100)
                })
        
        return touches
    
    def _process_touch_data(self, raw_touches: List[Dict]) -> Dict[int, TouchPoint]:
        """Process raw touch data"""
        processed = {}
        
        for raw_touch in raw_touches:
            # Apply calibration
            calibrated_pos = self._calibrate_coordinates(raw_touch['x'], raw_touch['y'])
            
            # Create touch point
            touch_point = TouchPoint(
                id=raw_touch['id'],
                x=calibrated_pos[0],
                y=calibrated_pos[1],
                pressure=raw_touch['pressure'],
                area=raw_touch['area'],
                state=TouchState.PRESSED,
                timestamp=time.time()
            )
            
            # Apply noise filtering
            if self.config.get('noise_filtering', True):
                touch_point = self._apply_noise_filtering(touch_point)
            
            # Apply dead zone
            if self.config.get('dead_zone', 0) > 0:
                touch_point = self._apply_dead_zone(touch_point)
            
            processed[raw_touch['id']] = touch_point
        
        return processed
    
    def _calibrate_coordinates(self, x: float, y: float) -> Tuple[float, float]:
        """Apply calibration matrix to coordinates"""
        # Apply calibration matrix transformation
        calibrated_x = (self.calibration_matrix[0][0] * x + 
                       self.calibration_matrix[0][1] * y + 
                       self.calibration_matrix[0][2])
        
        calibrated_y = (self.calibration_matrix[1][0] * x + 
                       self.calibration_matrix[1][1] * y + 
                       self.calibration_matrix[1][2])
        
        # Clamp to screen bounds
        calibrated_x = max(0, min(self.screen_width, calibrated_x))
        calibrated_y = max(0, min(self.screen_height, calibrated_y))
        
        return calibrated_x, calibrated_y
    
    def _apply_noise_filtering(self, touch_point: TouchPoint) -> TouchPoint:
        """Apply noise filtering to touch point"""
        # Simple median filter implementation
        # In a more sophisticated implementation, this would use historical data
        if self.current_touches.get(touch_point.id):
            prev_touch = self.current_touches[touch_point.id]
            
            # Apply smoothing
            alpha = self.config.get('sensitivity', 0.8)
            touch_point.x = alpha * prev_touch.x + (1 - alpha) * touch_point.x
            touch_point.y = alpha * prev_touch.y + (1 - alpha) * touch_point.y
        
        return touch_point
    
    def _apply_dead_zone(self, touch_point: TouchPoint) -> TouchPoint:
        """Apply dead zone filtering"""
        dead_zone = self.config.get('dead_zone', 5.0)
        
        if touch_point.id in self.current_touches:
            prev_touch = self.current_touches[touch_point.id]
            distance = touch_point.distance_to(prev_touch)
            
            if distance < dead_zone:
                # Don't move if within dead zone
                touch_point.x = prev_touch.x
                touch_point.y = prev_touch.y
        
        return touch_point
    
    def _generate_touch_events(self, touches: Dict[int, TouchPoint]):
        """Generate touch events from touch data"""
        # Generate events for new touches
        for touch_id, touch_point in touches.items():
            if touch_id not in self.current_touches:
                # New touch
                event = InputEvent(
                    event_type=EventType.TOUCH_START,
                    timestamp=touch_point.timestamp,
                    priority=EventPriority.HIGH,
                    device_id=self.device_id,
                    device_type='touch',
                    x=touch_point.x,
                    y=touch_point.y,
                    pressure=touch_point.pressure,
                    touch_id=touch_id,
                    metadata={'area': touch_point.area}
                )
                self._send_event(event)
            else:
                # Existing touch moved
                prev_touch = self.current_touches[touch_id]
                if (abs(touch_point.x - prev_touch.x) > 0.5 or 
                    abs(touch_point.y - prev_touch.y) > 0.5):
                    
                    event = InputEvent(
                        event_type=EventType.TOUCH_MOVE,
                        timestamp=touch_point.timestamp,
                        priority=EventPriority.HIGH,
                        device_id=self.device_id,
                        device_type='touch',
                        x=touch_point.x,
                        y=touch_point.y,
                        pressure=touch_point.pressure,
                        touch_id=touch_id,
                        metadata={'velocity_x': touch_point.velocity_x,
                                'velocity_y': touch_point.velocity_y}
                    )
                    self._send_event(event)
        
        # Generate end events for released touches
        for touch_id in list(self.current_touches.keys()):
            if touch_id not in touches:
                # Touch released
                event = InputEvent(
                    event_type=EventType.TOUCH_END,
                    timestamp=time.time(),
                    priority=EventPriority.HIGH,
                    device_id=self.device_id,
                    device_type='touch',
                    touch_id=touch_id
                )
                self._send_event(event)
    
    def _generate_gesture_events(self, touches: List[TouchPoint]):
        """Generate gesture recognition events"""
        gesture_events = self.gesture_recognizer.update_touches(touches)
        
        for gesture in gesture_events:
            # Map gesture types to event types
            event_type_mapping = {
                'tap': EventType.GESTURE_TAP,
                'double_tap': EventType.GESTURE_TAP,
                'swipe_up': EventType.GESTURE_SWIPE,
                'swipe_down': EventType.GESTURE_SWIPE,
                'swipe_left': EventType.GESTURE_SWIPE,
                'swipe_right': EventType.GESTURE_SWIPE,
                'pinch_in': EventType.GESTURE_PINCH,
                'pinch_out': EventType.GESTURE_PINCH,
                'rotate_cw': EventType.GESTURE_ROTATE,
                'rotate_ccw': EventType.GESTURE_ROTATE
            }
            
            event_type = event_type_mapping.get(gesture['type'])
            if event_type:
                event = InputEvent(
                    event_type=event_type,
                    timestamp=gesture['timestamp'],
                    priority=EventPriority.HIGH,
                    device_id=self.device_id,
                    device_type='touch',
                    x=gesture['x'],
                    y=gesture['y'],
                    gesture_data={
                        'gesture_type': gesture['type'],
                        'duration': gesture['duration'],
                        'touch_count': gesture['touch_count']
                    }
                )
                self._send_event(event)
    
    def get_touch_status(self) -> Dict[str, any]:
        """Get current touch screen status"""
        return {
            'touch_type': self.touch_type.value,
            'active_touches': len(self.current_touches),
            'max_touch_points': self.max_touch_points,
            'screen_resolution': (self.screen_width, self.screen_height),
            'is_calibrated': self.is_calibrated,
            'gestures_enabled': self.config.get('gesture_enabled', True),
            'current_touches': {
                touch_id: {
                    'x': touch.x,
                    'y': touch.y,
                    'pressure': touch.pressure,
                    'state': touch.state.value
                }
                for touch_id, touch in self.current_touches.items()
            }
        }
    
    def set_screen_resolution(self, width: int, height: int):
        """Set screen resolution"""
        self.screen_width = width
        self.screen_height = height
    
    def simulate_touch(self, x: float, y: float, pressure: float = 1.0, 
                      touch_id: int = 0) -> bool:
        """Simulate touch event for testing"""
        if not self.is_enabled:
            return False
        
        touch_point = TouchPoint(
            id=touch_id,
            x=x,
            y=y,
            pressure=pressure,
            area=50.0,
            state=TouchState.PRESSED,
            timestamp=time.time()
        )
        
        self.current_touches[touch_id] = touch_point
        
        # Generate touch start event
        event = InputEvent(
            event_type=EventType.TOUCH_START,
            timestamp=touch_point.timestamp,
            priority=EventPriority.HIGH,
            device_id=self.device_id,
            device_type='touch',
            x=x,
            y=y,
            pressure=pressure,
            touch_id=touch_id
        )
        self._send_event(event)
        
        return True
    
    def simulate_touch_release(self, touch_id: int = 0) -> bool:
        """Simulate touch release event"""
        if touch_id in self.current_touches:
            event = InputEvent(
                event_type=EventType.TOUCH_END,
                timestamp=time.time(),
                priority=EventPriority.HIGH,
                device_id=self.device_id,
                device_type='touch',
                touch_id=touch_id
            )
            self._send_event(event)
            del self.current_touches[touch_id]
            return True
        
        return False
    
    def simulate_multi_touch(self, touches: List[Tuple[float, float, float]]) -> bool:
        """Simulate multi-touch event"""
        if not self.is_enabled or len(touches) > self.max_touch_points:
            return False
        
        # Generate individual touch events
        for i, (x, y, pressure) in enumerate(touches):
            if not self.simulate_touch(x, y, pressure, i):
                return False
        
        return True