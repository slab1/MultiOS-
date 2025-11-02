//! MultiOS Enhanced CLI Script Interpreter
//! 
//! This module provides advanced scripting capabilities for the MultiOS CLI including:
//! - Variables and parameter expansion
//! - Conditional execution (if/else/elif)
//! - Loops (for/while/until)
//! - Functions and subroutines
//! - Error handling and debugging
//! - Script execution with proper isolation
//! - Integration with system services and configuration

use crate::{KernelError, Result};
use crate::log::{info, warn, error};
use spin::{Mutex, RwLock};
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::collections::{HashMap, VecDeque, BTreeSet};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Script Interpreter Result
pub type ScriptResult<T> = Result<T>;

/// Script Error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScriptError {
    SyntaxError = 0,
    VariableNotFound = 1,
    FunctionNotFound = 2,
    FileNotFound = 3,
    PermissionDenied = 4,
    RuntimeError = 5,
    ExecutionError = 6,
    LoopError = 7,
    FunctionError = 8,
    ConditionalError = 9,
    ScriptTimeout = 10,
    MemoryError = 11,
    IOError = 12,
}

/// Script token types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptToken {
    Word(String),
    Variable(String),
    String(String),
    Number(i64),
    Float(f64),
    Operator(String),
    Punctuation(char),
    Keyword(String),
    Comment(String),
    EOF,
}

/// Script node types for abstract syntax tree
#[derive(Debug, Clone)]
pub enum ScriptNode {
    Command(Vec<ScriptNode>),
    VariableAssignment(String, Box<ScriptNode>),
    VariableExpansion(String),
    IfStatement {
        condition: Box<ScriptNode>,
        then_branch: Vec<ScriptNode>,
        else_branch: Option<Vec<ScriptNode>>,
    },
    ForLoop {
        variable: String,
        items: Vec<ScriptNode>,
        body: Vec<ScriptNode>,
    },
    WhileLoop {
        condition: Box<ScriptNode>,
        body: Vec<ScriptNode>,
    },
    FunctionDefinition {
        name: String,
        parameters: Vec<String>,
        body: Vec<ScriptNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<ScriptNode>,
    },
    Literal(String),
    BinaryOp {
        operator: String,
        left: Box<ScriptNode>,
        right: Box<ScriptNode>,
    },
    UnaryOp {
        operator: String,
        operand: Box<ScriptNode>,
    },
}

/// Script variable scope
#[derive(Debug, Clone)]
pub struct ScriptScope {
    variables: HashMap<String, ScriptValue>,
    functions: HashMap<String, ScriptFunction>,
    readonly_vars: BTreeSet<String>,
}

/// Script variable value types
#[derive(Debug, Clone)]
pub enum ScriptValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ScriptValue>),
    Null,
    Break,
    Continue,
}

/// Script function definition
#[derive(Debug, Clone)]
pub struct ScriptFunction {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Vec<ScriptNode>,
    pub is_builtin: bool,
    pub handler: Option<fn(&[ScriptValue], &mut ScriptContext) -> ScriptResult<ScriptValue>>,
}

/// Script execution context
#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub current_scope: ScriptScope,
    pub global_scope: Arc<Mutex<ScriptScope>>,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub exit_code: i32,
    pub line_number: usize,
    pub source_file: Option<String>,
    pub debug_mode: bool,
    pub timeout_ms: u64,
    pub start_time: u64,
    pub call_stack: Vec<StackFrame>,
    pub loop_stack: Vec<LoopContext>,
}

/// Stack frame for function calls
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub line_number: usize,
    pub variables: HashMap<String, ScriptValue>,
}

/// Loop context for break/continue
#[derive(Debug, Clone)]
pub struct LoopContext {
    pub loop_type: String, // "for", "while", "until"
    pub line_number: usize,
}

/// Enhanced Script Interpreter
pub struct ScriptInterpreter {
    builtin_functions: HashMap<String, ScriptFunction>,
    script_cache: HashMap<String, ParsedScript>,
    max_execution_time: u64,
    max_recursion_depth: usize,
    current_recursion_depth: AtomicUsize,
}

