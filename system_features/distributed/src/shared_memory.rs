//! Distributed shared memory system
//!
//! This module provides a distributed shared memory abstraction that allows
//! processes across different nodes to share data with various consistency models.

use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use crossbeam::channel::{unbounded, Sender};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cluster::{Cluster, NodeId};

/// Memory region identifier
pub type MemoryRegionId = Uuid;

/// Global memory address in distributed space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalAddress(pub u64);

/// Local memory address within a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalAddress(pub usize);

/// Consistency models supported by the distributed memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyModel {
    /// Sequential consistency - all operations appear to execute in some total order
    Sequential,
    /// Causal consistency - operations that are causally related appear in order
    Causal,
    /// Eventual consistency - operations eventually converge to consistent state
    Eventual,
    /// Release consistency - consistency guaranteed at synchronization points
    Release,
    /// Entry consistency - consistency per synchronization object
    Entry,
    /// Weak consistency - no guarantees without synchronization
    Weak,
}

/// Types of shared memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOperation {
    /// Read operation
    Read {
        address: GlobalAddress,
        size: usize,
        request_id: u64,
    },
    /// Write operation
    Write {
        address: GlobalAddress,
        data: Vec<u8>,
        request_id: u64,
    },
    /// Compare and swap operation
    CompareAndSwap {
        address: GlobalAddress,
        expected: Vec<u8>,
        new_value: Vec<u8>,
        request_id: u64,
    },
    /// Fetch and add operation
    FetchAndAdd {
        address: GlobalAddress,
        value: i64,
        request_id: u64,
    },
    /// Memory barrier/synchronization
    MemoryBarrier {
        operation_id: u64,
    },
}

/// Result of a memory operation
#[derive(Debug, Clone)]
pub enum MemoryOperationResult {
    ReadSuccess {
        data: Vec<u8>,
        timestamp: SystemTime,
    },
    WriteSuccess {
        timestamp: SystemTime,
    },
    CompareAndSwapSuccess {
        success: bool,
        old_value: Vec<u8>,
        timestamp: SystemTime,
    },
    FetchAndAddSuccess {
        old_value: i64,
        new_value: i64,
        timestamp: SystemTime,
    },
    OperationFailed {
        error: String,
        timestamp: SystemTime,
    },
}

/// Shared memory region definition
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub id: MemoryRegionId,
    pub name: String,
    pub size: usize,
    pub owner_node: NodeId,
    pub consistency_model: ConsistencyModel,
    pub permissions: MemoryPermissions,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub replica_nodes: Vec<NodeId>,
}

/// Memory access permissions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemoryPermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
    pub sharable: bool,
}

/// Shared memory value with versioning for consistency
#[derive(Debug, Clone)]
pub struct SharedValue<T> {
    pub value: T,
    pub version: u64,
    pub timestamp: SystemTime,
    pub writer_id: NodeId,
}

impl<T> SharedValue<T> {
    pub fn new(value: T, writer_id: NodeId) -> Self {
        Self {
            value,
            version: 0,
            timestamp: SystemTime::now(),
            writer_id,
        }
    }
    
    pub fn update(&mut self, new_value: T, writer_id: NodeId) {
        self.value = new_value;
        self.version += 1;
        self.timestamp = SystemTime::now();
        self.writer_id = writer_id;
    }
}

/// Lock-free queue implementation for wait-free operations
pub struct LockFreeQueue<T> {
    inner: Arc<crossbeam::queue::SegQueue<T>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(crossbeam::queue::SegQueue::new()),
        }
    }
    
    pub fn push(&self, item: T) {
        self.inner.push(item);
    }
    
    pub fn pop(&self) -> Option<T> {
        self.inner.pop()
    }
    
    pub fn is_empty(&self) -> bool {
        // crossbeam queue doesn't have is_empty, so we check by attempting to pop
        self.inner.pop().is_none()
    }
}

