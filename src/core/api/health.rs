//! Health check API endpoints
//!
//! This module provides health check endpoints for the API.

use axum::debug_handler;
use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::router::AppState;

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Whether the service is up
    pub status: String,

    /// Current timestamp
    pub timestamp: u64,
}

/// Configure health check routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_check))
}

/// Health check handler
#[debug_handler]
async fn health_check(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthResponse>) {
    // Get current timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Create response
    let response = HealthResponse {
        status: "UP".to_string(),
        timestamp,
    };

    (StatusCode::OK, Json(response))
}
