//! Visualization Module
//! 
//! Performance visualization and reporting system including:
//! - Real-time performance charts
//! - Comparative analysis visualizations
//! - Scalability analysis charts
//! - HTML dashboard generation
//! - Statistical reporting

use crate::*;
use std::fs;
use std::path::Path;
use plotters::prelude::*;
use plotters::chart::SeriesChartPosition;

/// Visualization engine for performance data
pub struct VisualizationEngine {
    output_directory: String,
    /// Chart generation settings
    chart_settings: ChartSettings,
    /// Template engine for reports
    template_engine: ReportTemplateEngine,
}

/// Chart generation settings
#[derive(Debug, Clone)]
pub struct ChartSettings {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub color_scheme: ColorScheme,
    pub font_family: String,
}

/// Color schemes for charts
#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    Dark,
    Pastel,
    HighContrast,
    ColorBlindFriendly,
}

/// Report template engine
pub struct ReportTemplateEngine {
    /// HTML templates
    html_templates: HashMap<String, String>,
    /// CSS styles
    css_styles: HashMap<String, String>,
    /// JavaScript for interactivity
    javascript_scripts: HashMap<String, String>,
}

impl VisualizationEngine {
    /// Create new visualization engine
    pub fn new(output_directory: String) -> Self {
        Self {
            output_directory,
            chart_settings: ChartSettings::default(),
            template_engine: ReportTemplateEngine::new(),
        }
    }

    /// Generate comprehensive performance charts
    pub fn generate_performance_charts(
        &self,
        samples: &[PerformanceSample],
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if samples.is_empty() {
            return Err("No performance data to visualize".into());
        }

        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_directory)?;

        // Generate individual charts
        self.generate_cpu_utilization_chart(samples, "cpu_utilization.png")?;
        self.generate_scheduling_latency_chart(samples, "scheduling_latency.png")?;
        self.generate_throughput_chart(samples, "throughput.png")?;
        self.generate_fairness_chart(samples, "fairness.png")?;
        self.generate_context_switch_chart(samples, "context_switches.png")?;
        self.generate_load_balancing_chart(samples, "load_balancing.png")?;

        // Generate combined dashboard
        self.generate_dashboard_chart(samples, output_path)?;

