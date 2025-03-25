use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::core::router::app_router::AppState;
use config::Config;

pub fn create_router(
    _config: Config,
    _db_connection: Option<Arc<dyn PgPool>>,
) -> Router<Arc<AppState>> {
    // Create a basic app state with default settings
    let app_state = Arc::new(AppState::default());

    // Define routes
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        // Pet routes removed for stability
        .with_state(app_state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics() -> String {
    let handle = crate::core::metrics::init_metrics();
    crate::core::metrics::metrics_endpoint_handler(&handle).await
}
