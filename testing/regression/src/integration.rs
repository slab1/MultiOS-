//! Integration Module
//!
//! Provides integration capabilities with existing benchmarking frameworks,
//! CI/CD systems, monitoring tools, and external testing platforms for
//! comprehensive regression testing ecosystem integration.

use anyhow::{Result};
use chrono::{DateTime, Utc};
use log::{info, debug, warn};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    PerformanceMeasurement, TestSuiteConfig, TestSuiteResult, CodeChange,
    BenchmarkResult, Uuid,
};

/// Benchmark system integrator for connecting with existing frameworks
#[derive(Debug, Clone)]
pub struct BenchmarkIntegrator {
    /// Configuration for benchmarking system integration
    config: Option<BenchmarkConfig>,
    /// HTTP client for API calls
    http_client: Client,
    /// Cached benchmark results
    cached_results: HashMap<String, Vec<BenchmarkResult>>,
    /// Integration status tracking
    status: IntegrationStatus,
}

/// Configuration for benchmarking system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub api_url: String,
    pub api_key: String,
    pub sync_interval_minutes: usize,
    pub enabled_endpoints: Vec<String>,
    pub authentication_type: AuthType,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AuthType {
    ApiKey,
    Bearer,
    Basic,
    OAuth2,
}

/// Integration status
#[derive(Debug, Clone)]
struct IntegrationStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub connection_status: ConnectionStatus,
    pub sync_errors: Vec<String>,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
    Unknown,
}

/// CI/CD system integrator
#[derive(Debug, Clone)]
pub struct CICDIntegrator {
    /// CI/CD system configuration
    config: Option<CICDConfig>,
    /// Git integration
    git_integration: GitIntegration,
    /// Pipeline status tracking
    pipeline_status: HashMap<String, PipelineStatus>,
}

/// CI/CD system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CICDConfig {
    pub system_type: CICDSystemType,
    pub base_url: String,
    pub auth_token: String,
    pub auto_trigger_regression_tests: bool,
    pub webhook_url: Option<String>,
}

/// Supported CI/CD systems
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CICDSystemType {
    GitHubActions,
    GitLabCI,
    Jenkins,
    CircleCI,
    AzureDevOps,
}

/// Pipeline execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PipelineStatus {
    pub pipeline_id: String,
    pub status: PipelineExecutionStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub regression_test_results: Option<TestSuiteResult>,
}

/// Pipeline execution states
#[derive(Debug, Clone, Serialize, Deserialize)]
enum PipelineExecutionStatus {
    Running,
    Success,
    Failed,
    Cancelled,
    Timeout,
}

/// Git integration for code analysis
#[derive(Debug, Clone)]
struct GitIntegration {
    /// Repository path
    repository_path: String,
    /// Branch tracking
    tracked_branches: Vec<String>,
}

/// Monitoring system integrator
#[derive(Debug, Clone)]
pub struct MonitoringIntegrator {
    /// Monitoring system configuration
    config: Option<MonitoringConfig>,
    /// Metrics collection
    metrics_collector: MetricsCollector,
    /// Alert management
    alert_manager: AlertManager,
}

/// Monitoring system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub system_type: MonitoringSystemType,
    pub api_url: String,
    pub api_key: String,
    pub metric_collection_interval: usize,
    pub alert_thresholds: AlertThresholds,
}

/// Supported monitoring systems
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MonitoringSystemType {
    Prometheus,
    Grafana,
    DataDog,
    NewRelic,
    CloudWatch,
}

/// Alert thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlertThresholds {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub response_time_ms: u64,
    pub error_rate_percent: f64,
}

/// Metrics collector
#[derive(Debug, Default)]
struct MetricsCollector {
    /// Collected metrics
    metrics: HashMap<String, MetricValue>,
    /// Last collection time
    last_collection: Option<DateTime<Utc>>,
}

/// Individual metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricValue {
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

/// Alert manager
#[derive(Debug, Clone)]
struct AlertManager {
    /// Active alerts
    active_alerts: HashMap<String, ActiveAlert>,
    /// Alert history
    alert_history: Vec<AlertRecord>,
}

