# MultiOS File System Testing Framework

A comprehensive file system testing, validation, and debugging framework for MultiOS. This framework provides extensive testing capabilities including stress testing, integrity checking, recovery tools, performance benchmarking, and automated testing for various file system types and edge cases.

## Features

### 🔧 Core Testing Components

- **Stress Testing**: Comprehensive stress testing including concurrent access, memory pressure, disk exhaustion, and operation storm scenarios
- **Integrity Checking**: File system metadata validation, data corruption detection, cross-reference validation, and consistency checking
- **Recovery Tools**: Corruption repair, orphaned file recovery, data salvage, backup creation, and restoration capabilities
- **Performance Benchmarking**: Sequential/random I/O testing, file operation benchmarks, directory operation testing, and concurrent I/O measurement
- **Disk Analysis**: Usage analysis, fragmentation detection, bad block scanning, and performance monitoring
- **Image Creation**: File system image creation, mounting/unmounting utilities, format conversion, and image verification
- **Automated Testing**: Edge case testing, property-based testing, concurrent operation testing, fuzz testing, and long-running stability tests
- **Validation Framework**: Compliance checking, standard conformance testing, and consistency verification

### 📊 Advanced Capabilities

- **Multi-format Support**: Testing for various file systems (EXT2/3/4, FAT32, tmpfs, etc.)
- **Configurable Testing**: Extensive configuration options for all test types
- **Progress Monitoring**: Real-time progress tracking with detailed reporting
- **Concurrent Testing**: Multi-threaded testing capabilities for realistic load simulation
- **Comprehensive Reporting**: Detailed test reports with metrics, statistics, and compliance scores
- **Memory-Safe**: Built with Rust's memory safety guarantees
- **No-STD Compatible**: Can be used in kernel and bare-metal environments

## Architecture

```
filesystem_testing/
├── src/
│   ├── lib.rs                 # Main library interface
│   ├── test_suite.rs          # Base test framework
│   ├── stress_testing.rs      # Stress testing tools
│   ├── integrity_checking.rs  # Integrity validation
│   ├── recovery_tools.rs      # Recovery and repair
│   ├── performance_benchmarking.rs # Performance testing
│   ├── disk_analysis.rs       # Disk analysis tools
│   ├── image_creation.rs      # Image management
│   ├── automated_testing.rs   # Automated test suite
│   ├── validation_framework.rs # Validation and compliance
│   └── examples.rs            # Usage examples
├── Cargo.toml                 # Dependencies
└── README.md                  # This file
```

## Quick Start

### Basic Usage

```rust
use filesystem_testing::*;

// Initialize the testing framework
init_testing()?;

// Run the complete test suite
let result = run_full_test_suite();

match result {
    TestResult::Passed => println!("All tests passed!"),
    _ => println!("Some tests failed or had issues"),
}
```

### Running Individual Test Suites

```rust
// Run specific test suites
let stress_result = run_test_suite("stress_testing");
let integrity_result = run_test_suite("integrity_checking");
let performance_result = run_test_suite("performance_benchmarking");
```

### Custom Configuration

```rust
use filesystem_testing::stress_testing::{StressTestConfig, StressTestSuite};

// Create custom stress test configuration
let config = StressTestConfig {
    max_concurrent_files: 2000,
    max_file_size: 10 * 1024 * 1024, // 10MB
    max_directory_depth: 15,
    concurrent_threads: 8,
    ..StressTestConfig::default()
};

// Create test suite with custom config
let suite = StressTestSuite::with_config(config);
let result = suite.run();
```

## Detailed Component Documentation

### Stress Testing

The stress testing module provides comprehensive file system stress testing:

```rust
use filesystem_testing::stress_testing::*;

// Create stress test runner
let config = StressTestConfig::default();
let runner = StressTestRunner::new(config);

// Run concurrent file creation test
let result = runner.test_concurrent_file_creation();

// Run memory pressure test
let result = runner.test_memory_pressure();

// Run disk space exhaustion test
let result = runner.test_disk_space_exhaustion();

// Run operation storm test
let result = runner.test_operation_storm();
```

**Features:**
- Concurrent file creation with configurable thread counts
- Memory pressure simulation with configurable limits
- Disk space exhaustion testing
- High-frequency operation storm testing
- Progress monitoring and detailed reporting

### Integrity Checking

File system integrity validation and corruption detection:

