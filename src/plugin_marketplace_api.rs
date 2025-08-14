//! Plugin Marketplace REST API
//!
//! This module provides HTTP API endpoints for the plugin marketplace system,
//! including plugin discovery, installation, management, and marketplace operations.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    plugin_marketplace::{
        PluginMarketplace, PluginSearchCriteria, PluginInstallRequest,
        PluginInfo, InstalledPlugin, PluginSearchResults, PluginStatus,
        MarketplaceConfig,
    },
    web_types::{ApiError, ApiResponse},
};

/// Plugin marketplace application state
#[derive(Clone)]
pub struct PluginMarketplaceAppState {
    pub marketplace: Arc<RwLock<PluginMarketplace>>,
}

/// Create plugin marketplace routes
pub fn create_marketplace_routes() -> Router<PluginMarketplaceAppState> {
    Router::new()
        // Marketplace management
        .route("/api/plugins/marketplace/refresh", post(refresh_marketplace))
        .route("/api/plugins/marketplace/config", get(get_marketplace_config))
        .route("/api/plugins/marketplace/config", put(update_marketplace_config))
        
        // Plugin discovery and search
        .route("/api/plugins/search", get(search_plugins))
        .route("/api/plugins/available", get(list_available_plugins))
        .route("/api/plugins/available/:plugin_id", get(get_plugin_details))
        
        // Plugin installation and management
        .route("/api/plugins/installed", get(list_installed_plugins))
        .route("/api/plugins/installed/:plugin_id", get(get_installed_plugin))
        .route("/api/plugins/install", post(install_plugin))
        .route("/api/plugins/uninstall/:plugin_id", delete(uninstall_plugin))
        .route("/api/plugins/update/:plugin_id", post(update_plugin))
        
        // Plugin control
        .route("/api/plugins/:plugin_id/enable", post(enable_plugin))
        .route("/api/plugins/:plugin_id/disable", post(disable_plugin))
        .route("/api/plugins/:plugin_id/status", get(get_plugin_status))
        .route("/api/plugins/:plugin_id/config", get(get_plugin_config))
        .route("/api/plugins/:plugin_id/config", put(update_plugin_config))
        
        // Plugin execution and monitoring
        .route("/api/plugins/:plugin_id/execute", post(execute_plugin))
        .route("/api/plugins/:plugin_id/logs", get(get_plugin_logs))
        .route("/api/plugins/:plugin_id/stats", get(get_plugin_stats))
        
        // Marketplace statistics and health
        .route("/api/plugins/stats", get(get_marketplace_stats))
        .route("/api/plugins/health", get(get_marketplace_health))
}

/// Refresh the plugin marketplace
pub async fn refresh_marketplace(
    State(state): State<PluginMarketplaceAppState>,
) -> Result<Json<ApiResponse<MarketplaceRefreshResponse>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    
    let plugin_count = marketplace.refresh_marketplace().await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to refresh marketplace: {}", e)))?;
    
    let response = MarketplaceRefreshResponse {
        plugins_found: plugin_count,
        refresh_time: chrono::Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct MarketplaceRefreshResponse {
    pub plugins_found: u32,
    pub refresh_time: chrono::DateTime<chrono::Utc>,
}

/// Get marketplace configuration
pub async fn get_marketplace_config(
    State(state): State<PluginMarketplaceAppState>,
) -> Result<Json<ApiResponse<MarketplaceConfig>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    let config = marketplace.config.clone();
    Ok(Json(ApiResponse::success(config)))
}

/// Update marketplace configuration
#[derive(Debug, Deserialize)]
pub struct UpdateMarketplaceConfigRequest {
    pub config: MarketplaceConfig,
}

pub async fn update_marketplace_config(
    State(state): State<PluginMarketplaceAppState>,
    Json(request): Json<UpdateMarketplaceConfigRequest>,
) -> Result<Json<ApiResponse<MarketplaceConfig>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    marketplace.config = request.config.clone();
    Ok(Json(ApiResponse::success(request.config)))
}

/// Search for plugins
pub async fn search_plugins(
    State(state): State<PluginMarketplaceAppState>,
    Query(query): Query<PluginSearchQuery>,
) -> Result<Json<ApiResponse<PluginSearchResults>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let criteria = PluginSearchCriteria {
        query: query.query,
        category: query.category,
        tags: query.tags.unwrap_or_default(),
        author: query.author,
        min_rating: query.min_rating,
        sort_by: query.sort_by.unwrap_or(crate::plugin_marketplace::PluginSortOrder::Relevance),
        page_size: query.page_size.unwrap_or(20),
        page: query.page.unwrap_or(1),
    };
    
    let results = marketplace.search_plugins(&criteria);
    Ok(Json(ApiResponse::success(results)))
}

