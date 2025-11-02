//! Event Handling System for IPC
//! 
//! This module implements event objects for process coordination,
//! synchronization, and notification across processes and threads.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::{Mutex, RwLock};
use bitflags::bitflags;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::{IpcResult, IpcError};

/// Event ID type
pub type EventId = u32;

/// Maximum number of events
const MAX_EVENTS: usize = 1024;

/// Event handle for user-space access
#[derive(Debug, Clone, Copy)]
pub struct EventHandle {
    pub id: EventId,
}

impl EventHandle {
    pub const fn new(id: EventId) -> Self {
        Self { id }
    }
}

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    ManualReset = 0,
    AutoReset = 1,
    Notification = 2,
    Synchronization = 3,
}

/// Event flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventFlags: u32 {
        const SIGNALLED     = 1 << 0;
        const BROADCAST     = 1 << 1;
        const PRIORITY      = 1 << 2;
        const TIMEOUT       = 1 << 3;
        const RECURSIVE     = 1 << 4;
        const INTERPROCESS  = 1 << 5;
        const PRIVATE       = 1 << 6;
    }
}

/// Waiting process information
#[derive(Debug, Clone)]
pub struct EventWaiter {
    pub process_id: u32,
    pub thread_id: u32,
    pub timeout_ns: Option<u64>,
    pub priority: u32,
    pub wait_start_time: u64,
}

/// Event data payload
#[derive(Debug, Clone)]
pub struct EventData {
    pub data: Vec<u8>,
    pub data_type: u32,
    pub sender_id: u32,
    pub timestamp: u64,
}

/// Event statistics
#[derive(Debug, Clone, Default)]
pub struct EventStatistics {
    pub wait_operations: u64,
    pub signal_operations: u64,
    pub broadcast_operations: u64,
    pub timeout_operations: u64,
    pub active_waiters: u32,
    pub max_waiters: u32,
    pub signals_sent: u64,
    pub signals_received: u64,
    pub errors: u32,
}

/// Event object implementation
#[derive(Debug)]
pub struct Event {
    pub id: EventId,
    pub name: Option<Vec<u8>>,
    pub event_type: EventType,
    pub flags: EventFlags,
    pub created_by: u32,
    pub created_at: u64,
    pub signalled: AtomicU32,
    pub waiting_processes: RwLock<Vec<EventWaiter>>,
    pub event_data: Option<EventData>,
    pub reference_count: AtomicU32,
    pub statistics: EventStatistics,
}

impl Event {
    pub fn new(id: EventId, event_type: EventType, flags: EventFlags) -> Self {
        Self {
            id,
            name: None,
            event_type,
            flags,
            created_by: 0,
            created_at: 0,
            signalled: AtomicU32::new(0),
            waiting_processes: RwLock::new(Vec::new()),
            event_data: None,
            reference_count: AtomicU32::new(1),
            statistics: EventStatistics::default(),
        }
    }

    /// Wait for event to be signalled
    pub fn wait(&self, process_id: u32, thread_id: u32, timeout_ns: Option<u64>) -> IpcResult<()> {
        self.statistics.wait_operations += 1;

        // Check if event is already signalled
        if self.signalled.load(Ordering::SeqCst) == 1 {
            if self.event_type == EventType::AutoReset {
                // Auto-reset event: reset immediately
                self.signalled.store(0, Ordering::SeqCst);
            }
            log::debug!("Process {} immediately acquired event {}", process_id, self.id);
            return Ok(());
        }

        // Add process to wait list
        let mut waiters = self.waiting_processes.write();
        let waiter = EventWaiter {
            process_id,
            thread_id,
            timeout_ns,
            priority: 0,
            wait_start_time: 0, // Will be set by caller
        };
        waiters.push(waiter);
        self.statistics.active_waiters = waiters.len() as u32;
        
        // Update max waiters
        if waiters.len() as u32 > self.statistics.max_waiters {
            self.statistics.max_waiters = waiters.len() as u32;
        }

        drop(waiters);

        // In real implementation, this would block the process/thread
        log::debug!("Process {} waiting on event {}", process_id, self.id);
        Ok(())
    }

