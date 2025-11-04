//! MultiOS Benchmark Runner
//! 
//! This is the main executable for running comprehensive performance benchmarks
//! on MultiOS and comparing performance against other operating systems.

use clap::{Arg, Command};
use std::path::PathBuf;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};

mod cpu;
mod memory;
mod filesystem;
mod network;
mod boot_time;
mod syscalls;
mod utils;
mod reporter;

use crate::BenchmarkConfig;
use crate::{BenchmarkCategory, BenchmarkRunner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("multios-benchmark")
        .version("1.0.0")
        .about("MultiOS Comprehensive Performance Benchmarking")
        .author("MultiOS Development Team")
        .subcommand(
            Command::new("run")
                .about("Run benchmarks")
                .arg(Arg::new("category")
                    .short('c')
                    .long("category")
                    .value_name("CATEGORY")
                    .help("Benchmark category to run (cpu, memory, filesystem, network, boot, syscalls, all)")
                    .default_value("all"))
                .arg(Arg::new("iterations")
                    .short('i')
                    .long("iterations")
                    .value_name("N")
                    .help("Number of iterations for each benchmark")
                    .default_value("10000"))
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Output file for results"))
                .arg(Arg::new("format")
                    .short('f')
                    .long("format")
                    .value_name("FORMAT")
                    .help("Output format (human, json, csv, html)")
                    .default_value("human"))
                .arg(Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .help("Enable verbose output"))
                .arg(Arg::new("compare")
                    .long("compare")
                    .value_name("BASELINE_FILE")
                    .help("Compare results against baseline file"))
        )
        .subcommand(
            Command::new("list")
                .about("List available benchmarks")
                .arg(Arg::new("category")
                    .short('c')
                    .long("category")
                    .value_name("CATEGORY")
                    .help("Filter by category"))
        )
        .subcommand(
            Command::new("report")
                .about("Generate report from benchmark results")
                .arg(Arg::new("input")
                    .required(true)
                    .value_name("FILE")
                    .help("Input results file"))
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Output report file"))
                .arg(Arg::new("format")
                    .short('f')
                    .long("format")
                    .value_name("FORMAT")
                    .help("Report format (human, html)")
                    .default_value("html"))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("run", args)) => {
            run_benchmarks(args)
        }
        Some(("list", args)) => {
            list_benchmarks(args)
        }
        Some(("report", args)) => {
            generate_report(args)
        }
        _ => {
            println!("Use --help for usage information");
            Ok(())
        }
    }
}

