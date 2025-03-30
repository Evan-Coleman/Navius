//! Event bus for publishing and subscribing to events

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::task::JoinSet;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::error::{EventError, EventResult};
use crate::event::{Event, EventEnvelope};
use crate::handler::{EventHandler, HandlerId};

/// A simple event bus implementation that directly dispatches events to handlers.
#[derive(Clone)]
pub struct EventBus {
    /// Registered event handlers.
    handlers: Arc<Mutex<HashMap<HandlerId, Arc<dyn EventHandler>>>>,
    /// Whether the bus is running.
    running: Arc<Mutex<bool>>,
}

impl EventBus {
    /// Creates a new event bus.
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(true)),
        }
    }

    /// Publishes an event to the bus.
    #[instrument(skip(self, event), fields(event_type = std::any::type_name::<E>()))]
    pub async fn publish<E: Event>(&self, event: E) -> EventResult<()> {
        // Check if the bus is running
        if !*self.running.lock().unwrap() {
            return Err(EventError::BusShutdown);
        }

        // Create an envelope for the event
        let envelope = EventEnvelope::new(event);

        debug!(
            event_id = %envelope.metadata.id,
            event_topic = %envelope.metadata.topic,
            "Publishing event"
        );

        // Get a copy of the current handlers
        let handlers = {
            let lock = self.handlers.lock().unwrap();
            lock.values().cloned().collect::<Vec<_>>()
        };

        if handlers.is_empty() {
            debug!("No handlers registered, skipping event delivery");
            return Ok(());
        }

        debug!("Delivering event to {} handlers", handlers.len());

        // Process the event with all matching handlers
        let mut tasks = JoinSet::new();

        for handler in handlers {
            // Check if the handler should receive this event
            // Get subscriptions for the handler
            let subscriptions = handler.subscriptions();

            // Check if any subscription matches the event topic
            let has_matching_subscription = subscriptions.iter().any(|sub| {
                // Check if the topic matches (exact match or wildcard)
                let topic_matches = sub.topic == "*" || sub.topic == envelope.metadata.topic;

                // If there's a filter, apply it
                if topic_matches {
                    if let Some(filter) = &sub.filter {
                        filter(&envelope)
                    } else {
                        true
                    }
                } else {
                    false
                }
            });

            if !has_matching_subscription {
                continue;
            }

            let event_clone = EventEnvelope {
                payload: envelope.payload.clone_box(),
                metadata: envelope.metadata.clone(),
            };

            tasks.spawn(async move {
                if let Err(e) = handler.handle(event_clone).await {
                    warn!("Handler failed: {}", e);
                }
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                warn!("Handler task failed: {}", e);
            }
        }

        Ok(())
    }

    /// Subscribes a handler to events.
    #[instrument(skip(self, handler), fields(handler_type = std::any::type_name::<H>()))]
    pub async fn subscribe<H: EventHandler>(&self, handler: H) -> EventResult<HandlerId> {
        // Check if the bus is running
        if !*self.running.lock().unwrap() {
            return Err(EventError::BusShutdown);
        }

        let id = Uuid::new_v4();
        debug!(
            handler_id = %id,
            handler_type = std::any::type_name::<H>(),
            "Subscribing handler to event bus"
        );

        // Register the handler
        self.handlers.lock().unwrap().insert(id, Arc::new(handler));

        Ok(id)
    }

    /// Unsubscribes a handler from events.
    #[instrument(skip(self))]
    pub async fn unsubscribe(&self, handler_id: HandlerId) -> EventResult<()> {
        // Check if the bus is running
        if !*self.running.lock().unwrap() {
            return Err(EventError::BusShutdown);
        }

        debug!(
            handler_id = %handler_id,
            "Unsubscribing handler from events"
        );

        if self.handlers.lock().unwrap().remove(&handler_id).is_none() {
            warn!(
                handler_id = %handler_id,
                "Handler not found"
            );
            return Err(EventError::HandlerNotFound);
        }

        Ok(())
    }

    /// Shuts down the event bus.
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> EventResult<()> {
        info!("Shutting down event bus");

        // Set running to false
        {
            let mut running = self.running.lock().unwrap();
            if !*running {
                debug!("Event bus already shutdown");
                return Ok(());
            }
            *running = false;
        }

        // Clear all handlers
        self.handlers.lock().unwrap().clear();

        debug!("Event bus shutdown completed");
        Ok(())
    }

    /// Checks if the event bus is running.
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
