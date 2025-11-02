//! Nested Virtualization Support
//! 
//! Provides support for running virtual machines inside virtual machines,
//! enabling OS research and nested virtualization experiments.

use crate::{VmId, HypervisorError, VmConfig, VmFeatures, HypervisorCapabilities};
use crate::core::{VmState, Vcpu, VcpuStateType, VmExitReason};
use crate::cpu::{CpuVirtualization, VmcsRegion, VmcbRegion, SvmExitCode};
use crate::memory::{MemoryManager, VirtualizationType, EptPageTable, NptPageTable};

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use bitflags::bitflags;

/// Nested virtualization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NestingLevel {
    /// No nesting (L0 - host)
    Level0,
    /// First level guest (L1 - guest)
    Level1,
    /// Second level guest (L2 - nested guest)
    Level2,
    /// Third level guest (L3 - deeply nested)
    Level3,
}

/// Nested VM information
#[derive(Debug, Clone)]
pub struct NestedVmInfo {
    pub vm_id: VmId,
    pub nesting_level: NestingLevel,
    pub parent_vm_id: Option<VmId>,
    pub child_vms: Vec<VmId>,
    pub virtualization_type: VirtualizationType,
    pub enabled_features: NestedFeatures,
    pub performance_metrics: NestedPerformanceMetrics,
}

/// Nested virtualization features
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct NestedFeatures: u32 {
        const RECURSIVE_VT_X = 1 << 0;
        const RECURSIVE_AMD_V = 1 << 1;
        const NESTED_EPT = 1 << 2;
        const NESTED_NPT = 1 << 3;
        const VIRTUAL_APIC = 1 << 4;
        const VMCS_SHADOWING = 1 << 5;
        const MSR_BITMAPS = 1 << 6;
        const IO_BITMAPS = 1 << 7;
        const NESTED_INTERRUPTS = 1 << 8;
        const DEBUG_ASSIST = 1 << 9;
        const PERFORMANCE_MONITORING = 1 << 10;
        const SNAPSHOT_NESTING = 1 << 11;
    }
}

/// Nested VM state for virtualization
#[derive(Debug, Clone, Copy)]
pub struct NestedVmState {
    /// VMCS pointer for nested VM
    pub vmcs_pointer: Option<VmcsRegion>,
    /// VMCB pointer for nested VM
    pub vmcb_pointer: Option<VmcbRegion>,
    /// Nested page table
    pub nested_page_table: Option<NestingPageTable>,
    /// Virtualization state
    pub virt_state: VirtualizationState,
    /// Performance counters
    pub perf_counters: PerformanceCounters,
}

/// Virtualization state management
#[derive(Debug, Clone, Copy)]
pub struct VirtualizationState {
    /// Current VM state
    pub current_state: VcpuStateType,
    /// Nested VM state
    pub nested_state: NestedState,
    /// VM exit handling state
    pub exit_state: ExitState,
    /// Nested virtualization enabled
    pub nested_enabled: bool,
}

/// Nested state information
#[derive(Debug, Clone, Copy)]
pub struct NestedState {
    /// Guest VMCS/VMCB state
    pub guest_state: VmState,
    /// Host state
    pub host_state: VmState,
    /// Control state
    pub control_state: VmState,
    /// Nested execution level
    pub exec_level: NestingLevel,
}

/// VM exit state
#[derive(Debug, Clone, Copy)]
pub struct ExitState {
    /// VM exit reason
    pub exit_reason: Option<VmExitReason>,
    /// Exit qualification
    pub qualification: u64,
    /// Exit instruction length
    pub instruction_length: u32,
    /// Exit instruction pointer
    pub instruction_pointer: u64,
}

/// Performance counters for nested virtualization
#[derive(Debug, Clone)]
pub struct PerformanceCounters {
    pub nested_vm_exits: u64,
    pub shadow_vmcs_accesses: u64,
    pub nested_page_faults: u64,
    pub virtualization_overhead_ns: u64,
    pub nested_instruction_count: u64,
    pub performance_degradation_percent: f32,
}

