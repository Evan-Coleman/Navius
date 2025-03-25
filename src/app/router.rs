use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::app::api::pet_core::{create_pet, delete_pet, get_pet, update_pet};
use crate::core::database::connection::DatabaseConnection;
use crate::core::services::Services;
use config::Config;

pub fn create_router(config: Config, db_connection: Option<Arc<dyn DatabaseConnection>>) -> Router {
    let services = Arc::new(Services::new(db_connection));

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .route("/pets/{id}", get(get_pet))
        .route("/pets", post(create_pet))
        .route("/pets/{id}", put(update_pet))
        .route("/pets/{id}", delete(delete_pet))
        .with_state(services)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn metrics() -> String {
    let handle = crate::core::metrics::init_metrics();
    crate::core::metrics_endpoint_handler(&handle).await
}
