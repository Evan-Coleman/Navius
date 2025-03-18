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
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default();

    // Get cache stats if available
    let cache_stats = if let Some(cache) = &state.pet_cache {
        // Fix: access cache stats properly
        Some(CacheStats {
            cache_hits: cache.stats().hit_count(),
            cache_misses: cache.stats().miss_count(),
            cache_size: cache.entry_count() as u64,
            uptime_seconds: uptime.as_secs(),
        })
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
        // Fix: use uptime_seconds field instead of uptime
        uptime_seconds: uptime.as_secs(),
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}