/// Nested performance metrics
#[derive(Debug, Clone)]
pub struct NestedPerformanceMetrics {
    /// Average overhead per instruction
    pub avg_instruction_overhead_ns: u64,
    /// Memory access overhead
    pub memory_overhead_ns: u64,
    /// I/O virtualization overhead
    pub io_overhead_ns: u64,
    /// Context switch overhead
    pub context_switch_overhead_ns: u64,
    /// Total virtualization overhead
    pub total_overhead_ns: u64,
    /// Nested virtualization efficiency (0.0 to 1.0)
    pub efficiency: f32,
}

/// Nested page table structure
#[derive(Debug)]
pub struct NestingPageTable {
    /// L0 page table (host physical)
    pub l0_page_table: Box<[PageEntry]>,
    /// L1 page table (guest physical to host physical)
    pub l1_page_table: Box<[PageEntry]>,
    /// L2 page table (guest virtual to guest physical)
    pub l2_page_table: Box<[PageEntry]>,
    /// Page table levels
    pub levels: u8,
}

/// Page entry for nested paging
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageEntry {
    pub present: bool,
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub user: bool,
    pub accessed: bool,
    pub dirty: bool,
    pub address: u64,
    pub reserved: u64,
}

/// Nested Virtualization Manager
pub struct NestedVirtualizationManager {
    /// Nested VMs
    nested_vms: BTreeMap<VmId, NestedVmInfo>,
    /// Parent-child relationships
    parent_child_map: BTreeMap<VmId, Vec<VmId>>,
    /// Virtualization capabilities
    capabilities: HypervisorCapabilities,
    /// Manager statistics
    stats: NestedStats,
}

impl NestedVirtualizationManager {
    /// Create a new nested virtualization manager
    pub fn new(capabilities: HypervisorCapabilities) -> Self {
        NestedVirtualizationManager {
            nested_vms: BTreeMap::new(),
            parent_child_map: BTreeMap::new(),
            capabilities,
            stats: NestedStats::default(),
        }
    }
    
    /// Enable nested virtualization for a VM
    pub fn enable_nested_virtualization(&mut self, vm_id: VmId, config: &VmConfig) -> Result<(), HypervisorError> {
        if !self.capabilities.contains(HypervisorCapabilities::NESTED_VIRT) {
            return Err(HypervisorError::FeatureNotSupported);
        }
        
        if !config.features.contains(VmFeatures::NESTED) {
            return Err(HypervisorError::ConfigurationError(String::from("Nested virtualization not enabled in VM config")));
        }
        
        // Determine nesting level
        let nesting_level = self.determine_nesting_level(vm_id)?;
        
        // Create nested VM info
        let nested_features = self.determine_nested_features(config);
        let virt_type = self.determine_virtualization_type(nesting_level);
        
        let nested_vm = NestedVmInfo {
            vm_id,
            nesting_level,
            parent_vm_id: self.find_parent_vm(vm_id),
            child_vms: Vec::new(),
            virtualization_type: virt_type,
            enabled_features: nested_features,
            performance_metrics: NestedPerformanceMetrics::default(),
        };
        
        self.nested_vms.insert(vm_id, nested_vm);
        
        // Update parent-child relationships
        if let Some(parent_id) = self.find_parent_vm(vm_id) {
            self.parent_child_map.entry(parent_id)
                .or_insert_with(Vec::new)
                .push(vm_id);
        }
        
        // Configure nested virtualization
        self.configure_nested_features(vm_id, nested_features)?;
        
        info!("Enabled nested virtualization for VM {} at level {:?}", vm_id.0, nesting_level);
        Ok(())
    }
    
    /// Disable nested virtualization for a VM
    pub fn disable_nested_virtualization(&mut self, vm_id: VmId) -> Result<(), HypervisorError> {
        if let Some(nested_vm) = self.nested_vms.remove(&vm_id) {
            // Remove from parent-child relationships
            if let Some(parent_id) = nested_vm.parent_vm_id {
                if let Some(children) = self.parent_child_map.get_mut(&parent_id) {
                    children.retain(|&child| child != vm_id);
                }
            }
            
            // Clean up child VMs if any
            for child_id in &nested_vm.child_vms {
                self.disable_nested_virtualization(*child_id)?;
            }
            
            info!("Disabled nested virtualization for VM {}", vm_id.0);
            Ok(())
        } else {
            Err(HypervisorError::VmNotFound)
        }
    }
    
