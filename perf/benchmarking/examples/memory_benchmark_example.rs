//! Example: Custom Memory Benchmark Implementation
//! 
//! This example demonstrates how to create custom memory benchmarks
//! using the MultiOS benchmarking framework.

use multios_benchmarking::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::alloc::{GlobalAlloc, Layout};

/// Custom allocator for tracking allocations
struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        libc::malloc(layout.size())
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        libc::free(ptr as *mut libc::c_void);
    }
}

/// Benchmark for memory allocation patterns
pub struct AllocationPatterns;

impl AllocationPatterns {
    pub fn new() -> Self {
        Self
    }
    
    /// Test random allocation sizes
    fn random_allocations(iterations: u64) -> (Duration, u64) {
        let start = Instant::now();
        let mut allocations = Vec::new();
        
        for i in 0..iterations {
            let size = ((i * 37) % 4096) + 16; // Vary allocation sizes
            let layout = Layout::from_size_align(size, 8).unwrap();
            
            unsafe {
                let ptr = libc::malloc(layout.size());
                if !ptr.is_null() {
                    allocations.push((ptr, layout));
                }
            }
        }
        
        let elapsed = start.elapsed();
        
        // Clean up
        for (ptr, _) in allocations {
            unsafe {
                libc::free(ptr as *mut libc::c_void);
            }
        }
        
        (elapsed, iterations)
    }
    
    /// Test allocation size impact
    fn size_impact_test(size: usize, iterations: u64) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            unsafe {
                let ptr = libc::malloc(size);
                if !ptr.is_null() {
                    // Touch the memory to ensure it's actually allocated
                    std::ptr::write_bytes(ptr, 0xAA, size);
                    libc::free(ptr as *mut libc::c_void);
                }
            }
        }
        
        start.elapsed()
    }
}

