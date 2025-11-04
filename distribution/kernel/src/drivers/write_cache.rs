//! Write Cache System
//! 
//! Advanced write caching system with write-back, write-through,
//! and write-around policies for block devices.

use crate::log::{info, warn, error};
use super::block::{BlockDeviceId, BlockOperation, BlockDeviceError};
use super::block_device_interface::BlockDeviceInterface;

use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap, collections::HashMap};
use alloc::sync::Arc;
use core::time::{Duration, Instant};

/// Cache policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CachePolicy {
    WriteThrough = 0,  // Write immediately to disk
    WriteBack = 1,     // Write to cache, flush later
    WriteAround = 2,   // Write directly to disk, bypass cache
}

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    device_id: BlockDeviceId,
    sector: u64,
    sector_count: u32,
    data: Vec<u8>,
    is_dirty: bool,
    last_accessed: Instant,
    access_count: u32,
    size: usize,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
struct CacheStats {
    hits: u64,
    misses: u64,
    dirty_writes: u64,
    clean_writes: u64,
    evictions: u64,
    flushes: u64,
    total_cache_size: usize,
    dirty_data_size: usize,
    hit_rate: f32,
    avg_access_time: Duration,
}

/// Device cache information
#[derive(Debug, Clone)]
struct DeviceCache {
    device_id: BlockDeviceId,
    cache_policy: CachePolicy,
    max_cache_size: usize,
    current_size: usize,
    entries: BTreeMap<u64, CacheEntry>, // sector -> cache entry
    lru_list: Vec<u64>, // sectors in LRU order
    dirty_sectors: BTreeMap<u64, CacheEntry>, // dirty sectors for flushing
    flush_pending: bool,
    last_flush: Instant,
}

/// Write Cache Manager
pub struct WriteCache {
    device_caches: RwLock<HashMap<BlockDeviceId, DeviceCache>>,
    global_max_size: usize,
    default_policy: CachePolicy,
    flush_interval: Duration,
    stats: Arc<RwLock<HashMap<BlockDeviceId, CacheStats>>>,
    total_cache_size: usize,
}

impl WriteCache {
    /// Create new write cache
    pub fn new(default_policy: CachePolicy) -> Self {
        info!("Initializing Write Cache with policy: {:?}", default_policy);
        
        Self {
            device_caches: RwLock::new(HashMap::new()),
            global_max_size: 256 * 1024 * 1024, // 256MB default
            default_policy,
            flush_interval: Duration::from_secs(30),
            stats: Arc::new(RwLock::new(HashMap::new())),
            total_cache_size: 0,
        }
    }

    /// Initialize the write cache
    pub fn init(&self) -> Result<(), BlockDeviceError> {
        info!("Initializing Write Cache system");
        
        // Initialize statistics for all devices
        let stats = self.stats.read();
        drop(stats); // Release lock
        
        info!("Write Cache initialized successfully");
        Ok(())
    }

    /// Add device to cache
    pub fn add_device(&self, device_id: BlockDeviceId, max_cache_size: usize, policy: CachePolicy) {
        info!("Adding device {:?} to cache with policy {:?}, max size: {} bytes", device_id, policy, max_cache_size);
        
        let mut device_caches = self.device_caches.write();
        
        let device_cache = DeviceCache {
            device_id,
            cache_policy: policy,
            max_cache_size,
            current_size: 0,
            entries: BTreeMap::new(),
            lru_list: Vec::new(),
            dirty_sectors: BTreeMap::new(),
            flush_pending: false,
            last_flush: Instant::now(),
        };
        
        device_caches.insert(device_id, device_cache);
        
        // Initialize statistics
        let mut stats = self.stats.write();
        stats.insert(device_id, CacheStats::default());
    }

    /// Remove device from cache
    pub fn remove_device(&self, device_id: BlockDeviceId) {
        info!("Removing device {:?} from cache", device_id);
        
        let mut device_caches = self.device_caches.write();
        device_caches.remove(&device_id);
        
        let mut stats = self.stats.write();
        stats.remove(&device_id);
    }

