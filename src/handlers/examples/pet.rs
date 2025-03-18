use axum::{
    Json,
    extract::{Path, State},
};
use std::sync::Arc;
use tracing::info;

use crate::{
    app::AppState,
    error::{AppError, Result},
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

/// A simple handler for fetching a pet by ID
pub async fn fetch_pet_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Upet>> {
    // Convert the numeric ID to a string for the path parameter
    let id_str = id.to_string();

    // Define the fetch function inline to avoid lifetime issues
    let fetch_fn = move |state: &Arc<AppState>,
                         id: i64|
          -> futures::future::BoxFuture<'static, Result<Upet>> {
        let state = state.clone(); // Clone the state to avoid lifetime issues
        Box::pin(async move {
            let url = format!("{}/pet/{}", state.config.petstore_api_url(), id);

            // Create a closure that returns the actual request future
            let fetch_fn = || async { state.client.get(&url).send().await };

            // Make the API call using the common logger/handler
            api_logger::api_call("Petstore", &url, fetch_fn, "Pet", id).await
        })
    };

    // Create an API handler with enhanced options
    let handler = create_api_handler(
        fetch_fn,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
            max_retry_attempts: 3,
            cache_ttl_seconds: 600, // 10 minutes for pets
            detailed_logging: true,
        },
    );

    // Execute the handler with proper path extraction
    handler(State(state), Path(id_str)).await
}

/// The core function that does the actual API call to fetch a pet
async fn fetch_pet(state: &Arc<AppState>, id: i64) -> Result<Upet> {
    let url = format!("{}/pet/{}", state.config.petstore_api_url(), id);

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(&url).send().await };

    // Make the API call using the common logger/handler
    api_logger::api_call("Petstore", &url, fetch_fn, "Pet", id).await
}
