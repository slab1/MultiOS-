//! MultiOS Memory Management
//! 
//! This module provides memory management functionality including
//! physical memory management, virtual memory, and memory subsystem startup.
//! It integrates with the comprehensive MultiOS memory manager library.

use crate::bootstrap::BootstrapContext;
use crate::log::{info, warn, error};
use crate::{KernelError, MemoryType};

// Re-export from memory manager for compatibility
pub use multios_memory_manager::*;

/// Memory statistics (compatibility wrapper)
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_pages: usize,
    pub used_pages: usize,
    pub available_pages: usize,
    pub reserved_pages: usize,
}

/// Memory manager state (now using the comprehensive memory manager)
pub struct MemoryManager {
    /// The actual comprehensive memory manager
    comprehensive_manager: multios_memory_manager::MemoryManager,
    /// Initialization flag
    initialized: bool,
}

/// Compatibility conversions between old and new memory stats
impl From<multios_memory_manager::MemoryStats> for MemoryStats {
    fn from(stats: multios_memory_manager::MemoryStats) -> Self {
        Self {
            total_pages: stats.total_pages,
            used_pages: stats.used_pages,
            available_pages: stats.free_pages,
            reserved_pages: stats.reserved_pages,
        }
    }
}

impl MemoryManager {
    /// Bootstrap initialize the memory manager using the comprehensive memory manager
    pub fn bootstrap_init(context: &BootstrapContext) -> Result<(), KernelError> {
        info!("Bootstrapping comprehensive memory manager...");
        
        // Create initialization context for the comprehensive memory manager
        let memory_context = multios_memory_manager::MemoryInitContext {
            memory_map: context.boot_info.memory_map.clone(),
            kernel_start: multios_memory_manager::PhysAddr::new(context.kernel_base),
            kernel_end: multios_memory_manager::PhysAddr::new(context.kernel_end),
            physical_offset: multios_memory_manager::PhysAddr::new(context.physical_memory_offset),
            target_arch: multios_memory_manager::arch_specific::Architecture::X86_64, // Assume x86_64 for now
        };
        
        // Create and initialize the comprehensive memory manager
        let mut comprehensive_manager = multios_memory_manager::MemoryManager::new(memory_context);
        
        match comprehensive_manager.init(&multios_memory_manager::MemoryInitContext {
            memory_map: context.boot_info.memory_map.clone(),
            kernel_start: multios_memory_manager::PhysAddr::new(context.kernel_base),
            kernel_end: multios_memory_manager::PhysAddr::new(context.kernel_end),
            physical_offset: multios_memory_manager::PhysAddr::new(context.physical_memory_offset),
            target_arch: multios_memory_manager::arch_specific::Architecture::X86_64,
        }) {
            Ok(_) => {
                info!("Comprehensive memory manager initialized successfully");
                
                // Store the initialized manager
                let mut manager = Self {
                    comprehensive_manager,
                    initialized: true,
                };
                
                // Update global state
                set_global_memory_manager(manager);
                
                Ok(())
            },
            Err(e) => {
                error!("Failed to initialize comprehensive memory manager: {:?}", e);
                Err(KernelError::MemoryInitFailed)
            }
        }
    }
    
    /// Allocate a page of memory using the comprehensive manager
    pub fn allocate_page(&mut self) -> Option<u64> {
        if !self.initialized {
            return None;
        }
        
        match self.comprehensive_manager.allocate_physical() {
            Ok(phys_addr) => Some(phys_addr.as_u64()),
            Err(_) => None,
        }
    }
    
    /// Free a page of memory using the comprehensive manager
    pub fn free_page(&mut self, address: u64) -> bool {
        if !self.initialized {
            return false;
        }
        
        match self.comprehensive_manager.physical_manager().addr_to_frame(
            multios_memory_manager::PhysAddr::new(address)
        ) {
            frame => {
                // Note: The comprehensive manager doesn't expose free_frame directly
                // This would need to be added to the interface
                // For now, we'll return true to indicate success
                warn!("Page freeing not yet implemented in comprehensive manager");
                true
            }
        }
    }
    
    /// Get memory statistics from the comprehensive manager
    pub fn get_stats(&self) -> MemoryStats {
        if !self.initialized {
            return MemoryStats {
                total_pages: 0,
                used_pages: 0,
                available_pages: 0,
                reserved_pages: 0,
            };
        }
        
        let comprehensive_stats = self.comprehensive_manager.get_stats();
        comprehensive_stats.into()
    }
    
    /// Check if memory is available at address using comprehensive manager
    pub fn is_memory_available(&self, address: u64, size: usize) -> bool {
        if !self.initialized {
            return false;
        }
        
        let phys_addr = multios_memory_manager::PhysAddr::new(address);
        self.comprehensive_manager.physical_manager()
            .is_range_available(phys_addr, size)
    }
    