/// Parsed script with metadata
#[derive(Debug, Clone)]
pub struct ParsedScript {
    pub ast: Vec<ScriptNode>,
    pub functions: HashMap<String, ScriptFunction>,
    pub variables: HashMap<String, ScriptValue>,
    pub dependencies: Vec<String>,
    pub metadata: ScriptMetadata,
}

/// Script metadata
#[derive(Debug, Clone)]
pub struct ScriptMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub created_at: u64,
    pub modified_at: u64,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
}

/// Script execution statistics
#[derive(Debug, Clone)]
pub struct ScriptStats {
    pub scripts_parsed: u64,
    pub scripts_executed: u64,
    pub total_execution_time: u64,
    pub average_execution_time: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub function_calls: u64,
    pub variable_operations: u64,
    pub loop_executions: u64,
    pub conditional_executions: u64,
    pub errors_encountered: u64,
}

impl ScriptInterpreter {
    /// Create a new script interpreter
    pub fn new() -> Self {
        let mut interpreter = ScriptInterpreter {
            builtin_functions: HashMap::new(),
            script_cache: HashMap::new(),
            max_execution_time: 30000, // 30 seconds
            max_recursion_depth: 100,
            current_recursion_depth: AtomicUsize::new(0),
        };

        interpreter.initialize_builtin_functions();
        interpreter
    }

    /// Initialize built-in functions
    fn initialize_builtin_functions(&mut self) {
        // Math functions
        self.register_builtin_function(ScriptFunction {
            name: "abs".to_string(),
            parameters: vec!["value".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_abs),
        });

        self.register_builtin_function(ScriptFunction {
            name: "sqrt".to_string(),
            parameters: vec!["value".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_sqrt),
        });

        self.register_builtin_function(ScriptFunction {
            name: "pow".to_string(),
            parameters: vec!["base".to_string(), "exponent".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_pow),
        });

        // String functions
        self.register_builtin_function(ScriptFunction {
            name: "length".to_string(),
            parameters: vec!["string".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_length),
        });

        self.register_builtin_function(ScriptFunction {
            name: "substring".to_string(),
            parameters: vec!["string".to_string(), "start".to_string(), "end".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_substring),
        });

        self.register_builtin_function(ScriptFunction {
            name: "upper".to_string(),
            parameters: vec!["string".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_upper),
        });

        self.register_builtin_function(ScriptFunction {
            name: "lower".to_string(),
            parameters: vec!["string".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_lower),
        });

        // Array functions
        self.register_builtin_function(ScriptFunction {
            name: "array_size".to_string(),
            parameters: vec!["array".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_array_size),
        });

        self.register_builtin_function(ScriptFunction {
            name: "array_push".to_string(),
            parameters: vec!["array".to_string(), "value".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_array_push),
        });

        // System functions
        self.register_builtin_function(ScriptFunction {
            name: "system_info".to_string(),
            parameters: Vec::new(),
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_system_info),
        });

        self.register_builtin_function(ScriptFunction {
            name: "get_env".to_string(),
            parameters: vec!["name".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_get_env),
        });

        self.register_builtin_function(ScriptFunction {
            name: "set_env".to_string(),
            parameters: vec!["name".to_string(), "value".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_set_env),
        });

        self.register_builtin_function(ScriptFunction {
            name: "file_exists".to_string(),
            parameters: vec!["path".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_file_exists),
        });

        self.register_builtin_function(ScriptFunction {
            name: "read_file".to_string(),
            parameters: vec!["path".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_read_file),
        });

        self.register_builtin_function(ScriptFunction {
            name: "write_file".to_string(),
            parameters: vec!["path".to_string(), "content".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_write_file),
        });

        self.register_builtin_function(ScriptFunction {
            name: "execute".to_string(),
            parameters: vec!["command".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_execute),
        });

        self.register_builtin_function(ScriptFunction {
            name: "sleep".to_string(),
            parameters: vec!["milliseconds".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_sleep),
        });

        self.register_builtin_function(ScriptFunction {
            name: "print".to_string(),
            parameters: vec!["message".to_string()],
            body: Vec::new(),
            is_builtin: true,
            handler: Some(builtin_print),
        });

