//! Event System Event Definitions
//!
//! This file contains the domain event definitions for the Event System integration example.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use navius_core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{AlertLevel, EventMetadata, OrderItem};

/// Base trait for all domain events
#[async_trait]
pub trait DomainEvent: Send + Sync {
    fn event_type(&self) -> String;
    fn event_id(&self) -> Uuid;
    fn timestamp(&self) -> DateTime<Utc>;
    fn to_json(&self) -> Result<String>;
}

/// Order Created Event
/// Triggered when a new order is created in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreatedEvent {
    pub metadata: EventMetadata,
    pub order_id: Uuid,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
}

impl OrderCreatedEvent {
    pub fn new(
        order_id: Uuid,
        customer_id: &str,
        items: Vec<OrderItem>,
        total_amount: f64,
    ) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "OrderCreated".to_string(),
                timestamp: Utc::now(),
                source: "OrderService".to_string(),
            },
            order_id,
            customer_id: customer_id.to_string(),
            items,
            total_amount,
        }
    }
}

impl DomainEvent for OrderCreatedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

/// Order Shipped Event
/// Triggered when an order is shipped to the customer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderShippedEvent {
    pub metadata: EventMetadata,
    pub order_id: Uuid,
    pub tracking_number: String,
    pub shipping_method: String,
    pub shipped_at: DateTime<Utc>,
    pub estimated_delivery: DateTime<Utc>,
}

impl OrderShippedEvent {
    pub fn new(order_id: Uuid, tracking_number: &str, shipping_method: &str) -> Self {
        let now = Utc::now();
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "OrderShipped".to_string(),
                timestamp: now,
                source: "ShippingService".to_string(),
            },
            order_id,
            tracking_number: tracking_number.to_string(),
            shipping_method: shipping_method.to_string(),
            shipped_at: now,
            estimated_delivery: now + chrono::Duration::days(3),
        }
    }
}

impl DomainEvent for OrderShippedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

/// Inventory Updated Event
/// Triggered when the inventory for a product is changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryUpdatedEvent {
    pub metadata: EventMetadata,
    pub product_id: String,
    pub previous_quantity: i32,
    pub new_quantity: i32,
    pub warehouse_id: String,
    pub reason: String,
}

impl InventoryUpdatedEvent {
    pub fn new(
        product_id: &str,
        previous_quantity: i32,
        new_quantity: i32,
        warehouse_id: &str,
        reason: &str,
    ) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "InventoryUpdated".to_string(),
                timestamp: Utc::now(),
                source: "InventoryService".to_string(),
            },
            product_id: product_id.to_string(),
            previous_quantity,
            new_quantity,
            warehouse_id: warehouse_id.to_string(),
            reason: reason.to_string(),
        }
    }
}

impl DomainEvent for InventoryUpdatedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

/// System Alert Event
/// Triggered when the system needs to notify about important conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlertEvent {
    pub metadata: EventMetadata,
    pub alert_level: AlertLevel,
    pub message: String,
    pub service: String,
    pub requires_action: bool,
}

impl SystemAlertEvent {
    pub fn new(
        alert_level: AlertLevel,
        message: &str,
        service: &str,
        requires_action: bool,
    ) -> Self {
        Self {
            metadata: EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "SystemAlert".to_string(),
                timestamp: Utc::now(),
                source: "MonitoringService".to_string(),
            },
            alert_level,
            message: message.to_string(),
            service: service.to_string(),
            requires_action,
        }
    }
}

impl DomainEvent for SystemAlertEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}
