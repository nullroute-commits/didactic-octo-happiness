//! Database integration tests for Automation Nation
//! 
//! This module provides comprehensive tests for the PostgreSQL database
//! persistence layer, including user management, authentication, and data integrity.

use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use sqlx::Row;
use anyhow;

use crate::{
    database::DatabaseManager,
    rbac::{User, UserStatus}
};

/// Database integration test suite
pub struct DatabaseTestSuite {
    database: Option<DatabaseManager>,
}

impl DatabaseTestSuite {
    /// Create a new database test suite
    /// 
    /// Note: This requires a test database to be available.
    /// Set TEST_DATABASE_URL environment variable to run these tests.
    pub async fn new() -> Self {
        let database = if let Ok(database_url) = std::env::var("TEST_DATABASE_URL") {
            match DatabaseManager::new(&database_url).await {
                Ok(db) => Some(db),
                Err(e) => {
                    eprintln!("Failed to connect to test database: {}", e);
                    None
                }
            }
        } else {
            eprintln!("TEST_DATABASE_URL not set, skipping database tests");
            None
        };
        
        Self { database }
    }
    
    /// Test basic database connectivity
    pub async fn test_database_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            // Try a simple query to test connectivity
            let result = sqlx::query("SELECT 1 as test")
                .fetch_one(db.pool())
                .await?;
            
            let test_value: i32 = result.try_get("test")?;
            assert_eq!(test_value, 1);
            
