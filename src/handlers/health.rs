use axum::{Extension, Json, extract::State};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::app::AppState;
use crate::auth::middleware::EntraClaims;
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
pub async fn health_check(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<EntraClaims>>,
) -> Json<HealthCheckResponse> {
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

    // Include authentication info in status string
    let status = if let Some(Extension(user)) = claims {
        format!(
            "healthy - Authenticated as {} with roles: {:?}",
            user.sub, user.roles
        )
    } else {
        "healthy".to_string()
    };

    Json(HealthCheckResponse {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}
