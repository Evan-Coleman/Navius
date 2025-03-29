use std::sync::Arc;

use tracing::{error, info};

use crate::core::observability::config::ObservabilityConfig;
use crate::core::observability::error::ObservabilityError;
use crate::core::observability::opentelemetry::{JaegerProvider, OtlpProvider};
use crate::core::observability::operations::{
    MetricType, MetricValue, ObservabilityOperations, ProfilingSession, SpanContext, SpanStatus,
};
use crate::core::observability::prometheus::PrometheusProvider;
use crate::core::observability::provider::ObservabilityProviderRegistry;

/// Main service for observability operations
#[derive(Clone)]
pub struct ObservabilityService {
    /// The client for performing observability operations
    client: Arc<dyn ObservabilityOperations>,
    /// The provider registry
    registry: Arc<ObservabilityProviderRegistry>,
    /// The service configuration
    config: ObservabilityConfig,
}

impl ObservabilityService {
    /// Create a new observability service using the specified provider
    pub async fn new(
        registry: ObservabilityProviderRegistry,
        config: ObservabilityConfig,
    ) -> Result<Self, ObservabilityError> {
        let provider_name = config.provider.clone();
        let registry = Arc::new(registry);

        info!(
            "Creating observability service with provider: {}",
            provider_name
        );

        let client = registry
            .create_client(&provider_name, config.clone())
            .await?;

        Ok(Self {
            client: Arc::new(*client),
            registry,
            config,
        })
    }

    /// Create a new observability service using the default provider
    pub async fn new_default(
        registry: ObservabilityProviderRegistry,
        config: ObservabilityConfig,
    ) -> Result<Self, ObservabilityError> {
        let registry = Arc::new(registry);

        info!("Creating observability service with default provider");

        let client = registry.create_default_client(config.clone()).await?;

        Ok(Self {
            client: Arc::new(*client),
            registry,
            config,
        })
    }

