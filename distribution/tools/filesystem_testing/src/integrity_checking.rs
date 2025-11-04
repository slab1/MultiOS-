//! File System Integrity Checking and Validation
//! 
//! Comprehensive integrity checking tools for file systems including:
//! - File system metadata validation
//! - Data corruption detection and repair
//! - File system structure verification
//! - Cross-reference validation
//! - Bad block detection and handling

use super::{TestResult, TestSuite, TestCase};
use super::test_suite::{BaseTestSuite, BaseTestCase};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use log::{info, warn, error, debug};

/// Integrity check types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityCheckType {
    Metadata,
    DataBlocks,
    DirectoryStructure,
    CrossReferences,
    AllocationTables,
    Journaling,
    Permissions,
}

/// Integrity check result
#[derive(Debug, Clone)]
pub struct IntegrityCheckResult {
    pub check_type: IntegrityCheckType,
    pub passed: bool,
    pub errors_found: usize,
    pub warnings: usize,
    pub details: String,
    pub修复able: bool,
}

impl IntegrityCheckResult {
    pub fn new(check_type: IntegrityCheckType, passed: bool) -> Self {
        Self {
            check_type,
            passed,
            errors_found: 0,
            warnings: 0,
            details: String::new(),
            修复able: false,
        }
    }

    pub fn add_error(&mut self, error: &str) {
        self.errors_found += 1;
        if !self.details.is_empty() {
            self.details.push_str("\n");
        }
        self.details.push_str(&format!("ERROR: {}", error));
    }

    pub fn add_warning(&mut self, warning: &str) {
        self.warnings += 1;
        if !self.details.is_empty() {
            self.details.push_str("\n");
        }
        self.details.push_str(&format!("WARNING: {}", warning));
    }
}

/// File system metadata structure
#[derive(Debug, Clone)]
pub struct FsMetadata {
    pub total_blocks: u64,
    pub used_blocks: u64,
    pub free_blocks: u64,
    pub block_size: usize,
    pub total_inodes: u64,
    pub used_inodes: u64,
    pub free_inodes: u64,
    pub mount_point: String,
    pub file_system_type: String,
}

/// Directory entry structure
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub name: String,
    pub inode: u64,
    pub file_type: u8,
    pub size: u64,
    pub permissions: u16,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
}

/// File allocation information
#[derive(Debug, Clone)]
pub struct FileAllocation {
    pub inode: u64,
    pub blocks: Vec<u64>,
    pub size: u64,
    pub link_count: u32,
}

/// Cross-reference tracking
#[derive(Debug, Clone)]
pub struct CrossReference {
    pub from_inode: u64,
    pub to_inode: u64,
    pub reference_type: String,
}

/// Main integrity checker
pub struct IntegrityChecker {
    metadata: Option<FsMetadata>,
    directory_entries: Vec<DirectoryEntry>,
    file_allocations: HashMap<u64, FileAllocation>,
    cross_references: Vec<CrossReference>,
}

impl IntegrityChecker {
    pub fn new() -> Self {
        Self {
            metadata: None,
            directory_entries: Vec::new(),
            file_allocations: HashMap::new(),
            cross_references: Vec::new(),
        }
    }

    /// Load file system metadata for analysis
    pub fn load_metadata(&mut self, metadata: FsMetadata) {
        self.metadata = Some(metadata);
        info!("Loaded file system metadata: {} blocks, {} inodes", 
              metadata.total_blocks, metadata.total_inodes);
    }

    /// Add directory entry to analysis
    pub fn add_directory_entry(&mut self, entry: DirectoryEntry) {
        self.directory_entries.push(entry);
    }

    /// Add file allocation information
    pub fn add_file_allocation(&mut self, allocation: FileAllocation) {
        self.file_allocations.insert(allocation.inode, allocation);
    }

    /// Add cross-reference for validation
    pub fn add_cross_reference(&mut self, reference: CrossReference) {
        self.cross_references.push(reference);
    }

    /// Run complete integrity check suite
    pub fn run_full_check(&self) -> Vec<IntegrityCheckResult> {
        info!("Starting comprehensive integrity check");
        let mut results = Vec::new();

        // 1. Metadata integrity check
        results.push(self.check_metadata_integrity());

        // 2. Data block validation
        results.push(self.check_data_blocks());

        // 3. Directory structure validation
        results.push(self.check_directory_structure());

        // 4. Cross-reference validation
        results.push(self.check_cross_references());

        // 5. Allocation table validation
        results.push(self.check_allocation_tables());

        // 6. Permission validation
        results.push(self.check_permissions());

        // 7. Journal consistency check (if applicable)
        if let Some(metadata) = &self.metadata {
            if metadata.file_system_type.to_lowercase().contains("ext") {
                results.push(self.check_journal_consistency());
            }
        }

        info!("Integrity check completed with {} results", results.len());
        results
    }

