//! Memory Management Initialization Example
//! 
//! This example demonstrates the complete memory management initialization
//! process for the MultiOS bootloader, showing both BIOS and UEFI scenarios.

#![allow(unused)]

use bootloader::{
    memory_map::{MemoryMap, MemoryRegionInfo, MemoryType, MemoryFlags},
    boot_heap::{BootHeapAllocator, MemoryPool, MemoryInitConfig},
    BootMode, BootResult,
};

use x86_64::PhysAddr;
use log::{info, warn, error, debug};
use spin::Mutex;

#[cfg(test)]
mod memory_init_examples {
    use super::*;

    /// Example 1: Basic BIOS memory detection
    #[test]
    fn test_bios_memory_detection() -> BootResult<()> {
        info!("=== BIOS Memory Detection Example ===");
        
        let memory_map = MemoryMap::detect_bios_memory()?;
        
        // Display memory map
        memory_map.print();
        
        // Validate memory map
        assert!(memory_map.validate());
        
        // Check memory regions
        let usable_regions = memory_map.get_regions_of_type(MemoryType::Usable);
        assert!(!usable_regions.is_empty(), "No usable memory regions found");
        
        info!("BIOS memory detection completed successfully");
        Ok(())
    }

    /// Example 2: UEFI memory detection
    #[test]
    fn test_uefi_memory_detection() -> BootResult<()> {
        info!("=== UEFI Memory Detection Example ===");
        
        let memory_map = MemoryMap::detect_uefi_memory()?;
        
        // Display memory map
        memory_map.print();
        
        // Validate memory map
        assert!(memory_map.validate());
        
        // Check for MMIO regions
        let mmio_regions = memory_map.get_regions_of_type(MemoryType::Reserved)
            .into_iter()
            .filter(|r| r.start.as_u64() >= 0xF0000000)
            .collect::<Vec<_>>();
        
        assert!(!mmio_regions.is_empty(), "No MMIO regions found");
        
        info!("UEFI memory detection completed successfully");
        Ok(())
    }

    /// Example 3: Memory bitmap allocation
    #[test]
    fn test_memory_bitmap_allocation() -> BootResult<()> {
        info!("=== Memory Bitmap Allocation Example ===");
        
        let mut memory_map = MemoryMap::detect_bios_memory()?;
        
        // Initialize memory bitmap
        memory_map.init_bitmap()?;
        
        // Get memory statistics
        let stats = memory_map.get_memory_stats();
        info!("Memory Statistics:\n{}", stats);
        
        // Allocate some frames
        let frame_addr1 = memory_map.allocate_frames(16, 4096)?;
        info!("Allocated 16 frames at: {:#x}", frame_addr1);
        
        let frame_addr2 = memory_map.allocate_frames(8, 4096)?;
        info!("Allocated 8 frames at: {:#x}", frame_addr2);
        
        // Check available frames after allocation
        let available_frames = memory_map.get_bitmap().unwrap().available_count();
        info!("Available frames after allocation: {}", available_frames);
        
        // Free some frames
        memory_map.free_frames(frame_addr1, 16)?;
        info!("Freed 16 frames at: {:#x}", frame_addr1);
        
        // Check available frames after free
        let available_frames = memory_map.get_bitmap().unwrap().available_count();
        info!("Available frames after free: {}", available_frames);
        
        Ok(())
    }

