"""
Base Input Device Interface

Abstract base class for all input devices in MultiOS.
Provides common functionality for device management, event handling, and configuration.
"""

from abc import ABC, abstractmethod
from typing import Dict, List, Optional, Callable, Any
from collections import deque
import threading
import time
import logging

from .input_event import InputEvent, EventType, EventPriority


class DeviceCapabilities(ABC):
    """Device capabilities descriptor"""
    
    def __init__(self):
        self.supported_events: set[EventType] = set()
        self.max_touch_points: int = 0
        self.pressure_sensitive: bool = False
        self.tilt_sensing: bool = False
        self.gesture_support: bool = False
        self.calibration_required: bool = False
        self.sampling_rate: int = 0  # Hz
        self.accuracy: float = 0.0
        self.resolution_x: int = 0
        self.resolution_y: int = 0
        self.physical_size: Optional[tuple[float, float]] = None  # mm


class InputDevice(ABC):
    """Base input device class"""
    
    def __init__(self, device_id: str, device_type: str):
        self.device_id = device_id
        self.device_type = device_type
        self.is_connected: bool = False
        self.is_enabled: bool = True
        self.is_calibrated: bool = False
        self.last_activity: float = 0.0
        
        # Event handling
        self.event_queue: deque = deque(maxlen=1000)
        self.event_handlers: Dict[EventType, List[Callable]] = {}
        self.event_callback: Optional[Callable[[InputEvent], None]] = None
        self.filter_callback: Optional[Callable[[InputEvent], bool]] = None
        
        # Configuration
        self.config: Dict[str, Any] = {}
        self.calibration_data: Dict[str, Any] = {}
        
        # Threading
        self._lock = threading.RLock()
        self._running = False
        self._worker_thread: Optional[threading.Thread] = None
        
        # Logging
        self.logger = logging.getLogger(f"input.device.{device_type}.{device_id}")
        self.logger.setLevel(logging.INFO)
        
        # Initialize device
        self._initialize_device()
    
    def _initialize_device(self):
        """Initialize device-specific settings"""
        # Override in subclasses
        pass
    
    @abstractmethod
    def get_capabilities(self) -> DeviceCapabilities:
        """Get device capabilities"""
        pass
    
    @abstractmethod
    def connect(self) -> bool:
        """Connect to device"""
        pass
    
    @abstractmethod
    def disconnect(self) -> bool:
        """Disconnect from device"""
        pass
    
    @abstractmethod
    def start_polling(self):
        """Start device polling/reading"""
        pass
    
    @abstractmethod
    def stop_polling(self):
        """Stop device polling/reading"""
        pass
    
    @abstractmethod
    def calibrate(self) -> bool:
        """Calibrate device"""
        pass
    
    def configure(self, config: Dict[str, Any]) -> bool:
        """Configure device settings"""
        try:
            with self._lock:
                self.config.update(config)
                return self._apply_config()
        except Exception as e:
            self.logger.error(f"Configuration failed: {e}")
            return False
    
    def _apply_config(self) -> bool:
        """Apply configuration to device hardware"""
        # Override in subclasses for specific config
        return True
    
    def enable(self) -> bool:
        """Enable device"""
        with self._lock:
            if not self.is_connected:
                if not self.connect():
                    return False
            
            self.is_enabled = True
            self._start_worker()
            return True
    
    def disable(self) -> bool:
        """Disable device"""
        with self._lock:
            self.is_enabled = False
            self._stop_worker()
            return True
    
    def start(self) -> bool:
        """Start device operations"""
        if not self.enable():
            return False
        
        with self._lock:
            self._running = True
            self.last_activity = time.time()
            return True
    
    def stop(self) -> bool:
        """Stop device operations"""
        with self._lock:
            self._running = False
            self._stop_worker()
            return True
    
    def _start_worker(self):
        """Start worker thread"""
        if self._worker_thread is None or not self._worker_thread.is_alive():
            self._worker_thread = threading.Thread(target=self._worker_loop, daemon=True)
            self._worker_thread.start()
    
    def _stop_worker(self):
        """Stop worker thread"""
        if self._worker_thread and self._worker_thread.is_alive():
            # Note: In a real implementation, you'd need proper thread stopping
            pass
    
    def _worker_loop(self):
        """Main worker loop for device polling"""
        while self._running and self.is_enabled:
            try:
                self._poll_device()
                time.sleep(0.001)  # 1ms polling interval
            except Exception as e:
                self.logger.error(f"Worker loop error: {e}")
                time.sleep(0.1)
    
    def _poll_device(self):
        """Poll device for new data"""
        # Override in subclasses to read device data
        pass
    
    def set_event_handler(self, event_type: EventType, handler: Callable[[InputEvent], None]):
        """Set event handler for specific event type"""
        if event_type not in self.event_handlers:
            self.event_handlers[event_type] = []
        self.event_handlers[event_type].append(handler)
    
    def set_event_callback(self, callback: Callable[[InputEvent], None]):
        """Set general event callback"""
        self.event_callback = callback
    
    def set_filter_callback(self, filter_callback: Callable[[InputEvent], bool]):
        """Set event filter callback"""
        self.filter_callback = filter_callback
    
    def _send_event(self, event: InputEvent):
        """Send input event to handlers"""
        if not self.is_enabled or not self.is_connected:
            return
        
        # Apply filter if set
        if self.filter_callback and not self.filter_callback(event):
            return
        
        # Add to queue
        with self._lock:
            self.event_queue.append(event)
            self.last_activity = time.time()
        
        # Call specific handlers
        if event.event_type in self.event_handlers:
            for handler in self.event_handlers[event.event_type]:
                try:
                    handler(event)
                except Exception as e:
                    self.logger.error(f"Event handler error: {e}")
        
        # Call general callback
        if self.event_callback:
            try:
                self.event_callback(event)
            except Exception as e:
                self.logger.error(f"Event callback error: {e}")
    
    def get_recent_events(self, count: int = 10) -> List[InputEvent]:
        """Get recent events from queue"""
        with self._lock:
            return list(self.event_queue)[-count:]
    
    def get_event_history(self, event_type: Optional[EventType] = None) -> List[InputEvent]:
        """Get event history filtered by type"""
        with self._lock:
            events = list(self.event_queue)
            if event_type:
                events = [e for e in events if e.event_type == event_type]
            return events
    
    def clear_events(self):
        """Clear event queue"""
        with self._lock:
            self.event_queue.clear()
    
    def get_statistics(self) -> Dict[str, Any]:
        """Get device statistics"""
        with self._lock:
            events = list(self.event_queue)
            
            # Count events by type
            event_counts = {}
            for event in events:
                event_type = event.event_type.value
                event_counts[event_type] = event_counts.get(event_type, 0) + 1
            
            # Calculate event rate
            time_span = 1.0
            if len(events) >= 2:
                time_span = events[-1].timestamp - events[0].timestamp
            
            return {
                'device_id': self.device_id,
                'device_type': self.device_type,
                'is_connected': self.is_connected,
                'is_enabled': self.is_enabled,
                'is_calibrated': self.is_calibrated,
                'total_events': len(events),
                'event_rate': len(events) / max(time_span, 1.0),
                'event_counts': event_counts,
                'last_activity': self.last_activity,
                'queue_size': len(self.event_queue),
                'config': self.config,
                'uptime': time.time() - (events[0].timestamp if events else time.time())
            }
    
    def reset(self):
        """Reset device state"""
        with self._lock:
            self.clear_events()
            self.is_calibrated = False
            self.last_activity = time.time()
            self._reset_device_hardware()
    
    def _reset_device_hardware(self):
        """Reset device hardware"""
        # Override in subclasses
        pass
    
    def __enter__(self):
        """Context manager entry"""
        self.start()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.stop()
        self.disconnect()
    
    def __str__(self) -> str:
        return f"{self.device_type}({self.device_id})"
    
    def __repr__(self) -> str:
        return f"{self.__class__.__name__}(id='{self.device_id}', type='{self.device_type}')"