//! Authentication web handlers for SSO and password reset
//! 
//! This module provides HTTP endpoints for authentication, SSO integration,
//! and password reset functionality.

use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    response::{Html, Json, Redirect},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use log::{info, warn, error, debug};
use crate::SsoManager;
// use crate::{PasswordResetManager, PasswordResetRequest, PasswordResetConfirmation}; // Temporarily disabled for build

/// Authentication application state
#[derive(Clone)]
pub struct AuthState {
    pub sso_manager: Arc<tokio::sync::RwLock<SsoManager>>,
    // pub password_reset_manager: Arc<tokio::sync::RwLock<PasswordResetManager>>, // Temporarily disabled for build
}

/// SSO login request
#[derive(Debug, Deserialize)]
pub struct SsoLoginRequest {
    pub provider: String,
    pub redirect_url: Option<String>,
}

/// OAuth callback parameters
#[derive(Debug, Deserialize)]
pub struct OauthCallback {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Authentication response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub session_token: Option<String>,
    pub redirect_url: Option<String>,
}

/// Create authentication router with all authentication routes
pub fn create_auth_router(state: AuthState) -> Router {
    Router::new()
        // SSO routes
        .route("/auth/sso/providers", get(list_sso_providers))
        .route("/auth/sso/login", post(initiate_sso_login))
        .route("/auth/sso/callback", get(handle_sso_callback))
        
        // Password reset routes (temporarily disabled for build)
        // .route("/auth/password-reset", post(initiate_password_reset))
        // .route("/auth/password-reset/confirm", post(confirm_password_reset))
        
        // Authentication status and logout
        .route("/auth/status", get(get_auth_status))
        .route("/auth/logout", post(logout))
        
        // Authentication pages (HTML)
        .route("/login", get(login_page))
        .route("/reset-password", get(reset_password_page))
        .route("/reset-password/confirm", get(reset_password_confirm_page))
        
        .with_state(state)
}

/// List available SSO providers
pub async fn list_sso_providers(
    State(state): State<AuthState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let sso_manager = state.sso_manager.read().await;
    let providers = sso_manager.get_providers();
    Ok(Json(providers))
}

