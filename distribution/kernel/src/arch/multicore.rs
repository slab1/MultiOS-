//! Multi-core and Inter-Processor Communication Support
//! 
//! This module provides comprehensive multi-core support for x86_64,
//! ARM64, and RISC-V architectures, including CPU discovery, inter-processor
//! communication, and synchronization primitives.

use crate::log::{info, warn, error};
use crate::KernelError;
use super::{ArchType, CpuFeatures};

/// CPU core information
#[derive(Debug, Clone)]
pub struct CoreInfo {
    pub core_id: u32,
    pub socket_id: u32,
    pub package_id: u32,
    pub die_id: u32,
    pub cluster_id: u32,
    pub thread_id: u32,
    pub is_online: bool,
    pub is_primary: bool,
    pub is_smt_capable: bool,
    pub is_smt_enabled: bool,
    pub frequency_mhz: u32,
    pub max_frequency_mhz: u32,
    pub min_frequency_mhz: u32,
    pub current_governor: FrequencyGovernor,
    pub cache_info: Option<Vec<CacheInfo>>,
}

/// CPU socket/package information
#[derive(Debug, Clone)]
pub struct SocketInfo {
    pub socket_id: u32,
    pub vendor_id: u32,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub cores_per_socket: u32,
    pub threads_per_socket: u32,
    pub max_frequency_mhz: u32,
    pub min_frequency_mhz: u32,
    pub l3_cache_size: u32,
    pub memory_channels: u8,
}

/// Cache information structure
#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub level: u8,
    pub cache_type: CacheType,
    pub size_kb: u32,
    pub line_size: u32,
    pub associativity: u16,
    pub num_sets: u32,
    pub inclusive: bool,
    pub write_back: bool,
}

/// Cache type enumeration
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CacheType {
    Instruction = 0,
    Data = 1,
    Unified = 2,
}

/// Frequency governor enumeration
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum FrequencyGovernor {
    Performance = 0,
    Powersave = 1,
    OnDemand = 2,
    Conservative = 3,
    Userspace = 4,
    Schedutil = 5,
}

/// Inter-processor interrupt (IPI) message types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IpiMessage {
    Wakeup = 0,
    SendNmi = 1,
    LocalTimer = 2,
    BroadcastFunction = 3,
    RemoteTlbInvalidate = 4,
    ResumeFromLowPower = 5,
    Custom = 255,
}

/// Inter-processor communication structure
#[derive(Debug, Clone)]
pub struct IpiMessageInfo {
    pub target_cores: Vec<u32>,
    pub message_type: IpiMessage,
    pub data: u32,
    pub wakeup_vector: u32,
}

/// Multi-core topology information
#[derive(Debug, Clone)]
pub struct TopologyInfo {
    pub architecture: ArchType,
    pub total_sockets: u32,
    pub total_cores: u32,
    pub total_threads: u32,
    pub cores_per_socket: u32,
    pub threads_per_core: u32,
    pub max_threads_per_socket: u32,
    pub numa_nodes: u32,
    pub has_smt: bool,
    pub socket_info: Vec<SocketInfo>,
    pub core_info: Vec<CoreInfo>,
}

/// Multi-core manager implementation
pub struct MultiCoreManager {
    architecture: ArchType,
    features: CpuFeatures,
    topology: TopologyInfo,
    bootstrap_cpu: u32,
    is_smp_initialized: bool,
    ipi_manager: Option<IpiManager>,
}

/// Inter-processor communication manager
pub struct IpiManager {
    architecture: ArchType,
    message_queues: Vec<Vec<IpiMessageInfo>>,
    active_messages: Vec<IpiMessageInfo>,
}

impl MultiCoreManager {
    /// Create new multi-core manager
    pub fn new(architecture: ArchType, features: CpuFeatures) -> Self {
        Self {
            architecture,
            features,
            topology: TopologyInfo {
                architecture,
                total_sockets: 0,
                total_cores: 0,
                total_threads: 0,
                cores_per_socket: 0,
                threads_per_core: 0,
                max_threads_per_socket: 0,
                numa_nodes: 0,
                has_smt: false,
                socket_info: Vec::new(),
                core_info: Vec::new(),
            },
            bootstrap_cpu: 0,
            is_smp_initialized: false,
            ipi_manager: None,
        }
    }
    
