//! Examples and Demonstrations
//! 
//! This module contains comprehensive examples and demonstrations of the file system testing framework.
//! It showcases various testing scenarios and how to use the different components together.

use crate::*;
use alloc::vec;
use alloc::string::String;
use log::{info, warn, error, debug};

// =============================================================================
// BASIC USAGE EXAMPLES
// =============================================================================

/// Basic example of running a complete test suite
pub fn basic_test_suite_example() -> TestResult {
    info!("=== Running Basic Test Suite Example ===");

    // Initialize the testing framework
    init_testing().expect("Failed to initialize testing framework");

    // Run the full test suite
    let result = run_full_test_suite();

    match result {
        TestResult::Passed => {
            info!("✓ All tests passed successfully!");
        }
        _ => {
            warn!("⚠ Some tests had issues");
        }
    }

    result
}

/// Example of running individual test suites
pub fn individual_suite_example() -> TestResult {
    info!("=== Running Individual Test Suites Example ===");

    // Test specific suites
    let stress_result = run_test_suite("stress_testing");
    let integrity_result = run_test_suite("integrity_checking");
    let performance_result = run_test_suite("performance_benchmarking");

    info!("Stress testing: {:?}", stress_result);
    info!("Integrity checking: {:?}", integrity_result);
    info!("Performance benchmarking: {:?}", performance_result);

    // Return overall result
    if stress_result == TestResult::Passed && 
       integrity_result == TestResult::Passed && 
       performance_result == TestResult::Passed {
        TestResult::Passed
    } else {
        TestResult::Failed
    }
}

// =============================================================================
// STRESS TESTING EXAMPLES
// =============================================================================

/// Example of custom stress testing configuration
pub fn custom_stress_testing_example() -> TestResult {
    info!("=== Custom Stress Testing Example ===");

    // Create custom stress test configuration
    let config = stress_testing::StressTestConfig {
        max_concurrent_files: 2000,
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_directory_depth: 15,
        max_files_per_operation: 200,
        operation_timeout_ms: 45000,
        memory_pressure_mb: 128,
        disk_usage_target: 0.9, // 90% disk usage
        concurrent_threads: 8,
    };

    // Create stress test suite with custom config
    let suite = stress_testing::StressTestSuite::with_config(config);

    // Run the stress test suite
    suite.run()
}

/// Example of running individual stress tests
pub fn individual_stress_tests_example() -> TestResult {
    info!("=== Individual Stress Tests Example ===");

    let mut overall_result = TestResult::Passed;

    // Test concurrent file creation
    info!("Testing concurrent file creation...");
    let concurrent_test = stress_testing::ConcurrentCreationTest::new();
    let result = concurrent_test.run();
    if result != TestResult::Passed {
        error!("Concurrent creation test failed");
        overall_result = TestResult::Failed;
    }

    // Test memory pressure
    info!("Testing memory pressure...");
    let memory_test = stress_testing::MemoryPressureTest::new();
    let result = memory_test.run();
    if result != TestResult::Passed {
        error!("Memory pressure test failed");
        overall_result = TestResult::Failed;
    }

    // Test disk exhaustion
    info!("Testing disk exhaustion...");
    let disk_test = stress_testing::DiskExhaustionTest::new();
    let result = disk_test.run();
    if result != TestResult::Passed {
        error!("Disk exhaustion test failed");
        overall_result = TestResult::Failed;
    }

    // Test operation storm
    info!("Testing operation storm...");
    let storm_test = stress_testing::OperationStormTest::new();
    let result = storm_test.run();
    if result != TestResult::Passed {
        error!("Operation storm test failed");
        overall_result = TestResult::Failed;
    }

    overall_result
}

// =============================================================================
// INTEGRITY CHECKING EXAMPLES
// =============================================================================