/// Active alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveAlert {
    pub alert_id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged: bool,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AlertType {
    PerformanceRegression,
    HighCPUUsage,
    HighMemoryUsage,
    SlowResponse,
    HighErrorRate,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert record for history
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AlertRecord {
    pub alert_id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_time_seconds: Option<u64>,
}

/// Benchmark result from external system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub id: String,
    pub test_name: String,
    pub component: String,
    pub metric_type: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub environment: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BenchmarkIntegrator {
    /// Create new benchmark integrator
    pub fn new(config: Option<BenchmarkConfig>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        
        Self {
            config: config.clone(),
            http_client,
            cached_results: HashMap::new(),
            status: IntegrationStatus {
                last_sync: None,
                connection_status: ConnectionStatus::Unknown,
                sync_errors: Vec::new(),
            },
        }
    }

    /// Initialize connection to benchmarking system
    pub async fn initialize(&mut self) -> Result<()> {
        if let Some(config) = &self.config {
            info!("Initializing benchmark system integration");
            
            // Test connection
            match self.test_connection().await {
                Ok(_) => {
                    self.status.connection_status = ConnectionStatus::Connected;
                    info!("Successfully connected to benchmark system");
                }
                Err(e) => {
                    self.status.connection_status = ConnectionStatus::Error;
                    self.status.sync_errors.push(e.to_string());
                    warn!("Failed to connect to benchmark system: {}", e);
                }
            }
            
            // Start sync process if enabled
            if !config.enabled_endpoints.is_empty() {
                self.start_sync_process().await?;
            }
        }
        
        Ok(())
    }

    /// Test connection to benchmarking system
    pub async fn test_connection(&self) -> Result<()> {
        if let Some(config) = &self.config {
            let url = format!("{}/health", config.api_url);
            
            let response = self.http_client
                .get(&url)
                .header("Authorization", self.get_auth_header(config))
                .send()
                .await
                .context("Failed to send health check request")?;
            
            if response.status().is_success() {
                debug("Benchmark system health check successful");
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Health check failed with status: {}",
                    response.status()
                ))
            }
        } else {
            Err(anyhow::anyhow!("No benchmark configuration provided"))
        }
    }

    /// Get authorization header based on auth type
    fn get_auth_header(&self, config: &BenchmarkConfig) -> String {
        match config.authentication_type {
            AuthType::ApiKey => format!("Bearer {}", config.api_key),
            AuthType::Bearer => format!("Bearer {}", config.api_key),
            AuthType::Basic => {
                format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(
                    format!("{}:{}", config.api_key, "")
                ))
            }
            AuthType::OAuth2 => format!("Bearer {}", config.api_key),
        }
    }

    /// Start background sync process
    async fn start_sync_process(&mut self) -> Result<()> {
        if let Some(config) = &self.config {
            // This would start a background task for periodic sync
            info!("Starting benchmark data sync process");
            
            // For now, just log the intent
            // In a real implementation, this would start a tokio task
            debug!("Sync configured for endpoints: {:?}", config.enabled_endpoints);
        }
        
        Ok(())
    }

    /// Fetch benchmark results from external system
    pub async fn fetch_benchmark_results(
        &mut self,
        component: &str,
        time_range: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<Vec<BenchmarkResult>> {
        debug!("Fetching benchmark results for component: {}", component);
        
        let cache_key = format!("{}_{}_{}", component, time_range.0, time_range.1);
        
        // Check cache first
        if let Some(cached) = self.cached_results.get(&cache_key) {
            debug!("Returning cached benchmark results for {}", component);
            return Ok(cached.clone());
        }
        
        if let Some(config) = &self.config {
            // Fetch from external API
            let results = self.fetch_from_api(component, time_range).await?;
            
            // Cache results
            self.cached_results.insert(cache_key, results.clone());
            
            // Update sync status
            self.status.last_sync = Some(Utc::now());
            self.status.connection_status = ConnectionStatus::Connected;
            
            Ok(results)
        } else {
            warn!("No benchmark configuration available, returning empty results");
            Ok(Vec::new())
        }
    }

    /// Fetch benchmark data from external API
    async fn fetch_from_api(
        &self,
        component: &str,
        time_range: (DateTime<Utc>, DateTime<Utc>),
    ) -> Result<Vec<BenchmarkResult>> {
        if let Some(config) = &self.config {
            let url = format!(
                "{}/benchmarks?component={}&start_time={}&end_time={}",
                config.api_url,
                component,
                time_range.0.timestamp(),
                time_range.1.timestamp()
            );
            
            let response = self.http_client
                .get(&url)
                .header("Authorization", self.get_auth_header(config))
                .send()
                .await
                .context("Failed to fetch benchmark data")?;
            
            if response.status().is_success() {
                let benchmark_data: Vec<BenchmarkResult> = response
                    .json()
                    .await
                    .context("Failed to parse benchmark response")?;
                
                debug!("Fetched {} benchmark results for {}", benchmark_data.len(), component);
                Ok(benchmark_data)
            } else {
                Err(anyhow::anyhow!(
                    "API request failed with status: {}",
                    response.status()
                ))
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Sync benchmark results to external system
    pub async fn sync_benchmark_results(
        &mut self,
        results: &[BenchmarkResult],
    ) -> Result<()> {
        debug!("Syncing {} benchmark results to external system", results.len());
        
        if let Some(config) = &self.config {
            if config.enabled_endpoints.contains(&"write".to_string()) {
                self.upload_results(results).await?;
            }
        }
        
        Ok(())
    }

    /// Upload benchmark results to external system
    async fn upload_results(&self, results: &[BenchmarkResult]) -> Result<()> {
        if let Some(config) = &self.config {
            let url = format!("{}/benchmarks/batch", config.api_url);
            
            let response = self.http_client
                .post(&url)
                .header("Authorization", self.get_auth_header(config))
                .json(results)
                .send()
                .await
                .context("Failed to upload benchmark results")?;
            
            if response.status().is_success() {
                debug!("Successfully uploaded {} benchmark results", results.len());
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Upload failed with status: {}",
                    response.status()
                ))
            }
        } else {
            Ok(())
        }
    }

    /// Get integration status
    pub fn get_status(&self) -> &IntegrationStatus {
        &self.status
    }

    /// Clear cached results
    pub fn clear_cache(&mut self) {
        self.cached_results.clear();
        info!("Benchmark integration cache cleared");
    }
}

// ==========================================
// CI/CD INTEGRATION IMPLEMENTATION
// ==========================================

impl CICDIntegrator {
    /// Create new CI/CD integrator
    pub fn new(config: Option<CICDConfig>) -> Self {
        Self {
            config: config.clone(),
            git_integration: GitIntegration {
                repository_path: ".".to_string(),
                tracked_branches: vec!["main".to_string(), "develop".to_string()],
            },
            pipeline_status: HashMap::new(),
        }
    }

    /// Initialize CI/CD integration
    pub async fn initialize(&mut self) -> Result<()> {
        if let Some(config) = &self.config {
            info!("Initializing CI/CD integration for {:?}", config.system_type);
            
            // Initialize git integration
            self.git_integration.initialize().await?;
            
            // Test CI/CD system connection
            match self.test_connection().await {
                Ok(_) => info!("CI/CD integration initialized successfully"),
                Err(e) => warn!("CI/CD connection test failed: {}", e),
            }
        }
        
        Ok(())
    }

    /// Test connection to CI/CD system
    async fn test_connection(&self) -> Result<()> {
        if let Some(config) = &self.config {
            match config.system_type {
                CICDSystemType::GitHubActions => {
                    let url = format!("{}/repos/{}/actions/runs", config.base_url, "owner/repo");
                    self.test_api_connection(&url, &config.auth_token).await
                }
                CICDSystemType::GitLabCI => {
                    let url = format!("{}/api/v4/pipelines", config.base_url);
                    self.test_api_connection(&url, &config.auth_token).await
                }
                CICDSystemType::Jenkins => {
                    let url = format!("{}/api/json", config.base_url);
                    self.test_api_connection(&url, &config.auth_token).await
                }
                _ => {
                    // Generic test for other systems
                    let url = format!("{}/health", config.base_url);
                    self.test_api_connection(&url, &config.auth_token).await
                }
            }
        } else {
            Err(anyhow::anyhow!("No CI/CD configuration provided"))
        }
    }

    /// Test API connection generically
    async fn test_api_connection(&self, url: &str, token: &str) -> Result<()> {
        let response = self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .context("Failed to send test request")?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "API test failed with status: {}",
                response.status()
            ))
        }
    }

    /// Trigger regression tests in CI/CD pipeline
    pub async fn trigger_regression_tests(
        &mut self,
        code_changes: &[CodeChange],
        affected_components: &[String],
    ) -> Result<String> {
        info!("Triggering regression tests via CI/CD system");
        
        if let Some(config) = &self.config {
            match config.system_type {
                CICDSystemType::GitHubActions => {
                    self.trigger_github_actions(code_changes, affected_components).await
                }
                CICDSystemType::GitLabCI => {
                    self.trigger_gitlab_ci(code_changes, affected_components).await
                }
                CICDSystemType::Jenkins => {
                    self.trigger_jenkins(code_changes, affected_components).await
                }
                _ => {
                    Err(anyhow::anyhow!("CI/CD system not supported"))
                }
            }
        } else {
            Err(anyhow::anyhow!("No CI/CD configuration available"))
        }
    }

    /// Trigger GitHub Actions workflow
    async fn trigger_github_actions(
        &self,
        _code_changes: &[CodeChange],
        _affected_components: &[String],
    ) -> Result<String> {
        // This would trigger a GitHub Actions workflow
        // For now, return a mock pipeline ID
        let pipeline_id = format!("gh_run_{}", Uuid::new_v4());
        debug!("Triggered GitHub Actions workflow: {}", pipeline_id);
        Ok(pipeline_id)
    }

    /// Trigger GitLab CI pipeline
    async fn trigger_gitlab_ci(
        &self,
        _code_changes: &[CodeChange],
        _affected_components: &[String],
    ) -> Result<String> {
        // This would trigger a GitLab CI pipeline
        // For now, return a mock pipeline ID
        let pipeline_id = format!("gl_pipeline_{}", Uuid::new_v4());
        debug!("Triggered GitLab CI pipeline: {}", pipeline_id);
        Ok(pipeline_id)
    }

    /// Trigger Jenkins job
    async fn trigger_jenkins(
        &self,
        _code_changes: &[CodeChange],
        _affected_components: &[String],
    ) -> Result<String> {
        // This would trigger a Jenkins job
        // For now, return a mock pipeline ID
        let pipeline_id = format!("jenkins_build_{}", Uuid::new_v4());
        debug!("Triggered Jenkins build: {}", pipeline_id);
        Ok(pipeline_id)
    }

    /// Get pipeline status
    pub async fn get_pipeline_status(&self, pipeline_id: &str) -> Result<Option<PipelineStatus>> {
        Ok(self.pipeline_status.get(pipeline_id).cloned())
    }

    /// Process webhook from CI/CD system
    pub async fn process_webhook(
        &mut self,
        webhook_data: &serde_json::Value,
    ) -> Result<TestSuiteResult> {
        info!("Processing CI/CD webhook");
        
        // Parse webhook data based on CI/CD system type
        let pipeline_id = self.extract_pipeline_id(webhook_data)?;
        let execution_result = self.extract_test_results(webhook_data)?;
        
        // Update pipeline status
        self.pipeline_status.insert(pipeline_id.clone(), PipelineStatus {
            pipeline_id,
            status: PipelineExecutionStatus::Success,
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            regression_test_results: Some(execution_result.clone()),
        });
        
        Ok(execution_result)
    }

    /// Extract pipeline ID from webhook
    fn extract_pipeline_id(&self, webhook_data: &serde_json::Value) -> Result<String> {
        // This would extract pipeline ID based on CI/CD system type
        // For now, return a mock ID
        Ok(format!("pipeline_{}", Uuid::new_v4()))
    }

    /// Extract test results from webhook
    fn extract_test_results(&self, webhook_data: &serde_json::Value) -> Result<TestSuiteResult> {
        // This would parse actual test results from webhook data
        // For now, return mock results
        Ok(TestSuiteResult {
            id: Uuid::new_v4(),
            suite_name: "CI/CD Triggered Tests".to_string(),
            test_run_id: Uuid::new_v4().to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            total_tests: 100,
            passed_tests: 95,
            failed_tests: 5,
            skipped_tests: 0,
            regressions_detected: Vec::new(),
            summary: HashMap::new(),
        })
    }
}

