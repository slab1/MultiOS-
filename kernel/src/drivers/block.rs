//! MultiOS Block Device Management System
//! 
//! Comprehensive block device management with I/O scheduling, caching,
//! wear leveling, and error recovery.

use crate::log::{info, warn, error};
use crate::drivers::block_io_scheduler::{BlockIoScheduler, SchedulerType};
use crate::drivers::write_cache::{WriteCache, CachePolicy};
use crate::drivers::wear_leveling::{WearLevelingManager, WearLevelingStrategy};
use crate::drivers::sd_card::{SdCardDriver, SdCardType};
use crate::drivers::error_recovery::{ErrorRecoveryManager, RecoveryStrategy};
use crate::drivers::block_device_interface::{BlockDeviceInterface, BlockDeviceManager, BlockDeviceWrapper};

use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap, collections::HashMap};
use alloc::sync::Arc;
use core::time::Duration;

/// Block device identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDeviceId(pub u32);

/// Block device types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockDeviceType {
    Sata = 0,
    Nvme = 1,
    UsbMassStorage = 2,
    SdCard = 3,
    Virtual = 4,
}

/// Block device information
#[derive(Debug, Clone)]
pub struct BlockDeviceInfo {
    pub device_id: BlockDeviceId,
    pub device_type: BlockDeviceType,
    pub name: &'static str,
    pub sector_size: u32,
    pub total_sectors: u64,
    pub max_transfer_size: u32,
    pub queue_depth: u32,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub supports_trim: bool,
    pub supports_write_cache: bool,
    pub physical_sector_size: u32,
    pub max_sectors_per_io: u32,
    pub vendor: &'static str,
    pub model: &'static str,
    pub serial_number: &'static str,
}

/// Block I/O request
#[derive(Debug, Clone)]
pub struct BlockIoRequest {
    pub request_id: u64,
    pub device_id: BlockDeviceId,
    pub operation: BlockOperation,
    pub sector: u64,
    pub sector_count: u32,
    pub buffer: Arc<Vec<u8>>,
    pub callback: Option<fn(BlockIoResult)>,
    pub priority: RequestPriority,
    pub deadline: Option<u64>, // timestamp in nanoseconds
    pub flags: RequestFlags,
}

/// Block device operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlockOperation {
    Read = 0,
    Write = 1,
    Flush = 2,
    Trim = 3,
    Synchronize = 4,
    Verify = 5,
}

/// Request priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum RequestPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Idle = 4,
}

/// Request flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestFlags {
    pub no_retry: bool,
    pub skip_cache: bool,
    pub urgent: bool,
    pub metadata: bool,
}

impl Default for RequestFlags {
    fn default() -> Self {
        Self {
            no_retry: false,
            skip_cache: false,
            urgent: false,
            metadata: false,
        }
    }
}

/// Block I/O result
#[derive(Debug, Clone)]
pub struct BlockIoResult {
    pub request_id: u64,
    pub device_id: BlockDeviceId,
    pub success: bool,
    pub bytes_transferred: usize,
    pub error: Option<BlockDeviceError>,
    pub elapsed_time: Duration,
}

/// Block device errors
#[derive(Debug, Clone)]
pub enum BlockDeviceError {
    DeviceNotFound,
    InvalidSector,
    BufferTooSmall,
    HardwareError,
    Timeout,
    PermissionDenied,
    OutOfSpace,
    UnsupportedOperation,
    MediaError,
    RetryRequired,
    BadBlock,
}

/// Block device statistics
#[derive(Debug, Clone, Default)]
pub struct BlockDeviceStats {
    pub reads: u64,
    pub writes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_errors: u64,
    pub write_errors: u64,
    pub trim_operations: u64,
    pub flush_operations: u64,
    pub avg_read_latency: u64, // microseconds
    pub avg_write_latency: u64, // microseconds
    pub queue_depth: f32,
    pub utilization: f32,
}

/// Main block device manager
pub struct BlockDeviceManager {
    devices: RwLock<BTreeMap<BlockDeviceId, Arc<Mutex<dyn BlockDeviceInterface>>>>,
    scheduler: Arc<Mutex<BlockIoScheduler>>,
    write_cache: Arc<Mutex<WriteCache>>,
    wear_leveling: Arc<Mutex<WearLevelingManager>>,
    error_recovery: Arc<Mutex<ErrorRecoveryManager>>,
    stats: Arc<RwLock<BTreeMap<BlockDeviceId, BlockDeviceStats>>>,
    next_device_id: Arc<Mutex<u32>>,
}

