use async_trait::async_trait;
use chrono::{DateTime, Utc};
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

/// Trait for an event broker
#[async_trait]
pub trait EventBroker: Send + Sync {
    /// Get information about the broker
    async fn get_info(&self) -> Result<BrokerInfo>;

    /// List all topics
    async fn list_topics(&self) -> Result<Vec<TopicInfo>>;

    /// Check if a topic exists
    async fn topic_exists(&self, topic: &str) -> Result<bool>;

    /// Create a new topic
    async fn create_topic(&self, topic: &str) -> Result<()>;

    /// Delete a topic
    async fn delete_topic(&self, topic: &str) -> Result<()>;

    /// Publish an event
    async fn publish<T: Serialize + Send + Sync + 'static>(
        &self,
        event: Event<T>,
    ) -> Result<DeliveryStatus>;

    /// Subscribe to events
    async fn subscribe<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        topic: &str,
    ) -> Result<(String, EventStream<T>)>;

    /// Unsubscribe from events
    async fn unsubscribe(&self, subscription_id: &str) -> Result<bool>;

    /// Get information about a subscription
    async fn get_subscription_info(&self, subscription_id: &str) -> Result<SubscriptionInfo>;

    /// List all subscriptions
    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionInfo>>;

    /// Get recent events for a topic
    async fn get_recent_events(&self, topic: &str, limit: usize) -> Result<Vec<EventEnvelope>>;
}

/// Information about the broker
#[derive(Debug, Clone)]
pub struct BrokerInfo {
    /// Unique identifier
    pub id: String,
    /// Number of topics
    pub topic_count: usize,
    /// Number of subscriptions
    pub subscription_count: usize,
}

#[async_trait]
impl EventBroker for MockEventBroker {
    async fn get_info(&self) -> Result<BrokerInfo> {
        let topics = self.topics.lock().unwrap();
        let subscriptions = self.subscriptions.lock().unwrap();

        Ok(BrokerInfo {
            id: self.id.clone(),
            topic_count: topics.len(),
            subscription_count: subscriptions.len(),
        })
    }

    async fn list_topics(&self) -> Result<Vec<TopicInfo>> {
        let topics = self.topics.lock().unwrap();
        let events = self.events.lock().unwrap();
        let subscriptions = self.subscriptions.lock().unwrap();

        let mut topic_infos = Vec::new();
        for topic_name in topics.iter() {
            let event_count = events.iter().filter(|e| e.topic == *topic_name).count();
            let subscriber_count = subscriptions
                .iter()
                .filter(|s| s.topic == *topic_name)
                .count();

            topic_infos.push(TopicInfo {
                name: topic_name.clone(),
                subscriber_count,
                event_count,
            });
        }

        Ok(topic_infos)
    }

    async fn topic_exists(&self, topic: &str) -> Result<bool> {
        let topics = self.topics.lock().unwrap();
        Ok(topics.contains(topic))
    }

    async fn create_topic(&self, topic: &str) -> Result<()> {
        // Check if topic creation should fail
        if *self.fail_topic_creation.lock().unwrap() {
            return Err(MockEventError::TopicError(format!(
                "Failed to create topic '{}'",
                topic
            )));
        }

        let mut topics = self.topics.lock().unwrap();
        topics.insert(topic.to_string());
        Ok(())
    }

    async fn delete_topic(&self, topic: &str) -> Result<()> {
        let mut topics = self.topics.lock().unwrap();
        if topics.remove(topic) {
            // Remove all events for this topic
            let mut events = self.events.lock().unwrap();
            events.retain(|e| e.topic != topic);

            // Remove all subscriptions for this topic
            let mut subscriptions = self.subscriptions.lock().unwrap();
            subscriptions.retain(|s| s.topic != topic);

            Ok(())
        } else {
            Err(MockEventError::TopicError(format!(
                "Topic '{}' not found",
                topic
            )))
        }
    }

