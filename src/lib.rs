//! CI Test Suite for collect_info.sh
//! 
//! This library provides comprehensive testing infrastructure for the collect_info.sh script,
//! supporting both privileged and non-privileged execution across multiple Unix operating systems
//! and architectures.
//! 
//! Additionally, it includes a web interface for managing Podman deployments of open source software
//! based on system profiling information from collect_info.sh.

pub mod executor;
pub mod validator;
pub mod reporter;
pub mod config;
pub mod types;
pub mod privilege;
pub mod os_support;
// New modules for web application
pub mod web_types;
pub mod github_api;
pub mod system_profiler;
pub mod deployment_profiles;
pub mod podman_manager;
pub mod web_handlers;

pub use types::*;
pub use config::Config;
pub use executor::ScriptExecutor;
pub use validator::OutputValidator;
pub use reporter::TestReporter;
pub use privilege::PrivilegeManager;

// New exports for web application
pub use web_types::*;
pub use github_api::GitHubApiClient;
pub use system_profiler::SystemProfiler;
pub use deployment_profiles::DeploymentProfileManager;
pub use podman_manager::PodmanManager;

/// Main result type for the CI test suite
pub type Result<T> = anyhow::Result<T>;