    /// Determine nesting level for a VM
    fn determine_nesting_level(&self, vm_id: VmId) -> Result<NestingLevel, HypervisorError> {
        // Count nesting levels up to this VM
        let mut level = 0;
        let mut current_vm_id = vm_id;
        
        while let Some(parent_id) = self.find_parent_vm(current_vm_id) {
            level += 1;
            current_vm_id = parent_id;
            
            if level >= 3 {
                return Err(HypervisorError::ConfigurationError(String::from("Maximum nesting level reached")));
            }
        }
        
        match level {
            0 => Ok(NestingLevel::Level0),
            1 => Ok(NestingLevel::Level1),
            2 => Ok(NestingLevel::Level2),
            3 => Ok(NestingLevel::Level3),
            _ => Err(HypervisorError::ConfigurationError(String::from("Invalid nesting level"))),
        }
    }
    
    /// Find parent VM for a given VM
    fn find_parent_vm(&self, vm_id: VmId) -> Option<VmId> {
        // Simplified parent finding - would use actual VM hierarchy
        if vm_id.0 > 0 {
            Some(VmId(vm_id.0 - 1))
        } else {
            None
        }
    }
    
    /// Determine nested features based on VM config
    fn determine_nested_features(&self, config: &VmConfig) -> NestedFeatures {
        let mut features = NestedFeatures::empty();
        
        if self.capabilities.contains(HypervisorCapabilities::INTEL_VT_X) {
            features |= NestedFeatures::RECURSIVE_VT_X;
        }
        
        if self.capabilities.contains(HypervisorCapabilities::AMD_V) {
            features |= NestedFeatures::RECURSIVE_AMD_V;
        }
        
        if self.capabilities.contains(HypervisorCapabilities::NESTED_PAGING) {
            features |= NestedFeatures::NESTED_EPT | NestedFeatures::NESTED_NPT;
        }
        
        if config.features.contains(VmFeatures::DEBUG) {
            features |= NestedFeatures::DEBUG_ASSIST;
        }
        
        if config.features.contains(VmFeatures::RESOURCE_MONITORING) {
            features |= NestedFeatures::PERFORMANCE_MONITORING;
        }
        
        if config.features.contains(VmFeatures::SNAPSHOT_SUPPORT) {
            features |= NestedFeatures::SNAPSHOT_NESTING;
        }
        
        features
    }
    
    /// Determine virtualization type for nesting level
    fn determine_virtualization_type(&self, level: NestingLevel) -> VirtualizationType {
        match level {
            NestingLevel::Level0 => {
                if self.capabilities.contains(HypervisorCapabilities::INTEL_VT_X) {
                    VirtualizationType::IntelVTx
                } else if self.capabilities.contains(HypervisorCapabilities::AMD_V) {
                    VirtualizationType::AMDV
                } else {
                    VirtualizationType::Unknown
                }
            },
            _ => {
                // Nested guests use the same virtualization type as parent
                VirtualizationType::IntelVTx // Simplified
            },
        }
    }
    
    /// Configure nested features for a VM
    fn configure_nested_features(&mut self, vm_id: VmId, features: NestedFeatures) -> Result<(), HypervisorError> {
        if let Some(nested_vm) = self.nested_vms.get_mut(&vm_id) {
            // Configure recursive VMCS/VMCB
            if features.contains(NestedFeatures::RECURSIVE_VT_X) {
                self.configure_nested_vmcs(vm_id)?;
            }
            
            // Configure nested paging
            if features.contains(NestedFeatures::NESTED_EPT) {
                self.configure_nested_ept(vm_id)?;
            }
            
            // Configure VMCS shadowing
            if features.contains(NestedFeatures::VMCS_SHADOWING) {
                self.configure_vmcs_shadowing(vm_id)?;
            }
            
            // Configure MSR bitmaps
            if features.contains(NestedFeatures::MSR_BITMAPS) {
                self.configure_msr_bitmaps(vm_id)?;
            }
            
            info!("Configured nested features for VM {}: {:?}", vm_id.0, features);
            Ok(())
        } else {
            Err(HypervisorError::VmNotFound)
        }
    }
    
