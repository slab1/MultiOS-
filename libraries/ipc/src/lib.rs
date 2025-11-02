//! MultiOS Inter-Process Communication System
//! 
//! This module provides IPC mechanisms for communication between processes
//! and threads in the MultiOS hybrid microkernel architecture.

#![no_std]

use spin::Mutex;
use bitflags::bitflags;

pub mod channels;
pub mod shared_memory;
pub mod semaphores;
pub mod message_queue;
pub mod pipes;
pub mod signals;
pub mod events;
pub mod network;

#[cfg(test)]
mod tests;

#[cfg(feature = "examples")]
pub mod examples;

/// IPC mechanism types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IpcType {
    Channel = 0,
    SharedMemory = 1,
    Semaphore = 2,
    MessageQueue = 3,
    Signal = 4,
    Pipe = 5,
    Socket = 6,
}

/// IPC result type
pub type IpcResult<T> = Result<T, IpcError>;

/// Error types for IPC operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    InvalidHandle,
    PermissionDenied,
    ResourceExhausted,
    Timeout,
    NoSuchProcess,
    BufferTooSmall,
    WouldBlock,
    Interrupted,
    NotConnected,
}

/// Global IPC manager
pub static IPC_MANAGER: Mutex<Option<ipc_manager::IpcManager>> = Mutex::new(None);

/// Initialize the IPC system
/// 
/// This function sets up the global IPC manager and must be called
/// during kernel initialization.
pub fn init() -> IpcResult<()> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = ipc_manager::IpcManager::new();
    *manager_guard = Some(manager);
    
    Ok(())
}

/// Create a new IPC channel
pub fn create_channel(buffer_size: usize) -> IpcResult<channels::ChannelHandle> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_channel(buffer_size)
}

/// Open an existing IPC channel
pub fn open_channel(channel_id: u32) -> IpcResult<channels::ChannelHandle> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.open_channel(channel_id)
}

/// Create shared memory region
pub fn create_shared_memory(size: usize) -> IpcResult<shared_memory::SharedMemoryHandle> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_shared_memory(size)
}

/// Create a semaphore
pub fn create_semaphore(initial_value: u32) -> IpcResult<semaphores::SemaphoreHandle> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_semaphore(initial_value)
}

/// Create a pipe
pub fn create_pipe(buffer_size: usize, flags: pipes::PipeFlags) -> IpcResult<pipes::Pipe> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_pipe(buffer_size, flags)
}

/// Create a named pipe
pub fn create_named_pipe(name: &[u8], buffer_size: usize, flags: pipes::PipeFlags) -> IpcResult<pipes::Pipe> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_named_pipe(name, buffer_size, flags)
}

/// Create a message queue
pub fn create_message_queue(name: &[u8], max_message_size: usize, max_messages: usize, flags: message_queue::MessageQueueFlags) -> IpcResult<message_queue::MessageQueueId> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_message_queue(name, max_message_size, max_messages, flags)
}

/// Create an event
pub fn create_event(event_type: events::EventType, flags: events::EventFlags) -> IpcResult<events::EventHandle> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_event(event_type, flags)
}

/// Create a named event
pub fn create_named_event(name: &[u8], event_type: events::EventType, flags: events::EventFlags) -> IpcResult<events::EventHandle> {
    let mut manager_guard = IPC_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(IpcError::InvalidHandle)?;
        
    manager.create_named_event(name, event_type, flags)
}

/// Number of active IPC connections
pub fn get_connection_count() -> usize {
    let manager_guard = IPC_MANAGER.lock();
    
    if let Some(manager) = manager_guard.as_ref() {
        manager.get_connection_count()
    } else {
        0
    }
}

// Internal module for managing IPC resources
mod ipc_manager {
    use super::*;
    use spin::RwLock;
    use core::collections::BTreeMap;

    /// Global IPC resource manager
    pub struct IpcManager {
        channels: RwLock<BTreeMap<u32, channels::Channel>>,
        shared_memory: RwLock<BTreeMap<u32, shared_memory::SharedMemory>>,
        semaphores: RwLock<BTreeMap<u32, semaphores::Semaphore>>,
        pipes: RwLock<BTreeMap<u32, pipes::Pipe>>,
        message_queues: RwLock<BTreeMap<u32, message_queue::MessageQueue>>,
        events: RwLock<BTreeMap<u32, events::Event>>,
        next_channel_id: AtomicU32,
        next_shm_id: AtomicU32,
        next_sem_id: AtomicU32,
        next_pipe_id: AtomicU32,
        next_mq_id: AtomicU32,
        next_event_id: AtomicU32,
    }

    impl IpcManager {
        pub fn new() -> Self {
            IpcManager {
                channels: RwLock::new(BTreeMap::new()),
                shared_memory: RwLock::new(BTreeMap::new()),
                semaphores: RwLock::new(BTreeMap::new()),
                pipes: RwLock::new(BTreeMap::new()),
                message_queues: RwLock::new(BTreeMap::new()),
                events: RwLock::new(BTreeMap::new()),
                next_channel_id: AtomicU32::new(1),
                next_shm_id: AtomicU32::new(1),
                next_sem_id: AtomicU32::new(1),
                next_pipe_id: AtomicU32::new(1),
                next_mq_id: AtomicU32::new(1),
                next_event_id: AtomicU32::new(1),
            }
        }

