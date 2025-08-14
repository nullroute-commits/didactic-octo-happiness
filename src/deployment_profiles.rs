//! Deployment profile management for generating optimized container configurations

use crate::web_types::*;
use crate::system_profiler::SystemProfiler;
use crate::Result;
use uuid::Uuid;
use chrono::Utc;
use log::{debug, info};
use std::collections::HashMap;

/// Manages deployment profiles for open source software
pub struct DeploymentProfileManager {
    system_profiler: SystemProfiler,
}

impl DeploymentProfileManager {
    /// Create a new deployment profile manager
    pub fn new(script_path: String) -> Self {
        Self {
            system_profiler: SystemProfiler::new(script_path),
        }
    }

    /// Generate a deployment profile for a GitHub repository
    pub async fn generate_profile(
        &self,
        repository: GitHubRepository,
        system_profile: SystemProfile,
        custom_requirements: Option<SystemRequirements>,
    ) -> Result<GenerateProfileResponse> {
        info!("Generating deployment profile for {}", repository.full_name);
        
        let profile_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Analyze repository to determine requirements
        let base_requirements = self.analyze_repository_requirements(&repository);
        
        // Merge with custom requirements if provided
        let requirements = if let Some(custom) = custom_requirements {
            self.merge_requirements(base_requirements, custom)
        } else {
            base_requirements
        };
        
        // Calculate compatibility score
        let compatibility_score = self.system_profiler.calculate_compatibility(&system_profile, &requirements);
        
        // Generate container configuration
        let container_config = self.generate_container_config(&repository, &system_profile, &requirements)?;
        
        // Generate optimizations
        let optimizations = self.generate_optimizations(&repository, &system_profile, &requirements);
        
        // Generate warnings and recommendations
        let warnings = self.generate_warnings(&system_profile, &requirements, compatibility_score);
        let recommendations = self.system_profiler.generate_recommendations(&system_profile);
        
        let profile = DeploymentProfile {
            id: profile_id,
            name: format!("{}-deployment", repository.name),
            software_name: repository.name.clone(),
            repository,
            system_requirements: requirements,
            container_config,
            optimizations,
            created_at: now,
            updated_at: now,
        };
        
        info!("Generated profile {} with compatibility score {:.2}", 
              profile.name, compatibility_score);
        
        Ok(GenerateProfileResponse {
            profile,
            compatibility_score,
            warnings,
            recommendations,
        })
    }

    /// Analyze repository to determine system requirements
    fn analyze_repository_requirements(&self, repository: &GitHubRepository) -> SystemRequirements {
        debug!("Analyzing repository requirements for {}", repository.full_name);
        
        let mut min_memory_mb = 512; // Default
        let mut min_cpu_cores = 1;   // Default
        let mut min_disk_gb = 5;     // Default
        let required_architectures = vec!["x86_64".to_string(), "arm64".to_string()]; // Common architectures
        let required_os = vec!["linux".to_string()]; // Default to Linux
        let required_container_runtime = vec!["docker".to_string(), "podman".to_string()];
        let mut network_requirements = Vec::new();
        
        // Analyze based on language
        if let Some(language) = &repository.language {
            match language.to_lowercase().as_str() {
                "javascript" | "typescript" => {
                    min_memory_mb = 1024;
                    min_cpu_cores = 2;
                    min_disk_gb = 10;
                    network_requirements.push("http".to_string());
                }
                "python" => {
                    min_memory_mb = 768;
                    min_cpu_cores = 1;
                    min_disk_gb = 8;
                }
                "java" | "kotlin" | "scala" => {
                    min_memory_mb = 2048;
                    min_cpu_cores = 2;
                    min_disk_gb = 15;
                }
                "rust" | "go" => {
                    min_memory_mb = 512;
                    min_cpu_cores = 1;
                    min_disk_gb = 5;
                }
                "c++" | "c" => {
                    min_memory_mb = 256;
                    min_cpu_cores = 1;
                    min_disk_gb = 3;
                }
                _ => {}
            }
        }
        
        // Analyze based on topics
        for topic in &repository.topics {
            match topic.to_lowercase().as_str() {
                "database" | "sql" | "nosql" => {
                    min_memory_mb = std::cmp::max(min_memory_mb, 2048);
                    min_disk_gb = std::cmp::max(min_disk_gb, 20);
                    network_requirements.push("tcp".to_string());
                }
                "web" | "api" | "server" => {
                    network_requirements.push("http".to_string());
                    network_requirements.push("https".to_string());
                }
                "machine-learning" | "ai" | "deep-learning" => {
                    min_memory_mb = std::cmp::max(min_memory_mb, 4096);
                    min_cpu_cores = std::cmp::max(min_cpu_cores, 4);
                    min_disk_gb = std::cmp::max(min_disk_gb, 50);
                }
                "blockchain" | "cryptocurrency" => {
                    min_memory_mb = std::cmp::max(min_memory_mb, 8192);
                    min_cpu_cores = std::cmp::max(min_cpu_cores, 4);
                    min_disk_gb = std::cmp::max(min_disk_gb, 100);
                }
                "game" | "gaming" => {
                    min_memory_mb = std::cmp::max(min_memory_mb, 2048);
                    min_cpu_cores = std::cmp::max(min_cpu_cores, 2);
                }
                "monitoring" | "metrics" => {
                    min_memory_mb = std::cmp::max(min_memory_mb, 1024);
                    network_requirements.push("http".to_string());
                }
                _ => {}
            }
        }
        
        // Analyze based on repository size/popularity
        if repository.stargazers_count > 10000 {
            // Popular projects might be more resource intensive
            min_memory_mb = std::cmp::max(min_memory_mb, 1024);
        }
        
        SystemRequirements {
            min_memory_mb,
            min_cpu_cores,
            min_disk_gb,
            required_architectures,
            required_os,
            required_container_runtime,
            network_requirements,
        }
    }

