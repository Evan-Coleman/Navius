//! Event handler definitions and subscription management

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::EventResult;
use crate::event::{Event, EventEnvelope, EventPriority};

/// Type alias for handler identifiers.
pub type HandlerId = Uuid;

/// Type for event filtering functions.
pub type EventFilter = Box<dyn Fn(&EventEnvelope) -> bool + Send + Sync>;

/// Configuration for an event subscription.
pub struct EventSubscription {
    /// Unique identifier for the subscription.
    pub id: Uuid,
    /// Optional name for the subscription.
    pub name: Option<String>,
    /// Topic to subscribe to.
    pub topic: String,
    /// Optional filter for events.
    pub filter: Option<EventFilter>,
    /// Priority for handling the event.
    pub priority: EventPriority,
}

// Manual implementation of Debug for EventSubscription that skips the filter field
impl std::fmt::Debug for EventSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSubscription")
            .field("topic", &self.topic)
            .field(
                "filter",
                &if self.filter.is_some() {
                    "Some(..)"
                } else {
                    "None"
                },
            )
            .field("priority", &self.priority)
            .finish()
    }
}

impl EventSubscription {
    /// Creates a new subscription to the given topic.
    pub fn new<S: Into<String>>(topic: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: None,
            topic: topic.into(),
            filter: None,
            priority: EventPriority::default(),
        }
    }

    /// Sets a name for the subscription.
    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets a filter for the subscription.
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&EventEnvelope) -> bool + Send + Sync + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Checks if an event matches this subscription.
    pub fn matches(&self, envelope: &EventEnvelope) -> bool {
        // Check if the topic matches (exact match or wildcard)
        let topic_matches = self.topic == "*" || self.topic == envelope.metadata.topic;

        // If there's a filter, apply it
        let filter_matches = match &self.filter {
            Some(filter) => filter(envelope),
            None => true,
        };

        topic_matches && filter_matches
    }
}

/// Trait for handling events.
#[async_trait]
pub trait EventHandler: Send + Sync + Debug + 'static {
    /// Returns the subscriptions for this handler.
    fn subscriptions(&self) -> Vec<EventSubscription>;

    /// Handles an event.
    async fn handle(&self, event: EventEnvelope) -> EventResult<()>;
}

/// A wrapped event handler with its subscriptions.
#[derive(Debug)]
pub struct HandlerRegistration {
    /// The event handler.
    pub handler: Arc<dyn EventHandler>,
    /// The handler's subscriptions.
    pub subscriptions: Vec<EventSubscription>,
}

impl HandlerRegistration {
    /// Creates a new handler registration.
    pub fn new(handler: Arc<dyn EventHandler>) -> Self {
        let subscriptions = handler.subscriptions();
        Self {
            handler,
            subscriptions,
        }
    }

    /// Checks if any of the handler's subscriptions match the given event.
    pub fn matches(&self, envelope: &EventEnvelope) -> bool {
        self.subscriptions.iter().any(|sub| sub.matches(envelope))
    }
}
