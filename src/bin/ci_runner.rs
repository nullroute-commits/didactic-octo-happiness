//! CI Runner - Main binary for the CI test suite

use ci_test_suite::{
    Config, ScriptExecutor, OutputValidator, TestReporter, PrivilegeManager,
    types::*, os_support::OsSupport, Result,
    privilege::PrivilegeDifferenceAnalysis,
    reporter::TestReport,
};
use clap::{Parser, Subcommand};
use log::{info, warn, error};
use std::collections::HashMap;
use tokio::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full CI test suite
    Run {
        /// Configuration profile to use
        #[arg(short, long, default_value = "default")]
        profile: String,
        
        /// Script path to test
        #[arg(short, long)]
        script: Option<String>,
        
        /// Output directory for results
        #[arg(short, long)]
        output: Option<String>,
        
        /// Enable parallel execution
        #[arg(long)]
        parallel: bool,
        
        /// Skip privilege escalation tests
        #[arg(long)]
        skip_privilege: bool,
        
        /// Test only specific architectures (comma-separated)
        #[arg(long)]
        architectures: Option<String>,
        
        /// Test only specific operating systems (comma-separated)
        #[arg(long)]
        operating_systems: Option<String>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Test privilege escalation functionality
    Privilege {
        /// Script path to test
        #[arg(short, long)]
        script: Option<String>,
        
        /// Output directory for results
        #[arg(short, long)]
        output: Option<String>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Validate script output format
    Validate {
        /// Script path to test
        #[arg(short, long)]
        script: Option<String>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Show system information and capabilities
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Run { 
            profile, 
            script, 
            output, 
            parallel, 
            skip_privilege, 
            architectures,
            operating_systems,
            verbose 
        } => {
            init_logging(*verbose);
            run_test_suite(
                profile, 
                script.as_deref(), 
                output.as_deref(), 
                *parallel, 
                *skip_privilege,
                architectures.as_deref(),
                operating_systems.as_deref(),
            ).await
        }
        Commands::Privilege { script, output, verbose } => {
            init_logging(*verbose);
            run_privilege_test(script.as_deref(), output.as_deref()).await
        }
        Commands::Validate { script, verbose } => {
            init_logging(*verbose);
            validate_script_output(script.as_deref()).await
        }
        Commands::Info => {
            show_system_info().await
        }
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    std::env::set_var("RUST_LOG", level);
    env_logger::init();
}

/// Run the complete test suite
async fn run_test_suite(
    profile: &str,
    script_path: Option<&str>,
    output_dir: Option<&str>,
    parallel: bool,
    skip_privilege: bool,
    architectures: Option<&str>,
    operating_systems: Option<&str>,
) -> Result<()> {
    info!("Starting CI test suite with profile: {}", profile);
    
    let start_time = Instant::now();
    
    // Load configuration
    let mut config = match profile {
        "ci" => Config::for_ci(),
        "dev" => Config::for_development(),
        _ => Config::from_env(),
    };
    
    // Override with command line arguments
    if let Some(script) = script_path {
        config.script_path = script.to_string();
    }
    if let Some(output) = output_dir {
        config.output_directory = output.to_string();
    }
    config.parallel_execution = parallel;
    config.enable_privilege_comparison = !skip_privilege;
    
    // Parse architecture filter
    if let Some(arch_str) = architectures {
        let arch_names: Vec<&str> = arch_str.split(',').map(|s| s.trim()).collect();
        config.target_architectures = arch_names.iter()
            .filter_map(|name| parse_architecture(name))
            .collect();
    }
    
    // Parse OS filter
    if let Some(os_str) = operating_systems {
        let os_names: Vec<&str> = os_str.split(',').map(|s| s.trim()).collect();
        config.target_operating_systems = os_names.iter()
            .filter_map(|name| parse_operating_system(name))
            .collect();
    }
    
    // Validate configuration
    Config::validate(&config).map_err(|e| anyhow::anyhow!("Configuration error: {}", e))?;
    
    info!("Configuration loaded: {} architectures, {} operating systems", 
          config.target_architectures.len(), 
          config.target_operating_systems.len());
    
    // Initialize components
    let executor = ScriptExecutor::new(
        config.script_path.clone(),
        config.timeout_seconds,
        config.max_retries,
    );
    executor.validate_script()?;
    
    let reporter = TestReporter::new(config.clone());
    let privilege_info = PrivilegeManager::get_privilege_info();
    
    info!("Privilege capabilities: {}", privilege_info);
    
    // Generate test contexts
    let test_contexts = PrivilegeManager::generate_privilege_test_matrix(
        &config.target_operating_systems,
        &config.target_architectures,
        config.enable_privilege_comparison,
    );
    
    info!("Generated {} test contexts", test_contexts.len());
    
    // Execute tests
    let mut results = Vec::new();
    
    if config.parallel_execution {
        info!("Running tests in parallel");
        let futures: Vec<_> = test_contexts.iter()
            .map(|context| executor.execute(context))
            .collect();
        
        let parallel_results = futures::future::join_all(futures).await;
        for result in parallel_results {
            match result {
                Ok(test_result) => results.push(test_result),
                Err(e) => error!("Test execution failed: {}", e),
            }
        }
    } else {
        info!("Running tests sequentially");
        for context in &test_contexts {
            match executor.execute(context).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Test execution failed for {}: {}", context.test_id, e);
                    // Create a failed result entry
                    results.push(TestResult {
                        context: context.clone(),
                        success: false,
                        output: None,
                        execution_time_ms: 0,
                        error_message: Some(e.to_string()),
                        stdout: String::new(),
                        stderr: String::new(),
                    });
                }
            }
        }
    }
    
    info!("Completed {} tests in {:.2}s", results.len(), start_time.elapsed().as_secs_f64());
    
    // Perform comparisons and analysis
    let comparisons = if config.enable_regression_testing {
        perform_regression_analysis(&results).await?
    } else {
        Vec::new()
    };
    
    let privilege_analyses = if config.enable_privilege_comparison {
        perform_privilege_analysis(&results).await?
    } else {
        Vec::new()
    };
    
    // Generate report
    let report = reporter.generate_report(
        &results,
        &comparisons,
        &privilege_analyses,
        &privilege_info,
    ).await?;
    
    // Print summary
    print_test_summary(&report);
    
    // Exit with appropriate code
    let success_rate = report.summary.success_rate;
    if success_rate < 100.0 {
        warn!("Some tests failed (success rate: {:.1}%)", success_rate);
        std::process::exit(1);
    } else {
        info!("All tests passed successfully!");
        Ok(())
    }
}

