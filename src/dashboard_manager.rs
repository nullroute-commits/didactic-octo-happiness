//! # Custom Dashboard Management System
//!
//! This module provides comprehensive dashboard creation and management capabilities,
//! allowing users to create custom monitoring dashboards with real-time metrics,
//! alerts, and visualizations for the Automation Nation platform.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Dashboard configuration and layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    /// Unique dashboard identifier
    pub id: Uuid,
    /// Dashboard name
    pub name: String,
    /// Dashboard description
    pub description: Option<String>,
    /// Dashboard creator
    pub created_by: Uuid,
    /// Dashboard panels
    pub panels: Vec<DashboardPanel>,
    /// Dashboard layout configuration
    pub layout: DashboardLayout,
    /// Dashboard settings
    pub settings: DashboardSettings,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Dashboard tags for organization
    pub tags: Vec<String>,
    /// Dashboard visibility
    pub visibility: DashboardVisibility,
    /// Dashboard theme
    pub theme: DashboardTheme,
}

/// Individual dashboard panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    /// Panel unique identifier
    pub id: Uuid,
    /// Panel title
    pub title: String,
    /// Panel type (chart, table, text, etc.)
    pub panel_type: PanelType,
    /// Panel position and size
    pub position: PanelPosition,
    /// Panel data configuration
    pub data_config: DataConfiguration,
    /// Panel visualization settings
    pub visualization: VisualizationConfig,
    /// Panel alert configuration
    pub alerts: Vec<AlertConfiguration>,
    /// Panel refresh interval
    pub refresh_interval: Option<Duration>,
}

/// Panel position and dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    /// X coordinate (grid units)
    pub x: u32,
    /// Y coordinate (grid units)
    pub y: u32,
    /// Width (grid units)
    pub width: u32,
    /// Height (grid units)
    pub height: u32,
}

/// Dashboard layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    /// Grid columns
    pub columns: u32,
    /// Grid rows
    pub rows: u32,
    /// Panel margin
    pub margin: u32,
    /// Auto-arrange panels
    pub auto_arrange: bool,
}

/// Dashboard-wide settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSettings {
    /// Auto-refresh interval
    pub refresh_interval: Duration,
    /// Time range for data
    pub time_range: TimeRange,
    /// Enable real-time updates
    pub real_time: bool,
    /// Dashboard timezone
    pub timezone: String,
    /// Enable panel borders
    pub show_borders: bool,
    /// Enable panel titles
    pub show_titles: bool,
}

/// Dashboard visibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardVisibility {
    /// Private to creator
    Private,
    /// Shared with specific users
    Shared(Vec<Uuid>),
    /// Public to organization
    Organization,
    /// Public to all users
    Public,
}

/// Dashboard theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardTheme {
    Light,
    Dark,
    Auto,
    Custom(ThemeConfig),
}

/// Custom theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background_color: String,
    pub text_color: String,
    pub accent_color: String,
    pub panel_background: String,
    pub border_color: String,
}

/// Panel types supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    /// Time series chart
    TimeSeries,
    /// Bar chart
    BarChart,
    /// Pie chart
    PieChart,
    /// Gauge/meter
    Gauge,
    /// Single value display
    SingleValue,
    /// Data table
    Table,
    /// Text/markdown panel
    Text,
    /// Alert list
    AlertList,
    /// System health status
    HealthStatus,
    /// Container status grid
    ContainerGrid,
    /// Certificate status
    CertificateStatus,
    /// Custom plugin panel
    Custom(String),
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfiguration {
    /// Data source type
    pub source: DataSource,
    /// Metrics to display
    pub metrics: Vec<MetricConfiguration>,
    /// Data filters
    pub filters: HashMap<String, String>,
    /// Aggregation settings
    pub aggregation: Option<AggregationConfig>,
}

/// Data source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// Prometheus metrics
    Prometheus,
    /// Application logs
    Logs,
    /// System metrics
    System,
    /// Container metrics
    Container,
    /// Certificate metrics
    Certificate,
    /// Custom metrics
    Custom(String),
}

/// Metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfiguration {
    /// Metric name/query
    pub query: String,
    /// Display name
    pub label: String,
    /// Metric unit
    pub unit: Option<String>,
    /// Color configuration
    pub color: Option<String>,
}

