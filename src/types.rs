//! Type definitions for the CI test suite

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Supported architectures based on the collect_info.sh script
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    #[serde(rename = "x86_64")]
    X86_64,
    #[serde(rename = "arm64")]
    Arm64,
    #[serde(rename = "i386")]
    I386,
    #[serde(rename = "ppc64le")]
    Ppc64le,
    #[serde(rename = "s390x")]
    S390x,
    #[serde(rename = "riscv64")]
    RiscV64,
    #[serde(rename = "mips64")]
    Mips64,
    #[serde(rename = "aarch32")]
    Aarch32,
    #[serde(rename = "sparc64")]
    Sparc64,
    #[serde(rename = "loongarch64")]
    LoongArch64,
}

impl Architecture {
    /// Get all supported architectures
    pub fn all() -> Vec<Architecture> {
        vec![
            Architecture::X86_64,
            Architecture::Arm64,
            Architecture::I386,
            Architecture::Ppc64le,
            Architecture::S390x,
            Architecture::RiscV64,
            Architecture::Mips64,
            Architecture::Aarch32,
            Architecture::Sparc64,
            Architecture::LoongArch64,
        ]
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Arm64 => write!(f, "arm64"),
            Architecture::I386 => write!(f, "i386"),
            Architecture::Ppc64le => write!(f, "ppc64le"),
            Architecture::S390x => write!(f, "s390x"),
            Architecture::RiscV64 => write!(f, "riscv64"),
            Architecture::Mips64 => write!(f, "mips64"),
            Architecture::Aarch32 => write!(f, "aarch32"),
            Architecture::Sparc64 => write!(f, "sparc64"),
            Architecture::LoongArch64 => write!(f, "loongarch64"),
        }
    }
}

/// Supported Unix operating systems for testing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatingSystem {
    Ubuntu,
    Alpine,
    CentOS,
    Rocky,
    Debian,
}

impl std::fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingSystem::Ubuntu => write!(f, "ubuntu"),
            OperatingSystem::Alpine => write!(f, "alpine"),
            OperatingSystem::CentOS => write!(f, "centos"),
            OperatingSystem::Rocky => write!(f, "rocky"),
            OperatingSystem::Debian => write!(f, "debian"),
        }
    }
}

/// Privilege level for script execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrivilegeLevel {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "escalated")]
    Escalated,
}

impl std::fmt::Display for PrivilegeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivilegeLevel::Normal => write!(f, "normal"),
            PrivilegeLevel::Escalated => write!(f, "escalated"),
        }
    }
}

/// Collection metadata from the script output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub timestamp: DateTime<Utc>,
    pub plugin_count: u32,
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    pub hashing_enabled: bool,
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    pub sudo_support_enabled: bool,
    #[serde(deserialize_with = "deserialize_bool_from_int")]
    pub sudo_available: bool,
}

/// Custom deserializer for boolean values that might be integers
fn deserialize_bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Unexpected, Visitor};
    use std::fmt;

    struct BoolFromIntVisitor;

    impl<'de> Visitor<'de> for BoolFromIntVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean or integer (0 or 1)")
        }

        fn visit_bool<E>(self, value: bool) -> Result<bool, E>
        where
            E: Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<bool, E>
        where
            E: Error,
        {
            match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(Error::invalid_value(Unexpected::Signed(value), &self)),
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<bool, E>
        where
            E: Error,
        {
            match value {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(Error::invalid_value(Unexpected::Unsigned(value), &self)),
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<bool, E>
        where
            E: Error,
        {
            match value {
                "0" | "false" => Ok(false),
                "1" | "true" => Ok(true),
                _ => Err(Error::invalid_value(Unexpected::Str(value), &self)),
            }
        }
    }

    deserializer.deserialize_any(BoolFromIntVisitor)
}

/// Plugin execution data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    pub data: serde_json::Value,
    pub collection_timestamp: DateTime<Utc>,
    pub completion_timestamp: DateTime<Utc>,
    pub plugin_file_hash: String,
    pub function_data_hash: String,
}

/// Complete script output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptOutput {
    pub detected_architecture: Architecture,
    pub collection_metadata: CollectionMetadata,
    #[serde(flatten)]
    pub plugins: HashMap<String, PluginData>,
}

/// Test execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestContext {
    pub os: OperatingSystem,
    pub architecture: Architecture,
    pub privilege_level: PrivilegeLevel,
    pub test_id: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub context: TestContext,
    pub success: bool,
    pub output: Option<ScriptOutput>,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

/// Comparison result between two test outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub test_a: String,
    pub test_b: String,
    pub architecture_matches: bool,
    pub plugin_count_matches: bool,
    pub privilege_level_difference: bool,
    pub data_differences: Vec<DataDifference>,
    pub summary: String,
}

/// Individual data difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDifference {
    pub plugin_name: String,
    pub field_path: String,
    pub value_a: String,
    pub value_b: String,
    pub difference_type: DifferenceType,
}

/// Type of difference detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceType {
    ValueChanged,
    FieldMissing,
    FieldAdded,
    TypeChanged,
    PrivilegeRelated,
}

/// Test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteConfig {
    pub script_path: String,
    pub timeout_seconds: u64,
    pub parallel_execution: bool,
    pub max_retries: u32,
    pub output_directory: String,
    pub target_architectures: Vec<Architecture>,
    pub target_operating_systems: Vec<OperatingSystem>,
    pub enable_regression_testing: bool,
    pub enable_privilege_comparison: bool,
}