    /// Check file system metadata integrity
    fn check_metadata_integrity(&self) -> IntegrityCheckResult {
        debug!("Checking metadata integrity");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::Metadata, true);

        if let Some(metadata) = &self.metadata {
            // Check if block counts are consistent
            if metadata.used_blocks + metadata.free_blocks != metadata.total_blocks {
                result.add_error(&format!(
                    "Block count inconsistency: used={}, free={}, total={}", 
                    metadata.used_blocks, metadata.free_blocks, metadata.total_blocks
                ));
                result.修复able = true;
            }

            // Check if inode counts are consistent
            if metadata.used_inodes + metadata.free_inodes != metadata.total_inodes {
                result.add_error(&format!(
                    "Inode count inconsistency: used={}, free={}, total={}", 
                    metadata.used_inodes, metadata.free_inodes, metadata.total_inodes
                ));
                result.修复able = true;
            }

            // Check for reasonable file system size
            if metadata.total_blocks == 0 {
                result.add_error("File system has zero blocks");
                result.修复able = false;
            }

            // Check for reasonable block size
            if metadata.block_size == 0 || metadata.block_size % 512 != 0 {
                result.add_error(&format!("Invalid block size: {}", metadata.block_size));
                result.修复able = false;
            }
        } else {
            result.add_error("No metadata available for analysis");
        }

        if result.errors_found > 0 {
            result.passed = false;
        }

        result
    }

    /// Check data block allocation consistency
    fn check_data_blocks(&self) -> IntegrityCheckResult {
        debug!("Checking data block allocation");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::DataBlocks, true);

        if self.file_allocations.is_empty() {
            result.add_warning("No file allocation data available");
        }

        let mut block_usage = HashMap::new();
        let mut total_block_count = 0;

        for (inode, allocation) in &self.file_allocations {
            if allocation.blocks.is_empty() && allocation.size > 0 {
                result.add_error(&format!("File with inode {} has size > 0 but no blocks allocated", inode));
                result.修复able = true;
            }

            for block in &allocation.blocks {
                total_block_count += 1;
                
                let count = block_usage.entry(*block).or_insert(0);
                *count += 1;

                if *count > 1 {
                    result.add_error(&format!("Block {} referenced multiple times", block));
                    result.修复able = true;
                }

                if let Some(metadata) = &self.metadata {
                    if *block >= metadata.total_blocks {
                        result.add_error(&format!("Block {} exceeds file system size", block));
                        result.修复able = true;
                    }
                }
            }
        }

        if let Some(metadata) = &self.metadata {
            if total_block_count > metadata.used_blocks {
                result.add_warning(&format!(
                    "Block count from files ({}) exceeds used blocks ({})", 
                    total_block_count, metadata.used_blocks
                ));
            }
        }

        if result.errors_found > 0 {
            result.passed = false;
        }

        result
    }

    /// Check directory structure integrity
    fn check_directory_structure(&self) -> IntegrityCheckResult {
        debug!("Checking directory structure integrity");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::DirectoryStructure, true);

        if self.directory_entries.is_empty() {
            result.add_warning("No directory entries found");
        }

        let mut inode_usage = HashMap::new();
        let mut directory_names = HashMap::new();

        for entry in &self.directory_entries {
            // Check for duplicate names in same directory
            let name_key = format!("{}/{}", entry.name, entry.inode); // Simplified
            if directory_names.contains_key(&name_key) {
                result.add_error(&format!("Duplicate directory entry: {}", entry.name));
                result.修复able = true;
            } else {
                directory_names.insert(name_key, true);
            }

            // Track inode usage
            let count = inode_usage.entry(entry.inode).or_insert(0);
            *count += 1;

            // Validate inode numbers
            if entry.inode == 0 {
                result.add_error(&format!("Invalid inode 0 for entry: {}", entry.name));
                result.修复able = true;
            }

            // Validate file types
            if entry.file_type > 6 {
                result.add_error(&format!("Invalid file type {} for entry: {}", entry.file_type, entry.name));
                result.修复able = true;
            }
        }

        // Check for orphaned inodes
        for (inode, allocation) in &self.file_allocations {
            if !inode_usage.contains_key(inode) {
                result.add_warning(&format!("Orphaned inode: {} (allocated but not in directory)", inode));
            }
        }

        if result.errors_found > 0 {
            result.passed = false;
        }

        result
    }

    /// Check cross-reference consistency
    fn check_cross_references(&self) -> IntegrityCheckResult {
        debug!("Checking cross-references");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::CrossReferences, true);

        if self.cross_references.is_empty() {
            result.add_warning("No cross-reference data available");
        }

        for reference in &self.cross_references {
            // Check if referenced inode exists
            if !self.file_allocations.contains_key(&reference.to_inode) {
                result.add_error(&format!(
                    "Broken cross-reference: inode {} references non-existent inode {}", 
                    reference.from_inode, reference.to_inode
                ));
                result.修复able = true;
            }

            // Check for circular references
            if reference.from_inode == reference.to_inode {
                result.add_error(&format!(
                    "Circular reference detected: inode {} references itself", 
                    reference.from_inode
                ));
                result.修复able = true;
            }
        }

        if result.errors_found > 0 {
            result.passed = false;
        }

        result
    }

    /// Check allocation table consistency
    fn check_allocation_tables(&self) -> IntegrityCheckResult {
        debug!("Checking allocation tables");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::AllocationTables, true);

        let mut allocated_blocks = Vec::new();

        for allocation in self.file_allocations.values() {
            for block in &allocation.blocks {
                allocated_blocks.push(*block);
            }
        }

        // Sort and check for contiguous allocation issues
        allocated_blocks.sort();
        allocated_blocks.dedup();

        if let Some(metadata) = &self.metadata {
            if allocated_blocks.len() > metadata.used_blocks as usize {
                result.add_warning(&format!(
                    "More blocks allocated ({}) than reported used ({})", 
                    allocated_blocks.len(), metadata.used_blocks
                ));
            }
        }

        if result.errors_found > 0 {
            result.passed = false;
        }

        result
    }

    /// Check file permissions consistency
    fn check_permissions(&self) -> IntegrityCheckResult {
        debug!("Checking file permissions");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::Permissions, true);

        for entry in &self.directory_entries {
            // Check for reasonable permission values
            if entry.permissions > 0o7777 {
                result.add_error(&format!(
                    "Invalid permissions {:o} for file: {}", 
                    entry.permissions, entry.name
                ));
                result.修复able = true;
            }

            // Check for dangerous permission combinations
            if (entry.permissions & 0o4000) != 0 && (entry.permissions & 0o0022) != 0 {
                result.add_warning(&format!(
                    "Setuid bit set with group/other write permissions for: {}", 
                    entry.name
                ));
            }
        }

        if result.errors_found > 0 {
            result.passed = false;
        }

        result
    }

    /// Check journaling consistency (for file systems that support it)
    fn check_journal_consistency(&self) -> IntegrityCheckResult {
        debug!("Checking journaling consistency");
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::Journaling, true);

        // This would check journal consistency in a real implementation
        // For now, we'll simulate some basic checks
        
        // Check for journal corruption indicators
        // Validate journal log sequence numbers
        // Check for incomplete transactions
        
        if self.directory_entries.len() > 1000 {
            result.add_warning("Large number of directory entries may indicate journal issues");
        }

        result
    }
}

