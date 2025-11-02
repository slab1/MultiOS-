use std::collections::{HashMap, HashSet, BTreeMap};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use regex::Regex;
use crate::utils::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowAnalysis {
    pub variable_tracks: Vec<VariableTrack>,
    pub data_dependencies: Vec<DataDependency>,
    pub lifetime_analysis: Vec<LifetimeAnalysis>,
    pub data_hazards: Vec<DataHazard>,
    pub optimization_opportunities: Vec<DataFlowOptimization>,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTrack {
    pub variable_name: String,
    pub variable_type: String,
    pub declaration: DeclarationInfo,
    pub usage_points: Vec<UsagePoint>,
    pub data_flow_path: Vec<DataFlowStep>,
    pub transformation_chain: Vec<Transformation>,
    pub scope_analysis: ScopeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDependency {
    pub source_variable: String,
    pub target_variable: String,
    pub dependency_type: DependencyKind,
    pub strength: DependencyStrength,
    pub path: Vec<FlowPathStep>,
    pub potential_conflicts: Vec<ConflictPoint>,
    pub educational_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeAnalysis {
    pub variable_name: String,
    pub scope_start: u32,
    pub scope_end: u32,
    pub lifetime_type: LifetimeType,
    pub storage_location: StorageLocation,
    pub allocation_strategy: AllocationStrategy,
    pub deallocation_point: Option<u32>,
    pub memory_pressure: MemoryPressure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataHazard {
    pub hazard_type: HazardType,
    pub variables: Vec<String>,
    pub location: CodeLocation,
    pub severity: HazardSeverity,
    pub description: String,
    pub mitigation_strategies: Vec<String>,
    pub educational_implications: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowOptimization {
    pub optimization_type: DataFlowOptimizationType,
    pub variables_affected: Vec<String>,
    pub expected_benefit: String,
    pub implementation_difficulty: ImplementationDifficulty,
    pub code_transformation: Option<CodeTransformation>,
    pub educational_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclarationInfo {
    pub file_path: String,
    pub line_number: u32,
    pub column: u32,
    pub is_mutable: bool,
    pub initialization_value: Option<String>,
    pub inferred_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePoint {
    pub location: CodeLocation,
    pub usage_type: UsageType,
    pub access_mode: AccessMode,
    pub context: String,
    pub side_effects: Vec<SideEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowStep {
    pub from_location: CodeLocation,
    pub to_location: CodeLocation,
    pub operation: FlowOperation,
    pub transformation_description: String,
    pub safety_analysis: SafetyAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub operation: TransformationType,
    pub input_variable: String,
    pub output_variable: String,
    pub operation_location: CodeLocation,
    pub safety_guarantees: Vec<SafetyGuarantee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeAnalysis {
    pub scope_level: ScopeLevel,
    pub enclosing_functions: Vec<String>,
    pub variable_shadowing: Vec<ShadowInfo>,
    pub lifetime_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowPathStep {
    pub from: String,
    pub to: String,
    pub operation: String,
    pub location: CodeLocation,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPoint {
    pub variables_in_conflict: Vec<String>,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub detection_confidence: f64,
    pub resolution_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyKind {
    Direct,
    Indirect,
    Control,
    Data,
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyStrength {
    Strong,
    Medium,
    Weak,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifetimeType {
    Static,
    Stack,
    Heap,
    Register,
    ThreadLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageLocation {
    Global,
    Stack,
    Heap,
    Register,
    MemoryMapped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    Static,
    Automatic,
    Manual,
    Pool,
    Arena,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryPressure {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HazardType {
    DataRace,
    UseAfterFree,
    BufferOverflow,
    IntegerOverflow,
    NullPointerDereference,
    UninitializedVariable,
    ResourceLeak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HazardSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFlowOptimizationType {
    DeadCodeElimination,
    ConstantPropagation,
    LoopInvariantMotion,
    RegisterAllocation,
    MemoryCoalescing,
    LazyEvaluation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageType {
    Read,
    Write,
    ReadWrite,
    AddressOf,
    Reference,
    Move,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Exclusive,
    Shared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffect {
    MemoryAllocation,
    MemoryDeallocation,
    IOWrite,
    StateMutation,
    FunctionCall,
    ExternalModification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowOperation {
    Assignment,
    FunctionCall,
    Return,
    ConditionalBranch,
    LoopIteration,
    ParameterPassing,
    ReferenceCopy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyAnalysis {
    pub is_safe: bool,
    pub risks: Vec<Risk>,
    pub guarantees: Vec<SafetyGuarantee>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGuarantee {
    pub guarantee_type: GuaranteeType,
    pub description: String,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuaranteeType {
    NoNullDereference,
    NoOverflow,
    NoUnderflow,
    NoDivisionByZero,
    BoundsChecked,
    TypeSafe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationType {
    Copy,
    Move,
    Clone,
    Borrow,
    Cast,
    Arithmetic,
    Bitwise,
    Logical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowInfo {
    pub shadowing_variable: String,
    pub shadowed_variable: String,
    pub shadow_start: u32,
    pub shadow_end: u32,
    pub scope_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeLevel {
    Global,
    Function,
    Block,
    Loop,
    Match,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_text: String,
    pub line_number: u32,
    pub is_evaluable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    WriteAfterWrite,
    WriteAfterRead,
    ReadAfterWrite,
    DataRace,
    OrderingConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationDifficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTransformation {
    pub original_code: String,
    pub transformed_code: String,
    pub transformation_description: String,
    pub preserved_semantics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk_type: RiskType,
    pub description: String,
    pub probability: f64,
    pub impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    BufferOverflow,
    IntegerOverflow,
    NullPointerAccess,
    UseAfterFree,
    RaceCondition,
    MemoryLeak,
}

#[async_trait]
pub trait DataFlowAnalysisTrait {
    async fn analyze_data_flow(&self, code: &str, variable_name: &str, line_number: Option<u32>) -> Result<DataFlowAnalysis, anyhow::Error>;
    async fn track_variable_usage(&self, code: &str, variable_name: &str) -> Result<VariableTrack, anyhow::Error>;
    async fn find_data_dependencies(&self, code: &str, variable_name: &str) -> Result<Vec<DataDependency>, anyhow::Error>;
}

pub struct DataFlowAnalyzer {
    variable_tracker: VariableTracker,
    dependency_analyzer: DependencyAnalyzer,
    lifetime_analyzer: LifetimeAnalyzer,
    hazard_detector: HazardDetector,
}

impl DataFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            variable_tracker: VariableTracker::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            lifetime_analyzer: LifetimeAnalyzer::new(),
            hazard_detector: HazardDetector::new(),
        }
    }
}

#[async_trait]
impl DataFlowAnalysisTrait for DataFlowAnalyzer {
    async fn analyze_data_flow(&self, code: &str, variable_name: &str, line_number: Option<u32>) -> Result<DataFlowAnalysis, anyhow::Error> {
        let variable_track = self.variable_tracker.track_variable(code, variable_name).await?;
        let data_dependencies = self.dependency_analyzer.analyze_dependencies(code, variable_name).await?;
        let lifetime_analysis = self.lifetime_analyzer.analyze_lifetime(code, variable_name).await?;
        let data_hazards = self.hazard_detector.detect_hazards(code, variable_name).await?;
        let optimization_opportunities = self.analyze_optimization_opportunities(&variable_track).await?;
        let complexity_score = self.calculate_complexity_score(&variable_track).await;

        Ok(DataFlowAnalysis {
            variable_tracks: vec![variable_track],
            data_dependencies,
            lifetime_analysis,
            data_hazards,
            optimization_opportunities,
            complexity_score,
        })
    }

    async fn track_variable_usage(&self, code: &str, variable_name: &str) -> Result<VariableTrack, anyhow::Error> {
        self.variable_tracker.track_variable(code, variable_name).await
    }

    async fn find_data_dependencies(&self, code: &str, variable_name: &str) -> Result<Vec<DataDependency>, anyhow::Error> {
        self.dependency_analyzer.analyze_dependencies(code, variable_name).await
    }
}

impl DataFlowAnalyzer {
    async fn analyze_optimization_opportunities(&self, variable_track: &VariableTrack) -> Result<Vec<DataFlowOptimization>, anyhow::Error> {
        let mut opportunities = Vec::new();

        // Check for dead code elimination opportunities
        if self.has_unused_variables(variable_track).await {
            opportunities.push(DataFlowOptimization {
                optimization_type: DataFlowOptimizationType::DeadCodeElimination,
                variables_affected: vec![variable_track.variable_name.clone()],
                expected_benefit: "Reduced memory usage and improved performance".to_string(),
                implementation_difficulty: ImplementationDifficulty::Easy,
                code_transformation: None,
                educational_value: "Understanding variable lifetime helps eliminate unnecessary allocations".to_string(),
            });
        }

        // Check for constant propagation opportunities
        if self.is_constant_propagation_candidate(variable_track).await {
            opportunities.push(DataFlowOptimization {
                optimization_type: DataFlowOptimizationType::ConstantPropagation,
                variables_affected: vec![variable_track.variable_name.clone()],
                expected_benefit: "Compile-time optimization and reduced runtime overhead".to_string(),
                implementation_difficulty: ImplementationDifficulty::Medium,
                code_transformation: Some(CodeTransformation {
                    original_code: "let x = compute_value();".to_string(),
                    transformed_code: "const x: i32 = 42; // constant value".to_string(),
                    transformation_description: "Replace runtime computation with constant value".to_string(),
                    preserved_semantics: true,
                }),
                educational_value: "Constant propagation is a fundamental compiler optimization technique".to_string(),
            });
        }

        // Check for register allocation opportunities
        if self.is_register_allocation_candidate(variable_track).await {
            opportunities.push(DataFlowOptimization {
                optimization_type: DataFlowOptimizationType::RegisterAllocation,
                variables_affected: vec![variable_track.variable_name.clone()],
                expected_benefit: "Faster access and reduced memory traffic".to_string(),
                implementation_difficulty: ImplementationDifficulty::Hard,
                code_transformation: None,
                educational_value: "Register allocation demonstrates the trade-off between speed and memory usage".to_string(),
            });
        }

        Ok(opportunities)
    }

    async fn calculate_complexity_score(&self, variable_track: &VariableTrack) -> f64 {
        let usage_count = variable_track.usage_points.len();
        let flow_steps = variable_track.data_flow_path.len();
        let transformations = variable_track.transformation_chain.len();
        
        // Simple complexity calculation
        (usage_count as f64 * 1.0) + (flow_steps as f64 * 0.5) + (transformations as f64 * 0.3)
    }

    async fn has_unused_variables(&self, variable_track: &VariableTrack) -> bool {
        variable_track.usage_points.is_empty()
    }

    async fn is_constant_propagation_candidate(&self, variable_track: &VariableTrack) -> bool {
        // Check if variable is initialized once and never modified
        if variable_track.usage_points.len() == 1 {
            if let Some(first_usage) = variable_track.usage_points.first() {
                matches!(first_usage.usage_type, UsageType::Write)
            } else {
                false
            }
        } else {
            false
        }
    }

    async fn is_register_allocation_candidate(&self, variable_track: &VariableTrack) -> bool {
        // Variables used frequently in small scope are good register candidates
        let scope_size = variable_track.declaration.line_number..=
            variable_track.usage_points.last().map_or(variable_track.declaration.line_number, |p| p.location.line_number);
        let usage_frequency = variable_track.usage_points.len();
        
        usage_frequency > 5 && (scope_size.end - scope_size.start) < 20
    }
}

// Variable Tracker Implementation
pub struct VariableTracker {
    tracking_patterns: Vec<TrackingPattern>,
}

impl VariableTracker {
    fn new() -> Self {
        Self {
            tracking_patterns: vec![
                TrackingPattern {
                    pattern: Regex::new(r"let\s+(\w+)\s*(:\s*[^=]+)?\s*=").unwrap(),
                    tracking_type: TrackingType::Declaration,
                },
                TrackingPattern {
                    pattern: Regex::new(r"(\w+)\s*=").unwrap(),
                    tracking_type: TrackingType::Assignment,
                },
                TrackingPattern {
                    pattern: Regex::new(r"(\w+)\s*\.").unwrap(),
                    tracking_type: TrackingType::MethodCall,
                },
            ]
        }
    }

    async fn track_variable(&self, code: &str, variable_name: &str) -> Result<VariableTrack, anyhow::Error> {
        let lines: Vec<&str> = code.lines().collect();
        let mut usage_points = Vec::new();
        let mut data_flow_path = Vec::new();
        let mut transformation_chain = Vec::new();
        
        let mut declaration = DeclarationInfo {
            file_path: "kernel/src/main.rs".to_string(),
            line_number: 0,
            column: 0,
            is_mutable: false,
            initialization_value: None,
            inferred_type: None,
        };

        // Track variable usage across the code
        for (i, line) in lines.iter().enumerate() {
            let line_number = i as u32 + 1;
            
            // Check for variable assignment
            if let Some(captures) = Regex::new(r"(let|let mut)\s+(\w+)").unwrap().captures(line) {
                if captures[2] == variable_name {
                    let is_mutable = captures[1].contains("mut");
                    let inferred_type = Regex::new(r":\s*([^=]+)").unwrap()
                        .captures(line)
                        .map(|c| c[1].trim().to_string());
                    let initialization_value = Regex::new(r"=\s*([^;]+)")
                        .unwrap()
                        .captures(line)
                        .map(|c| c[1].trim().to_string());

                    declaration = DeclarationInfo {
                        file_path: "kernel/src/main.rs".to_string(),
                        line_number,
                        column: 0,
                        is_mutable,
                        initialization_value,
                        inferred_type,
                    };
                }
            }

            // Check for variable usage
            if line.contains(&variable_name) && !line.contains("let ") {
                let usage_type = if line.contains(&format!("{} =", variable_name)) {
                    UsageType::Write
                } else if line.contains(&format!("{}.", variable_name)) {
                    UsageType::MethodCall
                } else {
                    UsageType::Read
                };

                let usage_point = UsagePoint {
                    location: CodeLocation {
                        file_path: "kernel/src/main.rs".to_string(),
                        line_number,
                        column: 0,
                        function_name: self.extract_function_name(line, i, &lines).await,
                    },
                    usage_type,
                    access_mode: if declaration.is_mutable { AccessMode::ReadWrite } else { AccessMode::ReadOnly },
                    context: line.trim().to_string(),
                    side_effects: self.analyze_side_effects(line).await,
                };

                usage_points.push(usage_point);
            }
        }

        let scope_analysis = self.analyze_scope(&lines, &variable_name).await;
        
        Ok(VariableTrack {
            variable_name: variable_name.to_string(),
            variable_type: declaration.inferred_type.clone().unwrap_or("unknown".to_string()),
            declaration,
            usage_points,
            data_flow_path,
            transformation_chain,
            scope_analysis,
        })
    }

    async fn extract_function_name(&self, line: &str, line_idx: usize, lines: &[&str]) -> Option<String> {
        for i in (0..line_idx).rev() {
            if let Some(func_match) = Regex::new(r"fn\s+(\w+)").unwrap().captures(lines[i]) {
                return Some(func_match[1].to_string());
            }
        }
        None
    }

    async fn analyze_side_effects(&self, line: &str) -> Vec<SideEffect> {
        let mut side_effects = Vec::new();
        
        if line.contains("println!") || line.contains("print!") {
            side_effects.push(SideEffect::IOWrite);
        }
        if line.contains("=") {
            side_effects.push(SideEffect::StateMutation);
        }
        if line.contains("Vec::new()") || line.contains("Box::new(") {
            side_effects.push(SideEffect::MemoryAllocation);
        }
        
        side_effects
    }

    async fn analyze_scope(&self, lines: &[&str], variable_name: &str) -> ScopeAnalysis {
        let mut scope_level = ScopeLevel::Function;
        let mut enclosing_functions = Vec::new();
        let mut variable_shadowing = Vec::new();
        let mut lifetime_warnings = Vec::new();

        // Simplified scope analysis
        for (i, line) in lines.iter().enumerate() {
            if line.contains("fn ") {
                if let Some(func_match) = Regex::new(r"fn\s+(\w+)").unwrap().captures(line) {
                    enclosing_functions.push(func_match[1].to_string());
                }
            }
            
            if line.contains('{') {
                scope_level = ScopeLevel::Block;
            }
            
            // Check for potential shadowing
            if line.contains(&format!("let {} =", variable_name)) && line.contains("let") {
                variable_shadowing.push(ShadowInfo {
                    shadowing_variable: format!("{}_shadow_{}", variable_name, i),
                    shadowed_variable: variable_name.to_string(),
                    shadow_start: i as u32 + 1,
                    shadow_end: i as u32 + 1,
                    scope_level: 1,
                });
            }
        }

        ScopeAnalysis {
            scope_level,
            enclosing_functions,
            variable_shadowing,
            lifetime_warnings,
        }
    }
}

// Supporting Structures
#[derive(Debug)]
struct TrackingPattern {
    pattern: Regex,
    tracking_type: TrackingType,
}

#[derive(Debug, Clone)]
enum TrackingType {
    Declaration,
    Assignment,
    MethodCall,
}

pub struct DependencyAnalyzer {
    dependency_rules: Vec<DependencyRule>,
}

impl DependencyAnalyzer {
    fn new() -> Self {
        Self {
            dependency_rules: vec![
                DependencyRule {
                    pattern: Regex::new(r"(\w+)\s*=\s*(\w+)").unwrap(),
                    dependency_type: DependencyKind::Direct,
                    strength: DependencyStrength::Strong,
                },
                DependencyRule {
                    pattern: Regex::new(r"(\w+)\.([^(]+)\(").unwrap(),
                    dependency_type: DependencyKind::Control,
                    strength: DependencyStrength::Medium,
                },
            ]
        }
    }

    async fn analyze_dependencies(&self, code: &str, variable_name: &str) -> Result<Vec<DataDependency>, anyhow::Error> {
        let mut dependencies = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_number = i as u32 + 1;
            
            // Find variables that this variable depends on
            if let Some(captures) = Regex::new(r"(\w+)\s*=\s*([^;]+)").unwrap().captures(line) {
                if captures[1] == variable_name {
                    let expression = captures[2].trim();
                    
                    // Find variables in the expression
                    let dependent_vars: Vec<String> = Regex::new(r"\b(\w+)\b")
                        .unwrap()
                        .captures_iter(expression)
                        .filter_map(|cap| {
                            let var_name = cap[1].to_string();
                            if var_name != variable_name && !self.is_keyword(&var_name) {
                                Some(var_name)
                            } else {
                                None
                            }
                        })
                        .collect();

                    for dep_var in dependent_vars {
                        dependencies.push(DataDependency {
                            source_variable: dep_var,
                            target_variable: variable_name.to_string(),
                            dependency_type: DependencyKind::Direct,
                            strength: DependencyStrength::Strong,
                            path: vec![
                                FlowPathStep {
                                    from: dep_var.clone(),
                                    to: variable_name.to_string(),
                                    operation: "assignment".to_string(),
                                    location: CodeLocation {
                                        file_path: "kernel/src/main.rs".to_string(),
                                        line_number,
                                        column: 0,
                                        function_name: None,
                                    },
                                    conditions: vec![],
                                }
                            ],
                            potential_conflicts: vec![],
                            educational_context: format!("{} depends on {} through assignment", variable_name, dep_var),
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    fn is_keyword(&self, var_name: &str) -> bool {
        let keywords = ["let", "mut", "if", "else", "for", "while", "return", "fn", "struct", "enum"];
        keywords.contains(&var_name)
    }
}

pub struct LifetimeAnalyzer;

impl LifetimeAnalyzer {
    fn new() -> Self {
        Self
    }

    async fn analyze_lifetime(&self, code: &str, variable_name: &str) -> Result<Vec<LifetimeAnalysis>, anyhow::Error> {
        let lines: Vec<&str> = code.lines().collect();
        let mut scope_start = 0;
        let mut scope_end = lines.len() as u32 - 1;
        let mut declaration_line = 0;

        // Find declaration and scope
        for (i, line) in lines.iter().enumerate() {
            let line_number = i as u32 + 1;
            
            if line.contains(&format!("let {} =", variable_name)) || line.contains(&format!("let mut {} =", variable_name)) {
                declaration_line = line_number;
                scope_start = line_number;
                break;
            }
        }

        // Find scope end (simplified)
        let brace_count = Regex::new(r"\{").unwrap().find_iter(code).count();
        let closing_braces = Regex::new(r"\}").unwrap().find_iter(code).count();
        
        if brace_count == closing_braces {
            scope_end = lines.len() as u32;
        }

        let lifetime_analysis = LifetimeAnalysis {
            variable_name: variable_name.to_string(),
            scope_start,
            scope_end,
            lifetime_type: LifetimeType::Stack,
            storage_location: StorageLocation::Stack,
            allocation_strategy: AllocationStrategy::Automatic,
            deallocation_point: Some(scope_end),
            memory_pressure: MemoryPressure::Low,
        };

        Ok(vec![lifetime_analysis])
    }
}

pub struct HazardDetector {
    hazard_patterns: Vec<HazardPattern>,
}

impl HazardDetector {
    fn new() -> Self {
        Self {
            hazard_patterns: vec![
                HazardPattern {
                    pattern: Regex::new(r"(\w+)\[\s*(\w+)\s*\]").unwrap(),
                    hazard_type: HazardType::BufferOverflow,
                    severity: HazardSeverity::High,
                },
                HazardPattern {
                    pattern: Regex::new(r"\*\s*(\w+)").unwrap(),
                    hazard_type: HazardType::NullPointerDereference,
                    severity: HazardSeverity::Critical,
                },
            ]
        }
    }

    async fn detect_hazards(&self, code: &str, variable_name: &str) -> Result<Vec<DataHazard>, anyhow::Error> {
        let mut hazards = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_number = i as u32 + 1;
            
            for pattern in &self.hazard_patterns {
                if pattern.pattern.is_match(line) {
                    hazards.push(DataHazard {
                        hazard_type: pattern.hazard_type.clone(),
                        variables: vec![variable_name.to_string()],
                        location: CodeLocation {
                            file_path: "kernel/src/main.rs".to_string(),
                            line_number,
                            column: 0,
                            function_name: None,
                        },
                        severity: pattern.severity.clone(),
                        description: format!("Potential {} detected with variable {}", 
                            format!("{:?}", pattern.hazard_type).to_lowercase(), variable_name),
                        mitigation_strategies: self.get_mitigation_strategies(&pattern.hazard_type).await,
                        educational_implications: self.get_educational_implications(&pattern.hazard_type).await,
                    });
                }
            }
        }

        Ok(hazards)
    }

    async fn get_mitigation_strategies(&self, hazard_type: &HazardType) -> Vec<String> {
        match hazard_type {
            HazardType::BufferOverflow => vec![
                "Use bounds checking".to_string(),
                "Prefer safe collections".to_string(),
                "Validate input sizes".to_string(),
            ],
            HazardType::NullPointerDereference => vec![
                "Use Option<T> in Rust".to_string(),
                "Initialize all variables".to_string(),
                "Check for null before dereferencing".to_string(),
            ],
            HazardType::DataRace => vec![
                "Use mutexes or other synchronization".to_string(),
                "Ensure single writer, multiple readers".to_string(),
                "Use atomic operations where appropriate".to_string(),
            ],
            _ => vec!["Review code carefully".to_string()],
        }
    }

    async fn get_educational_implications(&self, hazard_type: &HazardType) -> String {
        match hazard_type {
            HazardType::BufferOverflow => 
                "Buffer overflows are common security vulnerabilities. Understanding memory safety is crucial for systems programming.".to_string(),
            HazardType::NullPointerDereference => 
                "Null pointer dereferences can cause crashes. Option types provide safer alternatives in languages like Rust.".to_string(),
            HazardType::DataRace => 
                "Data races occur when multiple threads access shared data without proper synchronization.".to_string(),
            _ => "Understanding these hazards helps write more robust and secure code.".to_string(),
        }
    }
}

#[derive(Debug)]
struct HazardPattern {
    pattern: Regex,
    hazard_type: HazardType,
    severity: HazardSeverity,
}

#[derive(Debug)]
struct DependencyRule {
    pattern: Regex,
    dependency_type: DependencyKind,
    strength: DependencyStrength,
}
