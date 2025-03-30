use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio::time::timeout;
use uuid::Uuid;

use crate::broker::MessageBroker;
use crate::error::{MessagingError, MessagingResult};
use crate::message::{Message, MessageId};

/// Helper for generating unique IDs
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Helper for generating correlation IDs
pub fn generate_correlation_id() -> String {
    format!("corr-{}", Uuid::new_v4())
}

/// Helper for generating consumer tags
pub fn generate_consumer_tag(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

/// Helper for creating a reply topic name
pub fn create_reply_topic(original_topic: &str, suffix: Option<&str>) -> String {
    let suffix = suffix.unwrap_or("reply");
    format!("{}.{}", original_topic, suffix)
}

/// Helper struct for request-reply pattern
pub struct RequestReply<T> {
    /// Broker to use for messaging
    broker: Arc<dyn MessageBroker>,

    /// Request exchange name
    request_exchange: String,

    /// Reply exchange name
    reply_exchange: String,

    /// Reply queue name
    reply_queue: String,

    /// Map of pending requests
    pending_requests: Arc<DashMap<MessageId, mpsc::Sender<T>>>,

    /// Default timeout for requests
    default_timeout: Duration,
}

impl<T: for<'de> Deserialize<'de> + Clone + Send + Sync + 'static> RequestReply<T> {
    /// Create a new request-reply helper
    pub async fn new(
        broker: Arc<dyn MessageBroker>,
        request_exchange: impl Into<String>,
        reply_exchange: impl Into<String>,
        default_timeout: Duration,
    ) -> MessagingResult<Self> {
        let request_exchange = request_exchange.into();
        let reply_exchange = reply_exchange.into();

        // Create a temporary reply queue
        let reply_queue = broker.create_reply_queue().await?;
        let reply_queue_name = reply_queue.name.clone();

        // Bind the reply queue to the reply exchange
        broker
            .bind_queue(&reply_queue_name, &reply_exchange, &reply_queue_name, None)
            .await?;

        let pending_requests = Arc::new(DashMap::new());
        let pending_requests_clone = pending_requests.clone();

        // Start a consumer for the reply queue
        let consumer_options = crate::consumer::ConsumerOptions::default()
            .with_exclusive(true)
            .with_auto_delete(true);

        broker
            .consume::<T>(&reply_queue_name, Some(consumer_options))
            .await?
            .for_each(move |result| {
                let pending_requests = pending_requests_clone.clone();
                async move {
                    match result {
                        Ok(received) => {
                            let message = received.message;

                            // Check if this is a reply to a pending request
                            if let Some(reply_to_id) = &message.reply_to_id {
                                if let Some((_, sender)) = pending_requests.remove(reply_to_id) {
                                    // Send the reply to the waiting task
                                    let _ = sender.send(message.payload).await;
                                }
                            }
                        }
                        Err(err) => {
                            // Log error but continue processing
                            eprintln!("Error receiving reply: {}", err);
                        }
                    }
                }
            });

        Ok(Self {
            broker,
            request_exchange,
            reply_exchange,
            reply_queue: reply_queue_name,
            pending_requests,
            default_timeout,
        })
    }

    /// Send a request and wait for a reply
    pub async fn request<R: Serialize + Send + Sync>(
        &self,
        request: &R,
        routing_key: &str,
        timeout_duration: Option<Duration>,
    ) -> MessagingResult<T> {
        // Create a channel for the reply
        let (tx, mut rx) = mpsc::channel(1);

        // Create a message with reply information
        let correlation_id = generate_correlation_id();
        let message_id = generate_id();

        let message = Message::new(request, routing_key)
            .with_id(message_id.clone())
            .with_correlation_id(correlation_id)
            .with_reply_to(self.reply_queue.clone(), None);

        // Store the request sender
        self.pending_requests.insert(message_id.clone(), tx);

        // Publish the request
        let options = crate::publisher::PublishOptions::new(&self.request_exchange)
            .with_routing_key(routing_key);

        self.broker.publish(&message, Some(options)).await?;

        // Wait for the reply with timeout
        let timeout_duration = timeout_duration.unwrap_or(self.default_timeout);
        match timeout(timeout_duration, rx.recv()).await {
            Ok(Some(reply)) => Ok(reply),
            Ok(None) => Err(MessagingError::TimeoutError(format!(
                "No reply channel for request {}",
                message_id
            ))),
            Err(_) => {
                // Remove the pending request on timeout
                self.pending_requests.remove(&message_id);
                Err(MessagingError::TimeoutError(format!(
                    "Timeout waiting for reply to request {}",
                    message_id
                )))
            }
        }
    }

    /// Get the reply queue name
    pub fn reply_queue(&self) -> &str {
        &self.reply_queue
    }

    /// Close the request-reply helper
    pub async fn close(&self) -> MessagingResult<()> {
        // Clear pending requests
        self.pending_requests.clear();

        // Delete the reply queue
        self.broker
            .delete_queue(&self.reply_queue, false, false)
            .await?;

        Ok(())
    }
}

