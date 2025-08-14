//! Precompiled Builder CLI for Automation Nation
//! 
//! This binary provides a command-line interface for building optimized,
//! precompiled releases of the Automation Nation webapp for multiple architectures.

use ci_test_suite::{PrecompiledBuilder, BuildConfig, TargetArch};
use clap::{Parser, Subcommand};
use env_logger;
use log::{info, error};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "precompiled-builder")]
#[command(about = "Precompiled build system for Automation Nation")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build precompiled binaries for all configured architectures
    Build {
        /// Output directory for build artifacts
        #[arg(long, default_value = "dist")]
        output: PathBuf,
        
        /// Version string for the build
        #[arg(long, default_value = env!("CARGO_PKG_VERSION"))]
        version: String,
        
        /// Enable Link Time Optimization (LTO)
        #[arg(long)]
        lto: bool,
        
        /// Strip debug symbols from binaries
        #[arg(long)]
        strip: bool,
        
        /// Compress binaries with gzip
        #[arg(long)]
        compress: bool,
        
        /// Target architectures (comma-separated)
        #[arg(long, value_delimiter = ',')]
        targets: Option<Vec<String>>,
        
        /// Optimization level
        #[arg(long, default_value = "release")]
        optimization: String,
    },
    
    /// Build for specific architecture only
    Target {
        /// Target architecture (e.g., x86_64-unknown-linux-gnu)
        target: String,
        
        /// Output directory
        #[arg(long, default_value = "dist")]
        output: PathBuf,
        
        /// Version string
        #[arg(long, default_value = env!("CARGO_PKG_VERSION"))]
        version: String,
    },
    
    /// List supported target architectures
    ListTargets,
    
    /// Generate container images for existing builds
    GenerateContainers {
        /// Build directory containing artifacts
        #[arg(long, default_value = "dist")]
        build_dir: PathBuf,
    },
    
    /// Verify build artifacts and checksums
    Verify {
        /// Build directory containing artifacts
        #[arg(long, default_value = "dist")]
        build_dir: PathBuf,
    },
    
    /// Show build statistics and information
    Info {
        /// Build directory containing artifacts
        #[arg(long, default_value = "dist")]
        build_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Build {
            output,
            version,
            lto,
            strip,
            compress,
            targets,
            optimization,
        } => {
            let target_architectures = if let Some(targets) = targets {
                parse_target_list(&targets)?
            } else {
                vec![
                    TargetArch::X86_64Linux,
                    TargetArch::X86_64LinuxMusl,
                    TargetArch::Aarch64Linux,
                    TargetArch::Aarch64LinuxMusl,
                ]
            };
            
            let optimization_level = match optimization.as_str() {
                "release" => ci_test_suite::precompiled_builder::OptimizationLevel::Release,
                "release-lto" => ci_test_suite::precompiled_builder::OptimizationLevel::ReleaseLto,
                "release-size" => ci_test_suite::precompiled_builder::OptimizationLevel::ReleaseSize,
                _ => {
                    error!("Invalid optimization level: {}", optimization);
                    return;
                }
            };
            
            let config = BuildConfig {
                target_architectures,
                optimization_level,
                include_debug_info: false,
                enable_lto: lto,
                strip_binaries: strip,
                compress_binaries: compress,
                output_directory: output,
                version,
            };
            
            build_precompiled(config).await
        }
        
        Commands::Target { target, output, version } => {
            let target_arch = parse_single_target(&target)?;
            
            let config = BuildConfig {
                target_architectures: vec![target_arch],
                output_directory: output,
                version,
                ..Default::default()
            };
            
            build_precompiled(config).await
        }
        
        Commands::ListTargets => {
            list_supported_targets().await
        }
        
        Commands::GenerateContainers { build_dir } => {
            generate_containers(build_dir).await
        }
        
        Commands::Verify { build_dir } => {
            verify_build_artifacts(build_dir).await
        }
        
        Commands::Info { build_dir } => {
            show_build_info(build_dir).await
        }
    };
    
    result
}

