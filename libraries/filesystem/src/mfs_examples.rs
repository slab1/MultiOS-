//! MFS Usage Examples
//! 
//! This module contains comprehensive examples showing how to use
//! the MultiOS File System (MFS) in various scenarios.

#![no_std]

use crate::mfs::*;

/// Example: Basic file system operations
pub fn example_basic_operations() -> FsResult<()> {
    info!("=== MFS Basic Operations Example ===");
    
    // Create a 1GB file system
    let mut fs = MfsFileSystem::new(1024 * 1024);
    
    // Mount the file system
    fs.mount()?;
    info!("File system mounted successfully");
    
    // Create a test file
    let inode = fs.create_file("test.txt", 1000, 1000, 0o644)?;
    info!("Created file with inode: {}", inode);
    
    // Write some data
    let data = b"Hello, MultiOS File System! This is a test.";
    let bytes_written = fs.write_file(inode, data, 0)?;
    info!("Written {} bytes to file", bytes_written);
    
    // Read the data back
    let read_data = fs.read_file(inode, bytes_written, 0)?;
    assert_eq!(&read_data, data);
    info!("Data verified successfully");
    
    // Create a directory
    fs.create_directory("testdir", 1000, 1000, 0o755)?;
    info!("Created directory testdir");
    
    // List directory contents
    let entries = fs.list_directory("/")?;
    for entry in entries.iter() {
        let name = String::from_utf8_lossy(&entry.name[..entry.name_length as usize]);
        info!("  - {} (inode: {}, type: {})", name, entry.inode, entry.file_type);
    }
    
    // Get file system statistics
    let stats = fs.get_stats();
    info!("File system stats:");
    info!("  Total blocks: {}", stats.total_blocks);
    info!("  Free blocks: {}", stats.free_blocks);
    info!("  Total inodes: {}", stats.total_inodes);
    info!("  Free inodes: {}", stats.free_inodes);
    info!("  Journal entries: {}", stats.journal_entries);
    
    // Unmount the file system
    fs.unmount()?;
    info!("File system unmounted successfully");
    
    Ok(())
}

/// Example: Large file operations
pub fn example_large_file_operations() -> FsResult<()> {
    info!("=== MFS Large File Operations Example ===");
    
    // Create a larger file system for big files
    let mut fs = MfsFileSystem::new(1024 * 1024 * 1024); // 1GB
    fs.mount()?;
    
    // Create a large file (simulate 1MB file)
    let inode = fs.create_file("largefile.bin", 1000, 1000, 0o644)?;
    info!("Created large file with inode: {}", inode);
    
    // Write data in chunks to simulate large file operations
    let chunk_size = 4096; // Block size
    let total_chunks = 256; // 1MB total
    let test_data = vec![0xAB; chunk_size];
    
    for chunk in 0..total_chunks {
        let offset = (chunk * chunk_size) as u64;
        let bytes_written = fs.write_file(inode, &test_data, offset)?;
        
        if chunk % 64 == 0 {
            info!("Written {} MB of data", (chunk * chunk_size) / (1024 * 1024));
        }
    }
    
    // Verify the data
    for chunk in 0..total_chunks {
        let offset = (chunk * chunk_size) as u64;
        let read_data = fs.read_file(inode, chunk_size, offset)?;
        assert_eq!(&read_data, &test_data);
    }
    
    info!("Large file operations completed successfully");
    
    fs.unmount()?;
    Ok(())
}

/// Example: Security operations
pub fn example_security_operations() -> FsResult<()> {
    info!("=== MFS Security Operations Example ===");
    
    let mut fs = MfsFileSystem::new(1024 * 1024);
    fs.enable_security()?;
    fs.mount()?;
    
    // Create files with different permissions
    let inode1 = fs.create_file("public.txt", 1000, 1000, 0o644)?;
    let inode2 = fs.create_file("private.txt", 1000, 1000, 0o600)?;
    let inode3 = fs.create_file("executable.sh", 1000, 1000, 0o755)?;
    
    info!("Created files with different permissions");
    
    // Enable audit logging
    fs.security_manager.enable_audit();
    info!("Audit logging enabled");
    
    // Demonstrate permission checking
    let permission_test = fs.security_manager.check_permission(
        1000, 1000, 0o644, MfsOperation::Read
    );
    info!("Permission check (read): {}", permission_test);
    
    let permission_test = fs.security_manager.check_permission(
        1000, 1000, 0o600, MfsOperation::Write
    );
    info!("Permission check (write): {}", permission_test);
    
    // Create security attributes
    let security_attr = fs.security_manager.create_security_attr(1000, 1000, 0o644);
    info!("Security attributes created:");
    info!("  Owner UID: {}", security_attr.owner_uid);
    info!("  Group GID: {}", security_attr.group_gid);
    info!("  Permissions: 0{:o}", security_attr.permissions);
    
    fs.unmount()?;
    Ok(())
}

