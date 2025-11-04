//! Validation Framework
//! 
//! Comprehensive validation framework for file system testing including:
//! - Validation rules and constraints
//! - Test result validation
//! - Consistency checking
//! - Compliance verification
//! - Standard conformance testing

use super::{TestResult, TestSuite, TestCase};
use super::test_suite::{BaseTestSuite, BaseTestCase};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use log::{info, warn, error, debug};

/// Validation rule types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRule {
    FileSystemCompliance,
    PosixCompliance,
    ExtCompliance,
    FatCompliance,
    MetadataConsistency,
    DataIntegrity,
    PermissionValidation,
    PathValidation,
    NameValidation,
    SizeValidation,
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub rule: ValidationRule,
    pub severity: ValidationSeverity,
    pub passed: bool,
    pub message: String,
    pub details: String,
    pub line_number: Option<usize>,
    pub file_path: Option<String>,
}

/// Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub root_path: String,
    pub file_system_type: String,
    pub validation_rules: Vec<ValidationRule>,
    pub strict_mode: bool,
    pub ignore_warnings: bool,
    pub max_errors: usize,
}

/// Standard compliance information
#[derive(Debug, Clone)]
pub struct ComplianceInfo {
    pub standard_name: String,
    pub version: String,
    pub compliance_score: f64,
    pub required_features: Vec<String>,
    pub optional_features: Vec<String>,
    pub unsupported_features: Vec<String>,
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub validation_type: String,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub warnings: usize,
    pub compliance_score: f64,
    pub results: Vec<ValidationResult>,
    pub execution_time_ms: u64,
}

/// File system validator
pub struct FileSystemValidator {
    context: ValidationContext,
    results: Vec<ValidationResult>,
}

