use navius_auth::error::AuthError;
use std::fmt;
use thiserror::Error;

/// Errors specific to the Microsoft Entra authentication provider
#[derive(Error, Debug)]
pub enum EntraError {
    /// Error occurred during configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error occurred during token validation
    #[error("Token validation error: {0}")]
    TokenValidation(String),

    /// Error with JWT processing
    #[error("JWT error: {0}")]
    Jwt(String),

    /// Error retrieving JWKS keys
    #[error("JWKS error: {0}")]
    Jwks(String),

    /// Error making HTTP requests to Microsoft Entra endpoints
    #[error("HTTP error: {0}")]
    Http(String),

    /// Error parsing or serializing data
    #[error("Data error: {0}")]
    Data(String),

    /// Unauthorized error (e.g., invalid credentials)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Error from the Microsoft Entra API
    #[error("Microsoft Entra API error: code={code}, message='{message}'")]
    Api {
        /// Error code from Microsoft Entra
        code: String,
        /// Error message from Microsoft Entra
        message: String,
    },

    /// Error during the authentication flow
    #[error("Authentication flow error: {0}")]
    AuthFlow(String),

    /// Error retrieving or parsing user profile
    #[error("User profile error: {0}")]
    UserProfile(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl EntraError {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        EntraError::Config(message.into())
    }

    /// Create a new token validation error
    pub fn token_validation<S: Into<String>>(message: S) -> Self {
        EntraError::TokenValidation(message.into())
    }

    /// Create a new JWT error
    pub fn jwt<S: Into<String>>(message: S) -> Self {
        EntraError::Jwt(message.into())
    }

    /// Create a new JWKS error
    pub fn jwks<S: Into<String>>(message: S) -> Self {
        EntraError::Jwks(message.into())
    }

    /// Create a new HTTP error
    pub fn http<S: Into<String>>(message: S) -> Self {
        EntraError::Http(message.into())
    }

    /// Create a new data error
    pub fn data<S: Into<String>>(message: S) -> Self {
        EntraError::Data(message.into())
    }

    /// Create a new unauthorized error
    pub fn unauthorized<S: Into<String>>(message: S) -> Self {
        EntraError::Unauthorized(message.into())
    }

    /// Create a new API error
    pub fn api<S1: Into<String>, S2: Into<String>>(code: S1, message: S2) -> Self {
        EntraError::Api {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create a new authentication flow error
    pub fn auth_flow<S: Into<String>>(message: S) -> Self {
        EntraError::AuthFlow(message.into())
    }

    /// Create a new user profile error
    pub fn user_profile<S: Into<String>>(message: S) -> Self {
        EntraError::UserProfile(message.into())
    }

    /// Create a new timeout error
    pub fn timeout<S: Into<String>>(message: S) -> Self {
        EntraError::Timeout(message.into())
    }

    /// Create a new internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        EntraError::Internal(message.into())
    }
}

impl From<reqwest::Error> for EntraError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            EntraError::Timeout(error.to_string())
        } else if error.is_status() {
            let status = error.status().unwrap_or_default();
            if status.is_client_error() {
                EntraError::Unauthorized(format!("HTTP error {}: {}", status.as_u16(), error))
            } else {
                EntraError::Http(format!("HTTP error {}: {}", status.as_u16(), error))
            }
        } else {
            EntraError::Http(error.to_string())
        }
    }
}

impl From<serde_json::Error> for EntraError {
    fn from(error: serde_json::Error) -> Self {
        EntraError::Data(format!("JSON error: {}", error))
    }
}

impl From<jsonwebtoken::errors::Error> for EntraError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        EntraError::Jwt(format!("JWT error: {}", error))
    }
}

impl From<url::ParseError> for EntraError {
    fn from(error: url::ParseError) -> Self {
        EntraError::Config(format!("URL parse error: {}", error))
    }
}

impl From<std::io::Error> for EntraError {
    fn from(error: std::io::Error) -> Self {
        EntraError::Internal(format!("IO error: {}", error))
    }
}

impl From<EntraError> for AuthError {
    fn from(error: EntraError) -> Self {
        match error {
            EntraError::Config(msg) => {
                AuthError::Configuration(format!("Entra config error: {}", msg))
            }
            EntraError::TokenValidation(msg) => {
                AuthError::TokenValidation(format!("Entra token error: {}", msg))
            }
            EntraError::Jwt(msg) => AuthError::TokenValidation(format!("Entra JWT error: {}", msg)),
            EntraError::Jwks(msg) => {
                AuthError::TokenValidation(format!("Entra JWKS error: {}", msg))
            }
            EntraError::Http(msg) => {
                AuthError::ExternalService(format!("Entra HTTP error: {}", msg))
            }
            EntraError::Data(msg) => AuthError::Data(format!("Entra data error: {}", msg)),
            EntraError::Unauthorized(msg) => {
                AuthError::Unauthorized(format!("Entra unauthorized: {}", msg))
            }
            EntraError::Api { code, message } => {
                AuthError::ExternalService(format!("Entra API error {}: {}", code, message))
            }
            EntraError::AuthFlow(msg) => AuthError::Flow(format!("Entra auth flow error: {}", msg)),
            EntraError::UserProfile(msg) => {
                AuthError::UserProfile(format!("Entra user profile error: {}", msg))
            }
            EntraError::Timeout(msg) => AuthError::Timeout(format!("Entra timeout: {}", msg)),
            EntraError::Internal(msg) => {
                AuthError::Internal(format!("Entra internal error: {}", msg))
            }
        }
    }
}

/// Result type for Entra authentication operations
pub type EntraResult<T> = Result<T, EntraError>;