/// Initiate SSO login
pub async fn initiate_sso_login(
    State(state): State<AuthState>,
    Json(request): Json<SsoLoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    debug!("SSO login requested for provider: {}", request.provider);
    
    let mut sso_manager = state.sso_manager.write().await;
    
    match sso_manager.generate_auth_url(&request.provider, request.redirect_url) {
        Ok(auth_url) => {
            info!("Generated SSO auth URL for provider: {}", request.provider);
            Ok(Json(AuthResponse {
                success: true,
                message: "SSO login URL generated".to_string(),
                session_token: None,
                redirect_url: Some(auth_url),
            }))
        }
        Err(e) => {
            error!("Failed to generate SSO auth URL: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Handle OAuth callback from SSO provider
pub async fn handle_sso_callback(
    State(state): State<AuthState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<OauthCallback>,
) -> Result<Redirect, StatusCode> {
    debug!("SSO callback received with state: {}", params.state);
    
    // Check for OAuth errors
    if let Some(error) = params.error {
        error!("OAuth error: {} - {}", error, params.error_description.unwrap_or_default());
        return Ok(Redirect::to("/login?error=oauth_error"));
    }
    
    let mut sso_manager = state.sso_manager.write().await;
    let client_ip = addr.ip().to_string();
    
    match sso_manager.handle_callback(&params.code, &params.state, &client_ip).await {
        Ok(session) => {
            info!("SSO authentication successful for user: {}", session.user_id);
            // In a real implementation, you would set a secure cookie or JWT token here
            Ok(Redirect::to(&format!("/dashboard?token={}", session.token)))
        }
        Err(e) => {
            error!("SSO callback failed: {}", e);
            Ok(Redirect::to("/login?error=sso_failed"))
        }
    }
}

/*
/// Initiate password reset (temporarily disabled)
pub async fn initiate_password_reset(
    State(state): State<AuthState>,
    Json(request): Json<PasswordResetRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("Password reset requested for email: {}", request.email);
    
    let mut password_reset_manager = state.password_reset_manager.write().await;
    
    match password_reset_manager.initiate_password_reset(request).await {
        Ok(_) => {
            info!("Password reset initiated successfully");
            Ok(Json(AuthResponse {
                success: true,
                message: "If the email address is registered, you will receive a password reset link.".to_string(),
                session_token: None,
                redirect_url: None,
            }))
        }
        Err(e) => {
            warn!("Password reset initiation failed: {}", e);
            // Don't reveal specific error details for security
            Ok(Json(AuthResponse {
                success: false,
                message: "Password reset request failed. Please try again later.".to_string(),
                session_token: None,
                redirect_url: None,
            }))
        }
    }
}

/// Confirm password reset with token (temporarily disabled)
pub async fn confirm_password_reset(
    State(state): State<AuthState>,
    Json(confirmation): Json<PasswordResetConfirmation>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("Password reset confirmation attempted");
    
    let mut password_reset_manager = state.password_reset_manager.write().await;
    
    match password_reset_manager.confirm_password_reset(confirmation).await {
        Ok(_) => {
            info!("Password reset completed successfully");
            Ok(Json(AuthResponse {
                success: true,
                message: "Password has been reset successfully. You can now log in with your new password.".to_string(),
                session_token: None,
                redirect_url: Some("/login".to_string()),
            }))
        }
        Err(e) => {
            warn!("Password reset confirmation failed: {}", e);
            Ok(Json(AuthResponse {
                success: false,
                message: format!("Password reset failed: {}", e),
                session_token: None,
                redirect_url: None,
            }))
        }
    }
}
*/

/// Get authentication status
pub async fn get_auth_status() -> Result<Json<AuthResponse>, StatusCode> {
    // This would check the current session/token
    // For now, return a placeholder response
    Ok(Json(AuthResponse {
        success: false,
        message: "Not authenticated".to_string(),
        session_token: None,
        redirect_url: None,
    }))
}

/// Logout user
pub async fn logout() -> Result<Json<AuthResponse>, StatusCode> {
    // This would invalidate the current session
    // For now, return a success response
    Ok(Json(AuthResponse {
        success: true,
        message: "Logged out successfully".to_string(),
        session_token: None,
        redirect_url: Some("/login".to_string()),
    }))
}

/// Login page HTML
pub async fn login_page(
    State(state): State<AuthState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    let sso_manager = state.sso_manager.read().await;
    let providers = sso_manager.get_providers();
    
    let error_message = params.get("error").map(|e| match e.as_str() {
        "oauth_error" => "OAuth authentication failed. Please try again.",
        "sso_failed" => "SSO authentication failed. Please try again.",
        _ => "An error occurred during authentication.",
    });
    
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Login - Automation Nation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 400px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ text-align: center; color: #333; }}
        .form-group {{ margin-bottom: 20px; }}
        label {{ display: block; margin-bottom: 5px; color: #555; }}
        input {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; }}
        button {{ width: 100%; padding: 12px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }}
        button:hover {{ background-color: #0056b3; }}
        .sso-section {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #eee; }}
        .sso-button {{ margin-bottom: 10px; background-color: #28a745; }}
        .sso-button:hover {{ background-color: #218838; }}
        .error {{ color: #dc3545; text-align: center; margin-bottom: 20px; }}
        .forgot-password {{ text-align: center; margin-top: 15px; }}
        .forgot-password a {{ color: #007bff; text-decoration: none; }}
        .forgot-password a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Automation Nation</h1>
        <h2>Login</h2>
        
        {}
        
        <form id="loginForm">
            <div class="form-group">
                <label for="username">Username or Email:</label>
                <input type="text" id="username" name="username" required>
            </div>
            <div class="form-group">
                <label for="password">Password:</label>
                <input type="password" id="password" name="password" required>
            </div>
            <button type="submit">Login</button>
        </form>
        
        <div class="forgot-password">
            <a href="/reset-password">Forgot your password?</a>
        </div>
        
        {}
    </div>
    
    <script>
        document.getElementById('loginForm').addEventListener('submit', function(e) {{
            e.preventDefault();
            // Add login logic here
            alert('Login functionality to be implemented');
        }});
        
        function ssoLogin(provider) {{
            fetch('/auth/sso/login', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{ provider: provider }})
            }})
            .then(response => response.json())
            .then(data => {{
                if (data.success && data.redirect_url) {{
                    window.location.href = data.redirect_url;
                }} else {{
                    alert('SSO login failed: ' + data.message);
                }}
            }})
            .catch(error => {{
                console.error('Error:', error);
                alert('SSO login failed');
            }});
        }}
    </script>
</body>
</html>
"#,
        error_message.map(|msg| format!("<div class=\"error\">{}</div>", msg)).unwrap_or_default(),
        if providers.is_empty() {
            String::new()
        } else {
            format!(
                r#"
        <div class="sso-section">
            <h3>Or sign in with:</h3>
            {}
        </div>
"#,
                providers.iter()
                    .map(|provider| format!(
                        "<button type=\"button\" class=\"sso-button\" onclick=\"ssoLogin('{}')\">{}</button>",
                        provider, provider
                    ))
                    .collect::<Vec<_>>()
                    .join("\n            ")
            )
        }
    );
    
    Ok(Html(html))
}

/// Password reset page HTML
pub async fn reset_password_page(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    let message = params.get("message").map(|m| match m.as_str() {
        "sent" => "If the email address is registered, you will receive a password reset link.",
        "error" => "An error occurred. Please try again.",
        _ => "",
    });
    
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Password - Automation Nation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 400px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ text-align: center; color: #333; }}
        .form-group {{ margin-bottom: 20px; }}
        label {{ display: block; margin-bottom: 5px; color: #555; }}
        input {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; }}
        button {{ width: 100%; padding: 12px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }}
        button:hover {{ background-color: #0056b3; }}
        .message {{ text-align: center; margin-bottom: 20px; padding: 10px; border-radius: 4px; }}
        .success {{ background-color: #d4edda; color: #155724; border: 1px solid #c3e6cb; }}
        .error {{ background-color: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; }}
        .back-link {{ text-align: center; margin-top: 15px; }}
        .back-link a {{ color: #007bff; text-decoration: none; }}
        .back-link a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Reset Password</h1>
        
        {}
        
        <form id="resetForm">
            <div class="form-group">
                <label for="email">Email Address:</label>
                <input type="email" id="email" name="email" required>
            </div>
            <button type="submit">Send Reset Link</button>
        </form>
        
        <div class="back-link">
            <a href="/login">Back to Login</a>
        </div>
    </div>
    
    <script>
        document.getElementById('resetForm').addEventListener('submit', function(e) {{
            e.preventDefault();
            
            const email = document.getElementById('email').value;
            
            fetch('/auth/password-reset', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{ 
                    email: email,
                    client_ip: '127.0.0.1',
                    user_agent: navigator.userAgent
                }})
            }})
            .then(response => response.json())
            .then(data => {{
                if (data.success) {{
                    window.location.href = '/reset-password?message=sent';
                }} else {{
                    window.location.href = '/reset-password?message=error';
                }}
            }})
            .catch(error => {{
                console.error('Error:', error);
                window.location.href = '/reset-password?message=error';
            }});
        }});
    </script>
</body>
</html>
"#,
        message.map(|msg| {
            let class = if msg.contains("sent") { "success" } else { "error" };
            format!("<div class=\"message {}\">{}</div>", class, msg)
        }).unwrap_or_default()
    );
    
    Ok(Html(html))
}

/// Password reset confirmation page HTML
pub async fn reset_password_confirm_page(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    let token = params.get("token").map_or("", |v| v);
    
    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Confirm Password Reset - Automation Nation</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 400px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ text-align: center; color: #333; }}
        .form-group {{ margin-bottom: 20px; }}
        label {{ display: block; margin-bottom: 5px; color: #555; }}
        input {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box; }}
        button {{ width: 100%; padding: 12px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }}
        button:hover {{ background-color: #0056b3; }}
        .password-requirements {{ margin-top: 10px; font-size: 12px; color: #666; }}
        .password-requirements ul {{ margin: 5px 0; padding-left: 20px; }}
        .error {{ color: #dc3545; text-align: center; margin-bottom: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Set New Password</h1>
        
        <form id="confirmForm">
            <input type="hidden" id="token" value="{}">
            
            <div class="form-group">
                <label for="newPassword">New Password:</label>
                <input type="password" id="newPassword" name="newPassword" required>
                <div class="password-requirements">
                    Password must contain:
                    <ul>
                        <li>At least 8 characters</li>
                        <li>One lowercase letter</li>
                        <li>One uppercase letter</li>
                        <li>One digit</li>
                        <li>One special character (!@#$%^&*()_+-=[]{{}}|;:,.<>?)</li>
                    </ul>
                </div>
            </div>
            
            <div class="form-group">
                <label for="confirmPassword">Confirm Password:</label>
                <input type="password" id="confirmPassword" name="confirmPassword" required>
            </div>
            
            <button type="submit">Reset Password</button>
        </form>
        
        <div id="errorMessage" class="error" style="display: none;"></div>
    </div>
    
    <script>
        document.getElementById('confirmForm').addEventListener('submit', function(e) {{
            e.preventDefault();
            
            const newPassword = document.getElementById('newPassword').value;
            const confirmPassword = document.getElementById('confirmPassword').value;
            const token = document.getElementById('token').value;
            const errorDiv = document.getElementById('errorMessage');
            
            // Clear previous errors
            errorDiv.style.display = 'none';
            
            // Validate passwords match
            if (newPassword !== confirmPassword) {{
                errorDiv.textContent = 'Passwords do not match';
                errorDiv.style.display = 'block';
                return;
            }}
            
            fetch('/auth/password-reset/confirm', {{
                method: 'POST',
                headers: {{
                    'Content-Type': 'application/json',
                }},
                body: JSON.stringify({{ 
                    token: token,
                    new_password: newPassword,
                    client_ip: '127.0.0.1',
                    user_agent: navigator.userAgent
                }})
            }})
            .then(response => response.json())
            .then(data => {{
                if (data.success) {{
                    alert('Password reset successfully! You can now log in with your new password.');
                    window.location.href = '/login';
                }} else {{
                    errorDiv.textContent = data.message;
                    errorDiv.style.display = 'block';
                }}
            }})
            .catch(error => {{
                console.error('Error:', error);
                errorDiv.textContent = 'An error occurred. Please try again.';
                errorDiv.style.display = 'block';
            }});
        }});
    </script>
</body>
</html>
"#,
        token
    );
    
    Ok(Html(html))
}

/* Password reset handlers temporarily disabled for build
/// Initiate password reset
pub async fn initiate_password_reset(
    State(state): State<AuthState>,
    Json(request): Json<PasswordResetRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    debug!("Password reset requested for email: {}", request.email);
    
    let mut password_reset_manager = state.password_reset_manager.write().await;
    
    match password_reset_manager.initiate_password_reset(request).await {
        Ok(_) => {
            info!("Password reset initiated for email: {}", request.email);
            Ok(Json(AuthResponse {
                success: true,
                message: "If the email address is registered, you will receive a password reset link.".to_string(),
                session_token: None,
                redirect_url: None,
            }))
        },
        Err(e) => {
            warn!("Password reset failed: {}", e);
            Ok(Json(AuthResponse {
                success: false,
                message: "Unable to process password reset request.".to_string(),
                session_token: None,
                redirect_url: None,
            }))
        }
    }
}

/// Confirm password reset
pub async fn confirm_password_reset(
    State(state): State<AuthState>,
    Json(confirmation): Json<PasswordResetConfirmation>,
) -> Result<Json<AuthResponse>, StatusCode> {
    debug!("Password reset confirmation for token: {}", confirmation.token);
    
    let mut password_reset_manager = state.password_reset_manager.write().await;
    
    match password_reset_manager.confirm_password_reset(confirmation).await {
        Ok(_) => {
            info!("Password reset completed successfully");
            Ok(Json(AuthResponse {
                success: true,
                message: "Password reset successfully!".to_string(),
                session_token: None,
                redirect_url: Some("/login".to_string()),
            }))
        },
        Err(e) => {
            warn!("Password reset confirmation failed: {}", e);
            Ok(Json(AuthResponse {
                success: false,
                message: "Invalid or expired reset token.".to_string(),
                session_token: None,
                redirect_url: None,
            }))
        }
    }
}
*/