    /// Merge base requirements with custom requirements
    fn merge_requirements(&self, base: SystemRequirements, custom: SystemRequirements) -> SystemRequirements {
        SystemRequirements {
            min_memory_mb: std::cmp::max(base.min_memory_mb, custom.min_memory_mb),
            min_cpu_cores: std::cmp::max(base.min_cpu_cores, custom.min_cpu_cores),
            min_disk_gb: std::cmp::max(base.min_disk_gb, custom.min_disk_gb),
            required_architectures: if custom.required_architectures.is_empty() {
                base.required_architectures
            } else {
                custom.required_architectures
            },
            required_os: if custom.required_os.is_empty() {
                base.required_os
            } else {
                custom.required_os
            },
            required_container_runtime: if custom.required_container_runtime.is_empty() {
                base.required_container_runtime
            } else {
                custom.required_container_runtime
            },
            network_requirements: {
                let mut combined = base.network_requirements;
                combined.extend(custom.network_requirements);
                combined.sort();
                combined.dedup();
                combined
            },
        }
    }

    /// Generate container configuration
    fn generate_container_config(
        &self,
        repository: &GitHubRepository,
        system_profile: &SystemProfile,
        requirements: &SystemRequirements,
    ) -> Result<ContainerConfig> {
        debug!("Generating container config for {}", repository.name);
        
        // Determine base image
        let image = self.determine_base_image(repository, system_profile);
        let tag = "latest".to_string();
        
        // Generate port mappings
        let ports = self.generate_port_mappings(repository, requirements);
        
        // Generate volume mappings
        let volumes = self.generate_volume_mappings(repository);
        
        // Generate environment variables
        let environment_variables = self.generate_environment_variables(repository);
        
        // Generate resource limits
        let resource_limits = self.generate_resource_limits(system_profile, requirements);
        
        // Generate health check
        let health_check = self.generate_health_check(repository);
        
        Ok(ContainerConfig {
            image,
            tag,
            ports,
            volumes,
            environment_variables,
            resource_limits,
            health_check,
        })
    }

    /// Determine appropriate base image
    fn determine_base_image(&self, repository: &GitHubRepository, _system_profile: &SystemProfile) -> String {
        if let Some(language) = &repository.language {
            match language.to_lowercase().as_str() {
                "javascript" | "typescript" => "node:18-alpine".to_string(),
                "python" => "python:3.11-alpine".to_string(),
                "java" => "openjdk:17-alpine".to_string(),
                "rust" => "rust:1.70-alpine".to_string(),
                "go" => "golang:1.20-alpine".to_string(),
                "php" => "php:8.2-apache".to_string(),
                "ruby" => "ruby:3.2-alpine".to_string(),
                _ => "alpine:latest".to_string(),
            }
        } else {
            "alpine:latest".to_string()
        }
    }

