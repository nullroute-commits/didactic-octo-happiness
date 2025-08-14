//! Role-Based Access Control (RBAC) system for Automation Nation
//! 
//! This module provides comprehensive authentication and authorization capabilities
//! for both the orchestrator web interface and deployed applications.
//! 
//! Features:
//! - JWT-based authentication with configurable token expiration
//! - Role-based authorization with granular permissions
//! - Session management with Redis backend support
//! - API key authentication for service-to-service communication
//! - Audit logging for security compliance
//! - Integration with external identity providers (future extensibility)

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};

/// User authentication and profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,
    /// Username for authentication
    pub username: String,
    /// Email address for notifications and recovery
    pub email: String,
    /// Display name for UI
    pub display_name: String,
    /// Encrypted password hash (using bcrypt)
    pub password_hash: String,
    /// Assigned roles for authorization
    pub roles: Vec<String>,
    /// User status for access control
    pub status: UserStatus,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last successful login timestamp
    pub last_login: Option<DateTime<Utc>>,
    /// Account expiration (None = never expires)
    pub expires_at: Option<DateTime<Utc>>,
    /// Additional metadata for extensibility
    pub metadata: HashMap<String, String>,
}

/// User account status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    /// Active account with full access
    Active,
    /// Temporarily disabled account
    Disabled,
    /// Account locked due to security policy violation
    Locked,
    /// Account pending activation
    Pending,
    /// Expired account requiring renewal
    Expired,
}

/// Role definition with associated permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier
    pub name: String,
    /// Human-readable role description
    pub description: String,
    /// Set of permissions granted by this role
    pub permissions: HashSet<Permission>,
    /// Whether this role can be assigned by other users
    pub assignable: bool,
    /// Role creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Permission enumeration for fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    // System administration permissions
    /// Read system configuration and status
    SystemRead,
    /// Modify system configuration
    SystemWrite,
    /// View system logs and audit trails
    SystemLogs,
    /// Manage system users and roles
    SystemUserManagement,
    
    // Container orchestration permissions
    /// List and view container deployments
    ContainerRead,
    /// Create new container deployments
    ContainerCreate,
    /// Modify existing container deployments
    ContainerUpdate,
    /// Delete container deployments
    ContainerDelete,
    /// Execute commands in containers
    ContainerExec,
    /// View container logs
    ContainerLogs,
    
    // Repository and deployment permissions
    /// Access GitHub repository integration
    RepositoryRead,
    /// Deploy from repositories
    RepositoryDeploy,
    /// Manage deployment profiles
    DeploymentProfileManage,
    
    // Monitoring and observability permissions
    /// View monitoring dashboards
    MonitoringRead,
    /// Configure monitoring settings
    MonitoringWrite,
    /// Access to raw metrics and logs
    MonitoringAdmin,
    
    // Network and infrastructure permissions
    /// View network configuration
    NetworkRead,
    /// Modify network configuration
    NetworkWrite,
    /// Access NetBox integration
    NetBoxAccess,
    
    // API and integration permissions
    /// Generate and manage API keys
    ApiKeyManage,
    /// Access administrative APIs
    ApiAdmin,
    /// Access read-only APIs
    ApiRead,
}

/// Authentication session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: Uuid,
    /// Associated user ID
    pub user_id: Uuid,
    /// JWT token for API authentication
    pub token: String,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp for idle timeout
    pub last_activity: DateTime<Utc>,
    /// Client IP address for security logging
    pub client_ip: String,
    /// User agent string for session tracking
    pub user_agent: String,
    /// Session metadata for extensibility
    pub metadata: HashMap<String, String>,
}

/// API key for service-to-service authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique API key identifier
    pub id: Uuid,
    /// Human-readable key name/description
    pub name: String,
    /// The actual API key value (hashed in storage)
    pub key_hash: String,
    /// Owner user ID
    pub owner_id: Uuid,
    /// Permissions granted to this API key
    pub permissions: HashSet<Permission>,
    /// Key creation timestamp
    pub created_at: DateTime<Utc>,
    /// Key expiration (None = never expires)
    pub expires_at: Option<DateTime<Utc>>,
    /// Last usage timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Usage counter for monitoring
    pub usage_count: u64,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimit>,
}

