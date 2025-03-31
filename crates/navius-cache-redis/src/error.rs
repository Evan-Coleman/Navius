use navius_cache::error::CacheError;
use thiserror::Error;

/// Result type for Redis cache operations
pub type RedisCacheResult<T> = Result<T, RedisCacheError>;

/// Redis cache specific errors
#[derive(Error, Debug)]
pub enum RedisCacheError {
    /// Connection error
    #[error("Redis connection error: {0}")]
    ConnectionError(String),

    /// Operation error
    #[error("Redis operation error: {0}")]
    OperationError(String),

    /// Configuration error
    #[error("Redis configuration error: {0}")]
    ConfigurationError(String),

    /// Serialization error
    #[error("Redis serialization error: {0}")]
    SerializationError(String),

    /// Unavailable error
    #[error("Redis unavailable: {0}")]
    Unavailable(String),
}

impl From<redis::RedisError> for RedisCacheError {
    fn from(err: redis::RedisError) -> Self {
        RedisCacheError::OperationError(err.to_string())
    }
}

impl From<serde_json::Error> for RedisCacheError {
    fn from(err: serde_json::Error) -> Self {
        RedisCacheError::SerializationError(err.to_string())
    }
}

impl From<RedisCacheError> for CacheError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::ConnectionError(msg) => CacheError::ConnectionError(msg),
            RedisCacheError::OperationError(msg) => CacheError::OperationError(msg),
            RedisCacheError::ConfigurationError(msg) => CacheError::ConfigurationError(msg),
            RedisCacheError::SerializationError(msg) => CacheError::SerializationError(msg),
            RedisCacheError::Unavailable(msg) => CacheError::UnavailableError(msg),
        }
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
                Err(RedisCacheError::OperationError(format!(
                    "Redis operation '{}' failed: {}",
                    operation, err
                )))
            }
        }
    }
}
