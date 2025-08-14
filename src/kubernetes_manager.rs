//! Kubernetes Container Management
//!
//! This module provides comprehensive Kubernetes cluster integration for
//! container deployment and management within the Automation Nation platform.

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use uuid::Uuid;

use crate::{
    container_runtime::{RuntimeCapabilities, RuntimeInfo},
};

/// Kubernetes deployment manager
pub struct KubernetesManager {
    /// kubectl binary path
    kubectl_path: String,
    /// Default namespace
    default_namespace: String,
    /// Cluster configuration
    cluster_config: KubernetesConfig,
}

/// Kubernetes cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    /// Kubernetes API server endpoint
    pub api_server: Option<String>,
    /// Authentication method
    pub auth_method: KubernetesAuthMethod,
    /// Default resource requests and limits
    pub default_resources: KubernetesResources,
    /// Ingress configuration
    pub ingress_config: IngressConfig,
    /// Storage class configuration
    pub storage_classes: Vec<String>,
    /// Node selector constraints
    pub node_selectors: HashMap<String, String>,
    /// Tolerations for pod scheduling
    pub tolerations: Vec<Toleration>,
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            api_server: None,
            auth_method: KubernetesAuthMethod::ServiceAccount,
            default_resources: KubernetesResources::default(),
            ingress_config: IngressConfig::default(),
            storage_classes: vec!["standard".to_string()],
            node_selectors: HashMap::new(),
            tolerations: Vec::new(),
        }
    }
}

/// Kubernetes authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KubernetesAuthMethod {
    /// Service account authentication (in-cluster)
    ServiceAccount,
    /// Kubeconfig file authentication
    Kubeconfig { path: String },
    /// Token-based authentication
    Token { token: String },
    /// Certificate-based authentication
    Certificate { cert_path: String, key_path: String },
}

/// Kubernetes resource specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesResources {
    /// CPU requests (millicores)
    pub cpu_request: u32,
    /// Memory requests (MB)
    pub memory_request: u32,
    /// CPU limits (millicores)
    pub cpu_limit: Option<u32>,
    /// Memory limits (MB)
    pub memory_limit: Option<u32>,
    /// Storage request (GB)
    pub storage_request: Option<u32>,
}

impl Default for KubernetesResources {
    fn default() -> Self {
        Self {
            cpu_request: 100,
            memory_request: 128,
            cpu_limit: Some(500),
            memory_limit: Some(512),
            storage_request: None,
        }
    }
}

/// Ingress configuration for exposing services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfig {
    /// Ingress class name
    pub ingress_class: String,
    /// TLS configuration
    pub tls_enabled: bool,
    /// Certificate issuer (cert-manager)
    pub cert_issuer: Option<String>,
    /// Custom annotations
    pub annotations: HashMap<String, String>,
}

impl Default for IngressConfig {
    fn default() -> Self {
        Self {
            ingress_class: "nginx".to_string(),
            tls_enabled: true,
            cert_issuer: Some("letsencrypt-prod".to_string()),
            annotations: HashMap::new(),
        }
    }
}

/// Pod toleration for scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    pub key: String,
    pub operator: String,
    pub value: Option<String>,
    pub effect: String,
}

/// Kubernetes deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesDeploymentRequest {
    /// Application name
    pub name: String,
    /// Namespace
    pub namespace: Option<String>,
    /// Container image
    pub image: String,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Resource specifications
    pub resources: Option<KubernetesResources>,
    /// Replica count
    pub replicas: u32,
    /// Service exposure configuration
    pub service_config: Option<ServiceConfig>,
    /// Ingress configuration
    pub ingress_config: Option<IngressConfig>,
    /// Health check configuration
    pub health_checks: Option<HealthCheckConfig>,
    /// Custom labels
    pub labels: HashMap<String, String>,
    /// Node selector
    pub node_selector: HashMap<String, String>,
}

/// Port mapping for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub name: String,
    pub container_port: u16,
    pub service_port: Option<u16>,
    pub protocol: String,
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub volume_type: VolumeType,
}

/// Kubernetes volume types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeType {
    EmptyDir,
    PersistentVolumeClaim { size: String, storage_class: Option<String> },
    ConfigMap { name: String },
    Secret { name: String },
    HostPath { path: String },
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    pub cluster_ip: Option<String>,
    pub external_traffic_policy: Option<String>,
}

/// Kubernetes service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub liveness_probe: Option<ProbeConfig>,
    pub readiness_probe: Option<ProbeConfig>,
    pub startup_probe: Option<ProbeConfig>,
}

/// Probe configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeConfig {
    pub probe_type: ProbeType,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub failure_threshold: u32,
}

/// Health check probe types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeType {
    HttpGet { path: String, port: u16 },
    TcpSocket { port: u16 },
    Exec { command: Vec<String> },
}

