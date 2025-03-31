// Integration tests for the navius-core error handling system
// These tests validate error propagation and context handling

use navius_core::error::{Error, ErrorCode, Result};
use navius_test::{
    TestFixture, TestResult,
    error::{assert_contains, assert_eq, assert_true},
    mock::{Expectation, Mock, MockRegistry},
};
use serde_json::Value;
use std::error::Error as StdError;
use std::sync::{Arc, Mutex};

// ----------- Mocks --------------

#[derive(Debug)]
struct MockDatabaseError {
    message: String,
}

impl std::fmt::Display for MockDatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database error: {}", self.message)
    }
}

impl StdError for MockDatabaseError {}

struct MockDatabase {
    registry: Arc<Mutex<MockRegistry>>,
    name: String,
}

impl MockDatabase {
    fn new(fixture: &TestFixture, name: &str) -> Self {
        Self {
            registry: fixture.mock_registry(),
            name: name.to_string(),
        }
    }

    fn query(&self, sql: &str) -> std::result::Result<Value, MockDatabaseError> {
        let result = self
            .registry
            .lock()
            .unwrap()
            .execute(&self.name, "query", &[sql.to_string()]);

        match result {
            Ok(value) => Ok(value.to_owned()),
            Err(msg) => Err(MockDatabaseError { message: msg }),
        }
    }
}

struct MockFileSystem {
    registry: Arc<Mutex<MockRegistry>>,
    name: String,
}

impl MockFileSystem {
    fn new(fixture: &TestFixture, name: &str) -> Self {
        Self {
            registry: fixture.mock_registry(),
            name: name.to_string(),
        }
    }

    fn read_file(&self, path: &str) -> std::io::Result<String> {
        let result =
            self.registry
                .lock()
                .unwrap()
                .execute(&self.name, "read_file", &[path.to_string()]);

        match result {
            Ok(value) => Ok(value.to_string()),
            Err(msg) => Err(std::io::Error::new(std::io::ErrorKind::Other, msg)),
        }
    }

    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()> {
        let result = self.registry.lock().unwrap().execute(
            &self.name,
            "write_file",
            &[path.to_string(), content.to_string()],
        );

        match result {
            Ok(_) => Ok(()),
            Err(msg) => Err(std::io::Error::new(std::io::ErrorKind::Other, msg)),
        }
    }
}

// ----------- Functions Under Test --------------

fn perform_database_operation(db: &MockDatabase, user_id: &str) -> Result<Value> {
    let sql = format!("SELECT * FROM users WHERE id = '{}'", user_id);
    db.query(&sql).map_err(|e| Error::database(&e.to_string()))
}

fn perform_file_operation(fs: &MockFileSystem, path: &str) -> Result<String> {
    fs.read_file(path).map_err(|e| Error::io(&e.to_string()))
}

fn complex_operation(db: &MockDatabase, fs: &MockFileSystem, user_id: &str) -> Result<String> {
    let user_data = perform_database_operation(db, user_id)?;

    let config_path = format!("/config/{}.json", user_id);
    let config = perform_file_operation(fs, &config_path)
        .map_err(|e| e.with_context(format!("Failed to load config for user {}", user_id)))?;

    Ok(format!(
        "User: {}, Config: {}",
        user_data["name"].as_str().unwrap_or("unknown"),
        config
    ))
}

// ----------- Tests --------------

#[test]
fn test_error_propagation() -> TestResult<()> {
    let fixture = TestFixture::new("test_error_propagation");
    let db = MockDatabase::new(&fixture, "db");

    fixture.mock_registry().lock().unwrap().expect(
        "db",
        "query",
        Expectation::once()
            .with(vec!["SELECT * FROM users WHERE id = 'user123'".to_string()])
            .returning(Err("User not found".to_string())),
    );

    let result = perform_database_operation(&db, "user123");

    assert_true(result.is_err(), "Operation should have failed")?;
    let err = result.unwrap_err();
    assert_eq(
        err.code,
        ErrorCode::Database,
        "Error code should be Database",
    )?;
    assert_contains(
        err.message,
        "User not found",
        "Error message should contain the original error",
    )?;

    fixture
        .mock_registry()
        .lock()
        .unwrap()
        .verify_expectations()
}

