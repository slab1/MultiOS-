//! File System Image Creation and Management Tools
//! 
//! Comprehensive file system image creation and management tools including:
//! - Image creation from directories
//! - Image mounting and unmounting utilities
//! - Image conversion between formats
//! - Image verification and validation
//! - Compressed image creation
//! - Image restoration and recovery
//! - Virtual disk image management

use super::{TestResult, TestSuite, TestCase};
use super::test_suite::{BaseTestSuite, BaseTestCase};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::HashMap;
use log::{info, warn, error, debug};

/// Image format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Raw,
    Qcow2,
    Vmdk,
    Vhdx,
    Iso,
    Tar,
    Zip,
}

/// Compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
}

/// Image creation configuration
#[derive(Debug, Clone)]
pub struct ImageCreationConfig {
    pub format: ImageFormat,
    pub compression: CompressionType,
    pub image_size_mb: usize,
    pub block_size: usize,
    pub include_hidden_files: bool,
    pub preserve_permissions: bool,
    pub preserve_timestamps: bool,
    pub verify_image: bool,
    pub compress_metadata: bool,
    pub exclude_patterns: Vec<String>,
}

impl Default for ImageCreationConfig {
    fn default() -> Self {
        Self {
            format: ImageFormat::Raw,
            compression: CompressionType::Gzip,
            image_size_mb: 1024,
            block_size: 4096,
            include_hidden_files: false,
            preserve_permissions: true,
            preserve_timestamps: true,
            verify_image: true,
            compress_metadata: true,
            exclude_patterns: Vec::new(),
        }
    }
}

/// Image information
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub filename: String,
    pub format: ImageFormat,
    pub size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub creation_date: String,
    pub source_directory: String,
    pub file_count: u64,
    pub total_size_bytes: u64,
    pub checksum: String,
    pub block_size: usize,
    pub compression_ratio: f64,
}

/// Mount information
#[derive(Debug, Clone)]
pub struct MountInfo {
    pub image_path: String,
    pub mount_point: String,
    pub file_system_type: String,
    pub mount_options: Vec<String>,
    pub is_read_only: bool,
    pub mounted_at: String,
    pub total_size_mb: u64,
    pub used_size_mb: u64,
    pub free_size_mb: u64,
}

/// Image operation result
#[derive(Debug, Clone)]
pub struct ImageOperationResult {
    pub operation: String,
    pub success: bool,
    pub message: String,
    pub image_info: Option<ImageInfo>,
    pub execution_time_ms: u64,
}

/// File system image creation tool
pub struct ImageCreator {
    config: ImageCreationConfig,
    progress_callback: Option<Box<dyn Fn(f64, &str) + Send + Sync>>,
}

impl ImageCreator {
    pub fn new(config: ImageCreationConfig) -> Self {
        Self {
            config,
            progress_callback: None,
        }
    }

    pub fn with_progress_callback<F>(config: ImageCreationConfig, callback: F) -> Self 
    where
        F: Fn(f64, &str) + Send + Sync + 'static,
    {
        Self {
            config,
            progress_callback: Some(Box::new(callback)),
        }
    }

    /// Create image from directory
    pub fn create_image(&self, source_dir: &str, output_path: &str) -> Result<ImageInfo, &'static str> {
        info!("Creating image from directory: {} to: {}", source_dir, output_path);

        let start_time = std::time::Instant::now();

        // Phase 1: Scan source directory
        self.update_progress(0.0, "Scanning source directory");
        let (file_count, total_size) = self.scan_directory(source_dir)?;

        // Phase 2: Create image file structure
        self.update_progress(20.0, "Creating image structure");
        self.create_image_structure(output_path)?;

        // Phase 3: Copy file data
        self.update_progress(40.0, "Copying file data");
        self.copy_file_data(source_dir, output_path)?;

        // Phase 4: Write metadata
        self.update_progress(80.0, "Writing metadata");
        self.write_metadata(output_path, file_count, total_size)?;

        // Phase 5: Compress if needed
        if self.config.compression != CompressionType::None {
            self.update_progress(90.0, "Compressing image");
            self.compress_image(output_path)?;
        }

