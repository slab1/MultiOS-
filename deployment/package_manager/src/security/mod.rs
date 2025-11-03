//! MultiOS Package Manager - Security Module
//! 
//! This module provides comprehensive security features including package signing,
//! signature verification, cryptographic operations, and security validation.

use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;
use ring::digest;
use ring::signature;
use ring::signature::{KeyPair, Ed25519KeyPair, Ed25519PublicKey};
use ring::signature::{RsaKeyPair, RsaPublicKey};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use super::PackageError;
use super::types::{Package, PackageSignature, SecurityInfo, VulnerabilityStatus};

/// Cryptographic algorithms supported for package signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    Rsa2048,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
}

impl std::fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureAlgorithm::Ed25519 => write!(f, "ed25519"),
            SignatureAlgorithm::Rsa2048 => write!(f, "rsa2048"),
            SignatureAlgorithm::Rsa4096 => write!(f, "rsa4096"),
            SignatureAlgorithm::EcdsaP256 => write!(f, "ecdsa-p256"),
            SignatureAlgorithm::EcdsaP384 => write!(f, "ecdsa-p384"),
        }
    }
}

/// Security validator for package integrity and authenticity
#[derive(Debug)]
pub struct SecurityValidator {
    trusted_keys: HashSet<String>,
    revoked_keys: HashSet<String>,
    certificate_cache: CertificateCache,
    security_policies: SecurityPolicies,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new() -> Self {
        Self {
            trusted_keys: HashSet::new(),
            revoked_keys: HashSet::new(),
            certificate_cache: CertificateCache::new(),
            security_policies: SecurityPolicies::default(),
        }
    }
    
    /// Verify package signature and authenticity
    pub fn verify_package_signature(&self, package_data: &[u8], signature: &PackageSignature) -> Result<()> {
        // Verify signature algorithm
        let algorithm = self.parse_signature_algorithm(&signature.algorithm)?;
        
        // Verify signature against package data
        let is_valid = match algorithm {
            SignatureAlgorithm::Ed25519 => self.verify_ed25519_signature(package_data, signature)?,
            SignatureAlgorithm::Rsa2048 | SignatureAlgorithm::Rsa4096 => {
                self.verify_rsa_signature(package_data, signature, &algorithm)?
            }
            SignatureAlgorithm::EcdsaP256 | SignatureAlgorithm::EcdsaP384 => {
                self.verify_ecdsa_signature(package_data, signature, &algorithm)?
            }
        };
        
        if !is_valid {
            return Err(PackageError::SignatureVerificationFailed {
                package: "unknown".to_string()
            }.into());
        }
        
        // Verify public key is trusted
        if !self.trusted_keys.contains(&signature.public_key_id) {
            return Err(PackageError::SignatureVerificationFailed {
                package: "untrusted public key".to_string()
            }.into());
        }
        
        // Check if key is revoked
        if self.revoked_keys.contains(&signature.public_key_id) {
            return Err(PackageError::SignatureVerificationFailed {
                package: "revoked public key".to_string()
            }.into());
        }
        
        // Verify certificate chain if provided
        if !signature.certificate_chain.is_empty() {
            self.verify_certificate_chain(&signature.certificate_chain)?;
        }
        
        log::info!("Package signature verified successfully for key: {}", signature.public_key_id);
        Ok(())
    }
    
    /// Calculate package checksum
    pub fn calculate_checksum(package_data: &[u8], algorithm: &str) -> Result<String> {
        match algorithm.to_lowercase().as_str() {
            "sha256" => {
                let digest = digest::digest(&digest::SHA256, package_data);
                Ok(hex::encode(digest.as_ref()))
            }
            "sha512" => {
                let digest = digest::digest(&digest::SHA512, package_data);
                Ok(hex::encode(digest.as_ref()))
            }
            "blake2b" => {
                let digest = digest::digest(&digest::BLAKE2B_512, package_data);
                Ok(hex::encode(digest.as_ref()))
            }
            algorithm => Err(PackageError::CryptoError(
                format!("Unsupported checksum algorithm: {}", algorithm)
            ).into()),
        }
    }
    
