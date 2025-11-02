use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::utils::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub syntax_highlighting: Vec<SyntaxHighlight>,
    pub functions: Vec<FunctionInfo>,
    pub variables: Vec<VariableInfo>,
    pub types: Vec<TypeInfo>,
    pub imports: Vec<ImportInfo>,
    pub inline_explanations: Vec<InlineExplanation>,
    pub complexity_score: u32,
    pub educational_comments: Vec<EducationalComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxHighlight {
    pub line: u32,
    pub start_col: u32,
    pub end_col: u32,
    pub token_type: String,
    pub token_value: String,
}

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
pub struct VariableInfo {
    pub name: String,
    pub var_type: String,
    pub line: u32,
    pub scope: String,
    pub is_mutable: bool,
    pub initialized_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub definition: String,
    pub line: u32,
    pub fields: Vec<TypeField>,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeField {
    pub name: String,
    pub field_type: String,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    pub module: String,
    pub items: Vec<String>,
    pub is_external: bool,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineExplanation {
    pub line: u32,
    pub start_col: u32,
    pub end_col: u32,
    pub explanation: String,
    pub complexity_level: ComplexityLevel,
    pub related_concepts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationalComment {
    pub line: u32,
    pub comment: String,
    pub category: CommentCategory,
    pub difficulty_level: ComplexityLevel,
    pub learning_objectives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub line: u32,
    pub column: u32,
    pub suggestion_type: SuggestionType,
    pub message: String,
    pub severity: Severity,
    pub fix_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentCategory {
    Concept,
    BestPractice,
    Warning,
    Performance,
    Security,
    Educational,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    SyntaxError,
    StyleSuggestion,
    PerformanceHint,
    SecurityWarning,
    BestPractice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_path: String,
    pub line: u32,
    pub match_text: String,
    pub context: String,
    pub result_type: ResultType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultType {
    Function,
    Variable,
    Type,
    Comment,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationLocation {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub symbol_type: String,
    pub definition_line: Option<u32>,
    pub references: Vec<ReferenceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceInfo {
    pub file_path: String,
    pub line: u32,
    pub context: String,
}

#[async_trait]
pub trait CodeAnalysisTrait {
    async fn analyze_code(&self, code: &str, language: &str) -> Result<CodeAnalysis, anyhow::Error>;
    async fn get_code_suggestions(&self, code: &str, language: &str) -> Result<Vec<CodeSuggestion>, anyhow::Error>;
    async fn get_educational_comments(&self, code: &str, language: &str) -> Result<Vec<EducationalComment>, anyhow::Error>;
    async fn search_code(&self, query: &str, file_path: Option<&str>, language: Option<&str>) -> Result<Vec<SearchResult>, anyhow::Error>;
    async fn navigate_to_symbol(&self, symbol_name: &str, file_path: &str, line_number: Option<u32>) -> Result<NavigationLocation, anyhow::Error>;
    async fn get_code_context(&self, file_path: &str, line_number: Option<u32>) -> Result<CodeContext, anyhow::Error>;
}

pub struct CodeAnalyzer {
    syntax_highlighters: HashMap<String, Arc<dyn SyntaxHighlighter + Send + Sync>>,
    educational_knowledge: EducationalKnowledge,
    pattern_matcher: PatternMatcher,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        let mut syntax_highlighters = HashMap::new();
        syntax_highlighters.insert("rust".to_string(), Arc::new(RustHighlighter));
        syntax_highlighters.insert("c".to_string(), Arc::new(CHighlighter));
        syntax_highlighters.insert("cpp".to_string(), Arc::new(CppHighlighter));
        syntax_highlighters.insert("assembly".to_string(), Arc::new(AssemblyHighlighter));

        Self {
            syntax_highlighters,
            educational_knowledge: EducationalKnowledge::new(),
            pattern_matcher: PatternMatcher::new(),
        }
    }
}

#[async_trait]
impl CodeAnalysisTrait for CodeAnalyzer {
    async fn analyze_code(&self, code: &str, language: &str) -> Result<CodeAnalysis, anyhow::Error> {
        let syntax_highlighter = self.syntax_highlighters.get(language)
            .ok_or_else(|| anyhow::Error::msg(format!("Unsupported language: {}", language)))?;

        let syntax_highlighting = syntax_highlighter.highlight_code(code).await?;
        let functions = self.extract_functions(code, language).await?;
        let variables = self.extract_variables(code, language).await?;
        let types = self.extract_types(code, language).await?;
        let imports = self.extract_imports(code, language).await?;
        let inline_explanations = self.generate_inline_explanations(code, language).await?;
        let complexity_score = self.calculate_complexity(code).await?;
        let educational_comments = self.generate_educational_comments(code, language).await?;

        Ok(CodeAnalysis {
            syntax_highlighting,
            functions,
            variables,
            types,
            imports,
            inline_explanations,
            complexity_score,
            educational_comments,
        })
    }

    async fn get_code_suggestions(&self, code: &str, language: &str) -> Result<Vec<CodeSuggestion>, anyhow::Error> {
        let mut suggestions = Vec::new();

        // Pattern-based suggestions
        suggestions.extend(self.pattern_matcher.find_pattern_issues(code).await?);

        // Language-specific suggestions
        suggestions.extend(self.get_language_specific_suggestions(code, language).await?);

        Ok(suggestions)
    }

    async fn get_educational_comments(&self, code: &str, language: &str) -> Result<Vec<EducationalComment>, anyhow::Error> {
        self.generate_educational_comments(code, language).await
    }

    async fn search_code(&self, query: &str, file_path: Option<&str>, language: Option<&str>) -> Result<Vec<SearchResult>, anyhow::Error> {
        // Implementation for searching across codebase
        let mut results = Vec::new();
        
        // This would typically search through the loaded codebase
        // For now, returning mock data
        results.push(SearchResult {
            file_path: file_path.unwrap_or("kernel/src/main.rs").to_string(),
            line: 42,
            match_text: query.to_string(),
            context: "fn main() { // This is the main function".to_string(),
            result_type: ResultType::Function,
        });

        Ok(results)
    }

    async fn navigate_to_symbol(&self, symbol_name: &str, file_path: &str, line_number: Option<u32>) -> Result<NavigationLocation, anyhow::Error> {
        Ok(NavigationLocation {
            file_path: file_path.to_string(),
            line: line_number.unwrap_or(1),
            column: 0,
            symbol_type: "function".to_string(),
            definition_line: Some(line_number.unwrap_or(1)),
            references: vec![],
        })
    }

    async fn get_code_context(&self, file_path: &str, line_number: Option<u32>) -> Result<CodeContext, anyhow::Error> {
        Ok(CodeContext {
            file_path: file_path.to_string(),
            line: line_number.unwrap_or(1),
            surrounding_code: "fn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
            previous_line: None,
            next_line: Some("}".to_string()),
            scope_info: ScopeInfo::default(),
        })
    }
}

impl CodeAnalyzer {
    async fn extract_functions(&self, code: &str, language: &str) -> Result<Vec<FunctionInfo>, anyhow::Error> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        match language {
            "rust" => {
                let function_pattern = Regex::new(r"fn\s+(\w+)\s*\(([^)]*)\)\s*(->\s*[^({\s]+)?")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = function_pattern.captures(line) {
                        let name = captures[1].to_string();
                        let params = if captures.len() > 2 && !captures[2].trim().is_empty() {
                            captures[2].split(',').map(|s| s.trim().to_string()).collect()
                        } else {
                            Vec::new()
                        };
                        let return_type = captures.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "()".to_string());
                        
                        functions.push(FunctionInfo {
                            name,
                            signature: line.trim().to_string(),
                            start_line: i as u32 + 1,
                            end_line: self.find_function_end(&lines, i),
                            parameters: params,
                            return_type,
                            complexity: self.calculate_function_complexity(line),
                            educational_description: self.educational_knowledge.get_function_description(&name).await,
                        });
                    }
                }
            }
            "c" | "cpp" => {
                let function_pattern = Regex::new(r"(\w+\s*[*]?\s+)?(\w+)\s*\(([^)]*)\)\s*(const\s*)?\{?")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = function_pattern.captures(line) {
                        let name = captures[2].to_string();
                        let return_type = captures[1].trim().to_string();
                        let params = if captures.len() > 3 && !captures[3].trim().is_empty() {
                            captures[3].split(',').map(|s| s.trim().to_string()).collect()
                        } else {
                            Vec::new()
                        };

                        functions.push(FunctionInfo {
                            name,
                            signature: line.trim().to_string(),
                            start_line: i as u32 + 1,
                            end_line: self.find_function_end(&lines, i),
                            parameters: params,
                            return_type,
                            complexity: self.calculate_function_complexity(line),
                            educational_description: self.educational_knowledge.get_function_description(&name).await,
                        });
                    }
                }
            }
            _ => {
                // Default pattern for other languages
                return Ok(functions);
            }
        }

        Ok(functions)
    }

    async fn extract_variables(&self, code: &str, language: &str) -> Result<Vec<VariableInfo>, anyhow::Error> {
        let mut variables = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        match language {
            "rust" => {
                let var_pattern = Regex::new(r"(let|let mut)\s+(\w+)\s*(:\s*[^=]+)?\s*=\s*([^;]+)")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = var_pattern.captures(line) {
                        let is_mutable = captures[1].contains("mut");
                        let name = captures[2].to_string();
                        let var_type = captures.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_else(|| "infer".to_string());
                        let initialized_value = Some(captures[4].to_string());

                        variables.push(VariableInfo {
                            name,
                            var_type,
                            line: i as u32 + 1,
                            scope: self.determine_variable_scope(line, i),
                            is_mutable,
                            initialized_value,
                        });
                    }
                }
            }
            "c" | "cpp" => {
                let var_pattern = Regex::new(r"(\w+\s*[*]?\s+)(const\s+)?(\w+)\s*=\s*([^;]+)")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = var_pattern.captures(line) {
                        let var_type = captures[1].trim().to_string();
                        let name = captures[3].to_string();
                        let initialized_value = Some(captures[4].to_string());
                        let is_mutable = !captures.get(2).map(|m| m.as_str().contains("const")).unwrap_or(false);

                        variables.push(VariableInfo {
                            name,
                            var_type,
                            line: i as u32 + 1,
                            scope: self.determine_variable_scope(line, i),
                            is_mutable,
                            initialized_value,
                        });
                    }
                }
            }
            _ => {
                return Ok(variables);
            }
        }

        Ok(variables)
    }

