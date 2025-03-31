//! External Message Broker Integration Example
//!
//! This example demonstrates how to integrate the Event System with external message
//! brokers like Kafka and RabbitMQ. It shows how to publish events to and
//! subscribe to events from external systems.

use async_trait::async_trait;
use chrono::Utc;
use event_system::{
    broker::{BrokerConfig, BrokerCredentials, BrokerType, MessageBrokerFactory},
    events::{DomainEvent, OrderCreatedEvent, SystemAlertEvent},
    handlers::NotificationHandler,
    models::{AlertLevel, OrderItem},
    services::EventBusService,
};
use navius_core::error::{Error, Result};
use navius_event::EventHandler;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

/// External Event Handler specifically for handling events from external systems
struct ExternalEventHandler {
    name: String,
    received_events: Arc<std::sync::Mutex<HashMap<String, usize>>>,
}

impl ExternalEventHandler {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            received_events: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn get_event_counts(&self) -> HashMap<String, usize> {
        let events = self.received_events.lock().unwrap();
        events.clone()
    }
}

#[async_trait]
impl EventHandler for ExternalEventHandler {
    async fn handle(&self, event_type: &str, event_data: &str) -> Result<()> {
        if !event_type.starts_with("External") {
            // Skip non-external events
            return Ok(());
        }

        info!("[{}] Processing external event: {}", self.name, event_type);

        // Parse the event_data based on its type
        match event_type {
            "ExternalOrderCreated" => {
                // Parse the external order data
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(event_data) {
                    let order_id = event
                        .get("order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let customer_id = event
                        .get("customer_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    info!(
                        "[{}] External order created: {} for customer {}",
                        self.name, order_id, customer_id
                    );
                } else {
                    warn!("[{}] Failed to parse external order data", self.name);
                }
            }
            "ExternalInventoryUpdate" => {
                // Parse the external inventory update
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(event_data) {
                    let product_id = event
                        .get("product_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let new_qty = event.get("new_qty").and_then(|v| v.as_i64()).unwrap_or(0);

                    info!(
                        "[{}] External inventory update: Product {} now has {} units",
                        self.name, product_id, new_qty
                    );
                } else {
                    warn!("[{}] Failed to parse external inventory data", self.name);
                }
            }
            "ExternalPaymentProcessed" => {
                // Parse the external payment data
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(event_data) {
                    let payment_id = event
                        .get("payment_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let amount = event.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let status = event
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    info!(
                        "[{}] External payment processed: {} for ${:.2} with status {}",
                        self.name, payment_id, amount, status
                    );
                } else {
                    warn!("[{}] Failed to parse external payment data", self.name);
                }
            }
            "ExternalShipmentUpdate" => {
                // Parse the external shipment data
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(event_data) {
                    let shipment_id = event
                        .get("shipment_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let status = event
                        .get("status")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let location = event
                        .get("location")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    info!(
                        "[{}] External shipment update: {} with status {} at {}",
                        self.name, shipment_id, status, location
                    );
                } else {
                    warn!("[{}] Failed to parse external shipment data", self.name);
                }
            }
            "ExternalSystemAlert" => {
                // Parse the external alert data
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(event_data) {
                    let level = event
                        .get("level")
                        .and_then(|v| v.as_str())
                        .unwrap_or("INFO");
                    let message = event
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No message");
                    let source = event
                        .get("source")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    info!(
                        "[{}] External system alert: [{}] {} from {}",
                        self.name, level, message, source
                    );
                } else {
                    warn!("[{}] Failed to parse external alert data", self.name);
                }
            }
            _ => {
                info!(
                    "[{}] Unrecognized external event type: {}",
                    self.name, event_type
                );
            }
        }

        // Count the event
        {
            let mut events = self.received_events.lock().unwrap();
            *events.entry(event_type.to_string()).or_insert(0) += 1;
        }

        Ok(())
    }
}

/// Custom payment event for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaymentProcessedEvent {
    #[serde(flatten)]
    metadata: event_system::models::EventMetadata,
    payment_id: String,
    order_id: String,
    amount: f64,
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl PaymentProcessedEvent {
    fn new(payment_id: &str, order_id: &str, amount: f64, status: &str) -> Self {
        Self {
            metadata: event_system::models::EventMetadata {
                event_id: Uuid::new_v4(),
                event_type: "PaymentProcessed".to_string(),
                timestamp: Utc::now(),
                source: "PaymentService".to_string(),
            },
            payment_id: payment_id.to_string(),
            order_id: order_id.to_string(),
            amount,
            status: status.to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl DomainEvent for PaymentProcessedEvent {
    fn event_type(&self) -> String {
        self.metadata.event_type.clone()
    }

    fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }

    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.metadata.timestamp
    }

    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Error::new(&format!("Serialization error: {}", e)))
    }
}

/// Publish sample events to show they reach external brokers
async fn publish_sample_events(event_bus: EventBusService) -> Result<()> {
    info!("Publishing sample events to demonstrate external broker integration");

    // Create a sample order
    let items = vec![
        OrderItem {
            product_id: "PROD-001".to_string(),
            quantity: 3,
            price: 29.99,
        },
        OrderItem {
            product_id: "PROD-002".to_string(),
            quantity: 1,
            price: 49.99,
        },
    ];

    let total = items
        .iter()
        .map(|item| item.price * item.quantity as f64)
        .sum();
    let order_event = OrderCreatedEvent::new(Uuid::new_v4(), "CUST-456", items, total);

    // Publish the order event
    info!("Publishing OrderCreatedEvent");
    event_bus.publish(order_event).await?;

    // Create and publish a payment event
    let payment_event = PaymentProcessedEvent::new(
        &format!("PAY-{}", Uuid::new_v4().to_string()[..8]),
        &format!("ORD-{}", Uuid::new_v4().to_string()[..8]),
        129.95,
        "completed",
    );

    info!("Publishing PaymentProcessedEvent");
    event_bus.publish(payment_event).await?;

    // Create and publish a system alert
    let alert_event = SystemAlertEvent::new(
        AlertLevel::Warning,
        "External integration test alert",
        "IntegrationTestService",
        false,
    );

    info!("Publishing SystemAlertEvent");
    event_bus.publish(alert_event).await?;

    Ok(())
}

/// Display broker status and received event counts
async fn display_status(
    event_bus: EventBusService,
    external_handler: Arc<ExternalEventHandler>,
    kafka_topic: &str,
    rabbitmq_queue: &str,
) -> Result<()> {
    info!("\n===== EXTERNAL BROKER STATUS =====");

    // Check broker health
    let health = event_bus.check_broker_health().await?;
    info!("Broker Health Status:");
    for (broker, status) in &health {
        info!(
            "  {} - Status: {}",
            broker,
            if *status { "Healthy" } else { "Unhealthy" }
        );
    }

    // Show event counts from external handler
    let event_counts = external_handler.get_event_counts();
    info!("\nReceived External Events:");
    if event_counts.is_empty() {
        info!("  No external events received yet");
    } else {
        for (event_type, count) in &event_counts {
            info!("  {} - Count: {}", event_type, count);
        }
    }

    // Show subscription info
    info!("\nActive Subscriptions:");
    info!("  Kafka Topic: {}", kafka_topic);
    info!("  RabbitMQ Queue: {}", rabbitmq_queue);

    info!("==================================\n");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logging
    tracing_subscriber::fmt::init();

    // Create the application
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

    info!("Setting up external message broker integration");

    // Create and register an external event handler
    let external_handler = Arc::new(ExternalEventHandler::new("ExternalEventHandler"));

    // Subscribe the handler to all external events using a wildcard
    event_bus.subscribe("ExternalEvent", external_handler.clone())?;
    event_bus.subscribe("ExternalOrderCreated", external_handler.clone())?;
    event_bus.subscribe("ExternalInventoryUpdate", external_handler.clone())?;
    event_bus.subscribe("ExternalSystemAlert", external_handler.clone())?;
    event_bus.subscribe("ExternalPaymentProcessed", external_handler.clone())?;
    event_bus.subscribe("ExternalShipmentUpdate", external_handler.clone())?;

    // Register a standard notification handler for comparison
    let notification_handler = Arc::new(NotificationHandler::new("StandardNotifier"));
    event_bus.subscribe("OrderCreated", notification_handler.clone())?;
    event_bus.subscribe("PaymentProcessed", notification_handler.clone())?;
    event_bus.subscribe("SystemAlert", notification_handler.clone())?;

    // Configure and register Kafka broker
    let kafka_config = BrokerConfig {
        broker_type: BrokerType::Kafka,
        connection_string: "kafka:9092".to_string(),
        client_id: "navius-event-system".to_string(),
        credentials: Some(BrokerCredentials {
            username: "kafka_user".to_string(),
            password: "kafka_password".to_string(),
            mechanism: "PLAIN".to_string(),
        }),
        options: serde_json::json!({
            "producer.batch.size": 16384,
            "socket.timeout.ms": 30000,
            "auto.offset.reset": "earliest"
        }),
    };

    info!("Registering Kafka broker");
    event_bus.register_message_broker(kafka_config).await?;

    // Configure and register RabbitMQ broker
    let rabbitmq_config = BrokerConfig {
        broker_type: BrokerType::RabbitMQ,
        connection_string: "amqp://localhost:5672".to_string(),
        client_id: "navius-event-system".to_string(),
        credentials: Some(BrokerCredentials {
            username: "guest".to_string(),
            password: "guest".to_string(),
            mechanism: "PLAIN".to_string(),
        }),
        options: serde_json::json!({
            "heartbeat": 30,
            "connection_timeout": 30000,
            "prefetch_count": 10
        }),
    };

    info!("Registering RabbitMQ broker");
    event_bus.register_message_broker(rabbitmq_config).await?;

    // Subscribe to topics/queues from external brokers
    // Kafka is at index 0, RabbitMQ at index 1
    let kafka_topic = "navius.events";
    let rabbitmq_queue = "navius.events.queue";

    info!("Subscribing to Kafka topic: {}", kafka_topic);
    event_bus
        .subscribe_to_external_broker(0, kafka_topic)
        .await?;

    info!("Subscribing to RabbitMQ queue: {}", rabbitmq_queue);
    event_bus
        .subscribe_to_external_broker(1, rabbitmq_queue)
        .await?;

    // Display initial status
    display_status(
        event_bus.clone(),
        external_handler.clone(),
        kafka_topic,
        rabbitmq_queue,
    )
    .await?;

    // Publish sample events to demonstrate integration
    publish_sample_events(event_bus.clone()).await?;

    // Wait to receive external events
    info!("Waiting for external events...");
    for i in 0..6 {
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Display status update every 5 seconds
        display_status(
            event_bus.clone(),
            external_handler.clone(),
            kafka_topic,
            rabbitmq_queue,
        )
        .await?;

        if i == 2 {
            // Publish more events after 15 seconds
            info!("Publishing additional events...");
            publish_sample_events(event_bus.clone()).await?;
        }
    }

    // Disconnect from brokers
    info!("Disconnecting from external message brokers");
    event_bus.disconnect_all_brokers().await?;

    // Shutdown
    app.stop().await?;

    info!("External message broker integration example completed successfully");
    Ok(())
}
