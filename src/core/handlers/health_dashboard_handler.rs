use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::core::router::AppState;
use crate::core::services::health_dashboard::{HealthDashboardConfig, HealthDashboardService};
use crate::core::services::health_discovery::HealthDiscoveryService;
use crate::core::services::health_indicators::CoreHealthIndicatorProvider;
use crate::core::services::health_provider::{HealthConfig, HealthIndicatorProviderRegistry};

/// Dashboard query parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardQuery {
    /// Whether to include full history
    #[serde(default)]
    pub history: bool,

    /// Whether to include performance metrics
    #[serde(default)]
    pub performance: bool,

    /// Whether to include detailed component information
    #[serde(default = "default_true")]
    pub details: bool,

    /// Max history entries to include
    #[serde(default = "default_max_history")]
    pub max_history: usize,
}

fn default_true() -> bool {
    true
}

fn default_max_history() -> usize {
    10
}

/// Create a health dashboard service for the request
fn create_dashboard_service() -> Arc<HealthDashboardService> {
    // Create registry and register core provider
    let mut registry = HealthIndicatorProviderRegistry::new();
    registry.register(Box::new(CoreHealthIndicatorProvider));
    let registry = Arc::new(registry);

    // Create discovery service
    let discovery = HealthDiscoveryService::new(registry, HealthConfig::default());
    let discovery = Arc::new(RwLock::new(discovery));

    // Create dashboard
    let dashboard = HealthDashboardService::new(discovery, HealthDashboardConfig::default());

    Arc::new(dashboard)
}

/// Get a cached dashboard service from app state or create a new one
fn get_or_create_dashboard(state: &Arc<AppState>) -> Arc<HealthDashboardService> {
    // In a real implementation, we would store this in AppState
    // For now, we'll create a new one each time
    create_dashboard_service()
}

/// Enhanced health dashboard endpoint
pub async fn health_dashboard_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DashboardQuery>,
) -> impl IntoResponse {
    // Get or create dashboard service
    let dashboard = get_or_create_dashboard(&state);

    // Create custom config from query parameters
    let config = HealthDashboardConfig {
        include_history: params.history,
        include_performance: params.performance,
        detailed_components: params.details,
        max_history: params.max_history,
        ..HealthDashboardConfig::default()
    };

    // Check health
    match dashboard.check_health(&state).await {
        Ok(health_status) => {
            // Return success
            (StatusCode::OK, Json(health_status))
        }
        Err(e) => {
            // Return error
            let error_response = json!({
                "status": "ERROR",
                "error": {
                    "message": e.to_string(),
                }
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// Clear health dashboard history
pub async fn clear_dashboard_history(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let dashboard = get_or_create_dashboard(&state);

    match dashboard.clear_history() {
        Ok(_) => {
            // Return success
            let response = json!({
                "status": "OK",
                "message": "History cleared"
            });
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            // Return error
            let error_response = json!({
                "status": "ERROR",
                "error": {
                    "message": e.to_string(),
                }
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// Register a dynamic health indicator
pub async fn register_dynamic_indicator(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    // In a real implementation, we would parse the payload and register the indicator
    // For this example, we'll return a not implemented response
    let response = json!({
        "status": "ERROR",
        "error": {
            "message": "Dynamic indicator registration not implemented in this handler"
        }
    });

    (StatusCode::NOT_IMPLEMENTED, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{self, Body},
        http::{Method, Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_dashboard_handler() {
        // Create app state
        let state = Arc::new(AppState::default());

        // Create router
        let app = Router::new()
            .route("/actuator/dashboard", get(health_dashboard_handler))
            .with_state(state);

        // Create request
        let request = Request::builder()
            .uri("/actuator/dashboard?details=true&performance=true")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        // Process request
        let response = app.oneshot(request).await.unwrap();

        // Check status
        assert_eq!(response.status(), StatusCode::OK);

        // Parse body
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Verify response
        assert!(json.get("status").is_some());
        assert!(json.get("components").is_some());
    }

    #[tokio::test]
    async fn test_clear_dashboard_history() {
        // Create app state
        let state = Arc::new(AppState::default());

        // Create router
        let app = Router::new()
            .route(
                "/actuator/dashboard/history/clear",
                get(clear_dashboard_history),
            )
            .with_state(state);

        // Create request
        let request = Request::builder()
            .uri("/actuator/dashboard/history/clear")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        // Process request
        let response = app.oneshot(request).await.unwrap();

        // Check status
        assert_eq!(response.status(), StatusCode::OK);

        // Parse body
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Verify response
        assert_eq!(json["status"], "OK");
        assert_eq!(json["message"], "History cleared");
    }
}
