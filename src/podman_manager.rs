//! Podman container management for deployment execution

use crate::web_types::*;
use crate::Result;
use log::{debug, info, warn, error};
use std::process::{Command, Stdio};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use serde_json;

/// Manages Podman containers for deployments
pub struct PodmanManager {
    podman_command: String,
}

impl PodmanManager {
    /// Create a new Podman manager
    pub fn new() -> Self {
        Self {
            podman_command: "podman".to_string(),
        }
    }

    /// Check if Podman is available and working
    pub async fn check_availability(&self) -> Result<bool> {
        debug!("Checking Podman availability");
        
        let output = Command::new(&self.podman_command)
            .args(["--version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("Podman available: {}", version.trim());
            Ok(true)
        } else {
            warn!("Podman not available or not working");
            Ok(false)
        }
    }

    /// Deploy a container based on deployment profile
    pub async fn deploy(&self, profile: &DeploymentProfile, request: &CreateDeploymentRequest) -> Result<CreateDeploymentResponse> {
        info!("Deploying container for profile: {}", profile.name);
        
        let deployment_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Validate Podman availability
        if !self.check_availability().await? {
            return Err(anyhow::anyhow!("Podman is not available"));
        }
        
        // Build container run command
        let mut command_args = vec!["run".to_string(), "-d".to_string()];
        
        // Add name
        command_args.push("--name".to_string());
        command_args.push(request.name.clone());
        
        // Add port mappings
        for port in &profile.container_config.ports {
            command_args.push("-p".to_string());
            command_args.push(format!("{}:{}/{}", port.host_port, port.container_port, port.protocol));
        }
        
        // Add volume mappings
        for volume in &profile.container_config.volumes {
            // Create host directory if it doesn't exist
            if let Some(parent) = std::path::Path::new(&volume.host_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            command_args.push("-v".to_string());
            let volume_spec = if volume.read_only {
                format!("{}:{}:ro", volume.host_path, volume.container_path)
            } else {
                format!("{}:{}", volume.host_path, volume.container_path)
            };
            command_args.push(volume_spec);
        }
        
        // Add environment variables
        for (key, value) in &profile.container_config.environment_variables {
            command_args.push("-e".to_string());
            command_args.push(format!("{}={}", key, value));
        }
        
        // Add custom environment variables from request
        if let Some(custom_config) = &request.custom_config {
            for (key, value) in custom_config {
                command_args.push("-e".to_string());
                command_args.push(format!("{}={}", key, value));
            }
        }
        
        // Add resource limits
        if let Some(memory_mb) = profile.container_config.resource_limits.memory_mb {
            command_args.push("--memory".to_string());
            command_args.push(format!("{}m", memory_mb));
        }
        
        if let Some(cpu_shares) = profile.container_config.resource_limits.cpu_shares {
            command_args.push("--cpu-shares".to_string());
            command_args.push(cpu_shares.to_string());
        }
        
        // Apply optimizations
        let mut warnings = Vec::new();
        for optimization in &profile.optimizations {
            match self.apply_optimization(&mut command_args, optimization) {
                Ok(applied) => {
                    if applied {
                        debug!("Applied optimization: {}", optimization.name);
                    }
                }
                Err(e) => {
                    warnings.push(format!("Failed to apply optimization {}: {}", optimization.name, e));
                }
            }
        }
        
        // Add image
        command_args.push(format!("{}:{}", profile.container_config.image, profile.container_config.tag));
        
        debug!("Running Podman command: {} {}", self.podman_command, command_args.join(" "));
        
        // Execute container run
        let output = Command::new(&self.podman_command)
            .args(&command_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start container: {}", error_msg);
            return Err(anyhow::anyhow!("Container deployment failed: {}", error_msg));
        }
        
        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Container deployed successfully: {}", container_id);
        
        // Get allocated ports
        let ports = self.get_container_ports(&container_id).await.unwrap_or_default();
        
        // Create deployment instance
        let deployment = DeploymentInstance {
            id: deployment_id,
            profile_id: profile.id,
            name: request.name.clone(),
            status: DeploymentStatus::Running,
            container_id: Some(container_id),
            ports,
            created_at: now,
            updated_at: now,
            logs: vec![
                DeploymentLog {
                    timestamp: now,
                    level: LogLevel::Info,
                    message: "Container deployment started".to_string(),
                    source: "podman_manager".to_string(),
                }
            ],
        };
        
        Ok(CreateDeploymentResponse {
            deployment,
            warnings,
        })
    }

    /// Stop and remove a deployed container
    pub async fn undeploy(&self, deployment: &DeploymentInstance) -> Result<()> {
        info!("Undeploying container: {}", deployment.name);
        
        if let Some(container_id) = &deployment.container_id {
            // Stop container
            let stop_output = Command::new(&self.podman_command)
                .args(["stop", container_id])
                .output()?;
                
            if !stop_output.status.success() {
                warn!("Failed to stop container {}: {}", container_id, String::from_utf8_lossy(&stop_output.stderr));
            }
            
            // Remove container
            let rm_output = Command::new(&self.podman_command)
                .args(["rm", container_id])
                .output()?;
                
            if !rm_output.status.success() {
                warn!("Failed to remove container {}: {}", container_id, String::from_utf8_lossy(&rm_output.stderr));
            } else {
                info!("Container {} removed successfully", container_id);
            }
        }
        
        Ok(())
    }

    /// Get the status of a deployed container
    pub async fn get_deployment_status(&self, deployment: &DeploymentInstance) -> Result<DeploymentStatus> {
        if let Some(container_id) = &deployment.container_id {
            let output = Command::new(&self.podman_command)
                .args(["inspect", container_id, "--format", "{{.State.Status}}"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
                
            if output.status.success() {
                let status_string = String::from_utf8_lossy(&output.stdout);
                let status = status_string.trim();
                match status {
                    "running" => Ok(DeploymentStatus::Running),
                    "exited" => Ok(DeploymentStatus::Stopped),
                    "created" => Ok(DeploymentStatus::Creating),
                    _ => Ok(DeploymentStatus::Failed),
                }
            } else {
                Ok(DeploymentStatus::Failed)
            }
        } else {
            Ok(DeploymentStatus::Failed)
        }
    }

    /// Get logs from a deployed container
    pub async fn get_container_logs(&self, deployment: &DeploymentInstance, tail_lines: u32) -> Result<Vec<DeploymentLog>> {
        if let Some(container_id) = &deployment.container_id {
            let output = Command::new(&self.podman_command)
                .args(["logs", "--tail", &tail_lines.to_string(), "--timestamps", container_id])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
                
            if output.status.success() {
                let log_text = String::from_utf8_lossy(&output.stdout);
                let mut logs = Vec::new();
                
                for line in log_text.lines() {
                    if let Some(log) = self.parse_log_line(line) {
                        logs.push(log);
                    }
                }
                
                Ok(logs)
            } else {
                Ok(Vec::new())
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// List all active deployments managed by this instance
    pub async fn list_deployments(&self) -> Result<Vec<HashMap<String, String>>> {
        let output = Command::new(&self.podman_command)
            .args(["ps", "--format", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if output.status.success() {
            let output_text = String::from_utf8_lossy(&output.stdout);
            let containers: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&output_text)
                .unwrap_or_default();
                
            let deployments = containers.into_iter()
                .map(|container| {
                    let mut deployment = HashMap::new();
                    for (key, value) in container {
                        deployment.insert(key, value.to_string().trim_matches('"').to_string());
                    }
                    deployment
                })
                .collect();
                
            Ok(deployments)
        } else {
            Ok(Vec::new())
        }
    }

    /// Restart a deployed container
    pub async fn restart_deployment(&self, deployment: &DeploymentInstance) -> Result<()> {
        info!("Restarting container: {}", deployment.name);
        
        if let Some(container_id) = &deployment.container_id {
            let output = Command::new(&self.podman_command)
                .args(["restart", container_id])
                .output()?;
                
            if output.status.success() {
                info!("Container {} restarted successfully", container_id);
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to restart container: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("No container ID available"))
        }
    }

    /// Update container resource limits
    pub async fn update_resources(&self, deployment: &DeploymentInstance, limits: &ResourceLimits) -> Result<()> {
        info!("Updating resources for container: {}", deployment.name);
        
        if let Some(container_id) = &deployment.container_id {
            let mut args = vec!["update".to_string()];
            
            if let Some(memory_mb) = limits.memory_mb {
                args.push("--memory".to_string());
                args.push(format!("{}m", memory_mb));
            }
            
            if let Some(cpu_shares) = limits.cpu_shares {
                args.push("--cpu-shares".to_string());
                args.push(cpu_shares.to_string());
            }
            
            args.push(container_id.clone());
            
            let output = Command::new(&self.podman_command)
                .args(&args)
                .output()?;
                
            if output.status.success() {
                info!("Container resources updated successfully");
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to update resources: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("No container ID available"))
        }
    }

    /// Apply optimization to command arguments
    fn apply_optimization(&self, command_args: &mut Vec<String>, optimization: &Optimization) -> Result<bool> {
        match optimization.optimization_type {
            OptimizationType::SecurityHardening => {
                if let Some(read_only) = optimization.parameters.get("read_only_root") {
                    if read_only == "true" {
                        command_args.push("--read-only".to_string());
                    }
                }
                
                if let Some(no_new_privs) = optimization.parameters.get("no_new_privileges") {
                    if no_new_privs == "true" {
                        command_args.push("--security-opt".to_string());
                        command_args.push("no-new-privileges".to_string());
                    }
                }
                
                if let Some(user) = optimization.parameters.get("user") {
                    command_args.push("--user".to_string());
                    command_args.push(user.clone());
                }
                
                Ok(true)
            }
            OptimizationType::NetworkOptimization => {
                if let Some(network_mode) = optimization.parameters.get("network_mode") {
                    command_args.push("--network".to_string());
                    command_args.push(network_mode.clone());
                }
                
                Ok(true)
            }
            OptimizationType::MemoryOptimization => {
                if let Some(swappiness) = optimization.parameters.get("swappiness") {
                    command_args.push("--sysctl".to_string());
                    command_args.push(format!("vm.swappiness={}", swappiness));
                }
                
                Ok(true)
            }
            OptimizationType::CpuOptimization => {
                if let Some(cpu_quota) = optimization.parameters.get("cpu_quota") {
                    command_args.push("--cpu-quota".to_string());
                    command_args.push(cpu_quota.clone());
                }
                
                if let Some(cpu_period) = optimization.parameters.get("cpu_period") {
                    command_args.push("--cpu-period".to_string());
                    command_args.push(cpu_period.clone());
                }
                
                Ok(true)
            }
            _ => Ok(false), // Other optimizations not implemented yet
        }
    }

    /// Get ports exposed by a container
    async fn get_container_ports(&self, container_id: &str) -> Result<Vec<u16>> {
        let output = Command::new(&self.podman_command)
            .args(["port", container_id])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if output.status.success() {
            let port_text = String::from_utf8_lossy(&output.stdout);
            let ports = port_text.lines()
                .filter_map(|line| {
                    // Parse format: "8080/tcp -> 0.0.0.0:8080"
                    if let Some(arrow_pos) = line.find(" -> ") {
                        let port_part = &line[arrow_pos + 4..];
                        if let Some(colon_pos) = port_part.rfind(':') {
                            let port_str = &port_part[colon_pos + 1..];
                            port_str.parse::<u16>().ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
                
            Ok(ports)
        } else {
            Ok(Vec::new())
        }
    }

    /// Parse a log line from Podman logs
    fn parse_log_line(&self, line: &str) -> Option<DeploymentLog> {
        // Try to parse a timestamp at the start of the line (RFC3339)
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        // Find the first space, but don't assume it's always there
        if let Some(space_pos) = trimmed.find(' ') {
            let timestamp_str = &trimmed[..space_pos];
            let message = &trimmed[space_pos + 1..];
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                return Some(DeploymentLog {
                    timestamp: timestamp.with_timezone(&Utc),
                    level: LogLevel::Info, // Default to Info, could be enhanced
                    message: message.to_string(),
                    source: "container".to_string(),
                });
            }
        }
        // Fallback: treat the whole line as the message, use current time
        Some(DeploymentLog {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: trimmed.to_string(),
            source: "container".to_string(),
        })
    }
}

impl Default for PodmanManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_podman_manager_creation() {
        let manager = PodmanManager::new();
        assert_eq!(manager.podman_command, "podman");
    }

    #[test]
    fn test_apply_security_optimization() {
        let manager = PodmanManager::new();
        let mut command_args = vec!["run".to_string(), "-d".to_string()];
        
        let optimization = Optimization {
            name: "Security".to_string(),
            description: "Security hardening".to_string(),
            optimization_type: OptimizationType::SecurityHardening,
            parameters: {
                let mut params = HashMap::new();
                params.insert("read_only_root".to_string(), "true".to_string());
                params.insert("no_new_privileges".to_string(), "true".to_string());
                params.insert("user".to_string(), "1000:1000".to_string());
                params
            },
        };
        
        let result = manager.apply_optimization(&mut command_args, &optimization);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Check that security options were added
        assert!(command_args.contains(&"--read-only".to_string()));
        assert!(command_args.contains(&"--security-opt".to_string()));
        assert!(command_args.contains(&"--user".to_string()));
        assert!(command_args.contains(&"1000:1000".to_string()));
    }

    #[test]
    fn test_parse_log_line() {
        let manager = PodmanManager::new();
        
        let log_line = "2023-12-25T12:00:00.000000000Z This is a test log message";
        let parsed = manager.parse_log_line(log_line);
        
        assert!(parsed.is_some());
        let log = parsed.unwrap();
        assert_eq!(log.message, "This is a test log message");
        assert_eq!(log.source, "container");
        assert!(matches!(log.level, LogLevel::Info));
    }

    #[test]
    fn test_parse_log_line_no_timestamp() {
        let manager = PodmanManager::new();
        
        let log_line = "Simple log message without timestamp";
        let parsed = manager.parse_log_line(log_line);
        
        assert!(parsed.is_some());
        let log = parsed.unwrap();
        assert_eq!(log.message, "Simple log message without timestamp");
    }

    #[tokio::test]
    async fn test_deployment_status_mapping() {
        let manager = PodmanManager::new();
        
        // Test deployment instance
        let deployment = DeploymentInstance {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            name: "test-deployment".to_string(),
            status: DeploymentStatus::Running,
            container_id: Some("test-container-id".to_string()),
            ports: vec![8080],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            logs: vec![],
        };
        
        // Note: This test will fail if Podman is not available, but demonstrates the interface
        let result = manager.get_deployment_status(&deployment).await;
        // Don't assert success since Podman may not be available in test environment
        assert!(result.is_ok() || result.is_err());
    }
}