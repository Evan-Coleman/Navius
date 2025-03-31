// Navius Metrics Prometheus
//
// This crate provides a Prometheus implementation for the Navius metrics system.

use async_trait::async_trait;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use navius_metrics::{MetricsCollector, MetricsError, MetricsExporter, Result};
use std::sync::Arc;

/// Prometheus metrics collector
pub struct PrometheusCollector {
    handle: PrometheusHandle,
    labels: Vec<(String, String)>,
}

impl PrometheusCollector {
    /// Create a new Prometheus collector
    pub fn new(handle: PrometheusHandle) -> Self {
        Self {
            handle,
            labels: Vec::new(),
        }
    }
}

impl MetricsCollector for PrometheusCollector {
    fn increment_counter(&self, name: &str, value: u64) -> Result<()> {
        // Add labels and increment counter
        // This is simplified - actual implementation would need to handle labels
        metrics::counter!(name).increment(value);
        Ok(())
    }

    fn record_gauge(&self, name: &str, value: f64) -> Result<()> {
        metrics::gauge!(name).set(value);
        Ok(())
    }

    fn record_histogram(&self, name: &str, value: f64) -> Result<()> {
        metrics::histogram!(name).record(value);
        Ok(())
    }

    fn with_labels(&self, labels: &[(&str, &str)]) -> Arc<dyn MetricsCollector> {
        // Create a new collector with the combined labels
        let mut new_labels = self.labels.clone();
        for (k, v) in labels {
            new_labels.push((k.to_string(), v.to_string()));
        }

        Arc::new(Self {
            handle: self.handle.clone(),
            labels: new_labels,
        })
    }
}

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    _handle: PrometheusHandle,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(handle: PrometheusHandle) -> Self {
        Self { _handle: handle }
    }
}

#[async_trait]
impl MetricsExporter for PrometheusExporter {
    async fn export(&self) -> Result<()> {
        // Prometheus automatically handles metrics exposure via HTTP
        // Nothing to do here as it's handled by the HTTP server
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // Clean up any resources if needed
        Ok(())
    }
}

/// Builder for Prometheus metrics
pub struct PrometheusMetricsBuilder {
    service_name: String,
    namespace: Option<String>,
    labels: Vec<(String, String)>,
    listen_address: Option<String>,
}

impl PrometheusMetricsBuilder {
    /// Create a new Prometheus metrics builder
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            namespace: None,
            labels: Vec::new(),
            listen_address: None,
        }
    }

    /// Set the metrics namespace
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    /// Add a global label
    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.push((key.to_string(), value.to_string()));
        self
    }

    /// Set the HTTP listener address for exposing metrics
    pub fn with_http_listener(mut self, address: &str) -> Self {
        self.listen_address = Some(address.to_string());
        self
    }

    /// Build the Prometheus metrics system
    pub fn build(self) -> Result<(Arc<dyn MetricsCollector>, Arc<dyn MetricsExporter>)> {
        let mut builder = PrometheusBuilder::new();

        // Add global labels
        for (key, value) in &self.labels {
            builder = builder.add_global_label(key, value);
        }

        // Set namespace prefix if provided
        if let Some(namespace) = &self.namespace {
            builder = builder.with_namespace(namespace);
        }

        // Install HTTP listener if address provided
        let handle = if let Some(addr) = self.listen_address {
            #[cfg(feature = "http-listener")]
            {
                // Install with HTTP listener
                match builder.install_recorder() {
                    Ok(h) => h,
                    Err(e) => return Err(MetricsError::InitializationError(e.to_string())),
                }
            }
            #[cfg(not(feature = "http-listener"))]
            {
                return Err(MetricsError::InitializationError(
                    "HTTP listener feature is not enabled".to_string(),
                ));
            }
        } else {
            // Install without HTTP listener
            match builder.install_recorder() {
                Ok(h) => h,
                Err(e) => return Err(MetricsError::InitializationError(e.to_string())),
            }
        };

        let collector = Arc::new(PrometheusCollector::new(handle.clone()));
        let exporter = Arc::new(PrometheusExporter::new(handle));

        Ok((collector, exporter))
    }
}