/// Data aggregation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Aggregation function
    pub function: AggregationFunction,
    /// Time window
    pub window: Duration,
    /// Group by fields
    pub group_by: Vec<String>,
}

/// Aggregation functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationFunction {
    Sum,
    Average,
    Min,
    Max,
    Count,
    Percentile(f64),
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Chart colors
    pub colors: Vec<String>,
    /// Chart options
    pub options: HashMap<String, serde_json::Value>,
    /// Axis configuration
    pub axes: Option<AxesConfig>,
    /// Legend settings
    pub legend: LegendConfig,
}

/// Chart axes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxesConfig {
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
}

/// Individual axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    pub label: Option<String>,
    pub unit: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub log_scale: bool,
}

/// Legend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegendConfig {
    pub show: bool,
    pub position: LegendPosition,
    pub max_width: Option<u32>,
}

/// Legend position options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegendPosition {
    Top,
    Bottom,
    Left,
    Right,
    Inside,
}

/// Alert configuration for panels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfiguration {
    /// Alert name
    pub name: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Notification settings
    pub notifications: Vec<NotificationConfig>,
    /// Alert enabled status
    pub enabled: bool,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Threshold-based alert
    Threshold {
        metric: String,
        operator: ComparisonOperator,
        value: f64,
        duration: Duration,
    },
    /// Change-based alert
    Change {
        metric: String,
        change_type: ChangeType,
        threshold: f64,
        window: Duration,
    },
    /// Anomaly detection
    Anomaly {
        metric: String,
        sensitivity: f64,
        baseline_window: Duration,
    },
}

/// Comparison operators for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Change detection types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Increase,
    Decrease,
    AbsoluteChange,
    PercentageChange,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Notification type
    pub notification_type: NotificationType,
    /// Notification settings
    pub settings: HashMap<String, String>,
    /// Notification enabled
    pub enabled: bool,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    Slack,
    Webhook,
    PagerDuty,
    Teams,
}

/// Time range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeRange {
    /// Last N minutes
    LastMinutes(u32),
    /// Last N hours
    LastHours(u32),
    /// Last N days
    LastDays(u32),
    /// Custom date range
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    /// Live/real-time
    Live,
}

/// Dashboard template for quick creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardTemplate {
    /// Template identifier
    pub id: Uuid,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template category
    pub category: String,
    /// Template panels
    pub panels: Vec<PanelTemplate>,
    /// Template settings
    pub settings: DashboardSettings,
    /// Template tags
    pub tags: Vec<String>,
}

/// Panel template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelTemplate {
    /// Panel title template
    pub title: String,
    /// Panel type
    pub panel_type: PanelType,
    /// Data configuration template
    pub data_config: DataConfiguration,
    /// Recommended position
    pub recommended_position: PanelPosition,
}

/// Dashboard Manager - Main interface for dashboard operations
pub struct DashboardManager {
    /// Dashboard storage
    dashboards: HashMap<Uuid, Dashboard>,
    /// Dashboard templates
    templates: HashMap<Uuid, DashboardTemplate>,
    /// Active dashboard sessions
    active_sessions: HashMap<Uuid, DashboardSession>,
}

/// Active dashboard session for real-time updates
#[derive(Debug)]
pub struct DashboardSession {
    /// Session ID
    pub id: Uuid,
    /// Dashboard ID
    pub dashboard_id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Last update time
    pub last_update: DateTime<Utc>,
    /// Session start time
    pub started_at: DateTime<Utc>,
}

impl DashboardManager {
    /// Create a new dashboard manager
    pub fn new() -> Self {
        Self {
            dashboards: HashMap::new(),
            templates: Self::load_default_templates(),
            active_sessions: HashMap::new(),
        }
    }