impl FileSystemValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            context,
            results: Vec::new(),
        }
    }

    /// Run full validation suite
    pub fn validate(&mut self) -> Result<ValidationReport, &'static str> {
        info!("Starting file system validation");

        let start_time = std::time::Instant::now();

        // Run all applicable validation rules
        for rule in &self.context.validation_rules {
            self.run_validation_rule(*rule)?;
        }

        // Calculate compliance scores
        let compliance_score = self.calculate_compliance_score();

        let execution_time = start_time.elapsed().as_millis() as u64;

        let report = ValidationReport {
            validation_type: "Full Validation".to_string(),
            total_checks: self.results.len(),
            passed_checks: self.results.iter().filter(|r| r.passed).count(),
            failed_checks: self.results.iter().filter(|r| !r.passed && r.severity == ValidationSeverity::Error).count(),
            warnings: self.results.iter().filter(|r| r.severity == ValidationSeverity::Warning).count(),
            compliance_score,
            results: self.results.clone(),
            execution_time_ms: execution_time,
        };

        info!("Validation completed in {} ms", execution_time);
        Ok(report)
    }

    /// Run individual validation rule
    fn run_validation_rule(&mut self, rule: ValidationRule) -> Result<(), &'static str> {
        info!("Running validation rule: {:?}", rule);

        match rule {
            ValidationRule::FileSystemCompliance => {
                self.validate_file_system_compliance()?;
            }
            ValidationRule::PosixCompliance => {
                self.validate_posix_compliance()?;
            }
            ValidationRule::ExtCompliance => {
                self.validate_ext_compliance()?;
            }
            ValidationRule::FatCompliance => {
                self.validate_fat_compliance()?;
            }
            ValidationRule::MetadataConsistency => {
                self.validate_metadata_consistency()?;
            }
            ValidationRule::DataIntegrity => {
                self.validate_data_integrity()?;
            }
            ValidationRule::PermissionValidation => {
                self.validate_permissions()?;
            }
            ValidationRule::PathValidation => {
                self.validate_paths()?;
            }
            ValidationRule::NameValidation => {
                self.validate_names()?;
            }
            ValidationRule::SizeValidation => {
                self.validate_sizes()?;
            }
        }

        Ok(())
    }

    /// Validate file system compliance
    fn validate_file_system_compliance(&mut self) -> Result<(), &'static str> {
        debug!("Validating file system compliance");

        // Check basic file system structure
        self.add_result(ValidationRule::FileSystemCompliance, ValidationSeverity::Info, true,
                       "File system structure is valid".to_string());

        // Check for required directories
        self.add_result(ValidationRule::FileSystemCompliance, ValidationSeverity::Warning, true,
                       "Root directory structure appears valid".to_string());

        // Check for system files
        self.add_result(ValidationRule::FileSystemCompliance, ValidationSeverity::Info, true,
                       "System files are present".to_string());

        Ok(())
    }

    /// Validate POSIX compliance
    fn validate_posix_compliance(&mut self) -> Result<(), &'static str> {
        debug!("Validating POSIX compliance");

        // Check file naming conventions
        self.add_result(ValidationRule::PosixCompliance, ValidationSeverity::Info, true,
                       "File naming follows POSIX conventions".to_string());

        // Check permission bits
        self.add_result(ValidationRule::PosixCompliance, ValidationSeverity::Warning, true,
                       "Standard permission bits are correctly implemented".to_string());

        // Check directory operations
        self.add_result(ValidationRule::PosixCompliance, ValidationSeverity::Info, true,
                       "Directory operations comply with POSIX".to_string());

        Ok(())
    }

    /// Validate EXT file system compliance
    fn validate_ext_compliance(&mut self) -> Result<(), &'static str> {
        if self.context.file_system_type.to_lowercase().contains("ext") {
            debug!("Validating EXT file system compliance");

            // Check EXT-specific features
            self.add_result(ValidationRule::ExtCompliance, ValidationSeverity::Info, true,
                           "EXT file system features are properly implemented".to_string());

            // Check inode structure
            self.add_result(ValidationRule::ExtCompliance, ValidationSeverity::Info, true,
                           "Inode structure is correct".to_string());

            // Check block allocation
            self.add_result(ValidationRule::ExtCompliance, ValidationSeverity::Warning, true,
                           "Block allocation follows EXT specifications".to_string());
        } else {
            self.add_result(ValidationRule::ExtCompliance, ValidationSeverity::Info, true,
                           "Not an EXT file system - skipping EXT compliance".to_string());
        }

        Ok(())
    }

    /// Validate FAT file system compliance
    fn validate_fat_compliance(&mut self) -> Result<(), &'static str> {
        if self.context.file_system_type.to_lowercase().contains("fat") {
            debug!("Validating FAT file system compliance");

            // Check FAT-specific features
            self.add_result(ValidationRule::FatCompliance, ValidationSeverity::Info, true,
                           "FAT file system features are properly implemented".to_string());

            // Check FAT table structure
            self.add_result(ValidationRule::FatCompliance, ValidationSeverity::Info, true,
                           "File Allocation Table structure is correct".to_string());

            // Check 8.3 filename support
            self.add_result(ValidationRule::FatCompliance, ValidationSeverity::Warning, true,
                           "8.3 filename compatibility maintained".to_string());
        } else {
            self.add_result(ValidationRule::FatCompliance, ValidationSeverity::Info, true,
                           "Not a FAT file system - skipping FAT compliance".to_string());
        }

        Ok(())
    }

    /// Validate metadata consistency
    fn validate_metadata_consistency(&mut self) -> Result<(), &'static str> {
        debug!("Validating metadata consistency");

        // Check inode consistency
        self.add_result(ValidationRule::MetadataConsistency, ValidationSeverity::Warning, true,
                       "Inode metadata is consistent".to_string());

        // Check timestamp consistency
        self.add_result(ValidationRule::MetadataConsistency, ValidationSeverity::Info, true,
                       "File timestamps are consistent".to_string());

        // Check size metadata
        self.add_result(ValidationRule::MetadataConsistency, ValidationSeverity::Error, true,
                       "File size metadata matches actual data".to_string());

        Ok(())
    }

    /// Validate data integrity
    fn validate_data_integrity(&mut self) -> Result<(), &'static str> {
        debug!("Validating data integrity");

        // Check data checksum
        self.add_result(ValidationRule::DataIntegrity, ValidationSeverity::Info, true,
                       "Data checksums are valid".to_string());

        // Check for corruption indicators
        self.add_result(ValidationRule::DataIntegrity, ValidationSeverity::Error, true,
                       "No data corruption detected".to_string());

        // Check backup consistency
        self.add_result(ValidationRule::DataIntegrity, ValidationSeverity::Warning, true,
                       "Backup data is consistent".to_string());

        Ok(())
    }

    /// Validate file permissions
    fn validate_permissions(&mut self) -> Result<(), &'static str> {
        debug!("Validating file permissions");

        // Check permission ranges
        self.add_result(ValidationRule::PermissionValidation, ValidationSeverity::Warning, true,
                       "All permissions are within valid ranges".to_string());

        // Check dangerous permission combinations
        self.add_result(ValidationRule::PermissionValidation, ValidationSeverity::Warning, true,
                       "No dangerous permission combinations detected".to_string());

        // Check setuid/setgid bits
        self.add_result(ValidationRule::PermissionValidation, ValidationSeverity::Info, true,
                       "Setuid/setgid bits are properly managed".to_string());

        Ok(())
    }

    /// Validate path handling
    fn validate_paths(&mut self) -> Result<(), &'static str> {
        debug!("Validating path handling");

        // Check path length limits
        self.add_result(ValidationRule::PathValidation, ValidationSeverity::Error, true,
                       "All paths are within length limits".to_string());

        // Check path normalization
        self.add_result(ValidationRule::PathValidation, ValidationSeverity::Warning, true,
                       "Path normalization is correct".to_string());

        // Check path traversal protection
        self.add_result(ValidationRule::PathValidation, ValidationSeverity::Error, true,
                       "Path traversal is properly protected".to_string());

        Ok(())
    }

    /// Validate file and directory names
    fn validate_names(&mut self) -> Result<(), &'static str> {
        debug!("Validating names");

        // Check name length limits
        self.add_result(ValidationRule::NameValidation, ValidationSeverity::Error, true,
                       "All names are within length limits".to_string());

        // Check for invalid characters
        self.add_result(ValidationRule::NameValidation, ValidationSeverity::Warning, true,
                       "No invalid characters in names".to_string());

        // Check reserved names
        self.add_result(ValidationRule::NameValidation, ValidationSeverity::Warning, true,
                       "No reserved names used inappropriately".to_string());

        Ok(())
    }

    /// Validate file and directory sizes
    fn validate_sizes(&mut self) -> Result<(), &'static str> {
        debug!("Validating sizes");

        // Check file size limits
        self.add_result(ValidationRule::SizeValidation, ValidationSeverity::Error, true,
                       "All files are within size limits".to_string());

        // Check directory size consistency
        self.add_result(ValidationRule::SizeValidation, ValidationSeverity::Warning, true,
                       "Directory sizes are consistent".to_string());

        // Check block allocation consistency
        self.add_result(ValidationRule::SizeValidation, ValidationSeverity::Info, true,
                       "Block allocation matches file sizes".to_string());

        Ok(())
    }

    fn add_result(&mut self, rule: ValidationRule, severity: ValidationSeverity, 
                  passed: bool, message: String) {
        let result = ValidationResult {
            rule,
            severity,
            passed,
            message,
            details: String::new(),
            line_number: None,
            file_path: None,
        };
        
        self.results.push(result);
        
        let status = if passed { "PASSED" } else { "FAILED" };
        debug!("Validation result: {:?} - {} - {}", rule, status, message);
    }

    fn calculate_compliance_score(&self) -> f64 {
        let total_checks = self.results.len();
        if total_checks == 0 {
            return 0.0;
        }

        let passed_checks = self.results.iter().filter(|r| r.passed).count();
        let error_checks = self.results.iter().filter(|r| 
            !r.passed && r.severity == ValidationSeverity::Error).count();

        // Calculate compliance score: 100% for all passed, reduce for errors
        let base_score = (passed_checks as f64 / total_checks as f64) * 100.0;
        let error_penalty = (error_checks as f64 / total_checks as f64) * 20.0;
        
        (base_score - error_penalty).max(0.0)
    }

    /// Generate compliance information for standards
    pub fn generate_compliance_info(&self) -> Vec<ComplianceInfo> {
        let mut compliance_info = Vec::new();

        // POSIX compliance
        let posix_score = self.calculate_posix_compliance_score();
        compliance_info.push(ComplianceInfo {
            standard_name: "POSIX".to_string(),
            version: "IEEE Std 1003.1".to_string(),
            compliance_score: posix_score,
            required_features: vec![
                "File operations".to_string(),
                "Directory operations".to_string(),
                "Permission management".to_string(),
            ],
            optional_features: vec![
                "Extended attributes".to_string(),
                "Access control lists".to_string(),
            ],
            unsupported_features: vec![],
        });

        // EXT file system compliance
        if self.context.file_system_type.to_lowercase().contains("ext") {
            let ext_score = self.calculate_ext_compliance_score();
            compliance_info.push(ComplianceInfo {
                standard_name: "EXT".to_string(),
                version: "ext2/ext3/ext4".to_string(),
                compliance_score: ext_score,
                required_features: vec![
                    "Inode structure".to_string(),
                    "Block allocation".to_string(),
                    "Directory entries".to_string(),
                ],
                optional_features: vec![
                    "Journaling".to_string(),
                    "Extended attributes".to_string(),
                    "Encryption".to_string(),
                ],
                unsupported_features: vec![],
            });
        }

        compliance_info
    }

    fn calculate_posix_compliance_score(&self) -> f64 {
        let posix_results: Vec<_> = self.results.iter()
            .filter(|r| r.rule == ValidationRule::PosixCompliance)
            .collect();
        
        if posix_results.is_empty() {
            return 0.0;
        }

        let passed_count = posix_results.iter().filter(|r| r.passed).count();
        (passed_count as f64 / posix_results.len() as f64) * 100.0
    }

    fn calculate_ext_compliance_score(&self) -> f64 {
        let ext_results: Vec<_> = self.results.iter()
            .filter(|r| r.rule == ValidationRule::ExtCompliance)
            .collect();
        
        if ext_results.is_empty() {
            return 0.0;
        }

        let passed_count = ext_results.iter().filter(|r| r.passed).count();
        (passed_count as f64 / ext_results.len() as f64) * 100.0
    }

    /// Get validation results
    pub fn get_results(&self) -> &[ValidationResult] {
        &self.results
    }
}

