use axum::{Json, extract::State};
use chrono::Utc;
use http::StatusCode;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::{
    app::AppState,
    error::{AppError, Result},
    models::Data,
};

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
    let fact_url = &state.config.api.cat_fact_url;

    // Try using the token client for secure downstream API calls
    if let Some(token_client) = &state.token_client {
        info!("Using token client for authenticated requests");

        // This is an example of how you would use the token client
        // with a real protected API (replace with your actual API)
        let scope = std::env::var("DOWNSTREAM_API_SCOPE")
            .unwrap_or_else(|_| "api://your-downstream-api/.default".to_string());

        match token_client.get_token(&scope).await {
            Ok(token) => {
                debug!("Successfully acquired token for downstream API");
                // Use token for downstream requests (this is just for demonstration)

                // In a real scenario, you might do:
                // let auth_client = token_client.create_client(&scope).await.map_err(...)?;
                // let response = auth_client.get(downstream_url).send().await?;
            }
            Err(e) => {
                error!("Failed to acquire token: {}", e);
                // Fall back to unauthenticated request or return error
            }
        }
    }

    // This is the original implementation (using unauthenticated request)
    let response = state.client.get(fact_url).send().await?;

    if response.status().is_success() {
        let mut data: Data = response.json().await?;
        data.timestamp = Utc::now().to_string();
        Ok(Json(data))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR.into())
    }
}