#[derive(Debug, Deserialize)]
pub struct PluginSearchQuery {
    pub query: Option<String>,
    pub category: Option<crate::plugin_marketplace::PluginCategory>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub min_rating: Option<f32>,
    pub sort_by: Option<crate::plugin_marketplace::PluginSortOrder>,
    pub page_size: Option<u32>,
    pub page: Option<u32>,
}

/// List available plugins
pub async fn list_available_plugins(
    State(state): State<PluginMarketplaceAppState>,
    Query(query): Query<ListPluginsQuery>,
) -> Result<Json<ApiResponse<Vec<PluginInfo>>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let mut plugins: Vec<PluginInfo> = marketplace.available_plugins.values().cloned().collect();
    
    // Apply limit
    if let Some(limit) = query.limit {
        plugins.truncate(limit as usize);
    }
    
    Ok(Json(ApiResponse::success(plugins)))
}

#[derive(Debug, Deserialize)]
pub struct ListPluginsQuery {
    pub limit: Option<u32>,
}

/// Get plugin details
pub async fn get_plugin_details(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<PluginInfo>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let plugin = marketplace.available_plugins.get(&plugin_id)
        .ok_or_else(|| ApiError::NotFound("Plugin not found".to_string()))?;
    
    Ok(Json(ApiResponse::success(plugin.clone())))
}

/// List installed plugins
pub async fn list_installed_plugins(
    State(state): State<PluginMarketplaceAppState>,
) -> Result<Json<ApiResponse<Vec<InstalledPlugin>>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    let plugins = marketplace.get_installed_plugins().into_iter().cloned().collect();
    Ok(Json(ApiResponse::success(plugins)))
}

/// Get installed plugin details
pub async fn get_installed_plugin(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<InstalledPlugin>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let plugin = marketplace.get_installed_plugin(&plugin_id)
        .ok_or_else(|| ApiError::NotFound("Plugin not installed".to_string()))?;
    
    Ok(Json(ApiResponse::success(plugin.clone())))
}

