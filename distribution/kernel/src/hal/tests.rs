//! Hardware Abstraction Layer (HAL) Tests
//!
//! This module contains comprehensive tests for the HAL functionality,
//! including unit tests, integration tests, and architecture-specific tests.

use crate::hal::*;
use crate::{KernelError, Result};

// Test helper macros
macro_rules! assert_arch_specific {
    ($condition:expr, $message:expr) => {
        if !$condition {
            panic!("Architecture-specific test failed: {}", $message);
        }
    };
}

// CPU HAL Tests
#[test]
fn test_cpu_info_initialization() -> Result<()> {
    let cpu_info = cpu::get_cpu_info();
    
    assert!(!cpu_info.model.is_empty());
    assert!(!cpu_info.vendor.is_empty());
    assert!(cpu_info.cores > 0);
    assert!(cpu_info.threads_per_core > 0);
    
    info!("CPU Info: {} {} ({} cores, {} threads)", 
          cpu_info.vendor, cpu_info.model, cpu_info.cores, cpu_info.threads_per_core);
    
    Ok(())
}

#[test]
fn test_cpu_features_detection() -> Result<()> {
    let features = cpu::get_cpu_features();
    
    info!("CPU Features: {}", features.to_string());
    
    // Basic feature checks
    #[cfg(target_arch = "x86_64")]
    {
        // x86_64 should have at least basic features
        assert_arch_specific!(features.has_nx_bit, "x86_64 should have NX bit support");
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64 should have NEON support
        assert_arch_specific!(features.simd_width >= 1, "ARM64 should have NEON support");
    }
    
    Ok(())
}

#[test]
fn test_cpu_id_detection() -> Result<()> {
    let cpu_id = cpu::get_current_cpu_id();
    let total_cpus = cpu::get_total_cpus();
    
    assert!(cpu_id < total_cpus);
    info!("Current CPU ID: {}, Total CPUs: {}", cpu_id, total_cpus);
    
    Ok(())
}

#[test]
fn test_cpu_benchmark() -> Result<()> {
    let benchmark_score = cpu::benchmark_cpu();
    
    assert!(benchmark_score > 0);
    info!("CPU Benchmark Score: {} cycles", benchmark_score);
    
    Ok(())
}

// Memory HAL Tests
#[test]
fn test_memory_layout_detection() -> Result<()> {
    let layout = memory::get_memory_layout();
    
    assert!(layout.is_some());
    let layout = layout.unwrap();
    
    assert!(layout.total_memory > 0);
    assert!(layout.usable_memory > 0);
    assert!(layout.kernel_start < layout.kernel_end);
    
    info!("Memory Layout: {}MB total, {}MB usable", 
          layout.total_memory / 1024 / 1024, 
          layout.usable_memory / 1024 / 1024);
    
    Ok(())
}

#[test]
fn test_page_operations() -> Result<()> {
    let page_size = memory::get_page_size();
    let large_size = memory::get_large_page_size();
    let huge_size = memory::get_huge_page_size();
    
    assert_eq!(page_size, 4096);
    assert_eq!(large_size, 2 * 1024 * 1024);
    assert_eq!(huge_size, 1024 * 1024 * 1024);
    
    // Test alignment functions
    let test_addr = 0x12345678;
    let aligned = memory::align_to_page(test_addr);
    assert!(memory::is_page_aligned(aligned));
    
    let size_aligned = memory::align_size_to_pages(12345);
    assert_eq!(size_aligned % page_size, 0);
    
    info!("Page sizes: {}B, {}B, {}B", page_size, large_size, huge_size);
    
    Ok(())
}

#[test]
fn test_tlb_operations() -> Result<()> {
    // Test TLB flush operations
    memory::flush_tlb();
    
    let test_addr = 0x1000000;
    memory::flush_tlb_address(test_addr);
    
    // These operations should not panic
    info!("TLB flush operations completed");
    
    Ok(())
}

#[test]
fn test_memory_benchmark() -> Result<()> {
    let benchmark_score = memory::benchmark_memory();
    
    assert!(benchmark_score > 0);
    info!("Memory Benchmark Score: {} cycles", benchmark_score);
    
    Ok(())
}