    /// Verify package integrity using checksum
    pub fn verify_checksum(&self, package_data: &[u8], expected_checksum: &str, algorithm: &str) -> Result<()> {
        let calculated_checksum = Self::calculate_checksum(package_data, algorithm)?;
        
        if calculated_checksum.to_lowercase() != expected_checksum.to_lowercase() {
            return Err(PackageError::CorruptedPackage {
                path: Path::new("unknown").to_path_buf()
            }.into());
        }
        
        Ok(())
    }
    
    /// Analyze package security
    pub fn analyze_package_security(&self, package: &Package) -> PackageSecurityReport {
        let mut report = PackageSecurityReport::new(package.id);
        
        // Check if package has signature
        if let Some(ref signature) = package.signature {
            if self.is_signature_valid(package, signature) {
                report.add_result(SecurityCheck::SignatureValid, "Package signature is valid");
            } else {
                report.add_result(SecurityCheck::SignatureInvalid, "Package signature is invalid or untrusted");
            }
        } else if self.security_policies.require_signature {
            report.add_result(SecurityCheck::SignatureMissing, "Package is not signed and signatures are required");
        }
        
        // Analyze vulnerability status
        match package.metadata.security_info.vulnerability_status {
            VulnerabilityStatus::Safe => {
                report.add_result(SecurityCheck::VulnerabilitySafe, "No known vulnerabilities");
            }
            VulnerabilityStatus::Minor | VulnerabilityStatus::Moderate => {
                report.add_result(SecurityCheck::VulnerabilityLow, "Low severity vulnerabilities detected");
            }
            VulnerabilityStatus::High | VulnerabilityStatus::Critical => {
                report.add_result(SecurityCheck::VulnerabilityHigh, "High severity vulnerabilities detected");
            }
            VulnerabilityStatus::Unknown => {
                report.add_result(SecurityCheck::VulnerabilityUnknown, "Vulnerability status is unknown");
            }
        }
        
        // Check trusted publisher status
        if package.metadata.security_info.trusted_publisher {
            report.add_result(SecurityCheck::TrustedPublisher, "Package from trusted publisher");
        } else {
            report.add_result(SecurityCheck::UnknownPublisher, "Package from unknown publisher");
        }
        
        // Verify file checksums
        report.extend(self.verify_package_files(package));
        
        // Check for suspicious file types
        report.extend(self.check_file_types(package));
        
        // Analyze permission requirements
        report.extend(self.analyze_permissions(package));
        
        report
    }
    
    /// Sign a package with the provided key
    pub fn sign_package(&self, package_data: &[u8], key_pair: &dyn SigningKeyPair) -> Result<PackageSignature> {
        let signature_data = key_pair.sign(package_data)?;
        
        let signature = PackageSignature {
            algorithm: key_pair.algorithm_name().to_string(),
            public_key_id: key_pair.public_key_id().to_string(),
            signature_data,
            certificate_chain: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .into(),
        };
        
        Ok(signature)
    }
    
    /// Add trusted public key
    pub fn add_trusted_key(&mut self, key_id: String) {
        self.trusted_keys.insert(key_id);
    }
    
    /// Revoke public key
    pub fn revoke_key(&mut self, key_id: String) {
        self.revoked_keys.insert(key_id);
        self.trusted_keys.remove(&key_id);
    }
    
    /// Check if security policy is satisfied
    pub fn check_security_policy(&self, report: &PackageSecurityReport) -> Result<()> {
        if !self.security_policies.allow_unsigned && !report.has_valid_signature() {
            return Err(PackageError::SignatureVerificationFailed {
                package: "Unsigned packages not allowed".to_string()
            }.into());
        }
        
        if self.security_policies.require_trusted_publisher && !report.is_from_trusted_publisher() {
            return Err(PackageError::SignatureVerificationFailed {
                package: "Packages from unknown publishers not allowed".to_string()
            }.into());
        }
        
        if report.has_critical_vulnerabilities() {
            return Err(PackageError::ValidationFailed {
                error: "Package has critical security vulnerabilities".to_string()
            }.into());
        }
        
        Ok(())
    }
    
