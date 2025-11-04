//! MultiOS Architecture-Specific Support Module
//! 
//! This module provides comprehensive architecture-specific functionality including
//! CPU feature detection, performance monitoring, multi-core support, and
//! architecture-specific features for x86_64, ARM64, and RISC-V.

use crate::log::{info, warn, error};

pub mod interrupts;

// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

// New comprehensive support modules
pub mod cpu_features;
pub mod performance;
pub mod multicore;
pub mod features;

/// Architecture-specific initialization
pub fn init() -> Result<(), crate::KernelError> {
    info!("Initializing comprehensive architecture support...");
    
    #[cfg(target_arch = "x86_64")]
    {
        info!("Detected x86_64 architecture");
        x86_64::init()?;
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        info!("Detected ARM64 architecture");
        aarch64::init()?;
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        info!("Detected RISC-V64 architecture");
        riscv64::init()?;
    }
    
    info!("Architecture support initialized successfully");
    Ok(())
}

/// Comprehensive CPU information with enhanced features
#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    pub vendor: &'static str,
    pub model: &'static str,
    pub family: u8,
    pub model_id: u8,
    pub stepping: u8,
    pub frequency_mhz: u32,
    pub cores: u8,
    pub threads_per_core: u8,
    pub total_cores: u8,
    pub total_threads: u8,
    pub socket_id: u8,
    pub package_id: u8,
    pub die_id: u8,
    pub cluster_id: u8,
}

/// Enhanced cache information
#[derive(Debug, Clone, Copy)]
pub struct CacheInfo {
    pub level: u8,
    pub cache_type: u8,
    pub size_kb: u32,
    pub associativity: u16,
    pub line_size: u16,
    pub num_sets: u32,
    pub inclusive: bool,
    pub write_back: bool,
    pub prefetch_supported: bool,
}

/// System configuration with enhanced features
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub page_size: usize,
    pub max_phys_addr: u64,
    pub max_virt_addr: u64,
    pub pointer_size: usize,
    pub endianness: Endianness,
    pub interrupt_controller: InterruptController,
    pub has_sse: bool,
    pub has_avx: bool,
    pub has_neon: bool,
    pub has_trustzone: bool,
    pub has_pmp: bool,
    pub has_svpbmt: bool,
    pub max_cores: u32,
    pub max_threads_per_core: u32,
    pub smp_enabled: bool,
    pub mobile_optimized: bool,
    pub low_power_mode: bool,
    pub touch_enabled: bool,
    pub gpu_accelerated: bool,
}

/// Architecture type enumeration (enhanced)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchType {
    X86_64 = 0,
    AArch64 = 1,
    Riscv64 = 2,
}

/// Endianness
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Endianness {
    Little = 0,
    Big = 1,
}

/// Interrupt controller type (enhanced)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptController {
    Apic = 0,
    Gic = 1,
    Clint = 2,
    Plic = 3,
    Unknown = 255,
}

/// Main architecture manager
pub struct ArchitectureManager {
    architecture: ArchType,
    cpu_features: Option<cpu_features::CpuFeatures>,
    performance_monitor: Option<performance::PerformanceMonitor>,
    multicore_manager: Option<multicore::MultiCoreManager>,
    feature_managers: Option<ArchitectureFeatures>,
}

/// Architecture-specific feature managers
#[cfg(target_arch = "x86_64")]
pub type ArchitectureFeatures = x86_features::X86Features;

#[cfg(target_arch = "aarch64")]
pub type ArchitectureFeatures = aarch64_features::AArch64Features;

#[cfg(target_arch = "riscv64")]
pub type ArchitectureFeatures = riscv_features::RiscVFeatures;

/// Architecture features trait for cross-platform compatibility
pub trait PlatformFeatures {
    fn init(&mut self) -> Result<(), crate::KernelError>;
    fn get_name(&self) -> &'static str;
}

#[cfg(target_arch = "x86_64")]
mod x86_features {
    use super::*;
    use super::features::x86_features::*;
    use super::cpu_features::{CpuFeatures, ArchType};
    
    pub struct X86Features {
        pub sse: SseSupport,
        pub avx: AvxSupport,
        pub acpi: AcpiSupport,
    }
    
    impl X86Features {
        pub fn new(features: &CpuFeatures) -> Self {
            Self {
                sse: SseSupport::new(features),
                avx: AvxSupport::new(features),
                acpi: AcpiSupport::new(),
            }
        }
        
        pub fn init_all(&mut self) -> Result<(), crate::KernelError> {
            self.sse.init()?;
            self.avx.init()?;
            self.acpi.init()?;
            Ok(())
        }
    }
    
    impl PlatformFeatures for X86Features {
        fn init(&mut self) -> Result<(), crate::KernelError> {
            self.init_all()
        }
        
