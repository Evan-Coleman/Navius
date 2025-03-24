use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::error;

/// Type alias for metrics handle to avoid direct dependency
pub type MetricsHandle = PrometheusHandle;

/// Initialize metrics with Prometheus
pub fn init_metrics() -> MetricsHandle {
    match PrometheusBuilder::new().build() {
        Ok((recorder, handle)) => {
            // Install the recorder
            if let Err(e) = metrics::set_global_recorder(recorder) {
                error!("Failed to set global metrics recorder: {}", e);
                panic!("Failed to set global metrics recorder: {}", e);
            }
            handle
        }
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
        // Basic metric without labels
        metrics::gauge!(name).set(value);
    } else if labels.len() == 1 {
        // Single label case
        let (key, val) = labels[0];
        metrics::gauge!(name, key => val).set(value);
    } else {
        // Multiple labels - just use the base name for now
        metrics::gauge!(name).set(value);
    }
}

/// Increment a counter metric
pub fn increment_counter(name: &str, value: u64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // Basic metric without labels
        metrics::counter!(name).increment(value);
    } else if labels.len() == 1 {
        // Single label case
        let (key, val) = labels[0];
        metrics::counter!(name, key => val).increment(value);
    } else {
        // Multiple labels - just use the base name for now
        metrics::counter!(name).increment(value);
    }
}

/// Record a histogram metric
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // Basic metric without labels
        metrics::histogram!(name).record(value);
    } else if labels.len() == 1 {
        // Single label case
        let (key, val) = labels[0];
        metrics::histogram!(name, key => val).record(value);
    } else {
        // Multiple labels - just use the base name for now
        metrics::histogram!(name).record(value);
    }
}
