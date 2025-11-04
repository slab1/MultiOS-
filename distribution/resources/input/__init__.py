"""
MultiOS Input Device Support System

This module provides comprehensive support for modern input devices
including touch screens, voice input, motion sensors, and interactive devices.
"""

from .core.device_manager import DeviceManager
from .core.input_device import InputDevice
from .core.input_event import InputEvent, EventType, EventPriority

__version__ = "1.0.0"
__all__ = [
    'DeviceManager',
    'InputDevice', 
    'InputEvent',
    'EventType',
    'EventPriority'
]