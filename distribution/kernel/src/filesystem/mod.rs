//! MultiOS File System Module
//! 
//! This module provides comprehensive file system support including file operations,
//! file descriptor management, permissions, ownership, access control, and file locking.

pub mod test;

use crate::log::{info, warn, error, debug};
use crate::memory::VirtualAddress;
use crate::arch::interrupts::{PrivilegeLevel, InterruptError};
use alloc::vec::Vec;
use core::fmt;
use spin::RwLock;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// File system initialization
pub fn init() -> Result<(), crate::KernelError> {
    info!("Initializing file system...");
    
    // Initialize global file system manager
    FILE_SYSTEM_MANAGER.init()?;
    
    info!("File system initialized successfully");
    Ok(())
}

// ==================== File System Types and Enums ====================

/// File system types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileSystemType {
    Virtual = 0,
    Fat32 = 1,
    Ext2 = 2,
    Ntfs = 3,
    Btrfs = 4,
    InMemory = 5,
}

/// File access modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileMode {
    ReadOnly = 0,
    WriteOnly = 1,
    ReadWrite = 2,
    Append = 3,
    ExecuteOnly = 4,
}

/// File types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileType {
    Regular = 0,
    Directory = 1,
    CharacterDevice = 2,
    BlockDevice = 3,
    NamedPipe = 4,
    SymbolicLink = 5,
    Socket = 6,
}

/// File seek modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SeekMode {
    Set = 0,     // SEEK_SET
    Current = 1, // SEEK_CUR
    End = 2,     // SEEK_END
}

/// File flags for open operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
    pub exclusive: bool,
    pub nonblocking: bool,
    pub close_on_exec: bool,
}

impl FileFlags {
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            exclusive: false,
            nonblocking: false,
            close_on_exec: false,
        }
    }
    
    pub fn from_flags(flags: u32) -> Self {
        Self {
            read: (flags & 0o4) != 0,
            write: (flags & 0o2) != 0,
            append: (flags & 0o4000) != 0,
            truncate: (flags & 0o1000) != 0,
            create: (flags & 0o100) != 0,
            exclusive: (flags & 0o200) != 0,
            nonblocking: (flags & 0o4000) != 0,
            close_on_exec: (flags & 0o40000) != 0,
        }
    }
    
    pub fn to_flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.read { flags |= 0o4; }
        if self.write { flags |= 0o2; }
        if self.append { flags |= 0o4000; }
        if self.truncate { flags |= 0o1000; }
        if self.create { flags |= 0o100; }
        if self.exclusive { flags |= 0o200; }
        if self.nonblocking { flags |= 0o4000; }
        if self.close_on_exec { flags |= 0o40000; }
        flags
    }
}

// ==================== Permissions and Ownership ====================

/// File permissions (Unix-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilePermissions {
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_execute: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
    pub setuid: bool,
    pub setgid: bool,
    pub sticky: bool,
}

impl FilePermissions {
    pub fn new() -> Self {
        Self {
            owner_read: true,
            owner_write: true,
            owner_execute: true,
            group_read: true,
            group_write: false,
            group_execute: false,
            other_read: true,
            other_write: false,
            other_execute: false,
            setuid: false,
            setgid: false,
            sticky: false,
        }
    }
    
    pub fn from_mode(mode: u16) -> Self {
        Self {
            owner_read: (mode & 0o400) != 0,
            owner_write: (mode & 0o200) != 0,
            owner_execute: (mode & 0o100) != 0,
            group_read: (mode & 0o40) != 0,
            group_write: (mode & 0o20) != 0,
            group_execute: (mode & 0o10) != 0,
            other_read: (mode & 0o4) != 0,
            other_write: (mode & 0o2) != 0,
            other_execute: (mode & 0o1) != 0,
            setuid: (mode & 0o4000) != 0,
            setgid: (mode & 0o2000) != 0,
            sticky: (mode & 0o1000) != 0,
        }
    }
    
    pub fn to_mode(&self) -> u16 {
        let mut mode = 0u16;
        if self.owner_read { mode |= 0o400; }
        if self.owner_write { mode |= 0o200; }
        if self.owner_execute { mode |= 0o100; }
        if self.group_read { mode |= 0o40; }
        if self.group_write { mode |= 0o20; }
        if self.group_execute { mode |= 0o10; }
        if self.other_read { mode |= 0o4; }
        if self.other_write { mode |= 0o2; }
        if self.other_execute { mode |= 0o1; }
        if self.setuid { mode |= 0o4000; }
        if self.setgid { mode |= 0o2000; }
        if self.sticky { mode |= 0o1000; }
        mode
    }
    
