//! MultiOS File System (MFS) Implementation
//! 
//! A modern, efficient file system with journaling, indexing, security,
//! and support for large files.

#![no_std]

use bitflags::bitflags;
use core::cmp::{min, max};
use core::mem::size_of;
use log::info;
use spin::{Mutex, RwLock};
use crate::{FsError, FsResult, FileType};

/// MFS magic number for superblock identification
pub const MFS_MAGIC: u32 = 0x4D465300; // "MFS\0"
pub const MFS_VERSION: u16 = 1;

/// Block size in bytes (4KB - modern standard)
pub const MFS_BLOCK_SIZE: u64 = 4096;
/// Number of blocks per group (simplified for this implementation)
pub const MFS_BLOCKS_PER_GROUP: u64 = 8192;

/// Maximum file size (16TB - supports very large files)
pub const MFS_MAX_FILE_SIZE: u64 = 16 * 1024 * 1024 * 1024 * 1024;
/// Maximum file name length
pub const MFS_MAX_NAME_LENGTH: u255 = 255;
/// Maximum path length
pub const MFS_MAX_PATH_LENGTH: u4096 = 4096;

/// Journal size (64MB default)
pub const MFS_JOURNAL_SIZE: u64 = 64 * 1024 * 1024;

/// Superblock structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MfsSuperblock {
    pub magic: u32,
    pub version: u16,
    pub block_size: u32,
    pub block_count: u64,
    pub free_blocks: u64,
    pub block_group_count: u32,
    pub inode_count: u64,
    pub free_inodes: u64,
    pub first_data_block: u64,
    pub journal_block: u64,
    pub journal_size: u32,
    pub features: MfsFeatures,
    pub create_time: u64,
    pub last_mount_time: u64,
    pub mount_count: u32,
    pub max_mount_count: u16,
    pub state: MfsState,
    pub error_handling: MfsErrorHandling,
    pub revision_level: u16,
    pub reserved_blocks_percentage: u8,
    pub block_allocation_hint: u32,
    pub encryption_key: u64,
    pub checksum: u32,
}

bitflags! {
    /// MFS features flags
    #[derive(Debug, Clone, Copy)]
    pub struct MfsFeatures: u32 {
        const JOURNALING = 0x00000001;
        const INDEXING = 0x00000002;
        const SECURITY = 0x00000004;
        const COMPRESSION = 0x00000008;
        const LARGE_FILES = 0x00000010;
        const ENCRYPTION = 0x00000020;
        const ATTRIBUTES = 0x00000040;
        const QUOTAS = 0x00000080;
        const ACL = 0x00000100;
    }

    /// File system state
    #[derive(Debug, Clone, Copy)]
    pub struct MfsState: u16 {
        const CLEAN = 0x0001;
        const ERROR = 0x0002;
        const ORPHAN_INODES = 0x0004;
    }

    /// Error handling modes
    #[derive(Debug, Clone, Copy)]
    pub struct MfsErrorHandling: u16 {
        const IGNORE = 0x0001;
        const REMOUNT_READ_ONLY = 0x0002;
        const PANIC = 0x0003;
    }
}

/// Block group descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MfsBlockGroup {
    pub block_bitmap: u64,
    pub inode_bitmap: u64,
    pub inode_table: u64,
    pub free_blocks_count: u16,
    pub free_inodes_count: u16,
    pub used_dirs_count: u16,
    pub flags: MfsBlockGroupFlags,
    pub checksum: u16,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MfsBlockGroupFlags: u16 {
        const INODE_UNINIT = 0x0001;
        const BLOCKS_UNINIT = 0x0002;
        const ITABLE_ZEROED = 0x0004;
    }
}

/// Inode structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MfsInode {
    pub mode: u16,
    pub uid: u16,
    pub size: u64,
    pub access_time: u64,
    pub create_time: u64,
    pub modify_time: u64,
    pub delete_time: u64,
    pub gid: u16,
    pub links_count: u16,
    pub blocks: u64,
    pub flags: MfsInodeFlags,
    pub version: u32,
    pub blocks_high: u32,
    pub fragment_address: u32,
    pub direct_blocks: [u32; 12],
    pub single_indirect: u32,
    pub double_indirect: u32,
    pub triple_indirect: u32,
    pub generation: u32,
    pub extended_attributes: u32,
    pub access_acl: u32,
    pub default_acl: u32,
    pub update_time: u32,
    pub encryption_key_id: u16,
    pub reserved: u16,
    pub checksum: u16,
}

