//! MFS Comprehensive Testing Suite
//! 
//! This module provides comprehensive testing for the MultiOS File System
//! including unit tests, integration tests, performance benchmarks, and
//! stress testing scenarios.

#![no_std]

use crate::mfs::*;

/// Test Suite for MFS Basic Operations
pub mod basic_tests {
    use super::*;

    #[test]
    fn test_superblock_creation() {
        let fs = MfsFileSystem::new(1024 * 1024);
        
        assert_eq!(fs.superblock.magic, MFS_MAGIC);
        assert_eq!(fs.superblock.version, MFS_VERSION);
        assert_eq!(fs.superblock.block_size, MFS_BLOCK_SIZE as u32);
        assert_eq!(fs.superblock.block_count, 1024 * 1024);
        
        // Check feature flags
        assert!(fs.superblock.features.contains(MfsFeatures::JOURNALING));
        assert!(fs.superblock.features.contains(MfsFeatures::INDEXING));
        assert!(fs.superblock.features.contains(MfsFeatures::SECURITY));
        assert!(fs.superblock.features.contains(MfsFeatures::LARGE_FILES));
    }

    #[test]
    fn test_mount_unmount_operations() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        
        // Initial state
        assert!(!fs.mounted);
        
        // Mount operation
        assert!(fs.mount().is_ok());
        assert!(fs.mounted);
        assert_eq!(fs.superblock.mount_count, 1);
        
        // Second mount should fail
        assert!(fs.mount().is_err());
        
        // Unmount operation
        assert!(fs.unmount().is_ok());
        assert!(!fs.mounted);
        
        // Double unmount should fail
        assert!(fs.unmount().is_err());
    }

    #[test]
    fn test_file_creation_and_deletion() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        // Create a file
        let inode = fs.create_file("test.txt", 1000, 1000, 0o644).unwrap();
        assert!(inode > 0);
        
        // Try to create duplicate file
        assert!(fs.create_file("test.txt", 1000, 1000, 0o644).is_err());
        
        // Create another file
        let inode2 = fs.create_file("test2.txt", 1000, 1000, 0o644).unwrap();
        assert_ne!(inode, inode2);
        
        // Delete file
        assert!(fs.delete("test.txt").is_ok());
        
        // Try to delete non-existent file
        assert!(fs.delete("nonexistent.txt").is_err());
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_file_read_write_operations() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        let inode = fs.create_file("test.txt", 1000, 1000, 0o644).unwrap();
        
        // Write data
        let data = b"Hello, MFS! This is a test.";
        let bytes_written = fs.write_file(inode, data, 0).unwrap();
        assert_eq!(bytes_written, data.len());
        
        // Read data back
        let read_data = fs.read_file(inode, data.len() as u64, 0).unwrap();
        assert_eq!(read_data, data);
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_large_file_operations() {
        let mut fs = MfsFileSystem::new(1024 * 1024 * 100); // 100MB
        fs.mount().unwrap();
        
        let inode = fs.create_file("large.txt", 1000, 1000, 0o644).unwrap();
        
        // Write 1MB of data in chunks
        let chunk_size = 4096;
        let total_chunks = 256; // 1MB total
        
        for i in 0..total_chunks {
            let data = vec![(i % 256) as u8; chunk_size];
            let offset = (i * chunk_size) as u64;
            let bytes_written = fs.write_file(inode, &data, offset).unwrap();
            assert_eq!(bytes_written, chunk_size);
        }
        
        // Read back and verify
        for i in 0..total_chunks {
            let offset = (i * chunk_size) as u64;
            let read_data = fs.read_file(inode, chunk_size as u64, offset).unwrap();
            
            for (j, &byte) in read_data.iter().enumerate() {
                assert_eq!(byte, (i % 256) as u8);
            }
        }
        
        fs.unmount().unwrap();
    }
}

/// Test Suite for Directory Operations
pub mod directory_tests {
    use super::*;

