use axum::Json;
use std::sync::Arc;
use tracing::info;

use crate::{
    app::AppState,
    error::Result,
    generated_apis::petstore_api::models::Upet,
    utils::{
        api_logger,
        api_resource::{ApiHandlerOptions, ApiResource, create_api_handler},
    },
};

// Implement the ApiResource trait for our model
impl ApiResource for Upet {
    type Id = i64;

    fn resource_type() -> &'static str {
        "pet"
    }

    fn api_name() -> &'static str {
        "Petstore"
    }
}

/// Handler for the pet endpoint
#[utoipa::path(
    get,
    path = "/pet/{id}",
    params(
        ("id" = i64, Path, description = "Pet ID to fetch")
    ),
    responses(
        (status = 200, description = "Pet found successfully", body = Upet, content_type = "application/json"),
        (status = 404, description = "Pet not found", body = String, content_type = "text/plain"),
        (status = 500, description = "Internal server error", body = String, content_type = "text/plain")
    ),
    tag = "pets"
)]
pub fn get_pet_by_id() -> impl axum::handler::Handler<(), axum::body::Body> {
    // Create a handler with default options (caching and retries enabled)
    create_api_handler(fetch_pet, ApiHandlerOptions::default())
}

async fn fetch_pet(state: &Arc<AppState>, id: i64) -> Result<Upet> {
    let url = format!("{}/pet/{}", state.config.petstore_api_url(), id);

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(&url).send().await };

    // Make the API call
    api_logger::api_call("Petstore", &url, fetch_fn, "Pet", id).await
}
