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

    // Try using the token client for secure downstream API calls
    if let Some(token_client) = &state.token_client {
        info!("Using token client for authenticated downstream API call");

        // Example of calling a downstream API with client credentials
        // In a real app, these would be from configuration
        let downstream_api_scope = std::env::var("DOWNSTREAM_API_SCOPE")
            .unwrap_or_else(|_| "api://your-downstream-api/.default".to_string());

        // Get a dedicated client with auth headers
        if let Ok(_auth_client) = token_client.create_client(&downstream_api_scope).await {
            debug!("Successfully acquired authenticated client for downstream API");
            // In a real implementation, you'd use this client instead
            // let response = _auth_client.get(downstream_url).send().await?;
        } else {
            error!("Failed to create authenticated client");
        }
    }

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
