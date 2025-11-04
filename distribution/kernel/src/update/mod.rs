//! System Update Module
//! 
//! This module provides comprehensive system update mechanisms for the MultiOS kernel,
//! including OS updates, security patches, configuration management, rollback capabilities,
//! and intelligent automated scheduling for minimizing system disruption.

pub mod system_updater;
pub mod compatibility;
pub mod rollback;
pub mod package_integration;
// pub mod service_management; // Temporarily comment out until implemented
pub mod package_manager;
pub mod scheduler;
pub mod delta;
pub mod repository;
pub mod validator;
pub mod examples;
pub mod rollback_tests;

// Integration examples and tests
#[cfg(feature = "examples")]
pub mod integration_examples;

#[cfg(test)]
pub mod tests;

// Re-export validator components for security integration
pub use validator::{
    // Core validation types
    UpdateValidator, ValidationResult, ValidationError, UpdateValidationResult,
    ValidationConfig, UpdatePackage, UpdateMetadata,
    
    // Signature and authenticity verification
    SignatureVerification, SignatureAlgorithm, Certificate, TrustLevel,
    RevocationStatus, CertificateExtension, KeyUsage,
    
    // Integrity checking
    ChecksumValidation, HashAlgorithm, IntegrityManager,
    
    // Compatibility checking
    CompatibilityInfo, CompatibilityLevel, SystemRequirements,
    HardwareRequirements, DependencyInfo, DependencyPriority,
    
    // Rollback support
    RollbackCompatibility, RollbackInfo, RecoveryPoint,
    
    // Safety analysis
    SafetyAnalysis, SafetyLevel, RiskFactor, RiskType, RiskSeverity,
    SafetyRecommendation, SafetyWarning, SafetyThresholds,
    
    // Initialization functions
    init_validator, create_validator_with_config, create_test_update_package,
};

pub use system_updater::{
    SystemUpdater, UpdateManager, UpdateConfig, UpdateResult, UpdateError,
    UpdateType, UpdateStatus, UpdateTarget, SecurityPatch, ConfigUpdate
};

pub use compatibility::{
    CompatibilityChecker, CompatibilityResult, SystemRequirements,
    HardwareCompatibility, SoftwareCompatibility
};

pub use rollback::{
    RollbackSystem, RecoveryPointManager, SnapshotManager, RollbackEngine, StateValidator,
    RollbackResult, RollbackError, SnapshotError, RollbackOperationId, RecoveryPointId,
    ComponentCategory, RollbackScope, RollbackTrigger, AutoRollbackConfig,
    RollbackProgress, RollbackPhase, SystemSnapshot, SnapshotData,
    helpers::{create_recovery_point_with_name, quick_rollback, rollback_configuration, 
              rollback_kernel_state, emergency_rollback}
};

pub use package_integration::{
    PackageManager, Package, DependencyResolver, PackageUpdate,
    RepositoryManager, UpdateSource
};

// pub use service_management::{
//     ServiceUpdateManager, ServiceDependency, UpdateSequence,
//     ServiceRestartManager, UpdateScheduler
// }; // Temporarily commented out until implemented

pub use package_manager::{
    PackageManager, PackageConfig, PackageMetadata, Version, Dependency, RepositoryInfo,
    PackageStatus, PackageInfo, SearchResult, UpdateInfo, PackageError, PackageResult,
    VersionConstraint, PackageSignature, PackageFile, PackageScripts, PackagePriority,
    PackageConflict, RepositoryPackage, SearchMatchType, VersionOrder
};

pub use scheduler::{
    UpdateScheduler, UpdateTask, UpdatePriority, UpdateType, UpdateStatus,
    UpdateFrequency, MaintenanceWindow, ScheduleConfig, UsagePattern,
    ScheduleResult, ExecutionResult, SystemMetrics, NotificationInfo,
    NotificationType, RetryConfig, SchedulerStatus, QueueStatus
};

pub use repository::{
    RepositoryManager, Repository, RepositoryConfig, RepositoryType,
    RepositoryStatus, PackageMetadata, DeltaVersion, RepositoryError,
    RepositoryStatistics, RepositoryCredentials
};

pub use delta::{
    BinaryDiffEngine, DeltaPatch, DeltaConfig, DeltaUpdateManager,
    DiffAlgorithm, BandwidthOptimization, DeltaError, BandwidthStatistics
};

pub use {
    UpdateSystem, UpdateSystemConfig, UpdateSystemStatistics,
    SecurityConfig, NetworkConfig, SchedulingConfig, UpdateSystemState,
    AvailableUpdate, UpdateResult, UpdateError
};