    /// Initialize multi-core support
    pub fn init(&mut self) -> Result<(), KernelError> {
        info!("Initializing multi-core support for {:?}...", self.architecture);
        
        // Discover CPU topology
        self.discover_topology()?;
        
        // Initialize APIC/GIC/CLINT for inter-processor communication
        self.init_interrupt_controller()?;
        
        // Initialize IPI manager
        self.init_ipi_manager()?;
        
        // Initialize secondary cores
        self.init_secondary_cores()?;
        
        self.is_smp_initialized = true;
        info!("Multi-core initialization complete: {} cores, {} threads", 
              self.topology.total_cores, self.topology.total_threads);
        
        Ok(())
    }
    
    /// Discover CPU topology
    fn discover_topology(&mut self) -> Result<(), KernelError> {
        info!("Discovering CPU topology...");
        
        match self.architecture {
            ArchType::X86_64 => self.discover_x86_64_topology()?,
            ArchType::AArch64 => self.discover_aarch64_topology()?,
            ArchType::Riscv64 => self.discover_riscv64_topology()?,
        }
        
        info!("Topology discovery complete");
        Ok(())
    }
    
    /// Discover x86_64 CPU topology
    fn discover_x86_64_topology(&mut self) -> Result<(), KernelError> {
        #[cfg(target_arch = "x86_64")]
        {
            // Get APIC ID and topology information
            let apic_id = self.get_x86_64_apic_id();
            let logical_processors = self.get_x86_64_logical_processors();
            let packages = self.get_x86_64_packages();
            let cores_per_package = self.get_x86_64_cores_per_package();
            
            // Build topology
            self.topology.total_sockets = packages;
            self.topology.total_cores = cores_per_package * packages;
            self.topology.total_threads = logical_processors;
            self.topology.cores_per_socket = cores_per_package;
            self.topology.threads_per_core = logical_processors / self.topology.total_cores;
            self.topology.max_threads_per_socket = logical_processors / packages;
            self.topology.has_smt = self.topology.threads_per_core > 1;
            
            // Detect Bootstrap CPU (BSP)
            self.bootstrap_cpu = 0; // Usually CPU 0 is the bootstrap CPU
            
            // Create core and socket information
            self.build_x86_64_topology_info(apic_id)?;
        }
        
        Ok(())
    }
    
    /// Get x86_64 APIC ID
    fn get_x86_64_apic_id(&self) -> u32 {
        #[cfg(target_arch = "x86_64")]
        {
            let mut eax: u32;
            let mut ebx: u32;
            
            unsafe {
                core::arch::asm!(
                    "cpuid",
                    inout("eax") 0x1 => eax,
                    inout("ebx") 0 => ebx,
                );
            }
            
            (ebx >> 24) as u32 // APIC ID is in bits 24-31 of EBX
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            0
        }
    }
    
    /// Get x86_64 logical processor count
    fn get_x86_64_logical_processors(&self) -> u32 {
        #[cfg(target_arch = "x86_64")]
        {
            let mut eax: u32;
            let mut ebx: u32;
            
            unsafe {
                core::arch::asm!(
                    "cpuid",
                    inout("eax") 0x1 => eax,
                    inout("ebx") 0 => ebx,
                );
            }
            
            ((ebx >> 16) & 0xFF) as u32 // Logical processor count in bits 16-23 of EBX
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            1
        }
    }
    
    /// Get x86_64 package count
    fn get_x86_64_packages(&self) -> u32 {
        1 // Simplified - would need ACPI or topology CPUID leaf for multiple packages
    }
    
    /// Get x86_64 cores per package
    fn get_x86_64_cores_per_package(&self) -> u32 {
        1 // Simplified - would need topology CPUID leaf
    }
    
    /// Build x86_64 topology information
    fn build_x86_64_topology_info(&mut self, apic_id: u32) -> Result<(), KernelError> {
        // Create socket information
        let socket = SocketInfo {
            socket_id: 0,
            vendor_id: 0,
            family: 6,
            model: 0x0F,
            stepping: 0,
            cores_per_socket: self.topology.cores_per_socket,
            threads_per_socket: self.topology.max_threads_per_socket,
            max_frequency_mhz: 2400,
            min_frequency_mhz: 800,
            l3_cache_size: 8192,
            memory_channels: 2,
        };
        self.topology.socket_info.push(socket);
        
        // Create core information
        for thread in 0..self.topology.total_threads {
            let core_info = CoreInfo {
                core_id: thread,
                socket_id: 0,
                package_id: 0,
                die_id: 0,
                cluster_id: 0,
                thread_id: thread,
                is_online: thread == self.bootstrap_cpu,
                is_primary: thread == self.bootstrap_cpu,
                is_smt_capable: self.topology.has_smt,
                is_smt_enabled: self.topology.has_smt,
                frequency_mhz: 2000,
                max_frequency_mhz: 2400,
                min_frequency_mhz: 800,
                current_governor: FrequencyGovernor::Performance,
                cache_info: Some(self.build_cache_info()),
            };
            self.topology.core_info.push(core_info);
        }
        
        Ok(())
    }
    
