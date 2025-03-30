//! Cross-crate test runner
//!
//! This module provides a test runner for executing integration tests across crates.
//! It supports running tests with fixtures, mocks, and environment configuration.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::TestConfig;
use crate::error::{TestError, TestResult};
use crate::fixture::TestFixture;
use crate::integration::{IntegrationContext, IntegrationTestConfig};
use crate::mock::MockRegistry;

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
}

/// Test result report
#[derive(Debug, Clone)]
pub struct TestReport {
    /// Name of the test
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Error message if the test failed
    pub error: Option<String>,
    /// Duration of the test
    pub duration: Duration,
    /// Additional information about the test
    pub info: HashMap<String, String>,
}

impl TestReport {
    /// Create a new test report for a successful test
    pub fn success(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            passed: true,
            error: None,
            duration,
            info: HashMap::new(),
        }
    }

    /// Create a new test report for a failed test
    pub fn failure(name: impl Into<String>, error: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            passed: false,
            error: Some(error.into()),
            duration,
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
                let report_path = report_dir.join(format!("{}.json", report.name));
                let report_json = serde_json::to_string_pretty(&report).map_err(|e| {
                    TestError::IoError(format!("Failed to serialize test report: {}", e))
                })?;
                std::fs::write(report_path, report_json).map_err(|e| {
                    TestError::IoError(format!("Failed to write test report: {}", e))
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

        let integration_config = IntegrationTestConfig {
            name: test.name().to_string(),
            test_dir: self.config.resources.base_dir.clone(),
            env_vars: self.config.environment.variables.clone(),
            timeout: Some(Duration::from_secs(self.config.timeouts.test_timeout_secs)),
            verify_mocks: self.config.mocks.verify_expectations,
            cleanup_resources: self.config.resources.cleanup,
        };

        let mut context = IntegrationContext::new(integration_config)?;

        // Set up the test
        if let Err(e) = test.setup(&mut context) {
            let duration = start_time.elapsed();
            return Ok(TestReport::failure(
                test.name(),
                format!("Test setup failed: {}", e),
                duration,
            ));
        }

        // Run the test
        let run_result = if let Some(timeout) =
            Some(Duration::from_secs(self.config.timeouts.test_timeout_secs))
        {
            let (tx, rx) = std::sync::mpsc::channel();

            let test_ref = test;
            let context_ref = &context;

            let handle = std::thread::spawn(move || {
                let result = test_ref.run(context_ref);
                let _ = tx.send(result);
            });

            match rx.recv_timeout(timeout) {
                Ok(result) => result,
                Err(_) => {
                    let _ = handle.join();
                    Err(TestError::TimeoutError(format!(
                        "Test timed out after {:?}",
                        timeout
                    )))
                }
            }
        } else {
            test.run(&context)
        };

        // Clean up the test
        let cleanup_result = test.cleanup(&context);

        let duration = start_time.elapsed();

        // Create the test report
        match run_result {
            Ok(_) => {
                if let Err(e) = cleanup_result {
                    Ok(TestReport::failure(
                        test.name(),
                        format!("Test cleanup failed: {}", e),
                        duration,
                    ))
                } else {
                    Ok(TestReport::success(test.name(), duration))
                }
            }
            Err(e) => Ok(TestReport::failure(
                test.name(),
                format!("Test failed: {}", e),
                duration,
            )),
        }
    }

    /// Print a test report to stdout
    fn print_report(&self, report: &TestReport) {
        if report.passed {
            println!("✅ {} - {:.2}s", report.name, report.duration.as_secs_f64());
        } else {
            println!("❌ {} - {:.2}s", report.name, report.duration.as_secs_f64());
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
        assert_eq!(report.name, "macro_test");
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
