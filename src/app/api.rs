//! # API Module
//!
//! This module contains user-facing API endpoints and handlers.

use axum::Router;
use std::sync::Arc;

use crate::core::router::core_app_router::AppState;

/// Configure all API routes
pub fn configure(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    // Return the router with no additional routes
    router
}

/// Register all API services
pub fn register_services(
    builder: crate::core::router::core_app_router::RouterBuilder,
) -> crate::core::router::core_app_router::RouterBuilder {
    // Return the builder with no additional services
    builder
}