/// A rate limiter for controlling message publishing rates
pub struct RateLimiter {
    /// Maximum number of messages per time window
    rate: u32,

    /// Time window in milliseconds
    window_ms: u64,

    /// Timestamps of recent messages
    timestamps: Arc<RwLock<Vec<SystemTime>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(rate: u32, window_ms: u64) -> Self {
        Self {
            rate,
            window_ms,
            timestamps: Arc::new(RwLock::new(Vec::with_capacity(rate as usize))),
        }
    }

    /// Check if a new message is allowed and update the state
    pub async fn check_and_update(&self) -> bool {
        let now = SystemTime::now();
        let window_start = now - Duration::from_millis(self.window_ms);

        let mut timestamps = self.timestamps.write().await;

        // Remove old timestamps
        timestamps.retain(|&ts| ts >= window_start);

        // Check if we're under the limit
        if timestamps.len() < self.rate as usize {
            timestamps.push(now);
            true
        } else {
            false
        }
    }

    /// Get the current rate (messages per second)
    pub async fn current_rate(&self) -> f64 {
        let now = SystemTime::now();
        let window_start = now - Duration::from_millis(self.window_ms);

        let timestamps = self.timestamps.read().await;
        let count = timestamps.iter().filter(|&&ts| ts >= window_start).count();

        count as f64 / (self.window_ms as f64 / 1000.0)
    }

    /// Reset the rate limiter
    pub async fn reset(&self) {
        let mut timestamps = self.timestamps.write().await;
        timestamps.clear();
    }
}

/// A message buffer for batching messages
pub struct MessageBuffer<T> {
    /// Maximum buffer size
    max_size: usize,

    /// Maximum buffer age before flush
    max_age: Duration,

    /// Buffered messages
    messages: Vec<T>,

    /// Timestamp of first message
    first_message_time: Option<SystemTime>,
}

impl<T> MessageBuffer<T> {
    /// Create a new message buffer
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        Self {
            max_size,
            max_age,
            messages: Vec::with_capacity(max_size),
            first_message_time: None,
        }
    }

    /// Add a message to the buffer
    pub fn add(&mut self, message: T) {
        if self.messages.is_empty() {
            self.first_message_time = Some(SystemTime::now());
        }

        self.messages.push(message);
    }

    /// Check if the buffer is ready to flush
    pub fn should_flush(&self) -> bool {
        // Flush if buffer is full
        if self.messages.len() >= self.max_size {
            return true;
        }

        // Flush if oldest message is too old
        if let Some(first_time) = self.first_message_time {
            if let Ok(age) = SystemTime::now().duration_since(first_time) {
                return age >= self.max_age;
            }
        }

        false
    }

    /// Get the messages and reset the buffer
    pub fn flush(&mut self) -> Vec<T> {
        let messages = std::mem::replace(&mut self.messages, Vec::with_capacity(self.max_size));
        self.first_message_time = None;
        messages
    }

    /// Get the number of messages in the buffer
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// A deduplication filter to prevent duplicate message processing
pub struct DeduplicationFilter {
    /// Recently seen message IDs
    seen_ids: Arc<RwLock<HashSet<String>>>,

    /// Time-to-live for entries
    ttl: Duration,

    /// Last cleanup time
    last_cleanup: Arc<RwLock<SystemTime>>,

    /// How often to clean up old entries
    cleanup_interval: Duration,
}

impl DeduplicationFilter {
    /// Create a new deduplication filter
    pub fn new(ttl: Duration, cleanup_interval: Duration) -> Self {
        Self {
            seen_ids: Arc::new(RwLock::new(HashSet::new())),
            ttl,
            last_cleanup: Arc::new(RwLock::new(SystemTime::now())),
            cleanup_interval,
        }
    }

    /// Check if a message is a duplicate
    pub async fn is_duplicate(&self, message_id: &str) -> bool {
        let seen_ids = self.seen_ids.read().await;
        seen_ids.contains(message_id)
    }

    /// Mark a message as seen
    pub async fn mark_seen(&self, message_id: String) {
        // Cleanup old entries if needed
        self.maybe_cleanup().await;

        // Add the message ID
        let mut seen_ids = self.seen_ids.write().await;
        seen_ids.insert(message_id);
    }

    /// Perform cleanup if necessary
    async fn maybe_cleanup(&self) {
        let now = SystemTime::now();

        // Check if we should clean up
        let should_cleanup = {
            let last_cleanup = self.last_cleanup.read().await;
            match now.duration_since(*last_cleanup) {
                Ok(duration) => duration >= self.cleanup_interval,
                Err(_) => false,
            }
        };

        if should_cleanup {
            self.cleanup().await;
        }
    }

    /// Clean up old entries
    async fn cleanup(&self) {
        let now = SystemTime::now();
        let cutoff = now - self.ttl;

        // We'd normally have timestamps per entry, but for simplicity we're just
        // clearing the whole set periodically
        let mut seen_ids = self.seen_ids.write().await;
        seen_ids.clear();

        // Update last cleanup time
        let mut last_cleanup = self.last_cleanup.write().await;
        *last_cleanup = now;
    }

