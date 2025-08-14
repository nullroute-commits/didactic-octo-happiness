//! # Certificate Management System
//!
//! This module provides comprehensive cryptographic certificate management including:
//! - X.509 certificate generation, validation, and lifecycle management
//! - Secure cryptographic key pair generation and storage
//! - Certificate Authority (CA) operations
//! - TLS/SSL certificate provisioning for secure communications
//! - Certificate revocation and renewal automation
//! - Support for modern cryptographic algorithms and secure protocols

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};

/// Algorithm compliance levels for security standards
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlgorithmCompliance {
    /// Deprecated - should not be used
    Deprecated,
    /// Standard security level - acceptable for most use cases
    Standard,
    /// High security level - recommended for sensitive applications
    HighSecurity,
    /// Post-quantum resistant - future-proof algorithms
    PostQuantumResistant,
}

/// Supported cryptographic algorithms for certificate generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CryptoAlgorithm {
    /// RSA with configurable key sizes (2048, 3072, 4096 bits)
    Rsa { key_size: u32 },
    /// Elliptic Curve Digital Signature Algorithm (P-256, P-384, P-521)
    EcdsaP256,
    EcdsaP384,
    EcdsaP521,
    /// Ed25519 - Modern elliptic curve signature algorithm
    Ed25519,
    /// X25519 - Elliptic curve key exchange
    X25519,
}

impl CryptoAlgorithm {
    /// Get recommended algorithms based on security level
    pub fn recommended_for_security_level(level: SecurityLevel) -> Vec<Self> {
        match level {
            SecurityLevel::Standard => vec![
                Self::EcdsaP256,
                Self::Rsa { key_size: 2048 },
            ],
            SecurityLevel::High => vec![
                Self::EcdsaP384,
                Self::Rsa { key_size: 3072 },
                Self::Ed25519,
            ],
            SecurityLevel::Maximum => vec![
                Self::EcdsaP521,
                Self::Rsa { key_size: 4096 },
                Self::Ed25519,
            ],
        }
    }

    /// Check if algorithm is considered secure as of 2025
    /// Based on NIST and industry security standards
    pub fn is_secure(&self) -> bool {
        match self {
            // RSA: Minimum 2048 bits required (NIST SP 800-57 Part 1)
            Self::Rsa { key_size } => *key_size >= 2048,
            // ECDSA: All P-curves are secure (FIPS 186-5)
            Self::EcdsaP256 | Self::EcdsaP384 | Self::EcdsaP521 => true,
            // Modern elliptic curve algorithms (RFC 8032, RFC 7748)
            Self::Ed25519 | Self::X25519 => true,
        }
    }

    /// Check if algorithm should be deprecated soon (post-quantum considerations)
    pub fn is_post_quantum_resistant(&self) -> bool {
        // Note: Current algorithms are NOT post-quantum resistant
        // This method is prepared for future PQC algorithm support
        false
    }

    /// Get algorithm compliance status for various standards
    pub fn compliance_status(&self) -> AlgorithmCompliance {
        match self {
            Self::Rsa { key_size } => {
                if *key_size >= 4096 {
                    AlgorithmCompliance::HighSecurity
                } else if *key_size >= 2048 {
                    AlgorithmCompliance::Standard
                } else {
                    AlgorithmCompliance::Deprecated
                }
            }
            Self::EcdsaP256 => AlgorithmCompliance::Standard,
            Self::EcdsaP384 | Self::EcdsaP521 => AlgorithmCompliance::HighSecurity,
            Self::Ed25519 | Self::X25519 => AlgorithmCompliance::HighSecurity,
        }
    }

    /// Get algorithm strength in bits
    pub fn security_bits(&self) -> u32 {
        match self {
            Self::Rsa { key_size } => *key_size / 8, // Approximate security level
            Self::EcdsaP256 => 128,
            Self::EcdsaP384 => 192,
            Self::EcdsaP521 => 256,
            Self::Ed25519 => 128,
            Self::X25519 => 128,
        }
    }
}

/// Security levels for certificate generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Standard security (suitable for most applications)
    Standard,
    /// High security (recommended for sensitive applications)
    High,
    /// Maximum security (for critical infrastructure)
    Maximum,
}

/// Certificate types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateType {
    /// Root Certificate Authority
    RootCA,
    /// Intermediate Certificate Authority
    IntermediateCA,
    /// Server certificate for TLS/SSL
    ServerCert,
    /// Client certificate for mutual TLS
    ClientCert,
    /// Code signing certificate
    CodeSigning,
    /// Email signing certificate
    EmailSigning,
}

/// Certificate subject information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSubject {
    /// Common Name (CN)
    pub common_name: String,
    /// Organization (O)
    pub organization: Option<String>,
    /// Organizational Unit (OU)
    pub organizational_unit: Option<String>,
    /// Country (C)
    pub country: Option<String>,
    /// State or Province (ST)
    pub state: Option<String>,
    /// Locality (L)
    pub locality: Option<String>,
    /// Email address
    pub email: Option<String>,
}

/// Certificate extensions and attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExtensions {
    /// Subject Alternative Names (DNS names, IP addresses)
    pub subject_alt_names: Vec<String>,
    /// Key usage flags
    pub key_usage: Vec<KeyUsage>,
    /// Extended key usage
    pub extended_key_usage: Vec<ExtendedKeyUsage>,
    /// Basic constraints
    pub is_ca: bool,
    /// Path length constraint for CA certificates
    pub path_length: Option<u32>,
    /// Certificate policies
    pub certificate_policies: Vec<String>,
}

/// Key usage enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyUsage {
    DigitalSignature,
    ContentCommitment,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CRLSign,
    EncipherOnly,
    DecipherOnly,
}

/// Extended key usage enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtendedKeyUsage {
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OCSPSigning,
}

/// Cryptographic key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Unique identifier for the key pair
    pub id: Uuid,
    /// Algorithm used for key generation
    pub algorithm: CryptoAlgorithm,
    /// Private key (encrypted)
    pub private_key: Vec<u8>,
    /// Public key
    pub public_key: Vec<u8>,
    /// Key generation timestamp
    pub created_at: DateTime<Utc>,
    /// Key expiration (if any)
    pub expires_at: Option<DateTime<Utc>>,
    /// Key metadata
    pub metadata: HashMap<String, String>,
}

