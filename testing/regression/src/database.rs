//! Database management module for regression testing system
//!
//! Provides PostgreSQL database connectivity and operations for storing
//! and retrieving regression testing data including baselines, measurements,
//! test results, and regression detections.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use deadpool::managed::{Manager, Object, Pool};
use deadpool_postgres::{Manager as PgManager, Pool as PgPool};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Pool, Postgres, Row};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    DetectedRegression, PerformanceMeasurement, RootCauseAnalysis, TestResult, TestSuiteResult,
    TrendData, Uuid,
};

/// Database connection pool manager
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create new database manager with connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Initializing database connection pool");
        
        let config = tokio_postgres::Config::from_str(database_url)
            .context("Invalid database URL format")?;
        
        let (client, connection) = config.connect_host(
            "localhost", 
            config.get_host().unwrap_or("localhost"),
            config.get_port().unwrap_or(5432),
            config.get_user().unwrap_or("postgres"),
            Some(&config.get_password().unwrap_or_default()),
            config.get_dbname().unwrap_or("multios_regression"),
        ).await
            .context("Failed to connect to database")?;
        
        // Spawn connection handling task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection error: {}", e);
            }
        });
        
        // Create pool
        let pool = PgPool::new(client, 5);
        
        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema");
        
        // Read and execute schema file
        let schema_sql = include_str!("../../database_schema.sql");
        
        self.pool.execute(schema_sql)
            .await
            .context("Failed to initialize database schema")?;
        
        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Get database connection
    pub async fn get_connection(&self) -> Result<Object<PgManager>> {
        self.pool.get().await
            .context("Failed to get database connection from pool")
    }

    // ==========================================
    // PERFORMANCE BASELINES OPERATIONS
    // ==========================================

    /// Store performance baseline
    pub async fn store_performance_baseline(&self, baseline: &PerformanceBaseline) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO performance_baselines 
            (test_name, component, metric_type, baseline_value, confidence_interval, 
             sample_count, measurement_unit, test_environment_hash, metadata, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (test_environment_hash) DO UPDATE SET
                baseline_value = EXCLUDED.baseline_value,
                confidence_interval = EXCLUDED.confidence_interval,
                sample_count = EXCLUDED.sample_count,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
            "#,
            baseline.test_name,
            baseline.component,
            baseline.metric_type,
            baseline.baseline_value,
            baseline.confidence_interval,
            baseline.sample_count,
            baseline.measurement_unit,
            baseline.test_environment_hash,
            serde_json::to_value(&baseline.metadata)?,
            baseline.is_active,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store performance baseline")?;

        Ok(())
    }

    /// Get performance baselines for component and metric type
    pub async fn get_performance_baselines(
        &self,
        component: &str,
        metric_type: &str,
    ) -> Result<Vec<PerformanceBaseline>> {
        let rows = sqlx::query!(
            r#"
            SELECT test_name, component, metric_type, baseline_value, confidence_interval,
                   sample_count, measurement_unit, test_environment_hash, created_at,
                   updated_at, metadata, is_active
            FROM performance_baselines
            WHERE component = $1 AND metric_type = $2 AND is_active = true
            ORDER BY created_at DESC
            "#,
            component,
            metric_type,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch performance baselines")?;

        let baselines = rows
            .into_iter()
            .map(|row| PerformanceBaseline {
                test_name: row.test_name,
                component: row.component,
                metric_type: row.metric_type,
                baseline_value: row.baseline_value,
                confidence_interval: row.confidence_interval,
                sample_count: row.sample_count,
                measurement_unit: row.measurement_unit,
                test_environment_hash: row.test_environment_hash,
                created_at: row.created_at,
                updated_at: row.updated_at,
                metadata: serde_json::from_value(row.metadata.unwrap_or_default())?,
                is_active: row.is_active,
            })
            .collect();

        Ok(baselines)
    }

    // ==========================================
    // PERFORMANCE MEASUREMENTS OPERATIONS
    // ==========================================

    /// Store performance measurement
    pub async fn store_performance_measurement(&self, measurement: &PerformanceMeasurement) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO performance_measurements
            (test_name, component, metric_type, measured_value, measurement_unit,
             test_environment_hash, test_run_id, execution_time_ms, regression_detected,
             severity_level, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            measurement.test_name,
            measurement.component,
            measurement.metric_type,
            measurement.value,
            measurement.unit,
            measurement.environment.environment_hash,
            measurement.test_run_id,
            measurement.execution_time_ms,
            false, // Will be updated after regression detection
            None,
            serde_json::to_value(&measurement.environment)?,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store performance measurement")?;

        Ok(())
    }

    /// Get performance measurements for trend analysis
    pub async fn get_performance_measurements(
        &self,
        component: &str,
        metric_type: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<PerformanceMeasurement>> {
        let rows = sqlx::query!(
            r#"
            SELECT pm.test_name, pm.component, pm.metric_type, pm.measured_value,
                   pm.measurement_unit, pm.test_environment_hash, pm.test_run_id,
                   pm.execution_time_ms, pm.timestamp, pm.regression_detected,
                   pm.severity_level, te.env_name, te.hardware_config, te.software_config
            FROM performance_measurements pm
            JOIN test_environments te ON pm.test_environment_hash = te.environment_hash
            WHERE pm.component = $1 AND pm.metric_type = $2
              AND pm.timestamp BETWEEN $3 AND $4
            ORDER BY pm.timestamp ASC
            "#,
            component,
            metric_type,
            start_time,
            end_time,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch performance measurements")?;

        let measurements = rows
            .into_iter()
            .map(|row| PerformanceMeasurement {
                id: Uuid::new_v4(), // Generate new ID as it's not stored
                test_name: row.test_name,
                component: row.component,
                metric_type: row.metric_type,
                value: row.measured_value,
                unit: row.measurement_unit,
                test_run_id: row.test_run_id,
                timestamp: row.timestamp,
                environment: TestEnvironment {
                    name: row.env_name,
                    hardware_config: serde_json::from_value(row.hardware_config.unwrap_or_default())?,
                    software_config: serde_json::from_value(row.software_config.unwrap_or_default())?,
                    environment_hash: row.test_environment_hash,
                },
            })
            .collect();

        Ok(measurements)
    }

    // ==========================================
    // FUNCTIONAL TEST RESULTS OPERATIONS
    // ==========================================

    /// Store functional test result
    pub async fn store_test_result(&self, test_result: &TestResult) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO functional_test_results
            (test_name, component, test_type, execution_status, execution_time_ms,
             failure_reason, error_details, logs, test_run_id, environment_hash,
             test_data_used, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            test_result.test_name,
            test_result.component,
            format!("{:?}", test_result.test_type),
            format!("{:?}", test_result.status),
            test_result.execution_time_ms,
            None, // Will be populated if failed
            None, // Will be populated if error
            None, // Test logs
            test_result.id.to_string(),
            test_result.environment.environment_hash,
            serde_json::to_value(&test_result.metadata)?,
            serde_json::to_value(&test_result.environment)?,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store test result")?;

        Ok(())
    }

    /// Get test results for analysis
    pub async fn get_test_results(
        &self,
        component: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TestResult>> {
        let rows = sqlx::query!(
            r#"
            SELECT ftr.test_name, ftr.component, ftr.test_type, ftr.execution_status,
                   ftr.execution_time_ms, ftr.failure_reason, ftr.error_details,
                   ftr.logs, ftr.test_run_id, ftr.timestamp, te.env_name,
                   te.hardware_config, te.software_config, te.environment_hash
            FROM functional_test_results ftr
            JOIN test_environments te ON ftr.environment_hash = te.environment_hash
            WHERE ftr.component = $1
              AND ftr.timestamp BETWEEN $2 AND $3
            ORDER BY ftr.timestamp ASC
            "#,
            component,
            start_time,
            end_time,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch test results")?;

        let results = rows
            .into_iter()
            .map(|row| TestResult {
                id: Uuid::parse_str(&row.test_run_id).unwrap_or_else(|_| Uuid::new_v4()),
                test_name: row.test_name,
                component: row.component,
                test_type: match row.test_type.as_str() {
                    "Unit" => crate::TestType::Unit,
                    "Integration" => crate::TestType::Integration,
                    "EndToEnd" => crate::TestType::EndToEnd,
                    "Performance" => crate::TestType::Performance,
                    "Functional" => crate::TestType::Functional,
                    "Security" => crate::TestType::Security,
                    "Compatibility" => crate::TestType::Compatibility,
                    _ => crate::TestType::Functional,
                },
                status: match row.execution_status.as_str() {
                    "Passed" => crate::TestStatus::Passed,
                    "Failed" => crate::TestStatus::Failed,
                    "Skipped" => crate::TestStatus::Skipped,
                    "Error" => crate::TestStatus::Error,
                    "Timeout" => crate::TestStatus::Timeout,
                    _ => crate::TestStatus::Error,
                },
                execution_time_ms: row.execution_time_ms as u64,
                timestamp: row.timestamp,
                environment: TestEnvironment {
                    name: row.env_name,
                    hardware_config: serde_json::from_value(row.hardware_config.unwrap_or_default())?,
                    software_config: serde_json::from_value(row.software_config.unwrap_or_default())?,
                    environment_hash: row.environment_hash,
                },
                metrics: HashMap::new(),
                metadata: HashMap::new(),
            })
            .collect();

        Ok(results)
    }

    // ==========================================
    // REGRESSION DETECTION OPERATIONS
    // ==========================================

    /// Store detected regression
    pub async fn store_regression(&self, regression: &DetectedRegression) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO detected_regressions
            (regression_type, severity, component, test_name, current_value,
             baseline_value, regression_percentage, detection_algorithm,
             confidence_score, test_run_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            format!("{:?}", regression.regression_type),
            format!("{:?}", regression.severity),
            regression.component,
            regression.test_name,
            regression.current_value,
            regression.baseline_value,
            regression.regression_percentage,
            regression.detection_algorithm,
            regression.confidence_score,
            regression.test_run_id,
            serde_json::to_value(&regression.metadata)?,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store regression")?;

        Ok(())
    }

    /// Get unresolved regressions
    pub async fn get_unresolved_regressions(&self) -> Result<Vec<DetectedRegression>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, regression_type, severity, component, test_name,
                   current_value, baseline_value, regression_percentage,
                   detection_algorithm, confidence_score, test_run_id,
                   detected_at, resolved, resolution_notes, assigned_to, metadata
            FROM detected_regressions
            WHERE resolved = false
            ORDER BY severity DESC, detected_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch unresolved regressions")?;

        let regressions = rows
            .into_iter()
            .map(|row| DetectedRegression {
                id: Uuid::new_v4(), // Generate ID since not stored in DB
                regression_type: match row.regression_type.as_str() {
                    "PerformanceLatency" => crate::RegressionType::PerformanceLatency,
                    "PerformanceThroughput" => crate::RegressionType::PerformanceThroughput,
                    "PerformanceMemory" => crate::RegressionType::PerformanceMemory,
                    "PerformanceCpu" => crate::RegressionType::PerformanceCpu,
                    "Functional" => crate::RegressionType::Functional,
                    "Security" => crate::RegressionType::Security,
                    "Compatibility" => crate::RegressionType::Compatibility,
                    "MemoryLeak" => crate::RegressionType::MemoryLeak,
                    "ResourceExhaustion" => crate::RegressionType::ResourceExhaustion,
                    _ => crate::RegressionType::Functional,
                },
                severity: match row.severity.as_str() {
                    "Minor" => crate::RegressionSeverity::Minor,
                    "Major" => crate::RegressionSeverity::Major,
                    "Critical" => crate::RegressionSeverity::Critical,
                    "Blocker" => crate::RegressionSeverity::Blocker,
                    _ => crate::RegressionSeverity::Minor,
                },
                component: row.component,
                test_name: row.test_name,
                current_value: row.current_value,
                baseline_value: row.baseline_value,
                regression_percentage: row.regression_percentage,
                detection_algorithm: row.detection_algorithm,
                confidence_score: row.confidence_score,
                test_run_id: row.test_run_id,
                timestamp: row.detected_at,
                metadata: serde_json::from_value(row.metadata.unwrap_or_default())?,
            })
            .collect();

        Ok(regressions)
    }

    // ==========================================
    // ROOT CAUSE ANALYSIS OPERATIONS
    // ==========================================

    /// Store root cause analysis
    pub async fn store_root_cause_analysis(&self, rca: &RootCauseAnalysis) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO regression_root_causes
            (regression_id, cause_type, root_cause, contributing_factors,
             code_changes, probability_score, analysis_method, investigator, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            rca.regression_id,
            format!("{:?}", rca.cause_type),
            rca.root_cause,
            serde_json::to_value(rca.contributing_factors)?,
            serde_json::to_value(rca.metadata.get("code_changes").cloned().unwrap_or_default())?,
            rca.probability_score,
            rca.analysis_method,
            None, // investigator not stored in DetectedRegression
            serde_json::to_value(rca.metadata)?,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store root cause analysis")?;

        Ok(())
    }

    // ==========================================
    // TREND ANALYSIS OPERATIONS
    // ==========================================

    /// Store trend analysis result
    pub async fn store_trend_data(&self, trend_data: &TrendData) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO test_trends
            (metric_type, component, time_period, period_start, period_end,
             total_tests, passed_tests, failed_tests, regression_count,
             avg_execution_time_ms, success_rate, trend_direction, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            trend_data.metric_name,
            trend_data.component,
            "daily", // Default time period
            trend_data.time_series.first().map_or(Utc::now(), |(t, _)| *t),
            trend_data.time_series.last().map_or(Utc::now(), |(t, _)| *t),
            trend_data.time_series.len(),
            0, // passed_tests - would need to be calculated
            0, // failed_tests - would need to be calculated
            0, // regression_count - would need to be calculated
            trend_data.statistics.mean,
            0.0, // success_rate - would need to be calculated
            format!("{:?}", trend_data.trend_direction),
            serde_json::to_value(trend_data)?,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store trend data")?;

        Ok(())
    }

    /// Get historical trends for component
    pub async fn get_component_trends(
        &self,
        component: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<TrendData>> {
        let rows = sqlx::query!(
            r#"
            SELECT metric_type, component, period_start, period_end,
                   total_tests, passed_tests, failed_tests, regression_count,
                   avg_execution_time_ms, success_rate, trend_direction, metadata
            FROM test_trends
            WHERE component = $1
              AND period_start BETWEEN $2 AND $3
            ORDER BY period_start ASC
            "#,
            component,
            start_time,
            end_time,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch component trends")?;

        let trends = rows
            .into_iter()
            .map(|row| {
                let metadata = serde_json::from_value(row.metadata.unwrap_or_default()).unwrap_or_default();
                let time_series: Vec<(DateTime<Utc>, f64)> = metadata.get("time_series")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| {
                        arr.iter().map(|item| {
                            if let Some(obj) = item.as_object() {
                                let timestamp = obj.get("timestamp")?.as_str()?;
                                let value = obj.get("value")?.as_f64()?;
                                Some((DateTime::parse_from_rfc3339(timestamp).ok()?.with_timezone(&Utc), value))
                            } else {
                                None
                            }
                        }).collect()
                    })
                    .unwrap_or_default();

                TrendData {
                    metric_name: row.metric_type,
                    component: row.component,
                    time_series,
                    statistics: crate::TrendStatistics {
                        mean: row.avg_execution_time_ms.unwrap_or(0.0),
                        standard_deviation: 0.0, // Would need calculation
                        median: 0.0, // Would need calculation
                        percentile_95: 0.0, // Would need calculation
                        percentile_99: 0.0, // Would need calculation
                        min_value: 0.0, // Would need calculation
                        max_value: 0.0, // Would need calculation
                    },
                    trend_direction: match row.trend_direction.as_str() {
                        "Improving" => crate::TrendDirection::Improving,
                        "Degrading" => crate::TrendDirection::Degrading,
                        "Stable" => crate::TrendDirection::Stable,
                        _ => crate::TrendDirection::Unknown,
                    },
                    predictions: Vec::new(), // Would need ML prediction
                }
            })
            .collect();

        Ok(trends)
    }

    // ==========================================
    // CLEANUP OPERATIONS
    // ==========================================

    /// Clean up old data
    pub async fn cleanup_old_data(&self, retention_days: u32) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        // Clean up old measurements
        let result1 = sqlx::query!(
            r#"DELETE FROM performance_measurements WHERE timestamp < $1"#,
            cutoff_date
        )
        .execute(&self.pool)
        .await?;

        // Clean up old test results
        let result2 = sqlx::query!(
            r#"DELETE FROM functional_test_results WHERE timestamp < $1"#,
            cutoff_date
        )
        .execute(&self.pool)
        .await?;

        // Clean up old alert history
        let result3 = sqlx::query!(
            r#"DELETE FROM alert_history WHERE triggered_at < $1"#,
            cutoff_date
        )
        .execute(&self.pool)
        .await?;

        let total_deleted = result1.rows_affected() + result2.rows_affected() + result3.rows_affected();
        
        info!("Cleaned up {} old records", total_deleted);
        Ok(total_deleted)
    }
}

// ==========================================
// DATA STRUCTURES
// ==========================================

/// Performance baseline record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PerformanceBaseline {
    pub test_name: String,
    pub component: String,
    pub metric_type: String,
    pub baseline_value: f64,
    pub confidence_interval: Option<f64>,
    pub sample_count: i32,
    pub measurement_unit: String,
    pub test_environment_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub is_active: bool,
}

/// Test environment configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TestEnvironmentRecord {
    pub env_name: String,
    pub env_type: String,
    pub hardware_config: serde_json::Value,
    pub software_config: serde_json::Value,
    pub environment_hash: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Regression detection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionStats {
    pub total_regressions: u32,
    pub resolved_regressions: u32,
    pub unresolved_regressions: u32,
    pub regressions_by_severity: HashMap<String, u32>,
    pub regressions_by_type: HashMap<String, u32>,
    pub avg_detection_time_hours: f64,
    pub avg_resolution_time_hours: f64,
}

/// Performance analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalytics {
    pub component: String,
    pub metric_name: String,
    pub statistical_analysis: HashMap<String, f64>,
    pub anomaly_score: f64,
    pub outlier_count: usize,
    pub trend_analysis: HashMap<String, serde_json::Value>,
    pub recommendations: Vec<String>,
    pub analysis_timestamp: DateTime<Utc>,
}

impl DatabaseManager {
    /// Get regression statistics
    pub async fn get_regression_stats(&self, days: u32) -> Result<RegressionStats> {
        let start_date = Utc::now() - chrono::Duration::days(days as i64);
        
        let row = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_regressions,
                COUNT(CASE WHEN resolved = true THEN 1 END) as resolved_regressions,
                COUNT(CASE WHEN resolved = false THEN 1 END) as unresolved_regressions,
                COUNT(CASE WHEN severity = 'Minor' THEN 1 END) as minor_count,
                COUNT(CASE WHEN severity = 'Major' THEN 1 END) as major_count,
                COUNT(CASE WHEN severity = 'Critical' THEN 1 END) as critical_count,
                COUNT(CASE WHEN severity = 'Blocker' THEN 1 END) as blocker_count,
                COUNT(CASE WHEN regression_type = 'PerformanceLatency' THEN 1 END) as performance_latency_count,
                COUNT(CASE WHEN regression_type = 'PerformanceThroughput' THEN 1 END) as performance_throughput_count,
                COUNT(CASE WHEN regression_type = 'PerformanceMemory' THEN 1 END) as performance_memory_count,
                COUNT(CASE WHEN regression_type = 'Functional' THEN 1 END) as functional_count
            FROM detected_regressions
            WHERE detected_at >= $1
            "#,
            start_date,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get regression statistics")?;

        let mut regressions_by_severity = HashMap::new();
        regressions_by_severity.insert("Minor".to_string(), row.minor_count as u32);
        regressions_by_severity.insert("Major".to_string(), row.major_count as u32);
        regressions_by_severity.insert("Critical".to_string(), row.critical_count as u32);
        regressions_by_severity.insert("Blocker".to_string(), row.blocker_count as u32);

        let mut regressions_by_type = HashMap::new();
        regressions_by_type.insert("PerformanceLatency".to_string(), row.performance_latency_count as u32);
        regressions_by_type.insert("PerformanceThroughput".to_string(), row.performance_throughput_count as u32);
        regressions_by_type.insert("PerformanceMemory".to_string(), row.performance_memory_count as u32);
        regressions_by_type.insert("Functional".to_string(), row.functional_count as u32);

        Ok(RegressionStats {
            total_regressions: row.total_regressions as u32,
            resolved_regressions: row.resolved_regressions as u32,
            unresolved_regressions: row.unresolved_regressions as u32,
            regressions_by_severity,
            regressions_by_type,
            avg_detection_time_hours: 0.0, // Would need additional query
            avg_resolution_time_hours: 0.0, // Would need additional query
        })
    }
}