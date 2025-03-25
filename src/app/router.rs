use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::app::api::pet_core::{
    create_pet as core_create_pet, delete_pet as core_delete_pet, get_pet as core_get_pet,
    update_pet as core_update_pet,
};
use crate::core::database::PgPool;
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
        .route("/pets/:id", get(core_get_pet))
        .route("/pets", post(core_create_pet))
        .route("/pets/:id", put(core_update_pet))
        .route("/pets/:id", delete(core_delete_pet))
        .with_state(app_state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics() -> String {
    let handle = crate::core::metrics::init_metrics();
    crate::core::metrics::metrics_endpoint_handler(&handle).await
}
