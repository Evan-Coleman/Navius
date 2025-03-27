use crate::core::error::error_types::{AppError, Result};
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
            AppError::InternalServerError(e.to_string())
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
            AppError::InternalServerError(msg)
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
            _ if self.is_server_error() => AppError::InternalServerError(msg),
            _ => AppError::InternalServerError(format!(
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
            Err(AppError::InternalServerError(msg)) => {
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected InternalServerError variant"),
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
            Err(AppError::InternalServerError(msg)) => {
                assert!(msg.contains("Context information"));
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected InternalServerError variant with context"),
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
    fn test_log_err() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let logged_result = result.log_err();
        assert!(logged_result.is_err());

        // Test that logging doesn't affect Ok values
        let ok_result: std::result::Result<&str, IoError> = Ok("success");
        let logged_ok = ok_result.log_err();
        assert_eq!(logged_ok.unwrap(), "success");
    }

    #[test]
    fn test_context_with_empty_message() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.context("");

        match app_result {
            Err(AppError::InternalServerError(msg)) => {
                assert!(msg.contains("test error"));
                assert!(msg.starts_with(": test error"));
            }
            _ => panic!("Expected InternalServerError variant"),
        }
    }

    #[test]
    fn test_context_with_multiple_contexts() {
        let result: std::result::Result<(), IoError> = Err(test_error());
        let app_result = result.context("First context").context("Second context");

        match app_result {
            Err(AppError::InternalServerError(msg)) => {
                assert!(msg.contains("Second context"));
                assert!(msg.contains("First context"));
                assert!(msg.contains("test error"));
            }
            _ => panic!("Expected InternalServerError variant"),
        }
    }

    #[test]
    fn test_status_code_to_error() {
        // Test all status code conversions
        let test_cases = vec![
            (StatusCode::NOT_FOUND, AppError::NotFound("test".into())),
            (StatusCode::BAD_REQUEST, AppError::BadRequest("test".into())),
            (
                StatusCode::UNAUTHORIZED,
                AppError::Unauthorized("test".into()),
            ),
            (StatusCode::FORBIDDEN, AppError::Forbidden("test".into())),
            (
                StatusCode::TOO_MANY_REQUESTS,
                AppError::RateLimited("test".into()),
            ),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                AppError::ValidationError("test".into()),
            ),
            (
                StatusCode::BAD_GATEWAY,
                AppError::ExternalServiceError("test".into()),
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::InternalServerError("test".into()),
            ),
            // Test unknown status code
            (
                StatusCode::SWITCHING_PROTOCOLS,
                AppError::InternalServerError(format!(
                    "Unexpected status code {}: test",
                    StatusCode::SWITCHING_PROTOCOLS.as_u16()
                )),
            ),
        ];

        for (status, expected_error) in test_cases {
            let error = status.with_error("test");
            assert_eq!(error.to_string(), expected_error.to_string());
        }
    }

    #[test]
    fn test_error_message_formatting() {
        // Test with different error message types
        let string_msg = String::from("string error");
        let str_msg = "str error";
        let custom_msg = CustomError("custom error");

        let error1 = StatusCode::BAD_REQUEST.with_error(string_msg);
        let error2 = StatusCode::NOT_FOUND.with_error(str_msg);
        let error3 = StatusCode::INTERNAL_SERVER_ERROR.with_error(custom_msg);

        assert!(matches!(error1, AppError::BadRequest(msg) if msg == "string error"));
        assert!(matches!(error2, AppError::NotFound(msg) if msg == "str error"));
        assert!(matches!(error3, AppError::InternalServerError(msg) if msg == "custom error"));
    }

    // Helper struct for testing Display trait
    #[derive(Debug)]
    struct CustomError(&'static str);

    impl Display for CustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}
