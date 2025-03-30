use navius_cache::error::CacheError;
use redis::RedisError;
use thiserror::Error;

/// Errors that can occur when working with Redis cache
#[derive(Error, Debug)]
pub enum RedisCacheError {
    /// Error from the underlying Redis connection
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    /// Error when connection pooling fails
    #[error("Connection pool error: {0}")]
    ConnectionPool(String),

    /// Error when converting data to/from Redis
    #[error("Data conversion error: {0}")]
    DataConversion(String),

    /// Error when serializing/deserializing data
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Error when a key is not found in the cache
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Error when Redis is unavailable
    #[error("Redis unavailable: {0}")]
    Unavailable(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Unexpected error
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl From<RedisCacheError> for CacheError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::Redis(e) => {
                if e.is_timeout() {
                    CacheError::Timeout(format!("Redis operation timed out: {}", e))
                } else if e.is_connection_dropped() || e.is_connection_refusal() {
                    CacheError::Unavailable(format!("Redis connection error: {}", e))
                } else {
                    CacheError::Provider(format!("Redis error: {}", e))
                }
            }
            RedisCacheError::ConnectionPool(e) => {
                CacheError::Provider(format!("Redis connection pool error: {}", e))
            }
            RedisCacheError::DataConversion(e) => CacheError::DataFormat(e),
            RedisCacheError::Serialization(e) => CacheError::Serialization(e),
            RedisCacheError::KeyNotFound(k) => CacheError::KeyNotFound(k),
            RedisCacheError::Unavailable(e) => CacheError::Unavailable(e),
            RedisCacheError::Configuration(e) => CacheError::Configuration(e),
            RedisCacheError::Timeout(e) => CacheError::Timeout(e),
            RedisCacheError::InvalidOperation(e) => CacheError::InvalidOperation(e),
            RedisCacheError::Unexpected(e) => CacheError::Unexpected(e),
        }
    }
}

impl From<serde_json::Error> for RedisCacheError {
    fn from(err: serde_json::Error) -> Self {
        RedisCacheError::Serialization(err.to_string())
    }
}

/// Result type for Redis cache operations
pub type RedisCacheResult<T> = Result<T, RedisCacheError>;

/// Utility functions for working with Redis errors
pub mod error_helpers {
    use super::*;

    /// Convert a Redis result to a RedisCacheResult, with key context
    pub fn handle_redis_result<T>(
        result: Result<T, RedisError>,
        key: &str,
        operation: &str,
    ) -> RedisCacheResult<T> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                if err.is_timeout() {
                    Err(RedisCacheError::Timeout(format!(
                        "Redis operation '{}' on key '{}' timed out: {}",
                        operation, key, err
                    )))
                } else if err.is_connection_dropped() || err.is_connection_refusal() {
                    Err(RedisCacheError::Unavailable(format!(
                        "Redis connection error during '{}' on key '{}': {}",
                        operation, key, err
                    )))
                } else {
                    Err(RedisCacheError::Redis(err))
                }
            }
        }
    }

    /// Handle a missing key with custom error message
    pub fn handle_missing_key<T>(result: Option<T>, key: &str) -> RedisCacheResult<T> {
        match result {
            Some(value) => Ok(value),
            None => Err(RedisCacheError::KeyNotFound(key.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let redis_err = RedisError::from(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "connection refused",
        ));

        let cache_err = RedisCacheError::Redis(redis_err);
        let converted: CacheError = cache_err.into();

        match converted {
            CacheError::Unavailable(_) => {}
            _ => panic!("Expected Unavailable error type"),
        }
    }

    #[test]
    fn test_serialization_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let cache_err: RedisCacheError = json_err.into();

        match cache_err {
            RedisCacheError::Serialization(_) => {}
            _ => panic!("Expected Serialization error type"),
        }
    }
}
