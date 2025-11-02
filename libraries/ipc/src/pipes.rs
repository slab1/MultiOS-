//! Pipe Implementation for Unidirectional IPC
//! 
//! This module implements pipes for unidirectional data flow between
//! processes and threads, supporting both anonymous and named pipes.

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use spin::{Mutex, RwLock};
use bitflags::bitflags;
use alloc::vec::Vec;

use crate::{IpcResult, IpcError};

/// Pipe file descriptor
pub type PipeFd = i32;

/// Maximum pipe buffer size (64KB)
const MAX_PIPE_SIZE: usize = 65536;
const DEFAULT_PIPE_SIZE: usize = 4096;

/// Pipe flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PipeFlags: u32 {
        const NON_BLOCKING = 1 << 0;
        const BUFFERED = 1 << 1;
        const ATOMIC = 1 << 2;
        const CLOSE_ON_EXEC = 1 << 3;
        const DIRECT = 1 << 4; // Direct I/O (no buffering)
    }
}

/// Pipe end types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PipeEnd {
    Read = 0,
    Write = 1,
}

/// Pipe buffer with ring structure
#[derive(Debug)]
struct PipeBuffer {
    buffer: Vec<u8>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
    available_bytes: AtomicUsize,
    read_waiters: Mutex<Vec<u32>>,
    write_waiters: Mutex<Vec<u32>>,
}

impl PipeBuffer {
    pub fn new(size: usize) -> Self {
        let capacity = size.min(MAX_PIPE_SIZE).max(256);
        Self {
            buffer: vec![0; capacity],
            capacity,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            available_bytes: AtomicUsize::new(0),
            read_waiters: Mutex::new(Vec::new()),
            write_waiters: Mutex::new(Vec::new()),
        }
    }

    /// Write data to pipe buffer
    pub fn write(&self, data: &[u8], flags: PipeFlags) -> IpcResult<usize> {
        if data.is_empty() {
            return Ok(0);
        }

        let available = self.available_bytes.load(Ordering::SeqCst);
        let data_len = data.len();
        
        // Check if we can write the data
        if available + data_len > self.capacity {
            if flags.contains(PipeFlags::NON_BLOCKING) {
                return Err(IpcError::WouldBlock);
            }
            
            // Block until space is available
            let current_process = 0; // In real implementation, get current process ID
            self.write_waiters.lock().push(current_process);
            
            // In real implementation, would block here
            log::debug!("Process {} waiting for pipe space", current_process);
            
            return Ok(0); // Would block in real implementation
        }

        // Perform write
        let mut head = self.head.load(Ordering::SeqCst);
        let mut tail = self.tail.load(Ordering::SeqCst);
        
        // Handle wrap-around
        if head + data_len > self.capacity {
            let first_chunk = self.capacity - head;
            let second_chunk = data_len - first_chunk;
            
            self.buffer[head..].copy_from_slice(&data[..first_chunk]);
            self.buffer[..second_chunk].copy_from_slice(&data[first_chunk..]);
            head = second_chunk;
        } else {
            self.buffer[head..head + data_len].copy_from_slice(data);
            head = (head + data_len) % self.capacity;
        }

        self.head.store(head, Ordering::SeqCst);
        self.available_bytes.fetch_add(data_len, Ordering::SeqCst);

        // Wake up read waiters
        self.wake_up_readers();

        Ok(data_len)
    }

    /// Read data from pipe buffer
    pub fn read(&self, buffer: &mut [u8], flags: PipeFlags) -> IpcResult<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let available = self.available_bytes.load(Ordering::SeqCst);
        
        if available == 0 {
            if flags.contains(PipeFlags::NON_BLOCKING) {
                return Err(IpcError::WouldBlock);
            }
            
            // Block until data is available
            let current_process = 0; // In real implementation, get current process ID
            self.read_waiters.lock().push(current_process);
            
            // In real implementation, would block here
            log::debug!("Process {} waiting for pipe data", current_process);
            
            return Ok(0); // Would block in real implementation
        }

