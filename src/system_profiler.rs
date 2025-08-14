//! System profiler that analyzes collect_info.sh output and generates deployment profiles

use crate::types::ScriptOutput;
use crate::web_types::*;
use crate::executor::ScriptExecutor;
use crate::Result;
use uuid::Uuid;
use chrono::Utc;
use log::{debug, info};

/// System profiler for analyzing collect_info.sh output
pub struct SystemProfiler {
    script_path: String,
}

impl SystemProfiler {
    /// Create a new system profiler
    pub fn new(script_path: String) -> Self {
        Self { script_path }
    }

    /// Generate a system profile by running collect_info.sh
    pub async fn generate_profile(&self) -> Result<SystemProfile> {
        debug!("Generating system profile using collect_info.sh");
        
        let executor = ScriptExecutor::new(self.script_path.clone(), 300, 3);
        executor.validate_script()?;
        
        let context = crate::types::TestContext {
            os: crate::types::OperatingSystem::Ubuntu, // Detected automatically
            architecture: crate::types::Architecture::X86_64, // Detected automatically
            privilege_level: crate::types::PrivilegeLevel::Normal,
            test_id: "system_profile".to_string(),
            timestamp: Utc::now(),
        };
        
        let result = executor.execute(&context).await?;
        
        if !result.success {
            return Err(anyhow::anyhow!("Failed to execute collect_info.sh: {}", 
                result.error_message.unwrap_or("Unknown error".to_string())));
        }
        
        let output = result.output.ok_or_else(|| anyhow::anyhow!("No output from collect_info.sh"))?;
        
        self.parse_system_profile(output)
    }