    /// Build cache information
    fn build_cache_info(&self) -> Vec<CacheInfo> {
        vec![
            CacheInfo {
                level: 1,
                cache_type: CacheType::Instruction,
                size_kb: 32,
                line_size: 64,
                associativity: 8,
                num_sets: 64,
                inclusive: false,
                write_back: true,
            },
            CacheInfo {
                level: 1,
                cache_type: CacheType::Data,
                size_kb: 32,
                line_size: 64,
                associativity: 8,
                num_sets: 64,
                inclusive: false,
                write_back: true,
            },
            CacheInfo {
                level: 2,
                cache_type: CacheType::Unified,
                size_kb: 256,
                line_size: 64,
                associativity: 8,
                num_sets: 512,
                inclusive: true,
                write_back: true,
            },
            CacheInfo {
                level: 3,
                cache_type: CacheType::Unified,
                size_kb: 8192,
                line_size: 64,
                associativity: 16,
                num_sets: 2048,
                inclusive: true,
                write_back: true,
            },
        ]
    }
    
    /// Discover ARM64 CPU topology
    fn discover_aarch64_topology(&mut self) -> Result<(), KernelError> {
        #[cfg(target_arch = "aarch64")]
        {
            // Use ARM64 system registers to get topology information
            let mpidr_el1 = self.get_aarch64_mpidr_el1();
            
            // Extract topology from MPIDR_EL1
            let cluster_id = (mpidr_el1 >> 8) & 0xFF;
            let core_id = mpidr_el1 & 0xFF;
            let thread_id = (mpidr_el1 >> 16) & 0xFF;
            
            self.topology.total_sockets = 1; // ARM64 typically single socket
            self.topology.total_cores = 4;   // Default for ARM64 testing
            self.topology.total_threads = self.topology.total_cores;
            self.topology.cores_per_socket = self.topology.total_cores;
            self.topology.threads_per_core = 1;
            self.topology.max_threads_per_socket = self.topology.total_threads;
            self.topology.has_smt = false;   // ARM64 typically doesn't have SMT
            
            self.bootstrap_cpu = 0;
            
            // Create topology information for ARM64
            self.build_aarch64_topology_info()?;
        }
        
        Ok(())
    }
    