    async fn extract_types(&self, code: &str, language: &str) -> Result<Vec<TypeInfo>, anyhow::Error> {
        let mut types = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        match language {
            "rust" => {
                // Struct definitions
                let struct_pattern = Regex::new(r"struct\s+(\w+)\s*\{([^}]+)\}")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = struct_pattern.captures(line) {
                        let name = captures[1].to_string();
                        let definition = line.trim().to_string();
                        let mut fields = Vec::new();
                        
                        let fields_text = captures[2].to_string();
                        for field_line in fields_text.lines() {
                            let field_pattern = Regex::new(r"(\w+)\s*:\s*([^,]+)")?;
                            if let Some(field_captures) = field_pattern.captures(field_line) {
                                fields.push(TypeField {
                                    name: field_captures[1].to_string(),
                                    field_type: field_captures[2].trim().to_string(),
                                    is_public: field_line.trim().starts_with("pub"),
                                });
                            }
                        }

                        types.push(TypeInfo {
                            name,
                            definition,
                            line: i as u32 + 1,
                            fields,
                            is_builtin: false,
                        });
                    }
                }

                // Enum definitions
                let enum_pattern = Regex::new(r"enum\s+(\w+)\s*\{([^}]+)\}")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = enum_pattern.captures(line) {
                        let name = captures[1].to_string();
                        let definition = line.trim().to_string();

                        types.push(TypeInfo {
                            name,
                            definition,
                            line: i as u32 + 1,
                            fields: vec![], // Enums don't have fields in the same way
                            is_builtin: false,
                        });
                    }
                }
            }
            _ => {
                return Ok(types);
            }
        }

        Ok(types)
    }

    async fn extract_imports(&self, code: &str, language: &str) -> Result<Vec<ImportInfo>, anyhow::Error> {
        let mut imports = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        match language {
            "rust" => {
                let use_pattern = Regex::new(r"use\s+([^;]+)")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = use_pattern.captures(line) {
                        let module_path = captures[1].to_string();
                        let is_external = module_path.starts_with("std::") || 
                                         module_path.starts_with("crate::") ||
                                         module_path.starts_with("super::") ||
                                         module_path.starts_with("self::");

                        imports.push(ImportInfo {
                            module: module_path,
                            items: vec![], // Could parse specific items imported
                            is_external,
                            line: i as u32 + 1,
                        });
                    }
                }
            }
            "c" | "cpp" => {
                let include_pattern = Regex::new(r"#include\s*[<\"]([^>\"]+)[>\"]")?;
                for (i, line) in lines.iter().enumerate() {
                    if let Some(captures) = include_pattern.captures(line) {
                        let header = captures[1].to_string();
                        let is_external = header.starts_with('<');

                        imports.push(ImportInfo {
                            module: header,
                            items: vec![],
                            is_external,
                            line: i as u32 + 1,
                        });
                    }
                }
            }
            _ => {
                return Ok(imports);
            }
        }

        Ok(imports)
    }

    async fn generate_inline_explanations(&self, code: &str, language: &str) -> Result<Vec<InlineExplanation>, anyhow::Error> {
        let mut explanations = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;
            
            // System call explanations
            if line.contains("syscall") || line.contains("int 0x80") || line.contains("svc") {
                explanations.push(InlineExplanation {
                    line: line_num,
                    start_col: self.find_keyword_position(line, "syscall"),
                    end_col: self.find_keyword_position(line, "syscall") + 7,
                    explanation: "System call - transfers control to the kernel to perform privileged operations".to_string(),
                    complexity_level: ComplexityLevel::Intermediate,
                    related_concepts: vec!["kernel".to_string(), "privileged operations".to_string(), "system interface".to_string()],
                });
            }

            // Memory management explanations
            if line.contains("mmap") || line.contains("malloc") || line.contains("virtual_memory") {
                explanations.push(InlineExplanation {
                    line: line_num,
                    start_col: 0,
                    end_col: line.len() as u32,
                    explanation: "Memory management operation - allocates or maps virtual memory regions".to_string(),
                    complexity_level: ComplexityLevel::Advanced,
                    related_concepts: vec!["virtual memory".to_string(), "page tables".to_string(), "memory allocation".to_string()],
                });
            }

            // Interrupt handling explanations
            if line.contains("interrupt") || line.contains("handler") || line.contains("irq") {
                explanations.push(InlineExplanation {
                    line: line_num,
                    start_col: self.find_keyword_position(line, "interrupt"),
                    end_col: self.find_keyword_position(line, "interrupt") + 9,
                    explanation: "Interrupt handling - asynchronous signal from hardware or software requiring immediate attention".to_string(),
                    complexity_level: ComplexityLevel::Advanced,
                    related_concepts: vec!["interrupt controller".to_string(), "context switching".to_string(), "hardware signals".to_string()],
                });
            }

            // Context switching explanations
            if line.contains("context_switch") || line.contains("switch_to") {
                explanations.push(InlineExplanation {
                    line: line_num,
                    start_col: self.find_keyword_position(line, "context_switch"),
                    end_col: self.find_keyword_position(line, "context_switch") + 13,
                    explanation: "Context switch - saves current process state and loads new process state for multitasking".to_string(),
                    complexity_level: ComplexityLevel::Expert,
                    related_concepts: vec!["process scheduling".to_string(), "CPU registers".to_string(), "task state".to_string()],
                });
            }
        }

        Ok(explanations)
    }

    async fn calculate_complexity(&self, code: &str) -> Result<u32, anyhow::Error> {
        let lines: Vec<&str> = code.lines().collect();
        let mut complexity = 0;

        for line in lines {
            // Count control structures
            if line.contains("if") || line.contains("while") || line.contains("for") {
                complexity += 1;
            }
            if line.contains("match") {
                complexity += 2; // Match statements are more complex
            }
            if line.contains("fn ") {
                complexity += 3; // Function definitions add significant complexity
            }
            // Count nested braces as complexity
            let nested_braces: Vec<_> = line.chars().filter(|&c| c == '{').collect();
            complexity += nested_braces.len() as u32;
        }

        Ok(complexity)
    }

    async fn generate_educational_comments(&self, code: &str, language: &str) -> Result<Vec<EducationalComment>, anyhow::Error> {
        let mut comments = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;

            // Add educational comments based on code patterns
            if line.contains("unsafe") {
                comments.push(EducationalComment {
                    line: line_num,
                    comment: "unsafe keyword bypasses Rust's safety guarantees - use carefully when interfacing with low-level code".to_string(),
                    category: CommentCategory::Warning,
                    difficulty_level: ComplexityLevel::Advanced,
                    learning_objectives: vec!["memory safety".to_string(), "unsafe Rust".to_string(), "FFI".to_string()],
                });
            }

            if line.contains("thread::spawn") {
                comments.push(EducationalComment {
                    line: line_num,
                    comment: "Thread creation - enables concurrent execution in the kernel".to_string(),
                    category: CommentCategory::Concept,
                    difficulty_level: ComplexityLevel::Intermediate,
                    learning_objectives: vec!["concurrency".to_string(), "threading".to_string(), "parallelism".to_string()],
                });
            }

            if line.contains("Mutex") {
                comments.push(EducationalComment {
                    line: line_num,
                    comment: "Mutual exclusion primitive - prevents race conditions in concurrent access".to_string(),
                    category: CommentCategory::Concept,
                    difficulty_level: ComplexityLevel::Advanced,
                    learning_objectives: vec!["synchronization".to_string(), "race conditions".to_string(), "concurrency".to_string()],
                });
            }
        }

        Ok(comments)
    }

    async fn get_language_specific_suggestions(&self, code: &str, language: &str) -> Result<Vec<CodeSuggestion>, anyhow::Error> {
        let mut suggestions = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        match language {
            "rust" => {
                for (i, line) in lines.iter().enumerate() {
                    // Suggest using Result for error handling instead of unwrap
                    if line.contains(".unwrap()") {
                        suggestions.push(CodeSuggestion {
                            line: i as u32 + 1,
                            column: 0,
                            suggestion_type: SuggestionType::BestPractice,
                            message: "Consider using ? operator or proper error handling instead of unwrap()".to_string(),
                            severity: Severity::Warning,
                            fix_suggestion: Some("Use ? operator for cleaner error propagation".to_string()),
                        });
                    }

                    // Suggest using Cow for temporary allocations
                    if line.contains("String::from(") && line.contains("\"") {
                        suggestions.push(CodeSuggestion {
                            line: i as u32 + 1,
                            column: 0,
                            suggestion_type: SuggestionType::PerformanceHint,
                            message: "Consider using string literals directly or Cow for performance".to_string(),
                            severity: Severity::Info,
                            fix_suggestion: Some("Use &str instead of String::from() for string literals".to_string()),
                        });
                    }
                }
            }
            _ => {
                // Add other language-specific suggestions
            }
        }

        Ok(suggestions)
    }
}

