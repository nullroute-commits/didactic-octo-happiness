//! Types for the web application

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Information about a GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub language: Option<String>,
    pub languages_url: String,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub topics: Vec<String>,
    pub license: Option<GitHubLicense>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub default_branch: String,
    pub exposed_port: Option<u16>,
    pub health_check_path: Option<String>,
}

/// GitHub license information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLicense {
    pub key: String,
    pub name: String,
    pub spdx_id: Option<String>,
}

/// System profile generated from collect_info.sh output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub architecture: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub memory_total_mb: u64,
    pub memory_available_mb: u64,
    pub virtualization_type: Option<String>,
    pub container_runtimes: Vec<String>,
    pub network_interfaces: Vec<NetworkInterface>,
    pub hardware_capabilities: HardwareCapabilities,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub mtu: u32,
    pub state: String,
}

/// Hardware capabilities extracted from system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub has_gpu: bool,
    pub gpu_vendor: Option<String>,
    pub pcie_devices: u32,
    pub usb_devices: u32,
    pub network_devices: u32,
    pub disk_total_gb: u64,
    pub virtualization_support: bool,
}

/// Deployment profile for a specific software package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentProfile {
    pub id: Uuid,
    pub name: String,
    pub software_name: String,
    pub repository: GitHubRepository,
    pub system_requirements: SystemRequirements,
    pub container_config: ContainerConfig,
    pub optimizations: Vec<Optimization>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// System requirements for a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRequirements {
    pub min_memory_mb: u64,
    pub min_cpu_cores: u32,
    pub min_disk_gb: u64,
    pub required_architectures: Vec<String>,
    pub required_os: Vec<String>,
    pub required_container_runtime: Vec<String>,
    pub network_requirements: Vec<String>,
}

/// Container configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub image: String,
    pub tag: String,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMapping>,
    pub environment_variables: HashMap<String, String>,
    pub resource_limits: ResourceLimits,
    pub health_check: Option<HealthCheck>,
}

/// Port mapping for container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String, // tcp, udp
}

/// Volume mapping for container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

/// Resource limits for container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_mb: Option<u64>,
    pub cpu_shares: Option<u32>,
    pub cpu_quota: Option<u32>,
    pub cpu_period: Option<u32>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub test: Vec<String>,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub retries: u32,
    pub start_period_seconds: u32,
}

/// Optimization applied to a deployment profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub name: String,
    pub description: String,
    pub optimization_type: OptimizationType,
    pub parameters: HashMap<String, String>,
}

/// Type of optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    MemoryOptimization,
    CpuOptimization,
    NetworkOptimization,
    StorageOptimization,
    SecurityHardening,
    PerformanceTuning,
}

/// Active deployment instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInstance {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub name: String,
    pub status: DeploymentStatus,
    pub container_id: Option<String>,
    pub ports: Vec<u16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub logs: Vec<DeploymentLog>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Creating,
    Running,
    Stopped,
    Failed,
    Updating,
    Removing,
}

/// Deployment log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Request to create a new deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeploymentRequest {
    pub profile_id: Uuid,
    pub name: String,
    pub custom_config: Option<HashMap<String, String>>,
}

/// Response from creating a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDeploymentResponse {
    pub deployment: DeploymentInstance,
    pub warnings: Vec<String>,
}

/// Request to search GitHub repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRepositoriesRequest {
    pub query: String,
    pub language: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

/// Response from GitHub repository search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRepositoriesResponse {
    pub repositories: Vec<GitHubRepository>,
    pub total_count: u32,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
}

/// Request to generate deployment profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProfileRequest {
    pub repository: GitHubRepository,
    pub system_profile_id: Uuid,
    pub custom_requirements: Option<SystemRequirements>,
}

/// Response from generating deployment profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProfileResponse {
    pub profile: DeploymentProfile,
    pub compatibility_score: f64,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}