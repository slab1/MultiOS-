//! NUMA Hardware Abstraction Layer
//!
//! This module provides unified NUMA (Non-Uniform Memory Access) interfaces
//! across architectures for memory topology management and NUMA-aware operations.

use crate::log::{info, warn, error};
use crate::{KernelError, Result};
use spin::RwLock;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// NUMA subsystem initialization
pub fn init() -> Result<()> {
    info!("Initializing NUMA HAL...");
    
    // Detect NUMA topology
    detect_numa_topology()?;
    
    // Initialize NUMA memory management
    init_numa_memory_management()?;
    
    // Set up NUMA-aware scheduling
    init_numa_scheduling()?;
    
    // Configure NUMA policies
    configure_numa_policies()?;
    
    Ok(())
}

/// NUMA subsystem shutdown
pub fn shutdown() -> Result<()> {
    info!("Shutting down NUMA HAL...");
    Ok(())
}

/// NUMA node information
#[derive(Debug, Clone, Copy)]
pub struct NumaNode {
    pub node_id: usize,
    pub memory_start: usize,
    pub memory_end: usize,
    pub memory_size: u64,
    pub cpu_list: Vec<usize>,
    pub distance: u32, // Distance to other nodes
}

/// NUMA topology information
#[derive(Debug, Clone)]
pub struct NumaTopology {
    pub node_count: usize,
    pub nodes: Vec<NumaNode>,
    pub total_memory: u64,
    pub supports_numa: bool,
    pub numa_enabled: bool,
}

/// NUMA memory policy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NumaPolicy {
    Default = 0,        // System default policy
    Preferred = 1,      // Preferred node
    Bind = 2,           // Bind to specific node
    Interleave = 3,     // Interleave across nodes
    Local = 4,          // Allocate on local node
    Current = 5,        // Allocate on current node
}

/// NUMA allocation request
#[derive(Debug, Clone)]
pub struct NumaAllocation {
    pub size: usize,
    pub alignment: usize,
    pub policy: NumaPolicy,
    pub preferred_node: Option<usize>,
    pub flags: u32,
}

/// NUMA statistics
#[derive(Debug, Clone, Copy)]
pub struct NumaStats {
    pub node_allocations: Vec<AtomicU64>,      // Bytes allocated per node
    pub node_memory_free: Vec<AtomicU64>,      // Free memory per node
    pub node_memory_total: Vec<u64>,          // Total memory per node
    pub numa_migrations: AtomicU64,           // Memory migrations
    pub numa_hits: AtomicU64,                // Local allocations
    pub numa_misses: AtomicU64,              // Remote allocations
    pub numa_foreign: AtomicU64,             // Foreign node allocations
}

/// NUMA node distance matrix
static NODE_DISTANCE: RwLock<Vec<Vec<u32>>> = RwLock::new(Vec::new());

/// Current NUMA topology
static NUMA_TOPOLOGY: RwLock<Option<NumaTopology>> = RwLock::new(None);

/// NUMA statistics
static NUMA_STATS: Mutex<NumaStats> = Mutex::new(NumaStats {
    node_allocations: Vec::new(),
    node_memory_free: Vec::new(),
    node_memory_total: Vec::new(),
    numa_migrations: AtomicU64::new(0),
    numa_hits: AtomicU64::new(0),
    numa_misses: AtomicU64::new(0),
    numa_foreign: AtomicU64::new(0),
});

/// NUMA policies per process/thread
static NUMA_POLICIES: RwLock<Vec<NumaPolicy>> = RwLock::new(Vec::new());

/// Detect NUMA topology
fn detect_numa_topology() -> Result<()> {
    info!("Detecting NUMA topology...");
    
    let topology = detect_numa_topology_arch()?;
    
    if topology.supports_numa && topology.numa_enabled {
        info!("NUMA topology detected: {} nodes", topology.node_count);
        for (i, node) in topology.nodes.iter().enumerate() {
            info!("Node {}: {}MB, {} CPUs", 
                  i, node.memory_size / 1024 / 1024, node.cpu_list.len());
        }
    } else {
        info!("NUMA not available, running in UMA mode");
    }
    
    *NUMA_TOPOLOGY.write() = Some(topology);
    init_numa_stats(&topology);
    
    Ok(())
}

