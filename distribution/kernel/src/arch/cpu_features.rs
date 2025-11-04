//! CPU Feature Detection and Capability Enumeration
//! 
//! This module provides comprehensive CPU feature detection for x86_64,
//! ARM64, and RISC-V architectures, including SIMD extensions, security
//! features, and performance monitoring capabilities.

use crate::log::{info, warn, error};
use crate::KernelError;

/// CPU features structure
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub architecture: ArchType,
    
    // Common features
    pub has_64bit: bool,
    pub has_fpu: bool,
    pub page_size: usize,
    pub max_physical_addr: u64,
    
    // x86_64 specific features
    pub sse: bool,
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse4_1: bool,
    pub sse4_2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512: bool,
    pub fma: bool,
    pub bmi1: bool,
    pub bmi2: bool,
    pub lzcnt: bool,
    pub popcnt: bool,
    pub rdtsc: bool,
    pub rdtscp: bool,
    pub sysenter_sysexit: bool,
    pub syscall: bool,
    pub clflush: bool,
    pub clflushopt: bool,
    
    // x86_64 security features
    pub nx_bit: bool,
    pub pae: bool,
    pub la57: bool,
    pub smep: bool,
    pub smap: bool,
    pub pge: bool,
    pub invpcid: bool,
    pub tsx: bool,
    pub intel_pt: bool,
    pub intel CET: bool,
    pub amd_sev: bool,
    pub amd_sme: bool,
    
    // ARM64 specific features
    pub neon: bool,
    pub fp16: bool,
    pub asimd: bool,
    pub aes: bool,
    pub sha1: bool,
    pub sha256: bool,
    pub sha512: bool,
    pub sm3: bool,
    pub sm4: bool,
    pub pmull: bool,
    pub crc32: bool,
    pub atomics: bool,
    
    // ARM64 security features
    pub pointer_auth: bool,
    pub mem_tag: bool,
    pub mte: bool,
    pub gcs: bool,
    pub sel2: bool,
    pub sve: bool,
    pub sve2: bool,
    pub rdm: bool,
    pub lse: bool,
    
    // ARM64 architecture features
    pub aarch32: bool,
    pub vfp: bool,
    pub vfp3: bool,
    pub vfp4: bool,
    pub armv8_1: bool,
    pub armv8_2: bool,
    pub armv8_3: bool,
    pub armv8_4: bool,
    pub armv8_5: bool,
    pub armv8_6: bool,
    pub armv8_7: bool,
    
    // RISC-V specific features
    pub rv64i: bool,
    pub rv32i: bool,
    pub rv32e: bool,
    pub rv64e: bool,
    pub rvc: bool,
    pub rva: bool,
    pub rvd: bool,
    pub rvf: bool,
    pub rvq: bool,
    pub rvh: bool,
    pub rvb: bool,
    pub rvk: bool,
    
    // RISC-V extensions
    pub m: bool,    // Integer multiply/divide
    pub a: bool,    // Atomics
    pub f: bool,    // Single precision float
    pub d: bool,    // Double precision float
    pub q: bool,    // Quad precision float
    pub c: bool,    // Compressed instructions
    pub b: bool,    // Bit manipulation
    pub k: bool,    // Cryptography
    pub j: bool,    // Dynamic translation
    pub t: bool,    // Transactional memory
    pub v: bool,    // Vector operations
    pub n: bool,    // User-level interrupts
    pub h: bool,    // Hypervisor
    pub g: bool,    // General extension
    pub p: bool,    // Packed SIMD
    
    // RISC-V security features
    pub pmp: bool,              // Physical Memory Protection
    pub pmp_grain_granule: u8,  // PMP grain granularity
    pub svpbmt: bool,           // Svpbmt extension
    pub svadu: bool,            // Svadu extension
    pub svinval: bool,          // Svinval extension
    pub svnapot: bool,          // Svnapot extension
    pub sstc: bool,             // Sstc extension
    pub zicbom: bool,           // Zicbom extension
    pub zicboz: bool,           // Zicboz extension
    pub zicntr: bool,           // Zicntr extension
    pub zicsr: bool,            // Zicsr extension
    pub zifencei: bool,         // Zifencei extension
}

