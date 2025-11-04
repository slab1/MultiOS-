//! Hardware detection example for MultiOS installer

use multios_installation_wizard::hardware::HardwareDetector;
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    env_logger::init();
    
    println!("Starting MultiOS Hardware Detection Example");
    
    // Detect all hardware
    println!("Detecting system hardware...");
    let hardware_info = HardwareDetector::detect_all().await?;
    
    // Display hardware summary
    println!("\n=== HARDWARE DETECTION RESULTS ===\n");
    
    // CPU Information
    println!("CPU Information:");
    println!("  Architecture: {}", hardware_info.cpu.architecture);
    println!("  Vendor: {}", hardware_info.cpu.vendor);
    println!("  Model: {}", hardware_info.cpu.model);
    println!("  Cores: {} ({} threads)", 
        hardware_info.cpu.core_count, 
        hardware_info.cpu.thread_count);
    println!("  Frequency: {} MHz", hardware_info.cpu.frequency_mhz);
    
    if !hardware_info.cpu.features.is_empty() {
        println!("  Features: {}", hardware_info.cpu.features.join(", "));
    }
    
    // Memory Information
    println!("\nMemory Information:");
    println!("  Total: {:.1} GB", hardware_info.memory.total_bytes as f64 / 1e9);
    println!("  Available: {:.1} GB", hardware_info.memory.available_bytes as f64 / 1e9);
    println!("  Modules: {}", hardware_info.memory.module_count);
    println!("  Type: {}", hardware_info.memory.memory_type);
    println!("  Speed: {} MHz", hardware_info.memory.speed_mhz);
    println!("  ECC: {}", if hardware_info.memory.ecc_enabled { "Yes" } else { "No" });
    
    // Storage Information
    println!("\nStorage Devices:");
    for (index, device) in hardware_info.storage.devices.iter().enumerate() {
        println!("  Device {}: {}", index + 1, device.device_name);
        println!("    Type: {}", device.device_type);
        println!("    Capacity: {:.1} GB", device.capacity as f64 / 1e9);
        println!("    Interface: {}", device.interface);
        println!("    Model: {}", device.model);
        println!("    Rotational: {}", if device.is_rotational { "Yes" } else { "No (SSD)" });
        println!("    Removable: {}", if device.is_removable { "Yes" } else { "No" });
    }
    
    println!("  Total Storage: {:.1} GB", 
        hardware_info.storage.total_capacity as f64 / 1e9);
    
    // Network Information
    println!("\nNetwork Interfaces:");
    for interface in &hardware_info.network.devices {
        println!("  Interface: {}", interface.interface_name);
        println!("    Type: {}", interface.device_type);
        println!("    MAC: {}", interface.mac_address);
        println!("    Speed: {} Mbps", interface.speed_mbps);
        println!("    State: {}", interface.state);
        println!("    Driver: {}", interface.driver);
    }
    
    // Graphics Information
    println!("\nGraphics Devices:");
    for (index, device) in hardware_info.graphics.devices.iter().enumerate() {
        println!("  GPU {}: {}", index + 1, device.device_name);
        println!("    Vendor: {}", device.vendor);
        println!("    Model: {}", device.model);
        println!("    Driver: {}", device.driver);
        println!("    Memory: {} MB", device.memory_mb);
        println!("    Max Resolution: {}x{}", 
            device.max_resolution.0, device.max_resolution.1);
    }
    
    // Boot Information
    println!("\nBoot System:");
    println!("  Boot Type: {}", hardware_info.boot.boot_type);
    println!("  Boot Loader: {}", hardware_info.boot.boot_loader);
    println!("  Firmware: {}", hardware_info.boot.firmware_vendor);
    println!("  Secure Boot: {}", if hardware_info.boot.secure_boot { "Enabled" } else { "Disabled" });
    println!("  Fast Boot: {}", if hardware_info.boot.fast_boot { "Enabled" } else { "Disabled" });
    
    // Audio Information
    println!("\nAudio Devices:");
    for device in &hardware_info.audio.devices {
        println!("  Device: {}", device.device_name);
        println!("    Driver: {}", device.driver);
        println!("    Channels: {}", device.channels);
        if !device.supported_formats.is_empty() {
            println!("    Formats: {}", device.supported_formats.join(", "));
        }
        if !device.sample_rates.is_empty() {
            println!("    Sample Rates: {} Hz", 
                device.sample_rates.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", "));
        }
    }
    
    // Input Devices
    println!("\nInput Devices:");
    println!("  Keyboards: {}", hardware_info.input.keyboards.len());
    println!("  Mice: {}", hardware_info.input.mice.len());
    println!("  Touchpads: {}", hardware_info.input.touchpads.len());
    println!("  Touchscreens: {}", hardware_info.input.touchscreens.len());
    
    // Save hardware info to JSON file
    let json_output = serde_json::to_string_pretty(&hardware_info)?;
    std::fs::write("hardware_detection_report.json", json_output)?;
    println!("\nHardware information saved to 'hardware_detection_report.json'");
    
    // Generate compatibility report
    println!("\n=== COMPATIBILITY CHECK ===");
    
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();
    
    // Check memory requirements
    let min_memory = 2 * 1024 * 1024 * 1024; // 2GB
    if hardware_info.memory.total_bytes < min_memory {
        issues.push(format!("Insufficient memory: {:.1} GB minimum required", min_memory as f64 / 1e9));
        recommendations.push("Install more RAM before installing MultiOS".to_string());
    } else {
        println!("✓ Memory requirements met");
    }
    
    // Check CPU architecture
    let compatible_architectures = ["x86_64", "ARM64", "RISC-V"];
    if !compatible_architectures.contains(&hardware_info.cpu.architecture.as_str()) {
        issues.push(format!("Unsupported CPU architecture: {}", hardware_info.cpu.architecture));
        recommendations.push("MultiOS may not run properly on this architecture".to_string());
    } else {
        println!("✓ CPU architecture supported");
    }
    
    // Check storage space
    let min_storage = 10 * 1024 * 1024 * 1024; // 10GB
    if hardware_info.storage.total_capacity < min_storage {
        issues.push(format!("Insufficient storage: {:.1} GB minimum required", min_storage as f64 / 1e9));
        recommendations.push("Free up disk space or use a larger disk".to_string());
    } else {
        println!("✓ Storage requirements met");
    }
    
    // Check for graphics drivers
    let gpu_vendor = &hardware_info.graphics.gpu_vendor;
    if !gpu_vendor.contains("NVIDIA") && 
       !gpu_vendor.contains("AMD") && 
       !gpu_vendor.contains("Intel") {
        recommendations.push(format!("Consider installing graphics drivers for {}", gpu_vendor));
    } else {
        println!("✓ Graphics hardware detected");
    }
    
    // Display issues and recommendations
    if !issues.is_empty() {
        println!("\n❌ ISSUES FOUND:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }
    
    if !recommendations.is_empty() {
        println!("\n💡 RECOMMENDATIONS:");
        for recommendation in recommendations {
            println!("  - {}", recommendation);
        }
    }
    
    if issues.is_empty() {
        println!("\n✅ No critical issues found. System is ready for MultiOS installation!");
    }
    
    println!("\nHardware detection completed successfully!");
    
    Ok(())
}