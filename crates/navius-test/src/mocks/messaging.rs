use async_trait::async_trait;
use futures::{Stream, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use navius_messaging::{
    broker::{MessageBroker, TypedMessageBroker},
    config::{
        BrokerConfig, ConnectionConfig, ConsumerDefaultConfig, EventConfig, PoolConfig,
        PublisherDefaultConfig, RecoveryConfig,
    },
    consumer::{ConsumerHandle, ConsumerOptions},
    error::{ConnectionStatus, MessagingError, MessagingResult},
    message::{Message, MessageFilter, MessageHandler, MessageProcessingResult, ReceivedMessage},
    publisher::PublishOptions,
    topology::{Binding, BindingDestination, Exchange, ExchangeType, Queue},
};

/// Error type for mock messaging operations
#[derive(Error, Debug, Clone)]
pub enum MockMessagingError {
    /// Error connecting to broker
    #[error("Failed to connect to broker: {0}")]
    ConnectionError(String),

    /// Error publishing message
    #[error("Failed to publish message: {0}")]
    PublishError(String),

    /// Error with queues or exchanges
    #[error("Queue or exchange error: {0}")]
    TopologyError(String),

    /// Error with message consumption
    #[error("Consumer error: {0}")]
    ConsumerError(String),

    /// Error with serialization
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Other error
    #[error("Messaging error: {0}")]
    Other(String),
}

/// Result type for mock messaging operations
pub type Result<T> = std::result::Result<T, MockMessagingError>;

/// Delivery mode for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    /// Non-persistent message
    NonPersistent = 1,
    /// Persistent message
    Persistent = 2,
}

impl Default for DeliveryMode {
    fn default() -> Self {
        Self::Persistent
    }
}

/// Status of message publishing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishStatus {
    /// Published successfully
    Published,
    /// Message enqueued for publishing
    Enqueued,
    /// Message was not published
    Failed,
}

/// Message headers/properties
#[derive(Debug, Clone, Default)]
pub struct MessageHeaders {
    /// Key-value headers
    headers: HashMap<String, String>,
}

impl MessageHeaders {
    /// Create new empty headers
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// Add a header
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.headers.insert(key.into(), value.into());
    }

    /// Get a header value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Check if header exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }

    /// Get all headers
    pub fn all(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

/// A generic message
#[derive(Debug, Clone)]
pub struct Message<T> {
    /// Unique message ID
    pub id: String,
    /// Topic for the message
    pub topic: String,
    /// Message payload
    pub payload: T,
    /// Message headers
    pub headers: MessageHeaders,
    /// Timestamp when created
    pub timestamp: SystemTime,
    /// Expiration time
    pub expiration: Option<SystemTime>,
    /// Priority (higher is more important)
    pub priority: Option<u8>,
    /// Delivery mode
    pub delivery_mode: DeliveryMode,
    /// Correlation ID for tracking related messages
    pub correlation_id: Option<String>,
}

impl<T> Message<T> {
    /// Create a new message
    pub fn new(payload: T, topic: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            topic: topic.into(),
            payload,
            headers: MessageHeaders::default(),
            timestamp: SystemTime::now(),
            expiration: None,
            priority: None,
            delivery_mode: DeliveryMode::default(),
            correlation_id: None,
        }
    }

    /// Set a correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }
}

/// A received message
#[derive(Debug, Clone)]
pub struct ReceivedMessage<T> {
    /// The message
    pub message: Message<T>,
    /// Delivery tag for acknowledgment
    pub delivery_tag: u64,
    /// Whether this message was redelivered
    pub redelivered: bool,
}

impl<T> ReceivedMessage<T> {
    /// Create a new received message
    pub fn new(message: Message<T>, delivery_tag: u64, redelivered: bool) -> Self {
        Self {
            message,
            delivery_tag,
            redelivered,
        }
    }

    /// Get the inner message
    pub fn inner(&self) -> &Message<T> {
        &self.message
    }
}

/// Queue configuration
#[derive(Debug, Clone)]
pub struct Queue {
    /// Queue name
    pub name: String,

    /// Whether the queue is durable (survives broker restart)
    pub durable: bool,

    /// Whether the queue is exclusive to one connection
    pub exclusive: bool,

    /// Whether the queue is auto-deleted when no longer in use
    pub auto_delete: bool,

    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

impl Queue {
    /// Create a new queue
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: true,
            exclusive: false,
            auto_delete: false,
            arguments: HashMap::new(),
        }
    }
}

