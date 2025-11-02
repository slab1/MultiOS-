//! Accessibility testing example demonstrating WCAG compliance checks
//! 
//! This example shows:
//! - Running accessibility audits on UI components
//! - Checking color contrast ratios
//! - Validating ARIA labels and attributes
//! - Testing keyboard navigation
//! - Generating accessibility reports

use multios_ui_testing::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Accessibility Testing Example");
    
    // Initialize the accessibility tester
    let mut accessibility = AccessibilityTester::new().await?;
    
    // Configure accessibility standards to test against
    let standards = vec![
        AccessibilityStandard::WCAG2AA,
        AccessibilityStandard::Section508,
        AccessibilityStandard::ADACompliance,
    ];
    
    // Run comprehensive accessibility audit
    let audit_result = accessibility.run_full_audit(
        "main_application_window",
        &standards
    ).await?;
    
    println!("Accessibility audit completed");
    println!("Total issues found: {}", audit_result.issues.len());
    
    // Display critical issues
    for issue in &audit_result.issues {
        if issue.severity >= Severity::Critical {
            println!("[{}] {}: {}", issue.severity, issue.rule_id, issue.description);
            println!("  Location: {:?}", issue.location);
            if let Some(suggestion) = &issue.suggestion {
                println!("  Suggestion: {}", suggestion);
            }
        }
    }
    
    // Test keyboard navigation
    let keyboard_test = accessibility.test_keyboard_navigation().await?;
    println!("Keyboard navigation test: {:?}", keyboard_test);
    
    // Test screen reader compatibility
    let screen_reader_test = accessibility.test_screen_reader_compatibility().await?;
    println!("Screen reader test: {:?}", screen_reader_test);
    
    // Check color contrast
    let contrast_check = accessibility.check_color_contrast().await?;
    println!("Color contrast check: {:?}", contrast_check);
    
    // Generate detailed accessibility report
    let report = accessibility.generate_report(&audit_result).await?;
    
    // Save report to file
    std::fs::write("test_data/reports/accessibility_report.html", report)?;
    println!("Accessibility report saved to test_data/reports/accessibility_report.html");
    
    // Clean up
    accessibility.shutdown().await?;
    
    Ok(())
}