//! Message Queue Implementation for IPC
//! 
//! This module implements POSIX-style message queues for reliable,
//! prioritized message passing between processes.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::{Mutex, RwLock};
use bitflags::bitflags;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::{IpcResult, IpcError};

/// Message queue ID
pub type MessageQueueId = u32;

/// Maximum message size (8KB)
const MAX_MESSAGE_SIZE: usize = 8192;
const DEFAULT_MESSAGE_SIZE: usize = 1024;

/// Maximum number of messages in queue
const MAX_MESSAGES: usize = 256;

/// Message queue handle for user-space access
#[derive(Debug, Clone, Copy)]
pub struct MessageQueueHandle {
    pub id: MessageQueueId,
}

impl MessageQueueHandle {
    pub const fn new(id: MessageQueueId) -> Self {
        Self { id }
    }
}

/// Message priority levels
const MQ_PRIO_MAX: u32 = 32;
const MQ_PRIO_MIN: u32 = 0;

/// Message structure
#[derive(Debug, Clone)]
pub struct Message {
    pub data: Vec<u8>,
    pub priority: u32,
    pub sender_id: u32,
    pub timestamp: u64,
    pub message_id: u64,
    pub delivery_count: u32,
}

/// Message queue flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MessageQueueFlags: u32 {
        const NON_BLOCKING = 1 << 0;
        const PRIORITY_ORDER = 1 << 1;
        const BROADCAST = 1 << 2;
        const RELIABLE = 1 << 3;
        const ATOMIC = 1 << 4;
    }
}

/// Consumer information
#[derive(Debug, Clone)]
pub struct Consumer {
    pub process_id: u32,
    pub permissions: u32,
    pub last_receive_time: u64,
    pub receive_count: u64,
}

/// Message queue statistics
#[derive(Debug, Clone, Default)]
pub struct MessageQueueStatistics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub dropped_messages: u64,
    pub consumers_connected: u32,
    pub max_queue_size: u32,
    pub current_queue_size: u32,
    pub errors: u32,
}

/// Message queue implementation
#[derive(Debug)]
pub struct MessageQueue {
    pub id: MessageQueueId,
    pub name: Vec<u8>,
    pub max_message_size: usize,
    pub max_messages: usize,
    pub flags: MessageQueueFlags,
    pub created_by: u32,
    pub created_at: u64,
    pub messages: Mutex<Vec<Message>>, // Sorted by priority
    pub consumers: RwLock<Vec<Consumer>>,
    pub waiting_receivers: Mutex<Vec<u32>>, // Process IDs waiting to receive
    pub waiting_senders: Mutex<Vec<u32>>, // Process IDs waiting to send
    pub statistics: MessageQueueStatistics,
}

impl MessageQueue {
    pub fn new(id: MessageQueueId, name: &[u8], max_message_size: usize, max_messages: usize) -> IpcResult<Self> {
        if max_message_size == 0 || max_message_size > MAX_MESSAGE_SIZE {
            return Err(IpcError::ResourceExhausted);
        }

        if max_messages == 0 || max_messages > MAX_MESSAGES {
            return Err(IpcError::ResourceExhausted);
        }

        Ok(Self {
            id,
            name: name.to_vec(),
            max_message_size,
            max_messages,
            flags: MessageQueueFlags::empty(),
            created_by: 0,
            created_at: 0,
            messages: Mutex::new(Vec::new()),
            consumers: RwLock::new(Vec::new()),
            waiting_receivers: Mutex::new(Vec::new()),
            waiting_senders: Mutex::new(Vec::new()),
            statistics: MessageQueueStatistics::default(),
        })
    }

