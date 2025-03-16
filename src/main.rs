mod petstore_api;

use crate::petstore_api::models::pet;
use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};

use config::{Config as ConfigSource, ConfigError, Environment, File};
use dotenvy::dotenv;
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::SystemTime;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{Level, error, info, warn};
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
            .build()?;

        // Deserialize the configuration
        config.try_deserialize()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

// App State to carry configuration and shared resources
struct AppState {
    client: Client,
    config: Arc<AppConfig>,
    start_time: SystemTime,
}

// MODEL ENHANCEMENT LAYER
// This layer allows us to enhance the generated models with custom behavior
// and documentation while still using them directly with Utoipa

// A simpler OpenAPI definition to make it easier to debug
#[derive(OpenApi)]
#[openapi(
    paths(get_pet_by_id, get_data, health_check),
    info(
        title = "Pet Store API",
        version = "1.0",
        description = "A sample Pet Store server with OpenAPI model integration",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
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

    let client = Client::new();

    let state = Arc::new(AppState {
        client,
        config: config.clone(),
        start_time: SystemTime::now(),
    });

    info!("Setting up API routes");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_check))
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

    Json(HealthCheckResponse {
        status: "ok".to_string(),
        version: state.config.app.version.clone(),
        uptime_seconds: uptime,
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
    let url = &state.config.api.cat_fact_url;
    info!("Fetching data from {}", url);

    let response = state.client.get(url).send().await?;
    let api_response = response.json::<ApiResponse>().await?;

    info!("Successfully retrieved fact data");
    Ok(Json(Data {
        data: api_response.fact,
    }))
}

async fn get_pet_by_id_request(
    client: &Client,
    base_url: &str,
    pet_id: i64,
) -> Result<pet::Pet, reqwest::Error> {
    let url = format!("{}/pet/{}", base_url, pet_id);
    info!("Fetching pet data from {}", url);
    let response = client.get(&url).send().await?.json::<pet::Pet>().await?;
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/pet/:id",
    params(
        ("id" = i64, Path, description = "Pet id to get")
    ),
    responses(
        (status = 200, description = "Pet found successfully", body = pet::Pet),
        (status = 404, description = "Pet not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "pets"
)]
async fn get_pet_by_id(
    State(state): State<Arc<AppState>>,
    Path(pet_id): Path<i64>,
) -> Result<Json<pet::Pet>, AppError> {
    match get_pet_by_id_request(&state.client, &state.config.api.petstore_url, pet_id).await {
        Ok(pet) => {
            info!("Successfully retrieved pet {}", pet_id);
            Ok(Json(pet))
        }
        Err(e) => {
            // Check if it's a 404 error
            if let Some(status) = e.status() {
                if status == StatusCode::NOT_FOUND {
                    return Err(AppError::NotFound(format!(
                        "Pet with id {} not found",
                        pet_id
                    )));
                }
            }
            Err(AppError::ClientError(e))
        }
    }
}
