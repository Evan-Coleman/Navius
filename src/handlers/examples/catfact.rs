use std::sync::Arc;
use tracing::info;

use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    app::AppState,
    error::{AppError, Result},
    models::Data,
    utils::{
        api_logger,
        api_resource::{ApiHandlerOptions, ApiResource, create_api_handler},
    },
};

// Implement the ApiResource trait for our Data model
impl ApiResource for Data {
    // Cat facts don't have real IDs, so we'll use a dummy string type
    type Id = String;

    fn resource_type() -> &'static str {
        "catfact"
    }

    fn api_name() -> &'static str {
        "Cat Facts"
    }
}

/// A simple handler for the cat fact API
pub async fn fetch_catfact_handler(State(state): State<Arc<AppState>>) -> Result<Json<Data>> {
    // Use a dummy ID since cat facts don't have IDs
    let dummy_id = "current".to_string();

    // Define the fetch function inline to avoid lifetime issues
    let fetch_fn = move |state: &Arc<AppState>,
                         _id: String|
          -> futures::future::BoxFuture<'static, Result<Data>> {
        let state = state.clone(); // Clone the state to avoid lifetime issues
        Box::pin(async move {
            // Get fact URL from config
            let fact_url = &state.config.api.cat_fact_url;

            // Create a closure that returns the actual request future
            let fetch_fn = || async { state.client.get(fact_url).send().await };

            // Make the API call
            api_logger::api_call(
                "Cat Facts",
                fact_url,
                fetch_fn,
                "Fact",
                "current", // No specific ID for cat facts
            )
            .await
        })
    };

    // Create an API handler with enhanced options
    let handler = create_api_handler(
        fetch_fn,
        ApiHandlerOptions {
            use_cache: true,
            use_retries: true,
            max_retry_attempts: 2,   // Less retries for cat facts
            cache_ttl_seconds: 3600, // 1 hour - facts don't change often
            detailed_logging: true,
        },
    );

    // Execute the handler with proper path extraction
    handler(State(state), Path(dummy_id)).await
}

/// The core function that does the actual API call
async fn fetch_catfact(state: &Arc<AppState>, _id: String) -> Result<Data> {
    // Get fact URL from config
    let fact_url = &state.config.api.cat_fact_url;

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(fact_url).send().await };

    // Make the API call
    api_logger::api_call(
        "Cat Facts",
        fact_url,
        fetch_fn,
        "Fact",
        "current", // No specific ID for cat facts
    )
    .await
}
