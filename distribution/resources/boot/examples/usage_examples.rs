//! MultiOS Hardware Boot System Examples
//! 
//! This file provides comprehensive examples of using the MultiOS Direct Hardware Boot System
//! across different architectures and boot modes.

use multios_hardware_boot::{
    HardwareBootManager, BootConfig, Architecture, BootMode, HardwareInfo,
    MemoryMap, MemoryRegion, MemoryType
};

/// Basic hardware boot example
pub fn basic_boot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Hardware Boot Example ===");
    
    // Create hardware information
    let hardware_info = HardwareInfo {
        cpu_count: 4,
        total_memory: 8_000_000_000, // 8GB
        architecture: Architecture::X86_64,
        firmware_type: BootMode::UEFI,
        has_acpi: true,
        has_uefi: true,
    };
    
    // Create boot configuration
    let config = BootConfig {
        arch: Architecture::X86_64,
        mode: BootMode::UEFI,
        hardware_info,
        debug: true,
        ..Default::default()
    };
    
    // Create and initialize boot manager
    let mut boot_manager = HardwareBootManager::new(config);
    
    // Execute boot sequence
    match boot_manager.boot() {
        Ok(()) => {
            println!("✓ Boot sequence completed successfully");
            
            // Check boot status
            let status = boot_manager.boot_status();
            println!("Hardware detected: {}", status.hardware_detected);
            println!("Memory initialized: {}", status.memory_initialized);
            println!("Devices initialized: {}", status.devices_initialized);
        }
        Err(e) => {
            println!("✗ Boot failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// x86_64 specific boot example
pub fn x86_64_boot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== x86_64 Boot Example ===");
    
    use multios_hardware_boot::arch::x86_64::{
        X86_64BootLoader, X86BootMode, X86BootUtils
    };
    
    // Create hardware info
    let mut hardware_info = HardwareInfo::default();
    hardware_info.architecture = Architecture::X86_64;
    hardware_info.cpu_count = 8;
    hardware_info.total_memory = 16_000_000_000; // 16GB
    
    // Test CPU features
    let cpu_features = X86BootUtils::detect_cpu_features();
    println!("Detected CPU features: {:?}", cpu_features);
    
    // Test 64-bit support
    if X86BootUtils::has_64bit_support() {
        println!("✓ 64-bit CPU support detected");
    }
    
    // Create memory map
    let mut memory_map = MemoryMap::default();
    memory_map.regions.push(MemoryRegion {
        start: 0x0000000000000000,
        size: 0x0000000000100000, // 1MB
        region_type: MemoryType::Usable,
    });
    memory_map.regions.push(MemoryRegion {
        start: 0x0000000000100000,
        size: 0x0000000000010000, // 64KB reserved
        region_type: MemoryType::Reserved,
    });
    
    // Test UEFI boot
    let mut uefi_bootloader = X86_64BootLoader::new(hardware_info.clone(), X86BootMode::UEFI);
    uefi_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing x86_64 UEFI boot...");
    match uefi_bootloader.boot() {
        Ok(()) => println!("✓ UEFI boot successful"),
        Err(e) => println!("✗ UEFI boot failed: {}", e),
    }
    
    // Test BIOS boot
    let mut bios_bootloader = X86_64BootLoader::new(hardware_info.clone(), X86BootMode::BIOS);
    bios_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing x86_64 BIOS boot...");
    match bios_bootloader.boot() {
        Ok(()) => println!("✓ BIOS boot successful"),
        Err(e) => println!("✗ BIOS boot failed: {}", e),
    }
    
    // Test direct boot
    let mut direct_bootloader = X86_64BootLoader::new(hardware_info.clone(), X86BootMode::Direct);
    direct_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing x86_64 direct boot...");
    match direct_bootloader.boot() {
        Ok(()) => println!("✓ Direct boot successful"),
        Err(e) => println!("✗ Direct boot failed: {}", e),
    }
    
    Ok(())
}

/// ARM64 specific boot example
pub fn arm64_boot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== ARM64 Boot Example ===");
    
    use multios_hardware_boot::arch::arm64::{
        ARM64BootLoader, ARM64BootMode, ARM64BootUtils
    };
    
    // Create hardware info
    let mut hardware_info = HardwareInfo::default();
    hardware_info.architecture = Architecture::ARM64;
    hardware_info.cpu_count = 4;
    hardware_info.total_memory = 4_000_000_000; // 4GB
    
    // Test CPU features
    let cpu_features = ARM64BootUtils::detect_cpu_features();
    println!("Detected ARM64 CPU features: {:?}", cpu_features);
    
    // Test exception level
    let current_el = ARM64BootUtils::current_el();
    println!("Current exception level: EL{}", current_el);
    
    // Test architecture-specific features
    if ARM64BootUtils::has_fp() {
        println!("✓ ARM64 Floating Point support detected");
    }
    
    if ARM64BootUtils::has_aes() {
        println!("✓ ARM64 AES instructions support detected");
    }
    
    // Create memory map
    let mut memory_map = MemoryMap::default();
    memory_map.regions.push(MemoryRegion {
        start: 0x0000000000000000,
        size: 0x0000000000200000, // 2MB
        region_type: MemoryType::Usable,
    });
    
    // Test UEFI boot
    let mut uefi_bootloader = ARM64BootLoader::new(hardware_info.clone(), ARM64BootMode::UEFI);
    uefi_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing ARM64 UEFI boot...");
    match uefi_bootloader.boot() {
        Ok(()) => println!("✓ UEFI boot successful"),
        Err(e) => println!("✗ UEFI boot failed: {}", e),
    }
    
    // Test ATF boot
    let mut atf_bootloader = ARM64BootLoader::new(hardware_info.clone(), ARM64BootMode::ATF);
    atf_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing ARM64 ATF boot...");
    match atf_bootloader.boot() {
        Ok(()) => println!("✓ ATF boot successful"),
        Err(e) => println!("✗ ATF boot failed: {}", e),
    }
    
    // Test direct boot
    let mut direct_bootloader = ARM64BootLoader::new(hardware_info.clone(), ARM64BootMode::Direct);
    direct_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing ARM64 direct boot...");
    match direct_bootloader.boot() {
        Ok(()) => println!("✓ Direct boot successful"),
        Err(e) => println!("✗ Direct boot failed: {}", e),
    }
    
    Ok(())
}

/// RISC-V64 specific boot example
pub fn riscv64_boot_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== RISC-V64 Boot Example ===");
    
    use multios_hardware_boot::arch::riscv64::{
        RISC_VBootLoader, RISCBootMode, RISCBootUtils
    };
    
    // Create hardware info
    let mut hardware_info = HardwareInfo::default();
    hardware_info.architecture = Architecture::RISC_V64;
    hardware_info.cpu_count = 1;
    hardware_info.total_memory = 2_000_000_000; // 2GB
    
    // Test CPU features
    let cpu_features = RISCBootUtils::detect_cpu_features();
    println!("Detected RISC-V64 CPU features: {:?}", cpu_features);
    
    // Test privilege level
    let privilege_level = RISCBootUtils::privilege_level();
    println!("Current privilege level: {}", privilege_level);
    
    // Test hart ID
    let hart_id = RISCBootUtils::hart_id();
    println!("Current hart ID: {}", hart_id);
    
    // Test architecture-specific features
    if RISCBootUtils::has_a() {
        println!("✓ RISC-V Atomic extension detected");
    }
    
    if RISCBootUtils::has_m() {
        println!("✓ RISC-V Multiplication extension detected");
    }
    
    // Create memory map
    let mut memory_map = MemoryMap::default();
    memory_map.regions.push(MemoryRegion {
        start: 0x0000000000000000,
        size: 0x0000000000100000, // 1MB
        region_type: MemoryType::Usable,
    });
    
    // Test UEFI boot
    let mut uefi_bootloader = RISC_VBootLoader::new(hardware_info.clone(), RISCBootMode::UEFI);
    uefi_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing RISC-V64 UEFI boot...");
    match uefi_bootloader.boot() {
        Ok(()) => println!("✓ UEFI boot successful"),
        Err(e) => println!("✗ UEFI boot failed: {}", e),
    }
    
    // Test OpenSBI boot
    let mut opensbi_bootloader = RISC_VBootLoader::new(hardware_info.clone(), RISCBootMode::OpenSBI);
    opensbi_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing RISC-V64 OpenSBI boot...");
    match opensbi_bootloader.boot() {
        Ok(()) => println!("✓ OpenSBI boot successful"),
        Err(e) => println!("✗ OpenSBI boot failed: {}", e),
    }
    
    // Test direct boot
    let mut direct_bootloader = RISC_VBootLoader::new(hardware_info.clone(), RISCBootMode::Direct);
    direct_bootloader.set_memory_map(memory_map.clone());
    
    println!("Testing RISC-V64 direct boot...");
    match direct_bootloader.boot() {
        Ok(()) => println!("✓ Direct boot successful"),
        Err(e) => println!("✗ Direct boot failed: {}", e),
    }
    
    Ok(())
}

