//! # Core module
//!
//! This module provides the core functionality for the application.
//! It includes common utilities, abstractions, and the foundation for the application.

pub mod api;
pub mod auth;
pub mod cache;
pub mod config;
pub mod core_logger;
pub mod core_middleware;
pub mod error;
pub mod handlers;
pub mod macros;
pub mod metrics;
pub mod models;
pub mod reliability;
pub mod router;
pub mod services;
pub mod utils;

// Re-export key components for easier access
#[cfg(feature = "auth")]
pub use auth::{EntraAuthLayer, EntraTokenClient};
pub use cache::{CacheRegistry, ResourceCache, get_resource_cache, init_cache_registry};
pub use config::app_config::{AppConfig, load_config};
pub use error::{AppError, Result};
pub use macros::core_macros;
pub use metrics::{init_metrics, metrics_endpoint_handler, try_record_metrics};
pub use reliability::apply_reliability;
pub use router::CoreRouter;
pub use utils::api_resource::{
    ApiHandlerOptions, ApiResource, ApiResourceRegistry, create_api_handler,
};

// Export specific items from modules to avoid name conflicts
pub use core_logger as logger;
pub use core_middleware as middleware;
pub use handlers::core_health as handlers_health;
pub use services::health as services_health;
pub use utils::log_request as utils_log_request;

#[cfg(feature = "auth")]
mod handler_utils {
    use crate::core::core_middleware::auth::{AuthError, TokenError};
    use crate::core::error::AppError;

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
