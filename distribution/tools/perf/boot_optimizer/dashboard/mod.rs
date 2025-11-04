use std::time::{Duration, Instant};
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};
use std::thread;
use std::fs;
use std::path::Path;

use super::analysis::{BootAnalyzer, BootPerformanceAnalysis, OptimizationRecommendation};
use super::measurement::{BootMeasurement, BootMeasurementReport};

#[derive(Clone, Debug)]
pub struct DashboardConfig {
    pub update_interval: Duration,
    pub enable_real_time_updates: bool,
    pub save_reports: bool,
    pub report_directory: String,
    pub enable_alerts: bool,
    pub performance_thresholds: PerformanceThresholds,
}

#[derive(Clone, Debug)]
pub struct PerformanceThresholds {
    pub good_cold_boot: Duration,
    pub acceptable_cold_boot: Duration,
    pub good_warm_boot: Duration,
    pub acceptable_warm_boot: Duration,
    pub critical_variance: Duration,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            enable_real_time_updates: true,
            save_reports: true,
            report_directory: "/tmp/boot_reports".to_string(),
            enable_alerts: true,
            performance_thresholds: PerformanceThresholds {
                good_cold_boot: Duration::from_millis(1500),
                acceptable_cold_boot: Duration::from_millis(2000),
                good_warm_boot: Duration::from_millis(800),
                acceptable_warm_boot: Duration::from_millis(1000),
                critical_variance: Duration::from_millis(200),
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DashboardData {
    pub current_boot_time: Option<Duration>,
    pub boot_type: Option<String>,
    pub performance_score: f64,
    pub recent_boots: Vec<BootHistoryEntry>,
    pub optimization_status: OptimizationStatus,
    pub system_health: SystemHealthMetrics,
    pub recommendations: Vec<OptimizationRecommendation>,
}

#[derive(Clone, Debug)]
pub struct BootHistoryEntry {
    pub timestamp: Instant,
    pub boot_type: String,
    pub duration: Duration,
    pub optimizations_applied: Vec<String>,
    pub performance_rating: String,
}

#[derive(Clone, Debug)]
pub struct OptimizationStatus {
    pub enabled_optimizations: Vec<String>,
    pub recent_improvements: Vec<Duration>,
    pub success_rate: f64,
}

#[derive(Clone, Debug)]
pub struct SystemHealthMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_io: f32,
    pub network_activity: f32,
    pub temperature: Option<f32>,
    pub power_consumption: Option<f32>,
}

pub struct BootDashboard {
    config: DashboardConfig,
    data: Arc<RwLock<DashboardData>>,
    analyzer: Arc<BootAnalyzer>,
    measurement: Arc<BootMeasurement>,
    update_thread: Option<thread::JoinHandle<()>>,
    is_running: bool,
}

impl BootDashboard {
    pub fn new(analyzer: BootAnalyzer, measurement: BootMeasurement) -> Self {
        Self {
            config: DashboardConfig::default(),
            data: Arc::new(RwLock::new(DashboardData {
                current_boot_time: None,
                boot_type: None,
                performance_score: 0.0,
                recent_boots: Vec::new(),
                optimization_status: OptimizationStatus {
                    enabled_optimizations: Vec::new(),
                    recent_improvements: Vec::new(),
                    success_rate: 0.0,
                },
                system_health: SystemHealthMetrics {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    disk_io: 0.0,
                    network_activity: 0.0,
                    temperature: None,
                    power_consumption: None,
                },
                recommendations: Vec::new(),
            })),
            analyzer: Arc::new(analyzer),
            measurement: Arc::new(measurement),
            update_thread: None,
            is_running: false,
        }
    }

    pub fn new_with_config(analyzer: BootAnalyzer, measurement: BootMeasurement, config: DashboardConfig) -> Self {
        let mut dashboard = Self::new(analyzer, measurement);
        dashboard.config = config;
        dashboard
    }

    pub fn start(&mut self) {
        if self.is_running {
            return;
        }

        self.is_running = true;

        // Create report directory if it doesn't exist
        if self.config.save_reports && !Path::new(&self.config.report_directory).exists() {
            fs::create_dir_all(&self.config.report_directory).ok();
        }

        let data = self.data.clone();
        let analyzer = self.analyzer.clone();
        let config = self.config.clone();

        self.update_thread = Some(thread::spawn(move || {
            DashboardUpdateLoop::run(data, analyzer, config);
        }));
    }

    pub fn stop(&mut self) {
        self.is_running = false;
        if let Some(thread) = self.update_thread.take() {
            thread.join().ok();
        }
    }