    /// Create a new dashboard
    pub async fn create_dashboard(
        &mut self,
        name: String,
        description: Option<String>,
        created_by: Uuid,
        template_id: Option<Uuid>,
    ) -> Result<Uuid> {
        let dashboard_id = Uuid::new_v4();
        let now = Utc::now();

        let dashboard_name = name.clone(); // Clone for logging
        let dashboard = if let Some(template_id) = template_id {
            // Create from template
            let template = self.templates.get(&template_id)
                .ok_or_else(|| anyhow!("Template not found"))?;
            
            Dashboard {
                id: dashboard_id,
                name,
                description,
                created_by,
                panels: self.create_panels_from_template(&template.panels)?,
                layout: DashboardLayout {
                    columns: 12,
                    rows: 8,
                    margin: 8,
                    auto_arrange: true,
                },
                settings: template.settings.clone(),
                created_at: now,
                updated_at: now,
                tags: template.tags.clone(),
                visibility: DashboardVisibility::Private,
                theme: DashboardTheme::Auto,
            }
        } else {
            // Create empty dashboard
            Dashboard {
                id: dashboard_id,
                name,
                description,
                created_by,
                panels: Vec::new(),
                layout: DashboardLayout {
                    columns: 12,
                    rows: 8,
                    margin: 8,
                    auto_arrange: true,
                },
                settings: DashboardSettings {
                    refresh_interval: Duration::minutes(5),
                    time_range: TimeRange::LastHours(1),
                    real_time: false,
                    timezone: "UTC".to_string(),
                    show_borders: true,
                    show_titles: true,
                },
                created_at: now,
                updated_at: now,
                tags: Vec::new(),
                visibility: DashboardVisibility::Private,
                theme: DashboardTheme::Auto,
            }
        };

        self.dashboards.insert(dashboard_id, dashboard);
        
        log::info!("Created dashboard '{}' with ID: {}", dashboard_name, dashboard_id);
        Ok(dashboard_id)
    }

    /// Get a dashboard by ID
    pub fn get_dashboard(&self, dashboard_id: Uuid) -> Option<&Dashboard> {
        self.dashboards.get(&dashboard_id)
    }

    /// Update a dashboard
    pub async fn update_dashboard(
        &mut self,
        dashboard_id: Uuid,
        updates: DashboardUpdate,
        user_id: Uuid,
    ) -> Result<()> {
        let dashboard = self.dashboards.get_mut(&dashboard_id)
            .ok_or_else(|| anyhow!("Dashboard not found"))?;

        // Check permissions
        if dashboard.created_by != user_id {
            // TODO: Implement proper permission checking
            return Err(anyhow!("Permission denied"));
        }

        // Apply updates
        if let Some(name) = updates.name {
            dashboard.name = name;
        }
        if let Some(description) = updates.description {
            dashboard.description = description;
        }
        if let Some(panels) = updates.panels {
            dashboard.panels = panels;
        }
        if let Some(layout) = updates.layout {
            dashboard.layout = layout;
        }
        if let Some(settings) = updates.settings {
            dashboard.settings = settings;
        }
        if let Some(tags) = updates.tags {
            dashboard.tags = tags;
        }
        if let Some(visibility) = updates.visibility {
            dashboard.visibility = visibility;
        }
        if let Some(theme) = updates.theme {
            dashboard.theme = theme;
        }

        dashboard.updated_at = Utc::now();

        log::info!("Updated dashboard with ID: {}", dashboard_id);
        Ok(())
    }

    /// Add a panel to a dashboard
    pub async fn add_panel(
        &mut self,
        dashboard_id: Uuid,
        panel: DashboardPanel,
        user_id: Uuid,
    ) -> Result<()> {
        let dashboard = self.dashboards.get_mut(&dashboard_id)
            .ok_or_else(|| anyhow!("Dashboard not found"))?;

        // Check permissions
        if dashboard.created_by != user_id {
            return Err(anyhow!("Permission denied"));
        }

        dashboard.panels.push(panel);
        dashboard.updated_at = Utc::now();

        log::info!("Added panel to dashboard: {}", dashboard_id);
        Ok(())
    }

    /// Remove a panel from a dashboard
    pub async fn remove_panel(
        &mut self,
        dashboard_id: Uuid,
        panel_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let dashboard = self.dashboards.get_mut(&dashboard_id)
            .ok_or_else(|| anyhow!("Dashboard not found"))?;

        // Check permissions
        if dashboard.created_by != user_id {
            return Err(anyhow!("Permission denied"));
        }

        dashboard.panels.retain(|panel| panel.id != panel_id);
        dashboard.updated_at = Utc::now();

        log::info!("Removed panel {} from dashboard: {}", panel_id, dashboard_id);
        Ok(())
    }

