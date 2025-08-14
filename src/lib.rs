//! CI Test Suite for collect_info.sh
//! 
//! This library provides comprehensive testing infrastructure for the collect_info.sh script,
//! supporting both privileged and non-privileged execution across multiple Unix operating systems
//! and architectures.
//! 
//! Additionally, it includes a web interface for managing container deployments of open source software
//! based on system profiling information from collect_info.sh, with support for multiple container runtimes.

pub mod executor;
pub mod validator;
pub mod reporter;
pub mod config;
pub mod types;
pub mod privilege;
pub mod os_support;
// Web application modules
pub mod web_types;
pub mod github_api;
pub mod system_profiler;
pub mod deployment_profiles;
// Container runtime modules
pub mod podman_manager;
pub mod docker_manager;
pub mod lxc_manager;
pub mod container_runtime;
pub mod web_handlers;

pub use types::*;
pub use config::Config;
pub use executor::ScriptExecutor;
pub use validator::OutputValidator;
pub use reporter::TestReporter;
pub use privilege::PrivilegeManager;

// Web application exports
pub use web_types::*;
pub use github_api::GitHubApiClient;
pub use system_profiler::SystemProfiler;
pub use deployment_profiles::DeploymentProfileManager;
// Container runtime exports
pub use podman_manager::PodmanManager;
pub use docker_manager::DockerManager;
pub use lxc_manager::LxcManager;
pub use container_runtime::{ContainerRuntimeManager, RuntimeType, RuntimeInfo, RuntimeCapabilities};

/// Main result type for the CI test suite
pub type Result<T> = anyhow::Result<T>;