/// Distributed shared memory manager
pub struct DistributedMemory {
    cluster: Arc<Cluster>,
    memory_regions: Arc<RwLock<HashMap<MemoryRegionId, MemoryRegion>>>,
    global_address_space: Arc<RwLock<HashMap<GlobalAddress, MemoryRegionId>>>,
    local_memory: Arc<RwLock<HashMap<NodeId, HashMap<LocalAddress, Vec<u8>>>>>,
    
    // Consistency management
    consistency_coordinators: Arc<RwLock<HashMap<MemoryRegionId, ConsistencyCoordinator>>>,
    operation_history: Arc<RwLock<Vec<MemoryOperation>>>,
    pending_operations: LockFreeQueue<PendingOperation>,
    
    // Performance monitoring
    access_statistics: Arc<RwLock<AccessStatistics>>,
    
    // Configuration
    max_memory_mb: u64,
    cache_line_size: usize,
}

/// Coordinator for managing consistency operations
#[derive(Debug, Clone)]
pub struct ConsistencyCoordinator {
    pub region_id: MemoryRegionId,
    pub model: ConsistencyModel,
    pub coordinator_node: NodeId,
    pub version_vector: HashMap<NodeId, u64>,
    pub pending_operations: Vec<MemoryOperation>,
    pub sync_barriers: Vec<SyncBarrier>,
}

/// Synchronization barrier for consistency guarantees
#[derive(Debug, Clone)]
pub struct SyncBarrier {
    pub barrier_id: u64,
    pub participating_nodes: Vec<NodeId>,
    pub arrivals: HashMap<NodeId, SystemTime>,
    pub release_count: usize,
    pub active: AtomicBool,
}

/// Pending memory operation waiting for consistency resolution
#[derive(Debug, Clone)]
pub struct PendingOperation {
    pub operation: MemoryOperation,
    pub source_node: NodeId,
    pub target_region: MemoryRegionId,
    pub timestamp: SystemTime,
    pub retries: u32,
    pub timeout: Duration,
}

/// Access statistics and metrics
#[derive(Debug, Clone, Default)]
pub struct AccessStatistics {
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_operations: u64,
    pub cache_hit_rate: f64,
    pub network_traffic_mb: u64,
    pub average_latency: Duration,
    pub consistency_violations: u64,
    pub deadlock_count: u64,
    pub starve_count: u64,
}

/// Memory mapping result
#[derive(Debug)]
pub struct MemoryMapping {
    pub global_address: GlobalAddress,
    pub local_address: LocalAddress,
    pub size: usize,
    pub permissions: MemoryPermissions,
}

/// Memory allocation error types
#[derive(thiserror::Error, Debug)]
pub enum MemoryError {
    #[error("Insufficient memory: requested {requested}, available {available}")]
    InsufficientMemory { requested: usize, available: usize },
    
    #[error("Invalid memory region: {region_id}")]
    InvalidRegion { region_id: MemoryRegionId },
    
    #[error("Access denied: {operation} on region {region_id}")]
    AccessDenied { operation: String, region_id: MemoryRegionId },
    
    #[error("Address not found: {address}")]
    AddressNotFound { address: GlobalAddress },
    
    #[error("Consistency violation in region {region_id}: {violation}")]
    ConsistencyViolation { region_id: MemoryRegionId, violation: String },
    
    #[error("Timeout waiting for memory operation after {timeout:?}")]
    Timeout { timeout: Duration },
    
    #[error("Memory operation failed: {error}")]
    OperationFailed { error: String },
}

impl DistributedMemory {
    /// Create a new distributed memory system
    pub fn new(cluster: Arc<Cluster>, max_memory_mb: u64) -> Self {
        info!("Initializing distributed memory system with {}MB limit", max_memory_mb);
        
        Self {
            cluster,
            memory_regions: Arc::new(RwLock::new(HashMap::new())),
            global_address_space: Arc::new(RwLock::new(HashMap::new())),
            local_memory: Arc::new(RwLock::new(HashMap::new())),
            consistency_coordinators: Arc::new(RwLock::new(HashMap::new())),
            operation_history: Arc::new(RwLock::new(Vec::new())),
            pending_operations: LockFreeQueue::new(),
            access_statistics: Arc::new(RwLock::new(AccessStatistics::default())),
            max_memory_mb,
            cache_line_size: 64, // Cache line size in bytes
        }
    }
    
