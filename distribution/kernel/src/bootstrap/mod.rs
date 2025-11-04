//! MultiOS Bootstrap and Early Initialization System
//! 
//! This module provides the kernel bootstrap sequence, early initialization,
//! and multi-architecture boot support for MultiOS.

pub mod early_init;
pub mod boot_sequence;
pub mod panic_handler;
pub mod error_handling;
pub mod arch_bootstrap;

#[cfg(test)]
pub mod test_suite;

use crate::{KernelError, ArchType, memory::MemoryManager};
use core::fmt::Write;

/// Bootstrap configuration and parameters
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    pub architecture: ArchType,
    pub boot_method: BootMethod,
    pub enable_debug: bool,
    pub enable_logging: bool,
    pub memory_test: bool,
    pub recovery_mode: bool,
}

/// Boot methods supported by MultiOS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BootMethod {
    Multiboot2 = 0,
    UEFI = 1,
    BIOS = 2,
    Direct = 3,
}

/// Bootstrap stages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BootstrapStage {
    EarlyInit = 0,
    MemoryInit = 1,
    InterruptInit = 2,
    ArchitectureInit = 3,
    DriverInit = 4,
    SchedulerInit = 5,
    UserModeInit = 6,
    Complete = 7,
}

/// Bootstrap context information
#[derive(Debug)]
pub struct BootstrapContext {
    pub current_stage: BootstrapStage,
    pub config: BootstrapConfig,
    pub boot_info: super::BootInfo,
    pub kernel_heap: Option<&'static mut [u8]>,
    pub stack_trace: Vec<BootstrapStage>,
    pub error_count: u32,
}

/// Early bootstrap result
pub type BootstrapResult<T> = Result<T, KernelError>;

/// Initialize the bootstrap system
pub fn init_bootstrap(config: BootstrapConfig, boot_info: super::BootInfo) -> BootstrapResult<BootstrapContext> {
    let mut context = BootstrapContext {
        current_stage: BootstrapStage::EarlyInit,
        config,
        boot_info,
        kernel_heap: None,
        stack_trace: Vec::new(),
        error_count: 0,
    };
    
    // Log bootstrap start
    crate::log::info!("=== MultiOS Bootstrap Starting ===");
    crate::log::info!("Architecture: {:?}", context.config.architecture);
    crate::log::info!("Boot Method: {:?}", context.config.boot_method);
    crate::log::info!("Debug Mode: {}", context.config.enable_debug);
    
    Ok(context)
}

/// Execute the complete bootstrap sequence
pub fn execute_bootstrap(mut context: BootstrapContext) -> BootstrapResult<()> {
    // Stage 1: Early initialization
    context.push_stage(BootstrapStage::EarlyInit);
    early_init::early_initialization(&mut context)?;
    
    // Stage 2: Memory initialization
    context.push_stage(BootstrapStage::MemoryInit);
    crate::log::info!("Initializing memory subsystem...");
    MemoryManager::bootstrap_init(&context)?;
    
    // Stage 3: Interrupt initialization
    context.push_stage(BootstrapStage::InterruptInit);
    arch_bootstrap::init_interrupts(&context)?;
    
    // Stage 4: Architecture-specific initialization
    context.push_stage(BootstrapStage::ArchitectureInit);
    arch_bootstrap::architecture_specific_init(&context)?;
    
    // Stage 5: Driver initialization
    context.push_stage(BootstrapStage::DriverInit);
    boot_sequence::init_core_drivers(&context)?;
    
    // Stage 6: Scheduler initialization
    context.push_stage(BootstrapStage::SchedulerInit);
    boot_sequence::init_scheduler(&context)?;
    
    // Stage 7: User mode initialization
    context.push_stage(BootstrapStage::UserModeInit);
    boot_sequence::init_user_mode(&context)?;
    
    // Complete bootstrap
    context.push_stage(BootstrapStage::Complete);
    crate::log::info!("=== Bootstrap Complete ===");
    
    Ok(())
}

/// Safe stage execution with error recovery
impl BootstrapContext {
    pub fn push_stage(&mut self, stage: BootstrapStage) {
        self.stack_trace.push(stage);
        self.current_stage = stage;
    }
    
    pub fn record_error(&mut self, error: KernelError) {
        self.error_count = self.error_count.checked_add(1).unwrap_or(u32::MAX);
        
        crate::log::error!("Bootstrap Error at stage {:?}: {:?}", stage, error);
        
        if self.config.recovery_mode && self.error_count < 3 {
            // Try to recover from the error
            self.recover_from_error(error);
        }
    }
    
    fn recover_from_error(&mut self, _error: KernelError) {
        crate::log::warn!("Attempting to recover from bootstrap error");
        
        // Simple recovery: skip the current stage and continue
        // In a real implementation, this would be more sophisticated
        if let Some(last_stage) = self.stack_trace.last() {
            let next_stage = match last_stage {
                BootstrapStage::EarlyInit => BootstrapStage::MemoryInit,
                BootstrapStage::MemoryInit => BootstrapStage::InterruptInit,
                BootstrapStage::InterruptInit => BootstrapStage::ArchitectureInit,
                BootstrapStage::ArchitectureInit => BootstrapStage::DriverInit,
                BootstrapStage::DriverInit => BootstrapStage::SchedulerInit,
                BootstrapStage::SchedulerInit => BootstrapStage::UserModeInit,
                BootstrapStage::UserModeInit => BootstrapStage::Complete,
                BootstrapStage::Complete => BootstrapStage::Complete,
            };
            
            self.current_stage = next_stage;
        }
    }
    
    pub fn is_recovery_mode(&self) -> bool {
        self.config.recovery_mode
    }
    
    pub fn get_error_count(&self) -> u32 {
        self.error_count
    }
}

/// Get boot time (used by system calls)
pub fn get_boot_time() -> u64 {
    // Return the boot time stored in kernel state
    // This would be implemented to return actual boot time
    1000000000 // Placeholder: boot time in nanoseconds
}

/// Get architecture-specific bootstrap information
pub fn get_arch_bootstrap_info() -> Option<ArchBootstrapInfo> {
    Some(ArchBootstrapInfo {
        page_size: 4096,
        stack_size: 4096 * 8,
        kernel_base: 0xFFFF_8000_0000_0000,
        kernel_end: 0xFFFF_8000_FFFF_FFFF,
    })
}

/// Architecture-specific bootstrap information
#[derive(Debug, Clone, Copy)]
pub struct ArchBootstrapInfo {
    pub page_size: usize,
    pub stack_size: usize,
    pub kernel_base: usize,
    pub kernel_end: usize,
}

/// Check if system is in debug mode
pub fn is_debug_mode() -> bool {
    // This would check debug flags or configuration
    true // Placeholder
}

/// Get current bootstrap stage
pub fn get_current_bootstrap_stage() -> BootstrapStage {
    // This would return the current stage of bootstrap
    BootstrapStage::Complete // Placeholder
}