/// Exchange type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeType {
    /// Direct exchange
    Direct,
    /// Topic exchange
    Topic,
    /// Fanout exchange
    Fanout,
    /// Headers exchange
    Headers,
}

/// Exchange configuration
#[derive(Debug, Clone)]
pub struct Exchange {
    /// Exchange name
    pub name: String,
    /// Exchange type
    pub exchange_type: ExchangeType,
    /// Whether durable
    pub durable: bool,
    /// Whether auto-delete
    pub auto_delete: bool,
}

impl Exchange {
    /// Create a new exchange
    pub fn new(name: impl Into<String>, exchange_type: ExchangeType) -> Self {
        Self {
            name: name.into(),
            exchange_type,
            durable: true,
            auto_delete: false,
        }
    }
}

/// Binding destination type
#[derive(Debug, Clone)]
pub enum BindingDestination {
    /// Binding to a queue
    Queue(String),
    /// Binding to an exchange
    Exchange(String),
}

/// Binding between exchange and queue
#[derive(Debug, Clone)]
pub struct Binding {
    /// Source exchange
    pub source: String,
    /// Destination (queue or exchange)
    pub destination: BindingDestination,
    /// Routing key
    pub routing_key: String,
    /// Additional arguments
    pub arguments: HashMap<String, String>,
}

impl Binding {
    /// Create a new binding
    pub fn new(
        source: impl Into<String>,
        destination: BindingDestination,
        routing_key: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            destination,
            routing_key: routing_key.into(),
            arguments: HashMap::new(),
        }
    }
}

/// Metrics about the broker
#[derive(Debug, Clone, Default)]
pub struct BrokerMetrics {
    /// Number of messages published
    pub messages_published: u64,
    /// Number of messages consumed
    pub messages_consumed: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Number of active channels
    pub active_channels: u32,
}

/// Handle for a consumer
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

/// A type-erased stream of messages
type MessageStream<T> = Pin<Box<dyn Stream<Item = Result<ReceivedMessage<T>>> + Send + Unpin>>;

/// Mock message broker implementation
#[derive(Debug, Clone)]
pub struct MockMessageBroker {
    /// Broker ID
    id: String,
    /// Broker name
    name: String,
    /// Whether connected
    connected: Arc<Mutex<bool>>,
    /// Whether connect should fail
    fail_connect: Arc<Mutex<bool>>,
    /// Whether publish should fail
    fail_publish: Arc<Mutex<bool>>,
    /// Declared queues
    queues: Arc<Mutex<HashMap<String, Queue>>>,
    /// Declared exchanges
    exchanges: Arc<Mutex<HashMap<String, Exchange>>>,
    /// Queue bindings
    bindings: Arc<Mutex<Vec<Binding>>>,
    /// Published messages (serialized)
    published_messages: Arc<Mutex<Vec<(String, String, Vec<u8>)>>>,
    /// Active consumers
    consumers: Arc<Mutex<HashMap<String, String>>>,
    /// Broker metrics
    metrics: Arc<Mutex<BrokerMetrics>>,
}

