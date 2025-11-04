//! CPU Manager for Multi-Core x86_64 Systems
//! 
//! Handles CPU detection, initialization, multi-core coordination,
//! and instruction set optimizations

use crate::log::{info, warn, error};
use crate::KernelError;

use super::SupportedFeatures;

/// CPU information
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub vendor: CpuVendor,
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
    pub brand_string: String,
    pub frequency_mhz: u32,
    pub core_count: u32,
    pub thread_count: u32,
    pub cache_l1_size: u32,
    pub cache_l2_size: u32,
    pub cache_l3_size: u32,
    pub supported_features: SupportedFeatures,
    pub topology: CpuTopology,
}

/// CPU vendors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CpuVendor {
    Intel,
    Amd,
    Unknown,
}

/// CPU topology information
#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub packages: u32,
    pub cores_per_package: u32,
    pub threads_per_core: u32,
    pub apic_ids: Vec<u32>,
}

/// CPU ID results structure
#[derive(Debug, Clone, Copy)]
struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// CPU state information
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    pub cpu_id: u32,
    pub apic_id: u32,
    pub is_bsp: bool,
    pub is_online: bool,
    pub current_frequency_mhz: u32,
    pub temperature_celsius: u32,
    pub usage_percent: u32,
}

/// APIC information
#[derive(Debug, Clone)]
pub struct ApicInfo {
    pub base_address: u64,
    pub version: u32,
    pub max_lvt_entries: u32,
    pub local_apic_id: u32,
    pub processor_id: u32,
}

/// Performance monitoring counters
#[derive(Debug, Clone)]
pub struct PmcInfo {
    pub version: u32,
    pub general_counters: u32,
    pub fixed_counters: u32,
    pub counter_width: u32,
    pub counter_frequency: u64,
}

/// CPU power management information
#[derive(Debug, Clone)]
pub struct CpuPowerInfo {
    pub supports_cstates: bool,
    pub supports_pstates: bool,
    pub supports_turbo_boost: bool,
    pub supports_speed_step: bool,
    pub min_frequency_mhz: u32,
    pub max_frequency_mhz: u32,
    pub base_frequency_mhz: u32,
    pub turbo_frequency_mhz: u32,
}

/// CPU Manager - handles all CPU-related operations
pub struct CpuManager {
    pub info: CpuInfo,
    pub states: Vec<CpuState>,
    pub apic_info: ApicInfo,
    pub pmc_info: PmcInfo,
    pub power_info: CpuPowerInfo,
    pub initialized: bool,
    pub bsp_id: u32,
    pub total_cpus: u32,
}

impl CpuManager {
    /// Create new CPU manager
    pub fn new() -> Self {
        Self {
            info: CpuInfo {
                vendor: CpuVendor::Unknown,
                family: 0,
                model: 0,
                stepping: 0,
                brand_string: String::new(),
                frequency_mhz: 0,
                core_count: 1,
                thread_count: 1,
                cache_l1_size: 0,
                cache_l2_size: 0,
                cache_l3_size: 0,
                supported_features: SupportedFeatures {
                    sse: false,
                    sse2: false,
                    sse3: false,
                    sse4_1: false,
                    sse4_2: false,
                    avx: false,
                    avx2: false,
                    avx512: false,
                    fma: false,
                    bmi1: false,
                    bmi2: false,
                    sha: false,
                    aes_ni: false,
                    rdrand: false,
                    rdseed: false,
                    clmul: false,
                    movbe: false,
                    popcnt: false,
                    lzcnt: false,
                    cmov: false,
                    fcmov: false,
                },
                topology: CpuTopology {
                    packages: 1,
                    cores_per_package: 1,
                    threads_per_core: 1,
                    apic_ids: Vec::new(),
                },
            },
            states: Vec::new(),
            apic_info: ApicInfo {
                base_address: 0xFEE00000,
                version: 0,
                max_lvt_entries: 0,
                local_apic_id: 0,
                processor_id: 0,
            },
            pmc_info: PmcInfo {
                version: 0,
                general_counters: 0,
                fixed_counters: 0,
                counter_width: 0,
                counter_frequency: 0,
            },
            power_info: CpuPowerInfo {
                supports_cstates: false,
                supports_pstates: false,
                supports_turbo_boost: false,
                supports_speed_step: false,
                min_frequency_mhz: 0,
                max_frequency_mhz: 0,
                base_frequency_mhz: 0,
                turbo_frequency_mhz: 0,
            },
            initialized: false,
            bsp_id: 0,
            total_cpus: 1,
        }
    }
    
