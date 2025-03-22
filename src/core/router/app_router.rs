use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::Request,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{Router, get, post},
};
use metrics_exporter_prometheus::PrometheusHandle;
use reqwest::Client;
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info};

use crate::{
    core::auth::{
        EntraAuthLayer, EntraTokenClient,
        middleware::{EntraAuthConfig, RoleRequirement},
    },
    core::config::app_config::AppConfig,
    handlers::logging,
    models::{ApiResponse, DetailedHealthResponse, HealthCheckResponse},
    reliability,
};

use super::CoreRouter;

/// Application state shared across all routes
pub struct AppState {
    pub client: Client,
    pub config: AppConfig,
    pub start_time: SystemTime,
    pub cache_registry: Option<crate::core::cache::CacheRegistry>,
    pub metrics_handle: PrometheusHandle,
    pub token_client: Option<EntraTokenClient>,
    pub resource_registry: crate::utils::api_resource::ApiResourceRegistry,
}

/// Create the core application router with middleware
pub fn create_core_app_router(state: Arc<AppState>, user_routes: Router) -> Router {
    // Create logging middleware
    let logging = middleware::from_fn_with_state(state.clone(), logging::log_request);

    // Get the core routes that should not be modified by users
    let core_routes = CoreRouter::create_core_routes(state.clone());

    // Combine core routes with user-defined routes and add all middleware
    Router::new()
        .merge(core_routes)
        .merge(user_routes)
        .layer(logging)
        // Add tracing with custom configuration that doesn't duplicate our logging
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    DefaultMakeSpan::new()
                        .include_headers(false)
                        .level(Level::DEBUG),
                )
                // Set level to TRACE to avoid duplicating our INFO logs
                .on_response(
                    DefaultOnResponse::new()
                        .include_headers(false)
                        .level(Level::TRACE),
                ),
        )
        // Add timeout
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(
            state.config.server.timeout_seconds,
        )))
        // Add socket address info to request extensions
        .layer(middleware::from_fn(
            |req: Request<Body>, next: Next| async move {
                let conn_info_opt = req.extensions().get::<ConnectInfo<SocketAddr>>().cloned();

                if let Some(conn_info) = conn_info_opt {
                    let mut req = req;
                    req.extensions_mut().insert(ConnectInfo(conn_info.0));
                    next.run(req).await
                } else {
                    next.run(req).await
                }
            },
        ))
}

