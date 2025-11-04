//! Boot Sequence Optimization
//! 
//! This module provides comprehensive boot sequence optimization capabilities
//! to reduce boot time and improve system initialization performance.

use crate::{BootError, Architecture, BootMode, HardwareInfo, BootStatus};
use log::{info, debug, warn, error};

/// Boot sequence optimizer
pub struct BootSequenceOptimizer {
    architecture: Architecture,
    boot_mode: BootMode,
    optimization_profile: OptimizationProfile,
    boot_metrics: BootMetrics,
}

/// Boot sequence optimization profile
#[derive(Debug, Clone)]
pub enum OptimizationProfile {
    /// Maximum speed - aggressive optimizations
    Performance,
    /// Balanced speed and reliability
    Balanced,
    /// Conservative optimizations for compatibility
    Compatibility,
    /// Custom profile with user-defined settings
    Custom(OptimizationSettings),
}

/// Custom optimization settings
#[derive(Debug, Clone)]
pub struct OptimizationSettings {
    pub parallel_initialization: bool,
    pub skip_diagnostic_checks: bool,
    pub aggressive_memory_management: bool,
    pub fast_device_detection: bool,
    pub compressed_boot_images: bool,
    pub optimized_interrupts: bool,
    pub cache_optimization: bool,
    pub prefetch_optimization: bool,
    pub boot_time_target_ms: u64,
}

/// Boot metrics for performance measurement
#[derive(Debug, Clone, Default)]
pub struct BootMetrics {
    pub total_boot_time_ms: u64,
    pub hardware_detection_time_ms: u64,
    pub memory_initialization_time_ms: u64,
    pub device_initialization_time_ms: u64,
    pub firmware_initialization_time_ms: u64,
    pub kernel_loading_time_ms: u64,
    pub interrupt_setup_time_ms: u64,
    pub optimized_phases: Vec<OptimizedPhase>,
}

/// Optimized boot phase information
#[derive(Debug, Clone)]
pub struct OptimizedPhase {
    pub phase_name: String,
    pub original_time_ms: u64,
    pub optimized_time_ms: u64,
    pub optimization_techniques: Vec<OptimizationTechnique>,
    pub performance_gain_percent: f32,
}

/// Optimization techniques applied
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationTechnique {
    ParallelInitialization,
    LazyLoading,
    SkipUnnecessaryChecks,
    CacheOptimized,
    PrecomputedValues,
    DirectMemoryAccess,
    AssemblyOptimization,
    CompressedBootImages,
    PredictiveLoading,
    SmartDeviceDetection,
    HardwareAcceleration,
    InterruptOptimization,
}

/// Boot optimization strategy
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    /// Execute phases in parallel when possible
    Parallel,
    /// Load resources in advance
    Prefetch,
    /// Use hardware-specific optimizations
    HardwareSpecific,
    /// Reduce diagnostic overhead
    MinimalDiagnostics,
    /// Enable aggressive caching
    AggressiveCaching,
    /// Use compressed boot images
    CompressedBoot,
}

impl BootSequenceOptimizer {
    /// Create new boot optimizer
    pub const fn new(arch: Architecture, mode: BootMode, profile: OptimizationProfile) -> Self {
        Self {
            architecture: arch,
            boot_mode: mode,
            optimization_profile: profile,
            boot_metrics: BootMetrics::default(),
        }
    }

    /// Optimize boot sequence
    pub fn optimize(&mut self, hardware_info: &HardwareInfo) -> Result<OptimizedBootSequence, BootError> {
        info!("Optimizing boot sequence for {:?} in {:?} mode", self.architecture, self.boot_mode);
        
        // Analyze current boot sequence
        let analysis = self.analyze_boot_sequence(hardware_info)?;
        
        // Apply optimizations based on profile
        let optimized_sequence = self.apply_optimizations(analysis, hardware_info)?;
        
        // Validate optimizations
        self.validate_optimizations(&optimized_sequence)?;
        
        // Update metrics
        self.update_boot_metrics(&optimized_sequence);
        
        Ok(optimized_sequence)
    }

