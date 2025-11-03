"""
OS Instrumentation Module

Provides comprehensive instrumentation capabilities for OS modification and monitoring.
"""

from .hooks import OSInstrumentor, CustomModifier

__all__ = ["OSInstrumentor", "CustomModifier"]