    /// Generate port mappings based on repository analysis
    fn generate_port_mappings(&self, repository: &GitHubRepository, requirements: &SystemRequirements) -> Vec<PortMapping> {
        let mut ports = Vec::new();
        
        // Default ports based on language/framework
        if let Some(language) = &repository.language {
            match language.to_lowercase().as_str() {
                "javascript" | "typescript" => {
                    ports.push(PortMapping {
                        host_port: 3000,
                        container_port: 3000,
                        protocol: "tcp".to_string(),
                    });
                }
                "python" => {
                    if repository.topics.iter().any(|t| t.contains("django")) {
                        ports.push(PortMapping {
                            host_port: 8000,
                            container_port: 8000,
                            protocol: "tcp".to_string(),
                        });
                    } else if repository.topics.iter().any(|t| t.contains("flask")) {
                        ports.push(PortMapping {
                            host_port: 5000,
                            container_port: 5000,
                            protocol: "tcp".to_string(),
                        });
                    }
                }
                "java" => {
                    ports.push(PortMapping {
                        host_port: 8080,
                        container_port: 8080,
                        protocol: "tcp".to_string(),
                    });
                }
                _ => {}
            }
        }
        
        // Add ports based on requirements
        if requirements.network_requirements.contains(&"http".to_string()) && ports.is_empty() {
            ports.push(PortMapping {
                host_port: 8080,
                container_port: 8080,
                protocol: "tcp".to_string(),
            });
        }
        
        ports
    }

    /// Generate volume mappings
    fn generate_volume_mappings(&self, repository: &GitHubRepository) -> Vec<VolumeMapping> {
        let mut volumes = Vec::new();
        
        // Common data directory
        volumes.push(VolumeMapping {
            host_path: format!("./data/{}", repository.name),
            container_path: "/app/data".to_string(),
            read_only: false,
        });
        
        // Configuration directory
        volumes.push(VolumeMapping {
            host_path: format!("./config/{}", repository.name),
            container_path: "/app/config".to_string(),
            read_only: true,
        });
        
        // Add database volume if needed
        if repository.topics.iter().any(|t| t.contains("database")) {
            volumes.push(VolumeMapping {
                host_path: format!("./db/{}", repository.name),
                container_path: "/var/lib/database".to_string(),
                read_only: false,
            });
        }
        
        volumes
    }

    /// Generate environment variables
    fn generate_environment_variables(&self, repository: &GitHubRepository) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        env.insert("APP_NAME".to_string(), repository.name.clone());
        env.insert("APP_ENV".to_string(), "production".to_string());
        
        if let Some(language) = &repository.language {
            env.insert("LANGUAGE".to_string(), language.clone());
        }
        
        // Add language-specific environment variables
        if let Some(language) = &repository.language {
            match language.to_lowercase().as_str() {
                "javascript" | "typescript" => {
                    env.insert("NODE_ENV".to_string(), "production".to_string());
                }
                "python" => {
                    env.insert("PYTHONUNBUFFERED".to_string(), "1".to_string());
                }
                "java" => {
                    env.insert("JAVA_OPTS".to_string(), "-Xmx512m".to_string());
                }
                _ => {}
            }
        }
        