        Ok(())
    }

    /// Generate CPU utilization chart
    fn generate_cpu_utilization_chart(
        &self,
        samples: &[PerformanceSample],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = format!("{}/{}", self.output_directory, filename);
        let root = BitMapBackend::new(&output_path, (self.chart_settings.width, self.chart_settings.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .caption("CPU Utilization Over Time", ("Arial", 20))
            .build_cartesian_2d(0.0..samples.len() as f32, 0.0..1.0)?;

        chart.configure_mesh()
            .light_line_style(&TRANSPARENT)
            .draw()?;

        // Calculate average CPU utilization across all cores for each sample
        let data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| {
                let avg_util = sample.cpu_utilization.iter().sum::<f32>() / sample.cpu_utilization.len() as f32;
                (i as f32, avg_util)
            })
            .collect();

        chart.draw_series(LineSeries::new(data, &BLUE))?
            .label("Average CPU Utilization")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .draw()?;

        // Add reference lines for optimal utilization ranges
        let optimal_range = Rectangle::new((0, 0.7), (samples.len() as f32, 0.9));
        root.draw(&optimal_range.fill(&RGBColor(0, 255, 0).alpha(0.1)))?;
        
        let warning_range = Rectangle::new((0, 0.9), (samples.len() as f32, 1.0));
        root.draw(&warning_range.fill(&RGBColor(255, 255, 0).alpha(0.1)))?;

        Ok(())
    }

    /// Generate scheduling latency chart
    fn generate_scheduling_latency_chart(
        &self,
        samples: &[PerformanceSample],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = format!("{}/{}", self.output_directory, filename);
        let root = BitMapBackend::new(&output_path, (self.chart_settings.width, self.chart_settings.height)).into_drawing_area();
        root.fill(&WHITE)?;

        // Determine y-axis range
        let max_latency = samples.iter()
            .map(|s| s.scheduling_latency.max_ns)
            .max()
            .unwrap_or(1000000) as f32;
        let y_max = (max_latency / 100000.0).ceil() * 100000.0; // Round up to nearest 100µs

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .caption("Scheduling Latency Over Time", ("Arial", 20))
            .build_cartesian_2d(0.0..samples.len() as f32, 0.0..y_max)?;

        chart.configure_mesh()
            .light_line_style(&TRANSPARENT)
            .y_label_formatter(|y| format!("{:.0}µs", y / 1000.0))
            .draw()?;

        // Plot different latency percentiles
        let avg_data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.scheduling_latency.avg_ns as f32 / 1000.0))
            .collect();
        
        let p95_data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.scheduling_latency.p95_ns as f32 / 1000.0))
            .collect();
        
        let p99_data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.scheduling_latency.p99_ns as f32 / 1000.0))
            .collect();

        chart.draw_series(LineSeries::new(avg_data, &GREEN))?
            .label("Average Latency")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));

        chart.draw_series(LineSeries::new(p95_data, &ORANGE))?
            .label("P95 Latency")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &ORANGE));

        chart.draw_series(LineSeries::new(p99_data, &RED))?
            .label("P99 Latency")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .draw()?;

        // Add latency threshold lines
        let target_latency = 50000.0; // 50µs target
        let warning_latency = 100000.0; // 100µs warning
        
        root.draw(&Circle::new((samples.len() as f32 / 2.0, target_latency), 3, &GREEN.filled()))?;
        root.draw(&Circle::new((samples.len() as f32 / 2.0, warning_latency), 3, &RED.filled()))?;

        Ok(())
    }

    /// Generate throughput chart
    fn generate_throughput_chart(
        &self,
        samples: &[PerformanceSample],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = format!("{}/{}", self.output_directory, filename);
        let root = BitMapBackend::new(&output_path, (self.chart_settings.width, self.chart_settings.height)).into_drawing_area();
        root.fill(&WHITE)?;

        // Determine y-axis range
        let max_throughput = samples.iter()
            .map(|s| s.throughput)
            .fold(0.0f64, f64::max);
        let y_max = (max_throughput / 1000.0).ceil() * 1000.0;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .caption("Throughput Over Time", ("Arial", 20))
            .build_cartesian_2d(0.0..samples.len() as f32, 0.0..y_max)?;

        chart.configure_mesh()
            .light_line_style(&TRANSPARENT)
            .y_label_formatter(|y| format!("{:.0}", y))
            .draw()?;

        let data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.throughput as f32))
            .collect();

        chart.draw_series(LineSeries::new(data, &BLUE))?
            .label("Tasks per Second")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        // Add moving average
        if data.len() >= 10 {
            let window_size = 10;
            let moving_avg: Vec<(f32, f32)> = data.windows(window_size)
                .enumerate()
                .map(|(i, window)| {
                    let avg = window.iter().map(|(_, y)| *y).sum::<f32>() / window_size as f32;
                    ((i + window_size / 2) as f32, avg)
                })
                .collect();

            chart.draw_series(LineSeries::new(moving_avg, &RED))?
                .label("Moving Average (10 samples)")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
        }

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .draw()?;

        Ok(())
    }

    /// Generate fairness index chart
    fn generate_fairness_chart(
        &self,
        samples: &[PerformanceSample],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = format!("{}/{}", self.output_directory, filename);
        let root = BitMapBackend::new(&output_path, (self.chart_settings.width, self.chart_settings.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .caption("Fairness Index Over Time", ("Arial", 20))
            .build_cartesian_2d(0.0..samples.len() as f32, 0.0..1.0)?;

        chart.configure_mesh()
            .light_line_style(&TRANSPARENT)
            .draw()?;

        let data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.fairness_index))
            .collect();

        chart.draw_series(LineSeries::new(data, &PURPLE))?
            .label("Jain's Fairness Index")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &PURPLE));

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .draw()?;

        // Add fairness threshold line
        let fairness_threshold = 0.8;
        root.draw(&Circle::new((samples.len() as f32 / 2.0, fairness_threshold), 3, &GREEN.filled()))?;

        Ok(())
    }

    /// Generate context switch overhead chart
    fn generate_context_switch_chart(
        &self,
        samples: &[PerformanceSample],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = format!("{}/{}", self.output_directory, filename);
        let root = BitMapBackend::new(&output_path, (self.chart_settings.width, self.chart_settings.height)).into_drawing_area();
        root.fill(&WHITE)?;

        // Determine y-axis range
        let max_overhead = samples.iter()
            .map(|s| s.context_switch_overhead.max_microseconds)
            .max()
            .unwrap_or(100);
        let y_max = (max_overhead as f32 / 10.0).ceil() * 10.0;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .caption("Context Switch Overhead", ("Arial", 20))
            .build_cartesian_2d(0.0..samples.len() as f32, 0.0..y_max)?;

        chart.configure_mesh()
            .light_line_style(&TRANSPARENT)
            .y_label_formatter(|y| format!("{:.1}µs", y))
            .draw()?;

        let avg_data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.context_switch_overhead.avg_microseconds))
            .collect();
        
        let max_data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.context_switch_overhead.max_microseconds))
            .collect();

        chart.draw_series(LineSeries::new(avg_data, &CYAN))?
            .label("Average Overhead")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &CYAN));

        chart.draw_series(LineSeries::new(max_data, &MAGENTA))?
            .label("Maximum Overhead")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &MAGENTA));

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .draw()?;

        Ok(())
    }

    /// Generate load balancing efficiency chart
    fn generate_load_balancing_chart(
        &self,
        samples: &[PerformanceSample],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = format!("{}/{}", self.output_directory, filename);
        let root = BitMapBackend::new(&output_path, (self.chart_settings.width, self.chart_settings.height)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .caption("Load Balancing Efficiency", ("Arial", 20))
            .build_cartesian_2d(0.0..samples.len() as f32, 0.0..1.0)?;

        chart.configure_mesh()
            .light_line_style(&TRANSPARENT)
            .draw()?;

        let data: Vec<(f32, f32)> = samples.iter().enumerate()
            .map(|(i, sample)| (i as f32, sample.load_balancing_efficiency))
            .collect();

        chart.draw_series(LineSeries::new(data, &BROWN))?
            .label("Load Balancing Efficiency")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BROWN));

        chart.configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE)
            .draw()?;

        // Add efficiency zones
        let good_efficiency = Rectangle::new((0, 0.8), (samples.len() as f32, 1.0));
        root.draw(&good_efficiency.fill(&RGBColor(0, 255, 0).alpha(0.1)))?;
        
        let poor_efficiency = Rectangle::new((0, 0.0), (samples.len() as f32, 0.6));
        root.draw(&poor_efficiency.fill(&RGBColor(255, 0, 0).alpha(0.1)))?;

        Ok(())
    }

    /// Generate comprehensive dashboard chart
    fn generate_dashboard_chart(
        &self,
        samples: &[PerformanceSample],
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(output_path, (1600, 1200)).into_drawing_area();
        root.fill(&WHITE)?;

        // Create a 3x2 grid of charts
        let areas = root.split_areas((
            (0, 0),
            (1600, 1200),
            vec![
                (0, 0, 799, 399),   // Top left
                (800, 0, 1599, 399), // Top right
                (0, 400, 799, 799), // Middle left
                (800, 400, 1599, 799), // Middle right
                (0, 800, 799, 1199), // Bottom left
                (800, 800, 1599, 1199), // Bottom right
            ],
        ))?;

        // CPU Utilization (top left)
        {
            let mut chart = ChartBuilder::on(&areas[0])
                .margin(10)
                .caption("CPU Utilization", ("Arial", 14))
                .build_cartesian_2d(0.0..samples.len() as f32, 0.0..1.0)?;

            chart.configure_mesh().light_line_style(&TRANSPARENT).draw()?;
            
            let data: Vec<(f32, f32)> = samples.iter().enumerate()
                .map(|(i, sample)| {
                    let avg_util = sample.cpu_utilization.iter().sum::<f32>() / sample.cpu_utilization.len() as f32;
                    (i as f32, avg_util)
                })
                .collect();

            chart.draw_series(LineSeries::new(data, &BLUE))?;
        }

        // Scheduling Latency (top right)
        {
            let mut chart = ChartBuilder::on(&areas[1])
                .margin(10)
                .caption("Scheduling Latency", ("Arial", 14))
                .build_cartesian_2d(0.0..samples.len() as f32, 0.0..100000.0)?;

            chart.configure_mesh().light_line_style(&TRANSPARENT).draw()?;
            
            let data: Vec<(f32, f32)> = samples.iter().enumerate()
                .map(|(i, sample)| (i as f32, sample.scheduling_latency.avg_ns as f32))
                .collect();

            chart.draw_series(LineSeries::new(data, &GREEN))?;
        }

        // Throughput (middle left)
        {
            let mut chart = ChartBuilder::on(&areas[2])
                .margin(10)
                .caption("Throughput", ("Arial", 14))
                .build_cartesian_2d(0.0..samples.len() as f32, 0.0..10000.0)?;

            chart.configure_mesh().light_line_style(&TRANSPARENT).draw()?;
            
            let data: Vec<(f32, f32)> = samples.iter().enumerate()
                .map(|(i, sample)| (i as f32, sample.throughput as f32))
                .collect();

            chart.draw_series(LineSeries::new(data, &RED))?;
        }

        // Fairness (middle right)
        {
            let mut chart = ChartBuilder::on(&areas[3])
                .margin(10)
                .caption("Fairness Index", ("Arial", 14))
                .build_cartesian_2d(0.0..samples.len() as f32, 0.0..1.0)?;

            chart.configure_mesh().light_line_style(&TRANSPARENT).draw()?;
            
            let data: Vec<(f32, f32)> = samples.iter().enumerate()
                .map(|(i, sample)| (i as f32, sample.fairness_index))
                .collect();

            chart.draw_series(LineSeries::new(data, &PURPLE))?;
        }

        // Context Switch Overhead (bottom left)
        {
            let mut chart = ChartBuilder::on(&areas[4])
                .margin(10)
                .caption("Context Switch Overhead", ("Arial", 14))
                .build_cartesian_2d(0.0..samples.len() as f32, 0.0..50.0)?;

            chart.configure_mesh().light_line_style(&TRANSPARENT).draw()?;
            
            let data: Vec<(f32, f32)> = samples.iter().enumerate()
                .map(|(i, sample)| (i as f32, sample.context_switch_overhead.avg_microseconds))
                .collect();

            chart.draw_series(LineSeries::new(data, &ORANGE))?;
        }

        // Load Balancing (bottom right)
        {
            let mut chart = ChartBuilder::on(&areas[5])
                .margin(10)
                .caption("Load Balancing", ("Arial", 14))
                .build_cartesian_2d(0.0..samples.len() as f32, 0.0..1.0)?;

            chart.configure_mesh().light_line_style(&TRANSPARENT).draw()?;
            
            let data: Vec<(f32, f32)> = samples.iter().enumerate()
                .map(|(i, sample)| (i as f32, sample.load_balancing_efficiency))
                .collect();

            chart.draw_series(LineSeries::new(data, &BROWN))?;
        }

        Ok(())
    }

    /// Convert samples to CSV format
    pub fn samples_to_csv(&self, samples: &[PerformanceSample]) -> Result<String, Box<dyn std::error::Error>> {
        let mut csv = String::new();
        
        // CSV header
        csv.push_str("timestamp,cpu_util_avg,scheduling_latency_avg_ns,throughput,fairness_index,responsiveness_score,load_balancing_efficiency\n");
        
        // Data rows
        for sample in samples {
            let avg_cpu_util = sample.cpu_utilization.iter().sum::<f32>() / sample.cpu_utilization.len() as f32;
            csv.push_str(&format!(
                "{},{:.3},{},{:.0},{:.3},{:.3},{:.3}\n",
                sample.timestamp,
                avg_cpu_util,
                sample.scheduling_latency.avg_ns,
                sample.throughput,
                sample.fairness_index,
                sample.responsiveness_score,
                sample.load_balancing_efficiency
            ));
        }
        
        Ok(csv)
    }

    /// Generate HTML report with embedded charts
    pub fn generate_html_report(
        &self,
        samples: &[PerformanceSample],
        report_title: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let html_content = self.template_engine.generate_html_report(samples, report_title)?;
        fs::write(output_path, html_content)?;
        
        Ok(())
    }

    /// Generate real-time dashboard HTML
    pub fn generate_realtime_dashboard(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let dashboard_html = self.template_engine.generate_realtime_dashboard()?;
        fs::write(output_path, dashboard_html)?;
        
        Ok(())
    }
}