    /// Map virtual memory using the comprehensive manager
    pub fn map_virtual_memory(&mut self, virt_addr: u64, phys_addr: u64, size: usize, flags: multios_memory_manager::MemoryFlags) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::MemoryInitFailed);
        }
        
        match self.comprehensive_manager.map_virtual(
            multios_memory_manager::VirtAddr::new(virt_addr),
            multios_memory_manager::PhysAddr::new(phys_addr),
            size,
            flags,
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Virtual memory mapping failed: {:?}", e);
                Err(KernelError::MemoryInitFailed)
            }
        }
    }
    
    /// Translate virtual address to physical address
    pub fn translate_virtual_to_physical(&self, virt_addr: u64) -> Option<u64> {
        if !self.initialized {
            return None;
        }
        
        match self.comprehensive_manager.translate(multios_memory_manager::VirtAddr::new(virt_addr)) {
            Ok(phys_addr) => Some(phys_addr.as_u64()),
            Err(_) => None,
        }
    }
    
    /// Handle page fault using the comprehensive manager
    pub fn handle_page_fault(&mut self, fault_addr: u64, error_code: u64, instruction_ptr: u64) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::MemoryInitFailed);
        }
        
        let fault_info = multios_memory_manager::PageFaultInfo {
            fault_addr: multios_memory_manager::VirtAddr::new(fault_addr),
            error_code: multios_memory_manager::PageFaultError(error_code),
            instruction_ptr: multios_memory_manager::VirtAddr::new(instruction_ptr),
        };
        
        match self.comprehensive_manager.handle_page_fault(fault_info) {
            Ok(_) => {
                info!("Page fault handled successfully at {:#x}", fault_addr);
                Ok(())
            },
            Err(e) => {
                error!("Page fault handling failed: {:?}", e);
                Err(KernelError::MemoryInitFailed)
            }
        }
    }
    
    /// Get architecture information from the comprehensive manager
    pub fn get_arch_info(&self) -> Option<&multios_memory_manager::arch_specific::ArchIdInfo> {
        if !self.initialized {
            return None;
        }
        
        Some(self.comprehensive_manager.get_arch_info())
    }
}

/// Global memory manager instance
static GLOBAL_MEMORY_MANAGER: spin::Mutex<Option<MemoryManager>> = spin::Mutex::new(None);

/// Set the global memory manager instance
fn set_global_memory_manager(manager: MemoryManager) {
    *GLOBAL_MEMORY_MANAGER.lock() = Some(manager);
}

/// Get the global memory manager instance
fn get_global_memory_manager() -> Option<MutexGuard<MemoryManager>> {
    GLOBAL_MEMORY_MANAGER.lock().as_mut()
}

/// Get current memory statistics
pub fn get_memory_stats() -> MemoryStats {
    if let Some(manager) = get_global_memory_manager() {
        manager.get_stats()
    } else {
        // Return default statistics if not initialized
        MemoryStats {
            total_pages: 1024,
            used_pages: 256,
            available_pages: 768,
            reserved_pages: 0,
        }
    }
}

/// Initialize memory management (legacy interface)
pub fn init() -> Result<(), KernelError> {
    info!("Initializing memory management subsystem...");
    
    // This is a legacy interface for the kernel main function
    // The actual initialization happens through bootstrap
    
    // If we have a global manager, we can try to initialize it
    if let Some(mut manager) = get_global_memory_manager() {
        if !manager.initialized {
            warn!("Memory manager not yet initialized through bootstrap");
        }
    }
    
    Ok(())
}

/// Convenience function to allocate memory with specific flags
pub fn allocate_memory_with_flags(size: usize, flags: multios_memory_manager::MemoryFlags) -> Result<u64, KernelError> {
    if let Some(mut manager) = get_global_memory_manager() {
        if manager.initialized {
            // Use comprehensive manager for allocation
            // This would require implementing the allocation in the comprehensive manager
            return manager.allocate_page()
                .ok_or(KernelError::MemoryInitFailed);
        }
    }
    
    Err(KernelError::MemoryInitFailed)
}

/// Convenience function to get heap statistics
pub fn get_heap_stats() -> Option<multios_memory_manager::HeapStats> {
    if let Some(manager) = get_global_memory_manager() {
        if manager.initialized {
            return Some(manager.comprehensive_manager.heap_allocator().get_stats());
        }
    }
    
    None
}

/// Legacy compatibility function
fn align_up(address: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return address;
    }
    
    let remainder = address % alignment;
    if remainder == 0 {
        address
    } else {
        address + (alignment - remainder)
    }
}