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

    // Get uptime as a string
    let uptime_secs = state.start_time.elapsed().unwrap_or_default().as_secs();
    let uptime = format!("{}s", uptime_secs);

    // Check dependencies
    let mut dependencies = Vec::new();

    // Check database if available
    if let Some(pool) = &state.db_pool {
        dependencies.push(check_database_connection(pool).await);
    }

    // Check cache if available
    if let Some(cache_registry) = &state.cache_registry {
        dependencies.push(DependencyStatus {
            name: "Cache".to_string(),
            status: "UP".to_string(),
            details: Some(format!(
                "Cache enabled with {} entries",
                cache_registry.count_entries()
            )),
        });
    }

    // Determine overall status - if any dependency is down, the whole service is down
    let status = if dependencies.iter().any(|d| d.status != "UP") {
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

/// Check database connection and return status
async fn check_database_connection(pool: &sqlx::Pool<sqlx::Postgres>) -> DependencyStatus {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => {
            let mut details = BTreeMap::new();
            details.insert("status".to_string(), "Connected".to_string());

            DependencyStatus {
                name: "Database".to_string(),
                status: "UP".to_string(),
                details: Some(serde_json::to_string(&details).unwrap_or_default()),
            }
        }
        Err(e) => {
            let mut details = BTreeMap::new();
            details.insert("error".to_string(), e.to_string());

            DependencyStatus {
                name: "Database".to_string(),
                status: "DOWN".to_string(),
                details: Some(serde_json::to_string(&details).unwrap_or_default()),
            }
        }
    }
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

        // No dependencies configured in default state
        assert!(response.0.dependencies.is_empty());
    }
}