/// Architecture-specific NUMA detection
#[cfg(target_arch = "x86_64")]
fn detect_numa_topology_arch() -> Result<NumaTopology> {
    // x86_64 NUMA detection using ACPI SRAT and SLIT tables
    // For now, assume single-node system or create a simple topology
    
    let multicore_info = crate::hal::multicore::get_cpu_topology()
        .unwrap_or(crate::hal::multicore::CpuTopology {
            socket_count: 1,
            cores_per_socket: 4,
            threads_per_core: 2,
            total_cores: 4,
            total_threads: 8,
        });
    
    let node_count = if multicore_info.cores_per_socket >= 8 {
        // Multi-socket or large system - assume 2 nodes
        2
    } else {
        // Single node for smaller systems
        1
    };
    
    let mut nodes = Vec::new();
    let cores_per_node = multicore_info.total_cores / node_count as u8;
    
    for node_id in 0..node_count {
        let start_core = node_id as usize * cores_per_node as usize;
        let end_core = start_core + cores_per_node as usize;
        let cpu_list = (start_core..end_core).collect();
        
        let memory_size = if node_count == 1 {
            8 * 1024 * 1024 * 1024 // 8GB single node
        } else {
            4 * 1024 * 1024 * 1024 // 4GB per node (2 nodes)
        };
        
        nodes.push(NumaNode {
            node_id,
            memory_start: 0, // Would be filled from firmware tables
            memory_end: memory_size,
            memory_size,
            cpu_list,
            distance: 10, // Local distance
        });
    }
    
    // Set up distance matrix
    let mut distance_matrix = vec![vec![10u32; node_count]; node_count];
    for i in 0..node_count {
        for j in 0..node_count {
            if i == j {
                distance_matrix[i][j] = 10; // Local access
            } else {
                distance_matrix[i][j] = 21; // Remote access
            }
        }
    }
    *NODE_DISTANCE.write() = distance_matrix;
    
    Ok(NumaTopology {
        node_count,
        nodes,
        total_memory: (memory_size as usize) * node_count,
        supports_numa: node_count > 1,
        numa_enabled: node_count > 1,
    })
}

#[cfg(target_arch = "aarch64")]
fn detect_numa_topology_arch() -> Result<NumaTopology> {
    // ARM64 NUMA detection using ACPI or device tree
    // For now, assume single-node system
    
    Ok(NumaTopology {
        node_count: 1,
        nodes: vec![NumaNode {
            node_id: 0,
            memory_start: 0x80000000,
            memory_end: 0x80000000 + 8 * 1024 * 1024 * 1024, // 8GB
            memory_size: 8 * 1024 * 1024 * 1024,
            cpu_list: vec![0, 1, 2, 3],
            distance: 10,
        }],
        total_memory: 8 * 1024 * 1024 * 1024,
        supports_numa: false, // Most ARM64 systems are UMA
        numa_enabled: false,
    })
}

#[cfg(target_arch = "riscv64")]
fn detect_numa_topology_arch() -> Result<NumaTopology> {
    // RISC-V NUMA detection - most systems are UMA
    // Some advanced RISC-V systems may have NUMA
    
    Ok(NumaTopology {
        node_count: 1,
        nodes: vec![NumaNode {
            node_id: 0,
            memory_start: 0x80000000,
            memory_end: 0x80000000 + 8 * 1024 * 1024 * 1024, // 8GB
            memory_size: 8 * 1024 * 1024 * 1024,
            cpu_list: vec![0, 1, 2, 3],
            distance: 10,
        }],
        total_memory: 8 * 1024 * 1024 * 1024,
        supports_numa: false, // Most RISC-V systems are UMA
        numa_enabled: false,
    })
}

/// Initialize NUMA statistics
fn init_numa_stats(topology: &NumaTopology) {
    let mut stats = NUMA_STATS.lock();
    
    stats.node_allocations = vec![AtomicU64::new(0); topology.node_count];
    stats.node_memory_free = Vec::new();
    
    for node in &topology.nodes {
        stats.node_memory_free.push(AtomicU64::new(node.memory_size));
        stats.node_memory_total.push(node.memory_size);
    }
}

/// Initialize NUMA memory management
fn init_numa_memory_management() -> Result<()> {
    info!("Initializing NUMA memory management...");
    
    let topology = get_numa_topology();
    if !topology.supports_numa || !topology.numa_enabled {
        info!("NUMA memory management disabled (UMA mode)");
        return Ok(());
    }
    
    // Initialize NUMA node memory allocators
    init_numa_node_allocators()?;
    
    // Set up NUMA memory policies
    init_numa_policies()?;
    
    Ok(())
}

