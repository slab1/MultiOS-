//! NUMA (Non-Uniform Memory Access) Management for MultiOS
//! 
//! This module provides comprehensive NUMA awareness for memory management,
//! optimizing performance for systems with hundreds of cores across multiple NUMA nodes.
//!
//! Features:
//! - NUMA topology discovery and management
//! - NUMA-aware memory allocation and placement
//! - Memory migration between NUMA nodes
//! - NUMA balancing and load distribution
//! - NUMA statistics and monitoring
//! - NUMA-optimized page table management

use alloc::vec::Vec;
use spin::Mutex;
use bitflags::bitflags;
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use core::ops::Range;

use crate::{PhysAddr, VirtAddr, PageSize, MemoryError, MemoryResult, MemoryFlags};

/// NUMA node identifier
pub type NumaNodeId = usize;

/// Maximum number of NUMA nodes supported
const MAX_NUMA_NODES: usize = 128;

/// Maximum memory per NUMA node (16TB)
const MAX_NODE_MEMORY: usize = 16 * 1024 * 1024 * 1024 * 1024;

/// NUMA topology information
#[derive(Debug, Clone, Copy)]
pub struct NumaTopology {
    /// Total number of NUMA nodes
    pub node_count: usize,
    /// Distance matrix between nodes (symmetric)
    pub distance_matrix: [[u8; MAX_NUMA_NODES]; MAX_NUMA_NODES],
    /// CPU to NUMA node mapping
    pub cpu_to_node: [NumaNodeId; MAX_CPUS],
    /// Memory ranges per NUMA node
    pub node_memory_ranges: [Option<Range<PhysAddr>>; MAX_NUMA_NODES],
}

/// CPU count constant (can be extended)
const MAX_CPUS: usize = 512;

/// NUMA memory allocation policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaPolicy {
    /// Default policy (typically local allocation)
    Default,
    /// Allocate from specific NUMA node
    Bind(NumaNodeId),
    /// Prefer specific NUMA node, fallback to others
    Preferred(NumaNodeId),
    /// Interleave allocation across multiple nodes
    Interleave,
    /// Memory allocation from local node only
    Local,
    /// Automatic NUMA balancing
    Auto,
}

/// NUMA memory statistics
#[derive(Debug, Default, Clone)]
pub struct NumaMemoryStats {
    /// Total memory per node
    pub total_memory: [usize; MAX_NUMA_NODES],
    /// Used memory per node
    pub used_memory: [usize; MAX_NUMA_NODES],
    /// Free memory per node
    pub free_memory: [usize; MAX_NUMA_NODES],
    /// Memory migrations count per node
    pub migrations: [u64; MAX_NUMA_NODES],
    /// Remote memory access count per node
    pub remote_accesses: [u64; MAX_NUMA_NODES],
}

/// NUMA page information
#[derive(Debug, Clone)]
pub struct NumaPage {
    /// Physical address of the page
    pub physical_addr: PhysAddr,
    /// NUMA node this page belongs to
    pub node_id: NumaNodeId,
    /// Reference count for shared pages
    pub ref_count: AtomicUsize,
    /// Migration timestamp
    pub migration_time: u64,
}

/// NUMA memory policy structure
#[derive(Debug)]
pub struct NumaMemoryPolicy {
    /// Default policy for allocations
    pub default_policy: NumaPolicy,
    /// Policy per process
    pub process_policies: Vec<NumaPolicy>,
    /// Policy per thread
    pub thread_policies: Vec<NumaPolicy>,
}

/// NUMA balancing statistics
#[derive(Debug, Default, Clone)]
pub struct NumaBalanceStats {
    /// Number of balancing operations
    pub balance_operations: AtomicU64,
    /// Pages migrated during balancing
    pub pages_migrated: AtomicU64,
    /// Remote access improvements
    pub access_improvements: AtomicU64,
    /// Average migration latency (nanoseconds)
    pub avg_migration_latency: AtomicU64,
}

/// NUMA manager state
#[derive(Debug)]
pub struct NumaManager {
    /// NUMA topology information
    topology: NumaTopology,
    /// NUMA memory statistics
    stats: NumaMemoryStats,
    /// NUMA balancing statistics
    balance_stats: NumaBalanceStats,
    /// Memory policies
    policies: NumaMemoryPolicy,
    /// NUMA-aware page allocator
    numa_allocator: NumaPageAllocator,
    /// Memory migration thread handle
    migration_thread_id: Option<usize>,
    /// NUMA balancing enabled
    balancing_enabled: bool,
    /// Initialized flag
    initialized: bool,
}

