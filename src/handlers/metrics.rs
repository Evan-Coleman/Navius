use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::info;

use crate::app::AppState;
use crate::models::MetricsResponse;

/// Get Prometheus metrics
///
/// Returns Prometheus metrics in text format
pub async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let raw_metrics = state.metrics_handle.render();

    // Format the metrics output for better readability
    let mut formatted_metrics = String::new();

    // Add a header to the metrics output
    formatted_metrics.push_str("# Petstore API Metrics\n\n");

    // Organize metrics by category
    let mut cache_metrics = Vec::new();
    let mut reliability_metrics = Vec::new();
    let mut other_metrics = Vec::new();

    // Collect and categorize metrics
    for line in raw_metrics.lines() {
        if line.starts_with('#') {
            continue; // Skip comments
        }

        if line.contains("cache_") {
            cache_metrics.push(line);
        } else if line.contains("reliability_") {
            reliability_metrics.push(line);
        } else {
            other_metrics.push(line);
        }
    }

    // Add cache metrics section with header
    if !cache_metrics.is_empty() {
        formatted_metrics.push_str("# HELP cache_* Cache-related metrics\n");
        formatted_metrics.push_str("# TYPE cache_hits_total counter\n");
        formatted_metrics.push_str("# TYPE cache_misses_total counter\n");
        formatted_metrics.push_str("# TYPE cache_entries_created counter\n");
        formatted_metrics.push_str("# TYPE cache_current_size gauge\n");
        formatted_metrics.push_str("# TYPE cache_active_entries gauge\n\n");

        // Group cache metrics by resource type for better readability
        let mut resource_metrics: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for metric in cache_metrics {
            // Extract resource type if present
            if let Some(start_idx) = metric.find("resource_type=\"") {
                let resource_start = start_idx + "resource_type=\"".len();
                if let Some(end_idx) = metric[resource_start..].find('"') {
                    let resource_type = &metric[resource_start..(resource_start + end_idx)];

                    resource_metrics
                        .entry(resource_type.to_string())
                        .or_insert_with(Vec::new)
                        .push(metric.to_string());
                } else {
                    formatted_metrics.push_str(metric);
                    formatted_metrics.push('\n');
                }
            } else {
                formatted_metrics.push_str(metric);
                formatted_metrics.push('\n');
            }
        }

        // Add metrics grouped by resource type
        for (resource_type, metrics) in resource_metrics {
            formatted_metrics.push_str(&format!("# Resource: {}\n", resource_type));
            for metric in metrics {
                formatted_metrics.push_str(&metric);
                formatted_metrics.push('\n');
            }
            formatted_metrics.push('\n');
        }
    }

    // Add reliability metrics section
    if !reliability_metrics.is_empty() {
        formatted_metrics.push_str("# HELP reliability_* Reliability-related metrics\n");
        formatted_metrics.push_str("# TYPE reliability_requests_total counter\n");
        formatted_metrics.push_str("# TYPE reliability_requests_successful counter\n");
        formatted_metrics.push_str("# TYPE reliability_error_rate gauge\n\n");

        for metric in reliability_metrics {
            formatted_metrics.push_str(metric);
            formatted_metrics.push('\n');
        }
        formatted_metrics.push('\n');
    }

    // Add other metrics
    if !other_metrics.is_empty() {
        formatted_metrics.push_str("# Other metrics\n");
        for metric in other_metrics {
            formatted_metrics.push_str(metric);
            formatted_metrics.push('\n');
        }
    }

    info!("📊 Metrics endpoint accessed");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain")],
        formatted_metrics,
    )
}
