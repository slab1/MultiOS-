//! Test Data Management and Cleanup Procedures
//! 
//! This module provides comprehensive test data management:
//! - Test data generation and seeding
//! - Data isolation and cleanup
//! - Historical test data retention
//! - Test data validation and integrity checking

use super::*;
use crate::*;
use crate::Result;
use log::{info, warn, error};

/// Test data manager for comprehensive test data lifecycle management
pub struct TestDataManager {
    pub data_directory: String,
    pub retention_policy: DataRetentionPolicy,
    pub isolation_level: DataIsolationLevel,
    pub validation_enabled: bool,
    pub cleanup_on_startup: bool,
}

/// Data retention policy configuration
#[derive(Debug, Clone)]
pub struct DataRetentionPolicy {
    pub max_age_days: usize,
    pub max_size_mb: usize,
    pub auto_cleanup_enabled: bool,
    pub compression_enabled: bool,
    pub archive_old_data: bool,
}

/// Data isolation levels for tests
#[derive(Debug, Clone)]
pub enum DataIsolationLevel {
    None,
    PerTest,
    PerCategory,
    PerSession,
    Full,
}

/// Test data categories
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestDataCategory {
    Admin,
    Security,
    Update,
    System,
    Performance,
    Integration,
}

/// Test data types
#[derive(Debug, Clone)]
pub enum TestDataType {
    UserAccounts,
    ConfigurationFiles,
    SecurityPolicies,
    SystemState,
    PerformanceBaselines,
    TestResults,
    LogFiles,
}

/// Test data record for tracking and management
#[derive(Debug, Clone)]
pub struct TestDataRecord {
    pub id: String,
    pub category: TestDataCategory,
    pub data_type: TestDataType,
    pub created_timestamp: u64,
    pub last_accessed: u64,
    pub size_bytes: usize,
    pub checksum: String,
    pub metadata: alloc::vec::Vec<(String, String)>,
}

/// Test data validation result
#[derive(Debug, Clone)]
pub struct DataValidationResult {
    pub is_valid: bool,
    pub integrity_score: f64,
    pub issues_found: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Test data statistics
#[derive(Debug, Clone)]
pub struct TestDataStatistics {
    pub total_records: usize,
    pub total_size_bytes: usize,
    pub category_breakdown: alloc::collections::BTreeMap<TestDataCategory, CategoryStats>,
    pub age_distribution: AgeDistribution,
}

/// Category-specific statistics
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub record_count: usize,
    pub total_size_bytes: usize,
    pub avg_record_size: usize,
    pub oldest_record_age_days: usize,
    pub newest_record_age_days: usize,
}

/// Age distribution of test data
#[derive(Debug, Clone)]
pub struct AgeDistribution {
    pub less_than_1_day: usize,
    pub less_than_1_week: usize,
    pub less_than_1_month: usize,
    pub older_than_1_month: usize,
}

/// Test data seeding configuration
#[derive(Debug, Clone)]
pub struct TestDataSeedingConfig {
    pub seed_admin_data: bool,
    pub seed_security_data: bool,
    pub seed_system_data: bool,
    pub seed_performance_baselines: bool,
    pub num_test_users: usize,
    pub num_test_services: usize,
    pub num_test_policies: usize,
}

/// Test environment cleanup configuration
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub aggressive_cleanup: bool,
    pub preserve_critical_data: bool,
    pub remove_test_logs: bool,
    pub cleanup_temp_files: bool,
    pub reset_permissions: bool,
    pub validate_after_cleanup: bool,
}

impl TestDataManager {
    /// Create a new test data manager
    pub fn new(data_directory: String, retention_policy: DataRetentionPolicy) -> Self {
        Self {
            data_directory,
            retention_policy,
            isolation_level: DataIsolationLevel::PerTest,
            validation_enabled: true,
            cleanup_on_startup: true,
        }
    }

    /// Initialize test data manager
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing test data manager...");
        
        // Create data directory structure
        self.create_directory_structure()?;
        
        // Perform startup cleanup if enabled
        if self.cleanup_on_startup {
            self.perform_startup_cleanup()?;
        }
        
