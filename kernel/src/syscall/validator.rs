//! MultiOS System Call Parameter Validator
//! 
//! This module provides comprehensive parameter validation for system calls,
//! including pointer validation, buffer bounds checking, and security checks.

use crate::log::{warn, error, debug};
use crate::arch::interrupts::*;
use crate::memory::{self, MemoryRegion};
use crate::ArchType;

type SyscallResult<T> = Result<T, SyscallError>;

/// Memory region for validation
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
}

/// System call parameter validator with comprehensive security checks
pub struct SyscallValidator {
    /// Maximum buffer size for validation
    pub max_buffer_size: usize,
    /// User space memory regions
    pub user_regions: Vec<MemoryRegion>,
    /// Kernel space memory regions
    pub kernel_regions: Vec<MemoryRegion>,
    /// Page size for alignment checks
    pub page_size: usize,
    /// Enable strict validation mode
    pub strict_mode: bool,
}

impl SyscallValidator {
    /// Create new system call parameter validator
    pub fn new() -> Self {
        Self {
            max_buffer_size: 16 * 1024 * 1024, // 16MB default
            user_regions: Vec::new(),
            kernel_regions: Vec::new(),
            page_size: 4096, // Standard page size
            strict_mode: true,
        }
    }

    /// Initialize validator with architecture-specific settings
    pub fn init_for_architecture(arch: ArchType) -> Self {
        let mut validator = Self::new();
        
        match arch {
            ArchType::X86_64 => {
                // x86_64 specific validation settings
                validator.page_size = 4096;
                validator.user_regions.push(MemoryRegion {
                    base: 0x00007FFF_FFFF_F000,
                    size: 0x00008000_0000_1000,
                });
            }
            ArchType::AArch64 => {
                // ARM64 specific validation settings
                validator.page_size = 4096;
                validator.user_regions.push(MemoryRegion {
                    base: 0x0000FFFF_FFFF_F000,
                    size: 0x00010000_0000_1000,
                });
            }
            ArchType::Riscv64 => {
                // RISC-V specific validation settings
                validator.page_size = 4096;
                validator.user_regions.push(MemoryRegion {
                    base: 0x00000000_7FFF_FFFF,
                    size: 0x00000000_8000_0000,
                });
            }
            _ => {
                // Unknown architecture, use conservative defaults
            }
        }
        
        // Add kernel regions
        validator.kernel_regions.push(MemoryRegion {
            base: 0xFFFF8000_0000_0000,
            size: 0x00007FFF_FFFF_FFFF,
        });
        
        validator
    }

    /// Validate all parameters for a system call
    pub fn validate_parameters(&self, params: &SystemCallParams) -> SyscallResult<()> {
        debug!("Validating parameters for syscall {}", params.syscall_number);
        
        // Validate each parameter based on its position and expected type
        self.validate_syscall_number(params.syscall_number)?;
        self.validate_integer(params.arg0)?;
        self.validate_integer(params.arg1)?;
        self.validate_integer(params.arg2)?;
        self.validate_integer(params.arg3)?;
        self.validate_integer(params.arg4)?;
        self.validate_integer(params.arg5)?;
        
        Ok(())
    }

    /// Validate system call number
    pub fn validate_syscall_number(&self, syscall_number: usize) -> SyscallResult<()> {
        if syscall_number >= 10000 {
            return Err(SyscallError::ValueOutOfRange);
        }
        Ok(())
    }