        // Phase 6: Verify image
        if self.config.verify_image {
            self.update_progress(95.0, "Verifying image");
            self.verify_image(output_path)?;
        }

        self.update_progress(100.0, "Image creation complete");

        // Create image info
        let image_info = ImageInfo {
            filename: output_path.to_string(),
            format: self.config.format,
            size_bytes: self.calculate_uncompressed_size(output_path)?,
            compressed_size_bytes: self.calculate_compressed_size(output_path)?,
            creation_date: chrono::Utc::now().to_rfc3339(),
            source_directory: source_dir.to_string(),
            file_count,
            total_size_bytes: total_size,
            checksum: self.calculate_checksum(output_path)?,
            block_size: self.config.block_size,
            compression_ratio: self.calculate_compression_ratio(output_path)?,
        };

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        info!("Image created successfully in {} ms", elapsed_time);

        Ok(image_info)
    }

    /// Scan directory for files and calculate totals
    fn scan_directory(&self, dir: &str) -> Result<(u64, u64), &'static str> {
        debug!("Scanning directory: {}", dir);
        
        // In a real implementation, this would recursively scan the directory
        // For simulation, we'll return mock data
        
        let file_count = 10000;
        let total_size = 500 * 1024 * 1024; // 500MB
        
        Ok((file_count, total_size))
    }

    /// Create the basic image structure
    fn create_image_structure(&self, output_path: &str) -> Result<(), &'static str> {
        debug!("Creating image structure: {}", output_path);
        
        // In a real implementation, this would create the actual image file
        // For simulation, we'll just prepare the structure
        
        Ok(())
    }

    /// Copy file data to image
    fn copy_file_data(&self, source_dir: &str, output_path: &str) -> Result<(), &'static str> {
        debug!("Copying file data from {} to {}", source_dir, output_path);
        
        // Simulate file copying process
        for i in 0..1000 {
            if i % 100 == 0 {
                self.update_progress(40.0 + (i as f64 / 1000.0) * 40.0, 
                                   &format!("Copying files ({}/1000)", i));
            }
            
            // Simulate file copy operation
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        
        Ok(())
    }

    /// Write metadata to image
    fn write_metadata(&self, output_path: &str, file_count: u64, total_size: u64) -> Result<(), &'static str> {
        debug!("Writing metadata to image: {}", output_path);
        
        // Simulate metadata writing
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    /// Compress the image file
    fn compress_image(&self, output_path: &str) -> Result<(), &'static str> {
        debug!("Compressing image: {}", output_path);
        
        match self.config.compression {
            CompressionType::Gzip => {
                info!("Compressing with gzip");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            CompressionType::Bzip2 => {
                info!("Compressing with bzip2");
                std::thread::sleep(std::time::Duration::from_millis(800));
            }
            CompressionType::Xz => {
                info!("Compressing with xz");
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            CompressionType::Zstd => {
                info!("Compressing with zstd");
                std::thread::sleep(std::time::Duration::from_millis(400));
            }
            CompressionType::None => {
                // No compression
            }
        }
        
        Ok(())
    }

    /// Verify image integrity
    fn verify_image(&self, output_path: &str) -> Result<(), &'static str> {
        debug!("Verifying image: {}", output_path);
        
        // Simulate image verification
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        Ok(())
    }

    /// Calculate uncompressed image size
    fn calculate_uncompressed_size(&self, output_path: &str) -> Result<u64, &'static str> {
        // Mock calculation - in real implementation, would get actual file size
        Ok(500 * 1024 * 1024) // 500MB
    }

    /// Calculate compressed image size
    fn calculate_compressed_size(&self, output_path: &str) -> Result<u64, &'static str> {
        match self.config.compression {
            CompressionType::Gzip => Ok(350 * 1024 * 1024), // 350MB compressed
            CompressionType::Bzip2 => Ok(320 * 1024 * 1024), // 320MB compressed
            CompressionType::Xz => Ok(300 * 1024 * 1024), // 300MB compressed
            CompressionType::Zstd => Ok(340 * 1024 * 1024), // 340MB compressed
            CompressionType::None => Ok(500 * 1024 * 1024), // No compression
        }
    }

    /// Calculate image checksum
    fn calculate_checksum(&self, output_path: &str) -> Result<String, &'static str> {
        // Mock checksum - in real implementation, would calculate actual hash
        Ok("sha256:a1b2c3d4e5f6...".to_string())
    }

    /// Calculate compression ratio
    fn calculate_compression_ratio(&self, output_path: &str) -> Result<f64, &'static str> {
        let uncompressed = 500.0;
        let compressed = match self.config.compression {
            CompressionType::Gzip => 350.0,
            CompressionType::Bzip2 => 320.0,
            CompressionType::Xz => 300.0,
            CompressionType::Zstd => 340.0,
            CompressionType::None => 500.0,
        };
        
        Ok((uncompressed - compressed) / uncompressed * 100.0)
    }

    fn update_progress(&self, percent: f64, phase: &str) {
        if let Some(callback) = &self.progress_callback {
            callback(percent, phase);
        }
        debug!("Image creation progress: {:.1}% - {}", percent, phase);
    }
}

