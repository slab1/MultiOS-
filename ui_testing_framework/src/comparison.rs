//! Screenshot Comparison and Visual Regression Testing Module
//!
//! Provides functionality for comparing screenshots, detecting visual differences,
//! and performing visual regression testing across different UI states and platforms.

use super::{FrameworkResult, FrameworkError, UIFrameworkConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use image::{ImageFormat, DynamicImage, GenericImageView, imageops};
use imageproc::noise::salt_and_pepper_noise;
use imageproc::filter::gaussian_blur_f32;
use std::path::PathBuf;
use std::fs;
use log::info;
use similar::{Algorithm, diff};

/// Screenshot comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotComparisonResult {
    pub test_name: String,
    pub baseline_path: PathBuf,
    pub current_path: PathBuf,
    pub difference_path: Option<PathBuf>,
    pub similarity_score: f64,
    pub pixel_difference_count: u64,
    pub total_pixels: u64,
    pub difference_percentage: f64,
    pub threshold_met: bool,
    pub timestamp: DateTime<Utc>,
    pub analysis: VisualAnalysis,
}

/// Detailed visual analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualAnalysis {
    pub color_differences: HashMap<String, u32>,
    pub structural_differences: Vec<StructuralDifference>,
    pub layout_differences: Vec<LayoutDifference>,
    pub performance_impact: PerformanceImpact,
}

/// Structural differences found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralDifference {
    pub position: (u32, u32),
    pub size: (u32, u32),
    pub element_type: String,
    pub change_type: String,
    pub severity: DifferenceSeverity,
}

/// Layout differences found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDifference {
    pub old_position: (i32, i32),
    pub new_position: (i32, i32),
    pub offset: (i32, i32),
    pub affected_elements: Vec<String>,
}

/// Performance impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub render_time_ms: u64,
    pub memory_usage_mb: u64,
    pub gpu_memory_usage_mb: u64,
    pub animation_smoothness_score: f64,
}

/// Severity levels for differences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DifferenceSeverity {
    Critical,
    High,
    Medium,
    Low,
    Cosmetic,
}

/// Visual difference with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDiff {
    pub id: String,
    pub region: ImageRegion,
    pub difference_type: DifferenceType,
    pub severity: DifferenceSeverity,
    pub description: String,
    pub affected_pixels: u32,
    pub color_histogram_diff: ColorHistogramDiff,
}

/// Image region definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceType {
    ColorChange,
    Movement,
    Addition,
    Removal,
    Blur,
    SharpnessChange,
    BrightnessChange,
    ContrastChange,
    Resize,
    LayoutChange,
}

/// Color histogram differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorHistogramDiff {
    pub red_histogram: HashMap<u8, f64>,
    pub green_histogram: HashMap<u8, f64>,
    pub blue_histogram: HashMap<u8, f64>,
    pub overall_similarity: f64,
}

/// Comparison configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    pub similarity_threshold: f64,
    pub pixel_difference_threshold: f64,
    pub color_difference_threshold: f64,
    pub ignore_animations: bool,
    pub ignore_time_sensitive_elements: bool,
    pub focus_regions: Vec<ImageRegion>,
    pub exclude_regions: Vec<ImageRegion>,
    pub comparison_algorithm: ComparisonAlgorithm,
}

/// Comparison algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonAlgorithm {
    PixelByPixel,
    StructuralSimilarity,
    PerceptualHash,
    FeatureMatching,
    HistogramComparison,
    TemplateMatching,
    MultiScale,
    Hybrid,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.95,
            pixel_difference_threshold: 0.05,
            color_difference_threshold: 10.0,
            ignore_animations: false,
            ignore_time_sensitive_elements: false,
            focus_regions: Vec::new(),
            exclude_regions: Vec::new(),
            comparison_algorithm: ComparisonAlgorithm::StructuralSimilarity,
        }
    }
}

/// Screenshot comparator for visual comparison
pub struct ScreenshotComparator {
    config: UIFrameworkConfig,
    comparison_config: ComparisonConfig,
    baseline_dir: PathBuf,
    current_dir: PathBuf,
    results_dir: PathBuf,
    cache: HashMap<String, DynamicImage>,
}

