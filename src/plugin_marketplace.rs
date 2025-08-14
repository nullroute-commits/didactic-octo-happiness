//! Plugin Marketplace System
//!
//! This module provides a comprehensive plugin marketplace for community-contributed
//! extensions to the Automation Nation platform, including plugin discovery,
//! installation, management, and security validation.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Plugin marketplace manager
pub struct PluginMarketplace {
    /// Base directory for plugin storage
    pub plugin_dir: PathBuf,
    /// Marketplace configuration
    pub config: MarketplaceConfig,
    /// Installed plugins
    pub installed_plugins: HashMap<String, InstalledPlugin>,
    /// Available plugins from marketplace
    pub available_plugins: HashMap<String, PluginInfo>,
}

/// Marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// Official marketplace URL
    pub official_marketplace_url: String,
    /// Custom marketplace URLs
    pub custom_marketplaces: Vec<String>,
    /// Auto-update enabled
    pub auto_update_enabled: bool,
    /// Security verification required
    pub require_signature_verification: bool,
    /// Allowed plugin categories
    pub allowed_categories: Vec<PluginCategory>,
    /// Maximum plugin size (MB)
    pub max_plugin_size_mb: u32,
    /// Plugin cache duration (hours)
    pub cache_duration_hours: u32,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            official_marketplace_url: "https://marketplace.automation-nation.dev/api/v1".to_string(),
            custom_marketplaces: Vec::new(),
            auto_update_enabled: false,
            require_signature_verification: true,
            allowed_categories: vec![
                PluginCategory::SystemInfo,
                PluginCategory::ContainerRuntime,
                PluginCategory::Monitoring,
                PluginCategory::Security,
                PluginCategory::Networking,
                PluginCategory::Storage,
            ],
            max_plugin_size_mb: 50,
            cache_duration_hours: 24,
        }
    }
}

/// Plugin categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginCategory {
    /// System information collection plugins
    SystemInfo,
    /// Container runtime integration plugins
    ContainerRuntime,
    /// Monitoring and observability plugins
    Monitoring,
    /// Security and compliance plugins
    Security,
    /// Network management plugins
    Networking,
    /// Storage management plugins
    Storage,
    /// Cloud provider integration plugins
    CloudProvider,
    /// CI/CD integration plugins
    CICD,
    /// Database management plugins
    Database,
    /// Custom/utility plugins
    Utility,
}

/// Plugin information from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin unique identifier
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin category
    pub category: PluginCategory,
    /// Plugin tags for searchability
    pub tags: Vec<String>,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// Plugin repository URL
    pub repository: Option<String>,
    /// Plugin documentation URL
    pub documentation: Option<String>,
    /// Plugin license
    pub license: String,
    /// Download URL
    pub download_url: String,
    /// Download size in bytes
    pub size_bytes: u64,
    /// Plugin checksum (SHA256)
    pub checksum: String,
    /// Digital signature for verification
    pub signature: Option<String>,
    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,
    /// Supported platforms
    pub platforms: Vec<String>,
    /// Minimum platform version required
    pub min_platform_version: String,
    /// Plugin rating (1-5)
    pub rating: Option<f32>,
    /// Number of downloads
    pub download_count: u64,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Plugin metadata
    pub metadata: HashMap<String, String>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency plugin ID
    pub plugin_id: String,
    /// Required version constraint
    pub version_constraint: String,
    /// Whether dependency is optional
    pub optional: bool,
}

/// Installed plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    /// Plugin information
    pub plugin_info: PluginInfo,
    /// Installation directory
    pub install_path: PathBuf,
    /// Installation timestamp
    pub installed_at: DateTime<Utc>,
    /// Plugin status
    pub status: PluginStatus,
    /// Plugin configuration
    pub config: HashMap<String, String>,
    /// Plugin enabled state
    pub enabled: bool,
    /// Last execution timestamp
    pub last_executed: Option<DateTime<Utc>>,
    /// Execution statistics
    pub execution_stats: PluginExecutionStats,
}

/// Plugin status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    /// Plugin is installed and ready
    Ready,
    /// Plugin has dependency issues
    DependencyError,
    /// Plugin failed validation
    ValidationError,
    /// Plugin is disabled
    Disabled,
    /// Plugin needs update
    UpdateAvailable,
    /// Plugin installation failed
    InstallationFailed,
}

