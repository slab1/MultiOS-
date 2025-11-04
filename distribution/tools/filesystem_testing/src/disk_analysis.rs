//! Disk Analysis Tools
//! 
//! Comprehensive disk analysis and monitoring tools including:
//! - Disk usage analysis and reporting
//! - Fragmentation detection and measurement
//! - Bad block detection and mapping
//! - File system health monitoring
//! - Usage pattern analysis
//! - Disk performance analysis
//! - Capacity planning and forecasting

use super::{TestResult, TestSuite, TestCase};
use super::test_suite::{BaseTestSuite, BaseTestCase};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::{HashMap, BTreeMap};
use log::{info, warn, error, debug};

/// Disk information structure
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total_capacity_gb: f64,
    pub used_capacity_gb: f64,
    pub free_capacity_gb: f64,
    pub available_capacity_gb: f64,
    pub block_size: usize,
    pub total_blocks: u64,
    pub total_inodes: u64,
    pub used_inodes: u64,
    pub file_system_type: String,
    pub mount_point: String,
    pub disk_model: String,
    pub serial_number: String,
}

/// File system usage statistics
#[derive(Debug, Clone)]
pub struct FsUsageStats {
    pub directory_count: u64,
    pub file_count: u64,
    pub symlink_count: u64,
    pub total_file_size_mb: f64,
    pub average_file_size_kb: f64,
    pub largest_file_mb: f64,
    pub smallest_file_kb: f64,
    pub oldest_file_age_days: u64,
    pub newest_file_age_hours: u64,
}

/// Directory usage information
#[derive(Debug, Clone)]
pub struct DirectoryInfo {
    pub path: String,
    pub file_count: u64,
    pub total_size_mb: f64,
    pub largest_file_mb: f64,
    pub files_per_level: BTreeMap<usize, u64>,
}

/// Fragmentation information
#[derive(Debug, Clone)]
pub struct FragmentationInfo {
    pub fragmentation_percent: f64,
    pub fragmented_files: u64,
    pub average_fragments_per_file: f64,
    pub severely_fragmented_files: u64,
    pub largest_fragmented_file_mb: f64,
    pub free_space_fragmentation: f64,
}

/// Bad block information
#[derive(Debug, Clone)]
pub struct BadBlockInfo {
    pub total_bad_blocks: u64,
    pub bad_blocks_in_use: u64,
    pub isolated_bad_blocks: u64,
    pub remapped_blocks: u64,
    pub last_scan_date: String,
    pub scan_health: String,
}

/// Disk usage analysis
#[derive(Debug, Clone)]
pub struct DiskUsageAnalysis {
    pub disk_info: DiskInfo,
    pub usage_stats: FsUsageStats,
    pub top_directories: Vec<DirectoryInfo>,
    pub file_type_distribution: HashMap<String, u64>,
    pub size_distribution: BTreeMap<String, u64>,
    pub age_distribution: HashMap<String, u64>,
}

/// Disk performance metrics
#[derive(Debug, Clone)]
pub struct DiskPerformanceMetrics {
    pub average_response_time_ms: f64,
    pub read_operations_per_second: f64,
    pub write_operations_per_second: f64,
    pub queue_depth: f64,
    pub read_cache_hit_ratio: f64,
    pub write_cache_hit_ratio: f64,
    pub disk_utilization_percent: f64,
}

/// Disk analysis configuration
#[derive(Debug, Clone)]
pub struct DiskAnalysisConfig {
    pub analyze_large_directories: bool,
    pub detect_fragmentation: bool,
    pub scan_for_bad_blocks: bool,
    pub analyze_file_age: bool,
    pub include_hidden_files: bool,
    pub max_directory_depth: usize,
    pub min_file_size_kb: usize,
    pub analysis_timeout_ms: u64,
}

impl Default for DiskAnalysisConfig {
    fn default() -> Self {
        Self {
            analyze_large_directories: true,
            detect_fragmentation: true,
            scan_for_bad_blocks: true,
            analyze_file_age: true,
            include_hidden_files: false,
            max_directory_depth: 10,
            min_file_size_kb: 1,
            analysis_timeout_ms: 60000,
        }
    }
}

