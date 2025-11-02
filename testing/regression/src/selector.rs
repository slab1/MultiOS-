//! Change-Based Test Selection Module
//!
//! Provides intelligent test selection algorithms that analyze code changes
//! and select the most relevant tests to run based on impact analysis,
//! risk assessment, and historical test effectiveness.

use anyhow::{Result};
use chrono::{DateTime, Utc};
use log::{info, debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use git2::{Repository, Commit, Diff, DiffOptions};

use crate::{CodeChange, TestSuiteConfig, Uuid};

/// Change-based test selector
#[derive(Debug, Clone)]
pub struct ChangeBasedSelector {
    /// Configuration for test selection
    pub config: ChangeBasedTestingConfig,
    /// Historical test effectiveness data
    historical_data: HistoricalTestData,
    /// Test impact analyzer
    impact_analyzer: ImpactAnalyzer,
}

/// Configuration for change-based testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBasedTestingConfig {
    pub enabled: bool,
    pub impact_analysis_depth: usize,
    pub max_tests_per_change: usize,
    pub test_selection_algorithm: String, // risk_based, coverage_based, history_based
    pub risk_threshold: f64,
    pub confidence_threshold: f64,
}

/// Historical test effectiveness data
#[derive(Debug, Default)]
struct HistoricalTestData {
    /// Test effectiveness by component
    test_effectiveness: HashMap<String, HashMap<String, TestEffectiveness>>,
    /// Component dependencies
    component_dependencies: HashMap<String, HashSet<String>>,
    /// Test failure patterns
    failure_patterns: HashMap<String, Vec<FailurePattern>>,
}

/// Test effectiveness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestEffectiveness {
    pub test_name: String,
    pub component: String,
    pub success_rate: f64,
    pub failure_detection_rate: f64,
    pub avg_execution_time_ms: u64,
    pub last_execution: DateTime<Utc>,
    pub failure_count: u32,
    pub total_executions: u32,
}

/// Pattern of test failures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FailurePattern {
    pub pattern_type: FailurePatternType,
    pub related_components: Vec<String>,
    pub code_change_types: Vec<String>,
    pub frequency: u32,
    pub last_occurrence: DateTime<Utc>,
}

/// Types of failure patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
enum FailurePatternType {
    Regression,
    Integration,
    Performance,
    Security,
    DataConsistency,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub changed_components: HashSet<String>,
    pub affected_components: HashSet<String>,
    pub risk_score: f64,
    pub confidence_score: f64,
    pub impact_type: ImpactType,
    pub reasoning: String,
}

/// Types of impact
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ImpactType {
    Low,
    Medium,
    High,
    Critical,
}

/// Test selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSelectionResult {
    pub code_changes: Vec<CodeChange>,
    pub selected_tests: Vec<SelectedTest>,
    pub selection_algorithm: String,
    pub total_risk_score: f64,
    pub confidence_score: f64,
    pub estimated_execution_time_ms: u64,
    pub reasoning: Vec<String>,
}

/// Individual selected test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedTest {
    pub test_name: String,
    pub component: String,
    pub test_type: SelectedTestType,
    pub priority: TestPriority,
    pub selection_reason: String,
    pub expected_execution_time_ms: u64,
    pub risk_score: f64,
}

/// Types of selected tests
#[derive(Debug, Clone, Serialize, Deserialize)]
enum SelectedTestType {
    Unit,
    Integration,
    Functional,
    Performance,
    Security,
    Regression,
}

/// Test priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
enum TestPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Code change analyzer
#[derive(Debug)]
struct ImpactAnalyzer {
    /// Repository for code analysis
    repository: Option<Repository>,
    /// Component mapping from file paths
    component_mapper: ComponentMapper,
}

/// Maps file paths to components
#[derive(Debug, Default)]
struct ComponentMapper {
    /// File patterns to component mapping
    file_patterns: HashMap<String, String>,
    /// Directory to component mapping
    directory_patterns: HashMap<String, String>,
}

impl ComponentMapper {
    fn new() -> Self {
        let mut mapper = Self::default();
        mapper.initialize_default_patterns();
        mapper
    }

