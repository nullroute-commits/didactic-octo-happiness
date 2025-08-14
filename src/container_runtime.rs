//! Unified container runtime abstraction layer

use crate::web_types::*;
use crate::Result;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import the individual runtime managers
use crate::podman_manager::PodmanManager;
use crate::docker_manager::DockerManager;
use crate::lxc_manager::LxcManager;
use crate::kubernetes_manager::KubernetesManager;

/// Supported container runtime types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeType {
    #[serde(rename = "podman")]
    Podman,
    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "lxc")]
    Lxc,
    #[serde(rename = "containerd")]
    Containerd,
    #[serde(rename = "kubernetes")]
    Kubernetes,
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeType::Podman => write!(f, "podman"),
            RuntimeType::Docker => write!(f, "docker"),
            RuntimeType::Lxc => write!(f, "lxc"),
            RuntimeType::Containerd => write!(f, "containerd"),
            RuntimeType::Kubernetes => write!(f, "kubernetes"),
        }
    }
}

/// Runtime availability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub runtime_type: String,
    pub available: bool,
    pub version: String,
    pub capabilities: RuntimeCapabilities,
}

/// Container runtime capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    pub supports_networking: bool,
    pub supports_volumes: bool,
    pub supports_resource_limits: bool,
    pub supports_health_checks: bool,
    pub supports_rolling_updates: bool,
    pub supports_load_balancing: bool,
    pub supports_service_discovery: bool,
    pub supports_secrets_management: bool,
    pub max_cpu_cores: Option<u32>,
    pub max_memory_gb: Option<u32>,
}

/// Unified container runtime manager
pub struct ContainerRuntimeManager {
    podman_manager: PodmanManager,
    docker_manager: DockerManager,
    lxc_manager: LxcManager,
    kubernetes_manager: KubernetesManager,
    available_runtimes: HashMap<RuntimeType, RuntimeInfo>,
}

impl ContainerRuntimeManager {
    /// Create a new container runtime manager
    pub async fn new() -> Self {
        let mut manager = Self {
            podman_manager: PodmanManager::new(),
            docker_manager: DockerManager::new(),
            lxc_manager: LxcManager::new(),
            kubernetes_manager: KubernetesManager::new(),
            available_runtimes: HashMap::new(),
        };
        
        // Check availability of all runtimes
        manager.detect_available_runtimes().await;
        manager
    }

    /// Detect and cache available container runtimes
    pub async fn detect_available_runtimes(&mut self) {
        info!("Detecting available container runtimes...");
        
        // Check Podman
        if let Ok(available) = self.podman_manager.check_availability().await {
            let info = RuntimeInfo {
                runtime_type: "podman".to_string(),
                available,
                version: "detected".to_string(),
                capabilities: RuntimeCapabilities {
                    supports_networking: true,
                    supports_volumes: true,
                    supports_resource_limits: true,
                    supports_health_checks: true,
                    supports_rolling_updates: false,
                    supports_load_balancing: false,
                    supports_service_discovery: false,
                    supports_secrets_management: true,
                    max_cpu_cores: Some(64),
                    max_memory_gb: Some(256),
                },
            };
            self.available_runtimes.insert(RuntimeType::Podman, info);
        }

        // Check Docker
        if let Ok(available) = self.docker_manager.check_availability().await {
            let info = RuntimeInfo {
                runtime_type: "docker".to_string(),
                available,
                version: "detected".to_string(),
                capabilities: RuntimeCapabilities {
                    supports_networking: true,
                    supports_volumes: true,
                    supports_resource_limits: true,
                    supports_health_checks: true,
                    supports_rolling_updates: false,
                    supports_load_balancing: false,
                    supports_service_discovery: false,
                    supports_secrets_management: true,
                    max_cpu_cores: Some(64),
                    max_memory_gb: Some(256),
                },
            };
            self.available_runtimes.insert(RuntimeType::Docker, info);
        }

        // Check LXC
        if let Ok(available) = self.lxc_manager.check_availability().await {
            let info = RuntimeInfo {
                runtime_type: "lxc".to_string(),
                available,
                version: "detected".to_string(),
                capabilities: RuntimeCapabilities {
                    supports_networking: true,
                    supports_volumes: true,
                    supports_resource_limits: true,
                    supports_health_checks: false,
                    supports_rolling_updates: false,
                    supports_load_balancing: false,
                    supports_service_discovery: false,
                    supports_secrets_management: false,
                    max_cpu_cores: Some(32),
                    max_memory_gb: Some(128),
                },
            };
            self.available_runtimes.insert(RuntimeType::Lxc, info);
        }

        // Check Kubernetes
        if let Ok(_available) = self.kubernetes_manager.check_availability().await {
            let k8s_info = self.kubernetes_manager.get_runtime_info().await.unwrap_or_else(|_| {
                RuntimeInfo {
                    runtime_type: "kubernetes".to_string(),
                    available: false,
                    version: "unknown".to_string(),
                    capabilities: RuntimeCapabilities {
                        supports_networking: false,
                        supports_volumes: false,
                        supports_resource_limits: false,
                        supports_health_checks: false,
                        supports_rolling_updates: false,
                        supports_load_balancing: false,
                        supports_service_discovery: false,
                        supports_secrets_management: false,
                        max_cpu_cores: None,
                        max_memory_gb: None,
                    },
                }
            });
            self.available_runtimes.insert(RuntimeType::Kubernetes, k8s_info);
        }

        info!("Runtime detection complete. Found {} available runtimes", 
              self.available_runtimes.len());
    }

