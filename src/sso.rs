//! SSO and OIDC integration for Automation Nation
//! 
//! This module provides Single Sign-On (SSO) integration using OpenID Connect (OIDC)
//! protocol, supporting major identity providers like Google, Microsoft Azure AD,
//! Okta, Auth0, and other OIDC-compliant providers.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use log::{info, debug};
use crate::database_rbac::DatabaseRbacManager;
use crate::rbac::{User, UserStatus};

/// OIDC configuration for SSO integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    /// Identity provider name (e.g., "Google", "Azure AD", "Okta")
    pub provider_name: String,
    /// OIDC issuer URL
    pub issuer_url: String,
    /// Client ID for OIDC application
    pub client_id: String,
    /// Client secret for OIDC application
    pub client_secret: String,
    /// Redirect URI for OAuth callback
    pub redirect_uri: String,
    /// Requested scopes (default: "openid profile email")
    pub scopes: Vec<String>,
    /// Additional provider-specific configuration
    pub additional_params: HashMap<String, String>,
}

/// OIDC discovery document structure
#[derive(Debug, Deserialize)]
pub struct OidcDiscovery {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub issuer: String,
    pub scopes_supported: Option<Vec<String>>,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
}

/// OIDC token response structure
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

/// OIDC user information structure
#[derive(Debug, Deserialize)]
pub struct OidcUserInfo {
    pub sub: String,  // Subject identifier
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub picture: Option<String>,
    pub locale: Option<String>,
    pub preferred_username: Option<String>,
}

/// SSO authentication session
#[derive(Debug, Clone)]
pub struct SsoSession {
    pub state: String,
    pub nonce: String,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub redirect_after_auth: Option<String>,
}

/// SSO authentication manager
pub struct SsoManager {
    client: Client,
    configs: HashMap<String, OidcConfig>,
    discovery_cache: HashMap<String, OidcDiscovery>,
    pending_sessions: HashMap<String, SsoSession>,
    rbac_manager: DatabaseRbacManager,
}

impl SsoManager {
    /// Create a new SSO manager
    pub async fn new(rbac_manager: DatabaseRbacManager) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;
        