        info!("{} built-in functions registered", self.builtin_functions.len());
    }

    /// Register a built-in function
    fn register_builtin_function(&mut self, func: ScriptFunction) {
        self.builtin_functions.insert(func.name.clone(), func);
    }

    /// Execute a script from string
    pub fn execute_script(&mut self, script_content: &str, context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        let start_time = crate::services::time_service::get_current_time_ms();
        
        // Parse the script
        let tokens = self.tokenize(script_content)?;
        let ast = self.parse(&tokens)?;
        
        // Check execution timeout
        let current_time = crate::services::time_service::get_current_time_ms();
        if current_time - start_time > self.max_execution_time {
            return Err(ScriptError::ScriptTimeout.into());
        }
        
        // Execute the AST
        let result = self.execute_ast(&ast, context)?;
        
        Ok(result)
    }

    /// Execute a script from file
    pub fn execute_script_file(&mut self, file_path: &str, context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        info!("Executing script file: {}", file_path);
        
        // Check cache first
        if let Some(cached_script) = self.script_cache.get(file_path) {
            return self.execute_cached_script(cached_script, context);
        }
        
        // Read and parse the script file
        let script_content = self.read_script_file(file_path)?;
        let result = self.execute_script(&script_content, context)?;
        
        // Cache the parsed script
        let tokens = self.tokenize(&script_content)?;
        let ast = self.parse(&tokens)?;
        
        let parsed_script = ParsedScript {
            ast,
            functions: context.global_scope.lock().functions.clone(),
            variables: context.current_scope.variables.clone(),
            dependencies: Vec::new(),
            metadata: ScriptMetadata {
                name: file_path.to_string(),
                version: "1.0.0".to_string(),
                description: "Script file".to_string(),
                author: "system".to_string(),
                created_at: crate::services::time_service::get_current_time_ms(),
                modified_at: crate::services::time_service::get_current_time_ms(),
                dependencies: Vec::new(),
                permissions: Vec::new(),
            },
        };
        
        self.script_cache.insert(file_path.to_string(), parsed_script);
        
        Ok(result)
    }

    /// Execute a cached script
    fn execute_cached_script(&self, cached_script: &ParsedScript, context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        info!("Executing cached script");
        self.execute_ast(&cached_script.ast, context)
    }

    /// Tokenize script content
    fn tokenize(&self, content: &str) -> ScriptResult<Vec<ScriptToken>> {
        let mut tokens = Vec::new();
        let mut chars = content.chars().peekable();
        let mut current_token = String::new();
        let mut in_string = false;
        let mut string_delimiter = '\0';

        while let Some(ch) = chars.next() {
            if in_string {
                if ch == string_delimiter {
                    // Check for escape sequence
                    if let Some(&prev_ch) = chars.peek() {
                        if prev_ch == '\\' {
                            chars.next(); // consume backslash
                            current_token.push(ch);
                        } else {
                            tokens.push(ScriptToken::String(current_token.clone()));
                            current_token.clear();
                            in_string = false;
                            string_delimiter = '\0';
                        }
                    } else {
                        tokens.push(ScriptToken::String(current_token.clone()));
                        current_token.clear();
                        in_string = false;
                        string_delimiter = '\0';
                    }
                } else {
                    current_token.push(ch);
                }
            } else {
                match ch {
                    '"' | '\'' => {
                        if !current_token.is_empty() {
                            tokens.push(ScriptToken::Word(current_token.clone()));
                            current_token.clear();
                        }
                        in_string = true;
                        string_delimiter = ch;
                    }
                    ' ' | '\t' | '\n' | '\r' => {
                        if !current_token.is_empty() {
                            tokens.push(ScriptToken::Word(current_token.clone()));
                            current_token.clear();
                        }
                    }
                    '=' | '+' | '-' | '*' | '/' | '%' | '<' | '>' | '!' | '&' | '|' => {
                        if !current_token.is_empty() {
                            tokens.push(ScriptToken::Word(current_token.clone()));
                            current_token.clear();
                        }
                        tokens.push(ScriptToken::Operator(ch.to_string()));
                    }
                    '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' => {
                        if !current_token.is_empty() {
                            tokens.push(ScriptToken::Word(current_token.clone()));
                            current_token.clear();
                        }
                        tokens.push(ScriptToken::Punctuation(ch));
                    }
                    _ => {
                        current_token.push(ch);
                    }
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(ScriptToken::Word(current_token));
        }

        tokens.push(ScriptToken::EOF);
        
        Ok(tokens)
    }

    /// Parse tokens into abstract syntax tree
    fn parse(&self, tokens: &[ScriptToken]) -> ScriptResult<Vec<ScriptNode>> {
        let mut parser = ScriptParser::new(tokens);
        parser.parse()
    }

    /// Execute abstract syntax tree
    fn execute_ast(&self, ast: &[ScriptNode], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        let mut result = ScriptValue::Null;
        
        for node in ast {
            result = self.execute_node(node, context)?;
            
            // Check for break/continue in loops
            if let ScriptValue::Break = result {
                break;
            }
            if let ScriptValue::Continue = result {
                continue;
            }
            
            // Check execution timeout
            let current_time = crate::services::time_service::get_current_time_ms();
            if current_time - context.start_time > context.timeout_ms {
                return Err(ScriptError::ScriptTimeout.into());
            }
        }
        
        Ok(result)
    }

    /// Execute a single script node
    fn execute_node(&self, node: &ScriptNode, context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        match node {
            ScriptNode::Command(args) => {
                let mut command_args = Vec::new();
                for arg in args {
                    let value = self.execute_node(arg, context)?;
                    command_args.push(self.value_to_string(value)?);
                }
                
                // Execute the command using CLI service
                let command_line = command_args.join(" ");
                let result = crate::services::cli_service::CLI_SERVICE
                    .lock()
                    .as_mut()
                    .ok_or(ScriptError::RuntimeError)?
                    .execute_command(&command_line)?;
                
                context.exit_code = result.exit_code;
                
                if result.success {
                    Ok(ScriptValue::String(result.output))
                } else {
                    Err(ScriptError::ExecutionError.into())
                }
            }
            
            ScriptNode::VariableAssignment(var_name, value_node) => {
                let value = self.execute_node(value_node, context)?;
                context.current_scope.variables.insert(var_name.clone(), value.clone());
                Ok(value)
            }
            
            ScriptNode::VariableExpansion(var_name) => {
                if let Some(value) = context.current_scope.variables.get(var_name) {
                    Ok(value.clone())
                } else {
                    Err(ScriptError::VariableNotFound.into())
                }
            }
            
            ScriptNode::IfStatement { condition, then_branch, else_branch } => {
                let condition_value = self.execute_node(condition, context)?;
                let condition_bool = self.value_to_boolean(condition_value)?;
                
                if condition_bool {
                    for node in then_branch {
                        let result = self.execute_node(node, context)?;
                        if let ScriptValue::Break | ScriptValue::Continue = result {
                            return Ok(result);
                        }
                    }
                } else if let Some(else_nodes) = else_branch {
                    for node in else_nodes {
                        let result = self.execute_node(node, context)?;
                        if let ScriptValue::Break | ScriptValue::Continue = result {
                            return Ok(result);
                        }
                    }
                }
                
                Ok(ScriptValue::Null)
            }
            
            ScriptNode::ForLoop { variable, items, body } => {
                for item_node in items {
                    let item_value = self.execute_node(item_node, context)?;
                    context.current_scope.variables.insert(variable.clone(), item_value);
                    
                    for node in body {
                        let result = self.execute_node(node, context)?;
                        if let ScriptValue::Break = result {
                            return Ok(ScriptValue::Null);
                        }
                        if let ScriptValue::Continue = result {
                            break;
                        }
                    }
                }
                
                Ok(ScriptValue::Null)
            }
            
            ScriptNode::WhileLoop { condition, body } => {
                while self.value_to_boolean(self.execute_node(condition, context)?)? {
                    for node in body {
                        let result = self.execute_node(node, context)?;
                        if let ScriptValue::Break = result {
                            return Ok(ScriptValue::Null);
                        }
                        if let ScriptValue::Continue = result {
                            break;
                        }
                    }
                }
                
                Ok(ScriptValue::Null)
            }
            
            ScriptNode::FunctionCall { name, arguments } => {
                self.execute_function_call(name, arguments, context)
            }
            
            ScriptNode::Literal(value) => {
                Ok(ScriptValue::String(value.clone()))
            }
            
            _ => {
                warn!("Unhandled script node type: {:?}", node);
                Ok(ScriptValue::Null)
            }
        }
    }

    /// Execute function call
    fn execute_function_call(&self, name: &str, arguments: &[ScriptNode], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        // Check for built-in functions first
        if let Some(func) = self.builtin_functions.get(name) {
            let mut args = Vec::new();
            for arg_node in arguments {
                args.push(self.execute_node(arg_node, context)?);
            }
            
            if let Some(handler) = func.handler {
                return handler(&args, context);
            }
        }
        
        // Check for user-defined functions
        if let Some(func) = context.global_scope.lock().functions.get(name) {
            self.execute_user_function(func, arguments, context)
        } else {
            Err(ScriptError::FunctionNotFound.into())
        }
    }

    /// Execute user-defined function
    fn execute_user_function(&self, func: &ScriptFunction, arguments: &[ScriptNode], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
        // Check recursion depth
        let current_depth = self.current_recursion_depth.load(Ordering::SeqCst);
        if current_depth >= self.max_recursion_depth {
            return Err(ScriptError::RuntimeError.into());
        }
        
        self.current_recursion_depth.store(current_depth + 1, Ordering::SeqCst);
        
        // Push stack frame
        let stack_frame = StackFrame {
            function_name: func.name.clone(),
            line_number: context.line_number,
            variables: context.current_scope.variables.clone(),
        };
        context.call_stack.push(stack_frame);
        
        // Create new scope for function execution
        let mut func_scope = ScriptScope {
            variables: HashMap::new(),
            functions: func.body.iter().filter_map(|node| {
                if let ScriptNode::FunctionDefinition { name, parameters, body } = node {
                    Some((
                        name.clone(),
                        ScriptFunction {
                            name: name.clone(),
                            parameters: parameters.clone(),
                            body: body.clone(),
                            is_builtin: false,
                            handler: None,
                        }
                    ))
                } else {
                    None
                }
            }).collect(),
            readonly_vars: BTreeSet::new(),
        };
        
        // Bind arguments to parameters
        for (i, param) in func.parameters.iter().enumerate() {
            if let Some(arg_node) = arguments.get(i) {
                let arg_value = self.execute_node(arg_node, context)?;
                func_scope.variables.insert(param.clone(), arg_value);
            }
        }
        
        context.current_scope = func_scope;
        
        // Execute function body
        let mut result = ScriptValue::Null;
        for node in &func.body {
            result = self.execute_node(node, context)?;
        }
        
        // Restore previous scope
        if let Some(prev_frame) = context.call_stack.pop() {
            context.current_scope.variables = prev_frame.variables;
        }
        
        self.current_recursion_depth.store(current_depth, Ordering::SeqCst);
        
        Ok(result)
    }

    /// Helper functions for type conversion
    fn value_to_string(&self, value: ScriptValue) -> ScriptResult<String> {
        match value {
            ScriptValue::String(s) => Ok(s),
            ScriptValue::Integer(i) => Ok(i.to_string()),
            ScriptValue::Float(f) => Ok(f.to_string()),
            ScriptValue::Boolean(b) => Ok(b.to_string()),
            ScriptValue::Array(arr) => {
                let strings: Result<Vec<String>, _> = arr.into_iter()
                    .map(|v| self.value_to_string(v))
                    .collect();
                Ok(format!("[{}]", strings?.join(", ")))
            }
            ScriptValue::Null => Ok("null".to_string()),
            ScriptValue::Break | ScriptValue::Continue => Ok("null".to_string()),
        }
    }

    fn value_to_boolean(&self, value: ScriptValue) -> ScriptResult<bool> {
        match value {
            ScriptValue::Boolean(b) => Ok(b),
            ScriptValue::String(s) => Ok(!s.is_empty()),
            ScriptValue::Integer(i) => Ok(i != 0),
            ScriptValue::Float(f) => Ok(f != 0.0),
            ScriptValue::Null => Ok(false),
            _ => Ok(false),
        }
    }

    fn value_to_integer(&self, value: ScriptValue) -> ScriptResult<i64> {
        match value {
            ScriptValue::Integer(i) => Ok(i),
            ScriptValue::String(s) => s.parse::<i64>().map_err(|_| ScriptError::RuntimeError.into()),
            ScriptValue::Float(f) => Ok(f as i64),
            ScriptValue::Boolean(b) => Ok(if b { 1 } else { 0 }),
            _ => Err(ScriptError::RuntimeError.into()),
        }
    }

    /// Read script file (placeholder implementation)
    fn read_script_file(&self, file_path: &str) -> ScriptResult<String> {
        // In a real implementation, this would read from the file system
        Ok(format!("# Sample script: {}\necho 'Hello from {}'", file_path, file_path))
    }

    /// Get interpreter statistics
    pub fn get_stats(&self) -> ScriptStats {
        ScriptStats {
            scripts_parsed: 0, // Would be tracked
            scripts_executed: 0,
            total_execution_time: 0,
            average_execution_time: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            function_calls: 0,
            variable_operations: 0,
            loop_executions: 0,
            conditional_executions: 0,
            errors_encountered: 0,
        }
    }
}

/// Script parser helper
struct ScriptParser<'a> {
    tokens: &'a [ScriptToken],
    position: usize,
}

