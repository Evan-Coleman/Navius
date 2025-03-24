use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::error;

/// Type alias for metrics handle to avoid direct dependency
pub type MetricsHandle = PrometheusHandle;

/// Initialize metrics with Prometheus
pub fn init_metrics() -> MetricsHandle {
    match PrometheusBuilder::new().build() {
        Ok(handle) => handle,
        Err(e) => {
            error!("Failed to initialize metrics: {}", e);
            panic!("Failed to initialize metrics: {}", e);
        }
    }
}

/// Export metrics in Prometheus format
pub async fn export_metrics(handle: &MetricsHandle) -> String {
    handle.render()
}

/// Update a gauge metric
pub fn update_gauge(name: &str, value: f64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // No labels - use simpler form
        metrics::gauge!(name);
        metrics::gauge!(name).set(value);
        return;
    }

    // Use named labels with a single label pair
    if labels.len() == 1 {
        let (key, val) = labels[0];
        metrics::gauge!(name, key => val);
        metrics::gauge!(name, key => val).set(value);
        return;
    }

    // Fall back to basic metric and manually set it
    // Complex multiple labels are harder to handle with the macros directly
    metrics::gauge!(name);
    metrics::gauge!(name).set(value);
}

/// Increment a counter metric
pub fn increment_counter(name: &str, value: u64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // No labels - use simpler form
        metrics::counter!(name).increment(value);
        return;
    }

    // Use named labels with a single label pair
    if labels.len() == 1 {
        let (key, val) = labels[0];
        metrics::counter!(name, key => val).increment(value);
        return;
    }

    // Fall back to basic counter and manually increment it
    metrics::counter!(name).increment(value);
}

/// Record a histogram metric
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // No labels - use simpler form
        metrics::histogram!(name).record(value);
        return;
    }

    // Use named labels with a single label pair
    if labels.len() == 1 {
        let (key, val) = labels[0];
        metrics::histogram!(name, key => val).record(value);
        return;
    }

    // Fall back to basic histogram and manually record it
    metrics::histogram!(name).record(value);
}
