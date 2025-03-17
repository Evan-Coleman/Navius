use axum::{Json, extract::State};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::app::AppState;
use crate::models::HealthCheckResponse;

/// Handler for the health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthCheckResponse)
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthCheckResponse> {
    // Calculate uptime
    let uptime_seconds = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs();

    // Get cache stats if enabled
    let cache_stats = state
        .pet_cache
        .as_ref()
        .map(|cache| crate::cache::get_cache_stats(cache));

    let auth_status = if std::env::var("AUTH_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false)
    {
        "Authentication enabled"
    } else {
        "Authentication disabled"
    };

    Json(HealthCheckResponse {
        status: format!("healthy - {}", auth_status),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}
