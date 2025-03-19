use axum::{Json, extract::State};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::app::AppState;
use crate::cache::cache_manager::{CacheStats, get_cache_stats, get_cache_stats_with_metrics};
use crate::models::{DependencyStatus, DetailedHealthResponse, HealthCheckResponse};

/// Handler for the simple health check endpoint
///
/// This endpoint is designed for load balancers and monitoring systems.
/// It returns minimal information, usually just whether the service is running.
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<HealthCheckResponse> {
    // Calculate uptime
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default();

    // Simple health check only returns basic status
    Json(HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
    })
}

/// Handler for the detailed health check endpoint
///
/// This endpoint provides comprehensive information about the service's health,
/// including component statuses, configs, and internal metrics.
/// This is typically secured in production environments.
pub async fn detailed_health_check(
    State(state): State<Arc<AppState>>,
) -> Json<DetailedHealthResponse> {
    // Calculate uptime
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap_or_default();

    // Get cache stats if available
    let cache_stats = if let Some(cache) = &state.pet_cache {
        // Get metrics for more accurate reporting
        let metrics_text = state.metrics_handle.render();

        // Use the enhanced stats function that includes metrics data
        Some(get_cache_stats_with_metrics(cache, &metrics_text))
    } else {
        None
    };

    // Gather status of dependencies
    let mut dependencies = Vec::new();

    // Database status (future expansion)
    // let db_status = ...

    // Cache status with structured details
    let mut cache_details = BTreeMap::new();
    cache_details.insert(
        "ttl_seconds".to_string(),
        state.config.cache.ttl_seconds.to_string(),
    );
    cache_details.insert(
        "max_capacity".to_string(),
        state.config.cache.max_capacity.to_string(),
    );

    dependencies.push(DependencyStatus {
        name: "cache".to_string(),
        status: if state.config.cache.enabled {
            "up"
        } else {
            "disabled"
        }
        .to_string(),
        details: Some(cache_details),
    });

    // Auth status with enhanced details
    let auth_details = if state.config.auth.enabled {
        let mut details = BTreeMap::new();
        details.insert("provider".to_string(), "Entra ID".to_string());

        // Include client ID status (not the actual ID)
        details.insert(
            "client_id_status".to_string(),
            if state.config.auth.entra.client_id.is_empty() {
                "not set".to_string()
            } else {
                "configured".to_string()
            },
        );

        // Include tenant ID status
        details.insert(
            "tenant_id_status".to_string(),
            if state.config.auth.entra.tenant_id.is_empty() {
                "not set".to_string()
            } else {
                "configured".to_string()
            },
        );

        // Include audience status
        details.insert(
            "audience_status".to_string(),
            if state.config.auth.entra.audience.is_empty() {
                "not set".to_string()
            } else {
                "configured".to_string()
            },
        );

        // Include role configurations
        details.insert(
            "admin_roles_configured".to_string(),
            (!state.config.auth.entra.admin_roles.is_empty()).to_string(),
        );

        details.insert(
            "read_only_roles_configured".to_string(),
            (!state.config.auth.entra.read_only_roles.is_empty()).to_string(),
        );

        details.insert(
            "full_access_roles_configured".to_string(),
            (!state.config.auth.entra.full_access_roles.is_empty()).to_string(),
        );

        // Include auth debug mode
        details.insert(
            "debug_mode".to_string(),
            state.config.auth.debug.to_string(),
        );

        Some(details)
    } else {
        None
    };

    dependencies.push(DependencyStatus {
        name: "authentication".to_string(),
        status: if state.config.auth.enabled {
            "enabled"
        } else {
            "disabled"
        }
        .to_string(),
        details: auth_details,
    });

    // Build full response
    Json(DetailedHealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime.as_secs(),
        environment: state.config.environment.to_string(),
        dependencies,
        cache_enabled: state.config.cache.enabled,
        cache_stats,
    })
}