bitflags! {
    /// Inode flags
    #[derive(Debug, Clone, Copy)]
    pub struct MfsInodeFlags: u32 {
        const APPEND_ONLY = 0x00000020;
        const IMMUTABLE = 0x00000040;
        const SYMBOLIC_LINK = 0x00000080;
        const COMPRESSED = 0x00000100;
        const ENCRYPTED = 0x00000800;
        const DAX = 0x00010000;
        const INDEX_DIR = 0x00020000;
    }
}

/// Directory entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MfsDirEntry {
    pub inode: u32,
    pub entry_length: u16,
    pub name_length: u8,
    pub file_type: u8,
    pub name: [u8; 255],
}

/// Journal entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MfsJournalEntry {
    pub sequence: u64,
    pub block: u64,
    pub timestamp: u64,
    pub data: [u8; MFS_BLOCK_SIZE as usize],
}

/// Security extended attributes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MfsSecurityAttr {
    pub type_id: u16,
    pub length: u16,
    pub permissions: u32,
    pub owner_uid: u16,
    pub group_gid: u16,
    pub security_mode: u16,
    pub audit_id: u64,
    pub acl_entries: u8,
    pub reserved: [u8; 11],
}

/// File allocation methods
pub trait MfsAllocator {
    /// Allocate blocks for a file
    fn allocate_blocks(&mut self, count: u32, preferred_hint: Option<u64>) -> FsResult<Vec<u64>>;
    
    /// Deallocate blocks
    fn deallocate_blocks(&mut self, blocks: &[u64]) -> FsResult<()>;
    
    /// Find free blocks
    fn find_free_blocks(&self, count: u32) -> FsResult<Vec<u64>>;
}

/// Indexed block allocator implementation
pub struct MfsIndexedAllocator {
    block_bitmap: Vec<u8>,
    free_blocks: u64,
    bitmap_blocks: u64,
}

impl MfsIndexedAllocator {
    pub fn new(total_blocks: u64) -> Self {
        let bitmap_blocks = (total_blocks + 7) / 8;
        let mut block_bitmap = vec![0u8; bitmap_blocks as usize];
        
        // Reserve blocks for superblock, bitmaps, and metadata
        let reserved_blocks = 1 + bitmap_blocks + 1; // superblock + bitmap + block groups
        
        // Mark reserved blocks as used
        for i in 0..reserved_blocks {
            block_bitmap[(i / 8) as usize] |= 1 << (i % 8);
        }
        
        Self {
            block_bitmap,
            free_blocks: total_blocks - reserved_blocks,
            bitmap_blocks,
        }
    }
    
    /// Mark blocks as used
    fn set_blocks_used(&mut self, blocks: &[u64]) {
        for &block in blocks {
            let byte_index = (block / 8) as usize;
            let bit_index = block % 8;
            self.block_bitmap[byte_index] |= 1 << bit_index;
            self.free_blocks -= 1;
        }
    }
    
    /// Mark blocks as free
    fn set_blocks_free(&mut self, blocks: &[u64]) {
        for &block in blocks {
            let byte_index = (block / 8) as usize;
            let bit_index = block % 8;
            self.block_bitmap[byte_index] &= !(1 << bit_index);
            self.free_blocks += 1;
        }
    }
    
    /// Find consecutive free blocks
    fn find_consecutive_free(&self, count: u32) -> Option<u64> {
        let mut consecutive_count = 0;
        let mut start_block = None;
        
        for byte_idx in 0..self.block_bitmap.len() {
            let byte = self.block_bitmap[byte_idx];
            for bit_idx in 0..8 {
                let block_num = (byte_idx as u64 * 8) + bit_idx;
                if block_num >= self.bitmap_blocks * 8 {
                    break;
                }
                
                if (byte & (1 << bit_idx)) == 0 {
                    // Block is free
                    if consecutive_count == 0 {
                        start_block = Some(block_num);
                    }
                    consecutive_count += 1;
                    
                    if consecutive_count >= count {
                        return start_block;
                    }
                } else {
                    // Block is used, reset counter
                    consecutive_count = 0;
                    start_block = None;
                }
            }
        }
        
        None
    }
}