    /// Get information about all detected runtimes
    pub fn get_runtime_info(&self) -> &HashMap<RuntimeType, RuntimeInfo> {
        &self.available_runtimes
    }

    /// Get available runtime types
    pub fn get_available_runtimes(&self) -> Vec<RuntimeType> {
        self.available_runtimes.keys().cloned().collect()
    }

    /// Get the preferred runtime (first available in priority order)
    pub fn get_preferred_runtime(&self) -> Option<RuntimeType> {
        // Priority order: Kubernetes > Podman > Docker > LXC > Containerd
        let priority_order = vec![
            RuntimeType::Kubernetes,
            RuntimeType::Podman,
            RuntimeType::Docker,
            RuntimeType::Lxc,
            RuntimeType::Containerd,
        ];
        
        for runtime_type in priority_order {
            if let Some(info) = self.available_runtimes.get(&runtime_type) {
                if info.available {
                    return Some(runtime_type);
                }
            }
        }
        
        None
    }

    /// Deploy a container using the specified runtime
    pub async fn deploy(&self, runtime_type: RuntimeType, profile: &DeploymentProfile, request: &CreateDeploymentRequest) -> Result<CreateDeploymentResponse> {
        // Verify runtime is available
        if let Some(info) = self.available_runtimes.get(&runtime_type) {
            if !info.available {
                return Err(anyhow::anyhow!("Runtime {} is not available", runtime_type));
            }
        } else {
            return Err(anyhow::anyhow!("Unknown runtime type: {}", runtime_type));
        }

        info!("Deploying container using {} runtime", runtime_type);
        
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.deploy(profile, request).await,
            RuntimeType::Docker => self.docker_manager.deploy(profile, request).await,
            RuntimeType::Lxc => self.lxc_manager.deploy(profile, request).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.deploy(profile, request).await,
        }
    }

    /// Deploy using the preferred runtime
    pub async fn deploy_preferred(&self, profile: &DeploymentProfile, request: &CreateDeploymentRequest) -> Result<CreateDeploymentResponse> {
        if let Some(runtime_type) = self.get_preferred_runtime() {
            self.deploy(runtime_type, profile, request).await
        } else {
            Err(anyhow::anyhow!("No container runtime available"))
        }
    }

    /// Undeploy a container
    pub async fn undeploy(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance) -> Result<()> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.undeploy(deployment).await,
            RuntimeType::Docker => self.docker_manager.undeploy(deployment).await,
            RuntimeType::Lxc => self.lxc_manager.undeploy(deployment).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.undeploy(deployment).await,
        }
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance) -> Result<DeploymentStatus> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.get_deployment_status(deployment).await,
            RuntimeType::Docker => self.docker_manager.get_deployment_status(deployment).await,
            RuntimeType::Lxc => self.lxc_manager.get_deployment_status(deployment).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.get_deployment_status(deployment).await,
        }
    }

    /// Get container logs
    pub async fn get_container_logs(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance, tail_lines: u32) -> Result<Vec<DeploymentLog>> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.get_container_logs(deployment, tail_lines).await,
            RuntimeType::Docker => self.docker_manager.get_container_logs(deployment, tail_lines).await,
            RuntimeType::Lxc => self.lxc_manager.get_container_logs(deployment, tail_lines).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.get_container_logs(deployment, tail_lines).await,
        }
    }

    /// List deployments
    pub async fn list_deployments(&self, runtime_type: RuntimeType) -> Result<Vec<HashMap<String, String>>> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.list_deployments().await,
            RuntimeType::Docker => self.docker_manager.list_deployments().await,
            RuntimeType::Lxc => self.lxc_manager.list_deployments().await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.list_deployments().await,
        }
    }

    /// Restart deployment
    pub async fn restart_deployment(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance) -> Result<()> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.restart_deployment(deployment).await,
            RuntimeType::Docker => self.docker_manager.restart_deployment(deployment).await,
            RuntimeType::Lxc => self.lxc_manager.restart_deployment(deployment).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.restart_deployment(deployment).await,
        }
    }

    /// Update resources
    pub async fn update_resources(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance, limits: &ResourceLimits) -> Result<()> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.update_resources(deployment, limits).await,
            RuntimeType::Docker => self.docker_manager.update_resources(deployment, limits).await,
            RuntimeType::Lxc => self.lxc_manager.update_resources(deployment, limits).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
            RuntimeType::Kubernetes => self.kubernetes_manager.update_resources(deployment, limits).await,
        }
    }

    /// Get runtime capabilities
    pub fn get_runtime_capabilities(&self, runtime_type: RuntimeType) -> RuntimeCapabilities {
        match runtime_type {
            RuntimeType::Podman => RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: true,
                supports_rolling_updates: false,
                supports_load_balancing: false,
                supports_service_discovery: false,
                supports_secrets_management: true,
                max_cpu_cores: Some(64),
                max_memory_gb: Some(256),
            },
            RuntimeType::Docker => RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: true,
                supports_rolling_updates: false,
                supports_load_balancing: false,
                supports_service_discovery: false,
                supports_secrets_management: true,
                max_cpu_cores: Some(64),
                max_memory_gb: Some(256),
            },
            RuntimeType::Lxc => RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: false,
                supports_rolling_updates: false,
                supports_load_balancing: false,
                supports_service_discovery: false,
                supports_secrets_management: false,
                max_cpu_cores: Some(32),
                max_memory_gb: Some(128),
            },
            RuntimeType::Containerd => RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: false,
                supports_rolling_updates: false,
                supports_load_balancing: false,
                supports_service_discovery: false,
                supports_secrets_management: true,
                max_cpu_cores: Some(64),
                max_memory_gb: Some(256),
            },
            RuntimeType::Kubernetes => RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: true,
                supports_rolling_updates: true,
                supports_load_balancing: true,
                supports_service_discovery: true,
                supports_secrets_management: true,
                max_cpu_cores: None, // Scalable based on cluster
                max_memory_gb: None, // Scalable based on cluster
            },
        }
    }

    /// Recommend best runtime for a deployment profile
    pub fn recommend_runtime(&self, profile: &DeploymentProfile) -> Option<RuntimeType> {
        let available_runtimes = self.get_available_runtimes();
        
        if available_runtimes.is_empty() {
            return None;
        }

        // Simple scoring system based on requirements
        let mut scores = HashMap::new();
        
        for runtime_type in &available_runtimes {
            let capabilities = self.get_runtime_capabilities(runtime_type.clone());
            let mut score = 0;
            
            // Base scores for runtime preference
            match runtime_type {
                RuntimeType::Podman => score += 10, // Preferred for security and rootless
                RuntimeType::Docker => score += 8,  // Good all-around
                RuntimeType::Lxc => score += 6,     // Good for system containers
                RuntimeType::Containerd => score += 4, // More low-level
                RuntimeType::Kubernetes => score += 12, // Best for production scaling
            }
            
            // Bonus for specific requirements
            if profile.system_requirements.required_architectures.len() > 1 && capabilities.supports_secrets_management {
                score += 3; // Multi-arch usually benefits from advanced security
            }
            
            if profile.container_config.volumes.len() > 3 && capabilities.supports_volumes {
                score += 2; // Many volumes
            }
            
            if profile.container_config.ports.len() > 5 && capabilities.supports_networking {
                score += 2; // Many ports
            }
            
            if profile.optimizations.iter().any(|opt| matches!(opt.optimization_type, OptimizationType::SecurityHardening)) {
                if capabilities.supports_secrets_management {
                    score += 5; // Security focus
                }
            }
            
            scores.insert(runtime_type.clone(), score);
        }
        
        // Return runtime with highest score
        scores.into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(runtime_type, _)| runtime_type)
    }
}