            println!("✓ Database connection test passed");
        } else {
            println!("⚠ Skipping database connection test (no test database configured)");
        }
        
        Ok(())
    }
    
    /// Test user CRUD operations
    pub async fn test_user_operations(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            // Create a test user
            let test_user = User {
                id: Uuid::new_v4(),
                username: format!("test_user_{}", Uuid::new_v4()),
                email: format!("test_{}@example.com", Uuid::new_v4()),
                display_name: "Test User".to_string(),
                password_hash: "test_hash".to_string(),
                roles: vec!["developer".to_string()],
                status: UserStatus::Active,
                created_at: Utc::now(),
                last_login: None,
                expires_at: None,
                metadata: [("test".to_string(), "value".to_string())].into(),
            };
            
            // Test create user
            db.create_user(&test_user).await?;
            println!("✓ User creation test passed");
            
            // Test get user by ID
            let retrieved_user = db.get_user_by_id(test_user.id).await?;
            assert!(retrieved_user.is_some());
            let retrieved_user = retrieved_user.unwrap();
            assert_eq!(retrieved_user.id, test_user.id);
            assert_eq!(retrieved_user.username, test_user.username);
            assert_eq!(retrieved_user.email, test_user.email);
            println!("✓ User retrieval by ID test passed");
            
            // Test get user by username
            let retrieved_user = db.get_user_by_username(&test_user.username).await?;
            assert!(retrieved_user.is_some());
            let retrieved_user = retrieved_user.unwrap();
            assert_eq!(retrieved_user.id, test_user.id);
            println!("✓ User retrieval by username test passed");
            
            // Test user not found
            let non_existent = db.get_user_by_username("nonexistent_user").await?;
            assert!(non_existent.is_none());
            println!("✓ User not found test passed");
            
            // Test password update
            let new_password_hash = "new_test_hash";
            db.update_user_password(test_user.id, new_password_hash).await?;
            
            let updated_user = db.get_user_by_id(test_user.id).await?.unwrap();
            assert_eq!(updated_user.password_hash, new_password_hash);
            println!("✓ Password update test passed");
            
            // Clean up test user
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(test_user.id)
                .execute(db.pool())
                .await?;
            
            println!("✓ All user operation tests passed");
        } else {
            println!("⚠ Skipping user operations test (no test database configured)");
        }
        
        Ok(())
    }
    
    /// Test password reset token operations
    pub async fn test_password_reset_tokens(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            // Create a test user first
            let test_user = User {
                id: Uuid::new_v4(),
                username: format!("reset_test_user_{}", Uuid::new_v4()),
                email: format!("reset_test_{}@example.com", Uuid::new_v4()),
                display_name: "Reset Test User".to_string(),
                password_hash: "test_hash".to_string(),
                roles: vec!["viewer".to_string()],
                status: UserStatus::Active,
                created_at: Utc::now(),
                last_login: None,
                expires_at: None,
                metadata: HashMap::new(),
            };
            
            db.create_user(&test_user).await?;
            
            // Test create password reset token
            let token = "test_reset_token_12345";
            let expires_at = Utc::now() + chrono::Duration::hours(24);
            let token_id = db.create_password_reset_token(test_user.id, token, expires_at).await?;
            
            assert!(token_id != Uuid::nil());
            println!("✓ Password reset token creation test passed");
            
            // Test validate password reset token
            let validated_user_id = db.validate_password_reset_token(token).await?;
            assert!(validated_user_id.is_some());
            assert_eq!(validated_user_id.unwrap(), test_user.id);
            println!("✓ Password reset token validation test passed");
            
            // Test token can only be used once
            let second_validation = db.validate_password_reset_token(token).await?;
            assert!(second_validation.is_none());
            println!("✓ Password reset token single-use test passed");
            
            // Clean up
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(test_user.id)
                .execute(db.pool())
                .await?;
            
            println!("✓ All password reset token tests passed");
        } else {
            println!("⚠ Skipping password reset token test (no test database configured)");
        }
        
        Ok(())
    }
    
    /// Test data integrity and constraints
    pub async fn test_data_integrity(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            // Test unique username constraint
            let user1 = User {
                id: Uuid::new_v4(),
                username: format!("duplicate_test_{}", Uuid::new_v4()),
                email: format!("user1_{}@example.com", Uuid::new_v4()),
                display_name: "User 1".to_string(),
                password_hash: "hash1".to_string(),
                roles: vec!["viewer".to_string()],
                status: UserStatus::Active,
                created_at: Utc::now(),
                last_login: None,
                expires_at: None,
                metadata: HashMap::new(),
            };
            
            let user2 = User {
                id: Uuid::new_v4(),
                username: user1.username.clone(), // Same username
                email: format!("user2_{}@example.com", Uuid::new_v4()),
                display_name: "User 2".to_string(),
                password_hash: "hash2".to_string(),
                roles: vec!["viewer".to_string()],
                status: UserStatus::Active,
                created_at: Utc::now(),
                last_login: None,
                expires_at: None,
                metadata: HashMap::new(),
            };
            
            // First user should succeed
            db.create_user(&user1).await?;
            
            // Second user with same username should fail
            let result = db.create_user(&user2).await;
            assert!(result.is_err());
            println!("✓ Unique username constraint test passed");
            
            // Clean up
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user1.id)
                .execute(db.pool())
                .await?;
            
            println!("✓ All data integrity tests passed");
        } else {
            println!("⚠ Skipping data integrity test (no test database configured)");
        }
        
        Ok(())
    }
    
    /// Test database performance
    pub async fn test_database_performance(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            let start = std::time::Instant::now();
            
            // Test query performance
            for _ in 0..10 {
                let _result = sqlx::query("SELECT COUNT(*) FROM users")
                    .fetch_one(db.pool())
                    .await?;
            }
            
            let duration = start.elapsed();
            assert!(duration.as_millis() < 1000, "Database queries took too long: {}ms", duration.as_millis());
            
            println!("✓ Database performance test passed ({:?})", duration);
        } else {
            println!("⚠ Skipping database performance test (no test database configured)");
        }
        
        Ok(())
    }
    
    /// Test concurrent database operations
    pub async fn test_concurrent_operations(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            let mut handles = Vec::new();
            
            // Create multiple concurrent operations
            for i in 0..5 {
                let db_clone = db.clone();
                let handle = tokio::spawn(async move {
                    let test_user = User {
                        id: Uuid::new_v4(),
                        username: format!("concurrent_user_{}_{}", i, Uuid::new_v4()),
                        email: format!("concurrent_{}@example.com", i),
                        display_name: format!("Concurrent User {}", i),
                        password_hash: format!("hash_{}", i),
                        roles: vec!["viewer".to_string()],
                        status: UserStatus::Active,
                        created_at: Utc::now(),
                        last_login: None,
                        expires_at: None,
                        metadata: HashMap::new(),
                    };
                    
                    // Create and then retrieve user
                    db_clone.create_user(&test_user).await?;
                    let retrieved = db_clone.get_user_by_id(test_user.id).await?;
                    
                    // Clean up
                    sqlx::query("DELETE FROM users WHERE id = $1")
                        .bind(test_user.id)
                        .execute(db_clone.pool())
                        .await?;
                    
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(retrieved.is_some())
                });
                handles.push(handle);
            }
            
            // Wait for all operations to complete
            for handle in handles {
                let result = handle.await.map_err(|e| anyhow::anyhow!("Task join error: {}", e))?;
                match result {
                    Ok(success) => assert!(success),
                    Err(e) => return Err(e),
                }
            }
            
            println!("✓ Concurrent operations test passed");
        } else {
            println!("⚠ Skipping concurrent operations test (no test database configured)");
        }
        
        Ok(())
    }
    
    /// Run all database tests
    pub async fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running database integration tests...");
        
        self.test_database_connection().await?;
        self.test_user_operations().await?;
        self.test_password_reset_tokens().await?;
        self.test_data_integrity().await?;
        self.test_database_performance().await?;
        self.test_concurrent_operations().await?;
        
        println!("✓ All database integration tests completed successfully!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    
    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_database_integration() {
        let test_suite = DatabaseTestSuite::new().await;
        test_suite.run_all_tests().await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_user_crud_operations() {
        let test_suite = DatabaseTestSuite::new().await;
        test_suite.test_user_operations().await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_password_reset_functionality() {
        let test_suite = DatabaseTestSuite::new().await;
        test_suite.test_password_reset_tokens().await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_database_constraints() {
        let test_suite = DatabaseTestSuite::new().await;
        test_suite.test_data_integrity().await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_performance_characteristics() {
        let test_suite = DatabaseTestSuite::new().await;
        test_suite.test_database_performance().await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // Requires test database
    async fn test_concurrency() {
        let test_suite = DatabaseTestSuite::new().await;
        test_suite.test_concurrent_operations().await.unwrap();
    }
}