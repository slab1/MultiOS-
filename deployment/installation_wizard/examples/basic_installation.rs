//! Basic installation example for MultiOS installer

use multios_installation_wizard::{
    core::{InstallationConfig, InstallationWizard},
    hardware::HardwareDetector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    env_logger::init();
    
    println!("Starting MultiOS Basic Installation Example");
    
    // Detect hardware
    println!("Detecting hardware...");
    let hardware_info = HardwareDetector::detect_all().await?;
    
    println!("Detected Hardware:");
    println!("- CPU: {} {} ({})", 
        hardware_info.cpu.vendor, 
        hardware_info.cpu.model,
        hardware_info.cpu.architecture);
    println!("- Memory: {:.1} GB", 
        hardware_info.memory.total_bytes as f64 / 1e9);
    println!("- Storage: {} devices", 
        hardware_info.storage.devices.len());
    println!("- Graphics: {}", 
        hardware_info.graphics.gpu_vendor);
    
    // Create installation configuration
    let config = InstallationConfig::minimal();
    
    println!("Installation Configuration:");
    println!("- Target: {:?}", config.target);
    println!("- Boot Type: {:?}", config.boot_type);
    println!("- Username: {}", config.username);
    println!("- Dry Run: {}", config.dry_run);
    
    // Create and run installation wizard
    let mut wizard = InstallationWizard::new(config, hardware_info);
    
    println!("Starting installation wizard (text mode)...");
    wizard.run_text_mode().await?;
    
    println!("Installation completed successfully!");
    
    Ok(())
}