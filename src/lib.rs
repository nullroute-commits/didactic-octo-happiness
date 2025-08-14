//! # Automation Nation CI Test Suite
//! 
//! This library provides comprehensive testing infrastructure for the `collect_info.sh` script,
//! supporting both privileged and non-privileged execution across multiple Unix operating systems
//! and architectures.
//! 
//! ## Architecture Overview
//! 
//! The library is organized into several key modules:
//! 
//! ### Core Testing Infrastructure
//! - **executor**: Script execution engine with privilege management
//! - **validator**: Output validation and JSON format verification
//! - **reporter**: Test result reporting and analysis
//! - **privilege**: Privilege escalation testing and security analysis
//! - **os_support**: Cross-platform operating system compatibility
//! 
//! ### Web Application Components
//! - **web_handlers**: HTTP API route handlers for container deployment
//! - **web_types**: Type definitions for web application data structures
//! - **github_api**: GitHub repository analysis and API integration
//! - **system_profiler**: System information profiling engine
//! - **deployment_profiles**: Container deployment profile management
//! 
//! ### Container Runtime Management
//! - **container_runtime**: Unified container runtime abstraction layer
//! - **docker_manager**: Docker container orchestration and management
//! - **podman_manager**: Podman container orchestration and management
//! - **lxc_manager**: LXC container orchestration and management
//! 
//! ### Core Utilities
//! - **config**: Configuration management and environment variable handling
//! - **types**: Common type definitions and data structures
//! 
//! ## Key Features
//! 
//! ### Multi-Runtime Container Support
//! The library provides a unified interface for managing containers across multiple runtimes:
//! - Docker (most common, production-ready)
//! - Podman (rootless, security-focused)
//! - LXC (system containers, lightweight virtualization)
//! 
//! ### System Profiling and Analysis
//! - Comprehensive system information collection via `collect_info.sh`
//! - Cross-architecture support (x86_64, ARM64, RISC-V, PowerPC, etc.)
//! - Performance analysis and resource utilization monitoring
//! - Network topology discovery and container integration
//! 
//! ### Web-Based Deployment Management
//! - RESTful API for container deployment automation
//! - GitHub repository analysis and automatic deployment suggestions
//! - Real-time deployment status monitoring and logging
//! - Security-focused container configuration and hardening
//! 
//! ### Comprehensive Testing Framework
//! - Privilege escalation testing across different execution contexts
//! - Cross-platform compatibility validation
//! - Performance regression detection and analysis
//! - Automated CI/CD integration with detailed reporting
//! 
//! ## Usage Examples
//! 
//! ### Basic System Profiling
//! ```rust,no_run
//! use ci_test_suite::{SystemProfiler, Result};
//! 
//! # async fn example() -> Result<()> {
//! let profiler = SystemProfiler::new("./collect_info.sh".to_string());
//! // Note: SystemProfiler provides methods for system analysis
//! # Ok(())
//! # }
//! ```
//! 
//! ### Container Deployment
//! ```rust,no_run
//! use ci_test_suite::{ContainerRuntimeManager, RuntimeType};
//! 
//! # async fn example() -> ci_test_suite::Result<()> {
//! let runtime_manager = ContainerRuntimeManager::new().await;
//! // Note: ContainerRuntimeManager provides runtime detection
//! # Ok(())
//! # }
//! ```
//! 
//! ### CI Test Execution
//! ```rust,no_run
//! use ci_test_suite::{ScriptExecutor, Config, TestReporter};
//! 
//! # fn example() -> ci_test_suite::Result<()> {
//! let executor = ScriptExecutor::new("./script.sh".to_string(), 60, 3);
//! // Note: ScriptExecutor provides test execution capabilities
//! # Ok(())
//! # }
//! ```

// Core testing infrastructure modules
pub mod executor;      // Script execution engine with privilege management
pub mod validator;     // Output validation and JSON format verification
pub mod reporter;      // Test result reporting and comprehensive analysis
pub mod config;        // Configuration management and environment variable handling
pub mod types;         // Common type definitions and shared data structures
pub mod privilege;     // Privilege escalation management and security testing
pub mod os_support;    // Cross-platform operating system compatibility layer

// Web application modules for container deployment management
pub mod web_types;           // Type definitions for web application data structures
pub mod github_api;          // GitHub API integration and repository analysis
pub mod system_profiler;     // System profiling engine and performance analysis
pub mod deployment_profiles; // Container deployment profile management and optimization