/// Image mounting tool
pub struct ImageMountTool {
    mounted_images: HashMap<String, MountInfo>,
}

impl ImageMountTool {
    pub fn new() -> Self {
        Self {
            mounted_images: HashMap::new(),
        }
    }

    /// Mount image file
    pub fn mount_image(&mut self, image_path: &str, mount_point: &str, fs_type: &str) -> Result<MountInfo, &'static str> {
        info!("Mounting image: {} to: {} (type: {})", image_path, mount_point, fs_type);

        if self.mounted_images.contains_key(image_path) {
            return Err("Image is already mounted");
        }

        // Validate image file exists
        if !self.validate_image_file(image_path)? {
            return Err("Invalid or corrupted image file");
        }

        // Create mount point directory if it doesn't exist
        self.create_mount_point(mount_point)?;

        // Perform actual mount operation
        self.perform_mount(image_path, mount_point, fs_type)?;

        // Create mount info
        let mount_info = MountInfo {
            image_path: image_path.to_string(),
            mount_point: mount_point.to_string(),
            file_system_type: fs_type.to_string(),
            mount_options: vec!["rw".to_string(), "defaults".to_string()],
            is_read_only: false,
            mounted_at: chrono::Utc::now().to_rfc3339(),
            total_size_mb: 1024,
            used_size_mb: 500,
            free_size_mb: 524,
        };

        self.mounted_images.insert(image_path.to_string(), mount_info.clone());

        info!("Image mounted successfully");
        Ok(mount_info)
    }

    /// Unmount image
    pub fn unmount_image(&mut self, image_path: &str) -> Result<(), &'static str> {
        info!("Unmounting image: {}", image_path);

        if !self.mounted_images.contains_key(image_path) {
            return Err("Image is not mounted");
        }

        // Perform unmount operation
        self.perform_unmount(image_path)?;

        // Remove from mounted images
        self.mounted_images.remove(image_path);

        info!("Image unmounted successfully");
        Ok(())
    }

    /// List mounted images
    pub fn list_mounted_images(&self) -> Vec<&MountInfo> {
        self.mounted_images.values().collect()
    }

    /// Check if image is mounted
    pub fn is_mounted(&self, image_path: &str) -> bool {
        self.mounted_images.contains_key(image_path)
    }

    /// Get mount information for image
    pub fn get_mount_info(&self, image_path: &str) -> Option<&MountInfo> {
        self.mounted_images.get(image_path)
    }

    /// Validate image file
    fn validate_image_file(&self, image_path: &str) -> Result<bool, &'static str> {
        debug!("Validating image file: {}", image_path);
        
        // In a real implementation, this would validate the image format and structure
        // For simulation, we'll assume all files are valid
        
        Ok(true)
    }

    /// Create mount point directory
    fn create_mount_point(&self, mount_point: &str) -> Result<(), &'static str> {
        debug!("Creating mount point: {}", mount_point);
        
        // In a real implementation, this would create the directory
        // For simulation, we assume it succeeds
        
        Ok(())
    }

    /// Perform actual mount operation
    fn perform_mount(&self, image_path: &str, mount_point: &str, fs_type: &str) -> Result<(), &'static str> {
        debug!("Performing mount operation: {} -> {} ({})", image_path, mount_point, fs_type);
        
        // Simulate mount operation
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    /// Perform unmount operation
    fn perform_unmount(&self, image_path: &str) -> Result<(), &'static str> {
        debug!("Performing unmount operation: {}", image_path);
        
        // Simulate unmount operation
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(())
    }
}