    /// List dashboards accessible to a user
    pub fn list_dashboards(&self, user_id: Uuid) -> Vec<&Dashboard> {
        self.dashboards
            .values()
            .filter(|dashboard| self.can_access_dashboard(dashboard, user_id))
            .collect()
    }

    /// Delete a dashboard
    pub async fn delete_dashboard(
        &mut self,
        dashboard_id: Uuid,
        user_id: Uuid,
    ) -> Result<()> {
        let dashboard = self.dashboards.get(&dashboard_id)
            .ok_or_else(|| anyhow!("Dashboard not found"))?;

        // Check permissions
        if dashboard.created_by != user_id {
            return Err(anyhow!("Permission denied"));
        }

        self.dashboards.remove(&dashboard_id);
        
        // Close any active sessions for this dashboard
        self.active_sessions.retain(|_, session| session.dashboard_id != dashboard_id);

        log::info!("Deleted dashboard with ID: {}", dashboard_id);
        Ok(())
    }

    /// Get available dashboard templates
    pub fn get_templates(&self) -> Vec<&DashboardTemplate> {
        self.templates.values().collect()
    }

    /// Create a dashboard session for real-time updates
    pub async fn create_session(
        &mut self,
        dashboard_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid> {
        // Verify dashboard exists and user has access
        let dashboard = self.dashboards.get(&dashboard_id)
            .ok_or_else(|| anyhow!("Dashboard not found"))?;

        if !self.can_access_dashboard(dashboard, user_id) {
            return Err(anyhow!("Permission denied"));
        }

        let session_id = Uuid::new_v4();
        let session = DashboardSession {
            id: session_id,
            dashboard_id,
            user_id,
            last_update: Utc::now(),
            started_at: Utc::now(),
        };

        self.active_sessions.insert(session_id, session);

        log::info!("Created dashboard session: {}", session_id);
        Ok(session_id)
    }

    /// Close a dashboard session
    pub async fn close_session(&mut self, session_id: Uuid) -> Result<()> {
        self.active_sessions.remove(&session_id);
        log::info!("Closed dashboard session: {}", session_id);
        Ok(())
    }

    /// Get dashboard data for a panel
    pub async fn get_panel_data(
        &self,
        dashboard_id: Uuid,
        panel_id: Uuid,
        time_range: Option<TimeRange>,
    ) -> Result<PanelData> {
        let dashboard = self.dashboards.get(&dashboard_id)
            .ok_or_else(|| anyhow!("Dashboard not found"))?;

        let panel = dashboard.panels.iter()
            .find(|p| p.id == panel_id)
            .ok_or_else(|| anyhow!("Panel not found"))?;

        // Fetch data based on panel configuration
        let data = self.fetch_panel_data(panel, time_range).await?;

        Ok(data)
    }

    // Private helper methods

    fn load_default_templates() -> HashMap<Uuid, DashboardTemplate> {
        let mut templates = HashMap::new();

        // System Overview Template
        let system_template = DashboardTemplate {
            id: Uuid::new_v4(),
            name: "System Overview".to_string(),
            description: "Comprehensive system monitoring dashboard".to_string(),
            category: "System".to_string(),
            panels: vec![
                PanelTemplate {
                    title: "CPU Usage".to_string(),
                    panel_type: PanelType::TimeSeries,
                    data_config: DataConfiguration {
                        source: DataSource::System,
                        metrics: vec![MetricConfiguration {
                            query: "cpu_usage_percent".to_string(),
                            label: "CPU %".to_string(),
                            unit: Some("%".to_string()),
                            color: Some("#ff6b35".to_string()),
                        }],
                        filters: HashMap::new(),
                        aggregation: Some(AggregationConfig {
                            function: AggregationFunction::Average,
                            window: Duration::minutes(1),
                            group_by: vec![],
                        }),
                    },
                    recommended_position: PanelPosition { x: 0, y: 0, width: 6, height: 3 },
                },
                PanelTemplate {
                    title: "Memory Usage".to_string(),
                    panel_type: PanelType::Gauge,
                    data_config: DataConfiguration {
                        source: DataSource::System,
                        metrics: vec![MetricConfiguration {
                            query: "memory_usage_percent".to_string(),
                            label: "Memory %".to_string(),
                            unit: Some("%".to_string()),
                            color: Some("#4ecdc4".to_string()),
                        }],
                        filters: HashMap::new(),
                        aggregation: None,
                    },
                    recommended_position: PanelPosition { x: 6, y: 0, width: 6, height: 3 },
                },
            ],
            settings: DashboardSettings {
                refresh_interval: Duration::seconds(30),
                time_range: TimeRange::LastHours(1),
                real_time: true,
                timezone: "UTC".to_string(),
                show_borders: true,
                show_titles: true,
            },
            tags: vec!["system".to_string(), "monitoring".to_string()],
        };

        // Container Overview Template
        let container_template = DashboardTemplate {
            id: Uuid::new_v4(),
            name: "Container Overview".to_string(),
            description: "Container orchestration monitoring dashboard".to_string(),
            category: "Containers".to_string(),
            panels: vec![
                PanelTemplate {
                    title: "Running Containers".to_string(),
                    panel_type: PanelType::SingleValue,
                    data_config: DataConfiguration {
                        source: DataSource::Container,
                        metrics: vec![MetricConfiguration {
                            query: "container_count_running".to_string(),
                            label: "Running".to_string(),
                            unit: None,
                            color: Some("#27ae60".to_string()),
                        }],
                        filters: HashMap::new(),
                        aggregation: None,
                    },
                    recommended_position: PanelPosition { x: 0, y: 0, width: 3, height: 2 },
                },
                PanelTemplate {
                    title: "Container Status Grid".to_string(),
                    panel_type: PanelType::ContainerGrid,
                    data_config: DataConfiguration {
                        source: DataSource::Container,
                        metrics: vec![MetricConfiguration {
                            query: "container_status".to_string(),
                            label: "Status".to_string(),
                            unit: None,
                            color: None,
                        }],
                        filters: HashMap::new(),
                        aggregation: None,
                    },
                    recommended_position: PanelPosition { x: 0, y: 2, width: 12, height: 4 },
                },
            ],
            settings: DashboardSettings {
                refresh_interval: Duration::seconds(10),
                time_range: TimeRange::Live,
                real_time: true,
                timezone: "UTC".to_string(),
                show_borders: true,
                show_titles: true,
            },
            tags: vec!["containers".to_string(), "docker".to_string(), "podman".to_string()],
        };

        templates.insert(system_template.id, system_template);
        templates.insert(container_template.id, container_template);

        templates
    }

    fn create_panels_from_template(&self, panel_templates: &[PanelTemplate]) -> Result<Vec<DashboardPanel>> {
        let mut panels = Vec::new();
        
        for template in panel_templates {
            let panel = DashboardPanel {
                id: Uuid::new_v4(),
                title: template.title.clone(),
                panel_type: template.panel_type.clone(),
                position: template.recommended_position.clone(),
                data_config: template.data_config.clone(),
                visualization: VisualizationConfig {
                    colors: vec![
                        "#ff6b35".to_string(),
                        "#4ecdc4".to_string(),
                        "#45b7d1".to_string(),
                        "#f9ca24".to_string(),
                        "#6c5ce7".to_string(),
                    ],
                    options: HashMap::new(),
                    axes: None,
                    legend: LegendConfig {
                        show: true,
                        position: LegendPosition::Bottom,
                        max_width: None,
                    },
                },
                alerts: Vec::new(),
                refresh_interval: Some(Duration::seconds(30)),
            };
            panels.push(panel);
        }
        
        Ok(panels)
    }

    fn can_access_dashboard(&self, dashboard: &Dashboard, user_id: Uuid) -> bool {
        match &dashboard.visibility {
            DashboardVisibility::Private => dashboard.created_by == user_id,
            DashboardVisibility::Shared(users) => {
                dashboard.created_by == user_id || users.contains(&user_id)
            }
            DashboardVisibility::Organization | DashboardVisibility::Public => true,
        }
    }

    async fn fetch_panel_data(&self, panel: &DashboardPanel, _time_range: Option<TimeRange>) -> Result<PanelData> {
        // This is a placeholder implementation
        // In a real implementation, this would fetch data from the appropriate data source
        
        match &panel.data_config.source {
            DataSource::Prometheus => {
                // Fetch from Prometheus
                Ok(PanelData::TimeSeries(vec![
                    TimeSeriesPoint {
                        timestamp: Utc::now(),
                        value: 75.5,
                    },
                    TimeSeriesPoint {
                        timestamp: Utc::now() - Duration::seconds(30),
                        value: 73.2,
                    },
                ]))
            }
            DataSource::System => {
                // Fetch system metrics
                Ok(PanelData::SingleValue(42.0))
            }
            DataSource::Container => {
                // Fetch container metrics
                Ok(PanelData::SingleValue(5.0))
            }
            _ => Ok(PanelData::SingleValue(0.0)),
        }
    }
}

/// Dashboard update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUpdate {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub panels: Option<Vec<DashboardPanel>>,
    pub layout: Option<DashboardLayout>,
    pub settings: Option<DashboardSettings>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<DashboardVisibility>,
    pub theme: Option<DashboardTheme>,
}