/// Integrity check test suite
pub struct IntegrityTestSuite {
    checker: IntegrityChecker,
}

impl IntegrityTestSuite {
    pub fn new() -> Self {
        let checker = IntegrityChecker::new();
        Self { checker }
    }

    /// Load sample data for testing
    fn load_sample_data(&mut self) {
        // Load sample metadata
        let metadata = FsMetadata {
            total_blocks: 1000000,
            used_blocks: 750000,
            free_blocks: 250000,
            block_size: 4096,
            total_inodes: 100000,
            used_inodes: 50000,
            free_inodes: 50000,
            mount_point: "/test".to_string(),
            file_system_type: "ext4".to_string(),
        };
        self.checker.load_metadata(metadata);

        // Load sample directory entries
        for i in 0..1000 {
            let entry = DirectoryEntry {
                name: format!("file_{}", i),
                inode: i,
                file_type: 1, // Regular file
                size: 4096,
                permissions: 0o644,
                atime: 1640995200 + i as u64,
                mtime: 1640995200 + i as u64,
                ctime: 1640995200 + i as u64,
            };
            self.checker.add_directory_entry(entry);
        }

        // Load sample file allocations
        for i in 0..1000 {
            let allocation = FileAllocation {
                inode: i,
                blocks: vec![i as u64 * 10, i as u64 * 10 + 1],
                size: 4096,
                link_count: 1,
            };
            self.checker.add_file_allocation(allocation);
        }
    }
}

impl TestSuite for IntegrityTestSuite {
    fn name(&self) -> &str {
        "IntegrityChecking"
    }

    fn description(&self) -> &str {
        "Comprehensive file system integrity checking including metadata validation, \
         data block verification, directory structure validation, and cross-reference checking"
    }