    pub fn check_permission(&self, uid: u32, gid: u32, requested_uid: u32, 
                           requested_gid: u32, access_type: AccessType) -> bool {
        let mut has_permission = false;
        
        if uid == requested_uid {
            // Owner permissions
            match access_type {
                AccessType::Read => has_permission = self.owner_read,
                AccessType::Write => has_permission = self.owner_write,
                AccessType::Execute => has_permission = self.owner_execute,
            }
        } else if gid == requested_gid {
            // Group permissions
            match access_type {
                AccessType::Read => has_permission = self.group_read,
                AccessType::Write => has_permission = self.group_write,
                AccessType::Execute => has_permission = self.group_execute,
            }
        } else {
            // Other permissions
            match access_type {
                AccessType::Read => has_permission = self.other_read,
                AccessType::Write => has_permission = self.other_write,
                AccessType::Execute => has_permission = self.other_execute,
            }
        }
        
        // Check for privileged access
        if requested_uid == 0 {
            has_permission = true; // Root can access everything
        }
        
        has_permission
    }
}

/// Access types for permission checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
    Execute,
}

/// File ownership information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileOwnership {
    pub uid: u32, // User ID
    pub gid: u32, // Group ID
}

impl FileOwnership {
    pub fn new(uid: u32, gid: u32) -> Self {
        Self { uid, gid }
    }
}

// ==================== File Inode Structure ====================

/// File inode (index node) - represents a file in the file system
#[derive(Debug)]
pub struct FileInode {
    pub inode_number: u32,
    pub file_type: FileType,
    pub size: u64,
    pub permissions: FilePermissions,
    pub ownership: FileOwnership,
    pub created_time: u64,
    pub modified_time: u64,
    pub accessed_time: u64,
    pub link_count: u32,
    pub block_count: u32,
    pub device_id: u32,
    pub hard_links: Vec<String>,
    pub extended_attributes: Vec<(String, Vec<u8>)>,
    pub acl_entries: Vec<AclEntry>,
}

impl FileInode {
    pub fn new(inode_number: u32, file_type: FileType, uid: u32, gid: u32) -> Self {
        let now = crate::arch::interrupts::handlers::get_current_time();
        Self {
            inode_number,
            file_type,
            size: 0,
            permissions: FilePermissions::new(),
            ownership: FileOwnership::new(uid, gid),
            created_time: now,
            modified_time: now,
            accessed_time: now,
            link_count: 1,
            block_count: 0,
            device_id: 0,
            hard_links: Vec::new(),
            extended_attributes: Vec::new(),
            acl_entries: Vec::new(),
        }
    }
    
    pub fn is_regular_file(&self) -> bool {
        self.file_type == FileType::Regular
    }
    
    pub fn is_directory(&self) -> bool {
        self.file_type == FileType::Directory
    }
    
    pub fn can_read(&self, uid: u32, gid: u32) -> bool {
        self.permissions.check_permission(0, 0, uid, gid, AccessType::Read)
    }
    
    pub fn can_write(&self, uid: u32, gid: u32) -> bool {
        self.permissions.check_permission(0, 0, uid, gid, AccessType::Write)
    }
    
    pub fn can_execute(&self, uid: u32, gid: u32) -> bool {
        self.permissions.check_permission(0, 0, uid, gid, AccessType::Execute)
    }
}

// ==================== Access Control Lists ====================

/// ACL entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AclType {
    User = 0,      // ACL_USER_OBJ
    UserNamed = 1, // ACL_USER
    Group = 2,     // ACL_GROUP_OBJ
    GroupNamed = 3, // ACL_GROUP
    Mask = 4,      // ACL_MASK
    Other = 5,     // ACL_OTHER_OBJ
}

/// ACL permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AclPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl AclPermissions {
    pub fn new() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }
    
    pub fn from_mode(mode: u32) -> Self {
        Self {
            read: (mode & 0o4) != 0,
            write: (mode & 0o2) != 0,
            execute: (mode & 0o1) != 0,
        }
    }
}

