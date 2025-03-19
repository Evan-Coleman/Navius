use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::Request,
    middleware::{self, Next},
    response::Response,
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
    cache::{CacheWithTTL, PetCache},
    config::AppConfig,
    generated_apis::petstore_api::models::Upet,
    handlers::{self},
    models::{ApiResponse, Data, HealthCheckResponse, MetricsResponse},
    reliability,
};

use crate::handlers::examples::pet::fetch_pet_handler;
use crate::handlers::health::detailed_health_check;
use crate::handlers::metrics::metrics;

/// Application state shared across all routes
pub struct AppState {
    pub client: Client,
    pub config: AppConfig,
    pub start_time: SystemTime,
    pub pet_cache: Option<CacheWithTTL>,
    pub metrics_handle: PrometheusHandle,
    pub token_client: Option<EntraTokenClient>,
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
        .route("/health", get(handlers::health::health_check))
        .route("/pet/{id}", get(fetch_pet_handler.clone()));

    // 2. READ-ONLY ROUTES - requires basic authentication
    let readonly_routes = Router::new()
        .route("/pet/{id}", get(fetch_pet_handler.clone()))
        .route_layer(logging.clone())
        .route_layer(readonly_auth.clone());

    // 3. FULL ACCESS ROUTES - requires admin role
    let fullaccess_routes = Router::new()
        .route("/pet/{id}", get(fetch_pet_handler.clone()))
        .route_layer(logging.clone())
        .route_layer(fullaccess_auth.clone());

    // 4. ACTUATOR ROUTES - for metrics, health checks, and admin functions
    let actuator_routes = Router::new()
        .route("/health", get(detailed_health_check))
        .route("/metrics", get(metrics))
        .route("/cache", get(handlers::cache_admin::cache_debug));

    // Apply authentication to actuator routes if enabled
    let actuator_routes = if auth_enabled {
        actuator_routes
            .route_layer(logging.clone())
            .route_layer(admin_auth)
    } else {
        actuator_routes
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

    // Initialize pet cache if enabled
    let pet_cache = if config.cache.enabled {
        Some(crate::cache::init_cache(
            config.cache.max_capacity,
            config.cache.ttl_seconds,
        ))
    } else {
        info!("🔧 Pet cache disabled");
        None
    };

    // Create the token client if auth is enabled
    let token_client = if config.auth.enabled {
        Some(EntraTokenClient::from_config(&config))
    } else {
        None
    };

    let state = Arc::new(AppState {
        client,
        config: config.clone(),
        start_time,
        pet_cache: pet_cache.clone(),
        metrics_handle,
        token_client,
    });

    // Start metrics updater with the cloned cache
    if let Some(cache) = pet_cache {
        crate::cache::cache_manager::start_metrics_updater(Some(cache));
    } else {
        info!("🔧 Pet cache disabled, metrics updater not started");
    }

    // Create router
    let app = create_router(state);

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (app, addr)
}