    /// Send a message to the queue
    pub fn send(&self, data: &[u8], priority: u32, sender_id: u64, timeout_ns: Option<u64>) -> IpcResult<()> {
        if data.len() > self.max_message_size {
            self.statistics.errors += 1;
            return Err(IpcError::BufferTooSmall);
        }

        let priority = if priority > MQ_PRIO_MAX {
            MQ_PRIO_MAX
        } else {
            priority
        };

        let mut messages = self.messages.lock();
        
        // Check if queue is full
        if messages.len() >= self.max_messages {
            self.statistics.dropped_messages += 1;
            
            // If full and blocking, add to waiting senders
            if !self.flags.contains(MessageQueueFlags::NON_BLOCKING) {
                drop(messages);
                self.waiting_senders.lock().push(sender_id as u32);
                
                if let Some(timeout) = timeout_ns {
                    // In real implementation, would set up timeout
                    log::debug!("Process {} waiting to send to queue {} (timeout: {} ns)", sender_id, self.id, timeout);
                } else {
                    log::debug!("Process {} waiting to send to queue {}", sender_id, self.id);
                }
                
                return Ok(()); // In real implementation, would block here
            } else {
                self.statistics.errors += 1;
                return Err(IpcError::WouldBlock);
            }
        }

        // Create message
        let message = Message {
            data: data.to_vec(),
            priority,
            sender_id: sender_id as u32,
            timestamp: 0, // Will be set by caller
            message_id: self.generate_message_id(),
            delivery_count: 1,
        };

        // Insert message (maintain priority order)
        let insert_pos = if self.flags.contains(MessageQueueFlags::PRIORITY_ORDER) {
            // Insert in priority order (higher priority first)
            messages.iter().position(|m| m.priority < priority).unwrap_or(messages.len())
        } else {
            messages.len() // Append to end
        };

        messages.insert(insert_pos, message);
        self.statistics.messages_sent += 1;
        self.statistics.bytes_sent += data.len() as u64;
        self.statistics.current_queue_size = messages.len() as u32;

        // Update max queue size
        if messages.len() as u32 > self.statistics.max_queue_size {
            self.statistics.max_queue_size = messages.len() as u32;
        }

        drop(messages);

        // Wake up waiting receivers
        self.wake_up_receivers();

        log::debug!("Process {} sent {} bytes to queue {} (priority: {})", sender_id, data.len(), self.id, priority);
        Ok(())
    }

    /// Receive a message from the queue
    pub fn receive(&self, buffer: &mut [u8], priority: &mut u32, receiver_id: u64, timeout_ns: Option<u64>) -> IpcResult<usize> {
        let mut messages = self.messages.lock();
        
        // Check if queue is empty
        if messages.is_empty() {
            if self.flags.contains(MessageQueueFlags::NON_BLOCKING) {
                self.statistics.errors += 1;
                return Err(IpcError::WouldBlock);
            } else {
                drop(messages);
                self.waiting_receivers.lock().push(receiver_id as u32);
                
                if let Some(timeout) = timeout_ns {
                    // In real implementation, would set up timeout
                    log::debug!("Process {} waiting to receive from queue {} (timeout: {} ns)", receiver_id, self.id, timeout);
                } else {
                    log::debug!("Process {} waiting to receive from queue {}", receiver_id, self.id);
                }
                
                return Ok(0); // In real implementation, would block here
            }
        }

        // Get first message (highest priority if priority order is enabled)
        let message = if self.flags.contains(MessageQueueFlags::PRIORITY_ORDER) {
            messages.remove(0)
        } else {
            messages.remove(messages.len() - 1) // FIFO
        };

        self.statistics.messages_received += 1;
        self.statistics.bytes_received += message.data.len() as u64;
        self.statistics.current_queue_size = messages.len() as u32;

        drop(messages);

        // Copy message data to buffer
        let bytes_to_copy = message.data.len().min(buffer.len());
        buffer[..bytes_to_copy].copy_from_slice(&message.data[..bytes_to_copy]);
        *priority = message.priority;

        // Update receiver statistics
        self.update_receiver_stats(receiver_id as u32, &message);

        log::debug!("Process {} received {} bytes from queue {} (priority: {})", receiver_id, bytes_to_copy, self.id, message.priority);
        Ok(bytes_to_copy)
    }

    /// Try to send a message (non-blocking)
    pub fn try_send(&self, data: &[u8], priority: u32, sender_id: u64) -> IpcResult<()> {
        if self.flags.contains(MessageQueueFlags::NON_BLOCKING) {
            self.send(data, priority, sender_id, None)
        } else {
            // Convert to non-blocking for this operation
            let original_flags = self.flags;
            self.flags.remove(MessageQueueFlags::NON_BLOCKING);
            let result = self.send(data, priority, sender_id, None);
            self.flags = original_flags;
            result
        }
    }

    /// Try to receive a message (non-blocking)
    pub fn try_receive(&self, buffer: &mut [u8], priority: &mut u32, receiver_id: u64) -> IpcResult<usize> {
        if self.flags.contains(MessageQueueFlags::NON_BLOCKING) {
            self.receive(buffer, priority, receiver_id, None)
        } else {
            // Convert to non-blocking for this operation
            let original_flags = self.flags;
            self.flags.insert(MessageQueueFlags::NON_BLOCKING);
            let result = self.receive(buffer, priority, receiver_id, None);
            self.flags = original_flags;
            result
        }
    }