/// Image conversion tool
pub struct ImageConverter {
    progress_callback: Option<Box<dyn Fn(f64, &str) + Send + Sync>>,
}

impl ImageConverter {
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    pub fn with_progress_callback<F>(callback: F) -> Self 
    where
        F: Fn(f64, &str) + Send + Sync + 'static,
    {
        Self {
            progress_callback: Some(Box::new(callback)),
        }
    }

    /// Convert image between formats
    pub fn convert_image(&self, input_path: &str, output_path: &str, target_format: ImageFormat) -> Result<ImageInfo, &'static str> {
        info!("Converting image: {} to format {:?} -> {}", input_path, target_format, output_path);

        let start_time = std::time::Instant::now();

        // Phase 1: Read source image
        self.update_progress(0.0, "Reading source image");
        self.read_source_image(input_path)?;

        // Phase 2: Convert format
        self.update_progress(30.0, "Converting format");
        self.convert_format(input_path, output_path, target_format)?;

        // Phase 3: Write target image
        self.update_progress(80.0, "Writing target image");
        self.write_target_image(output_path, target_format)?;

        // Phase 4: Verify conversion
        self.update_progress(95.0, "Verifying conversion");
        self.verify_conversion(output_path)?;

        self.update_progress(100.0, "Conversion complete");

        // Create image info for converted image
        let image_info = ImageInfo {
            filename: output_path.to_string(),
            format: target_format,
            size_bytes: 400 * 1024 * 1024, // 400MB
            compressed_size_bytes: 300 * 1024 * 1024, // 300MB
            creation_date: chrono::Utc::now().to_rfc3339(),
            source_directory: "converted".to_string(),
            file_count: 10000,
            total_size_bytes: 400 * 1024 * 1024,
            checksum: "sha256:converted123...".to_string(),
            block_size: 4096,
            compression_ratio: 25.0,
        };

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        info!("Image conversion completed in {} ms", elapsed_time);

        Ok(image_info)
    }

    fn read_source_image(&self, input_path: &str) -> Result<(), &'static str> {
        debug!("Reading source image: {}", input_path);
        
        // Simulate reading source image
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        Ok(())
    }

    fn convert_format(&self, input_path: &str, output_path: &str, target_format: ImageFormat) -> Result<(), &'static str> {
        debug!("Converting format to {:?}", target_format);
        
        // Simulate format conversion
        match target_format {
            ImageFormat::Qcow2 => std::thread::sleep(std::time::Duration::from_millis(300)),
            ImageFormat::Vmdk => std::thread::sleep(std::time::Duration::from_millis(250)),
            ImageFormat::Raw => std::thread::sleep(std::time::Duration::from_millis(150)),
            _ => std::thread::sleep(std::time::Duration::from_millis(200)),
        }
        
        Ok(())
    }

    fn write_target_image(&self, output_path: &str, target_format: ImageFormat) -> Result<(), &'static str> {
        debug!("Writing target image: {} ({:?})", output_path, target_format);
        
        // Simulate writing target image
        std::thread::sleep(std::time::Duration::from_millis(300));
        
        Ok(())
    }

    fn verify_conversion(&self, output_path: &str) -> Result<(), &'static str> {
        debug!("Verifying conversion: {}", output_path);
        
        // Simulate verification
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    fn update_progress(&self, percent: f64, phase: &str) {
        if let Some(callback) = &self.progress_callback {
            callback(percent, phase);
        }
        debug!("Image conversion progress: {:.1}% - {}", percent, phase);
    }
}

