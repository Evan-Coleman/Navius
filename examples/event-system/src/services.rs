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

use crate::broker::{BrokerConfig, MessageBrokerAdapter, MessageBrokerFactory};
use crate::events::DomainEvent;
use crate::models::EventRecord;

/// Event Bus Service implementation
#[derive(Clone)]
pub struct EventBusService {
    handlers: Arc<Mutex<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_store: Arc<Mutex<Vec<EventRecord>>>,
    message_brokers: Arc<Mutex<Vec<Arc<dyn MessageBrokerAdapter>>>>,
}

impl EventBusService {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            event_store: Arc::new(Mutex::new(Vec::new())),
            message_brokers: Arc::new(Mutex::new(Vec::new())),
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

    /// Register an external message broker
    pub async fn register_message_broker(&self, config: BrokerConfig) -> Result<()> {
        // Create the broker adapter
        let broker = MessageBrokerFactory::create(config)?;

        // Connect to the broker
        broker.connect().await?;

        // Add the broker to the list
        {
            let mut brokers = self.message_brokers.lock().unwrap();
            brokers.push(broker);
        }

        info!("Registered and connected to external message broker");
        Ok(())
    }

    /// Get all registered message brokers
    pub fn get_message_brokers(&self) -> Vec<Arc<dyn MessageBrokerAdapter>> {
        let brokers = self.message_brokers.lock().unwrap();
        brokers.clone()
    }

    /// Publish an event to all connected external message brokers
    pub async fn publish_to_external_brokers<E>(&self, event: E, topic: &str) -> Result<()>
    where
        E: DomainEvent + Clone + Send + Sync + 'static,
    {
        let brokers = self.get_message_brokers();

        if brokers.is_empty() {
            return Ok(());
        }

        info!(
            "Publishing event to {} external message brokers",
            brokers.len()
        );

        let mut errors = Vec::new();

        for broker in brokers {
            match broker.publish_event(event.clone(), topic).await {
                Ok(_) => {
                    info!(
                        "Successfully published event to external broker: {:?}",
                        broker.broker_type()
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to publish event to external broker {:?}: {}",
                        broker.broker_type(),
                        e
                    );
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            Err(Error::new(&format!(
                "Failed to publish to {} out of {} external brokers",
                errors.len(),
                brokers.len()
            )))
        } else {
            Ok(())
        }
    }

    /// Subscribe to events from an external message broker
    pub async fn subscribe_to_external_broker(
        &self,
        broker_index: usize,
        topic: &str,
    ) -> Result<()> {
        let broker = {
            let brokers = self.message_brokers.lock().unwrap();
            if broker_index >= brokers.len() {
                return Err(Error::new("Invalid broker index"));
            }
            brokers[broker_index].clone()
        };

        broker.subscribe(topic, self.clone()).await
    }

    /// Process an event received from an external message broker
    pub async fn process_external_event(&self, event_type: &str, event_data: &str) -> Result<()> {
        info!("Processing external event: {}", event_type);

        // Execute all handlers registered for this event type
        let handlers = {
            let handlers_map = self.handlers.lock().unwrap();

            // First look for exact match
            if let Some(exact_handlers) = handlers_map.get(event_type) {
                exact_handlers.clone()
            } else {
                // Then look for wildcard handlers that might be registered for external events
                handlers_map
                    .get("ExternalEvent")
                    .cloned()
                    .unwrap_or_else(Vec::new)
            }
        };

        if handlers.is_empty() {
            info!("No handlers found for external event type: {}", event_type);
            return Ok(());
        }

        info!(
            "Executing {} handlers for external event: {}",
            handlers.len(),
            event_type
        );

        for handler in handlers {
            match handler.handle(event_type, event_data).await {
                Ok(_) => {
                    info!("Handler processed external event successfully");
                }
                Err(e) => {
                    error!("Error handling external event {}: {}", event_type, e);
                    // Continue processing other handlers despite errors
                }
            }
        }

        Ok(())
    }

    /// Check the health of all connected message brokers
    pub async fn check_broker_health(&self) -> Result<HashMap<String, bool>> {
        let brokers = self.get_message_brokers();
        let mut results = HashMap::new();

        for (index, broker) in brokers.iter().enumerate() {
            let health = broker.health_check().await?;
            results.insert(
                format!("broker-{}-{:?}", index, broker.broker_type()),
                health,
            );
        }

        Ok(results)
    }

    /// Disconnect from all message brokers
    pub async fn disconnect_all_brokers(&self) -> Result<()> {
        let brokers = self.get_message_brokers();

        for broker in brokers {
            if let Err(e) = broker.disconnect().await {
                error!("Error disconnecting from broker: {}", e);
                // Continue disconnecting other brokers despite errors
            }
        }

        // Clear the brokers list
        {
            let mut brokers_list = self.message_brokers.lock().unwrap();
            brokers_list.clear();
        }

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

        // Attempt to publish to external brokers if available
        // Note: we use a clone of the event to avoid ownership issues
        if let Err(e) = self.publish_to_external_brokers(event, &event_type).await {
            // Log but don't fail the overall publish operation if external broker publishing fails
            error!("Error publishing to external brokers: {}", e);
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