/// Disk analysis tool
pub struct DiskAnalyzer {
    config: DiskAnalysisConfig,
    scan_progress: f64,
    current_phase: String,
}

impl DiskAnalyzer {
    pub fn new(config: DiskAnalysisConfig) -> Self {
        Self {
            config,
            scan_progress: 0.0,
            current_phase: String::new(),
        }
    }

    /// Perform comprehensive disk analysis
    pub fn analyze_disk(&mut self, mount_point: &str) -> Result<DiskUsageAnalysis, &'static str> {
        info!("Starting comprehensive disk analysis for: {}", mount_point);

        // Phase 1: Basic disk information gathering
        self.update_progress(0.0, "Gathering disk information");
        let disk_info = self.gather_disk_info(mount_point)?;

        // Phase 2: Usage statistics calculation
        self.update_progress(20.0, "Calculating usage statistics");
        let usage_stats = self.calculate_usage_stats(mount_point)?;

        // Phase 3: Directory analysis
        self.update_progress(40.0, "Analyzing directory structure");
        let top_directories = self.analyze_directories(mount_point)?;

        // Phase 4: File type analysis
        self.update_progress(60.0, "Analyzing file types");
        let file_type_distribution = self.analyze_file_types(mount_point)?;

        // Phase 5: Size distribution analysis
        self.update_progress(80.0, "Analyzing size distribution");
        let size_distribution = self.analyze_size_distribution(mount_point)?;

        // Phase 6: Age distribution analysis
        self.update_progress(90.0, "Analyzing file age distribution");
        let age_distribution = self.analyze_file_age(mount_point)?;

        self.update_progress(100.0, "Analysis complete");