impl ScreenshotComparator {
    /// Create a new screenshot comparator
    pub fn new(config: &UIFrameworkConfig) -> Self {
        let baseline_dir = PathBuf::from(&config.baseline_dir);
        let current_dir = PathBuf::from(&config.screenshots_dir);
        let results_dir = PathBuf::from(&config.results_dir);
        
        std::fs::create_dir_all(&baseline_dir).unwrap_or_default();
        std::fs::create_dir_all(&current_dir).unwrap_or_default();
        std::fs::create_dir_all(&results_dir).unwrap_or_default();
        
        Self {
            config: config.clone(),
            comparison_config: ComparisonConfig::default(),
            baseline_dir,
            current_dir,
            results_dir,
            cache: HashMap::new(),
        }
    }

    /// Set custom comparison configuration
    pub fn set_comparison_config(&mut self, config: ComparisonConfig) {
        self.comparison_config = config;
    }

    /// Compare two screenshots
    pub async fn compare_screenshots(
        &mut self,
        test_name: String,
        baseline_filename: String,
        current_filename: String,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        info!("Comparing screenshots for test: {}", test_name);
        
        let baseline_path = self.baseline_dir.join(&baseline_filename);
        let current_path = self.current_dir.join(&current_filename);
        
        if !baseline_path.exists() {
            return Err(FrameworkError::ScreenshotComparison(
                format!("Baseline screenshot not found: {:?}", baseline_path)
            ));
        }
        
        if !current_path.exists() {
            return Err(FrameworkError::ScreenshotComparison(
                format!("Current screenshot not found: {:?}", current_path)
            ));
        }
        
        let baseline_image = self.load_or_cache_image(&baseline_path).await?;
        let current_image = self.load_or_cache_image(&current_path).await?;
        
        let result = self.perform_comparison(
            &test_name,
            &baseline_image,
            &current_image,
            baseline_path.clone(),
            current_path.clone(),
        ).await?;
        
        // Save difference image if significant differences found
        if result.difference_percentage > self.comparison_config.pixel_difference_threshold {
            let diff_path = self.save_difference_image(&test_name, &baseline_image, &current_image).await?;
            info!("Saved difference image: {:?}", diff_path);
        }
        
        Ok(result)
    }

    /// Perform the actual comparison using specified algorithm
    async fn perform_comparison(
        &self,
        test_name: &str,
        baseline: &DynamicImage,
        current: &DynamicImage,
        baseline_path: PathBuf,
        current_path: PathBuf,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        match self.comparison_config.comparison_algorithm {
            ComparisonAlgorithm::PixelByPixel => {
                self.pixel_by_pixel_comparison(test_name, baseline, current, baseline_path, current_path).await
            }
            ComparisonAlgorithm::StructuralSimilarity => {
                self.structural_similarity_comparison(test_name, baseline, current, baseline_path, current_path).await
            }
            ComparisonAlgorithm::PerceptualHash => {
                self.perceptual_hash_comparison(test_name, baseline, current, baseline_path, current_path).await
            }
            ComparisonAlgorithm::HistogramComparison => {
                self.histogram_comparison(test_name, baseline, current, baseline_path, current_path).await
            }
            ComparisonAlgorithm::MultiScale => {
                self.multi_scale_comparison(test_name, baseline, current, baseline_path, current_path).await
            }
            _ => {
                self.pixel_by_pixel_comparison(test_name, baseline, current, baseline_path, current_path).await
            }
        }
    }

