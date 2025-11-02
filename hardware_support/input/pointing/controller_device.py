"""
Game Controller and VR Device Support for MultiOS

Provides support for game controllers, VR headsets, and VR controllers
with educational gaming and virtual reality features.
"""

from typing import Dict, List, Optional, Tuple, Set, Any
from dataclasses import dataclass, field
from enum import Enum
import time
import math
import threading
import logging
from collections import deque, defaultdict

from ..core.input_device import InputDevice, DeviceCapabilities
from ..core.input_event import InputEvent, EventType, EventPriority


class ControllerType(Enum):
    """Controller types"""
    GAMEPAD = "gamepad"
    JOYSTICK = "joystick"
    STEERING_WHEEL = "steering_wheel"
    VR_CONTROLLER = "vr_controller"
    VR_HEADSET = "vr_headset"
    HAPTIC_DEVICE = "haptic_device"


class ConnectionType(Enum):
    """Controller connection types"""
    USB = "usb"
    BLUETOOTH = "bluetooth"
    WIRELESS = "wireless"
    NFC = "nfc"


@dataclass
class ControllerButton:
    """Controller button state"""
    name: str
    pressed: bool
    analog_value: float = 0.0  # For analog buttons
    last_pressed: float = 0.0
    
    def to_event(self, button_name: str, device_id: str, timestamp: float) -> InputEvent:
        """Convert to input event"""
        return InputEvent(
            event_type=EventType.CONTROLLER_BUTTON,
            timestamp=timestamp,
            priority=EventPriority.NORMAL,
            device_id=device_id,
            device_type='controller',
            button=button_name,
            metadata={'analog_value': self.analog_value, 'pressed': self.pressed}
        )


@dataclass
class ControllerAxis:
    """Controller axis state"""
    name: str
    value: float  # -1.0 to 1.0
    dead_zone: float = 0.1
    sensitivity: float = 1.0
    last_changed: float = 0.0
    
    def to_event(self, device_id: str, timestamp: float) -> InputEvent:
        """Convert to input event"""
        return InputEvent(
            event_type=EventType.CONTROLLER_AXIS,
            timestamp=timestamp,
            priority=EventPriority.NORMAL,
            device_id=device_id,
            device_type='controller',
            axis={self.name: self.value}
        )


@dataclass
class VRControllerState:
    """VR controller state"""
    position: Tuple[float, float, float] = (0, 0, 0)
    rotation: Tuple[float, float, float, float] = (0, 0, 0, 1)  # Quaternion
    trigger_pressure: float = 0.0
    grip_pressure: float = 0.0
    thumbstick: Tuple[float, float] = (0, 0)
    buttons: Dict[str, bool] = field(default_factory=dict)
    
    def to_events(self, device_id: str, timestamp: float) -> List[InputEvent]:
        """Convert to input events"""
        events = []
        
        # Position and rotation events
        event = InputEvent(
            event_type=EventType.VR_CONTROLLER,
            timestamp=timestamp,
            priority=EventPriority.NORMAL,
            device_id=device_id,
            device_type='controller',
            x=self.position[0],
            y=self.position[1],
            z=self.position[2],
            gesture_data={
                'rotation': self.rotation,
                'trigger_pressure': self.trigger_pressure,
                'grip_pressure': self.grip_pressure,
                'thumbstick': self.thumbstick
            }
        )
        events.append(event)
        
        # Button events
        for button_name, pressed in self.buttons.items():
            button_event = InputEvent(
                event_type=EventType.CONTROLLER_BUTTON,
                timestamp=timestamp,
                priority=EventPriority.NORMAL,
                device_id=device_id,
                device_type='controller',
                button=button_name,
                metadata={'pressed': pressed, 'vr_controller': True}
            )
            events.append(button_event)
        
        # Axis events for thumbstick
        if abs(self.thumbstick[0]) > 0.1 or abs(self.thumbstick[1]) > 0.1:
            axis_event = InputEvent(
                event_type=EventType.CONTROLLER_AXIS,
                timestamp=timestamp,
                priority=EventPriority.NORMAL,
                device_id=device_id,
                device_type='controller',
                axis={'thumbstick_x': self.thumbstick[0], 'thumbstick_y': self.thumbstick[1]}
            )
            events.append(axis_event)
        
        return events


