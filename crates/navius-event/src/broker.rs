use crate::error::{DeliveryStatus, EventResult};
use crate::event::{
    Event, EventEnvelope, EventFilterConfig, SubscriptionInfo, SubscriptionOptions,
};
use crate::memory::InMemoryEventBroker;
use async_trait::async_trait;
use futures::Stream;
use serde::{Serialize, de::DeserializeOwned};
use std::pin::Pin;
use std::sync::Arc;

/// Configuration for an event broker
#[derive(Debug, Clone)]
pub struct EventBrokerConfig {
    /// Maximum number of topics
    pub max_topics: Option<usize>,

    /// Maximum subscribers per topic
    pub max_subscribers_per_topic: Option<usize>,

    /// Maximum event retention period in seconds (0 = no retention)
    pub event_retention_seconds: u64,

    /// Maximum events to retain per topic (0 = unlimited)
    pub max_retained_events_per_topic: usize,

    /// Whether to validate event topics against a known list
    pub validate_topics: bool,

    /// Known topics (if validation is enabled)
    pub known_topics: Vec<String>,

    /// Topic namespace prefix
    pub topic_namespace: Option<String>,
}

impl EventBrokerConfig {
    /// Create a new broker configuration with default values
    pub fn new() -> Self {
        Self {
            max_topics: None,
            max_subscribers_per_topic: None,
            event_retention_seconds: 3600, // 1 hour by default
            max_retained_events_per_topic: 1000,
            validate_topics: false,
            known_topics: Vec::new(),
            topic_namespace: None,
        }
    }

    /// Set the maximum number of topics
    pub fn with_max_topics(mut self, max_topics: usize) -> Self {
        self.max_topics = Some(max_topics);
        self
    }

    /// Set the maximum subscribers per topic
    pub fn with_max_subscribers_per_topic(mut self, max_subscribers: usize) -> Self {
        self.max_subscribers_per_topic = Some(max_subscribers);
        self
    }

    /// Set the event retention period in seconds
    pub fn with_event_retention_seconds(mut self, seconds: u64) -> Self {
        self.event_retention_seconds = seconds;
        self
    }

    /// Set the maximum retained events per topic
    pub fn with_max_retained_events(mut self, max_events: usize) -> Self {
        self.max_retained_events_per_topic = max_events;
        self
    }

    /// Enable topic validation and set known topics
    pub fn with_topic_validation(mut self, topics: Vec<impl Into<String>>) -> Self {
        self.validate_topics = true;
        self.known_topics = topics.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Set the topic namespace prefix
    pub fn with_topic_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.topic_namespace = Some(namespace.into());
        self
    }
}

impl Default for EventBrokerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about the broker
#[derive(Debug, Clone)]
pub struct BrokerInfo {
    /// Unique identifier for the broker
    pub id: String,

    /// Name of the broker
    pub name: String,

    /// Number of active topics
    pub topic_count: usize,

    /// Number of active subscriptions
    pub subscription_count: usize,

    /// Implementation type
    pub implementation_type: String,
}

/// Information about a topic
#[derive(Debug, Clone)]
pub struct TopicInfo {
    /// Name of the topic
    pub name: String,

    /// Number of active subscribers
    pub subscriber_count: usize,

    /// Number of retained events
    pub retained_event_count: usize,

    /// Total events published to this topic
    pub total_event_count: u64,
}

/// Type for a stream of events
pub type EventStream<T> = Pin<Box<dyn Stream<Item = EventResult<Event<T>>> + Send>>;

/// Core trait defining the event broker interface
#[async_trait]
pub trait EventBroker: Send + Sync {
    /// Get broker information
    async fn get_info(&self) -> EventResult<BrokerInfo>;

    /// Get information about all topics
    async fn list_topics(&self) -> EventResult<Vec<TopicInfo>>;

    /// Check if a topic exists
    async fn topic_exists(&self, topic: &str) -> EventResult<bool>;

    /// Create a new topic
    async fn create_topic(&self, topic: &str) -> EventResult<()>;

    /// Delete a topic and all its subscriptions
    async fn delete_topic(&self, topic: &str) -> EventResult<()>;

    /// Publish a typed event
    async fn publish<T>(&self, event: Event<T>) -> EventResult<DeliveryStatus>
    where
        T: Serialize + Send + Sync + 'static;

    /// Publish a JSON event
    async fn publish_json(
        &self,
        event_type: &str,
        topic: &str,
        source: &str,
        payload: serde_json::Value,
    ) -> EventResult<DeliveryStatus>;

    /// Subscribe to events on a topic with options
    async fn subscribe<T>(
        &self,
        topic: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(String, EventStream<T>)>
    where
        T: DeserializeOwned + Send + Sync + 'static;

    /// Subscribe to JSON events on a topic with options
    async fn subscribe_json(
        &self,
        topic: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(String, EventStream<serde_json::Value>)>;

    /// Unsubscribe from a topic
    async fn unsubscribe(&self, subscription_id: &str) -> EventResult<bool>;

    /// Get information about a subscription
    async fn get_subscription_info(&self, subscription_id: &str) -> EventResult<SubscriptionInfo>;

    /// Get information about all subscriptions
    async fn list_subscriptions(&self) -> EventResult<Vec<SubscriptionInfo>>;

    /// Get recent events for a topic
    async fn get_recent_events(&self, topic: &str, limit: usize)
    -> EventResult<Vec<EventEnvelope>>;

    /// Clear all events for a topic
    async fn clear_events(&self, topic: &str) -> EventResult<usize>;

    /// Get an event by ID
    async fn get_event_by_id(&self, event_id: &str) -> EventResult<Option<EventEnvelope>>;

    /// Apply a filter to recent events
    async fn filter_events(
        &self,
        topic: &str,
        filter: EventFilterConfig,
        limit: usize,
    ) -> EventResult<Vec<EventEnvelope>>;

    /// Check broker health
    async fn health_check(&self) -> EventResult<bool>;
}

/// Factory for creating event brokers
#[async_trait]
pub trait EventBrokerFactory: Send + Sync {
    /// Create a new event broker
    async fn create_broker(
        &self,
        config: EventBrokerConfig,
    ) -> EventResult<Arc<InMemoryEventBroker>>;
}