        let bytes_to_read = available.min(buffer.len());
        
        // Perform read
        let tail = self.tail.load(Ordering::SeqCst);
        
        // Handle wrap-around
        if tail + bytes_to_read > self.capacity {
            let first_chunk = self.capacity - tail;
            let second_chunk = bytes_to_read - first_chunk;
            
            buffer[..first_chunk].copy_from_slice(&self.buffer[tail..]);
            buffer[first_chunk..].copy_from_slice(&self.buffer[..second_chunk]);
        } else {
            buffer[..bytes_to_read].copy_from_slice(&self.buffer[tail..tail + bytes_to_read]);
        }

        let new_tail = (tail + bytes_to_read) % self.capacity;
        self.tail.store(new_tail, Ordering::SeqCst);
        self.available_bytes.fetch_sub(bytes_to_read, Ordering::SeqCst);

        // Wake up write waiters
        self.wake_up_writers();

        Ok(bytes_to_read)
    }

    /// Get available space for writing
    pub fn available_write_space(&self) -> usize {
        self.capacity - self.available_bytes.load(Ordering::SeqCst)
    }

    /// Get available data for reading
    pub fn available_read_data(&self) -> usize {
        self.available_bytes.load(Ordering::SeqCst)
    }

    /// Wake up waiting readers
    fn wake_up_readers(&self) {
        let mut waiters = self.read_waiters.lock();
        if !waiters.is_empty() {
            let waiter = waiters.remove(0);
            log::debug!("Waking up pipe reader: {}", waiter);
            // In real implementation, would wake up the actual process
        }
    }

    /// Wake up waiting writers
    fn wake_up_writers(&self) {
        let mut waiters = self.write_waiters.lock();
        if !waiters.is_empty() && self.available_write_space() > 0 {
            let waiter = waiters.remove(0);
            log::debug!("Waking up pipe writer: {}", waiter);
            // In real implementation, would wake up the actual process
        }
    }

    /// Check if pipe is empty
    pub fn is_empty(&self) -> bool {
        self.available_bytes.load(Ordering::SeqCst) == 0
    }

    /// Check if pipe is full
    pub fn is_full(&self) -> bool {
        self.available_bytes.load(Ordering::SeqCst) >= self.capacity
    }

    /// Clear the pipe buffer
    pub fn clear(&self) {
        self.head.store(0, Ordering::SeqCst);
        self.tail.store(0, Ordering::SeqCst);
        self.available_bytes.store(0, Ordering::SeqCst);
    }
}

/// Pipe statistics
#[derive(Debug, Clone, Default)]
pub struct PipeStatistics {
    pub bytes_written: u64,
    pub bytes_read: u64,
    pub write_operations: u64,
    pub read_operations: u64,
    pub blocked_reads: u32,
    pub blocked_writes: u32,
    pub errors: u32,
}

/// Pipe implementation
#[derive(Debug)]
pub struct Pipe {
    pub id: u32,
    pub buffer: PipeBuffer,
    pub flags: PipeFlags,
    pub capacity: usize,
    pub created_by: u32,
    pub created_at: u64,
    pub read_end_owner: AtomicU32,
    pub write_end_owner: AtomicU32,
    pub read_count: AtomicU32,
    pub write_count: AtomicU32,
    pub statistics: PipeStatistics,
    pub name: Option<Vec<u8>>, // For named pipes
}

impl Pipe {
    pub fn new(id: u32, capacity: usize, flags: PipeFlags) -> IpcResult<Self> {
        let capacity = capacity.min(MAX_PIPE_SIZE).max(256);
        
        Ok(Self {
            id,
            buffer: PipeBuffer::new(capacity),
            flags,
            capacity,
            created_by: 0,
            created_at: 0,
            read_end_owner: AtomicU32::new(0),
            write_end_owner: AtomicU32::new(0),
            read_count: AtomicU32::new(0),
            write_count: AtomicU32::new(0),
            statistics: PipeStatistics::default(),
            name: None,
        })
    }