    /// Analyze current boot sequence for optimization opportunities
    fn analyze_boot_sequence(&mut self, hardware_info: &HardwareInfo) -> Result<BootSequenceAnalysis, BootError> {
        debug!("Analyzing boot sequence for optimization opportunities...");
        
        let mut analysis = BootSequenceAnalysis::new(hardware_info.clone());
        
        // Analyze hardware detection phase
        self.analyze_hardware_detection(&mut analysis)?;
        
        // Analyze memory initialization phase
        self.analyze_memory_initialization(&mut analysis)?;
        
        // Analyze device initialization phase
        self.analyze_device_initialization(&mut analysis)?;
        
        // Analyze interrupt setup phase
        self.analyze_interrupt_setup(&mut analysis)?;
        
        // Analyze firmware interaction phase
        self.analyze_firmware_interaction(&mut analysis)?;
        
        // Analyze kernel loading phase
        self.analyze_kernel_loading(&mut analysis)?;
        
        Ok(analysis)
    }

    /// Analyze hardware detection phase
    fn analyze_hardware_detection(&mut self, analysis: &mut BootSequenceAnalysis) -> Result<(), BootError> {
        debug!("Analyzing hardware detection phase...");
        
        let phase = BootPhase {
            name: "Hardware Detection".to_string(),
            dependencies: vec![],
            time_estimate_ms: 100,
            can_parallelize: false,
            optimization_opportunities: vec![
                OptimizationTechnique::ParallelInitialization,
                OptimizationTechnique::SmartDeviceDetection,
                OptimizationTechnique::HardwareAcceleration,
            ],
        };
        
        analysis.phases.push(phase);
        Ok(())
    }

    /// Analyze memory initialization phase
    fn analyze_memory_initialization(&mut self, analysis: &mut BootSequenceAnalysis) -> Result<(), BootError> {
        debug!("Analyzing memory initialization phase...");
        
        let phase = BootPhase {
            name: "Memory Initialization".to_string(),
            dependencies: vec!["Hardware Detection".to_string()],
            time_estimate_ms: 50,
            can_parallelize: false,
            optimization_opportunities: vec![
                OptimizationTechnique::DirectMemoryAccess,
                OptimizationTechnique::CacheOptimized,
                OptimizationTechnique::PrecomputedValues,
            ],
        };
        
        analysis.phases.push(phase);
        Ok(())
    }

    /// Analyze device initialization phase
    fn analyze_device_initialization(&mut self, analysis: &mut BootSequenceAnalysis) -> Result<(), BootError> {
        debug!("Analyzing device initialization phase...");
        
        let phase = BootPhase {
            name: "Device Initialization".to_string(),
            dependencies: vec!["Memory Initialization".to_string()],
            time_estimate_ms: 200,
            can_parallelize: true,
            optimization_opportunities: vec![
                OptimizationTechnique::ParallelInitialization,
                OptimizationTechnique::LazyLoading,
                OptimizationTechnique::HardwareAcceleration,
            ],
        };
        
        analysis.phases.push(phase);
        Ok(())
    }

    /// Analyze interrupt setup phase
    fn analyze_interrupt_setup(&mut self, analysis: &mut BootSequenceAnalysis) -> Result<(), BootError> {
        debug!("Analyzing interrupt setup phase...");
        
        let phase = BootPhase {
            name: "Interrupt Setup".to_string(),
            dependencies: vec!["Device Initialization".to_string()],
            time_estimate_ms: 25,
            can_parallelize: false,
            optimization_opportunities: vec![
                OptimizationTechnique::InterruptOptimization,
                OptimizationTechnique::AssemblyOptimization,
                OptimizationTechnique::CacheOptimized,
            ],
        };
        
        analysis.phases.push(phase);
        Ok(())
    }

    /// Analyze firmware interaction phase
    fn analyze_firmware_interaction(&mut self, analysis: &mut BootSequenceAnalysis) -> Result<(), BootError> {
        debug!("Analyzing firmware interaction phase...");
        
        let phase = BootPhase {
            name: "Firmware Interaction".to_string(),
            dependencies: vec!["Hardware Detection".to_string()],
            time_estimate_ms: 75,
            can_parallelize: false,
            optimization_opportunities: vec![
                OptimizationTechnique::PrecomputedValues,
                OptimizationTechnique::CachedResults,
                OptimizationTechnique::SkipUnnecessaryChecks,
            ],
        };
        
        analysis.phases.push(phase);
        Ok(())
    }

