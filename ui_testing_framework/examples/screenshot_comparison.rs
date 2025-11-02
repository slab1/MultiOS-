//! Screenshot comparison example demonstrating visual regression testing
//! 
//! This example shows:
//! - Taking screenshots of UI components
//! - Comparing screenshots using different algorithms
//! - Generating visual diffs
//! - Setting comparison thresholds

use multios_ui_testing::*;
use tokio;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Screenshot Comparison Example");
    
    // Initialize the comparison engine
    let mut comparison = ComparisonEngine::new().await?;
    
    // Define paths to baseline and test screenshots
    let baseline_path = PathBuf::from("test_data/baseline/login_screen.png");
    let current_path = PathBuf::from("test_data/current/login_screen.png");
    let diff_path = PathBuf::from("test_data/diffs/login_screen_diff.png");
    
    // Take a screenshot of the current UI state
    let current_screenshot = comparison.capture_screenshot("login_screen").await?;
    
    // Load baseline screenshot
    let baseline_screenshot = comparison.load_baseline(&baseline_path).await?;
    
    // Perform pixel-perfect comparison
    let pixel_result = comparison.compare_pixel_perfect(
        &current_screenshot,
        &baseline_screenshot,
        0.0 // Exact match required
    ).await?;
    
    println!("Pixel-perfect comparison result: {:?}", pixel_result);
    
    // Perform SSIM comparison (Structural Similarity Index)
    let ssim_result = comparison.compare_ssim(
        &current_screenshot,
        &baseline_screenshot,
        0.95 // 95% similarity threshold
    ).await?;
    
    println!("SSIM comparison result: {:?}", ssim_result);
    
    // Generate visual diff
    if ssim_result.similarity < 0.95 {
        let diff_image = comparison.generate_diff(
            &current_screenshot,
            &baseline_screenshot,
            ComparisonMode::HighlightDifferences,
            30 // Highlight threshold
        ).await?;
        
        comparison.save_image(&diff_path, &diff_image).await?;
        println!("Visual diff saved to: {:?}", diff_path);
    }
    
    // Compare using perceptual hashing
    let phash_result = comparison.compare_perceptual_hash(
        &current_screenshot,
        &baseline_screenshot,
        10 // Hamming distance threshold
    ).await?;
    
    println!("Perceptual hash comparison: {:?}", phash_result);
    
    // Clean up
    comparison.shutdown().await?;
    
    Ok(())
}