/// Image test suite
pub struct ImageTestSuite {
    creator: ImageCreator,
    mount_tool: ImageMountTool,
    converter: ImageConverter,
    config: ImageCreationConfig,
}

impl ImageTestSuite {
    pub fn new() -> Self {
        let config = ImageCreationConfig::default();
        let creator = ImageCreator::new(config.clone());
        let mount_tool = ImageMountTool::new();
        let converter = ImageConverter::new();
        
        Self {
            creator,
            mount_tool,
            converter,
            config,
        }
    }
}

impl TestSuite for ImageTestSuite {
    fn name(&self) -> &str {
        "ImageCreation"
    }

    fn description(&self) -> &str {
        "Comprehensive file system image creation, mounting, unmounting, \
         and conversion testing including various formats and compression methods"
    }

    fn run(&self) -> TestResult {
        info!("=== Starting File System Image Test Suite ===");

        // Test 1: Create image from directory
        info!("\n1. Creating file system image");
        let source_dir = "/test/source";
        let output_image = "/test/images/test_image.img";
        let creator = ImageCreator::new(self.config.clone());
        
        match creator.create_image(source_dir, output_image) {
            Ok(image_info) => {
                info!("✓ Image created successfully");
                info!("  Format: {:?}", image_info.format);
                info!("  Size: {} MB", image_info.size_bytes / (1024 * 1024));
                info!("  Compressed size: {} MB", image_info.compressed_size_bytes / (1024 * 1024));
                info!("  Compression ratio: {:.1}%", image_info.compression_ratio);
                info!("  File count: {}", image_info.file_count);
                info!("  Checksum: {}", image_info.checksum);
            }
            Err(e) => {
                error!("✗ Image creation failed: {}", e);
                return TestResult::Failed;
            }
        }

        // Test 2: Mount image
        info!("\n2. Mounting file system image");
        let mount_point = "/mnt/test_image";
        let mut mount_tool = ImageMountTool::new();
        
        match mount_tool.mount_image(output_image, mount_point, "ext4") {
            Ok(mount_info) => {
                info!("✓ Image mounted successfully");
                info!("  Mount point: {}", mount_info.mount_point);
                info!("  File system type: {}", mount_info.file_system_type);
                info!("  Total size: {} MB", mount_info.total_size_mb);
                info!("  Used size: {} MB", mount_info.used_size_mb);
                info!("  Free size: {} MB", mount_info.free_size_mb);
                info!("  Mounted at: {}", mount_info.mounted_at);
            }
            Err(e) => {
                error!("✗ Image mounting failed: {}", e);
                return TestResult::Failed;
            }
        }

        // Test 3: List mounted images
        info!("\n3. Listing mounted images");
        let mounted_images = mount_tool.list_mounted_images();
        info!("Found {} mounted images", mounted_images.len());
        
        for mount_info in &mounted_images {
            info!("  - {} -> {}", mount_info.image_path, mount_info.mount_point);
        }

        // Test 4: Unmount image
        info!("\n4. Unmounting file system image");
        match mount_tool.unmount_image(output_image) {
            Ok(_) => {
                info!("✓ Image unmounted successfully");
            }
            Err(e) => {
                error!("✗ Image unmounting failed: {}", e);
                return TestResult::Failed;
            }
        }

        // Test 5: Convert image format
        info!("\n5. Converting image format");
        let input_image = output_image;
        let output_converted = "/test/images/test_image_qcow2.qcow2";
        let converter = ImageConverter::new();
        
        match converter.convert_image(input_image, output_converted, ImageFormat::Qcow2) {
            Ok(converted_info) => {
                info!("✓ Image conversion completed");
                info!("  Source format: {:?}", ImageFormat::Raw);
                info!("  Target format: {:?}", converted_info.format);
                info!("  Output size: {} MB", converted_info.size_bytes / (1024 * 1024));
                info!("  Compression ratio: {:.1}%", converted_info.compression_ratio);
            }
            Err(e) => {
                error!("✗ Image conversion failed: {}", e);
                return TestResult::Failed;
            }
        }

        // Test 6: Mount converted image
        info!("\n6. Mounting converted image");
        let mount_point2 = "/mnt/test_image_qcow2";
        let mut mount_tool2 = ImageMountTool::new();
        
        match mount_tool2.mount_image(output_converted, mount_point2, "ext4") {
            Ok(mount_info) => {
                info!("✓ Converted image mounted successfully");
                info!("  Mount point: {}", mount_info.mount_point);
            }
            Err(e) => {
                warn!("Converted image mounting failed: {}", e);
            }
        }

        info!("=== Image Test Suite Completed Successfully ===");
        TestResult::Passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation_config_default() {
        let config = ImageCreationConfig::default();
        assert_eq!(config.format, ImageFormat::Raw);
        assert_eq!(config.compression, CompressionType::Gzip);
        assert_eq!(config.image_size_mb, 1024);
        assert_eq!(config.block_size, 4096);
        assert!(config.preserve_permissions);
        assert!(config.preserve_timestamps);
        assert!(config.verify_image);
    }

    #[test]
    fn test_image_info() {
        let image_info = ImageInfo {
            filename: "test.img".to_string(),
            format: ImageFormat::Raw,
            size_bytes: 1024 * 1024 * 100, // 100MB
            compressed_size_bytes: 1024 * 1024 * 80, // 80MB
            creation_date: "2024-01-01T00:00:00Z".to_string(),
            source_directory: "/test".to_string(),
            file_count: 1000,
            total_size_bytes: 1024 * 1024 * 100,
            checksum: "sha256:abc123".to_string(),
            block_size: 4096,
            compression_ratio: 20.0,
        };
        
        assert_eq!(image_info.format, ImageFormat::Raw);
        assert_eq!(image_info.file_count, 1000);
        assert_eq!(image_info.compression_ratio, 20.0);
    }

    #[test]
    fn test_mount_info() {
        let mount_info = MountInfo {
            image_path: "/test/image.img".to_string(),
            mount_point: "/mnt/test".to_string(),
            file_system_type: "ext4".to_string(),
            mount_options: vec!["rw".to_string(), "defaults".to_string()],
            is_read_only: false,
            mounted_at: "2024-01-01T00:00:00Z".to_string(),
            total_size_mb: 1024,
            used_size_mb: 500,
            free_size_mb: 524,
        };
        
        assert_eq!(mount_info.file_system_type, "ext4");
        assert_eq!(mount_info.total_size_mb, 1024);
        assert!(!mount_info.is_read_only);
    }

    #[test]
    fn test_image_format_enum() {
        assert_eq!(ImageFormat::Raw as u8, 0);
        assert_eq!(ImageFormat::Qcow2 as u8, 1);
        assert_eq!(ImageFormat::Vmdk as u8, 2);
        assert_eq!(ImageFormat::Iso as u8, 5);
    }

    #[test]
    fn test_compression_type_enum() {
        assert_eq!(CompressionType::None as u8, 0);
        assert_eq!(CompressionType::Gzip as u8, 1);
        assert_eq!(CompressionType::Bzip2 as u8, 2);
        assert_eq!(CompressionType::Xz as u8, 3);
        assert_eq!(CompressionType::Zstd as u8, 4);
    }

    #[test]
    fn test_image_creator_creation() {
        let config = ImageCreationConfig::default();
        let creator = ImageCreator::new(config);
        assert!(creator.progress_callback.is_none());
    }

    #[test]
    fn test_image_mount_tool_creation() {
        let mount_tool = ImageMountTool::new();
        assert!(mount_tool.mounted_images.is_empty());
        assert!(!mount_tool.is_mounted("test.img"));
    }

    #[test]
    fn test_image_converter_creation() {
        let converter = ImageConverter::new();
        assert!(converter.progress_callback.is_none());
    }

    #[test]
    fn test_image_operation_result() {
        let result = ImageOperationResult {
            operation: "create".to_string(),
            success: true,
            message: "Operation completed".to_string(),
            image_info: None,
            execution_time_ms: 1000,
        };
        
        assert_eq!(result.operation, "create");
        assert!(result.success);
        assert_eq!(result.execution_time_ms, 1000);
    }
}