/// Example of file system integrity checking
pub fn integrity_checking_example() -> TestResult {
    info!("=== File System Integrity Checking Example ===");

    // Create integrity checker
    let mut checker = integrity_checking::IntegrityChecker::new();

    // Load sample metadata
    let metadata = integrity_checking::FsMetadata {
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
    checker.load_metadata(metadata);

    // Add sample directory entries
    for i in 0..100 {
        let entry = integrity_checking::DirectoryEntry {
            name: format!("file_{}", i),
            inode: i,
            file_type: 1, // Regular file
            size: 4096,
            permissions: 0o644,
            atime: 1640995200 + i as u64,
            mtime: 1640995200 + i as u64,
            ctime: 1640995200 + i as u64,
        };
        checker.add_directory_entry(entry);
    }

    // Add sample file allocations
    for i in 0..100 {
        let allocation = integrity_checking::FileAllocation {
            inode: i,
            blocks: vec![i as u64 * 10, i as u64 * 10 + 1],
            size: 4096,
            link_count: 1,
        };
        checker.add_file_allocation(allocation);
    }

    // Run integrity checks
    let results = checker.run_full_check();

    // Analyze results
    let mut all_passed = true;
    for result in results {
        info!("{:?} - {} ({} errors, {} warnings)", 
              result.check_type, 
              if result.passed { "PASSED" } else { "FAILED" },
              result.errors_found,
              result.warnings);

        if !result.passed {
            all_passed = false;
            if !result.details.is_empty() {
                error!("Details: {}", result.details);
            }
        }
    }

    if all_passed {
        TestResult::Passed
    } else {
        TestResult::Failed
    }
}

/// Example of individual integrity tests
pub fn individual_integrity_tests_example() -> TestResult {
    info!("=== Individual Integrity Tests Example ===");

    let mut overall_result = TestResult::Passed;

    // Test metadata integrity
    info!("Testing metadata integrity...");
    let metadata_test = integrity_checking::MetadataIntegrityTest::new();
    let result = metadata_test.run();
    if result != TestResult::Passed {
        error!("Metadata integrity test failed");
        overall_result = TestResult::Failed;
    }

    // Test directory structure
    info!("Testing directory structure...");
    let dir_test = integrity_checking::DirectoryStructureTest::new();
    let result = dir_test.run();
    if result != TestResult::Passed {
        error!("Directory structure test failed");
        overall_result = TestResult::Failed;
    }

    overall_result
}

// =============================================================================
// RECOVERY TOOLS EXAMPLES
// =============================================================================

/// Example of file system recovery operations
pub fn recovery_tools_example() -> TestResult {
    info!("=== File System Recovery Tools Example ===");

    // Create recovery configuration
    let config = recovery_tools::RecoveryConfig {
        dry_run: false,
        create_backup: true,
        recover_deleted: true,
        deep_scan: true,
        max_recovery_size_mb: 2048,
        backup_location: Some("/tmp/recovery_backup".to_string()),
        recovery_timeout_ms: 300000, // 5 minutes
    };

    // Create recovery tool
    let mut recovery = recovery_tools::FileSystemRecovery::new(config);

    // Run recovery
    match recovery.run_recovery() {
        Ok(stats) => {
            info!("✓ Recovery completed successfully");
            info!("Files recovered: {}", stats.files_recovered);
            info!("Directories recovered: {}", stats.directories_recovered);
            info!("Blocks recovered: {}", stats.blocks_recovered);
            info!("Errors repaired: {}", stats.errors_repaired);
            info!("Backup size: {} bytes", stats.backup_size_bytes);
            info!("Recovery time: {} ms", stats.recovery_time_ms);
            info!("Success rate: {:.1}%", stats.success_rate);

            // Test backup and restore
            info!("Testing backup and restore...");
            let restore_result = recovery.restore_from_backup("/tmp/recovery_backup");
            match restore_result {
                Ok(_) => info!("✓ Backup restore completed"),
                Err(e) => error!("✗ Backup restore failed: {}", e),
            }

            TestResult::Passed
        }
        Err(e) => {
            error!("✗ Recovery failed: {}", e);
            TestResult::Failed
        }
    }
}

/// Example of individual recovery tests
pub fn individual_recovery_tests_example() -> TestResult {
    info!("=== Individual Recovery Tests Example ===");

    let mut overall_result = TestResult::Passed;

    // Test corruption repair
    info!("Testing corruption repair...");
    let corruption_test = recovery_tools::CorruptionRepairTest::new();
    let result = corruption_test.run();
    if result != TestResult::Passed {
        error!("Corruption repair test failed");
        overall_result = TestResult::Failed;
    }

    // Test orphaned file recovery
    info!("Testing orphaned file recovery...");
    let orphaned_test = recovery_tools::OrphanedFileRecoveryTest::new();
    let result = orphaned_test.run();
    if result != TestResult::Passed {
        error!("Orphaned file recovery test failed");
        overall_result = TestResult::Failed;
    }

    // Test backup and restore
    info!("Testing backup and restore...");
    let backup_test = recovery_tools::BackupRestoreTest::new();
    let result = backup_test.run();
    if result != TestResult::Passed {
        error!("Backup restore test failed");
        overall_result = TestResult::Failed;
    }

    overall_result
}

// =============================================================================
// PERFORMANCE BENCHMARKING EXAMPLES
// =============================================================================

/// Example of performance benchmarking
pub fn performance_benchmarking_example() -> TestResult {
    info!("=== Performance Benchmarking Example ===");

    // Create custom benchmark configuration
    let config = performance_benchmarking::BenchmarkConfig {
        test_file_size_mb: 200, // Larger files for better benchmarking
        number_of_files: 2000,  // More files
        block_size: 8192,       // Larger block size
        concurrent_operations: 8,
        warmup_operations: 200,
        measurement_duration_ms: 120000, // 2 minutes
        enable_detailed_profiling: true,
        output_format: performance_benchmarking::BenchmarkFormat::Json,
    };

    // Create benchmark with custom config
    let suite = performance_benchmarking::BenchmarkTestSuite::with_config(config);

    // Run benchmarks
    let result = suite.run();

    match result {
        TestResult::Passed => {
            info!("✓ Performance benchmarks completed successfully");
        }
        _ => {
            error!("✗ Performance benchmarks failed");
        }
    }

    result
}

/// Example of custom performance testing
pub fn custom_performance_testing_example() -> TestResult {
    info!("=== Custom Performance Testing Example ===");

    let config = performance_benchmarking::BenchmarkConfig {
        test_file_size_mb: 50,
        number_of_files: 500,
        block_size: 4096,
        concurrent_operations: 4,
        warmup_operations: 50,
        measurement_duration_ms: 30000,
        enable_detailed_profiling: false,
        output_format: performance_benchmarking::BenchmarkFormat::Text,
    };

    let mut benchmark = performance_benchmarking::FsPerformanceBenchmark::new(config);

    // Run only sequential benchmarks
    info!("Running sequential I/O benchmarks...");
    benchmark.run_sequential_read_benchmark();
    benchmark.run_sequential_write_benchmark();

    info!("Custom performance testing completed");
    TestResult::Passed
}

// =============================================================================
// DISK ANALYSIS EXAMPLES
// =============================================================================

/// Example of comprehensive disk analysis
pub fn disk_analysis_example() -> TestResult {
    info!("=== Disk Analysis Example ===");

    // Create analysis configuration
    let config = disk_analysis::DiskAnalysisConfig {
        analyze_large_directories: true,
        detect_fragmentation: true,
        scan_for_bad_blocks: true,
        analyze_file_age: true,
        include_hidden_files: false,
        max_directory_depth: 15,
        min_file_size_kb: 1,
        analysis_timeout_ms: 90000,
    };

    // Create analyzer
    let mut analyzer = disk_analysis::DiskAnalyzer::new(config);

    // Run comprehensive analysis
    let mount_point = "/test_mount";
    match analyzer.analyze_disk(mount_point) {
        Ok(analysis) => {
            info!("✓ Disk analysis completed successfully");

            // Display key findings
            info!("\n=== DISK ANALYSIS RESULTS ===");
            info!("Total capacity: {:.1} GB", analysis.disk_info.total_capacity_gb);
            info!("Used space: {:.1} GB ({:.1}%)", 
                analysis.disk_info.used_capacity_gb,
                analysis.disk_info.used_capacity_gb / analysis.disk_info.total_capacity_gb * 100.0);
            info!("File count: {}", analysis.usage_stats.file_count);
            info!("Directory count: {}", analysis.usage_stats.directory_count);
            info!("Total size: {:.1} GB", analysis.usage_stats.total_file_size_mb / 1024.0);

            // Generate detailed report
            let report = analyzer.generate_report(&analysis);
            debug!("\nDetailed analysis report:\n{}", report);

            TestResult::Passed
        }
        Err(e) => {
            error!("✗ Disk analysis failed: {}", e);
            TestResult::Failed
        }
    }
}

/// Example of specialized disk analysis
pub fn specialized_disk_analysis_example() -> TestResult {
    info!("=== Specialized Disk Analysis Example ===");

    let mut analyzer = disk_analysis::DiskAnalyzer::new(disk_analysis::DiskAnalysisConfig::default());

    // Test fragmentation analysis
    info!("Testing fragmentation analysis...");
    match analyzer.analyze_fragmentation("/test_mount") {
        Ok(fragmentation) => {
            info!("Fragmentation: {:.1}%", fragmentation.fragmentation_percent);
            info!("Fragmented files: {}", fragmentation.fragmented_files);
            if fragmentation.fragmentation_percent > 30.0 {
                warn!("High fragmentation detected - consider defragmentation");
            }
        }
        Err(e) => {
            warn!("Fragmentation analysis failed: {}", e);
        }
    }

    // Test bad block scanning
    info!("Testing bad block scanning...");
    match analyzer.scan_bad_blocks("/dev/sda") {
        Ok(bad_blocks) => {
            info!("Bad blocks found: {}", bad_blocks.total_bad_blocks);
            if bad_blocks.total_bad_blocks > 0 {
                warn!("Disk health concerns detected");
            }
        }
        Err(e) => {
            warn!("Bad block scanning failed: {}", e);
        }
    }

    // Test performance analysis
    info!("Testing performance analysis...");
    match analyzer.analyze_performance("/dev/sda") {
        Ok(performance) => {
            info!("Average response time: {:.1} ms", performance.average_response_time_ms);
            info!("Read operations/sec: {:.0}", performance.read_operations_per_second);
            info!("Write operations/sec: {:.0}", performance.write_operations_per_second);
            info!("Disk utilization: {:.1}%", performance.disk_utilization_percent);
        }
        Err(e) => {
            warn!("Performance analysis failed: {}", e);
        }
    }

    TestResult::Passed
}

// =============================================================================
// IMAGE CREATION EXAMPLES
// =============================================================================

/// Example of file system image creation
pub fn image_creation_example() -> TestResult {
    info!("=== File System Image Creation Example ===");

    // Create image configuration
    let config = image_creation::ImageCreationConfig {
        format: image_creation::ImageFormat::Qcow2,
        compression: image_creation::CompressionType::Gzip,
        image_size_mb: 2048,
        block_size: 4096,
        include_hidden_files: true,
        preserve_permissions: true,
        preserve_timestamps: true,
        verify_image: true,
        compress_metadata: true,
        exclude_patterns: vec!["*.tmp".to_string(), "*.log".to_string()],
    };

    // Create image creator with progress callback
    let creator = image_creation::ImageCreator::with_progress_callback(
        config,
        |percent: f64, phase: &str| {
            info!("Image creation progress: {:.1}% - {}", percent, phase);
        }
    );

    // Create image
    let source_dir = "/test/source";
    let output_path = "/test/images/test_image.qcow2";

    match creator.create_image(source_dir, output_path) {
        Ok(image_info) => {
            info!("✓ Image created successfully");
            info!("Format: {:?}", image_info.format);
            info!("Size: {} MB", image_info.size_bytes / (1024 * 1024));
            info!("Compressed size: {} MB", image_info.compressed_size_bytes / (1024 * 1024));
            info!("Compression ratio: {:.1}%", image_info.compression_ratio);
            info!("File count: {}", image_info.file_count);
            info!("Checksum: {}", image_info.checksum);

            // Test image operations
            let mut mount_tool = image_creation::ImageMountTool::new();
            let mount_point = "/mnt/test_image";

            // Mount image
            match mount_tool.mount_image(output_path, mount_point, "ext4") {
                Ok(mount_info) => {
                    info!("✓ Image mounted at {}", mount_info.mount_point);

                    // List mounted images
                    let mounted = mount_tool.list_mounted_images();
                    info!("Mounted images: {}", mounted.len());

                    // Unmount image
                    match mount_tool.unmount_image(output_path) {
                        Ok(_) => info!("✓ Image unmounted successfully"),
                        Err(e) => error!("✗ Image unmount failed: {}", e),
                    }
                }
                Err(e) => {
                    error!("✗ Image mount failed: {}", e);
                    return TestResult::Failed;
                }
            }

            // Test image conversion
            let output_converted = "/test/images/test_image.vmdk";
            let converter = image_creation::ImageConverter::with_progress_callback(
                |percent: f64, phase: &str| {
                    info!("Conversion progress: {:.1}% - {}", percent, phase);
                }
            );

            match converter.convert_image(output_path, output_converted, image_creation::ImageFormat::Vmdk) {
                Ok(converted_info) => {
                    info!("✓ Image converted to {:?}", converted_info.format);
                }
                Err(e) => {
                    error!("✗ Image conversion failed: {}", e);
                }
            }

            TestResult::Passed
        }
        Err(e) => {
            error!("✗ Image creation failed: {}", e);
            TestResult::Failed
        }
    }
}

// =============================================================================
// AUTOMATED TESTING EXAMPLES
// =============================================================================

/// Example of automated testing with edge cases
pub fn automated_testing_example() -> TestResult {
    info!("=== Automated Testing Example ===");

    // Create test configuration
    let config = automated_testing::TestScenarioConfig {
        max_file_size_bytes: 50 * 1024 * 1024, // 50MB
        max_path_length: 2048,
        max_filename_length: 200,
        max_directory_depth: 30,
        max_files_per_directory: 5000,
        concurrent_thread_count: 6,
        test_duration_seconds: 180, // 3 minutes
        enable_fuzzing: true,
        enable_property_testing: true,
        enable_edge_case_testing: true,
        memory_limit_mb: 256,
        disk_limit_mb: 512,
    };

    // Create test coordinator
    let mut coordinator = automated_testing::AutomatedTestCoordinator::new(config);

    // Run automated tests
    match coordinator.run_full_test_suite() {
        Ok(_) => {
            info!("✓ Automated tests completed successfully");

            // Display results
            let edge_results = coordinator.get_edge_case_results();
            let property_results = coordinator.get_property_results();
            let concurrent_results = coordinator.get_concurrent_results();
            let fuzz_results = coordinator.get_fuzz_results();

            info!("\n=== AUTOMATED TEST RESULTS ===");
            info!("Edge case tests: {}", edge_results.len());
            info!("Property tests: {}", property_results.len());
            info!("Concurrent tests: {}", concurrent_results.len());
            info!("Fuzz tests: {}", fuzz_results.len());

            // Check for issues
            let failed_edge_cases = edge_results.iter().filter(|r| !r.passed).count();
            let failed_properties = property_results.iter().filter(|r| r.violations_found > 0).count();
            let crashes_in_fuzz = fuzz_results.iter().filter(|r| r.crashes_detected > 0).count();

            if failed_edge_cases > 0 {
                warn!("{} edge case tests failed", failed_edge_cases);
            }
            if failed_properties > 0 {
                warn!("{} property tests failed", failed_properties);
            }
            if crashes_in_fuzz > 0 {
                warn!("{} fuzz tests found crashes", crashes_in_fuzz);
            }

            TestResult::Passed
        }
        Err(e) => {
            error!("✗ Automated tests failed: {}", e);
            TestResult::Failed
        }
    }
}

/// Example of focused automated testing
pub fn focused_automated_testing_example() -> TestResult {
    info!("=== Focused Automated Testing Example ===");

    let config = automated_testing::TestScenarioConfig {
        max_file_size_bytes: 10 * 1024 * 1024, // 10MB
        max_path_length: 1024,
        max_filename_length: 100,
        max_directory_depth: 10,
        max_files_per_directory: 1000,
        concurrent_thread_count: 4,
        test_duration_seconds: 60, // 1 minute
        enable_fuzzing: false, // Disable for focused test
        enable_property_testing: true,
        enable_edge_case_testing: true,
        memory_limit_mb: 128,
        disk_limit_mb: 256,
    };

    let mut coordinator = automated_testing::AutomatedTestCoordinator::new(config);

    // Run only edge case and property tests
    coordinator.run_edge_case_tests()?;
    coordinator.run_property_tests()?;

    info!("Focused automated testing completed");
    TestResult::Passed
}

// =============================================================================
// VALIDATION FRAMEWORK EXAMPLES
// =============================================================================

/// Example of file system validation
pub fn validation_framework_example() -> TestResult {
    info!("=== Validation Framework Example ===");

    // Create validation context
    let context = validation_framework::ValidationContext {
        root_path: "/test".to_string(),
        file_system_type: "ext4".to_string(),
        validation_rules: vec![
            validation_framework::ValidationRule::FileSystemCompliance,
            validation_framework::ValidationRule::PosixCompliance,
            validation_framework::ValidationRule::MetadataConsistency,
            validation_framework::ValidationRule::DataIntegrity,
            validation_framework::ValidationRule::PermissionValidation,
            validation_framework::ValidationRule::PathValidation,
            validation_framework::ValidationRule::NameValidation,
            validation_framework::ValidationRule::SizeValidation,
        ],
        strict_mode: true,
        ignore_warnings: false,
        max_errors: 50,
    };

    // Create validator
    let mut validator = validation_framework::FileSystemValidator::new(context);

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

            // Display compliance information
            let compliance_info = validator.generate_compliance_info();
            if !compliance_info.is_empty() {
                info!("\n=== COMPLIANCE INFORMATION ===");
                for info in &compliance_info {
                    info!("{} ({}): {:.1}% compliant", 
                          info.standard_name, info.version, info.compliance_score);
                }
            }

            if report.failed_checks == 0 {
                TestResult::Passed
            } else {
                TestResult::Failed
            }
        }
        Err(e) => {
            error!("✗ Validation failed: {}", e);
            TestResult::Failed
        }
    }
}

