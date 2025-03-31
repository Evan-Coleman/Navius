use navius_test::{
    config::TestConfig,
    error::TestResult,
    integration::{IntegrationContext, ReportFormat},
    mock::MockRegistry,
    mocks::{
        cache::{CacheClient, MockCacheClient},
        database::{DatabaseClient, MockDatabaseClient, MockQueryResult, MockValue},
    },
    runner::ClosureTest,
    suite::{TestFilter, TestSuiteBuilder, TestSuiteMetadata, TestSuiteOptions},
    test_case,
};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Example demonstrating the Test Suite Framework
#[tokio::main]
async fn main() -> TestResult<()> {
    println!("Test Suite Framework Example");
    println!("============================\n");

    // Basic test suite demonstration
    println!("Running basic test suite...");
    run_basic_test_suite().await?;

    // Advanced test suite demonstration with filtering and reporting
    println!("\nRunning advanced test suite with filtering and reporting...");
    run_advanced_test_suite().await?;

    // Test suite with dependencies and complex configurations
    println!("\nRunning test suite with dependencies and complex configurations...");
    run_complex_test_suite().await?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// Basic test suite demonstration
async fn run_basic_test_suite() -> TestResult<()> {
    // Create a test configuration
    let config = TestConfig::default();

    // Create test cases
    let test1 = test_case!("simple_test_1", |_| {
        println!("Running simple test 1");
        Ok(())
    });

    let test2 = test_case!("simple_test_2", |_| {
        println!("Running simple test 2");
        Ok(())
    });

    let test3 = test_case!("simple_test_3", |_| {
        println!("Running simple test 3");
        // Intentionally fail this test
        Err(navius_test::error::TestError::generic_error(
            "Test 3 fails intentionally",
        ))
    });

    // Create a test suite
    let suite = TestSuiteBuilder::new("basic_test_suite")
        .with_description("A simple demonstration of the Test Suite Framework")
        .with_config(config)
        .add_test(test1, Some(vec!["simple".to_string()]), None)
        .add_test(test2, Some(vec!["simple".to_string()]), None)
        .add_test(
            test3,
            Some(vec!["simple".to_string(), "failing".to_string()]),
            None,
        )
        .with_parallel(false) // Run tests sequentially for better output in this example
        .build();

    // Run the test suite
    let summary = suite.run_all()?;

    // Display the results
    println!("\nTest Suite Summary: {}", suite.metadata.name);
    println!("  Total Tests: {}", summary.total_tests);
    println!("  Passed: {}", summary.passed);
    println!("  Failed: {}", summary.failed);
    println!("  Total Duration: {:?}", summary.total_duration);
    println!("  Average Duration: {:?}", summary.average_duration);

    if let Some(slowest) = &summary.slowest_test {
        println!(
            "  Slowest Test: {} ({:?})",
            slowest,
            summary.slowest_duration.unwrap()
        );
    }

    if !summary.failed_tests.is_empty() {
        println!("\nFailed Tests:");
        for test in &summary.failed_tests {
            println!("  - {}", test);
        }
    }

    Ok(())
}

/// Advanced test suite demonstration with filtering and reporting
async fn run_advanced_test_suite() -> TestResult<()> {
    // Create a test configuration
    let config = TestConfig::default();

    // Create a temporary directory for test reports
    let temp_dir = tempfile::tempdir()?;
    let report_dir = temp_dir.path().to_path_buf();
    println!("Report directory: {}", report_dir.display());

    // Create tests for different components
    let user_tests = vec![
        test_case!("user_create", |_| {
            println!("Testing user creation");
            std::thread::sleep(Duration::from_millis(100));
            Ok(())
        }),
        test_case!("user_update", |_| {
            println!("Testing user update");
            std::thread::sleep(Duration::from_millis(150));
            Ok(())
        }),
        test_case!("user_delete", |_| {
            println!("Testing user deletion");
            std::thread::sleep(Duration::from_millis(50));
            Ok(())
        }),
    ];

    let product_tests = vec![
        test_case!("product_create", |_| {
            println!("Testing product creation");
            std::thread::sleep(Duration::from_millis(125));
            Ok(())
        }),
        test_case!("product_update", |_| {
            println!("Testing product update");
            std::thread::sleep(Duration::from_millis(75));
            Ok(())
        }),
        test_case!("product_delete", |_| {
            println!("Testing product deletion");
            std::thread::sleep(Duration::from_millis(200));
            Ok(())
        }),
    ];

    let order_tests = vec![
        test_case!("order_create", |_| {
            println!("Testing order creation");
            std::thread::sleep(Duration::from_millis(175));
            Ok(())
        }),
        test_case!("order_process", |_| {
            println!("Testing order processing");
            std::thread::sleep(Duration::from_millis(225));
            // Intentionally fail this test
            Err(navius_test::error::TestError::generic_error(
                "Order processing test fails intentionally",
            ))
        }),
        test_case!("order_cancel", |_| {
            println!("Testing order cancellation");
            std::thread::sleep(Duration::from_millis(100));
            Ok(())
        }),
    ];

    // Create a test suite builder
    let mut builder = TestSuiteBuilder::new("advanced_test_suite")
        .with_description("Advanced demonstration of Test Suite Framework")
        .with_config(config.clone())
        .with_report_dir(report_dir)
        .with_report_format(ReportFormat::JSON)
        .with_parallel(true)
        .with_max_concurrent(2);

    // Add user tests with tags
    for test in user_tests {
        builder = builder.add_test(
            test,
            Some(vec!["user".to_string(), "entity".to_string()]),
            None,
        );
    }

    // Add product tests with tags
    for test in product_tests {
        builder = builder.add_test(
            test,
            Some(vec!["product".to_string(), "entity".to_string()]),
            None,
        );
    }

    // Add order tests with tags
    for test in order_tests {
        builder = builder.add_test(
            test,
            Some(vec!["order".to_string(), "transaction".to_string()]),
            None,
        );
    }

    // Build the test suite
    let suite = builder.build();

    // Create a filter to run only entity tests
    let entity_filter = TestFilter {
        include_tags: Some([("entity").to_string()].into_iter().collect()),
        ..Default::default()
    };

    // Run the filtered test suite
    println!("Running entity tests only:");
    let entity_summary = suite.run_filtered(&entity_filter)?;

    // Display the results
    println!("\nEntity Tests Summary:");
    println!("  Total Tests: {}", entity_summary.total_tests);
    println!("  Passed: {}", entity_summary.passed);
    println!("  Failed: {}", entity_summary.failed);

    // Create a filter to run only order tests
    let order_filter = TestFilter {
        include_tags: Some([("order").to_string()].into_iter().collect()),
        ..Default::default()
    };

    // Run the filtered test suite
    println!("\nRunning order tests only:");
    let order_summary = suite.run_filtered(&order_filter)?;

    // Display the results
    println!("\nOrder Tests Summary:");
    println!("  Total Tests: {}", order_summary.total_tests);
    println!("  Passed: {}", order_summary.passed);
    println!("  Failed: {}", order_summary.failed);

    // Show generated report location
    println!("\nTest reports generated in: {}", report_dir.display());

    Ok(())
}

/// Test suite with dependencies and complex configurations
async fn run_complex_test_suite() -> TestResult<()> {
    // Create a test configuration
    let config = TestConfig::default();

    // Create a registry for mocks
    let registry = Arc::new(MockRegistry::new());

    // Create a test context shared by all tests
    let shared_context = IntegrationContext::new(config.clone())?;

    // Create database test with mocks
    let db_test = test_case!("database_connection", move |context| {
        // Get the mock registry
        let registry = context.registry().clone();

        // Create and register a mock database
        let db_mock = MockDatabaseClient::new(&registry);

        // Set up expectations
        db_mock
            .expect_query()
            .with(|query| query.contains("SELECT 1"))
            .returns(Ok(vec![MockQueryResult::new(vec![MockValue::Integer(1)])]));

        // Test the database connection
        let client: Box<dyn DatabaseClient> = Box::new(db_mock);
        let result = client.query("SELECT 1", &[])?;

        assert_eq!(result.len(), 1, "Expected exactly one result");
        assert_eq!(result[0].get::<_, i32>(0)?, 1, "Expected value 1");

        Ok(())
    });

    // Create cache test with dependency on database
    let cache_test = test_case!("cache_connection", move |context| {
        // Get the mock registry
        let registry = context.registry().clone();

        // Create and register a mock cache
        let cache_mock = MockCacheClient::new(&registry);

        // Set up expectations
        cache_mock
            .expect_get()
            .with(|key| key == "test_key")
            .returns(Ok(Some("test_value".to_string())));

        // Test the cache connection
        let client: Box<dyn CacheClient> = Box::new(cache_mock);
        let result = client.get("test_key")?;

        assert_eq!(
            result,
            Some("test_value".to_string()),
            "Expected cache value 'test_value'"
        );

        Ok(())
    });

    // Create a data service test that depends on both database and cache
    let data_service_test = test_case!("data_service", move |context| {
        // This test would use both database and cache services
        // For brevity, we'll just simulate a successful test
        println!("Running data service test with database and cache dependencies");
        Ok(())
    });

    // Create a test suite with dependencies
    let suite = TestSuiteBuilder::new("complex_test_suite")
        .with_description("Complex test suite with dependencies")
        .with_config(config)
        .add_test(
            db_test,
            Some(vec!["database".to_string(), "connection".to_string()]),
            None,
        )
        .add_test(
            cache_test,
            Some(vec!["cache".to_string(), "connection".to_string()]),
            Some(vec!["database_connection".to_string()]),
        )
        .add_test(
            data_service_test,
            Some(vec!["service".to_string(), "integration".to_string()]),
            Some(vec![
                "database_connection".to_string(),
                "cache_connection".to_string(),
            ]),
        )
        .with_parallel(false) // Run sequentially to respect dependencies
        .with_verify_mocks(true)
        .build();

    // Run the test suite
    let summary = suite.run_all()?;

    // Display the results
    println!("\nTest Suite Summary: {}", suite.metadata.name);
    println!("  Total Tests: {}", summary.total_tests);
    println!("  Passed: {}", summary.passed);
    println!("  Failed: {}", summary.failed);
    println!("  Total Duration: {:?}", summary.total_duration);

    Ok(())
}