// ==========================================
// MONITORING INTEGRATION IMPLEMENTATION
// ==========================================

impl MonitoringIntegrator {
    /// Create new monitoring integrator
    pub fn new(config: Option<MonitoringConfig>) -> Self {
        Self {
            config: config.clone(),
            metrics_collector: MetricsCollector::default(),
            alert_manager: AlertManager {
                active_alerts: HashMap::new(),
                alert_history: Vec::new(),
            },
        }
    }

    /// Initialize monitoring integration
    pub async fn initialize(&mut self) -> Result<()> {
        if let Some(config) = &self.config {
            info!("Initializing monitoring integration for {:?}", config.system_type);
            
            // Initialize metrics collection
            self.metrics_collector.start_collection().await?;
            
            // Set up alert thresholds
            self.setup_alert_thresholds(&config.alert_thresholds)?;
            
            debug!("Monitoring integration initialized");
        }
        
        Ok(())
    }

    /// Start metrics collection
    async fn start_collection(&mut self) -> Result<()> {
        debug!("Starting metrics collection");
        
        // Start background collection task
        // This would be a tokio task in real implementation
        self.metrics_collector.last_collection = Some(Utc::now());
        
        Ok(())
    }

    /// Set up alert thresholds
    fn setup_alert_thresholds(&mut self, thresholds: &AlertThresholds) -> Result<()> {
        info!("Configuring alert thresholds: CPU {}%, Memory {}%, Response {}ms, Error Rate {}%", 
              thresholds.cpu_usage_percent,
              thresholds.memory_usage_percent,
              thresholds.response_time_ms,
              thresholds.error_rate_percent);
        
        // Store thresholds for alert evaluation
        // In real implementation, these would be stored and used for alert generation
        
        Ok(())
    }

