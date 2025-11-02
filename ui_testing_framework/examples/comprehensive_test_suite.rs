//! Comprehensive test suite combining multiple testing approaches
//! 
//! This example demonstrates:
//! - Running a complete test suite with automation, accessibility, and performance tests
//! - Integrating multiple testing frameworks
//! - Generating unified test reports
//! - Continuous testing workflow

use multios_ui_testing::*;
use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Comprehensive UI Test Suite");
    
    // Initialize all testing components
    let mut automation = AutomationEngine::new().await?;
    let mut accessibility = AccessibilityTester::new().await?;
    let mut performance = PerformanceBenchmark::new().await?;
    let mut regression = RegressionTester::new().await?;
    
    // Create comprehensive test scenario
    let test_scenario = ComprehensiveTestSuite::new("multios_ui_comprehensive_test")
        .with_automation_test(AutomationTest::new("user_login_workflow")
            .with_steps(vec![
                WidgetInteraction::find_by_id("login_button")?,
                WidgetInteraction::click()?,
                WidgetInteraction::find_by_id("username_field")?,
                WidgetInteraction::type_text("test_user")?,
                WidgetInteraction::find_by_id("password_field")?,
                WidgetInteraction::type_text("password123")?,
                WidgetInteraction::find_by_id("submit_button")?,
                WidgetInteraction::click()?,
            ]))
        .with_accessibility_test(AccessibilityTest::new("login_page_accessibility")
            .with_standards(vec![AccessibilityStandard::WCAG2AA])
            .with_critical_checks_only(true))
        .with_performance_test(PerformanceTest::new("login_performance")
            .with_metrics(vec![PerformanceMetric::FrameRate, PerformanceMetric::MemoryUsage])
            .with_duration(Duration::from_secs(30)))
        .with_regression_test(RegressionTest::new("login_ui_regression")
            .with_baseline_path("baseline/login_page.png")
            .with_threshold(0.95));
    
    println!("Running comprehensive test suite...");
    let start_time = std::time::Instant::now();
    
    // Execute automation tests
    println!("\n1. Running automation tests...");
    let automation_result = test_scenario.run_automation_tests(&mut automation).await?;
    println!("   Automation tests: {} passed, {} failed", 
             automation_result.passed, automation_result.failed);
    
    // Execute accessibility tests
    println!("\n2. Running accessibility tests...");
    let accessibility_result = test_scenario.run_accessibility_tests(&mut accessibility).await?;
    println!("   Accessibility issues found: {}", accessibility_result.issues.len());
    
    // Execute performance tests
    println!("\n3. Running performance tests...");
    let performance_result = test_scenario.run_performance_tests(&mut performance).await?;
    println!("   Average FPS: {:.2}", performance_result.frame_rate);
    println!("   Memory usage: {:.2}MB", performance_result.memory_usage);
    
    // Execute regression tests
    println!("\n4. Running regression tests...");
    let regression_result = test_scenario.run_regression_tests(&mut regression).await?;
    println!("   Regression tests: {} passed, {} failed", 
             regression_result.passed, regression_result.failed);
    
    let total_time = start_time.elapsed();
    println!("\nTest suite completed in {:.2} seconds", total_time.as_secs_f64());
    
    // Generate unified test report
    println!("\nGenerating unified test report...");
    let unified_report = UnifiedTestReport::new()
        .with_automation_results(automation_result)
        .with_accessibility_results(accessibility_result)
        .with_performance_results(performance_result)
        .with_regression_results(regression_result)
        .with_execution_time(total_time)
        .build();
    
    // Save reports in multiple formats
    let html_report = unified_report.generate_html_report();
    let json_report = unified_report.generate_json_report();
    let xml_report = unified_report.generate_junit_xml_report();
    
    std::fs::write("test_data/reports/comprehensive_test_report.html", html_report)?;
    std::fs::write("test_data/reports/comprehensive_test_report.json", json_report)?;
    std::fs::write("test_data/reports/comprehensive_test_report.xml", xml_report)?;
    
    println!("Reports saved:");
    println!("  - HTML: test_data/reports/comprehensive_test_report.html");
    println!("  - JSON: test_data/reports/comprehensive_test_report.json");
    println!("  - JUnit XML: test_data/reports/comprehensive_test_report.xml");
    
    // Determine overall test status
    let overall_status = if automation_result.failed == 0 
        && accessibility_result.issues.is_empty()
        && regression_result.failed == 0
        && performance_result.frame_rate > 30.0 {
        TestStatus::Passed
    } else {
        TestStatus::Failed
    };
    
    println!("\nOverall test status: {:?}", overall_status);
    
    // Clean up all components
    automation.shutdown().await?;
    accessibility.shutdown().await?;
    performance.shutdown().await?;
    regression.shutdown().await?;
    
    // Exit with appropriate code
    match overall_status {
        TestStatus::Passed => {
            println!("All tests passed successfully!");
            Ok(())
        }
        TestStatus::Failed => {
            eprintln!("Some tests failed. Check reports for details.");
            std::process::exit(1);
        }
    }
}