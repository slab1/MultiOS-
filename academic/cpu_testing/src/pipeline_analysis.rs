//! Pipeline Analysis Module
//!
//! This module provides comprehensive analysis of CPU pipeline behavior,
//! branch prediction performance, and execution unit utilization across
//! different architectures.

use crate::architecture::{Architecture, ArchitectureSpec, BranchPredictor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineAnalysisResult {
    pub architecture: Architecture,
    pub analysis_name: String,
    pub analysis_type: PipelineAnalysisType,
    pub pipeline_depth: u32,
    pub instruction_throughput: f64,
    pub branch_prediction_accuracy: f64,
    pub misprediction_rate: f64,
    pub pipeline_stalls: u64,
    pub execution_unit_utilization: HashMap<String, f64>,
    pub dependency_analysis: DependencyAnalysis,
    pub speculation_effectiveness: f64,
    pub detailed_metrics: PipelineMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineAnalysisType {
    BranchPrediction,
    PipelineDepth,
    OutOfOrder,
    Speculation,
    Dependencies,
    ExecutionUnits,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub data_dependencies: u64,
    pub control_dependencies: u64,
    pub structural_dependencies: u64,
    pub false_dependencies: u64,
    pub dependency_cycles: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetrics {
    pub instructions_per_cycle: f64,
    pub average_latency: f64,
    pub stall_cycles: u64,
    pub fetch_efficiency: f64,
    pub decode_efficiency: f64,
    pub execute_efficiency: f64,
    pub memory_efficiency: f64,
    pub writeback_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPredictionConfig {
    pub branch_frequency: f64,
    pub taken_percentage: f64,
    pub pattern_complexity: PatternComplexity,
    pub prediction_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternComplexity {
    Simple,
    Alternating,
    Random,
    Strided,
    Complex,
}

pub struct PipelineAnalyzer {
    architecture_specs: HashMap<Architecture, ArchitectureSpec>,
}

impl PipelineAnalyzer {
    pub fn new() -> Self {
        let mut specs = HashMap::new();
        
        for arch in ArchitectureSpec::all_architectures() {
            specs.insert(arch.clone(), ArchitectureSpec::get(&arch));
        }

        Self {
            architecture_specs: specs,
        }
    }

    /// Run pipeline analysis for multiple architectures
    pub fn run_pipeline_analysis(
        &self,
        architectures: Vec<Architecture>,
        analysis_type: &str,
    ) -> Result<HashMap<Architecture, Vec<PipelineAnalysisResult>>> {
        let mut results = HashMap::new();
        
        for arch in architectures {
            info!("Running pipeline analysis for architecture: {}", arch);
            let analysis_results = self.run_architecture_pipeline_analysis(&arch, analysis_type)?;
            results.insert(arch, analysis_results);
        }

        Ok(results)
    }

    /// Run pipeline analysis for a specific architecture
    fn run_architecture_pipeline_analysis(
        &self,
        architecture: &Architecture,
        analysis_type: &str,
    ) -> Result<Vec<PipelineAnalysisResult>> {
        let spec = self.architecture_specs.get(architecture)
            .context("Architecture specification not found")?;

        match analysis_type {
            "branch_prediction" => self.run_branch_prediction_analysis(architecture, spec),
            "pipeline" => self.run_pipeline_depth_analysis(architecture, spec),
            "out_of_order" => self.run_out_of_order_analysis(architecture, spec),
            "speculation" => self.run_speculation_analysis(architecture, spec),
            "dependencies" => self.run_dependency_analysis(architecture, spec),
            "execution_units" => self.run_execution_unit_analysis(architecture, spec),
            "all" => self.run_all_pipeline_analysis(architecture, spec),
            _ => self.run_all_pipeline_analysis(architecture, spec),
        }
    }

    /// Run branch prediction analysis
    fn run_branch_prediction_analysis(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let mut results = Vec::new();

        // Different branch patterns
        let branch_configs = vec![
            BranchPredictionConfig {
                branch_frequency: 0.2,
                taken_percentage: 0.6,
                pattern_complexity: PatternComplexity::Simple,
                prediction_window: 16,
            },
            BranchPredictionConfig {
                branch_frequency: 0.15,
                taken_percentage: 0.4,
                pattern_complexity: PatternComplexity::Alternating,
                prediction_window: 32,
            },
            BranchPredictionConfig {
                branch_frequency: 0.25,
                taken_percentage: 0.7,
                pattern_complexity: PatternComplexity::Complex,
                prediction_window: 64,
            },
        ];

        for config in branch_configs {
            let result = self.simulate_branch_prediction(architecture, spec, &config)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run pipeline depth analysis
    fn run_pipeline_depth_analysis(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let mut results = Vec::new();

        // Different instruction mixes
        let instruction_mixes = vec![
            ("ALU Intensive", 0.8),
            ("Memory Intensive", 0.3),
            ("Mixed Workload", 0.5),
            ("Floating Point", 0.6),
        ];

        for (mix_name, memory_intensive) in instruction_mixes {
            let result = self.simulate_pipeline_performance(architecture, spec, mix_name, memory_intensive)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run out-of-order execution analysis
    fn run_out_of_order_analysis(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let result = if spec.pipeline_info.out_of_order {
            self.analyze_out_of_order_performance(architecture, spec)?
        } else {
            self.analyze_in_order_performance(architecture, spec)?
        };

        Ok(vec![result])
    }

    /// Run speculation analysis
    fn run_speculation_analysis(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let mut results = Vec::new();

        let speculation_levels = vec![
            ("Conservative", 0.5),
            ("Moderate", 0.7),
            ("Aggressive", 0.9),
        ];

        for (level, aggressiveness) in speculation_levels {
            let result = self.simulate_speculation_performance(architecture, spec, level, aggressiveness)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run dependency analysis
    fn run_dependency_analysis(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let dependency_types = vec![
            ("Data Dependencies", 0.4),
            ("Control Dependencies", 0.2),
            ("Structural Dependencies", 0.1),
            ("Mixed Dependencies", 0.3),
        ];

        let mut results = Vec::new();

        for (dep_name, data_dep_ratio) in dependency_types {
            let result = self.simulate_dependency_impact(architecture, spec, dep_name, data_dep_ratio)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run execution unit analysis
    fn run_execution_unit_analysis(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let mut results = Vec::new();

        // Different execution unit mixes
        let unit_mixes = vec![
            ("ALU Heavy", "ALU", 0.8),
            ("Memory Heavy", "Memory", 0.7),
            ("FP Heavy", "FloatingPoint", 0.6),
            ("Balanced", "ALU", 0.5),
        ];

        for (mix_name, unit_type, utilization) in unit_mixes {
            let result = self.simulate_execution_unit_usage(architecture, spec, mix_name, unit_type, utilization)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Run all pipeline analysis
    fn run_all_pipeline_analysis(&self, architecture: &Architecture, spec: ArchitectureSpec) -> Result<Vec<PipelineAnalysisResult>> {
        let mut all_results = Vec::new();

        all_results.extend(self.run_branch_prediction_analysis(architecture, &spec)?);
        all_results.extend(self.run_pipeline_depth_analysis(architecture, &spec)?);
        all_results.extend(self.run_out_of_order_analysis(architecture, &spec)?);
        all_results.extend(self.run_speculation_analysis(architecture, &spec)?);
        all_results.extend(self.run_dependency_analysis(architecture, &spec)?);
        all_results.extend(self.run_execution_unit_analysis(architecture, &spec)?);

        Ok(all_results)
    }

    /// Simulate branch prediction performance
    fn simulate_branch_prediction(
        &self,
        architecture: &Architecture,
        spec: &ArchitectureSpec,
        config: &BranchPredictionConfig,
    ) -> Result<PipelineAnalysisResult> {
        let base_accuracy = match &spec.pipeline_info.branch_predictor {
            BranchPredictor::Static => 0.60,
            BranchPredictor::Dynamic2Bit => 0.85,
            BranchPredictor::GShare => 0.90,
            BranchPredictor::TAGE => 0.93,
            BranchPredictor::Tournament => 0.95,
            BranchPredictor::Custom(_) => 0.88,
        };

        let pattern_bonus = match &config.pattern_complexity {
            PatternComplexity::Simple => 0.05,
            PatternComplexity::Alternating => 0.02,
            PatternComplexity::Random => -0.10,
            PatternComplexity::Strided => 0.03,
            PatternComplexity::Complex => -0.05,
        };

        let branch_accuracy = (base_accuracy + pattern_bonus).min(0.99).max(0.01);
        let misprediction_rate = 1.0 - branch_accuracy;

        // Calculate pipeline stalls due to mispredictions
        let stall_penalty = match spec.pipeline_info.stages {
            stages if stages <= 5 => 1.0,
            stages if stages <= 10 => 2.0,
            _ => 3.0,
        };

        let pipeline_stalls = (config.branch_frequency * misprediction_rate * stall_penalty * 1000.0) as u64;

        // Simulate processing time
        std::thread::sleep(std::time::Duration::from_millis((pipeline_stalls as f64 / 1000.0) as u64));

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: format!("{} Branch Prediction", config.pattern_complexity.as_str()),
            analysis_type: PipelineAnalysisType::BranchPrediction,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: 1.0 / (1.0 + misprediction_rate * 2.0),
            branch_prediction_accuracy: branch_accuracy,
            misprediction_rate,
            pipeline_stalls,
            execution_unit_utilization: HashMap::new(),
            dependency_analysis: DependencyAnalysis {
                data_dependencies: (config.branch_frequency * 1000.0) as u64,
                control_dependencies: (config.branch_frequency * 800.0) as u64,
                structural_dependencies: 50,
                false_dependencies: 20,
                dependency_cycles: 2,
            },
            speculation_effectiveness: if spec.pipeline_info.speculation {
                branch_accuracy * 0.95
            } else {
                0.0
            },
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: 1.0 / (1.0 + misprediction_rate * 2.0),
                average_latency: spec.pipeline_info.stages as f64,
                stall_cycles: pipeline_stalls,
                fetch_efficiency: 0.95,
                decode_efficiency: 0.92,
                execute_efficiency: 0.88,
                memory_efficiency: 0.85,
                writeback_efficiency: 0.97,
            },
        })
    }

    /// Simulate pipeline performance
    fn simulate_pipeline_performance(
        &self,
        architecture: &Architecture,
        spec: &ArchitectureSpec,
        workload_name: &str,
        memory_intensive: f64,
    ) -> Result<PipelineAnalysisResult> {
        let pipeline_efficiency = match workload_name {
            "ALU Intensive" => 0.95,
            "Memory Intensive" => 0.60,
            "Mixed Workload" => 0.75,
            "Floating Point" => 0.85,
            _ => 0.80,
        };

        let memory_stalls = (memory_intensive * spec.pipeline_info.stages as f64 / 2.0) as u64;
        let total_stalls = memory_stalls;

        let throughput = (spec.pipeline_info.pipeline_width as f64 * pipeline_efficiency) / (1.0 + memory_stalls as f64 / 1000.0);

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: workload_name.to_string(),
            analysis_type: PipelineAnalysisType::PipelineDepth,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: throughput,
            branch_prediction_accuracy: 0.90, // Simplified
            misprediction_rate: 0.10,
            pipeline_stalls: total_stalls,
            execution_unit_utilization: self.get_unit_utilization(architecture, workload_name),
            dependency_analysis: DependencyAnalysis {
                data_dependencies: 400,
                control_dependencies: 200,
                structural_dependencies: 100,
                false_dependencies: 50,
                dependency_cycles: 3,
            },
            speculation_effectiveness: if spec.pipeline_info.speculation { 0.85 } else { 0.0 },
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: throughput,
                average_latency: spec.pipeline_info.stages as f64 + memory_stalls as f64 / 100.0,
                stall_cycles: total_stalls,
                fetch_efficiency: 0.90,
                decode_efficiency: pipeline_efficiency * 0.95,
                execute_efficiency: pipeline_efficiency,
                memory_efficiency: (1.0 - memory_intensive) * 0.9 + memory_intensive * 0.6,
                writeback_efficiency: 0.95,
            },
        })
    }

    /// Analyze out-of-order performance
    fn analyze_out_of_order_performance(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<PipelineAnalysisResult> {
        let reorder_buffer_size = spec.pipeline_info.stages * 4;
        let execution_window = spec.pipeline_info.stages * 8;
        
        let efficiency = if reorder_buffer_size > 32 {
            0.92
        } else {
            0.75 + (reorder_buffer_size as f64 / 32.0) * 0.17
        };

        let throughput = spec.pipeline_info.pipeline_width as f64 * efficiency;

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: "Out-of-Order Execution Analysis".to_string(),
            analysis_type: PipelineAnalysisType::OutOfOrder,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: throughput,
            branch_prediction_accuracy: 0.90,
            misprediction_rate: 0.10,
            pipeline_stalls: 100,
            execution_unit_utilization: HashMap::new(),
            dependency_analysis: DependencyAnalysis {
                data_dependencies: 300,
                control_dependencies: 150,
                structural_dependencies: 80,
                false_dependencies: 40,
                dependency_cycles: 2,
            },
            speculation_effectiveness: if spec.pipeline_info.speculation { 0.90 } else { 0.0 },
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: throughput,
                average_latency: spec.pipeline_info.stages as f64 * 0.8,
                stall_cycles: 100,
                fetch_efficiency: 0.95,
                decode_efficiency: 0.90,
                execute_efficiency: efficiency,
                memory_efficiency: 0.85,
                writeback_efficiency: 0.98,
            },
        })
    }

    /// Analyze in-order performance
    fn analyze_in_order_performance(&self, architecture: &Architecture, spec: &ArchitectureSpec) -> Result<PipelineAnalysisResult> {
        let efficiency = 0.70; // In-order is typically less efficient
        let throughput = spec.pipeline_info.pipeline_width as f64 * efficiency;

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: "In-Order Execution Analysis".to_string(),
            analysis_type: PipelineAnalysisType::OutOfOrder,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: throughput,
            branch_prediction_accuracy: 0.85,
            misprediction_rate: 0.15,
            pipeline_stalls: 200,
            execution_unit_utilization: HashMap::new(),
            dependency_analysis: DependencyAnalysis {
                data_dependencies: 500,
                control_dependencies: 300,
                structural_dependencies: 150,
                false_dependencies: 100,
                dependency_cycles: 4,
            },
            speculation_effectiveness: 0.0,
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: throughput,
                average_latency: spec.pipeline_info.stages as f64,
                stall_cycles: 200,
                fetch_efficiency: 0.80,
                decode_efficiency: 0.75,
                execute_efficiency: efficiency,
                memory_efficiency: 0.60,
                writeback_efficiency: 0.90,
            },
        })
    }

    /// Simulate speculation performance
    fn simulate_speculation_performance(
        &self,
        architecture: &Architecture,
        spec: &ArchitectureSpec,
        level: &str,
        aggressiveness: f64,
    ) -> Result<PipelineAnalysisResult> {
        let speculation_success_rate = aggressiveness * 0.85 + (1.0 - aggressiveness) * 0.20;
        let branch_accuracy = 0.90 + aggressiveness * 0.05;
        
        let pipeline_stalls = if spec.pipeline_info.speculation {
            ((1.0 - speculation_success_rate) * 200.0) as u64
        } else {
            500
        };

        let throughput = (spec.pipeline_info.pipeline_width as f64 * branch_accuracy) / (1.0 + pipeline_stalls as f64 / 1000.0);

        std::thread::sleep(std::time::Duration::from_millis((pipeline_stalls as f64 / 1000.0) as u64));

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: format!("{} Speculation", level),
            analysis_type: PipelineAnalysisType::Speculation,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: throughput,
            branch_prediction_accuracy: branch_accuracy,
            misprediction_rate: 1.0 - branch_accuracy,
            pipeline_stalls,
            execution_unit_utilization: HashMap::new(),
            dependency_analysis: DependencyAnalysis {
                data_dependencies: 350,
                control_dependencies: 180,
                structural_dependencies: 90,
                false_dependencies: 45,
                dependency_cycles: 3,
            },
            speculation_effectiveness: speculation_success_rate,
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: throughput,
                average_latency: spec.pipeline_info.stages as f64 * (1.0 - speculation_success_rate * 0.2),
                stall_cycles: pipeline_stalls,
                fetch_efficiency: 0.95,
                decode_efficiency: 0.88,
                execute_efficiency: 0.85,
                memory_efficiency: 0.80,
                writeback_efficiency: 0.96,
            },
        })
    }

    /// Simulate dependency impact
    fn simulate_dependency_impact(
        &self,
        architecture: &Architecture,
        spec: &ArchitectureSpec,
        dep_name: &str,
        data_dep_ratio: f64,
    ) -> Result<PipelineAnalysisResult> {
        let total_dependencies = 1000;
        let data_deps = (total_dependencies as f64 * data_dep_ratio) as u64;
        let control_deps = (total_dependencies as f64 * 0.2) as u64;
        let structural_deps = (total_dependencies as f64 * 0.1) as u64;
        let false_deps = (total_dependencies as f64 * 0.05) as u64;

        let stall_impact = data_deps as f64 * 1.5 + control_deps as f64 * 3.0 + structural_deps as f64 * 0.5;
        let throughput = (spec.pipeline_info.pipeline_width as f64) / (1.0 + stall_impact / 10000.0);

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: format!("Dependency Analysis: {}", dep_name),
            analysis_type: PipelineAnalysisType::Dependencies,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: throughput,
            branch_prediction_accuracy: 0.90,
            misprediction_rate: 0.10,
            pipeline_stalls: stall_impact as u64,
            execution_unit_utilization: HashMap::new(),
            dependency_analysis: DependencyAnalysis {
                data_dependencies: data_deps,
                control_dependencies: control_deps,
                structural_dependencies: structural_deps,
                false_dependencies: false_deps,
                dependency_cycles: 3,
            },
            speculation_effectiveness: if spec.pipeline_info.speculation { 0.85 } else { 0.0 },
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: throughput,
                average_latency: spec.pipeline_info.stages as f64 + stall_impact as f64 / 100.0,
                stall_cycles: stall_impact as u64,
                fetch_efficiency: 0.90,
                decode_efficiency: 0.85,
                execute_efficiency: 0.80,
                memory_efficiency: 0.75,
                writeback_efficiency: 0.95,
            },
        })
    }

    /// Simulate execution unit usage
    fn simulate_execution_unit_usage(
        &self,
        architecture: &Architecture,
        spec: &ArchitectureSpec,
        mix_name: &str,
        unit_type: &str,
        utilization: f64,
    ) -> Result<PipelineAnalysisResult> {
        let mut unit_utilization = HashMap::new();
        
        // Set utilization for the primary unit type
        unit_utilization.insert(unit_type.to_string(), utilization);
        
        // Set lower utilization for other units
        let other_units = ["ALU", "Memory", "FloatingPoint", "Vector"];
        for unit in &other_units {
            if unit != &unit_type {
                unit_utilization.insert(unit.to_string(), utilization * 0.3);
            }
        }

        let overall_throughput = spec.pipeline_info.pipeline_width as f64 * utilization;

        Ok(PipelineAnalysisResult {
            architecture: architecture.clone(),
            analysis_name: format!("Execution Unit Analysis: {}", mix_name),
            analysis_type: PipelineAnalysisType::ExecutionUnits,
            pipeline_depth: spec.pipeline_info.stages,
            instruction_throughput: overall_throughput,
            branch_prediction_accuracy: 0.90,
            misprediction_rate: 0.10,
            pipeline_stalls: ((1.0 - utilization) * 500.0) as u64,
            execution_unit_utilization: unit_utilization,
            dependency_analysis: DependencyAnalysis {
                data_dependencies: 400,
                control_dependencies: 200,
                structural_dependencies: 100,
                false_dependencies: 50,
                dependency_cycles: 2,
            },
            speculation_effectiveness: if spec.pipeline_info.speculation { 0.88 } else { 0.0 },
            detailed_metrics: PipelineMetrics {
                instructions_per_cycle: overall_throughput,
                average_latency: spec.pipeline_info.stages as f64 * (2.0 - utilization),
                stall_cycles: ((1.0 - utilization) * 500.0) as u64,
                fetch_efficiency: 0.92,
                decode_efficiency: 0.90,
                execute_efficiency: utilization,
                memory_efficiency: if unit_type == "Memory" { utilization } else { 0.75 },
                writeback_efficiency: 0.96,
            },
        })
    }

    /// Get unit utilization for different workloads
    fn get_unit_utilization(&self, architecture: &Architecture, workload_name: &str) -> HashMap<String, f64> {
        let mut utilization = HashMap::new();
        
        match workload_name {
            "ALU Heavy" => {
                utilization.insert("ALU".to_string(), 0.85);
                utilization.insert("Memory".to_string(), 0.20);
                utilization.insert("FloatingPoint".to_string(), 0.30);
                utilization.insert("Vector".to_string(), 0.10);
            }
            "Memory Heavy" => {
                utilization.insert("ALU".to_string(), 0.40);
                utilization.insert("Memory".to_string(), 0.90);
                utilization.insert("FloatingPoint".to_string(), 0.20);
                utilization.insert("Vector".to_string(), 0.15);
            }
            "FP Heavy" => {
                utilization.insert("ALU".to_string(), 0.50);
                utilization.insert("Memory".to_string(), 0.30);
                utilization.insert("FloatingPoint".to_string(), 0.80);
                utilization.insert("Vector".to_string(), 0.40);
            }
            "Balanced" => {
                utilization.insert("ALU".to_string(), 0.70);
                utilization.insert("Memory".to_string(), 0.60);
                utilization.insert("FloatingPoint".to_string(), 0.50);
                utilization.insert("Vector".to_string(), 0.30);
            }
            _ => {
                utilization.insert("ALU".to_string(), 0.60);
                utilization.insert("Memory".to_string(), 0.50);
                utilization.insert("FloatingPoint".to_string(), 0.40);
                utilization.insert("Vector".to_string(), 0.25);
            }
        }
        
        utilization
    }
}

