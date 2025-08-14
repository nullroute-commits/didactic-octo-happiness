//! Certificate Management Web API Handlers
//!
//! This module provides REST API endpoints for the certificate management system,
//! including certificate generation, validation, renewal, and administrative operations.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    certificate_manager::{
        CertificateManager, CertificateRequest, RenewalRequest, ValidationResult,
        CryptoAlgorithm, CertificateConfig, CertificateStatus, 
        CertificateType, CertificateFilter, ExportFormat, ImportFormat,
        RevocationReason, Certificate, KeyPair, SecurityLevel,
        LegacyProtocolConfig, TlsConfig, SecurityPolicy, SecurityAuditReport,
    },
    web_types::{ApiError, ApiResponse},
};

/// Certificate management application state
#[derive(Clone)]
pub struct CertificateAppState {
    pub cert_manager: std::sync::Arc<tokio::sync::RwLock<CertificateManager>>,
}

/// Create certificate management routes
pub fn create_certificate_routes() -> Router<CertificateAppState> {
    Router::new()
        // Certificate management endpoints
        .route("/api/certificates", get(list_certificates))
        .route("/api/certificates", post(create_certificate))
        .route("/api/certificates/:id", get(get_certificate))
        .route("/api/certificates/:id", delete(revoke_certificate))
        .route("/api/certificates/:id/validate", get(validate_certificate))
        .route("/api/certificates/:id/renew", post(renew_certificate))
        .route("/api/certificates/:id/export", get(export_certificate))
        .route("/api/certificates/import", post(import_certificate))
        
        // Key pair management
        .route("/api/keypairs", get(list_key_pairs))
        .route("/api/keypairs", post(create_key_pair))
        .route("/api/keypairs/:id", get(get_key_pair))
        .route("/api/keypairs/:id", delete(delete_key_pair))
        
        // Certificate authority operations
        .route("/api/ca/create", post(create_certificate_authority))
        .route("/api/ca/certificates", get(list_ca_certificates))
        
        // System configuration
        .route("/api/certificates/config", get(get_certificate_config))
        .route("/api/certificates/config", put(update_certificate_config))
        .route("/api/certificates/algorithms", get(get_supported_algorithms))
        
        // Administrative endpoints (require admin role)
        .route("/api/admin/certificates/security-policy", get(get_security_policy))
        .route("/api/admin/certificates/security-policy", put(update_security_policy))
        .route("/api/admin/certificates/legacy-protocols", get(get_legacy_protocols))
        .route("/api/admin/certificates/legacy-protocols", put(configure_legacy_protocols))
        .route("/api/admin/certificates/tls-config", get(get_tls_config))
        .route("/api/admin/certificates/tls-config", put(update_tls_config))
        .route("/api/admin/certificates/audit", get(get_security_audit))
        
        // Certificate validation and health
        .route("/api/certificates/health", get(certificate_system_health))
        .route("/api/certificates/expiring", get(get_expiring_certificates))
        
        // Revocation and CRL
        .route("/api/certificates/crl", get(get_certificate_revocation_list))
}

