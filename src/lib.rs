// Copyright (c) 2025 Navius Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Navius
//!
//! A modular Rust application with the following features:
//! - RESTful API endpoints using Axum
//! - OpenAPI documentation with Swagger UI
//! - Caching with Moka
//! - Metrics collection and reporting
//! - Structured error handling
//! - Configuration management

// ===============================================================================
// Core Framework Modules - Not intended for modification by users
// ===============================================================================

/// Core framework functionality not intended for modification by users
pub mod core {
    // Core API module
    pub mod api;

    // Authentication and authorization
    pub mod auth;

    // Caching functionality
    pub mod cache;

    // Configuration management
    pub mod config;

    // Core logger implementation
    pub mod core_logger;

    // Core middleware implementation
    pub mod core_middleware;

    // Error handling
    pub mod error;

    // API request handlers
    pub mod handlers {
        // Health check handlers
        pub mod core_health;

        // Debug and management actuator endpoints
        pub mod core_actuator;

        // API documentation handlers
        pub mod core_docs;

        // Logging middleware
        pub mod core_logging;

        pub use core_actuator::*;
        pub use core_docs::*;
        pub use core_health::*;
        pub use core_logging::*;
    }

    // Metrics collection and reporting
    pub mod metrics;

    // Data models and schemas
    pub mod models {
        // Error models
        pub mod core_error;

        // User-extensible models
        pub mod core_extensions;

        // Response models
        pub mod core_response;

        pub use core_error::*;
        pub use core_extensions::*;
        pub use core_response::*;
    }

    // Reliability features
    pub mod reliability;

    // Routing functionality
    pub mod router {
        // Core router implementation
        pub mod core_router;

        // Application router
        pub mod core_app_router;

        pub use core_app_router::*;
        pub use core_router::*;
    }

    // Service implementations
    pub mod services;

    // Utility functions
    pub mod utils;

    // Re-export key components for easier access
    pub use self::auth::middleware::EntraAuthLayer;
    pub use self::cache::ResourceCache;
    pub use self::cache::cache_manager::{CacheRegistry, get_resource_cache, init_cache_registry};
    pub use self::config::app_config::{AppConfig, load_config};
    pub use self::error::{AppError, Result};
    pub use self::metrics::{init_metrics, metrics_endpoint_handler, try_record_metrics};
    pub use self::reliability::apply_reliability;
    pub use self::router::CoreRouter;
    pub use self::utils::api_resource::{
        ApiHandlerOptions, ApiResource, ApiResourceRegistry, create_api_handler,
    };
    pub use crate::core::auth::TokenClient;

    // Export specific items from modules to avoid name conflicts
    pub use self::core_logger as logger;
    pub use self::core_middleware as middleware;
    pub use self::handlers::core_health as handlers_health;
    pub use self::services::health as services_health;
}

// ===============================================================================
// Application Modules - Can be extended and modified by users
// ===============================================================================

/// Application components that can be extended by users
pub mod app {
    // API endpoints
    pub mod api;

    // Service implementations
    pub mod services;
}

// ===============================================================================
// Convenience Re-exports - For easier access to common components
// ===============================================================================

/// Caching functionality
pub mod cache {
    pub use crate::core::cache::*;
}

/// Configuration management
pub mod config {
    pub use crate::core::config::*;
}

/// Error handling
pub mod error {
    pub use crate::core::error::*;
}

/// Metrics collection and reporting
pub mod metrics {
    pub use crate::core::metrics::*;
}

/// API endpoints and handlers
pub mod api {
    pub use crate::core::api::*;
}

/// API request handlers
pub mod handlers {
    pub use crate::app::api::*;
    pub use crate::core::handlers::*;
}

/// Data models and schemas
pub mod models {
    pub use crate::core::models::*;
}

/// Service module for business logic
pub mod services {
    pub use crate::app::services::*;
    pub use crate::core::services::*;
}

/// Reliability features for improved resilience
pub mod reliability {
    pub use crate::core::reliability::*;
}

/// Utility functions and helpers
pub mod utils {
    pub use crate::core::utils::*;
}

/// Authentication and authorization
pub mod auth {
    pub use crate::core::auth::*;
}

/// Routing functionality
pub mod router {
    pub use crate::core::router::*;
}

/// MockExtern trait implementation for test mocking
#[cfg(test)]
pub mod mockable {
    /// Marker trait for types that can be mocked externally
    pub trait MockExtern {}
}

// Direct re-exports for commonly used types
pub use crate::core::error::*;
pub use crate::core::models::*;
pub use crate::core::services::error as core_service_error;
pub use crate::core::services::health::HealthService as CoreHealthService;
