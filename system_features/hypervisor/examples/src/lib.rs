//! Educational Virtualization Examples and Tutorials
//! 
//! Provides comprehensive examples and tutorials for learning virtualization
//! concepts using the MultiOS hypervisor system.

use crate::{VmId, VmConfig, VmFeatures, HypervisorError};
use crate::core::{Hypervisor, vm_config::{VmArchitecture, BootConfig, DeviceConfig, NetworkConfig, StorageConfig, SecurityConfig}};

/// Educational example identifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EducationalExample {
    /// Simple boot example
    SimpleBoot,
    /// Multi-OS comparison example
    MultiOSComparison,
    /// Nested virtualization example
    NestedVirtualization,
    /// Kernel development example
    KernelDevelopment,
    /// Device driver example
    DeviceDriverExample,
    /// Memory management example
    MemoryManagement,
    /// Network virtualization example
    NetworkVirtualization,
    /// Security isolation example
    SecurityIsolation,
    /// Performance analysis example
    PerformanceAnalysis,
    /// Teaching lab setup
    TeachingLab,
}

/// Difficulty level for educational examples
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Educational tutorial structure
#[derive(Debug, Clone)]
pub struct EducationalTutorial {
    pub id: EducationalExample,
    pub title: String,
    pub description: String,
    pub difficulty: DifficultyLevel,
    pub estimated_duration_minutes: u32,
    pub learning_objectives: Vec<String>,
    pub prerequisites: Vec<String>,
    pub vm_configs: Vec<VmConfig>,
    pub steps: Vec<TutorialStep>,
    pub resources: Vec<TutorialResource>,
}

/// Tutorial step
#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub step_number: usize,
    pub title: String,
    pub description: String,
    pub code_example: Option<String>,
    pub expected_output: Option<String>,
    pub verification_commands: Vec<String>,
    pub troubleshooting_tips: Vec<String>,
}

/// Tutorial resource
#[derive(Debug, Clone)]
pub struct TutorialResource {
    pub title: String,
    pub resource_type: ResourceType,
    pub url: Option<String>,
    pub description: String,
}

/// Resource types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceType {
    Documentation,
    Video,
    Interactive,
    Code,
    Dataset,
}

/// Educational example manager
pub struct EducationalManager {
    tutorials: Vec<EducationalTutorial>,
    current_tutorial: Option<EducationalExample>,
    completed_tutorials: Vec<EducationalExample>,
}

impl EducationalManager {
    /// Create a new educational manager
    pub fn new() -> Self {
        EducationalManager {
            tutorials: Vec::new(),
            current_tutorial: None,
            completed_tutorials: Vec::new(),
        }
    }
    
    /// Initialize with standard educational examples
    pub fn initialize_standard_examples(&mut self) -> Result<(), HypervisorError> {
        self.create_simple_boot_example()?;
        self.create_multi_os_comparison_example()?;
        self.create_nested_virtualization_example()?;
        self.create_kernel_development_example()?;
        self.create_device_driver_example()?;
        self.create_memory_management_example()?;
        self.create_teaching_lab_example()?;
        
        info!("Initialized {} educational examples", self.tutorials.len());
        Ok(())
    }
    
