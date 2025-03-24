use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::error;

/// Type alias for metrics handle to avoid direct dependency
pub type MetricsHandle = PrometheusHandle;

pub fn init_metrics() -> Option<PrometheusHandle> {
    let builder = PrometheusBuilder::new();
    match builder.install() {
        Ok(handle) => Some(handle),
        Err(e) => {
            error!("Failed to initialize metrics: {}", e);
            None
        }
    }
}

/// Export metrics in Prometheus format
pub async fn export_metrics(handle: &MetricsHandle) -> String {
    handle.render()
}

/// Update a gauge metric
pub fn update_gauge(name: &str, value: f64, labels: &[(&str, &str)]) {
    let mut metric_labels = Vec::new();
    for (key, value) in labels {
        metric_labels.push((*key, *value));
    }

    metrics::gauge!(name, value, &metric_labels);
}

/// Increment a counter metric
pub fn increment_counter(name: &str, value: u64, labels: &[(&str, &str)]) {
    let mut metric_labels = Vec::new();
    for (key, value) in labels {
        metric_labels.push((*key, *value));
    }

    metrics::counter!(name, value, &metric_labels);
}

/// Record a histogram metric
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]) {
    let mut metric_labels = Vec::new();
    for (key, value) in labels {
        metric_labels.push((*key, *value));
    }

    metrics::histogram!(name, value, &metric_labels);
}
