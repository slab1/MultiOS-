//! Signal Handling System for IPC
//! 
//! This module implements signal handling for inter-process communication,
//! notification, and asynchronous event handling.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::{Mutex, RwLock};
use bitflags::bitflags;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::{IpcResult, IpcError};

/// Signal number type
pub type SignalNumber = u32;

/// Standard signal numbers (POSIX-like)
pub const SIGTERM: SignalNumber = 15;
pub const SIGINT: SignalNumber = 2;
pub const SIGKILL: SignalNumber = 9;
pub const SIGUSR1: SignalNumber = 10;
pub const SIGUSR2: SignalNumber = 11;
pub const SIGHUP: SignalNumber = 1;
pub const SIGCHLD: SignalNumber = 17;

/// Maximum number of signals
const MAX_SIGNALS: usize = 64;

/// Signal flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SignalFlags: u32 {
        const REALTIME = 1 << 0;     // Real-time signal
        const RELIABLE = 1 << 1;     // Reliable delivery
        const QUEUED = 1 << 2;       // Multiple instances allowed
        const URGENT = 1 << 3;       // High priority
        const DEBUG = 1 << 4;        // Debug signal
    }
}

/// Signal actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignalAction {
    Default = 0,
    Ignore = 1,
    Terminate = 2,
    Stop = 3,
    Continue = 4,
    Custom = 5,
}

/// Signal delivery information
#[derive(Debug, Clone)]
pub struct SignalDelivery {
    pub signal: SignalNumber,
    pub sender_pid: u32,
    pub timestamp: u64,
    pub delivery_id: u64,
    pub data: Vec<u8>,
}

/// Signal handler information
#[derive(Debug, Clone)]
pub struct SignalHandler {
    pub action: SignalAction,
    pub handler_function: Option<extern "C" fn(signal: SignalNumber, info: &SignalInfo, context: *mut u8)>,
    pub mask: Vec<SignalNumber>,
    pub flags: SignalFlags,
}

/// Signal information for handlers
#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub signal: SignalNumber,
    pub sender_pid: u32,
    pub error_code: i32,
    pub trapno: i32,
    pub status: i32,
    pub si_addr: usize,
    pub si_band: i32,
    pub si_fd: i32,
    pub si_code: i32,
}

/// Signal queue for pending signals
#[derive(Debug)]
pub struct SignalQueue {
    pub pending_signals: Mutex<Vec<SignalDelivery>>,
    pub signal_mask: Mutex<Vec<SignalNumber>>,
    pub blocked_signals: AtomicU64,
}

impl SignalQueue {
    pub fn new() -> Self {
        Self {
            pending_signals: Mutex::new(Vec::new()),
            signal_mask: Mutex::new(Vec::new()),
            blocked_signals: AtomicU64::new(0),
        }
    }

    /// Add signal to pending queue
    pub fn queue_signal(&self, signal: SignalNumber, sender_pid: u32, data: Vec<u8>) -> IpcResult<()> {
        let mut pending = self.pending_signals.lock();
        
        // Check if signal is blocked
        if self.is_signal_blocked(signal) {
            return Err(IpcError::WouldBlock);
        }
        
        let delivery = SignalDelivery {
            signal,
            sender_pid,
            timestamp: 0, // Will be set by caller
            delivery_id: 0, // Will be set by caller
            data,
        };
        
        pending.push(delivery);
        log::debug!("Signal {} queued for process {} (pending: {})", signal, sender_pid, pending.len());
        
        Ok(())
    }

    /// Get next pending signal
    pub fn get_pending_signal(&self) -> Option<SignalDelivery> {
        let mut pending = self.pending_signals.lock();
        if let Some(delivery) = pending.pop() {
            log::debug!("Delivering signal {} to process", delivery.signal);
            Some(delivery)
        } else {
            None
        }
    }

    /// Check if signal is blocked
    pub fn is_signal_blocked(&self, signal: SignalNumber) -> bool {
        let mask = self.signal_mask.lock();
        mask.contains(&signal)
    }

    /// Block/unblock signal
    pub fn block_signal(&self, signal: SignalNumber, block: bool) {
        let mut mask = self.signal_mask.lock();
        
        if block {
            if !mask.contains(&signal) {
                mask.push(signal);
            }
        } else {
            mask.retain(|&s| s != signal);
        }
    }

