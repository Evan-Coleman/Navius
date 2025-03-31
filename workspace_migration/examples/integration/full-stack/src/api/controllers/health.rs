use axum::Json;
use axum::http::StatusCode;
use navius_core::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub components: HashMap<String, ComponentStatus>,
}

/// Component status
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: String,
    pub details: Option<String>,
}

/// General health check
pub async fn health_check() -> Result<Json<HealthResponse>> {
    let mut response = HealthResponse {
        status: "UP".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        components: HashMap::new(),
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
        components,
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
