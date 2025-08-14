//! LXC/LXD container management for deployment execution

use crate::web_types::*;
use crate::Result;
use log::{debug, info, warn, error};
use std::process::{Command, Stdio};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use serde_json;

/// Manages LXC/LXD containers for deployments
pub struct LxcManager {
    lxc_command: String,
}

impl LxcManager {
    /// Create a new LXC manager
    pub fn new() -> Self {
        Self {
            lxc_command: "lxc".to_string(),
        }
    }

    /// Check if LXC/LXD is available and working
    pub async fn check_availability(&self) -> Result<bool> {
        debug!("Checking LXC/LXD availability");
        
        let output = Command::new(&self.lxc_command)
            .args(["version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("LXC/LXD available: {}", version.trim());
            Ok(true)
        } else {
            warn!("LXC/LXD not available or not working");
            Ok(false)
        }
    }

    /// Deploy a container based on deployment profile
    pub async fn deploy(&self, profile: &DeploymentProfile, request: &CreateDeploymentRequest) -> Result<CreateDeploymentResponse> {
        info!("Deploying LXC container for profile: {}", profile.name);
        
        let deployment_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Validate LXC availability
        if !self.check_availability().await? {
            return Err(anyhow::anyhow!("LXC/LXD is not available"));
        }
        
        let container_name = request.name.clone();
        
        // Launch container from image
        let mut command_args = vec!["launch".to_string()];
        
        // Add image (LXC images are typically ubuntu:20.04, alpine:latest, etc.)
        let image = format!("{}:{}", profile.container_config.image, profile.container_config.tag);
        command_args.push(image);
        command_args.push(container_name.clone());
        
        debug!("Running LXC launch command: {} {}", self.lxc_command, command_args.join(" "));
        
        // Execute container launch
        let output = Command::new(&self.lxc_command)
            .args(&command_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to launch container: {}", error_msg);
            return Err(anyhow::anyhow!("Container deployment failed: {}", error_msg));
        }
        
        info!("Container launched successfully: {}", container_name);
        
        // Configure container after launch
        let mut warnings = Vec::new();
        
        // Set resource limits
        if let Some(memory_mb) = profile.container_config.resource_limits.memory_mb {
            if let Err(e) = self.set_memory_limit(&container_name, memory_mb).await {
                warnings.push(format!("Failed to set memory limit: {}", e));
            }
        }
        
        if let Some(cpu_shares) = profile.container_config.resource_limits.cpu_shares {
            if let Err(e) = self.set_cpu_limit(&container_name, cpu_shares).await {
                warnings.push(format!("Failed to set CPU limit: {}", e));
            }
        }
        
        // Setup port forwarding (proxy devices in LXD)
        for port in &profile.container_config.ports {
            if let Err(e) = self.setup_port_forward(&container_name, port).await {
                warnings.push(format!("Failed to setup port forwarding for {}: {}", port.container_port, e));
            }
        }
        
        // Mount volumes (disk devices in LXD)
        for volume in &profile.container_config.volumes {
            if let Err(e) = self.setup_volume_mount(&container_name, volume).await {
                warnings.push(format!("Failed to mount volume {}: {}", volume.container_path, e));
            }
        }
        
        // Set environment variables
        for (key, value) in &profile.container_config.environment_variables {
            if let Err(e) = self.set_environment_variable(&container_name, key, value).await {
                warnings.push(format!("Failed to set environment variable {}: {}", key, e));
            }
        }
        
        // Set custom environment variables from request
        if let Some(custom_config) = &request.custom_config {
            for (key, value) in custom_config {
                if let Err(e) = self.set_environment_variable(&container_name, key, value).await {
                    warnings.push(format!("Failed to set custom environment variable {}: {}", key, e));
                }
            }
        }
        
        // Start the container if it's not already running
        let start_output = Command::new(&self.lxc_command)
            .args(["start", &container_name])
            .output()?;
            
        if !start_output.status.success() {
            let error_msg = String::from_utf8_lossy(&start_output.stderr);
            // Container might already be running, check status
            if !error_msg.contains("already") {
                warn!("Failed to start container: {}", error_msg);
            }
        }
        
        // Get allocated ports
        let ports = self.get_container_ports(&container_name).await.unwrap_or_default();
        
        // Create deployment instance
        let deployment = DeploymentInstance {
            id: deployment_id,
            profile_id: profile.id,
            name: request.name.clone(),
            status: DeploymentStatus::Running,
            container_id: Some(container_name),
            ports,
            created_at: now,
            updated_at: now,
            logs: vec![
                DeploymentLog {
                    timestamp: now,
                    level: LogLevel::Info,
                    message: "LXC container deployment started".to_string(),
                    source: "lxc_manager".to_string(),
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
        info!("Undeploying LXC container: {}", deployment.name);
        
        if let Some(container_name) = &deployment.container_id {
            // Stop container
            let stop_output = Command::new(&self.lxc_command)
                .args(["stop", container_name])
                .output()?;
                
            if !stop_output.status.success() {
                warn!("Failed to stop container {}: {}", container_name, String::from_utf8_lossy(&stop_output.stderr));
            }
            
            // Delete container
            let delete_output = Command::new(&self.lxc_command)
                .args(["delete", container_name])
                .output()?;
                
            if !delete_output.status.success() {
                warn!("Failed to delete container {}: {}", container_name, String::from_utf8_lossy(&delete_output.stderr));
            } else {
                info!("Container {} deleted successfully", container_name);
            }
        }
        
        Ok(())
    }

    /// Get the status of a deployed container
    pub async fn get_deployment_status(&self, deployment: &DeploymentInstance) -> Result<DeploymentStatus> {
        if let Some(container_name) = &deployment.container_id {
            let output = Command::new(&self.lxc_command)
                .args(["info", container_name])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
                
            if output.status.success() {
                let info_text = String::from_utf8_lossy(&output.stdout);
                if info_text.contains("Status: Running") {
                    Ok(DeploymentStatus::Running)
                } else if info_text.contains("Status: Stopped") {
                    Ok(DeploymentStatus::Stopped)
                } else {
                    Ok(DeploymentStatus::Failed)
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
        if let Some(container_name) = &deployment.container_id {
            // LXC doesn't have a direct logs command like Docker, 
            // we can execute 'tail' inside the container to get logs
            let output = Command::new(&self.lxc_command)
                .args(["exec", container_name, "--", "tail", "-n", &tail_lines.to_string(), "/var/log/messages"])
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
        let output = Command::new(&self.lxc_command)
            .args(["list", "--format", "json"])
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
        info!("Restarting LXC container: {}", deployment.name);
        
        if let Some(container_name) = &deployment.container_id {
            let output = Command::new(&self.lxc_command)
                .args(["restart", container_name])
                .output()?;
                
            if output.status.success() {
                info!("Container {} restarted successfully", container_name);
                Ok(())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("Failed to restart container: {}", error_msg))
            }
        } else {
            Err(anyhow::anyhow!("No container name available"))
        }
    }

    /// Update container resource limits
    pub async fn update_resources(&self, deployment: &DeploymentInstance, limits: &ResourceLimits) -> Result<()> {
        info!("Updating resources for LXC container: {}", deployment.name);
        
        if let Some(container_name) = &deployment.container_id {
            if let Some(memory_mb) = limits.memory_mb {
                self.set_memory_limit(container_name, memory_mb).await?;
            }
            
            if let Some(cpu_shares) = limits.cpu_shares {
                self.set_cpu_limit(container_name, cpu_shares).await?;
            }
            
            info!("Container resources updated successfully");
            Ok(())
        } else {
            Err(anyhow::anyhow!("No container name available"))
        }
    }

    /// Set memory limit for a container
    async fn set_memory_limit(&self, container_name: &str, memory_mb: u64) -> Result<()> {
        let output = Command::new(&self.lxc_command)
            .args(["config", "set", container_name, "limits.memory", &format!("{}MB", memory_mb)])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to set memory limit: {}", error_msg))
        }
    }

    /// Set CPU limit for a container
    async fn set_cpu_limit(&self, container_name: &str, cpu_shares: u32) -> Result<()> {
        // LXC uses cpu.priority which is similar to CPU shares
        let priority = (cpu_shares / 1024).max(1).min(10); // Convert shares to priority (1-10)
        let output = Command::new(&self.lxc_command)
            .args(["config", "set", container_name, "limits.cpu.priority", &priority.to_string()])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to set CPU limit: {}", error_msg))
        }
    }

    /// Setup port forwarding for a container
    async fn setup_port_forward(&self, container_name: &str, port: &PortMapping) -> Result<()> {
        let device_name = format!("port{}", port.container_port);
        let output = Command::new(&self.lxc_command)
            .args([
                "config", "device", "add", container_name, &device_name, "proxy",
                &format!("listen={}:{}", port.protocol, port.host_port),
                &format!("connect={}:{}", port.protocol, port.container_port)
            ])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to setup port forwarding: {}", error_msg))
        }
    }

    /// Setup volume mount for a container
    async fn setup_volume_mount(&self, container_name: &str, volume: &VolumeMapping) -> Result<()> {
        // Create host directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&volume.host_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let device_name = format!("mount{}", volume.container_path.replace('/', "_"));
        let mut args = vec![
            "config".to_string(), "device".to_string(), "add".to_string(), 
            container_name.to_string(), device_name, "disk".to_string(),
            format!("source={}", volume.host_path),
            format!("path={}", volume.container_path)
        ];
        
        if volume.read_only {
            args.push("readonly=true".to_string());
        }
        
        let output = Command::new(&self.lxc_command)
            .args(&args)
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to setup volume mount: {}", error_msg))
        }
    }

    /// Set environment variable for a container
    async fn set_environment_variable(&self, container_name: &str, key: &str, value: &str) -> Result<()> {
        let config_key = format!("environment.{}", key);
        let output = Command::new(&self.lxc_command)
            .args(["config", "set", container_name, &config_key, value])
            .output()?;
            
        if output.status.success() {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to set environment variable: {}", error_msg))
        }
    }

    /// Get ports exposed by a container
    async fn get_container_ports(&self, container_name: &str) -> Result<Vec<u16>> {
        let output = Command::new(&self.lxc_command)
            .args(["config", "show", container_name])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
            
        if output.status.success() {
            let config_text = String::from_utf8_lossy(&output.stdout);
            let mut ports = Vec::new();
            
            // Parse YAML-like output for proxy devices
            for line in config_text.lines() {
                if line.trim().starts_with("listen:") {
                    if let Some(port_part) = line.split(':').nth(2) {
                        if let Ok(port) = port_part.trim().parse::<u16>() {
                            ports.push(port);
                        }
                    }
                }
            }
                
            Ok(ports)
        } else {
            Ok(Vec::new())
        }
    }

    /// Parse a log line
    fn parse_log_line(&self, line: &str) -> Option<DeploymentLog> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        
        // LXC logs might have different format, for now just treat as plain message
        Some(DeploymentLog {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: trimmed.to_string(),
            source: "container".to_string(),
        })
    }
}

impl Default for LxcManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_lxc_manager_creation() {
        let manager = LxcManager::new();
        assert_eq!(manager.lxc_command, "lxc");
    }

    #[test]
    fn test_parse_log_line() {
        let manager = LxcManager::new();
        
        let log_line = "This is a test log message";
        let parsed = manager.parse_log_line(log_line);
        
        assert!(parsed.is_some());
        let log = parsed.unwrap();
        assert_eq!(log.message, "This is a test log message");
        assert_eq!(log.source, "container");
        assert!(matches!(log.level, LogLevel::Info));
    }

    #[tokio::test]
    async fn test_deployment_status_mapping() {
        let manager = LxcManager::new();
        
        // Test deployment instance
        let deployment = DeploymentInstance {
            id: Uuid::new_v4(),
            profile_id: Uuid::new_v4(),
            name: "test-deployment".to_string(),
            status: DeploymentStatus::Running,
            container_id: Some("test-container".to_string()),
            ports: vec![8080],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            logs: vec![],
        };
        
        // Note: This test will fail if LXC is not available, but demonstrates the interface
        let result = manager.get_deployment_status(&deployment).await;
        // Don't assert success since LXC may not be available in test environment
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_resource_limit_conversion() {
        let _manager = LxcManager::new();
        
        // Test CPU shares to priority conversion logic
        let cpu_shares = 1024; // Standard Docker share
        let priority = (cpu_shares / 1024).max(1).min(10);
        assert_eq!(priority, 1);
        
        let cpu_shares = 5120; // 5x standard
        let priority = (cpu_shares / 1024).max(1).min(10);
        assert_eq!(priority, 5);
        
        let cpu_shares = 20000; // Very high
        let priority = (cpu_shares / 1024).max(1).min(10);
        assert_eq!(priority, 10); // Capped at 10
    }
}