        Ok(DiskUsageAnalysis {
            disk_info,
            usage_stats,
            top_directories,
            file_type_distribution,
            size_distribution,
            age_distribution,
        })
    }

    /// Gather basic disk information
    fn gather_disk_info(&self, mount_point: &str) -> Result<DiskInfo, &'static str> {
        debug!("Gathering disk information for: {}", mount_point);
        
        // In a real implementation, this would query actual disk information
        // For simulation, we'll return mock data
        
        Ok(DiskInfo {
            total_capacity_gb: 1000.0,
            used_capacity_gb: 750.0,
            free_capacity_gb: 250.0,
            available_capacity_gb: 200.0,
            block_size: 4096,
            total_blocks: 262144000,
            total_inodes: 10000000,
            used_inodes: 5000000,
            file_system_type: "ext4".to_string(),
            mount_point: mount_point.to_string(),
            disk_model: "Samsung SSD 970 EVO Plus".to_string(),
            serial_number: "S4EMNX0N123456".to_string(),
        })
    }

    /// Calculate file system usage statistics
    fn calculate_usage_stats(&self, mount_point: &str) -> Result<FsUsageStats, &'static str> {
        debug!("Calculating usage statistics for: {}", mount_point);
        
        // Simulate usage statistics calculation
        let directory_count = 5000;
        let file_count = 50000;
        let symlink_count = 500;
        let total_file_size_mb = 50000.0; // 50GB
        let average_file_size_kb = total_file_size_mb * 1024.0 / file_count as f64;
        let largest_file_mb = 2000.0; // 2GB
        let smallest_file_kb = 1.0;
        let oldest_file_age_days = 365;
        let newest_file_age_hours = 1;

        Ok(FsUsageStats {
            directory_count,
            file_count,
            symlink_count,
            total_file_size_mb,
            average_file_size_kb,
            largest_file_mb,
            smallest_file_kb,
            oldest_file_age_days,
            newest_file_age_hours,
        })
    }

    /// Analyze directory structure
    fn analyze_directories(&self, mount_point: &str) -> Result<Vec<DirectoryInfo>, &'static str> {
        debug!("Analyzing directory structure for: {}", mount_point);
        
        let mut directories = Vec::new();
        
        // Simulate analysis of top directories
        let sample_directories = vec![
            ("/home", 15000, 15000.0),
            ("/var", 8000, 8000.0),
            ("/usr", 25000, 25000.0),
            ("/tmp", 1000, 100.0),
            ("/opt", 2000, 2000.0),
        ];

        for (path, file_count, total_size_mb) in sample_directories {
            let mut files_per_level = BTreeMap::new();
            files_per_level.insert(1, file_count / 2);
            files_per_level.insert(2, file_count / 4);
            files_per_level.insert(3, file_count / 8);

            directories.push(DirectoryInfo {
                path: path.to_string(),
                file_count,
                total_size_mb,
                largest_file_mb: total_size_mb / file_count as f64 * 100.0,
                files_per_level,
            });
        }

        Ok(directories)
    }

    /// Analyze file type distribution
    fn analyze_file_types(&self, mount_point: &str) -> Result<HashMap<String, u64>, &'static str> {
        debug!("Analyzing file types for: {}", mount_point);
        
        let mut distribution = HashMap::new();
        
        distribution.insert("txt".to_string(), 10000);
        distribution.insert("jpg".to_string(), 5000);
        distribution.insert("png".to_string(), 3000);
        distribution.insert("mp4".to_string(), 1000);
        distribution.insert("pdf".to_string(), 2000);
        distribution.insert("doc".to_string(), 1500);
        distribution.insert("xls".to_string(), 500);
        distribution.insert("zip".to_string(), 300);
        distribution.insert("gz".to_string(), 800);
        distribution.insert("bin".to_string(), 500);
        distribution.insert("exe".to_string(), 200);
        distribution.insert("so".to_string(), 150);
        distribution.insert("dll".to_string(), 100);
        distribution.insert("other".to_string(), 19950);

        Ok(distribution)
    }

    /// Analyze file size distribution
    fn analyze_size_distribution(&self, mount_point: &str) -> Result<BTreeMap<String, u64>, &'static str> {
        debug!("Analyzing size distribution for: {}", mount_point);
        
        let mut distribution = BTreeMap::new();
        
        distribution.insert("< 1KB".to_string(), 20000);
        distribution.insert("1-10KB".to_string(), 15000);
        distribution.insert("10-100KB".to_string(), 10000);
        distribution.insert("100KB-1MB".to_string(), 3000);
        distribution.insert("1-10MB".to_string(), 1500);
        distribution.insert("10-100MB".to_string(), 400);
        distribution.insert("100MB-1GB".to_string(), 90);
        distribution.insert("> 1GB".to_string(), 10);

        Ok(distribution)
    }

    /// Analyze file age distribution
    fn analyze_file_age(&self, mount_point: &str) -> Result<HashMap<String, u64>, &'static str> {
        debug!("Analyzing file age for: {}", mount_point);
        
        let mut age_distribution = HashMap::new();
        
        age_distribution.insert("< 1 day".to_string(), 5000);
        age_distribution.insert("1-7 days".to_string(), 3000);
        age_distribution.insert("1-4 weeks".to_string(), 5000);
        age_distribution.insert("1-3 months".to_string(), 8000);
        age_distribution.insert("3-12 months".to_string(), 15000);
        age_distribution.insert("1-3 years".to_string(), 10000);
        age_distribution.insert("> 3 years".to_string(), 4000);

        Ok(age_distribution)
    }

    /// Detect and analyze file system fragmentation
    pub fn analyze_fragmentation(&mut self, mount_point: &str) -> Result<FragmentationInfo, &'static str> {
        info!("Analyzing file system fragmentation for: {}", mount_point);
        
        if !self.config.detect_fragmentation {
            return Err("Fragmentation detection disabled in config");
        }

        // Simulate fragmentation analysis
        let fragmentation_percent = 15.5; // 15.5% fragmentation
        let fragmented_files = 7500;
        let average_fragments_per_file = 2.3;
        let severely_fragmented_files = 500;
        let largest_fragmented_file_mb = 500.0;
        let free_space_fragmentation = 8.2;

        Ok(FragmentationInfo {
            fragmentation_percent,
            fragmented_files,
            average_fragments_per_file,
            severely_fragmented_files,
            largest_fragmented_file_mb,
            free_space_fragmentation,
        })
    }

    /// Scan for bad blocks on the disk
    pub fn scan_bad_blocks(&mut self, device_path: &str) -> Result<BadBlockInfo, &'static str> {
        info!("Scanning for bad blocks on: {}", device_path);
        
        if !self.config.scan_for_bad_blocks {
            return Err("Bad block scanning disabled in config");
        }

        // Simulate bad block scanning
        let total_bad_blocks = 5;
        let bad_blocks_in_use = 2;
        let isolated_bad_blocks = 3;
        let remapped_blocks = 0;
        let last_scan_date = "2024-01-15".to_string();
        let scan_health = "Good".to_string();

        Ok(BadBlockInfo {
            total_bad_blocks,
            bad_blocks_in_use,
            isolated_bad_blocks,
            remapped_blocks,
            last_scan_date,
            scan_health,
        })
    }

    /// Analyze disk performance
    pub fn analyze_performance(&mut self, device_path: &str) -> Result<DiskPerformanceMetrics, &'static str> {
        info!("Analyzing disk performance for: {}", device_path);
        
        // Simulate performance analysis
        let average_response_time_ms = 8.5;
        let read_operations_per_second = 1500.0;
        let write_operations_per_second = 800.0;
        let queue_depth = 2.1;
        let read_cache_hit_ratio = 0.85;
        let write_cache_hit_ratio = 0.72;
        let disk_utilization_percent = 65.0;

        Ok(DiskPerformanceMetrics {
            average_response_time_ms,
            read_operations_per_second,
            write_operations_per_second,
            queue_depth,
            read_cache_hit_ratio,
            write_cache_hit_ratio,
            disk_utilization_percent,
        })
    }

    /// Generate disk usage report
    pub fn generate_report(&self, analysis: &DiskUsageAnalysis) -> String {
        let mut report = String::new();
        
        report.push_str("=== DISK USAGE ANALYSIS REPORT ===\n\n");
        
        // Disk Information
        report.push_str("DISK INFORMATION:\n");
        report.push_str(&format!("  Total Capacity: {:.1} GB\n", analysis.disk_info.total_capacity_gb));
        report.push_str(&format!("  Used Space: {:.1} GB ({:.1}%)\n", 
            analysis.disk_info.used_capacity_gb, 
            analysis.disk_info.used_capacity_gb / analysis.disk_info.total_capacity_gb * 100.0));
        report.push_str(&format!("  Free Space: {:.1} GB\n", analysis.disk_info.free_capacity_gb));
        report.push_str(&format!("  Available Space: {:.1} GB\n", analysis.disk_info.available_capacity_gb));
        report.push_str(&format!("  File System: {}\n", analysis.disk_info.file_system_type));
        report.push_str(&format!("  Block Size: {} bytes\n\n", analysis.disk_info.block_size));
        
        // Usage Statistics
        report.push_str("USAGE STATISTICS:\n");
        report.push_str(&format!("  Total Files: {}\n", analysis.usage_stats.file_count));
        report.push_str(&format!("  Total Directories: {}\n", analysis.usage_stats.directory_count));
        report.push_str(&format!("  Total Size: {:.1} GB\n", analysis.usage_stats.total_file_size_mb / 1024.0));
        report.push_str(&format!("  Average File Size: {:.1} KB\n", analysis.usage_stats.average_file_size_kb));
        report.push_str(&format!("  Largest File: {:.1} MB\n\n", analysis.usage_stats.largest_file_mb));
        
        // Top Directories
        report.push_str("TOP 5 DIRECTORIES BY SIZE:\n");
        for (i, dir) in analysis.top_directories.iter().take(5).enumerate() {
            report.push_str(&format!("  {}. {} - {} files, {:.1} MB\n", 
                i + 1, dir.path, dir.file_count, dir.total_size_mb));
        }
        report.push_str("\n");
        
        // File Type Distribution
        report.push_str("FILE TYPE DISTRIBUTION:\n");
        let mut sorted_types: Vec<_> = analysis.file_type_distribution.iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(a.1));
        
        for (file_type, count) in sorted_types.iter().take(10) {
            report.push_str(&format!("  {}: {} files\n", file_type, count));
        }
        report.push_str("\n");
        
        // Size Distribution
        report.push_str("FILE SIZE DISTRIBUTION:\n");
        for (size_range, count) in &analysis.size_distribution {
            report.push_str(&format!("  {}: {} files\n", size_range, count));
        }
        
        report
    }

    fn update_progress(&mut self, percent: f64, phase: &str) {
        self.scan_progress = percent;
        self.current_phase = phase.to_string();
        debug!("Analysis progress: {:.1}% - {}", percent, phase);
    }
}

