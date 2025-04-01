use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use futures::stream::BoxStream;
use futures::stream::StreamExt;
use futures::{Stream, StreamExt as _};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use thiserror::Error;
use uuid::Uuid;

/// Error type for mock event operations
#[derive(Error, Debug, Clone)]
pub enum MockEventError {
    /// Error publishing an event
    #[error("Failed to publish event: {0}")]
    PublishError(String),

    /// Error subscribing to an event
    #[error("Failed to subscribe to event: {0}")]
    SubscribeError(String),

    /// Error with topic operations
    #[error("Topic error: {0}")]
    TopicError(String),

    /// Error with subscription operations
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Error serializing or deserializing event
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Other error
    #[error("Event error: {0}")]
    Other(String),
}

/// Result type for mock event operations
pub type Result<T> = std::result::Result<T, MockEventError>;

/// Priority levels for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// Low priority
    Low = 0,
    /// Normal priority (default)
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Status of event delivery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Successfully delivered
    Delivered,
    /// Delivered to some subscribers
    PartiallyDelivered,
    /// Queued for later delivery
    Queued,
    /// Failed to deliver
    Failed,
}

/// A serializable event with metadata and payload
#[derive(Debug, Clone)]
pub struct Event<T> {
    /// Unique identifier
    pub id: Uuid,
    /// Type of event
    pub event_type: String,
    /// Topic the event is published to
    pub topic: String,
    /// When the event was created
    pub created_at: DateTime<Utc>,
    /// Priority of the event
    pub priority: EventPriority,
    /// Source of the event
    pub source: String,
    /// Optional correlation ID for related events
    pub correlation_id: Option<String>,
    /// Metadata for the event
    pub metadata: HashMap<String, String>,
    /// Event payload
    pub payload: T,
}

impl<T> Event<T> {
    /// Create a new event
    pub fn new(
        event_type: impl Into<String>,
        topic: impl Into<String>,
        source: impl Into<String>,
        payload: T,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            topic: topic.into(),
            created_at: Utc::now(),
            priority: EventPriority::Normal,
            source: source.into(),
            correlation_id: None,
            metadata: HashMap::new(),
            payload,
        }
    }

    /// Set the priority of the event
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the correlation ID for the event
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// An event envelope with serialized payload
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    /// Unique identifier
    pub id: Uuid,
    /// Type of event
    pub event_type: String,
    /// Topic the event is published to
    pub topic: String,
    /// When the event was created
    pub created_at: DateTime<Utc>,
    /// Priority of the event
    pub priority: EventPriority,
    /// Source of the event
    pub source: String,
    /// Optional correlation ID for related events
    pub correlation_id: Option<String>,
    /// Metadata for the event
    pub metadata: HashMap<String, String>,
    /// Serialized event payload
    pub payload: serde_json::Value,
}

/// Subscription information
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// Unique identifier
    pub id: String,
    /// Topic subscribed to
    pub topic: String,
    /// When the subscription was created
    pub created_at: DateTime<Utc>,
    /// Number of events delivered
    pub events_delivered: u64,
}

/// Topic information
#[derive(Debug, Clone)]
pub struct TopicInfo {
    /// Name of the topic
    pub name: String,
    /// Number of subscribers
    pub subscriber_count: usize,
    /// Number of retained events
    pub event_count: usize,
}

/// A type-erased stream of events
pub type EventStream<T> = Pin<Box<dyn Stream<Item = Result<Event<T>>> + Send>>;

/// A mock event broker for testing
#[derive(Debug, Clone)]
pub struct MockEventBroker {
    /// Published events
    events: Arc<Mutex<Vec<EventEnvelope>>>,
    /// Registered topics
    topics: Arc<Mutex<HashSet<String>>>,
    /// Whether topic creation should fail
    fail_topic_creation: Arc<Mutex<bool>>,
    /// Whether publishing should fail
    fail_publishing: Arc<Mutex<bool>>,
    /// Whether subscribing should fail
    fail_subscribing: Arc<Mutex<bool>>,
    /// Mock broker ID
    id: String,
    /// List of subscriptions
    subscriptions: Arc<Mutex<Vec<SubscriptionInfo>>>,
}