/// Example: Journal operations
pub fn example_journal_operations() -> FsResult<()> {
    info!("=== MFS Journal Operations Example ===");
    
    let mut fs = MfsFileSystem::new(1024 * 1024);
    fs.enable_journaling()?;
    fs.mount()?;
    
    // Create multiple files to test journal behavior
    let mut inodes = Vec::new();
    
    for i in 0..10 {
        let inode = fs.create_file(&format!("journal_test_{}.txt", i), 1000, 1000, 0o644)?;
        inodes.push(inode);
        
        // Write data with journal transactions
        let data = format!("Journal test data for file {}", i);
        fs.write_file(inode, data.as_bytes(), 0)?;
        
        if i % 3 == 0 {
            let stats = fs.journal.get_stats();
            info!("Journal stats after {} files: {} entries, max {}", 
                  i + 1, stats.1, stats.2);
        }
    }
    
    info!("Created 10 files with journal transactions");
    
    // Test journal commit and rollback simulation
    let sequence = fs.journal.start_transaction(1000)?;
    info!("Started journal transaction: {}", sequence);
    
    // Simulate some operations
    fs.journal.commit(sequence)?;
    info!("Committed journal transaction: {}", sequence);
    
    // Test rollback
    let rollback_seq = fs.journal.start_transaction(2000)?;
    info!("Started rollback transaction: {}", rollback_seq);
    fs.journal.rollback(rollback_seq)?;
    info!("Rolled back transaction: {}", rollback_seq);
    
    // Final journal statistics
    let final_stats = fs.journal.get_stats();
    info!("Final journal stats: {} entries", final_stats.1);
    
    fs.unmount()?;
    Ok(())
}

/// Example: Directory operations
pub fn example_directory_operations() -> FsResult<()> {
    info!("=== MFS Directory Operations Example ===");
    
    let mut fs = MfsFileSystem::new(1024 * 1024);
    fs.mount()?;
    
    // Create directory structure
    fs.create_directory("documents", 1000, 1000, 0o755)?;
    fs.create_directory("images", 1000, 1000, 0o755)?;
    fs.create_directory("scripts", 1000, 1000, 0o755)?;
    
    info!("Created directory structure");
    
    // Create files in different directories
    let inode1 = fs.create_file("document1.txt", 1000, 1000, 0o644)?;
    let inode2 = fs.create_file("image1.jpg", 1000, 1000, 0o644)?;
    let inode3 = fs.create_file("script.sh", 1000, 1000, 0o755)?;
    
    info!("Created files in root directory");
    
    // List root directory
    let root_entries = fs.list_directory("/")?;
    info!("Root directory contents:");
    for entry in root_entries.iter() {
        let name = String::from_utf8_lossy(&entry.name[..entry.name_length as usize]);
        let file_type = match entry.file_type {
            0 => "Regular File",
            1 => "Directory",
            _ => "Other",
        };
        info!("  {} - {} (inode: {})", name, file_type, entry.inode);
    }
    
    // Delete a file
    fs.delete("script.sh")?;
    info!("Deleted script.sh");
    
    // List directory again to verify deletion
    let root_entries_after = fs.list_directory("/")?;
    info!("Root directory after deletion:");
    for entry in root_entries_after.iter() {
        let name = String::from_utf8_lossy(&entry.name[..entry.name_length as usize]);
        info!("  {}", name);
    }
    
    fs.unmount()?;
    Ok(())
}

/// Example: File system statistics and monitoring
pub fn example_monitoring() -> FsResult<()> {
    info!("=== MFS Monitoring Example ===");
    
    let mut fs = MfsFileSystem::new(1024 * 1024);
    fs.mount()?;
    
    // Initial statistics
    let initial_stats = fs.get_stats();
    info!("Initial file system state:");
    info!("  Total blocks: {}", initial_stats.total_blocks);
    info!("  Free blocks: {}", initial_stats.free_blocks);
    info!("  Usage: {:.2}%", 
          100.0 * (initial_stats.total_blocks - initial_stats.free_blocks) as f64 / initial_stats.total_blocks as f64);
    
    // Create files and monitor usage
    let files_to_create = 100;
    let mut created_inodes = Vec::new();
    
    info!("Creating {} files...", files_to_create);
    for i in 0..files_to_create {
        let inode = fs.create_file(&format!("monitor_test_{}.txt", i), 1000, 1000, 0o644)?;
        created_inodes.push(inode);
        
        // Write small data to each file
        let data = format!("Test data for file {}", i);
        fs.write_file(inode, data.as_bytes(), 0)?;
        
        if i % 20 == 0 {
            let current_stats = fs.get_stats();
            let used_blocks = initial_stats.free_blocks - current_stats.free_blocks;
            info!("  After {} files: {} blocks used", i, used_blocks);
        }
    }
    
    // Final statistics
    let final_stats = fs.get_stats();
    let used_blocks = initial_stats.free_blocks - final_stats.free_blocks;
    let usage_percent = 100.0 * used_blocks as f64 / initial_stats.total_blocks as f64;
    
    info!("Final file system state:");
    info!("  Files created: {}", files_to_create);
    info!("  Blocks used: {}", used_blocks);
    info!("  Usage: {:.2}%", usage_percent);
    info!("  Mount count: {}", final_stats.mount_count);
    info!("  Journal entries: {}", final_stats.journal_entries);
    
    // Cleanup
    info!("Cleaning up test files...");
    for i in 0..files_to_create {
        fs.delete(&format!("monitor_test_{}.txt", i))?;
    }
    
    fs.unmount()?;
    Ok(())
}