// Helper methods
impl CodeAnalyzer {
    fn find_function_end(&self, lines: &[&str], start_idx: usize) -> u32 {
        let mut brace_count = 0;
        let mut in_function = false;

        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    in_function = true;
                } else if ch == '}' {
                    brace_count -= 1;
                    if in_function && brace_count == 0 {
                        return i as u32 + 1;
                    }
                }
            }
        }

        (start_idx as u32) + 1
    }

    fn calculate_function_complexity(&self, function_line: &str) -> u32 {
        let mut complexity = 1; // Base complexity for the function itself

        if function_line.contains("if ") {
            complexity += 1;
        }
        if function_line.contains("match ") {
            complexity += 2;
        }
        if function_line.contains("for ") {
            complexity += 1;
        }
        if function_line.contains("while ") {
            complexity += 1;
        }

        complexity
    }

    fn determine_variable_scope(&self, line: &str, line_idx: usize) -> String {
        let indent_level = line.chars().take_while(|&c| c.is_whitespace()).count();
        if indent_level == 0 {
            "global".to_string()
        } else if indent_level <= 4 {
            "function".to_string()
        } else {
            "block".to_string()
        }
    }

    fn find_keyword_position(&self, line: &str, keyword: &str) -> u32 {
        if let Some(pos) = line.find(keyword) {
            pos as u32
        } else {
            0
        }
    }
}

