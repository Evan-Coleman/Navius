use crate::broker::{MessageBroker, MessageBrokerManager};
use crate::config::MessagingConfig;
use crate::error::MessagingResult;
use crate::provider::MessageProviderRegistry;
use crate::publisher::MessagePublisher;
use crate::subscriber::MessageSubscriber;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

/// Manager for messaging brokers.
#[derive(Clone)]
pub struct MessagingManager {
    /// The configuration for the messaging system
    config: Arc<MessagingConfig>,

    /// The provider registry
    providers: Arc<MessageProviderRegistry>,

    /// The managed brokers
    brokers: Arc<RwLock<HashMap<String, Arc<dyn MessageBroker>>>>,
}

impl MessagingManager {
    /// Create a new MessagingManager.
    ///
    /// # Arguments
    /// * `config` - The messaging configuration
    /// * `providers` - The provider registry
    ///
    /// # Returns
    /// A new messaging manager
    pub fn new(config: MessagingConfig, providers: Arc<MessageProviderRegistry>) -> Self {
        Self {
            config: Arc::new(config),
            providers,
            brokers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new MessagingManager with the given configuration and an empty provider registry.
    ///
    /// # Arguments
    /// * `config` - The messaging configuration
    ///
    /// # Returns
    /// A new messaging manager
    pub fn with_config(config: MessagingConfig) -> Self {
        Self::new(config, Arc::new(MessageProviderRegistry::new()))
    }

    /// Register a messaging provider with this manager.
    ///
    /// # Arguments
    /// * `provider` - The provider to register
    pub fn register_provider(&self, provider: Arc<dyn crate::provider::MessageProvider>) {
        self.providers.register(provider);
    }

    /// Create and initialize a broker.
    ///
    /// # Arguments
    /// * `name` - The name for the broker
    ///
    /// # Returns
    /// A result containing the broker if successful
    pub async fn create_broker(&self, name: &str) -> MessagingResult<Arc<dyn MessageBroker>> {
        let config = self.config.get_broker_config(name);
        let broker = self.providers.create_broker(name, config).await?;

        // Store the broker
        let mut brokers = self.brokers.write().unwrap();
        brokers.insert(name.to_string(), broker.clone());

        Ok(broker)
    }

    /// Get a broker by name. If the broker does not exist, it will be created.
    ///
    /// # Arguments
    /// * `name` - The name of the broker to get
    ///
    /// # Returns
    /// A result containing the broker if successful
    pub async fn get_broker(&self, name: &str) -> MessagingResult<Arc<dyn MessageBroker>> {
        // First try to get the broker from the cache
        {
            let brokers = self.brokers.read().unwrap();
            if let Some(broker) = brokers.get(name) {
                return Ok(broker.clone());
            }
        }

        // If not found, create it
        self.create_broker(name).await
    }

    /// Create a new message publisher for the specified broker.
    ///
    /// # Arguments
    /// * `broker_name` - The name of the broker to use
    ///
    /// # Returns
    /// A result containing the publisher if successful
    pub async fn create_publisher(
        &self,
        broker_name: &str,
    ) -> MessagingResult<Arc<dyn MessagePublisher>> {
        let broker = self.get_broker(broker_name).await?;
        broker.create_publisher().await
    }

    /// Create a new message subscriber for the specified broker.
    ///
    /// # Arguments
    /// * `broker_name` - The name of the broker to use
    ///
    /// # Returns
    /// A result containing the subscriber if successful
    pub async fn create_subscriber(
        &self,
        broker_name: &str,
    ) -> MessagingResult<Arc<dyn MessageSubscriber>> {
        let broker = self.get_broker(broker_name).await?;
        broker.create_subscriber().await
    }

    /// Get the configuration for this manager.
    ///
    /// # Returns
    /// The configuration
    pub fn config(&self) -> &MessagingConfig {
        &self.config
    }

    /// Shut down all brokers managed by this manager.
    ///
    /// # Returns
    /// A result indicating success or failure
    pub async fn shutdown(&self) -> MessagingResult<()> {
        info!("Shutting down messaging manager");
        let brokers = {
            let brokers_guard = self.brokers.read().unwrap();
            brokers_guard.values().cloned().collect::<Vec<_>>()
        };

        let mut last_error = None;

        for broker in brokers {
            match broker.disconnect().await {
                Ok(_) => {
                    debug!("Successfully disconnected broker: {}", broker.name());
                }
                Err(err) => {
                    error!("Error disconnecting broker {}: {}", broker.name(), err);
                    last_error = Some(err);
                }
            }
        }

        // Clear the brokers map
        {
            let mut brokers_guard = self.brokers.write().unwrap();
            brokers_guard.clear();
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl MessageBrokerManager for MessagingManager {
    async fn get_broker(&self, name: &str) -> MessagingResult<Arc<dyn MessageBroker>> {
        self.get_broker(name).await
    }

    async fn create_publisher(
        &self,
        broker_name: &str,
    ) -> MessagingResult<Arc<dyn MessagePublisher>> {
        self.create_publisher(broker_name).await
    }

    async fn create_subscriber(
        &self,
        broker_name: &str,
    ) -> MessagingResult<Arc<dyn MessageSubscriber>> {
        self.create_subscriber(broker_name).await
    }

    fn config(&self) -> &MessagingConfig {
        &self.config
    }

    async fn shutdown(&self) -> MessagingResult<()> {
        self.shutdown().await
    }
}