        fn get_name(&self) -> &'static str {
            "x86_64"
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod aarch64_features {
    use super::*;
    use super::features::aarch64_features::*;
    use super::cpu_features::{CpuFeatures, ArchType};
    
    pub struct AArch64Features {
        pub neon: NeonSupport,
        pub trustzone: TrustZoneSupport,
        pub gic: GicSupport,
    }
    
    impl AArch64Features {
        pub fn new() -> Self {
            Self {
                neon: NeonSupport::new(&CpuFeatures {
                    architecture: ArchType::AArch64,
                    has_64bit: true,
                    has_fpu: true,
                    page_size: 4096,
                    max_physical_addr: 0xFFFF_FFFF_FFFF,
                    neon: true,
                    fp16: false,
                    asimd: true,
                    aes: false,
                    sha1: false,
                    sha256: false,
                    sha512: false,
                    sm3: false,
                    sm4: false,
                    pmull: false,
                    crc32: false,
                    atomics: false,
                    pointer_auth: false,
                    mem_tag: false,
                    mte: false,
                    gcs: false,
                    sel2: false,
                    sve: false,
                    sve2: false,
                    rdm: false,
                    lse: false,
                    aarch32: false,
                    vfp: true,
                    vfp3: true,
                    vfp4: false,
                    armv8_1: false,
                    armv8_2: false,
                    armv8_3: false,
                    armv8_4: false,
                    armv8_5: false,
                    armv8_6: false,
                    armv8_7: false,
                    sse: false,
                    sse2: false,
                    sse3: false,
                    ssse3: false,
                    sse4_1: false,
                    sse4_2: false,
                    avx: false,
                    avx2: false,
                    avx512: false,
                    fma: false,
                    bmi1: false,
                    bmi2: false,
                    lzcnt: false,
                    popcnt: false,
                    rdtsc: false,
                    rdtscp: false,
                    sysenter_sysexit: false,
                    syscall: true,
                    clflush: false,
                    clflushopt: false,
                    nx_bit: true,
                    pae: true,
                    la57: false,
                    smep: true,
                    smap: true,
                    pge: true,
                    invpcid: false,
                    tsx: false,
                    intel_pt: false,
                    intel CET: false,
                    amd_sev: false,
                    amd_sme: false,
                    rv64i: false,
                    rv32i: false,
                    rv32e: false,
                    rv64e: false,
                    rvc: false,
                    rva: false,
                    rvd: false,
                    rvf: false,
                    rvq: false,
                    rvh: false,
                    rvb: false,
                    rvk: false,
                    m: false,
                    a: false,
                    f: false,
                    d: false,
                    q: false,
                    c: false,
                    b: false,
                    k: false,
                    j: false,
                    t: false,
                    v: false,
                    n: false,
                    h: false,
                    g: false,
                    p: false,
                    pmp: false,
                    pmp_grain_granule: 0,
                    svpbmt: false,
                    svadu: false,
                    svinval: false,
                    svnapot: false,
                    sstc: false,
                    zicbom: false,
                    zicboz: false,
                    zicntr: false,
                    zicsr: false,
                    zifencei: false,
                }),
                trustzone: TrustZoneSupport::new(&CpuFeatures {
                    architecture: ArchType::AArch64,
                    has_64bit: true,
                    has_fpu: true,
                    page_size: 4096,
                    max_physical_addr: 0xFFFF_FFFF_FFFF,
                    sel2: true,
                    neon: false,
                    fp16: false,
                    asimd: false,
                    aes: false,
                    sha1: false,
                    sha256: false,
                    sha512: false,
                    sm3: false,
                    sm4: false,
                    pmull: false,
                    crc32: false,
                    atomics: false,
                    pointer_auth: false,
                    mem_tag: false,
                    mte: false,
                    gcs: false,
                    sve: false,
                    sve2: false,
                    rdm: false,
                    lse: false,
                    aarch32: false,
                    vfp: false,
                    vfp3: false,
                    vfp4: false,
                    armv8_1: false,
                    armv8_2: false,
                    armv8_3: false,
                    armv8_4: false,
                    armv8_5: false,
                    armv8_6: false,
                    armv8_7: false,
                    sse: false,
                    sse2: false,
                    sse3: false,
                    ssse3: false,
                    sse4_1: false,
                    sse4_2: false,
                    avx: false,
                    avx2: false,
                    avx512: false,
                    fma: false,
                    bmi1: false,
                    bmi2: false,
                    lzcnt: false,
                    popcnt: false,
                    rdtsc: false,
                    rdtscp: false,
                    sysenter_sysexit: false,
                    syscall: true,
                    clflush: false,
                    clflushopt: false,
                    nx_bit: true,
                    pae: true,
                    la57: false,
                    smep: true,
                    smap: true,
                    pge: true,
                    invpcid: false,
                    tsx: false,
                    intel_pt: false,
                    intel CET: false,
                    amd_sev: false,
                    amd_sme: false,
                    rv64i: false,
                    rv32i: false,
                    rv32e: false,
                    rv64e: false,
                    rvc: false,
                    rva: false,
                    rvd: false,
                    rvf: false,
                    rvq: false,
                    rvh: false,
                    rvb: false,
                    rvk: false,
                    m: false,
                    a: false,
                    f: false,
                    d: false,
                    q: false,
                    c: false,
                    b: false,
                    k: false,
                    j: false,
                    t: false,
                    v: false,
                    n: false,
                    h: false,
                    g: false,
                    p: false,
                    pmp: false,
                    pmp_grain_granule: 0,
                    svpbmt: false,
                    svadu: false,
                    svinval: false,
                    svnapot: false,
                    sstc: false,
                    zicbom: false,
                    zicboz: false,
                    zicntr: false,
                    zicsr: false,
                    zifencei: false,
                }),
                gic: GicSupport::new(),
            }
        }
        
        pub fn init_all(&mut self) -> Result<(), crate::KernelError> {
            self.neon.init()?;
            self.trustzone.init()?;
            self.gic.init()?;
            Ok(())
        }
    }
    
    impl PlatformFeatures for AArch64Features {
        fn init(&mut self) -> Result<(), crate::KernelError> {
            self.init_all()
        }
        
        fn get_name(&self) -> &'static str {
            "ARM64"
        }
    }
}

#[cfg(target_arch = "riscv64")]
mod riscv_features {
    use super::*;
    use super::features::riscv_features::*;
    use super::cpu_features::{CpuFeatures, ArchType};
    
