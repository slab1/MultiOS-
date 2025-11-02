"""
Stylus and Pen Input Device Support for MultiOS

Provides support for active and passive stylus pens with pressure sensitivity,
tilt detection, and educational drawing capabilities.
"""

from typing import Dict, List, Optional, Tuple, Any, Callable
from dataclasses import dataclass, field
from enum import Enum
import time
import math
import threading
import logging
from collections import deque

from ..core.input_device import InputDevice, DeviceCapabilities
from ..core.input_event import InputEvent, EventType, EventPriority


class PenType(Enum):
    """Types of stylus pens"""
    ACTIVE_STYLUS = "active_stylus"
    PASSIVE_STYLUS = "passive_stylus"
    DIGITAL_PEN = "digital_pen"
    EMR_PEN = "emr_pen"


class PenTool(Enum):
    """Stylus tool types"""
    PEN = "pen"
    PENCIL = "pencil"
    BRUSH = "brush"
    HIGHLIGHTER = "highlighter"
    ERASER = "eraser"
    SELECTOR = "selector"
    RULER = "ruler"
    COMPASS = "compass"


@dataclass
class PenStroke:
    """Pen stroke data"""
    points: List[Tuple[float, float, float]]  # x, y, pressure
    color: str = "#000000"
    width: float = 1.0
    tool: PenTool = PenTool.PEN
    timestamp: float = field(default_factory=time.time)
    duration: float = 0.0
    
    def add_point(self, x: float, y: float, pressure: float = 1.0):
        """Add point to stroke"""
        self.points.append((x, y, pressure))
    
    def get_length(self) -> float:
        """Calculate stroke length"""
        if len(self.points) < 2:
            return 0.0
        
        length = 0.0
        for i in range(1, len(self.points)):
            prev_point = self.points[i-1]
            curr_point = self.points[i]
            
            dx = curr_point[0] - prev_point[0]
            dy = curr_point[1] - prev_point[1]
            length += math.sqrt(dx*dx + dy*dy)
        
        return length
    
    def get_bounding_box(self) -> Tuple[float, float, float, float]:
        """Get bounding box (min_x, min_y, max_x, max_y)"""
        if not self.points:
            return (0, 0, 0, 0)
        
        x_coords = [p[0] for p in self.points]
        y_coords = [p[1] for p in self.points]
        
        return (min(x_coords), min(y_coords), max(x_coords), max(y_coords))
    
    def smooth_stroke(self, factor: float = 0.5) -> 'PenStroke':
        """Smooth stroke using simple averaging"""
        if len(self.points) < 3:
            return self
        
        smoothed_points = [self.points[0]]
        
        for i in range(1, len(self.points) - 1):
            prev_point = self.points[i-1]
            curr_point = self.points[i]
            next_point = self.points[i+1]
            
            smoothed_x = (prev_point[0] + 2*curr_point[0] + next_point[0]) / 4
            smoothed_y = (prev_point[1] + 2*curr_point[1] + next_point[1]) / 4
            smoothed_pressure = (prev_point[2] + 2*curr_point[2] + next_point[2]) / 4
            
            smoothed_points.append((smoothed_x, smoothed_y, smoothed_pressure))
        
        smoothed_points.append(self.points[-1])
        
        # Create new smoothed stroke
        smoothed_stroke = PenStroke(
            points=smoothed_points,
            color=self.color,
            width=self.width,
            tool=self.tool,
            timestamp=self.timestamp
        )
        
        return smoothed_stroke


@dataclass
class DrawingSession:
    """Drawing session data"""
    session_id: str
    start_time: float
    strokes: List[PenStroke] = field(default_factory=list)
    layer: int = 0
    zoom_level: float = 1.0
    canvas_size: Tuple[int, int] = (1920, 1080)
    
    def add_stroke(self, stroke: PenStroke):
        """Add stroke to session"""
        self.strokes.append(stroke)
    
    def get_total_stroke_length(self) -> float:
        """Get total length of all strokes"""
        return sum(stroke.get_length() for stroke in self.strokes)
    
    def get_layer_strokes(self, layer: int) -> List[PenStroke]:
        """Get strokes on specific layer"""
        return [stroke for stroke in self.strokes if hasattr(stroke, 'layer') and stroke.layer == layer]
    
    def undo_last_stroke(self) -> Optional[PenStroke]:
        """Undo last stroke"""
        if self.strokes:
            return self.strokes.pop()
        return None
    
    def clear_layer(self, layer: int):
        """Clear all strokes on specific layer"""
        self.strokes = [stroke for stroke in self.strokes 
                       if not (hasattr(stroke, 'layer') and stroke.layer == layer)]