/// X.509 Certificate structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Unique certificate identifier
    pub id: Uuid,
    /// Certificate serial number
    pub serial_number: Vec<u8>,
    /// Certificate subject
    pub subject: CertificateSubject,
    /// Certificate issuer
    pub issuer: CertificateSubject,
    /// Certificate type
    pub cert_type: CertificateType,
    /// Public key associated with certificate
    pub public_key_id: Uuid,
    /// Certificate extensions
    pub extensions: CertificateExtensions,
    /// Certificate validity period
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    /// DER-encoded certificate
    pub der_data: Vec<u8>,
    /// PEM-encoded certificate
    pub pem_data: String,
    /// Certificate fingerprint (SHA-256)
    pub fingerprint: String,
    /// Certificate status
    pub status: CertificateStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Certificate metadata
    pub metadata: HashMap<String, String>,
}

/// Certificate status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificateStatus {
    /// Certificate is valid and active
    Valid,
    /// Certificate has expired
    Expired,
    /// Certificate has been revoked
    Revoked,
    /// Certificate is pending validation
    Pending,
    /// Certificate has been suspended
    Suspended,
}

/// Certificate Revocation List entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRLEntry {
    /// Serial number of revoked certificate
    pub serial_number: Vec<u8>,
    /// Revocation date
    pub revocation_date: DateTime<Utc>,
    /// Revocation reason
    pub reason: RevocationReason,
}

/// Revocation reasons as per RFC 5280
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevocationReason {
    Unspecified,
    KeyCompromise,
    CACompromise,
    AffiliationChanged,
    Superseded,
    CessationOfOperation,
    CertificateHold,
    RemoveFromCRL,
    PrivilegeWithdrawn,
    AACompromise,
}

/// Certificate Request for certificate generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRequest {
    /// Certificate subject information
    pub subject: CertificateSubject,
    /// Certificate type
    pub cert_type: CertificateType,
    /// Cryptographic algorithm
    pub algorithm: CryptoAlgorithm,
    /// Certificate extensions
    pub extensions: CertificateExtensions,
    /// Certificate validity period
    pub validity_days: u32,
    /// Issuer certificate ID (for non-root certificates)
    pub issuer_cert_id: Option<Uuid>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Certificate renewal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalRequest {
    /// Original certificate ID
    pub certificate_id: Uuid,
    /// New validity period (days)
    pub validity_days: u32,
    /// Whether to generate new key pair
    pub generate_new_key: bool,
    /// Updated extensions (optional)
    pub extensions: Option<CertificateExtensions>,
}

/// Certificate validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether certificate is valid
    pub is_valid: bool,
    /// Validation errors/warnings
    pub issues: Vec<ValidationIssue>,
    /// Certificate chain validation
    pub chain_valid: bool,
    /// Expiration check
    pub expires_soon: bool,
    /// Days until expiration
    pub days_until_expiry: Option<i64>,
}

/// Certificate validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: ValidationSeverity,
    /// Issue description
    pub message: String,
    /// Issue category
    pub category: ValidationCategory,
}

/// Validation issue severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Validation issue categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationCategory {
    Expiration,
    ChainOfTrust,
    KeyStrength,
    AlgorithmSecurity,
    ExtensionCompliance,
    NameConstraints,
}

/// Security policy summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Only secure protocols are allowed
    pub secure_protocols_only: bool,
    /// List of allowed cryptographic algorithms
    pub allowed_algorithms: Vec<CryptoAlgorithm>,
    /// Minimum security level requirement
    pub minimum_security_level: SecurityLevel,
    /// Whether any legacy protocols are enabled
    pub legacy_protocols_enabled: bool,
    /// Minimum TLS version allowed
    pub tls_min_version: TlsVersion,
    /// Perfect forward secrecy enabled
    pub perfect_forward_secrecy: bool,
    /// Ready for post-quantum cryptography
    pub post_quantum_ready: bool,
}

/// Security audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    /// Total number of certificates
    pub total_certificates: usize,
    /// Number of certificates using secure algorithms
    pub secure_certificates: usize,
    /// Number of certificates using insecure algorithms
    pub insecure_certificates: usize,
    /// Number of certificates using deprecated algorithms
    pub deprecated_algorithms: usize,
    /// Number of certificates expiring soon
    pub expiring_certificates: usize,
    /// Number of revoked certificates
    pub revoked_certificates: usize,
    /// List of identified security issues
    pub security_issues: Vec<String>,
    /// Compliance summary by standard
    pub compliance_summary: HashMap<String, bool>,
}

/// Certificate Manager - Main interface for certificate operations
pub struct CertificateManager {
    /// Base directory for certificate storage
    base_path: PathBuf,
    /// Certificate storage
    certificates: HashMap<Uuid, Certificate>,
    /// Key pair storage
    key_pairs: HashMap<Uuid, KeyPair>,
    /// Certificate Revocation List
    crl_entries: Vec<CRLEntry>,
    /// Configuration
    pub config: CertificateConfig,
}

/// Legacy protocol configuration (admin-controlled for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyProtocolConfig {
    /// Allow TLS 1.0 (highly discouraged)
    pub allow_tls_1_0: bool,
    /// Allow TLS 1.1 (discouraged)
    pub allow_tls_1_1: bool,
    /// Allow RSA keys smaller than 2048 bits
    pub allow_weak_rsa: bool,
    /// Allow MD5 signatures (never recommended)
    pub allow_md5: bool,
    /// Allow SHA-1 signatures (deprecated)
    pub allow_sha1: bool,
    /// Admin justification for enabling legacy protocols
    pub admin_justification: Option<String>,
    /// Expiration date for legacy protocol allowance
    pub legacy_expires_at: Option<DateTime<Utc>>,
}

impl Default for LegacyProtocolConfig {
    fn default() -> Self {
        Self {
            allow_tls_1_0: false,
            allow_tls_1_1: false,
            allow_weak_rsa: false,
            allow_md5: false,
            allow_sha1: false,
            admin_justification: None,
            legacy_expires_at: None,
        }
    }
}

