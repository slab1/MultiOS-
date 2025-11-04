//! Inter-Process Communication Channels
//! 
//! This module implements message-passing channels for communication
//! between processes and threads with proper synchronization.

use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use spin::{Mutex, RwLock};
use bitflags::bitflags;

use crate::{IpcResult, IpcError};

/// Channel ID type
pub type ChannelId = u32;

/// Maximum channel buffer size
const MAX_CHANNEL_SIZE: usize = 1024 * 1024; // 1MB
const DEFAULT_CHANNEL_SIZE: usize = 4096; // 4KB

/// Channel handle for user-space access
#[derive(Debug, Clone, Copy)]
pub struct ChannelHandle {
    pub id: ChannelId,
}

impl ChannelHandle {
    pub const fn new(id: ChannelId) -> Self {
        Self { id }
    }
}

/// Channel message structure
#[derive(Debug, Clone)]
pub struct ChannelMessage {
    pub data: Vec<u8>,
    pub sender_id: u32,
    pub timestamp: u64,
    pub message_id: u64,
}

/// Channel buffer with circular queue
struct ChannelBuffer {
    buffer: Vec<u8>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    message_count: AtomicU32,
}

impl ChannelBuffer {
    pub fn new(size: usize) -> Self {
        let capacity = size.min(MAX_CHANNEL_SIZE).max(64);
        Self {
            buffer: vec![0; capacity],
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            message_count: AtomicU32::new(0),
        }
    }

    pub fn write(&self, data: &[u8]) -> IpcResult<usize> {
        if data.len() + 8 > self.capacity { // 8 bytes for header
            return Err(IpcError::BufferTooSmall);
        }

        let mut head = self.head.load(Ordering::SeqCst);
        let mut tail = self.tail.load(Ordering::SeqCst);

        // Check if we have enough space
        let available_space = if head <= tail {
            self.capacity - tail + head - 8
        } else {
            head - tail - 8
        };

        if data.len() + 8 > available_space {
            return Err(IpcError::ResourceExhausted);
        }

        // Write message header (length)
        let len_bytes = (data.len() as u32).to_le_bytes();
        for (i, &byte) in len_bytes.iter().enumerate() {
            self.buffer[tail] = byte;
            tail = (tail + 1) % self.capacity;
        }

        // Write message data
        for &byte in data {
            self.buffer[tail] = byte;
            tail = (tail + 1) % self.capacity;
        }

        self.head.store(head, Ordering::SeqCst);
        self.tail.store(tail, Ordering::SeqCst);
        self.message_count.fetch_add(1, Ordering::SeqCst);

        Ok(data.len())
    }

    pub fn read(&self) -> IpcResult<Option<Vec<u8>>> {
        let head = self.head.load(Ordering::SeqCst);
        let mut tail = self.tail.load(Ordering::SeqCst);

        if head == tail {
            return Ok(None); // Empty
        }

        // Read message length
        let len_bytes = [
            self.buffer[tail],
            self.buffer[(tail + 1) % self.capacity],
            self.buffer[(tail + 2) % self.capacity],
            self.buffer[(tail + 3) % self.capacity],
        ];
        
        let message_len = u32::from_le_bytes(len_bytes) as usize;
        tail = (tail + 4) % self.capacity;

        if message_len > self.buffer.len() {
            return Err(IpcError::BufferTooSmall);
        }

        // Read message data
        let mut data = Vec::with_capacity(message_len);
        for _ in 0..message_len {
            data.push(self.buffer[tail]);
            tail = (tail + 1) % self.capacity;
        }

        self.tail.store(tail, Ordering::SeqCst);
        self.message_count.fetch_sub(1, Ordering::SeqCst);

        Ok(Some(data))
    }

    pub fn available_space(&self) -> usize {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);
        
        if head == tail {
            self.capacity - 8
        } else if head < tail {
            tail - head - 8
        } else {
            self.capacity - head + tail - 8
        }
    }

    pub fn message_count(&self) -> u32 {
        self.message_count.load(Ordering::SeqCst)
    }
}

/// Channel state and flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ChannelFlags: u32 {
        const NON_BLOCKING = 1 << 0;
        const AUTO_CREATE   = 1 << 1;
        const BROADCAST     = 1 << 2;
        const PRIORITY      = 1 << 3;
        const SYSTEM        = 1 << 4;
    }
}

/// IPC Channel implementation
#[derive(Debug)]
pub struct Channel {
    pub id: ChannelId,
    pub buffer: ChannelBuffer,
    pub flags: ChannelFlags,
    pub created_by: u32,
    pub created_at: u64,
    pub connections: Mutex<Vec<u32>>,
    pub waiting_readers: Mutex<Vec<u32>>,
    pub waiting_writers: Mutex<Vec<u32>>,
    pub statistics: ChannelStatistics,
}

