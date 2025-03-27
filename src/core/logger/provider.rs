use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::config::LoggingConfig;
use super::error::LoggingError;
use super::interface::LoggingOperations;

/// Trait for logger provider implementations
#[async_trait]
pub trait LoggingProvider: Send + Sync + 'static {
    /// Creates a new logger instance with the specified configuration
    async fn create_logger(
        &self,
        config: &LoggingConfig,
    ) -> Result<Arc<dyn LoggingOperations>, LoggingError>;

    /// Returns the name of the provider
    fn name(&self) -> &'static str;

    /// Checks if this provider supports the given configuration
    fn supports(&self, config: &LoggingConfig) -> bool;

    /// Validates provider-specific configuration
    fn validate_config(&self, config: &LoggingConfig) -> Result<(), LoggingError> {
        // Default implementation just returns Ok
        Ok(())
    }
}

/// Registry for logging providers
#[derive(Default)]
pub struct LoggingProviderRegistry {
    providers: Mutex<HashMap<String, Arc<dyn LoggingProvider>>>,
    default_provider_name: Mutex<String>,
}

impl LoggingProviderRegistry {
    /// Create a new empty provider registry
    pub fn new() -> Self {
        Self {
            providers: Mutex::new(HashMap::new()),
            default_provider_name: Mutex::new("tracing".to_string()),
        }
    }

    /// Register a provider with the registry
    pub fn register_provider(
        &self,
        provider: Arc<dyn LoggingProvider>,
    ) -> Result<(), LoggingError> {
        let name = provider.name().to_string();
        let mut providers = self.providers.lock().unwrap();

        // Check if a provider with this name already exists
        if providers.contains_key(&name) {
            return Err(LoggingError::ProviderError(format!(
                "Provider with name '{}' already registered",
                name
            )));
        }

        // Add the provider to the registry
        providers.insert(name, provider);
        Ok(())
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Result<Arc<dyn LoggingProvider>, LoggingError> {
        let providers = self.providers.lock().unwrap();

        providers.get(name).cloned().ok_or_else(|| {
            LoggingError::ProviderNotFound(format!("Logging provider '{}' not found", name))
        })
    }

    /// Set the default provider name
    pub fn set_default_provider(&self, name: &str) -> Result<(), LoggingError> {
        // Verify the provider exists
        {
            let providers = self.providers.lock().unwrap();
            if !providers.contains_key(name) {
                return Err(LoggingError::ProviderNotFound(format!(
                    "Cannot set default provider: '{}' not found",
                    name
                )));
            }
        }

        // Set the default provider name
        let mut default_name = self.default_provider_name.lock().unwrap();
        *default_name = name.to_string();
        Ok(())
    }

    /// Get the default provider
    pub fn get_default_provider(&self) -> Result<Arc<dyn LoggingProvider>, LoggingError> {
        let name = self.default_provider_name.lock().unwrap().clone();
        self.get_provider(&name)
    }

    /// Create a logger using a specific provider
    pub async fn create_logger(
        &self,
        provider_name: &str,
        config: &LoggingConfig,
    ) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
        let provider = self.get_provider(provider_name)?;
        provider.create_logger(config).await
    }

    /// Create a logger using the default provider
    pub async fn create_default_logger(
        &self,
        config: &LoggingConfig,
    ) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
        let provider = self.get_default_provider()?;
        provider.create_logger(config).await
    }

    /// Create a logger based on the configuration
    pub async fn create_logger_from_config(
        &self,
        config: &LoggingConfig,
    ) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
        // Validate the configuration
        if let Err(err) = config.validate() {
            return Err(LoggingError::ConfigurationError(err));
        }

        // Use the logger type from config or fall back to default
        let provider_name = &config.logger_type;

        match self.get_provider(provider_name) {
            Ok(provider) => provider.create_logger(config).await,
            Err(_) => {
                // If requested provider doesn't exist, use default
                self.create_default_logger(config).await
            }
        }
    }

    /// List all available providers
    pub fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.lock().unwrap();
        providers.keys().cloned().collect()
    }
}