/// Disk analysis test suite
pub struct DiskAnalysisSuite {
    analyzer: DiskAnalyzer,
    config: DiskAnalysisConfig,
}

impl DiskAnalysisSuite {
    pub fn new() -> Self {
        let config = DiskAnalysisConfig::default();
        let analyzer = DiskAnalyzer::new(config.clone());
        
        Self {
            analyzer,
            config,
        }
    }

    pub fn with_config(config: DiskAnalysisConfig) -> Self {
        let analyzer = DiskAnalyzer::new(config.clone());
        
        Self {
            analyzer,
            config,
        }
    }
}

impl TestSuite for DiskAnalysisSuite {
    fn name(&self) -> &str {
        "DiskAnalysis"
    }

    fn description(&self) -> &str {
        "Comprehensive disk analysis including usage statistics, directory analysis, \
         fragmentation detection, bad block scanning, and performance monitoring"
    }

    fn run(&self) -> TestResult {
        info!("=== Starting Disk Analysis Test Suite ===");

        // Test 1: Comprehensive disk analysis
        info!("\n1. Running comprehensive disk analysis");
        let mut analyzer = DiskAnalyzer::new(self.config.clone());
        let mount_point = "/test_mount";
        
        match analyzer.analyze_disk(mount_point) {
            Ok(analysis) => {
                info!("✓ Disk analysis completed successfully");
                
                // Display key findings
                info!("  Total capacity: {:.1} GB", analysis.disk_info.total_capacity_gb);
                info!("  Used capacity: {:.1} GB ({:.1}%)", 
                    analysis.disk_info.used_capacity_gb,
                    analysis.disk_info.used_capacity_gb / analysis.disk_info.total_capacity_gb * 100.0);
                info!("  File count: {}", analysis.usage_stats.file_count);
                info!("  Directory count: {}", analysis.usage_stats.directory_count);
                
                // Generate and display report
                let report = analyzer.generate_report(&analysis);
                debug!("Analysis report:\n{}", report);
            }
            Err(e) => {
                error!("✗ Disk analysis failed: {}", e);
                return TestResult::Failed;
            }
        }

        // Test 2: Fragmentation analysis
        if self.config.detect_fragmentation {
            info!("\n2. Running fragmentation analysis");
            let mut analyzer = DiskAnalyzer::new(self.config.clone());
            
            match analyzer.analyze_fragmentation(mount_point) {
                Ok(fragmentation) => {
                    info!("✓ Fragmentation analysis completed");
                    info!("  Overall fragmentation: {:.1}%", fragmentation.fragmentation_percent);
                    info!("  Fragmented files: {}", fragmentation.fragmented_files);
                    info!("  Average fragments per file: {:.1}", fragmentation.average_fragments_per_file);
                    
                    if fragmentation.fragmentation_percent > 30.0 {
                        warn!("  High fragmentation detected - consider defragmentation");
                    }
                }
                Err(e) => {
                    warn!("Fragmentation analysis skipped: {}", e);
                }
            }
        }

        // Test 3: Bad block scanning
        if self.config.scan_for_bad_blocks {
            info!("\n3. Scanning for bad blocks");
            let mut analyzer = DiskAnalyzer::new(self.config.clone());
            
            match analyzer.scan_bad_blocks("/dev/sda") {
                Ok(bad_blocks) => {
                    info!("✓ Bad block scan completed");
                    info!("  Total bad blocks: {}", bad_blocks.total_bad_blocks);
                    info!("  Bad blocks in use: {}", bad_blocks.bad_blocks_in_use);
                    info!("  Scan health: {}", bad_blocks.scan_health);
                    
                    if bad_blocks.total_bad_blocks > 0 {
                        warn!("  Bad blocks detected - disk health concerns");
                    }
                }
                Err(e) => {
                    warn!("Bad block scan skipped: {}", e);
                }
            }
        }

        // Test 4: Performance analysis
        info!("\n4. Analyzing disk performance");
        let mut analyzer = DiskAnalyzer::new(self.config.clone());
        
        match analyzer.analyze_performance("/dev/sda") {
            Ok(performance) => {
                info!("✓ Performance analysis completed");
                info!("  Average response time: {:.1} ms", performance.average_response_time_ms);
                info!("  Read operations/sec: {:.0}", performance.read_operations_per_second);
                info!("  Write operations/sec: {:.0}", performance.write_operations_per_second);
                info!("  Read cache hit ratio: {:.1}%", performance.read_cache_hit_ratio * 100.0);
                info!("  Disk utilization: {:.1}%", performance.disk_utilization_percent);
                
                if performance.average_response_time_ms > 20.0 {
                    warn!("  High response time detected");
                }
                
                if performance.disk_utilization_percent > 80.0 {
                    warn!("  High disk utilization detected");
                }
            }
            Err(e) => {
                warn!("Performance analysis skipped: {}", e);
            }
        }

        info!("=== Disk Analysis Suite Completed Successfully ===");
        TestResult::Passed
    }
}

