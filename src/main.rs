use rust_backend::app;
use rust_backend::error::error_types::AppError;

use std::process;
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;
use utoipa::OpenApi;
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

    // Add Swagger UI
    app = app.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", app::ApiDoc::openapi()));

    // Start the server
    info!("Starting server on http://{}", addr);
    info!("API documentation available at http://{}/docs", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .await
        .map_err(|e| AppError::InternalError(format!("Server error: {}", e)))?;

    Ok(())
}
