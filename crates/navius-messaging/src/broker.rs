use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::Stream;
use tokio::sync::mpsc;

use crate::config::BrokerConfig;
use crate::consumer::ConsumerOptions;
use crate::error::{ConnectionStatus, MessagingError, MessagingResult};
use crate::message::{
    Message, MessageFilter, MessageHandler, MessageProcessingResult, ReceivedMessage,
};
use crate::publisher::PublishOptions;
use crate::topology::{Binding, Exchange, ExchangeType, Queue};

/// Trait defining a message broker that can send and receive messages
#[async_trait]
pub trait MessageBroker: Send + Sync + 'static {
    /// Get the broker ID
    fn id(&self) -> &str;

    /// Get the broker name
    fn name(&self) -> &str;

    /// Get the broker type
    fn broker_type(&self) -> &str;

    /// Get the broker configuration
    fn config(&self) -> &BrokerConfig;

    /// Connect to the message broker
    async fn connect(&self) -> MessagingResult<()>;

    /// Disconnect from the message broker
    async fn disconnect(&self) -> MessagingResult<()>;

    /// Check if connected to the broker
    async fn is_connected(&self) -> bool;

    /// Get the current connection status
    async fn connection_status(&self) -> ConnectionStatus;

    /// Get connection metrics
    async fn metrics(&self) -> BrokerMetrics;

    /// Declare a queue
    async fn declare_queue(&self, queue: &Queue) -> MessagingResult<Queue>;

    /// Delete a queue
    async fn delete_queue(
        &self,
        name: &str,
        if_unused: bool,
        if_empty: bool,
    ) -> MessagingResult<()>;

    /// Purge a queue
    async fn purge_queue(&self, name: &str) -> MessagingResult<()>;

    /// Declare an exchange
    async fn declare_exchange(&self, exchange: &Exchange) -> MessagingResult<Exchange>;

    /// Delete an exchange
    async fn delete_exchange(&self, name: &str, if_unused: bool) -> MessagingResult<()>;

    /// Bind a queue to an exchange
    async fn bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> MessagingResult<Binding>;

    /// Unbind a queue from an exchange
    async fn unbind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
    ) -> MessagingResult<()>;

    /// Bind an exchange to another exchange
    async fn bind_exchange(
        &self,
        destination: &str,
        source: &str,
        routing_key: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> MessagingResult<Binding>;

    /// Unbind an exchange from another exchange
    async fn unbind_exchange(
        &self,
        destination: &str,
        source: &str,
        routing_key: &str,
    ) -> MessagingResult<()>;

    /// Publish a message
    async fn publish<T: serde::Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<()>;

    /// Publish a message and wait for confirmation
    async fn publish_with_confirm<T: serde::Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<()>;

    /// Subscribe to messages
    async fn subscribe<T, F>(
        &self,
        queue_name: &str,
        handler: F,
        options: Option<ConsumerOptions>,
    ) -> MessagingResult<ConsumerHandle>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
        F: MessageHandler<T> + 'static;

    /// Subscribe to messages with a filter
    async fn subscribe_filtered<T, F, M>(
        &self,
        queue_name: &str,
        handler: F,
        filter: M,
        options: Option<ConsumerOptions>,
    ) -> MessagingResult<ConsumerHandle>
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
        F: MessageHandler<T> + 'static,
        M: MessageFilter<T> + 'static;

    /// Get a stream of messages
    async fn consume<T>(
        &self,
        queue_name: &str,
        options: Option<ConsumerOptions>,
    ) -> MessagingResult<
        Box<dyn Stream<Item = Result<ReceivedMessage<T>, MessagingError>> + Send + Unpin>,
    >
    where
        T: for<'de> serde::Deserialize<'de> + Send + Sync + 'static;

    /// Acknowledge a message
    async fn ack(&self, delivery_tag: u64, multiple: bool) -> MessagingResult<()>;

    /// Reject a message
    async fn reject(&self, delivery_tag: u64, requeue: bool) -> MessagingResult<()>;

    /// Negative acknowledgment
    async fn nack(&self, delivery_tag: u64, multiple: bool, requeue: bool) -> MessagingResult<()>;

    /// Get the number of messages in a queue
    async fn message_count(&self, queue_name: &str) -> MessagingResult<u32>;

    /// Get the number of consumers for a queue
    async fn consumer_count(&self, queue_name: &str) -> MessagingResult<u32>;

    /// Create a temporary reply queue
    async fn create_reply_queue(&self) -> MessagingResult<Queue>;

    /// Ping the broker to check connectivity
    async fn ping(&self) -> MessagingResult<Duration>;
}

/// Metrics about the broker and its connections
#[derive(Debug, Clone, Default)]
pub struct BrokerMetrics {
    /// Number of active connections
    pub active_connections: usize,

    /// Number of active channels
    pub active_channels: usize,

    /// Number of active publishers
    pub active_publishers: usize,

    /// Number of active consumers
    pub active_consumers: usize,

    /// Number of messages published
    pub published_messages: u64,

    /// Number of messages consumed
    pub consumed_messages: u64,

    /// Number of messages acknowledged
    pub acknowledged_messages: u64,

    /// Number of messages rejected
    pub rejected_messages: u64,

    /// Number of connection errors
    pub connection_errors: u64,

