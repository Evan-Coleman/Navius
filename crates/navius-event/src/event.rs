use crate::error::{EventPriority, EventResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

/// Represents an event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T> {
    /// Unique identifier for the event
    pub id: Uuid,

    /// Type of the event
    pub event_type: String,

    /// Topic the event was published to
    pub topic: String,

    /// Time the event was created
    pub created_at: DateTime<Utc>,

    /// Priority of the event
    pub priority: EventPriority,

    /// Source of the event (typically service or component name)
    pub source: String,

    /// Optional correlation ID for tracing related events
    pub correlation_id: Option<String>,

    /// Metadata associated with the event
    pub metadata: HashMap<String, String>,

    /// Payload of the event
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

    /// Create a new event with the specified priority
    pub fn with_priority(
        event_type: impl Into<String>,
        topic: impl Into<String>,
        source: impl Into<String>,
        payload: T,
        priority: EventPriority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            topic: topic.into(),
            created_at: Utc::now(),
            priority,
            source: source.into(),
            correlation_id: None,
            metadata: HashMap::new(),
            payload,
        }
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

    /// Add multiple metadata entries to the event
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
}

/// An envelope containing an event with its payload serialized to JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique identifier for the event
    pub id: Uuid,

    /// Type of the event
    pub event_type: String,

    /// Topic the event was published to
    pub topic: String,

    /// Time the event was created
    pub created_at: DateTime<Utc>,

    /// Priority of the event
    pub priority: EventPriority,

    /// Source of the event
    pub source: String,

    /// Optional correlation ID for tracing related events
    pub correlation_id: Option<String>,

    /// Metadata associated with the event
    pub metadata: HashMap<String, String>,

    /// JSON serialized payload
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Convert an event to an envelope
    pub fn from_event<T>(event: &Event<T>) -> EventResult<Self>
    where
        T: Serialize,
    {
        let payload = match serde_json::to_value(&event.payload) {
            Ok(value) => value,
            Err(err) => {
                return Err(crate::error::EventError::SerializationError(format!(
                    "Failed to serialize event payload: {}",
                    err
                )));
            }
        };

        Ok(Self {
            id: event.id,
            event_type: event.event_type.clone(),
            topic: event.topic.clone(),
            created_at: event.created_at,
            priority: event.priority,
            source: event.source.clone(),
            correlation_id: event.correlation_id.clone(),
            metadata: event.metadata.clone(),
            payload,
        })
    }

    /// Try to deserialize the payload into the specified type
    pub fn try_into_event<T>(&self) -> EventResult<Event<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let payload = match serde_json::from_value(self.payload.clone()) {
            Ok(value) => value,
            Err(err) => {
                return Err(crate::error::EventError::SerializationError(format!(
                    "Failed to deserialize event payload: {}",
                    err
                )));
            }
        };

        Ok(Event {
            id: self.id,
            event_type: self.event_type.clone(),
            topic: self.topic.clone(),
            created_at: self.created_at,
            priority: self.priority,
            source: self.source.clone(),
            correlation_id: self.correlation_id.clone(),
            metadata: self.metadata.clone(),
            payload,
        })
    }
}

/// Configuration for an event filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilterConfig {
    /// Filter events by type (if specified)
    pub event_types: Option<Vec<String>>,

    /// Filter events by source (if specified)
    pub sources: Option<Vec<String>>,

    /// Filter events by minimum priority (if specified)
    pub min_priority: Option<EventPriority>,

    /// Filter events by correlation ID (if specified)
    pub correlation_id: Option<String>,

    /// Filter events by metadata key-value pairs (if specified)
    pub metadata: Option<HashMap<String, String>>,
}

impl EventFilterConfig {
    /// Create a new empty event filter configuration
    pub fn new() -> Self {
        Self {
            event_types: None,
            sources: None,
            min_priority: None,
            correlation_id: None,
            metadata: None,
        }
    }

    /// Filter events by type
    pub fn with_event_types(mut self, event_types: Vec<impl Into<String>>) -> Self {
        self.event_types = Some(event_types.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Filter events by source
    pub fn with_sources(mut self, sources: Vec<impl Into<String>>) -> Self {
        self.sources = Some(sources.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Filter events by minimum priority
    pub fn with_min_priority(mut self, min_priority: EventPriority) -> Self {
        self.min_priority = Some(min_priority);
        self
    }

    /// Filter events by correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Filter events by metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }

        if let Some(metadata) = &mut self.metadata {
            metadata.insert(key.into(), value.into());
        }

        self
    }

    /// Apply the filter to an event envelope
    pub fn matches(&self, event: &EventEnvelope) -> bool {
        // Check event type filter
        if let Some(event_types) = &self.event_types {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }

        // Check source filter
        if let Some(sources) = &self.sources {
            if !sources.contains(&event.source) {
                return false;
            }
        }

        // Check priority filter
        if let Some(min_priority) = &self.min_priority {
            if event.priority < *min_priority {
                return false;
            }
        }

        // Check correlation ID filter
        if let Some(correlation_id) = &self.correlation_id {
            if let Some(event_correlation_id) = &event.correlation_id {
                if event_correlation_id != correlation_id {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check metadata filter
        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata {
                if !event.metadata.contains_key(key) || event.metadata.get(key).unwrap() != value {
                    return false;
                }
            }
        }

        true
    }
}

impl Default for EventFilterConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Subscription options
#[derive(Debug, Clone)]
pub struct SubscriptionOptions {
    /// Maximum number of events to buffer
    pub buffer_size: usize,

    /// Filter configuration
    pub filter: Option<EventFilterConfig>,

    /// Whether to deliver events from before the subscription was created
    pub deliver_historical_events: bool,

    /// Maximum number of delivery retries
    pub max_retries: u32,

    /// Subscription name (for logging/debugging)
    pub name: Option<String>,
}

impl SubscriptionOptions {
    /// Create new subscription options with default values
    pub fn new() -> Self {
        Self {
            buffer_size: 100,
            filter: None,
            deliver_historical_events: false,
            max_retries: 3,
            name: None,
        }
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    /// Set the filter configuration
    pub fn with_filter(mut self, filter: EventFilterConfig) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set whether to deliver historical events
    pub fn with_historical_events(mut self, deliver_historical_events: bool) -> Self {
        self.deliver_historical_events = deliver_historical_events;
        self
    }

    /// Set the maximum number of delivery retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the subscription name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about a subscription
#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    /// Unique identifier for the subscription
    pub id: String,

    /// Topic the subscription is for
    pub topic: String,

    /// Subscription options
    pub options: SubscriptionOptions,

    /// Time the subscription was created
    pub created_at: DateTime<Utc>,

    /// Number of events delivered through this subscription
    pub events_delivered: u64,

    /// Number of events dropped (e.g., due to buffer overflow)
    pub events_dropped: u64,
}
