//! Block Device Interface
//! 
//! Common interface trait for all block devices in MultiOS, providing
//! unified access to storage devices regardless of underlying hardware.

use crate::log::{info, warn, error};
use super::block::{BlockDeviceId, BlockIoResult, BlockOperation, BlockDeviceError, BlockDeviceInfo};

use alloc::sync::Arc;
use alloc::vec::Vec;

/// Common block device interface that all storage devices must implement
pub trait BlockDeviceInterface: Send + Sync {
    /// Read sectors from the device
    /// 
    /// # Arguments
    /// * `sector` - Starting sector number
    /// * `count` - Number of sectors to read
    /// * `buffer` - Buffer to store read data
    /// 
    /// # Returns
    /// Number of bytes read, or error
    fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> Result<usize, BlockDeviceError>;
    
    /// Write sectors to the device
    /// 
    /// # Arguments
    /// * `sector` - Starting sector number
    /// * `count` - Number of sectors to write
    /// * `buffer` - Data to write
    /// 
    /// # Returns
    /// Number of bytes written, or error
    fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> Result<usize, BlockDeviceError>;
    
    /// Flush write cache to the device
    fn flush(&self) -> Result<(), BlockDeviceError>;
    
    /// Trim (deallocate) sectors for SSD optimization
    fn trim_sectors(&self, sector: u64, count: u32) -> Result<(), BlockDeviceError>;
    
    /// Get device information
    fn get_device_info(&self) -> Result<BlockDeviceInfo, BlockDeviceError>;
    
    /// Check if device is ready for I/O operations
    fn is_ready(&self) -> bool;
    
    /// Get maximum transfer size for this device
    fn get_max_transfer_size(&self) -> usize {
        match self.get_device_info() {
            Ok(info) => info.max_transfer_size as usize,
            Err(_) => 512 * 1024, // Default 512KB
        }
    }
    
    /// Get optimal block size for I/O operations
    fn get_optimal_block_size(&self) -> u32 {
        match self.get_device_info() {
            Ok(info) => info.sector_size,
            Err(_) => 512,
        }
    }
    
    /// Synchronize device (flush and ensure consistency)
    fn synchronize(&self) -> Result<(), BlockDeviceError> {
        self.flush()
    }
    
    /// Get device-specific capabilities
    fn get_capabilities(&self) -> DeviceCapabilities {
        match self.get_device_info() {
            Ok(info) => DeviceCapabilities {
                supports_read: true,
                supports_write: !info.is_read_only,
                supports_trim: info.supports_trim,
                supports_flush: info.supports_write_cache,
                supports_multiple_io: info.queue_depth > 1,
                supports_removable_media: info.is_removable,
                supports_hot_plug: info.is_removable,
                max_concurrent_io: info.queue_depth,
            },
            Err(_) => DeviceCapabilities::default(),
        }
    }
    
    /// Perform device-specific initialization
    fn initialize(&self) -> Result<(), BlockDeviceError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Shutdown device gracefully
    fn shutdown(&self) -> Result<(), BlockDeviceError> {
        // Default implementation flushes device
        self.flush()
    }
    
    /// Get device statistics
    fn get_statistics(&self) -> Result<DeviceStatistics, BlockDeviceError> {
        // Default implementation returns empty statistics
        Ok(DeviceStatistics::default())
    }
    
    /// Reset device to initial state
    fn reset(&self) -> Result<(), BlockDeviceError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Check device health
    fn check_health(&self) -> Result<DeviceHealth, BlockDeviceError> {
        let info = self.get_device_info()?;
        
        Ok(DeviceHealth {
            is_healthy: self.is_ready(),
            total_sectors: info.total_sectors,
            bad_sectors: 0,
            temperature: None,
            power_on_hours: None,
            error_count: 0,
            last_error: None,
        })
    }
    
    /// Execute device-specific command
    fn execute_command(&self, command: DeviceCommand, data: &[u8]) -> Result<Vec<u8>, BlockDeviceError> {
        // Default implementation returns unsupported operation error
        Err(BlockDeviceError::UnsupportedOperation)
    }
    
    /// Enable or disable write cache
    fn set_write_cache_enabled(&self, enabled: bool) -> Result<(), BlockDeviceError> {
        // Default implementation does nothing
        warn!("Write cache control not supported by this device");
        Ok(())
    }
    