impl Default for MockMessageBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMessageBroker {
    /// Create a new mock message broker
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "mock-broker".to_string(),
            connected: Arc::new(Mutex::new(false)),
            fail_connect: Arc::new(Mutex::new(false)),
            fail_publish: Arc::new(Mutex::new(false)),
            queues: Arc::new(Mutex::new(HashMap::new())),
            exchanges: Arc::new(Mutex::new(HashMap::new())),
            bindings: Arc::new(Mutex::new(Vec::new())),
            published_messages: Arc::new(Mutex::new(Vec::new())),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(Mutex::new(BrokerMetrics::default())),
        }
    }

    /// Register this mock in the registry
    pub fn register() -> Arc<Self> {
        let mock = Arc::new(Self::new());
        // Registration would happen here
        mock
    }

    /// Set whether connect should fail
    pub fn set_connect_failure(&self, should_fail: bool) {
        let mut fail = self.fail_connect.lock().unwrap();
        *fail = should_fail;
    }

    /// Set whether publish should fail
    pub fn set_publish_failure(&self, should_fail: bool) {
        let mut fail = self.fail_publish.lock().unwrap();
        *fail = should_fail;
    }

    /// Get all published messages for a topic
    pub fn get_published_messages_for_topic(&self, topic: &str) -> Vec<Vec<u8>> {
        let messages = self.published_messages.lock().unwrap();
        messages
            .iter()
            .filter(|(t, _, _)| t == topic)
            .map(|(_, _, payload)| payload.clone())
            .collect()
    }

    /// Get all published messages
    pub fn get_all_published_messages(&self) -> Vec<(String, String, Vec<u8>)> {
        let messages = self.published_messages.lock().unwrap();
        messages.clone()
    }

    /// Assert that a message was published to a topic
    pub fn assert_published_to_topic(&self, topic: &str) -> Result<()> {
        let messages = self.published_messages.lock().unwrap();
        if messages.iter().any(|(t, _, _)| t == topic) {
            Ok(())
        } else {
            Err(MockMessagingError::Other(format!(
                "No messages published to topic '{}'",
                topic
            )))
        }
    }

    /// Assert number of messages published
    pub fn assert_message_count(&self, expected: usize) -> Result<()> {
        let messages = self.published_messages.lock().unwrap();
        let count = messages.len();
        if count == expected {
            Ok(())
        } else {
            Err(MockMessagingError::Other(format!(
                "Expected {} messages, found {}",
                expected, count
            )))
        }
    }

    /// Clear all published messages
    pub fn clear_messages(&self) {
        let mut messages = self.published_messages.lock().unwrap();
        messages.clear();
    }

    /// Deserialize a message from bytes
    fn deserialize_message<T: DeserializeOwned>(
        &self,
        topic: &str,
        correlation_id: Option<&str>,
        payload: &[u8],
    ) -> Result<Message<T>> {
        match serde_json::from_slice::<T>(payload) {
            Ok(value) => {
                let mut message = Message::new(value, topic);
                if let Some(id) = correlation_id {
                    message = message.with_correlation_id(id);
                }
                Ok(message)
            }
            Err(e) => Err(MockMessagingError::SerializationError(e.to_string())),
        }
    }
}

pub trait MessagePublisher: Send + Sync {
    fn publish_raw(
        &self,
        queue: String,
        payload: serde_json::Value,
    ) -> BoxFuture<'static, std::result::Result<(), MockMessagingError>>;
}

pub trait MessageConsumer: Send + Sync {
    fn consume_raw(
        &self,
        queue: String,
    ) -> BoxFuture<
        'static,
        std::result::Result<
            BoxStream<'static, std::result::Result<serde_json::Value, MockMessagingError>>,
            MockMessagingError,
        >,
    >;
}

#[async_trait]
impl MessageBroker for MockMessageBroker {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn broker_type(&self) -> &str {
        "mock"
    }

    fn config(&self) -> &BrokerConfig {
        static DEFAULT_CONFIG: Lazy<BrokerConfig> = Lazy::new(|| BrokerConfig {
            id: "mock".to_string(),
            name: "mock".to_string(),
            broker_type: "mock".to_string(),
            connection: ConnectionConfig::default(),
            pool: PoolConfig::default(),
            recovery: RecoveryConfig::default(),
            consumer: ConsumerDefaultConfig::default(),
            publisher: PublisherDefaultConfig::default(),
            default_exchange: None,
            events: EventConfig::default(),
            options: HashMap::new(),
        });
        &DEFAULT_CONFIG
    }

    async fn connect(&self) -> MessagingResult<()> {
        let mut connected = self.connected.lock().unwrap();
        let fail = self.fail_connect.lock().unwrap();
        if *fail {
            return Err(MessagingError::ConnectionError(
                "Simulated connection failure".to_string(),
            ));
        }
        *connected = true;
        Ok(())
    }