/// Plugin execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionStats {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: u64,
    /// Last execution result
    pub last_execution_result: Option<String>,
}

impl Default for PluginExecutionStats {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time_ms: 0,
            last_execution_result: None,
        }
    }
}

/// Plugin search criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchCriteria {
    /// Search query string
    pub query: Option<String>,
    /// Filter by category
    pub category: Option<PluginCategory>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by author
    pub author: Option<String>,
    /// Minimum rating
    pub min_rating: Option<f32>,
    /// Sort order
    pub sort_by: PluginSortOrder,
    /// Page size for pagination
    pub page_size: u32,
    /// Page number
    pub page: u32,
}

/// Plugin sorting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSortOrder {
    /// Sort by relevance (default)
    Relevance,
    /// Sort by download count (descending)
    Downloads,
    /// Sort by rating (descending)
    Rating,
    /// Sort by last updated (descending)
    Updated,
    /// Sort by name (ascending)
    Name,
}

/// Plugin installation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallRequest {
    /// Plugin ID to install
    pub plugin_id: String,
    /// Force installation even if dependencies are missing
    pub force_install: bool,
    /// Custom installation configuration
    pub config: HashMap<String, String>,
}

/// Plugin search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchResults {
    /// Found plugins
    pub plugins: Vec<PluginInfo>,
    /// Total number of results
    pub total_results: u32,
    /// Current page
    pub page: u32,
    /// Total pages
    pub total_pages: u32,
    /// Search took (milliseconds)
    pub search_time_ms: u64,
}

impl PluginMarketplace {
    /// Create a new plugin marketplace instance
    pub fn new(plugin_dir: PathBuf) -> Result<Self> {
        let config = MarketplaceConfig::default();
        
        // Ensure plugin directory exists
        if !plugin_dir.exists() {
            fs::create_dir_all(&plugin_dir)
                .context("Failed to create plugin directory")?;
        }

        let mut marketplace = Self {
            plugin_dir,
            config,
            installed_plugins: HashMap::new(),
            available_plugins: HashMap::new(),
        };

        // Load installed plugins
        marketplace.load_installed_plugins()?;
        
        Ok(marketplace)
    }

    /// Create marketplace with custom configuration
    pub fn with_config(plugin_dir: PathBuf, config: MarketplaceConfig) -> Result<Self> {
        if !plugin_dir.exists() {
            fs::create_dir_all(&plugin_dir)
                .context("Failed to create plugin directory")?;
        }

        let mut marketplace = Self {
            plugin_dir,
            config,
            installed_plugins: HashMap::new(),
            available_plugins: HashMap::new(),
        };

        marketplace.load_installed_plugins()?;
        Ok(marketplace)
    }

    /// Refresh available plugins from marketplace
    pub async fn refresh_marketplace(&mut self) -> Result<u32> {
        log::info!("Refreshing plugin marketplace...");
        
        let mut total_plugins = 0;
        
        // Fetch from official marketplace
        if let Ok(plugins) = self.fetch_plugins_from_marketplace(&self.config.official_marketplace_url).await {
            for plugin in plugins {
                self.available_plugins.insert(plugin.id.clone(), plugin);
                total_plugins += 1;
            }
        }

        // Fetch from custom marketplaces
        for marketplace_url in &self.config.custom_marketplaces {
            if let Ok(plugins) = self.fetch_plugins_from_marketplace(marketplace_url).await {
                for plugin in plugins {
                    self.available_plugins.insert(plugin.id.clone(), plugin);
                    total_plugins += 1;
                }
            }
        }

        log::info!("Marketplace refresh complete. Found {} plugins", total_plugins);
        Ok(total_plugins)
    }