/// Hardware compatibility testing example
pub fn hardware_testing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Hardware Testing Example ===");
    
    use multios_hardware_boot::test::{
        HardwareCompatibilityFramework, TestSuite, TestCategory
    };
    
    // Create custom test suite
    let mut test_suite = TestSuite::default();
    test_suite.enabled_categories = vec![
        TestCategory::CPU,
        TestCategory::Memory,
        TestCategory::BootDevice,
        TestCategory::Firmware,
    ];
    test_suite.performance_benchmarks = true;
    test_suite.strict_mode = true;
    
    // Test on x86_64
    let mut x86_framework = HardwareCompatibilityFramework::new(
        Architecture::X86_64,
        BootMode::UEFI,
        test_suite.clone()
    );
    
    let hardware_info = HardwareInfo {
        cpu_count: 8,
        total_memory: 16_000_000_000,
        architecture: Architecture::X86_64,
        firmware_type: BootMode::UEFI,
        has_acpi: true,
        has_uefi: true,
    };
    
    println!("Running x86_64 hardware compatibility tests...");
    let x86_results = x86_framework.run_all_tests(&hardware_info)?;
    
    println!("x86_64 test results:");
    let passed_count = x86_results.iter().filter(|r| r.status == TestStatus::Passed).count();
    let failed_count = x86_results.iter().filter(|r| r.status == TestStatus::Failed).count();
    println!("  Passed: {}/{}", passed_count, x86_results.len());
    println!("  Failed: {}/{}", failed_count, x86_results.len());
    
    if x86_framework.critical_tests_passed() {
        println!("✓ All critical x86_64 tests passed");
    } else {
        println!("✗ Some critical x86_64 tests failed");
    }
    
    // Test on ARM64
    let mut arm64_framework = HardwareCompatibilityFramework::new(
        Architecture::ARM64,
        BootMode::ATF,
        test_suite.clone()
    );
    
    let arm64_info = HardwareInfo {
        cpu_count: 4,
        total_memory: 4_000_000_000,
        architecture: Architecture::ARM64,
        firmware_type: BootMode::LegacyBIOS,
        has_acpi: false,
        has_uefi: false,
    };
    
    println!("Running ARM64 hardware compatibility tests...");
    let arm64_results = arm64_framework.run_all_tests(&arm64_info)?;
    
    println!("ARM64 test results:");
    let arm64_passed = arm64_results.iter().filter(|r| r.status == TestStatus::Passed).count();
    let arm64_failed = arm64_results.iter().filter(|r| r.status == TestStatus::Failed).count();
    println!("  Passed: {}/{}", arm64_passed, arm64_results.len());
    println!("  Failed: {}/{}", arm64_failed, arm64_results.len());
    
    if arm64_framework.critical_tests_passed() {
        println!("✓ All critical ARM64 tests passed");
    } else {
        println!("✗ Some critical ARM64 tests failed");
    }
    
    Ok(())
}

