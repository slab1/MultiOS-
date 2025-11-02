use actix_files::Files;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use std::collections::HashMap;

mod api;
mod code_analyzer;
mod call_graph;
mod performance_analyzer;
mod data_flow;
mod utils;

use api::*;
use code_analyzer::CodeAnalyzer;
use call_graph::CallGraphAnalyzer;
use performance_analyzer::PerformanceAnalyzer;
use data_flow::DataFlowAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let code_analyzer = Arc::new(CodeAnalyzer::new());
    let call_graph_analyzer = Arc::new(CallGraphAnalyzer::new());
    let performance_analyzer = Arc::new(PerformanceAnalyzer::new());
    let data_flow_analyzer = Arc::new(DataFlowAnalyzer::new());

    log::info!("Starting Code Browser Backend Server...");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(code_analyzer.clone()))
            .app_data(web::Data::new(call_graph_analyzer.clone()))
            .app_data(web::Data::new(performance_analyzer.clone()))
            .app_data(web::Data::new(data_flow_analyzer.clone()))
            .service(Files::new("/static", "./static"))
            .configure(api_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}

fn api_config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/api/v1")
            // Code analysis endpoints
            .service(analyze_code)
            .service(get_code_suggestions)
            .service(get_educational_comments)
            
            // Call graph endpoints
            .service(get_call_graph)
            .service(get_function_dependencies)
            .service(trace_function_calls)
            
            // Performance analysis endpoints
            .service(get_performance_hotspots)
            .service(analyze_function_performance)
            .service(get_optimization_suggestions)
            
            // Data flow analysis endpoints
            .service(analyze_data_flow)
            .service(track_variable_usage)
            .service(find_data_dependencies)
            
            // Search and navigation endpoints
            .service(search_code)
            .service(navigate_symbol)
            .service(get_code_context)
            
            // Integration endpoints
            .service(integrate_debugging)
            .service(get_educational_modules)
    );
}
