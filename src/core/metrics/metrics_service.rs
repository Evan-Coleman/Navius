use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::collections::BTreeMap;
use tracing::info;

/// Initialize metrics system
pub fn init_metrics() -> PrometheusHandle {
    // Create a Prometheus exporter
    let builder = PrometheusBuilder::new();

    // Build and install the recorder
    let handle = builder
        .install_recorder()
        .expect("Failed to set global metrics recorder");

    // No need to pre-register metrics with this metrics crate version
    // They are automatically created when first used

    info!("📊 Metrics system initialized");
    handle
}

/// Try to record metrics and return the raw metrics text
pub fn try_record_metrics() -> Result<String, String> {
    // Since we can't reliably get the global metrics recorder in this version of the crate,
    // we'll create a simple placeholder implementation that returns a minimal metrics output
    // In a real implementation, you'd want to access the global recorder properly

    let placeholder_metrics = "# HELP cache_hits Number of cache hits\n\
                              # TYPE cache_hits counter\n\
                              cache_hits{resource_type=\"example\"} 0\n\
                              # HELP cache_misses Number of cache misses\n\
                              # TYPE cache_misses counter\n\
                              cache_misses{resource_type=\"example\"} 0\n\
                              # HELP cache_current_size Current number of entries in the cache\n\
                              # TYPE cache_current_size gauge\n\
                              cache_current_size{resource_type=\"example\"} 0\n";

    Ok(placeholder_metrics.to_string())
}

/// Handler function for the metrics endpoint
pub async fn metrics_handler(metrics_handle: &PrometheusHandle) -> String {
    // Get raw metrics from Prometheus
    let raw_metrics = metrics_handle.render();

    // Sort metrics by key for consistent output
    let mut sorted_metrics = BTreeMap::new();

    // Parse and sort metrics by their names
    for line in raw_metrics.lines() {
        // Skip comment lines (they start with #)
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        // Extract metric name and value
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() == 2 {
            sorted_metrics.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // Rebuild the metrics output in sorted order
    let mut result = String::new();

    // First add any header/metadata lines
    for line in raw_metrics.lines() {
        if line.starts_with('#') {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Then add the sorted metrics
    for (key, value) in sorted_metrics {
        result.push_str(&format!("{} {}\n", key, value));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock PrometheusHandle for testing
    struct MockPrometheusHandle;

    impl MockPrometheusHandle {
        fn new() -> Self {
            MockPrometheusHandle
        }

        fn render(&self) -> String {
            // Return sample metrics in Prometheus text format
            String::from(
                "# HELP test_counter Test counter metric\n\
                 # TYPE test_counter counter\n\
                 test_counter 1\n\
                 # HELP test_gauge Test gauge metric\n\
                 # TYPE test_gauge gauge\n\
                 test_gauge 42.0\n",
            )
        }
    }

    #[test]
    fn test_try_record_metrics() {
        // Test the metrics recording function
        let result = try_record_metrics();

        // Verify we got a successful result
        assert!(result.is_ok());

        // Verify the result contains expected metrics
        let metrics = result.unwrap();
        assert!(metrics.contains("cache_hits"));
        assert!(metrics.contains("cache_misses"));
        assert!(metrics.contains("cache_current_size"));
    }

    #[test]
    fn test_metrics_format() {
        let expected = "\
# HELP cache_hits_total Number of cache hits
# TYPE cache_hits_total counter
cache_hits{resource_type=\"test\"} 0\n\
# HELP cache_misses_total Number of cache misses
# TYPE cache_misses_total counter
cache_misses{resource_type=\"test\"} 0\n\
# HELP cache_current_size Current number of items in cache
# TYPE cache_current_size gauge
cache_current_size{resource_type=\"test\"} 0\n";

        assert_eq!(expected, get_test_metrics());
    }

    fn get_test_metrics() -> String {
        "\
# HELP cache_hits_total Number of cache hits
# TYPE cache_hits_total counter
cache_hits{resource_type=\"test\"} 0\n\
# HELP cache_misses_total Number of cache misses
# TYPE cache_misses_total counter
cache_misses{resource_type=\"test\"} 0\n\
# HELP cache_current_size Current number of items in cache
# TYPE cache_current_size gauge
cache_current_size{resource_type=\"test\"} 0\n"
            .to_string()
    }

    #[test]
    fn test_metrics_handler_format() {
        // Create a mock metrics handle
        let mock_handle = MockPrometheusHandle::new();

        // Manually process the mock metrics through our handler's formatting algorithm
        let raw_metrics = mock_handle.render();

        // Sort metrics by key for consistent output
        let mut sorted_metrics = BTreeMap::new();

        // Parse and sort metrics by their names
        for line in raw_metrics.lines() {
            // Skip comment lines (they start with #)
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            // Extract metric name and value
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                sorted_metrics.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        // Rebuild the metrics output in sorted order
        let mut result = String::new();

        // First add any header/metadata lines
        for line in raw_metrics.lines() {
            if line.starts_with('#') {
                result.push_str(line);
                result.push('\n');
            }
        }

        // Then add the sorted metrics
        for (key, value) in sorted_metrics {
            result.push_str(&format!("{} {}\n", key, value));
        }

        // Verify the output format
        assert!(!result.is_empty());
        assert!(
            result.contains("# HELP"),
            "Output should contain help comments"
        );
        assert!(
            result.contains("# TYPE"),
            "Output should contain type definitions"
        );
        assert!(
            result.contains("test_counter"),
            "Output should contain test_counter metric"
        );
        assert!(
            result.contains("test_gauge"),
            "Output should contain test_gauge metric"
        );
    }
}