    /// Write data to the write end of the pipe
    pub fn write(&self, data: &[u8], process_id: u32) -> IpcResult<usize> {
        // Check if process has write permission
        if self.write_end_owner.load(Ordering::SeqCst) != process_id {
            // In real implementation, check process permissions
        }

        let bytes_written = self.buffer.write(data, self.flags)?;
        self.statistics.write_operations += 1;
        self.statistics.bytes_written += bytes_written as u64;
        self.write_count.fetch_add(1, Ordering::SeqCst);

        log::debug!("Process {} wrote {} bytes to pipe {}", process_id, bytes_written, self.id);
        Ok(bytes_written)
    }

    /// Read data from the read end of the pipe
    pub fn read(&self, buffer: &mut [u8], process_id: u32) -> IpcResult<usize> {
        // Check if process has read permission
        if self.read_end_owner.load(Ordering::SeqCst) != process_id {
            // In real implementation, check process permissions
        }

        let bytes_read = self.buffer.read(buffer, self.flags)?;
        self.statistics.read_operations += 1;
        self.statistics.bytes_read += bytes_read as u64;
        self.read_count.fetch_add(1, Ordering::SeqCst);

        log::debug!("Process {} read {} bytes from pipe {}", process_id, bytes_read, self.id);
        Ok(bytes_read)
    }

    /// Get the read end file descriptor
    pub fn read_fd(&self) -> PipeFd {
        (self.id << 1) | (PipeEnd::Read as i32)
    }

    /// Get the write end file descriptor
    pub fn write_fd(&self) -> PipeFd {
        (self.id << 1) | (PipeEnd::Write as i32)
    }

    /// Check if both ends of the pipe are still open
    pub fn is_broken(&self) -> bool {
        self.read_count.load(Ordering::SeqCst) == 0 || self.write_count.load(Ordering::SeqCst) == 0
    }

    /// Get current buffer state
    pub fn buffer_state(&self) -> BufferState {
        BufferState {
            available_data: self.buffer.available_read_data(),
            available_space: self.buffer.available_write_space(),
            total_capacity: self.capacity,
        }
    }

    /// Set pipe name (for named pipes)
    pub fn set_name(&mut self, name: &[u8]) {
        self.name = Some(name.to_vec());
    }

    /// Get pipe name
    pub fn get_name(&self) -> Option<&[u8]> {
        self.name.as_ref().map(|v| v.as_slice())
    }

    /// Flush the pipe (ensure all data is written)
    pub fn flush(&self) -> IpcResult<()> {
        // In buffered pipes, flush would write remaining buffered data
        // For direct I/O pipes, this is a no-op
        if self.flags.contains(PipeFlags::BUFFERED) {
            // Would flush actual buffer in real implementation
            log::debug!("Flushing pipe {}", self.id);
        }
        Ok(())
    }

    /// Get statistics
    pub fn get_statistics(&self) -> PipeStatistics {
        self.statistics.clone()
    }