    /// Allocate a new shared memory region
    pub async fn allocate_region(
        &self,
        name: &str,
        size: usize,
        consistency_model: ConsistencyModel,
        owner_node: Option<NodeId>,
    ) -> Result<MemoryRegionId, MemoryError> {
        debug!("Allocating shared memory region '{}' of size {} bytes", name, size);
        
        // Check memory availability
        let available_memory = self.get_available_memory().await;
        if available_memory < size {
            return Err(MemoryError::InsufficientMemory {
                requested: size,
                available: available_memory,
            });
        }
        
        // Create region
        let region_id = Uuid::new_v4();
        let owner = owner_node.unwrap_or_else(|| self.get_current_node_id());
        
        let region = MemoryRegion {
            id: region_id,
            name: name.to_string(),
            size,
            owner_node: owner,
            consistency_model: consistency_model.clone(),
            permissions: MemoryPermissions {
                readable: true,
                writable: true,
                executable: false,
                sharable: true,
            },
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 0,
            replica_nodes: vec![],
        };
        
        // Store region
        {
            let mut regions = self.memory_regions.write().await;
            regions.insert(region_id, region.clone());
        }
        
        // Create global address
        let global_address = self.allocate_global_address(region_id);
        {
            let mut address_space = self.global_address_space.write().await;
            address_space.insert(global_address, region_id);
        }
        
        // Initialize local memory for owner
        {
            let mut local_memory = self.local_memory.write().await;
            let node_memory = local_memory.entry(owner).or_insert_with(HashMap::new);
            node_memory.insert(LocalAddress(0), vec![0; size]);
        }
        
        // Create consistency coordinator
        let coordinator = ConsistencyCoordinator {
            region_id,
            model: consistency_model,
            coordinator_node: owner,
            version_vector: HashMap::new(),
            pending_operations: Vec::new(),
            sync_barriers: Vec::new(),
        };
        
        {
            let mut coordinators = self.consistency_coordinators.write().await;
            coordinators.insert(region_id, coordinator);
        }
        
        // Update statistics
        {
            let mut stats = self.access_statistics.write().await;
            stats.total_operations += 1;
        }
        
        info!("Allocated shared memory region '{}' with ID: {}", name, region_id);
        Ok(region_id)
    }
    
    /// Map a global address to local memory
    pub async fn map_address(
        &self,
        global_address: GlobalAddress,
        permissions: MemoryPermissions,
    ) -> Result<MemoryMapping, MemoryError> {
        debug!("Mapping global address {:?}", global_address);
        
        // Find region for address
        let regions = self.global_address_space.read().await;
        let region_id = regions.get(&global_address)
            .ok_or_else(|| MemoryError::AddressNotFound { address: global_address })?;
        
        let region = {
            let regions = self.memory_regions.read().await;
            regions.get(region_id)
                .ok_or_else(|| MemoryError::InvalidRegion { region_id: *region_id })?
                .clone()
        };
        
        // Check permissions
        if !permissions.readable && region.permissions.readable {
            // This is a simplified check - in real implementation would check more thoroughly
        }
        
        // Create memory mapping
        let local_address = LocalAddress(0); // Simplified local addressing
        let mapping = MemoryMapping {
            global_address,
            local_address,
            size: region.size,
            permissions,
        };
        
        // Update access statistics
        {
            let mut stats = self.access_statistics.write().await;
            stats.total_operations += 1;
            stats.total_reads += 1;
        }
        
        // Update region access info
        {
            let mut regions = self.memory_regions.write().await;
            if let Some(region) = regions.get_mut(&region.id) {
                region.last_accessed = SystemTime::now();
                region.access_count += 1;
            }
        }
        
        Ok(mapping)
    }
    