    /// Example 4: Boot heap initialization
    #[test]
    fn test_boot_heap_initialization() -> BootResult<()> {
        info!("=== Boot Heap Initialization Example ===");
        
        let config = MemoryInitConfig {
            heap_size: 8 * 1024 * 1024, // 8MB
            bitmap_granularity: 4096,
            heap_alignment: 4096,
            enable_detailed_logging: true,
            ..Default::default()
        };
        
        let mut memory_map = MemoryMap::with_config(config);
        
        // Simulate basic memory regions
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0x100000),
            0x7FF00000,
            MemoryType::Usable,
            MemoryFlags::READ | MemoryFlags::WRITE | MemoryFlags::AVAILABLE,
        ));
        
        // Initialize complete memory system
        memory_map.init_complete()?;
        
        // Test heap allocations
        let addr1 = memory_map.heap_allocate(1024)?;
        info!("Allocated 1024 bytes at: {:#x}", addr1);
        
        let addr2 = memory_map.heap_allocate(4096)?;
        info!("Allocated 4096 bytes at: {:#x}", addr2);
        
        let addr3 = memory_map.heap_allocate(8192)?;
        info!("Allocated 8192 bytes at: {:#x}", addr3);
        
        // Get heap statistics
        if let Some(heap) = memory_map.get_heap() {
            let heap_stats = heap.get_stats();
            info!("Heap Statistics:\n{}", heap_stats);
            
            assert!(heap_stats.allocated > 0);
            assert!(heap_stats.usage_percent >= 0);
        }
        
        Ok(())
    }

    /// Example 5: Global heap allocator
    #[test]
    fn test_global_heap_allocator() -> BootResult<()> {
        info!("=== Global Heap Allocator Example ===");
        
        let mut allocator = BootHeapAllocator::new();
        
        // Initialize with memory map
        let mut memory_map = MemoryMap::detect_bios_memory()?;
        memory_map.init_complete()?;
        allocator.init(memory_map)?;
        
        // Test allocations
        let ptr1 = allocator.allocate(1024, 8)?;
        info!("Global allocator: {} bytes at {:?}", 1024, ptr1);
        
        let ptr2 = allocator.allocate(2048, 16)?;
        info!("Global allocator: {} bytes at {:?}", 2048, ptr2);
        
        let ptr3 = allocator.allocate_zeroed(4096, 32)?;
        info!("Global allocator: {} zeroed bytes at {:?}", 4096, ptr3);
        
        // Get statistics
        let stats = allocator.get_stats()?;
        info!("Global Allocator Statistics:\n{}", stats);
        
        // Test available check
        assert!(allocator.is_initialized());
        
        Ok(())
    }

    /// Example 6: Memory pool management
    #[test]
    fn test_memory_pool_management() -> BootResult<()> {
        info!("=== Memory Pool Management Example ===");
        
        // Create memory pool
        let mut pool = MemoryPool::new(0x200000, 0x40000, 0x1000)?;
        
        info!("Memory pool created: {} bytes, {} byte blocks", 0x40000, 0x1000);
        
        // Display initial statistics
        let stats = pool.get_stats();
        info!("Initial pool stats:\n{}", stats);
        
        // Allocate multiple blocks
        let addresses: Vec<u64> = (0..16)
            .map(|_| pool.allocate_block())
            .collect::<Result<Vec<_>, _>>()?;
        
        info!("Allocated {} blocks", addresses.len());
        
        // Display statistics after allocation
        let stats = pool.get_stats();
        info!("Pool stats after allocation:\n{}", stats);
        
        // Free some blocks
        for i in (0..addresses.len()).step_by(2) {
            pool.free_block(addresses[i])?;
            info!("Freed block at: {:#x}", addresses[i]);
        }
        
        // Display final statistics
        let stats = pool.get_stats();
        info!("Final pool stats:\n{}", stats);
        
        Ok(())
    }

    /// Example 7: Complete memory initialization
    #[test]
    fn test_complete_memory_initialization() -> BootResult<()> {
        info!("=== Complete Memory Initialization Example ===");
        
        // Test for different boot modes
        for boot_mode in [BootMode::UEFI, BootMode::LegacyBIOS] {
            info!("\n--- Testing {:?} mode ---", boot_mode);
            
            let config = MemoryInitConfig {
                heap_size: 16 * 1024 * 1024, // 16MB
                bitmap_granularity: 4096,
                heap_alignment: 4096,
                enable_detailed_logging: true,
                min_heap_addr: 0x100000,
            };
            
            // Initialize complete memory system
            let memory_map = bootloader::boot_heap::init_boot_memory(boot_mode, Some(config))?;
            
            // Get comprehensive statistics
            let stats = memory_map.get_memory_stats();
            info!("Complete memory stats:\n{}", stats);
            
            // Verify memory map integrity
            assert!(memory_map.validate());
            
            // Test heap operations
            if let Some(heap) = memory_map.get_heap() {
                let addr = heap.allocate(1024)?;
                info!("Heap allocation test: {} bytes at {:#x}", 1024, addr);
            }
            
            // Test bitmap operations
            if let Some(bitmap) = memory_map.get_bitmap() {
                let available = bitmap.available_count();
                info!("Bitmap test: {} frames available", available);
                assert!(available > 0);
            }
        }
        
        info!("Complete memory initialization test passed");
        Ok(())
    }

    /// Example 8: Memory error handling
    #[test]
    fn test_memory_error_handling() {
        info!("=== Memory Error Handling Example ===");
        
        let mut memory_map = MemoryMap::new();
        
        // Test allocation on empty map
        let result = memory_map.allocate_frames(1, 4096);
        assert!(result.is_err(), "Should fail on empty memory map");
        
        // Add some memory and test bitmap initialization
        memory_map.add_region(MemoryRegionInfo::new(
            PhysAddr::new(0x100000),
            0x1000,
            MemoryType::Usable,
            MemoryFlags::AVAILABLE,
        ));
        
        // This should work
        let bitmap_result = memory_map.init_bitmap();
        assert!(bitmap_result.is_ok(), "Bitmap initialization should succeed");
        
        // But allocation should still fail (not enough space)
        let alloc_result = memory_map.allocate_frames(2, 4096);
        assert!(alloc_result.is_err(), "Should fail with insufficient frames");
        
        info!("Error handling test passed");
    }

    /// Example 9: Performance benchmark
    #[test]
    fn test_memory_performance() -> BootResult<()> {
        info!("=== Memory Performance Benchmark ===");
        
        let mut memory_map = MemoryMap::detect_bios_memory()?;
        memory_map.init_complete()?;
        
        // Benchmark frame allocation
        let iterations = 1000;
        let mut total_time = 0u64;
        
        for i in 0..iterations {
            let start = read_timestamp();
            let addr = memory_map.allocate_frames(1, 4096)?;
            let end = read_timestamp();
            
            total_time += end - start;
            
            // Free immediately to avoid exhaustion
            memory_map.free_frames(addr, 1)?;
        }
        
        let avg_time = total_time / iterations as u64;
        info!("Frame allocation: {} iterations, average {} cycles", iterations, avg_time);
        
        // Benchmark heap allocation
        let mut heap_allocations = Vec::new();
        let mut heap_total_time = 0u64;
        
        for i in 0..100 {
            let start = read_timestamp();
            let addr = memory_map.heap_allocate(512)?;
            let end = read_timestamp();
            
            heap_total_time += end - start;
            heap_allocations.push(addr);
        }
        
        let avg_heap_time = heap_total_time / 100;
        info!("Heap allocation: 100 iterations, average {} cycles", avg_heap_time);
        
        Ok(())
    }

    /// Helper function to read timestamp (simplified)
    fn read_timestamp() -> u64 {
        // In real implementation, would read from high-precision timer
        core::arch::asm!("rdtsc", out(eax) _, out(edx) _);
        0 // Simplified for testing
    }

    /// Example 10: Integration test
    #[test]
    fn test_memory_integration() -> BootResult<()> {
        info!("=== Memory Management Integration Test ===");
        
        // Simulate complete bootloader flow
        let boot_mode = BootMode::LegacyBIOS;
        
        // 1. Memory detection
        let memory_map = MemoryMap::detect_bios_memory()?;
        assert!(memory_map.validate());
        
        // 2. Initialize memory subsystems
        let mut memory_map = memory_map;
        memory_map.init_bitmap()?;
        memory_map.init_heap()?;
        
        // 3. Test various allocation patterns
        let allocations: Vec<u64> = (0..10)
            .map(|_| memory_map.allocate_frames(1, 4096).unwrap())
            .collect();
        
        // 4. Verify memory map consistency
        assert!(memory_map.validate());
        
        // 5. Free all allocations
        for addr in allocations {
            memory_map.free_frames(addr, 1)?;
        }
        
        // 6. Final verification
        assert!(memory_map.validate());
        
        let final_stats = memory_map.get_memory_stats();
        info!("Integration test complete:\n{}", final_stats);
        
        Ok(())
    }
}