impl MfsAllocator for MfsIndexedAllocator {
    fn allocate_blocks(&mut self, count: u32, preferred_hint: Option<u64>) -> FsResult<Vec<u64>> {
        let mut allocated_blocks = Vec::new();
        
        // Try preferred hint first if provided
        if let Some(hint) = preferred_hint {
            if self.is_block_free(hint) {
                let available = self.count_consecutive_free_from(hint);
                let to_allocate = min(count, available);
                
                for i in 0..to_allocate {
                    allocated_blocks.push(hint + i as u64);
                }
                
                if allocated_blocks.len() < count as usize {
                    let remaining = count - allocated_blocks.len() as u32;
                    let additional = self.find_free_blocks(remaining)?;
                    allocated_blocks.extend(additional);
                }
            } else {
                allocated_blocks = self.find_free_blocks(count)?;
            }
        } else {
            allocated_blocks = self.find_free_blocks(count)?;
        }
        
        self.set_blocks_used(&allocated_blocks);
        Ok(allocated_blocks)
    }
    
    fn deallocate_blocks(&mut self, blocks: &[u64]) -> FsResult<()> {
        self.set_blocks_free(blocks);
        Ok(())
    }
    
    fn find_free_blocks(&self, count: u32) -> FsResult<Vec<u64>> {
        let mut free_blocks = Vec::new();
        let mut remaining = count;
        
        let mut search_from = 0;
        while remaining > 0 && search_from < self.bitmap_blocks * 8 {
            // Find consecutive free blocks
            if let Some(start) = self.find_consecutive_free_from(search_from, remaining) {
                let available = self.count_consecutive_free_from(start);
                let to_take = min(remaining, available);
                
                for i in 0..to_take {
                    free_blocks.push(start + i as u64);
                }
                
                remaining -= to_take;
                search_from = start + to_take as u64;
            } else {
                break;
            }
        }
        
        if free_blocks.len() == count as usize {
            Ok(free_blocks)
        } else {
            Err(FsError::DiskFull)
        }
    }
}

impl MfsIndexedAllocator {
    /// Check if a block is free
    fn is_block_free(&self, block: u64) -> bool {
        let byte_index = (block / 8) as usize;
        let bit_index = block % 8;
        byte_index < self.block_bitmap.len() && 
        (self.block_bitmap[byte_index] & (1 << bit_index)) == 0
    }
    
    /// Count consecutive free blocks starting from a position
    fn count_consecutive_free_from(&self, start: u64) -> u32 {
        let mut count = 0;
        let mut current = start;
        
        while self.is_block_free(current) {
            count += 1;
            current += 1;
        }
        
        count
    }
    
    /// Find consecutive free blocks starting from a position
    fn find_consecutive_free_from(&self, start: u64, max_count: u32) -> Option<u64> {
        if self.is_block_free(start) {
            Some(start)
        } else {
            let next_block = self.find_next_free_block(start + 1);
            if self.count_consecutive_free_from(next_block) >= max_count {
                Some(next_block)
            } else {
                None
            }
        }
    }
    
    /// Find next free block starting from position
    fn find_next_free_block(&self, start: u64) -> u64 {
        let mut current = start;
        
        while current < self.bitmap_blocks * 8 {
            if self.is_block_free(current) {
                return current;
            }
            current += 1;
        }
        
        current // Will be out of bounds if no free blocks
    }
}

/// MFS directory implementation
pub struct MfsDirectory {
    entries: Vec<MfsDirEntry>,
    inode_table: Vec<MfsInode>,
}

