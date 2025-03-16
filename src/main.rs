mod petstore_api;

use crate::petstore_api::models::pet;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::Request,
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::get,
};

use config::{Config as ConfigSource, ConfigError, Environment, File};
use dotenvy::dotenv;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use moka::future::Cache;
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{Level, debug, error, info, warn};
use tracing_subscriber::FmtSubscriber;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

// Custom error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("HTTP client error: {0}")]
    ClientError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

// Implement conversion to HTTP response for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ClientError(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            AppError::ConfigError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::IoError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        // Increment error counter
        counter!("api.errors", "status" => status.as_u16().to_string(), "type" => format!("{:?}", status.canonical_reason()));

        // Log the error
        error!("{}: {}", status, error_message);

        // Return the HTTP response
        (
            status,
            Json(ErrorResponse {
                error: error_message,
            }),
        )
            .into_response()
    }
}

// Configuration structure
#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    server: ServerConfig,
    api: ApiConfig,
    app: ApplicationConfig,
    cache: CacheConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize, Clone)]
struct ApiConfig {
    cat_fact_url: String,
    petstore_url: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ApplicationConfig {
    name: String,
    version: String,
    log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CacheConfig {
    enabled: bool,
    ttl_seconds: u64,
    max_capacity: u64,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        let _ = dotenv();

        let config = ConfigSource::builder()
            // Start with default settings
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("api.cat_fact_url", "https://catfact.ninja/fact")?
            .set_default("api.petstore_url", "https://petstore3.swagger.io/api/v3")?
            .set_default("app.name", "Petstore API Server")?
            .set_default("app.version", "1.0.0")?
            .set_default("app.log_level", "info")?
            .set_default("cache.enabled", true)?
            .set_default("cache.ttl_seconds", 300)? // 5 minutes
            .set_default("cache.max_capacity", 1000)?
            // Add config file
            .add_source(File::with_name("config").required(false))
            // Add environment variables with prefix
            .add_source(
                Environment::with_prefix("APP")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .add_source(
                Environment::with_prefix("SERVER")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .add_source(
                Environment::with_prefix("API")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .add_source(
                Environment::with_prefix("CACHE")
                    .separator("_")
                    .prefix_separator("_")
                    .keep_prefix(false),
            )
            .build()?;

        // Deserialize the configuration
        config.try_deserialize()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

// Cached pet data
type PetCache = Cache<i64, pet::Pet>;

// App State to carry configuration and shared resources
struct AppState {
    client: Client,
    config: Arc<AppConfig>,
    start_time: SystemTime,
    pet_cache: Option<PetCache>,
    metrics_handle: PrometheusHandle,
}

// Define schemas for generated models that need to be used in OpenAPI docs
// This is a workaround for the Pet model not being recognized as implementing ToSchema
#[derive(ToSchema)]
#[schema(as = pet::Pet)]
struct PetSchema {
    #[schema(example = 10)]
    id: Option<i64>,
    #[schema(example = "doggie")]
    name: String,
    #[schema(nullable, example = json!({"id": 1, "name": "Dogs"}))]
    category: Option<Box<CategorySchema>>,
    #[schema(example = json!(["url1", "url2"]))]
    photo_urls: Vec<String>,
    #[schema(nullable, example = json!([{"id": 1, "name": "tag1"}]))]
    tags: Option<Vec<TagSchema>>,
    #[schema(nullable, example = "available")]
    status: Option<StatusSchema>,
}

#[derive(ToSchema)]
#[schema(as = pet::Category)]
struct CategorySchema {
    #[schema(example = 1)]
    id: Option<i64>,
    #[schema(example = "Dogs")]
    name: Option<String>,
}

#[derive(ToSchema)]
#[schema(as = pet::Tag)]
struct TagSchema {
    #[schema(example = 1)]
    id: Option<i64>,
    #[schema(example = "tag1")]
    name: Option<String>,
}

#[derive(ToSchema)]
enum StatusSchema {
    Available,
    Pending,
    Sold,
}

// MODEL ENHANCEMENT LAYER
// This layer allows us to enhance the generated models with custom behavior
// and documentation while still using them directly with Utoipa

// Complete OpenAPI definition
#[derive(OpenApi)]
#[openapi(
    paths(get_pet_by_id, get_data, health_check, metrics),
    info(
        title = "Pet Store API",
        version = "1.0",
        description = "A sample Pet Store server with OpenAPI model integration",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    components(
        schemas(PetSchema, CategorySchema, TagSchema, StatusSchema, Data, ErrorResponse, HealthCheckResponse, MetricsResponse)
    ),
    tags(
        (name = "pets", description = "Pet operations"),
        (name = "data", description = "Data operations"),
        (name = "system", description = "System operations")
    )
)]
struct ApiDoc;

// Helper to convert Status enum to string
fn status_to_string(status: &pet::Status) -> String {
    match status {
        pet::Status::Available => "available".to_string(),
        pet::Status::Pending => "pending".to_string(),
        pet::Status::Sold => "sold".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration
    let config = Arc::new(AppConfig::new()?);

    // Initialize tracing for better logging
    let log_level = match config.app.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            warn!(
                "Invalid log level: {}, defaulting to INFO",
                config.app.log_level
            );
            Level::INFO
        }
    };

    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Initializing {} v{}", config.app.name, config.app.version);
    info!("Log level set to: {}", log_level);

    // Initialize HTTP client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

    // Initialize cache if enabled
    let pet_cache = if config.cache.enabled {
        info!(
            "Initializing cache with TTL: {}s, capacity: {} items",
            config.cache.ttl_seconds, config.cache.max_capacity
        );
        Some(
            Cache::builder()
                .time_to_live(Duration::from_secs(config.cache.ttl_seconds))
                .max_capacity(config.cache.max_capacity)
                .build(),
        )
    } else {
        info!("Cache is disabled");
        None
    };

    // Initialize metrics
    let metrics_handle = PrometheusBuilder::new().install_recorder().map_err(|e| {
        AppError::InternalError(format!("Failed to install Prometheus recorder: {}", e))
    })?;

    // Set up application state
    let state = Arc::new(AppState {
        client,
        config: config.clone(),
        start_time: SystemTime::now(),
        pet_cache,
        metrics_handle,
    });

    // Register some initial metrics
    gauge!("app.start_time").set(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64,
    );
    gauge!("app.uptime_seconds").set(0.0);

    // Set up a background task to update metrics
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;

            // Update uptime metric
            if let Ok(uptime) = SystemTime::now().duration_since(state_clone.start_time) {
                gauge!("app.uptime_seconds").set(uptime.as_secs() as f64);
            }

            // Update cache metrics if enabled
            if let Some(cache) = &state_clone.pet_cache {
                gauge!("cache.size").set(cache.entry_count() as f64);
            }
        }
    });

    info!("Setting up API routes");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .route("/data", get(get_data))
        .route("/pet/{id}", get(get_pet_by_id))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    info!("Starting server on {}", config.server_addr());
    let listener = TcpListener::bind(&config.server_addr()).await?;
    info!(
        "API documentation available at http://{}/docs",
        config.server_addr()
    );
    info!(
        "Metrics available at http://{}/metrics",
        config.server_addr()
    );

    // Serve the application
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
struct ApiResponse {
    fact: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct Data {
    data: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct HealthCheckResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    cache_enabled: bool,
    cache_stats: Option<CacheStats>,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct CacheStats {
    size: u64,
    hit_rate: f64,
    ttl_seconds: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct MetricsResponse {
    metrics: String,
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = MetricsResponse)
    ),
    tag = "system"
)]
async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics_handle.render();
    metrics.into_response()
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check information", body = HealthCheckResponse)
    ),
    tag = "system"
)]
async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default()
        .as_secs();

    // Record uptime metric
    gauge!("app.uptime_seconds").set(uptime as f64);

    // Get cache stats if enabled
    let cache_stats = if let Some(cache) = &state.pet_cache {
        Some(CacheStats {
            size: cache.entry_count(),
            hit_rate: 0.0, // Not available directly in moka
            ttl_seconds: state.config.cache.ttl_seconds,
        })
    } else {
        None
    };

    Json(HealthCheckResponse {
        status: "ok".to_string(),
        version: state.config.app.version.clone(),
        uptime_seconds: uptime,
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}

#[utoipa::path(
    get,
    path = "/data",
    responses(
        (status = 200, description = "Successfully retrieved random fact", body = Data),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "data"
)]
async fn get_data(State(state): State<Arc<AppState>>) -> Result<Json<Data>, AppError> {
    // Track metrics
    counter!("api.requests", "endpoint" => "data");
    let start = Instant::now();

    let url = &state.config.api.cat_fact_url;
    info!("Fetching data from {}", url);

    let response = state.client.get(url).send().await?;

    // Log response details
    info!("Response status: {}", response.status());

    // Check for non-success status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("API error: {} - {}", status, error_text);
        counter!("api.errors", "endpoint" => "data");
        return Err(AppError::InternalError(format!("API error: {}", status)));
    }

    let api_response = response.json::<ApiResponse>().await?;

    // Record response time
    let duration = start.elapsed();
    histogram!("api.response_time").record(duration.as_millis() as f64);

    info!("Successfully retrieved fact data");
    Ok(Json(Data {
        data: api_response.fact,
    }))
}