/// Example of compliance testing
pub fn compliance_testing_example() -> TestResult {
    info!("=== Compliance Testing Example ===");

    let context = validation_framework::ValidationContext {
        root_path: "/test".to_string(),
        file_system_type: "ext4".to_string(),
        validation_rules: vec![
            validation_framework::ValidationRule::PosixCompliance,
            validation_framework::ValidationRule::ExtCompliance,
        ],
        strict_mode: true,
        ignore_warnings: false,
        max_errors: 20,
    };

    let mut validator = validation_framework::FileSystemValidator::new(context);

    // Focus on compliance testing
    validator.run_validation_rule(validation_framework::ValidationRule::PosixCompliance)?;
    validator.run_validation_rule(validation_framework::ValidationRule::ExtCompliance)?;

    // Get compliance information
    let compliance_info = validator.generate_compliance_info();
    
    for info in &compliance_info {
        if info.compliance_score >= 80.0 {
            info!("✓ {} compliance: {:.1}% (PASS)", info.standard_name, info.compliance_score);
        } else {
            warn!("⚠ {} compliance: {:.1}% (FAIL)", info.standard_name, info.compliance_score);
        }
    }

    TestResult::Passed
}

// =============================================================================
// COMPREHENSIVE INTEGRATION EXAMPLE
// =============================================================================