    /// Check if there are pending signals
    pub fn has_pending_signals(&self) -> bool {
        !self.pending_signals.lock().is_empty()
    }
}

/// Process signal state
#[derive(Debug)]
pub struct ProcessSignalState {
    pub pid: u32,
    pub signal_queue: SignalQueue,
    pub handlers: Vec<Option<SignalHandler>>,
    pub default_actions: BTreeMap<SignalNumber, SignalAction>,
    pub signal_disposition: Vec<SignalAction>,
    pub signal_mask: Vec<SignalNumber>,
    pub pending_signals: AtomicU64,
}

impl ProcessSignalState {
    pub fn new(pid: u32) -> Self {
        let mut default_actions = BTreeMap::new();
        default_actions.insert(SIGTERM, SignalAction::Terminate);
        default_actions.insert(SIGKILL, SignalAction::Terminate);
        default_actions.insert(SIGINT, SignalAction::Terminate);
        default_actions.insert(SIGHUP, SignalAction::Terminate);
        default_actions.insert(SIGUSR1, SignalAction::Default);
        default_actions.insert(SIGUSR2, SignalAction::Default);
        default_actions.insert(SIGCHLD, SignalAction::Ignore);

        Self {
            pid,
            signal_queue: SignalQueue::new(),
            handlers: vec![None; MAX_SIGNALS],
            default_actions,
            signal_disposition: vec![SignalAction::Default; MAX_SIGNALS],
            signal_mask: Vec::new(),
            pending_signals: AtomicU64::new(0),
        }
    }

    /// Install signal handler
    pub fn install_handler(&mut self, signal: SignalNumber, handler: SignalHandler) -> IpcResult<()> {
        if signal as usize >= MAX_SIGNALS {
            return Err(IpcError::InvalidHandle);
        }

        self.handlers[signal as usize] = Some(handler);
        log::debug!("Installed signal handler for signal {} in process {}", signal, self.pid);
        
        Ok(())
    }

    /// Set signal disposition
    pub fn set_disposition(&mut self, signal: SignalNumber, action: SignalAction) -> IpcResult<()> {
        if signal as usize >= MAX_SIGNALS {
            return Err(IpcError::InvalidHandle);
        }

        self.signal_disposition[signal as usize] = action;
        log::debug!("Set signal {} disposition to {:?} in process {}", signal, action, self.pid);
        
        Ok(())
    }

    /// Send signal to process
    pub fn send_signal(&self, signal: SignalNumber, sender_pid: u32, data: Vec<u8>) -> IpcResult<()> {
        // Check if signal is blocked for this process
        if self.signal_queue.is_signal_blocked(signal) {
            log::debug!("Signal {} is blocked for process {}", signal, self.pid);
            return Err(IpcError::WouldBlock);
        }

        self.signal_queue.queue_signal(signal, sender_pid, data)?;
        
        // Set pending signal bit
        let bit = 1u64 << (signal % 64);
        self.pending_signals.fetch_or(bit, Ordering::SeqCst);
        
        Ok(())
    }

    /// Process pending signals
    pub fn process_pending_signals(&self) -> Vec<SignalDelivery> {
        let mut delivered = Vec::new();
        
        while let Some(signal_delivery) = self.signal_queue.get_pending_signal() {
            let signal = signal_delivery.signal;
            
            // Check signal disposition
            let action = if signal as usize < self.signal_disposition.len() {
                self.signal_disposition[signal as usize]
            } else {
                SignalAction::Default
            };

            match action {
                SignalAction::Ignore => {
                    log::debug!("Ignoring signal {} in process {}", signal, self.pid);
                    continue;
                }
                SignalAction::Terminate => {
                    log::debug!("Signal {} would terminate process {}", signal, self.pid);
                    delivered.push(signal_delivery);
                    break;
                }
                SignalAction::Default | SignalAction::Custom => {
                    delivered.push(signal_delivery);
                    log::debug!("Delivering signal {} to process {}", signal, self.pid);
                }
                _ => {
                    delivered.push(signal_delivery);
                }
            }
        }

        // Clear pending signals
        self.pending_signals.store(0, Ordering::SeqCst);
        delivered
    }