    /// Validate integer parameter
    pub fn validate_integer(&self, value: usize) -> SyscallResult<()> {
        // Basic integer validation - check for obviously invalid values
        if value == usize::MAX {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        if self.strict_mode {
            // Additional strict checks
            if value & 0x8000_0000_0000_0000 != 0 {
                // Check for negative values represented as unsigned
                return Err(SyscallError::ValueOutOfRange);
            }
        }
        
        Ok(())
    }

    /// Validate pointer and size combination
    pub fn validate_pointer(&self, ptr: usize, size: usize) -> SyscallResult<()> {
        // Check for null pointer with size > 0
        if ptr == 0 && size > 0 {
            return Err(SyscallError::InvalidPointer);
        }
        
        // Check for obviously invalid pointers
        if ptr == usize::MAX {
            return Err(SyscallError::InvalidPointer);
        }
        
        // Check buffer size limits
        if size > self.max_buffer_size {
            return Err(SyscallError::BufferTooSmall);
        }
        
        // Check pointer alignment
        if ptr & (self.page_size - 1) != 0 {
            debug!("Pointer {:#x} is not page aligned", ptr);
        }
        
        // Check if pointer is in allowed regions
        if self.is_pointer_in_allowed_regions(ptr, size) {
            Ok(())
        } else {
            Err(SyscallError::AddressSpaceViolation)
        }
    }

    /// Validate string pointer
    pub fn validate_string_ptr(&self, ptr: usize) -> SyscallResult<usize> {
        if ptr == 0 {
            return Err(SyscallError::InvalidPointer);
        }
        
        // Check that the pointer points to valid memory
        self.validate_pointer(ptr, 1)?;
        
        // Estimate string length (in real implementation, would scan for null terminator)
        // For now, assume reasonable string length
        let max_string_length = 1024;
        
        Ok(max_string_length)
    }

    /// Validate file descriptor
    pub fn validate_file_descriptor(&self, fd: usize) -> SyscallResult<usize> {
        if fd > 1024 {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        if fd == 0 || fd == 1 || fd == 2 {
            // Standard file descriptors (stdin, stdout, stderr)
            return Ok(fd);
        }
        
        // For other file descriptors, would validate against process file table
        // For now, accept reasonable values
        Ok(fd)
    }

    /// Validate buffer for read/write operations
    pub fn validate_buffer(&self, ptr: usize, size: usize, access_type: BufferAccessType) -> SyscallResult<ValidatedBuffer> {
        self.validate_pointer(ptr, size)?;
        
        // Check for buffer size limits based on access type
        match access_type {
            BufferAccessType::Read => {
                if size > self.max_buffer_size {
                    return Err(SyscallError::BufferTooSmall);
                }
            }
            BufferAccessType::Write => {
                if size > self.max_buffer_size {
                    return Err(SyscallError::BufferTooSmall);
                }
            }
            BufferAccessType::ReadWrite => {
                if size > self.max_buffer_size / 2 {
                    return Err(SyscallError::BufferTooSmall);
                }
            }
        }
        
        // Create validated buffer structure
        Ok(ValidatedBuffer {
            ptr,
            size,
            access_type,
            aligned_ptr: self.align_pointer(ptr),
        })
    }

    /// Validate memory mapping parameters
    pub fn validate_memory_mapping(&self, addr: usize, size: usize, protection: u32) -> SyscallResult<()> {
        if addr == 0 {
            return Err(SyscallError::InvalidPointer);
        }
        
        // Check size limits for memory mapping
        if size > 1024 * 1024 * 1024 { // 1GB limit
            return Err(SyscallError::ValueOutOfRange);
        }
        
        // Check protection flags
        if protection & !0x7 != 0 {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        // Check alignment
        if addr & (self.page_size - 1) != 0 {
            return Err(SyscallError::InvalidAddress);
        }
        
        Ok(())
    }

    /// Validate process/thread ID
    pub fn validate_process_id(&self, pid: usize) -> SyscallResult<usize> {
        if pid == 0 {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        if pid > 65536 {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        Ok(pid)
    }

    /// Validate thread ID
    pub fn validate_thread_id(&self, tid: usize) -> SyscallResult<usize> {
        if tid == 0 {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        if tid > 1048576 { // 1M threads limit
            return Err(SyscallError::ValueOutOfRange);
        }
        
        Ok(tid)
    }

    /// Validate time value
    pub fn validate_time_value(&self, time_val: usize) -> SyscallResult<usize> {
        // Check for reasonable time values (not negative, not too large)
        if time_val > 0x7FFF_FFFF_FFFF_FFFF {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        Ok(time_val)
    }

    /// Validate flags parameter
    pub fn validate_flags(&self, flags: usize, allowed_flags: u32) -> SyscallResult<usize> {
        if flags & !allowed_flags as usize != 0 {
            return Err(SyscallError::ValueOutOfRange);
        }
        
        Ok(flags)
    }

    /// Check if pointer is in allowed memory regions
    fn is_pointer_in_allowed_regions(&self, ptr: usize, size: usize) -> bool {
        let end_addr = ptr.saturating_add(size);
        
        // Check user space regions
        for region in &self.user_regions {
            if ptr >= region.base && end_addr <= region.base + region.size {
                return true;
            }
        }
        
        // Check kernel space regions (for kernel syscalls)
        for region in &self.kernel_regions {
            if ptr >= region.base && end_addr <= region.base + region.size {
                return true;
            }
        }
        
        false
    }

    /// Align pointer to page boundary
    fn align_pointer(&self, ptr: usize) -> usize {
        ptr & !(self.page_size - 1)
    }

    /// Add user space memory region
    pub fn add_user_region(&mut self, base: usize, size: usize) {
        self.user_regions.push(MemoryRegion { base, size });
    }

    /// Add kernel space memory region
    pub fn add_kernel_region(&mut self, base: usize, size: usize) {
        self.kernel_regions.push(MemoryRegion { base, size });
    }

    /// Clear all memory regions
    pub fn clear_regions(&mut self) {
        self.user_regions.clear();
        self.kernel_regions.clear();
    }

    /// Enable or disable strict validation mode
    pub fn set_strict_mode(&mut self, enabled: bool) {
        self.strict_mode = enabled;
    }

    /// Set maximum buffer size
    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.max_buffer_size = size;
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> ValidationStats {
        ValidationStats {
            user_regions: self.user_regions.len(),
            kernel_regions: self.kernel_regions.len(),
            max_buffer_size: self.max_buffer_size,
            page_size: self.page_size,
            strict_mode: self.strict_mode,
        }
    }
}

/// Buffer access types for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferAccessType {
    Read,
    Write,
    ReadWrite,
}

/// Validated buffer structure
#[derive(Debug, Clone, Copy)]
pub struct ValidatedBuffer {
    pub ptr: usize,
    pub size: usize,
    pub access_type: BufferAccessType,
    pub aligned_ptr: usize,
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub user_regions: usize,
    pub kernel_regions: usize,
    pub max_buffer_size: usize,
    pub page_size: usize,
    pub strict_mode: bool,
}

impl ValidationStats {
    pub fn new() -> Self {
        Self {
            user_regions: 0,
            kernel_regions: 0,
            max_buffer_size: 0,
            page_size: 0,
            strict_mode: false,
        }
    }
}

/// Extended parameter validation for specific system call types
pub struct AdvancedValidator {
    /// Basic validator
    base_validator: SyscallValidator,
    /// Additional security constraints
    security_constraints: SecurityConstraints,
    /// Resource limits
    resource_limits: ResourceLimits,
}

impl AdvancedValidator {
    pub fn new() -> Self {
        Self {
            base_validator: SyscallValidator::new(),
            security_constraints: SecurityConstraints::new(),
            resource_limits: ResourceLimits::new(),
        }
    }

    /// Validate file operation parameters
    pub fn validate_file_operation(&self, path_ptr: usize, flags: usize, mode: usize) -> SyscallResult<FileOperationParams> {
        // Validate path pointer
        self.base_validator.validate_string_ptr(path_ptr)?;
        
        // Validate flags (common file flags)
        self.base_validator.validate_flags(flags, 0x3FF)?;
        
        // Validate mode (file permissions)
        self.base_validator.validate_integer(mode)?;
        
        Ok(FileOperationParams {
            path_ptr,
            flags,
            mode,
        })
    }

    /// Validate memory operation parameters
    pub fn validate_memory_operation(&self, addr: usize, size: usize, protection: usize) -> SyscallResult<MemoryOperationParams> {
        // Validate memory mapping parameters
        self.base_validator.validate_memory_mapping(addr, size, protection as u32)?;
        
        Ok(MemoryOperationParams {
            addr,
            size,
            protection,
        })
    }

    /// Validate process creation parameters
    pub fn validate_process_creation(&self, entry_point: usize, stack_ptr: usize) -> SyscallResult<ProcessCreationParams> {
        // Validate entry point and stack pointer
        self.base_validator.validate_pointer(entry_point, 0)?;
        self.base_validator.validate_pointer(stack_ptr, 0)?;
        
        Ok(ProcessCreationParams {
            entry_point,
            stack_ptr,
        })
    }
}

/// Security constraints for parameter validation
#[derive(Debug)]
pub struct SecurityConstraints {
    pub max_file_size: usize,
    pub max_memory_mapping: usize,
    pub max_open_files: usize,
    pub allow_raw_syscalls: bool,
}

impl SecurityConstraints {
    pub fn new() -> Self {
        Self {
            max_file_size: 1024 * 1024 * 1024, // 1GB
            max_memory_mapping: 256 * 1024 * 1024, // 256MB
            max_open_files: 1024,
            allow_raw_syscalls: false, // Disabled by default for security
        }
    }
}

/// Resource limits for system calls
#[derive(Debug)]
pub struct ResourceLimits {
    pub max_cpu_time: u64,
    pub max_memory: usize,
    pub max_file_descriptors: usize,
    pub max_threads: usize,
}

impl ResourceLimits {
    pub fn new() -> Self {
        Self {
            max_cpu_time: 3600, // 1 hour
            max_memory: 1024 * 1024 * 1024, // 1GB
            max_file_descriptors: 1024,
            max_threads: 1024,
        }
    }
}

/// File operation parameters
#[derive(Debug, Clone, Copy)]
pub struct FileOperationParams {
    pub path_ptr: usize,
    pub flags: usize,
    pub mode: usize,
}

/// Memory operation parameters
#[derive(Debug, Clone, Copy)]
pub struct MemoryOperationParams {
    pub addr: usize,
    pub size: usize,
    pub protection: usize,
}

/// Process creation parameters
#[derive(Debug, Clone, Copy)]
pub struct ProcessCreationParams {
    pub entry_point: usize,
    pub stack_ptr: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = SyscallValidator::new();
        assert_eq!(validator.max_buffer_size, 16 * 1024 * 1024);
        assert_eq!(validator.page_size, 4096);
        assert!(validator.strict_mode);
    }

    #[test]
    fn test_integer_validation() {
        let validator = SyscallValidator::new();
        
        // Valid integers
        assert!(validator.validate_integer(0).is_ok());
        assert!(validator.validate_integer(1000).is_ok());
        assert!(validator.validate_integer(usize::MAX - 1).is_ok());
        
        // Invalid integers
        assert!(validator.validate_integer(usize::MAX).is_err());
    }

    #[test]
    fn test_pointer_validation() {
        let validator = SyscallValidator::new();
        
        // Valid pointers
        assert!(validator.validate_pointer(0x1000, 0).is_ok());
        assert!(validator.validate_pointer(0x1000, 100).is_ok());
        
        // Invalid pointers
        assert!(validator.validate_pointer(0, 100).is_err()); // Null pointer with size
        assert!(validator.validate_pointer(usize::MAX, 0).is_err());
    }

    #[test]
    fn test_string_pointer_validation() {
        let validator = SyscallValidator::new();
        
        // Valid string pointer
        assert!(validator.validate_string_ptr(0x1000).is_ok());
        
        // Invalid string pointer
        assert!(validator.validate_string_ptr(0).is_err());
    }

    #[test]
    fn test_file_descriptor_validation() {
        let validator = SyscallValidator::new();
        
        // Valid file descriptors
        assert!(validator.validate_file_descriptor(0).is_ok());
        assert!(validator.validate_file_descriptor(1).is_ok());
        assert!(validator.validate_file_descriptor(100).is_ok());
        
        // Invalid file descriptors
        assert!(validator.validate_file_descriptor(2000).is_err());
    }
}