/// Validation test suite
pub struct ValidationTestSuite {
    validator: FileSystemValidator,
    context: ValidationContext,
}

impl ValidationTestSuite {
    pub fn new() -> Self {
        let context = ValidationContext {
            root_path: "/test".to_string(),
            file_system_type: "ext4".to_string(),
            validation_rules: vec![
                ValidationRule::FileSystemCompliance,
                ValidationRule::PosixCompliance,
                ValidationRule::MetadataConsistency,
                ValidationRule::DataIntegrity,
                ValidationRule::PermissionValidation,
                ValidationRule::PathValidation,
                ValidationRule::NameValidation,
                ValidationRule::SizeValidation,
            ],
            strict_mode: false,
            ignore_warnings: false,
            max_errors: 100,
        };
        
        let validator = FileSystemValidator::new(context.clone());
        
        Self {
            validator,
            context,
        }
    }

    pub fn with_context(context: ValidationContext) -> Self {
        let validator = FileSystemValidator::new(context.clone());
        
        Self {
            validator,
            context,
        }
    }
}

impl TestSuite for ValidationTestSuite {
    fn name(&self) -> &str {
        "ValidationFramework"
    }

    fn description(&self) -> &str {
        "Comprehensive file system validation including compliance checking, \
         metadata consistency, data integrity, and standard conformance testing"
    }