/// Initialize NUMA node allocators
fn init_numa_node_allocators() -> Result<()> {
    info!("Initializing NUMA node allocators...");
    
    let topology = get_numa_topology();
    
    for node in &topology.nodes {
        info!("Initializing allocator for node {} ({}MB)", 
              node.node_id, node.memory_size / 1024 / 1024);
        
        // Initialize per-node memory allocator
        // This would set up node-specific memory pools
    }
    
    Ok(())
}

/// Initialize NUMA scheduling
fn init_numa_scheduling() -> Result<()> {
    info!("Initializing NUMA-aware scheduling...");
    
    let topology = get_numa_topology();
    
    if topology.supports_numa {
        // Set up NUMA-aware scheduling
        info!("NUMA-aware scheduling initialized");
        
        // Enable NUMA balancing
        enable_numa_balancing()?;
    }
    
    Ok(())
}

/// Enable NUMA balancing
fn enable_numa_balancing() -> Result<()> {
    info!("Enabling NUMA balancing...");
    
    // NUMA balancing helps move memory to the node where processes are running
    // This reduces remote memory access penalties
    
    Ok(())
}

/// Configure NUMA policies
fn configure_numa_policies() -> Result<()> {
    info!("Configuring NUMA policies...");
    
    let topology = get_numa_topology();
    
    if topology.supports_numa {
        // Set up default NUMA policies
        *NUMA_POLICIES.write() = vec![NumaPolicy::Default; topology.node_count];
        
        info!("NUMA policies configured");
    }
    
    Ok(())
}

/// Get NUMA topology
pub fn get_numa_topology() -> NumaTopology {
    if let Some(topology) = NUMA_TOPOLOGY.read().as_ref() {
        *topology
    } else {
        // Fallback to single-node topology
        NumaTopology {
            node_count: 1,
            nodes: vec![NumaNode {
                node_id: 0,
                memory_start: 0,
                memory_end: 8 * 1024 * 1024 * 1024,
                memory_size: 8 * 1024 * 1024 * 1024,
                cpu_list: vec![0],
                distance: 10,
            }],
            total_memory: 8 * 1024 * 1024 * 1024,
            supports_numa: false,
            numa_enabled: false,
        }
    }
}

/// Get NUMA node for CPU
pub fn get_node_for_cpu(cpu_id: usize) -> Option<usize> {
    let topology = get_numa_topology();
    
    for node in &topology.nodes {
        if node.cpu_list.contains(&cpu_id) {
            return Some(node.node_id);
        }
    }
    
    None
}

/// Get NUMA node for address
pub fn get_node_for_address(address: usize) -> Option<usize> {
    let topology = get_numa_topology();
    
    for node in &topology.nodes {
        if address >= node.memory_start && address < node.memory_end {
            return Some(node.node_id);
        }
    }
    
    None
}

/// Get NUMA distance between nodes
pub fn get_numa_distance(node1: usize, node2: usize) -> u32 {
    let distances = NODE_DISTANCE.read();
    
    if node1 < distances.len() && node2 < distances[node1].len() {
        distances[node1][node2]
    } else {
        100 // High penalty for invalid nodes
    }
}

/// Get current NUMA node
pub fn get_current_numa_node() -> usize {
    let cpu_id = crate::hal::multicore::get_current_cpu_id();
    get_node_for_cpu(cpu_id).unwrap_or(0)
}

/// Allocate memory with NUMA policy
pub fn numa_allocate(request: &NumaAllocation) -> Result<usize> {
    let topology = get_numa_topology();
    
    if !topology.supports_numa || !topology.numa_enabled {
        // Fall back to regular allocation
        return allocate_regular_memory(request.size, request.alignment);
    }
    
    let node_id = determine_allocation_node(request)?;
    
    info!("Allocating {} bytes on node {} with policy {:?}", 
          request.size, node_id, request.policy);
    
    // Perform actual allocation on the specified node
    let address = allocate_on_node(node_id, request.size, request.alignment)?;
    
    // Update statistics
    update_allocation_stats(node_id, request.size);
    
    Ok(address)
}

