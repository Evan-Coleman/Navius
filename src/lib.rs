//! # Rust Backend
//!
//! A modular Rust backend application with the following features:
//! - RESTful API endpoints using Axum
//! - OpenAPI documentation with Swagger UI
//! - Caching with Moka
//! - Metrics collection and reporting
//! - Structured error handling
//! - Configuration management

/// Core framework functionality not intended for modification by users
pub mod core;

/// Application router and state management
pub mod app {
    pub use crate::app::router::*;
    pub use crate::core::auth::*;
    pub mod router;
}

/// Caching functionality
pub mod cache {
    pub use crate::cache::providers::*;
    pub use crate::cache::registry_stats::*;
    pub mod providers;
    pub mod registry_stats;
}

/// Configuration management
pub mod config;

/// Error handling
pub mod error;

/// Metrics collection and reporting
pub mod metrics {
    pub use crate::metrics::metrics_service::*;
    pub mod metrics_service;
}

/// API request handlers
pub mod handlers;

/// Data models and schemas
pub mod models;

/// Generated API clients
#[path = "generated_apis.rs"]
pub mod generated_apis;

/// Reliability features for improved resilience
pub mod reliability;

/// Utility functions and helpers
pub mod utils;