impl Default for MockEventBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEventBroker {
    /// Create a new mock event broker
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            topics: Arc::new(Mutex::new(HashSet::new())),
            fail_topic_creation: Arc::new(Mutex::new(false)),
            fail_publishing: Arc::new(Mutex::new(false)),
            fail_subscribing: Arc::new(Mutex::new(false)),
            id: Uuid::new_v4().to_string(),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register this mock in the registry
    pub fn register() -> Arc<Self> {
        let mock = Arc::new(Self::new());
        // Registration with a global registry would happen here
        mock
    }

    /// Set whether topic creation should fail
    pub fn set_topic_creation_failure(&self, should_fail: bool) {
        let mut fail = self.fail_topic_creation.lock().unwrap();
        *fail = should_fail;
    }

    /// Set whether publishing should fail
    pub fn set_publishing_failure(&self, should_fail: bool) {
        let mut fail = self.fail_publishing.lock().unwrap();
        *fail = should_fail;
    }

    /// Set whether subscribing should fail
    pub fn set_subscribing_failure(&self, should_fail: bool) {
        let mut fail = self.fail_subscribing.lock().unwrap();
        *fail = should_fail;
    }

    /// Get all published events
    pub fn get_events(&self) -> Vec<EventEnvelope> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    /// Get events filtered by topic
    pub fn get_events_by_topic(&self, topic: &str) -> Vec<EventEnvelope> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.topic == topic)
            .cloned()
            .collect()
    }

    /// Get events filtered by type
    pub fn get_events_by_type(&self, event_type: &str) -> Vec<EventEnvelope> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events filtered by correlation ID
    pub fn get_events_by_correlation_id(&self, correlation_id: &str) -> Vec<EventEnvelope> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.correlation_id.as_deref() == Some(correlation_id))
            .cloned()
            .collect()
    }

    /// Clear all published events
    pub fn clear_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }

    /// Assert that an event of the specified type was published
    pub fn assert_event_published(&self, event_type: &str) -> Result<()> {
        let events = self.events.lock().unwrap();
        if events.iter().any(|e| e.event_type == event_type) {
            Ok(())
        } else {
            Err(MockEventError::Other(format!(
                "No event of type '{}' was published",
                event_type
            )))
        }
    }

    /// Assert that a specific number of events were published
    pub fn assert_event_count(&self, count: usize) -> Result<()> {
        let events = self.events.lock().unwrap();
        let actual = events.len();
        if actual == count {
            Ok(())
        } else {
            Err(MockEventError::Other(format!(
                "Expected {} events, found {}",
                count, actual
            )))
        }
    }

    /// Convert an envelope to a typed event
    pub fn envelope_to_event<T: DeserializeOwned>(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<Event<T>> {
        match serde_json::from_value(envelope.payload.clone()) {
            Ok(payload) => Ok(Event {
                id: envelope.id,
                event_type: envelope.event_type.clone(),
                topic: envelope.topic.clone(),
                created_at: envelope.created_at,
                priority: envelope.priority,
                source: envelope.source.clone(),
                correlation_id: envelope.correlation_id.clone(),
                metadata: envelope.metadata.clone(),
                payload,
            }),
            Err(e) => Err(MockEventError::SerializationError(e.to_string())),
        }
    }
}

