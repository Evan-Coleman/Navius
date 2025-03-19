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
    auth::{
        EntraAuthLayer, EntraTokenClient,
        middleware::{EntraAuthConfig, RoleRequirement},
    },
    cache::PetCache,
    config::AppConfig,
    generated_apis::petstore_api::models::Upet,
    handlers::{self, detailed_health_check},
    models::{ApiResponse, Data, HealthCheckResponse, MetricsResponse},
    reliability,
};

use crate::handlers::examples::catfact::fetch_catfact_handler;
use crate::handlers::examples::pet::fetch_pet_handler;
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

/// Auth requirement for a route
pub enum AuthRequirement {
    /// No authentication required
    None,
    /// Basic authentication (no specific role required)
    Authenticated,
    /// Read-only role required
    ReadOnly,
    /// Full access role required
    FullAccess,
    /// Admin role required
    Admin,
}

/// Helper function to conditionally apply authentication
fn with_auth_if_enabled(
    router: Router<Arc<AppState>>,
    auth_requirement: AuthRequirement,
    state: &Arc<AppState>,
) -> Router<Arc<AppState>> {
    if !state.config.auth.enabled || matches!(auth_requirement, AuthRequirement::None) {
        return router;
    }

    // Add logging middleware
    let router = router.route_layer(middleware::from_fn_with_state(
        state.clone(),
        handlers::logging::log_request,
    ));

    // Apply appropriate auth layer based on the requirement
    match auth_requirement {
        AuthRequirement::None => router,
        AuthRequirement::Authenticated => {
            router.layer(EntraAuthLayer::from_app_config(&state.config))
        }
        AuthRequirement::ReadOnly => router.layer(
            EntraAuthLayer::from_app_config_require_read_only_role(&state.config),
        ),
        AuthRequirement::FullAccess => router.layer(
            EntraAuthLayer::from_app_config_require_full_access_role(&state.config),
        ),
        AuthRequirement::Admin => router.layer(EntraAuthLayer::from_app_config_require_admin_role(
            &state.config,
        )),
    }
}

/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    // Create route groups with appropriate auth requirements
    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route(
            "/catfact",
            get(handlers::examples::catfact::fetch_catfact_handler),
        )
        .route("/pet/{id}", get(handlers::examples::pet::fetch_pet_handler));

    let readonly_routes =
        Router::new().route("/pet/{id}", get(handlers::examples::pet::fetch_pet_handler));

    let fullaccess_routes =
        Router::new().route("/pet/{id}", get(handlers::examples::pet::fetch_pet_handler));

    let admin_routes = Router::new()
        .route("/metrics", get(metrics))
        .route(
            "/health/detailed",
            get(detailed_health_check(state.clone())),
        )
        .route("/api/admin/cache", get(handlers::cache_debug));

    // Apply authentication conditionally to each route group
    let public_routes = with_auth_if_enabled(public_routes, AuthRequirement::None, &state);
    let readonly_routes = with_auth_if_enabled(readonly_routes, AuthRequirement::ReadOnly, &state);
    let fullaccess_routes =
        with_auth_if_enabled(fullaccess_routes, AuthRequirement::FullAccess, &state);
    let admin_routes = with_auth_if_enabled(admin_routes, AuthRequirement::Admin, &state);

    // Combine all route groups
    let router = Router::new()
        .merge(public_routes)
        .merge(readonly_routes)
        .merge(fullaccess_routes)
        .merge(admin_routes)
        .with_state(state.clone());

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
