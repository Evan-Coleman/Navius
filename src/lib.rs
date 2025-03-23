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

/// Core framework functionality not intended for modification by users
pub mod core;

/// Application components that can be extended by users
pub mod app;

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
}

/// Data models and schemas
pub mod models {
    pub use crate::core::models::extensions::*;
    pub use crate::core::models::*;
}

/// Repository module for data access
pub mod repository {
    pub use crate::core::repository::*;
}

/// Service module for business logic
pub mod services {
    pub use crate::app::services::*;
    pub use crate::core::services::*;
}

/// Generated API clients
#[path = "generated_apis.rs"]
pub mod generated_apis;

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