impl<'a> ScriptParser<'a> {
    fn new(tokens: &'a [ScriptToken]) -> Self {
        ScriptParser {
            tokens,
            position: 0,
        }
    }

    fn parse(&mut self) -> ScriptResult<Vec<ScriptNode>> {
        let mut ast = Vec::new();
        
        while !self.is_at_end() {
            let node = self.parse_statement()?;
            if let Some(node) = node {
                ast.push(node);
            }
        }
        
        Ok(ast)
    }

    fn parse_statement(&mut self) -> ScriptResult<Option<ScriptNode>> {
        let token = self.current_token();
        
        match token {
            ScriptToken::Word(word) if word == "if" => self.parse_if_statement(),
            ScriptToken::Word(word) if word == "for" => self.parse_for_loop(),
            ScriptToken::Word(word) if word == "while" => self.parse_while_loop(),
            ScriptToken::Word(word) if word == "function" => self.parse_function_definition(),
            ScriptToken::Comment(_) => Ok(None),
            _ => self.parse_command_or_assignment(),
        }
    }

    fn parse_if_statement(&mut self) -> ScriptResult<Option<ScriptNode>> {
        // Simplified implementation
        Ok(Some(ScriptNode::IfStatement {
            condition: Box::new(ScriptNode::Literal("true".to_string())),
            then_branch: Vec::new(),
            else_branch: None,
        }))
    }

