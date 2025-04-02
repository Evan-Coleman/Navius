//! Cross-crate test runner
//!
//! This module provides a test runner for executing integration tests across crates.
//! It supports running tests with fixtures, mocks, and environment configuration.

use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::stream::StreamExt;

use crate::config::TestConfig;
use crate::error::{TestError, TestResult};
use crate::integration::{IntegrationContext, IntegrationTestConfig, TestLifecycleHooks};

/// Trait for integration tests
pub trait IntegrationTest: Send + Sync {
    /// Get the name of the test
    fn name(&self) -> &str;

    /// Set up the test environment
    fn setup(&self, context: &mut IntegrationContext) -> TestResult<()>;

    /// Run the test
    fn run(&self, context: &IntegrationContext) -> TestResult<()>;

    /// Clean up the test environment
    fn cleanup(&self, context: &IntegrationContext) -> TestResult<()>;

    /// Box the test to create an owned version we can move into the thread
    fn box_clone(&self) -> Box<dyn IntegrationTest + Send>;
}

// Add a Debug implementation for dyn IntegrationTest
impl fmt::Debug for dyn IntegrationTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IntegrationTest({})", self.name())
    }
}

/// Test result report
#[derive(Debug, Clone)]
pub struct TestReport {
    pub test_name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration: Duration,
    pub info: HashMap<String, String>,
}

impl TestReport {
    pub fn new(test_name: String) -> Self {
        Self {
            test_name,
            passed: true,
            error: None,
            duration: Duration::default(),
            info: HashMap::new(),
        }
    }

    pub fn failed(test_name: String, error: String) -> Self {
        Self {
            test_name,
            passed: false,
            error: Some(error),
            duration: Duration::default(),
            info: HashMap::new(),
        }
    }

    /// Add information to the test report
    pub fn with_info(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.info.insert(key.into(), value.into());
        self
    }
}

/// Test runner for cross-crate integration tests
#[derive(Debug)]
pub struct TestRunner {
    /// Test configuration
    config: TestConfig,
    /// Output directory for test reports
    report_dir: Option<PathBuf>,
    /// Tests to run
    tests: Vec<Box<dyn IntegrationTest>>,
    /// Whether to fail fast on the first test failure
    fail_fast: bool,
    /// Whether to print output to stdout during tests
    verbose: bool,
}

