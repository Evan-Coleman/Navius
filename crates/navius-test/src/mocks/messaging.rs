use async_trait::async_trait;
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

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
    /// Whether durable
    pub durable: bool,
    /// Whether exclusive
    pub exclusive: bool,
    /// Whether auto-delete
    pub auto_delete: bool,
}

impl Queue {
    /// Create a new queue
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: true,
            exclusive: false,
            auto_delete: false,
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

/// Binding between exchange and queue
#[derive(Debug, Clone)]
pub struct Binding {
    /// Queue name
    pub queue: String,
    /// Exchange name
    pub exchange: String,
    /// Routing key
    pub routing_key: String,
}

impl Binding {
    /// Create a new binding
    pub fn new(
        queue: impl Into<String>,
        exchange: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> Self {
        Self {
            queue: queue.into(),
            exchange: exchange.into(),
            routing_key: routing_key.into(),
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
#[derive(Debug, Clone)]
pub struct ConsumerHandle {
    /// Consumer ID
    pub id: String,
    /// Consumer tag
    pub tag: String,
    /// Queue being consumed
    pub queue: String,
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

/// Trait for message brokers
#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Get broker ID
    fn id(&self) -> &str;

    /// Get broker name
    fn name(&self) -> &str;

    /// Connect to broker
    async fn connect(&self) -> Result<()>;

    /// Disconnect from broker
    async fn disconnect(&self) -> Result<()>;

    /// Check if connected
    async fn is_connected(&self) -> bool;

    /// Get broker metrics
    async fn metrics(&self) -> BrokerMetrics;

    /// Declare a queue
    async fn declare_queue(&self, queue: &Queue) -> Result<Queue>;

    /// Delete a queue
    async fn delete_queue(&self, name: &str) -> Result<()>;

    /// Declare an exchange
    async fn declare_exchange(&self, exchange: &Exchange) -> Result<Exchange>;

    /// Delete an exchange
    async fn delete_exchange(&self, name: &str) -> Result<()>;

    /// Bind a queue to an exchange
    async fn bind_queue(&self, binding: &Binding) -> Result<()>;

    /// Unbind a queue from an exchange
    async fn unbind_queue(&self, binding: &Binding) -> Result<()>;

    /// Publish a message
    async fn publish<T: Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
    ) -> Result<PublishStatus>;

    /// Consume messages from a queue
    async fn consume<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        queue_name: &str,
    ) -> Result<(ConsumerHandle, MessageStream<T>)>;

    /// Acknowledge a message
    async fn ack(&self, delivery_tag: u64) -> Result<()>;

    /// Reject a message
    async fn reject(&self, delivery_tag: u64, requeue: bool) -> Result<()>;

    /// Get message count for a queue
    async fn message_count(&self, queue_name: &str) -> Result<u32>;
}

#[async_trait]
impl MessageBroker for MockMessageBroker {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn connect(&self) -> Result<()> {
        // Check if connect should fail
        if *self.fail_connect.lock().unwrap() {
            return Err(MockMessagingError::ConnectionError(
                "Failed to connect to mock broker".to_string(),
            ));
        }

        let mut connected = self.connected.lock().unwrap();
        *connected = true;
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        let mut connected = self.connected.lock().unwrap();
        *connected = false;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    async fn metrics(&self) -> BrokerMetrics {
        self.metrics.lock().unwrap().clone()
    }

    async fn declare_queue(&self, queue: &Queue) -> Result<Queue> {
        let mut queues = self.queues.lock().unwrap();
        queues.insert(queue.name.clone(), queue.clone());
        Ok(queue.clone())
    }

    async fn delete_queue(&self, name: &str) -> Result<()> {
        let mut queues = self.queues.lock().unwrap();
        if queues.remove(name).is_some() {
            // Also remove any bindings for this queue
            let mut bindings = self.bindings.lock().unwrap();
            bindings.retain(|b| b.queue != name);
            Ok(())
        } else {
            Err(MockMessagingError::TopologyError(format!(
                "Queue '{}' not found",
                name
            )))
        }
    }

    async fn declare_exchange(&self, exchange: &Exchange) -> Result<Exchange> {
        let mut exchanges = self.exchanges.lock().unwrap();
        exchanges.insert(exchange.name.clone(), exchange.clone());
        Ok(exchange.clone())
    }

    async fn delete_exchange(&self, name: &str) -> Result<()> {
        let mut exchanges = self.exchanges.lock().unwrap();
        if exchanges.remove(name).is_some() {
            // Also remove any bindings for this exchange
            let mut bindings = self.bindings.lock().unwrap();
            bindings.retain(|b| b.exchange != name);
            Ok(())
        } else {
            Err(MockMessagingError::TopologyError(format!(
                "Exchange '{}' not found",
                name
            )))
        }
    }

    async fn bind_queue(&self, binding: &Binding) -> Result<()> {
        // Verify queue and exchange exist
        let queues = self.queues.lock().unwrap();
        let exchanges = self.exchanges.lock().unwrap();

        if !queues.contains_key(&binding.queue) {
            return Err(MockMessagingError::TopologyError(format!(
                "Queue '{}' not found",
                binding.queue
            )));
        }

        if !exchanges.contains_key(&binding.exchange) {
            return Err(MockMessagingError::TopologyError(format!(
                "Exchange '{}' not found",
                binding.exchange
            )));
        }

        // Add binding
        let mut bindings = self.bindings.lock().unwrap();
        bindings.push(binding.clone());
        Ok(())
    }

    async fn unbind_queue(&self, binding: &Binding) -> Result<()> {
        let mut bindings = self.bindings.lock().unwrap();
        let initial_len = bindings.len();
        bindings.retain(|b| {
            !(b.queue == binding.queue
                && b.exchange == binding.exchange
                && b.routing_key == binding.routing_key)
        });

        if bindings.len() < initial_len {
            Ok(())
        } else {
            Err(MockMessagingError::TopologyError(
                "Binding not found".to_string(),
            ))
        }
    }

    async fn publish<T: Serialize + Send + Sync>(
        &self,
        message: &Message<T>,
    ) -> Result<PublishStatus> {
        // Check if publish should fail
        if *self.fail_publish.lock().unwrap() {
            return Err(MockMessagingError::PublishError(
                "Failed to publish message".to_string(),
            ));
        }

        // Check if connected
        if !self.is_connected().await {
            return Err(MockMessagingError::ConnectionError(
                "Not connected to broker".to_string(),
            ));
        }

        // Serialize message payload
        let payload = match serde_json::to_vec(&message.payload) {
            Ok(data) => data,
            Err(e) => {
                return Err(MockMessagingError::SerializationError(e.to_string()));
            }
        };

        // Store message
        {
            let mut messages = self.published_messages.lock().unwrap();
            messages.push((
                message.topic.clone(),
                message.correlation_id.clone().unwrap_or_default(),
                payload,
            ));
        }

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.messages_published += 1;
        }

        Ok(PublishStatus::Published)
    }

    async fn consume<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        queue_name: &str,
    ) -> Result<(ConsumerHandle, MessageStream<T>)> {
        // Check if connected
        if !self.is_connected().await {
            return Err(MockMessagingError::ConnectionError(
                "Not connected to broker".to_string(),
            ));
        }

        // Check if queue exists
        {
            let queues = self.queues.lock().unwrap();
            if !queues.contains_key(queue_name) {
                return Err(MockMessagingError::TopologyError(format!(
                    "Queue '{}' not found",
                    queue_name
                )));
            }
        }

        // Create consumer
        let consumer_id = Uuid::new_v4().to_string();
        let consumer_tag = format!("mock-consumer-{}", consumer_id);

        {
            let mut consumers = self.consumers.lock().unwrap();
            consumers.insert(consumer_id.clone(), queue_name.to_string());
        }

        let handle = ConsumerHandle {
            id: consumer_id,
            tag: consumer_tag,
            queue: queue_name.to_string(),
        };

        // Find bindings for this queue to determine routing keys
        let routing_keys = {
            let bindings = self.bindings.lock().unwrap();
            bindings
                .iter()
                .filter(|b| b.queue == queue_name)
                .map(|b| (b.exchange.clone(), b.routing_key.clone()))
                .collect::<Vec<_>>()
        };

        // Clone published messages for this queue based on routing keys
        let filtered_messages = {
            let messages = self.published_messages.lock().unwrap();
            let mut filtered = Vec::new();

            for (topic, corr_id, payload) in messages.iter() {
                // Simple routing logic - exact match on topic
                if routing_keys.iter().any(|(_, key)| key == topic) {
                    // Create messages for the stream
                    match self.deserialize_message::<T>(topic, Some(corr_id), payload) {
                        Ok(msg) => filtered.push((msg, filtered.len() as u64)),
                        Err(_) => {} // Skip messages that can't be deserialized to type T
                    }
                }
            }

            filtered
        };

        // Create a stream from the filtered messages
        let stream = futures::stream::iter(
            filtered_messages
                .into_iter()
                .map(|(msg, tag)| Ok(ReceivedMessage::new(msg, tag, false))),
        );

        Ok((handle, Box::pin(stream)))
    }

    async fn ack(&self, _delivery_tag: u64) -> Result<()> {
        // In the mock, we don't need to do anything for ack
        Ok(())
    }

    async fn reject(&self, _delivery_tag: u64, _requeue: bool) -> Result<()> {
        // In the mock, we don't need to do anything for reject
        Ok(())
    }

    async fn message_count(&self, queue_name: &str) -> Result<u32> {
        // Count messages that would be routed to this queue
        let binding_keys = {
            let bindings = self.bindings.lock().unwrap();
            bindings
                .iter()
                .filter(|b| b.queue == queue_name)
                .map(|b| b.routing_key.clone())
                .collect::<HashSet<_>>()
        };

        if binding_keys.is_empty() {
            // No bindings for this queue
            return Ok(0);
        }

        let count = {
            let messages = self.published_messages.lock().unwrap();
            messages
                .iter()
                .filter(|(topic, _, _)| binding_keys.contains(topic))
                .count() as u32
        };

        Ok(count)
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
        broker.publish(&message).await.unwrap();

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
        let message = Message::new("test payload", "test-topic");
        assert!(broker.publish(&message).await.is_err());

        // Reset and publish
        broker.set_publish_failure(false);
        broker.publish(&message).await.unwrap();

        // Verify published message
        assert!(broker.assert_published_to_topic("test-topic").await.is_ok());
        assert_eq!(broker.get_all_published_messages().len(), 1);
    }
}