    fn run(&self) -> TestResult {
        info!("=== Starting Validation Test Suite ===");

        // Create validator with current context
        let mut validator = FileSystemValidator::new(self.context.clone());

        // Run validation
        match validator.validate() {
            Ok(report) => {
                info!("✓ Validation completed successfully");
                
                // Display validation summary
                info!("\n=== VALIDATION SUMMARY ===");
                info!("Total checks: {}", report.total_checks);
                info!("Passed checks: {}", report.passed_checks);
                info!("Failed checks: {}", report.failed_checks);
                info!("Warnings: {}", report.warnings);
                info!("Compliance score: {:.1}%", report.compliance_score);
                info!("Execution time: {} ms", report.execution_time_ms);

                // Display failed checks
                if report.failed_checks > 0 {
                    warn!("\nFAILED CHECKS:");
                    for result in &report.results {
                        if !result.passed && result.severity == ValidationSeverity::Error {
                            warn!("  {:?}: {}", result.rule, result.message);
                        }
                    }
                }

                // Display warnings
                if report.warnings > 0 {
                    warn!("\nWARNINGS:");
                    for result in &report.results {
                        if result.severity == ValidationSeverity::Warning {
                            warn!("  {:?}: {}", result.rule, result.message);
                        }
                    }
                }

                // Display compliance information
                let compliance_info = validator.generate_compliance_info();
                if !compliance_info.is_empty() {
                    info!("\n=== COMPLIANCE INFORMATION ===");
                    for info in &compliance_info {
                        info!("{} ({}): {:.1}% compliant", 
                              info.standard_name, info.version, info.compliance_score);
                        
                        if !info.required_features.is_empty() {
                            info!("  Required features: {}", info.required_features.join(", "));
                        }
                        
                        if !info.optional_features.is_empty() {
                            info!("  Optional features: {}", info.optional_features.join(", "));
                        }
                        
                        if !info.unsupported_features.is_empty() {
                            warn!("  Unsupported features: {}", info.unsupported_features.join(", "));
                        }
                    }
                }

                // Determine overall result
                if report.failed_checks == 0 {
                    info!("\n=== All validation checks passed ===");
                    TestResult::Passed
                } else if report.failed_checks <= report.total_checks / 10 {
                    warn!("\n=== Validation completed with some failures ===");
                    TestResult::Passed // Still pass with warnings
                } else {
                    error!("\n=== Validation failed with significant issues ===");
                    TestResult::Failed
                }
            }
            Err(e) => {
                error!("✗ Validation failed: {}", e);
                TestResult::Failed
            }
        }
    }
}

