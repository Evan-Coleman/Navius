use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::collections::BTreeMap;
use tracing::info;

/// Initialize metrics system
pub fn init_metrics() -> PrometheusHandle {
    // Create a Prometheus exporter
    let builder = PrometheusBuilder::new();

    // Build and install the recorder
    let handle = builder
        .install_recorder()
        .expect("Failed to set global metrics recorder");

    // No need to pre-register metrics with this metrics crate version
    // They are automatically created when first used

    info!("📊 Metrics system initialized");
    handle
}

/// Handler function for the metrics endpoint
pub async fn metrics_handler(metrics_handle: &PrometheusHandle) -> String {
    // Get raw metrics from Prometheus
    let raw_metrics = metrics_handle.render();

    // Sort metrics by key for consistent output
    let mut sorted_metrics = BTreeMap::new();

    // Parse and sort metrics by their names
    for line in raw_metrics.lines() {
        // Skip comment lines (they start with #)
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Extract metric name and value
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            sorted_metrics.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // Rebuild the metrics output in sorted order
    let mut result = String::new();

    // First add any header/metadata lines
    for line in raw_metrics.lines() {
        if line.starts_with('#') {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Then add the sorted metrics
    for (key, value) in sorted_metrics {
        result.push_str(&format!("{} {}\n", key, value));
    }

    result
}