/// Initialize the application state and resources
pub async fn init_app_state() -> (Arc<AppState>, SocketAddr) {
    // Load configuration
    let config = crate::core::config::load_config().expect("Failed to load config");

    // Initialize metrics
    let metrics_handle = crate::core::metrics::init_metrics();

    // Create HTTP client with appropriate middleware
    let client = Client::builder()
        .timeout(Duration::from_secs(config.server.timeout_seconds))
        .build()
        .expect("Failed to create HTTP client");

    // Create application state
    let start_time = SystemTime::now();

    // Only set up the cache if enabled
    let cache_registry = if config.cache.enabled {
        let registry = crate::core::cache::init_cache_registry(
            true,
            config.cache.max_capacity,
            config.cache.ttl_seconds,
        );

        Some(registry)
    } else {
        info!("🔧 Cache disabled");
        None
    };

    // Create API resource registry
    let resource_registry = crate::utils::api_resource::ApiResourceRegistry::new();

    // Create the app state
    let state = Arc::new(AppState {
        client,
        config: config.clone(),
        start_time,
        cache_registry: cache_registry.clone(),
        metrics_handle,
        token_client: if config.auth.enabled {
            Some(EntraTokenClient::from_config(&config))
        } else {
            None
        },
        resource_registry,
    });

    // Register pet resources in the cache registry
    if let Some(_registry) = &cache_registry {
        // Register the Upet resource type with the cache
        use crate::generated_apis::petstore_api::models::Upet;
        use crate::utils::api_resource::{ApiResource, register_resource};

        // Make sure the pet resource type is registered with the cache
        match register_resource::<Upet>(&state, None) {
            Ok(_) => info!("✅ Successfully registered pet resource type in cache registry"),
            Err(e) => info!("⚠️ Failed to register pet resource: {}", e),
        }
    }

    // Start metrics updater for the new cache registry
    if let Some(registry) = &cache_registry {
        crate::core::cache::start_metrics_updater(registry).await;
    } else {
        info!("🔧 Cache registry disabled, metrics updater not started");
    }

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (state, addr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{self, Body},
        http::{HeaderMap, HeaderValue, Method, Request, StatusCode},
        response::Response,
        routing::get,
    };
    use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
    use std::sync::Arc;
    use tower::ServiceExt;

    // Simple handler for testing
    async fn test_handler() -> &'static str {
        "test_response"
    }

    // Handler that returns headers from the request
    async fn echo_headers(headers: HeaderMap) -> String {
        headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("invalid")))
            .collect::<Vec<String>>()
            .join(", ")
    }

    // Create minimal app state for testing
    fn create_test_state() -> Arc<AppState> {
        let config = AppConfig::default();
        let metrics_recorder = PrometheusBuilder::new().build_recorder();
        let metrics_handle: PrometheusHandle = metrics_recorder.handle();

        Arc::new(AppState {
            client: Client::new(),
            config,
            start_time: SystemTime::now(),
            cache_registry: None,
            metrics_handle,
            token_client: None,
            resource_registry: crate::utils::api_resource::ApiResourceRegistry::new(),
        })
    }

    // Helper to make a request to the router with optional headers
    async fn send_request(
        router: Router,
        uri: &str,
        method: Method,
        headers: Option<HeaderMap>,
    ) -> Response {
        let mut req_builder = Request::builder().uri(uri).method(method);

        // Add headers if provided
        if let Some(hdrs) = headers {
            for (name, value) in hdrs.iter() {
                req_builder = req_builder.header(name, value);
            }
        }

        let req = req_builder.body(Body::empty()).unwrap();
        router.oneshot(req).await.unwrap()
    }

    #[tokio::test]
    async fn test_merge_user_routes() {
        // Create test state
        let state = create_test_state();

        // Create test user routes
        let user_routes = Router::new().route("/test", get(test_handler));

        // Create app router by merging core and user routes
        let app_router = create_core_app_router(state, user_routes);

        // Test user route
        let response = send_request(app_router, "/test", Method::GET, None).await;
        assert_eq!(response.status(), StatusCode::OK);

        // Get response body
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "test_response");
    }

    #[tokio::test]
    async fn test_timeout_middleware() {
        // Create test state with very short timeout
        let state = {
            let config = {
                let mut cfg = AppConfig::default();
                // Set a very low timeout for testing
                cfg.server.timeout_seconds = 1;
                cfg
            };

            let metrics_recorder = PrometheusBuilder::new().build_recorder();
            let metrics_handle: PrometheusHandle = metrics_recorder.handle();

            Arc::new(AppState {
                client: Client::new(),
                config,
                start_time: SystemTime::now(),
                cache_registry: None,
                metrics_handle,
                token_client: None,
                resource_registry: crate::utils::api_resource::ApiResourceRegistry::new(),
            })
        };

        // Handler that simulates a slow response
        async fn slow_handler() -> &'static str {
            // Sleep for longer than the timeout
            tokio::time::sleep(Duration::from_secs(2)).await;
            "slow_response"
        }

        // Create test user routes with a slow handler
        let user_routes = Router::new().route("/slow", get(slow_handler));

        // Create app router with the timeout middleware
        let app_router = create_core_app_router(state, user_routes);

        // Test slow route - should time out
        let response = send_request(app_router, "/slow", Method::GET, None).await;

        // Expect request timeout status code
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[tokio::test]
    async fn test_request_headers_middleware() {
        // Create test state
        let state = create_test_state();

        // User route that tests if headers are properly propagated
        let user_routes = Router::new().route("/echo-headers", get(echo_headers));

        // Create app router with middleware
        let app_router = create_core_app_router(state, user_routes);

        // Create custom headers
        let mut headers = HeaderMap::new();
        headers.insert("X-Test-Header", HeaderValue::from_static("test-value"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        // Send request with headers
        let response = send_request(app_router, "/echo-headers", Method::GET, Some(headers)).await;
        assert_eq!(response.status(), StatusCode::OK);

        // Verify headers were passed through middleware
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Check that our test headers are present
        assert!(body_str.contains("x-test-header: test-value"));
        assert!(body_str.contains("accept: application/json"));
    }

    #[tokio::test]
    async fn test_init_app_state() {
        // Mock environment variables for testing - using unsafe blocks for environment manipulation
        unsafe {
            std::env::set_var("RUST_BACKEND_SERVER_PORT", "8081");
            std::env::set_var("RUST_BACKEND_CACHE_ENABLED", "false");
        }

        // Initialize app state
        let (state, addr) = init_app_state().await;

        // Verify state was initialized correctly
        assert_eq!(addr.port(), 8081);
        assert_eq!(state.config.cache.enabled, false);
        assert!(state.cache_registry.is_none());

        // Cleanup environment variables
        unsafe {
            std::env::remove_var("RUST_BACKEND_SERVER_PORT");
            std::env::remove_var("RUST_BACKEND_CACHE_ENABLED");
        }
    }
}