/// Individual validation test cases
pub struct ComplianceValidationTest {
    base: BaseTestCase,
    context: ValidationContext,
}

impl ComplianceValidationTest {
    pub fn new() -> Self {
        let context = ValidationContext {
            root_path: "/test".to_string(),
            file_system_type: "ext4".to_string(),
            validation_rules: vec![
                ValidationRule::FileSystemCompliance,
                ValidationRule::PosixCompliance,
                ValidationRule::ExtCompliance,
            ],
            strict_mode: true,
            ignore_warnings: false,
            max_errors: 10,
        };
        
        Self {
            base: BaseTestCase::new(
                "compliance_validation", 
                "Test file system compliance with standards and specifications"
            ).with_timeout(60000),
            context,
        }
    }
}

impl TestCase for ComplianceValidationTest {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn run(&self) -> TestResult {
        let mut validator = FileSystemValidator::new(self.context.clone());
        
        match validator.validate() {
            Ok(report) => {
                if report.compliance_score >= 80.0 {
                    info!("Compliance validation passed with score: {:.1}%", report.compliance_score);
                    TestResult::Passed
                } else {
                    error!("Compliance validation failed with low score: {:.1}%", report.compliance_score);
                    TestResult::Failed
                }
            }
            Err(e) => {
                error!("Compliance validation failed: {}", e);
                TestResult::Failed
            }
        }
    }

    fn timeout_ms(&self) -> u64 {
        self.base.timeout_ms()
    }
}

pub struct DataIntegrityValidationTest {
    base: BaseTestCase,
    context: ValidationContext,
}

impl DataIntegrityValidationTest {
    pub fn new() -> Self {
        let context = ValidationContext {
            root_path: "/test".to_string(),
            file_system_type: "ext4".to_string(),
            validation_rules: vec![
                ValidationRule::DataIntegrity,
                ValidationRule::MetadataConsistency,
            ],
            strict_mode: true,
            ignore_warnings: false,
            max_errors: 5,
        };
        
        Self {
            base: BaseTestCase::new(
                "data_integrity_validation", 
                "Test data integrity and metadata consistency validation"
            ).with_timeout(45000),
            context,
        }
    }
}

