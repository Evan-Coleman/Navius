use navius_cache::error::CacheError;
use thiserror::Error;

/// Result type for Redis cache operations
pub type RedisCacheResult<T> = Result<T, RedisCacheError>;

/// Redis cache specific errors
#[derive(Debug, Error)]
pub enum RedisCacheError {
    /// Connection error with Redis
    #[error("Redis error: {0}")]
    Redis(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Operation error
    #[error("Operation error: {0}")]
    Operation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Invalid key
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Circuit breaker error
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Script error
    #[error("Script error: {0}")]
    ScriptError(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Command execution error
    #[error("Command error: {0}")]
    CommandError(String),

    /// Unknown error
    #[error("Unknown error: {0}")]
    UnknownError(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    Unavailable(String),
}

impl From<RedisCacheError> for CacheError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::Redis(e) => CacheError::Redis(e),
            RedisCacheError::Serialization(e) => CacheError::Serialization(e),
            RedisCacheError::Timeout(e) => CacheError::Timeout(e),
            RedisCacheError::InvalidKey(e) => CacheError::InvalidKey(e),
            RedisCacheError::CircuitBreakerOpen(e) => CacheError::CircuitBreakerOpen(e),
            RedisCacheError::Operation(e) => CacheError::OperationError(e),
            RedisCacheError::ConfigurationError(e) => CacheError::ConfigurationError(e),
            RedisCacheError::InternalError(e) => CacheError::InternalError(e),
            RedisCacheError::Connection(e) => CacheError::ConnectionError(e),
            RedisCacheError::ScriptError(e) => CacheError::OperationError(e),
            RedisCacheError::KeyNotFound(e) => CacheError::NotFoundError(e),
            RedisCacheError::CommandError(e) => CacheError::OperationError(e),
            RedisCacheError::UnknownError(e) => CacheError::InternalError(e),
            RedisCacheError::Unavailable(e) => CacheError::UnavailableError(e),
        }
    }
}

impl From<redis::RedisError> for RedisCacheError {
    fn from(err: redis::RedisError) -> Self {
        RedisCacheError::Redis(err.to_string())
    }
}

impl From<serde_json::Error> for RedisCacheError {
    fn from(err: serde_json::Error) -> Self {
        RedisCacheError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for RedisCacheError {
    fn from(err: std::io::Error) -> Self {
        RedisCacheError::Operation(err.to_string())
    }
}

/// Helper functions for error handling
pub mod error_helpers {
    use super::{RedisCacheError, RedisCacheResult};
    use redis::RedisError;
    use tracing::error;

    /// Handle result from Redis operation
    pub fn handle_redis_result<T>(
        result: Result<T, RedisError>,
        key: &str,
        operation: &str,
    ) -> RedisCacheResult<T> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                error!(
                    "Redis operation '{}' failed on key '{}': {}",
                    operation, key, err
                );
                Err(RedisCacheError::Operation(format!(
                    "Redis operation '{}' failed: {}",
                    operation, err
                )))
            }
        }
    }
}