```rust
use filesystem_testing::integrity_checking::*;

// Create integrity checker
let mut checker = IntegrityChecker::new();

// Load file system metadata
let metadata = FsMetadata {
    total_blocks: 1000000,
    used_blocks: 750000,
    free_blocks: 250000,
    block_size: 4096,
    // ... other fields
};
checker.load_metadata(metadata);

// Run integrity checks
let results = checker.run_full_check();

// Analyze results
for result in results {
    println!("{:?} - {} ({} errors, {} warnings)", 
             result.check_type, 
             if result.passed { "PASSED" } else { "FAILED" },
             result.errors_found,
             result.warnings);
}
```

**Features:**
- Metadata integrity validation
- Data block allocation checking
- Directory structure validation
- Cross-reference consistency verification
- Allocation table validation
- Permission and security checking

### Recovery Tools

File system recovery and repair capabilities:

```rust
use filesystem_testing::recovery_tools::*;

// Create recovery configuration
let config = RecoveryConfig {
    dry_run: false,
    create_backup: true,
    recover_deleted: true,
    deep_scan: true,
    ..RecoveryConfig::default()
};

// Create recovery tool
let mut recovery = FileSystemRecovery::new(config);

// Run recovery operation
match recovery.run_recovery() {
    Ok(stats) => {
        println!("Recovery completed:");
        println!("Files recovered: {}", stats.files_recovered);
        println!("Errors repaired: {}", stats.errors_repaired);
        println!("Success rate: {:.1}%", stats.success_rate);
    }
    Err(e) => {
        println!("Recovery failed: {}", e);
    }
}
```

**Features:**
- Corruption detection and repair
- Orphaned file recovery
- Data salvage from damaged file systems
- Backup creation and restoration
- Deleted file recovery attempts

### Performance Benchmarking

Comprehensive file system performance testing:

```rust
use filesystem_testing::performance_benchmarking::*;

// Create benchmark configuration
let config = BenchmarkConfig {
    test_file_size_mb: 100,
    number_of_files: 1000,
    block_size: 4096,
    concurrent_operations: 4,
    measurement_duration_ms: 60000,
    ..BenchmarkConfig::default()
};

// Create benchmark suite
let suite = BenchmarkTestSuite::with_config(config);
let results = suite.run();
```

**Features:**
- Sequential read/write performance testing
- Random read/write performance testing
- File creation/deletion benchmarks
- Directory operation testing
- Metadata operation performance
- Concurrent I/O benchmarking
- Memory usage profiling

### Disk Analysis

Disk usage analysis and health monitoring:

```rust
use filesystem_testing::disk_analysis::*;

// Create analysis configuration
let config = DiskAnalysisConfig {
    analyze_large_directories: true,
    detect_fragmentation: true,
    scan_for_bad_blocks: true,
    analyze_file_age: true,
    ..DiskAnalysisConfig::default()
};

// Create analyzer
let mut analyzer = DiskAnalyzer::new(config);

// Run comprehensive analysis
match analyzer.analyze_disk("/test_mount") {
    Ok(analysis) => {
        println!("Total capacity: {:.1} GB", analysis.disk_info.total_capacity_gb);
        println!("Used space: {:.1} GB", analysis.disk_info.used_capacity_gb);
        println!("File count: {}", analysis.usage_stats.file_count);
        
        // Generate detailed report
        let report = analyzer.generate_report(&analysis);
        println!("{}", report);
    }
    Err(e) => {
        println!("Analysis failed: {}", e);
    }
}
```

**Features:**
- Disk usage statistics and reporting
- Directory analysis with size breakdowns
- File type and size distribution analysis
- Fragmentation detection and measurement
- Bad block detection and mapping
- Performance metrics collection

### Image Creation

File system image creation and management:

```rust
use filesystem_testing::image_creation::*;

// Create image configuration
let config = ImageCreationConfig {
    format: ImageFormat::Qcow2,
    compression: CompressionType::Gzip,
    image_size_mb: 2048,
    preserve_permissions: true,
    verify_image: true,
    ..ImageCreationConfig::default()
};

// Create image creator
let creator = ImageCreator::new(config);

// Create image from directory
match creator.create_image("/source", "/output/image.qcow2") {
    Ok(image_info) => {
        println!("Image created: {:?}", image_info.format);
        println!("Size: {} MB", image_info.size_bytes / (1024 * 1024));
        println!("Compression ratio: {:.1}%", image_info.compression_ratio);
    }
    Err(e) => {
        println!("Image creation failed: {}", e);
    }
}
```

**Features:**
- Image creation from directories
- Multiple format support (Raw, Qcow2, VMDK, VHDX, ISO)
- Compression support (Gzip, Bzip2, Xz, Zstd)
- Image mounting and unmounting utilities
- Format conversion capabilities
- Image verification and validation