    /// Collect metrics from monitoring system
    pub async fn collect_metrics(&mut self) -> Result<()> {
        debug!("Collecting metrics from monitoring system");
        
        if let Some(config) = &self.config {
            match config.system_type {
                MonitoringSystemType::Prometheus => {
                    self.collect_prometheus_metrics().await?;
                }
                MonitoringSystemType::Grafana => {
                    self.collect_grafana_metrics().await?;
                }
                MonitoringSystemType::DataDog => {
                    self.collect_datadog_metrics().await?;
                }
                _ => {
                    warn!("Monitoring system type not implemented yet");
                }
            }
        }
        
        self.metrics_collector.last_collection = Some(Utc::now());
        Ok(())
    }

    /// Collect metrics from Prometheus
    async fn collect_prometheus_metrics(&self) -> Result<()> {
        if let Some(config) = &self.config {
            // Query Prometheus for system metrics
            let cpu_query = "rate(cpu_usage_seconds_total[5m])";
            let memory_query = "memory_usage_bytes";
            
            // This would make actual Prometheus queries
            debug!("Collecting Prometheus metrics: CPU, Memory, etc.");
        }
        
        Ok(())
    }

    /// Collect metrics from Grafana
    async fn collect_grafana_metrics(&self) -> Result<()> {
        // This would query Grafana dashboards for metrics
        debug!("Collecting Grafana metrics");
        Ok(())
    }