    /// Get current write cache status
    fn is_write_cache_enabled(&self) -> bool {
        // Default implementation assumes write cache is enabled if supported
        match self.get_device_info() {
            Ok(info) => info.supports_write_cache,
            Err(_) => false,
        }
    }
}

/// Device capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub supports_read: bool,
    pub supports_write: bool,
    pub supports_trim: bool,
    pub supports_flush: bool,
    pub supports_multiple_io: bool,
    pub supports_removable_media: bool,
    pub supports_hot_plug: bool,
    pub max_concurrent_io: u32,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            supports_read: true,
            supports_write: true,
            supports_trim: false,
            supports_flush: true,
            supports_multiple_io: true,
            supports_removable_media: false,
            supports_hot_plug: false,
            max_concurrent_io: 1,
        }
    }
}

/// Device statistics
#[derive(Debug, Clone, Default)]
pub struct DeviceStatistics {
    pub total_reads: u64,
    pub total_writes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_errors: u64,
    pub write_errors: u64,
    pub trim_operations: u64,
    pub flush_operations: u64,
    pub avg_read_latency_us: u64,
    pub avg_write_latency_us: u64,
    pub queue_depth: f32,
    pub utilization: f32,
    pub uptime: core::time::Duration,
}

/// Device health information
#[derive(Debug, Clone)]
pub struct DeviceHealth {
    pub is_healthy: bool,
    pub total_sectors: u64,
    pub bad_sectors: u64,
    pub temperature: Option<u32>,     // Celsius
    pub power_on_hours: Option<u64>,
    pub error_count: u32,
    pub last_error: Option<String>,
}

/// Device commands
#[derive(Debug, Clone)]
pub enum DeviceCommand {
    Identify,
    GetSmartData,
    SetPowerMode,
    EnableWriteCache,
    DisableWriteCache,
    GetFirmwareVersion,
    SecurityErase,
    Format,
    Custom(u32), // Custom command with ID
}

/// Block device wrapper that provides enhanced functionality
pub struct BlockDeviceWrapper {
    device_id: BlockDeviceId,
    device: Arc<dyn BlockDeviceInterface>,
    statistics: Arc<spin::Mutex<DeviceStatistics>>,
    health_check_interval: core::time::Duration,
    last_health_check: core::time::Instant,
}

impl BlockDeviceWrapper {
    /// Create new block device wrapper
    pub fn new(device_id: BlockDeviceId, device: Arc<dyn BlockDeviceInterface>) -> Self {
        Self {
            device_id,
            device,
            statistics: Arc::new(spin::Mutex::new(DeviceStatistics::default())),
            health_check_interval: core::time::Duration::from_secs(300), // 5 minutes
            last_health_check: core::time::Instant::now(),
        }
    }

    /// Get device ID
    pub fn get_device_id(&self) -> BlockDeviceId {
        self.device_id
    }

    /// Get inner device reference
    pub fn get_device(&self) -> &dyn BlockDeviceInterface {
        self.device.as_ref()
    }

    /// Read sectors with statistics tracking
    pub fn read_sectors(&self, sector: u64, count: u32, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
        let start_time = core::time::Instant::now();
        
        match self.device.read_sectors(sector, count, buffer) {
            Ok(bytes_read) => {
                let mut stats = self.statistics.lock();
                stats.total_reads += 1;
                stats.bytes_read += bytes_read as u64;
                
                let elapsed = start_time.elapsed();
                stats.avg_read_latency_us = (stats.avg_read_latency_us + elapsed.as_micros() as u64) / 2;
                
                Ok(bytes_read)
            }
            Err(e) => {
                let mut stats = self.statistics.lock();
                stats.read_errors += 1;
                Err(e)
            }
        }
    }

    /// Write sectors with statistics tracking
    pub fn write_sectors(&self, sector: u64, count: u32, buffer: &[u8]) -> Result<usize, BlockDeviceError> {
        let start_time = core::time::Instant::now();
        
        match self.device.write_sectors(sector, count, buffer) {
            Ok(bytes_written) => {
                let mut stats = self.statistics.lock();
                stats.total_writes += 1;
                stats.bytes_written += bytes_written as u64;
                
                let elapsed = start_time.elapsed();
                stats.avg_write_latency_us = (stats.avg_write_latency_us + elapsed.as_micros() as u64) / 2;
                
                Ok(bytes_written)
            }
            Err(e) => {
                let mut stats = self.statistics.lock();
                stats.write_errors += 1;
                Err(e)
            }
        }
    }