    /// Initialize CPU manager
    pub fn initialize(&mut self) -> Result<(), KernelError> {
        info!("Initializing CPU Manager...");
        
        // Step 1: Detect CPU using CPUID
        self.detect_cpu()?;
        
        // Step 2: Enable CPU features and optimizations
        self.enable_features()?;
        
        // Step 3: Setup local APIC
        self.setup_local_apic()?;
        
        // Step 4: Detect multi-core topology
        self.detect_topology()?;
        
        // Step 5: Initialize performance monitoring
        self.init_performance_monitoring()?;
        
        // Step 6: Setup power management
        self.init_power_management()?;
        
        // Step 7: Initialize multi-core support
        self.init_multicore()?;
        
        self.initialized = true;
        info!("CPU Manager initialized successfully");
        
        Ok(())
    }
    
    /// Detect CPU using CPUID instruction
    fn detect_cpu(&mut self) -> Result<(), KernelError> {
        // Get CPUID function 0 - vendor identification
        let result = self.cpuid(0);
        let max_standard_function = result.eax;
        
        // Get vendor string
        let vendor_result_1 = self.cpuid(1);
        let vendor_result_2 = self.cpuid(2);
        let vendor_result_3 = self.cpuid(3);
        
        let vendor_string = format!(
            "{}{}{}",
            Self::decode_string(vendor_result_1.ebx, vendor_result_1.edx, vendor_result_1.ecx),
            Self::decode_string(vendor_result_2.eax, vendor_result_2.edx, vendor_result_2.ecx),
            Self::decode_string(vendor_result_3.eax, vendor_result_3.edx, vendor_result_3.ecx)
        );
        
        self.info.brand_string = vendor_string.clone();
        
        // Determine vendor
        self.info.vendor = match vendor_string.as_str() {
            "GenuineIntel" => CpuVendor::Intel,
            "AuthenticAMD" => CpuVendor::Amd,
            _ => CpuVendor::Unknown,
        };
        
        // Get extended CPUID function 0
        let extended_result = self.cpuid(0x80000000);
        let max_extended_function = extended_result.eax;
        
        // Get brand string if available
        if max_extended_function >= 0x80000004 {
            let brand_0 = self.cpuid(0x80000002);
            let brand_1 = self.cpuid(0x80000003);
            let brand_2 = self.cpuid(0x80000004);
            
            let brand = format!(
                "{}{}{}",
                Self::decode_string(brand_0.eax, brand_0.ebx, brand_0.ecx, brand_0.edx),
                Self::decode_string(brand_1.eax, brand_1.ebx, brand_1.ecx, brand_1.edx),
                Self::decode_string(brand_2.eax, brand_2.ebx, brand_2.ecx, brand_2.edx)
            );
            
            self.info.brand_string = brand.trim().to_string();
        }
        
        // Get CPU family and model
        if max_standard_function >= 1 {
            let result_1 = self.cpuid(1);
            self.info.family = ((result_1.eax >> 8) & 0xF) | ((result_1.eax >> 20) & 0xFF);
            self.info.model = ((result_1.eax >> 4) & 0xF) | ((result_1.eax >> 12) & 0xF0);
            self.info.stepping = result_1.eax & 0xF;
        }
        
        info!("Detected CPU: {} (Family {}, Model {}, Stepping {})", 
              self.info.brand_string, self.info.family, self.info.model, self.info.stepping);
        
        Ok(())
    }
    