/// TLS/SSL configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Minimum TLS version (default: TLS 1.2)
    pub min_tls_version: TlsVersion,
    /// Preferred cipher suites (ordered by preference)
    pub cipher_suites: Vec<CipherSuite>,
    /// Enable perfect forward secrecy
    pub perfect_forward_secrecy: bool,
    /// Enable OCSP stapling
    pub ocsp_stapling: bool,
    /// Certificate transparency logging
    pub ct_logging: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            min_tls_version: TlsVersion::Tls12,
            cipher_suites: vec![
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
            ],
            perfect_forward_secrecy: true,
            ocsp_stapling: true,
            ct_logging: true,
        }
    }
}

/// Certificate Authority configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaConfig {
    /// Root CA certificate lifetime (years)
    pub root_ca_lifetime_years: u32,
    /// Intermediate CA certificate lifetime (years)  
    pub intermediate_ca_lifetime_years: u32,
    /// Maximum certificate chain depth
    pub max_chain_depth: u32,
    /// Enable certificate revocation list (CRL)
    pub enable_crl: bool,
    /// CRL distribution points
    pub crl_distribution_points: Vec<String>,
    /// OCSP responder URLs
    pub ocsp_responder_urls: Vec<String>,
}

impl Default for CaConfig {
    fn default() -> Self {
        Self {
            root_ca_lifetime_years: 20,
            intermediate_ca_lifetime_years: 10,
            max_chain_depth: 3,
            enable_crl: true,
            crl_distribution_points: vec![],
            ocsp_responder_urls: vec![],
        }
    }
}

/// TLS protocol versions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TlsVersion {
    Tls10,  // Deprecated
    Tls11,  // Deprecated
    Tls12,  // Current standard
    Tls13,  // Preferred
}

/// Cipher suites (simplified list of secure options)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CipherSuite {
    TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
}

/// Certificate manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    /// Default certificate validity period (days)
    pub default_validity_days: u32,
    /// Default cryptographic algorithm
    pub default_algorithm: CryptoAlgorithm,
    /// Minimum security level
    pub minimum_security_level: SecurityLevel,
    /// Enable automatic renewal
    pub auto_renewal_enabled: bool,
    /// Days before expiration to trigger renewal warning
    pub renewal_warning_days: u32,
    /// Certificate storage directory
    pub storage_directory: PathBuf,
    /// Enable certificate validation logging
    pub validation_logging: bool,
    /// Allowed algorithms (empty = all secure algorithms allowed)
    pub allowed_algorithms: Vec<CryptoAlgorithm>,
    /// Admin can enable insecure protocols for legacy compatibility
    pub admin_allow_insecure: bool,
    /// Legacy protocol settings (admin controlled)
    pub legacy_protocols: LegacyProtocolConfig,
    /// TLS/SSL configuration settings
    pub tls_config: TlsConfig,
    /// Certificate Authority settings
    pub ca_config: CaConfig,
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            default_validity_days: 365,
            default_algorithm: CryptoAlgorithm::EcdsaP256,
            minimum_security_level: SecurityLevel::Standard,
            auto_renewal_enabled: true,
            renewal_warning_days: 30,
            storage_directory: PathBuf::from("./certs"),
            validation_logging: true,
            allowed_algorithms: CryptoAlgorithm::recommended_for_security_level(SecurityLevel::Standard),
            admin_allow_insecure: false,
            legacy_protocols: LegacyProtocolConfig::default(),
            tls_config: TlsConfig::default(),
            ca_config: CaConfig::default(),
        }
    }
}

impl CertificateManager {
    /// Create a new certificate manager instance
    pub async fn new(config: CertificateConfig) -> Result<Self> {
        // Ensure storage directory exists
        if !config.storage_directory.exists() {
            fs::create_dir_all(&config.storage_directory)
                .context("Failed to create certificate storage directory")?;
        }

        let mut manager = Self {
            base_path: config.storage_directory.clone(),
            certificates: HashMap::new(),
            key_pairs: HashMap::new(),
            crl_entries: Vec::new(),
            config,
        };

        // Load existing certificates and keys
        manager.load_existing_certificates().await?;
        manager.load_existing_keys().await?;

        Ok(manager)
    }