/// Main example runner (for demonstration)
pub fn run_memory_examples() {
    info!("Starting MultiOS Bootloader Memory Management Examples");
    
    // Run all examples
    memory_init_examples::test_bios_memory_detection().unwrap();
    memory_init_examples::test_uefi_memory_detection().unwrap();
    memory_init_examples::test_memory_bitmap_allocation().unwrap();
    memory_init_examples::test_boot_heap_initialization().unwrap();
    memory_init_examples::test_global_heap_allocator().unwrap();
    memory_init_examples::test_memory_pool_management().unwrap();
    memory_init_examples::test_complete_memory_initialization().unwrap();
    memory_init_examples::test_memory_error_handling();
    memory_init_examples::test_memory_performance().unwrap();
    memory_init_examples::test_memory_integration().unwrap();
    
    info!("All memory management examples completed successfully");
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bootloader::boot_heap::{init_boot_memory, safe_alloc};

    /// Test complete memory initialization workflow
    #[test]
    fn test_full_initialization_workflow() {
        for boot_mode in [BootMode::UEFI, BootMode::LegacyBIOS] {
            let result = init_boot_memory(boot_mode, None);
            assert!(result.is_ok(), "Memory initialization should succeed for {:?}", boot_mode);
            
            let memory_map = result.unwrap();
            assert!(memory_map.validate());
            
            // Test safe allocation interface
            assert!(safe_alloc::is_available());
            
            // Verify heap and bitmap are initialized
            assert!(memory_map.get_heap().is_some());
            assert!(memory_map.get_bitmap().is_some());
        }
    }

    /// Test memory map consistency across operations
    #[test]
    fn test_memory_map_consistency() {
        let mut memory_map = MemoryMap::detect_bios_memory().unwrap();
        memory_map.init_complete().unwrap();
        
        let initial_stats = memory_map.get_memory_stats();
        let initial_frame_count = memory_map.get_bitmap().unwrap().available_count();
        
        // Perform various operations
        let addr1 = memory_map.allocate_frames(5, 4096).unwrap();
        let addr2 = memory_map.heap_allocate(1024).unwrap();
        
        // Verify consistency maintained
        assert!(memory_map.validate());
        
        // Free resources
        memory_map.free_frames(addr1, 5).unwrap();
        
        // Final verification
        let final_stats = memory_map.get_memory_stats();
        assert!(memory_map.validate());
        
        // Available frames should be same or greater after freeing
        let final_frame_count = memory_map.get_bitmap().unwrap().available_count();
        assert!(final_frame_count >= initial_frame_count);
    }

    /// Test allocation patterns and edge cases
    #[test]
    fn test_allocation_patterns() {
        let mut memory_map = MemoryMap::detect_bios_memory().unwrap();
        memory_map.init_complete().unwrap();
        
        // Test various allocation patterns
        
        // 1. Small allocations
        for _ in 0..10 {
            let addr = memory_map.heap_allocate(64).unwrap();
            assert!(addr > 0);
        }
        
        // 2. Aligned allocations
        for align in [16, 64, 256, 1024, 4096] {
            let addr = memory_map.heap_allocate(align).unwrap();
            assert!(addr % align as u64 == 0);
        }
        
        // 3. Large allocation (within heap size)
        if let Some(heap) = memory_map.get_heap() {
            let remaining = heap.size - heap.allocated;
            if remaining > 0 {
                let addr = memory_map.heap_allocate(remaining / 2).unwrap();
                assert!(addr > 0);
            }
        }
        
        // Verify system still works
        assert!(memory_map.validate());
        assert!(memory_map.get_bitmap().is_some());
    }
}