fn run_benchmarks(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let category = args.get_one::<String>("category").unwrap();
    let iterations = args.get_one::<String>("iterations").unwrap().parse::<u64>()?;
    let verbose = args.get_flag("verbose");
    let output_file = args.get_one::<String>("output").map(PathBuf::from);
    let format = match args.get_one::<String>("format").unwrap().as_str() {
        "json" => crate::OutputFormat::Json,
        "csv" => crate::OutputFormat::Csv,
        "html" => crate::OutputFormat::Html,
        _ => crate::OutputFormat::Human,
    };
    let compare_file = args.get_one::<String>("compare").map(PathBuf::from);

    println!("MultiOS Performance Benchmarking");
    println!("================================");
    println!("Category: {}", category);
    println!("Iterations: {}", iterations);
    println!("Format: {}", format);
    println!();

    // Create benchmark configuration
    let config = BenchmarkConfig {
        iterations,
        warmup_iterations: iterations / 100,
        timeout: Some(Duration::from_secs(300)),
        batch_size: 1000,
        verbose,
        output_format: format,
        compare_baseline: compare_file.is_some(),
    };

    // Validate configuration
    crate::utils::ConfigValidator::validate_config(&config)
        .map_err(|e| format!("Configuration error: {}", e))?;

    // Collect benchmarks based on category
    let benchmarks = collect_benchmarks(category)?;
    
    println!("Found {} benchmarks to run", benchmarks.len());
    println!();

    // Run benchmarks
    let runner = BenchmarkRunner::new(verbose);
    let mut results = Vec::new();

    let progress = ProgressBar::new(benchmarks.len() as u64);
    progress.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40} {pos}/{len} {msg}")?
        .progress_chars("=>-"));

    for benchmark in benchmarks {
        if verbose {
            println!("Running: {}", benchmark.0);
        }
        
        progress.set_message(benchmark.0.to_string());
        
        match runner.run_benchmark(&benchmark.1, iterations) {
            Ok(result) => {
                results.push(result);
                if verbose {
                    println!("✓ Completed in {:?}", result.duration);
                }
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
            }
        }
        
        progress.inc(1);
    }

    progress.finish();

    // Load baseline for comparison if specified
    let mut comparisons = Vec::new();
    if let Some(baseline_file) = compare_file {
        if baseline_file.exists() {
            println!("Loading baseline from: {:?}", baseline_file);
            // In a real implementation, you would load and parse the baseline file
        } else {
            println!("Warning: Baseline file not found: {:?}", baseline_file);
        }
    }

    // Generate and save report
    if let Some(output_path) = output_file {
        println!("Saving results to: {:?}", output_path);
        let report_content = reporter::ReportGenerator::new(reporter::ReportConfig {
            output_format: format,
            include_statistics: true,
            include_comparison: !comparisons.is_empty(),
            include_system_info: true,
            sort_by: reporter::SortBy::OperationsPerSecond,
            group_by_category: true,
        }).generate_report(&results, Some(&comparisons))?;
        
        std::fs::write(&output_path, report_content)?;
        println!("Results saved successfully");
    }

    // Print summary
    print_summary(&results)?;

    Ok(())
}

