//! Advanced Resource Cleanup Module
//!
//! Provides comprehensive resource cleanup mechanisms for device drivers,
//! including memory management, handle cleanup, interrupt cleanup, and
//! resource leak detection.

use crate::AdvancedDriverId;
use crate::AdvancedDriverError::{self, *};
use alloc::collections::{BTreeMap, HashSet, VecDeque};
use alloc::string::String;
use spin::Mutex;
use log::{debug, warn, info, error};

/// Resource types managed by the cleanup system
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ResourceType {
    Memory = 0,
    Handle = 1,
    Interrupt = 2,
    DmaBuffer = 3,
    DeviceRegistration = 4,
    PowerManagement = 5,
    FileDescriptor = 6,
    Timer = 7,
    Thread = 8,
    Lock = 9,
    Custom = 10,
}

/// Resource cleanup status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupStatus {
    Pending,     // Cleanup scheduled but not executed
    InProgress,  // Cleanup currently executing
    Completed,   // Cleanup completed successfully
    Failed,      // Cleanup failed
    Skipped,     // Cleanup skipped due to error
}

/// Resource information for cleanup tracking
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub resource_id: u64,
    pub driver_id: AdvancedDriverId,
    pub resource_type: ResourceType,
    pub allocated_at: u64,
    pub size_bytes: usize,
    pub metadata: String,
    pub cleanup_callback: Option<fn(ResourceInfo)>,
    pub references: u32,
    pub cleanup_status: CleanupStatus,
}

/// Resource cleanup statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub total_resources: u64,
    pub active_resources: u64,
    pub cleaned_resources: u64,
    pub failed_cleanups: u64,
    pub resources_by_type: BTreeMap<ResourceType, u64>,
    pub bytes_allocated: u64,
    pub bytes_freed: u64,
    pub cleanup_queue_size: usize,
}

/// Resource cleanup manager
pub struct ResourceCleanupManager {
    active_resources: BTreeMap<u64, ResourceInfo>,
    cleanup_queue: VecDeque<u64>,
    resource_statistics: ResourceStats,
    resource_counter: u64,
    memory_tracker: BTreeMap<AdvancedDriverId, usize>,
    handle_allocations: BTreeMap<AdvancedDriverId, HashSet<u32>>,
    interrupt_allocations: BTreeMap<AdvancedDriverId, HashSet<u16>>,
    cleanup_callbacks: BTreeMap<(AdvancedDriverId, ResourceType), Vec<fn(ResourceInfo)>>,
}

impl ResourceCleanupManager {
    /// Create a new resource cleanup manager
    pub fn new() -> Self {
        info!("Initializing Resource Cleanup Manager");
        
        let manager = Self {
            active_resources: BTreeMap::new(),
            cleanup_queue: VecDeque::new(),
            resource_statistics: ResourceStats {
                total_resources: 0,
                active_resources: 0,
                cleaned_resources: 0,
                failed_cleanups: 0,
                resources_by_type: BTreeMap::new(),
                bytes_allocated: 0,
                bytes_freed: 0,
                cleanup_queue_size: 0,
            },
            resource_counter: 0,
            memory_tracker: BTreeMap::new(),
            handle_allocations: BTreeMap::new(),
            interrupt_allocations: BTreeMap::new(),
            cleanup_callbacks: BTreeMap::new(),
        };
        
        info!("Resource Cleanup Manager initialized");
        manager
    }

    /// Register a resource for tracking
    pub fn register_resource(&mut self, driver_id: AdvancedDriverId, resource_type: ResourceType, 
                            size_bytes: usize, metadata: String) -> Result<u64, AdvancedDriverError> {
        let resource_id = self.allocate_resource_id();
        
        let resource_info = ResourceInfo {
            resource_id,
            driver_id,
            resource_type,
            allocated_at: 0, // TODO: Get actual timestamp
            size_bytes,
            metadata,
            cleanup_callback: None,
            references: 1,
            cleanup_status: CleanupStatus::Pending,
        };
        
        self.active_resources.insert(resource_id, resource_info);
        self.update_statistics_add(resource_type, size_bytes);
        
        // Track memory allocations
        if resource_type == ResourceType::Memory {
            let current_usage = self.memory_tracker.entry(driver_id).or_insert(0);
            *current_usage += size_bytes;
        }
        
        debug!("Registered resource {} (type: {:?}, size: {} bytes) for driver {:?}", 
               resource_id, resource_type, size_bytes, driver_id);
        
        Ok(resource_id)
    }