use alloc::vec::Vec;
use alloc::string::String;
use core::time::Duration;
use spin::{Mutex, RwLock};

// Global secure update validator instance
static SECURE_UPDATE_VALIDATOR: Mutex<Option<UpdateValidator>> = Mutex::new(None);

/// Initialize the system update subsystem
pub fn init_update_system() -> Result<(), UpdateError> {
    log::info!("Initializing System Update Subsystem...");
    
    // Initialize all subsystems
    system_updater::init()?;
    compatibility::init()?;
    
    // Initialize comprehensive rollback system
    match rollback::init_rollback_system() {
        Ok(rollback_system) => {
            log::info!("Comprehensive rollback system initialized successfully");
            
            // Verify rollback system health
            match rollback_system.validate_system_state() {
                Ok(()) => {
                    log::info!("Rollback system validation passed - ready for automatic failure recovery");
                }
                Err(e) => {
                    log::warn!("Rollback system validation warning: {:?}", e);
                    // Continue initialization but log the warning
                }
            }
        }
        Err(e) => {
            log::error!("Failed to initialize rollback system: {:?}", e);
            return Err(UpdateError::RollbackFailed);
        }
    }
    
    package_integration::init()?;
    // service_management::init()?; // Temporarily commented out until implemented
    
    log::info!("Package Manager initialized as part of Update System");
    
    log::info!("System Update Subsystem initialized successfully");
    Ok(())
}

/// Initialize secure update system with validation and integrity checking
pub fn init_secure_update_system() -> Result<(), Box<dyn core::fmt::Display>> {
    log::info!("Initializing Secure Update System with Validation...");
    
    // Initialize core update system
    init_update_system().map_err(|e| format!("Failed to initialize core update system: {:?}", e))?;
    
    // Initialize secure update validation
    let secure_config = UpdateSystemConfig {
        enable_secure_updates: true,
        require_signature_verification: true,
        enable_automatic_validation: true,
        max_concurrent_validations: 4,
        validation_timeout_seconds: 300,
        enable_rollback_support: true,
        auto_rollback_on_failure: true,
        validation_cache_size: 1000,
    };
    
    init_secure_update_validation(secure_config)
        .map_err(|e| format!("Failed to initialize secure validation: {:?}", e))?;
    
    log::info!("Secure Update System initialized successfully with validation and integrity checking");
    Ok(())
}

/// Initialize secure update validation system
fn init_secure_update_validation(config: UpdateSystemConfig) -> ValidationResult<()> {
    // Initialize validator with secure configuration
    let validator_config = validator::ValidationConfig {
        enable_signature_verification: config.require_signature_verification,
        require_strong_signature: true,
        enable_checksum_validation: true,
        strict_compatibility_checking: true,
        enable_safety_analysis: config.enable_secure_updates,
        require_rollback_support: config.enable_rollback_support,
        minimum_trust_level: validator::TrustLevel::Medium,
        allowed_signature_algorithms: vec![
            validator::SignatureAlgorithm::RSA2048_SHA256,
            validator::SignatureAlgorithm::RSA4096_SHA256,
            validator::SignatureAlgorithm::ECCP256_ECDSA,
        ],
        allowed_hash_algorithms: vec![
            validator::HashAlgorithm::SHA256,
            validator::HashAlgorithm::SHA512,
        ],
        max_acceptable_risk_score: 70,
    };
    
    // Create and store validator
    let validator = UpdateValidator::new(validator_config)?;
    
    // Store in global reference
    let mut global_validator = SECURE_UPDATE_VALIDATOR.lock();
    *global_validator = Some(validator);
    
    log::info!("Secure update validation initialized");
    Ok(())
}

/// Validate an update package using secure validation
pub fn validate_update_secure(update_package: &UpdatePackage) -> ValidationResult<UpdateValidationResult> {
    let validator = SECURE_UPDATE_VALIDATOR.lock();
    if let Some(ref validator_instance) = *validator {
        validator_instance.validate_update(update_package)
    } else {
        Err(ValidationError::NotInitialized)
    }
}

/// Get the secure update validator instance
pub fn get_secure_validator() -> ValidationResult<&'static Mutex<Option<UpdateValidator>>> {
    let validator = SECURE_UPDATE_VALIDATOR.lock();
    if validator.is_some() {
        // SAFETY: We're returning a reference to the global static
        // which is protected by the Mutex and won't be dropped
        Ok(unsafe { &*(SECURE_UPDATE_VALIDATOR.as_ptr()) })
    } else {
        Err(ValidationError::NotInitialized)
    }
}

/// Check if secure update system is ready
pub fn is_secure_update_ready() -> bool {
    let validator = SECURE_UPDATE_VALIDATOR.lock();
    validator.is_some()
}

