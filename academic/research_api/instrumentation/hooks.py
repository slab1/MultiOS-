"""
Instrumentation Hooks and Event Tracking Framework

Provides comprehensive instrumentation capabilities for monitoring
system calls, function calls, memory access, and other system events.
"""

import os
import time
import json
import threading
import ctypes
import subprocess
import inspect
from typing import Dict, List, Any, Optional, Callable, Union, Set
from pathlib import Path
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import logging
import tempfile
import collections
from enum import Enum
import weakref
import traceback

from .config import ResearchConfig


class EventType(Enum):
    """Types of instrumentation events."""
    SYSCALL = "syscall"
    FUNCTION_CALL = "function_call"
    MEMORY_ACCESS = "memory_access"
    FILE_OPERATION = "file_operation"
    NETWORK_OPERATION = "network_operation"
    PROCESS_EVENT = "process_event"
    THREAD_EVENT = "thread_event"
    INTERRUPT = "interrupt"
    EXCEPTION = "exception"
    CUSTOM = "custom"


class HookType(Enum):
    """Types of instrumentation hooks."""
    PRE_CALL = "pre_call"
    POST_CALL = "post_call"
    REPLACE = "replace"
    WRAP = "wrap"
    FILTER = "filter"
    PROXY = "proxy"


@dataclass
class InstrumentationHook:
    """Represents an instrumentation hook."""
    hook_id: str
    name: str
    event_type: EventType
    hook_type: HookType
    target: str  # function, syscall, syscall number, etc.
    handler_function: Callable
    filters: Dict[str, Any]
    enabled: bool = True
    priority: int = 0
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'hook_id': self.hook_id,
            'name': self.name,
            'event_type': self.event_type.value,
            'hook_type': self.hook_type.value,
            'target': self.target,
            'enabled': self.enabled,
            'priority': self.priority,
            'filters': self.filters,
            'metadata': self.metadata
        }


@dataclass
class EventRecord:
    """Represents a captured instrumentation event."""
    event_id: str
    event_type: EventType
    timestamp: datetime
    source: str  # hook or source identifier
    data: Dict[str, Any]
    thread_id: Optional[int] = None
    process_id: Optional[int] = None
    call_stack: Optional[List[str]] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return {
            'event_id': self.event_id,
            'event_type': self.event_type.value,
            'timestamp': self.timestamp.isoformat(),
            'source': self.source,
            'data': self.data,
            'thread_id': self.thread_id,
            'process_id': self.process_id,
            'call_stack': self.call_stack
        }