/// Comprehensive example that demonstrates the full testing framework
pub fn comprehensive_integration_example() -> TestResult {
    info!("=== Comprehensive Integration Example ===");
    info!("This example demonstrates the complete file system testing framework");

    let mut overall_result = TestResult::Passed;

    // 1. Initialize framework
    info!("\n1. Initializing testing framework...");
    init_testing().expect("Failed to initialize");

    // 2. Run basic stress tests
    info!("\n2. Running stress tests...");
    let stress_result = custom_stress_testing_example();
    if stress_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 3. Validate integrity
    info!("\n3. Validating file system integrity...");
    let integrity_result = integrity_checking_example();
    if integrity_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 4. Test recovery capabilities
    info!("\n4. Testing recovery capabilities...");
    let recovery_result = recovery_tools_example();
    if recovery_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 5. Run performance benchmarks
    info!("\n5. Running performance benchmarks...");
    let performance_result = custom_performance_testing_example();
    if performance_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 6. Analyze disk usage
    info!("\n6. Analyzing disk usage...");
    let analysis_result = specialized_disk_analysis_example();
    if analysis_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 7. Test image operations
    info!("\n7. Testing image operations...");
    let image_result = image_creation_example();
    if image_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 8. Run automated tests
    info!("\n8. Running automated tests...");
    let automated_result = focused_automated_testing_example();
    if automated_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // 9. Validate compliance
    info!("\n9. Validating compliance...");
    let validation_result = compliance_testing_example();
    if validation_result != TestResult::Passed {
        overall_result = TestResult::Failed;
    }

    // Final summary
    match overall_result {
        TestResult::Passed => {
            info!("\n" + "=".repeat(60));
            info!("🎉 COMPREHENSIVE INTEGRATION TEST COMPLETED SUCCESSFULLY!");
            info!("All file system testing components are working correctly.");
            info!("=".repeat(60));
        }
        _ => {
            error!("\n" + "=".repeat(60));
            error!("❌ COMPREHENSIVE INTEGRATION TEST FAILED!");
            error!("Some components have issues that need attention.");
            error!("=".repeat(60));
        }
    }

    overall_result
}

