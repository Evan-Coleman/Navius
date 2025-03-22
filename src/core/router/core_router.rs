use axum::{
    extract::State,
    routing::{Router, get},
};
use std::sync::Arc;

use crate::{
    core::auth::EntraAuthLayer,
    handlers::{self, actuator, health},
};

use super::AppState;

/// Core router containing essential routes that should not be modified by users
pub struct CoreRouter;

impl CoreRouter {
    /// Create the essential core routes that should not be modified by users
    pub fn create_core_routes(state: Arc<AppState>) -> Router {
        // Define whether auth is enabled
        let auth_enabled = state.config.auth.enabled;

        // Create auth middleware for admin access
        let admin_auth = EntraAuthLayer::from_app_config_require_admin_role(&state.config);

        // Public core routes - accessible without authentication
        let public_routes = Router::new().route("/health", get(health::health_check));

        // Actuator routes - for metrics, health checks, docs, and admin functions
        let actuator_routes = Router::new()
            .route("/health", get(health::detailed_health_check))
            .route("/info", get(actuator::info))
            .route("/docs", get(handlers::docs::swagger_ui_handler))
            .route("/docs/{*file}", get(handlers::docs::openapi_spec_handler));

        // Apply authentication layers if enabled
        let actuator_routes = if auth_enabled {
            actuator_routes.layer(admin_auth)
        } else {
            actuator_routes
        };

        // Return only the core routes
        Router::new()
            .merge(public_routes)
            .nest("/actuator", actuator_routes)
            .with_state(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{auth::EntraTokenClient, config::app_config::AppConfig},
        models::{DetailedHealthResponse, HealthCheckResponse},
        utils::api_resource::ApiResourceRegistry,
    };
    use axum::{
        Router,
        body::{self, Body},
        http::{Method, Request, StatusCode},
        response::Response,
    };
    use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
    use reqwest::Client;
    use std::{sync::Arc, time::SystemTime};
    use tower::ServiceExt;

    // Helper function to create a minimal test state
    fn create_test_state(auth_enabled: bool) -> Arc<AppState> {
        let mut config = AppConfig::default();
        config.auth.enabled = auth_enabled;

        let metrics_recorder = PrometheusBuilder::new().build_recorder();
        let metrics_handle: PrometheusHandle = metrics_recorder.handle();

        Arc::new(AppState {
            client: Client::new(),
            config,
            start_time: SystemTime::now(),
            cache_registry: None,
            metrics_handle,
            token_client: None,
            resource_registry: ApiResourceRegistry::new(),
            db_pool: None,
        })
    }

    // Helper to make a request to the router
    async fn send_request(router: Router, uri: &str, method: Method) -> Response {
        let req = Request::builder()
            .uri(uri)
            .method(method)
            .body(Body::empty())
            .unwrap();

        router.oneshot(req).await.unwrap()
    }

    #[tokio::test]
    async fn test_public_routes_registration() {
        // Create state with auth disabled
        let state = create_test_state(false);

        // Create the core routes
        let router = CoreRouter::create_core_routes(state);

        // Test the public health endpoint
        let response = send_request(router, "/health", Method::GET).await;

        // Verify status and response body
        assert_eq!(response.status(), StatusCode::OK);
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_response: HealthCheckResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health_response.status, "healthy");
    }

    #[tokio::test]
    async fn test_actuator_routes_registration() {
        // Create state with auth disabled for easy testing
        let state = create_test_state(false);

        // Create the core routes
        let router = CoreRouter::create_core_routes(state);

        // Test the actuator health endpoint
        let response = send_request(router.clone(), "/actuator/health", Method::GET).await;

        // Verify status and response type
        assert_eq!(response.status(), StatusCode::OK);
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health_response: DetailedHealthResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(health_response.status, "healthy");

        // Test actuator info endpoint
        let response = send_request(router, "/actuator/info", Method::GET).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_actuator_routes_with_auth_enabled() {
        // Create state with auth enabled
        let state = create_test_state(true);

        // Create the core routes
        let router = CoreRouter::create_core_routes(state);

        // Test the actuator health endpoint without auth token
        // This should fail with 401 Unauthorized because auth is enabled
        let response = send_request(router, "/actuator/health", Method::GET).await;

        // Verify we get 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_route_not_found() {
        // Create state with auth disabled
        let state = create_test_state(false);

        // Create the core routes
        let router = CoreRouter::create_core_routes(state);

        // Test a non-existent route
        let response = send_request(router, "/not-found", Method::GET).await;

        // Verify we get a 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

// Test moved to tests/router_integration_test.rs
