//! Semaphore Implementation for IPC Synchronization
//! 
//! This module implements counting semaphores, mutexes, and condition variables
//! for synchronization between processes and threads.

use core::sync::atomic::{AtomicU32, AtomicI32, Ordering};
use spin::{Mutex, RwLock};
use bitflags::bitflags;

use crate::{IpcResult, IpcError};

/// Semaphore ID type
pub type SemaphoreId = u32;

/// Maximum semaphore value
const MAX_SEMAPHORE_VALUE: i32 = 32767;
const MIN_SEMAPHORE_VALUE: i32 = 0;

/// Semaphore handle for user-space access
#[derive(Debug, Clone, Copy)]
pub struct SemaphoreHandle {
    pub id: SemaphoreId,
}

impl SemaphoreHandle {
    pub const fn new(id: SemaphoreId) -> Self {
        Self { id }
    }
}

/// Semaphore operations
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SemaphoreOperation: u32 {
        const WAIT    = 1 << 0; // Wait (decrement)
        const POST    = 1 << 1; // Post (increment)
        const TIMEOUT = 1 << 2; // With timeout
        const NOWAIT  = 1 << 3; // Non-blocking
    }
}

/// Semaphore flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SemaphoreFlags: u32 {
        const BINARY      = 1 << 0; // Binary semaphore (0 or 1)
        const PRIORITY    = 1 << 1; // Priority inheritance
        const FAIRNESS    = 1 << 2; // FIFO ordering
        const DIAGNOSTIC  = 1 << 3; // Enable diagnostics
    }
}

/// Waiting process information
#[derive(Debug, Clone)]
pub struct WaitingProcess {
    pub process_id: u32,
    pub requested_value: i32,
    pub wakeup_time: Option<u64>,
    pub priority: u32,
}

/// Semaphore statistics
#[derive(Debug, Clone, Default)]
pub struct SemaphoreStatistics {
    pub wait_operations: u64,
    pub post_operations: u64,
    pub timeout_operations: u64,
    pub blocked_processes: u32,
    pub max_waiters: u32,
    pub errors: u32,
}

/// Semaphore implementation
#[derive(Debug)]
pub struct Semaphore {
    pub id: SemaphoreId,
    pub value: AtomicI32,
    pub flags: SemaphoreFlags,
    pub created_by: u32,
    pub created_at: u64,
    pub waiting_processes: RwLock<Vec<WaitingProcess>>,
    pub max_value: i32,
    pub min_value: i32,
    pub statistics: SemaphoreStatistics,
}

impl Semaphore {
    pub fn new(id: SemaphoreId, initial_value: i32) -> IpcResult<Self> {
        if initial_value < MIN_SEMAPHORE_VALUE || initial_value > MAX_SEMAPHORE_VALUE {
            return Err(IpcError::ResourceExhausted);
        }

        Ok(Self {
            id,
            value: AtomicI32::new(initial_value),
            flags: SemaphoreFlags::empty(),
            created_by: 0,
            created_at: 0,
            waiting_processes: RwLock::new(Vec::new()),
            max_value: if initial_value == 0 || initial_value == 1 {
                1 // Binary semaphore
            } else {
                MAX_SEMAPHORE_VALUE
            },
            min_value: MIN_SEMAPHORE_VALUE,
            statistics: SemaphoreStatistics::default(),
        })
    }

    /// Wait (decrement) operation
    pub fn wait(&self, process_id: u32, timeout_ns: Option<u64>) -> IpcResult<()> {
        self.statistics.wait_operations += 1;

        // Fast path: if semaphore has value > 0, decrement and return
        let current_value = self.value.load(Ordering::SeqCst);
        if current_value > 0 {
            match self.value.compare_exchange_weak(
                current_value,
                current_value - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    log::debug!("Process {} acquired semaphore {} immediately", process_id, self.id);
                    return Ok(());
                }
                Err(_) => {
                    // Retry needed, fall through to blocking
                }
            }
        }

