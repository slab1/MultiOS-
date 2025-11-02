//! File System I/O Benchmarks
//! 
//! This module implements comprehensive file system performance benchmarks including:
//! - Sequential read/write operations
//! - Random access patterns
//! - Metadata operations
//! - File system specific performance

use super::{Benchmark, BenchmarkCategory, BenchmarkResult};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use walkdir::WalkDir;

const TEST_FILE_SIZE: usize = 64 * 1024 * 1024; // 64MB test file
const SMALL_FILE_SIZE: usize = 4 * 1024; // 4KB small file
const MEDIUM_FILE_SIZE: usize = 64 * 1024; // 64KB medium file

/// Sequential file read benchmark
pub struct SequentialFileRead;

impl SequentialFileRead {
    pub fn new() -> Self {
        Self
    }
    
    /// Create test file for reading
    fn create_test_file(path: &Path, size: usize) -> Result<File, Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
            
        let buffer = vec![0u8; 8192]; // 8KB buffer
        let mut written = 0;
        
        while written < size {
            let to_write = std::cmp::min(buffer.len(), size - written);
            file.write_all(&buffer[..to_write])?;
            written += to_write;
        }
        
        file.sync_all()?;
        file.seek(SeekFrom::Start(0))?;
        
        Ok(file)
    }
}

