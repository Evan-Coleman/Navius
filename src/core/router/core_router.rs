use axum::{
    extract::State,
    routing::{Router, get, post},
};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{sync::Arc, time::SystemTime};

use crate::core::handlers::health_dashboard_handler::{
    clear_dashboard_history, health_dashboard_handler, register_dynamic_indicator,
};
use crate::core::{
    auth::middleware::EntraAuthLayer,
    config::app_config::AppConfig,
    handlers::{
        self, core_actuator, core_docs,
        core_health::{detailed_health_handler, health_handler},
    },
    models::{DetailedHealthResponse, HealthCheckResponse},
    router::core_app_router::ServiceRegistry,
};

use super::AppState;

/// Core router containing essential routes that should not be modified by users
pub struct CoreRouter;

impl CoreRouter {
    /// Creates the core routes for the application
    pub fn create_core_routes(state: Arc<AppState>) -> Router {
        // Get the auth enabled flag from config
        let auth_enabled = state.config.auth.enabled;

        // Create admin auth middleware only if auth is enabled
        let admin_auth = if auth_enabled {
            Some(EntraAuthLayer::from_app_config_require_admin(&state.config))
        } else {
            None
        };

        // Public core routes - accessible without authentication
        let public_routes = Router::new().route("/health", get(health_handler));

        // Create actuator routes
        let mut actuator_routes = Router::new();

        // Add all actuator routes
        actuator_routes = actuator_routes
            .route("/health", get(detailed_health_handler))
            .route("/info", get(core_actuator::info))
            .route("/docs", get(core_docs::swagger_ui_handler))
            .route("/docs/{*file}", get(core_docs::openapi_spec_handler))
            // Add health dashboard routes
            .route("/dashboard", get(health_dashboard_handler))
            .route("/dashboard/history/clear", get(clear_dashboard_history))
            .route("/dashboard/register", post(register_dynamic_indicator));

        // Apply authentication layers if enabled
        let actuator_routes = if auth_enabled {
            actuator_routes.layer(admin_auth.unwrap())
        } else {
            actuator_routes
        };

        // Return the final router with all routes
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
        core::{auth::EntraTokenClient, utils::api_resource::ApiResourceRegistry},
        models::{DetailedHealthResponse, HealthCheckResponse},
    };
    use axum::{
        Router,
        body::{self, Body},
        http::{Method, Request, StatusCode},
        response::Response,
    };
    use reqwest::Client;
    use std::{sync::Arc, time::SystemTime};
    use tower::ServiceExt;

    // Helper function to create a minimal test state
    fn create_test_state(auth_enabled: bool) -> Arc<AppState> {
        let mut config = AppConfig::default();
        config.auth.enabled = auth_enabled;

        // Add a default provider and role mappings to avoid the "Default provider not found" error
        if auth_enabled {
            use crate::core::config::app_config::ProviderConfig;
            use std::collections::HashMap;

            // Create a default provider config manually
            let provider_config = ProviderConfig {
                enabled: true,
                client_id: "test-client-id".to_string(),
                jwks_uri: "https://test.jwks".to_string(),
                issuer_url: "https://test.issuer".to_string(),
                audience: "test-audience".to_string(),
                role_mappings: {
                    let mut mappings = HashMap::new();
                    mappings.insert("admin".to_string(), vec!["Admin".to_string()]);
                    mappings.insert("read_only".to_string(), vec!["Reader".to_string()]);
                    mappings.insert("full_access".to_string(), vec!["Editor".to_string()]);
                    mappings
                },
                provider_specific: HashMap::new(),
            };

            // Add the provider to config
            config
                .auth
                .providers
                .insert("test-provider".to_string(), provider_config);
            config.auth.default_provider = "test-provider".to_string();
        }

        let metrics_recorder = PrometheusBuilder::new().build_recorder();
        let metrics_handle = metrics_recorder.handle();

        Arc::new(AppState {
            client: None,
            config,
            start_time: SystemTime::now(),
            cache_registry: None,
            metrics_handle: Some(metrics_handle),
            token_client: None,
            resource_registry: None,
            service_registry: Arc::new(ServiceRegistry::new()),
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
        assert_eq!(health_response.status, "UP");
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
        // The response is a JSON Value, not a DetailedHealthResponse struct
        let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // In tests, detailed health check might return DOWN since no real services are running
        assert!(health_response.get("status").is_some());

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