impl Default for ChartSettings {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            dpi: 100,
            color_scheme: ColorScheme::Default,
            font_family: "Arial".to_string(),
        }
    }
}

impl ReportTemplateEngine {
    fn new() -> Self {
        Self {
            html_templates: Self::initialize_html_templates(),
            css_styles: Self::initialize_css_styles(),
            javascript_scripts: Self::initialize_javascript_scripts(),
        }
    }

    fn initialize_html_templates() -> HashMap<String, String> {
        let mut templates = HashMap::new();
        
        templates.insert("performance_report".to_string(), r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
    <style>
        {{ css_styles }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{{ title }}</h1>
        <div class="timestamp">Generated: {{ timestamp }}</div>
        
        <div class="summary">
            <h2>Performance Summary</h2>
            <div class="metrics-grid">
                <div class="metric">
                    <div class="metric-value">{{ avg_cpu_utilization }}%</div>
                    <div class="metric-label">Average CPU Utilization</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ avg_scheduling_latency }}µs</div>
                    <div class="metric-label">Average Scheduling Latency</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ avg_throughput }}</div>
                    <div class="metric-label">Average Throughput (tasks/sec)</div>
                </div>
                <div class="metric">
                    <div class="metric-value">{{ avg_fairness }}</div>
                    <div class="metric-label">Average Fairness Index</div>
                </div>
            </div>
        </div>
        
        <div class="charts">
            <h2>Performance Charts</h2>
            <div class="chart-grid">
                <div class="chart-container">
                    <img src="cpu_utilization.png" alt="CPU Utilization">
                    <h3>CPU Utilization Over Time</h3>
                </div>
                <div class="chart-container">
                    <img src="scheduling_latency.png" alt="Scheduling Latency">
                    <h3>Scheduling Latency Over Time</h3>
                </div>
                <div class="chart-container">
                    <img src="throughput.png" alt="Throughput">
                    <h3>Throughput Over Time</h3>
                </div>
                <div class="chart-container">
                    <img src="fairness.png" alt="Fairness">
                    <h3>Fairness Index Over Time</h3>
                </div>
                <div class="chart-container">
                    <img src="context_switches.png" alt="Context Switches">
                    <h3>Context Switch Overhead</h3>
                </div>
                <div class="chart-container">
                    <img src="load_balancing.png" alt="Load Balancing">
                    <h3>Load Balancing Efficiency</h3>
                </div>
            </div>
        </div>
        
        <div class="analysis">
            <h2>Analysis</h2>
            <div class="analysis-content">
                {{ analysis_content }}
            </div>
        </div>
        
        <div class="footer">
            <p>Generated by MultiOS Scheduler Profiler</p>
        </div>
    </div>
    
    <script>
        {{ javascript_scripts }}
    </script>