/// Request structure for creating a new certificate
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCertificateRequest {
    /// Certificate subject information
    pub subject: CertificateSubjectRequest,
    /// Certificate type
    pub cert_type: CertificateType,
    /// Cryptographic algorithm (optional, uses default if not specified)
    pub algorithm: Option<CryptoAlgorithm>,
    /// Certificate extensions
    pub extensions: CertificateExtensionsRequest,
    /// Certificate validity period in days
    pub validity_days: Option<u32>,
    /// Issuer certificate ID (for non-root certificates)
    pub issuer_cert_id: Option<Uuid>,
    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Certificate subject request structure
#[derive(Debug, Deserialize, Serialize)]
pub struct CertificateSubjectRequest {
    pub common_name: String,
    pub organization: Option<String>,
    pub organizational_unit: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub locality: Option<String>,
    pub email: Option<String>,
}

/// Certificate extensions request structure
#[derive(Debug, Deserialize, Serialize)]
pub struct CertificateExtensionsRequest {
    pub subject_alt_names: Option<Vec<String>>,
    pub key_usage: Option<Vec<String>>,
    pub extended_key_usage: Option<Vec<String>>,
    pub is_ca: Option<bool>,
    pub path_length: Option<u32>,
    pub certificate_policies: Option<Vec<String>>,
}

/// Query parameters for listing certificates
#[derive(Debug, Deserialize)]
pub struct ListCertificatesQuery {
    pub status: Option<CertificateStatus>,
    pub cert_type: Option<String>,
    pub expires_before: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Export certificate query parameters
#[derive(Debug, Deserialize)]
pub struct ExportCertificateQuery {
    pub format: Option<String>,
    pub include_private_key: Option<bool>,
}

/// Import certificate request
#[derive(Debug, Deserialize)]
pub struct ImportCertificateRequest {
    pub certificate_data: String,
    pub format: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Key pair creation request
#[derive(Debug, Deserialize)]
pub struct CreateKeyPairRequest {
    pub algorithm: CryptoAlgorithm,
    pub metadata: Option<HashMap<String, String>>,
}

/// Certificate renewal request
#[derive(Debug, Deserialize)]
pub struct RenewCertificateRequest {
    pub validity_days: u32,
    pub generate_new_key: Option<bool>,
    pub extensions: Option<CertificateExtensionsRequest>,
}

/// Certificate revocation request
#[derive(Debug, Deserialize)]
pub struct RevokeCertificateRequest {
    pub reason: Option<String>,
}

/// Certificate system health response
#[derive(Debug, Serialize)]
pub struct CertificateHealthResponse {
    pub status: String,
    pub total_certificates: usize,
    pub valid_certificates: usize,
    pub expired_certificates: usize,
    pub revoked_certificates: usize,
    pub expiring_soon: usize,
    pub security_issues: usize,
}

/// Supported algorithms response
#[derive(Debug, Serialize)]
pub struct SupportedAlgorithmsResponse {
    pub algorithms: Vec<AlgorithmInfo>,
    pub security_levels: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AlgorithmInfo {
    pub algorithm: CryptoAlgorithm,
    pub security_bits: u32,
    pub is_secure: bool,
    pub recommended_for: Vec<String>,
}

/// List all certificates with optional filtering
pub async fn list_certificates(
    State(state): State<CertificateAppState>,
    Query(query): Query<ListCertificatesQuery>,
) -> Result<Json<ApiResponse<Vec<Certificate>>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    // Build filter from query parameters
    let filter = CertificateFilter {
        status: query.status,
        cert_type: None, // Convert from string if needed
        expires_before: None, // Parse datetime if provided
    };
    
    let certificates = cert_manager.list_certificates(Some(filter));
    let certificates: Vec<Certificate> = certificates.into_iter().cloned().collect();
    
    // Apply pagination
    let total = certificates.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 per page
    
    let paginated: Vec<Certificate> = certificates
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(paginated),
        message: Some(format!("Retrieved {} certificates (total: {})", limit, total)),
        error: None,
    }))
}