/// Architecture type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchType {
    X86_64,
    AArch64,
    Riscv64,
}

/// Capability information structure
#[derive(Debug, Clone)]
pub struct CapabilityInfo {
    pub architecture: ArchType,
    pub max_cores: u32,
    pub max_threads_per_core: u32,
    pub l1_cache: Option<CacheInfo>,
    pub l2_cache: Option<CacheInfo>,
    pub l3_cache: Option<CacheInfo>,
    pub cache_coherency: CacheCoherency,
    pub smt_support: bool,
    pub numa_support: bool,
    pub virtual_memory_levels: u8,
    pub features: CpuFeatures,
}

/// Cache information structure
#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub size_bytes: usize,
    pub line_size: usize,
    pub associativity: u16,
    pub num_sets: u32,
    pub level: u8,
    pub cache_type: CacheType,
    pub inclusive: bool,
    pub write_policy: WritePolicy,
}

/// Cache type enumeration
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CacheType {
    Instruction = 0,
    Data = 1,
    Unified = 2,
    Invalid = 255,
}

/// Write policy enumeration
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum WritePolicy {
    WriteThrough = 0,
    WriteBack = 1,
}

/// Cache coherency enumeration
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CacheCoherency {
    MESI = 0,
    MOESI = 1,
    MESIF = 2,
    MOESIF = 3,
    None = 255,
}

/// CPU feature detection implementation
pub struct CpuFeatureDetector {
    architecture: ArchType,
    detected_features: Option<CpuFeatures>,
    capability_info: Option<CapabilityInfo>,
}

impl CpuFeatureDetector {
    /// Create new CPU feature detector
    pub fn new() -> Self {
        Self {
            architecture: Self::detect_architecture(),
            detected_features: None,
            capability_info: None,
        }
    }
    
    /// Detect current architecture
    fn detect_architecture() -> ArchType {
        #[cfg(target_arch = "x86_64")]
        {
            return ArchType::X86_64;
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            return ArchType::AArch64;
        }
        
        #[cfg(target_arch = "riscv64")]
        {
            return ArchType::Riscv64;
        }
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
        {
            error!("Unsupported architecture: {}", env!("TARGET_ARCH"));
            ArchType::X86_64 // Fallback
        }
    }
    
    /// Detect CPU features for current architecture
    pub fn detect_features(&mut self) -> Result<&CpuFeatures, KernelError> {
        info!("Detecting CPU features for {:?} architecture...", self.architecture);
        
        let features = match self.architecture {
            ArchType::X86_64 => self.detect_x86_64_features()?,
            ArchType::AArch64 => self.detect_aarch64_features()?,
            ArchType::Riscv64 => self.detect_riscv64_features()?,
        };
        
        self.detected_features = Some(features);
        
        info!("CPU feature detection complete");
        Ok(self.detected_features.as_ref().unwrap())
    }
    
