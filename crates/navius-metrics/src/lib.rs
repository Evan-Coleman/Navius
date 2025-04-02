// Navius Metrics
//
// This crate provides metrics functionality for the Navius framework.
// It includes interfaces for collecting, recording, and exporting metrics,
// with support for various backend implementations.

use async_trait::async_trait;
use std::sync::Arc;
use thiserror::Error;

/// Error type for metrics operations
#[derive(Error, Debug)]
pub enum MetricsError {
    /// Failed to initialize metrics collector
    #[error("Failed to initialize metrics collector: {0}")]
    InitializationError(String),

    /// Failed to record metric
    #[error("Failed to record metric: {0}")]
    RecordingError(String),

    /// Failed to export metrics
    #[error("Failed to export metrics: {0}")]
    ExportError(String),

    /// General error
    #[error("Metrics error: {0}")]
    Other(String),
}

/// Result type for metrics operations
pub type Result<T> = std::result::Result<T, MetricsError>;

/// Trait for metrics collectors
pub trait MetricsCollector: Send + Sync {
    /// Record a counter increment
    fn increment_counter(&self, name: &str, value: u64) -> Result<()>;

    /// Record a gauge value
    fn record_gauge(&self, name: &str, value: f64) -> Result<()>;

    /// Record a histogram value
    fn record_histogram(&self, name: &str, value: f64) -> Result<()>;

    /// Add labels to a metric
    fn with_labels(&self, labels: &[(&str, &str)]) -> Arc<dyn MetricsCollector>;
}

/// Trait for metrics exporters
#[async_trait]
pub trait MetricsExporter: Send + Sync {
    /// Export collected metrics
    async fn export(&self) -> Result<()>;

    /// Shutdown the exporter
    async fn shutdown(&self) -> Result<()>;
}

/// Builder for metrics configuration
pub struct MetricsBuilder {
    #[allow(dead_code)]
    service_name: String,
    namespace: Option<String>,
    labels: Vec<(String, String)>,
}

impl MetricsBuilder {
    /// Create a new metrics builder
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            namespace: None,
            labels: Vec::new(),
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

    /// Build a metrics collector
    pub fn build(&self) -> Result<Arc<dyn MetricsCollector>> {
        // This would create a specific implementation based on features
        // For now, just return an error since no actual implementation exists yet
        Err(MetricsError::InitializationError(
            "No metrics implementation available".to_string(),
        ))
    }
}