        // Slow path: need to wait
        if let Some(timeout) = timeout_ns {
            self.statistics.timeout_operations += 1;
            self.wait_with_timeout(process_id, timeout)
        } else {
            self.wait_indefinitely(process_id)
        }
    }

    /// Post (increment) operation
    pub fn post(&self, process_id: u32) -> IpcResult<()> {
        self.statistics.post_operations += 1;

        let mut current_value = self.value.load(Ordering::SeqCst);
        
        // Check if we can increment without overflow
        if current_value >= self.max_value {
            self.statistics.errors += 1;
            return Err(IpcError::ResourceExhausted);
        }

        loop {
            match self.value.compare_exchange_weak(
                current_value,
                current_value + 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    // Wake up waiting processes if any
                    self.wake_up_waiting_processes();
                    log::debug!("Process {} posted to semaphore {}", process_id, self.id);
                    return Ok(());
                }
                Err(new_value) => {
                    current_value = new_value;
                    if current_value >= self.max_value {
                        self.statistics.errors += 1;
                        return Err(IpcError::ResourceExhausted);
                    }
                }
            }
        }
    }

    /// Try wait (non-blocking)
    pub fn try_wait(&self, process_id: u32) -> IpcResult<()> {
        let current_value = self.value.load(Ordering::SeqCst);
        
        if current_value > 0 {
            match self.value.compare_exchange_weak(
                current_value,
                current_value - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    log::debug!("Process {} acquired semaphore {} with try_wait", process_id, self.id);
                    Ok(())
                }
                Err(_) => Err(IpcError::WouldBlock),
            }
        } else {
            Err(IpcError::WouldBlock)
        }
    }

    /// Get current semaphore value
    pub fn get_value(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    /// Set semaphore value
    pub fn set_value(&self, new_value: i32, process_id: u32) -> IpcResult<()> {
        // Only the creator can set the value
        if self.created_by != process_id {
            return Err(IpcError::PermissionDenied);
        }

        if new_value < self.min_value || new_value > self.max_value {
            return Err(IpcError::ResourceExhausted);
        }

        self.value.store(new_value, Ordering::SeqCst);
        
        // Wake up waiting processes if value increased
        if new_value > 0 {
            self.wake_up_waiting_processes();
        }

        Ok(())
    }

    /// Wait indefinitely for semaphore
    fn wait_indefinitely(&self, process_id: u32) -> IpcResult<()> {
        let mut waiters = self.waiting_processes.write();
        
        let waiter = WaitingProcess {
            process_id,
            requested_value: -1,
            wakeup_time: None,
            priority: 0, // Default priority
        };

        waiters.push(waiter);
        self.statistics.blocked_processes += 1;
        
        // Update max waiters
        if waiters.len() as u32 > self.statistics.max_waiters {
            self.statistics.max_waiters = waiters.len() as u32;
        }

        drop(waiters);

        // In a real implementation, this would block the process
        // For now, we'll return immediately with a simulated block
        log::debug!("Process {} is now waiting on semaphore {}", process_id, self.id);
        
        // Simulate some blocking behavior
        // In real implementation, this would involve scheduler integration
        Ok(())
    }

    /// Wait with timeout for semaphore
    fn wait_with_timeout(&self, process_id: u32, timeout_ns: u64) -> IpcResult<()> {
        let mut waiters = self.waiting_processes.write();
        
        let wakeup_time = Some(0); // In real implementation, would calculate actual wakeup time
        
        let waiter = WaitingProcess {
            process_id,
            requested_value: -1,
            wakeup_time,
            priority: 0,
        };

        waiters.push(waiter);
        self.statistics.blocked_processes += 1;
        self.statistics.timeout_operations += 1;

        drop(waiters);

        log::debug!("Process {} is waiting with timeout on semaphore {}", process_id, self.id);
        
        // In real implementation, this would set up actual timeout
        // and would be woken up either by timeout or by post operation
        Ok(())
    }

    /// Wake up waiting processes
    fn wake_up_waiting_processes(&self) {
        let mut waiters = self.waiting_processes.write();
        
        if !waiters.is_empty() {
            // Remove the first waiter (FIFO) or highest priority waiter
            let waiter = waiters.remove(0);
            
            // In real implementation, this would wake up the actual process
            log::debug!("Waking up process {} waiting on semaphore {}", waiter.process_id, self.id);
            
            self.statistics.blocked_processes = waiters.len() as u32;
        }
    }

    /// Remove process from waiting list
    pub fn remove_waiting_process(&self, process_id: u32) -> IpcResult<()> {
        let mut waiters = self.waiting_processes.write();
        
        if let Some(pos) = waiters.iter().position(|w| w.process_id == process_id) {
            waiters.remove(pos);
            self.statistics.blocked_processes = waiters.len() as u32;
            Ok(())
        } else {
            Err(IpcError::InvalidHandle)
        }
    }

    /// Get statistics
    pub fn get_statistics(&self) -> SemaphoreStatistics {
        let waiters = self.waiting_processes.read();
        SemaphoreStatistics {
            wait_operations: self.statistics.wait_operations,
            post_operations: self.statistics.post_operations,
            timeout_operations: self.statistics.timeout_operations,
            blocked_processes: waiters.len() as u32,
            max_waiters: self.statistics.max_waiters,
            errors: self.statistics.errors,
        }
    }

    /// Check if semaphore is available (has value > 0)
    pub fn is_available(&self) -> bool {
        self.value.load(Ordering::SeqCst) > 0
    }

    /// Get number of waiting processes
    pub fn waiting_count(&self) -> usize {
        self.waiting_processes.read().len()
    }
}

