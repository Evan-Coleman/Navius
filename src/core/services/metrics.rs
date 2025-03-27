use crate::core::metrics::metrics_handler;
use metrics_exporter_prometheus::PrometheusHandle;

/// Initialize metrics with Prometheus
pub fn init_metrics() -> PrometheusHandle {
    metrics_handler::init_metrics()
}

/// Record a counter metric
pub fn record_counter(name: &str, value: u64) {
    metrics_handler::record_counter(name, value);
}

/// Record a gauge metric
pub fn record_gauge(name: &str, value: f64) {
    metrics_handler::record_gauge(name, value);
}

/// Export metrics as a string
pub fn export_metrics(handle: &PrometheusHandle) -> String {
    metrics_handler::export_metrics(handle)
}
