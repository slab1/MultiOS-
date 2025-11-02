//! Virtual Machine Manager
//! 
//! Manages the lifecycle of virtual machines, including creation, configuration,
//! startup, shutdown, and resource allocation.

use crate::{VmConfig, VmInfo, VmId, HypervisorError, MAX_VCPUS_PER_VM};
use crate::vcpu::Vcpu;
use crate::memory::MemoryManager;

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::RwLock;
use bitflags::bitflags;

/// VM state enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmState {
    /// VM is not created
    NotCreated,
    /// VM is created but not started
    Created,
    /// VM is running
    Running,
    /// VM is paused
    Paused,
    /// VM is stopped
    Stopped,
    /// VM is in error state
    Error,
}

bitflags! {
    /// VM flags
    #[derive(Debug, Clone, Copy)]
    pub struct VmFlags: u32 {
        const AUTOSTART = 1 << 0;
        const NESTED = 1 << 1;
        const DEBUG = 1 << 2;
        const MONITORING = 1 << 3;
        const SNAPSHOT = 1 << 4;
    }
}

/// Virtual Machine information
#[derive(Debug, Clone)]
pub struct VmInfo {
    pub id: VmId,
    pub name: alloc::string::String,
    pub state: VmState,
    pub vcpu_count: usize,
    pub memory_mb: u64,
    pub flags: VmFlags,
    pub creation_time_ms: u64,
    pub uptime_ms: u64,
}

/// Virtual machine structure
#[derive(Debug)]
struct VirtualMachine {
    id: VmId,
    config: VmConfig,
    state: VmState,
    vcpus: Vec<Arc<RwLock<Vcpu>>>,
    memory_manager: Arc<RwLock<MemoryManager>>,
    flags: VmFlags,
    creation_time_ms: u64,
    uptime_ms: u64,
}

impl VirtualMachine {
    /// Create a new virtual machine
    fn new(id: VmId, config: VmConfig) -> Result<Self, HypervisorError> {
        let vcpu_count = config.vcpu_count.min(MAX_VCPUS_PER_VM);
        
        // Create VCPUs
        let mut vcpus = Vec::with_capacity(vcpu_count);
        for i in 0..vcpu_count {
            let vcpu = Arc::new(RwLock::new(Vcpu::new(id, i)?));
            vcpus.push(vcpu);
        }
        
        // Create memory manager
        let memory_manager = Arc::new(RwLock::new(MemoryManager::new(config.memory_mb)?));
        
        // Calculate creation time (simplified)
        let creation_time_ms = 0; // Would use actual timestamp
        
        Ok(VirtualMachine {
            id,
            config,
            state: VmState::Created,
            vcpus,
            memory_manager,
            flags: VmFlags::empty(),
            creation_time_ms,
            uptime_ms: 0,
        })
    }
    
    /// Start the VM
    fn start(&mut self) -> Result<(), HypervisorError> {
        match self.state {
            VmState::Created | VmState::Stopped => {
                // Initialize VCPUs
                for vcpu in &self.vcpus {
                    vcpu.write().initialize()?;
                }
                
                // Start VCPUs
                for vcpu in &self.vcpus {
                    vcpu.write().start()?;
                }
                
                self.state = VmState::Running;
                Ok(())
            },
            VmState::Running => Ok(()), // Already running
            VmState::Paused => {
                // Resume VCPUs
                for vcpu in &self.vcpus {
                    vcpu.write().resume()?;
                }
                
                self.state = VmState::Running;
                Ok(())
            },
            _ => Err(HypervisorError::InvalidVmState),
        }
    }
    
    /// Stop the VM
    fn stop(&mut self, force: bool) -> Result<(), HypervisorError> {
        match self.state {
            VmState::Running | VmState::Paused => {
                if force {
                    // Force stop - immediate termination
                    for vcpu in &self.vcpus {
                        vcpu.write().force_stop()?;
                    }
                } else {
                    // Graceful stop - signal shutdown
                    for vcpu in &self.vcpus {
                        vcpu.write().signal_shutdown()?;
                    }
                    
                    // Wait for graceful shutdown (simplified)
                    self.state = VmState::Stopped;
                    return Ok(());
                }
                
                self.state = VmState::Stopped;
                Ok(())
            },
            VmState::Stopped => Ok(()), // Already stopped
            _ => Err(HypervisorError::InvalidVmState),
        }
    }
    
    /// Pause the VM
    fn pause(&mut self) -> Result<(), HypervisorError> {
        match self.state {
            VmState::Running => {
                for vcpu in &self.vcpus {
                    vcpu.write().pause()?;
                }
                
                self.state = VmState::Paused;
                Ok(())
            },
            VmState::Paused => Ok(()), // Already paused
            _ => Err(HypervisorError::InvalidVmState),
        }
    }
    