impl Benchmark for AllocationPatterns {
    fn name(&self) -> &str {
        "Memory Allocation Patterns"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let (random_time, successful_allocs) = Self::random_allocations(iterations / 2);
        let small_alloc_time = Self::size_impact_test(64, iterations / 4);
        let large_alloc_time = Self::size_impact_test(4096, iterations / 4);
        
        let total_time = random_time + small_alloc_time + large_alloc_time;
        let ops_per_sec = (iterations as f64) / total_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("successful_allocations".to_string(), successful_allocs.to_string());
        metadata.insert("random_allocation_time_ms".to_string(), random_time.as_millis().to_string());
        metadata.insert("small_64b_time_ms".to_string(), small_alloc_time.as_millis().to_string());
        metadata.insert("large_4kb_time_ms".to_string(), large_alloc_time.as_millis().to_string());
        
        // Calculate allocation rates
        metadata.insert("random_alloc_rate".to_string(), 
            (successful_allocs as f64 / random_time.as_secs_f64()).to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: total_time,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec,
            unit: "allocations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Memory access pattern benchmark
pub struct MemoryAccessPatterns;

impl MemoryAccessPatterns {
    pub fn new() -> Self {
        Self
    }
    
    /// Test different memory access patterns
    fn test_access_patterns(buffer: &mut [u8], pattern: &str, iterations: u64) -> Duration {
        let start = Instant::now();
        
        match pattern {
            "sequential" => {
                for _ in 0..iterations {
                    for chunk in buffer.chunks(64) {
                        for byte in chunk.iter_mut() {
                            *byte = *byte ^ 0xFF;
                        }
                    }
                }
            }
            "reverse" => {
                for _ in 0..iterations {
                    for chunk in buffer.chunks(64).rev() {
                        for byte in chunk.iter_mut() {
                            *byte = *byte ^ 0xFF;
                        }
                    }
                }
            }
            "random" => {
                use std::collections::HashSet;
                let mut accessed = HashSet::new();
                let len = buffer.len();
                
                for _ in 0..iterations {
                    // Generate random access pattern
                    for _ in 0..(len / 64) {
                        let offset = (fast_random() as usize) % len;
                        if let Some(byte) = buffer.get_mut(offset) {
                            *byte = *byte ^ 0xFF;
                        }
                    }
                }
            }
            _ => {}
        }
        
        start.elapsed()
    }
    
    /// Simple fast random number generator
    fn fast_random() -> u32 {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0x12345678);
        
        let prev = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut x = prev ^ (prev << 13);
        x ^= x >> 17;
        x ^= x << 5;
        x
    }
}

impl Benchmark for MemoryAccessPatterns {
    fn name(&self) -> &str {
        "Memory Access Patterns"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let buffer_size = 64 * 1024; // 64KB buffer
        let mut buffer = vec![0u8; buffer_size];
        
        // Initialize buffer
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        
        let test_iterations = iterations / 3;
        
        let sequential_time = Self::test_access_patterns(&mut buffer, "sequential", test_iterations);
        let reverse_time = Self::test_access_patterns(&mut buffer, "reverse", test_iterations);
        let random_time = Self::test_access_patterns(&mut buffer, "random", test_iterations);
        
        let total_time = sequential_time + reverse_time + random_time;
        let ops_per_sec = (iterations as f64) / total_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("buffer_size_kb".to_string(), (buffer_size / 1024).to_string());
        metadata.insert("sequential_time_ms".to_string(), sequential_time.as_millis().to_string());
        metadata.insert("reverse_time_ms".to_string(), reverse_time.as_millis().to_string());
        metadata.insert("random_time_ms".to_string(), random_time.as_millis().to_string());
        
        // Calculate access rates
        let total_bytes = (iterations as f64 * buffer_size as f64);
        metadata.insert("sequential_rate_mbps".to_string(), 
            (total_bytes / 3.0 / sequential_time.as_secs_f64() / 1_000_000.0).to_string());
        metadata.insert("random_rate_mbps".to_string(), 
            (total_bytes / 3.0 / random_time.as_secs_f64() / 1_000_000.0).to_string());
        
        // Calculate access pattern efficiency
        let efficiency = (sequential_time.as_secs_f64() / random_time.as_secs_f64()) * 100.0;
        metadata.insert("random_vs_sequential_efficiency".to_string(), efficiency.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: total_time,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: (total_bytes as f64) / total_time.as_secs_f64(),
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Memory compression benchmark
pub struct MemoryCompression;

impl MemoryCompression {
    pub fn new() -> Self {
        Self
    }
    
    /// Simple compression algorithm (RLE - Run Length Encoding)
    fn rle_compress(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            let current = data[i];
            let mut count = 1;
            
            // Count consecutive identical bytes
            while i + count < data.len() && data[i + count] == current && count < 255 {
                count += 1;
            }
            
            compressed.push(count);
            compressed.push(current);
            i += count;
        }
        
        compressed
    }
    
    fn rle_decompress(compressed: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();
        
        let mut i = 0;
        while i + 1 < compressed.len() {
            let count = compressed[i];
            let value = compressed[i + 1];
            
            for _ in 0..count {
                decompressed.push(value);
            }
            
            i += 2;
        }
        
        decompressed
    }
}

impl Benchmark for MemoryCompression {
    fn name(&self) -> &str {
        "Memory Compression"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Memory
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        // Create test data with repetitive patterns
        let mut test_data = Vec::new();
        for _ in 0..1000 {
            test_data.extend_from_slice(&[0xAA; 10]);
            test_data.extend_from_slice(&[0xBB; 10]);
            test_data.extend_from_slice(&[0xCC; 10]);
        }
        
        let start = Instant::now();
        let mut total_compressed_bytes = 0u64;
        let mut total_decompressed_bytes = 0u64;
        
        for _ in 0..iterations {
            let compressed = Self::rle_compress(&test_data);
            let decompressed = Self::rle_decompress(&compressed);
            
            total_compressed_bytes += compressed.len() as u64;
            total_decompressed_bytes += decompressed.len() as u64;
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        // Calculate compression ratio
        let original_size = test_data.len();
        let compressed_size = test_data.len() * 2 / 3; // Estimated compressed size
        let compression_ratio = (original_size as f64) / (compressed_size as f64);
        
        let mut metadata = HashMap::new();
        metadata.insert("original_size".to_string(), original_size.to_string());
        metadata.insert("estimated_compressed_size".to_string(), compressed_size.to_string());
        metadata.insert("compression_ratio".to_string(), format!("{:.2}:1", compression_ratio));
        metadata.insert("total_compressed_bytes".to_string(), total_compressed_bytes.to_string());
        metadata.insert("total_decompressed_bytes".to_string(), total_decompressed_bytes.to_string());
        
        let throughput = (total_compressed_bytes + total_decompressed_bytes) as f64 / elapsed.as_secs_f64();
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Example memory benchmark suite
pub struct CustomMemoryBenchmarkSuite {
    pub allocation_patterns: AllocationPatterns,
    pub access_patterns: MemoryAccessPatterns,
    pub compression: MemoryCompression,
}

impl CustomMemoryBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            allocation_patterns: AllocationPatterns::new(),
            access_patterns: MemoryAccessPatterns::new(),
            compression: MemoryCompression::new(),
        }
    }
    
    /// Run all custom memory benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use multios_benchmarking::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(true);
        let benchmarks = vec![
            &self.allocation_patterns,
            &self.access_patterns,
            &self.compression,
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Custom Memory Benchmark Example");
    println!("===============================");
    
    let suite = CustomMemoryBenchmarkSuite::new();
    let iterations = 1000;
    
    println!("Running {} iterations per benchmark...", iterations);
    
    let results = suite.run_all(iterations)?;
    
    println!("\nBenchmark Results:");
    println!("==================");
    
    for result in results {
        println!("\n{}:", result.name);
        println!("  Duration: {:?}", result.duration);
        println!("  Operations/Second: {:.2}", result.operations_per_second);
        println!("  Throughput: {}", 
            if result.throughput > 1_000_000.0 {
                format!("{:.2} M{}", result.throughput / 1_000_000.0, result.unit)
            } else if result.throughput > 1_000.0 {
                format!("{:.2} k{}", result.throughput / 1_000.0, result.unit)
            } else {
                format!("{:.2} {}", result.throughput, result.unit)
            });
        
        println!("  Metadata:");
        for (key, value) in &result.metadata {
            println!("    {}: {}", key, value);
        }
    }
    
    println!("\nExample completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allocation_patterns() {
        let bench = AllocationPatterns::new();
        let result = bench.run(10).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.metadata.contains_key("successful_allocations"));
    }
    
    #[test]
    fn test_rle_compression() {
        let test_data = vec![0xAA, 0xAA, 0xAA, 0xBB, 0xBB, 0xCC];
        let compressed = MemoryCompression::rle_compress(&test_data);
        let decompressed = MemoryCompression::rle_decompress(&compressed);
        
        assert_eq!(test_data, decompressed);
        assert!(compressed.len() < test_data.len());
    }
}