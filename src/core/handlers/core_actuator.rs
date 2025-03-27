use axum::{extract::State, response::Json};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{debug, info};

use crate::core::{
    models::{ActuatorEntry, InfoResponse},
    router::AppState,
};

/// Handler for the info endpoint
///
/// Returns basic information about the application, like version,
/// build info, etc. This is useful for verifying what version
/// of the application is running.
///
/// Models the Spring Boot Actuator info endpoint structure.
pub async fn info(State(_state): State<Arc<AppState>>) -> Json<Value> {
    info!("📊 Getting application info");

    // Create Spring Boot-style info response with sections
    let response = json!({
        "app": {
            "name": env!("CARGO_PKG_NAME"),
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "version": env!("CARGO_PKG_VERSION"),
            "encoding": "UTF-8",
            "java": {
                "equivalent": "Spring Boot 3.0"
            }
        },
        "build": {
            "artifact": env!("CARGO_PKG_NAME"),
            "name": env!("CARGO_PKG_NAME"),
            "time": get_build_time(),
            "version": env!("CARGO_PKG_VERSION"),
            "group": "io.navius"
        },
        "git": get_git_info(),
        "env": {
            "active": detect_active_environment(),
            "features": get_active_features()
        }
    });

    Json(response)
}

/// Returns the time the application was built
fn get_build_time() -> String {
    // In a real implementation, this would be derived from build info
    // or a timestamp file generated during the build process

    // For now, return a placeholder
    "2025-06-15T16:34:00Z".to_string()
}

/// Returns information about the git repository
fn get_git_info() -> Value {
    // In a real implementation, this would use the git2 crate
    // or system calls to get actual git information

    // For now, return a placeholder JSON structure
    json!({
        "branch": "main",
        "commit": {
            "id": "abc123def456",
            "time": "2025-06-15T16:30:00Z"
        }
    })
}

/// Detects the active environment profile
fn detect_active_environment() -> String {
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

/// Returns a list of active feature flags
fn get_active_features() -> Vec<String> {
    let mut features = Vec::new();

    // Add features based on compile-time feature flags
    #[cfg(feature = "default")]
    features.push("default".to_string());

    #[cfg(feature = "examples")]
    features.push("examples".to_string());

    #[cfg(feature = "production")]
    features.push("production".to_string());

    features
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_git_info() {
        let info = get_git_info();
        assert!(info.get("branch").is_some());
        assert!(info.get("commit").is_some());
    }

    #[test]
    fn test_detect_active_environment() {
        let env = detect_active_environment();
        assert!(!env.is_empty());
    }

    #[test]
    fn test_get_active_features() {
        let features = get_active_features();
        // We should have at least one feature active
        assert!(!features.is_empty());
    }
}
