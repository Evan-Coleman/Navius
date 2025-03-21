// Common test utilities for integration tests
use std::sync::Arc;

// Mock implementations for core services

/// A mock implementation to simulate API responses
pub struct MockApiClient {
    // Fields to control mock behavior
    pub should_fail: bool,
    pub response_data: Option<String>,
    pub status_code: u16,
}

impl MockApiClient {
    pub fn new() -> Self {
        // Default to successful response
        Self {
            should_fail: false,
            response_data: Some(r#"{"id": "test-id", "name": "test-entity"}"#.to_string()),
            status_code: 200,
        }
    }

    pub fn with_failure(status_code: u16) -> Self {
        Self {
            should_fail: true,
            response_data: None,
            status_code,
        }
    }

    pub fn with_response<T: serde::Serialize>(data: &T) -> Self {
        Self {
            should_fail: false,
            response_data: serde_json::to_string(data).ok(),
            status_code: 200,
        }
    }
}

/// A mock application state for testing
pub struct TestAppState {
    pub api_client: Arc<MockApiClient>,
    // Add other mock dependencies as needed
}

impl TestAppState {
    pub fn new() -> Self {
        Self {
            api_client: Arc::new(MockApiClient::new()),
        }
    }
}

/// Helper functions for test setup and teardown

/// Initialize test environment
pub fn setup() {
    // Configure environment for tests
    unsafe {
        std::env::set_var("APP_ENV", "test");
    }

    // Initialize tracing for tests if needed
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}

/// Create test fixtures

/// Create a test fixture with a predefined entity
pub fn create_test_entity() -> serde_json::Value {
    serde_json::json!({
        "id": "test-id-1",
        "name": "Test Entity",
        "created_at": "2023-01-01T00:00:00Z",
        "updated_at": "2023-01-01T00:00:00Z",
        "properties": {
            "attribute1": "value1",
            "attribute2": "value2"
        }
    })
}

/// Generate a random test ID for isolation
pub fn random_test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    format!("test-{}", timestamp)
}