    /// Register a consumer (process that can receive messages)
    pub fn register_consumer(&self, process_id: u32, permissions: u32) -> IpcResult<()> {
        let mut consumers = self.consumers.write();
        
        // Check if already registered
        if !consumers.iter().any(|c| c.process_id == process_id) {
            consumers.push(Consumer {
                process_id,
                permissions,
                last_receive_time: 0,
                receive_count: 0,
            });
            
            self.statistics.consumers_connected += 1;
            log::debug!("Process {} registered as consumer of queue {}", process_id, self.id);
        }

        Ok(())
    }

    /// Unregister a consumer
    pub fn unregister_consumer(&self, process_id: u32) -> IpcResult<()> {
        let mut consumers = self.consumers.write();
        
        if let Some(pos) = consumers.iter().position(|c| c.process_id == process_id) {
            consumers.remove(pos);
            self.statistics.consumers_connected -= 1;
            log::debug!("Process {} unregistered as consumer of queue {}", process_id, self.id);
            Ok(())
        } else {
            Err(IpcError::InvalidHandle)
        }
    }

    /// Get current queue state
    pub fn get_queue_state(&self) -> QueueState {
        let messages = self.messages.lock();
        QueueState {
            message_count: messages.len(),
            total_bytes: messages.iter().map(|m| m.data.len()).sum(),
            current_max_priority: messages.first().map(|m| m.priority).unwrap_or(0),
        }
    }

    /// Clear all messages from queue
    pub fn clear(&self) -> IpcResult<usize> {
        let mut messages = self.messages.lock();
        let count = messages.len();
        messages.clear();
        self.statistics.current_queue_size = 0;
        Ok(count)
    }

    /// Remove expired messages
    pub fn remove_expired_messages(&self, current_time: u64) -> IpcResult<usize> {
        let mut messages = self.messages.lock();
        let original_len = messages.len();
        
        // Remove messages older than 1 hour
        messages.retain(|m| current_time - m.timestamp < 3600_000_000_000); // 1 hour in nanoseconds
        
        let removed = original_len - messages.len();
        if removed > 0 {
            log::debug!("Removed {} expired messages from queue {}", removed, self.id);
        }
        
        Ok(removed)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> MessageQueueStatistics {
        self.statistics.clone()
    }

    /// Generate unique message ID
    fn generate_message_id(&self) -> u64 {
        // In real implementation, would use atomic counter
        use core::time::UNIX_EPOCH;
        let now = core::time::Duration::from_secs(0); // Would use actual time
        (now.as_nanos() as u64) ^ (self.id as u64)
    }

    /// Update receiver statistics
    fn update_receiver_stats(&self, process_id: u32, message: &Message) {
        let mut consumers = self.consumers.write();
        if let Some(consumer) = consumers.iter_mut().find(|c| c.process_id == process_id) {
            consumer.receive_count += 1;
            consumer.last_receive_time = message.timestamp;
        }
    }

    /// Wake up waiting receivers
    fn wake_up_receivers(&self) {
        let mut waiting_receivers = self.waiting_receivers.lock();
        
        if !waiting_receivers.is_empty() && !self.messages.lock().is_empty() {
            let receiver_id = waiting_receivers.remove(0);
            log::debug!("Waking up receiver: {}", receiver_id);
            // In real implementation, would wake up the actual process
        }
    }

    /// Wake up waiting senders
    fn wake_up_senders(&self) {
        let mut waiting_senders = self.waiting_senders.lock();
        
        if !waiting_senders.is_empty() && self.messages.lock().len() < self.max_messages {
            let sender_id = waiting_senders.remove(0);
            log::debug!("Waking up sender: {}", sender_id);
            // In real implementation, would wake up the actual process
        }
    }

    /// Remove process from waiting lists (cleanup)
    pub fn cleanup_process(&self, process_id: u32) -> IpcResult<()> {
        // Remove from waiting receivers
        let mut waiting_receivers = self.waiting_receivers.lock();
        if let Some(pos) = waiting_receivers.iter().position(|&id| id == process_id) {
            waiting_receivers.remove(pos);
        }

        // Remove from waiting senders
        let mut waiting_senders = self.waiting_senders.lock();
        if let Some(pos) = waiting_senders.iter().position(|&id| id == process_id) {
            waiting_senders.remove(pos);
        }

        Ok(())
    }
}

/// Message queue state information
#[derive(Debug, Clone, Default)]
pub struct QueueState {
    pub message_count: usize,
    pub total_bytes: usize,
    pub current_max_priority: u32,
}

/// Message queue manager
#[derive(Debug)]
pub struct MessageQueueManager {
    pub queues: RwLock<BTreeMap<MessageQueueId, MessageQueue>>,
    pub next_id: AtomicU32,
    pub global_statistics: MessageQueueStatistics,
}

impl MessageQueueManager {
    pub fn new() -> Self {
        Self {
            queues: RwLock::new(BTreeMap::new()),
            next_id: AtomicU32::new(1),
            global_statistics: MessageQueueStatistics::default(),
        }
    }