/// Individual disk analysis test cases
pub struct ComprehensiveDiskAnalysisTest {
    base: BaseTestCase,
    config: DiskAnalysisConfig,
}

impl ComprehensiveDiskAnalysisTest {
    pub fn new() -> Self {
        Self {
            base: BaseTestCase::new(
                "comprehensive_disk_analysis", 
                "Test comprehensive disk usage and health analysis"
            ).with_timeout(90000),
            config: DiskAnalysisConfig::default(),
        }
    }
}

impl TestCase for ComprehensiveDiskAnalysisTest {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn run(&self) -> TestResult {
        let mut analyzer = DiskAnalyzer::new(self.config.clone());
        
        match analyzer.analyze_disk("/test_mount") {
            Ok(analysis) => {
                info!("Comprehensive analysis completed");
                
                // Validate analysis results
                if analysis.usage_stats.file_count == 0 {
                    error!("No files found in analysis");
                    return TestResult::Failed;
                }
                
                if analysis.top_directories.is_empty() {
                    error!("No directory information found");
                    return TestResult::Failed;
                }
                
                TestResult::Passed
            }
            Err(e) => {
                error!("Comprehensive analysis failed: {}", e);
                TestResult::Failed
            }
        }
    }

    fn timeout_ms(&self) -> u64 {
        self.base.timeout_ms()
    }
}

