use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{DeliveryMode, MessagingError, MessagingResult};
use crate::serialization::Serializer;

/// Unique identifier for messages
pub type MessageId = String;

/// A generic message representation for the messaging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T> {
    /// Unique message identifier
    pub id: MessageId,

    /// Topic/routing key for the message
    pub topic: String,

    /// The actual message payload
    pub payload: T,

    /// Message headers/properties
    pub headers: MessageHeaders,

    /// Timestamp when the message was created
    pub timestamp: SystemTime,

    /// Expiration time for the message (None means no expiration)
    pub expiration: Option<SystemTime>,

    /// Message priority (higher means more important)
    pub priority: Option<u8>,

    /// Delivery mode for the message
    pub delivery_mode: DeliveryMode,

    /// Correlation ID for tracking related messages
    pub correlation_id: Option<String>,

    /// ID of the message this one is replying to
    pub reply_to_id: Option<String>,

    /// Queue/topic to send replies to
    pub reply_to: Option<String>,
}

impl<T: Serialize + Clone> Message<T> {
    /// Create a new message with the given payload and topic
    pub fn new(payload: T, topic: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            topic: topic.into(),
            payload,
            headers: MessageHeaders::default(),
            timestamp: SystemTime::now(),
            expiration: None,
            priority: None,
            delivery_mode: DeliveryMode::Persistent,
            correlation_id: None,
            reply_to_id: None,
            reply_to: None,
        }
    }

    /// Set the message ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the topic/routing key
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = topic.into();
        self
    }

    /// Add a message header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key, value);
        self
    }

    /// Add multiple headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set message expiration
    pub fn with_expiration(mut self, expiration: SystemTime) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Set message expiration from a duration from now
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.expiration = Some(SystemTime::now() + ttl);
        self
    }

    /// Set message priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Set delivery mode
    pub fn with_delivery_mode(mut self, mode: DeliveryMode) -> Self {
        self.delivery_mode = mode;
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set reply-to information
    pub fn with_reply_to(
        mut self,
        reply_to: impl Into<String>,
        reply_to_id: Option<impl Into<String>>,
    ) -> Self {
        self.reply_to = Some(reply_to.into());
        self.reply_to_id = reply_to_id.map(|id| id.into());
        self
    }

    /// Check if the message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            match SystemTime::now().duration_since(expiration) {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Get time remaining until expiration
    pub fn time_to_expiration(&self) -> Option<Duration> {
        self.expiration
            .and_then(|exp| exp.duration_since(SystemTime::now()).ok())
    }

    /// Convert this message to a message with a different payload type
    pub fn convert<U: DeserializeOwned>(
        &self,
        serializer: &Serializer,
    ) -> MessagingResult<Message<U>> {
        let bytes = serializer.serialize_value(&self.payload)?;
        let new_payload = serializer.deserialize_value(&bytes)?;

        Ok(Message {
            id: self.id.clone(),
            topic: self.topic.clone(),
            payload: new_payload,
            headers: self.headers.clone(),
            timestamp: self.timestamp,
            expiration: self.expiration,
            priority: self.priority,
            delivery_mode: self.delivery_mode,
            correlation_id: self.correlation_id.clone(),
            reply_to_id: self.reply_to_id.clone(),
            reply_to: self.reply_to.clone(),
        })
    }

    /// Create a reply message
    pub fn create_reply<R>(&self, reply_payload: R) -> Message<R>
    where
        R: Clone,
    {
        let reply_topic = self.reply_to.clone().unwrap_or_else(|| self.topic.clone());

        Message {
            id: Uuid::new_v4().to_string(),
            topic: reply_topic,
            payload: reply_payload,
            headers: MessageHeaders::default(),
            timestamp: SystemTime::now(),
            expiration: None,
            priority: self.priority,
            delivery_mode: self.delivery_mode,
            correlation_id: self.correlation_id.clone(),
            reply_to_id: Some(self.id.clone()),
            reply_to: None,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Message<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Message[id={}, topic={}, payload={}, priority={:?}, delivery_mode={}, correlation_id={:?}]",
            self.id,
            self.topic,
            self.payload,
            self.priority,
            self.delivery_mode,
            self.correlation_id
        )
    }
}

/// Headers/properties for messages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageHeaders {
    headers: HashMap<String, String>,
}

impl MessageHeaders {
    /// Create new empty headers
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// Insert a header
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

    /// Remove a header
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.headers.remove(key)
    }

    /// Check if headers contain a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }

    /// Get all headers
    pub fn all(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Extend with another map of headers
    pub fn extend<K, V>(&mut self, headers: HashMap<K, V>)
    where
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in headers {
            self.headers.insert(k.into(), v.into());
        }
    }

    /// Clear all headers
    pub fn clear(&mut self) {
        self.headers.clear();
    }

    /// Get number of headers
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Check if there are no headers
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}

