//! CPU Architecture Definitions and Support
//!
//! This module defines the core architecture types and characteristics
//! for supported CPU architectures in the MultiOS framework.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    ARM64,
    RISC_V64,
    SPARC64,
    PowerPC64,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::ARM64 => write!(f, "arm64"),
            Architecture::RISC_V64 => write!(f, "riscv64"),
            Architecture::SPARC64 => write!(f, "sparc64"),
            Architecture::PowerPC64 => write!(f, "powerpc64"),
        }
    }
}

impl std::str::FromStr for Architecture {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x86_64" | "x86-64" | "amd64" => Ok(Architecture::X86_64),
            "arm64" | "aarch64" => Ok(Architecture::ARM64),
            "riscv64" | "risc-v64" => Ok(Architecture::RISC_V64),
            "sparc64" | "sparc-v9" => Ok(Architecture::SPARC64),
            "powerpc64" | "ppc64" | "power9" => Ok(Architecture::PowerPC64),
            _ => Err(format!("Unknown architecture: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureSpec {
    pub name: Architecture,
    pub word_size: u32,
    pub endianness: Endianness,
    pub register_count: u32,
    pub isa_version: String,
    pub features: Vec<String>,
    pub cache_info: CacheConfig,
    pub pipeline_info: PipelineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Endianness {
    Little,
    Big,
    Bi,
}

impl fmt::Display for Endianness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Endianness::Little => write!(f, "Little"),
            Endianness::Big => write!(f, "Big"),
            Endianness::Bi => write!(f, "Bi"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub l1_instruction_size: u32,
    pub l1_instruction_associativity: u32,
    pub l1_data_size: u32,
    pub l1_data_associativity: u32,
    pub l2_size: u32,
    pub l2_associativity: u32,
    pub l3_size: u32,
    pub l3_associativity: u32,
    pub tlb_size: u32,
    pub tlb_associativity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub stages: u32,
    pub branch_prediction: BranchPredictor,
    pub out_of_order: bool,
    pub speculation: bool,
    pub pipeline_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchPredictor {
    Static,
    Dynamic2Bit,
    GShare,
    TAGE,
    Tournament,
    Custom(String),
}

impl ArchitectureSpec {
    /// Get the specification for a given architecture
    pub fn get(arch: &Architecture) -> Self {
        match arch {
            Architecture::X86_64 => Self::x86_64_spec(),
            Architecture::ARM64 => Self::arm64_spec(),
            Architecture::RISC_V64 => Self::riscv64_spec(),
            Architecture::SPARC64 => Self::sparc64_spec(),
            Architecture::PowerPC64 => Self::powerpc64_spec(),
        }
    }

    fn x86_64_spec() -> Self {
        Self {
            name: Architecture::X86_64,
            word_size: 64,
            endianness: Endianness::Little,
            register_count: 16, // GPRs
            isa_version: "3.1".to_string(),
            features: vec![
                "SSE2".to_string(),
                "AVX2".to_string(),
                "AVX-512".to_string(),
                "BMI2".to_string(),
                "AES-NI".to_string(),
            ],
            cache_info: CacheConfig {
                l1_instruction_size: 32 * 1024,
                l1_instruction_associativity: 8,
                l1_data_size: 32 * 1024,
                l1_data_associativity: 8,
                l2_size: 256 * 1024,
                l2_associativity: 8,
                l3_size: 8 * 1024 * 1024,
                l3_associativity: 16,
                tlb_size: 64,
                tlb_associativity: 4,
            },
            pipeline_info: PipelineConfig {
                stages: 14,
                branch_prediction: BranchPredictor::Tournament,
                out_of_order: true,
                speculation: true,
                pipeline_width: 4,
            },
        }
    }

    fn arm64_spec() -> Self {
        Self {
            name: Architecture::ARM64,
            word_size: 64,
            endianness: Endianness::Little,
            register_count: 31, // GPRs
            isa_version: "v8.3".to_string(),
            features: vec![
                "NEON".to_string(),
                "AES".to_string(),
                "SHA1".to_string(),
                "SHA256".to_string(),
                "CRC32".to_string(),
                "RCPC".to_string(),
            ],
            cache_info: CacheConfig {
                l1_instruction_size: 64 * 1024,
                l1_instruction_associativity: 4,
                l1_data_size: 64 * 1024,
                l1_data_associativity: 4,
                l2_size: 512 * 1024,
                l2_associativity: 8,
                l3_size: 4 * 1024 * 1024,
                l3_associativity: 16,
                tlb_size: 48,
                tlb_associativity: 4,
            },
            pipeline_info: PipelineConfig {
                stages: 15,
                branch_prediction: BranchPredictor::TAGE,
                out_of_order: true,
                speculation: true,
                pipeline_width: 3,
            },
        }
    }

    fn riscv64_spec() -> Self {
        Self {
            name: Architecture::RISC_V64,
            word_size: 64,
            endianness: Endianness::Little,
            register_count: 32,
            isa_version: "v2.2".to_string(),
            features: vec![
                "M".to_string(),
                "A".to_string(),
                "F".to_string(),
                "D".to_string(),
                "C".to_string(),
                "Zicsr".to_string(),
            ],
            cache_info: CacheConfig {
                l1_instruction_size: 32 * 1024,
                l1_instruction_associativity: 4,
                l1_data_size: 32 * 1024,
                l1_data_associativity: 4,
                l2_size: 256 * 1024,
                l2_associativity: 8,
                l3_size: 2 * 1024 * 1024,
                l3_associativity: 8,
                tlb_size: 16,
                tlb_associativity: 4,
            },
            pipeline_info: PipelineConfig {
                stages: 5,
                branch_prediction: BranchPredictor::Dynamic2Bit,
                out_of_order: false,
                speculation: false,
                pipeline_width: 1,
            },
        }
    }

    fn sparc64_spec() -> Self {
        Self {
            name: Architecture::SPARC64,
            word_size: 64,
            endianness: Endianness::Big,
            register_count: 32,
            isa_version: "V9".to_string(),
            features: vec![
                "VIS1".to_string(),
                "VIS2".to_string(),
                "VIS3".to_string(),
                "Crypto".to_string(),
            ],
            cache_info: CacheConfig {
                l1_instruction_size: 64 * 1024,
                l1_instruction_associativity: 4,
                l1_data_size: 64 * 1024,
                l1_data_associativity: 4,
                l2_size: 1 * 1024 * 1024,
                l2_associativity: 8,
                l3_size: 8 * 1024 * 1024,
                l3_associativity: 16,
                tlb_size: 128,
                tlb_associativity: 8,
            },
            pipeline_info: PipelineConfig {
                stages: 14,
                branch_prediction: BranchPredictor::Dynamic2Bit,
                out_of_order: true,
                speculation: true,
                pipeline_width: 3,
            },
        }
    }

    fn powerpc64_spec() -> Self {
        Self {
            name: Architecture::PowerPC64,
            word_size: 64,
            endianness: Endianness::Big,
            register_count: 32,
            isa_version: "v3.0".to_string(),
            features: vec![
                "VMX".to_string(),
                "VSX".to_string(),
                "Crypto".to_string(),
                "VectorAES".to_string(),
            ],
            cache_info: CacheConfig {
                l1_instruction_size: 32 * 1024,
                l1_instruction_associativity: 8,
                l1_data_size: 32 * 1024,
                l1_data_associativity: 8,
                l2_size: 512 * 1024,
                l2_associativity: 8,
                l3_size: 8 * 1024 * 1024,
                l3_associativity: 16,
                tlb_size: 64,
                tlb_associativity: 4,
            },
            pipeline_info: PipelineConfig {
                stages: 15,
                branch_prediction: BranchPredictor::GShare,
                out_of_order: true,
                speculation: true,
                pipeline_width: 6,
            },
        }
    }

    /// Get all supported architectures
    pub fn all_architectures() -> Vec<Architecture> {
        vec![
            Architecture::X86_64,
            Architecture::ARM64,
            Architecture::RISC_V64,
            Architecture::SPARC64,
            Architecture::PowerPC64,
        ]
    }

    /// Check if a feature is supported by the architecture
    pub fn supports_feature(&self, feature: &str) -> bool {
        self.features.contains(&feature.to_string())
    }

    /// Get cache configuration for a specific level
    pub fn get_cache_config(&self, level: u32) -> Option<(u32, u32)> {
        match level {
            1 => Some((self.cache_info.l1_instruction_size, self.cache_info.l1_instruction_associativity)),
            2 => Some((self.cache_info.l2_size, self.cache_info.l2_associativity)),
            3 => Some((self.cache_info.l3_size, self.cache_info.l3_associativity)),
            _ => None,
        }
    }

    /// Get the memory access characteristics
    pub fn get_memory_characteristics(&self) -> HashMap<String, f64> {
        let mut characteristics = HashMap::new();
        
        // Base latency (in cycles)
        characteristics.insert("l1_latency".to_string(), 1.0);
        characteristics.insert("l2_latency".to_string(), 4.0);
        characteristics.insert("l3_latency".to_string(), 12.0);
        characteristics.insert("memory_latency".to_string(), 100.0);
        
        // Bandwidth (in GB/s)
        characteristics.insert("l1_bandwidth".to_string(), 1000.0);
        characteristics.insert("l2_bandwidth".to_string(), 500.0);
        characteristics.insert("l3_bandwidth".to_string(), 200.0);
        characteristics.insert("memory_bandwidth".to_string(), 50.0);
        
        characteristics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_from_str() {
        assert_eq!("x86_64".parse::<Architecture>().unwrap(), Architecture::X86_64);
        assert_eq!("arm64".parse::<Architecture>().unwrap(), Architecture::ARM64);
        assert_eq!("riscv64".parse::<Architecture>().unwrap(), Architecture::RISC_V64);
        assert_eq!("sparc64".parse::<Architecture>().unwrap(), Architecture::SPARC64);
        assert_eq!("powerpc64".parse::<Architecture>().unwrap(), Architecture::PowerPC64);
    }

    #[test]
    fn test_architecture_specification() {
        let spec = ArchitectureSpec::get(&Architecture::X86_64);
        assert_eq!(spec.name, Architecture::X86_64);
        assert_eq!(spec.word_size, 64);
        assert!(spec.supports_feature("SSE2"));
    }

    #[test]
    fn test_cache_configuration() {
        let spec = ArchitectureSpec::get(&Architecture::ARM64);
        assert_eq!(spec.get_cache_config(1).unwrap().0, 64 * 1024);
        assert_eq!(spec.get_cache_config(2).unwrap().1, 8);
    }
}