/// Install a plugin
pub async fn install_plugin(
    State(state): State<PluginMarketplaceAppState>,
    Json(request): Json<PluginInstallRequest>,
) -> Result<Json<ApiResponse<PluginInstallResponse>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    
    let install_path = marketplace.install_plugin(&request).await
        .map_err(|e| ApiError::BadRequest(format!("Installation failed: {}", e)))?;
    
    let response = PluginInstallResponse {
        plugin_id: request.plugin_id,
        install_path,
        installed_at: chrono::Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct PluginInstallResponse {
    pub plugin_id: String,
    pub install_path: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

/// Uninstall a plugin
pub async fn uninstall_plugin(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<PluginUninstallResponse>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    
    marketplace.uninstall_plugin(&plugin_id)
        .map_err(|e| ApiError::BadRequest(format!("Uninstallation failed: {}", e)))?;
    
    let response = PluginUninstallResponse {
        plugin_id,
        uninstalled_at: chrono::Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct PluginUninstallResponse {
    pub plugin_id: String,
    pub uninstalled_at: chrono::DateTime<chrono::Utc>,
}

/// Update a plugin
pub async fn update_plugin(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<PluginUpdateResponse>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    
    marketplace.update_plugin(&plugin_id).await
        .map_err(|e| ApiError::BadRequest(format!("Update failed: {}", e)))?;
    
    let response = PluginUpdateResponse {
        plugin_id,
        updated_at: chrono::Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct PluginUpdateResponse {
    pub plugin_id: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Enable a plugin
pub async fn enable_plugin(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<PluginStatusResponse>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    
    marketplace.set_plugin_enabled(&plugin_id, true)
        .map_err(|e| ApiError::BadRequest(format!("Failed to enable plugin: {}", e)))?;
    
    let response = PluginStatusResponse {
        plugin_id,
        status: PluginStatus::Ready,
        changed_at: chrono::Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

/// Disable a plugin
pub async fn disable_plugin(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<PluginStatusResponse>>, ApiError> {
    let mut marketplace = state.marketplace.write().await;
    
    marketplace.set_plugin_enabled(&plugin_id, false)
        .map_err(|e| ApiError::BadRequest(format!("Failed to disable plugin: {}", e)))?;
    
    let response = PluginStatusResponse {
        plugin_id,
        status: PluginStatus::Disabled,
        changed_at: chrono::Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct PluginStatusResponse {
    pub plugin_id: String,
    pub status: PluginStatus,
    pub changed_at: chrono::DateTime<chrono::Utc>,
}

/// Get plugin status
pub async fn get_plugin_status(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<PluginStatus>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let plugin = marketplace.get_installed_plugin(&plugin_id)
        .ok_or_else(|| ApiError::NotFound("Plugin not installed".to_string()))?;
    
    Ok(Json(ApiResponse::success(plugin.status.clone())))
}

/// Get plugin configuration
pub async fn get_plugin_config(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<std::collections::HashMap<String, String>>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let plugin = marketplace.get_installed_plugin(&plugin_id)
        .ok_or_else(|| ApiError::NotFound("Plugin not installed".to_string()))?;
    
    Ok(Json(ApiResponse::success(plugin.config.clone())))
}

/// Update plugin configuration
#[derive(Debug, Deserialize)]
pub struct UpdatePluginConfigRequest {
    pub config: std::collections::HashMap<String, String>,
}

pub async fn update_plugin_config(
    State(_state): State<PluginMarketplaceAppState>,
    Path(_plugin_id): Path<String>,
    Json(_request): Json<UpdatePluginConfigRequest>,
) -> Result<Json<ApiResponse<std::collections::HashMap<String, String>>>, ApiError> {
    // Implementation would update plugin configuration
    // For now, return the request config
    Err(ApiError::BadRequest("Plugin configuration update not yet implemented".to_string()))
}

/// Execute a plugin
#[derive(Debug, Deserialize)]
pub struct ExecutePluginRequest {
    pub args: Option<Vec<String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

pub async fn execute_plugin(
    State(_state): State<PluginMarketplaceAppState>,
    Path(_plugin_id): Path<String>,
    Json(_request): Json<ExecutePluginRequest>,
) -> Result<Json<ApiResponse<PluginExecutionResponse>>, ApiError> {
    // Implementation would execute the plugin
    // For now, return a mock response
    let response = PluginExecutionResponse {
        execution_id: uuid::Uuid::new_v4().to_string(),
        started_at: chrono::Utc::now(),
        status: "running".to_string(),
    };
    
    Ok(Json(ApiResponse::success(response)))
}

#[derive(Debug, Serialize)]
pub struct PluginExecutionResponse {
    pub execution_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

/// Get plugin logs
pub async fn get_plugin_logs(
    State(_state): State<PluginMarketplaceAppState>,
    Path(_plugin_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    // Implementation would retrieve plugin logs
    // For now, return empty logs
    Ok(Json(ApiResponse::success(vec![])))
}

/// Get plugin statistics
pub async fn get_plugin_stats(
    State(state): State<PluginMarketplaceAppState>,
    Path(plugin_id): Path<String>,
) -> Result<Json<ApiResponse<crate::plugin_marketplace::PluginExecutionStats>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let plugin = marketplace.get_installed_plugin(&plugin_id)
        .ok_or_else(|| ApiError::NotFound("Plugin not installed".to_string()))?;
    
    Ok(Json(ApiResponse::success(plugin.execution_stats.clone())))
}

/// Get marketplace statistics
pub async fn get_marketplace_stats(
    State(state): State<PluginMarketplaceAppState>,
) -> Result<Json<ApiResponse<MarketplaceStats>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let stats = MarketplaceStats {
        total_available_plugins: marketplace.available_plugins.len() as u32,
        total_installed_plugins: marketplace.installed_plugins.len() as u32,
        enabled_plugins: marketplace.installed_plugins.values()
            .filter(|p| p.enabled)
            .count() as u32,
        disabled_plugins: marketplace.installed_plugins.values()
            .filter(|p| !p.enabled)
            .count() as u32,
        plugins_with_updates: marketplace.installed_plugins.values()
            .filter(|p| p.status == PluginStatus::UpdateAvailable)
            .count() as u32,
    };
    
    Ok(Json(ApiResponse::success(stats)))
}

#[derive(Debug, Serialize)]
pub struct MarketplaceStats {
    pub total_available_plugins: u32,
    pub total_installed_plugins: u32,
    pub enabled_plugins: u32,
    pub disabled_plugins: u32,
    pub plugins_with_updates: u32,
}

/// Get marketplace health
pub async fn get_marketplace_health(
    State(state): State<PluginMarketplaceAppState>,
) -> Result<Json<ApiResponse<MarketplaceHealth>>, ApiError> {
    let marketplace = state.marketplace.read().await;
    
    let health = MarketplaceHealth {
        status: "healthy".to_string(),
        plugin_directory_accessible: marketplace.plugin_dir.exists(),
        total_plugins: marketplace.installed_plugins.len() as u32,
        failed_plugins: marketplace.installed_plugins.values()
            .filter(|p| matches!(p.status, PluginStatus::ValidationError | PluginStatus::InstallationFailed))
            .count() as u32,
        last_marketplace_refresh: None, // Would track actual refresh time
    };
    
    Ok(Json(ApiResponse::success(health)))
}

#[derive(Debug, Serialize)]
pub struct MarketplaceHealth {
    pub status: String,
    pub plugin_directory_accessible: bool,
    pub total_plugins: u32,
    pub failed_plugins: u32,
    pub last_marketplace_refresh: Option<chrono::DateTime<chrono::Utc>>,
}