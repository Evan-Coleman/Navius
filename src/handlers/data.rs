use axum::{Extension, Json, extract::State};
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
pub async fn get_data(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<crate::auth::middleware::EntraClaims>>,
) -> Result<Json<Data>> {
    let fact_url = &state.config.api.cat_fact_url;

    // Log the caller's identity if authenticated
    if let Some(Extension(user)) = claims {
        info!(
            "Request from authenticated user: {} with roles: {:?}",
            user.sub, user.roles
        );
    } else {
        info!("Request from unauthenticated user");
    }

    // Try using the token client for secure downstream API calls
    if let Some(token_client) = &state.token_client {
        info!("Using token client for authenticated downstream API call");

        // Example of calling a downstream API with client credentials
        // In a real app, these would be from configuration
        let downstream_api_scope = std::env::var("DOWNSTREAM_API_SCOPE")
            .unwrap_or_else(|_| "api://your-downstream-api/.default".to_string());

        // Get a dedicated client with auth headers
        match token_client.create_client(&downstream_api_scope).await {
            Ok(auth_client) => {
                // Example of a protected API call - in real code, use the actual URL
                let protected_api_url = "https://protected-api.example.com/data";

                debug!("Making authenticated request to {}", protected_api_url);
                // In a real implementation, you'd use this client instead of the basic one
                // let response = auth_client.get(protected_api_url).send().await?;

                info!("Successfully acquired authenticated client for downstream API");
                // For demonstration, we'll continue with the normal API call
            }
            Err(e) => {
                error!("Failed to create authenticated client: {}", e);
                // Fall back to unauthenticated request
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
