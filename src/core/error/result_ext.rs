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
