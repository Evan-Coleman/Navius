use axum::{Json, extract::State};
use std::sync::Arc;

use crate::app::AppState;
use crate::models::HealthCheckResponse;

/// Handler for the health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check response", body = HealthCheckResponse)
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthCheckResponse> {
    // Calculate uptime
    let uptime_secs = state.start_time.elapsed().map(|d| d.as_secs()).unwrap_or(0);

    // Get cache stats if enabled
    let cache_stats = state
        .pet_cache
        .as_ref()
        .map(|cache| crate::cache::get_cache_stats(cache));

    // Return health check response
    Json(HealthCheckResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime_secs,
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}