    async fn disconnect(&self) -> MessagingResult<()> {
        let mut connected = self.connected.lock().unwrap();
        *connected = false;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    async fn connection_status(&self) -> ConnectionStatus {
        if self.is_connected().await {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }

    async fn metrics(&self) -> BrokerMetrics {
        self.metrics.lock().unwrap().clone()
    }

    async fn declare_queue(&self, queue: &Queue) -> MessagingResult<Queue> {
        let mut queues = self.queues.lock().unwrap();
        queues.insert(queue.name.clone(), queue.clone());
        Ok(queue.clone())
    }

    async fn delete_queue(
        &self,
        name: &str,
        if_unused: bool,
        if_empty: bool,
    ) -> MessagingResult<()> {
        let mut queues = self.queues.lock().unwrap();
        if if_unused {
            // Check if queue has consumers
            let consumer_count = self
                .consumers
                .lock()
                .unwrap()
                .values()
                .filter(|&q| q == name)
                .count();
            if consumer_count > 0 {
                return Err(MessagingError::QueueError(
                    name.to_string(),
                    "Queue has active consumers".to_string(),
                ));
            }
        }
        if if_empty {
            // In a mock implementation, we assume queues are always empty
            queues.remove(name);
            Ok(())
        } else {
            queues.remove(name);
            Ok(())
        }
    }

    async fn purge_queue(&self, name: &str) -> MessagingResult<()> {
        Ok(())
    }

    async fn declare_exchange(&self, exchange: &Exchange) -> MessagingResult<Exchange> {
        let mut exchanges = self.exchanges.lock().unwrap();
        exchanges.insert(exchange.name.clone(), exchange.clone());
        Ok(exchange.clone())
    }

    async fn delete_exchange(&self, name: &str, if_unused: bool) -> MessagingResult<()> {
        let mut exchanges = self.exchanges.lock().unwrap();
        if if_unused {
            // Check if exchange has bindings
            let has_bindings = self
                .bindings
                .lock()
                .unwrap()
                .iter()
                .any(|b| b.source == name);
            if has_bindings {
                return Err(MessagingError::ExchangeError(
                    name.to_string(),
                    "Exchange has active bindings".to_string(),
                ));
            }
        }
        exchanges.remove(name);
        Ok(())
    }

    async fn bind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> MessagingResult<Binding> {
        // Check if queue exists
        if !self.queues.lock().unwrap().contains_key(queue) {
            return Err(MessagingError::QueueError(
                queue.to_string(),
                "Queue does not exist".to_string(),
            ));
        }
        // Check if exchange exists
        if !self.exchanges.lock().unwrap().contains_key(exchange) {
            return Err(MessagingError::ExchangeError(
                exchange.to_string(),
                "Exchange does not exist".to_string(),
            ));
        }

        let binding = Binding {
            source: exchange.to_string(),
            destination: BindingDestination::Queue(queue.to_string()),
            routing_key: routing_key.to_string(),
            arguments: arguments.unwrap_or_default(),
        };
        self.bindings.lock().unwrap().push(binding.clone());
        Ok(binding)
    }

    async fn unbind_queue(
        &self,
        queue: &str,
        exchange: &str,
        routing_key: &str,
    ) -> MessagingResult<()> {
        let mut bindings = self.bindings.lock().unwrap();
        bindings.retain(|b| {
            b.source != exchange
                || !matches!(b.destination, BindingDestination::Queue(ref q) if q == queue)
                || b.routing_key != routing_key
        });
        Ok(())
    }

    async fn bind_exchange(
        &self,
        destination: &str,
        source: &str,
        routing_key: &str,
        arguments: Option<HashMap<String, String>>,
    ) -> MessagingResult<Binding> {
        // Check if exchanges exist
        if !self.exchanges.lock().unwrap().contains_key(source) {
            return Err(MessagingError::ExchangeError(
                source.to_string(),
                "Source exchange does not exist".to_string(),
            ));
        }
        if !self.exchanges.lock().unwrap().contains_key(destination) {
            return Err(MessagingError::ExchangeError(
                destination.to_string(),
                "Destination exchange does not exist".to_string(),
            ));
        }

        let binding = Binding {
            source: source.to_string(),
            destination: BindingDestination::Exchange(destination.to_string()),
            routing_key: routing_key.to_string(),
            arguments: arguments.unwrap_or_default(),
        };
        self.bindings.lock().unwrap().push(binding.clone());
        Ok(binding)
    }

    async fn unbind_exchange(
        &self,
        destination: &str,
        source: &str,
        routing_key: &str,
    ) -> MessagingResult<()> {
        let mut bindings = self.bindings.lock().unwrap();
        bindings.retain(|b| {
            b.source != source
                || !matches!(b.destination, BindingDestination::Exchange(ref e) if e == destination)
                || b.routing_key != routing_key
        });
        Ok(())
    }

    async fn ack(&self, delivery_tag: u64, multiple: bool) -> MessagingResult<()> {
        // Mock implementation - just return Ok
        Ok(())
    }

    async fn reject(&self, delivery_tag: u64, requeue: bool) -> MessagingResult<()> {
        // Mock implementation - just return Ok
        Ok(())
    }

    async fn nack(&self, delivery_tag: u64, multiple: bool, requeue: bool) -> MessagingResult<()> {
        // Mock implementation - just return Ok
        Ok(())
    }

    async fn message_count(&self, queue_name: &str) -> MessagingResult<u32> {
        // Check if queue exists
        if !self.queues.lock().unwrap().contains_key(queue_name) {
            return Err(MessagingError::QueueError(
                queue_name.to_string(),
                "Queue does not exist".to_string(),
            ));
        }
        // Mock implementation - return 0
        Ok(0)
    }

    async fn consumer_count(&self, queue_name: &str) -> MessagingResult<u32> {
        // Check if queue exists
        if !self.queues.lock().unwrap().contains_key(queue_name) {
            return Err(MessagingError::QueueError(
                queue_name.to_string(),
                "Queue does not exist".to_string(),
            ));
        }
        // Return number of consumers for the queue
        Ok(self
            .consumers
            .lock()
            .unwrap()
            .values()
            .filter(|&q| q == queue_name)
            .count() as u32)
    }

    async fn create_reply_queue(&self) -> MessagingResult<Queue> {
        let queue = Queue {
            name: format!("reply-{}", Uuid::new_v4()),
            durable: false,
            auto_delete: true,
            exclusive: true,
            arguments: HashMap::new(),
        };
        self.declare_queue(&queue).await
    }

    async fn ping(&self) -> MessagingResult<Duration> {
        Ok(Duration::from_millis(0))
    }
}

// Helper methods for type-safe publishing and consuming
impl MockMessageBroker {
    pub async fn publish<T: Serialize + Send + Sync>(
        &self,
        queue: &str,
        payload: T,
    ) -> std::result::Result<(), MockMessagingError> {
        let json = serde_json::to_value(payload)?;
        self.publish_raw(queue.to_string(), json).await
    }

    pub async fn consume<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        queue: &str,
    ) -> std::result::Result<
        BoxStream<'_, std::result::Result<T, MockMessagingError>>,
        MockMessagingError,
    > {
        let raw_stream = self.consume_raw(queue.to_string()).await?;
        Ok(Box::pin(raw_stream.map(|result| {
            result.and_then(|value| {
                serde_json::from_value(value)
                    .map_err(|e| MockMessagingError::SerializationError(e.to_string()))
            })
        })))
    }
}

impl From<serde_json::Error> for MockMessagingError {
    fn from(err: serde_json::Error) -> Self {
        MockMessagingError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        message: String,
        count: u32,
    }