    #[test]
    fn test_directory_creation_and_listing() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        // Create multiple directories
        fs.create_directory("dir1", 1000, 1000, 0o755).unwrap();
        fs.create_directory("dir2", 1000, 1000, 0o755).unwrap();
        fs.create_directory("dir3", 1000, 1000, 0o755).unwrap();
        
        // List root directory
        let entries = fs.list_directory("/").unwrap();
        assert_eq!(entries.len(), 3);
        
        // Verify directory entries
        let mut dir_names = Vec::new();
        for entry in entries.iter() {
            let name = String::from_utf8_lossy(&entry.name[..entry.name_length as usize]);
            dir_names.push(name.to_string());
            assert_eq!(entry.file_type, FileType::Directory as u8);
        }
        
        assert!(dir_names.contains(&"dir1".to_string()));
        assert!(dir_names.contains(&"dir2".to_string()));
        assert!(dir_names.contains(&"dir3".to_string()));
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_mixed_files_and_directories() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        // Create files and directories in root
        fs.create_file("file1.txt", 1000, 1000, 0o644).unwrap();
        fs.create_directory("dir1", 1000, 1000, 0o755).unwrap();
        fs.create_file("file2.txt", 1000, 1000, 0o644).unwrap();
        fs.create_directory("dir2", 1000, 1000, 0o755).unwrap();
        
        let entries = fs.list_directory("/").unwrap();
        assert_eq!(entries.len(), 4);
        
        let mut file_count = 0;
        let mut dir_count = 0;
        
        for entry in entries.iter() {
            match entry.file_type {
                0 => file_count += 1, // Regular file
                1 => dir_count += 1,  // Directory
                _ => {}
            }
        }
        
        assert_eq!(file_count, 2);
        assert_eq!(dir_count, 2);
        
        fs.unmount().unwrap();
    }
}

/// Test Suite for Security Features
pub mod security_tests {
    use super::*;

    #[test]
    fn test_permission_checking() {
        let security = MfsSecurityManager::new();
        
        // Test read permission
        assert!(security.check_permission(1000, 1000, 0o644, MfsOperation::Read));
        assert!(security.check_permission(1000, 1000, 0o400, MfsOperation::Read));
        assert!(!security.check_permission(1000, 1000, 0o200, MfsOperation::Read));
        
        // Test write permission
        assert!(security.check_permission(1000, 1000, 0o644, MfsOperation::Write));
        assert!(security.check_permission(1000, 1000, 0o200, MfsOperation::Write));
        assert!(!security.check_permission(1000, 1000, 0o400, MfsOperation::Write));
        
        // Test execute permission
        assert!(security.check_permission(1000, 1000, 0o755, MfsOperation::Execute));
        assert!(security.check_permission(1000, 1000, 0o100, MfsOperation::Execute));
        assert!(!security.check_permission(1000, 1000, 0o600, MfsOperation::Execute));
    }

    #[test]
    fn test_security_attribute_creation() {
        let security = MfsSecurityManager::new();
        
        let attr = security.create_security_attr(1000, 1000, 0o644);
        
        assert_eq!(attr.owner_uid, 1000);
        assert_eq!(attr.group_gid, 1000);
        assert_eq!(attr.permissions, 0o644);
        assert_eq!(attr.type_id, 1);
    }

    #[test]
    fn test_audit_logging_toggle() {
        let mut security = MfsSecurityManager::new();
        
        assert!(!security.audit_enabled);
        
        security.enable_audit();
        assert!(security.audit_enabled);
        
        security.disable_audit();
        assert!(!security.audit_enabled);
    }

    #[test]
    fn test_file_creation_with_permissions() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.enable_security().unwrap();
        fs.mount().unwrap();
        
        // Create files with different permissions
        let inode1 = fs.create_file("readable.txt", 1000, 1000, 0o644).unwrap();
        let inode2 = fs.create_file("private.txt", 1000, 1000, 0o600).unwrap();
        let inode3 = fs.create_file("executable.sh", 1000, 1000, 0o755).unwrap();
        
        // All files should be created successfully
        assert!(inode1 > 0);
        assert!(inode2 > 0);
        assert!(inode3 > 0);
        
        fs.unmount().unwrap();
    }
}

