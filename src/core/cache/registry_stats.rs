use std::collections::HashMap;

use super::cache_manager::{CacheRegistry, CacheStats, get_cache_stats_with_metrics};
use crate::metrics::metrics_service::try_record_metrics;

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
