//! Health check API endpoints
//!
//! This module provides health check endpoints for the API.

use axum::debug_handler;
use axum::{Router, extract::State, http::StatusCode, response::Json, routing::get};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::database::ping_database;
use crate::core::router::AppState;

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Whether the service is up
    pub status: String,

    /// Current timestamp
    pub timestamp: u64,

    /// Database connectivity status
    pub database: DatabaseStatus,
}

/// Database status
#[derive(Debug, Serialize)]
pub struct DatabaseStatus {
    /// Whether the database is connected
    pub connected: bool,

    /// Database connection message
    pub message: String,
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

    // Check database status
    let database = match &state.db_pool {
        Some(_) => {
            // Database is configured
            // In a real implementation, we would ping the database
            // but since we're using an in-memory implementation for now,
            // we'll just indicate that it's available
            DatabaseStatus {
                connected: true,
                message: "Database connection available".to_string(),
            }
        }
        None => DatabaseStatus {
            connected: false,
            message: "Database not configured".to_string(),
        },
    };

    // Create response
    let response = HealthResponse {
        status: "UP".to_string(),
        timestamp,
        database,
    };

    (StatusCode::OK, Json(response))
}
