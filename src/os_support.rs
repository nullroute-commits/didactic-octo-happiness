//! Operating system support for cross-platform testing

use crate::types::OperatingSystem;
use crate::Result;
use std::collections::HashMap;

/// Manages OS-specific testing configurations and Docker container support
pub struct OsSupport;

impl OsSupport {
    /// Get the top 3 Unix operating systems for fiscal year 2025 Q1
    pub fn get_top_unix_os() -> Vec<OperatingSystem> {
        vec![
            OperatingSystem::Ubuntu,  // Most popular for containers/cloud
            OperatingSystem::Alpine,  // Popular for containers due to small size
            OperatingSystem::Rocky,   // Enterprise standard (successor to CentOS)
        ]
    }

    /// Get all supported operating systems
    pub fn get_all_supported_os() -> Vec<OperatingSystem> {
        vec![
            OperatingSystem::Ubuntu,
            OperatingSystem::Alpine,
            OperatingSystem::CentOS,
            OperatingSystem::Rocky,
            OperatingSystem::Debian,
        ]
    }

    /// Get Docker image configurations for each OS
    pub fn get_docker_images() -> HashMap<OperatingSystem, Vec<DockerImageConfig>> {
        let mut images = HashMap::new();

        images.insert(
            OperatingSystem::Ubuntu,
            vec![
                DockerImageConfig {
                    image: "ubuntu:24.04".to_string(),
                    tag: "noble".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "arm/v7", "ppc64le", "s390x"],
                    package_manager: "apt".to_string(),
                    init_commands: vec![
                        "apt-get update".to_string(),
                        "apt-get install -y bash curl wget sudo".to_string(),
                    ],
                },
                DockerImageConfig {
                    image: "ubuntu:22.04".to_string(),
                    tag: "jammy".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "arm/v7", "ppc64le", "s390x"],
                    package_manager: "apt".to_string(),
                    init_commands: vec![
                        "apt-get update".to_string(),
                        "apt-get install -y bash curl wget sudo".to_string(),
                    ],
                },
            ],
        );

        images.insert(
            OperatingSystem::Alpine,
            vec![
                DockerImageConfig {
                    image: "alpine:3.19".to_string(),
                    tag: "latest".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "arm/v7", "arm/v6", "ppc64le", "s390x"],
                    package_manager: "apk".to_string(),
                    init_commands: vec![
                        "apk update".to_string(),
                        "apk add --no-cache bash curl wget sudo".to_string(),
                    ],
                },
                DockerImageConfig {
                    image: "alpine:3.18".to_string(),
                    tag: "stable".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "arm/v7", "arm/v6", "ppc64le", "s390x"],
                    package_manager: "apk".to_string(),
                    init_commands: vec![
                        "apk update".to_string(),
                        "apk add --no-cache bash curl wget sudo".to_string(),
                    ],
                },
            ],
        );

        images.insert(
            OperatingSystem::Rocky,
            vec![
                DockerImageConfig {
                    image: "rockylinux:9".to_string(),
                    tag: "latest".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "ppc64le", "s390x"],
                    package_manager: "dnf".to_string(),
                    init_commands: vec![
                        "dnf update -y".to_string(),
                        "dnf install -y bash curl wget sudo".to_string(),
                    ],
                },
                DockerImageConfig {
                    image: "rockylinux:8".to_string(),
                    tag: "stable".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "ppc64le"],
                    package_manager: "dnf".to_string(),
                    init_commands: vec![
                        "dnf update -y".to_string(),
                        "dnf install -y bash curl wget sudo".to_string(),
                    ],
                },
            ],
        );

        images.insert(
            OperatingSystem::CentOS,
            vec![
                DockerImageConfig {
                    image: "centos:stream9".to_string(),
                    tag: "stream".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "ppc64le", "s390x"],
                    package_manager: "dnf".to_string(),
                    init_commands: vec![
                        "dnf update -y".to_string(),
                        "dnf install -y bash curl wget sudo".to_string(),
                    ],
                },
            ],
        );

        images.insert(
            OperatingSystem::Debian,
            vec![
                DockerImageConfig {
                    image: "debian:12".to_string(),
                    tag: "bookworm".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "arm/v7", "ppc64le", "s390x"],
                    package_manager: "apt".to_string(),
                    init_commands: vec![
                        "apt-get update".to_string(),
                        "apt-get install -y bash curl wget sudo".to_string(),
                    ],
                },
                DockerImageConfig {
                    image: "debian:11".to_string(),
                    tag: "bullseye".to_string(),
                    architecture_support: vec!["x86_64", "arm64", "arm/v7", "ppc64le", "s390x"],
                    package_manager: "apt".to_string(),
                    init_commands: vec![
                        "apt-get update".to_string(),
                        "apt-get install -y bash curl wget sudo".to_string(),
                    ],
                },
            ],
        );

        images
    }

    /// Get the best Docker image for a given OS and architecture
    pub fn get_best_docker_image(
        os: &OperatingSystem,
        architecture: &str,
    ) -> Result<DockerImageConfig> {
        let images = Self::get_docker_images();
        
        let os_images = images.get(os).ok_or_else(|| {
            anyhow::anyhow!("No Docker images configured for OS: {}", os)
        })?;

        // Find the first image that supports the target architecture
        for image in os_images {
            if image.supports_architecture(architecture) {
                return Ok(image.clone());
            }
        }

        Err(anyhow::anyhow!(
            "No Docker image found for OS {} and architecture {}",
            os,
            architecture
        ))
    }

    /// Check if an OS is supported for testing
    pub fn is_os_supported(os: &OperatingSystem) -> bool {
        Self::get_all_supported_os().contains(os)
    }

    /// Get OS-specific test requirements
    pub fn get_os_requirements(os: &OperatingSystem) -> OsRequirements {
        match os {
            OperatingSystem::Ubuntu => OsRequirements {
                required_packages: vec!["bash", "curl", "sudo", "coreutils"],
                optional_packages: vec!["lshw", "pciutils", "usbutils", "net-tools"],
                privilege_support: true,
                container_runtime_available: true,
                expected_init_system: Some("systemd"),
            },
            OperatingSystem::Alpine => OsRequirements {
                required_packages: vec!["bash", "curl", "sudo", "coreutils"],
                optional_packages: vec!["pciutils", "usbutils", "net-tools"],
                privilege_support: true,
                container_runtime_available: true,
                expected_init_system: Some("openrc"),
            },
            OperatingSystem::Rocky => OsRequirements {
                required_packages: vec!["bash", "curl", "sudo", "coreutils"],
                optional_packages: vec!["pciutils", "usbutils", "net-tools", "lshw"],
                privilege_support: true,
                container_runtime_available: true,
                expected_init_system: Some("systemd"),
            },
            OperatingSystem::CentOS => OsRequirements {
                required_packages: vec!["bash", "curl", "sudo", "coreutils"],
                optional_packages: vec!["pciutils", "usbutils", "net-tools", "lshw"],
                privilege_support: true,
                container_runtime_available: true,
                expected_init_system: Some("systemd"),
            },
            OperatingSystem::Debian => OsRequirements {
                required_packages: vec!["bash", "curl", "sudo", "coreutils"],
                optional_packages: vec!["lshw", "pciutils", "usbutils", "net-tools"],
                privilege_support: true,
                container_runtime_available: true,
                expected_init_system: Some("systemd"),
            },
        }
    }

    /// Generate Docker run command for testing
    pub fn generate_docker_command(
        os: &OperatingSystem,
        architecture: &str,
        script_path: &str,
        privilege_level: crate::types::PrivilegeLevel,
    ) -> Result<Vec<String>> {
        let image_config = Self::get_best_docker_image(os, architecture)?;
        
        let mut command = vec![
            "docker".to_string(),
            "run".to_string(),
            "--rm".to_string(),
            "--platform".to_string(),
            format!("linux/{}", Self::docker_arch_name(architecture)),
        ];

        // Mount the script and plugins directory
        let script_parent = std::path::Path::new(script_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        
        command.extend(vec![
            "-v".to_string(),
            format!("{}:/workspace", script_parent.display()),
            "-w".to_string(),
            "/workspace".to_string(),
        ]);

        // Set environment variables
        match privilege_level {
            crate::types::PrivilegeLevel::Normal => {
                command.extend(vec!["-e".to_string(), "ENABLE_SUDO_SUPPORT=0".to_string()]);
            }
            crate::types::PrivilegeLevel::Escalated => {
                command.extend(vec!["-e".to_string(), "ENABLE_SUDO_SUPPORT=1".to_string()]);
            }
        }

        command.extend(vec![
            "-e".to_string(),
            "ENABLE_HASHING=1".to_string(),
            image_config.image,
        ]);

        // Add init commands and script execution
        command.extend(vec![
            "sh".to_string(),
            "-c".to_string(),
            format!(
                "{} && chmod +x {} && {}",
                image_config.init_commands.join(" && "),
                std::path::Path::new(script_path).file_name().unwrap().to_string_lossy(),
                std::path::Path::new(script_path).file_name().unwrap().to_string_lossy()
            ),
        ]);

        Ok(command)
    }

    /// Convert architecture name to Docker platform format
    fn docker_arch_name(arch: &str) -> &str {
        match arch {
            "x86_64" => "amd64",
            "arm64" | "aarch64" => "arm64",
            "aarch32" | "armv7l" => "arm/v7",
            "i386" | "i686" => "386",
            "ppc64le" => "ppc64le",
            "s390x" => "s390x",
            "riscv64" => "riscv64",
            "mips64" => "mips64",
            "sparc64" => "sparc64",
            _ => arch,
        }
    }
}

