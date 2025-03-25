use axum::{
    body::{self, Body},
    extract::{ConnectInfo, Extension, Request as AxumRequest, State},
    http::{HeaderMap, Method, Request, Response, StatusCode, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Response as AxumResponse},
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

use crate::{
    app::{
        api::handlers::{
            health_check::health_check,
            metrics_handler::metrics_handler,
            pet_handler::{create_pet, delete_pet, get_pet_by_id, get_pets, update_pet},
        },
        database::repositories::pet_repository::{PetRepository, PgPetRepository},
        services::pet_service::{MockPetRepository, PetService},
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

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub client: Arc<EntraTokenClient>,
    pub config: Arc<AppConfig>,
    pub start_time: DateTime<Utc>,
    pub cache_registry: Arc<CacheRegistry>,
    pub metrics_handle: Arc<PrometheusHandle>,
    pub service_registry: Arc<ServiceRegistry>,
    pub db_pool: Option<Arc<Pool<Postgres>>>,
}

impl AppState {
    pub fn new(
        client: Arc<EntraTokenClient>,
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
        let mock_repository = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

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

        let pet_repository: Arc<dyn PetRepository> = match &db_pool {
            Some(pool) => Arc::new(PgPetRepository::new(pool.clone())),
            None => Arc::new(MockPetRepository::default()),
        };
        let pet_service = Arc::new(PetService::new(pet_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

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

    pub fn new_test() -> Arc<Self> {
        let config = AppConfig::default();
        let metrics_handle = crate::core::metrics::init_metrics();
        let mock_repository = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

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

    pub fn new_test_with_config(config: AppConfig) -> Arc<Self> {
        let metrics_handle = crate::core::metrics::init_metrics();
        let mock_repository = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

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
        match crate::core::init_database(db_config).await {
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
    let resource_registry = crate::utils::api_resource::ApiResourceRegistry::new();

    // Create the app state
    let pet_repository: Arc<dyn PetRepository> = match &db_pool {
        Some(pool) => Arc::new(PgPetRepository::new(pool.clone())),
        None => Arc::new(MockPetRepository::default()),
    };
    let pet_service = Arc::new(PetService::new(pet_repository));
    let service_registry = Arc::new(ServiceRegistry::new(pet_service));

    let state = Arc::new(AppState {
        client: Arc::new(EntraTokenClient::from_config(&config)),
        config: Arc::new(config.clone()),
        start_time,
        cache_registry,
        metrics_handle: Arc::new(metrics_handle),
        service_registry,
        db_pool,
    });

    // Start metrics updater for the new cache registry
    crate::core::cache::start_metrics_updater(&cache_registry).await;

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    (state, addr)
}

pub fn create_test_router() -> Router<Arc<AppState>> {
    let config = AppConfig::default();
    let metrics_handle = PrometheusBuilder::new().build_recorder().handle();
    let mock_repository = Arc::new(MockPetRepository::default());
    let pet_service = Arc::new(PetService::new(mock_repository));
    let service_registry = Arc::new(ServiceRegistry::new(pet_service));

    let state = Arc::new(AppState {
        client: Arc::new(EntraTokenClient::from_config(&config)),
        config: Arc::new(config),
        start_time: Utc::now(),
        cache_registry: Arc::new(CacheRegistry::new()),
        metrics_handle: Arc::new(metrics_handle),
        service_registry,
        db_pool: None,
    });

    Router::new().with_state(state)
}

pub fn create_test_router_with_config(config: AppConfig) -> Router<Arc<AppState>> {
    let metrics_handle = PrometheusBuilder::new().build_recorder().handle();
    let mock_repository = Arc::new(MockPetRepository::default());
    let pet_service = Arc::new(PetService::new(mock_repository));
    let service_registry = Arc::new(ServiceRegistry::new(pet_service));

    let state = Arc::new(AppState {
        client: Arc::new(EntraTokenClient::from_config(&config)),
        config: Arc::new(config),
        start_time: Utc::now(),
        cache_registry: Arc::new(CacheRegistry::new()),
        metrics_handle: Arc::new(metrics_handle),
        service_registry,
        db_pool: None,
    });

    Router::new().with_state(state)
}

pub async fn app_router(
    config: &AppConfig,
    start_time: DateTime<Utc>,
    db_pool: Option<Arc<Arc<dyn DatabaseConnection>>>,
    api_client: Option<Arc<dyn ApiClient>>,
    cache_registry: Arc<CacheRegistry>,
    metrics_handle: Arc<PrometheusHandle>,
) -> Router {
    let app_state = Arc::new(AppState {
        start_time,
        config: config.clone(),
        client: api_client,
        cache_registry: cache_registry.clone(),
        metrics_handle: metrics_handle.clone(),
        service_registry: Arc::new(ServiceRegistry::new()),
        db_pool: db_pool.clone(),
    });

    // Create a pet repository based on the database connection
    let pet_repo: Arc<dyn pet_repository::PetRepository + Send + Sync> = match &db_pool {
        Some(conn) => {
            // If we have a database connection, check if it's a Postgres connection
            // and create a PgPetRepository. Otherwise, use a mock repository.
            if let Some(pool) = conn.downcast_pg_pool() {
                // Create PgPetRepository with the Postgres pool
                Arc::new(PgPetRepository::new(pool.clone()))
            } else {
                // Fallback to mock repository if not a Postgres connection
                Arc::new(MockPetRepository::new())
            }
        }
        None => Arc::new(MockPetRepository::new()),
    };

    let pet_service = Arc::new(PetService::new(pet_repo));
    let service_registry = Arc::new(ServiceRegistry::new(pet_service));

    let state = Arc::new(AppState {
        client: app_state.client.clone(),
        config: app_state.config.clone(),
        start_time: app_state.start_time,
        cache_registry: app_state.cache_registry.clone(),
        metrics_handle: app_state.metrics_handle.clone(),
        service_registry,
        db_pool: db_pool.clone(),
    });

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/pets", get(get_pets).post(create_pet))
        .route(
            "/pets/:id",
            get(get_pet_by_id).put(update_pet).delete(delete_pet),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // ...
}

async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::pet_service::{MockPetRepository, PetService};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use tower::ServiceExt;

    fn create_test_state() -> Arc<AppState> {
        let config = Arc::new(AppConfig::default());
        let metrics_handle = Arc::new(init_metrics());
        let cache_registry = Arc::new(CacheRegistry::new());
        let client = Arc::new(EntraTokenClient::from_config(&config));
        let mock_repository: Arc<dyn PetRepository> = Arc::new(MockPetRepository::default());
        let pet_service = Arc::new(PetService::new(mock_repository));
        let service_registry = Arc::new(ServiceRegistry::new(pet_service));

        Arc::new(AppState {
            client,
            config,
            start_time: Utc::now(),
            cache_registry,
            metrics_handle,
            service_registry,
            db_pool: None,
        })
    }

    async fn test_request(router: Router, uri: &str) -> Response {
        let request = Request::builder().uri(uri).body(Body::empty()).unwrap();

        router.oneshot(request).await.unwrap()
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state();
        let router = app_router(state).await;
        let response = test_request(router, "/health").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        let state = create_test_state();
        let router = app_router(state).await;
        let response = test_request(router, "/metrics").await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