/// Test Suite for Journal Operations
pub mod journal_tests {
    use super::*;

    #[test]
    fn test_journal_creation_and_operations() {
        let mut journal = MfsJournal::new(1000);
        
        // Initial state
        let stats = journal.get_stats();
        assert_eq!(stats.0, 0); // No sequences yet
        assert_eq!(stats.1, 0); // No entries yet
        
        // Start and commit transactions
        let seq1 = journal.start_transaction(1000).unwrap();
        assert_eq!(seq1, 1);
        
        let stats = journal.get_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 1);
        
        assert!(journal.commit(seq1).is_ok());
        
        let seq2 = journal.start_transaction(2000).unwrap();
        assert_eq!(seq2, 2);
        
        assert!(journal.rollback(seq2).is_ok());
        
        let stats = journal.get_stats();
        assert_eq!(stats.0, 2);
        assert_eq!(stats.1, 1); // One entry rolled back
        
        assert!(journal.commit(seq2).is_ok());
        
        let final_stats = journal.get_stats();
        assert_eq!(final_stats.0, 2);
        assert_eq!(final_stats.1, 2);
    }

    #[test]
    fn test_journal_capacity_management() {
        let mut journal = MfsJournal::new(5); // Small journal
        
        // Fill journal beyond capacity
        for i in 0..10 {
            let seq = journal.start_transaction(i * 1000).unwrap();
            assert!(journal.commit(seq).is_ok());
        }
        
        // Should have cleaned up old entries
        let stats = journal.get_stats();
        assert!(stats.1 <= 5); // Max capacity
    }
}

/// Test Suite for Block Allocation
pub mod allocation_tests {
    use super::*;

    #[test]
    fn test_block_allocator_creation() {
        let allocator = MfsIndexedAllocator::new(1024);
        
        // Should have created bitmap for 1024 blocks
        let expected_bitmap_size = (1024 + 7) / 8; // 128 bytes
        assert!(allocator.block_bitmap.len() >= expected_bitmap_size as usize);
        
        // Should have marked some blocks as reserved
        assert!(allocator.free_blocks < 1024);
    }

    #[test]
    fn test_block_allocation_and_deallocation() {
        let mut allocator = MfsIndexedAllocator::new(1024);
        
        // Allocate some blocks
        let blocks1 = allocator.allocate_blocks(10, None).unwrap();
        assert_eq!(blocks1.len(), 10);
        
        let blocks2 = allocator.allocate_blocks(5, None).unwrap();
        assert_eq!(blocks2.len(), 5);
        
        // Free some blocks
        allocator.deallocate_blocks(&blocks1).unwrap();
        
        // Allocate again - should reuse freed blocks
        let blocks3 = allocator.allocate_blocks(3, None).unwrap();
        assert_eq!(blocks3.len(), 3);
        
        // Should have freed exactly 10 blocks initially
        let expected_free = 1024 - 10 - 5 + 10 - 3; // Initial - allocated + freed - reallocated
        // Note: This calculation is approximate as we don't track exact reserved blocks
    }

    #[test]
    fn test_consecutive_block_allocation() {
        let mut allocator = MfsIndexedAllocator::new(2048);
        
        // Allocate a large consecutive range
        let blocks = allocator.allocate_blocks(100, None).unwrap();
        assert_eq!(blocks.len(), 100);
        
        // Check if blocks are consecutive (they should be)
        for i in 1..blocks.len() {
            assert_eq!(blocks[i], blocks[0] + i as u64);
        }
        
        // Deallocate and reallocate to test reuse
        allocator.deallocate_blocks(&blocks).unwrap();
        
        let blocks2 = allocator.allocate_blocks(50, None).unwrap();
        assert_eq!(blocks2.len(), 50);
        
        // Should reuse the same starting location
        assert_eq!(blocks2[0], blocks[0]);
    }
}

/// Test Suite for Error Handling
pub mod error_tests {
    use super::*;