pub trait EventPublisher: Send + Sync {
    fn publish_raw(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> BoxFuture<'static, std::result::Result<(), MockEventError>>;
}

pub trait EventSubscriber: Send + Sync {
    fn subscribe_raw(
        &self,
        event_type: &str,
    ) -> BoxFuture<
        'static,
        std::result::Result<
            BoxStream<'static, std::result::Result<serde_json::Value, MockEventError>>,
            MockEventError,
        >,
    >;
}

pub trait EventBroker: EventPublisher + EventSubscriber {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
}

#[async_trait]
impl EventPublisher for MockEventBroker {
    fn publish_raw(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> BoxFuture<'static, std::result::Result<(), MockEventError>> {
        // Clone all needed values to avoid lifetime issues
        let event_type = event_type.to_owned();
        let clone = self.clone();

        Box::pin(async move {
            // Now use the cloned self and owned strings
            clone.publish_raw(&event_type, payload).await
        })
    }
}

#[async_trait]
impl EventSubscriber for MockEventBroker {
    fn subscribe_raw(
        &self,
        event_type: &str,
    ) -> BoxFuture<
        'static,
        std::result::Result<
            BoxStream<'static, std::result::Result<serde_json::Value, MockEventError>>,
            MockEventError,
        >,
    > {
        // Clone all needed values to avoid lifetime issues
        let event_type = event_type.to_owned();
        let clone = self.clone();

        Box::pin(async move {
            // Now use the cloned self and owned strings
            clone.subscribe_raw(&event_type).await
        })
    }
}

impl EventBroker for MockEventBroker {
    fn id(&self) -> &str {
        "mock_event_broker"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }
}

// Helper methods for type-safe publishing and subscribing
impl MockEventBroker {
    pub async fn publish<T: Serialize + Send + Sync + 'static>(
        &self,
        event_type: &str,
        payload: T,
    ) -> std::result::Result<(), MockEventError> {
        let json = serde_json::to_value(payload)?;
        self.publish_raw(event_type, json).await
    }

    pub async fn subscribe<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        event_type: &str,
    ) -> std::result::Result<BoxStream<'_, std::result::Result<T, MockEventError>>, MockEventError>
    {
        let raw_stream = self.subscribe_raw(event_type).await?;
        Ok(Box::pin(raw_stream.map(|result| {
            result.and_then(|value| {
                serde_json::from_value(value)
                    .map_err(|e| MockEventError::SerializationError(e.to_string()))
            })
        })))
    }
}

impl From<serde_json::Error> for MockEventError {
    fn from(err: serde_json::Error) -> Self {
        MockEventError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        message: String,
        count: u32,
    }

    #[tokio::test]
    async fn test_create_topic() {
        let broker = MockEventBroker::new();

        // Create a topic
        broker.create_topic("test-topic").await.unwrap();

        // Check that topic exists
        assert!(broker.topic_exists("test-topic").await.unwrap());
    }

    #[tokio::test]
    async fn test_publish_event() {
        let broker = MockEventBroker::new();

        // Create event
        let event = Event::new(
            "test-event",
            "test-topic",
            "test-source",
            TestEvent {
                message: "Hello, world!".to_string(),
                count: 42,
            },
        );

        // Publish event
        let status = broker.publish("test-event", event.payload).await.unwrap();
        assert_eq!(status, DeliveryStatus::Delivered);

        // Check that event was stored
        let events = broker.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "test-event");
        assert_eq!(events[0].topic, "test-topic");
    }

    #[tokio::test]
    async fn test_subscribe() {
        let broker = MockEventBroker::new();

        // Create topic
        broker.create_topic("test-topic").await.unwrap();

        // Subscribe to topic
        let (subscription_id, mut stream) =
            broker.subscribe::<TestEvent>("test-topic").await.unwrap();

        // Publish event
        broker
            .publish(
                "test-event",
                TestEvent {
                    message: "Hello, subscriber!".to_string(),
                    count: 1,
                },
            )
            .await
            .unwrap();

        // Read event from stream
        let event = stream.next().await.unwrap().unwrap();
        assert_eq!(event.event_type, "test-event");
        assert_eq!(event.payload.message, "Hello, subscriber!");

        // Check subscription info
        let info = broker
            .get_subscription_info(&subscription_id)
            .await
            .unwrap();
        assert_eq!(info.topic, "test-topic");
    }

    #[tokio::test]
    async fn test_failure_modes() {
        let broker = MockEventBroker::new();

        // Set topic creation to fail
        broker.set_topic_creation_failure(true);

        // Creating topic should fail
        assert!(broker.create_topic("test-topic").await.is_err());

        // Reset failure flag
        broker.set_topic_creation_failure(false);

        // Creating topic should now succeed
        broker.create_topic("test-topic").await.unwrap();

        // Set publishing to fail
        broker.set_publishing_failure(true);

        // Publishing should fail
        let payload = TestEvent {
            message: "This should fail".to_string(),
            count: 0,
        };
        assert!(broker.publish("test-event", payload).await.is_err());
    }
}