impl MfsDirectory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            inode_table: Vec::new(),
        }
    }
    
    /// Add a directory entry
    pub fn add_entry(&mut self, inode: u32, name: &str, file_type: FileType) -> FsResult<()> {
        if name.len() > 255 {
            return Err(FsError::InvalidPath);
        }
        
        // Check for duplicates
        if self.find_entry(name).is_some() {
            return Err(FsError::AlreadyExists);
        }
        
        let mut entry = MfsDirEntry {
            inode,
            entry_length: (size_of::<MfsDirEntry>() as u16) + name.len() as u16,
            name_length: name.len() as u8,
            file_type: file_type as u8,
            name: [0; 255],
        };
        
        // Copy name into entry
        let name_bytes = name.as_bytes();
        entry.name[..name_bytes.len()].copy_from_slice(name_bytes);
        
        self.entries.push(entry);
        Ok(())
    }
    
    /// Find a directory entry by name
    pub fn find_entry(&self, name: &str) -> Option<&MfsDirEntry> {
        self.entries.iter().find(|entry| {
            &entry.name[..entry.name_length as usize] == name.as_bytes()
        })
    }
    
    /// Remove a directory entry
    pub fn remove_entry(&mut self, name: &str) -> FsResult<u32> {
        if let Some(index) = self.entries.iter().position(|entry| {
            &entry.name[..entry.name_length as usize] == name.as_bytes()
        }) {
            let inode = self.entries[index].inode;
            self.entries.remove(index);
            Ok(inode)
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// List all entries
    pub fn list_entries(&self) -> &[MfsDirEntry] {
        &self.entries
    }
}

/// Journal implementation for MFS
pub struct MfsJournal {
    entries: Vec<MfsJournalEntry>,
    current_sequence: u64,
    max_entries: usize,
}

impl MfsJournal {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            current_sequence: 0,
            max_entries,
        }
    }
    
    /// Start a journal transaction
    pub fn start_transaction(&mut self, block: u64) -> FsResult<u64> {
        self.current_sequence += 1;
        
        let entry = MfsJournalEntry {
            sequence: self.current_sequence,
            block,
            timestamp: 0, // Would use actual time in real implementation
            data: [0; MFS_BLOCK_SIZE as usize],
        };
        
        self.entries.push(entry);
        
        if self.entries.len() > self.max_entries {
            // In production, this would trigger commit/rollback
            info!("Journal full, cleaning old entries");
            self.entries.drain(0..self.entries.len() / 2);
        }
        
        Ok(self.current_sequence)
    }
    
    /// Commit a journal transaction
    pub fn commit(&mut self, sequence: u64) -> FsResult<()> {
        // In production, this would write the data to disk
        info!("Journal entry {} committed", sequence);
        Ok(())
    }
    
    /// Rollback a journal transaction
    pub fn rollback(&mut self, sequence: u64) -> FsResult<()> {
        // Remove the journal entry and restore previous state
        if let Some(index) = self.entries.iter().position(|entry| entry.sequence == sequence) {
            self.entries.remove(index);
        }
        Ok(())
    }
    
    /// Get journal statistics
    pub fn get_stats(&self) -> (u64, usize, usize) {
        (self.current_sequence, self.entries.len(), self.max_entries)
    }
}

/// Security manager for MFS
pub struct MfsSecurityManager {
    default_permissions: u32,
    security_mode: MfsSecurityMode,
    audit_enabled: bool,
}

impl MfsSecurityManager {
    pub fn new() -> Self {
        Self {
            default_permissions: 0o755, // Standard Unix permissions
            security_mode: MfsSecurityMode::Standard,
            audit_enabled: false,
        }
    }
    
    /// Check if a user has permission to access a file
    pub fn check_permission(&self, uid: u16, gid: u16, permissions: u32, 
                           operation: MfsOperation) -> bool {
        // Simplified permission check
        // In production, this would be more sophisticated
        let user_part = (permissions >> 6) & 0o7;
        let group_part = (permissions >> 3) & 0o7;
        let other_part = permissions & 0o7;
        
        match operation {
            MfsOperation::Read => (user_part & 0o4) != 0 || (group_part & 0o4) != 0,
            MfsOperation::Write => (user_part & 0o2) != 0 || (group_part & 0o2) != 0,
            MfsOperation::Execute => (user_part & 0o1) != 0 || (group_part & 0o1) != 0,
        }
    }
    
    /// Create security attributes for a file
    pub fn create_security_attr(&self, uid: u16, gid: u16, mode: u32) -> MfsSecurityAttr {
        MfsSecurityAttr {
            type_id: 1, // Security attribute type
            length: size_of::<MfsSecurityAttr>() as u16,
            permissions: mode,
            owner_uid: uid,
            group_gid: gid,
            security_mode: self.security_mode as u16,
            audit_id: 0,
            acl_entries: 0,
            reserved: [0; 11],
        }
    }
    
    /// Enable audit logging
    pub fn enable_audit(&mut self) {
        self.audit_enabled = true;
    }
    
