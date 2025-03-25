use metrics::{Label, counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::collections::HashMap;
use tracing::error;

use crate::core::error::AppError;

/// Initialize metrics with Prometheus
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder")
}

/// Export metrics in Prometheus format
pub fn export_metrics(handle: &PrometheusHandle) -> String {
    handle.render()
}

/// Create a unique key for a metric
pub fn create_key(name: &str) -> String {
    format!("navius_{}", name)
}

/// Record a gauge metric value
pub fn record_gauge(name: &str, value: f64) {
    let key = create_key(name);
    gauge!(key).set(value);
}

/// Record a gauge metric value with labels
pub fn record_gauge_with_labels(name: &str, labels: &[(&str, String)], value: f64) {
    let key = create_key(name);

    if !labels.is_empty() {
        // Clone the key for each iteration to avoid lifetime issues
        for (k, v) in labels {
            let key_clone = key.clone();
            metrics::gauge!(key_clone, *k => v.as_str()).set(value);
        }
    } else {
        gauge!(key).set(value);
    }
}

/// Record a counter metric value
pub fn record_counter(name: &str, value: u64) {
    let key = create_key(name);
    counter!(key).increment(value);
}

/// Record a counter metric value with labels
pub fn record_counter_with_labels(name: &str, labels: &[(&str, String)], value: u64) {
    let key = create_key(name);

    if !labels.is_empty() {
        // Clone the key for each iteration to avoid lifetime issues
        for (k, v) in labels {
            let key_clone = key.clone();
            metrics::counter!(key_clone, *k => v.as_str()).increment(value);
        }
    } else {
        counter!(key).increment(value);
    }
}

/// Record a histogram metric value
pub fn record_histogram(name: &str, value: f64) {
    let key = create_key(name);
    histogram!(key).record(value);
}

/// Record a histogram metric value with labels
pub fn record_histogram_with_labels(name: &str, labels: &[(&str, String)], value: f64) {
    let key = create_key(name);

    if !labels.is_empty() {
        // Clone the key for each iteration to avoid lifetime issues
        for (k, v) in labels {
            let key_clone = key.clone();
            metrics::histogram!(key_clone, *k => v.as_str()).record(value);
        }
    } else {
        histogram!(key).record(value);
    }
}

/// Try to get a counter value
pub fn try_get_counter(_name: &str) -> Option<u64> {
    // In this implementation, we can't easily get metric values
    // Return a placeholder implementation that always returns Some(0)
    Some(0)
}

/// Try to get a counter value with labels
pub fn try_get_counter_with_labels(_name: &str, _labels: &[(&str, String)]) -> Option<u64> {
    // In this implementation, we can't easily get metric values
    // Return a placeholder implementation that always returns Some(0)
    Some(0)
}

/// Try to get a gauge value
pub fn try_get_gauge(_name: &str) -> Option<f64> {
    // In this implementation, we can't easily get metric values
    // Return a placeholder implementation that always returns Some(0.0)
    Some(0.0)
}

/// Try to get a gauge value with labels
pub fn try_get_gauge_with_labels(_name: &str, _labels: &[(&str, String)]) -> Option<f64> {
    // In this implementation, we can't easily get metric values
    // Return a placeholder implementation that always returns Some(0.0)
    Some(0.0)
}

/// Try to record metrics and return the raw metrics text
pub fn try_record_metrics(handle: &PrometheusHandle) -> Result<String, String> {
    Ok(export_metrics(handle))
}

/// Handle metrics requests and return the metrics endpoint
pub fn metrics_handler(handle: &PrometheusHandle) -> Result<String, AppError> {
    let metrics_text = export_metrics(handle);
    Ok(metrics_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_key() {
        let key = create_key("test_metric");
        assert_eq!(key, "navius_test_metric");
    }
}