    /// Configure nested VMCS
    fn configure_nested_vmcs(&self, vm_id: VmId) -> Result<(), HypervisorError> {
        // Create nested VMCS for guest-to-host transitions
        // This would involve:
        // 1. Creating shadow VMCS structure
        // 2. Configuring VMCS field mappings
        // 3. Setting up nested control fields
        
        info!("Configured nested VMCS for VM {}", vm_id.0);
        Ok(())
    }
    
    /// Configure nested EPT
    fn configure_nested_ept(&self, vm_id: VmId) -> Result<(), HypervisorError> {
        // Configure Extended Page Tables for nested guests
        // This would involve:
        // 1. Creating nested EPT page tables
        // 2. Setting up EPTP pointers
        // 3. Configuring nested paging control fields
        
        info!("Configured nested EPT for VM {}", vm_id.0);
        Ok(())
    }
    
    /// Configure VMCS shadowing
    fn configure_vmcs_shadowing(&self, vm_id: VmId) -> Result<(), HypervisorError> {
        // Configure VMCS shadowing for performance optimization
        // This would involve:
        // 1. Setting up shadow VMCS structures
        // 2. Configuring shadow VMCS enable bit
        // 3. Setting up shadow VMCS pointer
        
        info!("Configured VMCS shadowing for VM {}", vm_id.0);
        Ok(())
    }
    
    /// Configure MSR bitmaps
    fn configure_msr_bitmaps(&self, vm_id: VmId) -> Result<(), HypervisorError> {
        // Configure MSR bitmaps for nested MSR access
        // This would involve:
        // 1. Creating MSR bitmap structures
        // 2. Configuring bitmap pointers
        // 3. Setting up MSR intercept rules
        
        info!("Configured MSR bitmaps for VM {}", vm_id.0);
        Ok(())
    }
    
    /// Handle nested VM exit
    pub fn handle_nested_vm_exit(&mut self, vm_id: VmId, exit_reason: VmExitReason) -> Result<(), HypervisorError> {
        if let Some(nested_vm) = self.nested_vms.get_mut(&vm_id) {
            nested_vm.performance_metrics.total_overhead_ns += self.calculate_exit_overhead(exit_reason);
            
            match exit_reason {
                VmExitReason::EPTViolation => {
                    self.handle_nested_ept_violation(vm_id)?;
                },
                VmExitReason::MsrRead => {
                    self.handle_nested_msr_access(vm_id, true)?;
                },
                VmExitReason::MsrWrite => {
                    self.handle_nested_msr_access(vm_id, false)?;
                },
                _ => {
                    // Handle other nested exits
                    self.handle_generic_nested_exit(vm_id, exit_reason)?;
                },
            }
            
            self.stats.total_nested_exits += 1;
            info!("Handled nested VM exit {:?} for VM {}", exit_reason, vm_id.0);
            Ok(())
        } else {
            Err(HypervisorError::VmNotFound)
        }
    }
    
    /// Handle nested EPT violation
    fn handle_nested_ept_violation(&self, vm_id: VmId) -> Result<(), HypervisorError> {
        // Handle EPT violation in nested guest
        // This would involve:
        // 1. Identifying the offending guest address
        // 2. Walking the nested page tables
        // 3. Allocating missing pages
        // 4. Updating nested EPT entries
        
        self.stats.nested_page_faults += 1;
        info!("Handled nested EPT violation for VM {}", vm_id.0);
        Ok(())
    }
    
    /// Handle nested MSR access
    fn handle_nested_msr_access(&self, vm_id: VmId, is_read: bool) -> Result<(), HypervisorError> {
        // Handle MSR read/write in nested guest
        // This would involve:
        // 1. Checking MSR intercept bitmap
        // 2. Reading/writing MSR values
        // 3. Handling nested MSR virtualization
        
        info!("Handled nested MSR {} for VM {}", if is_read { "read" } else { "write" }, vm_id.0);
        Ok(())
    }
    
    /// Handle generic nested exit
    fn handle_generic_nested_exit(&self, vm_id: VmId, exit_reason: VmExitReason) -> Result<(), HypervisorError> {
        // Handle other types of nested VM exits
        info!("Handled generic nested exit {:?} for VM {}", exit_reason, vm_id.0);
        Ok(())
    }
    