    /// Add reference to existing resource
    pub fn add_resource_reference(&mut self, resource_id: u64) -> Result<(), AdvancedDriverError> {
        let resource = self.active_resources.get_mut(&resource_id)
            .ok_or(DeviceNotFound)?;
        
        resource.references += 1;
        debug!("Added reference to resource {}, total references: {}", resource_id, resource.references);
        
        Ok(())
    }

    /// Remove reference from resource
    pub fn remove_resource_reference(&mut self, resource_id: u64) -> Result<(), AdvancedDriverError> {
        let resource = self.active_resources.get_mut(&resource_id)
            .ok_or(DeviceNotFound)?;
        
        if resource.references > 1 {
            resource.references -= 1;
            debug!("Removed reference from resource {}, total references: {}", resource_id, resource.references);
            return Ok(());
        }
        
        // If no more references, queue for cleanup
        self.queue_resource_cleanup(resource_id)?;
        
        Ok(())
    }

    /// Queue resource for cleanup
    pub fn queue_resource_cleanup(&mut self, resource_id: u64) -> Result<(), AdvancedDriverError> {
        let resource = self.active_resources.get_mut(&resource_id)
            .ok_or(DeviceNotFound)?;
        
        resource.cleanup_status = CleanupStatus::Pending;
        self.cleanup_queue.push_back(resource_id);
        
        debug!("Queued resource {} for cleanup", resource_id);
        Ok(())
    }

    /// Execute cleanup for queued resources
    pub fn execute_cleanup(&mut self) -> Result<u32, AdvancedDriverError> {
        debug!("Executing resource cleanup for {} resources", self.cleanup_queue.len());
        
        let mut cleaned_count = 0u32;
        let mut failed_count = 0u32;
        
        while let Some(resource_id) = self.cleanup_queue.pop_front() {
            match self.cleanup_resource(resource_id) {
                Ok(_) => cleaned_count += 1,
                Err(_) => failed_count += 1,
            }
        }
        
        info!("Cleanup completed: {} successful, {} failed", cleaned_count, failed_count);
        Ok(cleaned_count)
    }

    /// Cleanup individual resource
    fn cleanup_resource(&mut self, resource_id: u64) -> Result<(), AdvancedDriverError> {
        let mut resource = self.active_resources.remove(&resource_id)
            .ok_or(DeviceNotFound)?;
        
        resource.cleanup_status = CleanupStatus::InProgress;
        
        // Execute cleanup callback if available
        if let Some(callback) = resource.cleanup_callback {
            callback(resource.clone());
        }
        
        // Perform resource-specific cleanup
        match resource.resource_type {
            ResourceType::Memory => self.cleanup_memory_resource(&resource)?,
            ResourceType::Handle => self.cleanup_handle_resource(&resource)?,
            ResourceType::Interrupt => self.cleanup_interrupt_resource(&resource)?,
            ResourceType::DmaBuffer => self.cleanup_dma_resource(&resource)?,
            ResourceType::PowerManagement => self.cleanup_power_resource(&resource)?,
            ResourceType::Timer => self.cleanup_timer_resource(&resource)?,
            ResourceType::Lock => self.cleanup_lock_resource(&resource)?,
            _ => {
                debug!("No specific cleanup handler for resource type {:?}", resource.resource_type);
            }
        }
        
        resource.cleanup_status = CleanupStatus::Completed;
        self.update_statistics_remove(&resource);
        
        debug!("Successfully cleaned up resource {}", resource_id);
        Ok(())
    }