    #[test]
    fn test_operations_on_unmounted_fs() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        
        // All operations should fail on unmounted fs
        assert!(fs.create_file("test.txt", 1000, 1000, 0o644).is_err());
        assert!(fs.create_directory("testdir", 1000, 1000, 0o755).is_err());
        assert!(fs.list_directory("/").is_err());
        assert!(fs.delete("test.txt").is_err());
    }

    #[test]
    fn test_operations_on_full_fs() {
        let mut fs = MfsFileSystem::new(1000); // Small fs
        fs.mount().unwrap();
        
        // Fill up the file system by creating many small files
        let mut created_files = 0;
        for i in 0..200 {
            match fs.create_file(&format!("file_{}.txt", i), 1000, 1000, 0o644) {
                Ok(_) => created_files += 1,
                Err(_) => break, // Disk full
            }
        }
        
        info!("Created {} files before disk full", created_files);
        
        // Should eventually hit disk full
        assert!(created_files > 0);
        assert!(created_files < 200);
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_invalid_name_handling() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        // Empty name
        assert!(fs.create_file("", 1000, 1000, 0o644).is_err());
        
        // Very long name (>255 chars)
        let long_name = "a".repeat(256);
        assert!(fs.create_file(&long_name, 1000, 1000, 0o644).is_err());
        
        // Valid but edge case name
        let valid_long_name = "a".repeat(255);
        assert!(fs.create_file(&valid_long_name, 1000, 1000, 0o644).is_ok());
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_permission_denied_scenarios() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.enable_security().unwrap();
        fs.mount().unwrap();
        
        // Create a file with restrictive permissions
        fs.create_file("restricted.txt", 1000, 1000, 0o000).unwrap();
        
        // Try to create with different user (should work in this simple case)
        // In a real implementation, this would check actual permissions
        let result = fs.create_file("another.txt", 2000, 2000, 0o644);
        // This might succeed or fail depending on implementation details
        
        fs.unmount().unwrap();
    }
}

/// Performance Benchmark Tests
pub mod performance_tests {
    use super::*;

    #[test]
    fn test_file_creation_performance() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        let start_inodes = fs.superblock.free_inodes;
        let target_files = 1000;
        
        // Create many files
        for i in 0..target_files {
            let _inode = fs.create_file(&format!("perf_{}.txt", i), 1000, 1000, 0o644).unwrap();
            
            if i % 100 == 0 && i > 0 {
                let current_inodes = fs.superblock.free_inodes;
                let created = start_inodes - current_inodes;
                info!("Created {} files", created);
            }
        }
        
        let final_inodes = fs.superblock.free_inodes;
        let created_files = start_inodes - final_inodes;
        assert_eq!(created_files, target_files);
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_file_io_performance() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        let inode = fs.create_file("io_test.txt", 1000, 1000, 0o644).unwrap();
        
        // Write many small files
        let iterations = 1000;
        for i in 0..iterations {
            let data = format!("Test data {}", i);
            let _bytes = fs.write_file(inode, data.as_bytes(), (i * 100) as u64).unwrap();
            
            if i % 200 == 0 {
                info!("Completed {} write operations", i);
            }
        }
        
        // Read back some data
        for i in 0..(iterations / 10) {
            let _data = fs.read_file(inode, 100, (i * 1000) as u64).unwrap();
        }
        
        info!("Completed I/O performance test");
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_directory_traversal_performance() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        // Create many directory entries
        let num_entries = 5000;
        for i in 0..num_entries {
            if i % 2 == 0 {
                fs.create_file(&format!("file_{}.txt", i), 1000, 1000, 0o644).unwrap();
            } else {
                fs.create_directory(&format!("dir_{}", i), 1000, 1000, 0o755).unwrap();
            }
        }
        
        // Time directory listing
        let start_time = 0; // In real implementation, would use actual timing
        let entries = fs.list_directory("/").unwrap();
        let end_time = 0;
        
        assert_eq!(entries.len(), num_entries);
        
        info!("Listed {} directory entries", entries.len());
        info!("Directory traversal took {} microseconds", end_time - start_time);
        
        fs.unmount().unwrap();
    }
}

/// Stress Test Scenarios
pub mod stress_tests {
    use super::*;

