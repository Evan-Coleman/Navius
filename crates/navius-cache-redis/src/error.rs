use navius_cache::CacheError;
use redis::RedisError;
use serde_json::Error as SerdeError;
use std::fmt;
use thiserror::Error;

/// Result type for Redis cache operations
pub type RedisCacheResult<T> = Result<T, RedisCacheError>;

/// Redis cache specific errors
#[derive(Debug)]
pub enum RedisCacheError {
    ConnectionError(String),
    OperationError(String),
    SerializationError(SerdeError),
    DeserializationError(SerdeError),
    Timeout(String),
    UnsupportedOperation(String),
}

impl fmt::Display for RedisCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedisCacheError::ConnectionError(msg) => write!(f, "Redis connection error: {}", msg),
            RedisCacheError::OperationError(msg) => write!(f, "Redis operation error: {}", msg),
            RedisCacheError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            RedisCacheError::DeserializationError(err) => {
                write!(f, "Deserialization error: {}", err)
            }
            RedisCacheError::Timeout(msg) => write!(f, "Redis timeout: {}", msg),
            RedisCacheError::UnsupportedOperation(msg) => {
                write!(f, "Unsupported operation: {}", msg)
            }
        }
    }
}

impl std::error::Error for RedisCacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RedisCacheError::SerializationError(err) => Some(err),
            RedisCacheError::DeserializationError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<RedisError> for RedisCacheError {
    fn from(err: RedisError) -> Self {
        RedisCacheError::OperationError(err.to_string())
    }
}

impl From<SerdeError> for RedisCacheError {
    fn from(err: SerdeError) -> Self {
        RedisCacheError::SerializationError(err)
    }
}

impl From<RedisCacheError> for CacheError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::SerializationError(err) => {
                CacheError::SerializationError(err.to_string())
            }
            RedisCacheError::DeserializationError(err) => {
                CacheError::DeserializationError(err.to_string())
            }
            _ => CacheError::BackendError(err.to_string()),
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
                Err(RedisCacheError::OperationError(err.to_string()))
            }
        }
    }
}
