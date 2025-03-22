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
    pub use crate::core::metrics::*;
}

/// API endpoints and handlers
pub mod api;

/// API request handlers
pub mod handlers;

/// Data models and schemas
pub mod models;

/// Repository module for data access
pub mod repository;

/// Service module for business logic
pub mod services;

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

/// MockExtern trait implementation for test mocking
#[cfg(test)]
pub mod mockable {
    /// Marker trait for types that can be mocked externally
    pub trait MockExtern {}
}
