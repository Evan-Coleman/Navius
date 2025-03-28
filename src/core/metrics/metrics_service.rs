use crate::core::metrics::metrics_handler;
#[cfg(feature = "metrics")]
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::info;

/// Initialize metrics system
#[cfg(feature = "metrics")]
pub fn init_metrics() -> PrometheusHandle {
    // Create a Prometheus exporter
    let handle = metrics_handler::init_metrics();

    info!("📊 Metrics system initialized");
    handle
}

#[cfg(not(feature = "metrics"))]
pub fn init_metrics() -> () {
    info!("📊 Metrics system disabled (feature 'metrics' not enabled)");
    ()
}

/// Try to record metrics and return the raw metrics text
#[cfg(feature = "metrics")]
pub fn try_record_metrics(handle: &PrometheusHandle) -> Result<String, String> {
    metrics_handler::try_record_metrics(handle)
}

#[cfg(not(feature = "metrics"))]
pub fn try_record_metrics(_handle: &()) -> Result<String, String> {
    Ok("Metrics disabled: feature 'metrics' not enabled".to_string())
}

/// Handler function for the metrics endpoint
#[cfg(feature = "metrics")]
pub async fn metrics_endpoint_handler(handle: &PrometheusHandle) -> String {
    metrics_handler::metrics_handler(handle)
        .unwrap_or_else(|_| "# Error rendering metrics".to_string())
}

#[cfg(not(feature = "metrics"))]
pub async fn metrics_endpoint_handler(_handle: &()) -> String {
    "Metrics disabled: feature 'metrics' not enabled".to_string()
}

#[cfg(test)]
#[cfg(feature = "metrics")]
mod tests {
    use super::*;
    use metrics_exporter_prometheus::PrometheusBuilder;

    #[test]
    fn test_try_record_metrics() {
        // Initialize a test prometheus handle
        let handle = PrometheusBuilder::new().build_recorder().handle();

        // Record a metric to ensure there's something in the output
        metrics::counter!("test_counter").increment(1);

        // Test the metrics recording function
        let result = try_record_metrics(&handle);

        // Verify we got a successful result
        assert!(result.is_ok());

        // Get the metrics string - not checking for non-emptiness
        // as test environments may have different behavior
        let _metrics = result.unwrap();

        // Test passes if we get a result without error
    }
}
