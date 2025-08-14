//! Script execution module for running collect_info.sh with different privilege levels

use crate::types::{PrivilegeLevel, ScriptOutput, TestContext, TestResult};
use crate::Result;
use std::process::Command;
use std::time::Instant;
use tokio::time::timeout;
use std::time::Duration;

/// Handles execution of the collect_info.sh script
pub struct ScriptExecutor {
    script_path: String,
    timeout_duration: Duration,
    max_retries: u32,
}

impl ScriptExecutor {
    /// Create a new script executor
    pub fn new(script_path: String, timeout_seconds: u64, max_retries: u32) -> Self {
        Self {
            script_path,
            timeout_duration: Duration::from_secs(timeout_seconds),
            max_retries,
        }
    }

    /// Execute the script with the given test context
    pub async fn execute(&self, context: &TestContext) -> Result<TestResult> {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retries {
            log::info!(
                "Executing test {} (attempt {}/{}) - OS: {}, Arch: {}, Privilege: {}",
                context.test_id,
                attempt,
                self.max_retries,
                context.os,
                context.architecture,
                context.privilege_level
            );

            match self.execute_once(context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    log::warn!(
                        "Test {} attempt {}/{} failed: {}",
                        context.test_id,
                        attempt,
                        self.max_retries,
                        e
                    );
                    last_error = Some(e);
                    
                    if attempt < self.max_retries {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }

        // If all retries failed, return the last error
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Execute the script once
    async fn execute_once(&self, context: &TestContext) -> Result<TestResult> {
        let start_time = Instant::now();
        
        let mut cmd = self.build_command(context)?;
        
        log::debug!("Executing command: {:?}", cmd);

        let output = timeout(self.timeout_duration, async {
            tokio::task::spawn_blocking(move || cmd.output())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to spawn command: {}", e))
                .and_then(|result| result.map_err(|e| anyhow::anyhow!("Command execution failed: {}", e)))
        })
        .await
        .map_err(|_| anyhow::anyhow!("Command timed out after {:?}", self.timeout_duration))??;

        let execution_time = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        let parsed_output = if success && !stdout.trim().is_empty() {
            match self.parse_output(&stdout) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    log::warn!("Failed to parse script output: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let error_message = if !success {
            Some(format!(
                "Script failed with exit code: {}, stderr: {}",
                output.status.code().unwrap_or(-1),
                stderr
            ))
        } else if parsed_output.is_none() && !stdout.trim().is_empty() {
            Some("Failed to parse JSON output".to_string())
        } else {
            None
        };

        Ok(TestResult {
            context: context.clone(),
            success: success && parsed_output.is_some(),
            output: parsed_output,
            execution_time_ms: execution_time.as_millis() as u64,
            error_message,
            stdout,
            stderr,
        })
    }

    /// Build the command to execute based on the test context
    fn build_command(&self, context: &TestContext) -> Result<Command> {
        let mut cmd = Command::new("bash");
        cmd.arg(&self.script_path);

        // Set environment variables based on privilege level
        match context.privilege_level {
            PrivilegeLevel::Normal => {
                cmd.env("ENABLE_SUDO_SUPPORT", "0");
            }
            PrivilegeLevel::Escalated => {
                cmd.env("ENABLE_SUDO_SUPPORT", "1");
            }
        }

        // Always enable hashing for consistent testing
        cmd.env("ENABLE_HASHING", "1");

        // Set working directory to the script's directory
        if let Some(parent) = std::path::Path::new(&self.script_path).parent() {
            cmd.current_dir(parent);
        }

        Ok(cmd)
    }

    /// Parse the JSON output from the script
    fn parse_output(&self, output: &str) -> Result<ScriptOutput> {
        // Clean up the output - remove any non-JSON lines at the beginning
        let json_start = output.find('{').ok_or_else(|| {
            anyhow::anyhow!("No JSON object found in output")
        })?;
        
        let json_str = &output[json_start..];
        
        // Find the end of the JSON object by counting braces
        let mut brace_count = 0;
        let mut json_end = json_str.len();
        
        for (i, ch) in json_str.char_indices() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        json_end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        let clean_json = &json_str[..json_end];
        
        serde_json::from_str(clean_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
    }

    /// Validate that the script exists and is executable
    pub fn validate_script(&self) -> Result<()> {
        let path = std::path::Path::new(&self.script_path);
        
        if !path.exists() {
            return Err(anyhow::anyhow!("Script does not exist: {}", self.script_path));
        }
        
        if !path.is_file() {
            return Err(anyhow::anyhow!("Script path is not a file: {}", self.script_path));
        }

        // Check if the file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = path.metadata()
                .map_err(|e| anyhow::anyhow!("Failed to read script metadata: {}", e))?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return Err(anyhow::anyhow!("Script is not executable: {}", self.script_path));
            }
        }

        Ok(())
    }

    /// Get the script path
    pub fn script_path(&self) -> &str {
        &self.script_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Architecture;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_script_executor_creation() {
        let executor = ScriptExecutor::new(
            "./test_script.sh".to_string(),
            60,
            3
        );
        
        assert_eq!(executor.script_path(), "./test_script.sh");
        assert_eq!(executor.timeout_duration, Duration::from_secs(60));
        assert_eq!(executor.max_retries, 3);
    }

    #[tokio::test]
    async fn test_parse_output() {
        let executor = ScriptExecutor::new("./test.sh".to_string(), 60, 1);
        
        let test_json = r#"{"detected_architecture":"x86_64","collection_metadata":{"timestamp":"2025-01-01T00:00:00Z","plugin_count":1,"hashing_enabled":true,"sudo_support_enabled":false,"sudo_available":false}}"#;
        
        let result = executor.parse_output(test_json);
        assert!(result.is_ok());
        
        let parsed = result.unwrap();
        assert_eq!(parsed.detected_architecture, Architecture::X86_64);
        assert_eq!(parsed.collection_metadata.plugin_count, 1);
    }

    #[tokio::test]
    async fn test_parse_output_with_extra_text() {
        let executor = ScriptExecutor::new("./test.sh".to_string(), 60, 1);
        
        let test_output = r#"Some debug output
        {"detected_architecture":"x86_64","collection_metadata":{"timestamp":"2025-01-01T00:00:00Z","plugin_count":1,"hashing_enabled":true,"sudo_support_enabled":false,"sudo_available":false}}
        More text after"#;
        
        let result = executor.parse_output(test_output);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_script_nonexistent() {
        let executor = ScriptExecutor::new("./nonexistent.sh".to_string(), 60, 1);
        let result = executor.validate_script();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_script_existing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "#!/bin/bash\necho 'test'").unwrap();
        
        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = temp_file.as_file().metadata().unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(temp_file.path(), perms).unwrap();
        }

        let executor = ScriptExecutor::new(
            temp_file.path().to_string_lossy().to_string(),
            60,
            1
        );
        
        let result = executor.validate_script();
        assert!(result.is_ok());
    }
}