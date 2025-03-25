use axum::{extract::State, http::StatusCode, response::Json};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc, time::SystemTime};

use crate::core::{
    models::{DependencyStatus, DetailedHealthResponse, HealthCheckResponse},
    router::AppState,
};

/// Ultra-minimal health check endpoint for simple monitoring
/// Returns a simple JSON status object like Spring Boot's /health endpoint
pub async fn simple_health_handler() -> Json<Value> {
    Json(json!({ "status": "UP" }))
}

/// Simple health check endpoint
pub async fn health_handler() -> Json<HealthCheckResponse> {
    let version = env!("CARGO_PKG_VERSION").to_string();

    // Simple uptime is just the current time as a string
    Json(HealthCheckResponse {
        status: "UP".to_string(),
        version,
        uptime: "Active".to_string(),
    })
}

/// Detailed health check
pub async fn detailed_health_handler(
    State(state): State<Arc<AppState>>,
) -> Json<DetailedHealthResponse> {
    let version = env!("CARGO_PKG_VERSION").to_string();

    // Calculate uptime directly instead of using DateTime comparison
    let now = SystemTime::now();
    let uptime_secs = now
        .duration_since(state.start_time)
        .unwrap_or_default()
        .as_secs();
    let uptime = format!("{}s", uptime_secs);

    // Check dependencies
    let mut dependencies = Vec::new();

    // Check cache if available
    dependencies.push(DependencyStatus {
        name: "Cache".to_string(),
        status: "UP".to_string(),
        details: Some(format!(
            "Cache {}",
            match &state.cache_registry {
                Some(registry) => format!("enabled"),
                None => format!("disabled"),
            }
        )),
    });

    // Determine overall status - if any dependency is down, the whole service is down
    let status = if dependencies
        .iter()
        .any(|d| d.status != "UP" && d.status != "DISABLED")
    {
        "DOWN".to_string()
    } else {
        "UP".to_string()
    };

    Json(DetailedHealthResponse {
        status,
        version,
        uptime,
        dependencies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.0.status, "UP");
    }

    #[tokio::test]
    async fn test_detailed_health_handler() {
        let state = AppState::default();
        let response = detailed_health_handler(State(Arc::new(state))).await;

        // Basic service should be up even with no dependencies
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.0.status, "UP");

        assert!(!response.0.dependencies.is_empty());
    }
}
