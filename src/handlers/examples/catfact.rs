use crate::app::AppState;
use crate::error::AppError;
use crate::models::Data;
use crate::utils::api_logger;
use axum::{Json, extract::State};
use std::sync::Arc;
use tracing::{debug, info};

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

    // Use the simplified logging utility
    let api_name = "Cat Facts";
    let entity_type = "Fact";

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(fact_url).send().await };

    // Make the API call
    let data = api_logger::api_call(
        api_name,
        fact_url,
        fetch_fn,
        entity_type,
        "current", // No specific ID for cat facts
    )
    .await?;

    Ok(Json(data))
}
