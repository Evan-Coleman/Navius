use crate::broker::{BrokerInfo, EventBroker, EventBrokerConfig, TopicInfo};
use crate::error::{DeliveryStatus, EventError, EventResult};
use crate::event::{
    Event, EventEnvelope, EventFilterConfig, SubscriptionInfo, SubscriptionOptions,
};
use async_trait::async_trait;
use chrono::Utc;
use futures::Stream;
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::stream::StreamExt;
use serde::{Serialize, de::DeserializeOwned};
use std::collections::{HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use uuid::Uuid;

/// In-memory topic data structure
struct InMemoryTopic {
    /// Name of the topic
    name: String,

    /// Stored events
    events: VecDeque<EventEnvelope>,

    /// Active subscriptions for this topic
    subscriptions: HashMap<String, SubscriptionState>,

    /// Total events published to this topic
    total_events: u64,
}

/// Subscription state
struct SubscriptionState {
    /// Subscription information
    info: SubscriptionInfo,

    /// Sender to the subscription stream
    sender: Sender<EventResult<EventEnvelope>>,
}

/// In-memory implementation of the event broker
pub struct InMemoryEventBroker {
    /// Broker ID
    id: String,

    /// Broker name
    name: String,

    /// Broker configuration
    config: EventBrokerConfig,

    /// Topics managed by this broker
    topics: Arc<RwLock<HashMap<String, Arc<Mutex<InMemoryTopic>>>>>,

    /// Subscriptions by subscription ID
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionInfo>>>,
}

impl InMemoryEventBroker {
    /// Create a new in-memory event broker
    pub fn new(config: EventBrokerConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "InMemoryEventBroker".to_string(),
            config,
            topics: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a background task to clean up old events
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let topics = self.topics.clone();
        let retention_seconds = self.config.event_retention_seconds;
        let max_events_per_topic = self.config.max_retained_events_per_topic;

        tokio::spawn(async move {
            if retention_seconds == 0 && max_events_per_topic == 0 {
                // No cleanup needed
                return;
            }

            let mut interval = time::interval(Duration::from_secs(60)); // Clean up every minute
            loop {
                interval.tick().await;

                // Get all topics
                let topic_map = topics.read().unwrap();
                let topic_keys: Vec<String> = topic_map.keys().cloned().collect();

                // Process each topic
                for topic_key in topic_keys {
                    if let Some(topic) = topic_map.get(&topic_key) {
                        let mut topic_lock = topic.lock().await;

                        // Clean up by retention time
                        if retention_seconds > 0 {
                            let cutoff = Utc::now()
                                .checked_sub_signed(chrono::Duration::seconds(
                                    retention_seconds as i64,
                                ))
                                .unwrap_or_else(Utc::now);

                            // Remove events older than cutoff
                            while let Some(event) = topic_lock.events.front() {
                                if event.created_at < cutoff {
                                    topic_lock.events.pop_front();
                                } else {
                                    break;
                                }
                            }
                        }

                        // Clean up by max events
                        if max_events_per_topic > 0 {
                            while topic_lock.events.len() > max_events_per_topic {
                                topic_lock.events.pop_front();
                            }
                        }
                    }
                }
            }
        })
    }

    /// Create a subscription stream for a specific type
    async fn create_subscription_stream<T>(
        &self,
        topic_name: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(
        String,
        Pin<Box<dyn Stream<Item = EventResult<Event<T>>> + Send>>,
    )>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        // Check if topic exists
        let topics_read = self.topics.read().unwrap();
        let topic = if let Some(topic) = topics_read.get(topic_name) {
            topic.clone()
        } else {
            // Auto-create topic if it doesn't exist
            drop(topics_read);
            self.create_topic(topic_name).await?;
            self.topics.read().unwrap().get(topic_name).unwrap().clone()
        };

        // Check subscriber limit
        if let Some(max_subscribers) = self.config.max_subscribers_per_topic {
            let topic_lock = topic.lock().await;
            if topic_lock.subscriptions.len() >= max_subscribers {
                return Err(EventError::MaxSubscribersReached(topic_name.to_string()));
            }
        }

        // Create subscription
        let (sender, receiver) = mpsc::channel(options.buffer_size);
        let subscription_id = Uuid::new_v4().to_string();

        // Create subscription info
        let subscription_info = SubscriptionInfo {
            id: subscription_id.clone(),
            topic: topic_name.to_string(),
            options: options.clone(),
            created_at: Utc::now(),
            events_delivered: 0,
            events_dropped: 0,
        };

        // Register subscription
        {
            let mut topic_lock = topic.lock().await;
            let subscription_state = SubscriptionState {
                info: subscription_info.clone(),
                sender,
            };
            topic_lock
                .subscriptions
                .insert(subscription_id.clone(), subscription_state);
        }

        // Register in global subscriptions map
        {
            let mut subs = self.subscriptions.write().unwrap();
            subs.insert(subscription_id.clone(), subscription_info);
        }

        // Send historical events if requested
        if options.deliver_historical_events {
            let topic_lock = topic.lock().await;
            for event in &topic_lock.events {
                if options.filter.as_ref().map_or(true, |f| f.matches(event)) {
                    if let Ok(typed_event) = event.try_into_event::<T>() {
                        if let Some(sender) = topic_lock
                            .subscriptions
                            .get(&subscription_id)
                            .map(|s| &s.sender)
                        {
                            // Ignore error if channel is full
                            let _ = sender.clone().try_send(Ok(typed_event));
                        }
                    }
                }
            }
        }

        // Create stream from receiver
        let stream = receiver.map(|envelope_result| {
            envelope_result.and_then(|envelope| {
                // Deserialize the envelope into the requested type
                envelope.try_into_event::<T>()
            })
        });

        Ok((subscription_id, Box::pin(stream)))
    }

    /// Create JSON subscription stream
    async fn create_json_subscription_stream(
        &self,
        topic_name: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(
        String,
        Pin<Box<dyn Stream<Item = EventResult<Event<serde_json::Value>>> + Send>>,
    )> {
        self.create_subscription_stream::<serde_json::Value>(topic_name, options)
            .await
    }

    /// Distribute an event to all subscribers
    async fn distribute_event(
        &self,
        topic: &Arc<Mutex<InMemoryTopic>>,
        envelope: EventEnvelope,
    ) -> EventResult<DeliveryStatus> {
        let mut topic_lock = topic.lock().await;

        // Store event
        if self.config.max_retained_events_per_topic > 0 {
            // Ensure we don't exceed max events
            while topic_lock.events.len() >= self.config.max_retained_events_per_topic {
                topic_lock.events.pop_front();
            }
            topic_lock.events.push_back(envelope.clone());
        }

        // Track total events
        topic_lock.total_events += 1;

        // Distribute to all subscribers
        let mut delivered = 0;
        let mut failed = 0;
        let total_subscribers = topic_lock.subscriptions.len();

        for (sub_id, sub) in &topic_lock.subscriptions {
            // Check if the event matches the filter
            if sub
                .info
                .options
                .filter
                .as_ref()
                .map_or(true, |f| f.matches(&envelope))
            {
                // Try to send the event
                match sub.sender.clone().try_send(Ok(envelope.clone())) {
                    Ok(_) => {
                        // Update delivery stats
                        delivered += 1;
                        if let Some(sub_info) = self.subscriptions.write().unwrap().get_mut(sub_id)
                        {
                            sub_info.events_delivered += 1;
                        }
                    }
                    Err(_) => {
                        // Failed to deliver (likely channel full)
                        failed += 1;
                        if let Some(sub_info) = self.subscriptions.write().unwrap().get_mut(sub_id)
                        {
                            sub_info.events_dropped += 1;
                        }
                    }
                }
            }
        }

        // Determine delivery status
        let status = if delivered == 0 && total_subscribers > 0 {
            DeliveryStatus::Failed
        } else if delivered < total_subscribers {
            DeliveryStatus::PartiallyDelivered
        } else {
            DeliveryStatus::Delivered
        };

        Ok(status)
    }
}

#[async_trait]
impl EventBroker for InMemoryEventBroker {
    async fn get_info(&self) -> EventResult<BrokerInfo> {
        let topics = self.topics.read().unwrap();
        let subscriptions = self.subscriptions.read().unwrap();

        Ok(BrokerInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            topic_count: topics.len(),
            subscription_count: subscriptions.len(),
            implementation_type: "InMemoryEventBroker".to_string(),
        })
    }

    async fn list_topics(&self) -> EventResult<Vec<TopicInfo>> {
        let topics = self.topics.read().unwrap();
        let mut result = Vec::with_capacity(topics.len());

        for (_, topic) in topics.iter() {
            let topic_lock = topic.lock().await;
            result.push(TopicInfo {
                name: topic_lock.name.clone(),
                subscriber_count: topic_lock.subscriptions.len(),
                retained_event_count: topic_lock.events.len(),
                total_event_count: topic_lock.total_events,
            });
        }

        Ok(result)
    }

    async fn topic_exists(&self, topic: &str) -> EventResult<bool> {
        let topics = self.topics.read().unwrap();
        Ok(topics.contains_key(topic))
    }

    async fn create_topic(&self, topic: &str) -> EventResult<()> {
        // Validate topic name
        if topic.is_empty() {
            return Err(EventError::InvalidTopic(
                "Topic name cannot be empty".to_string(),
            ));
        }

        // Check if this broker is configured to validate topics
        if self.config.validate_topics && !self.config.known_topics.contains(&topic.to_string()) {
            return Err(EventError::InvalidTopic(format!(
                "Topic '{}' is not in the list of known topics",
                topic
            )));
        }

        // Check max topics limit
        if let Some(max_topics) = self.config.max_topics {
            let topics = self.topics.read().unwrap();
            if topics.len() >= max_topics && !topics.contains_key(topic) {
                return Err(EventError::BrokerError(format!(
                    "Maximum number of topics ({}) reached",
                    max_topics
                )));
            }
        }

        // Create the topic if it doesn't exist
        let mut topics = self.topics.write().unwrap();
        if !topics.contains_key(topic) {
            topics.insert(
                topic.to_string(),
                Arc::new(Mutex::new(InMemoryTopic {
                    name: topic.to_string(),
                    events: VecDeque::new(),
                    subscriptions: HashMap::new(),
                    total_events: 0,
                })),
            );
        }

        Ok(())
    }

    async fn delete_topic(&self, topic: &str) -> EventResult<()> {
        // Check if the topic exists
        let topic_exists = {
            let topics = self.topics.read().unwrap();
            topics.contains_key(topic)
        };

        if !topic_exists {
            return Err(EventError::TopicNotFound(topic.to_string()));
        }

        // Remove all subscriptions for this topic first
        {
            let mut topics = self.topics.write().unwrap();
            if let Some(topic_mutex) = topics.get(topic).cloned() {
                let topic_lock = topic_mutex.lock().await;
                let subscription_ids: Vec<String> =
                    topic_lock.subscriptions.keys().cloned().collect();

                // Remove subscriptions from global map
                let mut subs = self.subscriptions.write().unwrap();
                for sub_id in subscription_ids {
                    subs.remove(&sub_id);
                }
            }

            // Remove the topic
            topics.remove(topic);
        }

        Ok(())
    }

    async fn publish<T>(&self, event: Event<T>) -> EventResult<DeliveryStatus>
    where
        T: Serialize + Send + Sync + 'static,
    {
        let topic_name = event.topic.clone();

        // Validate topic name
        if topic_name.is_empty() {
            return Err(EventError::InvalidTopic(
                "Topic name cannot be empty".to_string(),
            ));
        }

        // Convert to envelope
        let envelope = EventEnvelope::from_event(&event)?;

        // Get the topic or create it
        let topic = {
            let topics_read = self.topics.read().unwrap();
            if let Some(topic) = topics_read.get(&topic_name) {
                topic.clone()
            } else {
                // Auto-create the topic
                drop(topics_read);
                self.create_topic(&topic_name).await?;
                self.topics
                    .read()
                    .unwrap()
                    .get(&topic_name)
                    .unwrap()
                    .clone()
            }
        };

        // Distribute the event
        self.distribute_event(&topic, envelope).await
    }

    async fn publish_json(
        &self,
        event_type: &str,
        topic: &str,
        source: &str,
        payload: serde_json::Value,
    ) -> EventResult<DeliveryStatus> {
        let event = Event::new(event_type, topic, source, payload);
        self.publish(event).await
    }

    async fn subscribe<T>(
        &self,
        topic: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(String, super::broker::EventStream<T>)>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        self.create_subscription_stream(topic, options).await
    }

    async fn subscribe_json(
        &self,
        topic: &str,
        options: SubscriptionOptions,
    ) -> EventResult<(String, super::broker::EventStream<serde_json::Value>)> {
        self.create_json_subscription_stream(topic, options).await
    }

    async fn unsubscribe(&self, subscription_id: &str) -> EventResult<bool> {
        // Check if subscription exists
        let sub_info = {
            let subs = self.subscriptions.read().unwrap();
            match subs.get(subscription_id) {
                Some(sub) => sub.clone(),
                None => {
                    return Err(EventError::SubscriptionNotFound(
                        subscription_id.to_string(),
                    ));
                }
            }
        };

        // Remove subscription from topic
        let topics = self.topics.read().unwrap();
        if let Some(topic) = topics.get(&sub_info.topic) {
            let mut topic_lock = topic.lock().await;
            topic_lock.subscriptions.remove(subscription_id);
        }

        // Remove from global subscriptions map
        let mut subs = self.subscriptions.write().unwrap();
        subs.remove(subscription_id);

        Ok(true)
    }

    async fn get_subscription_info(&self, subscription_id: &str) -> EventResult<SubscriptionInfo> {
        let subs = self.subscriptions.read().unwrap();
        match subs.get(subscription_id) {
            Some(sub) => Ok(sub.clone()),
            None => Err(EventError::SubscriptionNotFound(
                subscription_id.to_string(),
            )),
        }
    }

    async fn list_subscriptions(&self) -> EventResult<Vec<SubscriptionInfo>> {
        let subs = self.subscriptions.read().unwrap();
        Ok(subs.values().cloned().collect())
    }

    async fn get_recent_events(
        &self,
        topic: &str,
        limit: usize,
    ) -> EventResult<Vec<EventEnvelope>> {
        let topics = self.topics.read().unwrap();
        match topics.get(topic) {
            Some(topic) => {
                let topic_lock = topic.lock().await;
                let events: Vec<EventEnvelope> = topic_lock
                    .events
                    .iter()
                    .rev() // Get most recent first
                    .take(limit)
                    .cloned()
                    .collect();
                Ok(events)
            }
            None => Err(EventError::TopicNotFound(topic.to_string())),
        }
    }

    async fn clear_events(&self, topic: &str) -> EventResult<usize> {
        let topics = self.topics.read().unwrap();
        match topics.get(topic) {
            Some(topic) => {
                let mut topic_lock = topic.lock().await;
                let count = topic_lock.events.len();
                topic_lock.events.clear();
                Ok(count)
            }
            None => Err(EventError::TopicNotFound(topic.to_string())),
        }
    }

    async fn get_event_by_id(&self, event_id: &str) -> EventResult<Option<EventEnvelope>> {
        let event_uuid = match Uuid::parse_str(event_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(EventError::Other(format!("Invalid event ID: {}", event_id))),
        };

        let topics = self.topics.read().unwrap();
        for (_, topic) in topics.iter() {
            let topic_lock = topic.lock().await;
            for event in &topic_lock.events {
                if event.id == event_uuid {
                    return Ok(Some(event.clone()));
                }
            }
        }

        Ok(None)
    }

    async fn filter_events(
        &self,
        topic: &str,
        filter: EventFilterConfig,
        limit: usize,
    ) -> EventResult<Vec<EventEnvelope>> {
        let topics = self.topics.read().unwrap();
        match topics.get(topic) {
            Some(topic) => {
                let topic_lock = topic.lock().await;
                let events: Vec<EventEnvelope> = topic_lock
                    .events
                    .iter()
                    .filter(|event| filter.matches(event))
                    .take(limit)
                    .cloned()
                    .collect();
                Ok(events)
            }
            None => Err(EventError::TopicNotFound(topic.to_string())),
        }
    }

    async fn health_check(&self) -> EventResult<bool> {
        // In-memory broker is always healthy as long as we can access it
        Ok(true)
    }
}

/// Factory for creating in-memory event brokers
pub struct InMemoryEventBrokerFactory {}

impl InMemoryEventBrokerFactory {
    /// Create a new in-memory event broker factory
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for InMemoryEventBrokerFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::broker::EventBrokerFactory for InMemoryEventBrokerFactory {
    async fn create_broker(&self, config: EventBrokerConfig) -> EventResult<Arc<dyn EventBroker>> {
        let broker = InMemoryEventBroker::new(config);
        let _cleanup_task = broker.start_cleanup_task();
        Ok(Arc::new(broker))
    }
}
