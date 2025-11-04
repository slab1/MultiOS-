//! Benchmark Result Reporter
//! 
//! This module provides comprehensive reporting capabilities for benchmark results,
//! including HTML reports, CSV export, comparison analysis, and performance insights.

use super::{BenchmarkResult, BenchmarkCategory, PerformanceComparison};
use crate::utils::{Timer, Size, Stats};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Report generation configuration
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub output_format: super::OutputFormat,
    pub include_statistics: bool,
    pub include_comparison: bool,
    pub include_system_info: bool,
    pub sort_by: SortBy,
    pub group_by_category: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            output_format: super::OutputFormat::Human,
            include_statistics: true,
            include_comparison: true,
            include_system_info: true,
            sort_by: SortBy::Name,
            group_by_category: true,
        }
    }
}

/// Sort options for report generation
#[derive(Debug, Clone)]
pub enum SortBy {
    Name,
    Category,
    OperationsPerSecond,
    Duration,
    Throughput,
}

/// Report generator
pub struct ReportGenerator {
    config: ReportConfig,
}

impl ReportGenerator {
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }
    
    /// Generate comprehensive report
    pub fn generate_report(&self, results: &[BenchmarkResult], comparisons: Option<&[PerformanceComparison]>) -> Result<String, Box<dyn std::error::Error>> {
        match self.config.output_format {
            super::OutputFormat::Human => self.generate_human_readable(results, comparisons),
            super::OutputFormat::Json => self.generate_json(results, comparisons),
            super::OutputFormat::Csv => self.generate_csv(results),
            super::OutputFormat::Html => self.generate_html(results, comparisons),
        }
    }
    
    /// Generate human-readable report
    fn generate_human_readable(&self, results: &[BenchmarkResult], comparisons: Option<&[PerformanceComparison]>) -> Result<String, Box<dyn std::error::Error>> {
        let mut report = String::new();
        
        report.push_str("=== MultiOS Performance Benchmark Report ===\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));
        
        // System Information
        if self.config.include_system_info {
            report.push_str("=== System Information ===\n");
            if let Ok(sys_info) = super::SystemInfo::collect() {
                report.push_str(&format!("OS: {} ({} {})\n", sys_info.os_name, sys_info.kernel_version, sys_info.architecture));
                report.push_str(&format!("CPU: {} ({} cores)\n", sys_info.cpu_model, sys_info.cpu_cores));
                report.push_str(&format!("Memory: {} total, {} available\n\n", 
                    Size::format_bytes(sys_info.memory_total),
                    Size::format_bytes(sys_info.memory_available)));
            }
            report.push_str("\n");
        }
        
        // Results by category
        let mut categories = HashMap::new();
        for result in results {
            categories.entry(result.category.clone()).or_insert_with(Vec::new).push(result);
        }
        
        let mut sorted_categories: Vec<_> = categories.into_iter().collect();
        if !self.config.group_by_category {
            sorted_categories = vec![(BenchmarkCategory::CPU, results.to_vec())];
        }
        
        for (category, category_results) in sorted_categories {
            report.push_str(&format!("=== {} Performance ===\n", category_name(&category)));
            
            // Sort results based on configuration
            let mut sorted_results = category_results.clone();
            self.sort_results(&mut sorted_results);
            
            for result in sorted_results {
                report.push_str(&format!("\n{}:\n", result.name));
                report.push_str(&format!("  Duration: {}\n", Timer::format_duration(&result.duration)));
                report.push_str(&format!("  Iterations: {}\n", result.iterations));
                report.push_str(&format!("  Operations/Second: {:.2}\n", result.operations_per_second));
                report.push_str(&format!("  Throughput: {}\n", self.format_throughput(&result)));
                
                if self.config.include_statistics {
                    // Add metadata as statistics
                    for (key, value) in &result.metadata {
                        report.push_str(&format!("  {}: {}\n", key, value));
                    }
                }
            }
            report.push_str("\n");
        }
        
        // Performance comparisons
        if self.config.include_comparison {
            if let Some(comparisons) = comparisons {
                report.push_str("=== Performance Comparison ===\n");
                for comparison in comparisons {
                    report.push_str(&format!("\n{}:\n", comparison.benchmark_name));
                    report.push_str(&format!("  Current: {:.2} {}\n", 
                        comparison.current_result.operations_per_second,
                        comparison.current_result.unit));
                    
                    if let Some(baseline) = &comparison.baseline_result {
                        report.push_str(&format!("  Baseline: {:.2} {}\n",
                            baseline.operations_per_second,
                            baseline.unit));
                        report.push_str(&format!("  Change: {:.2}% ({})\n",
                            comparison.percentage_change,
                            if comparison.is_improvement { "faster" } else { "slower" }));
                    } else {
                        report.push_str("  No baseline for comparison\n");
                    }
                }
                report.push_str("\n");
            }
        }
        
        Ok(report)
    }
    
    /// Generate JSON report
    fn generate_json(&self, results: &[BenchmarkResult], comparisons: Option<&[PerformanceComparison]>) -> Result<String, Box<dyn std::error::Error>> {
        let mut json_data = serde_json::Map::new();
        
        json_data.insert("timestamp".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        json_data.insert("results".to_string(), serde_json::to_value(results)?);
        
        if let Some(comparisons) = comparisons {
            json_data.insert("comparisons".to_string(), serde_json::to_value(comparisons)?);
        }
        
        if self.config.include_system_info {
            if let Ok(sys_info) = super::SystemInfo::collect() {
                json_data.insert("system_info".to_string(), serde_json::to_value(sys_info)?);
            }
        }
        
        serde_json::to_string_pretty(&json_data).map_err(|e| e.into())
    }
    
    /// Generate CSV report
    fn generate_csv(&self, results: &[BenchmarkResult]) -> Result<String, Box<dyn std::error::Error>> {
        let mut csv = String::new();
        
        // Header
        csv.push_str("Category,Name,Duration_MS,Iterations,Ops_Per_Sec,Throughput,Unit,Timestamp\n");
        
        // Data rows
        for result in results {
            let category_name = category_name(&result.category);
            let duration_ms = result.duration.as_millis();
            let timestamp = result.timestamp.to_rfc3339();
            
            csv.push_str(&format!("{},{},{},{},{:.2},{:.2},{},{}\n",
                escape_csv(&category_name),
                escape_csv(&result.name),
                duration_ms,
                result.iterations,
                result.operations_per_second,
                result.throughput,
                escape_csv(&result.unit),
                timestamp
            ));
        }
        
        Ok(csv)
    }
    
    /// Generate HTML report
    fn generate_html(&self, results: &[BenchmarkResult], comparisons: Option<&[PerformanceComparison]>) -> Result<String, Box<dyn std::error::Error>> {
        let mut html = String::new();
        
        // HTML header
        html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MultiOS Performance Benchmark Report</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 20px; line-height: 1.6; }
        .container { max-width: 1200px; margin: 0 auto; }
        h1, h2, h3 { color: #333; }
        .section { margin-bottom: 30px; }
        .benchmark-group { margin-bottom: 25px; }
        .benchmark-item { background: #f5f5f5; padding: 15px; margin: 10px 0; border-radius: 5px; }
        .metric { display: inline-block; margin-right: 20px; }
        .value { font-weight: bold; color: #0066cc; }
        .unit { color: #666; font-size: 0.9em; }
        .improvement { color: #28a745; }
        .degradation { color: #dc3545; }
        .baseline { color: #6c757d; font-style: italic; }
        table { width: 100%; border-collapse: collapse; margin: 10px 0; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .progress-bar { width: 100%; height: 20px; background-color: #f0f0f0; border-radius: 10px; overflow: hidden; }
        .progress-fill { height: 100%; background-color: #007bff; transition: width 0.3s ease; }
    </style>
</head>
<body>
<div class="container">
"#);
        
        // Title
        html.push_str(&format!("<h1>MultiOS Performance Benchmark Report</h1>"));
        html.push_str(&format!("<p><em>Generated: {}</em></p>", chrono::Utc::now()));
        
        // System Information
        if self.config.include_system_info {
            if let Ok(sys_info) = super::SystemInfo::collect() {
                html.push_str(r#"<div class="section">
                    <h2>System Information</h2>
                    <table>
                        <tr><td><strong>Operating System</strong></td><td>"#);
                html.push_str(&format!("{} ({} {})", sys_info.os_name, sys_info.kernel_version, sys_info.architecture));
                html.push_str("</td></tr>");
                html.push_str("<tr><td><strong>CPU</strong></td><td>");
                html.push_str(&format!("{} ({} cores)", sys_info.cpu_model, sys_info.cpu_cores));
                html.push_str("</td></tr>");
                html.push_str("<tr><td><strong>Memory</strong></td><td>");
                html.push_str(&format!("{} total, {} available", 
                    Size::format_bytes(sys_info.memory_total),
                    Size::format_bytes(sys_info.memory_available)));
                html.push_str("</td></tr>");
                html.push_str("</table></div>");
            }
        }
        
        // Performance Results
        html.push_str("<div class='section'><h2>Benchmark Results</h2>");
        
        // Group by category
        let mut categories = HashMap::new();
        for result in results {
            categories.entry(result.category.clone()).or_insert_with(Vec::new).push(result);
        }
        
        for (category, category_results) in categories {
            html.push_str(&format!("<div class='benchmark-group'><h3>{} Performance</h3>", category_name(&category)));
            
            // Sort results
            let mut sorted_results = category_results.clone();
            self.sort_results(&mut sorted_results);
            
            for result in sorted_results {
                html.push_str("<div class='benchmark-item'>");
                html.push_str(&format!("<h4>{}</h4>", result.name));
                
                html.push_str("<div class='metrics'>");
                html.push_str(&format!("<div class='metric'>Duration: <span class='value'>{}</span></div>", 
                    Timer::format_duration(&result.duration)));
                html.push_str(&format!("<div class='metric'>Iterations: <span class='value'>{}</span></div>", 
                    result.iterations));
                html.push_str(&format!("<div class='metric'>Ops/Sec: <span class='value'>{:.2}</span><span class='unit'> {}</span></div>",
                    result.operations_per_second, self.escape_html(&result.unit)));
                html.push_str(&format!("<div class='metric'>Throughput: <span class='value'>{}</span></div>",
                    self.format_throughput(&result)));
                html.push_str("</div>");
                
                // Progress bar for relative performance
                let max_ops = results.iter().map(|r| r.operations_per_second).fold(0.0, f64::max);
                if max_ops > 0.0 {
                    let percentage = (result.operations_per_second / max_ops) * 100.0;
                    html.push_str("<div class='progress-bar'>");
                    html.push_str(&format!("<div class='progress-fill' style='width: {:.1}%'></div>", percentage));
                    html.push_str("</div>");
                }
                
                html.push_str("</div>");
            }
            
            html.push_str("</div>");
        }
        
        html.push_str("</div>");
        
        // Performance Comparisons
        if self.config.include_comparison {
            if let Some(comparisons) = comparisons {
                html.push_str("<div class='section'><h2>Performance Comparison</h2>");
                
                for comparison in comparisons {
                    let change_class = if comparison.is_improvement { "improvement" } else { "degradation" };
                    
                    html.push_str("<div class='benchmark-item'>");
                    html.push_str(&format!("<h4>{}</h4>", comparison.benchmark_name));
                    
                    html.push_str(&format!("<div>Current: <span class='value'>{:.2} {}</span></div>",
                        comparison.current_result.operations_per_second,
                        self.escape_html(&comparison.current_result.unit)));
                    
                    if let Some(baseline) = &comparison.baseline_result {
                        html.push_str(&format!("<div>Baseline: <span class='baseline'>{:.2} {}</span></div>",
                            baseline.operations_per_second,
                            self.escape_html(&baseline.unit)));
                        html.push_str(&format!("<div>Change: <span class='{}'>+{:.2}%</span></div>",
                            change_class, comparison.percentage_change));
                    }
                    
                    html.push_str("</div>");
                }
                
                html.push_str("</div>");
            }
        }
        
        // HTML footer
        html.push_str(r#"</div>
</body>
</html>"#);
        
        Ok(html)
    }
    
    /// Sort results based on configuration
    fn sort_results(&self, results: &mut [BenchmarkResult]) {
        match self.config.sort_by {
            SortBy::Name => results.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Category => results.sort_by(|a, b| a.category.cmp(&b.category)),
            SortBy::OperationsPerSecond => results.sort_by(|a, b| b.operations_per_second.partial_cmp(&a.operations_per_second).unwrap()),
            SortBy::Duration => results.sort_by(|a, b| a.duration.cmp(&b.duration)),
            SortBy::Throughput => results.sort_by(|a, b| b.throughput.partial_cmp(&a.throughput).unwrap()),
        }
    }
    
    /// Format throughput for display
    fn format_throughput(&self, result: &BenchmarkResult) -> String {
        let throughput = result.throughput;
        let unit = &result.unit;
        
        if throughput >= 1_000_000.0 {
            format!("{:.2} M{}", throughput / 1_000_000.0, unit)
        } else if throughput >= 1_000.0 {
            format!("{:.2} k{}", throughput / 1_000.0, unit)
        } else {
            format!("{:.2} {}", throughput, unit)
        }
    }
    
    /// Escape HTML characters
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

/// Export benchmark results to file
pub fn export_results(results: &[BenchmarkResult], path: &str, format: super::OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    let config = ReportConfig {
        output_format: format,
        include_statistics: true,
        include_comparison: false,
        include_system_info: true,
        sort_by: SortBy::Name,
        group_by_category: true,
    };
    
    let generator = ReportGenerator::new(config);
    let report = generator.generate_report(results, None)?;
    
    let mut file = File::create(path)?;
    file.write_all(report.as_bytes())?;
    
    Ok(())
}

/// Helper function to get category name as string
fn category_name(category: &BenchmarkCategory) -> &'static str {
    match category {
        BenchmarkCategory::CPU => "CPU",
        BenchmarkCategory::Memory => "Memory",
        BenchmarkCategory::FileSystem => "File System",
        BenchmarkCategory::Network => "Network",
        BenchmarkCategory::BootTime => "Boot Time",
        BenchmarkCategory::Syscalls => "System Calls",
    }
}

/// Helper function to escape CSV values
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Generate summary statistics
pub fn generate_summary(results: &[BenchmarkResult]) -> HashMap<String, String> {
    let mut summary = HashMap::new();
    
    if results.is_empty() {
        summary.insert("total_benchmarks".to_string(), "0".to_string());
        return summary;
    }
    
    summary.insert("total_benchmarks".to_string(), results.len().to_string());
    
    // Category counts
    let mut category_counts = HashMap::new();
    for result in results {
        let category = category_name(&result.category);
        *category_counts.entry(category.to_string()).or_insert(0) += 1;
    }
    
    for (category, count) in category_counts {
        summary.insert(format!("{}_benchmarks", category.to_lowercase()), count.to_string());
    }
    
    // Performance statistics
    let all_ops_per_sec: Vec<f64> = results.iter().map(|r| r.operations_per_second).collect();
    let all_throughputs: Vec<f64> = results.iter().map(|r| r.throughput).collect();
    
    if !all_ops_per_sec.is_empty() {
        let stats = Stats::analyze(&all_ops_per_sec);
        summary.insert("avg_ops_per_sec".to_string(), format!("{:.2}", stats.mean));
        summary.insert("max_ops_per_sec".to_string(), format!("{:.2}", stats.max));
        summary.insert("min_ops_per_sec".to_string(), format!("{:.2}", stats.min));
        summary.insert("cv_ops_per_sec".to_string(), format!("{:.2}%", Stats::coefficient_of_variation(&all_ops_per_sec)));
    }
    
    if !all_throughputs.is_empty() {
        let throughput_stats = Stats::analyze(&all_throughputs);
        summary.insert("avg_throughput".to_string(), format!("{:.2}", throughput_stats.mean));
        summary.insert("max_throughput".to_string(), format!("{:.2}", throughput_stats.max));
    }
    
    // Total execution time
    let total_time: Duration = results.iter().map(|r| r.duration).sum();
    summary.insert("total_execution_time".to_string(), Timer::format_duration(&total_time));
    
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchmarkResult;
    
    #[test]
    fn test_generate_summary() {
        let results = vec![
            BenchmarkResult {
                name: "Test 1".to_string(),
                category: BenchmarkCategory::CPU,
                duration: Duration::from_millis(100),
                iterations: 1000,
                operations_per_second: 100.0,
                throughput: 1000.0,
                unit: "ops/sec".to_string(),
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
            },
            BenchmarkResult {
                name: "Test 2".to_string(),
                category: BenchmarkCategory::Memory,
                duration: Duration::from_millis(200),
                iterations: 2000,
                operations_per_second: 200.0,
                throughput: 2000.0,
                unit: "ops/sec".to_string(),
                metadata: HashMap::new(),
                timestamp: chrono::Utc::now(),
            },
        ];
        
        let summary = generate_summary(&results);
        assert_eq!(summary.get("total_benchmarks"), Some(&"2".to_string()));
        assert!(summary.contains_key("avg_ops_per_sec"));
    }
}