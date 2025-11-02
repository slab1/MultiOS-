//! Shared Memory Management for IPC
//! 
//! This module implements shared memory segments that can be mapped
//! into multiple process address spaces for high-performance IPC.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use spin::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use bitflags::bitflags;
use alloc::collections::BTreeMap;

use crate::{IpcResult, IpcError};

/// Shared memory segment ID
pub type SharedMemoryId = u32;

/// Maximum shared memory size (16MB)
const MAX_SHM_SIZE: usize = 16 * 1024 * 1024;
const DEFAULT_SHM_SIZE: usize = 4096;

/// Shared memory handle for user-space access
#[derive(Debug, Clone, Copy)]
pub struct SharedMemoryHandle {
    pub id: SharedMemoryId,
}

impl SharedMemoryHandle {
    pub const fn new(id: SharedMemoryId) -> Self {
        Self { id }
    }
}

/// Memory permissions
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MemoryPermissions: u32 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXECUTE = 1 << 2;
        const SECURE  = 1 << 3;
    }
}

/// Shared memory flags
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SharedMemoryFlags: u32 {
        const ANONYMOUS = 1 << 0;
        const TEMPORARY = 1 << 1;
        const PERSISTENT = 1 << 2;
        const LOCKED = 1 << 3;
        const HUGE = 1 << 4;
        const NORESERVE = 1 << 5;
    }
}

/// Memory mapping information
#[derive(Debug, Clone)]
pub struct MemoryMapping {
    pub process_id: u32,
    pub base_address: usize,
    pub size: usize,
    pub permissions: MemoryPermissions,
    pub created_at: u64,
    pub access_count: u32,
}

/// Shared memory statistics
#[derive(Debug, Clone, Default)]
pub struct SharedMemoryStatistics {
    pub total_segments: u32,
    pub total_size: u64,
    pub active_mappings: u32,
    pub read_operations: u64,
    pub write_operations: u64,
    pub copy_operations: u64,
    pub errors: u32,
}

/// Shared memory segment
#[derive(Debug)]
pub struct SharedMemory {
    pub id: SharedMemoryId,
    pub size: usize,
    pub data: Vec<u8>,
    pub flags: SharedMemoryFlags,
    pub permissions: MemoryPermissions,
    pub created_by: u32,
    pub created_at: u64,
    pub mappings: RwLock<BTreeMap<u32, MemoryMapping>>, // process_id -> mapping
    pub reference_count: AtomicU32,
    pub access_lock: Mutex<()>, // For atomic operations
    pub statistics: SharedMemoryStatistics,
}

impl SharedMemory {
    pub fn new(id: SharedMemoryId, size: usize) -> IpcResult<Self> {
        if size == 0 || size > MAX_SHM_SIZE {
            return Err(IpcError::ResourceExhausted);
        }

        let mut data = Vec::with_capacity(size);
        data.resize(size, 0);

        Ok(Self {
            id,
            size,
            data,
            flags: SharedMemoryFlags::empty(),
            permissions: MemoryPermissions::READ | MemoryPermissions::WRITE,
            created_by: 0,
            created_at: 0,
            mappings: RwLock::new(BTreeMap::new()),
            reference_count: AtomicU32::new(1), // Created with reference count 1
            access_lock: Mutex::new(()),
            statistics: SharedMemoryStatistics::default(),
        })
    }

    /// Map the shared memory into a process address space
    pub fn map_to_process(&self, process_id: u32, base_address: usize, permissions: MemoryPermissions) -> IpcResult<MemoryMapping> {
        // Check permissions
        if !self.permissions.contains(permissions) {
            return Err(IpcError::PermissionDenied);
        }

        let mapping = MemoryMapping {
            process_id,
            base_address,
            size: self.size,
            permissions,
            created_at: 0, // Will be set by caller
            access_count: 0,
        };

        let mut mappings = self.mappings.write();
        mappings.insert(process_id, mapping.clone());
        
        // Update reference count
        self.reference_count.fetch_add(1, Ordering::SeqCst);
        
        Ok(mapping)
    }