    /// Initialize default file and directory patterns
    fn initialize_default_patterns(&mut self) {
        // Common component patterns
        self.file_patterns.insert("*.rs".to_string(), "rust_component".to_string());
        self.file_patterns.insert("*.py".to_string(), "python_component".to_string());
        self.file_patterns.insert("*.js".to_string(), "javascript_component".to_string());
        self.file_patterns.insert("*.go".to_string(), "go_component".to_string());
        self.file_patterns.insert("*.java".to_string(), "java_component".to_string());
        
        // Test file patterns
        self.file_patterns.insert("*_test.rs".to_string(), "test_component".to_string());
        self.file_patterns.insert("test_*.py".to_string(), "test_component".to_string());
        self.file_patterns.insert("*_spec.js".to_string(), "test_component".to_string());
        
        // Directory patterns
        self.directory_patterns.insert("src/".to_string(), "source".to_string());
        self.directory_patterns.insert("lib/".to_string(), "library".to_string());
        self.directory_patterns.insert("api/".to_string(), "api".to_string());
        self.directory_patterns.insert("db/".to_string(), "database".to_string());
        self.directory_patterns.insert("tests/".to_string(), "testing".to_string());
        self.directory_patterns.insert("test/".to_string(), "testing".to_string());
    }

    /// Map file path to component
    fn map_file_to_component(&self, file_path: &str) -> String {
        // Try directory patterns first
        for (pattern, component) in &self.directory_patterns {
            if file_path.contains(pattern) {
                return component.clone();
            }
        }
        
        // Try file patterns
        for (pattern, component) in &self.file_patterns {
            if self.matches_pattern(file_path, pattern) {
                return component.clone();
            }
        }
        
        // Default to filename-based component
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        file_name.split('.').next().unwrap_or("unknown").to_string()
    }

    /// Check if file matches pattern
    fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let middle = &pattern[1..pattern.len()-1];
            file_path.contains(middle)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            file_path.ends_with(suffix)
        } else if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            file_path.starts_with(prefix)
        } else {
            file_path == pattern
        }
    }
}

impl ChangeBasedSelector {
    /// Create new change-based test selector
    pub fn new(config: ChangeBasedTestingConfig) -> Self {
        Self {
            config: config.clone(),
            historical_data: HistoricalTestData::default(),
            impact_analyzer: ImpactAnalyzer {
                repository: None,
                component_mapper: ComponentMapper::new(),
            },
        }
    }

    /// Select tests based on code changes
    pub async fn select_tests_for_changes(
        &self,
        code_changes: &[CodeChange],
    ) -> Result<Vec<SelectedTest>> {
        info!("Selecting tests for {} code changes", code_changes.len());
        
        if !self.config.enabled {
            debug!("Change-based testing is disabled");
            return Ok(Vec::new());
        }
        
        // Analyze impact of code changes
        let impact_analysis = self.analyze_code_change_impact(code_changes).await?;
        
        debug!("Impact analysis: {} changed components, {} affected components",
               impact_analysis.changed_components.len(),
               impact_analysis.affected_components.len());
        
        // Select tests based on algorithm
        let selected_tests = match self.config.test_selection_algorithm.as_str() {
            "risk_based" => self.select_tests_risk_based(&impact_analysis).await?,
            "coverage_based" => self.select_tests_coverage_based(&impact_analysis).await?,
            "history_based" => self.select_tests_history_based(&impact_analysis).await?,
            algorithm => {
                warn!("Unknown test selection algorithm: {}, using risk_based", algorithm);
                self.select_tests_risk_based(&impact_analysis).await?
            }
        };
        
        // Apply max tests per change limit
        let limited_tests = self.apply_test_limits(selected_tests);
        
        info!("Selected {} tests for execution", limited_tests.len());
        Ok(limited_tests)
    }