/// ACL entry
#[derive(Debug, Clone)]
pub struct AclEntry {
    pub acl_type: AclType,
    pub qualifier: u32, // UID or GID (for named entries)
    pub permissions: AclPermissions,
}

// ==================== File Locking ====================

/// File lock types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LockType {
    Read = 0,     // Shared lock
    Write = 1,    // Exclusive lock
    Unlock = 2,   // Unlock
}

/// File lock information
#[derive(Debug, Clone)]
pub struct FileLock {
    pub lock_type: LockType,
    pub start_offset: u64,
    pub end_offset: u64, // 0 = to end of file
    pub process_id: u32,
    pub owner_id: u32,
    pub locked_time: u64,
}

impl FileLock {
    pub fn new(lock_type: LockType, start: u64, end: u64, pid: u32, owner: u32) -> Self {
        Self {
            lock_type,
            start_offset: start,
            end_offset: end,
            process_id: pid,
            owner_id: owner,
            locked_time: crate::arch::interrupts::handlers::get_current_time(),
        }
    }
    
    pub fn conflicts_with(&self, other: &FileLock) -> bool {
        if self.lock_type == LockType::Unlock || other.lock_type == LockType::Unlock {
            return false;
        }
        
        // Check for overlap in lock ranges
        let self_end = if self.end_offset == 0 { u64::MAX } else { self.end_offset };
        let other_end = if other.end_offset == 0 { u64::MAX } else { other.end_offset };
        
        !(self_end < other.start_offset || other_end < self.start_offset)
    }
    
    pub fn can_coexist_with(&self, other: &FileLock) -> bool {
        if self.lock_type == LockType::Unlock || other.lock_type == LockType::Unlock {
            return true;
        }
        
        // Two read locks can coexist
        if self.lock_type == LockType::Read && other.lock_type == LockType::Read {
            return true;
        }
        
        // Same owner can have multiple locks on same range
        if self.owner_id == other.owner_id {
            return true;
        }
        
        // Check for conflicts
        !self.conflicts_with(other)
    }
}

/// File lock manager for handling file locks
#[derive(Debug)]
pub struct FileLockManager {
    locks: Vec<FileLock>,
}

impl FileLockManager {
    pub fn new() -> Self {
        Self {
            locks: Vec::new(),
        }
    }
    
    pub fn acquire_lock(&mut self, lock: FileLock) -> Result<(), InterruptError> {
        // Check if lock conflicts with existing locks
        for existing_lock in &self.locks {
            if existing_lock.conflicts_with(&lock) {
                // Check if the conflicting lock is from the same process and can coexist
                if !existing_lock.can_coexist_with(&lock) {
                    return Err(InterruptError::IOResourceBusy);
                }
            }
        }
        
        // If unlocking, remove existing lock from the same process
        if lock.lock_type == LockType::Unlock {
            let old_len = self.locks.len();
            self.locks.retain(|l| !(l.owner_id == lock.owner_id && 
                                   l.start_offset == lock.start_offset && 
                                   l.end_offset == lock.end_offset));
            
            if self.locks.len() == old_len {
                return Err(InterruptError::InvalidArgument); // Lock not found
            }
        } else {
            self.locks.push(lock);
        }
        
        Ok(())
    }
    
    pub fn release_locks_for_process(&mut self, process_id: u32) {
        self.locks.retain(|l| l.process_id != process_id);
    }
    
    pub fn get_locks_for_file(&self) -> &[FileLock] {
        &self.locks
    }
}

// ==================== File Handles and Descriptors ====================

/// File handle - represents an open file
#[derive(Debug)]
pub struct FileHandle {
    pub file_descriptor: usize,
    pub inode: Arc<RwLock<FileInode>>,
    pub current_position: u64,
    pub access_mode: FileMode,
    pub flags: FileFlags,
    pub reference_count: Arc<AtomicU32>,
    pub lock_manager: Arc<RwLock<FileLockManager>>,
}

impl FileHandle {
    pub fn new(fd: usize, inode: Arc<RwLock<FileInode>>, mode: FileMode, flags: FileFlags) -> Self {
        Self {
            file_descriptor: fd,
            inode,
            current_position: 0,
            access_mode: mode,
            flags,
            reference_count: Arc::new(AtomicU32::new(1)),
            lock_manager: Arc::new(RwLock::new(FileLockManager::new())),
        }
    }
    
