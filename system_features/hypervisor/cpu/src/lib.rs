//! CPU Virtualization Extensions
//! 
//! Implements support for Intel VT-x and AMD-V hardware virtualization extensions,
//! providing the core mechanisms for efficient virtual machine execution.

use crate::{HypervisorCapabilities, HypervisorError, VmId, VcpuId};
use crate::core::{VmExitReason, VcpuState, VcpuRegs, VcpuCtrlRegs};

use bitflags::bitflags;
use alloc::vec::Vec;

/// VMCS field definitions for Intel VT-x
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmcsField {
    // Control fields
    VmcsLinkPointer = 0x2800,
    ExecutiveVmcsPointer = 0x2801,
    PinBasedVmExecutionControls = 0x4000,
    PrimaryProcessorBasedVmExecutionControls = 0x4002,
    SecondaryProcessorBasedVmExecutionControls = 0x4003,
    TprThreshold = 0x4004,
    NmiWindowExiting = 0x4008,
    VirtualProcessorIdentifier = 0x4000,
    PostedInterruptVector = 0x4002,
    TprShadow = 0x4004,
    NmiPromptExiting = 0x4008,
    
    // Exit controls
    PrimaryVmExitControls = 0x400C,
    SecondaryVmExitControls = 0x400E,
   VmFunctionControls = 0x4010,
    ExceptionBitmap = 0x4014,
    PageFaultErrorCodeMask = 0x4018,
    PageFaultErrorCodeMatch = 0x401C,
    CpuBasedVmExecutionControls = 0x4010,
    
    // Entry controls
    PrimaryVmEntryControls = 0x4012,
    SecondaryVmEntryControls = 0x4014,
    VmEntryMsrLoadCount = 0x4016,
    VmEntryMsrLoadAddress = 0x4018,
    VmEntryInterruptInfo = 0x4016,
    VmEntryExceptionErrorCode = 0x4018,
    VmEntryInstructionLength = 0x401A,
    VmEntryInstructionPointer = 0x401C,
    
    // Exit info
    VmExitReason = 0x4400,
    VmExitQualification = 0x4402,
    IoInstructionInfo = 0x4406,
    VmExitInstructionInfo = 0x4404,
    VmExitInstructionLength = 0x4408,
    VmExitInstructionPointer = 0x440A,
    
    // Guest state
    GuestEsSelector = 0x0808,
    GuestCsSelector = 0x080a,
    GuestSsSelector = 0x080c,
    GuestDsSelector = 0x080e,
    GuestFsSelector = 0x0810,
    GuestGsSelector = 0x0812,
    GuestLdtrSelector = 0x0814,
    GuestTrSelector = 0x0816,
    GuestInterruptStatus = 0x0818,
    
    // Guest control registers
    GuestCr0 = 0x6800,
    GuestCr3 = 0x6802,
    GuestCr4 = 0x6804,
    GuestEsBase = 0x6806,
    GuestCsBase = 0x6808,
    GuestSsBase = 0x680A,
    GuestDsBase = 0x680C,
    GuestFsBase = 0x680E,
    GuestGsBase = 0x6810,
    GuestLdtrBase = 0x6812,
    GuestTrBase = 0x6814,
    GuestGdtrBase = 0x6816,
    GuestIdtrBase = 0x6818,
    GuestDr7 = 0x681A,
    GuestRsp = 0x681C,
    GuestRip = 0x681E,
    GuestRflags = 0x6820,
    
    // Host state
    HostEsSelector = 0x0C00,
    HostCsSelector = 0x0C02,
    HostSsSelector = 0x0C04,
    HostDsSelector = 0x0C06,
    HostFsSelector = 0x0C08,
    HostGsSelector = 0x0C0A,
    HostTrSelector = 0x0C0C,
    HostIa32Pat = 0x2B00,
    HostIa32Efer = 0x2B02,
    HostCr0 = 0x6C00,
    HostCr3 = 0x6C02,
    HostCr4 = 0x6C04,
    HostFsBase = 0x6C06,
    HostGsBase = 0x6C08,
    HostTrBase = 0x6C0A,
    HostGdtrBase = 0x6C0C,
    HostIdtrBase = 0x6C0E,
    HostIa32SysenterCs = 0x6C10,
    HostIa32SysenterEsp = 0x6C12,
    HostIa32SysenterEip = 0x6C14,
    HostRsp = 0x6C16,
    HostRip = 0x6C18,
}