/// Panel data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelData {
    TimeSeries(Vec<TimeSeriesPoint>),
    SingleValue(f64),
    Table(Vec<HashMap<String, serde_json::Value>>),
    Text(String),
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_creation() {
        let mut manager = DashboardManager::new();
        let user_id = Uuid::new_v4();
        
        let dashboard_id = manager.create_dashboard(
            "Test Dashboard".to_string(),
            Some("Test description".to_string()),
            user_id,
            None,
        ).await.unwrap();
        
        let dashboard = manager.get_dashboard(dashboard_id).unwrap();
        assert_eq!(dashboard.name, "Test Dashboard");
        assert_eq!(dashboard.created_by, user_id);
    }

    #[tokio::test]
    async fn test_dashboard_from_template() {
        let mut manager = DashboardManager::new();
        let user_id = Uuid::new_v4();
        
        // Get a template
        let template_id = manager.get_templates().first().unwrap().id;
        
        let dashboard_id = manager.create_dashboard(
            "Template Dashboard".to_string(),
            None,
            user_id,
            Some(template_id),
        ).await.unwrap();
        
        let dashboard = manager.get_dashboard(dashboard_id).unwrap();
        assert!(!dashboard.panels.is_empty());
    }

    #[tokio::test]
    async fn test_panel_management() {
        let mut manager = DashboardManager::new();
        let user_id = Uuid::new_v4();
        
        let dashboard_id = manager.create_dashboard(
            "Panel Test".to_string(),
            None,
            user_id,
            None,
        ).await.unwrap();
        
        let panel = DashboardPanel {
            id: Uuid::new_v4(),
            title: "Test Panel".to_string(),
            panel_type: PanelType::SingleValue,
            position: PanelPosition { x: 0, y: 0, width: 4, height: 2 },
            data_config: DataConfiguration {
                source: DataSource::System,
                metrics: vec![],
                filters: HashMap::new(),
                aggregation: None,
            },
            visualization: VisualizationConfig {
                colors: vec![],
                options: HashMap::new(),
                axes: None,
                legend: LegendConfig {
                    show: false,
                    position: LegendPosition::Bottom,
                    max_width: None,
                },
            },
            alerts: vec![],
            refresh_interval: None,
        };
        
        let panel_id = panel.id;
        manager.add_panel(dashboard_id, panel, user_id).await.unwrap();
        
        let dashboard = manager.get_dashboard(dashboard_id).unwrap();
        assert_eq!(dashboard.panels.len(), 1);
        
        manager.remove_panel(dashboard_id, panel_id, user_id).await.unwrap();
        
        let dashboard = manager.get_dashboard(dashboard_id).unwrap();
        assert_eq!(dashboard.panels.len(), 0);
    }

    #[test]
    fn test_dashboard_templates() {
        let manager = DashboardManager::new();
        let templates = manager.get_templates();
        
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.name == "System Overview"));
        assert!(templates.iter().any(|t| t.name == "Container Overview"));
    }

    #[tokio::test]
    async fn test_session_management() {
        let mut manager = DashboardManager::new();
        let user_id = Uuid::new_v4();
        
        let dashboard_id = manager.create_dashboard(
            "Session Test".to_string(),
            None,
            user_id,
            None,
        ).await.unwrap();
        
        let session_id = manager.create_session(dashboard_id, user_id).await.unwrap();
        assert!(manager.active_sessions.contains_key(&session_id));
        
        manager.close_session(session_id).await.unwrap();
        assert!(!manager.active_sessions.contains_key(&session_id));
    }
}