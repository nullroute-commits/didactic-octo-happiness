//! Test reporting and result analysis

use crate::types::{ComparisonResult, TestResult, TestSuiteConfig};
use crate::privilege::{PrivilegeDifferenceAnalysis, PrivilegeInfo};
use crate::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Generates comprehensive test reports
pub struct TestReporter {
    config: TestSuiteConfig,
    output_dir: String,
}

impl TestReporter {
    /// Create a new test reporter
    pub fn new(config: TestSuiteConfig) -> Self {
        let output_dir = config.output_directory.clone();
        Self { config, output_dir }
    }

    /// Generate a comprehensive test report
    pub async fn generate_report(
        &self,
        results: &[TestResult],
        comparisons: &[ComparisonResult],
        privilege_analyses: &[PrivilegeDifferenceAnalysis],
        privilege_info: &PrivilegeInfo,
    ) -> Result<TestReport> {
        let report_timestamp = Utc::now();
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;

        // Analyze results
        let summary = self.generate_summary(results, comparisons, privilege_analyses);
        let detailed_results = self.organize_results(results);
        let regression_analysis = self.analyze_regression(comparisons);
        let privilege_analysis = self.analyze_privilege_results(privilege_analyses);

        let report = TestReport {
            metadata: ReportMetadata {
                generated_at: report_timestamp,
                test_suite_version: env!("CARGO_PKG_VERSION").to_string(),
                total_tests: results.len(),
                config: self.config.clone(),
                privilege_info: privilege_info.clone(),
            },
            summary,
            detailed_results,
            comparisons: comparisons.to_vec(),
            regression_analysis,
            privilege_analysis,
            recommendations: self.generate_recommendations(results, comparisons, privilege_analyses),
        };

        // Write report files
        self.write_report_files(&report).await?;

        Ok(report)
    }

    /// Generate test summary
    fn generate_summary(
        &self,
        results: &[TestResult],
        comparisons: &[ComparisonResult],
        privilege_analyses: &[PrivilegeDifferenceAnalysis],
    ) -> TestSummary {
        let total_tests = results.len();
        let successful_tests = results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;

        let execution_times: Vec<u64> = results.iter().map(|r| r.execution_time_ms).collect();
        let avg_execution_time = if !execution_times.is_empty() {
            execution_times.iter().sum::<u64>() / execution_times.len() as u64
        } else {
            0
        };

        let min_execution_time = execution_times.iter().min().copied().unwrap_or(0);
        let max_execution_time = execution_times.iter().max().copied().unwrap_or(0);

        let successful_comparisons = comparisons.iter().filter(|c| {
            crate::validator::OutputValidator::are_functionally_equivalent(c)
        }).count();

        let privilege_tests_count = privilege_analyses.len();
        let successful_privilege_tests = privilege_analyses.iter()
            .filter(|a| a.both_executions_successful)
            .count();

        TestSummary {
            total_tests,
            successful_tests,
            failed_tests,
            success_rate: (successful_tests as f64 / total_tests as f64) * 100.0,
            avg_execution_time_ms: avg_execution_time,
            min_execution_time_ms: min_execution_time,
            max_execution_time_ms: max_execution_time,
            total_comparisons: comparisons.len(),
            successful_comparisons,
            privilege_tests_count,
            successful_privilege_tests,
        }
    }

    /// Organize results by OS and architecture
    fn organize_results(&self, results: &[TestResult]) -> HashMap<String, Vec<TestResult>> {
        let mut organized = HashMap::new();

        for result in results {
            let key = format!("{}-{}", result.context.os, result.context.architecture);
            organized.entry(key).or_insert_with(Vec::new).push(result.clone());
        }

        organized
    }

    /// Analyze regression test results
    fn analyze_regression(&self, comparisons: &[ComparisonResult]) -> RegressionAnalysis {
        let total_comparisons = comparisons.len();
        let no_regressions = comparisons.iter()
            .filter(|c| crate::validator::OutputValidator::are_functionally_equivalent(c))
            .count();

        let regressions_found = total_comparisons - no_regressions;

        let critical_regressions = comparisons.iter()
            .filter(|c| {
                !c.architecture_matches || !c.plugin_count_matches ||
                c.data_differences.iter().any(|d| {
                    !matches!(d.difference_type, crate::types::DifferenceType::PrivilegeRelated)
                    && !d.field_path.contains("timestamp")
                    && !d.field_path.contains("hash")
                })
            })
            .count();

        RegressionAnalysis {
            total_comparisons,
            regressions_found,
            critical_regressions,
            regression_rate: (regressions_found as f64 / total_comparisons as f64) * 100.0,
            critical_regression_rate: (critical_regressions as f64 / total_comparisons as f64) * 100.0,
        }
    }