// Container runtime abstraction and management modules
pub mod podman_manager;      // Podman container orchestration and rootless management
pub mod docker_manager;      // Docker container orchestration and production deployment
pub mod lxc_manager;         // LXC container orchestration and system virtualization
pub mod kubernetes_manager;  // Kubernetes container orchestration and cloud-native deployment
pub mod container_runtime;   // Unified container runtime abstraction layer
pub mod web_handlers;        // HTTP API route handlers for web interface
pub mod rbac;                // Role-based access control and authentication system
pub mod database;            // Database connection and migration management
pub mod database_rbac;       // Database-backed RBAC implementation
pub mod sso;                 // SSO and OIDC integration
// pub mod password_reset;      // Password reset functionality (temporarily disabled for build)
pub mod auth_handlers;       // Authentication web handlers
pub mod certificate_manager; // Cryptographic certificate management system
pub mod certificate_api;     // Certificate management REST API
pub mod dashboard_manager;    // Custom dashboard management system
pub mod dashboard_api;        // Dashboard management REST API
pub mod plugin_marketplace;   // Plugin marketplace and management system
pub mod plugin_marketplace_api; // Plugin marketplace REST API
pub mod performance_optimizer; // Performance optimization and caching system
pub mod comprehensive_test_suite; // Comprehensive test framework
pub mod precompiled_builder;    // Precompiled build system

#[cfg(test)]
pub mod database_tests;      // Comprehensive database test suite

#[cfg(test)]
pub mod web_test_suite;      // Comprehensive web application test suite

// Re-export commonly used types and functions for convenient access
pub use types::*;                    // Core type definitions and data structures
pub use config::Config;              // Configuration management interface
pub use executor::ScriptExecutor;    // Main script execution engine
pub use validator::OutputValidator;  // JSON output validation utilities
pub use reporter::TestReporter;      // Test result reporting and analysis
pub use privilege::PrivilegeManager; // Privilege escalation testing framework

// Web application exports for container deployment and management
pub use web_types::*;                               // Web application type definitions
pub use github_api::GitHubApiClient;                // GitHub API integration client
pub use system_profiler::SystemProfiler;            // System profiling and analysis engine
pub use deployment_profiles::DeploymentProfileManager; // Deployment profile management

// Container runtime exports for multi-runtime container orchestration
pub use podman_manager::PodmanManager;              // Podman container management
pub use docker_manager::DockerManager;              // Docker container management
pub use lxc_manager::LxcManager;                    // LXC container management
pub use kubernetes_manager::{                       // Kubernetes container management
    KubernetesManager, KubernetesConfig, KubernetesDeploymentRequest,
    KubernetesDeploymentStatus, KubernetesResources, IngressConfig,
    ServiceConfig, HealthCheckConfig,
};
pub use container_runtime::{                        // Container runtime abstraction
    ContainerRuntimeManager,    // Runtime detection and management
    RuntimeType,               // Container runtime type enumeration
    RuntimeInfo,               // Runtime information and capabilities
    RuntimeCapabilities        // Runtime feature and security capabilities
};
pub use database::{DatabaseManager, DatabaseStats, CleanupStats}; // Database management
pub use database_rbac::DatabaseRbacManager;                  // Database-backed RBAC
pub use sso::{SsoManager, OidcConfig};                       // SSO integration
// pub use password_reset::{PasswordResetManager, PasswordResetRequest, PasswordResetConfirmation}; // Password reset (temporarily disabled for build)
pub use certificate_manager::{                               // Certificate management
    CertificateManager, Certificate, KeyPair, CertificateRequest, 
    RenewalRequest, ValidationResult, CryptoAlgorithm, SecurityLevel,
    CertificateConfig, CertificateStatus, CertificateType,
    AlgorithmCompliance, LegacyProtocolConfig, TlsConfig, CaConfig,
    TlsVersion, CipherSuite, SecurityPolicy, SecurityAuditReport,
};
pub use dashboard_manager::{                              // Dashboard management
    DashboardManager, Dashboard, DashboardPanel, DashboardTemplate,
    PanelType, DataSource, DashboardVisibility, TimeRange, PanelData
};
pub use plugin_marketplace::{                           // Plugin marketplace
    PluginMarketplace, PluginInfo, InstalledPlugin, PluginCategory,
    PluginSearchCriteria, PluginSearchResults, PluginInstallRequest,
    MarketplaceConfig, PluginStatus, PluginSortOrder,
};
pub use performance_optimizer::{                     // Performance optimization
    PerformanceOptimizer, CacheConfig, PerformanceMetrics,
    ResponseTimeMetrics, CacheMetrics, DatabaseMetrics,
    SystemMetrics, EndpointMetrics, CacheStats,
};
pub use comprehensive_test_suite::{ComprehensiveTestSuite, TestConfig, TestResults}; // Comprehensive testing
pub use precompiled_builder::{PrecompiledBuilder, BuildConfig, TargetArch, BuildArtifact}; // Precompiled builds

/// Main result type for the CI test suite operations
/// 
/// This type alias provides a convenient way to handle errors throughout the library
/// using the `anyhow` crate for flexible error handling and context preservation.
/// All public functions in this library return this result type for consistency.
pub type Result<T> = anyhow::Result<T>;
