//! Output validation and comparison module

use crate::types::{ComparisonResult, DataDifference, DifferenceType, ScriptOutput, TestResult};
use crate::Result;
use std::collections::HashMap;

/// Validates script outputs and performs comparisons
pub struct OutputValidator;

impl OutputValidator {
    /// Validate that a script output is well-formed and contains expected data
    pub fn validate_output(output: &ScriptOutput) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check that detected architecture is valid
        if output.detected_architecture.to_string().is_empty() {
            issues.push("Detected architecture is empty".to_string());
        }

        // Check collection metadata
        if output.collection_metadata.plugin_count == 0 {
            issues.push("No plugins were executed".to_string());
        }

        if output.collection_metadata.timestamp.timestamp() < Self::MIN_VALID_TIMESTAMP {
            issues.push("Invalid collection timestamp".to_string());
        }

        // Check that we have plugin data
        if output.plugins.is_empty() {
            issues.push("No plugin data found".to_string());
        }

        // Validate each plugin's data
        for (plugin_name, plugin_data) in &output.plugins {
            if plugin_name.is_empty() {
                issues.push("Plugin name is empty".to_string());
            }

            if plugin_data.data.is_null() {
                issues.push(format!("Plugin '{}' has null data", plugin_name));
            }

            if plugin_data.collection_timestamp.timestamp() < 0 {
                issues.push(format!("Plugin '{}' has invalid collection timestamp", plugin_name));
            }

            if plugin_data.completion_timestamp.timestamp() < 0 {
                issues.push(format!("Plugin '{}' has invalid completion timestamp", plugin_name));
            }

            if plugin_data.collection_timestamp > plugin_data.completion_timestamp {
                issues.push(format!(
                    "Plugin '{}' collection timestamp is after completion timestamp",
                    plugin_name
                ));
            }

            // Validate hash fields if hashing is enabled
            if plugin_data.plugin_file_hash != "disabled" && plugin_data.plugin_file_hash.is_empty() {
                issues.push(format!("Plugin '{}' has empty file hash", plugin_name));
            }

            if plugin_data.function_data_hash != "disabled" && plugin_data.function_data_hash.is_empty() {
                issues.push(format!("Plugin '{}' has empty data hash", plugin_name));
            }
        }

        // Check for expected core plugins
        let expected_plugins = vec![
            "get_os_info",
            "get_hardware_info", 
            "get_ip_info",
            "get_uptime_info",
        ];

        for expected in &expected_plugins {
            if !output.plugins.contains_key(*expected) {
                issues.push(format!("Missing expected plugin: {}", expected));
            }
        }

