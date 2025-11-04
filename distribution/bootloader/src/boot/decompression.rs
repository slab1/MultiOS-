//! Kernel Decompression Support
//! 
//! This module provides functionality to decompress compressed kernel images.
//! It supports simple compression formats commonly used in bootloaders.

use core::mem;
use core::slice;

const COMPRESSED_KERNEL_MAGIC: u32 = 0x4B5A4D43; // "CZM K" reversed magic
const UNCOMPRESSED_KERNEL_MAGIC: u32 = 0x4B554D43; // "CMU K" reversed magic

#[repr(C, packed)]
struct CompressedKernelHeader {
    magic: u32,
    compression_type: u32,
    original_size: u64,
    compressed_size: u64,
    load_address: u64,
    entry_point: u64,
    flags: u32,
    _reserved: u32,
}

#[repr(C, packed)]
struct SimpleCompressionHeader {
    magic: u32,
    compressed_size: u32,
    original_size: u32,
    algorithm: u16,
    _reserved: u16,
}

/// Compression algorithms supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CompressionAlgorithm {
    Uncompressed = 0,
    SimpleRLE = 1,
    Deflate = 2,    // Placeholder for future support
    LZ4 = 3,        // Placeholder for future support
}

bitflags! {
    /// Kernel image flags
    #[repr(C)]
    pub struct KernelImageFlags: u32 {
        const COMPRESSED = 1 << 0;
        const ENCRYPTED = 1 << 1;
        const CHECKSUMMED = 1 << 2;
        const PAGE_ALIGNED = 1 << 3;
        const RELOCATABLE = 1 << 4;
    }
}

/// Kernel image information
#[derive(Debug, Clone, Copy)]
pub struct KernelImageInfo {
    pub load_address: u64,
    pub entry_point: u64,
    pub size: u64,
    pub is_compressed: bool,
    pub compression_type: CompressionAlgorithm,
    pub flags: KernelImageFlags,
}

/// Decompression result
pub type DecompressionResult<T> = Result<T, DecompressionError>;

/// Decompression errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecompressionError {
    InvalidMagic,
    UnsupportedCompression,
    DecompressionFailed,
    MemoryInsufficient,
    ChecksumMismatch,
    InvalidHeader,
}

/// Decompressor trait for different algorithms
trait Decompressor {
    fn decompress(&self, compressed_data: &[u8], output: &mut [u8]) -> Result<usize, DecompressionError>;
}

/// Simple Run-Length Encoding decompressor
struct SimpleRLEDecompressor {
    // This would contain any RLE-specific state if needed
}

impl SimpleRLEDecompressor {
    const fn new() -> Self {
        Self {}
    }
}

impl Decompressor for SimpleRLEDecompressor {
    fn decompress(&self, compressed_data: &[u8], output: &mut [u8]) -> Result<usize, DecompressionError> {
        let mut input_pos = 0;
        let mut output_pos = 0;
        
        while input_pos < compressed_data.len() && output_pos < output.len() {
            let run_length = compressed_data[input_pos] as usize + 1;
            input_pos += 1;
            
            if input_pos >= compressed_data.len() {
                return Err(DecompressionError::InvalidHeader);
            }
            
            let byte_value = compressed_data[input_pos];
            input_pos += 1;
            
            if output_pos + run_length > output.len() {
                return Err(DecompressionError::MemoryInsufficient);
            }
            
            // Fill the run
            for i in 0..run_length {
                output[output_pos + i] = byte_value;
            }
            
            output_pos += run_length;
        }
        
        if input_pos != compressed_data.len() {
            return Err(DecompressionError::DecompressionFailed);
        }
        
        Ok(output_pos)
    }
}

/// LZ4-style decompressor (simplified version)
struct LZ4Decompressor {
    // This would contain LZ4-specific state if needed
}

impl LZ4Decompressor {
    const fn new() -> Self {
        Self {}
    }
}

