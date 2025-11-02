//! System Call Performance Benchmarks
//! 
//! This module implements system call performance benchmarks including:
//! - Basic system call overhead
//! - Process creation and management
//! - Inter-process communication
//! - File and I/O system calls
//! - Memory management system calls

use super::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::process::{Command, Stdio};
use libc::{c_int, pid_t};

const BENCHMARK_ITERATIONS: usize = 10000;

/// System call overhead benchmark
pub struct SyscallOverhead;

impl SyscallOverhead {
    pub fn new() -> Self {
        Self
    }
    
    /// Measure basic system call overhead
    fn measure_syscall_overhead(iterations: u64) -> Duration {
        let start = Instant::now();
        
        // Measure getpid() system call overhead
        for _ in 0..iterations {
            unsafe { libc::getpid() };
        }
        
        start.elapsed()
    }
    
    /// Measure time-related system call overhead
    fn measure_time_syscalls(iterations: u64) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            unsafe {
                let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
                libc::gettimeofday(&mut tv, std::ptr::null_mut());
            };
        }
        
        start.elapsed()
    }
}

impl Benchmark for SyscallOverhead {
    fn name(&self) -> &str {
        "System Call Overhead"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Syscalls
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // Measure different types of system calls
        let getpid_time = Self::measure_syscall_overhead(iterations / 2);
        let gettimeofday_time = Self::measure_time_syscalls(iterations / 2);
        
        let total_time = start.elapsed();
        
        // Calculate per-call overhead
        let getpid_overhead_us = getpid_time.as_micros() as f64 / (iterations / 2) as f64;
        let gettimeofday_overhead_us = gettimeofday_time.as_micros() as f64 / (iterations / 2) as f64;
        
        let mut metadata = HashMap::new();
        metadata.insert("getpid_calls".to_string(), (iterations / 2).to_string());
        metadata.insert("getpid_overhead_us".to_string(), getpid_overhead_us.to_string());
        metadata.insert("gettimeofday_calls".to_string(), (iterations / 2).to_string());
        metadata.insert("gettimeofday_overhead_us".to_string(), gettimeofday_overhead_us.to_string());
        metadata.insert("total_calls".to_string(), iterations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: total_time,
            iterations,
            operations_per_second: (iterations as f64) / total_time.as_secs_f64(),
            throughput: (iterations as f64) / total_time.as_secs_f64(),
            unit: "syscalls/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Process creation benchmark
pub struct ProcessCreation;

impl ProcessCreation {
    pub fn new() -> Self {
        Self
    }
    
    /// Measure fork() system call performance
    fn measure_fork_performance(iterations: u64) -> Result<(Duration, u64), Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut successful_forks = 0u64;
        
        for _ in 0..iterations {
            unsafe {
                let pid = libc::fork();
                if pid == 0 {
                    // Child process - exit immediately
                    libc::exit(0);
                } else if pid > 0 {
                    // Parent process - wait for child
                    let mut status = 0;
                    libc::waitpid(pid, &mut status, 0);
                    successful_forks += 1;
                }
            }
        }
        
        let elapsed = start.elapsed();
        Ok((elapsed, successful_forks))
    }
    
    /// Measure exec() system call performance
    fn measure_exec_performance(iterations: u64) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        for _ in 0..iterations {
            // Use a simple command that exits immediately
            let _output = Command::new("true")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output()?;
        }
        
        Ok(start.elapsed())
    }
}

impl Benchmark for ProcessCreation {
    fn name(&self) -> &str {
        "Process Creation"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Syscalls
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        let test_iterations = std::cmp::min(iterations as usize, 100); // Limit for fork test
        let (fork_time, successful_forks) = Self::measure_fork_performance(test_iterations as u64)?;
        let exec_time = Self::measure_exec_performance(iterations / 2)?;
        
        let total_time = start.elapsed();
        
        let fork_rate = (successful_forks as f64) / fork_time.as_secs_f64();
        let exec_rate = (iterations as f64 / 2.0) / exec_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("fork_attempts".to_string(), test_iterations.to_string());
        metadata.insert("successful_forks".to_string(), successful_forks.to_string());
        metadata.insert("fork_time_ms".to_string(), fork_time.as_millis().to_string());
        metadata.insert("exec_time_ms".to_string(), exec_time.as_millis().to_string());
        metadata.insert("fork_rate_per_sec".to_string(), fork_rate.to_string());
        metadata.insert("exec_rate_per_sec".to_string(), exec_rate.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: total_time,
            iterations,
            operations_per_second: (fork_rate + exec_rate) / 2.0,
            throughput: fork_rate,
            unit: "processes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Thread creation benchmark
pub struct ThreadCreation;

impl ThreadCreation {
    pub fn new() -> Self {
        Self
    }
    
    /// Measure thread creation overhead
    fn measure_thread_creation(iterations: u64) -> (Duration, u64) {
        let start = Instant::now();
        let mut created_threads = 0u64;
        
        let handles: Vec<_> = (0..iterations)
            .map(|_| {
                thread::spawn(|| {
                    // Small amount of work to ensure thread is actually created
                    let _ = 42 + 24;
                })
            })
            .collect();
        
        // Wait for all threads to complete
        for handle in handles {
            if handle.join().is_ok() {
                created_threads += 1;
            }
        }
        
        let elapsed = start.elapsed();
        (elapsed, created_threads)
    }
}

impl Benchmark for ThreadCreation {
    fn name(&self) -> &str {
        "Thread Creation"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Syscalls
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let test_iterations = std::cmp::min(iterations as usize, 1000); // Limit for thread test
        let (elapsed, created_threads) = Self::measure_thread_creation(test_iterations as u64);
        
        let threads_per_sec = (created_threads as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("attempted_threads".to_string(), test_iterations.to_string());
        metadata.insert("created_threads".to_string(), created_threads.to_string());
        metadata.insert("creation_time_ms".to_string(), elapsed.as_millis().to_string());
        metadata.insert("success_rate_percent".to_string(), 
            (created_threads as f64 / test_iterations as f64 * 100.0).to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations: created_threads,
            operations_per_second: threads_per_sec,
            throughput: threads_per_sec,
            unit: "threads/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// File I/O system calls benchmark
pub struct FileSyscalls;

impl FileSyscalls {
    pub fn new() -> Self {
        Self
    }
    
    /// Measure open/close system call performance
    fn measure_open_close(iterations: u64) -> Result<(Duration, u64), Box<dyn std::error::Error>> {
        let start = Instant::now();
        let mut successful_ops = 0u64;
        
        let test_dir = std::path::Path::new("/tmp");
        
        for i in 0..iterations {
            let filename = format!("{}/test_file_{}.txt", test_dir.display(), i);
            
            unsafe {
                let fd = libc::open(
                    filename.as_ptr(),
                    libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
                    0o644
                );
                
                if fd >= 0 {
                    libc::close(fd);
                    successful_ops += 1;
                    
                    // Clean up
                    let _ = std::fs::remove_file(filename);
                }
            }
        }
        
        let elapsed = start.elapsed();
        Ok((elapsed, successful_ops))
    }
    
    /// Measure read/write system call performance
    fn measure_read_write(iterations: u64) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        let test_file = "/tmp/syscall_benchmark.dat";
        let buffer = vec![0u8; 1024];
        
        for _ in 0..iterations {
            unsafe {
                let fd = libc::open(
                    test_file.as_ptr(),
                    libc::O_CREAT | libc::O_WRONLY | libc::O_TRUNC,
                    0o644
                );
                
                if fd >= 0 {
                    let _ = libc::write(fd, buffer.as_ptr() as *const libc::c_void, buffer.len());
                    libc::close(fd);
                }
            }
            
            // Clean up
            let _ = std::fs::remove_file(test_file);
        }
        
        Ok(start.elapsed())
    }
}

impl Benchmark for FileSyscalls {
    fn name(&self) -> &str {
        "File System Calls"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Syscalls
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let test_iterations = std::cmp::min(iterations as usize, 1000); // Limit for file operations
        let (open_close_time, successful_ops) = Self::measure_open_close(test_iterations as u64)?;
        let read_write_time = Self::measure_read_write(test_iterations as u64)?;
        
        let open_close_rate = (successful_ops as f64) / open_close_time.as_secs_f64();
        let read_write_rate = (test_iterations as f64) / read_write_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("open_close_operations".to_string(), successful_ops.to_string());
        metadata.insert("open_close_time_ms".to_string(), open_close_time.as_millis().to_string());
        metadata.insert("open_close_rate".to_string(), open_close_rate.to_string());
        metadata.insert("read_write_operations".to_string(), test_iterations.to_string());
        metadata.insert("read_write_time_ms".to_string(), read_write_time.as_millis().to_string());
        metadata.insert("read_write_rate".to_string(), read_write_rate.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: open_close_time + read_write_time,
            iterations: successful_ops + test_iterations as u64,
            operations_per_second: (open_close_rate + read_write_rate) / 2.0,
            throughput: open_close_rate + read_write_rate,
            unit: "file_ops/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// IPC (Inter-Process Communication) system calls benchmark
pub struct IpcSyscalls;

impl IpcSyscalls {
    pub fn new() -> Self {
        Self
    }
    
    /// Measure pipe creation performance
    fn measure_pipe_performance(iterations: u64) -> (Duration, u64) {
        let start = Instant::now();
        let mut successful_pipes = 0u64;
        
        for _ in 0..iterations {
            unsafe {
                let mut pipefd = [0i32; 2];
                if libc::pipe(pipefd.as_mut_ptr()) == 0 {
                    libc::close(pipefd[0]);
                    libc::close(pipefd[1]);
                    successful_pipes += 1;
                }
            }
        }
        
        let elapsed = start.elapsed();
        (elapsed, successful_pipes)
    }
    
    /// Measure signal handling performance
    fn measure_signal_performance(iterations: u64) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            unsafe {
                // Send a signal to self (SIGUSR1)
                libc::kill(libc::getpid(), libc::SIGUSR1);
            }
        }
        
        start.elapsed()
    }
}

impl Benchmark for IpcSyscalls {
    fn name(&self) -> &str {
        "IPC System Calls"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::Syscalls
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let test_iterations = std::cmp::min(iterations as usize, 5000);
        let (pipe_time, successful_pipes) = Self::measure_pipe_performance(test_iterations as u64);
        let signal_time = Self::measure_signal_performance(test_iterations as u64);
        
        let pipe_rate = (successful_pipes as f64) / pipe_time.as_secs_f64();
        let signal_rate = (test_iterations as f64) / signal_time.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("pipe_operations".to_string(), successful_pipes.to_string());
        metadata.insert("pipe_time_ms".to_string(), pipe_time.as_millis().to_string());
        metadata.insert("pipe_rate".to_string(), pipe_rate.to_string());
        metadata.insert("signal_operations".to_string(), test_iterations.to_string());
        metadata.insert("signal_time_ms".to_string(), signal_time.as_millis().to_string());
        metadata.insert("signal_rate".to_string(), signal_rate.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: pipe_time + signal_time,
            iterations: successful_pipes + test_iterations as u64,
            operations_per_second: (pipe_rate + signal_rate) / 2.0,
            throughput: pipe_rate + signal_rate,
            unit: "ipc_ops/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Complete system call benchmarking suite
pub struct SyscallBenchmarkSuite {
    pub syscall_overhead: SyscallOverhead,
    pub process_creation: ProcessCreation,
    pub thread_creation: ThreadCreation,
    pub file_syscalls: FileSyscalls,
    pub ipc_syscalls: IpcSyscalls,
}

impl SyscallBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            syscall_overhead: SyscallOverhead::new(),
            process_creation: ProcessCreation::new(),
            thread_creation: ThreadCreation::new(),
            file_syscalls: FileSyscalls::new(),
            ipc_syscalls: IpcSyscalls::new(),
        }
    }
    
    /// Run all system call benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use super::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(false);
        let benchmarks: Vec<_> = vec![
            &self.syscall_overhead,
            &self.thread_creation,
            &self.file_syscalls,
            &self.ipc_syscalls,
            &self.process_creation, // Run last as it's more resource intensive
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syscall_overhead() {
        let bench = SyscallOverhead::new();
        let result = bench.run(1000).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.metadata.contains_key("getpid_overhead_us"));
    }
    
    #[test]
    fn test_thread_creation() {
        let bench = ThreadCreation::new();
        let result = bench.run(10).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.metadata.contains_key("created_threads"));
    }
}