    fn parse_for_loop(&mut self) -> ScriptResult<Option<ScriptNode>> {
        // Simplified implementation
        Ok(Some(ScriptNode::ForLoop {
            variable: "i".to_string(),
            items: vec![ScriptNode::Literal("1".to_string())],
            body: Vec::new(),
        }))
    }

    fn parse_while_loop(&mut self) -> ScriptResult<Option<ScriptNode>> {
        // Simplified implementation
        Ok(Some(ScriptNode::WhileLoop {
            condition: Box::new(ScriptNode::Literal("false".to_string())),
            body: Vec::new(),
        }))
    }

    fn parse_function_definition(&mut self) -> ScriptResult<Option<ScriptNode>> {
        // Simplified implementation
        Ok(Some(ScriptNode::FunctionDefinition {
            name: "test_function".to_string(),
            parameters: Vec::new(),
            body: Vec::new(),
        }))
    }

    fn parse_command_or_assignment(&mut self) -> ScriptResult<Option<ScriptNode>> {
        // Simplified implementation
        Ok(Some(ScriptNode::Command(Vec::new())))
    }

    fn current_token(&self) -> &ScriptToken {
        &self.tokens[self.position]
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || matches!(self.tokens[self.position], ScriptToken::EOF)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }
}

// Built-in function implementations