    /// Enable CPU features and optimizations
    fn enable_features(&mut self) -> Result<(), KernelError> {
        if self.info.vendor == CpuVendor::Intel || self.info.vendor == CpuVendor::Amd {
            // Get feature flags from CPUID function 1
            let result_1 = self.cpuid(1);
            let ecx = result_1.ecx;
            let edx = result_1.edx;
            
            // Standard features
            self.info.supported_features.sse = (edx & (1 << 25)) != 0;
            self.info.supported_features.sse2 = (edx & (1 << 26)) != 0;
            self.info.supported_features.sse3 = (ecx & 0x1) != 0;
            self.info.supported_features.sse4_1 = (ecx & (1 << 19)) != 0;
            self.info.supported_features.sse4_2 = (ecx & (1 << 20)) != 0;
            self.info.supported_features.avx = (ecx & (1 << 28)) != 0;
            self.info.supported_features.fma = (ecx & (1 << 12)) != 0;
            self.info.supported_features.nx_bit = (edx & (1 << 20)) != 0;
            self.info.supported_features.pae = (edx & (1 << 6)) != 0;
            self.info.supported_features.pge = (edx & (1 << 13)) != 0;
            
            // Additional features from CPUID function 7, sub-leaf 0
            let result_7 = self.cpuid(7);
            if result_7.ebx > 0 {
                self.info.supported_features.avx2 = result_7.ebx & (1 << 5) != 0;
                self.info.supported_features.bmi1 = result_7.ebx & (1 << 3) != 0;
                self.info.supported_features.bmi2 = result_7.ebx & (1 << 8) != 0;
                self.info.supported_features.sha = result_7.ebx & (1 << 29) != 0;
                self.info.supported_features.movbe = result_7.ebx & (1 << 22) != 0;
                self.info.supported_features.popcnt = result_7.ebx & (1 << 23) != 0;
                
                // Extended features from ECX for function 7
                self.info.supported_features.avx512 = result_7.ecx & 0xE6 != 0;
            }
            
            // Extended features from CPUID function 0x80000001
            let extended_result = self.cpuid(0x80000001);
            let ext_ecx = extended_result.ecx;
            let ext_edx = extended_result.edx;
            
            self.info.supported_features.lzcnt = ext_ecx & (1 << 5) != 0;
            self.info.supported_features.aes_ni = ext_ecx & (1 << 1) != 0;
            self.info.supported_features.clmul = ext_ecx & (1 << 1) != 0;
            self.info.supported_features.cmov = ext_edx & (1 << 15) != 0;
            self.info.supported_features.fcmov = ext_edx & (1 << 16) != 0;
        }
        
        info!("CPU Features: {:?}", self.info.supported_features);
        
        Ok(())
    }
    
    /// Setup local APIC
    fn setup_local_apic(&mut self) -> Result<(), KernelError> {
        // Read IA32_APIC_BASE MSR to get APIC base address
        let apic_base_msr = 0x1B;
        let apic_base = self.read_msr(apic_base_msr);
        
        self.apic_info.base_address = (apic_base & 0xFFFFF000) as u64;
        self.apic_info.local_apic_id = ((apic_base >> 12) & 0xFF) as u32;
        
        // Enable APIC if not already enabled
        if (apic_base & (1 << 11)) == 0 {
            let new_apic_base = apic_base | (1 << 11);
            self.write_msr(apic_base_msr, new_apic_base);
        }
        
        info!("Local APIC initialized at 0x{:X}, ID: {}", 
              self.apic_info.base_address, self.apic_info.local_apic_id);
        
        Ok(())
    }
    
    /// Detect multi-core topology
    fn detect_topology(&mut self) -> Result<(), KernelError> {
        // Use CPUID function 1 to get logical processor count
        let result_1 = self.cpuid(1);
        let logical_cpus = ((result_1.ebx >> 16) & 0xFF) as u32;
        let cores_per_package = self.get_cores_per_package();
        
        self.info.core_count = cores_per_package;
        self.info.thread_count = logical_cpus;
        
        // Detect packages and topology using extended CPUID
        let extended_result = self.cpuid(0x80000008);
        if extended_result.ecx > 0 {
            self.info.topology.packages = (extended_result.ecx & 0xFF) as u32;
            self.info.topology.cores_per_package = cores_per_package;
            self.info.topology.threads_per_core = logical_cpus / cores_per_package;
        }
        
        info!("CPU Topology: {} packages, {} cores per package, {} threads per core",
              self.info.topology.packages, self.info.topology.cores_per_package, 
              self.info.topology.threads_per_core);
        
        Ok(())
    }
    
