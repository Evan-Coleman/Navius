use navius_cache::error::{CacheError, CacheResult};
use thiserror::Error;

/// Error type for Redis cache operations
#[derive(Error, Debug)]
pub enum RedisError {
    /// Connection errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Operation errors
    #[error("Operation error: {0}")]
    Operation(String),

    /// Command errors
    #[error("Command error: {0}")]
    Command(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Cluster errors
    #[error("Cluster error: {0}")]
    Cluster(String),

    /// TLS errors
    #[error("TLS error: {0}")]
    Tls(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Health check errors
    #[error("Health check error: {0}")]
    HealthCheck(String),

    /// Redis errors
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Other errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Type alias for Redis results
pub type RedisResult<T> = Result<T, RedisError>;

impl From<RedisError> for CacheError {
    fn from(err: RedisError) -> Self {
        match err {
            RedisError::Connection(msg) => CacheError::ConnectionError(msg),
            RedisError::Operation(msg) => CacheError::OperationError(msg),
            RedisError::Command(msg) => CacheError::OperationError(msg),
            RedisError::Serialization(msg) => CacheError::SerializationError(msg),
            RedisError::Configuration(msg) => CacheError::ConfigurationError(msg),
            RedisError::Timeout(msg) => CacheError::TimeoutError(msg),
            RedisError::Cluster(msg) => CacheError::BackendError(format!("Cluster error: {}", msg)),
            RedisError::Tls(msg) => CacheError::BackendError(format!("TLS error: {}", msg)),
            RedisError::Authentication(msg) => {
                CacheError::ConnectionError(format!("Authentication error: {}", msg))
            }
            RedisError::HealthCheck(msg) => {
                CacheError::ConnectionError(format!("Health check error: {}", msg))
            }
            RedisError::Redis(err) => translate_redis_error(err),
            RedisError::Unknown(msg) => CacheError::UnknownError(msg),
        }
    }
}

/// Translate Redis errors to Cache errors
pub fn translate_redis_error(err: redis::RedisError) -> CacheError {
    match err.kind() {
        redis::ErrorKind::IoError => CacheError::ConnectionError(format!("I/O error: {}", err)),
        redis::ErrorKind::ClientError => CacheError::ClientError(err.to_string()),
        redis::ErrorKind::ExtensionError => {
            CacheError::BackendError(format!("Extension error: {}", err))
        }
        redis::ErrorKind::AuthenticationFailed => {
            CacheError::ConnectionError(format!("Authentication failed: {}", err))
        }
        redis::ErrorKind::BusyLoadingError => {
            CacheError::OperationError(format!("Busy loading: {}", err))
        }
        redis::ErrorKind::InvalidClientConfig => {
            CacheError::ConfigurationError(format!("Invalid client config: {}", err))
        }
        redis::ErrorKind::Moved => CacheError::BackendError(format!("Moved error: {}", err)),
        redis::ErrorKind::Ask => CacheError::BackendError(format!("Ask error: {}", err)),
        redis::ErrorKind::TryAgain => CacheError::OperationError(format!("Try again: {}", err)),
        redis::ErrorKind::ClusterDown => {
            CacheError::ConnectionError(format!("Cluster down: {}", err))
        }
        redis::ErrorKind::CrossSlot => CacheError::BackendError(format!("Cross slot: {}", err)),
        redis::ErrorKind::MasterDown => {
            CacheError::ConnectionError(format!("Master down: {}", err))
        }
        redis::ErrorKind::ReadOnly => CacheError::OperationError(format!("Read only: {}", err)),
        redis::ErrorKind::NeedFullCoverage => {
            CacheError::BackendError(format!("Need full coverage: {}", err))
        }
        redis::ErrorKind::ResponseError => {
            CacheError::OperationError(format!("Response error: {}", err))
        }
        redis::ErrorKind::NoScriptError => {
            CacheError::OperationError(format!("No script: {}", err))
        }
        redis::ErrorKind::ExecAbortError => {
            CacheError::OperationError(format!("Exec abort: {}", err))
        }
        redis::ErrorKind::InvalidClientName => {
            CacheError::ConfigurationError(format!("Invalid client name: {}", err))
        }
        redis::ErrorKind::NoMem => CacheError::BackendError(format!("No memory: {}", err)),
        redis::ErrorKind::NoAuth => CacheError::ConnectionError(format!("No auth: {}", err)),
        redis::ErrorKind::NotBusy => CacheError::OperationError(format!("Not busy: {}", err)),
        redis::ErrorKind::OutOfRange => {
            CacheError::OperationError(format!("Out of range: {}", err))
        }
        redis::ErrorKind::NotFound => CacheError::NotFoundError(err.to_string()),
        redis::ErrorKind::TypeError => {
            CacheError::SerializationError(format!("Type error: {}", err))
        }
        redis::ErrorKind::Timeout => CacheError::TimeoutError(err.to_string()),
        _ => CacheError::UnknownError(err.to_string()),
    }
}

/// Convert a cache result to a Redis result
pub fn to_redis_result<T>(result: CacheResult<T>) -> RedisResult<T> {
    result.map_err(|err| match err {
        CacheError::ConnectionError(msg) => RedisError::Connection(msg),
        CacheError::OperationError(msg) => RedisError::Operation(msg),
        CacheError::SerializationError(msg) => RedisError::Serialization(msg),
        CacheError::ConfigurationError(msg) => RedisError::Configuration(msg),
        CacheError::TimeoutError(msg) => RedisError::Timeout(msg),
        CacheError::NotFoundError(msg) => RedisError::Operation(format!("Not found: {}", msg)),
        CacheError::ClientError(msg) => RedisError::Operation(format!("Client error: {}", msg)),
        CacheError::BackendError(msg) => RedisError::Operation(format!("Backend error: {}", msg)),
        CacheError::UnknownError(msg) => RedisError::Unknown(msg),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_error_to_cache_error() {
        // Test Connection error conversion
        let redis_err = RedisError::Connection("Failed to connect".to_string());
        let cache_err = CacheError::from(redis_err);
        assert!(matches!(cache_err, CacheError::ConnectionError(_)));

        // Test Operation error conversion
        let redis_err = RedisError::Operation("Failed operation".to_string());
        let cache_err = CacheError::from(redis_err);
        assert!(matches!(cache_err, CacheError::OperationError(_)));

        // Test Serialization error conversion
        let redis_err = RedisError::Serialization("Failed to serialize".to_string());
        let cache_err = CacheError::from(redis_err);
        assert!(matches!(cache_err, CacheError::SerializationError(_)));

        // Test Configuration error conversion
        let redis_err = RedisError::Configuration("Invalid config".to_string());
        let cache_err = CacheError::from(redis_err);
        assert!(matches!(cache_err, CacheError::ConfigurationError(_)));

        // Test Timeout error conversion
        let redis_err = RedisError::Timeout("Operation timed out".to_string());
        let cache_err = CacheError::from(redis_err);
        assert!(matches!(cache_err, CacheError::TimeoutError(_)));

        // Test HealthCheck error conversion
        let redis_err = RedisError::HealthCheck("Health check failed".to_string());
        let cache_err = CacheError::from(redis_err);
        assert!(matches!(cache_err, CacheError::ConnectionError(_)));
    }
}
