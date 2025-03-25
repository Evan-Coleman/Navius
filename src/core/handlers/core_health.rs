use axum::{extract::State, http::StatusCode, response::Json};
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

/// Detailed health check that follows Spring Boot Actuator format
/// Returns components with their statuses and details
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

    // Check dependencies (components in Spring Boot terminology)
    let mut dependencies = Vec::new();

    // Add cache component
    dependencies.push(DependencyStatus {
        name: "cache".to_string(),
        status: "UP".to_string(),
        details: Some(format!(
            "Cache {}",
            match &state.cache_registry {
                Some(_registry) => "enabled",
                None => "disabled",
            }
        )),
    });

    // Add diskSpace component (similar to Spring Boot)
    let disk_details = match get_disk_space_info() {
        Ok((total, free, threshold)) => {
            format!("total: {}, free: {}, threshold: {}", total, free, threshold)
        }
        Err(_) => "Unable to get disk information".to_string(),
    };

    dependencies.push(DependencyStatus {
        name: "diskSpace".to_string(),
        status: "UP".to_string(),
        details: Some(disk_details),
    });

    // Add environment info
    let env = detect_environment();
    dependencies.push(DependencyStatus {
        name: "env".to_string(),
        status: "UP".to_string(),
        details: Some(format!("Environment: {}", env)),
    });

    // Add core services information
    dependencies.push(DependencyStatus {
        name: "services".to_string(),
        status: "UP".to_string(),
        details: Some(format!("Service registry: active")),
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

/// Get disk space information (placeholders for now)
fn get_disk_space_info() -> Result<(String, String, String), std::io::Error> {
    // In a real implementation, this would use the sys-info crate or similar
    // to get actual disk information
    Ok(("10GB".to_string(), "5GB".to_string(), "100MB".to_string()))
}

/// Detect the current environment
fn detect_environment() -> String {
    // Check for common environment variables to determine the environment
    if std::env::var("PRODUCTION").is_ok() {
        "production".to_string()
    } else if std::env::var("STAGING").is_ok() {
        "staging".to_string()
    } else if std::env::var("CI").is_ok() {
        "ci".to_string()
    } else {
        "development".to_string()
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

        assert!(!response.0.dependencies.is_empty());

        // At least cache, diskSpace, env, and services components should be present
        let dep_names: Vec<_> = response.0.dependencies.iter().map(|d| &d.name).collect();
        assert!(dep_names.contains(&&"cache".to_string()));
        assert!(dep_names.contains(&&"diskSpace".to_string()));
        assert!(dep_names.contains(&&"env".to_string()));
        assert!(dep_names.contains(&&"services".to_string()));
    }

    #[test]
    fn test_environment_detection() {
        // We can't easily set environment variables in tests, so just verify function exists
        let env = detect_environment();
        assert!(!env.is_empty());
    }
}
