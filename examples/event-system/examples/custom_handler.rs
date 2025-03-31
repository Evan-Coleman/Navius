//! Custom Event Handler Example
//!
//! This example demonstrates how to create and register a custom event handler
//! with the Event System integration.

use async_trait::async_trait;
use event_system::{
    events::{DomainEvent, OrderCreatedEvent, OrderShippedEvent},
    services::EventBusService,
};
use navius_core::{
    di::{ApplicationBuilder, Environment},
    error::{Error, Result},
};
use navius_event::EventHandler;
use std::sync::Arc;
use tracing::{error, info};

/// A custom event handler that sends SMS notifications
struct SmsNotificationHandler {
    name: String,
    phone_number: String,
}

impl SmsNotificationHandler {
    fn new(name: &str, phone_number: &str) -> Self {
        Self {
            name: name.to_string(),
            phone_number: phone_number.to_string(),
        }
    }

    fn send_sms(&self, message: &str) -> Result<()> {
        // In a real implementation, this would connect to an SMS service
        // For this example, we'll just log the message
        info!(
            "[{}] Sending SMS to {}: {}",
            self.name, self.phone_number, message
        );
        Ok(())
    }
}

#[async_trait]
impl EventHandler for SmsNotificationHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        match event_type {
            "OrderCreated" => {
                let event: OrderCreatedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderCreatedEvent: {}", e))
                })?;

                let message = format!(
                    "New order #{} created with {} items for a total of ${:.2}",
                    event.order_id,
                    event.items.len(),
                    event.total_amount
                );

                self.send_sms(&message)?;
            }
            "OrderShipped" => {
                let event: OrderShippedEvent = serde_json::from_str(event_data).map_err(|e| {
                    Error::new(&format!("Failed to parse OrderShippedEvent: {}", e))
                })?;

                let message = format!(
                    "Order #{} has shipped! Tracking: {}. Estimated delivery: {}",
                    event.order_id, event.tracking_number, event.estimated_delivery
                );

                self.send_sms(&message)?;
            }
            _ => {
                // Ignore other event types
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt::init();

    // Create application
    let app = event_system::create_event_system_app(
        "postgres://navius:navius_password@localhost:5433/navius_events",
    )
    .await?;
    let mut app = app.build();

    // Start the application
    app.start().await?;

    // Retrieve the event bus component
    let event_bus = app
        .get_component::<EventBusService>("eventBus")
        .expect("Event bus component not found");

    // Create and register our custom SMS handler
    let sms_handler = Arc::new(SmsNotificationHandler::new(
        "CustomerSmsService",
        "+1234567890",
    ));

    // Subscribe to events
    event_bus.subscribe("OrderCreated", sms_handler.clone())?;
    event_bus.subscribe("OrderShipped", sms_handler.clone())?;

    info!("Custom SMS handler registered");

    // Create and publish a test order event
    let items = vec![
        event_system::models::OrderItem {
            product_id: "PROD-001".to_string(),
            quantity: 2,
            price: 29.99,
        },
        event_system::models::OrderItem {
            product_id: "PROD-002".to_string(),
            quantity: 1,
            price: 49.99,
        },
    ];

    let total = items
        .iter()
        .map(|item| item.price * item.quantity as f64)
        .sum();
    let order_event = OrderCreatedEvent::new(uuid::Uuid::new_v4(), "CUST-123", items, total);

    info!("Publishing test order event");
    event_bus.publish(order_event).await?;

    // Keep the application running for a bit to allow event processing
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Shutdown
    app.stop().await?;

    info!("Example completed successfully");
    Ok(())
}
