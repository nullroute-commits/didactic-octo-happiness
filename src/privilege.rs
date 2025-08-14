//! Privilege management for testing both escalated and non-escalated script execution

use crate::types::{Architecture, OperatingSystem, PrivilegeLevel, TestContext};
use crate::Result;
use chrono::Utc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Manages privilege levels for test execution
pub struct PrivilegeManager;

impl PrivilegeManager {
    /// Check if sudo is available on the current system
    pub fn is_sudo_available() -> bool {
        std::process::Command::new("sudo")
            .arg("-n")
            .arg("true")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if the current user has sudo privileges
    pub fn check_sudo_privileges() -> bool {
        // First check if sudo command exists
        if !Self::command_exists("sudo") {
            return false;
        }

        // Then check if we can run sudo without password
        std::process::Command::new("sudo")
            .arg("-n")
            .arg("true")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if a command exists in PATH
    fn command_exists(command: &str) -> bool {
        std::process::Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Generate test contexts for all required privilege levels
    pub fn generate_test_contexts(
        os: OperatingSystem,
        architecture: Architecture,
        enable_privilege_testing: bool,
    ) -> Vec<TestContext> {
        let mut contexts = Vec::new();
        let base_id = Uuid::new_v4().to_string();

        // Always test normal privilege level
        contexts.push(TestContext {
            os: os.clone(),
            architecture: architecture.clone(),
            privilege_level: PrivilegeLevel::Normal,
            test_id: format!("{}-normal", base_id),
            timestamp: Utc::now(),
        });

        // Test escalated privileges if enabled and available
        if enable_privilege_testing && Self::is_sudo_available() {
            contexts.push(TestContext {
                os,
                architecture,
                privilege_level: PrivilegeLevel::Escalated,
                test_id: format!("{}-escalated", base_id),
                timestamp: Utc::now(),
            });
        }

        contexts
    }

    /// Validate privilege requirements for a test context
    pub fn validate_context(context: &TestContext) -> Result<()> {
        match context.privilege_level {
            PrivilegeLevel::Normal => {
                // Normal privileges always work
                Ok(())
            }
            PrivilegeLevel::Escalated => {
                if !Self::is_sudo_available() {
                    return Err(anyhow::anyhow!(
                        "Escalated privileges requested but sudo is not available"
                    ));
                }
                Ok(())
            }
        }
    }

    /// Get a description of the current privilege capabilities
    pub fn get_privilege_info() -> PrivilegeInfo {
        PrivilegeInfo {
            sudo_command_exists: Self::command_exists("sudo"),
            sudo_available: Self::is_sudo_available(),
            sudo_privileges: Self::check_sudo_privileges(),
            current_user: Self::get_current_user(),
            effective_uid: Self::get_effective_uid(),
        }
    }

    /// Get the current username
    fn get_current_user() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Safely get effective user ID
    fn geteuid_safe() -> u32 {
        #[cfg(unix)]
        unsafe {
            libc::geteuid()
        }
        #[cfg(not(unix))]
        0
    }

    /// Get the effective user ID
    fn get_effective_uid() -> u32 {
        #[cfg(unix)]
        {
            Self::geteuid_safe()
        }
        #[cfg(not(unix))]
        {
            0 // Default for non-Unix systems
        }
    }

    /// Generate a comprehensive test matrix for privilege testing
    pub fn generate_privilege_test_matrix(
        target_os: &[OperatingSystem],
        target_architectures: &[Architecture],
        enable_privilege_testing: bool,
    ) -> Vec<TestContext> {
        let mut contexts = Vec::new();

        for os in target_os {
            for arch in target_architectures {
                let mut os_arch_contexts = Self::generate_test_contexts(
                    os.clone(),
                    arch.clone(),
                    enable_privilege_testing,
                );
                contexts.append(&mut os_arch_contexts);
            }
        }

        contexts
    }

    /// Analyze privilege differences between two test results
    pub fn analyze_privilege_differences(
        normal_result: &crate::types::TestResult,
        escalated_result: &crate::types::TestResult,
    ) -> Result<PrivilegeDifferenceAnalysis> {
        if normal_result.context.privilege_level != PrivilegeLevel::Normal {
            return Err(anyhow::anyhow!("First result must be from normal privilege execution"));
        }

        if escalated_result.context.privilege_level != PrivilegeLevel::Escalated {
            return Err(anyhow::anyhow!("Second result must be from escalated privilege execution"));
        }

        let normal_output = normal_result.output.as_ref();
        let escalated_output = escalated_result.output.as_ref();

        let both_successful = normal_result.success && escalated_result.success;
        
        let privilege_metadata_differs = if let (Some(normal), Some(escalated)) = (normal_output, escalated_output) {
            normal.collection_metadata.sudo_support_enabled != escalated.collection_metadata.sudo_support_enabled
                || normal.collection_metadata.sudo_available != escalated.collection_metadata.sudo_available
        } else {
            false
        };

        let additional_data_with_privileges = if let (Some(normal), Some(escalated)) = (normal_output, escalated_output) {
            escalated.plugins.len() > normal.plugins.len()
        } else {
            false
        };

        let analysis = PrivilegeDifferenceAnalysis {
            both_executions_successful: both_successful,
            privilege_metadata_differs,
            additional_data_with_privileges,
            normal_execution_time_ms: normal_result.execution_time_ms,
            escalated_execution_time_ms: escalated_result.execution_time_ms,
            performance_impact: calculate_performance_impact(
                normal_result.execution_time_ms,
                escalated_result.execution_time_ms,
            ),
            normal_plugin_count: normal_output.map(|o| o.collection_metadata.plugin_count).unwrap_or(0),
            escalated_plugin_count: escalated_output.map(|o| o.collection_metadata.plugin_count).unwrap_or(0),
        };

        Ok(analysis)
    }
}

/// Information about privilege capabilities on the current system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeInfo {
    pub sudo_command_exists: bool,
    pub sudo_available: bool,
    pub sudo_privileges: bool,
    pub current_user: String,
    pub effective_uid: u32,
}

impl std::fmt::Display for PrivilegeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Privilege Information:")?;
        writeln!(f, "  Current User: {}", self.current_user)?;
        writeln!(f, "  Effective UID: {}", self.effective_uid)?;
        writeln!(f, "  Sudo Command Exists: {}", self.sudo_command_exists)?;
        writeln!(f, "  Sudo Available: {}", self.sudo_available)?;
        writeln!(f, "  Sudo Privileges: {}", self.sudo_privileges)?;
        Ok(())
    }
}

