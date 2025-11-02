//! MultiOS - Universal Educational Operating System
//! 
//! This is the main MultiOS crate providing the core operating system functionality.

#![no_std]

use log::info;

/// MultiOS main entry point
#[no_mangle]
pub extern "C" fn multios_main() {
    info!("MultiOS kernel starting...");
    
    // Initialize kernel
    match multios_kernel::kernel_main(multios_kernel::ArchType::X86_64, &multios_kernel::BootInfo {
        boot_time: 0,
        memory_map: Vec::new(),
        command_line: Some("multios"),
        modules: Vec::new(),
        framebuffer: None,
    }) {
        Ok(_) => info!("MultiOS initialized successfully"),
        Err(e) => {
            info!("MultiOS initialization failed: {:?}", e);
        }
    }
}