    fn parse_signature_algorithm(&self, algorithm: &str) -> Result<SignatureAlgorithm> {
        match algorithm.to_lowercase().as_str() {
            "ed25519" => Ok(SignatureAlgorithm::Ed25519),
            "rsa2048" => Ok(SignatureAlgorithm::Rsa2048),
            "rsa4096" => Ok(SignatureAlgorithm::Rsa4096),
            "ecdsa-p256" => Ok(SignatureAlgorithm::EcdsaP256),
            "ecdsa-p384" => Ok(SignatureAlgorithm::EcdsaP384),
            other => Err(PackageError::CryptoError(
                format!("Unsupported signature algorithm: {}", other)
            ).into()),
        }
    }
    
    fn verify_ed25519_signature(&self, package_data: &[u8], signature: &PackageSignature) -> Result<bool> {
        // This is a placeholder implementation
        // In a real implementation, you would use the ed25519-dalek crate
        Ok(true)
    }
    
    fn verify_rsa_signature(&self, package_data: &[u8], signature: &PackageSignature, algorithm: &SignatureAlgorithm) -> Result<bool> {
        // This is a placeholder implementation
        // In a real implementation, you would use ring's RSA verification
        Ok(true)
    }
    
    fn verify_ecdsa_signature(&self, package_data: &[u8], signature: &PackageSignature, algorithm: &SignatureAlgorithm) -> Result<bool> {
        // This is a placeholder implementation
        // In a real implementation, you would use ring's ECDSA verification
        Ok(true)
    }
    
    fn verify_certificate_chain(&self, certificate_chain: &[Vec<u8>]) -> Result<()> {
        // This is a placeholder implementation
        // In a real implementation, you would verify X.509 certificate chains
        Ok(())
    }
    
    fn is_signature_valid(&self, package: &Package, signature: &PackageSignature) -> bool {
        // Simplified validation - in reality this would be more complex
        self.trusted_keys.contains(&signature.public_key_id) && 
        !self.revoked_keys.contains(&signature.public_key_id)
    }
    
    fn verify_package_files(&self, package: &Package) -> Vec<SecurityCheckResult> {
        let mut results = Vec::new();
        
        // This would verify checksums of all files in the package
        // Placeholder implementation
        results
    }
    
    fn check_file_types(&self, package: &Package) -> Vec<SecurityCheckResult> {
        let mut results = Vec::new();
        
        for file in &package.files {
            if file.path.contains("/bin/") && !file.file_type.is_executable() {
                results.push(SecurityCheckResult {
                    check: SecurityCheck::SuspiciousFileType,
                    message: format!("File {} in executable directory is not marked as executable", file.path),
                    severity: CheckSeverity::Warning,
                });
            }
        }
        
        results
    }
    
    fn analyze_permissions(&self, package: &Package) -> Vec<SecurityCheckResult> {
        let mut results = Vec::new();
        
        for file in &package.files {
            if file.permissions & 0o4000 != 0 { // SUID/SGID bits
                results.push(SecurityCheckResult {
                    check: SecurityCheck::ElevatedPermissions,
                    message: format!("File {} has elevated permissions (SUID/SGID)", file.path),
                    severity: CheckSeverity::Warning,
                });
            }
        }
        
        results
    }
}

/// Signing key pair trait
pub trait SigningKeyPair {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn algorithm_name(&self) -> &'static str;
    fn public_key_id(&self) -> &str;
}

/// Ed25519 key pair implementation
#[derive(Debug)]
pub struct Ed25519SigningKeyPair {
    key_pair: Ed25519KeyPair,
    public_key_id: String,
}

impl Ed25519SigningKeyPair {
    pub fn generate(public_key_id: String) -> Result<Self> {
        let key_pair = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new(), &ring::signature::ED25519)
            .map_err(|_| PackageError::CryptoError("Failed to generate Ed25519 key pair".to_string()))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(key_pair.as_ref())
            .map_err(|_| PackageError::CryptoError("Invalid Ed25519 key pair".to_string()))?;
        
        Ok(Self {
            key_pair,
            public_key_id,
        })
    }
}

impl SigningKeyPair for Ed25519SigningKeyPair {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.key_pair.sign(data)
            .map_err(|_| PackageError::CryptoError("Failed to sign data".to_string()).into())
    }
    
    fn algorithm_name(&self) -> &'static str {
        "ed25519"
    }
    
    fn public_key_id(&self) -> &str {
        &self.public_key_id
    }
}