@dataclass
class VRHeadsetState:
    """VR headset state"""
    position: Tuple[float, float, float] = (0, 0, 0)
    rotation: Tuple[float, float, float, float] = (0, 0, 0, 1)  # Quaternion
    left_eye_position: Tuple[float, float, float] = (0, 0, 0)
    right_eye_position: Tuple[float, float, float] = (0, 0, 0)
    fov_horizontal: float = 90.0
    fov_vertical: float = 90.0
    screen_resolution: Tuple[int, int] = (1920, 1080)
    
    def to_events(self, device_id: str, timestamp: float) -> List[InputEvent]:
        """Convert to input events"""
        return [
            InputEvent(
                event_type=EventType.VR_CONTROLLER,
                timestamp=timestamp,
                priority=EventPriority.NORMAL,
                device_id=device_id,
                device_type='controller',
                x=self.position[0],
                y=self.position[1],
                z=self.position[2],
                gesture_data={
                    'rotation': self.rotation,
                    'left_eye': self.left_eye_position,
                    'right_eye': self.right_eye_position,
                    'fov': {'horizontal': self.fov_horizontal, 'vertical': self.fov_vertical},
                    'headset': True
                }
            )
        ]


class EducationalGameMode(Enum):
    """Educational game modes"""
    MATH_TUTOR = "math_tutor"
    SCIENCE_EXPLORER = "science_explorer"
    LANGUAGE_LEARNER = "language_learner"
    PUZZLE_SOLVER = "puzzle_solver"
    VIRTUAL_LAB = "virtual_lab"
    HISTORY_SIMULATOR = "history_simulator"
    CREATIVE_WORKSPACE = "creative_workspace"
    ASSESSMENT_TOOL = "assessment_tool"


class ControllerInteractionHandler:
    """Handles educational controller interactions"""
    
    def __init__(self):
        self.current_game_mode: Optional[EducationalGameMode] = None
        self.interaction_history: deque = deque(maxlen=100)
        self.interaction_stats: Dict[str, int] = defaultdict(int)
        self.haptic_feedback_queue: List[Dict] = []
        
        # Educational interaction mappings
        self.interaction_mappings = {
            EducationalGameMode.MATH_TUTOR: {
                'button_a': 'submit_answer',
                'button_b': 'hint',
                'button_x': 'show_work',
                'button_y': 'skip_problem',
                'left_stick': 'navigate_menu',
                'right_stick': 'draw_diagram',
                'trigger': 'zoom_in',
                'bumpers': 'undo_redo'
            },
            EducationalGameMode.SCIENCE_EXPLORER: {
                'button_a': 'select_item',
                'button_b': 'back',
                'button_x': 'info',
                'button_y': 'tools',
                'left_stick': 'move_camera',
                'right_stick': 'rotate_view',
                'trigger': 'zoom_in',
                'bumpers': 'zoom_out'
            },
            EducationalGameMode.VIRTUAL_LAB: {
                'button_a': 'grab_item',
                'button_b': 'release_item',
                'button_x': 'measure',
                'button_y': 'record_data',
                'trigger': 'interact',
                'grip': 'hold_item',
                'thumbstick': 'navigate_lab'
            }
        }
    
    def handle_controller_input(self, event: InputEvent) -> Optional[Dict]:
        """Handle controller input for educational mode"""
        if not self.current_game_mode:
            return None
        
        mapping = self.interaction_mappings.get(self.current_game_mode, {})
        action = None
        
        if event.event_type == EventType.CONTROLLER_BUTTON:
            action = mapping.get(event.button)
        elif event.event_type == EventType.CONTROLLER_AXIS:
            axis_name = list(event.axis.keys())[0] if event.axis else None
            action = mapping.get(axis_name)
        
        if action:
            interaction = {
                'action': action,
                'device_id': event.device_id,
                'timestamp': event.timestamp,
                'game_mode': self.current_game_mode.value
            }
            
            self.interaction_history.append(interaction)
            self.interaction_stats[action] += 1
            
            return interaction
        
        return None
    
    def set_game_mode(self, game_mode: EducationalGameMode):
        """Set educational game mode"""
        self.current_game_mode = game_mode
    
    def get_interaction_stats(self) -> Dict[str, Any]:
        """Get interaction statistics"""
        return {
            'current_game_mode': self.current_game_mode.value if self.current_game_mode else None,
            'total_interactions': len(self.interaction_history),
            'interaction_counts': dict(self.interaction_stats),
            'recent_interactions': list(self.interaction_history)[-10:]
        }