/// Docker image configuration for a specific OS
#[derive(Debug, Clone)]
pub struct DockerImageConfig {
    pub image: String,
    pub tag: String,
    pub architecture_support: Vec<&'static str>,
    pub package_manager: String,
    pub init_commands: Vec<String>,
}

impl DockerImageConfig {
    /// Check if this image supports the given architecture
    pub fn supports_architecture(&self, arch: &str) -> bool {
        let docker_arch = OsSupport::docker_arch_name(arch);
        self.architecture_support.iter().any(|&supported| {
            supported == arch || supported == docker_arch
        })
    }
}

/// OS-specific requirements for testing
#[derive(Debug, Clone)]
pub struct OsRequirements {
    pub required_packages: Vec<&'static str>,
    pub optional_packages: Vec<&'static str>,
    pub privilege_support: bool,
    pub container_runtime_available: bool,
    pub expected_init_system: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_top_unix_os() {
        let top_os = OsSupport::get_top_unix_os();
        assert_eq!(top_os.len(), 3);
        assert!(top_os.contains(&OperatingSystem::Ubuntu));
        assert!(top_os.contains(&OperatingSystem::Alpine));
        assert!(top_os.contains(&OperatingSystem::Rocky));
    }

    #[test]
    fn test_get_docker_images() {
        let images = OsSupport::get_docker_images();
        assert!(!images.is_empty());
        assert!(images.contains_key(&OperatingSystem::Ubuntu));
        assert!(images.contains_key(&OperatingSystem::Alpine));
        assert!(images.contains_key(&OperatingSystem::Rocky));
    }

