use crate::core::error::AppError;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Error type for service-related operations
#[derive(Debug)]
pub enum ServiceError {
    /// Error when initializing a service
    InitializationError(String),

    /// Error when a service is not found
    NotFound(String),

    /// Error when a dependency is missing
    MissingDependency(String),

    /// Error when a circular dependency is detected
    CircularDependency(String),

    /// Error when a service is not available or not responsive
    Unavailable(String),

    /// Error when a service operation times out
    Timeout(String),

    /// Error when configuration is invalid
    ConfigurationError(String),

    /// Conversion error
    ConversionError(String),

    /// Validation errors
    Validation(String),

    /// Conflict errors
    Conflict(String),

    /// Repository/database errors
    Repository(String),

    /// Generic error with a message
    Other(String),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::InitializationError(msg) => {
                write!(f, "Service initialization error: {}", msg)
            }
            ServiceError::NotFound(msg) => write!(f, "Service not found: {}", msg),
            ServiceError::MissingDependency(msg) => write!(f, "Missing dependency: {}", msg),
            ServiceError::CircularDependency(msg) => {
                write!(f, "Circular dependency detected: {}", msg)
            }
            ServiceError::Unavailable(msg) => write!(f, "Service unavailable: {}", msg),
            ServiceError::Timeout(msg) => write!(f, "Service operation timed out: {}", msg),
            ServiceError::ConfigurationError(msg) => {
                write!(f, "Service configuration error: {}", msg)
            }
            ServiceError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            ServiceError::Validation(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::Conflict(msg) => write!(f, "Conflict error: {}", msg),
            ServiceError::Repository(msg) => write!(f, "Repository error: {}", msg),
            ServiceError::Other(msg) => write!(f, "Service error: {}", msg),
        }
    }
}

impl Error for ServiceError {}

impl From<&str> for ServiceError {
    fn from(msg: &str) -> Self {
        ServiceError::Other(msg.to_string())
    }
}

impl From<String> for ServiceError {
    fn from(msg: String) -> Self {
        ServiceError::Other(msg)
    }
}

// Helper macro to implement From for various error types
macro_rules! impl_from_error {
    ($error_type:ty, $variant:ident) => {
        impl From<$error_type> for ServiceError {
            fn from(error: $error_type) -> Self {
                ServiceError::$variant(error.to_string())
            }
        }
    };
}

// Implement From for common error types
impl_from_error!(std::io::Error, Other);
impl_from_error!(serde_json::Error, ConversionError);

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_display() {
        let error = ServiceError::NotFound("test service".to_string());
        assert_eq!(format!("{}", error), "Service not found: test service");

        let error = ServiceError::InitializationError("failed to connect".to_string());
        assert_eq!(
            format!("{}", error),
            "Service initialization error: failed to connect"
        );
    }

    #[test]
    fn test_service_error_from_str() {
        let error: ServiceError = "test error".into();
        assert!(matches!(error, ServiceError::Other(_)));
        assert_eq!(format!("{}", error), "Service error: test error");
    }

    #[test]
    fn test_service_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: ServiceError = io_error.into();
        assert!(matches!(error, ServiceError::Other(_)));
        assert!(format!("{}", error).contains("file not found"));
    }
}