    /// Collect metrics from DataDog
    async fn collect_datadog_metrics(&self) -> Result<()> {
        // This would query DataDog API for metrics
        debug!("Collecting DataDog metrics");
        Ok(())
    }

    /// Evaluate alerts based on collected metrics
    pub async fn evaluate_alerts(&mut self) -> Result<()> {
        debug!("Evaluating alert conditions");
        
        // This would evaluate collected metrics against thresholds
        // and trigger alerts if conditions are met
        
        // For now, just check for some mock conditions
        if let Some(_last_collection) = self.metrics_collector.last_collection {
            // Mock alert evaluation
            self.check_performance_alerts().await?;
        }
        
        Ok(())
    }

    /// Check performance-related alerts
    async fn check_performance_alerts(&mut self) -> Result<()> {
        // Check for performance regression alerts
        // This would integrate with the regression detection system
        
        debug!("Checking performance alerts");
        
        // Mock alert generation
        let mock_alert = ActiveAlert {
            alert_id: format!("perf_alert_{}", Uuid::new_v4()),
            alert_type: AlertType::PerformanceRegression,
            severity: AlertSeverity::Warning,
            message: "Performance regression detected in kernel latency".to_string(),
            triggered_at: Utc::now(),
            acknowledged: false,
        };
        
        self.alert_manager.active_alerts.insert(
            mock_alert.alert_id.clone(),
            mock_alert.clone(),
        );
        
        self.alert_manager.alert_history.push(AlertRecord {
            alert_id: mock_alert.alert_id,
            alert_type: mock_alert.alert_type,
            severity: mock_alert.alert_severity,
            triggered_at: mock_alert.triggered_at,
            resolved_at: None,
            resolution_time_seconds: None,
        });
        
        Ok(())
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<ActiveAlert> {
        self.alert_manager.active_alerts.values().cloned().collect()
    }

    /// Acknowledge alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.alert_manager.active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            info!("Alert acknowledged: {}", alert_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Alert not found: {}", alert_id))
        }
    }

    /// Get alert history
    pub fn get_alert_history(&self, hours_back: u32) -> Vec<AlertRecord> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours_back as i64);
        
        self.alert_manager.alert_history
            .iter()
            .filter(|record| record.triggered_at >= cutoff_time)
            .cloned()
            .collect()
    }
}