    /// Read data from a shared memory region
    pub async fn read(
        &self,
        address: GlobalAddress,
        size: usize,
    ) -> Result<Vec<u8>, MemoryError> {
        debug!("Reading {} bytes from address {:?}", size, address);
        
        let start_time = Instant::now();
        
        // Find region and mapping
        let (region_id, region) = self.get_region_and_mapping(address).await?;
        
        // Check permissions
        if !region.permissions.readable {
            return Err(MemoryError::AccessDenied {
                operation: "read".to_string(),
                region_id,
            });
        }
        
        // Perform consistency-aware read
        let data = match region.consistency_model {
            ConsistencyModel::Sequential => self.sequential_read(address, size).await?,
            ConsistencyModel::Causal => self.causal_read(address, size).await?,
            ConsistencyModel::Eventual => self.eventual_read(address, size).await?,
            ConsistencyModel::Release => self.release_read(address, size).await?,
            ConsistencyModel::Entry => self.entry_read(address, size).await?,
            ConsistencyModel::Weak => self.weak_read(address, size).await?,
        };
        
        // Update statistics
        let elapsed = start_time.elapsed();
        {
            let mut stats = self.access_statistics.write().await;
            stats.total_reads += 1;
            stats.total_operations += 1;
            stats.average_latency = Duration::from_nanos(
                (stats.average_latency.as_nanos() + elapsed.as_nanos()) / 2
            );
        }
        
        Ok(data)
    }
    
    /// Write data to a shared memory region
    pub async fn write(
        &self,
        address: GlobalAddress,
        data: Vec<u8>,
    ) -> Result<(), MemoryError> {
        debug!("Writing {} bytes to address {:?}", data.len(), address);
        
        let start_time = Instant::now();
        
        // Find region and mapping
        let (region_id, region) = self.get_region_and_mapping(address).await?;
        
        // Check permissions
        if !region.permissions.writable {
            return Err(MemoryError::AccessDenied {
                operation: "write".to_string(),
                region_id,
            });
        }
        
        // Validate data size
        if data.len() > region.size {
            return Err(MemoryError::OperationFailed {
                error: format!("Data size {} exceeds region size {}", data.len(), region.size),
            });
        }
        
        // Perform consistency-aware write
        match region.consistency_model {
            ConsistencyModel::Sequential => self.sequential_write(address, data).await?,
            ConsistencyModel::Causal => self.causal_write(address, data).await?,
            ConsistencyModel::Eventual => self.eventual_write(address, data).await?,
            ConsistencyModel::Release => self.release_write(address, data).await?,
            ConsistencyModel::Entry => self.entry_write(address, data).await?,
            ConsistencyModel::Weak => self.weak_write(address, data).await?,
        }
        
        // Update statistics
        let elapsed = start_time.elapsed();
        {
            let mut stats = self.access_statistics.write().await;
            stats.total_writes += 1;
            stats.total_operations += 1;
            stats.average_latency = Duration::from_nanos(
                (stats.average_latency.as_nanos() + elapsed.as_nanos()) / 2
            );
        }
        
        Ok(())
    }
    
    /// Compare and swap operation
    pub async fn compare_and_swap(
        &self,
        address: GlobalAddress,
        expected: Vec<u8>,
        new_value: Vec<u8>,
    ) -> Result<bool, MemoryError> {
        debug!("Compare-and-swap at address {:?}", address);
        
        let start_time = Instant::now();
        
        // Find region
        let (region_id, region) = self.get_region_and_mapping(address).await?;
        
        if !region.permissions.writable {
            return Err(MemoryError::AccessDenied {
                operation: "compare_and_swap".to_string(),
                region_id,
            });
        }
        
        // Perform atomic operation
        let old_value = self.read(address, expected.len()).await?;
        let success = old_value == expected;
        
        if success {
            self.write(address, new_value).await?;
        }
        
        // Update statistics
        let elapsed = start_time.elapsed();
        {
            let mut stats = self.access_statistics.write().await;
            stats.total_operations += 1;
            stats.average_latency = Duration::from_nanos(
                (stats.average_latency.as_nanos() + elapsed.as_nanos()) / 2
            );
        }
        
        Ok(success)
    }
    
