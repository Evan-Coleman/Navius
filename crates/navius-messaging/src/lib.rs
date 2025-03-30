/*!
 * Navius Messaging Framework
 *
 * This crate provides messaging interfaces and abstractions for the Navius framework.
 * It enables application components to exchange messages asynchronously using
 * various messaging providers like RabbitMQ, Kafka, or in-memory implementations.
 *
 * The primary abstractions are:
 * - `MessagePublisher`: For sending messages to topics/queues
 * - `MessageSubscriber`: For receiving messages from topics/queues
 * - `MessageBroker`: For managing connections to messaging systems
 * - `MessageEnvelope`: For wrapping message data with metadata
 */

pub mod broker;
pub mod config;
pub mod error;
pub mod message;
pub mod provider;
pub mod publisher;
pub mod subscriber;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(test)]
pub mod test_utils;

// Re-export primary types for convenience
pub use self::broker::{MessageBroker, MessageBrokerManager};
pub use self::config::MessagingConfig;
pub use self::error::{MessagingError, MessagingResult};
pub use self::message::{Message, MessageEnvelope, MessageProperties};
pub use self::provider::{MessageProvider, MessageProviderRegistry};
pub use self::publisher::{MessagePublisher, PublishOptions};
pub use self::subscriber::{MessageHandler, MessageSubscriber, SubscribeOptions};

/// Version of the navius-messaging crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

//! Navius Messaging provides an abstraction over different messaging systems.
//! 
//! This crate defines the core interfaces for message publishing, subscription,
//! and broker management. It is designed to be extensible, allowing different
//! messaging providers to implement the core interfaces.
//! 
//! # Examples
//! 
//! ```no_run
//! use navius_messaging::config::MessagingConfig;
//! use navius_messaging::manager::MessagingManager;
//! use navius_messaging::provider::MessageProviderRegistry;
//! use std::sync::Arc;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a configuration
//! let config = MessagingConfig::default();
//! 
//! // Create a provider registry
//! let registry = Arc::new(MessageProviderRegistry::new());
//! 
//! // Create the messaging manager
//! let manager = MessagingManager::new(config, registry);
//! 
//! // Get a broker
//! let broker = manager.get_broker("default").await?;
//! 
//! // Create a publisher
//! let publisher = broker.create_publisher().await?;
//! 
//! // Create a subscriber
//! let subscriber = broker.create_subscriber().await?;
//! 
//! // Publish a message
//! let options = navius_messaging::publisher::PublishOptions::default()
//!     .with_exchange("test-exchange")
//!     .with_routing_key("test-key");
//! 
//! publisher.publish("Hello, world!".as_bytes().to_vec(), &options).await?;
//! 
//! // Subscribe to messages
//! let subscribe_options = navius_messaging::subscriber::SubscribeOptions::default()
//!     .with_exchange("test-exchange")
//!     .with_routing_key("test-key");
//! 
//! subscriber.subscribe_fn(&subscribe_options, |message, ack, reject| async move {
//!     println!("Received message: {:?}", message);
//!     ack().await
//! }).await?;
//! 
//! // Cleanup
//! manager.shutdown().await?;
//! # Ok(())
//! # }
//! ```

/// Helper functions for common messaging tasks.
pub mod helpers {
    use crate::config::MessagingConfig;
    use crate::manager::MessagingManager;
    use crate::provider::MessageProviderRegistry;
    use std::sync::Arc;
    
    /// Initialize a messaging manager with the given configuration.
    /// 
    /// # Arguments
    /// * `config` - The messaging configuration
    /// 
    /// # Returns
    /// A new messaging manager
    pub fn init_messaging(config: MessagingConfig) -> MessagingManager {
        MessagingManager::with_config(config)
    }
    
    /// Initialize a messaging manager with the given configuration and providers.
    /// 
    /// # Arguments
    /// * `config` - The messaging configuration
    /// * `providers` - The provider registry
    /// 
    /// # Returns
    /// A new messaging manager
    pub fn init_messaging_with_providers(
        config: MessagingConfig,
        providers: Arc<MessageProviderRegistry>,
    ) -> MessagingManager {
        MessagingManager::new(config, providers)
    }
}
