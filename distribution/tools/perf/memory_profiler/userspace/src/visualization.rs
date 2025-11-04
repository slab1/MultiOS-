//! Memory Visualization Module
//!
//! Provides comprehensive memory profiling visualization capabilities including
//! real-time charts, heatmaps, and analysis graphs.

use plotters::prelude::*;
use plotters::series::LineSeries;
use plotters::chart::SeriesChartPosition;
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDataPoint {
    pub timestamp: u64,
    pub allocated: u64,
    pub free: u64,
    pub cache_used: u64,
    pub stack_used: u64,
    pub fragmentation: f32,
    pub memory_pressure: f32,
}

/// Cache performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDataPoint {
    pub timestamp: u64,
    pub l1_hit_ratio: f32,
    pub l2_hit_ratio: f32,
    pub l3_hit_ratio: f32,
    pub latency_ns: u32,
}

/// Stack usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackDataPoint {
    pub timestamp: u64,
    pub thread_id: u32,
    pub stack_depth: u32,
    pub stack_usage: usize,
    pub peak_usage: usize,
}

/// Leak detection data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakDataPoint {
    pub timestamp: u64,
    pub address: u64,
    pub size: usize,
    pub suspicion_score: f32,
    pub leak_type: String,
}

/// NUMA node data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NUMADataPoint {
    pub timestamp: u64,
    pub node_id: u8,
    pub local_access_ratio: f32,
    pub memory_utilization: f32,
    pub temperature: f32,
    pub bandwidth_utilization: f32,
}

/// Memory heatmap cell
#[derive(Debug, Clone)]
struct HeatmapCell {
    pub x: f32,
    pub y: f32,
    pub intensity: f32,
    pub label: String,
}

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub show_grid: bool,
    pub show_legend: bool,
    pub color_scheme: ColorScheme,
}

/// Color schemes for visualization
#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    Viridis,
    Plasma,
    CoolWarm,
    HeatMap,
}

/// Main memory visualizer
pub struct MemoryVisualizer {
    config: AppConfig,
    data_cache: HashMap<String, Vec<serde_json::Value>>,
}

impl MemoryVisualizer {
    /// Create new memory visualizer
    pub fn new(config: AppConfig) -> Self {
        MemoryVisualizer {
            config,
            data_cache: HashMap::new(),
        }
    }
    
    /// Set chart type for visualization
    pub fn set_chart_type(&mut self, chart_type: &str) {
        // Chart type configuration
    }
    