    /// Analyze privilege test results
    fn analyze_privilege_results(&self, analyses: &[PrivilegeDifferenceAnalysis]) -> PrivilegeAnalysisSummary {
        if analyses.is_empty() {
            return PrivilegeAnalysisSummary {
                total_privilege_tests: 0,
                successful_both_levels: 0,
                privilege_enhances_data: 0,
                average_performance_impact: 0.0,
                performance_concerns: Vec::new(),
            };
        }

        let total_privilege_tests = analyses.len();
        let successful_both_levels = analyses.iter()
            .filter(|a| a.both_executions_successful)
            .count();

        let privilege_enhances_data = analyses.iter()
            .filter(|a| a.additional_data_with_privileges)
            .count();

        let performance_impacts: Vec<f64> = analyses.iter()
            .map(|a| {
                let normal = a.normal_execution_time_ms as f64;
                let escalated = a.escalated_execution_time_ms as f64;
                if normal > 0.0 {
                    ((escalated - normal) / normal) * 100.0
                } else {
                    0.0
                }
            })
            .collect();

        let average_performance_impact = if !performance_impacts.is_empty() {
            performance_impacts.iter().sum::<f64>() / performance_impacts.len() as f64
        } else {
            0.0
        };

        let performance_concerns = analyses.iter()
            .enumerate()
            .filter_map(|(i, a)| {
                match &a.performance_impact {
                    crate::privilege::PerformanceImpact::Significant(pct) => {
                        Some(format!("Test {}: Significant performance impact ({}%)", i + 1, pct))
                    }
                    crate::privilege::PerformanceImpact::Moderate(pct) if *pct > 30.0 => {
                        Some(format!("Test {}: Moderate performance impact ({}%)", i + 1, pct))
                    }
                    _ => None
                }
            })
            .collect();

        PrivilegeAnalysisSummary {
            total_privilege_tests,
            successful_both_levels,
            privilege_enhances_data,
            average_performance_impact,
            performance_concerns,
        }
    }

    /// Generate recommendations based on test results
    fn generate_recommendations(
        &self,
        results: &[TestResult],
        comparisons: &[ComparisonResult],
        privilege_analyses: &[PrivilegeDifferenceAnalysis],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze failure patterns
        let failed_results: Vec<_> = results.iter().filter(|r| !r.success).collect();
        if !failed_results.is_empty() {
            let failure_rate = (failed_results.len() as f64 / results.len() as f64) * 100.0;
            if failure_rate > 10.0 {
                recommendations.push(format!(
                    "High failure rate detected ({:.1}%). Consider investigating common failure patterns.",
                    failure_rate
                ));
            }

            // Group failures by OS/Architecture
            let mut failure_groups = HashMap::new();
            for result in &failed_results {
                let key = format!("{}-{}", result.context.os, result.context.architecture);
                *failure_groups.entry(key).or_insert(0) += 1;
            }

            for (combo, count) in failure_groups {
                if count > 1 {
                    recommendations.push(format!(
                        "Multiple failures on {}: {} tests failed. May indicate platform-specific issues.",
                        combo, count
                    ));
                }
            }
        }

        // Analyze regression patterns
        let critical_regressions: Vec<_> = comparisons.iter()
            .filter(|c| !crate::validator::OutputValidator::are_functionally_equivalent(c))
            .collect();

        if !critical_regressions.is_empty() {
            recommendations.push(format!(
                "Found {} potential regressions. Review data differences for unexpected changes.",
                critical_regressions.len()
            ));
        }

        // Analyze privilege impact
        if !privilege_analyses.is_empty() {
            let significant_impacts: Vec<_> = privilege_analyses.iter()
                .filter(|a| matches!(a.performance_impact, crate::privilege::PerformanceImpact::Significant(_)))
                .collect();

            if !significant_impacts.is_empty() {
                recommendations.push(format!(
                    "Privilege escalation shows significant performance impact in {} tests. Consider optimizing privileged operations.",
                    significant_impacts.len()
                ));
            }

            let data_enhancement_count = privilege_analyses.iter()
                .filter(|a| a.additional_data_with_privileges)
                .count();

            if data_enhancement_count > 0 {
                recommendations.push(format!(
                    "Privilege escalation provides additional data in {} tests. This is expected behavior.",
                    data_enhancement_count
                ));
            }
        }

        // Performance recommendations
        let avg_time = if !results.is_empty() {
            results.iter().map(|r| r.execution_time_ms).sum::<u64>() / results.len() as u64
        } else {
            0
        };

        if avg_time > 30000 { // 30 seconds
            recommendations.push(
                "Average execution time is high. Consider optimizing script performance or increasing timeouts.".to_string()
            );
        }

        if recommendations.is_empty() {
            recommendations.push("All tests completed successfully with no significant issues detected.".to_string());
        }

        recommendations
    }