fn collect_benchmarks(category: &str) -> Result<Vec<(String, Box<dyn crate::Benchmark + Send + Sync>)>, Box<dyn std::error::Error>> {
    let mut benchmarks = Vec::new();

    match category {
        "cpu" | "all" => {
            let cpu_suite = cpu::CpuBenchmarkSuite::new();
            benchmarks.push(("CPU Integer Operations".to_string(), Box::new(cpu_suite.integer_bench) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("CPU Floating-Point Operations".to_string(), Box::new(cpu_suite.float_bench) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Matrix Multiplication".to_string(), Box::new(cpu_suite.matrix_bench) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Cryptographic Operations".to_string(), Box::new(cpu_suite.crypto_bench) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("SIMD Operations".to_string(), Box::new(cpu_suite.simd_bench) as Box<dyn crate::Benchmark + Send + Sync>));
        }
        "memory" | "all" => {
            let mem_suite = memory::MemoryBenchmarkSuite::new();
            benchmarks.push(("Sequential Memory Read".to_string(), Box::new(mem_suite.sequential_read) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Sequential Memory Write".to_string(), Box::new(mem_suite.sequential_write) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Random Memory Access".to_string(), Box::new(mem_suite.random_access) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Memory Allocation".to_string(), Box::new(mem_suite.allocation) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Cache Performance".to_string(), Box::new(mem_suite.cache_perf) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Memory Bandwidth".to_string(), Box::new(mem_suite.bandwidth) as Box<dyn crate::Benchmark + Send + Sync>));
        }
        "filesystem" | "fs" | "all" => {
            let fs_suite = filesystem::FileSystemBenchmarkSuite::new();
            benchmarks.push(("Sequential File Read".to_string(), Box::new(fs_suite.sequential_read) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Sequential File Write".to_string(), Box::new(fs_suite.sequential_write) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Random File Access".to_string(), Box::new(fs_suite.random_access) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("File Metadata Operations".to_string(), Box::new(fs_suite.metadata) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Small File Operations".to_string(), Box::new(fs_suite.small_files) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Directory Traversal".to_string(), Box::new(fs_suite.directory_traversal) as Box<dyn crate::Benchmark + Send + Sync>));
        }
        "network" | "all" => {
            let net_suite = network::NetworkBenchmarkSuite::new();
            benchmarks.push(("TCP Connection".to_string(), Box::new(net_suite.tcp_connection) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("TCP Throughput".to_string(), Box::new(net_suite.tcp_throughput) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("UDP Latency".to_string(), Box::new(net_suite.udp_latency) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Socket Creation".to_string(), Box::new(net_suite.socket_creation) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Protocol Overhead".to_string(), Box::new(net_suite.protocol_overhead) as Box<dyn crate::Benchmark + Send + Sync>));
        }
        "boot" | "boot_time" | "all" => {
            let boot_suite = boot_time::BootTimeBenchmarkSuite::new();
            benchmarks.push(("Boot Time Analysis".to_string(), Box::new(boot_suite.boot_analysis) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Boot Time Comparison".to_string(), Box::new(boot_suite.boot_comparison) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Boot Optimization Analysis".to_string(), Box::new(boot_suite.boot_optimization) as Box<dyn crate::Benchmark + Send + Sync>));
        }
        "syscalls" | "all" => {
            let syscall_suite = syscalls::SyscallBenchmarkSuite::new();
            benchmarks.push(("System Call Overhead".to_string(), Box::new(syscall_suite.syscall_overhead) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Process Creation".to_string(), Box::new(syscall_suite.process_creation) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("Thread Creation".to_string(), Box::new(syscall_suite.thread_creation) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("File System Calls".to_string(), Box::new(syscall_suite.file_syscalls) as Box<dyn crate::Benchmark + Send + Sync>));
            benchmarks.push(("IPC System Calls".to_string(), Box::new(syscall_suite.ipc_syscalls) as Box<dyn crate::Benchmark + Send + Sync>));
        }
        _ => {
            return Err(format!("Unknown category: {}", category).into());
        }
    }

    Ok(benchmarks)
}

fn list_benchmarks(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let category_filter = args.get_one::<String>("category");
    
    println!("Available Benchmarks:");
    println!("=====================");
    
    let categories = ["cpu", "memory", "filesystem", "network", "boot", "syscalls"];
    
    for category in categories {
        if let Some(filter) = category_filter {
            if filter != category && filter != "all" {
                continue;
            }
        }
        
        println!("\n{}:", category.to_uppercase());
        let benchmarks = collect_benchmarks(category)?;
        
        for (name, _) in benchmarks {
            println!("  - {}", name);
        }
    }
    
    Ok(())
}

fn generate_report(args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = args.get_one::<String>("input").unwrap();
    let output_file = args.get_one::<String>("output").map(PathBuf::from);
    let format = match args.get_one::<String>("format").unwrap().as_str() {
        "html" => crate::OutputFormat::Html,
        _ => crate::OutputFormat::Human,
    };

    println!("Generating report from: {}", input_file);
    
    // In a real implementation, you would load the results file
    // and generate a report. For now, we'll just print a message.
    println!("Note: Report generation from file is not yet implemented.");
    println!("This would load the JSON/CSV results and generate a formatted report.");
    
    Ok(())
}

fn print_summary(results: &[crate::BenchmarkResult]) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Benchmark Summary ===");
    
    let summary = reporter::generate_summary(results);
    
    for (key, value) in summary {
        println!("{}: {}", key, value);
    }
    
    // Show top performers
    if !results.is_empty() {
        let mut sorted_results = results.to_vec();
        sorted_results.sort_by(|a, b| b.operations_per_second.partial_cmp(&a.operations_per_second).unwrap());
        
        println!("\nTop 5 Performers:");
        for (i, result) in sorted_results.iter().take(5).enumerate() {
            println!("{}. {}: {:.2} {}/s", i + 1, result.name, result.operations_per_second, result.unit);
        }
    }
    
    Ok(())
}