        // Initialize data tracking
        self.initialize_data_tracking()?;
        
        info!("Test data manager initialized successfully");
        Ok(())
    }

    /// Create directory structure for test data
    fn create_directory_structure(&self) -> Result<()> {
        let directories = [
            "admin",
            "security", 
            "update",
            "system",
            "performance",
            "integration",
            "results",
            "logs",
            "temp",
        ];
        
        for dir in &directories {
            let full_path = format!("{}/{}", self.data_directory, dir);
            let _ = crate::filesystem::create_directory(&full_path);
        }
        
        info!("Test data directory structure created");
        Ok(())
    }

    /// Perform startup cleanup
    fn perform_startup_cleanup(&mut self) -> Result<()> {
        info!("Performing startup cleanup...");
        
        let cleanup_config = CleanupConfig {
            aggressive_cleanup: false,
            preserve_critical_data: true,
            remove_test_logs: true,
            cleanup_temp_files: true,
            reset_permissions: false,
            validate_after_cleanup: true,
        };
        
        self.cleanup_old_data(&cleanup_config)?;
        
        info!("Startup cleanup completed");
        Ok(())
    }

    /// Initialize data tracking
    fn initialize_data_tracking(&self) -> Result<()> {
        // Create data tracking database
        let tracking_db_path = format!("{}/tracking.db", self.data_directory);
        let _ = crate::filesystem::write_file(
            &tracking_db_path, 
            b"Test data tracking database initialized"
        );
        
        info!("Data tracking initialized");
        Ok(())
    }

    /// Seed test data based on configuration
    pub fn seed_test_data(&mut self, config: TestDataSeedingConfig) -> Result<Vec<TestDataRecord>> {
        info!("Seeding test data...");
        let mut created_records = Vec::new();
        let start_time = crate::hal::get_current_time_ms();
        
        // Seed admin data
        if config.seed_admin_data {
            let admin_records = self.seed_admin_test_data(&config)?;
            created_records.extend(admin_records);
        }
        
        // Seed security data
        if config.seed_security_data {
            let security_records = self.seed_security_test_data(&config)?;
            created_records.extend(security_records);
        }
        
        // Seed system data
        if config.seed_system_data {
            let system_records = self.seed_system_test_data(&config)?;
            created_records.extend(system_records);
        }
        
        // Seed performance baselines
        if config.seed_performance_baselines {
            let performance_records = self.seed_performance_baselines()?;
            created_records.extend(performance_records);
        }
        
        let seeding_time = crate::hal::get_current_time_ms() - start_time;
        info!("Test data seeding completed: {} records created in {}ms", 
              created_records.len(), seeding_time);
        
        Ok(created_records)
    }

    /// Seed admin test data
    fn seed_admin_test_data(&self, config: &TestDataSeedingConfig) -> Result<Vec<TestDataRecord>> {
        let mut records = Vec::new();
        
        // Create test users
        for i in 0..config.num_test_users {
            let username = format!("test_user_{}", i);
            let user_data = format!(r#"{{
                "username": "{}",
                "password": "TestPass123!",
                "role": "user",
                "created_at": {},
                "test_data": true
            }}"#, username, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/admin/users/{}.json", self.data_directory, username);
            let _ = crate::filesystem::write_file(&file_path, user_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("user_{}", i),
                category: TestDataCategory::Admin,
                data_type: TestDataType::UserAccounts,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: user_data.len(),
                checksum: self.calculate_checksum(&user_data),
                metadata: vec![
                    ("filename".to_string(), format!("{}.json", username)),
                    ("type".to_string(), "user_account".to_string()),
                ],
            };
            records.push(record);
        }
        
        // Create test configuration files
        for i in 0..10 {
            let config_data = format!(r#"{{
                "config_id": "test_config_{}",
                "settings": {{
                    "theme": "dark",
                    "language": "en",
                    "notifications": true
                }},
                "created_at": {},
                "test_data": true
            }}"#, i, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/admin/config/test_config_{}.json", self.data_directory, i);
            let _ = crate::filesystem::write_file(&file_path, config_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("config_{}", i),
                category: TestDataCategory::Admin,
                data_type: TestDataType::ConfigurationFiles,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: config_data.len(),
                checksum: self.calculate_checksum(&config_data),
                metadata: vec![
                    ("config_type".to_string(), "user_settings".to_string()),
                    ("version".to_string(), "1.0".to_string()),
                ],
            };
            records.push(record);
        }
        
        info!("Seeded {} admin test data records", records.len());
        Ok(records)
    }

    /// Seed security test data
    fn seed_security_test_data(&self, config: &TestDataSeedingConfig) -> Result<Vec<TestDataRecord>> {
        let mut records = Vec::new();
        
        // Create test security policies
        for i in 0..config.num_test_policies {
            let policy_data = format!(r#"{{
                "policy_id": "test_policy_{}",
                "name": "Test Security Policy {}",
                "rules": [
                    {{
                        "action": "allow",
                        "resource": "user_data",
                        "operations": ["read", "write"]
                    }}
                ],
                "created_at": {},
                "test_data": true
            }}"#, i, i, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/security/policies/test_policy_{}.json", self.data_directory, i);
            let _ = crate::filesystem::write_file(&file_path, policy_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("policy_{}", i),
                category: TestDataCategory::Security,
                data_type: TestDataType::SecurityPolicies,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: policy_data.len(),
                checksum: self.calculate_checksum(&policy_data),
                metadata: vec![
                    ("policy_type".to_string(), "access_control".to_string()),
                    ("version".to_string(), "1.0".to_string()),
                ],
            };
            records.push(record);
        }
        
        // Create test authentication tokens
        for i in 0..config.num_test_users {
            let token_data = format!(r#"{{
                "token_id": "test_token_{}",
                "user_id": "test_user_{}",
                "permissions": ["read", "write"],
                "expires_at": {},
                "created_at": {},
                "test_data": true
            }}"#, i, i, crate::hal::get_current_time_ms() + 3600000, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/security/tokens/test_token_{}.json", self.data_directory, i);
            let _ = crate::filesystem::write_file(&file_path, token_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("token_{}", i),
                category: TestDataCategory::Security,
                data_type: TestDataType::UserAccounts,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: token_data.len(),
                checksum: self.calculate_checksum(&token_data),
                metadata: vec![
                    ("token_type".to_string(), "session".to_string()),
                    ("status".to_string(), "active".to_string()),
                ],
            };
            records.push(record);
        }
        
        info!("Seeded {} security test data records", records.len());
        Ok(records)
    }

    /// Seed system test data
    fn seed_system_test_data(&self, config: &TestDataSeedingConfig) -> Result<Vec<TestDataRecord>> {
        let mut records = Vec::new();
        
        // Create test service configurations
        for i in 0..config.num_test_services {
            let service_data = format!(r#"{{
                "service_id": "test_service_{}",
                "name": "Test Service {}",
                "type": "system",
                "dependencies": ["filesystem", "network"],
                "startup_timeout": 5000,
                "created_at": {},
                "test_data": true
            }}"#, i, i, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/system/services/test_service_{}.json", self.data_directory, i);
            let _ = crate::filesystem::write_file(&file_path, service_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("service_{}", i),
                category: TestDataCategory::System,
                data_type: TestDataType::ConfigurationFiles,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: service_data.len(),
                checksum: self.calculate_checksum(&service_data),
                metadata: vec![
                    ("service_type".to_string(), "system".to_string()),
                    ("status".to_string(), "stopped".to_string()),
                ],
            };
            records.push(record);
        }
        
        // Create system state snapshots
        for i in 0..5 {
            let snapshot_data = format!(r#"{{
                "snapshot_id": "test_snapshot_{}",
                "kernel_version": "1.0.0",
                "memory_state": {{
                    "total_pages": 1024,
                    "used_pages": 512,
                    "available_pages": 512
                }},
                "services_state": {{
                    "running": ["system", "security"],
                    "stopped": ["update"]
                }},
                "created_at": {},
                "test_data": true
            }}"#, i, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/system/snapshots/test_snapshot_{}.json", self.data_directory, i);
            let _ = crate::filesystem::write_file(&file_path, snapshot_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("snapshot_{}", i),
                category: TestDataCategory::System,
                data_type: TestDataType::SystemState,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: snapshot_data.len(),
                checksum: self.calculate_checksum(&snapshot_data),
                metadata: vec![
                    ("snapshot_type".to_string(), "system_state".to_string()),
                    ("version".to_string(), "1.0".to_string()),
                ],
            };
            records.push(record);
        }
        
        info!("Seeded {} system test data records", records.len());
        Ok(records)
    }

    /// Seed performance baseline data
    fn seed_performance_baselines(&self) -> Result<Vec<TestDataRecord>> {
        let mut records = Vec::new();
        
        // Create performance baseline measurements
        for i in 0..10 {
            let baseline_data = format!(r#"{{
                "baseline_id": "test_baseline_{}",
                "test_name": "integration_test_{}",
                "metrics": {{
                    "memory_usage_kb": 2048,
                    "cpu_time_ms": 150,
                    "throughput_ops_per_sec": 50.0,
                    "latency_p95_ms": 50.0,
                    "latency_p99_ms": 100.0
                }},
                "environment": {{
                    "architecture": "x86_64",
                    "memory_total_mb": 8192,
                    "cpu_cores": 8
                }},
                "created_at": {},
                "test_data": true
            }}"#, i, i, crate::hal::get_current_time_ms());
            
            let file_path = format!("{}/performance/baselines/test_baseline_{}.json", self.data_directory, i);
            let _ = crate::filesystem::write_file(&file_path, baseline_data.as_bytes());
            
            let record = TestDataRecord {
                id: format!("baseline_{}", i),
                category: TestDataCategory::Performance,
                data_type: TestDataType::PerformanceBaselines,
                created_timestamp: crate::hal::get_current_time_ms(),
                last_accessed: crate::hal::get_current_time_ms(),
                size_bytes: baseline_data.len(),
                checksum: self.calculate_checksum(&baseline_data),
                metadata: vec![
                    ("baseline_type".to_string(), "integration_performance".to_string()),
                    ("version".to_string(), "1.0".to_string()),
                ],
            };
            records.push(record);
        }
        
        info!("Seeded {} performance baseline records", records.len());
        Ok(records)
    }

    /// Clean up test data based on retention policy
    pub fn cleanup_old_data(&self, config: &CleanupConfig) -> Result<CleanupStats> {
        info!("Cleaning up old test data...");
        let start_time = crate::hal::get_current_time_ms();
        
        let mut cleanup_stats = CleanupStats::default();
        let max_age_ms = (self.retention_policy.max_age_days * 24 * 60 * 60 * 1000) as u64;
        let cutoff_time = crate::hal::get_current_time_ms() - max_age_ms;
        
        // Clean up by category
        for category in &[TestDataCategory::Admin, TestDataCategory::Security, 
                         TestDataCategory::System, TestDataCategory::Performance] {
            let category_stats = self.cleanup_category_data(category, cutoff_time, config)?;
            cleanup_stats.category_stats.insert(category.clone(), category_stats);
        }
        
        // Clean up temporary files
        if config.cleanup_temp_files {
            let temp_stats = self.cleanup_temp_files()?;
            cleanup_stats.temp_files_cleaned = temp_stats.files_cleaned;
            cleanup_stats.temp_bytes_cleaned = temp_stats.bytes_cleaned;
        }
        
        // Clean up old logs
        if config.remove_test_logs {
            let log_stats = self.cleanup_test_logs(cutoff_time)?;
            cleanup_stats.log_files_cleaned = log_stats.files_cleaned;
            cleanup_stats.log_bytes_cleaned = log_stats.bytes_cleaned;
        }
        
        let cleanup_time = crate::hal::get_current_time_ms() - start_time;
        cleanup_stats.total_cleanup_time_ms = cleanup_time;
        
        info!("Test data cleanup completed in {}ms", cleanup_time);
        Ok(cleanup_stats)
    }

    /// Clean up data for a specific category
    fn cleanup_category_data(&self, category: &TestDataCategory, cutoff_time: u64, 
                           config: &CleanupConfig) -> Result<CategoryCleanupStats> {
        let mut stats = CategoryCleanupStats::default();
        
        let category_dir = match category {
            TestDataCategory::Admin => format!("{}/admin", self.data_directory),
            TestDataCategory::Security => format!("{}/security", self.data_directory),
            TestDataCategory::Update => format!("{}/update", self.data_directory),
            TestDataCategory::System => format!("{}/system", self.data_directory),
            TestDataCategory::Performance => format!("{}/performance", self.data_directory),
            TestDataCategory::Integration => format!("{}/integration", self.data_directory),
        };
        
        // Get list of files in category directory
        let files_result = crate::filesystem::list_directory(&category_dir);
        if let Ok(files) = files_result {
            for file_info in files {
                if file_info.is_file() {
                    let file_path = format!("{}/{}", category_dir, file_info.name);
                    let file_time = file_info.modification_time;
                    
                    if file_time < cutoff_time {
                        if !config.preserve_critical_data || !self.is_critical_file(&file_path) {
                            let file_size = file_info.size;
                            let _ = crate::filesystem::delete_file(&file_path);
                            
                            stats.files_cleaned += 1;
                            stats.bytes_cleaned += file_size;
                        }
                    }
                }
            }
        }
        
        Ok(stats)
    }

    /// Check if a file is critical and should be preserved
    fn is_critical_file(&self, file_path: &str) -> bool {
        // Preserve tracking database and configuration files
        file_path.contains("tracking.db") || 
        file_path.contains("config.json") ||
        file_path.ends_with("_baseline.json") // Performance baselines
    }

    /// Clean up temporary files
    fn cleanup_temp_files(&self) -> Result<TempCleanupStats> {
        let mut stats = TempCleanupStats::default();
        let temp_dir = format!("{}/temp", self.data_directory);
        
        let files_result = crate::filesystem::list_directory(&temp_dir);
        if let Ok(files) = files_result {
            for file_info in files {
                if file_info.is_file() {
                    let file_path = format!("{}/{}", temp_dir, file_info.name);
                    let _ = crate::filesystem::delete_file(&file_path);
                    
                    stats.files_cleaned += 1;
                    stats.bytes_cleaned += file_info.size;
                }
            }
        }
        
        Ok(stats)
    }

    /// Clean up test log files
    fn cleanup_test_logs(&self, cutoff_time: u64) -> Result<LogCleanupStats> {
        let mut stats = LogCleanupStats::default();
        let log_dir = format!("{}/logs", self.data_directory);
        
        let files_result = crate::filesystem::list_directory(&log_dir);
        if let Ok(files) = files_result {
            for file_info in files {
                if file_info.is_file() {
                    let file_path = format!("{}/{}", log_dir, file_info.name);
                    let file_time = file_info.modification_time;
                    
                    if file_time < cutoff_time {
                        let _ = crate::filesystem::delete_file(&file_path);
                        
                        stats.files_cleaned += 1;
                        stats.bytes_cleaned += file_info.size;
                    }
                }
            }
        }
        
        Ok(stats)
    }

    /// Validate test data integrity
    pub fn validate_test_data(&self) -> Result<DataValidationResult> {
        info!("Validating test data integrity...");
        let start_time = crate::hal::get_current_time_ms();
        
        let mut issues_found = Vec::new();
        let mut valid_records = 0;
        let mut total_records = 0;
        
        // Validate by category
        for category in &[TestDataCategory::Admin, TestDataCategory::Security, 
                         TestDataCategory::System, TestDataCategory::Performance] {
            let category_validation = self.validate_category_data(category)?;
            issues_found.extend(category_validation.issues);
            valid_records += category_validation.valid_records;
            total_records += category_validation.total_records;
        }
        
        // Calculate integrity score
        let integrity_score = if total_records > 0 {
            (valid_records as f64 / total_records as f64) * 100.0
        } else {
            100.0
        };
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        if integrity_score < 90.0 {
            recommendations.push("Consider running test data cleanup and reseeding".to_string());
        }
        if total_records == 0 {
            recommendations.push("No test data found. Consider seeding test data".to_string());
        }
        
        let validation_time = crate::hal::get_current_time_ms() - start_time;
        info!("Data validation completed in {}ms - Integrity score: {:.1}%", 
              validation_time, integrity_score);
        
        Ok(DataValidationResult {
            is_valid: integrity_score >= 80.0,
            integrity_score,
            issues_found,
            recommendations,
        })
    }

    /// Validate data for a specific category
    fn validate_category_data(&self, category: &TestDataCategory) -> Result<CategoryValidationResult> {
        let category_dir = match category {
            TestDataCategory::Admin => format!("{}/admin", self.data_directory),
            TestDataCategory::Security => format!("{}/security", self.data_directory),
            TestDataCategory::Update => format!("{}/update", self.data_directory),
            TestDataCategory::System => format!("{}/system", self.data_directory),
            TestDataCategory::Performance => format!("{}/performance", self.data_directory),
            TestDataCategory::Integration => format!("{}/integration", self.data_directory),
        };
        
        let mut issues = Vec::new();
        let mut valid_records = 0;
        let mut total_records = 0;
        
        let files_result = crate::filesystem::list_directory(&category_dir);
        if let Ok(files) = files_result {
            for file_info in files {
                if file_info.is_file() {
                    total_records += 1;
                    let file_path = format!("{}/{}", category_dir, file_info.name);
                    
                    // Check file existence
                    if !crate::filesystem::file_exists(&file_path) {
                        issues.push(format!("File does not exist: {}", file_path));
                        continue;
                    }
                    
                    // Check file readability
                    let read_result = crate::filesystem::read_file(&file_path);
                    if let Err(e) = read_result {
                        issues.push(format!("File not readable {}: {:?}", file_path, e));
                        continue;
                    }
                    
                    // Check file integrity (basic checks)
                    if file_info.size == 0 {
                        issues.push(format!("Empty file detected: {}", file_path));
                        continue;
                    }
                    
                    valid_records += 1;
                }
            }
        }
        
        Ok(CategoryValidationResult {
            issues,
            valid_records,
            total_records,
        })
    }

    /// Get test data statistics
    pub fn get_test_data_statistics(&self) -> Result<TestDataStatistics> {
        let mut category_breakdown = alloc::collections::BTreeMap::new();
        let mut total_records = 0;
        let mut total_size_bytes = 0;
        
        // Collect statistics by category
        for category in &[TestDataCategory::Admin, TestDataCategory::Security, 
                         TestDataCategory::System, TestDataCategory::Performance] {
            let category_stats = self.get_category_statistics(category)?;
            category_breakdown.insert(category.clone(), category_stats);
            total_records += category_stats.record_count;
            total_size_bytes += category_stats.total_size_bytes;
        }
        
        // Calculate age distribution
        let current_time = crate::hal::get_current_time_ms();
        let age_distribution = AgeDistribution {
            less_than_1_day: 0,
            less_than_1_week: 0,
            less_than_1_month: 0,
            older_than_1_month: 0,
        };
        
        Ok(TestDataStatistics {
            total_records,
            total_size_bytes,
            category_breakdown,
            age_distribution,
        })
    }

    /// Get statistics for a specific category
    fn get_category_statistics(&self, category: &TestDataCategory) -> Result<CategoryStats> {
        let category_dir = match category {
            TestDataCategory::Admin => format!("{}/admin", self.data_directory),
            TestDataCategory::Security => format!("{}/security", self.data_directory),
            TestDataCategory::Update => format!("{}/update", self.data_directory),
            TestDataCategory::System => format!("{}/system", self.data_directory),
            TestDataCategory::Performance => format!("{}/performance", self.data_directory),
            TestDataCategory::Integration => format!("{}/integration", self.data_directory),
        };
        
        let mut record_count = 0;
        let mut total_size_bytes = 0;
        let mut oldest_age_days = 0;
        let mut newest_age_days = 0;
        
        let files_result = crate::filesystem::list_directory(&category_dir);
        if let Ok(files) = files_result {
            for file_info in files {
                if file_info.is_file() {
                    record_count += 1;
                    total_size_bytes += file_info.size;
                    
                    let age_days = (crate::hal::get_current_time_ms() - file_info.modification_time) 
                                  / (24 * 60 * 60 * 1000);
                    
                    if oldest_age_days == 0 || age_days > oldest_age_days as u64 {
                        oldest_age_days = age_days as usize;
                    }
                    if newest_age_days == 0 || age_days < newest_age_days as u64 {
                        newest_age_days = age_days as usize;
                    }
                }
            }
        }
        
        let avg_record_size = if record_count > 0 { total_size_bytes / record_count } else { 0 };
        
        Ok(CategoryStats {
            record_count,
            total_size_bytes,
            avg_record_size,
            oldest_record_age_days: oldest_age_days,
            newest_record_age_days: newest_age_days,
        })
    }

    /// Calculate simple checksum for data integrity
    fn calculate_checksum(&self, data: &str) -> String {
        // Simple checksum calculation - in real implementation would use proper hashing
        let mut checksum = 0u32;
        for byte in data.bytes() {
            checksum = checksum.wrapping_add(byte as u32);
        }
        format!("{:08x}", checksum)
    }
}

