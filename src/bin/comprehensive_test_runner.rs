//! Comprehensive Test Runner for Automation Nation
//! 
//! This binary provides a command-line interface for running the complete
//! test suite including functional, regression, performance, and security tests.

use ci_test_suite::{ComprehensiveTestSuite, TestConfig};
use clap::{Parser, Subcommand};
use env_logger;
use log::{info, error};
use std::process;

#[derive(Parser)]
#[command(name = "comprehensive-test-runner")]
#[command(about = "Comprehensive test suite runner for Automation Nation")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests
    All {
        /// Database URL for testing (overrides environment variable)
        #[arg(long)]
        database_url: Option<String>,
        
        /// Disable performance tests
        #[arg(long)]
        no_performance: bool,
        
        /// Disable security tests
        #[arg(long)]
        no_security: bool,
        
        /// Disable integration tests
        #[arg(long)]
        no_integration: bool,
        
        /// Test timeout in seconds
        #[arg(long, default_value = "60")]
        timeout: u64,
        
        /// Performance test iterations
        #[arg(long, default_value = "100")]
        iterations: usize,
        
        /// Generate HTML report
        #[arg(long)]
        html_report: bool,
        
        /// Output file for report
        #[arg(long, default_value = "test_report.md")]
        output: String,
    },
    
    /// Run only functional tests
    Functional {
        #[arg(long)]
        database_url: Option<String>,
    },
    
    /// Run only performance tests
    Performance {
        #[arg(long)]
        database_url: Option<String>,
        
        #[arg(long, default_value = "100")]
        iterations: usize,
    },
    
    /// Run only security tests
    Security {
        #[arg(long)]
        database_url: Option<String>,
    },
    
    /// Run only integration tests
    Integration {
        #[arg(long)]
        database_url: Option<String>,
    },
    
    /// Validate test configuration
    Validate {
        #[arg(long)]
        database_url: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::All {
            database_url,
            no_performance,
            no_security,
            no_integration,
            timeout,
            iterations,
            html_report,
            output,
        } => {
            let config = TestConfig {
                database_url: database_url.or_else(|| std::env::var("DATABASE_URL").ok()),
                enable_performance_tests: !no_performance,
                enable_security_tests: !no_security,
                enable_integration_tests: !no_integration,
                test_timeout_seconds: timeout,
                performance_test_iterations: iterations,
            };
            
            run_comprehensive_tests(config, html_report, output).await
        }
        
        Commands::Functional { database_url } => {
            let config = TestConfig {
                database_url: database_url.or_else(|| std::env::var("DATABASE_URL").ok()),
                enable_performance_tests: false,
                enable_security_tests: false,
                enable_integration_tests: false,
                ..Default::default()
            };
            
            run_functional_tests(config).await
        }
        
        Commands::Performance { database_url, iterations } => {
            let config = TestConfig {
                database_url: database_url.or_else(|| std::env::var("DATABASE_URL").ok()),
                enable_performance_tests: true,
                enable_security_tests: false,
                enable_integration_tests: false,
                performance_test_iterations: iterations,
                ..Default::default()
            };
            
            run_performance_tests(config).await
        }
        
        Commands::Security { database_url } => {
            let config = TestConfig {
                database_url: database_url.or_else(|| std::env::var("DATABASE_URL").ok()),
                enable_performance_tests: false,
                enable_security_tests: true,
                enable_integration_tests: false,
                ..Default::default()
            };
            
            run_security_tests(config).await
        }
        
        Commands::Integration { database_url } => {
            let config = TestConfig {
                database_url: database_url.or_else(|| std::env::var("DATABASE_URL").ok()),
                enable_performance_tests: false,
                enable_security_tests: false,
                enable_integration_tests: true,
                ..Default::default()
            };
            
            run_integration_tests(config).await
        }
        
        Commands::Validate { database_url } => {
            let config = TestConfig {
                database_url: database_url.or_else(|| std::env::var("DATABASE_URL").ok()),
                ..Default::default()
            };
            
            validate_configuration(config).await
        }
    };
    
    match result {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            error!("Test runner failed: {}", e);
            process::exit(1);
        }
    }
}