    /// Disable audit logging
    pub fn disable_audit(&mut self) {
        self.audit_enabled = false;
    }
}

/// Security operation types
#[derive(Debug, Clone, Copy)]
pub enum MfsOperation {
    Read,
    Write,
    Execute,
}

/// Security modes
#[derive(Debug, Clone, Copy)]
pub enum MfsSecurityMode {
    Standard = 0,
    Enhanced = 1,
    Military = 2,
}

/// MFS main file system implementation
pub struct MfsFileSystem {
    superblock: MfsSuperblock,
    block_groups: Vec<MfsBlockGroup>,
    allocator: MfsIndexedAllocator,
    journal: MfsJournal,
    security_manager: MfsSecurityManager,
    directories: Vec<MfsDirectory>,
    mounted: bool,
}

impl MfsFileSystem {
    /// Create a new MFS instance
    pub fn new(total_blocks: u64) -> Self {
        let block_groups_count = ((total_blocks + MFS_BLOCKS_PER_GROUP - 1) / MFS_BLOCKS_PER_GROUP) as u32;
        
        let mut block_groups = Vec::new();
        for i in 0..block_groups_count {
            let group_start = i as u64 * MFS_BLOCKS_PER_GROUP;
            block_groups.push(MfsBlockGroup {
                block_bitmap: group_start + 1,
                inode_bitmap: group_start + 2,
                inode_table: group_start + 3,
                free_blocks_count: (MFS_BLOCKS_PER_GROUP - 4) as u16, // Reserve 4 blocks for metadata
                free_inodes_count: 1024, // Simplified
                used_dirs_count: 0,
                flags: MfsBlockGroupFlags::empty(),
                checksum: 0,
            });
        }
        
        Self {
            superblock: MfsSuperblock {
                magic: MFS_MAGIC,
                version: MFS_VERSION,
                block_size: MFS_BLOCK_SIZE as u32,
                block_count: total_blocks,
                free_blocks: total_blocks - (block_groups_count as u64 * 4),
                block_group_count: block_groups_count,
                inode_count: block_groups_count as u64 * 1024,
                free_inodes: block_groups_count as u64 * 1024,
                first_data_block: block_groups_count as u64,
                journal_block: 1,
                journal_size: MFS_JOURNAL_SIZE as u32,
                features: MfsFeatures::JOURNALING | MfsFeatures::INDEXING | MfsFeatures::SECURITY | MfsFeatures::LARGE_FILES,
                create_time: 0, // Would use actual time
                last_mount_time: 0,
                mount_count: 0,
                max_mount_count: 30,
                state: MfsState::CLEAN,
                error_handling: MfsErrorHandling::REMOUNT_READ_ONLY,
                revision_level: 1,
                reserved_blocks_percentage: 5,
                block_allocation_hint: 0,
                encryption_key: 0,
                checksum: 0,
            },
            block_groups,
            allocator: MfsIndexedAllocator::new(total_blocks),
            journal: MfsJournal::new(10000),
            security_manager: MfsSecurityManager::new(),
            directories: Vec::new(),
            mounted: false,
        }
    }
    
    /// Mount the file system
    pub fn mount(&mut self) -> FsResult<()> {
        if self.mounted {
            return Err(FsError::AlreadyExists);
        }
        
        info!("Mounting MFS with {} blocks", self.superblock.block_count);
        self.mounted = true;
        self.superblock.mount_count += 1;
        self.superblock.last_mount_time = 0; // Would use actual time
        
        // Create root directory
        self.directories.push(MfsDirectory::new());
        
        Ok(())
    }
    
    /// Unmount the file system
    pub fn unmount(&mut self) -> FsResult<()> {
        if !self.mounted {
            return Err(FsError::NotFound);
        }
        
        info!("Unmounting MFS");
        self.mounted = false;
        
        // Commit any pending journal entries
        self.journal.commit(self.journal.current_sequence)?;
        
        Ok(())
    }
    