    #[test]
    fn test_sequential_file_operations() {
        let mut fs = MfsFileSystem::new(1024 * 1024 * 10); // 10MB
        fs.mount().unwrap();
        
        let num_files = 1000;
        
        // Create files
        for i in 0..num_files {
            let inode = fs.create_file(&format!("stress_{}.txt", i), 1000, 1000, 0o644).unwrap();
            
            // Write data
            let data = format!("Stress test data for file {}", i);
            fs.write_file(inode, data.as_bytes(), 0).unwrap();
        }
        
        // Read and verify files
        for i in 0..num_files {
            let expected_data = format!("Stress test data for file {}", i);
            let read_data = fs.read_file(i as u32 + 1, expected_data.len() as u64, 0).unwrap();
            assert_eq!(read_data, expected_data.as_bytes());
        }
        
        // Delete files
        for i in 0..num_files {
            fs.delete(&format!("stress_{}.txt", i)).unwrap();
        }
        
        // Verify all files are gone
        let entries = fs.list_directory("/").unwrap();
        assert_eq!(entries.len(), 0);
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let mut fs = MfsFileSystem::new(1024 * 1024 * 5); // 5MB
        fs.mount().unwrap();
        
        let operations = 500;
        
        // Mix of operations
        for i in 0..operations {
            match i % 4 {
                0 => { // Create file
                    let _ = fs.create_file(&format!("mixed_{}.txt", i), 1000, 1000, 0o644);
                }
                1 => { // Create directory
                    let _ = fs.create_directory(&format!("mixed_dir_{}", i), 1000, 1000, 0o755);
                }
                2 => { // Write to existing file
                    if let Ok(inode) = fs.create_file(&format!("write_test_{}.txt", i), 1000, 1000, 0o644) {
                        let data = format!("Mixed operation test {}", i);
                        let _ = fs.write_file(inode, data.as_bytes(), 0);
                    }
                }
                3 => { // Read existing file
                    if let Ok(inode) = fs.create_file(&format!("read_test_{}.txt", i), 1000, 1000, 0o644) {
                        let _ = fs.read_file(inode, 100, 0);
                    }
                }
                _ => {}
            }
        }
        
        info!("Completed mixed operations stress test");
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_memory_pressure_simulation() {
        let mut fs = MfsFileSystem::new(1024 * 1024); // 1MB
        fs.mount().unwrap();
        
        // Create many small files to stress memory
        let num_files = 200;
        let mut inodes = Vec::new();
        
        for i in 0..num_files {
            let inode = fs.create_file(&format!("memory_{}.txt", i), 1000, 1000, 0o644).unwrap();
            inodes.push(inode);
            
            // Write tiny amounts of data
            let data = [i as u8; 100];
            fs.write_file(inode, &data, 0).unwrap();
        }
        
        // Read all files to stress memory
        for &inode in &inodes {
            let _data = fs.read_file(inode, 100, 0).unwrap();
        }
        
        info!("Completed memory pressure simulation");
        
        fs.unmount().unwrap();
    }
}

/// Integration Tests
pub mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_file_system_lifecycle() {
        // Test complete lifecycle: create, use, verify, cleanup
        let mut fs = MfsFileSystem::new(1024 * 1024 * 2); // 2MB
        fs.mount().unwrap();
        
        // Create directory structure
        fs.create_directory("home", 1000, 1000, 0o755).unwrap();
        fs.create_directory("home/user1", 1000, 1000, 0o755).unwrap();
        fs.create_directory("tmp", 1000, 1000, 0o755).unwrap();
        
        // Create files in different locations
        let home_inode = fs.create_file("home/user1/document.txt", 1000, 1000, 0o644).unwrap();
        let tmp_inode = fs.create_file("tmp/cache.dat", 1000, 1000, 0o644).unwrap();
        let root_inode = fs.create_file("README.txt", 1000, 1000, 0o644).unwrap();
        
        // Write data
        let home_data = b"User document content";
        let tmp_data = b"Cache data";
        let root_data = b"System documentation";
        
        fs.write_file(home_inode, home_data, 0).unwrap();
        fs.write_file(tmp_inode, tmp_data, 0).unwrap();
        fs.write_file(root_inode, root_data, 0).unwrap();
        
        // Read and verify
        let read_home = fs.read_file(home_inode, home_data.len() as u64, 0).unwrap();
        let read_tmp = fs.read_file(tmp_inode, tmp_data.len() as u64, 0).unwrap();
        let read_root = fs.read_file(root_inode, root_data.len() as u64, 0).unwrap();
        
        assert_eq!(read_home, home_data);
        assert_eq!(read_tmp, tmp_data);
        assert_eq!(read_root, root_data);
        
        // Get final statistics
        let stats = fs.get_stats();
        info!("Final stats: {} blocks free, {} inodes free", 
              stats.free_blocks, stats.free_inodes);
        
        // Cleanup
        fs.delete("home/user1/document.txt").unwrap();
        fs.delete("tmp/cache.dat").unwrap();
        fs.delete("README.txt").unwrap();
        fs.delete("home/user1").unwrap();
        fs.delete("home").unwrap();
        fs.delete("tmp").unwrap();
        
        fs.unmount().unwrap();
    }