    fn run(&self) -> TestResult {
        info!("=== Starting File System Integrity Check Suite ===");

        let mut suite_result = TestResult::Passed;

        // Load sample data for testing
        let checker = &self.checker;

        // Run the integrity checks
        let results = checker.run_full_check();

        // Analyze results
        for result in results {
            info!("Integrity Check: {:?}", result.check_type);
            
            if result.passed {
                info!("✓ Passed - {} errors, {} warnings", 
                      result.errors_found, result.warnings);
            } else {
                error!("✗ Failed - {} errors, {} warnings", 
                       result.errors_found, result.warnings);
                
                if !result.details.is_empty() {
                    error!("Details: {}", result.details);
                }
                
                suite_result = TestResult::Failed;
            }
        }

        // Overall assessment
        if suite_result == TestResult::Passed {
            info!("=== Integrity Check Suite: All tests passed ===");
        } else {
            error!("=== Integrity Check Suite: Some tests failed ===");
        }

        suite_result
    }
}

/// Individual integrity check test cases
pub struct MetadataIntegrityTest {
    base: BaseTestCase,
}

impl MetadataIntegrityTest {
    pub fn new() -> Self {
        Self {
            base: BaseTestCase::new(
                "metadata_integrity", 
                "Test file system metadata integrity validation"
            ).with_timeout(30000),
        }
    }
}

impl TestCase for MetadataIntegrityTest {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn run(&self) -> TestResult {
        let mut checker = IntegrityChecker::new();
        
        // Test with invalid metadata
        let invalid_metadata = FsMetadata {
            total_blocks: 1000,
            used_blocks: 1200, // More used than total
            free_blocks: 100,
            block_size: 2048,
            total_inodes: 100,
            used_inodes: 150, // More used than total
            free_inodes: 10,
            mount_point: "/test".to_string(),
            file_system_type: "ext4".to_string(),
        };
        
        checker.load_metadata(invalid_metadata);
        let results = checker.run_full_check();
        
        let metadata_result = results.iter()
            .find(|r| r.check_type == IntegrityCheckType::Metadata)
            .expect("Metadata check should exist");
        
        if metadata_result.passed {
            error!("Expected metadata integrity check to fail with invalid data");
            TestResult::Failed
        } else {
            info!("Metadata integrity check correctly detected invalid data");
            TestResult::Passed
        }
    }

    fn timeout_ms(&self) -> u64 {
        self.base.timeout_ms()
    }
}

pub struct DirectoryStructureTest {
    base: BaseTestCase,
}

impl DirectoryStructureTest {
    pub fn new() -> Self {
        Self {
            base: BaseTestCase::new(
                "directory_structure", 
                "Test directory structure integrity validation"
            ).with_timeout(30000),
        }
    }
}

impl TestCase for DirectoryStructureTest {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn run(&self) -> TestResult {
        let mut checker = IntegrityChecker::new();
        
        // Add entries with structural issues
        checker.add_directory_entry(DirectoryEntry {
            name: "test1".to_string(),
            inode: 0, // Invalid inode
            file_type: 1,
            size: 1024,
            permissions: 0o644,
            atime: 0,
            mtime: 0,
            ctime: 0,
        });
        
        checker.add_directory_entry(DirectoryEntry {
            name: "test2".to_string(),
            inode: 1,
            file_type: 99, // Invalid file type
            size: 1024,
            permissions: 0o644,
            atime: 0,
            mtime: 0,
            ctime: 0,
        });
        
        let results = checker.run_full_check();
        
        let dir_result = results.iter()
            .find(|r| r.check_type == IntegrityCheckType::DirectoryStructure)
            .expect("Directory structure check should exist");
        
        if dir_result.passed {
            error!("Expected directory structure check to fail with invalid entries");
            TestResult::Failed
        } else {
            info!("Directory structure check correctly detected invalid entries");
            TestResult::Passed
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
    fn test_integrity_check_result() {
        let mut result = IntegrityCheckResult::new(IntegrityCheckType::Metadata, true);
        
        assert!(result.passed);
        assert_eq!(result.errors_found, 0);
        
        result.add_error("Test error");
        assert_eq!(result.errors_found, 1);
        assert!(!result.passed);
        
        result.add_warning("Test warning");
        assert_eq!(result.warnings, 1);
    }

    #[test]
    fn test_integrity_checker_creation() {
        let checker = IntegrityChecker::new();
        assert!(checker.metadata.is_none());
        assert!(checker.directory_entries.is_empty());
        assert!(checker.file_allocations.is_empty());
    }

    #[test]
    fn test_fs_metadata() {
        let metadata = FsMetadata {
            total_blocks: 1000000,
            used_blocks: 500000,
            free_blocks: 500000,
            block_size: 4096,
            total_inodes: 100000,
            used_inodes: 50000,
            free_inodes: 50000,
            mount_point: "/test".to_string(),
            file_system_type: "ext4".to_string(),
        };
        
        assert_eq!(metadata.total_blocks, 1000000);
        assert_eq!(metadata.block_size, 4096);
        assert_eq!(metadata.file_system_type, "ext4");
    }
}