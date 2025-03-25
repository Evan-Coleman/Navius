use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::core::router::app_router::{AppState, RouterBuilder};
use config::Config;

/// Create an application router with the given configuration
///
/// This function provides a Spring Boot-like developer experience
/// where you can easily configure and extend the application.
pub fn create_router(config: Config) -> Router {
    // Convert from config to AppConfig
    let app_config = crate::core::config::app_config::AppConfig::default();

    // Create a basic app state to use with the router
    let app_state = Arc::new(AppState::default());

    // Create a router using the builder pattern
    let router = RouterBuilder::new()
        .with_config(app_config)
        .with_metrics_enabled(true)
        .with_cors(true)
        .build();

    // Convert the router to use our app state
    router.with_state(())
}

/// Create a Spring Boot-like application with sensible defaults
pub fn create_application() -> RouterBuilder {
    crate::core::router::app_router::create_application()
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics() -> String {
    let handle = crate::core::metrics::init_metrics();
    crate::core::metrics::metrics_endpoint_handler(&handle).await
}
