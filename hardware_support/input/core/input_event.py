"""
Input Event System for MultiOS

Provides comprehensive input event handling with priority levels,
timestamps, and device-specific metadata.
"""

from enum import Enum
from dataclasses import dataclass
from typing import Dict, Any, Optional
import time
import json


class EventType(Enum):
    """Input event types"""
    TOUCH_START = "touch_start"
    TOUCH_MOVE = "touch_move"
    TOUCH_END = "touch_end"
    VOICE_COMMAND = "voice_command"
    VOICE_TEXT = "voice_text"
    GESTURE_SWIPE = "gesture_swipe"
    GESTURE_PINCH = "gesture_pinch"
    GESTURE_ROTATE = "gesture_rotate"
    GESTURE_TAP = "gesture_tap"
    MOTION_ACCEL = "motion_accel"
    MOTION_GYRO = "motion_gyro"
    STYLUS_PRESSURE = "stylus_pressure"
    STYLUS_TILT = "stylus_tilt"
    KEY_PRESS = "key_press"
    MOUSE_MOVE = "mouse_move"
    CONTROLLER_BUTTON = "controller_button"
    CONTROLLER_AXIS = "controller_axis"
    VR_CONTROLLER = "vr_controller"
    SYSTEM_COMMAND = "system_command"


class EventPriority(Enum):
    """Input event priority levels"""
    LOW = 1
    NORMAL = 2
    HIGH = 3
    CRITICAL = 4
    SYSTEM = 5


@dataclass
class InputEvent:
    """Base input event class"""
    event_type: EventType
    timestamp: float
    priority: EventPriority
    device_id: str
    device_type: str
    
    # Event data
    x: Optional[float] = None
    y: Optional[float] = None
    z: Optional[float] = None
    pressure: Optional[float] = None
    tilt_x: Optional[float] = None
    tilt_y: Optional[float] = None
    
    # Multi-touch data
    touch_id: Optional[int] = None
    gesture_data: Optional[Dict[str, Any]] = None
    
    # Voice data
    voice_text: Optional[str] = None
    confidence: Optional[float] = None
    
    # Motion data
    acceleration: Optional[Dict[str, float]] = None
    rotation: Optional[Dict[str, float]] = None
    
    # Controller data
    button: Optional[str] = None
    axis: Optional[Dict[str, float]] = None
    
    # Metadata
    metadata: Optional[Dict[str, Any]] = None
    
    def __post_init__(self):
        """Initialize missing timestamp if not provided"""
        if self.timestamp is None:
            self.timestamp = time.time()
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert event to dictionary"""
        result = {}
        for key, value in self.__dict__.items():
            if isinstance(value, (EventType, EventPriority)):
                result[key] = value.value
            elif hasattr(value, 'value') and hasattr(value, '__class__'):
                result[key] = value.value
            else:
                result[key] = value
        return result
    
    def to_json(self) -> str:
        """Convert event to JSON string"""
        return json.dumps(self.to_dict(), indent=2)
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'InputEvent':
        """Create event from dictionary"""
        # Convert string enums back to enum objects
        if 'event_type' in data and isinstance(data['event_type'], str):
            data['event_type'] = EventType(data['event_type'])
        if 'priority' in data and isinstance(data['priority'], str):
            data['priority'] = EventPriority(data['priority'])
        
        return cls(**data)
    
    def get_distance(self, other: 'InputEvent') -> float:
        """Calculate distance to another event in screen coordinates"""
        if self.x is None or self.y is None or other.x is None or other.y is None:
            return float('inf')
        
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5
    
    def is_same_gesture(self, other: 'InputEvent', threshold: float = 50.0) -> bool:
        """Check if this event is part of the same gesture as another"""
        return (self.device_id == other.device_id and 
                self.event_type == other.event_type and
                self.get_distance(other) < threshold)


class InputEventFilter:
    """Filter for input events"""
    
    def __init__(self):
        self.event_types: set[EventType] = set()
        self.device_ids: set[str] = set()
        self.priority_threshold: Optional[EventPriority] = None
        self.x_range: Optional[tuple[float, float]] = None
        self.y_range: Optional[tuple[float, float]] = None
    
    def add_event_type(self, event_type: EventType) -> 'InputEventFilter':
        """Add event type to filter"""
        self.event_types.add(event_type)
        return self
    
    def add_device_id(self, device_id: str) -> 'InputEventFilter':
        """Add device ID to filter"""
        self.device_ids.add(device_id)
        return self
    
    def set_priority_threshold(self, priority: EventPriority) -> 'InputEventFilter':
        """Set minimum priority threshold"""
        self.priority_threshold = priority
        return self
    
    def set_area_filter(self, x_min: float, y_min: float, x_max: float, y_max: float) -> 'InputEventFilter':
        """Set area filter for coordinates"""
        self.x_range = (x_min, x_max)
        self.y_range = (y_min, y_max)
        return self
    
    def matches(self, event: InputEvent) -> bool:
        """Check if event matches filter criteria"""
        # Event type filter
        if self.event_types and event.event_type not in self.event_types:
            return False
        
        # Device ID filter
        if self.device_ids and event.device_id not in self.device_ids:
            return False
        
        # Priority filter
        if self.priority_threshold and event.priority.value < self.priority_threshold.value:
            return False
        
        # Area filter
        if (self.x_range and (event.x is None or not (self.x_range[0] <= event.x <= self.x_range[1]))) or \
           (self.y_range and (event.y is None or not (self.y_range[0] <= event.y <= self.y_range[1]))):
            return False
        
        return True