    /// Get pending signal count
    pub fn get_pending_count(&self) -> usize {
        self.signal_queue.pending_signals.lock().len()
    }

    /// Check if specific signal is pending
    pub fn is_signal_pending(&self, signal: SignalNumber) -> bool {
        let bit = 1u64 << (signal % 64);
        (self.pending_signals.load(Ordering::SeqCst) & bit) != 0
    }

    /// Signal mask operations
    pub fn add_to_mask(&self, signals: &[SignalNumber]) {
        let mut mask = self.signal_queue.signal_mask.lock();
        for &signal in signals {
            if !mask.contains(&signal) {
                mask.push(signal);
            }
        }
    }

    pub fn remove_from_mask(&self, signals: &[SignalNumber]) {
        let mut mask = self.signal_queue.signal_mask.lock();
        for &signal in signals {
            mask.retain(|&s| s != signal);
        }
    }
}

/// Signal event notification
#[derive(Debug, Clone)]
pub struct SignalEvent {
    pub event_type: SignalEventType,
    pub signal: SignalNumber,
    pub target_pid: u32,
    pub source_pid: u32,
    pub timestamp: u64,
}

/// Signal event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignalEventType {
    SignalSent = 0,
    SignalReceived = 1,
    SignalDelivered = 2,
    SignalBlocked = 3,
    SignalIgnored = 4,
}

/// Global signal manager
#[derive(Debug)]
pub struct SignalManager {
    pub process_states: RwLock<BTreeMap<u32, ProcessSignalState>>,
    pub signal_events: Mutex<Vec<SignalEvent>>,
    pub signal_statistics: SignalStatistics,
    pub next_delivery_id: AtomicU64,
}

impl SignalManager {
    pub fn new() -> Self {
        Self {
            process_states: RwLock::new(BTreeMap::new()),
            signal_events: Mutex::new(Vec::new()),
            signal_statistics: SignalStatistics::default(),
            next_delivery_id: AtomicU64::new(1),
        }
    }

    /// Register process for signal handling
    pub fn register_process(&self, pid: u32) {
        let mut states = self.process_states.write();
        if !states.contains_key(&pid) {
            states.insert(pid, ProcessSignalState::new(pid));
            log::debug!("Registered process {} for signal handling", pid);
        }
    }

    /// Unregister process
    pub fn unregister_process(&self, pid: u32) {
        let mut states = self.process_states.write();
        states.remove(&pid);
        log::debug!("Unregistered process {} from signal handling", pid);
    }

    /// Send signal to process
    pub fn send_signal(&self, signal: SignalNumber, target_pid: u32, sender_pid: u32, data: Vec<u8>) -> IpcResult<()> {
        let states = self.process_states.read();
        
        if let Some(process_state) = states.get(&target_pid) {
            process_state.send_signal(signal, sender_pid, data)?;
            
            // Record event
            let mut events = self.signal_events.lock();
            events.push(SignalEvent {
                event_type: SignalEventType::SignalSent,
                signal,
                target_pid,
                source_pid: sender_pid,
                timestamp: 0, // Will be set by caller
            });
            
            self.signal_statistics.signals_sent += 1;
            log::debug!("Signal {} sent from process {} to process {}", signal, sender_pid, target_pid);
            
            Ok(())
        } else {
            Err(IpcError::NoSuchProcess)
        }
    }

    /// Process pending signals for a process
    pub fn process_signals(&self, pid: u32) -> Vec<SignalDelivery> {
        let states = self.process_states.read();
        if let Some(process_state) = states.get(&pid) {
            let delivered = process_state.process_pending_signals();
            
            // Record events
            let mut events = self.signal_events.lock();
            for delivery in &delivered {
                events.push(SignalEvent {
                    event_type: SignalEventType::SignalDelivered,
                    signal: delivery.signal,
                    target_pid: pid,
                    source_pid: delivery.sender_pid,
                    timestamp: delivery.timestamp,
                });
                
                self.signal_statistics.signals_delivered += 1;
            }
            
            delivered
        } else {
            Vec::new()
        }
    }