    /// Flush device with statistics tracking
    pub fn flush(&self) -> Result<(), BlockDeviceError> {
        match self.device.flush() {
            Ok(_) => {
                let mut stats = self.statistics.lock();
                stats.flush_operations += 1;
                Ok(())
            }
            Err(e) => {
                let mut stats = self.statistics.lock();
                stats.write_errors += 1;
                Err(e)
            }
        }
    }

    /// Trim sectors with statistics tracking
    pub fn trim_sectors(&self, sector: u64, count: u32) -> Result<(), BlockDeviceError> {
        match self.device.trim_sectors(sector, count) {
            Ok(_) => {
                let mut stats = self.statistics.lock();
                stats.trim_operations += 1;
                Ok(())
            }
            Err(e) => {
                let mut stats = self.statistics.lock();
                stats.write_errors += 1;
                Err(e)
            }
        }
    }

    /// Get enhanced device information
    pub fn get_device_info(&self) -> Result<BlockDeviceInfo, BlockDeviceError> {
        self.device.get_device_info()
    }

    /// Check device readiness
    pub fn is_ready(&self) -> bool {
        self.device.is_ready()
    }

    /// Get device capabilities
    pub fn get_capabilities(&self) -> DeviceCapabilities {
        self.device.get_capabilities()
    }

    /// Get device statistics
    pub fn get_statistics(&self) -> DeviceStatistics {
        let mut stats = self.statistics.lock();
        stats.uptime = core::time::Instant::now().elapsed();
        stats.clone()
    }

    /// Perform health check if due
    pub fn perform_periodic_maintenance(&mut self) {
        let now = core::time::Instant::now();
        
        if now.duration_since(self.last_health_check) >= self.health_check_interval {
            match self.device.check_health() {
                Ok(health) => {
                    if !health.is_healthy {
                        warn!("Device {:?} health check failed: bad sectors: {}", self.device_id, health.bad_sectors);
                    }
                }
                Err(e) => {
                    warn!("Health check failed for device {:?}: {:?}", self.device_id, e);
                }
            }
            
            self.last_health_check = now;
        }
    }

    /// Read multiple sectors efficiently
    pub fn read_multiple_sectors(&self, sectors: &[(u64, u32)]) -> Result<Vec<Vec<u8>>, BlockDeviceError> {
        let mut results = Vec::new();
        let max_transfer_size = self.device.get_max_transfer_size();
        
        for &(sector, count) in sectors {
            // Check if transfer size is within limits
            let required_bytes = (count as usize) * self.device.get_optimal_block_size() as usize;
            if required_bytes > max_transfer_size {
                return Err(BlockDeviceError::BufferTooSmall);
            }
            
            let mut buffer = vec![0u8; required_bytes];
            let bytes_read = self.read_sectors(sector, count, &mut buffer)?;
            
            if bytes_read != required_bytes {
                return Err(BlockDeviceError::HardwareError);
            }
            
            results.push(buffer);
        }
        
        Ok(results)
    }

    /// Write multiple sectors efficiently
    pub fn write_multiple_sectors(&self, sectors: &[(u64, &[u8])]) -> Result<(), BlockDeviceError> {
        for &(sector, data) in sectors {
            let sector_count = (data.len() / self.device.get_optimal_block_size() as usize) as u32;
            
            // Check if transfer size is within limits
            let required_bytes = (sector_count as usize) * self.device.get_optimal_block_size() as usize;
            if required_bytes != data.len() {
                return Err(BlockDeviceError::BufferTooSmall);
            }
            
            self.write_sectors(sector, sector_count, data)?;
        }
        
        Ok(())
    }

    /// Perform zero-copy read using provided buffer
    pub fn read_sectors_into(&self, sector: u64, count: u32, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
        self.read_sectors(sector, count, buffer)
    }

    /// Perform zero-copy write using provided buffer
    pub fn write_sectors_from(&self, sector: u64, count: u32, buffer: &[u8]) -> Result<usize, BlockDeviceError> {
        self.write_sectors(sector, count, buffer)
    }