impl PatternComplexity {
    fn as_str(&self) -> &'static str {
        match self {
            PatternComplexity::Simple => "Simple",
            PatternComplexity::Alternating => "Alternating",
            PatternComplexity::Random => "Random",
            PatternComplexity::Strided => "Strided",
            PatternComplexity::Complex => "Complex",
        }
    }
}

/// Public API functions

/// Run pipeline analysis for multiple architectures
pub fn run_pipeline_analysis(architectures: Vec<Architecture>, analysis_type: &str) -> Result<HashMap<Architecture, Vec<PipelineAnalysisResult>>> {
    let analyzer = PipelineAnalyzer::new();
    analyzer.run_pipeline_analysis(architectures, analysis_type)
}

/// Save pipeline analysis results to JSON file
pub fn save_pipeline_results(results: &HashMap<Architecture, Vec<PipelineAnalysisResult>>, output_file: &str) -> Result<()> {
    let json_data = serde_json::to_string_pretty(results)
        .context("Failed to serialize pipeline analysis results")?;
    
    // Ensure directory exists
    if let Some(parent) = Path::new(output_file).parent() {
        fs::create_dir_all(parent)
            .context("Failed to create output directory")?;
    }
    
    fs::write(output_file, json_data)
        .context("Failed to write pipeline analysis results")?;
    
    info!("Pipeline analysis results saved to {}", output_file);
    Ok(())
}

