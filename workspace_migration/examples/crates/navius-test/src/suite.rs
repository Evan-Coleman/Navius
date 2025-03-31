//! Test Suite Framework
//!
//! Provides a comprehensive framework for organizing and running test suites.
//! Supports test discovery, filtering, parallel execution, and detailed reporting.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::config::TestConfig;
use crate::error::{TestError, TestResult};
use crate::integration::{CIEnvironment, CIReportConfig, IntegrationContext, ReportFormat};
use crate::runner::{IntegrationTest, TestReport, TestRunner};

/// Test suite metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteMetadata {
    /// The name of the test suite
    pub name: String,
    /// Description of the test suite
    pub description: Option<String>,
    /// Tags for categorizing and filtering tests
    pub tags: Vec<String>,
    /// Time when the suite was created
    pub created_at: String,
    /// Time when the suite was last updated
    pub updated_at: String,
    /// Author of the test suite
    pub author: Option<String>,
    /// Dependencies required by this test suite
    pub dependencies: Vec<String>,
    /// Estimated execution time in seconds
    pub estimated_time: Option<f64>,
    /// Additional custom properties
    pub properties: HashMap<String, String>,
}

impl Default for TestSuiteMetadata {
    fn default() -> Self {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();

        Self {
            name: "unnamed_suite".to_string(),
            description: None,
            tags: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            author: None,
            dependencies: Vec::new(),
            estimated_time: None,
            properties: HashMap::new(),
        }
    }
}

/// Test suite execution options
#[derive(Debug, Clone)]
pub struct TestSuiteOptions {
    /// Whether to run tests in parallel
    pub parallel: bool,
    /// Maximum number of concurrent tests
    pub max_concurrent: usize,
    /// Whether to stop on first failure
    pub fail_fast: bool,
    /// Filter tests by name pattern
    pub name_filter: Option<String>,
    /// Filter tests by tag
    pub tag_filter: Option<HashSet<String>>,
    /// Custom timeout for test suite execution
    pub suite_timeout: Option<Duration>,
    /// Whether to verify mock expectations
    pub verify_mocks: bool,
    /// Whether to clean up resources
    pub cleanup_resources: bool,
    /// Whether to retry failed tests
    pub retry_failed: bool,
    /// Maximum number of retries
    pub max_retries: usize,
    /// Directory for storing test reports
    pub report_dir: Option<PathBuf>,
    /// Format for test reports
    pub report_format: ReportFormat,
}

impl Default for TestSuiteOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            max_concurrent: num_cpus::get(),
            fail_fast: false,
            name_filter: None,
            tag_filter: None,
            suite_timeout: Some(Duration::from_secs(300)), // 5 minutes default
            verify_mocks: true,
            cleanup_resources: true,
            retry_failed: false,
            max_retries: 1,
            report_dir: None,
            report_format: ReportFormat::JSON,
        }
    }
}

/// Test filter criteria
#[derive(Debug, Clone)]
pub struct TestFilter {
    /// Filter by test name (supports glob patterns)
    pub name: Option<String>,
    /// Filter by tags (include tests with any of these tags)
    pub include_tags: Option<HashSet<String>>,
    /// Filter by tags (exclude tests with any of these tags)
    pub exclude_tags: Option<HashSet<String>>,
    /// Filter by test duration (include tests faster than this)
    pub max_duration: Option<Duration>,
    /// Filter by dependencies
    pub with_dependencies: Option<Vec<String>>,
}

impl Default for TestFilter {
    fn default() -> Self {
        Self {
            name: None,
            include_tags: None,
            exclude_tags: None,
            max_duration: None,
            with_dependencies: None,
        }
    }
}

/// Test suite summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteSummary {
    /// Name of the test suite
    pub name: String,
    /// Total number of tests
    pub total_tests: usize,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Number of skipped tests
    pub skipped: usize,
    /// Total duration of test suite execution
    pub total_duration: Duration,
    /// Average test duration
    pub average_duration: Duration,
    /// Slowest test
    pub slowest_test: Option<String>,
    /// Slowest test duration
    pub slowest_duration: Option<Duration>,
    /// Additional information
    pub info: HashMap<String, String>,
    /// Failed test names
    pub failed_tests: Vec<String>,
}