// Syntax Highlighter Implementations
#[async_trait]
pub trait SyntaxHighlighter {
    async fn highlight_code(&self, code: &str) -> Result<Vec<SyntaxHighlight>, anyhow::Error>;
}

pub struct RustHighlighter;

#[async_trait]
impl SyntaxHighlighter for RustHighlighter {
    async fn highlight_code(&self, code: &str) -> Result<Vec<SyntaxHighlight>, anyhow::Error> {
        let mut highlights = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;
            
            // Keywords
            let keywords = ["fn", "let", "mut", "struct", "enum", "impl", "trait", "match", "if", "else", "while", "for", "return", "unsafe"];
            for keyword in &keywords {
                if let Some(pos) = line.find(keyword) {
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: pos as u32,
                        end_col: (pos + keyword.len()) as u32,
                        token_type: "keyword".to_string(),
                        token_value: keyword.to_string(),
                    });
                }
            }

            // Comments
            if line.trim_start().starts_with("//") {
                highlights.push(SyntaxHighlight {
                    line: line_num,
                    start_col: line.find("//").unwrap_or(0) as u32,
                    end_col: line.len() as u32,
                    token_type: "comment".to_string(),
                    token_value: line.trim_start_matches("//").to_string(),
                });
            }

            // Strings
            let string_pattern = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)?;
            for capture in string_pattern.captures_iter(line) {
                let full_match = capture.get(0).unwrap();
                highlights.push(SyntaxHighlight {
                    line: line_num,
                    start_col: full_match.start() as u32,
                    end_col: full_match.end() as u32,
                    token_type: "string".to_string(),
                    token_value: full_match.as_str().to_string(),
                });
            }

            // Function names
            if let Some(pos) = line.find("fn ") {
                let remaining = &line[pos + 3..];
                if let Some(name_end) = remaining.find('(') {
                    let func_name = &remaining[..name_end];
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: (pos + 3) as u32,
                        end_col: (pos + 3 + func_name.len()) as u32,
                        token_type: "function".to_string(),
                        token_value: func_name.to_string(),
                    });
                }
            }
        }

        Ok(highlights)
    }
}