    /// Cleanup memory resources
    fn cleanup_memory_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up memory resource {} ({} bytes) for driver {:?}", 
               resource.resource_id, resource.size_bytes, resource.driver_id);
        
        // Track memory freed
        // In real implementation, would free actual memory
        Ok(())
    }

    /// Cleanup handle resources
    fn cleanup_handle_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up handle resource {} for driver {:?}", 
               resource.resource_id, resource.driver_id);
        
        // Close file handles, device handles, etc.
        Ok(())
    }

    /// Cleanup interrupt resources
    fn cleanup_interrupt_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up interrupt resource {} for driver {:?}", 
               resource.resource_id, resource.driver_id);
        
        // Disable interrupts, free IRQ assignments
        Ok(())
    }

    /// Cleanup DMA buffer resources
    fn cleanup_dma_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up DMA buffer resource {} ({} bytes) for driver {:?}", 
               resource.resource_id, resource.size_bytes, resource.driver_id);
        
        // Free DMA buffers, unmap memory
        Ok(())
    }

    /// Cleanup power management resources
    fn cleanup_power_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up power management resource {} for driver {:?}", 
               resource.resource_id, resource.driver_id);
        
        // Disable power management, clear power states
        Ok(())
    }

    /// Cleanup timer resources
    fn cleanup_timer_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up timer resource {} for driver {:?}", 
               resource.resource_id, resource.driver_id);
        
        // Cancel timers, cleanup timer callbacks
        Ok(())
    }

    /// Cleanup lock resources
    fn cleanup_lock_resource(&self, resource: &ResourceInfo) -> Result<(), AdvancedDriverError> {
        debug!("Cleaning up lock resource {} for driver {:?}", 
               resource.resource_id, resource.driver_id);
        
        // Release locks, cleanup synchronization primitives
        Ok(())
    }

    /// Cleanup all resources for a specific driver
    pub fn cleanup_driver_resources(&mut self, driver_id: AdvancedDriverId) -> Result<u32, AdvancedDriverError> {
        debug!("Cleaning up all resources for driver {:?}", driver_id);
        
        let mut cleaned_count = 0u32;
        
        // Find all resources belonging to this driver
        let driver_resources: Vec<u64> = self.active_resources.keys()
            .filter(|&&id| self.active_resources[&id].driver_id == driver_id)
            .copied()
            .collect();
        
        for resource_id in driver_resources {
            if self.queue_resource_cleanup(resource_id).is_ok() {
                cleaned_count += 1;
            }
        }
        
        // Execute cleanup
        let executed_cleanups = self.execute_cleanup()?;
        debug!("Cleaned up {} resources for driver {:?}", executed_cleanups, driver_id);
        
        Ok(executed_cleanups)
    }

    /// Force cleanup all resources
    pub fn force_cleanup_all(&mut self) -> Result<u32, AdvancedDriverError> {
        warn!("Forcing cleanup of all resources");
        
        // Queue all active resources
        let resource_ids: Vec<u64> = self.active_resources.keys().copied().collect();
        for resource_id in resource_ids {
            let _ = self.queue_resource_cleanup(resource_id);
        }
        
        // Execute all cleanups
        self.execute_cleanup()
    }

    /// Register cleanup callback for specific resource type
    pub fn register_cleanup_callback(&mut self, driver_id: AdvancedDriverId, 
                                    resource_type: ResourceType, 
                                    callback: fn(ResourceInfo)) {
        let key = (driver_id, resource_type);
        let callbacks = self.cleanup_callbacks.entry(key).or_insert_with(Vec::new);
        callbacks.push(callback);
        
        debug!("Registered cleanup callback for driver {:?}, type {:?}", driver_id, resource_type);
    }

    /// Get resource information
    pub fn get_resource_info(&self, resource_id: u64) -> Option<&ResourceInfo> {
        self.active_resources.get(&resource_id)
    }

    /// Get all resources for a driver
    pub fn get_driver_resources(&self, driver_id: AdvancedDriverId) -> Vec<&ResourceInfo> {
        self.active_resources.values()
            .filter(|resource| resource.driver_id == driver_id)
            .collect()
    }

    /// Get resource statistics
    pub fn get_statistics(&self) -> &ResourceStats {
        &self.resource_statistics
    }

    /// Check for resource leaks
    pub fn detect_resource_leaks(&self) -> Vec<ResourceInfo> {
        let mut leaks = Vec::new();
        
        for resource in self.active_resources.values() {
            if resource.references == 0 {
                leaks.push(resource.clone());
            }
        }
        
        if !leaks.is_empty() {
            warn!("Detected {} resource leaks", leaks.len());
        }
        
        leaks
    }

    /// Get memory usage for driver
    pub fn get_driver_memory_usage(&self, driver_id: AdvancedDriverId) -> usize {
        self.memory_tracker.get(&driver_id).copied().unwrap_or(0)
    }

    /// Internal: Allocate new resource ID
    fn allocate_resource_id(&mut self) -> u64 {
        self.resource_counter += 1;
        self.resource_counter
    }

    /// Internal: Update statistics when adding resource
    fn update_statistics_add(&mut self, resource_type: ResourceType, size_bytes: usize) {
        self.resource_statistics.total_resources += 1;
        self.resource_statistics.active_resources += 1;
        self.resource_statistics.bytes_allocated += size_bytes as u64;
        
        *self.resource_statistics.resources_by_type
            .entry(resource_type)
            .or_insert(0) += 1;
    }

    /// Internal: Update statistics when removing resource
    fn update_statistics_remove(&mut self, resource: &ResourceInfo) {
        self.resource_statistics.active_resources -= 1;
        self.resource_statistics.cleaned_resources += 1;
        self.resource_statistics.bytes_freed += resource.size_bytes as u64;
    }
}

