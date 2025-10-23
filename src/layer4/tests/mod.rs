//! Comprehensive test suite for Layer 4 Execution Fabric
//!
//! This module provides a complete testing framework for validating
//! the Layer 4 WASM agent runtime and orchestration system.
//!
//! ## Test Categories
//!
//! - **Unit Tests**: Individual component testing in isolation
//! - **Integration Tests**: Component interaction validation
//! - **Performance Tests**: Execution speed and resource usage benchmarks
//! - **Stress Tests**: High-load scenario validation
//! - **Security Tests**: WASM sandboxing and access control validation
//!
//! ## Test Organization
//!
//! ```text
//! tests/
//! ├── mod.rs                    # Test module organization
//! ├── unit_tests.rs            # Unit tests for all modules
//! ├── integration_tests.rs     # Integration test scenarios
//! ├── performance_tests.rs     # Performance benchmarks
//! ├── stress_tests.rs          # High-load testing
//! ├── security_tests.rs        # Security validation
//! └── test_utils.rs            # Test utilities and mocks
//! ```

pub mod unit_tests;
pub mod integration_tests;
pub mod performance_tests;
pub mod stress_tests;
pub mod security_tests;
pub mod test_utils;

// Re-export test utilities for convenience
pub use test_utils::*;

/// Run all Layer 4 tests
///
/// This function executes the complete test suite including unit tests,
/// integration tests, performance benchmarks, stress tests, and security
/// validation tests. Used for comprehensive validation of the Layer 4
/// execution fabric.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Test execution result
///
/// # Examples
/// ```rust,no_run
/// use layer4_tests::run_all_tests()?;
/// println!("All Layer 4 tests passed!");
/// ```
pub async fn run_all_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running Layer 4 comprehensive test suite...");

    // Run unit tests
    println!("📋 Running unit tests...");
    unit_tests::run_unit_tests().await?;

    // Run integration tests
    println!("🔗 Running integration tests...");
    integration_tests::run_integration_tests().await?;

    // Run performance tests
    println!("⚡ Running performance tests...");
    performance_tests::run_performance_tests().await?;

    // Run stress tests
    println!("🔥 Running stress tests...");
    stress_tests::run_stress_tests().await?;

    // Run security tests
    println!("🔒 Running security tests...");
    security_tests::run_security_tests().await?;

    println!("✅ All Layer 4 tests completed successfully!");
    Ok(())
}

/// Test configuration for different scenarios
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Enable verbose logging during tests
    pub verbose: bool,
    /// Timeout for individual tests in seconds
    pub test_timeout_secs: u64,
    /// Number of concurrent test workers
    pub concurrent_workers: usize,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Test data directory
    pub test_data_dir: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            test_timeout_secs: 300, // 5 minutes
            concurrent_workers: 4,
            enable_profiling: false,
            test_data_dir: "./test_data".to_string(),
        }
    }
}

/// Initialize test environment
///
/// Sets up the test environment including temporary directories,
/// mock services, and test data. Should be called before running
/// tests that require external resources.
///
/// # Arguments
/// * `config` - Test configuration options
///
/// # Returns
/// * `Result<TestEnvironment, Box<dyn std::error::Error>>` - Initialized test environment
pub async fn init_test_environment(config: TestConfig) -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    test_utils::init_test_environment(config).await
}

/// Cleanup test environment
///
/// Cleans up temporary files, stops mock services, and releases
/// resources created during testing.
///
/// # Arguments
/// * `env` - Test environment to cleanup
pub async fn cleanup_test_environment(env: TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    test_utils::cleanup_test_environment(env).await
}