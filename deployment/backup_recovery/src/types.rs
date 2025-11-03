use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Backup types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    /// Full system backup
    Full,
    /// Incremental backup (changes since last backup)
    Incremental,
    /// Differential backup (changes since last full backup)
    Differential,
    /// File-level backup only
    FileLevel,
    /// Partition-level backup
    PartitionLevel,
}

/// Compression algorithms available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// GZIP compression
    Gzip,
    /// LZ4 compression
    Lz4,
    /// Zstandard compression
    Zstd,
}

/// Encryption settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSettings {
    /// Enable encryption
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: String,
    /// Key derivation method
    pub key_derivation: String,
    /// Salt for key derivation
    pub salt: Option<Vec<u8>>,
}

/// Storage locations for backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    /// Unique identifier
    pub id: String,
    /// Storage type
    pub storage_type: StorageType,
    /// Connection string or path
    pub path: String,
    /// Additional configuration
    pub config: HashMap<String, String>,
    /// Whether this is the default location
    pub is_default: bool,
}

/// Supported storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// Local filesystem
    Local,
    /// Network attached storage
    Network,
    /// Amazon S3
    AmazonS3,
    /// Google Cloud Storage
    GoogleCloud,
    /// Azure Blob Storage
    AzureBlob,
    /// FTP server
    Ftp,
    /// SFTP server
    Sftp,
}

/// Backup job specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSpecification {
    /// Unique job identifier
    pub job_id: Uuid,
    /// Job name
    pub name: String,
    /// Backup type
    pub backup_type: BackupType,
    /// Source paths to backup
    pub sources: Vec<PathBuf>,
    /// Destination storage location
    pub destination: StorageLocation,
    /// Compression settings
    pub compression: CompressionAlgorithm,
    /// Encryption settings
    pub encryption: EncryptionSettings,
    /// Backup description
    pub description: Option<String>,
    /// Custom tags
    pub tags: HashMap<String, String>,
    /// Whether to create verification data
    pub verify_integrity: bool,
    /// Whether to create bootable media
    pub create_recovery_media: bool,
}

/// Backup job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupJobStatus {
    /// Job is queued
    Queued,
    /// Job is running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job is paused
    Paused,
}

/// Backup job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    /// Unique job identifier
    pub job_id: String,
    /// Job specification
    pub specification: BackupSpecification,
    /// Current status
    pub status: BackupJobStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last status change
    pub status_changed_at: DateTime<Utc>,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current phase
    pub phase: String,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Backup size in bytes
    pub size_bytes: u64,
    /// Number of files processed
    pub files_processed: u64,
    /// Processing rate (bytes/sec)
    pub rate_bytes_per_sec: u64,
}

/// Restore specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSpecification {
    /// Unique job identifier
    pub job_id: Uuid,
    /// Backup to restore from
    pub backup_id: String,
    /// Target location for restore
    pub target_path: PathBuf,
    /// Specific files to restore (empty = restore all)
    pub include_paths: Vec<PathBuf>,
    /// Files to exclude
    pub exclude_paths: Vec<PathBuf>,
    /// Point in time to restore to
    pub point_in_time: Option<DateTime<Utc>>,
    /// Whether to verify restore
    pub verify_restore: bool,
    /// Restore permissions
    pub restore_permissions: bool,
    /// Restore ownership
    pub restore_ownership: bool,
}

/// Restore job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RestoreJobStatus {
    /// Job is queued
    Queued,
    /// Job is running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job is paused
    Paused,
}

/// Restore job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreJob {
    /// Unique job identifier
    pub job_id: String,
    /// Job specification
    pub specification: RestoreSpecification,
    /// Current status
    pub status: RestoreJobStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last status change
    pub status_changed_at: DateTime<Utc>,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Current phase
    pub phase: String,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Files restored
    pub files_restored: u64,
    /// Data restored in bytes
    pub bytes_restored: u64,
}

/// Backup verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Backup identifier
    pub backup_id: String,
    /// Verification status
    pub status: VerificationStatus,
    /// Verification timestamp
    pub verified_at: DateTime<Utc>,
    /// Files verified
    pub files_verified: u64,
    /// Files with issues
    pub files_failed: u64,
    /// Integrity check details
    pub integrity_checks: Vec<IntegrityCheck>,
    /// Overall assessment
    pub assessment: String,
}

/// Verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// All checks passed
    Passed,
    /// Some checks failed
    Failed,
    /// Verification timed out
    Timeout,
    /// Unable to verify
    Error,
}

/// Integrity check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheck {
    /// Check type
    pub check_type: String,
    /// Status
    pub status: VerificationStatus,
    /// Details
    pub details: String,
}

/// Backup retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Policy name
    pub name: String,
    /// Keep daily backups for N days
    pub keep_daily: u32,
    /// Keep weekly backups for N weeks
    pub keep_weekly: u32,
    /// Keep monthly backups for N months
    pub keep_monthly: u32,
    /// Keep yearly backups for N years
    pub keep_yearly: u32,
    /// Whether to keep the last N backups regardless of age
    pub keep_last_n: Option<u32>,
}

/// Scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Schedule name
    pub name: String,
    /// Cron expression
    pub cron_expression: String,
    /// Backup specification reference
    pub backup_spec_id: String,
    /// Whether schedule is enabled
    pub enabled: bool,
    /// Next run time
    pub next_run: Option<DateTime<Utc>>,
    /// Last run time
    pub last_run: Option<DateTime<Utc>>,
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration version
    pub version: String,
    /// Default storage location
    pub default_storage: StorageLocation,
    /// Retention policies
    pub retention_policies: Vec<RetentionPolicy>,
    /// Scheduled backups
    pub schedules: Vec<ScheduleConfig>,
    /// Global settings
    pub global_settings: GlobalSettings,
    /// Educational lab profiles
    pub lab_profiles: Vec<LabProfile>,
}

/// Global system settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Maximum concurrent backup jobs
    pub max_concurrent_backups: u32,
    /// Maximum concurrent restore jobs
    pub max_concurrent_restores: u32,
    /// Default compression algorithm
    pub default_compression: CompressionAlgorithm,
    /// Default encryption settings
    pub default_encryption: EncryptionSettings,
    /// Backup directory
    pub backup_directory: PathBuf,
    /// Temp directory
    pub temp_directory: PathBuf,
    /// Log directory
    pub log_directory: PathBuf,
    /// Log level
    pub log_level: String,
    /// Network timeout (seconds)
    pub network_timeout: u64,
    /// Verification timeout (seconds)
    pub verification_timeout: u64,
}

/// Educational lab profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabProfile {
    /// Profile identifier
    pub id: String,
    /// Profile name
    pub name: String,
    /// Profile description
    pub description: String,
    /// Default backup sources
    pub default_sources: Vec<PathBuf>,
    /// Default retention policy
    pub default_retention: String,
    /// Schedule settings
    pub schedule_settings: ScheduleConfig,
    /// Custom configuration
    pub custom_config: HashMap<String, String>,
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// System version
    pub version: String,
    /// System uptime
    pub uptime: std::time::SystemTime,
    /// Configuration version
    pub config_version: String,
    /// Number of active backup jobs
    pub active_backups: u32,
    /// Number of active restore jobs
    pub active_restores: u32,
}

/// Network backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBackupConfig {
    /// Remote server address
    pub server_address: String,
    /// Port number
    pub port: u16,
    /// Protocol to use
    pub protocol: String,
    /// Authentication credentials
    pub credentials: NetworkCredentials,
    /// Connection timeout
    pub timeout: u64,
    /// Retry attempts
    pub retry_attempts: u32,
}

/// Network authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCredentials {
    /// Username
    pub username: String,
    /// Password (encrypted)
    pub password: String,
    /// SSH key path (if using key auth)
    pub ssh_key_path: Option<PathBuf>,
    /// Certificate path (if using cert auth)
    pub certificate_path: Option<PathBuf>,
}

/// Cloud backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudBackupConfig {
    /// Cloud provider
    pub provider: CloudProvider,
    /// Access credentials
    pub credentials: CloudCredentials,
    /// Bucket/container name
    pub bucket_name: String,
    /// Region
    pub region: String,
    /// Storage class
    pub storage_class: String,
}

/// Cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    AmazonWebServices,
    GoogleCloudPlatform,
    MicrosoftAzure,
    DigitalOcean,
}

/// Cloud credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCredentials {
    /// Access key ID
    pub access_key_id: String,
    /// Secret access key
    pub secret_access_key: String,
    /// Session token (for temporary credentials)
    pub session_token: Option<String>,
}

/// Quick restore options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickRestoreOptions {
    /// Restore type
    pub restore_type: QuickRestoreType,
    /// Target location
    pub target_path: PathBuf,
    /// Force overwrite existing files
    pub force_overwrite: bool,
    /// Verify after restore
    pub verify_after: bool,
}

/// Quick restore types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuickRestoreType {
    /// Restore corrupted system files
    SystemFiles,
    /// Restore driver files
    Drivers,
    /// Restore user documents
    UserDocuments,
    /// Restore application data
    ApplicationData,
    /// Restore configuration files
    ConfigurationFiles,
}

impl Default for Config {
    fn default() -> Self {
        let default_storage = StorageLocation {
            id: "local-default".to_string(),
            storage_type: StorageType::Local,
            path: "/var/lib/multios/backup".to_string(),
            config: HashMap::new(),
            is_default: true,
        };
        
        let global_settings = GlobalSettings {
            max_concurrent_backups: 4,
            max_concurrent_restores: 2,
            default_compression: CompressionAlgorithm::Zstd,
            default_encryption: EncryptionSettings {
                enabled: false,
                algorithm: "AES-256".to_string(),
                key_derivation: "PBKDF2".to_string(),
                salt: None,
            },
            backup_directory: PathBuf::from("/var/lib/multios/backup"),
            temp_directory: PathBuf::from("/tmp/multios-backup"),
            log_directory: PathBuf::from("/var/log/multios/backup"),
            log_level: "INFO".to_string(),
            network_timeout: 300,
            verification_timeout: 3600,
        };
        
        Self {
            version: "1.0.0".to_string(),
            default_storage,
            retention_policies: vec![],
            schedules: vec![],
            global_settings,
            lab_profiles: vec![],
        }
    }
}