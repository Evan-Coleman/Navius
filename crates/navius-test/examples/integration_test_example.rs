//! Example of using the integration test utilities
//!
//! This example demonstrates how to use the integration test utilities
//! to test interactions between multiple crates in the Navius workspace.

use std::collections::HashMap;
use std::sync::Arc;

use navius_test::{
    config::TestConfigBuilder,
    error::TestResult,
    integration::{CrossCrateTestBuilder, IntegrationContext},
    test_case, test_suite,
};

// Mock service implementations for demonstration
struct UserService {
    name: String,
    db_client: Arc<dyn DatabaseClient>,
}

struct User {
    id: i32,
    name: String,
    email: String,
}

// Mock trait for database client
trait DatabaseClient: Send + Sync {
    fn query(&self, query: &str, params: &[Value]) -> Result<Vec<Row>, DbError>;
    fn execute(&self, query: &str, params: &[Value]) -> Result<usize, DbError>;
}

// Simple database value type for the example
#[derive(Clone, Debug)]
enum Value {
    Integer(i32),
    Text(String),
}

// Simple row type for the example
#[derive(Clone, Debug)]
struct Row {
    values: HashMap<String, Value>,
}

impl Row {
    fn new(values: HashMap<String, Value>) -> Self {
        Self { values }
    }

    fn get_i32(&self, name: &str) -> Option<i32> {
        match self.values.get(name) {
            Some(Value::Integer(i)) => Some(*i),
            _ => None,
        }
    }

    fn get_string(&self, name: &str) -> Option<String> {
        match self.values.get(name) {
            Some(Value::Text(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

// Simple database error type for the example
#[derive(Clone, Debug)]
enum DbError {
    ConnectionFailed(String),
    QueryFailed(String),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            DbError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

// User service implementation
impl UserService {
    fn new(name: impl Into<String>, db_client: Arc<dyn DatabaseClient>) -> Self {
        Self {
            name: name.into(),
            db_client,
        }
    }

    fn get_user(&self, id: i32) -> Result<User, String> {
        let params = vec![Value::Integer(id)];
        let rows = self
            .db_client
            .query("SELECT * FROM users WHERE id = ?", &params)
            .map_err(|e| format!("Database error: {}", e))?;

        if rows.is_empty() {
            return Err(format!("User with id {} not found", id));
        }

        let row = &rows[0];
        Ok(User {
            id: row.get_i32("id").unwrap(),
            name: row.get_string("name").unwrap(),
            email: row.get_string("email").unwrap(),
        })
    }

    fn create_user(&self, name: &str, email: &str) -> Result<i32, String> {
        let params = vec![
            Value::Text(name.to_string()),
            Value::Text(email.to_string()),
        ];

        self.db_client
            .execute("INSERT INTO users (name, email) VALUES (?, ?)", &params)
            .map_err(|e| format!("Database error: {}", e))?;

        // For simplicity, return a dummy ID
        Ok(123)
    }
}

fn main() -> TestResult<()> {
    // Create test configuration
    let config = TestConfigBuilder::new("integration_tests")
        .with_description("Integration test examples")
        .with_test_timeout(60)
        .with_env_var("APP_ENV", "test")
        .build();

    // Create test cases using the test_case macro
    let test1 = test_case!("user_service_get_user", |context| {
        // Set up test dependencies
        let mut context_mut = context.to_owned();
        let fixture = context_mut.create_fixture()?;
        let registry = context.registry();

        // Configure mock database expectations
        let mut user_data = HashMap::new();
        user_data.insert("id".to_string(), Value::Integer(1));
        user_data.insert("name".to_string(), Value::Text("John Doe".to_string()));
        user_data.insert(
            "email".to_string(),
            Value::Text("john@example.com".to_string()),
        );

        let user_row = Row::new(user_data);

        registry
            .expect("DatabaseClient", "query")
            .with(("SELECT * FROM users WHERE id = ?", vec![Value::Integer(1)]))
            .returns(Ok(vec![user_row]));

        // Create service with mock database
        let user_service = UserService::new(
            "test_service",
            Arc::new(MockDatabaseClient::new(registry.clone())),
        );

        // Test the service
        let user = user_service.get_user(1)?;

        // Verify the result
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");

        Ok(())
    });

    let test2 = test_case!("user_service_create_user", |context| {
        // Set up test dependencies
        let mut context_mut = context.to_owned();
        let fixture = context_mut.create_fixture()?;
        let registry = context.registry();

        // Configure mock database expectations
        registry
            .expect("DatabaseClient", "execute")
            .with((
                "INSERT INTO users (name, email) VALUES (?, ?)",
                vec![
                    Value::Text("Jane Doe".to_string()),
                    Value::Text("jane@example.com".to_string()),
                ],
            ))
            .returns(Ok(1));

        // Create service with mock database
        let user_service = UserService::new(
            "test_service",
            Arc::new(MockDatabaseClient::new(registry.clone())),
        );

        // Test the service
        let user_id = user_service.create_user("Jane Doe", "jane@example.com")?;

        // Verify the result
        assert_eq!(user_id, 123);

        Ok(())
    });

    // Create and run a test suite
    let runner = test_suite!("user_service_tests", config, test1, test2);
    let reports = runner.run_all()?;

    // Check for failures
    let failures = reports.iter().filter(|r| !r.passed).count();
    assert_eq!(failures, 0, "Some tests failed");

    // Print a success message
    println!("All tests passed!");

    Ok(())
}

// Mock implementation of the database client
struct MockDatabaseClient {
    registry: Arc<navius_test::mock::MockRegistry>,
}

impl MockDatabaseClient {
    fn new(registry: Arc<navius_test::mock::MockRegistry>) -> Self {
        Self { registry }
    }
}

impl DatabaseClient for MockDatabaseClient {
    fn query(&self, query: &str, params: &[Value]) -> Result<Vec<Row>, DbError> {
        let args = Box::new((query.to_string(), params.to_vec()));

        // Record the method call
        self.registry
            .record_call("DatabaseClient", "query", Some(args.clone()))
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        // Get the expected result
        self.registry
            .get_result("DatabaseClient", "query", Some(args))
            .map_err(|e| DbError::QueryFailed(e.to_string()))
    }

    fn execute(&self, query: &str, params: &[Value]) -> Result<usize, DbError> {
        let args = Box::new((query.to_string(), params.to_vec()));

        // Record the method call
        self.registry
            .record_call("DatabaseClient", "execute", Some(args.clone()))
            .map_err(|e| DbError::QueryFailed(e.to_string()))?;

        // Get the expected result
        self.registry
            .get_result("DatabaseClient", "execute", Some(args))
            .map_err(|e| DbError::QueryFailed(e.to_string()))
    }
}