/// Run comprehensive test suite
async fn run_comprehensive_tests(config: TestConfig, html_report: bool, output: String) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Starting comprehensive test suite");
    info!("Configuration: {:?}", config);
    
    let mut suite = ComprehensiveTestSuite::new(config).await?;
    let results = suite.run_all_tests().await?;
    
    // Generate and save report
    let report = suite.generate_test_report();
    
    if html_report {
        let html_report = generate_html_report(&report);
        let html_output = output.replace(".md", ".html");
        std::fs::write(&html_output, html_report)?;
        info!("HTML report saved to: {}", html_output);
    }
    
    std::fs::write(&output, &report)?;
    info!("Test report saved to: {}", output);
    
    // Print summary to console
    println!("\n{}", report);
    
    // Calculate overall success
    let total_passed = results.functional.passed + results.regression.passed + 
                      results.performance.passed + results.security.passed + 
                      results.integration.passed;
    let total_failed = results.functional.failed + results.regression.failed + 
                      results.performance.failed + results.security.failed + 
                      results.integration.failed;
    
    let success_rate = if total_passed + total_failed > 0 {
        total_passed as f64 / (total_passed + total_failed) as f64 * 100.0
    } else {
        0.0
    };
    
    info!("Overall success rate: {:.1}%", success_rate);
    
    if total_failed > 0 {
        error!("{} test(s) failed", total_failed);
        Ok(1)
    } else {
        info!("All tests passed!");
        Ok(0)
    }
}

/// Run functional tests only
async fn run_functional_tests(config: TestConfig) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Running functional tests only");
    
    let mut suite = ComprehensiveTestSuite::new(config).await?;
    suite.run_all_tests().await?;
    
    let results = &suite.generate_test_report();
    println!("{}", results);
    
    Ok(0)
}

/// Run performance tests only
async fn run_performance_tests(config: TestConfig) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Running performance tests only");
    
    let mut suite = ComprehensiveTestSuite::new(config).await?;
    suite.run_all_tests().await?;
    
    let results = &suite.generate_test_report();
    println!("{}", results);
    
    Ok(0)
}

/// Run security tests only
async fn run_security_tests(config: TestConfig) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Running security tests only");
    
    let mut suite = ComprehensiveTestSuite::new(config).await?;
    suite.run_all_tests().await?;
    
    let results = &suite.generate_test_report();
    println!("{}", results);
    
    Ok(0)
}

/// Run integration tests only
async fn run_integration_tests(config: TestConfig) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Running integration tests only");
    
    let mut suite = ComprehensiveTestSuite::new(config).await?;
    suite.run_all_tests().await?;
    
    let results = &suite.generate_test_report();
    println!("{}", results);
    
    Ok(0)
}

/// Validate test configuration
async fn validate_configuration(config: TestConfig) -> Result<i32, Box<dyn std::error::Error>> {
    info!("Validating test configuration");
    
    // Check database connectivity
    if let Some(db_url) = &config.database_url {
        info!("Testing database connection: {}", mask_db_url(db_url));
        
        match ComprehensiveTestSuite::new(config).await {
            Ok(_) => {
                info!("✓ Database connection successful");
            }
            Err(e) => {
                error!("✗ Database connection failed: {}", e);
                return Ok(1);
            }
        }
    } else {
        info!("⚠ No database URL configured - database tests will be skipped");
    }
    
    // Check required tools
    check_required_tools();
    
    info!("✓ Configuration validation complete");
    Ok(0)
}

/// Check for required tools and dependencies
fn check_required_tools() {
    let tools = vec![
        ("collect_info.sh", "./collect_info.sh"),
        ("docker", "docker"),
        ("podman", "podman"),
    ];
    
    for (name, command) in tools {
        match std::process::Command::new(command).arg("--version").output() {
            Ok(_) => info!("✓ {} is available", name),
            Err(_) => info!("⚠ {} is not available - some tests may be skipped", name),
        }
    }
}

/// Generate HTML report from markdown
fn generate_html_report(markdown: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Automation Nation - Comprehensive Test Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }}
        h2 {{ color: #34495e; margin-top: 30px; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
        th {{ background-color: #f8f9fa; font-weight: 600; }}
        .success {{ color: #27ae60; font-weight: bold; }}
        .failure {{ color: #e74c3c; font-weight: bold; }}
        .warning {{ color: #f39c12; font-weight: bold; }}
        code {{ background-color: #f8f9fa; padding: 2px 6px; border-radius: 3px; font-family: 'Monaco', 'Courier New', monospace; }}
        pre {{ background-color: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; }}
        .timestamp {{ color: #7f8c8d; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="timestamp">Generated: {}</div>
        <pre>{}</pre>
    </div>
</body>
</html>"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        html_escape::encode_text(markdown)
    )
}

/// Mask sensitive information in database URL
fn mask_db_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "***");
            return masked;
        }
    }
    url.to_string()
}