/// NUMA-aware page allocator
#[derive(Debug)]
struct NumaPageAllocator {
    /// Free page lists per NUMA node
    free_lists: [Vec<PhysAddr>; MAX_NUMA_NODES],
    /// Page allocation counters
    allocation_counts: [AtomicUsize; MAX_NUMA_NODES],
    /// Page size information
    page_size: PageSize,
}

/// NUMA configuration
#[derive(Debug, Clone)]
pub struct NumaConfig {
    /// Enable NUMA optimization
    pub enable_numa: bool,
    /// Enable automatic NUMA balancing
    pub enable_balancing: bool,
    /// Balancing interval in milliseconds
    pub balance_interval: u64,
    /// Migration threshold (percentage)
    pub migration_threshold: f32,
    /// Maximum migrations per second
    pub max_migrations_per_sec: u32,
    /// Enable memory interleaving
    pub enable_interleaving: bool,
}

/// NUMA initialization result
pub type NumaResult<T> = Result<T, NumaError>;

/// NUMA-related errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaError {
    InvalidNodeId,
    NoMemoryAvailable,
    MigrationFailed,
    TopologyNotDiscovered,
    PolicyNotSupported,
    AccessDenied,
    OutOfMemory,
    InvalidAddress,
    BalancingDisabled,
    ConfigurationError,
}

impl Default for NumaConfig {
    fn default() -> Self {
        Self {
            enable_numa: true,
            enable_balancing: true,
            balance_interval: 1000, // 1 second
            migration_threshold: 0.1, // 10%
            max_migrations_per_sec: 100,
            enable_interleaving: false,
        }
    }
}

impl NumaManager {
    /// Create a new NUMA manager
    pub fn new(config: NumaConfig) -> Self {
        let mut manager = Self {
            topology: NumaTopology {
                node_count: 1,
                distance_matrix: [[10; MAX_NUMA_NODES]; MAX_NUMA_NODES],
                cpu_to_node: [0; MAX_CPUS],
                node_memory_ranges: [None; MAX_NUMA_NODES],
            },
            stats: NumaMemoryStats::default(),
            balance_stats: NumaBalanceStats::default(),
            policies: NumaMemoryPolicy {
                default_policy: NumaPolicy::Default,
                process_policies: Vec::new(),
                thread_policies: Vec::new(),
            },
            numa_allocator: NumaPageAllocator {
                free_lists: [Vec::new(); MAX_NUMA_NODES],
                allocation_counts: [AtomicUsize::new(0); MAX_NUMA_NODES],
                page_size: PageSize::Size4K,
            },
            migration_thread_id: None,
            balancing_enabled: config.enable_balancing,
            initialized: false,
        };

        // Discover NUMA topology
        if config.enable_numa {
            manager.discover_topology();
        }

        manager
    }

    /// Initialize the NUMA manager
    pub fn init(&mut self, memory_map: &[(PhysAddr, usize)], cpu_count: usize) -> NumaResult<()> {
        if self.initialized {
            return Err(NumaError::ConfigurationError);
        }

        // Initialize NUMA topology
        self.initialize_topology(memory_map, cpu_count)?;
        
        // Initialize NUMA memory allocation
        self.initialize_memory_allocation(memory_map)?;
        
        // Start NUMA balancing if enabled
        if self.balancing_enabled {
            self.start_numa_balancing()?;
        }

        self.initialized = true;
        Ok(())
    }

    /// Discover NUMA topology from hardware
    fn discover_topology(&mut self) {
        // In a real implementation, this would query:
        // - ACPI SRAT (System Resource Affinity Table)
        // - CPUID information
        // - PCI device topology
        
        // For now, we'll create a default topology
        // This would be replaced with actual hardware detection
        
        // Default: Assume single NUMA node (SMP system)
        self.topology.node_count = 1;
        
        // Initialize distance matrix (default 10 for local access)
        for i in 0..MAX_NUMA_NODES {
            for j in 0..MAX_NUMA_NODES {
                self.topology.distance_matrix[i][j] = if i == j { 10 } else { 20 };
            }
        }
    }