    /// Get device performance metrics
    pub fn get_performance_metrics(&self) -> DevicePerformanceMetrics {
        let stats = self.get_statistics();
        let info = match self.get_device_info() {
            Ok(info) => info,
            Err(_) => return DevicePerformanceMetrics::default(),
        };
        
        DevicePerformanceMetrics {
            throughput_read_mbps: if stats.avg_read_latency_us > 0 {
                (stats.bytes_read / 1024 / 1024) as f32 / (stats.avg_read_latency_us as f32 / 1_000_000.0)
            } else { 0.0 },
            throughput_write_mbps: if stats.avg_write_latency_us > 0 {
                (stats.bytes_written / 1024 / 1024) as f32 / (stats.avg_write_latency_us as f32 / 1_000_000.0)
            } else { 0.0 },
            avg_latency_read_us: stats.avg_read_latency_us,
            avg_latency_write_us: stats.avg_write_latency_us,
            io_operations_per_second: (stats.total_reads + stats.total_writes) as f32 / stats.uptime.as_secs_f32(),
            error_rate: if (stats.total_reads + stats.total_writes) > 0 {
                (stats.read_errors + stats.write_errors) as f32 / (stats.total_reads + stats.total_writes) as f32
            } else { 0.0 },
            queue_depth: stats.queue_depth,
            utilization: stats.utilization,
        }
    }
}

/// Device performance metrics
#[derive(Debug, Clone, Default)]
pub struct DevicePerformanceMetrics {
    pub throughput_read_mbps: f32,
    pub throughput_write_mbps: f32,
    pub avg_latency_read_us: u64,
    pub avg_latency_write_us: u64,
    pub io_operations_per_second: f32,
    pub error_rate: f32,
    pub queue_depth: f32,
    pub utilization: f32,
}

/// Block device manager that manages multiple block device wrappers
pub struct BlockDeviceManager {
    devices: spin::Mutex<BTreeMap<BlockDeviceId, BlockDeviceWrapper>>,
    next_device_id: spin::Mutex<u32>,
}

impl BlockDeviceManager {
    /// Create new block device manager
    pub fn new() -> Self {
        info!("Initializing Block Device Manager");
        
        Self {
            devices: spin::Mutex::new(BTreeMap::new()),
            next_device_id: spin::Mutex::new(1),
        }
    }

    /// Register a new block device
    pub fn register_device(&self, device: Arc<dyn BlockDeviceInterface>) -> Result<BlockDeviceId, BlockDeviceError> {
        info!("Registering new block device");
        
        // Initialize device
        device.initialize()?;
        
        // Generate unique device ID
        let device_id = BlockDeviceId(self.next_device_id.lock().0);
        *self.next_device_id.lock().0 += 1;
        
        // Create wrapper
        let wrapper = BlockDeviceWrapper::new(device_id, device);
        
        // Register wrapper
        self.devices.lock().insert(device_id, wrapper);
        
        info!("Device {:?} registered successfully", device_id);
        Ok(device_id)
    }

    /// Unregister a block device
    pub fn unregister_device(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        info!("Unregistering device {:?}", device_id);
        
        let mut devices = self.devices.lock();
        
        // Get device for graceful shutdown
        if let Some(wrapper) = devices.get(&device_id) {
            wrapper.get_device().shutdown()?;
        }
        
        devices.remove(&device_id);
        
        info!("Device {:?} unregistered successfully", device_id);
        Ok(())
    }

    /// Get device wrapper
    pub fn get_device(&self, device_id: BlockDeviceId) -> Option<BlockDeviceWrapper> {
        self.devices.lock().get(&device_id).cloned()
    }

    /// Get all device IDs
    pub fn get_device_ids(&self) -> Vec<BlockDeviceId> {
        self.devices.lock().keys().cloned().collect()
    }

    /// Get primary device (first available)
    pub fn get_primary_device(&self) -> Option<BlockDeviceWrapper> {
        let devices = self.devices.lock();
        devices.values().find(|w| w.is_ready()).cloned()
    }

    /// Perform maintenance on all devices
    pub fn perform_maintenance(&self) {
        let mut devices = self.devices.lock();
        
        for (_, wrapper) in devices.iter_mut() {
            wrapper.perform_periodic_maintenance();
        }
    }

    /// Get statistics for all devices
    pub fn get_all_statistics(&self) -> BTreeMap<BlockDeviceId, DeviceStatistics> {
        let devices = self.devices.lock();
        let mut stats = BTreeMap::new();
        
        for (device_id, wrapper) in devices.iter() {
            stats.insert(*device_id, wrapper.get_statistics());
        }
        
        stats
    }