/// Example: Error handling scenarios
pub fn example_error_handling() -> FsResult<()> {
    info!("=== MFS Error Handling Example ===");
    
    let mut fs = MfsFileSystem::new(1024 * 1024);
    fs.mount()?;
    
    // Test 1: Creating duplicate file
    info!("Test 1: Creating duplicate file");
    fs.create_file("duplicate.txt", 1000, 1000, 0o644)?;
    let result = fs.create_file("duplicate.txt", 1000, 1000, 0o644);
    assert!(result.is_err()); // Should fail
    info!("  ✓ Duplicate file creation properly rejected");
    
    // Test 2: Creating file with invalid name
    info!("Test 2: Creating file with invalid name");
    let invalid_name = "a".repeat(256); // Too long
    let result = fs.create_file(&invalid_name, 1000, 1000, 0o644);
    assert!(result.is_err()); // Should fail
    info!("  ✓ Invalid file name properly rejected");
    
    // Test 3: Operations on unmounted file system
    info!("Test 3: Operations on unmounted file system");
    fs.unmount()?;
    let result = fs.create_file("test.txt", 1000, 1000, 0o644);
    assert!(result.is_err()); // Should fail
    info!("  ✓ Operations on unmounted file system properly rejected");
    
    // Test 4: Double mount
    info!("Test 4: Double mount");
    let result = fs.mount();
    assert!(result.is_err()); // Should fail
    info!("  ✓ Double mount properly rejected");
    
    info!("Error handling tests completed successfully");
    Ok(())
}

/// Example: Performance testing
pub fn example_performance_testing() -> FsResult<()> {
    info!("=== MFS Performance Testing Example ===");
    
    let mut fs = MfsFileSystem::new(1024 * 1024 * 1024); // 1GB
    fs.mount()?;
    
    // Test 1: File creation performance
    info!("Test 1: File creation performance");
    let start_time = 0; // In real implementation, would use actual timing
    let num_files = 1000;
    
    let mut inodes = Vec::new();
    for i in 0..num_files {
        let inode = fs.create_file(&format!("perf_test_{}.txt", i), 1000, 1000, 0o644)?;
        inodes.push(inode);
    }
    
    info!("  Created {} files in performance test", num_files);
    
    // Test 2: Write performance
    info!("Test 2: Write performance");
    let test_data = vec![0x55; 4096]; // One block of data
    
    for &inode in &inodes[0..100] {
        fs.write_file(inode, &test_data, 0)?;
    }
    
    info!("  Wrote data to 100 files");
    
    // Test 3: Read performance
    info!("Test 3: Read performance");
    for &inode in &inodes[0..100] {
        let _read_data = fs.read_file(inode, 4096, 0)?;
    }
    
    info!("  Read data from 100 files");
    
    // Test 4: Directory listing performance
    info!("Test 4: Directory listing performance");
    let start_time = 0; // Would measure actual time
    let entries = fs.list_directory("/")?;
    let end_time = 0;
    
    info!("  Listed {} directory entries", entries.len());
    info!("  Directory listing took {} microseconds", end_time - start_time);
    
    // Cleanup
    info!("Cleaning up performance test files...");
    for i in 0..num_files {
        fs.delete(&format!("perf_test_{}.txt", i))?;
    }
    
    fs.unmount()?;
    Ok(())
}

#[cfg(test)]
mod examples_tests {
    use super::*;

    #[test]
    fn test_basic_operations_example() {
        let result = example_basic_operations();
        assert!(result.is_ok());
    }

    #[test]
    fn test_security_operations_example() {
        let result = example_security_operations();
        assert!(result.is_ok());
    }

    #[test]
    fn test_journal_operations_example() {
        let result = example_journal_operations();
        assert!(result.is_ok());
    }

    #[test]
    fn test_directory_operations_example() {
        let result = example_directory_operations();
        assert!(result.is_ok());
    }

    #[test]
    fn test_monitoring_example() {
        let result = example_monitoring();
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_handling_example() {
        let result = example_error_handling();
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_file_operations_example() {
        let result = example_large_file_operations();
        assert!(result.is_ok());
    }
}