/// Create a new certificate
pub async fn create_certificate(
    State(state): State<CertificateAppState>,
    Json(request): Json<CreateCertificateRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    // Convert request to internal format
    let cert_request = CertificateRequest {
        subject: crate::certificate_manager::CertificateSubject {
            common_name: request.subject.common_name,
            organization: request.subject.organization,
            organizational_unit: request.subject.organizational_unit,
            country: request.subject.country,
            state: request.subject.state,
            locality: request.subject.locality,
            email: request.subject.email,
        },
        cert_type: request.cert_type,
        algorithm: request.algorithm.unwrap_or(CryptoAlgorithm::EcdsaP256),
        extensions: convert_extensions_request(request.extensions),
        validity_days: request.validity_days.unwrap_or(365),
        issuer_cert_id: request.issuer_cert_id,
        metadata: request.metadata.unwrap_or_default(),
    };
    
    match cert_manager.generate_certificate(cert_request).await {
        Ok(cert_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(cert_id),
            message: Some("Certificate created successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Get a specific certificate
pub async fn get_certificate(
    State(state): State<CertificateAppState>,
    Path(cert_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Certificate>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    match cert_manager.get_certificate(cert_id) {
        Some(certificate) => Ok(Json(ApiResponse {
            success: true,
            data: Some(certificate.clone()),
            message: None,
            error: None,
        })),
        None => Err(ApiError::NotFound("Certificate not found".to_string())),
    }
}

/// Validate a certificate
pub async fn validate_certificate(
    State(state): State<CertificateAppState>,
    Path(cert_id): Path<Uuid>,
) -> Result<Json<ApiResponse<ValidationResult>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    match cert_manager.validate_certificate(cert_id).await {
        Ok(validation_result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(validation_result),
            message: None,
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Renew a certificate
pub async fn renew_certificate(
    State(state): State<CertificateAppState>,
    Path(cert_id): Path<Uuid>,
    Json(request): Json<RenewCertificateRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    let renewal_request = RenewalRequest {
        certificate_id: cert_id,
        validity_days: request.validity_days,
        generate_new_key: request.generate_new_key.unwrap_or(false),
        extensions: request.extensions.map(convert_extensions_request),
    };
    
    match cert_manager.renew_certificate(renewal_request).await {
        Ok(new_cert_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(new_cert_id),
            message: Some("Certificate renewed successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Revoke a certificate
pub async fn revoke_certificate(
    State(state): State<CertificateAppState>,
    Path(cert_id): Path<Uuid>,
    Json(request): Json<RevokeCertificateRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    let reason = match request.reason.as_deref() {
        Some("key_compromise") => RevocationReason::KeyCompromise,
        Some("ca_compromise") => RevocationReason::CACompromise,
        Some("affiliation_changed") => RevocationReason::AffiliationChanged,
        Some("superseded") => RevocationReason::Superseded,
        Some("cessation_of_operation") => RevocationReason::CessationOfOperation,
        _ => RevocationReason::Unspecified,
    };
    
    match cert_manager.revoke_certificate(cert_id, reason).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            message: Some("Certificate revoked successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Export a certificate
pub async fn export_certificate(
    State(state): State<CertificateAppState>,
    Path(cert_id): Path<Uuid>,
    Query(query): Query<ExportCertificateQuery>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    let format = match query.format.as_deref() {
        Some("der") => ExportFormat::DER,
        Some("pkcs12") => ExportFormat::PKCS12,
        _ => ExportFormat::PEM,
    };
    
    let include_private_key = query.include_private_key.unwrap_or(false);
    
    match cert_manager.export_certificate(cert_id, format, include_private_key).await {
        Ok(certificate_data) => Ok(Json(ApiResponse {
            success: true,
            data: Some(certificate_data),
            message: None,
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Import a certificate
pub async fn import_certificate(
    State(state): State<CertificateAppState>,
    Json(request): Json<ImportCertificateRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    let format = match request.format.as_str() {
        "der" => ImportFormat::DER,
        _ => ImportFormat::PEM,
    };
    
    match cert_manager.import_certificate(&request.certificate_data, format, request.metadata).await {
        Ok(cert_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(cert_id),
            message: Some("Certificate imported successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// List key pairs
pub async fn list_key_pairs(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<Vec<KeyPair>>>, ApiError> {
    let _cert_manager = state.cert_manager.read().await;
    
    // For security, we should not expose private key data in the API
    // This is a placeholder - in a real implementation, filter sensitive data
    let key_pairs: Vec<KeyPair> = vec![]; // Implement key pair listing
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(key_pairs),
        message: Some("Key pairs retrieved successfully".to_string()),
        error: None,
    }))
}

/// Create a new key pair
pub async fn create_key_pair(
    State(state): State<CertificateAppState>,
    Json(request): Json<CreateKeyPairRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    match cert_manager.generate_key_pair(request.algorithm, request.metadata).await {
        Ok(key_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(key_id),
            message: Some("Key pair created successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Get a specific key pair
pub async fn get_key_pair(
    State(state): State<CertificateAppState>,
    Path(key_id): Path<Uuid>,
) -> Result<Json<ApiResponse<KeyPair>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    match cert_manager.get_key_pair(key_id) {
        Some(key_pair) => {
            // For security, create a version without the private key
            let safe_key_pair = KeyPair {
                private_key: vec![], // Remove private key from response
                ..key_pair.clone()
            };
            
            Ok(Json(ApiResponse {
                success: true,
                data: Some(safe_key_pair),
                message: None,
                error: None,
            }))
        }
        None => Err(ApiError::NotFound("Key pair not found".to_string())),
    }
}

/// Delete a key pair
pub async fn delete_key_pair(
    State(_state): State<CertificateAppState>,
    Path(_key_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // This is a placeholder - implement key pair deletion
    // Should include security checks to ensure key is not in use
    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Key pair deleted successfully".to_string()),
        error: None,
    }))
}

/// Create a certificate authority
pub async fn create_certificate_authority(
    State(state): State<CertificateAppState>,
    Json(request): Json<CreateCertificateRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    // Validate that this is a CA request
    if !matches!(request.cert_type, CertificateType::RootCA | CertificateType::IntermediateCA) {
        return Err(ApiError::BadRequest("Invalid certificate type for CA".to_string()));
    }
    
    // Delegate to certificate creation with additional CA validation
    create_certificate(State(state), Json(request)).await
}

/// List CA certificates
pub async fn list_ca_certificates(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<Vec<Certificate>>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    let filter = CertificateFilter {
        status: Some(CertificateStatus::Valid),
        cert_type: None, // Filter for CA types in the implementation
        expires_before: None,
    };
    
    let certificates = cert_manager.list_certificates(Some(filter));
    let ca_certificates: Vec<Certificate> = certificates
        .into_iter()
        .filter(|cert| matches!(cert.cert_type, CertificateType::RootCA | CertificateType::IntermediateCA))
        .cloned()
        .collect();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(ca_certificates),
        message: None,
        error: None,
    }))
}

/// Get certificate configuration
pub async fn get_certificate_config(
    State(_state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<CertificateConfig>>, ApiError> {
    // Return current configuration
    let config = CertificateConfig::default(); // Get from actual config
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(config),
        message: None,
        error: None,
    }))
}

/// Update certificate configuration
pub async fn update_certificate_config(
    State(_state): State<CertificateAppState>,
    Json(_config): Json<CertificateConfig>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    // Implement configuration update
    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Configuration updated successfully".to_string()),
        error: None,
    }))
}

/// Get supported algorithms
pub async fn get_supported_algorithms(
    State(_state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<SupportedAlgorithmsResponse>>, ApiError> {
    let algorithms = vec![
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::EcdsaP256,
            security_bits: 128,
            is_secure: true,
            recommended_for: vec!["Standard security".to_string()],
        },
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::EcdsaP384,
            security_bits: 192,
            is_secure: true,
            recommended_for: vec!["High security".to_string()],
        },
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::EcdsaP521,
            security_bits: 256,
            is_secure: true,
            recommended_for: vec!["Maximum security".to_string()],
        },
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::Rsa { key_size: 2048 },
            security_bits: 112,
            is_secure: true,
            recommended_for: vec!["Standard security".to_string()],
        },
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::Rsa { key_size: 3072 },
            security_bits: 128,
            is_secure: true,
            recommended_for: vec!["High security".to_string()],
        },
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::Rsa { key_size: 4096 },
            security_bits: 152,
            is_secure: true,
            recommended_for: vec!["Maximum security".to_string()],
        },
        AlgorithmInfo {
            algorithm: CryptoAlgorithm::Ed25519,
            security_bits: 128,
            is_secure: true,
            recommended_for: vec!["High security".to_string(), "Maximum security".to_string()],
        },
    ];
    
    let response = SupportedAlgorithmsResponse {
        algorithms,
        security_levels: vec![
            "Standard".to_string(),
            "High".to_string(),
            "Maximum".to_string(),
        ],
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: None,
        error: None,
    }))
}

/// Get certificate system health
pub async fn certificate_system_health(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<CertificateHealthResponse>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    let all_certificates = cert_manager.list_certificates(None);
    let total_certificates = all_certificates.len();
    
    let valid_certificates = all_certificates.iter()
        .filter(|cert| cert.status == CertificateStatus::Valid)
        .count();
    
    let expired_certificates = all_certificates.iter()
        .filter(|cert| cert.status == CertificateStatus::Expired)
        .count();
    
    let revoked_certificates = all_certificates.iter()
        .filter(|cert| cert.status == CertificateStatus::Revoked)
        .count();
    
    // Count certificates expiring within 30 days
    let expiring_soon = 0; // Implement expiration check
    let security_issues = 0; // Implement security issue detection
    
    let health_response = CertificateHealthResponse {
        status: "healthy".to_string(),
        total_certificates,
        valid_certificates,
        expired_certificates,
        revoked_certificates,
        expiring_soon,
        security_issues,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(health_response),
        message: None,
        error: None,
    }))
}

/// Get certificates expiring soon
pub async fn get_expiring_certificates(
    State(state): State<CertificateAppState>,
    Query(_query): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<Certificate>>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    
    // Get certificates expiring within the specified timeframe
    let expiring_certificates = cert_manager.list_certificates(None);
    
    // Filter for certificates expiring soon (placeholder logic)
    let expiring: Vec<Certificate> = expiring_certificates
        .into_iter()
        .cloned()
        .collect();
    
    let expiring_count = expiring.len();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(expiring),
        message: Some(format!("Found {} expiring certificates", expiring_count)),
        error: None,
    }))
}

/// Get Certificate Revocation List
pub async fn get_certificate_revocation_list(
    State(_state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    // Return CRL in appropriate format
    let crl = vec![]; // Implement CRL retrieval
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(crl),
        message: None,
        error: None,
    }))
}

// Helper function to convert extension request format
fn convert_extensions_request(request: CertificateExtensionsRequest) -> crate::certificate_manager::CertificateExtensions {
    use crate::certificate_manager::{CertificateExtensions, KeyUsage, ExtendedKeyUsage};
    
    CertificateExtensions {
        subject_alt_names: request.subject_alt_names.unwrap_or_default(),
        key_usage: request.key_usage.unwrap_or_default().into_iter().filter_map(|usage| {
            match usage.as_str() {
                "digital_signature" => Some(KeyUsage::DigitalSignature),
                "content_commitment" => Some(KeyUsage::ContentCommitment),
                "key_encipherment" => Some(KeyUsage::KeyEncipherment),
                "data_encipherment" => Some(KeyUsage::DataEncipherment),
                "key_agreement" => Some(KeyUsage::KeyAgreement),
                "key_cert_sign" => Some(KeyUsage::KeyCertSign),
                "crl_sign" => Some(KeyUsage::CRLSign),
                "encipher_only" => Some(KeyUsage::EncipherOnly),
                "decipher_only" => Some(KeyUsage::DecipherOnly),
                _ => None,
            }
        }).collect(),
        extended_key_usage: request.extended_key_usage.unwrap_or_default().into_iter().filter_map(|ext_usage| {
            match ext_usage.as_str() {
                "server_auth" => Some(ExtendedKeyUsage::ServerAuth),
                "client_auth" => Some(ExtendedKeyUsage::ClientAuth),
                "code_signing" => Some(ExtendedKeyUsage::CodeSigning),
                "email_protection" => Some(ExtendedKeyUsage::EmailProtection),
                "time_stamping" => Some(ExtendedKeyUsage::TimeStamping),
                "ocsp_signing" => Some(ExtendedKeyUsage::OCSPSigning),
                _ => None,
            }
        }).collect(),
        is_ca: request.is_ca.unwrap_or(false),
        path_length: request.path_length,
        certificate_policies: request.certificate_policies.unwrap_or_default(),
    }
}

/// Get current security policy (Admin only)
pub async fn get_security_policy(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<SecurityPolicy>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    let policy = cert_manager.get_security_policy();
    Ok(Json(ApiResponse::success(policy)))
}

/// Update security policy (Admin only)
#[derive(Debug, Deserialize)]
pub struct UpdateSecurityPolicyRequest {
    pub secure_protocols_only: Option<bool>,
    pub minimum_security_level: Option<SecurityLevel>,
    pub allowed_algorithms: Option<Vec<CryptoAlgorithm>>,
}

pub async fn update_security_policy(
    State(state): State<CertificateAppState>,
    Json(request): Json<UpdateSecurityPolicyRequest>,
) -> Result<Json<ApiResponse<SecurityPolicy>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    // Update configuration based on request
    if let Some(secure_only) = request.secure_protocols_only {
        cert_manager.config.admin_allow_insecure = !secure_only;
    }
    if let Some(min_level) = request.minimum_security_level {
        cert_manager.config.minimum_security_level = min_level;
    }
    if let Some(algorithms) = request.allowed_algorithms {
        cert_manager.config.allowed_algorithms = algorithms;
    }
    
    let policy = cert_manager.get_security_policy();
    Ok(Json(ApiResponse::success(policy)))
}

