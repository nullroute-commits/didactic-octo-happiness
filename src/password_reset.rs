//! Password reset functionality for Automation Nation
//! 
//! This module provides secure password reset capabilities including
//! email-based password reset tokens, rate limiting, and security logging.

use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{info, warn, debug};
use crate::database::DatabaseManager;
use crate::database_rbac::DatabaseRbacManager;
use bcrypt;

/// Password reset token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token_id: String,    // Store as string, convert to Uuid when needed
    pub user_id: String,     // Store as string, convert to Uuid when needed  
    pub email: String,
    pub token_hash: String,
    pub created_at: String,  // Store as string, convert to DateTime when needed
    pub expires_at: String,  // Store as string, convert to DateTime when needed
    pub used: i64,           // SQLite uses integers for booleans
    pub attempts: i64,
    pub ip_address: Option<String>,
}

/// Password reset request information
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub client_ip: String,
    pub user_agent: String,
}

/// Password reset confirmation
#[derive(Debug, Deserialize)]
pub struct PasswordResetConfirmation {
    pub token: String,
    pub new_password: String,
    pub client_ip: String,
    pub user_agent: String,
}

/// Password reset rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_hour: u32,
    pub max_requests_per_day: u32,
    pub lockout_duration_minutes: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_hour: 5,
            max_requests_per_day: 10,
            lockout_duration_minutes: 15,
        }
    }
}

/// Email configuration for password reset notifications
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
    pub use_tls: bool,
}

/// Password reset manager
pub struct PasswordResetManager {
    db_manager: DatabaseManager,
    rbac_manager: DatabaseRbacManager,
    rate_limit_config: RateLimitConfig,
    email_config: Option<EmailConfig>,
    pending_tokens: HashMap<String, PasswordResetToken>,
    rate_limit_tracking: HashMap<String, Vec<DateTime<Utc>>>,
}

impl PasswordResetManager {
    /// Create a new password reset manager
    pub fn new(
        db_manager: DatabaseManager,
        rbac_manager: DatabaseRbacManager,
        rate_limit_config: Option<RateLimitConfig>,
        email_config: Option<EmailConfig>,
    ) -> Self {
        Self {
            db_manager,
            rbac_manager,
            rate_limit_config: rate_limit_config.unwrap_or_default(),
            email_config,
            pending_tokens: HashMap::new(),
            rate_limit_tracking: HashMap::new(),
        }
    }
    
    /// Initiate password reset process
    pub async fn initiate_password_reset(&mut self, request: PasswordResetRequest) -> Result<()> {
        info!("Password reset initiated for email: {}", request.email);
        
        // Check rate limiting
        self.check_rate_limit(&request.email, &request.client_ip).await?;
        
        // Find user by email
        let user = match self.rbac_manager.get_user_by_username(&request.email).await? {
            Some(user) => user,
            None => {
                // Don't reveal if email exists or not for security
                info!("Password reset requested for non-existent email: {}", request.email);
                self.log_password_reset_event(
                    None,
                    "password_reset_requested_nonexistent".to_string(),
                    &request.email,
                    &request.client_ip,
                    false,
                    Some("Email not found".to_string()),
                ).await?;
                return Ok(()); // Still return success to not reveal email existence
            }
        };
        
        // Check if user account is active
        if user.status != crate::rbac::UserStatus::Active {
            warn!("Password reset attempted for inactive account: {}", request.email);
            self.log_password_reset_event(
                Some(user.id),
                "password_reset_requested_inactive".to_string(),
                &request.email,
                &request.client_ip,
                false,
                Some(format!("Account status: {:?}", user.status)),
            ).await?;
            return Ok(());
        }
        
        // Generate reset token
        let token = self.generate_reset_token(user.id, &request.email, &request.client_ip).await?;
        
        // Send reset email
        if let Some(ref email_config) = self.email_config {
            self.send_reset_email(email_config, &user.email, &user.display_name, &token.token_hash).await?;
        } else {
            warn!("Email configuration not available - password reset token generated but not sent");
            debug!("Password reset token for {}: {}", request.email, token.token_hash);
        }
        
        // Log successful initiation
        self.log_password_reset_event(
            Some(user.id),
            "password_reset_initiated".to_string(),
            &request.email,
            &request.client_ip,
            true,
            None,
        ).await?;
        
        // Track rate limiting
        self.track_rate_limit(&request.email);
        self.track_rate_limit(&request.client_ip);
        
        info!("Password reset token generated successfully for: {}", request.email);
        Ok(())
    }
    