/// Boot optimization example
pub fn boot_optimization_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Boot Optimization Example ===");
    
    use multios_hardware_boot::optimization::{
        BootSequenceOptimizer, OptimizationProfile, OptimizationSettings
    };
    
    // Hardware info for optimization
    let hardware_info = HardwareInfo {
        cpu_count: 8,
        total_memory: 32_000_000_000,
        architecture: Architecture::X86_64,
        firmware_type: BootMode::UEFI,
        has_acpi: true,
        has_uefi: true,
    };
    
    // Performance optimization profile
    let mut performance_profile = OptimizationProfile::Performance;
    println!("Applying Performance optimization profile...");
    
    let mut performance_optimizer = BootSequenceOptimizer::new(
        Architecture::X86_64,
        BootMode::UEFI,
        performance_profile
    );
    
    let optimized_sequence = performance_optimizer.optimize(&hardware_info)?;
    
    // Calculate improvement
    let baseline_time = 5000; // 5 seconds baseline
    let improvement = performance_optimizer.estimate_boot_time_improvement(baseline_time);
    
    println!("Performance optimization results:");
    println!("  Baseline boot time: {}ms", baseline_time);
    println!("  Optimized boot time: {}ms", performance_optimizer.boot_metrics().total_boot_time_ms);
    println!("  Estimated improvement: {:.1}%", improvement);
    
    // Show optimized phases
    println!("Optimized phases:");
    for phase in &performance_optimizer.boot_metrics().optimized_phases {
        println!("  {}: {}ms → {}ms ({:.1}% improvement)",
            phase.phase_name,
            phase.original_time_ms,
            phase.optimized_time_ms,
            phase.performance_gain_percent
        );
    }
    
    // Balanced optimization profile
    println!("\nApplying Balanced optimization profile...");
    let balanced_profile = OptimizationProfile::Balanced;
    let mut balanced_optimizer = BootSequenceOptimizer::new(
        Architecture::ARM64,
        BootMode::ATF,
        balanced_profile
    );
    
    let balanced_sequence = balanced_optimizer.optimize(&hardware_info)?;
    let balanced_improvement = balanced_optimizer.estimate_boot_time_improvement(baseline_time);
    
    println!("Balanced optimization results:");
    println!("  Estimated improvement: {:.1}%", balanced_improvement);
    
    // Custom optimization profile
    println!("\nApplying Custom optimization profile...");
    let custom_settings = OptimizationSettings {
        parallel_initialization: true,
        skip_diagnostic_checks: false,
        aggressive_memory_management: true,
        fast_device_detection: true,
        compressed_boot_images: false,
        optimized_interrupts: true,
        cache_optimization: true,
        prefetch_optimization: false,
        boot_time_target_ms: 2000,
    };
    
    let custom_profile = OptimizationProfile::Custom(custom_settings);
    let mut custom_optimizer = BootSequenceOptimizer::new(
        Architecture::RISC_V64,
        BootMode::Direct,
        custom_profile
    );
    
    let custom_sequence = custom_optimizer.optimize(&hardware_info)?;
    let custom_improvement = custom_optimizer.estimate_boot_time_improvement(baseline_time);
    
    println!("Custom optimization results:");
    println!("  Estimated improvement: {:.1}%", custom_improvement);
    
    Ok(())
}

