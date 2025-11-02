//! Storage Device Drivers and Block Management System Example
//! 
//! This example demonstrates the complete implementation of storage device
//! drivers and block device management for MultiOS including I/O scheduling,
//! caching, wear leveling, and error recovery.

use crate::drivers::block::{
    BlockDeviceManager, BlockDeviceId, BlockDeviceType, BlockOperation,
    BlockIoRequest, RequestPriority, RequestFlags
};
use crate::drivers::block_io_scheduler::{BlockIoScheduler, SchedulerType};
use crate::drivers::write_cache::{WriteCache, CachePolicy};
use crate::drivers::wear_leveling::{WearLevelingManager, WearLevelingStrategy};
use crate::drivers::sd_card::{SdCardDriver, SdCardType, SdInterfaceMode};
use crate::drivers::error_recovery::{ErrorRecoveryManager, RecoveryStrategy};
use crate::drivers::block_device_interface::{BlockDeviceInterface, BlockDeviceManager, BlockDeviceWrapper};

use log::{info, warn, error};
use alloc::{sync::Arc, vec::Vec};
use core::time::Duration;

/// Example demonstrating the complete storage device drivers system
pub struct StorageSystemExample {
    block_manager: Arc<BlockDeviceManager>,
    sd_card_driver: Option<SdCardDriver>,
    error_recovery: Arc<ErrorRecoveryManager>,
    wear_leveling: Arc<WearLevelingManager>,
}

impl StorageSystemExample {
    /// Create new storage system example
    pub fn new() -> Self {
        info!("Creating Storage System Example");
        
        let block_manager = Arc::new(BlockDeviceManager::new());
        let error_recovery = Arc::new(ErrorRecoveryManager::new());
        let wear_leveling = Arc::new(WearLevelingManager::new(WearLevelingStrategy::Dynamic));
        
        Self {
            block_manager,
            sd_card_driver: None,
            error_recovery,
            wear_leveling,
        }
    }

    /// Initialize the complete storage system
    pub fn initialize(&mut self) -> Result<(), crate::drivers::block::BlockDeviceError> {
        info!("Initializing complete storage system");
        
        // Initialize block device manager
        // (In real implementation, this would be called during kernel initialization)
        
        // Initialize SD card driver
        self.initialize_sd_card()?;
        
        // Initialize other storage devices (SATA, NVMe, USB would be similar)
        
        info!("Storage system initialized successfully");
        Ok(())
    }

    /// Initialize SD card driver
    fn initialize_sd_card(&mut self) -> Result<(), crate::drivers::block::BlockDeviceError> {
        info!("Initializing SD card driver");
        
        // Create SD card driver with SPI pins
        let sd_card = SdCardDriver::new(
            BlockDeviceId(1), // device ID
            10, // CS pin
            11, // MOSI pin
            12, // MISO pin
            13, // CLK pin
        );
        
        // In real implementation, would initialize the SD card
        // For this example, we'll simulate it
        info!("SD card driver created: {:?}", sd_card.get_state());
        
        self.sd_card_driver = Some(sd_card);
        
        Ok(())
    }