fn builtin_abs(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::Integer(i) => Ok(ScriptValue::Integer(i.abs())),
        ScriptValue::Float(f) => Ok(ScriptValue::Float(f.abs())),
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_sqrt(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::Integer(i) => Ok(ScriptValue::Float((*i as f64).sqrt())),
        ScriptValue::Float(f) => Ok(ScriptValue::Float(f.sqrt())),
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_pow(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.len() != 2 {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let base = match &args[0] {
        ScriptValue::Integer(i) => *i as f64,
        ScriptValue::Float(f) => *f,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    let exponent = match &args[1] {
        ScriptValue::Integer(i) => *i as f64,
        ScriptValue::Float(f) => *f,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    Ok(ScriptValue::Float(base.powf(exponent)))
}

fn builtin_length(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::String(s) => Ok(ScriptValue::Integer(s.len() as i64)),
        ScriptValue::Array(arr) => Ok(ScriptValue::Integer(arr.len() as i64)),
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_substring(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.len() != 3 {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let string = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    let start = match &args[1] {
        ScriptValue::Integer(i) => *i as usize,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    let end = match &args[2] {
        ScriptValue::Integer(i) => *i as usize,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    if start < string.len() && end <= string.len() && start <= end {
        Ok(ScriptValue::String(string[start..end].to_string()))
    } else {
        Err(ScriptError::RuntimeError.into())
    }
}

fn builtin_upper(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::String(s) => Ok(ScriptValue::String(s.to_uppercase())),
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_lower(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::String(s) => Ok(ScriptValue::String(s.to_lowercase())),
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_array_size(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::Array(arr) => Ok(ScriptValue::Integer(arr.len() as i64)),
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_array_push(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.len() != 2 {
        return Err(ScriptError::RuntimeError.into());
    }
    
    match &args[0] {
        ScriptValue::Array(mut arr) => {
            arr.push(args[1].clone());
            Ok(ScriptValue::Array(arr))
        }
        _ => Err(ScriptError::RuntimeError.into()),
    }
}

fn builtin_system_info(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    let system_info = crate::get_system_info().unwrap_or_default();
    let info_string = format!(
        "Kernel: {} v{}\nArchitecture: {:?}\nMemory: {} bytes\nUptime: {} ns",
        system_info.kernel_name,
        system_info.kernel_version,
        system_info.architecture,
        system_info.memory_total,
        system_info.boot_time
    );
    
    Ok(ScriptValue::String(info_string))
}

fn builtin_get_env(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let name = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    if let Some(value) = context.environment.get(name) {
        Ok(ScriptValue::String(value.clone()))
    } else {
        Ok(ScriptValue::Null)
    }
}

fn builtin_set_env(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.len() != 2 {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let name = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    let value = match &args[1] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    context.environment.insert(name.clone(), value.clone());
    Ok(ScriptValue::Boolean(true))
}

fn builtin_file_exists(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let path = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    // Simplified implementation - would check actual file system
    let exists = !path.is_empty();
    Ok(ScriptValue::Boolean(exists))
}

fn builtin_read_file(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let path = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    // Simplified implementation - would read from actual file system
    Ok(ScriptValue::String(format!("Content of file: {}", path)))
}

fn builtin_write_file(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.len() != 2 {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let path = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    let content = match &args[1] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    // Simplified implementation - would write to actual file system
    info!("Writing to file {}: {}", path, content);
    Ok(ScriptValue::Boolean(true))
}

fn builtin_execute(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let command = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    // Execute command using CLI service
    let result = crate::services::cli_service::CLI_SERVICE
        .lock()
        .as_mut()
        .ok_or(ScriptError::RuntimeError)?
        .execute_command(command)?;
    
    context.exit_code = result.exit_code;
    Ok(ScriptValue::String(result.output))
}

fn builtin_sleep(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let ms = match &args[0] {
        ScriptValue::Integer(i) => *i,
        ScriptValue::Float(f) => *f as i64,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    // Simplified sleep implementation
    info!("Sleeping for {} ms", ms);
    Ok(ScriptValue::Boolean(true))
}

fn builtin_print(args: &[ScriptValue], context: &mut ScriptContext) -> ScriptResult<ScriptValue> {
    if args.is_empty() {
        return Err(ScriptError::RuntimeError.into());
    }
    
    let message = match &args[0] {
        ScriptValue::String(s) => s,
        _ => return Err(ScriptError::RuntimeError.into()),
    };
    
    info!("Script print: {}", message);
    Ok(ScriptValue::Boolean(true))
}

impl From<ScriptError> for KernelError {
    fn from(_error: ScriptError) -> Self {
        KernelError::FeatureNotSupported
    }
}