/// Run privilege escalation test
async fn run_privilege_test(script_path: Option<&str>, output_dir: Option<&str>) -> Result<()> {
    info!("Running privilege escalation test");
    
    let script = script_path.unwrap_or("./collect_info.sh");
    let _output = output_dir.unwrap_or("./privilege_test_results");
    
    let executor = ScriptExecutor::new(script.to_string(), 300, 3);
    executor.validate_script()?;
    
    let privilege_info = PrivilegeManager::get_privilege_info();
    info!("System privilege information:");
    println!("{}", privilege_info);
    
    if !privilege_info.sudo_available {
        warn!("Sudo is not available - can only test normal privileges");
    }
    
    // Test both privilege levels
    let test_contexts = PrivilegeManager::generate_test_contexts(
        OperatingSystem::Ubuntu, // Use current OS
        Architecture::X86_64,    // Use current arch
        privilege_info.sudo_available,
    );
    
    let mut results = Vec::new();
    for context in test_contexts {
        match executor.execute(&context).await {
            Ok(result) => {
                info!("Test {} completed: success={}, time={}ms", 
                      context.test_id, result.success, result.execution_time_ms);
                results.push(result);
            }
            Err(e) => {
                error!("Test {} failed: {}", context.test_id, e);
            }
        }
    }
    
    // Analyze privilege differences if we have both
    if results.len() >= 2 {
        let normal_result = results.iter()
            .find(|r| r.context.privilege_level == PrivilegeLevel::Normal);
        let escalated_result = results.iter()
            .find(|r| r.context.privilege_level == PrivilegeLevel::Escalated);
        
        if let (Some(normal), Some(escalated)) = (normal_result, escalated_result) {
            match PrivilegeManager::analyze_privilege_differences(normal, escalated) {
                Ok(analysis) => {
                    println!("\n=== Privilege Analysis ===");
                    println!("Both executions successful: {}", analysis.both_executions_successful);
                    println!("Privilege metadata differs: {}", analysis.privilege_metadata_differs);
                    println!("Additional data with privileges: {}", analysis.additional_data_with_privileges);
                    println!("Normal execution time: {}ms", analysis.normal_execution_time_ms);
                    println!("Escalated execution time: {}ms", analysis.escalated_execution_time_ms);
                    println!("Performance impact: {}", analysis.performance_impact);
                }
                Err(e) => error!("Failed to analyze privilege differences: {}", e),
            }
        }
    }
    
    Ok(())
}