    /// Get process signal state
    pub fn get_process_state(&self, pid: u32) -> Option<ProcessSignalState> {
        let states = self.process_states.read();
        states.get(&pid).cloned()
    }

    /// Install signal handler for process
    pub fn install_handler(&self, pid: u32, signal: SignalNumber, handler: SignalHandler) -> IpcResult<()> {
        let mut states = self.process_states.write();
        
        if let Some(process_state) = states.get_mut(&pid) {
            process_state.install_handler(signal, handler)
        } else {
            Err(IpcError::NoSuchProcess)
        }
    }

    /// Set signal disposition for process
    pub fn set_disposition(&self, pid: u32, signal: SignalNumber, action: SignalAction) -> IpcResult<()> {
        let mut states = self.process_states.write();
        
        if let Some(process_state) = states.get_mut(&pid) {
            process_state.set_disposition(signal, action)
        } else {
            Err(IpcError::NoSuchProcess)
        }
    }

    /// Broadcast signal to multiple processes
    pub fn broadcast_signal(&self, signal: SignalNumber, target_pids: &[u32], sender_pid: u32, data: Vec<u8>) -> IpcResult<usize> {
        let mut delivered_count = 0;
        
        for &target_pid in target_pids {
            match self.send_signal(signal, target_pid, sender_pid, data.clone()) {
                Ok(_) => delivered_count += 1,
                Err(_) => {
                    log::debug!("Failed to deliver signal {} to process {}", signal, target_pid);
                }
            }
        }
        
        Ok(delivered_count)
    }

    /// Get signal events
    pub fn get_events(&self, max_events: usize) -> Vec<SignalEvent> {
        let mut events = self.signal_events.lock();
        
        let to_return = events.iter().take(max_events).cloned().collect();
        events.retain(|_| false); // Clear events after reading
        
        to_return
    }

    /// Get global signal statistics
    pub fn get_statistics(&self) -> SignalStatistics {
        self.signal_statistics.clone()
    }
}

/// Signal statistics
#[derive(Debug, Clone, Default)]
pub struct SignalStatistics {
    pub signals_sent: u64,
    pub signals_received: u64,
    pub signals_delivered: u64,
    pub signals_blocked: u64,
    pub signals_ignored: u64,
    pub active_processes: u32,
    pub total_handlers: u32,
    pub errors: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_queuing() {
        let manager = SignalManager::new();
        
        // Register process
        manager.register_process(100);
        
        // Send signal
        assert!(manager.send_signal(SIGTERM, 100, 200, Vec::new()).is_ok());
        
        // Process signals
        let delivered = manager.process_signals(100);
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].signal, SIGTERM);
    }

    #[test]
    fn test_signal_blocking() {
        let manager = SignalManager::new();
        manager.register_process(100);
        
        // Block signal
        if let Some(state) = manager.get_process_state(100) {
            state.signal_queue.block_signal(SIGUSR1, true);
            assert!(state.signal_queue.is_signal_blocked(SIGUSR1));
            
            // Try to send blocked signal
            assert_eq!(
                manager.send_signal(SIGUSR1, 100, 200, Vec::new()),
                Err(IpcError::WouldBlock)
            );
        }
    }

    #[test]
    fn test_signal_disposition() {
        let manager = SignalManager::new();
        manager.register_process(100);
        
        // Set SIGTERM to ignore
        assert!(manager.set_disposition(100, SIGTERM, SignalAction::Ignore).is_ok());
        
        // Send signal
        assert!(manager.send_signal(SIGTERM, 100, 200, Vec::new()).is_ok());
        
        // Process signals - should be ignored
        let delivered = manager.process_signals(100);
        assert!(delivered.is_empty());
    }

    #[test]
    fn test_signal_broadcast() {
        let manager = SignalManager::new();
        
        manager.register_process(100);
        manager.register_process(200);
        manager.register_process(300);
        
        let targets = vec![100, 200, 300];
        let delivered_count = manager.broadcast_signal(SIGUSR1, &targets, 999, Vec::new()).unwrap();
        
        assert_eq!(delivered_count, 3);
        
        // Check each process got the signal
        for pid in targets {
            let delivered = manager.process_signals(pid);
            assert_eq!(delivered.len(), 1);
        }
    }
}