/// Determine NUMA node for allocation
fn determine_allocation_node(request: &NumaAllocation) -> Result<usize> {
    let current_node = get_current_numa_node();
    
    match request.policy {
        NumaPolicy::Default => Ok(current_node),
        NumaPolicy::Preferred => {
            if let Some(node) = request.preferred_node {
                Ok(node)
            } else {
                Ok(current_node)
            }
        }
        NumaPolicy::Bind => {
            if let Some(node) = request.preferred_node {
                if node < get_numa_topology().node_count {
                    Ok(node)
                } else {
                    Err(KernelError::InvalidParameter)
                }
            } else {
                Err(KernelError::InvalidParameter)
            }
        }
        NumaPolicy::Interleave => {
            // Round-robin allocation across nodes
            Ok(interleave_allocation_node())
        }
        NumaPolicy::Local => Ok(current_node),
        NumaPolicy::Current => Ok(current_node),
    }
}

/// Get node for interleave allocation
fn interleave_allocation_node() -> usize {
    static INTERLEAVE_COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    let node_count = get_numa_topology().node_count;
    if node_count == 0 {
        return 0;
    }
    
    let node = INTERLEAVE_COUNTER.fetch_add(1, Ordering::SeqCst) % node_count;
    node
}

/// Allocate memory on specific NUMA node
fn allocate_on_node(node_id: usize, size: usize, alignment: usize) -> Result<usize> {
    let topology = get_numa_topology();
    
    if node_id >= topology.node_count {
        return Err(KernelError::InvalidParameter);
    }
    
    // Check if node has enough memory
    let node = &topology.nodes[node_id];
    if size > node.memory_size as usize {
        return Err(KernelError::OutOfMemory);
    }
    
    // Perform allocation from node's memory pool
    let address = allocate_from_node_memory(node_id, size, alignment)?;
    
    Ok(address)
}

/// Allocate memory from node-specific pool
fn allocate_from_node_memory(node_id: usize, size: usize, alignment: usize) -> Result<usize> {
    // This would implement actual memory allocation from the node's memory pool
    // For now, return a placeholder address
    
    let node_base = 0x80000000 + (node_id * 0x40000000); // 1GB per node
    let aligned_address = (node_base + size + alignment - 1) & !(alignment - 1);
    
    info!("Allocating {} bytes on node {} at address {:#x}", 
          size, node_id, aligned_address);
    
    Ok(aligned_address)
}

/// Allocate regular memory (fallback for UMA)
fn allocate_regular_memory(size: usize, alignment: usize) -> Result<usize> {
    info!("Allocating {} bytes (regular allocation)", size);
    
    // This would allocate from the general memory pool
    let address = 0x100000 + size + alignment; // Placeholder
    
    Ok(address)
}

/// Update allocation statistics
fn update_allocation_stats(node_id: usize, size: usize) {
    let mut stats = NUMA_STATS.lock();
    
    if node_id < stats.node_allocations.len() {
        stats.node_allocations[node_id].fetch_add(size as u64, Ordering::SeqCst);
        stats.node_memory_free[node_id].fetch_sub(size as u64, Ordering::SeqCst);
    }
    
    let current_node = get_current_numa_node();
    if node_id == current_node {
        stats.numa_hits.fetch_add(1, Ordering::SeqCst);
    } else {
        stats.numa_misses.fetch_add(1, Ordering::SeqCst);
    }
}

/// Free NUMA-allocated memory
pub fn numa_free(address: usize, size: usize) -> Result<()> {
    let topology = get_numa_topology();
    
    if !topology.supports_numa || !topology.numa_enabled {
        // Fall back to regular free
        return free_regular_memory(address, size);
    }
    
    let node_id = get_node_for_address(address)
        .ok_or(KernelError::InvalidAddress)?;
    
    info!("Freeing {} bytes on node {}", size, node_id);
    
    // Return memory to node's pool
    return_memory_to_node(node_id, address, size)?;
    
    // Update statistics
    update_free_stats(node_id, size);
    
    Ok(())
}

/// Return memory to node pool
fn return_memory_to_node(node_id: usize, address: usize, size: usize) -> Result<()> {
    // This would return memory to the node's free pool
    info!("Returned {} bytes to node {} pool", size, node_id);
    
    Ok(())
}