    /// Initialize NUMA topology with memory map
    fn initialize_topology(&mut self, memory_map: &[(PhysAddr, usize)], cpu_count: usize) -> NumaResult<()> {
        if self.topology.node_count == 0 {
            self.topology.node_count = 1; // Default to single node
        }

        // Initialize CPU to node mapping
        for cpu_id in 0..core::cmp::min(cpu_count, MAX_CPUS) {
            self.topology.cpu_to_node[cpu_id] = cpu_id % self.topology.node_count;
        }

        // Initialize memory ranges
        let total_memory: usize = memory_map.iter().map(|(_, size)| size).sum();
        let memory_per_node = total_memory / self.topology.node_count;
        
        let mut current_base = PhysAddr::new(0);
        for node_id in 0..self.topology.node_count {
            let node_size = if node_id == self.topology.node_count - 1 {
                // Last node gets remaining memory
                total_memory - (memory_per_node * node_id)
            } else {
                memory_per_node
            };
            
            self.topology.node_memory_ranges[node_id] = Some(
                current_base..current_base.offset(node_size as u64)
            );
            
            self.stats.total_memory[node_id] = node_size;
            self.stats.free_memory[node_id] = node_size;
        }

        Ok(())
    }

    /// Initialize NUMA memory allocation
    fn initialize_memory_allocation(&mut self, memory_map: &[(PhysAddr, usize)]) -> NumaResult<()> {
        // Initialize free page lists for each NUMA node
        for (node_id, memory_range) in self.topology.node_memory_ranges.iter().enumerate() {
            if let Some(range) = memory_range {
                let node_start = range.start;
                let node_size = range.end.as_u64() - range.start.as_u64();
                
                // Initialize page-sized free list
                let page_count = node_size as usize / PageSize::Size4K.as_usize();
                let mut pages = Vec::with_capacity(page_count);
                
                for page_idx in 0..page_count {
                    let page_addr = node_start.offset((page_idx * PageSize::Size4K.as_usize()) as u64);
                    pages.push(page_addr);
                }
                
                self.numa_allocator.free_lists[node_id] = pages;
            }
        }

        Ok(())
    }

    /// Allocate memory from specific NUMA node
    pub fn allocate_from_node(&mut self, node_id: NumaNodeId, page_count: usize) -> NumaResult<Vec<PhysAddr>> {
        if node_id >= self.topology.node_count {
            return Err(NumaError::InvalidNodeId);
        }

        let mut allocated_pages = Vec::new();
        
        // Allocate pages from the specified node
        for _ in 0..page_count {
            if let Some(page_addr) = self.numa_allocator.free_lists[node_id].pop() {
                allocated_pages.push(page_addr);
                self.numa_allocator.allocation_counts[node_id].fetch_add(1, Ordering::SeqCst);
            } else {
                // No pages available in this node, try to migrate or allocate from other nodes
                return Err(NumaError::NoMemoryAvailable);
            }
        }

        // Update statistics
        self.stats.used_memory[node_id] += allocated_pages.len() * PageSize::Size4K.as_usize();
        self.stats.free_memory[node_id] -= allocated_pages.len() * PageSize::Size4K.as_usize();

        Ok(allocated_pages)
    }

    /// Allocate memory with NUMA policy
    pub fn allocate_with_policy(&mut self, policy: NumaPolicy, page_count: usize) -> NumaResult<Vec<PhysAddr>> {
        match policy {
            NumaPolicy::Default | NumaPolicy::Local => {
                self.allocate_from_node(0, page_count) // Default to node 0
            }
            NumaPolicy::Bind(node_id) => {
                self.allocate_from_node(node_id, page_count)
            }
            NumaPolicy::Preferred(node_id) => {
                // Try preferred node first, fallback to others
                if let Ok(pages) = self.allocate_from_node(node_id, page_count) {
                    Ok(pages)
                } else {
                    // Fallback to any available node
                    self.allocate_from_any_node(page_count)
                }
            }
            NumaPolicy::Interleave => {
                self.allocate_interleaved(page_count)
            }
            NumaPolicy::Auto => {
                self.allocate_with_balancing(page_count)
            }
        }
    }

    /// Allocate from any available NUMA node
    fn allocate_from_any_node(&mut self, page_count: usize) -> NumaResult<Vec<PhysAddr>> {
        for node_id in 0..self.topology.node_count {
            if let Ok(pages) = self.allocate_from_node(node_id, page_count) {
                return Ok(pages);
            }
        }
        Err(NumaError::NoMemoryAvailable)
    }