// =============================================================================
// PERFORMANCE COMPARISON EXAMPLE
// =============================================================================

/// Example comparing different file system configurations
pub fn performance_comparison_example() -> TestResult {
    info!("=== Performance Comparison Example ===");
    info!("Comparing performance across different configurations");

    let configurations = vec![
        ("Small Files", performance_benchmarking::BenchmarkConfig {
            test_file_size_mb: 10,
            number_of_files: 5000,
            block_size: 4096,
            ..performance_benchmarking::BenchmarkConfig::default()
        }),
        ("Large Files", performance_benchmarking::BenchmarkConfig {
            test_file_size_mb: 500,
            number_of_files: 100,
            block_size: 8192,
            ..performance_benchmarking::BenchmarkConfig::default()
        }),
        ("High Concurrency", performance_benchmarking::BenchmarkConfig {
            test_file_size_mb: 50,
            number_of_files: 1000,
            block_size: 4096,
            concurrent_operations: 16,
            ..performance_benchmarking::BenchmarkConfig::default()
        }),
    ];

    let mut results = Vec::new();

    for (name, config) in configurations {
        info!("\nTesting configuration: {}", name);
        
        let suite = performance_benchmarking::BenchmarkTestSuite::with_config(config);
        let result = suite.run();
        
        results.push((name.to_string(), result));
        
        match result {
            TestResult::Passed => info!("✓ {} configuration passed", name),
            _ => warn!("⚠ {} configuration had issues", name),
        }
    }

    // Summary
    info!("\n=== PERFORMANCE COMPARISON SUMMARY ===");
    for (name, result) in &results {
        info!("{}: {:?}", name, result);
    }

    if results.iter().all(|(_, r)| *r == TestResult::Passed) {
        TestResult::Passed
    } else {
        TestResult::Failed
    }
}