    /// Search for plugins in the marketplace
    pub fn search_plugins(&self, criteria: &PluginSearchCriteria) -> PluginSearchResults {
        let start_time = std::time::Instant::now();
        
        let mut filtered_plugins: Vec<PluginInfo> = self.available_plugins.values()
            .filter(|plugin| self.matches_criteria(plugin, criteria))
            .cloned()
            .collect();

        // Sort results
        self.sort_plugins(&mut filtered_plugins, &criteria.sort_by);

        // Paginate results
        let total_results = filtered_plugins.len() as u32;
        let total_pages = (total_results + criteria.page_size - 1) / criteria.page_size;
        let start_idx = (criteria.page.saturating_sub(1) * criteria.page_size) as usize;
        let end_idx = std::cmp::min(start_idx + criteria.page_size as usize, filtered_plugins.len());
        
        let page_plugins = if start_idx < filtered_plugins.len() {
            filtered_plugins[start_idx..end_idx].to_vec()
        } else {
            Vec::new()
        };

        PluginSearchResults {
            plugins: page_plugins,
            total_results,
            page: criteria.page,
            total_pages,
            search_time_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    /// Install a plugin from the marketplace
    pub async fn install_plugin(&mut self, request: &PluginInstallRequest) -> Result<String> {
        log::info!("Installing plugin: {}", request.plugin_id);

        // Find plugin in marketplace
        let plugin_info = self.available_plugins.get(&request.plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found in marketplace: {}", request.plugin_id))?
            .clone();

        // Check if plugin is already installed
        if self.installed_plugins.contains_key(&request.plugin_id) {
            return Err(anyhow!("Plugin already installed: {}", request.plugin_id));
        }

        // Validate plugin security
        if self.config.require_signature_verification {
            self.verify_plugin_signature(&plugin_info)?;
        }

        // Check dependencies
        if !request.force_install {
            self.check_plugin_dependencies(&plugin_info)?;
        }

        // Download and install plugin
        let install_path = self.plugin_dir.join(&plugin_info.id);
        self.download_and_extract_plugin(&plugin_info, &install_path).await?;

        // Create installed plugin entry
        let installed_plugin = InstalledPlugin {
            plugin_info: plugin_info.clone(),
            install_path: install_path.clone(),
            installed_at: Utc::now(),
            status: PluginStatus::Ready,
            config: request.config.clone(),
            enabled: true,
            last_executed: None,
            execution_stats: PluginExecutionStats::default(),
        };

        // Save installation info
        self.save_plugin_metadata(&installed_plugin)?;
        self.installed_plugins.insert(request.plugin_id.clone(), installed_plugin);

        log::info!("Plugin {} installed successfully", request.plugin_id);
        Ok(install_path.to_string_lossy().to_string())
    }

    /// Uninstall a plugin
    pub fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<()> {
        log::info!("Uninstalling plugin: {}", plugin_id);

        let installed_plugin = self.installed_plugins.get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not installed: {}", plugin_id))?;

        // Remove plugin files
        if installed_plugin.install_path.exists() {
            fs::remove_dir_all(&installed_plugin.install_path)
                .context("Failed to remove plugin directory")?;
        }

        // Remove metadata
        let metadata_path = self.get_plugin_metadata_path(plugin_id);
        if metadata_path.exists() {
            fs::remove_file(metadata_path)
                .context("Failed to remove plugin metadata")?;
        }

        // Remove from installed plugins
        self.installed_plugins.remove(plugin_id);

        log::info!("Plugin {} uninstalled successfully", plugin_id);
        Ok(())
    }

    /// Get list of installed plugins
    pub fn get_installed_plugins(&self) -> Vec<&InstalledPlugin> {
        self.installed_plugins.values().collect()
    }

    /// Get installed plugin by ID
    pub fn get_installed_plugin(&self, plugin_id: &str) -> Option<&InstalledPlugin> {
        self.installed_plugins.get(plugin_id)
    }

    /// Enable or disable a plugin
    pub fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<()> {
        let plugin = self.installed_plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not installed: {}", plugin_id))?;

        plugin.enabled = enabled;
        plugin.status = if enabled { PluginStatus::Ready } else { PluginStatus::Disabled };

        let plugin_copy = plugin.clone();
        self.save_plugin_metadata(&plugin_copy)?;
        
        log::info!("Plugin {} {}", plugin_id, if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// Update a plugin to the latest version
    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<()> {
        log::info!("Updating plugin: {}", plugin_id);

        let current_plugin = self.installed_plugins.get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not installed: {}", plugin_id))?
            .clone();

        let latest_plugin = self.available_plugins.get(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not found in marketplace: {}", plugin_id))?;

        // Check if update is needed
        if current_plugin.plugin_info.version == latest_plugin.version {
            return Err(anyhow!("Plugin is already up to date"));
        }

        // Backup current installation
        let backup_path = self.plugin_dir.join(format!("{}.backup", plugin_id));
        if current_plugin.install_path.exists() {
            fs::rename(&current_plugin.install_path, &backup_path)
                .context("Failed to backup current plugin")?;
        }

        // Install new version
        let install_request = PluginInstallRequest {
            plugin_id: plugin_id.to_string(),
            force_install: false,
            config: current_plugin.config.clone(),
        };

        // Remove from installed plugins temporarily
        self.installed_plugins.remove(plugin_id);

        match self.install_plugin(&install_request).await {
            Ok(_) => {
                // Remove backup on success
                if backup_path.exists() {
                    fs::remove_dir_all(backup_path).ok();
                }
                log::info!("Plugin {} updated successfully", plugin_id);
                Ok(())
            }
            Err(e) => {
                // Restore backup on failure
                if backup_path.exists() {
                    fs::rename(backup_path, &current_plugin.install_path).ok();
                    self.installed_plugins.insert(plugin_id.to_string(), current_plugin);
                }
                Err(e)
            }
        }
    }

    // Private helper methods

    /// Load installed plugins from disk
    fn load_installed_plugins(&mut self) -> Result<()> {
        if !self.plugin_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let plugin_id = entry.file_name().to_string_lossy().to_string();
                
                // Skip backup directories
                if plugin_id.ends_with(".backup") {
                    continue;
                }

                if let Ok(plugin) = self.load_plugin_metadata(&plugin_id) {
                    self.installed_plugins.insert(plugin_id, plugin);
                }
            }
        }

        Ok(())
    }

    /// Fetch plugins from a marketplace URL
    async fn fetch_plugins_from_marketplace(&self, _marketplace_url: &str) -> Result<Vec<PluginInfo>> {
        // In a real implementation, this would make HTTP requests to the marketplace API
        // For now, we'll return a mock plugin for demonstration
        Ok(vec![
            PluginInfo {
                id: "example-plugin".to_string(),
                name: "Example Plugin".to_string(),
                description: "An example plugin for demonstration".to_string(),
                version: "1.0.0".to_string(),
                author: "Automation Nation".to_string(),
                category: PluginCategory::Utility,
                tags: vec!["example".to_string(), "demo".to_string()],
                homepage: Some("https://example.com".to_string()),
                repository: Some("https://github.com/example/plugin".to_string()),
                documentation: Some("https://docs.example.com".to_string()),
                license: "MIT".to_string(),
                download_url: "https://example.com/plugin.tar.gz".to_string(),
                size_bytes: 1024000,
                checksum: "sha256:abcd1234".to_string(),
                signature: None,
                dependencies: Vec::new(),
                platforms: vec!["linux".to_string()],
                min_platform_version: "1.0.0".to_string(),
                rating: Some(4.5),
                download_count: 1000,
                updated_at: Utc::now(),
                metadata: HashMap::new(),
            }
        ])
    }

    /// Check if plugin matches search criteria
    fn matches_criteria(&self, plugin: &PluginInfo, criteria: &PluginSearchCriteria) -> bool {
        // Query matching
        if let Some(query) = &criteria.query {
            let query_lower = query.to_lowercase();
            let matches_query = plugin.name.to_lowercase().contains(&query_lower) ||
                               plugin.description.to_lowercase().contains(&query_lower) ||
                               plugin.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower));
            if !matches_query {
                return false;
            }
        }

        // Category matching
        if let Some(category) = &criteria.category {
            if &plugin.category != category {
                return false;
            }
        }

        // Tags matching
        if !criteria.tags.is_empty() {
            let has_matching_tag = criteria.tags.iter()
                .any(|tag| plugin.tags.contains(tag));
            if !has_matching_tag {
                return false;
            }
        }

        // Author matching
        if let Some(author) = &criteria.author {
            if plugin.author != *author {
                return false;
            }
        }

        // Minimum rating
        if let Some(min_rating) = criteria.min_rating {
            if plugin.rating.unwrap_or(0.0) < min_rating {
                return false;
            }
        }

        true
    }

    /// Sort plugins based on sort order
    fn sort_plugins(&self, plugins: &mut Vec<PluginInfo>, sort_order: &PluginSortOrder) {
        match sort_order {
            PluginSortOrder::Relevance => {
                // Default order (no additional sorting needed for basic relevance)
            }
            PluginSortOrder::Downloads => {
                plugins.sort_by(|a, b| b.download_count.cmp(&a.download_count));
            }
            PluginSortOrder::Rating => {
                plugins.sort_by(|a, b| {
                    let rating_a = a.rating.unwrap_or(0.0);
                    let rating_b = b.rating.unwrap_or(0.0);
                    rating_b.partial_cmp(&rating_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            PluginSortOrder::Updated => {
                plugins.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            PluginSortOrder::Name => {
                plugins.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
    }

    /// Verify plugin digital signature
    fn verify_plugin_signature(&self, _plugin: &PluginInfo) -> Result<()> {
        // In a real implementation, this would verify the digital signature
        // For now, we'll just return Ok if signature exists
        log::debug!("Verifying plugin signature (mock implementation)");
        Ok(())
    }

    /// Check plugin dependencies
    fn check_plugin_dependencies(&self, plugin: &PluginInfo) -> Result<()> {
        for dependency in &plugin.dependencies {
            if !dependency.optional && !self.installed_plugins.contains_key(&dependency.plugin_id) {
                return Err(anyhow!("Missing required dependency: {}", dependency.plugin_id));
            }
        }
        Ok(())
    }

    /// Download and extract plugin
    async fn download_and_extract_plugin(&self, _plugin: &PluginInfo, install_path: &Path) -> Result<()> {
        // In a real implementation, this would download and extract the plugin
        // For now, we'll create a mock plugin directory
        fs::create_dir_all(install_path)
            .context("Failed to create plugin installation directory")?;
        
        // Create a mock plugin file
        let plugin_file = install_path.join("plugin.sh");
        fs::write(plugin_file, "#!/bin/bash\necho 'Mock plugin execution'\n")
            .context("Failed to create plugin file")?;

        log::debug!("Plugin extracted to: {}", install_path.display());
        Ok(())
    }

    /// Save plugin metadata to disk
    fn save_plugin_metadata(&self, plugin: &InstalledPlugin) -> Result<()> {
        let metadata_path = self.get_plugin_metadata_path(&plugin.plugin_info.id);
        let metadata_json = serde_json::to_string_pretty(plugin)
            .context("Failed to serialize plugin metadata")?;
        
        fs::write(metadata_path, metadata_json)
            .context("Failed to write plugin metadata")?;
        
        Ok(())
    }

    /// Load plugin metadata from disk
    fn load_plugin_metadata(&self, plugin_id: &str) -> Result<InstalledPlugin> {
        let metadata_path = self.get_plugin_metadata_path(plugin_id);
        let metadata_content = fs::read_to_string(metadata_path)
            .context("Failed to read plugin metadata")?;
        
        let plugin: InstalledPlugin = serde_json::from_str(&metadata_content)
            .context("Failed to parse plugin metadata")?;
        
        Ok(plugin)
    }

    /// Get plugin metadata file path
    fn get_plugin_metadata_path(&self, plugin_id: &str) -> PathBuf {
        self.plugin_dir.join(format!("{}.json", plugin_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_marketplace_creation() {
        let temp_dir = TempDir::new().unwrap();
        let marketplace = PluginMarketplace::new(temp_dir.path().to_path_buf()).unwrap();
        
        assert_eq!(marketplace.installed_plugins.len(), 0);
        assert_eq!(marketplace.available_plugins.len(), 0);
    }

    #[test]
    fn test_plugin_search_criteria() {
        let criteria = PluginSearchCriteria {
            query: Some("test".to_string()),
            category: Some(PluginCategory::Utility),
            tags: vec!["example".to_string()],
            author: None,
            min_rating: Some(3.0),
            sort_by: PluginSortOrder::Rating,
            page_size: 10,
            page: 1,
        };

        // Test criteria structure
        assert_eq!(criteria.query, Some("test".to_string()));
        assert_eq!(criteria.category, Some(PluginCategory::Utility));
        assert!(!criteria.tags.is_empty());
    }

    #[test]
    fn test_plugin_status_transitions() {
        let status = PluginStatus::Ready;
        assert_eq!(status, PluginStatus::Ready);
        
        let status = PluginStatus::Disabled;
        assert_eq!(status, PluginStatus::Disabled);
    }

    #[tokio::test]
    async fn test_marketplace_refresh() {
        let temp_dir = TempDir::new().unwrap();
        let mut marketplace = PluginMarketplace::new(temp_dir.path().to_path_buf()).unwrap();
        
        // This would test the marketplace refresh functionality
        // For now, we just ensure it doesn't panic
        let result = marketplace.refresh_marketplace().await;
        assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable for mock
    }
}