/// A received message with metadata about the delivery
#[derive(Debug, Clone)]
pub struct ReceivedMessage<T> {
    /// The message content
    pub message: Message<T>,

    /// Delivery tag assigned by broker
    pub delivery_tag: u64,

    /// Whether the message was redelivered
    pub redelivered: bool,

    /// Exchange the message came from
    pub exchange: String,

    /// Routing key used for delivery
    pub routing_key: String,

    /// Consumer tag that received the message
    pub consumer_tag: String,
}

impl<T: Serialize + Clone> ReceivedMessage<T> {
    /// Create a new received message
    pub fn new(
        message: Message<T>,
        delivery_tag: u64,
        redelivered: bool,
        exchange: impl Into<String>,
        routing_key: impl Into<String>,
        consumer_tag: impl Into<String>,
    ) -> Self {
        Self {
            message,
            delivery_tag,
            redelivered,
            exchange: exchange.into(),
            routing_key: routing_key.into(),
            consumer_tag: consumer_tag.into(),
        }
    }

    /// Get a reference to the inner message
    pub fn inner(&self) -> &Message<T> {
        &self.message
    }

    /// Convert this message to a message with a different payload type
    pub fn convert<U: DeserializeOwned>(
        &self,
        serializer: &Serializer,
    ) -> MessagingResult<ReceivedMessage<U>> {
        let converted_message = self.message.convert(serializer)?;

        Ok(ReceivedMessage {
            message: converted_message,
            consumer_tag: self.consumer_tag.clone(),
            delivery_tag: self.delivery_tag,
            redelivered: self.redelivered,
            exchange: self.exchange.clone(),
            routing_key: self.routing_key.clone(),
        })
    }
}

/// Acknowledgment for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageAcknowledgment {
    /// Acknowledge message successfully processed
    Ack,

    /// Reject and requeue message
    Reject {
        /// Whether to put the message back in the queue (true) or discard it (false)
        requeue: bool,
    },

    /// Negative acknowledgment
    Nack {
        /// Whether to put the message back in the queue (true) or discard it (false)
        requeue: bool,
        /// Whether to nack multiple messages (true) or just the current one (false)
        multiple: bool,
    },
}

/// Result of message processing
pub type MessageProcessingResult = Result<MessageAcknowledgment, MessagingError>;

/// Trait for message handlers
pub trait MessageHandler<T>: Send + Sync {
    /// Handle a received message
    fn handle(&self, message: &ReceivedMessage<T>) -> MessageProcessingResult;
}

impl<T, F> MessageHandler<T> for F
where
    F: Fn(&ReceivedMessage<T>) -> MessageProcessingResult + Send + Sync,
{
    fn handle(&self, message: &ReceivedMessage<T>) -> MessageProcessingResult {
        self(message)
    }
}

/// Trait for message filters
pub trait MessageFilter<T>: Send + Sync {
    /// Check if a message should be processed
    fn matches(&self, message: &Message<T>) -> bool;

    /// Clone the filter box
    fn clone_box(&self) -> Box<dyn MessageFilter<T> + Send + Sync>
    where
        Self: 'static;
}

// Implement clone_box for all types that implement Clone
impl<T, F> MessageFilter<T> for F
where
    F: Clone + Send + Sync + 'static,
    F: Fn(&Message<T>) -> bool,
{
    fn matches(&self, message: &Message<T>) -> bool {
        self(message)
    }

    fn clone_box(&self) -> Box<dyn MessageFilter<T> + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Filter messages by topic
#[derive(Debug, Clone)]
pub struct TopicFilter {
    topics: Vec<String>,
}

impl TopicFilter {
    /// Create a new topic filter
    pub fn new(topics: Vec<String>) -> Self {
        Self { topics }
    }

    /// Check if the given topic matches any of the filter topics
    pub fn matches_topic(&self, topic: &str) -> bool {
        if self.topics.is_empty() {
            return true; // Empty filter matches everything
        }

        self.topics
            .iter()
            .any(|pattern| match_topic_pattern(pattern, topic))
    }
}

impl<T> MessageFilter<T> for TopicFilter {
    fn matches(&self, message: &Message<T>) -> bool {
        self.matches_topic(&message.topic)
    }

    fn clone_box(&self) -> Box<dyn MessageFilter<T> + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Header-based message filter
#[derive(Debug, Clone)]
pub struct HeaderFilter {
    /// Required headers that must match
    required_headers: HashMap<String, String>,
}

impl HeaderFilter {
    /// Create a new header filter
    pub fn new(required_headers: HashMap<String, String>) -> Self {
        Self { required_headers }
    }
}

impl<T> MessageFilter<T> for HeaderFilter {
    fn matches(&self, message: &Message<T>) -> bool {
        for (key, value) in &self.required_headers {
            if let Some(header_value) = message.headers.get(key) {
                if header_value != value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn clone_box(&self) -> Box<dyn MessageFilter<T> + Send + Sync> {
        Box::new(self.clone())
    }
}

/// A filter that requires all of its filters to match
pub struct AndFilter<T: 'static> {
    filters: Vec<Box<dyn MessageFilter<T> + Send + Sync>>,
}

impl<T: 'static> Clone for AndFilter<T> {
    fn clone(&self) -> Self {
        let cloned_filters = self
            .filters
            .iter()
            .map(|filter| filter.clone_box())
            .collect();

        Self {
            filters: cloned_filters,
        }
    }
}

impl<T: 'static> std::fmt::Debug for AndFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndFilter")
            .field("filters", &format!("{} filters", self.filters.len()))
            .finish()
    }
}

impl<T: 'static> AndFilter<T> {
    /// Create a new AND filter
    pub fn new(filters: Vec<Box<dyn MessageFilter<T> + Send + Sync>>) -> Self {
        Self { filters }
    }
}