    /// Detect x86_64 CPU features using CPUID
    fn detect_x86_64_features(&self) -> Result<CpuFeatures, KernelError> {
        info!("Detecting x86_64 CPU features...");
        
        let mut features = CpuFeatures {
            architecture: ArchType::X86_64,
            has_64bit: true, // Always true for x86_64 target
            has_fpu: true,
            page_size: 4096,
            max_physical_addr: 0xFFFFFFFFFFFFF000, // 48-bit PA
            
            // SIMD and vector features (will be set by CPUID)
            sse: self.cpuid_feature_bit(1, 1 << 25), // EDX bit 25
            sse2: self.cpuid_feature_bit(1, 1 << 26), // EDX bit 26
            sse3: self.cpuid_feature_bit(1, 1 << 0),  // ECX bit 0
            ssse3: self.cpuid_feature_bit(1, 1 << 9), // ECX bit 9
            sse4_1: self.cpuid_feature_bit(1, 1 << 19), // ECX bit 19
            sse4_2: self.cpuid_feature_bit(1, 1 << 20), // ECX bit 20
            avx: self.cpuid_feature_bit(1, 1 << 28), // ECX bit 28
            avx2: self.cpuid_feature_bit(7, 1 << 5), // EBX bit 5
            avx512: self.cpuid_feature_bit(7, 1 << 16), // EBX bit 16
            fma: self.cpuid_feature_bit(1, 1 << 12), // ECX bit 12
            bmi1: self.cpuid_feature_bit(7, 1 << 3), // EBX bit 3
            bmi2: self.cpuid_feature_bit(7, 1 << 8), // EBX bit 8
            lzcnt: self.cpuid_feature_bit(1, 1 << 5), // ECX bit 5
            popcnt: self.cpuid_feature_bit(1, 1 << 23), // POPCNT instruction
            rdtsc: true, // RDTSC always available in x86_64
            rdtscp: self.cpuid_feature_bit(1, 1 << 27), // ECX bit 27
            sysenter_sysexit: self.cpuid_feature_bit(1, 1 << 11), // EDX bit 11
            syscall: true, // SYSCALL always available in x86_64
            clflush: self.cpuid_feature_bit(1, 1 << 19), // EDX bit 19
            clflushopt: self.cpuid_feature_bit(7, 1 << 23), // EBX bit 23
            
            // Security features
            nx_bit: self.cpuid_feature_bit(1, 1 << 20), // EDX bit 20 (NX bit)
            pae: self.cpuid_feature_bit(1, 1 << 6), // EDX bit 6 (PAE)
            la57: self.cpuid_feature_bit(7, 1 << 16), // EBX bit 16 (LA57)
            smep: self.cpuid_feature_bit(7, 1 << 20), // EBX bit 20 (SMEP)
            smap: self.cpuid_feature_bit(7, 1 << 21), // EBX bit 21 (SMAP)
            pge: true, // PGE always available in x86_64
            invpcid: self.cpuid_feature_bit(7, 1 << 10), // EBX bit 10 (INVPCID)
            tsx: self.cpuid_feature_bit(7, 1 << 11), // EBX bit 11 (TSX)
            intel_pt: self.cpuid_feature_bit(7, 1 << 25), // EBX bit 25 (Intel PT)
            intel CET: false, // Will be set when CET extension is detected
            amd_sev: self.cpuid_extended_feature_bit(0, 1 << 1), // SEV support
            amd_sme: self.cpuid_extended_feature_bit(0, 1 << 0), // SME support
            
            // ARM64 features (default false for x86_64)
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
            sel2: false,
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
            
            // RISC-V features (default false for x86_64)
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
        };
        
        info!("x86_64 feature detection complete");
        Ok(features)
    }
    
