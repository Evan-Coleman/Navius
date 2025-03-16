use axum::{
    Json,
    extract::{Path, State},
};
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{app::AppState, generated_apis::petstore_api::models::Upet};

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
    Path(id): Path<i64>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Upet>, (StatusCode, String)> {
    // Log request
    info!("Fetching pet with ID: {}", id);

    // If cache is enabled, try to get from cache or fetch
    if let Some(cache) = &state.pet_cache {
        match crate::cache::get_or_fetch(cache, id, || async {
            fetch_pet_with_retry(&state, id).await
        })
        .await
        {
            Ok(pet) => {
                info!("Returning pet with ID: {}", id);
                return Ok(Json(pet));
            }
            Err(err) => {
                // Error is already logged in fetch_pet_with_retry
                if err.contains("not found") {
                    return Err((StatusCode::NOT_FOUND, err));
                } else {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, err));
                }
            }
        }
    } else {
        // No cache, fetch directly
        match fetch_pet_with_retry(&state, id).await {
            Ok(pet) => {
                info!("Returning pet with ID: {}", id);
                return Ok(Json(pet));
            }
            Err(err) => {
                // Error is already logged in fetch_pet_with_retry
                if err.contains("not found") {
                    return Err((StatusCode::NOT_FOUND, err));
                } else {
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, err));
                }
            }
        }
    }
}

async fn fetch_pet_with_retry(state: &Arc<AppState>, id: i64) -> Result<Upet, String> {
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
                if err.contains("not found (HTTP 404)") {
                    warn!("Pet not found: {}", err);
                    return Err(err);
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

    Err(last_error.unwrap_or_else(|| "Unknown error fetching pet".to_string()))
}

async fn fetch_pet(state: &Arc<AppState>, id: i64) -> Result<Upet, String> {
    let url = format!("{}/pet/{}", state.config.petstore_api_url(), id);

    let response = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let status = response.status();

    if status == StatusCode::NOT_FOUND {
        return Err(format!(
            "Pet with ID {} not found (HTTP {})",
            id,
            status.as_u16()
        ));
    }

    if !status.is_success() {
        return Err(format!(
            "API returned error status: HTTP {}",
            status.as_u16()
        ));
    }

    response
        .json::<Upet>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