    pub fn create_queue(&self, name: &[u8], max_message_size: usize, max_messages: usize, flags: MessageQueueFlags) -> IpcResult<MessageQueueId> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let queue = MessageQueue::new(id, name, max_message_size, max_messages)?;
        queue.flags = flags;

        let mut queues = self.queues.write();
        queues.insert(id, queue);

        self.global_statistics.max_queue_size += 1;
        Ok(id)
    }

    pub fn open_queue(&self, name: &[u8]) -> IpcResult<MessageQueueId> {
        let queues = self.queues.read();
        
        for (&id, queue) in queues.iter() {
            if queue.name == name {
                return Ok(id);
            }
        }
        
        Err(IpcError::InvalidHandle)
    }

    pub fn get_queue(&self, id: MessageQueueId) -> Option<MessageQueue> {
        let queues = self.queues.read();
        queues.get(&id).cloned()
    }

    pub fn remove_queue(&self, id: MessageQueueId) -> IpcResult<()> {
        let mut queues = self.queues.write();
        if queues.remove(&id).is_some() {
            self.global_statistics.max_queue_size -= 1;
            Ok(())
        } else {
            Err(IpcError::InvalidHandle)
        }
    }

    pub fn get_global_statistics(&self) -> MessageQueueStatistics {
        let queues = self.queues.read();
        
        let mut total_messages_sent = 0;
        let mut total_messages_received = 0;
        let mut total_bytes_sent = 0;
        let mut total_bytes_received = 0;
        let mut total_dropped = 0;
        let mut total_consumers = 0;

        for queue in queues.values() {
            let stats = queue.get_statistics();
            total_messages_sent += stats.messages_sent;
            total_messages_received += stats.messages_received;
            total_bytes_sent += stats.bytes_sent;
            total_bytes_received += stats.bytes_received;
            total_dropped += stats.dropped_messages;
            total_consumers += stats.consumers_connected;
        }

        MessageQueueStatistics {
            messages_sent: total_messages_sent,
            messages_received: total_messages_received,
            bytes_sent: total_bytes_sent,
            bytes_received: total_bytes_received,
            dropped_messages: total_dropped,
            consumers_connected: total_consumers,
            max_queue_size: queues.len() as u32,
            current_queue_size: 0,
            errors: self.global_statistics.errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_queue_send_receive() {
        let mq = MessageQueue::new(1, b"test_queue", 1024, 10).unwrap();
        
        let data = b"Hello, Message Queue!";
        assert!(mq.try_send(data, 1, 100).is_ok());
        
        let mut buffer = vec![0u8; data.len()];
        let mut priority = 0;
        let bytes_read = mq.try_receive(&mut buffer, &mut priority, 200).unwrap();
        
        assert_eq!(bytes_read, data.len());
        assert_eq!(buffer, data);
        assert_eq!(priority, 1);
    }

    #[test]
    fn test_priority_ordering() {
        let mut mq = MessageQueue::new(1, b"test_queue", 1024, 10).unwrap();
        mq.flags.insert(MessageQueueFlags::PRIORITY_ORDER);
        
        let data1 = b"low priority";
        let data2 = b"high priority";
        let data3 = b"medium priority";
        
        assert!(mq.try_send(data1, 1, 100).is_ok());
        assert!(mq.try_send(data2, 3, 101).is_ok());
        assert!(mq.try_send(data3, 2, 102).is_ok());
        
        let mut buffer = vec![0u8; 13];
        let mut priority = 0;
        
        // Should receive high priority first
        let bytes = mq.try_receive(&mut buffer, &mut priority, 200).unwrap();
        assert_eq!(priority, 3);
        assert_eq!(&buffer[..bytes], data2);
        
        // Then medium priority
        bytes = mq.try_receive(&mut buffer, &mut priority, 200).unwrap();
        assert_eq!(priority, 2);
        assert_eq!(&buffer[..bytes], data3);
        
        // Then low priority
        bytes = mq.try_receive(&mut buffer, &mut priority, 200).unwrap();
        assert_eq!(priority, 1);
        assert_eq!(&buffer[..bytes], data1);
    }

    #[test]
    fn test_queue_full_error() {
        let mq = MessageQueue::new(1, b"test_queue", 100, 1).unwrap();
        
        // Fill the queue
        assert!(mq.try_send(b"first message", 1, 100).is_ok());
        
        // Second message should fail
        assert_eq!(mq.try_send(b"second message", 1, 101), Err(IpcError::WouldBlock));
    }
}
