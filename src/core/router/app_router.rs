use axum::{
    Json,
    body::{self, Body},
    extract::{ConnectInfo, Extension, Path, Query, Request as AxumRequest, State},
    http::{HeaderMap, Method, Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response as AxumResponse},
    routing::{Router, get, post},
};
use chrono::{DateTime, Utc};
use config::ConfigError;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use reqwest::Client;
use sqlx::{Pool, Postgres};
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
use uuid::Uuid;

use crate::{
    app::api::handlers::{
        health_check::health_check as app_health_check,
        metrics_handler::metrics_handler as app_metrics_handler,
    },
    auth::middleware::EntraAuthLayer,
    core::auth::{EntraTokenClient, middleware::RoleRequirement, mock::MockTokenClient},
    core::cache::CacheRegistry,
    core::config::app_config::AppConfig,
    core::database::PgPool,
    core::error::AppError,
    core::handlers::logging,
    core::metrics::metrics_service,
    core::models::{ApiResponse, DetailedHealthResponse, HealthCheckResponse},
    core::services::ServiceRegistry,
    core::utils::api_resource::ApiResourceRegistry,
    reliability,
};

use super::CoreRouter;
use crate::core::auth::TokenClient;

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<dyn crate::core::auth::TokenClient + Send + Sync>,
    pub config: Arc<AppConfig>,
    pub start_time: DateTime<Utc>,
    pub cache_registry: Arc<CacheRegistry>,
    pub metrics_handle: Arc<PrometheusHandle>,
    pub service_registry: Arc<ServiceRegistry>,
    pub db_pool: Option<Arc<Pool<Postgres>>>,
}

impl AppState {
    pub fn new(
        client: Arc<dyn crate::core::auth::TokenClient + Send + Sync>,
        config: Arc<AppConfig>,
        start_time: DateTime<Utc>,
        cache_registry: Arc<CacheRegistry>,
        metrics_handle: Arc<PrometheusHandle>,
        service_registry: Arc<ServiceRegistry>,
        db_pool: Option<Arc<Pool<Postgres>>>,
    ) -> Self {
        Self {
            client,
            config,
            start_time,
            cache_registry,
            metrics_handle,
            service_registry,
            db_pool,
        }
    }

    pub fn get_db_pool(&self) -> Result<Arc<Pool<Postgres>>, AppError> {
        self.db_pool
            .clone()
            .ok_or_else(|| AppError::DatabaseError("Database pool not initialized".to_string()))
    }
}

impl Default for AppState {
    fn default() -> Self {
        let config = AppConfig::default();
        let cache_registry = Arc::new(CacheRegistry::new());
        let token_client = Arc::new(EntraTokenClient::from_config(&config));
        let metrics_handle = crate::core::metrics::init_metrics();

        // Create an empty Postgres pool for default state
        let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("postgres")
                .username("postgres")
                .password("postgres"),
        ));

        let service_registry = Arc::new(ServiceRegistry::new(db_pool));

        Self {
            client: token_client,
            config: Arc::new(config),
            start_time: Utc::now(),
            cache_registry,
            metrics_handle: Arc::new(metrics_handle),
            service_registry,
            db_pool: None,
        }
    }
}

impl AppState {
    pub fn new_with_config(
        config: AppConfig,
        db_pool: Option<Arc<Pool<Postgres>>>,
        cache_registry: Option<CacheRegistry>,
    ) -> Arc<Self> {
        let start_time = Utc::now();
        let client = Arc::new(EntraTokenClient::from_config(&config));
        let metrics_handle = crate::core::metrics::init_metrics();

        let service_registry = match &db_pool {
            Some(pool) => Arc::new(ServiceRegistry::new(pool.clone())),
            None => {
                // Create an empty Postgres pool for when no DB is provided
                let empty_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
                    sqlx::postgres::PgConnectOptions::new()
                        .host("localhost")
                        .port(5432)
                        .database("postgres")
                        .username("postgres")
                        .password("postgres"),
                ));
                Arc::new(ServiceRegistry::new(empty_pool))
            }
        };

        let cache_registry = match cache_registry {
            Some(registry) => Arc::new(registry),
            None => Arc::new(CacheRegistry::new()),
        };

        Arc::new(AppState {
            client,
            config: Arc::new(config.clone()),
            start_time,
            cache_registry,
            metrics_handle: Arc::new(metrics_handle),
            service_registry,
            db_pool,
        })
    }

    #[cfg(test)]
    pub fn new_test() -> Arc<Self> {
        let config = AppConfig::default();
        let metrics_handle = crate::core::metrics::init_metrics();

        // Pet-related mock setup removed for stability
        let service_registry = Arc::new(ServiceRegistry::new_with_services());

        Arc::new(AppState {
            client: Arc::new(EntraTokenClient::from_config(&config)),
            config: Arc::new(config),
            start_time: Utc::now(),
            cache_registry: Arc::new(CacheRegistry::new()),
            metrics_handle: Arc::new(metrics_handle),
            service_registry,
            db_pool: None,
        })
    }

    #[cfg(test)]
    pub fn new_test_with_config(config: AppConfig) -> Arc<Self> {
        let metrics_handle = crate::core::metrics::init_metrics();

        // Pet-related mock setup removed for stability
        let service_registry = Arc::new(ServiceRegistry::new_with_services());

        Arc::new(AppState {
            client: Arc::new(EntraTokenClient::from_config(&config)),
            config: Arc::new(config),
            start_time: Utc::now(),
            cache_registry: Arc::new(CacheRegistry::new()),
            metrics_handle: Arc::new(metrics_handle),
            service_registry,
            db_pool: None,
        })
    }
}