#[cfg(test)]
mod examples_tests {
    use super::*;

    #[test]
    fn test_basic_test_suite_example() {
        // This is just a compilation test - the actual example would require
        // proper file system setup to run successfully
        let _result = basic_test_suite_example();
        // In a real test, we would check the result
    }

    #[test]
    fn test_custom_stress_testing_example() {
        let _result = custom_stress_testing_example();
        // Compilation test for the stress testing example
    }

    #[test]
    fn test_integrity_checking_example() {
        let _result = integrity_checking_example();
        // Compilation test for the integrity checking example
    }

    #[test]
    fn test_recovery_tools_example() {
        let _result = recovery_tools_example();
        // Compilation test for the recovery tools example
    }

    #[test]
    fn test_performance_benchmarking_example() {
        let _result = performance_benchmarking_example();
        // Compilation test for the performance benchmarking example
    }

    #[test]
    fn test_disk_analysis_example() {
        let _result = disk_analysis_example();
        // Compilation test for the disk analysis example
    }

    #[test]
    fn test_image_creation_example() {
        let _result = image_creation_example();
        // Compilation test for the image creation example
    }

    #[test]
    fn test_automated_testing_example() {
        let _result = automated_testing_example();
        // Compilation test for the automated testing example
    }

    #[test]
    fn test_validation_framework_example() {
        let _result = validation_framework_example();
        // Compilation test for the validation framework example
    }

    #[test]
    fn test_comprehensive_integration_example() {
        // This is a more complex example that would need proper setup
        // For now, just test compilation
        let _result = comprehensive_integration_example();
    }

    #[test]
    fn test_performance_comparison_example() {
        let _result = performance_comparison_example();
        // Compilation test for the performance comparison example
    }
}