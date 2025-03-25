//! # API Module
//!
//! This module contains user-facing API endpoints and handlers.

pub mod examples;

use axum::Router;
use std::sync::Arc;

use crate::core::router::core_app_router::AppState;

/// Configure all API routes
pub fn configure(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    // Add example routes
    let router = examples::configure_routes(router);

    // Return the configured router
    router
}

/// Register all API services
pub fn register_services(
    builder: crate::core::router::core_app_router::RouterBuilder,
) -> crate::core::router::core_app_router::RouterBuilder {
    // Register example services
    let builder = examples::register_services(builder);

    // Return the builder with registered services
    builder
}