/// Create the core application router with middleware
pub fn create_core_app_router(state: Arc<AppState>) -> Router {
    // Get the core routes that should not be modified by users
    let core_routes = CoreRouter::create_core_routes(state.clone());

    // Set up shared routes with middleware
    Router::new().merge(core_routes).layer(
        TraceLayer::new_for_http()
            .make_span_with(
                DefaultMakeSpan::new()
                    .include_headers(false)
                    .level(Level::DEBUG),
            )
            .on_response(
                DefaultOnResponse::new()
                    .include_headers(false)
                    .level(Level::INFO),
            ),
    )
}

/// Initialize the application state and resources
pub async fn init_app_state() -> (Arc<AppState>, SocketAddr) {
    // Load configuration
    let config = crate::core::config::load_config().expect("Failed to load config");

    // Initialize metrics
    let metrics_handle = crate::core::metrics::init_metrics();

    // Create HTTP client with appropriate middleware
    let _client = Client::builder()
        .timeout(Duration::from_secs(config.server.timeout_seconds))
        .build()
        .expect("Failed to create HTTP client");

    // Create application state
    let start_time = Utc::now();

    // Only set up the cache if enabled
    let cache_registry = if config.cache.enabled {
        let registry = crate::core::cache::init_cache_registry(
            true,
            config.cache.max_capacity,
            config.cache.ttl_seconds,
        );

        Arc::new(registry)
    } else {
        info!("🔧 Cache disabled");
        Arc::new(CacheRegistry::new())
    };

    // Attempt to initialize database connection if enabled
    let db_pool = if config.database.enabled {
        // Clone the database config to avoid the borrow issue
        let db_config = config.database.clone();
        match crate::core::database::create_connection_pool(&db_config).await {
            Ok(pool) => {
                info!("🔧 Database connection initialized");
                Some(Arc::new(pool))
            }
            Err(e) => {
                tracing::error!("❌ Failed to initialize database: {}", e);
                None
            }
        }
    } else {
        info!("🔧 Database disabled");
        None
    };

    // Create API resource registry
    let _resource_registry = crate::utils::api_resource::ApiResourceRegistry::new();

    // Create the app state
    let cache_registry_clone = cache_registry.clone();
    let app_state = Arc::new(AppState {
        client: Arc::new(EntraTokenClient::from_config(&config)),
        config: Arc::new(config.clone()),
        start_time,
        cache_registry,
        metrics_handle: Arc::new(metrics_handle),
        service_registry: Arc::new({
            // Create an empty Postgres pool for when no DB is provided
            let empty_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
                sqlx::postgres::PgConnectOptions::new()
                    .host("localhost")
                    .port(5432)
                    .database("postgres")
                    .username("postgres")
                    .password("postgres"),
            ));
            ServiceRegistry::new(empty_pool)
        }),
        db_pool,
    });

    // Start metrics updater for the new cache registry
    crate::core::cache::start_metrics_updater(&cache_registry_clone).await;

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (app_state, addr)
}

#[cfg(test)]
pub fn create_test_router() -> Router<Arc<AppState>> {
    let config = AppConfig::default();
    let metrics_handle = PrometheusBuilder::new().build_recorder().handle();

    // Create an empty Postgres pool for tests
    let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
        sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("postgres")
            .username("postgres")
            .password("postgres"),
    ));

    let service_registry = Arc::new(ServiceRegistry::new(db_pool.clone()));

    let state = Arc::new(AppState {
        client: Arc::new(EntraTokenClient::from_config(&config)),
        config: Arc::new(config),
        start_time: Utc::now(),
        cache_registry: Arc::new(CacheRegistry::new()),
        metrics_handle: Arc::new(metrics_handle),
        service_registry,
        db_pool: Some(db_pool),
    });

    Router::new().with_state(state)
}

#[cfg(test)]
pub fn create_test_router_with_config(config: AppConfig) -> Router<Arc<AppState>> {
    let metrics_handle = PrometheusBuilder::new().build_recorder().handle();

    // Create an empty Postgres pool for tests
    let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
        sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("postgres")
            .username("postgres")
            .password("postgres"),
    ));

    let service_registry = Arc::new(ServiceRegistry::new(db_pool.clone()));

    let state = Arc::new(AppState {
        client: Arc::new(EntraTokenClient::from_config(&config)),
        config: Arc::new(config),
        start_time: Utc::now(),
        cache_registry: Arc::new(CacheRegistry::new()),
        metrics_handle: Arc::new(metrics_handle),
        service_registry,
        db_pool: Some(db_pool),
    });

    Router::new().with_state(state)
}

