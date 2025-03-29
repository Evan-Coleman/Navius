//! Observability module for metrics, tracing, and profiling
//!
//! This module provides a generic interface for observability operations
//! including metrics, distributed tracing, and profiling.
//!
//! # Example
//!
//! ```rust
//! use navius::core::observability::{
//!     ObservabilityConfig, ObservabilityProviderRegistry, PrometheusProvider
//! };
//!
//! async fn setup_observability() -> ObservabilityService {
//!     let mut registry = ObservabilityProviderRegistry::new();
//!     registry.register(PrometheusProvider::new());
//!     
//!     let config = ObservabilityConfig::new("prometheus", "my-service");
//!     let service = ObservabilityService::new(registry, config).await.unwrap();
//!     
//!     service
//! }
//! ```

pub mod config;
pub mod error;
pub mod operations;
pub mod prometheus;
pub mod provider;
pub mod service;

// Re-export key types
pub use config::ObservabilityConfig;
pub use error::ObservabilityError;
pub use operations::{
    MetricType, MetricValue, ObservabilityOperations, ProfilingSession, SpanContext, SpanStatus,
};
pub use prometheus::{PrometheusClient, PrometheusProvider, get_prometheus_metrics_text};
pub use provider::{ObservabilityProvider, ObservabilityProviderRegistry};
pub use service::ObservabilityService;

/// Initialize the observability system with the default configuration
///
/// This is a convenience function for applications that want to use
/// the default Prometheus provider.
pub async fn init_observability(
    service_name: &str,
) -> Result<ObservabilityService, ObservabilityError> {
    let mut registry = ObservabilityProviderRegistry::new();
    registry.register(PrometheusProvider::new());
    registry.set_default_provider("prometheus")?;

    let config = ObservabilityConfig::new("prometheus", service_name);
    ObservabilityService::new(registry, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_observability() {
        let result = init_observability("test-service").await;
        assert!(result.is_ok());
    }
}