### Automated Testing

Automated test suite with edge case and fuzz testing:

```rust
use filesystem_testing::automated_testing::*;

// Create test configuration
let config = TestScenarioConfig {
    max_file_size_bytes: 50 * 1024 * 1024,
    concurrent_thread_count: 8,
    test_duration_seconds: 300,
    enable_fuzzing: true,
    enable_property_testing: true,
    enable_edge_case_testing: true,
    ..TestScenarioConfig::default()
};

// Create test coordinator
let mut coordinator = AutomatedTestCoordinator::new(config);

// Run automated tests
match coordinator.run_full_test_suite() {
    Ok(_) => {
        println!("Automated tests completed successfully");
        
        // Display results
        let edge_results = coordinator.get_edge_case_results();
        let property_results = coordinator.get_property_results();
        let fuzz_results = coordinator.get_fuzz_results();
        
        println!("Edge case tests: {}", edge_results.len());
        println!("Property tests: {}", property_results.len());
        println!("Fuzz tests: {}", fuzz_results.len());
    }
    Err(e) => {
        println!("Automated tests failed: {}", e);
    }
}
```

**Features:**
- Edge case testing with maximum limits
- Property-based testing with automated verification
- Concurrent operation testing with race condition detection
- Fuzz testing for robustness
- Long-running stability tests
- Comprehensive test result reporting

### Validation Framework

File system validation and compliance checking:

```rust
use filesystem_testing::validation_framework::*;

// Create validation context
let context = ValidationContext {
    root_path: "/test".to_string(),
    file_system_type: "ext4".to_string(),
    validation_rules: vec![
        ValidationRule::FileSystemCompliance,
        ValidationRule::PosixCompliance,
        ValidationRule::MetadataConsistency,
        ValidationRule::DataIntegrity,
    ],
    strict_mode: true,
    ..ValidationContext::default()
};

// Create validator
let mut validator = FileSystemValidator::new(context);

// Run validation
match validator.validate() {
    Ok(report) => {
        println!("Validation completed:");
        println!("Total checks: {}", report.total_checks);
        println!("Passed checks: {}", report.passed_checks);
        println!("Compliance score: {:.1}%", report.compliance_score);
        
        // Get compliance information
        let compliance_info = validator.generate_compliance_info();
        for info in &compliance_info {
            println!("{} compliance: {:.1}%", info.standard_name, info.compliance_score);
        }
    }
    Err(e) => {
        println!("Validation failed: {}", e);
    }
}
```

**Features:**
- File system compliance validation
- POSIX compliance checking
- Metadata consistency verification
- Data integrity validation
- Permission and security checking
- Standard conformance testing

## Configuration Options

All components support extensive configuration options:

### Stress Testing Configuration

```rust
let config = StressTestConfig {
    max_concurrent_files: 1000,
    max_file_size: 1024 * 1024, // 1MB
    max_directory_depth: 10,
    max_files_per_operation: 100,
    operation_timeout_ms: 30000,
    memory_pressure_mb: 64,
    disk_usage_target: 0.8, // 80%
    concurrent_threads: 4,
};
```

### Benchmark Configuration

```rust
let config = BenchmarkConfig {
    test_file_size_mb: 100,
    number_of_files: 1000,
    block_size: 4096,
    concurrent_operations: 4,
    warmup_operations: 100,
    measurement_duration_ms: 60000,
    enable_detailed_profiling: false,
    output_format: BenchmarkFormat::Text,
};
```

### Recovery Configuration

```rust
let config = RecoveryConfig {
    dry_run: false,
    create_backup: true,
    recover_deleted: true,
    deep_scan: false,
    max_recovery_size_mb: 1024,
    backup_location: None,
    recovery_timeout_ms: 300000,
};
```

## Examples and Demonstrations

The framework includes comprehensive examples demonstrating various use cases:

```rust
use filesystem_testing::examples::*;

// Run basic test suite
let result = basic_test_suite_example();

// Run individual component examples
let result = custom_stress_testing_example();
let result = integrity_checking_example();
let result = recovery_tools_example();
let result = performance_benchmarking_example();
let result = disk_analysis_example();
let result = image_creation_example();
let result = automated_testing_example();
let result = validation_framework_example();

// Run comprehensive integration test
let result = comprehensive_integration_example();

// Run performance comparison across configurations
let result = performance_comparison_example();
```

## Testing Best Practices

### 1. Start with Basic Tests

Begin with fundamental operations before running complex tests:

