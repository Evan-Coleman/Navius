use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::app::AppState;
use crate::cache::cache_manager::{CacheStats, CacheWithTTL};

/// Response for the cache debug endpoint
#[derive(Serialize, Deserialize)]
pub struct CacheDebugResponse {
    pub enabled: bool,
    pub stats: Option<CacheStats>,
    pub entries: HashMap<String, String>,
    pub raw_metrics: HashMap<String, String>,
}

/// Handler for the cache debug endpoint
///
/// Returns detailed information about the cache for debugging purposes
pub async fn cache_debug(State(state): State<Arc<AppState>>) -> Json<CacheDebugResponse> {
    info!("🔍 Retrieving cache debug information");

    let mut entries = HashMap::new();
    let mut raw_metrics = HashMap::new();

    // Get metrics for the cache
    let metrics_text = state.metrics_handle.render();

    // Parse the metrics text to extract cache-related metrics
    for line in metrics_text.lines() {
        if line.contains("pet_cache") || line.contains("cache_entries") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                raw_metrics.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    // Get cache stats
    let cache_stats = if let Some(cache) = &state.pet_cache {
        // Try to extract the actual entries from the cache
        // This will help us debug if items are actually in the cache
        // Attempt to inspect the cache - this is a best effort
        let entry_count = cache.cache.entry_count();

        // For debugging, we can add a custom method or use metrics
        entries.insert("total_count".to_string(), entry_count.to_string());
        entries.insert("ttl_seconds".to_string(), cache.ttl_seconds.to_string());

        Some(crate::cache::cache_manager::get_cache_stats_with_metrics(
            cache,
            &metrics_text,
        ))
    } else {
        None
    };

    Json(CacheDebugResponse {
        enabled: state.config.cache.enabled,
        stats: cache_stats,
        entries,
        raw_metrics,
    })
}
