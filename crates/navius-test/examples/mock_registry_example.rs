use std::sync::Arc;

use navius_test::{
    error::TestResult,
    mock::{Expectation, ExpectedTimes, MockRegistry},
    mocks::{
        database::MockDatabaseClient,
        http::{HttpMethod, MockHttpClient},
    },
};

/// This example demonstrates how to use the Mock Interface Registry to set up
/// and verify expectations on mock implementations.
fn main() -> TestResult<()> {
    println!("Mock Registry Example");
    println!("=====================\n");

    // Create a basic registry example
    basic_registry_example()?;

    // Create a more complex example with multiple mocks and expectations
    complex_registry_example()?;

    println!("\nAll examples completed successfully!");
    Ok(())
}

/// A basic example that shows how to set up a single mock with one expectation
fn basic_registry_example() -> TestResult<()> {
    println!("Basic Registry Example");
    println!("---------------------");

    // Create a new mock registry
    let registry = Arc::new(MockRegistry::new());

    // Create a mock database client
    let mock_db = MockDatabaseClient::new();

    // Register the mock with the registry
    registry.register::<dyn navius_test::mocks::database::DatabaseClient, _>(Arc::new(mock_db))?;

    // Set up an expectation for a query method
    registry
        .expect("DatabaseClient", "query")
        .with_args(vec!["SELECT * FROM users WHERE id = ?", "1"])
        .times(ExpectedTimes::Exact(1))
        .returns(Ok(vec![("id", "1"), ("name", "John Doe")].into()))?;

    // Use the mock in our code
    println!("Running database query...");

    // Simulate a call to the mock
    registry.record_call(
        "DatabaseClient",
        "query",
        vec![
            "SELECT * FROM users WHERE id = ?".to_string(),
            "1".to_string(),
        ],
    )?;

    // Verify that all expectations were met
    registry.verify()?;

    println!("Basic example completed successfully");
    Ok(())
}

/// A more complex example with multiple mocks and expectations
fn complex_registry_example() -> TestResult<()> {
    println!("\nComplex Registry Example");
    println!("-----------------------");

    // Create a new mock registry
    let registry = Arc::new(MockRegistry::new());

    // Create and register a mock database client
    let mock_db = MockDatabaseClient::new();
    registry.register::<dyn navius_test::mocks::database::DatabaseClient, _>(Arc::new(mock_db))?;

    // Create and register a mock HTTP client
    let mock_http = MockHttpClient::default();
    registry.register::<dyn navius_test::mocks::http::HttpClient, _>(Arc::new(mock_http))?;

    // Set up expectations for the database client
    registry
        .expect("DatabaseClient", "query")
        .with_args(vec!["SELECT * FROM users", ""])
        .times(ExpectedTimes::Exact(1))
        .returns(Ok(vec![
            ("id", "1"),
            ("name", "John Doe"),
            ("id", "2"),
            ("name", "Jane Smith"),
        ]
        .into()))?;

    registry
        .expect("DatabaseClient", "execute")
        .with_args(vec![
            "INSERT INTO audit_log VALUES (?, ?)",
            "user_query",
            "list_all_users",
        ])
        .times(ExpectedTimes::AtLeast(1))
        .returns(Ok(1))?;

    // Set up expectations for the HTTP client
    registry
        .expect("HttpClient", "send")
        .with_args(vec![
            HttpMethod::GET.to_string(),
            "https://api.example.com/metrics".to_string(),
        ])
        .times(ExpectedTimes::Exact(1))
        .returns(Ok("metric data".into()))?;

    // Simulate calls to the mocks
    println!("Running database and HTTP operations...");

    // Simulate database query
    registry.record_call(
        "DatabaseClient",
        "query",
        vec!["SELECT * FROM users".to_string(), "".to_string()],
    )?;

    // Simulate database execute (audit logging)
    registry.record_call(
        "DatabaseClient",
        "execute",
        vec![
            "INSERT INTO audit_log VALUES (?, ?)".to_string(),
            "user_query".to_string(),
            "list_all_users".to_string(),
        ],
    )?;

    // Simulate HTTP request
    registry.record_call(
        "HttpClient",
        "send",
        vec![
            HttpMethod::GET.to_string(),
            "https://api.example.com/metrics".to_string(),
        ],
    )?;

    // Verify that all expectations were met
    registry.verify()?;

    println!("Complex example completed successfully");
    Ok(())
}

// A more realistic example of how this would be used in an actual test
#[cfg(test)]
mod tests {
    use super::*;
    use navius_test::{TestResult, fixture::TestFixture};

    #[test]
    fn test_user_service_with_mocks() -> TestResult<()> {
        // Create a test fixture with a mock registry
        let fixture = TestFixture::new();
        let registry = Arc::new(MockRegistry::new());
        fixture.register_component(registry.clone())?;

        // Set up mock database expectations
        registry
            .expect::<MockDatabaseClient>("DatabaseClient", "query")
            .with_args(vec!["SELECT * FROM users WHERE id = ?", "1"])
            .returns(Ok(vec![("id", "1"), ("name", "John Doe")].into()))?;

        // Set up mock HTTP client expectations
        registry
            .expect::<MockHttpClient>("HttpClient", "send")
            .with_args(vec![
                HttpMethod::GET.to_string(),
                "https://api.example.com/users/1".to_string(),
            ])
            .returns(Ok(r#"{"id": 1, "name": "John Doe"}"#.into()))?;

        // Simulate method calls
        registry.record_call(
            "DatabaseClient",
            "query",
            vec![
                "SELECT * FROM users WHERE id = ?".to_string(),
                "1".to_string(),
            ],
        )?;

        registry.record_call(
            "HttpClient",
            "send",
            vec![
                HttpMethod::GET.to_string(),
                "https://api.example.com/users/1".to_string(),
            ],
        )?;

        // Verify expectations
        registry.verify()?;

        Ok(())
    }
}