    #[test]
    fn test_get_best_docker_image() {
        let result = OsSupport::get_best_docker_image(&OperatingSystem::Ubuntu, "x86_64");
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert!(config.image.contains("ubuntu"));
        assert!(config.supports_architecture("x86_64"));
    }

    #[test]
    fn test_docker_arch_name() {
        assert_eq!(OsSupport::docker_arch_name("x86_64"), "amd64");
        assert_eq!(OsSupport::docker_arch_name("arm64"), "arm64");
        assert_eq!(OsSupport::docker_arch_name("aarch32"), "arm/v7");
        assert_eq!(OsSupport::docker_arch_name("i386"), "386");
    }

    #[test]
    fn test_is_os_supported() {
        assert!(OsSupport::is_os_supported(&OperatingSystem::Ubuntu));
        assert!(OsSupport::is_os_supported(&OperatingSystem::Alpine));
        assert!(OsSupport::is_os_supported(&OperatingSystem::Rocky));
    }

    #[test]
    fn test_get_os_requirements() {
        let ubuntu_req = OsSupport::get_os_requirements(&OperatingSystem::Ubuntu);
        assert!(ubuntu_req.privilege_support);
        assert!(ubuntu_req.container_runtime_available);
        assert_eq!(ubuntu_req.expected_init_system, Some("systemd"));
        assert!(ubuntu_req.required_packages.contains(&"bash"));

        let alpine_req = OsSupport::get_os_requirements(&OperatingSystem::Alpine);
        assert!(alpine_req.privilege_support);
        assert_eq!(alpine_req.expected_init_system, Some("openrc"));
    }

    #[test]
    fn test_docker_image_config_supports_architecture() {
        let config = DockerImageConfig {
            image: "ubuntu:24.04".to_string(),
            tag: "latest".to_string(),
            architecture_support: vec!["x86_64", "arm64"],
            package_manager: "apt".to_string(),
            init_commands: vec![],
        };

        assert!(config.supports_architecture("x86_64"));
        assert!(config.supports_architecture("arm64"));
        assert!(!config.supports_architecture("s390x"));
    }
}