impl<T: 'static> MessageFilter<T> for AndFilter<T> {
    fn matches(&self, message: &Message<T>) -> bool {
        self.filters.iter().all(|filter| filter.matches(message))
    }

    fn clone_box(&self) -> Box<dyn MessageFilter<T> + Send + Sync> {
        Box::new(self.clone())
    }
}

/// A filter that requires any of its filters to match
pub struct OrFilter<T: 'static> {
    filters: Vec<Box<dyn MessageFilter<T> + Send + Sync>>,
}

impl<T: 'static> Clone for OrFilter<T> {
    fn clone(&self) -> Self {
        let cloned_filters = self
            .filters
            .iter()
            .map(|filter| filter.clone_box())
            .collect();

        Self {
            filters: cloned_filters,
        }
    }
}

impl<T: 'static> std::fmt::Debug for OrFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrFilter")
            .field("filters", &format!("{} filters", self.filters.len()))
            .finish()
    }
}

impl<T: 'static> OrFilter<T> {
    /// Create a new OR filter
    pub fn new(filters: Vec<Box<dyn MessageFilter<T> + Send + Sync>>) -> Self {
        Self { filters }
    }
}

impl<T: 'static> MessageFilter<T> for OrFilter<T> {
    fn matches(&self, message: &Message<T>) -> bool {
        self.filters.iter().any(|filter| filter.matches(message))
    }

    fn clone_box(&self) -> Box<dyn MessageFilter<T> + Send + Sync> {
        Box::new(self.clone())
    }
}

/// Match a routing pattern against a topic string
///
/// Pattern can include:
/// - `*` to match a single word
/// - `#` to match zero or more words
fn match_topic_pattern(pattern: &str, topic: &str) -> bool {
    // If pattern is empty or equal to topic, it's a match
    if pattern.is_empty() || pattern == topic {
        return true;
    }

    // Split the pattern and topic into segments
    let pattern_parts: Vec<&str> = pattern.split('.').collect();
    let topic_parts: Vec<&str> = topic.split('.').collect();

    // Special case: "#" matches any topic
    if pattern == "#" || pattern == "*" {
        return true;
    }

    // Recursive matching function
    fn matches_recursive(p_parts: &[&str], t_parts: &[&str]) -> bool {
        match (p_parts.first(), t_parts.first()) {
            // If both are empty, we've matched everything
            (None, None) => true,

            // If pattern is empty but topic isn't, no match
            (None, Some(_)) => false,

            // If topic is empty but pattern isn't, only match if pattern is #
            (Some(&"#"), None) => true,
            (Some(_), None) => false,

            // Match current segments
            (Some(&p), Some(&t)) => {
                match p {
                    // # matches zero or more segments
                    "#" => {
                        // # at the end matches everything remaining
                        if p_parts.len() == 1 {
                            return true;
                        }

                        // Try matching the rest of the pattern at different positions
                        for i in 0..=t_parts.len() {
                            if matches_recursive(&p_parts[1..], &t_parts[i..]) {
                                return true;
                            }
                        }
                        false
                    }

                    // * matches exactly one segment
                    "*" => matches_recursive(&p_parts[1..], &t_parts[1..]),

                    // Exact match required
                    _ if p == t => matches_recursive(&p_parts[1..], &t_parts[1..]),
                    _ => false,
                }
            }
        }
    }

    matches_recursive(&pattern_parts, &topic_parts)
}