/// Validate script output format
async fn validate_script_output(script_path: Option<&str>) -> Result<()> {
    info!("Validating script output format");
    
    let script = script_path.unwrap_or("./collect_info.sh");
    let executor = ScriptExecutor::new(script.to_string(), 120, 1);
    executor.validate_script()?;
    
    let context = TestContext {
        os: OperatingSystem::Ubuntu,
        architecture: Architecture::X86_64,
        privilege_level: PrivilegeLevel::Normal,
        test_id: "validation".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    match executor.execute(&context).await {
        Ok(result) => {
            if result.success {
                if let Some(output) = &result.output {
                    match OutputValidator::validate_output(output) {
                        Ok(issues) => {
                            if issues.is_empty() {
                                info!("✓ Script output is valid and well-formed");
                                println!("✓ All validation checks passed");
                                println!("  - Architecture: {}", output.detected_architecture);
                                println!("  - Plugin count: {}", output.collection_metadata.plugin_count);
                                println!("  - Plugins: {}", output.plugins.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", "));
                            } else {
                                warn!("⚠ Script output has validation issues:");
                                for issue in &issues {
                                    println!("  - {}", issue);
                                }
                            }
                        }
                        Err(e) => error!("Validation failed: {}", e),
                    }
                } else {
                    error!("✗ Script succeeded but produced no parseable output");
                }
            } else {
                error!("✗ Script execution failed: {}", 
                       result.error_message.unwrap_or("Unknown error".to_string()));
            }
        }
        Err(e) => error!("Failed to execute script: {}", e),
    }
    
    Ok(())
}

/// Show system information and capabilities
async fn show_system_info() -> Result<()> {
    println!("=== CI Test Suite System Information ===\n");
    
    // Show privilege information
    let privilege_info = PrivilegeManager::get_privilege_info();
    println!("{}", privilege_info);
    
    // Show supported architectures
    println!("Supported Architectures:");
    for arch in Architecture::all() {
        println!("  - {}", arch);
    }
    
    // Show supported operating systems
    println!("\nSupported Operating Systems:");
    for os in OsSupport::get_all_supported_os() {
        println!("  - {}", os);
    }
    
    // Show top 3 Unix OS for FY 2025 Q1
    println!("\nTop 3 Unix OS for FY 2025 Q1:");
    for os in OsSupport::get_top_unix_os() {
        println!("  - {}", os);
    }
    
    // Show available Docker images
    println!("\nDocker Image Support:");
    let docker_images = OsSupport::get_docker_images();
    for (os, images) in docker_images {
        println!("  {}:", os);
        for image in images {
            println!("    - {} ({})", image.image, image.architecture_support.join(", "));
        }
    }
    
    Ok(())
}

/// Perform regression analysis between test results
async fn perform_regression_analysis(results: &[TestResult]) -> Result<Vec<ComparisonResult>> {
    info!("Performing regression analysis");
    
    let mut comparisons = Vec::new();
    let successful_results: Vec<_> = results.iter().filter(|r| r.success).collect();
    
    // Compare results with the same OS/architecture but different privilege levels
    let mut grouped_results: HashMap<String, Vec<&TestResult>> = HashMap::new();
    for result in &successful_results {
        let key = format!("{}-{}", result.context.os, result.context.architecture);
        grouped_results.entry(key).or_insert_with(Vec::new).push(result);
    }
    
    for (combo, group) in grouped_results {
        if group.len() >= 2 {
            for i in 0..group.len() {
                for j in i+1..group.len() {
                    match OutputValidator::compare_outputs(group[i], group[j]) {
                        Ok(comparison) => {
                            info!("Comparison for {}: {} differences", combo, comparison.data_differences.len());
                            comparisons.push(comparison);
                        }
                        Err(e) => warn!("Failed to compare results for {}: {}", combo, e),
                    }
                }
            }
        }
    }
    
    Ok(comparisons)
}

