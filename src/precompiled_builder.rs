//! Build system for precompiled webapp distribution
//! 
//! This module provides functionality for creating optimized, precompiled
//! releases of the Automation Nation webapp for multiple architectures.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use log::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

/// Supported target architectures for precompiled builds
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetArch {
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    X86_64Linux,
    #[serde(rename = "x86_64-unknown-linux-musl")]
    X86_64LinuxMusl,
    #[serde(rename = "aarch64-unknown-linux-gnu")]
    Aarch64Linux,
    #[serde(rename = "aarch64-unknown-linux-musl")]
    Aarch64LinuxMusl,
    #[serde(rename = "x86_64-apple-darwin")]
    X86_64Darwin,
    #[serde(rename = "aarch64-apple-darwin")]
    Aarch64Darwin,
}

impl TargetArch {
    /// Get the Rust target triple for this architecture
    pub fn target_triple(&self) -> &'static str {
        match self {
            TargetArch::X86_64Linux => "x86_64-unknown-linux-gnu",
            TargetArch::X86_64LinuxMusl => "x86_64-unknown-linux-musl",
            TargetArch::Aarch64Linux => "aarch64-unknown-linux-gnu",
            TargetArch::Aarch64LinuxMusl => "aarch64-unknown-linux-musl",
            TargetArch::X86_64Darwin => "x86_64-apple-darwin",
            TargetArch::Aarch64Darwin => "aarch64-apple-darwin",
        }
    }
    
    /// Get a human-readable name for this architecture
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetArch::X86_64Linux => "Linux x86_64 (glibc)",
            TargetArch::X86_64LinuxMusl => "Linux x86_64 (musl)",
            TargetArch::Aarch64Linux => "Linux ARM64 (glibc)",
            TargetArch::Aarch64LinuxMusl => "Linux ARM64 (musl)",
            TargetArch::X86_64Darwin => "macOS x86_64",
            TargetArch::Aarch64Darwin => "macOS ARM64",
        }
    }
    
    /// Get the file extension for binaries on this platform
    pub fn binary_extension(&self) -> &'static str {
        match self {
            TargetArch::X86_64Darwin | TargetArch::Aarch64Darwin => "",
            _ => "",
        }
    }
}

/// Build configuration for precompiled releases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target_architectures: Vec<TargetArch>,
    pub optimization_level: OptimizationLevel,
    pub include_debug_info: bool,
    pub enable_lto: bool,
    pub strip_binaries: bool,
    pub compress_binaries: bool,
    pub output_directory: PathBuf,
    pub version: String,
}

/// Optimization levels for builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "release-lto")]
    ReleaseLto,
    #[serde(rename = "release-size")]
    ReleaseSize,
}

impl OptimizationLevel {
    pub fn cargo_profile(&self) -> &'static str {
        match self {
            OptimizationLevel::Release => "release",
            OptimizationLevel::ReleaseLto => "release-lto",
            OptimizationLevel::ReleaseSize => "release-size",
        }
    }
}

/// Build artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub target_arch: TargetArch,
    pub binary_path: PathBuf,
    pub size_bytes: u64,
    pub checksum_sha256: String,
    pub build_time: chrono::DateTime<chrono::Utc>,
    pub cargo_metadata: HashMap<String, String>,
}

/// Precompiled build system
pub struct PrecompiledBuilder {
    config: BuildConfig,
    project_root: PathBuf,
    artifacts: Vec<BuildArtifact>,
}

impl PrecompiledBuilder {
    /// Create a new precompiled builder
    pub fn new(config: BuildConfig, project_root: PathBuf) -> Self {
        Self {
            config,
            project_root,
            artifacts: Vec::new(),
        }
    }
    
    /// Build precompiled binaries for all target architectures
    pub async fn build_all(&mut self) -> Result<Vec<BuildArtifact>> {
        info!("Starting precompiled build process for {} architectures", self.config.target_architectures.len());
        
        // Ensure output directory exists
        std::fs::create_dir_all(&self.config.output_directory)?;
        
        // Install required targets
        self.install_targets().await?;
        
        // Setup build environment
        self.setup_build_environment().await?;
        
        // Build for each target architecture
        for target_arch in &self.config.target_architectures {
            info!("Building for architecture: {}", target_arch.display_name());
            
            match self.build_target(target_arch).await {
                Ok(artifact) => {
                    info!("Successfully built for {}: {} bytes", 
                          target_arch.display_name(), artifact.size_bytes);
                    self.artifacts.push(artifact);
                }
                Err(e) => {
                    error!("Failed to build for {}: {}", target_arch.display_name(), e);
                    return Err(e);
                }
            }
        }
        
        // Generate build manifest
        self.generate_build_manifest().await?;
        
        // Generate container images
        self.generate_container_images().await?;
        
        info!("Precompiled build process completed successfully");
        Ok(self.artifacts.clone())
    }
    
