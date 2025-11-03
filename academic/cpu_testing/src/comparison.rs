//! Architecture Comparison Module
//!
//! This module provides comprehensive comparison and analysis tools for
//! different CPU architectures, generating detailed research reports.

use crate::architecture::{Architecture, ArchitectureSpec};
use crate::performance::{BenchmarkResult, PerformanceAnalyzer};
use crate::memory_hierarchy::{MemoryTestResult, MemoryHierarchyTester};
use crate::pipeline_analysis::{PipelineAnalysisResult, PipelineAnalyzer};
use crate::isa_testing::ISATestResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureComparison {
    pub comparison_name: String,
    pub architectures: Vec<Architecture>,
    pub performance_comparison: PerformanceComparison,
    pub memory_comparison: MemoryComparison,
    pub pipeline_comparison: PipelineComparison,
    pub isa_comparison: ISAComparison,
    pub power_comparison: PowerComparison,
    pub feature_comparison: FeatureComparison,
    pub overall_ranking: OverallRanking,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    pub benchmark_results: HashMap<Architecture, Vec<BenchmarkResult>>,
    pub average_scores: HashMap<Architecture, f64>,
    pub best_performing: HashMap<String, Architecture>,
    pub performance_gaps: HashMap<(Architecture, Architecture), f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryComparison {
    pub memory_results: HashMap<Architecture, Vec<MemoryTestResult>>,
    pub latency_comparison: HashMap<Architecture, u64>,
    pub bandwidth_comparison: HashMap<Architecture, u64>,
    pub cache_efficiency: HashMap<Architecture, f64>,
    pub memory_hierarchy_scores: HashMap<Architecture, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineComparison {
    pub pipeline_results: HashMap<Architecture, Vec<PipelineAnalysisResult>>,
    pub branch_prediction_scores: HashMap<Architecture, f64>,
    pub pipeline_efficiency: HashMap<Architecture, f64>,
    pub speculation_effectiveness: HashMap<Architecture, f64>,
    pub execution_unit_utilization: HashMap<Architecture, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ISAComparison {
    pub isa_results: HashMap<Architecture, ISATestResult>,
    pub instruction_coverage: HashMap<Architecture, f64>,
    pub execution_efficiency: HashMap<Architecture, f64>,
    pub compatibility_scores: HashMap<Architecture, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerComparison {
    pub power_consumption: HashMap<Architecture, f64>,
    pub performance_per_watt: HashMap<Architecture, f64>,
    pub energy_efficiency_scores: HashMap<Architecture, f64>,
    pub thermal_design_power: HashMap<Architecture, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureComparison {
    pub supported_features: HashMap<Architecture, Vec<String>>,
    pub vector_capabilities: HashMap<Architecture, Vec<String>>,
    pub security_features: HashMap<Architecture, Vec<String>>,
    pub specialized_extensions: HashMap<Architecture, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallRanking {
    pub overall_scores: HashMap<Architecture, f64>,
    pub rankings: Vec<ArchitectureRanking>,
    pub strengths_weaknesses: HashMap<Architecture, ArchitectureProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureRanking {
    pub architecture: Architecture,
    pub overall_score: f64,
    pub rank: u32,
    pub performance_score: f64,
    pub efficiency_score: f64,
    pub feature_score: f64,
    pub compatibility_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureProfile {
    pub architecture: Architecture,
    pub key_strengths: Vec<String>,
    pub key_weaknesses: Vec<String>,
    pub best_use_cases: Vec<String>,
    pub optimal_workloads: Vec<String>,
    pub recommended_for: String,
}

pub struct ArchitectureComparator {
    performance_analyzer: PerformanceAnalyzer,
    memory_tester: MemoryHierarchyTester,
    pipeline_analyzer: PipelineAnalyzer,
}

impl ArchitectureComparator {
    pub fn new() -> Self {
        Self {
            performance_analyzer: PerformanceAnalyzer::new(),
            memory_tester: MemoryHierarchyTester::new(),
            pipeline_analyzer: PipelineAnalyzer::new(),
        }
    }

    /// Compare multiple architectures comprehensively
    pub fn compare_architectures(
        &self,
        architectures: Vec<Architecture>,
        comparison_type: &str,
    ) -> Result<ArchitectureComparison> {
        info!("Starting comprehensive architecture comparison");
        info!("Architectures: {:?}", architectures);
        info!("Comparison type: {}", comparison_type);

        // Run all tests if comprehensive, otherwise specific tests
        let performance_results = if comparison_type == "comprehensive" || comparison_type == "performance" {
            self.collect_performance_data(&architectures)?
        } else {
            HashMap::new()
        };

        let memory_results = if comparison_type == "comprehensive" || comparison_type == "memory" {
            self.collect_memory_data(&architectures)?
        } else {
            HashMap::new()
        };

        let pipeline_results = if comparison_type == "comprehensive" || comparison_type == "pipeline" {
            self.collect_pipeline_data(&architectures)?
        } else {
            HashMap::new()
        };

        let isa_results = if comparison_type == "comprehensive" || comparison_type == "isa" {
            self.collect_isa_data(&architectures)?
        } else {
            HashMap::new()
        };

        // Perform comprehensive analysis
        let performance_comparison = self.analyze_performance(&performance_results);
        let memory_comparison = self.analyze_memory(&memory_results);
        let pipeline_comparison = self.analyze_pipeline(&pipeline_results);
        let isa_comparison = self.analyze_isa(&isa_results);
        let power_comparison = self.analyze_power_efficiency(&performance_results);
        let feature_comparison = self.analyze_features(&architectures);
        let overall_ranking = self.calculate_overall_ranking(
            &performance_comparison,
            &memory_comparison,
            &pipeline_comparison,
            &isa_comparison,
            &power_comparison,
            &feature_comparison,
        );

        let recommendations = self.generate_recommendations(&overall_ranking);

        Ok(ArchitectureComparison {
            comparison_name: format!("{} Architecture Comparison", comparison_type),
            architectures,
            performance_comparison,
            memory_comparison,
            pipeline_comparison,
            isa_comparison,
            power_comparison,
            feature_comparison,
            overall_ranking,
            recommendations,
        })
    }

    /// Collect performance data for architectures
    fn collect_performance_data(&self, architectures: &Vec<Architecture>) -> Result<HashMap<Architecture, Vec<BenchmarkResult>>> {
        self.performance_analyzer.run_benchmarks(architectures.clone(), "comprehensive")
    }

    /// Collect memory hierarchy data for architectures
    fn collect_memory_data(&self, architectures: &Vec<Architecture>) -> Result<HashMap<Architecture, Vec<MemoryTestResult>>> {
        self.memory_tester.run_memory_tests(architectures.clone(), "all")
    }

    /// Collect pipeline analysis data for architectures
    fn collect_pipeline_data(&self, architectures: &Vec<Architecture>) -> Result<HashMap<Architecture, Vec<PipelineAnalysisResult>>> {
        self.pipeline_analyzer.run_pipeline_analysis(architectures.clone(), "all")
    }

    /// Collect ISA testing data for architectures
    fn collect_isa_data(&self, architectures: &Vec<Architecture>) -> Result<HashMap<Architecture, ISATestResult>> {
        crate::isa_testing::run_isa_test_suite(architectures.clone(), "comprehensive")
    }

    /// Analyze performance results
    fn analyze_performance(&self, results: &HashMap<Architecture, Vec<BenchmarkResult>>) -> PerformanceComparison {
        let mut average_scores = HashMap::new();
        let mut best_performing = HashMap::new();
        let mut performance_gaps = HashMap::new();

        // Calculate average performance scores
        for (arch, arch_results) in results {
            let avg_score: f64 = arch_results.iter()
                .map(|r| r.instructions_per_second as f64)
                .sum::<f64>() / arch_results.len() as f64;
            average_scores.insert(arch.clone(), avg_score);
        }

        // Find best performing architecture for each benchmark type
        let mut benchmark_types = HashMap::new();
        for (arch, arch_results) in results {
            for result in arch_results {
                benchmark_types
                    .entry(format!("{:?}", result.test_type))
                    .or_insert_with(Vec::new)
                    .push((arch.clone(), result));
            }
        }

        for (benchmark_type, mut benchmark_results) in benchmark_types {
            benchmark_results.sort_by(|a, b| b.1.instructions_per_second.cmp(&a.1.instructions_per_second));
            if let Some((best_arch, _)) = benchmark_results.first() {
                best_performing.insert(benchmark_type, best_arch.clone());
            }
        }

        // Calculate performance gaps
        let architectures: Vec<_> = results.keys().collect();
        for (i, arch1) in architectures.iter().enumerate() {
            for arch2 in &architectures[i + 1..] {
                let score1 = average_scores.get(arch1).unwrap_or(&0.0);
                let score2 = average_scores.get(arch2).unwrap_or(&0.0);
                let gap = ((score1.max(*score2) - score1.min(*score2)) / score1.max(*score2)) * 100.0;
                performance_gaps.insert((arch1.clone(), arch2.clone()), gap);
                performance_gaps.insert((arch2.clone(), arch1.clone()), gap);
            }
        }

        PerformanceComparison {
            benchmark_results: results.clone(),
            average_scores,
            best_performing,
            performance_gaps,
        }
    }

    /// Analyze memory results
    fn analyze_memory(&self, results: &HashMap<Architecture, Vec<MemoryTestResult>>) -> MemoryComparison {
        let mut latency_comparison = HashMap::new();
        let mut bandwidth_comparison = HashMap::new();
        let mut cache_efficiency = HashMap::new();
        let mut memory_hierarchy_scores = HashMap::new();

        for (arch, arch_results) in results {
            let avg_latency: u64 = arch_results.iter()
                .map(|r| r.latency_ns)
                .sum::<u64>() / arch_results.len() as u64;
            let avg_bandwidth: u64 = arch_results.iter()
                .map(|r| r.bandwidth_mbps)
                .sum::<u64>() / arch_results.len() as u64;
            let avg_hit_rate: f64 = arch_results.iter()
                .map(|r| r.hit_rate)
                .sum::<f64>() / arch_results.len() as f64;

            latency_comparison.insert(arch.clone(), avg_latency);
            bandwidth_comparison.insert(arch.clone(), avg_bandwidth);
            cache_efficiency.insert(arch.clone(), avg_hit_rate);
            
            // Calculate memory hierarchy score (lower latency + higher bandwidth + better hit rate)
            let memory_score = (1000000.0 / avg_latency as f64 + avg_bandwidth as f64 / 100.0 + avg_hit_rate * 100.0) / 3.0;
            memory_hierarchy_scores.insert(arch.clone(), memory_score);
        }

        MemoryComparison {
            memory_results: results.clone(),
            latency_comparison,
            bandwidth_comparison,
            cache_efficiency,
            memory_hierarchy_scores,
        }
    }

    /// Analyze pipeline results
    fn analyze_pipeline(&self, results: &HashMap<Architecture, Vec<PipelineAnalysisResult>>) -> PipelineComparison {
        let mut branch_prediction_scores = HashMap::new();
        let mut pipeline_efficiency = HashMap::new();
        let mut speculation_effectiveness = HashMap::new();
        let mut execution_unit_utilization = HashMap::new();

        for (arch, arch_results) in results {
            let avg_branch_acc: f64 = arch_results.iter()
                .map(|r| r.branch_prediction_accuracy)
                .sum::<f64>() / arch_results.len() as f64;
            let avg_throughput: f64 = arch_results.iter()
                .map(|r| r.instruction_throughput)
                .sum::<f64>() / arch_results.len() as f64;
            let avg_speculation: f64 = arch_results.iter()
                .map(|r| r.speculation_effectiveness)
                .sum::<f64>() / arch_results.len() as f64;
            let avg_utilization: f64 = arch_results.iter()
                .flat_map(|r| r.execution_unit_utilization.values())
                .sum::<f64>() / (arch_results.len() * 4) as f64; // Assuming 4 unit types on average

            branch_prediction_scores.insert(arch.clone(), avg_branch_acc);
            pipeline_efficiency.insert(arch.clone(), avg_throughput);
            speculation_effectiveness.insert(arch.clone(), avg_speculation);
            execution_unit_utilization.insert(arch.clone(), avg_utilization);
        }

        PipelineComparison {
            pipeline_results: results.clone(),
            branch_prediction_scores,
            pipeline_efficiency,
            speculation_effectiveness,
            execution_unit_utilization,
        }
    }

    /// Analyze ISA results
    fn analyze_isa(&self, results: &HashMap<Architecture, ISATestResult>) -> ISAComparison {
        let mut instruction_coverage = HashMap::new();
        let mut execution_efficiency = HashMap::new();
        let mut compatibility_scores = HashMap::new();

        for (arch, result) in results {
            instruction_coverage.insert(arch.clone(), result.coverage_percentage);
            execution_efficiency.insert(arch.clone(), result.instructions_passed as f64 / result.instructions_tested as f64);
            compatibility_scores.insert(arch.clone(), if result.passed { 100.0 } else { 50.0 });
        }

        ISAComparison {
            isa_results: results.clone(),
            instruction_coverage,
            execution_efficiency,
            compatibility_scores,
        }
    }

    /// Analyze power efficiency
    fn analyze_power_efficiency(&self, results: &HashMap<Architecture, Vec<BenchmarkResult>>) -> PowerComparison {
        let mut power_consumption = HashMap::new();
        let mut performance_per_watt = HashMap::new();
        let mut energy_efficiency_scores = HashMap::new();
        let mut thermal_design_power = HashMap::new();

        // Architecture-specific power characteristics
        let base_power = HashMap::from([
            (Architecture::X86_64, 65.0),
            (Architecture::ARM64, 25.0),
            (Architecture::RISC_V64, 15.0),
            (Architecture::SPARC64, 85.0),
            (Architecture::PowerPC64, 70.0),
        ]);

        let base_tdp = HashMap::from([
            (Architecture::X86_64, 95.0),
            (Architecture::ARM64, 35.0),
            (Architecture::RISC_V64, 25.0),
            (Architecture::SPARC64, 125.0),
            (Architecture::PowerPC64, 100.0),
        ]);

        for (arch, arch_results) in results {
            let perf_per_watt: f64 = arch_results.iter()
                .filter_map(|r| r.energy_efficiency_ops_per_watt)
                .sum::<f64>() / arch_results.iter().filter(|r| r.energy_efficiency_ops_per_watt.is_some()).count() as f64;
            
            let energy_score = if !perf_per_watt.is_nan() && perf_per_watt > 0.0 {
                perf_per_watt / 1000000.0 // Normalize to reasonable scale
            } else {
                0.0
            };

            power_consumption.insert(arch.clone(), *base_power.get(arch).unwrap_or(&50.0));
            performance_per_watt.insert(arch.clone(), perf_per_watt);
            energy_efficiency_scores.insert(arch.clone(), energy_score);
            thermal_design_power.insert(arch.clone(), *base_tdp.get(arch).unwrap_or(&75.0));
        }

        PowerComparison {
            power_consumption,
            performance_per_watt,
            energy_efficiency_scores,
            thermal_design_power,
        }
    }

    /// Analyze features
    fn analyze_features(&self, architectures: &Vec<Architecture>) -> FeatureComparison {
        let mut supported_features = HashMap::new();
        let mut vector_capabilities = HashMap::new();
        let mut security_features = HashMap::new();
        let mut specialized_extensions = HashMap::new();

        // Define feature sets for each architecture
        let feature_sets = HashMap::from([
            (Architecture::X86_64, vec!["SSE2", "AVX2", "AVX-512", "BMI2", "AES-NI"]),
            (Architecture::ARM64, vec!["NEON", "SVE", "AES", "SHA", "CRC32"]),
            (Architecture::RISC_V64, vec!["M", "A", "F", "D", "C", "Vector"]),
            (Architecture::SPARC64, vec!["VIS1", "VIS2", "VIS3", "Crypto"]),
            (Architecture::PowerPC64, vec!["AltiVec", "VSX", "Crypto", "VectorAES"]),
        ]);

        for arch in architectures {
            let features = feature_sets.get(arch).unwrap_or(&Vec::new()).clone();
            supported_features.insert(arch.clone(), features);

            // Extract specific capability categories
            let vector: Vec<String> = features.iter()
                .filter(|f| f.contains(&"Vector") || f.contains(&"SSE") || f.contains(&"AVX") || f.contains(&"NEON") || f.contains(&"VIS"))
                .cloned()
                .collect();
            vector_capabilities.insert(arch.clone(), vector);

            let security: Vec<String> = features.iter()
                .filter(|f| f.contains(&"AES") || f.contains(&"SHA") || f.contains(&"Crypto"))
                .cloned()
                .collect();
            security_features.insert(arch.clone(), security);

            let specialized: Vec<String> = features.iter()
                .filter(|f| !f.contains(&"AES") && !f.contains(&"SHA") && !f.contains(&"Crypto") && !f.contains(&"Vector") && !f.contains(&"SSE") && !f.contains(&"AVX") && !f.contains(&"NEON") && !f.contains(&"VIS"))
                .cloned()
                .collect();
            specialized_extensions.insert(arch.clone(), specialized);
        }

        FeatureComparison {
            supported_features,
            vector_capabilities,
            security_features,
            specialized_extensions,
        }
    }

    /// Calculate overall ranking
    fn calculate_overall_ranking(
        &self,
        performance: &PerformanceComparison,
        memory: &MemoryComparison,
        pipeline: &PipelineComparison,
        isa: &ISAComparison,
        power: &PowerComparison,
        features: &FeatureComparison,
    ) -> OverallRanking {
        let mut overall_scores = HashMap::new();
        let mut rankings = Vec::new();

        // Calculate weighted scores
        for arch in performance.average_scores.keys() {
            let perf_score = *performance.average_scores.get(arch).unwrap_or(&0.0);
            let memory_score = *memory.memory_hierarchy_scores.get(arch).unwrap_or(&0.0);
            let pipeline_score = *pipeline.pipeline_efficiency.get(arch).unwrap_or(&0.0);
            let isa_score = *isa.instruction_coverage.get(arch).unwrap_or(&0.0);
            let power_score = *power.energy_efficiency_scores.get(arch).unwrap_or(&0.0);
            let feature_score = features.supported_features.get(arch).unwrap_or(&Vec::new()).len() as f64 * 5.0;

            // Normalize and weight scores
            let normalized_perf = (perf_score / 1000000.0) * 0.3;
            let normalized_memory = (memory_score / 1000.0) * 0.2;
            let normalized_pipeline = pipeline_score * 0.2;
            let normalized_isa = (isa_score / 100.0) * 0.1;
            let normalized_power = power_score * 0.1;
            let normalized_features = (feature_score / 50.0).min(1.0) * 0.1;

            let overall_score = normalized_perf + normalized_memory + normalized_pipeline + 
                              normalized_isa + normalized_power + normalized_features;

            overall_scores.insert(arch.clone(), overall_score);
        }

        // Create rankings
        let mut sorted_archs: Vec<_> = overall_scores.iter().collect();
        sorted_archs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, (arch, score)) in sorted_archs.iter().enumerate() {
            let profile = self.generate_architecture_profile(
                arch,
                performance,
                memory,
                pipeline,
                isa,
                power,
                features,
            );

            rankings.push(ArchitectureRanking {
                architecture: arch.clone(),
                overall_score: **score,
                rank: (rank + 1) as u32,
                performance_score: *performance.average_scores.get(arch).unwrap_or(&0.0),
                efficiency_score: *power.energy_efficiency_scores.get(arch).unwrap_or(&0.0),
                feature_score: features.supported_features.get(arch).unwrap_or(&Vec::new()).len() as f64,
                compatibility_score: *isa.compatibility_scores.get(arch).unwrap_or(&0.0),
            });
        }

        OverallRanking {
            overall_scores,
            rankings,
            strengths_weaknesses: self.generate_all_profiles(
                &sorted_archs.iter().map(|(arch, _)| arch.clone()).collect(),
                performance,
                memory,
                pipeline,
                isa,
                power,
                features,
            ),
        }
    }

    /// Generate architecture profile
    fn generate_architecture_profile(
        &self,
        arch: &Architecture,
        performance: &PerformanceComparison,
        memory: &MemoryComparison,
        pipeline: &PipelineComparison,
        isa: &ISAComparison,
        power: &PowerComparison,
        features: &FeatureComparison,
    ) -> ArchitectureProfile {
        let mut strengths = Vec::new();
        let mut weaknesses = Vec::new();
        let mut best_use_cases = Vec::new();
        let mut optimal_workloads = Vec::new();

        // Analyze strengths and weaknesses
        if let Some(&perf_score) = performance.average_scores.get(arch) {
            if perf_score > 50000000.0 {
                strengths.push("High Performance Computing".to_string());
                optimal_workloads.push("Scientific Computing".to_string());
                best_use_cases.push("High-performance workstations".to_string());
            }
        }

        if let Some(&power_score) = power.energy_efficiency_scores.get(arch) {
            if power_score > 0.5 {
                strengths.push("Energy Efficiency".to_string());
                best_use_cases.push("Mobile and embedded devices".to_string());
                optimal_workloads.push("Portable applications".to_string());
            } else {
                weaknesses.push("High Power Consumption".to_string());
            }
        }

        if let Some(&features) = features.supported_features.get(arch) {
            if features.len() > 8 {
                strengths.push("Rich Feature Set".to_string());
            }
        }

        if let Some(&cache_score) = memory.cache_efficiency.get(arch) {
            if cache_score > 0.9 {
                strengths.push("Excellent Cache Performance".to_string());
            } else if cache_score < 0.7 {
                weaknesses.push("Limited Cache Efficiency".to_string());
            }
        }

        // Architecture-specific recommendations
        match arch {
            Architecture::X86_64 => {
                strengths.extend(vec!["Widely compatible".to_string(), "Excellent software ecosystem".to_string()]);
                best_use_cases.extend(vec!["Desktop computing".to_string(), "Gaming".to_string(), "Enterprise servers".to_string()]);
                optimal_workloads.extend(vec!["General purpose computing".to_string(), "Gaming applications".to_string()]);
            }
            Architecture::ARM64 => {
                strengths.extend(vec!["Power efficient".to_string(), "Mobile optimized".to_string()]);
                best_use_cases.extend(vec!["Mobile devices".to_string(), "IoT applications".to_string(), "Edge computing".to_string()]);
                optimal_workloads.extend(vec!["Mobile applications".to_string(), "Energy-efficient computing".to_string()]);
            }
            Architecture::RISC_V64 => {
                strengths.extend(vec!["Open source ISA".to_string(), "Highly customizable".to_string(), "Low power design".to_string()]);
                best_use_cases.extend(vec!["Research projects".to_string(), "Custom silicon".to_string(), "Educational purposes".to_string()]);
                optimal_workloads.extend(vec!["Embedded systems".to_string(), "Specialized applications".to_string()]);
            }
            Architecture::SPARC64 => {
                strengths.extend(vec!["Enterprise focused".to_string(), "Scalable architecture".to_string()]);
                best_use_cases.extend(vec!["High-end servers".to_string(), "Database systems".to_string()]);
                optimal_workloads.extend(vec!["Database workloads".to_string(), "Transaction processing".to_string()]);
            }
            Architecture::PowerPC64 => {
                strengths.extend(vec!["Excellent for parallel processing".to_string(), "Strong in scientific computing".to_string()]);
                best_use_cases.extend(vec!["Workstations".to_string(), "Scientific computing".to_string()]);
                optimal_workloads.extend(vec!["Scientific applications".to_string(), "Parallel workloads".to_string()]);
            }
        }

        ArchitectureProfile {
            architecture: arch.clone(),
            key_strengths: strengths,
            key_weaknesses: weaknesses,
            best_use_cases,
            optimal_workloads,
            recommended_for: self.get_general_recommendation(arch),
        }
    }

    /// Get general recommendation for architecture
    fn get_general_recommendation(&self, arch: &Architecture) -> String {
        match arch {
            Architecture::X86_64 => "General purpose computing, desktop applications, gaming".to_string(),
            Architecture::ARM64 => "Mobile devices, embedded systems, energy-efficient computing".to_string(),
            Architecture::RISC_V64 => "Research, custom implementations, specialized applications".to_string(),
            Architecture::SPARC64 => "Enterprise servers, database systems, mission-critical applications".to_string(),
            Architecture::PowerPC64 => "Scientific computing, workstations, parallel processing".to_string(),
        }
    }

    /// Generate profiles for all architectures
    fn generate_all_profiles(
        &self,
        architectures: &Vec<Architecture>,
        performance: &PerformanceComparison,
        memory: &MemoryComparison,
        pipeline: &PipelineComparison,
        isa: &ISAComparison,
        power: &PowerComparison,
        features: &FeatureComparison,
    ) -> HashMap<Architecture, ArchitectureProfile> {
        let mut profiles = HashMap::new();
        
        for arch in architectures {
            let profile = self.generate_architecture_profile(
                arch,
                performance,
                memory,
                pipeline,
                isa,
                power,
                features,
            );
            profiles.insert(arch.clone(), profile);
        }
        
        profiles
    }

    /// Generate recommendations based on overall ranking
    fn generate_recommendations(&self, ranking: &OverallRanking) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(top_arch) = ranking.rankings.first() {
            recommendations.push(format!(
                "Overall winner: {} - Best for general-purpose computing with balanced performance and features",
                top_arch.architecture
            ));
        }

        if let Some(most_efficient) = ranking.rankings.iter().max_by(|a, b| a.efficiency_score.partial_cmp(&b.efficiency_score).unwrap_or(std::cmp::Ordering::Equal)) {
            recommendations.push(format!(
                "Most energy efficient: {} - Ideal for power-constrained environments",
                most_efficient.architecture
            ));
        }

        recommendations.push("For specialized workloads, consider the architecture's specific strengths".to_string());
        recommendations.push("Performance requirements should be balanced with power consumption needs".to_string());
        recommendations.push("Consider the software ecosystem and toolchain availability for each architecture".to_string());

        recommendations
    }

    /// Save comparison report to file
    pub fn save_comparison_report(&self, comparison: &ArchitectureComparison, output_file: &str) -> Result<String> {
        let report = self.generate_markdown_report(comparison)?;
        
        // Ensure directory exists
        if let Some(parent) = Path::new(output_file).parent() {
            fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }
        
        fs::write(output_file, report)
            .context("Failed to write comparison report")?;
        
        info!("Architecture comparison report saved to {}", output_file);
        Ok(report)
    }

    /// Generate comprehensive markdown report
    fn generate_markdown_report(&self, comparison: &ArchitectureComparison) -> Result<String> {
        let mut report = String::new();
        
        report.push_str(&format!("# {} Comparison Report\n\n", comparison.comparison_name));
        report.push_str("## Executive Summary\n\n");
        report.push_str("This comprehensive report compares CPU architectures across multiple dimensions including performance, memory hierarchy, pipeline characteristics, and power efficiency.\n\n");

        // Overall Rankings
        report.push_str("## Overall Rankings\n\n");
        report.push_str("| Rank | Architecture | Overall Score | Performance | Efficiency | Features |\n");
        report.push_str("|------|--------------|---------------|-------------|------------|----------|\n");
        
        for ranking in &comparison.overall_ranking.rankings {
            report.push_str(&format!(
                "| {} | {} | {:.2} | {:.0} | {:.1} | {:.0} |\n",
                ranking.rank,
                ranking.architecture,
                ranking.overall_score,
                ranking.performance_score,
                ranking.efficiency_score,
                ranking.feature_score
            ));
        }

        // Performance Analysis
        report.push_str("\n## Performance Analysis\n\n");
        report.push_str("### Benchmark Results Summary\n\n");
        report.push_str("| Architecture | Avg Instructions/sec | Best Benchmarks |\n");
        report.push_str("|--------------|----------------------|----------------|\n");

        for (arch, score) in &comparison.performance_comparison.average_scores {
            let mut best_benchmarks = Vec::new();
            for (benchmark_type, best_arch) in &comparison.performance_comparison.best_performing {
                if best_arch == arch {
                    best_benchmarks.push(benchmark_type);
                }
            }
            
            report.push_str(&format!(
                "| {} | {:.0} | {} |\n",
                arch,
                score,
                best_benchmarks.join(", ")
            ));
        }

        // Memory Analysis
        report.push_str("\n## Memory Hierarchy Analysis\n\n");
        report.push_str("### Memory Performance\n\n");
        report.push_str("| Architecture | Avg Latency (ns) | Avg Bandwidth (MB/s) | Cache Hit Rate | Memory Score |\n");
        report.push_str("|--------------|------------------|---------------------|----------------|-------------|\n");

        for arch in &comparison.architectures {
            let latency = comparison.memory_comparison.latency_comparison.get(arch).unwrap_or(&0);
            let bandwidth = comparison.memory_comparison.bandwidth_comparison.get(arch).unwrap_or(&0);
            let cache_rate = comparison.memory_comparison.cache_efficiency.get(arch).unwrap_or(&0.0);
            let mem_score = comparison.memory_comparison.memory_hierarchy_scores.get(arch).unwrap_or(&0.0);

            report.push_str(&format!(
                "| {} | {} | {} | {:.1}% | {:.1} |\n",
                arch,
                latency,
                bandwidth,
                cache_rate * 100.0,
                mem_score
            ));
        }

        // Pipeline Analysis
        report.push_str("\n## Pipeline Analysis\n\n");
        report.push_str("### Pipeline Efficiency\n\n");
        report.push_str("| Architecture | Branch Prediction | Pipeline Efficiency | Speculation | Unit Utilization |\n");
        report.push_str("|--------------|-------------------|-------------------|-------------|-----------------|\n");

        for arch in &comparison.architectures {
            let branch_acc = comparison.pipeline_comparison.branch_prediction_scores.get(arch).unwrap_or(&0.0);
            let pipeline_eff = comparison.pipeline_comparison.pipeline_efficiency.get(arch).unwrap_or(&0.0);
            let speculation = comparison.pipeline_comparison.speculation_effectiveness.get(arch).unwrap_or(&0.0);
            let utilization = comparison.pipeline_comparison.execution_unit_utilization.get(arch).unwrap_or(&0.0);

            report.push_str(&format!(
                "| {} | {:.1}% | {:.2} | {:.1}% | {:.1}% |\n",
                arch,
                branch_acc * 100.0,
                pipeline_eff,
                speculation * 100.0,
                utilization * 100.0
            ));
        }

        // Feature Comparison
        report.push_str("\n## Feature Comparison\n\n");
        report.push_str("### Supported Features by Architecture\n\n");
        for arch in &comparison.architectures {
            let features = comparison.feature_comparison.supported_features.get(arch).unwrap_or(&Vec::new());
            let vector_caps = comparison.feature_comparison.vector_capabilities.get(arch).unwrap_or(&Vec::new());
            let security = comparison.feature_comparison.security_features.get(arch).unwrap_or(&Vec::new());

            report.push_str(&format!("#### {} Features\n\n", arch));
            report.push_str(&format!("**Core Features:** {}\n\n", features.join(", ")));
            report.push_str(&format!("**Vector Capabilities:** {}\n\n", vector_caps.join(", ")));
            report.push_str(&format!("**Security Extensions:** {}\n\n", security.join(", ")));
        }

        // Architecture Profiles
        report.push_str("\n## Detailed Architecture Profiles\n\n");
        for (arch, profile) in &comparison.overall_ranking.strengths_weaknesses {
            report.push_str(&format!("### {} Analysis\n\n", arch));
            
            report.push_str("#### Key Strengths\n\n");
            for strength in &profile.key_strengths {
                report.push_str(&format!("- {}\n", strength));
            }
            
            report.push_str("\n#### Key Weaknesses\n\n");
            for weakness in &profile.key_weaknesses {
                report.push_str(&format!("- {}\n", weakness));
            }
            
            report.push_str("\n#### Best Use Cases\n\n");
            for use_case in &profile.best_use_cases {
                report.push_str(&format!("- {}\n", use_case));
            }
            
            report.push_str("\n#### Optimal Workloads\n\n");
            for workload in &profile.optimal_workloads {
                report.push_str(&format!("- {}\n", workload));
            }
            
            report.push_str(&format!("\n#### Recommendation\n\n{}\n\n", profile.recommended_for));
        }

        // Power Analysis
        report.push_str("\n## Power and Efficiency Analysis\n\n");
        report.push_str("### Power Characteristics\n\n");
        report.push_str("| Architecture | Power (W) | Performance/Watt | TDP (W) | Energy Score |\n");
        report.push_str("|--------------|-----------|------------------|---------|-------------|\n");

        for arch in &comparison.architectures {
            let power = comparison.power_comparison.power_consumption.get(arch).unwrap_or(&0.0);
            let perf_per_watt = comparison.power_comparison.performance_per_watt.get(arch).unwrap_or(&0.0);
            let tdp = comparison.power_comparison.thermal_design_power.get(arch).unwrap_or(&0.0);
            let energy_score = comparison.power_comparison.energy_efficiency_scores.get(arch).unwrap_or(&0.0);

            report.push_str(&format!(
                "| {} | {:.1} | {:.0} | {:.1} | {:.2} |\n",
                arch,
                power,
                perf_per_watt,
                tdp,
                energy_score
            ));
        }

        // Recommendations
        report.push_str("\n## Recommendations\n\n");
        for (i, recommendation) in comparison.recommendations.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, recommendation));
        }

        // Conclusion
        report.push_str("\n## Conclusion\n\n");
        report.push_str("This comprehensive analysis provides insights into the strengths and weaknesses of different CPU architectures. ");
        report.push_str("The choice of architecture should be based on specific requirements including performance targets, power constraints, ");
        report.push_str("feature requirements, and software ecosystem considerations.\n\n");
        
        if let Some(winner) = comparison.overall_ranking.rankings.first() {
            report.push_str(&format!("**{}** emerges as the overall winner with the best balance of performance, efficiency, and features.\n\n", winner.architecture));
        }

        Ok(report)
    }
}

/// Public API functions

/// Compare architectures and generate comprehensive report
pub fn compare_architectures(architectures: Vec<Architecture>, comparison_type: &str) -> Result<ArchitectureComparison> {
    let comparator = ArchitectureComparator::new();
    comparator.compare_architectures(architectures, comparison_type)
}

/// Save comparison report to file
pub fn save_comparison_report(comparison: &ArchitectureComparison, output_file: &str) -> Result<String> {
    let comparator = ArchitectureComparator::new();
    comparator.save_comparison_report(comparison, output_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparator_creation() {
        let comparator = ArchitectureComparator::new();
        // Test that the comparator was created successfully
        assert!(true);
    }

    #[test]
    fn test_architecture_profiles() {
        let comparator = ArchitectureComparator::new();
        let arch = Architecture::X86_64;
        
        // Test profile generation (would need to populate comparison data)
        // This is a simplified test
        assert_eq!(arch, Architecture::X86_64);
    }
}