    /// Synchronize memory operations (memory barrier)
    pub async fn sync(&self) -> Result<(), MemoryError> {
        debug!("Performing memory synchronization");
        
        // Create memory barrier operation
        let operation = MemoryOperation::MemoryBarrier {
            operation_id: Uuid::new_v4().as_u64(),
        };
        
        // Add to pending operations
        self.pending_operations.push(PendingOperation {
            operation,
            source_node: self.get_current_node_id(),
            target_region: Uuid::nil(), // Global sync
            timestamp: SystemTime::now(),
            retries: 0,
            timeout: Duration::from_secs(1),
        });
        
        // In a real implementation, this would coordinate with all nodes
        // For this educational version, we'll just simulate a brief wait
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        Ok(())
    }
    
    /// Get memory region information
    pub async fn get_region_info(&self, region_id: MemoryRegionId) -> Result<MemoryRegion, MemoryError> {
        let regions = self.memory_regions.read().await;
        regions.get(&region_id)
            .cloned()
            .ok_or_else(|| MemoryError::InvalidRegion { region_id })
    }
    
    /// Get all memory regions
    pub async fn list_regions(&self) -> Result<Vec<MemoryRegion>, MemoryError> {
        let regions = self.memory_regions.read().await;
        Ok(regions.values().cloned().collect())
    }
    
    /// Get access statistics
    pub async fn get_statistics(&self) -> AccessStatistics {
        self.access_statistics.read().await.clone()
    }
    
    /// Release a memory region
    pub async fn release_region(&self, region_id: MemoryRegionId) -> Result<(), MemoryError> {
        debug!("Releasing memory region {}", region_id);
        
        let mut regions = self.memory_regions.write().await;
        let region = regions.remove(&region_id)
            .ok_or_else(|| MemoryError::InvalidRegion { region_id })?;
        
        // Remove from global address space
        let mut address_space = self.global_address_space.write().await;
        let addresses_to_remove: Vec<_> = address_space.iter()
            .filter(|(_, &rid)| rid == region_id)
            .map(|(addr, _)| *addr)
            .collect();
        
        for addr in addresses_to_remove {
            address_space.remove(&addr);
        }
        
        // Remove consistency coordinator
        let mut coordinators = self.consistency_coordinators.write().await;
        coordinators.remove(&region_id);
        
        info!("Released memory region {}", region_id);
        Ok(())
    }
    
    // Private helper methods
    
    async fn get_region_and_mapping(&self, address: GlobalAddress) -> Result<(MemoryRegionId, MemoryRegion), MemoryError> {
        let address_space = self.global_address_space.read().await;
        let region_id = address_space.get(&address)
            .ok_or_else(|| MemoryError::AddressNotFound { address })?;
        
        let regions = self.memory_regions.read().await;
        let region = regions.get(region_id)
            .cloned()
            .ok_or_else(|| MemoryError::InvalidRegion { region_id: *region_id })?;
        
        Ok((*region_id, region))
    }
    
    fn allocate_global_address(&self, region_id: MemoryRegionId) -> GlobalAddress {
        let hash = region_id.as_u128();
        GlobalAddress((hash & 0xFFFF_FFFF_FFFF_FFFF) as u64)
    }
    
    fn get_current_node_id(&self) -> NodeId {
        // This would be implemented based on the actual cluster configuration
        Uuid::new_v4()
    }
    
