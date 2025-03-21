use crate::error::error_types::{AppError, Result};
use axum::http::StatusCode;
use std::fmt::Display;
use tracing::{error, warn};

/// Extension trait for Result to make error handling more ergonomic
pub trait ResultExt<T, E> {
    /// Convert any error to an AppError::InternalError
    fn internal_err(self) -> Result<T>;

    /// Convert any error to an AppError::BadRequest
    fn bad_request(self) -> Result<T>;

    /// Convert any error to an AppError::NotFound
    fn not_found(self) -> Result<T>;

    /// Convert any error to an AppError::ExternalServiceError
    fn external_err(self) -> Result<T>;

    /// Convert any error to an AppError::DatabaseError
    fn db_err(self) -> Result<T>;

    /// Convert any error to an AppError::CacheError
    fn cache_err(self) -> Result<T>;

    /// Log the error and then return it
    fn log_err(self) -> Self;

    /// Map error context and convert to AppError
    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: std::fmt::Display + std::fmt::Debug,
{
    fn internal_err(self) -> Result<T> {
        self.map_err(|e| {
            error!("Internal error: {}", e);
            AppError::InternalError(e.to_string())
        })
    }

    fn bad_request(self) -> Result<T> {
        self.map_err(|e| {
            warn!("Bad request: {}", e);
            AppError::BadRequest(e.to_string())
        })
    }

    fn not_found(self) -> Result<T> {
        self.map_err(|e| {
            warn!("Not found: {}", e);
            AppError::NotFound(e.to_string())
        })
    }

    fn external_err(self) -> Result<T> {
        self.map_err(|e| {
            error!("External service error: {}", e);
            AppError::ExternalServiceError(e.to_string())
        })
    }

    fn db_err(self) -> Result<T> {
        self.map_err(|e| {
            error!("Database error: {}", e);
            AppError::DatabaseError(e.to_string())
        })
    }

    fn cache_err(self) -> Result<T> {
        self.map_err(|e| {
            warn!("Cache error: {}", e);
            AppError::CacheError(e.to_string())
        })
    }

    fn log_err(self) -> Self {
        if let Err(ref e) = self {
            error!("Error occurred: {}", e);
        }
        self
    }

    fn context<C>(self, context: C) -> Result<T>
    where
        C: Display,
    {
        self.map_err(|e| {
            let msg = format!("{}: {}", context, e);
            error!("{}", msg);
            AppError::InternalError(msg)
        })
    }
}

/// Extension traits for standardized HTTP response conversion
pub trait StatusCodeExt {
    /// Convert to AppError with given message
    fn with_error<M: Display>(self, message: M) -> AppError;
}

impl StatusCodeExt for StatusCode {
    fn with_error<M: Display>(self, message: M) -> AppError {
        let msg = message.to_string();
        match self {
            StatusCode::NOT_FOUND => AppError::NotFound(msg),
            StatusCode::BAD_REQUEST => AppError::BadRequest(msg),
            StatusCode::UNAUTHORIZED => AppError::Unauthorized(msg),
            StatusCode::FORBIDDEN => AppError::Forbidden(msg),
            StatusCode::TOO_MANY_REQUESTS => AppError::RateLimited(msg),
            StatusCode::UNPROCESSABLE_ENTITY => AppError::ValidationError(msg),
            StatusCode::BAD_GATEWAY => AppError::ExternalServiceError(msg),
            _ if self.is_server_error() => AppError::InternalError(msg),
            _ => AppError::InternalError(format!(
                "Unexpected status code {}: {}",
                self.as_u16(),
                msg
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Error as IoError;
    use std::io::ErrorKind;

    // Helper function to create a test error
    fn test_error() -> IoError {
        IoError::new(ErrorKind::Other, "test error")
    }

    #[test]
    fn test_internal_err_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.internal_err();

        match app_result {
            Err(AppError::InternalError(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected InternalError variant"),
        }
    }

    #[test]
    fn test_bad_request_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.bad_request();

        match app_result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected BadRequest variant"),
        }
    }

    #[test]
    fn test_not_found_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.not_found();

        match app_result {
            Err(AppError::NotFound(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected NotFound variant"),
        }
    }

    #[test]
    fn test_external_err_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.external_err();

        match app_result {
            Err(AppError::ExternalServiceError(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected ExternalServiceError variant"),
        }
    }

    #[test]
    fn test_db_err_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.db_err();

        match app_result {
            Err(AppError::DatabaseError(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected DatabaseError variant"),
        }
    }

    #[test]
    fn test_cache_err_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.cache_err();

        match app_result {
            Err(AppError::CacheError(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected CacheError variant"),
        }
    }

    #[test]
    fn test_context_conversion() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.context("Context information");

        match app_result {
            Err(AppError::InternalError(msg)) => {
                assert!(msg.contains("Context information"));
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected InternalError variant with context"),
        }
    }

    #[test]
    fn test_ok_result_remains_ok() {
        let result: std::result::Result<&str, IoError> = Ok("success");
        let app_result = result.internal_err();

        assert!(app_result.is_ok());
        assert_eq!(app_result.unwrap(), "success");
    }

    #[test]
    fn test_status_code_to_error() {
        let not_found = StatusCode::NOT_FOUND.with_error("Resource not found");
        match not_found {
            AppError::NotFound(msg) => assert_eq!(msg, "Resource not found"),
            _ => panic!("Expected NotFound variant"),
        }

        let bad_request = StatusCode::BAD_REQUEST.with_error("Invalid input");
        match bad_request {
            AppError::BadRequest(msg) => assert_eq!(msg, "Invalid input"),
            _ => panic!("Expected BadRequest variant"),
        }

        let unauthorized = StatusCode::UNAUTHORIZED.with_error("Not authenticated");
        match unauthorized {
            AppError::Unauthorized(msg) => assert_eq!(msg, "Not authenticated"),
            _ => panic!("Expected Unauthorized variant"),
        }

        let forbidden = StatusCode::FORBIDDEN.with_error("Not authorized");
        match forbidden {
            AppError::Forbidden(msg) => assert_eq!(msg, "Not authorized"),
            _ => panic!("Expected Forbidden variant"),
        }

        let server_error = StatusCode::INTERNAL_SERVER_ERROR.with_error("Server failed");
        match server_error {
            AppError::InternalError(msg) => assert_eq!(msg, "Server failed"),
            _ => panic!("Expected InternalError variant"),
        }
    }
}