    pub struct RiscVFeatures {
        pub extensions: ExtensionSupport,
        pub pmp: PmpSupport,
        pub svpbmt: SvpbmtSupport,
    }
    
    impl RiscVFeatures {
        pub fn new(features: &CpuFeatures) -> Self {
            Self {
                extensions: ExtensionSupport::new(features),
                pmp: PmpSupport::new(features),
                svpbmt: SvpbmtSupport::new(features),
            }
        }
        
        pub fn init_all(&mut self) -> Result<(), crate::KernelError> {
            self.extensions.init()?;
            self.pmp.init()?;
            self.svpbmt.init()?;
            Ok(())
        }
    }
    
    impl PlatformFeatures for RiscVFeatures {
        fn init(&mut self) -> Result<(), crate::KernelError> {
            self.init_all()
        }
        
        fn get_name(&self) -> &'static str {
            "RISC-V64"
        }
    }
}

impl ArchitectureManager {
    /// Create new architecture manager
    pub fn new() -> Self {
        let architecture = match env!("TARGET_ARCH") {
            "x86_64" => ArchType::X86_64,
            "aarch64" => ArchType::AArch64,
            "riscv64" => ArchType::Riscv64,
            _ => ArchType::X86_64, // Fallback
        };
        
        Self {
            architecture,
            cpu_features: None,
            performance_monitor: None,
            multicore_manager: None,
            feature_managers: None,
        }
    }
    
    /// Initialize comprehensive architecture support
    pub fn init(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing comprehensive {} architecture support...", 
              match self.architecture {
                  ArchType::X86_64 => "x86_64",
                  ArchType::AArch64 => "ARM64",
                  ArchType::Riscv64 => "RISC-V64",
              });
        
        // Initialize basic architecture support
        init()?;
        
        // Detect CPU features
        self.init_cpu_features()?;
        
        // Initialize performance monitoring
        self.init_performance_monitoring()?;
        
        // Initialize multi-core support
        self.init_multicore_support()?;
        
        // Initialize architecture-specific features
        self.init_architecture_features()?;
        