    /// Demonstrate I/O scheduling
    pub fn demonstrate_io_scheduling(&self) {
        info!("=== I/O Scheduling Demonstration ===");
        
        // Create scheduler
        let mut scheduler = BlockIoScheduler::new(SchedulerType::Elevator);
        
        // Add a device to scheduler
        scheduler.add_device(BlockDeviceId(1), 32); // 32 queue depth
        
        // Create sample I/O requests
        let read_request = BlockIoRequest {
            request_id: 1,
            device_id: BlockDeviceId(1),
            operation: BlockOperation::Read,
            sector: 1000,
            sector_count: 8,
            buffer: Arc::new(vec![0u8; 4096]),
            callback: None,
            priority: RequestPriority::Normal,
            deadline: None,
            flags: RequestFlags::default(),
        };
        
        let write_request = BlockIoRequest {
            request_id: 2,
            device_id: BlockDeviceId(1),
            operation: BlockOperation::Write,
            sector: 2000,
            sector_count: 16,
            buffer: Arc::new(vec![1u8; 8192]),
            callback: None,
            priority: RequestPriority::High,
            deadline: None,
            flags: RequestFlags::default(),
        };
        
        // Submit requests
        let _ = scheduler.submit_request(read_request);
        let _ = scheduler.submit_request(write_request);
        
        // Get scheduler statistics
        let stats = scheduler.get_statistics();
        info!("Scheduler stats: pending reads: {}, pending writes: {}", 
              stats.pending_reads, stats.pending_writes);
        
        // Demonstrate different scheduler types
        for scheduler_type in [
            SchedulerType::Elevator,
            SchedulerType::Cfq,
            SchedulerType::Deadline,
            SchedulerType::NoneOps,
        ] {
            let mut sched = BlockIoScheduler::new(scheduler_type);
            sched.add_device(BlockDeviceId(1), 32);
            info!("Scheduler type {:?}: initialized successfully", scheduler_type);
        }
    }

    /// Demonstrate write caching
    pub fn demonstrate_write_caching(&self) {
        info!("=== Write Caching Demonstration ===");
        
        let mut cache = WriteCache::new(CachePolicy::WriteBack);
        
        // Add device to cache
        cache.add_device(BlockDeviceId(1), 1024 * 1024, CachePolicy::WriteBack); // 1MB cache
        
        // Demonstrate cache write
        let test_data = vec![0x42u8; 512];
        let _ = cache.update_cache(BlockDeviceId(1), 100, 1, &test_data);
        
        // Check if data is in cache
        if let Some(cached_data) = cache.check_cache_write(BlockDeviceId(1), 100, 1) {
            info!("Cache hit: data found in cache");
        }
        
        // Read from cache
        let mut read_buffer = vec![0u8; 512];
        let cache_hit = cache.read_cache(BlockDeviceId(1), 100, 1, &mut read_buffer).unwrap_or(false);
        
        if cache_hit {
            info!("Cache read successful");
        }
        
        // Get cache statistics
        let stats = cache.get_cache_stats(BlockDeviceId(1)).unwrap_or_default();
        info!("Cache stats: hits: {}, misses: {}, hit rate: {:.2}%", 
              stats.hits, stats.misses, stats.hit_rate * 100.0);
        
        // Demonstrate different cache policies
        for policy in [CachePolicy::WriteThrough, CachePolicy::WriteBack, CachePolicy::WriteAround] {
            let mut policy_cache = WriteCache::new(policy);
            policy_cache.add_device(BlockDeviceId(2), 1024 * 1024, policy);
            info!("Cache policy {:?}: configured successfully", policy);
        }
    }

    /// Demonstrate wear leveling
    pub fn demonstrate_wear_leveling(&self) {
        info!("=== Wear Leveling Demonstration ===");
        
        let mut wear_manager = WearLevelingManager::new(WearLevelingStrategy::Dynamic);
        
        // Add SSD device
        wear_manager.add_ssd_device(BlockDeviceId(1), 1_000_000_000); // 1 billion sectors
        
        // Record some write operations
        wear_manager.record_write(BlockDeviceId(1), 1000, 8);
        wear_manager.record_write(BlockDeviceId(1), 2000, 16);
        wear_manager.record_write(BlockDeviceId(1), 3000, 4);
        
        // Perform wear leveling
        let _ = wear_manager.perform_wear_leveling(BlockDeviceId(1));
        
        // Get wear statistics
        let stats = wear_manager.get_wear_statistics(BlockDeviceId(1)).unwrap_or_default();
        info!("Wear leveling stats: total writes: {}, total trims: {}, total erasures: {}", 
              stats.total_writes, stats.total_trims, stats.total_erasures);
        
        // Get device health
        let health = wear_manager.get_device_health(BlockDeviceId(1)).unwrap_or_default();
        info!("Device health: overall {:.2}%, healthy blocks: {}, failed blocks: {}", 
              health.overall_health * 100.0, health.healthy_blocks, health.failed_blocks);
        
        // Demonstrate different strategies
        for strategy in [
            WearLevelingStrategy::Static,
            WearLevelingStrategy::Dynamic,
            WearLevelingStrategy::Advanced,
            WearLevelingStrategy::Adaptive,
        ] {
            let mut strategy_manager = WearLevelingManager::new(strategy);
            strategy_manager.add_ssd_device(BlockDeviceId(2), 500_000_000);
            info!("Wear leveling strategy {:?}: configured successfully", strategy);
        }
    }

