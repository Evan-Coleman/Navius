#[cfg(test)]
use super::core::*;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock Application State
struct MockAppState {
    should_fail: Arc<Mutex<bool>>,
}

// Mock Resource Model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MockResource {
    id: i64,
    name: String,
}

// Implement ApiResource for our mock
impl ApiResource for MockResource {
    type Id = i64;

    fn resource_type() -> &'static str {
        "mock"
    }

    fn api_name() -> &'static str {
        "MockAPI"
    }
}

// Mock fetch function that can be configured to succeed or fail
async fn mock_fetch(state: &Arc<MockAppState>, id: i64) -> Result<MockResource, AppError> {
    // Check if we should fail this request
    let should_fail = *state.should_fail.lock().await;

    if should_fail {
        return Err(AppError::ExternalServiceError(
            "Mock service unavailable".to_string(),
        ));
    }

    // Otherwise, return a successful response
    Ok(MockResource {
        id,
        name: format!("Resource {}", id),
    })
}

#[tokio::test]
async fn test_fetch_with_retry_succeeds_on_first_attempt() {
    // Setup
    let state = Arc::new(MockAppState {
        should_fail: Arc::new(Mutex::new(false)), // Set to succeed
    });

    // Execute
    let result = fetch_with_retry(&state, 42, &mock_fetch, 3, true).await;

    // Verify
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource.id, 42);
    assert_eq!(resource.name, "Resource 42");
}

#[tokio::test]
async fn test_fetch_with_retry_fails_after_retries() {
    // Setup
    let state = Arc::new(MockAppState {
        should_fail: Arc::new(Mutex::new(true)), // Set to fail
    });

    // Execute
    let result = fetch_with_retry(&state, 42, &mock_fetch, 3, true).await;

    // Verify
    assert!(result.is_err());
    match result {
        Err(AppError::ExternalServiceError(msg)) => {
            assert!(msg.contains("Mock service unavailable"));
        }
        _ => panic!("Expected ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_fetch_with_retry_succeeds_after_recovery() {
    // Setup
    let should_fail = Arc::new(Mutex::new(true)); // Start with failure
    let state = Arc::new(MockAppState {
        should_fail: should_fail.clone(),
    });

    // Schedule a task to flip the flag after a short delay
    let handle = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let mut lock = should_fail.lock().await;
        *lock = false; // Set to succeed after delay
    });

    // Execute
    let result = fetch_with_retry(&state, 42, &mock_fetch, 3, true).await;

    // Wait for background task
    handle.await.unwrap();

    // Verify
    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource.id, 42);
    assert_eq!(resource.name, "Resource 42");
}

// Add test for caching behavior
#[tokio::test]
async fn test_api_resource_caching() {
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    // Setup a mock cache
    let cache = Arc::new(RwLock::new(HashMap::<i64, MockResource>::new()));

    // Create our mock state
    let state = Arc::new(MockAppState {
        should_fail: Arc::new(Mutex::new(false)), // Always succeed
    });

    // Create a counter to track how many times the fetch function is called
    let fetch_count = Arc::new(Mutex::new(0));

    // Create a fetch function that increments the counter
    let fetch_with_counter = {
        let fetch_count = fetch_count.clone();
        move |state: &Arc<MockAppState>, id: i64| {
            let fetch_count = fetch_count.clone();
            async move {
                // Increment the counter
                let mut count = fetch_count.lock().await;
                *count += 1;

                // Call the original mock fetch
                mock_fetch(state, id).await
            }
        }
    };

    // First request - should miss cache and call the fetch function
    let result1 = check_cache(&cache, 42).await;
    assert!(result1.is_none(), "Cache should be empty initially");

    let resource = fetch_with_counter(&state, 42).await.unwrap();
    store_in_cache(&cache, 42, resource.clone()).await;

    // Verify fetch count
    let count = *fetch_count.lock().await;
    assert_eq!(count, 1, "Fetch function should be called once");

    // Second request - should hit cache and not call the fetch function
    let result2 = check_cache(&cache, 42).await;
    assert!(result2.is_some(), "Cache should have the resource now");
    assert_eq!(result2.unwrap().id, 42);

    // Verify fetch count hasn't changed
    let count = *fetch_count.lock().await;
    assert_eq!(count, 1, "Fetch function shouldn't be called again");
}

// Test the full API handler
#[tokio::test]
async fn test_create_api_handler() {
    use axum::extract::{Path, State};
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    // Setup
    let state = Arc::new(MockAppState {
        should_fail: Arc::new(Mutex::new(false)), // Set to succeed
    });

    // Create a counter to track fetch calls
    let fetch_count = Arc::new(Mutex::new(0));

    // Create a fetch function with counter
    let fetch_with_counter = {
        let fetch_count = fetch_count.clone();
        move |state: &Arc<MockAppState>, id: i64| {
            let fetch_count = fetch_count.clone();
            async move {
                // Increment the counter
                let mut count = fetch_count.lock().await;
                *count += 1;

                // Call the original mock fetch
                mock_fetch(state, id).await
            }
        }
    };

    // Create the API handler with caching enabled
    let handler = create_api_handler(
        fetch_with_counter,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: false, // Disable retries for simplicity
            max_retry_attempts: 3,
            cache_ttl_seconds: 300,
            detailed_logging: true,
        },
    );

    // First call should miss cache
    let result1 = handler(State(state.clone()), Path("42".to_string()))
        .await
        .unwrap();
    let resource1 = result1.0;

    // Verify fetch function was called
    let count = *fetch_count.lock().await;
    assert_eq!(
        count, 1,
        "Fetch function should be called for the first request"
    );

    // Second call should hit cache
    let result2 = handler(State(state.clone()), Path("42".to_string()))
        .await
        .unwrap();
    let resource2 = result2.0;

    // Verify fetch function wasn't called again
    let count = *fetch_count.lock().await;
    assert_eq!(
        count, 1,
        "Fetch function shouldn't be called for the second request"
    );

    // Verify both results are the same
    assert_eq!(resource1.id, resource2.id);
    assert_eq!(resource1.name, resource2.name);
}

// In a real application, you would also test:
// - The cache behavior
// - Error handling
// - Different resource types
// - Edge cases like ID parsing failures