/// Rate limiting configuration for API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u32,
    /// Current request count in window
    pub current_count: u32,
    /// Window start timestamp
    pub window_start: DateTime<Utc>,
}

/// Audit log entry for security compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique audit entry ID
    pub id: Uuid,
    /// User who performed the action (None for system actions)
    pub user_id: Option<Uuid>,
    /// API key used for the action (for service authentication)
    pub api_key_id: Option<Uuid>,
    /// Action performed
    pub action: String,
    /// Resource affected by the action
    pub resource: String,
    /// Additional details about the action
    pub details: HashMap<String, String>,
    /// Action timestamp
    pub timestamp: DateTime<Utc>,
    /// Client IP address
    pub client_ip: String,
    /// Success status of the action
    pub success: bool,
    /// Error message if action failed
    pub error_message: Option<String>,
}

/// Main RBAC manager for authentication and authorization
pub struct RbacManager {
    /// User storage backend
    users: HashMap<Uuid, User>,
    /// Role definitions storage
    roles: HashMap<String, Role>,
    /// Active sessions storage
    sessions: HashMap<Uuid, Session>,
    /// API keys storage
    api_keys: HashMap<Uuid, ApiKey>,
    /// Audit log storage (in production, this would be persisted)
    audit_log: Vec<AuditLogEntry>,
    /// JWT secret for token signing
    #[allow(dead_code)]
    jwt_secret: String,
    /// Default session duration
    session_duration: Duration,
    /// Default API key duration
    api_key_duration: Option<Duration>,
}

impl RbacManager {
    /// Create a new RBAC manager with default configuration
    pub fn new(jwt_secret: String) -> Self {
        let mut manager = Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            sessions: HashMap::new(),
            api_keys: HashMap::new(),
            audit_log: Vec::new(),
            jwt_secret,
            session_duration: Duration::hours(24), // 24 hour default
            api_key_duration: Some(Duration::days(365)), // 1 year default
        };
        
        // Initialize default roles
        manager.initialize_default_roles();
        
        // Create default admin user if none exists
        if manager.users.is_empty() {
            if let Err(e) = manager.create_default_admin() {
                error!("Failed to create default admin user: {}", e);
            }
        }
        
