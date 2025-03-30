use crate::config::MessagingConfig;
use crate::error::MessagingResult;
use crate::publisher::MessagePublisher;
use crate::subscriber::MessageSubscriber;
use async_trait::async_trait;
use std::sync::Arc;

/// Status of a messaging broker connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrokerStatus {
    /// The broker is connected and ready to send/receive messages.
    Connected,

    /// The broker is currently connecting or reconnecting.
    Connecting,

    /// The broker is disconnected.
    Disconnected,

    /// There was an error with the broker connection.
    Error,

    /// The broker connection is closed.
    Closed,
}

impl std::fmt::Display for BrokerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokerStatus::Connected => write!(f, "Connected"),
            BrokerStatus::Connecting => write!(f, "Connecting"),
            BrokerStatus::Disconnected => write!(f, "Disconnected"),
            BrokerStatus::Error => write!(f, "Error"),
            BrokerStatus::Closed => write!(f, "Closed"),
        }
    }
}

/// Statistics for a message broker.
#[derive(Debug, Clone, Default)]
pub struct BrokerStats {
    /// The total number of messages sent.
    pub messages_sent: u64,

    /// The total number of messages received.
    pub messages_received: u64,

    /// The number of current subscribers.
    pub active_subscribers: u64,

    /// The number of active publishers.
    pub active_publishers: u64,

    /// The number of failed message sends.
    pub send_failures: u64,

    /// The number of failed message receives.
    pub receive_failures: u64,

    /// The number of reconnection attempts.
    pub reconnect_attempts: u64,

    /// The number of successful reconnections.
    pub successful_reconnects: u64,
}

/// A trait for message brokers.
///
/// A message broker manages connections to a messaging system and provides
/// publishers and subscribers for sending and receiving messages.
#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Get the name of this broker.
    fn name(&self) -> &str;

    /// Get the current status of the broker.
    fn status(&self) -> BrokerStatus;

    /// Get statistics for the broker.
    fn stats(&self) -> BrokerStats;

    /// Create a new publisher for sending messages.
    async fn create_publisher(&self) -> MessagingResult<Arc<dyn MessagePublisher>>;

    /// Create a new subscriber for receiving messages.
    async fn create_subscriber(&self) -> MessagingResult<Arc<dyn MessageSubscriber>>;

    /// Connect to the broker.
    ///
    /// This will establish a connection to the messaging system.
    async fn connect(&self) -> MessagingResult<()>;

    /// Disconnect from the broker.
    ///
    /// This will close all publishers and subscribers, and disconnect from the messaging system.
    async fn disconnect(&self) -> MessagingResult<()>;

    /// Check if the broker is connected.
    fn is_connected(&self) -> bool {
        self.status() == BrokerStatus::Connected
    }
}

/// A manager for message brokers.
///
/// This is the main entry point for applications that want to use the messaging system.
/// It provides access to brokers and their publishers and subscribers.
#[async_trait]
pub trait MessageBrokerManager: Send + Sync {
    /// Get the broker with the given name.
    ///
    /// If the broker doesn't exist, it will be created.
    async fn get_broker(&self, name: &str) -> MessagingResult<Arc<dyn MessageBroker>>;

    /// Get the default broker.
    async fn default_broker(&self) -> MessagingResult<Arc<dyn MessageBroker>>;

    /// Create a publisher for the default broker.
    async fn create_publisher(&self) -> MessagingResult<Arc<dyn MessagePublisher>> {
        let broker = self.default_broker().await?;
        broker.create_publisher().await
    }

    /// Create a subscriber for the default broker.
    async fn create_subscriber(&self) -> MessagingResult<Arc<dyn MessageSubscriber>> {
        let broker = self.default_broker().await?;
        broker.create_subscriber().await
    }

    /// Create a publisher for the specified broker.
    async fn create_publisher_for(
        &self,
        broker_name: &str,
    ) -> MessagingResult<Arc<dyn MessagePublisher>> {
        let broker = self.get_broker(broker_name).await?;
        broker.create_publisher().await
    }

    /// Create a subscriber for the specified broker.
    async fn create_subscriber_for(
        &self,
        broker_name: &str,
    ) -> MessagingResult<Arc<dyn MessageSubscriber>> {
        let broker = self.get_broker(broker_name).await?;
        broker.create_subscriber().await
    }

    /// Get the configuration for the messaging system.
    fn config(&self) -> &MessagingConfig;

    /// Shut down all brokers and release resources.
    async fn shutdown(&self) -> MessagingResult<()>;
}