    pub fn update_dashboard_data(&self) {
        let mut data = self.data.write().unwrap();
        
        // Update performance analysis
        let analysis = self.analyzer.analyze_boot_performance();
        data.performance_score = analysis.performance_score;
        data.recommendations = self.analyzer.generate_optimization_recommendations();
        
        // Update recent boots
        data.recent_boots = self.get_recent_boot_history();
        
        // Update optimization status
        data.optimization_status = self.calculate_optimization_status();
        
        // Update system health
        data.system_health = self.collect_system_health_metrics();
        
        // Check for alerts
        if self.config.enable_alerts {
            self.check_performance_alerts(&analysis, &mut data);
        }
    }

    fn get_recent_boot_history(&self) -> Vec<BootHistoryEntry> {
        // This would integrate with the actual boot measurement system
        // For now, we'll return sample data
        let now = Instant::now();
        vec![
            BootHistoryEntry {
                timestamp: now - Duration::from_secs(300),
                boot_type: "Cold Boot".to_string(),
                duration: Duration::from_millis(1800),
                optimizations_applied: vec!["parallel_init".to_string(), "module_opt".to_string()],
                performance_rating: "Good".to_string(),
            },
            BootHistoryEntry {
                timestamp: now - Duration::from_secs(1800),
                boot_type: "Warm Boot".to_string(),
                duration: Duration::from_millis(900),
                optimizations_applied: vec!["cache_optimization".to_string()],
                performance_rating: "Excellent".to_string(),
            },
        ]
    }

    fn calculate_optimization_status(&self) -> OptimizationStatus {
        let analysis = self.analyzer.analyze_boot_performance();
        
        OptimizationStatus {
            enabled_optimizations: vec![
                "Parallel Device Initialization".to_string(),
                "Kernel Module Optimization".to_string(),
                "Boot Splash Display".to_string(),
            ],
            recent_improvements: vec![
                Duration::from_millis(200),
                Duration::from_millis(150),
                Duration::from_millis(100),
            ],
            success_rate: 0.85,
        }
    }

    fn collect_system_health_metrics(&self) -> SystemHealthMetrics {
        // This would collect actual system metrics in a real implementation
        SystemHealthMetrics {
            cpu_usage: 15.0,
            memory_usage: 45.0,
            disk_io: 25.0,
            network_activity: 5.0,
            temperature: Some(45.0),
            power_consumption: Some(25.5),
        }
    }

    fn check_performance_alerts(&self, analysis: &BootPerformanceAnalysis, data: &mut DashboardData) {
        if let Some(cold_boot) = analysis.average_cold_boot {
            if cold_boot > self.config.performance_thresholds.acceptable_cold_boot {
                data.recommendations.insert(0, OptimizationRecommendation {
                    priority: super::analysis::Priority::High,
                    category: "Performance Alert".to_string(),
                    recommendation: "Cold boot time is above acceptable threshold!".to_string(),
                    expected_improvement: cold_boot - self.config.performance_thresholds.acceptable_cold_boot,
                    implementation_effort: "High".to_string(),
                    risk_level: "Low".to_string(),
                });
            }
        }

        if let Some(warm_boot) = analysis.average_warm_boot {
            if warm_boot > self.config.performance_thresholds.acceptable_warm_boot {
                data.recommendations.insert(0, OptimizationRecommendation {
                    priority: super::analysis::Priority::Medium,
                    category: "Performance Alert".to_string(),
                    recommendation: "Warm boot time is above acceptable threshold!".to_string(),
                    expected_improvement: warm_boot - self.config.performance_thresholds.acceptable_warm_boot,
                    implementation_effort: "Medium".to_string(),
                    risk_level: "Low".to_string(),
                });
            }
        }
    }

