//! CI Test Suite for collect_info.sh
//! 
//! This library provides comprehensive testing infrastructure for the collect_info.sh script,
//! supporting both privileged and non-privileged execution across multiple Unix operating systems
//! and architectures.

pub mod executor;
pub mod validator;
pub mod reporter;
pub mod config;
pub mod types;
pub mod privilege;
pub mod os_support;

pub use types::*;
pub use config::Config;
pub use executor::ScriptExecutor;
pub use validator::OutputValidator;
pub use reporter::TestReporter;
pub use privilege::PrivilegeManager;

/// Main result type for the CI test suite
pub type Result<T> = anyhow::Result<T>;