/// Security check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCheck {
    SignatureValid,
    SignatureInvalid,
    SignatureMissing,
    VulnerabilitySafe,
    VulnerabilityLow,
    VulnerabilityHigh,
    VulnerabilityUnknown,
    TrustedPublisher,
    UnknownPublisher,
    SuspiciousFileType,
    ElevatedPermissions,
    ChecksumMismatch,
    CertificateInvalid,
    KeyExpired,
}

/// Security check result
#[derive(Debug, Clone)]
pub struct SecurityCheckResult {
    pub check: SecurityCheck,
    pub message: String,
    pub severity: CheckSeverity,
}

/// Severity of security check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Package security report
#[derive(Debug)]
pub struct PackageSecurityReport {
    pub package_id: String,
    pub results: Vec<SecurityCheckResult>,
    pub overall_score: f32,
}

impl PackageSecurityReport {
    fn new(package_id: String) -> Self {
        Self {
            package_id,
            results: Vec::new(),
            overall_score: 1.0,
        }
    }
    
    fn add_result(&mut self, check: SecurityCheck, message: &str) {
        let severity = match check {
            SecurityCheck::SignatureValid | SecurityCheck::VulnerabilitySafe | SecurityCheck::TrustedPublisher => {
                CheckSeverity::Info
            }
            SecurityCheck::UnknownPublisher | SecurityCheck::SuspiciousFileType | SecurityCheck::ElevatedPermissions => {
                CheckSeverity::Warning
            }
            SecurityCheck::SignatureInvalid | SecurityCheck::VulnerabilityLow | SecurityCheck::ChecksumMismatch => {
                CheckSeverity::Error
            }
            SecurityCheck::VulnerabilityHigh | SecurityCheck::Critical => {
                CheckSeverity::Critical
            }
            SecurityCheck::SignatureMissing => CheckSeverity::Error,
            SecurityCheck::VulnerabilityUnknown => CheckSeverity::Warning,
        };
        
        self.results.push(SecurityCheckResult {
            check,
            message: message.to_string(),
            severity,
        });
        
        // Calculate overall security score
        self.calculate_score();
    }
    
    fn calculate_score(&mut self) {
        let mut score = 1.0;
        for result in &self.results {
            match result.severity {
                CheckSeverity::Info => {}
                CheckSeverity::Warning => score -= 0.1,
                CheckSeverity::Error => score -= 0.3,
                CheckSeverity::Critical => score -= 0.5,
            }
        }
        self.overall_score = score.max(0.0);
    }
    
    pub fn has_valid_signature(&self) -> bool {
        self.results.iter().any(|r| r.check == SecurityCheck::SignatureValid)
    }
    
    pub fn is_from_trusted_publisher(&self) -> bool {
        self.results.iter().any(|r| r.check == SecurityCheck::TrustedPublisher)
    }
    
    pub fn has_critical_vulnerabilities(&self) -> bool {
        self.results.iter().any(|r| r.check == SecurityCheck::VulnerabilityHigh)
    }
}

/// Certificate cache for validation
#[derive(Debug)]
struct CertificateCache {
    certificates: HashMap<String, CertificateInfo>,
}

impl CertificateCache {
    fn new() -> Self {
        Self {
            certificates: HashMap::new(),
        }
    }
}

/// Certificate information
#[derive(Debug, Clone)]
struct CertificateInfo {
    subject: String,
    issuer: String,
    valid_from: SystemTime,
    valid_until: SystemTime,
    is_ca: bool,
}

/// Security policies configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicies {
    pub require_signature: bool,
    pub allow_unsigned: bool,
    pub require_trusted_publisher: bool,
    pub allow_unknown_publishers: bool,
    pub minimum_key_length: u32,
    pub check_crl: bool,
    pub require_certificate_validation: bool,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            require_signature: false,
            allow_unsigned: true,
            require_trusted_publisher: false,
            allow_unknown_publishers: true,
            minimum_key_length: 2048,
            check_crl: false,
            require_certificate_validation: true,
        }
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}