```rust
// Initialize framework
init_testing()?;

// Run basic tests first
let basic_result = basic_test_suite_example();
if basic_result != TestResult::Passed {
    return Err("Basic tests failed");
}

// Then run more complex tests
let advanced_result = comprehensive_integration_example();
```

### 2. Use Appropriate Configurations

Configure tests based on your system capabilities:

```rust
// For high-performance systems
let high_perf_config = StressTestConfig {
    max_concurrent_files: 5000,
    concurrent_threads: 16,
    ..StressTestConfig::default()
};

// For resource-constrained systems
let low_resource_config = StressTestConfig {
    max_concurrent_files: 500,
    concurrent_threads: 2,
    memory_pressure_mb: 32,
    ..StressTestConfig::default()
};
```

### 3. Monitor Progress and Results

Use progress callbacks and detailed reporting:

```rust
let creator = ImageCreator::with_progress_callback(config, |percent, phase| {
    println!("Progress: {:.1}% - {}", percent, phase);
});

// Generate detailed reports
let report = analyzer.generate_report(&analysis);
println!("{}", report);
```

### 4. Validate Results

Always validate test results and check for issues:

```rust
let results = checker.run_full_check();

let mut issues_found = 0;
for result in results {
    if !result.passed {
        issues_found += 1;
        match result.severity {
            ValidationSeverity::Error => error!("ERROR: {}", result.message),
            ValidationSeverity::Warning => warn!("WARNING: {}", result.message),
            ValidationSeverity::Info => info!("INFO: {}", result.message),
        }
    }
}

if issues_found > 0 {
    return Err(format!("Found {} issues during testing", issues_found));
}
```

## Integration with MultiOS

This testing framework is designed to integrate seamlessly with MultiOS:

### Kernel Integration

```rust
// In kernel initialization
pub fn init_filesystem_testing() -> Result<(), FsError> {
    // Initialize testing framework
    filesystem_testing::init_testing()
        .map_err(|_| FsError::IoError)?;
    
    // Register custom test suites
    register_kernel_specific_tests();
    
    Ok(())
}
```

### Driver Integration

```rust
// In file system driver
pub struct Ext4Driver {
    testing_enabled: bool,
    test_coordinator: Option<TestCoordinator>,
}

impl Ext4Driver {
    pub fn enable_testing(&mut self) -> Result<(), FsError> {
        self.testing_enabled = true;
        
        // Create driver-specific test suite
        let suite = Ext4DriverTestSuite::new();
        
        // Add to global coordinator
        let mut coordinator = TEST_COORDINATOR.lock();
        if let Some(coord) = coordinator.as_mut() {
            coord.register_suite("ext4_driver".to_string(), Box::new(suite));
        }
        
        Ok(())
    }
}
```

## Performance Considerations

### Memory Usage

- All testing components are designed to minimize memory footprint
- Configurable memory limits prevent system overload
- Large data structures use appropriate allocation strategies

### Execution Time

- Tests are designed to complete within reasonable timeframes
- Configurable timeouts prevent infinite loops
- Progress monitoring allows for interruptible operations

### System Impact

- Tests can be configured to run in dry-run mode
- Non-destructive testing options available
- Resource usage monitoring and limits

## Troubleshooting

### Common Issues

1. **Test Timeouts**
   - Increase timeout values for slower systems
   - Reduce test complexity for resource-constrained environments
   - Use dry-run mode for initial testing

2. **Memory Issues**
   - Lower concurrent operation counts
   - Reduce test file sizes
   - Enable memory pressure monitoring

3. **Permission Errors**
   - Ensure adequate file system permissions
   - Run tests as appropriate user
   - Check file system mount options

### Debug Mode

Enable detailed logging for troubleshooting:

```rust
use log::{info, warn, error, debug};

// Set log level
log::set_max_level(log::LevelFilter::Debug);

// Run tests with detailed output
let result = run_full_test_suite();
```

## Contributing

Contributions to the file system testing framework are welcome! Please:

1. Follow Rust coding standards
2. Add comprehensive tests for new features
3. Update documentation for any API changes
4. Ensure all tests pass before submitting
5. Consider performance implications of changes

### Development Setup

```bash
# Clone the repository
git clone <repository-url>
cd filesystem_testing

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example comprehensive_test

# Generate documentation
cargo doc --open
```

## License

This file system testing framework is part of the MultiOS project. See the main project LICENSE file for details.

## Support

For support, bug reports, and feature requests, please refer to the MultiOS project issue tracker.

---

**Note**: This framework is designed for testing and development purposes. Some tests may be destructive and should only be run on test systems or with proper backups.