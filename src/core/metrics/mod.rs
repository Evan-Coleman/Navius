pub mod metrics_handler;
pub mod metrics_service;

// Import required external dependencies
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

// Re-export key components for easier access
pub use metrics_handler::{
    create_key, export_metrics, metrics_handler, try_get_counter, try_get_counter_with_labels,
    try_get_gauge, try_get_gauge_with_labels, try_record_metrics,
};
pub use metrics_service::metrics_endpoint_handler;

/// Initialize metrics with Prometheus for easy recording
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

// Metric recording functions with static strings
pub fn record_counter(name: &'static str, value: u64) {
    metrics::counter!(name).increment(value);
}

pub fn record_gauge(name: &'static str, value: f64) {
    metrics::gauge!(name).set(value);
}

pub fn record_histogram(name: &'static str, value: f64) {
    metrics::histogram!(name).record(value);
}

pub fn record_counter_with_labels(
    name: &'static str,
    labels: &[(&'static str, &'static str)],
    value: u64,
) {
    metrics::counter!(name, labels).increment(value);
}

pub fn record_gauge_with_labels(
    name: &'static str,
    labels: &[(&'static str, &'static str)],
    value: f64,
) {
    metrics::gauge!(name, labels).set(value);
}

pub fn record_histogram_with_labels(
    name: &'static str,
    labels: &[(&'static str, &'static str)],
    value: f64,
) {
    metrics::histogram!(name, labels).record(value);
}
