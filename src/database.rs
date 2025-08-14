//! Database persistence layer for Automation Nation
//! 
//! This module provides PostgreSQL-based persistence for user management,
//! authentication, sessions, and audit logging, replacing the in-memory
//! storage used in the RBAC manager.

use sqlx::{PgPool, Row, postgres::PgRow};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;
use log::{info, debug};

use crate::rbac::{User, UserStatus, Session};

/// Database connection and query management
#[derive(Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager with connection pool
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to PostgreSQL database");
        
        let pool = PgPool::connect(database_url).await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;
        
        info!("Database connection established");
        
        Ok(Self { pool })
    }
    
    /// Get database connection pool for direct use
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    // User management methods
    
    /// Create a new user
    pub async fn create_user(&self, user: &User) -> Result<()> {
        let roles_json = serde_json::to_value(&user.roles)?;
        let metadata_json = serde_json::to_value(&user.metadata)?;
        let status_str = match user.status {
            UserStatus::Active => "active",
            UserStatus::Disabled => "disabled", 
            UserStatus::Locked => "locked",
            UserStatus::Pending => "pending",
            UserStatus::Expired => "expired",
        };
        
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, display_name, password_hash, roles, status, 
                             created_at, last_login, expires_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.password_hash)
        .bind(roles_json)
        .bind(status_str)
        .bind(user.created_at)
        .bind(user.last_login)
        .bind(user.expires_at)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;
        
        debug!("Created user {} with ID {}", user.username, user.id);
        Ok(())
    }
    
    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch user by ID: {}", e))?;
        
        match row {
            Some(row) => Ok(Some(self.row_to_user(row)?)),
            None => Ok(None),
        }
    }
    
    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch user by username: {}", e))?;
        
        match row {
            Some(row) => Ok(Some(self.row_to_user(row)?)),
            None => Ok(None),
        }
    }
    
    /// Get all users with admin role
    pub async fn get_admin_users(&self) -> Result<Vec<User>> {
        let rows = sqlx::query("SELECT * FROM users WHERE 'admin' = ANY(roles)")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch admin users: {}", e))?;
        
        let mut users = Vec::new();
        for row in rows {
            users.push(self.row_to_user(row)?);
        }
        
        Ok(users)
    }
    
    /// Update user password
    pub async fn update_user_password(&self, user_id: Uuid, new_password_hash: &str) -> Result<()> {
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(new_password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to update user password: {}", e))?;
        
        debug!("Updated password for user {}", user_id);
        Ok(())
    }
    
    /// Create password reset token
    pub async fn create_password_reset_token(&self, user_id: Uuid, token: &str, expires_at: DateTime<Utc>) -> Result<Uuid> {
        let token_id = Uuid::new_v4();
        
        sqlx::query("INSERT INTO password_reset_tokens (id, user_id, token, expires_at) VALUES ($1, $2, $3, $4)")
            .bind(token_id)
            .bind(user_id)
            .bind(token)
            .bind(expires_at)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to create password reset token: {}", e))?;
        
        debug!("Created password reset token for user {}", user_id);
        Ok(token_id)
    }
    
    /// Validate and consume password reset token
    pub async fn validate_password_reset_token(&self, token: &str) -> Result<Option<Uuid>> {
        let row = sqlx::query("SELECT user_id FROM password_reset_tokens WHERE token = $1 AND expires_at > NOW() AND used = false")
            .bind(token)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow!("Failed to validate password reset token: {}", e))?;
        
        match row {
            Some(row) => {
                let user_id: Uuid = row.try_get("user_id")?;
                
                // Mark token as used
                sqlx::query("UPDATE password_reset_tokens SET used = true WHERE token = $1")
                    .bind(token)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| anyhow!("Failed to mark password reset token as used: {}", e))?;
                
                Ok(Some(user_id))
            }
            None => Ok(None),
        }
    }
    
    // Helper methods for converting database rows to structs
    
    fn row_to_user(&self, row: PgRow) -> Result<User> {
        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "active" => UserStatus::Active,
            "disabled" => UserStatus::Disabled,
            "locked" => UserStatus::Locked,
            "pending" => UserStatus::Pending,
            "expired" => UserStatus::Expired,
            _ => return Err(anyhow!("Invalid user status: {}", status_str)),
        };
        
        let roles_json: JsonValue = row.try_get("roles")?;
        let roles: Vec<String> = serde_json::from_value(roles_json)?;
        
        let metadata_json: JsonValue = row.try_get("metadata")?;
        let metadata: HashMap<String, String> = serde_json::from_value(metadata_json)?;
        
        Ok(User {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            password_hash: row.try_get("password_hash")?,
            roles,
            status,
            created_at: row.try_get("created_at")?,
            last_login: row.try_get("last_login")?,
            expires_at: row.try_get("expires_at")?,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rbac::UserStatus;
    
    #[tokio::test]
    #[ignore] // Requires PostgreSQL database setup
    async fn test_database_user_operations() {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://automation_user:test_password@localhost/automation_nation_test".to_string());
        
        let db = DatabaseManager::new(&database_url).await.unwrap();
        
        // Create test user
        let user = User {
            id: Uuid::new_v4(),
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            password_hash: "test_hash".to_string(),
            roles: vec!["developer".to_string()],
            status: UserStatus::Active,
            created_at: Utc::now(),
            last_login: None,
            expires_at: None,
            metadata: HashMap::new(),
        };
        
        // Test create and retrieve user
        db.create_user(&user).await.unwrap();
        let retrieved_user = db.get_user_by_username("test_user").await.unwrap().unwrap();
        
        assert_eq!(user.id, retrieved_user.id);
        assert_eq!(user.username, retrieved_user.username);
        assert_eq!(user.email, retrieved_user.email);
    }
}