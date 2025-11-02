//! Visual regression testing example demonstrating automated UI change detection
//! 
//! This example shows:
//! - Setting up visual regression tests
//! - Managing baseline screenshots
//! - Running automated regression tests
//! - Generating change reports with visual diffs

use multios_ui_testing::*;
use tokio;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Visual Regression Testing Example");
    
    // Initialize the regression tester
    let mut regression = RegressionTester::new().await?;
    
    // Configure regression testing parameters
    let config = RegressionConfig::new()
        .with_baseline_dir("test_data/baselines")
        .with_screenshot_dir("test_data/screenshots")
        .with_threshold(0.95) // 95% similarity threshold
        .with_ignore_regions(vec![ // Ignore dynamic areas like timestamps
            Region::new(800, 10, 120, 30), // Status bar
            Region::new(750, 580, 100, 20), // Timestamp area
        ]);
    
    // Create regression test suite
    let test_suite = RegressionTestSuite::new("ui_components_regression")
        .with_test_case(RegressionTest::new("login_page")
            .with_capture_area("login_window")
            .with_baseline_path("baseline/login_page.png")
            .with_ignore_dynamic_elements(true))
        .with_test_case(RegressionTest::new("main_dashboard")
            .with_capture_area("dashboard_window")
            .with_baseline_path("baseline/main_dashboard.png")
            .with_threshold(0.90)) // Lower threshold for dashboard
        .with_test_case(RegressionTest::new("settings_dialog")
            .with_capture_area("settings_window")
            .with_baseline_path("baseline/settings_dialog.png"));
    
    // Run the regression test suite
    let suite_result = regression.run_test_suite(test_suite).await?;
    
    println!("Regression test suite completed");
    println!("Total tests: {}", suite_result.total_tests);
    println!("Passed: {}", suite_result.passed);
    println!("Failed: {}", suite_result.failed);
    println!("Warnings: {}", suite_result.warnings);
    
    // Process failed tests and generate detailed reports
    for failed_test in &suite_result.failed_tests {
        println!("\nFailed test: {}", failed_test.test_name);
        println!("  Similarity: {:.2}%", failed_test.similarity * 100.0);
        println!("  Expected threshold: {:.2}%", failed_test.threshold * 100.0);
        
        // Generate visual diff for failed test
        if let Some(diff_path) = &failed_test.diff_path {
            println!("  Visual diff saved to: {}", diff_path);
        }
        
        // Save current screenshot for analysis
        if let Some(current_path) = &failed_test.current_screenshot_path {
            println!("  Current screenshot: {}", current_path);
        }
    }
    
    // Generate comprehensive regression report
    let report = regression.generate_report(&suite_result).await?;
    
    // Save the report
    std::fs::write("test_data/reports/regression_report.html", report)?;
    println!("\nRegression report saved to test_data/reports/regression_report.html");
    
    // Update baselines for passed tests (in development)
    if std::env::var("UPDATE_BASELINES").is_ok() {
        println!("\nUpdating baseline screenshots for passed tests...");
        regression.update_baselines(&suite_result).await?;
    }
    
    // Clean up
    regression.shutdown().await?;
    
    Ok(())
}