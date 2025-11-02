"""
Device Manager for MultiOS Input System

Central manager for all input devices, handles device discovery, 
registration, event routing, and lifecycle management.
"""

from typing import Dict, List, Optional, Callable, Type
from collections import defaultdict, deque
import logging
import threading
import time
from concurrent.futures import ThreadPoolExecutor
import json

from .input_device import InputDevice
from .input_event import InputEvent, EventType, EventPriority
from ..touch.touch_screen import TouchScreenDevice
from ..voice.voice_input import VoiceInputDevice
from ..motion.motion_sensor import MotionSensorDevice
from ..pointing.stylus_device import StylusDevice
from ..pointing.controller_device import GameControllerDevice


class DeviceManager:
    """Central device manager for all input devices"""
    
    def __init__(self):
        self.devices: Dict[str, InputDevice] = {}
        self.device_factories: Dict[str, Type[InputDevice]] = {}
        self.event_routers: Dict[EventType, List[Callable]] = defaultdict(list)
        self.global_event_handlers: List[Callable] = []
        self.device_event_handlers: Dict[str, List[Callable]] = defaultdict(list)
        
        # Global configuration
        self.config: Dict[str, any] = {
            'enable_device_discovery': True,
            'auto_connect_devices': True,
            'event_processing_threads': 4,
            'event_history_size': 10000,
            'event_logging': True,
            'statistics_interval': 60.0  # seconds
        }
        
        # Event processing
        self.event_queue: deque = deque(maxlen=self.config['event_history_size'])
        self.event_processors: List[threading.Thread] = []
        self.running = False
        self._lock = threading.RLock()
        self._executor = ThreadPoolExecutor(max_workers=self.config['event_processing_threads'])
        
        # Statistics
        self.statistics: Dict[str, any] = {
            'start_time': time.time(),
            'total_events': 0,
            'devices_connected': 0,
            'events_by_type': defaultdict(int),
            'events_by_device': defaultdict(int)
        }
        
        # Logging
        self.logger = logging.getLogger("input.manager")
        self.logger.setLevel(logging.INFO)
        
        # Register built-in device types
        self._register_builtin_devices()
    
    def _register_builtin_devices(self):
        """Register built-in device types"""
        self.register_device_type('touch', TouchScreenDevice)
        self.register_device_type('voice', VoiceInputDevice)
        self.register_device_type('motion', MotionSensorDevice)
        self.register_device_type('stylus', StylusDevice)
        self.register_device_type('controller', GameControllerDevice)
    
    def register_device_type(self, device_type: str, device_class: Type[InputDevice]):
        """Register a device factory"""
        with self._lock:
            self.device_factories[device_type] = device_class
            self.logger.info(f"Registered device type: {device_type}")
    
    def discover_devices(self) -> List[Dict[str, any]]:
        """Discover available input devices"""
        discovered = []
        
        # Simulate device discovery - in real implementation this would
        # scan hardware interfaces
        discovered.extend([
            {
                'device_id': 'touch_0',
                'device_type': 'touch',
                'name': 'Main Touch Screen',
                'capabilities': {
                    'max_touch_points': 10,
                    'pressure_sensitive': True,
                    'gesture_support': True
                }
            },
            {
                'device_id': 'voice_0',
                'device_type': 'voice',
                'name': 'Voice Input System',
                'capabilities': {
                    'voice_recognition': True,
                    'noise_cancellation': True,
                    'real_time_processing': True
                }
            },
            {
                'device_id': 'motion_0',
                'device_type': 'motion',
                'name': '9-DOF Motion Sensor',
                'capabilities': {
                    'accelerometer': True,
                    'gyroscope': True,
                    'magnetometer': True
                }
            },
            {
                'device_id': 'stylus_0',
                'device_type': 'stylus',
                'name': 'Active Stylus Pen',
                'capabilities': {
                    'pressure_sensitivity': 4096,
                    'tilt_sensing': True,
                    'palm_rejection': True
                }
            },
            {
                'device_id': 'controller_0',
                'device_type': 'controller',
                'name': 'Game Controller',
                'capabilities': {
                    'buttons': 16,
                    'axes': 8,
                    'rumble': True
                }
            }
        ])
        
        return discovered
    
    def create_device(self, device_info: Dict[str, any]) -> InputDevice:
        """Create device instance from info"""
        device_type = device_info['device_type']
        
        if device_type not in self.device_factories:
            raise ValueError(f"Unknown device type: {device_type}")
        
        device_class = self.device_factories[device_type]
        device_id = device_info['device_id']
        
        return device_class(device_id)
    
    def add_device(self, device: InputDevice) -> bool:
        """Add device to manager"""
        with self._lock:
            if device.device_id in self.devices:
                self.logger.warning(f"Device {device.device_id} already exists")
                return False
            
            self.devices[device.device_id] = device
            self.logger.info(f"Added device: {device}")
            return True
    
    def remove_device(self, device_id: str) -> bool:
        """Remove device from manager"""
        with self._lock:
            if device_id not in self.devices:
                return False
            
            device = self.devices[device_id]
            device.stop()
            device.disconnect()
            
            del self.devices[device_id]
            del self.device_event_handlers[device_id]
            
            self.logger.info(f"Removed device: {device}")
            return True
    
    def connect_device(self, device_id: str) -> bool:
        """Connect specific device"""
        with self._lock:
            if device_id not in self.devices:
                return False
            
            device = self.devices[device_id]
            
            if not device.connect():
                return False
            
            # Set up event routing
            device.set_event_callback(self._route_device_event)
            
            if device.start():
                self.statistics['devices_connected'] += 1
                return True
            
            return False
    
    def disconnect_device(self, device_id: str) -> bool:
        """Disconnect specific device"""
        with self._lock:
            if device_id not in self.devices:
                return False
            
            device = self.devices[device_id]
            device.stop()
            
            if device.disconnect():
                self.statistics['devices_connected'] -= 1
                return True
            
            return False
    
    def enable_device(self, device_id: str) -> bool:
        """Enable specific device"""
        if device_id not in self.devices:
            return False
        
        return self.devices[device_id].enable()
    
    def disable_device(self, device_id: str) -> bool:
        """Disable specific device"""
        if device_id not in self.devices:
            return False
        
        return self.devices[device_id].disable()
    
    def start(self) -> bool:
        """Start device manager"""
        with self._lock:
            if self.running:
                return True
            
            self.running = True
            
            # Start event processing threads
            for i in range(self.config['event_processing_threads']):
                thread = threading.Thread(target=self._event_processor, 
                                        args=(i,), daemon=True)
                thread.start()
                self.event_processors.append(thread)
            
            # Auto-connect devices if configured
            if self.config['auto_connect_devices']:
                for device in self.devices.values():
                    self.connect_device(device.device_id)
            
            self.logger.info("Device manager started")
            return True
    
    def stop(self):
        """Stop device manager"""
        with self._lock:
            self.running = False
            
            # Stop all devices
            for device in self.devices.values():
                device.stop()
                device.disconnect()
            
            # Wait for event processors
            for thread in self.event_processors:
                thread.join(timeout=2.0)
            
            self._executor.shutdown(wait=False)
            
            self.logger.info("Device manager stopped")
    
    def _route_device_event(self, event: InputEvent):
        """Route event from device to global handlers"""
        self._route_event(event)
    
    def _route_event(self, event: InputEvent):
        """Internal event routing"""
        if not self.running:
            return
        
        # Add to queue
        with self._lock:
            self.event_queue.append(event)
            self.statistics['total_events'] += 1
            self.statistics['events_by_type'][event.event_type.value] += 1
            self.statistics['events_by_device'][event.device_id] += 1
        
        # Submit to processing thread pool
        self._executor.submit(self._process_event, event)
    
    def _process_event(self, event: InputEvent):
        """Process event through routing system"""
        try:
            # Route to event-specific handlers
            if event.event_type in self.event_routers:
                for handler in self.event_routers[event.event_type]:
                    try:
                        handler(event)
                    except Exception as e:
                        self.logger.error(f"Event router error: {e}")
            
            # Route to device-specific handlers
            if event.device_id in self.device_event_handlers:
                for handler in self.device_event_handlers[event.device_id]:
                    try:
                        handler(event)
                    except Exception as e:
                        self.logger.error(f"Device handler error: {e}")
            
            # Route to global handlers
            for handler in self.global_event_handlers:
                try:
                    handler(event)
                except Exception as e:
                    self.logger.error(f"Global handler error: {e}")
        
        except Exception as e:
            self.logger.error(f"Event processing error: {e}")
    
    def _event_processor(self, processor_id: int):
        """Event processing thread"""
        while self.running:
            try:
                # Process statistics updates periodically
                self._update_statistics()
                time.sleep(self.config['statistics_interval'])
            except Exception as e:
                self.logger.error(f"Event processor {processor_id} error: {e}")
                time.sleep(1.0)
    
    def _update_statistics(self):
        """Update manager statistics"""
        with self._lock:
            self.statistics['uptime'] = time.time() - self.statistics['start_time']
            self.statistics['devices_count'] = len(self.devices)
            self.statistics['connected_devices'] = sum(1 for d in self.devices.values() 
                                                      if d.is_connected)
            self.statistics['event_queue_size'] = len(self.event_queue)
    
    def add_event_router(self, event_type: EventType, handler: Callable[[InputEvent], None]):
        """Add event router for specific event type"""
        with self._lock:
            self.event_routers[event_type].append(handler)
    
    def remove_event_router(self, event_type: EventType, handler: Callable[[InputEvent], None]):
        """Remove event router"""
        with self._lock:
            if handler in self.event_routers[event_type]:
                self.event_routers[event_type].remove(handler)
    
    def add_global_event_handler(self, handler: Callable[[InputEvent], None]):
        """Add global event handler"""
        with self._lock:
            self.global_event_handlers.append(handler)
    
    def remove_global_event_handler(self, handler: Callable[[InputEvent], None]):
        """Remove global event handler"""
        with self._lock:
            if handler in self.global_event_handlers:
                self.global_event_handlers.remove(handler)
    
    def add_device_event_handler(self, device_id: str, handler: Callable[[InputEvent], None]):
        """Add event handler for specific device"""
        with self._lock:
            self.device_event_handlers[device_id].append(handler)
    
    def remove_device_event_handler(self, device_id: str, handler: Callable[[InputEvent], None]):
        """Remove device event handler"""
        with self._lock:
            if handler in self.device_event_handlers[device_id]:
                self.device_event_handlers[device_id].remove(handler)
    
    def get_device(self, device_id: str) -> Optional[InputDevice]:
        """Get device by ID"""
        return self.devices.get(device_id)
    
    def get_devices_by_type(self, device_type: str) -> List[InputDevice]:
        """Get all devices of specific type"""
        return [device for device in self.devices.values() 
                if device.device_type == device_type]
    
    def get_all_devices(self) -> List[InputDevice]:
        """Get all registered devices"""
        return list(self.devices.values())
    
    def get_connected_devices(self) -> List[InputDevice]:
        """Get all connected devices"""
        return [device for device in self.devices.values() if device.is_connected]
    
    def get_event_history(self, event_type: Optional[EventType] = None, 
                         device_id: Optional[str] = None,
                         limit: int = 100) -> List[InputEvent]:
        """Get event history with filtering"""
        with self._lock:
            events = list(self.event_queue)
            
            if event_type:
                events = [e for e in events if e.event_type == event_type]
            
            if device_id:
                events = [e for e in events if e.device_id == device_id]
            
            return events[-limit:]
    
    def get_statistics(self) -> Dict[str, any]:
        """Get manager statistics"""
        with self._lock:
            stats = self.statistics.copy()
            
            # Add device-specific statistics
            stats['devices'] = {}
            for device_id, device in self.devices.items():
                stats['devices'][device_id] = device.get_statistics()
            
            return stats
    
    def configure(self, config: Dict[str, any]) -> bool:
        """Configure manager settings"""
        try:
            with self._lock:
                self.config.update(config)
                
                # Update event queue size
                max_size = self.config.get('event_history_size', 10000)
                self.event_queue = deque(maxlen=max_size)
                
                # Update thread pool size
                workers = self.config.get('event_processing_threads', 4)
                self._executor._max_workers = workers
                
                self.logger.info("Manager configuration updated")
                return True
        
        except Exception as e:
            self.logger.error(f"Configuration failed: {e}")
            return False
    
    def export_state(self, filepath: str):
        """Export manager state to file"""
        state = {
            'devices': {},
            'config': self.config,
            'statistics': self.statistics,
            'export_time': time.time()
        }
        
        with self._lock:
            # Export device states
            for device_id, device in self.devices.items():
                state['devices'][device_id] = {
                    'device_type': device.device_type,
                    'is_connected': device.is_connected,
                    'is_enabled': device.is_enabled,
                    'is_calibrated': device.is_calibrated,
                    'config': device.config,
                    'calibration_data': device.calibration_data,
                    'statistics': device.get_statistics()
                }
        
        with open(filepath, 'w') as f:
            json.dump(state, f, indent=2, default=str)
    
    def import_state(self, filepath: str) -> bool:
        """Import manager state from file"""
        try:
            with open(filepath, 'r') as f:
                state = json.load(f)
            
            # Restore configuration
            if 'config' in state:
                self.configure(state['config'])
            
            # Restore device states
            if 'devices' in state:
                for device_id, device_state in state['devices'].items():
                    if device_id in self.devices:
                        device = self.devices[device_id]
                        
                        # Restore basic state
                        device.config.update(device_state.get('config', {}))
                        device.calibration_data.update(device_state.get('calibration_data', {}))
                        
                        # Connect if previously connected
                        if device_state.get('is_connected', False):
                            self.connect_device(device_id)
                        
                        # Enable if previously enabled
                        if device_state.get('is_enabled', False):
                            self.enable_device(device_id)
            
            self.logger.info("Manager state imported")
            return True
        
        except Exception as e:
            self.logger.error(f"State import failed: {e}")
            return False
    
    def __enter__(self):
        """Context manager entry"""
        self.start()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.stop()
    
    def __repr__(self):
        return (f"DeviceManager(devices={len(self.devices)}, "
                f"connected={self.statistics['devices_connected']}, "
                f"events={self.statistics['total_events']})")