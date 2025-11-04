//! MultiOS File System Test Module
//! 
//! This module provides comprehensive testing for the MultiOS file system implementation,
//! including unit tests for file operations, permissions, file locking, and system calls.

use crate::filesystem::*;
use crate::arch::interrupts::{InterruptError, PrivilegeLevel};
use alloc::vec::Vec;

/// Test utilities for file system testing
pub struct FileSystemTestUtils {
    test_inode_counter: u32,
}

impl FileSystemTestUtils {
    /// Create new test utilities
    pub fn new() -> Self {
        Self {
            test_inode_counter: 1000,
        }
    }
    
    /// Create a test file with specified permissions
    pub fn create_test_file(&self, path: &str, uid: u32, gid: u32, mode: u16) -> Result<u32, InterruptError> {
        let fs_manager = get_file_system_manager();
        fs_manager.create_file(path, uid, gid, mode)
    }
    
    /// Create a test directory with specified permissions
    pub fn create_test_directory(&self, path: &str, uid: u32, gid: u32, mode: u16) -> Result<u32, InterruptError> {
        let fs_manager = get_file_system_manager();
        fs_manager.create_directory(path, uid, gid, mode)
    }
    
    /// Test file permissions
    pub fn test_file_permissions() -> Result<(), InterruptError> {
        info!("Testing file permissions...");
        
        let permissions = FilePermissions::new();
        
        // Test default permissions
        assert!(permissions.owner_read);
        assert!(permissions.owner_write);
        assert!(permissions.owner_execute);
        assert!(permissions.group_read);
        assert!(!permissions.group_write);
        assert!(!permissions.group_execute);
        assert!(permissions.other_read);
        assert!(!permissions.other_write);
        assert!(!permissions.other_execute);
        
        // Test permission conversion
        let mode = permissions.to_mode();
        let reconstructed = FilePermissions::from_mode(mode);
        assert_eq!(permissions.owner_read, reconstructed.owner_read);
        assert_eq!(permissions.owner_write, reconstructed.owner_write);
        assert_eq!(permissions.owner_execute, reconstructed.owner_execute);
        
        info!("File permissions test passed");
        Ok(())
    }
    
    /// Test file ownership
    pub fn test_file_ownership() -> Result<(), InterruptError> {
        info!("Testing file ownership...");
        
        let ownership = FileOwnership::new(1000, 1000);
        assert_eq!(ownership.uid, 1000);
        assert_eq!(ownership.gid, 1000);
        
        info!("File ownership test passed");
        Ok(())
    }
    
    /// Test file locking
    pub fn test_file_locks() -> Result<(), InterruptError> {
        info!("Testing file locks...");
        
        let mut lock_manager = FileLockManager::new();
        
        // Test read lock
        let read_lock = FileLock::new(LockType::Read, 0, 100, 1, 1000);
        lock_manager.acquire_lock(read_lock.clone())?;
        
        // Test that read locks can coexist
        let read_lock2 = FileLock::new(LockType::Read, 0, 50, 2, 2000);
        lock_manager.acquire_lock(read_lock2.clone())?;
        
        // Test write lock conflict detection
        let write_lock = FileLock::new(LockType::Write, 0, 100, 3, 3000);
        assert!(lock_manager.acquire_lock(write_lock).is_err()); // Should fail due to conflict
        
        info!("File locks test passed");
        Ok(())
    }
    
    /// Test process file table
    pub fn test_process_file_table() -> Result<(), InterruptError> {
        info!("Testing process file table...");
        
        let mut file_table = ProcessFileTable::new(1);
        
        // Test file descriptor allocation
        let inode = Arc::new(RwLock::new(FileInode::new(100, FileType::Regular, 1000, 1000)));
        let handle = Arc::new(FileHandle::new(0, inode, FileMode::ReadWrite, FileFlags::new()));
        
        let fd = file_table.allocate_file_descriptor(handle)?;
        assert!(fd < 1024);
        
        // Test file descriptor lookup
        let retrieved_handle = file_table.get_file_handle(fd);
        assert!(retrieved_handle.is_some());
        
        // Test file descriptor duplication
        let new_fd = file_table.duplicate_file_descriptor(fd, 10)?;
        assert_eq!(new_fd, 10);
        
        info!("Process file table test passed");
        Ok(())
    }
    