    /// Pixel-by-pixel comparison
    async fn pixel_by_pixel_comparison(
        &self,
        test_name: &str,
        baseline: &DynamicImage,
        current: &DynamicImage,
        baseline_path: PathBuf,
        current_path: PathBuf,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        let (baseline_width, baseline_height) = baseline.dimensions();
        let (current_width, current_height) = current.dimensions();
        
        let total_pixels = (baseline_width * baseline_height).max(current_width * current_height);
        let mut pixel_difference_count = 0;
        
        // Create difference image
        let mut diff_image = DynamicImage::new_rgb8(
            baseline_width.max(current_width),
            baseline_height.max(current_height)
        );
        
        for y in 0..baseline_height.min(current_height) {
            for x in 0..baseline_width.min(current_width) {
                let baseline_pixel = baseline.get_pixel(x, y);
                let current_pixel = current.get_pixel(x, y);
                
                let diff_r = (baseline_pixel[0] as i32 - current_pixel[0] as i32).abs() as u32;
                let diff_g = (baseline_pixel[1] as i32 - current_pixel[1] as i32).abs() as u32;
                let diff_b = (baseline_pixel[2] as i32 - current_pixel[2] as i32).abs() as u32;
                
                let diff_value = (diff_r + diff_g + diff_b) / 3;
                
                if diff_value > 10 {
                    pixel_difference_count += 1;
                    // Mark difference in red
                    diff_image.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
                } else {
                    // Keep original pixel
                    diff_image.put_pixel(x, y, current_pixel);
                }
            }
        }
        
        let difference_percentage = (pixel_difference_count as f64 / total_pixels as f64) * 100.0;
        let similarity_score = 1.0 - difference_percentage / 100.0;
        let threshold_met = difference_percentage <= self.comparison_config.pixel_difference_threshold * 100.0;
        
        let analysis = self.analyze_differences(baseline, current, &diff_image).await;
        
        Ok(ScreenshotComparisonResult {
            test_name: test_name.to_string(),
            baseline_path,
            current_path,
            difference_path: None,
            similarity_score,
            pixel_difference_count,
            total_pixels,
            difference_percentage,
            threshold_met,
            timestamp: Utc::now(),
            analysis,
        })
    }

    /// Structural similarity comparison (simplified implementation)
    async fn structural_similarity_comparison(
        &self,
        test_name: &str,
        baseline: &DynamicImage,
        current: &DynamicImage,
        baseline_path: PathBuf,
        current_path: PathBuf,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        // Simplified SSIM calculation
        let (width, height) = baseline.dimensions();
        let total_pixels = (width * height) as f64;
        let mut sum_1 = 0.0;
        let mut sum_2 = 0.0;
        let mut sum_1_2 = 0.0;
        let mut sum_1_sq = 0.0;
        let mut sum_2_sq = 0.0;
        
        for y in 0..height {
            for x in 0..width {
                let p1 = baseline.get_pixel(x, y);
                let p2 = current.get_pixel(x, y);
                
                let gray1 = ((p1[0] as f64 * 0.299) + (p1[1] as f64 * 0.587) + (p1[2] as f64 * 0.114));
                let gray2 = ((p2[0] as f64 * 0.299) + (p2[1] as f64 * 0.587) + (p2[2] as f64 * 0.114));
                
                sum_1 += gray1;
                sum_2 += gray2;
                sum_1_2 += gray1 * gray2;
                sum_1_sq += gray1 * gray1;
                sum_2_sq += gray2 * gray2;
            }
        }
        
        let mean_1 = sum_1 / total_pixels;
        let mean_2 = sum_2 / total_pixels;
        let variance_1 = (sum_1_sq / total_pixels) - (mean_1 * mean_1);
        let variance_2 = (sum_2_sq / total_pixels) - (mean_2 * mean_2);
        let covariance = (sum_1_2 / total_pixels) - (mean_1 * mean_2);
        
        // Simplified SSIM
        let c1 = (0.01 * 255.0).powi(2);
        let c2 = (0.03 * 255.0).powi(2);
        
        let ssim = ((2.0 * mean_1 * mean_2 + c1) * (2.0 * covariance + c2)) /
                   ((mean_1 * mean_1 + mean_2 * mean_2 + c1) * (variance_1 + variance_2 + c2));
        
        let pixel_difference_count = ((1.0 - ssim) * total_pixels) as u64;
        let difference_percentage = (pixel_difference_count as f64 / total_pixels) * 100.0;
        let threshold_met = ssim >= self.comparison_config.similarity_threshold;
        
        let analysis = self.analyze_differences(baseline, current, &DynamicImage::new_rgb8(width, height)).await;
        
        Ok(ScreenshotComparisonResult {
            test_name: test_name.to_string(),
            baseline_path,
            current_path,
            difference_path: None,
            similarity_score: ssim,
            pixel_difference_count,
            total_pixels: total_pixels as u64,
            difference_percentage,
            threshold_met,
            timestamp: Utc::now(),
            analysis,
        })
    }