    /// Set chart dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        // Dimension configuration
    }
    
    /// Generate memory usage visualization
    pub async fn generate_memory_usage_chart(
        &self,
        data: &[MemoryDataPoint],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption(config.title, ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0u64..data.last().map(|d| d.timestamp).unwrap_or(1000),
                0u64..data.iter().map(|d| d.allocated).max().unwrap_or(1000000)
            )?;
        
        chart.configure_mesh()
            .x_desc(config.x_label.as_str())
            .y_desc(config.y_label.as_str())
            .draw()?;
        
        // Plot allocated memory line
        let allocated_series: LineSeries<_, _> = data.iter()
            .map(|d| (d.timestamp, d.allocated))
            .collect();
        chart.draw_series(allocated_series)?
            .label("Allocated Memory")
            .style(&BLUE.stroke_width(2));
        
        // Plot free memory line
        let free_series: LineSeries<_, _> = data.iter()
            .map(|d| (d.timestamp, d.free))
            .collect();
        chart.draw_series(free_series)?
            .label("Free Memory")
            .style(&GREEN.stroke_width(2));
        
        if config.show_legend {
            chart.configure_series_labels()
                .border_style(&BLACK)
                .draw()?;
        }
        
        root.present()?;
        println!("Memory usage chart saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate cache performance visualization
    pub async fn generate_cache_performance_chart(
        &self,
        data: &[CacheDataPoint],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let max_time = data.last().map(|d| d.timestamp).unwrap_or(1000);
        let max_ratio = 1.0f32;
        
        let mut chart = ChartBuilder::on(&root)
            .caption(config.title, ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0u64..max_time, 0.0f32..max_ratio)?;
        
        chart.configure_mesh()
            .x_desc(config.x_label.as_str())
            .y_desc(config.y_label.as_str())
            .draw()?;
        
        // Plot L1 cache hit ratio
        let l1_series: LineSeries<_, _> = data.iter()
            .map(|d| (d.timestamp, d.l1_hit_ratio))
            .collect();
        chart.draw_series(l1_series)?
            .label("L1 Cache Hit Ratio")
            .style(&BLUE.stroke_width(2));
        
        // Plot L2 cache hit ratio
        let l2_series: LineSeries<_, _> = data.iter()
            .map(|d| (d.timestamp, d.l2_hit_ratio))
            .collect();
        chart.draw_series(l2_series)?
            .label("L2 Cache Hit Ratio")
            .style(&RED.stroke_width(2));
        
        // Plot L3 cache hit ratio
        let l3_series: LineSeries<_, _> = data.iter()
            .map(|d| (d.timestamp, d.l3_hit_ratio))
            .collect();
        chart.draw_series(l3_series)?
            .label("L3 Cache Hit Ratio")
            .style(&GREEN.stroke_width(2));
        
        if config.show_legend {
            chart.configure_series_labels()
                .border_style(&BLACK)
                .draw()?;
        }
        
        root.present()?;
        println!("Cache performance chart saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate stack usage visualization
    pub async fn generate_stack_usage_chart(
        &self,
        data: &[StackDataPoint],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        // Group data by thread
        let mut threads: HashMap<u32, Vec<&StackDataPoint>> = HashMap::new();
        for point in data {
            threads.entry(point.thread_id).or_insert_with(Vec::new).push(point);
        }
        
        let max_time = data.last().map(|d| d.timestamp).unwrap_or(1000);
        let max_usage = data.iter().map(|d| d.stack_usage).max().unwrap_or(100000);
        
        let mut chart = ChartBuilder::on(&root)
            .caption(config.title, ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0u64..max_time, 0usize..max_usage)?;
        
        chart.configure_mesh()
            .x_desc(config.x_label.as_str())
            .y_desc(config.y_label.as_str())
            .draw()?;
        
        // Plot stack usage for each thread
        let colors = [BLUE, RED, GREEN, YELLOW, MAGENTA, CYAN];
        for (i, (thread_id, thread_data)) in threads.iter().enumerate() {
            let color = colors[i % colors.len()];
            let series: LineSeries<_, _> = thread_data.iter()
                .map(|d| (d.timestamp, d.stack_usage))
                .collect();
            
            chart.draw_series(series)?
                .label(format!("Thread {}", thread_id))
                .style(color.stroke_width(2));
        }
        
        if config.show_legend {
            chart.configure_series_labels()
                .border_style(&BLACK)
                .draw()?;
        }
        
        root.present()?;
        println!("Stack usage chart saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate leak detection visualization
    pub async fn generate_leak_detection_chart(
        &self,
        data: &[LeakDataPoint],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        // Create scatter plot for leaks by size vs suspicion score
        let max_size = data.iter().map(|d| d.size).max().unwrap_or(1000000);
        let max_suspicion = data.iter().map(|d| d.suspicion_score).max().unwrap_or(1.0);
        
        let mut chart = ChartBuilder::on(&root)
            .caption(config.title, ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(0usize..max_size, 0.0f32..max_suspicion)?;
        
        chart.configure_mesh()
            .x_desc("Allocation Size (bytes)")
            .y_desc("Suspicion Score")
            .draw()?;
        
        // Color code by leak type
        let leak_types: HashMap<String, RGBColor> = [
            ("Direct".to_string(), RED),
            ("Indirect".to_string(), ORANGE),
            ("Resource".to_string(), YELLOW),
            ("Fragmentation".to_string(), MAGENTA),
        ].iter().cloned().collect();
        
        // Group data by leak type
        let mut type_groups: HashMap<String, Vec<&LeakDataPoint>> = HashMap::new();
        for point in data {
            type_groups.entry(point.leak_type.clone())
                .or_insert_with(Vec::new)
                .push(point);
        }
        
        // Plot each leak type
        for (leak_type, leaks) in type_groups {
            let color = leak_types.get(&leak_type).unwrap_or(&BLUE);
            let points: Vec<(usize, f32)> = leaks.iter()
                .map(|d| (d.size, d.suspicion_score))
                .collect();
            
            chart.draw_series(points.iter().map(|(x, y)| {
                Circle::new((*x, *y), 3, color.filled())
            }))?
                .label(leak_type.as_str());
        }
        
        if config.show_legend {
            chart.configure_series_labels()
                .border_style(&BLACK)
                .draw()?;
        }
        
        root.present()?;
        println!("Leak detection chart saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate NUMA topology visualization
    pub async fn generate_numa_topology_chart(
        &self,
        data: &[NUMADataPoint],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        // Create heatmap for NUMA nodes
        let nodes: HashMap<u8, Vec<&NUMADataPoint>> = data.iter()
            .filter(|d| d.node_id < 8) // Assume max 8 NUMA nodes
            .fold(HashMap::new(), |mut map, point| {
                map.entry(point.node_id).or_insert_with(Vec::new).push(point);
                map
            });
        
        let node_count = nodes.len();
        let grid_cols = (node_count as f32).sqrt().ceil() as usize;
        let grid_rows = (node_count as f32 / grid_cols as f32).ceil() as usize;
        
        let cell_width = config.width as f32 / grid_cols as f32;
        let cell_height = config.height as f32 / grid_rows as f32;
        
        // Draw NUMA node heatmap
        for (i, (node_id, node_data)) in nodes.iter().enumerate() {
            let col = i % grid_cols;
            let row = i / grid_cols;
            
            let x = col as f32 * cell_width;
            let y = row as f32 * cell_height;
            
            // Calculate average metrics for this node
            let avg_utilization = node_data.iter()
                .map(|d| d.memory_utilization)
                .sum::<f32>() / node_data.len() as f32;
            
            let avg_temperature = node_data.iter()
                .map(|d| d.temperature)
                .sum::<f32>() / node_data.len() as f32;
            
            // Color based on utilization (green = good, red = high usage)
            let color = if avg_utilization < 0.5 {
                RGBColor(0, 255, 0) // Green
            } else if avg_utilization < 0.8 {
                RGBColor(255, 255, 0) // Yellow
            } else {
                RGBColor(255, 0, 0) // Red
            };
            
            // Draw rectangle
            let area = Rectangle::new(
                (x, y),
                (x + cell_width, y + cell_height),
                color.filled()
            );
            root.draw(&area)?;
            
            // Draw label
            let label = format!("Node {}: {:.1}%", node_id, avg_utilization * 100.0);
            root.draw_text(
                &label,
                &("sans-serif", 12.0),
                (x + 5.0, y + 20.0),
                &BLACK
            )?;
        }
        
        println!("NUMA topology chart saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate comprehensive memory heatmap
    pub async fn generate_memory_heatmap(
        &self,
        data: &[MemoryDataPoint],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let heatmap_width = config.width as f32;
        let heatmap_height = config.height as f32;
        
        // Create heatmap cells
        let cell_width = heatmap_width / data.len() as f32;
        let mut max_allocated = 0u64;
        
        for point in data {
            if point.allocated > max_allocated {
                max_allocated = point.allocated;
            }
        }
        
        // Draw heatmap
        for (i, point) in data.iter().enumerate() {
            let x = i as f32 * cell_width;
            let intensity = point.allocated as f32 / max_allocated as f32;
            
            // Color based on intensity (blue = low, red = high)
            let red = (intensity * 255.0) as u8;
            let green = ((1.0 - intensity) * 128.0) as u8;
            let blue = 0;
            let color = RGBColor(red, green, blue);
            
            let area = Rectangle::new(
                (x, 0.0),
                (x + cell_width, heatmap_height),
                color.filled()
            );
            root.draw(&area)?;
        }
        
        // Add legend
        let legend_y = heatmap_height - 40.0;
        for i in 0..10 {
            let intensity = i as f32 / 9.0;
            let red = (intensity * 255.0) as u8;
            let green = ((1.0 - intensity) * 128.0) as u8;
            let blue = 0;
            let color = RGBColor(red, green, blue);
            
            let legend_x = (heatmap_width - 200.0) + (i as f32 * 18.0);
            let area = Rectangle::new(
                (legend_x, legend_y),
                (legend_x + 15.0, legend_y + 15.0),
                color.filled()
            );
            root.draw(&area)?;
        }
        
        root.draw_text(
            "Low",
            &("sans-serif", 10.0),
            (heatmap_width - 220.0, legend_y + 10.0),
            &BLACK
        )?;
        
        root.draw_text(
            "High",
            &("sans-serif", 10.0),
            (heatmap_width - 10.0, legend_y + 10.0),
            &BLACK
        )?;
        
        println!("Memory heatmap saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Generate allocation size distribution histogram
    pub async fn generate_allocation_distribution_chart(
        &self,
        allocation_sizes: &[usize],
        output_path: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = SVGBackend::new(output_path, (config.width, config.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        if allocation_sizes.is_empty() {
            return Ok(());
        }
        
        // Create bins for histogram
        let min_size = allocation_sizes.iter().min().unwrap();
        let max_size = allocation_sizes.iter().max().unwrap();
        let bin_count = 20;
        let bin_width = (max_size - min_size) as f32 / bin_count as f32;
        
        let mut bins = vec![0u32; bin_count];
        for &size in allocation_sizes {
            let bin_index = ((size - min_size) as f32 / bin_width).floor() as usize;
            if bin_index < bin_count {
                bins[bin_index] += 1;
            }
        }
        
        let max_count = bins.iter().max().unwrap_or(&1);
        
        let mut chart = ChartBuilder::on(&root)
            .caption(config.title, ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(60)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0usize..bin_count,
                0u32..*max_count
            )?;
        
        chart.configure_mesh()
            .x_desc("Allocation Size")
            .y_desc("Frequency")
            .draw()?;
        
        // Draw histogram bars
        for (i, count) in bins.iter().enumerate() {
            let x0 = i;
            let x1 = i + 1;
            let bar = Rectangle::new(
                (x0, 0),
                (x1, *count),
                BLUE.filled()
            );
            chart.draw_series(std::iter::once(bar))?;
            
            // Add value labels for non-zero bins
            if *count > 0 {
                chart.draw_series(std::iter::once(
                    Text::new(
                        format!("{}", count),
                        (i + 0.5, *count + max_count / 20),
                        ("sans-serif", 10.0),
                        &BLACK
                    )
                ))?;
            }
        }
        
        root.present()?;
        println!("Allocation distribution chart saved to: {}", output_path.display());
        Ok(())
    }
    
    /// Load data from JSON file
    pub async fn load_data_from_file(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let data: Vec<serde_json::Value> = serde_json::from_reader(reader)?;
        
        self.data_cache.insert(file_path.to_string_lossy().to_string(), data);
        println!("Loaded {} data points from {}", data.len(), file_path.display());
        Ok(())
    }
    
    /// Generate comprehensive visualization report
    pub async fn generate_comprehensive_report(
        &self,
        data: &HashMap<String, Vec<serde_json::Value>>,
        output_dir: &Path,
        config: ChartConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(output_dir)?;
        
        // Generate various charts based on available data
        if let Some(memory_data) = data.get("memory") {
            let memory_points: Vec<MemoryDataPoint> = memory_data.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            
            if !memory_points.is_empty() {
                let memory_chart_path = output_dir.join("memory_usage.svg");
                let memory_config = ChartConfig {
                    title: "Memory Usage Over Time".to_string(),
                    x_label: "Time".to_string(),
                    y_label: "Memory (bytes)".to_string(),
                    show_legend: true,
                    ..config.clone()
                };
                self.generate_memory_usage_chart(&memory_points, &memory_chart_path, memory_config).await?;
            }
        }
        
        if let Some(cache_data) = data.get("cache") {
            let cache_points: Vec<CacheDataPoint> = cache_data.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            
            if !cache_points.is_empty() {
                let cache_chart_path = output_dir.join("cache_performance.svg");
                let cache_config = ChartConfig {
                    title: "Cache Performance".to_string(),
                    x_label: "Time".to_string(),
                    y_label: "Hit Ratio".to_string(),
                    show_legend: true,
                    ..config.clone()
                };
                self.generate_cache_performance_chart(&cache_points, &cache_chart_path, cache_config).await?;
            }
        }
        
        if let Some(leak_data) = data.get("leaks") {
            let leak_points: Vec<LeakDataPoint> = leak_data.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            
            if !leak_points.is_empty() {
                let leak_chart_path = output_dir.join("leak_detection.svg");
                let leak_config = ChartConfig {
                    title: "Memory Leak Analysis".to_string(),
                    x_label: "Allocation Size".to_string(),
                    y_label: "Suspicion Score".to_string(),
                    show_legend: true,
                    ..config.clone()
                };
                self.generate_leak_detection_chart(&leak_points, &leak_chart_path, leak_config).await?;
            }
        }
        
        if let Some(numa_data) = data.get("numa") {
            let numa_points: Vec<NUMADataPoint> = numa_data.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            
            if !numa_points.is_empty() {
                let numa_chart_path = output_dir.join("numa_topology.svg");
                let numa_config = ChartConfig {
                    title: "NUMA Topology".to_string(),
                    x_label: "".to_string(),
                    y_label: "".to_string(),
                    show_legend: false,
                    ..config.clone()
                };
                self.generate_numa_topology_chart(&numa_points, &numa_chart_path, numa_config).await?;
            }
        }
        
        // Generate HTML report index
        self.generate_html_index(output_dir, data).await?;
        
        println!("Comprehensive report generated in: {}", output_dir.display());
        Ok(())
    }
    
    /// Generate HTML index for visualizations
    async fn generate_html_index(
        &self,
        output_dir: &Path,
        data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Memory Profiling Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .chart {{ margin: 20px 0; border: 1px solid #ccc; padding: 10px; }}
        .chart img {{ max-width: 100%; }}
        h1 {{ color: #333; }}
        h2 {{ color: #666; }}
        .summary {{ background: #f5f5f5; padding: 20px; margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>Memory Profiling Report</h1>
    
    <div class="summary">
        <h2>Summary</h2>
        <p>Generated on: {}</p>
        <p>Data points: {}</p>
        <p>Charts generated: {}</p>
    </div>
    
    <h2>Visualizations</h2>
    
    {}
    
</body>
</html>
        "#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            data.values().map(|v| v.len()).sum::<usize>(),
            data.len(),
            self.generate_chart_html_links(output_dir, data)?
        );
        
        let html_path = output_dir.join("index.html");
        std::fs::write(&html_path, html_content)?;
        
        println!("HTML report generated: {}", html_path.display());
        Ok(())
    }
    
    /// Generate HTML links for charts
    fn generate_chart_html_links(
        &self,
        output_dir: &Path,
        data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut links = String::new();
        
        // Memory usage chart
        if data.contains_key("memory") && output_dir.join("memory_usage.svg").exists() {
            links.push_str(r#"<div class="chart"><h3>Memory Usage</h3><img src="memory_usage.svg" alt="Memory Usage"></div>"#);
        }
        
        // Cache performance chart
        if data.contains_key("cache") && output_dir.join("cache_performance.svg").exists() {
            links.push_str(r#"<div class="chart"><h3>Cache Performance</h3><img src="cache_performance.svg" alt="Cache Performance"></div>"#);
        }
        
        // Leak detection chart
        if data.contains_key("leaks") && output_dir.join("leak_detection.svg").exists() {
            links.push_str(r#"<div class="chart"><h3>Leak Detection</h3><img src="leak_detection.svg" alt="Leak Detection"></div>"#);
        }
        
        // NUMA topology chart
        if data.contains_key("numa") && output_dir.join("numa_topology.svg").exists() {
            links.push_str(r#"<div class="chart"><h3>NUMA Topology</h3><img src="numa_topology.svg" alt="NUMA Topology"></div>"#);
        }
        
        Ok(links)
    }
}