impl Decompressor for LZ4Decompressor {
    fn decompress(&self, compressed_data: &[u8], output: &mut [u8]) -> Result<usize, DecompressionError> {
        // This is a simplified LZ4 implementation
        // In a real implementation, this would be more sophisticated
        
        let mut input_pos = 0;
        let mut output_pos = 0;
        
        while input_pos < compressed_data.len() && output_pos < output.len() {
            // Read token
            let token = compressed_data[input_pos];
            input_pos += 1;
            
            // Parse literal length
            let mut literal_length = (token & 0x0F) as usize;
            if literal_length == 15 {
                // Read additional length bytes
                while input_pos < compressed_data.len() {
                    let length_byte = compressed_data[input_pos];
                    input_pos += 1;
                    literal_length += length_byte as usize;
                    if length_byte != 0xFF {
                        break;
                    }
                }
            }
            
            // Copy literals
            if input_pos + literal_length > compressed_data.len() || output_pos + literal_length > output.len() {
                return Err(DecompressionError::DecompressionFailed);
            }
            
            output[output_pos..output_pos + literal_length]
                .copy_from_slice(&compressed_data[input_pos..input_pos + literal_length]);
            
            input_pos += literal_length;
            output_pos += literal_length;
            
            if input_pos >= compressed_data.len() {
                break;
            }
            
            // Read match offset
            if input_pos + 2 > compressed_data.len() {
                break;
            }
            
            let offset = (compressed_data[input_pos] as usize) | 
                        ((compressed_data[input_pos + 1] as usize) << 8);
            input_pos += 2;
            
            if offset == 0 {
                break;
            }
            
            // Parse match length
            let mut match_length = (token >> 4) as usize;
            if match_length == 15 {
                // Read additional length bytes
                while input_pos < compressed_data.len() {
                    let length_byte = compressed_data[input_pos];
                    input_pos += 1;
                    match_length += length_byte as usize;
                    if length_byte != 0xFF {
                        break;
                    }
                }
            }
            match_length += 4; // Minimum match length
            
            // Copy match from already decompressed data
            if output_pos < offset || output_pos - offset < match_length {
                return Err(DecompressionError::DecompressionFailed);
            }
            
            for i in 0..match_length {
                if output_pos + i >= output.len() {
                    return Err(DecompressionError::MemoryInsufficient);
                }
                output[output_pos + i] = output[output_pos + i - offset];
            }
            
            output_pos += match_length;
        }
        
        Ok(output_pos)
    }
}

/// Main kernel decompressor
pub struct KernelDecompressor;

impl KernelDecompressor {
    /// Create new kernel decompressor
    pub const fn new() -> Self {
        Self {}
    }

    /// Load and decompress kernel image
    pub fn load_kernel_image(kernel_data: &[u8]) -> DecompressionResult<KernelImageInfo> {
        if kernel_data.len() < mem::size_of::<CompressedKernelHeader>() {
            return Err(DecompressionError::InvalidHeader);
        }

        let header = unsafe {
            &*(kernel_data.as_ptr() as *const CompressedKernelHeader)
        };

        // Check magic number
        if header.magic != COMPRESSED_KERNEL_MAGIC && header.magic != UNCOMPRESSED_KERNEL_MAGIC {
            return Err(DecompressionError::InvalidMagic);
        }

        let is_compressed = header.magic == COMPRESSED_KERNEL_MAGIC;
        let compression_type = if is_compressed {
            CompressionAlgorithm::from(header.compression_type)
        } else {
            CompressionAlgorithm::Uncompressed
        };

        // Create kernel image info
        let info = KernelImageInfo {
            load_address: header.load_address,
            entry_point: header.entry_point,
            size: header.original_size,
            is_compressed,
            compression_type,
            flags: KernelImageFlags::from_bits_truncate(header.flags),
        };

        // If not compressed, just validate and return
        if !is_compressed {
            return Ok(info);
        }

        // Validate compression type
        if !matches!(compression_type, CompressionAlgorithm::SimpleRLE | CompressionAlgorithm::LZ4) {
            return Err(DecompressionError::UnsupportedCompression);
        }

        // Validate sizes
        if header.compressed_size > kernel_data.len() as u64 - mem::size_of::<CompressedKernelHeader>() as u64 {
            return Err(DecompressionError::InvalidHeader);
        }

        Ok(info)
    }

