use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc, time::SystemTime};

use crate::core::{
    models::{DependencyStatus, DetailedHealthResponse, HealthCheckResponse},
    router::AppState,
};

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

/// Detailed health check that includes database status
pub async fn detailed_health_handler(
    State(state): State<Arc<AppState>>,
) -> Json<DetailedHealthResponse> {
    let version = env!("CARGO_PKG_VERSION").to_string();

    // Calculate uptime by comparing current time to start time
    let now = Utc::now();
    let uptime_duration = now.signed_duration_since(state.start_time);
    let uptime_secs = uptime_duration.num_seconds() as u64;
    let uptime = format!("{}s", uptime_secs);

    // Check dependencies
    let mut dependencies = Vec::new();

    // Database access removed for stability
    // Always report database as disabled since it has been removed
    dependencies.push(DependencyStatus {
        name: "Database".to_string(),
        status: "DISABLED".to_string(),
        details: Some("Database functionality has been removed for stability".to_string()),
    });

    // Check cache if available
    dependencies.push(DependencyStatus {
        name: "Cache".to_string(),
        status: "UP".to_string(),
        details: Some(format!(
            "Cache enabled with {} entries",
            state.cache_registry.count_entries()
        )),
    });

    // Determine overall status - if any dependency is down, the whole service is down
    // Database being disabled doesn't count as DOWN since it's intentional
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

// Database connection check removed for stability as we no longer use a database
// The function has been simplified to always return a disabled status

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

        // Should have at least the database disabled dependency
        assert!(!response.0.dependencies.is_empty());
        assert!(
            response
                .0
                .dependencies
                .iter()
                .any(|d| d.name == "Database" && d.status == "DISABLED")
        );
    }
}