/// Build precompiled binaries with the given configuration
async fn build_precompiled(config: BuildConfig) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Starting precompiled build process");
    info!("Configuration: {:?}", config);
    
    let project_root = std::env::current_dir()?;
    let mut builder = PrecompiledBuilder::new(config, project_root);
    
    let artifacts = builder.build_all().await?;
    
    info!("Build completed successfully!");
    info!("Generated {} artifacts:", artifacts.len());
    
    for artifact in &artifacts {
        info!("  {} - {} bytes ({})", 
              artifact.target_arch.display_name(),
              artifact.size_bytes,
              artifact.binary_path.file_name().unwrap().to_string_lossy());
    }
    
    Ok(0)
}

/// List all supported target architectures
async fn list_supported_targets() -> Result<i32, Box<dyn std::error::Error>> {
    println!("Supported target architectures:");
    println!();
    
    let targets = vec![
        TargetArch::X86_64Linux,
        TargetArch::X86_64LinuxMusl,
        TargetArch::Aarch64Linux,
        TargetArch::Aarch64LinuxMusl,
        TargetArch::X86_64Darwin,
        TargetArch::Aarch64Darwin,
    ];
    
    for target in targets {
        println!("  {} - {}", target.target_triple(), target.display_name());
    }
    
    println!();
    println!("Usage examples:");
    println!("  precompiled-builder build --targets x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu");
    println!("  precompiled-builder target x86_64-unknown-linux-musl");
    
    Ok(0)
}

/// Generate container images for existing build artifacts
async fn generate_containers(build_dir: PathBuf) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Generating container images for build artifacts in: {}", build_dir.display());
    
    // Read build manifest
    let manifest_path = build_dir.join("build-manifest.json");
    if !manifest_path.exists() {
        return Err("Build manifest not found. Run 'build' command first.".into());
    }
    
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: ci_test_suite::precompiled_builder::BuildManifest = serde_json::from_str(&manifest_content)?;
    
    // Generate Dockerfiles for Linux artifacts
    let mut generated_count = 0;
    for artifact in &manifest.artifacts {
        match artifact.target_arch {
            TargetArch::X86_64Linux | TargetArch::X86_64LinuxMusl |
            TargetArch::Aarch64Linux | TargetArch::Aarch64LinuxMusl => {
                generate_dockerfile_for_artifact(&artifact, &build_dir)?;
                generated_count += 1;
            }
            _ => {
                info!("Skipping container generation for non-Linux target: {}", 
                      artifact.target_arch.display_name());
            }
        }
    }
    
    info!("Generated {} container configurations", generated_count);
    Ok(0)
}

/// Generate Dockerfile for a specific artifact
fn generate_dockerfile_for_artifact(
    artifact: &ci_test_suite::precompiled_builder::BuildArtifact, 
    build_dir: &PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
    let base_image = match artifact.target_arch {
        TargetArch::X86_64LinuxMusl | TargetArch::Aarch64LinuxMusl => "alpine:latest",
        _ => "debian:slim",
    };
    
    let binary_name = artifact.binary_path.file_name().unwrap().to_string_lossy();
    
    let dockerfile_content = format!(
        r#"# Optimized container image for Automation Nation
# Target: {target}
# Build time: {build_time}
# Binary size: {size} bytes

FROM {base_image}

# Install runtime dependencies
{install_deps}

# Create non-root user
{create_user}

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
    CMD /usr/local/bin/automation-nation --version || exit 1

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=3000

# Run the application
CMD ["/usr/local/bin/automation-nation", "serve", "--host", "0.0.0.0", "--port", "3000"]
"#,
        target = artifact.target_arch.display_name(),
        build_time = artifact.build_time.format("%Y-%m-%d %H:%M:%S UTC"),
        size = artifact.size_bytes,
        base_image = base_image,
        binary_name = binary_name,
        install_deps = if base_image.contains("alpine") {
            "RUN apk add --no-cache ca-certificates"
        } else {
            "RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*"
        },
        create_user = if base_image.contains("alpine") {
            "RUN adduser -D -s /bin/sh automation"
        } else {
            "RUN useradd --create-home --shell /bin/bash automation"
        }
    );
    
    let dockerfile_path = build_dir.join(format!("Dockerfile.{}", artifact.target_arch.target_triple()));
    std::fs::write(&dockerfile_path, dockerfile_content)?;
    
    info!("Generated Dockerfile: {}", dockerfile_path.display());
    Ok(())
}