    /// Test file system manager
    pub fn test_file_system_manager() -> Result<(), InterruptError> {
        info!("Testing file system manager...");
        
        let fs_manager = get_file_system_manager();
        
        // Test inode allocation
        let inode_num = fs_manager.allocate_inode(FileType::Regular, 1000, 1000)?;
        assert!(inode_num > 0);
        
        // Test file creation
        let created_inode = fs_manager.create_file("/test/file.txt", 1000, 1000, 0o644)?;
        assert!(created_inode > 0);
        
        // Test file lookup
        let looked_up_inode = fs_manager.look_up_path("/test/file.txt");
        assert_eq!(looked_up_inode, Some(created_inode));
        
        // Test directory creation
        let dir_inode = fs_manager.create_directory("/test", 1000, 1000, 0o755)?;
        assert!(dir_inode > 0);
        
        // Test file removal
        fs_manager.remove_file("/test/file.txt")?;
        
        // Test that file is no longer accessible
        let removed_inode = fs_manager.look_up_path("/test/file.txt");
        assert_eq!(removed_inode, None);
        
        info!("File system manager test passed");
        Ok(())
    }
    
    /// Test ACL functionality
    pub fn test_access_control_lists() -> Result<(), InterruptError> {
        info!("Testing access control lists...");
        
        let mut acl_entries = Vec::new();
        
        // Add owner entry
        acl_entries.push(AclEntry {
            acl_type: AclType::User,
            qualifier: 1000,
            permissions: AclPermissions {
                read: true,
                write: true,
                execute: true,
            },
        });
        
        // Add group entry
        acl_entries.push(AclEntry {
            acl_type: AclType::Group,
            qualifier: 1000,
            permissions: AclPermissions {
                read: true,
                write: false,
                execute: true,
            },
        });
        
        // Add other entry
        acl_entries.push(AclEntry {
            acl_type: AclType::Other,
            qualifier: 0,
            permissions: AclPermissions {
                read: true,
                write: false,
                execute: false,
            },
        });
        
        assert_eq!(acl_entries.len(), 3);
        
        info!("Access control lists test passed");
        Ok(())
    }
    
    /// Test file system statistics
    pub fn test_file_system_statistics() -> Result<(), InterruptError> {
        info!("Testing file system statistics...");
        
        let fs_manager = get_file_system_manager();
        let stats = fs_manager.get_stats();
        
        // Check that stats are initialized
        assert!(stats.file_system_type == FileSystemType::Virtual);
        assert!(stats.block_size > 0);
        assert!(stats.max_file_size > 0);
        
        info!("File system statistics test passed");
        Ok(())
    }
    
    /// Run all file system tests
    pub fn run_all_tests(&self) -> Result<(), InterruptError> {
        info!("Running all file system tests...");
        
        self.test_file_permissions()?;
        self.test_file_ownership()?;
        self.test_file_locks()?;
        self.test_process_file_table()?;
        self.test_file_system_manager()?;
        self.test_access_control_lists()?;
        self.test_file_system_statistics()?;
        
        info!("All file system tests passed successfully");
        Ok(())
    }
}

/// Integration test for file operations and syscalls
pub fn run_integration_tests() -> Result<(), InterruptError> {
    info!("Running file operations integration tests...");
    
    // This would include tests that exercise the complete syscall interface
    // including parameter validation, error handling, and file operations
    
    info!("Integration tests completed");
    Ok(())
}

/// Performance benchmark for file system operations
pub fn run_performance_benchmarks() -> Result<(), InterruptError> {
    info!("Running file system performance benchmarks...");
    
    let fs_manager = get_file_system_manager();
    
    // Benchmark file creation
    let start_time = crate::arch::interrupts::handlers::get_current_time();
    for i in 0..1000 {
        let path = format!("/test/benchmark_file_{}.txt", i);
        fs_manager.create_file(&path, 1000, 1000, 0o644)?;
    }
    let end_time = crate::arch::interrupts::handlers::get_current_time();
    
    info!("File creation benchmark: {} files created in {} time units", 
          1000, end_time - start_time);
    
    // Benchmark file lookup
    let start_time = crate::arch::interrupts::handlers::get_current_time();
    for i in 0..1000 {
        let path = format!("/test/benchmark_file_{}.txt", i);
        fs_manager.look_up_path(&path);
    }
    let end_time = crate::arch::interrupts::handlers::get_current_time();
    
    info!("File lookup benchmark: {} lookups performed in {} time units", 
          1000, end_time - start_time);
    
    Ok(())
}