/// Update free statistics
fn update_free_stats(node_id: usize, size: usize) {
    let mut stats = NUMA_STATS.lock();
    
    if node_id < stats.node_memory_free.len() {
        stats.node_memory_free[node_id].fetch_add(size as u64, Ordering::SeqCst);
    }
    
    if node_id < stats.node_allocations.len() {
        stats.node_allocations[node_id].fetch_sub(size as u64, Ordering::SeqCst);
    }
}

/// Free regular memory
fn free_regular_memory(address: usize, size: usize) -> Result<()> {
    info!("Freeing {} bytes at {:#x} (regular free)", size, address);
    Ok(())
}

/// Set NUMA policy for current process
pub fn set_numa_policy(policy: NumaPolicy, node_id: Option<usize>) -> Result<()> {
    let topology = get_numa_topology();
    
    if policy == NumaPolicy::Bind && node_id.is_none() {
        return Err(KernelError::InvalidParameter);
    }
    
    if let Some(node) = node_id {
        if node >= topology.node_count {
            return Err(KernelError::InvalidParameter);
        }
    }
    
    // For now, set process-wide policy
    // In a real implementation, this would be per-process/thread
    
    info!("Set NUMA policy to {:?} for current process", policy);
    
    Ok(())
}

/// Get NUMA policy for current process
pub fn get_numa_policy() -> NumaPolicy {
    // For now, return default policy
    // In real implementation, would get current process policy
    NumaPolicy::Default
}

/// Get NUMA statistics
pub fn get_numa_statistics() -> NumaStats {
    NUMA_STATS.lock().clone()
}

/// Get memory usage per NUMA node
pub fn get_node_memory_usage() -> Vec<u64> {
    let stats = NUMA_STATS.lock();
    let mut usage = Vec::new();
    
    for i in 0..stats.node_allocations.len() {
        let allocated = stats.node_allocations[i].load(Ordering::SeqCst);
        let free = stats.node_memory_free[i].load(Ordering::SeqCst);
        let total = stats.node_memory_total[i];
        let used = total - free;
        usage.push(used);
    }
    
    usage
}

/// Check if NUMA balancing is beneficial
pub fn should_balance_numa() -> bool {
    let topology = get_numa_topology();
    
    if !topology.supports_numa {
        return false;
    }
    
    let stats = NUMA_STATS.lock();
    
    // If we have many remote accesses, balancing might help
    let total_accesses = stats.numa_hits.load(Ordering::SeqCst) + 
                         stats.numa_misses.load(Ordering::SeqCst);
    
    if total_accesses > 1000 {
        let remote_ratio = stats.numa_misses.load(Ordering::SeqCst) as f64 / 
                          total_accesses as f64;
        remote_ratio > 0.3 // If >30% remote accesses
    } else {
        false
    }
}

/// NUMA utility functions
pub mod utils {
    use super::*;
    
    /// Get memory locality score for address
    pub fn get_memory_locality_score(address: usize) -> f64 {
        let current_node = get_current_numa_node();
        let address_node = get_node_for_address(address).unwrap_or(0);
        
        if current_node == address_node {
            1.0 // Perfect locality
        } else {
            let distance = get_numa_distance(current_node, address_node);
            // Lower distance = better locality
            (21.0 - distance as f64) / 21.0
        }
    }
    
    /// Get optimal NUMA node for memory allocation
    pub fn get_optimal_numa_node(size: usize) -> usize {
        let topology = get_numa_topology();
        
        if !topology.supports_numa {
            return 0;
        }
        
        let current_node = get_current_numa_node();
        
        // Check if current node has enough memory
        let stats = NUMA_STATS.lock();
        if current_node < stats.node_memory_free.len() {
            let free_memory = stats.node_memory_free[current_node].load(Ordering::SeqCst);
            if free_memory >= size as u64 {
                return current_node;
            }
        }
        
        // Find node with most free memory
        let mut best_node = 0;
        let mut max_free = 0u64;
        
        for i in 0..stats.node_memory_free.len() {
            let free = stats.node_memory_free[i].load(Ordering::SeqCst);
            if free > max_free {
                max_free = free;
                best_node = i;
            }
        }
        
        best_node
    }
    
    /// Calculate NUMA memory access cost
    pub fn calculate_numa_cost(size: usize, node_id: usize) -> u64 {
        let current_node = get_current_numa_node();
        
        if current_node == node_id {
            // Local access
            size as u64
        } else {
            // Remote access - add penalty
            let distance = get_numa_distance(current_node, node_id);
            (size as u64) * distance / 10
        }
    }
}