</body>
</html>
        "#.to_string());
        
        templates
    }

    fn initialize_css_styles() -> HashMap<String, String> {
        let mut styles = HashMap::new();
        
        styles.insert("default".to_string(), r#"
            * {
                margin: 0;
                padding: 0;
                box-sizing: border-box;
            }
            
            body {
                font-family: Arial, sans-serif;
                line-height: 1.6;
                color: #333;
                background-color: #f4f4f4;
            }
            
            .container {
                max-width: 1200px;
                margin: 0 auto;
                padding: 20px;
                background: white;
                box-shadow: 0 0 10px rgba(0,0,0,0.1);
            }
            
            h1 {
                color: #2c3e50;
                border-bottom: 3px solid #3498db;
                padding-bottom: 10px;
                margin-bottom: 30px;
            }
            
            h2 {
                color: #34495e;
                margin: 30px 0 15px 0;
                border-left: 4px solid #3498db;
                padding-left: 15px;
            }
            
            .timestamp {
                color: #7f8c8d;
                font-style: italic;
                margin-bottom: 30px;
            }
            
            .summary {
                background: #ecf0f1;
                padding: 20px;
                border-radius: 8px;
                margin-bottom: 30px;
            }
            
            .metrics-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 20px;
                margin-top: 15px;
            }
            
            .metric {
                background: white;
                padding: 20px;
                border-radius: 8px;
                text-align: center;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
            
            .metric-value {
                font-size: 2em;
                font-weight: bold;
                color: #2c3e50;
                margin-bottom: 5px;
            }
            
            .metric-label {
                color: #7f8c8d;
                font-size: 0.9em;
            }
            
            .charts {
                margin-bottom: 30px;
            }
            
            .chart-grid {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                gap: 20px;
            }
            
            .chart-container {
                background: white;
                padding: 15px;
                border-radius: 8px;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
            
            .chart-container img {
                width: 100%;
                height: auto;
                border-radius: 4px;
            }
            
            .chart-container h3 {
                margin-top: 10px;
                color: #34495e;
                font-size: 1.1em;
            }
            
            .analysis {
                background: #f8f9fa;
                padding: 20px;
                border-radius: 8px;
            }
            
            .footer {
                text-align: center;
                margin-top: 40px;
                padding-top: 20px;
                border-top: 1px solid #ddd;
                color: #7f8c8d;
            }
            
            /* Responsive design */
            @media (max-width: 768px) {
                .container {
                    padding: 10px;
                }
                
                .metrics-grid {
                    grid-template-columns: 1fr;
                }
                
                .chart-grid {
                    grid-template-columns: 1fr;
                }
            }
        "#.to_string());
        
        styles
    }

    fn initialize_javascript_scripts() -> HashMap<String, String> {
        let mut scripts = HashMap::new();
        
        scripts.insert("realtime_dashboard".to_string(), r#"
            // Real-time dashboard functionality
            let ws;
            let isConnected = false;
            
            function connectWebSocket() {
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                const wsUrl = `${protocol}//${window.location.host}/ws`;
                
                ws = new WebSocket(wsUrl);
                
                ws.onopen = function(event) {
                    isConnected = true;
                    console.log('Connected to performance monitor');
                    updateConnectionStatus(true);
                };
                
                ws.onmessage = function(event) {
                    const data = JSON.parse(event.data);
                    updateMetrics(data);
                };
                
                ws.onclose = function(event) {
                    isConnected = false;
                    updateConnectionStatus(false);
                    // Reconnect after delay
                    setTimeout(connectWebSocket, 5000);
                };
                
                ws.onerror = function(error) {
                    console.error('WebSocket error:', error);
                    updateConnectionStatus(false);
                };
            }
            
            function updateConnectionStatus(connected) {
                const statusElement = document.getElementById('connection-status');
                if (statusElement) {
                    statusElement.textContent = connected ? 'Connected' : 'Disconnected';
                    statusElement.className = connected ? 'status-connected' : 'status-disconnected';
                }
            }
            
            function updateMetrics(data) {
                // Update CPU utilization
                const cpuUtil = document.getElementById('cpu-utilization');
                if (cpuUtil && data.cpu_utilization !== undefined) {
                    cpuUtil.textContent = (data.cpu_utilization * 100).toFixed(1) + '%';
                }
                
                // Update scheduling latency
                const schedLatency = document.getElementById('scheduling-latency');
                if (schedLatency && data.scheduling_latency !== undefined) {
                    schedLatency.textContent = (data.scheduling_latency / 1000).toFixed(1) + 'µs';
                }
                
                // Update throughput
                const throughput = document.getElementById('throughput');
                if (throughput && data.throughput !== undefined) {
                    throughput.textContent = Math.round(data.throughput).toLocaleString();
                }
                
                // Update fairness
                const fairness = document.getElementById('fairness');
                if (fairness && data.fairness !== undefined) {
                    fairness.textContent = data.fairness.toFixed(3);
                }
                
                // Update charts if chart library is available
                updateCharts(data);
            }
            
            function updateCharts(data) {
                // This would integrate with a charting library like Chart.js
                // For now, just log the data
                console.log('Updating charts with:', data);
            }
            
            // Initialize dashboard when page loads
            document.addEventListener('DOMContentLoaded', function() {
                connectWebSocket();
                
                // Add some interactive features
                const toggleButtons = document.querySelectorAll('.toggle-metric');
                toggleButtons.forEach(button => {
                    button.addEventListener('click', function() {
                        const metricId = this.getAttribute('data-metric');
                        const metric = document.getElementById(metricId);
                        if (metric) {
                            metric.style.display = metric.style.display === 'none' ? 'block' : 'none';
                        }
                    });
                });
            });
            
            // Chart update functions (would use Chart.js or similar)
            function updateCPUChart(data) {
                // Implementation would update CPU utilization chart
            }
            
            function updateLatencyChart(data) {
                // Implementation would update latency chart
            }
            
            function updateThroughputChart(data) {
                // Implementation would update throughput chart
            }
        "#.to_string());
        
        scripts
    }

    /// Generate HTML report from template
    fn generate_html_report(&self, samples: &[PerformanceSample], title: &str) -> Result<String, Box<dyn std::error::Error>> {
        let template = self.html_templates.get("performance_report")
            .ok_or("Performance report template not found")?;
        
        // Calculate summary statistics
        let total_samples = samples.len();
        let avg_cpu_utilization = if !samples.is_empty() {
            samples.iter()
                .map(|s| s.cpu_utilization.iter().sum::<f32>() / s.cpu_utilization.len() as f32)
                .sum::<f32>() / total_samples as f32 * 100.0
        } else {
            0.0
        };
        
        let avg_scheduling_latency = if !samples.is_empty() {
            samples.iter()
                .map(|s| s.scheduling_latency.avg_ns)
                .sum::<f64>() / total_samples as f64 / 1000.0
        } else {
            0.0
        };
        
        let avg_throughput = if !samples.is_empty() {
            samples.iter().map(|s| s.throughput).sum::<f64>() / total_samples as f64
        } else {
            0.0
        };
        
        let avg_fairness = if !samples.is_empty() {
            samples.iter().map(|s| s.fairness_index).sum::<f32>() / total_samples as f32
        } else {
            0.0
        };
        
        // Generate analysis content
        let analysis_content = self.generate_analysis_content(samples);
        
        // Replace template variables
        let mut html = template.clone();
        html = html.replace("{{ title }}", title);
        html = html.replace("{{ timestamp }}", &chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
        html = html.replace("{{ avg_cpu_utilization }}", &format!("{:.1}", avg_cpu_utilization));
        html = html.replace("{{ avg_scheduling_latency }}", &format!("{:.1}", avg_scheduling_latency));
        html = html.replace("{{ avg_throughput }}", &format!("{:.0}", avg_throughput));
        html = html.replace("{{ avg_fairness }}", &format!("{:.3}", avg_fairness));
        html = html.replace("{{ analysis_content }}", &analysis_content);
        
        // Insert CSS and JavaScript
        let css_styles = self.css_styles.get("default").unwrap_or(&String::new());
        html = html.replace("{{ css_styles }}", css_styles);
        
        let javascript = self.javascript_scripts.get("realtime_dashboard").unwrap_or(&String::new());
        html = html.replace("{{ javascript_scripts }}", javascript);
        
        Ok(html)
    }

    /// Generate real-time dashboard HTML
    fn generate_realtime_dashboard(&self) -> Result<String, Box<dyn std::error::Error>> {
        let template = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MultiOS Scheduler Performance Dashboard</title>
    <style>
        {{ css_styles }}
        .dashboard-header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            text-align: center;
            margin-bottom: 30px;
        }
        .dashboard-header h1 {
            margin: 0;
            border: none;
        }
        .status-indicator {
            display: inline-block;
            padding: 5px 10px;
            border-radius: 15px;
            font-size: 0.9em;
            margin-left: 10px;
        }
        .status-connected {
            background-color: #27ae60;
        }
        .status-disconnected {
            background-color: #e74c3c;
        }
        .metrics-grid {
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        }
        .chart-container {
            min-height: 400px;
        }
        .chart-container canvas {
            max-height: 350px;
        }
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div class="dashboard-header">
        <h1>MultiOS Scheduler Performance Dashboard</h1>
        <div>
            <span>Connection Status:</span>
            <span id="connection-status" class="status-indicator status-disconnected">Disconnected</span>
        </div>
    </div>
    
    <div class="container">
        <div class="summary">
            <h2>Real-time Metrics</h2>
            <div class="metrics-grid">
                <div class="metric">
                    <div id="cpu-utilization" class="metric-value">0%</div>
                    <div class="metric-label">CPU Utilization</div>
                </div>
                <div class="metric">
                    <div id="scheduling-latency" class="metric-value">0µs</div>
                    <div class="metric-label">Scheduling Latency</div>
                </div>
                <div class="metric">
                    <div id="throughput" class="metric-value">0</div>
                    <div class="metric-label">Throughput (tasks/sec)</div>
                </div>
                <div class="metric">
                    <div id="fairness" class="metric-value">0</div>
                    <div class="metric-label">Fairness Index</div>
                </div>
            </div>
        </div>
        
        <div class="charts">
            <h2>Performance Charts</h2>
            <div class="chart-grid">
                <div class="chart-container">
                    <canvas id="cpuChart"></canvas>
                    <h3>CPU Utilization</h3>
                </div>
                <div class="chart-container">
                    <canvas id="latencyChart"></canvas>
                    <h3>Scheduling Latency</h3>
                </div>
                <div class="chart-container">
                    <canvas id="throughputChart"></canvas>
                    <h3>Throughput</h3>
                </div>
                <div class="chart-container">
                    <canvas id="fairnessChart"></canvas>
                    <h3>Fairness Index</h3>
                </div>
            </div>
        </div>
    </div>
    
    <script>
        {{ javascript_scripts }}
        
        // Chart.js initialization
        let charts = {};
        
        function initCharts() {
            const chartConfig = {
                type: 'line',
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        x: {
                            display: true,
                            title: {
                                display: true,
                                text: 'Time'
                            }
                        },
                        y: {
                            display: true,
                            title: {
                                display: true,
                                text: 'Value'
                            }
                        }
                    },
                    plugins: {
                        legend: {
                            display: false
                        }
                    }
                }
            };
            
            // Initialize CPU utilization chart
            charts.cpu = new Chart(document.getElementById('cpuChart'), {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        label: 'CPU Utilization',
                        data: [],
                        borderColor: 'rgb(75, 192, 192)',
                        backgroundColor: 'rgba(75, 192, 192, 0.2)',
                        tension: 0.1
                    }]
                },
                options: {
                    ...chartConfig.options,
                    scales: {
                        ...chartConfig.options.scales,
                        y: {
                            ...chartConfig.options.scales.y,
                            min: 0,
                            max: 100,
                            title: {
                                ...chartConfig.options.scales.y.title,
                                text: 'Percentage'
                            }
                        }
                    }
                }
            });
            
            // Initialize other charts similarly...
            charts.latency = new Chart(document.getElementById('latencyChart'), {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Scheduling Latency',
                        data: [],
                        borderColor: 'rgb(255, 99, 132)',
                        backgroundColor: 'rgba(255, 99, 132, 0.2)',
                        tension: 0.1
                    }]
                }
            });
            
            charts.throughput = new Chart(document.getElementById('throughputChart'), {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Throughput',
                        data: [],
                        borderColor: 'rgb(54, 162, 235)',
                        backgroundColor: 'rgba(54, 162, 235, 0.2)',
                        tension: 0.1
                    }]
                }
            });
            
            charts.fairness = new Chart(document.getElementById('fairnessChart'), {
                ...chartConfig,
                data: {
                    labels: [],
                    datasets: [{
                        label: 'Fairness Index',
                        data: [],
                        borderColor: 'rgb(153, 102, 255)',
                        backgroundColor: 'rgba(153, 102, 255, 0.2)',
                        tension: 0.1
                    }]
                },
                options: {
                    ...chartConfig.options,
                    scales: {
                        ...chartConfig.options.scales,
                        y: {
                            ...chartConfig.options.scales.y,
                            min: 0,
                            max: 1,
                            title: {
                                ...chartConfig.options.scales.y.title,
                                text: 'Index Value'
                            }
                        }
                    }
                }
            });
        }
        
        function updateCharts(data) {
            const now = new Date().toLocaleTimeString();
            
            // Update CPU chart
            if (charts.cpu && data.cpu_utilization !== undefined) {
                const cpuData = charts.cpu.data;
                cpuData.labels.push(now);
                cpuData.datasets[0].data.push(data.cpu_utilization * 100);
                
                // Keep only last 50 data points
                if (cpuData.labels.length > 50) {
                    cpuData.labels.shift();
                    cpuData.datasets[0].data.shift();
                }
                
                charts.cpu.update('none');
            }
            
            // Update other charts...
            if (charts.latency && data.scheduling_latency !== undefined) {
                const latencyData = charts.latency.data;
                latencyData.labels.push(now);
                latencyData.datasets[0].data.push(data.scheduling_latency / 1000);
                
                if (latencyData.labels.length > 50) {
                    latencyData.labels.shift();
                    latencyData.datasets[0].data.shift();
                }
                
                charts.latency.update('none');
            }
        }
        
        // Initialize charts when page loads
        document.addEventListener('DOMContentLoaded', function() {
            initCharts();
        });
    </script>