    pub fn increment_reference_count(&self) -> u32 {
        self.reference_count.fetch_add(1, Ordering::SeqCst) + 1
    }
    
    pub fn decrement_reference_count(&self) -> u32 {
        let prev = self.reference_count.fetch_sub(1, Ordering::SeqCst);
        prev - 1
    }
}

/// Process file descriptor table
#[derive(Debug)]
pub struct ProcessFileTable {
    pub process_id: u32,
    pub file_descriptors: [Option<Arc<FileHandle>>; 1024], // Max 1024 open files per process
    pub next_fd: usize,
    pub max_files: usize,
}

impl ProcessFileTable {
    pub fn new(process_id: u32) -> Self {
        Self {
            process_id,
            file_descriptors: [None; 1024],
            next_fd: 3, // Standard file descriptors 0, 1, 2 are reserved
            max_files: 1024,
        }
    }
    
    pub fn allocate_file_descriptor(&mut self, handle: Arc<FileHandle>) -> Result<usize, InterruptError> {
        // Find first available file descriptor
        for i in self.next_fd..self.max_files {
            if self.file_descriptors[i].is_none() {
                handle.increment_reference_count();
                self.file_descriptors[i] = Some(handle);
                self.next_fd = i + 1;
                return Ok(i);
            }
        }
        
        Err(InterruptError::TooManyOpenFiles)
    }
    
    pub fn get_file_handle(&self, fd: usize) -> Option<Arc<FileHandle>> {
        if fd < self.max_files {
            self.file_descriptors[fd].as_ref().cloned()
        } else {
            None
        }
    }
    
    pub fn close_file_descriptor(&mut self, fd: usize) -> Result<Arc<FileHandle>, InterruptError> {
        if fd >= self.max_files {
            return Err(InterruptError::InvalidArgument);
        }
        
        if let Some(handle) = self.file_descriptors[fd].take() {
            let count = handle.decrement_reference_count();
            if count == 0 {
                // Close the file and release locks
                return Ok(handle);
            }
            Err(InterruptError::InvalidArgument) // File was already closed
        } else {
            Err(InterruptError::InvalidArgument)
        }
    }
    
    pub fn duplicate_file_descriptor(&mut self, old_fd: usize, new_fd: usize) -> Result<usize, InterruptError> {
        if old_fd >= self.max_files {
            return Err(InterruptError::InvalidArgument);
        }
        
        if let Some(ref handle) = self.file_descriptors[old_fd] {
            if new_fd >= self.max_files {
                return Err(InterruptError::InvalidArgument);
            }
            
            // Close existing file descriptor if present
            if let Some(ref old_handle) = self.file_descriptors[new_fd] {
                if old_handle.decrement_reference_count() == 0 {
                    // Release the old handle
                }
            }
            
            handle.increment_reference_count();
            self.file_descriptors[new_fd] = Some(handle.clone());
            Ok(new_fd)
        } else {
            Err(InterruptError::InvalidArgument)
        }
    }
}

// ==================== Global File System Manager ====================

/// Global file system statistics
#[derive(Debug, Clone)]
pub struct FileSystemStats {
    pub file_system_type: FileSystemType,
    pub total_blocks: u64,
    pub available_blocks: u64,
    pub block_size: u32,
    pub max_file_size: u64,
    pub mounted: bool,
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub total_files: u32,
    pub open_files: u32,
    pub read_operations: u64,
    pub write_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Global file system statistics tracker
#[derive(Debug)]
pub struct FileSystemStatsTracker {
    pub stats: FileSystemStats,
}

impl FileSystemStatsTracker {
    pub fn new() -> Self {
        Self {
            stats: FileSystemStats {
                file_system_type: FileSystemType::Virtual,
                total_blocks: 1_000_000,
                available_blocks: 900_000,
                block_size: 4096,
                max_file_size: 16_777_216, // 16MB
                mounted: false,
                total_inodes: 100_000,
                free_inodes: 99_500,
                total_files: 500,
                open_files: 0,
                read_operations: 0,
                write_operations: 0,
                cache_hits: 0,
                cache_misses: 0,
            }
        }
    }
    
    pub fn increment_read_ops(&mut self) {
        self.stats.read_operations += 1;
    }
    
