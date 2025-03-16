// Modules with better organization
mod app {
    pub use crate::app::router::*;
    pub mod router;
}

mod cache {
    pub use crate::cache::cache_manager::*;
    pub mod cache_manager;
}

mod config {
    pub use crate::config::app_config::*;
    pub mod app_config;
}

mod error {
    pub use crate::error::error_types::*;
    pub mod error_types;
}

mod metrics {
    pub use crate::metrics::metrics_service::*;
    pub mod metrics_service;
}

mod handlers;
mod models;
mod petstore_api;

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Initialize the application
    let (mut app, addr) = app::init().await;

    // Add Swagger UI
    app = app.merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", app::ApiDoc::openapi()));

    // Start the server
    info!("Starting server on http://{}", addr);
    info!("API documentation available at http://{}/docs", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .expect("Failed to start server");
}