    /// Perceptual hash comparison (simplified implementation)
    async fn perceptual_hash_comparison(
        &self,
        test_name: &str,
        baseline: &DynamicImage,
        current: &DynamicImage,
        baseline_path: PathBuf,
        current_path: PathBuf,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        // Simplified pHash calculation
        let hash1 = self.calculate_perceptual_hash(baseline).await;
        let hash2 = self.calculate_perceptual_hash(current).await;
        
        let hamming_distance = self.calculate_hamming_distance(&hash1, &hash2);
        let total_bits = 64.0; // Assuming 64-bit hash
        let similarity_score = 1.0 - (hamming_distance as f64 / total_bits);
        let difference_percentage = (hamming_distance as f64 / total_bits) * 100.0;
        let pixel_difference_count = (difference_percentage / 100.0 * (baseline.width() * baseline.height()) as f64) as u64;
        let threshold_met = similarity_score >= self.comparison_config.similarity_threshold;
        
        let analysis = self.analyze_differences(baseline, current, &DynamicImage::new_rgb8(baseline.width(), baseline.height())).await;
        
        Ok(ScreenshotComparisonResult {
            test_name: test_name.to_string(),
            baseline_path,
            current_path,
            difference_path: None,
            similarity_score,
            pixel_difference_count,
            total_pixels: baseline.width() * baseline.height(),
            difference_percentage,
            threshold_met,
            timestamp: Utc::now(),
            analysis,
        })
    }

    /// Histogram comparison
    async fn histogram_comparison(
        &self,
        test_name: &str,
        baseline: &DynamicImage,
        current: &DynamicImage,
        baseline_path: PathBuf,
        current_path: PathBuf,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        let hist1 = self.calculate_histogram(baseline).await;
        let hist2 = self.calculate_histogram(current).await;
        
        let mut similarity = 0.0;
        let mut total_bins = 0;
        
        for (key, value1) in hist1 {
            if let Some(value2) = hist2.get(&key) {
                let min_val = value1.min(*value2);
                similarity += min_val as f64;
                total_bins += 1;
            }
        }
        
        if total_bins > 0 {
            similarity /= total_bins as f64;
        }
        
        let difference_percentage = (1.0 - similarity) * 100.0;
        let similarity_score = similarity;
        let pixel_difference_count = (difference_percentage / 100.0 * (baseline.width() * baseline.height()) as f64) as u64;
        let threshold_met = similarity_score >= self.comparison_config.similarity_threshold;
        
        let analysis = self.analyze_differences(baseline, current, &DynamicImage::new_rgb8(baseline.width(), baseline.height())).await;
        
        Ok(ScreenshotComparisonResult {
            test_name: test_name.to_string(),
            baseline_path,
            current_path,
            difference_path: None,
            similarity_score,
            pixel_difference_count,
            total_pixels: baseline.width() * baseline.height(),
            difference_percentage,
            threshold_met,
            timestamp: Utc::now(),
            analysis,
        })
    }