/// Generate pipeline analysis report
pub fn generate_pipeline_analysis_report(results: &HashMap<Architecture, Vec<PipelineAnalysisResult>>, output_file: &str) -> Result<String> {
    let mut report = String::new();
    
    report.push_str("# Pipeline Analysis Report\n\n");
    report.push_str("## Executive Summary\n\n");
    report.push_str("This report analyzes pipeline performance across different CPU architectures.\n\n");

    // Summary table
    report.push_str("## Pipeline Performance Summary\n\n");
    report.push_str("| Architecture | Avg Throughput | Branch Accuracy | Pipeline Stalls | Speculation Eff |\n");
    report.push_str("|--------------|----------------|-----------------|-----------------|-----------------|\n");

    for (arch, arch_results) in results {
        let avg_throughput: f64 = arch_results.iter().map(|r| r.instruction_throughput).sum::<f64>() / arch_results.len() as f64;
        let avg_branch_acc: f64 = arch_results.iter().map(|r| r.branch_prediction_accuracy).sum::<f64>() / arch_results.len() as f64;
        let avg_stalls: f64 = arch_results.iter().map(|r| r.pipeline_stalls as f64).sum::<f64>() / arch_results.len() as f64;
        let avg_speculation: f64 = arch_results.iter().map(|r| r.speculation_effectiveness).sum::<f64>() / arch_results.len() as f64;

        report.push_str(&format!(
            "| {} | {:.2} | {:.2} | {:.0} | {:.2} |\n",
            arch, avg_throughput, avg_branch_acc, avg_stalls, avg_speculation
        ));
    }

    report.push_str("\n## Detailed Analysis\n\n");

    for (arch, arch_results) in results {
        report.push_str(&format!("### {} Pipeline Performance\n\n", arch));
        
        // Group by analysis type
        let mut branch_results = Vec::new();
        let mut pipeline_results = Vec::new();
        let mut ooo_results = Vec::new();
        let mut spec_results = Vec::new();
        let mut dep_results = Vec::new();
        let mut exec_results = Vec::new();

        for result in &arch_results {
            match result.analysis_type {
                PipelineAnalysisType::BranchPrediction => branch_results.push(result),
                PipelineAnalysisType::PipelineDepth => pipeline_results.push(result),
                PipelineAnalysisType::OutOfOrder => ooo_results.push(result),
                PipelineAnalysisType::Speculation => spec_results.push(result),
                PipelineAnalysisType::Dependencies => dep_results.push(result),
                PipelineAnalysisType::ExecutionUnits => exec_results.push(result),
                _ => {}
            }
        }

        if !branch_results.is_empty() {
            report.push_str("#### Branch Prediction Performance\n\n");
            for result in branch_results {
                report.push_str(&format!("- **{}**: {:.1}% accuracy, {:.0} stalls\n", 
                    result.analysis_name, result.branch_prediction_accuracy * 100.0, result.pipeline_stalls));
            }
            report.push_str("\n");
        }

        if !pipeline_results.is_empty() {
            report.push_str("#### Pipeline Depth Analysis\n\n");
            for result in pipeline_results {
                report.push_str(&format!("- **{}**: {:.2} IPC, {:.0} stalls\n", 
                    result.analysis_name, result.instruction_throughput, result.pipeline_stalls));
            }
            report.push_str("\n");
        }

        if !spec_results.is_empty() {
            report.push_str("#### Speculation Performance\n\n");
            for result in spec_results {
                report.push_str(&format!("- **{}**: {:.1}% effectiveness\n", 
                    result.analysis_name, result.speculation_effectiveness * 100.0));
            }
            report.push_str("\n");
        }

        if !exec_results.is_empty() {
            report.push_str("#### Execution Unit Utilization\n\n");
            for result in exec_results {
                if !result.execution_unit_utilization.is_empty() {
                    report.push_str(&format!("- **{}**: ", result.analysis_name));
                    for (unit, util) in &result.execution_unit_utilization {
                        report.push_str(&format!("{}: {:.1}%, ", unit, util * 100.0));
                    }
                    report.push_str("\n");
                }
            }
            report.push_str("\n");
        }

        report.push_str("\n");
    }

    // Ensure directory exists
    if let Some(parent) = Path::new(output_file).parent() {
        fs::create_dir_all(parent)
            .context("Failed to create output directory")?;
    }
    
    fs::write(output_file, report)
        .context("Failed to write pipeline analysis report")?;
    
    info!("Pipeline analysis report saved to {}", output_file);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_analyzer_creation() {
        let analyzer = PipelineAnalyzer::new();
        assert_eq!(analyzer.architecture_specs.len(), 5);
    }

    #[test]
    fn test_branch_prediction_simulation() {
        let analyzer = PipelineAnalyzer::new();
        let arch = Architecture::X86_64;
        let spec = ArchitectureSpec::get(&arch);
        
        let config = BranchPredictionConfig {
            branch_frequency: 0.2,
            taken_percentage: 0.6,
            pattern_complexity: PatternComplexity::Simple,
            prediction_window: 16,
        };
        
        let result = analyzer.simulate_branch_prediction(&arch, &spec, &config).unwrap();
        assert!(result.branch_prediction_accuracy >= 0.0 && result.branch_prediction_accuracy <= 1.0);
        assert!(result.misprediction_rate >= 0.0 && result.misprediction_rate <= 1.0);
    }
}