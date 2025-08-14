//! Database-backed RBAC (Role-Based Access Control) implementation
//! 
//! This module provides a database-backed implementation of the RBAC system
//! using PostgreSQL for persistent storage of users, roles, sessions, and audit logs.

use crate::database::DatabaseManager;
use crate::rbac::{User, Session, UserStatus};
use sqlx::{PgPool, Row};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{Utc, Duration};
use bcrypt;
use log::{info, debug};
use std::collections::HashMap;
use serde_json;

/// Database-backed RBAC manager
pub struct DatabaseRbacManager {
    pool: PgPool,
    #[allow(dead_code)]
    jwt_secret: String,
    session_duration: Duration,
    api_key_duration: Option<Duration>,
}

impl DatabaseRbacManager {
    /// Create a new database-backed RBAC manager
    pub async fn new(database_manager: &DatabaseManager, jwt_secret: String) -> Result<Self> {
        let manager = Self {
            pool: database_manager.pool().clone(),
            jwt_secret,
            session_duration: Duration::hours(24),
            api_key_duration: Some(Duration::days(365)),
        };
        
        info!("Database-backed RBAC manager initialized");
        Ok(manager)
    }
    
    /// Authenticate user with username/password
    pub async fn authenticate(&self, username: &str, password: &str, client_ip: &str, user_agent: &str) -> Result<Session> {
        debug!("Authentication attempt for user: {}", username);
        
        // Find user by username
        let user = self.get_user_by_username(username).await?
            .ok_or_else(|| anyhow!("Invalid username or password"))?;
        
        // Verify password
        if !bcrypt::verify(password, &user.password_hash)
            .map_err(|e| anyhow!("Password verification failed: {}", e))? {
            self.log_audit_event(
                Some(user.id),
                "authentication_failed".to_string(),
                format!("user:{}", user.id),
                client_ip.to_string(),
                false,
                Some("Invalid password".to_string()),
                [("username".to_string(), username.to_string())].into(),
            ).await?;
            return Err(anyhow!("Invalid username or password"));
        }
        
        // Check user status
        if user.status != UserStatus::Active {
            self.log_audit_event(
                Some(user.id),
                "authentication_failed".to_string(),
                format!("user:{}", user.id),
                client_ip.to_string(),
                false,
                Some(format!("User status: {:?}", user.status)),
                [("username".to_string(), username.to_string())].into(),
            ).await?;
            return Err(anyhow!("User account is not active"));
        }
        
        // Create session
        let session = self.create_session(user.id, client_ip, user_agent).await?;
        
        // Update last login
        self.update_user_last_login(user.id).await?;
        
        // Log successful authentication
        self.log_audit_event(
            Some(user.id),
            "authentication_success".to_string(),
            format!("user:{}", user.id),
            client_ip.to_string(),
            true,
            None,
            [
                ("username".to_string(), username.to_string()),
                ("session_id".to_string(), session.id.to_string()),
            ].into(),
        ).await?;
        
        info!("User {} authenticated successfully", username);
        Ok(session)
    }
    
    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let result = sqlx::query(
            "SELECT id, username, email, display_name, password_hash, status, created_at, last_login, expires_at, metadata FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to query user: {}", e))?;
        
        if let Some(row) = result {
            let user = self.row_to_user(row)?;
            // Get user roles
            let roles = self.get_user_roles(user.id).await?;
            let mut user_with_roles = user;
            user_with_roles.roles = roles;
            Ok(Some(user_with_roles))
        } else {
            Ok(None)
        }
    }
    
    /// Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        let result = sqlx::query(
            "SELECT id, username, email, display_name, password_hash, status, created_at, last_login, expires_at, metadata FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to query user: {}", e))?;
        
        if let Some(row) = result {
            let user = self.row_to_user(row)?;
            // Get user roles
            let roles = self.get_user_roles(user.id).await?;
            let mut user_with_roles = user;
            user_with_roles.roles = roles;
            Ok(Some(user_with_roles))
        } else {
            Ok(None)
        }
    }
    
    /// Get user roles
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<String>> {
        let results = sqlx::query(
            "SELECT role_name FROM user_roles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to query user roles: {}", e))?;
        
        Ok(results.into_iter().map(|row| row.get("role_name")).collect())
    }
    
    /// Create a new session
    async fn create_session(&self, user_id: Uuid, client_ip: &str, user_agent: &str) -> Result<Session> {
        let session_id = Uuid::new_v4();
        let token = format!("{}_{}", session_id, Uuid::new_v4());
        let expires_at = Utc::now() + self.session_duration;
        
        sqlx::query(
            "INSERT INTO sessions (id, user_id, token, client_ip, user_agent, expires_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&token)
        .bind(client_ip)
        .bind(user_agent)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to create session: {}", e))?;
        
        Ok(Session {
            id: session_id,
            user_id,
            token,
            client_ip: client_ip.to_string(),
            user_agent: user_agent.to_string(),
            created_at: Utc::now(),
            expires_at,
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        })
    }
    
    /// Update user last login timestamp
    async fn update_user_last_login(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to update last login: {}", e))?;
        Ok(())
    }
    
    /// Log audit event
    async fn log_audit_event(
        &self,
        user_id: Option<Uuid>,
        action: String,
        resource: String,
        client_ip: String,
        success: bool,
        error_message: Option<String>,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let metadata_json = serde_json::to_value(metadata)
            .map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;
        
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, resource, client_ip, success, error_message, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(user_id)
        .bind(action)
        .bind(resource)
        .bind(client_ip)
        .bind(success)
        .bind(error_message)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to log audit event: {}", e))?;
        
        Ok(())
    }
    
    /// Convert database row to User struct
    fn row_to_user(&self, row: sqlx::postgres::PgRow) -> Result<User> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "active" => UserStatus::Active,
            "disabled" => UserStatus::Disabled,
            "locked" => UserStatus::Locked,
            "pending" => UserStatus::Pending,
            "expired" => UserStatus::Expired,
            _ => UserStatus::Disabled,
        };
        
        let metadata: serde_json::Value = row.get("metadata");
        let metadata_map: HashMap<String, String> = serde_json::from_value(metadata)
            .unwrap_or_default();
        
        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            password_hash: row.get("password_hash"),
            roles: vec![], // Will be populated separately
            status,
            created_at: row.get("created_at"),
            last_login: row.get("last_login"),
            expires_at: row.get("expires_at"),
            metadata: metadata_map,
        })
    }
    
    /// Get admin user ID (for compatibility with existing tests)
    pub async fn get_admin_user_id(&self) -> Result<Option<Uuid>> {
        let result = sqlx::query(
            "SELECT id FROM users WHERE username = 'admin'"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to query admin user: {}", e))?;
        
        Ok(result.map(|row| row.get("id")))
    }
    
    /// Create a test user (for testing purposes)
    pub async fn create_test_user(&self, user: User) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, email, display_name, password_hash, status, created_at, last_login, expires_at, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(user.id)
        .bind(user.username)
        .bind(user.email)
        .bind(user.display_name)
        .bind(user.password_hash)
        .bind(match user.status {
            UserStatus::Active => "active",
            UserStatus::Disabled => "disabled",
            UserStatus::Locked => "locked",
            UserStatus::Pending => "pending",
            UserStatus::Expired => "expired",
        })
        .bind(user.created_at)
        .bind(user.last_login)
        .bind(user.expires_at)
        .bind(serde_json::to_value(user.metadata).unwrap_or_default())
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to create test user: {}", e))?;
        
        // Add user roles
        for role in &user.roles {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_name) VALUES ($1, $2)"
            )
            .bind(user.id)
            .bind(role)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to assign role: {}", e))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_test_database;
    use tokio_test;
    
    #[tokio::test]
    async fn test_database_rbac_manager_creation() {
        if let Ok(db) = create_test_database().await {
            let rbac_manager = DatabaseRbacManager::new(&db, "test_secret".to_string()).await;
            assert!(rbac_manager.is_ok());
        } else {
            println!("Skipping test - no test database available");
        }
    }
}