    /// Multi-scale comparison
    async fn multi_scale_comparison(
        &self,
        test_name: &str,
        baseline: &DynamicImage,
        current: &DynamicImage,
        baseline_path: PathBuf,
        current_path: PathBuf,
    ) -> FrameworkResult<ScreenshotComparisonResult> {
        let scales = [1.0, 0.5, 0.25];
        let mut total_similarity = 0.0;
        let mut total_pixels = 0.0;
        let mut pixel_differences = 0.0;
        
        for scale in scales {
            let scaled_baseline = baseline.resize_exact(
                (baseline.width() as f64 * scale) as u32,
                (baseline.height() as f64 * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            );
            
            let scaled_current = current.resize_exact(
                (current.width() as f64 * scale) as u32,
                (current.height() as f64 * scale) as u32,
                image::imageops::FilterType::Lanczos3,
            );
            
            let comparison_result = self.pixel_by_pixel_comparison(
                &format!("{}_scale_{}", test_name, scale),
                &scaled_baseline,
                &scaled_current,
                baseline_path.clone(),
                current_path.clone(),
            ).await?;
            
            total_similarity += comparison_result.similarity_score;
            total_pixels += comparison_result.total_pixels as f64;
            pixel_differences += comparison_result.pixel_difference_count as f64;
        }
        
        let avg_similarity = total_similarity / scales.len() as f64;
        let avg_difference_percentage = (pixel_differences / total_pixels) * 100.0;
        let threshold_met = avg_similarity >= self.comparison_config.similarity_threshold;
        
        let analysis = self.analyze_differences(baseline, current, &DynamicImage::new_rgb8(baseline.width(), baseline.height())).await;
        
        Ok(ScreenshotComparisonResult {
            test_name: test_name.to_string(),
            baseline_path,
            current_path,
            difference_path: None,
            similarity_score: avg_similarity,
            pixel_difference_count: pixel_differences as u64,
            total_pixels: total_pixels as u64,
            difference_percentage: avg_difference_percentage,
            threshold_met,
            timestamp: Utc::now(),
            analysis,
        })
    }

    /// Load or cache image
    async fn load_or_cache_image(&mut self, path: &PathBuf) -> FrameworkResult<DynamicImage> {
        if let Some(cached) = self.cache.get(path.to_str().unwrap_or("")) {
            return Ok(cached.clone());
        }
        
        let image = image::open(path)?;
        self.cache.insert(path.to_str().unwrap_or("").to_string(), image.clone());
        Ok(image)
    }

    /// Analyze differences in detail
    async fn analyze_differences(
        &self,
        baseline: &DynamicImage,
        current: &DynamicImage,
        diff_image: &DynamicImage,
    ) -> VisualAnalysis {
        let color_differences = self.analyze_color_differences(baseline, current).await;
        let structural_differences = self.find_structural_differences(diff_image).await;
        let layout_differences = self.find_layout_differences(baseline, current).await;
        let performance_impact = self.assess_performance_impact(diff_image).await;
        
        VisualAnalysis {
            color_differences,
            structural_differences,
            layout_differences,
            performance_impact,
        }
    }

    /// Analyze color differences
    async fn analyze_color_differences(&self, baseline: &DynamicImage, current: &DynamicImage) -> HashMap<String, u32> {
        let mut color_diffs = HashMap::new();
        
        let (width, height) = baseline.dimensions();
        let mut total_color_diff = 0u32;
        
        for y in 0..height {
            for x in 0..width {
                let b_pixel = baseline.get_pixel(x, y);
                let c_pixel = current.get_pixel(x, y);
                
                let diff = ((b_pixel[0] as i32 - c_pixel[0] as i32).abs() +
                           (b_pixel[1] as i32 - c_pixel[1] as i32).abs() +
                           (b_pixel[2] as i32 - c_pixel[2] as i32).abs()) / 3;
                
                total_color_diff += diff as u32;
            }
        }
        
        color_diffs.insert("total_color_difference".to_string(), total_color_diff);
        color_diffs.insert("average_color_difference".to_string(), total_color_diff / (width * height));
        
        color_diffs
    }

    /// Find structural differences
    async fn find_structural_differences(&self, diff_image: &DynamicImage) -> Vec<StructuralDifference> {
        let mut differences = Vec::new();
        let (width, height) = diff_image.dimensions();
        
        // Simple implementation to find connected regions of differences
        let mut visited = vec![vec![false; width as usize]; height as usize];
        
        for y in 0..height {
            for x in 0..width {
                if !visited[y as usize][x as usize] {
                    let pixel = diff_image.get_pixel(x, y);
                    
                    // Check if pixel is significantly different (red)
                    if pixel[0] > 200 && pixel[1] < 50 && pixel[2] < 50 {
                        let region = self.flood_fill_diff_region(diff_image, x, y, &mut visited);
                        
                        if region.width > 10 && region.height > 10 { // Ignore very small differences
                            differences.push(StructuralDifference {
                                position: (x, y),
                                size: (region.width, region.height),
                                element_type: "unknown".to_string(),
                                change_type: "visual_difference".to_string(),
                                severity: self.calculate_severity(region.width, region.height),
                            });
                        }
                    }
                }
            }
        }
        
        differences
    }

    /// Helper function for flood fill
    fn flood_fill_diff_region(&self, image: &DynamicImage, start_x: u32, start_y: u32, visited: &mut [Vec<bool>]) -> ImageRegion {
        let (width, height) = image.dimensions();
        let mut min_x = start_x;
        let mut min_y = start_y;
        let mut max_x = start_x;
        let mut max_y = start_y;
        
        let mut stack = vec![(start_x, start_y)];
        
        while let Some((x, y)) = stack.pop() {
            if y as usize >= visited.len() || x as usize >= visited[0].len() || visited[y as usize][x as usize] {
                continue;
            }
            
            visited[y as usize][x as usize] = true;
            
            let pixel = image.get_pixel(x, y);
            if pixel[0] > 200 && pixel[1] < 50 && pixel[2] < 50 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                
                // Add neighbors to stack
                if x > 0 { stack.push((x - 1, y)); }
                if x + 1 < width { stack.push((x + 1, y)); }
                if y > 0 { stack.push((x, y - 1)); }
                if y + 1 < height { stack.push((x, y + 1)); }
            }
        }
        
        ImageRegion {
            x: min_x,
            y: min_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        }
    }