/// VMCS control bits for Intel VT-x
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct VmcsControls: u32 {
        const INTERRUPT_WINDOW = 1 << 2;
        const NMI_WINDOW = 1 << 3;
        const VIRTUAL_NMI = 1 << 5;
        const ENABLE_VM_FUNCTIONS = 1 << 13;
        const ENABLE_EPT = 1 << 18;
        const ENABLE_VPID = 1 << 19;
        const ENABLE_PML = 1 << 21;
        const ENABLE_UNRESTRICTED_GUEST = 1 << 7;
        const ENABLE_XSAVES = 1 << 20;
        const ENABLE_RDRAND = 1 << 24;
        const ENABLE_RDSEED = 1 << 25;
        const ENABLE_PCOMMIT = 1 << 26;
    }
}

/// VMCS pin-based execution controls
bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct VmcsPinControls: u32 {
        const EXTERNAL_INTERRUPT = 1 << 0;
        const NMI = 1 << 3;
        const VIRTUAL_NMIS = 1 << 5;
        const PREEMPT_TIMER = 1 << 6;
        const POSTED_INTERRUPTS = 1 << 7;
    }
}

/// AMD-V SVM control block structure
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct VmcB {
    pub vmcb_tlb_control: u8,
    pub reserved_0: [u8; 3],
    pub intercept_cr_read: [u16; 4],
    pub intercept_cr_write: [u16; 4],
    pub intercept_exceptions: u32,
    pub intercept: u64,
    pub iopm_base_pa: u64,
    pub msrpm_base_pa: u64,
    pub tsc_offset: u64,
    pub guest_asid: u32,
    pub tlb_control: u8,
    pub reserved_1: [u8; 3],
    pub guest_virq: u8,
    pub reserved_2: [u8; 3],
    pub v_tpr: u8,
    pub reserved_3: [u8; 3],
    pub event_injection: u32,
    pub npt_enable: u32,
    pub reserved_4: u32,
    pub exit_code: u64,
    pub exit_info_1: u64,
    pub exit_info_2: u64,
    pub exit_int_info: u32,
    pub exit_int_info_err: u32,
    pub np_enable: u32,
    pub available_0: u64,
    pub available_1: u64,
    pub available_2: u64,
    pub available_3: u64,
    pub available_4: u64,
    pub available_5: u64,
    pub available_6: u64,
    pub available_7: u64,
    pub available_8: u64,
    pub available_9: u64,
    pub available_10: u64,
    pub available_11: u64,
    pub available_12: u64,
    pub available_13: u64,
    pub available_14: u64,
}