    async fn publish<T: Serialize + Send + Sync + 'static>(
        &self,
        event: Event<T>,
    ) -> Result<DeliveryStatus> {
        // Check if publishing should fail
        if *self.fail_publishing.lock().unwrap() {
            return Err(MockEventError::PublishError(format!(
                "Failed to publish event of type '{}'",
                event.event_type
            )));
        }

        // Check if topic exists, create it if not
        let topic_exists = {
            let topics = self.topics.lock().unwrap();
            topics.contains(&event.topic)
        };

        if !topic_exists {
            self.create_topic(&event.topic).await?;
        }

        // Convert to envelope
        let payload = match serde_json::to_value(&event.payload) {
            Ok(value) => value,
            Err(e) => return Err(MockEventError::SerializationError(e.to_string())),
        };

        let envelope = EventEnvelope {
            id: event.id,
            event_type: event.event_type,
            topic: event.topic,
            created_at: event.created_at,
            priority: event.priority,
            source: event.source,
            correlation_id: event.correlation_id,
            metadata: event.metadata,
            payload,
        };

        // Store the event
        let mut events = self.events.lock().unwrap();
        events.push(envelope);

        Ok(DeliveryStatus::Delivered)
    }

    async fn subscribe<T: DeserializeOwned + Send + Sync + 'static>(
        &self,
        topic: &str,
    ) -> Result<(String, EventStream<T>)> {
        // Check if subscribing should fail
        if *self.fail_subscribing.lock().unwrap() {
            return Err(MockEventError::SubscribeError(format!(
                "Failed to subscribe to topic '{}'",
                topic
            )));
        }

        // Check if topic exists
        let topic_exists = {
            let topics = self.topics.lock().unwrap();
            topics.contains(topic)
        };

        if !topic_exists {
            return Err(MockEventError::TopicError(format!(
                "Topic '{}' not found",
                topic
            )));
        }

        // Create subscription
        let subscription_id = Uuid::new_v4().to_string();

        let mut subscriptions = self.subscriptions.lock().unwrap();
        subscriptions.push(SubscriptionInfo {
            id: subscription_id.clone(),
            topic: topic.to_string(),
            created_at: Utc::now(),
            events_delivered: 0,
        });

        // Clone relevant data for the stream
        let events = self.events.clone();
        let topic_name = topic.to_string();

        // Create a stream of existing events
        let stream = futures::stream::unfold((0, topic_name), move |(index, topic)| {
            let events_clone = events.clone();
            async move {
                let events = events_clone.lock().unwrap();
                let filtered_events: Vec<_> = events.iter().filter(|e| e.topic == topic).collect();

                if index < filtered_events.len() {
                    let envelope = &filtered_events[index];

                    // Convert envelope to typed event
                    let result = match serde_json::from_value(envelope.payload.clone()) {
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
                    };

                    Some((result, (index + 1, topic)))
                } else {
                    None
                }
            }
        });

        Ok((subscription_id, Box::pin(stream)))
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<bool> {
        let mut subscriptions = self.subscriptions.lock().unwrap();
        let initial_len = subscriptions.len();
        subscriptions.retain(|s| s.id != subscription_id);

        Ok(subscriptions.len() < initial_len)
    }

    async fn get_subscription_info(&self, subscription_id: &str) -> Result<SubscriptionInfo> {
        let subscriptions = self.subscriptions.lock().unwrap();
        subscriptions
            .iter()
            .find(|s| s.id == subscription_id)
            .cloned()
            .ok_or_else(|| {
                MockEventError::SubscriptionError(format!(
                    "Subscription '{}' not found",
                    subscription_id
                ))
            })
    }

    async fn list_subscriptions(&self) -> Result<Vec<SubscriptionInfo>> {
        let subscriptions = self.subscriptions.lock().unwrap();
        Ok(subscriptions.clone())
    }

    async fn get_recent_events(&self, topic: &str, limit: usize) -> Result<Vec<EventEnvelope>> {
        let events = self.events.lock().unwrap();
        let mut filtered: Vec<_> = events
            .iter()
            .filter(|e| e.topic == topic)
            .cloned()
            .collect();

        // Sort by created_at (newest first)
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply limit
        if filtered.len() > limit {
            filtered.truncate(limit);
        }

        Ok(filtered)
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
        let status = broker.publish(event).await.unwrap();
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
            .publish(Event::new(
                "test-event",
                "test-topic",
                "test-source",
                TestEvent {
                    message: "Hello, subscriber!".to_string(),
                    count: 1,
                },
            ))
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
        let event = Event::new(
            "test-event",
            "test-topic",
            "test-source",
            TestEvent {
                message: "This should fail".to_string(),
                count: 0,
            },
        );
        assert!(broker.publish(event).await.is_err());
    }
}
