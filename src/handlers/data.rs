use axum::{Json, extract::State};
use std::sync::Arc;
use tracing::{info, warn};

use crate::{app::AppState, models::Data};

/// Handler for the data endpoint
#[utoipa::path(
    get,
    path = "/data",
    responses(
        (status = 200, description = "Data retrieved successfully", body = Data),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "data"
)]
pub async fn get_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Data>, (axum::http::StatusCode, String)> {
    // Log request
    info!("Fetching data from external API");

    // Make request to external API
    let response = state
        .client
        .get(&state.config.api.cat_fact_url)
        .send()
        .await
        .map_err(|e| {
            warn!("Failed to fetch data: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch data: {}", e),
            )
        })?;

    // Check if response is successful
    if !response.status().is_success() {
        warn!("API returned error status: {}", response.status());
        return Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("API returned error status: {}", response.status()),
        ));
    }

    // Parse response
    let data = response.json::<Data>().await.map_err(|e| {
        warn!("Failed to parse response: {}", e);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse response: {}", e),
        )
    })?;

    info!("Successfully fetched data");
    Ok(Json(data))
}
