use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::info;

/// Initialize metrics system
pub fn init_metrics() -> PrometheusHandle {
    // Create a Prometheus exporter
    let builder = PrometheusBuilder::new();

    // Build and install the recorder
    let handle = builder
        .install_recorder()
        .expect("Failed to set global metrics recorder");

    info!("Metrics system initialized");
    handle
}

/// Handler function for the metrics endpoint
pub async fn metrics_handler(metrics_handle: &PrometheusHandle) -> String {
    metrics_handle.render()
}