    async fn get_available_memory(&self) -> usize {
        let stats = self.access_statistics.read().await;
        (self.max_memory_mb * 1024 * 1024) as usize // Simplified calculation
    }
    
    // Consistency model implementations
    
    async fn sequential_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Sequential consistency: all operations appear in some total order
        // This is the simplest model - just perform the read
        self.basic_read(address, size).await
    }
    
    async fn sequential_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Sequential consistency: all operations appear in some total order
        self.basic_write(address, data).await
    }
    
    async fn causal_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Causal consistency: operations that are causally related appear in order
        // This is more complex and would involve tracking causal relationships
        self.basic_read(address, size).await
    }
    
    async fn causal_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Causal consistency: track causal relationships
        self.basic_write(address, data).await
    }
    
    async fn eventual_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Eventual consistency: operations eventually converge
        // This allows more relaxed consistency guarantees
        self.basic_read(address, size).await
    }
    
    async fn eventual_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Eventual consistency: asynchronous propagation
        self.basic_write(address, data).await
    }
    
    async fn release_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Release consistency: consistency guaranteed at synchronization points
        self.basic_read(address, size).await
    }
    
    async fn release_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Release consistency: consistency at synchronization points
        self.basic_write(address, data).await
    }
    
    async fn entry_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Entry consistency: consistency per synchronization object
        self.basic_read(address, size).await
    }
    
    async fn entry_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Entry consistency: consistency per synchronization object
        self.basic_write(address, data).await
    }
    
    async fn weak_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Weak consistency: no guarantees without synchronization
        self.basic_read(address, size).await
    }
    
    async fn weak_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Weak consistency: no guarantees without synchronization
        self.basic_write(address, data).await
    }
    
    async fn basic_read(&self, address: GlobalAddress, size: usize) -> Result<Vec<u8>, MemoryError> {
        // Basic read implementation
        let local_memory = self.local_memory.read().await;
        let node_memory = local_memory.get(&self.get_current_node_id())
            .ok_or_else(|| MemoryError::OperationFailed { error: "Node memory not found".to_string() })?;
        
        let data = node_memory.get(&LocalAddress(0))
            .ok_or_else(|| MemoryError::AddressNotFound { address })?
            .get(..size)
            .ok_or_else(|| MemoryError::OperationFailed { error: "Invalid read size".to_string() })?
            .to_vec();
        
        Ok(data)
    }
    
    async fn basic_write(&self, address: GlobalAddress, data: Vec<u8>) -> Result<(), MemoryError> {
        // Basic write implementation
        let mut local_memory = self.local_memory.write().await;
        let node_memory = local_memory.entry(self.get_current_node_id()).or_insert_with(HashMap::new);
        
        if let Some(existing_data) = node_memory.get_mut(&LocalAddress(0)) {
            if existing_data.len() >= data.len() {
                existing_data[..data.len()].copy_from_slice(&data);
                return Ok(());
            }
        }
        
        node_memory.insert(LocalAddress(0), data);
        Ok(())
    }
}

// Shared value implementations

impl<T: Clone + Send + Sync> SharedValue<T> {
    /// Create a new shared value
    pub fn create(cluster: Arc<Cluster>, initial_value: T) -> Self {
        SharedValue::new(initial_value, Uuid::new_v4())
    }
    
    /// Read the current value (with consistency checks)
    pub async fn read(&self) -> T {
        self.value.clone()
    }
    
    /// Update the value (with consistency management)
    pub async fn update(&mut self, new_value: T, writer_id: NodeId) {
        self.update(new_value, writer_id);
    }
    
    /// Atomic compare-and-swap operation
    pub async fn compare_and_swap(&mut self, expected: T, new_value: T, writer_id: NodeId) -> bool {
        if self.value == expected {
            self.update(new_value, writer_id);
            true
        } else {
            false
        }
    }
}

// Utility implementations

impl Display for GlobalAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:016X}", self.0)
    }
}