        env
    }

    /// Generate resource limits based on system capabilities
    fn generate_resource_limits(&self, system_profile: &SystemProfile, requirements: &SystemRequirements) -> ResourceLimits {
        // Conservative limits based on system capacity
        let memory_mb = Some(std::cmp::min(
            requirements.min_memory_mb * 2,
            system_profile.memory_available_mb / 2,
        ));
        
        let cpu_shares = if system_profile.cpu_cores >= 4 {
            Some(1024) // Full share
        } else {
            Some(512) // Half share for limited systems
        };
        
        ResourceLimits {
            memory_mb,
            cpu_shares,
            cpu_quota: None,
            cpu_period: None,
        }
    }

    /// Generate health check configuration
    fn generate_health_check(&self, repository: &GitHubRepository) -> Option<HealthCheck> {
        // Attempt to get port and health check path from repository metadata, fallback to defaults
        let port = repository
            .exposed_port
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "8080".to_string());
        let path = repository
            .health_check_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("/health");
        let url = format!("http://localhost:{}{}", port, path);
        Some(HealthCheck {
            test: vec!["CMD".to_string(), "curl".to_string(), "-f".to_string(), url],
            interval_seconds: 30,
            timeout_seconds: 10,
            retries: 3,
            start_period_seconds: 40,
        })
    }

    /// Generate optimizations based on system profile and requirements
    fn generate_optimizations(
        &self,
        _repository: &GitHubRepository,
        system_profile: &SystemProfile,
        requirements: &SystemRequirements,
    ) -> Vec<Optimization> {
        let mut optimizations = Vec::new();
        
        // Memory optimizations
        if system_profile.memory_total_mb < 2048 {
            optimizations.push(Optimization {
                name: "Memory Optimization".to_string(),
                description: "Reduce memory usage for low-memory systems".to_string(),
                optimization_type: OptimizationType::MemoryOptimization,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("swappiness".to_string(), "10".to_string());
                    params.insert("memory_limit".to_string(), format!("{}m", requirements.min_memory_mb));
                    params
                },
            });
        }
        
        // CPU optimizations
        if system_profile.cpu_cores <= 2 {
            optimizations.push(Optimization {
                name: "CPU Optimization".to_string(),
                description: "Optimize for limited CPU cores".to_string(),
                optimization_type: OptimizationType::CpuOptimization,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("cpu_shares".to_string(), "512".to_string());
                    params.insert("cpu_quota".to_string(), "50000".to_string());
                    params
                },
            });
        }
        
        // Security hardening
        optimizations.push(Optimization {
            name: "Security Hardening".to_string(),
            description: "Apply security best practices".to_string(),
            optimization_type: OptimizationType::SecurityHardening,
            parameters: {
                let mut params = HashMap::new();
                params.insert("read_only_root".to_string(), "true".to_string());
                params.insert("no_new_privileges".to_string(), "true".to_string());
                params.insert("user".to_string(), "1000:1000".to_string());
                params
            },
        });
        
        // Network optimizations
        if requirements.network_requirements.len() > 1 {
            optimizations.push(Optimization {
                name: "Network Optimization".to_string(),
                description: "Optimize network performance".to_string(),
                optimization_type: OptimizationType::NetworkOptimization,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("network_mode".to_string(), "host".to_string());
                    params
                },
            });
        }
        
        optimizations
    }

    /// Generate warnings based on compatibility analysis
    fn generate_warnings(&self, system_profile: &SystemProfile, requirements: &SystemRequirements, compatibility_score: f64) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if compatibility_score < 0.5 {
            warnings.push("Low compatibility score - deployment may not work properly".to_string());
        }
        
        if system_profile.memory_available_mb < requirements.min_memory_mb {
            warnings.push(format!(
                "Insufficient memory: available {}MB, required {}MB",
                system_profile.memory_available_mb, requirements.min_memory_mb
            ));
        }
        
        if system_profile.cpu_cores < requirements.min_cpu_cores {
            warnings.push(format!(
                "Insufficient CPU cores: available {}, required {}",
                system_profile.cpu_cores, requirements.min_cpu_cores
            ));
        }
        
        if system_profile.hardware_capabilities.disk_total_gb < requirements.min_disk_gb {
            warnings.push(format!(
                "Insufficient disk space: available {}GB, required {}GB",
                system_profile.hardware_capabilities.disk_total_gb, requirements.min_disk_gb
            ));
        }
        
        if !requirements.required_architectures.contains(&system_profile.architecture) {
            warnings.push(format!(
                "Architecture mismatch: system {}, required {:?}",
                system_profile.architecture, requirements.required_architectures
            ));
        }
        
        let runtime_available = requirements.required_container_runtime.iter().any(|req_runtime| {
            system_profile.container_runtimes.iter().any(|available| {
                available.to_lowercase().contains(&req_runtime.to_lowercase())
            })
        });
        
        if !runtime_available {
            warnings.push(format!(
                "No compatible container runtime found. Required: {:?}, Available: {:?}",
                requirements.required_container_runtime, system_profile.container_runtimes
            ));
        }
        
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_deployment_profile_manager_creation() {
        let _manager = DeploymentProfileManager::new("./collect_info.sh".to_string());
        // We can't access private fields, so just verify construction succeeds
        assert!(true);
    }

    #[test]
    fn test_analyze_repository_requirements_javascript() {
        let manager = DeploymentProfileManager::new("test".to_string());
        
        let repository = GitHubRepository {
            id: 1,
            name: "test-app".to_string(),
            full_name: "user/test-app".to_string(),
            description: None,
            html_url: "https://github.com/user/test-app".to_string(),
            clone_url: "https://github.com/user/test-app.git".to_string(),
            ssh_url: "git@github.com:user/test-app.git".to_string(),
            language: Some("JavaScript".to_string()),
            languages_url: "".to_string(),
            stargazers_count: 100,
            forks_count: 10,
            open_issues_count: 5,
            topics: vec!["web".to_string(), "api".to_string()],
            license: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            default_branch: "main".to_string(),
            exposed_port: Some(8080),
            health_check_path: Some("/health".to_string()),
        };
        
        let requirements = manager.analyze_repository_requirements(&repository);
        
        assert!(requirements.min_memory_mb >= 1024); // JavaScript apps need more memory
        assert!(requirements.min_cpu_cores >= 2);
        assert!(requirements.network_requirements.contains(&"http".to_string()));
    }

    #[test]
    fn test_determine_base_image() {
        let manager = DeploymentProfileManager::new("test".to_string());
        
        let system_profile = SystemProfile {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            architecture: "x86_64".to_string(),
            os_name: "Ubuntu".to_string(),
            os_version: "20.04".to_string(),
            kernel_version: "5.4.0".to_string(),
            cpu_model: "Intel Core".to_string(),
            cpu_cores: 4,
            memory_total_mb: 8192,
            memory_available_mb: 6144,
            virtualization_type: None,
            container_runtimes: vec!["docker".to_string()],
            network_interfaces: vec![],
            hardware_capabilities: HardwareCapabilities {
                has_gpu: false,
                gpu_vendor: None,
                pcie_devices: 5,
                usb_devices: 3,
                network_devices: 1,
                disk_total_gb: 100,
                virtualization_support: true,
            },
        };
        
        let repository = GitHubRepository {
            id: 1,
            name: "test-app".to_string(),
            full_name: "user/test-app".to_string(),
            description: None,
            html_url: "https://github.com/user/test-app".to_string(),
            clone_url: "https://github.com/user/test-app.git".to_string(),
            ssh_url: "git@github.com:user/test-app.git".to_string(),
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
            exposed_port: Some(5000),
            health_check_path: Some("/api/health".to_string()),
        };
        
        let image = manager.determine_base_image(&repository, &system_profile);
        assert_eq!(image, "python:3.11-alpine");
    }

    #[test]
    fn test_generate_resource_limits() {
        let manager = DeploymentProfileManager::new("test".to_string());
        
        let system_profile = SystemProfile {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            architecture: "x86_64".to_string(),
            os_name: "Ubuntu".to_string(),
            os_version: "20.04".to_string(),
            kernel_version: "5.4.0".to_string(),
            cpu_model: "Intel Core".to_string(),
            cpu_cores: 2, // Limited cores
            memory_total_mb: 4096,
            memory_available_mb: 3072,
            virtualization_type: None,
            container_runtimes: vec!["docker".to_string()],
            network_interfaces: vec![],
            hardware_capabilities: HardwareCapabilities {
                has_gpu: false,
                gpu_vendor: None,
                pcie_devices: 5,
                usb_devices: 3,
                network_devices: 1,
                disk_total_gb: 100,
                virtualization_support: true,
            },
        };
        
        let requirements = SystemRequirements {
            min_memory_mb: 1024,
            min_cpu_cores: 1,
            min_disk_gb: 10,
            required_architectures: vec!["x86_64".to_string()],
            required_os: vec!["linux".to_string()],
            required_container_runtime: vec!["docker".to_string()],
            network_requirements: vec![],
        };
        
        let limits = manager.generate_resource_limits(&system_profile, &requirements);
        
        assert!(limits.memory_mb.is_some());
        assert!(limits.cpu_shares.is_some());
        // Should use half shares for limited system
        assert_eq!(limits.cpu_shares, Some(512));
    }
}