        pub fn create_channel(&self, buffer_size: usize) -> IpcResult<channels::ChannelHandle> {
            let id = self.next_channel_id.fetch_add(1, Ordering::SeqCst);
            let channel = channels::Channel::new(id, buffer_size)?;
            
            let mut channels = self.channels.write();
            channels.insert(id, channel);
            
            Ok(channels::ChannelHandle::new(id))
        }

        pub fn open_channel(&self, channel_id: u32) -> IpcResult<channels::ChannelHandle> {
            let channels = self.channels.read();
            if !channels.contains_key(&channel_id) {
                return Err(IpcError::InvalidHandle);
            }
            Ok(channels::ChannelHandle::new(channel_id))
        }

        pub fn create_shared_memory(&self, size: usize) -> IpcResult<shared_memory::SharedMemoryHandle> {
            let id = self.next_shm_id.fetch_add(1, Ordering::SeqCst);
            let shm = shared_memory::SharedMemory::new(id, size)?;
            
            let mut shared_memory = self.shared_memory.write();
            shared_memory.insert(id, shm);
            
            Ok(shared_memory::SharedMemoryHandle::new(id))
        }

        pub fn create_semaphore(&self, initial_value: u32) -> IpcResult<semaphores::SemaphoreHandle> {
            let id = self.next_sem_id.fetch_add(1, Ordering::SeqCst);
            let semaphore = semaphores::Semaphore::new(id, initial_value)?;
            
            let mut semaphores = self.semaphores.write();
            semaphores.insert(id, semaphore);
            
            Ok(semaphores::SemaphoreHandle::new(id))
        }

        pub fn get_connection_count(&self) -> usize {
            let channels = self.channels.read();
            let shared_memory = self.shared_memory.read();
            let semaphores = self.semaphores.read();
            let pipes = self.pipes.read();
            let message_queues = self.message_queues.read();
            let events = self.events.read();
            
            channels.len() + shared_memory.len() + semaphores.len() + 
            pipes.len() + message_queues.len() + events.len()
        }

        pub fn create_pipe(&self, buffer_size: usize, flags: pipes::PipeFlags) -> IpcResult<pipes::Pipe> {
            let id = self.next_pipe_id.fetch_add(1, Ordering::SeqCst);
            let pipe = pipes::Pipe::new(id, buffer_size, flags)?;
            
            let mut pipes = self.pipes.write();
            pipes.insert(id, pipe);
            
            Ok(pipes.get(&id).unwrap().clone())
        }

        pub fn create_named_pipe(&self, name: &[u8], buffer_size: usize, flags: pipes::PipeFlags) -> IpcResult<pipes::Pipe> {
            let id = self.next_pipe_id.fetch_add(1, Ordering::SeqCst);
            let mut pipe = pipes::Pipe::new(id, buffer_size, flags)?;
            pipe.set_name(name);
            
            let mut pipes = self.pipes.write();
            pipes.insert(id, pipe);
            
            Ok(pipes.get(&id).unwrap().clone())
        }

        pub fn create_message_queue(&self, name: &[u8], max_message_size: usize, max_messages: usize, flags: message_queue::MessageQueueFlags) -> IpcResult<message_queue::MessageQueueId> {
            let id = self.next_mq_id.fetch_add(1, Ordering::SeqCst);
            let mq = message_queue::MessageQueue::new(id, name, max_message_size, max_messages)?;
            mq.flags = flags;
            
            let mut message_queues = self.message_queues.write();
            message_queues.insert(id, mq);
            
            Ok(id)
        }

        pub fn create_event(&self, event_type: events::EventType, flags: events::EventFlags) -> IpcResult<events::EventHandle> {
            let id = self.next_event_id.fetch_add(1, Ordering::SeqCst);
            let event = events::Event::new(id, event_type, flags);
            
            let mut events = self.events.write();
            events.insert(id, event);
            
            Ok(events::EventHandle::new(id))
        }

        pub fn create_named_event(&self, name: &[u8], event_type: events::EventType, flags: events::EventFlags) -> IpcResult<events::EventHandle> {
            let id = self.next_event_id.fetch_add(1, Ordering::SeqCst);
            let mut event = events::Event::new(id, event_type, flags);
            event.set_name(name);
            
            let mut events = self.events.write();
            events.insert(id, event);
            
            Ok(events::EventHandle::new(id))
        }
    }

    // Atomic counter for thread-safe ID generation
    use core::sync::atomic::{AtomicU32, Ordering};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_type_ordering() {
        assert_eq!(IpcType::Channel as u8, 0);
        assert_eq!(IpcType::SharedMemory as u8, 1);
        assert_eq!(IpcType::Socket as u8, 6);
    }

    #[test]
    fn test_ipc_error_variants() {
        let errors = [
            IpcError::InvalidHandle,
            IpcError::PermissionDenied,
            IpcError::ResourceExhausted,
            IpcError::Timeout,
            IpcError::NoSuchProcess,
            IpcError::BufferTooSmall,
            IpcError::WouldBlock,
            IpcError::Interrupted,
            IpcError::NotConnected,
        ];
        
        for (i, &error) in errors.iter().enumerate() {
            assert_eq!(error as usize, i);
        }
    }
}