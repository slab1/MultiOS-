//! Memory Profiling Framework for MultiOS Kernel
//! 
//! This module provides comprehensive memory profiling and optimization tools for both
//! kernel and user applications. It includes real-time tracking, leak detection,
//! cache monitoring, and NUMA-aware allocation strategies.

#![no_std]

pub mod allocator_hook;
pub mod cache_profiler;
pub mod fragmentation_analyzer;
pub mod leak_detector;
pub mod memory_mapper;
pub mod numa_profiler;
pub mod stack_profiler;
pub mod realtime_tracker;

pub use allocator_hook::AllocatorHook;
pub use cache_profiler::CacheProfiler;
pub use fragmentation_analyzer::FragmentationAnalyzer;
pub use leak_detector::LeakDetector;
pub use memory_mapper::MemoryMapper;
pub use numa_profiler::NUMAProfiler;
pub use realtime_tracker::RealtimeTracker;
pub use stack_profiler::StackProfiler;

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use spin::Mutex;

/// Memory profiling statistics structure
#[derive(Debug, Clone)]
pub struct MemoryProfilingStats {
    pub total_allocations: AtomicU64,
    pub total_deallocations: AtomicU64,
    pub current_allocated: AtomicU64,
    pub peak_allocated: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub heap_fragmentation: AtomicU64,
    pub stack_usage: AtomicU64,
    pub numa_node_usage: Vec<AtomicU64>,
}

/// Global memory profiling system
pub static MEMORY_PROFILER: Mutex<MemoryProfilingStats> = Mutex::new(MemoryProfilingStats {
    total_allocations: AtomicU64::new(0),
    total_deallocations: AtomicU64::new(0),
    current_allocated: AtomicU64::new(0),
    peak_allocated: AtomicU64::new(0),
    cache_hits: AtomicU64::new(0),
    cache_misses: AtomicU64::new(0),
    heap_fragmentation: AtomicU64::new(0),
    stack_usage: AtomicU64::new(0),
    numa_node_usage: Vec::new(),
});

/// Initialize memory profiling system
pub fn init() {
    let profiler = MEMORY_PROFILER.lock();
    
    // Initialize NUMA node usage counters
    let numa_node_count = numa_profiler::get_num_nodes();
    drop(profiler);
    
    let mut profiler = MEMORY_PROFILER.lock();
    profiler.numa_node_usage = (0..numa_node_count)
        .map(|_| AtomicU64::new(0))
        .collect();
    drop(profiler);
    
    // Initialize all profiling components
    AllocatorHook::init();
    CacheProfiler::init();
    FragmentationAnalyzer::init();
    LeakDetector::init();
    RealtimeTracker::init();
    StackProfiler::init();
    NUMAProfiler::init();
    
    log::info!("Memory profiling system initialized");
}

/// Get current memory statistics
pub fn get_stats() -> MemoryProfilingStats {
    MEMORY_PROFILER.lock().clone()
}

/// Reset all statistics
pub fn reset_stats() {
    let profiler = MEMORY_PROFILER.lock();
    
    profiler.total_allocations.store(0, Ordering::SeqCst);
    profiler.total_deallocations.store(0, Ordering::SeqCst);
    profiler.current_allocated.store(0, Ordering::SeqCst);
    profiler.peak_allocated.store(0, Ordering::SeqCst);
    profiler.cache_hits.store(0, Ordering::SeqCst);
    profiler.cache_misses.store(0, Ordering::SeqCst);
    profiler.heap_fragmentation.store(0, Ordering::SeqCst);
    profiler.stack_usage.store(0, Ordering::SeqCst);
    
    for node_counter in &profiler.numa_node_usage {
        node_counter.store(0, Ordering::SeqCst);
    }
}