    /// Create a new file
    pub fn create_file(&mut self, name: &str, uid: u16, gid: u16, mode: u32) -> FsResult<u32> {
        if !self.mounted {
            return Err(FsError::IoError);
        }
        
        // Check permissions
        if !self.security_manager.check_permission(uid, gid, mode, MfsOperation::Write) {
            return Err(FsError::PermissionDenied);
        }
        
        // Allocate inode
        let inode_num = self.allocate_inode()?;
        
        // Create inode
        let inode = MfsInode {
            mode: mode | (FileType::Regular as u16),
            uid,
            size: 0,
            access_time: 0,
            create_time: 0,
            modify_time: 0,
            delete_time: 0,
            gid,
            links_count: 1,
            blocks: 0,
            flags: MfsInodeFlags::empty(),
            version: 1,
            blocks_high: 0,
            fragment_address: 0,
            direct_blocks: [0; 12],
            single_indirect: 0,
            double_indirect: 0,
            triple_indirect: 0,
            generation: 0,
            extended_attributes: 0,
            access_acl: 0,
            default_acl: 0,
            update_time: 0,
            encryption_key_id: 0,
            reserved: 0,
            checksum: 0,
        };
        
        // Add to root directory
        self.directories[0].add_entry(inode_num, name, FileType::Regular)?;
        
        Ok(inode_num)
    }
    
    /// Write data to a file
    pub fn write_file(&mut self, inode_num: u32, data: &[u8], offset: u64) -> FsResult<usize> {
        if !self.mounted {
            return Err(FsError::IoError);
        }
        
        let mut written = 0;
        let mut current_offset = offset;
        let mut remaining = data.len() as u64;
        let mut data_slice = data;
        
        while remaining > 0 {
            let block_size = MFS_BLOCK_SIZE;
            let bytes_to_write = min(remaining, block_size - (current_offset % block_size));
            
            // Calculate which blocks we need
            let start_block = current_offset / block_size;
            let blocks_needed = ((current_offset % block_size) + bytes_to_write + block_size - 1) / block_size;
            
            // Allocate blocks if needed
            let allocated_blocks = self.allocator.allocate_blocks(blocks_needed as u32, None)?;
            
            // Start journal transaction
            let sequence = self.journal.start_transaction(allocated_blocks[0])?;
            
            // Write data (simplified - would actually write to disk)
            written += bytes_to_write as usize;
            current_offset += bytes_to_write;
            remaining -= bytes_to_write;
            
            // Commit journal entry
            self.journal.commit(sequence)?;
            
            data_slice = &data_slice[bytes_to_write as usize..];
        }
        
        Ok(written)
    }
    
    /// Read data from a file
    pub fn read_file(&self, inode_num: u32, size: u64, offset: u64) -> FsResult<Vec<u8>> {
        if !self.mounted {
            return Err(FsError::IoError);
        }
        
        // Simplified read - would actually read from disk blocks
        let mut data = vec![0u8; size as usize];
        
        // In production, this would:
        // 1. Get file inode
        // 2. Calculate which blocks contain the requested data
        // 3. Read from disk blocks into buffer
        
        Ok(data)
    }
    
    /// Create a directory
    pub fn create_directory(&mut self, name: &str, uid: u16, gid: u16, mode: u32) -> FsResult<()> {
        if !self.mounted {
            return Err(FsError::IoError);
        }
        
        // Check permissions
        if !self.security_manager.check_permission(uid, gid, mode, MfsOperation::Write) {
            return Err(FsError::PermissionDenied);
        }
        
        // Allocate inode
        let inode_num = self.allocate_inode()?;
        
        // Create directory entry in parent
        self.directories[0].add_entry(inode_num, name, FileType::Directory)?;
        
        // Create new directory
        self.directories.push(MfsDirectory::new());
        
        Ok(())
    }
    
    /// List directory contents
    pub fn list_directory(&self, path: &str) -> FsResult<Vec<&MfsDirEntry>> {
        if !self.mounted {
            return Err(FsError::IoError);
        }
        
        // Simplified - only root directory for now
        if path == "/" || path.is_empty() {
            Ok(self.directories[0].list_entries().to_vec())
        } else {
            Err(FsError::NotFound)
        }
    }
    
    /// Delete a file or directory
    pub fn delete(&mut self, name: &str) -> FsResult<()> {
        if !self.mounted {
            return Err(FsError::IoError);
        }
        
        let inode_num = self.directories[0].remove_entry(name)?;
        
        // Deallocate blocks if it's a file
        // In production, this would be more complex
        
        Ok(())
    }
    
