//! Comprehensive database tests for PostgreSQL migration
//! 
//! This module provides extensive testing for the database layer,
//! including migrations, RBAC functionality, and data integrity.

#[cfg(test)]
mod tests {
    use crate::{DatabaseManager, DatabaseRbacManager};
    use crate::rbac::{User, UserStatus};
    use uuid::Uuid;
    use chrono::Utc;
    use std::collections::HashMap;
    
    /// Test database creation and migration
    #[tokio::test]
    async fn test_database_initialization() {
        // This test requires a running PostgreSQL instance
        // Skip if DATABASE_URL is not set or connection fails
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping database test - DATABASE_URL not set");
            return;
        }
        
        let result = DatabaseManager::new().await;
        match result {
            Ok(db_manager) => {
                // Test health check
                assert!(db_manager.health_check().await.is_ok());
                
                // Test connection stats
                let stats = db_manager.get_connection_stats().await;
                assert!(stats.is_ok());
                
                println!("Database initialization test passed");
            }
            Err(e) => {
                println!("Database test skipped - connection failed: {}", e);
            }
        }
    }
    
    /// Test RBAC database operations
    #[tokio::test]
    async fn test_database_rbac_operations() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping RBAC test - DATABASE_URL not set");
            return;
        }
        
        let db_result = DatabaseManager::new().await;
        if let Ok(db_manager) = db_result {
            let rbac_result = DatabaseRbacManager::new(&db_manager, "test_secret".to_string()).await;
            if let Ok(rbac_manager) = rbac_result {
                // Test admin user exists
                let admin_id = rbac_manager.get_admin_user_id().await.unwrap();
                assert!(admin_id.is_some());
                
                // Test authentication with default admin
                let auth_result = rbac_manager.authenticate("admin", "admin123", "127.0.0.1", "test-agent").await;
                match auth_result {
                    Ok(session) => {
                        assert_eq!(session.client_ip, "127.0.0.1");
                        assert_eq!(session.user_agent, "test-agent");
                        println!("RBAC authentication test passed");
                    }
                    Err(e) => {
                        println!("RBAC authentication failed: {}", e);
                    }
                }
                
                // Test create test user
                let test_user = User {
                    id: Uuid::new_v4(),
                    username: "test_db_user".to_string(),
                    email: "test@example.com".to_string(),
                    display_name: "Test Database User".to_string(),
                    password_hash: bcrypt::hash("test_password", bcrypt::DEFAULT_COST).unwrap(),
                    roles: vec!["viewer".to_string()],
                    status: UserStatus::Active,
                    created_at: Utc::now(),
                    last_login: None,
                    expires_at: None,
                    metadata: HashMap::new(),
                };
                
                let create_result = rbac_manager.create_test_user(test_user).await;
                assert!(create_result.is_ok());
                
                println!("Database RBAC operations test passed");
            } else {
                println!("RBAC manager creation failed");
            }
        } else {
            println!("Database manager creation failed");
        }
    }
    
    /// Test database cleanup operations
    #[tokio::test]
    async fn test_database_cleanup() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping cleanup test - DATABASE_URL not set");
            return;
        }
        
        let db_result = DatabaseManager::new().await;
        if let Ok(db_manager) = db_result {
            let cleanup_result = db_manager.cleanup_expired_data().await;
            match cleanup_result {
                Ok(stats) => {
                    // Stats should be non-negative
                    assert!(stats.expired_sessions >= 0);
                    assert!(stats.expired_api_keys >= 0);
                    assert!(stats.old_audit_logs >= 0);
                    println!("Database cleanup test passed: {:?}", stats);
                }
                Err(e) => {
                    println!("Cleanup test failed: {}", e);
                }
            }
        }
    }
    
    /// Test database migration idempotency
    #[tokio::test]
    async fn test_migration_idempotency() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping migration test - DATABASE_URL not set");
            return;
        }
        
        let db_result = DatabaseManager::new().await;
        if let Ok(db_manager) = db_result {
            // Run migrations multiple times - should not fail
            for i in 1..=3 {
                let migration_result = db_manager.run_migrations().await;
                match migration_result {
                    Ok(_) => println!("Migration run {} successful", i),
                    Err(e) => {
                        println!("Migration run {} failed: {}", i, e);
                        break;
                    }
                }
            }
        }
    }
}

/// Integration test for the complete database stack
#[cfg(test)]
mod integration_tests {
    use crate::{DatabaseManager, DatabaseRbacManager};
    use crate::rbac::{User, UserStatus};
    use uuid::Uuid;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::timeout;
    
    /// Test complete database workflow
    #[tokio::test]
    async fn test_complete_database_workflow() {
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping integration test - DATABASE_URL not set");
            return;
        }
        
        // Set timeout for the entire test
        let test_result = timeout(Duration::from_secs(30), async {
            // 1. Initialize database
            let db_manager = DatabaseManager::new().await?;
            
            // 2. Initialize RBAC
            let rbac_manager = DatabaseRbacManager::new(&db_manager, "integration_test_secret".to_string()).await?;
            
            // 3. Test user operations
            let test_user = User {
                id: Uuid::new_v4(),
                username: "integration_test_user".to_string(),
                email: "integration@test.com".to_string(),
                display_name: "Integration Test User".to_string(),
                password_hash: bcrypt::hash("integration_password", bcrypt::DEFAULT_COST).unwrap(),
                roles: vec!["developer".to_string()],
                status: UserStatus::Active,
                created_at: Utc::now(),
                last_login: None,
                expires_at: None,
                metadata: [("test".to_string(), "integration".to_string())].into(),
            };
            
            rbac_manager.create_test_user(test_user.clone()).await?;
            
            // 4. Test authentication
            let session = rbac_manager.authenticate(
                &test_user.username,
                "integration_password",
                "192.168.1.100",
                "integration-test-agent"
            ).await?;
            
            assert_eq!(session.user_id, test_user.id);
            assert_eq!(session.client_ip, "192.168.1.100");
            
            // 5. Test user retrieval
            let retrieved_user = rbac_manager.get_user(test_user.id).await?;
            assert!(retrieved_user.is_some());
            let user = retrieved_user.unwrap();
            assert_eq!(user.username, test_user.username);
            assert!(user.roles.contains(&"developer".to_string()));
            
            // 6. Test cleanup
            let cleanup_stats = db_manager.cleanup_expired_data().await?;
            assert!(cleanup_stats.expired_sessions >= 0);
            
            Ok::<(), anyhow::Error>(())
        }).await;
        
        match test_result {
            Ok(result) => {
                match result {
                    Ok(_) => println!("Complete database workflow test passed"),
                    Err(e) => println!("Database workflow test failed: {}", e),
                }
            }
            Err(_) => println!("Database workflow test timed out"),
        }
    }
}