    /// Update read end owner
    pub fn set_read_owner(&self, process_id: u32) {
        self.read_end_owner.store(process_id, Ordering::SeqCst);
        self.read_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Update write end owner
    pub fn set_write_owner(&self, process_id: u32) {
        self.write_end_owner.store(process_id, Ordering::SeqCst);
        self.write_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Close read end
    pub fn close_read_end(&self, process_id: u32) -> IpcResult<()> {
        if self.read_end_owner.load(Ordering::SeqCst) == process_id {
            self.read_count.fetch_sub(1, Ordering::SeqCst);
            log::debug!("Process {} closed read end of pipe {}", process_id, self.id);
            Ok(())
        } else {
            Err(IpcError::PermissionDenied)
        }
    }

    /// Close write end
    pub fn close_write_end(&self, process_id: u32) -> IpcResult<()> {
        if self.write_end_owner.load(Ordering::SeqCst) == process_id {
            self.write_count.fetch_sub(1, Ordering::SeqCst);
            log::debug!("Process {} closed write end of pipe {}", process_id, self.id);
            Ok(())
        } else {
            Err(IpcError::PermissionDenied)
        }
    }
}

/// Buffer state information
#[derive(Debug, Clone, Default)]
pub struct BufferState {
    pub available_data: usize,
    pub available_space: usize,
    pub total_capacity: usize,
}

/// Pipe manager for handling multiple pipes
#[derive(Debug)]
pub struct PipeManager {
    pub pipes: RwLock<Vec<Pipe>>,
    pub named_pipes: RwLock<Vec<Pipe>>, // Separate storage for named pipes
    pub next_id: AtomicU32,
    pub statistics: PipeStatistics,
}

impl PipeManager {
    pub fn new() -> Self {
        Self {
            pipes: RwLock::new(Vec::new()),
            named_pipes: RwLock::new(Vec::new()),
            next_id: AtomicU32::new(1),
            statistics: PipeStatistics::default(),
        }
    }

    /// Create an anonymous pipe
    pub fn create_pipe(&self, capacity: usize, flags: PipeFlags) -> IpcResult<Pipe> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let pipe = Pipe::new(id, capacity, flags)?;
        
        let mut pipes = self.pipes.write();
        pipes.push(pipe);
        let created_pipe = pipes.last().unwrap().clone();
        
        Ok(created_pipe)
    }

    /// Create a named pipe
    pub fn create_named_pipe(&self, name: &[u8], capacity: usize, flags: PipeFlags) -> IpcResult<Pipe> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut pipe = Pipe::new(id, capacity, flags)?;
        pipe.set_name(name);
        
        let mut named_pipes = self.named_pipes.write();
        named_pipes.push(pipe);
        let created_pipe = named_pipes.last().unwrap().clone();
        
        Ok(created_pipe)
    }

    /// Open an existing named pipe
    pub fn open_named_pipe(&self, name: &[u8]) -> IpcResult<Pipe> {
        let named_pipes = self.named_pipes.read();
        
        for pipe in named_pipes.iter() {
            if let Some(pipe_name) = pipe.get_name() {
                if pipe_name == name {
                    return Ok(pipe.clone());
                }
            }
        }
        
        Err(IpcError::InvalidHandle)
    }

    /// Get pipe by file descriptor
    pub fn get_pipe_by_fd(&self, fd: PipeFd) -> IpcResult<Pipe> {
        let pipe_id = (fd >> 1) as u32;
        let is_read_end = (fd & 1) == (PipeEnd::Read as i32);
        
        let pipes = self.pipes.read();
        let named_pipes = self.named_pipes.read();
        
        // Check anonymous pipes first
        for pipe in pipes.iter().chain(named_pipes.iter()) {
            if pipe.id == pipe_id {
                if is_read_end && pipe.read_count.load(Ordering::SeqCst) == 0 {
                    return Err(IpcError::InvalidHandle);
                }
                if !is_read_end && pipe.write_count.load(Ordering::SeqCst) == 0 {
                    return Err(IpcError::InvalidHandle);
                }
                return Ok(pipe.clone());
            }
        }
        
        Err(IpcError::InvalidHandle)
    }