// ==========================================
// GIT INTEGRATION IMPLEMENTATION
// ==========================================

impl GitIntegration {
    /// Initialize git integration
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing git integration");
        
        // This would initialize git repository connection
        // For now, just log the intent
        debug!("Git integration initialized for repository: {}", self.repository_path);
        
        Ok(())
    }

    /// Get recent code changes
    pub async fn get_recent_changes(
        &self,
        since: DateTime<Utc>,
        branch: Option<&str>,
    ) -> Result<Vec<CodeChange>> {
        debug!("Getting recent code changes since {}", since);
        
        // This would actually query git for changes
        // For now, return mock changes
        
        Ok(vec![
            CodeChange {
                commit_hash: "abc123".to_string(),
                commit_message: "Fix performance regression in kernel".to_string(),
                author: "developer@multios.com".to_string(),
                files_changed: vec!["kernel/scheduler.rs".to_string()],
                timestamp: Utc::now(),
                change_type: "bugfix".to_string(),
            }
        ])
    }

    /// Analyze code changes for test impact
    pub async fn analyze_code_impact(
        &self,
        changes: &[CodeChange],
    ) -> Result<HashMap<String, f64>> {
        debug!("Analyzing code change impact for {} changes", changes.len());
        
        // This would analyze changed files and determine test impact
        // For now, return mock impact scores
        
        let mut impact_scores = HashMap::new();
        impact_scores.insert("kernel".to_string(), 0.9);
        impact_scores.insert("scheduler".to_string(), 0.8);
        impact_scores.insert("filesystem".to_string(), 0.3);
        
        Ok(impact_scores)
    }
}

// ==========================================
// HTTP CLIENT ACCESSOR
// ==========================================

impl CICDIntegrator {
    fn http_client(&self) -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default()
    }
}

impl MonitoringIntegrator {
    fn http_client(&self) -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default()
    }
}

// ==========================================
// INTEGRATION FACTORY
// ==========================================

/// Factory for creating all integration types
#[derive(Debug)]
pub struct IntegrationFactory;

impl IntegrationFactory {
    /// Create benchmark integrator
    pub fn create_benchmark_integrator(config: Option<BenchmarkConfig>) -> BenchmarkIntegrator {
        BenchmarkIntegrator::new(config)
    }

    /// Create CI/CD integrator
    pub fn create_cicd_integrator(config: Option<CICDConfig>) -> CICDIntegrator {
        CICDIntegrator::new(config)
    }

    /// Create monitoring integrator
    pub fn create_monitoring_integrator(config: Option<MonitoringConfig>) -> MonitoringIntegrator {
        MonitoringIntegrator::new(config)
    }
}