pub struct FragmentationAnalysisTest {
    base: BaseTestCase,
    config: DiskAnalysisConfig,
}

impl FragmentationAnalysisTest {
    pub fn new() -> Self {
        let mut config = DiskAnalysisConfig::default();
        config.detect_fragmentation = true;
        
        Self {
            base: BaseTestCase::new(
                "fragmentation_analysis", 
                "Test file system fragmentation detection and analysis"
            ).with_timeout(60000),
            config,
        }
    }
}

impl TestCase for FragmentationAnalysisTest {
    fn name(&self) -> &str {
        self.base.name()
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn run(&self) -> TestResult {
        let mut analyzer = DiskAnalyzer::new(self.config.clone());
        
        match analyzer.analyze_fragmentation("/test_mount") {
            Ok(fragmentation) => {
                info!("Fragmentation analysis completed");
                
                // Validate fragmentation data
                if fragmentation.fragmentation_percent < 0.0 || fragmentation.fragmentation_percent > 100.0 {
                    error!("Invalid fragmentation percentage: {:.1}%", fragmentation.fragmentation_percent);
                    return TestResult::Failed;
                }
                
                if fragmentation.fragmented_files < 0 {
                    error!("Invalid fragmented files count: {}", fragmentation.fragmented_files);
                    return TestResult::Failed;
                }
                
                TestResult::Passed
            }
            Err(e) => {
                error!("Fragmentation analysis failed: {}", e);
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
    fn test_disk_analysis_config_default() {
        let config = DiskAnalysisConfig::default();
        assert!(config.analyze_large_directories);
        assert!(config.detect_fragmentation);
        assert!(config.scan_for_bad_blocks);
        assert_eq!(config.max_directory_depth, 10);
        assert_eq!(config.min_file_size_kb, 1);
    }

    #[test]
    fn test_disk_info() {
        let disk_info = DiskInfo {
            total_capacity_gb: 1000.0,
            used_capacity_gb: 750.0,
            free_capacity_gb: 250.0,
            available_capacity_gb: 200.0,
            block_size: 4096,
            total_blocks: 262144000,
            total_inodes: 10000000,
            used_inodes: 5000000,
            file_system_type: "ext4".to_string(),
            mount_point: "/test".to_string(),
            disk_model: "Test SSD".to_string(),
            serial_number: "TEST123".to_string(),
        };
        
        assert_eq!(disk_info.total_capacity_gb, 1000.0);
        assert_eq!(disk_info.file_system_type, "ext4");
        assert_eq!(disk_info.block_size, 4096);
    }

    #[test]
    fn test_fs_usage_stats() {
        let stats = FsUsageStats {
            directory_count: 1000,
            file_count: 10000,
            symlink_count: 100,
            total_file_size_mb: 50000.0,
            average_file_size_kb: 5120.0,
            largest_file_mb: 2000.0,
            smallest_file_kb: 1.0,
            oldest_file_age_days: 365,
            newest_file_age_hours: 1,
        };
        
        assert_eq!(stats.file_count, 10000);
        assert_eq!(stats.total_file_size_mb, 50000.0);
        assert!(stats.average_file_size_kb > 0.0);
    }

    #[test]
    fn test_fragmentation_info() {
        let frag_info = FragmentationInfo {
            fragmentation_percent: 15.5,
            fragmented_files: 7500,
            average_fragments_per_file: 2.3,
            severely_fragmented_files: 500,
            largest_fragmented_file_mb: 500.0,
            free_space_fragmentation: 8.2,
        };
        
        assert_eq!(frag_info.fragmentation_percent, 15.5);
        assert_eq!(frag_info.fragmented_files, 7500);
        assert!(frag_info.average_fragments_per_file > 0.0);
    }

    #[test]
    fn test_bad_block_info() {
        let bad_blocks = BadBlockInfo {
            total_bad_blocks: 5,
            bad_blocks_in_use: 2,
            isolated_bad_blocks: 3,
            remapped_blocks: 0,
            last_scan_date: "2024-01-15".to_string(),
            scan_health: "Good".to_string(),
        };
        
        assert_eq!(bad_blocks.total_bad_blocks, 5);
        assert_eq!(bad_blocks.bad_blocks_in_use, 2);
        assert_eq!(bad_blocks.scan_health, "Good");
    }

    #[test]
    fn test_disk_analyzer_creation() {
        let config = DiskAnalysisConfig::default();
        let analyzer = DiskAnalyzer::new(config);
        assert_eq!(analyzer.scan_progress, 0.0);
        assert!(analyzer.current_phase.is_empty());
    }

    #[test]
    fn test_disk_performance_metrics() {
        let metrics = DiskPerformanceMetrics {
            average_response_time_ms: 8.5,
            read_operations_per_second: 1500.0,
            write_operations_per_second: 800.0,
            queue_depth: 2.1,
            read_cache_hit_ratio: 0.85,
            write_cache_hit_ratio: 0.72,
            disk_utilization_percent: 65.0,
        };
        
        assert_eq!(metrics.average_response_time_ms, 8.5);
        assert_eq!(metrics.read_operations_per_second, 1500.0);
        assert_eq!(metrics.disk_utilization_percent, 65.0);
        assert!(metrics.read_cache_hit_ratio <= 1.0);
        assert!(metrics.write_cache_hit_ratio <= 1.0);
    }
}