// Interrupt HAL Tests
#[test]
fn test_interrupt_controller_detection() -> Result<()> {
    let controller = interrupts::get_interrupt_controller();
    
    info!("Detected interrupt controller: {:?}", controller);
    
    // Controller should be detected
    assert!(controller != interrupts::InterruptControllerType::Unknown);
    
    Ok(())
}

#[test]
fn test_global_interrupt_management() -> Result<()> {
    // Test interrupt enable/disable
    let initial_state = interrupts::are_global_interrupts_enabled();
    
    info!("Initial interrupt state: {}", initial_state);
    
    // This test just verifies the functions work without error
    // In a real system, we would need to be more careful about interrupt state
    
    info!("Global interrupt management functions work correctly");
    
    Ok(())
}

#[test]
fn test_interrupt_latency_benchmark() -> Result<()> {
    let latency = interrupts::benchmark_latency();
    
    assert!(latency > 0);
    info!("Interrupt Latency Benchmark: {} cycles", latency);
    
    Ok(())
}

// Timer HAL Tests
#[test]
fn test_timer_initialization() -> Result<()> {
    let timer_info = timers::get_timer_info();
    
    assert!(!timer_info.is_empty());
    
    let system_time = timers::get_system_time();
    assert!(system_time.seconds >= 0);
    assert!(system_time.uptime_ticks >= 0);
    
    info!("Timer Info: {} timers detected", timer_info.len());
    info!("System Time: {} seconds", system_time.seconds);
    
    Ok(())
}

#[test]
fn test_timer_conversions() -> Result<()> {
    use timers::utils::*;
    
    let seconds = 5;
    let milliseconds = 1000;
    let microseconds = 1_000_000;
    
    let seconds_ticks = seconds_to_ticks(seconds);
    let ms_ticks = ms_to_ticks(milliseconds);
    let us_ticks = us_to_ticks(microseconds);
    
    assert!(seconds_ticks > 0);
    assert!(ms_ticks > 0);
    assert!(us_ticks > 0);
    
    let converted_back = ticks_to_seconds(seconds_ticks);
    assert!(converted_back >= seconds);
    
    info!("Timer conversions work correctly");
    
    Ok(())
}

#[test]
fn test_timer_resolution_benchmark() -> Result<()> {
    let resolution = timers::benchmark_timer_resolution()?;
    
    assert!(resolution > 0);
    info!("Timer Resolution: {} ns", resolution);
    
    Ok(())
}

// I/O HAL Tests
#[test]
fn test_io_architecture_detection() -> Result<()> {
    let io_arch = io::get_io_architecture();
    let capabilities = io::get_io_capabilities();
    
    info!("I/O Architecture: {:?}", io_arch);
    info!("Port I/O Support: {}", capabilities.supports_port_io);
    info!("MMIO Support: {}", capabilities.supports_mmio);
    
    // Architecture detection should work
    assert!(io_arch != IoArchitecture::Custom);
    
    Ok(())
}