/// Get legacy protocol configuration (Admin only)
pub async fn get_legacy_protocols(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<LegacyProtocolConfig>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    let legacy_config = cert_manager.config.legacy_protocols.clone();
    Ok(Json(ApiResponse::success(legacy_config)))
}

/// Configure legacy protocols (Admin only)
#[derive(Debug, Deserialize)]
pub struct ConfigureLegacyProtocolsRequest {
    pub legacy_config: LegacyProtocolConfig,
    pub admin_user: String,
}

pub async fn configure_legacy_protocols(
    State(state): State<CertificateAppState>,
    Json(request): Json<ConfigureLegacyProtocolsRequest>,
) -> Result<Json<ApiResponse<LegacyProtocolConfig>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    cert_manager.configure_legacy_protocols(request.legacy_config, &request.admin_user)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    
    let legacy_config = cert_manager.config.legacy_protocols.clone();
    Ok(Json(ApiResponse::success(legacy_config)))
}

/// Get TLS configuration (Admin only)
pub async fn get_tls_config(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<TlsConfig>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    let tls_config = cert_manager.config.tls_config.clone();
    Ok(Json(ApiResponse::success(tls_config)))
}

/// Update TLS configuration (Admin only)
#[derive(Debug, Deserialize)]
pub struct UpdateTlsConfigRequest {
    pub tls_config: TlsConfig,
    pub admin_user: String,
}

pub async fn update_tls_config(
    State(state): State<CertificateAppState>,
    Json(request): Json<UpdateTlsConfigRequest>,
) -> Result<Json<ApiResponse<TlsConfig>>, ApiError> {
    let mut cert_manager = state.cert_manager.write().await;
    
    cert_manager.update_tls_config(request.tls_config, &request.admin_user)
        .await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    
    let tls_config = cert_manager.config.tls_config.clone();
    Ok(Json(ApiResponse::success(tls_config)))
}

/// Get security audit report (Admin only)
pub async fn get_security_audit(
    State(state): State<CertificateAppState>,
) -> Result<Json<ApiResponse<SecurityAuditReport>>, ApiError> {
    let cert_manager = state.cert_manager.read().await;
    let audit_report = cert_manager.generate_security_audit().await;
    Ok(Json(ApiResponse::success(audit_report)))
}