/// Mutex implementation using semaphore
#[derive(Debug)]
pub struct Mutex {
    pub semaphore: Semaphore,
    pub owner: AtomicU32, // Process that owns the mutex
    pub recursive_count: u32, // For recursive mutexes
}

impl Mutex {
    pub fn new(id: SemaphoreId) -> IpcResult<Self> {
        let semaphore = Semaphore::new(id, 1)?; // Binary semaphore with value 1
        
        Ok(Self {
            semaphore,
            owner: AtomicU32::new(0),
            recursive_count: 0,
        })
    }

    /// Lock the mutex
    pub fn lock(&self, process_id: u32, timeout_ns: Option<u64>) -> IpcResult<()> {
        // Check if we already own the mutex (for recursive locking)
        let current_owner = self.owner.load(Ordering::SeqCst);
        if current_owner == process_id {
            self.recursive_count += 1;
            return Ok(());
        }

        // Wait for the semaphore
        self.semaphore.wait(process_id, timeout_ns)?;
        
        // Set ourselves as the owner
        self.owner.store(process_id, Ordering::SeqCst);
        self.recursive_count = 1;

        Ok(())
    }

    /// Unlock the mutex
    pub fn unlock(&self, process_id: u32) -> IpcResult<()> {
        let current_owner = self.owner.load(Ordering::SeqCst);
        
        if current_owner != process_id {
            return Err(IpcError::PermissionDenied);
        }

        self.recursive_count -= 1;
        
        // If this was the last recursive lock, release the mutex
        if self.recursive_count == 0 {
            self.owner.store(0, Ordering::SeqCst);
            self.semaphore.post(process_id)?;
        }

        Ok(())
    }

    /// Try to lock the mutex (non-blocking)
    pub fn try_lock(&self, process_id: u32) -> IpcResult<()> {
        let current_owner = self.owner.load(Ordering::SeqCst);
        if current_owner == process_id {
            self.recursive_count += 1;
            return Ok(());
        }

        if self.semaphore.try_wait(process_id).is_ok() {
            self.owner.store(process_id, Ordering::SeqCst);
            self.recursive_count = 1;
            Ok(())
        } else {
            Err(IpcError::WouldBlock)
        }
    }