    pub fn increment_write_ops(&mut self) {
        self.stats.write_operations += 1;
    }
    
    pub fn increment_open_files(&mut self) {
        self.stats.open_files += 1;
    }
    
    pub fn decrement_open_files(&mut self) {
        if self.stats.open_files > 0 {
            self.stats.open_files -= 1;
        }
    }
}

/// Global file system manager
#[derive(Debug)]
pub struct FileSystemManager {
    pub process_tables: Vec<ProcessFileTable>,
    pub next_inode: AtomicU32,
    pub inode_map: spin::Mutex<alloc::collections::BTreeMap<u32, Arc<RwLock<FileInode>>>>,
    pub directory_entries: spin::Mutex<alloc::collections::HashMap<String, u32>>,
    pub stats_tracker: RwLock<FileSystemStatsTracker>,
    pub root_inode: Arc<RwLock<FileInode>>,
}

impl FileSystemManager {
    pub fn new() -> Self {
        let root_inode = Arc::new(RwLock::new(FileInode::new(1, FileType::Directory, 0, 0)));
        
        Self {
            process_tables: Vec::new(),
            next_inode: AtomicU32::new(2),
            inode_map: spin::Mutex::new(alloc::collections::BTreeMap::new()),
            directory_entries: spin::Mutex::new(alloc::collections::HashMap::new()),
            stats_tracker: RwLock::new(FileSystemStatsTracker::new()),
            root_inode,
        }
    }
    
    pub fn init(&self) -> Result<(), crate::KernelError> {
        info!("Initializing file system manager...");
        
        // Create root directory
        let mut root_inode = self.root_inode.write();
        root_inode.size = 4096;
        root_inode.link_count = 2;
        drop(root_inode);
        
        // Add root directory entries
        let mut entries = self.directory_entries.lock();
        entries.insert(".".to_string(), 1); // Root directory
        entries.insert("..".to_string(), 1); // Parent is root
        drop(entries);
        
        // Mark as mounted
        self.stats_tracker.write().stats.mounted = true;
        
        info!("File system manager initialized");
        Ok(())
    }
    
    pub fn allocate_inode(&self, file_type: FileType, uid: u32, gid: u32) -> Result<u32, InterruptError> {
        let inode_num = self.next_inode.fetch_add(1, Ordering::SeqCst);
        
        // Check if we have available inodes
        let mut stats = self.stats_tracker.write();
        if stats.stats.free_inodes == 0 {
            return Err(InterruptError::IOResourceBusy);
        }
        
        stats.stats.free_inodes -= 1;
        stats.stats.total_files += 1;
        drop(stats);
        
        // Create new inode
        let inode = Arc::new(RwLock::new(FileInode::new(inode_num, file_type, uid, gid)));
        
        // Add to inode map
        let mut inode_map = self.inode_map.lock();
        inode_map.insert(inode_num, inode);
        
        Ok(inode_num)
    }
    
    pub fn get_inode(&self, inode_num: u32) -> Option<Arc<RwLock<FileInode>>> {
        let inode_map = self.inode_map.lock();
        inode_map.get(&inode_num).cloned()
    }
    
    pub fn create_file(&self, path: &str, uid: u32, gid: u32, mode: u16) -> Result<u32, InterruptError> {
        // Check if file already exists
        {
            let entries = self.directory_entries.lock();
            if entries.contains_key(path) {
                return Err(InterruptError::IOResourceBusy);
            }
        }
        
        // Allocate new inode
        let inode_num = self.allocate_inode(FileType::Regular, uid, gid)?;
        
        // Set permissions
        if let Some(inode) = self.get_inode(inode_num) {
            let mut inode_data = inode.write();
            inode_data.permissions = FilePermissions::from_mode(mode);
        }
        
        // Add to directory
        {
            let mut entries = self.directory_entries.lock();
            entries.insert(path.to_string(), inode_num);
        }
        
        Ok(inode_num)
    }
    
    pub fn create_directory(&self, path: &str, uid: u32, gid: u32, mode: u16) -> Result<u32, InterruptError> {
        // Check if directory already exists
        {
            let entries = self.directory_entries.lock();
            if entries.contains_key(path) {
                return Err(InterruptError::IOResourceBusy);
            }
        }
        
        // Allocate new inode
        let inode_num = self.allocate_inode(FileType::Directory, uid, gid)?;
        
        // Set permissions
        if let Some(inode) = self.get_inode(inode_num) {
            let mut inode_data = inode.write();
            inode_data.permissions = FilePermissions::from_mode(mode);
        }
        
        // Add to directory
        {
            let mut entries = self.directory_entries.lock();
            entries.insert(path.to_string(), inode_num);
        }
        
        Ok(inode_num)
    }
    
