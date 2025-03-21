use serde_json::Value;
use std::collections::HashMap;

use crate::cache::providers::CacheProvider;
use crate::core::cache::{CacheRegistry, CacheStats, get_cache_stats_with_metrics};
use crate::core::metrics::try_record_metrics;

/// Get cache statistics with metrics data for all resource types in the registry
pub fn get_all_cache_stats_with_metrics(registry: &CacheRegistry) -> HashMap<String, CacheStats> {
    let mut result = HashMap::new();

    if !registry.enabled {
        return result;
    }

    // Get metrics text for parsing
    let metrics_text = match try_record_metrics() {
        Ok(metrics) => metrics,
        Err(_) => return result,
    };

    // Get all resource types from the registry by extracting them from metrics
    // This is a bit of a workaround since we don't have direct access to the caches
    for line in metrics_text.lines() {
        if line.starts_with("cache_current_size{resource_type=\"") {
            // Extract the resource type from the metric name
            let start = "cache_current_size{resource_type=\"".len();
            let end_pos = line[start..].find('"').unwrap_or(0);
            if end_pos > 0 {
                let resource_type = &line[start..(start + end_pos)];
                // Get stats for this resource type
                if let Some(stats) =
                    get_cache_stats_with_metrics(registry, resource_type, &metrics_text)
                {
                    result.insert(resource_type.to_string(), stats);
                }
            }
        }
    }

    result
}

/// Get cache statistics as JSON from any cache provider implementation
pub async fn get_provider_stats<P: CacheProvider>(provider: &P) -> Result<Value, String> {
    provider.get_stats().await
}