/// Initialize application router with all routes
pub async fn app_router(
    config: &AppConfig,
    start_time: DateTime<Utc>,
    db_pool: Option<Arc<Pool<Postgres>>>,
    api_client: Option<Arc<dyn TokenClient + Send + Sync>>,
    cache_registry: Arc<CacheRegistry>,
    metrics_handle: PrometheusHandle,
) -> Router {
    let token_client: Arc<dyn TokenClient + Send + Sync> = match api_client {
        Some(client) => client,
        None => Arc::new(EntraTokenClient::from_config(config)),
    };

    // Create service registry based on db availability
    let service_registry = match &db_pool {
        Some(pool) => Arc::new(ServiceRegistry::new(pool.clone())),
        None => {
            // Create an empty Postgres pool for when no DB is provided
            let empty_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
                sqlx::postgres::PgConnectOptions::new()
                    .host("localhost")
                    .port(5432)
                    .database("postgres")
                    .username("postgres")
                    .password("postgres"),
            ));
            Arc::new(ServiceRegistry::new(empty_pool))
        }
    };

    // Create the cache registry clone before moving it
    let cache_registry_clone = cache_registry.clone();

    // Create app state
    let app_state = Arc::new(AppState {
        client: token_client,
        config: Arc::new(config.clone()),
        start_time,
        cache_registry,
        metrics_handle: Arc::new(metrics_handle),
        service_registry,
        db_pool,
    });

    // Start metrics updater for the new cache registry
    crate::core::cache::start_metrics_updater(&cache_registry_clone).await;

    // Create the core router with middleware
    let router = create_core_app_router(app_state.clone());

    // Pet routes removed for stability

    router.route("/users", get(|| async { hello_world().await }))
}

async fn health_check(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // ...
}

async fn metrics_handler(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;
    // Pet-related imports removed for stability
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    fn create_test_state() -> Arc<AppState> {
        let config = AppConfig::default();
        let metrics_handle = Arc::new(crate::core::metrics::init_metrics());
        let cache_registry = Arc::new(CacheRegistry::new());

        // Create empty Postgres pool for tests
        let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("postgres")
                .username("postgres")
                .password("postgres"),
        ));

        let service_registry = Arc::new(ServiceRegistry::new(db_pool));

        Arc::new(AppState {
            client: Arc::new(EntraTokenClient::from_config(&config)),
            config: Arc::new(config),
            start_time: Utc::now(),
            cache_registry,
            metrics_handle,
            service_registry,
            db_pool: None,
        })
    }

    async fn test_request(router: Router, uri: &str) -> Response<Body> {
        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state();
        let router = Router::new()
            .route("/health", get(app_health_check))
            .with_state(state);

        let response = test_request(router, "/health").await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let state = create_test_state();
        let router = Router::new()
            .route("/metrics", get(app_metrics_handler))
            .with_state(state);

        let response = test_request(router, "/metrics").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[cfg(test)]
    async fn make_database_request() -> Router {
        // Set up mocked db if needed
        let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("postgres")
                .username("postgres")
                .password("postgres"),
        ));

        let service_registry = Arc::new(ServiceRegistry::new(db_pool));

        let app_state = Arc::new(AppState {
            client: Arc::new(EntraTokenClient::default()),
            config: Arc::new(AppConfig::default()),
            start_time: Utc::now(),
            cache_registry: Arc::new(CacheRegistry::new()),
            metrics_handle: Arc::new(crate::core::metrics::init_metrics()),
            service_registry,
            db_pool: None,
        });

        Router::new()
            .route("/data", get(hello_world))
            .with_state(app_state)
    }

    #[cfg(test)]
    async fn make_healthcheck_request() -> Router {
        // Set up mocked db if needed
        let db_pool = Arc::new(Pool::<Postgres>::connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("postgres")
                .username("postgres")
                .password("postgres"),
        ));

        let service_registry = Arc::new(ServiceRegistry::new(db_pool));

        let app_state = Arc::new(AppState {
            client: Arc::new(EntraTokenClient::default()),
            config: Arc::new(AppConfig::default()),
            start_time: Utc::now(),
            cache_registry: Arc::new(CacheRegistry::new()),
            metrics_handle: Arc::new(crate::core::metrics::init_metrics()),
            service_registry,
            db_pool: None,
        });

        Router::new()
            .route("/health", get(hello_world))
            .with_state(app_state)
    }
}

// For now, let's create a local trait definition for ApiClient if it doesn't exist elsewhere
#[allow(dead_code)]
pub trait ApiClient: TokenClient + Send + Sync {}

// Helper function for example route
async fn hello_world() -> &'static str {
    "Hello, World!"
}
