use axum::{
    body::Body,
    extract::ConnectInfo,
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
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;

use crate::{
    auth::middleware::{EntraAuthConfig, RoleRequirement},
    auth::{EntraAuthLayer, EntraTokenClient},
    cache::PetCache,
    config::AppConfig,
    generated_apis::petstore_api::models::Upet,
    handlers,
    models::{ApiResponse, Data, HealthCheckResponse, MetricsResponse},
    reliability,
};

use crate::handlers::examples::catfact::fetch_catfact_handler;
use crate::handlers::examples::pet::fetch_pet_handler;
use crate::handlers::examples::pets::{create_pet, delete_pet, get_pet, list_pets};
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
    // Check if auth is enabled from config
    let auth_enabled = state.config.auth.enabled;

    let router = if auth_enabled {
        // Create different route groups with different authorization requirements

        // Public routes - no authentication required
        let public_routes = Router::new()
            .route("/health", get(handlers::health::health_check))
            .route("/metrics", get(metrics))
            .route(
                "/catfact",
                get(handlers::examples::catfact::fetch_catfact_handler),
            )
            .route("/pet/{id}", get(handlers::examples::pet::fetch_pet_handler));

        // Protected routes - require authentication
        let protected_routes = Router::new()
            .route("/api/pets", get(list_pets))
            .route("/api/pets/{id}", get(get_pet))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                handlers::logging::log_request,
            ))
            .layer(EntraAuthLayer::from_app_config(&state.config));

        // Read-only routes - require authentication with read-only access
        let read_only_routes = Router::new()
            .route("/api/readonly/pets", get(handlers::examples::pets::get_pet))
            .route(
                "/api/readonly/pets/{id}",
                get(handlers::examples::pets::get_pet),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                handlers::logging::log_request,
            ))
            .layer(EntraAuthLayer::from_app_config_require_read_only_role(
                &state.config,
            ));

        // Full access routes - require authentication with full access
        let full_access_routes = Router::new()
            .route(
                "/api/fullaccess/pets",
                get(handlers::examples::pets::get_pet),
            )
            .route(
                "/api/fullaccess/pets/{id}",
                get(handlers::examples::pets::get_pet),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                handlers::logging::log_request,
            ))
            .layer(EntraAuthLayer::from_app_config_require_full_access_role(
                &state.config,
            ));

        // Admin routes - require specific roles
        let admin_routes = Router::new()
            .route("/api/admin/pets", post(create_pet))
            .route("/api/admin/pets/{id}", delete(delete_pet))
            .route("/api/admin/cache", get(handlers::cache_debug))
            .route(
                "/api/admin/openapi",
                post(crate::utils::openapi::upload_openapi_spec),
            )
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                handlers::logging::log_request,
            ))
            .layer(EntraAuthLayer::from_app_config_require_admin_role(
                &state.config,
            ));

        // Combine all route groups
        Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .merge(read_only_routes)
            .merge(full_access_routes)
            .merge(admin_routes)
            .with_state(state.clone())
    } else {
        // No authentication - use the default router
        Router::new()
            .route("/health", get(handlers::health::health_check))
            .route("/metrics", get(metrics))
            .route(
                "/catfact",
                get(handlers::examples::catfact::fetch_catfact_handler),
            )
            .route("/pet/{id}", get(handlers::examples::pet::fetch_pet_handler))
            .route("/api/pets", get(list_pets))
            .route("/api/pets/{id}", get(get_pet))
            .route("/api/admin/pets", post(create_pet))
            .route("/api/admin/pets/{id}", delete(delete_pet))
            .route("/api/admin/cache", get(handlers::cache_debug))
            .route(
                "/api/admin/openapi",
                post(crate::utils::openapi::upload_openapi_spec),
            )
            .with_state(state.clone())
    };

    // Add common middleware (tracing, timeout, etc)
    let router = router
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
        ));

    // Apply reliability features
    reliability::apply_reliability(router, &state.config.reliability)
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