    /// Create simple boot example
    fn create_simple_boot_example(&mut self) -> Result<(), HypervisorError> {
        let vm_config = VmConfig {
            name: String::from("Simple Boot Demo"),
            vcpu_count: 1,
            memory_mb: 512,
            arch: VmArchitecture::X86_64,
            boot: BootConfig {
                boot_order: crate::core::vm_config::BootOrder::DiskFirst,
                kernel_path: Some(String::from("simple_kernel.bin")),
                initrd_path: None,
                kernel_args: String::from("console=ttyS0"),
                timeout_sec: 10,
            },
            devices: DeviceConfig::educational(),
            features: VmFeatures::EDUCATIONAL | VmFeatures::DEBUG,
            network: NetworkConfig::disabled(),
            storage: StorageConfig::minimal(),
            security: SecurityConfig::default(),
        };
        
        let tutorial = EducationalTutorial {
            id: EducationalExample::SimpleBoot,
            title: String::from("Simple Boot Example"),
            description: String::from("Learn basic VM creation and boot process"),
            difficulty: DifficultyLevel::Beginner,
            estimated_duration_minutes: 30,
            learning_objectives: vec![
                String::from("Understand VM creation process"),
                String::from("Learn boot sequence in virtualization"),
                String::from("Practice basic hypervisor management"),
            ],
            prerequisites: vec![
                String::from("Basic understanding of operating systems"),
                String::from("Knowledge of x86 architecture"),
            ],
            vm_configs: vec![vm_config],
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Create Virtual Machine"),
                    description: String::from("Create a new VM with minimal configuration"),
                    code_example: Some(String::from("let vm_id = hypervisor.create_vm(config);")),
                    expected_output: Some(String::from("VM created successfully with ID: 1")),
                    verification_commands: vec![String::from("hypervisor list")],
                    troubleshooting_tips: vec![
                        String::from("Check hardware virtualization support"),
                        String::from("Verify sufficient memory allocation"),
                    ],
                },
                TutorialStep {
                    step_number: 2,
                    title: String::from("Start Virtual Machine"),
                    description: String::from("Start the VM and observe boot process"),
                    code_example: Some(String::from("hypervisor start --vm 1")),
                    expected_output: Some(String::from("VM started, boot sequence initiated")),
                    verification_commands: vec![String::from("hypervisor status --vm 1")],
                    troubleshooting_tips: vec![
                        String::from("Check kernel image exists"),
                        String::from("Verify boot configuration"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Boot Process Documentation"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/boot_process.md")),
                    description: String::from("Detailed explanation of the boot process"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Create multi-OS comparison example
    fn create_multi_os_comparison_example(&mut self) -> Result<(), HypervisorError> {
        let vm_configs = vec![
            // Linux VM
            VmConfig {
                name: String::from("Linux Comparison"),
                vcpu_count: 2,
                memory_mb: 2048,
                arch: VmArchitecture::X86_64,
                boot: BootConfig {
                    boot_order: crate::core::vm_config::BootOrder::DiskFirst,
                    kernel_path: Some(String::from("linux_kernel.bin")),
                    initrd_path: Some(String::from("linux_initrd.img")),
                    kernel_args: String::from("console=ttyS0 root=/dev/sda1"),
                    timeout_sec: 30,
                },
                devices: DeviceConfig::default(),
                features: VmFeatures::EDUCATIONAL,
                network: NetworkConfig::default(),
                storage: StorageConfig::default(),
                security: SecurityConfig::default(),
            },
            // Windows VM
            VmConfig {
                name: String::from("Windows Comparison"),
                vcpu_count: 2,
                memory_mb: 2048,
                arch: VmArchitecture::X86_64,
                boot: BootConfig {
                    boot_order: crate::core::vm_config::BootOrder::CdromFirst,
                    kernel_path: None,
                    initrd_path: None,
                    kernel_args: String::new(),
                    timeout_sec: 30,
                },
                devices: DeviceConfig::default(),
                features: VmFeatures::EDUCATIONAL,
                network: NetworkConfig::default(),
                storage: StorageConfig::default(),
                security: SecurityConfig::default(),
            },
            // BSD VM
            VmConfig {
                name: String::from("BSD Comparison"),
                vcpu_count: 2,
                memory_mb: 2048,
                arch: VmArchitecture::X86_64,
                boot: BootConfig {
                    boot_order: crate::core::vm_config::BootOrder::DiskFirst,
                    kernel_path: Some(String::from("bsd_kernel.bin")),
                    initrd_path: None,
                    kernel_args: String::from("console=ttyS0"),
                    timeout_sec: 30,
                },
                devices: DeviceConfig::default(),
                features: VmFeatures::EDUCATIONAL,
                network: NetworkConfig::default(),
                storage: StorageConfig::default(),
                security: SecurityConfig::default(),
            },
        ];
        
        let tutorial = EducationalTutorial {
            id: EducationalExample::MultiOSComparison,
            title: String::from("Multi-OS Virtualization Comparison"),
            description: String::from("Compare different operating systems running in VMs"),
            difficulty: DifficultyLevel::Intermediate,
            estimated_duration_minutes: 90,
            learning_objectives: vec![
                String::from("Understand virtualization of different OS types"),
                String::from("Learn VM configuration differences"),
                String::from("Practice multi-VM management"),
                String::from("Analyze performance differences"),
            ],
            prerequisites: vec![
                String::from("Basic virtualization concepts"),
                String::from("Understanding of different OS architectures"),
            ],
            vm_configs,
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Create Multiple VMs"),
                    description: String::from("Create VMs for different operating systems"),
                    code_example: Some(String::from("hypervisor create --name linux_vm --config linux.toml")),
                    expected_output: Some(String::from("Created VM for Linux comparison")),
                    verification_commands: vec![String::from("hypervisor list --type all")],
                    troubleshooting_tips: vec![
                        String::from("Ensure different boot configurations"),
                        String::from("Check storage requirements"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Operating System Comparison Guide"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/os_comparison.md")),
                    description: String::from("Guide comparing different OS behaviors in virtualization"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Create nested virtualization example
    fn create_nested_virtualization_example(&mut self) -> Result<(), HypervisorError> {
        let host_vm_config = VmConfig {
            name: String::from("Nested Host VM"),
            vcpu_count: 4,
            memory_mb: 4096,
            arch: VmArchitecture::X86_64,
            boot: BootConfig {
                boot_order: crate::core::vm_config::BootOrder::DiskFirst,
                kernel_path: Some(String::from("linux_kernel.bin")),
                initrd_path: Some(String::from("linux_initrd.img")),
                kernel_args: String::from("console=ttyS0 nested=y"),
                timeout_sec: 30,
            },
            devices: DeviceConfig::nested(),
            features: VmFeatures::NESTED | VmFeatures::EDUCATIONAL | VmFeatures::DEBUG,
            network: NetworkConfig::default(),
            storage: StorageConfig::nested(),
            security: SecurityConfig::default(),
        };
        
        let guest_vm_config = VmConfig {
            name: String::from("Nested Guest VM"),
            vcpu_count: 2,
            memory_mb: 1024,
            arch: VmArchitecture::X86_64,
            boot: BootConfig {
                boot_order: crate::core::vm_config::BootOrder::DiskFirst,
                kernel_path: Some(String::from("linux_kernel.bin")),
                initrd_path: Some(String::from("linux_initrd.img")),
                kernel_args: String::from("console=ttyS0"),
                timeout_sec: 10,
            },
            devices: DeviceConfig::educational(),
            features: VmFeatures::EDUCATIONAL,
            network: NetworkConfig::disabled(),
            storage: StorageConfig::minimal(),
            security: SecurityConfig::default(),
        };
        
        let tutorial = EducationalTutorial {
            id: EducationalExample::NestedVirtualization,
            title: String::from("Nested Virtualization Experiment"),
            description: String::from("Run VMs inside VMs to understand virtualization layers"),
            difficulty: DifficultyLevel::Advanced,
            estimated_duration_minutes: 120,
            learning_objectives: vec![
                String::from("Understand nested virtualization concepts"),
                String::from("Learn EPT/NPT usage in nested scenarios"),
                String::from("Analyze performance overhead"),
                String::from("Practice VM hierarchy management"),
            ],
            prerequisites: vec![
                String::from("Basic virtualization experience"),
                String::from("Understanding of hardware virtualization extensions"),
            ],
            vm_configs: vec![host_vm_config, guest_vm_config],
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Enable Nested Virtualization"),
                    description: String::from("Configure host VM for nested virtualization"),
                    code_example: Some(String::from("hypervisor config --enable-nested --features nested,debug")),
                    expected_output: Some(String::from("Nested virtualization enabled")),
                    verification_commands: vec![String::from("hypervisor capabilities --check nested")],
                    troubleshooting_tips: vec![
                        String::from("Verify hardware support for nested virtualization"),
                        String::from("Check CPU virtualization extensions"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Nested Virtualization Theory"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/nested_virtualization.md")),
                    description: String::from("Theoretical background on nested virtualization"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Create kernel development example
    fn create_kernel_development_example(&mut self) -> Result<(), HypervisorError> {
        let kernel_vm_config = VmConfig {
            name: String::from("Kernel Development"),
            vcpu_count: 2,
            memory_mb: 2048,
            arch: VmArchitecture::X86_64,
            boot: BootConfig {
                boot_order: crate::core::vm_config::BootOrder::DiskFirst,
                kernel_path: Some(String::from("custom_kernel.bin")),
                initrd_path: Some(String::from("dev_initrd.img")),
                kernel_args: String::from("console=ttyS0 debug=y"),
                timeout_sec: 10,
            },
            devices: DeviceConfig {
                graphics: crate::core::vm_config::GraphicsConfig::default(),
                network_adapters: Vec::new(),
                storage_devices: vec![
                    crate::core::vm_config::StorageDeviceConfig {
                        device_type: crate::core::vm_config::StorageDeviceType::Qcow2,
                        file_path: Some(String::from("kernel_disk.qcow2")),
                        size_bytes: 10 * 1024 * 1024 * 1024,
                        read_only: false,
                        cache_mode: crate::core::vm_config::CacheMode::WriteThrough,
                    }
                ],
                serial_console: crate::core::vm_config::SerialConfig::enabled(),
                audio: crate::core::vm_config::AudioConfig::disabled(),
                usb: crate::core::vm_config::UsbConfig::disabled(),
            },
            features: VmFeatures::KERNEL_DEBUG | VmFeatures::EDUCATIONAL,
            network: NetworkConfig::disabled(),
            storage: StorageConfig::minimal(),
            security: SecurityConfig::default(),
        };
        
        let tutorial = EducationalTutorial {
            id: EducationalExample::KernelDevelopment,
            title: String::from("Operating System Kernel Development"),
            description: String::from("Develop and test operating system kernels in virtual machines"),
            difficulty: DifficultyLevel::Advanced,
            estimated_duration_minutes: 180,
            learning_objectives: vec![
                String::from("Set up kernel development environment"),
                String::from("Learn debugging techniques for OS development"),
                String::from("Understand interrupt handling in VMs"),
                String::from("Practice kernel memory management"),
            ],
            prerequisites: vec![
                String::from("Advanced systems programming knowledge"),
                String::from("Understanding of x86 architecture"),
                String::from("Debugging experience"),
            ],
            vm_configs: vec![kernel_vm_config],
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Create Debug Environment"),
                    description: String::from("Set up VM with debug capabilities"),
                    code_example: Some(String::from("hypervisor create --config kernel_dev.toml --features debug,kernel_debug")),
                    expected_output: Some(String::from("Debug VM created with kernel development features")),
                    verification_commands: vec![String::from("hypervisor verify --vm 1 --debug")],
                    troubleshooting_tips: vec![
                        String::from("Ensure debug kernel is built"),
                        String::from("Check serial console configuration"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Kernel Development Guide"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/kernel_development.md")),
                    description: String::from("Comprehensive guide for kernel development"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Create device driver example
    fn create_device_driver_example(&mut self) -> Result<(), HypervisorError> {
        let tutorial = EducationalTutorial {
            id: EducationalExample::DeviceDriverExample,
            title: String::from("Device Driver Development"),
            description: String::from("Develop and test device drivers in virtual environments"),
            difficulty: DifficultyLevel::Advanced,
            estimated_duration_minutes: 150,
            learning_objectives: vec![
                String::from("Understand device driver architecture"),
                String::from("Learn device virtualization concepts"),
                String::from("Practice driver debugging techniques"),
                String::from("Understand interrupt handling"),
            ],
            prerequisites: vec![
                String::from("Kernel development experience"),
                String::from("Hardware interface knowledge"),
            ],
            vm_configs: vec![VmConfig::educational()],
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Create Educational Device"),
                    description: String::from("Create VM with educational demo device"),
                    code_example: Some(String::from("hypervisor create --with-demo-device")),
                    expected_output: Some(String::from("VM created with educational demo device")),
                    verification_commands: vec![String::from("lspci | grep Educational")],
                    troubleshooting_tips: vec![
                        String::from("Check device registration"),
                        String::from("Verify interrupt handling"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Device Driver Tutorial"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/device_drivers.md")),
                    description: String::from("Tutorial for device driver development"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Create memory management example
    fn create_memory_management_example(&mut self) -> Result<(), HypervisorError> {
        let tutorial = EducationalTutorial {
            id: EducationalExample::MemoryManagement,
            title: String::from("Memory Management in Virtualization"),
            description: String::from("Learn memory virtualization techniques and EPT/NPT"),
            difficulty: DifficultyLevel::Advanced,
            estimated_duration_minutes: 120,
            learning_objectives: vec![
                String::from("Understand extended page tables (EPT)"),
                String::from("Learn nested page tables (NPT)"),
                String::from("Practice memory debugging"),
                String::from("Analyze memory performance"),
            ],
            prerequisites: vec![
                String::from("Memory management concepts"),
                String::from("Page table understanding"),
            ],
            vm_configs: vec![VmConfig::educational()],
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Examine EPT Structure"),
                    description: String::from("Analyze extended page table organization"),
                    code_example: Some(String::from("hypervisor debug --show-ept --vm 1")),
                    expected_output: Some(String::from("EPT structure displayed")),
                    verification_commands: vec![String::from("cat /proc/vmstat | grep ept")],
                    troubleshooting_tips: vec![
                        String::from("Check EPTP configuration"),
                        String::from("Verify page table setup"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Memory Virtualization Theory"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/memory_virtualization.md")),
                    description: String::from("Theory behind memory virtualization"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Create teaching lab example
    fn create_teaching_lab_example(&mut self) -> Result<(), HypervisorError> {
        let mut vm_configs = Vec::new();
        
        // Create multiple student VMs
        for i in 0..5 {
            let vm_config = VmConfig {
                name: format!("Student VM {}", i + 1),
                vcpu_count: 1,
                memory_mb: 1024,
                arch: VmArchitecture::X86_64,
                boot: BootConfig::default(),
                devices: DeviceConfig::educational(),
                features: VmFeatures::EDUCATIONAL | VmFeatures::SNAPSHOT_SUPPORT,
                network: NetworkConfig::default(),
                storage: StorageConfig::minimal(),
                security: SecurityConfig::default(),
            };
            vm_configs.push(vm_config);
        }
        
        let tutorial = EducationalTutorial {
            id: EducationalExample::TeachingLab,
            title: String::from("Multi-Student Virtualization Lab"),
            description: String::from("Set up virtualization environment for multiple students"),
            difficulty: DifficultyLevel::Intermediate,
            estimated_duration_minutes: 60,
            learning_objectives: vec![
                String::from("Learn multi-VM management"),
                String::from("Practice VM snapshotting"),
                String::from("Understand resource allocation"),
                String::from("Learn student isolation techniques"),
            ],
            prerequisites: vec![
                String::from("Basic hypervisor usage"),
                String::from("Understanding of VM isolation"),
            ],
            vm_configs,
            steps: vec![
                TutorialStep {
                    step_number: 1,
                    title: String::from("Create Student VMs"),
                    description: String::from("Create isolated VMs for multiple students"),
                    code_example: Some(String::from("hypervisor create-lab --students 5 --name student_lab")),
                    expected_output: Some(String::from("Created 5 isolated student VMs")),
                    verification_commands: vec![String::from("hypervisor list --filter students")],
                    troubleshooting_tips: vec![
                        String::from("Ensure sufficient system resources"),
                        String::from("Check network isolation settings"),
                    ],
                },
            ],
            resources: vec![
                TutorialResource {
                    title: String::from("Teaching Lab Guide"),
                    resource_type: ResourceType::Documentation,
                    url: Some(String::from("/docs/teaching_lab.md")),
                    description: String::from("Guide for setting up teaching environments"),
                },
            ],
        };
        
        self.tutorials.push(tutorial);
        Ok(())
    }
    
    /// Get tutorial by ID
    pub fn get_tutorial(&self, id: EducationalExample) -> Option<&EducationalTutorial> {
        self.tutorials.iter().find(|t| t.id == id)
    }
    
    /// List all available tutorials
    pub fn list_tutorials(&self) -> Vec<EducationalExample> {
        self.tutorials.iter().map(|t| t.id).collect()
    }
    
    /// Get tutorials by difficulty level
    pub fn get_tutorials_by_difficulty(&self, difficulty: DifficultyLevel) -> Vec<&EducationalTutorial> {
        self.tutorials.iter().filter(|t| t.difficulty == difficulty).collect()
    }
    
    /// Start a tutorial
    pub fn start_tutorial(&mut self, id: EducationalExample) -> Result<(), HypervisorError> {
        if self.get_tutorial(id).is_some() {
            self.current_tutorial = Some(id);
            info!("Started tutorial: {:?}", id);
            Ok(())
        } else {
            Err(HypervisorError::ConfigurationError(String::from("Tutorial not found")))
        }
    }
    
    /// Complete a tutorial
    pub fn complete_tutorial(&mut self, id: EducationalExample) -> Result<(), HypervisorError> {
        if let Some(index) = self.completed_tutorials.iter().position(|&t| t == id) {
            return Err(HypervisorError::ConfigurationError(String::from("Tutorial already completed")));
        }
        
        self.completed_tutorials.push(id);
        info!("Completed tutorial: {:?}", id);
        Ok(())
    }
    
    /// Get current tutorial
    pub fn get_current_tutorial(&self) -> Option<EducationalExample> {
        self.current_tutorial
    }
    
    /// Get completion statistics
    pub fn get_completion_stats(&self) -> CompletionStats {
        let total = self.tutorials.len();
        let completed = self.completed_tutorials.len();
        
        CompletionStats {
            total_tutorials: total,
            completed_tutorials: completed,
            completion_percentage: if total > 0 {
                (completed as f32 / total as f32) * 100.0
            } else {
                0.0
            },
            difficulty_distribution: self.calculate_difficulty_distribution(),
        }
    }
    
    /// Calculate difficulty distribution
    fn calculate_difficulty_distribution(&self) -> DifficultyDistribution {
        let mut beginner = 0;
        let mut intermediate = 0;
        let mut advanced = 0;
        let mut expert = 0;
        
        for tutorial in &self.tutorials {
            match tutorial.difficulty {
                DifficultyLevel::Beginner => beginner += 1,
                DifficultyLevel::Intermediate => intermediate += 1,
                DifficultyLevel::Advanced => advanced += 1,
                DifficultyLevel::Expert => expert += 1,
            }
        }
        
        DifficultyDistribution {
            beginner,
            intermediate,
            advanced,
            expert,
        }
    }
    
    /// Generate educational report
    pub fn generate_educational_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Educational Virtualization Report\n");
        report.push_str("==================================\n\n");
        
        let stats = self.get_completion_stats();
        report.push_str(&format!("Total tutorials: {}\n", stats.total_tutorials));
        report.push_str(&format!("Completed tutorials: {}\n", stats.completed_tutorials));
        report.push_str(&format!("Completion percentage: {:.1}%\n\n", stats.completion_percentage));
        
        report.push_str("Difficulty Distribution:\n");
        report.push_str(&format!("  Beginner: {}\n", stats.difficulty_distribution.beginner));
        report.push_str(&format!("  Intermediate: {}\n", stats.difficulty_distribution.intermediate));
        report.push_str(&format!("  Advanced: {}\n", stats.difficulty_distribution.advanced));
        report.push_str(&format!("  Expert: {}\n\n", stats.difficulty_distribution.expert));
        
        report.push_str("Available Tutorials:\n");
        for tutorial in &self.tutorials {
            report.push_str(&format!("  {:?}: {} ({:?})\n", 
                                  tutorial.id, tutorial.title, tutorial.difficulty));
        }
        
        report
    }
}

/// Completion statistics
#[derive(Debug, Clone)]
pub struct CompletionStats {
    pub total_tutorials: usize,
    pub completed_tutorials: usize,
    pub completion_percentage: f32,
    pub difficulty_distribution: DifficultyDistribution,
}

/// Difficulty distribution
#[derive(Debug, Clone)]
pub struct DifficultyDistribution {
    pub beginner: usize,
    pub intermediate: usize,
    pub advanced: usize,
    pub expert: usize,
}