    /// Write report files to disk
    async fn write_report_files(&self, report: &TestReport) -> Result<()> {
        let output_path = Path::new(&self.output_dir);

        // Write JSON report
        let json_report = serde_json::to_string_pretty(report)?;
        let json_path = output_path.join("test_report.json");
        fs::write(json_path, json_report)?;

        // Write human-readable summary
        let summary_content = self.format_summary_report(report);
        let summary_path = output_path.join("test_summary.md");
        fs::write(summary_path, summary_content)?;

        // Write detailed results CSV
        let csv_content = self.format_csv_report(&report.detailed_results);
        let csv_path = output_path.join("test_results.csv");
        fs::write(csv_path, csv_content)?;

        log::info!("Test reports written to {}", self.output_dir);
        Ok(())
    }

    /// Format human-readable summary report
    fn format_summary_report(&self, report: &TestReport) -> String {
        let mut content = String::new();
        
        content.push_str("# CI Test Suite Report\n\n");
        content.push_str(&format!("Generated: {}\n", report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        content.push_str(&format!("Version: {}\n\n", report.metadata.test_suite_version));

        content.push_str("## Summary\n\n");
        content.push_str(&format!("- **Total Tests**: {}\n", report.summary.total_tests));
        content.push_str(&format!("- **Successful**: {} ({:.1}%)\n", report.summary.successful_tests, report.summary.success_rate));
        content.push_str(&format!("- **Failed**: {}\n", report.summary.failed_tests));
        content.push_str(&format!("- **Average Execution Time**: {}ms\n", report.summary.avg_execution_time_ms));
        content.push_str(&format!("- **Privilege Tests**: {}/{} successful\n", report.privilege_analysis.successful_both_levels, report.privilege_analysis.total_privilege_tests));

        content.push_str("\n## Regression Analysis\n\n");
        content.push_str(&format!("- **Total Comparisons**: {}\n", report.regression_analysis.total_comparisons));
        content.push_str(&format!("- **Regressions Found**: {} ({:.1}%)\n", report.regression_analysis.regressions_found, report.regression_analysis.regression_rate));
        content.push_str(&format!("- **Critical Regressions**: {} ({:.1}%)\n", report.regression_analysis.critical_regressions, report.regression_analysis.critical_regression_rate));

        content.push_str("\n## Privilege Analysis\n\n");
        if report.privilege_analysis.total_privilege_tests > 0 {
            content.push_str(&format!("- **Average Performance Impact**: {:.1}%\n", report.privilege_analysis.average_performance_impact));
            content.push_str(&format!("- **Data Enhancement Cases**: {}\n", report.privilege_analysis.privilege_enhances_data));
            
            if !report.privilege_analysis.performance_concerns.is_empty() {
                content.push_str("\n### Performance Concerns\n\n");
                for concern in &report.privilege_analysis.performance_concerns {
                    content.push_str(&format!("- {}\n", concern));
                }
            }
        } else {
            content.push_str("No privilege escalation tests performed.\n");
        }

        content.push_str("\n## Recommendations\n\n");
        for recommendation in &report.recommendations {
            content.push_str(&format!("- {}\n", recommendation));
        }

        content.push_str("\n## System Information\n\n");
        content.push_str(&format!("- **Current User**: {}\n", report.metadata.privilege_info.current_user));
        content.push_str(&format!("- **Effective UID**: {}\n", report.metadata.privilege_info.effective_uid));
        content.push_str(&format!("- **Sudo Available**: {}\n", report.metadata.privilege_info.sudo_available));

        content
    }

    /// Format CSV report
    fn format_csv_report(&self, detailed_results: &HashMap<String, Vec<TestResult>>) -> String {
        let mut content = String::new();
        content.push_str("OS,Architecture,Privilege,TestID,Success,ExecutionTimeMs,ErrorMessage\n");

        for (_combo, results) in detailed_results {
            for result in results {
                content.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    result.context.os,
                    result.context.architecture,
                    result.context.privilege_level,
                    result.context.test_id,
                    result.success,
                    result.execution_time_ms,
                    result.error_message.as_deref().unwrap_or("").replace(",", ";")
                ));
            }
        }

        content
    }
}

