# Test Suite Framework

## Overview

The Test Suite Framework is a comprehensive system for organizing, running, and reporting test suites in the Navius workspace. It builds on the Mock Interface Registry and Integration Test Utilities to provide a complete testing solution, enabling developers to organize tests into logical suites, execute them in parallel or sequentially, and generate detailed reports for analysis.

## Key Features

- **Test Suite Organization**: Group related tests into logical suites with metadata
- **Hierarchical Test Structure**: Organize tests with dependencies and relationships
- **Parallel Execution**: Run tests concurrently with configurable concurrency levels
- **Test Filtering**: Filter tests by name, tags, or dependencies
- **Comprehensive Reporting**: Generate detailed reports in multiple formats
- **CI/CD Integration**: Seamless integration with various CI/CD environments
- **Flexible Configuration**: Extensive configuration options for test execution

## Core Components

### TestSuite

The central component for managing a collection of related tests:

```rust
pub struct TestSuite {
    pub metadata: TestSuiteMetadata,
    pub options: TestSuiteOptions,
    pub config: TestConfig,
    // ...private fields...
}
```

Key methods:
- `new(name: impl Into<String>, config: TestConfig) -> Self`
- `add_test(&mut self, test: impl IntegrationTest + Send + Sync + 'static, tags: Option<Vec<String>>, dependencies: Option<Vec<String>>) -> &mut Self`
- `run_all(&self) -> TestResult<TestSuiteSummary>`
- `run_filtered(&self, filter: &TestFilter) -> TestResult<TestSuiteSummary>`
- `filter_tests(&self, filter: &TestFilter) -> Vec<Arc<dyn IntegrationTest + Send + Sync>>`
- `discover_tests(&mut self, dir: impl AsRef<Path>) -> TestResult<usize>`

### TestSuiteMetadata

Contains information about the test suite:

```rust
pub struct TestSuiteMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub author: Option<String>,
    pub dependencies: Vec<String>,
    pub estimated_time: Option<f64>,
    pub properties: HashMap<String, String>,
}
```

### TestSuiteOptions

Configures how the test suite is executed:

```rust
pub struct TestSuiteOptions {
    pub parallel: bool,
    pub max_concurrent: usize,
    pub fail_fast: bool,
    pub name_filter: Option<String>,
    pub tag_filter: Option<HashSet<String>>,
    pub suite_timeout: Option<Duration>,
    pub verify_mocks: bool,
    pub cleanup_resources: bool,
    pub retry_failed: bool,
    pub max_retries: usize,
    pub report_dir: Option<PathBuf>,
    pub report_format: ReportFormat,
}
```

### TestFilter

Provides criteria for filtering tests within a suite:

```rust
pub struct TestFilter {
    pub name: Option<String>,
    pub include_tags: Option<HashSet<String>>,
    pub exclude_tags: Option<HashSet<String>>,
    pub max_duration: Option<Duration>,
    pub with_dependencies: Option<Vec<String>>,
}
```

### TestSuiteSummary

Provides statistics about the test run:

```rust
pub struct TestSuiteSummary {
    pub name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub slowest_test: Option<String>,
    pub slowest_duration: Option<Duration>,
    pub info: HashMap<String, String>,
    pub failed_tests: Vec<String>,
}
```

### TestSuiteBuilder

Provides a fluent interface for building test suites:

```rust
pub struct TestSuiteBuilder {
    // ...private fields...
}
```

Key methods:
- `new(name: impl Into<String>) -> Self`
- `with_config(mut self, config: TestConfig) -> Self`
- `with_description(mut self, description: impl Into<String>) -> Self`
- `with_author(mut self, author: impl Into<String>) -> Self`
- `with_tags(mut self, tags: Vec<String>) -> Self`
- `with_parallel(mut self, enable: bool) -> Self`
- `with_fail_fast(mut self, enable: bool) -> Self`
- `with_report_format(mut self, format: ReportFormat) -> Self`
- `add_test(mut self, test: impl IntegrationTest + Send + Sync + 'static, tags: Option<Vec<String>>, dependencies: Option<Vec<String>>) -> Self`
- `build(self) -> TestSuite`

## Usage Examples

### Basic Test Suite

```rust
use navius_test::{
    config::TestConfig,
    error::TestResult,
    suite::{TestSuite, TestFilter},
};

async fn run_basic_test_suite() -> TestResult<()> {
    // Create a test suite
    let mut suite = TestSuite::new("basic-suite", TestConfig::default());
    
    // Add tests to the suite
    suite.add_test(
        MyTest::new("test1", "Test 1 Description"),
        Some(vec!["basic".to_string()]),
        None,
    );
    
    suite.add_test(
        MyTest::new("test2", "Test 2 Description"),
        Some(vec!["basic".to_string(), "important".to_string()]),
        None,
    );
    
    // Run all tests in the suite
    let summary = suite.run_all()?;
    
    // Print the summary
    println!("Test suite: {}", summary.name);
    println!("Total tests: {}", summary.total_tests);
    println!("Passed: {}, Failed: {}, Skipped: {}", 
        summary.passed, summary.failed, summary.skipped);
    println!("Total duration: {:?}", summary.total_duration);
    
    Ok(())
}
```