/// Kubernetes deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesDeploymentStatus {
    pub deployment_id: Uuid,
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub replicas: u32,
    pub ready_replicas: u32,
    pub unavailable_replicas: u32,
    pub created_at: DateTime<Utc>,
    pub conditions: Vec<DeploymentCondition>,
    pub pods: Vec<PodStatus>,
}

/// Deployment condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: DateTime<Utc>,
}

/// Pod status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
    pub name: String,
    pub phase: String,
    pub ready: bool,
    pub restart_count: u32,
    pub node: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl KubernetesManager {
    /// Create a new Kubernetes manager instance
    pub fn new() -> Self {
        Self {
            kubectl_path: "kubectl".to_string(),
            default_namespace: "default".to_string(),
            cluster_config: KubernetesConfig::default(),
        }
    }

    /// Create Kubernetes manager with custom configuration
    pub fn with_config(config: KubernetesConfig) -> Self {
        Self {
            kubectl_path: "kubectl".to_string(),
            default_namespace: "default".to_string(),
            cluster_config: config,
        }
    }

    /// Check if kubectl is available and cluster is accessible
    pub async fn check_availability(&self) -> Result<bool> {
        let output = Command::new(&self.kubectl_path)
            .args(&["cluster-info"])
            .output()
            .context("Failed to execute kubectl command")?;

        Ok(output.status.success())
    }

    /// Get cluster information and capabilities
    pub async fn get_runtime_info(&self) -> Result<RuntimeInfo> {
        let version_output = Command::new(&self.kubectl_path)
            .args(&["version", "--client", "--output=json"])
            .output()
            .context("Failed to get kubectl version")?;

        let version = if version_output.status.success() {
            String::from_utf8_lossy(&version_output.stdout)
                .lines()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else {
            "unknown".to_string()
        };

        let capabilities = RuntimeCapabilities {
            supports_networking: true,
            supports_volumes: true,
            supports_resource_limits: true,
            supports_health_checks: true,
            supports_rolling_updates: true,
            supports_load_balancing: true,
            supports_service_discovery: true,
            supports_secrets_management: true,
            max_cpu_cores: Some(1000), // Kubernetes can handle very large clusters
            max_memory_gb: Some(1000),
        };

        Ok(RuntimeInfo {
            runtime_type: "kubernetes".to_string(),
            version,
            available: true,
            capabilities,
        })
    }

    /// Deploy an application to Kubernetes
    pub async fn deploy_application(
        &self,
        request: KubernetesDeploymentRequest,
    ) -> Result<Uuid> {
        log::info!("Deploying application {} to Kubernetes", request.name);

        let deployment_id = Uuid::new_v4();
        let namespace = request.namespace.as_deref().unwrap_or(&self.default_namespace);

        // Generate Kubernetes manifests
        let deployment_yaml = self.generate_deployment_manifest(&request)?;
        let service_yaml = self.generate_service_manifest(&request)?;
        let ingress_yaml = self.generate_ingress_manifest(&request)?;

        // Apply manifests
        self.apply_manifest(&deployment_yaml, namespace).await?;
        
        if service_yaml.is_some() {
            self.apply_manifest(&service_yaml.unwrap(), namespace).await?;
        }
        
        if ingress_yaml.is_some() {
            self.apply_manifest(&ingress_yaml.unwrap(), namespace).await?;
        }

        log::info!("Successfully deployed {} with ID {}", request.name, deployment_id);
        Ok(deployment_id)
    }

    /// Get deployment status (native Kubernetes method)
    pub async fn get_k8s_deployment_status(
        &self,
        deployment_name: &str,
        namespace: Option<&str>,
    ) -> Result<KubernetesDeploymentStatus> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        let output = Command::new(&self.kubectl_path)
            .args(&[
                "get", "deployment", deployment_name,
                "-n", ns,
                "-o", "json"
            ])
            .output()
            .context("Failed to get deployment status")?;

        if !output.status.success() {
            return Err(anyhow!("Deployment not found: {}", deployment_name));
        }

        // Parse deployment status (simplified)
        let status = KubernetesDeploymentStatus {
            deployment_id: Uuid::new_v4(), // Would be stored/retrieved from metadata
            name: deployment_name.to_string(),
            namespace: ns.to_string(),
            status: "Running".to_string(), // Would be parsed from kubectl output
            replicas: 1,
            ready_replicas: 1,
            unavailable_replicas: 0,
            created_at: Utc::now(),
            conditions: Vec::new(),
            pods: Vec::new(),
        };

        Ok(status)
    }

    /// Scale deployment
    pub async fn scale_deployment(
        &self,
        deployment_name: &str,
        replicas: u32,
        namespace: Option<&str>,
    ) -> Result<()> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        let output = Command::new(&self.kubectl_path)
            .args(&[
                "scale", "deployment", deployment_name,
                "--replicas", &replicas.to_string(),
                "-n", ns
            ])
            .output()
            .context("Failed to scale deployment")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to scale deployment: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        log::info!("Scaled deployment {} to {} replicas", deployment_name, replicas);
        Ok(())
    }

    /// Delete deployment
    pub async fn delete_deployment(
        &self,
        deployment_name: &str,
        namespace: Option<&str>,
    ) -> Result<()> {
        let ns = namespace.unwrap_or(&self.default_namespace);
        
        let output = Command::new(&self.kubectl_path)
            .args(&[
                "delete", "deployment", deployment_name,
                "-n", ns
            ])
            .output()
            .context("Failed to delete deployment")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to delete deployment: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        log::info!("Deleted deployment {}", deployment_name);
        Ok(())
    }

    /// Generate Kubernetes deployment manifest
    fn generate_deployment_manifest(&self, request: &KubernetesDeploymentRequest) -> Result<String> {
        // This is a simplified manifest generation
        // In production, you'd use a proper templating system or Kubernetes API client
        
        let namespace = request.namespace.as_deref().unwrap_or(&self.default_namespace);
        let resources = request.resources.as_ref().unwrap_or(&self.cluster_config.default_resources);
        
        let manifest = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
  namespace: {}
  labels:
    app: {}
    managed-by: automation-nation
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: {}
        image: {}
        resources:
          requests:
            cpu: {}m
            memory: {}Mi
          limits:
            cpu: {}m
            memory: {}Mi
        env:
{}
        ports:
{}
"#,
            request.name,
            namespace,
            request.name,
            request.replicas,
            request.name,
            request.name,
            request.name,
            request.image,
            resources.cpu_request,
            resources.memory_request,
            resources.cpu_limit.unwrap_or(resources.cpu_request * 2),
            resources.memory_limit.unwrap_or(resources.memory_request * 2),
            self.generate_env_vars(&request.env_vars),
            self.generate_ports(&request.ports)
        );

        Ok(manifest)
    }

    /// Generate service manifest if needed
    fn generate_service_manifest(&self, request: &KubernetesDeploymentRequest) -> Result<Option<String>> {
        if request.service_config.is_none() || request.ports.is_empty() {
            return Ok(None);
        }

        let namespace = request.namespace.as_deref().unwrap_or(&self.default_namespace);
        let service_config = request.service_config.as_ref().unwrap();
        
        let service_type = match service_config.service_type {
            ServiceType::ClusterIP => "ClusterIP",
            ServiceType::NodePort => "NodePort", 
            ServiceType::LoadBalancer => "LoadBalancer",
            ServiceType::ExternalName => "ExternalName",
        };

        let manifest = format!(r#"
apiVersion: v1
kind: Service
metadata:
  name: {}
  namespace: {}
  labels:
    app: {}
    managed-by: automation-nation
spec:
  type: {}
  selector:
    app: {}
  ports:
{}
"#,
            request.name,
            namespace,
            request.name,
            service_type,
            request.name,
            self.generate_service_ports(&request.ports)
        );

        Ok(Some(manifest))
    }

    /// Generate ingress manifest if needed
    fn generate_ingress_manifest(&self, request: &KubernetesDeploymentRequest) -> Result<Option<String>> {
        if request.ingress_config.is_none() {
            return Ok(None);
        }

        let namespace = request.namespace.as_deref().unwrap_or(&self.default_namespace);
        let ingress_config = request.ingress_config.as_ref().unwrap();
        
        let manifest = format!(r#"
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {}
  namespace: {}
  labels:
    app: {}
    managed-by: automation-nation
  annotations:
    kubernetes.io/ingress.class: {}
{}
spec:
  rules:
  - host: {}.local
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: {}
            port:
              number: 80
"#,
            request.name,
            namespace,
            request.name,
            ingress_config.ingress_class,
            self.generate_annotations(&ingress_config.annotations),
            request.name,
            request.name
        );

        Ok(Some(manifest))
    }

    /// Apply Kubernetes manifest
    async fn apply_manifest(&self, manifest: &str, namespace: &str) -> Result<()> {
        // Write manifest to temporary file
        let temp_file = format!("/tmp/k8s-manifest-{}.yaml", Uuid::new_v4());
        std::fs::write(&temp_file, manifest)
            .context("Failed to write manifest to temporary file")?;

        let output = Command::new(&self.kubectl_path)
            .args(&["apply", "-f", &temp_file, "-n", namespace])
            .output()
            .context("Failed to apply manifest")?;

        // Clean up temporary file
        std::fs::remove_file(&temp_file).ok();

        if !output.status.success() {
            return Err(anyhow!("Failed to apply manifest: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        Ok(())
    }

    /// Helper function to generate environment variables YAML
    fn generate_env_vars(&self, env_vars: &HashMap<String, String>) -> String {
        env_vars.iter()
            .map(|(key, value)| format!("        - name: {}\n          value: \"{}\"", key, value))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Helper function to generate ports YAML
    fn generate_ports(&self, ports: &[PortMapping]) -> String {
        ports.iter()
            .map(|port| format!("        - containerPort: {}\n          name: {}", port.container_port, port.name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Helper function to generate service ports YAML
    fn generate_service_ports(&self, ports: &[PortMapping]) -> String {
        ports.iter()
            .map(|port| format!("  - port: {}\n    targetPort: {}\n    name: {}", 
                port.service_port.unwrap_or(port.container_port), 
                port.container_port, 
                port.name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Helper function to generate annotations YAML
    fn generate_annotations(&self, annotations: &HashMap<String, String>) -> String {
        if annotations.is_empty() {
            return String::new();
        }
        
        let annotation_lines: Vec<String> = annotations.iter()
            .map(|(key, value)| format!("    {}: \"{}\"", key, value))
            .collect();
        
        format!("  annotations:\n{}", annotation_lines.join("\n"))
    }

    // Adapter methods to match ContainerRuntimeManager interface
    
    /// Deploy using the container runtime interface
    pub async fn deploy(&self, _profile: &crate::web_types::DeploymentProfile, _request: &crate::web_types::CreateDeploymentRequest) -> crate::Result<crate::web_types::CreateDeploymentResponse> {
        Err(anyhow!("Kubernetes deploy adapter not yet implemented"))
    }
    
    /// Undeploy using the container runtime interface
    pub async fn undeploy(&self, _deployment: &crate::web_types::DeploymentInstance) -> crate::Result<()> {
        Err(anyhow!("Kubernetes undeploy adapter not yet implemented"))
    }
    
    /// Get container logs using the container runtime interface
    pub async fn get_container_logs(&self, _deployment: &crate::web_types::DeploymentInstance, _tail_lines: u32) -> crate::Result<Vec<crate::web_types::DeploymentLog>> {
        Err(anyhow!("Kubernetes get_container_logs adapter not yet implemented"))
    }
    
    /// Get deployment status using the container runtime interface (adapter)
    pub async fn get_deployment_status(&self, _deployment: &crate::web_types::DeploymentInstance) -> crate::Result<crate::web_types::DeploymentStatus> {
        Err(anyhow!("Kubernetes get_deployment_status adapter not yet implemented"))
    }
    
    /// List deployments using the container runtime interface
    pub async fn list_deployments(&self) -> crate::Result<Vec<std::collections::HashMap<String, String>>> {
        Err(anyhow!("Kubernetes list_deployments adapter not yet implemented"))
    }
    
    /// Restart deployment using the container runtime interface
    pub async fn restart_deployment(&self, _deployment: &crate::web_types::DeploymentInstance) -> crate::Result<()> {
        Err(anyhow!("Kubernetes restart_deployment adapter not yet implemented"))
    }
    
    /// Update resources using the container runtime interface
    pub async fn update_resources(&self, _deployment: &crate::web_types::DeploymentInstance, _limits: &crate::web_types::ResourceLimits) -> crate::Result<()> {
        Err(anyhow!("Kubernetes update_resources adapter not yet implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kubernetes_manager_creation() {
        let manager = KubernetesManager::new();
        assert_eq!(manager.kubectl_path, "kubectl");
        assert_eq!(manager.default_namespace, "default");
    }

    #[test]
    fn test_kubernetes_config_default() {
        let config = KubernetesConfig::default();
        assert_eq!(config.default_resources.cpu_request, 100);
        assert_eq!(config.default_resources.memory_request, 128);
        assert!(config.ingress_config.tls_enabled);
    }

    #[test]
    fn test_deployment_manifest_generation() {
        let manager = KubernetesManager::new();
        let request = KubernetesDeploymentRequest {
            name: "test-app".to_string(),
            namespace: Some("test".to_string()),
            image: "nginx:latest".to_string(),
            env_vars: HashMap::new(),
            ports: vec![PortMapping {
                name: "http".to_string(),
                container_port: 80,
                service_port: Some(80),
                protocol: "TCP".to_string(),
            }],
            volumes: Vec::new(),
            resources: None,
            replicas: 2,
            service_config: None,
            ingress_config: None,
            health_checks: None,
            labels: HashMap::new(),
            node_selector: HashMap::new(),
        };

        let manifest = manager.generate_deployment_manifest(&request).unwrap();
        assert!(manifest.contains("test-app"));
        assert!(manifest.contains("nginx:latest"));
        assert!(manifest.contains("replicas: 2"));
    }
}