    /// Reset the filter
    pub async fn reset(&self) {
        let mut seen_ids = self.seen_ids.write().await;
        seen_ids.clear();
    }
}

/// Header names for standard message headers
pub struct StandardHeaders;

impl StandardHeaders {
    /// Content type header
    pub const CONTENT_TYPE: &'static str = "content-type";

    /// Message ID header
    pub const MESSAGE_ID: &'static str = "message-id";

    /// Correlation ID header
    pub const CORRELATION_ID: &'static str = "correlation-id";

    /// Reply-to header
    pub const REPLY_TO: &'static str = "reply-to";

    /// Application ID header
    pub const APP_ID: &'static str = "app-id";

    /// User ID header
    pub const USER_ID: &'static str = "user-id";

    /// Type header
    pub const TYPE: &'static str = "type";

    /// Content encoding header
    pub const CONTENT_ENCODING: &'static str = "content-encoding";

    /// Expiration header
    pub const EXPIRATION: &'static str = "expiration";

    /// Timestamp header
    pub const TIMESTAMP: &'static str = "timestamp";
}

/// A message tracker for monitoring in-flight messages
pub struct MessageTracker {
    /// In-flight messages
    in_flight: Arc<DashMap<MessageId, MessageInfo>>,
}

impl MessageTracker {
    /// Create a new message tracker
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(DashMap::new()),
        }
    }

    /// Track a published message
    pub fn track_publish(&self, message_id: &str, topic: &str) {
        let now = SystemTime::now();
        self.in_flight.insert(
            message_id.to_string(),
            MessageInfo {
                message_id: message_id.to_string(),
                topic: topic.to_string(),
                publish_time: now,
                delivery_time: None,
                ack_time: None,
                status: MessageStatus::Published,
                properties: HashMap::new(),
            },
        );
    }

    /// Track message delivery
    pub fn track_delivery(&self, message_id: &str) {
        let now = SystemTime::now();
        if let Some(mut entry) = self.in_flight.get_mut(message_id) {
            entry.delivery_time = Some(now);
            entry.status = MessageStatus::Delivered;
        }
    }

    /// Track message acknowledgment
    pub fn track_ack(&self, message_id: &str) {
        let now = SystemTime::now();
        if let Some(mut entry) = self.in_flight.get_mut(message_id) {
            entry.ack_time = Some(now);
            entry.status = MessageStatus::Acknowledged;
        }
    }

    /// Get information about a message
    pub fn get_message_info(&self, message_id: &str) -> Option<MessageInfo> {
        self.in_flight.get(message_id).map(|entry| entry.clone())
    }

    /// Get all in-flight messages
    pub fn get_in_flight_messages(&self) -> Vec<MessageInfo> {
        self.in_flight
            .iter()
            .filter(|entry| {
                entry.status == MessageStatus::Published || entry.status == MessageStatus::Delivered
            })
            .map(|entry| entry.clone())
            .collect()
    }

    /// Remove a message from tracking
    pub fn remove(&self, message_id: &str) {
        self.in_flight.remove(message_id);
    }

    /// Clean up old completed messages
    pub fn cleanup_old(&self, max_age: Duration) {
        let now = SystemTime::now();
        let cutoff = now - max_age;

        self.in_flight.retain(|_, info| {
            // Keep if not acknowledged or if acknowledged recently
            info.status != MessageStatus::Acknowledged
                || info.ack_time.map(|t| t >= cutoff).unwrap_or(false)
        });
    }
}

/// Information about a tracked message
#[derive(Debug, Clone)]
pub struct MessageInfo {
    /// Message ID
    pub message_id: String,

    /// Topic/routing key
    pub topic: String,

    /// Time the message was published
    pub publish_time: SystemTime,

    /// Time the message was delivered
    pub delivery_time: Option<SystemTime>,

    /// Time the message was acknowledged
    pub ack_time: Option<SystemTime>,

    /// Current status of the message
    pub status: MessageStatus,

    /// Additional properties
    pub properties: HashMap<String, String>,
}

impl MessageInfo {
    /// Calculate the delivery latency
    pub fn delivery_latency(&self) -> Option<Duration> {
        self.delivery_time
            .and_then(|dt| dt.duration_since(self.publish_time).ok())
    }

    /// Calculate the processing time
    pub fn processing_time(&self) -> Option<Duration> {
        match (self.delivery_time, self.ack_time) {
            (Some(dt), Some(at)) => at.duration_since(dt).ok(),
            _ => None,
        }
    }

    /// Calculate the total latency
    pub fn total_latency(&self) -> Option<Duration> {
        self.ack_time
            .and_then(|at| at.duration_since(self.publish_time).ok())
    }
}

/// Status of a tracked message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    /// Message has been published but not delivered
    Published,

    /// Message has been delivered but not acknowledged
    Delivered,

    /// Message has been acknowledged
    Acknowledged,

    /// Message has been rejected
    Rejected,
}

impl fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageStatus::Published => write!(f, "published"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Acknowledged => write!(f, "acknowledged"),
            MessageStatus::Rejected => write!(f, "rejected"),
        }
    }
}