    /// Demonstrate error recovery
    pub fn demonstrate_error_recovery(&self) {
        info!("=== Error Recovery Demonstration ===");
        
        let mut recovery_manager = ErrorRecoveryManager::new();
        
        // Register device for error recovery
        recovery_manager.register_device(BlockDeviceId(1), 1_000_000);
        
        // Demonstrate error handling
        let hardware_error = crate::drivers::block::BlockDeviceError::HardwareError;
        let recovery_result = recovery_manager.handle_error(BlockDeviceId(1), hardware_error.clone());
        
        match recovery_result {
            Ok(_) => info!("Error recovery successful"),
            Err(e) => info!("Error recovery result: {:?}", e),
        }
        
        // Get recovery statistics
        let stats = recovery_manager.get_global_stats();
        info!("Recovery stats: total errors: {}, recovered: {}, permanent failures: {}", 
              stats.total_errors, stats.recovered_errors, stats.permanent_failures);
        
        // Get device health
        let health_info = recovery_manager.get_device_health(BlockDeviceId(1)).unwrap_or_default();
        info!("Device health info: healthy: {}, error count: {}, error rate: {:.4}", 
              health_info.is_healthy, health_info.error_count, health_info.current_error_rate);
        
        // Configure recovery parameters
        let config = crate::drivers::error_recovery::RecoveryConfig {
            max_retries: 10,
            retry_delay_ms: 200,
            exponential_backoff: true,
            backoff_factor: 1.5,
            max_retry_delay_ms: 10000,
            enable_sector_remapping: true,
            enable_device_switching: true,
            enable_performance_degradation: true,
            error_rate_threshold: 0.02,
            health_check_interval: Duration::from_secs(120),
        };
        
        recovery_manager.configure_recovery(config);
        info!("Recovery parameters configured with max retries: 10");
    }

    /// Demonstrate SD card operations
    pub fn demonstrate_sd_card_operations(&self) {
        info!("=== SD Card Operations Demonstration ===");
        
        if let Some(ref sd_card) = self.sd_card_driver {
            // Check card status
            info!("SD card state: {:?}", sd_card.get_state());
            info!("SD card ready: {}", sd_card.is_ready());
            
            // Get card information (would work after real initialization)
            match sd_card.get_card_info() {
                Ok(info) => {
                    info!("SD card info: type: {:?}, size: {} bytes, sectors: {}", 
                          info.card_type, info.total_size, info.total_sectors);
                }
                Err(e) => {
                    info!("SD card not yet initialized: {:?}", e);
                }
            }
            
            // Demonstrate SD card types and interfaces
            for card_type in [SdCardType::SdSc, SdCardType::SdHc, SdCardType::SdXc] {
                info!("SD card type {:?} supported", card_type);
            }
            
            for interface_mode in [SdInterfaceMode::Spi, SdInterfaceMode::Sd1Bit, SdInterfaceMode::Sd4Bit] {
                info!("SD interface mode {:?} supported", interface_mode);
            }
        }
    }

    /// Demonstrate complete workflow
    pub fn demonstrate_complete_workflow(&self) {
        info!("=== Complete Storage System Workflow ===");
        
        // 1. System initialization
        info!("1. Storage system initialized");
        
        // 2. Device discovery and registration
        info!("2. Devices discovered and registered");
        
        // 3. I/O operations with scheduling
        info!("3. Performing I/O operations with intelligent scheduling");
        
        // 4. Write caching optimization
        info!("4. Write cache optimizing performance");
        
        // 5. Wear leveling for SSDs
        info!("5. Wear leveling extending SSD lifespan");
        
        // 6. Error handling and recovery
        info!("6. Error recovery ensuring reliability");
        
        // 7. Performance monitoring
        info!("7. System monitoring and maintenance");
        
        info!("Complete workflow demonstration finished");
    }

