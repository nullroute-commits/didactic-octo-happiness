//! Dashboard Management REST API
//!
//! This module provides REST API endpoints for the dashboard management system,
//! including dashboard creation, panel management, and real-time data updates.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::{Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    dashboard_manager::{
        DashboardManager, Dashboard, DashboardPanel, DashboardTemplate,
        DashboardUpdate, PanelType, DataSource, DashboardVisibility, 
        TimeRange, PanelData, DashboardSettings, DashboardLayout,
    },
    web_types::{ApiError, ApiResponse},
};

/// Dashboard application state
#[derive(Clone)]
pub struct DashboardAppState {
    pub dashboard_manager: Arc<RwLock<DashboardManager>>,
}

/// Create dashboard management routes
pub fn create_dashboard_routes() -> Router<DashboardAppState> {
    Router::new()
        // Dashboard CRUD operations
        .route("/api/dashboards", get(list_dashboards))
        .route("/api/dashboards", post(create_dashboard))
        .route("/api/dashboards/:id", get(get_dashboard))
        .route("/api/dashboards/:id", put(update_dashboard))
        .route("/api/dashboards/:id", delete(delete_dashboard))
        
        // Panel management
        .route("/api/dashboards/:id/panels", post(add_panel))
        .route("/api/dashboards/:id/panels/:panel_id", delete(remove_panel))
        .route("/api/dashboards/:id/panels/:panel_id/data", get(get_panel_data))
        
        // Dashboard templates
        .route("/api/dashboard-templates", get(list_templates))
        .route("/api/dashboard-templates/:id", get(get_template))
        
        // Real-time data streaming
        .route("/api/dashboards/:id/stream", get(dashboard_websocket))
        
        // Dashboard sharing and permissions
        .route("/api/dashboards/:id/share", post(share_dashboard))
        .route("/api/dashboards/:id/permissions", get(get_dashboard_permissions))
        
        // Dashboard export/import
        .route("/api/dashboards/:id/export", get(export_dashboard))
        .route("/api/dashboards/import", post(import_dashboard))
}

/// Request structure for creating a new dashboard
#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<DashboardVisibility>,
}

/// Request structure for updating a dashboard
#[derive(Debug, Deserialize)]
pub struct UpdateDashboardRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub panels: Option<Vec<DashboardPanel>>,
    pub layout: Option<DashboardLayout>,
    pub settings: Option<DashboardSettings>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<DashboardVisibility>,
}

/// Request structure for adding a panel
#[derive(Debug, Deserialize)]
pub struct AddPanelRequest {
    pub title: String,
    pub panel_type: PanelType,
    pub position: PanelPosition,
    pub data_config: DataConfigRequest,
}

#[derive(Debug, Deserialize)]
pub struct PanelPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct DataConfigRequest {
    pub source: DataSource,
    pub metrics: Vec<MetricConfigRequest>,
    pub filters: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct MetricConfigRequest {
    pub query: String,
    pub label: String,
    pub unit: Option<String>,
    pub color: Option<String>,
}

/// Query parameters for listing dashboards
#[derive(Debug, Deserialize)]
pub struct ListDashboardsQuery {
    pub tags: Option<String>,
    pub created_by: Option<Uuid>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Query parameters for panel data
#[derive(Debug, Deserialize)]
pub struct PanelDataQuery {
    pub time_range: Option<String>,
    pub refresh: Option<bool>,
}

/// Dashboard sharing request
#[derive(Debug, Deserialize)]
pub struct ShareDashboardRequest {
    pub visibility: DashboardVisibility,
    pub users: Option<Vec<Uuid>>,
}

/// Dashboard export format
#[derive(Debug, Deserialize)]
pub struct ExportDashboardQuery {
    pub format: Option<String>,
}

/// Dashboard import request
#[derive(Debug, Deserialize)]
pub struct ImportDashboardRequest {
    pub dashboard_data: String,
    pub name: Option<String>,
}

/// List all dashboards accessible to the current user
pub async fn list_dashboards(
    State(state): State<DashboardAppState>,
    Query(query): Query<ListDashboardsQuery>,
) -> Result<Json<ApiResponse<Vec<Dashboard>>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4(); // Placeholder
    
    let mut dashboards = dashboard_manager.list_dashboards(user_id);
    
    // Apply filters
    if let Some(tags_filter) = query.tags {
        let filter_tags: Vec<&str> = tags_filter.split(',').collect();
        dashboards.retain(|dashboard| {
            dashboard.tags.iter().any(|tag| filter_tags.contains(&tag.as_str()))
        });
    }
    
    if let Some(created_by) = query.created_by {
        dashboards.retain(|dashboard| dashboard.created_by == created_by);
    }
    
    // Apply pagination
    let total = dashboards.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).min(100);
    
    let paginated: Vec<Dashboard> = dashboards
        .into_iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(paginated),
        message: Some(format!("Retrieved {} dashboards (total: {})", limit, total)),
        error: None,
    }))
}