    /// Analyze impact of code changes
    async fn analyze_code_change_impact(&self, code_changes: &[CodeChange]) -> Result<ImpactAnalysis> {
        debug!("Analyzing impact of code changes");
        
        let mut changed_components = HashSet::new();
        let mut affected_components = HashSet::new();
        let mut total_risk_score = 0.0;
        let mut total_confidence = 0.0;
        
        for change in code_changes {
            // Analyze changed files
            for file_path in &change.files_changed {
                let component = self.impact_analyzer.component_mapper.map_file_to_component(file_path);
                changed_components.insert(component.clone());
                
                // Analyze dependencies to find affected components
                let deps = self.get_component_dependencies(&component);
                affected_components.extend(deps);
                
                // Calculate risk score for this change
                let change_risk = self.calculate_change_risk(change, &component);
                total_risk_score += change_risk;
                
                // Calculate confidence score
                let confidence = self.calculate_change_confidence(change, &component);
                total_confidence += confidence;
            }
        }
        
        // Remove changed components from affected (they're already included)
        for component in &changed_components {
            affected_components.remove(component);
        }
        
        // Calculate overall scores
        let avg_risk = if !code_changes.is_empty() {
            total_risk_score / code_changes.len() as f64
        } else {
            0.0
        };
        
        let avg_confidence = if !code_changes.is_empty() {
            total_confidence / code_changes.len() as f64
        } else {
            0.0
        };
        
        // Determine impact type
        let impact_type = match avg_risk {
            r if r >= 0.8 => ImpactType::Critical,
            r if r >= 0.6 => ImpactType::High,
            r if r >= 0.3 => ImpactType::Medium,
            _ => ImpactType::Low,
        };
        
        // Generate reasoning
        let reasoning = self.generate_impact_reasoning(
            &changed_components,
            &affected_components,
            avg_risk,
            impact_type.clone(),
        );
        
        Ok(ImpactAnalysis {
            changed_components,
            affected_components,
            risk_score: avg_risk,
            confidence_score: avg_confidence,
            impact_type,
            reasoning,
        })
    }