    /// Check cache for write optimization
    pub fn check_cache_write(&self, device_id: BlockDeviceId, sector: u64, sector_count: u32) -> Option<Vec<u8>> {
        let device_caches = self.device_caches.read();
        let device_cache = match device_caches.get(&device_id) {
            Some(cache) => cache,
            None => return None,
        };
        
        if device_cache.cache_policy == CachePolicy::WriteAround {
            return None;
        }
        
        // Check if exact match exists in cache
        for (cached_sector, entry) in &device_cache.entries {
            if *cached_sector == sector && entry.sector_count == sector_count {
                let mut stats = self.stats.write();
                if let Some(device_stats) = stats.get_mut(&device_id) {
                    device_stats.hits += 1;
                    device_stats.hit_rate = device_stats.hits as f32 / (device_stats.hits + device_stats.misses) as f32;
                }
                
                return Some(entry.data.clone());
            }
        }
        
        // Cache miss
        let mut stats = self.stats.write();
        if let Some(device_stats) = stats.get_mut(&device_id) {
            device_stats.misses += 1;
            device_stats.hit_rate = device_stats.hits as f32 / (device_stats.hits + device_stats.misses) as f32;
        }
        
        None
    }

    /// Update cache with write data
    pub fn update_cache(&self, device_id: BlockDeviceId, sector: u64, sector_count: u32, data: &[u8]) -> Result<(), BlockDeviceError> {
        let mut device_caches = self.device_caches.write();
        let device_cache = match device_caches.get_mut(&device_id) {
            Some(cache) => cache,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        let entry_size = (sector_count as usize) * 512; // Assume 512-byte sectors
        
        // Check if we need to evict entries
        if device_cache.current_size + entry_size > device_cache.max_cache_size {
            self.evict_entries(device_id, entry_size)?;
        }
        
        // Create or update cache entry
        let entry = CacheEntry {
            device_id,
            sector,
            sector_count,
            data: data.to_vec(),
            is_dirty: true,
            last_accessed: Instant::now(),
            access_count: 1,
            size: entry_size,
        };
        
        // Update LRU list
        self.update_lru_list(device_id, sector);
        
        // Insert or replace entry
        let old_entry = device_cache.entries.insert(sector, entry.clone());
        if let Some(old) = old_entry {
            device_cache.current_size -= old.size;
        }
        device_cache.current_size += entry_size;
        
        // Add to dirty list if policy is write-back
        if device_cache.cache_policy == CachePolicy::WriteBack {
            device_cache.dirty_sectors.insert(sector, entry);
            device_cache.flush_pending = true;
        }
        
        // Update statistics
        let mut stats = self.stats.write();
        if let Some(device_stats) = stats.get_mut(&device_id) {
            if device_cache.cache_policy == CachePolicy::WriteBack {
                device_stats.dirty_writes += 1;
                device_stats.dirty_data_size += entry_size;
            } else {
                device_stats.clean_writes += 1;
            }
        }
        
        info!("Cache updated for device {:?}, sector {}, size: {} bytes", device_id, sector, entry_size);
        
        Ok(())
    }

    /// Read from cache
    pub fn read_cache(&self, device_id: BlockDeviceId, sector: u64, sector_count: u32, buffer: &mut [u8]) -> Result<bool, BlockDeviceError> {
        let device_caches = self.device_caches.read();
        let device_cache = match device_caches.get(&device_id) {
            Some(cache) => cache,
            None => return Ok(false),
        };
        
        // Find matching entry
        let entry = match device_cache.entries.get(&sector) {
            Some(entry) => entry,
            None => return Ok(false),
        };
        
        if entry.sector_count != sector_count || buffer.len() < entry.data.len() {
            return Ok(false);
        }
        
        // Copy data from cache
        buffer[..entry.data.len()].copy_from_slice(&entry.data);
        
        // Update access information
        let mut device_caches = self.device_caches.write();
        let device_cache = device_caches.get_mut(&device_id).unwrap();
        
        if let Some(entry) = device_cache.entries.get_mut(&sector) {
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
        }
        
        // Update LRU
        self.update_lru_list(device_id, sector);
        
        // Update statistics
        let mut stats = self.stats.write();
        if let Some(device_stats) = stats.get_mut(&device_id) {
            device_stats.hits += 1;
            device_stats.hit_rate = device_stats.hits as f32 / (device_stats.hits + device_stats.misses) as f32;
        }
        
        info!("Cache hit for device {:?}, sector {}", device_id, sector);
        
        Ok(true)
    }

    /// Flush cache for a specific device
    pub fn flush_device(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        info!("Flushing cache for device {:?}", device_id);
        
        let device_caches = self.device_caches.read();
        let device_cache = match device_caches.get(&device_id) {
            Some(cache) => cache,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        // Get dirty sectors
        let dirty_sectors: Vec<_> = device_cache.dirty_sectors.keys().cloned().collect();
        
        if dirty_sectors.is_empty() {
            info!("No dirty sectors to flush for device {:?}", device_id);
            return Ok(());
        }
        
        // In a real implementation, this would write the dirty sectors to the device
        // For now, we'll simulate the flush operation
        let mut device_caches = self.device_caches.write();
        let device_cache = device_caches.get_mut(&device_id).unwrap();
        
        for sector in dirty_sectors {
            if let Some(entry) = device_cache.dirty_sectors.remove(&sector) {
                info!("Flushing sector {} ({} bytes)", sector, entry.size);
                device_cache.current_size -= entry.size;
            }
        }
        
        device_cache.flush_pending = false;
        device_cache.last_flush = Instant::now();
        
        // Update statistics
        let mut stats = self.stats.write();
        if let Some(device_stats) = stats.get_mut(&device_id) {
            device_stats.flushes += 1;
            device_stats.dirty_data_size = 0;
        }
        
        info!("Cache flush completed for device {:?}", device_id);
        
        Ok(())
    }

    /// Flush all dirty data
    pub fn flush_all(&self) -> Result<(), BlockDeviceError> {
        info!("Flushing all dirty cache data");
        
        let device_caches = self.device_caches.read();
        let device_ids: Vec<_> = device_caches.keys().cloned().collect();
        drop(device_caches);
        
        for device_id in device_ids {
            self.flush_device(device_id)?;
        }
        
        info!("All cache data flushed successfully");
        
        Ok(())
    }

    /// Periodic cache maintenance
    pub fn periodic_maintenance(&self) {
        let current_time = Instant::now();
        
        let device_caches = self.device_caches.read();
        for (device_id, cache) in device_caches.iter() {
            // Check if it's time to flush
            if current_time.duration_since(cache.last_flush) > self.flush_interval && cache.flush_pending {
                info!("Periodic flush triggered for device {:?}", device_id);
                
                // Release read lock and acquire write lock for flushing
                drop(device_caches);
                let _ = self.flush_device(*device_id);
                break; // Reacquire locks for next iteration
            }
            
            // Clean up old entries if cache is getting full
            if cache.current_size > cache.max_cache_size * 90 / 100 {
                drop(device_caches);
                let _ = self.evict_entries(*device_id, 1024 * 1024); // Evict 1MB
                break;
            }
        }
    }

    /// Evict entries from cache
    fn evict_entries(&self, device_id: BlockDeviceId, required_size: usize) -> Result<(), BlockDeviceError> {
        let mut device_caches = self.device_caches.write();
        let device_cache = match device_caches.get_mut(&device_id) {
            Some(cache) => cache,
            None => return Err(BlockDeviceError::DeviceNotFound),
        };
        
        info!("Evicting cache entries for device {:?}, need {} bytes", device_id, required_size);
        
        let mut freed_size = 0;
        let mut eviction_count = 0;
        
        // Sort entries by LRU (least recently used)
        let mut sectors_to_evict: Vec<_> = device_cache.lru_list.clone();
        sectors_to_evict.sort_by(|a, b| {
            let entry_a = &device_cache.entries[a];
            let entry_b = &device_cache.entries[b];
            entry_a.last_accessed.cmp(&entry_b.last_accessed)
        });
        
        // Evict least recently used entries
        for sector in sectors_to_evict {
            if freed_size >= required_size {
                break;
            }
            
            if let Some(entry) = device_cache.entries.remove(&sector) {
                // Remove from dirty list if present
                device_cache.dirty_sectors.remove(&sector);
                
                freed_size += entry.size;
                eviction_count += 1;
                device_cache.current_size -= entry.size;
                
                info!("Evicted sector {} ({} bytes)", sector, entry.size);
            }
        }
        
        // Update LRU list
        device_cache.lru_list.retain(|s| device_cache.entries.contains_key(s));
        
        // Update statistics
        let mut stats = self.stats.write();
        if let Some(device_stats) = stats.get_mut(&device_id) {
            device_stats.evictions += eviction_count as u64;
        }
        
        if freed_size < required_size {
            warn!("Could not free enough space, freed: {}, needed: {}", freed_size, required_size);
        } else {
            info!("Evicted {} entries, freed {} bytes", eviction_count, freed_size);
        }
        
        Ok(())
    }

    /// Update LRU list for a sector
    fn update_lru_list(&self, device_id: BlockDeviceId, sector: u64) {
        let mut device_caches = self.device_caches.write();
        let device_cache = device_caches.get_mut(&device_id).unwrap();
        
        // Remove sector from LRU list if it exists
        device_cache.lru_list.retain(|&s| s != sector);
        
        // Add to end (most recently used)
        device_cache.lru_list.push(sector);
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self, device_id: BlockDeviceId) -> Result<CacheStats, BlockDeviceError> {
        let stats = self.stats.read();
        match stats.get(&device_id) {
            Some(device_stats) => Ok(device_stats.clone()),
            None => Err(BlockDeviceError::DeviceNotFound),
        }
    }

    /// Get all cache statistics
    pub fn get_all_cache_stats(&self) -> HashMap<BlockDeviceId, CacheStats> {
        self.stats.read().clone()
    }

    /// Set cache policy for a device
    pub fn set_cache_policy(&self, device_id: BlockDeviceId, policy: CachePolicy) -> Result<(), BlockDeviceError> {
        let mut device_caches = self.device_caches.write();
        let device_cache = device_caches.get_mut(&device_id).unwrap();
        
        info!("Changing cache policy for device {:?} to {:?}", device_id, policy);
        
        // If changing from write-back to another policy, flush dirty data
        if device_cache.cache_policy == CachePolicy::WriteBack && policy != CachePolicy::WriteBack {
            drop(device_caches);
            self.flush_device(device_id)?;
            device_caches = self.device_caches.write();
            device_cache = device_caches.get_mut(&device_id).unwrap();
        }
        
        device_cache.cache_policy = policy;
        
        Ok(())
    }

    /// Get cache policy for a device
    pub fn get_cache_policy(&self, device_id: BlockDeviceId) -> Result<CachePolicy, BlockDeviceError> {
        let device_caches = self.device_caches.read();
        let device_cache = device_caches.get(&device_id).ok_or(BlockDeviceError::DeviceNotFound)?;
        Ok(device_cache.cache_policy)
    }

    /// Clear cache for a device
    pub fn clear_cache(&self, device_id: BlockDeviceId) -> Result<(), BlockDeviceError> {
        info!("Clearing cache for device {:?}", device_id);
        
        let mut device_caches = self.device_caches.write();
        let device_cache = device_caches.get_mut(&device_id).unwrap();
        
        // Clear all entries
        device_cache.entries.clear();
        device_cache.dirty_sectors.clear();
        device_cache.lru_list.clear();
        device_cache.current_size = 0;
        device_cache.flush_pending = false;
        
        Ok(())
    }

    /// Get cache efficiency metrics
    pub fn get_cache_efficiency(&self, device_id: BlockDeviceId) -> Result<CacheEfficiency, BlockDeviceError> {
        let stats = self.stats.read();
        let device_stats = stats.get(&device_id).ok_or(BlockDeviceError::DeviceNotFound)?;
        
        let device_caches = self.device_caches.read();
        let device_cache = device_caches.get(&device_id).unwrap();
        
        Ok(CacheEfficiency {
            hit_rate: device_stats.hit_rate,
            utilization: device_cache.current_size as f32 / device_cache.max_cache_size as f32,
            dirty_ratio: if device_cache.current_size > 0 {
                device_stats.dirty_data_size as f32 / device_cache.current_size as f32
            } else {
                0.0
            },
            total_requests: device_stats.hits + device_stats.misses,
            effective_throughput: if device_stats.hits > 0 {
                // Estimate based on hit rate and I/O patterns
                device_stats.hits as f32 / (device_stats.hits + device_stats.misses) as f32
            } else {
                0.0
            },
        })
    }
}

/// Cache efficiency metrics
#[derive(Debug, Clone, Default)]
pub struct CacheEfficiency {
    pub hit_rate: f32,
    pub utilization: f32,
    pub dirty_ratio: f32,
    pub total_requests: u64,
    pub effective_throughput: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = WriteCache::new(CachePolicy::WriteBack);
        assert_eq!(cache.default_policy, CachePolicy::WriteBack);
    }

    #[test]
    fn test_cache_entry() {
        let entry = CacheEntry {
            device_id: BlockDeviceId(0),
            sector: 100,
            sector_count: 8,
            data: vec![0u8; 4096],
            is_dirty: true,
            last_accessed: Instant::now(),
            access_count: 1,
            size: 4096,
        };
        
        assert_eq!(entry.sector, 100);
        assert_eq!(entry.is_dirty, true);
    }

    #[test]
    fn test_cache_efficiency() {
        let efficiency = CacheEfficiency::default();
        assert_eq!(efficiency.hit_rate, 0.0);
        assert_eq!(efficiency.utilization, 0.0);
    }
}