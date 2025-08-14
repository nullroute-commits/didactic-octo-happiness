//! Database connection and migration management
//! 
//! This module provides database connection pooling, migration management,
//! and database operations for the Automation Nation platform.

use sqlx::{Postgres, PgPool, migrate::MigrateDatabase, Row};
use anyhow::{Result, anyhow};
use std::env;
use log::{info, warn, debug};

/// Database manager for PostgreSQL connections and migrations
pub struct DatabaseManager {
    pool: PgPool,
    database_url: String,
}

impl DatabaseManager {
    /// Create a new database manager with connection pool
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://automation_user:automation_password@localhost:5432/automation_nation".to_string());
        
        info!("Connecting to database: {}", Self::mask_password(&database_url));
        
        // Create database if it doesn't exist
        if !Postgres::database_exists(&database_url).await.unwrap_or(false) {
            info!("Database does not exist, creating...");
            Postgres::create_database(&database_url).await
                .map_err(|e| anyhow!("Failed to create database: {}", e))?;
            info!("Database created successfully");
        }
        
        // Create connection pool
        let pool = PgPool::connect(&database_url).await
            .map_err(|e| anyhow!("Failed to connect to database: {}", e))?;
        
        info!("Database connection pool established");
        
        let manager = Self {
            pool,
            database_url,
        };
        
        // Run migrations
        manager.run_migrations().await?;
        
        Ok(manager)
    }
    
    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");
        
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| anyhow!("Migration failed: {}", e))?;
        
        info!("Database migrations completed successfully");
        Ok(())
    }
    
    /// Check database connection health
    pub async fn health_check(&self) -> Result<()> {
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow!("Database health check failed: {}", e))?;
        
        let value: i32 = result.get(0);
        if value == 1 {
            debug!("Database health check passed");
            Ok(())
        } else {
            Err(anyhow!("Database health check returned unexpected value: {}", value))
        }
    }
    
    /// Get database connection statistics
    pub async fn get_connection_stats(&self) -> Result<DatabaseStats> {
        let size = self.pool.size();
        let idle = self.pool.num_idle();
        
        // Get additional stats from database
        let result = sqlx::query(
            "SELECT count(*) as active_connections FROM pg_stat_activity WHERE state = 'active'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow!("Failed to get connection stats: {}", e))?;
        
        let active_db_connections: i64 = result.get("active_connections");
        
        Ok(DatabaseStats {
            pool_size: size,
            idle_connections: idle as u32,
            active_connections: (size - idle as u32),
            database_active_connections: active_db_connections as u32,
        })
    }
    
    /// Clean up expired sessions and tokens
    pub async fn cleanup_expired_data(&self) -> Result<CleanupStats> {
        let mut tx = self.pool.begin().await
            .map_err(|e| anyhow!("Failed to start transaction: {}", e))?;
        
        // Clean up expired sessions
        let expired_sessions = sqlx::query(
            "DELETE FROM sessions WHERE expires_at < NOW() RETURNING id"
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to clean up sessions: {}", e))?;
        
        // Clean up expired API keys
        let expired_api_keys = sqlx::query(
            "DELETE FROM api_keys WHERE expires_at IS NOT NULL AND expires_at < NOW() RETURNING id"
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to clean up API keys: {}", e))?;
        
        // Clean up old audit log entries (older than 90 days)
        let old_audit_logs = sqlx::query(
            "DELETE FROM audit_log WHERE created_at < NOW() - INTERVAL '90 days' RETURNING id"
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| anyhow!("Failed to clean up audit logs: {}", e))?;
        
        tx.commit().await
            .map_err(|e| anyhow!("Failed to commit cleanup transaction: {}", e))?;
        
        let stats = CleanupStats {
            expired_sessions: expired_sessions.len(),
            expired_api_keys: expired_api_keys.len(),
            old_audit_logs: old_audit_logs.len(),
        };
        
        info!("Database cleanup completed: {:?}", stats);
        Ok(stats)
    }
    
    /// Backup database to SQL file
    pub async fn backup_database(&self, backup_path: &str) -> Result<()> {
        info!("Creating database backup to: {}", backup_path);
        
        // This would typically use pg_dump, but for now we'll create a simple approach
        // In a real implementation, you'd want to use external tools or a more sophisticated method
        warn!("Database backup feature needs implementation with pg_dump integration");
        
        Ok(())
    }
    
    /// Mask password in database URL for logging
    fn mask_password(url: &str) -> String {
        if let Some(at_pos) = url.find('@') {
            if let Some(colon_pos) = url[..at_pos].rfind(':') {
                let mut masked = url.to_string();
                masked.replace_range(colon_pos + 1..at_pos, "***");
                return masked;
            }
        }
        url.to_string()
    }
}

/// Database connection statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub pool_size: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub database_active_connections: u32,
}

/// Database cleanup statistics
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub expired_sessions: usize,
    pub expired_api_keys: usize,
    pub old_audit_logs: usize,
}

/// Database configuration for testing
pub struct TestDatabaseConfig {
    pub database_url: String,
    pub pool_size: u32,
}

impl TestDatabaseConfig {
    /// Create test database configuration
    pub fn new() -> Self {
        Self {
            database_url: "postgresql://test_user:test_password@localhost:5432/automation_nation_test".to_string(),
            pool_size: 5,
        }
    }
}

/// Create a test database manager for integration tests
#[cfg(test)]
pub async fn create_test_database() -> Result<DatabaseManager> {
    use uuid::Uuid;
    
    let test_db_name = format!("automation_nation_test_{}", Uuid::new_v4().to_string().replace('-', ""));
    let database_url = format!("postgresql://postgres:postgres@localhost:5432/{}", test_db_name);
    
    // Create test database
    if !Postgres::database_exists(&database_url).await.unwrap_or(false) {
        Postgres::create_database(&database_url).await
            .map_err(|e| anyhow!("Failed to create test database: {}", e))?;
    }
    
    env::set_var("DATABASE_URL", &database_url);
    DatabaseManager::new().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_mask_password() {
        let url = "postgresql://user:password@localhost:5432/dbname";
        let masked = DatabaseManager::mask_password(url);
        assert!(masked.contains("***"));
        assert!(!masked.contains("password"));
    }
    
    #[tokio::test]
    async fn test_database_config() {
        let config = TestDatabaseConfig::new();
        assert!(config.database_url.contains("automation_nation_test"));
        assert_eq!(config.pool_size, 5);
    }
}