    /// Generate a new cryptographic key pair
    pub async fn generate_key_pair(
        &mut self,
        algorithm: CryptoAlgorithm,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Uuid> {
        // Validate algorithm security
        if !algorithm.is_secure() && !self.config.admin_allow_insecure {
            return Err(anyhow!("Algorithm {:?} is not considered secure", algorithm));
        }

        // Check if algorithm is allowed
        if !self.config.allowed_algorithms.is_empty() 
            && !self.config.allowed_algorithms.contains(&algorithm) {
            return Err(anyhow!("Algorithm {:?} is not in allowed algorithms list", algorithm));
        }

        let key_id = Uuid::new_v4();
        let (private_key, public_key) = self.generate_key_pair_data(&algorithm).await?;

        let key_pair = KeyPair {
            id: key_id,
            algorithm,
            private_key,
            public_key,
            created_at: Utc::now(),
            expires_at: None, // Keys don't expire by default
            metadata: metadata.unwrap_or_default(),
        };

        // Store key pair securely
        self.store_key_pair(&key_pair).await?;
        self.key_pairs.insert(key_id, key_pair);

        log::info!("Generated new key pair with ID: {}", key_id);
        Ok(key_id)
    }

    /// Generate a new certificate
    pub async fn generate_certificate(
        &mut self,
        request: CertificateRequest,
    ) -> Result<Uuid> {
        // Validate certificate request
        self.validate_certificate_request(&request)?;

        // Generate key pair if needed
        let key_pair_id = self.generate_key_pair(request.algorithm.clone(), None).await?;

        // Generate certificate
        let cert_id = Uuid::new_v4();
        let serial_number = self.generate_serial_number();
        let now = Utc::now();
        let not_after = now + Duration::days(request.validity_days as i64);

        // Create certificate structure
        let certificate = Certificate {
            id: cert_id,
            serial_number: serial_number.clone(),
            subject: request.subject.clone(),
            issuer: self.get_issuer_subject(&request)?,
            cert_type: request.cert_type,
            public_key_id: key_pair_id,
            extensions: request.extensions,
            not_before: now,
            not_after,
            der_data: Vec::new(), // Will be populated by certificate generation
            pem_data: String::new(), // Will be populated by certificate generation
            fingerprint: String::new(), // Will be calculated after generation
            status: CertificateStatus::Valid,
            created_at: now,
            updated_at: now,
            metadata: request.metadata,
        };

        // Generate actual certificate data (X.509)
        let (der_data, pem_data) = self.generate_certificate_data(&certificate).await?;
        
        let mut certificate = certificate;
        certificate.der_data = der_data;
        certificate.pem_data = pem_data;
        certificate.fingerprint = self.calculate_fingerprint(&certificate.der_data);

        // Store certificate
        self.store_certificate(&certificate).await?;
        self.certificates.insert(cert_id, certificate);

        log::info!("Generated new certificate with ID: {}", cert_id);
        Ok(cert_id)
    }

    /// Validate a certificate
    pub async fn validate_certificate(&self, cert_id: Uuid) -> Result<ValidationResult> {
        let certificate = self.certificates.get(&cert_id)
            .ok_or_else(|| anyhow!("Certificate not found"))?;

        let mut issues = Vec::new();
        let now = Utc::now();

        // Check expiration
        let is_expired = certificate.not_after < now;
        let days_until_expiry = (certificate.not_after - now).num_days();
        let expires_soon = days_until_expiry <= self.config.renewal_warning_days as i64;

        if is_expired {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Critical,
                message: "Certificate has expired".to_string(),
                category: ValidationCategory::Expiration,
            });
        } else if expires_soon {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Warning,
                message: format!("Certificate expires in {} days", days_until_expiry),
                category: ValidationCategory::Expiration,
            });
        }

        // Check key pair exists and is secure
        if let Some(key_pair) = self.key_pairs.get(&certificate.public_key_id) {
            if !key_pair.algorithm.is_secure() {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    message: format!("Certificate uses insecure algorithm: {:?}", key_pair.algorithm),
                    category: ValidationCategory::AlgorithmSecurity,
                });
            }

            if key_pair.algorithm.security_bits() < 128 {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    message: "Certificate key strength is below recommended minimum".to_string(),
                    category: ValidationCategory::KeyStrength,
                });
            }
        } else {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                message: "Certificate key pair not found".to_string(),
                category: ValidationCategory::ChainOfTrust,
            });
        }

        // Validate certificate chain
        let chain_valid = self.validate_certificate_chain(cert_id).await?;
        if !chain_valid {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                message: "Certificate chain validation failed".to_string(),
                category: ValidationCategory::ChainOfTrust,
            });
        }

        let is_valid = !is_expired && 
                      certificate.status == CertificateStatus::Valid &&
                      chain_valid &&
                      !issues.iter().any(|i| matches!(i.severity, ValidationSeverity::Critical | ValidationSeverity::Error));

        Ok(ValidationResult {
            is_valid,
            issues,
            chain_valid,
            expires_soon,
            days_until_expiry: Some(days_until_expiry),
        })
    }

    /// Renew a certificate
    pub async fn renew_certificate(
        &mut self,
        renewal_request: RenewalRequest,
    ) -> Result<Uuid> {
        let original_cert = self.certificates.get(&renewal_request.certificate_id)
            .ok_or_else(|| anyhow!("Original certificate not found"))?
            .clone();

        // Create new certificate request based on original
        let cert_request = CertificateRequest {
            subject: original_cert.subject,
            cert_type: original_cert.cert_type,
            algorithm: self.key_pairs.get(&original_cert.public_key_id)
                .map(|kp| kp.algorithm.clone())
                .unwrap_or(self.config.default_algorithm.clone()),
            extensions: renewal_request.extensions.unwrap_or(original_cert.extensions),
            validity_days: renewal_request.validity_days,
            issuer_cert_id: None, // Will be determined by cert type
            metadata: original_cert.metadata,
        };

        // Generate new certificate
        let new_cert_id = self.generate_certificate(cert_request).await?;

        // Revoke original certificate
        self.revoke_certificate(
            renewal_request.certificate_id,
            RevocationReason::Superseded,
        ).await?;

        log::info!("Renewed certificate {} with new certificate {}", 
                  renewal_request.certificate_id, new_cert_id);

        Ok(new_cert_id)
    }

    /// Revoke a certificate
    pub async fn revoke_certificate(
        &mut self,
        cert_id: Uuid,
        reason: RevocationReason,
    ) -> Result<()> {
        // Update certificate status
        {
            let certificate = self.certificates.get_mut(&cert_id)
                .ok_or_else(|| anyhow!("Certificate not found"))?;

            certificate.status = CertificateStatus::Revoked;
            certificate.updated_at = Utc::now();

            // Add to CRL
            let crl_entry = CRLEntry {
                serial_number: certificate.serial_number.clone(),
                revocation_date: Utc::now(),
                reason,
            };

            self.crl_entries.push(crl_entry);
        }

        // Get the certificate again for persistence
        let certificate = self.certificates.get(&cert_id).unwrap();

        // Persist changes
        self.store_certificate(certificate).await?;
        self.store_crl().await?;

        log::info!("Revoked certificate with ID: {}", cert_id);
        Ok(())
    }

    /// List all certificates with optional filtering
    pub fn list_certificates(
        &self,
        filter: Option<CertificateFilter>,
    ) -> Vec<&Certificate> {
        let mut certs: Vec<&Certificate> = self.certificates.values().collect();

        if let Some(filter) = filter {
            certs.retain(|cert| {
                if let Some(status) = &filter.status {
                    if cert.status != *status {
                        return false;
                    }
                }

                if let Some(cert_type) = &filter.cert_type {
                    if std::mem::discriminant(&cert.cert_type) != std::mem::discriminant(cert_type) {
                        return false;
                    }
                }

                if let Some(expires_before) = filter.expires_before {
                    if cert.not_after > expires_before {
                        return false;
                    }
                }

                true
            });
        }

        certs.sort_by(|a, b| a.not_after.cmp(&b.not_after));
        certs
    }

    /// Get certificate by ID
    pub fn get_certificate(&self, cert_id: Uuid) -> Option<&Certificate> {
        self.certificates.get(&cert_id)
    }

    /// Get key pair by ID
    pub fn get_key_pair(&self, key_id: Uuid) -> Option<&KeyPair> {
        self.key_pairs.get(&key_id)
    }

    /// Export certificate in various formats
    pub async fn export_certificate(
        &self,
        cert_id: Uuid,
        format: ExportFormat,
        include_private_key: bool,
    ) -> Result<String> {
        let certificate = self.certificates.get(&cert_id)
            .ok_or_else(|| anyhow!("Certificate not found"))?;

        match format {
            ExportFormat::PEM => {
                let mut output = certificate.pem_data.clone();
                
                if include_private_key {
                    if let Some(key_pair) = self.key_pairs.get(&certificate.public_key_id) {
                        output.push_str("\n");
                        output.push_str(&self.format_private_key_pem(&key_pair.private_key)?);
                    }
                }
                
                Ok(output)
            }
            ExportFormat::DER => {
                Ok(general_purpose::STANDARD.encode(&certificate.der_data))
            }
            ExportFormat::PKCS12 => {
                // PKCS#12 format implementation would go here
                Err(anyhow!("PKCS#12 export not yet implemented"))
            }
        }
    }

    /// Import certificate from external source
    pub async fn import_certificate(
        &mut self,
        certificate_data: &str,
        format: ImportFormat,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<Uuid> {
        let (der_data, pem_data) = match format {
            ImportFormat::PEM => {
                let der_data = self.parse_pem_certificate(certificate_data)?;
                (der_data, certificate_data.to_string())
            }
            ImportFormat::DER => {
                let der_data = general_purpose::STANDARD.decode(certificate_data)
                    .context("Failed to decode base64 DER data")?;
                let pem_data = self.format_certificate_pem(&der_data)?;
                (der_data, pem_data)
            }
        };

        // Parse certificate information
        let cert_info = self.parse_certificate_info(&der_data)?;
        let cert_id = Uuid::new_v4();

        let certificate = Certificate {
            id: cert_id,
            serial_number: cert_info.serial_number,
            subject: cert_info.subject,
            issuer: cert_info.issuer,
            cert_type: cert_info.cert_type,
            public_key_id: Uuid::new_v4(), // Will need to import/generate key pair separately
            extensions: cert_info.extensions,
            not_before: cert_info.not_before,
            not_after: cert_info.not_after,
            der_data,
            pem_data,
            fingerprint: self.calculate_fingerprint(&cert_info.der_data),
            status: CertificateStatus::Valid,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: metadata.unwrap_or_default(),
        };

        // Store certificate
        self.store_certificate(&certificate).await?;
        self.certificates.insert(cert_id, certificate);

        log::info!("Imported certificate with ID: {}", cert_id);
        Ok(cert_id)
    }

    // Private helper methods

    async fn load_existing_certificates(&mut self) -> Result<()> {
        let cert_dir = self.base_path.join("certificates");
        if !cert_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(cert_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())?;
                let certificate: Certificate = serde_json::from_str(&content)?;
                self.certificates.insert(certificate.id, certificate);
            }
        }

        Ok(())
    }

    async fn load_existing_keys(&mut self) -> Result<()> {
        let key_dir = self.base_path.join("keys");
        if !key_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(key_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())?;
                let key_pair: KeyPair = serde_json::from_str(&content)?;
                self.key_pairs.insert(key_pair.id, key_pair);
            }
        }

        Ok(())
    }

    async fn generate_key_pair_data(&self, algorithm: &CryptoAlgorithm) -> Result<(Vec<u8>, Vec<u8>)> {
        // This is a placeholder implementation
        // In a real implementation, you would use proper cryptographic libraries
        // like openssl, ring, or rustls to generate actual key pairs
        
        match algorithm {
            CryptoAlgorithm::Rsa { key_size } => {
                // Generate RSA key pair
                let private_key = format!("RSA-{}-PRIVATE-KEY", key_size).into_bytes();
                let public_key = format!("RSA-{}-PUBLIC-KEY", key_size).into_bytes();
                Ok((private_key, public_key))
            }
            CryptoAlgorithm::EcdsaP256 => {
                let private_key = b"ECDSA-P256-PRIVATE-KEY".to_vec();
                let public_key = b"ECDSA-P256-PUBLIC-KEY".to_vec();
                Ok((private_key, public_key))
            }
            CryptoAlgorithm::EcdsaP384 => {
                let private_key = b"ECDSA-P384-PRIVATE-KEY".to_vec();
                let public_key = b"ECDSA-P384-PUBLIC-KEY".to_vec();
                Ok((private_key, public_key))
            }
            CryptoAlgorithm::EcdsaP521 => {
                let private_key = b"ECDSA-P521-PRIVATE-KEY".to_vec();
                let public_key = b"ECDSA-P521-PUBLIC-KEY".to_vec();
                Ok((private_key, public_key))
            }
            CryptoAlgorithm::Ed25519 => {
                let private_key = b"ED25519-PRIVATE-KEY".to_vec();
                let public_key = b"ED25519-PUBLIC-KEY".to_vec();
                Ok((private_key, public_key))
            }
            CryptoAlgorithm::X25519 => {
                let private_key = b"X25519-PRIVATE-KEY".to_vec();
                let public_key = b"X25519-PUBLIC-KEY".to_vec();
                Ok((private_key, public_key))
            }
        }
    }

    async fn generate_certificate_data(&self, certificate: &Certificate) -> Result<(Vec<u8>, String)> {
        // This is a placeholder implementation
        // In a real implementation, you would use proper X.509 libraries
        // to generate actual certificate data
        
        let der_data = format!("X509-CERTIFICATE-DER-{}", certificate.id).into_bytes();
        let pem_data = format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
            general_purpose::STANDARD.encode(&der_data)
        );
        
        Ok((der_data, pem_data))
    }

    fn validate_certificate_request(&self, request: &CertificateRequest) -> Result<()> {
        // Validate algorithm security
        if !request.algorithm.is_secure() && !self.config.admin_allow_insecure {
            return Err(anyhow!("Requested algorithm is not secure and admin has not allowed insecure protocols"));
        }

        // Check algorithm compliance
        match request.algorithm.compliance_status() {
            AlgorithmCompliance::Deprecated => {
                if !self.config.legacy_protocols.allow_weak_rsa && 
                   matches!(request.algorithm, CryptoAlgorithm::Rsa { key_size } if key_size < 2048) {
                    return Err(anyhow!("RSA keys smaller than 2048 bits are not allowed"));
                }
            }
            _ => {} // Standard and HighSecurity are always allowed
        }

        // Validate subject
        if request.subject.common_name.is_empty() {
            return Err(anyhow!("Common name is required"));
        }

        // Validate validity period
        if request.validity_days == 0 {
            return Err(anyhow!("Validity period must be greater than 0"));
        }

        // Check against maximum allowed validity based on certificate type
        let max_validity = match request.cert_type {
            CertificateType::RootCA => self.config.ca_config.root_ca_lifetime_years * 365,
            CertificateType::IntermediateCA => self.config.ca_config.intermediate_ca_lifetime_years * 365,
            _ => self.config.default_validity_days * 2, // Max 2x default for end-entity certs
        };

        if request.validity_days > max_validity {
            return Err(anyhow!(
                "Validity period {} days exceeds maximum allowed {} days for certificate type {:?}",
                request.validity_days, max_validity, request.cert_type
            ));
        }

        // Additional validation logic...
        Ok(())
    }

    /// Admin method to configure legacy protocol settings
    /// This should only be called by administrators with proper justification
    pub async fn configure_legacy_protocols(
        &mut self,
        legacy_config: LegacyProtocolConfig,
        admin_user: &str,
    ) -> Result<()> {
        // Log security configuration change
        log::warn!(
            "Admin {} is configuring legacy protocol settings: {:?}",
            admin_user, legacy_config
        );

        // Validate justification is provided for any enabled legacy protocols
        if (legacy_config.allow_tls_1_0 || 
            legacy_config.allow_tls_1_1 || 
            legacy_config.allow_weak_rsa ||
            legacy_config.allow_md5 ||
            legacy_config.allow_sha1) && 
           legacy_config.admin_justification.is_none() {
            return Err(anyhow!("Admin justification required for enabling legacy protocols"));
        }

        // Check if legacy allowance has expired
        if let Some(expires_at) = legacy_config.legacy_expires_at {
            if expires_at < Utc::now() {
                return Err(anyhow!("Legacy protocol allowance has expired"));
            }
        }

        self.config.legacy_protocols = legacy_config;
        Ok(())
    }

    /// Get current security policy summary
    pub fn get_security_policy(&self) -> SecurityPolicy {
        SecurityPolicy {
            secure_protocols_only: !self.config.admin_allow_insecure,
            allowed_algorithms: self.config.allowed_algorithms.clone(),
            minimum_security_level: self.config.minimum_security_level.clone(),
            legacy_protocols_enabled: self.config.legacy_protocols.allow_tls_1_0 ||
                                    self.config.legacy_protocols.allow_tls_1_1 ||
                                    self.config.legacy_protocols.allow_weak_rsa,
            tls_min_version: self.config.tls_config.min_tls_version.clone(),
            perfect_forward_secrecy: self.config.tls_config.perfect_forward_secrecy,
            post_quantum_ready: false, // Will be true when PQC algorithms are added
        }
    }

    /// Admin method to update TLS configuration
    pub async fn update_tls_config(
        &mut self,
        tls_config: TlsConfig,
        admin_user: &str,
    ) -> Result<()> {
        log::info!("Admin {} updating TLS configuration", admin_user);
        
        // Validate TLS configuration
        if matches!(tls_config.min_tls_version, TlsVersion::Tls10 | TlsVersion::Tls11) &&
           !self.config.legacy_protocols.allow_tls_1_0 && 
           !self.config.legacy_protocols.allow_tls_1_1 {
            return Err(anyhow!("TLS 1.0/1.1 not allowed without legacy protocol configuration"));
        }

        self.config.tls_config = tls_config;
        Ok(())
    }

    /// Get certificate security audit report
    pub async fn generate_security_audit(&self) -> SecurityAuditReport {
        let mut report = SecurityAuditReport {
            total_certificates: self.certificates.len(),
            secure_certificates: 0,
            insecure_certificates: 0,
            deprecated_algorithms: 0,
            expiring_certificates: 0,
            revoked_certificates: 0,
            security_issues: Vec::new(),
            compliance_summary: HashMap::new(),
        };

        let now = Utc::now();
        
        for certificate in self.certificates.values() {
            // Check security status
            if let Some(key_pair) = self.key_pairs.get(&certificate.public_key_id) {
                if key_pair.algorithm.is_secure() {
                    report.secure_certificates += 1;
                } else {
                    report.insecure_certificates += 1;
                    report.security_issues.push(format!(
                        "Certificate {} uses insecure algorithm {:?}",
                        certificate.id, key_pair.algorithm
                    ));
                }

                // Check for deprecated algorithms
                if matches!(key_pair.algorithm.compliance_status(), AlgorithmCompliance::Deprecated) {
                    report.deprecated_algorithms += 1;
                }
            }

            // Check expiration
            if certificate.not_after <= now + Duration::days(self.config.renewal_warning_days as i64) {
                report.expiring_certificates += 1;
            }

            // Check revocation status
            if certificate.status == CertificateStatus::Revoked {
                report.revoked_certificates += 1;
            }
        }

        report
    }

    fn get_issuer_subject(&self, request: &CertificateRequest) -> Result<CertificateSubject> {
        match &request.issuer_cert_id {
            Some(issuer_id) => {
                let issuer_cert = self.certificates.get(issuer_id)
                    .ok_or_else(|| anyhow!("Issuer certificate not found"))?;
                Ok(issuer_cert.subject.clone())
            }
            None => {
                // Self-signed certificate
                Ok(request.subject.clone())
            }
        }
    }

    fn generate_serial_number(&self) -> Vec<u8> {
        // Generate a unique serial number
        Uuid::new_v4().as_bytes().to_vec()
    }

    fn calculate_fingerprint(&self, der_data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(der_data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    async fn validate_certificate_chain(&self, _cert_id: Uuid) -> Result<bool> {
        // Placeholder for certificate chain validation
        // In a real implementation, this would validate the entire certificate chain
        Ok(true)
    }

    async fn store_certificate(&self, certificate: &Certificate) -> Result<()> {
        let cert_dir = self.base_path.join("certificates");
        fs::create_dir_all(&cert_dir)?;
        
        let cert_file = cert_dir.join(format!("{}.json", certificate.id));
        let content = serde_json::to_string_pretty(certificate)?;
        fs::write(cert_file, content)?;
        
        Ok(())
    }

    async fn store_key_pair(&self, key_pair: &KeyPair) -> Result<()> {
        let key_dir = self.base_path.join("keys");
        fs::create_dir_all(&key_dir)?;
        
        let key_file = key_dir.join(format!("{}.json", key_pair.id));
        let content = serde_json::to_string_pretty(key_pair)?;
        fs::write(key_file, content)?;
        
        Ok(())
    }

    async fn store_crl(&self) -> Result<()> {
        let crl_file = self.base_path.join("crl.json");
        let content = serde_json::to_string_pretty(&self.crl_entries)?;
        fs::write(crl_file, content)?;
        
        Ok(())
    }

    fn format_private_key_pem(&self, _private_key: &[u8]) -> Result<String> {
        // Placeholder for PEM formatting
        Ok("-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----".to_string())
    }

    fn parse_pem_certificate(&self, _pem_data: &str) -> Result<Vec<u8>> {
        // Placeholder for PEM parsing
        Ok(b"parsed-der-data".to_vec())
    }

    fn format_certificate_pem(&self, _der_data: &[u8]) -> Result<String> {
        // Placeholder for PEM formatting
        Ok("-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----".to_string())
    }

    fn parse_certificate_info(&self, _der_data: &[u8]) -> Result<ParsedCertificateInfo> {
        // Placeholder for certificate parsing
        Ok(ParsedCertificateInfo {
            serial_number: vec![1, 2, 3, 4],
            subject: CertificateSubject {
                common_name: "parsed.example.com".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            issuer: CertificateSubject {
                common_name: "CA".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            cert_type: CertificateType::ServerCert,
            extensions: CertificateExtensions {
                subject_alt_names: vec![],
                key_usage: vec![],
                extended_key_usage: vec![],
                is_ca: false,
                path_length: None,
                certificate_policies: vec![],
            },
            not_before: Utc::now(),
            not_after: Utc::now() + Duration::days(365),
            der_data: Vec::new(),
        })
    }
}

/// Certificate filter for listing operations
#[derive(Debug, Clone)]
pub struct CertificateFilter {
    pub status: Option<CertificateStatus>,
    pub cert_type: Option<CertificateType>,
    pub expires_before: Option<DateTime<Utc>>,
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    PEM,
    DER,
    PKCS12,
}

/// Import format options
#[derive(Debug, Clone)]
pub enum ImportFormat {
    PEM,
    DER,
}

/// Parsed certificate information (internal use)
struct ParsedCertificateInfo {
    serial_number: Vec<u8>,
    subject: CertificateSubject,
    issuer: CertificateSubject,
    cert_type: CertificateType,
    extensions: CertificateExtensions,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    der_data: Vec<u8>,
}

// Add hex dependency to Cargo.toml for fingerprint calculation
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_certificate_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = CertificateManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_key_pair_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        let key_id = manager.generate_key_pair(CryptoAlgorithm::EcdsaP256, None).await;
        
        assert!(key_id.is_ok());
        let key_id = key_id.unwrap();
        assert!(manager.get_key_pair(key_id).is_some());
    }

    #[tokio::test]
    async fn test_certificate_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        let request = CertificateRequest {
            subject: CertificateSubject {
                common_name: "test.example.com".to_string(),
                organization: Some("Test Org".to_string()),
                organizational_unit: None,
                country: Some("US".to_string()),
                state: None,
                locality: None,
                email: None,
            },
            cert_type: CertificateType::ServerCert,
            algorithm: CryptoAlgorithm::EcdsaP256,
            extensions: CertificateExtensions {
                subject_alt_names: vec!["*.example.com".to_string()],
                key_usage: vec![KeyUsage::DigitalSignature, KeyUsage::KeyEncipherment],
                extended_key_usage: vec![ExtendedKeyUsage::ServerAuth],
                is_ca: false,
                path_length: None,
                certificate_policies: vec![],
            },
            validity_days: 365,
            issuer_cert_id: None,
            metadata: HashMap::new(),
        };

        let cert_id = manager.generate_certificate(request).await;
        assert!(cert_id.is_ok());
        
        let cert_id = cert_id.unwrap();
        assert!(manager.get_certificate(cert_id).is_some());
    }

    #[tokio::test]
    async fn test_certificate_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        let request = CertificateRequest {
            subject: CertificateSubject {
                common_name: "test.example.com".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            cert_type: CertificateType::ServerCert,
            algorithm: CryptoAlgorithm::EcdsaP256,
            extensions: CertificateExtensions {
                subject_alt_names: vec![],
                key_usage: vec![],
                extended_key_usage: vec![],
                is_ca: false,
                path_length: None,
                certificate_policies: vec![],
            },
            validity_days: 365,
            issuer_cert_id: None,
            metadata: HashMap::new(),
        };

        let cert_id = manager.generate_certificate(request).await.unwrap();
        let validation = manager.validate_certificate(cert_id).await;
        
        assert!(validation.is_ok());
        let validation = validation.unwrap();
        assert!(validation.is_valid);
    }

    #[test]
    fn test_algorithm_security() {
        assert!(CryptoAlgorithm::EcdsaP256.is_secure());
        assert!(CryptoAlgorithm::Rsa { key_size: 2048 }.is_secure());
        assert!(!CryptoAlgorithm::Rsa { key_size: 1024 }.is_secure());
    }

    #[test]
    fn test_security_level_recommendations() {
        let standard = CryptoAlgorithm::recommended_for_security_level(SecurityLevel::Standard);
        assert!(!standard.is_empty());
        
        let high = CryptoAlgorithm::recommended_for_security_level(SecurityLevel::High);
        assert!(!high.is_empty());
        
        let maximum = CryptoAlgorithm::recommended_for_security_level(SecurityLevel::Maximum);
        assert!(!maximum.is_empty());
    }

    #[test]
    fn test_algorithm_compliance() {
        // Test RSA compliance
        assert_eq!(CryptoAlgorithm::Rsa { key_size: 4096 }.compliance_status(), AlgorithmCompliance::HighSecurity);
        assert_eq!(CryptoAlgorithm::Rsa { key_size: 2048 }.compliance_status(), AlgorithmCompliance::Standard);
        assert_eq!(CryptoAlgorithm::Rsa { key_size: 1024 }.compliance_status(), AlgorithmCompliance::Deprecated);
        
        // Test ECDSA compliance
        assert_eq!(CryptoAlgorithm::EcdsaP256.compliance_status(), AlgorithmCompliance::Standard);
        assert_eq!(CryptoAlgorithm::EcdsaP384.compliance_status(), AlgorithmCompliance::HighSecurity);
        assert_eq!(CryptoAlgorithm::EcdsaP521.compliance_status(), AlgorithmCompliance::HighSecurity);
        
        // Test modern algorithms
        assert_eq!(CryptoAlgorithm::Ed25519.compliance_status(), AlgorithmCompliance::HighSecurity);
        assert_eq!(CryptoAlgorithm::X25519.compliance_status(), AlgorithmCompliance::HighSecurity);
    }

    #[test]
    fn test_post_quantum_resistance() {
        // Current algorithms are not post-quantum resistant
        assert!(!CryptoAlgorithm::EcdsaP256.is_post_quantum_resistant());
        assert!(!CryptoAlgorithm::Rsa { key_size: 4096 }.is_post_quantum_resistant());
        assert!(!CryptoAlgorithm::Ed25519.is_post_quantum_resistant());
    }

    #[tokio::test]
    async fn test_legacy_protocol_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        // Test configuring legacy protocols with justification
        let legacy_config = LegacyProtocolConfig {
            allow_tls_1_1: true,
            admin_justification: Some("Testing legacy system compatibility".to_string()),
            legacy_expires_at: Some(Utc::now() + Duration::days(30)),
            ..Default::default()
        };

        let result = manager.configure_legacy_protocols(legacy_config, "test_admin").await;
        assert!(result.is_ok());
        assert!(manager.config.legacy_protocols.allow_tls_1_1);
    }

    #[tokio::test]
    async fn test_legacy_protocol_validation_failure() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        // Test configuring legacy protocols without justification (should fail)
        let legacy_config = LegacyProtocolConfig {
            allow_tls_1_0: true,
            admin_justification: None,
            ..Default::default()
        };

        let result = manager.configure_legacy_protocols(legacy_config, "test_admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("justification required"));
    }

    #[tokio::test]
    async fn test_security_policy_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = CertificateManager::new(config).await.unwrap();
        
        let policy = manager.get_security_policy();
        assert!(policy.secure_protocols_only);
        assert!(!policy.legacy_protocols_enabled);
        assert_eq!(policy.tls_min_version, TlsVersion::Tls12);
        assert!(policy.perfect_forward_secrecy);
        assert!(!policy.post_quantum_ready);
    }

    #[tokio::test]
    async fn test_tls_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        // Test updating TLS configuration
        let tls_config = TlsConfig {
            min_tls_version: TlsVersion::Tls13,
            cipher_suites: vec![CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384],
            perfect_forward_secrecy: true,
            ocsp_stapling: true,
            ct_logging: true,
        };

        let result = manager.update_tls_config(tls_config, "test_admin").await;
        assert!(result.is_ok());
        assert_eq!(manager.config.tls_config.min_tls_version, TlsVersion::Tls13);
    }

    #[tokio::test]
    async fn test_security_audit_report() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        // Generate a certificate for testing
        let request = CertificateRequest {
            subject: CertificateSubject {
                common_name: "test.example.com".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            cert_type: CertificateType::ServerCert,
            algorithm: CryptoAlgorithm::EcdsaP256,
            extensions: CertificateExtensions {
                subject_alt_names: vec![],
                key_usage: vec![],
                extended_key_usage: vec![],
                is_ca: false,
                path_length: None,
                certificate_policies: vec![],
            },
            validity_days: 365,
            issuer_cert_id: None,
            metadata: HashMap::new(),
        };

        let _cert_id = manager.generate_certificate(request).await.unwrap();
        
        // Generate audit report
        let audit = manager.generate_security_audit().await;
        assert_eq!(audit.total_certificates, 1);
        assert_eq!(audit.secure_certificates, 1);
        assert_eq!(audit.insecure_certificates, 0);
    }

    #[tokio::test]
    async fn test_enhanced_certificate_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CertificateConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ca_config: CaConfig {
                root_ca_lifetime_years: 10,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut manager = CertificateManager::new(config).await.unwrap();
        
        // Test certificate request with excessive validity period
        let request = CertificateRequest {
            subject: CertificateSubject {
                common_name: "test.example.com".to_string(),
                organization: None,
                organizational_unit: None,
                country: None,
                state: None,
                locality: None,
                email: None,
            },
            cert_type: CertificateType::ServerCert,
            algorithm: CryptoAlgorithm::EcdsaP256,
            extensions: CertificateExtensions {
                subject_alt_names: vec![],
                key_usage: vec![],
                extended_key_usage: vec![],
                is_ca: false,
                path_length: None,
                certificate_policies: vec![],
            },
            validity_days: 3000, // Excessive validity period
            issuer_cert_id: None,
            metadata: HashMap::new(),
        };

        let result = manager.generate_certificate(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum allowed"));
    }
}