    /// Unmap shared memory from a process
    pub fn unmap_from_process(&self, process_id: u32) -> IpcResult<()> {
        let mut mappings = self.mappings.write();
        if let Some(mapping) = mappings.remove(&process_id) {
            // Update reference count
            self.reference_count.fetch_sub(1, Ordering::SeqCst);
            
            log::debug!("Unmapped shared memory {} from process {}", self.id, process_id);
            Ok(())
        } else {
            Err(IpcError::InvalidHandle)
        }
    }

    /// Read data from shared memory at offset
    pub fn read(&self, offset: usize, buffer: &mut [u8]) -> IpcResult<usize> {
        if offset >= self.size {
            return Err(IpcError::BufferTooSmall);
        }

        let _lock = self.access_lock.lock();
        
        let end_offset = offset + buffer.len();
        let actual_end = end_offset.min(self.size);
        let bytes_to_read = actual_end - offset;

        if bytes_to_read > 0 {
            buffer[..bytes_to_read].copy_from_slice(&self.data[offset..actual_end]);
            self.statistics.read_operations += 1;
            self.statistics.bytes_received += bytes_to_read as u64;
        }

        Ok(bytes_to_read)
    }

    /// Write data to shared memory at offset
    pub fn write(&self, offset: usize, data: &[u8]) -> IpcResult<usize> {
        if offset >= self.size {
            return Err(IpcError::BufferTooSmall);
        }

        // Check write permissions
        if !self.permissions.contains(MemoryPermissions::WRITE) {
            return Err(IpcError::PermissionDenied);
        }

        let _lock = self.access_lock.lock();
        
        let end_offset = offset + data.len();
        let actual_end = end_offset.min(self.size);
        let bytes_to_write = actual_end - offset;

        if bytes_to_write > 0 {
            self.data[offset..actual_end].copy_from_slice(&data[..bytes_to_write]);
            self.statistics.write_operations += 1;
            self.statistics.bytes_sent += bytes_to_write as u64;
        }

        Ok(bytes_to_write)
    }

    /// Copy data from shared memory at offset
    pub fn copy_from(&self, offset: usize, length: usize) -> IpcResult<Vec<u8>> {
        if offset >= self.size {
            return Err(IpcError::BufferTooSmall);
        }

        let _lock = self.access_lock.lock();
        
        let end_offset = offset + length;
        let actual_end = end_offset.min(self.size);
        let bytes_to_copy = actual_end - offset;

        let mut result = Vec::with_capacity(bytes_to_copy);
        result.extend_from_slice(&self.data[offset..actual_end]);
        
        self.statistics.copy_operations += 1;
        self.statistics.bytes_received += bytes_to_copy as u64;

        Ok(result)
    }

    /// Get process mappings
    pub fn get_mappings(&self) -> RwLockReadGuard<BTreeMap<u32, MemoryMapping>> {
        self.mappings.read()
    }

    /// Check if memory is accessible to process
    pub fn is_accessible_by(&self, process_id: u32) -> bool {
        let mappings = self.mappings.read();
        mappings.contains_key(&process_id)
    }

    /// Get shared memory statistics
    pub fn get_statistics(&self) -> SharedMemoryStatistics {
        let mappings = self.mappings.read();
        SharedMemoryStatistics {
            total_segments: 1,
            total_size: self.size as u64,
            active_mappings: mappings.len() as u32,
            read_operations: self.statistics.read_operations,
            write_operations: self.statistics.write_operations,
            copy_operations: self.statistics.copy_operations,
            errors: self.statistics.errors,
        }
    }

    /// Update access count for a process
    pub fn update_access_count(&self, process_id: u32) -> IpcResult<()> {
        let mut mappings = self.mappings.write();
        if let Some(mapping) = mappings.get_mut(&process_id) {
            mapping.access_count += 1;
            Ok(())
        } else {
            Err(IpcError::InvalidHandle)
        }
    }

    /// Clear the shared memory (fill with zeros)
    pub fn clear(&self) -> IpcResult<()> {
        let _lock = self.access_lock.lock();
        self.data.fill(0);
        Ok(())
    }

    /// Resize shared memory (only possible if no processes are mapped)
    pub fn resize(&mut self, new_size: usize) -> IpcResult<()> {
        if new_size == 0 || new_size > MAX_SHM_SIZE {
            return Err(IpcError::ResourceExhausted);
        }

        // Check if any processes are mapped
        let mappings = self.mappings.read();
        if !mappings.is_empty() {
            return Err(IpcError::InvalidHandle); // Cannot resize while mapped
        }

        self.data.resize(new_size, 0);
        self.size = new_size;
        
        Ok(())
    }