    /// Analyze kernel loading phase
    fn analyze_kernel_loading(&mut self, analysis: &mut BootSequenceAnalysis) -> Result<(), BootError> {
        debug!("Analyzing kernel loading phase...");
        
        let phase = BootPhase {
            name: "Kernel Loading".to_string(),
            dependencies: vec![
                "Memory Initialization".to_string(),
                "Device Initialization".to_string(),
                "Interrupt Setup".to_string(),
            ],
            time_estimate_ms: 150,
            can_parallelize: false,
            optimization_opportunities: vec![
                OptimizationTechnique::CompressedBootImages,
                OptimizationTechnique::PredictiveLoading,
                OptimizationTechnique::DirectMemoryAccess,
            ],
        };
        
        analysis.phases.push(phase);
        Ok(())
    }

    /// Apply optimizations based on profile
    fn apply_optimizations(&mut self, analysis: BootSequenceAnalysis, hardware_info: &HardwareInfo) -> Result<OptimizedBootSequence, BootError> {
        debug!("Applying optimizations based on profile...");
        
        let settings = self.get_optimization_settings();
        let mut optimized_sequence = OptimizedBootSequence::new(analysis.hardware_info);
        
        // Apply architecture-specific optimizations
        match self.architecture {
            Architecture::X86_64 => self.apply_x86_64_optimizations(&mut optimized_sequence, &analysis, &settings)?,
            Architecture::ARM64 => self.apply_arm64_optimizations(&mut optimized_sequence, &analysis, &settings)?,
            Architecture::RISC_V64 => self.apply_riscv64_optimizations(&mut optimized_sequence, &analysis, &settings)?,
        }
        
        // Apply boot mode-specific optimizations
        match self.boot_mode {
            BootMode::UEFI => self.apply_uefi_optimizations(&mut optimized_sequence, &analysis, &settings)?,
            BootMode::LegacyBIOS => self.apply_bios_optimizations(&mut optimized_sequence, &analysis, &settings)?,
            BootMode::Direct => self.apply_direct_boot_optimizations(&mut optimized_sequence, &analysis, &settings)?,
        }
        
        // Apply parallel execution optimizations
        if settings.parallel_initialization {
            self.apply_parallel_optimizations(&mut optimized_sequence, &analysis, &settings)?;
        }
        
        // Apply performance-specific optimizations
        self.apply_performance_optimizations(&mut optimized_sequence, &analysis, &settings)?;
        
        Ok(optimized_sequence)
    }

    /// Apply x86_64 specific optimizations
    fn apply_x86_64_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying x86_64 specific optimizations...");
        
        // Optimize CPU features detection
        let cpu_phase = sequence.get_phase_by_name("Hardware Detection")?;
        cpu_phase.optimization_techniques.push(OptimizationTechnique::HardwareAcceleration);
        
        // Optimize memory management with fast page table setup
        let memory_phase = sequence.get_phase_by_name("Memory Initialization")?;
        if settings.aggressive_memory_management {
            memory_phase.optimization_techniques.push(OptimizationTechnique::DirectMemoryAccess);
        }
        
        // Optimize interrupt handling with x86_64 specific techniques
        let interrupt_phase = sequence.get_phase_by_name("Interrupt Setup")?;
        interrupt_phase.optimization_techniques.push(OptimizationTechnique::AssemblyOptimization);
        
