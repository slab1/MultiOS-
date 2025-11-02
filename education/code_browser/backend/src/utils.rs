use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Common data structures used across analyzers

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub signature: String,
    pub start_line: u32,
    pub end_line: u32,
    pub parameters: Vec<String>,
    pub return_type: String,
    pub complexity: u32,
    pub educational_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub file_path: String,
    pub line: u32,
    pub surrounding_code: String,
    pub previous_line: Option<String>,
    pub next_line: Option<String>,
    pub scope_info: ScopeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    pub current_function: Option<String>,
    pub enclosing_function: Option<String>,
    pub block_level: u32,
    pub variable_scope: HashMap<String, ScopeVariable>,
    pub active_macros: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeVariable {
    pub name: String,
    pub var_type: String,
    pub is_mutable: bool,
    pub line_declared: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalModule {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty_level: ComplexityLevel,
    pub estimated_time: String,
    pub prerequisites: Vec<String>,
    pub topics: Vec<String>,
    pub code_examples: Vec<CodeExample>,
    pub learning_objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub title: String,
    pub description: String,
    pub code: String,
    pub language: String,
    pub explanation: String,
    pub related_concepts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendType {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    Inlining,
    LoopOptimization,
    MemoryOptimization,
    AlgorithmImprovement,
    Parallelization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPotential {
    High,
    Medium,
    Low,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub match_text: String,
    pub context: String,
    pub result_type: SearchResultType,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResultType {
    Function,
    Variable,
    Type,
    Comment,
    String,
    SystemCall,
    Macro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub symbol_name: String,
    pub definition_location: CodeLocation,
    pub references: Vec<Reference>,
    pub symbol_info: SymbolInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub location: CodeLocation,
    pub context: String,
    pub reference_type: ReferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    Usage,
    Declaration,
    Modification,
    Call,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol_type: SymbolType,
    pub visibility: SymbolVisibility,
    pub documentation: Option<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    Variable,
    Type,
    Constant,
    Macro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolVisibility {
    Public,
    Private,
    Internal,
}

// Helper functions for common operations
pub struct CodeUtils;

impl CodeUtils {
    pub fn extract_line_numbers(text: &str) -> Vec<u32> {
        text.lines()
            .enumerate()
            .map(|(i, _)| (i + 1) as u32)
            .collect()
    }

    pub fn get_line_context(text: &str, line_number: u32, context_lines: u32) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let start = (line_number as usize).saturating_sub(context_lines as usize);
        let end = (line_number as usize + context_lines as usize).min(lines.len());
        
        lines[start..end].join("\n")
    }

    pub fn find_matching_braces(text: &str, line: u32, column: u32) -> Option<(u32, u32)> {
        // Simplified brace matching - would need more sophisticated implementation
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = (line - 1) as usize;
        
        if line_idx >= lines.len() {
            return None;
        }
        
        let current_line = lines[line_idx];
        let brace_pos = (column - 1) as usize;
        
        if brace_pos >= current_line.len() || current_line.chars().nth(brace_pos) != Some('{') {
            return None;
        }
        
        // Find matching closing brace
        let mut brace_count = 1;
        let mut search_line = line_idx;
        let mut search_col = brace_pos + 1;
        
        loop {
            if search_col >= current_line.len() {
                search_line += 1;
                if search_line >= lines.len() {
                    break;
                }
                search_col = 0;
            }
            
            let current_char = lines[search_line].chars().nth(search_col).unwrap_or('\0');
            
            match current_char {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        return Some((search_line as u32 + 1, search_col as u32 + 1));
                    }
                }
                _ => {}
            }
            
            search_col += 1;
        }
        
        None
    }

    pub fn calculate_indentation(line: &str) -> u32 {
        line.chars().take_while(|&c| c.is_whitespace()).count() as u32
    }

    pub fn extract_function_signature(function_line: &str) -> Option<(String, Vec<String>, String)> {
        // Simplified function signature extraction
        let pattern = regex::Regex::new(r"fn\s+(\w+)\s*\(([^)]*)\)\s*(->\s*[^({\s]+)?")?;
        
        if let Some(captures) = pattern.captures(function_line) {
            let name = captures[1].to_string();
            let params = if captures.len() > 2 && !captures[2].trim().is_empty() {
                captures[2].split(',').map(|s| s.trim().to_string()).collect()
            } else {
                Vec::new()
            };
            let return_type = captures.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "()".to_string());
            
            Some((name, params, return_type))
        } else {
            None
        }
    }

    pub fn is_valid_identifier(name: &str) -> bool {
        !name.is_empty() && 
        name.chars().all(|c| c.is_alphanumeric() || c == '_') &&
        !name.chars().next().unwrap_or('0').is_digit(10)
    }

    pub fn extract_variable_name(assignment_line: &str) -> Option<String> {
        let pattern = regex::Regex::new(r"(?:let|let\s+mut)\s+(\w+)")?;
        if let Some(captures) = pattern.captures(assignment_line) {
            Some(captures[1].to_string())
        } else {
            None
        }
    }

    pub fn get_code_snippet(text: &str, start_line: u32, end_line: u32) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let start_idx = (start_line - 1) as usize;
        let end_idx = (end_line - 1).min(lines.len() as u32 - 1) as usize;
        
        if start_idx < lines.len() && start_idx <= end_idx {
            lines[start_idx..=end_idx].join("\n")
        } else {
            String::new()
        }
    }

    pub fn find_function_bounds(text: &str, start_line: u32) -> Option<(u32, u32)> {
        let lines: Vec<&str> = text.lines().collect();
        let mut brace_count = 0;
        let mut in_function = false;
        let mut function_start = 0;
        let mut function_end = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if i + 1 == start_line as usize {
                if line.contains('{') {
                    in_function = true;
                    function_start = i as u32 + 1;
                    brace_count = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                } else {
                    return None;
                }
            } else if in_function {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                if brace_count == 0 {
                    function_end = i as u32 + 1;
                    break;
                }
            }
        }
        
        if in_function && function_end > 0 {
            Some((function_start, function_end))
        } else {
            None
        }
    }

    pub fn get_educational_content(concept: &str) -> Option<EducationalContent> {
        let educational_db = HashMap::from([
            ("system_call".to_string(), EducationalContent {
                concept: "System Calls".to_string(),
                explanation: "System calls are the fundamental interface between a process and the kernel. They provide controlled access to hardware resources and privileged operations.".to_string(),
                examples: vec![
                    "read() - read data from a file descriptor".to_string(),
                    "write() - write data to a file descriptor".to_string(),
                    "fork() - create a new process".to_string(),
                ],
                related_concepts: vec!["kernel interface".to_string(), "process isolation".to_string(), "privilege levels".to_string()],
                best_practices: vec![
                    "Check return values for errors".to_string(),
                    "Minimize the number of system calls".to_string(),
                    "Use buffered I/O when appropriate".to_string(),
                ],
            }),
            ("memory_management".to_string(), EducationalContent {
                concept: "Memory Management".to_string(),
                explanation: "Memory management is a core kernel responsibility that involves allocating, deallocating, and organizing memory for processes and kernel data structures.".to_string(),
                examples: vec![
                    "Virtual memory mapping".to_string(),
                    "Page table management".to_string(),
                    "Memory allocation strategies".to_string(),
                ],
                related_concepts: vec!["virtual memory".to_string(), "page tables".to_string(), "memory protection".to_string()],
                best_practices: vec![
                    "Use appropriate allocation strategies".to_string(),
                    "Implement memory pooling for performance".to_string(),
                    "Handle allocation failures gracefully".to_string(),
                ],
            }),
            ("process_scheduling".to_string(), EducationalContent {
                concept: "Process Scheduling".to_string(),
                explanation: "Process scheduling determines which process runs on the CPU at any given time, managing the fair and efficient sharing of CPU resources among competing processes.".to_string(),
                examples: vec![
                    "Round-robin scheduling".to_string(),
                    "Priority-based scheduling".to_string(),
                    "Multilevel feedback queues".to_string(),
                ],
                related_concepts: vec!["context switching".to_string(), "time slices".to_string(), "process priorities".to_string()],
                best_practices: vec![
                    "Consider both fairness and performance".to_string(),
                    "Implement preemption carefully".to_string(),
                    "Use appropriate scheduling algorithms for workloads".to_string(),
                ],
            }),
        ]);

        educational_db.get(concept).cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalContent {
    pub concept: String,
    pub explanation: String,
    pub examples: Vec<String>,
    pub related_concepts: Vec<String>,
    pub best_practices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub cache_hit_rate: f64,
    pub context_switches: u32,
    pub system_calls_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    pub complexity: u32,
    pub cyclomatic_complexity: u32,
    pub maintainability_index: f64,
    pub technical_debt: f64,
    pub test_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugIntegration {
    pub breakpoints: Vec<Breakpoint>,
    pub watchpoints: Vec<Watchpoint>,
    pub current_execution_point: Option<ExecutionPoint>,
    pub call_stack: Vec<StackFrame>,
    pub variable_state: HashMap<String, VariableState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub file_path: String,
    pub line_number: u32,
    pub condition: Option<String>,
    pub is_enabled: bool,
    pub hit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    pub variable_name: String,
    pub watch_type: WatchType,
    pub condition: Option<String>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchType {
    Write,
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPoint {
    pub file_path: String,
    pub line_number: u32,
    pub function_name: String,
    pub instruction_pointer: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file_path: String,
    pub line_number: u32,
    pub local_variables: HashMap<String, VariableState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableState {
    pub name: String,
    pub value: String,
    pub var_type: String,
    pub is_modified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex_pattern: bool,
    pub include_comments: bool,
    pub file_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub file_type: Option<String>,
    pub complexity_threshold: Option<u32>,
    pub performance_threshold: Option<f64>,
    pub educational_level: Option<ComplexityLevel>,
    pub exclude_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub layout_type: LayoutType,
    pub color_scheme: ColorScheme,
    pub show_labels: bool,
    pub animation_enabled: bool,
    pub max_nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    Hierarchical,
    ForceDirected,
    Circular,
    Tree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Educational,
    Performance,
    Complexity,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveFeature {
    pub feature_id: String,
    pub name: String,
    pub description: String,
    pub is_enabled: bool,
    pub configuration: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPath {
    pub path_id: String,
    pub title: String,
    pub description: String,
    pub difficulty_progression: Vec<ComplexityLevel>,
    pub modules: Vec<String>,
    pub estimated_completion_time: String,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub assessment_id: String,
    pub title: String,
    pub questions: Vec<AssessmentQuestion>,
    pub passing_score: f64,
    pub time_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentQuestion {
    pub question_id: String,
    pub question_text: String,
    pub question_type: QuestionType,
    pub options: Vec<String>,
    pub correct_answer: String,
    pub explanation: String,
    pub difficulty: ComplexityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    MultipleChoice,
    TrueFalse,
    CodeAnalysis,
    ConceptExplanation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressTracking {
    pub user_id: String,
    pub modules_completed: Vec<String>,
    pub current_module: Option<String>,
    pub progress_percentage: f64,
    pub time_spent: u64,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlocked_at: chrono::DateTime<chrono::Utc>,
}