    /// Install required Rust targets
    async fn install_targets(&self) -> Result<()> {
        info!("Installing required Rust targets");
        
        for target_arch in &self.config.target_architectures {
            let target = target_arch.target_triple();
            debug!("Installing target: {}", target);
            
            let output = Command::new("rustup")
                .args(&["target", "add", target])
                .output()?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to install target {}: {}", target, error);
                // Continue anyway - target might already be installed
            }
        }
        
        Ok(())
    }
    
    /// Setup build environment with optimizations
    async fn setup_build_environment(&self) -> Result<()> {
        info!("Setting up build environment");
        
        // Create Cargo.toml build profiles if they don't exist
        self.ensure_build_profiles().await?;
        
        // Set environment variables for optimization
        std::env::set_var("CARGO_PROFILE_RELEASE_LTO", if self.config.enable_lto { "true" } else { "false" });
        std::env::set_var("CARGO_PROFILE_RELEASE_STRIP", if self.config.strip_binaries { "true" } else { "false" });
        std::env::set_var("CARGO_PROFILE_RELEASE_OPT_LEVEL", "3");
        std::env::set_var("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "1");
        
        Ok(())
    }
    
    /// Ensure build profiles exist in Cargo.toml
    async fn ensure_build_profiles(&self) -> Result<()> {
        let cargo_toml_path = self.project_root.join("Cargo.toml");
        let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
        
        // Check if release-lto profile exists
        if !cargo_content.contains("[profile.release-lto]") {
            info!("Adding optimized build profiles to Cargo.toml");
            
            let additional_profiles = r#"

# Optimized build profiles for precompiled releases
[profile.release-lto]
inherits = "release"
lto = "fat"
codegen-units = 1
panic = "abort"

[profile.release-size]
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
"#;
            
            std::fs::write(&cargo_toml_path, cargo_content + additional_profiles)?;
            info!("Added optimized build profiles to Cargo.toml");
        }
        
        Ok(())
    }
    
    /// Build for a specific target architecture
    async fn build_target(&self, target_arch: &TargetArch) -> Result<BuildArtifact> {
        let target = target_arch.target_triple();
        let profile = self.config.optimization_level.cargo_profile();
        
        debug!("Building target {} with profile {}", target, profile);
        
        // Run cargo build
        let mut command = Command::new("cargo");
        command
            .args(&["build", "--release", "--target", target])
            .current_dir(&self.project_root);
        
        if profile != "release" {
            command.arg("--profile").arg(profile);
        }
        
        let output = command.output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Build failed for target {}: {}", target, error));
        }
        
        // Find the built binary
        let binary_path = self.find_built_binary(target_arch)?;
        
        // Calculate size and checksum
        let size_bytes = std::fs::metadata(&binary_path)?.len();
        let checksum_sha256 = self.calculate_sha256(&binary_path)?;
        
        // Copy to output directory with versioned name
        let output_name = format!("automation-nation-{}-{}{}", 
                                 self.config.version, 
                                 target_arch.target_triple(),
                                 target_arch.binary_extension());
        let output_path = self.config.output_directory.join(&output_name);
        
        std::fs::copy(&binary_path, &output_path)?;
        
        // Compress if requested
        if self.config.compress_binaries {
            self.compress_binary(&output_path).await?;
        }
        
        Ok(BuildArtifact {
            target_arch: target_arch.clone(),
            binary_path: output_path,
            size_bytes,
            checksum_sha256,
            build_time: chrono::Utc::now(),
            cargo_metadata: self.get_cargo_metadata(target_arch)?,
        })
    }
    
    /// Find the built binary for a target
    fn find_built_binary(&self, target_arch: &TargetArch) -> Result<PathBuf> {
        let target_dir = self.project_root.join("target").join(target_arch.target_triple()).join("release");
        
        // Look for web_server binary (our main webapp)
        let binary_name = format!("web_server{}", target_arch.binary_extension());
        let binary_path = target_dir.join(&binary_name);
        
        if binary_path.exists() {
            Ok(binary_path)
        } else {
            Err(anyhow!("Built binary not found: {}", binary_path.display()))
        }
    }
    
    /// Calculate SHA256 checksum of a file
    fn calculate_sha256(&self, file_path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = std::fs::read(file_path)?;
        let hash = Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }
    
    /// Compress a binary using gzip
    async fn compress_binary(&self, binary_path: &Path) -> Result<()> {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;
        
        let compressed_path = binary_path.with_extension("gz");
        
        let input = std::fs::read(binary_path)?;
        let output_file = std::fs::File::create(&compressed_path)?;
        let mut encoder = GzEncoder::new(output_file, Compression::best());
        encoder.write_all(&input)?;
        encoder.finish()?;
        
        info!("Compressed {} -> {} ({:.1}% size reduction)", 
              binary_path.file_name().unwrap().to_string_lossy(),
              compressed_path.file_name().unwrap().to_string_lossy(),
              (1.0 - (std::fs::metadata(&compressed_path)?.len() as f64 / input.len() as f64)) * 100.0);
        
        Ok(())
    }
    
    /// Get cargo metadata for a target
    fn get_cargo_metadata(&self, target_arch: &TargetArch) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        metadata.insert("target".to_string(), target_arch.target_triple().to_string());
        metadata.insert("optimization".to_string(), self.config.optimization_level.cargo_profile().to_string());
        metadata.insert("lto".to_string(), self.config.enable_lto.to_string());
        metadata.insert("stripped".to_string(), self.config.strip_binaries.to_string());
        metadata.insert("version".to_string(), self.config.version.clone());
        
        // Get Rust version
        if let Ok(output) = Command::new("rustc").arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                metadata.insert("rust_version".to_string(), version);
            }
        }
        
        Ok(metadata)
    }
    
    /// Generate build manifest with all artifacts
    async fn generate_build_manifest(&self) -> Result<()> {
        info!("Generating build manifest");
        
        let manifest = BuildManifest {
            version: self.config.version.clone(),
            build_time: chrono::Utc::now(),
            artifacts: self.artifacts.clone(),
            build_config: self.config.clone(),
        };
        
        let manifest_path = self.config.output_directory.join("build-manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, manifest_json)?;
        
        // Generate checksums file
        let checksums_path = self.config.output_directory.join("checksums.txt");
        let mut checksums = String::new();
        
        for artifact in &self.artifacts {
            let filename = artifact.binary_path.file_name().unwrap().to_string_lossy();
            checksums.push_str(&format!("{}  {}\n", artifact.checksum_sha256, filename));
        }
        
        std::fs::write(&checksums_path, checksums)?;
        
        info!("Build manifest saved to: {}", manifest_path.display());
        Ok(())
    }
    
    /// Generate optimized container images
    async fn generate_container_images(&self) -> Result<()> {
        info!("Generating optimized container images");
        
        for artifact in &self.artifacts {
            // Only generate container images for Linux targets
            match artifact.target_arch {
                TargetArch::X86_64Linux | TargetArch::X86_64LinuxMusl |
                TargetArch::Aarch64Linux | TargetArch::Aarch64LinuxMusl => {
                    self.generate_container_image(&artifact).await?;
                }
                _ => {
                    debug!("Skipping container image for non-Linux target: {}", artifact.target_arch.display_name());
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate a container image for a specific artifact
    async fn generate_container_image(&self, artifact: &BuildArtifact) -> Result<()> {
        let base_image = match artifact.target_arch {
            TargetArch::X86_64LinuxMusl | TargetArch::Aarch64LinuxMusl => "alpine:latest",
            _ => "debian:slim",
        };
        
        let dockerfile_content = format!(
            r#"FROM {base_image}

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* || \
    apk add --no-cache ca-certificates

# Create non-root user
RUN adduser --disabled-password --gecos '' automation || \
    adduser -D -s /bin/sh automation

# Copy precompiled binary
COPY {binary_name} /usr/local/bin/automation-nation
RUN chmod +x /usr/local/bin/automation-nation

# Create data directory
RUN mkdir -p /app/data && chown automation:automation /app/data

# Switch to non-root user
USER automation
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD automation-nation health || exit 1

# Expose port
EXPOSE 3000

# Run the application
CMD ["automation-nation", "serve", "--port", "3000"]
"#,
            base_image = base_image,
            binary_name = artifact.binary_path.file_name().unwrap().to_string_lossy()
        );
        
        let dockerfile_path = self.config.output_directory.join(format!("Dockerfile.{}", artifact.target_arch.target_triple()));
        std::fs::write(&dockerfile_path, dockerfile_content)?;
        
        info!("Generated Dockerfile for {}: {}", artifact.target_arch.display_name(), dockerfile_path.display());
        Ok(())
    }
}

/// Build manifest containing all artifacts and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    pub version: String,
    pub build_time: chrono::DateTime<chrono::Utc>,
    pub artifacts: Vec<BuildArtifact>,
    pub build_config: BuildConfig,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target_architectures: vec![
                TargetArch::X86_64Linux,
                TargetArch::X86_64LinuxMusl,
                TargetArch::Aarch64Linux,
                TargetArch::Aarch64LinuxMusl,
            ],
            optimization_level: OptimizationLevel::Release,
            include_debug_info: false,
            enable_lto: true,
            strip_binaries: true,
            compress_binaries: true,
            output_directory: PathBuf::from("dist"),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_target_arch_methods() {
        let arch = TargetArch::X86_64Linux;
        assert_eq!(arch.target_triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(arch.display_name(), "Linux x86_64 (glibc)");
        assert_eq!(arch.binary_extension(), "");
    }
    
    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert!(!config.target_architectures.is_empty());
        assert_eq!(config.optimization_level.cargo_profile(), "release");
        assert!(config.enable_lto);
        assert!(config.strip_binaries);
    }
}