    /// Number of publish errors
    pub publish_errors: u64,

    /// Number of consume errors
    pub consume_errors: u64,
}

/// Factory for creating message brokers
#[async_trait]
pub trait MessageBrokerFactory: Send + Sync {
    /// Create a new message broker
    async fn create_broker(&self, config: BrokerConfig) -> MessagingResult<Arc<dyn MessageBroker>>;
}

/// Handle for a message consumer
#[derive(Debug)]
pub struct ConsumerHandle {
    /// Consumer tag
    pub tag: String,

    /// Queue being consumed
    pub queue: String,

    /// Control channel for the consumer
    control_tx: mpsc::Sender<ConsumerControl>,
}

impl ConsumerHandle {
    /// Create a new consumer handle
    pub fn new(
        tag: impl Into<String>,
        queue: impl Into<String>,
        control_tx: mpsc::Sender<ConsumerControl>,
    ) -> Self {
        Self {
            tag: tag.into(),
            queue: queue.into(),
            control_tx,
        }
    }

    /// Cancel the consumer
    pub async fn cancel(&self) -> MessagingResult<()> {
        self.control_tx
            .send(ConsumerControl::Cancel)
            .await
            .map_err(|_| MessagingError::ConsumerError("Failed to send cancel command".into()))?;
        Ok(())
    }

    /// Pause the consumer
    pub async fn pause(&self) -> MessagingResult<()> {
        self.control_tx
            .send(ConsumerControl::Pause)
            .await
            .map_err(|_| MessagingError::ConsumerError("Failed to send pause command".into()))?;
        Ok(())
    }

    /// Resume the consumer
    pub async fn resume(&self) -> MessagingResult<()> {
        self.control_tx
            .send(ConsumerControl::Resume)
            .await
            .map_err(|_| MessagingError::ConsumerError("Failed to send resume command".into()))?;
        Ok(())
    }

    /// Update consumer prefetch count
    pub async fn set_prefetch(&self, prefetch: u16) -> MessagingResult<()> {
        self.control_tx
            .send(ConsumerControl::SetPrefetch(prefetch))
            .await
            .map_err(|_| {
                MessagingError::ConsumerError("Failed to send set prefetch command".into())
            })?;
        Ok(())
    }
}

/// Control commands for consumers
#[derive(Debug)]
pub enum ConsumerControl {
    /// Cancel the consumer
    Cancel,

    /// Pause consuming
    Pause,

    /// Resume consuming
    Resume,

    /// Set prefetch count
    SetPrefetch(u16),
}

/// Helper for building common messaging topologies
pub struct TopologyBuilder {
    broker: Arc<dyn MessageBroker>,
}

impl TopologyBuilder {
    /// Create a new topology builder
    pub fn new(broker: Arc<dyn MessageBroker>) -> Self {
        Self { broker }
    }

    /// Build a simple pub/sub topology
    pub async fn build_pubsub(
        &self,
        exchange_name: &str,
        queue_names: &[&str],
    ) -> MessagingResult<()> {
        // Create a fanout exchange
        let exchange = Exchange::new(exchange_name, ExchangeType::Fanout);
        self.broker.declare_exchange(&exchange).await?;

        // Create queues and bind them to the exchange
        for &queue_name in queue_names {
            let queue = Queue::new(queue_name);
            self.broker.declare_queue(&queue).await?;
            self.broker
                .bind_queue(queue_name, exchange_name, "", None)
                .await?;
        }

        Ok(())
    }

    /// Build a direct topic topology
    pub async fn build_topic(
        &self,
        exchange_name: &str,
        topics: &[(&str, &str)],
    ) -> MessagingResult<()> {
        // Create a topic exchange
        let exchange = Exchange::new(exchange_name, ExchangeType::Topic);
        self.broker.declare_exchange(&exchange).await?;

        // Create queues and bind them to the exchange with routing keys
        for &(queue_name, routing_key) in topics {
            let queue = Queue::new(queue_name);
            self.broker.declare_queue(&queue).await?;
            self.broker
                .bind_queue(queue_name, exchange_name, routing_key, None)
                .await?;
        }

        Ok(())
    }

    /// Build a request-reply topology
    pub async fn build_request_reply(
        &self,
        request_exchange: &str,
        reply_exchange: &str,
        services: &[&str],
    ) -> MessagingResult<()> {
        // Create direct exchanges for request and reply
        let req_exchange = Exchange::new(request_exchange, ExchangeType::Direct);
        let reply_exchange = Exchange::new(reply_exchange, ExchangeType::Direct);

        self.broker.declare_exchange(&req_exchange).await?;
        self.broker.declare_exchange(&reply_exchange).await?;

        // Create service queues
        for &service in services {
            let queue = Queue::new(service);
            self.broker.declare_queue(&queue).await?;
            self.broker
                .bind_queue(service, request_exchange, service, None)
                .await?;
        }

        Ok(())
    }

    /// Build a work queue topology
    pub async fn build_work_queue(&self, queue_name: &str, durable: bool) -> MessagingResult<()> {
        let mut queue = Queue::new(queue_name);

        if durable {
            queue = queue.durable(true);
        }

        self.broker.declare_queue(&queue).await?;
        Ok(())
    }
}
