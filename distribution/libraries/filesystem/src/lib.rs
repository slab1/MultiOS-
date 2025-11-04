//! MultiOS File System Framework
//! 
//! This module provides file system functionality for the MultiOS kernel,
//! supporting various file system types in a unified interface.

#![no_std]

use spin::Mutex;
use bitflags::bitflags;

pub mod vfs;
pub mod fat32;
pub mod ext2;
pub mod tmpfs;
pub mod mfs;
pub mod mfs_examples;
pub mod mfs_tests;

/// File system types supported by MultiOS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileSystemType {
    Unknown = 0,
    Mfs = 1,
    TmpFs = 2,
    Fat32 = 3,
    Ext2 = 4,
    ProcFs = 5,
    DevFs = 6,
}

/// File system result type
pub type FsResult<T> = Result<T, FsError>;

/// Error types for file system operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    IsDirectory,
    IsFile,
    DiskFull,
    InvalidPath,
    UnsupportedOperation,
    IoError,
    Corrupted,
    DirectoryNotEmpty,
}

/// File types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileType {
    Regular = 0,
    Directory = 1,
    SymbolicLink = 2,
    BlockDevice = 3,
    CharacterDevice = 4,
    FIFO = 5,
    Socket = 6,
}

/// Global virtual file system manager
pub static VFS_MANAGER: Mutex<Option<vfs::VfsManager>> = Mutex::new(None);

/// Initialize the file system framework
/// 
/// This function sets up the global VFS manager and must be called
/// during kernel initialization.
pub fn init() -> FsResult<()> {
    let mut manager_guard = VFS_MANAGER.lock();
    
    let manager = vfs::VfsManager::new();
    *manager_guard = Some(manager);
    
    Ok(())
}

/// Register a file system
pub fn register_fs(fs_type: FileSystemType, fs: vfs::FileSystemHandle) -> FsResult<()> {
    let mut manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(FsError::IoError)?;
        
    manager.register_fs(fs_type, fs)
}

/// Mount a file system
pub fn mount(mount_point: &str, fs_type: FileSystemType, device: Option<&str>) -> FsResult<()> {
    let mut manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(FsError::IoError)?;
        
    manager.mount(mount_point, fs_type, device)
}

/// Open a file
pub fn open_file(path: &str, flags: vfs::OpenFlags) -> FsResult<vfs::FileHandle> {
    let manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_ref()
        .ok_or(FsError::IoError)?;
        
    manager.open_file(path, flags)
}

/// Create a directory
pub fn create_dir(path: &str, mode: u32) -> FsResult<()> {
    let mut manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(FsError::IoError)?;
        
    manager.create_dir(path, mode)
}

/// Remove a file or directory
pub fn remove(path: &str, recursive: bool) -> FsResult<()> {
    let mut manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_mut()
        .ok_or(FsError::IoError)?;
        
    manager.remove(path, recursive)
}

/// Get file statistics
pub fn stat(path: &str) -> FsResult<vfs::FileStats> {
    let manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_ref()
        .ok_or(FsError::IoError)?;
        
    manager.stat(path)
}

/// List directory contents
pub fn read_dir(path: &str) -> FsResult<Vec<vfs::DirEntry>> {
    let manager_guard = VFS_MANAGER.lock();
    
    let manager = manager_guard
        .as_ref()
        .ok_or(FsError::IoError)?;
        
    manager.read_dir(path)
}

/// Number of mounted file systems
pub fn get_mount_count() -> usize {
    let manager_guard = VFS_MANAGER.lock();
    
    if let Some(manager) = manager_guard.as_ref() {
        manager.get_mount_count()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_system_type_ordering() {
        assert_eq!(FileSystemType::Unknown as u8, 0);
        assert_eq!(FileSystemType::Mfs as u8, 1);
        assert_eq!(FileSystemType::TmpFs as u8, 2);
        assert_eq!(FileSystemType::Fat32 as u8, 3);
        assert_eq!(FileSystemType::DevFs as u8, 6);
    }

    #[test]
    fn test_file_type_ordering() {
        assert_eq!(FileType::Regular as u8, 0);
        assert_eq!(FileType::Directory as u8, 1);
        assert_eq!(FileType::Socket as u8, 6);
    }

    #[test]
    fn test_fs_error_variants() {
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
        
        for (i, &error) in errors.iter().enumerate() {
            assert_eq!(error as usize, i);
        }
    }
}