/// SVM exit codes
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum SvmExitCode {
    Invalid = 0,
    VmExit = 1,
    InitiateVmlaunch = 3,
    VmResume = 4,
    VmRun = 5,
    Shutdown = 6,
    EnterSMM = 7,
    ExitSMM = 8,
    Pause = 9,
    Hlt = 10,
    Instructions = 11,
    Cr0Read = 12,
    Cr3Read = 13,
    Cr4Read = 14,
    Cr8Read = 15,
    Dr0Read = 16,
    Dr1Read = 17,
    Dr2Read = 18,
    Dr3Read = 19,
    Dr4Read = 20,
    Dr5Read = 21,
    Dr6Read = 22,
    Dr7Read = 23,
    Dr8Read = 24,
    Dr9Read = 25,
    Dr10Read = 26,
    Dr11Read = 27,
    Dr12Read = 28,
    Dr13Read = 29,
    Dr14Read = 30,
    Dr15Read = 31,
    Cr0Write = 32,
    Cr3Write = 33,
    Cr4Write = 34,
    Cr8Write = 35,
    Dr0Write = 36,
    Dr1Write = 37,
    Dr2Write = 38,
    Dr3Write = 39,
    Dr4Write = 40,
    Dr5Write = 41,
    Dr6Write = 42,
    Dr7Write = 43,
    Dr8Write = 44,
    Dr9Write = 45,
    Dr10Write = 46,
    Dr11Write = 47,
    Dr12Write = 48,
    Dr13Write = 49,
    Dr14Write = 50,
    Dr15Write = 51,
    Excp0 = 52,
    Excp1 = 53,
    Excp2 = 54,
    Excp3 = 55,
    Excp4 = 56,
    Excp5 = 57,
    Excp6 = 58,
    Excp7 = 59,
    Excp8 = 60,
    Excp9 = 61,
    Excp10 = 62,
    Excp11 = 63,
    Excp12 = 64,
    Excp13 = 65,
    Excp14 = 66,
    Excp15 = 67,
    Excp16 = 68,
    Excp17 = 69,
    Excp18 = 70,
    Excp19 = 71,
    Excp20 = 72,
    Excp21 = 73,
    Excp22 = 74,
    Excp23 = 75,
    Excp24 = 76,
    Excp25 = 77,
    Excp26 = 78,
    Excp27 = 79,
    Excp28 = 80,
    Excp29 = 81,
    Excp30 = 82,
    Excp31 = 83,
    Intr = 84,
    Nmi = 85,
    Smi = 86,
    Init = 87,
    Vintr = 88,
    Cr0WriteTwitch = 89,
    IdtVectoring = 90,
    Unknown,
}

/// CPU Virtualization Extension Manager
pub struct CpuVirtualization {
    /// Hardware capabilities
    capabilities: HypervisorCapabilities,
    /// VMCS regions for Intel VT-x
    vmcs_regions: Vec<VmcsRegion>,
    /// VMCB regions for AMD-V
    vmcb_regions: Vec<VmcbRegion>,
    /// Active VMCS pointers for each VCPU
    active_vmcs: Vec<VmcsPointer>,
    /// Active VMCB pointers for each VCPU
    active_vmcb: Vec<VmcbPointer>,
}

impl CpuVirtualization {
    /// Create a new CPU virtualization manager
    pub fn new(capabilities: HypervisorCapabilities) -> Result<Self, HypervisorError> {
        let manager = CpuVirtualization {
            capabilities,
            vmcs_regions: Vec::new(),
            vmcb_regions: Vec::new(),
            active_vmcs: Vec::new(),
            active_vmcb: Vec::new(),
        };
        
        info!("CPU Virtualization Manager created with capabilities: {:?}", capabilities);
        Ok(manager)
    }
    
    /// Get hardware capabilities
    pub fn get_capabilities(&self) -> HypervisorCapabilities {
        self.capabilities
    }
    
    /// Check if Intel VT-x is available
    pub fn is_intel_vtx_supported(&self) -> bool {
        self.capabilities.contains(HypervisorCapabilities::INTEL_VT_X)
    }
    
    /// Check if AMD-V is available
    pub fn is_amd_v_supported(&self) -> bool {
        self.capabilities.contains(HypervisorCapabilities::AMD_V)
    }
    
    /// Create VMCS for a VCPU (Intel VT-x)
    pub fn create_vmcs(&mut self, vm_id: VmId, vcpu_id: VcpuId) -> Result<VmcsRegion, HypervisorError> {
        if !self.is_intel_vtx_supported() {
            return Err(HypervisorError::HardwareVirtNotAvailable);
        }
        
        // Allocate and initialize VMCS region
        let vmcs_region = VmcsRegion::new(vm_id, vcpu_id)?;
        self.vmcs_regions.push(vmcs_region);
        
        // Setup VMCS configuration
        self.setup_vmcs(&self.vmcs_regions.last().unwrap())?;
        
        Ok(*self.vmcs_regions.last().unwrap())
    }
    