impl TestRunner {
    /// Create a new test runner with the given configuration
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            report_dir: None,
            tests: Vec::new(),
            fail_fast: false,
            verbose: false,
        }
    }

    /// Set the output directory for test reports
    pub fn with_report_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.report_dir = Some(dir.into());
        self
    }

    /// Add a test to the runner
    pub fn with_test<T: IntegrationTest + 'static>(mut self, test: T) -> Self {
        self.tests.push(Box::new(test));
        self
    }

    /// Set whether to fail fast on the first test failure
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set whether to print output to stdout during tests
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Run all tests and return a list of test reports
    pub fn run_all(&self) -> TestResult<Vec<TestReport>> {
        let mut reports = Vec::new();

        if let Some(report_dir) = &self.report_dir {
            std::fs::create_dir_all(report_dir)?;
        }

        for test in &self.tests {
            let report = self.run_test(test.as_ref())?;

            if self.verbose {
                self.print_report(&report);
            }

            if let Some(report_dir) = &self.report_dir {
                let report_path = report_dir.join(format!("{}.txt", report.test_name));
                std::fs::write(&report_path, format!("{:?}", report)).map_err(|e| {
                    TestError::SetupError(format!("Failed to write test report: {}", e))
                })?;
            }

            if !report.passed && self.fail_fast {
                return Ok(reports);
            }

            reports.push(report);
        }

        Ok(reports)
    }

    /// Run a single test and return a test report
    pub fn run_test(&self, test: &dyn IntegrationTest) -> TestResult<TestReport> {
        let start_time = Instant::now();
        let test_name = test.name().to_string();

        // Use the correct timeout from the config
        let timeout = Duration::from_secs(self.config.timeouts.test_timeout_secs);

        // Use the integration test config from the TestConfig
        let integration_config = IntegrationTestConfig {
            name: test.name().to_string(),
            test_dir: self.config.resources.base_dir.clone(),
            env_vars: self.config.environment.variables.clone(),
            timeout: Some(timeout),
            verify_mocks: self.config.mocks.verify_expectations,
            cleanup_resources: self.config.resources.cleanup,
            service_configs: HashMap::new(),
            test_data_path: Some(PathBuf::from("tests/data")),
            db_setup_scripts: Vec::new(),
            lifecycle_hooks: TestLifecycleHooks::default(),
        };

        // Create the context here
        let context = IntegrationContext::new(integration_config)?;

        // Box clone the test to create an owned version
        let boxed_test = test.box_clone();

        let (tx, rx) = std::sync::mpsc::channel::<TestResult<()>>();

        let handle = std::thread::spawn(move || {
            let result = boxed_test.run(&context);
            let _ = tx.send(result);
        });

        // Run the test
        let run_result = match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => {
                let _ = handle.join();
                Err(TestError::execution_error(format!(
                    "Test '{}' timed out after {:?}",
                    test_name, timeout
                )))
            }
        };

        // Create the report
        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);

        let report = TestReport {
            test_name,
            passed: run_result.is_ok(),
            error: run_result.err().map(|e| format!("{}", e)),
            duration,
            info: HashMap::new(),
        };

        Ok(report)
    }

    /// Print a test report to stdout
    fn print_report(&self, report: &TestReport) {
        if report.passed {
            println!(
                "✅ {} - {:.2}s",
                report.test_name,
                report.duration.as_secs_f64()
            );
        } else {
            println!(
                "❌ {} - {:.2}s",
                report.test_name,
                report.duration.as_secs_f64()
            );
            if let Some(error) = &report.error {
                println!("   Error: {}", error);
            }
        }

        if !report.info.is_empty() {
            println!("   Info:");
            for (key, value) in &report.info {
                println!("     {}: {}", key, value);
            }
        }
    }
}

/// Default implementation of an integration test using closures
pub struct ClosureTest {
    /// Name of the test
    name: String,
    /// Setup function
    setup: Box<dyn Fn(&mut IntegrationContext) -> TestResult<()> + Send + Sync>,
    /// Test function
    test: Box<dyn Fn(&IntegrationContext) -> TestResult<()> + Send + Sync>,
    /// Cleanup function
    cleanup: Box<dyn Fn(&IntegrationContext) -> TestResult<()> + Send + Sync>,
}

impl ClosureTest {
    /// Create a new closure test with the given name and test function
    pub fn new<F>(name: impl Into<String>, test_fn: F) -> Self
    where
        F: Fn(&IntegrationContext) -> TestResult<()> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            setup: Box::new(|_| Ok(())),
            test: Box::new(test_fn),
            cleanup: Box::new(|_| Ok(())),
        }
    }

    /// Set the setup function
    pub fn with_setup<F>(mut self, setup_fn: F) -> Self
    where
        F: Fn(&mut IntegrationContext) -> TestResult<()> + Send + Sync + 'static,
    {
        self.setup = Box::new(setup_fn);
        self
    }

    /// Set the cleanup function
    pub fn with_cleanup<F>(mut self, cleanup_fn: F) -> Self
    where
        F: Fn(&IntegrationContext) -> TestResult<()> + Send + Sync + 'static,
    {
        self.cleanup = Box::new(cleanup_fn);
        self
    }
}

impl IntegrationTest for ClosureTest {
    fn name(&self) -> &str {
        &self.name
    }

    fn setup(&self, context: &mut IntegrationContext) -> TestResult<()> {
        (self.setup)(context)
    }

    fn run(&self, context: &IntegrationContext) -> TestResult<()> {
        (self.test)(context)
    }

    fn cleanup(&self, context: &IntegrationContext) -> TestResult<()> {
        (self.cleanup)(context)
    }