</body>
</html>
        "#.to_string();
        
        let css_styles = self.css_styles.get("default").unwrap_or(&String::new());
        let javascript = self.javascript_scripts.get("realtime_dashboard").unwrap_or(&String::new());
        
        let mut html = template;
        html = html.replace("{{ css_styles }}", css_styles);
        html = html.replace("{{ javascript_scripts }}", javascript);
        
        Ok(html)
    }

    /// Generate analysis content from samples
    fn generate_analysis_content(&self, samples: &[PerformanceSample]) -> String {
        if samples.is_empty() {
            return "No performance data available for analysis.".to_string();
        }

        let mut analysis = String::new();
        
        // Performance trends
        let avg_cpu_util = samples.iter()
            .map(|s| s.cpu_utilization.iter().sum::<f32>() / s.cpu_utilization.len() as f32)
            .sum::<f32>() / samples.len() as f32;
        
        let avg_latency = samples.iter()
            .map(|s| s.scheduling_latency.avg_ns)
            .sum::<f64>() / samples.len() as f64;
        
        let avg_fairness = samples.iter()
            .map(|s| s.fairness_index)
            .sum::<f32>() / samples.len() as f32;
        
        // Generate insights
        analysis.push_str("<h3>Key Insights</h3>");
        analysis.push_str("<ul>");
        
        if avg_cpu_util > 0.8 {
            analysis.push_str("<li><strong>High CPU Utilization:</strong> System is running at high capacity. Consider optimizing workload distribution.</li>");
        } else if avg_cpu_util < 0.3 {
            analysis.push_str("<li><strong>Low CPU Utilization:</strong> System has available capacity for additional workloads.</li>");
        } else {
            analysis.push_str("<li><strong>Optimal CPU Utilization:</strong> System is operating within efficient range.</li>");
        }
        
        if avg_latency > 100000.0 { // 100ms
            analysis.push_str("<li><strong>High Scheduling Latency:</strong> Consider reducing time quantum or optimizing scheduler algorithm.</li>");
        } else if avg_latency < 10000.0 { // 10ms
            analysis.push_str("<li><strong>Low Scheduling Latency:</strong> Excellent responsiveness achieved.</li>");
        }
        
        if avg_fairness < 0.7 {
            analysis.push_str("<li><strong>Fairness Concerns:</strong> Thread scheduling appears uneven. Enable aging or review priority settings.</li>");
        } else {
            analysis.push_str("<li><strong>Good Fairness:</strong> Thread scheduling demonstrates good balance.</li>");
        }
        
        analysis.push_str("</ul>");
        
        // Recommendations
        analysis.push_str("<h3>Recommendations</h3>");
        analysis.push_str("<ul>");
        
        if avg_cpu_util > 0.9 {
            analysis.push_str("<li>Consider adding more CPU cores for better scalability.</li>");
        }
        
        if avg_latency > 50000.0 {
            analysis.push_str("<li>Optimize context switch overhead by adjusting threshold parameters.</li>");
        }
        
        if avg_fairness < 0.8 {
            analysis.push_str("<li>Implement thread aging to prevent starvation of lower priority threads.</li>");
        }
        
        analysis.push_str("<li>Monitor performance trends and adjust scheduler parameters based on workload characteristics.</li>");
        analysis.push_str("</ul>");
        
        analysis
    }
}