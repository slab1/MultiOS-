//! Example: Custom CPU Benchmark Implementation
//! 
//! This example demonstrates how to create custom CPU benchmarks
//! using the MultiOS benchmarking framework.

use multios_benchmarking::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Custom benchmark that tests prime number calculation
pub struct PrimeCalculation;

impl PrimeCalculation {
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate prime numbers using Sieve of Eratosthenes
    fn calculate_primes(limit: usize) -> Vec<usize> {
        let mut is_prime = vec![true; limit + 1];
        is_prime[0] = false;
        is_prime[1] = false;
        
        for i in 2..=((limit as f64).sqrt() as usize) {
            if is_prime[i] {
                for j in (i * i..=limit).step_by(i) {
                    if j <= limit {
                        is_prime[j] = false;
                    }
                }
            }
        }
        
        (0..=limit)
            .enumerate()
            .filter(|(_, is_p)| *is_p)
            .map(|(i, _)| i)
            .collect()
    }
}

impl Benchmark for PrimeCalculation {
    fn name(&self) -> &str {
        "Prime Number Calculation"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut total_primes = 0u64;
        let limit = 10000; // Calculate primes up to 10,000
        
        for _ in 0..iterations {
            let primes = Self::calculate_primes(limit);
            total_primes += primes.len() as u64;
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("prime_limit".to_string(), limit.to_string());
        metadata.insert("total_primes_found".to_string(), total_primes.to_string());
        metadata.insert("average_primes_per_iteration".to_string(), 
            (total_primes / iterations).to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * 1229.0, // Approximate primes found per calculation
            unit: "prime_calculations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Custom benchmark for string processing performance
pub struct StringProcessing;

impl StringProcessing {
    pub fn new() -> Self {
        Self
    }
    
    fn process_string(text: &str) -> String {
        let mut result = String::new();
        
        for word in text.split_whitespace() {
            let reversed: String = word.chars().rev().collect();
            result.push_str(&reversed);
            result.push(' ');
        }
        
        result.trim().to_string()
    }
}

impl Benchmark for StringProcessing {
    fn name(&self) -> &str {
        "String Processing"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let test_text = "The quick brown fox jumps over the lazy dog. \
                        This is a test sentence for string processing benchmarks.";
        
        let start = Instant::now();
        let mut total_bytes = 0u64;
        
        for _ in 0..iterations {
            let processed = Self::process_string(test_text);
            total_bytes += processed.len() as u64;
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("text_length".to_string(), test_text.len().to_string());
        metadata.insert("total_bytes_processed".to_string(), total_bytes.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: (total_bytes as f64) / elapsed.as_secs_f64(),
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Example benchmark suite combining custom benchmarks
pub struct CustomCpuBenchmarkSuite {
    pub prime_calc: PrimeCalculation,
    pub string_processing: StringProcessing,
}

impl CustomCpuBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            prime_calc: PrimeCalculation::new(),
            string_processing: StringProcessing::new(),
        }
    }
    
    /// Run all custom CPU benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use multios_benchmarking::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(true);
        let benchmarks = vec![
            &self.prime_calc,
            &self.string_processing,
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Custom CPU Benchmark Example");
    println!("============================");
    
    let suite = CustomCpuBenchmarkSuite::new();
    let iterations = 1000;
    
    println!("Running {} iterations per benchmark...", iterations);
    
    let results = suite.run_all(iterations)?;
    
    println!("\nBenchmark Results:");
    println!("==================");
    
    for result in results {
        println!("\n{}:", result.name);
        println!("  Duration: {:?}", result.duration);
        println!("  Operations/Second: {:.2}", result.operations_per_second);
        println!("  Throughput: {}", 
            if result.throughput > 1_000_000.0 {
                format!("{:.2} M{}", result.throughput / 1_000_000.0, result.unit)
            } else if result.throughput > 1_000.0 {
                format!("{:.2} k{}", result.throughput / 1_000.0, result.unit)
            } else {
                format!("{:.2} {}", result.throughput, result.unit)
            });
        
        println!("  Metadata:");
        for (key, value) in &result.metadata {
            println!("    {}: {}", key, value);
        }
    }
    
    println!("\nExample completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prime_calculation() {
        let bench = PrimeCalculation::new();
        let result = bench.run(10).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.metadata.contains_key("total_primes_found"));
    }
    
    #[test]
    fn test_string_processing() {
        let bench = StringProcessing::new();
        let result = bench.run(10).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.metadata.contains_key("total_bytes_processed"));
    }
}