/// Analysis of differences between privileged and non-privileged execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeDifferenceAnalysis {
    pub both_executions_successful: bool,
    pub privilege_metadata_differs: bool,
    pub additional_data_with_privileges: bool,
    pub normal_execution_time_ms: u64,
    pub escalated_execution_time_ms: u64,
    pub performance_impact: PerformanceImpact,
    pub normal_plugin_count: u32,
    pub escalated_plugin_count: u32,
}

/// Performance impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceImpact {
    Negligible(f64),  // Less than 10% difference
    Moderate(f64),    // 10-50% difference
    Significant(f64), // More than 50% difference
}

impl std::fmt::Display for PerformanceImpact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceImpact::Negligible(pct) => write!(f, "Negligible ({:.1}%)", pct),
            PerformanceImpact::Moderate(pct) => write!(f, "Moderate ({:.1}%)", pct),
            PerformanceImpact::Significant(pct) => write!(f, "Significant ({:.1}%)", pct),
        }
    }
}

fn calculate_performance_impact(normal_ms: u64, escalated_ms: u64) -> PerformanceImpact {
    if normal_ms == 0 {
        return PerformanceImpact::Negligible(0.0);
    }

    let difference_pct = ((escalated_ms as f64 - normal_ms as f64) / normal_ms as f64) * 100.0;
    let abs_difference = difference_pct.abs();

    if abs_difference < 10.0 {
        PerformanceImpact::Negligible(difference_pct)
    } else if abs_difference < 50.0 {
        PerformanceImpact::Moderate(difference_pct)
    } else {
        PerformanceImpact::Significant(difference_pct)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_contexts_normal_only() {
        let contexts = PrivilegeManager::generate_test_contexts(
            OperatingSystem::Ubuntu,
            Architecture::X86_64,
            false,
        );

        assert_eq!(contexts.len(), 1);
        assert_eq!(contexts[0].privilege_level, PrivilegeLevel::Normal);
        assert_eq!(contexts[0].os, OperatingSystem::Ubuntu);
        assert_eq!(contexts[0].architecture, Architecture::X86_64);
    }

    #[test]
    fn test_generate_privilege_test_matrix() {
        let os_list = vec![OperatingSystem::Ubuntu, OperatingSystem::Alpine];
        let arch_list = vec![Architecture::X86_64, Architecture::Arm64];

        let contexts = PrivilegeManager::generate_privilege_test_matrix(
            &os_list,
            &arch_list,
            false, // No privilege testing
        );

        // Should have 4 combinations (2 OS × 2 Arch × 1 privilege level each)
        assert_eq!(contexts.len(), 4);

        // Check that all combinations are present
        let mut combinations = std::collections::HashSet::new();
        for context in &contexts {
            combinations.insert((
                context.os.clone(),
                context.architecture.clone(),
                context.privilege_level,
            ));
        }

        assert_eq!(combinations.len(), 4);
        assert!(combinations.contains(&(OperatingSystem::Ubuntu, Architecture::X86_64, PrivilegeLevel::Normal)));
        assert!(combinations.contains(&(OperatingSystem::Ubuntu, Architecture::Arm64, PrivilegeLevel::Normal)));
        assert!(combinations.contains(&(OperatingSystem::Alpine, Architecture::X86_64, PrivilegeLevel::Normal)));
        assert!(combinations.contains(&(OperatingSystem::Alpine, Architecture::Arm64, PrivilegeLevel::Normal)));
    }

    #[test]
    fn test_calculate_performance_impact() {
        // Test negligible impact
        let impact = calculate_performance_impact(1000, 1050);
        match impact {
            PerformanceImpact::Negligible(pct) => assert!((pct - 5.0).abs() < 0.1),
            _ => panic!("Expected negligible impact"),
        }

        // Test moderate impact
        let impact = calculate_performance_impact(1000, 1300);
        match impact {
            PerformanceImpact::Moderate(pct) => assert!((pct - 30.0).abs() < 0.1),
            _ => panic!("Expected moderate impact"),
        }

        // Test significant impact
        let impact = calculate_performance_impact(1000, 2000);
        match impact {
            PerformanceImpact::Significant(pct) => assert!((pct - 100.0).abs() < 0.1),
            _ => panic!("Expected significant impact"),
        }
    }

    #[test]
    fn test_validate_context_normal() {
        let context = TestContext {
            os: OperatingSystem::Ubuntu,
            architecture: Architecture::X86_64,
            privilege_level: PrivilegeLevel::Normal,
            test_id: "test".to_string(),
            timestamp: Utc::now(),
        };

        let result = PrivilegeManager::validate_context(&context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_current_user() {
        let user = PrivilegeManager::get_current_user();
        assert!(!user.is_empty());
        assert_ne!(user, "unknown");
    }
}