### Advanced Test Suite with Filtering

```rust
use navius_test::{
    config::TestConfig,
    error::TestResult,
    suite::{TestSuite, TestSuiteBuilder, TestFilter},
};
use std::collections::HashSet;

async fn run_advanced_test_suite() -> TestResult<()> {
    // Create a test suite using the builder
    let suite = TestSuiteBuilder::new("advanced-suite")
        .with_config(TestConfig::default())
        .with_description("Advanced test suite with filtering")
        .with_author("Test Team")
        .with_parallel(true)
        .with_max_concurrent(4)
        .with_fail_fast(true)
        .with_report_format(ReportFormat::JUnit)
        .add_test(
            MyTest::new("api-test", "API Test"),
            Some(vec!["api".to_string(), "core".to_string()]),
            None
        )
        .add_test(
            MyTest::new("db-test", "Database Test"),
            Some(vec!["database".to_string(), "integration".to_string()]),
            None
        )
        .add_test(
            MyTest::new("cache-test", "Cache Test"),
            Some(vec!["cache".to_string(), "integration".to_string()]),
            None
        )
        .build();
    
    // Create a filter for integration tests
    let mut include_tags = HashSet::new();
    include_tags.insert("integration".to_string());
    
    let filter = TestFilter {
        name: None,
        include_tags: Some(include_tags),
        exclude_tags: None,
        max_duration: None,
        with_dependencies: None,
    };
    
    // Run filtered tests
    let summary = suite.run_filtered(&filter)?;
    
    // Output would only include db-test and cache-test
    println!("Filtered test suite results:");
    println!("Tests run: {}", summary.total_tests);
    println!("Passed: {}, Failed: {}", summary.passed, summary.failed);
    
    Ok(())
}
```

## Integration with CI/CD

The Test Suite Framework integrates seamlessly with CI/CD environments, automatically detecting the current environment and adapting its behavior:

```rust
use navius_test::{
    config::TestConfig,
    error::TestResult,
    suite::{TestSuiteBuilder, ReportFormat},
    integration::CIEnvironment,
};

async fn run_in_ci() -> TestResult<()> {
    // Check if running in a CI environment
    if CIEnvironment::is_ci() {
        // Get current CI environment information
        let ci_env = CIEnvironment::detect().unwrap();
        println!("Running in CI environment: {}", ci_env.name);
        println!("Branch: {:?}, Commit: {:?}", ci_env.branch, ci_env.commit);
        
        // Create a test suite configured for CI
        let suite = TestSuiteBuilder::new("ci-test-suite")
            .with_config(TestConfig::default())
            .with_report_format(ReportFormat::JUnit) // CI-friendly format
            .with_report_dir(PathBuf::from("test-reports"))
            .with_retry_failed(true) // Retry flaky tests in CI
            .build();
            
        // ...add tests and run the suite...
    }
    
    Ok(())
}
```

## Best Practices

1. **Organize tests logically**: Group related tests into suites based on functionality or components
2. **Use descriptive names and tags**: Make it easy to identify and filter tests
3. **Set appropriate timeouts**: Avoid tests hanging indefinitely
4. **Configure concurrency appropriately**: Set max_concurrent based on resource usage
5. **Use dependencies for ordered tests**: When tests must run in a specific order
6. **Generate reports in CI environments**: Use JUnit format for integration with CI tools
7. **Include comprehensive metadata**: Add descriptions, authors, and estimated duration for better visibility

## Error Handling

The Test Suite Framework uses the standard `TestResult<T>` type for error handling. Key error variants include:

- `TestError::SuiteError`: Errors related to the test suite configuration or execution
- `TestError::RunnerError`: Errors in the test runner
- `TestError::TimeoutError`: Tests exceeded their timeout duration
- `TestError::DependencyError`: Issues with test dependencies
- `TestError::ConfigurationError`: Problems with test configuration

## Performance Considerations

- Parallel test execution can significantly reduce total test time
- Set `max_concurrent` appropriately based on system resources
- Avoid excessively large test suites in a single execution
- Consider using test filtering to run only relevant tests during development
- For large test suites, use the discovery mechanism to find tests automatically

## Conclusion

The Test Suite Framework provides a robust foundation for organizing and executing tests across the Navius workspace. By combining the power of the Mock Interface Registry and Integration Test Utilities with a flexible suite execution engine, it enables comprehensive testing strategies that scale from simple unit tests to complex integration scenarios. 