    /// Close and remove pipe
    pub fn close_pipe(&self, pipe_id: u32) -> IpcResult<()> {
        let mut pipes = self.pipes.write();
        let mut named_pipes = self.named_pipes.write();
        
        // Check anonymous pipes
        if let Some(pos) = pipes.iter().position(|p| p.id == pipe_id) {
            let pipe = &pipes[pos];
            if pipe.read_count.load(Ordering::SeqCst) == 0 && pipe.write_count.load(Ordering::SeqCst) == 0 {
                pipes.remove(pos);
                return Ok(());
            }
        }
        
        // Check named pipes
        if let Some(pos) = named_pipes.iter().position(|p| p.id == pipe_id) {
            let pipe = &named_pipes[pos];
            if pipe.read_count.load(Ordering::SeqCst) == 0 && pipe.write_count.load(Ordering::SeqCst) == 0 {
                named_pipes.remove(pos);
                return Ok(());
            }
        }
        
        Err(IpcError::InvalidHandle)
    }

    /// Get global pipe statistics
    pub fn get_global_statistics(&self) -> PipeStatistics {
        let pipes = self.pipes.read();
        let named_pipes = self.named_pipes.read();
        
        let mut total_bytes_written = 0;
        let mut total_bytes_read = 0;
        let mut total_writes = 0;
        let mut total_reads = 0;
        let mut total_blocked_reads = 0;
        let mut total_blocked_writes = 0;
        let mut total_errors = 0;
        
        for pipe in pipes.iter().chain(named_pipes.iter()) {
            let stats = pipe.get_statistics();
            total_bytes_written += stats.bytes_written;
            total_bytes_read += stats.bytes_read;
            total_writes += stats.write_operations;
            total_reads += stats.read_operations;
            total_blocked_reads += stats.blocked_reads;
            total_blocked_writes += stats.blocked_writes;
            total_errors += stats.errors;
        }
        
        PipeStatistics {
            bytes_written: total_bytes_written,
            bytes_read: total_bytes_read,
            write_operations: total_writes,
            read_operations: total_reads,
            blocked_reads: total_blocked_reads,
            blocked_writes: total_blocked_writes,
            errors: total_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_write_read() {
        let pipe_manager = PipeManager::new();
        let pipe = pipe_manager.create_pipe(1024, PipeFlags::empty()).unwrap();
        
        let data = b"Hello, Pipe!";
        assert_eq!(pipe.write(data, 100).unwrap(), data.len());
        
        let mut buffer = vec![0u8; data.len()];
        assert_eq!(pipe.read(&mut buffer, 101).unwrap(), data.len());
        assert_eq!(buffer, data);
    }

    #[test]
    fn test_named_pipe() {
        let pipe_manager = PipeManager::new();
        let pipe = pipe_manager.create_named_pipe(b"my_pipe", 1024, PipeFlags::empty()).unwrap();
        
        assert_eq!(pipe.get_name(), Some(b"my_pipe" as &[u8]));
        
        let opened_pipe = pipe_manager.open_named_pipe(b"my_pipe").unwrap();
        assert_eq!(opened_pipe.get_name(), Some(b"my_pipe" as &[u8]));
    }

    #[test]
    fn test_pipe_buffer_state() {
        let pipe_manager = PipeManager::new();
        let pipe = pipe_manager.create_pipe(1024, PipeFlags::empty()).unwrap();
        
        let state = pipe.buffer_state();
        assert_eq!(state.total_capacity, 1024);
        assert_eq!(state.available_data, 0);
        assert_eq!(state.available_space, 1024);
        
        // Write some data
        let data = b"test";
        pipe.write(data, 100).unwrap();
        
        let state = pipe.buffer_state();
        assert_eq!(state.available_data, 4);
        assert_eq!(state.available_space, 1020);
    }

    #[test]
    fn test_pipe_non_blocking() {
        let pipe_manager = PipeManager::new();
        let pipe = pipe_manager.create_pipe(10, PipeFlags::NON_BLOCKING).unwrap();
        
        // Fill the pipe
        let data = b"1234567890";
        assert_eq!(pipe.write(data, 100).unwrap(), 10);
        
        // Try to write more - should fail
        assert_eq!(pipe.write(b"extra", 100), Err(IpcError::WouldBlock));
    }
}