    /// Box the test to create an owned version we can move into the thread
    fn box_clone(&self) -> Box<dyn IntegrationTest + Send> {
        // Create a new instance with empty setup, test, and cleanup that matches the signature
        let empty_setup = Box::new(|_: &mut IntegrationContext| Ok(()));
        let empty_test = Box::new(|_: &IntegrationContext| Ok(()));
        let empty_cleanup = Box::new(|_: &IntegrationContext| Ok(()));

        Box::new(ClosureTest {
            name: self.name.clone(),
            setup: empty_setup,
            test: empty_test,
            cleanup: empty_cleanup,
        })
    }
}

/// Macro to create a test suite from multiple test cases
#[macro_export]
macro_rules! test_suite {
    ($name:expr, $config:expr, $($test:expr),* $(,)?) => {{
        let mut runner = $crate::runner::TestRunner::new($config);

        $(
            runner = runner.with_test($test);
        )*

        runner
    }};
}

/// Macro to create a test case with setup, test, and cleanup functions
#[macro_export]
macro_rules! test_case {
    ($name:expr, $test:expr) => {
        $crate::runner::ClosureTest::new($name, $test)
    };

    ($name:expr, $test:expr, setup: $setup:expr) => {
        $crate::runner::ClosureTest::new($name, $test).with_setup($setup)
    };

    ($name:expr, $test:expr, cleanup: $cleanup:expr) => {
        $crate::runner::ClosureTest::new($name, $test).with_cleanup($cleanup)
    };

    ($name:expr, $test:expr, setup: $setup:expr, cleanup: $cleanup:expr) => {
        $crate::runner::ClosureTest::new($name, $test)
            .with_setup($setup)
            .with_cleanup($cleanup)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_closure_test() {
        let setup_called = Arc::new(AtomicBool::new(false));
        let cleanup_called = Arc::new(AtomicBool::new(false));

        let setup_called_clone = setup_called.clone();
        let cleanup_called_clone = cleanup_called.clone();

        let test = ClosureTest::new("test_closure", |_| Ok(()))
            .with_setup(move |_| {
                setup_called_clone.store(true, Ordering::SeqCst);
                Ok(())
            })
            .with_cleanup(move |_| {
                cleanup_called_clone.store(true, Ordering::SeqCst);
                Ok(())
            });

        let config = TestConfig::default();
        let runner = TestRunner::new(config);

        let report = runner.run_test(&test).unwrap();

        assert!(report.passed);
        assert!(setup_called.load(Ordering::SeqCst));
        assert!(cleanup_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_test_failure() {
        let test = ClosureTest::new("test_failure", |_| {
            Err(TestError::AssertionFailed("Test assertion failed".into()))
        });

        let config = TestConfig::default();
        let runner = TestRunner::new(config);

        let report = runner.run_test(&test).unwrap();

        assert!(!report.passed);
        assert!(report.error.is_some());
        assert!(report.error.unwrap().contains("Test assertion failed"));
    }

    #[test]
    fn test_test_timeout() {
        let test = ClosureTest::new("test_timeout", |_| {
            std::thread::sleep(Duration::from_millis(500));
            Ok(())
        });

        let mut config = TestConfig::default();
        config.timeouts.test_timeout_secs = 0; // Set a very short timeout

        let runner = TestRunner::new(config);

        let report = runner.run_test(&test).unwrap();

        assert!(!report.passed);
        assert!(report.error.is_some());
        assert!(report.error.unwrap().contains("Test timed out"));
    }

    #[test]
    fn test_macro_test_case() {
        let test = test_case!("macro_test", |_| Ok(()));

        let config = TestConfig::default();
        let runner = TestRunner::new(config);

        let report = runner.run_test(&test).unwrap();

        assert!(report.passed);
        assert_eq!(report.test_name, "macro_test");
    }

    #[test]
    fn test_macro_test_suite() {
        let config = TestConfig::default();

        let runner = test_suite!(
            "test_suite",
            config,
            test_case!("test1", |_| Ok(())),
            test_case!("test2", |_| Ok(())),
        );

        let reports = runner.run_all().unwrap();

        assert_eq!(reports.len(), 2);
        assert!(reports[0].passed);
        assert!(reports[1].passed);
    }
}
