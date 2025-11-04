//! CPU Performance Benchmarks
//! 
//! This module implements comprehensive CPU performance benchmarks including:
//! - Integer operations
//! - Floating-point operations
//! - Matrix calculations
//! - Cryptographic operations

use super::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;

const MATRIX_SIZE: usize = 64;

/// Integer operation benchmark
pub struct IntegerBenchmarks;

impl IntegerBenchmarks {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for IntegerBenchmarks {
    fn name(&self) -> &str {
        "CPU Integer Operations"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut result = 0i64;
        
        for i in 0..iterations {
            // Mix of integer operations
            result = result.wrapping_add(i);
            result = result.wrapping_mul(i.wrapping_add(1));
            result ^= i.wrapping_mul(123456789);
            
            // Division and modulo operations
            if i > 0 {
                result = result.wrapping_div(i % 1000 + 1);
                result %= 1000000007;
            }
            
            // Bit operations
            result = result.wrapping_shl(1) | result.wrapping_shr(1);
            result &= 0xFFFF_FFFF_FFFF_FFFF;
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("result".to_string(), result.to_string());
        metadata.insert("operations_per_iteration".to_string(), "8".to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * 8.0, // 8 operations per iteration
            unit: "integer_operations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Floating-point operation benchmark
pub struct FloatingPointBenchmarks;

impl FloatingPointBenchmarks {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for FloatingPointBenchmarks {
    fn name(&self) -> &str {
        "CPU Floating-Point Operations"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut result = 0.0f64;
        
        for i in 0..iterations {
            let x = (i as f64) * 0.001;
            
            // Mix of floating-point operations
            result += x.sin() * x.cos();
            result = result.exp().ln();
            result = result.sqrt().powf(0.5);
            
            // Trigonometric and hyperbolic functions
            result += x.tan().atanh();
            
            // Power and logarithmic operations
            if i % 100 == 0 {
                result = result.powf(2.0).log10();
            }
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("final_result".to_string(), result.to_string());
        metadata.insert("operations_per_iteration".to_string(), "10".to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * 10.0, // 10 operations per iteration
            unit: "fp_operations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Matrix multiplication benchmark
pub struct MatrixMultiplication;

impl MatrixMultiplication {
    pub fn new() -> Self {
        Self
    }
    
    fn multiply_matrices(a: &[f64], b: &[f64], size: usize) -> Vec<f64> {
        let mut result = vec![0.0; size * size];
        
        for i in 0..size {
            for j in 0..size {
                let mut sum = 0.0;
                for k in 0..size {
                    sum += a[i * size + k] * b[k * size + j];
                }
                result[i * size + j] = sum;
            }
        }
        
        result
    }
}

impl Benchmark for MatrixMultiplication {
    fn name(&self) -> &str {
        "Matrix Multiplication"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        // Create test matrices
        let mut matrix_a = vec![0.0; MATRIX_SIZE * MATRIX_SIZE];
        let mut matrix_b = vec![0.0; MATRIX_SIZE * MATRIX_SIZE];
        
        // Initialize matrices with test data
        for i in 0..MATRIX_SIZE * MATRIX_SIZE {
            matrix_a[i] = (i as f64 * 0.001).sin();
            matrix_b[i] = (i as f64 * 0.002).cos();
        }
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _result = Self::multiply_matrices(&matrix_a, &matrix_b, MATRIX_SIZE);
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        // Calculate theoretical operations (n^3 for matrix multiplication)
        let ops_per_multiplication = (MATRIX_SIZE as f64).powi(3);
        
        let mut metadata = HashMap::new();
        metadata.insert("matrix_size".to_string(), MATRIX_SIZE.to_string());
        metadata.insert("ops_per_multiplication".to_string(), ops_per_multiplication.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * ops_per_multiplication,
            unit: "mathematical_ops/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Cryptographic operation benchmark
pub struct CryptoBenchmarks;

impl CryptoBenchmarks {
    pub fn new() -> Self {
        Self
    }
    
    fn simple_hash(data: &[u8]) -> u32 {
        let mut hash = 0u32;
        for &byte in data {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }
    
    fn xor_cipher(data: &[u8], key: u8) -> Vec<u8> {
        data.iter().map(|&b| b ^ key).collect()
    }
}

impl Benchmark for CryptoBenchmarks {
    fn name(&self) -> &str {
        "Cryptographic Operations"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let test_data = b"Hello, World! This is a test message for cryptographic benchmarking.";
        let key = 0x5A;
        
        let start = Instant::now();
        let mut total_bytes = 0u64;
        
        for _ in 0..iterations {
            // Hash operation
            let hash = Self::simple_hash(test_data);
            total_bytes += test_data.len() as u64;
            
            // XOR cipher operations
            let encrypted = Self::xor_cipher(test_data, key);
            let _decrypted = Self::xor_cipher(&encrypted, key);
            total_bytes += encrypted.len() as u64 * 2; // Both encryption and decryption
            
            // Prevent optimization
            let _ = hash;
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("test_data_size".to_string(), test_data.len().to_string());
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

/// SIMD operations benchmark (if available)
pub struct SimdBenchmarks;

impl SimdBenchmarks {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for SimdBenchmarks {
    fn name(&self) -> &str {
        "SIMD Operations"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::CPU
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let vector_size = 256;
        let mut data = vec![1.0f32; vector_size];
        
        let start = Instant::now();
        let mut result = 0.0f32;
        
        for _ in 0..iterations {
            // Simulate SIMD operations using regular operations
            // In a real implementation, this would use std::simd or similar
            for i in 0..vector_size {
                data[i] = data[i].mul_add(data[i], 2.0);
                result += data[i];
            }
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("vector_size".to_string(), vector_size.to_string());
        metadata.insert("vector_operations".to_string(), (vector_size * 2).to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * (vector_size as f64 * 2.0),
            unit: "simd_operations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Complete CPU benchmarking suite
pub struct CpuBenchmarkSuite {
    pub integer_bench: IntegerBenchmarks,
    pub float_bench: FloatingPointBenchmarks,
    pub matrix_bench: MatrixMultiplication,
    pub crypto_bench: CryptoBenchmarks,
    pub simd_bench: SimdBenchmarks,
}

impl CpuBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            integer_bench: IntegerBenchmarks::new(),
            float_bench: FloatingPointBenchmarks::new(),
            matrix_bench: MatrixMultiplication::new(),
            crypto_bench: CryptoBenchmarks::new(),
            simd_bench: SimdBenchmarks::new(),
        }
    }
    
    /// Run all CPU benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use super::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(false);
        let benchmarks: Vec<_> = vec![
            &self.integer_bench,
            &self.float_bench,
            &self.matrix_bench,
            &self.crypto_bench,
            &self.simd_bench,
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integer_benchmark() {
        let bench = IntegerBenchmarks::new();
        let result = bench.run(1000).unwrap();
        assert!(result.operations_per_second > 0.0);
    }
    
    #[test]
    fn test_matrix_multiplication() {
        let bench = MatrixMultiplication::new();
        let result = bench.run(10).unwrap();
        assert!(result.operations_per_second > 0.0);
    }
}