/// Verify build artifacts and checksums
async fn verify_build_artifacts(build_dir: PathBuf) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Verifying build artifacts in: {}", build_dir.display());
    
    // Read checksums file
    let checksums_path = build_dir.join("checksums.txt");
    if !checksums_path.exists() {
        return Err("Checksums file not found".into());
    }
    
    let checksums_content = std::fs::read_to_string(&checksums_path)?;
    let mut verified_count = 0;
    let mut failed_count = 0;
    
    for line in checksums_content.lines() {
        if let Some((expected_hash, filename)) = line.split_once("  ") {
            let file_path = build_dir.join(filename);
            
            if file_path.exists() {
                match calculate_sha256(&file_path) {
                    Ok(actual_hash) => {
                        if actual_hash == expected_hash {
                            info!("✓ {} - checksum verified", filename);
                            verified_count += 1;
                        } else {
                            error!("✗ {} - checksum mismatch", filename);
                            error!("  Expected: {}", expected_hash);
                            error!("  Actual:   {}", actual_hash);
                            failed_count += 1;
                        }
                    }
                    Err(e) => {
                        error!("✗ {} - failed to calculate checksum: {}", filename, e);
                        failed_count += 1;
                    }
                }
            } else {
                error!("✗ {} - file not found", filename);
                failed_count += 1;
            }
        }
    }
    
    info!("Verification complete: {} verified, {} failed", verified_count, failed_count);
    
    if failed_count > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
}

/// Show build information and statistics
async fn show_build_info(build_dir: PathBuf) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Analyzing build artifacts in: {}", build_dir.display());
    
    // Read build manifest
    let manifest_path = build_dir.join("build-manifest.json");
    if !manifest_path.exists() {
        return Err("Build manifest not found".into());
    }
    
    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: ci_test_suite::precompiled_builder::BuildManifest = serde_json::from_str(&manifest_content)?;
    
    println!();
    println!("Build Information");
    println!("=================");
    println!("Version: {}", manifest.version);
    println!("Build Time: {}", manifest.build_time.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Artifacts: {}", manifest.artifacts.len());
    println!();
    
    println!("Build Configuration");
    println!("-------------------");
    println!("Optimization: {:?}", manifest.build_config.optimization_level);
    println!("LTO Enabled: {}", manifest.build_config.enable_lto);
    println!("Strip Binaries: {}", manifest.build_config.strip_binaries);
    println!("Compress Binaries: {}", manifest.build_config.compress_binaries);
    println!();
    
    println!("Artifacts");
    println!("---------");
    let mut total_size = 0u64;
    
    for artifact in &manifest.artifacts {
        println!("• {} ({})", 
                artifact.target_arch.display_name(),
                artifact.target_arch.target_triple());
        println!("  Size: {} bytes ({:.2} MB)", 
                artifact.size_bytes,
                artifact.size_bytes as f64 / 1024.0 / 1024.0);
        println!("  File: {}", 
                artifact.binary_path.file_name().unwrap().to_string_lossy());
        println!("  Checksum: {}", artifact.checksum_sha256);
        println!();
        
        total_size += artifact.size_bytes;
    }
    
    println!("Total Size: {} bytes ({:.2} MB)", total_size, total_size as f64 / 1024.0 / 1024.0);
    
    Ok(0)
}

/// Calculate SHA256 checksum of a file
fn calculate_sha256(file_path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    use sha2::{Sha256, Digest};
    
    let content = std::fs::read(file_path)?;
    let hash = Sha256::digest(&content);
    Ok(format!("{:x}", hash))
}

/// Parse a list of target architecture strings
fn parse_target_list(targets: &[String]) -> Result<Vec<TargetArch>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    
    for target in targets {
        result.push(parse_single_target(target)?);
    }
    
    Ok(result)
}

/// Parse a single target architecture string
fn parse_single_target(target: &str) -> Result<TargetArch, Box<dyn std::error::Error>> {
    match target {
        "x86_64-unknown-linux-gnu" => Ok(TargetArch::X86_64Linux),
        "x86_64-unknown-linux-musl" => Ok(TargetArch::X86_64LinuxMusl),
        "aarch64-unknown-linux-gnu" => Ok(TargetArch::Aarch64Linux),
        "aarch64-unknown-linux-musl" => Ok(TargetArch::Aarch64LinuxMusl),
        "x86_64-apple-darwin" => Ok(TargetArch::X86_64Darwin),
        "aarch64-apple-darwin" => Ok(TargetArch::Aarch64Darwin),
        _ => Err(format!("Unsupported target architecture: {}", target).into()),
    }
}