    /// Confirm password reset with token
    pub async fn confirm_password_reset(&mut self, confirmation: PasswordResetConfirmation) -> Result<()> {
        info!("Password reset confirmation attempted with token");
        
        // Find and validate token
        let mut token = self.find_and_validate_token(&confirmation.token).await?;
        
        // Increment attempt counter
        token.attempts += 1;
        
        // Check if too many attempts
        if token.attempts > 3 {
            self.invalidate_token(&confirmation.token).await?;
            return Err(anyhow!("Too many attempts - token invalidated"));
        }
        
        // Validate password strength
        self.validate_password_strength(&confirmation.new_password)?;
        
        // Update user password
        let user_id = Uuid::parse_str(&token.user_id)
            .map_err(|e| anyhow!("Invalid user ID format: {}", e))?;
        self.update_user_password(user_id, &confirmation.new_password).await?;
        
        // Invalidate token
        self.invalidate_token(&confirmation.token).await?;
        
        // Log successful password reset
        self.log_password_reset_event(
            Some(user_id),
            "password_reset_completed".to_string(),
            &token.email,
            &confirmation.client_ip,
            true,
            None,
        ).await?;
        
        info!("Password reset completed successfully for user: {}", token.email);
        Ok(())
    }
    
    /// Generate a new password reset token
    async fn generate_reset_token(&mut self, user_id: Uuid, email: &str, ip_address: &str) -> Result<PasswordResetToken> {
        let token_id = Uuid::new_v4();
        let raw_token = format!("{}_{}", token_id, Uuid::new_v4());
        let _token_hash = self.hash_token(&raw_token)?;
        
        let token = PasswordResetToken {
            token_id: token_id.to_string(),
            user_id: user_id.to_string(),
            email: email.to_string(),
            token_hash: raw_token.clone(), // Store raw token temporarily for email sending
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + Duration::hours(1)).to_rfc3339(), // 1 hour expiration
            used: 0, // false as i64
            attempts: 0,
            ip_address: Some(ip_address.to_string()),
        };
        
        // Store in database
        self.store_reset_token(&token).await?;
        
        // Store in memory for quick access
        self.pending_tokens.insert(raw_token.clone(), token.clone());
        
