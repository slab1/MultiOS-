//! Memory Performance Benchmarks
//! 
//! This module implements comprehensive memory performance benchmarks including:
//! - Memory read/write operations
//! - Cache performance tests
//! - Memory allocation speed
//! - Memory bandwidth tests

use super::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

const CACHE_LINE_SIZE: usize = 64;
const L1_CACHE_SIZE: usize = 32 * 1024; // 32KB typical L1 cache
const L2_CACHE_SIZE: usize = 256 * 1024; // 256KB typical L2 cache
const TEST_BUFFER_SIZE: usize = 1024 * 1024; // 1MB test buffer

/// Memory allocator for tracking allocations
struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        libc::malloc(layout.size())
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut libc::c_void);
    }
}

/// Sequential memory read benchmark
pub struct SequentialRead;

impl SequentialRead {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for SequentialRead {
    fn name(&self) -> &str {
        "Sequential Memory Read"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        // Allocate test buffer
        let mut buffer = vec![0u8; TEST_BUFFER_SIZE];
        
        // Initialize buffer with test data
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        let start = Instant::now();
        let mut checksum = 0u64;
        let mut data: &[u8] = &buffer;
        
        for _ in 0..iterations {
            // Sequential read operations
            for chunk in data.chunks(CACHE_LINE_SIZE) {
                for &byte in chunk {
                    checksum ^= byte as u64;
                }
            }
        }
        
        let elapsed = start.elapsed();
        let bytes_per_sec = (iterations as f64 * TEST_BUFFER_SIZE as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("buffer_size".to_string(), TEST_BUFFER_SIZE.to_string());
        metadata.insert("checksum".to_string(), checksum.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
            throughput: bytes_per_sec,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Sequential memory write benchmark
pub struct SequentialWrite;

impl SequentialWrite {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for SequentialWrite {
    fn name(&self) -> &str {
        "Sequential Memory Write"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; TEST_BUFFER_SIZE];
        let test_value: u8 = 0xAA;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            // Sequential write operations
            for chunk in buffer.chunks_mut(CACHE_LINE_SIZE) {
                for byte in chunk.iter_mut() {
                    *byte = test_value;
                }
            }
        }
        
        let elapsed = start.elapsed();
        let bytes_per_sec = (iterations as f64 * TEST_BUFFER_SIZE as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("buffer_size".to_string(), TEST_BUFFER_SIZE.to_string());
        metadata.insert("test_value".to_string(), format!("0x{:02X}", test_value));
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
            throughput: bytes_per_sec,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Random memory access benchmark
pub struct RandomAccess;

impl RandomAccess {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for RandomAccess {
    fn name(&self) -> &str {
        "Random Memory Access"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u64; TEST_BUFFER_SIZE / 8]; // 64-bit values
        let mut accesses = 0u64;
        
        // Initialize buffer
        for (i, value) in buffer.iter_mut().enumerate() {
            *value = i as u64;
        }
        
        let start = Instant::now();
        
        for i in 0..iterations {
            // Random access pattern using linear congruential generator
            let mut seed = i.wrapping_mul(1664525).wrapping_add(1013904223);
            let index = (seed % buffer.len() as u64) as usize;
            accesses ^= buffer[index];
            buffer[index] ^= accesses;
        }
        
        let elapsed = start.elapsed();
        let bytes_per_sec = (iterations as f64 * 8.0) / elapsed.as_secs_f64(); // 8 bytes per 64-bit access
        
        let mut metadata = HashMap::new();
        metadata.insert("buffer_elements".to_string(), buffer.len().to_string());
        metadata.insert("access_size".to_string(), "8".to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
            throughput: bytes_per_sec,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Memory allocation benchmark
pub struct MemoryAllocation;

impl MemoryAllocation {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for MemoryAllocation {
    fn name(&self) -> &str {
        "Memory Allocation"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let alloc_size = 1024; // 1KB allocations
        
        let start = Instant::now();
        let mut allocations = Vec::new();
        let mut deallocations = 0u64;
        
        for i in 0..iterations {
            // Allocate memory
            let layout = Layout::from_size_align(alloc_size, 8).unwrap();
            let ptr = unsafe { libc::malloc(layout.size()) };
            
            if !ptr.is_null() {
                // Write to allocated memory to ensure it's actually allocated
                unsafe {
                    std::ptr::write_bytes(ptr, (i % 256) as u8, alloc_size);
                }
                allocations.push(ptr);
                
                // Periodically free some allocations
                if i % 100 == 0 && !allocations.is_empty() {
                    if let Some(old_ptr) = allocations.pop() {
                        unsafe { libc::free(old_ptr as *mut libc::c_void) };
                        deallocations += 1;
                    }
                }
            }
        }
        
        // Clean up remaining allocations
        for ptr in allocations {
            unsafe { libc::free(ptr as *mut libc::c_void) };
            deallocations += 1;
        }
        
        let elapsed = start.elapsed();
        let allocs_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("allocation_size".to_string(), alloc_size.to_string());
        metadata.insert("deallocations".to_string(), deallocations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: allocs_per_sec,
            throughput: allocs_per_sec * (alloc_size as f64),
            unit: "allocations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Cache performance benchmark
pub struct CachePerformance;

impl CachePerformance {
    pub fn new() -> Self {
        Self
    }
    
    /// Test L1 cache performance with different access patterns
    fn test_cache_access_pattern(pattern: &str, buffer: &mut [u8], iterations: u64) -> f64 {
        match pattern {
            "linear" => {
                // Linear access - should hit L1 cache well
                let start = Instant::now();
                let mut sum = 0u64;
                
                for _ in 0..iterations {
                    for chunk in buffer.chunks(64) {
                        for &byte in chunk {
                            sum ^= byte as u64;
                        }
                    }
                }
                
                start.elapsed().as_secs_f64()
            }
            "stride_64" => {
                // Access every 64th byte - should still hit L1 cache
                let start = Instant::now();
                let mut sum = 0u64;
                
                for _ in 0..iterations {
                    for i in (0..buffer.len()).step_by(64) {
                        sum ^= buffer[i] as u64;
                    }
                }
                
                start.elapsed().as_secs_f64()
            }
            "stride_512" => {
                // Access every 512th byte - likely to miss L1 cache
                let start = Instant::now();
                let mut sum = 0u64;
                
                for _ in 0..iterations {
                    for i in (0..buffer.len()).step_by(512) {
                        sum ^= buffer[i] as u64;
                    }
                }
                
                start.elapsed().as_secs_f64()
            }
            _ => 0.0,
        }
    }
}

impl Benchmark for CachePerformance {
    fn name(&self) -> &str {
        "Cache Performance"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut buffer = vec![0u8; L1_CACHE_SIZE * 2]; // 64KB buffer
        
        // Initialize buffer
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        let start = Instant::now();
        
        // Test different cache access patterns
        let linear_time = Self::test_cache_access_pattern("linear", &mut buffer, iterations / 3);
        let stride64_time = Self::test_cache_access_pattern("stride_64", &mut buffer, iterations / 3);
        let stride512_time = Self::test_cache_access_pattern("stride_512", &mut buffer, iterations / 3);
        
        let total_time = start.elapsed();
        let total_ops = iterations as f64;
        let ops_per_sec = total_ops / total_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("linear_access_time".to_string(), linear_time.to_string());
        metadata.insert("stride_64_time".to_string(), stride64_time.to_string());
        metadata.insert("stride_512_time".to_string(), stride512_time.to_string());
        metadata.insert("buffer_size".to_string(), buffer.len().to_string());
        
        // Calculate cache hit ratio (simplified)
        let cache_efficiency = if stride512_time > 0.0 {
            ((stride512_time - linear_time) / stride512_time) * 100.0
        } else {
            0.0
        };
        metadata.insert("cache_efficiency_estimate".to_string(), cache_efficiency.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: total_time,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * 8.0, // 8 bytes per cache line access
            unit: "cache_accesses/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Memory bandwidth benchmark
pub struct MemoryBandwidth;

impl MemoryBandwidth {
    pub fn new() -> Self {
        Self
    }
    
    /// Perform memory copy operation to test bandwidth
    fn memory_copy_test(src: &[u8], dst: &mut [u8], iterations: u64) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            dst.copy_from_slice(src);
        }
        
        start.elapsed()
    }
    
    /// Perform memory set operation
    fn memory_set_test(dst: &mut [u8], value: u8, iterations: u64) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            dst.fill(value);
        }
        
        start.elapsed()
    }
}

impl Benchmark for MemoryBandwidth {
    fn name(&self) -> &str {
        "Memory Bandwidth"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let size = 64 * 1024; // 64KB test size
        let mut src_buffer = vec![0u8; size];
        let mut dst_buffer = vec![0u8; size];
        let mut fill_buffer = vec![0u8; size];
        
        // Initialize buffers
        for (i, byte) in src_buffer.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        let start = Instant::now();
        
        // Memory copy test
        let copy_time = Self::memory_copy_test(&src_buffer, &mut dst_buffer, iterations / 2);
        
        // Memory set test
        let set_time = Self::memory_set_test(&mut fill_buffer, 0xFF, iterations / 2);
        
        let total_time = start.elapsed();
        let total_bytes = (iterations as f64) * (size as f64);
        let bandwidth = total_bytes / total_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("test_size".to_string(), size.to_string());
        metadata.insert("copy_time_sec".to_string(), copy_time.as_secs_f64().to_string());
        metadata.insert("set_time_sec".to_string(), set_time.as_secs_f64().to_string());
        metadata.insert("total_time_sec".to_string(), total_time.as_secs_f64().to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: total_time,
            iterations,
            operations_per_second: (iterations as f64) / total_time.as_secs_f64(),
            throughput: bandwidth,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Complete memory benchmarking suite
pub struct MemoryBenchmarkSuite {
    pub sequential_read: SequentialRead,
    pub sequential_write: SequentialWrite,
    pub random_access: RandomAccess,
    pub allocation: MemoryAllocation,
    pub cache_perf: CachePerformance,
    pub bandwidth: MemoryBandwidth,
}

impl MemoryBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            sequential_read: SequentialRead::new(),
            sequential_write: SequentialWrite::new(),
            random_access: RandomAccess::new(),
            allocation: MemoryAllocation::new(),
            cache_perf: CachePerformance::new(),
            bandwidth: MemoryBandwidth::new(),
        }
    }
    
    /// Run all memory benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use super::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(false);
        let benchmarks: Vec<_> = vec![
            &self.sequential_read,
            &self.sequential_write,
            &self.random_access,
            &self.allocation,
            &self.cache_perf,
            &self.bandwidth,
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sequential_read() {
        let bench = SequentialRead::new();
        let result = bench.run(10).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.throughput > 0.0);
    }
    
    #[test]
    fn test_random_access() {
        let bench = RandomAccess::new();
        let result = bench.run(1000).unwrap();
        assert!(result.operations_per_second > 0.0);
    }
}