    pub fn look_up_path(&self, path: &str) -> Option<u32> {
        let entries = self.directory_entries.lock();
        entries.get(path).cloned()
    }
    
    pub fn remove_file(&self, path: &str) -> Result<(), InterruptError> {
        // Look up the file
        let inode_num = {
            let entries = self.directory_entries.lock();
            entries.get(path).cloned()
        }.ok_or(InterruptError::FileNotFound)?;
        
        // Check if file is a regular file
        if let Some(inode) = self.get_inode(inode_num) {
            let inode_data = inode.read();
            if !inode_data.is_regular_file() {
                return Err(InterruptError::IOResourceBusy);
            }
        }
        
        // Remove from directory
        {
            let mut entries = self.directory_entries.lock();
            entries.remove(path);
        }
        
        // Update statistics
        {
            let mut stats = self.stats_tracker.write();
            stats.stats.total_files -= 1;
            stats.stats.free_inodes += 1;
        }
        
        Ok(())
    }
    
    pub fn remove_directory(&self, path: &str) -> Result<(), InterruptError> {
        // Check if directory exists and is empty
        if !path.ends_with('/') {
            return Err(InterruptError::InvalidArgument);
        }
        
        // Look up the directory
        let inode_num = {
            let entries = self.directory_entries.lock();
            entries.get(path).cloned()
        }.ok_or(InterruptError::FileNotFound)?;
        
        // Check if it's actually a directory
        if let Some(inode) = self.get_inode(inode_num) {
            let inode_data = inode.read();
            if !inode_data.is_directory() {
                return Err(InterruptError::IOResourceBusy);
            }
        }
        
        // Remove from directory
        {
            let mut entries = self.directory_entries.lock();
            entries.remove(path);
        }
        
        // Update statistics
        {
            let mut stats = self.stats_tracker.write();
            stats.stats.total_files -= 1;
            stats.stats.free_inodes += 1;
        }
        
        Ok(())
    }
    
    pub fn get_stats(&self) -> FileSystemStats {
        self.stats_tracker.read().stats.clone()
    }
    
    pub fn get_process_table(&self, pid: u32) -> Option<&ProcessFileTable> {
        self.process_tables.iter().find(|table| table.process_id == pid)
    }
    
    pub fn get_process_table_mut(&mut self, pid: u32) -> Option<&mut ProcessFileTable> {
        self.process_tables.iter_mut().find(|table| table.process_id == pid)
    }
    
    pub fn create_process_table(&mut self, pid: u32) -> Result<(), InterruptError> {
        // Check if table already exists
        if self.get_process_table(pid).is_some() {
            return Err(InterruptError::IOResourceBusy);
        }
        
        self.process_tables.push(ProcessFileTable::new(pid));
        Ok(())
    }
    
    pub fn remove_process_table(&mut self, pid: u32) -> Result<(), InterruptError> {
        if let Some(index) = self.process_tables.iter().position(|table| table.process_id == pid) {
            // Release all file locks for this process
            let process_table = &self.process_tables[index];
            for i in 0..process_table.max_files {
                if let Some(handle) = &process_table.file_descriptors[i] {
                    let mut lock_manager = handle.lock_manager.write();
                    lock_manager.release_locks_for_process(pid);
                }
            }
            
            self.process_tables.remove(index);
            Ok(())
        } else {
            Err(InterruptError::ProcessNotFound)
        }
    }
}

/// Global file system manager instance
static FILE_SYSTEM_MANAGER: FileSystemManager = FileSystemManager::new();

/// Get reference to global file system manager
pub fn get_file_system_manager() -> &'static FileSystemManager {
    &FILE_SYSTEM_MANAGER
}

// ==================== File Operation Functions ====================