/// Comprehensive test suite
#[derive(Clone)]
pub struct TestSuite {
    /// Metadata for the test suite
    pub metadata: TestSuiteMetadata,
    /// Options for test execution
    pub options: TestSuiteOptions,
    /// Test configuration
    pub config: TestConfig,
    /// Tests in this suite
    tests: Vec<Arc<dyn IntegrationTest + Send + Sync>>,
    /// Test dependencies as a directed acyclic graph
    test_dependencies: HashMap<String, Vec<String>>,
    /// Test tags
    test_tags: HashMap<String, HashSet<String>>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: impl Into<String>, config: TestConfig) -> Self {
        let mut metadata = TestSuiteMetadata::default();
        metadata.name = name.into();

        Self {
            metadata,
            options: TestSuiteOptions::default(),
            config,
            tests: Vec::new(),
            test_dependencies: HashMap::new(),
            test_tags: HashMap::new(),
        }
    }

    /// Add a test to the suite
    pub fn add_test(
        &mut self,
        test: impl IntegrationTest + Send + Sync + 'static,
        tags: Option<Vec<String>>,
        dependencies: Option<Vec<String>>,
    ) -> &mut Self {
        let test_name = test.name().to_string();
        let test = Arc::new(test);

        // Store test
        self.tests.push(test);

        // Store dependencies if any
        if let Some(deps) = dependencies {
            self.test_dependencies.insert(test_name.clone(), deps);
        }

        // Store tags if any
        if let Some(tags) = tags {
            let tags_set: HashSet<String> = tags.into_iter().collect();
            self.test_tags.insert(test_name, tags_set);
        }

        self
    }

    /// Add multiple tests to the suite
    pub fn add_tests<I>(&mut self, tests: I) -> &mut Self
    where
        I: IntoIterator<Item = Arc<dyn IntegrationTest + Send + Sync>>,
    {
        for test in tests {
            self.tests.push(test);
        }
        self
    }

    /// Set options for test execution
    pub fn with_options(mut self, options: TestSuiteOptions) -> Self {
        self.options = options;
        self
    }

    /// Set metadata for the test suite
    pub fn with_metadata(mut self, metadata: TestSuiteMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set description for the test suite
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    /// Add tags to the test suite
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Set the author of the test suite
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = Some(author.into());
        self
    }

    /// Filter tests based on the given criteria
    pub fn filter_tests(&self, filter: &TestFilter) -> Vec<Arc<dyn IntegrationTest + Send + Sync>> {
        let mut filtered_tests = Vec::new();

        for test in &self.tests {
            let test_name = test.name();

            // Filter by name if specified
            if let Some(name_pattern) = &filter.name {
                if !glob_match::glob_match(name_pattern, test_name) {
                    continue;
                }
            }

            // Filter by included tags if specified
            if let Some(include_tags) = &filter.include_tags {
                if let Some(test_tags) = self.test_tags.get(test_name) {
                    if test_tags.is_disjoint(include_tags) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Filter by excluded tags if specified
            if let Some(exclude_tags) = &filter.exclude_tags {
                if let Some(test_tags) = self.test_tags.get(test_name) {
                    if !test_tags.is_disjoint(exclude_tags) {
                        continue;
                    }
                }
            }

            // Filter by dependencies if specified
            if let Some(deps) = &filter.with_dependencies {
                if let Some(test_deps) = self.test_dependencies.get(test_name) {
                    if !deps.iter().all(|dep| test_deps.contains(dep)) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Test passed all filters
            filtered_tests.push(Arc::clone(test));
        }

        filtered_tests
    }

    /// Discover tests in a directory
    pub fn discover_tests(&mut self, dir: impl AsRef<Path>) -> TestResult<usize> {
        // This is a placeholder for test discovery logic
        // In a real implementation, this would scan the directory for test files,
        // load them, and register the tests with the suite

        // For now, just return the current number of tests
        Ok(self.tests.len())
    }

    /// Run all tests in the suite
    pub fn run_all(&self) -> TestResult<TestSuiteSummary> {
        self.run_filtered(&TestFilter::default())
    }

    /// Run tests matching the filter
    pub fn run_filtered(&self, filter: &TestFilter) -> TestResult<TestSuiteSummary> {
        let filtered_tests = self.filter_tests(filter);
        let total_tests = filtered_tests.len();

        if total_tests == 0 {
            return Err(TestError::suite_error("No tests match the filter criteria"));
        }

        // Create a test runner
        let runner = TestRunner::new(self.config.clone());

        // Track test results
        let mut summary = TestSuiteSummary {
            name: self.metadata.name.clone(),
            total_tests,
            passed: 0,
            failed: 0,
            skipped: 0,
            total_duration: Duration::default(),
            average_duration: Duration::default(),
            slowest_test: None,
            slowest_duration: None,
            info: HashMap::new(),
            failed_tests: Vec::new(),
        };

        let start_time = Instant::now();

        if self.options.parallel {
            // Run tests in parallel
            let results = self.run_parallel(&runner, filtered_tests)?;

            // Process results
            for report in results {
                summary.total_duration += report.duration;

                if report.passed {
                    summary.passed += 1;
                } else {
                    summary.failed += 1;
                    summary.failed_tests.push(report.name.clone());
                }

                // Update slowest test info
                if let Some(slowest_duration) = summary.slowest_duration {
                    if report.duration > slowest_duration {
                        summary.slowest_test = Some(report.name);
                        summary.slowest_duration = Some(report.duration);
                    }
                } else {
                    summary.slowest_test = Some(report.name);
                    summary.slowest_duration = Some(report.duration);
                }
            }
        } else {
            // Run tests sequentially
            for test in filtered_tests {
                let report = runner.run_test(&*test)?;

                summary.total_duration += report.duration;

                if report.passed {
                    summary.passed += 1;
                } else {
                    summary.failed += 1;
                    summary.failed_tests.push(report.name.clone());

                    // Stop on first failure if fail_fast is enabled
                    if self.options.fail_fast {
                        break;
                    }
                }

                // Update slowest test info
                if let Some(slowest_duration) = summary.slowest_duration {
                    if report.duration > slowest_duration {
                        summary.slowest_test = Some(report.name);
                        summary.slowest_duration = Some(report.duration);
                    }
                } else {
                    summary.slowest_test = Some(report.name);
                    summary.slowest_duration = Some(report.duration);
                }
            }
        }

        // Calculate the average duration
        if summary.passed + summary.failed > 0 {
            summary.average_duration =
                summary.total_duration / (summary.passed + summary.failed) as u32;
        }

        // Add additional information
        summary.info.insert(
            "execution_time".to_string(),
            format!("{:?}", start_time.elapsed()),
        );
        summary.info.insert(
            "parallel_execution".to_string(),
            self.options.parallel.to_string(),
        );

        // Check if we're running in a CI environment
        if let Some(ci) = CIEnvironment::detect() {
            summary.info.insert("ci_provider".to_string(), ci.name);
            if let Some(build_id) = ci.build_id {
                summary.info.insert("ci_build_id".to_string(), build_id);
            }
        }

        // Generate test report if a report directory is specified
        if let Some(report_dir) = &self.options.report_dir {
            self.generate_report(report_dir, &summary)?;
        }

        Ok(summary)
    }

    /// Run tests in parallel
    fn run_parallel(
        &self,
        runner: &TestRunner,
        tests: Vec<Arc<dyn IntegrationTest + Send + Sync>>,
    ) -> TestResult<Vec<TestReport>> {
        // Create a runtime for async execution
        let runtime = Runtime::new().map_err(|e| {
            TestError::suite_error(format!("Failed to create async runtime: {}", e))
        })?;

        // Execute in the runtime
        runtime.block_on(async {
            let max_concurrent = self.options.max_concurrent;
            let fail_fast = self.options.fail_fast;

            // Shared state for fail-fast mode
            let failure_occurred = Arc::new(Mutex::new(false));

            // Create a set of futures
            let mut futures = FuturesUnordered::new();
            let mut results = Vec::new();

            // Process tests in batches to respect max_concurrent
            for test_batch in tests.chunks(max_concurrent) {
                for test in test_batch {
                    let test = Arc::clone(test);
                    let runner = runner.clone();
                    let failure_occurred = Arc::clone(&failure_occurred);

                    // Spawn a task for each test
                    futures.push(tokio::spawn(async move {
                        // Check if we should skip due to previous failures
                        if fail_fast {
                            let should_skip = *failure_occurred.lock().unwrap();
                            if should_skip {
                                return TestReport {
                                    name: test.name().to_string(),
                                    passed: false,
                                    error: Some("Skipped due to previous test failure".to_string()),
                                    duration: Duration::default(),
                                    info: HashMap::new(),
                                };
                            }
                        }

                        // Run the test
                        let report = runner.run_test(&*test).unwrap_or_else(|e| TestReport {
                            name: test.name().to_string(),
                            passed: false,
                            error: Some(format!("Failed to run test: {}", e)),
                            duration: Duration::default(),
                            info: HashMap::new(),
                        });

                        // Update failure state if needed
                        if !report.passed && fail_fast {
                            let mut failure = failure_occurred.lock().unwrap();
                            *failure = true;
                        }

                        report
                    }));
                }

                // Process completed futures
                while let Some(result) = futures.next().await {
                    match result {
                        Ok(report) => {
                            results.push(report);

                            // Check if we should stop due to failures
                            if fail_fast && !report.passed {
                                break;
                            }
                        }
                        Err(e) => {
                            // Handle task join error
                            results.push(TestReport {
                                name: "unknown".to_string(),
                                passed: false,
                                error: Some(format!("Task execution error: {}", e)),
                                duration: Duration::default(),
                                info: HashMap::new(),
                            });

                            if fail_fast {
                                break;
                            }
                        }
                    }
                }

                // If fail_fast is enabled and a failure occurred, stop processing
                if fail_fast && *failure_occurred.lock().unwrap() {
                    break;
                }
            }

            Ok(results)
        })
    }

    /// Generate a test report
    fn generate_report(
        &self,
        dir: impl AsRef<Path>,
        summary: &TestSuiteSummary,
    ) -> TestResult<PathBuf> {
        let dir = dir.as_ref();

        // Create the directory if it doesn't exist
        std::fs::create_dir_all(dir).map_err(|e| {
            TestError::suite_error(format!("Failed to create report directory: {}", e))
        })?;

        // Generate the report file path
        let file_name = format!(
            "{}_report.{}",
            self.metadata.name,
            match self.options.report_format {
                ReportFormat::JUnit => "xml",
                ReportFormat::JSON => "json",
                ReportFormat::Text => "txt",
            }
        );

        let report_path = dir.join(file_name);

        // Generate the report based on the format
        match self.options.report_format {
            ReportFormat::JUnit => {
                self.generate_junit_report(&report_path, summary)?;
            }
            ReportFormat::JSON => {
                self.generate_json_report(&report_path, summary)?;
            }
            ReportFormat::Text => {
                self.generate_text_report(&report_path, summary)?;
            }
        }

        Ok(report_path)
    }

    /// Generate a JUnit XML report
    fn generate_junit_report(
        &self,
        path: impl AsRef<Path>,
        summary: &TestSuiteSummary,
    ) -> TestResult<()> {
        // Simple placeholder implementation
        // In a real implementation, this would generate a proper JUnit XML report
        let path = path.as_ref();

        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="{name}" tests="{total}" failures="{failed}" skipped="{skipped}" time="{time}">
  <testsuite name="{name}" tests="{total}" failures="{failed}" skipped="{skipped}" time="{time}">
    <!-- Test cases would be listed here -->
  </testsuite>
</testsuites>"#,
            name = self.metadata.name,
            total = summary.total_tests,
            failed = summary.failed,
            skipped = summary.skipped,
            time = summary.total_duration.as_secs_f64(),
        );

        std::fs::write(path, content)
            .map_err(|e| TestError::suite_error(format!("Failed to write JUnit report: {}", e)))?;

        Ok(())
    }

    /// Generate a JSON report
    fn generate_json_report(
        &self,
        path: impl AsRef<Path>,
        summary: &TestSuiteSummary,
    ) -> TestResult<()> {
        let path = path.as_ref();

        let json = serde_json::to_string_pretty(summary).map_err(|e| {
            TestError::suite_error(format!("Failed to serialize JSON report: {}", e))
        })?;

        std::fs::write(path, json)
            .map_err(|e| TestError::suite_error(format!("Failed to write JSON report: {}", e)))?;

        Ok(())
    }

    /// Generate a text report
    fn generate_text_report(
        &self,
        path: impl AsRef<Path>,
        summary: &TestSuiteSummary,
    ) -> TestResult<()> {
        let path = path.as_ref();

        let content = format!(
            r#"Test Suite: {}
Description: {}
Date: {}

Summary:
  Total Tests: {}
  Passed: {}
  Failed: {}
  Skipped: {}
  Total Duration: {:?}
  Average Duration: {:?}

Slowest Test: {} ({:?})

Failed Tests:
{}

Additional Information:
{}
"#,
            self.metadata.name,
            self.metadata.description.as_deref().unwrap_or(""),
            chrono::Utc::now().to_rfc3339(),
            summary.total_tests,
            summary.passed,
            summary.failed,
            summary.skipped,
            summary.total_duration,
            summary.average_duration,
            summary.slowest_test.as_deref().unwrap_or("N/A"),
            summary.slowest_duration.unwrap_or_default(),
            summary
                .failed_tests
                .iter()
                .map(|t| format!("  - {}", t))
                .collect::<Vec<_>>()
                .join("\n"),
            summary
                .info
                .iter()
                .map(|(k, v)| format!("  {}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        std::fs::write(path, content)
            .map_err(|e| TestError::suite_error(format!("Failed to write text report: {}", e)))?;

        Ok(())
    }
}

// Implement a builder for the test suite
pub struct TestSuiteBuilder {
    name: String,
    config: TestConfig,
    metadata: TestSuiteMetadata,
    options: TestSuiteOptions,
    tests: Vec<Arc<dyn IntegrationTest + Send + Sync>>,
    test_dependencies: HashMap<String, Vec<String>>,
    test_tags: HashMap<String, HashSet<String>>,
}

impl TestSuiteBuilder {
    /// Create a new test suite builder
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut metadata = TestSuiteMetadata::default();
        metadata.name = name.clone();

        Self {
            name,
            config: TestConfig::default(),
            metadata,
            options: TestSuiteOptions::default(),
            tests: Vec::new(),
            test_dependencies: HashMap::new(),
            test_tags: HashMap::new(),
        }
    }

    /// Set the test configuration
    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the test suite description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    /// Set the test suite author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = Some(author.into());
        self
    }

    /// Add tags to the test suite
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.metadata.tags = tags;
        self
    }

    /// Enable parallel test execution
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.options.parallel = enable;
        self
    }

    /// Set the maximum number of concurrent tests
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.options.max_concurrent = max;
        self
    }

    /// Enable fail-fast mode
    pub fn with_fail_fast(mut self, enable: bool) -> Self {
        self.options.fail_fast = enable;
        self
    }

    /// Set a name filter for tests
    pub fn with_name_filter(mut self, pattern: impl Into<String>) -> Self {
        self.options.name_filter = Some(pattern.into());
        self
    }

    /// Set a tag filter for tests
    pub fn with_tag_filter(mut self, tags: HashSet<String>) -> Self {
        self.options.tag_filter = Some(tags);
        self
    }

    /// Set a custom timeout for the test suite
    pub fn with_suite_timeout(mut self, timeout: Duration) -> Self {
        self.options.suite_timeout = Some(timeout);
        self
    }

    /// Enable mock verification
    pub fn with_verify_mocks(mut self, enable: bool) -> Self {
        self.options.verify_mocks = enable;
        self
    }

    /// Enable resource cleanup
    pub fn with_cleanup_resources(mut self, enable: bool) -> Self {
        self.options.cleanup_resources = enable;
        self
    }

    /// Enable retrying failed tests
    pub fn with_retry_failed(mut self, enable: bool) -> Self {
        self.options.retry_failed = enable;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max: usize) -> Self {
        self.options.max_retries = max;
        self
    }

    /// Set the directory for test reports
    pub fn with_report_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.options.report_dir = Some(dir.into());
        self
    }

    /// Set the format for test reports
    pub fn with_report_format(mut self, format: ReportFormat) -> Self {
        self.options.report_format = format;
        self
    }

    /// Add a test to the suite
    pub fn add_test(
        mut self,
        test: impl IntegrationTest + Send + Sync + 'static,
        tags: Option<Vec<String>>,
        dependencies: Option<Vec<String>>,
    ) -> Self {
        let test_name = test.name().to_string();
        let test = Arc::new(test);

        // Store test
        self.tests.push(test);

        // Store dependencies if any
        if let Some(deps) = dependencies {
            self.test_dependencies.insert(test_name.clone(), deps);
        }

        // Store tags if any
        if let Some(tags) = tags {
            let tags_set: HashSet<String> = tags.into_iter().collect();
            self.test_tags.insert(test_name, tags_set);
        }

        self
    }

    /// Build the test suite
    pub fn build(self) -> TestSuite {
        let mut suite = TestSuite {
            metadata: self.metadata,
            options: self.options,
            config: self.config,
            tests: self.tests,
            test_dependencies: self.test_dependencies,
            test_tags: self.test_tags,
        };

        // Apply any filters from options
        if let Some(name_filter) = &suite.options.name_filter {
            let filter = TestFilter {
                name: Some(name_filter.clone()),
                ..Default::default()
            };

            let filtered_tests = suite.filter_tests(&filter);
            suite.tests = filtered_tests;
        }

        if let Some(tag_filter) = &suite.options.tag_filter {
            let filter = TestFilter {
                include_tags: Some(tag_filter.clone()),
                ..Default::default()
            };

            let filtered_tests = suite.filter_tests(&filter);
            suite.tests = filtered_tests;
        }

        suite
    }
}

