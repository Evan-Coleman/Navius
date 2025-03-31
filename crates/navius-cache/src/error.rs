use navius_core::error::Error as AppError;
use thiserror::Error;

/// Cache errors that can occur during cache operations
#[derive(Debug, Error)]
pub enum CacheError {
    /// Connection errors
    #[error("Failed to connect to cache: {0}")]
    ConnectionError(String),

    /// Operation errors
    #[error("Cache operation failed: {0}")]
    OperationError(String),

    /// Serialization errors
    #[error("Failed to serialize or deserialize cache data: {0}")]
    SerializationError(String),

    /// Key not found
    #[error("Key not found in cache: {0}")]
    NotFoundError(String),

    /// Configuration errors
    #[error("Invalid cache configuration: {0}")]
    ConfigurationError(String),

    /// Timeout errors
    #[error("Cache operation timed out: {0}")]
    TimeoutError(String),

    /// Invalidation errors
    #[error("Cache invalidation failed: {0}")]
    InvalidationError(String),

    /// Backend errors (Redis, etc.)
    #[error("Cache backend error: {0}")]
    BackendError(String),
}

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Implement From<CacheError> for AppError
impl From<CacheError> for AppError {
    fn from(err: CacheError) -> Self {
        match err {
            CacheError::ConnectionError(msg) => {
                AppError::internal(format!("Cache connection error: {}", msg))
            }
            CacheError::OperationError(msg) => {
                AppError::internal(format!("Cache operation error: {}", msg))
            }
            CacheError::SerializationError(msg) => {
                AppError::internal(format!("Cache serialization error: {}", msg))
            }
            CacheError::NotFoundError(msg) => {
                AppError::not_found(format!("Cache key not found: {}", msg))
            }
            CacheError::ConfigurationError(msg) => {
                AppError::configuration(format!("Cache configuration error: {}", msg))
            }
            CacheError::TimeoutError(msg) => AppError::internal(format!("Cache timeout: {}", msg)),
            CacheError::InvalidationError(msg) => {
                AppError::internal(format!("Cache invalidation error: {}", msg))
            }
            CacheError::BackendError(msg) => {
                AppError::internal(format!("Cache backend error: {}", msg))
            }
        }
    }
}

/// Implement From<redis::RedisError> for CacheError
#[cfg(feature = "redis")]
impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        match err.kind() {
            redis::ErrorKind::IoError => Self::ConnectionError(err.to_string()),
            redis::ErrorKind::ResponseError => Self::OperationError(err.to_string()),
            redis::ErrorKind::AuthenticationFailed => {
                Self::ConnectionError(format!("Redis authentication failed: {}", err))
            }
            redis::ErrorKind::BusyLoadingError => {
                Self::BackendError(format!("Redis busy loading: {}", err))
            }
            redis::ErrorKind::NoScriptError => {
                Self::OperationError(format!("Redis script error: {}", err))
            }
            redis::ErrorKind::ExtensionError => {
                Self::BackendError(format!("Redis extension error: {}", err))
            }
            redis::ErrorKind::ClientError => {
                Self::OperationError(format!("Redis client error: {}", err))
            }
            _ => Self::BackendError(format!("Unknown Redis error: {}", err)),
        }
    }
}
