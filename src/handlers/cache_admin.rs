use axum::response::{Html, IntoResponse};
use axum::{Json, extract::State};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::SystemTime;
use tracing::info;

use crate::app::AppState;
use crate::cache::cache_manager::CacheStats;

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
    for line in metrics_text.lines() {
        if line.contains("cache") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                raw_metrics.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    // Get cache stats
    let cache_stats = if let Some(registry) = &state.cache_registry {
        // Add basic registry info
        entries.insert("enabled".to_string(), registry.enabled.to_string());
        entries.insert("ttl_seconds".to_string(), registry.ttl_seconds.to_string());
        entries.insert(
            "max_capacity".to_string(),
            registry.max_capacity.to_string(),
        );
        entries.insert(
            "uptime_seconds".to_string(),
            SystemTime::now()
                .duration_since(registry.creation_time)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_else(|_| "0".to_string()),
        );

        // Try to find any resource type from metrics
        let resource_types: Vec<String> = raw_metrics
            .keys()
            .filter_map(|key| {
                if key.contains("resource_type=\"") {
                    // Extract the resource type from the metric name
                    let start = key.find("resource_type=\"").unwrap() + "resource_type=\"".len();
                    let end = key[start..].find('"').unwrap() + start;
                    Some(key[start..end].to_string())
                } else {
                    None
                }
            })
            .collect::<std::collections::HashSet<String>>() // Deduplicate
            .into_iter()
            .collect();

        if !resource_types.is_empty() {
            // Use the first resource type we find
            let resource_type = &resource_types[0];
            entries.insert("found_resource_type".to_string(), resource_type.clone());

            // Get detailed stats for this resource type
            crate::cache::get_cache_stats_with_metrics(registry, resource_type, &metrics_text)
        } else {
            // No registered resource types found in metrics
            entries.insert("found_resource_types".to_string(), "none".to_string());
            entries.insert(
                "resource_registration_status".to_string(),
                "pending".to_string(),
            );

            // Create a placeholder stat with the available information
            Some(CacheStats {
                resource_type: "unknown".to_string(),
                uptime_seconds: SystemTime::now()
                    .duration_since(registry.creation_time)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                size: 0,
                active_entries: 0,
                entries_created: 0,
                hits: 0,
                misses: 0,
                hit_ratio: 0.0,
            })
        }
    } else {
        entries.insert("cache_status".to_string(), "disabled".to_string());
        None
    };

    Json(CacheDebugResponse {
        enabled: state.config.cache.enabled,
        stats: cache_stats,
        entries,
        raw_metrics,
    })
}

/// Handler for displaying cache information
pub async fn show_cache_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut html = String::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Cache Information</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                table { border-collapse: collapse; width: 100%; }
                th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
                th { background-color: #f2f2f2; }
                tr:nth-child(even) { background-color: #f9f9f9; }
            </style>
        </head>
        <body>
            <h1>Cache Information</h1>
            <p>Current time: "#,
    );

    html.push_str(&Utc::now().to_string());
    html.push_str("</p>");

    // Get the server debug info - use basic info from config
    let mut server_debug = Vec::new();
    server_debug.push(format!("Cache enabled: {}", state.config.cache.enabled));

    if let Some(registry) = &state.cache_registry {
        server_debug.push(format!("Cache TTL: {} seconds", registry.ttl_seconds));
        server_debug.push(format!("Cache max capacity: {}", registry.max_capacity));
    }

    // Only include cache-related lines
    for line in server_debug.iter() {
        if line.contains("cache") {
            html.push_str(&format!("<p>{}</p>", line));
        }
    }

    // Get cache registry stats
    if let Some(registry) = &state.cache_registry {
        // Implement a simplified version directly in this handler
        let mut cache_stats = HashMap::new();

        // Get metrics text for parsing
        let metrics_text = state.metrics_handle.render();

        // Extract resource types from metrics
        for line in metrics_text.lines() {
            if line.starts_with("cache_current_size{resource_type=\"") {
                // Extract the resource type from the metric name
                let start = "cache_current_size{resource_type=\"".len();
                let end_pos = line[start..].find('"').unwrap_or(0);
                if end_pos > 0 {
                    let resource_type = &line[start..(start + end_pos)];
                    // Get stats for this resource type
                    if let Some(stats) = crate::cache::get_cache_stats_with_metrics(
                        registry,
                        resource_type,
                        &metrics_text,
                    ) {
                        cache_stats.insert(resource_type.to_string(), stats);
                    }
                }
            }
        }

        html.push_str("<h2>Cache Registry Stats</h2>");
        html.push_str(
            r#"<table>
                <tr>
                    <th>Resource Type</th>
                    <th>Size</th>
                    <th>Hits</th>
                    <th>Misses</th>
                    <th>Hit Rate</th>
                </tr>"#,
        );

        for (resource_type, stats) in cache_stats {
            let hit_rate = if stats.hits + stats.misses > 0 {
                stats.hits as f64 / (stats.hits + stats.misses) as f64 * 100.0
            } else {
                0.0
            };

            html.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{:.1}%</td>
                </tr>"#,
                resource_type, stats.size, stats.hits, stats.misses, hit_rate
            ));
        }

        html.push_str("</table>");
    } else {
        html.push_str("<p>Cache registry is disabled</p>");
    }

    html.push_str(
        r#"
        </body>
        </html>
        "#,
    );

    info!("🔍 Cache info page viewed");
    Html(html)
}