    /// Get file system statistics
    pub fn get_stats(&self) -> MfsFileSystemStats {
        MfsFileSystemStats {
            total_blocks: self.superblock.block_count,
            free_blocks: self.allocator.free_blocks,
            total_inodes: self.superblock.inode_count,
            free_inodes: self.superblock.free_inodes,
            block_size: self.superblock.block_size,
            journal_entries: self.journal.get_stats().1,
            mounted: self.mounted,
            mount_count: self.superblock.mount_count,
        }
    }
    
    /// Enable journaling
    pub fn enable_journaling(&mut self) -> FsResult<()> {
        if self.mounted {
            return Err(FsError::UnsupportedOperation);
        }
        
        self.superblock.features |= MfsFeatures::JOURNALING;
        Ok(())
    }
    
    /// Enable security features
    pub fn enable_security(&mut self) -> FsResult<()> {
        if self.mounted {
            return Err(FsError::UnsupportedOperation);
        }
        
        self.superblock.features |= MfsFeatures::SECURITY;
        self.security_manager.enable_audit();
        Ok(())
    }
    
    /// Allocate an inode (simplified)
    fn allocate_inode(&mut self) -> FsResult<u32> {
        if self.superblock.free_inodes == 0 {
            return Err(FsError::DiskFull);
        }
        
        self.superblock.free_inodes -= 1;
        Ok(1) // Simplified - would allocate actual inode number
    }
}

/// File system statistics
#[derive(Debug, Clone, Copy)]
pub struct MfsFileSystemStats {
    pub total_blocks: u64,
    pub free_blocks: u64,
    pub total_inodes: u64,
    pub free_inodes: u64,
    pub block_size: u32,
    pub journal_entries: usize,
    pub mounted: bool,
    pub mount_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superblock_creation() {
        let fs = MfsFileSystem::new(1024 * 1024); // 1GB
        assert_eq!(fs.superblock.magic, MFS_MAGIC);
        assert_eq!(fs.superblock.version, MFS_VERSION);
        assert!(fs.superblock.features.contains(MfsFeatures::JOURNALING));
        assert!(fs.superblock.features.contains(MfsFeatures::INDEXING));
        assert!(fs.superblock.features.contains(MfsFeatures::SECURITY));
    }

    #[test]
    fn test_mount_unmount() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        assert!(!fs.mounted);
        
        assert!(fs.mount().is_ok());
        assert!(fs.mounted);
        assert_eq!(fs.superblock.mount_count, 1);
        
        assert!(fs.unmount().is_ok());
        assert!(!fs.mounted);
    }

    #[test]
    fn test_file_creation() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        let inode = fs.create_file("test.txt", 1000, 1000, 0o644).unwrap();
        assert!(inode > 0);
    }

    #[test]
    fn test_directory_operations() {
        let mut fs = MfsFileSystem::new(1024 * 1024);
        fs.mount().unwrap();
        
        fs.create_directory("testdir", 1000, 1000, 0o755).unwrap();
        
        let entries = fs.list_directory("/").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_type, FileType::Directory as u8);
    }

    #[test]
    fn test_journal_operations() {
        let mut journal = MfsJournal::new(100);
        let sequence = journal.start_transaction(100).unwrap();
        assert!(journal.commit(sequence).is_ok());
        
        let stats = journal.get_stats();
        assert_eq!(stats.0, 1); // current_sequence
        assert_eq!(stats.1, 1); // entries.len()
    }

    #[test]
    fn test_security_permissions() {
        let security = MfsSecurityManager::new();
        let permissions = 0o644;
        
        assert!(security.check_permission(1000, 1000, permissions, MfsOperation::Read));
        assert!(security.check_permission(1000, 1000, permissions, MfsOperation::Write));
        assert!(security.check_permission(1000, 1000, permissions, MfsOperation::Execute));
    }

    #[test]
    fn test_allocator_block_allocation() {
        let mut allocator = MfsIndexedAllocator::new(1024);
        
        let blocks = allocator.allocate_blocks(10, None).unwrap();
        assert_eq!(blocks.len(), 10);
        
        let result = allocator.deallocate_blocks(&blocks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_system_stats() {
        let fs = MfsFileSystem::new(1024 * 1024);
        let stats = fs.get_stats();
        
        assert_eq!(stats.total_blocks, 1024 * 1024);
        assert!(stats.free_blocks > 0);
        assert!(stats.block_size == MFS_BLOCK_SIZE as u32);
        assert!(!stats.mounted);
    }
}