    /// Select tests using risk-based algorithm
    async fn select_tests_risk_based(&self, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        debug!("Using risk-based test selection algorithm");
        
        let mut selected_tests = Vec::new();
        
        // High priority tests for changed components
        for component in &impact_analysis.changed_components {
            let tests = self.select_tests_for_component(component, TestPriority::Critical).await?;
            selected_tests.extend(tests);
        }
        
        // Medium priority tests for affected components
        for component in &impact_analysis.affected_components {
            let tests = self.select_tests_for_component(component, TestPriority::High).await?;
            selected_tests.extend(tests);
        }
        
        // Add risk-based tests based on impact type
        match &impact_analysis.impact_type {
            ImpactType::Critical => {
                let critical_tests = self.select_critical_risk_tests(impact_analysis).await?;
                selected_tests.extend(critical_tests);
            }
            ImpactType::High => {
                let high_risk_tests = self.select_high_risk_tests(impact_analysis).await?;
                selected_tests.extend(high_risk_tests);
            }
            _ => {}
        }
        
        // Sort by priority and risk score
        selected_tests.sort_by(|a, b| {
            match (a.priority.partial_cmp(&b.priority), a.risk_score.partial_cmp(&b.risk_score)) {
                (Some(std::cmp::Ordering::Greater), _) | 
                (Some(std::cmp::Ordering::Equal), Some(std::cmp::Ordering::Greater)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Less,
            }
        });
        
        Ok(selected_tests)
    }

    /// Select tests using coverage-based algorithm
    async fn select_tests_coverage_based(&self, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        debug!("Using coverage-based test selection algorithm");
        
        let mut selected_tests = Vec::new();
        
        // Get all components that could be affected
        let mut all_components = HashSet::new();
        all_components.extend(impact_analysis.changed_components.clone());
        all_components.extend(impact_analysis.affected_components.clone());
        
        // Select tests based on code coverage potential
        for component in &all_components {
            let coverage_tests = self.select_tests_by_coverage(component, impact_analysis).await?;
            selected_tests.extend(coverage_tests);
        }
        
        // Ensure we have good coverage across different test types
        let diversified_tests = self.ensure_coverage_diversity(&mut selected_tests).await?;
        selected_tests.extend(diversified_tests);
        
        Ok(selected_tests)
    }

    /// Select tests using history-based algorithm
    async fn select_tests_history_based(&self, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        debug!("Using history-based test selection algorithm");
        
        let mut selected_tests = Vec::new();
        
        // Find historically effective tests for affected components
        for component in impact_analysis.changed_components.iter().chain(&impact_analysis.affected_components) {
            let historical_tests = self.select_historical_tests(component, impact_analysis).await?;
            selected_tests.extend(historical_tests);
        }
        
        // Add tests that have caught similar issues in the past
        let pattern_tests = self.select_pattern_based_tests(impact_analysis).await?;
        selected_tests.extend(pattern_tests);
        
        Ok(selected_tests)
    }

    /// Select tests for a specific component and priority
    async fn select_tests_for_component(&self, component: &str, priority: TestPriority) -> Result<Vec<SelectedTest>> {
        let mut tests = Vec::new();
        
        // Unit tests for the component
        let unit_tests = vec![
            SelectedTest {
                test_name: format!("test_{}_unit", component),
                component: component.to_string(),
                test_type: SelectedTestType::Unit,
                priority,
                selection_reason: format!("Unit tests for component {}", component),
                expected_execution_time_ms: 100,
                risk_score: 0.8,
            }
        ];
        tests.extend(unit_tests);
        
        // Add different test types based on priority
        match priority {
            TestPriority::Critical | TestPriority::High => {
                tests.push(SelectedTest {
                    test_name: format!("test_{}_integration", component),
                    component: component.to_string(),
                    test_type: SelectedTestType::Integration,
                    priority,
                    selection_reason: format!("Integration tests for component {}", component),
                    expected_execution_time_ms: 500,
                    risk_score: 0.7,
                });
                
                tests.push(SelectedTest {
                    test_name: format!("test_{}_functional", component),
                    component: component.to_string(),
                    test_type: SelectedTestType::Functional,
                    priority,
                    selection_reason: format!("Functional tests for component {}", component),
                    expected_execution_time_ms: 300,
                    risk_score: 0.6,
                });
            }
            TestPriority::Medium => {
                tests.push(SelectedTest {
                    test_name: format!("test_{}_regression", component),
                    component: component.to_string(),
                    test_type: SelectedTestType::Regression,
                    priority,
                    selection_reason: format!("Regression tests for component {}", component),
                    expected_execution_time_ms: 200,
                    risk_score: 0.5,
                });
            }
            TestPriority::Low => {
                // Fewer tests for low priority
                if tests.is_empty() {
                    tests.push(SelectedTest {
                        test_name: format!("test_{}_smoke", component),
                        component: component.to_string(),
                        test_type: SelectedTestType::Unit,
                        priority,
                        selection_reason: format!("Smoke tests for component {}", component),
                        expected_execution_time_ms: 50,
                        risk_score: 0.3,
                    });
                }
            }
        }
        
        Ok(tests)
    }

    /// Select critical risk tests
    async fn select_critical_risk_tests(&self, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        let mut tests = Vec::new();
        
        // Add performance tests for high-risk changes
        for component in &impact_analysis.changed_components {
            tests.push(SelectedTest {
                test_name: format!("test_{}_performance", component),
                component: component.to_string(),
                test_type: SelectedTestType::Performance,
                priority: TestPriority::Critical,
                selection_reason: "Performance regression testing for critical changes".to_string(),
                expected_execution_time_ms: 1000,
                risk_score: 0.9,
            });
        }
        
        // Add security tests for system-level changes
        if impact_analysis.changed_components.contains("security") || 
           impact_analysis.changed_components.contains("auth") {
            tests.push(SelectedTest {
                test_name: "test_security_comprehensive".to_string(),
                component: "security".to_string(),
                test_type: SelectedTestType::Security,
                priority: TestPriority::Critical,
                selection_reason: "Comprehensive security testing for critical system changes".to_string(),
                expected_execution_time_ms: 2000,
                risk_score: 0.95,
            });
        }
        
        Ok(tests)
    }

    /// Select high risk tests
    async fn select_high_risk_tests(&self, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        let mut tests = Vec::new();
        
        // Add regression tests for API changes
        if impact_analysis.affected_components.contains("api") ||
           impact_analysis.affected_components.contains("interface") {
            tests.push(SelectedTest {
                test_name: "test_api_regression".to_string(),
                component: "api".to_string(),
                test_type: SelectedTestType::Regression,
                priority: TestPriority::High,
                selection_reason: "API regression testing for interface changes".to_string(),
                expected_execution_time_ms: 1500,
                risk_score: 0.8,
            });
        }
        
        Ok(tests)
    }

    /// Select tests by coverage potential
    async fn select_tests_by_coverage(&self, component: &str, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        let mut tests = Vec::new();
        
        // Unit tests for basic coverage
        tests.push(SelectedTest {
            test_name: format!("test_{}_coverage", component),
            component: component.to_string(),
            test_type: SelectedTestType::Unit,
            priority: TestPriority::High,
            selection_reason: format!("Code coverage for component {}", component),
            expected_execution_time_ms: 150,
            risk_score: 0.7,
        });
        
        // Integration tests for interface coverage
        tests.push(SelectedTest {
            test_name: format!("test_{}_interface_coverage", component),
            component: component.to_string(),
            test_type: SelectedTestType::Integration,
            priority: TestPriority::Medium,
            selection_reason: format!("Interface coverage for component {}", component),
            expected_execution_time_ms: 400,
            risk_score: 0.6,
        });
        
        Ok(tests)
    }

    /// Ensure coverage diversity across test types
    async fn ensure_coverage_diversity(&self, tests: &mut Vec<SelectedTest>) -> Result<Vec<SelectedTest>> {
        let mut additional_tests = Vec::new();
        
        // Check if we have each test type represented
        let test_types: HashSet<_> = tests.iter().map(|t| &t.test_type).collect();
        
        if !test_types.contains(&SelectedTestType::Unit) {
            additional_tests.push(SelectedTest {
                test_name: "test_unit_diversity".to_string(),
                component: "core".to_string(),
                test_type: SelectedTestType::Unit,
                priority: TestPriority::Medium,
                selection_reason: "Unit test coverage diversity".to_string(),
                expected_execution_time_ms: 100,
                risk_score: 0.5,
            });
        }
        
        if !test_types.contains(&SelectedTestType::Integration) {
            additional_tests.push(SelectedTest {
                test_name: "test_integration_diversity".to_string(),
                component: "core".to_string(),
                test_type: SelectedTestType::Integration,
                priority: TestPriority::Medium,
                selection_reason: "Integration test coverage diversity".to_string(),
                expected_execution_time_ms: 500,
                risk_score: 0.6,
            });
        }
        
        Ok(additional_tests)
    }

    /// Select tests based on historical effectiveness
    async fn select_historical_tests(&self, component: &str, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        let mut tests = Vec::new();
        
        // Get historical effectiveness data for component
        if let Some(component_effectiveness) = self.historical_data.test_effectiveness.get(component) {
            // Select most effective tests
            let mut effective_tests: Vec<_> = component_effectiveness.values()
                .filter(|&eff| eff.success_rate >= 0.8 && eff.failure_detection_rate >= 0.7)
                .cloned()
                .collect();
            
            // Sort by failure detection rate
            effective_tests.sort_by(|a, b| b.failure_detection_rate.partial_cmp(&a.failure_detection_rate).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take top 3 most effective tests
            for eff_test in effective_tests.iter().take(3) {
                tests.push(SelectedTest {
                    test_name: eff_test.test_name.clone(),
                    component: component.to_string(),
                    test_type: SelectedTestType::Unit, // Default type
                    priority: TestPriority::High,
                    selection_reason: format!("Historically effective test ({}% success rate)", eff_test.success_rate * 100.0),
                    expected_execution_time_ms: eff_test.avg_execution_time_ms,
                    risk_score: eff_test.failure_detection_rate,
                });
            }
        }
        
        Ok(tests)
    }

    /// Select tests based on failure patterns
    async fn select_pattern_based_tests(&self, impact_analysis: &ImpactAnalysis) -> Result<Vec<SelectedTest>> {
        let mut tests = Vec::new();
        
        // Check for relevant failure patterns
        for pattern in &self.historical_data.failure_patterns {
            // Check if current changes match pattern conditions
            if self.pattern_matches_changes(pattern, impact_analysis) {
                tests.push(SelectedTest {
                    test_name: format!("test_pattern_{:?}", pattern.pattern_type).to_lowercase(),
                    component: "pattern_matched".to_string(),
                    test_type: self.get_test_type_for_pattern(&pattern.pattern_type),
                    priority: TestPriority::Critical,
                    selection_reason: format!("Pattern-based test for {:?} failures (occurred {} times)", 
                                           pattern.pattern_type, pattern.frequency),
                    expected_execution_time_ms: 800,
                    risk_score: 0.85,
                });
            }
        }
        
        Ok(tests)
    }

    /// Check if pattern matches current changes
    fn pattern_matches_changes(&self, pattern: &FailurePattern, impact_analysis: &ImpactAnalysis) -> bool {
        // Check if any affected components match pattern
        let has_matching_components = impact_analysis.changed_components.iter()
            .any(|comp| pattern.related_components.contains(comp)) ||
            impact_analysis.affected_components.iter()
            .any(|comp| pattern.related_components.contains(comp));
        
        // Check if change types match
        let has_matching_changes = true; // Simplified - would analyze actual change types
        
        has_matching_components && has_matching_changes
    }

    /// Get test type for failure pattern
    fn get_test_type_for_pattern(&self, pattern_type: &FailurePatternType) -> SelectedTestType {
        match pattern_type {
            FailurePatternType::Regression => SelectedTestType::Regression,
            FailurePatternType::Integration => SelectedTestType::Integration,
            FailurePatternType::Performance => SelectedTestType::Performance,
            FailurePatternType::Security => SelectedTestType::Security,
            FailurePatternType::DataConsistency => SelectedTestType::Functional,
        }
    }

    /// Get component dependencies
    fn get_component_dependencies(&self, component: &str) -> HashSet<String> {
        self.historical_data.component_dependencies
            .get(component)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    /// Calculate risk score for a code change
    fn calculate_change_risk(&self, change: &CodeChange, component: &str) -> f64 {
        let mut risk = 0.3; // Base risk
        
        // Higher risk for complex changes
        let file_count = change.files_changed.len();
        if file_count > 10 {
            risk += 0.3;
        } else if file_count > 5 {
            risk += 0.2;
        } else if file_count > 2 {
            risk += 0.1;
        }
        
        // Higher risk for certain change types
        match change.change_type.as_str() {
            "refactor" => risk += 0.1,
            "feature" => risk += 0.2,
            "bugfix" => risk += 0.1,
            "security" => risk += 0.4,
            "performance" => risk += 0.3,
            _ => {}
        }
        
        // Higher risk for core components
        if ["core", "kernel", "security", "api"].contains(&component) {
            risk += 0.2;
        }
        
        risk.min(1.0)
    }

    /// Calculate confidence score for a code change
    fn calculate_change_confidence(&self, change: &CodeChange, component: &str) -> f64 {
        let mut confidence = 0.7; // Base confidence
        
        // Higher confidence for experienced authors (simplified)
        if change.author.contains("senior") || change.author.contains("lead") {
            confidence += 0.1;
        }
        
        // Higher confidence for smaller, focused changes
        let file_count = change.files_changed.len();
        if file_count == 1 {
            confidence += 0.1;
        } else if file_count > 20 {
            confidence -= 0.2;
        }
        
        // Higher confidence for well-tested components
        if let Some(component_effectiveness) = self.historical_data.test_effectiveness.get(component) {
            let avg_effectiveness: f64 = component_effectiveness.values()
                .map(|eff| eff.success_rate)
                .sum::<f64>() / component_effectiveness.len() as f64;
            confidence += avg_effectiveness * 0.2;
        }
        
        confidence.min(1.0).max(0.0)
    }

    /// Generate impact reasoning
    fn generate_impact_reasoning(
        &self,
        changed: &HashSet<String>,
        affected: &HashSet<String>,
        risk_score: f64,
        impact_type: ImpactType,
    ) -> String {
        let mut reasoning = String::new();
        
        reasoning.push_str(&format!("Analyzed {} code changes affecting {} components. ", 
                                   changed.len() + affected.len(), changed.len() + affected.len()));
        
        if !changed.is_empty() {
            reasoning.push_str(&format!("Directly changed components: {}. ", 
                                       changed.iter().cloned().collect::<Vec<_>>().join(", ")));
        }
        
        if !affected.is_empty() {
            reasoning.push_str(&format!("Indirectly affected components: {}. ", 
                                       affected.iter().cloned().collect::<Vec<_>>().join(", ")));
        }
        
        reasoning.push_str(&format!("Overall risk score: {:.2} ({:?} impact).", risk_score, impact_type));
        
        reasoning
    }

    /// Apply test selection limits
    fn apply_test_limits(&self, mut tests: Vec<SelectedTest>) -> Vec<SelectedTest> {
        // Sort by priority and risk score
        tests.sort_by(|a, b| {
            match (a.priority.partial_cmp(&b.priority), a.risk_score.partial_cmp(&b.risk_score)) {
                (Some(std::cmp::Ordering::Greater), _) => std::cmp::Ordering::Greater,
                (Some(std::cmp::Ordering::Equal), Some(std::cmp::Ordering::Greater)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Less,
            }
        });
        
        // Apply max tests per change limit
        if tests.len() > self.config.max_tests_per_change {
            tests.truncate(self.config.max_tests_per_change);
        }
        
        tests
    }
}

impl ChangeBasedSelector {
    /// Load historical test data from database
    pub async fn load_historical_data(&mut self, test_execution_data: &[TestExecutionRecord]) -> Result<()> {
        info!("Loading {} historical test execution records", test_execution_data.len());
        
        for record in test_execution_data {
            // Update test effectiveness
            let effectiveness = self.historical_data.test_effectiveness
                .entry(record.component.clone())
                .or_insert_with(HashMap::new)
                .entry(record.test_name.clone())
                .or_insert_with(|| TestEffectiveness {
                    test_name: record.test_name.clone(),
                    component: record.component.clone(),
                    success_rate: 0.0,
                    failure_detection_rate: 0.0,
                    avg_execution_time_ms: 0,
                    last_execution: Utc::now(),
                    failure_count: 0,
                    total_executions: 0,
                });
            
            effectiveness.total_executions += 1;
            effectiveness.last_execution = record.execution_time;
            
            if record.success {
                // Update success rate
                effectiveness.success_rate = ((effectiveness.success_rate * (effectiveness.total_executions - 1) as f64) + 1.0) / effectiveness.total_executions as f64;
            } else {
                effectiveness.failure_count += 1;
                // Update failure detection rate (simplified)
                effectiveness.failure_detection_rate = effectiveness.failure_count as f64 / effectiveness.total_executions as f64;
            }
            
            // Update average execution time
            effectiveness.avg_execution_time_ms = ((effectiveness.avg_execution_time_ms * (effectiveness.total_executions - 1) as u64) + record.execution_time_ms) / effectiveness.total_executions as u64;
        }
        
        debug!("Loaded historical data for {} components", self.historical_data.test_effectiveness.len());
        Ok(())
    }

    /// Build component dependency graph
    pub async fn build_dependency_graph(&mut self, dependency_data: &[ComponentDependency]) -> Result<()> {
        info!("Building component dependency graph from {} dependencies", dependency_data.len());
        
        for dependency in dependency_data {
            self.historical_data.component_dependencies
                .entry(dependency.component.clone())
                .or_insert_with(HashSet::new)
                .insert(dependency.depends_on.clone());
        }
        
        debug!("Built dependency graph with {} components", self.historical_data.component_dependencies.len());
        Ok(())
    }
}

/// Historical test execution record
#[derive(Debug, Clone)]
struct TestExecutionRecord {
    pub test_name: String,
    pub component: String,
    pub success: bool,
    pub execution_time: DateTime<Utc>,
    pub execution_time_ms: u64,
}

/// Component dependency
#[derive(Debug, Clone)]
struct ComponentDependency {
    pub component: String,
    pub depends_on: String,
}

impl ChangeBasedSelector {
    /// Get test selection statistics
    pub fn get_selection_stats(&self) -> SelectionStatistics {
        let total_historical_tests = self.historical_data.test_effectiveness
            .values()
            .map(|component_tests| component_tests.len())
            .sum();
        
        let avg_effectiveness: f64 = if total_historical_tests > 0 {
            let mut total_effectiveness = 0.0;
            let mut count = 0;
            
            for component_tests in self.historical_data.test_effectiveness.values() {
                for test_effectiveness in component_tests.values() {
                    total_effectiveness += test_effectiveness.success_rate;
                    count += 1;
                }
            }
            
            if count > 0 {
                total_effectiveness / count as f64
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        SelectionStatistics {
            total_historical_tests,
            components_tracked: self.historical_data.test_effectiveness.len(),
            average_effectiveness: avg_effectiveness,
            dependency_graph_size: self.historical_data.component_dependencies.len(),
            failure_patterns_count: self.historical_data.failure_patterns.len(),
        }
    }
}

/// Test selection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionStatistics {
    pub total_historical_tests: usize,
    pub components_tracked: usize,
    pub average_effectiveness: f64,
    pub dependency_graph_size: usize,
    pub failure_patterns_count: usize,
}