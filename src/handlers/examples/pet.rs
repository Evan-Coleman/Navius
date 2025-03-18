use axum::{
    Json,
    extract::{Path, State},
};
use metrics::counter;
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    app::AppState,
    error::{AppError, Result},
    generated_apis::petstore_api::models::Upet,
    utils::api_logger,
};

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
pub async fn get_pet_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Upet>> {
    // Log the request
    info!("🔍 Getting pet by ID: {}", id);

    // Parse the ID string to i64 for cache lookup
    let id_i64 = id
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest(format!("Invalid pet ID format: {}", id)))?;

    // Check cache first if enabled
    if let Some(cache) = &state.pet_cache {
        // Use get_with_async_loading to handle cache misses properly
        let pet_result = cache.get(&id_i64).await;
        if let Some(pet) = pet_result {
            // Increment cache hit counter
            counter!("pet_cache_hits_total").increment(1);
            api_logger::log_cache_hit("pet", &id);
            return Ok(Json(pet));
        } else {
            // Increment cache miss counter
            counter!("pet_cache_misses_total").increment(1);
            api_logger::log_cache_miss("pet", &id);
        }
    }

    // If not in cache, get from API
    let pet = fetch_pet_with_retry(&state, id_i64).await?;

    // Store in cache if enabled
    if let Some(cache) = &state.pet_cache {
        // Make sure to await the cache insert operation
        cache.insert(id_i64, pet.clone()).await;
        counter!("cache_entries_created").increment(1);
        api_logger::log_cache_store("pet", &id);
    }

    Ok(Json(pet))
}

async fn fetch_pet_with_retry(state: &Arc<AppState>, id: i64) -> Result<Upet> {
    let max_retries = state.config.server.max_retries;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            info!("Retry attempt {} for pet ID: {}", attempt, id);
        }

        match fetch_pet(state, id).await {
            Ok(pet) => return Ok(pet),
            Err(err) => {
                // Don't log attempt number or retry on 404 Not Found errors
                if err.to_string().contains("not found (HTTP 404)") {
                    warn!("❓ Pet not found: {}", err);
                    return Err(AppError::NotFound(format!("Pet with ID {} not found", id)));
                }

                warn!("Attempt {} failed: {}", attempt + 1, err);
                last_error = Some(err);

                // Don't sleep on the last attempt
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        100 * 2u64.pow(attempt as u32),
                    ))
                    .await;
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| AppError::InternalError("Unknown error fetching pet".to_string())))
}

async fn fetch_pet(state: &Arc<AppState>, id: i64) -> Result<Upet> {
    let url = format!("{}/pet/{}", state.config.petstore_api_url(), id);

    // Use the simplified logging utility
    let api_name = "Petstore";
    let entity_type = "Pet";

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(&url).send().await };

    // Make the API call
    api_logger::api_call(api_name, &url, fetch_fn, entity_type, id).await
}