class GameControllerDevice(InputDevice):
    """Game controller device driver"""
    
    def __init__(self, device_id: str, controller_type: ControllerType = ControllerType.GAMEPAD):
        super().__init__(device_id, 'controller')
        
        self.controller_type = controller_type
        self.connection_type = ConnectionType.USB
        
        # Controller state
        self.buttons: Dict[str, ControllerButton] = {}
        self.axes: Dict[str, ControllerAxis] = {}
        self.rumble_enabled = True
        self.led_enabled = True
        
        # VR-specific state
        self.vr_controller_state: Optional[VRControllerState] = None
        self.vr_headset_state: Optional[VRHeadsetState] = None
        
        # Configuration
        self.default_button_mapping = {
            'button_a': 0,
            'button_b': 1,
            'button_x': 2,
            'button_y': 3,
            'left_bumper': 4,
            'right_bumper': 5,
            'left_trigger': 6,
            'right_trigger': 7,
            'back': 8,
            'start': 9,
            'left_stick_press': 10,
            'right_stick_press': 11,
            'dpad_up': 12,
            'dpad_down': 13,
            'dpad_left': 14,
            'dpad_right': 15
        }
        
        self.default_axis_mapping = {
            'left_stick_x': 0,
            'left_stick_y': 1,
            'right_stick_x': 2,
            'right_stick_y': 3,
            'left_trigger': 4,
            'right_trigger': 5
        }
        
        # Educational features
        self.interaction_handler = ControllerInteractionHandler()
        self.educational_mode_enabled = False
        
        # Configuration
        self.config.update({
            'rumble_feedback': True,
            'led_feedback': True,
            'educational_mode': False,
            'dead_zone': 0.1,
            'sensitivity': 1.0,
            'vibration_intensity': 0.8
        })
        
        self.logger = logging.getLogger(f"controller.{device_id}")
    
    def get_capabilities(self) -> DeviceCapabilities:
        """Get device capabilities"""
        capabilities = DeviceCapabilities()
        capabilities.supported_events = {
            EventType.CONTROLLER_BUTTON,
            EventType.CONTROLLER_AXIS,
            EventType.VR_CONTROLLER
        }
        capabilities.sampling_rate = 60  # 60Hz
        capabilities.accuracy = 0.98
        
        if self.controller_type == ControllerType.VR_CONTROLLER:
            capabilities.gesture_support = True
            capabilities.motion_tracking = True
        
        return capabilities
    
    def connect(self) -> bool:
        """Connect to game controller"""
        try:
            self._init_controller_hardware()
            self._initialize_button_state()
            self._initialize_axis_state()
            
            # Initialize VR state if applicable
            if self.controller_type == ControllerType.VR_CONTROLLER:
                self.vr_controller_state = VRControllerState()
            elif self.controller_type == ControllerType.VR_HEADSET:
                self.vr_headset_state = VRHeadsetState()
            
            self.is_connected = True
            self.logger.info(f"Connected {self.controller_type.value} controller")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to connect controller: {e}")
            return False
    
    def disconnect(self) -> bool:
        """Disconnect from game controller"""
        try:
            self._cleanup_controller_hardware()
            self.is_connected = False
            self.logger.info("Disconnected game controller")
            return True
        
        except Exception as e:
            self.logger.error(f"Failed to disconnect: {e}")
            return False
    
    def _init_controller_hardware(self):
        """Initialize controller hardware"""
        # In real implementation, this would initialize USB/Bluetooth connection
        pass
    
    def _cleanup_controller_hardware(self):
        """Cleanup controller hardware"""
        # In real implementation, this would cleanup hardware resources
        pass
    
    def _initialize_button_state(self):
        """Initialize button state"""
        button_names = list(self.default_button_mapping.keys())
        for button_name in button_names:
            self.buttons[button_name] = ControllerButton(
                name=button_name,
                pressed=False,
                analog_value=0.0
            )
    
    def _initialize_axis_state(self):
        """Initialize axis state"""
        axis_names = list(self.default_axis_mapping.keys())
        for axis_name in axis_names:
            self.axes[axis_name] = ControllerAxis(
                name=axis_name,
                value=0.0,
                dead_zone=self.config.get('dead_zone', 0.1)
            )
    
    def start_polling(self):
        """Start controller polling"""
        pass
    
    def stop_polling(self):
        """Stop controller polling"""
        pass
    
    def calibrate(self) -> bool:
        """Calibrate game controller"""
        try:
            self.logger.info("Starting controller calibration")
            
            # Calibrate analog sticks
            self._calibrate_analog_sticks()
            
            # Calibrate triggers
            self._calibrate_triggers()
            
            # Test all buttons and axes
            self._test_controller_inputs()
            
            self.is_calibrated = True
            self.logger.info("Controller calibration completed")
            return True
        
        except Exception as e:
            self.logger.error(f"Controller calibration failed: {e}")
            return False
    
    def _calibrate_analog_sticks(self):
        """Calibrate analog stick dead zones and ranges"""
        self.logger.info("Calibrating analog sticks...")
        # Simulate calibration process
        time.sleep(2)
    
    def _calibrate_triggers(self):
        """Calibrate trigger sensitivity"""
        self.logger.info("Calibrating triggers...")
        # Simulate calibration process
        time.sleep(1)
    
    def _test_controller_inputs(self):
        """Test all controller inputs"""
        self.logger.info("Testing controller inputs...")
        # Simulate input testing
        time.sleep(1)
    
    def _poll_device(self):
        """Poll controller for new input data"""
        try:
            if not self.is_enabled or not self.is_connected:
                return
            
            # Read raw controller data
            raw_data = self._read_raw_controller_data()
            
            if raw_data:
                self._process_controller_data(raw_data)
        
        except Exception as e:
            self.logger.error(f"Controller polling error: {e}")
    
    def _read_raw_controller_data(self) -> Optional[Dict]:
        """Read raw controller data from hardware"""
        # Simulate controller data reading
        import random
        
        if random.random() < 0.3:  # 30% chance of input
            return {
                'buttons': [random.choice([True, False]) for _ in range(len(self.default_button_mapping))],
                'axes': [random.uniform(-1.0, 1.0) for _ in range(len(self.default_axis_mapping))]
            }
        
        return None
    
    def _process_controller_data(self, raw_data: Dict):
        """Process raw controller data"""
        button_data = raw_data.get('buttons', [])
        axis_data = raw_data.get('axes', [])
        
        # Process buttons
        for button_name, button_index in self.default_button_mapping.items():
            if button_index < len(button_data):
                button_state = button_data[button_index]
                if isinstance(button_state, bool):
                    # Digital button
                    self._update_button_state(button_name, button_state)
                else:
                    # Analog button
                    self._update_button_state(button_name, button_state > 0.5, button_state)
        
        # Process axes
        for axis_name, axis_index in self.default_axis_mapping.items():
            if axis_index < len(axis_data):
                axis_value = axis_data[axis_index]
                self._update_axis_state(axis_name, axis_value)
        
        # Update VR state if applicable
        if self.vr_controller_state:
            self._update_vr_controller_state()
        elif self.vr_headset_state:
            self._update_vr_headset_state()
    
    def _update_button_state(self, button_name: str, pressed: bool, analog_value: float = 0.0):
        """Update button state"""
        if button_name in self.buttons:
            button = self.buttons[button_name]
            was_pressed = button.pressed
            
            button.pressed = pressed
            button.analog_value = analog_value
            
            # Generate event on state change
            if was_pressed != pressed:
                button.last_pressed = time.time()
                
                event = button.to_event(button_name, self.device_id, time.time())
                self._send_event(event)
                
                # Handle educational interaction
                if self.config.get('educational_mode', False):
                    interaction = self.interaction_handler.handle_controller_input(event)
                    if interaction:
                        self.logger.info(f"Educational interaction: {interaction}")
    
    def _update_axis_state(self, axis_name: str, value: float):
        """Update axis state"""
        if axis_name in self.axes:
            axis = self.axes[axis_name]
            
            # Apply dead zone
            if abs(value) < axis.dead_zone:
                value = 0.0
            
            # Apply sensitivity
            value *= axis.sensitivity
            
            # Clamp to range
            value = max(-1.0, min(1.0, value))
            
            # Check for significant change
            if abs(value - axis.value) > 0.01:
                axis.value = value
                axis.last_changed = time.time()
                
                event = axis.to_event(self.device_id, time.time())
                self._send_event(event)
    
    def _update_vr_controller_state(self):
        """Update VR controller state"""
        if not self.vr_controller_state:
            return
        
        # Simulate VR controller tracking
        import random
        
        # Update position (random movement for simulation)
        position = self.vr_controller_state.position
        new_position = (
            position[0] + random.uniform(-0.01, 0.01),
            position[1] + random.uniform(-0.01, 0.01),
            position[2] + random.uniform(-0.01, 0.01)
        )
        
        # Update rotation (simulate rotation)
        rotation = self.vr_controller_state.rotation
        yaw_rotation = rotation[3] + random.uniform(-0.1, 0.1)
        new_rotation = (rotation[0], rotation[1], rotation[2], yaw_rotation % 1.0)
        
        self.vr_controller_state.position = new_position
        self.vr_controller_state.rotation = new_rotation
        
        # Generate VR events
        events = self.vr_controller_state.to_events(self.device_id, time.time())
        for event in events:
            self._send_event(event)
    
    def _update_vr_headset_state(self):
        """Update VR headset state"""
        if not self.vr_headset_state:
            return
        
        # Simulate VR headset tracking
        import random
        
        # Update position
        position = self.vr_headset_state.position
        new_position = (
            position[0] + random.uniform(-0.005, 0.005),
            position[1] + random.uniform(-0.005, 0.005),
            position[2] + random.uniform(-0.005, 0.005)
        )
        
        # Update rotation
        rotation = self.vr_headset_state.rotation
        yaw_rotation = rotation[3] + random.uniform(-0.05, 0.05)
        new_rotation = (rotation[0], rotation[1], rotation[2], yaw_rotation % 1.0)
        
        self.vr_headset_state.position = new_position
        self.vr_headset_state.rotation = new_rotation
        
        # Generate VR events
        events = self.vr_headset_state.to_events(self.device_id, time.time())
        for event in events:
            self._send_event(event)
    
    def set_rumble(self, left_intensity: float, right_intensity: float, duration: float = 0.1) -> bool:
        """Set haptic rumble feedback"""
        if not self.rumble_enabled or not self.is_connected:
            return False
        
        left_intensity = max(0.0, min(1.0, left_intensity))
        right_intensity = max(0.0, min(1.0, right_intensity))
        
        self.logger.info(f"Rumble: L={left_intensity:.2f}, R={right_intensity:.2f}, Duration={duration}s")
        
        # In real implementation, this would send rumble commands to hardware
        return True
    
    def set_led_color(self, r: int, g: int, b: int) -> bool:
        """Set controller LED color"""
        if not self.led_enabled or not self.is_connected:
            return False
        
        r, g, b = max(0, min(255, r)), max(0, min(255, g)), max(0, min(255, b))
        
        self.logger.info(f"LED Color: RGB({r}, {g}, {b})")
        
        # In real implementation, this would send LED commands to hardware
        return True
    
    def set_game_mode(self, game_mode: EducationalGameMode):
        """Set educational game mode"""
        self.interaction_handler.set_game_mode(game_mode)
        self.config['educational_mode'] = True
        
        # Set appropriate LED color for game mode
        mode_colors = {
            EducationalGameMode.MATH_TUTOR: (255, 0, 255),      # Magenta
            EducationalGameMode.SCIENCE_EXPLORER: (0, 255, 0),   # Green
            EducationalGameMode.LANGUAGE_LEARNER: (255, 255, 0), # Yellow
            EducationalGameMode.VIRTUAL_LAB: (0, 255, 255),      # Cyan
            EducationalGameMode.HISTORY_SIMULATOR: (255, 128, 0), # Orange
        }
        
        color = mode_colors.get(game_mode, (255, 255, 255))
        self.set_led_color(*color)
        
        self.logger.info(f"Set game mode: {game_mode.value}")
    
    def get_controller_status(self) -> Dict[str, any]:
        """Get current controller status"""
        status = {
            'controller_type': self.controller_type.value,
            'connection_type': self.connection_type.value,
            'is_calibrated': self.is_calibrated,
            'rumble_enabled': self.rumble_enabled,
            'led_enabled': self.led_enabled,
            'educational_mode': self.config.get('educational_mode', False),
            'buttons': {
                name: {
                    'pressed': button.pressed,
                    'analog_value': button.analog_value,
                    'last_pressed': button.last_pressed
                }
                for name, button in self.buttons.items()
            },
            'axes': {
                name: {
                    'value': axis.value,
                    'dead_zone': axis.dead_zone,
                    'last_changed': axis.last_changed
                }
                for name, axis in self.axes.items()
            }
        }
        
        if self.vr_controller_state:
            status['vr_controller'] = {
                'position': self.vr_controller_state.position,
                'rotation': self.vr_controller_state.rotation,
                'trigger_pressure': self.vr_controller_state.trigger_pressure,
                'grip_pressure': self.vr_controller_state.grip_pressure,
                'buttons': self.vr_controller_state.buttons
            }
        
        if self.vr_headset_state:
            status['vr_headset'] = {
                'position': self.vr_headset_state.position,
                'rotation': self.vr_headset_state.rotation,
                'fov': {
                    'horizontal': self.vr_headset_state.fov_horizontal,
                    'vertical': self.vr_headset_state.fov_vertical
                }
            }
        
        if self.config.get('educational_mode', False):
            status['interaction_stats'] = self.interaction_handler.get_interaction_stats()
        
        return status
    
    def simulate_button_press(self, button_name: str) -> bool:
        """Simulate button press for testing"""
        if button_name not in self.buttons:
            return False
        
        self._update_button_state(button_name, True)
        
        # Auto-release after short delay
        def release():
            time.sleep(0.1)
            self._update_button_state(button_name, False)
        
        threading.Thread(target=release, daemon=True).start()
        
        return True
    
    def simulate_axis_move(self, axis_name: str, value: float) -> bool:
        """Simulate axis movement for testing"""
        if axis_name not in self.axes:
            return False
        
        self._update_axis_state(axis_name, value)
        return True
    
    def get_supported_games(self) -> List[Dict[str, str]]:
        """Get list of supported educational games"""
        return [
            {
                'name': 'Math Challenge',
                'description': 'Practice arithmetic with controller',
                'game_mode': EducationalGameMode.MATH_TUTOR.value,
                'difficulty_levels': ['beginner', 'intermediate', 'advanced']
            },
            {
                'name': 'Science Lab',
                'description': 'Virtual laboratory experiments',
                'game_mode': EducationalGameMode.VIRTUAL_LAB.value,
                'subjects': ['physics', 'chemistry', 'biology']
            },
            {
                'name': 'History Explorer',
                'description': 'Navigate through historical periods',
                'game_mode': EducationalGameMode.HISTORY_SIMULATOR.value,
                'periods': ['ancient', 'medieval', 'modern']
            },
            {
                'name': 'Language Quest',
                'description': 'Learn languages through adventure',
                'game_mode': EducationalGameMode.LANGUAGE_LEARNER.value,
                'languages': ['spanish', 'french', 'german', 'mandarin']
            }
        ]