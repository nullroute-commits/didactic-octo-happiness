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
}

impl std::fmt::Display for RuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeType::Podman => write!(f, "podman"),
            RuntimeType::Docker => write!(f, "docker"),
            RuntimeType::Lxc => write!(f, "lxc"),
            RuntimeType::Containerd => write!(f, "containerd"),
        }
    }
}

/// Runtime availability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub runtime_type: RuntimeType,
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Container runtime capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    pub supports_rootless: bool,
    pub supports_pods: bool,
    pub supports_volumes: bool,
    pub supports_networks: bool,
    pub supports_healthchecks: bool,
    pub supports_resource_limits: bool,
    pub native_security_features: Vec<String>,
}

/// Unified container runtime manager
pub struct ContainerRuntimeManager {
    podman_manager: PodmanManager,
    docker_manager: DockerManager,
    lxc_manager: LxcManager,
    available_runtimes: HashMap<RuntimeType, RuntimeInfo>,
}

impl ContainerRuntimeManager {
    /// Create a new container runtime manager
    pub async fn new() -> Self {
        let mut manager = Self {
            podman_manager: PodmanManager::new(),
            docker_manager: DockerManager::new(),
            lxc_manager: LxcManager::new(),
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
        let podman_info = match self.podman_manager.check_availability().await {
            Ok(available) => RuntimeInfo {
                runtime_type: RuntimeType::Podman,
                available,
                version: if available { Some("detected".to_string()) } else { None },
                error: None,
            },
            Err(e) => RuntimeInfo {
                runtime_type: RuntimeType::Podman,
                available: false,
                version: None,
                error: Some(e.to_string()),
            },
        };
        self.available_runtimes.insert(RuntimeType::Podman, podman_info);

        // Check Docker
        let docker_info = match self.docker_manager.check_availability().await {
            Ok(available) => RuntimeInfo {
                runtime_type: RuntimeType::Docker,
                available,
                version: if available { Some("detected".to_string()) } else { None },
                error: None,
            },
            Err(e) => RuntimeInfo {
                runtime_type: RuntimeType::Docker,
                available: false,
                version: None,
                error: Some(e.to_string()),
            },
        };
        self.available_runtimes.insert(RuntimeType::Docker, docker_info);

        // Check LXC
        let lxc_info = match self.lxc_manager.check_availability().await {
            Ok(available) => RuntimeInfo {
                runtime_type: RuntimeType::Lxc,
                available,
                version: if available { Some("detected".to_string()) } else { None },
                error: None,
            },
            Err(e) => RuntimeInfo {
                runtime_type: RuntimeType::Lxc,
                available: false,
                version: None,
                error: Some(e.to_string()),
            },
        };
        self.available_runtimes.insert(RuntimeType::Lxc, lxc_info);

        // Containerd would go here when implemented
        self.available_runtimes.insert(RuntimeType::Containerd, RuntimeInfo {
            runtime_type: RuntimeType::Containerd,
            available: false,
            version: None,
            error: Some("Not yet implemented".to_string()),
        });

        let available_count = self.available_runtimes.values()
            .filter(|info| info.available)
            .count();
        info!("Detected {} available container runtimes", available_count);
    }

    /// Get information about all detected runtimes
    pub fn get_runtime_info(&self) -> &HashMap<RuntimeType, RuntimeInfo> {
        &self.available_runtimes
    }

    /// Get available runtime types
    pub fn get_available_runtimes(&self) -> Vec<RuntimeType> {
        self.available_runtimes.values()
            .filter(|info| info.available)
            .map(|info| info.runtime_type.clone())
            .collect()
    }

    /// Get the preferred runtime (first available in priority order)
    pub fn get_preferred_runtime(&self) -> Option<RuntimeType> {
        // Priority order: Podman > Docker > LXC > Containerd
        let priority_order = vec![
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
                return Err(anyhow::anyhow!("Runtime {} is not available: {:?}", runtime_type, info.error));
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
        }
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance) -> Result<DeploymentStatus> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.get_deployment_status(deployment).await,
            RuntimeType::Docker => self.docker_manager.get_deployment_status(deployment).await,
            RuntimeType::Lxc => self.lxc_manager.get_deployment_status(deployment).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
        }
    }

    /// Get container logs
    pub async fn get_container_logs(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance, tail_lines: u32) -> Result<Vec<DeploymentLog>> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.get_container_logs(deployment, tail_lines).await,
            RuntimeType::Docker => self.docker_manager.get_container_logs(deployment, tail_lines).await,
            RuntimeType::Lxc => self.lxc_manager.get_container_logs(deployment, tail_lines).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
        }
    }

    /// List deployments
    pub async fn list_deployments(&self, runtime_type: RuntimeType) -> Result<Vec<HashMap<String, String>>> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.list_deployments().await,
            RuntimeType::Docker => self.docker_manager.list_deployments().await,
            RuntimeType::Lxc => self.lxc_manager.list_deployments().await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
        }
    }

    /// Restart deployment
    pub async fn restart_deployment(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance) -> Result<()> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.restart_deployment(deployment).await,
            RuntimeType::Docker => self.docker_manager.restart_deployment(deployment).await,
            RuntimeType::Lxc => self.lxc_manager.restart_deployment(deployment).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
        }
    }

    /// Update resources
    pub async fn update_resources(&self, runtime_type: RuntimeType, deployment: &DeploymentInstance, limits: &ResourceLimits) -> Result<()> {
        match runtime_type {
            RuntimeType::Podman => self.podman_manager.update_resources(deployment, limits).await,
            RuntimeType::Docker => self.docker_manager.update_resources(deployment, limits).await,
            RuntimeType::Lxc => self.lxc_manager.update_resources(deployment, limits).await,
            RuntimeType::Containerd => Err(anyhow::anyhow!("Containerd runtime not yet implemented")),
        }
    }

    /// Get runtime capabilities
    pub fn get_runtime_capabilities(&self, runtime_type: RuntimeType) -> RuntimeCapabilities {
        match runtime_type {
            RuntimeType::Podman => RuntimeCapabilities {
                supports_rootless: true,
                supports_pods: true,
                supports_volumes: true,
                supports_networks: true,
                supports_healthchecks: true,
                supports_resource_limits: true,
                native_security_features: vec![
                    "SELinux integration".to_string(),
                    "Rootless mode".to_string(),
                    "User namespaces".to_string(),
                ],
            },
            RuntimeType::Docker => RuntimeCapabilities {
                supports_rootless: true, // Available in recent versions
                supports_pods: false,
                supports_volumes: true,
                supports_networks: true,
                supports_healthchecks: true,
                supports_resource_limits: true,
                native_security_features: vec![
                    "AppArmor profiles".to_string(),
                    "Seccomp profiles".to_string(),
                    "User namespaces".to_string(),
                ],
            },
            RuntimeType::Lxc => RuntimeCapabilities {
                supports_rootless: true,
                supports_pods: false,
                supports_volumes: true,
                supports_networks: true,
                supports_healthchecks: false,
                supports_resource_limits: true,
                native_security_features: vec![
                    "Mandatory Access Control".to_string(),
                    "Resource isolation".to_string(),
                    "Privilege dropping".to_string(),
                ],
            },
            RuntimeType::Containerd => RuntimeCapabilities {
                supports_rootless: true,
                supports_pods: false,
                supports_volumes: true,
                supports_networks: true,
                supports_healthchecks: false,
                supports_resource_limits: true,
                native_security_features: vec![
                    "gVisor integration".to_string(),
                    "Kata Containers".to_string(),
                ],
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
            }
            
            // Bonus for specific requirements
            if profile.system_requirements.required_architectures.len() > 1 && capabilities.supports_rootless {
                score += 3; // Multi-arch usually benefits from rootless
            }
            
            if profile.container_config.volumes.len() > 3 && capabilities.supports_volumes {
                score += 2; // Many volumes
            }
            
            if profile.container_config.ports.len() > 5 && capabilities.supports_networks {
                score += 2; // Many ports
            }
            
            if profile.optimizations.iter().any(|opt| matches!(opt.optimization_type, OptimizationType::SecurityHardening)) {
                if capabilities.supports_rootless {
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
        assert!(podman_caps.supports_rootless);
        assert!(podman_caps.supports_pods);
        
        let docker_caps = manager.get_runtime_capabilities(RuntimeType::Docker);
        assert!(docker_caps.supports_volumes);
        assert!(!docker_caps.supports_pods);
        
        let lxc_caps = manager.get_runtime_capabilities(RuntimeType::Lxc);
        assert!(lxc_caps.supports_rootless);
        assert!(!lxc_caps.supports_healthchecks);
    }

    #[test]
    fn test_runtime_recommendation() {
        let mut manager = ContainerRuntimeManager::default();
        
        // Mock available runtimes
        manager.available_runtimes.insert(RuntimeType::Podman, RuntimeInfo {
            runtime_type: RuntimeType::Podman,
            available: true,
            version: Some("4.0.0".to_string()),
            error: None,
        });
        manager.available_runtimes.insert(RuntimeType::Docker, RuntimeInfo {
            runtime_type: RuntimeType::Docker,
            available: true,
            version: Some("24.0.0".to_string()),
            error: None,
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