        manager
    }
    
    /// Initialize default system roles
    fn initialize_default_roles(&mut self) {
        // Super Administrator role with all permissions
        let admin_permissions = vec![
            Permission::SystemRead, Permission::SystemWrite, Permission::SystemLogs,
            Permission::SystemUserManagement, Permission::ContainerRead, Permission::ContainerCreate,
            Permission::ContainerUpdate, Permission::ContainerDelete, Permission::ContainerExec,
            Permission::ContainerLogs, Permission::RepositoryRead, Permission::RepositoryDeploy,
            Permission::DeploymentProfileManage, Permission::MonitoringRead, Permission::MonitoringWrite,
            Permission::MonitoringAdmin, Permission::NetworkRead, Permission::NetworkWrite,
            Permission::NetBoxAccess, Permission::ApiKeyManage, Permission::ApiAdmin, Permission::ApiRead,
        ].into_iter().collect();
        
        self.roles.insert("admin".to_string(), Role {
            name: "admin".to_string(),
            description: "System Administrator with full access".to_string(),
            permissions: admin_permissions,
            assignable: false, // Only system can assign admin role
            created_at: Utc::now(),
        });
        
        // Operations role for container and deployment management
        let ops_permissions = vec![
            Permission::SystemRead, Permission::ContainerRead, Permission::ContainerCreate,
            Permission::ContainerUpdate, Permission::ContainerDelete, Permission::ContainerLogs,
            Permission::RepositoryRead, Permission::RepositoryDeploy, Permission::DeploymentProfileManage,
            Permission::MonitoringRead, Permission::NetworkRead, Permission::ApiRead,
        ].into_iter().collect();
        
        self.roles.insert("operator".to_string(), Role {
            name: "operator".to_string(),
            description: "Operations Engineer with deployment management access".to_string(),
            permissions: ops_permissions,
            assignable: true,
            created_at: Utc::now(),
        });
        
        // Developer role for application deployment
        let dev_permissions = vec![
            Permission::ContainerRead, Permission::ContainerCreate, Permission::ContainerLogs,
            Permission::RepositoryRead, Permission::RepositoryDeploy, Permission::MonitoringRead,
            Permission::ApiRead,
        ].into_iter().collect();
        
        self.roles.insert("developer".to_string(), Role {
            name: "developer".to_string(),
            description: "Developer with application deployment access".to_string(),
            permissions: dev_permissions,
            assignable: true,
            created_at: Utc::now(),
        });
        
        // Read-only viewer role
        let viewer_permissions = vec![
            Permission::SystemRead, Permission::ContainerRead, Permission::RepositoryRead,
            Permission::MonitoringRead, Permission::NetworkRead, Permission::ApiRead,
        ].into_iter().collect();
        
        self.roles.insert("viewer".to_string(), Role {
            name: "viewer".to_string(),
            description: "Read-only access to system information".to_string(),
            permissions: viewer_permissions,
            assignable: true,
            created_at: Utc::now(),
        });
        
        info!("Initialized {} default RBAC roles", self.roles.len());
    }
    
    /// Create default admin user for initial setup
    fn create_default_admin(&mut self) -> Result<Uuid> {
        let admin_id = Uuid::new_v4();
        let default_password = "admin123"; // Should be changed on first login
        
        let password_hash = bcrypt::hash(default_password, bcrypt::DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        
        let admin_user = User {
            id: admin_id,
            username: "admin".to_string(),
            email: "admin@automation-nation.local".to_string(),
            display_name: "System Administrator".to_string(),
            password_hash,
            roles: vec!["admin".to_string()],
            status: UserStatus::Active,
            created_at: Utc::now(),
            last_login: None,
            expires_at: None,
            metadata: HashMap::new(),
        };
        
        self.users.insert(admin_id, admin_user);
        
        self.log_audit_event(
            None,
            "user_create".to_string(),
            format!("user:{}", admin_id),
            "system".to_string(),
            true,
            None,
            [("username".to_string(), "admin".to_string())].into(),
        );
        
        warn!("Created default admin user with password '{}' - CHANGE IMMEDIATELY!", default_password);
        
        Ok(admin_id)
    }
    
    /// Authenticate user with username/password
    pub fn authenticate(&mut self, username: &str, password: &str, client_ip: &str, user_agent: &str) -> Result<Session> {
        debug!("Authentication attempt for user: {}", username);
        
        // Find user by username and clone user data to avoid borrowing issues
        let user_data = {
            let user = self.users.values()
                .find(|u| u.username == username)
                .ok_or_else(|| anyhow!("Invalid username or password"))?;
            
            // Check user status
            if user.status != UserStatus::Active {
                self.log_audit_event(
                    Some(user.id),
                    "auth_failed".to_string(),
                    format!("user:{}", user.id),
                    client_ip.to_string(),
                    false,
                    Some("User account not active".to_string()),
                    HashMap::new(),
                );
                return Err(anyhow!("User account is not active"));
            }
            
            // Check password
            if !bcrypt::verify(password, &user.password_hash)
                .map_err(|e| anyhow!("Password verification failed: {}", e))? {
                self.log_audit_event(
                    Some(user.id),
                    "auth_failed".to_string(),
                    format!("user:{}", user.id),
                    client_ip.to_string(),
                    false,
                    Some("Invalid password".to_string()),
                    HashMap::new(),
                );
                return Err(anyhow!("Invalid username or password"));
            }
            
            // Clone user data for later use
            (user.id, user.username.clone())
        };
        
        // Create session
        let session = self.create_session(user_data.0, client_ip, user_agent)?;
        
        // Update last login
        if let Some(user) = self.users.get_mut(&user_data.0) {
            user.last_login = Some(Utc::now());
        }
        
        self.log_audit_event(
            Some(user_data.0),
            "auth_success".to_string(),
            format!("user:{}", user_data.0),
            client_ip.to_string(),
            true,
            None,
            [("session_id".to_string(), session.id.to_string())].into(),
        );
        
        info!("User {} authenticated successfully", user_data.1);
        Ok(session)
    }
    
    /// Create a new session for authenticated user
    fn create_session(&mut self, user_id: Uuid, client_ip: &str, user_agent: &str) -> Result<Session> {
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + self.session_duration;
        
        // Generate JWT token (simplified - in production use proper JWT library)
        let token = format!("{}:{}:{}", session_id, user_id, expires_at.timestamp());
        
        let session = Session {
            id: session_id,
            user_id,
            token: token.clone(),
            created_at: now,
            expires_at,
            last_activity: now,
            client_ip: client_ip.to_string(),
            user_agent: user_agent.to_string(),
            metadata: HashMap::new(),
        };
        
        self.sessions.insert(session_id, session.clone());
        Ok(session)
    }
    
    /// Validate session and check permissions
    pub fn authorize(&mut self, token: &str, required_permission: Permission) -> Result<User> {
        // Find session by token and extract necessary data
        let session_data = {
            let session = self.sessions.values()
                .find(|s| s.token == token)
                .ok_or_else(|| anyhow!("Invalid or expired session"))?;
            
            // Check session expiration
            if session.expires_at < Utc::now() {
                return Err(anyhow!("Session expired"));
            }
            
            (session.id, session.user_id, session.client_ip.clone())
        };
        
        // Remove expired session if needed
        if self.sessions.get(&session_data.0).map(|s| s.expires_at < Utc::now()).unwrap_or(false) {
            self.sessions.remove(&session_data.0);
            return Err(anyhow!("Session expired"));
        }
        
        // Get user
        let user = self.users.get(&session_data.1)
            .ok_or_else(|| anyhow!("User not found"))?;
        
        // Check if user has required permission
        if !self.user_has_permission(user, &required_permission) {
            self.log_audit_event(
                Some(user.id),
                "authz_failed".to_string(),
                format!("permission:{:?}", required_permission),
                session_data.2,
                false,
                Some("Insufficient permissions".to_string()),
                HashMap::new(),
            );
            return Err(anyhow!("Insufficient permissions"));
        }
        
        // Update session activity
        if let Some(session) = self.sessions.get_mut(&session_data.0) {
            session.last_activity = Utc::now();
        }
        
        Ok(user.clone())
    }
    
    /// Check if user has specific permission
    fn user_has_permission(&self, user: &User, permission: &Permission) -> bool {
        for role_name in &user.roles {
            if let Some(role) = self.roles.get(role_name) {
                if role.permissions.contains(permission) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Create API key for service authentication
    pub fn create_api_key(&mut self, owner_id: Uuid, name: String, permissions: HashSet<Permission>) -> Result<(Uuid, String)> {
        let api_key_id = Uuid::new_v4();
        let raw_key = format!("ak_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let key_hash = bcrypt::hash(&raw_key, bcrypt::DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash API key: {}", e))?;
        
        let expires_at = self.api_key_duration.map(|d| Utc::now() + d);
        
        let api_key = ApiKey {
            id: api_key_id,
            name,
            key_hash,
            owner_id,
            permissions,
            created_at: Utc::now(),
            expires_at,
            last_used: None,
            usage_count: 0,
            rate_limit: Some(RateLimit {
                max_requests: 1000,
                window_seconds: 3600, // 1 hour window
                current_count: 0,
                window_start: Utc::now(),
            }),
        };
        
        self.api_keys.insert(api_key_id, api_key);
        
        self.log_audit_event(
            Some(owner_id),
            "api_key_create".to_string(),
            format!("api_key:{}", api_key_id),
            "system".to_string(),
            true,
            None,
            HashMap::new(),
        );
        
        Ok((api_key_id, raw_key))
    }
    
    /// Validate API key and check permissions
    pub fn authorize_api_key(&mut self, api_key: &str, required_permission: Permission) -> Result<Uuid> {
        // Find API key
        let key_entry = self.api_keys.values_mut()
            .find(|k| bcrypt::verify(api_key, &k.key_hash).unwrap_or(false))
            .ok_or_else(|| anyhow!("Invalid API key"))?;
        
        // Check expiration
        if let Some(expires_at) = key_entry.expires_at {
            if expires_at < Utc::now() {
                return Err(anyhow!("API key expired"));
            }
        }
        
        // Check permissions
        if !key_entry.permissions.contains(&required_permission) {
            return Err(anyhow!("API key lacks required permission"));
        }
        
        // Check rate limit
        if let Some(ref mut rate_limit) = key_entry.rate_limit {
            let now = Utc::now();
            let window_elapsed = (now - rate_limit.window_start).num_seconds() as u32;
            
            if window_elapsed >= rate_limit.window_seconds {
                // Reset window
                rate_limit.window_start = now;
                rate_limit.current_count = 0;
            }
            
            if rate_limit.current_count >= rate_limit.max_requests {
                return Err(anyhow!("Rate limit exceeded"));
            }
            
            rate_limit.current_count += 1;
        }
        
        // Update usage
        key_entry.last_used = Some(Utc::now());
        key_entry.usage_count += 1;
        
        Ok(key_entry.owner_id)
    }
    
    /// Log audit event for security compliance
    fn log_audit_event(
        &mut self,
        user_id: Option<Uuid>,
        action: String,
        resource: String,
        client_ip: String,
        success: bool,
        error_message: Option<String>,
        details: HashMap<String, String>,
    ) {
        let entry = AuditLogEntry {
            id: Uuid::new_v4(),
            user_id,
            api_key_id: None, // TODO: Track API key usage
            action,
            resource,
            details,
            timestamp: Utc::now(),
            client_ip,
            success,
            error_message,
        };
        
        self.audit_log.push(entry);
        
        // In production, this would be written to persistent storage
        // and potentially forwarded to SIEM systems
    }
    
    /// Get user by ID
    pub fn get_user(&self, user_id: Uuid) -> Option<&User> {
        self.users.get(&user_id)
    }
    
    /// Get all roles (for administration)
    pub fn get_roles(&self) -> &HashMap<String, Role> {
        &self.roles
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = Utc::now();
        let expired_sessions: Vec<Uuid> = self.sessions
            .iter()
            .filter(|(_, session)| session.expires_at < now)
            .map(|(id, _)| *id)
            .collect();
        
        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
        }
    }
    
    /// Get audit log entries (for security monitoring)
    pub fn get_audit_log(&self, limit: Option<usize>) -> Vec<&AuditLogEntry> {
        let mut entries: Vec<&AuditLogEntry> = self.audit_log.iter().collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        match limit {
            Some(n) => entries.into_iter().take(n).collect(),
            None => entries,
        }
    }
    
    /// Add a test user (for testing purposes only)
    #[cfg(test)]
    pub fn add_test_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }
    
    /// Get first admin user ID (for testing purposes only)
    #[cfg(test)]
    pub fn get_admin_user_id(&self) -> Option<Uuid> {
        self.users.values()
            .find(|u| u.roles.contains(&"admin".to_string()))
            .map(|u| u.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rbac_manager_creation() {
        let manager = RbacManager::new("test_secret".to_string());
        assert_eq!(manager.roles.len(), 4); // admin, operator, developer, viewer
        assert_eq!(manager.users.len(), 1); // default admin user
    }
    
    #[test]
    fn test_user_authentication() {
        let mut manager = RbacManager::new("test_secret".to_string());
        
        // Should be able to authenticate with default admin
        let result = manager.authenticate("admin", "admin123", "127.0.0.1", "test-agent");
        assert!(result.is_ok());
        
        // Should fail with wrong password
        let result = manager.authenticate("admin", "wrong", "127.0.0.1", "test-agent");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_permission_checking() {
        let manager = RbacManager::new("test_secret".to_string());
        let admin_user = manager.users.values().next().unwrap();
        
        // Admin should have all permissions
        assert!(manager.user_has_permission(admin_user, &Permission::SystemWrite));
        assert!(manager.user_has_permission(admin_user, &Permission::ContainerDelete));
    }
    
    #[test]
    fn test_api_key_creation() {
        let mut manager = RbacManager::new("test_secret".to_string());
        let admin_id = manager.users.values().next().unwrap().id;
        
        let permissions = [Permission::ApiRead].iter().cloned().collect();
        let result = manager.create_api_key(admin_id, "test-key".to_string(), permissions);
        
        assert!(result.is_ok());
        let (key_id, raw_key) = result.unwrap();
        assert!(raw_key.starts_with("ak_"));
        assert!(manager.api_keys.contains_key(&key_id));
    }
}