    /// Get reference count
    pub fn reference_count(&self) -> u32 {
        self.reference_count.load(Ordering::SeqCst)
    }

    /// Decrement reference count and return true if should be freed
    pub fn decrement_reference(&self) -> bool {
        let count = self.reference_count.fetch_sub(1, Ordering::SeqCst);
        count <= 1 // Should be freed if count becomes 0
    }
}

/// Shared memory manager for handling multiple segments
#[derive(Debug)]
pub struct SharedMemoryManager {
    pub segments: RwLock<BTreeMap<SharedMemoryId, SharedMemory>>,
    pub next_id: AtomicU32,
    pub statistics: SharedMemoryStatistics,
}

impl SharedMemoryManager {
    pub fn new() -> Self {
        Self {
            segments: RwLock::new(BTreeMap::new()),
            next_id: AtomicU32::new(1),
            statistics: SharedMemoryStatistics::default(),
        }
    }

    pub fn create_segment(&self, size: usize, flags: SharedMemoryFlags) -> IpcResult<SharedMemoryId> {
        if size == 0 || size > MAX_SHM_SIZE {
            return Err(IpcError::ResourceExhausted);
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let segment = SharedMemory::new(id, size)?;
        segment.flags = flags;

        let mut segments = self.segments.write();
        segments.insert(id, segment);
        
        self.statistics.total_segments += 1;
        self.statistics.total_size += size as u64;

        Ok(id)
    }

    pub fn get_segment(&self, id: SharedMemoryId) -> Option<SharedMemory> {
        let segments = self.segments.read();
        segments.get(&id).cloned()
    }

    pub fn remove_segment(&self, id: SharedMemoryId) -> IpcResult<()> {
        let mut segments = self.segments.write();
        if let Some(segment) = segments.remove(&id) {
            self.statistics.total_segments -= 1;
            self.statistics.total_size -= segment.size as u64;
            Ok(())
        } else {
            Err(IpcError::InvalidHandle)
        }
    }

    pub fn get_global_statistics(&self) -> SharedMemoryStatistics {
        let segments = self.segments.read();
        let mut total_reads = 0;
        let mut total_writes = 0;
        let mut total_copies = 0;
        let mut total_errors = 0;
        let mut total_mappings = 0;

        for segment in segments.values() {
            total_reads += segment.statistics.read_operations;
            total_writes += segment.statistics.write_operations;
            total_copies += segment.statistics.copy_operations;
            total_errors += segment.statistics.errors;
            total_mappings += segment.mappings.read().len() as u64;
        }

        SharedMemoryStatistics {
            total_segments: self.statistics.total_segments,
            total_size: self.statistics.total_size,
            active_mappings: total_mappings as u32,
            read_operations: total_reads,
            write_operations: total_writes,
            copy_operations: total_copies,
            errors: total_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_memory_read_write() {
        let shm = SharedMemory::new(1, 1024).unwrap();
        
        // Test write
        let data = b"Hello, Shared Memory!";
        assert_eq!(shm.write(0, data).unwrap(), data.len());
        
        // Test read
        let mut buffer = vec![0u8; data.len()];
        assert_eq!(shm.read(0, &mut buffer).unwrap(), data.len());
        assert_eq!(buffer, data);
    }

    #[test]
    fn test_shared_memory_mapping() {
        let shm = SharedMemory::new(1, 1024).unwrap();
        
        // Test mapping
        let mapping = shm.map_to_process(100, 0x1000, MemoryPermissions::READ).unwrap();
        assert_eq!(mapping.process_id, 100);
        assert_eq!(mapping.base_address, 0x1000);
        
        // Test unmapping
        assert!(shm.unmap_from_process(100).is_ok());
    }

    #[test]
    fn test_shared_memory_permissions() {
        let mut shm = SharedMemory::new(1, 1024).unwrap();
        shm.permissions = MemoryPermissions::READ;
        
        // Should fail to write
        let data = b"test";
        assert_eq!(shm.write(0, data), Err(IpcError::PermissionDenied));
    }
}
