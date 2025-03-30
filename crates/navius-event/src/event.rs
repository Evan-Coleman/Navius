//! Core event definitions for the event system

use std::any::Any;
use std::fmt::Debug;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority levels for events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Lowest priority, processed after all other events.
    Low = 0,
    /// Normal priority, processed after high priority events.
    Normal = 1,
    /// High priority, processed after critical events.
    High = 2,
    /// Highest priority, processed before all other events.
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

/// Metadata associated with an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique identifier for the event.
    pub id: Uuid,
    /// Time when the event was created.
    pub created_at: SystemTime,
    /// Type of the event.
    pub event_type: String,
    /// Topic the event belongs to.
    pub topic: String,
    /// Priority of the event.
    pub priority: EventPriority,
    /// Optional correlation ID for tracing related events.
    pub correlation_id: Option<Uuid>,
    /// Optional causation ID indicating which event caused this one.
    pub causation_id: Option<Uuid>,
    /// Optional time-to-live for the event.
    pub ttl: Option<Duration>,
    /// Additional metadata for the event.
    pub additional: serde_json::Value,
}

impl EventMetadata {
    /// Creates new event metadata with the given event type and topic.
    pub fn new<S: Into<String>>(event_type: S, topic: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: SystemTime::now(),
            event_type: event_type.into(),
            topic: topic.into(),
            priority: EventPriority::default(),
            correlation_id: None,
            causation_id: None,
            ttl: None,
            additional: serde_json::Value::Null,
        }
    }

    /// Sets the priority of the event.
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the correlation ID of the event.
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Sets the causation ID of the event.
    pub fn with_causation_id(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// Sets the time-to-live for the event.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Sets additional metadata for the event.
    pub fn with_additional(mut self, additional: serde_json::Value) -> Self {
        self.additional = additional;
        self
    }

    /// Checks if the event has expired based on its TTL.
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            if let Ok(elapsed) = self.created_at.elapsed() {
                return elapsed > ttl;
            }
        }
        false
    }
}

/// Base trait for all events in the system.
pub trait Event: Debug + Send + Sync + 'static {
    /// Returns the type name of the event.
    fn event_type(&self) -> &'static str;

    /// Returns the topic the event belongs to.
    fn topic(&self) -> &'static str;

    /// Returns the priority of the event.
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }

    /// Returns the event as a boxed Any trait object for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Creates a boxed clone of the event.
    fn clone_box(&self) -> Box<dyn Event>;
}

/// An envelope containing an event and its metadata.
#[derive(Debug)]
pub struct EventEnvelope {
    /// The event payload.
    pub payload: Box<dyn Event>,
    /// Metadata for the event.
    pub metadata: EventMetadata,
}

impl Clone for EventEnvelope {
    fn clone(&self) -> Self {
        Self {
            payload: self.payload.clone_box(),
            metadata: self.metadata.clone(),
        }
    }
}

impl EventEnvelope {
    /// Creates a new event envelope.
    pub fn new<E: Event>(event: E) -> Self {
        let metadata =
            EventMetadata::new(event.event_type().to_string(), event.topic().to_string())
                .with_priority(event.priority());

        Self {
            payload: Box::new(event),
            metadata,
        }
    }

    /// Creates a new event envelope with the given metadata.
    pub fn with_metadata<E: Event>(event: E, metadata: EventMetadata) -> Self {
        Self {
            payload: Box::new(event),
            metadata,
        }
    }

    /// Returns a reference to the event payload.
    pub fn payload(&self) -> &dyn Event {
        self.payload.as_ref()
    }

    /// Returns a reference to the event metadata.
    pub fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    /// Attempts to downcast the event payload to a specific type.
    pub fn downcast_payload<T: Event>(&self) -> Option<&T> {
        self.payload.as_any().downcast_ref::<T>()
    }

    /// Checks if the event has expired based on its TTL.
    pub fn is_expired(&self) -> bool {
        self.metadata.is_expired()
    }
}