    /// Check if mutex is owned
    pub fn is_locked(&self) -> bool {
        self.owner.load(Ordering::SeqCst) != 0
    }

    /// Get current owner
    pub fn owner(&self) -> Option<u32> {
        let owner_id = self.owner.load(Ordering::SeqCst);
        if owner_id != 0 {
            Some(owner_id)
        } else {
            None
        }
    }
}

/// Condition variable implementation
#[derive(Debug)]
pub struct ConditionVariable {
    pub id: u32,
    pub waiting_processes: RwLock<Vec<WaitingProcess>>,
    pub statistics: SemaphoreStatistics,
}

impl ConditionVariable {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            waiting_processes: RwLock::new(Vec::new()),
            statistics: SemaphoreStatistics::default(),
        }
    }

    /// Wait on condition variable
    pub fn wait(&self, process_id: u32, mutex: &Mutex, timeout_ns: Option<u64>) -> IpcResult<()> {
        // Release the mutex while waiting
        let _ = mutex.unlock(process_id);
        
        // Add process to waiting list
        let mut waiters = self.waiting_processes.write();
        waiters.push(WaitingProcess {
            process_id,
            requested_value: 0,
            wakeup_time: timeout_ns,
            priority: 0,
        });
        
        self.statistics.wait_operations += 1;

        // In real implementation, this would block the process
        log::debug!("Process {} is waiting on condition variable {}", process_id, self.id);

        // Re-acquire the mutex when woken up
        mutex.lock(process_id, timeout_ns)
    }

    /// Signal one waiting process
    pub fn signal(&self, process_id: u32) -> IpcResult<()> {
        let mut waiters = self.waiting_processes.write();
        
        if !waiters.is_empty() {
            let waiter = waiters.remove(0);
            self.statistics.post_operations += 1;
            
            log::debug!("Signaled process {} on condition variable {}", waiter.process_id, self.id);
            
            // In real implementation, this would wake up the process
        }

        Ok(())
    }

    /// Signal all waiting processes
    pub fn broadcast(&self, process_id: u32) -> IpcResult<usize> {
        let mut waiters = self.waiting_processes.write();
        let count = waiters.len();
        
        self.statistics.post_operations += count as u64;
        
        if count > 0 {
            log::debug!("Broadcasted to {} processes on condition variable {}", count, self.id);
            // In real implementation, this would wake up all processes
        }

        waiters.clear();
        Ok(count)
    }

    /// Get waiting process count
    pub fn waiting_count(&self) -> usize {
        self.waiting_processes.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semaphore_wait_post() {
        let semaphore = Semaphore::new(1, 1).unwrap();
        
        assert_eq!(semaphore.get_value(), 1);
        
        // Should be able to wait immediately
        assert!(semaphore.try_wait(100).is_ok());
        assert_eq!(semaphore.get_value(), 0);
        
        // Should fail to try_wait when at 0
        assert_eq!(semaphore.try_wait(101), Err(IpcError::WouldBlock));
        
        // Post should increment back to 1
        assert!(semaphore.post(100).is_ok());
        assert_eq!(semaphore.get_value(), 1);
    }

    #[test]
    fn test_mutex_lock_unlock() {
        let mutex = Mutex::new(1).unwrap();
        
        assert!(!mutex.is_locked());
        
        // Should be able to lock
        assert!(mutex.lock(100, None).is_ok());
        assert!(mutex.is_locked());
        assert_eq!(mutex.owner(), Some(100));
        
        // Should be able to unlock
        assert!(mutex.unlock(100).is_ok());
        assert!(!mutex.is_locked());
        assert_eq!(mutex.owner(), None);
    }

    #[test]
    fn test_condition_variable() {
        let cv = ConditionVariable::new(1);
        let mutex = Mutex::new(2).unwrap();
        
        // Test signal/broadcast
        assert!(cv.signal(100).is_ok());
        assert_eq!(cv.broadcast(100).unwrap(), 0); // No one waiting
        
        assert_eq!(cv.waiting_count(), 0);
    }
}