        Ok(issues)
    }

    /// Compare two script outputs and identify differences
    pub fn compare_outputs(
        result_a: &TestResult,
        result_b: &TestResult,
    ) -> Result<ComparisonResult> {
        let output_a = result_a.output.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Test result A has no output")
        })?;
        
        let output_b = result_b.output.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Test result B has no output")
        })?;

        let mut differences = Vec::new();

        // Compare architectures
        let architecture_matches = output_a.detected_architecture == output_b.detected_architecture;
        if !architecture_matches {
            differences.push(DataDifference {
                plugin_name: "metadata".to_string(),
                field_path: "detected_architecture".to_string(),
                value_a: output_a.detected_architecture.to_string(),
                value_b: output_b.detected_architecture.to_string(),
                difference_type: DifferenceType::ValueChanged,
            });
        }

        // Compare plugin counts
        let plugin_count_matches = output_a.collection_metadata.plugin_count == output_b.collection_metadata.plugin_count;
        if !plugin_count_matches {
            differences.push(DataDifference {
                plugin_name: "metadata".to_string(),
                field_path: "plugin_count".to_string(),
                value_a: output_a.collection_metadata.plugin_count.to_string(),
                value_b: output_b.collection_metadata.plugin_count.to_string(),
                difference_type: DifferenceType::ValueChanged,
            });
        }

        // Check privilege level difference
        let privilege_level_difference = result_a.context.privilege_level != result_b.context.privilege_level;

        // Compare privilege-related metadata
        if output_a.collection_metadata.sudo_support_enabled != output_b.collection_metadata.sudo_support_enabled {
            differences.push(DataDifference {
                plugin_name: "metadata".to_string(),
                field_path: "sudo_support_enabled".to_string(),
                value_a: output_a.collection_metadata.sudo_support_enabled.to_string(),
                value_b: output_b.collection_metadata.sudo_support_enabled.to_string(),
                difference_type: DifferenceType::PrivilegeRelated,
            });
        }

        if output_a.collection_metadata.sudo_available != output_b.collection_metadata.sudo_available {
            differences.push(DataDifference {
                plugin_name: "metadata".to_string(),
                field_path: "sudo_available".to_string(),
                value_a: output_a.collection_metadata.sudo_available.to_string(),
                value_b: output_b.collection_metadata.sudo_available.to_string(),
                difference_type: DifferenceType::PrivilegeRelated,
            });
        }

        // Compare plugin data
        let all_plugin_names: std::collections::HashSet<_> = output_a
            .plugins
            .keys()
            .chain(output_b.plugins.keys())
            .collect();

        for plugin_name in all_plugin_names {
            match (output_a.plugins.get(plugin_name), output_b.plugins.get(plugin_name)) {
                (Some(data_a), Some(data_b)) => {
                    // Both have the plugin, compare data
                    Self::compare_plugin_data(plugin_name, data_a, data_b, &mut differences)?;
                }
                (Some(_), None) => {
                    differences.push(DataDifference {
                        plugin_name: plugin_name.clone(),
                        field_path: "plugin".to_string(),
                        value_a: "present".to_string(),
                        value_b: "missing".to_string(),
                        difference_type: DifferenceType::FieldMissing,
                    });
                }
                (None, Some(_)) => {
                    differences.push(DataDifference {
                        plugin_name: plugin_name.clone(),
                        field_path: "plugin".to_string(),
                        value_a: "missing".to_string(),
                        value_b: "present".to_string(),
                        difference_type: DifferenceType::FieldAdded,
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        let summary = Self::generate_summary(&differences, privilege_level_difference);

        Ok(ComparisonResult {
            test_a: result_a.context.test_id.clone(),
            test_b: result_b.context.test_id.clone(),
            architecture_matches,
            plugin_count_matches,
            privilege_level_difference,
            data_differences: differences,
            summary,
        })
    }

    /// Compare data from two plugins
    fn compare_plugin_data(
        plugin_name: &str,
        data_a: &crate::types::PluginData,
        data_b: &crate::types::PluginData,
        differences: &mut Vec<DataDifference>,
    ) -> Result<()> {
        // Compare the actual data payloads
        Self::compare_json_values(
            plugin_name,
            "data",
            &data_a.data,
            &data_b.data,
            differences,
        )?;

        // Compare hashes (but allow for expected differences)
        if data_a.plugin_file_hash != data_b.plugin_file_hash
            && data_a.plugin_file_hash != "disabled"
            && data_b.plugin_file_hash != "disabled"
        {
            differences.push(DataDifference {
                plugin_name: plugin_name.to_string(),
                field_path: "plugin_file_hash".to_string(),
                value_a: data_a.plugin_file_hash.clone(),
                value_b: data_b.plugin_file_hash.clone(),
                difference_type: DifferenceType::ValueChanged,
            });
        }

        // Function data hashes may differ due to timestamps
        if data_a.function_data_hash != data_b.function_data_hash
            && data_a.function_data_hash != "disabled"
            && data_b.function_data_hash != "disabled"
        {
            differences.push(DataDifference {
                plugin_name: plugin_name.to_string(),
                field_path: "function_data_hash".to_string(),
                value_a: data_a.function_data_hash.clone(),
                value_b: data_b.function_data_hash.clone(),
                difference_type: DifferenceType::ValueChanged,
            });
        }

        Ok(())
    }

    /// Recursively compare JSON values
    fn compare_json_values(
        plugin_name: &str,
        path: &str,
        value_a: &serde_json::Value,
        value_b: &serde_json::Value,
        differences: &mut Vec<DataDifference>,
    ) -> Result<()> {
        use serde_json::Value;

        match (value_a, value_b) {
            (Value::Object(obj_a), Value::Object(obj_b)) => {
                let all_keys: std::collections::HashSet<_> = obj_a.keys().chain(obj_b.keys()).collect();
                
                for key in all_keys {
                    let new_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                    
                    match (obj_a.get(key), obj_b.get(key)) {
                        (Some(val_a), Some(val_b)) => {
                            Self::compare_json_values(plugin_name, &new_path, val_a, val_b, differences)?;
                        }
                        (Some(_), None) => {
                            differences.push(DataDifference {
                                plugin_name: plugin_name.to_string(),
                                field_path: new_path,
                                value_a: "present".to_string(),
                                value_b: "missing".to_string(),
                                difference_type: DifferenceType::FieldMissing,
                            });
                        }
                        (None, Some(_)) => {
                            differences.push(DataDifference {
                                plugin_name: plugin_name.to_string(),
                                field_path: new_path,
                                value_a: "missing".to_string(),
                                value_b: "present".to_string(),
                                difference_type: DifferenceType::FieldAdded,
                            });
                        }
                        (None, None) => unreachable!(),
                    }
                }
            }
            (Value::Array(arr_a), Value::Array(arr_b)) => {
                // For arrays, just compare lengths and some sample values
                if arr_a.len() != arr_b.len() {
                    differences.push(DataDifference {
                        plugin_name: plugin_name.to_string(),
                        field_path: format!("{}.length", path),
                        value_a: arr_a.len().to_string(),
                        value_b: arr_b.len().to_string(),
                        difference_type: DifferenceType::ValueChanged,
                    });
                }
            }
            (a, b) if a != b => {
                // Skip timestamp differences as they're expected
                if !path.contains("timestamp") && !path.contains("uptime") && !path.contains("load_average") {
                    differences.push(DataDifference {
                        plugin_name: plugin_name.to_string(),
                        field_path: path.to_string(),
                        value_a: a.to_string(),
                        value_b: b.to_string(),
                        difference_type: DifferenceType::ValueChanged,
                    });
                }
            }
            _ => {
                // Values are equal, no difference
            }
        }

        Ok(())
    }

    /// Generate a summary of the differences
    fn generate_summary(differences: &[DataDifference], privilege_difference: bool) -> String {
        if differences.is_empty() {
            return "No significant differences found".to_string();
        }

        let mut summary = format!("Found {} differences", differences.len());
        
        if privilege_difference {
            summary.push_str(" (privilege level comparison)");
        }

        let privilege_related = differences.iter().filter(|d| matches!(d.difference_type, DifferenceType::PrivilegeRelated)).count();
        let value_changes = differences.iter().filter(|d| matches!(d.difference_type, DifferenceType::ValueChanged)).count();
        let missing_fields = differences.iter().filter(|d| matches!(d.difference_type, DifferenceType::FieldMissing)).count();
        let added_fields = differences.iter().filter(|d| matches!(d.difference_type, DifferenceType::FieldAdded)).count();

        if privilege_related > 0 {
            summary.push_str(&format!(", {} privilege-related", privilege_related));
        }
        if value_changes > 0 {
            summary.push_str(&format!(", {} value changes", value_changes));
        }
        if missing_fields > 0 {
            summary.push_str(&format!(", {} missing fields", missing_fields));
        }
        if added_fields > 0 {
            summary.push_str(&format!(", {} added fields", added_fields));
        }

        summary
    }

    /// Check if outputs are functionally equivalent (ignoring timestamps and privilege differences)
    pub fn are_functionally_equivalent(comparison: &ComparisonResult) -> bool {
        comparison.data_differences.iter().all(|diff| {
            matches!(diff.difference_type, DifferenceType::PrivilegeRelated)
                || diff.field_path.contains("timestamp")
                || diff.field_path.contains("uptime")
                || diff.field_path.contains("load_average")
                || diff.field_path.contains("hash")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;
    use serde_json::json;

    fn create_test_output() -> ScriptOutput {
        ScriptOutput {
            detected_architecture: Architecture::X86_64,
            collection_metadata: CollectionMetadata {
                timestamp: Utc::now(),
                plugin_count: 4,
                hashing_enabled: true,
                sudo_support_enabled: false,
                sudo_available: false,
            },
            plugins: {
                let mut plugins = HashMap::new();
                plugins.insert("get_os_info".to_string(), PluginData {
                    data: json!({"os_name": "Ubuntu", "architecture": "x86_64"}),
                    collection_timestamp: Utc::now(),
                    completion_timestamp: Utc::now(),
                    plugin_file_hash: "123456".to_string(),
                    function_data_hash: "789012".to_string(),
                });
                plugins.insert("get_hardware_info".to_string(), PluginData {
                    data: json!({"cpu_model": "Test CPU", "memory_total": "8GB"}),
                    collection_timestamp: Utc::now(),
                    completion_timestamp: Utc::now(),
                    plugin_file_hash: "234567".to_string(),
                    function_data_hash: "890123".to_string(),
                });
                plugins.insert("get_ip_info".to_string(), PluginData {
                    data: json!({"network_interfaces": []}),
                    collection_timestamp: Utc::now(),
                    completion_timestamp: Utc::now(),
                    plugin_file_hash: "345678".to_string(),
                    function_data_hash: "901234".to_string(),
                });
                plugins.insert("get_uptime_info".to_string(), PluginData {
                    data: json!({"uptime_seconds": 3600}),
                    collection_timestamp: Utc::now(),
                    completion_timestamp: Utc::now(),
                    plugin_file_hash: "456789".to_string(),
                    function_data_hash: "012345".to_string(),
                });
                plugins
            },
        }
    }

    #[test]
    fn test_validate_output_success() {
        let output = create_test_output();
        let issues = OutputValidator::validate_output(&output).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_output_missing_plugin() {
        let mut output = create_test_output();
        output.plugins.clear();
        
        let issues = OutputValidator::validate_output(&output).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.contains("No plugin data found")));
    }

    #[test]
    fn test_compare_identical_outputs() {
        let output = create_test_output();
        let context = TestContext {
            os: OperatingSystem::Ubuntu,
            architecture: Architecture::X86_64,
            privilege_level: PrivilegeLevel::Normal,
            test_id: "test1".to_string(),
            timestamp: Utc::now(),
        };
        
        let result_a = TestResult {
            context: context.clone(),
            success: true,
            output: Some(output.clone()),
            execution_time_ms: 1000,
            error_message: None,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };
        
        let result_b = TestResult {
            context,
            success: true,
            output: Some(output),
            execution_time_ms: 1000,
            error_message: None,
            stdout: "".to_string(),
            stderr: "".to_string(),
        };

        let comparison = OutputValidator::compare_outputs(&result_a, &result_b).unwrap();
        assert!(comparison.architecture_matches);
        assert!(comparison.plugin_count_matches);
        assert!(!comparison.privilege_level_difference);
    }
}