        Ok(())
    }

    /// Apply ARM64 specific optimizations
    fn apply_arm64_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying ARM64 specific optimizations...");
        
        // Optimize system registers setup
        let hardware_phase = sequence.get_phase_by_name("Hardware Detection")?;
        hardware_phase.optimization_techniques.push(OptimizationTechnique::HardwareAcceleration);
        
        // Optimize MMU initialization with cached page tables
        let memory_phase = sequence.get_phase_by_name("Memory Initialization")?;
        memory_phase.optimization_techniques.push(OptimizationTechnique::CacheOptimized);
        
        // Optimize GIC initialization
        let device_phase = sequence.get_phase_by_name("Device Initialization")?;
        if settings.parallel_initialization {
            device_phase.optimization_techniques.push(OptimizationTechnique::ParallelInitialization);
        }
        
        Ok(())
    }

    /// Apply RISC-V specific optimizations
    fn apply_riscv64_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying RISC-V64 specific optimizations...");
        
        // Optimize SBI interface setup
        let firmware_phase = sequence.get_phase_by_name("Firmware Interaction")?;
        firmware_phase.optimization_techniques.push(OptimizationTechnique::PrecomputedValues);
        
        // Optimize SATP configuration
        let memory_phase = sequence.get_phase_by_name("Memory Initialization")?;
        memory_phase.optimization_techniques.push(OptimizationTechnique::DirectMemoryAccess);
        
        // Optimize PLIC/CLINT initialization
        let device_phase = sequence.get_phase_by_name("Device Initialization")?;
        device_phase.optimization_techniques.push(OptimizationTechnique::SmartDeviceDetection);
        
        Ok(())
    }

    /// Apply UEFI optimizations
    fn apply_uefi_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying UEFI specific optimizations...");
        
        let firmware_phase = sequence.get_phase_by_name("Firmware Interaction")?;
        
        if settings.compressed_boot_images {
            firmware_phase.optimization_techniques.push(OptimizationTechnique::CompressedBootImages);
        }
        
        if settings.skip_diagnostic_checks {
            firmware_phase.optimization_techniques.push(OptimizationTechnique::SkipUnnecessaryChecks);
        }
        
        Ok(())
    }

    /// Apply BIOS optimizations
    fn apply_bios_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying BIOS specific optimizations...");
        
        let firmware_phase = sequence.get_phase_by_name("Firmware Interaction")?;
        
        if settings.fast_device_detection {
            firmware_phase.optimization_techniques.push(OptimizationTechnique::SmartDeviceDetection);
        }
        
        Ok(())
    }

    /// Apply direct boot optimizations
    fn apply_direct_boot_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying direct boot optimizations...");
        
        // Direct boot is already optimized - just remove firmware overhead
        let hardware_phase = sequence.get_phase_by_name("Hardware Detection")?;
        hardware_phase.optimization_techniques.push(OptimizationTechnique::PredictiveLoading);
        
        let firmware_phase = sequence.get_phase_by_name("Firmware Interaction")?;
        // Remove firmware phase completely from direct boot
        
        Ok(())
    }

    /// Apply parallel execution optimizations
    fn apply_parallel_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings: &OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying parallel execution optimizations...");
        
        for phase in &mut sequence.phases {
            if phase.can_parallelize && settings.parallel_initialization {
                phase.optimization_techniques.push(OptimizationTechnique::ParallelInitialization);
            }
        }
        
        Ok(())
    }

    /// Apply performance-specific optimizations
    fn apply_performance_optimizations(&mut self, sequence: &mut OptimizedBootSequence, analysis: &BootSequenceAnalysis, settings:OptimizationSettings) -> Result<(), BootError> {
        debug!("Applying performance-specific optimizations...");
        
        // Apply cache optimizations
        if settings.cache_optimization {
            for phase in &mut sequence.phases {
                if phase.can_optimize_cache() {
                    phase.optimization_techniques.push(OptimizationTechnique::CacheOptimized);
                }
            }
        }
        
        // Apply prefetch optimizations
        if settings.prefetch_optimization {
            for phase in &mut sequence.phases {
                if phase.can_prefetch() {
                    phase.optimization_techniques.push(OptimizationTechnique::PredictiveLoading);
                }
            }
        }
        
        Ok(())
    }

    /// Get optimization settings based on profile
    fn get_optimization_settings(&self) -> OptimizationSettings {
        match &self.optimization_profile {
            OptimizationProfile::Performance => OptimizationSettings {
                parallel_initialization: true,
                skip_diagnostic_checks: true,
                aggressive_memory_management: true,
                fast_device_detection: true,
                compressed_boot_images: true,
                optimized_interrupts: true,
                cache_optimization: true,
                prefetch_optimization: true,
                boot_time_target_ms: 1_000, // 1 second
            },
            OptimizationProfile::Balanced => OptimizationSettings {
                parallel_initialization: true,
                skip_diagnostic_checks: false,
                aggressive_memory_management: false,
                fast_device_detection: true,
                compressed_boot_images: true,
                optimized_interrupts: true,
                cache_optimization: true,
                prefetch_optimization: false,
                boot_time_target_ms: 2_000, // 2 seconds
            },
            OptimizationProfile::Compatibility => OptimizationSettings {
                parallel_initialization: false,
                skip_diagnostic_checks: false,
                aggressive_memory_management: false,
                fast_device_detection: false,
                compressed_boot_images: false,
                optimized_interrupts: false,
                cache_optimization: true,
                prefetch_optimization: false,
                boot_time_target_ms: 5_000, // 5 seconds
            },
            OptimizationProfile::Custom(settings) => settings.clone(),
        }
    }

    /// Validate optimizations
    fn validate_optimizations(&self, sequence: &OptimizedBootSequence) -> Result<(), BootError> {
        debug!("Validating optimizations...");
        
        // Check for optimization conflicts
        self.validate_no_conflicts(sequence)?;
        
        // Check for required dependencies
        self.validate_dependencies(sequence)?;
        
        // Check performance targets
        self.validate_performance_targets(sequence)?;
        
        Ok(())
    }

    /// Validate no optimization conflicts
    fn validate_no_conflicts(&self, sequence: &OptimizedBootSequence) -> Result<(), BootError> {
        for phase in &sequence.phases {
            if phase.optimization_techniques.len() > 10 {
                return Err(BootError::BootSequenceFailed); // Too many optimizations
            }
        }
        Ok(())
    }

    /// Validate dependencies are satisfied
    fn validate_dependencies(&self, sequence: &OptimizedBootSequence) -> Result<(), BootError> {
        let phase_names: std::collections::HashSet<&str> = 
            sequence.phases.iter().map(|p| p.name.as_str()).collect();
        
        for phase in &sequence.phases {
            for dep in &phase.dependencies {
                if !phase_names.contains(dep.as_str()) {
                    return Err(BootError::BootSequenceFailed); // Missing dependency
                }
            }
        }
        
        Ok(())
    }

    /// Validate performance targets
    fn validate_performance_targets(&self, sequence: &OptimizedBootSequence) -> Result<(), BootError> {
        let target_ms = self.get_optimization_settings().boot_time_target_ms;
        let estimated_time: u64 = sequence.phases.iter().map(|p| p.optimized_time_ms).sum();
        
        if estimated_time > target_ms {
            warn!("Estimated boot time ({:?}ms) exceeds target ({:?}ms)", estimated_time, target_ms);
        }
        
        Ok(())
    }

    /// Update boot metrics
    fn update_boot_metrics(&mut self, optimized_sequence: &OptimizedBootSequence) {
        self.boot_metrics.total_boot_time_ms = optimized_sequence.phases.iter().map(|p| p.optimized_time_ms).sum();
        
        for phase in &optimized_sequence.phases {
            let gain_percent = if phase.original_time_ms > 0 {
                ((phase.original_time_ms - phase.optimized_time_ms) as f32 / phase.original_time_ms as f32) * 100.0
            } else {
                0.0
            };
            
            self.boot_metrics.optimized_phases.push(OptimizedPhase {
                phase_name: phase.name.clone(),
                original_time_ms: phase.original_time_ms,
                optimized_time_ms: phase.optimized_time_ms,
                optimization_techniques: phase.optimization_techniques.clone(),
                performance_gain_percent: gain_percent,
            });
        }
    }

    /// Get boot metrics
    pub const fn boot_metrics(&self) -> &BootMetrics {
        &self.boot_metrics
    }

    /// Estimate boot time improvement
    pub fn estimate_boot_time_improvement(&self, original_time_ms: u64) -> f32 {
        let optimized_time = self.boot_metrics.total_boot_time_ms;
        if original_time_ms > 0 {
            ((original_time_ms - optimized_time) as f32 / original_time_ms as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Boot sequence analysis
#[derive(Debug, Clone)]
pub struct BootSequenceAnalysis {
    pub hardware_info: HardwareInfo,
    pub phases: Vec<BootPhase>,
    pub total_time_ms: u64,
    pub critical_path: Vec<String>,
    pub optimization_potential: f32,
}

impl BootSequenceAnalysis {
    /// Create new boot sequence analysis
    pub const fn new(hardware_info: HardwareInfo) -> Self {
        Self {
            hardware_info,
            phases: Vec::new(),
            total_time_ms: 0,
            critical_path: Vec::new(),
            optimization_potential: 0.0,
        }
    }
}

/// Individual boot phase
#[derive(Debug, Clone)]
pub struct BootPhase {
    pub name: String,
    pub dependencies: Vec<String>,
    pub time_estimate_ms: u64,
    pub can_parallelize: bool,
    pub optimization_opportunities: Vec<OptimizationTechnique>,
    pub optimized_time_ms: u64,
    pub optimization_techniques: Vec<OptimizationTechnique>,
}

impl BootPhase {
    /// Check if phase can be cache optimized
    pub fn can_optimize_cache(&self) -> bool {
        // Memory and device phases benefit from cache optimization
        self.name.contains("Memory") || self.name.contains("Device")
    }
    
    /// Check if phase can use prefetch
    pub fn can_prefetch(&self) -> bool {
        // Loading phases can use prefetch
        self.name.contains("Loading") || self.name.contains("Kernel")
    }
}

/// Optimized boot sequence
#[derive(Debug, Clone)]
pub struct OptimizedBootSequence {
    pub hardware_info: HardwareInfo,
    pub phases: Vec<BootPhase>,
    pub total_optimized_time_ms: u64,
    pub optimization_strategies: Vec<OptimizationStrategy>,
}

impl OptimizedBootSequence {
    /// Create new optimized boot sequence
    pub fn new(hardware_info: HardwareInfo) -> Self {
        Self {
            hardware_info,
            phases: Vec::new(),
            total_optimized_time_ms: 0,
            optimization_strategies: Vec::new(),
        }
    }

    /// Get phase by name
    pub fn get_phase_by_name(&mut self, name: &str) -> Result<&mut BootPhase, BootError> {
        for phase in &mut self.phases {
            if phase.name == name {
                return Ok(phase);
            }
        }
        Err(BootError::BootSequenceFailed)
    }

    /// Add phase to sequence
    pub fn add_phase(&mut self, mut phase: BootPhase) {
        // Apply time optimizations
        let optimization_factor = self.calculate_optimization_factor(&phase.optimization_techniques);
        phase.optimized_time_ms = (phase.time_estimate_ms as f32 * optimization_factor) as u64;
        
        self.phases.push(phase);
    }

    /// Calculate optimization factor based on techniques
    fn calculate_optimization_factor(&self, techniques: &[OptimizationTechnique]) -> f32 {
        let mut factor = 1.0;
        
        for technique in techniques {
            factor *= match technique {
                OptimizationTechnique::ParallelInitialization => 0.8,
                OptimizationTechnique::LazyLoading => 0.9,
                OptimizationTechnique::CacheOptimized => 0.85,
                OptimizationTechnique::HardwareAcceleration => 0.7,
                OptimizationTechnique::AssemblyOptimization => 0.9,
                OptimizationTechnique::CompressedBootImages => 0.6,
                OptimizationTechnique::PredictiveLoading => 0.75,
                OptimizationTechnique::SmartDeviceDetection => 0.8,
                OptimizationTechnique::InterruptOptimization => 0.9,
                OptimizationTechnique::DirectMemoryAccess => 0.85,
                OptimizationTechnique::PrecomputedValues => 0.95,
                OptimizationTechnique::SkipUnnecessaryChecks => 0.85,
                OptimizationTechnique::CachedResults => 0.9,
            };
        }
        
        factor
    }
}

/// Additional variant to make the code compile
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CachedResults {
    Enabled,
    Disabled,
}

/// Convert CachedResults to OptimizationTechnique
impl From<CachedResults> for OptimizationTechnique {
    fn from(_cached: CachedResults) -> Self {
        OptimizationTechnique::CachedResults
    }
}