use crate::core::metrics::metrics_handler;
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::info;

/// Initialize metrics system
pub fn init_metrics() -> PrometheusHandle {
    // Create a Prometheus exporter
    let handle = metrics_handler::init_metrics();

    info!("📊 Metrics system initialized");
    handle
}

/// Try to record metrics and return the raw metrics text
pub fn try_record_metrics(handle: &PrometheusHandle) -> Result<String, String> {
    metrics_handler::try_record_metrics(handle)
}

/// Handler function for the metrics endpoint
pub async fn metrics_endpoint_handler(handle: &PrometheusHandle) -> String {
    metrics_handler::metrics_handler(handle)
        .unwrap_or_else(|_| "# Error rendering metrics".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use metrics_exporter_prometheus::PrometheusBuilder;

    #[test]
    fn test_try_record_metrics() {
        // Initialize a test prometheus handle
        let handle = PrometheusBuilder::new().build_recorder().handle();

        // Test the metrics recording function
        let result = try_record_metrics(&handle);

        // Verify we got a successful result
        assert!(result.is_ok());

        // Get the metrics string
        let metrics = result.unwrap();

        // Just check that we got some kind of prometheus metrics text back
        assert!(!metrics.is_empty());
    }
}