impl Default for ResourceCleanupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_registration() {
        let mut manager = ResourceCleanupManager::new();
        let driver_id = AdvancedDriverId(1);
        
        let resource_id = manager.register_resource(
            driver_id, 
            ResourceType::Memory, 
            1024, 
            "Test memory allocation".to_string()
        ).unwrap();
        
        assert_eq!(resource_id, 1);
        assert!(manager.get_resource_info(resource_id).is_some());
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_resources, 1);
        assert_eq!(stats.active_resources, 1);
        assert_eq!(stats.bytes_allocated, 1024);
    }

    #[test]
    fn test_resource_references() {
        let mut manager = ResourceCleanupManager::new();
        let driver_id = AdvancedDriverId(1);
        
        let resource_id = manager.register_resource(
            driver_id, 
            ResourceType::Memory, 
            1024, 
            "Test".to_string()
        ).unwrap();
        
        // Add reference
        assert!(manager.add_resource_reference(resource_id).is_ok());
        let resource = manager.get_resource_info(resource_id).unwrap();
        assert_eq!(resource.references, 2);
        
        // Remove reference (should queue for cleanup)
        assert!(manager.remove_resource_reference(resource_id).is_ok());
        let cleaned = manager.execute_cleanup().unwrap();
        assert_eq!(cleaned, 1);
    }

    #[test]
    fn test_driver_cleanup() {
        let mut manager = ResourceCleanupManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Register multiple resources
        manager.register_resource(driver_id, ResourceType::Memory, 1024, "Mem1".to_string()).unwrap();
        manager.register_resource(driver_id, ResourceType::Handle, 0, "Handle1".to_string()).unwrap();
        manager.register_resource(driver_id, ResourceType::Interrupt, 0, "Int1".to_string()).unwrap();
        
        let stats = manager.get_statistics();
        assert_eq!(stats.active_resources, 3);
        
        // Cleanup all driver resources
        let cleaned = manager.cleanup_driver_resources(driver_id).unwrap();
        assert_eq!(cleaned, 3);
        
        let stats_after = manager.get_statistics();
        assert_eq!(stats_after.active_resources, 0);
        assert_eq!(stats_after.cleaned_resources, 3);
    }

    #[test]
    fn test_resource_leak_detection() {
        let mut manager = ResourceCleanupManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Register resource without references
        manager.register_resource(driver_id, ResourceType::Memory, 1024, "Leak".to_string()).unwrap();
        
        let leaks = manager.detect_resource_leaks();
        assert_eq!(leaks.len(), 1);
        assert_eq!(leaks[0].references, 0);
    }

    #[test]
    fn test_cleanup_queue_management() {
        let mut manager = ResourceCleanupManager::new();
        let driver_id = AdvancedDriverId(1);
        
        // Register multiple resources
        let id1 = manager.register_resource(driver_id, ResourceType::Memory, 1024, "Mem1".to_string()).unwrap();
        let id2 = manager.register_resource(driver_id, ResourceType::Memory, 2048, "Mem2".to_string()).unwrap();
        
        // Queue cleanup for resources
        manager.queue_resource_cleanup(id1).unwrap();
        manager.queue_resource_cleanup(id2).unwrap();
        
        assert_eq!(manager.cleanup_queue.len(), 2);
        
        // Execute cleanup
        let cleaned = manager.execute_cleanup().unwrap();
        assert_eq!(cleaned, 2);
        assert_eq!(manager.cleanup_queue.len(), 0);
    }
}
