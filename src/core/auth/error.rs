#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Authentication required: Missing authorization token")]
    MissingToken,

    #[error("Authentication failed: Invalid token format")]
    InvalidTokenFormat,

    #[error("Token validation failed: {0}")]
    ValidationFailed(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    #[error("Circuit breaker open")]
    CircuitOpen,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AuthError::ValidationFailed(format!("JWT error: {}", e))
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::NetworkError(format!("HTTP client error: {}", e))
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(e: serde_json::Error) -> Self {
        AuthError::SerializationError(format!("JSON error: {}", e))
    }
}

impl From<watch::error::SendError<common::CircuitState>> for AuthError {
    fn from(_e: watch::error::SendError<common::CircuitState>) -> Self {
        AuthError::InternalError("Failed to update circuit breaker state".to_string())
    }
}

use crate::core::auth::providers::common;
use crate::core::{AppError, error::ErrorType};
use crate::reliability::circuit_breaker::CircuitState;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;
use tokio::sync::watch;
