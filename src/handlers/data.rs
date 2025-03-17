use crate::app::AppState;
use crate::error::AppError;
use crate::models::Data;
use axum::{Json, extract::State};
use std::sync::Arc;
use tracing::{debug, error, info};

type Result<T> = std::result::Result<T, AppError>;

/// Get data from a downstream API
#[utoipa::path(
    get,
    path = "/data",
    responses(
        (status = 200, description = "Data retrieved successfully", body = Data),
        (status = 500, description = "Internal server error", body = String)
    ),
    tag = "data"
)]
pub async fn get_data(State(state): State<Arc<AppState>>) -> Result<Json<Data>> {
    // Log request
    info!("Fetching data from external API");

    let fact_url = &state.config.api.cat_fact_url;

    // Make request to external API
    let response = state
        .client
        .get(fact_url)
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to fetch data: {}", e)))?;

    // Check if response is successful
    if !response.status().is_success() {
        return Err(AppError::ExternalServiceError(format!(
            "API returned error status: {}",
            response.status()
        )));
    }

    // Parse response
    let data = response
        .json::<Data>()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse response: {}", e)))?;

    info!("Successfully fetched data");
    Ok(Json(data))
}
