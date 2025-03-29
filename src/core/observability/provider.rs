use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::core::observability::config::ObservabilityConfig;
use crate::core::observability::error::ObservabilityError;
use crate::core::observability::operations::{
    MetricType, MetricValue, ObservabilityOperations, ProfilingSession, SpanContext, SpanStatus,
};

/// Observability provider interface
#[async_trait]
pub trait ObservabilityProvider: Send + Sync + 'static {
    /// Create a new observability client
    async fn create_client(
        &self,
        config: ObservabilityConfig,
    ) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError>;

    /// Check if this provider supports the given configuration
    fn supports(&self, config: &ObservabilityConfig) -> bool;

    /// Get the name of this provider
    fn name(&self) -> &str;
}

/// Observability provider registry
pub struct ObservabilityProviderRegistry {
    providers: HashMap<String, Arc<dyn ObservabilityProvider>>,
    default_provider: Option<String>,
}

impl ObservabilityProviderRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    /// Register a provider
    pub fn register<P: ObservabilityProvider>(&mut self, provider: P) {
        let name = provider.name().to_string();
        self.providers.insert(name, Arc::new(provider));
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: &str) -> Result<(), ObservabilityError> {
        if !self.providers.contains_key(name) {
            return Err(ObservabilityError::ProviderNotFound(format!(
                "Cannot set default provider. Provider not found: {}",
                name
            )));
        }
        self.default_provider = Some(name.to_string());
        Ok(())
    }

    /// Get a provider by name
    pub fn get_provider(
        &self,
        name: &str,
    ) -> Result<Arc<dyn ObservabilityProvider>, ObservabilityError> {
        self.providers.get(name).cloned().ok_or_else(|| {
            ObservabilityError::ProviderNotFound(format!("Provider not found: {}", name))
        })
    }

    /// Get the default provider
    pub fn get_default_provider(
        &self,
    ) -> Result<Arc<dyn ObservabilityProvider>, ObservabilityError> {
        match &self.default_provider {
            Some(name) => self.get_provider(name),
            None => Err(ObservabilityError::NoDefaultProvider(
                "No default provider has been set".to_string(),
            )),
        }
    }

    /// Create an observability client with the specified provider
    pub async fn create_client(
        &self,
        provider_name: &str,
        config: ObservabilityConfig,
    ) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError> {
        let provider = self.get_provider(provider_name)?;

        if !provider.supports(&config) {
            return Err(ObservabilityError::UnsupportedConfiguration(format!(
                "Provider {} does not support the given configuration",
                provider_name
            )));
        }

        provider.create_client(config).await
    }

    /// Create an observability client with the default provider
    pub async fn create_default_client(
        &self,
        config: ObservabilityConfig,
    ) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError> {
        let provider = self.get_default_provider()?;

        if !provider.supports(&config) {
            return Err(ObservabilityError::UnsupportedConfiguration(
                "Default provider does not support the given configuration".to_string(),
            ));
        }

        provider.create_client(config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use std::sync::Arc;

    mock! {
        pub ObservabilityProviderMock {}

        #[async_trait]
        impl ObservabilityProvider for ObservabilityProviderMock {
            async fn create_client(&self, config: ObservabilityConfig) -> Result<Box<dyn ObservabilityOperations>, ObservabilityError>;
            fn supports(&self, config: &ObservabilityConfig) -> bool;
            fn name(&self) -> &str;
        }
    }

    mock! {
        pub ObservabilityOperationsMock {}

        impl ObservabilityOperations for ObservabilityOperationsMock {
            fn record_counter(&self, name: &str, value: u64, labels: &[(&str, String)]) -> Result<(), ObservabilityError>;
            fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, String)]) -> Result<(), ObservabilityError>;
            fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, String)]) -> Result<(), ObservabilityError>;
            fn get_metric(&self, name: &str, metric_type: MetricType, labels: &[(&str, String)]) -> Result<Option<MetricValue>, ObservabilityError>;
            fn start_span(&self, name: &str) -> SpanContext;
            fn end_span(&self, context: SpanContext);
            fn set_span_attribute(&self, context: &SpanContext, key: &str, value: &str);
            fn set_span_status(&self, context: &SpanContext, status: SpanStatus, description: Option<&str>);
            fn start_profiling(&self, name: &str) -> Result<ProfilingSession, ObservabilityError>;
            fn health_check(&self) -> Result<bool, ObservabilityError>;
        }
    }

    #[tokio::test]
    async fn test_registry_register_and_get() {
        // Create a mock provider
        let mut mock_provider = MockObservabilityProviderMock::new();
        mock_provider
            .expect_name()
            .return_const("mock-provider".to_string());
        mock_provider.expect_supports().return_const(true);

        // Create registry and register the provider
        let mut registry = ObservabilityProviderRegistry::new();
        registry.register(mock_provider);

        // Get the provider
        let result = registry.get_provider("mock-provider");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_default_provider() {
        // Create a mock provider
        let mut mock_provider = MockObservabilityProviderMock::new();
        mock_provider
            .expect_name()
            .return_const("mock-provider".to_string());

        // Create registry and register the provider
        let mut registry = ObservabilityProviderRegistry::new();
        registry.register(mock_provider);

        // Set default provider
        let result = registry.set_default_provider("mock-provider");
        assert!(result.is_ok());

        // Get default provider
        let result = registry.get_default_provider();
        assert!(result.is_ok());
    }
}