/// Supporting structures for cleanup operations
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub category_stats: alloc::collections::BTreeMap<TestDataCategory, CategoryCleanupStats>,
    pub temp_files_cleaned: usize,
    pub temp_bytes_cleaned: usize,
    pub log_files_cleaned: usize,
    pub log_bytes_cleaned: usize,
    pub total_cleanup_time_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct CategoryCleanupStats {
    pub files_cleaned: usize,
    pub bytes_cleaned: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TempCleanupStats {
    pub files_cleaned: usize,
    pub bytes_cleaned: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LogCleanupStats {
    pub files_cleaned: usize,
    pub bytes_cleaned: usize,
}

#[derive(Debug, Clone)]
pub struct CategoryValidationResult {
    pub issues: Vec<String>,
    pub valid_records: usize,
    pub total_records: usize,
}

/// Global test data manager instance
static TEST_DATA_MANAGER: spin::Mutex<Option<TestDataManager>> = spin::Mutex::new(None);

/// Initialize global test data manager
pub fn init_test_data_manager(data_directory: String, retention_policy: DataRetentionPolicy) -> Result<()> {
    let mut manager_guard = TEST_DATA_MANAGER.lock();
    let mut manager = TestDataManager::new(data_directory, retention_policy);
    manager.initialize()?;
    *manager_guard = Some(manager);
    Ok(())
}

/// Get global test data manager
pub fn get_test_data_manager() -> Option<TestDataManager> {
    let manager_guard = TEST_DATA_MANAGER.lock();
    manager_guard.clone()
}

/// Clean up all test data
pub fn cleanup_test_data(test_data_dir: &Option<String>) -> Result<()> {
    info!("Cleaning up test data...");
    
    if let Some(dir) = test_data_dir {
        // Remove entire test data directory
        let _ = crate::filesystem::delete_directory(dir);
        
        // Clear global manager
        let mut manager_guard = TEST_DATA_MANAGER.lock();
        *manager_guard = None;
        
        info!("Test data cleanup completed");
    } else {
        warn!("No test data directory specified for cleanup");
    }
    
    Ok(())
}

/// Default test data seeding configuration
pub fn get_default_seeding_config() -> TestDataSeedingConfig {
    TestDataSeedingConfig {
        seed_admin_data: true,
        seed_security_data: true,
        seed_system_data: true,
        seed_performance_baselines: true,
        num_test_users: 20,
        num_test_services: 10,
        num_test_policies: 5,
    }
}

/// Quick test data setup for development
pub fn setup_quick_test_data() -> Result<Vec<TestDataRecord>> {
    let manager = get_test_data_manager()
        .ok_or_else(|| KernelError::InitializationFailed)?;
    
    let config = TestDataSeedingConfig {
        seed_admin_data: true,
        seed_security_data: true,
        seed_system_data: false,
        seed_performance_baselines: false,
        num_test_users: 5,
        num_test_services: 3,
        num_test_policies: 2,
    };
    
    manager.seed_test_data(config)
}

/// Comprehensive test data setup for full integration tests
pub fn setup_comprehensive_test_data() -> Result<Vec<TestDataRecord>> {
    let manager = get_test_data_manager()
        .ok_or_else(|| KernelError::InitializationFailed)?;
    
    manager.seed_test_data(get_default_seeding_config())
}
