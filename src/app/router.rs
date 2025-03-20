use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, State},
    http::Request,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{Router, delete, get, post},
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
    auth::{
        EntraAuthLayer, EntraTokenClient,
        middleware::{EntraAuthConfig, RoleRequirement},
    },
    config::AppConfig,
    core::CoreRouter,
    handlers::logging,
    models::{ApiResponse, Data, DetailedHealthResponse, HealthCheckResponse},
    reliability,
};

// Import user router from the crate root
use crate::app::UserRouter;

/// Application state shared across all routes
pub struct AppState {
    pub client: Client,
    pub config: AppConfig,
    pub start_time: SystemTime,
    pub cache_registry: Option<crate::cache::CacheRegistry>,
    pub metrics_handle: PrometheusHandle,
    pub token_client: Option<EntraTokenClient>,
    pub resource_registry: crate::utils::api_resource::ApiResourceRegistry,
}

/// Simple health check handler that returns a static string
/// This is used for basic health monitoring
async fn simple_health_check() -> &'static str {
    "OK"
}

/// Create the application router with standardized route groups
pub fn create_router(state: Arc<AppState>) -> Router {
    // Create logging middleware
    let logging = middleware::from_fn_with_state(state.clone(), logging::log_request);

    // First, get the core routes that should not be modified by users
    let core_routes = CoreRouter::create_core_routes(state.clone());

    // Now get the user-defined routes
    let user_routes = UserRouter::create_user_routes(state.clone());

    // Combine core routes with user-defined routes
    let app = Router::new()
        .merge(core_routes)
        .merge(user_routes)
        .layer(logging);

    // Create a plain Router with common middleware
    Router::new()
        // Use fallback_service for the main app
        .fallback_service(app)
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

/// Initialize the application
pub async fn init() -> (Router, SocketAddr) {
    // Load configuration
    let config = crate::config::app_config::load_config().expect("Failed to load configuration");

    // Initialize metrics
    let metrics_handle = crate::metrics::metrics_service::init_metrics();

    // Create HTTP client with appropriate middleware
    let client = Client::builder()
        .timeout(Duration::from_secs(config.server.timeout_seconds))
        .build()
        .expect("Failed to create HTTP client");

    // Create application state
    let start_time = SystemTime::now();

    // Only set up the cache if enabled
    let cache_registry = if config.cache.enabled {
        let registry = crate::cache::init_cache_registry(
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
        crate::cache::start_metrics_updater(registry).await;
    } else {
        info!("🔧 Cache registry disabled, metrics updater not started");
    }

    // Create router
    let app = create_router(state);

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (app, addr)
}
