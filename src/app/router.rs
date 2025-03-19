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
    handlers::{self, actuator, cache_admin, health, metrics as metrics_handlers},
    models::{ApiResponse, Data, DetailedHealthResponse, HealthCheckResponse, MetricsResponse},
    reliability,
};

// Import the specific modules to avoid confusion
use crate::handlers::examples::pet;

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
    // Define whether auth is enabled
    let auth_enabled = state.config.auth.enabled;

    // Create logging middleware
    let logging = middleware::from_fn_with_state(state.clone(), handlers::logging::log_request);

    // Create auth middleware for different access levels
    let readonly_auth = EntraAuthLayer::from_app_config_require_read_only_role(&state.config);
    let fullaccess_auth = EntraAuthLayer::from_app_config_require_full_access_role(&state.config);
    let admin_auth = EntraAuthLayer::from_app_config_require_admin_role(&state.config);

    // 1. PUBLIC ROUTES - available without authentication
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/pet/{id}", get(pet::fetch_pet_handler));

    // 2. READ-ONLY ROUTES - requires basic authentication
    let readonly_routes = Router::new().route("/pet/{id}", get(pet::fetch_pet_handler));

    // 3. FULL ACCESS ROUTES - requires admin role
    let fullaccess_routes = Router::new().route("/pet/{id}", get(pet::fetch_pet_handler));

    // 4. ACTUATOR ROUTES - for metrics, health checks, and admin functions
    let actuator_routes = Router::new()
        // Use the actuator health handlers that are specifically designed for the router
        .route("/health", get(health::detailed_health_check))
        // Use the metrics handler
        .route("/metrics", get(metrics_handlers::metrics))
        // Use the cache debug handler
        .route("/cache", get(cache_admin::show_cache_info))
        // Use the info handler
        .route("/info", get(actuator::info));

    // Apply authentication layers if enabled
    let (readonly_routes, fullaccess_routes, actuator_routes) = if auth_enabled {
        let readonly_with_auth = readonly_routes.layer(logging.clone()).layer(readonly_auth);

        let fullaccess_with_auth = fullaccess_routes
            .layer(logging.clone())
            .layer(fullaccess_auth);

        let actuator_with_auth = actuator_routes.layer(logging.clone()).layer(admin_auth);

        (readonly_with_auth, fullaccess_with_auth, actuator_with_auth)
    } else {
        // No auth enabled, return routes as-is
        (readonly_routes, fullaccess_routes, actuator_routes)
    };

    // Combine all route groups
    let app = Router::new()
        .merge(public_routes)
        .nest("/read", readonly_routes)
        .nest("/full", fullaccess_routes)
        .nest("/actuator", actuator_routes)
        .with_state(state.clone());

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