    /// Detect ARM64 CPU features
    fn detect_aarch64_features(&self) -> Result<CpuFeatures, KernelError> {
        info!("Detecting ARM64 CPU features...");
        
        let mut features = CpuFeatures {
            architecture: ArchType::AArch64,
            has_64bit: true, // Always true for AArch64 target
            has_fpu: true,
            page_size: 4096,
            max_physical_addr: 0xFFFF_FFFF_FFFF, // 48-bit PA
            
            // Common features
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
            syscall: true, // SYSCALL available in ARM64
            clflush: false,
            clflushopt: false,
            
            // Security features
            nx_bit: true, // ARM64 always has NX bit
            pae: true,    // ARM64 always has PAE (64-bit)
            la57: false,  // 57-bit addressing (optional)
            smep: true,   // SMEP available in ARM64
            smap: true,   // SMAP available in ARM64
            pge: true,    // PGE available in ARM64
            invpcid: false,
            tsx: false,
            intel_pt: false,
            intel CET: false,
            amd_sev: false,
            amd_sme: false,
            
            // ARM64 SIMD and vector features
            neon: self.read_id_dfr0_bit(28), // NEON support
            fp16: self.read_id_afr0_bit(1),  // FP16 support
            asimd: true, // ASIMD always available in ARM64
            aes: self.read_id_dfr0_bit(31),  // AES extension
            sha1: self.read_id_dfr0_bit(30), // SHA1 extension
            sha256: self.read_id_dfr0_bit(29), // SHA256 extension
            sha512: self.read_id_dfr0_bit(28), // SHA512 extension (ARMv8.2)
            sm3: false, // SM3 extension (ARMv8.2)
            sm4: false, // SM4 extension (ARMv8.2)
            pmull: self.read_id_dfr0_bit(27), // PMULL extension
            crc32: self.read_id_dfr0_bit(26), // CRC32 extension
            atomics: self.read_id_afr0_bit(3), // Atomics extension
            
            // ARM64 security features
            pointer_auth: self.read_id_afr0_bit(2), // Pointer authentication
            mem_tag: self.read_id_dfr0_bit(24), // Memory tagging
            mte: self.read_id_dfr0_bit(23), // Memory tagging extension
            gcs: self.read_id_dfr0_bit(22), // Guarded control stack
            sel2: self.read_id_aa64pfr1_bit(0), // SEL2 extension
            sve: self.read_id_afr0_bit(4), // SVE extension
            sve2: self.read_id_afr0_bit(5), // SVE2 extension
            rdm: self.read_id_afr0_bit(6), // Rounding double multiply
            lse: self.read_id_afr0_bit(0), // Large System Extensions
            
            // ARM64 architecture version features
            aarch32: self.read_id_afr0_bit(7), // AArch32 support
            vfp: true, // VFP always available in ARM64
            vfp3: true, // VFP v3 always available
            vfp4: self.read_id_afr0_bit(1), // VFP v4 support
            armv8_1: self.read_id_afr0_bit(8), // ARMv8.1 features
            armv8_2: self.read_id_afr0_bit(9), // ARMv8.2 features
            armv8_3: self.read_id_afr0_bit(10), // ARMv8.3 features
            armv8_4: self.read_id_afr0_bit(11), // ARMv8.4 features
            armv8_5: self.read_id_afr0_bit(12), // ARMv8.5 features
            armv8_6: self.read_id_afr0_bit(13), // ARMv8.6 features
            armv8_7: self.read_id_afr0_bit(14), // ARMv8.7 features
            
            // RISC-V features (default false for ARM64)
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
        };
        
        info!("ARM64 feature detection complete");
        Ok(features)
    }
    