    pub fn generate_html_report(&self) -> String {
        let data = self.data.read().unwrap();
        
        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Boot Performance Dashboard</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; border-radius: 10px; padding: 30px; box-shadow: 0 10px 30px rgba(0,0,0,0.3); }}
        h1 {{ text-align: center; color: #333; margin-bottom: 30px; }}
        .metrics-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .metric-card {{ background: #f8f9fa; border-radius: 8px; padding: 20px; text-align: center; border-left: 4px solid #007bff; }}
        .metric-value {{ font-size: 2em; font-weight: bold; color: #007bff; }}
        .metric-label {{ color: #666; margin-top: 5px; }}
        .progress-bar {{ width: 100%; height: 10px; background: #e9ecef; border-radius: 5px; overflow: hidden; margin: 10px 0; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #28a745, #20c997); transition: width 0.3s ease; }}
        .performance-score {{ font-size: 3em; font-weight: bold; text-align: center; margin: 20px 0; }}
        .score-excellent {{ color: #28a745; }}
        .score-good {{ color: #17a2b8; }}
        .score-fair {{ color: #ffc107; }}
        .score-poor {{ color: #dc3545; }}
        .recommendations {{ background: #fff3cd; border: 1px solid #ffeaa7; border-radius: 8px; padding: 20px; margin-top: 20px; }}
        .boot-history {{ margin-top: 20px; }}
        .history-item {{ background: #f8f9fa; border-radius: 5px; padding: 15px; margin-bottom: 10px; display: flex; justify-content: space-between; align-items: center; }}
        .history-time {{ color: #666; }}
        .history-duration {{ font-weight: bold; }}
        .status-excellent {{ color: #28a745; }}
        .status-good {{ color: #17a2b8; }}
        .status-fair {{ color: #ffc107; }}
        .status-poor {{ color: #dc3545; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Boot Performance Dashboard</h1>
        
        <div class="performance-score {score_class}">
            {:.1}
        </div>
        
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-value">{:?}</div>
                <div class="metric-label">Current Boot Time</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}%</div>
                <div class="metric-label">CPU Usage</div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {:.1}%"></div>
                </div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}%</div>
                <div class="metric-label">Memory Usage</div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {:.1}%"></div>
                </div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Active Optimizations</div>
            </div>
        </div>
        
        <div class="boot-history">
            <h3>Recent Boot History</h3>
            {}
        </div>
        
        <div class="recommendations">
            <h3>💡 Optimization Recommendations</h3>
            {}
        </div>
    </div>
</body>
</html>
        "#,
        self.get_score_class(data.performance_score),
        data.performance_score,
        data.current_boot_time.map_or_else(|| "N/A".to_string(), |d| format!("{:.1}ms", d.as_millis() as f64 / 1000.0)),
        data.system_health.cpu_usage,
        data.system_health.cpu_usage,
        data.system_health.memory_usage,
        data.system_health.memory_usage,
        data.optimization_status.enabled_optimizations.len(),
        data.recent_boots.iter().map(|entry| {
            format!(r#"
                <div class="history-item">
                    <div>
                        <strong>{}</strong>
                        <div class="history-time">{} seconds ago</div>
                    </div>
                    <div class="history-duration {}">{}</div>
                </div>
            "#,
            entry.boot_type,
            (Instant::now().duration_since(entry.timestamp).as_secs() / 60),
            self.get_status_class(&entry.performance_rating),
            format!("{:.1}s", entry.duration.as_millis() as f64 / 1000.0))
        }).collect::<String>(),
        data.recommendations.iter().map(|rec| {
            format!(r#"
                <div style="margin-bottom: 10px; padding: 10px; background: white; border-radius: 5px;">
                    <strong>{}</strong> - {}
                    <div style="color: #666; font-size: 0.9em;">
                        Expected improvement: {:.1}s | Risk: {} | Effort: {}
                    </div>
                </div>
            "#,
            rec.category,
            rec.recommendation,
            rec.expected_improvement.as_millis() as f64 / 1000.0,
            rec.risk_level,
            rec.implementation_effort)
        }).collect::<String>()
        )
    }

    fn get_score_class(&self, score: f64) -> &'static str {
        if score >= 90.0 { "score-excellent" }
        else if score >= 75.0 { "score-good" }
        else if score >= 60.0 { "score-fair" }
        else { "score-poor" }
    }

    fn get_status_class(&self, rating: &str) -> &'static str {
        match rating.to_lowercase().as_str() {
            "excellent" => "status-excellent",
            "good" => "status-good", 
            "fair" => "status-fair",
            _ => "status-poor"
        }
    }

    pub fn generate_json_report(&self) -> String {
        let data = self.data.read().unwrap();
        
        let json = serde_json::json!({
            "performance_score": data.performance_score,
            "current_boot_time": data.current_boot_time.map(|d| d.as_millis()),
            "boot_type": data.boot_type,
            "system_health": {
                "cpu_usage": data.system_health.cpu_usage,
                "memory_usage": data.system_health.memory_usage,
                "disk_io": data.system_health.disk_io,
                "network_activity": data.system_health.network_activity,
                "temperature": data.system_health.temperature,
                "power_consumption": data.system_health.power_consumption
            },
            "recent_boots": data.recent_boots.iter().map(|entry| {
                serde_json::json!({
                    "timestamp": entry.timestamp.elapsed().as_secs(),
                    "boot_type": entry.boot_type,
                    "duration_ms": entry.duration.as_millis(),
                    "optimizations_applied": entry.optimizations_applied,
                    "performance_rating": entry.performance_rating
                })
            }).collect::<Vec<_>>(),
            "recommendations": data.recommendations.iter().map(|rec| {
                serde_json::json!({
                    "priority": format!("{:?}", rec.priority),
                    "category": rec.category,
                    "recommendation": rec.recommendation,
                    "expected_improvement_ms": rec.expected_improvement.as_millis(),
                    "implementation_effort": rec.implementation_effort,
                    "risk_level": rec.risk_level
                })
            }).collect::<Vec<_>>()
        });
        
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn save_report(&self, filename: &str) -> Result<(), String> {
        if !self.config.save_reports {
            return Err("Report saving is disabled".to_string());
        }

        let html_content = self.generate_html_report();
        let json_content = self.generate_json_report();
        
        let html_path = format!("{}/{}.html", self.config.report_directory, filename);
        let json_path = format!("{}/{}.json", self.config.report_directory, filename);
        
        fs::write(&html_path, html_content)
            .map_err(|e| format!("Failed to save HTML report: {}", e))?;
        
        fs::write(&json_path, json_content)
            .map_err(|e| format!("Failed to save JSON report: {}", e))?;
        
        println!("Reports saved to: {} and {}", html_path, json_path);
        Ok(())
    }

    pub fn get_current_data(&self) -> DashboardData {
        self.data.read().unwrap().clone()
    }
}

struct DashboardUpdateLoop;

impl DashboardUpdateLoop {
    fn run(
        data: Arc<RwLock<DashboardData>>,
        analyzer: Arc<BootAnalyzer>,
        config: DashboardConfig,
    ) {
        let mut last_update = Instant::now();
        
        loop {
            if !config.enable_real_time_updates {
                break;
            }
            
            let now = Instant::now();
            if now.duration_since(last_update) >= config.update_interval {
                // Update dashboard data
                let analysis = analyzer.analyze_boot_performance();
                let mut dashboard_data = data.write().unwrap();
                
                dashboard_data.performance_score = analysis.performance_score;
                dashboard_data.recommendations = analyzer.generate_optimization_recommendations();
                
                // Update system health (simplified)
                dashboard_data.system_health = SystemHealthMetrics {
                    cpu_usage: 10.0 + (now.elapsed().as_secs() as f32 % 50.0),
                    memory_usage: 45.0 + (now.elapsed().as_secs() as f32 % 10.0),
                    disk_io: 20.0 + (now.elapsed().as_secs() as f32 % 30.0),
                    network_activity: 5.0 + (now.elapsed().as_secs() as f32 % 15.0),
                    temperature: Some(40.0 + (now.elapsed().as_secs() as f32 % 20.0)),
                    power_consumption: Some(20.0 + (now.elapsed().as_secs() as f32 % 10.0)),
                };
                
                last_update = now;
            }
            
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for BootDashboard {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let analyzer = BootAnalyzer::new();
        let measurement = BootMeasurement::new(super::measurement::BootMeasurementConfig::default());
        let dashboard = BootDashboard::new(analyzer, measurement);
        
        assert!(!dashboard.is_running);
    }

    #[test]
    fn test_html_report_generation() {
        let analyzer = BootAnalyzer::new();
        let measurement = BootMeasurement::new(super::measurement::BootMeasurementConfig::default());
        let dashboard = BootDashboard::new(analyzer, measurement);
        
        let html = dashboard.generate_html_report();
        assert!(html.contains("<html"));
        assert!(html.contains("Boot Performance Dashboard"));
    }

    #[test]
    fn test_json_report_generation() {
        let analyzer = BootAnalyzer::new();
        let measurement = BootMeasurement::new(super::measurement::BootMeasurementConfig::default());
        let dashboard = BootDashboard::new(analyzer, measurement);
        
        let json = dashboard.generate_json_report();
        assert!(json.contains("\"performance_score\""));
        assert!(json.contains("\"system_health\""));
    }

    #[test]
    fn test_performance_thresholds() {
        let config = DashboardConfig::default();
        assert_eq!(config.performance_thresholds.good_cold_boot, Duration::from_millis(1500));
        assert_eq!(config.performance_thresholds.acceptable_cold_boot, Duration::from_millis(2000));
    }
}