/// Channel statistics
#[derive(Debug, Clone, Default)]
pub struct ChannelStatistics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u32,
}

impl Channel {
    pub fn new(id: ChannelId, buffer_size: usize) -> IpcResult<Self> {
        let buffer_size = buffer_size.max(64).min(MAX_CHANNEL_SIZE);
        
        Ok(Self {
            id,
            buffer: ChannelBuffer::new(buffer_size),
            flags: ChannelFlags::empty(),
            created_by: 0, // Will be set by IPC manager
            created_at: 0, // Will be set by IPC manager
            connections: Mutex::new(Vec::new()),
            waiting_readers: Mutex::new(Vec::new()),
            waiting_writers: Mutex::new(Vec::new()),
            statistics: ChannelStatistics::default(),
        })
    }

    /// Send a message to the channel
    pub fn send(&self, data: &[u8], sender_id: u32) -> IpcResult<usize> {
        // Check if sender is connected
        let connections = self.connections.lock();
        if !connections.contains(&sender_id) && self.created_by != sender_id {
            return Err(IpcError::PermissionDenied);
        }

        // Try to write to buffer
        match self.buffer.write(data) {
            Ok(bytes_written) => {
                drop(connections);
                let mut stats = self.statistics;
                stats.messages_sent += 1;
                stats.bytes_sent += bytes_written as u64;
                
                // Wake up waiting readers
                self.wake_up_readers();
                
                Ok(bytes_written)
            }
            Err(e) => {
                drop(connections);
                let mut stats = self.statistics;
                stats.errors += 1;
                Err(e)
            }
        }
    }

    /// Receive a message from the channel
    pub fn receive(&self, receiver_id: u32) -> IpcResult<Option<Vec<u8>>> {
        // Check if receiver is connected
        let connections = self.connections.lock();
        if !connections.contains(&receiver_id) && self.created_by != receiver_id {
            return Err(IpcError::PermissionDenied);
        }

        match self.buffer.read() {
            Ok(data) => {
                if let Some(ref msg_data) = data {
                    let mut stats = self.statistics;
                    stats.messages_received += 1;
                    stats.bytes_received += msg_data.len() as u64;
                }
                
                drop(connections);
                Ok(data)
            }
            Err(e) => {
                drop(connections);
                let mut stats = self.statistics;
                stats.errors += 1;
                Err(e)
            }
        }
    }

    /// Connect a process to this channel
    pub fn connect(&self, process_id: u32) -> IpcResult<()> {
        let mut connections = self.connections.lock();
        if !connections.contains(&process_id) {
            connections.push(process_id);
        }
        Ok(())
    }

    /// Disconnect a process from this channel
    pub fn disconnect(&self, process_id: u32) -> IpcResult<()> {
        let mut connections = self.connections.lock();
        if let Some(pos) = connections.iter().position(|&id| id == process_id) {
            connections.remove(pos);
        }
        Ok(())
    }

    /// Check if channel has available space
    pub fn has_space(&self) -> bool {
        self.buffer.available_space() > 0
    }

    /// Get channel statistics
    pub fn get_statistics(&self) -> ChannelStatistics {
        self.statistics.clone()
    }

    /// Wake up waiting readers
    fn wake_up_readers(&self) {
        let waiting_readers = self.waiting_readers.lock();
        for &reader_id in waiting_readers.iter() {
            // In a real implementation, this would wake up the actual threads
            // For now, we just log the wake-up
            log::debug!("Waking up reader: {}", reader_id);
        }
    }

    /// Wake up waiting writers
    fn wake_up_writers(&self) {
        let waiting_writers = self.waiting_writers.lock();
        for &writer_id in waiting_writers.iter() {
            // In a real implementation, this would wake up the actual threads
            log::debug!("Waking up writer: {}", writer_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_buffer_write_read() {
        let buffer = ChannelBuffer::new(1024);
        
        let test_data = b"Hello, World!";
        assert!(buffer.write(test_data).is_ok());
        
        let read_data = buffer.read().unwrap();
        assert_eq!(read_data, Some(test_data.to_vec()));
    }

    #[test]
    fn test_channel_statistics() {
        let channel = Channel::new(1, 1024).unwrap();
        
        // Test send/receive statistics
        let data = b"test message";
        let _ = channel.send(data, 100);
        let _ = channel.receive(101);
        
        let stats = channel.get_statistics();
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.bytes_sent, 12);
        assert_eq!(stats.bytes_received, 12);
    }

    #[test]
    fn test_channel_connection() {
        let channel = Channel::new(1, 1024).unwrap();
        
        // Test connection
        assert!(channel.connect(100).is_ok());
        
        let connections = channel.connections.lock();
        assert!(connections.contains(&100));
    }
}