    /// Decompress kernel image to destination buffer
    pub fn decompress_kernel(
        &self,
        kernel_data: &[u8],
        output_buffer: &mut [u8],
    ) -> DecompressionResult<usize> {
        if kernel_data.len() < mem::size_of::<CompressedKernelHeader>() {
            return Err(DecompressionError::InvalidHeader);
        }

        let header = unsafe {
            &*(kernel_data.as_ptr() as *const CompressedKernelHeader)
        };

        // Check magic number
        if header.magic != COMPRESSED_KERNEL_MAGIC {
            if header.magic == UNCOMPRESSED_KERNEL_MAGIC {
                // Uncompressed kernel, just copy
                if output_buffer.len() < header.original_size as usize {
                    return Err(DecompressionError::MemoryInsufficient);
                }
                
                let kernel_payload = &kernel_data[mem::size_of::<CompressedKernelHeader>()..];
                output_buffer[..kernel_payload.len()].copy_from_slice(kernel_payload);
                return Ok(kernel_payload.len());
            }
            return Err(DecompressionError::InvalidMagic);
        }

        let compressed_data = &kernel_data[mem::size_of::<CompressedKernelHeader>()..];
        
        // Select appropriate decompressor
        let decompressor: Box<dyn Decompressor> = match CompressionAlgorithm::from(header.compression_type) {
            CompressionAlgorithm::SimpleRLE => Box::new(SimpleRLEDecompressor::new()),
            CompressionAlgorithm::LZ4 => Box::new(LZ4Decompressor::new()),
            CompressionAlgorithm::Uncompressed => return Err(DecompressionError::UnsupportedCompression),
            _ => return Err(DecompressionError::UnsupportedCompression),
        };

        // Decompress
        let decompressed_size = decompressor.decompress(compressed_data, output_buffer)?;

        // Validate decompressed size
        if decompressed_size != header.original_size as usize {
            return Err(DecompressionError::DecompressionFailed);
        }

        Ok(decompressed_size)
    }

    /// Calculate checksum for integrity verification
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        let mut checksum: u32 = 0;
        
        for &byte in data {
            checksum = checksum.wrapping_add(byte as u32);
        }
        
        checksum
    }

    /// Verify integrity using checksum
    pub fn verify_checksum(data: &[u8], expected_checksum: u32) -> bool {
        Self::calculate_checksum(data) == expected_checksum
    }

    /// Get optimal buffer size for decompression
    pub fn get_decompression_buffer_size(kernel_data: &[u8]) -> Result<usize, DecompressionError> {
        if kernel_data.len() < mem::size_of::<CompressedKernelHeader>() {
            return Err(DecompressionError::InvalidHeader);
        }

        let header = unsafe {
            &*(kernel_data.as_ptr() as *const CompressedKernelHeader)
        };

        Ok(header.original_size as usize)
    }
}

/// Extension methods for CompressionAlgorithm
trait CompressionAlgorithmExt {
    fn is_supported(&self) -> bool;
    fn get_decompressor(&self) -> Option<Box<dyn Decompressor>>;
}

impl CompressionAlgorithmExt for CompressionAlgorithm {
    fn is_supported(&self) -> bool {
        matches!(self, 
            CompressionAlgorithm::Uncompressed | 
            CompressionAlgorithm::SimpleRLE | 
            CompressionAlgorithm::LZ4)
    }

    fn get_decompressor(&self) -> Option<Box<dyn Decompressor>> {
        match self {
            CompressionAlgorithm::Uncompressed => None,
            CompressionAlgorithm::SimpleRLE => Some(Box::new(SimpleRLEDecompressor::new())),
            CompressionAlgorithm::LZ4 => Some(Box::new(LZ4Decompressor::new())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rle_decompression() {
        // Create compressed data: [1, 0xAA, 3, 0xBB, 5, 0xCC]
        // Decompresses to: [0xAA, 0xBB, 0xBB, 0xBB, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC]
        let compressed = vec![1, 0xAA, 3, 0xBB, 5, 0xCC];
        let mut output = vec![0u8; 9];
        
        let decompressor = SimpleRLEDecompressor::new();
        let result = decompressor.decompress(&compressed, &mut output);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 9);
        assert_eq!(output, vec![0xAA, 0xBB, 0xBB, 0xBB, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC]);
    }

    #[test]
    fn test_kernel_decompressor_creation() {
        let decompressor = KernelDecompressor::new();
        // Basic creation test
        assert!(true);
    }

    #[test]
    fn test_checksum_calculation() {
        let data = vec![1, 2, 3, 4, 5];
        let checksum = KernelDecompressor::calculate_checksum(&data);
        assert_eq!(checksum, 15); // 1+2+3+4+5 = 15
    }

    #[test]
    fn test_checksum_verification() {
        let data = vec![1, 2, 3, 4, 5];
        let checksum = KernelDecompressor::calculate_checksum(&data);
        assert!(KernelDecompressor::verify_checksum(&data, checksum));
        assert!(!KernelDecompressor::verify_checksum(&data, checksum + 1));
    }

    #[test]
    fn test_compression_algorithm_conversion() {
        assert_eq!(CompressionAlgorithm::from(0), CompressionAlgorithm::Uncompressed);
        assert_eq!(CompressionAlgorithm::from(1), CompressionAlgorithm::SimpleRLE);
        assert_eq!(CompressionAlgorithm::from(2), CompressionAlgorithm::Deflate);
    }
}