/// Perform privilege analysis
async fn perform_privilege_analysis(results: &[TestResult]) -> Result<Vec<PrivilegeDifferenceAnalysis>> {
    info!("Performing privilege analysis");
    
    let mut analyses = Vec::new();
    let successful_results: Vec<_> = results.iter().filter(|r| r.success).collect();
    
    // Group by OS/architecture, then find normal/escalated pairs
    let mut grouped_results: HashMap<String, Vec<&TestResult>> = HashMap::new();
    for result in &successful_results {
        let key = format!("{}-{}", result.context.os, result.context.architecture);
        grouped_results.entry(key).or_insert_with(Vec::new).push(result);
    }
    
    for (combo, group) in grouped_results {
        let normal = group.iter().find(|r| r.context.privilege_level == PrivilegeLevel::Normal);
        let escalated = group.iter().find(|r| r.context.privilege_level == PrivilegeLevel::Escalated);
        
        if let (Some(normal_result), Some(escalated_result)) = (normal, escalated) {
            match PrivilegeManager::analyze_privilege_differences(normal_result, escalated_result) {
                Ok(analysis) => {
                    info!("Privilege analysis for {}: performance impact {:?}", combo, analysis.performance_impact);
                    analyses.push(analysis);
                }
                Err(e) => warn!("Failed to analyze privilege differences for {}: {}", combo, e),
            }
        }
    }
    
    Ok(analyses)
}

/// Print test summary to console
fn print_test_summary(report: &TestReport) {
    println!("\n=== Test Suite Summary ===");
    println!("Total Tests: {}", report.summary.total_tests);
    println!("Successful: {} ({:.1}%)", report.summary.successful_tests, report.summary.success_rate);
    println!("Failed: {}", report.summary.failed_tests);
    println!("Average Execution Time: {}ms", report.summary.avg_execution_time_ms);
    
    if report.summary.total_comparisons > 0 {
        println!("\nRegression Analysis:");
        println!("  Comparisons: {}", report.summary.total_comparisons);
        println!("  Regressions: {} ({:.1}%)", 
                report.regression_analysis.regressions_found, 
                report.regression_analysis.regression_rate);
    }
    
    if report.privilege_analysis.total_privilege_tests > 0 {
        println!("\nPrivilege Analysis:");
        println!("  Tests: {}/{} successful", 
                report.privilege_analysis.successful_both_levels,
                report.privilege_analysis.total_privilege_tests);
        println!("  Average Performance Impact: {:.1}%", 
                report.privilege_analysis.average_performance_impact);
    }
    
    if !report.recommendations.is_empty() {
        println!("\nRecommendations:");
        for rec in &report.recommendations {
            println!("  - {}", rec);
        }
    }
    
    println!("\nDetailed reports available in: {}", report.metadata.config.output_directory);
}

/// Parse architecture string to enum
fn parse_architecture(name: &str) -> Option<Architecture> {
    match name.to_lowercase().as_str() {
        "x86_64" | "amd64" => Some(Architecture::X86_64),
        "arm64" | "aarch64" => Some(Architecture::Arm64),
        "i386" | "i686" => Some(Architecture::I386),
        "ppc64le" => Some(Architecture::Ppc64le),
        "s390x" => Some(Architecture::S390x),
        "riscv64" => Some(Architecture::RiscV64),
        "mips64" => Some(Architecture::Mips64),
        "aarch32" | "armv7l" => Some(Architecture::Aarch32),
        "sparc64" => Some(Architecture::Sparc64),
        "loongarch64" => Some(Architecture::LoongArch64),
        _ => None,
    }
}

/// Parse operating system string to enum
fn parse_operating_system(name: &str) -> Option<OperatingSystem> {
    match name.to_lowercase().as_str() {
        "ubuntu" => Some(OperatingSystem::Ubuntu),
        "alpine" => Some(OperatingSystem::Alpine),
        "centos" => Some(OperatingSystem::CentOS),
        "rocky" => Some(OperatingSystem::Rocky),
        "debian" => Some(OperatingSystem::Debian),
        _ => None,
    }
}