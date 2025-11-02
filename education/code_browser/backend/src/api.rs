use actix_web::{post, get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::code_analyzer::CodeAnalyzer;
use crate::call_graph::CallGraphAnalyzer;
use crate::performance_analyzer::PerformanceAnalyzer;
use crate::data_flow::DataFlowAnalyzer;
use crate::utils::CodeContext;

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub code: String,
    pub language: String,
    pub file_path: Option<String>,
    pub context_lines: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallGraphRequest {
    pub file_path: String,
    pub function_name: Option<String>,
    pub depth_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceAnalysisRequest {
    pub code: String,
    pub file_path: String,
    pub function_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataFlowRequest {
    pub code: String,
    pub file_path: String,
    pub variable_name: String,
    pub line_number: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub file_path: Option<String>,
    pub language: Option<String>,
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigationRequest {
    pub symbol_name: String,
    pub file_path: String,
    pub line_number: Option<u32>,
}

// Code Analysis Endpoints

#[post("/analyze/code")]
pub async fn analyze_code(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<AnalysisRequest>,
) -> impl Responder {
    match analyzer.analyze_code(&request.code, &request.language).await {
        Ok(analysis) => HttpResponse::Ok().json(analysis),
        Err(e) => HttpResponse::InternalServerError().json(format!("Analysis failed: {}", e)),
    }
}

#[post("/analyze/suggestions")]
pub async fn get_code_suggestions(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<AnalysisRequest>,
) -> impl Responder {
    match analyzer.get_code_suggestions(&request.code, &request.language).await {
        Ok(suggestions) => HttpResponse::Ok().json(suggestions),
        Err(e) => HttpResponse::InternalServerError().json(format!("Suggestions failed: {}", e)),
    }
}

#[post("/analyze/educational")]
pub async fn get_educational_comments(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<AnalysisRequest>,
) -> impl Responder {
    match analyzer.get_educational_comments(&request.code, &request.language).await {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => HttpResponse::InternalServerError().json(format!("Educational comments failed: {}", e)),
    }
}

// Call Graph Endpoints

#[post("/callgraph/generate")]
pub async fn get_call_graph(
    analyzer: web::Data<Arc<CallGraphAnalyzer>>,
    request: web::Json<CallGraphRequest>,
) -> impl Responder {
    match analyzer.generate_call_graph(&request.file_path, request.function_name.as_deref(), request.depth_limit).await {
        Ok(graph) => HttpResponse::Ok().json(graph),
        Err(e) => HttpResponse::InternalServerError().json(format!("Call graph generation failed: {}", e)),
    }
}

#[post("/callgraph/dependencies")]
pub async fn get_function_dependencies(
    analyzer: web::Data<Arc<CallGraphAnalyzer>>,
    request: web::Json<CallGraphRequest>,
) -> impl Responder {
    match analyzer.get_function_dependencies(&request.file_path, request.function_name.as_deref()).await {
        Ok(dependencies) => HttpResponse::Ok().json(dependencies),
        Err(e) => HttpResponse::InternalServerError().json(format!("Dependencies analysis failed: {}", e)),
    }
}

#[post("/callgraph/trace")]
pub async fn trace_function_calls(
    analyzer: web::Data<Arc<CallGraphAnalyzer>>,
    request: web::Json<CallGraphRequest>,
) -> impl Responder {
    match analyzer.trace_function_calls(&request.file_path, request.function_name.as_deref(), request.depth_limit).await {
        Ok(trace) => HttpResponse::Ok().json(trace),
        Err(e) => HttpResponse::InternalServerError().json(format!("Function tracing failed: {}", e)),
    }
}

// Performance Analysis Endpoints

#[post("/performance/hotspots")]
pub async fn get_performance_hotspots(
    analyzer: web::Data<Arc<PerformanceAnalyzer>>,
    request: web::Json<PerformanceAnalysisRequest>,
) -> impl Responder {
    match analyzer.identify_performance_hotspots(&request.code, &request.file_path).await {
        Ok(hotspots) => HttpResponse::Ok().json(hotspots),
        Err(e) => HttpResponse::InternalServerError().json(format!("Hotspot identification failed: {}", e)),
    }
}

#[post("/performance/function")]
pub async fn analyze_function_performance(
    analyzer: web::Data<Arc<PerformanceAnalyzer>>,
    request: web::Json<PerformanceAnalysisRequest>,
) -> impl Responder {
    match analyzer.analyze_function_performance(&request.code, &request.function_name).await {
        Ok(analysis) => HttpResponse::Ok().json(analysis),
        Err(e) => HttpResponse::InternalServerError().json(format!("Function performance analysis failed: {}", e)),
    }
}

#[post("/performance/optimization")]
pub async fn get_optimization_suggestions(
    analyzer: web::Data<Arc<PerformanceAnalyzer>>,
    request: web::Json<PerformanceAnalysisRequest>,
) -> impl Responder {
    match analyzer.get_optimization_suggestions(&request.code, &request.file_path).await {
        Ok(suggestions) => HttpResponse::Ok().json(suggestions),
        Err(e) => HttpResponse::InternalServerError().json(format!("Optimization suggestions failed: {}", e)),
    }
}

// Data Flow Analysis Endpoints

#[post("/dataflow/analyze")]
pub async fn analyze_data_flow(
    analyzer: web::Data<Arc<DataFlowAnalyzer>>,
    request: web::Json<DataFlowRequest>,
) -> impl Responder {
    match analyzer.analyze_data_flow(&request.code, &request.variable_name, request.line_number).await {
        Ok(analysis) => HttpResponse::Ok().json(analysis),
        Err(e) => HttpResponse::InternalServerError().json(format!("Data flow analysis failed: {}", e)),
    }
}

#[post("/dataflow/variables")]
pub async fn track_variable_usage(
    analyzer: web::Data<Arc<DataFlowAnalyzer>>,
    request: web::Json<DataFlowRequest>,
) -> impl Responder {
    match analyzer.track_variable_usage(&request.code, &request.variable_name).await {
        Ok(tracking) => HttpResponse::Ok().json(tracking),
        Err(e) => HttpResponse::InternalServerError().json(format!("Variable tracking failed: {}", e)),
    }
}

#[post("/dataflow/dependencies")]
pub async fn find_data_dependencies(
    analyzer: web::Data<Arc<DataFlowAnalyzer>>,
    request: web::Json<DataFlowRequest>,
) -> impl Responder {
    match analyzer.find_data_dependencies(&request.code, &request.variable_name).await {
        Ok(dependencies) => HttpResponse::Ok().json(dependencies),
        Err(e) => HttpResponse::InternalServerError().json(format!("Data dependencies analysis failed: {}", e)),
    }
}

// Search and Navigation Endpoints

#[post("/search")]
pub async fn search_code(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<SearchRequest>,
) -> impl Responder {
    match analyzer.search_code(&request.query, request.file_path.as_deref(), request.language.as_deref()).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(e) => HttpResponse::InternalServerError().json(format!("Search failed: {}", e)),
    }
}

#[post("/navigate")]
pub async fn navigate_symbol(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<NavigationRequest>,
) -> impl Responder {
    match analyzer.navigate_to_symbol(&request.symbol_name, &request.file_path, request.line_number).await {
        Ok(location) => HttpResponse::Ok().json(location),
        Err(e) => HttpResponse::InternalServerError().json(format!("Navigation failed: {}", e)),
    }
}

#[post("/context")]
pub async fn get_code_context(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<NavigationRequest>,
) -> impl Responder {
    match analyzer.get_code_context(&request.file_path, request.line_number).await {
        Ok(context) => HttpResponse::Ok().json(context),
        Err(e) => HttpResponse::InternalServerError().json(format!("Context retrieval failed: {}", e)),
    }
}

// Integration Endpoints

#[post("/debug/integrate")]
pub async fn integrate_debugging(
    analyzer: web::Data<Arc<CodeAnalyzer>>,
    request: web::Json<NavigationRequest>,
) -> impl Responder {
    // This would integrate with debugging systems
    // For now, returning mock integration data
    let integration_info = serde_json::json!({
        "status": "integrated",
        "debugger_type": "gdb",
        "breakpoints": [],
        "watchpoints": [],
        "step_info": {
            "available": true,
            "current_line": request.line_number.unwrap_or(1),
            "variables_in_scope": []
        }
    });
    
    HttpResponse::Ok().json(integration_info)
}

#[get("/education/modules")]
pub async fn get_educational_modules() -> impl Responder {
    let modules = serde_json::json!({
        "modules": [
            {
                "id": "kernel_basics",
                "title": "Kernel Basics",
                "description": "Introduction to kernel architecture and design",
                "difficulty": "beginner",
                "estimated_time": "2 hours",
                "prerequisites": [],
                "topics": ["memory management", "process scheduling", "system calls"]
            },
            {
                "id": "memory_management",
                "title": "Memory Management",
                "description": "Deep dive into memory management mechanisms",
                "difficulty": "intermediate",
                "estimated_time": "4 hours",
                "prerequisites": ["kernel_basics"],
                "topics": ["virtual memory", "page tables", "memory allocation", "garbage collection"]
            },
            {
                "id": "process_scheduling",
                "title": "Process Scheduling",
                "description": "Understanding process scheduling algorithms",
                "difficulty": "intermediate",
                "estimated_time": "3 hours",
                "prerequisites": ["kernel_basics"],
                "topics": ["scheduling algorithms", "priority systems", "context switching"]
            },
            {
                "id": "device_drivers",
                "title": "Device Drivers",
                "description": "Building and integrating device drivers",
                "difficulty": "advanced",
                "estimated_time": "6 hours",
                "prerequisites": ["memory_management", "process_scheduling"],
                "topics": ["driver architecture", "interrupt handling", "I/O operations"]
            },
            {
                "id": "performance_optimization",
                "title": "Performance Optimization",
                "description": "Advanced performance analysis and optimization",
                "difficulty": "expert",
                "estimated_time": "8 hours",
                "prerequisites": ["device_drivers"],
                "topics": ["profiling", "cache optimization", "multicore scaling", "benchmarking"]
            }
        ]
    });
    
    HttpResponse::Ok().json(modules)
}