    /// Detect RISC-V CPU features
    fn detect_riscv64_features(&self) -> Result<CpuFeatures, KernelError> {
        info!("Detecting RISC-V CPU features...");
        
        // Detect standard extensions
        let mutisa = self.read_misa();
        let base_isa = (mutisa & 0x1F) as u8; // Bits 0-4: base ISA
        
        let mut features = CpuFeatures {
            architecture: ArchType::Riscv64,
            has_64bit: base_isa == 2, // RV64I
            has_fpu: (mutisa & (1 << 5)) != 0, // F extension
            page_size: 4096,
            max_physical_addr: 0xFFFF_FFFF_FFFF, // 48-bit PA
            
            // Common features
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
            syscall: true, // SYSCALL available in RISC-V
            clflush: false,
            clflushopt: false,
            
            // Security features
            nx_bit: false, // Check PMP for NX-like features
            pae: true,    // RISC-V always supports 64-bit PA
            la57: false,
            smep: false,
            smap: false,
            pge: false,
            invpcid: false,
            tsx: false,
            intel_pt: false,
            intel CET: false,
            amd_sev: false,
            amd_sme: false,
            
            // ARM64 features (default false for RISC-V)
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
            sel2: false,
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
            
            // RISC-V base ISA
            rv64i: base_isa == 2,
            rv32i: base_isa == 1,
            rv32e: base_isa == 1, // Check MISA for RV32E
            rv64e: base_isa == 2, // Check MISA for RV64E
            rvc: (mutisa & (1 << 2)) != 0, // C extension
            rva: (mutisa & (1 << 8)) != 0, // A extension (atomics)
            rvd: (mutisa & (1 << 9)) != 0, // D extension (double precision)
            rvf: (mutisa & (1 << 5)) != 0, // F extension (single precision)
            rvq: (mutisa & (1 << 21)) != 0, // Q extension (quad precision)
            rvh: (mutisa & (1 << 7)) != 0, // H extension (hypervisor)
            rvb: (mutisa & (1 << 1)) != 0, // B extension (bit manipulation)
            rvk: (mutisa & (1 << 20)) != 0, // K extension (crypto)
            
            // RISC-V standard extensions (ASCII encoding)
            m: (mutisa & (1 << 12)) != 0, // M extension
            a: (mutisa & (1 << 8)) != 0,  // A extension (atomics)
            f: (mutisa & (1 << 5)) != 0,  // F extension
            d: (mutisa & (1 << 9)) != 0,  // D extension
            q: (mutisa & (1 << 21)) != 0, // Q extension
            c: (mutisa & (1 << 2)) != 0,  // C extension
            b: (mutisa & (1 << 1)) != 0,  // B extension
            k: (mutisa & (1 << 20)) != 0, // K extension
            j: (mutisa & (1 << 13)) != 0, // J extension (dynamic translation)
            t: (mutisa & (1 << 14)) != 0, // T extension (transactional memory)
            v: (mutisa & (1 << 21)) != 0, // V extension (vector operations)
            n: (mutisa & (1 << 15)) != 0, // N extension (user-level interrupts)
            h: (mutisa & (1 << 7)) != 0,  // H extension (hypervisor)
            g: (mutisa & (1 << 20)) != 0, // G extension (general)
            p: false, // P extension (packed SIMD) - check when implemented
            
            // RISC-V security features
            pmp: self.check_riscv_security_feature(0), // Check PMP support
            pmp_grain_granule: self.get_pmp_grain_granule(),
            svpbmt: self.check_satp_features(2), // Svpbmt extension
            svadu: self.check_satp_features(3), // Svadu extension
            svinval: self.check_satp_features(4), // Svinval extension
            svnapot: self.check_satp_features(5), // Svnapot extension
            sstc: self.check_satp_features(6), // Sstc extension
            zicbom: self.check_riscv_isa_string('z', "icbom"),
            zicboz: self.check_riscv_isa_string('z', "icboz"),
            zicntr: self.check_riscv_isa_string('z', "icntr"),
            zicsr: (mutisa & (1 << 20)) != 0, // Zicsr extension
            zifencei: (mutisa & (1 << 21)) != 0, // Zifencei extension
        };
        
        info!("RISC-V feature detection complete");
        Ok(features)
    }
    
    /// Execute CPUID instruction (x86_64)
    fn cpuid_feature_bit(&self, leaf: u32, bit: u32) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            let mut eax: u32;
            let mut ebx: u32;
            let mut ecx: u32;
            let mut edx: u32;
            
            unsafe {
                core::arch::asm!(
                    "cpuid",
                    inout("eax") leaf => eax,
                    inout("ebx") 0 => ebx,
                    inout("ecx") 0 => ecx,
                    inout("edx") 0 => edx,
                );
            }
            