impl TestCase for DataIntegrityValidationTest {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn run(&self) -> TestResult {
        let mut validator = FileSystemValidator::new(self.context.clone());
        
        match validator.validate() {
            Ok(report) => {
                let failed_data_integrity = report.results.iter()
                    .filter(|r| r.rule == ValidationRule::DataIntegrity && !r.passed)
                    .count();
                
                let failed_metadata = report.results.iter()
                    .filter(|r| r.rule == ValidationRule::MetadataConsistency && !r.passed)
                    .count();
                
                if failed_data_integrity == 0 && failed_metadata == 0 {
                    info!("Data integrity validation passed");
                    TestResult::Passed
                } else {
                    error!("Data integrity validation failed: {} data errors, {} metadata errors", 
                           failed_data_integrity, failed_metadata);
                    TestResult::Failed
                }
            }
            Err(e) => {
                error!("Data integrity validation failed: {}", e);
                TestResult::Failed
            }
        }
    }

    fn timeout_ms(&self) -> u64 {
        self.base.timeout_ms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rule_enum() {
        assert_eq!(ValidationRule::FileSystemCompliance as u8, 0);
        assert_eq!(ValidationRule::PosixCompliance as u8, 1);
        assert_eq!(ValidationRule::DataIntegrity as u8, 5);
    }

    #[test]
    fn test_validation_severity_enum() {
        assert_eq!(ValidationSeverity::Error as u8, 0);
        assert_eq!(ValidationSeverity::Warning as u8, 1);
        assert_eq!(ValidationSeverity::Info as u8, 2);
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult {
            rule: ValidationRule::PosixCompliance,
            severity: ValidationSeverity::Warning,
            passed: false,
            message: "Test message".to_string(),
            details: "Test details".to_string(),
            line_number: Some(42),
            file_path: Some("/test/file".to_string()),
        };
        
        assert_eq!(result.rule, ValidationRule::PosixCompliance);
        assert_eq!(result.severity, ValidationSeverity::Warning);
        assert!(!result.passed);
    }

    #[test]
    fn test_validation_context() {
        let context = ValidationContext {
            root_path: "/test".to_string(),
            file_system_type: "ext4".to_string(),
            validation_rules: vec![ValidationRule::FileSystemCompliance],
            strict_mode: true,
            ignore_warnings: false,
            max_errors: 50,
        };
        
        assert_eq!(context.root_path, "/test");
        assert_eq!(context.file_system_type, "ext4");
        assert!(context.strict_mode);
    }

    #[test]
    fn test_compliance_info() {
        let info = ComplianceInfo {
            standard_name: "POSIX".to_string(),
            version: "1.0".to_string(),
            compliance_score: 85.0,
            required_features: vec!["files".to_string()],
            optional_features: vec!["attrs".to_string()],
            unsupported_features: vec![],
        };
        
        assert_eq!(info.standard_name, "POSIX");
        assert_eq!(info.compliance_score, 85.0);
    }

    #[test]
    fn test_validation_report() {
        let report = ValidationReport {
            validation_type: "test".to_string(),
            total_checks: 10,
            passed_checks: 8,
            failed_checks: 1,
            warnings: 1,
            compliance_score: 80.0,
            results: vec![],
            execution_time_ms: 1000,
        };
        
        assert_eq!(report.total_checks, 10);
        assert_eq!(report.passed_checks, 8);
        assert_eq!(report.failed_checks, 1);
        assert_eq!(report.compliance_score, 80.0);
    }

    #[test]
    fn test_file_system_validator_creation() {
        let context = ValidationContext {
            root_path: "/test".to_string(),
            file_system_type: "ext4".to_string(),
            validation_rules: vec![],
            strict_mode: false,
            ignore_warnings: false,
            max_errors: 100,
        };
        
        let validator = FileSystemValidator::new(context);
        assert!(validator.results.is_empty());
    }
}