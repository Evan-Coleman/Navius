//! # Core module
//!
//! This module provides the core functionality for the application.
//! It includes common utilities, abstractions, and the foundation for the application.

pub mod api;
pub mod auth;
pub mod cache;
pub mod config;
pub mod error;
pub mod handlers;
pub mod logger;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod reliability;
pub mod router;
pub mod services;
pub mod utils;

// Re-export key components for easier access
pub use auth::{EntraAuthLayer, EntraTokenClient};
pub use cache::{CacheRegistry, ResourceCache, get_resource_cache, init_cache_registry};
pub use config::app_config::{AppConfig, load_config};
pub use error::{AppError, Result};
pub use metrics::{init_metrics, metrics_endpoint_handler, try_record_metrics};
pub use reliability::apply_reliability;
pub use router::CoreRouter;
pub use utils::api_resource::{
    ApiHandlerOptions, ApiResource, ApiResourceRegistry, create_api_handler,
};

pub use auth::*;
pub use cache::*;
pub use config::*;
pub use handlers::*;
pub use metrics::*;
pub use models::*;
pub use router::*;
pub use services::*;
pub use utils::*;

mod handler_utils {
    use crate::core::error::AppError;
    use crate::core::middleware::auth::{AuthError, TokenError};

    pub fn map_auth_error(err: AuthError) -> AppError {
        match err {
            AuthError::InvalidToken => AppError::unauthorized("Invalid token"),
            AuthError::MissingToken => AppError::unauthorized("Missing token"),
            AuthError::TokenExpired => AppError::unauthorized("Token expired"),
            AuthError::UnauthorizedRole => AppError::forbidden("Unauthorized role"),
            AuthError::Other(msg) => {
                AppError::internal_server_error(format!("Authentication error: {}", msg))
            }
        }
    }

    pub fn map_token_error(err: TokenError) -> AppError {
        match err {
            TokenError::RequestError(msg) => AppError::internal_server_error(msg),
            TokenError::ResponseError(msg) => AppError::internal_server_error(msg),
            TokenError::UnexpectedResponse(msg) => AppError::internal_server_error(msg),
        }
    }
}
