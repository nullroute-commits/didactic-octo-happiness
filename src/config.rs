//! Configuration management for the CI test suite

use crate::types::{Architecture, OperatingSystem, TestSuiteConfig};
use std::path::PathBuf;

/// Default configuration for the CI test suite
pub struct Config;

impl Config {
    /// Get the default test suite configuration
    pub fn default() -> TestSuiteConfig {
        TestSuiteConfig {
            script_path: "./collect_info.sh".to_string(),
            timeout_seconds: 300, // 5 minutes
            parallel_execution: true,
            max_retries: 3,
            output_directory: "./test_results".to_string(),
            target_architectures: vec![
                Architecture::X86_64,
                Architecture::Arm64,
                Architecture::I386,
            ],
            target_operating_systems: vec![
                OperatingSystem::Ubuntu,
                OperatingSystem::Alpine,
                OperatingSystem::Rocky,
            ],
            enable_regression_testing: true,
            enable_privilege_comparison: true,
        }
    }

    /// Get configuration for CI environment
    pub fn for_ci() -> TestSuiteConfig {
        let mut config = Self::default();
        config.parallel_execution = false; // More predictable in CI
        config.timeout_seconds = 600; // Longer timeout for CI
        config.max_retries = 1; // Fewer retries in CI
        config
    }

    /// Get configuration for local development
    pub fn for_development() -> TestSuiteConfig {
        let mut config = Self::default();
        config.timeout_seconds = 120; // Shorter timeout for dev
        config.target_architectures = vec![Architecture::X86_64]; // Only test current arch
        config.target_operating_systems = vec![OperatingSystem::Ubuntu]; // Only test current OS
        config
    }

    /// Load configuration from environment variables
    pub fn from_env() -> TestSuiteConfig {
        let mut config = Self::default();

        if let Ok(script_path) = std::env::var("CI_SCRIPT_PATH") {
            config.script_path = script_path;
        }

        if let Ok(timeout) = std::env::var("CI_TIMEOUT_SECONDS") {
            if let Ok(timeout_val) = timeout.parse() {
                config.timeout_seconds = timeout_val;
            }
        }

        if let Ok(output_dir) = std::env::var("CI_OUTPUT_DIR") {
            config.output_directory = output_dir;
        }

        if let Ok(parallel) = std::env::var("CI_PARALLEL") {
            config.parallel_execution = parallel.to_lowercase() == "true";
        }

        if let Ok(retries) = std::env::var("CI_MAX_RETRIES") {
            if let Ok(retries_val) = retries.parse() {
                config.max_retries = retries_val;
            }
        }

        if let Ok(regression) = std::env::var("CI_ENABLE_REGRESSION") {
            config.enable_regression_testing = regression.to_lowercase() == "true";
        }

        if let Ok(privilege_comp) = std::env::var("CI_ENABLE_PRIVILEGE_COMPARISON") {
            config.enable_privilege_comparison = privilege_comp.to_lowercase() == "true";
        }

        config
    }

    /// Validate the configuration
    pub fn validate(config: &TestSuiteConfig) -> Result<(), String> {
        let script_path = PathBuf::from(&config.script_path);
        if !script_path.exists() {
            return Err(format!("Script path does not exist: {}", config.script_path));
        }

        if !script_path.is_file() {
            return Err(format!("Script path is not a file: {}", config.script_path));
        }

        if config.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if config.target_architectures.is_empty() {
            return Err("At least one target architecture must be specified".to_string());
        }

        if config.target_operating_systems.is_empty() {
            return Err("At least one target operating system must be specified".to_string());
        }

        Ok(())
    }

    /// Get the script path as an absolute path
    pub fn get_script_path(config: &TestSuiteConfig) -> PathBuf {
        let path = PathBuf::from(&config.script_path);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    }

    /// Get the output directory as an absolute path
    pub fn get_output_dir(config: &TestSuiteConfig) -> PathBuf {
        let path = PathBuf::from(&config.output_directory);
        if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(path)
        }
    }
}