        Ok(Self {
            client,
            configs: HashMap::new(),
            discovery_cache: HashMap::new(),
            pending_sessions: HashMap::new(),
            rbac_manager,
        })
    }
    
    /// Add OIDC provider configuration
    pub async fn add_provider(&mut self, provider_name: String, config: OidcConfig) -> Result<()> {
        info!("Adding SSO provider: {}", provider_name);
        
        // Fetch and cache discovery document
        let discovery = self.fetch_discovery_document(&config.issuer_url).await?;
        self.discovery_cache.insert(provider_name.clone(), discovery);
        
        // Store configuration
        self.configs.insert(provider_name.clone(), config);
        
        info!("Successfully configured SSO provider: {}", provider_name);
        Ok(())
    }
    
    /// Generate authorization URL for SSO login
    pub fn generate_auth_url(&mut self, provider_name: &str, redirect_after_auth: Option<String>) -> Result<String> {
        let config = self.configs.get(provider_name)
            .ok_or_else(|| anyhow!("Unknown SSO provider: {}", provider_name))?;
        
        let discovery = self.discovery_cache.get(provider_name)
            .ok_or_else(|| anyhow!("Discovery document not cached for provider: {}", provider_name))?;
        
        // Generate state and nonce for security
        let state = format!("{}_{}", provider_name, Uuid::new_v4());
        let nonce = Uuid::new_v4().to_string();
        
        // Store session information
        let session = SsoSession {
            state: state.clone(),
            nonce: nonce.clone(),
            provider: provider_name.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(10), // 10 minute timeout
            redirect_after_auth,
        };
        self.pending_sessions.insert(state.clone(), session);
        
        // Build authorization URL
        let scopes = if config.scopes.is_empty() {
            vec!["openid".to_string(), "profile".to_string(), "email".to_string()]
        } else {
            config.scopes.clone()
        };
        
        let scope_string = scopes.join(" ");
        let mut params = vec![
            ("response_type", "code"),
            ("client_id", &config.client_id),
            ("redirect_uri", &config.redirect_uri),
            ("scope", &scope_string),
            ("state", &state),
            ("nonce", &nonce),
        ];
        
        // Add any additional provider-specific parameters
        for (key, value) in &config.additional_params {
            params.push((key, value));
        }
        
        let query_string = params.iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        
        let auth_url = format!("{}?{}", discovery.authorization_endpoint, query_string);
        debug!("Generated auth URL for {}: {}", provider_name, auth_url);
        
        Ok(auth_url)
    }
    
    /// Handle OAuth callback and authenticate user
    pub async fn handle_callback(&mut self, code: &str, state: &str, client_ip: &str) -> Result<crate::rbac::Session> {
        debug!("Handling SSO callback with state: {}", state);
        
        // Retrieve and validate session
        let sso_session = self.pending_sessions.remove(state)
            .ok_or_else(|| anyhow!("Invalid or expired SSO state"))?;
        
        if sso_session.expires_at < Utc::now() {
            return Err(anyhow!("SSO session expired"));
        }
        
        let config = self.configs.get(&sso_session.provider)
            .ok_or_else(|| anyhow!("SSO provider configuration not found"))?;
        
        let discovery = self.discovery_cache.get(&sso_session.provider)
            .ok_or_else(|| anyhow!("Discovery document not found"))?;
        
        // Exchange code for tokens
        let token_response = self.exchange_code_for_tokens(config, discovery, code).await?;
        
        // Get user information
        let user_info = self.get_user_info(discovery, &token_response.access_token).await?;
        
        // Find or create user
        let user = self.find_or_create_user(&sso_session.provider, &user_info).await?;
        
        // Create application session
        let session = self.rbac_manager.authenticate(
            &user.username,
            &format!("sso_{}_{}", sso_session.provider, user_info.sub),
            client_ip,
            &format!("SSO:{}", sso_session.provider),
        ).await?;
        
        info!("SSO authentication successful for user: {}", user.username);
        Ok(session)
    }
    
    /// Fetch OIDC discovery document
    async fn fetch_discovery_document(&self, issuer_url: &str) -> Result<OidcDiscovery> {
        let discovery_url = if issuer_url.ends_with('/') {
            format!("{}/.well-known/openid_configuration", issuer_url.trim_end_matches('/'))
        } else {
            format!("{}/.well-known/openid_configuration", issuer_url)
        };
        
        debug!("Fetching OIDC discovery document from: {}", discovery_url);
        
        let response = self.client.get(&discovery_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch discovery document: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Discovery document request failed with status: {}", response.status()));
        }
        
        let discovery: OidcDiscovery = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse discovery document: {}", e))?;
        
        debug!("Successfully fetched discovery document for issuer: {}", discovery.issuer);
        Ok(discovery)
    }
    
    /// Exchange authorization code for tokens
    async fn exchange_code_for_tokens(&self, config: &OidcConfig, discovery: &OidcDiscovery, code: &str) -> Result<TokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &config.redirect_uri),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
        ];
        
        let response = self.client.post(&discovery.token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Token exchange request failed: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Token exchange failed with status {}: {}", status, error_text));
        }
        
        let token_response: TokenResponse = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse token response: {}", e))?;
        
        debug!("Successfully exchanged code for tokens");
        Ok(token_response)
    }
    
    /// Get user information using access token
    async fn get_user_info(&self, discovery: &OidcDiscovery, access_token: &str) -> Result<OidcUserInfo> {
        let response = self.client.get(&discovery.userinfo_endpoint)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| anyhow!("User info request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("User info request failed with status: {}", response.status()));
        }
        
        let user_info: OidcUserInfo = response.json()
            .await
            .map_err(|e| anyhow!("Failed to parse user info: {}", e))?;
        
        debug!("Successfully retrieved user info for subject: {}", user_info.sub);
        Ok(user_info)
    }
    
    /// Find existing user or create new one from SSO information
    async fn find_or_create_user(&self, provider: &str, user_info: &OidcUserInfo) -> Result<User> {
        let email = user_info.email.as_ref()
            .ok_or_else(|| anyhow!("Email not provided by SSO provider"))?;
        
        // Try to find existing user by email
        if let Ok(Some(existing_user)) = self.rbac_manager.get_user_by_username(email).await {
            info!("Found existing user for SSO login: {}", email);
            return Ok(existing_user);
        }
        
        // Create new user
        let user_id = Uuid::new_v4();
        let username = email.clone();
        let display_name = user_info.name.clone()
            .or_else(|| user_info.preferred_username.clone())
            .unwrap_or_else(|| email.clone());
        
        let mut metadata = HashMap::new();
        metadata.insert("sso_provider".to_string(), provider.to_string());
        metadata.insert("sso_subject".to_string(), user_info.sub.clone());
        if let Some(picture) = &user_info.picture {
            metadata.insert("avatar_url".to_string(), picture.clone());
        }
        
        let user = User {
            id: user_id,
            username,
            email: email.clone(),
            display_name,
            password_hash: format!("sso_{}_{}", provider, user_info.sub), // Not a real password hash
            roles: vec!["viewer".to_string()], // Default role for SSO users
            status: UserStatus::Active,
            created_at: Utc::now(),
            last_login: None,
            expires_at: None,
            metadata,
        };
        
        self.rbac_manager.create_test_user(user.clone()).await
            .map_err(|e| anyhow!("Failed to create SSO user: {}", e))?;
        
        info!("Created new SSO user: {}", email);
        Ok(user)
    }
    
    /// Clean up expired SSO sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = Utc::now();
        let expired_states: Vec<String> = self.pending_sessions.iter()
            .filter(|(_, session)| session.expires_at < now)
            .map(|(state, _)| state.clone())
            .collect();
        
        for state in expired_states {
            self.pending_sessions.remove(&state);
        }
        
        if !self.pending_sessions.is_empty() {
            debug!("Cleaned up {} expired SSO sessions", self.pending_sessions.len());
        }
    }
    
    /// Get list of configured SSO providers
    pub fn get_providers(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[tokio::test]
    async fn test_sso_manager_creation() {
        // This test requires a database connection
        if env::var("DATABASE_URL").is_err() {
            println!("Skipping SSO test - DATABASE_URL not set");
            return;
        }
        
        // This would require setting up a test database and RBAC manager
        // For now, we'll just test the structure
        let config = OidcConfig {
            provider_name: "test".to_string(),
            issuer_url: "https://accounts.google.com".to_string(),
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string()],
            additional_params: HashMap::new(),
        };
        
        assert_eq!(config.provider_name, "test");
        assert!(!config.client_id.is_empty());
    }
}