    /// Resume the VM
    fn resume(&mut self) -> Result<(), HypervisorError> {
        match self.state {
            VmState::Paused => {
                for vcpu in &self.vcpus {
                    vcpu.write().resume()?;
                }
                
                self.state = VmState::Running;
                Ok(())
            },
            VmState::Running => Ok(()), // Already running
            _ => Err(HypervisorError::InvalidVmState),
        }
    }
    
    /// Get VM information
    fn get_info(&self) -> VmInfo {
        VmInfo {
            id: self.id,
            name: self.config.name.clone(),
            state: self.state,
            vcpu_count: self.vcpus.len(),
            memory_mb: self.config.memory_mb,
            flags: self.flags,
            creation_time_ms: self.creation_time_ms,
            uptime_ms: self.uptime_ms,
        }
    }
    
    /// Get VM statistics
    fn get_stats(&self) -> VmStats {
        VmStats {
            vcpu_stats: self.vcpus.iter().map(|v| v.read().get_stats()).collect(),
            memory_stats: self.memory_manager.read().get_stats(),
            total_uptime_ms: self.uptime_ms,
        }
    }
}

/// VM Statistics
#[derive(Debug, Clone)]
pub struct VmStats {
    pub vcpu_stats: Vec<CpuStats>,
    pub memory_stats: MemoryStats,
    pub total_uptime_ms: u64,
}

/// CPU Statistics
#[derive(Debug, Clone)]
pub struct CpuStats {
    pub vcpu_id: usize,
    pub total_time_ms: u64,
    pub vm_exit_count: u64,
    pub instruction_count: u64,
}

/// Memory Statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocated_mb: u64,
    pub used_mb: u64,
    pub page_faults: u64,
}

/// Virtual Machine Manager
pub struct VmManager {
    vms: BTreeMap<VmId, VirtualMachine>,
    next_vm_id: VmId,
}

impl VmManager {
    /// Create a new VM manager
    pub fn new() -> Result<Self, HypervisorError> {
        Ok(VmManager {
            vms: BTreeMap::new(),
            next_vm_id: VmId::new(1),
        })
    }
    
    /// Create a new virtual machine
    pub fn create_vm(&mut self, config: VmConfig) -> Result<VmId, HypervisorError> {
        let vm_id = self.next_vm_id;
        self.next_vm_id = VmId::new(vm_id.0 + 1);
        
        // Create the VM
        let vm = VirtualMachine::new(vm_id, config)?;
        self.vms.insert(vm_id, vm);
        
        Ok(vm_id)
    }
    
    /// Start a virtual machine
    pub fn start_vm(&mut self, vm_id: VmId) -> Result<(), HypervisorError> {
        let vm = self.vms.get_mut(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        vm.start()
    }
    
    /// Stop a virtual machine
    pub fn stop_vm(&mut self, vm_id: VmId, force: bool) -> Result<(), HypervisorError> {
        let vm = self.vms.get_mut(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        vm.stop(force)
    }
    
    /// Pause a virtual machine
    pub fn pause_vm(&mut self, vm_id: VmId) -> Result<(), HypervisorError> {
        let vm = self.vms.get_mut(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        vm.pause()
    }
    
    /// Resume a virtual machine
    pub fn resume_vm(&mut self, vm_id: VmId) -> Result<(), HypervisorError> {
        let vm = self.vms.get_mut(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        vm.resume()
    }
    
    /// Delete a virtual machine
    pub fn delete_vm(&mut self, vm_id: VmId) -> Result<(), HypervisorError> {
        // Ensure VM is stopped
        if let Some(vm) = self.vms.get(&vm_id) {
            if vm.state == VmState::Running {
                return Err(HypervisorError::CannotDeleteRunningVm);
            }
        }
        
        self.vms.remove(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        Ok(())
    }
    
    /// Get VM configuration
    pub fn get_vm_config(&self, vm_id: VmId) -> Result<VmConfig, HypervisorError> {
        let vm = self.vms.get(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        Ok(vm.config.clone())
    }
    
    /// Get VM information
    pub fn get_vm_info(&self, vm_id: VmId) -> Result<VmInfo, HypervisorError> {
        let vm = self.vms.get(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        Ok(vm.get_info())
    }
    
    /// Get VM statistics
    pub fn get_vm_stats(&self, vm_id: VmId) -> Result<VmStats, HypervisorError> {
        let vm = self.vms.get(&vm_id)
            .ok_or(HypervisorError::VmNotFound)?;
        
        Ok(vm.get_stats())
    }
    
    /// List all VMs
    pub fn list_vms(&self) -> Result<Vec<VmInfo>, HypervisorError> {
        let mut vm_list = Vec::new();
        
        for vm in self.vms.values() {
            vm_list.push(vm.get_info());
        }
        
        Ok(vm_list)
    }
    
    /// Get total VM count
    pub fn get_vm_count(&self) -> usize {
        self.vms.len()
    }
}