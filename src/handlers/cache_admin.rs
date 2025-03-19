use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tracing::info;

use crate::app::AppState;
use crate::cache::cache_manager::{CacheStats, CacheWithTTL};

/// Response for the cache debug endpoint
#[derive(Serialize, Deserialize)]
pub struct CacheDebugResponse {
    pub enabled: bool,
    pub stats: Option<CacheStats>,
    pub entries: BTreeMap<String, String>,
    pub raw_metrics: BTreeMap<String, String>,
}

/// Handler for the cache debug endpoint
///
/// Returns detailed information about the cache for debugging purposes
pub async fn cache_debug(State(state): State<Arc<AppState>>) -> Json<CacheDebugResponse> {
    let mut entries = BTreeMap::new();
    let mut raw_metrics = BTreeMap::new();

    // Get metrics for the cache
    let metrics_text = state.metrics_handle.render();

    // Parse the metrics text to extract cache-related metrics
    // Capture ALL cache-related metrics - use a broader filter
    for line in metrics_text.lines() {
        if line.contains("cache") || line.contains("pet_cache") {
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
        let entry_count = cache.cache.entry_count();

        // For debugging, we can add a custom method or use metrics
        entries.insert("total_count".to_string(), entry_count.to_string());
        entries.insert("ttl_seconds".to_string(), cache.ttl_seconds.to_string());

        // Force an update of the cache size metric to ensure it's current
        let current_size = cache.cache.entry_count();
        entries.insert("current_size".to_string(), current_size.to_string());

        // Add active entries counter
        let active_entries = cache
            .active_entries
            .load(std::sync::atomic::Ordering::Relaxed);
        entries.insert("active_entries".to_string(), active_entries.to_string());

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