    /// Get system overview
    pub fn get_system_overview(&self) -> StorageSystemOverview {
        StorageSystemOverview {
            total_devices: 1, // SD card + would include others
            scheduler_type: "Elevator/CFQ/Deadline".to_string(),
            cache_policy: "Write-Back with LRU".to_string(),
            wear_leveling_strategy: "Dynamic".to_string(),
            error_recovery_enabled: true,
            supported_interfaces: vec![
                "SATA".to_string(),
                "NVMe".to_string(),
                "USB Mass Storage".to_string(),
                "SD Card".to_string(),
            ],
            features: vec![
                "Multi-algorithm I/O scheduling".to_string(),
                "Intelligent write caching".to_string(),
                "Advanced wear leveling".to_string(),
                "Comprehensive error recovery".to_string(),
                "Performance monitoring".to_string(),
                "Device health management".to_string(),
            ],
        }
    }
}

/// Storage system overview information
#[derive(Debug, Clone)]
pub struct StorageSystemOverview {
    pub total_devices: usize,
    pub scheduler_type: String,
    pub cache_policy: String,
    pub wear_leveling_strategy: String,
    pub error_recovery_enabled: bool,
    pub supported_interfaces: Vec<String>,
    pub features: Vec<String>,
}

/// Run comprehensive storage system demonstration
pub fn run_storage_system_demo() {
    info!("Starting MultiOS Storage Device Drivers and Block Management Demo");
    
    let mut storage_demo = StorageSystemExample::new();
    
    // Initialize system
    let _ = storage_demo.initialize();
    
    // Run demonstrations
    storage_demo.demonstrate_io_scheduling();
    storage_demo.demonstrate_write_caching();
    storage_demo.demonstrate_wear_leveling();
    storage_demo.demonstrate_error_recovery();
    storage_demo.demonstrate_sd_card_operations();
    storage_demo.demonstrate_complete_workflow();
    
    // Get system overview
    let overview = storage_demo.get_system_overview();
    
    info!("=== Storage System Overview ===");
    info!("Total devices: {}", overview.total_devices);
    info!("Scheduler: {}", overview.scheduler_type);
    info!("Cache policy: {}", overview.cache_policy);
    info!("Wear leveling: {}", overview.wear_leveling_strategy);
    info!("Error recovery: {}", if overview.error_recovery_enabled { "Enabled" } else { "Disabled" });
    
    info!("Supported interfaces:");
    for interface in &overview.supported_interfaces {
        info!("  - {}", interface);
    }
    
    info!("Key features:");
    for feature in &overview.features {
        info!("  - {}", feature);
    }
    
    info!("Storage Device Drivers and Block Management demonstration completed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_system_overview() {
        let overview = StorageSystemOverview {
            total_devices: 4,
            scheduler_type: "Multi-algorithm".to_string(),
            cache_policy: "Advanced".to_string(),
            wear_leveling_strategy: "Adaptive".to_string(),
            error_recovery_enabled: true,
            supported_interfaces: vec!["SATA".to_string(), "NVMe".to_string()],
            features: vec!["I/O Scheduling".to_string()],
        };
        
        assert_eq!(overview.total_devices, 4);
        assert!(overview.error_recovery_enabled);
        assert_eq!(overview.supported_interfaces.len(), 2);
    }

    #[test]
    fn test_demo_creation() {
        let demo = StorageSystemExample::new();
        assert!(demo.block_manager.get_device_ids().is_empty());
    }

    #[test]
    fn test_complete_workflow() {
        let demo = StorageSystemExample::new();
        demo.demonstrate_complete_workflow();
        // This should complete without panicking
    }
}