    /// Get cores per package
    fn get_cores_per_package() -> u32 {
        // Use CPUID function 4 for core count on Intel
        let result_4 = CpuManager::cpuid_static(4);
        ((result_4.eax >> 26) & 0x3F) as u32 + 1
    }
    
    /// Initialize performance monitoring
    fn init_performance_monitoring(&mut self) -> Result<(), KernelError> {
        // Check for performance monitoring support
        let result_0a = self.cpuid(0x0A);
        self.pmc_info.version = result_0a.eax & 0xFF;
        self.pmc_info.general_counters = (result_0a.ebx & 0xFFFF) as u32;
        self.pmc_info.fixed_counters = (result_0a.edx >> 5) & 0x3FF;
        self.pmc_info.counter_width = match self.pmc_info.version {
            2 => 32,
            3 | 4 => 40,
            _ => 32,
        };
        
        info!("Performance Monitoring: {} general, {} fixed, width {} bits",
              self.pmc_info.general_counters, self.pmc_info.fixed_counters, 
              self.pmc_info.counter_width);
        
        Ok(())
    }
    
    /// Initialize power management
    fn init_power_management(&mut self) -> Result<(), KernelError> {
        // Check for power management support using CPUID
        let result_6 = self.cpuid(6);
        
        self.power_info.supports_cstates = (result_6.eax & 0x1) != 0;
        self.power_info.supports_turbo_boost = (result_6.eax & (1 << 1)) != 0;
        self.power_info.supports_pstates = (result_6.eax & (1 << 2)) != 0;
        self.power_info.supports_speed_step = (result_6.eax & (1 << 7)) != 0;
        
        // Get frequency information
        if self.info.vendor == CpuVendor::Intel {
            // Use Intel-specific MSRs for frequency info
            let perf_status = self.read_msr(0x198);
            let perf_ctl = self.read_msr(0x199);
            
            self.power_info.max_frequency_mhz = ((perf_status >> 8) & 0xFF) as u32 * 100;
            self.power_info.base_frequency_mhz = self.info.frequency_mhz;
        }
        
        info!("Power Management: C-states: {}, P-states: {}, Turbo: {}, SpeedStep: {}",
              self.power_info.supports_cstates, self.power_info.supports_pstates,
              self.power_info.supports_turbo_boost, self.power_info.supports_speed_step);
        
        Ok(())
    }
    
    /// Initialize multi-core support
    fn init_multicore(&mut self) -> Result<(), KernelError> {
        // Get local APIC ID to determine BSP
        self.bsp_id = self.apic_info.local_apic_id;
        
        // Initialize CPU states for all logical processors
        for i in 0..self.info.thread_count {
            let state = CpuState {
                cpu_id: i,
                apic_id: i, // Simplified mapping
                is_bsp: i == 0,
                is_online: true,
                current_frequency_mhz: self.info.frequency_mhz,
                temperature_celsius: 0, // Would be read from thermal sensors
                usage_percent: 0,
            };
            self.states.push(state);
        }
        
        self.total_cpus = self.info.thread_count;
        
        info!("Multi-core initialized: {} CPUs online", self.total_cpus);
        
        Ok(())
    }
    
    /// Execute CPUID instruction
    fn cpuid(&self, function: u32) -> CpuidResult {
        let mut eax = function;
        let mut ebx: u32 = 0;
        let mut ecx: u32 = 0;
        let mut edx: u32 = 0;
        
        unsafe {
            core::arch::asm!(
                "cpuid",
                inout(eax) eax,
                inout(ebx) ebx,
                inout(ecx) ecx,
                inout(edx) edx
            );
        }
        
        CpuidResult { eax, ebx, ecx, edx }
    }
    