async fn get_pet_by_id_request(
    client: &Client,
    base_url: &str,
    pet_id: i64,
    cache: Option<&PetCache>,
) -> Result<pet::Pet, reqwest::Error> {
    // Check cache first if enabled
    if let Some(cache) = cache {
        if let Some(pet) = cache.get(&pet_id).await {
            debug!("Cache hit for pet {}", pet_id);
            counter!("cache.hits", "type" => "pet");
            return Ok(pet);
        }
        debug!("Cache miss for pet {}", pet_id);
        counter!("cache.misses", "type" => "pet");
    }

    // Fetch from API if not in cache
    let url = format!("{}/pet/{}", base_url, pet_id);
    info!("Fetching pet data from {}", url);

    let response = client.get(&url).send().await?;

    // Log response details
    info!("Response status: {}", response.status());

    let pet_data = response.json::<pet::Pet>().await?;

    // Store in cache if enabled
    if let Some(cache) = cache {
        cache.insert(pet_id, pet_data.clone()).await;
        debug!("Cached pet {}", pet_id);
    }

    Ok(pet_data)
}

#[utoipa::path(
    get,
    path = "/pet/{id}",
    params(
        ("id" = i64, Path, description = "Pet id to get")
    ),
    responses(
        (status = 200, description = "Pet found successfully", body = PetSchema),
        (status = 404, description = "Pet not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "pets"
)]
async fn get_pet_by_id(
    State(state): State<Arc<AppState>>,
    Path(pet_id): Path<i64>,
) -> Result<Json<pet::Pet>, AppError> {
    // Track metrics
    counter!("api.requests", "endpoint" => "pet_by_id");
    let start = Instant::now();

    // Implement retry logic for better resilience
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 1..=max_retries {
        match get_pet_by_id_request(
            &state.client,
            &state.config.api.petstore_url,
            pet_id,
            state.pet_cache.as_ref(),
        )
        .await
        {
            Ok(pet) => {
                info!(
                    "Successfully retrieved pet {} on attempt {}",
                    pet_id, attempt
                );

                // Record response time
                let duration = start.elapsed();
                histogram!("api.response_time").record(duration.as_millis() as f64);

                return Ok(Json(pet));
            }
            Err(e) => {
                // Check if it's a 404 error
                if let Some(status) = e.status() {
                    if status == StatusCode::NOT_FOUND {
                        counter!("api.errors", "endpoint" => "pet_by_id", "error" => "not_found");
                        return Err(AppError::NotFound(format!(
                            "Pet with id {} not found",
                            pet_id
                        )));
                    }
                }

                warn!(
                    "Failed to retrieve pet {} on attempt {}: {}",
                    pet_id, attempt, e
                );
                counter!("api.retries", "endpoint" => "pet_by_id");
                last_error = Some(e);

                // Only sleep if we're going to retry
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempt)).await;
                }
            }
        }
    }

    // If we get here, all retries have failed
    counter!("api.errors", "endpoint" => "pet_by_id", "error" => "max_retries");
    Err(AppError::ClientError(last_error.unwrap()))
}