            match leaf {
                1 => (ecx & bit) != 0 || (edx & bit) != 0,
                7 => (ebx & bit) != 0,
                _ => false,
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false // Not available on other architectures
        }
    }
    
    /// Execute CPUID extended feature leaf (x86_64)
    fn cpuid_extended_feature_bit(&self, leaf: u32, bit: u32) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            let mut eax: u32;
            let mut ebx: u32;
            let mut ecx: u32;
            let mut edx: u32;
            
            unsafe {
                core::arch::asm!(
                    "cpuid",
                    inout("eax") leaf => eax,
                    inout("ebx") 0 => ebx,
                    inout("ecx") 0 => ecx,
                    inout("edx") 0 => edx,
                );
            }
            
            match leaf {
                0 => (ecx & bit) != 0 || (edx & bit) != 0,
                _ => false,
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false // Not available on other architectures
        }
    }
    
    /// Read ARM64 system register (ID_DFR0)
    fn read_id_dfr0_bit(&self, bit: u8) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            let mut id_dfr0: u64;
            unsafe {
                core::arch::asm!("mrs {}, s3_0_c0_c7_0", out(reg) id_dfr0);
            }
            (id_dfr0 >> bit) & 1 == 1
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            false
        }
    }
    
    /// Read ARM64 system register (ID_AFR0)
    fn read_id_afr0_bit(&self, bit: u8) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            let mut id_afr0: u64;
            unsafe {
                core::arch::asm!("mrs {}, s3_0_c3_c0_4", out(reg) id_afr0);
            }
            (id_afr0 >> bit) & 1 == 1
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            false
        }
    }
    
    /// Read ARM64 system register (ID_AA64PFR1)
    fn read_id_aa64pfr1_bit(&self, bit: u8) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            let mut id_aa64pfr1: u64;
            unsafe {
                core::arch::asm!("mrs {}, s3_0_c4_c0_1", out(reg) id_aa64pfr1);
            }
            (id_aa64pfr1 >> bit) & 1 == 1
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            false
        }
    }
    
    /// Read RISC-V MISA register
    fn read_misa(&self) -> u64 {
        #[cfg(target_arch = "riscv64")]
        {
            let mut misa: u64;
            unsafe {
                core::arch::asm!("csrr {}, 0x301", out(reg) misa);
            }
            misa
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            0
        }
    }
    
    /// Check RISC-V security feature (PMP)
    fn check_riscv_security_feature(&self, feature: u8) -> bool {
        #[cfg(target_arch = "riscv64")]
        {
            match feature {
                0 => {
                    // Check PMP support
                    let mut pmpcfg0: u64;
                    unsafe {
                        core::arch::asm!("csrr {}, 0x3A0", out(reg) pmpcfg0);
                    }
                    pmpcfg0 != 0
                },
                _ => false,
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            false
        }
    }
    
    /// Get PMP grain granularity
    fn get_pmp_grain_granule(&self) -> u8 {
        #[cfg(target_arch = "riscv64")]
        {
            // Check pmpcfg0 for grain field
            let mut pmpcfg0: u64;
            unsafe {
                core::arch::asm!("csrr {}, 0x3A0", out(reg) pmpcfg0);
            }
            
            let grain = ((pmpcfg0 >> 27) & 0xF) as u8;
            match grain {
                0 => 0,   // Grain = 0 (4 bytes)
                1 => 3,   // Grain = 1 (8 bytes)
                2 => 4,   // Grain = 2 (16 bytes)
                3 => 5,   // Grain = 3 (32 bytes)
                4 => 6,   // Grain = 4 (64 bytes)
                _ => 0,   // Unknown
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            0
        }
    }
    
    /// Check RISC-V SATP features
    fn check_satp_features(&self, feature: u8) -> bool {
        #[cfg(target_arch = "riscv64")]
        {
            let mut satp: u64;
            unsafe {
                core::arch::asm!("csrr {}, 0x180", out(reg) satp);
            }
            
            match feature {
                2 => (satp >> 60) & 0xF == 0b0010, // Svpbmt
                3 => (satp >> 60) & 0xF == 0b0011, // Svadu
                4 => (satp >> 60) & 0xF == 0b0100, // Svinval
                5 => (satp >> 60) & 0xF == 0b0101, // Svnapot
                6 => (satp >> 60) & 0xF == 0b0110, // Sstc
                _ => false,
            }
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            false
        }
    }
    
    /// Check RISC-V ISA string extensions
    fn check_riscv_isa_string(&self, prefix: char, extension: &str) -> bool {
        #[cfg(target_arch = "riscv64")]
        {
            // This would typically read from /proc/cpuinfo or similar
            // For now, return false as this requires runtime string parsing
            false
        }
        #[cfg(not(target_arch = "riscv64"))]
        {
            false
        }
    }
    
    /// Get CPU capabilities
    pub fn get_capabilities(&self) -> Result<&CapabilityInfo, KernelError> {
        if let Some(ref info) = self.capability_info {
            Ok(info)
        } else {
            Err(KernelError::NotInitialized)
        }
    }
    
    /// Get architecture type
    pub fn get_architecture(&self) -> ArchType {
        self.architecture
    }
    
    /// Get detected features
    pub fn get_features(&self) -> Option<&CpuFeatures> {
        self.detected_features.as_ref()
    }
}