/// A mock for implementation testing
#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::ClosureTest;

    #[test]
    fn test_test_suite_builder() {
        // Create a test suite
        let builder = TestSuiteBuilder::new("test_suite")
            .with_description("Test suite for testing")
            .with_parallel(true)
            .with_max_concurrent(4)
            .with_fail_fast(true);

        let suite = builder.build();

        assert_eq!(suite.metadata.name, "test_suite");
        assert_eq!(
            suite.metadata.description,
            Some("Test suite for testing".to_string())
        );
        assert!(suite.options.parallel);
        assert_eq!(suite.options.max_concurrent, 4);
        assert!(suite.options.fail_fast);
    }

    #[test]
    fn test_test_suite_with_tests() {
        // Create test cases
        let test1 = ClosureTest::new("test1", |_| Ok(()));
        let test2 = ClosureTest::new("test2", |_| Ok(()));

        // Create a test suite
        let builder = TestSuiteBuilder::new("test_suite")
            .add_test(test1, Some(vec!["unit".to_string()]), None)
            .add_test(test2, Some(vec!["integration".to_string()]), None);

        let suite = builder.build();

        assert_eq!(suite.tests.len(), 2);
        assert!(suite.test_tags.contains_key("test1"));
        assert!(suite.test_tags.contains_key("test2"));
        assert!(suite.test_tags.get("test1").unwrap().contains("unit"));
        assert!(
            suite
                .test_tags
                .get("test2")
                .unwrap()
                .contains("integration")
        );
    }

    #[test]
    fn test_test_filter() {
        // Create test cases
        let test1 = ClosureTest::new("test1", |_| Ok(()));
        let test2 = ClosureTest::new("test2", |_| Ok(()));
        let test3 = ClosureTest::new("other_test", |_| Ok(()));

        // Create a test suite
        let mut suite = TestSuite::new("test_suite", TestConfig::default());

        // Add tests with tags
        suite.add_test(test1, Some(vec!["unit".to_string()]), None);
        suite.add_test(test2, Some(vec!["integration".to_string()]), None);
        suite.add_test(
            test3,
            Some(vec!["unit".to_string(), "slow".to_string()]),
            None,
        );

        // Create a filter for unit tests
        let filter = TestFilter {
            include_tags: Some([("unit").to_string()].into_iter().collect()),
            ..Default::default()
        };

        // Apply the filter
        let filtered_tests = suite.filter_tests(&filter);

        // Should include test1 and test3 (both tagged as "unit")
        assert_eq!(filtered_tests.len(), 2);
        assert!(filtered_tests.iter().any(|t| t.name() == "test1"));
        assert!(filtered_tests.iter().any(|t| t.name() == "other_test"));
        assert!(!filtered_tests.iter().any(|t| t.name() == "test2"));

        // Create a filter with name pattern
        let filter = TestFilter {
            name: Some("test*".to_string()),
            ..Default::default()
        };

        // Apply the filter
        let filtered_tests = suite.filter_tests(&filter);

        // Should include test1 and test2 (matching the pattern)
        assert_eq!(filtered_tests.len(), 2);
        assert!(filtered_tests.iter().any(|t| t.name() == "test1"));
        assert!(filtered_tests.iter().any(|t| t.name() == "test2"));
        assert!(!filtered_tests.iter().any(|t| t.name() == "other_test"));
    }
}