    #[tokio::test]
    async fn test_connection() {
        let broker = MockMessageBroker::new();

        // Initially disconnected
        assert!(!broker.is_connected().await);

        // Connect
        broker.connect().await.unwrap();
        assert!(broker.is_connected().await);

        // Disconnect
        broker.disconnect().await.unwrap();
        assert!(!broker.is_connected().await);
    }

    #[tokio::test]
    async fn test_queue_operations() {
        let broker = MockMessageBroker::new();
        broker.connect().await.unwrap();

        // Declare queue
        let queue = Queue::new("test-queue");
        broker.declare_queue(&queue).await.unwrap();

        // Delete queue
        broker.delete_queue("test-queue").await.unwrap();

        // Deleting non-existent queue should fail
        assert!(broker.delete_queue("non-existent").await.is_err());
    }

    #[tokio::test]
    async fn test_publish_and_consume() {
        let broker = MockMessageBroker::new();
        broker.connect().await.unwrap();

        // Set up topology
        let queue = Queue::new("test-queue");
        broker.declare_queue(&queue).await.unwrap();

        let exchange = Exchange::new("test-exchange", ExchangeType::Direct);
        broker.declare_exchange(&exchange).await.unwrap();

        let binding = Binding::new("test-queue", "test-exchange", "test-topic");
        broker.bind_queue(&binding).await.unwrap();

        // Publish a message
        let payload = TestPayload {
            message: "Hello, world!".to_string(),
            count: 42,
        };
        let message = Message::new(payload.clone(), "test-topic");
        broker.publish("test-queue", payload.clone()).await.unwrap();

        // Check message count
        assert_eq!(broker.message_count("test-queue").await.unwrap(), 1);

        // Consume messages
        let (handle, mut stream) = broker.consume::<TestPayload>("test-queue").await.unwrap();

        // Read message from stream
        let received = stream.next().await.unwrap().unwrap();
        assert_eq!(received.message.payload, payload);

        // Acknowledge message
        broker.ack(received.delivery_tag).await.unwrap();
    }

    #[tokio::test]
    async fn test_failure_modes() {
        let broker = MockMessageBroker::new();

        // Set connect to fail
        broker.set_connect_failure(true);
        assert!(broker.connect().await.is_err());

        // Reset and connect
        broker.set_connect_failure(false);
        broker.connect().await.unwrap();

        // Set publish to fail
        broker.set_publish_failure(true);
        let payload = "test payload";
        assert!(broker.publish("test-queue", payload).await.is_err());

        // Reset and publish
        broker.set_publish_failure(false);
        broker.publish("test-queue", payload).await.unwrap();

        // Verify published message
        assert!(broker.assert_published_to_topic("test-topic").await.is_ok());
        assert_eq!(broker.get_all_published_messages().len(), 1);
    }

