use navius_cache::error::CacheError;
use redis::RedisError;
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

/// Redis cache error type
#[derive(Debug, Error, Clone)]
pub enum RedisCacheError {
    /// Redis connection error
    #[error("Redis connection error: {0}")]
    Connection(String),

    /// Redis command error
    #[error("Redis command error: {0}")]
    Command(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Invalid key error
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<RedisError> for RedisCacheError {
    fn from(err: RedisError) -> Self {
        if err.is_io_error() {
            RedisCacheError::Connection(err.to_string())
        } else if err.is_timeout() {
            RedisCacheError::Timeout(err.to_string())
        } else {
            RedisCacheError::Command(err.to_string())
        }
    }
}

impl From<serde_json::Error> for RedisCacheError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() {
            RedisCacheError::Deserialization(err.to_string())
        } else {
            RedisCacheError::Serialization(err.to_string())
        }
    }
}

impl From<rmp_serde::encode::Error> for RedisCacheError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        RedisCacheError::Serialization(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for RedisCacheError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        RedisCacheError::Deserialization(err.to_string())
    }
}

impl From<RedisCacheError> for CacheError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::Connection(msg) => CacheError::ConnectionError(msg),
            RedisCacheError::Command(msg) => CacheError::OperationError(msg),
            RedisCacheError::Serialization(msg) => CacheError::SerializationError(msg),
            RedisCacheError::Deserialization(msg) => CacheError::SerializationError(msg),
            RedisCacheError::Timeout(msg) => {
                CacheError::OperationError(format!("Timeout: {}", msg))
            }
            RedisCacheError::InvalidKey(msg) => {
                CacheError::OperationError(format!("Invalid key: {}", msg))
            }
            RedisCacheError::Configuration(msg) => CacheError::ConfigurationError(msg),
        }
    }
}

/// Helper function to convert a Redis result to a Cache result
pub fn to_cache_result<T>(result: Result<T, RedisError>) -> Result<T, CacheError> {
    result.map_err(|err| {
        let redis_err = RedisCacheError::from(err);
        CacheError::from(redis_err)
    })
}

/// Redis cache result type
pub type RedisCacheResult<T> = Result<T, RedisCacheError>;

// Fix error conversion for RedisCacheError to RedisError
impl From<RedisCacheError> for redis::RedisError {
    fn from(err: RedisCacheError) -> Self {
        match err {
            RedisCacheError::Command(msg) => {
                redis::RedisError::from((redis::ErrorKind::ResponseError, "Command error", msg))
            }
            RedisCacheError::Connection(msg) => {
                redis::RedisError::from((redis::ErrorKind::IoError, "Connection error", msg))
            }
            RedisCacheError::Timeout(msg) => {
                redis::RedisError::from((redis::ErrorKind::IoError, "Timeout error", msg))
            }
            RedisCacheError::Serialization(msg) => {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Serialization error", msg))
            }
            RedisCacheError::Deserialization(msg) => {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization error", msg))
            }
            RedisCacheError::InvalidKey(msg) => {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Invalid key error", msg))
            }
            RedisCacheError::Configuration(msg) => {
                redis::RedisError::from((redis::ErrorKind::TypeError, "Configuration error", msg))
            }
        }
    }
}

// Implement From<CacheError> for RedisCacheError to help with error conversion
impl From<CacheError> for RedisCacheError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::ConnectionError(msg) => RedisCacheError::Connection(msg),
            CacheError::OperationError(msg) => RedisCacheError::Command(msg),
            CacheError::SerializationError(msg) => RedisCacheError::Serialization(msg),
            CacheError::NotFoundError(msg) => {
                RedisCacheError::Command(format!("Not found: {}", msg))
            }
            CacheError::ConfigurationError(msg) => RedisCacheError::Configuration(msg),
            CacheError::TimeoutError(msg) => RedisCacheError::Timeout(msg),
            CacheError::InvalidationError(msg) => {
                RedisCacheError::Command(format!("Invalidation error: {}", msg))
            }
            CacheError::BackendError(msg) => {
                RedisCacheError::Command(format!("Backend error: {}", msg))
            }
            CacheError::UnsupportedOperation(msg) => {
                RedisCacheError::Command(format!("Unsupported operation: {}", msg))
            }
            CacheError::Redis(err) => RedisCacheError::from(err),
            CacheError::Serialization(err) => RedisCacheError::Serialization(err.to_string()),
            CacheError::InvalidArgument(msg) => RedisCacheError::InvalidKey(msg),
            CacheError::LuaManagerNotInitialized => {
                RedisCacheError::Command("Lua manager not initialized".to_string())
            }
        }
    }
}