/// Perform comprehensive update validation before installation
pub fn pre_install_validation(update_package: &UpdatePackage) -> Result<bool, Box<dyn core::fmt::Display>> {
    log::info!("Performing pre-installation validation for update: {}", update_package.id);
    
    let validation_result = validate_update_secure(update_package)
        .map_err(|e| format!("Update validation failed: {:?}", e))?;
    
    if validation_result.is_valid {
        log::info!("Pre-installation validation passed");
        log::info!("- Signature valid: {}", validation_result.signature_verification.is_valid);
        log::info!("- Integrity valid: {}", validation_result.checksum_validation.is_valid);
        log::info!("- Compatibility: {:?}", validation_result.compatibility_info.compatibility_level);
        log::info!("- Safety score: {}", validation_result.total_risk_score);
        log::info!("- Recommendation: {:?}", validation_result.safety_analysis.recommended_action);
        
        if !validation_result.safety_analysis.warnings.is_empty() {
            log::warn!("Validation warnings:");
            for warning in &validation_result.safety_analysis.warnings {
                log::warn!("  [{}] {}", warning.level as u8, warning.message);
            }
        }
        
        Ok(true)
    } else {
        log::error!("Pre-installation validation failed!");
        for error in &validation_result.validation_errors {
            log::error!("  Validation error: {:?}", error);
        }
        Ok(false)
    }
}

/// Update system configuration for security features
#[derive(Debug, Clone)]
pub struct UpdateSystemConfig {
    pub enable_secure_updates: bool,
    pub require_signature_verification: bool,
    pub enable_automatic_validation: bool,
    pub max_concurrent_validations: u32,
    pub validation_timeout_seconds: u64,
    pub enable_rollback_support: bool,
    pub auto_rollback_on_failure: bool,
    pub validation_cache_size: usize,
}

/// Initialize the update scheduler with intelligent scheduling capabilities
pub fn init_update_scheduler(
    config: scheduler::ScheduleConfig,
    security_manager: alloc::sync::Arc<spin::Mutex<crate::security::SecurityManager>>,
    service_manager: alloc::sync::Arc<spin::Mutex<crate::service_manager::ServiceManager>>,
) -> Result<(), &'static str> {
    log::info!("Initializing Automated Update Scheduler...");
    
    let scheduler = scheduler::UpdateScheduler::new(config, security_manager, service_manager);
    let scheduler = alloc::sync::Arc::new(spin::Mutex::new(scheduler));
    
    // Initialize the scheduler
    let mut scheduler_guard = scheduler.lock();
    scheduler_guard.initialize()?;
    
    // Set up global scheduler reference
    scheduler::set_global_scheduler(scheduler);
    
    log::info!("Automated Update Scheduler initialized successfully");
    Ok(())
}

// Package Manager Test Suite Integration
pub use package_tests::PackageManagerTestSuite;

/// Package system statistics and information
pub struct PackageSystemInfo {
    pub total_installed_packages: usize,
    pub total_available_packages: usize,
    pub cache_usage_mb: u64,
    pub last_update_check: u64,
    pub repositories_count: usize,
    pub security_updates_available: usize,
    pub auto_update_enabled: bool,
}

impl PackageSystemInfo {
    /// Collect package system statistics
    pub fn collect(package_manager: &PackageManager) -> Result<Self, PackageError> {
        let total_installed = package_manager.installed_packages.len();
        let mut total_available = 0;
        
        for packages in package_manager.repository_cache.values() {
            total_available += packages.len();
        }
        
        Ok(Self {
            total_installed_packages: total_installed,
            total_available_packages: total_available,
            cache_usage_mb: 0, // Would calculate actual cache usage
            last_update_check: 0, // Would get from configuration
            repositories_count: package_manager.config.default_repositories.len(),
            security_updates_available: 0, // Would calculate from update check
            auto_update_enabled: package_manager.config.auto_update,
        })
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_package_system_info_collection() {
        let config = PackageConfig {
            default_repositories: vec!["test-repo".to_string()],
            cache_dir: "/tmp/test-cache".to_string(),
            install_dir: "/tmp/test-install".to_string(),
            temp_dir: "/tmp/test-temp".to_string(),
            verify_signatures: false,
            auto_update: false,
            max_cache_size: 1024 * 1024,
            timeout_seconds: 60,
        };
        
        let manager = PackageManager::new(config);
        let info = PackageSystemInfo::collect(&manager);
        
        assert!(info.is_ok());
        let info = info.unwrap();
        assert_eq!(info.total_installed_packages, 0);
        assert_eq!(info.auto_update_enabled, false);
    }
}

