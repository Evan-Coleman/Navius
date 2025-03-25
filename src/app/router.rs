use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::core::router::app_router::AppState;
use config::Config;

pub fn create_router(
    _config: Config,
    // Database connection parameter removed for stability
) -> Router<Arc<AppState>> {
    // Create a basic app state with default settings
    let app_state = Arc::new(AppState::default());

    // Define routes
    Router::new().with_state(app_state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics() -> String {
    let handle = crate::core::metrics::init_metrics();
    crate::core::metrics::metrics_endpoint_handler(&handle).await
}