    /// Calculate severity of difference
    fn calculate_severity(&self, width: u32, height: u32) -> DifferenceSeverity {
        let area = width * height;
        
        if area > 10000 {
            DifferenceSeverity::Critical
        } else if area > 5000 {
            DifferenceSeverity::High
        } else if area > 1000 {
            DifferenceSeverity::Medium
        } else if area > 100 {
            DifferenceSeverity::Low
        } else {
            DifferenceSeverity::Cosmetic
        }
    }

    /// Find layout differences
    async fn find_layout_differences(&self, baseline: &DynamicImage, current: &DynamicImage) -> Vec<LayoutDifference> {
        // Simplified layout difference detection
        // In a real implementation, this would use more sophisticated methods
        let differences = Vec::new();
        differences
    }

    /// Assess performance impact
    async fn assess_performance_impact(&self, diff_image: &DynamicImage) -> PerformanceImpact {
        // Simplified performance assessment
        PerformanceImpact {
            render_time_ms: 0, // Would be measured in real implementation
            memory_usage_mb: 0, // Would be measured in real implementation
            gpu_memory_usage_mb: 0, // Would be measured in real implementation
            animation_smoothness_score: 0.95, // Simulated score
        }
    }

    /// Save difference image
    async fn save_difference_image(&self, test_name: &str, baseline: &DynamicImage, current: &DynamicImage) -> FrameworkResult<PathBuf> {
        let diff_path = self.results_dir.join(format!("{}_diff.png", test_name));
        
        // Create simple difference visualization
        let mut diff_image = DynamicImage::new_rgb8(
            baseline.width().max(current.width()),
            baseline.height().max(current.height())
        );
        
        // Copy baseline
        for y in 0..baseline.height() {
            for x in 0..baseline.width() {
                let pixel = baseline.get_pixel(x, y);
                diff_image.put_pixel(x, y, pixel);
            }
        }
        
        // Add current overlay where different
        for y in 0..current.height() {
            for x in 0..current.width() {
                let b_pixel = baseline.get_pixel(x, y);
                let c_pixel = current.get_pixel(x, y);
                
                // Check if significantly different
                let diff = ((b_pixel[0] as i32 - c_pixel[0] as i32).abs() +
                           (b_pixel[1] as i32 - c_pixel[1] as i32).abs() +
                           (b_pixel[2] as i32 - c_pixel[2] as i32).abs()) / 3;
                
                if diff > 10 {
                    // Overlay current pixel with some transparency
                    let mut overlay_pixel = c_pixel;
                    overlay_pixel[3] = 150; // Semi-transparent
                    diff_image.put_pixel(x, y, overlay_pixel);
                }
            }
        }
        
        diff_image.save_with_format(&diff_path, ImageFormat::Png)?;
        Ok(diff_path)
    }