pub struct CHighlighter;

#[async_trait]
impl SyntaxHighlighter for CHighlighter {
    async fn highlight_code(&self, code: &str) -> Result<Vec<SyntaxHighlight>, anyhow::Error> {
        let mut highlights = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;
            
            // Keywords
            let keywords = ["int", "char", "float", "double", "void", "struct", "enum", "if", "else", "while", "for", "return", "typedef", "static", "extern"];
            for keyword in &keywords {
                if let Some(pos) = line.find(keyword) {
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: pos as u32,
                        end_col: (pos + keyword.len()) as u32,
                        token_type: "keyword".to_string(),
                        token_value: keyword.to_string(),
                    });
                }
            }

            // Comments
            if line.trim_start().starts_with("//") {
                highlights.push(SyntaxHighlight {
                    line: line_num,
                    start_col: line.find("//").unwrap_or(0) as u32,
                    end_col: line.len() as u32,
                    token_type: "comment".to_string(),
                    token_value: line.trim_start_matches("//").to_string(),
                });
            }

            // Strings
            let string_pattern = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)""#)?;
            for capture in string_pattern.captures_iter(line) {
                let full_match = capture.get(0).unwrap();
                highlights.push(SyntaxHighlight {
                    line: line_num,
                    start_col: full_match.start() as u32,
                    end_col: full_match.end() as u32,
                    token_type: "string".to_string(),
                    token_value: full_match.as_str().to_string(),
                });
            }
        }

        Ok(highlights)
    }
}