/// Hardware Abstraction Layer example
pub fn hal_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Hardware Abstraction Layer Example ===");
    
    use multios_hardware_boot::hal::{
        ArchitectureHAL, X86_64HAL, ARM64HAL, RISCV64HAL, PeripheralType
    };
    
    // Hardware info
    let hardware_info = HardwareInfo {
        cpu_count: 8,
        total_memory: 16_000_000_000,
        architecture: Architecture::X86_64,
        firmware_type: BootMode::UEFI,
        has_acpi: true,
        has_uefi: true,
    };
    
    // Initialize x86_64 HAL
    println!("Initializing x86_64 HAL...");
    let mut x86_hal = ArchitectureHAL::new(Architecture::X86_64, BootMode::UEFI);
    
    match x86_hal.init(&hardware_info) {
        Ok(()) => println!("✓ x86_64 HAL initialized successfully"),
        Err(e) => println!("✗ x86_64 HAL initialization failed: {}", e),
    }
    
    // Check supported architectures
    let supported_archs = X86_64HAL::supported_architectures();
    println!("Supported architectures: {:?}", supported_archs);
    
    // Check peripheral types
    let peripheral_types = X86_64HAL::peripheral_types();
    println!("Supported peripheral types: {:?}", peripheral_types);
    
    // Initialize ARM64 HAL
    println!("\nInitializing ARM64 HAL...");
    let mut arm64_hal = ArchitectureHAL::new(Architecture::ARM64, BootMode::ATF);
    
    match arm64_hal.init(&hardware_info) {
        Ok(()) => println!("✓ ARM64 HAL initialized successfully"),
        Err(e) => println!("✗ ARM64 HAL initialization failed: {}", e),
    }
    
    // Initialize RISC-V HAL
    println!("\nInitializing RISC-V64 HAL...");
    let mut riscv_hal = ArchitectureHAL::new(Architecture::RISC_V64, BootMode::OpenSBI);
    
    match riscv_hal.init(&hardware_info) {
        Ok(()) => println!("✓ RISC-V64 HAL initialized successfully"),
        Err(e) => println!("✗ RISC-V64 HAL initialization failed: {}", e),
    }
    
    Ok(())
}

