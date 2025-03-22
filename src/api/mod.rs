//! API module
//!
//! This module contains the API routes and handlers.

mod users;

use axum::Router;

use crate::core::router::AppState;
use std::sync::Arc;

/// Configure all API routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        // User management endpoints
        .merge(users::configure())
}
