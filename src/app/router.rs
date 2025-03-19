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
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;

use crate::{
    auth::{
        EntraAuthLayer, EntraTokenClient,
        middleware::{EntraAuthConfig, RoleRequirement},
    },
    cache::PetCache,
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
    pub pet_cache: Option<PetCache>,
    pub metrics_handle: PrometheusHandle,
    pub token_client: Option<EntraTokenClient>,
}

/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    // Build a regular Router without state
    let mut api_router = Router::new();

    // Add public routes
    api_router = api_router
        .route("/health", get(handlers::health::health_check))
        .route("/pet/{id}", get(fetch_pet_handler));

    // Conditionally add authenticated routes
    if state.config.auth.enabled {
        // Add logging middleware to auth routes
        let logging = middleware::from_fn_with_state(state.clone(), handlers::logging::log_request);

        // Add authenticated routes
        api_router = api_router.route(
            "/metrics",
            get(metrics)
                .route_layer(logging.clone())
                .route_layer(EntraAuthLayer::from_app_config(&state.config)),
        );

        // Add admin routes
        api_router = api_router
            .route(
                "/health/detailed",
                get(detailed_health_check)
                    .route_layer(logging.clone())
                    .route_layer(EntraAuthLayer::from_app_config_require_admin_role(
                        &state.config,
                    )),
            )
            .route(
                "/api/admin/cache",
                get(handlers::cache_debug)
                    .route_layer(logging.clone())
                    .route_layer(EntraAuthLayer::from_app_config_require_admin_role(
                        &state.config,
                    )),
            );
    } else {
        // Add non-authenticated versions of the routes
        api_router = api_router
            .route("/metrics", get(metrics))
            .route("/health/detailed", get(detailed_health_check))
            .route("/api/admin/cache", get(handlers::cache_debug));
    }

    // Add state to router (this is what would make it Router<Arc<AppState>>)
    let api_router = api_router.with_state(state.clone());

    // Create a plain Router with common middleware
    Router::new()
        // Use fallback_service instead of nest_service at root
        .fallback_service(api_router)
        // Add tracing
        .layer(TraceLayer::new_for_http())
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
        info!(
            "🔧 Initializing pet cache with capacity {} and TTL {} seconds",
            config.cache.max_capacity, config.cache.ttl_seconds
        );
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
        info!("🔧 Starting metrics updater for cache");
        crate::cache::cache_manager::start_metrics_updater(start_time, Some(cache));
    } else {
        info!("🔧 Metrics updater not started - cache is disabled");
    }

    // Create router
    let app = create_router(state);

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (app, addr)
}