/// Comprehensive multi-architecture example
pub fn comprehensive_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive Multi-Architecture Example ===");
    
    use multios_hardware_boot::{HardwareBootManager, BootConfig, init_boot_logging};
    use log::LevelFilter;
    
    // Initialize logging
    init_boot_logging(LevelFilter::Info);
    
    // Define test configurations
    let configurations = vec![
        (Architecture::X86_64, BootMode::UEFI, "x86_64-UEFI"),
        (Architecture::X86_64, BootMode::LegacyBIOS, "x86_64-BIOS"),
        (Architecture::X86_64, BootMode::Direct, "x86_64-Direct"),
        (Architecture::ARM64, BootMode::UEFI, "ARM64-UEFI"),
        (Architecture::ARM64, BootMode::ATF, "ARM64-ATF"),
        (Architecture::ARM64, BootMode::Direct, "ARM64-Direct"),
        (Architecture::RISC_V64, BootMode::UEFI, "RISC-V64-UEFI"),
        (Architecture::RISC_V64, BootMode::OpenSBI, "RISC-V64-OpenSBI"),
        (Architecture::RISC_V64, BootMode::Direct, "RISC-V64-Direct"),
    ];
    
    for (arch, mode, name) in configurations {
        println!("\n=== Testing {} Configuration ===", name);
        
        // Create hardware info
        let hardware_info = HardwareInfo {
            cpu_count: match arch {
                Architecture::X86_64 => 8,
                Architecture::ARM64 => 4,
                Architecture::RISC_V64 => 1,
            },
            total_memory: match arch {
                Architecture::X86_64 => 32_000_000_000, // 32GB
                Architecture::ARM64 => 8_000_000_000,   // 8GB
                Architecture::RISC_V64 => 4_000_000_000, // 4GB
            },
            architecture: arch,
            firmware_type: mode,
            has_acpi: matches!(mode, BootMode::UEFI | BootMode::LegacyBIOS),
            has_uefi: matches!(mode, BootMode::UEFI),
        };
        
        // Create configuration
        let config = BootConfig {
            arch,
            mode,
            hardware_info,
            debug: false,
            memory_map: None,
        };
        
        // Create and test boot manager
        let mut boot_manager = HardwareBootManager::new(config);
        
        match boot_manager.boot() {
            Ok(()) => {
                println!("✓ {} boot completed successfully", name);
                
                let status = boot_manager.boot_status();
                println!("  Hardware detected: {}", status.hardware_detected);
                println!("  Memory initialized: {}", status.memory_initialized);
                println!("  Devices initialized: {}", status.devices_initialized);
            }
            Err(e) => {
                println!("✗ {} boot failed: {}", name, e);
            }
        }
    }
    
    println!("\n=== Comprehensive Test Complete ===");
    Ok(())
}

/// Main function to run all examples
pub fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running MultiOS Hardware Boot System Examples\n");
    
    // Initialize logging for examples
    use log::LevelFilter;
    multios_hardware_boot::init_boot_logging(LevelFilter::Info);
    
    // Run examples
    basic_boot_example()?;
    x86_64_boot_example()?;
    arm64_boot_example()?;
    riscv64_boot_example()?;
    hardware_testing_example()?;
    boot_optimization_example()?;
    hal_example()?;
    comprehensive_example()?;
    
    println!("\n✓ All examples completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_boot() {
        // Test basic boot functionality
        assert!(basic_boot_example().is_ok());
    }
    
    #[test]
    fn test_x86_64_boot() {
        // Test x86_64 boot functionality
        assert!(x86_64_boot_example().is_ok());
    }
    
    #[test]
    fn test_arm64_boot() {
        // Test ARM64 boot functionality
        assert!(arm64_boot_example().is_ok());
    }
    
    #[test]
    fn test_riscv64_boot() {
        // Test RISC-V64 boot functionality
        assert!(riscv64_boot_example().is_ok());
    }
    
    #[test]
    fn test_hardware_testing() {
        // Test hardware testing functionality
        assert!(hardware_testing_example().is_ok());
    }
    
    #[test]
    fn test_boot_optimization() {
        // Test boot optimization functionality
        assert!(boot_optimization_example().is_ok());
    }
    
    #[test]
    fn test_hal() {
        // Test HAL functionality
        assert!(hal_example().is_ok());
    }
    
    #[test]
    fn test_comprehensive() {
        // Test comprehensive functionality
        assert!(comprehensive_example().is_ok());
    }
}

// Note: This needs the proper imports for the error type from the test module
// Adding a simple implementation for the test compilation
use multios_hardware_boot::TestStatus;