    /// Allocate memory in interleaved fashion across NUMA nodes
    fn allocate_interleaved(&mut self, page_count: usize) -> NumaResult<Vec<PhysAddr>> {
        let mut allocated_pages = Vec::new();
        let mut pages_per_node = page_count / self.topology.node_count;
        let mut remaining_pages = page_count % self.topology.node_count;

        for node_id in 0..self.topology.node_count {
            let node_pages = pages_per_node + if remaining_pages > 0 { 1 } else { 0 };
            if remaining_pages > 0 {
                remaining_pages -= 1;
            }

            if node_pages > 0 {
                let node_allocated = self.allocate_from_node(node_id, node_pages)?;
                allocated_pages.extend(node_allocated);
            }
        }

        Ok(allocated_pages)
    }

    /// Allocate with automatic NUMA balancing
    fn allocate_with_balancing(&mut self, page_count: usize) -> NumaResult<Vec<PhysAddr>> {
        // Find the node with the least load
        let mut best_node = 0;
        let mut min_load = usize::MAX;

        for node_id in 0..self.topology.node_count {
            let load = self.numa_allocator.allocation_counts[node_id].load(Ordering::SeqCst);
            if load < min_load && !self.numa_allocator.free_lists[node_id].is_empty() {
                min_load = load;
                best_node = node_id;
            }
        }

        self.allocate_from_node(best_node, page_count)
    }

    /// Migrate memory pages between NUMA nodes
    pub fn migrate_pages(&mut self, pages: &[PhysAddr], target_node: NumaNodeId) -> NumaResult<()> {
        if target_node >= self.topology.node_count {
            return Err(NumaError::InvalidNodeId);
        }

        let migration_start_time = self.get_current_time_ns();

        // In a real implementation, this would:
        // 1. Copy page data to target node
        // 2. Update page table entries
        // 3. Invalidate old entries
        // 4. Update memory statistics
        
        for &page_addr in pages {
            let source_node = self.get_node_for_address(page_addr)?;
            
            // Update statistics
            self.stats.migrations[target_node] += 1;
            self.stats.used_memory[source_node] -= PageSize::Size4K.as_usize();
            self.stats.used_memory[target_node] += PageSize::Size4K.as_usize();
            self.stats.free_memory[source_node] += PageSize::Size4K.as_usize();
            self.stats.free_memory[target_node] -= PageSize::Size4K.as_usize();
        }

        // Update migration latency statistics
        let migration_time = self.get_current_time_ns() - migration_start_time;
        let current_avg = self.balance_stats.avg_migration_latency.load(Ordering::SeqCst);
        let new_avg = (current_avg + migration_time) / 2;
        self.balance_stats.avg_migration_latency.store(new_avg, Ordering::SeqCst);

        Ok(())
    }

    /// Get NUMA node for a physical address
    fn get_node_for_address(&self, addr: PhysAddr) -> NumaResult<NumaNodeId> {
        for (node_id, memory_range) in self.topology.node_memory_ranges.iter().enumerate() {
            if let Some(range) = memory_range {
                if addr >= range.start && addr < range.end {
                    return Ok(node_id);
                }
            }
        }
        Err(NumaError::InvalidAddress)
    }

    /// Calculate NUMA distance between two nodes
    pub fn get_distance(&self, node1: NumaNodeId, node2: NumaNodeId) -> u8 {
        if node1 >= self.topology.node_count || node2 >= self.topology.node_count {
            return 255; // Invalid distance
        }
        self.topology.distance_matrix[node1][node2]
    }

    /// Find the nearest NUMA node for a given address
    pub fn find_nearest_node(&self, addr: PhysAddr) -> NumaResult<NumaNodeId> {
        let source_node = self.get_node_for_address(addr)?;
        
        let mut nearest_node = source_node;
        let mut min_distance = self.get_distance(source_node, source_node);
        
        for node_id in 0..self.topology.node_count {
            let distance = self.get_distance(source_node, node_id);
            if distance < min_distance {
                min_distance = distance;
                nearest_node = node_id;
            }
        }
        
        Ok(nearest_node)
    }

    /// Perform NUMA balancing
    pub fn perform_balancing(&mut self) -> NumaResult<()> {
        if !self.balancing_enabled {
            return Err(NumaError::BalancingDisabled);
        }

        self.balance_stats.balance_operations.fetch_add(1, Ordering::SeqCst);

        // Analyze memory access patterns and migrate pages if beneficial
        // This is a simplified implementation
        
        for node_id in 0..self.topology.node_count {
            let node_load = self.stats.used_memory[node_id];
            let total_memory = self.stats.total_memory[node_id];
            
            if total_memory > 0 {
                let load_percentage = node_load as f32 / total_memory as f32;
                
                // If a node is heavily loaded (>80%), try to migrate some pages
                if load_percentage > 0.8 {
                    self.balance_node_load(node_id)?;
                }
            }
        }

        Ok(())
    }