class EducationalDrawingFeatures:
    """Educational drawing and note-taking features"""
    
    def __init__(self):
        self.math_tools = {
            'ruler': self._draw_ruler,
            'protractor': self._draw_protractor,
            'compass': self._draw_compass,
            'triangle': self._draw_triangle,
            'square': self._draw_square,
            'circle': self._draw_circle
        }
        
        self.text_recognition_enabled = True
        self.shape_recognition_enabled = True
        self.formula_recognition_enabled = True
    
    def recognize_shape(self, stroke: PenStroke) -> Optional[str]:
        """Recognize hand-drawn shape"""
        if not self.shape_recognition_enabled:
            return None
        
        # Simple shape recognition logic
        bounding_box = stroke.get_bounding_box()
        width = bounding_box[2] - bounding_box[0]
        height = bounding_box[3] - bounding_box[1]
        
        # Check for circle (width ≈ height, stroke forms closed shape)
        if abs(width - height) < 20 and self._is_closed_shape(stroke):
            return 'circle'
        
        # Check for square (aspect ratio close to 1:1, 4 corners)
        elif abs(width - height) < 50 and self._has_corners(stroke, 4):
            return 'square'
        
        # Check for triangle (3 corners)
        elif self._has_corners(stroke, 3):
            return 'triangle'
        
        # Check for line (very elongated)
        elif width > height * 3 or height > width * 3:
            return 'line'
        
        return None
    
    def recognize_text(self, stroke: PenStroke) -> Optional[str]:
        """Recognize hand-written text"""
        if not self.text_recognition_enabled:
            return None
        
        # In real implementation, this would use OCR/handwriting recognition
        # For simulation, return placeholder
        return "recognized_text"
    
    def recognize_formula(self, stroke: PenStroke) -> Optional[str]:
        """Recognize mathematical formula"""
        if not self.formula_recognition_enabled:
            return None
        
        # In real implementation, this would use mathematical formula recognition
        # For simulation, return placeholder
        return "x^2 + y^2 = r^2"
    
    def _is_closed_shape(self, stroke: PenStroke) -> bool:
        """Check if stroke forms a closed shape"""
        if len(stroke.points) < 10:
            return False
        
        start_point = stroke.points[0]
        end_point = stroke.points[-1]
        distance = math.sqrt((start_point[0] - end_point[0])**2 + (start_point[1] - end_point[1])**2)
        
        return distance < 50  # Consider closed if start and end are close
    
    def _has_corners(self, stroke: PenStroke, corner_count: int) -> bool:
        """Check if stroke has specified number of corners"""
        # Simplified corner detection
        if len(stroke.points) < 3 * corner_count:
            return False
        
        # Count significant direction changes
        direction_changes = 0
        for i in range(2, len(stroke.points)):
            p1 = stroke.points[i-2]
            p2 = stroke.points[i-1]
            p3 = stroke.points[i]
            
            # Calculate angles between consecutive segments
            angle1 = math.atan2(p2[1] - p1[1], p2[0] - p1[0])
            angle2 = math.atan2(p3[1] - p2[1], p3[0] - p2[0])
            
            angle_diff = abs(angle2 - angle1)
            if angle_diff > math.pi/4:  # Significant corner
                direction_changes += 1
        
        return direction_changes >= corner_count - 1
    
    def _draw_ruler(self, start_point: Tuple[float, float], end_point: Tuple[float, float]) -> List[PenStroke]:
        """Generate ruler drawing strokes"""
        strokes = []
        
        # Main line
        main_stroke = PenStroke(
            points=[start_point, end_point],
            color="#000000",
            width=2.0,
            tool=PenTool.RULER
        )
        strokes.append(main_stroke)
        
        # Tick marks
        x1, y1 = start_point
        x2, y2 = end_point
        length = math.sqrt((x2-x1)**2 + (y2-y1)**2)
        
        # Generate tick marks every 10 units
        num_ticks = int(length / 50)
        for i in range(num_ticks + 1):
            t = i / num_ticks
            tick_x = x1 + (x2 - x1) * t
            tick_y = y1 + (y2 - y1) * t
            
            # Perpendicular line for tick
            if i % 5 == 0:  # Major tick
                tick_length = 20
            else:  # Minor tick
                tick_length = 10
            
            # Calculate perpendicular direction
            if x2 != x1:
                perp_x = -(y2 - y1) / length
                perp_y = (x2 - x1) / length
            else:
                perp_x = 1
                perp_y = 0
            
            tick_start = (tick_x, tick_y, 1.0)
            tick_end = (tick_x + perp_x * tick_length, tick_y + perp_y * tick_length, 1.0)
            
            tick_stroke = PenStroke(
                points=[tick_start, tick_end],
                color="#666666",
                width=1.0,
                tool=PenTool.RULER
            )
            strokes.append(tick_stroke)
        
        return strokes
    
    def _draw_protractor(self, center: Tuple[float, float], radius: float) -> List[PenStroke]:
        """Generate protractor drawing strokes"""
        strokes = []
        
        # Outer circle
        circle_points = []
        for angle in range(0, 360, 5):
            rad = math.radians(angle)
            x = center[0] + radius * math.cos(rad)
            y = center[1] + radius * math.sin(rad)
            circle_points.append((x, y, 1.0))
        
        circle_stroke = PenStroke(
            points=circle_points,
            color="#000000",
            width=2.0,
            tool=PenTool.RULER
        )
        strokes.append(circle_stroke)
        
        # Degree marks
        for angle in range(0, 360, 10):
            rad = math.radians(angle)
            x1 = center[0] + radius * math.cos(rad)
            y1 = center[1] + radius * math.sin(rad)
            
            tick_length = 15 if angle % 30 == 0 else 8
            x2 = center[0] + (radius - tick_length) * math.cos(rad)
            y2 = center[1] + (radius - tick_length) * math.sin(rad)
            
            tick_stroke = PenStroke(
                points=[(x1, y1, 1.0), (x2, y2, 1.0)],
                color="#666666",
                width=1.0,
                tool=PenTool.RULER
            )
            strokes.append(tick_stroke)
        
        return strokes
    
    def _draw_compass(self, center: Tuple[float, float], radius: float) -> List[PenStroke]:
        """Generate compass drawing strokes"""
        strokes = []
        
        # Circle
        circle_points = []
        for angle in range(0, 360, 5):
            rad = math.radians(angle)
            x = center[0] + radius * math.cos(rad)
            y = center[1] + radius * math.sin(rad)
            circle_points.append((x, y, 1.0))
        
        circle_stroke = PenStroke(
            points=circle_points,
            color="#000000",
            width=2.0,
            tool=PenTool.COMPASS
        )
        strokes.append(circle_stroke)
        
        # Center point
        center_stroke = PenStroke(
            points=[center],
            color="#FF0000",
            width=3.0,
            tool=PenTool.COMPASS
        )
        strokes.append(center_stroke)
        
        return strokes
    
    def _draw_triangle(self, vertices: List[Tuple[float, float]]) -> List[PenStroke]:
        """Generate triangle drawing strokes"""
        strokes = []
        
        if len(vertices) != 3:
            return strokes
        
        # Triangle edges
        for i in range(3):
            p1 = vertices[i]
            p2 = vertices[(i + 1) % 3]
            edge_stroke = PenStroke(
                points=[p1, p2],
                color="#000000",
                width=2.0,
                tool=PenTool.RULER
            )
            strokes.append(edge_stroke)
        
        return strokes
    
    def _draw_square(self, center: Tuple[float, float], size: float) -> List[PenStroke]:
        """Generate square drawing strokes"""
        strokes = []
        
        x, y = center
        half_size = size / 2
        
        vertices = [
            (x - half_size, y - half_size),
            (x + half_size, y - half_size),
            (x + half_size, y + half_size),
            (x - half_size, y + half_size),
            (x - half_size, y - half_size)
        ]
        
        square_stroke = PenStroke(
            points=[(v[0], v[1], 1.0) for v in vertices],
            color="#000000",
            width=2.0,
            tool=PenTool.RULER
        )
        strokes.append(square_stroke)
        
        return strokes
    
    def _draw_circle(self, center: Tuple[float, float], radius: float) -> List[PenStroke]:
        """Generate circle drawing strokes"""
        strokes = []
        
        circle_points = []
        for angle in range(0, 360, 5):
            rad = math.radians(angle)
            x = center[0] + radius * math.cos(rad)
            y = center[1] + radius * math.sin(rad)
            circle_points.append((x, y, 1.0))
        
        circle_stroke = PenStroke(
            points=circle_points,
            color="#000000",
            width=2.0,
            tool=PenTool.RULER
        )
        strokes.append(circle_stroke)
        
        return strokes