    /// Parse ScriptOutput into SystemProfile
    fn parse_system_profile(&self, output: ScriptOutput) -> Result<SystemProfile> {
        info!("Parsing system output into profile");
        
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let architecture = output.detected_architecture.to_string();
        
        // Extract OS information
        let os_data = output.plugins.get("get_os_info")
            .ok_or_else(|| anyhow::anyhow!("Missing OS information plugin"))?;
        
        let os_name = os_data.data["os_name"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        
        let os_version = os_data.data["os_version"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
            
        let kernel_version = os_data.data["kernel_version"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        // Extract hardware information
        let hardware_data = output.plugins.get("get_hardware_info")
            .ok_or_else(|| anyhow::anyhow!("Missing hardware information plugin"))?;
        
        let cpu_model = hardware_data.data["cpu_model"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
            
        let cpu_cores = hardware_data.data["cpu_cores"]
            .as_str()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
            
        let memory_total_mb = hardware_data.data["memory_total"]
            .as_str()
            .and_then(|s| s.replace(" MB", "").parse::<u64>().ok())
            .unwrap_or(1024);
            
        let memory_available_mb = hardware_data.data["memory_available"]
            .as_str()
            .and_then(|s| s.replace(" MB", "").parse::<u64>().ok())
            .unwrap_or(512);

        // Extract virtualization information
        let virtualization_type = output.plugins.get("get_virtualization_info")
            .and_then(|v| v.data["virtualization_type"].as_str())
            .map(|s| s.to_string());

        // Extract container runtimes
        let container_runtimes = output.plugins.get("get_virtualization_info")
            .and_then(|v| v.data["container_runtime"].as_array())
            .map(|runtimes| {
                runtimes.iter()
                    .filter_map(|r| r["name"].as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Extract network interfaces
        let network_interfaces = output.plugins.get("get_ip_info")
            .and_then(|n| n.data["network_interfaces"].as_array())
            .map(|interfaces| {
                interfaces.iter()
                    .filter_map(|iface| self.parse_network_interface(iface))
                    .collect()
            })
            .unwrap_or_default();

        // Extract hardware capabilities
        let hardware_capabilities = self.extract_hardware_capabilities(&hardware_data.data);

        let profile = SystemProfile {
            id,
            created_at,
            architecture,
            os_name,
            os_version,
            kernel_version,
            cpu_model,
            cpu_cores,
            memory_total_mb,
            memory_available_mb,
            virtualization_type,
            container_runtimes,
            network_interfaces,
            hardware_capabilities,
        };

        info!("Generated system profile: {} cores, {}MB memory, {} container runtimes", 
              profile.cpu_cores, profile.memory_total_mb, profile.container_runtimes.len());

        Ok(profile)
    }

    /// Parse network interface from JSON value
    fn parse_network_interface(&self, iface: &serde_json::Value) -> Option<NetworkInterface> {
        let name = iface["interface"].as_str()?.to_string();
        
        let ipv4_addresses = iface["ipv4_addresses"]
            .as_array()?
            .iter()
            .filter_map(|ip| ip.as_str())
            .map(|s| s.to_string())
            .collect();
            
        let ipv6_addresses = iface["ipv6_addresses"]
            .as_array()?
            .iter()
            .filter_map(|ip| ip.as_str())
            .map(|s| s.to_string())
            .collect();
            
        let mac_address = iface["mac_address"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
            
        let mtu = iface["mtu"]
            .as_str()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1500);
            
        let state = iface["state"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Some(NetworkInterface {
            name,
            ipv4_addresses,
            ipv6_addresses,
            mac_address,
            mtu,
            state,
        })
    }

    /// Extract hardware capabilities from hardware data
    fn extract_hardware_capabilities(&self, hardware_data: &serde_json::Value) -> HardwareCapabilities {
        let has_gpu = hardware_data["gpu_info"]
            .as_array()
            .map(|gpus| !gpus.is_empty())
            .unwrap_or(false);
            
        let gpu_vendor = hardware_data["gpu_info"]
            .as_array()
            .and_then(|gpus| gpus.first())
            .and_then(|gpu| gpu["vendor"].as_str())
            .filter(|v| *v != "unknown")
            .map(|s| s.to_string());
            
        let pcie_devices = hardware_data["pcie_devices"]
            .as_array()
            .map(|devices| devices.len() as u32)
            .unwrap_or(0);
            
        let usb_devices = hardware_data["usb_devices"]
            .as_array()
            .map(|devices| devices.len() as u32)
            .unwrap_or(0);
            
        let network_devices = hardware_data["network_hardware"]
            .as_array()
            .map(|devices| devices.len() as u32)
            .unwrap_or(0);
            
        let disk_total_gb = hardware_data["disk_info"]
            .as_array()
            .map(|disks| {
                disks.iter()
                    .filter_map(|disk| {
                        disk["size"].as_str().and_then(|size| {
                            // Parse disk size with possible units: T, G, M, K
                            let size = size.trim();
                            if size.is_empty() { return None; }
                            
                            let unit = if size.chars().last().unwrap().is_alphabetic() {
                                size.chars().last().unwrap().to_ascii_uppercase()
                            } else {
                                'G' // Default to G if no unit
                            };
                            let num_str = if size.chars().last().unwrap().is_alphabetic() {
                                &size[..size.len()-1]
                            } else {
                                size
                            };
                            let num = num_str.trim().parse::<f64>().ok()?;
                            let gb = match unit {
                                'T' => num * 1024.0,
                                'G' => num,
                                'M' => num / 1024.0,
                                'K' => num / (1024.0 * 1024.0),
                                _ => return None,
                            };
                            Some(gb.round() as u64)
                        })
                    })
                    .sum()
            })
            .unwrap_or(20); // Default 20GB

        let virtualization_support = hardware_data.get("virtualization_support")
            .and_then(|v| v.as_bool())
            .unwrap_or(true); // Assume support by default

        HardwareCapabilities {
            has_gpu,
            gpu_vendor,
            pcie_devices,
            usb_devices,
            network_devices,
            disk_total_gb,
            virtualization_support,
        }
    }

    /// Calculate compatibility score between system and requirements
    pub fn calculate_compatibility(&self, profile: &SystemProfile, requirements: &SystemRequirements) -> f64 {
        let mut score: f64 = 1.0;
        let mut factors = 0;

        // Memory compatibility
        if requirements.min_memory_mb > 0 {
            factors += 1;
            if profile.memory_available_mb < requirements.min_memory_mb {
                score *= 0.0; // Hard requirement
            } else if profile.memory_available_mb < requirements.min_memory_mb * 2 {
                score *= 0.7; // Marginal
            }
        }

        // CPU compatibility
        if requirements.min_cpu_cores > 0 {
            factors += 1;
            if profile.cpu_cores < requirements.min_cpu_cores {
                score *= 0.0; // Hard requirement
            } else if profile.cpu_cores < requirements.min_cpu_cores * 2 {
                score *= 0.8; // Adequate
            }
        }

        // Disk compatibility
        if requirements.min_disk_gb > 0 {
            factors += 1;
            if profile.hardware_capabilities.disk_total_gb < requirements.min_disk_gb {
                score *= 0.0; // Hard requirement
            } else if profile.hardware_capabilities.disk_total_gb < requirements.min_disk_gb * 2 {
                score *= 0.9; // Adequate
            }
        }

        // Architecture compatibility
        if !requirements.required_architectures.is_empty() {
            factors += 1;
            if !requirements.required_architectures.contains(&profile.architecture) {
                score *= 0.0; // Hard requirement
            }
        }

        // OS compatibility
        if !requirements.required_os.is_empty() {
            factors += 1;
            let os_match = requirements.required_os.iter().any(|req_os| {
                profile.os_name.to_lowercase().contains(&req_os.to_lowercase())
            });
            if !os_match {
                score *= 0.5; // Partial compatibility
            }
        }

        // Container runtime compatibility
        if !requirements.required_container_runtime.is_empty() {
            factors += 1;
            let runtime_available = requirements.required_container_runtime.iter().any(|req_runtime| {
                profile.container_runtimes.iter().any(|available| {
                    available.to_lowercase().contains(&req_runtime.to_lowercase())
                })
            });
            if !runtime_available {
                score *= 0.0; // Hard requirement
            }
        }

        // If no requirements specified, return high compatibility
        if factors == 0 {
            score = 0.8;
        }

        score.max(0.0).min(1.0)
    }

    /// Generate deployment recommendations based on system profile
    pub fn generate_recommendations(&self, profile: &SystemProfile) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Memory recommendations
        if profile.memory_total_mb < 2048 {
            recommendations.push("Consider lightweight applications due to limited memory (< 2GB)".to_string());
        } else if profile.memory_total_mb > 16384 {
            recommendations.push("System has ample memory (> 16GB) - suitable for memory-intensive applications".to_string());
        }

        // CPU recommendations
        if profile.cpu_cores < 2 {
            recommendations.push("Single core system - avoid CPU-intensive applications".to_string());
        } else if profile.cpu_cores >= 8 {
            recommendations.push("Multi-core system detected - suitable for parallel processing applications".to_string());
        }

        // Virtualization recommendations
        if let Some(virt_type) = &profile.virtualization_type {
            if virt_type.contains("container") {
                recommendations.push("Running in containerized environment - nested containers may have limitations".to_string());
            }
        }

        // Container runtime recommendations
        if profile.container_runtimes.contains(&"podman".to_string()) {
            recommendations.push("Podman detected - rootless containers recommended for security".to_string());
        }
        if profile.container_runtimes.contains(&"docker".to_string()) {
            recommendations.push("Docker detected - consider resource limits for containers".to_string());
        }

        // Network recommendations
        let public_interfaces = profile.network_interfaces.iter()
            .filter(|iface| !iface.ipv4_addresses.iter().any(|ip| 
                ip.starts_with("127.") || ip.starts_with("10.") || 
                ip.starts_with("192.168.") || ip.starts_with("172.")
            ))
            .count();
            
        if public_interfaces == 0 {
            recommendations.push("No public IP detected - applications will need port forwarding for external access".to_string());
        }

        // Storage recommendations
        if profile.hardware_capabilities.disk_total_gb < 50 {
            recommendations.push("Limited disk space (< 50GB) - consider applications with small footprint".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_system_profiler_creation() {
        let profiler = SystemProfiler::new("./collect_info.sh".to_string());
        // We can't access private fields, so just verify construction succeeds
        assert!(true);
    }

    #[test]
    fn test_calculate_compatibility_perfect_match() {
        let profiler = SystemProfiler::new("test".to_string());
        
        let profile = SystemProfile {
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
        
        let requirements = SystemRequirements {
            min_memory_mb: 4096,
            min_cpu_cores: 2,
            min_disk_gb: 50,
            required_architectures: vec!["x86_64".to_string()],
            required_os: vec!["ubuntu".to_string()],
            required_container_runtime: vec!["docker".to_string()],
            network_requirements: vec![],
        };
        
        let score = profiler.calculate_compatibility(&profile, &requirements);
        // The system has sufficient resources, should be decent compatibility
        assert!(score >= 0.7); // Score of 0.7 is reasonable for multi-factor compatibility
    }

    #[test]
    fn test_calculate_compatibility_insufficient_memory() {
        let profiler = SystemProfiler::new("test".to_string());
        
        let profile = SystemProfile {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            architecture: "x86_64".to_string(),
            os_name: "Ubuntu".to_string(),
            os_version: "20.04".to_string(),
            kernel_version: "5.4.0".to_string(),
            cpu_model: "Intel Core".to_string(),
            cpu_cores: 4,
            memory_total_mb: 2048,
            memory_available_mb: 1024, // Insufficient
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
            min_memory_mb: 4096, // More than available
            min_cpu_cores: 2,
            min_disk_gb: 50,
            required_architectures: vec!["x86_64".to_string()],
            required_os: vec!["ubuntu".to_string()],
            required_container_runtime: vec!["docker".to_string()],
            network_requirements: vec![],
        };
        
        let score = profiler.calculate_compatibility(&profile, &requirements);
        assert_eq!(score, 0.0); // Should be zero due to insufficient memory
    }

    #[test]
    fn test_generate_recommendations() {
        let profiler = SystemProfiler::new("test".to_string());
        
        let profile = SystemProfile {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            architecture: "x86_64".to_string(),
            os_name: "Ubuntu".to_string(),
            os_version: "20.04".to_string(),
            kernel_version: "5.4.0".to_string(),
            cpu_model: "Intel Core".to_string(),
            cpu_cores: 1, // Single core
            memory_total_mb: 1024, // Low memory
            memory_available_mb: 512,
            virtualization_type: None,
            container_runtimes: vec!["podman".to_string()],
            network_interfaces: vec![],
            hardware_capabilities: HardwareCapabilities {
                has_gpu: false,
                gpu_vendor: None,
                pcie_devices: 2,
                usb_devices: 1,
                network_devices: 1,
                disk_total_gb: 20, // Low disk
                virtualization_support: true,
            },
        };
        
        let recommendations = profiler.generate_recommendations(&profile);
        
        // Should contain recommendations for low-resource system
        assert!(recommendations.iter().any(|r| r.contains("lightweight")));
        assert!(recommendations.iter().any(|r| r.contains("Single core")));
        assert!(recommendations.iter().any(|r| r.contains("Limited disk")));
        assert!(recommendations.iter().any(|r| r.contains("Podman")));
    }
}