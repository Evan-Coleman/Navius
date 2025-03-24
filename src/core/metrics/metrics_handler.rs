use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::error;

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