    /// Create VMCB for a VCPU (AMD-V)
    pub fn create_vmcb(&mut self, vm_id: VmId, vcpu_id: VcpuId) -> Result<VmcbRegion, HypervisorError> {
        if !self.is_amd_v_supported() {
            return Err(HypervisorError::HardwareVirtNotAvailable);
        }
        
        // Allocate and initialize VMCB region
        let vmcb_region = VmcbRegion::new(vm_id, vcpu_id)?;
        self.vmcb_regions.push(vmcb_region);
        
        // Setup VMCB configuration
        self.setup_vmcb(&self.vmcb_regions.last().unwrap())?;
        
        Ok(*self.vmcb_regions.last().unwrap())
    }
    
    /// Launch VMCS (Intel VT-x)
    pub fn vmcs_launch(&mut self, vmcs_region: VmcsRegion) -> Result<(), HypervisorError> {
        self.setup_vmcs(&vmcs_region)?;
        
        // Execute VMLAUNCH instruction
        unsafe {
            core::arch::asm!(
                "vmcs_launch",
                in("rcx") vmcs_region.get_address() as u64
            );
        }
        
        Ok(())
    }
    
    /// Resume VMCS (Intel VT-x)
    pub fn vmcs_resume(&mut self, vmcs_region: VmcsRegion) -> Result<(), HypervisorError> {
        unsafe {
            core::arch::asm!(
                "vmcs_resume",
                in("rcx") vmcs_region.get_address() as u64
            );
        }
        
        Ok(())
    }
    
    /// Get VMCS exit reason
    pub fn get_vmcs_exit_reason(&self, vmcs_region: VmcsRegion) -> Result<VmExitReason, HypervisorError> {
        let exit_reason = vmcs_region.read_field(VmcsField::VmExitReason)? as u32;
        
        // Convert VMCS exit reason to VmExitReason
        match exit_reason {
            0 => Ok(VmExitReason::Exception),
            1 => Ok(VmExitReason::Interrupt),
            2 => Ok(VmExitReason::TripleFault),
            3 => Ok(VmExitReason::IoInstruction),
            4 => Ok(VmExitReason::MsrRead),
            5 => Ok(VmExitReason::MsrWrite),
            6 => Ok(VmExitReason::CpuidInstruction),
            7 => Ok(VmExitReason::HltInstruction),
            8 => Ok(VmExitReason::InvalidState),
            9 => Ok(VmExitReason::ControlRegisterAccess),
            10 => Ok(VmExitReason::MovCr3),
            11 => Ok(VmExitReason::MovDr3),
            12 => Ok(VmExitReason::MovDr),
            13 => Ok(VmExitReason::DescriptorTableAccess),
            14 => Ok(VmExitReason::RdmsrInstruction),
            15 => Ok(VmExitReason::WrmsrInstruction),
            16 => Ok(VmExitReason::InvalidState),
            17 => Ok(VmExitReason::SoftwareInterrupt),
            18 => Ok(VmExitReason::ShadowVmcs),
            19 => Ok(VmExitReason::PendingMtpr),
            20 => Ok(VmExitReason::NmiWindow),
            21 => Ok(VmExitReason::TaskSwitch),
            22 => Ok(VmExitReason::Vmfunc),
            23 => Ok(VmExitReason::EnableEptViolation),
            24 => Ok(VmExitReason::AccessToVmcs),
            _ => Ok(VmExitReason::Unknown),
        }
    }
    
    /// Run VM with VMCB (AMD-V)
    pub fn vmcb_run(&mut self, vmcb_region: VmcbRegion) -> Result<(), HypervisorError> {
        // Clear TLB
        let tlb_control = 1; // FLUSH_ALL
        
        unsafe {
            core::arch::asm!(
                "vmcb_run",
                in("rcx") vmcb_region.get_address() as u64,
                in("rdx") tlb_control as u64
            );
        }
        
        Ok(())
    }
    