impl Default for ContainerRuntimeManager {
    fn default() -> Self {
        // Note: This blocks, should use new() for async initialization
        Self {
            podman_manager: PodmanManager::new(),
            docker_manager: DockerManager::new(),
            lxc_manager: LxcManager::new(),
            kubernetes_manager: KubernetesManager::new(),
            available_runtimes: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_runtime_manager_creation() {
        let manager = ContainerRuntimeManager::new().await;
        // Should have some runtimes detected (even if not available)
        assert!(!manager.available_runtimes.is_empty());
    }

    #[test]
    fn test_runtime_type_display() {
        assert_eq!(RuntimeType::Podman.to_string(), "podman");
        assert_eq!(RuntimeType::Docker.to_string(), "docker");
        assert_eq!(RuntimeType::Lxc.to_string(), "lxc");
        assert_eq!(RuntimeType::Containerd.to_string(), "containerd");
    }

    #[test]
    fn test_runtime_capabilities() {
        let manager = ContainerRuntimeManager::default();
        
        let podman_caps = manager.get_runtime_capabilities(RuntimeType::Podman);
        assert!(podman_caps.supports_networking);
        assert!(podman_caps.supports_volumes);
        
        let docker_caps = manager.get_runtime_capabilities(RuntimeType::Docker);
        assert!(docker_caps.supports_volumes);
        assert!(docker_caps.supports_health_checks);
        
        let lxc_caps = manager.get_runtime_capabilities(RuntimeType::Lxc);
        assert!(lxc_caps.supports_networking);
        assert!(!lxc_caps.supports_health_checks);
    }

    #[test]
    fn test_runtime_recommendation() {
        let mut manager = ContainerRuntimeManager::default();
        
        // Mock available runtimes
        manager.available_runtimes.insert(RuntimeType::Podman, RuntimeInfo {
            runtime_type: "podman".to_string(),
            available: true,
            version: "4.0.0".to_string(),
            capabilities: RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: true,
                supports_rolling_updates: false,
                supports_load_balancing: false,
                supports_service_discovery: false,
                supports_secrets_management: true,
                max_cpu_cores: Some(64),
                max_memory_gb: Some(256),
            },
        });
        manager.available_runtimes.insert(RuntimeType::Docker, RuntimeInfo {
            runtime_type: "docker".to_string(),
            available: true,
            version: "24.0.0".to_string(),
            capabilities: RuntimeCapabilities {
                supports_networking: true,
                supports_volumes: true,
                supports_resource_limits: true,
                supports_health_checks: true,
                supports_rolling_updates: false,
                supports_load_balancing: false,
                supports_service_discovery: false,
                supports_secrets_management: true,
                max_cpu_cores: Some(64),
                max_memory_gb: Some(256),
            },
        });
        
        // Create a test profile
        let profile = DeploymentProfile {
            id: uuid::Uuid::new_v4(),
            name: "test-app".to_string(),
            software_name: "Test App".to_string(),
            repository: crate::web_types::GitHubRepository {
                id: 1,
                name: "test".to_string(),
                full_name: "user/test".to_string(),
                description: None,
                html_url: "https://github.com/user/test".to_string(),
                clone_url: "https://github.com/user/test.git".to_string(),
                ssh_url: "git@github.com:user/test.git".to_string(),
                language: Some("Python".to_string()),
                languages_url: "".to_string(),
                stargazers_count: 100,
                forks_count: 10,
                open_issues_count: 5,
                topics: vec![],
                license: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                default_branch: "main".to_string(),
                exposed_port: Some(8080),
                health_check_path: Some("/health".to_string()),
            },
            system_requirements: SystemRequirements {
                min_memory_mb: 512,
                min_cpu_cores: 1,
                min_disk_gb: 1,
                required_architectures: vec!["x86_64".to_string()],
                required_os: vec!["linux".to_string()],
                required_container_runtime: vec!["podman".to_string(), "docker".to_string()],
                network_requirements: vec!["tcp".to_string()],
            },
            container_config: ContainerConfig {
                image: "nginx".to_string(),
                tag: "latest".to_string(),
                ports: vec![],
                volumes: vec![],
                environment_variables: HashMap::new(),
                resource_limits: ResourceLimits {
                    memory_mb: Some(1024),
                    cpu_shares: Some(1024),
                    cpu_quota: None,
                    cpu_period: None,
                },
                health_check: None,
            },
            optimizations: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let recommended = manager.recommend_runtime(&profile);
        assert!(recommended.is_some());
        // Podman should be preferred due to higher base score
        assert_eq!(recommended.unwrap(), RuntimeType::Podman);
    }
}