use rust_backend::app;
use rust_backend::config;
use rust_backend::error::error_types::AppError;
use rust_backend::utils::openapi;

use axum::routing::get;
use std::{fs, path::Path, process};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Print environment variables for debugging
    println!(
        "RUST_BACKEND_TENANT_ID: {}",
        std::env::var("RUST_BACKEND_TENANT_ID").unwrap_or_else(|_| "NOT SET".to_string())
    );
    println!(
        "RUST_BACKEND_CLIENT_ID: {}",
        std::env::var("RUST_BACKEND_CLIENT_ID").unwrap_or_else(|_| "NOT SET".to_string())
    );
    println!(
        "RUST_BACKEND_AUDIENCE: {}",
        std::env::var("RUST_BACKEND_AUDIENCE").unwrap_or_else(|_| "NOT SET".to_string())
    );

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set tracing subscriber: {}", err);
        process::exit(1);
    }

    // Run the application
    if let Err(err) = run_app().await {
        error!("Application error: {}", err);
        process::exit(1);
    }

    Ok(())
}

async fn run_app() -> Result<(), AppError> {
    // Initialize the application
    let (mut app, addr) = app::init().await;

    // Load configuration
    let config = config::app_config::load_config()?;
    let protocol = &config.server.protocol;

    // Ensure the OpenAPI directory exists
    let spec_directory = &config.openapi.spec_directory;
    if !Path::new(spec_directory).exists() {
        info!("Creating OpenAPI spec directory: {}", spec_directory);
        fs::create_dir_all(spec_directory).map_err(|e| {
            AppError::InternalError(format!("Failed to create OpenAPI directory: {}", e))
        })?;
    }

    // Get the application-specific OpenAPI spec path
    let spec_path = config.openapi_spec_path();
    let spec_exists = Path::new(&spec_path).exists();

    if spec_exists {
        info!("Using existing OpenAPI spec from: {}", spec_path);
    } else {
        info!("OpenAPI spec file not found at: {}", spec_path);
        info!("Upload a spec file via the /api/admin/openapi endpoint");
        info!("Default spec filename will be: {}.yaml", config.app.name);
    }

    // Add route to serve the user's OpenAPI spec
    app = app.route("/api-docs/openapi", get(openapi::serve_user_openapi_spec));

    // Add Swagger UI pointing to the user's spec
    app = app.merge(SwaggerUi::new("/docs").url(
        "/api-docs/openapi",
        format!("{} API Documentation", config.app.name),
    ));

    // Start the server
    info!("Starting server on {}://{}", protocol, addr);
    info!(
        "API documentation available at {}://{}/docs",
        protocol, addr
    );

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .await
        .map_err(|e| AppError::InternalError(format!("Server error: {}", e)))?;

    Ok(())
}