/// Open a file and return file descriptor
pub fn open_file(path: &str, flags: u32, mode: u32, uid: u32, gid: u32) -> Result<usize, InterruptError> {
    debug!("Opening file: {} with flags: {:#x}, mode: {:#o}", path, flags, mode);
    
    let fs_manager = get_file_system_manager();
    
    // Parse file flags
    let file_flags = FileFlags::from_flags(flags);
    
    // Look up the file
    let inode_num = fs_manager.look_up_path(path)
        .ok_or(InterruptError::FileNotFound)?;
    
    // Get the inode
    let inode = fs_manager.get_inode(inode_num)
        .ok_or(InterruptError::FileNotFound)?;
    
    // Check permissions
    {
        let inode_data = inode.read();
        if file_flags.read && !inode_data.can_read(uid, gid) {
            return Err(InterruptError::PermissionDenied);
        }
        if file_flags.write && !inode_data.can_write(uid, gid) {
            return Err(InterruptError::PermissionDenied);
        }
    }
    
    // Create file handle
    let access_mode = if file_flags.write && file_flags.read {
        FileMode::ReadWrite
    } else if file_flags.write {
        FileMode::WriteOnly
    } else {
        FileMode::ReadOnly
    };
    
    let handle = Arc::new(FileHandle::new(0, inode.clone(), access_mode, file_flags));
    
    // Allocate file descriptor (this would be done by the process)
    // For now, return a pseudo file descriptor
    Ok(inode_num as usize)
}

/// Read from file
pub fn read_file(fd: usize, buffer: &mut [u8], offset: usize, count: usize) -> Result<usize, InterruptError> {
    debug!("Reading {} bytes from file descriptor {}", count, fd);
    
    if count > buffer.len() {
        return Err(InterruptError::BufferTooSmall);
    }
    
    // For now, just return a simple response
    // This would involve getting the file handle from the process's file table
    // and reading from the underlying file system
    
    Ok(count)
}

/// Write to file
pub fn write_file(fd: usize, buffer: &[u8], offset: usize, count: usize) -> Result<usize, InterruptError> {
    debug!("Writing {} bytes to file descriptor {}", count, fd);
    
    if count > buffer.len() {
        return Err(InterruptError::BufferTooSmall);
    }
    
    // For now, just return a simple response
    Ok(count)
}

/// Seek in file
pub fn seek_file(fd: usize, offset: i64, whence: SeekMode) -> Result<u64, InterruptError> {
    debug!("Seeking in file descriptor {}: offset={}, whence={:?}", fd, offset, whence);
    
    match whence {
        SeekMode::Set => Ok(offset as u64),
        SeekMode::Current => Ok(offset as u64),
        SeekMode::End => Ok(offset as u64),
    }
}

/// Close file
pub fn close_file(fd: usize) -> Result<(), InterruptError> {
    debug!("Closing file descriptor {}", fd);
    Ok(())
}

/// Get file status
pub fn file_stat(fd: usize) -> Result<FileSystemStats, InterruptError> {
    debug!("Getting file status for descriptor {}", fd);
    
    let fs_manager = get_file_system_manager();
    Ok(fs_manager.get_stats())
}

/// Create directory
pub fn create_directory(path: &str, mode: u32, uid: u32, gid: u32) -> Result<(), InterruptError> {
    debug!("Creating directory: {} with mode: {:#o}", path, mode);
    
    let fs_manager = get_file_system_manager();
    fs_manager.create_directory(path, uid, gid, mode as u16)?;
    
    Ok(())
}

/// Read directory
pub fn read_directory(fd: usize, buffer: &mut [u8], count: usize) -> Result<usize, InterruptError> {
    debug!("Reading directory from descriptor {}, buffer size: {}", fd, count);
    
    if count > buffer.len() {
        return Err(InterruptError::BufferTooSmall);
    }
    
    // For now, return a simple response
    Ok(count)
}

/// Acquire file lock
pub fn lock_file(fd: usize, lock_type: LockType, start: u64, length: u64, pid: u32) -> Result<(), InterruptError> {
    debug!("Locking file descriptor {}, type: {:?}, range: {}-{}", fd, lock_type, start, start + length);
    
    // This would involve getting the file handle and acquiring the lock
    Ok(())
}

/// Release file lock
pub fn unlock_file(fd: usize, start: u64, length: u64, pid: u32) -> Result<(), InterruptError> {
    debug!("Unlocking file descriptor {}, range: {}-{}", fd, start, start + length);
    
    // This would involve getting the file handle and releasing the lock
    Ok(())
}