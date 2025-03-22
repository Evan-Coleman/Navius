//! API module
//!
//! This module contains the API routes and handlers.

mod health;
mod users;

use axum::Router;

use crate::core::router::AppState;

/// Configure all API routes
pub fn configure() -> Router<AppState> {
    Router::new()
        // Health check endpoints
        .merge(health::configure())
        // User management endpoints
        .merge(users::configure())
}