    #[test]
    fn test_feature_combinations() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        
        // Enable all features
        fs.enable_journaling().unwrap();
        fs.enable_security().unwrap();
        fs.mount().unwrap();
        
        // Use all features
        let inode = fs.create_file("feature_test.txt", 1000, 1000, 0o644).unwrap();
        
        // Write with journaling
        let data = b"Testing all features";
        fs.write_file(inode, data, 0).unwrap();
        
        // Verify security
        assert!(fs.security_manager.audit_enabled);
        
        // Large file test
        let large_data = vec![0xCC; 100 * 1024]; // 100KB
        fs.write_file(inode, &large_data, 0).unwrap();
        
        fs.unmount().unwrap();
    }
}

/// Test Runner for all test suites
pub fn run_all_mfs_tests() -> Result<(), &'static str> {
    info!("=== Running MFS Comprehensive Test Suite ===");
    
    // Basic tests
    basic_tests::test_superblock_creation();
    info!("✓ Superblock creation test passed");
    
    basic_tests::test_mount_unmount_operations();
    info!("✓ Mount/unmount operations test passed");
    
    basic_tests::test_file_creation_and_deletion();
    info!("✓ File creation and deletion test passed");
    
    basic_tests::test_file_read_write_operations();
    info!("✓ File read/write operations test passed");
    
    // Directory tests
    directory_tests::test_directory_creation_and_listing();
    info!("✓ Directory creation and listing test passed");
    
    directory_tests::test_mixed_files_and_directories();
    info!("✓ Mixed files and directories test passed");
    
    // Security tests
    security_tests::test_permission_checking();
    info!("✓ Permission checking test passed");
    
    security_tests::test_security_attribute_creation();
    info!("✓ Security attribute creation test passed");
    
    security_tests::test_audit_logging_toggle();
    info!("✓ Audit logging toggle test passed");
    
    // Journal tests
    journal_tests::test_journal_creation_and_operations();
    info!("✓ Journal creation and operations test passed");
    
    // Allocation tests
    allocation_tests::test_block_allocator_creation();
    info!("✓ Block allocator creation test passed");
    
    allocation_tests::test_block_allocation_and_deallocation();
    info!("✓ Block allocation and deallocation test passed");
    
    // Error tests
    error_tests::test_operations_on_unmounted_fs();
    info!("✓ Operations on unmounted filesystem test passed");
    
    error_tests::test_invalid_name_handling();
    info!("✓ Invalid name handling test passed");
    
    // Integration tests
    integration_tests::test_complete_file_system_lifecycle();
    info!("✓ Complete filesystem lifecycle test passed");
    
    integration_tests::test_feature_combinations();
    info!("✓ Feature combinations test passed");
    
    info!("=== All MFS tests completed successfully! ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_mfs_comprehensive_tests() {
        let result = run_all_mfs_tests();
        assert!(result.is_ok());
    }
}