    /// Static CPUID for initial detection
    fn cpuid_static(function: u32) -> CpuidResult {
        let mut eax = function;
        let mut ebx: u32 = 0;
        let mut ecx: u32 = 0;
        let mut edx: u32 = 0;
        
        unsafe {
            core::arch::asm!(
                "cpuid",
                inout(eax) eax,
                inout(ebx) ebx,
                inout(ecx) ecx,
                inout(edx) edx
            );
        }
        
        CpuidResult { eax, ebx, ecx, edx }
    }
    
    /// Read MSR (Model Specific Register)
    fn read_msr(&self, msr: u32) -> u64 {
        let mut result: u64 = 0;
        
        unsafe {
            core::arch::asm!(
                "rdmsr",
                inout(reg) msr => msr,
                out(reg) result
            );
        }
        
        result
    }
    
    /// Write MSR (Model Specific Register)
    fn write_msr(&self, msr: u32, value: u64) {
        unsafe {
            core::arch::asm!(
                "wrmsr",
                in(reg) msr,
                in(reg) value
            );
        }
    }
    
    /// Decode CPUID string result
    fn decode_string(eax: u32, ebx: u32, ecx: u32, edx: u32) -> String {
        let mut bytes = Vec::new();
        
        bytes.extend_from_slice(&eax.to_le_bytes());
        bytes.extend_from_slice(&ebx.to_le_bytes());
        bytes.extend_from_slice(&ecx.to_le_bytes());
        bytes.extend_from_slice(&edx.to_le_bytes());
        
        String::from_utf8_lossy(&bytes).to_string()
    }
    
    /// Overload for 3-register variant
    fn decode_string(eax: u32, ebx: u32, ecx: u32) -> String {
        let mut bytes = Vec::new();
        
        bytes.extend_from_slice(&eax.to_le_bytes());
        bytes.extend_from_slice(&ebx.to_le_bytes());
        bytes.extend_from_slice(&ecx.to_le_bytes());
        
        String::from_utf8_lossy(&bytes).to_string()
    }
    
    /// Get current CPU frequency
    pub fn get_current_frequency(&self, cpu_id: u32) -> u32 {
        if let Some(state) = self.states.get(cpu_id as usize) {
            state.current_frequency_mhz
        } else {
            0
        }
    }
    
    /// Get CPU temperature
    pub fn get_cpu_temperature(&self, cpu_id: u32) -> u32 {
        if let Some(state) = self.states.get(cpu_id as usize) {
            state.temperature_celsius
        } else {
            0
        }
    }
    
    /// Get CPU usage
    pub fn get_cpu_usage(&self, cpu_id: u32) -> u32 {
        if let Some(state) = self.states.get(cpu_id as usize) {
            state.usage_percent
        } else {
            0
        }
    }
    
    /// Check if feature is supported
    pub fn has_feature(&self, feature_name: &str) -> bool {
        match feature_name {
            "sse" => self.info.supported_features.sse,
            "sse2" => self.info.supported_features.sse2,
            "sse3" => self.info.supported_features.sse3,
            "sse4_1" => self.info.supported_features.sse4_1,
            "sse4_2" => self.info.supported_features.sse4_2,
            "avx" => self.info.supported_features.avx,
            "avx2" => self.info.supported_features.avx2,
            "avx512" => self.info.supported_features.avx512,
            "fma" => self.info.supported_features.fma,
            "bmi1" => self.info.supported_features.bmi1,
            "bmi2" => self.info.supported_features.bmi2,
            "sha" => self.info.supported_features.sha,
            "aes_ni" => self.info.supported_features.aes_ni,
            "rdrand" => self.info.supported_features.rdrand,
            "rdseed" => self.info.supported_features.rdseed,
            "clmul" => self.info.supported_features.clmul,
            "movbe" => self.info.supported_features.movbe,
            "popcnt" => self.info.supported_features.popcnt,
            "lzcnt" => self.info.supported_features.lzcnt,
            "cmov" => self.info.supported_features.cmov,
            "fcmov" => self.info.supported_features.fcmov,
            _ => false,
        }
    }
}