    /// Calculate perceptual hash (simplified)
    async fn calculate_perceptual_hash(&self, image: &DynamicImage) -> u64 {
        // Simplified pHash calculation
        // Resize to 32x32, convert to grayscale, and create hash
        let resized = image.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
        let mut hash = 0u64;
        
        for y in 0..32 {
            for x in 0..32 {
                let pixel = resized.get_pixel(x, y);
                let gray = ((pixel[0] as f64 * 0.299) + (pixel[1] as f64 * 0.587) + (pixel[2] as f64 * 0.114)) as u32;
                
                if gray > 128 {
                    hash |= 1 << (y * 32 + x);
                }
            }
        }
        
        hash
    }

    /// Calculate hamming distance between two hashes
    fn calculate_hamming_distance(&self, hash1: &u64, hash2: &u64) -> u32 {
        let mut hamming = 0;
        let mut x = hash1 ^ hash2;
        
        while x != 0 {
            hamming += 1;
            x &= x - 1; // Clear the lowest set bit
        }
        
        hamming
    }

    /// Calculate histogram
    async fn calculate_histogram(&self, image: &DynamicImage) -> HashMap<String, Vec<u32>> {
        let mut hist = HashMap::new();
        hist.insert("red".to_string(), vec![0; 256]);
        hist.insert("green".to_string(), vec![0; 256]);
        hist.insert("blue".to_string(), vec![0; 256]);
        
        let (width, height) = image.dimensions();
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                
                hist.get_mut("red").unwrap()[pixel[0] as usize] += 1;
                hist.get_mut("green").unwrap()[pixel[1] as usize] += 1;
                hist.get_mut("blue").unwrap()[pixel[2] as usize] += 1;
            }
        }
        
        hist
    }

    /// Compare all screenshots in the test suite
    pub async fn compare_all(&mut self) -> FrameworkResult<Vec<ScreenshotComparisonResult>> {
        info!("Comparing all screenshots...");
        
        let mut results = Vec::new();
        
        // Find all screenshots in current directory
        let entries = fs::read_dir(&self.current_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                
                // Only process PNG files
                if filename_str.ends_with(".png") {
                    let test_name = filename_str.trim_end_matches(".png").to_string();
                    let baseline_filename = filename_str.clone();
                    
                    match self.compare_screenshots(
                        test_name,
                        baseline_filename,
                        filename_str.to_string(),
                    ).await {
                        Ok(result) => results.push(result),
                        Err(e) => log::warn!("Failed to compare screenshot {}: {}", filename_str, e),
                    }
                }
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comparison_config_default() {
        let config = ComparisonConfig::default();
        assert_eq!(config.similarity_threshold, 0.95);
        assert_eq!(config.pixel_difference_threshold, 0.05);
        assert_eq!(config.ignore_animations, false);
    }
    
    #[test]
    fn test_difference_severity_ordering() {
        assert!(DifferenceSeverity::Critical > DifferenceSeverity::High);
        assert!(DifferenceSeverity::High > DifferenceSeverity::Medium);
        assert!(DifferenceSeverity::Medium > DifferenceSeverity::Low);
        assert!(DifferenceSeverity::Low > DifferenceSeverity::Cosmetic);
    }
    
    #[test]
    fn test_image_region_creation() {
        let region = ImageRegion {
            x: 10,
            y: 20,
            width: 100,
            height: 150,
        };
        
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 100);
        assert_eq!(region.height, 150);
    }
}