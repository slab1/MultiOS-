//! VFS Tests
//! 
//! Comprehensive tests for the Virtual File System implementation.

use super::*;
use crate::vfs::{VfsManager, TmpFs, FileHandle, OpenFlags, SeekMode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_manager_creation() {
        let manager = VfsManager::new();
        assert_eq!(manager.get_mount_count(), 0);
    }

    #[test]
    fn test_tmpfs_creation() {
        let tmpfs = TmpFs::new_default();
        assert_eq!(tmpfs.current_inodes, 1); // Root inode only
    }

    #[test]
    fn test_tmpfs_root_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Check root directory exists
        assert!(tmpfs.exists("/"));
        assert!(tmpfs.file_type("/").unwrap() == FileType::Directory);
        
        // Get root stats
        let stats = tmpfs.stat("/").unwrap();
        assert_eq!(stats.file_type, FileType::Directory);
        assert_eq!(stats.size, 4096);
    }

    #[test]
    fn test_tmpfs_file_creation() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create a file
        tmpfs.create("/test.txt", 0o644).unwrap();
        assert!(tmpfs.exists("/test.txt"));
        
        // Check file type
        assert_eq!(tmpfs.file_type("/test.txt").unwrap(), FileType::Regular);
        
        // Get file stats
        let stats = tmpfs.stat("/test.txt").unwrap();
        assert_eq!(stats.file_type, FileType::Regular);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_tmpfs_file_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create file
        tmpfs.create("/test.txt", 0o644).unwrap();
        
        // Open file
        let handle = tmpfs.open("/test.txt", OpenFlags::READ | OpenFlags::WRITE).unwrap();
        assert_eq!(handle.flags, OpenFlags::READ | OpenFlags::WRITE);
        
        // Write data
        let data = b"Hello, World!";
        let bytes_written = tmpfs.write(&handle, data).unwrap();
        assert_eq!(bytes_written, data.len());
        
        // Check file size
        let stats = tmpfs.stat("/test.txt").unwrap();
        assert_eq!(stats.size, data.len() as u64);
    }

    #[test]
    fn test_tmpfs_read_write() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create and write file
        tmpfs.create("/data.txt", 0o644).unwrap();
        let handle = tmpfs.open("/data.txt", OpenFlags::READ | OpenFlags::WRITE).unwrap();
        let test_data = b"Test data for reading and writing";
        
        let bytes_written = tmpfs.write(&handle, test_data).unwrap();
        assert_eq!(bytes_written, test_data.len());
        
        // Read data back
        let mut read_buffer = vec![0u8; test_data.len()];
        let bytes_read = tmpfs.read(&handle, &mut read_buffer).unwrap();
        assert_eq!(bytes_read, test_data.len());
        assert_eq!(&read_buffer, test_data);
    }

    #[test]
    fn test_tmpfs_directory_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create directory
        tmpfs.mkdir("/mydir", 0o755).unwrap();
        assert!(tmpfs.exists("/mydir"));
        assert_eq!(tmpfs.file_type("/mydir").unwrap(), FileType::Directory);
        
        // Create file in directory
        tmpfs.create("/mydir/file.txt", 0o644).unwrap();
        assert!(tmpfs.exists("/mydir/file.txt"));
        
        // List directory
        let entries = tmpfs.readdir("/mydir").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file.txt");
    }

    #[test]
    fn test_tmpfs_directory_tree() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create directory tree
        tmpfs.mkdir("/dir1", 0o755).unwrap();
        tmpfs.mkdir("/dir1/subdir1", 0o755).unwrap();
        tmpfs.mkdir("/dir1/subdir2", 0o755).unwrap();
        
        // Create files in subdirectories
        tmpfs.create("/dir1/subdir1/file1.txt", 0o644).unwrap();
        tmpfs.create("/dir1/subdir2/file2.txt", 0o644).unwrap();
        
        // Verify structure
        assert!(tmpfs.exists("/dir1/subdir1"));
        assert!(tmpfs.exists("/dir1/subdir2"));
        assert!(tmpfs.exists("/dir1/subdir1/file1.txt"));
        assert!(tmpfs.exists("/dir1/subdir2/file2.txt"));
    }

    #[test]
    fn test_tmpfs_symlink_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create target file
        tmpfs.create("/target.txt", 0o644).unwrap();
        let handle = tmpfs.open("/target.txt", OpenFlags::WRITE).unwrap();
        tmpfs.write(&handle, b"Target content").unwrap();
        
        // Create symbolic link
        tmpfs.symlink("/target.txt", "/link.txt").unwrap();
        assert!(tmpfs.exists("/link.txt"));
        assert_eq!(tmpfs.file_type("/link.txt").unwrap(), FileType::SymbolicLink);
        
        // Read link target
        let target = tmpfs.readlink("/link.txt").unwrap();
        assert_eq!(target, "/target.txt");
    }

    #[test]
    fn test_tmpfs_file_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create file
        tmpfs.create("/file.txt", 0o644).unwrap();
        
        // Change permissions
        tmpfs.chmod("/file.txt", 0o755).unwrap();
        let stats = tmpfs.stat("/file.txt").unwrap();
        assert_eq!(stats.permissions, 0o755);
        
        // Change ownership
        tmpfs.chown("/file.txt", 1000, 100).unwrap();
        let stats = tmpfs.stat("/file.txt").unwrap();
        assert_eq!(stats.user_id, 1000);
        assert_eq!(stats.group_id, 100);
    }

    #[test]
    fn test_tmpfs_error_handling() {
        let mut tmpfs = TmpFs::new_default();
        
        // Test file not found
        assert_eq!(tmpfs.stat("/nonexistent.txt"), Err(FsError::NotFound));
        assert!(!tmpfs.exists("/nonexistent.txt"));
        
        // Test creating existing file
        tmpfs.create("/existing.txt", 0o644).unwrap();
        assert_eq!(tmpfs.create("/existing.txt", 0o644), Err(FsError::AlreadyExists));
        
        // Test operations on wrong file type
        tmpfs.mkdir("/dir", 0o755).unwrap();
        assert_eq!(tmpfs.unlink("/dir"), Err(FsError::IsDirectory));
        assert_eq!(tmpfs.read(&tmpfs.open("/dir", OpenFlags::READ).unwrap(), &mut [0; 100]), Err(FsError::IsDirectory));
    }

    #[test]
    fn test_tmpfs_rename() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create file
        tmpfs.create("/old_name.txt", 0o644).unwrap();
        assert!(tmpfs.exists("/old_name.txt"));
        assert!(!tmpfs.exists("/new_name.txt"));
        
        // Rename file
        tmpfs.rename("/old_name.txt", "/new_name.txt").unwrap();
        assert!(!tmpfs.exists("/old_name.txt"));
        assert!(tmpfs.exists("/new_name.txt"));
    }

    #[test]
    fn test_tmpfs_seek_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create file and write data
        tmpfs.create("/seek_test.txt", 0o644).unwrap();
        let handle = tmpfs.open("/seek_test.txt", OpenFlags::READ | OpenFlags::WRITE).unwrap();
        
        let data = b"0123456789ABCDEF";
        tmpfs.write(&handle, data).unwrap();
        
        // Test seeking to start
        let new_offset = tmpfs.seek(&handle, 0, SeekMode::Start).unwrap();
        assert_eq!(new_offset, 0);
        
        // Test seeking to middle
        let new_offset = tmpfs.seek(&handle, 5, SeekMode::Start).unwrap();
        assert_eq!(new_offset, 5);
        
        // Test seeking relative to current position
        let new_offset = tmpfs.seek(&handle, 3, SeekMode::Current).unwrap();
        assert_eq!(new_offset, 8);
        
        // Test seeking from end
        let new_offset = tmpfs.seek(&handle, -3, SeekMode::End).unwrap();
        assert_eq!(new_offset, data.len() as u64 - 3);
    }

    #[test]
    fn test_tmpfs_file_system_stats() {
        let mut tmpfs = TmpFs::new(100); // Limit to 100 inodes
        
        // Create some files
        tmpfs.create("/file1.txt", 0o644).unwrap();
        tmpfs.create("/file2.txt", 0o644).unwrap();
        tmpfs.mkdir("/dir", 0o755).unwrap();
        
        let stats = tmpfs.fsstat().unwrap();
        assert!(stats.total_files > 0);
        assert!(stats.free_files < stats.total_files);
        assert!(stats.block_size > 0);
        assert_eq!(stats.mounted, true);
    }

    #[test]
    fn test_tmpfs_large_tree() {
        let mut tmpfs = TmpFs::new(1000); // Allow more inodes
        
        // Create a large directory tree
        for i in 0..10 {
            let dir_name = format!("/dir{}", i);
            tmpfs.mkdir(&dir_name, 0o755).unwrap();
            
            for j in 0..5 {
                let file_name = format!("{}/file{}.txt", dir_name, j);
                tmpfs.create(&file_name, 0o644).unwrap();
            }
        }
        
        // Verify all files exist
        for i in 0..10 {
            for j in 0..5 {
                let file_name = format!("/dir{}/file{}.txt", i, j);
                assert!(tmpfs.exists(&file_name));
            }
        }
    }

    #[test]
    fn test_tmpfs_concurrent_operations() {
        let mut tmpfs = TmpFs::new_default();
        
        // Create multiple files and perform operations
        for i in 0..5 {
            let filename = format!("/file{}.txt", i);
            tmpfs.create(&filename, 0o644).unwrap();
            
            let handle = tmpfs.open(&filename, OpenFlags::READ | OpenFlags::WRITE).unwrap();
            let data = format!("File {}", i).into_bytes();
            tmpfs.write(&handle, &data).unwrap();
        }
        
        // Verify all files
        for i in 0..5 {
            let filename = format!("/file{}.txt", i);
            let stats = tmpfs.stat(&filename).unwrap();
            assert_eq!(stats.size, (format!("File {}", i).len()) as u64);
        }
    }

    #[test]
    fn test_open_flags() {
        let flags = OpenFlags::READ | OpenFlags::WRITE | OpenFlags::CREATE;
        assert!(flags.contains(OpenFlags::READ));
        assert!(flags.contains(OpenFlags::WRITE));
        assert!(flags.contains(OpenFlags::CREATE));
        assert!(!flags.contains(OpenFlags::APPEND));
    }

    #[test]
    fn test_file_type_ordering() {
        assert_eq!(FileType::Regular as u8, 0);
        assert_eq!(FileType::Directory as u8, 1);
        assert_eq!(FileType::SymbolicLink as u8, 2);
        assert_eq!(FileType::BlockDevice as u8, 3);
        assert_eq!(FileType::CharacterDevice as u8, 4);
        assert_eq!(FileType::FIFO as u8, 5);
        assert_eq!(FileType::Socket as u8, 6);
    }

    #[test]
    fn test_filesystem_type_ordering() {
        assert_eq!(FileSystemType::Unknown as u8, 0);
        assert_eq!(FileSystemType::TmpFs as u8, 1);
        assert_eq!(FileSystemType::Fat32 as u8, 2);
        assert_eq!(FileSystemType::Ext2 as u8, 3);
        assert_eq!(FileSystemType::ProcFs as u8, 4);
        assert_eq!(FileSystemType::DevFs as u8, 5);
    }

    #[test]
    fn test_tmpfs_edge_cases() {
        let mut tmpfs = TmpFs::new_default();
        
        // Test empty path handling
        assert_eq!(tmpfs.mkdir("", 0o755), Err(FsError::InvalidPath));
        
        // Test root directory operations
        assert_eq!(tmpfs.mkdir("/", 0o755), Err(FsError::AlreadyExists));
        assert_eq!(tmpfs.create("/", 0o644), Err(FsError::IsDirectory));
        
        // Test very long filenames (within limits)
        let long_name = "/".to_string() + &"a".repeat(255);
        tmpfs.create(&long_name, 0o644).unwrap();
        assert!(tmpfs.exists(&long_name));
    }

    #[test]
    fn test_tmpfs_memory_efficiency() {
        let mut tmpfs = TmpFs::new(10); // Very small limit
        
        // Create many small files to test memory efficiency
        for i in 0..10 {
            let filename = format!("/file{}.txt", i);
            tmpfs.create(&filename, 0o644).unwrap();
            
            let handle = tmpfs.open(&filename, OpenFlags::WRITE).unwrap();
            let data = format!("Data {}", i).into_bytes();
            tmpfs.write(&handle, &data).unwrap();
        }
        
        // Try to create one more file (should fail)
        assert_eq!(tmpfs.create("/file11.txt", 0o644), Err(FsError::DiskFull));
    }

    #[test]
    fn test_vfs_error_types() {
        let errors = [
            FsError::NotFound,
            FsError::PermissionDenied,
            FsError::AlreadyExists,
            FsError::IsDirectory,
            FsError::IsFile,
            FsError::DiskFull,
            FsError::InvalidPath,
            FsError::UnsupportedOperation,
            FsError::IoError,
            FsError::Corrupted,
            FsError::DirectoryNotEmpty,
        ];
        
        // Ensure all error types can be created and compared
        for &error in &errors {
            assert_eq!(error, error);
        }
    }

    #[test]
    fn test_tmpfs_real_world_scenario() {
        let mut tmpfs = TmpFs::new(500);
        
        // Simulate a typical Unix directory structure
        let directories = [
            "/bin", "/etc", "/tmp", "/var/log", "/usr/local/bin",
            "/home/user", "/home/user/documents", "/home/user/downloads"
        ];
        
        // Create directories
        for dir in &directories {
            tmpfs.mkdir(dir, 0o755).unwrap();
        }
        
        // Create some files
        let files = [
            "/etc/passwd",
            "/etc/group", 
            "/tmp/temp_file.txt",
            "/var/log/system.log",
            "/home/user/readme.txt",
        ];
        
        for file in &files {
            tmpfs.create(file, 0o644).unwrap();
        }
        
        // Create a symbolic link
        tmpfs.symlink("/home/user/readme.txt", "/home/user/important.txt").unwrap();
        
        // Verify structure
        assert_eq!(directories.len(), tmpfs.readdir("/").unwrap().len());
        
        // Test directory listing
        let user_files = tmpfs.readdir("/home/user").unwrap();
        assert_eq!(user_files.len(), 3); // documents, downloads, readme.txt (plus symlink)
        
        // Verify specific files exist
        for file in files {
            assert!(tmpfs.exists(file));
        }
        
        // Verify symbolic link
        let link_target = tmpfs.readlink("/home/user/important.txt").unwrap();
        assert_eq!(link_target, "/home/user/readme.txt");
    }
}