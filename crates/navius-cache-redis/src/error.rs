use navius_cache::error::CacheError;
use thiserror::Error;

/// Result type for Redis cache operations
pub type RedisCacheResult<T> = Result<T, RedisCacheError>;

/// Redis cache specific errors
#[derive(Error, Debug, Clone)]
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

    /// Timeout error
    #[error("Redis operation timed out: {0}")]
    Timeout(String),

    /// Script error
    #[error("Redis script error: {0}")]
    ScriptError(String),

    /// Key not found error
    #[error("Key not found in Redis: {0}")]
    KeyNotFound(String),

    /// Command error
    #[error("Redis command error: {0}")]
    CommandError(String),

    /// Deserialization error
    #[error("Failed to deserialize Redis data: {0}")]
    DeserializationError(String),

    /// Operation timed out.
    #[error("Operation timed out: {0}")]
    TimeoutError(String),

    /// Generic internal error.
    #[error("Internal error: {0}")]
    InternalError(String),
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

/// Conversion from RedisCacheError to the generic CacheError
impl From<RedisCacheError> for CacheError {
    fn from(error: RedisCacheError) -> Self {
        match error {
            RedisCacheError::ConnectionError(msg) => CacheError::ConnectionError(msg),
            RedisCacheError::OperationError(msg) => CacheError::OperationError(msg),
            RedisCacheError::ConfigurationError(msg) => CacheError::ConfigurationError(msg),
            RedisCacheError::SerializationError(msg) => CacheError::SerializationError(msg),
            RedisCacheError::DeserializationError(msg) => CacheError::SerializationError(msg), // Map DeserializationError
            RedisCacheError::Unavailable(msg) => CacheError::UnavailableError(msg), // Map Unavailable
            RedisCacheError::Timeout(msg) => CacheError::TimeoutError(msg),         // Map Timeout
            RedisCacheError::ScriptError(msg) => CacheError::OperationError(msg), // Map ScriptError
            RedisCacheError::KeyNotFound(msg) => CacheError::NotFoundError(msg),  // Map KeyNotFound
            RedisCacheError::CommandError(msg) => CacheError::OperationError(msg), // Map CommandError
            RedisCacheError::TimeoutError(msg) => CacheError::TimeoutError(msg), // Map TimeoutError (the one we added)
            RedisCacheError::InternalError(msg) => CacheError::InternalError(msg),
        }
    }
}

impl From<RedisCacheError> for redis::RedisError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::ConnectionError(msg) => {
                redis::RedisError::from((redis::ErrorKind::IoError, "Connection error", msg))
            }
            RedisCacheError::OperationError(msg) => {
                redis::RedisError::from((redis::ErrorKind::ResponseError, "Operation error", msg))
            }
            RedisCacheError::ConfigurationError(msg) => {
                redis::RedisError::from((redis::ErrorKind::ClientError, "Configuration error", msg))
            }
            RedisCacheError::SerializationError(msg) => {
                redis::RedisError::from((redis::ErrorKind::ClientError, "Serialization error", msg))
            }
            RedisCacheError::Unavailable(msg) => {
                redis::RedisError::from((redis::ErrorKind::IoError, "Service unavailable", msg))
            }
            RedisCacheError::Timeout(msg) => {
                redis::RedisError::from((redis::ErrorKind::IoError, "Timeout", msg))
            }
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
