//! Event System Handlers
//!
//! This file contains the event handler implementations for the Event System integration example.

use async_trait::async_trait;
use navius_core::error::{Error, Result};
use navius_event::EventHandler;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{error, info, warn};

use crate::events::{
    DomainEvent, InventoryUpdatedEvent, OrderCreatedEvent, OrderShippedEvent, SystemAlertEvent,
};
use crate::models::{AlertLevel, EventRecord};
use crate::repository::EventRepository;
use crate::services::EventBusService;

/// Notification Handler - sends notifications for events
pub struct NotificationHandler {
    name: String,
}

impl NotificationHandler {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for NotificationHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        info!("[{}] Notification for event: {}", self.name, event_type);

        match event_type {
            "OrderCreated" => {
                let event: OrderCreatedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderCreatedEvent: {}", e))
                })?;

                info!(
                    "[{}] Order created notification: Order #{} with {} items for customer {}",
                    self.name,
                    event.order_id,
                    event.items.len(),
                    event.customer_id
                );
            }
            "OrderShipped" => {
                let event: OrderShippedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderShippedEvent: {}", e))
                })?;

                info!(
                    "[{}] Order shipped notification: Order #{} shipped via {} with tracking number {}",
                    self.name, event.order_id, event.shipping_method, event.tracking_number
                );
            }
            "SystemAlert" => {
                let event: SystemAlertEvent = serde_json::from_str(event_data)
                    .map_err(|e| Error::new(&format!("Failed to parse SystemAlertEvent: {}", e)))?;

                let level = match event.alert_level {
                    AlertLevel::Info => "INFO",
                    AlertLevel::Warning => "WARNING",
                    AlertLevel::Error => "ERROR",
                    AlertLevel::Critical => "CRITICAL",
                };

                info!(
                    "[{}] System alert notification: [{}] {} - Service: {}",
                    self.name, level, event.message, event.service
                );
            }
            _ => {
                info!(
                    "[{}] Generic notification for event: {}",
                    self.name, event_type
                );
            }
        }

        Ok(())
    }
}

/// Analytics Handler - records metrics about events
pub struct AnalyticsHandler {
    name: String,
    metrics: Arc<Mutex<HashMap<String, i32>>>,
}

impl AnalyticsHandler {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_metrics(&self) -> HashMap<String, i32> {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

#[async_trait]
impl EventHandler for AnalyticsHandler {
    async fn handle(&self, event_type: &str, _event_data: &str) -> Result<()> {
        let mut metrics = self.metrics.lock().unwrap();
        let count = metrics.entry(event_type.to_string()).or_insert(0);
        *count += 1;

        info!(
            "[{}] Analytics recorded for event: {} (count: {})",
            self.name, event_type, *count
        );

        Ok(())
    }
}

/// Inventory Handler - updates inventory based on order events
pub struct InventoryHandler {
    name: String,
    inventory: Arc<Mutex<HashMap<String, i32>>>,
    event_bus: EventBusService,
}

impl InventoryHandler {
    pub fn new(name: &str, event_bus: EventBusService) -> Self {
        // Initialize with some sample inventory
        let mut inventory = HashMap::new();
        inventory.insert("PROD-001".to_string(), 100);
        inventory.insert("PROD-002".to_string(), 50);
        inventory.insert("PROD-003".to_string(), 75);

        Self {
            name: name.to_string(),
            inventory: Arc::new(Mutex::new(inventory)),
            event_bus,
        }
    }

    pub fn get_inventory(&self) -> HashMap<String, i32> {
        let inventory = self.inventory.lock().unwrap();
        inventory.clone()
    }
}

#[async_trait]
impl EventHandler for InventoryHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        match event_type {
            "OrderCreated" => {
                let event: OrderCreatedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderCreatedEvent: {}", e))
                })?;

                info!(
                    "[{}] Processing inventory updates for order: {}",
                    self.name, event.order_id
                );

                let mut inventory = self.inventory.lock().unwrap();

                // Update inventory for each item in the order
                for item in &event.items {
                    if let Some(current_quantity) = inventory.get_mut(&item.product_id) {
                        let previous_quantity = *current_quantity;
                        *current_quantity = (*current_quantity - item.quantity).max(0);

                        info!(
                            "[{}] Updated inventory for product {}: {} -> {}",
                            self.name, item.product_id, previous_quantity, *current_quantity
                        );

                        // Publish inventory updated event
                        let inventory_event = InventoryUpdatedEvent::new(
                            &item.product_id,
                            previous_quantity,
                            *current_quantity,
                            "WAREHOUSE-001",
                            &format!("Order: {}", event.order_id),
                        );

                        // Drop the mutex before publishing to avoid deadlocks
                        drop(inventory);

                        if let Err(e) = self.event_bus.publish(inventory_event).await {
                            error!(
                                "[{}] Failed to publish inventory update event: {}",
                                self.name, e
                            );
                        }

                        // Re-acquire the mutex
                        inventory = self.inventory.lock().unwrap();
                    } else {
                        warn!(
                            "[{}] Product not found in inventory: {}",
                            self.name, item.product_id
                        );
                    }
                }

                // Check for low inventory and generate alerts if needed
                for (product_id, quantity) in inventory.iter() {
                    if *quantity < 10 {
                        // Generate a system alert for low inventory
                        let alert = SystemAlertEvent::new(
                            AlertLevel::Warning,
                            &format!(
                                "Low inventory for product {}: {} units remaining",
                                product_id, quantity
                            ),
                            "InventoryService",
                            true,
                        );

                        // Drop the mutex before publishing
                        drop(inventory);

                        if let Err(e) = self.event_bus.publish(alert).await {
                            error!(
                                "[{}] Failed to publish low inventory alert: {}",
                                self.name, e
                            );
                        }

                        break; // To avoid multiple alerts, we'll just trigger one for now
                    }
                }
            }
            _ => {
                // Other event types not handled by this handler
            }
        }

        Ok(())
    }
}

/// Audit Log Handler - persists events to the database
pub struct AuditLogHandler {
    name: String,
    event_repository: Arc<EventRepository>,
}

impl AuditLogHandler {
    pub fn new(name: &str, event_repository: Arc<EventRepository>) -> Self {
        Self {
            name: name.to_string(),
            event_repository,
        }
    }
}

#[async_trait]
impl EventHandler for AuditLogHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        info!(
            "[{}] Persisting event to audit log: {}",
            self.name, event_type
        );

        let record = EventRecord {
            id: uuid::Uuid::new_v4(),
            event_type: event_type.to_string(),
            payload: event_data.to_string(),
            timestamp: chrono::Utc::now(),
            source: "AuditLog".to_string(),
        };

        if let Err(e) = self.event_repository.save_event(&record).await {
            error!("[{}] Failed to persist event to database: {}", self.name, e);
            return Err(e);
        }

        info!("[{}] Event successfully persisted to audit log", self.name);
        Ok(())
    }
}