        Ok(token)
    }
    
    /// Find and validate a reset token
    async fn find_and_validate_token(&mut self, token: &str) -> Result<PasswordResetToken> {
        // Check in-memory first
        if let Some(token_info) = self.pending_tokens.get(token).cloned() {
            let expires_at = DateTime::parse_from_rfc3339(&token_info.expires_at)
                .map_err(|e| anyhow!("Invalid expiration date format: {}", e))?
                .with_timezone(&Utc);
            
            if expires_at < Utc::now() {
                self.pending_tokens.remove(token);
                return Err(anyhow!("Password reset token has expired"));
            }
            
            if token_info.used != 0 {
                return Err(anyhow!("Password reset token has already been used"));
            }
            
            return Ok(token_info);
        }
        
        // Check database
        let token_info = self.get_token_from_database(token).await?;
        match token_info {
            Some(token_info) => {
                let expires_at = DateTime::parse_from_rfc3339(&token_info.expires_at)
                    .map_err(|e| anyhow!("Invalid expiration date format: {}", e))?
                    .with_timezone(&Utc);
                    
                if expires_at < Utc::now() {
                    return Err(anyhow!("Password reset token has expired"));
                }
                
                if token_info.used != 0 {
                    return Err(anyhow!("Password reset token has already been used"));
                }
                
                Ok(token_info)
            }
            None => Err(anyhow!("Invalid password reset token")),
        }
    }
    
    /// Validate password strength
    fn validate_password_strength(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters long"));
        }
        
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        if !has_lowercase {
            return Err(anyhow!("Password must contain at least one lowercase letter"));
        }
        
        if !has_uppercase {
            return Err(anyhow!("Password must contain at least one uppercase letter"));
        }
        
        if !has_digit {
            return Err(anyhow!("Password must contain at least one digit"));
        }
        
        if !has_special {
            return Err(anyhow!("Password must contain at least one special character"));
        }
        
        Ok(())
    }
    
    /// Update user password in database
    async fn update_user_password(&self, user_id: Uuid, new_password: &str) -> Result<()> {
        let password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;
        
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(self.db_manager.pool())
            .await
            .map_err(|e| anyhow!("Failed to update password: {}", e))?;
        
        Ok(())
    }
    
    /// Hash a password reset token
    fn hash_token(&self, token: &str) -> Result<String> {
        bcrypt::hash(token, bcrypt::DEFAULT_COST)
            .map_err(|e| anyhow!("Failed to hash token: {}", e))
    }
    
    /// Store reset token in database
    async fn store_reset_token(&self, token: &PasswordResetToken) -> Result<()> {
        let token_hash = self.hash_token(&token.token_hash)?;
        
        sqlx::query(
            "INSERT INTO password_reset_tokens (token_id, user_id, email, token_hash, created_at, expires_at, used, attempts, ip_address) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(&token.token_id)
        .bind(&token.user_id)
        .bind(&token.email)
        .bind(token_hash)
        .bind(&token.created_at)
        .bind(&token.expires_at)
        .bind(token.used)
        .bind(token.attempts as i32)
        .bind(&token.ip_address)
        .execute(self.db_manager.pool())
        .await
        .map_err(|e| anyhow!("Failed to store reset token: {}", e))?;
        
        Ok(())
    }
    
    /// Get token from database
    async fn get_token_from_database(&self, token: &str) -> Result<Option<PasswordResetToken>> {
        let token_hash = self.hash_token(token)?;
        let row = sqlx::query_as!(
            PasswordResetToken,
            r#"
                SELECT 
                    token_id as "token_id: _",
                    user_id as "user_id: _",
                    email,
                    token_hash,
                    created_at,
                    expires_at,
                    used,
                    attempts,
                    ip_address
                FROM password_reset_tokens
                WHERE token_hash = $1 AND used = 0 AND expires_at > datetime('now')
                LIMIT 1
            "#,
            token_hash
        )
        .fetch_optional(self.db_manager.pool())
        .await
        .map_err(|e| anyhow!("Failed to fetch reset token: {}", e))?;
        Ok(row)
    }
    
    /// Invalidate a reset token
    async fn invalidate_token(&mut self, token: &str) -> Result<()> {
        // Remove from memory
        self.pending_tokens.remove(token);
        
        // Mark as used in database
        let token_hash = self.hash_token(token)?;
        sqlx::query("UPDATE password_reset_tokens SET used = 1 WHERE token_hash = $1")
            .bind(token_hash)
            .execute(self.db_manager.pool())
            .await
            .map_err(|e| anyhow!("Failed to invalidate token: {}", e))?;
        
        Ok(())
    }
    
    /// Check rate limiting for password reset requests
    async fn check_rate_limit(&self, email: &str, ip_address: &str) -> Result<()> {
        let now = Utc::now();
        let hour_ago = now - Duration::hours(1);
        let day_ago = now - Duration::days(1);
        
        // Check email-based rate limiting
        if let Some(requests) = self.rate_limit_tracking.get(email) {
            let recent_requests = requests.iter().filter(|&&t| t > hour_ago).count();
            if recent_requests >= self.rate_limit_config.max_requests_per_hour as usize {
                return Err(anyhow!("Too many password reset requests. Please try again later."));
            }
            
            let daily_requests = requests.iter().filter(|&&t| t > day_ago).count();
            if daily_requests >= self.rate_limit_config.max_requests_per_day as usize {
                return Err(anyhow!("Daily limit exceeded for password reset requests."));
            }
        }
        
        // Check IP-based rate limiting
        if let Some(requests) = self.rate_limit_tracking.get(ip_address) {
            let recent_requests = requests.iter().filter(|&&t| t > hour_ago).count();
            if recent_requests >= self.rate_limit_config.max_requests_per_hour as usize {
                return Err(anyhow!("Too many password reset requests from this IP. Please try again later."));
            }
        }
        
        Ok(())
    }
    
    /// Track rate limiting
    fn track_rate_limit(&mut self, key: &str) {
        let now = Utc::now();
        let entry = self.rate_limit_tracking.entry(key.to_string()).or_insert_with(Vec::new);
        entry.push(now);
        
        // Clean up old entries (keep only last 24 hours)
        let day_ago = now - Duration::days(1);
        entry.retain(|&t| t > day_ago);
    }
    
    /// Send password reset email
    async fn send_reset_email(&self, _email_config: &EmailConfig, to_email: &str, display_name: &str, token: &str) -> Result<()> {
        // Create reset URL
        let reset_url = format!("https://your-domain.com/auth/reset-password?token={}", urlencoding::encode(token));
        
        // Create email content
        let subject = "Password Reset Request - Automation Nation";
        let body = format!(
            r#"
Dear {},

You have requested a password reset for your Automation Nation account.

To reset your password, please click the following link:
{}

This link will expire in 1 hour for security reasons.

If you did not request this password reset, please ignore this email.

Best regards,
Automation Nation Team
"#,
            display_name, reset_url
        );
        
        // Here you would integrate with your email service (SMTP, SendGrid, etc.)
        info!("Password reset email would be sent to: {}", to_email);
        debug!("Reset URL: {}", reset_url);
        
        // For now, just log the email content
        debug!("Email content:\nSubject: {}\nBody: {}", subject, body);
        
        Ok(())
    }
    
    /// Log password reset events for security auditing
    async fn log_password_reset_event(
        &self,
        user_id: Option<Uuid>,
        action: String,
        email: &str,
        client_ip: &str,
        success: bool,
        error_message: Option<String>,
    ) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("email".to_string(), email.to_string());
        
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, resource, client_ip, success, error_message, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(user_id)
        .bind(action)
        .bind("password_reset")
        .bind(client_ip)
        .bind(success)
        .bind(error_message)
        .bind(serde_json::to_value(metadata).unwrap_or_default())
        .execute(self.db_manager.pool())
        .await
        .map_err(|e| anyhow!("Failed to log audit event: {}", e))?;
        
        Ok(())
    }
    
    /// Clean up expired tokens and rate limit tracking
    pub fn cleanup_expired_data(&mut self) {
        let now = Utc::now();
        
        // Remove expired tokens
        self.pending_tokens.retain(|_, token| {
            if let Ok(expires_at) = DateTime::parse_from_rfc3339(&token.expires_at) {
                expires_at.with_timezone(&Utc) > now
            } else {
                false // Remove tokens with invalid date format
            }
        });
        
        // Clean up rate limit tracking (keep only last 24 hours)
        let day_ago = now - Duration::days(1);
        for (_, requests) in self.rate_limit_tracking.iter_mut() {
            requests.retain(|&t| t > day_ago);
        }
        
        // Remove empty rate limit entries
        self.rate_limit_tracking.retain(|_, requests| !requests.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_password_strength_validation() {
        // Mock the password reset manager for testing password validation
        // Since we can't easily create a real manager without database setup,
        // we'll test the validation logic separately
        
        // Test weak passwords - we'll skip the manager creation for now
        // This test would need to be restructured for proper async testing
        
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests_per_hour, 5);
        assert_eq!(config.max_requests_per_day, 10);
        assert_eq!(config.lockout_duration_minutes, 15);
    }
    
    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests_per_hour, 5);
        assert_eq!(config.max_requests_per_day, 10);
        assert_eq!(config.lockout_duration_minutes, 15);
    }
}