impl Benchmark for SequentialFileRead {
    fn name(&self) -> &str {
        "Sequential File Read"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::FileSystem
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test_read.dat");
        
        // Create test file
        Self::create_test_file(&test_file, TEST_FILE_SIZE)?;
        
        let start = Instant::now();
        let mut total_bytes = 0u64;
        let mut checksum = 0u64;
        
        for _ in 0..iterations {
            let mut file = File::open(&test_file)?;
            let mut buffer = vec![0u8; 8192];
            
            loop {
                let bytes_read = file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                
                for &byte in &buffer[..bytes_read] {
                    checksum ^= byte as u64;
                }
                
                total_bytes += bytes_read as u64;
            }
        }
        
        let elapsed = start.elapsed();
        let bytes_per_sec = (total_bytes as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("file_size".to_string(), TEST_FILE_SIZE.to_string());
        metadata.insert("total_bytes_read".to_string(), total_bytes.to_string());
        metadata.insert("checksum".to_string(), checksum.to_string());
        metadata.insert("iterations".to_string(), iterations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
            throughput: bytes_per_sec,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Sequential file write benchmark
pub struct SequentialFileWrite;

impl SequentialFileWrite {
    pub fn new() -> Self {
        Self
    }
    
    /// Generate test data
    fn generate_test_data(size: usize) -> Vec<u8> {
        let mut data = vec![0u8; size];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        data
    }
}

impl Benchmark for SequentialFileWrite {
    fn name(&self) -> &str {
        "Sequential File Write"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::FileSystem
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test_write.dat");
        
        let test_data = Self::generate_test_data(SMALL_FILE_SIZE);
        let start = Instant::now();
        let mut total_bytes = 0u64;
        
        for _ in 0..iterations {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&test_file)?;
                
            file.write_all(&test_data)?;
            file.sync_all()?;
            total_bytes += test_data.len() as u64;
        }
        
        let elapsed = start.elapsed();
        let bytes_per_sec = (total_bytes as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("write_size".to_string(), SMALL_FILE_SIZE.to_string());
        metadata.insert("total_bytes_written".to_string(), total_bytes.to_string());
        metadata.insert("iterations".to_string(), iterations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
            throughput: bytes_per_sec,
            unit: "bytes/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Random file access benchmark
pub struct RandomFileAccess;

impl RandomFileAccess {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for RandomFileAccess {
    fn name(&self) -> &str {
        "Random File Access"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::FileSystem
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let test_file = temp_dir.path().join("test_random.dat");
        
        // Create test file
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&test_file)?;
            
        let buffer = vec![0u8; 512];
        for _ in 0..(TEST_FILE_SIZE / buffer.len()) {
            file.write_all(&buffer)?;
        }
        file.sync_all()?;
        
        let start = Instant::now();
        let mut total_reads = 0u64;
        let mut buffer = vec![0u8; 512];
        
        for i in 0..iterations {
            // Generate random access pattern
            let offset = ((i * 4096) % (TEST_FILE_SIZE - buffer.len())) as u64;
            
            // Read from random position
            file.seek(SeekFrom::Start(offset))?;
            let bytes_read = file.read(&mut buffer)?;
            total_reads += bytes_read as u64;
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("file_size".to_string(), TEST_FILE_SIZE.to_string());
        metadata.insert("read_size".to_string(), "512".to_string());
        metadata.insert("total_reads".to_string(), total_reads.to_string());
        metadata.insert("iterations".to_string(), iterations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: (total_reads as f64) / elapsed.as_secs_f64(),
            unit: "random_accesses/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// File metadata operations benchmark
pub struct FileMetadata;

impl FileMetadata {
    pub fn new() -> Self {
        Self
    }
    
    /// Create test files for metadata testing
    fn create_test_files(dir: &Path, count: usize) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        
        for i in 0..count {
            let file_path = dir.join(format!("test_file_{}.dat", i));
            let mut file = File::create(&file_path)?;
            
            // Write some data to make file creation realistic
            let data = format!("Test file {}", i).into_bytes();
            file.write_all(&data)?;
            
            files.push(file_path);
        }
        
        Ok(files)
    }
}

impl Benchmark for FileMetadata {
    fn name(&self) -> &str {
        "File Metadata Operations"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::FileSystem
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        
        // Create test files
        let test_files = Self::create_test_files(temp_dir.path(), 100)?;
        
        let start = Instant::now();
        let mut metadata_ops = 0u64;
        
        for _ in 0..iterations {
            for file_path in &test_files {
                // Perform various metadata operations
                if let Ok(metadata) = file_path.metadata() {
                    let _ = metadata.len();
                    let _ = metadata.is_file();
                    metadata_ops += 1;
                }
                
                // Directory operations
                if let Ok(parent) = file_path.parent() {
                    if let Ok(entries) = std::fs::read_dir(parent) {
                        for entry in entries {
                            if let Ok(_entry) = entry {
                                metadata_ops += 1;
                            }
                        }
                    }
                }
            }
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (metadata_ops as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("test_files_created".to_string(), test_files.len().to_string());
        metadata.insert("metadata_operations".to_string(), metadata_ops.to_string());
        metadata.insert("iterations".to_string(), iterations.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec,
            unit: "metadata_operations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Small file operations benchmark (for database-like workloads)
pub struct SmallFileOperations;

impl SmallFileOperations {
    pub fn new() -> Self {
        Self
    }
}

impl Benchmark for SmallFileOperations {
    fn name(&self) -> &str {
        "Small File Operations"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::FileSystem
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let start = Instant::now();
        
        let mut file_operations = 0u64;
        
        for i in 0..iterations {
            let file_path = temp_dir.path().join(format!("small_{}.dat", i));
            
            // Write operation
            {
                let mut file = File::create(&file_path)?;
                let data = format!("Small file test data {}", i);
                file.write_all(data.as_bytes())?;
                file_operations += 1;
            }
            
            // Read operation
            {
                let mut file = File::open(&file_path)?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)?;
                file_operations += 1;
            }
            
            // Delete operation
            if i % 10 == 0 {
                std::fs::remove_file(&file_path)?;
                file_operations += 1;
            }
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = (file_operations as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("iterations".to_string(), iterations.to_string());
        metadata.insert("file_operations".to_string(), file_operations.to_string());
        metadata.insert("operations_per_iteration".to_string(), "3".to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: ops_per_sec,
            throughput: ops_per_sec * 3.0, // 3 operations per iteration on average
            unit: "file_operations/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Directory traversal benchmark
pub struct DirectoryTraversal;

impl DirectoryTraversal {
    pub fn new() -> Self {
        Self
    }
    
    /// Create directory structure for traversal testing
    fn create_directory_structure(base_path: &Path, depth: usize, files_per_dir: usize) -> Result<(), Box<dyn std::error::Error>> {
        if depth == 0 {
            return Ok(());
        }
        
        // Create files in current directory
        for i in 0..files_per_dir {
            let file_path = base_path.join(format!("file_{}.txt", i));
            File::create(&file_path)?.write_all(format!("File at depth {}", depth).as_bytes())?;
        }
        
        // Create subdirectories
        for i in 0..2 {
            let subdir = base_path.join(format!("subdir_{}", i));
            std::fs::create_dir(&subdir)?;
            Self::create_directory_structure(&subdir, depth - 1, files_per_dir)?;
        }
        
        Ok(())
    }
}

impl Benchmark for DirectoryTraversal {
    fn name(&self) -> &str {
        "Directory Traversal"
    }
    
    fn category(&self) -> BenchmarkCategory {
        BenchmarkCategory::FileSystem
    }
    
    fn run(&self, iterations: u64) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        
        // Create directory structure
        Self::create_directory_structure(temp_dir.path(), 3, 5)?;
        
        let start = Instant::now();
        let mut total_entries = 0u64;
        
        for _ in 0..iterations {
            for entry in WalkDir::new(temp_dir.path()) {
                if let Ok(entry) = entry {
                    total_entries += 1;
                    let _ = entry.file_type();
                }
            }
        }
        
        let elapsed = start.elapsed();
        let entries_per_sec = (total_entries as f64) / elapsed.as_secs_f64();
        
        let mut metadata = HashMap::new();
        metadata.insert("total_entries_traversed".to_string(), total_entries.to_string());
        metadata.insert("iterations".to_string(), iterations.to_string());
        
        // Estimate directory structure size
        let estimated_files = 5 * (2usize.pow(3) - 1); // 5 files per dir, 2^3 - 1 dirs
        metadata.insert("estimated_files".to_string(), estimated_files.to_string());
        
        Ok(BenchmarkResult {
            name: self.name().to_string(),
            category: self.category(),
            duration: elapsed,
            iterations,
            operations_per_second: (iterations as f64) / elapsed.as_secs_f64(),
            throughput: entries_per_sec,
            unit: "directory_entries/sec".to_string(),
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Complete file system benchmarking suite
pub struct FileSystemBenchmarkSuite {
    pub sequential_read: SequentialFileRead,
    pub sequential_write: SequentialFileWrite,
    pub random_access: RandomFileAccess,
    pub metadata: FileMetadata,
    pub small_files: SmallFileOperations,
    pub directory_traversal: DirectoryTraversal,
}

impl FileSystemBenchmarkSuite {
    pub fn new() -> Self {
        Self {
            sequential_read: SequentialFileRead::new(),
            sequential_write: SequentialFileWrite::new(),
            random_access: RandomFileAccess::new(),
            metadata: FileMetadata::new(),
            small_files: SmallFileOperations::new(),
            directory_traversal: DirectoryTraversal::new(),
        }
    }
    
    /// Run all file system benchmarks
    pub fn run_all(&self, iterations: u64) -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
        use super::BenchmarkRunner;
        
        let runner = BenchmarkRunner::new(false);
        let benchmarks: Vec<_> = vec![
            &self.sequential_read,
            &self.sequential_write,
            &self.random_access,
            &self.metadata,
            &self.small_files,
            &self.directory_traversal,
        ];
        
        runner.run_benchmarks(benchmarks.into_iter().cloned().collect(), iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sequential_file_read() {
        let bench = SequentialFileRead::new();
        let result = bench.run(2).unwrap();
        assert!(result.operations_per_second > 0.0);
        assert!(result.throughput > 0.0);
    }
    
    #[test]
    fn test_file_metadata() {
        let bench = FileMetadata::new();
        let result = bench.run(5).unwrap();
        assert!(result.operations_per_second > 0.0);
    }
}