    #[tokio::test]
    async fn test_bind_queue() {
        let broker = MockMessageBroker::new();
        broker.connect().await.unwrap();

        let binding = Binding::new(
            "test-exchange",
            BindingDestination::Queue("test-queue".to_string()),
            "test-topic",
        );
        assert!(
            broker
                .bind_queue("test-queue", "test-exchange", "test-topic", None)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_bind_exchange() {
        let broker = MockMessageBroker::new();
        broker.connect().await.unwrap();

        let binding = Binding::new(
            "source-exchange",
            BindingDestination::Exchange("dest-exchange".to_string()),
            "test-topic",
        );
        assert!(
            broker
                .bind_exchange("dest-exchange", "source-exchange", "test-topic", None)
                .await
                .is_ok()
        );
    }
}

#[async_trait]
impl<T> TypedMessageBroker<T> for MockMessageBroker
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
{
    async fn publish(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
    ) -> MessagingResult<()> {
        if !self.is_connected().await {
            return Err(MessagingError::ConnectionError("Not connected".to_string()));
        }

        let fail = self.fail_publish.lock().unwrap();
        if *fail {
            return Err(MessagingError::PublishError(
                "Simulated publish failure".to_string(),
            ));
        }

        let serialized = serde_json::to_vec(&message.payload)
            .map_err(|e| MessagingError::SerializationError(e.to_string()))?;

        let mut messages = self.published_messages.lock().unwrap();
        messages.push((
            message.topic.clone(),
            message.correlation_id.clone().unwrap_or_default(),
            serialized,
        ));

        let mut metrics = self.metrics.lock().unwrap();
        metrics.published_messages += 1;

        Ok(())
    }

    async fn publish_with_confirm(
        &self,
        message: &Message<T>,
        options: Option<PublishOptions>,
        timeout: Option<Duration>,
    ) -> MessagingResult<()> {
        self.publish(message, options).await
    }

    async fn subscribe<F>(
        &self,
        queue_name: &str,
        handler: F,
        options: Option<ConsumerOptions>,
    ) -> MessagingResult<ConsumerHandle>
    where
        F: MessageHandler<T> + 'static,
    {
        if !self.is_connected().await {
            return Err(MessagingError::ConnectionError("Not connected".to_string()));
        }

        let queues = self.queues.lock().unwrap();
        if !queues.contains_key(queue_name) {
            return Err(MessagingError::QueueError(
                queue_name.to_string(),
                "Queue does not exist".to_string(),
            ));
        }

        let consumer_id = Uuid::new_v4().to_string();
        let consumer_tag = format!("mock-consumer-{}", consumer_id);

        let (control_tx, _control_rx) = mpsc::channel(1);

        let mut consumers = self.consumers.lock().unwrap();
        consumers.insert(consumer_id.clone(), queue_name.to_string());

        let mut metrics = self.metrics.lock().unwrap();
        metrics.active_consumers += 1;

        Ok(ConsumerHandle {
            tag: consumer_tag,
            queue: queue_name.to_string(),
            control_tx,
        })
    }

    async fn subscribe_filtered<F, M>(
        &self,
        queue_name: &str,
        handler: F,
        filter: M,
        options: Option<ConsumerOptions>,
    ) -> MessagingResult<ConsumerHandle>
    where
        F: MessageHandler<T> + 'static,
        M: MessageFilter<T> + 'static,
    {
        self.subscribe(queue_name, handler, options).await
    }

    async fn consume(
        &self,
        queue_name: &str,
        options: Option<ConsumerOptions>,
    ) -> MessagingResult<
        Box<dyn Stream<Item = Result<ReceivedMessage<T>, MessagingError>> + Send + Unpin>,
    > {
        if !self.is_connected().await {
            return Err(MessagingError::ConnectionError("Not connected".to_string()));
        }

        let queues = self.queues.lock().unwrap();
        if !queues.contains_key(queue_name) {
            return Err(MessagingError::QueueError(
                queue_name.to_string(),
                "Queue does not exist".to_string(),
            ));
        }

        // Return an empty stream since this is a mock
        Ok(Box::new(futures::stream::empty()))
    }
}