    /// Signal the event
    pub fn signal(&self, process_id: u32, data: Option<EventData>) -> IpcResult<usize> {
        self.statistics.signal_operations += 1;
        self.statistics.signals_sent += 1;

        // Store event data
        if let Some(data) = data {
            self.event_data = Some(data);
        }

        // Set signalled flag
        self.signalled.store(1, Ordering::SeqCst);
        self.flags.insert(EventFlags::SIGNALLED);

        // Wake up waiting processes
        let woken_processes = self.wake_up_waiting_processes();

        log::debug!("Process {} signalled event {} (woke {} processes)", process_id, self.id, woken_processes);
        Ok(woken_processes)
    }

    /// Broadcast signal to all waiters
    pub fn broadcast(&self, process_id: u32, data: Option<EventData>) -> IpcResult<usize> {
        self.statistics.broadcast_operations += 1;
        self.statistics.signals_sent += 1;

        // Store event data
        if let Some(data) = data {
            self.event_data = Some(data);
        }

        // Set signalled flag
        self.signalled.store(1, Ordering::SeqCst);
        self.flags.insert(EventFlags::SIGNALLED);

        // Wake up all waiting processes
        let waiters = self.waiting_processes.read();
        let woken_count = waiters.len();
        drop(waiters);

        let mut waiters = self.waiting_processes.write();
        waiters.clear();
        self.statistics.active_waiters = 0;

        // Reset if manual reset event
        if self.event_type == EventType::ManualReset {
            self.signalled.store(0, Ordering::SeqCst);
        }

        log::debug!("Process {} broadcast event {} (woke {} processes)", process_id, self.id, woken_count);
        Ok(woken_count)
    }

    /// Reset the event
    pub fn reset(&self, process_id: u32) -> IpcResult<()> {
        // Only the creator or a process with appropriate permissions can reset
        if self.created_by != process_id {
            // In real implementation, check process permissions
        }

        self.signalled.store(0, Ordering::SeqCst);
        self.flags.remove(EventFlags::SIGNALLED);
        
        log::debug!("Process {} reset event {}", process_id, self.id);
        Ok(())
    }

    /// Pulse the event (signal then reset immediately)
    pub fn pulse(&self, process_id: u32, data: Option<EventData>) -> IpcResult<usize> {
        self.statistics.signal_operations += 1;
        self.statistics.signals_sent += 1;

        // Store event data
        if let Some(data) = data {
            self.event_data = Some(data);
        }

        // Wake up waiters
        let woken_count = self.wake_up_waiting_processes();

        // Immediately reset for pulse operation
        self.signalled.store(0, Ordering::SeqCst);
        self.flags.remove(EventFlags::SIGNALLED);

        log::debug!("Process {} pulsed event {} (woke {} processes)", process_id, self.id, woken_count);
        Ok(woken_count)
    }

    /// Try to wait (non-blocking)
    pub fn try_wait(&self, process_id: u32, thread_id: u32) -> IpcResult<()> {
        if self.signalled.load(Ordering::SeqCst) == 1 {
            if self.event_type == EventType::AutoReset {
                self.signalled.store(0, Ordering::SeqCst);
            }
            log::debug!("Process {} acquired event {} with try_wait", process_id, self.id);
            Ok(())
        } else {
            Err(IpcError::WouldBlock)
        }
    }

    /// Get current event state
    pub fn get_state(&self) -> EventState {
        EventState {
            is_signalled: self.signalled.load(Ordering::SeqCst) == 1,
            waiters_count: self.waiting_processes.read().len(),
            event_type: self.event_type,
            reference_count: self.reference_count.load(Ordering::SeqCst),
        }
    }

    /// Get event data
    pub fn get_data(&self) -> Option<EventData> {
        self.event_data.clone()
    }

    /// Check if event is signalled
    pub fn is_signalled(&self) -> bool {
        self.signalled.load(Ordering::SeqCst) == 1
    }

    /// Get waiting process count
    pub fn waiting_count(&self) -> usize {
        self.waiting_processes.read().len()
    }