impl BlockDeviceManager {
    /// Create new block device manager
    pub fn new() -> Self {
        info!("Initializing Block Device Manager");
        
        Self {
            devices: RwLock::new(BTreeMap::new()),
            scheduler: Arc::new(Mutex::new(BlockIoScheduler::new(SchedulerType::Elevator))),
            write_cache: Arc::new(Mutex::new(WriteCache::new(CachePolicy::WriteBack))),
            wear_leveling: Arc::new(Mutex::new(WearLevelingManager::new(WearLevelingStrategy::Dynamic))),
            error_recovery: Arc::new(Mutex::new(ErrorRecoveryManager::new())),
            stats: Arc::new(RwLock::new(BTreeMap::new())),
            next_device_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Initialize the block device manager
    pub fn init(&self) -> Result<(), BlockDeviceError> {
        info!("Starting Block Device Manager initialization");
        
        // Initialize scheduler
        self.scheduler.lock().init()?;
        
        // Initialize write cache
        self.write_cache.lock().init()?;
        
        // Initialize wear leveling
        self.wear_leveling.lock().init()?;
        
        // Initialize error recovery
        self.error_recovery.lock().init()?;
        
        info!("Block Device Manager initialized successfully");
        Ok(())
    }

    /// Register a block device
    pub fn register_device(&self, device: Arc<Mutex<dyn BlockDeviceInterface>>) -> Result<BlockDeviceId, BlockDeviceError> {
        let device_info = device.lock().get_device_info()?;
        let device_id = BlockDeviceId(self.next_device_id.lock().0);
        self.next_device_id.lock().0 += 1;
        
        info!("Registering block device: {} (ID: {:?})", device_info.name, device_id);
        
        // Insert device into management map
        self.devices.write().insert(device_id, device);
        
        // Initialize stats
        self.stats.write().insert(device_id, BlockDeviceStats::default());
        
        // Configure scheduler for this device
        self.scheduler.lock().add_device(device_id, device_info.queue_depth);
        
        // Setup wear leveling if SSD
        if device_info.supports_trim {
            self.wear_leveling.lock().add_ssd_device(device_id, device_info.total_sectors);
        }
        
        info!("Block device registered successfully");
        Ok(device_id)
    }

    /// Unregister a block device
    pub fn unregister_device(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        info!("Unregistering block device: {:?}", device_id);
        
        // Remove from scheduler
        self.scheduler.lock().remove_device(device_id);
        
        // Remove from wear leveling
        self.wear_leveling.lock().remove_device(device_id);
        
        // Remove device
        self.devices.write().remove(&device_id);
        
        // Remove stats
        self.stats.write().remove(&device_id);
        
        info!("Block device unregistered successfully");
        Ok(())
    }

    /// Read sectors from a device
    pub fn read_sectors(&self, device_id: BlockDeviceId, sector: u64, count: u32, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
        let start_time = crate::arch::get_time_ns();
        
        // Check if device exists
        let device = match self.devices.read().get(&device_id) {
            Some(d) => d.clone(),
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Validate buffer size
        let device_info = device.lock().get_device_info()?;
        let required_size = (count as usize) * (device_info.sector_size as usize);
        if buffer.len() < required_size {
            return Err(BlockDeviceError::BufferTooSmall);
        }
        
        // Check sector bounds
        if sector + count as u64 > device_info.total_sectors {
            return Err(BlockDeviceError::InvalidSector);
        }
        
        info!("Read {} sectors from device {:?} at sector {}", count, device_id, sector);
        
        // Create I/O request
        let request = BlockIoRequest {
            request_id: start_time, // Use timestamp as ID
            device_id,
            operation: BlockOperation::Read,
            sector,
            sector_count: count,
            buffer: Arc::new(buffer.to_vec()),
            callback: None,
            priority: RequestPriority::Normal,
            deadline: None,
            flags: RequestFlags::default(),
        };
        
        // Submit to scheduler
        let bytes_read = self.submit_io_request(request)?;
        
        // Update statistics
        self.update_read_stats(device_id, bytes_read, start_time);
        
        Ok(bytes_read)
    }

    /// Write sectors to a device
    pub fn write_sectors(&self, device_id: BlockDeviceId, sector: u64, count: u32, buffer: &[u8]) -> Result<usize, BlockDeviceError> {
        let start_time = crate::arch::get_time_ns();
        
        // Check if device exists
        let device = match self.devices.read().get(&device_id) {
            Some(d) => d.clone(),
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Validate buffer size
        let device_info = device.lock().get_device_info()?;
        let required_size = (count as usize) * (device_info.sector_size as usize);
        if buffer.len() < required_size {
            return Err(BlockDeviceError::BufferTooSmall);
        }
        
        // Check if device is read-only
        if device_info.is_read_only {
            return Err(BlockDeviceError::PermissionDenied);
        }
        
        // Check sector bounds
        if sector + count as u64 > device_info.total_sectors {
            return Err(BlockDeviceError::InvalidSector);
        }
        
        info!("Write {} sectors to device {:?} at sector {}", count, device_id, sector);
        
        // Check write cache first
        if device_info.supports_write_cache {
            if let Some(cached_data) = self.write_cache.lock().check_cache_write(device_id, sector, count) {
                if cached_data == buffer {
                    info!("Write operation satisfied by cache");
                    return Ok(required_size);
                }
            }
        }
        
        // Create I/O request
        let request = BlockIoRequest {
            request_id: start_time,
            device_id,
            operation: BlockOperation::Write,
            sector,
            sector_count: count,
            buffer: Arc::new(buffer.to_vec()),
            callback: None,
            priority: RequestPriority::Normal,
            deadline: None,
            flags: RequestFlags::default(),
        };
        
        // Submit to scheduler
        let bytes_written = self.submit_io_request(request)?;
        
        // Update write cache
        if device_info.supports_write_cache {
            self.write_cache.lock().update_cache(device_id, sector, count, &buffer[..bytes_written]);
        }
        
        // Apply wear leveling for SSDs
        if device_info.supports_trim {
            self.wear_leveling.lock().record_write(device_id, sector, count);
        }
        
        // Update statistics
        self.update_write_stats(device_id, bytes_written, start_time);
        
        Ok(bytes_written)
    }

    /// Flush write cache for a device
    pub fn flush_device(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        info!("Flushing device {:?}", device_id);
        
        // Check if device exists
        if !self.devices.read().contains_key(&device_id) {
            return Err(BlockDeviceError::DeviceNotFound);
        }
        
        // Flush write cache
        self.write_cache.lock().flush_device(device_id)?;
        
        // Submit flush request
        let request = BlockIoRequest {
            request_id: crate::arch::get_time_ns(),
            device_id,
            operation: BlockOperation::Flush,
            sector: 0,
            sector_count: 0,
            buffer: Arc::new(Vec::new()),
            callback: None,
            priority: RequestPriority::High,
            deadline: None,
            flags: RequestFlags::default(),
        };
        
        self.submit_io_request(request)?;
        
        // Update statistics
        self.update_flush_stats(device_id);
        
        Ok(())
    }

    /// Trim (deallocate) sectors for SSD wear leveling
    pub fn trim_sectors(&self, device_id: BlockDeviceId, sector: u64, count: u32) -> Result<(), BlockDeviceError> {
        info!("TRIM {} sectors on device {:?} starting at sector {}", count, device_id, sector);
        
        // Check if device exists
        let device = match self.devices.read().get(&device_id) {
            Some(d) => d.clone(),
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Check if device supports TRIM
        let device_info = device.lock().get_device_info()?;
        if !device_info.supports_trim {
            return Err(BlockDeviceError::UnsupportedOperation);
        }
        
        // Create TRIM request
        let request = BlockIoRequest {
            request_id: crate::arch::get_time_ns(),
            device_id,
            operation: BlockOperation::Trim,
            sector,
            sector_count: count,
            buffer: Arc::new(Vec::new()),
            callback: None,
            priority: RequestPriority::Low,
            deadline: None,
            flags: RequestFlags::default(),
        };
        
        // Submit to scheduler
        self.submit_io_request(request)?;
        
        // Update wear leveling
        self.wear_leveling.lock().trim_sectors(device_id, sector, count);
        
        // Update statistics
        self.update_trim_stats(device_id);
        
        Ok(())
    }

    /// Submit I/O request to scheduler
    fn submit_io_request(&self, request: BlockIoRequest) -> Result<usize, BlockDeviceError> {
        // Submit to scheduler
        let result = self.scheduler.lock().submit_request(request);
        
        match result {
            Ok(bytes) => Ok(bytes),
            Err(e) => {
                error!("I/O request failed: {:?}", e);
                
                // Attempt error recovery
                if !request.flags.no_retry {
                    self.error_recovery.lock().handle_error(request.device_id, e.clone())?;
                }
                
                Err(e)
            }
        }
    }

    /// Get device information
    pub fn get_device_info(&self, device_id: BlockDeviceId) -> Result<BlockDeviceInfo, BlockDeviceError> {
        let device = match self.devices.read().get(&device_id) {
            Some(d) => d.clone(),
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        device.lock().get_device_info()
    }

    /// Get all registered device IDs
    pub fn get_device_ids(&self) -> Vec<BlockDeviceId> {
        self.devices.read().keys().cloned().collect()
    }

    /// Get device statistics
    pub fn get_device_stats(&self, device_id: BlockDeviceId) -> Result<BlockDeviceStats, BlockDeviceError> {
        match self.stats.read().get(&device_id) {
            Some(stats) => Ok(stats.clone()),
            None => Err(BlockDeviceError::DeviceNotFound),
        }
    }

    /// Get overall statistics
    pub fn get_overall_stats(&self) -> BlockDeviceStats {
        let mut total_stats = BlockDeviceStats::default();
        
        for stats in self.stats.read().values() {
            total_stats.reads += stats.reads;
            total_stats.writes += stats.writes;
            total_stats.bytes_read += stats.bytes_read;
            total_stats.bytes_written += stats.bytes_written;
            total_stats.read_errors += stats.read_errors;
            total_stats.write_errors += stats.write_errors;
            total_stats.trim_operations += stats.trim_operations;
            total_stats.flush_operations += stats.flush_operations;
        }
        
        total_stats
    }

    /// Update read statistics
    fn update_read_stats(&self, device_id: BlockDeviceId, bytes: usize, start_time: u64) {
        if let Some(stats) = self.stats.write().get_mut(&device_id) {
            let elapsed_time = crate::arch::get_time_ns() - start_time;
            stats.reads += 1;
            stats.bytes_read += bytes as u64;
            stats.avg_read_latency = (stats.avg_read_latency + elapsed_time / 1000) / 2; // Convert to microseconds
        }
    }

    /// Update write statistics
    fn update_write_stats(&self, device_id: BlockDeviceId, bytes: usize, start_time: u64) {
        if let Some(stats) = self.stats.write().get_mut(&device_id) {
            let elapsed_time = crate::arch::get_time_ns() - start_time;
            stats.writes += 1;
            stats.bytes_written += bytes as u64;
            stats.avg_write_latency = (stats.avg_write_latency + elapsed_time / 1000) / 2; // Convert to microseconds
        }
    }

    /// Update flush statistics
    fn update_flush_stats(&self, device_id: BlockDeviceId) {
        if let Some(stats) = self.stats.write().get_mut(&device_id) {
            stats.flush_operations += 1;
        }
    }

    /// Update trim statistics
    fn update_trim_stats(&self, device_id: BlockDeviceId) {
        if let Some(stats) = self.stats.write().get_mut(&device_id) {
            stats.trim_operations += 1;
        }
    }
}

/// Global block device manager instance
static BLOCK_DEVICE_MANAGER: once_cell::sync::OnceCell<Arc<BlockDeviceManager>> = once_cell::sync::OnceCell::new();

/// Initialize the global block device manager
pub fn init_block_device_manager() -> Result<(), BlockDeviceError> {
    let manager = Arc::new(BlockDeviceManager::new());
    manager.init()?;
    
    BLOCK_DEVICE_MANAGER.set(manager).map_err(|_| BlockDeviceError::DeviceNotFound)?;
    info!("Global Block Device Manager initialized");
    Ok(())
}

/// Get the global block device manager
pub fn get_block_device_manager() -> Option<Arc<BlockDeviceManager>> {
    BLOCK_DEVICE_MANAGER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_device_manager_creation() {
        let manager = BlockDeviceManager::new();
        assert!(manager.get_device_ids().is_empty());
    }

    #[test]
    fn test_block_device_error() {
        let error = BlockDeviceError::DeviceNotFound;
        assert!(matches!(error, BlockDeviceError::DeviceNotFound));
    }

    #[test]
    fn test_request_priority_ordering() {
        assert!(RequestPriority::Critical < RequestPriority::High);
        assert!(RequestPriority::High < RequestPriority::Normal);
        assert!(RequestPriority::Normal < RequestPriority::Low);
        assert!(RequestPriority::Low < RequestPriority::Idle);
    }

    #[test]
    fn test_request_flags_default() {
        let flags = RequestFlags::default();
        assert!(!flags.no_retry);
        assert!(!flags.skip_cache);
        assert!(!flags.urgent);
        assert!(!flags.metadata);
    }
}