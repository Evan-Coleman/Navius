use std::collections::HashMap;

use super::cache_manager::{CacheRegistry, CacheStats};
use crate::core::metrics::metrics_handler::export_metrics;

/// Get cache statistics with metrics data for all resource types in the registry
#[cfg(feature = "metrics")]
pub fn get_all_cache_stats_with_metrics(
    registry: &CacheRegistry,
    handle: &metrics_exporter_prometheus::PrometheusHandle,
) -> HashMap<String, CacheStats> {
    let mut result = HashMap::new();

    if !registry.enabled {
        return result;
    }

    // Get the registry stats directly from the registry
    result = registry.get_all_stats();

    // If no stats from registry method, fallback to parsing metrics output
    if result.is_empty() {
        // Get metrics text for parsing
        let metrics_text = export_metrics(handle);

        // Get all resource types from the registry by extracting them from metrics
        // This is a bit of a workaround since we don't have direct access to the caches
        // We parse the Prometheus metrics output to extract the cache statistics
        for line in metrics_text.lines() {
            // Skip comment lines and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Look for cache metrics
            if line.contains("cache_") {
                // Try to extract the resource type and metric value
                if let Some(resource_type) = extract_resource_type_from_metric(line) {
                    // If we don't already have this resource type, create it
                    let stats = result.entry(resource_type).or_insert_with(|| CacheStats {
                        size: 0,
                        hits: 0,
                        misses: 0,
                        hit_ratio: 0.0,
                    });

                    // Update the stats based on the metric
                    update_stats_from_metric(stats, line);
                }
            }
        }
    }

    result
}

#[cfg(not(feature = "metrics"))]
pub fn get_all_cache_stats_with_metrics(
    registry: &CacheRegistry,
    _handle: &(),
) -> HashMap<String, CacheStats> {
    // When metrics are disabled, just return stats from registry
    registry.get_all_stats()
}

/// Extract the resource type from a metric line
fn extract_resource_type_from_metric(line: &str) -> Option<String> {
    // Example line: cache_hits{resource_type="user"} 42
    if let Some(resource_type_part) = line.find("resource_type=\"") {
        let start_idx = resource_type_part + "resource_type=\"".len();
        if let Some(end_idx) = line[start_idx..].find('\"') {
            return Some(line[start_idx..start_idx + end_idx].to_string());
        }
    }
    None
}

/// Update cache stats based on a metric line
fn update_stats_from_metric(stats: &mut CacheStats, line: &str) {
    // Extract the metric value
    if let Some(value_str) = line.split_whitespace().last() {
        if let Ok(value) = value_str.parse::<u64>() {
            // Update the appropriate stat based on the metric name
            if line.contains("cache_hits") {
                stats.hits = value;
            } else if line.contains("cache_misses") {
                stats.misses = value;
            } else if line.contains("cache_current_size") {
                stats.size = value as usize;
            }

            // Calculate hit ratio
            let total = stats.hits + stats.misses;
            if total > 0 {
                stats.hit_ratio = (stats.hits as f64 / total as f64) * 100.0;
            }
        }
    }
}