    /// Get VMCB exit code
    pub fn get_vmcb_exit_code(&self, vmcb_region: VmcbRegion) -> Result<SvmExitCode, HypervisorError> {
        let exit_code = vmcb_region.get_exit_code()?;
        
        // Convert SVM exit code
        match exit_code {
            code if code == SvmExitCode::VmExit as u64 => Ok(SvmExitCode::VmExit),
            code if code == SvmExitCode::Hlt as u64 => Ok(SvmExitCode::Hlt),
            code if code == SvmExitCode::Intr as u64 => Ok(SvmExitCode::Intr),
            code if code == SvmExitCode::Nmi as u64 => Ok(SvmExitCode::Nmi),
            code if code == SvmExitCode::CpuidInstruction as u64 => Ok(SvmExitCode::Instructions),
            _ => Ok(SvmExitCode::Unknown),
        }
    }
    
    /// Setup VMCS configuration
    fn setup_vmcs(&self, vmcs_region: &VmcsRegion) -> Result<(), HypervisorError> {
        // Setup pin-based execution controls
        let pin_controls = VmcsPinControls::EXTERNAL_INTERRUPT | 
                          VmcsPinControls::NMI | 
                          VmcsPinControls::VIRTUAL_NMIS;
        vmcs_region.write_field(VmcsField::PinBasedVmExecutionControls, pin_controls.bits())?;
        
        // Setup processor-based execution controls
        let proc_controls = VmcsControls::INTERRUPT_WINDOW |
                           VmcsControls::NMI_WINDOW |
                           VmcsControls::ENABLE_VM_FUNCTIONS |
                           VmcsControls::ENABLE_EPT |
                           VmcsControls::ENABLE_VPID;
        vmcs_region.write_field(VmcsField::PrimaryProcessorBasedVmExecutionControls, proc_controls.bits())?;
        
        // Setup secondary processor-based execution controls
        let secondary_controls = VmcsControls::ENABLE_UNRESTRICTED_GUEST |
                                VmcsControls::ENABLE_XSAVES;
        vmcs_region.write_field(VmcsField::SecondaryProcessorBasedVmExecutionControls, secondary_controls.bits())?;
        
        // Setup exit controls
        let exit_controls = 0x7E7; // Standard exit controls
        vmcs_region.write_field(VmcsField::PrimaryVmExitControls, exit_controls)?;
        
        Ok(())
    }
    
    /// Setup VMCB configuration
    fn setup_vmcb(&self, vmcb_region: &VmcbRegion) -> Result<(), HypervisorError> {
        // Setup intercepts
        let mut intercepts = vmcb_region.get_intercept()?;
        
        // Enable standard exits
        intercepts |= 1 << 0; // VMEXIT_READ_CR0
        intercepts |= 1 << 1; // VMEXIT_READ_CR3
        intercepts |= 1 << 2; // VMEXIT_READ_CR4
        intercepts |= 1 << 3; // VMEXIT_READ_CR8
        intercepts |= 1 << 4; // VMEXIT_WRITE_CR0
        intercepts |= 1 << 5; // VMEXIT_WRITE_CR3
        intercepts |= 1 << 6; // VMEXIT_WRITE_CR4
        intercepts |= 1 << 7; // VMEXIT_WRITE_CR8
        intercepts |= 1 << 9; // VMEXIT_INTR
        intercepts |= 1 << 10; // VMEXIT_NMI
        intercepts |= 1 << 12; // VMEXIT_CP
        
        vmcb_region.set_intercept(intercepts)?;
        
        Ok(())
    }
    
    /// Enable nested paging (EPT/SLBIT)
    pub fn enable_nested_paging(&mut self, enable: bool) -> Result<(), HypervisorError> {
        if enable && !self.capabilities.contains(HypervisorCapabilities::NESTED_PAGING) {
            return Err(HypervisorError::FeatureNotSupported);
        }
        
        // Configure nested paging in VMCS/VMCB
        for vmcs in &self.vmcs_regions {
            if enable {
                vmcs.write_field(VmcsField::SecondaryProcessorBasedVmExecutionControls, 
                               VmcsControls::ENABLE_EPT.bits())?;
            }
        }
        
        for vmcb in &self.vmcb_regions {
            if enable {
                vmcb.set_npt_enable(true)?;
            }
        }
        
        Ok(())
    }
}

