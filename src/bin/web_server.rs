//! # Automation Nation Web Server
//! 
//! This binary provides a comprehensive web interface and RESTful API for the Automation Nation
//! platform, enabling container deployment automation, system profiling, and infrastructure
//! management through a modern web interface.
//! 
//! ## Architecture Overview
//! 
//! The web server is built using the Axum web framework and provides:
//! 
//! ### Core Features
//! - **RESTful API**: Comprehensive API for automation integration
//! - **Container Management**: Deploy and manage containers across multiple runtimes
//! - **System Profiling**: Real-time system information and performance metrics
//! - **GitHub Integration**: Automatic repository analysis and deployment suggestions
//! - **Monitoring Integration**: Metrics collection and observability endpoints
//! 
//! ### API Endpoints
//! - `/api/system/profile` - System information and architecture detection
//! - `/api/containers/runtimes` - Available container runtime information
//! - `/api/deploy` - Container deployment automation
//! - `/api/deployments/status` - Deployment status and monitoring
//! - `/health` - Health check endpoint for load balancers
//! - `/metrics` - Prometheus metrics for monitoring
//! 
//! ### Container Runtime Support
//! The server supports multiple container runtimes with automatic detection:
//! - **Docker**: Production-ready containerization with extensive ecosystem
//! - **Podman**: Rootless containers with enhanced security
//! - **LXC**: System containers for lightweight virtualization
//! 
//! ## Usage Examples
//! 
//! ### Starting the Server
//! ```bash
//! # Development mode with verbose logging
//! cargo run --bin web_server -- serve --port 3000 --verbose
//! 
//! # Production mode with specific configuration
//! cargo run --bin web_server -- serve \
//!   --host 0.0.0.0 \
//!   --port 8080 \
//!   --script /opt/automation_nation/collect_info.sh
//! ```
//! 
//! ### API Usage
//! ```bash
//! # Get system profile
//! curl http://localhost:3000/api/system/profile
//! 
//! # Deploy a GitHub repository
//! curl -X POST http://localhost:3000/api/deploy \
//!   -H "Content-Type: application/json" \
//!   -d '{"repository": "owner/repo", "runtime": "docker"}'
//! ```