pub struct CppHighlighter;

#[async_trait]
impl SyntaxHighlighter for CppHighlighter {
    async fn highlight_code(&self, code: &str) -> Result<Vec<SyntaxHighlight>, anyhow::Error> {
        let mut highlights = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;
            
            // C++ specific keywords
            let keywords = ["class", "template", "namespace", "using", "public", "private", "protected", "virtual", "override", "new", "delete"];
            for keyword in &keywords {
                if let Some(pos) = line.find(keyword) {
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: pos as u32,
                        end_col: (pos + keyword.len()) as u32,
                        token_type: "keyword".to_string(),
                        token_value: keyword.to_string(),
                    });
                }
            }

            // Also include C keywords
            let c_keywords = ["int", "char", "float", "double", "void", "struct", "enum", "if", "else", "while", "for", "return", "typedef", "static", "extern"];
            for keyword in &c_keywords {
                if let Some(pos) = line.find(keyword) {
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: pos as u32,
                        end_col: (pos + keyword.len()) as u32,
                        token_type: "keyword".to_string(),
                        token_value: keyword.to_string(),
                    });
                }
            }
        }

        Ok(highlights)
    }
}

pub struct AssemblyHighlighter;

#[async_trait]
impl SyntaxHighlighter for AssemblyHighlighter {
    async fn highlight_code(&self, code: &str) -> Result<Vec<SyntaxHighlight>, anyhow::Error> {
        let mut highlights = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;
            
            // Assembly instructions
            let instructions = ["mov", "add", "sub", "mul", "div", "push", "pop", "call", "ret", "jmp", "je", "jne", "jl", "jg"];
            for instruction in &instructions {
                if let Some(pos) = line.find(instruction) {
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: pos as u32,
                        end_col: (pos + instruction.len()) as u32,
                        token_type: "instruction".to_string(),
                        token_value: instruction.to_string(),
                    });
                }
            }

            // Registers
            let registers = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"];
            for reg in &registers {
                if let Some(pos) = line.find(reg) {
                    highlights.push(SyntaxHighlight {
                        line: line_num,
                        start_col: pos as u32,
                        end_col: (pos + reg.len()) as u32,
                        token_type: "register".to_string(),
                        token_value: reg.to_string(),
                    });
                }
            }

            // Comments (semicolon)
            if let Some(pos) = line.find(';') {
                highlights.push(SyntaxHighlight {
                    line: line_num,
                    start_col: pos as u32,
                    end_col: line.len() as u32,
                    token_type: "comment".to_string(),
                    token_value: line[pos..].to_string(),
                });
            }
        }

        Ok(highlights)
    }
}