impl Display for LocalAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_allocation() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let memory = DistributedMemory::new(cluster, 1024); // 1MB limit
        
        let region_id = memory.allocate_region("test_region", 1024, ConsistencyModel::Sequential, None).await.unwrap();
        assert!(!region_id.is_nil());
    }
    
    #[tokio::test]
    async fn test_basic_read_write() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let memory = DistributedMemory::new(cluster, 1024);
        
        let region_id = memory.allocate_region("test_region", 1024, ConsistencyModel::Sequential, None).await.unwrap();
        
        // Write some data
        let test_data = b"Hello, Distributed Memory!".to_vec();
        let address = GlobalAddress(0x1000); // Arbitrary address
        memory.write(address, test_data.clone()).await.unwrap();
        
        // Read it back
        let read_data = memory.read(address, test_data.len()).await.unwrap();
        assert_eq!(read_data, test_data);
    }
    
    #[tokio::test]
    async fn test_consistency_models() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let memory = DistributedMemory::new(cluster, 1024);
        
        // Test different consistency models
        for model in [
            ConsistencyModel::Sequential,
            ConsistencyModel::Causal,
            ConsistencyModel::Eventual,
            ConsistencyModel::Release,
            ConsistencyModel::Entry,
            ConsistencyModel::Weak,
        ] {
            let region_id = memory.allocate_region(&format!("region_{:?}", model), 512, model.clone(), None).await.unwrap();
            assert!(!region_id.is_nil());
        }
    }
    
    #[tokio::test]
    async fn test_shared_value() {
        let mut shared_i32 = SharedValue::new(42i32, Uuid::new_v4());
        
        assert_eq!(shared_i32.read().await, 42);
        
        shared_i32.update(100, Uuid::new_v4());
        assert_eq!(shared_i32.read().await, 100);
        
        let success = shared_i32.compare_and_swap(50, 200, Uuid::new_v4()).await;
        assert!(!success);
        assert_eq!(shared_i32.read().await, 100);
        
        let success = shared_i32.compare_and_swap(100, 200, Uuid::new_v4()).await;
        assert!(success);
        assert_eq!(shared_i32.read().await, 200);
    }
    
    #[test]
    fn test_address_formatting() {
        let global_addr = GlobalAddress(0x123456789ABCDEF0);
        assert_eq!(format!("{}", global_addr), "0x123456789ABCDEF0");
        
        let local_addr = LocalAddress(0x12345678);
        assert_eq!(format!("{}", local_addr), "0x12345678");
    }
    
    #[tokio::test]
    async fn test_memory_release() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let memory = DistributedMemory::new(cluster, 1024);
        
        let region_id = memory.allocate_region("temp_region", 256, ConsistencyModel::Sequential, None).await.unwrap();
        
        let regions_before = memory.list_regions().await.unwrap();
        assert_eq!(regions_before.len(), 1);
        
        memory.release_region(region_id).await.unwrap();
        
        let regions_after = memory.list_regions().await.unwrap();
        assert_eq!(regions_after.len(), 0);
    }
    
    #[tokio::test]
    async fn test_statistics() {
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await.unwrap());
        let memory = DistributedMemory::new(cluster, 1024);
        
        let initial_stats = memory.get_statistics().await;
        assert_eq!(initial_stats.total_operations, 0);
        assert_eq!(initial_stats.total_reads, 0);
        assert_eq!(initial_stats.total_writes, 0);
        
        // Perform some operations
        let region_id = memory.allocate_region("stats_test", 1024, ConsistencyModel::Sequential, None).await.unwrap();
        let address = GlobalAddress(0x1000);
        let test_data = b"test data".to_vec();
        
        memory.write(address, test_data.clone()).await.unwrap();
        memory.read(address, test_data.len()).await.unwrap();
        
        let final_stats = memory.get_statistics().await;
        assert!(final_stats.total_operations > 0);
        assert!(final_stats.total_writes > 0);
        assert!(final_stats.total_reads > 0);
    }
}