#[test]
fn test_error_with_context() -> TestResult<()> {
    let fixture = TestFixture::new("test_error_with_context");
    let db = MockDatabase::new(&fixture, "db");
    let fs = MockFileSystem::new(&fixture, "fs");

    // Set up expectations for successful database query
    fixture.mock_registry().lock().unwrap().expect(
        "db",
        "query",
        Expectation::once()
            .with(vec!["SELECT * FROM users WHERE id = 'user456'".to_string()])
            .returning(Ok(serde_json::json!({
                "id": "user456",
                "name": "John Doe"
            }))),
    );

    // Set up expectations for failed file operation
    fixture.mock_registry().lock().unwrap().expect(
        "fs",
        "read_file",
        Expectation::once()
            .with(vec!["/config/user456.json".to_string()])
            .returning(Err("File not found".to_string())),
    );

    let result = complex_operation(&db, &fs, "user456");

    assert_true(result.is_err(), "Operation should have failed")?;
    let err = result.unwrap_err();
    assert_eq(err.code, ErrorCode::IO, "Error code should be IO")?;
    assert_contains(
        err.message,
        "File not found",
        "Error message should contain the original error",
    )?;

    // Check for context
    let context = err.context.unwrap();
    assert_contains(
        context,
        "Failed to load config for user user456",
        "Error context should contain the added context",
    )?;

    fixture
        .mock_registry()
        .lock()
        .unwrap()
        .verify_expectations()
}

#[test]
fn test_database_error_handling() -> TestResult<()> {
    let fixture = TestFixture::new("test_database_error_handling");
    let db = MockDatabase::new(&fixture, "db");

    fixture.mock_registry().lock().unwrap().expect(
        "db",
        "query",
        Expectation::once()
            .with(vec!["SELECT * FROM users WHERE id = 'invalid'".to_string()])
            .returning(Err("SQL syntax error".to_string())),
    );

    let result = perform_database_operation(&db, "invalid");

    assert_true(result.is_err(), "Operation should have failed")?;
    let err = result.unwrap_err();
    assert_eq(
        err.code,
        ErrorCode::Database,
        "Error code should be Database",
    )?;
    assert_contains(
        err.message,
        "SQL syntax error",
        "Error message should contain SQL error details",
    )?;

    let json = err.to_json();
    assert_eq(
        json["error"]["code"],
        "database",
        "JSON error code should be 'database'",
    )?;
    assert_eq(
        json["error"]["status"],
        500,
        "Database errors should have 500 status code",
    )?;

    fixture
        .mock_registry()
        .lock()
        .unwrap()
        .verify_expectations()
}

#[test]
fn test_file_error_handling() -> TestResult<()> {
    let fixture = TestFixture::new("test_file_error_handling");
    let fs = MockFileSystem::new(&fixture, "fs");

    fixture.mock_registry().lock().unwrap().expect(
        "fs",
        "read_file",
        Expectation::once()
            .with(vec!["/data/missing.txt".to_string()])
            .returning(Err("No such file or directory".to_string())),
    );

    let result = perform_file_operation(&fs, "/data/missing.txt");

    assert_true(result.is_err(), "Operation should have failed")?;
    let err = result.unwrap_err();
    assert_eq(err.code, ErrorCode::IO, "Error code should be IO")?;
    assert_contains(
        err.message,
        "No such file or directory",
        "Error message should contain file system error details",
    )?;

    let json = err.to_json();
    assert_eq(
        json["error"]["code"],
        "io",
        "JSON error code should be 'io'",
    )?;
    assert_eq(
        json["error"]["status"],
        500,
        "IO errors should have 500 status code",
    )?;

    fixture
        .mock_registry()
        .lock()
        .unwrap()
        .verify_expectations()
}

#[test]
fn test_successful_operation() -> TestResult<()> {
    let fixture = TestFixture::new("test_successful_operation");
    let db = MockDatabase::new(&fixture, "db");
    let fs = MockFileSystem::new(&fixture, "fs");

    // Set up expectations for successful database query
    fixture.mock_registry().lock().unwrap().expect(
        "db",
        "query",
        Expectation::once()
            .with(vec!["SELECT * FROM users WHERE id = 'user789'".to_string()])
            .returning(Ok(serde_json::json!({
                "id": "user789",
                "name": "Jane Smith"
            }))),
    );

    // Set up expectations for successful file operation
    fixture.mock_registry().lock().unwrap().expect(
        "fs",
        "read_file",
        Expectation::once()
            .with(vec!["/config/user789.json".to_string()])
            .returning(Ok("{ \"theme\": \"dark\" }".to_string())),
    );

    let result = complex_operation(&db, &fs, "user789");

    assert_true(result.is_ok(), "Operation should have succeeded")?;
    assert_eq(
        result.unwrap(),
        "User: Jane Smith, Config: { \"theme\": \"dark\" }",
        "Result should contain user data and config",
    )?;

    fixture
        .mock_registry()
        .lock()
        .unwrap()
        .verify_expectations()
}