/// Create a new dashboard
pub async fn create_dashboard(
    State(state): State<DashboardAppState>,
    Json(request): Json<CreateDashboardRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    match dashboard_manager.create_dashboard(
        request.name,
        request.description,
        user_id,
        request.template_id,
    ).await {
        Ok(dashboard_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(dashboard_id),
            message: Some("Dashboard created successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Get a specific dashboard
pub async fn get_dashboard(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Dashboard>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    match dashboard_manager.get_dashboard(dashboard_id) {
        Some(dashboard) => Ok(Json(ApiResponse {
            success: true,
            data: Some(dashboard.clone()),
            message: None,
            error: None,
        })),
        None => Err(ApiError::NotFound("Dashboard not found".to_string())),
    }
}

/// Update a dashboard
pub async fn update_dashboard(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(request): Json<UpdateDashboardRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    let update = DashboardUpdate {
        name: request.name,
        description: request.description,
        panels: request.panels,
        layout: request.layout,
        settings: request.settings,
        tags: request.tags,
        visibility: request.visibility,
        theme: None, // Not included in request for now
    };
    
    match dashboard_manager.update_dashboard(dashboard_id, update, user_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            message: Some("Dashboard updated successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Delete a dashboard
pub async fn delete_dashboard(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    match dashboard_manager.delete_dashboard(dashboard_id, user_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            message: Some("Dashboard deleted successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Add a panel to a dashboard
pub async fn add_panel(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(request): Json<AddPanelRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    let panel_id = Uuid::new_v4();
    let panel = DashboardPanel {
        id: panel_id,
        title: request.title,
        panel_type: request.panel_type,
        position: crate::dashboard_manager::PanelPosition {
            x: request.position.x,
            y: request.position.y,
            width: request.position.width,
            height: request.position.height,
        },
        data_config: crate::dashboard_manager::DataConfiguration {
            source: request.data_config.source,
            metrics: request.data_config.metrics.into_iter().map(|m| {
                crate::dashboard_manager::MetricConfiguration {
                    query: m.query,
                    label: m.label,
                    unit: m.unit,
                    color: m.color,
                }
            }).collect(),
            filters: request.data_config.filters.unwrap_or_default(),
            aggregation: None, // Can be added later
        },
        visualization: crate::dashboard_manager::VisualizationConfig {
            colors: vec![
                "#ff6b35".to_string(),
                "#4ecdc4".to_string(),
                "#45b7d1".to_string(),
            ],
            options: HashMap::new(),
            axes: None,
            legend: crate::dashboard_manager::LegendConfig {
                show: true,
                position: crate::dashboard_manager::LegendPosition::Bottom,
                max_width: None,
            },
        },
        alerts: Vec::new(),
        refresh_interval: Some(chrono::Duration::seconds(30)),
    };
    
    match dashboard_manager.add_panel(dashboard_id, panel, user_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(panel_id),
            message: Some("Panel added successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Remove a panel from a dashboard
pub async fn remove_panel(
    State(state): State<DashboardAppState>,
    Path((dashboard_id, panel_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    match dashboard_manager.remove_panel(dashboard_id, panel_id, user_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            message: Some("Panel removed successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Get data for a specific panel
pub async fn get_panel_data(
    State(state): State<DashboardAppState>,
    Path((dashboard_id, panel_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<PanelDataQuery>,
) -> Result<Json<ApiResponse<PanelData>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    let time_range = query.time_range.map(|tr| {
        // Parse time range string
        match tr.as_str() {
            "1h" => TimeRange::LastHours(1),
            "6h" => TimeRange::LastHours(6),
            "24h" => TimeRange::LastHours(24),
            "7d" => TimeRange::LastDays(7),
            "live" => TimeRange::Live,
            _ => TimeRange::LastHours(1),
        }
    });
    
    match dashboard_manager.get_panel_data(dashboard_id, panel_id, time_range).await {
        Ok(data) => Ok(Json(ApiResponse {
            success: true,
            data: Some(data),
            message: None,
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// List available dashboard templates
pub async fn list_templates(
    State(state): State<DashboardAppState>,
) -> Result<Json<ApiResponse<Vec<DashboardTemplate>>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    let templates: Vec<DashboardTemplate> = dashboard_manager
        .get_templates()
        .into_iter()
        .cloned()
        .collect();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(templates),
        message: None,
        error: None,
    }))
}

/// Get a specific dashboard template
pub async fn get_template(
    State(state): State<DashboardAppState>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<ApiResponse<DashboardTemplate>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    let template = dashboard_manager
        .get_templates()
        .into_iter()
        .find(|t| t.id == template_id)
        .cloned();
    
    match template {
        Some(template) => Ok(Json(ApiResponse {
            success: true,
            data: Some(template),
            message: None,
            error: None,
        })),
        None => Err(ApiError::NotFound("Template not found".to_string())),
    }
}

/// WebSocket endpoint for real-time dashboard updates
pub async fn dashboard_websocket(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_dashboard_websocket(socket, state, dashboard_id))
}

/// Handle WebSocket connection for real-time dashboard updates
async fn handle_dashboard_websocket(
    mut socket: WebSocket,
    state: DashboardAppState,
    dashboard_id: Uuid,
) {
    // TODO: Get actual user ID from authentication
    let user_id = Uuid::new_v4();
    
    // Create dashboard session
    let session_id = {
        let mut dashboard_manager = state.dashboard_manager.write().await;
        match dashboard_manager.create_session(dashboard_id, user_id).await {
            Ok(session_id) => session_id,
            Err(e) => {
                let _ = socket.send(Message::Text(format!("Error: {}", e))).await;
                return;
            }
        }
    };
    
    // Send initial data
    let initial_data = {
        let dashboard_manager = state.dashboard_manager.read().await;
        dashboard_manager.get_dashboard(dashboard_id)
            .map(|d| serde_json::to_string(d).unwrap_or_default())
            .unwrap_or_else(|| "{}".to_string())
    };
    
    if socket.send(Message::Text(initial_data)).await.is_err() {
        return;
    }
    
    // Handle incoming messages and send periodic updates
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle client requests for specific panel data
                        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(panel_id) = request.get("panel_id").and_then(|v| v.as_str()) {
                                if let Ok(panel_uuid) = Uuid::parse_str(panel_id) {
                                    let dashboard_manager = state.dashboard_manager.read().await;
                                    if let Ok(data) = dashboard_manager.get_panel_data(dashboard_id, panel_uuid, None).await {
                                        let response = serde_json::json!({
                                            "type": "panel_data",
                                            "panel_id": panel_id,
                                            "data": data
                                        });
                                        let _ = socket.send(Message::Text(response.to_string())).await;
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }
            
            // Send periodic updates (every 30 seconds)
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                let dashboard_manager = state.dashboard_manager.read().await;
                if let Some(dashboard) = dashboard_manager.get_dashboard(dashboard_id) {
                    let update = serde_json::json!({
                        "type": "dashboard_update",
                        "timestamp": chrono::Utc::now(),
                        "panel_count": dashboard.panels.len()
                    });
                    if socket.send(Message::Text(update.to_string())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
    
    // Clean up session
    let mut dashboard_manager = state.dashboard_manager.write().await;
    let _ = dashboard_manager.close_session(session_id).await;
}

/// Share a dashboard with other users
pub async fn share_dashboard(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
    Json(request): Json<ShareDashboardRequest>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    let update = DashboardUpdate {
        name: None,
        description: None,
        panels: None,
        layout: None,
        settings: None,
        tags: None,
        visibility: Some(request.visibility),
        theme: None,
    };
    
    match dashboard_manager.update_dashboard(dashboard_id, update, user_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            message: Some("Dashboard sharing updated successfully".to_string()),
            error: None,
        })),
        Err(e) => Err(ApiError::InternalServerError(e.to_string())),
    }
}

/// Get dashboard permissions
pub async fn get_dashboard_permissions(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<ApiResponse<DashboardVisibility>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    match dashboard_manager.get_dashboard(dashboard_id) {
        Some(dashboard) => Ok(Json(ApiResponse {
            success: true,
            data: Some(dashboard.visibility.clone()),
            message: None,
            error: None,
        })),
        None => Err(ApiError::NotFound("Dashboard not found".to_string())),
    }
}

/// Export a dashboard
pub async fn export_dashboard(
    State(state): State<DashboardAppState>,
    Path(dashboard_id): Path<Uuid>,
    Query(_query): Query<ExportDashboardQuery>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let dashboard_manager = state.dashboard_manager.read().await;
    
    match dashboard_manager.get_dashboard(dashboard_id) {
        Some(dashboard) => {
            match serde_json::to_string_pretty(dashboard) {
                Ok(json_data) => Ok(Json(ApiResponse {
                    success: true,
                    data: Some(json_data),
                    message: None,
                    error: None,
                })),
                Err(e) => Err(ApiError::InternalServerError(format!("Export failed: {}", e))),
            }
        }
        None => Err(ApiError::NotFound("Dashboard not found".to_string())),
    }
}

/// Import a dashboard
pub async fn import_dashboard(
    State(state): State<DashboardAppState>,
    Json(request): Json<ImportDashboardRequest>,
) -> Result<Json<ApiResponse<Uuid>>, ApiError> {
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    // TODO: Get actual user ID from authentication context
    let user_id = Uuid::new_v4();
    
    // Parse the dashboard data
    let dashboard_data: Dashboard = serde_json::from_str(&request.dashboard_data)
        .map_err(|e| ApiError::BadRequest(format!("Invalid dashboard data: {}", e)))?;
    
    // Create new dashboard with imported data
    let dashboard_id = dashboard_manager.create_dashboard(
        request.name.unwrap_or(dashboard_data.name),
        dashboard_data.description,
        user_id,
        None,
    ).await.map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    
    // Update with imported panels and settings
    let update = DashboardUpdate {
        name: None,
        description: None,
        panels: Some(dashboard_data.panels),
        layout: Some(dashboard_data.layout),
        settings: Some(dashboard_data.settings),
        tags: Some(dashboard_data.tags),
        visibility: Some(DashboardVisibility::Private), // Always import as private
        theme: Some(dashboard_data.theme),
    };
    
    dashboard_manager.update_dashboard(dashboard_id, update, user_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(dashboard_id),
        message: Some("Dashboard imported successfully".to_string()),
        error: None,
    }))
}