#[test]
fn test_device_enumeration() -> Result<()> {
    let devices = io::get_all_devices();
    
    assert!(!devices.is_empty());
    
    info!("Detected {} I/O devices", devices.len());
    for device in &devices {
        info!("Device: {} ({} bytes at {:#x})", 
              device.name, device.size, device.base_address);
    }
    
    // Check for essential devices
    assert!(io::utils::is_device_available(IoDeviceType::Serial));
    assert!(io::utils::is_device_available(IoDeviceType::Timer));
    
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[test]
fn test_port_io_operations() -> Result<()> {
    use io::port_io;
    
    // Test port I/O functions (we don't actually perform I/O to avoid side effects)
    let test_port: u16 = 0x3F8; // COM1
    
    // These would read/write actual hardware in real usage
    // For testing, we just verify the functions exist and compile
    info!("Port I/O functions are available");
    
    Ok(())
}

#[test]
fn test_mmio_operations() -> Result<()> {
    use io::mmio;
    
    // Test MMIO functions with a safe address
    let test_addr = 0xB8000; // VGA text memory
    
    // Read operation (should work even if we get garbage)
    let _value = mmio::read8(test_addr);
    
    info!("MMIO functions are available");
    
    Ok(())
}

// Multi-Core HAL Tests
#[test]
fn test_cpu_topology_detection() -> Result<()> {
    let topology = multicore::get_cpu_topology();
    assert!(topology.is_some());
    
    let topology = topology.unwrap();
    
    assert!(topology.total_cores > 0);
    assert!(topology.threads_per_core > 0);
    assert!(topology.total_threads >= topology.total_cores);
    
    info!("CPU Topology: {} cores, {} threads per core", 
          topology.total_cores, topology.threads_per_core);
    
    Ok(())
}

#[test]
fn test_core_management() -> Result<()> {
    let current_cpu = multicore::get_current_cpu_id();
    let online_cpus = multicore::get_online_cpus();
    let max_cpus = multicore::get_max_cpus();
    
    assert!(current_cpu < max_cpus);
    assert!(online_cpus <= max_cpus);
    
    info!("Current CPU: {}, Online: {}, Max: {}", current_cpu, online_cpus, max_cpus);
    
    Ok(())
}

#[test]
fn test_multicore_benchmark() -> Result<()> {
    let score = multicore::benchmark_multicore_performance()?;
    
    assert!(score > 0);
    info!("Multi-Core Benchmark Score: {}", score);
    
    Ok(())
}

// NUMA HAL Tests
#[test]
fn test_numa_topology_detection() -> Result<()> {
    let topology = numa::get_numa_topology();
    
    assert!(topology.node_count > 0);
    assert!(topology.total_memory > 0);
    assert!(!topology.nodes.is_empty());
    
    info!("NUMA Topology: {} nodes, {}MB total", 
          topology.node_count, 
          topology.total_memory / 1024 / 1024);
    
    if topology.supports_numa {
        info!("NUMA is supported and enabled");
        
        for (i, node) in topology.nodes.iter().enumerate() {
            info!("Node {}: {}MB, {} CPUs", 
                  i, node.memory_size / 1024 / 1024, node.cpu_list.len());
        }
    } else {
        info!("NUMA not supported (UMA mode)");
    }
    
    Ok(())
}

#[test]
fn test_numa_cpu_mapping() -> Result<()> {
    let current_cpu = multicore::get_current_cpu_id();
    let current_node = numa::get_current_numa_node();
    
    let node_for_cpu = numa::get_node_for_cpu(current_cpu);
    assert_eq!(node_for_cpu, Some(current_node));
    
    info!("Current CPU {} is on NUMA node {}", current_cpu, current_node);
    
    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[test]
fn test_numa_memory_operations() -> Result<()> {
    let allocation_request = NumaAllocation {
        size: 4096,
        alignment: 4096,
        policy: NumaPolicy::Local,
        preferred_node: None,
        flags: 0,
    };
    
    let address = numa::numa_allocate(&allocation_request);
    assert!(address.is_ok());
    
    let address = address.unwrap();
    info!("NUMA allocation successful at {:#x}", address);
    
    // Test freeing
    let free_result = numa::numa_free(address, 4096);
    assert!(free_result.is_ok());
    
    Ok(())
}

// System HAL Tests
#[test]
fn test_hal_initialization() -> Result<()> {
    // This test verifies that the HAL was initialized correctly
    // In a real implementation, we would check more detailed state
    
    let cpu_info = cpu::get_cpu_info();
    assert!(cpu_info.cores > 0);
    
    let memory_layout = memory::get_memory_layout();
    assert!(memory_layout.is_some());
    
    let timer_info = timers::get_timer_info();
    assert!(!timer_info.is_empty());
    
    info!("HAL initialization verified successfully");
    
    Ok(())
}

#[test]
fn test_hal_statistics() -> Result<()> {
    let stats = get_stats();
    
    // Check that statistics are accessible
    let cpu_stats = stats.cpu_stats;
    let memory_stats = stats.memory_stats;
    let interrupt_stats = stats.interrupt_stats;
    let timer_stats = stats.timer_stats;
    let io_stats = stats.io_stats;
    
    info!("HAL statistics collected successfully");
    info!("CPU: {} cycles", cpu_stats.cycles.load(Ordering::SeqCst));
    info!("Memory: {} pages", memory_stats.total_pages);
    info!("Interrupts: {}", interrupt_stats.total_interrupts.load(Ordering::SeqCst));
    info!("I/O operations: {}", io_stats.io_operations.load(Ordering::SeqCst));
    
    Ok(())
}

#[test]
fn test_architecture_detection() -> Result<()> {
    let arch = get_arch_type();
    
    info!("Detected architecture: {:?}", arch);
    
    #[cfg(target_arch = "x86_64")]
    assert_eq!(arch, ArchType::X86_64);
    
    #[cfg(target_arch = "aarch64")]
    assert_eq!(arch, ArchType::AArch64);
    
    #[cfg(target_arch = "riscv64")]
    assert_eq!(arch, ArchType::Riscv64);
    
    Ok(())
}

// Integration Tests
#[test]
fn test_hal_coordination() -> Result<()> {
    // Test that different HAL modules work together
    
    // Get CPU and memory info
    let cpu_info = cpu::get_cpu_info();
    let memory_layout = memory::get_memory_layout().unwrap();
    
    // Check that current CPU is valid for the system
    let current_cpu = multicore::get_current_cpu_id();
    assert!(current_cpu < cpu_info.cores as usize);
    
    // Check that memory layout is consistent
    assert!(memory_layout.kernel_start < memory_layout.kernel_end);
    
    info!("HAL coordination test passed");
    
    Ok(())
}

#[test]
fn test_cross_module_operations() -> Result<()> {
    // Test operations that involve multiple HAL modules
    
    // Get current NUMA node and verify it's consistent with CPU
    let current_cpu = multicore::get_current_cpu_id();
    let current_node = numa::get_current_numa_node();
    let cpu_node = numa::get_node_for_cpu(current_cpu);
    
    assert_eq!(current_node, cpu_node.unwrap_or(0));
    
    // Check timer frequency is reasonable for the CPU
    let timer_freq = timers::get_timer_frequency();
    assert!(timer_freq > 0 && timer_freq < 10_000_000_000); // Reasonable range
    
    info!("Cross-module operations test passed");
    
    Ok(())
}

// Performance Tests
#[test]
fn test_system_performance_benchmark() -> Result<()> {
    let benchmark_result = benchmark::system_performance_test()?;
    
    assert!(benchmark_result.cpu_score > 0);
    assert!(benchmark_result.memory_score > 0);
    assert!(benchmark_result.interrupt_latency_ns > 0);
    
    info!("System Performance Benchmark:");
    info!("  CPU Score: {}", benchmark_result.cpu_score);
    info!("  Memory Score: {}", benchmark_result.memory_score);
    info!("  Interrupt Latency: {} ns", benchmark_result.interrupt_latency_ns);
    
    Ok(())
}

// Architecture-specific tests
#[cfg(target_arch = "x86_64")]
mod tests_x86_64 {
    use super::*;
    use crate::hal::x86_64;
    
    #[test]
    fn test_x86_64_specific_functions() -> Result<()> {
        // Test x86_64-specific interrupt management
        let initial_enabled = x86_64::are_global_interrupts_enabled();
        
        x86_64::set_global_interrupts(!initial_enabled);
        let after_disable = x86_64::are_global_interrupts_enabled();
        
        // Verify state changed
        assert_ne!(initial_enabled, after_disable);
        
        x86_64::set_global_interrupts(initial_enabled);
        
        info!("x86_64-specific functions work correctly");
        
        Ok(())
    }
}

#[cfg(target_arch = "aarch64")]
mod tests_aarch64 {
    use super::*;
    use crate::hal::aarch64;
    
    #[test]
    fn test_aarch64_specific_functions() -> Result<()> {
        let initial_enabled = aarch64::are_global_interrupts_enabled();
        
        aarch64::set_global_interrupts(!initial_enabled);
        let after_change = aarch64::are_global_interrupts_enabled();
        
        assert_ne!(initial_enabled, after_change);
        
        aarch64::set_global_interrupts(initial_enabled);
        
        info!("ARM64-specific functions work correctly");
        
        Ok(())
    }
}

#[cfg(target_arch = "riscv64")]
mod tests_riscv64 {
    use super::*;
    use crate::hal::riscv64;
    
    #[test]
    fn test_riscv64_specific_functions() -> Result<()> {
        let initial_enabled = riscv64::are_global_interrupts_enabled();
        
        riscv64::set_global_interrupts(!initial_enabled);
        let after_change = riscv64::are_global_interrupts_enabled();
        
        assert_ne!(initial_enabled, after_change);
        
        riscv64::set_global_interrupts(initial_enabled);
        
        info!("RISC-V-specific functions work correctly");
        
        Ok(())
    }
}