    /// Calculate overhead for VM exit
    fn calculate_exit_overhead(&self, exit_reason: VmExitReason) -> u64 {
        // Simulate overhead calculation based on exit type
        match exit_reason {
            VmExitReason::EPTViolation => 1000,    // 1µs
            VmExitReason::MsrRead | VmExitReason::MsrWrite => 500, // 0.5µs
            VmExitReason::CpuidInstruction => 300, // 0.3µs
            _ => 100, // 0.1µs
        }
    }
    
    /// Get nested VM information
    pub fn get_nested_vm_info(&self, vm_id: VmId) -> Option<&NestedVmInfo> {
        self.nested_vms.get(&vm_id)
    }
    
    /// List all nested VMs
    pub fn list_nested_vms(&self) -> Vec<&VmId> {
        self.nested_vms.keys().collect()
    }
    
    /// Get nested statistics
    pub fn get_nested_stats(&self) -> &NestedStats {
        &self.stats
    }
    
    /// Generate nested virtualization report
    pub fn generate_nested_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Nested Virtualization Report\n");
        report.push_str("============================\n\n");
        
        report.push_str(&format!("Total nested VMs: {}\n", self.nested_vms.len()));
        report.push_str(&format!("Total nested exits: {}\n", self.stats.total_nested_exits));
        report.push_str(&format!("Nested page faults: {}\n", self.stats.nested_page_faults));
        report.push_str(&format!("Total overhead: {} ns\n", self.stats.total_overhead_ns));
        report.push_str(&format!("Average overhead: {} ns\n", 
                               if self.stats.total_nested_exits > 0 {
                                   self.stats.total_overhead_ns / self.stats.total_nested_exits
                               } else { 0 }));
        
        report.push_str("\nNested VM Details:\n");
        for (vm_id, nested_vm) in &self.nested_vms {
            report.push_str(&format!("  VM {}: Level {:?}, Parent: {:?}, Children: {}\n",
                                  vm_id.0, nested_vm.nesting_level, 
                                  nested_vm.parent_vm_id, nested_vm.child_vms.len()));
        }
        
        report.push_str("\nHierarchy:\n");
        for (parent_id, children) in &self.parent_child_map {
            report.push_str(&format!("  VM {} -> {:?}\n", parent_id.0, children));
        }
        
        report
    }
    
    /// Get maximum supported nesting level
    pub fn get_max_nesting_level(&self) -> NestingLevel {
        if self.capabilities.contains(HypervisorCapabilities::NESTED_VIRT) {
            if self.capabilities.contains(HypervisorCapabilities::INTEL_VT_X) {
                // Intel VT-x typically supports up to 3 levels
                NestingLevel::Level3
            } else {
                // AMD-V might have different limits
                NestingLevel::Level2
            }
        } else {
            NestingLevel::Level0
        }
    }
}

/// Nested virtualization statistics
#[derive(Debug, Clone, Default)]
pub struct NestedStats {
    pub total_nested_vms: u64,
    pub total_nested_exits: u64,
    pub nested_page_faults: u64,
    pub total_overhead_ns: u64,
    pub max_nesting_level: u8,
}

impl NestedPerformanceMetrics {
    /// Create default nested performance metrics
    fn default() -> Self {
        NestedPerformanceMetrics {
            avg_instruction_overhead_ns: 0,
            memory_overhead_ns: 0,
            io_overhead_ns: 0,
            context_switch_overhead_ns: 0,
            total_overhead_ns: 0,
            efficiency: 1.0,
        }
    }
}

/// Default implementation for NestingPageTable
impl Default for NestingPageTable {
    fn default() -> Self {
        NestingPageTable {
            l0_page_table: vec![PageEntry::default(); 512].into_boxed_slice(),
            l1_page_table: vec![PageEntry::default(); 512].into_boxed_slice(),
            l2_page_table: vec![PageEntry::default(); 512].into_boxed_slice(),
            levels: 3,
        }
    }
}

/// Default implementation for PageEntry
impl Default for PageEntry {
    fn default() -> Self {
        PageEntry {
            present: false,
            read: false,
            write: false,
            execute: false,
            user: false,
            accessed: false,
            dirty: false,
            address: 0,
            reserved: 0,
        }
    }
}