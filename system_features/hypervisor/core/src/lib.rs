//! MultiOS Type-2 Hypervisor Core
//! 
//! This module provides the core hypervisor functionality for running
//! nested operating systems and virtualization experiments.

#![no_std]
#![feature(asm)]

extern crate alloc;
extern crate spin;
extern crate bitflags;
extern crate log;

use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::boxed::Box;
use spin::RwLock;
use bitflags::bitflags;

mod vm_manager;
mod vcpu;
mod hypervisor;
mod vm_config;

pub use vm_manager::*;
pub use vcpu::*;
pub use hypervisor::*;
pub use vm_config::*;

/// Hypervisor version information
pub const HYPERVISOR_VERSION: &str = "1.0.0";
pub const HYPERVISOR_NAME: &str = "MultiOS Type-2 Hypervisor";

/// Maximum number of virtual machines supported
pub const MAX_VMS: usize = 64;

/// Maximum number of VCPUs per VM
pub const MAX_VCPUS_PER_VM: usize = 32;

/// Hypervisor capabilities flags
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct HypervisorCapabilities: u32 {
        const INTEL_VT_X = 1 << 0;
        const AMD_V = 1 << 1;
        const NESTED_PAGING = 1 << 2;
        const APIC_VIRTUALIZATION = 1 << 3;
        const IO_BITMAP = 1 << 4;
        const MSR_BITMAP = 1 << 5;
        const PV_ASYNC_INTR = 1 << 6;
        const EPT_VIOLATION = 1 << 7;
        const SINGLE_STEP = 1 << 8;
        const DEBUG_ASSIST = 1 << 9;
        const NESTED_VIRT = 1 << 10;
    }
}

/// Hypervisor architecture type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArchType {
    /// Intel x86_64 architecture
    X86_64,
    /// AMD x86_64 architecture
    AMD64,
    /// ARM64 architecture
    AArch64,
    /// Unknown/invalid architecture
    Unknown,
}

/// Main hypervisor state
static HYPERVISOR: RwLock<Option<Hypervisor>> = RwLock::new(None);

/// Initialize the hypervisor subsystem
pub fn initialize() -> Result<(), HypervisorError> {
    info!("Initializing MultiOS Hypervisor v{}", HYPERVISOR_VERSION);
    
    // Detect CPU virtualization support
    let capabilities = detect_cpu_capabilities();
    info!("CPU Virtualization Capabilities: {:?}", capabilities);
    
    // Create hypervisor instance
    let hypervisor = Hypervisor::new(capabilities)?;
    
    // Store in global state
    *HYPERVISOR.write() = Some(hypervisor);
    
    info!("Hypervisor initialized successfully");
    Ok(())
}

/// Get the global hypervisor instance
pub fn get_hypervisor() -> Option<Arc<RwLock<Hypervisor>>> {
    HYPERVISOR.read().as_ref().map(|h| Arc::new(RwLock::new(*h)))
}

/// Detect CPU virtualization capabilities
fn detect_cpu_capabilities() -> HypervisorCapabilities {
    let mut caps = HypervisorCapabilities::empty();
    
    #[cfg(target_arch = "x86_64")]
    {
        // Detect Intel VT-x and AMD-V support
        if is_intel_vtx_supported() {
            caps |= HypervisorCapabilities::INTEL_VT_X;
        }
        
        if is_amd_v_supported() {
            caps |= HypervisorCapabilities::AMD_V;
        }
        
        // Check for additional features
        if has_nested_paging() {
            caps |= HypervisorCapabilities::NESTED_PAGING;
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // ARM virtualization extensions
        caps |= HypervisorCapabilities::NESTED_PAGING;
    }
    
    caps
}

#[cfg(target_arch = "x86_64")]
fn is_intel_vtx_supported() -> bool {
    // Check CPUID.1:ECX.VMX[5] = 1
    unsafe {
        let ecx: u32;
        core::arch::asm!(
            "cpuid",
            in("eax") 1,
            out("ecx") ecx
        );
        (ecx & (1 << 5)) != 0
    }
}

#[cfg(target_arch = "x86_64")]
fn is_amd_v_supported() -> bool {
    // Check CPUID.0x1:ECX.SVM[2] = 1 or CPUID.8000_0001:ECX.SVM[2] = 1
    unsafe {
        let ecx: u32;
        core::arch::asm!(
            "cpuid",
            in("eax") 0x80000001,
            out("ecx") ecx
        );
        (ecx & (1 << 2)) != 0
    }
}

#[cfg(target_arch = "x86_64")]
fn has_nested_paging() -> bool {
    // Check IA32_VMX_PROCBASED_CTLS2 MSR bit 1 (SLAT)
    // This is a simplified check - real implementation would need MSR access
    true // Assume supported for now
}

#[cfg(target_arch = "aarch64")]
fn is_intel_vtx_supported() -> bool { false }

#[cfg(target_arch = "aarch64")]
fn is_amd_v_supported() -> bool { false }

#[cfg(target_arch = "aarch64")]
fn has_nested_paging() -> bool { true }