/// VMCS Region structure
#[derive(Debug, Clone, Copy)]
pub struct VmcsRegion {
    vm_id: VmId,
    vcpu_id: VcpuId,
    address: usize,
}

impl VmcsRegion {
    /// Create a new VMCS region
    pub fn new(vm_id: VmId, vcpu_id: VcpuId) -> Result<Self, HypervisorError> {
        // Allocate 4KB aligned VMCS region
        let mut address: usize = 0;
        
        // In real implementation, this would allocate actual memory
        // For now, use a placeholder address
        address = 0xFFFF800000000000 + (vm_id.0 as usize * 0x1000) + (vcpu_id.0 as usize * 0x1000);
        
        Ok(VmcsRegion {
            vm_id,
            vcpu_id,
            address,
        })
    }
    
    /// Get VMCS region address
    pub fn get_address(&self) -> *mut u8 {
        self.address as *mut u8
    }
    
    /// Read VMCS field
    pub fn read_field(&self, field: VmcsField) -> Result<u64, HypervisorError> {
        // In real implementation, this would use VMREAD instruction
        // Simplified for demonstration
        unsafe {
            let value: u64;
            core::arch::asm!(
                "vmread %1, %0",
                out(reg) value,
                in(reg) field as u32
            );
            Ok(value)
        }
    }
    
    /// Write VMCS field
    pub fn write_field(&self, field: VmcsField, value: u64) -> Result<(), HypervisorError> {
        unsafe {
            core::arch::asm!(
                "vmwrite %1, %0",
                in(reg) field as u32,
                in(reg) value
            );
        }
        Ok(())
    }
}

/// VMCB Region structure
#[derive(Debug, Clone, Copy)]
pub struct VmcbRegion {
    vm_id: VmId,
    vcpu_id: VcpuId,
    address: usize,
}

impl VmcbRegion {
    /// Create a new VMCB region
    pub fn new(vm_id: VmId, vcpu_id: VcpuId) -> Result<Self, HypervisorError> {
        // Allocate 4KB aligned VMCB region
        let mut address: usize = 0;
        
        // In real implementation, this would allocate actual memory
        address = 0xFFFF800000001000 + (vm_id.0 as usize * 0x1000) + (vcpu_id.0 as usize * 0x1000);
        
        Ok(VmcbRegion {
            vm_id,
            vcpu_id,
            address,
        })
    }
    
    /// Get VMCB region address
    pub fn get_address(&self) -> *mut u8 {
        self.address as *mut u8
    }
    
    /// Get exit code
    pub fn get_exit_code(&self) -> Result<u64, HypervisorError> {
        // Read exit_code field from VMCB
        Ok(0) // Simplified
    }
    
    /// Get intercept bitmap
    pub fn get_intercept(&self) -> Result<u64, HypervisorError> {
        // Read intercept field from VMCB
        Ok(0) // Simplified
    }
    
    /// Set intercept bitmap
    pub fn set_intercept(&self, intercepts: u64) -> Result<(), HypervisorError> {
        // Write intercept field to VMCB
        Ok(())
    }
    
    /// Enable nested page tables
    pub fn set_npt_enable(&self, enable: bool) -> Result<(), HypervisorError> {
        // Set npt_enable field in VMCB
        Ok(())
    }
}

/// VMCS pointer for active VMCS tracking
#[derive(Debug, Clone, Copy)]
pub struct VmcsPointer {
    pub vm_id: VmId,
    pub vcpu_id: VcpuId,
    pub vmcs_region: VmcsRegion,
}

/// VMCB pointer for active VMCB tracking
#[derive(Debug, Clone, Copy)]
pub struct VmcbPointer {
    pub vm_id: VmId,
    pub vcpu_id: VcpuId,
    pub vmcb_region: VmcbRegion,
}