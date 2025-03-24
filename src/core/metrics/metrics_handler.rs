use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::error;

/// Type alias for metrics handle to avoid direct dependency
pub type MetricsHandle = PrometheusHandle;

/// Initialize metrics with Prometheus
pub fn init_metrics() -> MetricsHandle {
    match PrometheusBuilder::new().build() {
        Ok((recorder, exporter)) => {
            // Install the recorder globally
            metrics::set_recorder(recorder).expect("Failed to set global metrics recorder");

            // Return the handle
            PrometheusHandle::new()
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
        let metric_name = name.to_string();
        metrics::gauge!(&metric_name).set(value);
    } else if labels.len() == 1 {
        // Single label case
        let metric_name = name.to_string();
        let (key, val) = (labels[0].0.to_string(), labels[0].1.to_string());
        metrics::gauge!(&metric_name, &key => &val).set(value);
    } else {
        // Multiple labels - just use the base name
        let metric_name = name.to_string();
        metrics::gauge!(&metric_name).set(value);
    }
}

/// Increment a counter metric
pub fn increment_counter(name: &str, value: u64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // Basic metric without labels
        let metric_name = name.to_string();
        metrics::counter!(&metric_name).increment(value);
    } else if labels.len() == 1 {
        // Single label case
        let metric_name = name.to_string();
        let (key, val) = (labels[0].0.to_string(), labels[0].1.to_string());
        metrics::counter!(&metric_name, &key => &val).increment(value);
    } else {
        // Multiple labels - just use the base name
        let metric_name = name.to_string();
        metrics::counter!(&metric_name).increment(value);
    }
}

/// Record a histogram metric
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]) {
    if labels.is_empty() {
        // Basic metric without labels
        let metric_name = name.to_string();
        metrics::histogram!(&metric_name).record(value);
    } else if labels.len() == 1 {
        // Single label case
        let metric_name = name.to_string();
        let (key, val) = (labels[0].0.to_string(), labels[0].1.to_string());
        metrics::histogram!(&metric_name, &key => &val).record(value);
    } else {
        // Multiple labels - just use the base name
        let metric_name = name.to_string();
        metrics::histogram!(&metric_name).record(value);
    }
}
