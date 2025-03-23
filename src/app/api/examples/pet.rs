use axum::{
    Json,
    extract::{Path, State},
};
use std::sync::Arc;
use tracing::info;

use crate::core::{
    error::{AppError, Result},
    router::AppState,
    utils::{
        api_logger,
        api_resource::{ApiHandlerOptions, ApiResource, create_api_handler},
    },
};
use crate::generated_apis::petstore_api::models::Upet;

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

    // Log request for easier tracing
    info!("🔍 Pet lookup requested for ID: {}", id);

    // Define the fetch function inline to avoid lifetime issues
    let fetch_fn = move |state: &Arc<AppState>,
                         id: i64|
          -> futures::future::BoxFuture<'static, Result<Upet>> {
        let state = state.clone(); // Clone the state to avoid lifetime issues

        // Log external API call
        info!("🌐 Calling external Petstore API for pet ID: {}", id);

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
            cache_ttl_seconds: state.config.cache.ttl_seconds, // Use configured TTL instead of hardcoded value
            detailed_logging: true,
        },
    );

    // Execute the handler with proper path extraction
    let result = handler(State(state), Path(id_str)).await;

    // Log the result of the operation with specific source information
    match &result {
        Ok(_) => {
            // Check if it was retrieved from cache using our thread-local
            let from_cache = crate::core::cache::last_fetch_from_cache();
            let source = if from_cache { "from cache" } else { "from API" };
            info!("✅ Successfully retrieved pet ID: {} ({})", id, source)
        }
        Err(e) => info!("❌ Failed to retrieve pet ID: {}, error: {}", id, e),
    }

    result
}

/// The core function that does the actual API call to fetch a pet
async fn fetch_pet(state: &Arc<AppState>, id: i64) -> Result<Upet> {
    let url = format!("{}/pet/{}", state.config.petstore_api_url(), id);

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(&url).send().await };

    // Make the API call using the common logger/handler
    api_logger::api_call("Petstore", &url, fetch_fn, "Pet", id).await
}
