use axum::{
    body::Body,
    extract::ConnectInfo,
    http::Request,
    middleware::{self, Next},
    response::Response,
    routing::{Router, get},
};
use metrics_exporter_prometheus::PrometheusHandle;
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc, time::SystemTime};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

use crate::{
    cache::PetCache,
    config::AppConfig,
    generated_apis::petstore_api::models::Upet,
    handlers,
    models::{ApiResponse, Data, HealthCheckResponse, MetricsResponse},
};

/// Application state shared across all routes
pub struct AppState {
    pub client: Client,
    pub config: AppConfig,
    pub start_time: SystemTime,
    pub pet_cache: Option<PetCache>,
    pub metrics_handle: PrometheusHandle,
}

/// API Security Scheme
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("api_key"))),
        );
    }
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pet Store API",
        version = "1.0.0",
        description = "A sample Pet Store server with OpenAPI model integration"
    ),
    paths(
        crate::handlers::health::health_check,
        crate::handlers::metrics::metrics,
        crate::handlers::data::get_data,
        crate::handlers::pet::get_pet_by_id,
    ),
    components(
        schemas(
            HealthCheckResponse,
            MetricsResponse,
            Data,
            ApiResponse,
            Upet,
            crate::generated_apis::petstore_api::models::Category,
            crate::generated_apis::petstore_api::models::Tag,
            crate::cache::CacheStats,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "metrics", description = "Prometheus metrics endpoints"),
        (name = "data", description = "Data endpoints"),
        (name = "pets", description = "Pet endpoints"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::metrics))
        .route("/data", get(handlers::data::get_data))
        .route("/pet/{id}", get(handlers::pet::get_pet_by_id))
        // Add tracing
        .layer(TraceLayer::new_for_http())
        // Add timeout
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(
            state.config.server.timeout_seconds,
        )))
        .with_state(state)
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
    let config = crate::config::load_config().expect("Failed to load configuration");

    // Initialize metrics
    let metrics_handle = crate::metrics::init_metrics();

    // Initialize HTTP client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(
            config.server.timeout_seconds,
        ))
        .build()
        .expect("Failed to create HTTP client");

    // Initialize cache if enabled
    let pet_cache = if config.cache.enabled {
        Some(crate::cache::init_cache(
            config.cache.max_capacity,
            config.cache.ttl_seconds,
        ))
    } else {
        None
    };

    // Create application state
    let start_time = SystemTime::now();
    let state = Arc::new(AppState {
        client,
        config: config.clone(),
        start_time,
        pet_cache: pet_cache.clone(),
        metrics_handle,
    });

    // Start metrics updater
    crate::cache::start_metrics_updater(start_time, pet_cache);

    // Create router
    let app = create_router(state);

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (app, addr)
}
