use axum::{Json, extract::State};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::app::AppState;
use crate::cache::cache_manager::{CacheStats, get_cache_stats, get_cache_stats_with_metrics};
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
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default();

    // Get cache stats if available
    let cache_stats = if let Some(cache) = &state.pet_cache {
        // Get metrics for more accurate reporting
        let metrics_text = state.metrics_handle.render();

        // Use the enhanced stats function that includes metrics data
        Some(get_cache_stats_with_metrics(
            cache,
            uptime.as_secs(),
            &metrics_text,
        ))
    } else {
        None
    };

    let auth_status = if state.config.auth.enabled {
        "Authentication enabled"
    } else {
        "Authentication disabled"
    };

    Json(HealthCheckResponse {
        status: format!("healthy - {}", auth_status),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}
