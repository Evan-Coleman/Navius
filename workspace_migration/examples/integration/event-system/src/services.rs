//! Event System Services
//!
//! This file contains service implementations for the Event System integration example.

use async_trait::async_trait;
use navius_core::error::{Error, Result};
use navius_event::{EventBus, EventHandler, EventSubscriber};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{error, info};

use crate::events::DomainEvent;
use crate::models::EventRecord;

/// Event Bus Service implementation
#[derive(Clone)]
pub struct EventBusService {
    handlers: Arc<Mutex<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_store: Arc<Mutex<Vec<EventRecord>>>,
}

impl EventBusService {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            event_store: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_event_history(&self) -> Vec<EventRecord> {
        let store = self.event_store.lock().unwrap();
        store.clone()
    }

    pub fn record_event(&self, event_type: &str, payload: &str, source: &str) -> Result<()> {
        let record = EventRecord {
            id: uuid::Uuid::new_v4(),
            event_type: event_type.to_string(),
            payload: payload.to_string(),
            timestamp: chrono::Utc::now(),
            source: source.to_string(),
        };

        let mut store = self.event_store.lock().unwrap();
        store.push(record);
        Ok(())
    }
}

#[async_trait]
impl EventBus for EventBusService {
    async fn publish<E>(&self, event: E) -> Result<()>
    where
        E: DomainEvent + Send + Sync + 'static,
    {
        let event_type = event.event_type();
        let event_json = event.to_json()?;

        // Store the event for history
        self.record_event(&event_type, &event_json, "system")?;

        // Execute all handlers for this event type
        let handlers = {
            let handlers_map = self.handlers.lock().unwrap();
            handlers_map
                .get(&event_type)
                .cloned()
                .unwrap_or_else(Vec::new)
        };

        info!(
            "Publishing event: {} to {} handlers",
            event_type,
            handlers.len()
        );

        for handler in handlers {
            if let Err(e) = handler.handle(&event_type, &event_json).await {
                error!("Error handling event {}: {}", event_type, e);
                // Continue processing other handlers despite errors
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventSubscriber for EventBusService {
    fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>) -> Result<()> {
        let mut handlers_map = self.handlers.lock().unwrap();
        let handlers = handlers_map
            .entry(event_type.to_string())
            .or_insert_with(Vec::new);

        handlers.push(handler);
        info!("Subscribed handler to event type: {}", event_type);

        Ok(())
    }

    fn unsubscribe(&self, event_type: &str, handler_id: &str) -> Result<()> {
        let mut handlers_map = self.handlers.lock().unwrap();

        if let Some(handlers) = handlers_map.get_mut(event_type) {
            // Filter out the handler with the matching ID
            // In a real implementation, handler would have a unique ID
            // For this example, we'll assume handler_id is not used
            info!("Unsubscribed handler from event type: {}", event_type);
        }

        Ok(())
    }
}