// Supporting data structures
pub struct EducationalKnowledge {
    function_descriptions: HashMap<String, String>,
}

impl EducationalKnowledge {
    fn new() -> Self {
        let mut function_descriptions = HashMap::new();
        function_descriptions.insert(
            "main".to_string(),
            "Main entry point - initializes kernel and starts system services".to_string(),
        );
        function_descriptions.insert(
            "syscall_handler".to_string(),
            "System call handler - processes user requests for kernel services".to_string(),
        );
        function_descriptions.insert(
            "interrupt_handler".to_string(),
            "Interrupt handler - responds to hardware and software interrupts".to_string(),
        );
        function_descriptions.insert(
            "memory_allocate".to_string(),
            "Memory allocator - manages dynamic memory allocation in the kernel".to_string(),
        );
        function_descriptions.insert(
            "process_sched".to_string(),
            "Process scheduler - determines which process runs next on CPU".to_string(),
        );
        function_descriptions.insert(
            "context_switch".to_string(),
            "Context switcher - saves and restores process execution state".to_string(),
        );

        Self {
            function_descriptions,
        }
    }

    async fn get_function_description(&self, function_name: &str) -> Option<String> {
        self.function_descriptions.get(function_name).cloned()
    }
}

pub struct PatternMatcher {
    patterns: Vec<PatternInfo>,
}

impl PatternMatcher {
    fn new() -> Self {
        let patterns = vec![
            PatternInfo {
                pattern: Regex::new(r"unwrap\(\)").unwrap(),
                suggestion: "Consider using ? operator for error handling".to_string(),
                severity: Severity::Warning,
            },
            PatternInfo {
                pattern: Regex::new(r"clone\(\)").unwrap(),
                suggestion: "Consider using references or copying strategies".to_string(),
                severity: Severity::Info,
            },
            PatternInfo {
                pattern: Regex::new(r"\.collect::<Vec<").unwrap(),
                suggestion: "Consider using iterators directly to avoid unnecessary collections".to_string(),
                severity: Severity::Info,
            },
        ];

        Self { patterns }
    }

    async fn find_pattern_issues(&self, code: &str) -> Result<Vec<CodeSuggestion>, anyhow::Error> {
        let mut suggestions = Vec::new();
        let lines: Vec<&str> = code.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            for pattern_info in &self.patterns {
                if pattern_info.pattern.is_match(line) {
                    suggestions.push(CodeSuggestion {
                        line: i as u32 + 1,
                        column: 0,
                        suggestion_type: SuggestionType::BestPractice,
                        message: pattern_info.suggestion.clone(),
                        severity: pattern_info.severity.clone(),
                        fix_suggestion: None,
                    });
                }
            }
        }

        Ok(suggestions)
    }
}

struct PatternInfo {
    pattern: Regex,
    suggestion: String,
    severity: Severity,
}
