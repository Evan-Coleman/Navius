use crate::broker::MessageBroker;
use crate::config::{BrokerConfig, MessagingConfig};
use crate::error::MessagingResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A trait for message provider implementations.
///
/// Providers implement specific messaging backends such as RabbitMQ, Kafka, etc.
/// They are responsible for creating and managing message brokers for their specific backend.
#[async_trait]
pub trait MessageProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Get the provider version.
    fn version(&self) -> &str;

    /// Create a new broker instance with the given configuration.
    ///
    /// # Arguments
    /// * `name` - The name for the broker
    /// * `config` - The broker configuration
    ///
    /// # Returns
    /// A result containing the broker if successful
    async fn create_broker(
        &self,
        name: &str,
        config: &BrokerConfig,
    ) -> MessagingResult<Arc<dyn MessageBroker>>;

    /// Check if this provider supports the given provider type.
    ///
    /// # Arguments
    /// * `provider_type` - The provider type to check
    ///
    /// # Returns
    /// `true` if this provider supports the given type, `false` otherwise
    fn supports(&self, provider_type: &str) -> bool {
        self.name().eq_ignore_ascii_case(provider_type)
    }
}

/// A registry of message providers.
///
/// This registry manages all available providers and is used to create brokers
/// of the appropriate type for a given configuration.
#[derive(Default)]
pub struct MessageProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn MessageProvider>>>,
}

impl MessageProviderRegistry {
    /// Create a new provider registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider.
    ///
    /// # Arguments
    /// * `provider` - The provider to register
    pub fn register(&self, provider: Arc<dyn MessageProvider>) {
        let name = provider.name().to_lowercase();
        let mut providers = self.providers.write().unwrap();
        providers.insert(name, provider);
    }

    /// Get a provider by name.
    ///
    /// # Arguments
    /// * `name` - The name of the provider to get
    ///
    /// # Returns
    /// The provider if found, or `None` if no provider with the given name exists
    pub fn get(&self, name: &str) -> Option<Arc<dyn MessageProvider>> {
        let providers = self.providers.read().unwrap();
        providers.get(&name.to_lowercase()).cloned()
    }

    /// Get a provider for the given provider type.
    ///
    /// # Arguments
    /// * `provider_type` - The type of provider to get
    ///
    /// # Returns
    /// The first provider that supports the given type, or `None` if no such provider exists
    pub fn get_for_type(&self, provider_type: &str) -> Option<Arc<dyn MessageProvider>> {
        let providers = self.providers.read().unwrap();

        // First try exact match by name
        if let Some(provider) = providers.get(&provider_type.to_lowercase()) {
            return Some(provider.clone());
        }

        // Then try to find a provider that supports the given type
        for provider in providers.values() {
            if provider.supports(provider_type) {
                return Some(provider.clone());
            }
        }

        None
    }

    /// Get all registered providers.
    ///
    /// # Returns
    /// A vector of all registered providers
    pub fn all(&self) -> Vec<Arc<dyn MessageProvider>> {
        let providers = self.providers.read().unwrap();
        providers.values().cloned().collect()
    }

    /// Get the number of registered providers.
    ///
    /// # Returns
    /// The number of registered providers
    pub fn count(&self) -> usize {
        let providers = self.providers.read().unwrap();
        providers.len()
    }

    /// Create a broker using the appropriate provider for the given configuration.
    ///
    /// # Arguments
    /// * `name` - The name for the broker
    /// * `config` - The broker configuration
    ///
    /// # Returns
    /// A result containing the broker if successful
    pub async fn create_broker(
        &self,
        name: &str,
        config: &BrokerConfig,
    ) -> MessagingResult<Arc<dyn MessageBroker>> {
        let provider = self.get_for_type(&config.provider).ok_or_else(|| {
            crate::error::MessagingError::ConfigurationError(format!(
                "No provider available for type: {}",
                config.provider
            ))
        })?;

        provider.create_broker(name, config).await
    }
}
