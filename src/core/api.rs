//! API module
//!
//! This module contains the API routes and handlers.

use crate::core::router::AppState;
use axum::Router;
use std::sync::Arc;

/// Configure all API routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
}
