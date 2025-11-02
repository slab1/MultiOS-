//! Container Image Management
//! 
//! This module provides comprehensive container image management including
//! image format, storage, layering, and distribution capabilities.

use super::*;
use std::io::{Read, Write};
use std::fs::{File, create_dir_all};
use std::time::{SystemTime, Duration};

/// Container Image Manager - Handles all container image operations
pub struct ImageManager {
    image_root: PathBuf,
    registry_path: PathBuf,
    layer_cache: Arc<Mutex<HashMap<String, ImageLayer>>>,
}

impl ImageManager {
    /// Create a new image manager
    pub fn new() -> Self {
        let image_root = PathBuf::from("/var/lib/multios/images");
        let registry_path = PathBuf::from("/var/lib/multios/registry");
        
        Self {
            image_root,
            registry_path,
            layer_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize image manager directories
    pub async fn initialize(&self) -> ContainerResult<()> {
        create_dir_all(&self.image_root)
            .map_err(|e| ContainerError::System(format!("Failed to create image root: {}", e)))?;
        
        create_dir_all(&self.registry_path)
            .map_err(|e| ContainerError::System(format!("Failed to create registry path: {}", e)))?;

        // Create subdirectories
        let subdirs = ["layers", "images", "metadata", "manifests"];
        for subdir in &subdirs {
            let path = self.image_root.join(subdir);
            create_dir_all(&path)
                .map_err(|e| ContainerError::System(format!("Failed to create subdir {}: {}", subdir, e)))?;
        }

        Ok(())
    }

    /// Pull an image from registry or local storage
    pub async fn pull_image(&self, image_spec: &str) -> ContainerResult<String> {
        // Parse image specification
        let image_info = self.parse_image_spec(image_spec)?;
        
        // Check if image already exists locally
        if self.image_exists(&image_info.name, &image_info.tag).await? {
            log::info!("Image {}:{} already exists locally", image_info.name, image_info.tag);
            return Ok(format!("{}:{}", image_info.name, image_info.tag));
        }

        // Check if it's a local educational template
        if self.is_educational_template(image_spec).await? {
            return self.pull_educational_template(image_spec).await;
        }

        // In a real implementation, this would pull from a registry
        // For now, we'll simulate pulling from a local mirror
        self.pull_from_mirror(&image_info).await?;

        Ok(format!("{}:{}", image_info.name, image_info.tag))
    }

    /// Remove an image from local storage
    pub async fn remove_image(&self, image_name: &str, force: bool) -> ContainerResult<()> {
        let image_path = self.image_root.join("images").join(format!("{}.json", image_name));
        
        if !image_path.exists() {
            return Err(ContainerError::NotFound(format!("Image {} not found", image_name)));
        }

        // Load image metadata
        let metadata: ImageMetadata = self.load_image_metadata(image_name).await?;

        // Check if image is in use
        if !force && self.is_image_in_use(image_name).await? {
            return Err(ContainerError::InvalidConfig(
                "Image is in use by running containers. Use force to remove anyway.".to_string()
            ));
        }

        // Remove all layers associated with this image
        for layer_id in &metadata.layers {
            self.remove_layer(layer_id).await?;
        }

        // Remove image metadata
        std::fs::remove_file(&image_path)
            .map_err(|e| ContainerError::System(format!("Failed to remove image metadata: {}", e)))?;

        Ok(())
    }

    /// List all available images
    pub async fn list_images(&self) -> ContainerResult<Vec<ImageInfo>> {
        let images_dir = self.image_root.join("images");
        let mut images = Vec::new();

        if images_dir.exists() {
            for entry in std::fs::read_dir(&images_dir)
                .map_err(|e| ContainerError::System(format!("Failed to read images directory: {}", e)))? {
                
                let entry = entry.map_err(|e| ContainerError::System(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(image_name) = path.file_stem().and_then(|s| s.to_str()) {
                        let metadata = self.load_image_metadata(image_name).await?;
                        images.push(ImageInfo {
                            id: image_name.to_string(),
                            name: metadata.name,
                            tag: metadata.tag,
                            size: metadata.size,
                            created: metadata.created,
                            layers: metadata.layers.len(),
                            description: metadata.description,
                        });
                    }
                }
            }
        }

        Ok(images)
    }

    /// Get detailed information about an image
    pub async fn get_image_info(&self, image_name: &str) -> ContainerResult<ImageDetails> {
        let metadata = self.load_image_metadata(image_name).await?;
        
        let layers_info = self.get_layers_info(&metadata.layers).await?;

        Ok(ImageDetails {
            metadata,
            layers: layers_info,
            configuration: self.load_image_config(image_name).await?,
            history: self.load_image_history(image_name).await?,
        })
    }

    /// Extract image layers for container creation
    pub async fn extract_image(&self, image_name: &str, target_path: &PathBuf) -> ContainerResult<ExtractedImage> {
        let metadata = self.load_image_metadata(image_name).await?;

        // Create target directory
        create_dir_all(target_path)
            .map_err(|e| ContainerError::System(format!("Failed to create target directory: {}", e)))?;

        // Extract all layers in order
        let mut layer_hashes = Vec::new();
        for layer_id in &metadata.layers {
            self.extract_layer(layer_id, target_path).await?;
            layer_hashes.push(layer_id.clone());
        }

        // Create container configuration
        let config = self.generate_container_config(image_name, &metadata).await?;

        Ok(ExtractedImage {
            rootfs_path: target_path.clone(),
            layers: layer_hashes,
            config,
            metadata,
        })
    }

    /// Create a new image from a directory
    pub async fn create_image(&self, config: &ImageCreationConfig) -> ContainerResult<String> {
        // Validate input
        self.validate_image_config(config)?;

        // Calculate layer hash
        let layer_hash = self.calculate_layer_hash(&config.source_path)?;

        // Create layer directory
        let layer_path = self.image_root.join("layers").join(&layer_hash);
        create_dir_all(&layer_path)
            .map_err(|e| ContainerError::System(format!("Failed to create layer directory: {}", e)))?;

        // Copy layer content
        self.copy_layer_content(&config.source_path, &layer_path).await?;

        // Create image metadata
        let metadata = ImageMetadata {
            name: config.name.clone(),
            tag: config.tag.clone(),
            id: format!("sha256:{}", layer_hash),
            created: SystemTime::now(),
            size: self.calculate_directory_size(&config.source_path)?,
            layers: vec![layer_hash],
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            description: config.description.clone(),
            author: config.author.clone(),
            version: config.version.clone(),
            working_dir: config.working_dir.clone(),
            entrypoint: config.entrypoint.clone(),
            cmd: config.cmd.clone(),
            env: config.env.clone(),
            volumes: config.volumes.clone(),
            exposed_ports: config.exposed_ports.clone(),
            labels: config.labels.clone(),
        };

        // Save metadata
        self.save_image_metadata(&metadata).await?;

        Ok(format!("{}:{}", config.name, config.tag))
    }

    /// Build educational image from template
    pub async fn build_educational_image(&self, template_id: &str, customizations: HashMap<String, String>) -> ContainerResult<String> {
        // Get template configuration
        let template = self.get_template_config(template_id).await?;
        
        // Apply customizations
        let customized_config = self.apply_template_customizations(template, customizations).await?;

        // Create image from customized configuration
        self.create_image(&customized_config).await
    }

    /// Get image history
    pub async fn get_image_history(&self, image_name: &str) -> ContainerResult<Vec<ImageHistoryEntry>> {
        self.load_image_history(image_name).await
    }

    /// Save image metadata
    pub async fn save_image_metadata(&self, metadata: &ImageMetadata) -> ContainerResult<()> {
        let metadata_path = self.image_root.join("images").join(format!("{}.json", metadata.id));
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| ContainerError::System(format!("Failed to serialize metadata: {}", e)))?;
        
        std::fs::write(&metadata_path, content)
            .map_err(|e| ContainerError::System(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    // Private helper methods

    fn parse_image_spec(&self, spec: &str) -> ContainerResult<ImageSpec> {
        let parts: Vec<&str> = spec.split(':').collect();
        
        if parts.len() == 1 {
            Ok(ImageSpec {
                name: parts[0].to_string(),
                tag: "latest".to_string(),
                registry: "docker.io".to_string(),
                namespace: "library".to_string(),
            })
        } else if parts.len() == 2 {
            Ok(ImageSpec {
                name: parts[0].to_string(),
                tag: parts[1].to_string(),
                registry: "docker.io".to_string(),
                namespace: "library".to_string(),
            })
        } else {
            Err(ContainerError::InvalidConfig(format!("Invalid image specification: {}", spec)))
        }
    }

    async fn image_exists(&self, name: &str, tag: &str) -> ContainerResult<bool> {
        let image_id = format!("{}:{}", name, tag);
        let metadata_path = self.image_root.join("images").join(format!("{}.json", image_id));
        Ok(metadata_path.exists())
    }

    async fn is_educational_template(&self, spec: &str) -> bool {
        // Check if the spec starts with "edu:" prefix
        spec.starts_with("edu:") || spec.starts_with("multios:edu-")
    }

    async fn pull_educational_template(&self, template_id: &str) -> ContainerResult<String> {
        // Load educational template configuration
        let template_config = self.get_template_config(template_id).await?;
        
        // Create image from template
        self.create_image(&template_config.into()).await
    }

    async fn pull_from_mirror(&self, image_info: &ImageSpec) -> ContainerResult<()> {
        // This would pull from an educational mirror
        // Simplified implementation - in reality this would involve network operations
        log::info!("Pulling image {}:{} from educational mirror", image_info.name, image_info.tag);
        
        // Create dummy image for demonstration
        let metadata = ImageMetadata {
            name: image_info.name.clone(),
            tag: image_info.tag.clone(),
            id: format!("sha256:{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            created: SystemTime::now(),
            size: 1024 * 1024, // 1MB
            layers: vec![uuid::Uuid::new_v4().to_string().replace("-", "")],
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            description: format!("Educational image for {}", image_info.name),
            author: "MultiOS Educational System".to_string(),
            version: "1.0".to_string(),
            working_dir: Some("/root".to_string()),
            entrypoint: Some(vec!["/bin/sh".to_string()]),
            cmd: None,
            env: vec![],
            volumes: vec![],
            exposed_ports: vec![],
            labels: vec![],
        };

        self.save_image_metadata(&metadata).await?;
        Ok(())
    }

    fn is_image_in_use(&self, image_name: &str) -> ContainerResult<bool> {
        // Check if image is being used by running containers
        // This is a stub implementation
        Ok(false)
    }

    async fn load_image_metadata(&self, image_name: &str) -> ContainerResult<ImageMetadata> {
        let metadata_path = self.image_root.join("images").join(format!("{}.json", image_name));
        
        if !metadata_path.exists() {
            return Err(ContainerError::NotFound(format!("Image {} not found", image_name)));
        }

        let content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| ContainerError::System(format!("Failed to read image metadata: {}", e)))?;

        let metadata: ImageMetadata = serde_json::from_str(&content)
            .map_err(|e| ContainerError::System(format!("Failed to parse image metadata: {}", e)))?;

        Ok(metadata)
    }

    async fn get_layers_info(&self, layer_ids: &[String]) -> ContainerResult<Vec<LayerInfo>> {
        let mut layers_info = Vec::new();

        for layer_id in layer_ids {
            let layer_path = self.image_root.join("layers").join(layer_id);
            
            let metadata_path = layer_path.join("metadata.json");
            if metadata_path.exists() {
                let content = std::fs::read_to_string(&metadata_path)
                    .map_err(|e| ContainerError::System(format!("Failed to read layer metadata: {}", e)))?;
                
                let layer_metadata: LayerMetadata = serde_json::from_str(&content)
                    .map_err(|e| ContainerError::System(format!("Failed to parse layer metadata: {}", e)))?;

                layers_info.push(LayerInfo {
                    id: layer_id.clone(),
                    size: layer_metadata.size,
                    digest: layer_metadata.digest,
                    created: layer_metadata.created,
                    instruction: layer_metadata.instruction,
                });
            }
        }

        Ok(layers_info)
    }

    async fn load_image_config(&self, image_name: &str) -> ContainerResult<ImageConfig> {
        // Load the runtime configuration for the image
        let metadata = self.load_image_metadata(image_name).await?;
        
        Ok(ImageConfig {
            env: metadata.env,
            cmd: metadata.cmd,
            entrypoint: metadata.entrypoint,
            working_dir: metadata.working_dir,
            exposed_ports: metadata.exposed_ports,
            volumes: metadata.volumes,
            labels: metadata.labels,
            hostname: Some(format!("container-{}", image_name.replace(":", "-"))),
            user: None,
            attach_stdin: false,
            attach_stdout: false,
            attach_stderr: false,
            tty: false,
            open_stdin: false,
            stdin_once: false,
        })
    }

    async fn load_image_history(&self, image_name: &str) -> ContainerResult<Vec<ImageHistoryEntry>> {
        // Load image history (layer creation history)
        let history_path = self.image_root.join("images").join(format!("{}-history.json", image_name));
        
        if history_path.exists() {
            let content = std::fs::read_to_string(&history_path)
                .map_err(|e| ContainerError::System(format!("Failed to read image history: {}", e)))?;
            
            let history: Vec<ImageHistoryEntry> = serde_json::from_str(&content)
                .map_err(|e| ContainerError::System(format!("Failed to parse image history: {}", e)))?;

            Ok(history)
        } else {
            Ok(vec![])
        }
    }

    async fn extract_layer(&self, layer_id: &str, target_path: &PathBuf) -> ContainerResult<()> {
        let layer_path = self.image_root.join("layers").join(layer_id);
        let layer_content = layer_path.join("content");

        if layer_content.exists() {
            // Copy layer content to target
            self.copy_directory(&layer_content, target_path).await?;
        }

        Ok(())
    }

    fn calculate_layer_hash(&self, path: &PathBuf) -> ContainerResult<String> {
        // Calculate SHA256 hash of directory
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.hash_directory(path, &mut hasher)?;
        
        Ok(format!("{:x}", hasher.finish()))
    }

    fn hash_directory(&self, path: &PathBuf, hasher: &mut impl Hasher) -> ContainerResult<()> {
        for entry in std::fs::read_dir(path)
            .map_err(|e| ContainerError::System(format!("Failed to read directory: {}", e)))? {
            
            let entry = entry.map_err(|e| ContainerError::System(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            // Hash the file path
            path.hash(hasher);
            
            if path.is_file() {
                let mut file = File::open(&path)
                    .map_err(|e| ContainerError::System(format!("Failed to open file: {}", e)))?;
                
                let mut buffer = vec![0; 4096];
                loop {
                    let count = file.read(&mut buffer)
                        .map_err(|e| ContainerError::System(format!("Failed to read file: {}", e)))?;
                    if count == 0 {
                        break;
                    }
                    hasher.write(&buffer[..count]);
                }
            } else if path.is_dir() {
                self.hash_directory(&path, hasher)?;
            }
        }
        
        Ok(())
    }

    async fn copy_layer_content(&self, source: &PathBuf, target: &PathBuf) -> ContainerResult<()> {
        self.copy_directory(source, target).await
    }

    async fn copy_directory(&self, source: &PathBuf, target: &PathBuf) -> ContainerResult<()> {
        if source.is_file() {
            if let Some(parent) = target.parent() {
                create_dir_all(parent)
                    .map_err(|e| ContainerError::System(format!("Failed to create parent directory: {}", e)))?;
            }
            
            std::fs::copy(source, target)
                .map_err(|e| ContainerError::System(format!("Failed to copy file: {}", e)))?;
        } else if source.is_dir() {
            create_dir_all(target)
                .map_err(|e| ContainerError::System(format!("Failed to create directory: {}", e)))?;
            
            for entry in std::fs::read_dir(source)
                .map_err(|e| ContainerError::System(format!("Failed to read source directory: {}", e)))? {
                
                let entry = entry.map_err(|e| ContainerError::System(format!("Failed to read directory entry: {}", e)))?;
                let source_path = entry.path();
                let target_path = target.join(source_path.file_name().unwrap());
                
                self.copy_directory(&source_path, &target_path).await?;
            }
        }
        
        Ok(())
    }

    fn calculate_directory_size(&self, path: &PathBuf) -> ContainerResult<u64> {
        let mut total_size = 0u64;
        
        if path.is_file() {
            total_size = std::fs::metadata(path)
                .map_err(|e| ContainerError::System(format!("Failed to get file metadata: {}", e)))?
                .len();
        } else if path.is_dir() {
            for entry in std::fs::read_dir(path)
                .map_err(|e| ContainerError::System(format!("Failed to read directory: {}", e)))? {
                
                let entry = entry.map_err(|e| ContainerError::System(format!("Failed to read directory entry: {}", e)))?;
                let path = entry.path();
                
                total_size += self.calculate_directory_size(&path)?;
            }
        }
        
        Ok(total_size)
    }

    async fn generate_container_config(&self, image_name: &str, metadata: &ImageMetadata) -> ContainerResult<ContainerConfig> {
        Ok(ContainerConfig {
            container_id: uuid::Uuid::new_v4().to_string(),
            name: format!("container-{}", image_name.replace(":", "-")),
            image: image_name.to_string(),
            command: metadata.cmd.clone().unwrap_or_default(),
            environment: metadata.env.iter().cloned().collect(),
            ports: vec![],
            volumes: vec![],
            resource_limits: ResourceLimits::default(),
            security: SecurityConfig::default(),
            network: NetworkConfig::default(),
            namespace_mode: NamespaceMode::default(),
            template_id: None,
            created_at: Utc::now(),
        })
    }

    fn validate_image_config(&self, config: &ImageCreationConfig) -> ContainerResult<()> {
        if config.name.is_empty() {
            return Err(ContainerError::InvalidConfig("Image name cannot be empty".to_string()));
        }
        
        if config.source_path.is_empty() {
            return Err(ContainerError::InvalidConfig("Source path cannot be empty".to_string()));
        }

        if !config.source_path.exists() {
            return Err(ContainerError::InvalidConfig("Source path does not exist".to_string()));
        }

        Ok(())
    }

    async fn remove_layer(&self, layer_id: &str) -> ContainerResult<()> {
        let layer_path = self.image_root.join("layers").join(layer_id);
        if layer_path.exists() {
            std::fs::remove_dir_all(&layer_path)
                .map_err(|e| ContainerError::System(format!("Failed to remove layer: {}", e)))?;
        }

        // Remove from cache
        {
            let mut cache = self.layer_cache.lock().unwrap();
            cache.remove(layer_id);
        }

        Ok(())
    }

    async fn get_template_config(&self, template_id: &str) -> Result<ImageCreationConfig, ContainerError> {
        // Load educational template configuration
        // This is a simplified implementation
        let templates_path = PathBuf::from("/etc/multios/container-templates");
        
        let template_file = templates_path.join(format!("{}.json", template_id));
        
        if !template_file.exists() {
            return Err(ContainerError::TemplateError(format!("Template {} not found", template_id)));
        }

        let content = std::fs::read_to_string(&template_file)
            .map_err(|e| ContainerError::System(format!("Failed to read template: {}", e)))?;

        let template_config: ImageCreationConfig = serde_json::from_str(&content)
            .map_err(|e| ContainerError::System(format!("Failed to parse template: {}", e)))?;

        Ok(template_config)
    }

    async fn apply_template_customizations(&self, mut template: ImageCreationConfig, 
                                        customizations: HashMap<String, String>) -> ContainerResult<ImageCreationConfig> {
        // Apply customizations to template
        for (key, value) in customizations {
            match key.as_str() {
                "name" => template.name = value,
                "tag" => template.tag = value,
                "description" => template.description = value,
                "entrypoint" => template.entrypoint = Some(vec![value]),
                _ => {
                    log::warn!("Unknown customization key: {}", key);
                }
            }
        }

        Ok(template)
    }
}

/// Image specification
#[derive(Debug, Clone)]
pub struct ImageSpec {
    pub name: String,
    pub tag: String,
    pub registry: String,
    pub namespace: String,
}

/// Image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub name: String,
    pub tag: String,
    pub id: String,
    pub created: SystemTime,
    pub size: u64,
    pub layers: Vec<String>,
    pub architecture: String,
    pub os: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub working_dir: Option<String>,
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub env: Vec<(String, String)>,
    pub volumes: Vec<String>,
    pub exposed_ports: Vec<u16>,
    pub labels: HashMap<String, String>,
}

/// Layer metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetadata {
    pub size: u64,
    pub digest: String,
    pub created: SystemTime,
    pub instruction: String,
}

/// Image information for listing
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub size: u64,
    pub created: SystemTime,
    pub layers: usize,
    pub description: String,
}

/// Detailed image information
#[derive(Debug, Clone)]
pub struct ImageDetails {
    pub metadata: ImageMetadata,
    pub layers: Vec<LayerInfo>,
    pub configuration: ImageConfig,
    pub history: Vec<ImageHistoryEntry>,
}

/// Layer information
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub id: String,
    pub size: u64,
    pub digest: String,
    pub created: SystemTime,
    pub instruction: String,
}

/// Image runtime configuration
#[derive(Debug, Clone)]
pub struct ImageConfig {
    pub env: Vec<(String, String)>,
    pub cmd: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub exposed_ports: Vec<u16>,
    pub volumes: Vec<String>,
    pub labels: HashMap<String, String>,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub tty: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
}

/// Image creation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCreationConfig {
    pub name: String,
    pub tag: String,
    pub source_path: PathBuf,
    pub description: String,
    pub author: String,
    pub version: String,
    pub working_dir: Option<String>,
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub env: Vec<(String, String)>,
    pub volumes: Vec<String>,
    pub exposed_ports: Vec<u16>,
    pub labels: HashMap<String, String>,
}

impl Default for ImageCreationConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            tag: "latest".to_string(),
            source_path: PathBuf::new(),
            description: String::new(),
            author: "MultiOS System".to_string(),
            version: "1.0".to_string(),
            working_dir: Some("/".to_string()),
            entrypoint: None,
            cmd: None,
            env: vec![],
            volumes: vec![],
            exposed_ports: vec![],
            labels: HashMap::new(),
        }
    }
}

/// Image history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageHistoryEntry {
    pub id: String,
    pub created: SystemTime,
    pub size: u64,
    pub instruction: String,
    pub comment: Option<String>,
}

/// Extracted image information
#[derive(Debug, Clone)]
pub struct ExtractedImage {
    pub rootfs_path: PathBuf,
    pub layers: Vec<String>,
    pub config: ContainerConfig,
    pub metadata: ImageMetadata,
}

/// Container image layer
#[derive(Debug, Clone)]
pub struct ImageLayer {
    pub id: String,
    pub path: PathBuf,
    pub metadata: LayerMetadata,
    pub created_at: SystemTime,
}