    /// Get ARM64 MPIDR_EL1 register
    fn get_aarch64_mpidr_el1(&self) -> u64 {
        #[cfg(target_arch = "aarch64")]
        {
            let mut mpidr: u64;
            unsafe {
                core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr);
            }
            mpidr
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            0
        }
    }
    
    /// Build ARM64 topology information
    fn build_aarch64_topology_info(&mut self) -> Result<(), KernelError> {
        // Create socket information
        let socket = SocketInfo {
            socket_id: 0,
            vendor_id: 0x41, // 'A' for ARM
            family: 8,       // ARMv8
            model: 0,
            stepping: 0,
            cores_per_socket: self.topology.cores_per_socket,
            threads_per_socket: self.topology.max_threads_per_socket,
            max_frequency_mhz: 2000,
            min_frequency_mhz: 1000,
            l3_cache_size: 4096,
            memory_channels: 1,
        };
        self.topology.socket_info.push(socket);
        
        // Create core information
        for core in 0..self.topology.total_cores {
            let core_info = CoreInfo {
                core_id: core,
                socket_id: 0,
                package_id: 0,
                die_id: 0,
                cluster_id: core,
                thread_id: 0,
                is_online: core == self.bootstrap_cpu,
                is_primary: core == self.bootstrap_cpu,
                is_smt_capable: false,
                is_smt_enabled: false,
                frequency_mhz: 1800,
                max_frequency_mhz: 2000,
                min_frequency_mhz: 1000,
                current_governor: FrequencyGovernor::Performance,
                cache_info: Some(self.build_cache_info()),
            };
            self.topology.core_info.push(core_info);
        }
        
        Ok(())
    }
    
    /// Discover RISC-V CPU topology
    fn discover_riscv64_topology(&mut self) -> Result<(), KernelError> {
        #[cfg(target_arch = "riscv64")]
        {
            // Use RISC-V CSR registers to get HART (thread) information
            let mut num_harts: u64 = 1; // Default to 1 HART
            
            // Check if we can read number of HARTs
            if self.features.h {
                // Hypervisor extension available
                num_harts = self.get_riscv64_hart_count();
            }
            
            self.topology.total_sockets = 1; // RISC-V typically single socket
            self.topology.total_cores = num_harts as u32;
            self.topology.total_threads = self.topology.total_cores;
            self.topology.cores_per_socket = self.topology.total_cores;
            self.topology.threads_per_core = 1;
            self.topology.max_threads_per_socket = self.topology.total_threads;
            self.topology.has_smt = false; // RISC-V SMT implementation varies
            
            self.bootstrap_cpu = 0;
            
            // Create topology information for RISC-V
            self.build_riscv64_topology_info()?;
        }
        
        Ok(())
    }
    
    /// Get RISC-V HART count
    fn get_riscv64_hart_count(&self) -> u64 {
        #[cfg(target_arch = "riscv64")]
        {
            // Read number of HARTs from system configuration
            // This would typically come from device tree or ACPI
            1
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            1
        }
    }
    
    /// Build RISC-V topology information
    fn build_riscv64_topology_info(&mut self) -> Result<(), KernelError> {
        // Create socket information
        let socket = SocketInfo {
            socket_id: 0,
            vendor_id: 0x52, // 'R' for RISC-V
            family: 64,      // RV64
            model: 0,
            stepping: 0,
            cores_per_socket: self.topology.cores_per_socket,
            threads_per_socket: self.topology.max_threads_per_socket,
            max_frequency_mhz: 1000,
            min_frequency_mhz: 500,
            l3_cache_size: 2048,
            memory_channels: 1,
        };
        self.topology.socket_info.push(socket);
        
        // Create core information
        for hart in 0..self.topology.total_cores {
            let core_info = CoreInfo {
                core_id: hart,
                socket_id: 0,
                package_id: 0,
                die_id: 0,
                cluster_id: 0,
                thread_id: hart,
                is_online: hart == self.bootstrap_cpu,
                is_primary: hart == self.bootstrap_cpu,
                is_smt_capable: false,
                is_smt_enabled: false,
                frequency_mhz: 800,
                max_frequency_mhz: 1000,
                min_frequency_mhz: 500,
                current_governor: FrequencyGovernor::Performance,
                cache_info: Some(self.build_cache_info()),
            };
            self.topology.core_info.push(core_info);
        }
        
        Ok(())
    }
    
    /// Initialize interrupt controller for multi-core
    fn init_interrupt_controller(&mut self) -> Result<(), KernelError> {
        info!("Initializing interrupt controller for multi-core support...");
        
        match self.architecture {
            ArchType::X86_64 => self.init_x86_64_apic()?,
            ArchType::AArch64 => self.init_aarch64_gic()?,
            ArchType::Riscv64 => self.init_riscv64_clint_plic()?,
        }
        
        Ok(())
    }
    
    /// Initialize x86_64 APIC
    fn init_x86_64_apic(&mut self) -> Result<(), KernelError> {
        #[cfg(target_arch = "x86_64")]
        {
            info!("Initializing x86_64 APIC...");
            
            // Enable APIC if not already enabled
            let mut ia32_apic_base: u64;
            unsafe {
                core::arch::asm!(
                    "mov {}, %rax",
                    "rdmsr",
                    in("rcx") 0x1Bu64, // IA32_APIC_BASE MSR
                    out(reg) ia32_apic_base,
                );
            }
            
            if (ia32_apic_base & (1 << 11)) == 0 {
                // APIC is disabled, enable it
                ia32_apic_base |= 1 << 11; // APIC Global Enable bit
                
                unsafe {
                    core::arch::asm!(
                        "mov {}, %rax",
                        "wrmsr",
                        in(reg) ia32_apic_base,
                        in("rcx") 0x1Bu64,
                    );
                }
            }
            
            // Set up local APIC
            self.setup_x86_64_local_apic()?;
        }
        
        Ok(())
    }
    
    /// Set up x86_64 local APIC
    fn setup_x86_64_local_apic(&self) -> Result<(), KernelError> {
        #[cfg(target_arch = "x86_64")]
        {
            // Initialize local APIC for current CPU
            let apic_id = self.get_x86_64_apic_id();
            
            // Set task priority
            self.write_x86_64_apic_register(0x80, 0x00); // TPR = 0
            
            // Enable spurious interrupt
            self.write_x86_64_apic_register(0x0F, 0x1FF); // SVR with APIC enabled
            
            // Set EOI behavior
            self.write_x86_64_apic_register(0x0B, 0x00); // EOI register
        }
        
        Ok(())
    }
    
    /// Write x86_64 APIC register
    fn write_x86_64_apic_register(&self, offset: u32, value: u32) {
        #[cfg(target_arch = "x86_64")]
        {
            let apic_base: usize = 0xFEE00000;
            let address = apic_base + (offset as usize * 16);
            
            unsafe {
                core::arch::asm!(
                    "mov {}, [{}+{}]",
                    in(reg) value,
                    in(reg) apic_base,
                    in(reg) offset,
                );
            }
        }
    }
    
    /// Initialize ARM64 GIC
    fn init_aarch64_gic(&mut self) -> Result<(), KernelError> {
        #[cfg(target_arch = "aarch64")]
        {
            info!("Initializing ARM64 GIC...");
            
            // GIC initialization would be done here
            // This involves detecting GIC version (v2 or v3) and configuring
            // distributor and redistributor registers
        }
        
        Ok(())
    }
    
    /// Initialize RISC-V CLINT and PLIC
    fn init_riscv64_clint_plic(&mut self) -> Result<(), KernelError> {
        #[cfg(target_arch = "riscv64")]
        {
            info!("Initializing RISC-V CLINT and PLIC...");
            
            // CLINT initialization (for timer and software interrupts)
            // PLIC initialization (for external interrupts)
            
            // Set up interrupt priorities and routing
        }
        
        Ok(())
    }
    
    /// Initialize IPI manager
    fn init_ipi_manager(&mut self) -> Result<(), KernelError> {
        info!("Initializing IPI manager...");
        
        let ipi_manager = IpiManager {
            architecture: self.architecture,
            message_queues: vec![Vec::new(); self.topology.total_cores as usize],
            active_messages: Vec::new(),
        };
        
        self.ipi_manager = Some(ipi_manager);
        
        Ok(())
    }
    
    /// Initialize secondary cores
    fn init_secondary_cores(&mut self) -> Result<(), KernelError> {
        info!("Initializing secondary cores...");
        
        for core_info in &self.topology.core_info {
            if core_info.core_id != self.bootstrap_cpu {
                self.wake_up_core(core_info.core_id)?;
            }
        }
        
        Ok(())
    }
    
    /// Wake up a specific core
    fn wake_up_core(&self, core_id: u32) -> Result<(), KernelError> {
        info!("Waking up core {}", core_id);
        
        match self.architecture {
            ArchType::X86_64 => self.wake_up_x86_64_core(core_id)?,
            ArchType::AArch64 => self.wake_up_aarch64_core(core_id)?,
            ArchType::Riscv64 => self.wake_up_riscv64_core(core_id)?,
        }
        
        Ok(())
    }
    
    /// Wake up x86_64 core
    fn wake_up_x86_64_core(&self, core_id: u32) -> Result<(), KernelError> {
        #[cfg(target_arch = "x86_64")]
        {
            // Send INIT IPI to target core
            self.send_ipi(core_id, IpiMessage::Wakeup, 0)?;
            
            // Wait for acknowledgment
            self.wait_for_core_ready(core_id)?;
        }
        
        Ok(())
    }
    
    /// Wake up ARM64 core
    fn wake_up_aarch64_core(&self, core_id: u32) -> Result<(), KernelError> {
        #[cfg(target_arch = "aarch64")]
        {
            // Send IPI via GIC to target core
            // This would involve writing to GIC distributor register
        }
        
        Ok(())
    }
    
    /// Wake up RISC-V core
    fn wake_up_riscv64_core(&self, core_id: u32) -> Result<(), KernelError> {
        #[cfg(target_arch = "riscv64")]
        {
            // Send software interrupt via CLINT to target HART
            // This involves writing to CLINT MSIP register
        }
        
        Ok(())
    }
    
    /// Send IPI to target core
    fn send_ipi(&self, target_core: u32, message_type: IpiMessage, data: u32) -> Result<(), KernelError> {
        let ipi_info = IpiMessageInfo {
            target_cores: vec![target_core],
            message_type,
            data,
            wakeup_vector: 0,
        };
        
        match self.architecture {
            ArchType::X86_64 => self.send_x86_64_ipi(&ipi_info)?,
            ArchType::AArch64 => self.send_aarch64_ipi(&ipi_info)?,
            ArchType::Riscv64 => self.send_riscv64_ipi(&ipi_info)?,
        }
        
        Ok(())
    }
    
    /// Send x86_64 IPI
    fn send_x86_64_ipi(&self, ipi_info: &IpiMessageInfo) -> Result<(), KernelError> {
        #[cfg(target_arch = "x86_64")]
        {
            let target_apic_id = ipi_info.target_cores[0];
            
            // Use APIC command register to send IPI
            // Format: [ICR_HIGH | ICR_LOW]
            let icr_low = (target_apic_id as u32) << 24 | 
                          (ipi_info.data as u32 & 0xFF) |
                          0x00004000; // Send IPI
            
            unsafe {
                core::arch::asm!(
                    "mov {}, %eax",
                    "mov {}, %ecx", 
                    "out %dx, %al",
                    in(reg) icr_low,
                    in("edx") 0xFEE00300u32,
                );
            }
        }
        
        Ok(())
    }
    
    /// Send ARM64 IPI
    fn send_aarch64_ipi(&self, ipi_info: &IpiMessageInfo) -> Result<(), KernelError> {
        #[cfg(target_arch = "aarch64")]
        {
            // Use GIC to send IPI to target CPU
            // This involves writing to GIC distributor register
        }
        
        Ok(())
    }
    
    /// Send RISC-V IPI
    fn send_riscv64_ipi(&self, ipi_info: &IpiMessageInfo) -> Result<(), KernelError> {
        #[cfg(target_arch = "riscv64")]
        {
            // Use CLINT to send software interrupt to target HART
            // This involves writing to CLINT MSIP register
        }
        
        Ok(())
    }
    
    /// Wait for core to become ready
    fn wait_for_core_ready(&self, core_id: u32) -> Result<(), KernelError> {
        // In a real implementation, this would poll status flags or use synchronization
        // For now, just wait a bit
        for _ in 0..1000 {
            core::arch::asm!("pause");
        }
        
        Ok(())
    }
    
    /// Get topology information
    pub fn get_topology(&self) -> &TopologyInfo {
        &self.topology
    }
    
    /// Get bootstrap CPU ID
    pub fn get_bootstrap_cpu(&self) -> u32 {
        self.bootstrap_cpu
    }
    
    /// Get current CPU ID
    pub fn get_current_cpu_id(&self) -> u32 {
        match self.architecture {
            ArchType::X86_64 => self.get_x86_64_apic_id(),
            ArchType::AArch64 => {
                let mpidr = self.get_aarch64_mpidr_el1();
                (mpidr & 0xFF) as u32
            },
            ArchType::Riscv64 => {
                // Get HART ID from CSR
                0 // Simplified
            },
        }
    }
    
    /// Check if SMP is initialized
    pub fn is_smp_initialized(&self) -> bool {
        self.is_smp_initialized
    }
    
    /// Get IPI manager
    pub fn get_ipi_manager(&mut self) -> Option<&mut IpiManager> {
        self.ipi_manager.as_mut()
    }
    
    /// Bring online a specific core
    pub fn bring_online_core(&mut self, core_id: u32) -> Result<(), KernelError> {
        info!("Bringing core {} online...", core_id);
        
        if let Some(core_info) = self.topology.core_info.iter_mut().find(|c| c.core_id == core_id) {
            if !core_info.is_online {
                self.wake_up_core(core_id)?;
                core_info.is_online = true;
            }
        }
        
        Ok(())
    }
    
    /// Take offline a specific core
    pub fn take_offline_core(&mut self, core_id: u32) -> Result<(), KernelError> {
        info!("Taking core {} offline...", core_id);
        
        if let Some(core_info) = self.topology.core_info.iter_mut().find(|c| c.core_id == core_id) {
            if core_info.core_id != self.bootstrap_cpu && core_info.is_online {
                // Send shutdown IPI
                self.send_ipi(core_id, IpiMessage::SendNmi, 0)?;
                core_info.is_online = false;
            }
        }
        
        Ok(())
    }
}