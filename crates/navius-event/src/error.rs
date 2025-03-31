use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::io;

/// Result type for event operations
pub type EventResult<T> = Result<T, EventError>;

/// Represents errors that can occur in the event system
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    /// Error publishing an event
    #[error("Failed to publish event: {0}")]
    PublishError(String),

    /// Error subscribing to an event
    #[error("Failed to subscribe to event: {0}")]
    SubscribeError(String),

    /// Error unsubscribing from an event
    #[error("Failed to unsubscribe from event: {0}")]
    UnsubscribeError(String),

    /// Topic does not exist
    #[error("Topic '{0}' does not exist")]
    TopicNotFound(String),

    /// Invalid topic name
    #[error("Invalid topic name: {0}")]
    InvalidTopic(String),

    /// Subscription not found
    #[error("Subscription '{0}' not found")]
    SubscriptionNotFound(String),

    /// Too many subscriptions
    #[error("Too many subscriptions: {0}")]
    TooManySubscriptions(String),

    /// Error serializing or deserializing event payload
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error delivering an event
    #[error("Event delivery error: {0}")]
    DeliveryError(String),

    /// Maximum subscribers reached for topic
    #[error("Maximum subscribers reached for topic '{0}'")]
    MaxSubscribersReached(String),

    /// Event backpressure error (too many pending events)
    #[error("Event backpressure error: {0}")]
    BackpressureError(String),

    /// Timeout waiting for event
    #[error("Timeout waiting for event")]
    TimeoutError,

    /// Event filter error
    #[error("Event filter error: {0}")]
    FilterError(String),

    /// Event broker error
    #[error("Event broker error: {0}")]
    BrokerError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Other error
    #[error("Event error: {0}")]
    Other(String),
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority events
    Low = 0,

    /// Normal priority events (default)
    Normal = 1,

    /// High priority events
    High = 2,

    /// Critical priority events (processed before all others)
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Event delivery status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    /// Event was successfully delivered to all subscribers
    Delivered,

    /// Event was delivered to some subscribers but not all
    PartiallyDelivered,

    /// Event was queued for later delivery
    Queued,

    /// Event failed to be delivered
    Failed,
}

impl fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryStatus::Delivered => write!(f, "Delivered"),
            DeliveryStatus::PartiallyDelivered => write!(f, "PartiallyDelivered"),
            DeliveryStatus::Queued => write!(f, "Queued"),
            DeliveryStatus::Failed => write!(f, "Failed"),
        }
    }
}