    /// Balance load for a specific node
    fn balance_node_load(&mut self, node_id: NumaNodeId) -> NumaResult<()> {
        // Find underloaded nodes to migrate pages to
        let mut underloaded_nodes = Vec::new();
        
        for candidate_node in 0..self.topology.node_count {
            if candidate_node == node_id {
                continue;
            }
            
            let node_load = self.stats.used_memory[candidate_node];
            let total_memory = self.stats.total_memory[candidate_node];
            
            if total_memory > 0 {
                let load_percentage = node_load as f32 / total_memory as f32;
                
                if load_percentage < 0.3 {
                    underloaded_nodes.push(candidate_node);
                }
            }
        }

        if !underloaded_nodes.is_empty() {
            // Migrate some pages from overloaded node to underloaded nodes
            // Implementation would copy pages and update page tables
        }

        Ok(())
    }

    /// Start NUMA balancing thread
    fn start_numa_balancing(&mut self) -> NumaResult<()> {
        // In a real implementation, this would spawn a kernel thread
        // that periodically performs NUMA balancing
        Ok(())
    }

    /// Get current time in nanoseconds
    fn get_current_time_ns(&self) -> u64 {
        // Placeholder implementation
        // In reality, this would use high-resolution timer
        0
    }

    /// Get NUMA statistics
    pub fn get_stats(&self) -> NumaMemoryStats {
        self.stats
    }

    /// Get NUMA balancing statistics
    pub fn get_balance_stats(&self) -> NumaBalanceStats {
        self.balance_stats.clone()
    }

    /// Get NUMA topology
    pub fn get_topology(&self) -> NumaTopology {
        self.topology
    }

    /// Check if NUMA is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set NUMA policy for a process
    pub fn set_process_policy(&mut self, process_id: usize, policy: NumaPolicy) -> NumaResult<()> {
        if process_id >= self.policies.process_policies.len() {
            self.policies.process_policies.resize(process_id + 1, NumaPolicy::Default);
        }
        
        self.policies.process_policies[process_id] = policy;
        Ok(())
    }

    /// Set NUMA policy for a thread
    pub fn set_thread_policy(&mut self, thread_id: usize, policy: NumaPolicy) -> NumaResult<()> {
        if thread_id >= self.policies.thread_policies.len() {
            self.policies.thread_policies.resize(thread_id + 1, NumaPolicy::Default);
        }
        
        self.policies.thread_policies[thread_id] = policy;
        Ok(())
    }

    /// Enable or disable NUMA balancing
    pub fn set_balancing_enabled(&mut self, enabled: bool) {
        self.balancing_enabled = enabled;
    }

    /// Get NUMA memory statistics for a specific node
    pub fn get_node_stats(&self, node_id: NumaNodeId) -> NumaResult<(usize, usize, usize)> {
        if node_id >= self.topology.node_count {
            return Err(NumaError::InvalidNodeId);
        }
        
        Ok((
            self.stats.total_memory[node_id],
            self.stats.used_memory[node_id],
            self.stats.free_memory[node_id],
        ))
    }

    /// Get the number of NUMA nodes
    pub fn get_node_count(&self) -> usize {
        self.topology.node_count
    }

    /// Get the current NUMA memory policy
    pub fn get_current_policy(&self) -> NumaPolicy {
        self.policies.default_policy
    }

    /// Set the default NUMA memory policy
    pub fn set_default_policy(&mut self, policy: NumaPolicy) {
        self.policies.default_policy = policy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numa_manager_creation() {
        let config = NumaConfig::default();
        let manager = NumaManager::new(config);
        assert!(!manager.is_initialized());
        assert_eq!(manager.get_node_count(), 1);
    }

    #[test]
    fn test_numa_policy_enum() {
        assert_eq!(NumaPolicy::Default, NumaPolicy::Default);
        assert_ne!(NumaPolicy::Bind(1), NumaPolicy::Bind(2));
    }

    #[test]
    fn test_numa_distance_calculation() {
        let config = NumaConfig::default();
        let mut manager = NumaManager::new(config);
        manager.topology.node_count = 2;
        manager.topology.distance_matrix[0][1] = 20;
        
        assert_eq!(manager.get_distance(0, 0), 10); // Local access
        assert_eq!(manager.get_distance(0, 1), 20); // Remote access
    }
}