    /// Add reference (for shared access)
    pub fn add_reference(&self) {
        self.reference_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Remove reference (decrement)
    pub fn remove_reference(&self) -> bool {
        let count = self.reference_count.fetch_sub(1, Ordering::SeqCst);
        count <= 1 // Should be freed if count becomes 0
    }

    /// Wake up waiting processes
    fn wake_up_waiting_processes(&self) -> usize {
        let mut waiters = self.waiting_processes.write();
        
        if waiters.is_empty() {
            return 0;
        }

        let woken_count = if self.event_type == EventType::AutoReset {
            // Auto-reset: wake only one waiter
            let waiter = waiters.remove(0);
            log::debug!("Waking up process {} waiting on event {}", waiter.process_id, self.id);
            1
        } else {
            // Manual reset: wake all waiters
            let count = waiters.len();
            waiters.clear();
            
            log::debug!("Waking up {} processes waiting on event {}", count, self.id);
            count
        };

        self.statistics.active_waiters = waiters.len() as u32;
        woken_count
    }

    /// Set event name
    pub fn set_name(&mut self, name: &[u8]) {
        self.name = Some(name.to_vec());
    }

    /// Get event name
    pub fn get_name(&self) -> Option<&[u8]> {
        self.name.as_ref().map(|v| v.as_slice())
    }

    /// Get statistics
    pub fn get_statistics(&self) -> EventStatistics {
        self.statistics.clone()
    }
}

/// Event state information
#[derive(Debug, Clone)]
pub struct EventState {
    pub is_signalled: bool,
    pub waiters_count: usize,
    pub event_type: EventType,
    pub reference_count: u32,
}

/// Event notification structure
#[derive(Debug, Clone)]
pub struct EventNotification {
    pub event_id: EventId,
    pub notification_type: NotificationType,
    pub process_id: u32,
    pub timestamp: u64,
    pub data: Option<EventData>,
}

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NotificationType {
    EventSignalled = 0,
    EventReset = 1,
    WaitCompleted = 2,
    WaitTimeout = 3,
    BroadcastSent = 4,
}

/// Event manager for handling multiple events
#[derive(Debug)]
pub struct EventManager {
    pub events: RwLock<Vec<Event>>,
    pub named_events: RwLock<Vec<Event>>,
    pub next_id: AtomicU32,
    pub notifications: Mutex<Vec<EventNotification>>,
    pub global_statistics: EventStatistics,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            named_events: RwLock::new(Vec::new()),
            next_id: AtomicU32::new(1),
            notifications: Mutex::new(Vec::new()),
            global_statistics: EventStatistics::default(),
        }
    }

    /// Create an unnamed event
    pub fn create_event(&self, event_type: EventType, flags: EventFlags) -> IpcResult<Event> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let event = Event::new(id, event_type, flags);
        
        let mut events = self.events.write();
        events.push(event);
        let created_event = events.last().unwrap().clone();
        
        Ok(created_event)
    }

    /// Create a named event
    pub fn create_named_event(&self, name: &[u8], event_type: EventType, flags: EventFlags) -> IpcResult<Event> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut event = Event::new(id, event_type, flags);
        event.set_name(name);
        
        let mut named_events = self.named_events.write();
        named_events.push(event);
        let created_event = named_events.last().unwrap().clone();
        
        Ok(created_event)
    }

    /// Open an existing named event
    pub fn open_named_event(&self, name: &[u8]) -> IpcResult<Event> {
        let named_events = self.named_events.read();
        
        for event in named_events.iter() {
            if let Some(event_name) = event.get_name() {
                if event_name == name {
                    event.add_reference();
                    return Ok(event.clone());
                }
            }
        }
        
        Err(IpcError::InvalidHandle)
    }

    /// Get event by ID
    pub fn get_event(&self, event_id: EventId) -> Option<Event> {
        let events = self.events.read();
        let named_events = self.named_events.read();
        
        for event in events.iter().chain(named_events.iter()) {
            if event.id == event_id {
                event.add_reference();
                return Some(event.clone());
            }
        }
        
        None
    }

    /// Close and remove event
    pub fn close_event(&self, event_id: EventId, process_id: u32) -> IpcResult<()> {
        let mut events = self.events.write();
        let mut named_events = self.named_events.write();
        
        // Check unnamed events
        if let Some(pos) = events.iter().position(|e| e.id == event_id) {
            let event = &events[pos];
            if event.remove_reference() {
                events.remove(pos);
                log::debug!("Process {} closed unnamed event {}", process_id, event_id);
                return Ok(());
            }
        }
        
        // Check named events
        if let Some(pos) = named_events.iter().position(|e| e.id == event_id) {
            let event = &named_events[pos];
            if event.remove_reference() {
                named_events.remove(pos);
                log::debug!("Process {} closed named event {}", process_id, event_id);
                return Ok(());
            }
        }
        
        Err(IpcError::InvalidHandle)
    }

    /// Wait for multiple events (wait_any)
    pub fn wait_any(&self, event_ids: &[EventId], process_id: u32, thread_id: u32, timeout_ns: Option<u64>) -> IpcResult<usize> {
        let events = self.events.read();
        let named_events = self.named_events.read();
        
        let mut available_events = Vec::new();
        
        // Find available events
        for &event_id in event_ids {
            for event in events.iter().chain(named_events.iter()) {
                if event.id == event_id && event.is_signalled() {
                    available_events.push(event_id);
                    break;
                }
            }
        }
        
        if !available_events.is_empty() {
            // Return index of first available event
            let event_index = event_ids.iter().position(|&id| available_events.contains(&id)).unwrap();
            Ok(event_index)
        } else {
            // Wait for any event to be signalled
            // In real implementation, would register for callbacks
            Err(IpcError::WouldBlock)
        }
    }

    /// Wait for all events (wait_all)
    pub fn wait_all(&self, event_ids: &[EventId], process_id: u32, thread_id: u32, timeout_ns: Option<u64>) -> IpcResult<()> {
        let events = self.events.read();
        let named_events = self.named_events.read();
        
        // Check if all events are signalled
        let mut all_signalled = true;
        
        for &event_id in event_ids {
            let mut found = false;
            for event in events.iter().chain(named_events.iter()) {
                if event.id == event_id && event.is_signalled() {
                    found = true;
                    break;
                }
            }
            if !found {
                all_signalled = false;
                break;
            }
        }
        
        if all_signalled {
            Ok(())
        } else {
            // Wait for all events to be signalled
            // In real implementation, would register for callbacks
            Err(IpcError::WouldBlock)
        }
    }

    /// Register for event notifications
    pub fn register_notification(&self, event_id: EventId, process_id: u32) -> IpcResult<()> {
        // In real implementation, would register process for event notifications
        log::debug!("Process {} registered for notifications on event {}", process_id, event_id);
        Ok(())
    }

    /// Get event notifications
    pub fn get_notifications(&self, max_notifications: usize) -> Vec<EventNotification> {
        let mut notifications = self.notifications.lock();
        
        let to_return = notifications.iter().take(max_notifications).cloned().collect();
        notifications.retain(|_| false); // Clear notifications after reading
        
        to_return
    }

    /// Get global event statistics
    pub fn get_global_statistics(&self) -> EventStatistics {
        let events = self.events.read();
        let named_events = self.named_events.read();
        
        let mut total_waits = 0;
        let mut total_signals = 0;
        let mut total_broadcasts = 0;
        let mut total_timeouts = 0;
        let mut total_errors = 0;
        let mut total_active_waiters = 0;
        let mut total_max_waiters = 0;
        
        for event in events.iter().chain(named_events.iter()) {
            let stats = event.get_statistics();
            total_waits += stats.wait_operations;
            total_signals += stats.signal_operations;
            total_broadcasts += stats.broadcast_operations;
            total_timeouts += stats.timeout_operations;
            total_errors += stats.errors;
            total_active_waiters += stats.active_waiters;
            total_max_waiters += stats.max_waiters;
        }
        
        EventStatistics {
            wait_operations: total_waits,
            signal_operations: total_signals,
            broadcast_operations: total_broadcasts,
            timeout_operations: total_timeouts,
            active_waiters: total_active_waiters,
            max_waiters: total_max_waiters,
            signals_sent: 0, // Will be computed separately
            signals_received: 0, // Will be computed separately
            errors: total_errors,
        }
    }

    /// Cleanup events with no references
    pub fn cleanup_events(&self) -> usize {
        let mut events = self.events.write();
        let mut named_events = self.named_events.write();
        
        let mut cleaned = 0;
        
        // Clean unnamed events
        events.retain(|event| {
            if event.remove_reference() {
                cleaned += 1;
                false // Remove event
            } else {
                true // Keep event
            }
        });
        
        // Clean named events
        named_events.retain(|event| {
            if event.remove_reference() {
                cleaned += 1;
                false // Remove event
            } else {
                true // Keep event
            }
        });
        
        if cleaned > 0 {
            log::debug!("Cleaned up {} events with no references", cleaned);
        }
        
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_reset_event() {
        let manager = EventManager::new();
        let event = manager.create_event(EventType::ManualReset, EventFlags::empty()).unwrap();
        
        // Initially not signalled
        assert!(!event.is_signalled());
        
        // Signal the event
        assert_eq!(event.signal(100, None).unwrap(), 0); // No waiters
        assert!(event.is_signalled());
        
        // Multiple waits should succeed
        assert!(event.try_wait(200, 1).is_ok());
        assert!(event.try_wait(300, 1).is_ok());
        assert!(event.is_signalled()); // Still signalled because manual reset
        
        // Reset the event
        assert!(event.reset(100).is_ok());
        assert!(!event.is_signalled());
    }

    #[test]
    fn test_auto_reset_event() {
        let manager = EventManager::new();
        let event = manager.create_event(EventType::AutoReset, EventFlags::empty()).unwrap();
        
        // Signal the event
        assert!(event.signal(100, None).is_ok());
        assert!(event.is_signalled());
        
        // First wait succeeds and resets
        assert!(event.try_wait(200, 1).is_ok());
        assert!(!event.is_signalled()); // Automatically reset
        
        // Second wait should fail
        assert_eq!(event.try_wait(300, 1), Err(IpcError::WouldBlock));
    }

    #[test]
    fn test_event_broadcast() {
        let manager = EventManager::new();
        let event = manager.create_event(EventType::ManualReset, EventFlags::BROADCAST).unwrap();
        
        // Broadcast the event
        assert_eq!(event.broadcast(100, None).unwrap(), 0); // No waiters
        
        // Add waiters and broadcast again
        let mut waiters = event.waiting_processes.write();
        waiters.push(EventWaiter {
            process_id: 200,
            thread_id: 1,
            timeout_ns: None,
            priority: 0,
            wait_start_time: 0,
        });
        waiters.push(EventWaiter {
            process_id: 300,
            thread_id: 1,
            timeout_ns: None,
            priority: 0,
            wait_start_time: 0,
        });
        drop(waiters);
        
        assert_eq!(event.broadcast(100, None).unwrap(), 2); // Woke 2 waiters
        assert!(!event.is_signalled()); // Manual reset event resets after broadcast
    }

    #[test]
    fn test_named_event() {
        let manager = EventManager::new();
        
        // Create named event
        let event1 = manager.create_named_event(b"test_event", EventType::ManualReset, EventFlags::empty()).unwrap();
        assert_eq!(event1.get_name(), Some(b"test_event" as &[u8]));
        
        // Open same named event
        let event2 = manager.open_named_event(b"test_event").unwrap();
        assert_eq!(event2.get_name(), Some(b"test_event" as &[u8]));
        assert_eq!(event1.id, event2.id);
    }

    #[test]
    fn test_event_state() {
        let manager = EventManager::new();
        let event = manager.create_event(EventType::AutoReset, EventFlags::empty()).unwrap();
        
        let state = event.get_state();
        assert!(!state.is_signalled);
        assert_eq!(state.waiters_count, 0);
        assert_eq!(state.event_type, EventType::AutoReset);
        assert_eq!(state.reference_count, 1);
    }

    #[test]
    fn test_event_wait_timeout() {
        let manager = EventManager::new();
        let event = manager.create_event(EventType::ManualReset, EventFlags::TIMEOUT).unwrap();
        
        // Wait with timeout
        assert!(event.wait(100, 1, Some(1000_000_000)).is_ok()); // 1 second timeout
        
        // Since event is not signalled, this would timeout in real implementation
        let state = event.get_state();
        assert_eq!(state.waiters_count, 1);
    }
}
