use crate::app::AppState;
use crate::error::AppError;
use crate::models::{Data, LoggableResponse};
use crate::utils::api_logger;
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

    // Get configured fields to log
    let log_fields = &state.config.logging.response_fields.cat_fact_fields;

    // Use the logging utility to handle the API call
    let api_name = "Cat Facts";
    let entity_type = "Fact";

    // Create a closure that returns the actual request future
    let fetch_fn = || async { state.client.get(fact_url).send().await };

    // Make the API call with consistent logging
    let data = api_logger::fetch_and_log_api_call(
        api_name,
        fact_url,
        fetch_fn,
        entity_type,
        "current", // No specific ID for cat facts
        log_fields,
    )
    .await?;

    Ok(Json(data))
}