class EventTracker:
    """
    Event tracking and collection system.
    
    Collects, processes, and stores instrumentation events.
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize event tracker.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Event storage
        self.event_buffer = collections.deque(maxlen=10000)
        self.event_storage = {}
        self.event_counters = collections.defaultdict(int)
        
        # Event filtering and processing
        self.active_filters = {}
        self.event_processors = []
        self.event_aggregators = {}
        
        # Event queue for asynchronous processing
        self.event_queue = collections.deque()
        self.processing_active = False
        self.processing_thread = None
        
        # Statistics
        self.start_time = datetime.now()
        self.total_events = 0
        self.events_per_second = collections.defaultdict(int)
        
        # Initialize tracking capabilities
        self._initialize_tracking()
        
        self.logger.info("Event tracker initialized")
    
    def _initialize_tracking(self):
        """Initialize event tracking capabilities."""
        # Set up basic event types
        self.supported_event_types = set(event_type.value for event_type in EventType)
        
        # Initialize event storage
        self.event_storage = {
            'events_by_type': collections.defaultdict(list),
            'events_by_source': collections.defaultdict(list),
            'events_by_time_window': collections.defaultdict(list)
        }
    
    def start_tracking(self, 
                      event_types: Optional[List[EventType]] = None,
                      buffer_size: int = 10000):
        """
        Start event tracking.
        
        Args:
            event_types: Types of events to track
            buffer_size: Maximum buffer size
        """
        if self.processing_active:
            self.logger.warning("Event tracking already active")
            return
        
        # Configure tracking
        if event_types:
            self.tracked_event_types = set(event_type for event_type in event_types)
        else:
            self.tracked_event_types = set(EventType)  # Track all types
        
        self.event_buffer = collections.deque(maxlen=buffer_size)
        self.processing_active = True
        
        # Start processing thread
        self.processing_thread = threading.Thread(target=self._process_events_loop)
        self.processing_thread.start()
        
        self.logger.info(f"Started tracking {len(self.tracked_event_types)} event types")
    
    def stop_tracking(self) -> Dict[str, Any]:
        """Stop event tracking and return statistics."""
        if not self.processing_active:
            return {'error': 'Event tracking not active'}
        
        self.processing_active = False
        
        if self.processing_thread:
            self.processing_thread.join(timeout=5)
        
        # Generate statistics
        stats = self._generate_tracking_statistics()
        
        self.logger.info("Stopped event tracking")
        return stats
    
    def track_event(self, 
                   event_type: EventType,
                   data: Dict[str, Any],
                   source: str,
                   metadata: Optional[Dict[str, Any]] = None):
        """
        Track an event.
        
        Args:
            event_type: Type of event
            data: Event data
            source: Source identifier
            metadata: Additional metadata
        """
        if not self.processing_active:
            return
        
        # Create event record
        event_record = EventRecord(
            event_id=f"{source}_{int(time.time() * 1000000)}",
            event_type=event_type,
            timestamp=datetime.now(),
            source=source,
            data=data,
            thread_id=threading.get_ident(),
            process_id=os.getpid()
        )
        
        # Add call stack if enabled
        if self.config.instrumentation.event_tracking:
            event_record.call_stack = traceback.format_stack()[-5:]  # Last 5 stack frames
        
        # Add to buffer
        self.event_buffer.append(event_record)
        
        # Add to queue for processing
        self.event_queue.append(event_record)
        
        # Update counters
        self.event_counters[event_type.value] += 1
        self.total_events += 1
        
        # Update events per second
        current_second = int(time.time())
        self.events_per_second[current_second] += 1
    
    def _process_events_loop(self):
        """Main event processing loop."""
        while self.processing_active:
            try:
                # Process events from queue
                batch_size = 100
                batch = []
                
                for _ in range(min(batch_size, len(self.event_queue))):
                    if self.event_queue:
                        batch.append(self.event_queue.popleft())
                
                if batch:
                    self._process_event_batch(batch)
                
                # Sleep briefly to avoid busy waiting
                time.sleep(0.01)
                
            except Exception as e:
                self.logger.error(f"Event processing error: {e}")
    
    def _process_event_batch(self, events: List[EventRecord]):
        """Process a batch of events."""
        for event in events:
            # Store event
            self._store_event(event)
            
            # Apply filters
            if self._should_filter_event(event):
                continue
            
            # Apply processors
            for processor in self.event_processors:
                try:
                    processor(event)
                except Exception as e:
                    self.logger.warning(f"Event processor failed: {e}")
    
    def _store_event(self, event: EventRecord):
        """Store event in various storage structures."""
        # Store by type
        self.event_storage['events_by_type'][event.event_type.value].append(event)
        
        # Store by source
        self.event_storage['events_by_source'][event.source].append(event)
        
        # Store by time window (for efficient time-based queries)
        time_window = int(event.timestamp.timestamp() // 60)  # Minute-level windows
        self.event_storage['events_by_time_window'][time_window].append(event)
    
    def _should_filter_event(self, event: EventRecord) -> bool:
        """Check if event should be filtered out."""
        for filter_name, filter_func in self.active_filters.items():
            try:
                if filter_func(event):
                    return True
            except Exception as e:
                self.logger.warning(f"Filter {filter_name} failed: {e}")
        
        return False
    
    def add_filter(self, name: str, filter_func: Callable[[EventRecord], bool]):
        """
        Add an event filter.
        
        Args:
            name: Filter name
            filter_func: Filter function
        """
        self.active_filters[name] = filter_func
        self.logger.info(f"Added event filter: {name}")
    
    def remove_filter(self, name: str):
        """Remove an event filter."""
        if name in self.active_filters:
            del self.active_filters[name]
            self.logger.info(f"Removed event filter: {name}")
    
    def add_event_processor(self, processor_func: Callable[[EventRecord], None]):
        """
        Add an event processor.
        
        Args:
            processor_func: Processor function
        """
        self.event_processors.append(processor_func)
        self.logger.info(f"Added event processor: {processor_func.__name__}")
    
    def get_events(self, 
                  event_type: Optional[EventType] = None,
                  source: Optional[str] = None,
                  time_range: Optional[tuple] = None,
                  limit: Optional[int] = None) -> List[EventRecord]:
        """
        Retrieve events based on criteria.
        
        Args:
            event_type: Filter by event type
            source: Filter by source
            time_range: Filter by time range (start, end)
            limit: Maximum number of events to return
            
        Returns:
            List of matching events
        """
        events = []
        
        # Base query
        if event_type:
            events.extend(self.event_storage['events_by_type'][event_type.value])
        else:
            for event_list in self.event_storage['events_by_type'].values():
                events.extend(event_list)
        
        # Apply filters
        if source:
            events = [e for e in events if e.source == source]
        
        if time_range:
            start_time, end_time = time_range
            events = [e for e in events if start_time <= e.timestamp <= end_time]
        
        # Sort by timestamp (newest first)
        events.sort(key=lambda x: x.timestamp, reverse=True)
        
        # Apply limit
        if limit:
            events = events[:limit]
        
        return events
    
    def get_event_statistics(self) -> Dict[str, Any]:
        """Get event tracking statistics."""
        # Calculate rates
        runtime = (datetime.now() - self.start_time).total_seconds()
        events_per_second = self.total_events / max(runtime, 1)
        
        # Get recent event rate
        recent_events = sum(self.events_per_second.values())
        recent_rate = recent_events / max(len(self.events_per_second), 1)
        
        return {
            'total_events': self.total_events,
            'runtime_seconds': runtime,
            'events_per_second': events_per_second,
            'recent_rate': recent_rate,
            'buffer_size': len(self.event_buffer),
            'event_type_counts': dict(self.event_counters),
            'sources_active': len(self.event_storage['events_by_source']),
            'time_windows': len(self.event_storage['events_by_time_window'])
        }
    
    def export_events(self, 
                     file_path: str,
                     event_type: Optional[EventType] = None,
                     time_range: Optional[tuple] = None):
        """
        Export events to file.
        
        Args:
            file_path: Path to export file
            event_type: Filter by event type
            time_range: Filter by time range
        """
        events = self.get_events(event_type=event_type, time_range=time_range)
        
        export_data = {
            'exported_at': datetime.now().isoformat(),
            'total_events': len(events),
            'filters': {
                'event_type': event_type.value if event_type else None,
                'time_range': time_range[0].isoformat() if time_range else None
            },
            'events': [event.to_dict() for event in events]
        }
        
        with open(file_path, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Exported {len(events)} events to {file_path}")
    
    def _generate_tracking_statistics(self) -> Dict[str, Any]:
        """Generate comprehensive tracking statistics."""
        runtime = (datetime.now() - self.start_time).total_seconds()
        
        # Event type distribution
        event_type_distribution = dict(self.event_counters)
        
        # Source distribution
        source_distribution = {}
        for source, events in self.event_storage['events_by_source'].items():
            source_distribution[source] = len(events)
        
        # Performance metrics
        performance = {
            'total_processing_time': 0,  # Would be calculated from actual processing times
            'average_events_per_second': self.total_events / max(runtime, 1),
            'peak_events_per_second': max(self.events_per_second.values()) if self.events_per_second else 0,
            'memory_usage': len(self.event_buffer) * 1024  # Rough estimate
        }
        
        return {
            'tracking_duration': runtime,
            'total_events': self.total_events,
            'event_type_distribution': event_type_distribution,
            'source_distribution': source_distribution,
            'performance_metrics': performance,
            'active_filters': list(self.active_filters.keys()),
            'active_processors': len(self.event_processors)
        }


class InstrumentationHooks:
    """
    Comprehensive instrumentation hooks system.
    
    Provides hooks for:
    - System calls
    - Function calls
    - Memory access
    - File operations
    - Network operations
    """
    
    def __init__(self, config: ResearchConfig):
        """
        Initialize instrumentation hooks.
        
        Args:
            config: Research configuration
        """
        self.config = config
        self.logger = logging.getLogger(__name__)
        
        # Hook registry
        self.hooks = {}
        self.active_hooks = {}
        self.hook_priorities = {}
        
        # Event tracking
        self.event_tracker = EventTracker(config)
        
        # Hook execution context
        self.execution_context = {}
        self.call_stack = collections.deque(maxlen=1000)
        
        # Performance monitoring
        self.hook_performance = collections.defaultdict(list)
        self.performance_monitoring_enabled = True
        
        # Initialize hook capabilities
        self._initialize_hooks()
        
        self.logger.info("Instrumentation hooks system initialized")
    
    def _initialize_hooks(self):
        """Initialize hook capabilities."""
        # Set up supported hook types
        self.supported_hook_types = set(hook_type for hook_type in HookType)
        self.supported_event_types = set(event_type for event_type in EventType)
        
        # Initialize hook storage
        self.hooks = {
            'syscall_hooks': {},
            'function_hooks': {},
            'memory_hooks': {},
            'file_hooks': {},
            'network_hooks': {},
            'custom_hooks': {}
        }
    
    def setup_system_monitoring(self) -> bool:
        """
        Setup comprehensive system monitoring.
        
        Returns:
            True if successful, False otherwise
        """
        try:
            # Start event tracking
            self.event_tracker.start_tracking()
            
            # Setup basic hooks
            self._setup_basic_system_hooks()
            
            # Setup performance monitoring
            if self.performance_monitoring_enabled:
                self._setup_performance_monitoring()
            
            self.logger.info("System monitoring setup completed")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to setup system monitoring: {e}")
            return False
    
    def _setup_basic_system_hooks(self):
        """Setup basic system hooks."""
        # File operation hooks
        self.install_function_hook(
            hook_name="file_open_hook",
            module="builtins",
            function="open",
            hook_type=HookType.WRAP,
            handler_function=self._file_open_handler
        )
        
        # Process creation hooks (simplified)
        self.install_function_hook(
            hook_name="process_create_hook",
            module="os",
            function="spawnve",
            hook_type=HookType.PRE_CALL,
            handler_function=self._process_create_handler
        )
    
    def _setup_performance_monitoring(self):
        """Setup performance monitoring hooks."""
        # Hook into time functions to measure execution time
        self.install_function_hook(
            hook_name="time_measure_hook",
            module="time",
            function="time",
            hook_type=HookType.WRAP,
            handler_function=self._time_measure_handler
        )
        
        # Add performance tracking processor
        self.event_tracker.add_event_processor(self._track_performance_metrics)
    
    def install_syscall_hook(self, 
                           syscall_name: str,
                           hook_type: HookType,
                           handler_function: Callable,
                           parameters: Optional[Dict[str, Any]] = None) -> str:
        """
        Install a system call hook.
        
        Args:
            syscall_name: Name of system call to hook
            hook_type: Type of hook
            handler_function: Handler function
            parameters: Additional parameters
            
        Returns:
            Hook ID
        """
        hook_id = f"syscall_{syscall_name}_{int(time.time())}"
        
        hook = InstrumentationHook(
            hook_id=hook_id,
            name=f"Syscall hook for {syscall_name}",
            event_type=EventType.SYSCALL,
            hook_type=hook_type,
            target=syscall_name,
            handler_function=handler_function,
            filters=parameters or {}
        )
        
        self.hooks['syscall_hooks'][hook_id] = hook
        self.active_hooks[hook_id] = hook
        
        # Initialize hook execution context
        if syscall_name not in self.execution_context:
            self.execution_context[syscall_name] = {
                'call_count': 0,
                'total_time': 0,
                'errors': 0
            }
        
        self.logger.info(f"Installed syscall hook: {hook_id}")
        return hook_id
    
    def install_function_hook(self,
                            hook_name: str,
                            module: str,
                            function: str,
                            hook_type: HookType,
                            handler_function: Callable,
                            parameters: Optional[Dict[str, Any]] = None) -> str:
        """
        Install a function hook.
        
        Args:
            hook_name: Name of the hook
            module: Module containing function
            function: Function name to hook
            hook_type: Type of hook
            handler_function: Handler function
            parameters: Additional parameters
            
        Returns:
            Hook ID
        """
        hook_id = f"func_{module}_{function}_{int(time.time())}"
        
        hook = InstrumentationHook(
            hook_id=hook_id,
            name=hook_name,
            event_type=EventType.FUNCTION_CALL,
            hook_type=hook_type,
            target=f"{module}.{function}",
            handler_function=handler_function,
            filters=parameters or {}
        )
        
        self.hooks['function_hooks'][hook_id] = hook
        self.active_hooks[hook_id] = hook
        
        self.logger.info(f"Installed function hook: {hook_id}")
        return hook_id
    
    def install_memory_hook(self,
                          hook_name: str,
                          target_type: str,  # 'malloc', 'free', 'read', 'write'
                          hook_type: HookType,
                          handler_function: Callable,
                          parameters: Optional[Dict[str, Any]] = None) -> str:
        """
        Install a memory access hook.
        
        Args:
            hook_name: Name of the hook
            target_type: Type of memory operation
            hook_type: Type of hook
            handler_function: Handler function
            parameters: Additional parameters
            
        Returns:
            Hook ID
        """
        hook_id = f"mem_{target_type}_{int(time.time())}"
        
        hook = InstrumentationHook(
            hook_id=hook_id,
            name=hook_name,
            event_type=EventType.MEMORY_ACCESS,
            hook_type=hook_type,
            target=target_type,
            handler_function=handler_function,
            filters=parameters or {}
        )
        
        self.hooks['memory_hooks'][hook_id] = hook
        self.active_hooks[hook_id] = hook
        
        self.logger.info(f"Installed memory hook: {hook_id}")
        return hook_id
    
    def install_network_hook(self,
                           hook_name: str,
                           operation: str,  # 'connect', 'send', 'recv', 'bind'
                           hook_type: HookType,
                           handler_function: Callable,
                           parameters: Optional[Dict[str, Any]] = None) -> str:
        """
        Install a network operation hook.
        
        Args:
            hook_name: Name of the hook
            operation: Network operation to hook
            hook_type: Type of hook
            handler_function: Handler function
            parameters: Additional parameters
            
        Returns:
            Hook ID
        """
        hook_id = f"net_{operation}_{int(time.time())}"
        
        hook = InstrumentationHook(
            hook_id=hook_id,
            name=hook_name,
            event_type=EventType.NETWORK_OPERATION,
            hook_type=hook_type,
            target=operation,
            handler_function=handler_function,
            filters=parameters or {}
        )
        
        self.hooks['network_hooks'][hook_id] = hook
        self.active_hooks[hook_id] = hook
        
        self.logger.info(f"Installed network hook: {hook_id}")
        return hook_id
    
    def enable_hook(self, hook_id: str) -> bool:
        """
        Enable a hook.
        
        Args:
            hook_id: Hook ID to enable
            
        Returns:
            True if successful, False otherwise
        """
        # Find hook in any category
        for category, hooks in self.hooks.items():
            if hook_id in hooks:
                hooks[hook_id].enabled = True
                self.active_hooks[hook_id] = hooks[hook_id]
                self.logger.info(f"Enabled hook: {hook_id}")
                return True
        
        self.logger.error(f"Hook not found: {hook_id}")
        return False
    
    def disable_hook(self, hook_id: str) -> bool:
        """
        Disable a hook.
        
        Args:
            hook_id: Hook ID to disable
            
        Returns:
            True if successful, False otherwise
        """
        if hook_id in self.active_hooks:
            self.active_hooks[hook_id].enabled = False
            del self.active_hooks[hook_id]
            self.logger.info(f"Disabled hook: {hook_id}")
            return True
        
        self.logger.error(f"Hook not found: {hook_id}")
        return False
    
    def remove_hook(self, hook_id: str) -> bool:
        """
        Remove a hook completely.
        
        Args:
            hook_id: Hook ID to remove
            
        Returns:
            True if successful, False otherwise
        """
        # Remove from all categories
        for category, hooks in self.hooks.items():
            if hook_id in hooks:
                del hooks[hook_id]
        
        # Remove from active hooks
        if hook_id in self.active_hooks:
            del self.active_hooks[hook_id]
        
        self.logger.info(f"Removed hook: {hook_id}")
        return True
    
    def get_hook_status(self, hook_id: str) -> Optional[Dict[str, Any]]:
        """Get status of a specific hook."""
        # Find hook
        for category, hooks in self.hooks.items():
            if hook_id in hooks:
                return hooks[hook_id].to_dict()
        
        return None
    
    def list_active_hooks(self) -> List[Dict[str, Any]]:
        """List all active hooks."""
        return [hook.to_dict() for hook in self.active_hooks.values()]
    
    def _file_open_handler(self, original_func: Callable, *args, **kwargs) -> Any:
        """Handler for file open operations."""
        hook_start_time = time.time()
        
        try:
            # Track the call
            if len(args) > 0:
                filename = args[0]
                mode = args[1] if len(args) > 1 else kwargs.get('mode', 'r')
            else:
                filename = kwargs.get('file', 'unknown')
                mode = kwargs.get('mode', 'r')
            
            # Log the event
            self.event_tracker.track_event(
                event_type=EventType.FILE_OPERATION,
                data={
                    'operation': 'open',
                    'filename': str(filename),
                    'mode': mode,
                    'args': str(args)[:200],  # Limit size
                    'kwargs': str(kwargs)[:200]
                },
                source='file_open_hook'
            )
            
            # Call original function
            result = original_func(*args, **kwargs)
            
            # Track success
            execution_time = time.time() - hook_start_time
            self.hook_performance['file_open'].append(execution_time)
            
            return result
            
        except Exception as e:
            # Track error
            self.event_tracker.track_event(
                event_type=EventType.EXCEPTION,
                data={
                    'operation': 'file_open',
                    'error': str(e),
                    'args': str(args)[:200]
                },
                source='file_open_hook'
            )
            raise
    
    def _process_create_handler(self, *args, **kwargs):
        """Handler for process creation."""
        # Track process creation
        self.event_tracker.track_event(
            event_type=EventType.PROCESS_EVENT,
            data={
                'operation': 'create',
                'args': str(args)[:200],
                'kwargs': str(kwargs)[:200]
            },
            source='process_create_hook'
        )
    
    def _time_measure_handler(self, original_func: Callable, *args, **kwargs) -> Any:
        """Handler for performance measurement."""
        start_time = time.time()
        
        try:
            result = original_func(*args, **kwargs)
            execution_time = time.time() - start_time
            
            self.event_tracker.track_event(
                event_type=EventType.CUSTOM,
                data={
                    'operation': 'performance_measure',
                    'function': original_func.__name__,
                    'execution_time': execution_time
                },
                source='time_measure_hook'
            )
            
            return result
            
        except Exception as e:
            execution_time = time.time() - start_time
            self.event_tracker.track_event(
                event_type=EventType.EXCEPTION,
                data={
                    'operation': 'performance_measure',
                    'function': original_func.__name__,
                    'error': str(e),
                    'execution_time': execution_time
                },
                source='time_measure_hook'
            )
            raise
    
    def _track_performance_metrics(self, event: EventRecord):
        """Track performance metrics from events."""
        if event.event_type == EventType.CUSTOM and 'execution_time' in event.data:
            execution_time = event.data['execution_time']
            function_name = event.data.get('function', 'unknown')
            
            # Store in performance metrics
            key = f"hook_performance_{function_name}"
            if key not in self.hook_performance:
                self.hook_performance[key] = []
            
            self.hook_performance[key].append(execution_time)
            
            # Keep only recent measurements
            if len(self.hook_performance[key]) > 1000:
                self.hook_performance[key] = self.hook_performance[key][-1000:]
    
    def execute_hook(self, 
                    hook_id: str,
                    original_func: Callable,
                    args: tuple,
                    kwargs: dict) -> Any:
        """
        Execute a hook and return the result.
        
        Args:
            hook_id: Hook ID to execute
            original_func: Original function being hooked
            args: Function arguments
            kwargs: Function keyword arguments
            
        Returns:
            Function result
        """
        if hook_id not in self.active_hooks:
            raise ValueError(f"Hook {hook_id} not active")
        
        hook = self.active_hooks[hook_id]
        
        # Add to call stack
        self.call_stack.append({
            'hook_id': hook_id,
            'target': hook.target,
            'timestamp': datetime.now(),
            'args_preview': str(args)[:100]  # Limit preview size
        })
        
        try:
            # Execute hook based on type
            if hook.hook_type == HookType.PRE_CALL:
                # Execute pre-call handler
                handler_result = hook.handler_function(*args, **kwargs)
                
                # Call original function
                result = original_func(*args, **kwargs)
                
                return result
                
            elif hook.hook_type == HookType.POST_CALL:
                # Call original function
                result = original_func(*args, **kwargs)
                
                # Execute post-call handler
                handler_result = hook.handler_function(result, *args, **kwargs)
                
                return result
                
            elif hook.hook_type == HookType.WRAP:
                # Wrap original function
                return hook.handler_function(original_func, *args, **kwargs)
                
            elif hook.hook_type == HookType.REPLACE:
                # Replace original function
                return hook.handler_function(*args, **kwargs)
                
            else:
                # Default behavior
                return original_func(*args, **kwargs)
                
        finally:
            # Remove from call stack
            if self.call_stack:
                self.call_stack.pop()
    
    def get_instrumentation_summary(self) -> Dict[str, Any]:
        """Get comprehensive instrumentation summary."""
        # Hook statistics
        hook_stats = {}
        for category, hooks in self.hooks.items():
            hook_stats[category] = {
                'total': len(hooks),
                'active': len([h for h in hooks.values() if h.enabled])
            }
        
        # Performance statistics
        performance_stats = {}
        for func_name, times in self.hook_performance.items():
            if times:
                performance_stats[func_name] = {
                    'calls': len(times),
                    'avg_time': sum(times) / len(times),
                    'min_time': min(times),
                    'max_time': max(times),
                    'total_time': sum(times)
                }
        
        # Event tracking statistics
        event_stats = self.event_tracker.get_event_statistics()
        
        return {
            'instrumentation_active': len(self.active_hooks) > 0,
            'hook_statistics': hook_stats,
            'performance_statistics': performance_stats,
            'event_statistics': event_stats,
            'call_stack_depth': len(self.call_stack),
            'configuration': {
                'performance_monitoring': self.performance_monitoring_enabled,
                'event_tracking': self.config.instrumentation.event_tracking,
                'hooks_enabled': self.config.instrumentation.hooks_enabled
            }
        }
    
    def cleanup(self):
        """Cleanup instrumentation system."""
        self.logger.info("Cleaning up instrumentation system")
        
        # Stop event tracking
        if hasattr(self, 'event_tracker'):
            self.event_tracker.stop_tracking()
        
        # Disable all hooks
        self.active_hooks.clear()
        
        # Clear performance data
        self.hook_performance.clear()
        
        # Clear call stack
        self.call_stack.clear()
        
        self.logger.info("Instrumentation cleanup completed")


# Add missing module imports
import platform
import collections