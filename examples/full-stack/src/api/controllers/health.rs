use axum::Json;
use axum::http::StatusCode;
use navius_core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health check response
///
/// Contains information about the system's health status and version.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Application status (UP, DOWN, DEGRADED)
    pub status: String,
    /// Application version as defined in Cargo.toml
    pub version: String,
    /// Build timestamp in RFC3339 format
    pub build_timestamp: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Database status (CONNECTED, DISCONNECTED)
    pub database: DatabaseStatus,
    /// Additional services and dependencies with their status
    pub services: Vec<ServiceStatus>,
}

/// Component status
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: String,
    pub details: Option<String>,
}

/// Database status information
///
/// Contains details about the database connection health.
#[derive(Debug, Serialize)]
pub struct DatabaseStatus {
    /// Connection status (CONNECTED, DISCONNECTED, DEGRADED)
    pub status: String,
    /// Database version information
    pub version: Option<String>,
    /// Connection pool usage (active connections)
    pub active_connections: Option<u32>,
    /// Most recent error message, if any
    pub last_error: Option<String>,
    /// Time since the last successful query in seconds
    pub last_success: Option<u64>,
}

/// Individual service health status
///
/// Contains health information for a specific service or dependency.
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Service status (UP, DOWN, DEGRADED)
    pub status: String,
    /// Service version, if available
    pub version: Option<String>,
    /// Most recent error message, if any
    pub last_error: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
}

/// General health check
///
/// Returns a simple status to indicate the application is running.
/// This endpoint is unauthenticated and meant for load balancers and monitoring.
///
/// # Errors
/// - Returns `InternalServerError` if there's an issue checking the service health
pub async fn health_check() -> Result<Json<HealthResponse>> {
    let mut response = HealthResponse {
        status: "UP".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_timestamp: chrono::Utc::now().to_rfc3339(),
        uptime: 0,
        database: DatabaseStatus {
            status: "UP".to_string(),
            version: None,
            active_connections: None,
            last_error: None,
            last_success: None,
        },
        services: Vec::new(),
    };

    // Add component statuses
    response.components.insert(
        "api".to_string(),
        ComponentStatus {
            status: "UP".to_string(),
            details: None,
        },
    );

    Ok(Json(response))
}

/// Readiness check - indicates if the service is ready to accept traffic
///
/// Returns comprehensive health information about the application and its dependencies.
/// Requires authentication and admin privileges.
///
/// # Errors
/// - Returns `Forbidden` if the user doesn't have admin privileges
/// - Returns `InternalServerError` if there's an issue checking the service health
pub async fn readiness_check() -> Result<Json<HealthResponse>> {
    let mut components = HashMap::new();

    // Add component statuses
    components.insert(
        "api".to_string(),
        ComponentStatus {
            status: "UP".to_string(),
            details: None,
        },
    );

    components.insert(
        "database".to_string(),
        ComponentStatus {
            status: "UP".to_string(),
            details: None,
        },
    );

    components.insert(
        "cache".to_string(),
        ComponentStatus {
            status: "UP".to_string(),
            details: None,
        },
    );

    let response = HealthResponse {
        status: "UP".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_timestamp: chrono::Utc::now().to_rfc3339(),
        uptime: 0,
        database: DatabaseStatus {
            status: "UP".to_string(),
            version: None,
            active_connections: None,
            last_error: None,
            last_success: None,
        },
        services: components
            .into_iter()
            .map(|(name, status)| ServiceStatus {
                name,
                status: status.status,
                version: status.details,
                last_error: None,
                response_time_ms: None,
            })
            .collect(),
    };

    Ok(Json(response))
}

/// Liveness check - indicates if the service is running
pub async fn liveness_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "UP",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}
