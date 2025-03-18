use crate::app::AppState;
use crate::error::AppError;
use crate::models::{Data, LoggableResponse};
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
pub async fn get_catfact(State(state): State<Arc<AppState>>) -> Result<Json<Data>> {
    // Get fact URL from config
    let fact_url = &state.config.api.cat_fact_url;

    // Log request with API details
    info!("🔄 Fetching data from Cat Facts API: {}", fact_url);

    // Make request to external API
    let response = state.client.get(fact_url).send().await.map_err(|e| {
        AppError::ExternalServiceError(format!("Failed to fetch data from {}: {}", fact_url, e))
    })?;

    // Check if response is successful
    if !response.status().is_success() {
        error!(
            "❌ Cat Facts API returned error status: {}",
            response.status()
        );
        return Err(AppError::ExternalServiceError(format!(
            "Cat Facts API returned error status: {}",
            response.status()
        )));
    }

    // Parse response
    let data = response.json::<Data>().await.map_err(|e| {
        AppError::ExternalServiceError(format!("Failed to parse Cat Facts API response: {}", e))
    })?;

    // Get configured fields to log
    let log_fields = &state.config.logging.response_fields.cat_fact_fields;

    // Log success with configured fields
    info!(
        "✅ Successfully fetched data from {}: {}",
        fact_url,
        data.preview_with_fields(log_fields)
    );

    // Full data at debug level
    debug!("📊 Complete response data: {:?}", data);

    Ok(Json(data))
}