        info!("Comprehensive architecture support initialized successfully");
        Ok(())
    }
    
    /// Initialize CPU feature detection
    fn init_cpu_features(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing CPU feature detection...");
        
        let mut detector = cpu_features::CpuFeatureDetector::new();
        let features = detector.detect_features()?;
        self.cpu_features = Some(features.clone());
        
        info!("CPU features detected: {} cores, {} threads", 
              features.architecture as u8, 1);
        
        Ok(())
    }
    
    /// Initialize performance monitoring
    fn init_performance_monitoring(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing performance monitoring...");
        
        if let Some(ref features) = self.cpu_features {
            let mut monitor = performance::PerformanceMonitor::new(self.architecture, features.clone());
            monitor.init()?;
            self.performance_monitor = Some(monitor);
        }
        
        Ok(())
    }
    
    /// Initialize multi-core support
    fn init_multicore_support(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing multi-core support...");
        
        if let Some(ref features) = self.cpu_features {
            let mut manager = multicore::MultiCoreManager::new(self.architecture, features.clone());
            manager.init()?;
            self.multicore_manager = Some(manager);
        }
        
        Ok(())
    }
    
    /// Initialize architecture-specific features
    fn init_architecture_features(&mut self) -> Result<(), crate::KernelError> {
        info!("Initializing architecture-specific features...");
        
        #[cfg(target_arch = "x86_64")]
        {
            if let Some(ref features) = self.cpu_features {
                let mut features_manager = ArchitectureFeatures::new(features);
                features_manager.init()?;
                self.feature_managers = Some(features_manager);
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            let mut features_manager = ArchitectureFeatures::new();
            features_manager.init()?;
            self.feature_managers = Some(features_manager);
        }
        
        #[cfg(target_arch = "riscv64")]
        {
            if let Some(ref features) = self.cpu_features {
                let mut features_manager = ArchitectureFeatures::new(features);
                features_manager.init()?;
                self.feature_managers = Some(features_manager);
            }
        }
        
        Ok(())
    }
    
    /// Get architecture type
    pub fn get_architecture(&self) -> ArchType {
        self.architecture
    }
    
    /// Get CPU features
    pub fn get_cpu_features(&self) -> Option<&cpu_features::CpuFeatures> {
        self.cpu_features.as_ref()
    }
    
    /// Get performance monitor
    pub fn get_performance_monitor(&self) -> Option<&performance::PerformanceMonitor> {
        self.performance_monitor.as_ref()
    }
    
    /// Get multi-core manager
    pub fn get_multicore_manager(&self) -> Option<&multicore::MultiCoreManager> {
        self.multicore_manager.as_ref()
    }
    
    /// Get architecture features
    pub fn get_architecture_features(&self) -> Option<&ArchitectureFeatures> {
        self.feature_managers.as_ref()
    }
    
    /// Get current CPU information
    pub fn get_current_cpu_info(&self) -> CpuInfo {
        let cores = match self.multicore_manager.as_ref() {
            Some(manager) => manager.get_topology().total_cores as u8,
            None => 1,
        };
        
        let threads_per_core = match self.multicore_manager.as_ref() {
            Some(manager) => manager.get_topology().threads_per_core as u8,
            None => 1,
        };
        
        CpuInfo {
            vendor: "Unknown",
            model: match self.architecture {
                ArchType::X86_64 => "x86_64 CPU",
                ArchType::AArch64 => "ARM64 CPU",
                ArchType::Riscv64 => "RISC-V64 CPU",
            },
            family: 0,
            model_id: 0,
            stepping: 0,
            frequency_mhz: 2000,
            cores: cores,
            threads_per_core: threads_per_core,
            total_cores: cores,
            total_threads: cores * threads_per_core,
            socket_id: 0,
            package_id: 0,
            die_id: 0,
            cluster_id: 0,
        }
    }
    
    /// Get system configuration
    pub fn get_system_config(&self) -> SystemConfig {
        let max_cores = match self.multicore_manager.as_ref() {
            Some(manager) => manager.get_topology().total_cores,
            None => 1,
        };
        
        let max_threads_per_core = match self.multicore_manager.as_ref() {
            Some(manager) => manager.get_topology().threads_per_core,
            None => 1,
        };
        
        SystemConfig {
            page_size: 4096,
            max_phys_addr: match self.architecture {
                ArchType::X86_64 => 0xFFFFFFFFFFFFF000,
                ArchType::AArch64 => 0xFFFF_FFFF_FFFF,
                ArchType::Riscv64 => 0xFFFF_FFFF_FFFF,
            },
            max_virt_addr: 0xFFFF_FFFF_FFFF_FFFF,
            pointer_size: 8,
            endianness: Endianness::Little,
            interrupt_controller: match self.architecture {
                ArchType::X86_64 => InterruptController::Apic,
                ArchType::AArch64 => InterruptController::Gic,
                ArchType::Riscv64 => InterruptController::Clint,
            },
            has_sse: self.cpu_features.as_ref().map_or(false, |f| f.sse),
            has_avx: self.cpu_features.as_ref().map_or(false, |f| f.avx),
            has_neon: self.cpu_features.as_ref().map_or(false, |f| f.neon),
            has_trustzone: self.cpu_features.as_ref().map_or(false, |f| f.sel2),
            has_pmp: self.cpu_features.as_ref().map_or(false, |f| f.pmp),
            has_svpbmt: self.cpu_features.as_ref().map_or(false, |f| f.svpbmt),
            max_cores,
            max_threads_per_core,
            smp_enabled: max_cores > 1,
        }
    }
}

/// Initialize global architecture manager
pub fn init_architecture_manager() -> Result<ArchitectureManager, crate::KernelError> {
    let mut manager = ArchitectureManager::new();
    manager.init()?;
    Ok(manager)
}