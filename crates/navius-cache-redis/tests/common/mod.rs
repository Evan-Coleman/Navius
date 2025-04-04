//! Common test utilities for Redis cache tests

/// Start or connect to a Redis server for testing
///
/// Returns the URL of the Redis server
///
/// This implementation simply returns a URL for a local Redis instance.
/// In a real CI environment, you might want to start a Redis container
/// or connect to a real Redis instance.
pub fn start_redis_server() -> String {
    // Use environment variable if provided
    if let Ok(url) = std::env::var("REDIS_TEST_URL") {
        return url;
    }

    // Default to localhost:6379
    "redis://127.0.0.1:6379/0".to_string()
}