    /// Create a new observability service with all available providers registered
    pub async fn new_with_all_providers(
        config: ObservabilityConfig,
    ) -> Result<Self, ObservabilityError> {
        let mut registry = ObservabilityProviderRegistry::new();

        // Register all available providers
        registry.register(PrometheusProvider::new());

        #[cfg(feature = "opentelemetry-jaeger")]
        registry.register(JaegerProvider::new());

        #[cfg(feature = "otlp")]
        registry.register(OtlpProvider::new());

        // Use the specified provider from config, or the first available one
        let provider_name = config.provider.clone();
        if !registry.provider_exists(&provider_name) {
            // If the requested provider isn't available, use the first available one
            let default_provider = "prometheus";
            info!(
                "Requested provider '{}' not available, using '{}' instead",
                provider_name, default_provider
            );

            registry.set_default_provider(default_provider)?;

            let mut new_config = config.clone();
            new_config.provider = default_provider.to_string();

            Self::new_default(registry, new_config).await
        } else {
            // Use the requested provider
            registry.set_default_provider(&provider_name)?;
            Self::new(registry, config).await
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }

    /// Record a counter metric
    pub fn record_counter(&self, name: &str, value: u64) -> Result<(), ObservabilityError> {
        self.client.record_counter(name, value, &[])
    }

    /// Record a counter metric with labels
    pub fn record_counter_with_labels(
        &self,
        name: &str,
        value: u64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError> {
        self.client.record_counter(name, value, labels)
    }

    /// Record a gauge metric
    pub fn record_gauge(&self, name: &str, value: f64) -> Result<(), ObservabilityError> {
        self.client.record_gauge(name, value, &[])
    }

    /// Record a gauge metric with labels
    pub fn record_gauge_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError> {
        self.client.record_gauge(name, value, labels)
    }

    /// Record a histogram metric
    pub fn record_histogram(&self, name: &str, value: f64) -> Result<(), ObservabilityError> {
        self.client.record_histogram(name, value, &[])
    }

    /// Record a histogram metric with labels
    pub fn record_histogram_with_labels(
        &self,
        name: &str,
        value: f64,
        labels: &[(&str, String)],
    ) -> Result<(), ObservabilityError> {
        self.client.record_histogram(name, value, labels)
    }

    /// Get a metric value by name and type
    pub fn get_metric(
        &self,
        name: &str,
        metric_type: MetricType,
    ) -> Result<Option<MetricValue>, ObservabilityError> {
        self.client.get_metric(name, metric_type, &[])
    }

    /// Get a metric value by name, type, and labels
    pub fn get_metric_with_labels(
        &self,
        name: &str,
        metric_type: MetricType,
        labels: &[(&str, String)],
    ) -> Result<Option<MetricValue>, ObservabilityError> {
        self.client.get_metric(name, metric_type, labels)
    }

    /// Start a span for distributed tracing
    pub fn start_span(&self, name: &str) -> SpanContext {
        self.client.start_span(name)
    }

    /// End a span
    pub fn end_span(&self, context: SpanContext) {
        self.client.end_span(context)
    }

    /// Set an attribute on a span
    pub fn set_span_attribute(&self, context: &SpanContext, key: &str, value: &str) {
        self.client.set_span_attribute(context, key, value)
    }

    /// Set the status of a span
    pub fn set_span_status(
        &self,
        context: &SpanContext,
        status: SpanStatus,
        description: Option<&str>,
    ) {
        self.client.set_span_status(context, status, description)
    }

    /// Start a profiling session
    pub fn start_profiling(&self, name: &str) -> Result<ProfilingSession, ObservabilityError> {
        if !self.config.profiling_enabled {
            error!("Profiling is disabled in the configuration");
            return Err(ObservabilityError::UnsupportedConfiguration(
                "Profiling is disabled in the configuration".to_string(),
            ));
        }

        self.client.start_profiling(name)
    }

    /// Perform a health check on the observability system
    pub fn health_check(&self) -> Result<bool, ObservabilityError> {
        self.client.health_check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::observability::prometheus::PrometheusProvider;

    #[tokio::test]
    async fn test_observability_service_creation() {
        // Create registry and register provider
        let mut registry = ObservabilityProviderRegistry::new();
        registry.register(PrometheusProvider::new());

        // Set default provider
        let result = registry.set_default_provider("prometheus");
        assert!(result.is_ok());

        // Create configuration
        let config = ObservabilityConfig::new("prometheus", "test-service");

        // Create service
        let service_result = ObservabilityService::new(registry, config).await;
        assert!(service_result.is_ok());
    }

    #[tokio::test]
    async fn test_observability_service_metrics() {
        // Create registry and register provider
        let mut registry = ObservabilityProviderRegistry::new();
        registry.register(PrometheusProvider::new());

        // Create configuration
        let config = ObservabilityConfig::new("prometheus", "test-service");

        // Create service
        let service = ObservabilityService::new(registry, config).await.unwrap();

        // Record counter
        let result = service.record_counter("test_counter", 42);
        assert!(result.is_ok());

        // Record gauge
        let result = service.record_gauge("test_gauge", 3.14);
        assert!(result.is_ok());

        // Record histogram
        let result = service.record_histogram("test_histogram", 2.71);
        assert!(result.is_ok());

        // Record with labels
        let labels = vec![("test_key", "test_value".to_string())];
        let result = service.record_counter_with_labels("test_counter_labeled", 1, &labels);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_observability_service_with_all_providers() {
        // Create configuration
        let config = ObservabilityConfig::new("prometheus", "test-service");

        // Create service with all providers
        let service_result = ObservabilityService::new_with_all_providers(config).await;
        assert!(service_result.is_ok());

        // Check that the service uses the requested provider
        let service = service_result.unwrap();
        assert_eq!(service.config().provider, "prometheus");
    }
}