/// Complete test report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub metadata: ReportMetadata,
    pub summary: TestSummary,
    pub detailed_results: HashMap<String, Vec<TestResult>>,
    pub comparisons: Vec<ComparisonResult>,
    pub regression_analysis: RegressionAnalysis,
    pub privilege_analysis: PrivilegeAnalysisSummary,
    pub recommendations: Vec<String>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub generated_at: DateTime<Utc>,
    pub test_suite_version: String,
    pub total_tests: usize,
    pub config: TestSuiteConfig,
    pub privilege_info: PrivilegeInfo,
}

/// Test execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub avg_execution_time_ms: u64,
    pub min_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub total_comparisons: usize,
    pub successful_comparisons: usize,
    pub privilege_tests_count: usize,
    pub successful_privilege_tests: usize,
}

/// Regression analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub total_comparisons: usize,
    pub regressions_found: usize,
    pub critical_regressions: usize,
    pub regression_rate: f64,
    pub critical_regression_rate: f64,
}

/// Privilege analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeAnalysisSummary {
    pub total_privilege_tests: usize,
    pub successful_both_levels: usize,
    pub privilege_enhances_data: usize,
    pub average_performance_impact: f64,
    pub performance_concerns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use tempfile::TempDir;

    fn create_test_config() -> TestSuiteConfig {
        TestSuiteConfig {
            script_path: "./test.sh".to_string(),
            timeout_seconds: 60,
            parallel_execution: false,
            max_retries: 1,
            output_directory: "./test_output".to_string(),
            target_architectures: vec![Architecture::X86_64],
            target_operating_systems: vec![OperatingSystem::Ubuntu],
            enable_regression_testing: true,
            enable_privilege_comparison: true,
        }
    }

    #[tokio::test]
    async fn test_reporter_creation() {
        let config = create_test_config();
        let reporter = TestReporter::new(config.clone());
        assert_eq!(reporter.config.script_path, config.script_path);
    }

    #[test]
    fn test_generate_summary() {
        let config = create_test_config();
        let reporter = TestReporter::new(config);

        let results = vec![
            TestResult {
                context: TestContext {
                    os: OperatingSystem::Ubuntu,
                    architecture: Architecture::X86_64,
                    privilege_level: PrivilegeLevel::Normal,
                    test_id: "test1".to_string(),
                    timestamp: Utc::now(),
                },
                success: true,
                output: None,
                execution_time_ms: 1000,
                error_message: None,
                stdout: "".to_string(),
                stderr: "".to_string(),
            }
        ];

        let summary = reporter.generate_summary(&results, &[], &[]);
        assert_eq!(summary.total_tests, 1);
        assert_eq!(summary.successful_tests, 1);
        assert_eq!(summary.failed_tests, 0);
        assert_eq!(summary.success_rate, 100.0);
    }

    #[test]
    fn test_organize_results() {
        let config = create_test_config();
        let reporter = TestReporter::new(config);

        let results = vec![
            TestResult {
                context: TestContext {
                    os: OperatingSystem::Ubuntu,
                    architecture: Architecture::X86_64,
                    privilege_level: PrivilegeLevel::Normal,
                    test_id: "test1".to_string(),
                    timestamp: Utc::now(),
                },
                success: true,
                output: None,
                execution_time_ms: 1000,
                error_message: None,
                stdout: "".to_string(),
                stderr: "".to_string(),
            },
            TestResult {
                context: TestContext {
                    os: OperatingSystem::Alpine,
                    architecture: Architecture::Arm64,
                    privilege_level: PrivilegeLevel::Normal,
                    test_id: "test2".to_string(),
                    timestamp: Utc::now(),
                },
                success: true,
                output: None,
                execution_time_ms: 1200,
                error_message: None,
                stdout: "".to_string(),
                stderr: "".to_string(),
            }
        ];

        let organized = reporter.organize_results(&results);
        assert_eq!(organized.len(), 2);
        assert!(organized.contains_key("ubuntu-x86_64"));
        assert!(organized.contains_key("alpine-arm64"));
    }
}