    /// Check health of all devices
    pub fn check_all_health(&self) -> BTreeMap<BlockDeviceId, Result<DeviceHealth, BlockDeviceError>> {
        let devices = self.devices.lock();
        let mut health_status = BTreeMap::new();
        
        for (device_id, wrapper) in devices.iter() {
            health_status.insert(*device_id, wrapper.get_device().check_health());
        }
        
        health_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::sd_card::SdCardDriver;
    use crate::drivers::block::BlockDeviceType;

    struct MockBlockDevice {
        device_info: BlockDeviceInfo,
    }

    impl MockBlockDevice {
        fn new() -> Self {
            Self {
                device_info: BlockDeviceInfo {
                    device_id: BlockDeviceId(1),
                    device_type: BlockDeviceType::Virtual,
                    name: "Mock Device",
                    sector_size: 512,
                    total_sectors: 1_000_000,
                    max_transfer_size: 1024 * 1024,
                    queue_depth: 32,
                    is_removable: false,
                    is_read_only: false,
                    supports_trim: true,
                    supports_write_cache: true,
                    physical_sector_size: 512,
                    max_sectors_per_io: 2048,
                    vendor: "MultiOS",
                    model: "Mock Device",
                    serial_number: "MOCK123",
                },
            }
        }
    }

    impl BlockDeviceInterface for MockBlockDevice {
        fn read_sectors(&self, _sector: u64, _count: u32, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
            // Simulate read operation
            for byte in buffer.iter_mut() {
                *byte = 0x42;
            }
            Ok(buffer.len())
        }

        fn write_sectors(&self, _sector: u64, _count: u32, _buffer: &[u8]) -> Result<usize, BlockDeviceError> {
            Ok(_buffer.len())
        }

        fn flush(&self) -> Result<(), BlockDeviceError> {
            Ok(())
        }

        fn trim_sectors(&self, _sector: u64, _count: u32) -> Result<(), BlockDeviceError> {
            Ok(())
        }

        fn get_device_info(&self) -> Result<BlockDeviceInfo, BlockDeviceError> {
            Ok(self.device_info.clone())
        }

        fn is_ready(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_device_capabilities_default() {
        let capabilities = DeviceCapabilities::default();
        assert!(capabilities.supports_read);
        assert!(capabilities.supports_write);
        assert_eq!(capabilities.max_concurrent_io, 1);
    }

    #[test]
    fn test_device_statistics_default() {
        let stats = DeviceStatistics::default();
        assert_eq!(stats.total_reads, 0);
        assert_eq!(stats.total_writes, 0);
    }

    #[test]
    fn test_device_performance_metrics_default() {
        let metrics = DevicePerformanceMetrics::default();
        assert_eq!(metrics.throughput_read_mbps, 0.0);
        assert_eq!(metrics.avg_latency_read_us, 0);
    }

    #[test]
    fn test_block_device_wrapper_creation() {
        let mock_device = Arc::new(MockBlockDevice::new());
        let wrapper = BlockDeviceWrapper::new(BlockDeviceId(1), mock_device);
        
        assert_eq!(wrapper.get_device_id(), BlockDeviceId(1));
        assert!(wrapper.is_ready());
    }

    #[test]
    fn test_block_device_manager() {
        let manager = BlockDeviceManager::new();
        assert_eq(manager.get_device_ids().len(), 0);
        
        // Register a mock device
        let mock_device = Arc::new(MockBlockDevice::new());
        let device_id = manager.register_device(mock_device).unwrap();
        
        assert_eq!(manager.get_device_ids().len(), 1);
        assert_eq!(device_id, BlockDeviceId(1));
    }

    #[test]
    fn test_device_health_info() {
        let health = DeviceHealth {
            is_healthy: true,
            total_sectors: 1_000_000,
            bad_sectors: 0,
            temperature: Some(35),
            power_on_hours: Some(8760),
            error_count: 0,
            last_error: None,
        };
        
        assert!(health.is_healthy);
        assert_eq!(health.temperature, Some(35));
    }

    #[test]
    fn test_device_command_variants() {
        let commands = vec![
            DeviceCommand::Identify,
            DeviceCommand::GetSmartData,
            DeviceCommand::SecurityErase,
            DeviceCommand::Custom(123),
        ];
        
        assert_eq!(commands.len(), 4);
    }
}