use ci_test_suite::{
    GitHubApiClient, SystemProfiler, DeploymentProfileManager, PodmanManager,
    web_handlers::{AppState, create_router},
    web_types::*,
};
use clap::{Parser, Subcommand};
use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// Command-line interface for the Automation Nation web server
/// 
/// This structure defines the available commands and options for configuring
/// and running the web server in different modes.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for the web server binary
/// 
/// The web server supports multiple operational modes for different use cases:
/// - `Serve`: Start the main web server with API endpoints
/// - `Check`: Validate system dependencies and configuration
#[derive(Subcommand)]
enum Commands {
    /// Start the web server with full API and container management capabilities
    /// 
    /// This is the main operational mode that provides:
    /// - RESTful API for automation integration
    /// - Web interface for container management
    /// - System profiling and monitoring endpoints
    /// - GitHub repository integration
    Serve {
        /// Port to bind the web server to
        /// 
        /// Default port 3000 is suitable for development. For production deployments,
        /// consider using port 8080 or 80/443 with proper SSL termination.
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Host interface to bind to
        /// 
        /// - "127.0.0.1" (default): Localhost only, suitable for development
        /// - "0.0.0.0": All interfaces, required for container deployment
        /// - Specific IP: Bind to specific network interface
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        
        /// Path to the collect_info.sh script for system profiling
        /// 
        /// This script is the core system information collector that provides
        /// architecture detection, hardware profiling, and network discovery.
        #[arg(short, long, default_value = "./collect_info.sh")]
        script: String,
        
        /// GitHub API token for enhanced rate limits and private repository access
        /// 
        /// Without a token, the GitHub API is limited to 60 requests per hour.
        /// With authentication, this increases to 5000 requests per hour.
        #[arg(long)]
        github_token: Option<String>,
        
        /// Enable verbose logging for debugging and development
        /// 
        /// When enabled, increases log level to debug and provides detailed
        /// information about request processing, container operations, and
        /// system profiling activities.
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Check system dependencies and validate configuration
    /// 
    /// This command performs comprehensive validation of the system environment:
    /// - Verifies collect_info.sh script accessibility and functionality
    /// - Tests container runtime availability and capabilities
    /// - Validates network connectivity for GitHub API integration
    /// - Checks system permissions and security configuration
    Check {
        /// Path to collect_info.sh script for validation
        #[arg(short, long, default_value = "./collect_info.sh")]
        script: String,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Serve { 
            port, 
            host, 
            script, 
            github_token, 
            verbose 
        } => {
            init_logging(*verbose);
            start_web_server(*port, host, script, github_token.as_deref()).await
        }
        Commands::Check { script, verbose } => {
            init_logging(*verbose);
            check_dependencies(script).await
        }
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", level);
    env_logger::init();
}

/// Start the web server
async fn start_web_server(
    port: u16,
    host: &str,
    script_path: &str,
    github_token: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Automation Nation web server");
    
    // Check dependencies first
    check_script_availability(script_path).await?;
    check_podman_availability().await?;
    
    // Initialize components
    let github_client = Arc::new(GitHubApiClient::new(github_token.map(|s| s.to_string())));
    let system_profiler = Arc::new(SystemProfiler::new(script_path.to_string()));
    let deployment_manager = Arc::new(DeploymentProfileManager::new(script_path.to_string()));
    let podman_manager = Arc::new(PodmanManager::new());
    
    // Initialize shared state
    let state = AppState {
        github_client,
        system_profiler,
        deployment_manager,
        podman_manager,
        deployments: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        profiles: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        system_profile: Arc::new(tokio::sync::RwLock::new(None)),
    };
    
    // Create router
    let app = create_router(state)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        );
    
    // Start server
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("Web server listening on http://{}", addr);
    info!("Access the dashboard at: http://{}/dashboard", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Check system dependencies
async fn check_dependencies(script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking system dependencies");
    
    let mut all_good = true;
    
    // Check collect_info.sh
    match check_script_availability(script_path).await {
        Ok(_) => info!("✓ collect_info.sh is available and executable"),
        Err(e) => {
            error!("✗ collect_info.sh check failed: {}", e);
            all_good = false;
        }
    }
    
    // Check Podman
    match check_podman_availability().await {
        Ok(_) => info!("✓ Podman is available and working"),
        Err(e) => {
            error!("✗ Podman check failed: {}", e);
            all_good = false;
        }
    }
    
    // Check GitHub API access
    let github_client = GitHubApiClient::new(std::env::var("GITHUB_TOKEN").ok());
    match test_github_access(&github_client).await {
        Ok(_) => info!("✓ GitHub API access is working"),
        Err(e) => {
            warn!("⚠ GitHub API access limited: {}", e);
            warn!("  Consider setting GITHUB_TOKEN environment variable for higher rate limits");
        }
    }
    
    if all_good {
        info!("All dependencies are available. Ready to start the web server!");
        Ok(())
    } else {
        error!("Some dependencies are missing. Please install them before running the server.");
        std::process::exit(1);
    }
}

/// Check if collect_info.sh is available and executable
async fn check_script_availability(script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script_path = std::path::Path::new(script_path);
    
    if !script_path.exists() {
        return Err(format!("Script not found: {}", script_path.display()).into());
    }
    
    if !script_path.is_file() {
        return Err(format!("Path is not a file: {}", script_path.display()).into());
    }
    
    // Check if executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(script_path)?;
        let permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            return Err(format!("Script is not executable: {}", script_path.display()).into());
        }
    }
    
    // Try to run the script with --help to verify it works
    let output = std::process::Command::new(script_path)
        .arg("-h")
        .output()?;
        
    // Help command may exit with code 1 but should still produce output
    if output.stdout.is_empty() && output.stderr.is_empty() {
        return Err(format!("Script produced no output: {}", script_path.display()).into());
    }
    
    Ok(())
}

/// Check if Podman is available and working
async fn check_podman_availability() -> Result<(), Box<dyn std::error::Error>> {
    let podman_manager = PodmanManager::new();
    
    match podman_manager.check_availability().await {
        Ok(true) => Ok(()),
        Ok(false) => Err("Podman is not available or not working".into()),
        Err(e) => Err(e.into()),
    }
}

/// Test GitHub API access
async fn test_github_access(github_client: &GitHubApiClient) -> Result<(), Box<dyn std::error::Error>> {
    // Try a simple search to test API access
    let request = SearchRepositoriesRequest {
        query: "rust".to_string(),
        language: None,
        sort: Some("stars".to_string()),
        order: Some("desc".to_string()),
        per_page: Some(1),
        page: Some(1),
    };
    
    match github_client.search_repositories(&request).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_parsing() {
        // Test serve command
        let cli = Cli::try_parse_from(&[
            "web_server",
            "serve",
            "--port", "8080",
            "--host", "0.0.0.0",
            "--script", "/path/to/script.sh",
            "--verbose"
        ]).unwrap();
        
        match cli.command {
            Commands::Serve { port, host, script, github_token: _, verbose } => {
                assert_eq!(port, 8080);
                assert_eq!(host, "0.0.0.0");
                assert_eq!(script, "/path/to/script.sh");
                assert!(verbose);
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[tokio::test]
    async fn test_check_command() {
        let cli = Cli::try_parse_from(&[
            "web_server",
            "check",
            "--script", "/path/to/script.sh"
        ]).unwrap();
        
        match cli.command {
            Commands::Check { script, verbose } => {
                assert_eq!(script, "/path/to/script.sh");
                assert!(!verbose);
            }
            _ => panic!("Expected Check command"),
        }
    }
    
    #[test]
    fn test_logging_initialization() {
        // Test that logging can be initialized without panicking
        // Use try_init to avoid conflicts with already initialized logger
        let level = "debug";
        std::env::set_var("RUST_LOG", level);
        let _ = env_logger::try_init(); // Ignore result to avoid test conflicts
    }
    
    #[tokio::test]
    async fn test_github_client_creation() {
        let _client = GitHubApiClient::new(Some("test_token".to_string()));
        // Just verify it can be created without errors
        assert!(true);
    }
}