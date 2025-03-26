use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{Value, json};
use std::{sync::Arc, time::SystemTime};

use crate::core::{
    models::{DependencyStatus, DetailedHealthResponse, HealthCheckResponse},
    router::AppState,
    services::health::HealthService,
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

/// Detailed health check that follows Spring Boot Actuator format
/// Returns components with their statuses and details
pub async fn detailed_health_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    // Create health service with default indicators
    let health_service = HealthService::new();

    // Get health status from service
    match health_service.check_health(&state).await {
        Ok(health_status) => Json(health_status),
        Err(_) => Json(json!({
            "status": "DOWN",
            "components": {}
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::app_config::AppConfig;
    use axum::http::StatusCode;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        assert_eq!(response.status, "UP");
        assert!(!response.version.is_empty());
        assert!(!response.uptime.is_empty());
    }

    #[tokio::test]
    async fn test_detailed_health_handler() {
        let state = AppState::default();
        let response = detailed_health_handler(State(Arc::new(state))).await;

        // Get the response as a Value
        let health_status = response.0;

        // Basic service should be up even with no dependencies
        assert!(health_status.get("status").is_some());
        assert!(health_status.get("components").is_some());

        // Verify components exist
        let components = health_status
            .get("components")
            .unwrap()
            .as_object()
            .unwrap();
        assert!(components.contains_key("cache"));
        assert!(components.contains_key("diskSpace"));
        assert!(components.contains_key("env"));
        assert!(components.contains_key("services"));
    }

    #[tokio::test]
    async fn test_simple_health_handler() {
        let response = simple_health_handler().await;
        assert_eq!(response.0.get("status").unwrap(), "UP");
    }
}
