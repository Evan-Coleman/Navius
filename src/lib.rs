//! # Rust Backend
//!
//! A modular Rust backend application with the following features:
//! - RESTful API endpoints using Axum
//! - OpenAPI documentation with Swagger UI
//! - Caching with Moka
//! - Metrics collection and reporting
//! - Structured error handling
//! - Configuration management

/// Application router and state management
pub mod app {
    pub use crate::app::router::*;
    pub mod router;
}

/// Caching functionality
pub mod cache {
    pub use crate::cache::cache_manager::*;
    pub mod cache_manager;
}

/// Configuration management
pub mod config {
    pub use crate::config::app_config::*;
    pub mod app_config;
    pub mod constants;
}

/// Error handling
pub mod error {
    pub use crate::error::error_types::*;
    pub use crate::error::logger::*;
    pub use crate::error::middleware::*;
    pub use crate::error::result_ext::*;
    pub mod error_types;
    pub mod logger;
    pub mod middleware;
    pub mod result_ext;
}

/// Metrics collection and reporting
pub mod metrics {
    pub use crate::metrics::metrics_service::*;
    pub mod metrics_service;
}

/// API request handlers
pub mod handlers;

/// Data models and schemas
pub mod models;

/// Authentication and authorization
pub mod auth;

/// Generated API clients
#[path = "generated_apis.rs"]
pub mod generated_apis;

/// Reliability features for improved resilience
pub mod reliability;

/// Utility functions and helpers
pub mod utils;