class StylusDevice(InputDevice):
    """Stylus and pen device driver"""
    
    def __init__(self, device_id: str, pen_type: PenType = PenType.ACTIVE_STYLUS):
        super().__init__(device_id, 'stylus')
        
        self.pen_type = pen_type
        
        # Pen capabilities
        self.pressure_levels = 4096 if pen_type == PenType.ACTIVE_STYLUS else 256
        self.tilt_support = pen_type in [PenType.ACTIVE_STYLUS, PenType.DIGITAL_PEN]
        self.palm_rejection = pen_type == PenType.ACTIVE_STYLUS
        
        # Current pen state
        self.is_pressing = False
        self.current_pressure = 0.0
        self.current_tilt_x = 0.0
        self.current_tilt_y = 0.0
        self.current_position = (0.0, 0.0)
        self.current_tool = PenTool.PEN
        
        # Drawing session
        self.current_stroke: Optional[PenStroke] = None
        self.drawing_session = DrawingSession(f"session_{int(time.time())}", time.time())
        
        # Educational features
        self.educational_features = EducationalDrawingFeatures()
        self.shape_recognition_enabled = True
        self.text_recognition_enabled = True
        self.mathematical_tools_enabled = True
        
        # Pen settings
        self.default_color = "#000000"
        self.default_width = 2.0
        self.smoothing_enabled = True
        
        # Configuration
        self.config.update({
            'pressure_sensitivity': 1.0,
            'tilt_sensitivity': 1.0,
            'smoothing_factor': 0.5,
            'educational_mode': True,
            'shape_recognition': True,
            'palm_rejection': True
        })
        
        self.logger = logging.getLogger(f"stylus.{device_id}")
    
    def get_capabilities(self) -> DeviceCapabilities:
        """Get device capabilities"""
        capabilities = DeviceCapabilities()
        capabilities.supported_events = {
            EventType.STYLUS_PRESSURE,
            EventType.STYLUS_TILT,
            EventType.TOUCH_START,
            EventType.TOUCH_MOVE,
            EventType.TOUCH_END
        }
        capabilities.max_touch_points = 1  # Stylus is single-point input
        capabilities.pressure_sensitive = True
        capabilities.tilt_sensing = self.tilt_support
        capabilities.sampling_rate = 120  # 120Hz for smooth drawing
        capabilities.accuracy = 0.99
        
        if self.pen_type == PenType.ACTIVE_STYLUS:
            capabilities.pressure_levels = self.pressure_levels
        
        return capabilities
    
    def connect(self) -> bool:
        """Connect to stylus device"""
        try:
            self._init_stylus_hardware()
            self.is_connected = True
            self.logger.info(f"Connected {self.pen_type.value}")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to connect stylus: {e}")
            return False
    
    def disconnect(self) -> bool:
        """Disconnect from stylus device"""
        try:
            # End current stroke if drawing
            if self.is_pressing and self.current_stroke:
                self._end_stroke()
            
            self._cleanup_stylus_hardware()
            self.is_connected = False
            self.logger.info("Disconnected stylus")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to disconnect: {e}")
            return False
    
    def _init_stylus_hardware(self):
        """Initialize stylus hardware"""
        # In real implementation, this would initialize Wacom/EMR/Bluetooth connection
        pass
    
    def _cleanup_stylus_hardware(self):
        """Cleanup stylus hardware"""
        # In real implementation, this would cleanup hardware resources
        pass
    
    def start_polling(self):
        """Start stylus polling"""
        pass
    
    def stop_polling(self):
        """Stop stylus polling"""
        pass
    
    def calibrate(self) -> bool:
        """Calibrate stylus device"""
        try:
            self.logger.info("Starting stylus calibration")
            
            # Calibrate pressure sensitivity
            self._calibrate_pressure()
            
            # Calibrate tilt detection if supported
            if self.tilt_support:
                self._calibrate_tilt()
            
            # Test drawing accuracy
            self._test_drawing_accuracy()
            
            self.is_calibrated = True
            self.logger.info("Stylus calibration completed")
            return True
        
        except Exception as e:
            self.logger.error(f"Stylus calibration failed: {e}")
            return False
    
    def _calibrate_pressure(self):
        """Calibrate pressure sensitivity"""
        self.logger.info("Calibrating pressure sensitivity...")
        # Simulate calibration process
        time.sleep(2)
    
    def _calibrate_tilt(self):
        """Calibrate tilt detection"""
        self.logger.info("Calibrating tilt detection...")
        # Simulate calibration process
        time.sleep(2)
    
    def _test_drawing_accuracy(self):
        """Test drawing accuracy"""
        self.logger.info("Testing drawing accuracy...")
        # Simulate accuracy testing
        time.sleep(1)
    
    def _poll_device(self):
        """Poll stylus for new data"""
        try:
            if not self.is_enabled or not self.is_connected:
                return
            
            # Read raw stylus data
            raw_data = self._read_raw_stylus_data()
            
            if raw_data:
                self._process_stylus_data(raw_data)
        
        except Exception as e:
            self.logger.error(f"Stylus polling error: {e}")
    
    def _read_raw_stylus_data(self) -> Optional[Dict]:
        """Read raw stylus data from hardware"""
        # Simulate stylus data reading
        import random
        
        if random.random() < 0.2:  # 20% chance of input
            is_pressing = random.choice([True, False])
            pressure = random.uniform(0.0, 1.0) if is_pressing else 0.0
            
            return {
                'x': random.uniform(0, 1920),
                'y': random.uniform(0, 1080),
                'pressure': pressure,
                'tilt_x': random.uniform(-45, 45) if self.tilt_support else 0.0,
                'tilt_y': random.uniform(-45, 45) if self.tilt_support else 0.0,
                'pressing': is_pressing
            }
        
        return None
    
    def _process_stylus_data(self, raw_data: Dict):
        """Process raw stylus data"""
        x = raw_data['x']
        y = raw_data['y']
        pressure = raw_data['pressure']
        tilt_x = raw_data.get('tilt_x', 0.0)
        tilt_y = raw_data.get('tilt_y', 0.0)
        pressing = raw_data['pressing']
        
        # Apply palm rejection if enabled
        if self.config.get('palm_rejection', True) and not self._is_valid_stylus_input(pressure, tilt_x, tilt_y):
            return
        
        # Apply calibration
        if self.is_calibrated:
            x, y = self._apply_position_calibration(x, y)
            pressure = self._apply_pressure_calibration(pressure)
            if self.tilt_support:
                tilt_x = self._apply_tilt_calibration(tilt_x, 'x')
                tilt_y = self._apply_tilt_calibration(tilt_y, 'y')
        
        # Update current state
        self.current_position = (x, y)
        self.current_pressure = pressure
        self.current_tilt_x = tilt_x
        self.current_tilt_y = tilt_y
        
        # Handle press/release events
        if pressing and not self.is_pressing:
            self._start_stroke(x, y, pressure)
        elif not pressing and self.is_pressing:
            self._end_stroke()
        elif pressing and self.is_pressing:
            self._continue_stroke(x, y, pressure)
        
        # Send pressure event
        if pressure > 0:
            event = InputEvent(
                event_type=EventType.STYLUS_PRESSURE,
                timestamp=time.time(),
                priority=EventPriority.HIGH,
                device_id=self.device_id,
                device_type='stylus',
                x=x,
                y=y,
                pressure=pressure,
                metadata={'tilt_x': tilt_x, 'tilt_y': tilt_y}
            )
            self._send_event(event)
        
        # Send tilt event if significant
        if self.tilt_support and (abs(tilt_x) > 5 or abs(tilt_y) > 5):
            tilt_event = InputEvent(
                event_type=EventType.STYLUS_TILT,
                timestamp=time.time(),
                priority=EventPriority.NORMAL,
                device_id=self.device_id,
                device_type='stylus',
                tilt_x=tilt_x,
                tilt_y=tilt_y,
                x=x,
                y=y,
                pressure=pressure
            )
            self._send_event(tilt_event)
    
    def _is_valid_stylus_input(self, pressure: float, tilt_x: float, tilt_y: float) -> bool:
        """Check if input is from valid stylus (palm rejection)"""
        # Simple palm rejection logic
        if pressure > 0.8 and tilt_x == 0 and tilt_y == 0:
            return False  # Likely palm
        return True
    
    def _apply_position_calibration(self, x: float, y: float) -> Tuple[float, float]:
        """Apply position calibration"""
        # In real implementation, apply calibration matrix
        return x, y
    
    def _apply_pressure_calibration(self, pressure: float) -> float:
        """Apply pressure calibration"""
        # Apply sensitivity multiplier
        sensitivity = self.config.get('pressure_sensitivity', 1.0)
        return max(0.0, min(1.0, pressure * sensitivity))
    
    def _apply_tilt_calibration(self, tilt: float, axis: str) -> float:
        """Apply tilt calibration"""
        # Apply sensitivity multiplier
        sensitivity = self.config.get('tilt_sensitivity', 1.0)
        return max(-90.0, min(90.0, tilt * sensitivity))
    
    def _start_stroke(self, x: float, y: float, pressure: float):
        """Start new stroke"""
        self.is_pressing = True
        
        # Create new stroke
        self.current_stroke = PenStroke(
            points=[(x, y, pressure)],
            color=self.default_color,
            width=self.default_width,
            tool=self.current_tool
        )
        
        # Send touch start event
        event = InputEvent(
            event_type=EventType.TOUCH_START,
            timestamp=time.time(),
            priority=EventPriority.HIGH,
            device_id=self.device_id,
            device_type='stylus',
            x=x,
            y=y,
            pressure=pressure
        )
        self._send_event(event)
        
        self.logger.debug(f"Started stroke at ({x:.1f}, {y:.1f}) with pressure {pressure:.2f}")
    
    def _continue_stroke(self, x: float, y: float, pressure: float):
        """Continue existing stroke"""
        if self.current_stroke and pressure > 0:
            # Apply smoothing if enabled
            if self.smoothing_enabled and len(self.current_stroke.points) > 0:
                last_point = self.current_stroke.points[-1]
                # Simple smoothing: blend with previous point
                smooth_x = last_point[0] * 0.7 + x * 0.3
                smooth_y = last_point[1] * 0.7 + y * 0.3
                self.current_stroke.add_point(smooth_x, smooth_y, pressure)
            else:
                self.current_stroke.add_point(x, y, pressure)
            
            # Send touch move event
            event = InputEvent(
                event_type=EventType.TOUCH_MOVE,
                timestamp=time.time(),
                priority=EventPriority.HIGH,
                device_id=self.device_id,
                device_type='stylus',
                x=x,
                y=y,
                pressure=pressure
            )
            self._send_event(event)
    
    def _end_stroke(self):
        """End current stroke"""
        if self.current_stroke:
            # Add stroke to drawing session
            self.drawing_session.add_stroke(self.current_stroke)
            
            # Recognize shapes if enabled
            if self.shape_recognition_enabled:
                shape = self.educational_features.recognize_shape(self.current_stroke)
                if shape:
                    self.logger.info(f"Recognized shape: {shape}")
                    # Could trigger educational feedback
            
            # Recognize text if enabled
            if self.text_recognition_enabled:
                text = self.educational_features.recognize_text(self.current_stroke)
                if text:
                    self.logger.info(f"Recognized text: {text}")
            
            self.logger.debug(f"Completed stroke with {len(self.current_stroke.points)} points")
        
        # Send touch end event
        event = InputEvent(
            event_type=EventType.TOUCH_END,
            timestamp=time.time(),
            priority=EventPriority.HIGH,
            device_id=self.device_id,
            device_type='stylus'
        )
        self._send_event(event)
        
        self.current_stroke = None
        self.is_pressing = False
    
    def set_tool(self, tool: PenTool):
        """Set drawing tool"""
        self.current_tool = tool
        self.logger.info(f"Set tool to {tool.value}")
    
    def set_color(self, color: str):
        """Set pen color"""
        self.default_color = color
    
    def set_width(self, width: float):
        """Set pen width"""
        self.default_width = max(0.1, min(20.0, width))
    
    def set_smoothing(self, enabled: bool, factor: float = 0.5):
        """Enable/disable stroke smoothing"""
        self.smoothing_enabled = enabled
        if enabled:
            self.config['smoothing_factor'] = max(0.0, min(1.0, factor))
    
    def undo_last_stroke(self) -> bool:
        """Undo last stroke"""
        if self.drawing_session.undo_last_stroke():
            self.logger.info("Undid last stroke")
            return True
        return False
    
    def clear_canvas(self):
        """Clear entire canvas"""
        self.drawing_session.strokes.clear()
        self.logger.info("Cleared canvas")
    
    def get_drawing_stats(self) -> Dict[str, Any]:
        """Get drawing session statistics"""
        return {
            'session_id': self.drawing_session.session_id,
            'total_strokes': len(self.drawing_session.strokes),
            'total_length': self.drawing_session.get_total_stroke_length(),
            'current_tool': self.current_tool.value,
            'current_color': self.default_color,
            'current_width': self.default_width,
            'is_drawing': self.is_pressing,
            'pressure_levels': self.pressure_levels,
            'tilt_supported': self.tilt_support
        }
    
    def get_educational_tools(self) -> Dict[str, Callable]:
        """Get available educational drawing tools"""
        return self.educational_features.math_tools.copy()
    
    def simulate_stroke(self, points: List[Tuple[float, float, float]], 
                       tool: PenTool = PenTool.PEN) -> bool:
        """Simulate stylus stroke for testing"""
        if not self.is_enabled or not self.is_connected:
            return False
        
        # Start stroke
        x, y, pressure = points[0]
        self._start_stroke(x, y, pressure)
        
        # Continue stroke
        for x, y, pressure in points[1:-1]:
            self._continue_stroke(x, y, pressure)
        
        # End stroke
        if points:
            x, y, pressure = points[-1]
            self._end_stroke()
        
        return True
    
    def get_current_position(self) -> Tuple[float, float]:
        """Get current stylus position"""
        return self.current_position
    
    def get_current_pressure(self) -> float:
        """Get current pressure reading"""
        return self.current_pressure
    
    def get_current_tilt(self) -> Tuple[float, float]:
        """Get current tilt reading"""
        return (self.current_tilt_x, self.current_tilt_y)
    
    def export_strokes(self) -> List[Dict]:
        """Export all strokes to dictionary format"""
        return [
            {
                'points': stroke.points,
                'color': stroke.color,
                'width': stroke.width,
                'tool': stroke.tool.value,
                'timestamp': stroke.timestamp,
                'duration': stroke.duration,
                'length': stroke.get_length(),
                'bounding_box': stroke.get_bounding_box()
            }
            for stroke in self.drawing_session.strokes
        ]